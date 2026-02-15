mod activitypub;
#[cfg(feature = "api")]
mod api;
mod back;
mod cli;
#[cfg(feature = "web")]
mod web;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    cli::parse::parse().await;
}
