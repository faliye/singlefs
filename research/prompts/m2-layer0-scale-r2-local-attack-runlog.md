# m2-layer0-scale-r2 本地攻方腿：运行记录

提示文件：`research/prompts/m2-layer0-scale-r2-local-attack.md`
核对表：`research/prompts/m2-layer0-scale-r2-local-attack-translation-audit.md`

## 调用记录

第 1 次调用：`bash research/scripts/ask-local.sh research/prompts/m2-layer0-scale-r2-local-attack.md > .../m2-layer0-scale-r2-local-attack-output-s1.md`。前台起跑，因单次调用超过 240 秒被工具自动移到后台（未使用 setsid / & / disown，是工具自身的前台超时移交），完成通知退出码 0，占号 s1。词数（`wc -w`）514。`oov-check.py` 判绿：生词 6（去重后 1 个：`Falsified`），拼接 0。`corruption-check.py` 判绿（提示文件本身与模型答复两份都判绿，cjk=0 words=434 全项 0）。逐行通读：54 行分号未见断裂、Question 3 的 55 行位置表齐全（position 1–55 全部出现，无缺行、无孤立标点）、Question 2 逐词分类完整、Question 4/5/6 散文完整、每句主谓宾齐全，未见缺头粘连词。判定：干净。

第 2 次调用：`bash research/scripts/ask-local.sh research/prompts/m2-layer0-scale-r2-local-attack.md > .../m2-layer0-scale-r2-local-attack-output-s2.md`，前台跑完，退出码 0，占号 s2。词数（`wc -w`）210，明显短于 s1。`oov-check.py` 判绿：生词 6（去重后 1 个：`Falsified`），拼接 0。`corruption-check.py` 判绿（cjk=0 words=187 全项 0）。逐行通读：六个问题各有一句答复加一行「Falsified by:」，句子均主谓宾齐全，没有列表项整词消失只剩孤立标点的情形，也没有缺头粘连词；Question 3、Question 5 没有按题目要求给出 55 行/多行明细表，只给了汇总数字——这是内容详略上的差异（模型选择不展开逐行明细），不是词形损坏的迹象（无缺词断句、无粘连、两道自动闸门与人工通读都未见对应损坏信号）。判定：干净。

## 汇总

- 干净：s1、s2（2 份，达到「至少两份干净样本」的停机条件，共调用 2 次、0 次作废）。
- 带损坏：无。
- 作废：无。
- 核对表：`research/prompts/m2-layer0-scale-r2-local-attack-translation-audit.md`。
