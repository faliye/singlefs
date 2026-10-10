#!/bin/bash
# E171 第 1 次重跑第一段·修订 8 那一趟（最终主线）的跑法入口：登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.4、8.9 节与其后「主 agent 认定：修订 8 交回」。
# 起法（主 agent，在仓根）：bash research/scripts/detached-run.sh start --wall-clock over-2h --label e171-r1-s1-revision-8 -- bash <这一份> [<占位配置>]
#   不给配置取同目录 revision_8_placeholders.json（拍树那一刻照登记第 8.8 节表填好的那一份）。同一条命令再起就是续跑。
#   PLAN_ONLY=1 bash <这一份> [<占位配置>]：干跑——只出占位与前提的核对、本机额度、单元表、依赖图、次序、各单元的命令与逐行编排的 plan，一个装置都不起、不编、不写产物。
# long-run-shards: 第 0–10 步按单元切：第 3、4 步每项、第 10 步每次各一个单元，与第 5、6、7、9 步同时跑（segment_one_units_revision_8.py 文件头「单元」）；第 11–13 步按行，每行一个单线道驱动，最长先派到空出来的槽（rows_orchestrator_revision_8.py 调第一段编排器）
# long-run-resume: 同一条命令再起就是续跑：第 0–10 步按单元（序列日志有绿行、产物在且行数同、单元绿时当场写下的清单 sha256 不变），没做完的单元连同下游重跑、名字带 -attempt<N>；第 11–13 步只补行目录里还没有 row-result.tsv 的行
# long-run-hosts: 台从 research/scripts/layer0-shard-configuration-check.sh --emit-hosts 读，每台槽数与每槽线程从 research/scripts/host-resources.py --json 现查现算（逐行编排起跑、续跑与每次加槽都重查；第 0–10 步的本机额度每次起跑、续跑现查），不写死
# 这一份只把配置交给单元编排：判法、阈值、限时、线程与内存的取法都在 segment_one_unit_revision_8.sh 与它照抄的修订 6 跑法那几行里；占位没填齐由单元编排开跑先核、退 78 并点名。
set -u
REPO=/home/fy5090/code/singlefs
cd "$REPO" || exit 2
RESEARCH=research/e7-index-bench/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes
CONFIG=${1:-$RESEARCH/revision_8_placeholders.json}
export PYTHONDONTWRITEBYTECODE=1
if [ "${PLAN_ONLY:-0}" = 1 ]; then
  exec nice -n 19 python3 -B "$RESEARCH/segment_one_units_revision_8.py" plan --config "$CONFIG" --repo "$REPO"
fi
exec nice -n 19 python3 -B "$RESEARCH/segment_one_units_revision_8.py" run --config "$CONFIG" --repo "$REPO"
