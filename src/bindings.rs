use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;
use crate::config::{Config, BindingAction};
use crate::x11;

pub struct BindingManager {
    bindings: Vec<(ModMask, u8, BindingAction)>,
    pub modifiers: ModMask,
    root: Window,
}

impl BindingManager {
    pub fn new(config: &Config, conn: &RustConnection) -> anyhow::Result<Self> {
        let root = conn.setup().roots[0].root;
        let mut mgr = Self { bindings: vec![], modifiers: ModMask::M4, root };
        mgr.rebuild(config, conn)?;
        Ok(mgr)
    }

    pub fn rebuild(&mut self, config: &Config, conn: &RustConnection) -> anyhow::Result<()> {
        let mod_mask = ModMask::from(x11::mod_string_to_mask(&config.mod_key));
        let mut bindings = Vec::new();

        let min_kc = conn.setup().min_keycode;
        let max_kc = conn.setup().max_keycode;
        let kc_count = max_kc - min_kc + 1;
        let mapping = conn.get_keyboard_mapping(min_kc, kc_count)?.reply()?;
        let ks_per_kc = mapping.keysyms_per_keycode as usize;

        for kb in &config.keybindings {
            let mut mask: u16 = u16::from(mod_mask);
            for m in &kb.mods {
                let m = x11::mod_string_to_mask(m);
                if m != 0 {
                    mask |= m;
                }
            }

            let keycode = key_str_to_keycode(conn, &kb.key, ks_per_kc)?;
            if let Some(kc) = keycode {
                bindings.push((ModMask::from(mask), kc, kb.action.clone()));
            }
        }

        for &(mask, keycode, _) in &bindings {
            conn.grab_key(true, self.root, mask, keycode, GrabMode::ASYNC, GrabMode::ASYNC)?;
        }
        conn.flush()?;

        self.bindings = bindings;
        self.modifiers = mod_mask;
        Ok(())
    }

    pub fn grab(&self, conn: &RustConnection) {
        for &(mask, keycode, _) in &self.bindings {
            let _ = conn.grab_key(true, self.root, mask, keycode, GrabMode::ASYNC, GrabMode::ASYNC);
        }
        let _ = conn.flush();
    }

    pub fn handle_keypress(&self, detail: u8, state: KeyButMask) -> Option<&BindingAction> {
        let filtered: u16 = u16::from(state) & !(1 << 1 | 1 << 4 | 1 << 5 | 1 << 7);
        for &(mask, keycode, ref action) in &self.bindings {
            let m: u16 = u16::from(mask);
            if keycode == detail && m == filtered {
                return Some(action);
            }
        }
        None
    }
}

fn key_str_to_keycode(conn: &RustConnection, key: &str, ks_per_kc: usize) -> anyhow::Result<Option<u8>> {
    let keysym = x11::string_to_keysym(key);
    if keysym == 0 && key.len() == 1 {
        let c = key.chars().next().unwrap_or(' ');
        return find_keycode(conn, c as u32, ks_per_kc);
    }
    if keysym != 0 {
        return find_keycode(conn, keysym, ks_per_kc);
    }
    Ok(None)
}

fn find_keycode(conn: &RustConnection, keysym: u32, ks_per_kc: usize) -> anyhow::Result<Option<u8>> {
    let min_kc = conn.setup().min_keycode;
    let max_kc = conn.setup().max_keycode;
    let kc_count = max_kc - min_kc + 1;

    let mapping = conn.get_keyboard_mapping(min_kc, kc_count)?.reply()?;
    let keysyms = mapping.keysyms;

    for (i, chunk) in keysyms.chunks(ks_per_kc).enumerate() {
        if chunk.contains(&keysym) {
            return Ok(Some(min_kc + i as u8));
        }
    }
    log::warn!("No keycode found for keysym: 0x{:x}", keysym);
    Ok(None)
}
