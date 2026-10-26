# m3-close-g4-rewrite-r1：取号补写例外落地之后，判官改法 G4 那条判据还站不站得住（推论轮第一轮）正文（2026-10-21）

<!-- doc-lint:not-numbers G4 Q3 U4 U6 S4 R2 -->

## 一、题面

取号那一步的补写例外（下称丙'，D22（单元原子性怎么合成） 已定项 8 的依据里「取号那一步的补写例外」那一行；三方判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第二节 U4 与第三节；用户 2026-10-19 定「开例外，四件闸」、2026-10-20 采两样补法）已经写进 `crates/singlefs-core/src/mount.rs`。之后 checker 档库单测
`crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage` 红：它造的二次崩溃镜像（盘 0 两个系统配置槽各翻一个字节，别的都没动）过得了补写闸，再挂做成，不再以「读阶段判完之后系统配置又读不出」拒，测试的前提「再挂被拒」不成立。

这一轮要回答两件事：

1. **判据那一半**：判官 `crates/singlefs-harness/src/mount_transparency.rs` 里改法 G4 那一臂（注释原话「改法 G4：没有读故障时设备不报读错，某块盘两槽都自证不过只能是盘面本身如此——正确实现在没有读故障的历史里留不下它。」）在丙' 之后还站不站得住。具体说：崩溃态没注入读故障、再挂因为「某块盘一份本池自证过的系统配置都没有、又过不了补写闸」而拒，判官把它判成「拒得不合法（造出这份盘面的那一段不对）」，这个判法对不对；要不要改，改成什么（照旧、收窄到「补写闸也不过」、或别的），每个候选写清它在哪几种盘面上与今天判得不同。
2. **测试那一半**：那条红的测试改哪条路。
   - 甲：测试镜像另外过不了补写闸的某一件（例：盘 0 上所选那一版的某个单元改坏一个字节，盘 1 那一份不动），再挂照旧以 G4 拒，测试断言不变，只改文档注释写明「补写闸某一件不过」。
   - 乙：测试改成断言新行为（再挂做成、两槽补写、I-8.4（重放幂等） 判同），G4 第三截的判别力另找一张过不了闸的镜像来钉，`crates/mutations.tsv` 里「第29批 G4 只判第二截不判第三截」那一行改指新测试。
   - 别的候选也可以提，写清它比甲、乙多罩或少罩什么。

第 2 件的选法依赖第 1 件的答案：判据改了，测试断言的「不合法一次」也跟着变。

被测条款抄在附录里：D22（单元原子性怎么合成） 已定项 8（固定结构的放置，含容错承诺与补写例外的依据）；D23（journal 的角色与格式） 已定项 14（可写挂载的拒因单子）；D13（验证路线） 已定项 1（挂载透明只管不带故障的状态）。

## 二、实现今天的样子（主 agent 的观测，2026-10-21，行号按主工作区，HEAD 841b3beab 加未提交的改动；快照在 `research/prompts/m3-close-g4-rewrite-r1-snapshot/`）

- 补写闸的入口：`crates/singlefs-core/src/mount.rs:5758` `devices_without_the_selected_version`。某块盘两槽里一份本池自证过的系统配置都没有时（`mount.rs:5780`），先看有没有设备报读错的槽，有就 `Err(device)`；没有就过 `rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`（`mount.rs:5851`），五件都过记进补写的盘（`mount.rs:5794`）、挂载往下走、取号写两轮时补写两槽；有一件不过 `Err(device)`，调用方报「读阶段判完之后系统配置又读不出」。五件的名字在 `RewriteGateNotMet` 枚举里：所选那一版的单元不全在这块盘上、环槽设备报读错、环里有比所选那一版新的根或记录、单元区里有更晚实例写的单元、所选那一版的根不在它该住的环槽里。
- 判官：`crates/singlefs-harness/src/mount_transparency.rs` 的 `judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount`。拒因是「读阶段判完之后系统配置又读不出」、崩溃态没注入读故障时，调 `system_configuration_unreadable_after_the_read_stage_on_the_crash_image`（同文件 2419 行起）复算：择不到系统配置、见证判不出、或某块盘一份本池自证过的都没有，就推一条 `SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults`（拒得不合法）。复算里没有补写闸那五件。
- 红的测试：`crates/singlefs-checker-tier/src/crash_injection.rs` 第 8964 行起那一条（断言在第 9029 行附近）。它的镜像：历史种子 0，第一个文件之后接一次覆盖写，写表施加到覆盖写的第一个写之前，盘 0 两槽各在偏移 2048 翻一个字节。
- 调查员的探针（harness 档直接调核心层可写挂载，不经 checker 档第三截）：同一张镜像上 `remount=Mounted instance=InstanceGeneration(2)`；把补写闸恒设成不过之后同一条变成 `remount=Refused error=SystemConfigurationUnreadableAfterTheReadStage(…)`。全文在 `~/.local/share/singlefs/work/m4-investigate-two-red-groups/report.md`「第一组 ③」（仓外，腿可以读）。
- 变异：`crates/mutations.tsv` 第 2406–2408 行三条「第29批 G4」，其中 2408「只判第二截不判第三截」靠这条红的测试杀。
- mkfs 那一侧：用过的盘上重做 mkfs 的第一段把每块盘 [0, 根环基址) 清零（`crates/singlefs-core/src/make_filesystem.rs`），这一段持久之后、系统配置槽写之前崩溃，两块盘的两槽都是零。这种盘面由 mkfs 自己的崩溃枚举流 `crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs` 按 mkfs 中途崩溃的条文判，不进崩溃注入第二、三截。它算不算「没有读故障的历史留得下两槽全废」的反例，是第 1 件要答的一部分。

## 三、格

| 格 | 问什么 |
|---|---|
| V1 | 没有读故障、只有崩溃的历史（含用过的盘上重做 mkfs 中途崩溃、取号写两轮中途崩溃、补写本身中途崩溃），留不留得下「某块盘两槽都自证不过」的盘面；留得下的，逐条写出历史与崩溃点，在 `crates/` 的哪几步 |
| V2 | 照 V1 的答案，G4 那一臂今天的判法（判据复算为真即不合法）对不对；要改的，候选各写清：在哪几种盘面上与今天判得不同、要不要读补写闸那五件、要不要与核心层共用代码（判官与实现不许共用同一段算法，`.claude/rules/fs-design.md`「运行时与 checker 必须用不同的算法」） |
| V3 | 测试走甲、乙还是别的：每个候选写清改完之后 `crates/mutations.tsv` 第 2406–2408 行三条各由哪条测试杀、测试的镜像在补写闸哪一件上不过（或过），以及它测的东西与 V2 选定的判法是否一致 |
| V4 | 同形的另外两臂（C670 那一臂、Q3 那一臂）会不会因丙' 同样失效：各自的判据在什么盘面上为真、补写闸放过那块盘之后它们还会不会被调到 |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | V1–V4 | 从 `crates/` 今天的代码与附录里的条款正推：先答 V1（逐条历史），再推 V2、V3、V4 |
| 云端攻方（Opus） | V1–V4 | 找反例：V1 里能留下两槽全废盘面的无读故障历史；V2 每个候选在哪一张盘面上判错；V3 每条路改完之后哪条变异不再被杀、或测试测的东西与判法对不上 |

方案按 `crates/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
