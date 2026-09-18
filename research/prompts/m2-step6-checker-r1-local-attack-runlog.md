# 运行记录：m2-step6-checker-r1-local-attack（2026-09-17）

提示文件：`research/prompts/m2-step6-checker-r1-local-attack.md`（英文，问题按 1–6 编号，覆盖正文第三节 Y3——坏镜像与不适用，不用任何 markdown 强调，不写 Rust 路径分隔符两个冒号，不写汉字变量名）。
转述核对表：`research/prompts/m2-step6-checker-r1-local-attack-translation-audit.md`（21 处转述逐句核对，均已按原文补全限定词或否定性事实）。
调用方式：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-step6-checker-r1-local-attack.md`，全部前台跑，未用 `setsid` / `&` / `disown`。

## 调用记录

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py 判定 | corruption-check.py（经 ask-local.sh 内置） 判定 | 结论 |
|---|---|---|---|---|---|---|
| 1 | `m2-step6-checker-r1-local-attack-output-s1.md`（第一次调用） | 5 | 0（空文件，判红时重定向建出的占位） | 未跑（判红即拒绝，未到输出阶段） | 红：生词=5 拼接=2（`coincidental` 被误拆、`qualifiesqualifies` 实词自复读） | 作废，原样输出留存 `m2-step6-checker-r1-local-attack-output-void1.md` |
| 1（沿用同一个号重跑） | `m2-step6-checker-r1-local-attack-output-s1.md` | 0 | 667 | 绿，生词=5（`mutations checker's invariant's`，均为合法词/所有格，非拼接） 拼接=0 | 绿 | 干净样本 s1 |
| 2 | `m2-step6-checker-r1-local-attack-output-s2.md` | 0 | 725 | 绿，生词=11（`comment's mutations checker's unjustified`，均为合法词/所有格，非拼接） 拼接=0 | 绿 | 干净样本 s2 |

样本号只数干净样本：第一次判红的那次重定向建出的 `s1` 是空文件，沿用同一个号重跑成功，因此 s1、s2 两个号都对应干净样本，未触发「连续五次拿不到两份就停」那条。

## 通读复查（闸抓不到的那一类）

`oov-check.py` 判绿只说明没有可切分的拼接生词；按流程另外通读两份样本，找缺词、断句、孤立标点这类闸抓不到的损坏，以及缺头的粘连词（例如计划第十一节点名的 `achievesceives` 这种形态）：

- s1（6 问全部按编号作答，1–6 无缺号；每问的答案段落里包含判断与部分理由，但没有像 s2 那样把「什么现象会推翻它」单独成段）：逐句读过，句子完整，没有孤立标点或半截句子，没有发现看似生造但因缺头而拼出「合法词」从而被生词表放过的词（如 `warm-up`、`self-consistency` 均为正常连字符复合词，不是缺头粘连）。判定：干净，不带损坏。
- s2（6 问全部作答，每问答案之后单独一段以 "If..." 开头，显式给出「什么观测会推翻这条答案」）：逐句读过，句子完整，编号连续，无孤立标点，无可疑缺头粘连词。判定：干净，不带损坏。

两份都不需要 `ASK_LOCAL_ALLOW_CORRUPT=1`。

## 没做什么

- 不解读、不总结样本里的论点，不判它是否打中 Y3（回退候选集、I-7.4/I-4.8 是否被两份坏镜像分开测出、抬 F 后「不适用」对不对）——那是主 agent 的事。
- 没有核对本地攻方在 s1 问题 1 里给出的具体推理（例如它认为「第一个文件发布必然更新树表」这一步本身是否站得住）——那属于判打没打中的范畴，留给主 agent。
- 没有跑到第 3 次干净样本调用：s1（沿用号重跑后）与 s2 都是干净样本，按「攒到至少两份干净样本为止」已达标，未继续抽样。
- 没有跑本地辩方腿、Opus 攻方腿或 Sonnet 正推腿，不在这条腿的任务范围内；未读 `research/prompts/m2-step6-checker-r1-*-output*.md` 里别的腿的输出（这一轮的禁读清单）。
- 未核实提示里「Fact about how a referenced unit's checksum is actually checked」等段落转述的代码事实在 s1/s2 答复里被引用之后是否被正确使用——那也是判打没打中的一部分，不是这条腿的职责。

## 历史版本

（暂无历史）
