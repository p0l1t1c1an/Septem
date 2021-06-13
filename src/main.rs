//pub mod application;
pub mod config;
mod server;

use server::{Server, ServerError};

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    Server::new(None);
    println!("Main End");
    Ok(())
}
