#!/bin/bash
set -eux
cargo fmt --check
cargo clippy
cargo test
