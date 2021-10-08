package util

import (
	"bytes"
	"io"
	"os/exec"
)

type CmdRunner interface {
	Check() error
	Version() (string, error)
}

// Run executes the program at the given path with the given args, and returns the output from
// stdout and stderr as bytes. Output may also be tee'd to an io.Writer for either one, or dropped
// if they are set to nil.
func Run(stdin io.Reader, stdout, stderr io.Writer, executable string, args ...string) ([]byte, []byte, error) {
	cmd := exec.Command(executable, args...)

	cmd.Stdin = stdin

	var stdoutBuf, stderrBuf bytes.Buffer
	cmd.Stdout = io.MultiWriter(stdout, &stdoutBuf)
	cmd.Stderr = io.MultiWriter(stderr, &stderrBuf)

	err := cmd.Run()
	outBytes, errBytes := bytes.TrimSpace(stdoutBuf.Bytes()), bytes.TrimSpace(stderrBuf.Bytes())
	return outBytes, errBytes, err
}
