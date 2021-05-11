use crate::application::client::{Client, ClientResult, Condition, Pid, Running};

use async_trait::async_trait;
use futures::future::{select, Either};
//use tokio::{select, try_join};

use xcb::{ConnError, GenericError};
use xcb_util::ewmh;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Connection to the X11 server failed to start or running")]
    ConnectionError(#[from] ConnError),

    #[error("Failed to find screen with id: {0}")]
    ScreenIteratorError(i32),

    // Could integrate xcb-util-errors to get verbose error string
    #[error("Failed either the mask change, EWMH connection, or active window\nError Code is {1}")]
    GenericXcbError(GenericError, u8),

    #[error("Wait_for_event returned None")]
    WaitReturnsNoneError,
}

impl From<GenericError> for EventError {
    fn from(error: GenericError) -> Self {
        let code = error.error_code();
        EventError::GenericXcbError(error, code)
    }
}

type EventResult<T> = Result<T, EventError>;

pub struct EventHandler {
    pid: Pid,
    running: Running,
    cond: Condition,
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

    pub fn new(pid: Pid, running: Running, cond: Condition) -> EventResult<EventHandler> {
        let (conn, screen_id) = Self::establish_conn()?;

        let active_win = conn.ACTIVE_WINDOW();
        let wm_name = conn.WM_NAME();
        let vis_name = conn.WM_VISIBLE_NAME();

        Ok(EventHandler {
            pid,
            running,
            cond,
            conn,
            screen_id,
            active_win,
            wm_name,
            vis_name,
        })
    }

    async fn wait_for_event(self) -> EventResult<()> {
        let get_aw = |conn, id| -> EventResult<u32> {
            Ok(xcb_util::ewmh::get_active_window(conn, id).get_reply()?)
        };

        let get_pid = |conn, aw| -> EventResult<u32> {
            Ok(xcb_util::ewmh::get_wm_pid(conn, aw).get_reply()?)
        };

        while self.running.load() {
            let wait = self.conn.wait_for_event();
            if let Some(event) = wait {
                let e = event.response_type() & !0x80;
                let prop: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&event) };
                let a = prop.atom();

                if e == xcb::PROPERTY_NOTIFY
                    && (a == self.active_win || a == self.wm_name || a == self.vis_name)
                {
                    let active = get_aw(&self.conn, self.screen_id)?;
                    {
                        self.pid.set_pid(if active == xcb::NONE {
                                None
                            } else {
                                Some(get_pid(&self.conn, active)?)
                            }).await;
                    }
                    self.pid.notify_one();
                }
            } else {
                return Err(EventError::WaitReturnsNoneError.into());
            }
        }
        println!("Event End");
        Ok(())
    }

    async fn wait_for_condition(pid: Pid, running: Running, cond: Condition) {
        cond.wait().await;
        running.store(false);
        println!("Cond End");
        pid.notify_one();
    }
}

#[async_trait]
impl Client for EventHandler {
    async fn start(self) -> ClientResult<()> {
        {
            let stopped = tokio::spawn(EventHandler::wait_for_condition(
                self.pid.clone(),
                self.running.clone(),
                self.cond.clone(),
            ));

            let event = tokio::spawn(self.wait_for_event());

            match select(event, stopped).await {
                Either::Left((left, _)) => left??,
                Either::Right((right, _)) => right?,
            }
        }

        println!("Very End");
        Ok(())
    }
}
