package gitsearch_test

import (
	"testing"

	collectionlist "github.com/arcgolabs/collectionx/list"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	gitsearch "github.com/lyonbrown4d/gity/internal/infrastructure/git_search"
)

func TestAppendMatchesPreservesLineNumbersAndLimit(t *testing.T) {
	t.Parallel()

	plan, err := gitsearch.NewPlan(gitports.SearchParams{Query: "needle", Limit: 2, MatchCase: true})
	if err != nil {
		t.Fatalf("NewPlan() error = %v", err)
	}
	results := collectionlist.NewList[gitports.SearchResult]()
	gitsearch.AppendMatches("README.md", []byte("skip\nneedle first\nskip\nneedle second\nneedle third"), plan, results)

	got := results.Values()
	if len(got) != 2 {
		t.Fatalf("result count = %d, want 2", len(got))
	}
	if got[0].LineNumber != 2 || got[0].LineContent != "needle first" {
		t.Fatalf("first result = %+v, want line 2", got[0])
	}
	if got[1].LineNumber != 4 || got[1].LineContent != "needle second" {
		t.Fatalf("second result = %+v, want line 4", got[1])
	}
}

func TestAppendMatchesIncludesTrailingEmptyLine(t *testing.T) {
	t.Parallel()

	plan, err := gitsearch.NewPlan(gitports.SearchParams{Query: "^$", Limit: 1, MatchCase: true, UseRegex: true})
	if err != nil {
		t.Fatalf("NewPlan() error = %v", err)
	}
	results := collectionlist.NewList[gitports.SearchResult]()
	gitsearch.AppendMatches("empty.txt", []byte("first\nsecond\n"), plan, results)

	got := results.Values()
	if len(got) != 1 {
		t.Fatalf("result count = %d, want 1", len(got))
	}
	if got[0].LineNumber != 3 || got[0].LineContent != "" {
		t.Fatalf("result = %+v, want trailing empty line 3", got[0])
	}
}
