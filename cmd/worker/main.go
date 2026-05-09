// Package main contains the gity worker command.
package main

import (
	"log"

	workerapp "github.com/DaiYuANg/gity/cmd/workerapp"
)

func main() {
	if err := workerapp.NewApp().Run(); err != nil {
		log.Fatal(err)
	}
}
