use eyre::{Result, WrapErr};

use structopt::StructOpt;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;

use git_cache_http_server::first_attempt;

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

    let auth = first_attempt::parse_authorization(req.headers()).expect("failed to parse auth");
    if let Some(basic_auth) = auth {
        eprintln!("requested by {}", basic_auth.safe_username());
    }

    let uri_info = first_attempt::parse_uri(req.uri()).expect("failed to parse request URI");
    eprintln!(
        "requesting service {} from repository {}",
        uri_info.service, uri_info.repository
    );

    Ok(Response::new(Body::empty()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    // TODO get proxy settings from the environment

    let service = make_service_fn(|_| async { Ok::<_, Infallible>(service_fn(http_service)) });
    let addr = SocketAddr::from(([0, 0, 0, 0], opt.port));
    let server = Server::try_bind(&addr)
        .wrap_err_with(|| format!("could not bind to {}", addr))?
        .serve(service);

    eprintln!("INFO: cache directory: {}", opt.cache_dir.to_string_lossy());
    eprintln!("INFO: port number: {}", server.local_addr().port());
    eprintln!("INFO: starting server");

    server.await.wrap_err("server terminated unexpectedly")
}
