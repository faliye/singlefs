#!/usr/bin/env bash
# m3-close-g4-rewrite-r3 攻方腿模型目录的复跑脚本：从 scripts/agents/three-way-rerun-template.sh 拷来，只改 rerun_steps 那一段（连同它调的三个函数与流表）。
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
  # 冻结副本是开工时的仓（crates/、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default）。打第二轮那份开关补丁（$MODEL/toggles.patch，
  # 与第二轮模型目录里那份逐字节相同），装进两个原型（第二轮原样的 r2-prototype/ 与这一轮的 src/），只 cargo build，直接执行二进制。
  # 第二轮的 18 条流用第二轮的二进制跑，与 $MODEL/out-r2/ 逐字节比；仓里还有第二轮模型目录的，另与它的 out/ 比（去掉老几版原型不打的
  # history=、o2_reading、judged_tally= 三种行）。这一轮的流用这一轮的二进制跑，与 $MODEL/out/ 逐字节比。最后跑手算那一份，与 $MODEL/out/ 比。
  # 每批 4 件并行，每件退出码写进自己的 .rc，等齐、数件数，对不上整批作废（退 3）。
  mkdir -p "$WORK/tree" "$WORK/out" "$WORK/out-r2"
  rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  (cd "$WORK/tree" && patch -p1 < "$MODEL/toggles.patch")
  mkdir -p "$WORK/tree/crates/g4-attack-prototype-r2" "$WORK/tree/crates/g4-attack-prototype-r3"
  cp -r "$MODEL/r2-prototype/Cargo.toml" "$MODEL/r2-prototype/src" "$WORK/tree/crates/g4-attack-prototype-r2/"
  cp -r "$MODEL/Cargo.toml" "$MODEL/src" "$WORK/tree/crates/g4-attack-prototype-r3/"
  sed -i 's#    "crates/singlefs-unit-graph",#    "crates/singlefs-unit-graph",\n    "crates/g4-attack-prototype-r2",\n    "crates/g4-attack-prototype-r3",#' "$WORK/tree/Cargo.toml"
  (cd "$WORK/tree" && CARGO_TARGET_DIR="$WORK/target" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo build --release --offline \
    -p g4-attack-prototype-r2 -p g4-attack-prototype-r3)
  local r2_old="$REPO_ROOT/research/prompts/m3-close-g4-rewrite-r2-opus-model/out"
  run_batch r2 "$WORK/target/release/g4-attack-prototype-r2" "$WORK/out-r2" \
    "remount|none|remount" "remount-SINGLEFS_PROTO_MUT_B1|SINGLEFS_PROTO_MUT_B1|remount" \
    "remount-SINGLEFS_PROTO_MUT_B1+SINGLEFS_PROTO_GATE_NEVER_PASSES|SINGLEFS_PROTO_MUT_B1,SINGLEFS_PROTO_GATE_NEVER_PASSES|remount" \
    "remount-SINGLEFS_PROTO_MUT_B2|SINGLEFS_PROTO_MUT_B2|remount" "zeros_mount|none|zeros_mount" \
    "cold_start_then_remount|none|cold_start_then_remount" \
    "second_crash_base_0|none|second_crash_base 0" "second_crash_base_1|none|second_crash_base 1" \
    "second_crash_base_2|none|second_crash_base 2" "second_crash_base_3|none|second_crash_base 3" \
    "second_crash_base_4|none|second_crash_base 4" "second_crash_base_5|none|second_crash_base 5" \
    "second_crash_base_6|none|second_crash_base 6" "second_crash_base_7|none|second_crash_base 7" \
    "hand_made_image_both_slots-none|none|hand_made_image_both_slots" \
    "hand_made_image_both_slots-SINGLEFS_PROTO_GATE_NEVER_PASSES|SINGLEFS_PROTO_GATE_NEVER_PASSES|hand_made_image_both_slots" \
    "hand_made_image_both_slots_and_a_unit-none|none|hand_made_image_both_slots_and_a_unit" \
    "hand_made_image_both_slots_and_a_unit-SINGLEFS_PROTO_GATE_NEVER_PASSES|SINGLEFS_PROTO_GATE_NEVER_PASSES|hand_made_image_both_slots_and_a_unit"
  compare_all "$WORK/out-r2" "$MODEL/out-r2" byte
  if [[ -d "$r2_old" ]]; then compare_all "$WORK/out-r2" "$r2_old" stripped; fi
  run_batch r3 "$WORK/target/release/g4-attack-prototype-r3" "$WORK/out" "${R3_FLOWS[@]}"
  compare_all "$WORK/out" "$MODEL/out" byte
  python3 -I "$MODEL/tools/independent_count.py" > "$WORK/out/independent_count.txt"
  if cmp -s "$WORK/out/independent_count.txt" "$MODEL/out/independent_count.txt"; then
    echo "name=rerun_compare tag=independent_count same=yes"
  else
    echo "name=rerun_compare tag=independent_count same=no"
  fi
}

# 这一轮原型的流：标签|开关逗号表或 none|参数（空格分开）。
R3_FLOWS=(
  "make_filesystem_stream|none|make_filesystem_stream"
  "rewrite_after_a_make_filesystem_crash_0|none|rewrite_after_a_make_filesystem_crash 0"
  "rewrite_after_a_make_filesystem_crash_1|none|rewrite_after_a_make_filesystem_crash 1"
  "hand_made_image_one_slot_0|none|hand_made_image_one_slot 0"
  "hand_made_image_one_slot_1|none|hand_made_image_one_slot 1"
  "second_crash_on_a_hand_made_dead_base|none|second_crash_on_a_hand_made_dead_base"
  "amf_remount|none|after_make_filesystem_ops amf_remount remount"
  "amf_remount-SINGLEFS_PROTO_MUT_B1|SINGLEFS_PROTO_MUT_B1|after_make_filesystem_ops amf_remount remount"
  "amf_remount-SINGLEFS_PROTO_MUT_B2|SINGLEFS_PROTO_MUT_B2|after_make_filesystem_ops amf_remount remount"
  "amf_remount_twice|none|after_make_filesystem_ops amf_remount_twice remount remount"
  "amf_remount_twice-SINGLEFS_PROTO_MUT_B1|SINGLEFS_PROTO_MUT_B1|after_make_filesystem_ops amf_remount_twice remount remount"
  "amf_remount_twice-SINGLEFS_PROTO_MUT_B2|SINGLEFS_PROTO_MUT_B2|after_make_filesystem_ops amf_remount_twice remount remount"
  "amf_remount_zeros_mount|none|after_make_filesystem_ops amf_remount_zeros_mount remount zeros_mount"
  "amf_zeros_mount|none|after_make_filesystem_ops amf_zeros_mount zeros_mount"
  "amf_remount_publish_without_units|none|after_make_filesystem_ops amf_remount_publish_without_units remount publish_without_units"
  "aff_remount_roll_back|none|after_first_file_ops aff_remount_roll_back remount roll_back"
  "aff_remount_twice|none|after_first_file_ops aff_remount_twice remount remount"
  "aff_remount_zeros_mount|none|after_first_file_ops aff_remount_zeros_mount remount zeros_mount"
  "aff_remount_twice-SINGLEFS_PROTO_MUT_B1|SINGLEFS_PROTO_MUT_B1|after_first_file_ops aff_remount_twice remount remount"
  "aff_remount_twice-SINGLEFS_PROTO_MUT_B2|SINGLEFS_PROTO_MUT_B2|after_first_file_ops aff_remount_twice remount remount"
)

# run_batch <批名> <二进制> <输出目录> <标签|开关|参数>…：每 4 件一批并行跑，每件退出码进 $WORK/<批名>-<批号>-<件号>.rc；等齐后数件数。
run_batch() {
  local batch_name="$1" binary="$2" out="$3"; shift 3
  local -a specs=("$@")
  local start=0 batch=0
  while ((start < ${#specs[@]})); do
    batch=$((batch + 1))
    rm -f "$WORK/$batch_name-$batch-"*.rc
    local piece=0 spec
    for spec in "${specs[@]:start:4}"; do
      piece=$((piece + 1))
      # set -e 下照收退出码：放进 if 的条件里，else 分支的 $? 就是 run_one 的退出码
      { if run_one "$binary" "$out" "$spec"; then echo 0; else echo "$?"; fi > "$WORK/$batch_name-$batch-$piece.rc"; } &
    done
    wait
    local found
    found=$(find "$WORK" -maxdepth 1 -name "$batch_name-$batch-*.rc" | wc -l)
    if ((found != piece)); then echo "name=rerun_batch batch=$batch_name-$batch pieces=$piece rc_files=$found void=yes"; exit 3; fi
    local index
    for ((index = 1; index <= piece; index++)); do
      echo "name=rerun_batch batch=$batch_name-$batch piece=$index rc=$(cat "$WORK/$batch_name-$batch-$index.rc")"
    done
    start=$((start + 4))
  done
}

# run_one <二进制> <输出目录> <标签|开关|参数>：库目录每条流一份新的（枚举入口自己建），判法单元指纹要 CARGO 指着编它的那个 cargo。
run_one() {
  local binary="$1" out="$2" tag toggles arguments
  IFS='|' read -r tag toggles arguments <<< "$3"
  local -a envs=()
  if [[ "$toggles" != none ]]; then local t; local -a list; IFS=, read -ra list <<< "$toggles"; for t in "${list[@]}"; do envs+=("$t=1"); done; fi
  # shellcheck disable=SC2086
  (cd "$WORK" && env "${envs[@]}" CARGO="$(rustup which cargo)" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY="$WORK/library-$tag" \
    bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 "$binary" $arguments) > "$out/$tag.txt" 2> "$out/$tag.stderr"
  local rc=$?
  grep '^name=' "$out/$tag.txt" | grep -v ' seconds=' > "$out/$tag.counts"
  rm -rf "$WORK/library-$tag"
  return $rc
}

# compare_all <这一趟的输出目录> <对照目录> <byte|stripped>：对照目录里有的每一份 .counts 都比（-tally、-o2 两份比同一条流）。
compare_all() {
  local now="$1" reference="$2" how="$3" file tag source
  for file in "$reference"/*.counts; do
    tag="$(basename "$file" .counts)"; source="${tag%-tally}"; source="${source%-o2}"
    if [[ ! -f "$now/$source.counts" ]]; then echo "name=rerun_compare how=$how tag=$tag missing=yes"; continue; fi
    if [[ "$how" == byte ]]; then
      if cmp -s "$file" "$now/$source.counts"; then echo "name=rerun_compare how=$how tag=$tag same=yes"; else echo "name=rerun_compare how=$how tag=$tag same=no"; fi
    else
      if diff -q <(grep -v ' history=\| o2_reading \| judged_tally=' "$file") <(grep -v ' history=\| o2_reading \| judged_tally=' "$now/$source.counts") > /dev/null; then
        echo "name=rerun_compare how=$how tag=$tag same=yes"
      else
        echo "name=rerun_compare how=$how tag=$tag same=no"
      fi
    fi
  done
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
