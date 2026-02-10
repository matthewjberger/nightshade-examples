use std::path::{Path, PathBuf};

pub struct AssetSource {
    pub name: String,
    pub categories: Vec<Category>,
}

pub struct Category {
    pub name: String,
    pub packs: Vec<Pack>,
}

pub struct Pack {
    pub name: String,
    pub path: PathBuf,
}

pub struct ImageFile {
    pub path: PathBuf,
    pub filename: String,
}

pub fn scan_kenney(root: &str) -> Option<AssetSource> {
    let root_path = Path::new(root);
    if !root_path.exists() {
        return None;
    }

    let mut categories = Vec::new();
    for (name, path) in collect_sorted_dirs(root_path) {
        if matches!(name.as_str(), "Goodies" | "Archive" | "Other") || name.starts_with('.') {
            continue;
        }

        let packs: Vec<Pack> = collect_sorted_dirs(&path)
            .into_iter()
            .map(|(pack_name, pack_path)| Pack {
                name: pack_name,
                path: pack_path,
            })
            .collect();

        if !packs.is_empty() {
            categories.push(Category { name, packs });
        }
    }

    Some(AssetSource {
        name: "Kenney".to_string(),
        categories,
    })
}

pub fn scan_polyhaven(root: &str) -> Option<AssetSource> {
    let root_path = Path::new(root);
    if !root_path.exists() {
        return None;
    }

    let mut hdris = Vec::new();
    let mut textures = Vec::new();
    let mut models = Vec::new();

    for (_dir_name, dir_path) in collect_sorted_dirs(root_path) {
        let info_path = dir_path.join("info.json");
        let (display_name, asset_type) = if let Some(info) = read_polyhaven_info(&info_path) {
            info
        } else {
            continue;
        };

        let pack = Pack {
            name: display_name,
            path: dir_path,
        };

        match asset_type {
            0 => hdris.push(pack),
            1 => textures.push(pack),
            2 => models.push(pack),
            _ => {}
        }
    }

    let mut categories = Vec::new();
    if !hdris.is_empty() {
        categories.push(Category {
            name: "HDRIs".to_string(),
            packs: hdris,
        });
    }
    if !textures.is_empty() {
        categories.push(Category {
            name: "Textures".to_string(),
            packs: textures,
        });
    }
    if !models.is_empty() {
        categories.push(Category {
            name: "Models".to_string(),
            packs: models,
        });
    }

    if categories.is_empty() {
        return None;
    }

    Some(AssetSource {
        name: "Poly Haven".to_string(),
        categories,
    })
}

fn read_polyhaven_info(path: &Path) -> Option<(String, u32)> {
    let data = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&data).ok()?;
    let name = json.get("name")?.as_str()?.to_string();
    let asset_type = json.get("type")?.as_u64()? as u32;
    Some((name, asset_type))
}

pub fn scan_pack_images(pack_path: &Path) -> Vec<ImageFile> {
    let mut images = Vec::new();

    let png_dir = pack_path.join("PNG");
    if png_dir.exists() {
        collect_images_recursive(&png_dir, &mut images);
    } else {
        collect_images_recursive(pack_path, &mut images);
    }

    images.sort_by(|a, b| a.filename.cmp(&b.filename));
    images
}

fn collect_images_recursive(dir: &Path, images: &mut Vec<ImageFile>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|a| a.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_images_recursive(&path, images);
        } else if is_image_file(&path) {
            let filename = entry.file_name().to_string_lossy().to_string();
            images.push(ImageFile { path, filename });
        }
    }
}

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            matches!(lower.as_str(), "png" | "jpg" | "jpeg" | "webp" | "hdr")
        })
        .unwrap_or(false)
}

fn collect_sorted_dirs(dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut dirs: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
        .collect();
    dirs.sort_by(|a, b| a.0.cmp(&b.0));
    dirs
}
