extern crate core;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use clap::{Parser, Subcommand};

fn main() {

    let args: CliArgs = CliArgs::parse();
    match args.sub_command {
        CliSubCommand::Read { path } => {
            read(&path)
        }
        CliSubCommand::Write { path, value } => {
            write(&path, &value)
        }
        CliSubCommand::Append { path, value } => {
            append(&path, &value)
        }
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(subcommand)]
    sub_command: CliSubCommand,

    #[clap(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum CliSubCommand {
    Read {
        path: String,
    },
    Write {
        path: String,
        value: String
    },
    Append {
        path: String,
        value: String,
    }
}

fn read(path: &str) {
    let mut file = open(path, &OpenOptions::new().read(true));
    let mut buf = String::new();
    file.read_to_string(&mut buf).expect("Failed to read");
    println!("{}", buf);
}

fn write(path: &str, value: &str) {
    let mut file = open(path, &OpenOptions::new().write(true));
    file.write(&value.as_bytes()).expect("Failed to write");
}

fn append(path: &str, value: &str) {
    let mut file = open(path, &OpenOptions::new().append(true));
    file.write(&value.as_bytes()).expect("Failed to append");
}

fn open(path: &str, options: &OpenOptions) -> File {
    let path = fix_path(path);
    options.open("config:/".to_owned() + path)
        .expect("Failed to open config file")
}

fn fix_path(path: &str) -> &str {
    assert_ne!(path.len(), 0, "Path length must be greater than 0");
    path.trim_matches('/')
}