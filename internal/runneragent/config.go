package runneragent

import (
	"flag"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

type Config struct {
	ServerURL      string
	Token          string
	WorkDir        string
	RepoRoot       string
	PollInterval   time.Duration
	LeaseSeconds   int
	MaxOutputBytes int
	Once           bool
}

func ConfigFromEnv(args []string) (Config, error) {
	cfg := Config{
		ServerURL:      envString("GITY_RUNNER_URL", "http://localhost:8080/v1"),
		Token:          envString("GITY_RUNNER_TOKEN", ""),
		WorkDir:        envString("GITY_RUNNER_WORKDIR", "./data/runner"),
		RepoRoot:       envString("GITY_RUNNER_REPO_ROOT", envString("GITY_GIT__REPO_ROOT", envString("GITY_GIT_REPO_ROOT", ""))),
		PollInterval:   envDuration("GITY_RUNNER_POLL_INTERVAL", time.Second),
		LeaseSeconds:   envInt("GITY_RUNNER_LEASE_SECONDS", 600),
		MaxOutputBytes: envInt("GITY_RUNNER_MAX_OUTPUT_BYTES", 65536),
		Once:           envBool("GITY_RUNNER_ONCE", false),
	}

	flags := flag.NewFlagSet("gity-runner", flag.ContinueOnError)
	flags.StringVar(&cfg.ServerURL, "server", cfg.ServerURL, "Gity API base URL, for example http://localhost:8080/v1")
	flags.StringVar(&cfg.Token, "token", cfg.Token, "runner registration token")
	flags.StringVar(&cfg.WorkDir, "workdir", cfg.WorkDir, "runner workspace directory")
	flags.StringVar(&cfg.RepoRoot, "repo-root", cfg.RepoRoot, "local bare repository root used for CI checkout")
	flags.DurationVar(&cfg.PollInterval, "poll-interval", cfg.PollInterval, "job polling interval")
	flags.IntVar(&cfg.LeaseSeconds, "lease-seconds", cfg.LeaseSeconds, "claim lease in seconds")
	flags.IntVar(&cfg.MaxOutputBytes, "max-output-bytes", cfg.MaxOutputBytes, "maximum captured job output bytes")
	flags.BoolVar(&cfg.Once, "once", cfg.Once, "claim and run at most one job")
	if err := flags.Parse(args); err != nil {
		return Config{}, err
	}
	if strings.TrimSpace(cfg.Token) == "" {
		return Config{}, fmt.Errorf("runner token is required")
	}
	if strings.TrimSpace(cfg.ServerURL) == "" {
		return Config{}, fmt.Errorf("runner server URL is required")
	}
	if cfg.PollInterval <= 0 {
		cfg.PollInterval = time.Second
	}
	if cfg.LeaseSeconds <= 0 {
		cfg.LeaseSeconds = 600
	}
	if cfg.MaxOutputBytes <= 0 {
		cfg.MaxOutputBytes = 65536
	}
	return cfg, nil
}

func envString(key string, fallback string) string {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	return value
}

func envInt(key string, fallback int) int {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	parsed, err := strconv.Atoi(value)
	if err != nil {
		return fallback
	}
	return parsed
}

func envBool(key string, fallback bool) bool {
	value := strings.TrimSpace(strings.ToLower(os.Getenv(key)))
	if value == "" {
		return fallback
	}
	return value == "1" || value == "true" || value == "yes" || value == "on"
}

func envDuration(key string, fallback time.Duration) time.Duration {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	parsed, err := time.ParseDuration(value)
	if err != nil {
		return fallback
	}
	return parsed
}
