use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
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
    #[structopt(name = "unpublish")]
    /// Unpublish published post
    Unpublish {
        #[structopt(long = "dry-run")]
        dry_run: bool,
        post: String,
    },
}


fn load_config() -> Result<Config, std::io::Error> {
    let home = std::env::var("HOME").expect("No home directory detectable, wat");
    let path = format!("{}/.config/blogctl/config.json", home);
    let config_file = File::open(path)?;
    let buf_reader = BufReader::new(config_file);
    let config: Config = serde_json::from_reader(buf_reader)?;
    Ok(config)
}


#[paw::main]
fn main(args: Args) -> Result<(), std::io::Error> {
    let config = load_config()?;
    let client = api::Client::new(config.host, config.token)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    match args {
        Args::List {verbose} => {
            let posts = client
                .list_posts(verbose)
                .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
            for post in posts {
                println!("{}", post);
            }
        },
        Args::Publish {dry_run, diff, post} => {
            let mut buf = String::new();
            let mut file = File::open(post)?;
            file.read_to_string(&mut buf)?;
            let diff = client.publish(&buf, dry_run, diff)?;
            println!("{}", diff);
        },
        Args::Unpublish {dry_run, post} => {
            client.unpublish(&post, dry_run)?;
        },
    };
    Ok(())
}
