use clap::Parser;
use rusty_grass::parser::parse_prog;
use rusty_grass::vm::VM;
use std::fs::File;
use std::io::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    prog_file: String,
}

fn main() {
    let args = Args::parse();

    let mut f = File::open(args.prog_file).expect("program file not found");

    let mut prog_source = String::new();
    f.read_to_string(&mut prog_source)
        .expect("failed to read program file");

    let prog = parse_prog(&prog_source).expect("failed to parse program");

    let mut vm = VM::new(&prog);
    vm.run().expect("runtime error occurred");
}
