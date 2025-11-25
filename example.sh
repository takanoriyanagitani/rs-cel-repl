#!/bin/sh

# Exit immediately if a command exits with a non-zero status.
set -e

example1() {
	echo "--- Testing direct evaluation with context file ---"
	# Create a temporary context file
	jq -c -n '{"multiplier": 10, "offset": 5}' >example-ctx.json

	# Run cel-repl with context file and piped input
	echo "2" |
		./cel-repl -e "input * multiplier + offset" \
			--context-file example-ctx.json
	echo "3" |
		./cel-repl -e "input * multiplier + offset" \
			--context-file example-ctx.json

	rm example-ctx.json
}

example2() {
	echo "--- Testing direct evaluation with inline JSON context ---"
	# Run cel-repl with inline JSON context and piped input
	echo "2" |
		./cel-repl -e "input * multiplier + offset" \
			--json-context '{"multiplier": 20, "offset": 10}'
	echo "3" |
		./cel-repl -e "input * multiplier + offset" \
			--json-context '{"multiplier": 20, "offset": 10}'
}

example3() {
	echo "--- Testing direct evaluation without context ---"
	# Run cel-repl with a simple expression and piped input (input var defaults to "input")
	echo "10" | ./cel-repl -e "input > 5"
	echo "3" | ./cel-repl -e "input > 5"
}

example4() {
	echo "--- Testing with structured input (JSON objects) ---"
	# Run cel-repl with structured JSON input
	jq -c -n '{"name": "Alice", "age": 30}' |
		./cel-repl -e 'input.name + " is " + string(input.age) + " years old"'
	jq -c -n '{"name": "Bob", "age": 25}' |
		./cel-repl -e 'input.age < 30'
}

example5() {
	echo "--- Testing with nested context ---"
	# Create a temporary context file with nested structure
	jq -c -n '{"user": {"name": "Charlie", "role": "admin"}, "app": {"version": "1.0"}}' >example-nested-ctx.json

	# Run cel-repl with nested context
	echo 'true' |
		./cel-repl -e 'user.role == "admin" && app.version == "1.0"' \
			--context-file example-nested-ctx.json
	echo 'false' |
		./cel-repl -e 'user.name == "Guest"' \
			--context-file example-nested-ctx.json

	rm example-nested-ctx.json
}

example6() {
	echo "--- Testing with structured input and context ---"
	# Run cel-repl with structured JSON input and inline context
	jq -c -n '{"name": "Dave", "age": 20}' |
		./cel-repl -e 'input.name + " is " + (input.age >= adult_age ? "adult" : "not adult")' \
			--json-context '{"adult_age": 18}'
	jq -c -n '{"name": "Eve", "age": 16}' |
		./cel-repl -e 'input.name + " is " + (input.age >= adult_age ? "adult" : "not adult")' \
			--json-context '{"adult_age": 18}'
}

case "$1" in
1)
	example1
	;;
2)
	example2
	;;
3)
	example3
	;;
4)
	example4
	;;
5)
	example5
	;;
6)
	example6
	;;
*)
	example1
	echo ""
	example2
	echo ""
	example3
	echo ""
	example4
	echo ""
	example5
	echo ""
	example6
	echo ""
	echo "--- All direct evaluation tests passed ---"
	;;
esac
