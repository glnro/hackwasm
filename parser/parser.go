package main

import (
	"bufio"
	"fmt"
	"log"
	"os"
	"strings"
)

const (
	STATE_FILE = "state.rs"
	ADDR       = "Addr"
	U32        = "U32"
	U128       = "U128"
	COIN       = "Coin"
	ARR_ADDR   = "Set[Addr]"
	TIMESTMP   = "Timestamp"
)

var (
	QUINT_RS = map[string]string{
		ADDR:     "Addr",
		U32:      "u32",
		U128:     "Uint128",
		COIN:     "Coin",
		ARR_ADDR: "Vec<Addr>",
		TIMESTMP: "Timestamp",
	}
)

func main() {
	file, err := os.Open("../quint/lotto.qnt")
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

	writeImports(STATE_FILE)

	parseStructs(lines)
}

func hasStruct(line string) bool {
	return strings.Contains(line, "SS_")
}

func hasContractState(line string) bool {
	return strings.Contains(line, "SCS_ContractState")
}

func parseContractState(lines []string, start int) int {
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
	endTmp := len(tmp) - 1
	fields := getFields(tmp[1:endTmp])
	// generate keys
	for k, v := range fields {
		fmt.Printf("This is contract state field: %s: %s\n", k, v)
	}
	writeStateStorage(fields)
	// generate storage
	return start
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
	name := getStructName(tmp[0])
	endTmp := len(tmp) - 1
	fields := getFields(tmp[1:endTmp])

	writeStruct(name, fields)
	fmt.Printf("Struct name: %s\n", name)
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

		if hasContractState(lines[i]) {
			fmt.Printf("Current index: %d\n", i)
			newCursor := parseContractState(lines, i)
			fmt.Printf("This is the new cursor value: %d\n\n", newCursor)
			return
		}
	}
}

func getFields(lines []string) map[string]string {
	fields := map[string]string{}
	for _, line := range lines {
		tokens := strings.Fields(line)
		fmt.Printf("This is a field string: %s", tokens)
		name := strings.Trim(tokens[0], ":")
		var fieldVal string
		if strings.Contains(line, "->") {
			fmt.Printf("\nthis is the tokens len: %d\n", len(tokens))
			fmt.Printf("\nTokens: 1: %s, 2: %s, 3: %s, 4: %s", tokens[0], tokens[1], tokens[2], tokens[3])
			fieldVal = fmt.Sprintf("Map:%s:%s", tokens[1], tokens[3])
		} else {
			fmt.Printf("\nThis is the normal token: %s\n", tokens[1])
			fieldVal = tokens[1]
		}
		fields[name] = fieldVal
	}

	return fields
}

func writeStruct(name string, fields map[string]string) {
	macros := fmt.Sprintf("\n#[cw_serde]")
	openClosure := fmt.Sprintf("\npub struct %s {", name)
	closeClosure := fmt.Sprintf("\n}\n")

	writeLines([]string{macros, openClosure}, STATE_FILE)
	writeFields(fields)
	writeLines([]string{closeClosure}, STATE_FILE)
}

func getStructName(line string) string {
	tokens := strings.Fields(line)
	trimmed := strings.Trim(tokens[1], "SS_")
	return trimmed
}

func writeFields(fields map[string]string) {
	fieldLines := []string{}
	for val, entry := range fields {
		fmt.Printf("fields :: %s: %s", val, entry)
		field := fmt.Sprintf("\n	pub %s: %s,", val, formatType(entry))
		if strings.Contains(val, "op_") {
			fieldKey := strings.Trim(val, "op_")

			field = fmt.Sprintf("\n	pub %s: Option<%s>,", fieldKey, formatType(entry))
		}
		fieldLines = append(fieldLines, field)
	}
	writeLines(fieldLines, STATE_FILE)
}

func writeStateStorage(fields map[string]string) {
	keyLines := []string{}
	storageLines := []string{}

	for val, entry := range fields {
		fmt.Printf("fields :: %s: %s", val, entry)
		storageType := "Item"
		entryType := strings.Trim(entry, ",")
		entryType = strings.Trim(entryType, "SS_")
		if strings.Contains(entry, "Map") {
			storageType = "Map"
			tokens := strings.Split(entry, ":")
			mapKey := QUINT_RS[tokens[1]]
			mapValue := strings.Title(strings.ToLower(tokens[2]))
			entryType = fmt.Sprintf("%s,%s", mapKey, mapValue)
		}
		storageKey := fmt.Sprintf("%s_KEY", strings.ToUpper(val))
		storageKeyLine := fmt.Sprintf("\npub const %s_KEY: &str = \"%s\";", strings.ToUpper(val), strings.ToLower(val))
		field := fmt.Sprintf("\npub const %s: %s<%s> = %s::new(%s);", strings.ToUpper(val), storageType, entryType, storageType, storageKey)
		if !strings.Contains(val, "time") {
			keyLines = append(keyLines, storageKeyLine)
			storageLines = append(storageLines, field)
		}
	}
	keyLines = append(keyLines, "\n")

	writeLines(keyLines, STATE_FILE)
	writeLines(storageLines, STATE_FILE)
}

func formatType(fieldType string) string {
	rsType := QUINT_RS[strings.Trim(fieldType, ",")]
	fmt.Printf("\nThis is the conversion: %s -> %s\n", fieldType, rsType)
	return rsType
}

func writeLines(lines []string, filename string) {
	f, err := os.OpenFile(filename, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0644)
	if err != nil {
		panic(err)
	}

	defer f.Close()

	for _, line := range lines {
		if _, err = f.WriteString(line); err != nil {
			log.Fatal(err)
		}
	}
}

func writeImports(file string) error {
	imports := fmt.Sprintf("use cosmwasm_schema::cw_serde;\nuse cosmwasm_std::{Addr, Coin, Timestamp, Uint128};\nuse cw_storage_plus::{Item, Map};\n")

	f, err := os.Create(file)
	if err != nil {
		return fmt.Errorf("Unable to write file")
	}
	f.WriteString(imports)

	return nil
}
