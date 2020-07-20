use base64;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Method, Request, Response, Server, StatusCode, Uri};
use std::convert::Infallible;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
struct Error(String);

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

trait ErrorFromDisplay: fmt::Display {
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

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "/var/cache/git",
        help = "Location of the git cache",
        name = "path"
    )]
    cache_dir: PathBuf,
    #[structopt(short, long, default_value = "8080", help = "Bind to port")]
    port: u16,
}

const GIT_UPLOAD_PACK: &str = "/git-upload-pack";

#[derive(Debug, PartialEq)]
struct UriInfo<'a> {
    repository: &'a str,
    service: &'a str,
}

/// Parses incoming URIs into a remote repository and desired service.
///
/// Currently only `git-upload-pack` is supported: clients may only clone or fetch from this cache.
fn parse_uri(u: &Uri) -> Result<UriInfo> {
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
struct BasicAuth {
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
fn parse_authorization(headers: &HeaderMap) -> Result<Option<BasicAuth>> {
    match headers.get(hyper::header::AUTHORIZATION) {
        Some(val) => Ok(Some(BasicAuth::from_authorization(
            val.to_str()?.to_string(),
        )?)),
        None => Ok(None),
    }
}

async fn http_service(req: Request<Body>) -> std::result::Result<Response<Body>, Infallible> {
    eprintln!("{} {}", req.method(), req.uri());
    let _auth = dbg!(parse_authorization(req.headers())); // FIXME
    let mut res = Response::new(Body::empty());
    match (req.method(), parse_uri(req.uri())) {
        (&Method::GET, Ok(_)) => {
            *res.body_mut() = Body::from("All good?\n");
        }
        (&Method::POST, Ok(_)) => {
            *res.body_mut() = Body::from("All good?\n");
        }
        _ => {
            *res.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    eprintln!("{} {}  {}", req.method(), req.uri(), res.status());
    Ok(res)
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    // TODO get proxy settings from the environment

    let addr = SocketAddr::from(([127, 0, 0, 1], opt.port));
    let http_service =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(http_service)) });
    let server = Server::bind(&addr).serve(http_service);

    eprintln!("INFO: cache directory: {}", opt.cache_dir.to_string_lossy());
    eprintln!("INFO: port number: {}", server.local_addr().port());

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
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
