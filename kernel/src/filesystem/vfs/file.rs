use alloc::{collections::btree_map::BTreeMap, string::String, sync::Arc};
use spin::RwLock;

use crate::filesystem::{InodeData, InodeOperation, Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    File,
    Directory,
    CharDevice,
    BlockDevice,
}

pub struct File {
    data: Arc<InodeData>,
    inner: RwLock<FileInner>,
    file_type: FileType,
    mount: RwLock<Option<Arc<Self>>>,
}

struct FileInner {
    path: Path,
    children: BTreeMap<String, Arc<File>>,
}

impl File {
    pub(in crate::filesystem) fn new<T>(
        path: Path,
        inode_operation: T,
        file_type: FileType,
    ) -> Arc<Self>
    where
        T: InodeOperation,
    {
        Arc::new(Self {
            data: Arc::new(InodeData::new(inode_operation)),
            file_type,
            inner: RwLock::new(FileInner {
                path,
                children: BTreeMap::new(),
            }),
            mount: RwLock::new(None),
        })
    }
}

impl File {
    pub fn mount(self: &Arc<Self>, mount_point: Arc<Self>) {
        mount_point.mount.write().replace(self.clone());
    }
}

impl File {
    pub fn r#type(&self) -> FileType {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.r#type()
        } else {
            self.file_type
        }
    }

    pub fn name(&self) -> String {
        self.inner.read().path.name()
    }

    pub fn path(&self) -> Path {
        self.inner.read().path.clone()
    }

    fn add_child(&self, child: Arc<Self>) {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.add_child(child);
        } else {
            self.inner
                .write()
                .children
                .insert(child.name(), child.clone());
        }
    }

    fn remove_child(&self, child: Arc<Self>) {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.remove_child(child);
        } else {
            self.inner.write().children.remove(&child.name());
        }
    }

    pub fn get_child(&self, name: &str) -> Option<Arc<Self>> {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.get_child(name)
        } else {
            self.inner.read().children.get(name).cloned()
        }
    }

    pub fn get_children(&self) -> BTreeMap<String, Arc<Self>> {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.get_children()
        } else {
            self.inner.read().children.clone()
        }
    }

    pub fn rename(&self, new_name: &str) {
        let mut inner = self.inner.write();
        let ancestors = inner.path.ancestors();
        let mut new_path = Path::new("");

        for ancestor in ancestors {
            new_path = new_path.join(ancestor);
        }

        new_path = new_path.join(Path::new(new_name));

        inner.path = new_path;
    }

    pub fn create(self: &Arc<Self>, name: String, file_type: FileType) -> Option<Arc<Self>> {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.create(name, file_type)
        } else {
            if self.r#type() == FileType::File {
                return None;
            }

            let child = Arc::new(Self {
                data: self.data.create(name.clone(), file_type)?,
                file_type,
                inner: RwLock::new(FileInner {
                    path: self.path().join(Path::new(&name)),
                    children: BTreeMap::new(),
                }),
                mount: RwLock::new(None),
            });
            self.add_child(child.clone());

            child
                .inner
                .write()
                .children
                .insert(".".into(), child.clone());
            child
                .inner
                .write()
                .children
                .insert("..".into(), self.clone());

            Some(child)
        }
    }

    pub fn remove(&self, name: String) -> Option<()> {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.remove(name)
        } else {
            if self.r#type() == FileType::File {
                return None;
            }

            if let Some(child) = self.get_child(&name) {
                self.remove_child(child);
            }

            Some(())
        }
    }
}

impl File {
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.read_at(offset, buf)
        } else {
            self.data.read_at(offset, buf)
        }
    }
    
    pub fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.write_at(offset, buf)
        } else {
            self.data.write_at(offset, buf)
        }
    }
    
    pub fn len(&self) -> usize {
        if let Some(mount) = self.mount.read().as_ref() {
            mount.len()
        } else {
            self.data.len()
        }
    }
}
