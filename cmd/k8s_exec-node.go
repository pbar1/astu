package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// execNodeCmd represents the execNode command
var execNodeCmd = &cobra.Command{
	Use:     "exec-node",
	Aliases: []string{"enter-node"},
	Short:   "(WIP) Gets a host-level shell on a Kubernetes node",
	Long: `(WIP) Gets a host-level shell on a Kubernetes node.

Flags/args after an initial "--" are passed through to kubectl.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("execNode called")
	},
}

func init() {
	k8sCmd.AddCommand(execNodeCmd)
}
