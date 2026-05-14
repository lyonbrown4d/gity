package runneragent

import (
	"flag"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/samber/oops"
)

type Config struct {
	ServerURL      string
	Token          string
	WorkDir        string
	RepoRoot       string
	PollInterval   time.Duration
	LeaseSeconds   int
	MaxOutputBytes int
	AllowedShells  []string
	CleanWorkspace bool
	Once           bool
}

func ConfigFromEnv(args []string) (Config, error) {
	allowedShells := envString("GITY_RUNNER_ALLOWED_SHELLS", strings.Join(defaultAllowedShells(), ","))
	cfg := Config{
		ServerURL:      envString("GITY_RUNNER_URL", "http://localhost:8080/v1"),
		Token:          envString("GITY_RUNNER_TOKEN", ""),
		WorkDir:        envString("GITY_RUNNER_WORKDIR", "./data/runner"),
		RepoRoot:       envString("GITY_RUNNER_REPO_ROOT", envString("GITY_GIT__REPO_ROOT", envString("GITY_GIT_REPO_ROOT", ""))),
		PollInterval:   envDuration("GITY_RUNNER_POLL_INTERVAL", time.Second),
		LeaseSeconds:   envInt("GITY_RUNNER_LEASE_SECONDS", 600),
		MaxOutputBytes: envInt("GITY_RUNNER_MAX_OUTPUT_BYTES", 65536),
		CleanWorkspace: envBool("GITY_RUNNER_CLEAN_WORKSPACE", true),
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
	flags.StringVar(&allowedShells, "allowed-shells", allowedShells, "comma-separated shell allowlist")
	flags.BoolVar(&cfg.CleanWorkspace, "clean-workspace", cfg.CleanWorkspace, "delete job workspace after execution")
	flags.BoolVar(&cfg.Once, "once", cfg.Once, "claim and run at most one job")
	if err := flags.Parse(args); err != nil {
		return Config{}, oops.In("runner_agent").Wrapf(err, "parse runner config flags")
	}
	cfg.AllowedShells = parseAllowedShells(allowedShells)
	if strings.TrimSpace(cfg.Token) == "" {
		return Config{}, oops.In("runner_agent").New("runner token is required")
	}
	if strings.TrimSpace(cfg.ServerURL) == "" {
		return Config{}, oops.In("runner_agent").New("runner server URL is required")
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
	if len(cfg.AllowedShells) == 0 {
		cfg.AllowedShells = defaultAllowedShells()
	}
	return cfg, nil
}

func parseAllowedShells(value string) []string {
	parts := strings.Split(value, ",")
	shells := make([]string, 0, len(parts))
	for _, part := range parts {
		shell := normalizeShellName(part)
		if shell != "" {
			shells = append(shells, shell)
		}
	}
	return shells
}

func defaultAllowedShells() []string {
	return []string{"sh", "bash", "powershell", "pwsh", "cmd"}
}

func envString(key, fallback string) string {
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
