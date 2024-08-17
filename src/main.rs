mod magnet;
mod torrent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "E:/Projects/_rust/rtdownloader/example_torrents/oshi.torrent";
    match torrent::read_torrent(path) {
        Ok(t) => {
            torrent::render_torrent(&t);
            let info_bytes = torrent::calculate_info_hash(&t.info)?;
            let info_hash_str = magnet::info_bytes_to_string(&info_bytes);
            println!("Retrived {:?}", info_hash_str);
        }
        Err(e) => {
            eprintln!("Failed to read torrent: {e}");
        }
    }

    Ok(())
}
