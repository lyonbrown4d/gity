// Package main implements the search index maintenance command.
package main

import (
	"context"
	"flag"
	"log"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"syscall"

	searchindexapp "github.com/lyonbrown4d/gity/internal/layout/searchindex"
	"github.com/samber/oops"
)

func main() {
	if err := run(os.Args[1:]); err != nil {
		log.Fatal(err)
	}
}

type projectIDList []int64

func (ids *projectIDList) Set(raw string) error {
	value, err := strconv.ParseInt(strings.TrimSpace(raw), 10, 64)
	if err != nil {
		return oops.In("cmd.search_index").Wrapf(err, "parse project id")
	}
	if value <= 0 {
		return oops.In("cmd.search_index").New("project id must be greater than zero")
	}
	*ids = append(*ids, value)
	return nil
}

func (ids *projectIDList) String() string {
	if ids == nil || len(*ids) == 0 {
		return ""
	}
	parts := make([]string, len(*ids))
	for i, id := range *ids {
		parts[i] = strconv.FormatInt(id, 10)
	}
	return strings.Join(parts, ",")
}

func run(argv []string) error {
	parsed, err := parseArgs(argv)
	if err != nil {
		return oops.In("cmd.search_index").Wrapf(err, "parse command arguments")
	}

	ctx, cancel := withSignalContext(context.Background())
	defer cancel()

	if parsed.refreshAll {
		return oops.In("cmd.search_index").Wrapf(searchindexapp.Run(ctx), "run search index rebuild for all projects")
	}

	return oops.In("cmd.search_index").Wrapf(searchindexapp.Run(ctx, parsed.projectIDs...), "run search index rebuild for selected projects")
}

type searchIndexArgs struct {
	refreshAll bool
	projectIDs []int64
}

func parseArgs(argv []string) (*searchIndexArgs, error) {
	var projectIDs projectIDList
	flags := flag.NewFlagSet("gity-search-index", flag.ContinueOnError)
	refreshAll := flags.Bool("all", false, "refresh all project search indexes")
	flags.Var(&projectIDs, "project-id", "refresh one project by id; can be repeated")
	if err := flags.Parse(argv); err != nil {
		return nil, oops.In("cmd.search_index").Wrapf(err, "parse command line")
	}

	parsedIDs, err := parseProjectIDs(flags.Args())
	if err != nil {
		return nil, oops.In("cmd.search_index").Wrapf(err, "parse positional project ids")
	}
	if len(parsedIDs) > 0 || len(projectIDs) > 0 {
		combined := make(projectIDList, 0, len(projectIDs)+len(parsedIDs))
		combined = append(combined, projectIDs...)
		combined = append(combined, parsedIDs...)
		projectIDs = combined
	}

	if *refreshAll && len(projectIDs) > 0 {
		return nil, oops.In("cmd.search_index").New("cannot combine --all with project ids")
	}

	if *refreshAll || len(projectIDs) == 0 {
		return &searchIndexArgs{refreshAll: true}, nil
	}

	return &searchIndexArgs{projectIDs: projectIDs}, nil
}

func parseProjectIDs(rawIDs []string) ([]int64, error) {
	if len(rawIDs) == 0 {
		return nil, nil
	}
	projectIDs := make([]int64, 0, len(rawIDs))
	for _, rawProjectID := range rawIDs {
		parsed, err := strconv.ParseInt(strings.TrimSpace(rawProjectID), 10, 64)
		if err != nil {
			return nil, oops.In("cmd.search_index").With("project_id", rawProjectID).Wrapf(err, "parse positional project id")
		}
		if parsed <= 0 {
			return nil, oops.In("cmd.search_index").With("project_id", rawProjectID).New("project id must be greater than zero")
		}
		projectIDs = append(projectIDs, parsed)
	}
	return projectIDs, nil
}

func withSignalContext(ctx context.Context) (context.Context, context.CancelFunc) {
	if ctx == nil {
		ctx = context.Background()
	}

	ctx, cancel := context.WithCancel(ctx)
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, os.Interrupt, syscall.SIGTERM)
	go func() {
		<-sigCh
		cancel()
	}()
	return ctx, func() {
		signal.Stop(sigCh)
		cancel()
	}
}
