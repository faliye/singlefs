#!/bin/bash
# E171 第 1 次重跑·修订 12：修订 8 那一趟第一段（第 0–13 步）带复用的跑法入口（登记 research/prompts/e171-r1-prereg.md 修订 12 第 11.10、11.11 节
# 与其后「主 agent 认定：修订 12 交回」）。修订 8 那一份入口 run_segment_one_on_the_revision_8_tree_multi_host.sh 一个字节不改。
# 起法（主 agent，在仓根；H30 认过、H31 填好认过之后）：bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label e171-r1-s1-revision-11 -- bash <这一份> [<修订 12 占位>]
#   不给配置取同目录 revision_11_placeholders.json（它点名修订 8 那一份占位）。同一条命令再起就是续跑。
#   PLAN_ONLY=1 bash <这一份> [<修订 12 占位>]：干跑——修订 8 的 plan 原样，另打每个读数单元的键与库里有没有；一个装置都不起、不编、不写产物、不建库、不写标记。
# long-run-shards: 同修订 8：第 0–10 步按单元切（第 3、4 步每项、第 10 步每次各一个单元，另加 reuse-selfproof），第 11–13 步按行，每行一个单线道驱动；命中复用库的读数单元在单元脚本里取用、不起装置，命中的行不进派发队列（segment_one_units_revision_11.py、rows_orchestrator_revision_11.py 文件头）
# long-run-resume: 同一条命令再起就是续跑：第 0–10 步按单元（序列日志有绿行、产物在且行数同、清单 sha256 不变），第 11–13 步只补行目录里还没有 row-result.tsv 的行；取用断在中间的照标记删掉重取，入库经 staging/ 加 rename
# long-run-hosts: 台数从 scripts/crash/layer0-shard-configuration-check.sh --emit-hosts 读，每台线程取登记值 .claude/heavy-thread-default（经 capped.sh default）
# 这一份只把配置交给单元编排：H31 没填或对不上、库目录写不了、修订 8 那一份占位没填齐，都由编排开跑先核、退 78 并点名。
set -u
REPO=/home/fy5090/code/singlefs
cd "$REPO" || exit 2
RESEARCH=scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes
CONFIG=${1:-$RESEARCH/revision_11_placeholders.json}
export PYTHONDONTWRITEBYTECODE=1
if [ "${PLAN_ONLY:-0}" = 1 ]; then
  exec nice -n 19 python3 -B "$RESEARCH/segment_one_units_revision_11.py" plan --config "$CONFIG" --repo "$REPO"
fi
exec nice -n 19 python3 -B "$RESEARCH/segment_one_units_revision_11.py" run --config "$CONFIG" --repo "$REPO"
