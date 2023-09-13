#!/bin/bash

set -e

VERSION=$(git describe --tags `git rev-list --tags --max-count=1` --always)

# dynamically pull more interesting stuff from latest git commit
HASH=$(git show-ref --head --hash=8 head)  # first 8 letters of hash should be enough; that's what GitHub uses

# Change the version in the Cargo.toml file
TRIMMED_VERSION=$(echo $VERSION | sed 's/^v//')
# If running on macOS, use sed -i '' instead of sed -i
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i "" "s/^version = \".*\"/version = \"${TRIMMED_VERSION}\"/g" Cargo.toml
else
  sed -i "s/^version = \".*\"/version = \"${TRIMMED_VERSION}\"/g" Cargo.toml
fi

# Build standalone docker image
docker build -t nordata/biomedgps:${VERSION}-${HASH} .

if [ "$1" == "--push" ]; then
  docker push nordata/biomedgps:${VERSION}-${HASH}
  docker tag nordata/biomedgps:${VERSION}-${HASH} ghcr.io/yjcyxky/biomedgps:${VERSION}-${HASH} && \
  docker push ghcr.io/yjcyxky/biomedgps:${VERSION}-${HASH}
fi
