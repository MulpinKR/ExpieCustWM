use std::io::Write;
use std::process::Command;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use crate::bindings::BindingManager;
use crate::client::{Client, LayoutMode};
use crate::config::{BindingAction, Config};
use crate::layout::{Area, LayoutEngine};
use crate::x11::{self, Atoms};

pub struct Wm {
    conn: RustConnection,
    screen_num: usize,
    root: Window,
    atoms: Atoms,
    clients: Vec<Client>,
    focus_index: Option<usize>,
    layout: LayoutEngine,
    config: Config,
    bindings: BindingManager,
    bar_top: u32,
}

impl Wm {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let screen = conn.setup().roots[screen_num].clone();
        let root = screen.root;

        x11::acquire_wm(&conn, &screen)?;
        let atoms = Atoms::intern(&conn)?;
        x11::setup_ewmh(&conn, &screen, &atoms)?;

        let screen_width = screen.width_in_pixels;
        let screen_height = screen.height_in_pixels;

        let mut layout = LayoutEngine::new();
        layout.gap = config.gap;
        layout.border_width = config.border_width;
        layout.master_count = config.master_count;
        layout.master_ratio = config.master_ratio;

        let bindings = BindingManager::new(&config, &conn)?;

        let mut wm = Wm {
            conn,
            screen_num,
            root,
            atoms,
            clients: Vec::new(),
            focus_index: None,
            layout,
            config,
            bindings,
            bar_top: 0,
        };

        wm.scan_existing_windows()?;
        wm.update_ewmh()?;

        log::info!("expiecustWM initialized on screen {} ({}x{})",
            screen_num, screen_width, screen_height);

        Ok(wm)
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.run_autostart();
        self.conn.flush()?;
        std::thread::sleep(std::time::Duration::from_millis(300));
        self.bar_top = self.query_strut_top();

        {
            let screen = &self.conn.setup().roots[self.screen_num];
            if let Some(ref color) = self.config.wallpaper_color {
                let val = u32::from_str_radix(color.trim_start_matches('#'), 16).unwrap_or(0x1a1a2e);
                crate::wallpaper::set_solid(&self.conn, screen, 0xff000000 | val)?;
            } else {
                let path = self.config.wallpaper.as_ref()
                    .map(std::path::PathBuf::from)
                    .or_else(|| {
                        dirs_home().map(|p| p.join("Pictures/expiecustWM.png"))
                    })
                    .filter(|p| p.exists());
                match path {
                    Some(p) => {
                        if let Ok(data) = std::fs::read(&p) {
                            if !data.is_empty() {
                                crate::wallpaper::set_from_png_bytes(&self.conn, screen, &data)?;
                            }
                        }
                    }
                    None => {
                        log::error!(
                            "Wallpaper file not found at ~/Pictures/expiecustWM.png. \
                             Please report to the developer or reinstall the WM. \
                             Falling back to debug wallpaper."
                        );
                        crate::wallpaper::set_debug(&self.conn, screen)?;
                    }
                }
            }
        }

        loop {
            let event = match self.conn.wait_for_event() {
                Ok(ev) => ev,
                Err(e) => {
                    log::error!("X11 connection: {}", e);
                    self.bindings.grab(&self.conn);
                    self.conn.flush()?;
                    continue;
                }
            };
            let result = match event {
                Event::MapRequest(ev) => self.handle_map_request(ev),
                Event::ConfigureRequest(ev) => self.handle_configure_request(ev),
                Event::DestroyNotify(ev) => self.handle_destroy(ev),
                Event::UnmapNotify(ev) => self.handle_unmap(ev),
                Event::KeyPress(ev) => self.handle_keypress(ev),
                Event::ButtonPress(ev) => self.handle_button_press(ev),
                Event::EnterNotify(ev) => self.handle_enter(ev),
                Event::ClientMessage(ev) => self.handle_client_message(ev),
                Event::Expose(ev) => self.handle_expose(ev),
                Event::PropertyNotify(ev) => self.handle_property_notify(ev),
                Event::CreateNotify(_) => {
                    self.bar_top = self.query_strut_top();
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(e) = result {
                log::error!("Handler error: {}", e);
            }
            // re-grab after each event
            self.bindings.grab(&self.conn);
            self.conn.flush()?;
        }
    }

    fn run_autostart(&self) {
        for cmd in &self.config.autostart {
            log::info!("autostart: {}", cmd);
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some(program) = parts.first() {
                let mut child = Command::new(program);
                for arg in &parts[1..] {
                    child.arg(arg);
                }
                if let Err(e) = child.spawn() {
                    log::error!("autostart failed: {}: {}", cmd, e);
                }
            }
        }
    }

    fn query_strut_top(&self) -> u32 {
        let tree = match self.conn.query_tree(self.root) {
            Ok(c) => match c.reply() {
                Ok(r) => r,
                Err(_) => return 0,
            },
            Err(_) => return 0,
        };
        for &child in &tree.children {
            let r = self.conn.get_property(
                false, child, self.atoms.net_wm_strut_partial,
                AtomEnum::CARDINAL, 0, 12,
            );
            let reply = match r {
                Ok(c) => match c.reply() {
                    Ok(r) => r,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };
            if reply.format == 32 && reply.value.len() >= 4 {
                let vals: &[u32] = bytemuck::cast_slice(&reply.value);
                let left = vals[0];
                let right = vals[1];
                let top = vals[2];
                let bottom = vals[3];
                if top > 0 {
                    log::info!("Bar detected: top={} left={} right={} bottom={}", top, left, right, bottom);
                    return top;
                }
            }
        }
        0
    }

    fn scan_existing_windows(&mut self) -> anyhow::Result<()> {
        let tree = self.conn.query_tree(self.root)?.reply()?;
        for &window in &tree.children {
            let attrs = self.conn.get_window_attributes(window)?.reply()?;
            if attrs.override_redirect || attrs.map_state == MapState::UNMAPPED {
                continue;
            }
            if self.is_dock_window(window)? {
                continue;
            }
            self.manage_window(window)?;
        }
        Ok(())
    }

    fn update_ewmh(&mut self) -> anyhow::Result<()> {
        let client_windows: Vec<u32> = self.clients.iter().map(|c| c.window).collect();
        let stacked: Vec<u32> = self.clients.iter().rev().map(|c| c.window).collect();

        conn_change_property32(
            &self.conn, PropMode::REPLACE, self.root,
            self.atoms.net_client_list, AtomEnum::WINDOW,
            &client_windows,
        )?;
        conn_change_property32(
            &self.conn, PropMode::REPLACE, self.root,
            self.atoms.net_client_list_stacking, AtomEnum::WINDOW,
            &stacked,
        )?;
        self.conn.flush()?;
        Ok(())
    }

    fn manage_window(&mut self, window: Window) -> anyhow::Result<()> {
        let geom = self.conn.get_geometry(window)?.reply()?;
        let scr = &self.conn.setup().roots[self.screen_num];

        let frame = self.conn.generate_id()?;
        let frame_aux = CreateWindowAux::new()
            .event_mask(
                EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::BUTTON_PRESS
                | EventMask::EXPOSURE
            )
            .background_pixel(0x00333333)
            .border_pixel(0x00222222);

        conn_create_window(
            &self.conn, scr.root_depth, frame, self.root,
            geom.x, geom.y, geom.width, geom.height, 0,
            WindowClass::COPY_FROM_PARENT, 0, &frame_aux,
        )?;

        self.conn.reparent_window(window, frame, 0, 0)?;
        self.conn.map_window(frame)?;
        self.conn.map_window(window)?;

        let mut client = Client::new(window, frame);
        client.x = geom.x as i32;
        client.y = geom.y as i32;
        client.width = geom.width as u32;
        client.height = geom.height as u32;

        self.detect_window_type(&mut client)?;

        let idx = self.clients.len();
        self.clients.push(client);

        if self.focus_index.is_none() {
            self.focus_index = Some(idx);
            if let Err(e) = self.clients[idx].focus(&self.conn) {
                log::warn!("Failed to focus new window: {}", e);
            }
        }

        log::info!("Managed window {} -> frame {}", window, frame);
        self.update_ewmh()?;
        self.arrange()?;
        Ok(())
    }

    fn detect_window_type(&self, client: &mut Client) -> anyhow::Result<()> {
        let reply = self.conn.get_property(
            false, client.window,
            self.atoms.net_wm_window_type,
            AtomEnum::ATOM,
            0, 32,
        )?.reply()?;

        if reply.format == 32 {
            let types: &[Atom] = bytemuck::cast_slice(&reply.value);
            for &t in types {
                if t == self.atoms.net_wm_window_type_dialog {
                    client.is_dialog = true;
                    client.floating = true;
                }
            }
        }
        Ok(())
    }

    fn unmanage_window(&mut self, idx: usize) -> anyhow::Result<()> {
        if idx >= self.clients.len() {
            return Ok(());
        }
        let frame = self.clients[idx].frame;
        self.conn.destroy_window(frame)?;
        self.conn.flush()?;
        self.clients.remove(idx);

        if self.clients.is_empty() {
            self.focus_index = None;
            // Ensure focus is on root so key grabs can intercept events
            self.conn.set_input_focus(InputFocus::POINTER_ROOT, self.root, 0u32)?;
            self.conn.flush()?;
        } else {
            let new_focus = self.focus_index.map_or(0, |f| {
                if f >= self.clients.len() { self.clients.len() - 1 } else { f }
            });
            self.focus_index = Some(new_focus);
            if let Err(e) = self.clients[new_focus].focus(&self.conn) {
                log::warn!("Failed to focus after unmanage: {}", e);
            }
        }
        self.update_ewmh()?;
        self.arrange()?;
        Ok(())
    }

    fn arrange(&mut self) -> anyhow::Result<()> {
        let sw = self.conn.setup().roots[self.screen_num].width_in_pixels as i32;
        let sh = self.conn.setup().roots[self.screen_num].height_in_pixels as i32;
        let bar_top = self.bar_top as i32;

        let area = Area { x: 0, y: bar_top, width: sw, height: sh - bar_top };
        let bw = self.config.border_width;
        let placements = self.layout.arrange(&self.clients, &area, self.focus_index);

        for p in &placements {
            if let Some(client) = self.clients.get(p.client_index) {
                if client.fullscreen {
                    self.conn.configure_window(client.frame, &ConfigureWindowAux::new()
                        .x(0).y(0).width(sw as u32).height(sh as u32))?;
                    self.conn.configure_window(client.window, &ConfigureWindowAux::new()
                        .x(0).y(0).width(sw as u32).height(sh as u32))?;
                    continue;
                }
                if let Err(e) = client.resize(&self.conn, p.x, p.y, p.width, p.height, bw) {
                    log::warn!("Failed to resize client: {}", e);
                }
            }
        }
        self.conn.flush()?;
        Ok(())
    }

    fn focus_next(&mut self) -> anyhow::Result<()> {
        if self.clients.len() < 2 { return Ok(()); }
        let next = match self.focus_index {
            Some(i) if i + 1 < self.clients.len() => i + 1,
            _ => 0,
        };
        self.set_focus(next)
    }

    fn focus_prev(&mut self) -> anyhow::Result<()> {
        if self.clients.len() < 2 { return Ok(()); }
        let prev = match self.focus_index {
            Some(0) => self.clients.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.set_focus(prev)
    }

    fn set_focus(&mut self, idx: usize) -> anyhow::Result<()> {
        if idx >= self.clients.len() { return Ok(()); }
        if let Some(old) = self.focus_index {
            if old < self.clients.len() {
                let _ = self.clients[old].unfocus(&self.conn);
            }
        }
        self.focus_index = Some(idx);
        self.clients[idx].focus(&self.conn)?;
        Ok(())
    }

    fn toggle_floating(&mut self) -> anyhow::Result<()> {
        if let Some(idx) = self.focus_index {
            if let Some(client) = self.clients.get_mut(idx) {
                client.floating = !client.floating;
                self.arrange()?;
            }
        }
        Ok(())
    }

    fn swap_with_master(&mut self) -> anyhow::Result<()> {
        if self.clients.len() < 2 { return Ok(()); }
        let focused = self.focus_index.unwrap_or(0);
        if focused == 0 { return Ok(()); }
        self.clients.swap(0, focused);
        self.focus_index = Some(0);
        self.arrange()?;
        Ok(())
    }

    fn close_focused(&mut self) -> anyhow::Result<()> {
        let mut idx_to_remove = None;
        if let Some(idx) = self.focus_index {
            if let Some(client) = self.clients.get(idx) {
                match self.conn.get_window_attributes(client.window)?.reply() {
                    Ok(_) => {
                        client.close(&self.conn, self.atoms.wm_protocols, self.atoms.wm_delete_window)?;
                    }
                    Err(_) => {
                        idx_to_remove = Some(idx);
                    }
                }
            }
        }
        if let Some(idx) = idx_to_remove {
            let _ = self.conn.destroy_window(self.clients[idx].frame);
            self.clients.remove(idx);
            if self.clients.is_empty() {
                self.focus_index = None;
            } else {
                self.focus_index = Some(0);
            }
            self.update_ewmh()?;
            self.arrange()?;
        }
        Ok(())
    }

    fn set_layout(&mut self, mode: LayoutMode) -> anyhow::Result<()> {
        self.layout.mode = mode;
        self.arrange()
    }

    fn handle_map_request(&mut self, ev: MapRequestEvent) -> anyhow::Result<()> {
        if self.clients.iter().any(|c| c.window == ev.window) {
            return Ok(());
        }
        if self.is_dock_window(ev.window)? {
            log::info!("Dock window {} mapped (not managed)", ev.window);
            self.conn.change_window_attributes(ev.window, &ChangeWindowAttributesAux::new()
                .event_mask(EventMask::PROPERTY_CHANGE))?;
            self.conn.map_window(ev.window)?;
            self.bar_top = self.query_strut_top();
            if self.bar_top > 0 {
                self.arrange()?;
            }
            self.conn.flush()?;
            return Ok(());
        }
        self.manage_window(ev.window)
    }

    fn is_dock_window(&self, window: Window) -> anyhow::Result<bool> {
        let reply = self.conn.get_property(
            false, window,
            self.atoms.net_wm_window_type,
            AtomEnum::ATOM,
            0, 32,
        )?.reply()?;
        if reply.format == 32 {
            let types: &[Atom] = bytemuck::cast_slice(&reply.value);
            if types.iter().any(|&t| t == self.atoms.net_wm_window_type_dock) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn handle_configure_request(&mut self, ev: ConfigureRequestEvent) -> anyhow::Result<()> {
        if let Some(client) = self.clients.iter().find(|c| c.window == ev.window) {
            if client.floating {
                let mut aux = ConfigureWindowAux::new();
                let vm = ev.value_mask;
                if (vm & ConfigWindow::X) != ConfigWindow::default() {
                    aux = aux.x(ev.x as i32);
                }
                if (vm & ConfigWindow::Y) != ConfigWindow::default() {
                    aux = aux.y(ev.y as i32);
                }
                if (vm & ConfigWindow::WIDTH) != ConfigWindow::default() {
                    aux = aux.width(ev.width as u32);
                }
                if (vm & ConfigWindow::HEIGHT) != ConfigWindow::default() {
                    aux = aux.height(ev.height as u32);
                }
                if (vm & ConfigWindow::BORDER_WIDTH) != ConfigWindow::default() {
                    aux = aux.border_width(ev.border_width as u32);
                }
                self.conn.configure_window(ev.window, &aux)?;
                self.conn.flush()?;
            }
        } else {
            let mut aux = ConfigureWindowAux::new()
                .x(ev.x as i32).y(ev.y as i32)
                .width(ev.width as u32).height(ev.height as u32)
                .border_width(ev.border_width as u32);
            if ev.sibling != 0 {
                aux = aux.sibling(ev.sibling);
            }
            aux = aux.stack_mode(ev.stack_mode);
            self.conn.configure_window(ev.window, &aux)?;
            self.conn.flush()?;
        }
        Ok(())
    }

    fn handle_destroy(&mut self, ev: DestroyNotifyEvent) -> anyhow::Result<()> {
        if let Some(idx) = self.clients.iter().position(|c| c.window == ev.window) {
            self.unmanage_window(idx)?;
        }
        Ok(())
    }

    fn handle_unmap(&mut self, ev: UnmapNotifyEvent) -> anyhow::Result<()> {
        if let Some(idx) = self.clients.iter().position(|c| c.window == ev.window) {
            self.unmanage_window(idx)?;
        }
        Ok(())
    }

    fn handle_keypress(&mut self, ev: KeyPressEvent) -> anyhow::Result<()> {
        let detail = ev.detail;
        let state: u16 = u16::from(ev.state);
        eprintln!("KEYPRESS kc={} state={}", detail, state);
        log::info!("KeyPress keycode={} state={}", detail, state);
        let action = self.bindings.handle_keypress(detail, ev.state).cloned();
        if let Some(action) = action {
            eprintln!("MATCHED: {:?}", action);
            log::info!("Action matched: {:?}", action);
            self.execute_action(&action)?;
            eprintln!("EXECUTED: {:?}", action);
        } else {
            eprintln!("NO BINDING kc={} state={}", detail, state);
            log::warn!("No binding for keycode={} state={}", detail, state);
        }
        let _ = std::io::stderr().flush();
        Ok(())
    }

    fn handle_button_press(&mut self, ev: ButtonPressEvent) -> anyhow::Result<()> {
        if let Some(idx) = self.clients.iter().position(|c| c.frame == ev.event) {
            self.set_focus(idx)?;
            let state_val: u16 = u16::from(ev.state);
            let mod_val: u16 = u16::from(self.bindings.modifiers);
            if ev.detail == 1 && (state_val & mod_val) != 0 {
                self.toggle_floating()?;
            }
        }
        Ok(())
    }

    fn handle_enter(&mut self, ev: EnterNotifyEvent) -> anyhow::Result<()> {
        if let Some(idx) = self.clients.iter().position(|c| c.frame == ev.event) {
            self.set_focus(idx)?;
        }
        Ok(())
    }

    fn handle_client_message(&mut self, _ev: ClientMessageEvent) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_expose(&mut self, _ev: ExposeEvent) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle_property_notify(&mut self, ev: PropertyNotifyEvent) -> anyhow::Result<()> {
        if ev.atom == self.atoms.net_wm_strut_partial {
            self.bar_top = self.query_strut_top();
            if self.bar_top > 0 {
                self.arrange()?;
            }
            return Ok(());
        }
        if ev.atom != self.atoms.net_wm_window_type {
            return Ok(());
        }
        let idx = self.clients.iter().position(|c| c.window == ev.window);
        let idx = match idx { Some(i) => i, None => return Ok(()) };
        let client = &mut self.clients[idx];
        let reply = self.conn.get_property(
            false, client.window,
            self.atoms.net_wm_window_type,
            AtomEnum::ATOM, 0, 32,
        )?.reply()?;
        if reply.format == 32 {
            let types: &[Atom] = bytemuck::cast_slice(&reply.value);
            for &t in types {
                if t == self.atoms.net_wm_window_type_dialog {
                    client.is_dialog = true;
                    client.floating = true;
                }
            }
        }
        Ok(())
    }

    fn execute_action(&mut self, action: &BindingAction) -> anyhow::Result<()> {
        match action {
            BindingAction::FocusNext => self.focus_next()?,
            BindingAction::FocusPrev => self.focus_prev()?,
            BindingAction::SwapWithMaster | BindingAction::MoveToMaster => self.swap_with_master()?,
            BindingAction::ToggleFloating => self.toggle_floating()?,
            BindingAction::CloseWindow => self.close_focused()?,
            BindingAction::LayoutTiling => self.set_layout(LayoutMode::Tiling)?,
            BindingAction::LayoutFloating => self.set_layout(LayoutMode::Floating)?,
            BindingAction::LayoutMonocle => self.set_layout(LayoutMode::Monocle)?,
            BindingAction::Spawn(cmd) => {
                log::info!("Spawning: {}", cmd);
                match Command::new("sh").args(["-c", cmd.as_str()]).spawn() {
                    Ok(_) => {}
                    Err(e) => log::warn!("Failed to spawn {}: {}", cmd, e),
                }
            }
            BindingAction::ReloadConfig => {
                log::info!("Reloading config...");
                match self.config.reload() {
                    Ok(new_cfg) => {
                        self.config = new_cfg;
                        self.layout.gap = self.config.gap;
                        self.layout.border_width = self.config.border_width;
                        self.layout.master_count = self.config.master_count;
                        self.layout.master_ratio = self.config.master_ratio;
                        if let Err(e) = self.bindings.rebuild(&self.config, &self.conn) {
                            log::error!("Failed to rebuild bindings: {}", e);
                        } else {
                            log::info!("Config reloaded successfully");
                        }
                        let _ = self.arrange();
                    }
                    Err(e) => log::error!("Failed to reload config: {}", e),
                }
            }
            BindingAction::Quit => { log::info!("Quitting expiecustWM"); std::process::exit(0); }
        }
        Ok(())
    }
}

fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
}

fn conn_create_window(
    conn: &RustConnection, depth: u8, wid: Window, parent: Window,
    x: i16, y: i16, w: u16, h: u16, bw: u16,
    class: WindowClass, visual: u32,
    aux: &CreateWindowAux,
) -> anyhow::Result<()> {
    conn.create_window(depth, wid, parent, x, y, w, h, bw, class, visual, aux)?;
    Ok(())
}

fn conn_change_property32(
    conn: &RustConnection,
    mode: PropMode, window: Window,
    property: Atom, type_: AtomEnum,
    data: &[u32],
) -> anyhow::Result<()> {
    let len = data.len() as u32;
    let bytes: Vec<u8> = data.iter().flat_map(|&v| v.to_ne_bytes()).collect();
    conn.change_property(mode, window, property, type_, 32, len, &bytes)?;
    Ok(())
}
