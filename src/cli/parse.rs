use crate::cli;
use std::env;

pub async fn parse() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return;
    }

    match args[1].as_str() {
        "--version" => cli::version::show(),
        "serve" => cli::serve::serve().await,
        "useradd" => cli::useradd::useradd(args[2..].to_vec()).await,
        "passwd" => cli::passwd::passwd(args[2..].to_vec()).await,
        _ => (),
    }
}
