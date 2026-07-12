# r34-crawler

A simple, self-contained, CLI tool to fetch and download images from [rule34.xxx](https://rule34.xxx) **(⚠️ Attention: This site definetly contains NSFW content!)**.

You can download the latest release binaries from the [releases page](https://github.com/zekroTJA/r34-crawler/releases).

Alternatively, you can also install the tool using cargo when you have the Rust toolchain installed.

```
cargo install --git https://github.com/zekroTJA/r34-crawler r34-crawler
```

Then, just open up a terminal _(bash or powershell)_ and execute the binary with the `--help` flag:

```
Usage: r34-crawler.exe [OPTIONS] [TAGS]...

Arguments:
  [TAGS]...  Image tags

Options:
  -U, --user-id <USER_ID>          User account ID [env: R34_USER_ID=]
  -A, --api-key <API_KEY>          User account API key [env: R34_API_KEY=]
  -c, --credentials <CREDENTIALS>  Combined credentials of user ID and API key [env: R34_CREDENTIALS=]
  -o, --output <OUTPUT>            The output directory for downloaded images [env: R34_OUTPUT=] [default: output]
  -l, --limit <LIMIT>              Number of images to be collected [env: R34_LIMIT=]
  -O, --offset <OFFSET>            Offset to be skipped in collected images [env: R34_OFFSET=]
  -a, --after-id <AFTER_ID>        Query posts created after the given post ID [env: R34_AFTER_ID=]
  -p, --page-size <PAGE_SIZE>      The page size used per request when listing images [env: R34_PAGE_SIZE=] [default: 250]
      --overwrite                  Force overwriting already downloaded images [env: R34_OVERWRITE=]
  -w, --workers <WORKERS>          Number of threads used for downloading images in parallel [env: R34_WORKERS=] [default: 4]
  -m, --meta <META>                Store image post metadata in the given file as JSON [env: R34_META=]
  -h, --help                       Print help
  -V, --version                    Print version
```

## Authentication

You need to authenticate against the rule34 API with a User ID and API key. Therefore, you need a user account. If you do not have one already, you can create one on [this page](https://rule34.xxx/index.php?page=account&s=reg). After that, you can get your credentials on [this page](https://rule34.xxx/index.php?page=account&s=options).

You can either pass the value of the `API Access Credentials` field directly using the `--credentials` flag

```
r34-crawler --credentials '&api_key=your_api_key&user_id=your_user_id'
```

... or pass the values separately using the `--user-id` and `--api-key` flags

```
r34-crawler --user-id your_user_id --api-key your_api_key
```

You can also pass these flags via environment variables, just like all other flags of the tool.

```
R34_CREDENTIALS=&api_key=your_api_key&user_id=your_user_id
# or
R34_USER_ID=your_user_id
R34_API_KEY=your_api_key
```

You can also put these values in a `.env` or `secrets.env` file in the current working directory or in your config dir in a folder `r34-crawler`.

Where yo ucan find this directory on your OS, you can look up [here](https://github.com/dirs-dev/dirs-rs#features).

## Performance

You can specify an ammount of `threads` with the `--threads` _(or `-w`)_ flag. That means, if you specify 4 threads, for example, that 4 images will be downloaded in parallel. 4 threads is also the default value, if not further specified, because it yields the best results in my personal tests. Your mileage may vary depending on your system performance and your network speed.

_Tests were executed on a 250 MBit/s downstream. Of course, the speeds are also depending on the image sizes and compression rates as same as the speed of the machine and drives._

| Threads | Rust Version | _(old) Go Version_ |
| ------- | ------------ | ------------------ |
| 1       | 27.888s      | _30.358s_          |
| 2       | 22.794s      | _24.962s_          |
| 4       | 20.889s      | _21.353s_          |
| 8       | 22.362s      | _20.517s_          |
| 16      | 20.379s      | _20.505s_          |

---

© 2026 Ringo Hoffmann
Covered by the MIT Licence.
