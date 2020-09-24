use eyre::{bail, eyre, Result, WrapErr};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tokio::io::AsyncWrite;

use url::Url;

mod git;

pub struct Repository {
    upstream: String,
    local_path: PathBuf,
}

impl Repository {
    pub fn local_path(&self) -> &PathBuf {
        &self.local_path
    }

    pub async fn update(&mut self, _credentials: &Credentials) -> Result<()> {
        let url = format!("https://{}", self.upstream);

        let url = Url::parse(&url).wrap_err("resulting URL is invalid")?;

        git::fetch(&self.local_path, &url, "+refs/*:refs/*").await
    }

    pub async fn refs(
        &self,
        _service: &str,
        timeout: u16,
        writer: &mut (impl AsyncWrite + Unpin),
    ) -> Result<()> {
        let service = git::upload_pack(&self.local_path, true, true, timeout)?;

        let mut reader = service.stdout.ok_or(eyre!("missing stdout"))?;

        let _copied = tokio::io::copy(&mut reader, writer).await?;

        Ok(())
    }
}

pub struct Credentials {}

impl Credentials {
    pub fn new() -> Credentials {
        Credentials {}
    }
}

pub struct Cache {
    directory: PathBuf,
    registry: HashMap<PathBuf, Arc<Mutex<Repository>>>,
}

impl Cache {
    pub fn new<P: AsRef<Path>>(directory: P) -> Cache {
        Cache {
            directory: directory.as_ref().to_path_buf(),
            registry: HashMap::new(),
        }
    }

    /// Open or create an existing local repository to cache `upstream`.
    pub async fn open(&mut self, upstream: &str) -> Result<&Arc<Mutex<Repository>>> {
        let local_path = self.local_path(upstream);

        use std::collections::hash_map::Entry;

        match self.registry.entry(local_path) {
            Entry::Occupied(e) => Ok(e.into_mut()),
            Entry::Vacant(e) => {
                let local_path = e.key();

                match fs::metadata(local_path) {
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        git::init_bare(local_path).await?;
                    }
                    Ok(x) if x.is_dir() => {
                        if local_path.read_dir().unwrap().next().is_none() {
                            git::init_bare(local_path).await?;
                        } else {
                            // assume it's a repository; later git calls will fail if it isn't, but at
                            // least we didn't pollute an unrelated directory
                        }
                    }
                    _ => bail!("directory exists but is not repository: {:?}", local_path),
                }

                let repository = Arc::new(Mutex::new(Repository {
                    upstream: upstream.to_string(),
                    local_path: local_path.clone(),
                }));
                Ok(e.insert(repository))
            }
        }
    }

    fn local_path(&self, upstream: &str) -> PathBuf {
        let mut local_path = self.directory.clone();
        local_path.push(upstream);

        if !matches!(local_path.extension(), Some(ext) if ext.to_str() == Some("git")) {
            local_path.set_extension("git");
        }

        // TODO normalize and validate

        local_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[tokio::test]
    async fn smoke_test() {
        let upstream = "github.com/jonasmalacofilho/git-cache-http-server";

        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        // open a given repository

        let mut repo = cache.open(upstream).await.unwrap().lock().unwrap();

        assert_eq!(
            repo.local_path().as_os_str(),
            dir.path().join(format!("{}.git", upstream)).as_os_str()
        );
        assert!(repo.local_path().join("HEAD").is_file());

        // update the cached copy

        let _credentials = Credentials::new();

        let updated = repo.update(&_credentials).await;

        assert_eq!(updated.unwrap(), ());
        assert!(repo.local_path().join("FETCH_HEAD").is_file());

        // allow cloning/fetching (1/2): get all refs

        let mut buf = vec![];
        repo.refs("git-upload-pack", 300, &mut buf).await.unwrap();
        let refs = std::str::from_utf8(&buf).unwrap();

        assert!(refs.contains("refs/heads/master"));
        assert!(refs.ends_with("0000"));

        // repo.serve_upload_pack();

        // repo.serve_receive_pack();
    }

    #[tokio::test]
    async fn opens_existing_repository() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        cache.open("example.com/foo/bar.git").await.unwrap();

        assert!(cache.open("example.com/foo/bar.git").await.is_ok());
    }

    #[tokio::test]
    async fn opens_in_empty_directory() {
        const EXAMPLE_REPOSITORY: &str = "example.com/foo/bar.git";
        use std::fs;

        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        fs::create_dir_all(dir.path().join(EXAMPLE_REPOSITORY)).unwrap();

        assert!(cache.open(EXAMPLE_REPOSITORY).await.is_ok());
        assert!(dir.path().join(EXAMPLE_REPOSITORY).join("HEAD").is_file());
    }

    #[tokio::test]
    async fn exclusive_access() {
        use std::sync::TryLockError;
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::new(&dir);

        let path = "example.com/foo/bar";

        let client1 = Arc::clone(cache.open(path).await.unwrap());
        let repo = client1.lock().unwrap();

        assert!(matches!(
            cache.open(path).await.unwrap().try_lock(),
            Err(TryLockError::WouldBlock)
        ));

        drop(cache);
        drop(repo);
    }
}

// Global FIXMEs/TODOs:
// - include git output in errors that come from git
// - delete branches and tags from the cache as they are deleted on upstream
// - only accept https URLs since they are the only one we can provide credentials to
// - make sure no server credentials are used as a fallback to missing/invalid user credentials

/// Mess from previous attempt (will eventually be removed)
pub mod first_attempt;
