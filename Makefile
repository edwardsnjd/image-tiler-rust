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

build:
	cargo build --release
.PHONY: build

tile:
	time target/release/tile images/2.jpg > tile2.jpg
	chafa tile2.jpg
.PHONY: tile

pile:
	time target/release/pile tile_images > pile.jpg
	chafa pile.jpg
.PHONY: pile

mosaic:
	time target/release/mosaic images/3.jpg tile_images > mosaic.jpg
	chafa mosaic.jpg
.PHONY: mosaic

performance:
	cargo flamegraph --root --bin mosaic -- images/3.jpg tile_images > mosaic.jpg
	chafa flamegraph.svg
.PHONY: performance
