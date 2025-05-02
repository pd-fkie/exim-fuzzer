#!/bin/bash
set -e
test -d /shared
taskset -c 0 /aflnet/afl-fuzz -i /corpus/ -o /shared/output -m none -x /exim-fuzzer/smtp.dict -N tcp://127.0.0.1/2525 -P SMTP -K -E -R -q 3 -s 3 -- /exim-fuzzer/Exim/src/build-Linux-x86_64/exim -bdf

