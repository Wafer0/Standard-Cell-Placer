.PHONY: all clean rust latex report help test validate visualize benchmark

# Default target
all: rust

# Build Rust project
rust:
	@echo "Building Rust project..."
	cargo build --release
	@echo "Copying executables to build/..."
	@mkdir -p build
	@if [ -f "target/release/floorplan" ]; then \
		cp target/release/floorplan build/floorplan; \
	fi
	@if [ -f "target/release/validate" ]; then \
		cp target/release/validate build/validate; \
	fi
	@if [ -f "target/release/visualize" ]; then \
		cp target/release/visualize build/visualize; \
	fi
	@if [ -f "target/release/benchmark" ]; then \
		cp target/release/benchmark build/benchmark; \
	fi
	@echo "Build complete!"

# Compile LaTeX report
latex: report

report:
	@echo "Compiling LaTeX report..."
	@mkdir -p docs
	cd docs && pdflatex -interaction=nonstopmode report.tex
	cd docs && pdflatex -interaction=nonstopmode report.tex
	@echo "Report compiled to docs/report.pdf"

# Build both Rust and LaTeX
build: rust latex

# Run quick test on B10
test: rust
	@echo "Running quick test on B10..."
	./build/floorplan B10 0.5 100 1000 0.01

# Validate a placement
validate: rust
	@echo "Usage: make validate FILE=B10"
	@if [ -n "$(FILE)" ]; then \
		./build/validate $(FILE); \
	fi

# Visualize a placement
visualize: rust
	@echo "Usage: make visualize FILE=B10"
	@if [ -n "$(FILE)" ]; then \
		./build/visualize $(FILE); \
	fi

# Run comprehensive benchmark suite
benchmark: rust
	@echo "Running comprehensive benchmark suite..."
	./build/benchmark

# Clean build artifacts
clean:
	@echo "Cleaning..."
	cargo clean
	rm -rf build/*
	rm -f docs/*.aux docs/*.log docs/*.out docs/*.toc
	@echo "Clean complete!"

# Clean everything including PDF and outputs
clean-all: clean
	rm -f docs/report.pdf
	rm -f out/*.result out/*.pl out/*.SA out/*.svg out/*.json

# Help target
help:
	@echo "Available targets:"
	@echo "  all         - Build Rust project (default)"
	@echo "  rust        - Build Rust project only"
	@echo "  latex       - Compile LaTeX report"
	@echo "  report      - Alias for latex"
	@echo "  build       - Build both Rust and LaTeX"
	@echo "  test        - Run quick test on B10"
	@echo "  validate    - Validate placement (use FILE=B10)"
	@echo "  visualize   - Generate SVG visualization (use FILE=B10)"
	@echo "  benchmark   - Run comprehensive benchmark suite"
	@echo "  clean       - Clean build artifacts"
	@echo "  clean-all   - Clean everything including outputs"
	@echo "  help        - Show this help message"
