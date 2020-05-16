package args

import (
	"github.com/alexflint/go-arg"
)

type Args struct {
	Tags   []string `arg:"positional" help:"image tags to look for"`
	Output string   `arg:"-o" help:"output directory" default:"./output"`
	Meta   string   `arg:"-m" help:"save image meta data to file"`
	Limit  int      `help:"set a maximum limit for images to crawl" default:"-1"`
	Offset int      `help:"set an offset of how many images should be skipped"`
}

func Parse() *Args {
	a := new(Args)
	arg.MustParse(a)
	return a
}
