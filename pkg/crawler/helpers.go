package crawler

import (
	"errors"
	"os"
	"strings"
)

// createDirIfNotExist creates the provided loc
// directory and sub directories if they do
// not exist.
func createDirIfNotExist(loc string) error {
	s, err := os.Stat(loc)
	if os.IsNotExist(err) {
		return os.MkdirAll(loc, os.ModeDir)
	}

	if !s.IsDir() {
		return errors.New("output path is not a directory")
	}

	return err
}

// getFileExt returns a file extension from
// a fileName.
func getFileExt(fileName string) string {
	i := strings.LastIndex(fileName, ".")
	return fileName[i+1:]
}
