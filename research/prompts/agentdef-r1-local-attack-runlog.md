# agentdef-r1 本地攻方 运行记录

提示文件：`research/prompts/agentdef-r1-local-attack.md`
核对表（逐句核转述）：`research/prompts/agentdef-r1-local-attack-translation-audit.md`
调用方式：前台 `nice -n 19 bash research/scripts/ask-local.sh research/prompts/agentdef-r1-local-attack.md > research/prompts/agentdef-r1-local-attack-output-sN.md`，未用 `setsid` / `&` / `disown`。
禁读清单：派发消息未给，未读任何被禁文件。
草稿目录：派发消息未给；本轮没有用到草稿（提示与核对表直接现写进交付路径，没有中间产物需要暂存），因此空着。

## 逐次调用

| 序号 | 退出码 | 词数（`wc -w`） | corruption-check.py 自报词数 | oov-check.py 生词清单 | 判定 |
|---|---|---|---|---|---|
| s1 | 0 | 1035 | 1167 | 无（生词=0，拼接=0） | 干净 |
| s2 | 0 | 803 | 914 | judgment's、vocabulary、checker's（生词=4，拼接=0；均为正常英文词/所有格，逐一现查原文上下文，不是缺头粘连词，不记带损坏） | 干净 |

两次调用都在第一次尝试就退出码 0、且两道检测器（`corruption-check.py`、`oov-check.py`）都判绿；没有出现退出码 5（字词损坏作废）的情形，因此没有 `-output-void*.md` 副本。样本自带的任何行号（本轮样本里模型没有引用任何行号，提示本身也不含文件路径与行号）：无需标注，因为两份样本都没有写出行号。

## 干净样本数

两份（s1、s2），达到「至少两份干净样本」的要求，未续跑第三次。

## 没做什么

- 没有解读、总结或采纳两份样本的答复内容，也没有比较两份样本之间方向是否一致——这些判定留给主 agent。
- 没有使用草稿目录（派发消息未给，且本轮不需要中间产物）。
- 没有读任何禁读清单文件（派发消息未给禁读清单）。
