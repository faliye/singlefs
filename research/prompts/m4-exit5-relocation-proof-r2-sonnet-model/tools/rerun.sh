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
# 路径只用 $REPO_ROOT、$FROZEN、$MODEL、$WORK 四个变量引；cargo 与 verus 都经 "$REPO_ROOT/scripts/hosts/capped.sh" 31 起（线程上限 31）。
# 每一步的输出去掉耗时与路径之后写进 $WORK/out/<名>.txt，与 $MODEL/out/<名>.txt 逐行比；对不上的件数就是退出码（0 = 全对上）。
# RERUN_STEPS=y1,y2,... 只跑点名的段（默认全跑，段名：setup y1 y1p y2 y3 y4 y5）；RERUN_WRITE_REFERENCE=1 把这一趟的输出写成参照（只给腿自己建参照用）。
MISMATCHES=0
VERUS_DIR="$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux"
VERUS="$VERUS_DIR/verus"
VSTD_SRC="$(ls -d "$HOME"/.cargo/registry/src/*/vstd-0.0.0-2026-09-20-0158 | head -1)"

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

cargo_in() {   # cargo_in <树> <编译目录名> <子命令（可含空格）> <参数…>：cargo 起在树里，--manifest-path 紧跟子命令
  local tree="$1" tdir="$2" sub="$3"; shift 3
  PATH="$VERUS_DIR:$PATH" CARGO_TARGET_DIR="$WORK/build/$tdir" capped cargo $sub --manifest-path "$tree/Cargo.toml" "$@"
}

summary_lines() {   # 只留稳定的行：verification results、error: 开头的首行（去路径）、warning 条数
  grep -E '^(verification results|error)' | sed -E 's#/[^ ]*/(crates/)#\1#g' | cut -c1-160 | sort -u
}

make_tree() {   # make_tree <名> [补丁]：最小工作区（format、core、replay-prefix）+ 补丁
  local name="$1" patch_file="${2:-}"
  mkdir -p "$WORK/trees/$name/crates" "$WORK/trees/$name/.claude"
  rsync -a --exclude target "$FROZEN/crates/singlefs-format" "$FROZEN/crates/singlefs-core" "$FROZEN/crates/singlefs-replay-prefix" "$WORK/trees/$name/crates/"
  cp "$FROZEN/Cargo.lock" "$WORK/trees/$name/"; cp -r "$FROZEN/.cargo" "$WORK/trees/$name/"
  cp "$MODEL/tools/workspace-Cargo.toml" "$WORK/trees/$name/Cargo.toml"
  cp "$FROZEN/.claude/heavy-thread-default" "$WORK/trees/$name/.claude/" 2>/dev/null || true
  if [[ -n "$patch_file" ]]; then (cd "$WORK/trees/$name" && patch -s -p1 < "$MODEL/patches/$patch_file"); fi
}

mir_normalized() {   # 去掉路径印法与行号的差别（vstd 进了作用域之后 Map 等名字的印法变长，行号随多出的 use 行移动）
  sed -E 's/std::(iter::|array::|slice::|vec::|collections::|cell::|rc::|fmt::|option::|result::|string::|str::)/\1/g; s/\.rs:[0-9]+:[0-9]+: [0-9]+:[0-9]+/.rs:L/g; s/iter::Map/Map/g' "$1"
}

ensure_variants() {   # 五个变体的最小工作区：base（原样）、verusbang（verus! 包一个函数）、attribute（属性模式）、inplace（core 里直验的 m4）、kernelmodule（core 里新模块的 s1）
  local entry
  [[ -d "$WORK/trees/base" ]] || make_tree base
  for entry in verusbang:y1-m1.patch attribute:y1-m2.patch inplace:y3-m4.patch kernelmodule:y3-s1.patch; do
    [[ -d "$WORK/trees/${entry%%:*}" ]] || make_tree "${entry%%:*}" "${entry#*:}"
  done
}

ensure_callees() {
  ensure_variants
  if [[ ! -s "$WORK/out/callees.tsv" ]]; then
    local mir_file
    cargo_in "$WORK/trees/base" mir rustc --offline -p singlefs-core --lib -- --emit=mir > /dev/null 2>&1
    mir_file=$(ls "$WORK/build/mir/debug/deps/"singlefs_core-*.mir | head -1)
    python3 -I "$MODEL/tools/mir_callees.py" "$mir_file" "$WORK/out/callees.tsv" > "$WORK/out/y2-mir.txt"
  fi
}

step_setup() {
  mkdir -p "$WORK/out" "$WORK/build" "$WORK/trees"
  rsync -a --exclude target --exclude .git "$FROZEN/crates" "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/.cargo" "$WORK/tree-a/"
  mkdir -p "$WORK/tree-a/.claude" && cp "$FROZEN/.claude/heavy-thread-default" "$WORK/tree-a/.claude/" 2>/dev/null || true
  ensure_variants
  echo "name=rerun_roots repo=$REPO_ROOT frozen=$FROZEN model=$MODEL work=$WORK"
}

# Y1：怎么接——最小工作区里五个变体各自「产品构建」「Verus」两样，和 MIR 比对
step_y1() {
  : > "$WORK/out/y1.txt"
  local variant
  ensure_variants
  for variant in base verusbang attribute inplace kernelmodule; do
    cargo_in "$WORK/trees/$variant" prod build --offline -p singlefs-core > "$WORK/out/y1-$variant-build.raw" 2>&1
    echo "name=y1_build variant=$variant rc=$? warnings=$(grep -c '^warning' "$WORK/out/y1-$variant-build.raw")" >> "$WORK/out/y1.txt"
    if [[ "$variant" != base ]]; then
      cargo_in "$WORK/trees/$variant" verify "verus focus" --offline -p singlefs-core > "$WORK/out/y1-$variant-verify.raw" 2>&1
      echo "name=y1_verify variant=$variant rc=$? $(summary_lines < "$WORK/out/y1-$variant-verify.raw" | tr '\n' '|')" >> "$WORK/out/y1.txt"
    fi
  done
  # MIR：产品构建的全部 MIR 函数，原版与两种接法逐行比（去掉印法差别之后）
  local mir_dir
  for variant in base verusbang attribute; do
    cargo_in "$WORK/trees/$variant" "mir-$variant" rustc --offline -p singlefs-core --lib -- --emit=mir > /dev/null 2>&1
    mir_dir=$(ls "$WORK/build/mir-$variant/debug/deps/"singlefs_core-*.mir | head -1)
    mir_normalized "$mir_dir" > "$WORK/out/mir-$variant.norm"
  done
  echo "name=y1_mir_fn_count base=$(grep -c '^fn ' "$WORK/out/mir-base.norm") verusbang=$(grep -c '^fn ' "$WORK/out/mir-verusbang.norm") attribute=$(grep -c '^fn ' "$WORK/out/mir-attribute.norm")" >> "$WORK/out/y1.txt"
  echo "name=y1_mir_diff_lines verusbang=$(diff "$WORK/out/mir-base.norm" "$WORK/out/mir-verusbang.norm" | wc -l) attribute=$(diff "$WORK/out/mir-base.norm" "$WORK/out/mir-attribute.norm" | wc -l)" >> "$WORK/out/y1.txt"
  compare_with_reference y1
}

# Y1 的单文件探针：类型在 verus! 外、属性模式、external_type_specification / assume_specification / external_body 在 --no-cheating 下、验外代码不被检查
step_y1p() {
  mkdir -p "$WORK/out"
  : > "$WORK/out/y1-probes.txt"
  local probe
  for probe in "$MODEL"/probes/y1_*.rs; do
    for mode in "--no-cheating" ""; do
      capped "$VERUS" $mode --edition=2021 --num-threads 8 --rlimit 10 "$probe" > "$WORK/out/y1-probe.raw" 2>&1
      probe_rc=$?
      echo "$(basename "$probe" .rs) [${mode:-允许作弊标记}] rc=$probe_rc $(grep -m1 -E '^(error|verification results)' "$WORK/out/y1-probe.raw" | cut -c1-90)" >> "$WORK/out/y1-probes.txt"
    done
  done
  compare_with_reference y1-probes
}

probe_all() {   # probe_all <拆分源> <拆分目录>：对每个 pub fn 各起一次 verus，并行；输出 名<tab>rc<tab>首行，按名字排
  local src="$1" dir="$2"
  mkdir -p "$dir"
  if [[ -n "$src" ]]; then python3 -I "$MODEL/tools/probe-split.py" "$src" "$dir" > /dev/null; fi
  ( cd "$dir" && ls ./*.rs | xargs -P 12 -I{} bash -c '
      f="{}"; "$0" --no-cheating --edition=2021 --num-threads 2 --rlimit 10 "$f" > "$f.out" 2>&1; rc=$?
      printf "%s\t%s\t%s\n" "$(basename "$f" .rs)" "$rc" "$(grep -m1 -E "^(error|verification results)" "$f.out" | cut -c1-100)"' "$VERUS" | sort )
}

# Y2：std 规格——MIR 调用抽取、vstd 规格按名字配、五批探针、登记式 assume_specification 的演示
step_y2() {
  mkdir -p "$WORK/out"
  ensure_callees
  ensure_closures
  local core_src="$WORK/trees/base/crates/singlefs-core/src"
  { CORE_SRC="$core_src" python3 -I "$MODEL/tools/std_coverage.py" "$WORK/out/callees.tsv" "$VSTD_SRC" | head -12
    CORE_SRC="$core_src" python3 -I "$MODEL/tools/std_coverage.py" "$WORK/out/callees.tsv" "$VSTD_SRC" "$WORK/out/closures/closure-publish_relocation_batch.txt" "$WORK/out/closures/closure-roll_back_by_a_forward_publish.txt" | head -12; } >> "$WORK/out/y2-mir.txt"
  cut -f3 "$WORK/out/callees.tsv" | sort | uniq -c | sort -rn | grep -E " (Result::expect|Option::expect|rt::panic_fmt)\$" >> "$WORK/out/y2-mir.txt"
  compare_with_reference y2-mir
  { probe_all "$MODEL/probes/y2_constructs_a.rs" "$WORK/probes/a"; probe_all "$MODEL/probes/y2_constructs_b.rs" "$WORK/probes/b"
    mkdir -p "$WORK/probes/c" && cp "$MODEL"/probes/y2c-split/*.rs "$WORK/probes/c/" && probe_all "" "$WORK/probes/c"
    probe_all "$MODEL/probes/y2_constructs_d.rs" "$WORK/probes/d"; probe_all "$MODEL/probes/y2_constructs_e.rs" "$WORK/probes/e"; } > "$WORK/out/y2-probes.txt"
  compare_with_reference y2-probes
  mkdir -p "$WORK/demo"
  SINGLEFS_RUN_SCOPE='种类=target; 跑什么=演示 crate assume-demo 的差分测试与它的必红世界; 范围=1 个测试; 为什么=核登记式 assume_specification 的差分测试能抓住写错的规格，演示 crate 是模型目录里的小库' \
    PATH="$VERUS_DIR:$PATH" capped python3 -I "$MODEL/tools/assume-registry-check.py" "$MODEL/assume-demo" "$WORK/demo/w" 2>&1 | grep '^name=' > "$WORK/out/y2-assume.txt"
  compare_with_reference y2-assume
}

ensure_closures() {
  if [[ ! -x "$WORK/build/ug/release/unit-graph" ]]; then
    CARGO_TARGET_DIR="$WORK/build/ug" capped cargo build --release --offline --manifest-path "$WORK/tree-a/Cargo.toml" -p singlefs-unit-graph --bin unit-graph > "$WORK/out/build-unit-graph.raw" 2>&1
  fi
  if [[ ! -s "$WORK/out/closures/closure-publish_relocation_batch.txt" ]]; then
    capped bash "$MODEL/tools/closures.sh" "$WORK/build/ug/release/unit-graph" "$WORK/tree-a" "$WORK/out/closures" \
      singlefs_core::transaction::publish_relocation_batch singlefs_core::mount::roll_back_by_a_forward_publish \
      | sed -E 's/，[0-9]+ 线程，[0-9.]+ 秒$//' > "$WORK/out/closures-table.txt"
  fi
}

# Y3：闭包规模、分类、第一阶段原型的必红世界
step_y3() {
  ensure_callees
  ensure_closures
  cp "$WORK/out/closures-table.txt" "$WORK/out/y3-closures.txt"
  local core_src="$WORK/trees/base/crates/singlefs-core/src"
  CORE_SRC="$core_src" python3 -I "$MODEL/tools/closure_classes.py" "$WORK/out/callees.tsv" "$VSTD_SRC" "$WORK/out/closures/closure-publish_relocation_batch.txt" "$WORK/out/closures/closure-roll_back_by_a_forward_publish.txt" >> "$WORK/out/y3-closures.txt"
  # 第一阶段原型接上 core 之后，发布闭包里「被验的」fn（核里的加 core 里 verus! 新模块里的）占多少
  capped "$WORK/build/ug/release/unit-graph" closure "$WORK/trees/kernelmodule" singlefs_core::transaction::publish_relocation_batch --toolchain-line "host: x86_64-unknown-linux-gnu" --threads 31 > "$WORK/out/s1-closure.txt" 2> "$WORK/out/s1-closure.err"
  awk -F"\t" '$1=="fn"{split($3,a,":"); split(a[2],b,"-"); n=b[2]-b[1]+1; t++; tl+=n; if (a[1] ~ /relocation_judgement.rs/ || a[1] ~ /singlefs-replay-prefix/) {k++; kl+=n}} END{print "接上第一阶段原型之后（publish_relocation_batch 闭包）：fn=" t " fn_lines=" tl " 被验 fn=" k " 被验行=" kl}' "$WORK/out/s1-closure.txt" >> "$WORK/out/y3-closures.txt"
  compare_with_reference y3-closures
  mkdir -p "$WORK/rwc"
  { PATH="$VERUS_DIR:$PATH" capped python3 -I "$MODEL/tools/red-worlds-core.py" "$WORK/trees/kernelmodule" "$MODEL/tools/red-worlds-core-s1.tsv" "$WORK/rwc"
    PATH="$VERUS_DIR:$PATH" capped python3 -I "$MODEL/tools/red-worlds-core.py" "$WORK/trees/inplace" "$MODEL/tools/red-worlds-core-m4.tsv" "$WORK/rwc"; } 2>&1 | grep -v '^run-with' | sed -E 's/首个错：.*落点/落点/' > "$WORK/out/y3-red-worlds.txt"
  compare_with_reference y3-red-worlds
}

# Y4：cargo 的指纹不看 --verify-*，同一个编译目录里换参数再验会拿到上一趟的结果
step_y4() {
  ensure_variants
  : > "$WORK/out/y4.txt"
  local tree="$WORK/trees/inplace"
  cargo_in "$tree" cache verus\ focus --offline -p singlefs-core -- --verify-only-module compaction_plan --verify-function region_is_empty_by_the_records > "$WORK/out/y4-first.raw" 2>&1
  echo "name=y4_first rc=$? $(grep -E '^note: verifying|^verification results' "$WORK/out/y4-first.raw" | tr '\n' '|')" >> "$WORK/out/y4.txt"
  cargo_in "$tree" cache verus\ focus --offline -p singlefs-core -- --verify-only-module logged_operations --verify-function CompactionRegion::contains_slot > "$WORK/out/y4-second.raw" 2>&1
  echo "name=y4_second_same_target rc=$? $(grep -E '^note: verifying|^verification results' "$WORK/out/y4-second.raw" | tr '\n' '|')" >> "$WORK/out/y4.txt"
  cargo_in "$tree" cache2 verus\ focus --offline -p singlefs-core -- --verify-only-module logged_operations --verify-function CompactionRegion::contains_slot > "$WORK/out/y4-third.raw" 2>&1
  echo "name=y4_second_fresh_target rc=$? $(grep -E '^note: verifying|^verification results' "$WORK/out/y4-third.raw" | tr '\n' '|')" >> "$WORK/out/y4.txt"
  cargo_in "$tree" cache3 verus\ focus --offline -p singlefs-core -- --verify-only-module logged_operations --verify-function contains_slot > "$WORK/out/y4-fourth.raw" 2>&1
  echo "name=y4_substring_ambiguity rc=$? $(grep -m1 -E 'more than one match' "$WORK/out/y4-fourth.raw" | cut -c1-100)" >> "$WORK/out/y4.txt"
  # --output-json：一趟整 crate 的结果里每个函数一项（func-details），逐函数判定不必逐函数起 Verus
  cargo_in "$tree" cache4 verus\ focus --offline -p singlefs-core -- --output-json > "$WORK/out/y4-fifth.raw" 2>&1
  echo "name=y4_json_per_function rc=$? $(python3 -I -c "
import json,re,sys
t=open(sys.argv[1],encoding=\"utf-8\").read()
j=None
for m in re.finditer(r\"^\\{\", t, re.M):
    try:
        j=json.loads(t[m.start():].split(\"\\n    Finished\")[0]); break
    except Exception:
        pass
fd=j.get(\"func-details\",{}) if j else {}
core=[k for k in fd if k.startswith(\"singlefs_core\")]
v=j.get(\"verification-results\",{}) if j else {}
print(\"func_details_total=%d core_entries=%d verified=%s errors=%s\" % (len(fd), len(core), v.get(\"verified\"), v.get(\"errors\")))" "$WORK/out/y4-fifth.raw")" >> "$WORK/out/y4.txt"
  compare_with_reference y4
}

# Y5：回退窗口与多逻辑版本的协议、区间版 P4：放进 replay-prefix 的副本整 crate 验，另各自单文件验、必红世界
step_y5() {
  : > "$WORK/out/y5.txt"
  local crate="$WORK/y5-crate"
  mkdir -p "$crate" && rsync -a --exclude target "$FROZEN/crates/singlefs-replay-prefix" "$crate/"
  cp "$MODEL/src/relocation_interval_proofs.rs" "$MODEL/src/gc_ring_proofs.rs" "$crate/singlefs-replay-prefix/src/"
  sed -i 's/^pub mod gc_crash_proofs;$/pub mod gc_crash_proofs;\npub mod gc_ring_proofs;\npub mod relocation_interval_proofs;/' "$crate/singlefs-replay-prefix/src/lib.rs"
  local base_crate="$WORK/y5-base" ; mkdir -p "$base_crate" && rsync -a --exclude target "$FROZEN/crates/singlefs-replay-prefix" "$base_crate/"
  local label dir
  for label in 冻结副本原样:"$base_crate" 加两份新证明:"$crate"; do
    dir="${label#*:}"; label="${label%%:*}"
    ( cd "$dir/singlefs-replay-prefix" && capped "$VERUS" --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads 31 --output-json --rlimit 10 src/lib.rs > "$WORK/out/y5-crate.json" 2> "$WORK/out/y5-crate.err" )
    echo "name=y5_crate label=$label rc=$? $(python3 -I -c "
import json,sys
t=open(sys.argv[1]).read().strip(); j=json.loads(t[t.find('{'):]); v=j['verification-results']; print('verified=%s errors=%s' % (v['verified'], v['errors']))" "$WORK/out/y5-crate.json")" >> "$WORK/out/y5.txt"
  done
  local f
  for f in relocation_interval_proofs gc_ring_proofs; do
    capped "$VERUS" --no-cheating --crate-type=lib --edition=2021 --num-threads 31 --rlimit 10 "$MODEL/src/$f.rs" > "$WORK/out/y5-$f.raw" 2>&1
    echo "name=y5_single_file file=$f rc=$? $(grep -E '^verification results' "$WORK/out/y5-$f.raw")" >> "$WORK/out/y5.txt"
  done
  mkdir -p "$WORK/red"
  THREADS=31 capped python3 -I "$MODEL/tools/red-worlds.py" "$MODEL/src" "$MODEL/tools/red-worlds-y5.tsv" "$WORK/red" >> "$WORK/out/y5.txt" 2>&1
  compare_with_reference y5
}

rerun_steps() {
  set +e   # 各步里有意让命令退非 0（必红世界、探针里的不支持），退出码都跟在后面单独记
  want setup && step_setup
  want y1 && step_y1
  want y1p && step_y1p
  want y2 && step_y2
  want y3 && step_y3
  want y4 && step_y4
  want y5 && step_y5
  if [[ -z "${RERUN_KEEP_BUILD:-}" ]]; then rm -rf "$WORK/build" "$WORK/trees" "$WORK/tree-a" "$WORK/y5-crate" "$WORK/y5-base" "$WORK/rwc" "$WORK/demo" "$WORK/red" "$WORK/probes"; fi
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
