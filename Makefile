# Sanity
# From: https://tech.davis-hansson.com/p/make/

SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

# Convenience targets

test:
	cargo test
.PHONY: test

clean:
	cargo clean
	rm -f flamegraph.svg
.PHONY: clean

format:
	cargo fmt
.PHONY: format

lint:
	cargo fmt
	cargo clippy
.PHONY: lint

build: target/release/mosaic target/release/tile
.PHONY: build

tile: tile.jpg
.PHONY: tile

tile.jpg: target/release/tile
	time target/release/tile images/2.jpg > tile.jpg
#	chafa tile.jpg

mosaic: mosaic.jpg
.PHONY: mosaic

mosaic.jpg: target/release/mosaic
	time target/release/mosaic images/242.jpg tiles_lib/ > mosaic.jpg
#	chafa mosaic.jpg

target/release/%:
	cargo build --release

performance:
	cargo flamegraph --root --bin mosaic -- images/3.jpg tiles_lib/ > mosaic.jpg
#	chafa flamegraph.svg
.PHONY: performance

bench:
	cargo bench
.PHONY: bench
