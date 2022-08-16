mod arguments_parser;

use clap::Parser;
use arguments_parser::Cli;

pub fn run() {
    let cli = Cli::parse();
}
