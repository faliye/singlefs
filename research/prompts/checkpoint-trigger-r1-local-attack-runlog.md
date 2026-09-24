# 运行记录：checkpoint-trigger-r1-local-attack（2026-09-22 UTC）

提示文件：`research/prompts/checkpoint-trigger-r1-local-attack.md`（英文，覆盖 K3 三支打不打架
的三问，加 K1 里「journal 环已占用的字节这个量今天在 crates/ 里存不存在」那一格；4 道 judgment，
各 3 项子答复，共 12 格，在「一份不超过 16 格」之内，未拆分、未撞输出预算）。不用任何 markdown
强调、不用 pipe 表格，答案按 judgment 编号，指路只用 fact 编号（1 至 10），明令不写文件行号或代码
行号（需要指代码时按函数名）。转述核对表：
`research/prompts/checkpoint-trigger-r1-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
前台跑，全程未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,etimes,pcpu,args -u "$(id -u)"` 查过
`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`：零命中，不是性能测量在跑。

开工前用 `checkpoint-trigger-r1-start-snapshot.sha256` 核对过背景材料与相关 kb 文件的 sha256，
与快照逐个一致，工作区无漂移。

## 逐次调用

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 通读复核 | 结论 |
|---|---|---|---|---|---|---|---|
| 1 | `checkpoint-trigger-r1-local-attack-output-s1.md` | 0 | 325 | 绿，生词=1 拼接=0（plausible，规则表外正常英文词） | 绿，全部计数为 0 | 无缺词、无断句、无孤立标点、无粘连词 | 干净 |
| 2 | `checkpoint-trigger-r1-local-attack-output-s2.md` | 0 | 319 | 绿，生词=2 拼接=0（altersals、plausible） | 绿，全部计数为 0 | 通读发现 judgment 2 的 refutation 句里 "the third branch altersals the durability timing" 一词是缺头/粘连词（疑似 alters 与 als 粘连），不成词；闸未拦住，人工判定为损坏 | 带损坏（不算干净样本） |
| 3 | `checkpoint-trigger-r1-local-attack-output-void1.md` | 5 | （作废副本，模型侧原样输出，`wc -w` 未统计） | 脚本自报：生词=1 拼接=1（unaffectedocused=unaffected+(f)ocused） | 脚本自报判红（字词损坏），`ask-local.sh` 判定退出码 5 | 未通读（脚本判红即作废，不进入人工复核） | 判红作废；同一次重定向建出的 `-output-s3.md` 是空文件（0 行），沿用 s3 号重跑 |
| 3（重跑） | `checkpoint-trigger-r1-local-attack-output-s3.md` | 0 | 415 | 绿，生词=2 拼接=0（uniformlyactly、plausible） | 绿，全部计数为 0 | 通读发现 judgment 1 的 justification 句里 "the action is uniformlyactly checkpointing" 一词是缺头/粘连词（疑似 uniformly 与 exactly 一类词粘连），不成词；闸未拦住，人工判定为损坏 | 带损坏（不算干净样本） |
| 4 | `checkpoint-trigger-r1-local-attack-output-s4.md` | 0 | 359 | 绿，生词=0 拼接=0 | 绿，全部计数为 0 | 无缺词、无断句、无孤立标点、无粘连词 | 干净 |

## 干净样本清点

- 干净：s1、s4，共 2 份，达到「至少两份干净样本」的门槛，到此停止抽样。
- 带损坏：s2（"altersals"）、s3 第一次判红作废（"unaffectedocused"）、s3 重跑后仍带损坏
  （"uniformlyactly"）——三次均为 oov-check.py 列出的生词经通读判定为缺头或粘连的损坏词，
  不当干净样本；其中只有 void1 那一次是脚本自身判红（退出码 5），另两次（s2、s3 重跑）脚本判绿
  （退出码 0），损坏是人工通读加 oov-check.py 生词表复核出来的，不是脚本红判。
- 全部调用累计 5 次（含 1 次判红作废），未超过「连续五次调用」的上限；退出码分布：0 出现 4 次
  （s1、s2、s3 重跑、s4），5 出现 1 次（void1）；没有非 0 非 5 的退出码，没有网关不通，本地腿全程
  未缺席。
- 目录核对：`ls research/prompts/ | grep checkpoint-trigger-r1-local-attack` 显示提示文件 1 份、
  核对表 1 份、运行记录 1 份（本文件）、`-output-s1.md` 至 `-output-s4.md` 共 4 份、
  `-output-void1.md` 1 份，与本记录逐行一致。

## 没做什么

- 不解读、不总结、不采纳四份样本（含带损坏的两份）里对 judgment 1 至 4 的具体判词，不判多次
  抽样之间方向是否一致——那是主 agent 的事。
- 未跑云端攻方腿（Opus，K2/K4）、云端正推腿（Sonnet，K1 落在 fs-design 三格哪一格）或核实员，
  不在这条腿的任务范围内。
- 未碰 K2、K4，未碰 K1 里「落在 fs-design 三格哪一格」与「是不是随时读得出」两问，按分工表
  不许碰；judgment 4 已在提示里明令模型不评价这两问，只答存在性。
- 未读这一轮别的腿的提示与产出、主 agent 这一轮的核实、同时在飞的别的轮次。
