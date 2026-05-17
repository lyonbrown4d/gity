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

	ExecutionMode            string
	ContainerRuntime         string
	ContainerRuntimeEndpoint string
	DockerBinary             string
	DockerImage              string
	DockerNetwork            string
	DockerHostNetwork        bool
	DockerWorkDir            string
	DockerMemoryLimit        string
	DockerCPUs               string
	FirecrackerSocket        string
}

func ConfigFromEnv(args []string) (Config, error) {
	cfg := defaultConfigFromEnv()
	allowedShells := envString("GITY_RUNNER_ALLOWED_SHELLS", strings.Join(defaultAllowedShells(), ","))
	if err := parseRunnerFlags(args, &cfg, &allowedShells); err != nil {
		return Config{}, err
	}
	cfg.AllowedShells = parseAllowedShells(allowedShells)
	cfg = normalizeRunnerConfig(cfg)
	if err := validateRunnerConfig(cfg); err != nil {
		return Config{}, err
	}
	return cfg, nil
}

func defaultConfigFromEnv() Config {
	return Config{
		ServerURL:                envString("GITY_RUNNER_URL", "http://localhost:8080/v1"),
		Token:                    envString("GITY_RUNNER_TOKEN", ""),
		WorkDir:                  envString("GITY_RUNNER_WORKDIR", "./data/runner"),
		RepoRoot:                 envString("GITY_RUNNER_REPO_ROOT", envString("GITY_GIT__REPO_ROOT", envString("GITY_GIT_REPO_ROOT", ""))),
		PollInterval:             envDuration("GITY_RUNNER_POLL_INTERVAL", time.Second),
		LeaseSeconds:             envInt("GITY_RUNNER_LEASE_SECONDS", 600),
		MaxOutputBytes:           envInt("GITY_RUNNER_MAX_OUTPUT_BYTES", 65536),
		CleanWorkspace:           envBool("GITY_RUNNER_CLEAN_WORKSPACE", true),
		Once:                     envBool("GITY_RUNNER_ONCE", false),
		ExecutionMode:            envString("GITY_RUNNER_EXECUTION_MODE", ""),
		ContainerRuntime:         envString("GITY_RUNNER_CONTAINER_RUNTIME", runnerExecutionModeDocker),
		ContainerRuntimeEndpoint: envString("GITY_RUNNER_CONTAINER_RUNTIME_ENDPOINT", ""),
		DockerBinary:             envString("GITY_RUNNER_DOCKER_BINARY", dockerBinaryDefault),
		DockerImage:              envString("GITY_RUNNER_CONTAINER_IMAGE", envString("GITY_RUNNER_DOCKER_IMAGE", envString("GITY_RUNNER_DOCKER_DEFAULT_IMAGE", ""))),
		DockerNetwork:            envString("GITY_RUNNER_CONTAINER_NETWORK", envString("GITY_RUNNER_DOCKER_NETWORK", "")),
		DockerHostNetwork:        envBool("GITY_RUNNER_CONTAINER_HOST_NETWORK", envBool("GITY_RUNNER_DOCKER_HOST_NETWORK", false)),
		DockerWorkDir:            envString("GITY_RUNNER_CONTAINER_WORKDIR", envString("GITY_RUNNER_DOCKER_WORKDIR", "/workspace")),
		DockerMemoryLimit:        envString("GITY_RUNNER_CONTAINER_MEMORY", envString("GITY_RUNNER_DOCKER_MEMORY", "")),
		DockerCPUs:               envString("GITY_RUNNER_CONTAINER_CPUS", envString("GITY_RUNNER_DOCKER_CPUS", "")),
		FirecrackerSocket:        envString("GITY_RUNNER_FIRECRACKER_SOCKET", ""),
	}
}

func parseRunnerFlags(args []string, cfg *Config, allowedShells *string) error {
	flags := flag.NewFlagSet("gity-runner", flag.ContinueOnError)
	bindRunnerFlags(flags, cfg, allowedShells)
	if err := flags.Parse(args); err != nil {
		return oops.In("runner_agent").Wrapf(err, "parse runner config flags")
	}
	return nil
}

func bindRunnerFlags(flags *flag.FlagSet, cfg *Config, allowedShells *string) {
	flags.StringVar(&cfg.ServerURL, "server", cfg.ServerURL, "Gity API base URL, for example http://localhost:8080/v1")
	flags.StringVar(&cfg.Token, "token", cfg.Token, "runner registration token")
	flags.StringVar(&cfg.WorkDir, "workdir", cfg.WorkDir, "runner workspace directory")
	flags.StringVar(&cfg.RepoRoot, "repo-root", cfg.RepoRoot, "local bare repository root used for CI checkout")
	flags.DurationVar(&cfg.PollInterval, "poll-interval", cfg.PollInterval, "job polling interval")
	flags.IntVar(&cfg.LeaseSeconds, "lease-seconds", cfg.LeaseSeconds, "claim lease in seconds")
	flags.IntVar(&cfg.MaxOutputBytes, "max-output-bytes", cfg.MaxOutputBytes, "maximum captured job output bytes")
	flags.StringVar(allowedShells, "allowed-shells", *allowedShells, "comma-separated shell allowlist")
	flags.BoolVar(&cfg.CleanWorkspace, "clean-workspace", cfg.CleanWorkspace, "delete job workspace after execution")
	flags.BoolVar(&cfg.Once, "once", cfg.Once, "claim and run at most one job")
	flags.StringVar(&cfg.ExecutionMode, "execution-mode", cfg.ExecutionMode, "script execution mode: host, docker, podman, containerd, or firecracker")
	flags.StringVar(
		&cfg.ContainerRuntime,
		"container-runtime",
		cfg.ContainerRuntime,
		"default container runtime for script jobs: docker, podman, containerd, or firecracker",
	)
	flags.StringVar(&cfg.ContainerRuntimeEndpoint, "container-runtime-endpoint", cfg.ContainerRuntimeEndpoint, "container runtime API endpoint")
	flags.StringVar(&cfg.DockerBinary, "docker-binary", cfg.DockerBinary, "deprecated: runtime API mode is used by default")
	flags.StringVar(&cfg.DockerBinary, "container-runtime-binary", cfg.DockerBinary, "deprecated: runtime API mode is used by default")
	flags.StringVar(&cfg.DockerImage, "container-image", cfg.DockerImage, "default container image for script jobs")
	flags.StringVar(&cfg.DockerNetwork, "container-network", cfg.DockerNetwork, "container network")
	flags.BoolVar(&cfg.DockerHostNetwork, "container-host-network", cfg.DockerHostNetwork, "run container in host network mode")
	flags.StringVar(&cfg.DockerWorkDir, "container-workdir", cfg.DockerWorkDir, "working directory inside container")
	flags.StringVar(&cfg.DockerMemoryLimit, "container-memory", cfg.DockerMemoryLimit, "container memory limit (e.g. 512m)")
	flags.StringVar(&cfg.DockerCPUs, "container-cpus", cfg.DockerCPUs, "container cpus limit (e.g. 1.5)")
	flags.StringVar(&cfg.FirecrackerSocket, "firecracker-socket", cfg.FirecrackerSocket, "firecracker runtime socket (unix path or unix/tcp URL)")
	flags.StringVar(&cfg.DockerImage, "docker-image", cfg.DockerImage, "default docker image for script jobs")
	flags.StringVar(&cfg.DockerNetwork, "docker-network", cfg.DockerNetwork, "docker container network (deprecated)")
	flags.BoolVar(&cfg.DockerHostNetwork, "docker-host-network", cfg.DockerHostNetwork, "run docker container in host network mode (deprecated)")
	flags.StringVar(&cfg.DockerWorkDir, "docker-workdir", cfg.DockerWorkDir, "working directory inside docker container (deprecated)")
	flags.StringVar(&cfg.DockerMemoryLimit, "docker-memory", cfg.DockerMemoryLimit, "docker memory limit (e.g. 512m)")
	flags.StringVar(&cfg.DockerCPUs, "docker-cpus", cfg.DockerCPUs, "docker cpus limit (e.g. 1.5)")
}

func normalizeRunnerConfig(cfg Config) Config {
	cfg.ExecutionMode = normalizeExecutionMode(cfg.ExecutionMode)
	cfg.ContainerRuntime = normalizeExecutionMode(cfg.ContainerRuntime)
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
	if strings.TrimSpace(cfg.DockerWorkDir) == "" {
		cfg.DockerWorkDir = "/workspace"
	}
	if cfg.ContainerRuntime == "" {
		cfg.ContainerRuntime = runnerExecutionModeDocker
	}
	return cfg
}

func validateRunnerConfig(cfg Config) error {
	if !isExecutionModeSupported(cfg.ExecutionMode) {
		return oops.In("runner_agent").New("unsupported runner execution mode")
	}
	if !isContainerRuntimeSupported(cfg.ContainerRuntime) {
		return oops.In("runner_agent").New("unsupported runner container runtime")
	}
	if strings.TrimSpace(cfg.Token) == "" {
		return oops.In("runner_agent").New("runner token is required")
	}
	if strings.TrimSpace(cfg.ServerURL) == "" {
		return oops.In("runner_agent").New("runner server URL is required")
	}
	return nil
}

func normalizeExecutionMode(mode string) string {
	return strings.ToLower(strings.TrimSpace(mode))
}

func isExecutionModeSupported(mode string) bool {
	return mode == "" || mode == runnerExecutionModeHost || mode == runnerExecutionModeDocker || mode == runnerExecutionModePodman || mode == runnerExecutionModeContainerd || mode == runnerExecutionModeFirecracker
}

func isContainerRuntimeSupported(runtime string) bool {
	return runtime == "" || runtime == runnerExecutionModeDocker || runtime == runnerExecutionModePodman || runtime == runnerExecutionModeContainerd || runtime == runnerExecutionModeFirecracker
}

const (
	runnerExecutionModeHost        = "host"
	runnerExecutionModeDocker      = "docker"
	runnerExecutionModePodman      = "podman"
	runnerExecutionModeContainerd  = "containerd"
	runnerExecutionModeFirecracker = "firecracker"
	dockerBinaryDefault            = "docker"
)

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
