// Package main contains the gity migration command.
package main

import (
	"context"
	"log"

	migrationapp "github.com/DaiYuANg/gity/internal/layout/migration"
)

func main() {
	if err := migrationapp.Run(context.Background()); err != nil {
		log.Fatal(err)
	}
}
