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
                        if prop.atom() == ewmh.ACTIVE_WINDOW() {
                            println!("Property: AW");
                        } else if prop.atom() == ewmh.WM_NAME() {
                            println!("Property: WN");
                        } else if prop.atom() == ewmh.WM_VISIBLE_NAME() {
                            println!("Property: WVN");
                        } else if prop.atom() == ewmh.CURRENT_DESKTOP() {
                            println!("Property: CD");
                        } else {
                            println!("Other Property: {}", prop.atom());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
