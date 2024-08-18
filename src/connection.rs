use std::{net::SocketAddr, time::Duration};
use rand::{thread_rng, Rng};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, time::{sleep, timeout}};

pub async fn connect_to_peer(peer_ip: &SocketAddr) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let connection_timeout = Duration::from_secs(5);

    match timeout(connection_timeout, TcpStream::connect(peer_ip)).await {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(e)) => Err(Box::new(e)),
        Err(_) => Err(Box::from("Connection timed out"))
    } 
}

pub async fn perform_handshake(mut stream: TcpStream, info_hash: &[u8; 20], peer_id: &[u8; 20]) -> Result<(), Box<dyn std::error::Error>> {
    let pstr = b"\x13BitTorrent protocol";
    let res_buf = [0u8; 8];
    let mut handshake = Vec::new();

    handshake.extend_from_slice(pstr);
    handshake.extend_from_slice(&res_buf);
    handshake.extend_from_slice(info_hash);
    handshake.extend_from_slice(peer_id);


    // println!("Attempting handshake:\n\tinfo_hash: {:?}\n\tpeer_id: {:?}", info_hash, peer_id);
    if let Err(_) =  stream.write_all(&handshake).await {
        return Err(Box::from("Failed to send handshake"));
    }; 

    // Response Buffer
    let mut buffer = vec![0u8; 256]; // 1 + 19 + 8 + 20 + 20
    let mut total_read = 0;
    while total_read < buffer.len() {
        let n = stream.read(&mut buffer[total_read..]).await?;
        if n == 0 {
            break; // EOF reached
        }
        total_read += n;
    }

    if let Some(start_index) = buffer
        .windows(info_hash.len())
        .position(|window| window == info_hash) {
        let end_index = start_index + info_hash.len() - 1;
        println!("Found hash in buffer at {}..{}", start_index, end_index);
        process_bitfield(stream).await?;
        return Ok(());
    }

    Err(Box::from("Hadshake\tFailed"))
}

pub fn generate_peer_id() -> [u8; 20] {
    let mut peer_id = [0u8; 20];
    let mut rng = thread_rng();
    rng.fill(&mut peer_id);
    peer_id
}

pub async fn process_bitfield(mut stream: TcpStream) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut length_buf = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut length_buf).await {
        return Err(Box::from(format!("Could not read length\t{:?}", e)));
    };
    let message_length = u32::from_be_bytes(length_buf);
    println!("\nRaw buffer length: {:?}", length_buf);
    println!("Interpreted message length: {}", message_length);

    if message_length > 1024 * 1024 {
        return Err("Message length is too large".into())
    }

    let mut message_buf = vec![0u8; message_length as usize];
    if let Err(_) = stream.read_exact(&mut message_buf).await {
        return Err(Box::from("Could not read buffer {e}"))
    };

    if message_buf[0] == 5 {
        let bitfield = message_buf[1..].to_vec();
        println!("Recieved bitfield: {:?}", bitfield);
        return Ok(bitfield)
    }
    Ok(Vec::new())
}