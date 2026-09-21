# archive-rename-r1 本地攻方腿：运行记录

提示文件：`research/prompts/archive-rename-r1-local-attack.md`（英文，J5、J7，两组问题）。
核对表：`research/prompts/archive-rename-r1-local-attack-translation-audit.md`。
调用方式：前台 `nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <样本>`，
没有用 `setsid`/`&`/`disown`。共调用 2 次，2 次都在第一次尝试就拿到干净样本，未触发退出码 5，
没有作废副本（`-output-void*.md` 不存在，已用 `ls` 核过）。

| 样本 | 退出码 | 词数（`wc -w`） | `oov-check.py` 生词清单 | 判定 |
|---|---|---|---|---|
| `archive-rename-r1-local-attack-output-s1.md` | 0 | 438 | discriminate, exemption | 干净 |
| `archive-rename-r1-local-attack-output-s2.md` | 0 | 667 | discriminate, organizations', exemption | 干净 |

两份样本的 `oov-check.py` 都判绿（退出码 0，`拼接=0`），生词清单里的词（discriminate、exemption、organizations'）
都是完整英文词，不是缺头或粘连的形态（对照 `.claude/rules/three-way-inference.md`「损坏有四类」逐类看过：
没有复读、没有成对标记落单、没有拼接、没有实词自复读）。另外通读了两份样本全文：
编号 1–7 逐条齐整，没有缺词、没有断句只剩孤零零标点的情况，两份都记「干净」。

`ask-local.sh` 内部串跑的 `corruption-check.py`、`oov-check.py` 两道闸在两次调用里都判绿（退出码 0 直接落盘，
没有走到「判红作废」那条分支），本次另外单独重跑了一遍 `oov-check.py` 核对，结果与内部闸一致。
