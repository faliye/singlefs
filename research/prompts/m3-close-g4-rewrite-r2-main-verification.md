# m3-close-g4-rewrite-r2 判决（推论轮第二轮，2026-10-22）

正文 `research/prompts/_m3-close-g4-rewrite-r2-body.md`，背景材料 `research/prompts/_m3-close-g4-rewrite-r2-background.md`（sha256 7ec4fb72da8a483674a2e36bbda61b92a0600be943f23c17b9a34195d1221e03）。开工快照 `research/prompts/m3-close-g4-rewrite-r2-snapshot/`（`code-sha256.txt`、`kb-sha256.txt`）。上一轮判决 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`。

两条腿：
- 辩方（Sonnet）：`research/prompts/m3-close-g4-rewrite-r2-sonnet-output.md`，sha256 1e2d539793a0f4431d706e547dc6124f73e8fc922a6cedfe0e90556abfdc1391。模型目录 `research/prompts/m3-close-g4-rewrite-r2-sonnet-model/`。
- 攻方（Opus）：`research/prompts/m3-close-g4-rewrite-r2-opus-output.md`，sha256 dffe0509f14a5c50b6b8f86336c9fe19a6f102bbe5d335f1c7d572dea29f2bd5。模型目录 `research/prompts/m3-close-g4-rewrite-r2-opus-model/`。

## 一、核查

1. 快照：两腿交齐之后 `sha256sum -c` 两份清单。
   - 代码五份都 OK。
   - kb 里 `.claude/kb/decisions/22-单元原子性怎么合成.md` 与 `.claude/kb/checks-owed.md` 对不上，改它们的是本会话批 15 的 kb 写回（替换清单 `~/.local/share/singlefs/work/m3-close-w15-kb-writeback/replace.json`；D22 4 处、欠账表 3 处），另有主 agent 把已定项 26 进度那一处反向替换回去。
   - 倒推：先把那一处反向替换撤掉，再把写回员的 7 处反着做一遍，两份的 sha256 与快照相同（D22 40728119…、欠账表 c3b73d6f…）。倒推出的副本在 `~/.local/share/singlefs/work/m3-close-g4-rewrite-r2-d22-snapshot-copy.md`、`…-owed-snapshot-copy.md`。
   - 腿引这两份的行：D22 第 181、190 行（两腿都引），在倒推出的快照原样里逐字找到；欠账表第 578 行（两腿都引来说 C873 已被占），引的是工作区此刻的样子，主 agent 现查第 578 行是「| C873 | 隔离记录两盘不对称没人判 |」。
2. 引文：`python3 scripts/agents/cite-check.py <两份报告> --root . --background <背景材料>` 末行「✓ 核了 100 处引文、28 处裸行号，对上 100 处，没判 28 处（其中裸行号没有引文可比 25 处；逐处列在上面）」。判决用到的行主 agent 逐处读过：
   - `crates/singlefs-checker-tier/src/crash_injection.rs` 里 `grep -c Tearable` 为 0：崩溃注入第二、三截摆的是整写子集，不摆撕裂。
   - `crates/singlefs-harness/src/mount_transparency.rs:2374`：「if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {」；同文件第 36 行从核心层引入 `choose_system_configuration`，第 2422、2454 行起调它。判官判槽自证用的是核心层的那一段。
   - `crates/singlefs-checker/src/image.rs:1492`：「pub fn system_configuration_redundancy_report(」，池级 checker 自己的读法。
   - `crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:159` 的 `a_device_holding_the_version_whose_both_slots_were_zeroed_is_rewritten_by_the_acquisition_and_the_checker_stays_green`，`crates/mutations.tsv` 第 5010 行点名它。
   - `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3529` 按 `RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults` 写了一臂。
3. 复跑：
   - 辩方模型拷到 `~/.local/share/singlefs/work/m3-close-g4-rewrite-r2-main-checks/model-copy` 经 `run-with-memory-cap.sh 2G` 跑它的 `tools/rerun.sh`：「name=rerun_compare file=explore.out sha256=7b6b7514c4f204f9979d30c6717dc44fd0f1aea47664169b75e110899597a094 same=yes」。
   - 攻方原型要现编、整趟复跑约 20–30 分钟（腿报的，推的），此刻本机 IO 压力 full avg300 85.38（`/proc/pressure/io`），没复跑。它报的数（W1 四组历史合计 172 366 态零发现、MUT-B2 撕裂域 16 个单盘全废态、MUT-B1 整写域 8 态与撕裂域 34 态）判决里只当线索，不当已核的数；第三轮「报出的数字能不能核」那一格要它们在入库装置或复跑过的原型上坐实。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| W1 | 两腿都没找到误报，不记「没打中」 | 辩方读码加槽协议模型（复跑 same=yes），攻方原型四组历史零发现（只抽一次）。照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，「G4-丙 不误报」这一句不作结论。辩方另指出：域内补写根本不执行，「补写本身中途崩溃」那一类 G4-丙 罩不到，仓里也没有补写的崩溃枚举流 |
| W2 | G4-丙 按正文写的形态站不住 | 三处，两腿一致：① 挂在崩溃注入只看得见整写子集（`Tearable` 0 命中，主 agent 现查），撕裂域留得下的单盘全废态它判不到（攻方 MUT-B2 的 16 态，线索）；② 「无槽且闸不过」那一形与 G4 拒绝那一臂一件事记两条；③ 判自证用核心层的 `parse_slot` 与 `choose_system_configuration`，与被判的实现共用同一段算法，违背 `.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」；池级 checker 有独立读法（`image.rs:1492`）。第 ③ 条今天的 G4 拒绝那一臂同样有（`mount_transparency.rs:2374`），不是 G4-丙 才带进来的 |
| W3 | 甲-二 不做 | 两腿一致：判官收到的输入里没有补写闸，甲-二 在判官一侧多罩为零（辩方读码；攻方在两张盘面上判官结论逐字段相同，线索）；「补写闸恒不过」那一类改坏由现成的正向测试（第 159 行那条，`crates/mutations.tsv` 第 5010 行点名）杀，主 agent 现查两处都在。攻方另报（读码）：补写那一支进不了崩溃测试、补写闸哪一件没过运行时看不出，分支纪律第 3、4 条没满足，「恒不过」开关对这两条都不起作用 |
| W4 | 条文要改，怎么改等第三轮 | 两腿一致：D13（验证路线） 第 187 行「由 G4 判」与「与 C670 那条同接法」在补写例外之后说过了头；D23（journal 的角色与格式） 第 396 行拒因单子不带补写例外，与 D22 已定项 8 不一致。攻方另指：D23 改了而判官复算不跟，D13 第 72 行对这一拒因不再成立。改成什么取决于第三轮判的判官形态，这一轮不写回 |

辩方「撤掉 G4 拒绝那一臂撤不得」（E171 装置按这个成员写了一臂、变异第 2406–2408 行押在它上面）与攻方的丙-撤（撤掉、三行变异改指新判法）相反，交第三轮。

## 三、去向

### 这一轮定的

- 甲-二 不做。批 15 根环实现员手上没有它，不用改派发。
- 新 C 号：C873 已被占（欠账表第 578 行），这一会话之后取号从 C874 起，取号前现查。

### 第三轮再攻（被攻过零轮）

- 丙-置：G4-丙 改挂成崩溃点重放的观察者，只判 mkfs 返回之后的段（撕裂域在内），判槽自证读池级 checker 的读法（辩方的 G4-丁 与它同一处改法）。攻：撕裂域里正确实现留不留得下单盘全废态（W1 换到撕裂域重问一次）；观察者挂在哪几条流、判几次。
- 丙-择（择不到系统配置也记一条）；G4-戊（第三截只判新增的无槽盘）。
- G4 拒绝那一臂撤与留（丙-撤 对 辩方的撤不得），连同第 2406–2408 行三条变异与 E171 那一臂的去向。
- 判官判槽自证改读池级 checker 的读法：G4 拒绝那一臂今天就共用核心层算法，与丙-置一起判。
- D13 第 187 行、第 72 行与 D23 第 396 行条文。
- 攻方报的数（MUT-B2、MUT-B1、W1 四组历史）在入库装置或复跑过的原型上坐实。

### 记下、不在这一轮改

| 项 | 去向 |
|---|---|
| 补写那一支进不了崩溃测试、补写闸哪一件没过运行时看不出（分支纪律第 3、4 条） | 欠账表新立一行（C874 起取号），随下一份 kb 写回；改法并进第三轮判完之后的实现批 |
| 补写本身中途崩溃没有崩溃枚举流 | 同一行 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 支撑 | 2026-10-22 不受影响：挂载透明的射程照旧；G4 判官形态交第三轮 |
| D13（验证路线） 已定项 6 | 支撑 | 2026-10-22 不受影响：第 187 行要改，改成什么等第三轮，这一轮不写回 |
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-10-22 不受影响：补写例外与它的闸照用户定案，这一轮只判判官与测试 |
| D23（journal 的角色与格式） 已定项 14 | 支撑 | 2026-10-22 不受影响：第 396 行拒因单子要不要带补写例外交第三轮 |
