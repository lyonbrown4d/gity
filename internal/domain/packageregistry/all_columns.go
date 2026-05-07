package packageregistry

import (
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx/querydsl"
)

func (s ProjectPackageSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectPackageFileSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s ProjectPackageVersionSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}
