#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条经内存包装跑，工作目录随意）：
#   bash scripts/hosts/run-with-memory-cap.sh <上限> bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 scripts/hosts/run-with-memory-cap.sh（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 scripts/hosts/run-with-memory-cap.sh）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
# m4-segment-code-r2 攻方腿：在冻结副本的拷贝上打只打印的 TRACE（src/trace-prints.patch），放进原型，编，逐条复现报告里的打中与量的数，
# 每份产出与 $MODEL/out/ 里同名那一份逐字节比（reference-filesystem-probe.txt 两边都去掉「finished in」计时那一行再比）。
# RERUN_ONLY 选几步（空格隔开，默认全部、按下面的次序）；RERUN_REUSE_TREE 指一棵已经打好补丁、编过的拷贝时跳过 prepare 与 build；
# 线程取 RERUN_THREADS（默认 31，与派发提示的「线程上限：31」同一个数）。model-fix 在拷贝上打模型那一处改法（src/model-rollback-relocations.patch）
# 前后各跑一遍同一批随机历史，所以排在最后；它跑完那棵拷贝里的模型是改过的。
rerun_steps() {
  local tree="${RERUN_REUSE_TREE:-$WORK/tree}" log="$WORK/logs" out="$WORK/out"
  local cap="$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" capped="$REPO_ROOT/scripts/hosts/capped.sh" threads="${RERUN_THREADS:-31}"
  local only="${RERUN_ONLY:-prepare build ceiling transient margins empty-file-set reference-filesystem rollback-shapes mount-reads model-fix}"
  local -a compared=()
  RUN_RC=0
  mkdir -p "$log" "$out"
  wants() { [[ " $only " == *" $1 "* ]]; }
  bin_of() { ls -t "$tree"/target/release/deps/"$1"-* | grep -v '\.d$' | head -1; }
  # keep <正则> <日志> <每行截到几个字>：照报告取那几行（一行不中也不算错）
  keep() { { grep -E "$1" "$2" || true; } | cut -c1-"$3"; }
  # run_bin <测试目标> <日志> [变量=值 …] -- [二进制参数 …]：经内存包装直接跑编出来的测试二进制，退出码记在 RUN_RC
  run_bin() {
    local target=$1 logfile=$2; shift 2
    local -a vars=()
    while (($#)) && [[ $1 != -- ]]; do vars+=("$1"); shift; done
    shift
    if env "${vars[@]}" RUST_TEST_THREADS="$threads" nice -n 19 bash "$cap" 4G "$(bin_of "$target")" --nocapture "$@" > "$logfile" 2>&1; then
      RUN_RC=0
    else
      RUN_RC=$?
    fi
  }
  # run_batch <批号> <比重> <起始种子下标> <件数> <每件段数> [变量=值 …]：随机历史分件并行，每件退出码写进 rc 文件，单独一个 wait 等齐，数 rc
  run_batch() {
    local batch=$1 weights=$2 start=$3 pieces=$4 per=$5; shift 5
    local piece bin
    bin="$(bin_of attack_r2_random_histories)"
    rm -f "$log"/b"$batch"-*.rc
    for piece in $(seq 0 $((pieces - 1))); do
      { local piece_rc=0
        env "$@" ATTACK_WEIGHTS="$weights" ATTACK_SEED_START=$((start + piece * per)) ATTACK_SEEDS="$per" ATTACK_OPS=40 nice -n 19 \
          bash "$cap" 2G "$bin" --nocapture > "$log/b$batch-$piece.log" 2>&1 || piece_rc=$?
        echo "$piece_rc" > "$log/b$batch-$piece.rc"; } &
    done
    wait
    [[ "$(ls "$log"/b"$batch"-*.rc | wc -l)" -eq "$pieces" ]] || { echo "name=rerun_batch batch=$batch 作废：rc 文件数对不上"; return 1; }
  }
  if wants prepare && [[ -z "${RERUN_REUSE_TREE:-}" ]]; then
    mkdir -p "$tree"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$tree/"
    mkdir -p "$tree/.cargo" "$tree/.claude" "$tree/crates/singlefs-harness/tests/attack_common"
    cp "$REPO_ROOT/.cargo/config.toml" "$tree/.cargo/"
    cp "$REPO_ROOT/.claude/heavy-thread-default" "$tree/.claude/"
    (cd "$tree" && patch -p1 --quiet < "$MODEL/src/trace-prints.patch")
    cp "$MODEL"/src/attack_r2_*.rs "$tree/crates/singlefs-harness/tests/"
    cp "$MODEL/src/attack_common/weights_r2.rs" "$tree/crates/singlefs-harness/tests/attack_common/"
    echo "name=rerun_prepared tree=$tree"
  fi
  local -a tests=(--test attack_r2_random_histories --test attack_r2_minimize --test attack_r2_ceiling_dump --test attack_r2_rollback_shapes
    --test attack_r2_transient_read --test attack_r2_empty_file_set_snapshots --test attack_r2_reference_filesystem_probe)
  if wants build && [[ -z "${RERUN_REUSE_TREE:-}" ]]; then
    (cd "$tree" && nice -n 19 bash "$capped" "$threads" cargo test --release --offline -p singlefs-harness "${tests[@]}" --no-run) > "$log/build.log" 2>&1
    echo "name=rerun_built $(tail -n 1 "$log/build.log")"
  fi
  if wants ceiling; then
    run_bin attack_r2_ceiling_dump "$log/dump476.log" ATTACK_TRACE=1 ATTACK_WEIGHTS=with-rollback ATTACK_SEED_INDEX=476 ATTACK_KEEP=0,2,3,5,6,7 --
    keep '^(START|STEP|ROOTS|ENDING|TRACE compaction publish)' "$log/dump476.log" 2000 > "$out/trace476.txt"
    run_bin attack_r2_ceiling_dump "$log/dump535.log" ATTACK_WEIGHTS=with-rollback ATTACK_SEED_INDEX=535 ATTACK_KEEP=0,4,5,6,7,11,12,15 --
    keep '^(STEP|ROOTS|ENDING)' "$log/dump535.log" 2000 > "$out/dump535.txt"
    run_bin attack_r2_ceiling_dump "$log/dump640.log" ATTACK_WEIGHTS=with-rollback ATTACK_SEED_INDEX=640 ATTACK_KEEP=14,15,16,19,20,22,23,24 --
    keep '^(STEP|ROOTS|ENDING)' "$log/dump640.log" 2000 > "$out/dump640.txt"
    run_bin attack_r2_ceiling_dump "$log/dump3395.log" ATTACK_TRACE=1 ATTACK_DEVICE_WIDTH=384 ATTACK_WEIGHTS=r2-mix ATTACK_SEED_INDEX=3395 ATTACK_KEEP=0,2,3,4,5 --
    keep '^(STEP|ROOTS|ENDING|TRACE compaction publish|TRACE issue)' "$log/dump3395.log" 2000 > "$out/dump3395.txt"
    run_bin attack_r2_minimize "$log/trace297.log" ATTACK_TRACE=1 ATTACK_VERBOSE=1 ATTACK_WEIGHTS=with-rollback ATTACK_SEED_INDEX=297 \
      ATTACK_KEEP=0,1,2,5,8,11,12,14,15,16,18,19,20,21,22,23,24,25,26,27 --
    keep '^(START|STEP|ENDING|TRACE compaction|TRACE   planned)' "$log/trace297.log" 1500 > "$out/trace297.txt"
    compared+=(trace476.txt dump535.txt dump640.txt dump3395.txt trace297.txt)
  fi
  if wants transient; then
    run_bin attack_r2_transient_read "$log/transient-completing.log" ATTACK_TRACE=1 ATTACK_FILL_THE_OPEN_SEGMENT=1 ATTACK_FAIL_PAIRS=1 \
      ATTACK_FAIL_FROM=216 ATTACK_FAIL_TO=217 -- a_transient_read_error_between
    keep '^(BASELINE|INJECT|TRACE (fallback|carry|issue segs0|compaction publish))' "$log/transient-completing.log" 400 > "$out/transient-completing.txt"
    run_bin attack_r2_transient_read "$log/transient-1055.log" ATTACK_PREFIX=seed1055 ATTACK_REGION_START=50432 ATTACK_BATCH_UNITS=1 ATTACK_TRACE=1 \
      ATTACK_FAIL_PAIRS=1 ATTACK_FAIL_FROM=339 ATTACK_FAIL_TO=340 -- a_transient_read_error_between
    keep '^(BASELINE|INJECT|TRACE (carry|fallback|compaction publish txg=3[0-9]))' "$log/transient-1055.log" 400 > "$out/transient-1055.txt"
    run_bin attack_r2_transient_read "$log/transient-pairs.log" ATTACK_FILL_THE_OPEN_SEGMENT=1 ATTACK_FAIL_PAIRS=1 ATTACK_FAIL_FROM=21 ATTACK_FAIL_TO=37 -- a_transient_read_error_between
    keep '^(BASELINE|INJECT)' "$log/transient-pairs.log" 400 > "$out/transient-issue-pairs.txt"
    compared+=(transient-completing.txt transient-1055.txt transient-issue-pairs.txt)
  fi
  if wants margins; then
    local units target
    for units in 3 1; do
      for target in default 8; do
        if [[ $target == default ]]; then
          run_bin attack_r2_transient_read "$log/margin-$units-$target.log" ATTACK_BATCH_UNITS=$units -- fixed_point_slots
        else
          run_bin attack_r2_transient_read "$log/margin-$units-$target.log" ATTACK_BATCH_UNITS=$units ATTACK_ROLLBACK_TXG=$target -- fixed_point_slots
        fi
        echo "batch=$units target=$target rc=$RUN_RC"
        keep '^(MARGIN|ROLLBACK)' "$log/margin-$units-$target.log" 160
      done
    done > "$out/checkpoint-cost-margins.txt"
    compared+=(checkpoint-cost-margins.txt)
  fi
  if wants empty-file-set; then
    run_bin attack_r2_empty_file_set_snapshots "$log/empty-sweep.log" -- snapshots_on_an_empty_file_set_sweep
    run_bin attack_r2_empty_file_set_snapshots "$log/empty-rollback.log" -- rollback_after
    local case_name
    for case_name in "snapshots=11 then=read-newest" "snapshots=3 then=rollback-6" "snapshots=10 then=destroy-oldest"; do
      run_bin attack_r2_empty_file_set_snapshots "$log/empty-sample.log" ATTACK_VERBOSE=1 "ATTACK_ONLY=$case_name" -- snapshots_on_an_empty_file_set_sweep
      keep '^(CASE|OUTCOMES)' "$log/empty-sample.log" 2500
    done > "$log/empty-samples.log"
    { keep '^(CASE|SWEEP)' "$log/empty-sweep.log" 100000; keep '^(CASE|SWEEP)' "$log/empty-rollback.log" 100000; cut -c1-3000 "$log/empty-samples.log"; } \
      > "$out/empty-file-set-snapshots.txt"
    compared+=(empty-file-set-snapshots.txt)
  fi
  if wants reference-filesystem; then
    run_bin attack_r2_reference_filesystem_probe "$log/reffs-probe.log" -- --test-threads 1
    cut -c1-600 "$log/reffs-probe.log" | { grep -v 'finished in' || true; } > "$out/reference-filesystem-probe.txt"
    { grep -v 'finished in' "$MODEL/out/reference-filesystem-probe.txt" || true; } > "$log/reference-filesystem-probe.expected"
    if cmp -s "$out/reference-filesystem-probe.txt" "$log/reference-filesystem-probe.expected"; then same=true; else same=false; fi
    echo "name=rerun_compare file=reference-filesystem-probe.txt identical=$same（两边去掉计时那一行）"
    [[ $same == true ]] && RERUN_SAME=$((${RERUN_SAME:-0} + 1)) || RERUN_DIFFER=$((${RERUN_DIFFER:-0} + 1))
  fi
  if wants rollback-shapes; then
    local shape piece=0
    rm -f "$log"/shapes-*.rc
    for shape in two-runs compaction-then-overwrite crash-mid-intent-then-resume crash-mid-intent-overwrite-then-resume \
        single-leaf-selector-0 single-leaf-selector-1 single-leaf-selector-2 single-leaf-selector-3 one-write-batch-3 one-write-crash-mid-intent; do
      { local shape_rc=0
        ATTACK_SHAPE=$shape RUST_TEST_THREADS="$threads" nice -n 19 bash "$cap" 2G "$(bin_of attack_r2_rollback_shapes)" --nocapture \
          > "$log/shapes-$piece.log" 2>&1 || shape_rc=$?
        echo "$shape_rc" > "$log/shapes-$piece.rc"; } &
      piece=$((piece + 1))
    done
    wait
    [[ "$(ls "$log"/shapes-*.rc | wc -l)" -eq 10 ]] || { echo "name=rerun_shapes 作废：rc 文件数对不上"; return 1; }
    for piece in $(seq 0 9); do keep '^(CASE|SWEEP)' "$log/shapes-$piece.log" 100000; done > "$out/rollback-shapes-sweep.txt"
    for case_name in "crash-mid-intent-then-resume remount=false rollback_index=3" "two-runs remount=true rollback_index=7" \
        "compaction-then-overwrite remount=false rollback_index=5"; do
      run_bin attack_r2_rollback_shapes "$log/shapes-sample.log" ATTACK_TRACE=1 ATTACK_VERBOSE=1 "ATTACK_SHAPE=${case_name%% *}" "ATTACK_ONLY=shape=$case_name" --
      keep '^(STEP|CASE|TRACE compaction)' "$log/shapes-sample.log" 700 | { grep -v '^TRACE compaction [^p]' || true; }
    done > "$out/rollback-shapes-samples.txt"
    compared+=(rollback-shapes-sweep.txt rollback-shapes-samples.txt)
  fi
  if wants mount-reads; then
    local chunk
    for mode in single pairs; do
      rm -f "$log"/mount-"$mode"-*.rc
      for chunk in $(seq 0 15); do
        { local chunk_rc=0
          if [[ $mode == pairs ]]; then pair=1; else pair=0; fi
          ATTACK_FAIL_PAIRS=$pair ATTACK_FAIL_FROM=$((chunk * 114)) ATTACK_FAIL_TO=$(((chunk + 1) * 114)) RUST_TEST_THREADS="$threads" nice -n 19 \
            bash "$cap" 2G "$(bin_of attack_r2_transient_read)" --nocapture a_transient_read_error_during > "$log/mount-$mode-$chunk.log" 2>&1 || chunk_rc=$?
          echo "$chunk_rc" > "$log/mount-$mode-$chunk.rc"; } &
      done
      wait
      [[ "$(ls "$log"/mount-"$mode"-*.rc | wc -l)" -eq 16 ]] || { echo "name=rerun_mount_reads 作废：rc 文件数对不上"; return 1; }
    done
    {
      echo "== 单次读错（每个读号单独注入）"
      for chunk in $(seq 0 15); do grep -h MOUNT_SWEEP "$log/mount-single-$chunk.log"; done
      grep -h BASELINE "$log/mount-single-0.log"
      echo "== 一次读与它的重读一起错"
      for chunk in $(seq 0 15); do grep -h MOUNT_SWEEP "$log/mount-pairs-$chunk.log"; done
      for chunk in $(seq 0 15); do grep -h '^INJECT' "$log/mount-pairs-$chunk.log" || true; done > "$log/mount-pairs-injected.txt"
      { grep DIFFERENT "$log/mount-pairs-injected.txt" || true; } | head -6 | cut -c1-400
      { grep REFUSED "$log/mount-pairs-injected.txt" || true; } | cut -c1-300
    } > "$out/mount-transient-reads.txt"
    compared+=(mount-transient-reads.txt)
  fi
  if wants model-fix; then
    # 原样模型：小盘 384 槽 r2-mix 种子下标 3000–3495（31 件 × 16 段）；之后在拷贝上打模型那一处改法、重编随机历史原型，同一批再跑一遍，
    # 另跑 4 GiB 上打中过的四个种子（with-rollback）。
    run_batch 9 r2-mix 3000 31 16 ATTACK_TRACE=1 ATTACK_DEVICE_WIDTH=384
    (cd "$tree" && patch -p1 --quiet < "$MODEL/src/model-rollback-relocations.patch")
    (cd "$tree" && nice -n 19 bash "$capped" "$threads" cargo test --release --offline -p singlefs-harness --test attack_r2_random_histories --no-run) \
      > "$log/build-model-fix.log" 2>&1
    run_batch 11 r2-mix 3000 31 16 ATTACK_DEVICE_WIDTH=384
    local seed index=0
    rm -f "$log"/b12-*.rc
    for seed in 297 476 535 640; do
      { local seed_rc=0
        ATTACK_WEIGHTS=with-rollback ATTACK_SEED_START=$seed ATTACK_SEEDS=1 ATTACK_OPS=40 nice -n 19 \
          bash "$cap" 4G "$(bin_of attack_r2_random_histories)" --nocapture > "$log/b12-$index.log" 2>&1 || seed_rc=$?
        echo "$seed_rc" > "$log/b12-$index.rc"; } &
      index=$((index + 1))
    done
    wait
    [[ "$(ls "$log"/b12-*.rc | wc -l)" -eq 4 ]] || { echo "name=rerun_model_fix 作废：rc 文件数对不上"; return 1; }
    {
      echo "== 原样模型（batch 9，小盘 384 槽，r2-mix，种子下标 3000–3495）"
      for index in $(seq 0 30); do grep -h '^ATTACK_SUMMARY' "$log/b9-$index.log"; done
      for index in $(seq 0 30); do grep -h '^ATTACK_PROBLEM' "$log/b9-$index.log" || true; done | cut -c1-700
      echo "== 副本上改了模型回退那一步（batch 11，同一批种子）"
      for index in $(seq 0 30); do grep -h '^ATTACK_SUMMARY' "$log/b11-$index.log"; done
      for index in $(seq 0 30); do grep -h '^ATTACK_PROBLEM' "$log/b11-$index.log" || true; done | cut -c1-700
      echo "== 副本上改了模型，4 GiB 上打中过的四个种子（with-rollback）"
      for index in 0 1 2 3; do grep -h '^ATTACK_SUMMARY\|^ATTACK_PROBLEM' "$log/b12-$index.log" || true; done | cut -c1-400
    } > "$out/model-rollback-relocations-before-and-after.txt"
    compared+=(model-rollback-relocations-before-and-after.txt)
  fi
  local name same
  for name in "${compared[@]}"; do
    if cmp -s "$out/$name" "$MODEL/out/$name"; then same=true; else same=false; fi
    echo "name=rerun_compare file=$name identical=$same"
    [[ $same == true ]] && RERUN_SAME=$((${RERUN_SAME:-0} + 1)) || RERUN_DIFFER=$((${RERUN_DIFFER:-0} + 1))
  done
  echo "name=rerun_total same=${RERUN_SAME:-0} differ=${RERUN_DIFFER:-0}"
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/scripts/hosts/run-with-memory-cap.sh"
  echo 冻结副本里的文件 > "$scratch/frozen/crates/marker.txt"
  echo 编译产物 > "$scratch/frozen/crates/target/junk"
  copy="$scratch/elsewhere/r1-opus-model/tools/rerun.sh"
  cp "${BASH_SOURCE[0]}" "$copy"

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1") \
      > "$scratch/run1.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 0 ]] || ! grep -qF "name=rerun_roots repo=$scratch/fake-repo frozen=$scratch/frozen model=$scratch/elsewhere/r1-opus-model " "$scratch/run1.out" \
      || [[ ! -f "$scratch/work1/tree/crates/marker.txt" || -e "$scratch/work1/tree/crates/target" ]]; then
    failures+=("拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 $rc：$(tail -n 4 "$scratch/run1.out")")
  fi

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy") > "$scratch/run2.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "用法：rerun.sh --repo" "$scratch/run2.out"; then
    failures+=("不传参应当退 2 并列出用法，实际退 $rc：$(tail -n 4 "$scratch/run2.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --work "$scratch/work3" > "$scratch/run3.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "缺 --frozen" "$scratch/run3.out"; then
    failures+=("缺 --frozen 应当退 2 并点名缺它，实际退 $rc：$(tail -n 4 "$scratch/run3.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/cwd" --frozen "$scratch/frozen" --work "$scratch/work4" > "$scratch/run4.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run4.out"; then
    failures+=("--repo 指到不是仓的目录应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run4.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1" > "$scratch/run5.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "已存在且非空" "$scratch/run5.out"; then
    failures+=("--work 指到已有输出的目录应当退 2、不在旧输出上接着写，实际退 $rc：$(tail -n 4 "$scratch/run5.out")")
  fi

  if ((${#failures[@]})); then
    local failure
    for failure in "${failures[@]}"; do
      echo "  ✗ 自检：$failure"   # gate-lint:detail
    done
    echo "    → 怎么办：看 parse_arguments() 与 rerun_steps()；RERUN_TEMPLATE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--work 非空都退 2 并说明"
  return 0
}

if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
