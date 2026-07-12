use anyhow::Result;
use clap::Parser;
use crossbeam::channel;
use indicatif::{ProgressBar, ProgressStyle};
use r34_api::client::{Client, Credentials, API_ROOT_URL};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use spinoff::{spinners, Color, Spinner};
use std::{
    fs::{self, File},
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
    thread,
};

fn page_arg_parser(v: &str) -> core::result::Result<usize, String> {
    clap_num::number_range(v, 1, 1000)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Image tags
    tags: Vec<String>,

    /// User account ID
    #[arg(
        short = 'U',
        long,
        env = "R34_USER_ID",
        requires = "api_key",
        required_unless_present = "credentials",
        conflicts_with = "credentials"
    )]
    user_id: Option<String>,

    /// User account API key
    #[arg(
        short = 'A',
        long,
        env = "R34_API_KEY",
        requires = "user_id",
        required_unless_present = "credentials",
        conflicts_with = "credentials"
    )]
    api_key: Option<String>,

    /// Combined credentials of user ID and API key
    #[arg(short, long, env = "R34_CREDENTIALS", required_unless_present_any = ["user_id", "api_key"], conflicts_with_all = ["user_id", "api_key"])]
    credentials: Option<String>,

    /// The output directory for downloaded images
    #[arg(short, long, default_value = "output", env = "R34_OUTPUT")]
    output: PathBuf,

    /// Number of images to be collected
    #[arg(short, long, env = "R34_LIMIT")]
    limit: Option<NonZeroUsize>,

    /// Offset to be skipped in collected images
    #[arg(short = 'O', long, env = "R34_OFFSET")]
    offset: Option<NonZeroUsize>,

    /// Query posts created after the given post ID
    #[arg(short, long, env = "R34_AFTER_ID")]
    after_id: Option<usize>,

    /// The page size used per request when listing images
    #[arg(short, long, default_value = "250", value_parser = page_arg_parser, env = "R34_PAGE_SIZE")]
    page_size: usize,

    /// Force overwriting already downloaded images
    #[arg(long, env = "R34_OVERWRITE")]
    overwrite: bool,

    /// Number of threads used for downloading images in parallel
    #[arg(
        short,
        long,
        default_value = "4",
        short_alias = 't',
        alias = "threads",
        env = "R34_WORKERS"
    )]
    workers: NonZeroUsize,

    /// Store image post metadata in the given file as JSON
    #[arg(short, long, env = "R34_META")]
    meta: Option<PathBuf>,
}

enum Message {
    Success,
    Skipped,
    Error(r34_api::errors::Error),
}

fn main() -> Result<()> {
    dotenv::from_filename(".env").ok();
    dotenv::from_filename("secrets.env").ok();
    if let Some(config_dir) = dirs::config_dir() {
        let cfg_dir = config_dir.join("r34-crawler");
        dotenv::from_path(cfg_dir.join(".env")).ok();
        dotenv::from_path(cfg_dir.join("secrets.env")).ok();
    }

    let cli = Cli::parse();

    let credentials: Credentials = match (cli.credentials, cli.user_id, cli.api_key) {
        (Some(credentials), None, None) => credentials.as_str().try_into()?,
        (None, Some(user_id), Some(api_key)) => (user_id, api_key).into(),
        v => panic!("this value combination should not happen - this is a bug: {v:#?}"),
    };
    let client = Client::new(API_ROOT_URL, credentials)?;

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
        .num_threads(cli.workers.get())
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
