use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Repository {
    local_path: PathBuf,
}

impl Repository {
    pub fn local_path(&self) -> &PathBuf {
        &self.local_path
    }

    pub fn update(&mut self, _credentials: &Credentials) -> Result<(), Error> {
        todo!()
    }
}

pub struct Credentials {}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("local path exists but not a repository")]
    ExistsButNotRepository,

    #[error("could not create local repository")]
    CouldNotCreate,

    #[error("could not update from upstream")]
    UpdateFailure,
}

impl Credentials {
    pub fn new() -> Credentials {
        Credentials {}
    }
}

pub struct Cache {
    directory: PathBuf,
}

impl Cache {
    pub fn new<P: AsRef<Path>>(directory: P) -> Cache {
        Cache {
            directory: directory.as_ref().to_path_buf(),
        }
    }

    /// Open or create an existing local repository to cache `upstream`.
    pub fn open(&mut self, upstream: &str) -> Result<Repository, Error> {
        let mut local_path = self.directory.clone();
        local_path.push(upstream);
        if !matches!(local_path.extension(), Some(ext) if ext.to_str() == Some("git")) {
            local_path.set_extension("git");
        }

        fn init_repository(path: PathBuf) -> Result<Repository, Error> {
            let status = Command::new("git")
                .arg("init")
                .arg("--bare")
                .arg(path.as_os_str())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status() // FIXME need to store stdout/stderr
                .map_err(|_| Error::CouldNotCreate)?;
            if status.success() {
                Ok(Repository { local_path: path })
            } else {
                Err(Error::CouldNotCreate)
            }
        }

        match fs::metadata(&local_path) {
            Ok(x) if x.is_dir() => {
                if local_path.read_dir().unwrap().next().is_none() {
                    init_repository(local_path)
                } else {
                    // let git error later if this isn't a real repository
                    Ok(Repository { local_path })
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => init_repository(local_path),
            _ => Err(Error::CouldNotCreate),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn smoke_test() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        let repo = cache.open("example.com/foo/bar").unwrap();

        assert_eq!(
            repo.local_path().as_os_str(),
            dir.path().join("example.com/foo/bar.git").as_os_str()
        );

        assert!(repo.local_path().join("HEAD").is_file());

        // let credentials = Credentials::new();

        // let mut repo = cache.open("github.com/jonasmalacofilho/git-cache-http-server").unwrap();
        // assert_eq!(repo.update(&credentials), Ok(()));

        // repo.serve_upload_pack();

        // repo.serve_receive_pack();
    }

    #[test]
    fn opens_existing_repository() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        cache.open("example.com/foo/bar.git").unwrap();

        assert!(cache.open("example.com/foo/bar.git").is_ok());
    }

    #[test]
    fn opens_in_emptyt_directory() {
        use std::fs;
        const EXAMPLE_REPOSITORY: &str = "example.com/foo/bar.git";

        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        fs::create_dir_all(dir.path().join(EXAMPLE_REPOSITORY)).unwrap();

        assert!(cache.open(EXAMPLE_REPOSITORY).is_ok());
        assert!(dir.path().join(EXAMPLE_REPOSITORY).join("HEAD").is_file());
    }
}

/// Mess from previous attempt (will eventually be removed)
pub mod first_attempt;
