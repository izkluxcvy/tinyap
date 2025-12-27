mod activitypub;
mod back;
mod cli;
#[cfg(feature = "web")]
mod web;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    cli::parse::parse().await;
}
