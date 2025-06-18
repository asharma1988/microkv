

use std::io::{self, Write};
use microkv::MicroKV;
use std::path::PathBuf;
use std::fmt::Error;
use clap::{App, Arg, ArgMatches, Result};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::runtime::Runtime;

fn get_db_name<'a>() -> ArgMatches<'a> {
    App::new("microkv-client")
        .version("0.1.0")
        .author("ex0dus <ex0dus at codemuch.tech>")
        .arg(
            Arg::with_name("DATABASE")
                .required(false)
                .help("Name of database to interact with. Will be created if doesn't exist.")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("remote")
                .short("r")
                .long("remote")
                .help("Connect to a remote database server.")
                .takes_value(true)
                .required(false),
        )
        .get_matches()
}

fn get_db(dbname: &str) -> Result<MicroKV> {

    let path: PathBuf = MicroKV::get_db_path(dbname);
    let kv: MicroKV = match path.as_path().exists() {
        true => MicroKV::open(dbname).expect("Failed to open existing database"),
        false => MicroKV::new(dbname),
    };
    Ok(kv)
}

/*fn parse_args(args: &ArgMatches) -> Result<String> {

}*/

fn process_cmds(kv: &mut MicroKV, args: &Vec<String>) -> Result<()> {
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

async fn connect_db(dbname: &str, server: &str) -> Result<()> {
    let ep = format!("{}:7878", server);
    let mut stream = TcpStream::connect(ep).await?;

    print!("{dbname}@{server}> ");
    io::stdout().flush().unwrap();

    stream.write_all(dbname.as_bytes()).await?;

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let status = String::from_utf8_lossy(&buffer[..n]);
    if status.trim() == "OK" {
        println!("Connected to database: {}", dbname);
    }
    //println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    loop {
        print!("{dbname}@{server}> ");

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "exit" | "quit" => {
                println!("Exiting...");
                break;
            }
            _ => {
                let args: Vec<String> = input.trim().split_whitespace().map(String::from).collect();
                if args.is_empty() {
                    continue;
                }
                let cmd = &args[0];
                if cmd == "get" || cmd == "put" || cmd == "rm" || cmd == "list" || cmd == "flush" {
                    if let Err(e) = stream.write_all(input.as_bytes()).await {
                        eprintln!("Error sending command: {}", e);
                        continue;
                    }
                    // Read response
                    let n = stream.read(&mut buffer).await?;
                    println!("Response: {}", String::from_utf8_lossy(&buffer[..n]));
                } else {
                    println!("Unknown command: {}", cmd);
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let dbname_args: ArgMatches = get_db_name();
    let dbname: &str= dbname_args.value_of("DATABASE").expect("Cant get database name");
    let mut server: &str = "localhost"; // Default to localhost if not specified
    match dbname_args.value_of("remote") {
        Some(remote) => server = remote,
        None => {}
    };


    let mut kv: MicroKV = match get_db(dbname) {
        Ok(kv) => kv,
        Err(e) => {
            eprintln!("Error opening database {}: {}", dbname, e);
            return;
        }
    };

    if server != "localhost" {
        let rt = Runtime::new().expect("Failed to create runtime");
        rt.block_on(connect_db(dbname, server)).expect("Failed to connect to remote database");
    }

    loop {

        print!("{dbname}@{server}> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "exit" | "quit" => {
                println!("Exiting...");
                break;
            }
            _ => {
                let args: Vec<String> = input.trim().split_whitespace().map(String::from).collect();
                if server == "localhost" {
                    if let Err(e) = process_cmds(&mut kv, &args) {
                        eprintln!("Error processing command: {}", e);
                    }
                } else {
                    println!("Control shouldn't have reached here");
                }
            }
        }
    }
}