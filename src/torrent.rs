use core::str;
use serde::{Deserialize, Serialize};
use serde_bencode::{de, to_bytes};
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use std::{
    collections::BTreeMap, fs::File, io::{self, Read}, path::Path
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Node(String, i64);

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TorrentFile {
    path: Vec<String>,
    crc32: String,
    md5: String,
    sha1: String,
    mtime: String,
    length: i64,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    pub name: String,
    #[serde(default)]
    pub collections: Option<Vec<String>>,
    pub pieces: ByteBuf,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<TorrentFile>>,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Torrent {
    pub info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
    #[serde(default)]
    #[serde(rename = "url-list")]
    url_list: Option<Vec<String>>,
}

pub fn render_torrent(torrent: &Torrent) {
    println!("name:\t\t{}", torrent.info.name);
    println!("announce:\t{:?}", torrent.announce);
    println!("nodes:\t\t{:?}", torrent.nodes);
    if let Some(al) = &torrent.announce_list {
        for a in al {
            println!("announce list:\t{}", a[0]);
        }
    }
    println!("httpseeds:\t{:?}", torrent.httpseeds);
    println!("creation date:\t{:?}", torrent.creation_date);
    println!("comment:\t{:?}", torrent.comment);
    println!("created by:\t{:?}", torrent.created_by);
    println!("encoding:\t{:?}", torrent.encoding);
    println!("piece length:\t{:?}", torrent.info.piece_length);
    println!("private:\t{:?}", torrent.info.private);
    println!("root hash:\t{:?}", torrent.info.root_hash);
    println!("path:\t\t{:?}", torrent.info.path);
    if let Some(files) = &torrent.info.files {
        println!("FILES:");
        for f in files {
            println!("\tpath:\t{:?}", f.path);
            println!("\tlength:\t{}", f.length);
            println!("\tmd5:\t{:?}", f.md5);
            println!("\tcrc32:\t{:?}", f.crc32);
            println!("\tmtime:\t{:?}", f.mtime);
            println!("\tsha1\t{:?}", f.sha1);
            println!("");
        }
    }
    if let Some(ul) = &torrent.url_list {
        for url in ul {
            println!("url\t\t{:?}", url);
        }
    }

    println!("-------------------");
}

pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let path = Path::new(path);
    let mut file = File::open(&path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn read_torrent(path: &str) -> Result<Torrent, Box<dyn std::error::Error>> {
    match read_file(path) {
        Ok(buffer) => match de::from_bytes::<Torrent>(&buffer) {
            Ok(torrent) => Ok(torrent),
            Err(e) => Err(Box::new(e)),
        },
        Err(e) => Err(Box::new(e)),
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum BencodeValue {
    String(String),
    Integer(i64),
    ByteBuf(ByteBuf),
    List(Vec<BencodeValue>),
    Map(BTreeMap<String, BencodeValue>),
}

pub fn calculate_info_hash(info: &Info) -> Result<[u8; 20], Box<dyn std::error::Error>> {
    // Create a BTreeMap to hold the serialized fields with BencodeValue as the value type
    let mut bencode_map: BTreeMap<String, BencodeValue> = BTreeMap::new();

    // Manually add fields that were present in the original Info struct
    bencode_map.insert("name".to_string(), BencodeValue::String(info.name.clone()));
    bencode_map.insert("pieces".to_string(), BencodeValue::ByteBuf(info.pieces.clone()));
    bencode_map.insert("piece length".to_string(), BencodeValue::Integer(info.piece_length));

    if let Some(length) = info.length {
        bencode_map.insert("length".to_string(), BencodeValue::Integer(length));
    }

    if let Some(files) = &info.files {
        let files_list = files.iter().map(|f| {
            let mut file_map = BTreeMap::new();
            file_map.insert("crc32".to_string(), BencodeValue::String(f.crc32.clone()));
            file_map.insert("md5".to_string(), BencodeValue::String(f.md5.clone()));
            file_map.insert("sha1".to_string(), BencodeValue::String(f.sha1.clone()));
            file_map.insert("mtime".to_string(), BencodeValue::String(f.mtime.clone()));
            file_map.insert("length".to_string(), BencodeValue::Integer(f.length));
            file_map.insert("path".to_string(), BencodeValue::List(
                f.path.iter().map(|p| BencodeValue::String(p.clone())).collect()
            ));
            BencodeValue::Map(file_map)
        }).collect();
        bencode_map.insert("files".to_string(), BencodeValue::List(files_list));
    }

    if let Some(private) = info.private {
        bencode_map.insert("private".to_string(), BencodeValue::Integer(private as i64));
    }

    if let Some(path) = &info.path {
        bencode_map.insert("path".to_string(), BencodeValue::List(
            path.iter().map(|p| BencodeValue::String(p.clone())).collect()
        ));
    }

    if let Some(root_hash) = &info.root_hash {
        bencode_map.insert("root hash".to_string(), BencodeValue::String(root_hash.clone()));
    }

    if let Some(source) = &info.source {
        bencode_map.insert("source".to_string(), BencodeValue::String(source.clone()));
    }

    if let Some(collections) = &info.collections {
        bencode_map.insert("collections".to_string(), BencodeValue::List(
            collections.iter().map(|c| BencodeValue::String(c.clone())).collect()
        ));
    }

    // Serialize the BTreeMap into a Bencoded byte array
    let bencoded_info = to_bytes(&bencode_map)?;

    // Calculate the SHA-1 hash of the Bencoded info dictionary
    let mut hasher = Sha1::new();
    hasher.update(&bencoded_info);
    let result = hasher.finalize();

    Ok(result.into())
}
