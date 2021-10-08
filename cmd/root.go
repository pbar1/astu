package cmd

import (
	"github.com/spf13/cobra"
)

var rootFlagColor bool

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "astu",
	Short: "All-Seeing Trace Utility",
	Long: `All-Seeing Trace Utility

Hello, friend.`,
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute(version, commit, date, builtBy string) {
	versionInfo = VersionInfo{version, commit, date, builtBy}

	cobra.CheckErr(rootCmd.Execute())
}

func init() {
	rootCmd.PersistentFlags().BoolVar(&rootFlagColor, "no-color", false, "(WIP) Disables colorized output")
}
