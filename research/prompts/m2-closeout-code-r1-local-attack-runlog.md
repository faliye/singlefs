# m2-closeout-code-r1 本地攻方腿 运行记录

提示：`research/prompts/m2-closeout-code-r1-local-attack.md`
核对表：`research/prompts/m2-closeout-code-r1-local-attack-translation-audit.md`
快照取法：`git archive refs/sop/m2-closeout-code-r1-snapshot crates | tar -x -C /tmp/claude-1000/m2-closeout-code-r1-local/tree`（草稿目录 `/tmp/claude-1000/m2-closeout-code-r1-local/`）。

## 调用序列（同一份提示，连续 4 次调用；未到 5 次上限）

1. 第一次调用 → 判红，退出码 5。作废副本：`research/prompts/m2-closeout-code-r1-local-attack-output-void1.md`（`corruption-check.py` 报「星号落单=1 粘连=1」）。这一次不算样本，`s1` 号沿用重跑。
2. 第二次调用（沿用 `s1` 号）→ 退出码 0，406 词判红那次之外全新一份，字数 1895。`oov-check.py` 判绿：`生词=1 拼接=0`，生词 `plausibly`（合法英文词，误报，非损坏）。通读一遍未见断句、缺词、成对标记落单一类损坏。记为干净样本 `s1`。
3. 第三次调用（新 `s2` 号）→ 判红，退出码 5。作废副本：`research/prompts/m2-closeout-code-r1-local-attack-output-void2.md`（`corruption-check.py` 报「星号落单=1」）。这一次不算样本，`s2` 号沿用重跑。
4. 第四次调用（沿用 `s2` 号）→ 退出码 0，字数 890。`oov-check.py` 判绿：`生词=2 拼接=0`，生词都是 `segments'`（英文所有格，合法词，误报，非损坏）。通读一遍未见断句、缺词、成对标记落单一类损坏。记为干净样本 `s2`。

干净样本数达到 2（`s1`、`s2`），停止抽样。

## 样本一览

| 样本 | 退出码 | 词数 | oov-check | 判定 |
|---|---|---|---|---|
| `m2-closeout-code-r1-local-attack-output-void1.md` | 5 | — | 判红（作废，脚本自动写，不计入样本） | 作废 |
| `m2-closeout-code-r1-local-attack-output-s1.md` | 0 | 1895 | 绿，生词=1（`plausibly`，合法词） | 干净 |
| `m2-closeout-code-r1-local-attack-output-void2.md` | 5 | — | 判红（作废，脚本自动写，不计入样本） | 作废 |
| `m2-closeout-code-r1-local-attack-output-s2.md` | 0 | 890 | 绿，生词=2（均为 `segments'`，合法所有格，非损坏） | 干净 |

sha256（干净样本）：
```
ec33326d8f908569b91dd18f8e676dd8a92fa299f82b18ec8b6e4f8dbf14c91c  m2-closeout-code-r1-local-attack-output-s1.md
d4922f6ffb78626611cdd1c30d64bcd08d45ec764285ecd018b69eb956631b00  m2-closeout-code-r1-local-attack-output-s2.md
```

## 5b. 答案表

主 agent 派发提示里没有给算好的答案表路径。**没有答案表，算术没比。**

## 没做什么

- 不解读、不总结、不采纳两份样本里的具体答复（表格填的数、算术过程、匹配与否的判断），判它打没打中是主 agent 的事。
- 不判两份样本方向是否一致。
- 没有主 agent 给的答案表，第 5b 步的逐格比数没有做。
