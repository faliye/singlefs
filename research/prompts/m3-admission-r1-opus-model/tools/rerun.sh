#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。（攻方腿 m3-admission-r1-opus 拷的这一份）
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
rerun_steps() {
  # 攻方腿 m3-admission-r1-opus 的探针：拷冻结副本、把 $MODEL/probes 下的四个测试文件与共用模块放进 singlefs-harness/tests、
  # 编 release、逐步跑（每步一个测试目标与一组环境变量），只留从 PROBE / DF / FP / FPB / GEO / CAP / QD / ==== 起的那一截行，与 $MODEL/out/ 里同名文件逐行比。
  # 只跑其中几步：RERUN_STEPS=名字,名字（不设就全跑；单步挂钟见 $MODEL/out/README.txt）。
  local steps="${RERUN_STEPS:-held_small,held_wide,df_small_equal,df_small_unequal,df_wide_big,df_wide_long,fp_small,fp_wide,fp_wide_single,fp_which,geometry,qd}"
  # RERUN_TREE / RERUN_TARGET：腿自己分步跑时指到已经拷好探针的副本与它的编译目录（每步一个新的 --work 只装 out/）；判决复跑不设，从冻结副本现拷现编。
  local tree="${RERUN_TREE:-$WORK/tree}"
  mkdir -p "$WORK/out"
  if [[ -z "${RERUN_TREE:-}" ]]; then
    mkdir -p "$tree"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$tree/"
  fi
  if [[ ! -d "$MODEL/probes" ]]; then   # 只拷了这一份脚本（模板自证那一格）：没有探针可跑，只交回四个路径
    echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK probes=absent"
    return 0
  fi
  if [[ -z "${RERUN_TREE:-}" ]]; then
    cp -r "$MODEL/probes/opus_probe" "$tree/crates/singlefs-harness/tests/opus_probe"
    cp "$MODEL"/probes/opus_probe_*.rs "$tree/crates/singlefs-harness/tests/"
    [[ -d "$tree/.cargo" ]] || { [[ -d "$REPO_ROOT/.cargo" ]] && cp -r "$REPO_ROOT/.cargo" "$tree/.cargo"; }
  else
    local probe
    for probe in "$MODEL"/probes/opus_probe_*.rs "$MODEL"/probes/opus_probe/common.rs; do
      cmp -s "$probe" "$tree/crates/singlefs-harness/tests/${probe#"$MODEL"/probes/}" \
        || { echo "RERUN_TREE 里的探针与模型目录里的不同：$probe" >&2; return 3; }
    done
  fi
  local capped="$REPO_ROOT/scripts/hosts/capped.sh"
  local threads="${RERUN_THREADS:-15}"
  export CARGO_TARGET_DIR="${RERUN_TARGET:-$WORK/target}"
  (cd "$tree" && nice -n 19 bash "$capped" "$threads" cargo test --release -p singlefs-harness \
      --test opus_probe_held_slots --test opus_probe_df_candidates --test opus_probe_fixed_point --test opus_probe_geometry_bounds \
      --test opus_probe_quarantine_divergence --no-run) \
      > "$WORK/out/build.log" 2>&1
  run_step() {   # run_step <步名（RERUN_STEPS 认的）> <输出名> <测试目标> <测试名过滤或空> <环境变量…>
    local group="$1" name="$2" target="$3" filter="$4"; shift 4
    case ",$steps," in *",$group,"*) ;; *) return 0 ;; esac
    local raw="$WORK/out/$name.raw.log"
    (cd "$tree" && env "$@" nice -n 19 bash "$capped" "$threads" cargo test --release -p singlefs-harness --test "$target" \
        ${filter:+"$filter"} -- --nocapture --test-threads 1) > "$raw" 2>&1
    grep -oE '(PROBE|DF|FPB|FP|GEO|CAP|QD|====) .*' "$raw" > "$WORK/out/$name.lines" || true
    if [[ -f "$MODEL/out/$name.lines" ]]; then
      if diff -q "$MODEL/out/$name.lines" "$WORK/out/$name.lines" > /dev/null; then
        echo "name=rerun_step step=$group out=$name lines=$(wc -l < "$WORK/out/$name.lines") same_as_model_out=yes"
      else
        echo "name=rerun_step step=$group out=$name lines=$(wc -l < "$WORK/out/$name.lines") same_as_model_out=NO diff=$(diff "$MODEL/out/$name.lines" "$WORK/out/$name.lines" | wc -l)"
      fi
    else
      echo "name=rerun_step step=$group out=$name lines=$(wc -l < "$WORK/out/$name.lines") same_as_model_out=no_model_copy"
    fi
  }
  run_step held_small held_small opus_probe_held_slots opus_probe_a3_held_slots_after_a_raise_failing_midway OPUS_PROBE_UNITS=4,8,16,32,48
  run_step held_wide held_wide opus_probe_held_slots opus_probe_a3_holds_exceeding_the_reserves_on_a_wider_pool \
      OPUS_PROBE_UNIT_AREA_SLOTS=16384 OPUS_PROBE_JOURNAL_RING_BYTES=67108864 OPUS_PROBE_WIDE_UNITS=5000 OPUS_PROBE_SMALL_OVERWRITES=4,5,6
  run_step df_small_equal df_small_equal opus_probe_df_candidates "" OPUS_DF_UNIT_AREA_SLOTS=1391,1391
  run_step df_small_unequal df_small_unequal opus_probe_df_candidates "" OPUS_DF_UNIT_AREA_SLOTS=1391,2031
  run_step df_wide_big df_wide_big opus_probe_df_candidates opus_probe_df_history_big_then_small \
      OPUS_DF_UNIT_AREA_SLOTS=16384,16384 OPUS_DF_JOURNAL_RING_BYTES=67108864 OPUS_DF_BIG=3000 OPUS_DF_SMALL_STEPS=4
  run_step df_wide_long df_wide_long opus_probe_df_candidates opus_probe_df_history_long_mount \
      OPUS_DF_UNIT_AREA_SLOTS=16384,16384 OPUS_DF_JOURNAL_RING_BYTES=67108864 OPUS_DF_ROUNDS=600 OPUS_DF_EVERY=300 OPUS_DF_LONG_SIZES=8,150,300,40,290,7
  run_step fp_small fp_small opus_probe_fixed_point opus_probe_c573_fixed_point_of_a_publish_with_ordinary_allocation_against_the_checkpoint_cost OPUS_FP_ROUNDS=40
  run_step fp_wide fp_wide opus_probe_fixed_point opus_probe_c573_fixed_point_of_a_publish_with_ordinary_allocation_against_the_checkpoint_cost \
      OPUS_FP_UNIT_AREA_SLOTS=16384,16384 OPUS_FP_JOURNAL_RING_BYTES=67108864 OPUS_FP_ROUNDS=60 OPUS_FP_SIZES=1,8,144,145,2,300,1,289,1500,3,433,1,2000,5,3000
  local size
  for size in 3500 4000 4500 5000 5400; do
    run_step fp_wide_single "fp_wide_single_$size" opus_probe_fixed_point opus_probe_c573_fixed_point_of_a_publish_with_ordinary_allocation_against_the_checkpoint_cost \
        OPUS_FP_UNIT_AREA_SLOTS=16384,16384 OPUS_FP_JOURNAL_RING_BYTES=67108864 OPUS_FP_ROUNDS=1 OPUS_FP_SIZES=$size
  done
  local smalls
  for smalls in 1 2; do
    run_step fp_which "fp_which_$smalls" opus_probe_fixed_point opus_probe_c573_which_judgment_refuses_near_the_landing_bound OPUS_FP_SMALLS=$smalls
  done
  run_step geometry geometry opus_probe_geometry_bounds "" RERUN_GEOMETRY=1
  run_step qd qd opus_probe_quarantine_divergence "" RERUN_QD=1
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
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
