# m3-close-g4-rewrite-r2：判官改法 G4 改成按盘面判（G4-丙）与反向测试（甲-二）站不站得住（推论轮第二轮）正文（2026-10-22）

<!-- doc-lint:not-numbers G4 Q3 U4 U6 S4 R2 V1 V2 V3 V4 W1 W2 W3 W4 -->

## 一、题面

第一轮 m3-close-g4-rewrite-r1（判决 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md`）判下三件，都已改进主工作区：
- 那条红的 checker 档测试走甲：盘面另坏盘 0 上所选那一版的一个单元，补写闸闸一不过，断言不变。
- 判官 `crates/singlefs-harness/src/mount_transparency.rs` 改法 G4 那一臂的注释，收窄到 D13（验证路线） 已定项 6 写的射程（mkfs 返回之后的崩溃态）。
- 打中一处空档，留到第二轮：D13 已定项 6 把「无读故障的崩溃态上每块盘至少一份自证过的系统配置」交给 G4 判。但 G4 只在可写挂载被拒时判；取号补写例外（D22（单元原子性怎么合成） 已定项 8 依据里「取号那一步的补写例外」那一行）放过的两槽全废盘面挂上了，就没人判。

第一轮两条腿各自提了同一个改法，另有攻方的一个测试改法。两样都被攻过零轮：
- **G4-丙（按盘面判）**：崩溃态没注入读故障、某块盘一份本池自证过的系统配置都没有，就记一条发现，不管可写挂载拒了还是挂上了。挂在崩溃注入第二截的崩溃镜像与第三截的二次崩溃镜像上，各判一次。正推腿的实现形态：复用判官里已有的「某盘无自证槽」谓词（`system_configuration_unreadable_after_the_read_stage_on_the_crash_image` 里那一段，读槽走判官自己的读法），不读补写闸那五件。
- **甲-二（反向测试）**：同一张闸过的盘面，在补写闸恒不过的副本上断言 G4 报；再加一行变异「补写闸恒不过」。

这一轮要回答四件：
1. G4-丙 站不站得住：在 mkfs 返回之后、没注入读故障的崩溃态上会不会误报；要不要读补写闸；与 C670、Q3 两臂怎么接（同一张盘面上会不会一件事记两条、或漏一条）；判官与核心层不许共用同一段算法（`.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」）。
2. 甲-二 值不值得做：「补写闸恒不过」要在核心层留一个只供测试的开关（`.claude/rules/fs-design.md`「每条分支必须能被测试强制进入」），这个开关与它测的东西比甲多罩了什么；采了 G4-丙 之后，甲-二 还要不要。
3. D23（journal 的角色与格式） 已定项 14 的拒因单子与判官判据要不要写进补写例外（第一轮判决「记下、不在这一轮改」表第二行）。
4. D13 已定项 6 那一句「由 G4 判」：采 G4-丙 之后要不要改写、改成什么；不采的话，空档的账记在哪。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区：HEAD 841b3beab 加未提交的改动，含批 14 补丁；快照在 `research/prompts/m3-close-g4-rewrite-r2-snapshot/`）

- 补写闸：`crates/singlefs-core/src/mount.rs:5758` `devices_without_the_selected_version`，五件在 `mount.rs:5731` 的 `RewriteGateNotMet` 枚举、判法在 `mount.rs:5851` `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`。今天没有「补写闸恒不过」的开关：`grep -rn 'rewrite_gate' crates/singlefs-core/src/ | grep -i 'test\|force\|switch'` 零命中（主 agent 现跑）。
- 判官：`crates/singlefs-harness/src/mount_transparency.rs:3031` `judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`；G4 那一臂在 `mount_transparency.rs:3066` 起（注释已收窄到 mkfs 返回之后）；复算在 `mount_transparency.rs:2419` `system_configuration_unreadable_after_the_read_stage_on_the_crash_image`；拒得不合法的成员 `RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`（`mount_transparency.rs:2950`），Q3 那一臂紧接着（`mount_transparency.rs:2951`）。「某盘无自证槽」的谓词已经在判官里：`mount_transparency.rs:2406` `some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`；今天没有按盘面判的调用：`grep -rn 'device_without_any_self_verified_system_configuration' crates/singlefs-harness/src crates/singlefs-checker-tier/src` 只命中 `mount_transparency.rs` 第 2406、2416、2440 行，调用点只有复算里第 2440 行那一处，checker 档零命中（主 agent 现跑）。
- 测试：`crates/singlefs-checker-tier/src/crash_injection.rs:8968` 那一条已走甲（盘 0 两槽各翻一个字节，另坏盘 0 上所选那一版的一个单元）。崩溃注入第二截、第三截的调用点在同一文件（第一轮正推腿报告引的 `crash_injection.rs:3485`、`:4426`，行号已随批 14 漂，腿自己现查）。
- 变异：`crates/mutations.tsv` 第 2406–2408 行三条「第29批 G4」。
- 攻方第一轮量过的坏实现 MUT-B1（同一块盘的两个系统配置槽写进同一段）：42 个态全部挂上、今天零发现（`research/prompts/m3-close-g4-rewrite-r1-opus-output.md` 第 47、105 行；量在攻方副本原型上，主 agent 没复跑）。
- 同时在写、不在这一轮快照里的：根环一簇实现员正在副本里改 `mount.rs`、`mount_transparency.rs`、`crash_injection.rs`（C743、C751、C755、C866 与几处小项），补丁等这一轮判决写完再打；腿只按快照里的样子谈。

方案按 `crates/` 今天的实现来谈。

## 三、格

| 格 | 问什么 |
|---|---|
| W1 | G4-丙 的判据：「没注入读故障的崩溃态上，某块盘一份本池自证过的系统配置都没有」在崩溃注入第二截、第三截的全部崩溃态（mkfs 返回之后）上，正确实现留不留得下（逐类历史：取号写两轮中途崩溃、补写本身中途崩溃、二次崩溃、清扫与覆盖写）；留得下的就是误报，逐条写出历史与崩溃点 |
| W2 | G4-丙 的接法：挂在哪几处、每张盘面判几次；与 G4 拒绝那一臂、C670、Q3 三臂同一张盘面上记几条；采了它，G4 拒绝那一臂还要不要留（留着是否多余、撤掉是否丢判别力）；变异行要几条、各由哪条测试杀 |
| W3 | 甲-二：只供测试的「补写闸恒不过」开关放在哪、是不是分支纪律要的那种开关；甲-二 比甲多罩什么；与 G4-丙 同时采时还剩什么价值 |
| W4 | 条文：D23 已定项 14 拒因单子与判官判据要不要带补写例外；D13 已定项 6 第 187 行那一句采 G4-丙 后怎么写；不采时空档的账记在哪（`.claude/kb/checks-owed.md`，新 C 号从 C873 起） |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | W1–W4 | 复核第一轮判决：判决里「两腿一致」的 V1、V2 够不够得着 G4-丙 要的全称（mkfs 之后的每一类历史）；替被判出局或没被采的那几个候选（照旧、收窄、判据带闸、乙）辩护，看「只有 G4-丙 修得到」是不是同样打中它自己 |
| 云端攻方（Opus） | W1–W4 | 攻第一轮的修补本身与产生结论的方法：找让 G4-丙 误报的正确历史、让甲-二 的开关变成第二条发布路径的接法、两样合起来漏判或重判的盘面；不许重复第一轮攻过的角度（V1 的 mkfs 末段、V4 的 C670 与 Q3 拒因位置） |

引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
