use crate::storage::VoteStorageActor;
use actix::prelude::*;
use kademlia::*;
use log::error;
use std::cmp::max;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use pbr::ProgressBar;

use log::info;

use std::sync::mpsc::{channel, Receiver, Sender};

use std::io::{stdin, stdout, Write};
use termion::clear;
use termion::cursor::*;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::style::*;

use rand::prelude::*;

use num::integer::gcd;

fn option_menu(
    msg: &str,
    options: &[&str],
    default: usize,
) -> Result<Option<usize>, std::io::Error> {
    let stdin = stdin();
    let s = HideCursor::from(stdout());
    let mut stdout = s.into_raw_mode()?;
    write!(stdout, "{}{}{}:", clear::All, Goto(1, 1), msg)?;
    for (i, option) in options.iter().enumerate() {
        //println!("{}", i);
        writeln!(stdout, "{}{}", Goto(3, 2 + (i as u16)), option)?;
    }
    writeln!(
        stdout,
        "{}Press q to cancel selection, Ctrl-C to quit.",
        Goto(1, 2 + (options.len() as u16))
    )?;
    let mut curr = default;
    write!(
        stdout,
        "{}> {}{}",
        Goto(1, 2 + curr as u16),
        Bold,
        options[curr]
    )?;
    stdout.flush()?;
    for c in stdin.keys() {
        match c? {
            Key::Char('q') | Key::Esc => {
                write!(stdout, "{}{}{}", clear::All, Goto(1, 1), Reset)?;
                stdout.flush()?;
                return Ok(None);
            }
            Key::Ctrl('c') => {
                write!(stdout, "{}{}{}", clear::All, Goto(1, 1), Reset)?;
                stdout.flush()?;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cancelled by user",
                ));
            }
            Key::Up | Key::PageUp => {
                if curr > 0 {
                    write!(stdout, "{}{}", Goto(1, 2 + curr as u16), clear::CurrentLine)?;
                    write!(
                        stdout,
                        "{}{}  {}",
                        Goto(1, 2 + curr as u16),
                        Reset,
                        options[curr]
                    )?;
                    curr -= 1;
                } else {
                    continue;
                }
            }
            Key::Down | Key::PageDown => {
                if curr < options.len() - 1 {
                    write!(
                        stdout,
                        "{}  {}{}",
                        Goto(1, 2 + curr as u16),
                        Reset,
                        options[curr]
                    )?;
                    curr += 1;
                } else {
                    continue;
                }
            }
            Key::Char('\n') => {
                break;
            }
            _ => {
                continue;
            }
        }
        write!(
            stdout,
            "{}> {}{}",
            Goto(1, 2 + curr as u16),
            Bold,
            options[curr]
        )?;
        stdout.flush()?;
    }
    write!(stdout, "{}{}{}", clear::All, Goto(1, 1), Reset)?;
    stdout.flush()?;
    //stdout.suspend_raw_mode()?;
    //stdout.flush()?;
    Ok(Some(curr))
}

fn tally_up(options: &[&str], results: &[usize]) -> Result<(), std::io::Error> {
    let max_width_label = options.iter().fold(0, |m, s| max(m, s.len()));
    let mut votes: Vec<(&str, usize)> = options.into_iter().map(|s| (*s, 0)).collect();
    for vote in results {
        if *vote >= options.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Vote {} is out of bounds", vote),
            ));
        }
        votes[*vote].1 += 1;
    }
    votes.sort_by(|(_, a), (_, b)| b.cmp(a));
    let max_width_votes = votes
        .iter()
        .fold(0, |m, (_, s)| max(m, s.to_string().len()));

    let divisor = votes.iter().fold(votes[0].1, |acc, (_, n)| gcd(acc, *n));

    let mut stdout = HideCursor::from(stdout());
    let stdin = stdin();

    for (label, count) in votes {
        writeln!(
            stdout,
            "{:3$}: {:4$} {:#<5$}",
            label,
            count,
            "│",
            max_width_label,
            max_width_votes,
            count / divisor + 1
        )?;
    }

    writeln!(stdout, "Press any key to continue...")?;
    stdout.flush()?;
    let mut stdout = stdout.into_raw_mode()?;
    for c in stdin.keys() {
        match c? {
            Key::Ctrl('c') => {
                write!(stdout, "{}{}{}", clear::All, Goto(1, 1), Reset)?;
                stdout.flush()?;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Cancelled by user",
                ));
            }
            _ => {
                break;
            }
        }
    }
    writeln!(stdout, "{}", Reset)?;
    stdout.flush()?;
    Ok(())
}

// pub fn interaction<K: FindNodeCount, C: ConcurrenceCount>(controller: Addr<Controller<K, C, VoteStorageActor, VoteMainActor>>,
// vote_mappings: Option<Vec<String>>) {
//     -> Result<(), Box<dyn Error>> {
//         Ok(())

// }

fn interaction_loop<K: FindNodeCount, C: ConcurrenceCount>(
    vote_mappings: Vec<String>,
    id_pool: Vec<u32>,
    rx: Receiver<ControllerResponse<VoteStorageActor>>,
    controller: Addr<Controller<K, C, VoteStorageActor, VoteMainActor<K, C>>>,
) -> Result<(), std::io::Error> {
    let mut rng = thread_rng();
    loop {
        let mappings: Vec<&str> = vote_mappings.iter().map(|d| d.as_ref()).collect();
        if let Some(sel) =
            option_menu("What do you want to do", &["Cast a vote", "Count votes"], 0)?
        {
            if sel == 0 {
                if let Some(v) = option_menu("Select your choice", &mappings, 0)? {
                    let id = KademliaKey(*id_pool.choose(&mut rng).unwrap());
                    controller.do_send(ControllerMessage::StoreKey(id, v));
                    println!("Cast vote \"{}\" with id {}!", vote_mappings[v], id);
                    sleep(Duration::from_secs(1));
                }
            } else {
                let mut ids = id_pool.clone();
                ids.shuffle(&mut rng);
                print!("How many votes should be tallied? ");
                stdout().flush()?;
                let mut x = String::new();
                stdin().read_line(&mut x)?;
                x.pop().unwrap();

                if let Ok(num) = x.parse::<usize>() {
                    let mut votes: HashMap<KademliaKey, usize> = HashMap::new();
                    let mut pb = ProgressBar::new(num as u64);
                    pb.message("Gathering votes: ");
                    while votes.len() < num && !ids.is_empty() {
                        controller
                            .do_send(ControllerMessage::Retrieve(KademliaKey(ids.pop().unwrap())));
                        let result = rx.recv().unwrap();
                        match result {
                            ControllerResponse::Found(id, d) => {
                                info!("Found vote \"{}\" for key {}", vote_mappings[d], id);
                                votes.insert(id, d);
                                pb.inc();
                            }
                            ControllerResponse::NotFound(id) => {
                                info!("No vote found for key {}", id);
                            }
                            _ => unreachable!(),
                        }
                    }
                    pb.finish_print("Done! Results are:");
                    let v: Vec<usize> = votes.values().map(|s| *s).collect();
                    tally_up(&mappings, &v)?;
                }
            }
        }
    }
}

pub fn interactive<K: FindNodeCount, C: ConcurrenceCount>(
    key: KademliaKey,
    addr: SocketAddr,
    other_key: KademliaKey,
    other_addr: SocketAddr,
    bucket_size: usize,
    timeout: Duration,
    vote_mappings: Vec<String>,
    id_pool: Vec<u32>,
) -> Result<(), std::io::Error> {
    System::run(move || {
        let storage_address = VoteStorageActor::create(move |_| VoteStorageActor::new());
        let main_address = VoteMainActor::create(move |_| VoteMainActor::new());

        let controller_address = create_controller::<K, C, VoteStorageActor, VoteMainActor<K, C>>(
            key,
            other_key,
            addr,
            other_addr,
            bucket_size,
            timeout,
            timeout,
            storage_address,
            Some(main_address.clone()),
        );
        main_address.do_send(MainMessage::Init(
            controller_address,
            vote_mappings,
            id_pool,
        ));
        //main_address.send()
    })
}

#[derive(Message)]
pub enum MainMessage<K: FindNodeCount, C: ConcurrenceCount> {
    Init(
        Addr<Controller<K, C, VoteStorageActor, VoteMainActor<K, C>>>,
        Vec<String>,
        Vec<u32>,
    ),
}

pub struct VoteMainActor<K: FindNodeCount, C: ConcurrenceCount> {
    tx: Option<Sender<ControllerResponse<VoteStorageActor>>>,
    p: std::marker::PhantomData<K>,
    p2: std::marker::PhantomData<C>,
}

impl<K: FindNodeCount, C: ConcurrenceCount> VoteMainActor<K, C> {
    pub fn new() -> Self {
        VoteMainActor {
            tx: None,
            p: std::marker::PhantomData::default(),
            p2: std::marker::PhantomData::default(),
        }
    }
}

impl<K: FindNodeCount, C: ConcurrenceCount> Actor for VoteMainActor<K, C> {
    type Context = actix::Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl<K: FindNodeCount, C: ConcurrenceCount> Handler<ControllerResponse<VoteStorageActor>>
    for VoteMainActor<K, C>
{
    type Result = ();
    fn handle(&mut self, msg: ControllerResponse<VoteStorageActor>, ctx: &mut Context<Self>) {
        info!("Main got response {:?}", msg);
        match msg {
            ControllerResponse::Shutdown => {
                ctx.stop();
                System::current().stop();
            }
            msg => {
                if let Some(tx) = &self.tx {
                    tx.send(msg).unwrap();
                }
            }
        }
    }
}

impl<K: FindNodeCount, C: ConcurrenceCount> Handler<MainMessage<K, C>> for VoteMainActor<K, C> {
    type Result = ();
    fn handle(&mut self, msg: MainMessage<K, C>, _: &mut Context<Self>) {
        match msg {
            MainMessage::Init(controller, vote_mappings, id_pool) => {
                let (tx, rx) = channel();
                self.tx = Some(tx);
                thread::spawn(move || {
                    if let Err(e) = interaction_loop(vote_mappings, id_pool, rx, controller.clone())
                    {
                        error!("Received io error {}", e);
                        println!("{}Bye!", Reset);
                        controller.do_send(ControllerMessage::Shutdown);
                    }
                });
            }
        }
    }
}
