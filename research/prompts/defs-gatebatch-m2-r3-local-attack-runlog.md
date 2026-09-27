# defs-gatebatch-m2-r3 本地攻方（L3）运行记录

提示文件：`research/prompts/defs-gatebatch-m2-r3-local-attack.md`
核对表：`research/prompts/defs-gatebatch-m2-r3-local-attack-translation-audit.md`

程序性偏差：第一次调用（s1）图省事先用前台跑了 `bash research/scripts/ask-local.sh`（未套 `run_in_background`），它在前台上限内就返回了退出码 0，没有被工具挪进后台，没有产生双写风险；之后一条（s2）改用规定写法。记在这里备查，不重跑 s1（结果本身干净，取号与文件都只写了一次）。

## 样本 s1
- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/defs-gatebatch-m2-r3-local-attack.md > research/prompts/defs-gatebatch-m2-r3-local-attack-output-s1.md`
- 退出码：0
- 词数：419（`wc -w`）
- `oov-check.py`（带提示文件）：`绿 … 生词=0 拼接=0`，退出码 0
- 逐段通读：58 行，未见粘连词、未见断句缺词、未见成对标记落单
- 判定：干净

## 样本 s2
- 命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/defs-gatebatch-m2-r3-local-attack.md > research/prompts/defs-gatebatch-m2-r3-local-attack-output-s2.md`
- 退出码：0
- 词数：507（`wc -w`）
- `oov-check.py`（带提示文件）：`绿 … 生词=0 拼接=0`，退出码 0
- 逐段通读：59 行，未见粘连词、未见断句缺词、未见成对标记落单
- 判定：干净

## 5b 算术核对
没有答案表，算术没比（这一轮 L3 提示里的表格题是「登记字段与源码逐格对照」的是否判定，不是主 agent 另给的写死数字答案表；主 agent 没有给这一类的答案表路径）。

## 汇总
干净样本 2 份（s1、s2），已达到「至少两份干净样本」的门槛，未触发「连续五次调用拿不到两份干净的」停下条款。没有作废（退出码 5）或带损坏（退出码非 0 非 5，或干净判定不成立）的调用。
