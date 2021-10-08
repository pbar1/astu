package cmd

import (
	"github.com/spf13/cobra"
)

// k8sCmd represents the k8s command
var k8sCmd = &cobra.Command{
	Use:     "k8s",
	Aliases: []string{"kubernetes", "kube"},
	Short:   "Kubernetes commands",
	Long: `Kubernetes commands.

Flags/args after an initial "--" are passed through to kubectl.`,
}

func init() {
	rootCmd.AddCommand(k8sCmd)
}
