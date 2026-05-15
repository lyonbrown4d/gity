package runneragent

import (
	"fmt"

	"github.com/samber/oops"
)

const (
	firecrackerNotImplementedMessage = "firecracker runner runtime is not implemented yet"
)

func runFirecrackerScriptJob(socket string) error {
	return oops.In("runner_agent").With("runtime", runnerExecutionModeFirecracker, "socket", socket).New(firecrackerNotImplementedMessage)
}

func validateFirecrackerSocket(socket string) error {
	if socket == "" {
		return fmt.Errorf("firecracker socket is required")
	}
	return nil
}
