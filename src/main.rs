use clap::Parser;
use rusty_grass::parser::parse_prog;
use rusty_grass::vm::VM;
use std::fs::File;
use std::io::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

#[derive(Parser, Debug)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    #[arg(short, long, value_name = "program", default_value = None)]
    eval: Option<String>,

    #[arg(value_name = "path/to/progfile", default_value = None)]
    prog_file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        init_trace();
    }

    let prog_source = prog_source(args.eval, args.prog_file.as_deref());
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

fn prog_source(eval: Option<String>, prog_file: Option<&str>) -> String {
    if let Some(source) = eval {
        source
    } else if let Some(file_path) = prog_file {
        let mut f = File::open(file_path).expect("program file not found");
        let mut prog_source = String::new();
        f.read_to_string(&mut prog_source)
            .expect("failed to read program file");
        prog_source
    } else {
        panic!("either --eval or program file must be provided");
    }
}
