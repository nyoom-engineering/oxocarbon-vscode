#!/bin/bash

nix run nixpkgs#hyperfine -- --warmup 3 'make all'

for i in {1..100}; do ./scripts/benchneon.py --iterations 2 --warmups 0 --tokens 1000 --color-keys 200 | grep "Speedup" | awk '{print $3}' | sed 's/x//'; done | awk '{sum+=$1; count++} END {print "Average speedup: " sum/count "x"}'