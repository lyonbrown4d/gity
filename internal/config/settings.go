package config

import "github.com/arcgolabs/configx"

type Settings struct {
	App      AppSettings      `json:"app"      koanf:"app"      mapstructure:"app"`
	HTTP     HTTPSettings     `json:"http"     koanf:"http"     mapstructure:"http"`
	Database DatabaseSettings `json:"database" koanf:"database" mapstructure:"database"`
	Git      GitSettings      `json:"git"      koanf:"git"      mapstructure:"git"`
	Storage  StorageSettings  `json:"storage"  koanf:"storage"  mapstructure:"storage"`
	Worker   WorkerSettings   `json:"worker"   koanf:"worker"   mapstructure:"worker"`
}

type AppSettings struct {
	Name        string `json:"name"        koanf:"name"        mapstructure:"name"`
	Environment string `json:"environment" koanf:"environment" mapstructure:"environment"`
}

type HTTPSettings struct {
	Address string `json:"address"  koanf:"address"  mapstructure:"address"`
	BaseURL string `json:"base_url" koanf:"base_url" mapstructure:"base_url"`
}

type DatabaseSettings struct {
	Driver string `json:"driver"  koanf:"driver"  mapstructure:"driver"`
	DSN    string `json:"dsn"     koanf:"dsn"     mapstructure:"dsn"`
	NodeID int    `json:"node_id" koanf:"node_id" mapstructure:"node_id"`
}

type GitSettings struct {
	RepoRoot string `json:"repo_root" koanf:"repo_root" mapstructure:"repo_root"`
	Bin      string `json:"bin"       koanf:"bin"       mapstructure:"bin"`
}

type StorageSettings struct {
	Driver             string `json:"driver"                koanf:"driver"                mapstructure:"driver"`
	Root               string `json:"root"                  koanf:"root"                  mapstructure:"root"`
	S3Bucket           string `json:"s3_bucket"             koanf:"s3_bucket"             mapstructure:"s3_bucket"`
	S3Region           string `json:"s3_region"             koanf:"s3_region"             mapstructure:"s3_region"`
	S3Endpoint         string `json:"s3_endpoint"           koanf:"s3_endpoint"           mapstructure:"s3_endpoint"`
	S3AccessKey        string `json:"s3_access_key"         koanf:"s3_access_key"         mapstructure:"s3_access_key"`
	S3SecretKey        string `json:"s3_secret_key"         koanf:"s3_secret_key"         mapstructure:"s3_secret_key"`
	S3UsePathStyle     bool   `json:"s3_use_path_style"     koanf:"s3_use_path_style"     mapstructure:"s3_use_path_style"`
	S3AutoCreateBucket bool   `json:"s3_auto_create_bucket" koanf:"s3_auto_create_bucket" mapstructure:"s3_auto_create_bucket"`
}

type WorkerSettings struct {
	Enabled            bool   `json:"enabled"              koanf:"enabled"              mapstructure:"enabled"`
	ID                 string `json:"id"                   koanf:"id"                   mapstructure:"id"`
	PollIntervalMillis int    `json:"poll_interval_millis" koanf:"poll_interval_millis" mapstructure:"poll_interval_millis"`
	LeaseSeconds       int    `json:"lease_seconds"        koanf:"lease_seconds"        mapstructure:"lease_seconds"`
	MaxJobsPerTick     int    `json:"max_jobs_per_tick"    koanf:"max_jobs_per_tick"    mapstructure:"max_jobs_per_tick"`
}

func DefaultSettings() Settings {
	return Settings{
		App: AppSettings{
			Name:        "gity",
			Environment: "development",
		},
		HTTP: HTTPSettings{
			Address: ":8080",
			BaseURL: "http://localhost:8080",
		},
		Database: DatabaseSettings{
			Driver: "sqlite",
			DSN:    "file:gity.db?_pragma=foreign_keys(1)",
			NodeID: 1,
		},
		Git: GitSettings{
			RepoRoot: "./data/repos",
			Bin:      "git",
		},
		Storage: StorageSettings{
			Driver:             "local",
			Root:               "./data/storage",
			S3Region:           "us-east-1",
			S3UsePathStyle:     true,
			S3AutoCreateBucket: false,
		},
		Worker: WorkerSettings{
			Enabled:            true,
			PollIntervalMillis: 1000,
			LeaseSeconds:       60,
			MaxJobsPerTick:     1,
		},
	}
}

func NewConfig() (*configx.Config, error) {
	return configx.NewConfig(
		configx.WithDotenv(".env"),
		configx.WithEnvPrefix("GITY"),
		configx.WithEnvSeparator("__"),
		configx.WithTypedDefaults(DefaultSettings()),
	)
}

func NewSettings(cfg *configx.Config) (Settings, error) {
	settings := DefaultSettings()
	if cfg == nil {
		return settings, nil
	}
	loaded, err := configx.GetAs[Settings](cfg, "")
	if err != nil {
		return settings, err
	}
	return loaded, nil
}
