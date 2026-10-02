package main

import (
	"bytes"
	"context"
	"runtime"
	"runtime/pprof"
	"strings"
	"testing"
)

const goroutineLifecycleLabel = "gity.goroutine-lifecycle"

func withLabeledSignalContext(parent context.Context, label string) (context.Context, context.CancelFunc) {
	labeledParent := pprof.WithLabels(parent, pprof.Labels(goroutineLifecycleLabel, label))
	pprof.SetGoroutineLabels(labeledParent)
	defer pprof.SetGoroutineLabels(parent)
	return withSignalContext(labeledParent)
}

func waitForGoroutineLabel(t *testing.T, profileName, label string) {
	t.Helper()

	for range 1000 {
		if strings.Contains(writeRuntimeProfile(t, profileName), label) {
			return
		}
		runtime.Gosched()
	}
	t.Fatalf("%s profile never contained goroutine label %q", profileName, label)
}

func writeRuntimeProfile(t *testing.T, name string) string {
	t.Helper()

	profile := pprof.Lookup(name)
	if profile == nil {
		t.Fatalf("runtime profile %q is unavailable", name)
	}

	var output bytes.Buffer
	if err := profile.WriteTo(&output, 1); err != nil {
		t.Fatalf("write runtime profile %q: %v", name, err)
	}
	return output.String()
}
