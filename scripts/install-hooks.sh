#!/bin/sh
# Points git at the repository hooks in .githooks/.
set -eu

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

git config core.hooksPath .githooks
echo "Git hooks installed: core.hooksPath=.githooks"
