use actix::prelude::{Actor, Handler};
use kademlia::*;
use log::info;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VoteStorageActor {
    pub storage: HashMap<KademliaKey, usize>,
}

impl VoteStorageActor {
    pub fn new() -> Self {
        VoteStorageActor {
            storage: HashMap::new(),
        }
    }
}

impl StorageActor for VoteStorageActor {
    type StorageData = usize;
}

impl Actor for VoteStorageActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {}

    fn stopped(&mut self, _: &mut Self::Context) {}
}

impl Handler<StorageMessage<Self>> for VoteStorageActor {
    type Result = StorageResult<Self>;
    fn handle(&mut self, msg: StorageMessage<Self>, _: &mut Self::Context) -> Self::Result {
        match msg {
            StorageMessage::Store(id, data) => {
                info!("Storing \"{}\" with id \"{}\"", data, id);
                self.storage.insert(id, data);
                // TODO this is a bit ugly because if we store we do not need a result
                // maybe we add a result for store here?
                StorageResult::NotFound
            }
            StorageMessage::Retrieve(id) => {
                if let Some(data) = self.storage.get(&id) {
                    StorageResult::Found(data.clone())
                } else {
                    StorageResult::NotFound
                }
            }
        }
    }
}
