// Package main contains the gity migration command.
package main

import (
	"context"
	"log"

	"github.com/DaiYuANg/gity/internal/bootstrap"
)

func main() {
	if err := bootstrap.RunMigration(context.Background()); err != nil {
		log.Fatal(err)
	}
}
