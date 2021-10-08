package cmd

import (
	"fmt"
	"os"

	"github.com/pbar1/astu/internal/pkg/k8s"
	"github.com/spf13/cobra"
)

// execContainerCmd represents the execContainer command
var execContainerCmd = &cobra.Command{
	Use:     "exec-container",
	Aliases: []string{"exec-ctr", "enter-container", "enter-ctr", "xx"},
	Short:   "Gets an interactive shell within a Kubernetes container",
	Long: `Gets an interactive shell within a Kubernetes container.

Flags/args after an initial "--" are passed through to kubectl.`,
	Run: func(cmd *cobra.Command, args []string) {
		err := k8s.FullExecContainer(args...)
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
	},
}

func init() {
	k8sCmd.AddCommand(execContainerCmd)
}
