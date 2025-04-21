@default:
    just --list

build-container:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(cargo metadata --no-deps --format-version 1 | jq --raw-output '.packages[] | select(.name == "outboxd") | .version')
    podman build -t "codeberg.org/tautropfli/outboxd:${version}" .
