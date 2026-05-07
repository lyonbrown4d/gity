package identity

import (
	collectionx "github.com/arcgolabs/collectionx/list"
	"github.com/arcgolabs/dbx/querydsl"
)

func (s UserSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}

func (s UserAccessTokenSchemaDef) AllColumns() *collectionx.List[querydsl.SelectItem] {
	return querydsl.AllColumns(s)
}
