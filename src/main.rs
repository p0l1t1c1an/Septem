mod application;
use application::AppError;

use tokio::runtime::Runtime;

fn main() -> Result<(), AppError> {
    let run = Runtime::new().unwrap();
    run.block_on(async { application::start(application::init().await.unwrap()).await.unwrap() } );
    run.shutdown_timeout(std::time::Duration::from_millis(10));
    println!("Main End");
    Ok(())
}
