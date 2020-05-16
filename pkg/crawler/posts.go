package crawler

import (
	"encoding/xml"
	"fmt"
	"io"
	"net/http"
	"os"
	"path"
	"time"
)

// Posts wraps an XML or JSON object of
// a list of Posts with the Count and
// Offset of posts.
type Posts struct {
	XMLName xml.Name `xml:"posts" json:"-"`

	Posts []*Post `xml:"post" json:"post"`

	Count  int `xml:"count,attr" json:"count"`
	Offset int `xml:"offset,attr" json:"offset"`
}

// Post wraps an XML or JSON object of
// an image post containing various metadata
// about the post.
type Post struct {
	XMLName xml.Name `xml:"post" json:"-"`

	Height        int    `xml:"height,attr" json:"height"`
	Score         int    `xml:"score,attr" json:"score"`
	FileURL       string `xml:"file_url,attr" json:"file_url"`
	ParentId      string `xml:"parent_id,attr" json:"parent_id"`
	SampleUrl     string `xml:"sample_url,attr" json:"sample_url"`
	SampleWidth   int    `xml:"sample_width,attr" json:"sample_width"`
	SampleHeight  int    `xml:"sample_height,attr" json:"sample_height"`
	PreviewUrl    string `xml:"preview_url,attr" json:"preview_url"`
	Rating        string `xml:"rating,attr" json:"rating"`
	Tags          string `xml:"tags,attr" json:"tags"`
	Id            string `xml:"id,attr" json:"id"`
	Width         int    `xml:"width,attr" json:"width"`
	Change        int    `xml:"change,attr" json:"change"`
	Md5           string `xml:"md5,attr" json:"md5"`
	CreatorId     string `xml:"creator_id,attr" json:"creator_id"`
	HasChildren   bool   `xml:"has_children,attr" json:"has_children"`
	CreatedAt     string `xml:"created_at,attr" json:"created_at"`
	Status        string `xml:"status,attr" json:"status"`
	Source        string `xml:"source,attr" json:"source"`
	HasNotes      bool   `xml:"has_notes,attr" json:"has_notes"`
	HasComments   bool   `xml:"has_comments,attr" json:"has_comments"`
	PreviewWidth  int    `xml:"preview_width,attr" json:"preview_width"`
	PreviewHeight int    `xml:"preview_height,attr" json:"preview_height"`
}

// Download tries to get the source image file
// of the posts and tries to write it to the
// specified loc with a generated name assembled
// by UnixCreationDate-PostID-HeightxWidth.ext
func (p *Post) Download(loc string) error {
	res, err := http.Get(p.FileURL)
	if err != nil {
		return err
	}
	defer res.Body.Close()

	fileName := path.Join(loc, p.GetFileName())

	fh, err := os.Create(fileName)
	if err != nil {
		return err
	}
	defer fh.Close()

	_, err = io.Copy(fh, res.Body)
	return err
}

// GetFileName assembles a unique file name for the
// post by following pattern:
// UnixCreationDate-PostID-HeightxWidth.ext
func (p *Post) GetFileName() string {
	timeStamp, _ := time.Parse(time.RubyDate, p.CreatedAt)
	return fmt.Sprintf("%d-%s-%dx%d.%s", timeStamp.Unix(), p.Id, p.Width, p.Height, getFileExt(p.FileURL))
}
