#!/usr/bin/env bash
# 门禁 58 号的样本现场：这一份正文的标题日期要落在 58 号的 SIDE_AND_SNAPSHOT_CUTOFF 及以后才走得到那一支，
# 而那个截止日晚于今天；把日期字面写进样本，门禁 95 号会判「日期不可能是真的」。所以在跑样本的临时目录里现写，
# 日期取今天往后 30 天（总不早于那个截止日），不在任何样本文件里出现字面日期。
set -euo pipefail
sample_date="$(date -u -d '+30 days' +%Y-%m-%d)"
cat > research/prompts/_c996-r1-body.md <<'SAMPLE_BODY'
# 背景材料：C996（新样本） 第一轮（SAMPLE_DATE）

| 前提 | 内容 | 出处 |
|---|---|---|
| 实现今天的样子 | `crates/singlefs-core/src/mount.rs` 读过 | 主 agent 观测 |

分工：本地腿：辩（这一轮复核上一轮判决，缺辩方）
SAMPLE_BODY
sed -i "s/SAMPLE_DATE/$sample_date/" research/prompts/_c996-r1-body.md
