# 运行记录：m2-final-code-r2-local-attack（2026-09-25 UTC）

提示文件：`research/prompts/m2-final-code-r2-local-attack.md`（10 格：W.max_s、W.table_bytes、
W.fits_in_slot、AC.leaf_cap、AC.internal_cap、MP.leaf_cap、MP.internal_cap、ROWS.height_at_15、
ROWS.min_devices_to_split、REC.named_cap，对应正文第四节分工表本地攻方那一行的①-⑤五个算术格）。
不用任何 markdown 强调、不用 pipe 表格，答案按 10 个标签编号，指路只用 FACT 编号（FACT 1 至 FACT 9），
明令不写文件行号或代码行号。转述核对表：`research/prompts/m2-final-code-r2-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，前台跑，
全程未用 `setsid` / `&` / `disown`。调用前 `ps -o pid,args -u "$(id -u)"` 查过
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：零命中，不是性能测量在跑；
`curl -m 5 http://127.0.0.1:8200/v1/models` 确认网关在监听（返回 401，非连接失败）。

## 样本表

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `m2-final-code-r2-local-attack-output-s1.md` | 0 | 350 | 绿，生词=0 拼接=0 | 绿，cjk=0 words=304 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 | 10/10 格齐全、按序、各带公式、代入、中间步骤与「This would be refuted by」；ROWS.min_devices_to_split 逐个 D 列到 80；无缺词、无断句、无孤立标点、无缺头粘连词 | 干净 |
| 2 | `m2-final-code-r2-local-attack-output-s2.md` | 0 | 680 | 绿，生词=0 拼接=0 | 绿，cjk=0 words=559 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 | 10/10 格齐全、按序、各带公式、代入、中间步骤与「This would be refuted by」；ROWS.min_devices_to_split 逐个 D 列到 80；无缺词、无断句、无孤立标点、无缺头粘连词 | 干净 |

两份干净样本达标，到此为止，不再抽样。全程未出现退出码 5（字词损坏闸判红）、无作废副本
（`-output-void*.md`）、无本地腿缺席（两次调用退出码均为 0，网关全程应答成功）。

## 目录核对

`ls research/prompts/ | grep "m2-final-code-r2-local-attack"` 列出：1 份提示、2 份 `-output-sN.md`、
1 份核对表、本文件，共 5 份，与本记录逐项一致，无遗漏、无多余、无 void 副本。

## 没做什么

- 不解读、不总结、不采纳两份样本里 10 格算术的具体数值方向是否一致，不判两次抽样之间是否一致——
  那是主 agent 的事；这份记录只报退出码、词数、生词清单、干净 / 带损坏 / 作废。
- 未跑云端攻方腿（Opus，Z8、Z9、Z12）或云端正推腿（Sonnet，Z7、Z10、Z11），不在这条腿的任务范围内。
- 主 agent 这一轮派发提示未给出明确的「草稿目录」与「禁读清单」；本任务未用到草稿空间（全部产物
  直接落在写范围内的 `research/prompts/`），按共用约束「用不上可以空着」处理；禁读清单按既有先例
  （`m2-treesplit-r1-local-attack-runlog.md` 的同类记法）视为「这一轮未特别禁读」，仅读了完成本任务
  必需的文件（正文、冻结代码 `crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/unit.rs`、
  `code_two_tree.rs`、`recovery.rs`、`transaction.rs`，kb 快照 `decisions/08-核心索引结构.md`、
  `checks-owed.md`、既有本地攻方提示先例）。
- 未读这一轮的 `_m2-final-code-r2-appendix.md`、`_m2-final-code-r2-checklist.md`、`_m2-final-code-r2-diff.md`
  之外用不到的部分；未读主 agent 这一轮的核实、同时在飞的别的轮次。
