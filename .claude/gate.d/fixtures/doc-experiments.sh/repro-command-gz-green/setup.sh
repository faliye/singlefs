#!/usr/bin/env bash
# 产物只以 .out.gz 存在（超过单文件推送上限、压成 .gz 入库的样子）：样本里放的是 .out，这里压成 .out.gz 并删掉原文件。
set -e
gzip -n -9 -f research/results/e96-sample-2026-10-21.out
