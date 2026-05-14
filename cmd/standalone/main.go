// Package main contains the gity standalone command.
package main

import (
	"log"

	"github.com/lyonbrown4d/gity/internal/layout/standalone"
)

func main() {
	if err := standalone.NewStandaloneApp().Run(); err != nil {
		log.Fatal(err)
	}
}
