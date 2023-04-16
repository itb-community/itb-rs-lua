use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};

use crate::io::has_parent::HasParent;
use crate::io::has_path::HasPath;
use crate::io::has_relative_path::HasRelativePath;
use crate::io::has_root::HasRoot;
use crate::io::path_filter::PathFilter;
use crate::io::util::normalize;

#[derive(Debug)]
pub struct File {
    pub path: PathBuf,
}

impl File {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn name_without_extension(&self) -> String {
        self.path.file_stem().unwrap().to_str().unwrap().to_string()
    }

    pub fn extension(&self) -> Option<String> {
        self.path.extension()
            .map(|s| s.to_str().unwrap().to_string())
    }

    pub fn read_to_byte_array(&self) -> std::io::Result<Vec<u8>> {
        if self.exists() {
            std::fs::read(&self.path)
        } else {
            Err(Error::new(ErrorKind::Other, "File doesn't exist"))
        }
    }

    pub fn read_to_string(&self) -> std::io::Result<String> {
        if self.exists() {
            std::fs::read_to_string(&self.path)
        } else {
            Err(Error::new(ErrorKind::Other, "File doesn't exist"))
        }
    }

    pub fn write_string<S: AsRef<str> + AsRef<[u8]>>(&self, content: S) -> std::io::Result<()> {
        let maybe_parent = &self.path.parent();
        if let Some(parent) = maybe_parent {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, content)
    }

    pub fn append_string<S: AsRef<str>>(&self, content: S) -> std::io::Result<()> {
        let maybe_parent = &self.path.parent();
        if let Some(parent) = maybe_parent {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)
            .unwrap();

        file.write(content.as_ref().as_bytes())
            .map(|_| ())
    }

    pub fn write_byte_array(&self, content: Vec<u8>) -> std::io::Result<()> {
        let maybe_parent = &self.path.parent();
        if let Some(parent) = maybe_parent {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, content)
    }

    pub fn copy<P: AsRef<Path>>(&self, destination: &P) -> std::io::Result<()> {
        if PathFilter::is_whitelisted(destination)? {
            let maybe_dest_parent = destination.as_ref().parent();
            if let Some(dest_parent) = maybe_dest_parent {
                std::fs::create_dir_all(dest_parent)?;
            }
            std::fs::copy(&self.path, destination).map(|_| ())
        } else {
            Err(Error::new(ErrorKind::Other, "Destination is not within allowed directory"))
        }
    }

    pub fn move_file<P: AsRef<Path>>(&self, destination: &P) -> std::io::Result<()> {
        if PathFilter::is_whitelisted(destination)? {
            let maybe_dest_parent = destination.as_ref().parent();
            if let Some(dest_parent) = maybe_dest_parent {
                std::fs::create_dir_all(dest_parent)?;
            }
            std::fs::rename(&self.path, destination)
        } else {
            Err(Error::new(ErrorKind::Other, "Destination is not within allowed directory"))
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn delete(&self) -> std::io::Result<()> {
        if self.exists() {
            std::fs::remove_file(&self.path)
        } else {
            Ok(())
        }
    }
}

impl HasPath for File {
    fn path(&self) -> String {
        normalize(&self.path)
    }
}

impl HasParent for File {}

impl HasRoot for File {}

impl HasRelativePath for File {}

impl<P: AsRef<Path>> From<P> for File where PathBuf: From<P> {
    fn from(path: P) -> Self {
        File {
            path: PathBuf::from(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::file::File;
    use crate::io::has_path::HasPath;
    use crate::io::HasRelativePath;

    #[test]
    fn path_should_be_reported_without_trailing_slash() {
        let file = File::from("test.txt");

        assert_eq!("test.txt", file.path());
        assert_eq!("test.txt", file.path.to_str().unwrap())
    }

    #[test]
    fn relative_path_should_be_reported_without_trailing_slash() {
        let tmp_file = tempfile::NamedTempFile::new().unwrap();
        let file = File::from(tmp_file.path());

        assert!(!file.relative_path().unwrap().ends_with("/"));
    }

    #[test]
    fn append_should_create_if_file_does_not_exist() {
        let tmp_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_path = tmp_file.into_temp_path();

        let file = File::from(tmp_path.to_path_buf());
        file.delete().unwrap();
        file.append_string("qwe").unwrap();
        file.append_string("asd").unwrap();
        let result = file.read_to_string().unwrap();
        assert_eq!(result, "qweasd");
    }
}