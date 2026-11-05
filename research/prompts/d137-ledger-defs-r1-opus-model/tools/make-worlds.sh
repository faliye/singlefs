#!/usr/bin/env bash
# 造样本根：每个世界的 before/ 拷进 <输出目录>/<世界名>/ 并提交成基准，再把 after/（跑前登记）拷进去、不提交（门禁照「这次改动」认它）。
# 用法：bash make-worlds.sh <worlds 源目录> <输出目录：不在或是空目录>
# 退出码：0 造完；2 用法错或输出目录非空；别的是 git 或 cp 的退出码。
set -euo pipefail
source_directory="${1:?要给 worlds 源目录}"
out="${2:?要给输出目录}"
if [[ -e "$out" && -n "$(ls -A "$out")" ]]; then
  echo "  ✗ 输出目录 $out 已存在且非空：给一个新的" >&2
  exit 2
fi
mkdir -p "$out"
for name in control-single-line-red control-single-line-green continued-bin-on-next-line continued-features-on-next-line marker-without-judge alias-import-judge declared-cargo-test-green; do
  world="$out/$name"
  mkdir -p "$world"
  cp -a "$source_directory/$name/before/." "$world/"
  (cd "$world" && git init -q . && git config user.email selftest@example.invalid && git config user.name selftest \
    && git add -A && git commit -qm '基准：checker 档包清单与一个装置 bin')
  cp -a "$source_directory/$name/after/." "$world/"
  echo "name=world_built world=$name untracked=$(cd "$world" && git status --porcelain --untracked-files=all | grep -c '^??')"
done
