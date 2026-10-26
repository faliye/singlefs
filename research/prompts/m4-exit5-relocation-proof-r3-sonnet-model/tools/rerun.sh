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

# ── 这一腿要跑的步骤 ──
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引；cargo 与 verus 经 "$REPO_ROOT/scripts/hosts/capped.sh" 31 起（线程上限 31）。
# 每一步的输出去掉耗时与路径之后写进 $WORK/out/<名>.txt，与 $MODEL/out/<名>.txt 逐行比；对不上的件数就是退出码（0 = 全对上）。
# RERUN_STEPS=z1,z2,... 只跑点名的段（默认全跑，段名：z1 z2 z3 z4 z5）；RERUN_WRITE_REFERENCE=1 把这一趟的输出写成参照（只给腿自己建参照用）。
# 第二轮两条腿的模型目录当输入读（$REPO_ROOT/research/prompts/m4-exit5-relocation-proof-r2-{sonnet,opus}-model/），不改。
MISMATCHES=0
export PYTHONDONTWRITEBYTECODE=1
VERUS_DIR="$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux"
VERUS="$VERUS_DIR/verus"

capped() { bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 nice -n 19 "$@"; }
want() { [[ -z "${RERUN_STEPS:-}" || ",${RERUN_STEPS}," == *",$1,"* ]]; }

compare_with_reference() {   # compare_with_reference <名>
  if [[ -n "${RERUN_WRITE_REFERENCE:-}" ]]; then
    mkdir -p "$MODEL/out"; cp "$WORK/out/$1.txt" "$MODEL/out/$1.txt"
    echo "name=rerun_reference_written item=$1"; return
  fi
  if diff -u "$MODEL/out/$1.txt" "$WORK/out/$1.txt" > "$WORK/out/$1.diff"; then
    echo "name=rerun_compare item=$1 result=same"
  else
    echo "name=rerun_compare item=$1 result=DIFFERENT"; cat "$WORK/out/$1.diff"
    MISMATCHES=$((MISMATCHES + 1))
  fi
}

# verus_case <标签> <工作目录> <文件> [额外参数…]：直接起 Verus（--no-cheating，--rlimit 10），打一行稳定的结果
verus_case() {
  local label="$1" dir="$2" file="$3"; shift 3
  local raw="$WORK/raw-$label.txt" rc scratch="$WORK/vc/$label"
  mkdir -p "$scratch" && cp "$dir/$file" "$scratch/"
  ( cd "$scratch" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=直接起 Verus 验一个证明文件; 范围=1 个文件; 为什么=第三轮辩方腿复核改后协议与门禁判法' \
    capped "$VERUS" --no-cheating --crate-type=lib --edition=2021 --num-threads 31 --rlimit 10 "$@" "$file" > "$raw" 2>&1 )
  rc=$?
  echo "name=verus case=$label rc=$rc $(grep -E '^(verification results|error)' "$raw" | grep -v 'aborting' | sed -E 's#/[^ ]*/##g' | cut -c1-140 | sort -u | tr '\n' '|')"
}
# ---- Z1 ----
step_z1() {
  : > "$WORK/out/z1.txt"
  mkdir -p "$WORK/z1"
  local crate="$WORK/z1/crate" label
  mkdir -p "$crate" && rsync -a --exclude target "$FROZEN/crates/singlefs-replay-prefix" "$crate/"
  cp "$R2S/src/relocation_interval_proofs.rs" "$R2S/src/gc_ring_proofs.rs" "$MODEL/verus/gc_ring_runs.rs" "$MODEL/verus/z1_bridge_probe.rs" "$crate/singlefs-replay-prefix/src/"
  local lib="$crate/singlefs-replay-prefix/src/lib.rs"
  python3 -I - "$lib" <<'PY'
import sys
p = sys.argv[1]; t = open(p).read()
old = "pub mod gc_crash_proofs;\n"
assert t.count(old) == 1
open(p, "w").write(t.replace(old, old + "pub mod gc_ring_proofs;\npub mod relocation_interval_proofs;\n"))
PY
  for label in round-two-modules-only with-runs-and-bridge; do
    if [[ "$label" == with-runs-and-bridge ]]; then
      python3 -I - "$lib" <<'PY'
import sys
p = sys.argv[1]; t = open(p).read()
old = "pub mod relocation_interval_proofs;\n"
assert t.count(old) == 1
open(p, "w").write(t.replace(old, old + "pub mod gc_ring_runs;\npub mod z1_bridge_probe;\n"))
PY
    fi
    ( cd "$crate/singlefs-replay-prefix" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=直接起 Verus 验 replay-prefix 副本加第二轮与第三轮的证明; 范围=1 个 crate; 为什么=第三轮辩方腿复核改后协议' \
      capped "$VERUS" --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads 31 --rlimit 10 src/lib.rs > "$WORK/z1/crate-$label.raw" 2>&1 )
    echo "name=z1_crate label=$label rc=$? $(grep -E '^verification results' "$WORK/z1/crate-$label.raw")" >> "$WORK/out/z1.txt"
  done
  verus_case runs-protocol "$MODEL/verus" gc_ring_runs.rs >> "$WORK/out/z1.txt"
  verus_case original-protocol-probe "$MODEL/verus" z1_original_probe.rs >> "$WORK/out/z1.txt"
  mkdir -p "$WORK/z1/worlds"
  python3 -I - "$MODEL/verus/gc_ring_runs.rs" "$WORK/z1/worlds" <<'PY'
import sys
src, out = sys.argv[1:3]
t = open(src).read()
rows = [
 ("start_only", "forall|key: u64| mapping.contains_key(key) ==> !run_covers(#[trigger] mapping[key], slot)", "forall|key: u64| mapping.contains_key(key) ==> #[trigger] mapping[key].start != slot"),
 ("no_candidates", "    &&& forall|i: int| s.floor <= i < s.roots.len() ==> run_map_avoids(#[trigger] s.roots[i].mapping, slot)\n    &&& forall|i: int| 0 <= i < s.readers.len() ==> run_map_avoids(#[trigger] s.readers[i].mapping, slot)\n}", "    &&& forall|i: int| 0 <= i < s.readers.len() ==> run_map_avoids(#[trigger] s.readers[i].mapping, slot)\n}"),
 ("forced_to_zero", "RunStep::Write { slot, .. } | RunStep::Reclaim { slot } => run_slot_is_unreferenced(s, slot),", "RunStep::Write { slot, .. } => run_slot_is_unreferenced(s, slot),\n        RunStep::Reclaim { slot } => true,"),
]
for name, old, new in rows:
    assert t.count(old) == 1, (name, t.count(old))
    open(f"{out}/world_{name}.rs", "w").write(t.replace(old, new))
PY
  for label in start_only no_candidates forced_to_zero; do
    verus_case "world-$label" "$WORK/z1/worlds" "world_$label.rs" >> "$WORK/out/z1.txt"
  done
  capped python3 -I "$MODEL/tools/g7_window_shift.py" 5 >> "$WORK/out/z1.txt"
  capped python3 -I "$MODEL/tools/g7_named_roots.py" 5 >> "$WORK/out/z1.txt"
  capped python3 -I "$MODEL/tools/g7_named_roots.py" 9 rand:20000:7 >> "$WORK/out/z1.txt"
  compare_with_reference z1
}

# ---- Z2 ----
step_z2() {
  : > "$WORK/out/z2.txt"
  mkdir -p "$WORK/z2"
  local f
  for f in f1_feature_fork f1b_debug_assertions_fork f2_trait_impl_outside f3_unsafe f4_default_method f5_macro_outside g1_trait_spec g2_external_attr; do
    verus_case "$f" "$MODEL/verus" "z2_$f.rs" >> "$WORK/out/z2.txt"
  done
  verus_case f1_feature_fork-with-cfg-feature-wide "$MODEL/verus" z2_f1_feature_fork.rs --cfg 'feature="wide"' >> "$WORK/out/z2.txt"
  # verus --compile：Verus 自己编出来的可执行代码（verus_keep_ghost 为真）
  ( cd "$MODEL/verus" && SINGLEFS_RUN_SCOPE='种类=target; 跑什么=直接起 Verus 带 --compile 编一个探针; 范围=1 个文件; 为什么=第三轮辩方核 Verus 看到的可执行代码与产品构建' \
    capped "$VERUS" --no-cheating --edition=2021 --compile -o "$WORK/z2/f5_bin" z2_f5_main.rs > "$WORK/z2/f5-compile.raw" 2>&1 )
  echo "name=z2_verus_compile rc=$? $(grep -E '^verification results' "$WORK/z2/f5-compile.raw")" >> "$WORK/out/z2.txt"
  bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 2G "$WORK/z2/f5_bin" 2>&1 | grep '^name=' >> "$WORK/out/z2.txt"
  # 产品构建：cargo build（默认特性、debug）与 --release --features wide，各跑一遍
  cp -r "$MODEL/z2crate" "$WORK/z2/z2crate"; cp "$FROZEN/Cargo.lock" "$WORK/z2/z2crate/"
  local rc
  SINGLEFS_RUN_SCOPE='种类=build; 跑什么=cargo build 辩方腿的探针 crate; 范围=1 个 crate; 为什么=第三轮辩方核 Z2 写法在产品构建下的行为' \
    CARGO_TARGET_DIR="$WORK/z2/target" capped cargo build --offline --manifest-path "$WORK/z2/z2crate/Cargo.toml" > "$WORK/z2/build-debug.raw" 2>&1; rc=$?
  echo "name=z2_product_build profile=debug features=none rc=$rc" >> "$WORK/out/z2.txt"
  bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 2G "$WORK/z2/target/debug/z2crate" 2>&1 | grep '^name=' | sed 's/^name=product-run /name=z2_product_run profile=debug features=none /' >> "$WORK/out/z2.txt"
  SINGLEFS_RUN_SCOPE='种类=build; 跑什么=cargo build 辩方腿的探针 crate 的 release 与 wide 特性; 范围=1 个 crate; 为什么=第三轮辩方核 Z2' \
    CARGO_TARGET_DIR="$WORK/z2/target" capped cargo build --offline --release --features wide --manifest-path "$WORK/z2/z2crate/Cargo.toml" > "$WORK/z2/build-release.raw" 2>&1; rc=$?
  echo "name=z2_product_build profile=release features=wide rc=$rc" >> "$WORK/out/z2.txt"
  bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 2G "$WORK/z2/target/release/z2crate" 2>&1 | grep '^name=' | sed 's/^name=product-run /name=z2_product_run profile=release features=wide /' >> "$WORK/out/z2.txt"
  # 差分测试：写错的 binary_search 规格在有界域上判得出
  SINGLEFS_RUN_SCOPE='种类=build; 跑什么=rustc 编辩方腿的差分测试小程序; 范围=1 个文件; 为什么=第三轮辩方核差分测试能否判出写错的 std 规格' \
    nice -n 19 rustc -O --edition 2021 -o "$WORK/z2/std_spec_differential" "$MODEL/std-diff/std_spec_differential.rs" > "$WORK/z2/rustc.raw" 2>&1
  bash "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" 2G "$WORK/z2/std_spec_differential" 2>&1 | grep '^name=' >> "$WORK/out/z2.txt"
  echo "name=z2_gate_scan verus_keep_ghost_hits_in_verus_proof_check_py=$(grep -c 'verus_keep_ghost' "$REPO_ROOT/scripts/gate/verus-proof-check.py")" >> "$WORK/out/z2.txt"
  compare_with_reference z2
}

# ---- Z3 ----
closure_of() {   # closure_of <名> <入口>：在冻结副本上收一条闭包（lib_function_units.py closure）到 $WORK/z3/closure-<名>.txt
  nice -n 19 python3 -I "$REPO_ROOT/scripts/gate/lib_function_units.py" closure "$FROZEN" "$2" > "$WORK/z3/closure-$1.txt" 2> "$WORK/z3/closure-$1.err"
  echo "name=z3_closure entry=$1 rc=$? fn=$(awk -F'\t' '$1=="fn"' "$WORK/z3/closure-$1.txt" | wc -l) units=$(wc -l < "$WORK/z3/closure-$1.txt") refused=$(grep -c '^refused' "$WORK/z3/closure-$1.txt")"
}
step_z3() {
  : > "$WORK/out/z3.txt"
  mkdir -p "$WORK/z3"
  { closure_of publish 'singlefs_core::transaction::publish_relocation_batch'
    closure_of roll_back_by_a_forward_publish 'singlefs_core::mount::roll_back_by_a_forward_publish'
    closure_of recover 'singlefs_core::recovery::recover'
    closure_of run_compaction 'singlefs_core::mounted_session::<impl MountedSession>::run_compaction'
    closure_of run_compaction_in_the_background 'singlefs_core::mounted_session::<impl MountedSession>::run_compaction_in_the_background'
    closure_of discard_segments 'singlefs_core::mounted_session::discard_the_segments_reusable_as_a_whole'
    closure_of raise_rollback_floor 'singlefs_core::mount::raise_rollback_floor'
    closure_of raise_floor_to_ceiling 'singlefs_core::mount::raise_rollback_floor_to_the_admission_ceiling'
    closure_of sweep_end_of_mount 'singlefs_core::instance_table_sweep::sweep_the_instance_table_at_the_end_of_the_mount'
    closure_of sweep_while_mounted 'singlefs_core::instance_table_sweep::sweep_the_instance_table_while_mounted'
    closure_of mount_writable 'singlefs_core::mount::mount_writable'
    closure_of scan_region 'singlefs_core::region_scan::scan_region'; } >> "$WORK/out/z3.txt"
  nice -n 19 python3 -I "$REPO_ROOT/scripts/gate/lib_function_units.py" dump "$FROZEN" > "$WORK/z3/dump-frozen.txt" 2>/dev/null
  FROZEN="$FROZEN" python3 -I "$MODEL/tools/z3_membership.py" "$WORK/z3" >> "$WORK/out/z3.txt"
  FROZEN="$FROZEN" python3 -I "$MODEL/tools/z3_writers.py" "$WORK/z3" >> "$WORK/out/z3.txt"
  # 主工作区此刻的闭包（GPU 后台计算 trait 只在主工作区有）：发布、回退、整理（后台）、恢复四条的 fn 数与含 GPU 字样的单元数
  local e n en
  for e in publish:singlefs_core::transaction::publish_relocation_batch rollback:singlefs_core::mount::roll_back_by_a_forward_publish \
           "run_compaction_in_the_background:singlefs_core::mounted_session::<impl MountedSession>::run_compaction_in_the_background" recover:singlefs_core::recovery::recover; do
    n=${e%%:*}; en=${e#*:}
    nice -n 19 python3 -I "$REPO_ROOT/scripts/gate/lib_function_units.py" closure "$REPO_ROOT" "$en" > "$WORK/z3/main-$n.txt" 2> /dev/null
    echo "name=z3_main_tree_closure entry=$n fn=$(awk -F'\t' '$1=="fn"' "$WORK/z3/main-$n.txt" | wc -l) units_naming_a_gpu_item=$(grep -c 'BackgroundGpuCompute\|background_gpu' "$WORK/z3/main-$n.txt")" >> "$WORK/out/z3.txt.main"
  done
  : # 主工作区的数随主工作区变（别的会话在改），单独存，不进逐行比
  compare_with_reference z3
}

# ---- Z4 ----
step_z4() {
  : > "$WORK/out/z4.txt"
  local f
  for f in h1_proof_calls_exec h2_ghost_in_exec_branch h3_ghost_param_selects_exec h4_proof_block_assign_exec_var; do
    verus_case "$f" "$MODEL/verus" "z4_$f.rs" >> "$WORK/out/z4.txt"
  done
  # 原型：strip_verus.py 在 strip 文件头列的写法上，t1.rs 之类自证
  bash "$MODEL/tools/key_variants_stripped.sh" "$WORK" "$REPO_ROOT" "$FROZEN" "$MODEL/tools/strip_verus.py" 2>&1 | grep '^name=' >> "$WORK/out/z4.txt"
  bash "$MODEL/tools/key_lemma_edit.sh" "$WORK" "$REPO_ROOT" "$FROZEN" 2>&1 | grep '^name=' >> "$WORK/out/z4.txt"
  compare_with_reference z4
}

# ---- Z5 ----
step_z5() {
  : > "$WORK/out/z5.txt"
  mkdir -p "$WORK/z5/base/crates" "$WORK/z5/base/.claude" "$WORK/z3"
  [[ -s "$WORK/z3/closure-publish.txt" ]] || { closure_of publish 'singlefs_core::transaction::publish_relocation_batch' > /dev/null; closure_of roll_back_by_a_forward_publish 'singlefs_core::mount::roll_back_by_a_forward_publish' > /dev/null; }
  python3 -I "$MODEL/tools/z5_union.py" "$WORK/z3" "$FROZEN" >> "$WORK/out/z5.txt"
  rsync -a --exclude target "$FROZEN/crates/singlefs-format" "$FROZEN/crates/singlefs-core" "$FROZEN/crates/singlefs-replay-prefix" "$WORK/z5/base/crates/"
  cp "$FROZEN/Cargo.lock" "$WORK/z5/base/"; cp -r "$FROZEN/.cargo" "$WORK/z5/base/"; cp "$R2S/tools/workspace-Cargo.toml" "$WORK/z5/base/Cargo.toml"
  cp "$FROZEN/.claude/heavy-thread-default" "$WORK/z5/base/.claude/" 2>/dev/null || true
  SINGLEFS_RUN_SCOPE='种类=build; 跑什么=cargo rustc 出 core 的 MIR; 范围=1 个 crate; 为什么=第三轮辩方复现第二轮正推的 A 至 D 分类' \
    PATH="$VERUS_DIR:$PATH" CARGO_TARGET_DIR="$WORK/z5/build" capped cargo rustc --offline --manifest-path "$WORK/z5/base/Cargo.toml" -p singlefs-core --lib -- --emit=mir > "$WORK/z5/mir-build.raw" 2>&1
  local mir vstd
  mir=$(ls "$WORK/z5/build/debug/deps/"singlefs_core-*.mir | head -1)
  vstd=$(ls -d "$HOME"/.cargo/registry/src/*/vstd-0.0.0-2026-09-20-0158 | head -1)
  python3 -I "$R2S/tools/mir_callees.py" "$mir" "$WORK/z5/callees.tsv" >> "$WORK/out/z5.txt"
  CORE_SRC="$WORK/z5/base/crates/singlefs-core/src" python3 -I "$R2S/tools/closure_classes.py" "$WORK/z5/callees.tsv" "$vstd" "$WORK/z3/closure-publish.txt" "$WORK/z3/closure-roll_back_by_a_forward_publish.txt" | sed -n '1,6p' >> "$WORK/out/z5.txt"
  CORE_SRC="$WORK/z5/base/crates/singlefs-core/src" python3 -I "$R2S/tools/std_coverage.py" "$WORK/z5/callees.tsv" "$vstd" "$WORK/z3/closure-publish.txt" "$WORK/z3/closure-roll_back_by_a_forward_publish.txt" | sed -n '8,9p' >> "$WORK/out/z5.txt"
  # 证明行对代码行：现有核（冻结副本里的 replay-prefix）与第二轮正推的新模块
  mkdir -p "$WORK/z5/ratiotree/crates"
  rsync -a --exclude target "$FROZEN/crates/singlefs-replay-prefix" "$WORK/z5/ratiotree/crates/"
  cp "$R2S/src/relocation_judgement.rs" "$R2S/src/gc_ring_proofs.rs" "$R2S/src/relocation_interval_proofs.rs" "$WORK/z5/ratiotree/crates/singlefs-replay-prefix/src/"
  cp "$FROZEN/Cargo.lock" "$WORK/z5/ratiotree/"
  printf '[workspace]\nmembers = ["crates/singlefs-replay-prefix"]\nresolver = "2"\n' > "$WORK/z5/ratiotree/Cargo.toml"
  REPO_ROOT="$REPO_ROOT" python3 -I "$MODEL/tools/proof_ratio.py" "$WORK/z5/ratiotree" | sed -E 's/^/name=z5_proof_ratio /' >> "$WORK/out/z5.txt"
  local S="$WORK/z5/ratiotree/crates/singlefs-replay-prefix/src"
  python3 -I "$MODEL/tools/erased_lines.py" "$S/compaction_plan_proofs.rs" "$S/relocation_judgement.rs" "$S/gc_crash_proofs.rs" "$S/lib.rs" 2>&1 | sed -E 's/^/name=z5_erased /' >> "$WORK/out/z5.txt"
  compare_with_reference z5
}

rerun_steps() {
  set +e   # 各步里有意让命令退非 0（必红世界、探针里的不支持），退出码都跟在后面单独记
  mkdir -p "$WORK/out"
  R2S="$REPO_ROOT/research/prompts/m4-exit5-relocation-proof-r2-sonnet-model"
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
  want z1 && step_z1
  want z2 && step_z2
  want z3 && step_z3
  want z4 && step_z4
  want z5 && step_z5
  if [[ -z "${RERUN_KEEP_BUILD:-}" ]]; then rm -rf "$WORK/z1/crate" "$WORK/z2/target" "$WORK/z2/z2crate" "$WORK/z5/build" "$WORK/z5/base" "$WORK/z5/ratiotree" "$WORK/keytree" "$WORK/keytree2"; fi
  echo "name=rerun_result mismatches=$MISMATCHES"
  return "$MISMATCHES"
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
