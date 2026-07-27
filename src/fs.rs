use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub content: String,
    pub absolute: PathBuf,
}

pub trait FileSystem: Default {
    fn read(&self, path: &Path) -> Option<Module>;
}

#[derive(Debug, Default)]
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read(&self, path: &Path) -> Option<Module> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(ToString::to_string)?;
        let absolute = std::fs::canonicalize(path).ok()?;
        let content = std::fs::read_to_string(&absolute).ok()?;
        Some(Module {
            name,
            content,
            absolute,
        })
    }
}
