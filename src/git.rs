//! Utilities for manipulating Git repositories.
//!
//! Because libgit2 and the `git2` crate do not yet implement the `git-upload-pack` and
//! `git-receive-pack` services, this module wraps `git` commands.

use super::Error;
use std::path::Path;
use std::process::Command;

pub fn init_bare<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    let status = Command::new("git")
        .args(&["init", "--bare", "--quiet"])
        .arg(path.as_ref().as_os_str())
        .status()
        .map_err(|_| Error::CouldNotCreate)?; // FIXME include stdout/stderr
    if status.success() {
        Ok(())
    } else {
        Err(Error::CouldNotCreate) // FIXME include stdou/stderr
    }
}

pub fn fetch<P: AsRef<Path>>(path: P, url: &str, refspec: &str) -> Result<(), Error> {
    let status = Command::new("git")
        .current_dir(path.as_ref().as_os_str())
        .args(&["fetch", "--quiet", url, refspec])
        .status()
        .map_err(|_| Error::UpdateFailure)?; // FIXME include stdout/stderr
    if status.success() {
        Ok(())
    } else {
        Err(Error::UpdateFailure) // FIXME include stdou/stderr
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn smoke_test() {
        let local = tempfile::tempdir().unwrap();
        let remote = "https://github.com/jonasmalacofilho/git-cache-http-server";

        assert!(init_bare(&local).is_ok());
        assert!(fetch(&local, remote, "+refs/*:refs/*").is_ok());
    }
}
