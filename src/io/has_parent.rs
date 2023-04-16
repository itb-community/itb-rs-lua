use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use crate::io::{Directory, PathFilter};
use crate::io::has_path::HasPath;

pub trait HasParent: HasPath {
    fn parent(&self) -> std::io::Result<Directory> {
        let maybe_dir = PathBuf::from(self.path()).parent()
            .map(|parent| Directory::from(parent));

        if let Some(dir) = maybe_dir {
            if PathFilter::is_whitelisted(&dir.path())? {
                Ok(dir)
            } else {
                Err(Error::new(ErrorKind::Other, "Parent is not an allowed directory"))
            }
        } else {
            // We do not allow traversal to root directories, so we should never encounter this case.
            Err(Error::new(ErrorKind::Other, "Directory does not have a parent - this should never happen"))
        }
    }
}
