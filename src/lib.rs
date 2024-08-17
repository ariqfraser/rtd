pub mod torrent_parser;
pub mod announcement;
pub mod dht;

pub type ResBox<T> = Result<T, Box<dyn std::error::Error>>;
