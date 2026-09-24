# 本地攻方运行记录：c510-date-gate-r1

只攻 G1（射程：文件名全仓，正文只罩 fixtures）的「误判」那一半——哪些合法的对象会被文件名维度判红；不谈 G3（判别力）、不谈 G4（落点），也不谈正文维度。

按主 agent 派发提示的显式指示，这一轮的提示文件、逐句核转述表、样本原始输出**不写进仓里**，只落草稿目录：`/tmp/claude-1000/c510-local-attack/`。仓里只写这一份报告，路径：`research/prompts/_c510-date-gate-r1-local-attack-output.md`。这是对本地攻方定义默认写范围（提示/核对表/样本三者本应写进 `research/prompts/`）的显式偏离，依据是派发提示原文最后一句「草稿放 `/tmp/claude-1000/c510-local-attack/`，不许往仓里写别的文件」。

草稿目录下的文件与 sha256sum：

```
619cb665aa1317f57f04efe29af6355ccc3e3ebce9399110f5edb895e1833a9a  c510-date-gate-r1-local-attack.md
b682478b01099f855b37cf7571981ca5af1d2445b03744e53f0d36777575ded1  c510-date-gate-r1-local-attack-translation-audit.md
306ee5ae83816868553ca0380950fd689cd45be49e2c5d7d9bc6d09bf8bb4d0c  c510-date-gate-r1-local-attack-output-s1.md
611c070ae240eab6625e449512cc5bd92e4883c980ac8a41e1ed654c87f90b0f  c510-date-gate-r1-local-attack-output-s2.md
24093c45f68d7d3c4a0a347275adc644fedb972e41a2cf6a3f9463e7c317f9c1  c510-date-gate-r1-local-attack-output-void1.md
57dcd9296fd9d107fc46a45f3dad5de7f6fd7425447b7810afb8e9192ad14cfc  c510-date-gate-r1-local-attack-output-void2.md
```

提示：英文，自足，无 markdown 强调；18 格判据，覆盖六类（CATEGORY_A 外部文献日期入文件名、CATEGORY_B 归档迁入的历史文件带旧日期、CATEGORY_C 生成物带上游日期、CATEGORY_D 浅克隆或非顶层调用、CATEGORY_E 跨时区书写、CATEGORY_F fixtures 故意造的越界日期样本），每格要求「具体形态 / 为什么合法 / 今天在不在仓里」三项，逐项都要一句「This would be refuted by:」。写死的事实 8 条（FACT 1–8）全部由本 agent 现查得出（读 `.claude/gate.d/95-fixture-claims.sh`、`.claude/singlefs-ai-sop/scripts/lib.sh`、`.claude/kb/prior-art.md`、`.claude/kb/decisions-history/2026-09.md`、`.gitignore`，以及 `git ls-files` / `git log` 系列命令），不是转述派发提示的转述——逐句核对表见草稿目录 `c510-date-gate-r1-local-attack-translation-audit.md`（17 行核对，含本 agent 自己现查补出的限定词，例如 FACT 4 里 95-fixture-claims.sh 对 `project_start_date` 返空时的回退替换机制，以及 FACT 5 里 decisions-history 文件名只有年-月两段、正则命不中）。

四次前台调用（未用 setsid / 后台 / disown），命令一致（除路径外）：
`nice -n 19 bash research/scripts/ask-local.sh /tmp/claude-1000/c510-local-attack/c510-date-gate-r1-local-attack.md > /tmp/claude-1000/c510-local-attack/c510-date-gate-r1-local-attack-output-s<n>.md`

## 第 1 次调用（判红作废，留存为 output-void1.md）

- 提示当时的版本：日期占位写法用大写 `YYYY-MM-DD`
- 退出码：5
- 判红的不是模型答复，是提示文件本身：`corruption-check.py` 把提示文件（`$1` 参数）判红，`缩写自粘: YYYY | YYYY | YYYY`——`ACRO_DUP` 正则 `\b([A-Z]{2,})\1\b` 把 `YYYY` 拆成 `YY`+`YY` 命中；模型自己的答复（临时文件）当时判绿（cjk=0 words=486，各项损坏计数均 0）
- 处置：把提示里 3 处 `YYYY-MM-DD` 改成小写 `yyyy-mm-dd`（正则只认大写，改完复核 0 命中），重跑，仍沿用 output-s1 这个号
- 分类：作废（提示文本本身的检测器假阳性，不算三方不一致，也不算模型答复的字词损坏）

## 第 2 次调用（判红作废，留存为 output-void2.md）

- 提示版本：已改小写 `yyyy-mm-dd`
- 退出码：5
- `oov-check.py` 判模型答复本身为红：`拼接=4`，`traceability(=trace+ability)`、`substrings(=subst+rings)`×3；生词表来自 Linux 内核文档、非完整英文词典，`traceability`（原句「for clarity and traceability」）与 `substrings`（复述提示里 FACT 1 的「yyyy-mm-dd substring」）均为正常英文词，被规则拆成两个表内词
- 处置：把提示里 2 处 `substring` 改成 `occurrence`（降低模型复述提示原词触发同一假阳性的概率；这是本 agent 自主的编辑选择，不改变任何事实内容），重跑，仍沿用 output-s1 这个号
- 分类：作废（模型答复本身触发检测器假阳性，按规则「闸判红之后：那一轮作废重跑，不许记成三方不一致」处置，未设 `ASK_LOCAL_ALLOW_CORRUPT=1`）

## 第 3 次调用：`c510-date-gate-r1-local-attack-output-s1.md`

- 退出码：0
- 词数：`wc -w` = 308；`corruption-check.py` 自报 words=359
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=11（去重后 3 个：legitimacy、foreseeably、preproject）拼接=0，退出码 0——三个均为正常英文词或合理复合词（pre+project 作为示例文件名的一部分），非损坏形态
- 通读结果：全文 24 行逐行读过，句子完整、无缺词、无断句、无孤零标点
- 判定：干净

## 第 4 次调用：`c510-date-gate-r1-local-attack-output-s2.md`

- 退出码：0
- 词数：`wc -w` = 790；`corruption-check.py` 自报 words=834
- `corruption-check.py`：绿，cjk=0 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0，退出码 0
- `oov-check.py`：绿，生词=11（去重后 4 个：legitimacy、foreseeably、archiving、toolkit's）拼接=0，退出码 0——均为正常英文词或所有格形式，非损坏形态
- 通读结果：全文 18 组答案逐行读过，每格三项（shape / legitimacy / today）各带一句「This would be refuted by:」，句子完整、无缺词、无断句、无孤零标点
- 判定：干净

## 样本数与停止条件

四次前台调用（两次判红作废、两次判绿）即拿到两份干净样本，达到「至少两份干净样本」的下限，停在这里，未继续消耗调用额度，未触及「连续五次拿不到两份干净的」这条停下条件（本轮共用去 4 次，含两次作废，均在同一个 output-s1 号位上被判红作废并沿用同一个号重跑，符合脚本自己把作废副本另存为 `-output-void<n>.md`、s 号位留给下一次判绿或空文件的约定）。

## 没做什么

- 不解读、不总结、不采纳模型的答复内容；本报告不写六个 CATEGORY 各自答了什么，也不写两份样本之间方向是否一致——判它打没打中是主 agent 的事。
- 不碰 G3（判别力）、G4（落点）、正文维度，也不碰云端攻方与云端正推两条腿的攻击面。
- 未按定义默认写法把提示文件、核对表、样本原始输出写进 `research/prompts/`——按派发提示的显式指示，这些改放草稿目录 `/tmp/claude-1000/c510-local-attack/`，本报告已列出对应 sha256sum；如需要把它们正式入库，需要另一条指示。
