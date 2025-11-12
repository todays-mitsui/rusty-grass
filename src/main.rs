use clap::Parser;
use rusty_grass::parser::parse_prog;
use rusty_grass::vm::VM;
use std::fs::File;
use std::io::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[arg()]
    prog_file: String,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        init_trace();
    }

    let mut f = File::open(args.prog_file).expect("program file not found");

    let mut prog_source = String::new();
    f.read_to_string(&mut prog_source)
        .expect("failed to read program file");

    let prog = parse_prog(&prog_source).expect("failed to parse program");

    let mut vm = VM::new(&prog);
    vm.run().expect("runtime error occurred");
}

fn init_trace() {
    let filter = EnvFilter::new("debug");

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact() // 1行表示
        .with_level(false)
        .with_ansi(false) // 色無し
        .with_target(false) // target表示無し
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_writer(std::io::stderr);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
