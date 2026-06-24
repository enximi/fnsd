mod cli;
mod config;
mod core;
mod daemon;
mod hash;
mod protocol;
mod store;
mod sync;
mod vault;
mod ws;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    cli::run().await
}
