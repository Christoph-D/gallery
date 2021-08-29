#!/bin/bash
set -eu

readonly p=$(readlink -f "$(dirname "$0")")

if [[ $# -ne 1 ]]; then
    echo "Please provide the path to the gallery source directory with the branch 'main' checked out."
    exit 1
fi

cd "$1"

cargo run -- --page_title='Example gallery' \
  --footer='All rights reserved. Contact: <a href="mailto:github@yozora.eu">github@yozora.eu</a>' \
  --input="$p/source" --output="$p"
