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
build_root_dir="$temp_dir/buildroot"
mkdir -p "$build_root_dir"

step "Using directory $temp_dir"

function cleanup {
  echo "Cleaning directory $temp_dir"
  rm -rf "$temp_dir"
}

trap cleanup EXIT

ref=$(git stash create)
if [[ -z "$ref" ]];
  then ref=HEAD
fi
step "Exporting working tree ($ref)"
git archive --format tar.gz --output "$source_dir/sources.tar.gz" "$ref"

step "Building RPM packages"
rpmbuild -ba outbox.spec \
    --define "_sourcedir $source_dir" \
    --define "_builddir $build_dir" \
    --define "_buildrootdir $build_root_dir" \
    --buildroot "$build_root_dir" \
    --define '_rpmdir _rpms' \
    --define '_srcrpmdir _rpms'
