use anyhow::Result;
use clap::Parser;
use crossbeam::channel;
use indicatif::{ProgressBar, ProgressStyle};
use r34_api::client::Client;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use spinoff::{spinners, Color, Spinner};
use std::{
    fs::{self, File},
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
    thread,
};

fn default_num_threads() -> NonZeroUsize {
    NonZeroUsize::new(num_cpus::get()).unwrap()
}

fn page_arg_parser(v: &str) -> core::result::Result<usize, String> {
    clap_num::number_range(v, 1, 1000)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Image tags
    tags: Vec<String>,

    /// The output directory for downloaded images
    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    /// Number of images to be collected
    #[arg(short, long)]
    limit: Option<NonZeroUsize>,

    /// Offset to be skipped in collected images
    #[arg(short = 'O', long)]
    offset: Option<NonZeroUsize>,

    /// Query posts created after the given post ID
    #[arg(short, long)]
    after_id: Option<usize>,

    /// The page size used per request when listing images
    #[arg(short, long, default_value = "250", value_parser = page_arg_parser)]
    page_size: usize,

    /// Force overwriting already downloaded images
    #[arg(long)]
    overwrite: bool,

    /// Number of threads used for downloading images in parallel
    #[arg(short, long, default_value_t = default_num_threads())]
    threads: NonZeroUsize,

    /// Store image post metadata in the given file as JSON
    #[arg(short, long)]
    meta: Option<PathBuf>,
}

enum Message {
    Success,
    Skipped,
    Error(r34_api::errors::Error),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let client = Client::default();

    if !cli.output.exists() {
        fs::create_dir_all(&cli.output)?;
    }

    let page_size = cli.page_size;
    let mut all_posts = vec![];
    let limit = cli.limit.map(|v| v.get()).unwrap_or(usize::MAX);

    let mut spinner = Spinner::new(spinners::Dots, "[0] Collecting posts info ...", Color::Cyan);

    let (start_page, rest_offset) = match cli.offset {
        Some(offset) => {
            let start_page = offset.get() / page_size;
            (start_page, offset.get() - (start_page * page_size))
        }
        None => (0, 0),
    };

    for page in start_page..usize::MAX {
        let mut posts = client.posts(&cli.tags, Some(page_size), Some(page), cli.after_id)?;
        let collected = posts.len();
        all_posts.append(&mut posts);

        if all_posts.len() >= limit + rest_offset {
            all_posts = all_posts[rest_offset..limit + rest_offset].to_vec();
            break;
        }

        spinner.update_text(format!("[{}] Collecting posts info ...", all_posts.len()));

        if collected < page_size {
            break;
        }
    }

    spinner.stop_with_message(&format!("✔️  {} posts collected.", all_posts.len()));

    if let Some(meta_dir) = cli.meta {
        let mut spinner = Spinner::new(spinners::Dots, "Writing meta file ...", Color::Cyan);

        let f = File::create(&meta_dir)
            .map_err(|err| anyhow::anyhow!("failed creating meta file: {err}"))?;
        serde_json::to_writer_pretty(f, &all_posts)
            .map_err(|err| anyhow::anyhow!("failed encoding meta JSON: {err}"))?;

        spinner.stop_with_message(&format!("✔️  Meta file written to {}.", meta_dir.display()));
    }

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
                match client.download_post(&post, &cli.output, cli.overwrite) {
                    Ok(downloaded) => match downloaded {
                        true => tx.send(Message::Success).unwrap(),
                        false => tx.send(Message::Skipped).unwrap(),
                    },
                    Err(err) => tx.send(Message::Error(err)).unwrap(),
                };
            })
    });

    receiver_thread.join().unwrap();

    Ok(())
}
