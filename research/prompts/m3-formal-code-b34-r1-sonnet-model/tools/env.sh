# 复跑公共环境：S=自己的草稿目录（必填），FROZEN=冻结副本根（默认主 agent 给的那一份），REPO=仓根
: "${S:?要先设 S=<自己的草稿目录>}"
FROZEN=${FROZEN:-/tmp/claude-1000/m3-formal-code-b34-r1/tree}
REPO=${REPO:-/home/fy5090/code/singlefs}
MODEL=${MODEL:-$REPO/research/prompts/m3-formal-code-b34-r1-sonnet-model}
export CARGO_TARGET_DIR=$S/target
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-m3-formal-code-b34-r1 复跑在自己的仓副本与自己的编译目录里，不与别的会话共用副本}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-派发提示要求显式 8G 上限，不用 auto}"
# 每条 cargo 经内存包装（8G）与线程上限 10
export RUNNER="nice -n 19 bash $REPO/scripts/run-with-memory-cap.sh 8G bash $REPO/scripts/capped.sh 10 cargo test --offline"
