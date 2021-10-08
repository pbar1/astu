package util

import (
	"math/rand"
	"time"
)

const DictLowerAlphaNum = "0123456789abcdefghijklmnopqrstuvwxyz"

func init() {
	rand.Seed(time.Now().Unix())
}

// RandHash generates a random string of lowercase alphanumeric characters of the given length.
func RandHash(length int) string {
	bytes := make([]byte, length)
	rand.Read(bytes)
	for k, v := range bytes {
		bytes[k] = DictLowerAlphaNum[v%byte(len(DictLowerAlphaNum))]
	}
	return string(bytes)
}
