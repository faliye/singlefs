# 运行记录：`gate-fix-forks-r1-local-attack`（轮 gate-fix-forks-r1，本地攻方腿）

这条腿是接续腿：同一轮、同一立场的上一条本地攻方腿（子 agent a709ac0c84b604db0）进程被杀，
未交回。交接摘要：`research/prompts/gate-fix-forks-r1-local-attack-handover.md`。
提示文件、译文核对表、void1/void2、s1/s2 四份产出在接手时已落盘，未重做，
本条腿只核实了这些既有产物的完整性（重跑 `corruption-check.py`／`oov-check.py`、通读全文），
判断了 `-output-s3.md`（0 字节）是否需要补跑，并写这份运行记录。

核对表：`research/prompts/gate-fix-forks-r1-local-attack-translation-audit.md`（覆盖 T4、T7、T8、T9、T10，共 5 节）。

提示文件 `research/prompts/gate-fix-forks-r1-local-attack.md` 当前 sha256：
`9256574228da42ec60882f2fb524406a014fb5dc6449f9e5f9cc5340e5ec8b5a`（736 行），
与产出 s1、s2 所用的提示文件一致（两次调用均在这份提示最后一次落盘之后发起）。

## 每次调用

| 号 | 文件 | 退出码 | 词数（`wc -w`） | 生词清单（`oov-check.py`） | 判定 |
|---|---|---|---|---|---|
| void1 | `gate-fix-forks-r1-local-attack-output-void1.md` | 5 | 2033 | 未跑到（`corruption-check.py` 先判红：实词自复读 `category`，`ask-local.sh` 已退出，没到 `oov-check.py` 那一步） | 作废 |
| void2 | `gate-fix-forks-r1-local-attack-output-void2.md` | 5 | 1759 | 未跑到（同上：实词自复读 `category`） | 作废 |
| s1 | `gate-fix-forks-r1-local-attack-output-s1.md` | 0 | 2833 | 生词=5：apparatus measurement's exemption vocabulary（拼接=0） | 干净 |
| s2 | `gate-fix-forks-r1-local-attack-output-s2.md` | 0 | 2100 | 生词=10：exemption apparatus（拼接=0） | 干净 |
| s3 | `gate-fix-forks-r1-local-attack-output-s3.md` | 137（外部 SIGKILL 杀掉了上一条腿的整个进程，不是 `ask-local.sh` 自身返回的码；该次调用未产生任何 stdout/stderr） | 0 | 未跑（进程未完成，没有正文） | 不计入三档，视为未完成的调用，不算一次观测 |

s1、s2 由本条腿重跑 `python3 research/scripts/corruption-check.py <样本> <提示文件>` 与
`python3 research/scripts/oov-check.py <样本>` 复核，结果与上一条腿记录的一致（均判绿，生词清单不变）；
另外通读了 s1、s2 全文（17 题逐题读完），未见缺词、断句、成对标记落单或粘连词形态的损坏。
void1、void2 由 `ask-local.sh` 内建的 `corruption-check.py` 判红（实词自复读 `category`），
按规则作废，未再对其内容做额外核查。

## s3 是否需要补跑

不需要补跑。判据（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」与
本定义第 6 条「攒到至少两份干净样本为止」）：
s1 于该轮 14:53:29 confirm 判绿、s2 于 14:57:59 confirm 判绿——两份干净样本在
s3 那次调用发起（14:58:27）之前已经齐了，门槛（至少两份干净样本）已满足。
s3 那次调用是上一条腿在门槛已满足之后另起的第三次调用，尚未完成即被外部 SIGKILL
杀掉整个进程（退出码 137，不属于 `ask-local.sh` 自己会给的退出码 0/2/3/4/5 中任何一个，
也不是网关不通——s1、s2 两次成功往返已经现查证明网关在同一时段是通的），
留下一个 0 字节的空文件。这不是「判红作废」（没有红判记录），也不是一次有效观测
（没有正文、没有退出码判定），因此不计入「连续五次调用」的计数，也不需要沿用同一个号重跑：
门槛已经用 s1、s2 满足，继续跑第三次不会改变「已有两份干净样本」这个结论。
0 字节的 `-output-s3.md` 原样留在原处，未删除、未重写。

## 没做什么

- 未重新生成提示文件与核对表（已由上一条腿写好，按接续指令原样保留，未改一个字）。
- 未对 void1、void2 的正文做逐句通读（作废样本，规则不要求）。
- 未判本轮 T4/T7/T8/T9/T10 各题两份干净样本的答案方向是否一致——那是主 agent 的事，不是这条腿的事。
- 未跑 54、55、57、59、87 号门禁，未改仓里其它文件。
