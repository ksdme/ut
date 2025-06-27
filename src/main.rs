use clap::Parser;

use crate::tool::Tool;

mod tool;
pub mod tools;

fn main() {
    let tool = tools::crypto::token::TokenGenerator::parse();
    println!("{:?}", tool.execute());
}
