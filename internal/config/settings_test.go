package config_test

import (
	"testing"

	"github.com/lyonbrown4d/gity/internal/config"
)

func TestSettingsEnvironmentOverridesTypedDefaults(t *testing.T) {
	t.Setenv("GITY_HTTP__ADDRESS", "127.0.0.1:0")
	t.Setenv("GITY_DATABASE__DSN", "file:test-override.db?_pragma=foreign_keys(1)")
	t.Setenv("GITY_DATABASE__NODE_ID", "7")
	t.Setenv("GITY_GIT__REPO_ROOT", "./tmp/repos")
	t.Setenv("GITY_STORAGE__ROOT", "./tmp/storage")
	t.Setenv("GITY_WORKER__POLL_INTERVAL_MILLIS", "42")

	cfg, err := config.NewConfig()
	if err != nil {
		t.Fatalf("new config: %v", err)
	}
	settings, err := config.NewSettings(cfg)
	if err != nil {
		t.Fatalf("new settings: %v", err)
	}
	if settings.HTTP.Address != "127.0.0.1:0" {
		t.Fatalf("http address = %q", settings.HTTP.Address)
	}
	if settings.Database.DSN != "file:test-override.db?_pragma=foreign_keys(1)" {
		t.Fatalf("database dsn = %q", settings.Database.DSN)
	}
	if settings.Database.NodeID != 7 {
		t.Fatalf("database node id = %d", settings.Database.NodeID)
	}
	if settings.Git.RepoRoot != "./tmp/repos" {
		t.Fatalf("git repo root = %q", settings.Git.RepoRoot)
	}
	if settings.Storage.Root != "./tmp/storage" {
		t.Fatalf("storage root = %q", settings.Storage.Root)
	}
	if settings.Worker.PollIntervalMillis != 42 {
		t.Fatalf("worker poll interval = %d", settings.Worker.PollIntervalMillis)
	}
}
