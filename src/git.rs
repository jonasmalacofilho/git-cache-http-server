//! Utilities for manipulating Git repositories.
//!
//! Because libgit2 and the `git2` crate do not yet implement the `git-upload-pack` and
//! `git-receive-pack` services, this module wraps `git` commands.
#![allow(dead_code)] // FIXME

use eyre::Result;
use eyre::{eyre, WrapErr};

use versions::Version;

use std::path::Path;
use url::Url;

use std::process::Stdio;
use tokio::process::{Child, Command};

pub async fn version() -> Result<Version> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .await
        .wrap_err("failed to spawn `git`")?;

    if !output.status.success() {
        return Err(eyre!("could not get git version"));
    }

    parse_version(&String::from_utf8_lossy(&output.stdout))
}

fn parse_version(input: &str) -> Result<Version> {
    let input = input.trim();

    input
        .split_whitespace()
        .next_back()
        .and_then(|v| Version::new(&v))
        .ok_or(eyre!("could not parse `git` version from: {}", input))
}

pub async fn init_bare<P: AsRef<Path>>(path: P) -> Result<()> {
    let status = Command::new("git")
        .args(&["init", "--bare", "--quiet"])
        .arg(path.as_ref().as_os_str())
        .status()
        .await
        .wrap_err("failed to spawn `git`")?;

    if !status.success() {
        return Err(eyre!("could not initialize repository"));
    }

    Ok(())
}

pub async fn fetch<P: AsRef<Path>>(path: P, url: &Url, refspec: &str) -> Result<()> {
    let status = Command::new("git")
        .current_dir(path.as_ref().as_os_str())
        .args(&["fetch", "--quiet", url.as_str(), refspec])
        .status()
        .await
        .wrap_err("failed to spawn `git`")?;

    if !status.success() {
        return Err(eyre!("could not fetch from URL"));
    }

    Ok(())
}

pub fn upload_pack<P: AsRef<Path>>(
    path: P,
    stateless_rpc: bool,
    advertise_refs: bool,
    timeout: u16,
) -> Result<Child> {
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
        .wrap_err("failed to spawn `git`")
}

#[cfg(test)]
mod tests {
    use crate::git;
    use tempfile;
    use url::Url;
    use versions::Version;

    #[tokio::test]
    async fn init_fetch_and_upload_smoke_test() {
        let local = tempfile::tempdir().unwrap();
        let remote =
            Url::parse("https://github.com/jonasmalacofilho/git-cache-http-server").unwrap();

        assert_eq!(git::init_bare(&local).await.unwrap(), ());

        assert_eq!(
            git::fetch(&local, &remote, "+refs/*:refs/*").await.unwrap(),
            ()
        );

        let refs_service = git::upload_pack(&local, false, true, 1).unwrap();
        let refs = refs_service.wait_with_output().await.unwrap();
        assert!(String::from_utf8_lossy(&refs.stdout).contains("HEAD"));
    }

    #[tokio::test]
    async fn git_version_smoke_test() {
        let git_version = git::version().await.unwrap();

        assert!(git_version >= Version::new("1.4.0").unwrap());
    }

    #[test]
    fn parses_1_9_and_later_versions() {
        // the version number did not resemble semantic Version until 1.9.0
        // (see: https://github.com/git/git/blob/master/Documentation/howto/maintain-git.txt)

        let input = "git version 2.26.0-rc2\n";

        assert_eq!(
            git::parse_version(input).unwrap(),
            Version::new("2.26.0-rc2").unwrap()
        );
    }

    #[test]
    fn parses_1_4_and_later_versions() {
        // the "git version <number>\n" format has been stable at least since git 1.4.0

        let input = "git version 1.8.3.1\n";

        assert_eq!(
            git::parse_version(input).unwrap(),
            Version::new("1.8.3.1").unwrap()
        );
    }
}
