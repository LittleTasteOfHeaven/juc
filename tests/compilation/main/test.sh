#!/bin/bash

source tests/test.sh

print_test_name
$BIN -d tests/compilation/main/ test.ju -o test_main
./tests/compilation/main/test_main
print_test_end $?
