package main

import (
	"fmt"
	"os"

	"github.com/ktr0731/go-fuzzyfinder"
	"github.com/pbar1/astu/internal/pkg/k8s"
)

func main() {
	pods, err := k8s.GetPods()
	check(err)

	podIdx, err := fuzzyfinder.Find(pods, func(i int) string {
		return fmt.Sprintf("%s/%s [%s]", pods[i].Namespace, pods[i].Name, pods[i].Status)
	})
	check(err)

	pod := pods[podIdx]
	ctrs := pod.Containers

	ctrIdx, err := fuzzyfinder.Find(ctrs, func(i int) string {
		return fmt.Sprintf("%s [%s]", ctrs[i].Name, ctrs[i].Image)
	})
	check(err)

	ctr := ctrs[ctrIdx]

	shells, err := k8s.GetShells(pod.Namespace, pod.Name, ctr.Name)
	check(err)

	shellIdx, err := fuzzyfinder.Find(shells, func(i int) string {
		return shells[i]
	})
	check(err)

	shell := shells[shellIdx]

	err = k8s.ExecContainer(pod.Namespace, pod.Name, ctr.Name, shell)
	check(err)
}

func check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
