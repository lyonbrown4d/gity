package gittransport

import (
	"fmt"
	"strings"
	"testing"
)

func TestParseReceivePackBranchUpdates(t *testing.T) {
	body := []byte(
		pkt("0000000000000000000000000000000000000000 1111111111111111111111111111111111111111 refs/heads/main\x00report-status\n") +
			pkt("1111111111111111111111111111111111111111 2222222222222222222222222222222222222222 refs/heads/feature/test\n") +
			pkt("1111111111111111111111111111111111111111 2222222222222222222222222222222222222222 refs/tags/v1\n") +
			"0000",
	)
	branches := parseReceivePackBranchUpdates(body)
	if strings.Join(branches, ",") != "main,feature/test" {
		t.Fatalf("unexpected branches: %+v", branches)
	}
}

func pkt(payload string) string {
	return fmt.Sprintf("%04x%s", len(payload)+4, payload)
}
