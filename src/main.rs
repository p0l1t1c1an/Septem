mod application;
use application::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    application::start(application::init().await?).await?;
    Ok(())
}
