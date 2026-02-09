# Benchmark Results

All results in this directory use **parallel simulated annealing with 8 independent runs**, automatically selecting the best solution for each benchmark.

## Directory Structure

```
out/
├── standard/           - Standard placement (area + wirelength)
├── timing/             - Timing-driven placement (critical path optimization)
├── congestion/         - Congestion-aware placement (routing resources)
└── combined/           - Combined timing + congestion optimization
```

## Benchmarks

Each variation contains results for:
- **B10** - 10 modules, 118 nets
- **B30** - 30 modules, 349 nets
- **B50** - 50 modules, 485 nets  
- **B100** - 100 modules, 885 nets

## Files Per Benchmark

- `<name>.result` - Summary metrics (modules, nets, wirelength, area, dimensions)
- `<name>.pl` - Placement coordinates (x, y positions)
- `<name>.svg` - Color-coded floorplan visualization
- `benchmark_results.json` - Statistical summary for the variation

## How Results Were Generated

All results generated using:
```bash
./run_all_benchmarks.sh <variation> 0.5 100 1000 0.01 [timing_weight] [congestion_weight]
```

**Parameters:**
- Alpha: 0.5 (balanced area/wirelength)
- Times: 100 iterations per temperature
- Init temp: 1000
- Term temp: 0.01
- Parallel runs: 8 (best result selected)

**Variation-specific weights:**
- Standard: timing_weight=0, congestion_weight=0
- Timing: timing_weight=0.1, congestion_weight=0
- Congestion: timing_weight=0, congestion_weight=0.05
- Combined: timing_weight=0.1, congestion_weight=0.05

## Quality Metrics

All placements validated with zero overlaps and achieve 89-95% area utilization.

Results vary by optimization objective - best variation depends on design goals.
