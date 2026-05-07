package main

import (
	"log"

	"github.com/DaiYuANg/gity/internal/bootstrap"
)

func main() {
	if err := bootstrap.NewWorkerApp().Run(); err != nil {
		log.Fatal(err)
	}
}
