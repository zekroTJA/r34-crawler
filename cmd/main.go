package main

import (
	"log"

	"github.com/zekroTJA/r34-crawler/internal/args"
	"github.com/zekroTJA/r34-crawler/pkg/crawler"
)

func main() {
	params := args.Parse()

	if len(params.Tags) < 1 {
		log.Fatal("At least one tag must be provided")
	}

	if params.Limit == -1 {
		log.Println("WARNING: No limit is set so the crawler will download " +
			"images until no images can be found anymore!")
	}

	crawler.GetAllAndSave(
		params.Tags, params.Limit, params.Offset, params.Output, params.Meta)
}