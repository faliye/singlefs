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
#       连带红的格：「✗ 自检：--work 指到已有输出的目录应当退 2、不在旧输出上接着写」
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

# ── 这一腿要跑的步骤（攻方腿 m3-close-b19-code-r3-opus）──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引。一棵树 $WORK/tree：冻结副本的 crates 等，加 $MODEL/tests/ 里四份探针
# （本腿 V1/V2 探针一份；V5 复核用的前两轮探针三份，原样拷自第一轮攻方、第二轮辩方的模型目录）。
# 每一步输出落 $WORK/out/<名>.raw，筛出 PROBE、test、test result、panicked 四类行（去掉耗时、线程号，按行排序）与 $MODEL/out/<名>.txt 逐行比，
# 差异打 DIFF、一致打 SAME；V5 那三步另与前两轮存下的那一份（$MODEL/earlier-rounds-out/）比，只打 SAME/DIFF、不算失败。
# 有一步 DIFF（与本腿 out/ 比）就退 1。cargo 命令逐条写字面，整条脚本由外面的内存包装跑。
filtered_lines_without_the_timing() {   # filtered_lines_without_the_timing <raw 文件>
  grep -E 'PROBE|^test |test result|panicked' "$1" | sed -E 's/finished in [0-9.]+s/finished in Ns/; s#\(/[^)]*\)##; s/ \([0-9]+\) panicked/ panicked/' | LC_ALL=C sort || true
}

DIFFS_AGAINST_THIS_LEG=0
compare_with_the_saved_output() {   # compare_with_the_saved_output <名>
  local name="$1"
  filtered_lines_without_the_timing "$WORK/out/$name.raw" > "$WORK/out/$name.txt"
  if diff -u "$MODEL/out/$name.txt" "$WORK/out/$name.txt" > "$WORK/out/$name.diff"; then
    echo "SAME name=$name lines=$(wc -l < "$WORK/out/$name.txt")"
  else
    echo "DIFF name=$name see=$WORK/out/$name.diff"
    DIFFS_AGAINST_THIS_LEG=$((DIFFS_AGAINST_THIS_LEG + 1))
  fi
}

# 两边都按行排序再比（前两轮存下的那一份是按跑的次序记的，这里一个测试一个线程跑，PROBE 行可能接在「test … 」后面，取 PROBE 起的那一截）。
compare_with_an_earlier_round() {   # compare_with_an_earlier_round <名> <前两轮存下的那一份的文件名> <比的是哪几类行：probe 或 all>
  local name="$1" earlier="$2" kind="$3"
  if [[ "$kind" == probe ]]; then
    grep -o 'PROBE .*' "$WORK/out/$name.raw" | LC_ALL=C sort > "$WORK/out/$name.probe-lines.txt" || true
  else
    cp "$WORK/out/$name.txt" "$WORK/out/$name.probe-lines.txt"
  fi
  if diff -u <(LC_ALL=C sort "$MODEL/earlier-rounds-out/$earlier") "$WORK/out/$name.probe-lines.txt" > "$WORK/out/$name.earlier.diff"; then
    echo "SAME_AS_EARLIER_ROUND name=$name earlier=$earlier"
  else
    echo "DIFF_FROM_EARLIER_ROUND name=$name earlier=$earlier changed_lines=$(grep -c '^[-+][^-+]' "$WORK/out/$name.earlier.diff")"
    grep '^[-+][^-+]' "$WORK/out/$name.earlier.diff"
  fi
}

rerun_steps() {
  mkdir -p "$WORK/out" "$WORK/tree/.claude"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/tree/"
  cp "$FROZEN/.claude/heavy-thread-default" "$WORK/tree/.claude/heavy-thread-default"
  cp "$MODEL"/tests/*.rs "$WORK/tree/crates/singlefs-harness/tests/"
  echo "name=tree mount_sha256=$(sha256sum "$WORK/tree/crates/singlefs-core/src/mount.rs" | cut -c1-16)"

  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 r3_attack_probe_slots_of_member_devices_in_the_rehearsal_ledgers; 范围=1 个测试目标; 为什么=复跑攻方第三轮 V1/V2 探针：补写盘之外的成员盘一槽读坏读错进预演两本账时的拒因' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r3_attack_probe_slots_of_member_devices_in_the_rehearsal_ledgers -- --nocapture --test-threads=1) > "$WORK/out/v1-member-slots.raw" 2>&1 || true
  compare_with_the_saved_output v1-member-slots

  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 attack_probe_rehearsal_reads_a_device_without_any_self_verified_system_configuration; 范围=1 个测试目标; 为什么=复跑攻方第三轮 V5：第一轮攻方探针五行输出在开工快照的树上复现不复现' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test attack_probe_rehearsal_reads_a_device_without_any_self_verified_system_configuration -- --nocapture --test-threads=1) > "$WORK/out/v5-r1-opus-probe.raw" 2>&1 || true
  compare_with_the_saved_output v5-r1-opus-probe
  compare_with_an_earlier_round v5-r1-opus-probe r1-opus-probe-output-with-the-reorder.txt probe

  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 r2_probe_u2_zeroed_slots_with_read_faults_in_the_rehearsal; 范围=1 个测试目标; 为什么=复跑攻方第三轮 V5：第二轮辩方 U2 探针的拒因偏移与长度在开工快照的树上怎么变' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r2_probe_u2_zeroed_slots_with_read_faults_in_the_rehearsal -- --nocapture --test-threads=1) > "$WORK/out/v5-r2-u2.raw" 2>&1 || true
  compare_with_the_saved_output v5-r2-u2
  compare_with_an_earlier_round v5-r2-u2 r2-sonnet-new-u2.txt all

  (cd "$WORK/tree" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 r2_probe_u6_instance_table_sweep_threshold_by_mounting_many_times; 范围=1 个测试目标; 为什么=复跑攻方第三轮 V5：第二轮辩方 U6 探针 420 次挂载清扫第一次触发的行数' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 cargo test --offline -p singlefs-harness --test r2_probe_u6_instance_table_sweep_threshold_by_mounting_many_times -- --nocapture --test-threads=1) > "$WORK/out/v5-r2-u6.raw" 2>&1 || true
  compare_with_the_saved_output v5-r2-u6
  compare_with_an_earlier_round v5-r2-u6 r2-sonnet-new-u6.txt all

  rm -rf "$WORK/target" "$WORK/tree"
  echo "name=diffs_against_this_leg count=$DIFFS_AGAINST_THIS_LEG"
  ((DIFFS_AGAINST_THIS_LEG == 0))
}


# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  export RERUN_TEMPLATE_SELFTEST_MARK="rerun-template-selftest"   # 自证里拷出去起的子进程（环境原样继承）靠它认：弄坏开关只在自证里认
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

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（RERUN_TEMPLATE_SELFTEST_MARK 对得上）而变量非空，参数解析之前就退 2（不管值是什么）。
if [[ -n "${RERUN_TEMPLATE_BREAK:-}" && ! ( "$#" -eq 1 && "${1:-}" == --selftest ) && "${RERUN_TEMPLATE_SELFTEST_MARK:-}" != rerun-template-selftest ]]; then
  echo "  ✗ RERUN_TEMPLATE_BREAK=${RERUN_TEMPLATE_BREAK} 是弄坏开关，只在自证（bash <这份脚本> --selftest）里认，平常跑设了就拒，这一趟什么都没做" >&2
  echo "     → 怎么办：去掉 RERUN_TEMPLATE_BREAK 再跑；要证红就跑 RERUN_TEMPLATE_BREAK=<项> bash scripts/agents/three-way-rerun-template.sh --selftest" >&2
  exit 2
fi
if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
