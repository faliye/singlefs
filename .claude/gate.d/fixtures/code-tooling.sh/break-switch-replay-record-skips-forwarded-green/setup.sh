#!/usr/bin/env bash
# code-tooling 的 break-switch-replay 格转「!」行样本的现场：git init（按条复用的判定库名字取被判那棵树的 git common-dir），.unit-verdict-sample/ 不进 git
set -euo pipefail
git init -q .
printf '%s\n' .unit-verdict-sample/ >> .git/info/exclude
