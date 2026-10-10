# 复跑公共环境：S=自己的草稿目录（必填），FROZEN=冻结副本根，REPO=仓根；OPT=1 时探针用 opt-level 2 的另一个编译目录（debug 版逐状态判太慢）
: "${S:?要先设 S=<自己的草稿目录>}"
FROZEN=${FROZEN:-/tmp/claude-1000/m3-formal-code-b35-r1/tree}
REPO=${REPO:-/home/fy5090/code/singlefs}
MODEL=${MODEL:-$REPO/research/prompts/m3-formal-code-b35-r1-sonnet-model}
export CARGO_TARGET_DIR=$S/target
if [ "${OPT:-0}" = 1 ]; then
  export CARGO_TARGET_DIR=$S/target-opt CARGO_PROFILE_DEV_OPT_LEVEL=2 CARGO_PROFILE_TEST_OPT_LEVEL=2
fi
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-m3-formal-code-b35-r1 复跑在自己的仓副本与自己的编译目录里，不与别的会话共用副本}"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-上限按峰值表给不到：这一组测试目标组合峰值表里没有量过，给 8G 与 b34 正推同档}"
# 每条 cargo 经内存包装（8G）与线程上限 10
export RUNNER="nice -n 19 bash $REPO/research/scripts/run-with-memory-cap.sh 8G bash $REPO/research/scripts/capped.sh 10 cargo test --offline"
