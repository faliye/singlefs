# 崩溃状态逐写独立判：第二轮判决（2026-11-06）

正文：`research/prompts/_per-write-judging-r2-body.md`。四份腿报告：攻方 `research/prompts/per-write-judging-r2-opus-output.md`（写模型与预测）、`research/prompts/per-write-judging-r2-opus-s2-output.md`（读复跑产物）；辩方 `research/prompts/per-write-judging-r2-sonnet-output.md`（复核与收严判据）、`research/prompts/per-write-judging-r2-sonnet-s2-output.md`（读复跑产物）。

## 核查怎么做的

- 复跑：
  - 攻方模型：任务 8699（第 3 次尝试退 0），产物拷进 `research/prompts/per-write-judging-r2-opus-model/out/`（counterexamples.txt 440 行、user_actions.txt 645 行、unseen_shapes.txt 447 行、small_segments_exhaustive.txt 792 行）。
  - 辩方探针：任务 8700（第 4 次尝试退 0），冻结副本用攻方那一份（里面没有辩方的探针），产物拷进 `research/prompts/per-write-judging-r2-sonnet-model/out/`。辩方读产物那条腿报：`tools/rerun.sh` 第 95 行的 `grep -h '^PROBE_'` 漏了每个测试第一行，`out.txt` 比各 `out-*.txt` 少 4 行；判决引的整行都取自各 `out-*.txt`。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/per-write-judging-r2-opus-s2-output.md research/prompts/per-write-judging-r2-sonnet-s2-output.md --root /home/fy5090/code/singlefs --root /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/rerun-work --root /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-defense-rerun --background research/prompts/_per-write-judging-r2-background.md`，末行「✓ 核了 165 处引文、131 处裸行号，对上 161 处，没判 135 处（其中裸行号没有引文可比 120 处；逐处列在上面）」。只给仓根时四份报告合起来「核了 229 处引文…对上 75 处」，没对上的是产物在仓外、路径认不出。
- 主 agent 现查的一行（辩方探针产物）：`research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt` 里 `PROBE_TIGHT_FLOW label=unmodified segments=36 unit_segments=7 passing_unit_segments=2 engine_total=1463812286 reduced_total_group_form=1379926285`。

## 逐条判

| 论点 | 判 | 依据 |
|---|---|---|
| 辩方收严判据 E1–E6 在真实删除流上能把态数降到 471 | 不采，量出来不是 | 上面那一行：7 个单元段只有 2 段通过，整条流从 1,463,812,286 只降到 1,379,926,285。辩方读产物那条腿：24 个真实单元段里 E2、E3、E4 各列全是 0，只有 E6 有读数，16 段为 2 |
| E6 在真实流上的基线 2 是判据宽度造成的误报，还是真有记录点名这一段要写的槽 | 未定，第三轮核 | 辩方读产物那条腿：产物分不开 |
| 收严之后判据收下的段上，候选乙与全组合等价 | 只在极少取样上量过 | 判据之内有量的只有段 20 的 3 次、共 3072 个状态（辩方读产物那条腿） |
| 反例 A（一对）：甲的两种合成、乙的并合成在红项层与全组合不同，乙的增减合成两层都不差 | 采（取样） | 攻方读产物那条腿逐格引了 counterexamples.txt 与 small_segments_exhaustive.txt 的行；错法全是「全组合判绿、代表判红」，没有一态漏红 |
| 反例 A（两对）分辨四种合成 | 分辨不了 | 四种合成逐格同数；能分辨它们的「两组都整组落」不在任何一种合成判的态里（判别子观测不到） |
| 反例 B（记录先写、单元后写，甲漏红） | 没造出，不采为「没有」 | 只抽了一次 |
| 两组同时被盖时，乙在红项层与全组合不同 | 采（量的） | 辩方读产物那条腿：91 对里 78 对有差（243/1456 个状态） |
| 混合种类段、多组被盖两臂上四种代表法都有差 | 采（量的），但这两臂判据不收 | 攻方读产物那条腿 |
| 第一轮判决 11 行都站得住，4 行打中的射程要缩小 | 采 | 辩方复核；transaction.rs:10564、walk.rs:17931 两处辩方逐行核过 |
| 方法限定：判的态只占被判段全量约万分之一到万分之二，合计 464438 态 | 采 | 攻方读产物那条腿 |

## 结论

第二轮也不立判据。量出来两件对方案不利的事：

1. 要保证逐写判与全组合等价而收严出来的判据（E1–E6），在真实删除流上几乎什么都收不下：7 个单元段只通过 2 段，态数只降了约 6%。卡住其余段的是 E6（前缀里有 journal 记录点名这一段要写的槽）。
2. 判据放宽到能收下那些段，两组同时被盖时按单元副本组判在红项层就会与全组合不同（91 对里 78 对）。

所以「逐写独立判」要有用，取决于 E6 那 16 段的读数是不是误报：是误报，就能把 E6 收窄，那些段才收得下；是真有记录点名，这条路在今天的流上就降不下态数。这是第三轮唯一要定的事。

第三轮（最终轮）照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」分工（「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核）：只攻 E6 基线那 16 段——读出每段被点名的槽与点名它的记录，判是宽度误报还是真点名；若是误报，给出收窄后的 E6 与它在真实流与对拍三条流上的态数，并在那几段上逐态（不取样）核与全组合等价。第三轮之后不再开，交用户定。

## 交用户的表

| 项 | 状态 | 说明 |
|---|---|---|
| 收严判据 E1–E6 | 被攻过两轮，没立 | 真实删除流只降约 6%（量的） |
| E6 基线 16 段 | 未定 | 第三轮核误报还是真点名 |
| 候选乙（按单元副本组判、增减合成） | 被攻过一轮 | 反例 A 一对两层不差（取样）；两组同被盖时红项有差（量的） |
| 候选甲（逐写判新旧） | 红项层不够（量的） | 第一轮与本轮都量到 |

## 回看决策

不涉及决策：这一轮没有立判据，D13（验证路线）的判法没有改，任何决策分项都不写回。
