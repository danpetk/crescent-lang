#!/usr/bin/env bash

# Usage: $0 [file (default: test.crsnt)] [-a assembler (default: gcc)]
# Example: ./compile_and_run.sh other.crsnt -a clang

file="${1:-test.crsnt}"
assembler="gcc"

shift
while getopts "a:" opt; do
    case $opt in
    a) assembler="$OPTARG" ;;
    *)
        echo "Usage: $0 [file (default: test.crsnt)] [-a assembler (default: gcc)]"
        exit 1
        ;;
    esac
done

cargo run "$file" && "$assembler" out.s && ./a.out
