use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

const DEBUG_WALLPAPER: &[u8] = include_bytes!("../assets/debug_wallpaper.png");

const MAX_PUTIMAGE_BYTES: usize = 260_000;

pub fn set_debug(conn: &RustConnection, screen: &Screen) -> anyhow::Result<()> {
    set_from_png_bytes(conn, screen, DEBUG_WALLPAPER)
}

pub fn set_from_png_bytes(conn: &RustConnection, screen: &Screen, png_data: &[u8]) -> anyhow::Result<()> {
    let img = match image::load_from_memory(png_data) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            log::warn!("Failed to decode wallpaper PNG: {}", e);
            set_solid(conn, screen, 0x001a1a2e)?;
            return Ok(());
        }
    };

    let img_w = img.width();
    let img_h = img.height();
    let pixels = img.into_raw();
    let zdata = rgba_to_zpixmap(&pixels, conn.setup().image_byte_order);

    log::info!("Wallpaper: {}x{} depth={} byte_order={:?} pixels={} zdata={}",
        img_w, img_h, screen.root_depth, conn.setup().image_byte_order,
        pixels.len(), zdata.len());

    // Write wallpaper directly to root window in strips
    {
        let gc = conn.generate_id()?;
        conn.create_gc(gc, screen.root, &CreateGCAux::new()
            .subwindow_mode(SubwindowMode::INCLUDE_INFERIORS))?;

        let stride = img_w as usize * 4;
        let max_rows = MAX_PUTIMAGE_BYTES / stride;
        if max_rows == 0 {
            anyhow::bail!("Image too wide ({} px)", img_w);
        }

        let mut n_strips = 0u32;
        let mut y = 0u32;
        while y < img_h {
            let chunk_h = (img_h - y).min(max_rows as u32);
            let start = y as usize * stride;
            let end = start + chunk_h as usize * stride;
            conn.put_image(
                ImageFormat::Z_PIXMAP,
                screen.root,
                gc,
                img_w as u16,
                chunk_h as u16,
                0, y as i16, 0,
                screen.root_depth,
                &zdata[start..end],
            )?;
            y += chunk_h;
            n_strips += 1;
        }
        conn.free_gc(gc)?;
        log::info!("Wallpaper: wrote {} strips directly to root window", n_strips);
    }

    // Also set as background_pixmap for Expose repaints
    {
        let pixmap = conn.generate_id()?;
        conn.create_pixmap(screen.root_depth, pixmap, screen.root, img_w as u16, img_h as u16)?;
        let pgc = conn.generate_id()?;
        conn.create_gc(pgc, pixmap, &CreateGCAux::new())?;
        let stride = img_w as usize * 4;
        let max_rows = MAX_PUTIMAGE_BYTES / stride;
        let mut n_strips = 0u32;
        let mut y = 0u32;
        while y < img_h {
            let chunk_h = (img_h - y).min(max_rows as u32);
            let start = y as usize * stride;
            let end = start + chunk_h as usize * stride;
            if conn.put_image(
                ImageFormat::Z_PIXMAP,
                pixmap,
                pgc,
                img_w as u16,
                chunk_h as u16,
                0, y as i16, 0,
                screen.root_depth,
                &zdata[start..end],
            ).is_err() {
                log::error!("Failed to upload pixmap strip at y={}", y);
                break;
            }
            y += chunk_h;
            n_strips += 1;
        }
        conn.free_gc(pgc)?;
        log::info!("Wallpaper: uploaded {} strips to pixmap", n_strips);
        conn.change_window_attributes(screen.root, &ChangeWindowAttributesAux::new()
            .background_pixmap(Pixmap::from(pixmap)))?;
        conn.clear_area(true, screen.root, 0, 0, 0, 0)?;
    }

    conn.flush()?;
    Ok(())
}

pub fn set_solid(conn: &RustConnection, screen: &Screen, color: u32) -> anyhow::Result<()> {
    conn.change_window_attributes(screen.root, &ChangeWindowAttributesAux::new()
        .background_pixel(color))?;
    conn.clear_area(true, screen.root, 0, 0, 0, 0)?;
    conn.flush()?;
    Ok(())
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
