mod activitypub;
#[cfg(feature = "api")]
mod api;
mod back;
mod cli;
#[cfg(feature = "web")]
mod web;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    // max blocking threads = hash_queue_size
    let hash_queue_size = back::init::hash_queue_size();
    // Build a multi-threaded tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .max_blocking_threads(hash_queue_size)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Entry point: parse CLI
            cli::parse::parse().await;
        })
}
