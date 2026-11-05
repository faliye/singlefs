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

# ── 这一腿要跑的步骤：腿把这一段换成自己的 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引，不写死、不从 $0 推；cargo 经 "$REPO_ROOT/scripts/hosts/capped.sh" 起，
# 输出写进 $WORK，最后打出与 $MODEL/out/ 里哪一份逐行比。模板自带的这一段只拷冻结副本、打出四个路径，自证跑的就是它。
rerun_steps() {
  local tree="$WORK/tree" outputs="$WORK/outputs" rc
  mkdir -p "$tree/.claude" "$outputs"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$tree/"
  if [[ -d "$FROZEN/.cargo" ]]; then rsync -a "$FROZEN/.cargo" "$tree/"; fi
  cp "$FROZEN/.claude/heavy-thread-default" "$tree/.claude/"
  find "$tree/crates" -type f -exec touch {} +
  bash "$MODEL/tools/apply-probes.sh" "$MODEL" "$tree"
  export CARGO_TARGET_DIR="${RERUN_CARGO_TARGET_DIR:-$WORK/target}"
  cd "$tree"
  # 一、三条探针（T1-1、T1-2、T1-3、T3-1）
  if SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试目标里攻方加的三条探针; 范围=1 个测试目标; 为什么=复跑攻方腿的按段重交、没点名的卡拒挂载、一批里最小一片随映射变三件反例' bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 cargo test -p singlefs-background-gpu --lib attack_probe -- --nocapture > "$outputs/probes-lib.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=probes-lib rc=$rc"
  # 二、T6-1 泄漏边界改成全部卡
  cp crates/singlefs-background-gpu/src/background_gpu.rs "$WORK/background_gpu.rs.before-mutation"
  python3 "$MODEL/tools/mutate-once.py" "$tree" crates/singlefs-background-gpu/src/background_gpu.rs "$MODEL/mutations/drop-leaks-every-card.original" "$MODEL/mutations/drop-leaks-every-card.replacement"
  if SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试目标（泄漏边界改成全部卡的变异副本）; 范围=1 个测试目标; 为什么=复跑攻方腿 T6-1 那一件反例，只跑那一个测试目标看它红不红、与报告里抄的输出比' bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 cargo test -p singlefs-background-gpu --lib > "$outputs/mutation-drop-leaks-every-card.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=mutation-drop-leaks-every-card rc=$rc"
  cp "$WORK/background_gpu.rs.before-mutation" crates/singlefs-background-gpu/src/background_gpu.rs
  # 三、T6-4 重交次数改 2
  cp crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs "$WORK/waiting_for_the_gpu.rs.before-mutation"
  python3 "$MODEL/tools/mutate-once.py" "$tree" crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs "$MODEL/mutations/resubmissions-two.original" "$MODEL/mutations/resubmissions-two.replacement"
  if SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试目标（重交次数改 2 的变异副本）; 范围=1 个测试目标; 为什么=复跑攻方腿 T6-4 那一件反例，只跑那一个测试目标看它红不红、与报告里抄的输出比' bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 cargo test -p singlefs-background-gpu --lib > "$outputs/mutation-resubmissions-two.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=mutation-resubmissions-two rc=$rc"
  cp "$FROZEN/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs" crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
  # 四、T6-3 切片合并少带最后一片（那一次 waiting_for_the_gpu.rs 不带探针，同原跑）
  python3 "$MODEL/tools/mutate-once.py" "$tree" crates/singlefs-background-gpu/src/checksum_kernels.rs "$MODEL/mutations/combine-drops-the-last-piece.original" "$MODEL/mutations/combine-drops-the-last-piece.replacement"
  if SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试目标（切片合并少带最后一片的变异副本）; 范围=1 个测试目标; 为什么=复跑攻方腿 T6-3 那一件反例，只跑那一个测试目标看它红不红、与报告里抄的输出比' bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 cargo test -p singlefs-background-gpu --lib > "$outputs/mutation-combine-drops-the-last-piece.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=mutation-combine-drops-the-last-piece rc=$rc"
  cp "$FROZEN/crates/singlefs-background-gpu/src/checksum_kernels.rs" crates/singlefs-background-gpu/src/checksum_kernels.rs
  # 五、T4-1 删除续做那一支
  { echo '#![allow(dead_code, unused_imports)]'; sed -n 14,281p "$FROZEN/crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs"; cat "$MODEL/probes/harness_deletion_continuation_test.rs"; } > crates/singlefs-harness/tests/attack_probe_deletion_continuation_while_stopped.rs
  if SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 attack_probe_deletion_continuation_while_stopped; 范围=1 个测试目标; 为什么=复跑攻方腿 T4-1 那一件反例，只跑那一个测试目标看它红不红、与报告里抄的输出比' bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 cargo test -p singlefs-harness --test attack_probe_deletion_continuation_while_stopped -- --nocapture > "$outputs/deletion-continuation.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=deletion-continuation rc=$rc"
  # 六、T6-2 补跑判定锁存
  python3 "$MODEL/tools/mutate-once.py" "$tree" crates/singlefs-core/src/background_compute.rs "$MODEL/mutations/compensation-latched-after-a-compensating-batch.original" "$MODEL/mutations/compensation-latched-after-a-compensating-batch.replacement"
  if SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 compaction_on_the_cpu_while_stopped_after_a_gpu_failure（补跑判定锁存的变异副本）; 范围=1 个测试目标; 为什么=复跑攻方腿 T6-2 那一件反例，只跑那一个测试目标看它红不红、与报告里抄的输出比' bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 cargo test -p singlefs-harness --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure > "$outputs/mutation-compensation-latched.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=mutation-compensation-latched rc=$rc"
  cp "$FROZEN/crates/singlefs-core/src/background_compute.rs" crates/singlefs-core/src/background_compute.rs
  # 比：每份输出的 ATTACK 行与 test result 行（排序后）与 $MODEL/outputs 里同名那一份逐行比
  local name differing=0
  for name in probes-lib mutation-drop-leaks-every-card mutation-resubmissions-two mutation-combine-drops-the-last-piece deletion-continuation mutation-compensation-latched; do
    if diff <(grep -E '^ATTACK|^test result' "$MODEL/outputs/$name.out" | sed -E 's/finished in [0-9.]+s//' | sort) <(grep -E '^ATTACK|^test result' "$outputs/$name.out" | sed -E 's/finished in [0-9.]+s//' | sort) > "$outputs/$name.diff"; then
      echo "name=compare-$name same=yes"
    else
      echo "name=compare-$name same=no"
      differing=$((differing + 1))
    fi
  done
  echo "name=compare-total differing=$differing"
  ((differing == 0))
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
