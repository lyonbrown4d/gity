package runneragent

import (
	"archive/zip"
	"bytes"
	"fmt"
	"io"
	"os"
	"path"
	"path/filepath"
	"strings"

	"github.com/samber/oops"
)

func ExtractSourceArchive(content []byte, workDir string) (err error) {
	reader, err := zip.NewReader(bytes.NewReader(content), int64(len(content)))
	if err != nil {
		return fmt.Errorf("open source archive: %w", err)
	}
	root, err := filepath.Abs(workDir)
	if err != nil {
		return fmt.Errorf("resolve source archive target: %w", err)
	}
	rootHandle, err := os.OpenRoot(root)
	if err != nil {
		return fmt.Errorf("open source archive target: %w", err)
	}
	defer func() {
		if closeErr := rootHandle.Close(); closeErr != nil && err == nil {
			err = oops.In("runner_agent").With("root", root).Wrapf(closeErr, "close source archive target")
		}
	}()
	for _, file := range reader.File {
		relative, err := archiveTargetPath(file.Name)
		if err != nil {
			return err
		}
		info := file.FileInfo()
		if info.IsDir() {
			if mkdirErr := rootHandle.MkdirAll(relative, 0o750); mkdirErr != nil {
				return fmt.Errorf("create source directory: %w", mkdirErr)
			}
			continue
		}
		if !info.Mode().IsRegular() {
			continue
		}
		if parent := path.Dir(relative); parent != "." {
			if mkdirErr := rootHandle.MkdirAll(parent, 0o750); mkdirErr != nil {
				return fmt.Errorf("create source parent directory: %w", mkdirErr)
			}
		}
		rc, err := file.Open()
		if err != nil {
			return fmt.Errorf("open source archive entry: %w", err)
		}
		if err := writeArchiveFile(rootHandle, relative, rc, archiveFileMode(info.Mode().Perm())); err != nil {
			if closeErr := rc.Close(); closeErr != nil {
				return oops.In("runner_agent").With("target", relative).Wrapf(oops.Join(err, closeErr), "write source archive file and close archive entry")
			}
			return oops.In("runner_agent").With("target", relative).Wrapf(err, "write source archive file")
		}
		if err := rc.Close(); err != nil {
			return fmt.Errorf("close source archive entry: %w", err)
		}
	}
	return nil
}

func archiveTargetPath(name string) (string, error) {
	normalized := strings.Trim(strings.ReplaceAll(name, "\\", "/"), "/")
	if normalized == "" || normalized == "." || strings.HasPrefix(normalized, "../") || strings.Contains(normalized, "/../") || filepath.IsAbs(normalized) {
		return "", fmt.Errorf("invalid source archive path: %s", name)
	}
	return path.Clean(normalized), nil
}

func writeArchiveFile(root *os.Root, target string, reader io.Reader, mode os.FileMode) error {
	file, err := root.OpenFile(target, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, mode)
	if err != nil {
		return fmt.Errorf("create source archive file: %w", err)
	}
	if _, err := io.Copy(file, reader); err != nil {
		if closeErr := file.Close(); closeErr != nil {
			return oops.In("runner_agent").With("target", target).Wrapf(oops.Join(err, closeErr), "write source archive file and close target")
		}
		return fmt.Errorf("write source archive file: %w", err)
	}
	if err := file.Close(); err != nil {
		return fmt.Errorf("close source archive file: %w", err)
	}
	return nil
}

func archiveFileMode(mode os.FileMode) os.FileMode {
	if mode&0o111 != 0 {
		return 0o700
	}
	return 0o600
}
