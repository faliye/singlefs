# agentdef-r3 本地攻方 运行记录

只攻 H 格（候选表漏召回）。提示文件：`research/prompts/agentdef-r3-local-attack.md`。核对表：`research/prompts/agentdef-r3-local-attack-translation-audit.md`。

## 调用记录

| 样本 | 调用命令 | 退出码 | 作废副本 |
|---|---|---|---|
| s1 | `bash research/scripts/ask-local.sh research/prompts/agentdef-r3-local-attack.md > research/prompts/agentdef-r3-local-attack-output-s1.md` | 0 | 无 |
| s2 | `bash research/scripts/ask-local.sh research/prompts/agentdef-r3-local-attack.md > research/prompts/agentdef-r3-local-attack-output-s2.md` | 0 | 无 |

两次调用退出码均为 0，一次都没判红，没有产生 `-output-void*.md`。连续尝试次数 2（未到「连续五次拿不到两份干净的」停手线）。

## 逐份质检

| 样本 | 词数（`wc -w`） | `oov-check.py` | `corruption-check.py` | 判定 |
|---|---|---|---|---|
| s1 | 276 | 绿，生词=0 拼接=0 | 绿，cjk=0 words=260 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 | 干净 |
| s2 | 277 | 绿，生词=0 拼接=0 | 绿，cjk=0 words=263 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 | 干净 |

两次 `oov-check.py` 都列出生词=0（没有需要人工复核的缺头粘连词）。两份都另外通读一遍全文：没有发现缺词、断句、列表项整个消失只剩孤零零标点这类损坏；两份第一行都单独一行写着裸整数 `1`（对应 Q1「先单独给出最小整数」那句指令的字面执行），不算断句损坏。判定：两份都记「干净」，不算「带损坏」。

已攒够至少两份干净样本，按定义第 6 条停止取样，不再续跑。

## 产出路径

- 提示文件：`research/prompts/agentdef-r3-local-attack.md`
- 核对表：`research/prompts/agentdef-r3-local-attack-translation-audit.md`
- 样本：`research/prompts/agentdef-r3-local-attack-output-s1.md`、`research/prompts/agentdef-r3-local-attack-output-s2.md`
- 本运行记录：`research/prompts/agentdef-r3-local-attack-runlog.md`

## 没做什么

- 不解读、不总结、不采纳两份样本的具体答复内容，也不判两份样本方向是否一致——判它打没打中是主 agent 的事，本记录只报退出码、词数、生词清单、干净/带损坏/作废。
- 没有再多跑第三次：定义只要求「攒到至少两份干净样本为止」，前两次都干净就停，不额外抽样。
