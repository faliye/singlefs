#!/usr/bin/env bash
# 样本：核数的一半（注释里的 nproc 不点名）
threads=$(( $(nproc) / 2 ))
echo "并行度按 nproc 取"
jobs=$(nproc)  # grown-row 样本：基准之后才加进排除表的那一行放行它
