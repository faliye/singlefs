#!/usr/bin/env bash
# research/scripts/mutate.sh 在一棵小树上用假 cargo 与假内存包装跑：包装退 252（排不上）、254（被总上限挤掉）时这一条变异记成什么、整轮退几。
# 用法：mutate-probe.sh <仓拷贝根>；在 <仓拷贝根> 旁边建 mut-tree（只拷 .claude/scripts、.claude/singlefs-ai-sop 与 research/scripts），
# 把 mut-tree 里的 run-with-memory-cap.sh 换成假的（带 RUN_WITH_MEMORY_CAP_TIME_LIMIT 的那几次照 FAKE_WRAPPER_MODE 演，基线照常跑）。复跑挂钟：十几秒。
set -uo pipefail
root="$1"
tree="$(dirname "$root")/mut-tree"
rm -rf -- "${tree:?}"
mkdir -p "$tree/.claude" "$tree/research"
cp -a "$root/.claude/scripts" "$tree/.claude/scripts"
cp -a "$root/.claude/singlefs-ai-sop" "$tree/.claude/singlefs-ai-sop"
cp -a "$root/research/scripts" "$tree/research/scripts"
cat > "$tree/research/scripts/run-with-memory-cap.sh.new" <<'WRAP'
#!/usr/bin/env bash
# 假内存包装：$1 上限，其余是命令。--check 照真包装退 0；没带限时（基线、还原之后）照跑；带了限时照 FAKE_WRAPPER_MODE 演
[[ "${1:-}" == --check ]] && exit 0
shift
if [[ -z "${RUN_WITH_MEMORY_CAP_TIME_LIMIT:-}" ]]; then exec "$@"; fi
case "${FAKE_WRAPPER_MODE:-pass}" in
  pass) exec "$@" ;;
  queue-full) echo "run-with-memory-cap: 等满 3600 秒还放不下（假包装），退 252" >&2; exit 252 ;;
  memory-hit) echo "run-with-memory-cap: 撞了内存上限（假包装），退 250" >&2; exit 250 ;;
  slow-queue-full) sleep 4; echo "run-with-memory-cap: 排了很久还放不下（假包装），退 252" >&2; exit 252 ;;
  squeezed) printf 'running 2 tests\nerror: test failed, to rerun pass `--bin fake-bench`\n\nCaused by:\n  process didn'"'"'t exit successfully: `target/release/deps/fake_bench-0123456789abcdef` (signal: 9, SIGKILL: kill)\n'
            echo "run-with-memory-cap: 被总上限挤掉（假包装），退 254" >&2; exit 254 ;;
esac
WRAP
chmod +x "$tree/research/scripts/run-with-memory-cap.sh.new"
mv "$tree/research/scripts/run-with-memory-cap.sh.new" "$tree/research/scripts/run-with-memory-cap.sh"
work="$tree/work"; mkdir -p "$work/bin" "$work/research/bench/src/bin" "$work/research/mutations"
cat > "$work/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
# 假 cargo（照 mutate.sh 自证的那一份）：源码里 BEHAVIOR: PASS 就两条用例都过
behavior="$(sed -n 's/.*BEHAVIOR: \([A-Z]*\).*/\1/p' bench/src/bin/fake_bench.rs | head -1)"
case "$behavior" in
  PASS) printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts ... ok\n\ntest result: ok. 2 passed; 0 failed\n'; exit 0 ;;
  RED) printf 'running 2 tests\ntest tests::adds ... FAILED\ntest tests::subtracts ... ok\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
esac
CARGO
chmod +x "$work/bin/cargo"
printf '[workspace]\nmembers = ["bench"]\n' > "$work/research/Cargo.toml"
printf '[package]\nname = "bench"\n\n[[bin]]\nname = "fake-bench"\npath = "src/bin/fake_bench.rs"\n' > "$work/research/bench/Cargo.toml"
printf '// 注释行\nfn main() {} // BEHAVIOR: PASS\n' > "$work/research/bench/src/bin/fake_bench.rs"
# 一条谁都抓不到的变异（只改注释）：照理是「没红」、整轮退 1
printf '没人抓得到\t// 注释行\t// 改过的注释行\n' > "$work/research/mutations/survivor.tsv"
run() { # <标签> <环境…>
  local label="$1"; shift
  local output exit_code
  output="$(cd "$work/research" && env PATH="$work/bin:$PATH" MUTATE_TARGET_DIR="$work/target-$label" MUTATE_JOBS=1 MUTATE_MEMORY_MAX=64M "$@" \
    bash "$tree/research/scripts/mutate.sh" fake-bench bench/src/bin/fake_bench.rs mutations/survivor.tsv 2>&1)"; exit_code=$?
  echo "### $label：退 $exit_code"
  grep -E '没人抓得到|计数|无效|超时|没跑完|撞' <<<"$output" | sed 's/^/    /'
}
run control-pass FAKE_WRAPPER_MODE=pass
run queue-full-252 FAKE_WRAPPER_MODE=queue-full
run squeezed-254 FAKE_WRAPPER_MODE=squeezed
run slow-queue-new FAKE_WRAPPER_MODE=slow-queue-full MUTATE_TIMEOUT=2
# 弄坏开关在平常跑时认不认：撞顶本该判失败（退 1），带着 MUTATE_BREAK=memoryascaught 平常跑一次
run memory-hit-control FAKE_WRAPPER_MODE=memory-hit
run memory-hit-with-leftover-break FAKE_WRAPPER_MODE=memory-hit MUTATE_BREAK=memoryascaught
