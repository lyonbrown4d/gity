package runneragent_test

import (
	json "encoding/json/v2"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	runneragent "github.com/lyonbrown4d/gity/internal/infrastructure/runner_agent"
)

func TestDecodeScriptPayloadRejectsAmbiguousJSON(t *testing.T) {
	t.Parallel()

	tests := []struct {
		name    string
		payload []byte
	}{
		{
			name:    "duplicate member",
			payload: []byte(`{"script":["echo trusted"],"script":["echo untrusted"]}`),
		},
		{
			name:    "invalid UTF-8",
			payload: append([]byte(`{"script":["echo `), append([]byte{0xff}, []byte(`"]}`)...)...),
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			t.Parallel()

			job := cidomain.ProjectJob{Payload: string(test.payload)}
			if _, err := runneragent.ExecuteScriptJob(t.Context(), runneragent.Config{
				WorkDir:        t.TempDir(),
				LeaseSeconds:   30,
				MaxOutputBytes: 1024,
			}, job); err == nil {
				t.Fatalf("ExecuteScriptJob(%q) succeeded, want error", test.payload)
			}
		})
	}
}

func TestClientAllowsUnknownResponseMembers(t *testing.T) {
	t.Parallel()

	server := httptest.NewServer(http.HandlerFunc(func(response http.ResponseWriter, request *http.Request) {
		if request.URL.Path != "/runners/jobs/claim" {
			t.Errorf("path = %q, want /runners/jobs/claim", request.URL.Path)
		}
		response.Header().Set("Content-Type", "application/json")
		if _, err := response.Write([]byte(`{
			"body":{"claimed":false,"job":{},"added_later":true},
			"metadata":{"request_id":"future"}
		}`)); err != nil {
			t.Errorf("write response: %v", err)
		}
	}))
	t.Cleanup(server.Close)

	response, err := runneragent.NewClient(server.URL, "token").ClaimJob(t.Context(), 30)
	if err != nil {
		t.Fatalf("claim job with future response members: %v", err)
	}
	if response.Claimed {
		t.Fatal("claimed = true, want false")
	}
}

func TestLegacyScriptJobPayloadRoundTrip(t *testing.T) {
	t.Parallel()

	payload := `{
		"project_full_path":"core/gity",
		"ref_name":"main",
		"script":["echo compatibility"],
		"env":null,
		"artifacts":null,
		"tags":null,
		"masked_values":null,
		"timeout_seconds":5,
		"added_later":{"enabled":true}
	}`
	resultJSON, err := runneragent.ExecuteScriptJob(t.Context(), runneragent.Config{
		WorkDir:        t.TempDir(),
		LeaseSeconds:   30,
		MaxOutputBytes: 1024,
	}, cidomain.ProjectJob{
		ID:        501,
		ProjectID: 7,
		Kind:      "script",
		Payload:   payload,
		Attempts:  1,
	})
	if err != nil {
		t.Fatalf("execute legacy script job payload: %v", err)
	}

	var result runneragent.ScriptResult
	if err := json.Unmarshal([]byte(resultJSON), &result, json.RejectUnknownMembers(false)); err != nil {
		t.Fatalf("decode script result: %v", err)
	}
	if result.ExitCode != 0 || !strings.Contains(result.Output, "compatibility") {
		t.Fatalf("unexpected script result: %+v", result)
	}
}
