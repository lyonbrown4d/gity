package runneragent

// ResolveExecutionMode returns the effective script execution mode.
func ResolveExecutionMode(cfg Config, payload ScriptPayload) string {
	return resolveExecutionMode(cfg, payload)
}

// ResolveContainerRuntimeEndpoint returns the API endpoint for a container runtime.
func ResolveContainerRuntimeEndpoint(cfg Config, runtimeName string) string {
	return resolveContainerRuntimeEndpoint(cfg, runtimeName)
}

// ResolveContainerImage returns the payload image or configured default image.
func ResolveContainerImage(cfg Config, payload ScriptPayload) string {
	return resolveContainerImage(cfg, payload)
}

// ResolveScriptRunnerKind returns the concrete runner mode chosen for a script job.
func ResolveScriptRunnerKind(cfg Config, payload ScriptPayload) (string, error) {
	runner, err := resolveScriptRunner(cfg, payload)
	if err != nil {
		return "", err
	}
	switch runner.(type) {
	case hostScriptRunner:
		return runnerExecutionModeHost, nil
	case dockerScriptRunner:
		return runnerExecutionModeDocker, nil
	case podmanScriptRunner:
		return runnerExecutionModePodman, nil
	case containerdScriptRunner:
		return runnerExecutionModeContainerd, nil
	case firecrackerScriptRunner:
		return runnerExecutionModeFirecracker, nil
	default:
		return "", nil
	}
}

// ParseContainerByteSize parses a Docker-style byte size string.
func ParseContainerByteSize(value string) (int64, error) {
	return parseByteSize(value)
}

// ParseContainerCPUs parses a Docker-style CPU quota value.
func ParseContainerCPUs(value string) (int64, error) {
	return parseCPUs(value)
}

// IsFirecrackerCompatibleWithContainerRuntime reports whether a Firecracker endpoint can use the container compatibility path.
func IsFirecrackerCompatibleWithContainerRuntime(socket string) bool {
	return isFirecrackerCompatibleWithContainerRuntime(socket)
}
