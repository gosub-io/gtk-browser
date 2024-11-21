use anyhow::{anyhow, Error};
use gtk4::gdk_pixbuf::{Colorspace, Pixbuf};
use log::info;

#[allow(dead_code)]
pub fn convert_to_pixbuf(buf: &[u8]) -> Result<Pixbuf, Error> {
    let Ok(img) = image::load_from_memory(&buf) else {
        info!("Failed to load favicon into buffer (image)");
        return Err(anyhow!("Failed to load favicon into buffer (image)"));
    };

    let rgba_image = img.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();
    if pixels.len() == 0 {
        return Err(anyhow!("Failed to convert image to raw pixels"));
    }

    let pixbuf = Pixbuf::from_mut_slice(
        pixels,
        Colorspace::Rgb,
        true, // Has alpha channel
        8,    // Bits per channel
        width as i32,
        height as i32,
        width as i32 * 4,
    );

    Ok(pixbuf)
}
