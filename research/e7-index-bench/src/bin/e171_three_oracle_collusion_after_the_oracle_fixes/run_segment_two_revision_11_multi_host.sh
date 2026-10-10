#!/bin/bash
# E171 第 1 次重跑·修订 11：修订 8 那一趟第二段整表带复用的跑法入口（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.10、11.11 节与
# 「主 agent 认定：修订 11 交回」）。修订 10 那一份入口 run_segment_two_revision_10_multi_host.sh 一个字节不改；编排交 rows_orchestrator_segment_two_revision_11.py
# （它按 sha256 载入修订 10 那一层，修订 10 的起跑闸、本机闸、让路、停派文件、输入清单、S17 联动原样）。
# 起法（主 agent，在仓根；修订 8 那一趟第 0、1、2 步与研究一侧变异表（排了的话）绿、H30 认过、H31 填好认过之后）：
#   bash research/scripts/detached-run.sh start --wall-clock over-2h --label e171-r1-s2-revision-11 -- bash <这一份> [<修订 11 占位>]
#   不给配置取同目录 revision_11_placeholders.json（它点名修订 10 那一份占位，修订 10 那一份再点名修订 8 那一份）。同一条命令再起就是续跑。
#   PLAN_ONLY=1 bash <这一份> [<修订 11 占位>]：干跑——先打整表每行永远现跑与否，再交修订 10 的 plan（T₂ 定了时另打每行的键与库里有没有）；
#   一行都不起、不编、不写产物、导出根、日志、输入清单，不建库、不写标记。
# long-run-shards: 按行切：整表 281 行每行一个单线道逐行驱动，行按推的用时最长先派到各台空出来的槽（同修订 10）；命中复用库的行（RC3 抽到的除外）取用、不进派发队列
# long-run-resume: 同一条命令再起就是续跑：开跑先核输入清单与 H3、H8（修订 10 原样），只补行目录里还没有 row-result.tsv 的行；RC3 挑中的行照第一次挑的那份续用；取用断在中间的照标记删掉重取
# long-run-hosts: 台从 research/scripts/layer0-shard-configuration-check.sh --emit-hosts 读，每台槽数与每槽线程从 research/scripts/host-resources.py --json 现查现算（起跑、续跑、每 600 秒给放不下的台重查加槽），不写死；复用库只在本机，别的台不查库、不留库
set -u
REPO=/home/fy5090/code/singlefs
cd "$REPO" || exit 2
RESEARCH=research/e7-index-bench/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes
CONFIG=${1:-$RESEARCH/revision_11_placeholders.json}
export PYTHONDONTWRITEBYTECODE=1
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一会话派出去的几个执行员各跑各的装置"
if [ "${PLAN_ONLY:-0}" = 1 ]; then
  exec nice -n 19 python3 -B "$RESEARCH/rows_orchestrator_segment_two_revision_11.py" plan --config "$CONFIG" --repo "$REPO"
fi
exec nice -n 19 python3 -B "$RESEARCH/rows_orchestrator_segment_two_revision_11.py" run --config "$CONFIG" --repo "$REPO"
