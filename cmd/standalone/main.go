// Package main contains the gity standalone command.
package main

import (
	"log"
)

func main() {
	if err := newStandaloneApp().Run(); err != nil {
		log.Fatal(err)
	}
}
