use std::path::Path;

use thumbnails::Thumbnailer;

use crate::errors::AppError;

pub fn generate_thumbnail(file_path: &Path, thumbnail_path: &Path) -> Result<(), AppError> {
    let thumbnailer = Thumbnailer::new(250, 250);
    let thumb = thumbnailer.get(file_path)?;

    thumb.save_with_format(thumbnail_path, image::ImageFormat::Png)?;

    Ok(())
}
