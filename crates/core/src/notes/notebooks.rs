use std::{fs, path::Path};

use unicode_normalization::UnicodeNormalization;

use super::{DEFAULT_NOTEBOOK, Notes, io_err};
use crate::{
    error::{Error, Result},
    index,
};

impl Notes {
    /// Every notebook with its note count, including empty ones.
    pub fn notebooks(&self) -> Result<Vec<(String, i64)>> {
        let counts: std::collections::HashMap<String, i64> =
            index::notebook_counts(&self.conn())?.into_iter().collect();
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|e| io_err(&self.root, e))? {
            let entry = entry.map_err(|e| io_err(&self.root, e))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with('.') && entry.path().is_dir() {
                let count = counts.get(&name).copied().unwrap_or(0);
                out.push((name, count));
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn create_notebook(&self, name: &str) -> Result<()> {
        let name = validate_name(name)?;
        let dir = self.root.join(&name);
        if dir.exists() {
            return Err(Error::NotebookExists);
        }
        fs::create_dir(&dir).map_err(|e| io_err(&dir, e))
    }

    pub fn rename_notebook(&self, name: &str, new_name: &str) -> Result<()> {
        let name = self.existing_notebook(name)?;
        let new_name = validate_name(new_name)?;
        if name == DEFAULT_NOTEBOOK {
            return Err(Error::ProtectedNotebook);
        }
        let new_dir = self.root.join(&new_name);
        if new_dir.exists() {
            return Err(Error::NotebookExists);
        }
        fs::rename(self.root.join(&name), &new_dir).map_err(|e| io_err(&new_dir, e))?;
        let mut conn = self.conn();
        self.reconcile(&mut conn)
    }

    /// Only empty notebooks can be deleted.
    pub fn delete_notebook(&self, name: &str) -> Result<()> {
        let name = self.existing_notebook(name)?;
        if name == DEFAULT_NOTEBOOK {
            return Err(Error::ProtectedNotebook);
        }
        let dir = self.root.join(&name);
        if has_notes(&dir)? {
            return Err(Error::NotebookNotEmpty);
        }
        fs::remove_dir(&dir).map_err(|e| io_err(&dir, e))
    }

    /// Normalized name of a notebook that exists on disk.
    pub(super) fn existing_notebook(&self, name: &str) -> Result<String> {
        let name = validate_name(name)?;
        if self.root.join(&name).is_dir() {
            Ok(name)
        } else {
            Err(Error::NotebookNotFound)
        }
    }
}

fn has_notes(dir: &Path) -> Result<bool> {
    for entry in fs::read_dir(dir).map_err(|e| io_err(dir, e))? {
        let path = entry.map_err(|e| io_err(dir, e))?.path();
        if path.extension().is_some_and(|x| x == "md") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_name(name: &str) -> Result<String> {
    let name: String = name.trim().nfc().collect();
    let bad = name.is_empty()
        || name.starts_with('.')
        || name.contains(['/', '\\'])
        || name.chars().any(char::is_control);
    if bad {
        Err(Error::InvalidName)
    } else {
        Ok(name)
    }
}
