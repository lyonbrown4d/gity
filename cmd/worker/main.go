package main

import (
	"log"

	"github.com/DaiYuANg/gity/internal/app"
)

func main() {
	if err := app.NewWorkerApp().Run(); err != nil {
		log.Fatal(err)
	}
}
