use crate::cli;
use std::env;

pub async fn parse() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Use --help to show available commands.");
        return;
    }

    match args[1].as_str() {
        "--help" => cli::help::show(),
        "--version" => cli::version::show(),
        "serve" => cli::serve::serve().await,
        "useradd" => cli::useradd::useradd(args[2..].to_vec()).await,
        "passwd" => cli::passwd::passwd(args[2..].to_vec()).await,
        "block" => cli::block::block(args[2..].to_vec()).await,
        "unblock" => cli::block::unblock(args[2..].to_vec()).await,
        "blocklist" => cli::block::list().await,
        _ => println!("Use --help to show available commands."),
    }
}
