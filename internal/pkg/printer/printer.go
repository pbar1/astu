package printer

import (
	"fmt"

	"github.com/muesli/termenv"
)

var (
	SymbolGood    = termenv.String("✔").Foreground(termenv.ANSIGreen)
	SymbolBad     = termenv.String("✗").Foreground(termenv.ANSIRed)
	SymbolNeutral = termenv.String("•").Foreground(termenv.ANSIYellow)
)

func Goodf(style func(s *termenv.Style), format string, a ...interface{}) {
	printf(SymbolGood, style, format, a)
}

func Badf(style func(s *termenv.Style), format string, a ...interface{}) {
	printf(SymbolBad, style, format, a)
}

func Neutralf(style func(s *termenv.Style), format string, a ...interface{}) {
	printf(SymbolNeutral, style, format, a)
}

func printf(symbol termenv.Style, style func(s *termenv.Style), format string, a ...interface{}) {
	if style != nil {
		style(&symbol)
	}
	fmt.Printf("%v "+format, symbol, a)
}
