package ci

import (
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx/querydsl"
)

func (s ProjectJobArtifactSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectJobLogSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectJobSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
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
