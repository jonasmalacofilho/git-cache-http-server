//! Utilities for manipulating Git repositories.
//!
//! Because libgit2 and the `git2` crate do not yet implement the `git-upload-pack` and
//! `git-receive-pack` services, this module wraps `git` commands.

use crate::Error;
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};
use url::Url;

pub async fn init_bare<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let status = Command::new("git")
        .args(&["init", "--bare", "--quiet"])
        .arg(path.as_ref().as_os_str())
        .status().await
        .map_err(|err| Error::CannotRunGit { reason: err.kind() })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::CouldNotCreate) // FIXME include stdou/stderr
    }
}

pub async fn fetch<P: AsRef<Path>>(path: P, url: &Url, refspec: &str) -> Result<(), Error> {
    let status = Command::new("git")
        .current_dir(path.as_ref().as_os_str())
        .args(&["fetch", "--quiet", url.as_str(), refspec])
        .status().await
        .map_err(|err| Error::CannotRunGit { reason: err.kind() })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::UpdateFailure) // FIXME include stdou/stderr
    }
}

pub fn upload_pack<P: AsRef<Path>>(
    path: P,
    stateless_rpc: bool,
    advertise_refs: bool,
    timeout: u16,
) -> Result<Child, Error> {
    let mut command = Command::new("git-upload-pack");

    if stateless_rpc {
        command.arg("--stateless-rpc");
    }

    if advertise_refs {
        command.arg("--advertise-refs");
    }

    command.arg("--strict");
    command.arg(format!("--timeout={}", timeout));
    command.arg(path.as_ref());

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| Error::CannotRunGit { reason: err.kind() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[tokio::test]
    async fn smoke_test() {
        let local = tempfile::tempdir().unwrap();
        let remote =
            Url::parse("https://github.com/jonasmalacofilho/git-cache-http-server").unwrap(); // FIXME not good for CI tests

        assert_eq!(init_bare(&local).await, Ok(()));

        assert_eq!(fetch(&local, &remote, "+refs/*:refs/*").await, Ok(()));

        let refs_service = upload_pack(&local, false, true, 1).unwrap();
        let refs = refs_service.wait_with_output().await.unwrap();
        assert!(String::from_utf8_lossy(&refs.stdout).contains("HEAD"));
    }
}
