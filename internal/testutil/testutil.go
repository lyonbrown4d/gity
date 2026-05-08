// Package testutil contains shared test helpers.
package testutil

import "testing"

type closeFunc interface {
	Close() error
}

func Must[T any](value T, err error) T {
	if err != nil {
		panic(err)
	}
	return value
}

func CleanupClose(tb testing.TB, name string, closer closeFunc) {
	tb.Helper()
	tb.Cleanup(func() {
		tb.Helper()
		if err := closer.Close(); err != nil {
			tb.Errorf("close %s: %v", name, err)
		}
	})
}
