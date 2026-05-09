// Package debug exposes build metadata used by application composition.
package debug

import (
	runtimedebug "runtime/debug"
	"strings"

	"github.com/arcgolabs/dix"
)

const developmentVersion = "dev"

// VersionOverride can be set with -ldflags for release builds.
var VersionOverride string

// Version returns the current build version.
func Version() string {
	if version := normalizeVersion(VersionOverride); version != "" {
		return version
	}
	info, ok := runtimedebug.ReadBuildInfo()
	if !ok {
		return developmentVersion
	}
	if version := normalizeVersion(info.Main.Version); version != "" {
		return version
	}
	if revision := vcsRevision(info.Settings); revision != "" {
		return developmentVersion + "-" + revision
	}
	return developmentVersion
}

// AppMeta returns dix application metadata using build version information.
func AppMeta(name, description string) dix.AppMeta {
	return dix.AppMeta{
		Name:        name,
		Version:     Version(),
		Description: description,
	}
}

// Module provides application metadata through dix dependency injection.
func Module(moduleName, appName, description string) dix.Module {
	return dix.NewModule(
		moduleName,
		dix.Description("Build metadata"),
		dix.Providers(
			dix.Provider0(func() dix.AppMeta {
				return AppMeta(appName, description)
			}, dix.Eager()),
		),
	)
}

func vcsRevision(settings []runtimedebug.BuildSetting) string {
	for _, setting := range settings {
		if setting.Key != "vcs.revision" {
			continue
		}
		return shortRevision(setting.Value)
	}
	return ""
}

func normalizeVersion(version string) string {
	version = strings.TrimSpace(version)
	if version == "" || version == "(devel)" {
		return ""
	}
	return version
}

func shortRevision(revision string) string {
	revision = strings.TrimSpace(revision)
	if len(revision) <= 12 {
		return revision
	}
	return revision[:12]
}
