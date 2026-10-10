# 删除批第 33 批 T（core 那一簇）合入的代码：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 T1 T2 T3 T4 T5 T6 T7 T8 F1 F2 A B C D E S1 R2 U1 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / diff / 背景材料 | `research/prompts/_m3-batch33t-code-r1-body.md`、`_m3-batch33t-code-r1-diff.md`、`_m3-batch33t-code-r1-background.md` | 材料第一次跑漏了 `.claude/kb/invariants.md`，半成品挪进会话草稿目录 `t-materials-first-try/`，加 `--files` 重跑那一份是腿读的 |
| 开工快照 | `research/prompts/m3-batch33t-code-r1-snapshot/crates-sha256.txt`（22 份）、`kb-sha256.txt`；冻结副本 `~/.local/share/singlefs/work/m3-batch33t-code-r1/tree/`（T 合后、根环线 10y 合入前） | 腿跑着的时候主 agent 没改快照里的 22 份 crates 文件。别的会话改了快照里两份 kb：根环线 2026-10-08 把它的 K4-2 写回 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14（第 390 行那一段 B 凭据放行句后补一句）与 `.claude/kb/decisions/16-发布语义.md` 已定项 1（第 37 行末补同义句、依据段加一条），各一处；两条腿引的 D16、D23 行是背景材料里冻结的那一版，判决只用腿引的代码与产物，不用那两处补句 |
| 云端正推（Sonnet） | `research/prompts/m3-batch33t-code-r1-sonnet-output.md`（187 行，sha256 496a55e3…），模型 `…-sonnet-model/` | — |
| 云端攻方（Opus） | `research/prompts/m3-batch33t-code-r1-opus-output.md`（259 行，sha256 ce5510b9…），模型 `…-opus-model/` | — |
| 引文 | 两份腿报告 | `cite-check.py --root <冻结副本> --root . --background <背景材料>`：正推「核了 97 处引文，对上 97 处，没判 0 处」；攻方「核了 22 处引文，对上 22 处，没判 0 处」 |
| 主 agent 复跑：正推 | 模型拷进会话草稿目录 `m3b33t-sonnet-model/`，`SINGLEFS_REPO=<仓根> rerun.sh ~/.local/share/singlefs/work/m3b33t-sonnet-rerun`，再 `compare.sh <复跑 out> <它的 out>` | 「probes.txt 全同（38 行）」「mutations.txt 全同（19 行）」「control.txt 全同（4 行）」「static.txt 全同（2 行）」 |
| 主 agent 复跑：攻方 | 模型拷进会话草稿目录 `m3b33t-opus-model/`，`SINGLEFS_REPO=<冻结副本> SINGLEFS_TOOLS=<仓根> THREADS=15 rerun.sh ~/.local/share/singlefs/work/m3b33t-opus-rerun`，再 `compare.sh <summary.txt>` | 「全同（1280 行）」，汇总 sha256 3a5464f7… 与它 `expected/summary.txt` 相同；第二节引的行原样都在 |
| 主 agent 现查 | 冻结副本 `crates/singlefs-core/src/recovery.rs` 第 5380–5401 行（F2 退到「比所选根旧、不超过上限的最新可读根」，上限读不出时取所选根自己）、第 3525–3540 行（`every_named_unit_of_the_record_verifies` 要每条位置条目都验过，F1 与重放的点名单元验证共用这一份）、`crates/singlefs-harness/src/mount_transparency.rs` 第 195–213 行（`read_recovery_fields` 断言再择根重放落到 `report.effective_root`） | 坐实 |
| 根环线 m3-batch10y-code-r1 判决 K4-1 | `research/prompts/m3-batch10y-code-r1-main-verification.md`；它正推模型 `research/prompts/m3-batch10y-code-r1-sonnet-model/out/k4-units1.log`（没打补丁时 M1 之后 R 的单元被写掉）、`k4-units2-patched.log`（拿掉 C 那一圈的候选豁免后写掉 0 份） | 主 agent 读日志坐实；与本轮正推「发现 2」同一形 |

两条腿的数都是腿副本上量的，不进 kb。

## 二、逐格判决

| 格 | 正推 | 攻方（一次抽样） | 判决 |
|---|---|---|---|
| K1 T1 C 的集合、水位、解开、重读不解、抬 F 重算、第九项、影子账开关 | 一致（七处） | — | **采** |
| K1 / K2 T1 C 的候选豁免 | 发现 2：R 的账读得出时 R 是候选根，C 把它点名的槽整个跳过（冻结副本 `mount.rs` 第 1566 行）；D、E 形写掉 R 的 7 个单元 14 份、池级 checker 红 I-7.2 / I-7.4；对照拿掉豁免后写掉 0 份 | — | **坐实**（同根环线 K4-1）。病根是主 agent 在 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 3 条与 D28（挂载期承诺量） 已定项 1 第九项写回里给 C 那部分加的「不被回退候选集里的根引用的槽」——用户定的 C 是全部、判决选法 `all`；去掉豁免回到用户定案。进下一批 |
| K2 T1 C 的盲区 | — | **打中**，比正文写的宽：根槽加末条、中间一条或第一条记录读回全 0、单元读得出时 F1 不拒，C 看不见那条记录点名的单元，M1 之后第一次写数据写掉，深度 3 每种藏法 58 个结点里 48 个红 I-7.4 | **坐实**（复跑原样同上）。进下一批照攻方 `C_ACCOUNT`：水位之上每次没施加的发布，用它任一条读得出的记录的新根段拼一条根、按影子账同一份读法读它的账，账里的落点并进 C 的集合（读不出就不加，交 F1）；零轮 |
| K2 T1 假 ENOSPC 无界、两种次序分叉 | — | 没打中（次序分叉推的：要连续两次发布根槽与全部记录都藏、再藏系统配置较新槽） | **记「没打中（一次抽样）」** |
| K1 T2 B、T3 第九项改名、T6 C695、T7 C692、T8 变异与新测试 | 一致；T7 15 / 15 个几何预测值与实有逐盘相等，400 次混合写最小值 22、下限 12；追加 18 行逐行打进去全红 | T7 没打中：34 段随机历史 1263 步每步恢复不红，记录数比下限至少多 8 | **采**；T7 没打中记一次抽样 |
| K1 / K2 T4 F1 | 一致：判法、正常崩溃分得开；记：F1 序号判法比池级 checker I-8.9 更严、不是同一个函数；F1 的拒被 `mount_transparency.rs` 第 2251 行分类成「不在 D23 单子上」（推的：崩溃注入第二截会把一次按条款该拒的 F1 拒报红） | 纯正常崩溃没打中（写记录那一段 64 个子集都挂得上）；**打中**单故障：写记录中途断电、没落根的那次发布有一个单元的一份副本坏了，每次可写挂载都拒、没有出口——F1 要求每一份副本都验过（冻结副本 `recovery.rs` 第 3534 行一带、`mount.rs` 第 6012 行一带） | **坐实**（复跑原样同上）。进下一批照攻方 `F1_ANY`：「点名单元验不过」改成某个点名单元两份都验不过；只改 F1 自己的判法，重放的点名单元验证不动（两处共用 `every_named_unit_of_the_record_verifies`，要拆）；`mount_transparency.rs` 把 F1 的拒归到条款认可的那一类；零轮 |
| K1 / K2 T5 F2 | 一致：三处同一份判法、「见证过」判法与 D23 一致、三处报告一致；**冲突**：`read_recovery_fields`（冻结副本 `crates/singlefs-harness/src/mount_transparency.rs` 第 210 行）断言「再择根、再重放落到 `report.effective_root`」，F2 退回后两边不等、panic（它 `out/probes.txt` 第 30–31 行 S1 两种 `ComparedVersion` 都 panic；调用方含 checker 档 `crash_injection.rs` 第 1485、3391 行）；推的：F2 没有 N-配置「c_见证 = 0 不判」那一支、退到的候选不按实例表滤；排演函数在盘比单元区起点短时 panic | **打中**两形（双故障）：A 形未见证的 R 走读失败、同时已确认的 V 根槽读回全 0，三处都落到 V 之前那一版、可写挂载还把 V 抛弃（8 / 8 组；关掉 F2 报读不出）；B 形见证计数器那条记录读不出，上限退成所选根，落到被新实例实例表抛弃了的 R；**打中**层 0 oracle（`crash.rs` 第 1111 行）把 F2 设计里那一退也判 `RecoveredToAnOlderRoot`，记录核对器两形都不报 | **坐实** panic 与 A、B 两形（主 agent 现读第 5380–5401 行：退到「比所选根旧、不超过上限的最新可读根」，上限读不出取所选根自己）。进下一批：照攻方 `F2_EXACT` 只退到 c_见证 那条记录所属那一版本身（环里恰好相等、读得出的那条根），那条记录读不出就不退——这是往用户定案原话「退回见证到的那一版」收严，A、B 两形变回报读不出、设计那一退照旧落 V、已有 4 条 F2 用例过（攻方量的）；c_见证 = 0 时没有见证到的记录、不退（随 `F2_EXACT`）；`read_recovery_fields` 的期望按同一份 F2 判法算；排演函数在小盘上不 panic。零轮。oracle 不改：它判红的那一形是根槽持久而单元不在持久集合里，纯崩溃造不出（单元在根槽之前、隔着屏障），只在读故障下走到；提交时快档看崩溃注入流（登记第 47 行）碰不碰到，碰到再判 |
| K1 T8 小条 | `mount.rs` 第 1790 行与 `allocator.rs` 第 266 行各留一句旧注释；一条测试函数名带日期来历（`user_accepted_cost_of_2026_10_07_…`） | — | **采**：注释改实话；测试名去掉日期（`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用例名写场景和预期，不带里程碑、步号、批次这类来历」，日期同属来历） |
| K3 | 22 份每份一格：`mount_transparency.rs` 冲突、`recovery.rs` 带规则没说、其余一致 | — | **采** |

## 三、去向

1. 删除线 core 修补批（K4-1 与这一轮打中的 core 部分，形式化线拍树之前合）：去掉 C 的候选豁免；`C_ACCOUNT`；`F1_ANY`（拆开与重放共用的那一份）；`F2_EXACT`；`read_recovery_fields` 按 F2 判法算期望；F1 的拒在 `mount_transparency.rs` 归类；排演函数小盘不 panic；两处旧注释；测试名去日期。每条配坏盘面用例与变异。合入之后根环线 10z 第二块在那一版上取副本（两边 2026-10-08 约定）。
2. 删除线 GPU 那一批（同时派、不撞文件）：GPU 恢复内核照 CPU 的 F2（`F2_EXACT` 那一形）补那一支、逐位同判（二分坐实对拍 52–53 条红由 T 的 F2 带进来，`~/.local/share/singlefs/work/bisect-gpu-red/`）；连同 m3-batch33s-code-r1 判决第三节第 1 条与根环线托的 `lease_card_waiting_for_room` 那件。
3. kb：D23（journal 的角色与格式） 已定项 14 与 D28（挂载期承诺量） 已定项 1 第九项去掉 C 的候选豁免、写进 `C_ACCOUNT`、`F1_ANY`、`F2_EXACT`，在根环线 K4-2 补过的那一版上改。
4. 不做：oracle 认 F2 那一退（理由见第二节 T5 行）；F1 序号判法与 I-8.9 合成一个函数（池级 checker 与实现分开写是 D13（验证路线） 已定项 5 要的）。

`C_ACCOUNT`、`F1_ANY`、`F2_EXACT` 只在攻方模型上量过、被攻过零轮；去掉候选豁免在根环线正推与本轮正推的模型上各量过一次。

## 四、交用户

不交：打中的几处都有腿给的、往用户定案收严的改法；K4-1 是主 agent 起草时多加的豁免，去掉即回到用户定案。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 不受影响：C、F1、F2 的写法收严（第三节第 3 条），条款意思不变，下一批写回 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-08 不受影响：第九项去掉主 agent 多加的候选豁免，下一批写回 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-10-08 不受影响：C 的解开时机照旧 |
