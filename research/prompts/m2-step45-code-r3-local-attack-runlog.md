# 运行记录：m2-step45-code-r3-local-attack（2026-09-17）

提示文件：`research/prompts/m2-step45-code-r3-local-attack.md`（英文，问题按 1–9 编号，覆盖正文第三节 X4 与 X5，不用任何 markdown 强调）。
转述核对表：`research/prompts/m2-step45-code-r3-local-attack-translation-audit.md`（10 处转述逐句核对，均已按原文补全限定词）。
调用方式：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-step45-code-r3-local-attack.md`，全部前台跑，未用 `setsid` / `&` / `disown`。

## 调用记录

| 次序 | 目标文件 | 退出码 | 词数（`wc -w`） | oov-check.py 判定 | corruption-check.py 判定 | 结论 |
|---|---|---|---|---|---|---|
| 1 | `m2-step45-code-r3-local-attack-output-s1.md` | 0 | 1310 | 绿，生词=0 拼接=0 | 绿，汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 | 干净样本 s1 |
| 2 | `m2-step45-code-r3-local-attack-output-s2.md` | 0 | 1022 | 绿，生词=0 拼接=0 | 绿，同上全 0 | 干净样本 s2 |

两次调用都在第一次就拿到干净样本，没有产生 `-output-void*.md`（`ls research/prompts/m2-step45-code-r3-local-attack*` 确认过，目录下没有 void 文件）。样本号连续，两份都算干净样本，未触发「连续五次拿不到两份就停」那条。

## 通读复查（闸抓不到的那一类）

`oov-check.py` 判绿只说明没有可切分的拼接生词；按流程另外通读两份样本，找缺词、断句、孤立标点这类闸抓不到的损坏：

- s1（9 问全部按编号作答，每问都带一段「Observation that would refute this」）：逐句读过，句子完整、列表编号连续（1–9 无缺号）、没有发现孤立标点或半截句子、没有发现看似生造但因缺头而拼出「合法词」从而被生词表放过的词。判定：干净，不带损坏。
- s2（同样 9 问全部作答，格式与 s1 一致）：同样通读，句子完整、编号连续、无孤立标点、无可疑缺头粘连词。判定：干净，不带损坏。

两份都不需要 `ASK_LOCAL_ALLOW_CORRUPT=1`。

## 没做什么

- 不解读、不总结样本里的论点，不判它是否打中 X4 / X5——那是主 agent 的事。
- 没有跑到第 3 次调用：前两次都是干净样本，按「攒到至少两份干净样本为止」已达标，未继续抽样。
- 没有核对本地攻方给出的具体数字（比如它自己举的「50 − 20 = 30」「34 − 10 = 24」这类例子）是不是与固定脚本的真实账目吻合——那些数是本地模型自己造的例子用来说明机制，不是它对 X4 追问的「34、36 那两步」给出的确定性计算结果（它在两份样本里都明说了「the facts do not provide the exact totals」），核实这一步同样留给主 agent。
- 没有跑第三、四条云端腿或本地辩方腿，不在这条腿的任务范围内。
