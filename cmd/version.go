package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

type VersionInfo struct {
	Version string
	Commit  string
	Date    string
	BuiltBy string
}

var versionInfo VersionInfo

// versionCmd represents the version command
var versionCmd = &cobra.Command{
	Use:   "version",
	Short: "Version and build info for this program",
	Long:  `Version and build info for this program`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Printf("version: %s\n", versionInfo.Version)
		fmt.Printf("commit: %s\n", versionInfo.Commit)
		fmt.Printf("date: %s\n", versionInfo.Date)
		fmt.Printf("builtBy: %s\n", versionInfo.BuiltBy)
	},
}

func init() {
	rootCmd.AddCommand(versionCmd)
}
