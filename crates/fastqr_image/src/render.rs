use std::{fs, io::Cursor, path::Path};

use fastqr_core::{QrCode, encode_bytes, encode_text};
use image::{DynamicImage, ImageBuffer, Rgba};

use crate::{
    RasterError, RasterFormat, RenderOptions,
    format::{infer_format, raster_format_to_image_format},
};

pub fn encode_text_to_image(
    text: &str,
    render: RenderOptions,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, RasterError> {
    let code = encode_text(text, Default::default())?;
    render_to_image(&code, render)
}

pub fn encode_bytes_to_image(
    data: &[u8],
    render: RenderOptions,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, RasterError> {
    let code = encode_bytes(data, Default::default())?;
    render_to_image(&code, render)
}

pub fn render_to_image(
    code: &QrCode,
    render: RenderOptions,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, RasterError> {
    let rgba = render_to_rgba(code, render);
    let size = (code.size() as u32 + render.border * 2) * render.scale;
    ImageBuffer::from_vec(size, size, rgba).ok_or(RasterError::InvalidBuffer)
}

pub fn render_to_rgba(code: &QrCode, render: RenderOptions) -> Vec<u8> {
    let size = code.size() as u32 + render.border * 2;
    let pixels_per_side = size * render.scale;
    let mut rgba = vec![0_u8; pixels_per_side as usize * pixels_per_side as usize * 4];
    fill_rgba(&mut rgba, render.light);

    for y in 0..code.size() {
        for x in 0..code.size() {
            if !code.module(x, y) {
                continue;
            }
            let start_x = (x as u32 + render.border) * render.scale;
            let start_y = (y as u32 + render.border) * render.scale;
            for py in start_y..start_y + render.scale {
                let row = py as usize * pixels_per_side as usize;
                for px in start_x..start_x + render.scale {
                    let offset = (row + px as usize) * 4;
                    rgba[offset..offset + 4].copy_from_slice(&render.dark);
                }
            }
        }
    }
    rgba
}

pub fn write_to_bytes(
    code: &QrCode,
    format: RasterFormat,
    render: RenderOptions,
) -> Result<Vec<u8>, RasterError> {
    let image = DynamicImage::ImageRgba8(render_to_image(code, render)?);
    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, raster_format_to_image_format(format))?;
    Ok(cursor.into_inner())
}

pub fn write_to_path<P: AsRef<Path>>(
    code: &QrCode,
    path: P,
    render: RenderOptions,
) -> Result<(), RasterError> {
    let path = path.as_ref();
    let format = infer_format(path)?;
    let bytes = write_to_bytes(code, format, render)?;
    fs::write(path, bytes).map_err(image::ImageError::IoError)?;
    Ok(())
}

fn fill_rgba(bytes: &mut [u8], rgba: [u8; 4]) {
    for pixel in bytes.chunks_exact_mut(4) {
        pixel.copy_from_slice(&rgba);
    }
}
