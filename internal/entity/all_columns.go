package entity

import (
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx/querydsl"
)

func (s NamespaceSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s NamespaceMemberSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectBranchProtectionSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectIssueSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectIssueAttachmentSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectIssueCommentSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectJobArtifactSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectJobLogSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectJobSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectLFSLockSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectLFSObjectSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectMergeRequestSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectPackageSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectPackageFileSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectPackageVersionSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectPipelineSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectPipelineJobSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectRunnerSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectWikiPageSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s UserSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s UserAccessTokenSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}
