# m3-close-g4-rewrite-r1 判决（推论轮第一轮，2026-10-21）

正文 `research/prompts/_m3-close-g4-rewrite-r1-body.md`，背景材料 `research/prompts/_m3-close-g4-rewrite-r1-background.md`（sha256 9dac765fa0dcf467aa49497cbf528f145d2bf02b426f6b4fff9746c43acb061c）。开工快照 `research/prompts/m3-close-g4-rewrite-r1-snapshot/`（`code-sha256.txt`、`kb-sha256.txt`）。

两条腿：
- 正推（Sonnet）：`research/prompts/m3-close-g4-rewrite-r1-sonnet-output.md`，sha256 bfe62e8548c0880077b41777af4ff8c5435274d7ee23cec3e6e7e1ea37a4de3a。
- 攻方（Opus）：`research/prompts/m3-close-g4-rewrite-r1-opus-output.md`，sha256 54c63ef081d66f03da2e735bd6ea8e9da7f40b8acef76b0992625a66c610daea。模型目录 `research/prompts/m3-close-g4-rewrite-r1-opus-model/`。

## 一、核查

1. 快照：腿交齐之后 `sha256sum -c` 两份清单，代码六份都 OK；kb 只有 `.claude/kb/checks-owed.md` 对不上。它是别的会话（doc 门禁会话的阶段同步）照约定定点改的，替换清单在 `~/.local/share/singlefs/work/batch-2026-10-21-writeback-a/replacements.tsv`。两份腿报告里 `checks-owed.md:<行号>` 的引用 0 处（`grep -no 'checks-owed\.md:[0-9]*'` 两份都空），腿引的行都不落在被改的那份里，所以没有反推。
2. 引文：`python3 scripts/agents/cite-check.py <两份报告> --root . --background <背景材料>` 末行「✓ 核了 52 处引文、49 处裸行号，对上 52 处，没判 49 处（其中裸行号没有引文可比 49 处；逐处列在上面）」。判决用到的行我逐处读过：
   - `.claude/kb/decisions/13-验证路线.md:187`（D13（验证路线） 已定项 6 那一节）原话：「「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由 G4 判（崩溃注入第二、三截，与 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那条同接法）：mkfs 返回之前的崩溃态不在 G4 的射程，mkfs 逐盘两槽在同一个循环里写、之后才逐盘屏障，崩溃注入的候选段与 E171（三oracle合谋测试） 的基镜像都取在 mkfs 之后」。
   - `crates/singlefs-harness/src/mount_transparency.rs` 改法 G4 那一臂的注释：「改法 G4：没有读故障时设备不报读错，某块盘两槽都自证不过只能是盘面本身如此——正确实现在没有读故障的历史里留不下它。」它在 `judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount` 里，只在可写挂载被拒、拒因是「读阶段判完之后系统配置又读不出」时被调到。
   - `crates/singlefs-core/src/mount.rs` `devices_without_the_selected_version`：两槽全废、没有读错的槽、补写闸五件都过，就把盘记进补写的盘，挂载往下走。
3. 攻方在副本原型上量的数（mkfs 末段 6 态与 40 态、MUT-B1 的 42 态全挂上、`violations: 0`）判决里只当线索，不当已核的数：原型调的是 checker 档的崩溃枚举，复跑是重型，这一轮没复跑。下面的判定都落在读过的代码与条文上。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| V1 | 两腿一致：mkfs 之后的历史留不下「某块盘两槽都自证不过」，mkfs 末段（四个槽写同段、没有屏障）留得下 | D13 第 187 行已把 mkfs 返回之前的崩溃态排出 G4 的射程，与此相符；G4 那一臂的注释（「正确实现在没有读故障的历史里留不下它」）是全称句，说过了头，要收窄到 D13 第 187 行的射程 |
| V2 | 判法本身在调用域内不改；打中一处空档 | 调用域（崩溃注入第二、三截，mkfs 之后的崩溃态）里没有两槽全废的盘面，所以「拒因为真即不合法」照旧对（两腿一致）。空档：D13 第 187 行交给 G4 的那条性质，G4 只在被拒时判；补写闸放过的两槽全废盘面挂上了就没人判。这是从代码结构读出来的（G4 那一臂只在拒绝那一支里），两腿各自独立指出，提的改法也相同（正推叫「按盘面判」，攻方叫 G4-丙：崩溃态没注入读故障、某块盘一份本池自证过的系统配置都没有就记一条发现，不管拒没拒）。这条改法被攻过零轮 |
| V3 | 测试走甲 | 两腿一致：甲在任何一种 V2 判法下都对；乙等于甲加一条手造盘面的新测试，第 2408 行的判别力不比甲多。甲的盘面另坏盘 0 上所选那一版的一个单元（闸一不过），断言不变，文档注释写明「补写闸闸一不过」。攻方另提甲-二（同一张闸过的盘面在补写闸恒不过的副本上断言 G4 报，再加一行变异「补写闸恒不过」），零轮，与 G4-丙 一起进第二轮 |
| V4 | C670、Q3 两臂不受影响 | 两臂的拒因出在读阶段与重建分配器，早于补写闸（两腿一致，读码）。正推另报：Q3 第三截的计数没有测试断言、变异表没有行点名，与丙' 无关，记下不在这一轮改 |

## 三、改法

### 进批 14 改红批（只改测试代码与注释）

1. `crates/singlefs-checker-tier/src/crash_injection.rs` 那条红的测试走甲：盘面另坏盘 0 上所选那一版的一个单元，让补写闸闸一不过；断言不变；文档注释写明闸一不过。验收：这条测试绿；`crates/mutations.tsv` 第 2408 行改回去它就红（实现员证红）。
2. `crates/singlefs-harness/src/mount_transparency.rs` G4 那一臂的注释与 `RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults` 的文档注释，收窄到 D13 第 187 行的射程（mkfs 之后的崩溃态）。只改注释。

### 第二轮再攻（被攻过零轮）

- G4-丙（按盘面判）与甲-二（反向测试加一行「补写闸恒不过」变异）。第二轮攻这两样站不站得住：盘面判法在 mkfs 之后的崩溃态上会不会误报、要不要读补写闸、与 C670、Q3 两臂的接法。站住了，下一批实现。

### 记下、不在这一轮改

| 项 | 去向 |
|---|---|
| 池级 checker I-8.5 的旁路说明仍写「可写挂载会拒」（`crates/singlefs-checker/invariants/03-journal.md`、`crates/singlefs-checker/src/image.rs`），丙' 之后过闸的盘不拒 | `image.rs` 在 m4-i315-r3 的开工快照里，等那一轮判完与 C773 同批；不变量文字进 kb 写回批 |
| D23 已定项 14 那一句与判官判据都没带补写例外 | 第二轮一起判要不要改条文 |
| Q3 第三截计数没有测试断言、没有变异行 | 记下，下一批 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 支撑 | 2026-10-21 不受影响：挂载透明的射程照旧只管不带故障的状态，G4 那一臂在调用域内判法不变 |
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-21 不受影响：第 187 行「每块盘至少一份自证过的系统配置由 G4 判」那一句照旧；G4 只在拒绝时判留下的空档交第二轮（G4-丙），第二轮站住再改这一句 |
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-10-21 不受影响：补写例外照用户定案落地，这一轮只判它落地之后判官与测试怎么改 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-10-21 不受影响：拒因单子不变；条文要不要带补写例外交第二轮 |
