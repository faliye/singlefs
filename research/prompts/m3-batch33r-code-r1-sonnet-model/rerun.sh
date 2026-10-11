#!/usr/bin/env bash
# 用法：SINGLEFS_REPO=<仓根> [SINGLEFS_FROZEN_TREE=<冻结副本>] [SINGLEFS_THREADS=6] [SINGLEFS_STAGES="1 2 3 4 5 6 7 8"] bash rerun.sh <输出目录>
# 变量：SINGLEFS_REPO（仓根，取 scripts 里的包装与快照里的 sha256）；SINGLEFS_FROZEN_TREE（冻结副本，默认
#   $HOME/.local/share/singlefs/work/m3-batch33r-code-r1/tree）；SINGLEFS_THREADS（线程上限，默认 6）；SINGLEFS_STAGES（要跑哪几步，默认全部；
#   前台命令有时限时分几次跑同一个输出目录，第一次含 1 步，最后一次含 8 步）。含第 1 步时输出目录必须不存在或为空。
# 做什么（顺序、同一时刻只一条 cargo；编译 6G、跑 4G，经 run-with-memory-cap.sh；编译目录在输出目录里）：
#   1 四份 sha256 对快照  2 变异行锚点（anchors.py）  3 oracle 小模型（oracle_model.py）
#   4 把冻结副本拷进输出目录、放进两份 harness 档探针、编、跑  5 judge_states.py 判探针二  6 自证：改坏探针二输出必须判红
#   7 变异行替换后的 cargo check（mutation_compile_check.py），自证：写坏的替换文必须编不过  8 汇总
# 只跑 harness 档与 cargo check，不跑 singlefs-checker-tier 的测试。结果汇总在 <输出目录>/summary.txt（排序、无耗时字段）。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO（仓根）}"
tree="${SINGLEFS_FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-batch33r-code-r1/tree}"
threads="${SINGLEFS_THREADS:-6}"
out="${1:?要给输出目录}"
stages=" ${SINGLEFS_STAGES:-1 2 3 4 5 6 7 8} "
has() { [[ "$stages" == *" $1 "* ]]; }
if has 1 && [[ -e "$out" && -n "$(ls -A "$out" 2>/dev/null)" ]]; then echo "输出目录不空：$out" >&2; exit 2; fi
export PYTHONDONTWRITEBYTECODE=1
mkdir -p "$out"; out="$(cd "$out" && pwd)"
cap="$SINGLEFS_REPO/scripts/run-with-memory-cap.sh"
capped="$SINGLEFS_REPO/scripts/capped.sh"
export CARGO_TARGET_DIR="$out/target"
copy="$out/copy"

if has 1; then
echo "== 1 sha256" | tee -a "$out/log.txt"
( cd "$tree" && sha256sum crates/singlefs-checker-tier/src/crash.rs crates/singlefs-harness/src/memory_pool.rs \
    crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs crates/mutations.tsv ) > "$out/sha-now.txt"
if diff "$out/sha-now.txt" "$SINGLEFS_REPO/research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt" > /dev/null; then echo "SHA same_4_of_4" > "$out/part-sha.txt"; else echo "SHA DIFFERENT" > "$out/part-sha.txt"; fi
fi
if has 2; then
echo "== 2 anchors" | tee -a "$out/log.txt"
SINGLEFS_FROZEN_TREE="$tree" python3 "$here/anchors.py" > "$out/part-anchors.txt"
fi
if has 3; then
echo "== 3 model" | tee -a "$out/log.txt"
python3 "$here/oracle_model.py" > "$out/part-model.txt"
fi
if has 4; then
echo "== 4 probes" | tee -a "$out/log.txt"
mkdir -p "$copy"; cp -r "$tree/." "$copy/"
cp "$here/m33_stream_probe.rs" "$here/m33_state_probe.rs" "$copy/crates/singlefs-harness/tests/"
( cd "$copy" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo test --offline -p singlefs-harness --test m33_stream_probe --no-run > "$out/build1.log" 2>&1 )
( cd "$copy" && nice -n 19 bash "$cap" 6G bash "$capped" "$threads" cargo test --offline -p singlefs-harness --test m33_state_probe --no-run > "$out/build2.log" 2>&1 )
( cd "$copy" && nice -n 19 bash "$cap" 4G bash "$capped" "$threads" cargo test --offline -p singlefs-harness --test m33_stream_probe -- --nocapture > "$out/run1.log" 2>&1 )
( cd "$copy" && SINGLEFS_THREAD_CAP="$threads" nice -n 19 bash "$cap" 4G bash "$capped" "$threads" cargo test --offline -p singlefs-harness --test m33_state_probe -- --nocapture > "$out/run2.log" 2>&1 )
grep -E '^PROBE ' "$out/run1.log" > "$out/part-probe1.txt"
grep -E '^PROBE2 ' "$out/run2.log" > "$out/part-probe2.txt"
grep -c '^test result: ok. 1 passed' "$out/run1.log" "$out/run2.log" | sed 's/^/TESTRESULT_OK_COUNT /' | sed -E 's#^TESTRESULT_OK_COUNT .*/#TESTRESULT_OK_COUNT #' > "$out/part-testresult.txt"
fi
if has 5; then
echo "== 5 judge" | tee -a "$out/log.txt"
python3 "$here/judge_states.py" "$out/part-probe2.txt" > "$out/part-judge.txt"
fi
if has 6; then
echo "== 6 self-red of the judge" | tee -a "$out/log.txt"
python3 - "$out/part-probe2.txt" "$out/doctored.txt" <<'PY'
import sys
lines = open(sys.argv[1], encoding="utf-8").read().split("\n")
out = []
for line in lines:
    if line.startswith("PROBE2 state=prefix28 "):
        line = line.replace("outcome=NoFile", "outcome=C")               # 第 7 代读出内容：oracle 必须判违例
    if line.startswith("PROBE2 state=prefix24 "):
        line = line.replace("steps=3", "steps=9")                        # 续做步数改坏：必须判红
    out.append(line)
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(out))
PY
python3 "$here/judge_states.py" "$out/doctored.txt" > "$out/doctored-judge.txt" || true
if grep -q '^JUDGE states=62 bad=2$' "$out/doctored-judge.txt"; then echo "SELFRED judge_reports_the_two_doctored_states=yes" > "$out/part-selfred-judge.txt"; else echo "SELFRED judge_reports_the_two_doctored_states=NO" > "$out/part-selfred-judge.txt"; fi
fi
if has 7; then
echo "== 7 mutation compile" | tee -a "$out/log.txt"
SINGLEFS_FROZEN_TREE="$tree" SINGLEFS_COPY="$copy" SINGLEFS_REPO="$SINGLEFS_REPO" SINGLEFS_THREADS="$threads" CARGO_TARGET_DIR="$out/target-check" \
  nice -n 19 bash "$cap" 6G python3 "$here/mutation_compile_check.py" 16 3102 3103 3104 3105 3106 3107 3108 3109 > "$out/part-mutcheck.txt"
SINGLEFS_MUTCHECK_SELFTEST=1 SINGLEFS_FROZEN_TREE="$tree" SINGLEFS_COPY="$copy" SINGLEFS_REPO="$SINGLEFS_REPO" SINGLEFS_THREADS="$threads" CARGO_TARGET_DIR="$out/target-check" \
  nice -n 19 bash "$cap" 6G python3 "$here/mutation_compile_check.py" 3106 > "$out/selfred-mutcheck.txt"
if grep -q 'cargo_check_exit=101' "$out/selfred-mutcheck.txt"; then echo "SELFRED broken_replacement_does_not_compile=yes" > "$out/part-selfred-mut.txt"; else echo "SELFRED broken_replacement_does_not_compile=NO" > "$out/part-selfred-mut.txt"; fi
for f in crates/singlefs-checker-tier/src/crash.rs crates/singlefs-core/src/mount.rs crates/singlefs-core/src/transaction.rs; do cmp "$copy/$f" "$tree/$f"; done
fi
if has 8; then
cat "$out"/part-*.txt | grep -E '^(SHA|ANCHOR|MODEL|PROBE |PROBE2|JUDGE|MUTCHECK|SELFRED|TESTRESULT)' | LC_ALL=C sort > "$out/summary.txt"
echo "summary: $(wc -l < "$out/summary.txt") 行 -> $out/summary.txt" | tee -a "$out/log.txt"
fi
