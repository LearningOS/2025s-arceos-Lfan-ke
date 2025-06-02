use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use alloc::{string::String, vec::Vec};
use alloc::string::ToString;
#[allow(unused_imports)]
use axfs_vfs::{VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType};
use axfs_vfs::{VfsError, VfsResult};
use log::debug;
use spin::RwLock;

use crate::file::FileNode;

/// The directory node in the RAM filesystem.
/// 一个目录树啊
/// It implements [`axfs_vfs::VfsNodeOps`].
pub struct DirNode {
    this: Weak<DirNode>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    children: RwLock<BTreeMap<String, VfsNodeRef>>,
}

impl DirNode {
    pub(super) fn new(parent: Option<Weak<dyn VfsNodeOps>>) -> Arc<Self> {
        Arc::new_cyclic(|this| Self {
            this: this.clone(),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
            children: RwLock::new(BTreeMap::new()),
        })
    }

    pub(super) fn set_parent(&self, parent: Option<&VfsNodeRef>) {
        *self.parent.write() = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
    }

    /// Returns a string list of all entries in this directory.
    pub fn get_entries(&self) -> Vec<String> {
        self.children.read().keys().cloned().collect()
    }

    /// Checks whether a node with the given name exists in this directory.
    pub fn exist(&self, name: &str) -> bool {
        self.children.read().contains_key(name)
    }

    /// Creates a new node with the given name and type in this directory.
    pub fn create_node(&self, name: &str, ty: VfsNodeType) -> VfsResult {
        if self.exist(name) {
            log::error!("AlreadyExists {}", name);
            return Err(VfsError::AlreadyExists);
        }
        let node: VfsNodeRef = match ty {
            VfsNodeType::File => Arc::new(FileNode::new()),
            VfsNodeType::Dir => Self::new(Some(self.this.clone())),
            _ => return Err(VfsError::Unsupported),
        };
        self.children.write().insert(name.into(), node);
        Ok(())
    }

    /// Removes a node by the given name in this directory.
    pub fn remove_node(&self, name: &str) -> VfsResult {
        let mut children = self.children.write();
        let node = children.get(name).ok_or(VfsError::NotFound)?;
        if let Some(dir) = node.as_any().downcast_ref::<DirNode>() {
            if !dir.children.read().is_empty() {
                return Err(VfsError::DirectoryNotEmpty);
            }
        }
        children.remove(name);
        Ok(())
    }
}

impl VfsNodeOps for DirNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_dir(4096, 0))
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.parent.read().upgrade()
    }

    fn lookup(self: Arc<Self>, path: &str) -> VfsResult<VfsNodeRef> {
        let (name, rest) = split_path(path);
        let node = match name {
            "" | "." => Ok(self.clone() as VfsNodeRef),
            ".." => self.parent().ok_or(VfsError::NotFound),
            _ => self
                .children
                .read()
                .get(name)
                .cloned()
                .ok_or(VfsError::NotFound),
        }?;

        if let Some(rest) = rest {
            node.lookup(rest)
        } else {
            Ok(node)
        }
    }

    fn create(&self, path: &str, ty: VfsNodeType) -> VfsResult {
        log::debug!("create {:?} at ramfs: {}", ty, path);
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.create(rest, ty),
                ".." => self.parent().ok_or(VfsError::NotFound)?.create(rest, ty),
                _ => {
                    let subdir = self
                        .children
                        .read()
                        .get(name)
                        .ok_or(VfsError::NotFound)?
                        .clone();
                    subdir.create(rest, ty)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Ok(()) // already exists
        } else {
            self.create_node(name, ty)
        }
    }

    fn remove(&self, path: &str) -> VfsResult {
        log::debug!("remove at ramfs: {}", path);
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                "" | "." => self.remove(rest),
                ".." => self.parent().ok_or(VfsError::NotFound)?.remove(rest),
                _ => {
                    let subdir = self
                        .children
                        .read()
                        .get(name)
                        .ok_or(VfsError::NotFound)?
                        .clone();
                    subdir.remove(rest)
                }
            }
        } else if name.is_empty() || name == "." || name == ".." {
            Err(VfsError::InvalidInput)
        } else {
            self.remove_node(name)
        }
    }

    fn rename(&self, src: &str, dst: &str) -> VfsResult {
        log::debug!("\n\nself: [{:?}]\n\n", self.children.read().keys().cloned().collect::<Vec<_>>());
        let this = self.this.upgrade().unwrap();
        let Ok(node) = this.clone().lookup(src) else {
            return Err(VfsError::NotFound);
        };
        if let Ok(_) = this.clone().lookup(dst) {
            return Err(VfsError::AlreadyExists);
        }

        let (_dst_dir, dst_name) = split_rpath(dst);
        // match dst_dir {
        //     None => {
        //         let mut children = self.children.write();
        //         children.insert(dst_name.to_string(), node);
        //         children.remove(src);
        //     }
        //     Some(prefix) => {
        //         let this = self.parent().expect("xx").as_any()
        //             .downcast_ref::<DirNode>()
        //             .unwrap().this.upgrade().unwrap();
        //         let Ok(dir) = this.clone().lookup(prefix) else {
        //             return Err(VfsError::NotFound);
        //         };
        //         let dir = dir.as_any().downcast_ref::<DirNode>().unwrap();
        //         let mut children = dir.children.write();
        //         children.insert(dst_name.to_string(), node);
        //         let mut children = self.children.write();
        //         children.remove(src);
        //     }
        // }
        let mut children = self.children.write();
        children.insert(dst_name.to_string(), node);
        log::debug!("x..............................xx");
        children.remove(src);
        // log::debug!("\n\nself-after: [{:?}]\n\n", self.children.read().keys().cloned().collect::<Vec<_>>());
        Ok(())
    }

    axfs_vfs::impl_vfs_dir_default! {}
}

/// a/b/c  ->  a Some(b/c)
/// 但是只做了一级
fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        log::debug!("lpp: [{:?}] - [{:?}]", &trimmed_path[..n], Some(&trimmed_path[n + 1..]));
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

fn split_rpath(path: &str) -> (Option<&str>, &str) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.rfind('/').map_or((None, trimmed_path), |n| {
        log::debug!("rpp: [{:?}] - [{:#}]", Some(&trimmed_path[..n]), &trimmed_path[n + 1..]);
        (Some(&trimmed_path[..n]), &trimmed_path[n + 1..])
    })
}
