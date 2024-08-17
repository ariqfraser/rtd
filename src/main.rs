use tokio::{self};
use mainline::Id;
use tracing::Level;
use tracing_subscriber;

mod announcement;
mod torrent_parser;
mod dht;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let dht_client = dht::new_dht_client();

    let path = "E:/Projects/_rust/rtdownloader/example_torrents/xmen.torrent";
    match torrent_parser::read_torrent(path) {
        Ok(t) => {
            torrent_parser::render_torrent(&t);
            let info_bytes = torrent_parser::calculate_info_hash(&t.info)?;
            let info_hash_str = torrent_parser::info_bytes_to_string(&info_bytes);
            println!("Retrived {:?}", info_hash_str);

            let info_hash = Id::from_bytes(info_bytes).unwrap();
            let peers = dht::find_peers(&dht_client, info_hash);
            
        }
        Err(e) => {
            eprintln!("Failed to read torrent: {e}");
        }
    }

    Ok(())
}


