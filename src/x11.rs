use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub struct Atoms {
    pub wm_protocols: Atom,
    pub wm_delete_window: Atom,
    pub wm_state: Atom,
    pub net_wm_name: Atom,
    pub net_wm_pid: Atom,
    pub net_wm_window_type: Atom,
    pub net_wm_window_type_dialog: Atom,
    pub net_wm_window_type_dock: Atom,
    pub net_wm_state: Atom,
    pub net_wm_state_fullscreen: Atom,
    pub net_active_window: Atom,
    pub net_supported: Atom,
    pub net_supporting_wm_check: Atom,
    pub net_client_list: Atom,
    pub net_client_list_stacking: Atom,
    pub net_number_of_desktops: Atom,
    pub net_current_desktop: Atom,
    pub net_desktop_names: Atom,
    pub net_wm_window_opacity: Atom,
    pub motif_wm_hints: Atom,
    pub net_wm_strut_partial: Atom,
    pub ewmh_atoms: Vec<Atom>,
}

impl Atoms {
    pub fn intern(conn: &RustConnection) -> anyhow::Result<Self> {
        let names = [
            "WM_PROTOCOLS",
            "WM_DELETE_WINDOW",
            "WM_STATE",
            "_NET_WM_NAME",
            "_NET_WM_PID",
            "_NET_WM_WINDOW_TYPE",
            "_NET_WM_WINDOW_TYPE_DIALOG",
            "_NET_WM_WINDOW_TYPE_DOCK",
            "_NET_WM_STATE",
            "_NET_WM_STATE_FULLSCREEN",
            "_NET_ACTIVE_WINDOW",
            "_NET_SUPPORTED",
            "_NET_SUPPORTING_WM_CHECK",
            "_NET_CLIENT_LIST",
            "_NET_CLIENT_LIST_STACKING",
            "_NET_NUMBER_OF_DESKTOPS",
            "_NET_CURRENT_DESKTOP",
            "_NET_DESKTOP_NAMES",
            "_NET_WM_WINDOW_OPACITY",
            "_MOTIF_WM_HINTS",
            "_NET_WM_STRUT_PARTIAL",
        ];

        let mut atoms = Vec::with_capacity(names.len());
        for &name in &names {
            let cookie = conn.intern_atom(false, name.as_bytes())?;
            atoms.push(cookie.reply()?.atom);
        }

        let ewmh_start = 11; // _NET_SUPPORTED
        let ewmh_atoms = atoms[ewmh_start..].to_vec();

        Ok(Self {
            wm_protocols: atoms[0],
            wm_delete_window: atoms[1],
            wm_state: atoms[2],
            net_wm_name: atoms[3],
            net_wm_pid: atoms[4],
            net_wm_window_type: atoms[5],
            net_wm_window_type_dialog: atoms[6],
            net_wm_window_type_dock: atoms[7],
            net_wm_state: atoms[8],
            net_wm_state_fullscreen: atoms[9],
            net_active_window: atoms[10],
            net_supported: atoms[11],
            net_supporting_wm_check: atoms[12],
            net_client_list: atoms[13],
            net_client_list_stacking: atoms[14],
            net_number_of_desktops: atoms[15],
            net_current_desktop: atoms[16],
            net_desktop_names: atoms[17],
            net_wm_window_opacity: atoms[18],
            motif_wm_hints: atoms[19],
            net_wm_strut_partial: atoms[20],
            ewmh_atoms,
        })
    }
}

pub fn setup_ewmh(conn: &RustConnection, screen: &Screen, atoms: &Atoms, ws_count: u32) -> anyhow::Result<()> {
    let root = screen.root;

    let wm_check_window = conn.generate_id()?;
    {
        use x11rb::protocol::xproto::ConnectionExt;
        conn.create_window(
            0, // depth must be 0 for InputOnly windows
            wm_check_window,
            root,
            -1, -1, 1, 1, 0,
            WindowClass::INPUT_ONLY,
            0,
            &CreateWindowAux::new().override_redirect(1u32),
        )?;
    }

    set_prop32(conn, wm_check_window, atoms.net_supporting_wm_check, AtomEnum::WINDOW, &[root])?;

    // Set _NET_WM_NAME with STRING type (only on check window, not root — root triggers DE detection)
    let wm_name_fmt = format!("expiecustwm {} {}", crate::version::VERSION, crate::version::CODENAME);
    set_prop8(conn, wm_check_window, atoms.net_wm_name, AtomEnum::STRING, wm_name_fmt.as_bytes())?;

    // Set _NET_WM_PID
    let pid = std::process::id();
    set_prop32(conn, wm_check_window, atoms.net_wm_pid, AtomEnum::CARDINAL, &[pid])?;

    set_prop32(conn, root, atoms.net_supporting_wm_check, AtomEnum::WINDOW, &[wm_check_window])?;

    let ewmh_atom_ids: Vec<u32> = atoms.ewmh_atoms.iter().map(|a| *a).collect();
    set_prop32(conn, root, atoms.net_supported, AtomEnum::ATOM, &ewmh_atom_ids)?;

    set_prop32(conn, root, atoms.net_client_list, AtomEnum::WINDOW, &[])?;
    set_prop32(conn, root, atoms.net_client_list_stacking, AtomEnum::WINDOW, &[])?;
    set_prop32(conn, root, atoms.net_number_of_desktops, AtomEnum::CARDINAL, &[ws_count])?;
    set_prop32(conn, root, atoms.net_current_desktop, AtomEnum::CARDINAL, &[0])?;

    let desktop_names: Vec<u8> = (1..=ws_count)
        .flat_map(|i| {
            let mut s = i.to_string().into_bytes();
            s.push(0);
            s
        })
        .collect();
    set_prop8(conn, root, atoms.net_desktop_names, AtomEnum::STRING, &desktop_names)?;

    conn.map_window(wm_check_window)?;
    conn.flush()?;
    Ok(())
}

pub fn acquire_wm(conn: &RustConnection, screen: &Screen) -> anyhow::Result<()> {
    let root = screen.root;
    let atom_name = format!("WM_S{}", 0);
    let wm_sn = conn.intern_atom(false, atom_name.as_bytes())?.reply()?.atom;

    conn.set_selection_owner(root, wm_sn, 0u32)?;
    conn.flush()?;

    let owner = conn.get_selection_owner(wm_sn)?.reply()?.owner;
    if owner != root {
        anyhow::bail!("Another WM is already running on screen 0 (selection owner: {})", owner);
    }

    log::info!("Acquired WM_S0 selection");

    let event_mask = EventMask::SUBSTRUCTURE_REDIRECT
        | EventMask::SUBSTRUCTURE_NOTIFY
        | EventMask::BUTTON_PRESS
        | EventMask::KEY_PRESS
        | EventMask::ENTER_WINDOW;

    conn.change_window_attributes(root, &ChangeWindowAttributesAux::new()
        .event_mask(event_mask))?;
    conn.flush()?;
    Ok(())
}

pub fn parse_color(color: &str) -> u32 {
    let color = color.trim_start_matches('#');
    if color.len() == 6 {
        let r = u8::from_str_radix(&color[0..2], 16).unwrap_or(0) as u32;
        let g = u8::from_str_radix(&color[2..4], 16).unwrap_or(0) as u32;
        let b = u8::from_str_radix(&color[4..6], 16).unwrap_or(0) as u32;
        (r << 16) | (g << 8) | b
    } else {
        0x222222
    }
}

pub fn string_to_keysym(s: &str) -> u32 {
    match s {
        "Tab" => 0xff09,
        "Return" => 0xff0d,
        "Escape" => 0xff1b,
        "BackSpace" => 0xff08,
        "Insert" => 0xff63,
        "Delete" => 0xffff,
        "Home" => 0xff50,
        "End" => 0xff57,
        "Prior" => 0xff55,
        "Next" => 0xff56,
        "Left" => 0xff51,
        "Up" => 0xff52,
        "Right" => 0xff53,
        "Down" => 0xff54,
        "Shift" => 0xffe1,
        "Control" => 0xffe3,
        "Mod1" | "Alt" => 0xffe9,
        "Mod2" => 0xffea,
        "Mod3" => 0xffeb,
        "Mod4" | "Super" | "Win" => 0xffeb,
        "Mod5" => 0xffec,
        "F1" => 0xffbe, "F2" => 0xffbf, "F3" => 0xffc0,
        "F4" => 0xffc1, "F5" => 0xffc2, "F6" => 0xffc3,
        "F7" => 0xffc4, "F8" => 0xffc5, "F9" => 0xffc6,
        "F10" => 0xffc7, "F11" => 0xffc8, "F12" => 0xffc9,
        "Print" => 0xff61,
        "XF86AudioLowerVolume" => 0x1008ff11,
        "XF86AudioMute" => 0x1008ff12,
        "XF86AudioRaiseVolume" => 0x1008ff13,
        "XF86AudioMicMute" => 0x1008ffb2,
        "XF86MonBrightnessUp" => 0x1008ff02,
        "XF86MonBrightnessDown" => 0x1008ff03,
        "XF86Bluetooth" => 0x1008ff2e,
        "space" => 0x0020,
        s if s.len() == 1 => s.chars().next().unwrap_or(' ') as u32,
        _ => 0,
    }
}

pub fn mod_string_to_mask(s: &str) -> u16 {
    match s {
        "Shift" => 1 << 0,
        "Control" | "Ctrl" => 1 << 2,
        "Mod1" | "Alt" => 1 << 3,
        "Mod2" => 1 << 4,
        "Mod3" => 1 << 5,
        "Mod4" | "Super" | "Win" => 1 << 6,
        "Mod5" => 1 << 7,
        _ => 0,
    }
}

fn set_prop32(conn: &RustConnection, win: Window, prop: Atom, type_: AtomEnum, data: &[u32]) -> anyhow::Result<()> {
    let bytes: Vec<u8> = data.iter().flat_map(|&v| v.to_ne_bytes()).collect();
    let len = data.len() as u32;
    conn.change_property(PropMode::REPLACE, win, prop, u32::from(type_), 32, len, &bytes)?;
    Ok(())
}

fn set_prop8(conn: &RustConnection, win: Window, prop: Atom, type_: AtomEnum, data: &[u8]) -> anyhow::Result<()> {
    let len = data.len() as u32;
    conn.change_property(PropMode::REPLACE, win, prop, u32::from(type_), 8, len, data)?;
    Ok(())
}


