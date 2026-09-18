# m2-agentdef-r1 本地攻方运行记录

提示文件：`research/prompts/m2-agentdef-r1-local-attack.md`
核对表：`research/prompts/m2-agentdef-r1-local-attack-translation-audit.md`
样本前缀：`research/prompts/m2-agentdef-r1-local-attack-output`
全部前台跑 `bash research/scripts/ask-local.sh`，未用 setsid / & / disown。

## 调用记录

| 调用序 | 命令重定向到 | 退出码 | 处置 |
|---|---|---|---|
| 第 1 次 | `-output-s1.md` | 5（判红作废） | 脚本自留 `-output-void1.md`；`corruption-check` 报「实词自复读：shice \| shice」；这次重定向建出的 `s1` 是空文件（0 字节），沿用同一个号重跑 |
| 第 2 次（沿用 s1） | `-output-s1.md` | 0 | 占用 s1 号 |
| 第 3 次 | `-output-s2.md` | 0 | 占用 s2 号 |

一共 3 次调用（1 次判红作废 + 2 次退出码 0），未达到「连续五次调用拿不到两份干净」的停止线；两份退出码 0 的样本都判「干净」（见下），已达到「至少两份干净样本」，停止调用。

## 样本判定

| 样本 | 退出码 | 词数（`wc -w`） | `oov-check.py` 结果 | 生词清单（去重） | 判定 |
|---|---|---|---|---|---|
| `-output-void1.md` | 5 | 544 | 未跑（作废副本不计入判定，脚本本身已判红：`corruption-check` 报实词自复读 `shice \| shice`） | 不适用 | 作废 |
| `-output-s1.md`（第 2 次调用） | 0 | 1022 | 绿：生词=15，拼接=0 | BIANGENGSHI、WEISHENME、narrative（后两者去重后各出现多次，均为提示自带标签或普通英文词，非缺头粘连词） | 干净 |
| `-output-s2.md`（第 3 次调用） | 0 | 922 | 绿：生词=19，拼接=0 | BIANGENGSHI、WEISHENME、exemption（同上，均非缺头粘连词） | 干净 |

`oov-check.py` 两份样本 `拼接=0`，即 `splice_of` 对全部生词都返回空——没有一个能切成「词表词+词表词」的缺头粘连形态；`生词` 列表里出现的都是提示自己引入的全大写标签（BIANGENGSHI、WEISHENME 等）或本身就是词表外的正常英文词（narrative、exemption），逐一核对后没有一个是缺头粘连词，不改判「带损坏」。另通读两份样本全文（各 44 条编号答复），没有断句、缺词、孤立标点这类损坏形态。

## 没做什么

- 未解读、未采信两份样本的具体答复内容，也未比较两份样本方向是否一致；判它们打没打中判据是主 agent 的事。
- 未对判红作废的那次调用（`-output-void1.md`）做任何内容分析，只记了脚本自己报的判红理由与词数，其余不看。
