package main

import (
	"context"
	"log"

	"github.com/DaiYuANg/gity/internal/app"
)

func main() {
	if err := app.RunMigration(context.Background()); err != nil {
		log.Fatal(err)
	}
}
