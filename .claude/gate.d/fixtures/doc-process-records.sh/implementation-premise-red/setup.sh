#!/usr/bin/env bash
# implementation-premise 格的样本现场：这一份正文的标题日期要落在那一格的 SIDE_AND_SNAPSHOT_CUTOFF 及以后才走得到那一支；
# 日期在跑样本的临时目录里现写，取今天往后 30 天（总不早于那个截止日），不在任何样本文件里出现字面日期
# （写一个晚于今天的字面日期，门禁 code-tooling.sh 的 fixture-claims 格会判「日期不可能是真的」）。
# 同一个阶段的 abandoned-rounds 格在这份样本上也红（几份正文发过腿、没有判决）；expect 只认 implementation-premise 格自己的 ✗ 行。
set -euo pipefail
sample_date="$(TZ=Asia/Tokyo date -d '+30 days' +%F)"
cat > research/prompts/_c995-r1-body.md <<'SAMPLE_BODY'
# 背景材料：C995（新样本） 第一轮（SAMPLE_DATE）

| 实现今天的样子 | `crates/singlefs-core/src/mount.rs` 读过 | 主 agent 观测 |
SAMPLE_BODY
sed -i "s/SAMPLE_DATE/$sample_date/" research/prompts/_c995-r1-body.md
