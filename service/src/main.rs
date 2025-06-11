

use std::io::{self, Write};
use microkv::MicroKV;
use std::path::PathBuf;
use std::fmt::Error;
use clap::{App, Arg, ArgMatches, Result};

fn get_db_name<'a>() -> ArgMatches<'a> {
    App::new("microkv-svc")
        .version("0.1.0")
        .author("ex0dus <ex0dus at codemuch.tech>")
        .arg(
            Arg::with_name("DATABASE")
                .required(true)
                .index(1)
                .help("Name of database to interact with. Will be created if doesn't exist.")
                .takes_value(false),
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

fn main() {
    let dbname_args: ArgMatches = get_db_name();
    let dbname: &str= dbname_args.value_of("DATABASE").expect("Cant get database name");

    let mut kv: MicroKV = match get_db(dbname) {
        Ok(kv) => kv,
        Err(e) => {
            eprintln!("Error opening database {}: {}", dbname, e);
            return;
        }
    };

    loop {
        print!("{dbname}> ");

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
                if let Err(e) = process_cmds(&mut kv, &args) {
                    eprintln!("Error processing command: {}", e);
                }
            }
        }
    }
}