#!/usr/bin/env bash
# 红样本：调共用取法不判退出码
source "$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/changed-paths.sh" || exit 1
base="$(gate_diff_base gate)"
changed="$(gate_changed_paths "$base" untracked)"
