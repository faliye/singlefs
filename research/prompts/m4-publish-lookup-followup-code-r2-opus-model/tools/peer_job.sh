#!/usr/bin/env bash
# peer_job.sh <作业名> <变异名（几个用 + 连起来一起打；baseline 不打）> <singlefs-harness 的测试目标> [测试二进制参数…]
# 在树根跑（multi-host-run.sh --peer 把整棵副本拷过去、经内存包装起它）。模型目录在树里的 attack-model/：打变异、把模型的测试文件拷进
# crates/singlefs-harness/tests/、编那个测试目标（debug 档）、直接执行编出来的测试二进制，输出写 out/<作业名>.txt，最后把改过的两份源文件换回去。
# 编译目录由 multi-host-run.sh 的共用编译目录守卫设（CARGO_TARGET_DIR）；线程照 capped.sh 31。打一行 name=job 的汇总。
set -u
job=$1; mutants=$2; target=$3; shift 3
model=attack-model
mkdir -p out
backup=$(mktemp -d)
cp crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/recovery.rs "$backup/"
if [[ "$mutants" != baseline ]]; then
  for one in ${mutants//+/ }; do
    python3 "$model/tools/apply_mutation.py" "$model/mutations/attack-mutations.tsv" "$one" . || { echo "name=job job=$job mutant=$mutants apply_failed=$one"; exit 3; }
  done
fi
cp "$model"/tests/attack_r2_*.rs crates/singlefs-harness/tests/
# 共用编译目录按 mtime 判新鲜：上一趟打过变异编出来的产物比拷过来的源文件新，不 touch 就会拿上一趟的变异体来跑。两份可能被改的源文件每趟都 touch。
touch crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/recovery.rs
SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 $target（只编）; 范围=1 个测试目标; 为什么=攻方腿在副本上编变异或探针" \
  bash scripts/hosts/capped.sh 31 cargo test --offline -p singlefs-harness --test "$target" --no-run > "out/$job.build.txt" 2>&1
build_rc=$?
binary=$(grep -o "([^)]*/deps/${target}-[0-9a-f]*)" "out/$job.build.txt" | tail -1 | tr -d '()'); [[ -n "$binary" ]] && binary=$(realpath "$binary")
rc=build-failed
if [[ $build_rc -eq 0 && -n "$binary" ]]; then
  started=$(date +%s)
  ( cd crates/singlefs-harness && bash ../../scripts/hosts/capped.sh 31 "$binary" "$@" ) > "out/$job.txt" 2>&1
  rc=$?
  echo "name=job_seconds job=$job seconds=$(( $(date +%s) - started ))"
fi
cp "$backup/transaction.rs" "$backup/recovery.rs" crates/singlefs-core/src/
rm -rf "${backup:?}"
result=$(grep -h '^test result' "out/$job.txt" 2>/dev/null | tail -1 | sed 's/; finished in [0-9.]*s//')
failed=$(grep -h '^test .* FAILED$' "out/$job.txt" 2>/dev/null | sed 's/^test //; s/ \.\.\. FAILED$//' | sort | tr '\n' ' ')
echo "name=job job=$job mutant=$mutants target=$target build_rc=$build_rc rc=$rc $result failed=[$failed]"
grep -h '^error' "out/$job.build.txt" | head -5 | sed "s/^/name=build_error job=$job /"
