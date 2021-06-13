use crate::server::client::{Client, ClientResult, Timeout, PidSender, Running};
use crate::config::event_config::EventConfig;

use async_trait::async_trait;
use tokio::time::Duration;

use xcb::{ConnError, GenericError};
use xcb_util::ewmh;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Connection to the X11 server failed to start or stopped running")]
    ConnectionError(#[from] ConnError),

    #[error("Failed to find screen with id: {0}")]
    ScreenIteratorError(i32),

    // Could integrate xcb-util-errors to get verbose error string
    #[error("Failed either the mask change, EWMH connection, or active window\nError Code is {1}")]
    GenericXcbError(GenericError, u8),

    #[error("Wait_for_event returned None")]
    WaitReturnsNoneError,
    
    #[error("Pid Recv must have closed")]
    PidSenderError,
}


impl From<GenericError> for EventError {
    fn from(error: GenericError) -> Self {
        let code = error.error_code();
        EventError::GenericXcbError(error, code)
    }
}

type EventResult<T> = Result<T, EventError>;

pub struct EventHandler {
    sender: PidSender,
    running: Running,
    timeout: Timeout,
    config: EventConfig,
    conn: ewmh::Connection,
    screen_id: i32,
    active_win: u32,
    wm_name: u32,
    vis_name: u32,
}

unsafe impl Send for EventHandler {}
unsafe impl Sync for EventHandler {}

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

        let ewmh = xcb_util::ewmh::Connection::connect(conn).map_err(|(e, _)| e)?;

        Ok((ewmh, screen_id))
    }

    pub fn new(config: EventConfig, sender: PidSender, running: Running, timeout: Timeout) -> EventResult<EventHandler> {
        let (conn, screen_id) = Self::establish_conn()?;

        let active_win = conn.ACTIVE_WINDOW();
        let wm_name = conn.WM_NAME();
        let vis_name = conn.WM_VISIBLE_NAME();

        Ok(EventHandler {
            sender,
            running,
            timeout,
            config,
            conn,
            screen_id,
            active_win,
            wm_name,
            vis_name,
        })
    }
}

#[async_trait]
impl Client for EventHandler {
    async fn start(self) -> ClientResult<()> {
        let get_aw = |conn, id| -> EventResult<u32> {
            Ok(xcb_util::ewmh::get_active_window(conn, id).get_reply()?)
        };

        let get_pid = |conn, aw| -> EventResult<u32> {
            Ok(xcb_util::ewmh::get_wm_pid(conn, aw).get_reply()?)
        };

        let has_error = || -> EventResult<()> { self.conn.has_error()?; Ok(()) };

        while self.running.load() {
            let wait = self.conn.poll_for_event();
            if let Some(event) = wait {
                let e = event.response_type() & !0x80;
                let prop: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&event) };
                let a = prop.atom();

                if e == xcb::PROPERTY_NOTIFY
                    && (a == self.active_win || a == self.wm_name || a == self.vis_name)
                {
                    let active = get_aw(&self.conn, self.screen_id)?;
                    let pid = if active == xcb::NONE {
                            None
                        } else {
                            Some(get_pid(&self.conn, active)?)
                        };

                    if let Err(_) = self.sender.send(pid).await {
                        return Err(EventError::PidSenderError.into());
                    }
                }
            } else {
                has_error()?;
            }
            self.timeout.wait_timeout(Duration::from_millis(self.config.delay())).await?;
        }
        println!("Event End");
        Ok(())
    }
}
