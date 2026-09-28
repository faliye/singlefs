#!/usr/bin/env bash
# 样本在仓里用不冲突的编号存放（doc-lint 全仓查「一个编号只许一处登记位」），
# 跑的时候才改成 D1、D2 —— 格「kb 形状」第 5 段要求决策编号从 D1 起连号。
sed -i "s/D94/D1/g" .claude/kb/decisions.md .claude/kb/decisions/01-样本.md
sed -i "s/D95/D2/g" .claude/kb/decisions/02-样本.md
sed -i "s/D96/D3/g" .claude/kb/decisions.md .claude/kb/decisions/03-样本.md
# 一份读不了的规则文件：悬空的符号链接。grep 读它退 2，本该判红，而不是把报错丢进 /dev/null 当成没命中。
# 放在 .claude/rules/ 下而不是 kb 下：第 3 段的 python 读 kb 下每一份，悬空链接会让它整段跑不完，盖掉那一段的判定。
mkdir -p .claude/rules
ln -s 不存在的规则.md .claude/rules/悬空.md
