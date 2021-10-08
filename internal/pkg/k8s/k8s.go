package k8s

import (
	"fmt"
	"os"
	"os/exec"
	"path"
	"strings"

	"github.com/ktr0731/go-fuzzyfinder"
	"github.com/pbar1/astu/internal/pkg/util"
	"gopkg.in/yaml.v3"
)

type Node struct {
	Name   string
	Labels map[string]string
}

type Pod struct {
	Name       string
	Namespace  string
	Status     string
	Containers []Container
}

type Container struct {
	Name  string
	Image string
}

var (
	KubectlBinary = "kubectl"
	RunCtrPrefix  = "astu-runctr"
)

func Check() error {
	cmd := exec.Command(KubectlBinary)
	return cmd.Run()
}

func GetNodes(args ...string) ([]Node, error) {
	a := []string{"get", "nodes", `--output=go-template={{range .items}}{{printf "- name: %s\n  labels:\n" .metadata.name}}{{range $k, $v := .metadata.labels}}{{printf "    %s: %s\n" $k $v}}{{end}}{{end}}`}
	a = append(a, args...)
	cmd := exec.Command(KubectlBinary, a...)

	raw, err := cmd.Output()
	if err != nil {
		return nil, err
	}

	var nodes []Node
	if err := yaml.Unmarshal(raw, &nodes); err != nil {
		return nil, err
	}

	return nodes, nil
}

func GetPods(args ...string) ([]Pod, error) {
	a := []string{"get", "pods", `--output=go-template={{range .items}}{{printf "- name: %s\n  namespace: %s\n  status: %s\n  containers:\n" .metadata.name .metadata.namespace .status.phase}}{{range .spec.containers}}{{printf "  - name: %s\n    image: %s\n" .name .image}}{{end}}{{end}}`}
	a = append(a, args...)
	cmd := exec.Command(KubectlBinary, a...)

	raw, err := cmd.Output()
	if err != nil {
		return nil, err
	}

	var pods []Pod
	if err := yaml.Unmarshal(raw, &pods); err != nil {
		return nil, err
	}

	return pods, nil
}

func GetShells(namespace, pod, container string) ([]string, error) {
	a := []string{"exec", pod, "--namespace=" + namespace, "--container=" + container, "--", "cat", "/etc/shells"}
	cmd := exec.Command(KubectlBinary, a...)

	raw, err := cmd.Output()
	if err != nil {
		return nil, err
	}
	str := string(raw)

	return strings.Split(str, "\n"), nil
}

func ExecContainer(namespace, pod, container, shell string) error {
	a := []string{"exec", pod, "--namespace=" + namespace, "--container=" + container, "--stdin", "--tty", "--", shell}
	cmd := exec.Command(KubectlBinary, a...)

	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	err := cmd.Run()
	return err
}

func FullExecContainer(args ...string) error {
	pods, err := GetPods(args...)
	if err != nil {
		return fmt.Errorf("unable to get pods: %v", err)
	}

	podIdx, err := fuzzyfinder.Find(pods, func(i int) string {
		return fmt.Sprintf("%s/%s [%s]", pods[i].Namespace, pods[i].Name, pods[i].Status)
	})
	if err != nil {
		return fmt.Errorf("unable to filter pods: %v", err)
	}

	pod := pods[podIdx]
	ctrs := pod.Containers

	ctrIdx, err := fuzzyfinder.Find(ctrs, func(i int) string {
		return fmt.Sprintf("%s [%s]", ctrs[i].Name, ctrs[i].Image)
	})
	if err != nil {
		return fmt.Errorf("unable to filter containers: %v", err)
	}

	ctr := ctrs[ctrIdx]

	shells, err := GetShells(pod.Namespace, pod.Name, ctr.Name)
	if err != nil {
		return fmt.Errorf("unable to get shells: %v", err)
	}

	shellIdx, err := fuzzyfinder.Find(shells, func(i int) string {
		return shells[i]
	})
	if err != nil {
		return fmt.Errorf("unable to filter shells: %v", err)
	}

	shell := shells[shellIdx]

	err = ExecContainer(pod.Namespace, pod.Name, ctr.Name, shell)
	if err != nil {
		return fmt.Errorf("error exec-ing container: %v", err)
	}

	return nil
}

func RunContainer(image string, args ...string) error {
	name := path.Base(image)
	name = strings.SplitN(name, ":", 2)[0]
	name = strings.SplitN(name, "@", 2)[0]
	pod := fmt.Sprintf("%s-%s-%s", RunCtrPrefix, name, util.RandHash(6))

	aShells := []string{"run", pod + "-temp", "--image=" + image, "--rm", "--stdin", "--tty", "--restart=Never", "--command"}
	aShells = append(aShells, args...)
	aShells = append(aShells, "--", "cat", "/etc/shells")
	cmdShells := exec.Command(KubectlBinary, aShells...)

	raw, err := cmdShells.Output()
	if err != nil {
		return err
	}
	str := string(raw)
	str = strings.ReplaceAll(str, "\r", "")
	shells := strings.Split(str, "\n")

	idx, err := fuzzyfinder.Find(shells, func(i int) string {
		return shells[i]
	})
	shell := shells[idx]

	a := []string{"run", pod, "--image=" + image, "--rm", "--restart=Never", "--stdin", "--tty", "--command"}
	a = append(a, args...)
	a = append(a, "--", shell)
	cmd := exec.Command(KubectlBinary, a...)

	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	err = cmd.Run()
	return err
}

// https://github.com/alexei-led/nsenter/blob/master/nsenter-node.sh
// func ExecNode(node string) error {}
