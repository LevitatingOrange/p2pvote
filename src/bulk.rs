use crate::interactive::VoteMainActor;
use actix::prelude::*;
use log::error;
use pbr::ProgressBar;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::storage::*;
use kademlia::*;

#[derive(Serialize, Deserialize)]
struct Vote {
    key: u32,
    vote: usize,
}

pub fn bulk_voting<K: FindNodeCount, C: ConcurrenceCount>(
    key: KademliaKey,
    addr: SocketAddr,
    other_key: KademliaKey,
    other_addr: SocketAddr,
    bucket_size: usize,
    timeout: Duration,
    file: String,
) -> Result<(), std::io::Error> {
    System::run(move || {
        let storage_address = VoteStorageActor::create(move |_| VoteStorageActor::new());

        let controller_address = create_controller::<K, C, VoteStorageActor, VoteMainActor<K, C>>(
            key,
            other_key,
            addr,
            other_addr,
            bucket_size,
            timeout,
            timeout,
            storage_address,
            None,
        );
        thread::spawn(move || {
            if let Err(err) = send_votes(controller_address.clone(), &file) {
                error!("Error while storing bulk votes: {}", err);
            }
            controller_address.do_send(ControllerMessage::Shutdown);
            //System::current().stop();
        });
    })
}

fn send_votes<K: FindNodeCount, C: ConcurrenceCount>(
    controller: Addr<Controller<K, C, VoteStorageActor, VoteMainActor<K, C>>>,
    file: &str,
) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file)?;
    let mut votes = Vec::new();
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let vote: Vote = result?;
        votes.push(vote);
    }
    sleep(Duration::from_millis(500));
    let mut pb = ProgressBar::new(votes.len() as u64);
    pb.message("Sending bulk votes: ");
    for vote in votes {
        controller.try_send(ControllerMessage::StoreKey(
            KademliaKey(vote.key),
            vote.vote,
        ))?;
        sleep(Duration::from_millis(20));
        pb.inc();
    }

    pb.finish_println("Stored votes, shutting down...\n");
    Ok(())
}
