use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use core::str;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

type ResBox<T> = Result<T, Box<dyn std::error::Error>>;

pub fn new_announcement_request(announce_base: &str, info_hash: &str) -> String {
    let peer_id = generate_peer_id();
    format!(
        "{}&info_hash={}&peer_id={}",
        announce_base, info_hash, peer_id
    )
}

fn generate_peer_id() -> String {
    let mut peer_id = [0u8; 20];
    let mut rng = thread_rng();
    rng.fill(&mut peer_id);

    let peer_id_hex = hex::encode(peer_id);
    peer_id_hex[..20].to_string()
}

fn strip_announce_url(url: &str) -> String {
    // let url = Url::parse(url).expect("msg");
    // let scheme = url.scheme();
    // let host = url.host_str().expect("No host string");
    // let port = url.port().unwrap_or(80);
    // let addr = format!("{}://{}:{}", scheme, host, port.to_string());
    String::from(url).replace("/announce", "")
}

pub fn handle_udp_request(addr: &str) -> std::io::Result<()> {
    println!("Requesting: {:?}", addr);
    let socket = UdpSocket::bind("127.0.0.1:6881").expect("Could not bind address");
    println!("UDP Socket initialised");
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    let connection_request = create_connection_request();

    println!("Req: {:?}", &connection_request);
    let req = socket
        .send_to(&connection_request, addr)
        .map_err(|e| format!("Could not send message: {e}"));

    match req {
        Ok(_) => {
            // Buffer to store response
            let mut buffer = [0u8; 1024];

            let (amt, src) = socket.recv_from(&mut buffer)?;

            let connection_id = parse_connection_response(&buffer[..amt]).unwrap();

            println!("Received connection ID: {}", connection_id);
        }
        Err(e) => println!("Err: {:?}", e),
    }

    Ok(())
}

fn create_connection_request() -> Vec<u8> {
    let mut buf = Vec::with_capacity(16);
    buf.write_u64::<BigEndian>(0x41727101980).unwrap(); // Protocol ID for BitTorrent
    buf.write_u32::<BigEndian>(0).unwrap(); // Action: 0 for connect
    buf.write_u32::<BigEndian>(rand::thread_rng().gen())
        .unwrap(); // Transaction ID
    buf
}

fn parse_connection_response(response: &[u8]) -> Result<u64, &'static str> {
    let mut cursor = std::io::Cursor::new(response);

    let action = cursor.read_u32::<BigEndian>().unwrap();
    if action != 0 {
        return Err("Invalid action in connection response");
    }

    let _transaction_id = cursor.read_u32::<BigEndian>().unwrap();
    let connection_id = cursor.read_u64::<BigEndian>().unwrap();

    Ok(connection_id)
}

async fn handle_http_request(url: &str) -> ResBox<()> {
    println!("Requesting: {}", url);
    let addr = strip_announce_url(url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let res = client
        .get(addr)
        .header(reqwest::header::USER_AGENT, "qBittorrent/4.3.3")
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Request failed with status: {}", res.status()).into());
    }
    println!("test");
    let text = res.text().await?;
    println!("raw body: {}", text);

    Ok(())
}

pub async fn query_trackers(announce_list: &Vec<Vec<String>>) -> ResBox<()> {
    for tracker in announce_list {
        let url = strip_announce_url(&tracker[0]);
        if tracker[0].starts_with("udp://") {
            if let Err(e) = handle_udp_request(&url) {
                println!("Failed: {:?}", e);
                println!("");
                continue;
            }
        } else if url.starts_with("http://") || url.starts_with("https://") {
            println!("Tracker is on http(s): {} | {}", url, tracker[0]);

            if let Err(e) = handle_http_request(&url).await {
                println!("Failed top level request: {:?}", e);
                println!("");
                continue;
            };
        }
        println!("");
    }

    Ok(())
}
