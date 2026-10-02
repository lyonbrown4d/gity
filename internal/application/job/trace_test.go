package job_test

import (
	"testing"

	"github.com/lyonbrown4d/gity/internal/testutil"
)

func TestDecodeScriptResultUsesLastJSONTrailer(t *testing.T) {
	t.Parallel()

	fixture := newJobFixture(t, true)
	jobID := assertClaimScriptJob(t, fixture)
	result := "building\n{not json}\n{\"exit_code\":2,\"output_truncated\":true,\"duration_millis\":17}"
	testutil.Must(fixture.service.CompleteProjectJob(fixture.ctx, fixture.projectID, jobID, result))

	got := testutil.Must(fixture.service.GetProjectJobTrace(fixture.ctx, fixture.projectID, jobID))
	if got.Trace != "building\n{not json}" {
		t.Fatalf("trace = %q, want prefix before final trailer", got.Trace)
	}
	if got.ExitCode != 2 || !got.OutputTruncated || got.DurationMillis != 17 {
		t.Fatalf("decoded trailer = %+v", got)
	}
}

func TestParseScriptResultPreservesAmbiguousJSONAsOutput(t *testing.T) {
	t.Parallel()

	fixture := newJobFixture(t, true)
	jobID := assertClaimScriptJob(t, fixture)
	result := `{"output":"first","output":"second"}`
	testutil.Must(fixture.service.CompleteProjectJob(fixture.ctx, fixture.projectID, jobID, result))

	got := testutil.Must(fixture.service.GetProjectJobTrace(fixture.ctx, fixture.projectID, jobID))
	if got.Trace != result {
		t.Fatalf("trace = %q, want ambiguous JSON preserved as %q", got.Trace, result)
	}
}
