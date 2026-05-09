package wiki

import (
	"context"

	wikiservice "github.com/DaiYuANg/gity/internal/application/wiki"
	"github.com/DaiYuANg/gity/internal/infrastructure/mapperx"
	"github.com/DaiYuANg/gity/internal/interfaces/http_api"
)

func (e *Endpoint) listPages(ctx context.Context, in *wikiPagesInput) (*wikiOutput, error) {
	items, err := e.service.ListPages(ctx, in.ProjectID)
	if err != nil {
		return nil, err
	}
	return &wikiOutput{Body: items}, nil
}

func (e *Endpoint) createPage(ctx context.Context, in *createWikiPageInput) (*wikiOutput, error) {
	input, err := mapperx.MapStrict[wikiservice.CreatePageInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	authorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.AuthorUserID)
	if err != nil {
		return nil, err
	}
	input.AuthorUserID = authorUserID
	item, err := e.service.CreatePage(ctx, in.ProjectID, input)
	if err != nil {
		return nil, err
	}
	return &wikiOutput{Body: item}, nil
}

func (e *Endpoint) getPage(ctx context.Context, in *wikiPageInput) (*wikiOutput, error) {
	item, err := e.service.GetPage(ctx, in.ProjectID, in.Slug)
	if err != nil {
		return nil, err
	}
	return &wikiOutput{Body: item}, nil
}

func (e *Endpoint) updatePage(ctx context.Context, in *updateWikiPageInput) (*wikiOutput, error) {
	input, err := mapperx.MapStrict[wikiservice.UpdatePageInput](e.mapper, in.Body)
	if err != nil {
		return nil, err
	}
	editorUserID, err := httpapi.ActorUserID(ctx, e.authRuntime, in.Authorization, input.EditorUserID)
	if err != nil {
		return nil, err
	}
	input.EditorUserID = editorUserID
	item, err := e.service.UpdatePage(ctx, in.ProjectID, in.Slug, input)
	if err != nil {
		return nil, err
	}
	return &wikiOutput{Body: item}, nil
}

func (e *Endpoint) deletePage(ctx context.Context, in *wikiPageInput) (*wikiOutput, error) {
	item, err := e.service.DeletePage(ctx, in.ProjectID, in.Slug)
	if err != nil {
		return nil, err
	}
	return &wikiOutput{Body: item}, nil
}
