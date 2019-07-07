use generic_array::typenum::{U20, U3};
use log::error;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;

use bulk::*;
use headless::*;
use interactive::*;
use kademlia::*;

mod bulk;
mod headless;
mod interactive;
mod storage;

#[derive(Serialize, Deserialize)]
struct PoolID(u32);

#[derive(Serialize, Deserialize)]
struct OptionName(String);

use clap::{App, Arg, SubCommand};

type NodeCount = U20;
type Concurrency = U3;

fn start() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("error")).init();

    let matches = App::new("A peer to peer voting application")
        .version("0.1")
        .author("Lennart V. <lennart@vogelsang.berlin>")
        .about("Stores votes in a p2p kademlia network and counts them")
        .arg(
            Arg::with_name("OUR_ADDRESS")
                .help("Our hostname:port pair")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUR_ID")
                .help("Our id")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("THEIR_ADDRESS")
                .help("The hostname:port pair of a peer in the network")
                .required(true)
                .index(3),
        )
        .arg(
            Arg::with_name("THEIR_ID")
                .help("The id of the peer")
                .required(true)
                .index(4),
        )
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .value_name("SECS")
                .help("Seconds until kademlia timeouts are triggered. Default is 60s.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bucket_size")
                .short("b")
                .long("bucket_size")
                .value_name("SIZE")
                .help("maximum size of the buckets. Default is 20.")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("headless").about("run in headless mode for backbone network"),
        )
        .subcommand(
            SubCommand::with_name("interactive")
                .about("cast votes, do count interactively")
                .arg(
                    Arg::with_name("VOTE_MAPPINGS")
                        .help("Single column file containing the names of the voting options")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("ID_POOL")
                        .help("Single column file containing the vaild ids for votes")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("bulk")
                .about("Cast votes in bulk")
                .arg(
                    Arg::with_name("FILE")
                        .help("CSV file with vote ids and vote values")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    let our_id = KademliaKey(matches.value_of("OUR_ID").unwrap().parse()?);
    let our_address = matches.value_of("OUR_ADDRESS").unwrap().parse()?;
    let their_id = KademliaKey(matches.value_of("THEIR_ID").unwrap().parse()?);
    let their_address = matches.value_of("THEIR_ADDRESS").unwrap().parse()?;
    let bucket_size = matches.value_of("bucket_size").unwrap_or("20").parse()?;
    let timeout = Duration::from_secs(matches.value_of("timeout").unwrap_or("60").parse()?);

    if let Some(matches) = matches.subcommand_matches("bulk") {
        bulk_voting::<NodeCount, Concurrency>(
            our_id,
            our_address,
            their_id,
            their_address,
            bucket_size,
            timeout,
            matches.value_of("FILE").unwrap().to_owned(),
        )?;
    }
    if let Some(_) = matches.subcommand_matches("headless") {
        headless::<NodeCount, Concurrency>(
            our_id,
            our_address,
            their_id,
            their_address,
            bucket_size,
            timeout,
        )?;
    }

    if let Some(matches) = matches.subcommand_matches("interactive") {
        let vote_mappings_file = matches.value_of("VOTE_MAPPINGS").unwrap();
        let id_pool_file = matches.value_of("ID_POOL").unwrap();

        let vote_mappings = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(vote_mappings_file)
            .unwrap()
            .deserialize()
            .map(|d| {
                let r: OptionName = d.unwrap();
                r.0
            })
            .collect();
        let id_pool = csv::Reader::from_path(id_pool_file)
            .unwrap()
            .deserialize()
            .map(|d| {
                let r: PoolID = d.unwrap();
                r.0
            })
            .collect();

        interactive::<NodeCount, Concurrency>(
            our_id,
            our_address,
            their_id,
            their_address,
            bucket_size,
            timeout,
            vote_mappings,
            id_pool,
        )?;
        //interactive(matches.value_of("FILE").unwrap().to_owned());
    }
    Ok(())
}

fn main() {
    if let Err(e) = start() {
        error!("Program terminated with error: {}", e);
    }
}
