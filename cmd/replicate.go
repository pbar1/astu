package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// replicateCmd represents the replicate command
var replicateCmd = &cobra.Command{
	Use:   "replicate",
	Short: "Install astu on a target",
	Long:  `Install astu on a target`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("replicate called")
	},
}

func init() {
	rootCmd.AddCommand(replicateCmd)
}
