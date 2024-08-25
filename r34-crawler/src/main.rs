use std::{fs, num::NonZeroUsize, path::PathBuf, sync::Arc, thread};

use anyhow::Result;
use clap::Parser;
use crossbeam::channel;
use indicatif::{ProgressBar, ProgressStyle};
use r34_api::client::Client;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use spinoff::{spinners, Color, Spinner};

fn default_num_threads() -> NonZeroUsize {
    NonZeroUsize::new(num_cpus::get()).unwrap()
}

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

    #[arg(short, long, default_value_t = default_num_threads())]
    threads: NonZeroUsize,
}

enum Message {
    Success,
    Error(r34_api::errors::Error),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = Client::default();

    if !cli.output.exists() {
        fs::create_dir_all(&cli.output)?;
    }

    let page_size = cli.page_size.get();
    let mut all_posts = vec![];
    let limit = cli.limit.map(|v| v.get()).unwrap_or(usize::MAX);

    let mut spinner = Spinner::new(spinners::Dots, "[0] Collecting posts info ...", Color::Cyan);

    for page in 0..usize::MAX {
        let mut posts = client.posts(&cli.tags, Some(page_size), Some(page))?;
        let collected = posts.len();
        all_posts.append(&mut posts);

        if all_posts.len() >= limit {
            all_posts = all_posts[..limit].to_vec();
            break;
        }

        spinner.update_text(format!("[{}] Collecting posts info ...", all_posts.len()));

        if collected < page_size {
            break;
        }
    }

    spinner.stop_with_message(&format!("✔️  {} posts collected.", all_posts.len()));

    let (tx, rx) = channel::unbounded::<Message>();

    let total = all_posts.len();
    let output_dir = cli.output.to_string_lossy().to_string();
    let receiver_thread = thread::spawn(move || {
        let pb_style = ProgressStyle::with_template(
            "{spinner:.green} Downloading images ... [{wide_bar:.cyan/dim}] {pos}/{len} ({per_sec}, {eta})",
        )
        .unwrap()
        .progress_chars("#>-");
        let pb = ProgressBar::new(total as u64).with_style(pb_style);

        let mut errors = vec![];
        for msg in rx {
            pb.inc(1);
            if let Message::Error(err) = msg {
                errors.push(err);
            }
        }

        pb.finish_and_clear();

        if errors.is_empty() {
            println!("✔️  All images downloaded successfully to {}.", output_dir)
        } else {
            println!("❌  {} of {} failed to download.", errors.len(), total)
        }
    });

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cli.threads.get())
        .build()?;

    let client = Arc::new(client);

    pool.install(|| {
        all_posts
            .into_par_iter()
            .for_each_with((tx, client), |(tx, client), post| {
                match client.download_post(&post, &cli.output) {
                    Ok(_) => tx.send(Message::Success).unwrap(),
                    Err(err) => tx.send(Message::Error(err)).unwrap(),
                };
            })
    });

    receiver_thread.join().unwrap();

    Ok(())
}
