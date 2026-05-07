package main

import (
	"log"

	"github.com/DaiYuANg/gity/internal/bootstrap"
)

func main() {
	if err := bootstrap.NewStandaloneApp().Run(); err != nil {
		log.Fatal(err)
	}
}
