#!/usr/bin/env bash
# Measure click-to-editable project load time of a phichain editor binary.
# Usage: bench_load.sh <editor-binary> <project-dir> [runs]
set -euo pipefail

BIN="$1"
PROJECT="$2"
RUNS="${3:-5}"

results=()
for i in $(seq 1 "$RUNS"); do
    out=$(PHICHAIN_BENCH_LOAD=1 "$BIN" --project "$PROJECT" 2>/dev/null | grep -o 'BENCH_LOAD_MS=[0-9]*' | cut -d= -f2)
    if [ -z "$out" ]; then
        echo "run $i: FAILED (no BENCH_LOAD_MS in output)" >&2
        exit 1
    fi
    echo "run $i: ${out}ms"
    results+=("$out")
done

median=$(printf '%s\n' "${results[@]}" | sort -n | awk '{a[NR]=$1} END {print (NR%2) ? a[(NR+1)/2] : int((a[NR/2]+a[NR/2+1])/2)}')
echo "median: ${median}ms over $RUNS runs"
