
mod application;
use application::event_handler::EventHandler;     

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let handler = EventHandler::new(5)?;
    
    let thread = tokio::spawn(handler.start());
    let _ = thread.await?;

    Ok(())
}
