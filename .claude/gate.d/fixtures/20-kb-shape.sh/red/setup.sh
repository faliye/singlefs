#!/usr/bin/env bash
# 样本在仓里用不冲突的编号存放（doc-lint 全仓查「一个编号只许一处登记位」），
# 跑的时候才改成 D1 —— 20-kb-shape 第 5 段要求决策编号从 D1 起连号。
sed -i "s/D94/D1/g" .claude/kb/decisions.md .claude/kb/decisions/01-样本.md
