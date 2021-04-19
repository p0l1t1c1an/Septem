
use crate::application::process;
use process::Process;

use xcb::{ConnError, GenericError, GenericEvent};
use xcb_util::ewmh;

use tokio::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Failed to establish connection to the X11 server")]
    ConnectionError(#[from] ConnError),

    #[error("Failed to find screen with id: {0}")]
    ScreenIteratorError(i32),

    // Could integrate xcb-util-errors to get verbose error string
    #[error("Failed either the mask change, EWMH connection, or active window. \nError Code is {1}")]
    GenericXcbError(GenericError, u8),
   
    #[error("Why are you running?")]
    UnknownError,
}

impl From<GenericError> for EventError {
    fn from(error : GenericError) -> Self {
        let code = error.error_code();
        EventError::GenericXcbError(error, code) 
    }
}

type EventResult<T> = Result<T, EventError>;

pub struct EventHandler {
    event : u32,
    conn : ewmh::Connection,
    screen_id : i32,
    min_time : Duration,
    active_win : u32,
    wm_name : u32,
    vis_name : u32,
    curr_desk : u32,
}

unsafe impl Send for EventHandler { }

impl EventHandler {
   
    fn establish_conn() -> EventResult<(ewmh::Connection, i32)> {
        let (conn, screen_id) = xcb::Connection::connect(None)?;
        conn.has_error()?;

        let screen = conn
            .get_setup()
            .roots()
            .nth(screen_id as usize)
            .ok_or(EventError::ScreenIteratorError(screen_id))?;
        
        let list = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE)];

        let cookie = xcb::change_window_attributes_checked(&conn, screen.root(), &list);
        cookie.request_check()?;

        let ewmh = xcb_util::ewmh::Connection::connect(conn).or_else(|(e,_)| { Err(e) } )?;
        
        Ok((ewmh, screen_id))
    }

    pub fn new(sec : u64) -> EventResult<EventHandler> {
        let (ewmh, screen) = Self::establish_conn()?;
        
        let aw = ewmh.ACTIVE_WINDOW();  
        let wm = ewmh.WM_NAME();
        let vn = ewmh.WM_VISIBLE_NAME();
        let cd = ewmh.CURRENT_DESKTOP();

        Ok( 
            EventHandler {
                event: 0,
                conn : ewmh,
                screen_id : screen,
                min_time : Duration::from_secs(sec),
                active_win : aw,
                wm_name : wm,
                vis_name : vn,
                curr_desk : cd,
            }
        )
    }


    // TODO set up so that it can async check if min_time has pass before updating 
    // (sending that new window has been focused)
    pub async fn start(mut self) -> Result<(), EventError> {
        loop {{ 
            let polled = self.conn.wait_for_event();
            
            match polled {
                None => {
                    break;  // Catch and return error
                }
                Some(e) => { 
                    let r = e.response_type() & !0x80;
                    if r == xcb::PROPERTY_NOTIFY {
                        let prop: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&e) };
                        let atom = prop.atom();

                        if atom == self.active_win || atom == self.wm_name || atom == self.vis_name || atom == self.curr_desk {
                            let active = xcb_util::ewmh::get_active_window(&self.conn, self.screen_id).get_reply()?;
                            self.event = xcb_util::ewmh::get_wm_pid(&self.conn, active).get_reply()?;
                            println!("pid: {}", self.event);
                        }
                    }
                }
            }}

            let proc = Process::new(self.event as i32).await;
            if proc.is_ok() {
                println!("Application Process name: {}", proc.unwrap().name);
            }
        }
        
        Ok(())
    } 
}

