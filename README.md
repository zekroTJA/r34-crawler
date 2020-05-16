# r34-crawler

> This project was created for self-education purpose because I wanted to try to fetch an XML API and working a bit with worker pools and sync groups. ;)

A simple, self-contained, CLI tool to fetch and download images from [rule34.xxx](https://rule34.xxx) **(Attention: Site definetly contains NSFW content!)**.

You can download the latest build from the automated [**Actions CI Builds**](https://github.com/zekroTJA/r34-crawler/actions).  
Just select the latest succeed build and click on `Build Artifacts`. This will download an archive containing a pre-compiled executable for linux and windows (amd64).

If you want to compile it yourself, folow the instructions [below](#self-compiling).

Then, just open up a console *(bash or powershell)* and execute the binary with the `--help` flag:

```
Usage: r34-crawler [--output OUTPUT] [--meta META] [--limit LIMIT] [--offset OFFSET] [--overwrite] [--workers WORKERS] [TAGS [TAGS ...]]

Positional arguments:
  TAGS                   image tags to look for

Options:
  --output OUTPUT, -o OUTPUT
                         output directory [default: ./output]
  --meta META, -m META   file to safe metadata in (JSON file format)
  --limit LIMIT          set a limit for images to be crawled [default: -1]
  --offset OFFSET        set an offset of how many images should be skipped
  --overwrite            downloads and overwrites existing images [default: false]
  --workers WORKERS, -w WORKERS
                         number of concurrent download workers [default: 5]
  --help, -h             display this help and exit
```

The options and flags are *(hopefully)* self-describing.

You can specify an ammount of `workers` with the `--workers` *(or `-w`)* flag. That means, if you specify 5 workers, for example, that 5 images will be downloaded in parallel. If you have a really slow or instable connection, you should set this to `2` or `1`.

I've tested around with some ammounts of workers and got following results:

*Tests were executed on a 100 MBit/s downstream. Of course, the speeds are also depending on the image sizes and compression rates as same as the speed of the machine and drives.*
| n Workers | t for 200 images | t / image |
|-----------|------------------|-----------|
| 1 | 39,13s | 0,196s |
| 5 | 27,44s | 0,137s |
| 50 | 27,39s | 0,137s |

## Self-Compiling

Of course, you need to have the go compiler toolchain installed.
See: https://golang.org/doc/install

First of all, clone the repository and cd into the source dir:
```
$ git clone https://github.com/zekroTJA/r34-crawler
$ cd r34-crawler
```

Then, compile the source files:
```
$ go build -o bin/r34-crawler cmd/main.go
```

*Go build should atiomatically download all nessecary dependencies. If not, execute `go mod download` before.*

---

© 2020 Ringo Hoffmann (zekro Development)  
Covered by the MIT Licence.