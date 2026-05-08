package runneragent

import (
	"archive/zip"
	"bytes"
	"testing"
)

func testSourceArchive(t *testing.T, files map[string]string) []byte {
	t.Helper()
	var buffer bytes.Buffer
	writer := zip.NewWriter(&buffer)
	for name, content := range files {
		file, err := writer.Create(name)
		if err != nil {
			t.Fatalf("create archive file: %v", err)
		}
		if _, err := file.Write([]byte(content)); err != nil {
			t.Fatalf("write archive file: %v", err)
		}
	}
	if err := writer.Close(); err != nil {
		t.Fatalf("close archive: %v", err)
	}
	return buffer.Bytes()
}
