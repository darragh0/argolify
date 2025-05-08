mod ast;
mod common;
mod parser;
mod tokenizer;

use crate::common::errors::print_err;
use colored::Colorize;
use parser::parse;
use std::env;
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print_err("Exactly 1 positional argument required (file to parse)");
        exit(1);
    }

    let fpath = Path::new(&args[1]);
    let mut err = false;
    if !fpath.exists() {
        print_err(&format!("File does not exist: {}", args[1].yellow()));
        err = true;
    }

    if fpath.extension().is_none_or(|e| e == ".argol") {
        print_err(&format!("File must be a valid {} file", ".argol".yellow()));
        err = true;
    }

    if err {
        exit(2);
    }

    println!("\n{} `{}` ...\n", "Parsing".bright_green().bold(), args[1]);

    if let Err(e) = parse(fpath) {
        eprintln!("{e}");
    }
}
