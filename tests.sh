#!/bin/bash
set -e

PASSED=0
FAILED=0

echo "=== Running valid input tests ==="
for INPUT in ./testdata/input*.txt
do
  OUTPUT=${INPUT//input/output}

  if [ ! -f "$OUTPUT" ]; then
    echo "Skipping $INPUT (no expected output file)"
    continue
  fi

  RES=/tmp/$(basename "$OUTPUT")

  echo -n "Testing $INPUT: "
  # Run and capture only the actual output (skip cargo compilation messages)
  if cargo run --quiet --bin weighted_path -- "$INPUT" > "$RES" 2>&1; then
    # Extract just the last line (the actual output)
    ACTUAL=$(tail -n 1 "$RES")
    EXPECTED=$(cat "$OUTPUT" | tr -d '\n')
    if [ "$ACTUAL" = "$EXPECTED" ]; then
      echo "PASSED"
      ((PASSED++))
    else
      echo "FAILED (output mismatch)"
      echo "Expected: $EXPECTED"
      echo "Got: $ACTUAL"
      ((FAILED++))
    fi
  else
    echo "FAILED (program exited with error)"
    cat "$RES"
    ((FAILED++))
  fi

  rm -f "$RES"
done

echo ""
echo "=== Running invalid input tests ==="
for INPUT in ./testdata/invalid*.txt
do
  if [ ! -f "$INPUT" ]; then
    continue
  fi

  ERROR_OUTPUT=${INPUT//invalid/error_invalid}

  if [ ! -f "$ERROR_OUTPUT" ]; then
    echo "Skipping $INPUT (no expected error output file)"
    continue
  fi

  RES=/tmp/$(basename "$ERROR_OUTPUT")

  echo -n "Testing $INPUT: "
  # Run and capture stderr (errors go to stderr)
  if cargo run --quiet --bin weighted_path -- "$INPUT" > /dev/null 2>"$RES"; then
    echo "FAILED (expected error but program succeeded)"
    ((FAILED++))
  else
    # Extract error message (everything after "Error: ")
    ERROR_MSG=$(grep "Error: " "$RES" | sed 's/Error: //' | tr -d '\n')
    EXPECTED=$(cat "$ERROR_OUTPUT" | tr -d '\n')
    if [ "$ERROR_MSG" = "$EXPECTED" ]; then
      echo "PASSED"
      ((PASSED++))
    else
      echo "FAILED (error message mismatch)"
      echo "Expected: $EXPECTED"
      echo "Got: $ERROR_MSG"
      ((FAILED++))
    fi
  fi

  rm -f "$RES"
done

echo ""
echo "=== Summary ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"

if [ $FAILED -eq 0 ]; then
  echo "All tests passed!"
  exit 0
else
  echo "Some tests failed."
  exit 1
fi
