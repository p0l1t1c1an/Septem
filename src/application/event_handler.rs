use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};

use futures::future::{select, Either};
use tokio::task::JoinError;
use tokio::{select, try_join};

use xcb::{ConnError, GenericError, GenericEvent};
use xcb_util::ewmh;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventError {
    #[error("Connection to the X11 server failed to start or shutdown")]
    ConnectionError(#[from] ConnError),

    #[error("Failed to find screen with id: {0}")]
    ScreenIteratorError(i32),

    // Could integrate xcb-util-errors to get verbose error string
    #[error("Failed either the mask change, EWMH connection, or active window\nError Code is {1}")]
    GenericXcbError(GenericError, u8),

    #[error("Wait_for_event returned None")]
    WaitReturnsNoneError,

    #[error("The {0} mutex failed to lock")]
    PosionedMutexError(String),

    #[error("The {0} condvar failed to load")]
    PosionedCondvarError(String),

    #[error("{0}")]
    SelectError(#[from] JoinError),
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

    async fn wait_for_event(
        self,
        shutdown: Arc<(AtomicBool, Mutex<()>, Condvar)>,
        pid_cond: Arc<(Mutex<u32>, Condvar)>,
    ) -> EventResult<()> {
        while !shutdown.0.load(Ordering::Relaxed) {
            match self.conn.wait_for_event() {
                None => Err(EventError::WaitReturnsNoneError)?,
                Some(event) => {
                    let e = event.response_type() & !0x80;
                    let prop: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&event) };
                    let a = prop.atom();

                    if e == xcb::PROPERTY_NOTIFY {
                        if a == self.active_win || a == self.wm_name || a == self.vis_name {
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
                                    Err(_) => {
                                        Err(EventError::PosionedMutexError("pid".to_owned()))?
                                    }
                                }

                                cond.notify_one();
                            }
                        }
                    }
                }
            }
        }
        println!("Event End");
        Ok(())
    }

    async fn wait_for_condition(
        shutdown: Arc<(AtomicBool, Mutex<()>, Condvar)>,
    ) -> EventResult<()> {
        let (_, m, c) = &*shutdown;
        match m.lock() {
            Ok(guard) => match c.wait(guard) {
                Ok(_) => {
                    shutdown.0.store(true, Ordering::Relaxed);
                    println!("Cond End");
                }
                Err(_) => Err(EventError::PosionedCondvarError("shutdown".to_owned()))?,
            },
            Err(_) => Err(EventError::PosionedMutexError("shutdown".to_owned()))?,
        }
        Ok(())
    }

    pub async fn start(
        self,
        pid_cond: Arc<(Mutex<u32>, Condvar)>,
        shutdown: Arc<(AtomicBool, Mutex<()>, Condvar)>,
    ) -> EventResult<()> {
        {
            let event = tokio::spawn(self.wait_for_event(shutdown.clone(), pid_cond.clone()));
            let stopped = tokio::spawn(EventHandler::wait_for_condition(shutdown.clone()));

            match select(event, stopped).await {
                Either::Left((left, _)) => left??,
                Either::Right((right, _)) => right??,
            };
        }

        let (_, c) = &*pid_cond;
        c.notify_one();
        println!("Very End");

        Ok(())
    }
}
