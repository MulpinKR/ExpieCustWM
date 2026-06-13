use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutMode {
    Tiling,
    Floating,
    Monocle,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub window: Window,
    pub frame: Window,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub floating: bool,
    pub fullscreen: bool,
    pub urgent: bool,
    pub is_dialog: bool,
}

impl Client {
    pub fn new(window: Window, frame: Window) -> Self {
        Self {
            window,
            frame,
            x: 0, y: 0,
            width: 800, height: 600,
            floating: false,
            fullscreen: false,
            urgent: false,
            is_dialog: false,
        }
    }

    pub fn resize(&self, conn: &RustConnection, x: i32, y: i32, w: u32, h: u32, bw: u32) -> anyhow::Result<()> {
        conn.configure_window(self.frame, &ConfigureWindowAux::new()
            .x(x)
            .y(y)
            .width(w + bw * 2)
            .height(h + bw * 2))?;
        conn.configure_window(self.window, &ConfigureWindowAux::new()
            .x(bw as i32)
            .y(bw as i32)
            .width(w)
            .height(h))?;
        conn.flush()?;
        Ok(())
    }

    pub fn focus(&self, conn: &RustConnection) -> anyhow::Result<()> {
        conn.set_input_focus(InputFocus::PARENT, self.window, 0u32)?;
        conn.change_window_attributes(self.frame, &ChangeWindowAttributesAux::new()
            .border_pixel(0x005294e2))?;
        conn.flush()?;
        Ok(())
    }

    pub fn unfocus(&self, conn: &RustConnection) -> anyhow::Result<()> {
        conn.change_window_attributes(self.frame, &ChangeWindowAttributesAux::new()
            .border_pixel(0x00222222))?;
        conn.flush()?;
        Ok(())
    }

    pub fn close(&self, conn: &RustConnection, wm_protocols: Atom, wm_delete_window: Atom) -> anyhow::Result<()> {
        let reply = conn.get_property(
            false,
            self.window,
            wm_protocols,
            AtomEnum::ATOM,
            0,
            32,
        )?.reply()?;

        let has_delete = if reply.format == 32 {
            let atoms: &[Atom] = bytemuck::cast_slice(&reply.value);
            atoms.iter().any(|&a| a == wm_delete_window)
        } else {
            false
        };

        if has_delete {
            let event = ClientMessageEvent::new(
                32,
                self.window,
                wm_protocols,
                ClientMessageData::from([wm_delete_window as u32, 0u32, 0, 0, 0]),
            );
            conn.send_event(false, self.window, EventMask::NO_EVENT, event)?.check()?;
        } else {
            conn.kill_client(self.window)?.check()?;
        }
        conn.flush()?;
        Ok(())
    }
}
