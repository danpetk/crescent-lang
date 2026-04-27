#!/usr/bin/env bash
# Compile and run the test file, then print the exit code returned from main
cargo run test.crsnt && g++ out.s && ./a.out
echo $?
