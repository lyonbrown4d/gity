package project_test

import "testing"

import projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"

func TestWildcardMatchKeepsBranchNameSemantics(t *testing.T) {
	tests := []struct {
		name    string
		pattern string
		value   string
		want    bool
	}{
		{
			name:    "wildcard matches nested branch path",
			pattern: "*",
			value:   "feature/project-access",
			want:    true,
		},
		{
			name:    "prefix wildcard matches nested branch path",
			pattern: "release/*",
			value:   "release/2026/05",
			want:    true,
		},
		{
			name:    "double wildcard keeps legacy wildcard semantics",
			pattern: "feature/**",
			value:   "feature/auth/login",
			want:    true,
		},
		{
			name:    "literal branch prefix still matters",
			pattern: "release/*",
			value:   "feature/release/2026",
			want:    false,
		},
		{
			name:    "question mark matches one branch character",
			pattern: "v1.?",
			value:   "v1.x",
			want:    true,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			protection := projectdomain.ProjectBranchProtection{
				BranchName: test.pattern,
				RuleType:   projectdomain.ProjectBranchProtectionRulePattern,
			}
			if got := protection.MatchesBranch(test.value); got != test.want {
				t.Fatalf("MatchesBranch(%q) with pattern %q = %v, want %v", test.value, test.pattern, got, test.want)
			}
		})
	}
}
