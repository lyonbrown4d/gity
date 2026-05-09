package project

import (
	"regexp"
	"strings"
	"time"
)

const (
	ProjectBranchProtectionRuleExact   = "exact"
	ProjectBranchProtectionRulePattern = "pattern"

	ProjectBranchProtectionAccessNoOne      = "no_one"
	ProjectBranchProtectionAccessDeveloper  = "developer"
	ProjectBranchProtectionAccessMaintainer = "maintainer"
	ProjectBranchProtectionAccessOwner      = "owner"
)

type ProjectBranchProtection struct {
	ID                     int64     `dbx:"id"`
	ProjectID              int64     `dbx:"project_id"`
	BranchName             string    `dbx:"branch_name"`
	RuleType               string    `dbx:"rule_type"`
	PushAccessLevel        string    `dbx:"push_access_level"`
	MergeAccessLevel       string    `dbx:"merge_access_level"`
	RequireMergeRequest    int       `dbx:"require_merge_request"`
	RequirePipelineSuccess int       `dbx:"require_pipeline_success"`
	AllowForcePush         int       `dbx:"allow_force_push"`
	AllowDelete            int       `dbx:"allow_delete"`
	CreatedAt              time.Time `dbx:"created_at"`
	UpdatedAt              time.Time `dbx:"updated_at"`
}

func NewProjectBranchProtection(projectID int64, branchName string, now time.Time) ProjectBranchProtection {
	branchName = strings.TrimSpace(branchName)
	return ProjectBranchProtection{
		ProjectID:              projectID,
		BranchName:             branchName,
		RuleType:               DefaultProjectBranchProtectionRuleType(branchName),
		PushAccessLevel:        ProjectBranchProtectionAccessNoOne,
		MergeAccessLevel:       ProjectBranchProtectionAccessMaintainer,
		RequireMergeRequest:    1,
		RequirePipelineSuccess: 0,
		AllowForcePush:         0,
		AllowDelete:            0,
		CreatedAt:              now,
		UpdatedAt:              now,
	}
}

func DefaultProjectBranchProtectionRuleType(branchName string) string {
	if strings.ContainsAny(branchName, "*?") {
		return ProjectBranchProtectionRulePattern
	}
	return ProjectBranchProtectionRuleExact
}

func NormalizeProjectBranchProtectionRuleType(value, branchName string) string {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case ProjectBranchProtectionRuleExact:
		return ProjectBranchProtectionRuleExact
	case ProjectBranchProtectionRulePattern:
		return ProjectBranchProtectionRulePattern
	default:
		return DefaultProjectBranchProtectionRuleType(branchName)
	}
}

func NormalizeProjectBranchProtectionAccessLevel(value, fallback string) string {
	switch strings.ToLower(strings.TrimSpace(value)) {
	case ProjectBranchProtectionAccessNoOne:
		return ProjectBranchProtectionAccessNoOne
	case ProjectBranchProtectionAccessDeveloper:
		return ProjectBranchProtectionAccessDeveloper
	case ProjectBranchProtectionAccessMaintainer:
		return ProjectBranchProtectionAccessMaintainer
	case ProjectBranchProtectionAccessOwner:
		return ProjectBranchProtectionAccessOwner
	default:
		return fallback
	}
}

func (p ProjectBranchProtection) MatchesBranch(branchName string) bool {
	branchName = strings.TrimSpace(branchName)
	ruleBranch := strings.TrimSpace(p.BranchName)
	switch NormalizeProjectBranchProtectionRuleType(p.RuleType, ruleBranch) {
	case ProjectBranchProtectionRulePattern:
		return wildcardMatch(ruleBranch, branchName)
	default:
		return ruleBranch == branchName
	}
}

func (p ProjectBranchProtection) BlocksDirectPush() bool {
	if p.RequiresMergeRequest() {
		return true
	}
	accessLevel := NormalizeProjectBranchProtectionAccessLevel(p.PushAccessLevel, ProjectBranchProtectionAccessNoOne)
	return accessLevel == ProjectBranchProtectionAccessNoOne
}

func (p ProjectBranchProtection) BlocksDelete() bool {
	return p.AllowDelete == 0
}

func (p ProjectBranchProtection) BlocksForcePush() bool {
	return p.AllowForcePush == 0
}

func (p ProjectBranchProtection) RequiresMergeRequest() bool {
	return p.RequireMergeRequest != 0
}

func (p ProjectBranchProtection) RequiresPipelineSuccess() bool {
	return p.RequirePipelineSuccess != 0
}

func wildcardMatch(pattern, value string) bool {
	var builder strings.Builder
	writePatternPart(&builder, "^")
	for _, char := range pattern {
		switch char {
		case '*':
			writePatternPart(&builder, ".*")
		case '?':
			writePatternPart(&builder, ".")
		default:
			writePatternPart(&builder, regexp.QuoteMeta(string(char)))
		}
	}
	writePatternPart(&builder, "$")
	matched, err := regexp.MatchString(builder.String(), value)
	return err == nil && matched
}

func writePatternPart(builder *strings.Builder, value string) {
	if _, err := builder.WriteString(value); err != nil {
		panic(err)
	}
}
