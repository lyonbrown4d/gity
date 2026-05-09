// Package main contains the gity server command.
package main

import (
	"log"

	serverapp "github.com/DaiYuANg/gity/cmd/serverapp"
)

func main() {
	if err := serverapp.NewApp().Run(); err != nil {
		log.Fatal(err)
	}
}
