use crate::io::HasRoot;

pub trait HasRelativePath: HasRoot {
    fn relative_path(&self) -> std::io::Result<String> {
        let normalized_path_relative_to_root = self.root()?.relativize(&self.path())
            .unwrap_or_else(|| "".to_string());

        Ok(normalized_path_relative_to_root)
    }
}
