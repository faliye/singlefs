#!/usr/bin/env bash
# 造世界根（d137-ledger-defs-r3 攻方腿）：每个世界先拷 worlds-shared/judge-sources/（checker 档包清单、crash_amplification.rs、research/prompts/README.md），
# 再拷这个世界的 before/ 盖上去，提交成基准；有 after/ 的（跑前登记）拷进去、不提交（门禁照「这次改动」认它）。
# 用法：bash make-worlds.sh <模型目录> <输出目录：不在或是空目录>
# 退出码：0 造完；2 用法错或输出目录非空；别的是 git 或 cp 的退出码。只拷文件、只起 git 与 bash 自己，不起本仓的代码。
set -euo pipefail
model="${1:?要给模型目录}"
out="${2:?要给输出目录}"
if [[ -e "$out" && -n "$(ls -A "$out")" ]]; then
  echo "  ✗ 输出目录 $out 已存在且非空：给一个新的" >&2
  exit 2
fi
mkdir -p "$out"
for name in sync-array-parameter-new-entry sync-slice-parameter-new-entry-control sync-array-parameter-on-the-path \
            sync-slice-parameter-on-the-path-control sync-cycle-new-entry-after sync-cycle-new-entry-before-control \
            chain-array-parameter-unjudged prereg-list-item-trailing-comment prereg-table-cell-trailing-comment \
            prereg-fence-trailing-comment-control; do
  world="$out/$name"
  mkdir -p "$world"
  cp -a "$model/worlds-shared/judge-sources/." "$world/"
  cp -a "$model/worlds/$name/before/." "$world/"
  (cd "$world" && git init -q . && git config user.email selftest@example.invalid && git config user.name selftest \
    && git add -A && git commit -qm '基准：checker 档包清单、判段源文件与装置 bin')
  if [[ -d "$model/worlds/$name/after" ]]; then
    cp -a "$model/worlds/$name/after/." "$world/"
  fi
  echo "name=world_built world=$name tracked=$(cd "$world" && git ls-files | wc -l) untracked=$(cd "$world" && git status --porcelain --untracked-files=all | grep -c '^??' || true)"
done
