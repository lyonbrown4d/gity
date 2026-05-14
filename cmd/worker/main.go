// Package main contains the gity worker command.
package main

import (
	"log"

	"github.com/lyonbrown4d/gity/internal/layout/worker"
)

func main() {
	if err := worker.NewApp().Run(); err != nil {
		log.Fatal(err)
	}
}
