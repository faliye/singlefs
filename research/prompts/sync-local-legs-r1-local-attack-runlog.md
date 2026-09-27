# sync-local-legs-r1 本地攻方运行记录

提示文件：`research/prompts/sync-local-legs-r1-local-attack.md`
译文核对表：`research/prompts/sync-local-legs-r1-local-attack-translation-audit.md`
答案表：主 agent 没给算好的答案表，算术没比。

## 逐份样本

| 号 | 命令退出码 | 词数 | oov-check 生词（前300字符内） | 判定 |
|---|---|---|---|---|
| s1 | 0 | 583 | `TIMEOUT's uncreated` | 带损坏——闸判绿，但逐段通读时第 4 题子情形 (i) 结尾处「Exit code: Non-zero (e 124 for timeout).」里「e」是「e.g.」一类缩写掉了后半，剩一个孤零零的字母，判为缺字类损坏，不当干净样本 |
| s2 | 0 | 518 | `contradicting` | 带损坏——闸判绿；逐段通读时第 4 题子情形 (i) 「Exit code: Non-zero (eating timeout termination).」里「eating」与同一份样本第 38 行同一句式「indicating timeout termination」对不上，判为「indicating」被吞掉一段字母后留下的变形词，与 s1、s3 在同一位置出现的同类问题（见下）合起来看，拿不准是不是新造词还是掉字，一律记带损坏 |
| s3 | 0 | 410 | 无 | 带损坏——闸判绿（生词=0）；逐段通读时第 4 题子情形 (i) 「the exit code is non-zero (indic 124 for timeout termination)」里「indic」是「indicating」掉了后半个词，剩下的片段不成词，判缺字类损坏 |
| s4（第一次） | 5 | — | — | 判红作废，脚本已留成 `research/prompts/sync-local-legs-r1-local-attack-output-void1.md`；判红那次的 `s4` 是空文件（`ls -la` 核过，0 字节），已用 `>|` 同号重跑 |
| s4（重跑） | 0 | 558 | 无 | 干净——闸判绿（生词=0）；逐段通读全篇，第 4 题两个子情形都写「Exit code: 124 (standard timeout exit code)」，没有出现 s1/s2/s3 那处缺字/变形词，也没有别的粘连词、缺词或断句 |
| s5 | 0 | 497 | 无 | 干净——闸判绿（生词=0）；逐段通读全篇，未见缺字、粘连词、缺词或断句 |

累计干净样本：s4（重跑）、s5，共两份，达到「至少两份干净样本」的收尾条件，停止调用。

## 调用次数核算与一处超额

本轮共发出 6 次 `ask-local.sh` 调用：s1、s2、s3、s4（判红作废）、s4（重跑）、s5。按定义第 6 步「连续五次调用（判红作废的也算）」，第 5 次已是 s4 的重跑；发第 6 次（s5）之前没有先核对次数，发出之后才发现已经过了 5 次——这是本轮执行上的一处失误，如实记在这里。s5 的结果（干净）已经拿到，本轮到此为止，不再发第 7 次。

## 没有走到的步骤

- 5b（算术核对）：主 agent 这一轮没有另给算好的答案表路径，按定义写「没有答案表，算术没比」，没有跑逐格比数的 python。
- 本轮不涉及钩子、写范围闸或工具层（那是云端攻方的攻击面），运行记录里没有钩子判定要贴。
