use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;

mod api;

#[derive(serde::Deserialize, Debug)]
struct Config {
    token: String,
    host: String,
}


#[derive(structopt::StructOpt, serde::Deserialize, Debug)]
#[structopt(name = "blogctl")]
/// Control nanoblog contents
enum Args {
    #[structopt(name = "list")]
    /// list published posts
    List {
        #[structopt(short = "v")]
        verbose: bool,
    },
    #[structopt(name = "publish")]
    /// publish a new post
    Publish {
        #[structopt(long = "dry-run")]
        dry_run: bool,
        /// Show diff output. Use with --dry-run to preview changes.
        #[structopt(long = "diff")]
        diff: bool,

        #[structopt(name = "file")]
        post: PathBuf,
    },
    #[structopt(name = "remove")]
    /// revoke published post
    Revoke {
        #[structopt(long = "dry-run")]
        dry_run: bool,
        post: String,
    },
}


#[paw::main]
fn main(args: Args) -> Result<(), std::io::Error> {
    let config_file = File::open("/home/mclement/.config/blogctl/config.json")?;
    let buf_reader = BufReader::new(config_file);
    let config: Config = serde_json::from_reader(buf_reader)?;
    eprintln!("{:?}\n{:?}", args, config);

    let client = api::Client::new(config.host, config.token)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::ConnectionRefused))?;

    match args {
        Args::List {verbose} => {
            let posts = client.list(verbose)?;
            println!("{:?}", posts);
        },
        Args::Publish {dry_run, diff, post} => {
            let mut buf = String::new();
            let mut file = File::open(post)?;
            file.read_to_string(&mut buf)?;
            let diff = client.publish(dry_run, diff, &buf)?;
            println!("{:?}", diff);
        },
        Args::Revoke {dry_run, post} => {
            client.revoke(dry_run, &post)?;
        },
    };
    Ok(())
}
