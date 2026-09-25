# m2-final-code-r1 本地攻方运行记录

提示文件：`research/prompts/m2-final-code-r1-local-attack.md`
核对表：`research/prompts/m2-final-code-r1-local-attack-translation-audit.md`

| 调用号 | 命令 | 退出码 | 词数（corruption-check.py） | oov-check.py 生词 | 判定 |
|---|---|---|---|---|---|
| s1 | `bash research/scripts/ask-local.sh research/prompts/m2-final-code-r1-local-attack.md > research/prompts/m2-final-code-r1-local-attack-output-s1.md` | 0 | 622 | 生词=0 拼接=0 | 干净 |
| s2 | `bash research/scripts/ask-local.sh research/prompts/m2-final-code-r1-local-attack.md > research/prompts/m2-final-code-r1-local-attack-output-s2.md` | 0 | 415 | 生词=19（全部是同一个词 `Falsification`，完整英文词，非缺头粘连、非新造词，判定不算带损坏） | 干净 |

两份都判干净，第二次调用即凑够两份，未再往下抽样、无判红作废副本（无 `-output-void*.md`）。

corruption-check.py 两份都判绿：`cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0`。
另外通读两份样本全文一遍：未见缺词、断句（未见列表中整词缺失只剩孤零孤零标点的情形）。

样本自带的行号：两份样本正文里都没有出现代码行号或文件行号（提示第 3 条已明令不写，两份都照办），因此没有「模型自给、未核」需要标注的行号。
