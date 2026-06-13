use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

const DEBUG_WALLPAPER: &[u8] = include_bytes!("../assets/debug_wallpaper.png");

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

    // Negotiate BIG-REQUESTS before sending large PutImage
    let _max = conn.maximum_request_bytes();

    let pixmap = conn.generate_id()?;
    conn.create_pixmap(screen.root_depth, pixmap, screen.root, img_w as u16, img_h as u16)?;

    let gc = conn.generate_id()?;
    conn.create_gc(gc, pixmap, &CreateGCAux::new())?;

    let zdata = rgba_to_zpixmap(&pixels, conn.setup().image_byte_order);

    if let Err(e) = conn.put_image(
        ImageFormat::Z_PIXMAP,
        pixmap,
        gc,
        img_w as u16,
        img_h as u16,
        0, 0, 0,
        screen.root_depth,
        &zdata,
    ) {
        log::error!("Failed to set wallpaper via PutImage: {}. Falling back to solid color.", e);
        conn.free_gc(gc)?;
        set_solid(conn, screen, 0x001a1a2e)?;
        return Ok(());
    }

    conn.change_window_attributes(screen.root, &ChangeWindowAttributesAux::new()
        .background_pixmap(Pixmap::from(pixmap)))?;

    conn.clear_area(true, screen.root, 0, 0, 0, 0)?;

    conn.free_gc(gc)?;
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
