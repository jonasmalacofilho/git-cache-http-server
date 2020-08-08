use git_cache_http_server::{parse_authorization, parse_uri};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use structopt::StructOpt;

/// A caching Git HTTP server.
///
/// Serve local mirror repositories over HTTP/HTTPS, updating them as they are requested.
#[derive(StructOpt)]
struct Opt {
    /// Location of the git cache.
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "/var/cache/git",
        name = "path"
    )]
    cache_dir: PathBuf,

    /// Bind to port.
    #[structopt(short, long, default_value = "8080")]
    port: u16,
}

async fn http_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
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
