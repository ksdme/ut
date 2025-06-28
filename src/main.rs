use clap::Parser;

use crate::tool::Tool;

mod tool;
pub mod tools;

fn main() {
    let tool = tools::crypto::hash::Hash::parse();
    println!("{:?}", tool.execute());
}
