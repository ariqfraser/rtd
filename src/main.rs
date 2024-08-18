use futures::future::join_all;
use mainline::Id;
use tokio::{self, net::TcpStream};
use tracing::Level;
use tracing_subscriber;

mod announcement;
mod connection;
mod dht;
mod torrent_parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let dht_client = dht::new_dht_client();

    let path = "E:/Projects/_rust/rtdownloader/example_torrents/xmen.torrent";
    match torrent_parser::read_torrent(path) {
        Ok(t) => {
            torrent_parser::render_torrent(&t);
            let info_bytes = torrent_parser::calculate_info_hash(&t.info)?;
            let info_hash = Id::from_bytes(info_bytes).unwrap();
            let peers = dht::find_peers(&dht_client, info_hash);
            let n_peers = &peers.len();

            let peer_id = connection::generate_peer_id();
            println!("Peer ID: {:?}", peer_id);

            let connections: Vec<_> = join_all(peers
                .into_iter()
                .map(|peer_addr| {
                    let peer_addr_clone = peer_addr.clone();
                    tokio::spawn(async move {
                        println!("{}\t\tAttempting connection", &peer_addr_clone);
                        match connection::connect_to_peer(&peer_addr_clone).await {
                            Ok(stream) => {
                                println!("{}\t\tConnection Success", &peer_addr_clone);
                                Some(stream)
                            }
                            Err(_) => {
                                // eprintln!("{}\t\tFailed to connect", &peer_addr);
                                None
                            }
                        }
                    })
                })).await;

            let successful_connections: Vec<TcpStream> = connections
                .into_iter()
                .filter_map(|res| match res {
                    Ok(Some(stream)) => Some(stream),
                    _ => None,
                })
                .collect();

            println!(
                "{} successful connections out of {} total connections",
                successful_connections.len(),
                n_peers
            );

            let tasks: Vec<_> = successful_connections
                .into_iter()
                .map(|stream| {
                    tokio::spawn(async move {
                        match connection::perform_handshake(stream, &info_bytes, &peer_id).await {
                            Ok(stream) => {
                                println!("Handshake\tSuccess");
                            }
                            Err(e) => {
                                eprintln!("{}", e)
                            }
                        };
                    })
                })
                .collect();

            join_all(tasks).await;
        }

        Err(e) => {
            eprintln!("Failed to read torrent: {e}");
        }
    }

    Ok(())
}
