#!/usr/bin/env bash
# run_mutant.sh <仓根> <副本根> <编译目录> <模型目录> <变异名|baseline> <日志目录> [测试名过滤…]
# 在副本里打上攻方变异表里那一行（baseline 不打），经线程上限与内存上限编 singlefs-harness 的 publish_lookup_by_identity，
# 直接执行编出来的测试二进制（不带过滤就跑全部不忽略的用例），打一行结果；再跑攻方的见证用例（副本里要已拷进 tests/attack_followup_witness.rs），
# 打它的计数行；最后把那份文件换回打之前的样子。
set -u
repo=$1; copy=$2; target=$3; model=$4; mutant=$5; logs=$6; shift 6
file=crates/singlefs-core/src/transaction.rs
mkdir -p "$logs"
cp "$copy/$file" "$logs/pristine-transaction.rs.$mutant"
if [[ "$mutant" != baseline ]]; then
  for one in ${mutant//+/ }; do   # 名字用 + 连起来就一起打（MA1+MA2）
    python3 "$model/tools/apply_mutation.py" "$model/mutations/attack-mutations.tsv" "$one" "$copy" \
      || { cp "$logs/pristine-transaction.rs.$mutant" "$copy/$file"; exit 3; }
  done
fi
build_log="$logs/build-$mutant"
( cd "$copy" && CARGO_TARGET_DIR="$target" nice -n 19 bash "$repo/scripts/hosts/two-logs.sh" --quiet "$build_log" -- \
    bash "$repo/scripts/hosts/capped.sh" 31 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G \
    cargo test --offline -p singlefs-harness --test publish_lookup_by_identity --test attack_followup_witness --no-run )
build_rc=$?
binary=$(grep -o "$target/debug/deps/publish_lookup_by_identity-[0-9a-f]*" "$build_log.log" | tail -1)
rc=build-failed
if [[ $build_rc -eq 0 && -n "$binary" ]]; then
  ( cd "$copy/crates/singlefs-harness" && nice -n 19 bash "$repo/scripts/hosts/capped.sh" 31 \
      bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G "$binary" --test-threads 31 --nocapture "$@" ) > "$logs/test-$mutant.txt" 2>&1
  rc=$?
  witness=$(grep -o "$target/debug/deps/attack_followup_witness-[0-9a-f]*" "$build_log.log" | tail -1)
  ( cd "$copy/crates/singlefs-harness" && nice -n 19 bash "$repo/scripts/hosts/capped.sh" 31 \
      bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G "$witness" --exact attack_followup_witness_of_the_counted_sites --nocapture ) > "$logs/witness-$mutant.txt" 2>&1
fi
cp "$logs/pristine-transaction.rs.$mutant" "$copy/$file"
result=$(grep -h '^test result' "$logs/test-$mutant.txt" 2>/dev/null | tail -1 | sed 's/; finished in [0-9.]*s//')
failed=$(grep -h '^test .* FAILED$' "$logs/test-$mutant.txt" 2>/dev/null | sed 's/^test //; s/ \.\.\. FAILED$//' | sort | tr '\n' ' ')
echo "name=mutant mutant=$mutant build_rc=$build_rc rc=$rc $result failed=[$failed]"
grep -h "^witness" "$logs/witness-$mutant.txt" 2>/dev/null | sed "s/^/name=witness mutant=$mutant /"
