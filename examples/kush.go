package main

import (
	"fmt"
	"os"

	"github.com/pbar1/astu/internal/pkg/k8s"
)

func main() {
	err := k8s.RunContainer("ubuntu")
	check(err)
}

func check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
