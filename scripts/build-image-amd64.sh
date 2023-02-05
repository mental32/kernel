#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

ROOT=`git rev-parse --show-toplevel`
cd $ROOT

echo "Building kernel crate with amd64 target file."
cargo build -p kernel --target ./amd64-target.json
