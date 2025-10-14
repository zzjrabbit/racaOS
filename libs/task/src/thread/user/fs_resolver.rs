use alloc::sync::Arc;
use filesystem::{File, FileType, Path, open_file};

#[derive(Debug, Clone)]
pub struct FsResolver {
    root: Path,
    cwd: Path,
}

impl FsResolver {
    pub(crate) fn new(root: Path, cwd: Path) -> Self {
        FsResolver { root, cwd }
    }

    /// Gets the path of the root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Gets the path of the current working directory.
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Sets the current working directory to the given `path`.
    pub fn set_cwd(&mut self, path: Path) {
        self.cwd = self.absolute_path(&path);
    }

    /// Sets the root directory to the given `path`.
    pub fn set_root(&mut self, path: Path) {
        self.root = self.absolute_path(&path);
    }
}

impl FsResolver {
    pub fn open_file(&self, path: &Path) -> Option<Arc<File>> {
        open_file(&self.absolute_path(path))
    }
    
    pub fn create_file(&self, path: &Path, file_type: FileType) -> Option<Arc<File>> {
        let parent = self.open_file(&path.parent()?)?;
        parent.create(path.name(), file_type)
    }
}

impl FsResolver {
    fn absolute_path(&self, path: &Path) -> Path {
        if path.is_absolute() {
            self.root.join(path)
        } else {
            self.root.join(self.cwd()).join(path)
        }
    }
}
