package project

import (
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx/querydsl"
)

func (s ProjectSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectBranchProtectionSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}
