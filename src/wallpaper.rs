use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

use image::imageops;

use crate::x11::{set_prop32, Atoms};

const DEBUG_WALLPAPER: &[u8] = include_bytes!("../assets/debug_wallpaper.png");

pub fn set_debug(conn: &RustConnection, screen: &Screen, atoms: &Atoms) -> anyhow::Result<()> {
    set_from_png_bytes(conn, screen, atoms, DEBUG_WALLPAPER)
}

pub fn set_from_png_bytes(conn: &RustConnection, screen: &Screen, atoms: &Atoms, png_data: &[u8]) -> anyhow::Result<()> {
    let dyn_img = match image::load_from_memory(png_data) {
        Ok(img) => img,
        Err(e) => {
            log::warn!("Failed to decode PNG: {}", e);
            return Ok(());
        }
    };

    let screen_w = screen.width_in_pixels as u32;
    let screen_h = screen.height_in_pixels as u32;
    let scaled = scale_to_fill(&dyn_img.to_rgba8(), screen_w, screen_h);
    let pixels = scaled.into_raw();
    let zdata = rgba_to_zpixmap(&pixels, conn.setup().image_byte_order);
    log::info!("Wallpaper scaled to {}x{}", screen_w, screen_h);

    let pixmap = conn.generate_id()?;
    conn.create_pixmap(screen.root_depth, pixmap, screen.root, screen_w as u16, screen_h as u16)?;
    let pgc = conn.generate_id()?;
    conn.create_gc(pgc, pixmap, &CreateGCAux::new())?;

    let stride = screen_w as usize * 4;
    let max_rows = 260_000 / stride;
    if max_rows == 0 {
        anyhow::bail!("Image too wide ({} px)", screen_w);
    }

    let mut n = 0u32;
    let mut y = 0u32;
    while y < screen_h {
        let chunk_h = (screen_h - y).min(max_rows as u32);
        let start = y as usize * stride;
        let end = start + chunk_h as usize * stride;
        conn.put_image(
            ImageFormat::Z_PIXMAP, pixmap, pgc,
            screen_w as u16, chunk_h as u16,
            0, y as i16, 0, screen.root_depth,
            &zdata[start..end],
        )?;
        y += chunk_h;
        n += 1;
    }
    conn.free_gc(pgc)?;
    log::info!("Uploaded {} strips to pixmap {}", n, pixmap);

    let win = conn.generate_id()?;
    conn.create_window(
        screen.root_depth, win, screen.root,
        0, 0, screen_w as u16, screen_h as u16, 0,
        WindowClass::COPY_FROM_PARENT,
        screen.root_visual,
        &CreateWindowAux::new()
            .background_pixmap(Pixmap::from(pixmap))
            .override_redirect(1u32)
            .event_mask(EventMask::EXPOSURE),
    )?;

    set_prop32(conn, win, atoms.net_wm_window_type, AtomEnum::ATOM, &[atoms.net_wm_window_type_desktop])?;

    conn.map_window(win)?;
    conn.configure_window(win, &ConfigureWindowAux::new().stack_mode(StackMode::BELOW))?;
    conn.flush()?;
    log::info!("Desktop wallpaper window created ({}x{})", screen_w, screen_h);
    Ok(())
}

pub fn set_solid(conn: &RustConnection, screen: &Screen, atoms: &Atoms, color: u32) -> anyhow::Result<()> {
    let screen_w = screen.width_in_pixels as u32;
    let screen_h = screen.height_in_pixels as u32;

    let win = conn.generate_id()?;
    conn.create_window(
        screen.root_depth, win, screen.root,
        0, 0, screen_w as u16, screen_h as u16, 0,
        WindowClass::COPY_FROM_PARENT,
        screen.root_visual,
        &CreateWindowAux::new()
            .background_pixel(color)
            .override_redirect(1u32)
            .event_mask(EventMask::EXPOSURE),
    )?;

    set_prop32(conn, win, atoms.net_wm_window_type, AtomEnum::ATOM, &[atoms.net_wm_window_type_desktop])?;

    conn.map_window(win)?;
    conn.configure_window(win, &ConfigureWindowAux::new().stack_mode(StackMode::BELOW))?;
    conn.flush()?;
    log::info!("Solid desktop wallpaper window created ({}x{}, color=0x{:08x})", screen_w, screen_h, color);
    Ok(())
}

fn scale_to_fill(img: &image::RgbaImage, target_w: u32, target_h: u32) -> image::RgbaImage {
    let (w, h) = img.dimensions();
    let scale = f64::max(target_w as f64 / w as f64, target_h as f64 / h as f64);
    let new_w = (w as f64 * scale).round() as u32;
    let new_h = (h as f64 * scale).round() as u32;
    let mut scaled = imageops::resize(img, new_w, new_h, imageops::Lanczos3);
    let x = (new_w - target_w) / 2;
    let y = (new_h - target_h) / 2;
    imageops::crop(&mut scaled, x, y, target_w, target_h).to_image()
}

fn rgba_to_zpixmap(rgba: &[u8], byte_order: ImageOrder) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgba.len());
    for chunk in rgba.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        if byte_order == ImageOrder::LSB_FIRST {
            out.push(b);
            out.push(g);
            out.push(r);
            out.push(0);
        } else {
            out.push(0);
            out.push(r);
            out.push(g);
            out.push(b);
        }
    }
    out
}
