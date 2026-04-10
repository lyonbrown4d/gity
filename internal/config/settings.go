package config

import "github.com/DaiYuANg/arcgo/configx"

type Settings struct {
	App      AppSettings      `mapstructure:"app"`
	HTTP     HTTPSettings     `mapstructure:"http"`
	Database DatabaseSettings `mapstructure:"database"`
	Git      GitSettings      `mapstructure:"git"`
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
}

type GitSettings struct {
	RepoRoot string `mapstructure:"repo_root"`
	Bin      string `mapstructure:"bin"`
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
		},
		Git: GitSettings{
			RepoRoot: "./data/repos",
			Bin:      "git",
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
