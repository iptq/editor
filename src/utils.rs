use std::path::{Path, PathBuf};

use anyhow::Result;
use ggez::graphics::Rect;

pub fn fuck_you_windows(
    parent: impl AsRef<Path>,
    name: impl AsRef<str>,
) -> Result<Option<PathBuf>> {
    let parent = parent.as_ref();
    let name = name.as_ref();

    let name_lower = name.to_ascii_lowercase();
    for entry in parent.read_dir()? {
        let entry = entry?;
        let entry_lower = entry.file_name().to_str().unwrap().to_ascii_lowercase();
        if name_lower == entry_lower {
            return Ok(Some(entry.path()));
        }
    }

    Ok(None)
}

pub fn rect_contains(rect: &Rect, x: f32, y: f32) -> bool {
    x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h
}
