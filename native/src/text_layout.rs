use std::fs;
use std::sync::Mutex;

use rustybuzz::{Face, UnicodeBuffer};
use walkdir::WalkDir;

struct CachedFace {
    path: String,
    data: Vec<u8>,
}

struct FontCache {
    faces: Vec<CachedFace>,
}

static FONT_CACHE: Mutex<Option<FontCache>> = Mutex::new(None);

fn with_face<F, R>(font_path: &str, index: u32, f: F) -> Option<R>
where
    F: FnOnce(&Face) -> R,
{
    let mut cache = FONT_CACHE.lock().unwrap();
    let cached_idx = cache
        .as_ref()
        .and_then(|c| c.faces.iter().position(|cf| cf.path == font_path));

    let _idx = if let Some(i) = cached_idx {
        let cached = &cache.as_ref().unwrap().faces[i];
        let face = Face::from_slice(&cached.data, index)?;
        return Some(f(&face));
    } else {
        let font_data = fs::read(font_path).ok()?;
        let face = Face::from_slice(&font_data, index)?;
        let result = f(&face);
        if let Some(ref mut c) = *cache {
            c.faces.push(CachedFace {
                path: font_path.to_string(),
                data: font_data,
            });
        } else {
            *cache = Some(FontCache {
                faces: vec![CachedFace {
                    path: font_path.to_string(),
                    data: font_data,
                }],
            });
        }
        return Some(result);
    };
}

#[napi]
pub fn measure_text_width(font_path: String, text: String, font_size: f64, index: i32) -> f64 {
    let result = with_face(&font_path, index as u32, |face| {
        let scale = font_size / face.units_per_em() as f64;
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(&text);
        buffer.set_direction(rustybuzz::Direction::LeftToRight);

        let glyph_buffer = rustybuzz::shape(face, &[], buffer);
        let total_advance: f64 = glyph_buffer
            .glyph_positions()
            .iter()
            .map(|gp| gp.x_advance as f64)
            .sum();

        total_advance * scale
    });

    result.unwrap_or_else(|| text.len() as f64 * font_size * 0.5)
}

#[napi]
pub fn measure_text_widths(
    font_path: String,
    texts: Vec<String>,
    font_size: f64,
    index: i32,
) -> Vec<f64> {
    let result = with_face(&font_path, index as u32, |face| {
        let scale = font_size / face.units_per_em() as f64;
        let mut results = Vec::with_capacity(texts.len());

        for text in &texts {
            let mut buffer = UnicodeBuffer::new();
            buffer.push_str(text);
            buffer.set_direction(rustybuzz::Direction::LeftToRight);

            let glyph_buffer = rustybuzz::shape(face, &[], buffer);
            let total_advance: f64 = glyph_buffer
                .glyph_positions()
                .iter()
                .map(|gp| gp.x_advance as f64)
                .sum();

            results.push(total_advance * scale);
        }

        results
    });

    result.unwrap_or_else(|| {
        texts
            .iter()
            .map(|t| t.len() as f64 * font_size * 0.5)
            .collect()
    })
}

fn font_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    if cfg!(target_os = "windows") {
        dirs.push("C:\\Windows\\Fonts".into());
    }
    if cfg!(target_os = "linux") {
        dirs.push("/usr/share/fonts".into());
        dirs.push("/usr/local/share/fonts".into());
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(format!("{}/.fonts", home));
            dirs.push(format!("{}/.local/share/fonts", home));
        }
    }
    if cfg!(target_os = "macos") {
        dirs.push("/System/Library/Fonts".into());
        dirs.push("/Library/Fonts".into());
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(format!("{}/Library/Fonts", home));
        }
    }
    dirs
}

fn clean_font_name(name: &str) -> String {
    name.trim_matches('\'')
        .trim_matches('"')
        .trim()
        .to_lowercase()
        .replace(' ', "")
}

fn find_font_file(font_name: &str, dir: &str) -> Option<String> {
    let clean = clean_font_name(font_name);
    for entry in WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if let Some(ext) = p.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if ext != "ttf" && ext != "otf" && ext != "ttc" {
                continue;
            }
            let stem = p.file_stem()?.to_string_lossy().to_lowercase();
            if stem.contains(&clean) || clean.contains(&stem) {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }
    None
}

#[napi]
pub fn get_font_path(font_name: String) -> Option<String> {
    for dir in font_dirs() {
        if std::path::Path::new(&dir).exists() {
            if let Some(path) = find_font_file(&font_name, &dir) {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_font_name() {
        assert_eq!(clean_font_name("'Consolas'"), "consolas");
        assert_eq!(clean_font_name("\"Courier New\""), "couriernew");
        assert_eq!(clean_font_name("monospace"), "monospace");
    }

    #[test]
    fn test_font_dirs_not_empty() {
        let dirs = font_dirs();
        assert!(!dirs.is_empty());
    }

    #[test]
    fn test_get_font_path_consolas() {
        let result = get_font_path("Consolas".to_string());
        if cfg!(target_os = "windows") {
            assert!(result.is_some(), "Consolas should be found on Windows");
            if let Some(p) = result {
                assert!(p.to_lowercase().contains("consola"));
            }
        }
    }
}
