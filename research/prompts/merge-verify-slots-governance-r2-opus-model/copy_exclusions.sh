#!/usr/bin/env bash
# 攻方腿模型（正文判据格第 1 格「拷进槽之前删 target 一类」那一项）：冻结的 take_snapshot 与 refresh_slot_snapshot 原样起，
# 源树里放几样名字撞 target、target-*、__pycache__ 的东西，比两种快照里各剩什么，槽里事先放的旧东西删了哪些、留了哪些。
# 用法：bash copy_exclusions.sh --frozen <冻结副本> --repo <仓根> --work <新目录>
# 打：name=copy_exclusions kind=<一趟一份|槽> files=<份数> 与逐份 missing=<源树里有、快照里没有的>、slot_leftover=<槽里事先放的、拷完还在的>，
#     再一行 name=copy_exclusions_compare same=<yes|no>，一行 name=copy_exclusions_tracked count=<仓里跟踪的文件路径里有这三种目录名的份数>
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/slot_model_lib.sh"
model_arguments "$@"
model_load

source_tree="$MODEL_WORK/source"
mkdir -p -- "$source_tree/crates/a/src" "$source_tree/crates/a/tests/fixtures/target" "$source_tree/crates/a/src/target-spec" \
  "$source_tree/tools/__pycache__" "$source_tree/docs"
printf 'fn main() {}\n' > "$source_tree/crates/a/src/main.rs"
printf 'fixture under a directory named target\n' > "$source_tree/crates/a/tests/fixtures/target/data.txt"
printf 'pub fn spec() {}\n' > "$source_tree/crates/a/src/target-spec/mod.rs"
printf 'cached\n' > "$source_tree/tools/__pycache__/m.pyc"
printf 'a regular file named target\n' > "$source_tree/crates/a/target"
printf 'doc\n' > "$source_tree/docs/target.md"
ln -s fixtures "$source_tree/crates/a/tests/link-target"
ln -s fixtures "$source_tree/crates/a/tests/target-link"

listing() { (cd "$1" && find . -path ./.git -prune -o \( -type f -o -type l \) -print | sort); }

take_snapshot "$source_tree" "$MODEL_WORK/one-run" > "$MODEL_WORK/one-run.out" 2>&1
one_rc=$?
slot_repo="$MODEL_WORK/slot/repo"
mkdir -p -- "$slot_repo/target/debug" "$slot_repo/crates/a/tests/fixtures/target" "$slot_repo/.git/target" "$slot_repo/tools/__pycache__"
printf 'stale\n' > "$slot_repo/target/debug/junk"
printf 'stale fixture\n' > "$slot_repo/crates/a/tests/fixtures/target/old.txt"
printf 'kept\n' > "$slot_repo/.git/target/keep"
printf 'stale\n' > "$slot_repo/tools/__pycache__/old.pyc"
refresh_slot_snapshot "$source_tree" "$slot_repo" "$MODEL_WORK/changed" > "$MODEL_WORK/slot.out" 2>&1
slot_rc=$?

listing "$source_tree" > "$MODEL_WORK/source.list"
listing "$MODEL_WORK/one-run" > "$MODEL_WORK/one-run.list"
listing "$slot_repo" > "$MODEL_WORK/slot.list"
for kind in one-run slot; do
  echo "name=copy_exclusions kind=$kind rc=$([[ $kind == one-run ]] && echo "$one_rc" || echo "$slot_rc") files=$(grep -c . "$MODEL_WORK/$kind.list")" \
    "missing=$(comm -23 "$MODEL_WORK/source.list" "$MODEL_WORK/$kind.list" | paste -sd, -)" \
    "extra=$(comm -13 "$MODEL_WORK/source.list" "$MODEL_WORK/$kind.list" | paste -sd, -)"
done
echo "name=copy_exclusions slot_leftover_git_target=$([[ -f "$slot_repo/.git/target/keep" ]] && echo 在 || echo 删了)"
if cmp -s "$MODEL_WORK/one-run.list" "$MODEL_WORK/slot.list"; then same=yes; else same=no; fi
echo "name=copy_exclusions_compare same=$same"
sed 's/^/    /' "$MODEL_WORK/slot.out"
tracked="$(git -C "$MODEL_REPO" ls-files | grep -cE '(^|/)(target|target-[^/]*|__pycache__)/')"
echo "name=copy_exclusions_tracked count=$tracked"
