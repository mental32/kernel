#!/bin/bash

set -eo pipefail
IFS=$'\n\t'

ROOT=`git rev-parse --show-toplevel`
cd $ROOT

if [[ -z $1 ]]; then
    echo "error: you must specify the target architecture"
    echo "info: available architectures:"
    for file in `ls *-target.json`; do
        echo "- $file"
    done
    exit 1
elif [[ ! -e $1 ]]; then
    echo "error: target architecture file does not exist ($1)"
    exit 1
fi

set -u # using this before the checks would give unhelpful errors like '$1 is unbound'

cat .vscode/settings-base.json | jq ". += { \"rust-analyzer.checkOnSave.extraArgs\": [\"--target\", \"$1\"] }" > .vscode/settings.json
cp .cargo/config-base.toml .cargo/config.toml
