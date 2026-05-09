// Package main contains the gity migration command.
package main

import (
	"context"
	"log"

	migrationapp "github.com/DaiYuANg/gity/cmd/migrationapp"
)

func main() {
	if err := migrationapp.Run(context.Background()); err != nil {
		log.Fatal(err)
	}
}
