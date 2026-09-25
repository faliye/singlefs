# m2-safety-r1 本地攻方：运行记录

提示文件：`research/prompts/m2-safety-r1-local-attack.md`
译文核对表：`research/prompts/m2-safety-r1-local-attack-translation-audit.md`

调用方式：`bash research/scripts/ask-local.sh research/prompts/m2-safety-r1-local-attack.md > research/prompts/m2-safety-r1-local-attack-output-s<n>.md`，前台跑，未用 `setsid`/`&`/`disown`。

| 样本 | 退出码 | 词数（`wc -w`） | 行数（`wc -l`） | `oov-check.py` 结果 | 判定 |
|---|---|---|---|---|---|
| s1 | 0 | 1621 | 165 | `绿 ... 生词=0 拼接=0`（退出码 0） | 干净 |
| s2 | 0 | 1489 | 201 | `绿 ... 生词=0 拼接=0`（退出码 0） | 干净 |

两次调用都是退出码 0 的合法样本，没有退出码 5 的作废副本，没有跳号。`ask-local.sh` 内部串跑的 `corruption-check.py` 与 `oov-check.py` 在两次调用里都判绿（脚本退出码 0，不是「没跑成」）；额外又对两份样本各跑一遍 `oov-check.py`，结果同上。

另外通读两份样本全文，找缺词、断句一类闸检测不到的损坏（例如列表里整词消失、只剩孤零零的标点）：两份都没有这类现象，句子与表格都完整、没有孤立标点。

s2 在答案里用了 markdown 强调（`**Row A1:**`、`### Item 1` 这类粗体与标题），与提示第 472-476 行「不许用 markdown 强调」的指令不符；这不属于闸判的四类字词损坏（复读、成对标记落单、拼接、实词自复读）之一——corruption-check.py 判的是「成对标记有没有落单」，s2 里的 `**` 每一处都成对出现，闸判绿；这里只是如实记录 s2 没照办「不用 markdown」这条指令，不改判它「带损坏」。s1 全文没有使用任何 markdown 强调，按提示的字面要求作答。

样本自带的行号（`wc -l` 数出的行数、样本正文里出现的 t 值序号等）均为模型自给、未核，不当依据引用。

不写样本答了什么、几份样本方向是否一致——按定义不归本腿的职责。
