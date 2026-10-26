#!/usr/bin/env bash
# 一条变异（或用 + 连起来的几条）：把改后那棵拷贝里的被改文件还原、打上变异、编 publish_lookup_by_identity 并跑全部六条（连慢档），
# 打一行「name=mutant mutant=<名> rc=<退出码> <test result 行> failed=[…] growth=<两个读数>」。
# 用法：run_mutant.sh <仓根> <改后原样那棵树> <变异用的树> <编译目录> <mutants.tsv> <变异名|baseline> <输出目录>
set -uo pipefail
repo="$1" pristine="$2" tree="$3" target="$4" table="$5" mutant="$6" out="$7"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cp -p "$pristine/crates/singlefs-core/src/transaction.rs" "$tree/crates/singlefs-core/src/transaction.rs"
cp -p "$pristine/crates/singlefs-core/src/mounted_session.rs" "$tree/crates/singlefs-core/src/mounted_session.rs"
cp -p "$pristine/crates/singlefs-harness/tests/publish_lookup_by_identity.rs" "$tree/crates/singlefs-harness/tests/publish_lookup_by_identity.rs"
touch "$tree/crates/singlefs-core/src/transaction.rs" "$tree/crates/singlefs-core/src/mounted_session.rs" "$tree/crates/singlefs-harness/tests/publish_lookup_by_identity.rs"
if [[ "$mutant" != baseline ]]; then
  python3 -I "$here/apply_mutant.py" "$table" "$tree" "$mutant" || { echo "name=mutant mutant=$mutant rc=apply-failed"; exit 3; }
fi
log="$out/mutant-${mutant//+/-}"
cd "$tree" || exit 2
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity（攻方腿变异拷贝）; 范围=1 个测试目标; 为什么=看攻方腿这一条变异让不让点名的六条用例连慢档变红，判测试的判别力' CARGO_TARGET_DIR="$target" nice -n 19 bash "$repo/scripts/hosts/two-logs.sh" --quiet "$log" -- bash "$repo/scripts/hosts/capped.sh" 31 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 16G cargo test -p singlefs-harness --test publish_lookup_by_identity -- --include-ignored --nocapture
rc=$?
result="$(grep -E '^test result' "$log.log" | tail -n 1 | sed -E 's/; finished in [0-9.]+s//')"
failed="$(grep -E '^test .* FAILED$' "$log.log" | sed -E 's/^test ([^ ]+) .*/\1/' | sed -E 's/^(.{40}).*/\1/' | sort | paste -sd, -)"
growth="$(grep -oE 'growth_in_percent=[0-9]+( index_builds_growth_in_percent=[0-9]+)?' "$log.log" | paste -sd' ' -)"
echo "name=mutant mutant=$mutant rc=$rc $result failed=[$failed] $growth"
grep -E '^probe_' "$log.log" | sort | uniq -c | sort -rn | head -n 12 || true
