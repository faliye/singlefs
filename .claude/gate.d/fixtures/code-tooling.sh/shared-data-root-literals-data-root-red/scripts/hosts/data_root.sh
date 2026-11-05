#!/usr/bin/env bash
# 样本：数据根本身、名单外的名字、两样运行记录（四处都该点名）
data="${SINGLEFS_DATA_ROOT:-$HOME/.local/share/singlefs}"
extra=~/.local/share/singlefs/unlisted-sample-records
amplification="${SAMPLE_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
peaks="$HOME/.local/share/singlefs/memory-peaks.tsv"
ls "$data" "$extra" "$amplification" "$peaks"
