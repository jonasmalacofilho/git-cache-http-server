use std::path::{Path, PathBuf};

pub struct Repository {
    local_path: PathBuf,
    upstream_url: String, // FIXME
}

impl Repository {
    pub fn local_path(&self) -> &PathBuf {
        &self.local_path
    }

    pub fn upstream_url(&self) -> &str {
        self.upstream_url.as_str()
    }

    pub fn update(&mut self, _credentials: &Credentials) -> Result<(), GitError> {
        todo!()
    }
}

pub struct Credentials {
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GitError {
    #[error("access not granted by upstream")]
    AccessNotGranted,
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

    pub fn open(&mut self, upstream: &str) -> Result<Repository, anyhow::Error> {
        let mut local_path = self.directory.clone();
        local_path.push(upstream);
        if !matches!(local_path.extension(), Some(ext) if ext.to_str() == Some("git")) {
            local_path.set_extension("git");
        }

        // git init --base <local_path>


        Ok(Repository { local_path, upstream_url: upstream.to_string() })
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

        assert!(repo.local_path().starts_with(dir.path()), "{:?}", repo.local_path());
        assert!(repo.local_path().ends_with("example.com/foo/bar.git"), "{:?}", repo.local_path());

        // let credentials = Credentials::new();

        // let mut repo = cache.open("github.com/jonasmalacofilho/git-cache-http-server").unwrap();
        // assert_eq!(repo.update(&credentials), Ok(()));

        // repo.serve_upload_pack();

        // repo.serve_receive_pack();
    }
}

/// Mess from previous attempt (will eventually be removed)
pub mod first_attempt;
