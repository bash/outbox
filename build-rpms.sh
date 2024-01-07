#!/usr/bin/env bash

set -e

function step {
    echo "▷ $(tput bold)$1$(tput sgr0)"
}

temp_dir=$(mktemp -d -t outbox-rpm.XXXXXXXXXX)
source_dir="$temp_dir/source"
mkdir -p "$source_dir"
build_dir="$temp_dir/build"
mkdir -p "$build_dir"

step "Using directory $temp_dir"

function cleanup {
  echo "Cleaning directory $temp_dir"
  rm -rf "$temp_dir"
}

trap cleanup EXIT

step "Exporting working tree"
stash=$(git stash create)
git archive --format tar.gz --output "$source_dir/sources.tar.gz" "$stash"

step "Building RPM packages"
rpmbuild -ba outbox.spec \
    --define "_sourcedir $source_dir" \
    --define "_builddir $build_dir" \
    --buildroot "$temp_dir/buildroot" \
    --define '_rpmdir _rpms'
