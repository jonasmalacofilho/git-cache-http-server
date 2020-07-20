use base64;
use hyper::{HeaderMap, Uri};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Error(String);

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub trait ErrorFromDisplay: fmt::Display {
    fn to_error(&self) -> Error {
        Error(self.to_string())
    }
}

impl ErrorFromDisplay for base64::DecodeError {}
impl ErrorFromDisplay for hyper::header::ToStrError {}
impl ErrorFromDisplay for std::string::FromUtf8Error {}
impl ErrorFromDisplay for String {}
impl ErrorFromDisplay for &str {}

impl<T> From<T> for Error
where
    T: ErrorFromDisplay,
{
    fn from(error: T) -> Self {
        error.to_error()
    }
}

const GIT_UPLOAD_PACK: &str = "/git-upload-pack";

#[derive(Debug, PartialEq)]
pub struct UriInfo<'a> {
    repository: &'a str,
    service: &'a str,
}

/// Parses incoming URIs into a remote repository and desired service.
///
/// Currently only `git-upload-pack` is supported: clients may only clone or fetch from this cache.
pub fn parse_uri(u: &Uri) -> Result<UriInfo> {
    const INFO_REFS_SUFIX: &str = "/info/refs";
    let p = u.path().trim_start_matches("/");
    if p.ends_with(GIT_UPLOAD_PACK) {
        // POST /<remote-domain>/<path-to-repository>/git-upload-pack
        Ok(UriInfo {
            repository: p.trim_end_matches(GIT_UPLOAD_PACK),
            service: GIT_UPLOAD_PACK,
        })
    } else if p.ends_with(INFO_REFS_SUFIX) && u.query() == Some("service=git-upload-pack") {
        // GET /<remote-domain>/<path-to-repository>/info/refs?service=git-upload-pack
        Ok(UriInfo {
            repository: p.trim_end_matches(INFO_REFS_SUFIX),
            service: GIT_UPLOAD_PACK,
        })
    } else {
        Err("Unsupported remote service or could not parse the URI".into())
    }
}

#[derive(Debug, PartialEq)]
pub struct BasicAuth {
    raw: String,
    decoded: String,
    colon_position: usize,
}

impl BasicAuth {
    pub fn from_authorization(raw: String) -> Result<BasicAuth> {
        let mut parts = raw.split(" ");
        if parts.next() != Some("Basic") {
            Err("Unsupported authorization type")?
        }
        let creds = parts.next().ok_or("Missing Basic credentials")?;
        let decoded = String::from_utf8(base64::decode(&creds)?.to_vec())?;
        let colon_position = decoded
            .find(':')
            .ok_or("Invalid Basic credentials: no colon")?;
        Ok(BasicAuth {
            raw,
            decoded,
            colon_position,
        })
    }

    pub fn sensitive_raw(&self) -> &String {
        &self.raw
    }

    pub fn sensitive_decoded(&self) -> &String {
        &self.decoded
    }

    /// Return a probably safe string to identify the user.
    ///
    /// Uses a few heuristics to strip OAuth tokens that may have been sent in the user-id portion
    /// of the Basic authentication header.
    pub fn safe_username(&self) -> String {
        let (user, pwd) = self.decoded.split_at(self.colon_position);
        if (user == &pwd[1..] || pwd == ":" || pwd.contains("oauth")) && user.len() > 5 {
            format!("{}...", &user[..5]) // FIXME not guaranteed to be char boundary
        } else {
            user.to_string()
        }
    }
}

/// Parse HTTP Basic Authorization received from the git client.
pub fn parse_authorization(headers: &HeaderMap) -> Result<Option<BasicAuth>> {
    match headers.get(hyper::header::AUTHORIZATION) {
        Some(val) => Ok(Some(BasicAuth::from_authorization(
            val.to_str()?.to_string(),
        )?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_uris() {
        let github_post = "http://localhost/github.com/user/repo/git-upload-pack";
        let github_get = "http://localhost/github.com/user/repo/info/refs?service=git-upload-pack";
        assert_eq!(
            parse_uri(&Uri::from_static(github_post)),
            Ok(UriInfo {
                repository: "github.com/user/repo",
                service: GIT_UPLOAD_PACK
            })
        );
        assert_eq!(
            parse_uri(&Uri::from_static(github_get)),
            Ok(UriInfo {
                repository: "github.com/user/repo",
                service: GIT_UPLOAD_PACK
            })
        );
    }

    #[test]
    fn parses_longer_uris() {
        let post = "http://localhost/domain.com/project/user/repo/git-upload-pack";
        assert_eq!(
            parse_uri(&Uri::from_static(post)),
            Ok(UriInfo {
                repository: "domain.com/project/user/repo",
                service: GIT_UPLOAD_PACK
            })
        );
    }

    #[test]
    fn refuses_invalid_uris() {
        assert!(parse_uri(&Uri::from_static("http://localhost")).is_err());
        assert!(parse_uri(&Uri::from_static("http://localhost/git-upload-pack")).is_err());
        assert!(parse_uri(&Uri::from_static("http://localhost/repo/git-receive-pack")).is_err());
    }

    #[test]
    fn parses_basic_auth() {
        let mut headers = hyper::header::HeaderMap::new();
        let raw = String::from("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
        headers.insert(hyper::header::AUTHORIZATION, raw.parse().unwrap());
        let creds = parse_authorization(&headers).unwrap().unwrap();
        assert_eq!(creds.safe_username(), "Aladdin");
    }

    #[test]
    fn sanitizes_oauth_tokens() {
        let token = "MTQ0NjJkZmQ5OTM2NDE1ZTZjNGZmZjI3";
        let auth = format!("Basic {}", base64::encode(format!("{}:", token)));
        assert_eq!(
            BasicAuth::from_authorization(auth).unwrap().safe_username(),
            "MTQ0N..."
        );
        let auth = format!("Basic {}", base64::encode(format!("{}:{}", token, token)));
        assert_eq!(
            BasicAuth::from_authorization(auth).unwrap().safe_username(),
            "MTQ0N..."
        );
        let auth = format!("Basic {}", base64::encode(format!("{}:oauth", token)));
        assert_eq!(
            BasicAuth::from_authorization(auth).unwrap().safe_username(),
            "MTQ0N..."
        );
    }
}
