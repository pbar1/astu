package cmd

import (
	"fmt"
	"os"
	"time"

	"github.com/pbar1/astu/internal/pkg/ping"
	"github.com/spf13/cobra"
)

var (
	pingOpts    *ping.PingOptions
	pingFlagTCP bool
	pingFlagUDP bool
)

// pingCmd represents the ping command
var pingCmd = &cobra.Command{
	Use:   "ping",
	Short: "Check connectivity to a target",
	Long:  `Check connectivity to a target`,
	Run: func(cmd *cobra.Command, args []string) {
		pingOpts.RawURL = args[0]

		if pingFlagTCP {
			pingOpts.Scheme = "tcp"
		} else if pingFlagUDP {
			pingOpts.Scheme = "udp"
		}

		if err := ping.Ping(pingOpts); err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
	},
}

func init() {
	rootCmd.AddCommand(pingCmd)

	pingOpts = &ping.PingOptions{}

	pingCmd.Flags().DurationVarP(&pingOpts.Timeout, "timeout", "w", 5*time.Second, "Connection time limit")
	pingCmd.Flags().BoolVarP(&pingFlagTCP, "tcp", "t", false, "Force use TCP")
	pingCmd.Flags().BoolVarP(&pingFlagUDP, "udp", "u", false, "Force use UDP")
	pingCmd.Flags().BoolVarP(&pingOpts.AllowIPv6, "ipv6", "6", false, "Allow IPv6 addresses")
}
