package main

import (
	"context"
	"os"
	"testing"
	"time"
)

func TestParseArgsDefault(t *testing.T) {
	args, err := parseArgs([]string{})
	if err != nil {
		t.Fatalf("parseArgs() error = %v", err)
	}
	if !args.refreshAll || len(args.projectIDs) != 0 {
		t.Fatalf("expected refreshAll=true projectIDs empty; got %#v", args)
	}
}

func TestParseArgsWithProjectID(t *testing.T) {
	args, err := parseArgs([]string{"--project-id", "789", "123", "456"})
	if err != nil {
		t.Fatalf("parseArgs() error = %v", err)
	}
	if args.refreshAll {
		t.Fatalf("expected refreshAll=false")
	}
	if len(args.projectIDs) != 3 {
		t.Fatalf("expected 3 project IDs; got %d", len(args.projectIDs))
	}
	if args.projectIDs[0] != 789 || args.projectIDs[1] != 123 || args.projectIDs[2] != 456 {
		t.Fatalf("unexpected project IDs: %#v", args.projectIDs)
	}
}

func TestParseArgsConflict(t *testing.T) {
	if _, err := parseArgs([]string{"--all", "--project-id", "10"}); err == nil {
		t.Fatalf("expected conflict error")
	}
}

func TestParseArgsInvalidID(t *testing.T) {
	if _, err := parseArgs([]string{"--project-id", "not-a-number"}); err == nil {
		t.Fatalf("expected parse error")
	}
}

func TestWithSignalContextCancel(t *testing.T) {
	ctx, cancel := withSignalContext(context.Background())
	defer cancel()

	if deadlineCtx, ok := ctx.Deadline(); ok {
		t.Fatalf("unexpected deadline: %v", deadlineCtx)
	}

	goCtx := t.Context()
	if goCtx == nil {
		t.Fatalf("testing context is nil")
	}

	// Force a manual shutdown signal through the returned cancel.
	cancel()
	select {
	case <-ctx.Done():
	case <-time.After(2 * time.Second):
		t.Fatal("context should be canceled after cancel()")
	}
}

func TestProjectIDListSet(t *testing.T) {
	var ids projectIDList
	if err := ids.Set("11"); err != nil {
		t.Fatalf("Set() error: %v", err)
	}
	if err := ids.Set("22"); err != nil {
		t.Fatalf("Set() error: %v", err)
	}
	if got := ids.String(); got != "11,22" {
		t.Fatalf("String() = %q", got)
	}

	if err := ids.Set("0"); err == nil {
		t.Fatal("expected error for non-positive id")
	}
}

func TestMainRunParseFail(t *testing.T) {
	oldArgs := os.Args
	defer func() {
		os.Args = oldArgs
	}()

	os.Args = []string{"gity-search-index", "--all", "--project-id", "1"}
	if err := run(os.Args[1:]); err == nil {
		t.Fatal("expected run() to fail for conflicting args")
	}
}
