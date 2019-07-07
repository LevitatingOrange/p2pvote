use crate::interactive::VoteMainActor;
use actix::prelude::*;
use std::net::SocketAddr;
use std::time::Duration;

use crate::storage::*;
use kademlia::*;

pub fn headless<K: FindNodeCount, C: ConcurrenceCount>(
    key: KademliaKey,
    addr: SocketAddr,
    other_key: KademliaKey,
    other_addr: SocketAddr,
    bucket_size: usize,
    timeout: Duration,
) -> Result<(), std::io::Error> {
    System::run(move || {
        let storage_address = VoteStorageActor::create(move |_| VoteStorageActor::new());

        let _controller_address = create_controller::<K, C, VoteStorageActor, VoteMainActor<K, C>>(
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
    })
}
