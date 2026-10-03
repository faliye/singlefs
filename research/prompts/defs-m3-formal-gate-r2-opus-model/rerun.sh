#!/usr/bin/env bash
# 复跑这条腿的全部探针：bash rerun.sh <仓根> <草稿目录（不存在或空）>。不跑门禁、不跑判法脚本的干活用法；钩子只喂 JSON。
# e2-verus-toy.sh 直接起 Verus 验一份三行的玩具文件 5 次（每次经内存包装 500M、capped.sh 4），别的都不起 Verus、不编。挂钟约 3 分钟（拷仓与拷 Verus 安装占大半）。
set -uo pipefail
REPO="${1:?仓根}"; WORK="${2:?草稿目录}"; HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$WORK"
rsync -a --exclude target --exclude .git "$REPO/" "$WORK/repo/"
for variant in g9 g10; do rsync -a "$WORK/repo/" "$WORK/repo-$variant/"; done
python3 "$HERE/fix-draft-e1.py" "$WORK/repo-g9"
python3 "$HERE/fix-draft-e2.py" "$WORK/repo-g10"
python3 "$HERE/e1-hook-cases.py" "$WORK/repo" "$WORK/e1" > "$WORK/e1-output.txt" 2>&1
python3 "$HERE/e1b-shared-shapes.py" "$WORK/repo" "$WORK/e1b" > "$WORK/e1b-output.txt" 2>&1
python3 "$HERE/e1-hook-cases.py" "$WORK/repo-g9" "$WORK/e1-g9" > "$WORK/e1-G9-output.txt" 2>&1
python3 "$HERE/e1b-shared-shapes.py" "$WORK/repo-g9" "$WORK/e1b-g9" > "$WORK/e1b-G9-output.txt" 2>&1
bash "$HERE/e2-verus-toy.sh" "$WORK/repo" "$WORK/e2toy" > "$WORK/e2-verus-toy-output.txt" 2>&1
bash "$HERE/e2-reuse.sh" "$WORK/repo" "$WORK/e2toy" > "$WORK/e2-reuse-output.txt" 2>&1
bash "$HERE/e2-reuse.sh" "$WORK/repo-g10" "$WORK/e2toy" > "$WORK/e2-reuse-G10-output.txt" 2>&1
python3 "$HERE/e2-lkmm-hook.py" "$WORK/repo" "$WORK/e2lkmm" > "$WORK/e2-lkmm-hook-output.txt" 2>&1
python3 "$HERE/e3-mixed.py" "$WORK/repo" "$WORK/e3" > "$WORK/e3-mixed-output.txt" 2>&1
for name in e1-output e1b-output e1-G9-output e1b-G9-output e2-reuse-output e2-reuse-G10-output e2-lkmm-hook-output e3-mixed-output; do
  if diff -q <(sed 's#/tmp/[^ 	]*##g' "$HERE/$name.txt") <(sed 's#/tmp/[^ 	]*##g' "$WORK/$name.txt") >/dev/null; then echo "相同 $name"; else echo "不同 $name"; fi
done
