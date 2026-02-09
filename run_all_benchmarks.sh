#!/bin/bash

# Comprehensive Benchmark Runner for Standard-Cell Placer
# Runs parallel SA (8 runs) for all optimization variations
# Usage: ./run_all_benchmarks.sh <variation> <alpha> <times> <init_temp> <term_temp> [timing_weight] [congestion_weight]

set -e

VARIATION=$1
ALPHA=$2
TIMES=$3
INIT_TEMP=$4
TERM_TEMP=$5
TIMING_WEIGHT=${6:-0}
CONGESTION_WEIGHT=${7:-0}

if [ -z "$VARIATION" ] || [ -z "$ALPHA" ] || [ -z "$TIMES" ] || [ -z "$INIT_TEMP" ] || [ -z "$TERM_TEMP" ]; then
    echo "Usage: $0 <variation> <alpha> <times> <init_temp> <term_temp> [timing_weight] [congestion_weight]"
    echo ""
    echo "Variations:"
    echo "  standard   - Area + wirelength optimization"
    echo "  timing     - With timing-driven placement (provide timing_weight)"
    echo "  congestion - With congestion-aware placement (provide congestion_weight)"
    echo "  combined   - With both timing and congestion"
    echo ""
    echo "Example:"
    echo "  $0 standard 0.5 100 1000 0.01"
    echo "  $0 timing 0.5 100 1000 0.01 0.1"
    echo "  $0 congestion 0.5 100 1000 0.01 0 0.05"
    echo "  $0 combined 0.5 100 1000 0.01 0.1 0.05"
    exit 1
fi

# Validate variation
if [ "$VARIATION" != "standard" ] && [ "$VARIATION" != "timing" ] && [ "$VARIATION" != "congestion" ] && [ "$VARIATION" != "combined" ]; then
    echo "Error: Invalid variation. Must be: standard, timing, congestion, or combined"
    exit 1
fi

# Set output directory
OUTPUT_DIR="out/$VARIATION"
mkdir -p "$OUTPUT_DIR"

echo "======================================================================="
echo "  RUNNING ALL BENCHMARKS - $VARIATION VARIATION"
echo "======================================================================="
echo "Parameters:"
echo "  Alpha: $ALPHA"
echo "  Times: $TIMES"
echo "  Init Temp: $INIT_TEMP"
echo "  Term Temp: $TERM_TEMP"
echo "  Timing Weight: $TIMING_WEIGHT"
echo "  Congestion Weight: $CONGESTION_WEIGHT"
echo "  Output: $OUTPUT_DIR/"
echo "======================================================================="
echo ""

BENCHMARKS=("B10" "B30" "B50" "B100")

for BENCH in "${BENCHMARKS[@]}"; do
    echo "=========================================="
    echo "  Running $BENCH ($VARIATION)"
    echo "=========================================="
    
    # Run benchmark with appropriate parameters
    ./build/benchmark "$BENCH" 8 "$ALPHA" "$TIMES" "$INIT_TEMP" "$TERM_TEMP" "$TIMING_WEIGHT" "$CONGESTION_WEIGHT" 2>&1 | tail -20
    
    # Move results to appropriate directory
    mv out/${BENCH}.result "$OUTPUT_DIR/" 2>/dev/null || true
    mv out/${BENCH}.pl "$OUTPUT_DIR/" 2>/dev/null || true
    mv out/${BENCH}.svg "$OUTPUT_DIR/" 2>/dev/null || true
    
    echo "✓ $BENCH complete - results in $OUTPUT_DIR/"
    echo ""
done

# Move benchmark summary
mv out/benchmark_results.json "$OUTPUT_DIR/" 2>/dev/null || true

echo "======================================================================="
echo "  ALL BENCHMARKS COMPLETE FOR $VARIATION"
echo "======================================================================="
echo "Results stored in: $OUTPUT_DIR/"
echo ""
