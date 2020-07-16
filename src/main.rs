use std::path::PathBuf;
use structopt::StructOpt;

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
    #[structopt(short, long, default_value = "8080", help = "Port to bind to")]
    port: u16,
}

fn main() {
    let opt = Opt::from_args();
    assert!(opt.port > 0, "Zero is not a valid port number");
    println!("INFO: cache directory: {}", opt.cache_dir);
    println!("INFO: port number: {}", opt.port);
}
