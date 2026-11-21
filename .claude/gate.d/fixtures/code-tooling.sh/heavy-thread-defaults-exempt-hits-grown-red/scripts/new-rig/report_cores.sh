#!/usr/bin/env bash
# 样本：基准那一版只有 cores 那一行 $(nproc)；这一版多了 shown 那一行，表还是那一行
cores=$(nproc)
shown=$(nproc)
printf '本机 %s 核、%s 核\n' "$cores" "$shown"
