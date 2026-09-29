#!/usr/bin/env bash
# admission: always 每一批补丁都要打一次、验一次，上一次的结论不替这一次作保
# run-condition: command git cargo
# 一批实现员交回之后的「合入后验证」：打全部补丁、编一次、跑 harness 档全部测试二进制、fmt 与 clippy，红的列出来。替掉派一个实现员做这件活。
#
#   merge-and-verify.sh [--memory <上限>] [--threads <N>] [--skip-apply] <补丁目录> …
#   merge-and-verify.sh --selftest                     # 用假的 apply-writer-patch 与假 cargo 走一遍；MERGE_AND_VERIFY_BREAK=ignore-red 时必须判红
#
# 为什么：2026-09-20 起 8 天里「合入后验证」派了 3 个实现员、平均 14.3M 折算，是实现员里最贵的一种；它做的前三步（打补丁、编、逐个跑测试）
# 是 apply-writer-patch.py 加几条命令，只有「红了改钉值」要判断——那一步红了才派实现员。
# 做法：每个补丁目录经 research/scripts/apply-writer-patch.py 打（退 2 就整批停）；然后经 run-with-memory-cap.sh 与 capped.sh 跑
# `cargo test --no-fail-fast` 只挑 harness 档的四个包（不碰 checker 档包 singlefs-checker-tier，那归提交时的崩溃验证员）、`cargo fmt --check`、
# `cargo clippy -D warnings`；日志落 $MERGE_AND_VERIFY_LOG_DIRECTORY（默认 /tmp/claude-1000/merge-and-verify/<时间>）。
# 退出码：0 全绿；1 有红（测试、fmt、clippy 或 apply 退 1）；2 补丁核不过（apply 退 2）、用法错。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
APPLY="${MERGE_AND_VERIFY_APPLY:-$here/apply-writer-patch.py}"
WRAPPER="${MERGE_AND_VERIFY_WRAPPER:-$here/run-with-memory-cap.sh}"
CAPPED="${MERGE_AND_VERIFY_CAPPED:-$here/capped.sh}"
HARNESS_PACKAGES=(-p singlefs-format -p singlefs-core -p singlefs-harness -p singlefs-checker)   # harness 档：checker 档包 singlefs-checker-tier 不在这里

die() { echo "  ✗ $1"; echo "  → 怎么办：$2"; exit "${3:-2}"; }

run_capped() {   # $1 日志文件，其余是命令；经内存包装与线程上限（WRAPPER 为 none 时直接跑，自检用）
  local log="$1"; shift
  if [[ "$WRAPPER" == "none" ]]; then "$@" > "$log" 2>&1
  else bash "$WRAPPER" "$MEMORY" bash "$CAPPED" "$THREADS" "$@" > "$log" 2>&1; fi
}

verify() {
  local logs="$1"; shift
  local -a dirs=("$@")
  local dir rc applied=0 warned=0 failed_tests reds=0
  mkdir -p "$logs"
  for dir in "${dirs[@]}"; do
    [[ -d "$dir" ]] || die "补丁目录不在：$dir" "给实现员交回的补丁目录（里面有 crates.patch 或 mutations-append.tsv）"
    python3 "$APPLY" "$dir" > "$logs/apply-$(basename "$dir").log" 2>&1; rc=$?
    case "$rc" in
      0) applied=$((applied+1)); echo "  打上：$dir" ;;
      1) applied=$((applied+1)); warned=$((warned+1)); echo "  打上、但 code-source-discipline 红：$dir（日志 $logs/apply-$(basename "$dir").log）" ;;
      *) die "补丁核不过、一个字没改：$dir（apply-writer-patch.py 退 $rc，日志 $logs/apply-$(basename "$dir").log）" "按那份日志给的下一步让实现员改补丁再交" 2 ;;
    esac
  done
  echo "  编译并跑 harness 档四个包的全部测试二进制（内存上限 $MEMORY、线程上限 $THREADS）…"
  run_capped "$logs/test.log" cargo test --offline --no-fail-fast "${HARNESS_PACKAGES[@]}"; rc=$?
  failed_tests="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$logs/test.log")"
  if [[ -n "$failed_tests" && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ 红的测试 $(printf '%s\n' "$failed_tests" | wc -l) 条："
    printf '%s\n' "$failed_tests" | sed 's/^/      /'   # gate-lint:detail
    echo "  → 怎么办：只重跑红的这几条：python3 research/scripts/rerun-failed-tests.py $logs/test.log -p <包>；照实现员报告里的算法改钉值，改完再只跑这几条"
  elif [[ $rc -ne 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 $rc 而一条 FAILED 都没抓到（编不过、或包装退 250–254）：$(tail -3 "$logs/test.log" | tr '\n' ' ')"
    echo "  → 怎么办：看 $logs/test.log；退 250–254 是内存包装自己的结局，按 run-with-memory-cap.sh 文件头处理"
  else
    echo "  ✓ 测试全绿（$(grep -c '^test result: ok' "$logs/test.log") 个测试二进制）"
  fi
  run_capped "$logs/fmt.log" cargo fmt --all -- --check; rc=$?
  if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo fmt --check 红（日志 $logs/fmt.log）"; echo "  → 怎么办：cargo fmt --all，再看 diff 只动了格式"; else echo "  ✓ fmt 过"; fi
  run_capped "$logs/clippy.log" cargo clippy --offline --all-targets --all-features -- -D warnings; rc=$?
  if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ clippy 红（日志 $logs/clippy.log）"; echo "  → 怎么办：按日志逐条改，不加 allow 绕过"; else echo "  ✓ clippy 过"; fi
  echo "  汇总：打上 ${applied} 个补丁目录（其中 code-source-discipline 红 $warned 个），验证红 $reds 项，日志在 $logs"
  [[ $reds -eq 0 && $warned -eq 0 ]]
}

selftest() {
  local scratch fake_cargo_dir failures=0 checked=0 out rc
  scratch="$(mktemp -d)"; fake_cargo_dir="$scratch/bin"; mkdir -p "$fake_cargo_dir" "$scratch/patch-a" "$scratch/patch-b"
  cat > "$scratch/fake-apply.py" <<'FAKE'
import sys
sys.exit(2 if sys.argv[1].endswith("patch-b") else 0)
FAKE
  cat > "$fake_cargo_dir/cargo" <<'FAKE'
#!/usr/bin/env bash
case "$1" in
  test) printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts_in_%s ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed\n' "${FAKE_CARGO_MODE:-x}"; [[ "${FAKE_CARGO_MODE:-}" == "green" ]] && exit 0 || exit 101 ;;
  fmt|clippy) exit 0 ;;
esac
FAKE
  [[ "${FAKE_CARGO_MODE:-}" == "" ]] || true
  chmod +x "$fake_cargo_dir/cargo"
  run_case() { PATH="$fake_cargo_dir:$PATH" MERGE_AND_VERIFY_APPLY="$scratch/fake-apply.py" MERGE_AND_VERIFY_WRAPPER=none MERGE_AND_VERIFY_LOG_DIRECTORY="$scratch/logs-$1" bash "$0" "${@:2}" > "$out" 2>&1; rc=$?; }
  out="$scratch/out1.txt"; run_case one "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 1 ]] || ! grep -q 'subtracts_in_x' "$out" || ! grep -q 'rerun-failed-tests.py' "$out"; then
    echo "  ✗ 自检：一条测试红时应当退 1、列出那条测试与只重跑它的命令，实际退出码 $rc：$(tail -4 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  out="$scratch/out2.txt"; run_case two "$scratch/patch-a" "$scratch/patch-b"; checked=$((checked+1))
  if [[ $rc -ne 2 ]] || ! grep -q '补丁核不过' "$out"; then
    echo "  ✗ 自检：apply 退 2 时应当整批停、退 2，实际 $rc：$(tail -2 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  # 假 cargo 全绿的一趟：把 FAILED 行换掉
  sed -i.bak 's/ \.\.\. FAILED/ ... ok/; s/FAILED\. 1 passed; 1 failed/ok. 2 passed; 0 failed/' "$fake_cargo_dir/cargo" && rm -f "$fake_cargo_dir/cargo.bak"
  out="$scratch/out3.txt"; FAKE_CARGO_MODE=green run_case three "$scratch/patch-a"; checked=$((checked+1))
  if [[ $rc -ne 0 ]] || ! grep -q '测试全绿' "$out"; then
    echo "  ✗ 自检：全绿时应当退 0 并说全绿，实际 $rc：$(tail -3 "$out" | tr '\n' ' ')"; failures=1; fi   # gate-lint:detail
  rm -rf "${scratch:?}"
  if [[ $failures -ne 0 ]]; then
    echo "    → 怎么办：看 verify() 抓 FAILED 行与退出码的那几处；MERGE_AND_VERIFY_BREAK=ignore-red 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 种）：测试红时退 1 并列出红的与重跑命令、补丁核不过整批停退 2、全绿退 0"
  return 0
}

if [[ "${1:-}" == "--selftest" ]]; then selftest; exit $?; fi
MEMORY="${REPLAY_MEMORY_CAP:-8G}"; THREADS="$(nproc)"; skip_apply=0; dirs=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --memory) MEMORY="${2:?}"; shift 2 ;;
    --threads) THREADS="${2:?}"; shift 2 ;;
    --skip-apply) skip_apply=1; shift ;;
    --*) die "认不出参数 $1" "只认 --memory <上限>、--threads <N>、--skip-apply" ;;
    *) dirs+=("$1"); shift ;;
  esac
done
[[ ${#dirs[@]} -gt 0 || $skip_apply -eq 1 ]] || die "没有给补丁目录" "merge-and-verify.sh [--memory 16G] [--threads N] <补丁目录> …；只验不打用 --skip-apply"
logs="${MERGE_AND_VERIFY_LOG_DIRECTORY:-/tmp/claude-$(id -u)/merge-and-verify/$(TZ=Asia/Tokyo date +%F)-$$}"
cd "$root" || die "进不去仓根 $root" "在仓里跑"
if [[ $skip_apply -eq 1 ]]; then dirs=(); fi
verify "$logs" "${dirs[@]}"
