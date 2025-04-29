mod parser;
mod tokenizer;
use parser::parse;

use std::result::Result;

fn main() -> Result<(), String> {
    parse("./test.argol")
}
