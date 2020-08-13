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

    #[test]
    fn git2_fetch_smoke_test() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        let upstream = "github.com/jonasmalacofilho/git-cache-http-server";
        let repository = cache.open(upstream).expect("could not open repository");

        // use this copy instead of really fetch from GitHub
        let mut remote = repository.remote_anonymous(".git").unwrap();

        remote
            .fetch(&["master"], None, None)
            .expect("could not fetch from remote");

        let objects = repository.odb().expect("could not get object database");
        let some_commit = "e22660e40203ffe4a3f24ebba616529d92a6d085";

        let short_len = 12;
        let short_id = git2::Oid::from_str(&some_commit[0..short_len]).unwrap();

        assert_eq!(
            objects
                .exists_prefix(short_id, short_len)
                .unwrap()
                .to_string(),
            some_commit
        );
    }
}

/// Mess from previous attempt (will eventually be removed)
pub mod first_attempt;
