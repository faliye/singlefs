#!/usr/bin/env bash
# 槽位攻方腿的探针（拷进槽与占盘那一段）：source 冻结的 merge-and-verify-snapshot-lib.sh，只调 refresh_slot_snapshot 与 trim_slot_target，量三件事：
#   non-rs-change：只改了一份 .wgsl（include_str! 读的那类）时，槽里那一份的修改时刻刷到此刻、没变的 .rs 留着旧时刻、slot-changed-sources 一行都没有；
#   excluded-left-in-slot：槽里 repo/target、repo/scripts/__pycache__（验证段第 2 步在留着的槽里起的命令会编出来的那类）拷进槽之后还在；
#   trim-scope：占盘上限设 0 时 trim_slot_target 叫去删的只有 SLOT_TARGET，repo/target 不在它量的范围里。
# 用法：bash refresh_probe.sh <冻结的 merge-and-verify-snapshot-lib.sh> <不在或空的草稿目录> <remove_stub.py>
# 每格打一行 name=<格名> … verdict=<…>；退 0 跑完，2 用法错，1 探针自己出错。不起后台进程。
set -uo pipefail
lib="${1:-}"; work="${2:-}"; stub="${3:-}"
[[ -f "$lib" && -n "$work" && -f "$stub" ]] || { echo "用法：bash refresh_probe.sh <snapshot-lib> <草稿目录> <remove_stub.py>" >&2; exit 2; }
if [[ -e "$work" && -n "$(ls -A "$work")" ]]; then echo "草稿目录 $work 已存在且非空" >&2; exit 2; fi
mkdir -p "$work" || exit 1
unset MERGE_AND_VERIFY_BREAK
# shellcheck source=/dev/null
source "$lib"
source_tree="$work/source"; slot_repo="$work/slot/repo"; logs="$work/logs"
mkdir -p "$source_tree/crates/demo/src" "$logs" || exit 1
printf '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n' > "$source_tree/crates/demo/Cargo.toml"
printf 'const SHADER: &str = include_str!("shader.wgsl");\nfn main() { println!("{}", SHADER.len()); }\n' > "$source_tree/crates/demo/src/main.rs"
printf '@compute @workgroup_size(1) fn entry() {}\n' > "$source_tree/crates/demo/src/shader.wgsl"
refresh_slot_snapshot "$source_tree" "$slot_repo" "$logs/into-empty-slot" > "$logs/into-empty-slot.out" 2>&1 || { echo "name=into-empty-slot verdict=probe-error"; cat "$logs/into-empty-slot.out"; exit 1; }
into_empty_lines="$(wc -l < "$logs/into-empty-slot")"
echo "name=into-empty-slot changed_sources_lines=$into_empty_lines verdict=$([[ $into_empty_lines -eq 1 ]] && echo copied || echo UNEXPECTED)"
# 上一趟留下的样子：槽里每份文件的修改时刻拨到纪元秒 978307200（比任何一趟的编译产物都旧），另放验证段第 2 步编出来的那类东西
find "$slot_repo" -type f -exec touch -c -d @978307200 {} + || exit 1
old_stamp="$(stat -c %Y "$slot_repo/crates/demo/src/main.rs")"
mkdir -p "$slot_repo/target/release" "$slot_repo/scripts/__pycache__" || exit 1
head -c 1048576 /dev/zero > "$slot_repo/target/release/tool"
printf 'pyc\n' > "$slot_repo/scripts/__pycache__/module.pyc"
printf '@compute @workgroup_size(2) fn entry() {}\n' > "$source_tree/crates/demo/src/shader.wgsl"
refresh_slot_snapshot "$source_tree" "$slot_repo" "$logs/wgsl-only" > "$logs/wgsl-only.out" 2>&1 || { echo "name=non-rs-change verdict=probe-error"; cat "$logs/wgsl-only.out"; exit 1; }
wgsl_stamp="$(stat -c %Y "$slot_repo/crates/demo/src/shader.wgsl")"
rs_stamp="$(stat -c %Y "$slot_repo/crates/demo/src/main.rs")"
changed_lines="$(wc -l < "$logs/wgsl-only")"
itemized_written="$(grep -c '^>f' "$logs/wgsl-only.itemized")"
echo "name=non-rs-change itemized_written=$itemized_written changed_sources_lines=$changed_lines wgsl_newer_than_old=$([[ $wgsl_stamp -gt $old_stamp ]] && echo yes || echo no) rs_kept_old=$([[ $rs_stamp -eq $old_stamp ]] && echo yes || echo no) verdict=$([[ $changed_lines -eq 0 && $wgsl_stamp -gt $old_stamp && $rs_stamp -eq $old_stamp ]] && echo mtime-fresh-but-rebuild-check-skips || echo UNEXPECTED)"
target_left=no; [[ -f "$slot_repo/target/release/tool" ]] && target_left=yes
pycache_left=no; [[ -f "$slot_repo/scripts/__pycache__/module.pyc" ]] && pycache_left=yes
echo "name=excluded-left-in-slot repo_target_left=$target_left pycache_left=$pycache_left verdict=$([[ $target_left == yes && $pycache_left == yes ]] && echo HIT-excluded-build-output-persists-in-slot || echo excluded-cleared)"
SLOT_TARGET="$work/host-local-slot-target"; SLOT_TARGET_LIMIT_GIB=0; RUN_BUILD_DIRECTORY="$stub"
mkdir -p "$SLOT_TARGET/debug" || exit 1
printf 'x\n' > "$SLOT_TARGET/debug/artifact"
trim_output="$(trim_slot_target 收尾 2>&1)"
printf '%s\n' "$trim_output" > "$logs/trim.out"
repo_target_bytes="$(du -s --block-size=1 -- "$slot_repo/target" | cut -f1)"
asked_slot_target=no; grep -qF "清了这一槽的编译目录 $SLOT_TARGET（" "$logs/trim.out" && asked_slot_target=yes
asked_repo_target=no; grep -qF "$slot_repo/target" "$logs/trim.out" && asked_repo_target=yes
echo "name=trim-scope limit_gib=0 asked_slot_target=$asked_slot_target asked_repo_target=$asked_repo_target repo_target_bytes_at_least_1MiB=$([[ $repo_target_bytes -ge 1048576 ]] && echo yes || echo no) verdict=$([[ $asked_slot_target == yes && $asked_repo_target == no ]] && echo HIT-limit-covers-host-target-only || echo UNEXPECTED)"
exit 0
