package mergerequest

import (
	"context"
	"encoding/base64"
	"errors"
	"fmt"
	"path"
	"regexp"
	"strings"
	"unicode"

	setx "github.com/arcgolabs/collectionx/set"
	"github.com/bmatcuk/doublestar/v4"
	gitports "github.com/lyonbrown4d/gity/internal/application/ports"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

type codeOwnerIDCache struct {
	loaded bool
	ids    []int64
}

type codeOwnerRule struct {
	Pattern   string
	Usernames []string
}

var codeOwnerUsernamePattern = regexp.MustCompile(`@([A-Za-z0-9][A-Za-z0-9_.-]*)`)

func (s *Service) approvalRuleCheck(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, rule mergedomain.ProjectMergeRequestApprovalRule, approvers *setx.Set[int64], cache *codeOwnerIDCache) (ApprovalRuleCheck, bool, error) {
	eligibleUserIDs, err := s.approvalRuleEligibleUserIDs(ctx, project, mr, rule, cache)
	if err != nil {
		return ApprovalRuleCheck{}, false, err
	}
	if rule.RequiresCodeOwner() && len(eligibleUserIDs) == 0 {
		return ApprovalRuleCheck{}, false, nil
	}
	check := ApprovalRuleCheck{
		RuleID:            rule.ID,
		Name:              rule.Name,
		TargetBranch:      rule.TargetBranch,
		ApprovalsRequired: normalizedRequiredApprovals(rule.ApprovalsRequired),
		EligibleUserIDs:   eligibleUserIDs,
		CodeOwner:         rule.RequiresCodeOwner(),
	}
	return evaluateApprovalRuleCheck(check, approvers), true, nil
}

func (s *Service) approvalRuleEligibleUserIDs(ctx context.Context, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, rule mergedomain.ProjectMergeRequestApprovalRule, cache *codeOwnerIDCache) ([]int64, error) {
	eligibleUserIDs, err := decodeApprovalRuleUserIDs(rule.EligibleUserIDs)
	if err != nil {
		return nil, oops.In("merge_request").With("project_id", project.ID, "rule_id", rule.ID).Wrapf(err, "decode approval rule eligible users")
	}
	if !rule.RequiresCodeOwner() {
		return eligibleUserIDs, nil
	}
	if cache == nil {
		cache = &codeOwnerIDCache{}
	}
	if !cache.loaded {
		cache.loaded = true
		cache.ids, err = s.resolveCodeOwnerIDs(ctx, project, mr)
		if err != nil {
			return nil, err
		}
	}
	return cache.ids, nil
}

func (s *Service) loadCodeOwners(ctx context.Context, project projectdomain.Project, targetBranch string) ([]codeOwnerRule, error) {
	for _, codeOwnersPath := range []string{"CODEOWNERS", ".gitlab/CODEOWNERS", "docs/CODEOWNERS"} {
		blob, err := s.gitRepo.GetBlob(ctx, project.FullPath+".git", targetBranch, project.DefaultBranch, codeOwnersPath)
		if err == nil {
			content, contentErr := codeOwnersBlobContent(blob)
			if contentErr != nil {
				return nil, oops.In("merge_request").With("project_id", project.ID, "path", codeOwnersPath).Wrapf(contentErr, "read CODEOWNERS")
			}
			return parseCodeOwners(content), nil
		}
		if !errors.Is(err, gitports.ErrPathNotFound) && !errors.Is(err, gitports.ErrReferenceNotFound) && !errors.Is(err, gitports.ErrEmptyRepository) {
			return nil, oops.In("merge_request").With("project_id", project.ID, "path", codeOwnersPath).Wrapf(err, "load CODEOWNERS")
		}
	}
	return nil, nil
}

func codeOwnersBlobContent(blob gitports.Blob) (string, error) {
	if blob.Encoding == "" || blob.Encoding == "utf-8" {
		return blob.Content, nil
	}
	if blob.Encoding == "base64" {
		data, err := base64.StdEncoding.DecodeString(blob.Content)
		if err != nil {
			return "", oops.In("merge_request").Wrapf(err, "decode CODEOWNERS blob")
		}
		return string(data), nil
	}
	return "", fmt.Errorf("unsupported CODEOWNERS encoding %s", blob.Encoding)
}

func parseCodeOwners(content string) []codeOwnerRule {
	rules := make([]codeOwnerRule, 0)
	for line := range strings.SplitSeq(content, "\n") {
		rule, ok := parseCodeOwnerLine(line)
		if ok {
			rules = append(rules, rule)
		}
	}
	return rules
}

func parseCodeOwnerLine(line string) (codeOwnerRule, bool) {
	line = strings.TrimSpace(stripCodeOwnerComment(line))
	if line == "" || strings.HasPrefix(line, "[") || strings.HasPrefix(line, "!") {
		return codeOwnerRule{}, false
	}
	fields := splitCodeOwnerFields(line)
	if len(fields) < 2 {
		return codeOwnerRule{}, false
	}
	usernames := codeOwnerUsernames(fields[1:])
	if len(usernames) == 0 {
		return codeOwnerRule{}, false
	}
	return codeOwnerRule{Pattern: fields[0], Usernames: usernames}, true
}

func codeOwnerUsernames(owners []string) []string {
	usernames := make([]string, 0, len(owners))
	for _, owner := range owners {
		matches := codeOwnerUsernamePattern.FindStringSubmatch(owner)
		if len(matches) == 2 {
			usernames = append(usernames, strings.ToLower(matches[1]))
		}
	}
	return usernames
}

func stripCodeOwnerComment(line string) string {
	line = strings.TrimSpace(line)
	if line == "" {
		return line
	}
	var output strings.Builder
	escaped := false
	prevWasSpaceOrStart := true

	for _, r := range line {
		if escaped {
			output.WriteRune('\\')
			output.WriteRune(r)
			prevWasSpaceOrStart = false
			escaped = false
			continue
		}

		if r == '\\' {
			escaped = true
			continue
		}

		if r == '#' && prevWasSpaceOrStart {
			return output.String()
		}

		output.WriteRune(r)
		prevWasSpaceOrStart = unicode.IsSpace(r)
	}

	if escaped {
		output.WriteRune('\\')
	}

	return output.String()
}

func splitCodeOwnerFields(line string) []string {
	line = strings.TrimSpace(line)
	if line == "" {
		return nil
	}

	fields := make([]string, 0)
	var field strings.Builder
	escaped := false

	flush := func() {
		if field.Len() > 0 {
			fields = append(fields, field.String())
			field.Reset()
		}
	}

	for _, r := range line {
		if escaped {
			field.WriteRune(r)
			escaped = false
			continue
		}

		if r == '\\' {
			escaped = true
			continue
		}

		if unicode.IsSpace(r) {
			flush()
			continue
		}

		field.WriteRune(r)
	}

	if escaped {
		field.WriteRune('\\')
	}

	flush()
	return fields
}

func matchedCodeOwnerUsernames(rules []codeOwnerRule, files []string) []string {
	usernames := setx.NewSet[string]()
	for _, file := range files {
		matched := lastMatchingCodeOwnerRule(rules, file)
		if matched == nil {
			continue
		}
		for _, username := range matched.Usernames {
			usernames.Add(username)
		}
	}
	return usernames.Values()
}

func lastMatchingCodeOwnerRule(rules []codeOwnerRule, file string) *codeOwnerRule {
	var matched *codeOwnerRule
	for index := range rules {
		rule := rules[index]
		if codeOwnerPatternMatches(rule.Pattern, file) {
			current := rule
			matched = &current
		}
	}
	return matched
}

func codeOwnerPatternMatches(pattern, filePath string) bool {
	pattern = strings.TrimSpace(strings.TrimPrefix(pattern, "/"))
	filePath = strings.TrimSpace(strings.ReplaceAll(filePath, "\\", "/"))
	if pattern == "" {
		return false
	}
	if pattern == "*" {
		return true
	}
	for _, candidate := range codeOwnerPatternCandidates(pattern) {
		matched, err := doublestar.PathMatch(candidate, filePath)
		if err == nil && matched {
			return true
		}
	}
	return pattern == filePath
}

func codeOwnerPatternCandidates(pattern string) []string {
	candidates := []string{pattern}
	if !strings.Contains(pattern, "/") {
		candidates = append(candidates, path.Join("**", pattern), path.Join("**", pattern+"*"))
	}
	if strings.HasSuffix(pattern, "/") {
		candidates = append(candidates, path.Join(pattern, "**"))
	}
	return candidates
}
