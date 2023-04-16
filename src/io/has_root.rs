use std::path::PathBuf;
use crate::io::{Directory, HasPath, PathFilter};

pub trait HasRoot: HasPath {
    fn root(&self) -> std::io::Result<Directory> {
        let path = PathBuf::from(self.path());
        let root_path = if path.starts_with(PathFilter::game_directory()?) {
            PathFilter::game_directory()?
        } else {
            PathFilter::save_data_directory()?
        };

        Ok(Directory::from(root_path))
    }
}
