# P2P Voting Network

Based on my [actor-based kademlia implementation](https://github.com/LevitatingOrange/kademlia) (that gets pulled in as a dependency), this project realizes a very simple and barebones voting system. There is a predefined id pool (specified as a CSV-file), voters draw random ids from this pool and save votes on the network. To count, one selects a number of random votes from the id pool and retrieves these keys from the network.

## To compile & run
* *Only tested on linux, should work on other platforms*
* Install rust *nightly* (best way is [rustup](https://rustup.rs/))
* do `cargo build --release` in this directory
* run the binary `./target/release/p2pvote` (`./target/release/p2pvote help` will show information on how to run it)
* *Optionally* (already generated csv files are provided in `example_data/`): To create bulk network information and do the 1000 node testrun (for the vote choices in `example_data/vote_mappings.csv`):
    * install `python3`, `numpy` and `scipy`
    * run `python3 ./scripts/create_pool.py` to create the id pool of 5000
    * run `python3 ./scripts/create_network_csv.py` to create a csv with 1000 nodes
    * run `python3 ./scripts/create_bulk_example.py` to create a csv with 500 random bulk votes
    * run `python3 ./scripts/run_network.py` to start the 1000 nodes locally

* run `./target/release/p2pvote 127.0.0.1:8000 4124 127.0.0.1:<any port from the example_data/>network.csv> <id of the selected peer> bulk ./example_data/bulk.csv`. This will insert the bulk votes into the network.
* Now you can run the the program interactively via `cargo run --release 127.0.0.1:8000 4124 127.0.0.1:<any port from the example_data/>network.csv> <id of the selected peer> interactive ./example_data/vote_mappings.csv ./example_data/id_pool.csv`. The interactive program should be self explanatory.
