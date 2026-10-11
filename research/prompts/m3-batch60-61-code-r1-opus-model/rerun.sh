#!/usr/bin/env bash
# 攻方腿（m3-batch60-61-code-r1 opus）的复跑：rerun.sh <工作目录> <阶段>…（或 all；阶段表 `rerun.sh <工作目录> list`）。
# 环境：SINGLEFS_REPO（被判的冻结副本，缺省 ~/.local/share/singlefs/work/m3-batch60-61-code-r1/repo）、SINGLEFS_SCRIPTS（仓的 scripts，
# 缺省按模型目录在仓里的位置推）、OPUS_THREADS（线程上限，缺省 15）、OPUS_MEMORY_CAP（内存包装上限，缺省 4G：16G 在别的会话占着 slice 时排队）。
# 量过的跑法（2026-10-11，本腿）：每个阶段单独起、经 timeout 230 前台跑；机器满载时 mut-mv 拆成 mut-mv-a、mut-mv-b。
# cost-build 带 OPUS_COST_SIZES=0,3000,30000,120000（进度落盘，没建完重起接着建），cost-measure 每次一档；resume 带
# OPUS_RESUME_STORE=/dev/shm/<目录>（盘上同步写排队时）与 OPUS_RESUME_FRACTIONS（每次几档）。resume-frozen（冻结副本原样的 64 MiB 批长）没跑成。
# 副本只拷 Cargo.toml、Cargo.lock、crates/；原型 crate 拷成副本工作区成员 crates/opus-probe-b6061（测试目标名不带 layer0，不开 gpu 特性）；
# 冻结副本的流水线用例文件 crash_amplification_pipeline_records_checks_and_reuses.rs 原样拷成原型的 tests/pipeline_import_probe.rs
# （只把 `mod common_crash_points;` 指回 checker 档 tests/ 下那一份），末尾接 probes/pipeline_import_append.rs。
# base 与变异副本 mut-<名> 共用一个编译目录 target（release；三方依赖与 librocksdb-sys 只编一次，工作区成员按路径各编各的）；
# 同一时刻只起一条 cargo，先只编不跑（经 capped.sh），再经 capped.sh 与 run-with-memory-cap.sh 跑。产物写 <工作目录>/out/<阶段>.txt。
# compare 只比确定的那几份（pipeline 与变异）；cost-* 与 resume 是计时与杀进程时刻定的，只落盘、不比。
set -euo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="${SINGLEFS_REPO:-$HOME/.local/share/singlefs/work/m3-batch60-61-code-r1/repo}"
SCRIPTS="${SINGLEFS_SCRIPTS:-$MODEL/../../scripts}"
THREADS="${OPUS_THREADS:-15}"
CAP="${OPUS_MEMORY_CAP:-4G}"
WORK="$(mkdir -p "$1" && cd "$1" && pwd)"
shift
mkdir -p "$WORK/out" "$WORK/tmp" "$WORK/stores" "$WORK/stores-rb"
export TMPDIR="$WORK/tmp"
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include"
PROBE_TEST="crates/opus-probe-b6061/tests/pipeline_import_probe.rs"

copy_of() { # copy_of <名> [变异名]：建副本、装原型 crate、拼流水线原型、打变异
  local name="$1" mutant="${2:-}"
  local dir="$WORK/$name"
  rm -rf "${dir:?}"; mkdir -p "$dir"
  cp "$REPO/Cargo.toml" "$REPO/Cargo.lock" "$dir/"
  rsync -a --exclude target "$REPO/crates/" "$dir/crates/"
  cp -r "$MODEL/probe-crate" "$dir/crates/opus-probe-b6061"
  python3 - "$dir" "$MODEL/probes/pipeline_import_append.rs" <<'PY'
import sys
from pathlib import Path
root, append = Path(sys.argv[1]), Path(sys.argv[2])
manifest = root / "Cargo.toml"
text = manifest.read_text()
old = '    "crates/singlefs-reference-filesystem",\n'
assert text.count(old) == 1
manifest.write_text(text.replace(old, old + '    "crates/opus-probe-b6061",\n'))
source = (root / "crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs").read_text()
old = "\nmod common_crash_points;\n"
assert source.count(old) == 1
source = source.replace(old, '\n#[path = "../../singlefs-checker-tier/tests/common_crash_points/mod.rs"]\nmod common_crash_points;\n')
(root / "crates/opus-probe-b6061/tests/pipeline_import_probe.rs").write_text(source + append.read_text())
PY
  (cd "$dir" && find crates -type f -exec touch {} +)
  if [[ -n "$mutant" ]]; then python3 "$MODEL/mutants.py" "$dir" "$mutant"; fi
}

cargo_in() { # cargo_in <副本名> <cargo 参数…>：经线程上限跑；不带 --no-run 的再经内存包装
  local name="$1"; shift
  if [[ " $* " == *" --no-run "* ]]; then
    (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/target" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" cargo "$@")
  else
    (cd "$WORK/$name" && CARGO_TARGET_DIR="$WORK/target" OPUS_STORE_DIR="${OPUS_STORE_DIR_FOR_RUN:-}" nice -n 19 bash "$SCRIPTS/capped.sh" "$THREADS" \
      bash "$SCRIPTS/run-with-memory-cap.sh" "$CAP" cargo "$@")
  fi
}

run_tests() { # run_tests <副本名> <输出> <cargo test 参数…> [-- 测试参数…]：先只编、再跑；输出经 extract.py 摘录
  local name="$1" out="$2"; shift 2
  local args=() after=() seen=0
  for a in "$@"; do
    if [[ $seen == 1 ]]; then after+=("$a"); elif [[ "$a" == -- ]]; then seen=1; else args+=("$a"); fi
  done
  set +e
  cargo_in "$name" test --offline --release "${args[@]}" --no-run > "$WORK/tmp/last-build.log" 2>&1
  local build_rc=$?
  local started; started=$(date +%s)
  cargo_in "$name" test --offline --release --no-fail-fast "${args[@]}" -- "${after[@]}" > "$WORK/tmp/last.log" 2>&1
  local rc=$?
  set -e
  echo "args=$* build_rc=$build_rc wall_s=$(( $(date +%s) - started ))" >> "${out%.txt}.wall"
  { echo "command args=$* rc=$rc"; python3 "$MODEL/extract.py" < "$WORK/tmp/last.log"; } >> "$out"
}

PIPE=(-p opus-probe-b6061 --test pipeline_import_probe)
STORE=(-p opus-probe-b6061 --test planned_import_store_probe)
BATCH_TESTS=(--exact importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow
  given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import)

mutant_copy() { # 同一个变异的阶段被打断后重起时不重拷
  local mutant="$1"
  if [[ "$(cat "$WORK/mut.applied" 2>/dev/null)" != "$mutant" || ! -d "$WORK/mut" ]]; then
    copy_of mut "$mutant" > /dev/null
    echo "$mutant" > "$WORK/mut.applied"
  fi
}

P2_TARGETS=(-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search --test corrupt_on_disk_content_is_refused_instead_of_panicking)
P3_TARGETS=(-p singlefs-harness --test system_configuration_slot_is_overwritten_only_after_a_barrier
  --test shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring)
ORDER_FILTER=(allocation_records_whose_generation_is_after_the_root)

STAGES=(base-build base-batch base-h1 base-h2 base-h3 base-h4 mut-mv mut-mi-pipe mut-mi-given mut-mp cost-build cost-measure resume resume-frozen
  p2-base p3-base mut-tsv3957 mut-tsv3958 mut-tsv857 mut-tsv2755 mut-row-65 mut-row-396 mut-row-657 mut-row-2232)

stage() {
  local out="$WORK/out/$1.txt"
  rm -f "$out" "${out%.txt}.wall"
  case "$1" in
    base-build)
      [[ -d "$WORK/base/crates/opus-probe-b6061" ]] || copy_of base
      local rc=0
      cargo_in base test --offline --release -p opus-probe-b6061 --no-run >> "$WORK/tmp/base-build.log" 2>&1 || rc=$?
      echo "base-build rc=$rc" >> "$out" ;;
    base-batch) run_tests base "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 "${BATCH_TESTS[@]}" ;;
    base-h1) run_tests base "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h01 opus_h02 ;;
    base-h2) run_tests base "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h03 opus_h04 ;;
    base-h3) run_tests base "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h05 opus_h06 ;;
    base-h4) run_tests base "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h07 opus_h08 ;;
    mut-mv) mutant_copy mv
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 "${BATCH_TESTS[@]}"
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h02 opus_h03 ;;
    mut-mv-a) mutant_copy mv   # 机器满载时 mut-mv 拆两段跑：a 段被判的两条，b 段 h02、h03
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 "${BATCH_TESTS[@]}" ;;
    mut-mv-b) mutant_copy mv
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h02 opus_h03 ;;
    mut-mi-pipe) mutant_copy mi-pipe
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 "${BATCH_TESTS[@]}"
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h07 ;;
    mut-mi-given) mutant_copy mi-given
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 "${BATCH_TESTS[@]}" ;;
    mut-mp) mutant_copy mp
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 "${BATCH_TESTS[@]}"
      run_tests mut "$out" "${PIPE[@]}" -- --nocapture --test-threads 1 opus_h02 opus_h03 ;;
    cost-build) # 每次最多建 OPUS_BUILD_SECONDS 秒（缺省 200），没建完的阶段重起接着建
      OPUS_STORE_DIR_FOR_RUN="$WORK/stores" run_tests base "$out" "${STORE[@]}" -- --nocapture --test-threads 1 --exact opus_cost_build ;;
    cost-measure) OPUS_STORE_DIR_FOR_RUN="$WORK/stores" run_tests base "$out" "${STORE[@]}" -- --nocapture --test-threads 1 --exact opus_cost_measure ;;
    resume) # 批长改成 256 KiB 的副本（mutants.py 的 rb），40 块、每块 4096 个状态带恢复字（约 16 批）；杀的时刻取 OPUS_RESUME_FRACTIONS
      mutant_copy rb
      # 库落 OPUS_RESUME_STORE（缺省 <工作目录>/stores-rb；盘上同步写排队时设成 tmpfs 下的目录：杀的是进程不是断电，页缓存里的写照样留着）
      mkdir -p "${OPUS_RESUME_STORE:-$WORK/stores-rb}"
      OPUS_STORE_DIR_FOR_RUN="${OPUS_RESUME_STORE:-$WORK/stores-rb}" OPUS_RESUME_BLOCKS=40 OPUS_RESUME_STATES=4096 \
        run_tests mut "$out" "${STORE[@]}" -- --nocapture --test-threads 1 --exact opus_resume_after_a_kill ;;
    resume-frozen) # 冻结副本原样的批长（64 MiB）、120 块、每块 65536 个状态（约 4 批）；机器空闲时跑
      OPUS_STORE_DIR_FOR_RUN="$WORK/stores" run_tests base "$out" "${STORE[@]}" -- --nocapture --test-threads 1 --exact opus_resume_after_a_kill ;;
    p2-base) run_tests base "$out" "${P2_TARGETS[@]}" ;;
    p3-base) run_tests base "$out" "${P3_TARGETS[@]}" ;;
    mut-tsv3957|mut-tsv3958) mutant_copy "${1#mut-}"   # 第 60 批 N3 两行：分配记录判序挪到条目宽之后（冷走读、重建各一行）
      run_tests mut "$out" -p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- "${ORDER_FILTER[@]}" ;;
    mut-tsv857|mut-tsv2755) mutant_copy "${1#mut-}"    # 第 60 批 N4 替换的两行：取号写之前不过屏障、影子账候选过滤改回一圈窗口
      run_tests mut "$out" "${P3_TARGETS[@]}" ;;
    mut-row-*) # 变异表第 N 行照它自己第五段的 cargo test 参数跑（只收 harness 档的行；checker 档的行不跑，照规矩交主 agent）
      local row="${1#mut-row-}"; mutant_copy "tsv$row"
      local row_args; row_args="$(awk -F"\t" -v n="$row" "NR == n { print \$5 }" "$WORK/mut/crates/mutations.tsv")"
      if [[ "$row_args" == *singlefs-checker-tier* ]]; then echo "row $row 是 checker 档的行，不跑：$row_args" >> "$out"; return 0; fi
      local -a split_args; read -r -a split_args <<< "$row_args"
      run_tests mut "$out" "${split_args[@]}" ;;
    *) echo "不认识的阶段 $1" >&2; return 2 ;;
  esac
}

if [[ "${1:-}" == list ]]; then printf '%s\n' "${STAGES[@]}"; exit 0; fi
if [[ "${1:-}" == all ]]; then set -- "${STAGES[@]}" compare; fi
for name in "$@"; do
  if [[ "$name" == compare ]]; then python3 "$MODEL/compare.py" "$MODEL/expected" "$WORK/out"; else stage "$name"; fi
done
