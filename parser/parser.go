package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"strings"
)

func main() {
	file, err := os.Open("./lotto.qnt")
	if err != nil {
		fmt.Errorf("Unable to open file")
		return
	}
	defer file.Close()
	scanner := bufio.NewScanner(file)
	lines := []string{}
	for scanner.Scan() {
		//fmt.Println(scanner.Text())
		lines = append(lines, scanner.Text())
	}

	if err := scanner.Err(); err != nil {
		log.Fatal(err)
	}
	//for _, line := range lines {
	//	fmt.Println(line)
	//}
	parseStructs(lines)
}

func hasStruct(line string) bool {
	return strings.Contains(line, "state_struct")
}

func parseStruct(lines []string, start int) int {
	tmp := []string{}
	cursor := start
	fmt.Printf("here is the line: %s\n", lines[start])
	for {
		if strings.Contains(lines[cursor], "}") {
			tmp = append(tmp, lines[cursor])
			break
		}
		tmp = append(tmp, lines[cursor])
		cursor++
	}
	for _, line := range tmp {
		fmt.Println(line)
	}
	return cursor
}

func parseStructs(lines []string) {
	cursor := 0
	lastIdx := len(lines) - 1
	for i := cursor; i <= lastIdx; i++ {
		if hasStruct(lines[i]) {
			fmt.Printf("Current index: %d\n", i)
			newCursor := parseStruct(lines, i)
			fmt.Printf("This is the new cursor value: %d\n\n", newCursor)
		}
	}
}
