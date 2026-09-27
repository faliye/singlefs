# m2-layer0-scale-r3 本地攻方腿：运行记录

提示文件：`research/prompts/m2-layer0-scale-r3-local-attack.md`。
逐句核转述核对表：`research/prompts/m2-layer0-scale-r3-local-attack-translation-audit.md`。

## 调用记录（按发生顺序）

1. 第 1 次调用（占号 s1）：`bash research/scripts/ask-local.sh research/prompts/m2-layer0-scale-r3-local-attack.md > research/prompts/m2-layer0-scale-r3-local-attack-output-s1.md`，退出码 0。词数 513（`wc -w`）。`oov-check.py` 判绿，生词=13 拼接=0，生词清单：Falsified、reconstruction、extrapolation（其余 10 个为常见派生形态，脚本原样列出前几个）。通读一遍未见缺词、断句或孤立标点。样本自身没有出现任何文件名/行号（`grep -ni "line \|\.rs:\|\.md:"` 命中 0 次），无需标「模型自给、未核」。判定：**干净**。

2. 第 2 次调用（占号 s2，第一次尝试）：同一提示，重定向到 `-output-s2.md`。退出码 5——`corruption-check.py` 判红：拼接 `statesstates(=states+states)`，生词 `statesstates` 等。按脚本设计，`s2` 落盘为空文件，作废副本留存为 `research/prompts/m2-layer0-scale-r3-local-attack-output-void1.md`（词数 517，模式 0600，不算样本）。按定义第 4 条，`s2` 这个号沿用重跑。

3. 第 3 次调用（占号 s2，第二次尝试）：同一提示，重定向到 `-output-s2.md`。退出码 5——`corruption-check.py` 判红：实词自复读 `higher`（同一个 ≥5 字母词连着出现两次）。`s2` 再次落空，作废副本留存为 `research/prompts/m2-layer0-scale-r3-local-attack-output-void2.md`（词数 422，模式 0600，不算样本）。`s2` 号再次沿用重跑。

4. 第 4 次调用（占号 s2，第三次尝试）：同一提示，重定向到 `-output-s2.md`。退出码 0。词数 652（`wc -w`）。`oov-check.py` 判绿，生词=18 拼接=0，生词清单：Falsified、reconstruction、measurement's、reconstruction's、extrapolation（其余为常见派生形态）。通读一遍未见缺词、断句或孤立标点。样本自身没有出现任何文件名/行号（同样 `grep` 命中 0 次）。判定：**干净**。

## 汇总

| 号 | 调用次序 | 退出码 | 词数 | 生词清单（`oov-check.py` 原样） | 判定 |
|---|---|---|---|---|---|
| s1 | 第 1 次 | 0 | 513 | Falsified reconstruction extrapolation（等 13 个） | 干净 |
| void1 | 第 2 次（占 s2 号，作废） | 5 | 517 | 拼接=1：statesstates(=states+states)；生词：conditions' statesstates extrapolations | 作废 |
| void2 | 第 3 次（占 s2 号，作废） | 5 | 422 | 拼接=0；实词自复读=1：higher | 作废 |
| s2 | 第 4 次 | 0 | 652 | Falsified reconstruction measurement's reconstruction's extrapolation（等 18 个） | 干净 |

干净样本已达到 2 份（s1、s2），按定义第 6 条停止，不再继续调用。总调用 4 次（含 2 次判红作废），未触及「连续五次调用拿不到两份干净就停」的停机线。

## 没做什么

- 不解读、不总结、不采纳样本 s1、s2 里对代价表五个数的具体判定，也不比较两份样本方向是否一致——判它们打没打中、哪一步外推的判断对不对，按定义归主 agent。
- 没有跑云端攻方（Opus）与云端辩方（Sonnet）的提示与产出，那两条腿的材料本轮不读、也不知道其结论。
