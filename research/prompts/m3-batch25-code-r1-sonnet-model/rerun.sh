#!/usr/bin/env bash
# 在空目录里从头跑：把仓里的 Cargo.toml、Cargo.lock、crates 拷进工作目录（核 7 份被判文件的 sha256 与快照一致），
# 副本里追加 probe.rs（只在副本里），经 run-with-memory-cap.sh 编（6G）跑（4G）；峰值表会按实测把上限收紧到 2 倍峰值，这里用 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON 声明理由放行，上限仍是 6G 与 4G。。
# 环境变量：SINGLEFS_REPO（仓路径，必填）；SONNET_WORK（工作目录，默认 /tmp/claude-1000/m3-batch25-code-r1-sonnet-rerun，必须不存在）。
# 输出：stdout 与 $SONNET_WORK/summary.txt；与 expected-summary.txt 排序后比较，一致退 0。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK="${SONNET_WORK:-/tmp/claude-1000/m3-batch25-code-r1-sonnet-rerun}"
[[ ! -e "$WORK" ]] || { echo "工作目录已存在：$WORK" >&2; exit 2; }
mkdir -p "$WORK/orig" "$WORK/repo"
for item in Cargo.toml Cargo.lock crates; do
    cp -a "$SINGLEFS_REPO/$item" "$WORK/orig/$item"
    cp -a "$SINGLEFS_REPO/$item" "$WORK/repo/$item"
done
(cd "$WORK/orig" && sha256sum -c "$SINGLEFS_REPO/research/prompts/m3-batch25-code-r1-snapshot/crates-sha256.txt" >/dev/null) || { echo "被判文件与快照 sha256 不一致" >&2; exit 3; }
cat "$HERE/probe.rs" >> "$WORK/repo/crates/singlefs-harness/tests/checker_known_bad_images.rs"
export CARGO_TARGET_DIR="$WORK/target"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-上限按派发提示写死 4G 与 6G，峰值表按实测收紧}"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-派出去的几个 agent 各跑各的}"
SCRIPTS="$SINGLEFS_REPO/research/scripts"
cd "$WORK/repo"
nice -n 19 bash "$SCRIPTS/run-with-memory-cap.sh" 6G bash "$SCRIPTS/capped.sh" 10 cargo test --offline -p singlefs-harness --test checker_known_bad_images --no-run >"$WORK/build.log" 2>&1
{
    nice -n 19 bash "$SCRIPTS/run-with-memory-cap.sh" 4G bash "$SCRIPTS/capped.sh" 10 cargo test --offline -p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table zz_probe an_abandoned_root_naming_another_object_birth --nocapture --test-threads 1 2>&1 \
        | grep -E 'PROBE|^test ' | grep -v '^test result' | sed -E 's/^test zz_probe_what_each_batch25_image_reddens \.\.\. //' | sed -E 's/ finished in .*//'
    python3 "$HERE/mut.py" "$WORK" "$WORK/target" '第 25 批 I-9.29 门（CPU）：判不了的门改回另用只读不判的读法读最新根的实例表（攻方 N5-a' '第 25 批 I-9.29 门（CPU）：判不了的门改回另用只读不判的读法读最新根的实例表（攻方 N5-c' '第 25 批 I-9.29 门（CPU）：最新根的实例表链指针记录不合法也当主走读走到了末片' '第23批 X I-9.29：最新根的实例表读不出时当它滤过了被抛弃的根，照旧跨版本比' own-never-open
} >"$WORK/summary.raw" 2>&1
LC_ALL=C sort "$WORK/summary.raw" >"$WORK/summary.txt"
cat "$WORK/summary.txt"
LC_ALL=C sort "$HERE/expected-summary.txt" | diff - "$WORK/summary.txt" && echo "与 expected-summary.txt 一致"
