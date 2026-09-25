# m2-witness-r1 本地攻方运行记录

提示文件：`research/prompts/m2-witness-r1-local-attack.md`
核对表：`research/prompts/m2-witness-r1-local-attack-translation-audit.md`
调用方式：前台 `bash research/scripts/ask-local.sh research/prompts/m2-witness-r1-local-attack.md > <样本>`，未用 `setsid` / `&` / `disown`。线程上限 4（本次调用不跑编译、变异或产物，未触发 `capped.sh`）。

## 逐次调用

| 调用序号 | 样本文件 | ask-local.sh 退出码 | 词数（corruption-check.py 的 words 字段） | oov-check.py 生词 | 判定 |
|---|---|---|---|---|---|
| 1 | `research/prompts/m2-witness-r1-local-attack-output-s1.md` | 0 | 607 | 无（生词=0，拼接=0） | 干净 |
| 2 | `research/prompts/m2-witness-r1-local-attack-output-s2.md` | 0 | 595 | witnesses（生词=1，拼接=0） | 干净——`witnesses` 是 F5/候选定义里就在用的常规英文复数词，不是缺头粘连（如 `achievesceives`）那一类；`corruption-check.py` 同一份样本 `实词自复读=0`，另通读全文未见缺词、断句或列表里孤零标点那类损坏 |

两次调用都是退出码 0，没有触发退出码 5（作废）或退出码非 0/非 5 的情形，没有作废副本（`-output-void*.md`）产生。

## 干净样本数

2 份（s1、s2），已达到「至少两份干净样本」的下限，未继续第 3 次调用。

## 没做什么

- 未触发本地腿缺席（网关两次调用都在 900 秒超时内返回，退出码均为 0）。
- 未产生作废样本，因此没有沿用同一个号重跑的情形。
- 样本里的行号（例如它自己写的算式顺序）一律未核、未在本记录或核对表里当成权威引用。
