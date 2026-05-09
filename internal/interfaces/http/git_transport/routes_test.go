package gittransport_test

import (
	"fmt"
	"strings"
	"testing"

	gittransport "github.com/DaiYuANg/gity/internal/interfaces/http/git_transport"
)

func TestParseReceivePackBranchUpdates(t *testing.T) {
	body := []byte(
		pkt("0000000000000000000000000000000000000000 1111111111111111111111111111111111111111 refs/heads/main\x00report-status\n") +
			pkt("1111111111111111111111111111111111111111 2222222222222222222222222222222222222222 refs/heads/feature/test\n") +
			pkt("1111111111111111111111111111111111111111 2222222222222222222222222222222222222222 refs/tags/v1\n") +
			"0000",
	)
	branches := receivePackBranchNames(body)
	if strings.Join(branches, ",") != "main,feature/test" {
		t.Fatalf("unexpected branches: %+v", branches)
	}
}

func TestParseReceivePackUpdatesMarksBranchDeletes(t *testing.T) {
	body := []byte(
		pkt("1111111111111111111111111111111111111111 0000000000000000000000000000000000000000 refs/heads/obsolete\x00report-status\n") +
			pkt("1111111111111111111111111111111111111111 2222222222222222222222222222222222222222 refs/heads/main\n") +
			"0000",
	)
	updates := gittransport.ParseReceivePackUpdates(body)
	if len(updates) != 2 {
		t.Fatalf("updates = %d: %+v", len(updates), updates)
	}
	if updates[0].BranchName != "obsolete" || !updates[0].Delete {
		t.Fatalf("expected obsolete delete update: %+v", updates[0])
	}
	if updates[1].BranchName != "main" || updates[1].Delete {
		t.Fatalf("expected main update: %+v", updates[1])
	}
}

func pkt(payload string) string {
	return fmt.Sprintf("%04x%s", len(payload)+4, payload)
}

func receivePackBranchNames(body []byte) []string {
	updates := gittransport.ParseReceivePackUpdates(body)
	branches := make([]string, 0, len(updates))
	for _, update := range updates {
		if update.BranchName != "" {
			branches = append(branches, update.BranchName)
		}
	}
	return branches
}
