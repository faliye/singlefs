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
copy_tree() {   # copy_tree <目的地>：只拷要编的
  mkdir -p "$1/.claude"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$1/"
  cp "$FROZEN/.claude/heavy-thread-default" "$1/.claude/"
  cp "$MODEL/tests/i718_oldslot_attack_probe.rs" "$1/crates/singlefs-harness/tests/"
}

rerun_steps() {
  local tree="$WORK/tree" patched="$WORK/tree-investigator-patch" probe c
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  copy_tree "$tree"
  echo "name=walk_rs_before_the_switch sha256=$(sha256sum "$tree/crates/singlefs-checker/src/walk.rs" | cut -d" " -f1) judged=e30ca36bc489a5c9a1b97ae87f6e51501d93c67504289f2180548078f44ecbc3"
  python3 "$MODEL/tools/walk-candidate-switch.py" "$tree/crates/singlefs-checker/src/walk.rs"
  python3 "$MODEL/tools/walk-candidate-switch-ding.py" "$tree/crates/singlefs-checker/src/walk.rs"
  copy_tree "$patched"
  (cd "$patched" && patch -p1 < "$MODEL/tools/investigator-prototype-i718-next-slot-by-its-old-content.patch")
  python3 "$MODEL/tools/walk-helpers-only-for-the-investigator-patch.py" "$patched/crates/singlefs-checker/src/walk.rs"
  cd "$tree"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 i718_oldslot_attack_probe; 范围=1 个测试目标; 为什么=复跑攻方腿的六格' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "${RERUN_THREADS:-31}" cargo test -p singlefs-harness --test i718_oldslot_attack_probe -- --test-threads 1 --nocapture cell_ > "$WORK/cells.out" 2>&1
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 i718_oldslot_attack_probe; 范围=1 个测试目标; 为什么=复跑攻方腿的一圈扫描' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "${RERUN_THREADS:-31}" cargo test -p singlefs-harness --test i718_oldslot_attack_probe -- --test-threads 1 --nocapture scan_the_base_length > "$WORK/scan.out" 2>&1
  for c in today jia yi yi_after_first_lap jia_any_lap jia_any_unreadable ding_jia ding_yi; do
    I718_CANDIDATE="$c" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot; 范围=1 个测试目标; 为什么=复跑攻方腿各候选下的 I-7.18 回归' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "${RERUN_THREADS:-31}" cargo test -p singlefs-harness --test root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot > "$WORK/regress-root-record-$c.out" 2>&1 || true
    I718_CANDIDATE="$c" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 checker_known_bad_images; 范围=1 个测试目标; 为什么=复跑攻方腿各候选下环健康坏镜像那一条' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "${RERUN_THREADS:-31}" cargo test -p singlefs-harness --test checker_known_bad_images -- --exact the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs > "$WORK/regress-known-bad-$c.out" 2>&1 || true
  done
  for c in today jia yi; do
    I718_CANDIDATE="$c" SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker 的库单测 --lib; 范围=1 个测试目标; 为什么=复跑攻方腿各候选下的池级 checker 库单测' CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "${RERUN_THREADS:-31}" cargo test -p singlefs-checker --lib > "$WORK/checker-lib-$c.out" 2>&1 || true
  done
  cd "$patched"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 i718_oldslot_attack_probe; 范围=1 个测试目标; 为什么=复跑调查员甲补丁原样在题面形与环洞形上的判定' CARGO_TARGET_DIR="$WORK/target-investigator-patch" nice -n 19 bash "$REPO_ROOT/scripts/hosts/capped.sh" "${RERUN_THREADS:-31}" cargo test -p singlefs-harness --test i718_oldslot_attack_probe -- --test-threads 1 --nocapture cell_a cell_b > "$WORK/investigator-patch.out" 2>&1
  python3 "$MODEL/tools/condense.py" "$WORK/cells.out" | sort > "$WORK/cells-condensed.sorted"
  sort "$MODEL/out/cells-condensed.txt" | diff - "$WORK/cells-condensed.sorted" > "$WORK/cells.diff" && echo "name=cells_condensed same_as=out/cells-condensed.txt lines=$(wc -l < "$WORK/cells-condensed.sorted")" || echo "name=cells_condensed DIFFERS see=$WORK/cells.diff"
  grep -o "scan cell=.*" "$WORK/scan.out" | sort > "$WORK/scan.sorted"
  sort "$MODEL/out/scan-one-revolution.txt" | diff - "$WORK/scan.sorted" > "$WORK/scan.diff" && echo "name=scan same_as=out/scan-one-revolution.txt lines=$(wc -l < "$WORK/scan.sorted")" || echo "name=scan DIFFERS see=$WORK/scan.diff"
  python3 "$MODEL/tools/condense.py" "$WORK/investigator-patch.out" | sort > "$WORK/investigator.sorted"
  sort "$MODEL/out/investigator-patch-cells-a-b-condensed.txt" | diff - "$WORK/investigator.sorted" > "$WORK/investigator.diff" && echo "name=investigator_patch same_as=out/investigator-patch-cells-a-b-condensed.txt" || echo "name=investigator_patch DIFFERS see=$WORK/investigator.diff"
  for c in today jia yi yi_after_first_lap jia_any_lap jia_any_unreadable ding_jia ding_yi; do
    echo "root_record_carries candidate=$c $(grep "test result" "$WORK/regress-root-record-$c.out")"
    echo "known_bad_ring_health candidate=$c $(grep "test result" "$WORK/regress-known-bad-$c.out") $(grep -E "^  left:" "$WORK/regress-known-bad-$c.out" | tr -s " ")"
  done > "$WORK/regressions.txt"
  for c in today jia yi; do echo "checker_lib candidate=$c $(grep "test result" "$WORK/checker-lib-$c.out")"; done >> "$WORK/regressions.txt"
  sed -E "s/finished in [0-9.]+s//" "$MODEL/out/regressions.txt" | diff - <(sed -E "s/finished in [0-9.]+s//" "$WORK/regressions.txt") > "$WORK/regressions.diff" && echo "name=regressions same_as=out/regressions.txt" || echo "name=regressions DIFFERS see=$WORK/regressions.diff"
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
