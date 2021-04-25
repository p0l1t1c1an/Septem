use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};

use futures::future::{select, Either};
use futures::pin_mut;
use tokio::sync::Notify;

use xcb::{ConnError, GenericError};
use xcb_util::ewmh;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Connection to the X11 server failed to start or shutdown")]
    ConnectionError(#[from] ConnError),

    #[error("Failed to find screen with id: {0}")]
    ScreenIteratorError(i32),

    // Could integrate xcb-util-errors to get verbose error string
    #[error(
        "Failed either the mask change, EWMH connection, or active window. \nError Code is {1}"
    )]
    GenericXcbError(GenericError, u8),

    #[error("The pid mutex failed to load")]
    PosionedMutexError,
}

impl From<GenericError> for EventError {
    fn from(error: GenericError) -> Self {
        let code = error.error_code();
        EventError::GenericXcbError(error, code)
    }
}

type EventResult<T> = Result<T, EventError>;

pub struct EventHandler {
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

        let ewmh = xcb_util::ewmh::Connection::connect(conn).or_else(|(e, _)| Err(e))?;

        Ok((ewmh, screen_id))
    }

    pub fn new() -> EventResult<EventHandler> {
        let (ewmh, screen) = Self::establish_conn()?;

        let aw = ewmh.ACTIVE_WINDOW();
        let wm = ewmh.WM_NAME();
        let vn = ewmh.WM_VISIBLE_NAME();

        Ok(EventHandler {
            conn: ewmh,
            screen_id: screen,
            active_win: aw,
            wm_name: wm,
            vis_name: vn,
        })
    }

    pub async fn start(
        self,
        pid_cond: Arc<(Mutex<u32>, Condvar)>,
        shutdown: Arc<AtomicBool>,
    ) -> EventResult<()> {
        while !shutdown.load(Ordering::Relaxed) {
            let polled = self.conn.wait_for_event();
            match polled {
                None => {
                    break;
                }
                Some(e) => {
                    let r = e.response_type() & !0x80;
                    if r == xcb::PROPERTY_NOTIFY {
                        let prop: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&e) };
                        let atom = prop.atom();

                        if atom == self.active_win || atom == self.wm_name || atom == self.vis_name
                        {
                            let active =
                                xcb_util::ewmh::get_active_window(&self.conn, self.screen_id)
                                    .get_reply()?;
                            {
                                let (pid, cond) = &*pid_cond;

                                match pid.lock() {
                                    Ok(mut p) => {
                                        *p = xcb_util::ewmh::get_wm_pid(&self.conn, active)
                                            .get_reply()?
                                    }
                                    Err(_) => Err(EventError::PosionedMutexError)?,
                                }

                                cond.notify_one();
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
