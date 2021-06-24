//pub mod application;
pub mod config;
mod server;

use server::{Server, ServerError};

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    let server = Server::new(None)?;
    while server.is_running() { } 
    println!("Main End");
    server.close().await;
    Ok(())
}
