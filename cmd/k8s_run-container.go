package cmd

import (
	"fmt"
	"os"

	"github.com/pbar1/astu/internal/pkg/k8s"
	"github.com/spf13/cobra"
)

// runContainerCmd represents the runContainer command
var runContainerCmd = &cobra.Command{
	Use:     "run-container",
	Aliases: []string{"run-ctr"},
	Short:   "Runs a container image in Kubernetes and attaches to it",
	Long: `Runs a container image in Kubernetes and attaches to it.

Flags/args after an initial "--" are passed through to kubectl.`,
	Run: func(cmd *cobra.Command, args []string) {
		var k8sArgs []string
		if cmd.ArgsLenAtDash() < 0 {
			k8sArgs = make([]string, 0)
		} else {
			k8sArgs = args[cmd.ArgsLenAtDash():]
		}

		err := k8s.RunContainer(args[0], k8sArgs...)
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
	},
}

func init() {
	k8sCmd.AddCommand(runContainerCmd)

	// TODO flag to make attaching to the container optional
}
