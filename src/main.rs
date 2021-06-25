//pub mod application;
pub mod config;
mod server;

use server::{Server, ServerResult};

#[tokio::main]
async fn main() -> ServerResult<()> {
    Server::new(None)?.await?;
    println!("Main End");
    Ok(())
}
