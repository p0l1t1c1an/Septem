
use crate::application::process;

use std::time::SystemTime;
use async_trait::async_trait;
use tokio::sync::{mpsc, watch};


/*
 * TODO:
 *
 * Create the values for the errors.
 * It should be based on values that couldn't 
 * be received, maybe a timeout error, 
 * a tokio mspc / watch error, and 
 * incorrect Data received / asked for.
 * 
 * Then, it should have one for each threads
 * own errors. 
 *
 */


pub enum ClientServerError {
    
}

pub enum Data { 
    CurrectProc(process::Process),
    CurrentProcStart(SystemTime), 
    Interupt(u32),
    Shutdown,
}


pub enum Message {
    GetFromServer{from : u8, data : Data},
    GetFromClient{to : u8, data : Data},
    
    ReturnToServer{data : Data},
    ReturnToClient{to : u8, data : Data},
    
    SendToClient{to : u8, data : Data},

    Shutdown{data : Data},
}

/*
 * TODO: 
 *
 * Start clients needs to return the join handles created 
 * when spawning new threads for each client.
 * 
 * It should run tokio spawn on each start for a client. 
 * Then, use join! macro to combine handle and return.
 *
 */


#[async_trait]
pub trait Client {
    fn for_me(&self, to : u8) -> bool;

    async fn handle_message(&self, message : Message) -> Result<(), ClientServerError>; 

    async fn start(self, sender : mpsc::Sender<Message>, receiver : watch::Receiver<Message> ) -> Result<(), ClientServerError>;
}

#[async_trait]
pub trait Server { 
    async fn handle_message(&self, message : Message) -> Result<(), ClientServerError>; 

    async fn start_clients(&self, clients : Vec<Box<dyn Client>>) -> Result<(), ClientServerError>;

    async fn start(&self, sender : mpsc::Sender<Message>, receiver : watch::Receiver<Message>) -> Result<(), ClientServerError>;
}

