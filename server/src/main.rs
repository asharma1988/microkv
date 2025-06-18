use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use microkv::MicroKV;
use std::fmt::Error;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, KVError>;

#[derive(Debug)]
pub enum ErrorType {
    KVError,     // issues involving database interactions
    CryptoError, // problems arisen from performing authentication encryption
    FileError,   // unified type for io::Error
    PoisonError, // locking error, indicating poisoned mutex
}

pub struct KVError {
    pub error: ErrorType,
    pub msg: Option<String>,
}

struct Connection {
    //socket: &'a tokio::net::TcpStream, 
    db: String,
    kv: Option<MicroKV>,
}

fn get_db(dbname: &str) -> Result<MicroKV> {

    let path: PathBuf = MicroKV::get_db_path(dbname);
    match std::fs::metadata(&path) {
        Ok(_) => println!("metadata() succeeded"),
        Err(e) => println!("metadata() failed: {}", e),
    }
    let kv: MicroKV = match path.as_path().exists() {
        true => {
            println!("Using database: {}", dbname);
            MicroKV::open(dbname).expect("Failed to open existing database")
        }
        false => {
            println!("New database: {}", path.to_str().unwrap().to_string());
            MicroKV::new(dbname)
        }
    };
    Ok(kv)
}

fn process_cmds(kv: &mut MicroKV, args: &Vec<String>) -> clap::Result<()> {
    let cmd: &String = &args[0];
    if cmd == "get" {
        let key: &String = &args[1];
        let val: Option<String> = kv.get(key).expect("Failed to get value");
        match val {
            Some(val) => {
                println!("{val}");
            }
            None => {
                println!("Key not found: {key}");
                return Err(Error.into());
            }
        }
    } else if cmd == "put" {
        let key: &String = &args[1];
        let value: &String = &args[2];
        kv.put(&key, &value).expect("Failed to put value");
    } else if cmd == "list" {
        println!("Processing command: {}", cmd);
        let keys = match kv.keys() {
            Ok(_keys) => _keys,
            Err(e) => {
                eprintln!("Error listing keys: {}", e);
                return Err(Error.into());
            }
        };
        for key in keys {
            println!("{}", key);
        }
    } else if cmd == "flush" {
        if let Err(e) = kv.commit() {
            eprintln!("Error while flushing: {}", e);
            return Err(Error.into());
        }
    } else if cmd == "rm" {
        let key: &String = &args[1];
        if let Err(e) = kv.delete(key) {
            eprintln!("Error while deleting key {}: {}", key, e);
            return Err(Error.into());
        }
    } else {
        println!("Unknown command: {}", cmd);
        return Err(Error.into());
    }
    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("Server listening on 7878");

    loop {
        let (mut socket, addr) = listener.accept().await?;

        let mut conn: Box<Connection> = Box::new(Connection {
            //socket: &socket,
            db: "default".to_string(),
            kv: None,
        });

        if conn.kv.is_none() {
            println!("New connection from {}", addr);
        }

        tokio::spawn(async move {
            let mut socket = socket;
            let mut conn = conn;
            let mut buf = [0u8; 1024];
            loop {
                match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => {
                        if conn.kv.is_none() {
                            let db = std::str::from_utf8(&buf).unwrap();
                            let dbname = db.trim_matches(char::from(0)).to_string();
                            match get_db(&dbname) {
                                Ok(kv) => {
                                    conn.kv = Some(kv);
                                    socket.write_all(b"OK").await.unwrap();
                                }
                                Err(e) => {
                                    eprintln!("Error connecting to database: {}", db);
                                    return;
                                }
                            }
                        } else {
                            let args = String::from_utf8_lossy(&buf[..n]).split_whitespace()
                                .map(String::from)
                                .collect::<Vec<String>>();
                            let kv = conn.kv.as_mut();
                            process_cmds(kv.unwrap(), &args);
                            
                        //let _ = socket.write_all(b"Hello async\n").await;
                        }

                    }
                    Ok(_) => println!("Connection closed by client"),
                    Err(e) => eprintln!("Failed to read from socket: {}", e),
                }
            }
        });
    }
}