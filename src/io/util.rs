use std::path::Path;

pub(super) fn normalize<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_str().unwrap().to_string().replace("\\", "/")
}
