package core

import (
	"context"

	"github.com/arcgolabs/dbx"
	"github.com/samber/oops"
)

const projectWikiSlugUniqueMigration = "0013_project_wiki_page_slug_unique"

func fixProjectWikiPageSlugUniqueIndex(ctx context.Context, tx *dbx.Tx) error {
	dialectName := migrationDialectName(tx)
	switch dialectName {
	case "sqlite", "postgres":
		return fixProjectWikiPageSlugUniqueIndexSQL(ctx, tx, dialectName)
	case "mysql":
		return fixProjectWikiPageSlugUniqueIndexMySQL(ctx, tx, dialectName)
	default:
		return unsupportedMigrationDialect(projectWikiSlugUniqueMigration, dialectName)
	}
}

func fixProjectWikiPageSlugUniqueIndexSQL(ctx context.Context, tx *dbx.Tx, dialectName string) error {
	if _, err := tx.ExecContext(ctx, `DROP INDEX IF EXISTS "ux_project_wiki_pages_project_slug_unique"`); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectWikiSlugUniqueMigration, "dialect", dialectName).Wrapf(err, "drop old project wiki unique index")
	}
	if _, err := tx.ExecContext(ctx, `CREATE UNIQUE INDEX IF NOT EXISTS "ux_project_wiki_pages_project_slug_unique" ON "project_wiki_pages" ("project_id", "slug")`); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectWikiSlugUniqueMigration, "dialect", dialectName).Wrapf(err, "create project wiki project slug unique index")
	}
	return nil
}

func fixProjectWikiPageSlugUniqueIndexMySQL(ctx context.Context, tx *dbx.Tx, dialectName string) error {
	exists, err := mysqlIndexExists(ctx, tx, projectWikiSlugUniqueMigration, "project_wiki_pages", "ux_project_wiki_pages_project_slug_unique")
	if err != nil {
		return err
	}
	if exists {
		if _, err := tx.ExecContext(ctx, "DROP INDEX `ux_project_wiki_pages_project_slug_unique` ON `project_wiki_pages`"); err != nil {
			return oops.In("persistence.schema").With("migration_version", projectWikiSlugUniqueMigration, "dialect", dialectName).Wrapf(err, "drop old project wiki unique index")
		}
	}
	if _, err := tx.ExecContext(ctx, "CREATE UNIQUE INDEX `ux_project_wiki_pages_project_slug_unique` ON `project_wiki_pages` (`project_id`, `slug`)"); err != nil {
		return oops.In("persistence.schema").With("migration_version", projectWikiSlugUniqueMigration, "dialect", dialectName).Wrapf(err, "create project wiki project slug unique index")
	}
	return nil
}
