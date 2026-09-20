# 运行记录：d13-item4-fua-r1-local-defense

提示 `research/prompts/d13-item4-fua-r1-local-defense.md`（265 行，英文，无 markdown 强调，不含 Rust `::`）；
核对表 `research/prompts/d13-item4-fua-r1-local-defense-translation-audit.md`。
按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，本轮共调用 4 次，
第 3、4 次拿到干净样本，达到两份的要求，未跑满五次上限。

## 逐次调用

| 次序 | 文件 | 退出码 | 词数 | `oov-check.py` 生词/拼接 | 人工通读结论 | 判定 |
|---|---|---|---|---|---|---|
| 1 | `-output-s1.md` | 0 | 1063 | 生词=2（strongest / refuterefute），拼接=0 | 第 15 行「would refuterefute row 2's reading」：两个词粘死成一个复读词，与「creates creates」同一类损坏 | 带损坏 |
| 2 | `-output-s2.md` | 0 | 946 | 生词=3（strongest / observesing / wording's），拼接=0 | 第 9 行「the measured test observesing behavior」：`observesing` 是粘连词（非英语单词）；第 12 行「Row 2 and: the segment boundary interpretation」读起来像中间掉了一个词 | 带损坏 |
| 3 | `-output-s3.md` | 0 | 839 | 生词=5（strongest / misaligns / misstatement / contradicting，脚本只列出 4 个，未细查计数差异），拼接=0 | 通读全文（OBJECTION / 表格四行 / VERDICT / REFUTING OBSERVATION）未见黏连词、掉词或断句；`oov-check.py` 报的生词均为合法英语词，非拼接损坏 | 干净（第 1 份） |
| 4 | `-output-s4.md` | 0 | 1033 | 生词=7（strongest / misstatement / enumeratable / misphrasing / contradiction / wording's / imprecision），拼接=0 | 通读全文未见黏连词或掉词；生词均为合法或合理派生的英语词 | 干净（第 2 份） |

本轮四次调用全部退出码 0，`ask-local.sh` 自身的损坏闸（`corruption-check.py` + `oov-check.py`）均未拦下，
没有产生 `-output-void*.md`；第 1、2 份的损坏（粘连词、疑似掉词）都是人工通读另外核出来的，不是脚本判红。

## 结论：两份干净样本（`-output-s3.md`、`-output-s4.md`）

不解读、不总结、不采纳这两份样本的辩护内容，也不比较四份样本方向是否一致，是不是打得中由主 agent 判。
