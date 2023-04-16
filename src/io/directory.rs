use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::io::file::File;
use crate::io::has_path::HasPath;
use crate::io::has_relative_path::HasRelativePath;
use crate::io::has_root::HasRoot;
use crate::io::HasParent;
use crate::io::path_filter::PathFilter;
use crate::io::util::normalize;

#[derive(Debug)]
pub struct Directory {
    pub path: PathBuf,
}

impl Directory {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn relativize<P: AsRef<Path>>(&self, path: P) -> Option<String> {
        let is_dir = path.as_ref().is_dir();
        let normalized_path_relative_to_self = pathdiff::diff_paths(path, &self.path)
            .map(|path| {
                if is_dir {
                    normalize(&path) + "/"
                } else {
                    normalize(&path)
                }
            });

        normalized_path_relative_to_self
    }

    pub fn files(&self) -> std::io::Result<Vec<File>> {
        if self.exists() {
            let mut result = Vec::new();

            for entry in WalkDir::new(&self.path)
                .min_depth(1)
                .max_depth(1)
                .follow_links(true)
                .into_iter()
            {
                let entry = entry?;
                if entry.file_type().is_file() {
                    result.push(File::from(entry.path()));
                }
            }

            Ok(result)
        } else {
            Err(Error::new(ErrorKind::Other, "Directory doesn't exist"))
        }
    }

    pub fn directories(&self) -> std::io::Result<Vec<Directory>> {
        if self.exists() {
            let mut result = Vec::new();

            for entry in WalkDir::new(&self.path)
                .min_depth(1)
                .max_depth(1)
                .follow_links(true)
                .into_iter()
            {
                let entry = entry?;
                if entry.file_type().is_dir() {
                    result.push(Directory::from(entry.path()));
                }
            }

            Ok(result)
        } else {
            Err(Error::new(ErrorKind::Other, "Directory doesn't exist"))
        }
    }

    pub fn make_directories(&self) -> std::io::Result<()> {
        if PathFilter::is_whitelisted(&self.path)? {
            std::fs::create_dir_all(&self.path)
        } else {
            Err(Error::new(ErrorKind::Other, "Path does not point to an allowed directory"))
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn is_ancestor<P: AsRef<Path>>(&self, path: P) -> std::io::Result<bool> {
        let path = path.as_ref();
        if path.is_absolute() {
            Ok(path.starts_with(&self.path))
        } else {
            Err(Error::new(ErrorKind::Other, "Not an absolute path"))
        }
    }

    pub fn delete(&self) -> std::io::Result<()> {
        if self.exists() {
            std::fs::remove_dir_all(&self.path)
        } else {
            Ok(())
        }
    }
}

impl HasPath for Directory {
    fn path(&self) -> String {
        // Have directories report their path with a trailing slash, since that's sometimes
        // convenient when working with paths in Lua.
        normalize(&self.path) + "/"
    }
}

impl HasParent for Directory {}

impl HasRoot for Directory {}

impl HasRelativePath for Directory {}

impl<P: AsRef<Path>> From<P> for Directory where PathBuf: From<P> {
    fn from(path: P) -> Self {
        Directory {
            path: PathBuf::from(path)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::io::directory::Directory;
    use crate::io::has_parent::HasParent;
    use crate::io::has_path::HasPath;
    use crate::io::has_relative_path::HasRelativePath;
    use crate::io::path_filter::PathFilter;

    #[test]
    fn path_should_be_reported_with_trailing_slash() {
        let dir = Directory::from("test");

        assert_eq!("test/", dir.path());
        assert_eq!("test", dir.path.to_str().unwrap())
    }

    #[test]
    fn relative_path_should_be_reported_with_trailing_slash() {
        let tmp_dir = tempfile::TempDir::new().unwrap();
        let dir = Directory::from(tmp_dir.path());

        assert!(dir.relative_path().unwrap().ends_with("/"));
    }

    #[test]
    fn relativize_should_return_none_for_path_in_different_root() {
        let dir = Directory::from("test");
        let separate_dir = Directory::from(PathFilter::save_data_directory().unwrap());

        let result = separate_dir.relativize(dir.path);

        assert!(result.is_none());
    }

    #[test]
    fn relativize_should_remove_common_path() {
        let dir = Directory::from("some/path/test");
        let parent_dir = dir.parent().unwrap();

        let result = parent_dir.relativize(dir.path());

        assert!(result.is_some());
        let relative_path = result.unwrap();
        assert!(!relative_path.contains(&parent_dir.path()));
    }

    #[test]
    fn relativize_should_add_parent_shorthands() {
        let dir = Directory::from("some/path/test");
        let result = dir.relativize("some");

        assert!(result.is_some());
        let relative_path = result.unwrap();
        assert_eq!("../..", relative_path);
    }

    #[test]
    fn relativize_should_report_path_with_trailing_slash_if_directory_exists_on_file_system() {
        let tmp_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_dir = tmp_file.path().parent().unwrap();

        let dir = Directory::from(tmp_dir);
        let parent_dir = Directory::from(tmp_dir.parent().unwrap());

        let result = parent_dir.relativize(dir.path());

        assert!(result.is_some());
        let relative_path = result.unwrap();
        assert!(relative_path.ends_with("/"));
    }

    #[test]
    fn is_ancestor_should_return_true_for_child_absolute_path() {
        let dir = Directory::from(PathFilter::game_directory().unwrap().join("some/path"));
        let test_path = dir.path.join("test");
        let result = dir.is_ancestor(test_path);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn is_ancestor_should_return_false_for_non_child_absolute_path() {
        let dir = Directory::from(PathFilter::game_directory().unwrap().join("some/path"));
        let test_path = dir.path.parent().unwrap().join("test");

        let result = dir.is_ancestor(test_path);

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn is_ancestor_should_return_error_for_relative_path() {
        let dir = Directory::from(PathFilter::game_directory().unwrap().join("some/path"));
        let test_path = PathBuf::from("test");

        let result = dir.is_ancestor(test_path);

        assert!(result.is_err());
    }
}
