package runneragent

import (
	"context"
	"errors"
	"os/exec"
	"runtime"
	"strings"

	"golang.org/x/sys/execabs"
)

func scriptCommand(ctx context.Context, shell, script string) *exec.Cmd {
	normalized := strings.ToLower(strings.TrimSpace(shell))
	if normalized == "" {
		return scriptCommandWithStdin(defaultScriptShellCommand(ctx), script)
	}
	switch normalized {
	case "powershell", "powershell.exe":
		return scriptCommandWithStdin(execabs.CommandContext(ctx, "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", "-"), script)
	case "pwsh", "pwsh.exe":
		return scriptCommandWithStdin(execabs.CommandContext(ctx, "pwsh", "-NoProfile", "-NonInteractive", "-Command", "-"), script)
	case "cmd", "cmd.exe":
		return scriptCommandWithStdin(execabs.CommandContext(ctx, "cmd.exe", "/Q", "/D"), script)
	case "bash":
		return scriptCommandWithStdin(execabs.CommandContext(ctx, "bash", "-s"), script)
	case "sh":
		return scriptCommandWithStdin(execabs.CommandContext(ctx, "/bin/sh", "-s"), script)
	default:
		return scriptCommandWithStdin(defaultScriptShellCommand(ctx), script)
	}
}

func scriptCommandWithStdin(command *exec.Cmd, script string) *exec.Cmd {
	command.Stdin = strings.NewReader(script)
	return command
}

func defaultScriptShellCommand(ctx context.Context) *exec.Cmd {
	if runtime.GOOS == "windows" {
		return execabs.CommandContext(ctx, "powershell.exe", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", "-")
	}
	return execabs.CommandContext(ctx, "/bin/sh", "-s")
}

func supportedScriptShell(shell string) bool {
	switch normalizeShellName(shell) {
	case "", "powershell", "pwsh", "cmd", "bash", "sh":
		return true
	default:
		return false
	}
}

func exitCodeFromError(err error) int {
	if err == nil {
		return 0
	}
	var exitError *exec.ExitError
	if errors.As(err, &exitError) {
		return exitError.ExitCode()
	}
	return 1
}
