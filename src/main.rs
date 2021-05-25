//pub mod application;
pub mod config;
pub mod server;

use application::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    application::start().await?;
    println!("Main End");
    Ok(())
}
