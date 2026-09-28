#!/usr/bin/env bash
# 格「同一结构的写频率只有一个量级」的绿样本。样本在仓里用不冲突的编号 D2452 存放（doc-lint 全仓查「一个编号只许一处登记位」），
# 跑的时候才改成 D1 —— 格「kb 形状」第 5 段要求决策编号从 D1 起连号；登记表的权威分项一起改。
# 不是 git 仓、没有 D15、没有一处「待用户复核」，看 git 的两格、冻结层归属登记表与待用户复核那四格退 77；
# 写频率那一格有登记表，判绿。
sed -i "s/D2452/D1/g" .claude/kb/decisions.md .claude/kb/decisions/01-写频率样本乙.md .claude/gate.d/write-frequency.tsv
