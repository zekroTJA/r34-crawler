use std::{num::NonZeroUsize, path::PathBuf, usize};

use anyhow::Result;
use clap::Parser;
use r34_api::client::Client;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    tags: Vec<String>,

    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    #[arg(short, long)]
    limit: Option<NonZeroUsize>,

    #[arg(short = 'O', long)]
    offset: Option<NonZeroUsize>,

    #[arg(short, long, default_value = "250")]
    page_size: NonZeroUsize,

    #[arg(long)]
    overwrite: bool,

    #[arg(short, long)]
    threads: Option<NonZeroUsize>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = Client::default();

    let page_size = cli.page_size.get();
    let mut all_posts = vec![];
    let limit = cli.limit.map(|v| v.get()).unwrap_or(usize::MAX);

    for page in 0..usize::MAX {
        let mut posts = client.posts(&cli.tags, Some(page_size), Some(page))?;
        let collected = posts.len();
        all_posts.append(&mut posts);

        if all_posts.len() >= limit {
            all_posts = all_posts[..limit].to_vec();
            break;
        }

        if collected < page_size {
            break;
        }
    }

    dbg!(all_posts.len());

    Ok(())
}
