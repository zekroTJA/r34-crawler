package crawler

import (
	"encoding/json"
	"encoding/xml"
	"fmt"
	"log"
	"net/http"
	"net/url"
	"os"
	"path"
	"strings"
	"sync/atomic"

	"github.com/zekroTJA/r34-crawler/pkg/workerpool"
)

const (
	// rootUri is the root HTTP uri of the provider
	rootUri = "https://rule34.xxx/index.php?page=dapi&s=post&q=index&tags=%s&pid=%d&limit=%d"
	// pageSize is the ammount of posts requested per page
	// By definition of the rule34.xxx API, 100 is the maximum page size.
	pageSize = 100
)

// Get tries to fetch a Post object containing all posts of the
// defined page by tags and page size limit.
func Get(tags []string, page, limit int) (*Posts, error) {
	tagsStr := url.QueryEscape(strings.Join(tags, " "))
	res, err := http.Get(fmt.Sprintf(rootUri, tagsStr, page, limit))
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()

	var posts Posts
	err = xml.NewDecoder(res.Body).Decode(&posts)

	return &posts, err
}

// GetAll tries to fetch all posts by dialing through pages
// limited and offset by specified parameters.
// This function returns a channel where each pages result is
// pushed into as []*Post array.
// If a page request fails, the error will be provided in the
// returned error channel.
func GetAll(tags []string, limit, offset int) (<-chan []*Post, <-chan error) {
	cout := make(chan []*Post)
	cerr := make(chan error)

	go getAll(tags, limit, offset, cout, cerr)

	return cout, cerr
}

// GetAllAndSave tries to fetch all images with specified tag limited
// an offset by specified parameters and tries to save them to the
// specified disk location.
// When meta is porvided, a JSON file containing all meta information
// of all collected images is saved on this file location.
// When overwrite is false, only images not existent on the location
// will be downloaded and saved. Else, all images will be downloaded and
// existing ones will be owerwritten.
// Workers defines the ammount of concurrent image downloads.
//
// This function will stop the program on failure.
func GetAllAndSave(tags []string, limit, offset int, loc, meta string, overwrite bool, workers int) {
	cposts, cerr := GetAll(tags, limit, offset)

	allPosts := make([]*Post, 0)

	log.Println("Collecting image info...")
mainLoop:
	for {
		select {

		case err := <-cerr:
			log.Fatalf("Request failed: %s", err.Error())
			break mainLoop

		case posts, ok := <-cposts:
			if !ok {
				break mainLoop
			}
			allPosts = append(allPosts, posts...)
			log.Printf("Collected images: %d", len(allPosts))
		}
	}

	if meta != "" {
		log.Printf("Saving metadata to %s...", meta)
		fh, err := os.Create(meta)
		if err != nil {
			log.Fatalf("Failed creating meta file: %s", err.Error())
		}
		defer fh.Close()

		enc := json.NewEncoder(fh)
		enc.SetIndent("", "  ")
		if err := enc.Encode(allPosts); err != nil {
			log.Fatalf("Failed decoding meta file: %s", err.Error())
		}
	}

	log.Printf("Saving images to %s...", loc)
	if err := createDirIfNotExist(loc); err != nil {
		log.Fatalf("Creating output directory failed: %s", err.Error())
	}

	if limit < 0 {
		limit = len(allPosts)
	}

	allPosts = allPosts[:limit]

	if !overwrite {
		allPosts = filterNotExistingPosts(allPosts, loc)
		log.Printf("Only downloading %d images which are not existing (provide --overwrite flag to bypass this)", len(allPosts))
	}

	downloadWithWorkersBlocking(allPosts, loc, workers)
}

// getAll is a shorthand function for GetAll to be executed
// in a seperate goroutine.
func getAll(tags []string, limit, offset int, cout chan []*Post, cerr chan error) {
	preflight, err := Get(tags, 0, 0)
	if err != nil {
		cerr <- err
		return
	}

	count := preflight.Count - offset
	if count < limit || limit < 0 {
		limit = count
	}

	skipPages := int(offset / pageSize)
	skipImages := offset % pageSize
	pages := limit / pageSize

	if limit%pageSize > 0 {
		pages++
	}

	finished := uint32(skipPages)
	for i := skipPages; i < skipPages+pages; i++ {

		go func(page int) {
			posts, err := Get(tags, page, pageSize)
			if err != nil {
				cerr <- err
			}

			if page == 0 {
				cout <- posts.Posts[skipImages:]
			} else {
				cout <- posts.Posts
			}

			atomic.AddUint32(&finished, 1)

			f := atomic.LoadUint32(&finished)
			if int(f) == skipPages+pages {
				close(cout)
			}
		}(i)
	}
}

// filterNotExistingPosts returns a sub list of posts
// which only contains post which are not present in
// the provided loc directory.
func filterNotExistingPosts(posts []*Post, loc string) []*Post {
	res := make([]*Post, len(posts))

	var i int
	var err error
	for _, p := range posts {
		_, err = os.Stat(path.Join(loc, p.GetFileName()))
		if os.IsNotExist(err) {
			res[i] = p
			i++
		}
	}

	return res[:i]
}

// downloadWithWorkersBlocking tries to download all source images
// of the provided posts to the specified loc directory with
// the specified ammount of download workers.
// This function will block until all download jobs are
// finished.
func downloadWithWorkersBlocking(posts []*Post, loc string, workers int) {
	lPosts := len(posts)

	pool := workerpool.New(workers)

	go func() {
		for {
			<-pool.Results()
		}
	}()

	for i, p := range posts {
		pool.Push(func(id int, params ...interface{}) interface{} {
			post := params[0].(*Post)
			log.Printf("[worker #%d] Get image [%d/%d] %s...",
				id, i+1, lPosts, post.Id)
			if err := post.Download(loc); err != nil {
				log.Printf("Failed download: %s", err.Error())
			}
			return id
		}, p)
	}
	pool.Close()

	pool.WaitBlocking()
}
