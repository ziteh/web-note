use std::fs;

use super::{ASSETS_DIR, Notes, TRASH_DIR, io_err};
use crate::{
    error::{Error, Result},
    index,
    store::atomic_write,
};

const HASH_CHARS: usize = 16;

struct ImageType {
    ext: &'static str,
    content_type: &'static str,
    matches: fn(&[u8]) -> bool,
}

const TYPES: [ImageType; 4] = [
    ImageType {
        ext: "png",
        content_type: "image/png",
        matches: |b| b.starts_with(b"\x89PNG\r\n\x1a\n"),
    },
    ImageType {
        ext: "jpg",
        content_type: "image/jpeg",
        matches: |b| b.starts_with(&[0xFF, 0xD8, 0xFF]),
    },
    ImageType {
        ext: "gif",
        content_type: "image/gif",
        matches: |b| b.starts_with(b"GIF8"),
    },
    ImageType {
        ext: "webp",
        content_type: "image/webp",
        matches: |b| b.len() > 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP",
    },
];

impl Notes {
    /// Stores an image under its content hash and returns the file name.
    /// Identical content is stored once.
    pub fn add_asset(&self, bytes: &[u8]) -> Result<String> {
        if bytes.len() as u64 > self.max_asset_bytes {
            return Err(Error::AssetTooLarge);
        }
        let ext = TYPES
            .iter()
            .find(|t| (t.matches)(bytes))
            .map(|t| t.ext)
            .ok_or(Error::UnsupportedAsset)?;
        let name = format!("{}.{ext}", &blake3::hash(bytes).to_hex()[..HASH_CHARS]);
        let path = self.root.join(ASSETS_DIR).join(&name);
        if !path.exists() {
            atomic_write(&path, bytes)?;
        }
        Ok(name)
    }

    /// Assets that no note references, in the index or in the trash.
    pub fn orphan_assets(&self) -> Result<Vec<String>> {
        let mut texts = index::bodies(&self.conn())?;
        for (_, path) in self.trash_files()? {
            texts.push(fs::read_to_string(&path).map_err(|e| io_err(&path, e))?);
        }

        let dir = self.root.join(ASSETS_DIR);
        let mut orphans = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| io_err(&dir, e))? {
            let name = entry.map_err(|e| io_err(&dir, e))?.file_name();
            let name = name.to_string_lossy().into_owned();
            if !texts.iter().any(|t| t.contains(&name)) {
                orphans.push(name);
            }
        }
        orphans.sort();
        Ok(orphans)
    }

    /// Moves orphan assets into the trash instead of deleting them.
    pub fn trash_orphan_assets(&self) -> Result<usize> {
        let orphans = self.orphan_assets()?;
        let target_dir = self.root.join(TRASH_DIR).join(ASSETS_DIR);
        fs::create_dir_all(&target_dir).map_err(|e| io_err(&target_dir, e))?;
        for name in &orphans {
            let target = target_dir.join(name);
            fs::rename(self.root.join(ASSETS_DIR).join(name), &target)
                .map_err(|e| io_err(&target, e))?;
        }
        Ok(orphans.len())
    }

    /// Returns the bytes and content type of an asset.
    pub fn read_asset(&self, name: &str) -> Result<(Vec<u8>, &'static str)> {
        let (stem, ext) = name.rsplit_once('.').ok_or(Error::AssetNotFound)?;
        let content_type = TYPES
            .iter()
            .find(|t| t.ext == ext)
            .map(|t| t.content_type)
            .ok_or(Error::AssetNotFound)?;
        if stem.is_empty() || !stem.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::AssetNotFound);
        }
        let path = self.root.join(ASSETS_DIR).join(name);
        match fs::read(&path) {
            Ok(bytes) => Ok((bytes, content_type)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Error::AssetNotFound),
            Err(e) => Err(io_err(&path, e)),
        }
    }
}
