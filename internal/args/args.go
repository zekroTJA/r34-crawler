package args

import (
	"github.com/alexflint/go-arg"
)

type Args struct {
	Tags      []string `arg:"positional" help:"image tags to look for"`
	Output    string   `arg:"-o" help:"output directory" default:"./output"`
	Meta      string   `arg:"-m" help:"file to safe metadata in (JSON file format)"`
	Limit     int      `help:"set a limit for images to be crawled" default:"-1"`
	Offset    int      `help:"set an offset of how many images should be skipped"`
	Overwrite bool     `help:"downloads and overwrites existing images" default:"false"`
}

func Parse() *Args {
	a := new(Args)
	arg.MustParse(a)
	return a
}
