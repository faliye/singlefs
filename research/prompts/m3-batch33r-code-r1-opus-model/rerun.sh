# 用法：SINGLEFS_REPO=<仓根> [FROZEN_TREE=<冻结副本>] bash rerun.sh <空的输出目录> <阶段>…   阶段：build roots threads verdicts warm-ups:<i> instance-row:<i> merge all
# m3-batch33r-code-r1 攻方腿（Opus）原型的复跑。仓路径取环境变量 SINGLEFS_REPO（只用它的 scripts/run-with-memory-cap.sh、capped.sh
# 与 research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt）；被编的代码取 FROZEN_TREE（默认冻结副本），先核被判 4 份的 sha256。
# 编译经内存包装、上限 6G，跑经内存包装、上限 4G，线程 6（capped.sh 6）；编译目录是 <输出目录>/target，不碰仓里的 target。
# 阶段分开起是因为 oracle 两截合计约 39 万个状态、轻载时约半小时：子 agent 前台一条命令四分钟以内，一片一条；主 agent 可以 `all` 一次跑完。
# warm-ups 分 16 片（i = 0..15）、instance-row 分 24 片（i = 0..23），merge 并两截的账本。每条命令的输出写 <输出目录>/<阶段>.out。
set -euo pipefail
: "${SINGLEFS_REPO:?设 SINGLEFS_REPO 为仓根}"
FROZEN_TREE="${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-batch33r-code-r1/tree}"
OUT="${1:?给输出目录}"
shift
MODEL="$(cd "$(dirname "$0")" && pwd)"
CAP="$SINGLEFS_REPO/scripts/run-with-memory-cap.sh"
CAPPED="$SINGLEFS_REPO/scripts/capped.sh"
BIN="$OUT/target/release/opus-probe-b33r"
run() { # run <输出名> <环境…> -- <参数…>
    local name="$1"; shift
    local -a environment=()
    while [[ "$1" != "--" ]]; do environment+=("$1"); shift; done
    shift
    env "${environment[@]}" OPUS_PROBE_THREADS=6 nice -n 19 bash "$CAP" 4G bash "$CAPPED" 6 "$BIN" "$@" > "$OUT/$name.out" 2>&1
}
stage() {
    case "$1" in
        build)
            mkdir -p "$OUT"
            [[ -z "$(ls -A "$OUT")" ]] || { echo "输出目录 $OUT 不是空的" >&2; exit 2; }
            (cd "$FROZEN_TREE" && sha256sum -c "$SINGLEFS_REPO/research/prompts/m3-batch33r-code-r1-snapshot/crates-sha256.txt")
            rsync -a --exclude target --exclude .git "$FROZEN_TREE/" "$OUT/tree/"
            mkdir -p "$OUT/tree/crates/opus-probe-b33r/src"
            cp "$MODEL/probe/Cargo.toml" "$OUT/tree/crates/opus-probe-b33r/Cargo.toml"
            cp "$MODEL/probe/src/main.rs" "$OUT/tree/crates/opus-probe-b33r/src/main.rs"
            sed -i 's|    "crates/singlefs-reference-filesystem",|    "crates/singlefs-reference-filesystem",\n    "crates/opus-probe-b33r",|' "$OUT/tree/Cargo.toml"
            grep -q '"crates/opus-probe-b33r"' "$OUT/tree/Cargo.toml"
            (cd "$OUT/tree" && CARGO_TARGET_DIR="$OUT/target" nice -n 19 bash "$CAP" 6G bash "$CAPPED" 6 \
                cargo build --offline --release -p opus-probe-b33r) > "$OUT/build.out" 2>&1
            ;;
        roots) run roots -- roots ;;
        threads) mkdir -p "$OUT/progress-threads"; run threads OPUS_PROBE_STATES_PER_SLICE=1 OPUS_PROBE_PROGRESS_DIR="$OUT/progress-threads" -- threads h1 2 ;;
        verdicts) run verdicts-h1 -- verdicts h1 100 ;;
        warm-ups:*)
            mkdir -p "$OUT/ledgers-warm-ups"
            run "oracle-warm-ups-s${1#*:}" OPUS_PROBE_SHARD="${1#*:}/16" OPUS_PROBE_PROGRESS_DIR="$OUT/ledgers-warm-ups" -- oracle h2 2 mount-warm-ups ;;
        instance-row:*)
            mkdir -p "$OUT/ledgers-instance-row"
            run "oracle-instance-row-s${1#*:}" OPUS_PROBE_SHARD="${1#*:}/24" OPUS_PROBE_PROGRESS_DIR="$OUT/ledgers-instance-row" -- oracle h2 2 mount-instance-row ;;
        merge)
            run oracle-warm-ups-merge OPUS_PROBE_MERGE=16 OPUS_PROBE_PROGRESS_DIR="$OUT/ledgers-warm-ups" -- oracle h2 2 mount-warm-ups
            run oracle-instance-row-merge OPUS_PROBE_MERGE=24 OPUS_PROBE_PROGRESS_DIR="$OUT/ledgers-instance-row" -- oracle h2 2 mount-instance-row ;;
        all)
            stage build; stage roots; stage threads; stage verdicts
            for i in $(seq 0 15); do stage "warm-ups:$i"; done
            for i in $(seq 0 23); do stage "instance-row:$i"; done
            stage merge ;;
        *) echo "没有这个阶段：$1" >&2; exit 2 ;;
    esac
}
for each in "$@"; do stage "$each"; echo "stage=$each done"; done
