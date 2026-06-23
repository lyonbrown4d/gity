package mergerequest_test

import (
	"reflect"
	"testing"

	mergerequest "github.com/lyonbrown4d/gity/internal/application/merge_request"
)

func TestStripCodeOwnerCommentSupportsEscapedHash(t *testing.T) {
	t.Parallel()

	line := `docs/\#notes.md @alice # trailing comment`
	got := mergerequest.StripCodeOwnerComment(line)
	if got != `docs/\#notes.md @alice ` {
		t.Fatalf("unexpected comment-stripped line: %q", got)
	}
}

func TestStripCodeOwnerCommentPreservesPathWithHashtagWithoutLeadingSpace(t *testing.T) {
	t.Parallel()

	line := `docs/#notes.md @alice`
	got := mergerequest.StripCodeOwnerComment(line)
	if got != line {
		t.Fatalf("unexpected comment-stripped line: %q", got)
	}
}

func TestSplitCodeOwnerFieldsSupportsEscapedSpace(t *testing.T) {
	t.Parallel()

	fields := mergerequest.SplitCodeOwnerFields(`src/my\ file.txt @alice @bob`)
	expected := []string{"src/my file.txt", "@alice", "@bob"}
	if !reflect.DeepEqual(fields, expected) {
		t.Fatalf("split fields mismatch: %#v", fields)
	}
}

func TestParseCodeOwnerLineWithEscapedSpaceAndComment(t *testing.T) {
	t.Parallel()

	rule, ok := mergerequest.ParseCodeOwnerLine(`src/my\ file.txt @alice # comment`)
	if !ok {
		t.Fatal("expected parse success")
	}
	if rule.Pattern != "src/my file.txt" {
		t.Fatalf("unexpected pattern: %q", rule.Pattern)
	}
	if got := rule.Usernames; !reflect.DeepEqual(got, []string{"alice"}) {
		t.Fatalf("unexpected usernames: %#v", got)
	}
}

func TestParseCodeOwnerLineSkipsSectionAndNegatedPattern(t *testing.T) {
	t.Parallel()

	for _, line := range []string{"[team]", "!"} {
		if _, ok := mergerequest.ParseCodeOwnerLine(line); ok {
			t.Fatalf("expected line %q to be skipped", line)
		}
	}
}
