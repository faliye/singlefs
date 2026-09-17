# 运行记录：agent-defs-r2 本地攻方

提示文件：`research/prompts/agent-defs-r2-local-attack.md`（英文，J8「方法」攻击面，五问；不攻单个定义的条文）。
核对表：`research/prompts/agent-defs-r2-local-attack-translation-audit.md`。
全部调用前台跑（`bash research/scripts/ask-local.sh ...`），未用 `setsid` / `&` / `disown`。

## 五次调用，逐次记录

| 次序 | 目标文件 | `ask-local.sh` 退出码 | 词数 | `oov-check.py` 结果 | 判定 |
|---|---|---|---|---|---|
| 1 | `agent-defs-r2-local-attack-output-s1.md` | 0 | 686 | 绿，生词=1：`definition's`（正常所有格，非粘连） | 干净 |
| 2 | `agent-defs-r2-local-attack-output-s2.md`（第一次） | 5 | 0（重定向出的文件为空） | 未跑（判红即作废，不进 oov 复核） | 作废；原样输出另存 `agent-defs-r2-local-attack-output-void1.md`（745 词，拼接：`encounteringwriting(=encountering+writing)`） |
| 3 | `agent-defs-r2-local-attack-output-s2.md`（第二次，沿用同一个号） | 5 | 0 | 未跑 | 作废；原样输出另存 `agent-defs-r2-local-attack-output-void2.md`（837 词，拼接：`functioned(=function+ed)`——`functioned` 本身是正常英语词，判红的判据是「切得开」不是「像不像词」，按规则不越权改判，照原样作废） |
| 4 | `agent-defs-r2-local-attack-output-s2.md`（第三次，沿用同一个号） | 0 | 840 | 绿，生词=2：`disposable`（正常词，只是不在词表里，且已在提示原文里出现）、`discrepanciesal`（出现在第 13 行「the trial run only used a list with no discrepanciesal errors」，切不出闸能识别的两个词表词，但形态可疑，判不清是新造词还是缺头粘连词） | **带损坏，不当干净样本**（按规则「拿不准是新造词还是粘连的，一律记带损坏」）；文件保留、不删、不覆盖 |
| 5 | `agent-defs-r2-local-attack-output-s3.md` | 0 | 764 | 绿，生词=7（去重后 3 类）：`disposable`（同上，正常词）、`confidential'`（模型在正文里给 `'confidential'` 加了引号，OOV 检测把结尾的引号一起算进词干，非粘连）、`suppressive`（正常词，只是不在词表里） | 干净 |

## 逐份人工通读（闸与 `oov-check.py` 都抓不到的缺词、断句）

- s1：686 词，五问各一段，句子完整，没有缺词或孤立标点，结尾句式一致（`This would be refuted if...`）。
- s2（第三次，带损坏样本）：除 `discrepanciesal` 外通读无缺词、无断句；但该词已单独记为带损坏，整份不当干净样本。
- s3：764 词，五问各一段，句子完整，列表（第 5 问三种情形）逐条完整，没有孤零零的标点或缺词。

## 干净样本汇总

两份：`agent-defs-r2-local-attack-output-s1.md`、`agent-defs-r2-local-attack-output-s3.md`。达到「至少两份干净样本」，停止取样。

五次调用（K5：判红作废的算一次；本轮另加一次「判绿但人工复核判带损坏」，同样不算干净）不到五次连续拿不到两份干净——第 5 次即凑齐两份，未触发「连续五次拿不到两份就停下报告」的停机条款。

## 没做什么

- 不解读、不总结、不采纳本地模型对五个问题给出的答案；打没打中由主 agent 判。
- 没有用 `ASK_LOCAL_ALLOW_CORRUPT=1` 强行采用任何一份判红或带损坏的输出。
- 没有跑第六次或更多次调用（已凑够两份干净样本）。
- 没有修改 `research/scripts/ask-local.sh`、`oov-check.py`、`corruption-check.py` 本身。
