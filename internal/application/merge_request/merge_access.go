package mergerequest

import (
	"context"
	"strings"

	apperror "github.com/lyonbrown4d/gity/internal/application/app_error"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	"github.com/samber/oops"
)

func (s *Service) ensureTargetBranchMergeAccess(ctx context.Context, projectID, mergeIID int64, project projectdomain.Project, mr mergedomain.ProjectMergeRequest, actorUserID int64) error {
	protection, protected, err := s.targetBranchProtection(ctx, project.ID, mr.TargetBranch)
	if err != nil || !protected {
		return err
	}
	accessLevel := projectdomain.NormalizeProjectBranchProtectionAccessLevel(protection.MergeAccessLevel, projectdomain.ProjectBranchProtectionAccessMaintainer)
	if accessLevel == projectdomain.ProjectBranchProtectionAccessNoOne {
		return apperror.Forbidden("protected branch cannot be merged", oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "target_branch", mr.TargetBranch).New("protected branch merge access is no_one"))
	}
	return s.ensureActorMatchesMergeAccess(ctx, projectID, mergeIID, project, accessLevel, actorUserID)
}

func (s *Service) ensureActorMatchesMergeAccess(ctx context.Context, projectID, mergeIID int64, project projectdomain.Project, accessLevel string, actorUserID int64) error {
	if actorUserID <= 0 {
		return apperror.Forbidden("merge actor is required", oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID).New("merge actor is required for protected branch"))
	}
	user, err := s.userRepo.GetByID(ctx, actorUserID)
	if err != nil {
		return apperror.NotFound("merge actor not found", err)
	}
	if user.IsSuperAdmin != 0 {
		return nil
	}
	if s.projectMemberRepo != nil {
		member, memberErr := s.projectMemberRepo.FindByProjectAndUser(ctx, project.ID, actorUserID)
		if memberErr == nil && branchProtectionAccessAllowsRole(accessLevel, member.Role) {
			return nil
		}
	}
	if s.memberRepo == nil {
		return apperror.Forbidden("merge access cannot be verified", oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID).New("organization member repository is not configured"))
	}
	member, err := s.memberRepo.FindByOrganizationAndUser(ctx, project.OrganizationID, actorUserID)
	if err != nil {
		return apperror.Forbidden("protected branch merge access denied", oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "actor_user_id", actorUserID, "organization_id", project.OrganizationID).Wrapf(err, "load merge actor membership"))
	}
	if !branchProtectionAccessAllowsRole(accessLevel, member.Role) {
		return apperror.Forbidden("protected branch merge access denied", oops.In("merge_request").With("project_id", projectID, "merge_iid", mergeIID, "actor_user_id", actorUserID, "role", member.Role, "access_level", accessLevel).New("protected branch merge access denied"))
	}
	return nil
}

func branchProtectionAccessAllowsRole(accessLevel, role string) bool {
	role = strings.ToLower(strings.TrimSpace(role))
	switch projectdomain.NormalizeProjectBranchProtectionAccessLevel(accessLevel, projectdomain.ProjectBranchProtectionAccessNoOne) {
	case projectdomain.ProjectBranchProtectionAccessDeveloper:
		return projectBranchProtectionDeveloperRoles.Contains(role)
	case projectdomain.ProjectBranchProtectionAccessMaintainer:
		return projectBranchProtectionMaintainerRoles.Contains(role)
	case projectdomain.ProjectBranchProtectionAccessOwner:
		return role == projectdomain.ProjectBranchProtectionAccessOwner
	default:
		return false
	}
}
