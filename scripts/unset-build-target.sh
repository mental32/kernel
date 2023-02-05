#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

ROOT=`git rev-parse --show-toplevel`
cd $ROOT

if [[ -e .vscode/settings.json ]]; then
    rm -v .vscode/settings.json
fi

if [[ -e .cargo/config.toml ]]; then
    rm -v .cargo/config.toml
fi

