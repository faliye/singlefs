#!/bin/bash
# E171 第 1 次重跑·修订 10：修订 8 那一趟（最终主线）第二段整表的跑法入口（登记 research/prompts/e171-r1-prereg.md 修订 10 第 10.2、10.3、10.9 节与
# 「主 agent 认定：修订 10 交回」）。281 行加补位，流一、流二，批 1 批 2，不跑执行器、不带探针，在修订 8 树（H3）上跑；编排交
# rows_orchestrator_segment_two_revision_10.py（它按 sha256 载入 rows_orchestrator_segment_two.py；修订 6 那一趟的第二段入口与编排层一个字节不改）。
# 起法（主 agent，在仓根；修订 8 那一趟第 0、1、2 步与研究一侧变异表（排了的话）绿之后）：
#   bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label e171-r1-s2-revision-8 -- bash <这一份> [<占位配置>]
#   不给配置取同目录 revision_10_placeholders.json（照登记第 10.6 节表填好的那一份；它点名修订 8 那一趟的占位配置）。同一条命令再起就是续跑。
#   PLAN_ONLY=1 bash <这一份> [<占位配置>]：干跑——占位核对（不过只报）、起跑闸、本机闸、让路、停派文件、T₂、现查的分配与推的各台挂钟、每一行驱动的命令；
#   一行都不起、不编、不写产物、导出根、日志与输入清单。
# long-run-shards: 按行切：整表 281 行每行一个单线道逐行驱动，行按推的用时最长先派到各台空出来的槽（rows_orchestrator_segment_two.py 文件头「派发」，第二段·修订 8 编排层照用）；本机等修订 8 第 7 步绿、逐行编排起了再加槽
# long-run-resume: 同一条命令再起就是续跑：开跑先核输入清单（<H23 导出根>/inputs.sha256 与 run-values.tsv）与 H3、H8，只补行目录里还没有 row-result.tsv 的行，别的台被停时取回的行先收进来、写到一半的重做
# long-run-hosts: 台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
# 多机的说明：每台槽数按每台线程与 scripts/hosts/host-resources.py --json 现查的内存额度算（起跑、续跑、每 600 秒给放不下的台重查内存加槽）
# 两份配置都由编排层核（revision_10_placeholders.py check --phase start 那一套：这一份的 H21–H29 与起跑闸，修订 8 那一份的 tree、H3、H8、H16、H18），
# 没填齐或核不过退 78 并点名；这一份入口只把配置交给编排层。
set -u
REPO=/home/fy5090/code/singlefs
cd "$REPO" || exit 2
RESEARCH=scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes
CONFIG=${1:-$RESEARCH/revision_10_placeholders.json}
export PYTHONDONTWRITEBYTECODE=1
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一会话派出去的几个执行员各跑各的装置"
if [ "${PLAN_ONLY:-0}" = 1 ]; then
  exec nice -n 19 python3 -B "$RESEARCH/rows_orchestrator_segment_two_revision_10.py" plan --config "$CONFIG" --repo "$REPO"
fi
exec nice -n 19 python3 -B "$RESEARCH/rows_orchestrator_segment_two_revision_10.py" run --config "$CONFIG" --repo "$REPO"
