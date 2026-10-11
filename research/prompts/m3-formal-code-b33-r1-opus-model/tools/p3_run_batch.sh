#!/usr/bin/env bash
# 第 33 批攻方 P3：在当前目录这棵树（仓副本根）里，每个回收错法各拷一份 crates/ 与 Cargo.*、各用自己的编译目录，
# 并行施加、编、跑 812 两条用例与 root_ring_turn_in_one_mount 三条用例。
# 用法：p3_run_batch.sh <输出目录> <批号> <线程数> <名字>…（名字见 p3_mutants.py；baseline 不施加）
# 每件退出码写 <输出目录>/b<批号>-<件号>.rc，日志写 <输出目录>/p3-<名字>.log；最后一个不带参数的 wait 等齐，数 .rc 个数。
set -u
out="$1"; batch="$2"; threads="$3"; shift 3
here="$(pwd)"
tools="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$tools/../../../.." && pwd)"
mkdir -p "$out"
rm -f "$out"/b"$batch"-*.rc
index=0
for name in "$@"; do
  index=$((index + 1))
  {
    work="$here/p3w/$name"
    rm -rf "$work"; mkdir -p "$work"
    cp -r "$here/crates" "$here/Cargo.toml" "$here/Cargo.lock" "$work/"
    (
      cd "$work" || exit 90
      if [ "$name" != baseline ]; then python3 "$tools/p3_mutants.py" . "$name" || exit 91; fi
      CARGO_TARGET_DIR="$work/target" nice -n 19 bash "$repo/scripts/hosts/capped.sh" "$threads" \
        cargo test --offline --no-fail-fast -p singlefs-harness \
        --test overwrite_in_one_instance --test root_ring_turn_in_one_mount \
        -- allocated_statistic_equals repeated_overwrites_go_past turning_the_root_ring \
        zero_unit_publish_the_caller an_overwrite_that_goes_out
    ) > "$out/p3-$name.log" 2>&1
    echo "$?" > "$out/b$batch-$index.rc"
  } &
done
wait
count=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
echo "P3BATCH batch=$batch dispatched=$index rc_files=$count"
[ "$count" -eq "$index" ] || { echo "P3BATCH rc 文件数对不上，整批作废"; exit 1; }
index=0
for name in "$@"; do
  index=$((index + 1))
  rc=$(cat "$out/b$batch-$index.rc")
  compiled=$(grep -c 'Compiling singlefs-core' "$out/p3-$name.log")
  results=$(grep -E '^test result' "$out/p3-$name.log" | tr '\n' ' ')
  failed=$(grep -E '^test .* FAILED$|^    [a-z_]+$' "$out/p3-$name.log" | tr '\n' ' ')
  echo "P3RESULT name=$name rc=$rc compiled_core=$compiled results=[$results] failed=[$failed]"
done
