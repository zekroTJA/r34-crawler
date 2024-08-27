# r34-crawler

A simple, self-contained, CLI tool to fetch and download images from [rule34.xxx](https://rule34.xxx) **(Attention: This site definetly contains NSFW content!)**.

You can download the latest release binaries from the [releases page](https://github.com/zekroTJA/r34-crawler/releases).

Alternatively, you can also install the tool using cargo when you have the Rust toolchain installed.
```
cargo install --git https://github.com/zekroTJA/r34-crawler r34-crawler
```

Then, just open up a terminal *(bash or powershell)* and execute the binary with the `--help` flag:

```
Usage: r34-crawler.exe [OPTIONS] [TAGS]...

Arguments:
  [TAGS]...  Image tags

Options:
  -o, --output <OUTPUT>        The output directory for downloaded images [default: output]
  -l, --limit <LIMIT>          Number of images to be collected
  -O, --offset <OFFSET>        Offset to be skipped in collected images
  -a, --after-id <AFTER_ID>    Query posts created after the given post ID
  -p, --page-size <PAGE_SIZE>  The page size used per request when listing images [default: 250]
      --overwrite              Force overwriting already downloaded images
  -t, --threads <THREADS>      Number of threads used for downloading images in parallel [default: 4]
  -m, --meta <META>            Store image post metadata in the given file as JSON
  -h, --help                   Print help
  -V, --version                Print version
```

You can specify an ammount of `threads` with the `--threads` *(or `-w`)* flag. That means, if you specify 4 threads, for example, that 4 images will be downloaded in parallel. 4 threads is also the default value, if not further specified, because it yields the best results in my personal tests. Your mileage may vary depending on your system performance and your network speed.

*Tests were executed on a 250 MBit/s downstream. Of course, the speeds are also depending on the image sizes and compression rates as same as the speed of the machine and drives.*

| Threads | Rust Version | *(old) Go Version* |
|---------|--------------|--------------------|
| 1       | 27.888s      | *30.358s*          |
| 2       | 22.794s      | *24.962s*          |
| 4       | 20.889s      | *21.353s*          |
| 8       | 22.362s      | *20.517s*          |
| 16      | 20.379s      | *20.505s*          |

---

© 2024 Ringo Hoffmann (zekro Development)  
Covered by the MIT Licence.