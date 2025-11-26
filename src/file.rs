use std::path::{Path, PathBuf};

#[derive(Debug)]
#[non_exhaustive]
pub enum DirError {
    PathNotExist(String),
    PathNotDir(String),
    PathInvalid(String),
}

/// # Errors
///
/// Can return an error if the given directory path is invalid
pub fn directory_exists(dir: &str) -> Result<PathBuf, DirError> {
    let path = Path::new(dir);
    if path.exists() {
        if path.is_dir() {
            Ok(path.to_path_buf())
        } else {
            Err(DirError::PathNotDir(dir.to_string()))
        }
    } else {
        Err(DirError::PathNotExist(dir.to_string()))
    }
}

/// # Errors
///
/// Can return an error if the given file path is invalid
pub fn create_dest_path(dir: &Path, file: &Path) -> Result<PathBuf, DirError> {
    let mut new_path = PathBuf::from(dir);
    let Some(file) = Path::new(file).file_name() else {
        return Err(DirError::PathInvalid(file.display().to_string()));
    };
    new_path.push(file);
    Ok(new_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directory_exists_test_tmp() {
        let tmpdir = directory_exists("/tmp").unwrap();
        assert_eq!(tmpdir, PathBuf::from("/tmp"))
    }
    #[test]
    fn directory_does_not_exist() {
        match directory_exists("/tmpdirprobablydoesnotexist") {
            Ok(_r) => panic!(),
            Err(_e) => {}
        }
    }
    #[test]
    fn path_valid() {
        let newpath = create_dest_path(Path::new("/tmp"), Path::new("myfile")).unwrap();
        println!("{}", newpath.display());
        assert_eq!(newpath, Path::new("/tmp/myfile"))
    }
    #[test]
    fn path_notvalid() {
        match create_dest_path(Path::new("/tmp"), Path::new("..")) {
            Ok(_) => panic!(),
            Err(_) => {}
        }
    }
}
