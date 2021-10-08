package main

import "github.com/pbar1/astu/cmd"

var (
	version = "dev"
	commit  = "none"
	date    = "unknown"
	builtBy = "unknown"
)

func main() {
	cmd.Execute(version, commit, date, builtBy)
}
