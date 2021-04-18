
use xcb::{ConnError, GenericError, GenericEvent};
use xcb_util::ewmh;

use tokio::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
enum EventError {
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
        EventError::GenericXcbError(error, error.error_code())
    }
}


type EventResult<T> = Result<T, EventError>;

struct EventHandler {
    event : u32,
    conn : ewmh::Connection,
    screen_id : i32,
    min_time : Duration,
    active_win : u32,
    wm_name : u32,
    vis_name : u32,
    curr_desk : u32,
}

impl EventHandler {
    
    pub fn new(sec : u32) -> EventResult<EventHandler> {
        let mut handler : EventHandler;
        handler.event = 0;
        handler.min_time = Duration::from_secs(sec);

        let (conn, screen_id) = xcb::Connection::connect(None)?;
        conn.has_error()?;

        handler.screen_id = screen_id;

        let screen = conn
            .get_setup()
            .roots()
            .nth(screen_id as usize)
            .ok_or(EventError::ScreenIteratorError(screen_id))?;
        
        let list = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE)];

        let cookie = xcb::change_window_attributes_checked(&conn, screen.root(), &list);
        cookie.request_check()?;

        handler.conn = xcb_util::ewmh::Connection::connect(conn).or_else(|(e,_)| { Err(e) } )?;
        handler.active_win = handler.conn.ACTIVE_WINDOW();
        handler.wm_name = handler.conn.WM_NAME();
        handler.vis_name = handler.conn.WM_VISIBLE_NAME();
        handler.curr_desk = handler.conn.CURRENT_DESKTOP();
        Ok(handler)
    }


    // TODO set up so that it can async check if min_time has pass before updating 
    // (sending that new window has been focused)
    pub async fn start(mut self) -> Result<(), EventError>{
        loop { 
            let event = self.conn.wait_for_event();
            
            match event {
                None => {
                    break;  // Catch and return error
                }
                Some(event) => {
                    let r = event.response_type() & !0x80;
                     if r == xcb::PROPERTY_NOTIFY {
                        let prop: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&event) };
                        let atom = prop.atom();

                        if atom == self.active_win || atom == self.wm_name || atom == self.vis_name || atom == self.curr_desk {
                            let active = xcb_util::ewmh::get_active_window(&self.conn, self.screen_id).get_reply()?;
                            self.event = xcb_util::ewmh::get_wm_pid(&self.conn, active).get_reply()?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    } 
}

