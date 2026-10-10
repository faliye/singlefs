# m3-batch10-code-r2 正推腿报告：K1、K3、K5、K6（2026-10-05）

立场：从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行。代码行号一律引冻结副本 `/tmp/claude-1000/m3-batch10-code-r2/tree/`（下称「冻结副本」，引文脚本用 `--root` 指它）；kb 行号是 kb 文件自己的行号。
方案按 `crates/` 今天的实现来谈。线程上限：开工时是 10，中途收到改成 7；我起的编译与测试都是 `capped.sh 7`。开跑前 `ps -o pid,args -u "$(id -u)"` 过滤 `qemu-system|vm-bench|e152-file-system-benchmark|fio` 的结果是 0 行，只有别的 `cargo`、checker 档测试在跑，我都加了 `nice -n 19`。

## 零、做了什么、没碰什么

- 冻结副本 312 份 crates 文件对 `research/prompts/m3-batch10-code-r2-snapshot/crates-sha256.txt` 跑 `sha256sum -c`，`awk '{print $NF}' | sort | uniq -c` 的结果：`312 OK`。
- 我把冻结副本的 `Cargo.toml`、`Cargo.lock`、`.cargo`、`crates` 拷到 `/tmp/claude-1000/m3-batch10-code-r2-sonnet/work/`，在那里编、跑 harness 档与 `singlefs-checker --lib`；每次变异跑完还原，还原后对冻结副本 `diff -q`：`mount.rs`、`mount_transparency.rs`、`walk.rs` 三份都是 `same`。变异行取自冻结副本 `crates/mutations.tsv`；脚本与日志入 `research/prompts/m3-batch10-code-r2-sonnet-model/`（README 写了每个文件是什么）。这些数是冻结副本拷贝上的数，不是入库装置上的数。
- 经内存包装的跑法：同会话已有别的包装在跑，第一次起撞退出码 252（「同一个会话同时只许一条」「内存不够排不上」）；照它的提示设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`、`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=200`，上限取 2G / 3G（带变异重编时另给 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`）后都跑成了；没有绕开包装。
- 我自己犯的一个错：第一版 `mut.py` 还原后没 touch，cargo 把还原回去的旧 mtime 当没变，第一趟 2773 的日志（`mut-2773.log`）混进了 2772 的变异；改成 `mut2.py`（加 `os.utime`）后重跑（`mut-2773b.log`），第一趟日志不引、不入库。
- 没碰：攻方腿这一轮的提示与产出；主工作区；checker 档（`singlefs-checker-tier` 的测试、门禁 `checker-tier-*`）。

## K1　Q3 拒因与它的复算（A）

### K1-1　新拒因只在一处交回；别的拒因没被它顶掉，也没顶掉它

`grep -n 'NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread' crates/singlefs-core/src/mount.rs` 的原样输出（行尾截到 110 列）：

```
104:    NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
540:/// （`MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`，不交回这个值）
1840:/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q
1974:                        MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,
2974:        | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => false,
```

104 是成员定义，540、1840 是文档，2974 是 `floor_raise_refused_for_space` 的穷举臂；构造它的只有 1974 一处，在「第一遍没取到窗口锚」那个 `None` 臂里、重读之后的 `else` 里。第一遍的锚是
「.filter(|_| a_valid_root_was_read_on_every_device(&first_readings));」（`crates/singlefs-core/src/mount.rs:1959`），重读那一遍的锚是「newest_valid_root_txg_of_the_readings(&readings_of_the_reread)」（`crates/singlefs-core/src/mount.rs:1971`），它取不到才走到 1974 的 `return Err`。

读阶段在前：调用写成「let rebuilt = rebuilt_allocator(」（`crates/singlefs-core/src/mount.rs:5799`），所以进 `rebuilt_allocator` 时读阶段已经择到一条根；读阶段择不到根的拒因仍是 `Recovery(NoValidRoot)`，源头是「choose_root(cache, system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;」（`crates/singlefs-core/src/mount.rs:5586`）。`rebuilt_allocator` 里实例表读不出报的是另一个成员（「None => Err(MountError::NewerStateStillUnreadableAfterOneReread(」`crates/singlefs-core/src/mount.rs:1441`），在根环那一遍之前。
`grep -rn 'RecoveryFailure::NoValidRoot' crates/singlefs-core/src` 去掉文档行之后，构造点只有 `mount.rs:2463`、`mount.rs:2506`、`mount.rs:5586`、`mounted_read.rs:584`（只读挂载）、`recovery.rs:5179`（只读恢复的报告）。重建分配器那条路（`allocator_of_version_without_file` → `recovery.rs:2094` 的那个函数）里一个都没有。
所以：读阶段择不到根被报成 Q3 的路走不通；该报 Q3 的被报成 `Recovery(NoValidRoot)`，只剩 `mount.rs:2463`、`mount.rs:2506` 两处：它们在准入抬 F 的上限里（`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`），在 Q3 那一步之后，要「txg ≥ F、没被抛弃的有效根一条都没有」。无故障时写行那次发布的新根 txg 高于环里一切，我没造出它为真的盘面（推的）；带故障层、这第三遍读才全坏时它会报 `Recovery(NoValidRoot)`，不是 Q3、也不在单子上。这是第一轮 H3 同名问题在准入那一步的残留，不在这一轮被判范围，记一条线索，没造盘面。

推翻 K1-1 的现象：在 1974 之外找到另一处构造这个成员的地方；或一次读阶段已择到根的可写挂载在重建分配器之前报 `Recovery(NoValidRoot)`。

### K1-2　复算的判据与实现判拒的读数是同一个定义

- 被抛弃的谓词：实现是「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:1540`），`abandoned_by_table` 直接调它；复算另写一份，「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-harness/src/mount_transparency.rs:2559`）。两处逐字相同。
- 按哪条根的表：实现取最新那条根（`choose_root` 现择）的实例表，读一次不行重读一次、仍不行拒（「Some(table) => Ok((table, InstanceTableOfTheNewestRootRead::OnTheOneReread)),」`crates/singlefs-core/src/mount.rs:1440`）；复算「let Some(newest) = choose_root(crash_image, &system_configuration) else {」（`crates/singlefs-harness/src/mount_transparency.rs:2541`）再取它的表。同一条根、同一张表。
- 自证有效：实现的读法把全零单分一类，其余走「match RootRecord::parse_slot(&bytes, filesystem_identifier) {」（`crates/singlefs-core/src/mount.rs:1760`）；复算的 `readable_roots` 走「RootRingSlotReading::SelfVerified(root) => visit(ring_slot, root),」（`crates/singlefs-core/src/recovery.rs:1204`）。全零槽两边都不是有效根（`parse_slot` 过不了全零），读错两边都不是。
- 「抛弃之后还有」：实现的窗口锚是「(!abandoned_by_table(root, &newest_table)).then_some(root.checkpoint_txg)」（`crates/singlefs-core/src/mount.rs:1915`）的最大值，取不到即「一条不被抛弃的自证有效根都没有」；复算 `!readable_roots(…).iter().any(|root| 不被抛弃)`。两边是同一个量。
- 读数的取法：第一遍与重读都是「每槽读一次」（`read_every_root_ring_slot_once`，起于 `mount.rs:1956`）；镜像上复算读的是盘上字节、没有故障层。

### K1-3　没注读故障的合法崩溃镜像上复算为真吗

复算为真有两条路：择不到系统配置或择不到根（`mount_transparency.rs:2541` 的 `else` 里 `return true`），或最新那条根按它自己指着的表被抛弃。
- 第一条：择不到根的盘面，实现在读阶段就报 `Recovery(NoValidRoot)`（`mount.rs:5586`），到不了 Q3；判官收到的是「MountError::Recovery(_)」（`crates/singlefs-harness/src/mount_transparency.rs:2258`），那一行把它排在单子外。
- 第二条：判官文档自己写了「没有读故障时最新那条根按它自己指着的表恒不被抛弃（表里的行只记更早的实例，推的）」（`crates/singlefs-harness/src/mount_transparency.rs:2534`）。我对着 D23 核它：行落在被恢复抛弃的那些实例上，「行只落在 [这个根的实例, 新实例)」（`.claude/kb/decisions/23-journal的角色与格式.md:400`，原句在实例切换一条里）；新实例的第一次发布 txg 压过环里一切：「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg」（`.claude/kb/decisions/23-journal的角色与格式.md:371`）。两句合起来：最新根属于最大的实例，它的表里没有它自己实例的行。这是推的，我没造反例盘面。

结论：没注读故障的合法崩溃镜像上 Q3 复算为真，只会在「实现把读阶段择不到根的盘面报成 Q3」这种实现错上出现，不会假红。推翻它的现象：一张没注读故障、读阶段择得到根、而最新根按自己的表被抛弃的合法盘面（造得出就是假红，也说明 D23 注 2、3 之外还有一条路）。

### K1-4　实例表读不出时「不按表滤」那一读法会不会让某一格判反

复算里表读不出的那一支是「let Some(table_of_the_newest_root) = instance_table_of_root(crash_image, &newest) else {」（`crates/singlefs-harness/src/mount_transparency.rs:2544`），下一行 `return false;`：择得到根就判据为假。它与 `effective_rollback_floor` 读不出时的取法同向：「那张表读不出、解不开时不按表滤，每块盘取读得出的根里最新的那条。」（`crates/singlefs-core/src/recovery.rs:1548`）。

这一支走得到吗：
- 被拒的那次挂载不带故障层（崩溃注入第二、三截、E171 那种）：实现里表读不出先报 `NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)`（`mount.rs:1441`），判官把它映射成单子上另一行（`InstanceTableOfTheNewestRootUnreadable`，`mount_transparency.rs:2243` 附近），根环那一遍根本没读，到不了 Q3。
- 被拒的那次挂载经故障层：实现读表读成功了（读不成功就报上一条），镜像上读同一份字节也读得成；这一支够不着。
所以在我能造出的盘面上它没有判反的格。它判 `false` 是严的那个方向：表读不出还报 Q3 的挂载没有故障层豁免时判不合法（`CriterionFalseWhenRecomputedOnTheCrashImage`）。推翻它的现象：实现里读表成功、镜像上读同一份字节读不成（两处读的不是同一份字节），那就是读法分叉。

### K1-5　Q3 与读故障：三种组合各走哪一臂

判官里 Q3 的分臂：没注读故障 ⇒ 复算为真才推新条件（「RefusalReasonOnTheD23List::NoValidRootInTheRootRingOfTheRebuiltAllocator,」`crates/singlefs-harness/src/mount_transparency.rs:2753`）；注入过读故障 ⇒ 两条都不判（同一文件 2767 的合并臂）；被拒的那次挂载经故障层 ⇒ 去掉「判据复算为假」：「.can_hide_a_transient_read_error_from_the_crash_image()」（`crates/singlefs-harness/src/mount_transparency.rs:2783`）。崩溃注入第二、三截与 E171 都传不带故障层：「const READ_FAULTS_UNDER_A_MOUNT_ON_A_CRASH_IMAGE: ReadFaultsUnderTheRefusedMount =」（`crates/singlefs-checker-tier/src/crash_injection.rs:2584`）。
变异行证它们各有一条会红的用例，我逐行跑了（K5-3 的表）：2768 只红手工盘面那条；2769 只红注入过读故障那条；2770 只红「读得出的盘面判据为假」那条；2771 只红「经故障层」那条；2767 五条全红。

### K1-6　实现员读法 ②（第二遍比第一遍坏 ⇒ Q3）：代码这样走，没有用例钉它

代码：第一遍有锚但不是每块盘都读出（有一块盘没读出）⇒ 进 `None` 臂、重读，重读那一遍整份换掉第一遍：「readings_of_the_reread,」（`crates/singlefs-core/src/mount.rs:1978`）；重读那一遍一条都没有就 1974 拒，不回头用第一遍。这与 D23 里同族那一处的口径一致：「重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载、盘上逐字节不变，只读挂载照常」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）——重读读数替换第一遍，重读仍坏就拒。
钉不钉：我在拷贝上写了对立读法的变异（重读一条都没有时退回第一遍的读数、第一遍有锚就用第一遍往下走，改在 `mount.rs` 的 1970–1976 那一段，源码见模型目录 README 与 `logs/mut-fallback.log`），跑 `rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read`、`rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root`、`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable` 三份，原样输出：

```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.08s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.19s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.57s
```

22 条全绿，所以读法 ② 既没被用例钉住也没被它们反驳（与实现员报告「没加用例钉它」一致）。判法上我的读数：读法 ② 与 D23 那一处同族、方向偏保守（拒），读法的另一支（退回第一遍）会让一次「读到一条根又读不到」的瞬时读错之后仍走 m 取不到那一支；两支在带故障层的挂载上才分得开，崩溃注入没有故障层，分不开。交用户时附这个读数。推翻它的现象：D16 或 D23 里有一句写「重读比第一遍坏时退回第一遍」。

### K1-7　崩溃注入第三截的计数与报红，漏的一形

第三截的判官取舍函数把判官的结局分三类：不在单子上或实例表读不出的交回 `None`（「WritableMountRefusalVerdict::ReasonNotOnTheD23List」`crates/singlefs-checker-tier/src/crash_injection.rs:3728`，那一臂连同 `Legal`、`Illegal` 的两个实例表臂一起 `return None`）；Q3 合法或不合法都先计数（`crash_injection.rs:3771`、`crash_injection.rs:3778`）；只有判官列出「判据为真」那一条才报红（「criterion_true_without_read_faults.then_some((reason, conditions_not_met))」`crates/singlefs-checker-tier/src/crash_injection.rs:3806`）。文档写明这是设计：「（r5 判决第二节 U6）条款只管第二截，这一截不拿它们定红绿：只差那几条的不报红、不计不合法」（`crates/singlefs-checker-tier/src/crash_injection.rs:3636`）。

漏的一形：二次崩溃镜像读得出（判据为假）、实现却报了 Q3（误拒）：第三截记一次 Q3 的计数、不报红；同一个误拒在第二截（第一次崩溃的镜像）判官给 `Illegal`，那一截是报红的（`crates/singlefs-checker-tier/src/crash_injection.rs:2646` 起的 `Illegal` 臂）。第三截别的拒因（含 `Recovery(NoValidRoot)`）也是这样一律不报。这与 C670、G4 同口径，是既有设计，不是 Q3 新漏的；差别在 Q3 的「判据为真」在无故障层的挂载里几乎不可达（K1-3：读阶段先拒），所以第三截那个新计数的实际作用只剩「抓把读阶段择不到根误报成 Q3」，而 Q3 最现实的实现错——误拒——在第三截看不见。归 K4（攻方腿）的口径问题，不算 K1 打中。
另一处小的：第三截报红那句 detail 与模型那一句没改，「没注入读故障的崩溃态上，见证判据与」（`crates/singlefs-checker-tier/src/crash_injection.rs:3687`）这句话只写见证与 G4 两个判据，Q3 报红时读到的是它；实现员报告说过这是有意不动（改了会动已知红的签名）。我没核签名。

推翻 K1-7：第三截里有一处把 Q3 判据为假的误拒报红（我读的 3644–3715 与它的调用点 3544 都没有）。

### K1 判定

一致，没打中：拒因只在 1974 一处交回；判据与实现读数同一个定义；表读不出那一读法在能造出的盘面上没有判反的格；第三截的「漏一形」是既有口径、归 K4。读法 ② 附读数（K1-6）。

## K3　两处覆盖用例（C）

### K3-1　条款要的件事，和两处实现、两条用例各钉在哪

- 条款（D16 已定项 1「环里最旧有效根」那一行）：「checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 环里最旧有效根)）」（`.claude/kb/decisions/16-发布语义.md:34`，同一行里接着是「影子账的候选过滤」与「六处都不再按一圈窗口滤根」）。影子账「有效根」的口径在 D23：「指回退候选集里的根（按实例表判仍然有效 ∧ txg ≥ F_生效）」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）。
- 影子账实现：候选过滤是「.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)」（`crates/singlefs-core/src/mount.rs:1495`），没有窗口。
- 丁-defer 实现：门槛由冻结副本 `walk.rs` 第 9499 行的 `let deferred_threshold = deferred_slots_reclaim_threshold(` 算（转述：主工作区的 `walk.rs` 已被别的线改动、行号后移 48 行，所以这几处不写成引文），函数体最后取「环里最旧有效根」与 F_生效 的较大者（冻结副本 `walk.rs` 第 5868 行，转述）；喂它的是全部读得出的根，窗口只留给 I-7.9（「候选过滤与丁-defer 门槛不再看它（RGall）。」`crates/singlefs-checker/src/walk.rs:9074`）。并进遍历的槽要「没被候选那一遍走到」且「已释放、释放代高于门槛」（冻结副本 `walk.rs` 第 5890 行，转述）。

### K3-2　用例钉住的是不是这件事；改回窗口红是因为门槛还是因为别的

我在拷贝上先跑基线，再逐行跑变异行 2772（影子账候选改回窗口）与 2773（丁-defer 门槛改回窗口），都经 `run-with-memory-cap.sh`（2G、`capped.sh 7`）。原样输出（`logs/base-k3b.log`、`logs/mut-2772.log`、`logs/mut-2773b.log`）：

```
基线  ：test result: ok. 2 passed; 0 failed
2772  ：a_remount_isolates_no_slot_…_outside_the_one_revolution_window ... FAILED；raising_the_floor_over_a_root_left_in_a_hole_…_on_the_whole_ring ... ok
        left: {DeviceIdentity(0): 14, DeviceIdentity(1): 14}
       right: {DeviceIdentity(0): 12, DeviceIdentity(1): 12}
2773  ：raising_the_floor_over_a_root_left_in_a_hole_…_on_the_whole_ring ... FAILED；a_remount_isolates_no_slot_…_outside_the_one_revolution_window ... ok
        I-3.1：盘 0：记账的已分配 Some(2392064)，遍历全部有效根得到 2260992（其中隔离豁免 0、F 之下仍在 defer 的 0）
```

读法：
- 两行各只红自己的用例，另一条绿，互不替对方担着。2772 只改候选过滤，所以红的原因只能是候选集里窗口外的有效根没了；2773 只改门槛的输入集，红的原因只能是门槛，日志里「F 之下仍在 defer 的 0」就是门槛抬高之后 B 引用的槽被丢出 defer 并集。
- 影子账那条的期望不是写死的 12：它由用例自己从盘上读出来算（候选 ＝ 有效且 txg ≥ F，被抛弃根引用的槽里候选根与现行账都不引用的跨度和：`let referenced_by_candidates = referenced_by(&candidates);` `crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs:454`，断言是「isolated_per_device, expected_isolated_per_device,」同文件 500）。前提断言（486 行起）要求「窗口外有效根与被抛弃根同引用、窗口里的有效根与现行账都不引用」的槽集非空，所以不是空转。14 只出现在变异日志里，用例里没有这个数。
- 丁-defer 那条的前提断言（`crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs:203` 起）要求 F 之下、窗口外的有效根恰是 B（txg 7），窗口里最旧的有效根高于 F；末项是「allocated_statistic_violations(&image_after_the_raise),」（同文件 209）为空。2392064 − 2260992 = 131072，恰好 4 个 32768 字节（`DATA_UNIT_BYTES`）：记账多出的就是 B 引用、释放代落在 (F, 窗口里最旧有效根] 的那几槽（算术，槽数没单独核）。

结论：钉住的是「六处不按窗口滤」里这两处，改回窗口时红在窗口那一项上，不是别的。

### K3-3　它没钉的：门槛里 max(F_生效, 环里最旧有效根) 的 max 项

我在拷贝上另写一条变异：门槛只取 F_生效、丢掉环里最旧有效根那一项（改 `walk.rs:5868` 那个 `map_or` 的闭包）。新用例两条都绿（`logs/mut-fonly.log`：`test result: ok. 2 passed`），因为这个盘面上环里最旧有效根 7 低于 F 8，max 还是 F。别的用例抓它：`cargo test -p singlefs-checker --lib` 在这个变异下 `13 passed; 1 failed`，红的是 `walk::tests::deferred_selection_merging_the_abandoned_roots_too_selects_the_same_slots_on_every_small_world_but_the_double_fault_form`（`logs/mut-fonly-lib.log`），基线同命令 `14 passed; 0 failed`（`logs/base-lib.log`）。所以 max 项有别的用例钉着，新用例不替它钉；这不是缺口，只是新用例的射程。

### K3-4　K1 那一形为什么钉不住门槛；换的那一形有没有被别的路径掩护

- K1 那一形不钉门槛的理由，源码里有据：被记录施加出来、根槽没落盘的版本，在 `versions_applied_only_by_records` 里 ④ 用的是环里最旧有效根（「let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;」`crates/singlefs-checker/src/walk.rs:7283`），这一版因此进候选走一遍，它引用的槽在 `walked_slots` 里，丁-defer 并集会排除它们（`walk.rs:5890` 的 `!walked_slots.contains(key)`），门槛取什么都一样。
- 复核实现员的说法：我在拷贝上照影子账那条的前半段造 K1 那一形（实例 1 到 24 被抛弃、连崩 11 次、做完一次可写挂载），F 抬到 24 时被拒：`Err("RollbackFloorAboveCeiling { requested: CheckpointTxg(24), ceiling: CheckpointTxg(20) }")`（`logs/probe-base2.log`）；抬到上限 20：基线与 2773 变异下 I-3.1 都是 `[]`（`logs/probe20-base.log`、`logs/probe20-mut.log`），即变异在 K1 形上没红，定性与实现员一致。实现员报告里那组具体数（实例表槽 1088、释放代 36、F = 24、「PROBE I-3.1 Holds」）只在它的临时探针里量过，我的造法 F = 24 取不到，**这一组数我复核不了**；它们没进用例（用例文件头只写了理由，没写这些数，`shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs:14`）。
- 换的那一形被别的路径掩护吗：2773 在它上面红了，红的是 I-3.1 记账多的一边、F 之下仍在 defer 的 0，说明 B 的槽没被候选那一遍走到、也没被记录施加那一路带进遍历，门槛是唯一的闸。
- 窗口锚的取法：用例用「最新有效根」（盘上读出的有效根里 txg 最大的），2772 的变异锚取这一遍读出的最新不被抛弃的根，同一个量。

推翻 K3：存在一个改回窗口的写法，两条用例都不红（我用的是变异表里那两行，不是所有写法）；或 2773 的红法在用例里换成 F 之下 defer 以外的原因。

### K3 判定

一致，没打中。用例钉的是条款这件事；改回窗口红在门槛（丁-defer）与候选过滤（影子账）；K1 那一形的数我复核不了，定性复现。

## K5　数能不能核

### K5-1　「隔离 14 槽、期望 12」

- 从哪一段推出：期望 12 是影子账用例自己从盘上算的（K3-2：`expected_isolated_per_device`，不是常量）；14 是候选过滤改回窗口之后实现隔离的槽数（候选里少了窗口外的有效根，它们与被抛弃根同引用的槽不再被豁免）。代码上改的就是「.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)」那一处（`crates/singlefs-core/src/mount.rs:1495`）。
- 钉它的断言：「isolated_per_device, expected_isolated_per_device,」（`crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs:500`）。
- 改回旧值红不红：红。变异行 2772 在拷贝上跑出 `left: {DeviceIdentity(0): 14, DeviceIdentity(1): 14}` 对 `right: {DeviceIdentity(0): 12, DeviceIdentity(1): 12}`，与实现员报告的「每块盘隔离 14、期望 12」逐数相同。
- 只在探针上核过吗：没有；12 每次跑都算出来，14 在我复跑的变异日志里看到。

### K5-2　rr2 臂 8 段与拿掉 rr2 时 8 条红在 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`

- 8 的来历：2 个起点（192 槽、216 槽）乘 4 份读坏计划，`grep -c '^#\[test\]'` 那份用例文件：`8`。
- 从哪一段推出：第一遍每块盘都读出有效根才算有锚（「.filter(|_| a_valid_root_was_read_on_every_device(&first_readings));」`crates/singlefs-core/src/mount.rs:1959`），否则整环重读；拿掉这个 `.filter`，第一遍只要有一块盘读出就不重读，往下走「None => oldest_valid_root_in_the_ring,」（`crates/singlefs-core/src/mount.rs:2078`，m 取不到、当场只放到环里最旧有效根），窄盘上写行取不到落点，取号之前拒。
- 钉它的断言：每条用例整体比 `RemountUnderThePlan`；期望值是「root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),」（`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:655`）加钩子恰调一次、盘面与不注错的再挂相同、池级 checker 无违例、丢盘整读不变。
- 改回旧值红不红：变异行 2764（192 起点那一条点名）我在拷贝上跑：基线 `test result: ok. 8 passed; 0 failed`（`logs/base-rr2.log`）；变异 `test result: FAILED. 0 passed; 8 failed`（`logs/mut-2764.log`）。日志里拒的成员数：`grep -o 'PlacementRefusedBeforeAcquisitionMountAdmissionUndecided' | wc -l` → 8；`grep -o 'NoFreeSlotOnAnyDevice' | wc -l` → 8；`unit: InstanceTable` 4 次（192 起点）、`unit: AllocationTreeNodeBelowTheRoot` 4 次（216 起点）。与实现员报告和第一轮判决的描述逐项相同。2765 与 2764 同一处同一替换、点名的是 216 起点那一条；它在 2764 的 8 条红里（日志里那条名字是 `FAILED`）。
- 「照常挂上」的钉：不只是 `Ok`——钩子那一刻每个槽恰读过 6 次也是断言（「.all(|reads| *reads == READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD)」`…one_device_reads_no_valid_root.rs:639`）。我把那个常量改成 5 跑（拷贝，`logs/mut-const5.log`）：`test result: FAILED. 0 passed; 8 failed`，所以 6 这个数是被钉住的，不是探针数。
- 第一轮判决里「36 → 28」「挂上的 8 段 0 红 0 坏」是攻方在它的装置上量的数（判决第 41 行），我没复跑它的装置，**复核不了**；这里 8 段是实现里的 8 条用例，数对得上（8 = 8）。

### K5-3　手工盘面「根环每一槽清 0」那条用例

- 从哪一段推出：盘面是「for ring_slot in every_root_ring_slot(&immutable.sizes) {」（`crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs:39`）把根环每个槽写成全 0；复算里 `choose_root` 为 `None` 走「else {」下的 `return true`（`mount_transparency.rs:2541`）。前提断言「"前提：根环全 0，只读恢复也择不到根"」（同用例文件 85）。
- 钉它的断言：整个判决比 `Illegal { reason: …, conditions_not_met: [NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults] }`（同用例文件 99）。
- 改回旧值红不红：变异行 2768（判官不复算那一条）：这一条 `FAILED`，另四条 `ok`（`logs/row-2768.log`）。2767、2769、2770、2771 各红自己那一条，见 K1-5。
- 这条用例是手工盘面，不是崩溃摆得出来的（用例注释写明「静默损坏，不是崩溃摆得出来的」）：它只罩复算里「择不到根」那一支。复算里「按表判被抛弃」那半句（`.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)`，`mount_transparency.rs:2559`）harness 档没有用例钉，也没有变异行：我把它改成「表里有这个实例的行就算被抛弃」（拷贝，`logs/mut-abandoned-predicate.log`），Q3 那份文件 `5 passed; 0 failed`。理由与 K1-3 同：这一支在合法盘面上不可达，没有盘面可钉。不算打中（没有数是错的），记成「射程」。

### K5-4　有没有一个数只在临时探针上核过

- 14 / 12、2392064 / 2260992、8 条红与拒的成员、6 次读：都在我复跑里出现，且有断言或变异行钉着。
- 只在实现员探针里有的：K1 那一形的「实例表槽 1088、释放代 36、F = 24」（K3-4）：我用同一个盘面前半段造不出 F = 24（上限 20），复核不了；它没进用例也没进条款，只是实现员解释「为什么换一形」的旁证，不压任何断言。
- 第一轮 K6 那几个数（3145728、1441792、GPU 内核展开大小）不在这一轮的改动里，没碰。

### K5 判定

一致：每个数都能从代码推出来，钉它的测试与变异行改回旧值都红（拷贝上复跑）；只在探针上的那一组数不压任何断言。

## K6　欠账与读法清单全不全

范围：实现员报告里「停下交主 agent 的」四条与它的「要主 agent 跑的命令」三条（在 `/tmp/claude-1000/m3-r1/batch10g/report.md`，主 agent 给的材料；这一节按条引它，不当依据写进 kb），对着调度记录 `records/2026-09-30-里程碑三开工第一轮.md`、`.claude/kb/checks-owed.md`、判决与这一轮正文找去向；再看第一轮判决第三节五条关没关。kb 此刻在被写回线动（`git status --short .claude/kb` 里 `checks-owed.md`、`decisions-history.md` 是 `MM`），下面的 grep 是 2026-10-05 此刻的计数，主 agent 判决前请重跑。

### K6-1　停下交主 agent 的四条

| 条 | 内容 | 去向（现查） | 判 |
|---|---|---|---|
| 1 | rr2 让新池与年轻的环每次可写挂载都整环重读 | 调度记录：「三处并进 10g 代码轮的格，量过之后再交用户」（`records/2026-09-30-里程碑三开工第一轮.md:656`）；这一轮正文第二节 ①、格 K2 | 有去向；kb 写回待做（见第 4 条） |
| 2 | 第二遍比第一遍坏 ⇒ Q3 | 同上 ②；这一轮 K1-6（我给了读数） | 有去向 |
| 3 | Q3 判据在实例表读不出时「不按表滤」 | 同上 ③；这一轮 K1-4（我给了读数） | 有去向 |
| 4 | kb 写回不归实现员：D23 已定项 14 单子加 Q3 一行；D16 已定项 1「根槽这一次读坏」补 rr2；第 1、2 条两个后果 | 调度记录：「第 10 批 kb 写回只等第 10g 批合入」（`records/2026-09-30-里程碑三开工第一轮.md:655`）；判决第六节 Q3 行「条款随第 10 批 kb 写回」 | 有去向（泛指）；kb 里还没有：`grep -rn 'rr2' .claude/kb --include=*.md | wc -l` → 0，`grep -rn 'NoValidRootInTheRootRing' .claude/kb --include=*.md | wc -l` → 0，`grep -rn '重建分配器读根环' .claude/kb --include=*.md | wc -l` → 0 |

### K6-2　要主 agent 跑的三条命令

| 条 | 命令 | 去向 | 判 |
|---|---|---|---|
| 1 | 第 4 条那一行新名的复跑（门禁 `checker-tier-crates-mutation-replay` 的 `--item`） | 调度记录：「checker 档 1 行留主 agent」（`records/2026-09-30-里程碑三开工第一轮.md:656`） | 有去向；我复核不了（checker 档重型，不跑） |
| 2 | checker 档快档（第三截改了、rr2 改了读次数） | 同一份记录里 10g 合入验证那一条起 checker 档库与全部集成测试（用户要求档）| 有去向；结果还没有，归 K4 补进判决 |
| 3 | 实验复跑里 E158 / E171 装置数重读的那几列 | 记录里提到 E171 装置那一臂（`refusal_condition_text`，修订 7 带上）；`grep -c 'E158' records/2026-09-30-里程碑三开工第一轮.md` → 0，**E158 没有去向** | **缺一格**：E158 装置的重读计数列 rr2 之后可能变，调度记录没有记；它在门禁 experiment-replay 格里，提交时才跑到 |

### K6-3　第一轮判决第三节五条关没关

| 条 | 判决要的 | 现状 | 判 |
|---|---|---|---|
| 1 | 影子账候选、丁-defer 门槛改回窗口补会红的用例与变异行 | 新用例两条、变异行 2772、2773；我复跑都红（K3） | 关 |
| 2 | 第 927 行与留档用例从「等价」改写成第三类 | 用例文档改成「它是第三类「取样点不敏感」」（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1616`），并写明「补不上敏感的取样点，是因为要给崩溃注入」换恢复策略的开关（同文件 1617）；变异行换名为 2774，必须红的是同一条用例。它是 checker 档，我没跑，**复核不了** | 关（待主 agent 跑 checker 档那一行） |
| 3 | `mount.rs:2020` 过时注释 | `grep -n '一圈窗口里最旧\|窗口里最旧有效根' crates/singlefs-core/src/mount.rs` 零命中；注释现在在 2061–2066 行，写 RGall 与 rr2 之后走到这一支的形 | 关 |
| 4 | Q3 另起拒因、单子加一行 | 代码、判官、第三截、用例都有（K1）；kb 单子那一行待写回 | 代码关，kb 待写回 |
| 5 | Q4 rr2 | 代码与 8 条用例有（K5-2）；kb 待写回；「补层 0」那一半：这一轮没看到层 0 的改动（补丁文件表里没有层 0 文件） | 代码关；层 0 与 kb 未见 |

第 5 条「补层 0」：第一轮判决第六节 Q4 行写「下一批实现，补层 0 与用例」；实现员报告的文件表（13 份）里没有层 0（`research/` 或 checker 档层 0 装置）文件，报告第「受影响的层 0 流与崩溃枚举用例」一节写的是「推的，没跑」。这一格去向是主 agent 的 checker 档整轮（K6-2 第 2 条），不是新代码；我只记「没有补层 0 的改动」。

### K6-4　第一轮判决里别的「记欠账」与那张不存在的收拢表

第一轮判决 K7 一节写「去向见第三、五节与调度记录第 10 批收拢表」，调度记录里没有第 10 批收拢表：`grep -n '收拢' records/2026-09-30-里程碑三开工第一轮.md | grep -c '第 10 批'` → 0（只有第 4、5、6 批收拢）。判决第五节「记欠账」那几行在 `checks-owed.md` 此刻都没有：`盲带`、`偏移不重叠`、`换恢复策略`、`抽掉电`、`DeletedRowsStillPresent`、`3145728` 各 0 命中，`影子账判抛弃`、`读出被抛弃的根` 各 0 命中。它们的去向是「第 10 批 kb 写回」，这一轮正文第二节把它们列成「已知没做」，判决照原去向写——此刻没有缺失，但「去向」目前只有这一轮正文与第一轮判决两处文字，不在欠账表里；写回线动 `checks-owed.md` 时要带上。

### K6 判定

读法清单（三处）、kb 写回（泛指）、命令三条里的两条有去向；缺一格：E158 装置的重读计数列没有记（K6-2 第 3 条）；第一轮判决第三节五条里代码全关、层 0 与 kb 写回未见。第一轮判决 K7 提到的「第 10 批收拢表」不存在。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 拒因只在一处交回 | 一致 | 构造点只有 `mount.rs:1974`；读阶段择不到根仍是 `Recovery(NoValidRoot)`；准入那一步另有同名拒因，线索未造盘面 |
| K1-2 判据与读数同一定义 | 一致 | 谓词逐字相同，同一条根同一张表，全零与读错两边同处置 |
| K1-3 无读故障合法镜像复算为真 | 一致（推的） | 只会在「读阶段择不到根被误报成 Q3」上为真，不假红；没造反例 |
| K1-4 表读不出「不按表滤」 | 一致 | 两种挂载下这一支都走不到；判 false 是严的方向 |
| K1-5 读故障三种组合 | 一致 | 五条变异各红各自那一条 |
| K1-6 读法 ②（第二遍更坏 ⇒ Q3） | 与实现员写的读法相同，附读数 | 22 条用例钉不住它；与 D23 392 行同族口径；方向偏拒 |
| K1-7 第三截计数与报红 | 没打中，既有口径 | 只在判据为真时报红；误拒一形（判据为假）第三截不红，与 C670、G4 同；归 K4 |
| K3 两处覆盖用例 | 一致 | 两行变异各红自己那条；红在门槛与候选过滤；K1 形定性复现、具体数复核不了 |
| K5 数能不能核 | 一致 | 14/12、8 条红、6 次读、手工盘面都被钉；只在探针上的一组数不压断言 |
| K6 欠账与读法清单 | 缺一格 | E158 装置重读计数列无去向；kb 写回与「记欠账」尚未落；「第 10 批收拢表」不存在 |

## 没做什么

- 没替主 agent 采纳或出判决；「打中 / 站住」只写我这一腿的读数。
- 没跑 checker 档（`singlefs-checker-tier` 的测试、`checker-tier-crates-mutation-replay`、E158 / E171 装置）：行 2774 与第三截的实跑结果、层 0 与读序号后挪后原先打中的槽换没换（格 K4）都复核不了。
- 没复跑攻方腿的装置（「36 → 28」「8 段 0 红 0 坏」）与实现员的临时探针（K1 形的槽 1088、释放代 36、F = 24）。
- K3-2 里「日志里 F 之下仍在 defer 的 0 就是门槛抬高之后 B 引用的槽被丢出 defer 并集」一句是我从代码推的，没单独打印并集。
- K1-3 的「最新根属于最大的实例」是读 D23 注 2、3 推的，没造反例盘面；K1-1 里准入那一步同名拒因的线索同样没造盘面。
- 没读禁读清单里的文件；kb 是 `git status` 里 `MM` 的状态，我读的是此刻的工作区文件。
- 交回前删了自己建的仓副本 `/tmp/claude-1000/m3-batch10-code-r2-sonnet/work/` 与编译目录 `/tmp/claude-1000/m3-batch10-code-r2-sonnet/target/`；草稿目录里只剩日志与脚本，脚本与日志已拷进 `research/prompts/m3-batch10-code-r2-sonnet-model/`（`logs/` 下是原样日志，不是 `/tmp` 路径当依据）。
