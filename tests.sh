#!/bin/bash
set -e

for INPUT in ./testdata/input*.txt
do
  OUTPUT=${INPUT//input/output}

  RES=/tmp/$(basename "$OUTPUT")

  echo "Running test $INPUT:"
  cargo run --bin weighted_path -- "$INPUT" | tee "$RES"

  diff -w "$OUTPUT" "$RES"

#  rm -f "$RES"
done
