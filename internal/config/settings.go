package config

import "github.com/DaiYuANg/arcgo/configx"

type Settings struct {
	App      AppSettings      `mapstructure:"app"`
	HTTP     HTTPSettings     `mapstructure:"http"`
	Database DatabaseSettings `mapstructure:"database"`
	Git      GitSettings      `mapstructure:"git"`
	Storage  StorageSettings  `mapstructure:"storage"`
}

type AppSettings struct {
	Name        string `mapstructure:"name"`
	Environment string `mapstructure:"environment"`
}

type HTTPSettings struct {
	Address string `mapstructure:"address"`
	BaseURL string `mapstructure:"base_url"`
}

type DatabaseSettings struct {
	Driver string `mapstructure:"driver"`
	DSN    string `mapstructure:"dsn"`
	NodeID uint16 `mapstructure:"node_id"`
}

type GitSettings struct {
	RepoRoot string `mapstructure:"repo_root"`
	Bin      string `mapstructure:"bin"`
}

type StorageSettings struct {
	Driver             string `mapstructure:"driver"`
	Root               string `mapstructure:"root"`
	S3Bucket           string `mapstructure:"s3_bucket"`
	S3Region           string `mapstructure:"s3_region"`
	S3Endpoint         string `mapstructure:"s3_endpoint"`
	S3AccessKey        string `mapstructure:"s3_access_key"`
	S3SecretKey        string `mapstructure:"s3_secret_key"`
	S3UsePathStyle     bool   `mapstructure:"s3_use_path_style"`
	S3AutoCreateBucket bool   `mapstructure:"s3_auto_create_bucket"`
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
	}
}

func NewConfig() (*configx.Config, error) {
	return configx.NewConfig(
		configx.WithDotenv(".env"),
		configx.WithEnvPrefix("GITY"),
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
