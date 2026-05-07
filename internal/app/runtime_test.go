package app

import (
	"context"
	"path/filepath"
	"testing"
	"time"
)

func TestRuntimeAppsValidate(t *testing.T) {
	cases := map[string]func() interface{ Validate() error }{
		"migration":  func() interface{ Validate() error } { return NewMigrationApp() },
		"server":     func() interface{ Validate() error } { return NewServerApp() },
		"worker":     func() interface{ Validate() error } { return NewWorkerApp() },
		"standalone": func() interface{ Validate() error } { return NewStandaloneApp() },
	}
	for name, factory := range cases {
		name := name
		factory := factory
		t.Run(name, func(t *testing.T) {
			if err := factory().Validate(); err != nil {
				t.Fatalf("validate runtime app: %v", err)
			}
		})
	}
}

func TestServerRuntimeStarts(t *testing.T) {
	tempDir := t.TempDir()
	t.Setenv("GITY_HTTP_ADDRESS", "127.0.0.1:0")
	t.Setenv("GITY_DATABASE_DSN", "file:"+filepath.ToSlash(filepath.Join(tempDir, "gity.db"))+"?_pragma=foreign_keys(1)")
	t.Setenv("GITY_GIT_REPO_ROOT", filepath.ToSlash(filepath.Join(tempDir, "repos")))
	t.Setenv("GITY_STORAGE_ROOT", filepath.ToSlash(filepath.Join(tempDir, "storage")))

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	runtime, err := NewServerApp().Start(ctx)
	if err != nil {
		t.Fatalf("start server runtime: %v", err)
	}
	stopCtx, stopCancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer stopCancel()
	if err := runtime.Stop(stopCtx); err != nil {
		t.Fatalf("stop server runtime: %v", err)
	}
}
