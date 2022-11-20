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
.PHONY: clean

format:
	cargo fmt
.PHONY: format

dev:
	cargo run -- images > output.jpg
	chafa output.jpg
.PHONY: dev

build:
	cargo build --release
.PHONY: build

performance:
	cargo flamegraph --root -- images > output.jpg
.PHONY: performance
