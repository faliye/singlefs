# 本地攻方运行记录：c363b-r1（V3）

提示文件：`research/prompts/c363b-r1-local-attack.md`（英文，自足，无 markdown 强调；FACT A 到 FACT H2 共写死的事实表若干条，逐条标来源文件:行——只覆盖 V3 树高与 ckpt_cost 算术，不碰 V1（固定点分配失败走不走得到）与 V2（准入放行推不推得出分配成功、Σ 罩哪几棵这一判断本身），提示正文 SCOPE 一节显式排除这两题）。
核对表：`research/prompts/c363b-r1-local-attack-translation-audit.md`（表一 5 条 kb 引文逐句核对，表二给提示里每个 FACT 编号的来源文件:行）。

两次前台调用（未用 setsid / 后台 / disown），命令一致：
`nice -n 19 bash research/scripts/ask-local.sh research/prompts/c363b-r1-local-attack.md > research/prompts/c363b-r1-local-attack-output-s<n>.md`

## 样本 1：`c363b-r1-local-attack-output-s1.md`

- 退出码：0
- 词数：`wc -w` = 1081
- `oov-check.py`（对样本与提示文件一起跑）：绿，生词=0，拼接=0，退出码 0
- 通读结果：28 项答案齐全（Q1.1-Q1.5、Q2.1-Q2.5、Q3a.1-Q3a.3、Q3b、Q4-HEIGHTS 五档、Q4-SUMS 五档）；未见缺词、断句或孤零标点；每项自带的「This would be refuted by」句齐全
- 判定：干净

## 样本 2：`c363b-r1-local-attack-output-s2.md`

- 退出码：0
- 词数：`wc -w` = 1239
- `oov-check.py`（对样本与提示文件一起跑）：绿，生词=0，拼接=0，退出码 0
- 通读结果：28 项答案齐全；未见缺词、断句或孤零标点；每项自带的「This would be refuted by」句齐全（Q3a.3 的 HEIGHT IF VALID 一格留空，属于该行判定「无有效状态」时的预期空白，非损坏）
- 判定：干净

## 样本数与停止条件

两次前台调用（均退出码 0，无一次判红作废）取得两份干净样本（样本 1、样本 2），达到「至少两份干净样本」的下限。已停止，不再调用 `ask-local.sh`。
