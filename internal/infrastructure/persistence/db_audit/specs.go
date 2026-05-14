// Package dbaudit declares dbx/audit bindings for persistence repositories.
package dbaudit

import (
	auditx "github.com/arcgolabs/dbx/audit"
	cidomain "github.com/lyonbrown4d/gity/internal/domain/ci"
	identitydomain "github.com/lyonbrown4d/gity/internal/domain/identity"
	mergedomain "github.com/lyonbrown4d/gity/internal/domain/merge"
	projectdomain "github.com/lyonbrown4d/gity/internal/domain/project"
	dbschema "github.com/lyonbrown4d/gity/internal/infrastructure/persistence/db_schema"
)

func revisionSpec() auditx.RevisionSpec {
	revisions := dbschema.AuditRevisionSchema
	return auditx.RevisionTable(
		revisions,
		revisions.ID,
		revisions.CreatedAt,
		auditx.RevisionActor(revisions.Actor),
		auditx.RevisionTenant(revisions.Tenant),
		auditx.RevisionReason(revisions.Reason),
		auditx.RevisionMetadata(revisions.Metadata),
	)
}

func ProjectBranchProtectionAudit() auditx.EntitySpec[projectdomain.ProjectBranchProtection] {
	source := dbschema.ProjectBranchProtectionSchema
	audits := dbschema.ProjectBranchProtectionAuditSchema
	return auditx.MustEntity[projectdomain.ProjectBranchProtection](
		revisionSpec(),
		source,
		audits,
		auditx.AuditRevisionID(audits.RevisionID),
		auditx.OperationColumn(audits.Operation),
		auditx.Key(source.ID, audits.ProjectBranchProtectionID),
		auditx.Copy(source.ProjectID, audits.ProjectID),
		auditx.Copy(source.BranchName, audits.BranchName),
		auditx.Copy(source.RuleType, audits.RuleType),
		auditx.Copy(source.PushAccessLevel, audits.PushAccessLevel),
		auditx.Copy(source.MergeAccessLevel, audits.MergeAccessLevel),
		auditx.Copy(source.RequireMergeRequest, audits.RequireMergeRequest),
		auditx.Copy(source.RequirePipelineSuccess, audits.RequirePipelineSuccess),
		auditx.Copy(source.AllowForcePush, audits.AllowForcePush),
		auditx.Copy(source.AllowDelete, audits.AllowDelete),
		auditx.Copy(source.CreatedAt, audits.CreatedAt),
		auditx.Copy(source.UpdatedAt, audits.UpdatedAt),
	)
}

func ProjectMemberAudit() auditx.EntitySpec[projectdomain.ProjectMember] {
	source := dbschema.ProjectMemberSchema
	audits := dbschema.ProjectMemberAuditSchema
	return auditx.MustEntity[projectdomain.ProjectMember](
		revisionSpec(),
		source,
		audits,
		auditx.AuditRevisionID(audits.RevisionID),
		auditx.OperationColumn(audits.Operation),
		auditx.Key(source.ID, audits.ProjectMemberID),
		auditx.Copy(source.ProjectID, audits.ProjectID),
		auditx.Copy(source.UserID, audits.UserID),
		auditx.Copy(source.Role, audits.Role),
		auditx.Copy(source.CreatedAt, audits.CreatedAt),
		auditx.Copy(source.UpdatedAt, audits.UpdatedAt),
	)
}

func ProjectMergeRequestApprovalRuleAudit() auditx.EntitySpec[mergedomain.ProjectMergeRequestApprovalRule] {
	source := dbschema.ProjectMergeRequestApprovalRuleSchema
	audits := dbschema.ProjectMergeRequestApprovalRuleAuditSchema
	return auditx.MustEntity[mergedomain.ProjectMergeRequestApprovalRule](
		revisionSpec(),
		source,
		audits,
		auditx.AuditRevisionID(audits.RevisionID),
		auditx.OperationColumn(audits.Operation),
		auditx.Key(source.ID, audits.ProjectMergeRequestApprovalRuleID),
		auditx.Copy(source.ProjectID, audits.ProjectID),
		auditx.Copy(source.Name, audits.Name),
		auditx.Copy(source.TargetBranch, audits.TargetBranch),
		auditx.Copy(source.ApprovalsRequired, audits.ApprovalsRequired),
		auditx.Copy(source.EligibleUserIDs, audits.EligibleUserIDs),
		auditx.Copy(source.CodeOwner, audits.CodeOwner),
		auditx.Copy(source.CreatedAt, audits.CreatedAt),
		auditx.Copy(source.UpdatedAt, audits.UpdatedAt),
	)
}

func ProjectCIVariableAudit() auditx.EntitySpec[cidomain.ProjectCIVariable] {
	source := dbschema.ProjectCIVariableSchema
	audits := dbschema.ProjectCIVariableAuditSchema
	return auditx.MustEntity[cidomain.ProjectCIVariable](
		revisionSpec(),
		source,
		audits,
		auditx.AuditRevisionID(audits.RevisionID),
		auditx.OperationColumn(audits.Operation),
		auditx.Key(source.ID, audits.ProjectCIVariableID),
		auditx.Copy(source.ProjectID, audits.ProjectID),
		auditx.Copy(source.Key, audits.Key),
		auditx.Copy(source.Masked, audits.Masked),
		auditx.Copy(source.Protected, audits.Protected),
		auditx.Copy(source.CreatedAt, audits.CreatedAt),
		auditx.Copy(source.UpdatedAt, audits.UpdatedAt),
	)
}

func ProjectDeployKeyAudit() auditx.EntitySpec[identitydomain.ProjectDeployKey] {
	source := dbschema.ProjectDeployKeySchema
	audits := dbschema.ProjectDeployKeyAuditSchema
	return auditx.MustEntity[identitydomain.ProjectDeployKey](
		revisionSpec(),
		source,
		audits,
		auditx.AuditRevisionID(audits.RevisionID),
		auditx.OperationColumn(audits.Operation),
		auditx.Key(source.ID, audits.ProjectDeployKeyID),
		auditx.Copy(source.ProjectID, audits.ProjectID),
		auditx.Copy(source.Title, audits.Title),
		auditx.Copy(source.Fingerprint, audits.Fingerprint),
		auditx.Copy(source.CanPush, audits.CanPush),
		auditx.Copy(source.CreatedByUserID, audits.CreatedByUserID),
		auditx.Copy(source.LastUsedAt, audits.LastUsedAt),
		auditx.Copy(source.CreatedAt, audits.CreatedAt),
		auditx.Copy(source.UpdatedAt, audits.UpdatedAt),
	)
}
