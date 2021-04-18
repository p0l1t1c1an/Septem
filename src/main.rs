
mod config;
mod application;

use xcb;
use xcb_util;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let (conn, screen_id) = xcb::Connection::connect(None)?;
    conn.has_error()?;

    println!("xcb connect and screen id");

    let screen = conn
        .get_setup()
        .roots()
        .nth(screen_id as usize)
        .ok_or("Failed to get screen")?;
    let list = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE)];

    let cookie = xcb::change_window_attributes_checked(&conn, screen.root(), &list);
    cookie.request_check()?;

    println!("Screen Attributes Changed");

    let ewmh;
    match xcb_util::ewmh::Connection::connect(conn) {
        Ok(e) => ewmh = e,
        Err((error, _)) => {
            println!("{:?}", error);
            return Err(Box::new(error));
        }
    }
    println!("ewmh connect");

    loop {
        let event = ewmh.wait_for_event();
        println!("event get");
        match event {
            None => {
                break;
            }
            Some(event) => {
                let r = event.response_type() & !0x80;
                match r {
                    xcb::PROPERTY_NOTIFY => {
                        let prop: &xcb::PropertyNotifyEvent = unsafe { xcb::cast_event(&event) };
                        if prop.atom() == ewmh.ACTIVE_WINDOW() || prop.atom() == ewmh.WM_NAME() || prop.atom() == ewmh.WM_VISIBLE_NAME() || prop.atom() == ewmh.CURRENT_DESKTOP() {
                            let active = xcb_util::ewmh::get_active_window(&ewmh, screen_id).get_reply()?;
                            let pid = xcb_util::ewmh::get_wm_pid(&ewmh, active).get_reply()?;
                            let name = xcb_util::ewmh::get_wm_name(&ewmh, active).get_reply()?;

                            println!("Name: {}, PID: {}", name.string(), pid);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
