package dbschema

import (
	"time"

	"github.com/arcgolabs/dbx/column"
	"github.com/arcgolabs/dbx/schema"
)

type AuditRevision struct {
	ID        int64     `dbx:"id"`
	CreatedAt time.Time `dbx:"created_at"`
	Actor     string    `dbx:"actor"`
	Tenant    string    `dbx:"tenant"`
	Reason    string    `dbx:"reason"`
	Metadata  string    `dbx:"metadata"`
}

type AuditRevisionSchemaDef struct {
	schema.Schema[AuditRevision]
	ID        column.Column[AuditRevision, int64]     `dbx:"id,pk"`
	CreatedAt column.Column[AuditRevision, time.Time] `dbx:"created_at,type=TIMESTAMP"`
	Actor     column.Column[AuditRevision, string]    `dbx:"actor,null"`
	Tenant    column.Column[AuditRevision, string]    `dbx:"tenant,null"`
	Reason    column.Column[AuditRevision, string]    `dbx:"reason,null"`
	Metadata  column.Column[AuditRevision, string]    `dbx:"metadata,type=TEXT,null"`
}

type ProjectBranchProtectionAudit struct {
	RevisionID                int64     `dbx:"revision_id"`
	Operation                 string    `dbx:"operation"`
	ProjectBranchProtectionID int64     `dbx:"project_branch_protection_id"`
	ProjectID                 int64     `dbx:"project_id"`
	BranchName                string    `dbx:"branch_name"`
	RuleType                  string    `dbx:"rule_type"`
	PushAccessLevel           string    `dbx:"push_access_level"`
	MergeAccessLevel          string    `dbx:"merge_access_level"`
	RequireMergeRequest       int       `dbx:"require_merge_request"`
	RequirePipelineSuccess    int       `dbx:"require_pipeline_success"`
	AllowForcePush            int       `dbx:"allow_force_push"`
	AllowDelete               int       `dbx:"allow_delete"`
	CreatedAt                 time.Time `dbx:"created_at"`
	UpdatedAt                 time.Time `dbx:"updated_at"`
}

type ProjectBranchProtectionAuditSchemaDef struct {
	schema.Schema[ProjectBranchProtectionAudit]
	RevisionID                column.Column[ProjectBranchProtectionAudit, int64]     `dbx:"revision_id,index,ref=audit_revisions.id"`
	Operation                 column.Column[ProjectBranchProtectionAudit, string]    `dbx:"operation,index"`
	ProjectBranchProtectionID column.Column[ProjectBranchProtectionAudit, int64]     `dbx:"project_branch_protection_id,index"`
	ProjectID                 column.Column[ProjectBranchProtectionAudit, int64]     `dbx:"project_id,index"`
	BranchName                column.Column[ProjectBranchProtectionAudit, string]    `dbx:"branch_name,index"`
	RuleType                  column.Column[ProjectBranchProtectionAudit, string]    `dbx:"rule_type"`
	PushAccessLevel           column.Column[ProjectBranchProtectionAudit, string]    `dbx:"push_access_level"`
	MergeAccessLevel          column.Column[ProjectBranchProtectionAudit, string]    `dbx:"merge_access_level"`
	RequireMergeRequest       column.Column[ProjectBranchProtectionAudit, int]       `dbx:"require_merge_request"`
	RequirePipelineSuccess    column.Column[ProjectBranchProtectionAudit, int]       `dbx:"require_pipeline_success"`
	AllowForcePush            column.Column[ProjectBranchProtectionAudit, int]       `dbx:"allow_force_push"`
	AllowDelete               column.Column[ProjectBranchProtectionAudit, int]       `dbx:"allow_delete"`
	CreatedAt                 column.Column[ProjectBranchProtectionAudit, time.Time] `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt                 column.Column[ProjectBranchProtectionAudit, time.Time] `dbx:"updated_at,type=TIMESTAMP"`
}

type ProjectMemberAudit struct {
	RevisionID      int64     `dbx:"revision_id"`
	Operation       string    `dbx:"operation"`
	ProjectMemberID int64     `dbx:"project_member_id"`
	ProjectID       int64     `dbx:"project_id"`
	UserID          int64     `dbx:"user_id"`
	Role            string    `dbx:"role"`
	CreatedAt       time.Time `dbx:"created_at"`
	UpdatedAt       time.Time `dbx:"updated_at"`
}

type ProjectMemberAuditSchemaDef struct {
	schema.Schema[ProjectMemberAudit]
	RevisionID      column.Column[ProjectMemberAudit, int64]     `dbx:"revision_id,index,ref=audit_revisions.id"`
	Operation       column.Column[ProjectMemberAudit, string]    `dbx:"operation,index"`
	ProjectMemberID column.Column[ProjectMemberAudit, int64]     `dbx:"project_member_id,index"`
	ProjectID       column.Column[ProjectMemberAudit, int64]     `dbx:"project_id,index"`
	UserID          column.Column[ProjectMemberAudit, int64]     `dbx:"user_id,index"`
	Role            column.Column[ProjectMemberAudit, string]    `dbx:"role"`
	CreatedAt       column.Column[ProjectMemberAudit, time.Time] `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt       column.Column[ProjectMemberAudit, time.Time] `dbx:"updated_at,type=TIMESTAMP"`
}

type ProjectMergeRequestApprovalRuleAudit struct {
	RevisionID                        int64     `dbx:"revision_id"`
	Operation                         string    `dbx:"operation"`
	ProjectMergeRequestApprovalRuleID int64     `dbx:"project_merge_request_approval_rule_id"`
	ProjectID                         int64     `dbx:"project_id"`
	Name                              string    `dbx:"name"`
	TargetBranch                      string    `dbx:"target_branch"`
	ApprovalsRequired                 int       `dbx:"approvals_required"`
	EligibleUserIDs                   string    `dbx:"eligible_user_ids"`
	CodeOwner                         int       `dbx:"code_owner"`
	CreatedAt                         time.Time `dbx:"created_at"`
	UpdatedAt                         time.Time `dbx:"updated_at"`
}

type ProjectMergeRequestApprovalRuleAuditSchemaDef struct {
	schema.Schema[ProjectMergeRequestApprovalRuleAudit]
	RevisionID                        column.Column[ProjectMergeRequestApprovalRuleAudit, int64]     `dbx:"revision_id,index,ref=audit_revisions.id"`
	Operation                         column.Column[ProjectMergeRequestApprovalRuleAudit, string]    `dbx:"operation,index"`
	ProjectMergeRequestApprovalRuleID column.Column[ProjectMergeRequestApprovalRuleAudit, int64]     `dbx:"project_merge_request_approval_rule_id,index"`
	ProjectID                         column.Column[ProjectMergeRequestApprovalRuleAudit, int64]     `dbx:"project_id,index"`
	Name                              column.Column[ProjectMergeRequestApprovalRuleAudit, string]    `dbx:"name"`
	TargetBranch                      column.Column[ProjectMergeRequestApprovalRuleAudit, string]    `dbx:"target_branch,index"`
	ApprovalsRequired                 column.Column[ProjectMergeRequestApprovalRuleAudit, int]       `dbx:"approvals_required"`
	EligibleUserIDs                   column.Column[ProjectMergeRequestApprovalRuleAudit, string]    `dbx:"eligible_user_ids"`
	CodeOwner                         column.Column[ProjectMergeRequestApprovalRuleAudit, int]       `dbx:"code_owner,index"`
	CreatedAt                         column.Column[ProjectMergeRequestApprovalRuleAudit, time.Time] `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt                         column.Column[ProjectMergeRequestApprovalRuleAudit, time.Time] `dbx:"updated_at,type=TIMESTAMP"`
}

type ProjectCIVariableAudit struct {
	RevisionID          int64     `dbx:"revision_id"`
	Operation           string    `dbx:"operation"`
	ProjectCIVariableID int64     `dbx:"project_ci_variable_id"`
	ProjectID           int64     `dbx:"project_id"`
	Key                 string    `dbx:"key"`
	Masked              int       `dbx:"masked"`
	Protected           int       `dbx:"protected"`
	CreatedAt           time.Time `dbx:"created_at"`
	UpdatedAt           time.Time `dbx:"updated_at"`
}

type ProjectCIVariableAuditSchemaDef struct {
	schema.Schema[ProjectCIVariableAudit]
	RevisionID          column.Column[ProjectCIVariableAudit, int64]     `dbx:"revision_id,index,ref=audit_revisions.id"`
	Operation           column.Column[ProjectCIVariableAudit, string]    `dbx:"operation,index"`
	ProjectCIVariableID column.Column[ProjectCIVariableAudit, int64]     `dbx:"project_ci_variable_id,index"`
	ProjectID           column.Column[ProjectCIVariableAudit, int64]     `dbx:"project_id,index"`
	Key                 column.Column[ProjectCIVariableAudit, string]    `dbx:"key,index"`
	Masked              column.Column[ProjectCIVariableAudit, int]       `dbx:"masked"`
	Protected           column.Column[ProjectCIVariableAudit, int]       `dbx:"protected"`
	CreatedAt           column.Column[ProjectCIVariableAudit, time.Time] `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt           column.Column[ProjectCIVariableAudit, time.Time] `dbx:"updated_at,type=TIMESTAMP"`
}

type ProjectDeployKeyAudit struct {
	RevisionID         int64     `dbx:"revision_id"`
	Operation          string    `dbx:"operation"`
	ProjectDeployKeyID int64     `dbx:"project_deploy_key_id"`
	ProjectID          int64     `dbx:"project_id"`
	Title              string    `dbx:"title"`
	Fingerprint        string    `dbx:"fingerprint"`
	CanPush            int       `dbx:"can_push"`
	CreatedByUserID    int64     `dbx:"created_by_user_id"`
	LastUsedAt         time.Time `dbx:"last_used_at"`
	CreatedAt          time.Time `dbx:"created_at"`
	UpdatedAt          time.Time `dbx:"updated_at"`
}

type ProjectDeployKeyAuditSchemaDef struct {
	schema.Schema[ProjectDeployKeyAudit]
	RevisionID         column.Column[ProjectDeployKeyAudit, int64]     `dbx:"revision_id,index,ref=audit_revisions.id"`
	Operation          column.Column[ProjectDeployKeyAudit, string]    `dbx:"operation,index"`
	ProjectDeployKeyID column.Column[ProjectDeployKeyAudit, int64]     `dbx:"project_deploy_key_id,index"`
	ProjectID          column.Column[ProjectDeployKeyAudit, int64]     `dbx:"project_id,index"`
	Title              column.Column[ProjectDeployKeyAudit, string]    `dbx:"title"`
	Fingerprint        column.Column[ProjectDeployKeyAudit, string]    `dbx:"fingerprint,index"`
	CanPush            column.Column[ProjectDeployKeyAudit, int]       `dbx:"can_push"`
	CreatedByUserID    column.Column[ProjectDeployKeyAudit, int64]     `dbx:"created_by_user_id,index"`
	LastUsedAt         column.Column[ProjectDeployKeyAudit, time.Time] `dbx:"last_used_at,type=TIMESTAMP,null"`
	CreatedAt          column.Column[ProjectDeployKeyAudit, time.Time] `dbx:"created_at,type=TIMESTAMP"`
	UpdatedAt          column.Column[ProjectDeployKeyAudit, time.Time] `dbx:"updated_at,type=TIMESTAMP"`
}

var (
	AuditRevisionSchema                        = schema.MustSchema("audit_revisions", AuditRevisionSchemaDef{})
	ProjectBranchProtectionAuditSchema         = schema.MustSchema("project_branch_protection_audits", ProjectBranchProtectionAuditSchemaDef{})
	ProjectMemberAuditSchema                   = schema.MustSchema("project_member_audits", ProjectMemberAuditSchemaDef{})
	ProjectMergeRequestApprovalRuleAuditSchema = schema.MustSchema("project_merge_request_approval_rule_audits", ProjectMergeRequestApprovalRuleAuditSchemaDef{})
	ProjectCIVariableAuditSchema               = schema.MustSchema("project_ci_variable_audits", ProjectCIVariableAuditSchemaDef{})
	ProjectDeployKeyAuditSchema                = schema.MustSchema("project_deploy_key_audits", ProjectDeployKeyAuditSchemaDef{})
)
