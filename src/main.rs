use std::env;
use std::process;

fn main() {

    // Take in file path
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: crablox -- [script]");
        process::exit(64);
    }
    else if args.len() == 2 { 
        crablox::run_file(&args[1]);
    }
    else {
        crablox::run_prompt();
    }

}