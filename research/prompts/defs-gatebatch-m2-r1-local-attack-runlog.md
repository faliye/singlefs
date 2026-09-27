# defs-gatebatch-m2-r1 本地攻方：运行记录

提示文件：`research/prompts/defs-gatebatch-m2-r1-local-attack.md`
核对表：`research/prompts/defs-gatebatch-m2-r1-local-attack-translation-audit.md`
命令：`bash research/scripts/ask-local.sh research/prompts/defs-gatebatch-m2-r1-local-attack.md`（前台跑，未用 setsid / & / disown）

| 号 | 退出码 | 词数（wc -w） | oov-check 生词 | 判定 |
|---|---|---|---|---|
| s1 | 0 | 302 | falsified（1 个，真词，非损坏） | 干净 |
| s2 | 0 | 184 | falsified（1 个，真词，非损坏） | 干净 |

两次调用都判绿；另通读一遍两份样本：都是 12 行编号答复（G1.1–G3.4），没有断句、没有粘连词、没有整词缺失、没有实词自复读、没有成对标记落单。攒到两份干净样本，按定义第 6 条止步，不再抽样。
