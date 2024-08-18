use flume::IntoIter;
use mainline::{Dht, Id};
use std::net::SocketAddr;
use tokio::time::Instant;

pub fn new_dht_client() -> Dht {
    Dht::client().unwrap()
}

pub fn find_peers(dht_client: &Dht, info_hash: Id) -> Vec<SocketAddr> {
    let start = Instant::now();
    let mut first = false;
    let peers: Vec<SocketAddr> = dht_client
        .get_peers(info_hash)
        .unwrap()
        .into_iter()
        .flat_map(|addr| {
            if !first {
                first = true;
                println!("Got first result in {:?}ms", start.elapsed().as_millis());
            }
            addr
        })
        .collect();
    println!(
        "Got {:?} peers in {:?}ms",
        peers.len(),
        start.elapsed().as_millis()
    );

    peers
}