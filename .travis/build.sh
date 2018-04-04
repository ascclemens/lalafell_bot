#!/usr/bin/env sh

set -ex

if [ "$TRAVIS_BRANCH" = "release" ]; then
  cargo build --verbose --release --bin lalafell_bot
else
  cargo build --verbose --bin lalafell_bot
  cargo build --verbose --features source --bin generate_handlers
fi
