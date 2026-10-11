#!/bin/bash
# E171 第 1 次重跑第二段（整表）·修订 6 树·多机跑法（2026-10-07 写）：登记 research/prompts/e171-r1-prereg.md 第五节 5.5 第二段、修订 2 第 2.5 节、
# 修订 6 第 6.6 节「第二段（整表）照原文，在修订 6 树上跑」。281 行加补位，流一、流二，批 1 批 2，不跑执行器、不带探针；编排交 rows_orchestrator_segment_two.py
# （文件头写行集、分配、派发、补位协调、合并、判法、跨机对照、超时重跑、续跑）。第一段那一份入口 run_segment_one_on_the_revision_6_tree_multi_host.sh 不改。
# 起法（主 agent，仓根）：bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label e171-r1-s2-revision-6 -- bash scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_two_on_the_revision_6_tree_multi_host.sh
#   同一条命令就是续跑命令。只看分配与推的挂钟、不起任何一行：PLAN_ONLY=1 bash <这一份>（不写导出目录、不写清单）。
# long-run-shards: 按行切：整表 281 行每行一个单线道逐行驱动，行按推的用时最长先派到各台空出来的槽（rows_orchestrator_segment_two.py 文件头「派发」）
# long-run-resume: 同一条命令再起就是续跑：开跑先核输入清单与第一趟记下的相同，只补行目录里还没有 row-result.tsv 的行，别的台被停时取回的行先收进来、写到一半的重做，已装的产物不重合
# long-run-hosts: 台从 scripts/crash/layer0-shard-configuration-check.sh --emit-hosts 读，每台槽数与每槽线程从 scripts/hosts/host-resources.py --json 现查现算（起跑时、续跑时；起跑时放不下的台每 10 分钟重查），不写死
# 环境变量：RUN_SUFFIX（默认 -r6；整表从头另跑一趟给 -r6-r2 这类，导出目录跟着另起）、RUN_DATE（产物名里的日期：第一趟记进导出目录的 run-date，续跑照它；
#   给了而与记下的不同就拒）、PLAN_ONLY=1（见上）。
# 输入清单（第一趟开跑时写进 $DUMPS/inputs.sha256，续跑先 sha256sum -c）：逐行驱动与它引的模块、两张行表、两份编排器、别的台那一头、判法、基线计时、冻结树里的变异表。
#   清单对不上就停：续跑会把两份驱动跑的行混进一份产物。冻结树另由快照摘要核（作废 Z5，编排器每行起止各核一次）。
set -u
REPO=/home/fy5090/code/singlefs
cd "$REPO" || exit 2
SCRATCH=/home/fy5090/.local/share/singlefs/work/e171-r1-segment2-multi-host
SUFFIX=${RUN_SUFFIX:--r6}
DUMPS=scripts/experiments/target/e171-r1-dumps/revision-6-segment-2${SUFFIX#-r6}
TREE=scripts/experiments/target/e171-r1-frozen-tree-revision-6
SNAPSHOT_DIGEST=7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4
RESEARCH=scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes
ORCHESTRATOR=$RESEARCH/rows_orchestrator_segment_two.py
CHECK=$RESEARCH/check_product.py
# 修订 2 第 2.5 节第二段：T 取修订 6 树上 S2 那一趟新装置基线（--streams 1,2、不带探针）的进程挂钟（这一份第 25 行 process_wall seconds=75.274）
BASELINE_TIMING=research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-s2-device-baseline-streams-1-2-r6-r3-timing.out
BASELINE_TIMING_SHA256=1e19e717be8fe913e099d24a59a54b0ce55bfafa230777bcdafe0909b8403a91
FROZEN_MUTATIONS_SHA256=4140c7847b56fa710755aebb0180a673cab6dd7f748297f6a95b4e2db7150cb8
INPUTS=("$RESEARCH/row_driver.py" "$RESEARCH/device_timing.py" "$RESEARCH/frozen_o3.py" "$RESEARCH/run_directory.py" "$RESEARCH/bad_implementation_table.py"
        "$RESEARCH/bad-implementations-117.tsv" "$RESEARCH/bad-implementations-expansion-164.tsv" "$RESEARCH/rows_orchestrator.py" "$ORCHESTRATOR"
        "$RESEARCH/peer_side.py" "$CHECK" "$BASELINE_TIMING" "$TREE/crates/mutations.tsv")
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一会话派出去的几个执行员各跑各的装置"
export PYTHONDONTWRITEBYTECODE=1

if [ -n "${PLAN_ONLY:-}" ]; then
  DATE=${RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
  exec nice -n 19 python3 -B "$ORCHESTRATOR" plan --repo "$REPO" --prefix "research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-$DATE-s2" \
    --suffix="$SUFFIX" --dumps "$DUMPS" --tree "$TREE" --snapshot-digest "$SNAPSHOT_DIGEST" --baseline-timing "$BASELINE_TIMING" --check "$CHECK" \
    --logs "$SCRATCH/plan-logs" --work-root "$SCRATCH/plan-work"
fi

mkdir -p "$DUMPS"
if [ -s "$DUMPS/run-date" ]; then
  DATE=$(cat "$DUMPS/run-date")
  if [ -n "${RUN_DATE:-}" ] && [ "$RUN_DATE" != "$DATE" ]; then echo "✗ RUN_DATE=$RUN_DATE 与 $DUMPS/run-date 记下的 $DATE 不同（续跑照记下的）" >&2; exit 2; fi
else
  DATE=${RUN_DATE:-$(TZ=Asia/Tokyo date +%F)}
  ( set -o noclobber; echo "$DATE" > "$DUMPS/run-date" ) || { echo "✗ 写不了 $DUMPS/run-date" >&2; exit 2; }
fi
PREFIX=research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-$DATE-s2
LOGS=$SCRATCH/sequence-segment-2-$DATE$SUFFIX
WORK_ROOT=$SCRATCH/rows-work$SUFFIX
mkdir -p "$LOGS"

say() { echo "$(TZ=Asia/Tokyo date +%F) $*" | tee -a "$LOGS/sequence.log"; }
stop() { say "✗ 停在前提：$*"; exit 1; }

say "开跑：DATE=$DATE SUFFIX=$SUFFIX 导出 $DUMPS 日志 $LOGS 产物前缀 $PREFIX"
digest=$(python3 -B -c "import sys; sys.path.insert(0, '$RESEARCH'); import row_driver; print(row_driver.snapshot_digest('$TREE')[0])")
[ "$digest" = "$SNAPSHOT_DIGEST" ] || stop "冻结树快照摘要变了（Z5）：$digest"
[ "$(sha256sum "$BASELINE_TIMING" | cut -d' ' -f1)" = "$BASELINE_TIMING_SHA256" ] || stop "基线计时不是修订 6 树 S2 那一份：$BASELINE_TIMING"
[ "$(sha256sum "$TREE/crates/mutations.tsv" | cut -d' ' -f1)" = "$FROZEN_MUTATIONS_SHA256" ] || stop "冻结树里的变异表变了：$TREE/crates/mutations.tsv"
if [ -s "$DUMPS/inputs.sha256" ]; then
  sha256sum --quiet -c "$DUMPS/inputs.sha256" >> "$LOGS/sequence.log" 2>&1 || stop "输入清单对不上第一趟记下的 $DUMPS/inputs.sha256（见 $LOGS/sequence.log）"
  say "  输入清单与第一趟相同（$(grep -c '' "$DUMPS/inputs.sha256") 个文件）"
else
  ( set -o noclobber; sha256sum "${INPUTS[@]}" > "$DUMPS/inputs.sha256" ) || stop "写不了输入清单 $DUMPS/inputs.sha256"
  say "  输入清单记下：$DUMPS/inputs.sha256（$(grep -c '' "$DUMPS/inputs.sha256") 个文件）"
fi

say "▶ 第二段整表：多机编排 $ORCHESTRATOR（行输出根 $DUMPS/segment2-rows，分片 $DUMPS/segment2-rows-shards）"
nice -n 19 python3 -B "$ORCHESTRATOR" run --repo "$REPO" --prefix "$PREFIX" --suffix="$SUFFIX" --dumps "$DUMPS" --tree "$TREE" \
  --snapshot-digest "$SNAPSHOT_DIGEST" --baseline-timing "$BASELINE_TIMING" --check "$CHECK" --logs "$LOGS" --work-root "$WORK_ROOT" &
ORCHESTRATOR_PID=$!
trap 'say "收到停止信号：等编排器收尾（停各台在跑的行、取回别的台已做完的行）"' TERM INT HUP
while :; do  # 信号打断 wait 时编排器还在：接着等它收完尾，拿它真的退出码
  wait "$ORCHESTRATOR_PID"; ORCHESTRATOR_EXIT=$?
  kill -0 "$ORCHESTRATOR_PID" 2>/dev/null || break
done
trap - TERM INT HUP
case "$ORCHESTRATOR_EXIT" in
  0) ;;
  78) say "✗ 运行条件不满足（见上面 ✗ 那一行；修好那台或改多机配置，再用同一条命令起）"; exit 78 ;;
  143) say "✗ 被停（同一条命令再起就是续跑）"; exit 143 ;;
  *) say "✗ 多机编排退 $ORCHESTRATOR_EXIT（见上面 ✗ 那一行）"; exit 1 ;;
esac
say "✓ 第二段整表跑完。比对器归下一段执行员；产物前缀 $PREFIX，行输出根 $DUMPS/segment2-rows，日志 $LOGS"
exit 0
