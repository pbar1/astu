package util

import (
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
)

const (
	filterCmd      = "fzf"
	multiFilterCmd = "fzf -m"
)

func StringFilter(input []string) string {
	command := filterCmd
	shell := os.Getenv("SHELL")
	if len(shell) == 0 {
		shell = "sh"
	}
	cmd := exec.Command(shell, "-c", command)
	cmd.Stderr = os.Stderr
	in, _ := cmd.StdinPipe()
	go func() {
		for _, item := range input {
			fmt.Fprintln(in, item)
		}
		in.Close()
	}()
	result, _ := cmd.Output()
	return strings.TrimSpace(string(result))
}

func GenericFilter(input func(in io.WriteCloser)) string {
	command := filterCmd
	shell := os.Getenv("SHELL")
	if len(shell) == 0 {
		shell = "sh"
	}
	cmd := exec.Command(shell, "-c", command)
	cmd.Stderr = os.Stderr
	in, _ := cmd.StdinPipe()
	go func() {
		input(in)
		in.Close()
	}()
	result, _ := cmd.Output()
	return string(result)
}

// GenericMultiFilter shells out to the fzf binary to perform user-controlled filtering
// on a given input. From https://junegunn.kr/2016/02/using-fzf-in-your-program
func GenericMultiFilter(input func(in io.WriteCloser)) []string {
	command := multiFilterCmd
	shell := os.Getenv("SHELL")
	if len(shell) == 0 {
		shell = "sh"
	}
	cmd := exec.Command(shell, "-c", command)
	cmd.Stderr = os.Stderr
	in, _ := cmd.StdinPipe()
	go func() {
		input(in)
		in.Close()
	}()
	result, _ := cmd.Output()
	return strings.Split(string(result), "\n")
}
