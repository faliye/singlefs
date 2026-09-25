#!/usr/bin/env bash
# 证明「这些测试会红」：逐条注入一个已知的破坏，跑测试，记下红了哪几个，然后还原。
#
#   mutate.sh <bin 名> <源文件> <变异表>
#
# 变异表每行三段，用制表符分隔：变异名 <TAB> 原文 <TAB> 替换文（原文/替换文里的 \n 表示换行）
# 替换必须命中，命中数为 0 直接报错退出 —— 静默失效的替换会让整份证明变成假的
# （singlefs-ai-sop/rules/command-safety.md「脚本改文件之后要回读确认」）。
#
#   mutate.sh --selftest    拿一个假的 cargo 在临时 workspace 里走一遍各种结局（抓到、没红、无效、超时、内存撞顶、基线撞顶、scope 起不来），
#                           再走一遍并行的几样（进程数不同 stdout 逐字相同、各用各的副本、丢一份结果整道判红、锚点腐化派活之前退 3）
#
# 并行（command-safety.md「一个脚本里的检测项，能并行就并行」，形态照 .claude/gate.d/59-crates-mutation-replay.sh）：
#   每个工作进程一份被测装置的副本（research/ 里编译要用的那部分）、一个自己的 CARGO_TARGET_DIR，不在同一份源码上并发改：
#   第 1 个用 MUTATE_TARGET_DIR（跨轮热着），第 i 个用它后面加 -w<i>（跨轮复用，第一次要冷编译）。
#   工作进程数取 min(核数的一半且至多 16、表里的条数)，至少 1；MUTATE_JOBS 只能压小、不能加大（设成 1 就是原来的串行跑法）。
#   实际开几个、各项给几个、卡在哪一项、每个 cargo 的编译并行度，打在 stderr 的头一行。
#   内存不在这里判：每条 cargo test 照旧经 run-with-memory-cap.sh 跑，几条同时跑要多少、放不放得下由它管，这里不按内存收工作进程数。
#   每个 cargo 的编译并行度：先用 MUTATE_CARGO_JOBS，其次调用方已经设了的 CARGO_BUILD_JOBS，都没有就取 核数 ÷ 工作进程数（至少 1），免得 N 个 cargo 各按核数开 rustc。
#   基线在每份副本上各跑一次（第 1 份那次就是原来的基线，其余几份顺带把各自的编译目录热起来，不占每条变异的限时），还原之后那一次也在每份副本上各跑一次。
#   锚点在派活之前逐条核（每条原文在源码里恰好命中一次），有一条不是就退 3，一条变异都不跑。
#   活是动态领的：每条变异一个认领目录（mkdir 是原子的），谁先跑完谁领表里下一条还没人领的。
#   每条变异的判定写进自己的结果文件，父进程按表里的次序逐个读、逐行打到 stdout；工作进程一个字都不往 stdout 写。
#   stdout 与进程数无关：MUTATE_JOBS=1 与 =4 逐字相同；头一行在 stderr，是两次跑唯一不同的那一行。
#   派出去（表里）多少条、被认领多少条、收回多少份判定，数一遍对不上就退 8，不许少一条还报绿（command-safety.md「并行不许把失败吃掉」）。
#
# 退出码：0 每条都判完、没有判失败的；1 有没红、超时、内存撞顶或抓名字的规则有盲区；2 用法、环境或基线不绿；3 锚点不是恰好命中一次；
#   4 还原之后没回到全绿；5 跑的时候工作区里的源码被改过；6 源文件与二进制对不上；7 带上限的 scope 起不来；8 收回的判定与派出去的对不上。
#
# 每次 cargo test（基线、每条变异、还原之后那一次）都放进内存上限里跑（research/scripts/run-with-memory-cap.sh：systemd 的临时 scope，
# MemoryMax=<上限>、MemorySwapMax=0），撞上限只杀这一次 cargo test 里的进程，不把整机拖进 OOM
# （2026-09-25 两次整机 OOM 连带杀掉本地模型服务与 Claude Code 会话，records/2026-09-16-subagent拆分提案.md 第四十节第 28 行）。
#   MUTATE_MEMORY_MAX  上限，systemd 写法，默认 16G。依据：本机 60 GiB 内存，本地模型服务（vllm 与 ray）连同会话常驻约 12 GiB
#                      （2026-09-25 `free -g` 的 used 列是 12），同时可能有 3 件重活 ⇒ 每件 (60 − 12) ÷ 3 = 16 GiB；
#                      一条变异按一件重活的份额给；同时跑几条时合计由 run-with-memory-cap.sh 管（见「并行」那一段）。换机器、同时跑的重活变多都要重算（这些数只在本机成立）。
#   撞了上限记「内存撞顶」，与「超时」同一类：这条破坏让被测代码无界分配，不是「没抓到」，也不算「抓到」；整轮判失败，收尾单列计数。
#   带上限的 scope 起不来（没有用户级 systemd、D-Bus 连不上）就退 7，一条都不跑，不退回无上限去跑。
# 弄坏开关 MUTATE_BREAK（只给 --selftest 证明它会红用，几个用逗号连）：memoryascaught 把撞顶记成抓到、memorynotfail 撞顶不让整轮判失败、
#   droprow 工作进程把表里第 2 条的判定丢掉不写、countblind 父进程不数派出去与收回来的条数（只读在的那几份）、
#   sharedcopy 所有工作进程挤在第 1 份副本与第 1 个编译目录上（并行之前要防的那种并发改同一份源码）；
#   RUN_WITH_MEMORY_CAP_BREAK=nocap 原样传给 run-with-memory-cap.sh，就是改前那种不设上限的跑法。
set -uo pipefail
SELF_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
MEMORY_CAP_RUNNER="$(dirname "$SELF_PATH")/run-with-memory-cap.sh"
MEMORY_CAP_HIT_EXIT=250          # run-with-memory-cap.sh：撞了上限
MEMORY_CAP_UNAVAILABLE_EXIT=251  # run-with-memory-cap.sh：带上限的 scope 起不来
MEMORY_MAX="${MUTATE_MEMORY_MAX:-16G}"
BROKEN_JUDGEMENT="${MUTATE_BREAK:-}"
MAXIMUM_WORKERS_BY_PROCESSOR=16  # 核数那一项的上限，与 59 号的 worker_count_for 相同
COLLECTION_POLL_SECONDS=0.5      # 父进程隔这么久看一次哪几条判完了，按表序接着往 stdout 打

is_broken() { [[ ",$BROKEN_JUDGEMENT," == *",$1,"* ]]; }

# ── 自证：假的 cargo 按被测源码里写的 BEHAVIOR 演一种结局，不编译任何东西 ──
run_selftest() {
  local scratch failures=0 checked=0 output status
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/mutate-selftest-XXXXXX")"
  mkdir -p "$scratch/bin" "$scratch/research/bench/src/bin" "$scratch/research/mutations"
  cat >"$scratch/bin/cargo" <<'FAKE_CARGO'
#!/usr/bin/env bash
# 假 cargo：只认 mutate.sh 调的 `cargo test --release --bin <名字>`，按被测源码里的 BEHAVIOR 演一种结局；
# 每次被调记一行「当前目录|CARGO_TARGET_DIR|BEHAVIOR|参数」（并行的自证拿前两段数各用了几份副本、几个编译目录）
behavior="$(sed -n 's/.*BEHAVIOR: \([A-Z]*\).*/\1/p' bench/src/bin/fake_bench.rs | head -1)"
echo "$PWD|${CARGO_TARGET_DIR:-}|$behavior|$*" >>"$FAKE_CARGO_LOG"
passing_output() { printf 'running 2 tests\ntest tests::adds ... ok\ntest tests::subtracts ... ok\n\ntest result: ok. 2 passed; 0 failed\n'; }
case "$behavior" in
  PASS) passing_output; exit 0 ;;
  RED) printf 'running 2 tests\ntest tests::adds ... FAILED\ntest tests::subtracts ... ok\n\ntest result: FAILED. 1 passed; 1 failed\n'; exit 101 ;;
  # 每次拿 16 MiB 写满、拿到 512 MiB 就停：上限 64M 时半路被杀；没有上限时分配完照常通过（改前被记成「没红」），吃不满机器
  HOG) python3 -c 'held = [b"\x01" * (16 * 1024 * 1024) for _ in range(32)]' || exit 101; passing_output; exit 0 ;;
  HANG) sleep 30; passing_output; exit 0 ;;
  BROKEN) printf 'error[E0425]: cannot find value `missing` in this scope\nerror: could not compile `bench`\n'; exit 101 ;;
  *) echo "假 cargo 认不出 BEHAVIOR「$behavior」" >&2; exit 2 ;;
esac
FAKE_CARGO
  chmod +x "$scratch/bin/cargo"
  printf '[workspace]\nmembers = ["bench"]\n' >"$scratch/research/Cargo.toml"
  printf '[package]\nname = "bench"\n\n[[bin]]\nname = "fake-bench"\npath = "src/bin/fake_bench.rs"\n' >"$scratch/research/bench/Cargo.toml"
  printf '// 注释行\nfn main() {} // BEHAVIOR: PASS\n' >"$scratch/research/bench/src/bin/fake_bench.rs"
  printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n没红\t// 注释行\t// 改过的注释行\n撞内存\tBEHAVIOR: PASS\tBEHAVIOR: HOG\n超时\tBEHAVIOR: PASS\tBEHAVIOR: HANG\n无效\tBEHAVIOR: PASS\tBEHAVIOR: BROKEN\n' \
    >"$scratch/research/mutations/every-outcome.tsv"
  printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n撞内存\tBEHAVIOR: PASS\tBEHAVIOR: HOG\n' >"$scratch/research/mutations/caught-and-memory.tsv"
  printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' >"$scratch/research/mutations/caught-only.tsv"
  printf '抓到一\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到二\tBEHAVIOR: PASS\tBEHAVIOR: RED\n抓到三\tBEHAVIOR: PASS\tBEHAVIOR: RED\n' \
    >"$scratch/research/mutations/three-caught.tsv"
  printf '抓到\tBEHAVIOR: PASS\tBEHAVIOR: RED\n锚点腐化\tBEHAVIOR: GONE\tBEHAVIOR: RED\n' >"$scratch/research/mutations/stale-anchor.tsv"
  printf '改注释\t// 注释行\t// 改过的注释行\n' >"$scratch/research/mutations/comment-only.tsv"

  # 跑一次 mutate.sh：$1 标签、$2 变异表，其余是额外的环境变量；stdout 落 $scratch/<标签>.stdout、stderr 落 .stderr、两样接起来落 .out，
  # 退出码落 $scratch/<标签>.rc，假 cargo 的调用记录落 $scratch/<标签>.cargo
  run_case() {
    local label="$1" table="$2"
    shift 2
    : >"$scratch/$label.cargo"
    if (cd "$scratch/research" && env PATH="$scratch/bin:$PATH" FAKE_CARGO_LOG="$scratch/$label.cargo" MUTATE_MEMORY_MAX=64M MUTATE_TIMEOUT=2 \
          MUTATE_TARGET_DIR="$scratch/target" "$@" bash "$SELF_PATH" fake-bench bench/src/bin/fake_bench.rs "$table" \
          >"$scratch/$label.stdout" 2>"$scratch/$label.stderr"); then
      echo 0 >"$scratch/$label.rc"
    else
      echo $? >"$scratch/$label.rc"
    fi
    cat "$scratch/$label.stderr" "$scratch/$label.stdout" >"$scratch/$label.out"
  }
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail

  run_case every-outcome mutations/every-outcome.tsv
  output="$(cat "$scratch/every-outcome.out")"; status="$(cat "$scratch/every-outcome.rc")"
  checked=$((checked + 1))
  [[ "$status" == 1 ]] || fail "五种结局各一条的表应当整轮判失败退 1，实际退 $status：$output"
  local wanted
  for wanted in "✅ [抓到] 红：tests::adds" "❌ [没红]" "⏭  [无效]" "⏱  [超时]" "🧱 [撞内存] 撞了内存上限 64M（内存撞顶）" \
                "计数：内存撞顶 1 条（上限 64M）、超时 1 条" "已还原，基线仍全绿"; do
    checked=$((checked + 1))
    grep -qF -- "$wanted" <<<"$output" || fail "五种结局的表输出里应当有「$wanted」，实际：$output"
  done
  checked=$((checked + 1))
  if grep -E '^(✅|❌|💥|⏭|⚠️) +\[撞内存\]' <<<"$output" >/dev/null; then
    fail "撞了内存上限的那一条不许记成抓到、没红、没跑完或无效：$(grep -F '[撞内存]' <<<"$output")"
  fi

  run_case caught-and-memory mutations/caught-and-memory.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/caught-and-memory.rc")"
  [[ "$status" == 1 ]] || fail "只有一条抓到、一条内存撞顶的表应当判失败退 1（撞顶与超时同类），实际退 $status：$(cat "$scratch/caught-and-memory.out")"

  run_case caught-only mutations/caught-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/caught-only.rc")"
  output="$(cat "$scratch/caught-only.out")"
  [[ "$status" == 0 ]] && grep -qF "计数：内存撞顶 0 条（上限 64M）、超时 0 条" <<<"$output" \
    || fail "上限之内正常的变异照旧判抓到、整轮退 0 并报「内存撞顶 0 条」，实际退 $status：$output"

  # 带上限的 scope 起不来：退 7，假 cargo 一次都没被调（连基线都没跑，不退回无上限）
  run_case unavailable mutations/caught-only.tsv DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent
  checked=$((checked + 1))
  status="$(cat "$scratch/unavailable.rc")"
  if [[ "$status" != 7 || -s "$scratch/unavailable.cargo" ]] || ! grep -q '起不来' "$scratch/unavailable.out"; then
    fail "D-Bus 连不上时应当退 7、报「起不来」、假 cargo 一次都不调，实际退 $status、调了 $(wc -l <"$scratch/unavailable.cargo") 次：$(cat "$scratch/unavailable.out")"
  fi

  # ── 并行 ──
  # 这台机器上 MUTATE_JOBS=4、五条的表应当开几个：min(核数的一半且至多 16、4、5)，至少 1
  local processor_count expected_workers mutated_runs distinct_copies distinct_targets distinct_pairs
  processor_count="$(nproc 2>/dev/null || echo 2)"
  expected_workers=$((processor_count / 2))
  ((expected_workers > MAXIMUM_WORKERS_BY_PROCESSOR)) && expected_workers=$MAXIMUM_WORKERS_BY_PROCESSOR
  ((expected_workers > 4)) && expected_workers=4
  ((expected_workers < 1)) && expected_workers=1
  run_case serial mutations/every-outcome.tsv MUTATE_JOBS=1
  run_case parallel mutations/every-outcome.tsv MUTATE_JOBS=4
  checked=$((checked + 1))
  cmp -s "$scratch/serial.stdout" "$scratch/parallel.stdout" \
    || fail "MUTATE_JOBS=1 与 =4 的 stdout 应当逐字相同（判定行按表序打），实际不同：$(diff "$scratch/serial.stdout" "$scratch/parallel.stdout")"
  checked=$((checked + 1))
  status="$(cat "$scratch/parallel.rc")"
  if [[ "$status" != 1 ]] || ! grep -qF "❌ [没红]" "$scratch/parallel.stdout" || ! grep -qF "✅ [抓到] 红：tests::adds" "$scratch/parallel.stdout"; then
    fail "MUTATE_JOBS=4 时没红的那一条照样让整轮退 1、抓到的那一条照样判抓到，实际退 $status：$(cat "$scratch/parallel.out")"
  fi
  checked=$((checked + 1))
  [[ "$(head -1 "$scratch/parallel.stderr")" == *"这一轮开 $expected_workers 个工作进程"* ]] \
    || fail "MUTATE_JOBS=4、五条的表，stderr 头一行应当报「这一轮开 $expected_workers 个工作进程」，实际：$(head -1 "$scratch/parallel.stderr")"
  checked=$((checked + 1))
  [[ "$(head -1 "$scratch/serial.stderr")" == *"这一轮开 1 个工作进程"*"卡在「MUTATE_JOBS」这一项"* ]] \
    || fail "MUTATE_JOBS=1 时 stderr 头一行应当报开 1 个、卡在「MUTATE_JOBS」，实际：$(head -1 "$scratch/serial.stderr")"
  # 施加了变异的那几次（BEHAVIOR 不是 PASS）落在几份副本上：至少两份，才说明变异真的分给了不同的工作进程；
  # 每份副本只配一个编译目录、几份各不相同，才说明没有两个工作进程在同一份源码或同一个编译目录上并发
  if ((expected_workers >= 2)); then
    checked=$((checked + 1))
    mutated_runs="$(awk -F'|' '$3 != "PASS"' "$scratch/parallel.cargo")"
    distinct_copies="$(cut -d'|' -f1 <<<"$mutated_runs" | sort -u | wc -l)"
    distinct_targets="$(cut -d'|' -f2 "$scratch/parallel.cargo" | sort -u | wc -l)"
    distinct_pairs="$(cut -d'|' -f1,2 "$scratch/parallel.cargo" | sort -u | wc -l)"
    if ((distinct_copies < 2 || distinct_targets != expected_workers || distinct_pairs != expected_workers)); then
      fail "MUTATE_JOBS=4 时变异应当落在至少 2 份副本上、$expected_workers 个工作进程各用一份副本与一个编译目录，实际施加变异的副本 $distinct_copies 份、编译目录 $distinct_targets 个、(副本, 编译目录) 组合 $distinct_pairs 个：$(cat "$scratch/parallel.cargo")"
    fi
  fi

  # 丢一份判定：收回的比派出去的少，整道退 8，不许少一条还报绿；再拿掉那道闸（countblind），同一个丢法要静默判绿，才说明退 8 是那道闸给的
  run_case lost-result mutations/three-caught.tsv MUTATE_JOBS=3 MUTATE_BREAK="droprow${BROKEN_JUDGEMENT:+,$BROKEN_JUDGEMENT}"
  checked=$((checked + 1))
  status="$(cat "$scratch/lost-result.rc")"
  if [[ "$status" != 8 ]] || ! grep -qF "派出去 3 条" "$scratch/lost-result.stderr" || ! grep -qF "收回 2 份" "$scratch/lost-result.stderr"; then
    fail "丢了一份判定时应当退 8 并报「派出去 3 条……收回 2 份」，实际退 $status：$(cat "$scratch/lost-result.out")"
  fi
  run_case lost-result-unchecked mutations/three-caught.tsv MUTATE_JOBS=3 MUTATE_BREAK=droprow,countblind
  checked=$((checked + 1))
  status="$(cat "$scratch/lost-result-unchecked.rc")"
  if [[ "$status" != 0 || "$(grep -c '^✅' "$scratch/lost-result-unchecked.stdout")" != 2 ]]; then
    fail "拿掉收束的计数闸之后，同一个丢法应当静默退 0、只报 2 条抓到（这样上一项的退 8 才是那道闸给的），实际退 $status：$(cat "$scratch/lost-result-unchecked.out")"
  fi

  # MUTATE_JOBS 不是正整数：退 2，一次 cargo 都不调
  run_case bad-jobs mutations/caught-only.tsv MUTATE_JOBS=0
  checked=$((checked + 1))
  status="$(cat "$scratch/bad-jobs.rc")"
  if [[ "$status" != 2 || -s "$scratch/bad-jobs.cargo" ]] || ! grep -qF "MUTATE_JOBS" "$scratch/bad-jobs.stderr"; then
    fail "MUTATE_JOBS=0 应当退 2、点名 MUTATE_JOBS、一次 cargo 都不调，实际退 $status、调了 $(wc -l <"$scratch/bad-jobs.cargo") 次：$(cat "$scratch/bad-jobs.out")"
  fi

  # 锚点腐化：派活之前就退 3，连基线都不跑、一条判定都不打（并行下不能让前面几条先跑了、后面那条才炸）
  run_case stale-anchor mutations/stale-anchor.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/stale-anchor.rc")"
  if [[ "$status" != 3 || -s "$scratch/stale-anchor.cargo" || -s "$scratch/stale-anchor.stdout" ]] \
     || ! grep -qF "[锚点腐化] 替换没命中" "$scratch/stale-anchor.stderr"; then
    fail "锚点腐化的表应当在派活之前退 3、点名那一条、一次 cargo 都不调、stdout 一行都没有，实际退 $status、调了 $(wc -l <"$scratch/stale-anchor.cargo") 次：$(cat "$scratch/stale-anchor.out")"
  fi

  # 基线（没改的源码）就撞上限：退 2 并说是内存上限，不说成「基线就是红的」。
  # 表里的原文要在 HOG 版源码里命中得上（锚点在基线之前核，命中不上就先退 3 了），所以用只改注释的那张表
  printf '// 注释行\nfn main() {} // BEHAVIOR: HOG\n' >"$scratch/research/bench/src/bin/fake_bench.rs"
  run_case baseline-memory mutations/comment-only.tsv
  checked=$((checked + 1))
  status="$(cat "$scratch/baseline-memory.rc")"
  if [[ "$status" != 2 ]] || ! grep -q '基线.*撞了内存上限 64M' "$scratch/baseline-memory.out"; then
    fail "基线就撞上限时应当退 2 并报「基线……撞了内存上限 64M」，实际退 $status：$(cat "$scratch/baseline-memory.out")"
  fi

  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ mutate.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "  → 怎么办：照上面逐条改——判结局的在 judge_mutation（退出码 124 / $MEMORY_CAP_HIT_EXIT / $MEMORY_CAP_UNAVAILABLE_EXIT 的分支），基线与收尾计数在主流程，"
    echo "    并行的在 worker_count_and_head_line、run_worker 与 print_judged_rows_in_table_order；MUTATE_BREAK 或 RUN_WITH_MEMORY_CAP_BREAK 设着的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ mutate.sh 自检通过（查了 $checked 项：抓到、没红、无效、超时、内存撞顶五种结局各归各类，撞顶不记成抓到或没红、整轮判失败并单列计数，\
上限之内正常的变异照旧判抓到退 0，scope 起不来退 7 且一次 cargo 都没跑，基线撞顶退 2 并说是内存上限；\
并行：MUTATE_JOBS=1 与 =4 的 stdout 逐字相同、没红的照样判失败、头一行报开了几个、各工作进程各用一份副本与编译目录，\
丢一份判定退 8 而拿掉那道闸就静默判绿，MUTATE_JOBS 写错退 2，锚点腐化派活之前退 3）"
}

if [[ "${1:-}" == "--selftest" ]]; then
  run_selftest
  exit 0
fi
BIN="$1"; SRC="$2"; TABLE="$3"

# ⚠️ **先确认「改的文件」与「跑的二进制」对得上。**
# 不确认的话，变异会改 X 而跑 Y 的测试：一条真能被抓的破坏被报成盲区
# （2026-08-29 实测踩过：`e7_index.rs` 配上 crate 主二进制的名字，
# 三条变异全被误报成盲区，换对名字后三条全被抓）。
# 反方向更坏：Y 的测试恰好因别的原因红了，会被记成「变异被抓」。
_manifest="$(dirname "$SRC")/../../Cargo.toml"
[[ -f "$_manifest" ]] || _manifest="e7-index-bench/Cargo.toml"
_declared="$(awk -v src="$SRC" '
  /^\[\[bin\]\]/ { name=""; path=""; next }
  /^name *=/ { gsub(/.*= *"|"/,""); name=$0; next }
  /^path *=/ { gsub(/.*= *"|"/,""); path=$0;
               if (src ~ path"$") print name; next }
' "$_manifest" 2>/dev/null | head -1)"
if [[ -n "$_declared" ]]; then
  if [[ "$_declared" != "$BIN" ]]; then
    echo "mutate: 源文件 $SRC 在 Cargo.toml 里声明的二进制是 '$_declared'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_declared $SRC $TABLE" >&2
    exit 6
  fi
else
  # 没有显式 [[bin]] ⇒ cargo 自动发现，名字就是文件名去掉扩展名
  _stem="$(basename "$SRC" .rs)"
  if [[ "$_stem" != "$BIN" ]]; then
    echo "mutate: $SRC 没有显式 [[bin]]，自动发现的名字是 '$_stem'，不是 '$BIN'" >&2
    echo "        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh $_stem $SRC $TABLE" >&2
    exit 6
  fi
fi

# ⚠️ **变异只改副本，不碰工作区里的源文件**（2026-09-12 改）。此前是就地改 $SRC 再还原：一轮变异要几分钟，
# 这几分钟里仓里那份源码是坏的——几个会话共写一个仓时，别的会话此刻跑门禁（15 号阶段会编译并跑 research 的全部单测）
# 就编到被改坏的源码，红得莫名其妙、还可能当成自己改坏的；此刻有人整份提交这个文件，提交进去的就是变异。
# ⇒ 把 research/ 里编译要用的部分（workspace、crate 源码、include_str! 读的 results/、data/）拷到临时目录，
#   在那里改、在那里编；CARGO_TARGET_DIR 单独一份，不碰 research/target 里别人要用的二进制。跑完删掉副本。
#   并行时每个工作进程各一份这样的副本、各一个编译目录（文件头「并行」那一段）。
if [[ ! -f Cargo.toml ]] || ! grep -q '^\[workspace\]' Cargo.toml; then
  echo "mutate: 要在 research/ 下跑（那里才有 workspace 的 Cargo.toml）" >&2; exit 2
fi
REQUESTED_JOBS="${MUTATE_JOBS:-}"
if [[ -n "$REQUESTED_JOBS" && ! "$REQUESTED_JOBS" =~ ^[1-9][0-9]*$ ]]; then
  echo "mutate: MUTATE_JOBS 写成了「$REQUESTED_JOBS」，要写正整数（它只能把工作进程数压小），一条变异都没跑" >&2
  echo "        → 怎么办：写成 1、2、4 这样的正整数再跑；不设就按核数的一半（至多 $MAXIMUM_WORKERS_BY_PROCESSOR）与表里的条数取小" >&2
  exit 2
fi
REQUESTED_CARGO_JOBS="${MUTATE_CARGO_JOBS:-}"
if [[ -n "$REQUESTED_CARGO_JOBS" && ! "$REQUESTED_CARGO_JOBS" =~ ^[1-9][0-9]*$ ]]; then
  echo "mutate: MUTATE_CARGO_JOBS 写成了「$REQUESTED_CARGO_JOBS」，要写正整数，一条变异都没跑" >&2
  echo "        → 怎么办：写成正整数再跑；不设就用调用方的 CARGO_BUILD_JOBS，那也没设就取 核数 ÷ 工作进程数" >&2
  exit 2
fi
# 开跑之前先试一次带上限的 scope 起不起得来：起不来就一条都不跑，不退回无上限（run-with-memory-cap.sh 自己把原因打到 stderr）
if bash "$MEMORY_CAP_RUNNER" --check "$MEMORY_MAX"; then memory_cap_check=0; else memory_cap_check=$?; fi
if [[ $memory_cap_check -ne 0 ]]; then
  echo "mutate: 带内存上限（MUTATE_MEMORY_MAX=$MEMORY_MAX）的 systemd scope 起不来，一条变异都没跑" >&2
  echo "        → 怎么办：看上面 run-with-memory-cap.sh 的报错，修好用户级 systemd / D-Bus 或上限的写法再跑；不许拿掉上限去跑（无界分配的变异会把整机拖进 OOM）" >&2
  exit 7
fi

# 读表：一行三段、制表符分隔，空行与 # 开头的行跳过（与改成并行之前逐行读的口径相同），按表序进三个数组
mutation_names=(); mutation_originals=(); mutation_replacements=()
while IFS=$'\t' read -r name from to; do
  [[ -z "${name:-}" || "${name:0:1}" == "#" ]] && continue
  mutation_names+=("$name"); mutation_originals+=("$from"); mutation_replacements+=("$to")
done < "$TABLE"
mutation_count=${#mutation_names[@]}

# 开几个工作进程：各项取最小、至少 1，连同各项给几个、卡在哪一项、每个 cargo 的编译并行度一起打在 stderr 的头一行（文件头「并行」那一段）
worker_count_and_head_line() {
  local processor_count by_processor smallest binding="" floor_note="" bound_index
  local -a bound_names=() bound_counts=() bound_texts=()
  processor_count="$(nproc 2>/dev/null || echo 2)"
  by_processor=$((processor_count / 2))
  ((by_processor > MAXIMUM_WORKERS_BY_PROCESSOR)) && by_processor=$MAXIMUM_WORKERS_BY_PROCESSOR
  ((by_processor < 1)) && by_processor=1
  bound_names+=("核数"); bound_counts+=("$by_processor"); bound_texts+=("核数 $processor_count 的一半、至多 $MAXIMUM_WORKERS_BY_PROCESSOR，给 $by_processor 个")
  if [[ -n "$REQUESTED_JOBS" ]]; then
    bound_names+=("MUTATE_JOBS"); bound_counts+=("$REQUESTED_JOBS"); bound_texts+=("MUTATE_JOBS 给 $REQUESTED_JOBS 个")
  fi
  bound_names+=("表里的条数"); bound_counts+=("$mutation_count"); bound_texts+=("表里 $mutation_count 条")
  smallest="${bound_counts[0]}"
  for bound_index in "${!bound_counts[@]}"; do
    ((bound_counts[bound_index] < smallest)) && smallest="${bound_counts[bound_index]}"
  done
  for bound_index in "${!bound_counts[@]}"; do
    ((bound_counts[bound_index] == smallest)) && binding+="${binding:+、}「${bound_names[bound_index]}」"
  done
  ((smallest < 1)) && floor_note="（那一项不足 1，按 1 个开）"
  WORKER_COUNT=$((smallest < 1 ? 1 : smallest))
  if [[ -n "$REQUESTED_CARGO_JOBS" ]]; then
    CARGO_JOBS_PER_WORKER="$REQUESTED_CARGO_JOBS"; CARGO_JOBS_SOURCE="MUTATE_CARGO_JOBS"
  elif [[ -n "${CARGO_BUILD_JOBS:-}" ]]; then
    CARGO_JOBS_PER_WORKER="$CARGO_BUILD_JOBS"; CARGO_JOBS_SOURCE="调用方的 CARGO_BUILD_JOBS"
  else
    CARGO_JOBS_PER_WORKER=$((processor_count / WORKER_COUNT < 1 ? 1 : processor_count / WORKER_COUNT)); CARGO_JOBS_SOURCE="核数 ÷ 工作进程数"
  fi
  local joined_texts
  joined_texts="$(printf '%s；' "${bound_texts[@]}")"
  echo "mutate: 这一轮开 $WORKER_COUNT 个工作进程（各项取最小、至少 1）：${joined_texts%；}。卡在${binding}这一项${floor_note}；每个 cargo 编译并行度 $CARGO_JOBS_PER_WORKER（${CARGO_JOBS_SOURCE}）" >&2
}
worker_count_and_head_line
export CARGO_BUILD_JOBS="$CARGO_JOBS_PER_WORKER"

SHARD_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-mutate-XXXXXX")"
PRISTINE="$SHARD_ROOT/pristine-source"   # 开跑时那一份被测源码，每条变异都从它改、每次还原都拷它
CLAIMS="$SHARD_ROOT/claims"              # <行号>/：那一条被哪个工作进程认领了（mkdir 建成的那个领走）
RESULTS="$SHARD_ROOT/results"            # <行号>.line（要打到 stdout 的那一行）、.stderr、.verdict（「档 要不要判失败」，最后写）
STOP_FLAG="$SHARD_ROOT/stop"             # 有一条判出「scope 起不来」或锚点没命中：其余工作进程领完手上那条就不再领
mkdir -p "$CLAIMS" "$RESULTS"
worker_pids=()
cleanup() {
  local pid
  for pid in "${worker_pids[@]}"; do
    if kill -0 "$pid" 2>/dev/null; then
      kill -TERM "$pid" 2>/dev/null
      wait "$pid" 2>/dev/null
    fi
  done
  rm -rf "${SHARD_ROOT:?}"
}
trap cleanup EXIT
MUTATE_TARGET="${MUTATE_TARGET_DIR:-${TMPDIR:-/tmp}/singlefs-mutate-target}"
copy_directory_of() { echo "$SHARD_ROOT/copy-$1"; }
target_directory_of() { if (($1 == 1)); then echo "$MUTATE_TARGET"; else echo "$MUTATE_TARGET-w$1"; fi; }

if ! rsync -a --exclude target --exclude prompts --exclude mutations --exclude scripts ./ "$(copy_directory_of 1)/"; then
  echo "mutate: 拷副本失败" >&2; exit 2
fi
ORIGINAL_SUM="$(sha256sum "$SRC" | cut -d' ' -f1)"
cp "$(copy_directory_of 1)/$SRC" "$PRISTINE"

# 从开跑时那一份源码施加一条变异：$1 原文、$2 替换文、$3 写到哪（空串只核命中、不写）。原文要恰好命中一次，否则退 1 并把命中数打到 stderr
write_mutated_source() {
  FROM="$1" TO="$2" PRISTINE="$PRISTINE" DESTINATION="$3" python3 - <<'PY'
import os,sys
s=open(os.environ["PRISTINE"]).read()
f=os.environ["FROM"].replace("\\n","\n"); t=os.environ["TO"].replace("\\n","\n")
n=s.count(f)
if n!=1:
    sys.stderr.write("命中 %d 次（要求恰好 1 次）：%r\n"%(n,f)); sys.exit(1)
if os.environ["DESTINATION"]:
    open(os.environ["DESTINATION"],"w").write(s.replace(f,t))
PY
}
# 锚点在派活之前逐条核：并行下不能让前面几条先跑了、后面那条才炸（mutation-sampling.md 第七类：锚点腐化那一轮一条都不算数）
for ((row_index = 0; row_index < mutation_count; row_index++)); do
  if ! write_mutated_source "${mutation_originals[row_index]}" "${mutation_replacements[row_index]}" ""; then
    echo "mutate: [${mutation_names[row_index]}] 替换没命中，证明作废（锚点在派活之前逐条核，这一轮一条变异都没跑）" >&2
    echo "        → 怎么办：把变异表里这一条的原文改到源码今天的写法、让它恰好命中一次，再整张表重跑" >&2
    exit 3
  fi
done

# 第 2 份起的副本从第 1 份拷：同一份源码快照，不在拷的间隙里读到别人刚改的工作区
for ((worker_index = 2; worker_index <= WORKER_COUNT; worker_index++)); do
  if ! rsync -a "$(copy_directory_of 1)/" "$(copy_directory_of "$worker_index")/"; then
    echo "mutate: 拷第 $worker_index 份副本失败" >&2; exit 2
  fi
done
# ⚠️ rsync -a 保留源码的旧 mtime，而编译目录跨轮共用：上一轮中途退出（替换没命中 exit 3）时
# 那里留着一份**打着变异**编出来的二进制，它比副本里的源码新 ⇒ cargo 判「不用重编」，基线直接跑变异版。
# 实测（2026-09-12）：e114 在 M6 处退出后，下一轮基线连报两次「基线就是红的」，而单测直接跑 14 条全绿。
# 先 touch 每份副本里的被测源码，逼 cargo 按这一轮的副本重编（每个工作进程的编译目录都跨轮留着，每份都要 touch）。
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  touch "$(copy_directory_of "$worker_index")/$SRC"
done
# 在第 $1 个工作进程的副本上、用它的编译目录跑一次不限时的 cargo test（基线与还原之后那一次）
run_tests_in_copy() {
  ( cd "$(copy_directory_of "$1")" && CARGO_TARGET_DIR="$(target_directory_of "$1")" bash "$MEMORY_CAP_RUNNER" "$MEMORY_MAX" cargo test --release --bin "$BIN" )
}

# 基线必须全绿，否则后面「红了」分不清是变异造成的还是本来就红。每份副本各跑一次、一起跑，按副本号逐个收退出码
baseline_pids=()
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  run_tests_in_copy "$worker_index" >/dev/null 2>&1 &
  baseline_pids+=($!)
done
baseline_exits=()
for pid in "${baseline_pids[@]}"; do
  if wait "$pid"; then baseline_exits+=(0); else baseline_exits+=($?); fi
done
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  baseline_exit="${baseline_exits[worker_index - 1]}"
  copy_note=""
  ((worker_index > 1)) && copy_note="（第 $worker_index 份副本上；第 1 份是绿的就多半是这份的编译目录坏了，删掉 $(target_directory_of "$worker_index") 再来）"
  if [[ $baseline_exit -eq $MEMORY_CAP_HIT_EXIT ]]; then
    echo "mutate: 基线（没改过的源码）就撞了内存上限 $MEMORY_MAX${copy_note}：先查被测代码为什么吃这么多，确实要这么多就调大 MUTATE_MEMORY_MAX 再来" >&2; exit 2
  elif [[ $baseline_exit -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]]; then
    echo "mutate: 跑基线时带内存上限的 scope 起不来了${copy_note}，一条变异都没跑；修好用户级 systemd / D-Bus 再来" >&2; exit 7
  elif [[ $baseline_exit -ne 0 ]]; then
    echo "mutate: 基线就是红的${copy_note}，先修好再来" >&2; exit 2
  fi
done
echo "基线：全绿"

# 一条变异的判定写进结果文件：$1 行号、$2 档、$3 要不要判失败（0 / 1）、$4 要打到 stdout 的那一行（可空）、$5 要打到 stderr 的话（可空）。
# .verdict 最后写、先写临时名再改名：父进程看见它就说明这一条的几份都齐了
record_verdict() {
  local row_index="$1"
  if is_broken droprow && ((row_index == 1)); then
    return
  fi
  [[ -n "$4" ]] && printf '%s\n' "$4" >"$RESULTS/$row_index.line"
  [[ -n "$5" ]] && printf '%s\n' "$5" >"$RESULTS/$row_index.stderr"
  printf '%s %s\n' "$2" "$3" >"$RESULTS/$row_index.verdict.partial"
  mv "$RESULTS/$row_index.verdict.partial" "$RESULTS/$row_index.verdict"
}

# 第 $1 个工作进程判表里第 $2 条：从开跑时那一份源码施加变异、限时跑、判结局，写进结果文件（record_verdict）
judge_mutation() {
  local worker_index="$1" row_index="$2" name copy_directory target_directory out row_exit sig red memory_fails
  name="${mutation_names[row_index]}"
  copy_directory="$(copy_directory_of "$worker_index")"
  target_directory="$(target_directory_of "$worker_index")"
  if is_broken sharedcopy; then
    copy_directory="$(copy_directory_of 1)"; target_directory="$(target_directory_of 1)"
  fi
  if ! write_mutated_source "${mutation_originals[row_index]}" "${mutation_replacements[row_index]}" "$copy_directory/$SRC" 2>"$RESULTS/$row_index.anchor"; then
    touch "$STOP_FLAG"
    record_verdict "$row_index" anchor 1 "" "$(cat "$RESULTS/$row_index.anchor")
mutate: [$name] 替换没命中，证明作废
        → 怎么办：把变异表里这一条的原文改到源码今天的写法、让它恰好命中一次，再整张表重跑"
    return
  fi
  # ⚠️ **第五种结局：测试根本不终止。** 一条破坏可以让被测代码进死循环——
  # 那时 `cargo test` 永远不返回，整轮变异**挂在这里**，既没有红也没有绿。
  # **挂住不是判红**：它看起来像脚本坏了，而实际发生的事是「这条破坏让代码不终止」。
  # 实测踩过（2026-09-01，E73）：`tree_height` 的循环无界，变异把扇出下界的 guard 改松之后
  # 扇出为 1 时 `cap` 不增长，整轮挂死，只能靠 `kill` 收场。
  # ⇒ 每条变异单独限时。默认 120 秒——本仓最慢的单测不到 1 秒，撞到它就是真挂了。
  # ⚠️ **第六种结局：内存先于超时撞顶。** 无界分配的破坏几十秒就能吃满整机，等不到 120 秒；2026-09-25 E142 的一条变异两次把整机拖进 OOM。
  # ⇒ 每条变异同时放进内存上限（MUTATE_MEMORY_MAX，见文件头），撞顶与超时同一类记。
  out="$(cd "$copy_directory" && CARGO_TARGET_DIR="$target_directory" timeout "${MUTATE_TIMEOUT:-120}" bash "$MEMORY_CAP_RUNNER" "$MEMORY_MAX" cargo test --release --bin "$BIN" 2>&1)"
  row_exit=$?
  cp "$PRISTINE" "$copy_directory/$SRC"   # 还原：下一条从开跑时那一份改，领完之后那一次测试也跑在还原过的源码上
  if [[ $row_exit -eq 124 ]]; then
    record_verdict "$row_index" timeout 1 "⏱  [$name] ${MUTATE_TIMEOUT:-120} 秒没跑完 —— 这条破坏让被测代码不终止了，不是「没抓到」" ""
    return
  fi
  if [[ $row_exit -eq $MEMORY_CAP_HIT_EXIT ]]; then
    if is_broken memoryascaught; then
      record_verdict "$row_index" caught 0 "✅ [$name] 红：（弄坏开关 memoryascaught）" ""
      return
    fi
    memory_fails=1
    is_broken memorynotfail && memory_fails=0
    record_verdict "$row_index" memory "$memory_fails" "🧱 [$name] 撞了内存上限 $MEMORY_MAX（内存撞顶）—— 这条破坏让被测代码无界分配，不是「没抓到」，也不算抓到" ""
    return
  fi
  if [[ $row_exit -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]]; then
    touch "$STOP_FLAG"
    record_verdict "$row_index" unavailable 1 "" "mutate: [$name] 跑到一半带内存上限的 scope 起不来了（上面是 run-with-memory-cap.sh 的报错），后面的变异不跑：$out
        → 怎么办：修好用户级 systemd / D-Bus 整张表重跑；不许拿掉上限去跑"
    return
  fi
  # ⚠️ **编译失败不等于「没抓到」。** 一个改坏了语法或穷尽性的变异根本跑不到测试，
  # 那时既不能记成「测试抓到了」，也不能记成「测试没抓到」——它是一条**无效变异**。
  # 混成一类的话，一个编译不过的变异会被报成测试盲区，把人引去改测试（实测踩过）。
  # ⚠️ 判据是「测试进程有没有跑起来」，**不是「输出里有没有 error」**——
  # `cargo test` 在测试变红时也会打 `error: test failed`，
  # 拿它当编译失败的判据会把每一条成功的变异都误判成无效（实测踩过，全套复跑才发现）。
  # ⚠️ **不许写成 `printf ... | grep -q ...`。** `grep -q` 命中后立刻退出并关闭管道，
  # 上游 printf 收到 SIGPIPE；本脚本开了 `pipefail` ⇒ **整条管道判失败，命中被读成没命中**。
  # 实测：那样写会让每一条「测试确实变红了」的变异都被误报成「编译失败」。
  # 这正是 `.claude/singlefs-ai-sop/rules/command-safety.md`「管道里的退出码不是你想要的那个」。
  # ⇒ 用 here-string，根本不建管道。
  if ! grep -q '^running [0-9]* test' <<<"$out"; then
    record_verdict "$row_index" invalid 0 "⏭  [$name] 变异导致编译失败，本条无效（不计入盲区，也不算命中）" ""
    return
  fi
  # ⚠️ **第四种结局：测试进程根本没跑完**（挂死被 OOM 杀、段错误、abort）。
  # 它既不是「编译失败」也不是「一个测试都没红」——报成后者会把人引去改测试，
  # 而实际发生的事是「这条破坏让被测代码不终止了」。实测踩过：
  # 摘掉一条循环终止条件之后 recover 无限接受记录，进程 SIGKILL，
  # 当时被报成「没有被任何检查看见」。
  if grep -q "process didn't exit successfully" <<<"$out"; then
    sig="$(sed -n 's/.*(signal: \([0-9]*\).*/\1/p' <<<"$out" | head -1)"
    record_verdict "$row_index" crashed 0 "💥 [$name] 测试进程没跑完（signal ${sig:-?}）——破坏被看见了，但不是断言抓到的" ""
    return
  fi
  # ⚠️ **不许枚举字符类——这个坑犯过两次，形状一模一样。**
  # 2026-08-29 写成 `[a-z_]*`，名字带数字的测试匹配不上；
  # 2026-08-31 写成 `[A-Za-z0-9_]*`，**中文测试名**匹配不上——E57 的 9 条变异
  # 全被报成「一个测试都没红」，实际 9 条全红。两次都是**谎报盲区**（把人引去补
  # 一条本来就有的检查），而不是漏报命中，所以两次都很难自己发现。
  # ⇒ 改成「`...` 之前的都算名字」：Rust 的测试名可以是任意 Unicode 标识符，
  #   枚举允许的字符永远追不上。下面那条旧注释留着，它记的是同一个坑的第一次。
  # ⚠️ **字符类必须含数字与大写。** 写成 `[a-z_]*` 时，任何名字里带数字的测试
  # （例如 `..._analytic_io_per_op_of_1_9375`、`..._reads_exactly_122`）匹配不上，
  # `red` 为空 ⇒ **一条确实红了的变异被报成「一个测试都没红」**。
  # 方向是谎报盲区，会把人引去补一条本来就有的检查。2026-08-29 实测踩过并修。
  # ⚠️ **第三次同形：模块名也不许写死。** 2026-09-17 写的是 `^test tests::`，E154 的单测分在
  # `gate_tests` 与 `integration_tests` 两个模块里，7 条变异全被报成「一个测试都没红」，合并成 `mod tests` 之后 7 条全红。
  # ⇒ `test ` 与 ` ... FAILED` 之间的整段都算名字（含模块路径）；另加一道兜底：测试进程报了失败、名字却一个没抓到，
  #   说明抓取规则又有盲区，单独报，不报成「没被看见」。
  red="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' <<<"$out" | paste -sd, -)"
  if [[ -z "$red" ]] && grep -q '^test result: FAILED' <<<"$out"; then
    record_verdict "$row_index" nameblind 1 "⚠️  [$name] 测试进程报了失败，但一个失败的测试名都没抓到 —— mutate.sh 抓名字的规则有盲区，这一条不算盲区也不算命中" ""
  elif [[ -z "$red" ]]; then
    record_verdict "$row_index" notred 1 "❌ [$name] 一个测试都没红 —— 这条破坏没有被任何检查看见" ""
  else
    record_verdict "$row_index" caught 0 "✅ [$name] 红：$red" ""
  fi
}

# 第 $1 个工作进程：按表序认领还没人领的变异、逐条判；领完在自己的副本上跑一次还原之后的测试，退出码写进 restore-<号>.exit。
# 它的 stdout 与 stderr 都进自己的日志（worker-<号>.log），一个字都不往这个脚本的 stdout 写
run_worker() {
  local worker_index="$1" row_index restore_exit
  trap - EXIT
  for ((row_index = 0; row_index < mutation_count; row_index++)); do
    [[ -e "$STOP_FLAG" ]] && break
    mkdir "$CLAIMS/$row_index" 2>/dev/null || continue   # 认领：mkdir 是原子的，同一条只有一个工作进程建得成
    judge_mutation "$worker_index" "$row_index"
  done
  [[ -e "$STOP_FLAG" ]] && return 0
  if run_tests_in_copy "$worker_index" >/dev/null 2>&1; then restore_exit=0; else restore_exit=$?; fi
  echo "$restore_exit" >"$SHARD_ROOT/restore-$worker_index.exit"
}

for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  run_worker "$worker_index" >"$SHARD_ROOT/worker-$worker_index.log" 2>&1 &
  worker_pids+=($!)
done

# 父进程：按表序把已经判完的那几条接着打到 stdout（前面有一条还没判完就停下等它），顺手计数。
# 碰到「scope 起不来」或锚点没命中那一条就停：记下是哪一种，后面的一条都不再打
fail=0
timeout_count=0
memory_hit_count=0
next_row=0
stopped_verdict=""
print_judged_rows_in_table_order() {
  local verdict_category verdict_fails
  while ((next_row < mutation_count)) && [[ -z "$stopped_verdict" && -f "$RESULTS/$next_row.verdict" ]]; do
    read -r verdict_category verdict_fails <"$RESULTS/$next_row.verdict"
    if [[ "$verdict_category" == unavailable || "$verdict_category" == anchor ]]; then
      stopped_verdict="$verdict_category"
      return
    fi
    [[ -f "$RESULTS/$next_row.line" ]] && cat "$RESULTS/$next_row.line"
    [[ "$verdict_category" == timeout ]] && timeout_count=$((timeout_count + 1))
    [[ "$verdict_category" == memory ]] && memory_hit_count=$((memory_hit_count + 1))
    ((verdict_fails)) && fail=1
    next_row=$((next_row + 1))
  done
}
any_worker_alive() {
  local pid
  for pid in "${worker_pids[@]}"; do
    kill -0 "$pid" 2>/dev/null && return 0
  done
  return 1
}
while any_worker_alive; do
  print_judged_rows_in_table_order
  sleep "$COLLECTION_POLL_SECONDS"
done
worker_exits=()
for pid in "${worker_pids[@]}"; do
  if wait "$pid"; then worker_exits+=(0); else worker_exits+=($?); fi
done
print_judged_rows_in_table_order

if [[ "$stopped_verdict" == unavailable ]]; then
  cat "$RESULTS/$next_row.stderr" >&2
  exit 7
fi
if [[ "$stopped_verdict" == anchor ]]; then
  cat "$RESULTS/$next_row.stderr" >&2
  exit 3
fi

# 派出去多少条就要收回来多少份判定：表里的条数、被认领的条数、收回的判定份数、按表序打到的位置，四个数要对得上
claimed_count=0
collected_count=0
missing_names=()
for ((row_index = 0; row_index < mutation_count; row_index++)); do
  [[ -d "$CLAIMS/$row_index" ]] && claimed_count=$((claimed_count + 1))
  if [[ -f "$RESULTS/$row_index.verdict" ]]; then
    collected_count=$((collected_count + 1))
  else
    missing_names+=("${mutation_names[row_index]}")
  fi
done
unhealthy_workers=""
for worker_index in "${!worker_exits[@]}"; do
  ((worker_exits[worker_index] != 0)) && unhealthy_workers+="${unhealthy_workers:+、}第 $((worker_index + 1)) 个退 ${worker_exits[worker_index]}"
done
if is_broken countblind; then
  # 弄坏开关：不数，只把在的那几份按表序打出来（并行化之后最容易写成的那种收法）
  while ((next_row < mutation_count)); do
    if [[ -f "$RESULTS/$next_row.verdict" ]]; then print_judged_rows_in_table_order; else next_row=$((next_row + 1)); fi
  done
elif ((claimed_count != mutation_count || collected_count != mutation_count || next_row != mutation_count)) || [[ -n "$unhealthy_workers" ]]; then
  echo "mutate: 派出去 $mutation_count 条变异，被认领 $claimed_count 条、收回 $collected_count 份判定，按表序打到第 $next_row 条；工作进程退出码不是 0 的：${unhealthy_workers:-无}" >&2
  if ((${#missing_names[@]})); then
    echo "        没收回判定的：$(printf '%s、' "${missing_names[@]}" | sed 's/、$//')" >&2
  fi
  for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
    if [[ -s "$SHARD_ROOT/worker-$worker_index.log" ]]; then
      echo "        第 $worker_index 个工作进程的日志末尾：" >&2
      tail -n 5 "$SHARD_ROOT/worker-$worker_index.log" | sed 's/^/          /' >&2
    fi
  done
  echo "        → 怎么办：这是并行收束自己的完整性闸红了，这一轮的判定不算数；MUTATE_JOBS=1 再跑一遍看串行下齐不齐，" >&2
  echo "          齐就去查认领与收束那一段（run_worker、record_verdict、print_judged_rows_in_table_order）；工作进程被信号杀的，先查是不是被 OOM 或人停了" >&2
  exit 8
fi
# 与超时同一类的两种结局单列计数：它们既不是抓到也不是没红，整轮已经判失败
echo "计数：内存撞顶 $memory_hit_count 条（上限 $MEMORY_MAX）、超时 $timeout_count 条"

# 还原之后那一次：每个工作进程领完之后在自己的副本上各跑了一次
for ((worker_index = 1; worker_index <= WORKER_COUNT; worker_index++)); do
  restore_file="$SHARD_ROOT/restore-$worker_index.exit"
  if [[ ! -f "$restore_file" ]]; then
    echo "mutate: 第 $worker_index 份副本还原之后那一次没跑成（工作进程没留下退出码）" >&2
    echo "        → 怎么办：MUTATE_JOBS=1 再跑一遍；还是没跑成就去查 run_worker 收尾那几行" >&2
    exit 4
  fi
  restore_exit="$(cat "$restore_file")"
  if [[ "$restore_exit" != 0 ]]; then
    echo "mutate: 还原后没回到全绿（第 $worker_index 份副本，退出码 $restore_exit；$MEMORY_CAP_HIT_EXIT 是撞了内存上限，$MEMORY_CAP_UNAVAILABLE_EXIT 是带上限的 scope 起不来）" >&2
    exit 4
  fi
done
echo "已还原，基线仍全绿"
# 这一轮量的是开跑时那一份源码：跑的这几分钟里有人改了工作区里的原件，结果就不算数，要重跑。
if [[ "$(sha256sum "$SRC" | cut -d' ' -f1)" != "$ORIGINAL_SUM" ]]; then
  echo "mutate: $SRC 在这一轮变异跑的时候被改过；上面的结果量的是开跑时那一份，改完之后重跑" >&2
  exit 5
fi
exit $fail
