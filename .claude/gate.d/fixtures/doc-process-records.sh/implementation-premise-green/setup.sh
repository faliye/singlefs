#!/usr/bin/env bash
# implementation-premise 格的样本现场：这一份正文的标题日期要落在那一格的 SIDE_AND_SNAPSHOT_CUTOFF 及以后才走得到那一支；
# 日期在跑样本的临时目录里现写，取今天往后 30 天（总不早于那个截止日），不在任何样本文件里出现字面日期
# （写一个晚于今天的字面日期，门禁 code-tooling.sh 的 fixture-claims 格会判「日期不可能是真的」）。
# 同一个阶段的 abandoned-rounds 格要这几轮有去向：research/prompts/abandoned-rounds.tsv 把三轮登记成撂下（腿一条没派），它在这里判绿。
set -euo pipefail
sample_date="$(TZ=Asia/Tokyo date -d '+30 days' +%F)"
cat > research/prompts/_c996-r1-body.md <<'SAMPLE_BODY'
# 背景材料：C996（新样本） 第一轮（SAMPLE_DATE）

| 前提 | 内容 | 出处 |
|---|---|---|
| 实现今天的样子 | `crates/singlefs-core/src/mount.rs` 读过 | 主 agent 观测 |

分工：本地腿：辩（这一轮复核上一轮判决，缺辩方）
SAMPLE_BODY
sed -i "s/SAMPLE_DATE/$sample_date/" research/prompts/_c996-r1-body.md
