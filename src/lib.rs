use git2::{self, Repository};
use std::error::Error;
use std::path::{Path, PathBuf};

pub struct Cache {
    directory: PathBuf,
}

impl Cache {
    pub fn new<P: AsRef<Path>>(directory: P) -> Cache {
        Cache {
            directory: directory.as_ref().to_path_buf(),
        }
    }

    pub fn open(&mut self, upstream: &str) -> Result<Repository, Box<dyn Error>> {
        let mut local_path = self.directory.clone();
        local_path.push(upstream);

        Repository::open(&local_path)
            .or_else(|err| {
                if err.code() == git2::ErrorCode::NotFound {
                    return Repository::init(&local_path);
                }
                Err(err)
            })
            .map_err(|x| x.into())
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

        assert!(repo.path().starts_with(dir.path()));
        assert!(repo.path().ends_with("example.com/foo/bar/.git"));
    }
}

/// Mess from previous attempt (will eventually be removed)
pub mod first_attempt;
