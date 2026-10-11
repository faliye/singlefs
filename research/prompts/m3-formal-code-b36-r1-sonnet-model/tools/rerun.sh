#!/usr/bin/env bash
# 正推腿复跑：在自己的草稿目录里照这份重跑（不动仓、不动冻结副本）。
#   S=<自己的草稿目录> bash research/prompts/m3-formal-code-b36-r1-sonnet-model/tools/rerun.sh <阶段…>
# 阶段（次序即依赖）：
#   setup    把冻结副本拷到 $S/work、核 sha256 清单（0 行不对才算数）
#   build    只编（日志 $S/build.log 要有 Compiling singlefs-checker / singlefs-core / singlefs-harness，没有这一趟不作数）
#   tests    被测的 4 条 harness 用例基线（两条新用例、B2、35-1 FL 那条）→ $S/t-base-rows.log
#   probes-build 拼探针文件并只编；p1-a / p1-b / p1-c  P1 子集矩阵 9 种改法分三趟；probes-small  P1b 网格、P1c、P1d、R3、H2 / H3；probes-r2a / probes-r2b  R2 新旧造法对照（两条）
#            （探针拼进 zz_probe_b36.rs，opt-level 2 的编译目录 $S/target-opt）：P1 子集矩阵（9 种改法，分两趟跑）、P1b 网格、P1c、P1d、R3 理由文字、
#            H2 / H3 合法历史逐状态签名、R2 新旧造法对照（两条）→ $S/t-*.log
#   r1-off   把第 36 批 36-1 R1 那一行变异（改回旧认法）临时施加到副本，重跑 P1 矩阵与 H2 / H3，还原并核 sha256；比 R1 开 / 关的逐状态签名
#   rows     第 36 批 8 行变异：select_rows36.py 核行与锚点、anchors_all.py 核整表锚点；harness 档 4 行逐行施加、跑、还原（checker-tier 的 4 行不跑）→ $S/mut-results.jsonl
#   wgsl     把两个池级 checker 内核照 crash_judge_gpu.rs 拼起来交 naga 解析与校验，再把 GPU 四行变异施加上去各校验一遍（不跑 GPU）
# 每条 cargo 经 run-with-memory-cap（8G）与 capped.sh 16（tools/env.sh）；同一时刻只起一条。同会话并行时先设 RUN_WITH_MEMORY_CAP_PARALLEL_REASON。
# 峰值表里没有这几组命令的键，上限 8G 不触发「超过峰值两倍」那一条；若退 251，设 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON（env.sh 已给默认理由）。
# 单条前台命令不超过 240 秒：P1 矩阵 9 种改法逐掩码约 4.5 分钟，分三趟（PROBE_P1_ONLY 选改法）。
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
. "$HERE/env.sh"
W="$S/work"
D="$(dirname "$FROZEN")"
probe() { if [ -n "${PROBE_P1_ONLY:-}" ]; then OPT=1 PROBE_P1_ONLY="$PROBE_P1_ONLY" bash "$HERE/rerun_probe.sh" "$@"; else env -u PROBE_P1_ONLY OPT=1 bash "$HERE/rerun_probe.sh" "$@"; fi; }
for stage in "$@"; do
  case "$stage" in
  setup)
    mkdir -p "$S"; rm -rf "$W"; mkdir -p "$W"; cp -r "$FROZEN"/. "$W"/
    (cd "$W" && sha256sum -c "$D/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/不对的行数: /' ;;
  build) bash "$HERE/build.sh"; grep -E 'Compiling singlefs|Finished|exit=' "$S/build.log" ;;
  tests)
    bash "$HERE/run-tests.sh" "$S/t-base-rows.log" -p singlefs-harness --test checker_known_bad_images -- --exact \
      a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first \
      a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first \
      a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first \
      a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot
    grep -hE '^test |test result|exit=' "$S/t-base-rows.log" ;;
  probes-build)
    rm -f "$W/crates/singlefs-harness/tests/zz_probe_b36.rs"
    python3 "$HERE/build_probes.py" "$W" "$MODEL"
    OPT=1 bash "$HERE/run-tests.sh" "$S/t-probe-build.log" -p singlefs-harness --test zz_probe_b36 --no-run
    grep -E 'Compiling singlefs|Finished|exit=' "$S/t-probe-build.log" ;;
  p1-a) PROBE_P1_ONLY=clean,overlap,overlap+stale_first probe t-p1-a.log probe_p1_subsets_of_the_record_copies_of_a_three_record_publish; tail -3 "$S/t-p1-a.log" | cut -c1-120 ;;
  p1-b) PROBE_P1_ONLY=overlap+stale_last,overlap+no_anchor,overlap+first_is_instance2 probe t-p1-b.log probe_p1_subsets_of_the_record_copies_of_a_three_record_publish; tail -3 "$S/t-p1-b.log" | cut -c1-120 ;;
  p1-c) PROBE_P1_ONLY=overlap+gen,overlap+txg5,overlap+txg5+no_anchor probe t-p1-c.log probe_p1_subsets_of_the_record_copies_of_a_three_record_publish; tail -3 "$S/t-p1-c.log" | cut -c1-120
    cat "$S"/t-p1-a.log "$S"/t-p1-b.log "$S"/t-p1-c.log | grep '^PROBE-P1 ' > "$S/t-p1-all.log"
    python3 "$HERE/summarize_p1.py" "$S/t-p1-all.log" ;;
  probes-small)
    probe t-p1bgrid.log probe_p1b_grid_of_two_single_record_publishes_with_copies_and_named_units
    probe t-p1c.log probe_p1c_anchor_undecided
    probe t-p1d.log probe_p1d_after_a_rollback_publish_one_more_unrooted_publish
    probe t-r3.log probe_r3_first_data_unit_reason_text_by_form
    probe t-h2-on.log probe_h2_states_of_a_legal_three_record_publish
    probe t-h3-on.log probe_h3_states_after_a_mount_over_a_version_applied_only_by_its_three_records
    probe t-h3s-on.log probe_h3_stale_first_record_of_b
    python3 "$HERE/summarize_p1b.py" "$S/t-p1bgrid.log"
    grep -hE 'PROBE-(P1C|P1D|R3|H2|H3|H3S)' "$S"/t-p1c.log "$S"/t-p1d.log "$S"/t-r3.log "$S"/t-h2-on.log "$S"/t-h3-on.log "$S"/t-h3s-on.log | cut -c1-300 ;;
  probes-r2a)
    probe t-r2.log probe_r2_old_and_new_construction_of_the_applied_only_by_its_records_flow
    grep -hE 'PROBE-R2' "$S"/t-r2.log | cut -c1-300 ;;
  probes-r2b)
    probe t-r2f.log probe_r2_the_two_flows_by_name
    grep -hE 'PROBE-(R2|R2F1|R2F2)' "$S"/t-r2f.log | cut -c1-300 ;;
  r1-off)
    P="python3 $HERE/patch_b36.py $W $FROZEN"
    $P apply "第36批 36-1 R1：下一次" || exit 1
    OPT=1 bash "$HERE/run-tests.sh" "$S/t-r1-off-build.log" -p singlefs-harness --test zz_probe_b36 --no-run
    PROBE_P1_ONLY=overlap,overlap+first_is_instance2 probe t-p1-r1-off.log probe_p1_subsets_of_the_record_copies_of_a_three_record_publish
    probe t-h2-off.log probe_h2_states_of_a_legal_three_record_publish
    probe t-h3-off.log probe_h3_states_after_a_mount_over_a_version_applied_only_by_its_three_records
    probe t-h3s-off.log probe_h3_stale_first_record_of_b
    $P restore
    (cd "$W" && sha256sum -c "$D/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/还原后不对的行数: /'
    python3 "$HERE/summarize_p1.py" "$S/t-p1-r1-off.log"
    for p in h2 h3; do echo "$p 逐状态签名 R1 开 / 关不同的行数: $(diff <(grep '^SIG' "$S/t-$p-on.log") <(grep '^SIG' "$S/t-$p-off.log") | grep -c '^[<>]')"; done ;;
  rows)
    (cd "$S" && python3 "$HERE/select_rows36.py" "$W" "$REPO/research/prompts/_m3-formal-code-b36-r1-diff.md")
    python3 "$HERE/anchors_all.py" "$W" | tee "$S/anchors-all.log"
    rm -f "$S/mut-results.jsonl"
    for range in "0 1" "1 2" "2 3" "3 4"; do
      set -- $range
      python3 "$HERE/mutate_rows2.py" "$W" "$S/rows.json" "$1" "$2" "$S/mut-results.jsonl" | cut -c1-300
    done
    (cd "$W" && sha256sum -c "$D/crates-sha256.txt" 2>&1 | grep -vc ': OK$') | sed 's/^/还原后不对的行数: /' ;;
  wgsl)
    NAGA_TARGET="${NAGA_TARGET:-$S/target-naga}"
    mkdir -p "$S/naga-check/src"
    cp "$HERE/../naga-check/Cargo.toml" "$S/naga-check/Cargo.toml"; cp "$HERE/../naga-check/src/main.rs" "$S/naga-check/src/main.rs"; cp "$FROZEN/Cargo.lock" "$S/naga-check/Cargo.lock"
    (cd "$S/naga-check" && CARGO_TARGET_DIR="$NAGA_TARGET" nice -n 19 bash "$REPO/scripts/hosts/capped.sh" 16 cargo build --offline 2>&1 | tail -2)
    python3 - <<PY
import json
rows = json.load(open("$S/rows.json"))
for k, r in enumerate(rows): r["k"] = k
json.dump(rows, open("$S/rows-k.json", "w"), ensure_ascii=False)
PY
    # naga-check 是 cargo 编出来的二进制：整条校验脚本经内存包装跑（4G 上限）
    nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 4G python3 "$HERE/wgsl_validate.py" "$FROZEN" "$S/wgsl" "$NAGA_TARGET/debug/naga-check" "$S/rows-k.json"
    # 校验路径自己会红：两处故意写坏的源码（构造器少一个参数、引用不存在的常量）必须判红
    python3 - <<PY
import pathlib
w = pathlib.Path("$S/wgsl")
a = (w / "walk-unmutated.wgsl").read_text(encoding="utf-8"); b = (w / "scan-unmutated.wgsl").read_text(encoding="utf-8")
x = "    let nothing_applied = ReplayTheNextMountWouldDo(NONE, NONE);"; y = "    let replay_index = sw(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED);"
assert a.count(x) == 1 and b.count(y) == 1
(w / "walk-broken.wgsl").write_text(a.replace(x, "    let nothing_applied = ReplayTheNextMountWouldDo(NONE);"), encoding="utf-8")
(w / "scan-broken.wgsl").write_text(b.replace(y, y.replace("APPLIED);", "APPLIED_X);")), encoding="utf-8")
PY
    for f in walk-broken scan-broken; do
      nice -n 19 bash "$REPO/scripts/hosts/run-with-memory-cap.sh" 4G "$NAGA_TARGET/debug/naga-check" "$S/wgsl/$f.wgsl" | head -1 | cut -c1-120; echo "$f rc=${PIPESTATUS[0]}"
    done ;;
  *) echo "不认识的阶段 $stage" >&2; exit 2 ;;
  esac
done
