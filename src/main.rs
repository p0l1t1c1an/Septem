pub mod config;
mod server;

use server::{Server, ServerResult};

use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    /// Optional Config file location
    #[structopt(short, long)]
    config: Option<String>,
    
    // Daemonize the program
     #[structopt(short, long)]
    daemonize: bool,

    // Print to log files 
     #[structopt(long)]
    enable_logs: bool,

    // Print debug information 
     #[structopt(long)]
    debug: bool,
}


#[tokio::main]
async fn main() -> ServerResult<()> {
    let opt = Options::from_args();
    Server::new(opt.config.to_owned())?.await?;
    println!("Main End");
    Ok(())
}
