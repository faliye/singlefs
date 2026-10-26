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
  # 1) 冻结副本里要编的部分拷进草稿目录（不拷 target），探针文件装进 harness 的 tests/：
  #    探针文件 = 冻结副本里同名用例文件的辅助函数（逐字）+ 模型目录 probes/ 里的探针尾巴。
  mkdir -p "$WORK/tree"
  rsync -a --exclude target --exclude .git "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$WORK/tree/"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$WORK/tree/"
  # harness 用 include_str! 读 ../../.claude/heavy-thread-default：冻结副本没带，从仓根拷（和 .cargo 一起，不拷别的）。
  mkdir -p "$WORK/tree/.claude"
  cp "$REPO_ROOT/.claude/heavy-thread-default" "$WORK/tree/.claude/heavy-thread-default"
  rsync -a "$REPO_ROOT/.cargo" "$WORK/tree/"
  # 编译目录里已有的产物可能是别的树编的（cargo 的单元哈希不含绝对路径，按源文件时间判新旧）：每棵树开跑前把源文件时间碰新，逼它重编，
  # 不拿别的树的二进制当这一棵的。冻结副本的 rsync -a 保留了旧时间，不碰就会认成“没改”。
  find "$WORK/tree/crates" -type f -exec touch {} +
  local tests="$WORK/tree/crates/singlefs-harness/tests"
  { sed -n '1,287p' "$FROZEN/crates/singlefs-harness/tests/whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs"; cat "$MODEL/probes/tag_zero_probe_tail.rs"; } \
    > "$tests/m4_r2_sonnet_tag_zero_probe.rs"
  { sed -n '1,115p' "$FROZEN/crates/singlefs-harness/tests/release_checksum_quarantine_mark_written_to_disk_and_cleared_by_the_rollback.rs"; cat "$MODEL/probes/quarantine_probe_tail.rs"; } \
    > "$tests/m4_r2_sonnet_quarantine_probe.rs"
  sha256sum "$tests/m4_r2_sonnet_tag_zero_probe.rs" "$tests/m4_r2_sonnet_quarantine_probe.rs" | sed "s#$WORK/tree/##"
  mkdir -p "$WORK/out"
  # 编译目录默认在草稿目录里；RERUN_TARGET_DIR 给了就用它（分几趟热编译时用，跑完照样要删；上面每棵树都碰过源文件时间，共用也不会拿别的树的二进制）。
  local target="${RERUN_TARGET_DIR:-$WORK/target}"
  cd "$WORK/tree"
  local test_target target_rc lib_rc filter
  export M4R2_DUMP_DIR="$WORK/dump"
  mkdir -p "$M4R2_DUMP_DIR"
  # 2) 探针（打 PROBE 行）与冻结副本里已有的相关测试目标，逐个经线程上限与内存包装跑（整条脚本外面已经包了内存上限）。
  for test_target in m4_r2_sonnet_tag_zero_probe m4_r2_sonnet_quarantine_probe \
      whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session \
      whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader \
      release_checksum_quarantine \
      release_checksum_quarantine_mark_written_to_disk_and_cleared_by_the_rollback \
      checker_quarantine_mark_agrees_with_the_references \
      rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references; do
    SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 $test_target; 范围=1 个测试目标; 为什么=辩方腿复核代码轮第二轮的标签 0 写者与隔离记号" \
      CARGO_TARGET_DIR="$target" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
      cargo test --offline -p singlefs-harness --test "$test_target" -- --nocapture --test-threads=4 \
      > "$WORK/out/$test_target.txt" 2>&1 && target_rc=0 || target_rc=$?
    echo "name=target_done target=$test_target rc=$target_rc"
  done
  # 3) 核心层单测：隔离记号、分配记录解码、截断计划、测试临时目录守卫。
  for filter in allocator::tests allocation_record_tree::tests truncate::tests allocation_record_pool_geometry_tests block_device::tests; do
    SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-core 的 lib 测试目标按名字过滤 $filter; 范围=1 个测试目标; 为什么=辩方腿复核代码轮第二轮的隔离记号、截断计划与测试临时目录守卫" \
      CARGO_TARGET_DIR="$target" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
      cargo test --offline -p singlefs-core --lib "$filter" -- --test-threads=4 \
      > "$WORK/out/core_lib_${filter//:/_}.txt" 2>&1 && lib_rc=0 || lib_rc=$?
    echo "name=core_lib_done filter=$filter rc=$lib_rc"
  done
  # 4) 不认隔离记号的旧读者：HEAD 导出的代码（没有 0x4000 的读法）读第 2 步倒出来的两份镜像（带记号的、没隔离的对照）。
  local old="$WORK/old"
  mkdir -p "$old/.claude"
  echo "name=old_reader_code head=$(git -C "$REPO_ROOT" rev-parse HEAD)"
  git -C "$REPO_ROOT" archive HEAD crates Cargo.toml Cargo.lock | tar -x -C "$old"
  cp "$REPO_ROOT/.claude/heavy-thread-default" "$old/.claude/heavy-thread-default"
  rsync -a "$REPO_ROOT/.cargo" "$old/"
  cp "$MODEL/probes/old_reader_probe.rs" "$old/crates/singlefs-harness/tests/m4_r2_sonnet_old_reader_probe.rs"
  find "$old/crates" -type f -exec touch {} +   # git archive 给的是提交时间，比编译目录里的产物旧，同上要碰新
  cd "$old"
  SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 m4_r2_sonnet_old_reader_probe（HEAD 导出的旧代码）; 范围=1 个测试目标; 为什么=辩方腿核不认隔离记号的旧读者读带记号的池在哪一层拒" \
    CARGO_TARGET_DIR="$target" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
    cargo test --offline -p singlefs-harness --test m4_r2_sonnet_old_reader_probe -- --nocapture --test-threads=1 \
    > "$WORK/out/m4_r2_sonnet_old_reader_probe.txt" 2>&1 && target_rc=0 || target_rc=$?
  echo "name=target_done target=m4_r2_sonnet_old_reader_probe rc=$target_rc"
  # 5) 辩方腿的对照：把 of_a_file_without_data_units 改成 size 0 也返回 Some(NoDataUnit)（只改草稿目录里的副本，一处），
  #    再跑标签 0 探针与三份标签 0 用例文件，看截到 0 之后重写上段叶那两条读不读得回、遗留条目有没有了。
  local variant="$WORK/variant"
  mkdir -p "$variant"
  rsync -a --exclude target "$WORK/tree/" "$variant/"
  python3 - "$variant/crates/singlefs-core/src/extent_tree.rs" <<'PY'
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = """        if size_in_bytes > 0 {
            Some(ExtentUpperLeafTarget::NoDataUnit)
        } else {
            None
        }"""
new = """        let _ = size_in_bytes;
        Some(ExtentUpperLeafTarget::NoDataUnit)"""
assert text.count(old) == 1, "旧串恰好一次"
open(path, "w", encoding="utf-8").write(text.replace(old, new))
PY
  find "$variant/crates" -type f -exec touch {} +
  cd "$variant"
  for test_target in m4_r2_sonnet_tag_zero_probe whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session \
      whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader; do
    SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 $test_target（对照副本：size 0 也写标签 0）; 范围=1 个测试目标; 为什么=辩方腿核截到 0 之后重写上段叶读不回是不是这一处的判法造成" \
      CARGO_TARGET_DIR="$target" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
      cargo test --offline -p singlefs-harness --test "$test_target" -- --nocapture --test-threads=4 \
      > "$WORK/out/variant_$test_target.txt" 2>&1 && target_rc=0 || target_rc=$?
    echo "name=variant_target_done target=$test_target rc=$target_rc"
  done
  grep -h '^PROBE ' "$WORK/out/variant_m4_r2_sonnet_tag_zero_probe.txt" | sed 's/^PROBE /PROBE variant=size_zero_also_tag0 /' | sort
  cd "$WORK"
  grep -h '^PROBE ' "$WORK"/out/m4_r2_sonnet_*_probe.txt | sort
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
