mod application;
use application::AppError;

use tokio::runtime::Runtime;

async fn start() -> Result<(), AppError> {
    application::start(application::init().await?).await?;
    Ok(())
}

fn main() -> Result<(), AppError> {
    let run = Runtime::new().unwrap();
    let to_check = run.block_on(start());
    run.shutdown_timeout(std::time::Duration::from_millis(10));
    println!("Main End");
    to_check
}
