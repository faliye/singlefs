# m3-batch33t-code-r1 正推腿报告（Sonnet，K1 与 K3，2026-10-08）

立场：从条款（D16 已定项 1、D23 已定项 14、D28 已定项 1、C692 / C694 / C695 / C746）与两份设计判决推「代码应当是什么」，再与冻结副本逐格比。引代码写冻结副本 `~/.local/share/singlefs/work/m3-batch33t-code-r1/tree/` 里的路径加行号；引 kb 写 kb 文件自己的行号。模型与复跑：`research/prompts/m3-batch33t-code-r1-sonnet-model/`（`rerun.sh` 在空目录从头跑了两遍，`compare.sh` 比出三份输出全同）。

## 0 前置与复跑环境

- 22 份被判文件：在冻结副本根下 `sha256sum -c research/prompts/m3-batch33t-code-r1-snapshot/crates-sha256.txt 2>&1 | grep -vc OK`，原样输出 `0`（22 份全 OK）。
- 没读禁读清单里的任何一份。读了：正文、背景材料附录、diff 全文、两份设计判决 `research/prompts/m3-c694-r1-main-verification.md`、`research/prompts/m3-c694-r2-main-verification.md`、实现员报告 `~/.local/share/singlefs/work/m3-batch33/t/report.md`（只当线索）。
- 开跑前 `ps -o pid,args -u "$(id -u)"` 看到别的会话的 `cargo test`（`-p singlefs-harness -- --ignored`、`-p singlefs-checker-tier …`），没有 qemu-system、vm-bench、e152、fio；我的命令都加 `nice -n 19`，经 `scripts/run-with-memory-cap.sh`（编译 6G、跑 4G）与 `capped.sh 10`，一次只起一条 cargo，用自己的编译目录与一份仓副本。
- 编译 + 探针 + 18 行变异整套 80 至 100 秒（`rerun.sh`，两遍）；checker 档一条没跑。
- 复跑产物：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/{probes,mutations,static}.txt`（四份输出两遍复跑后排序、去掉耗时字段，`compare.sh` 比出全同）。

## 1 K1 逐格

判定三种：一致 / 冲突 / 规则没说。「冲突」两边整行并排抄；每格末尾写什么现象会推翻它。

### T1 C 预隔离（`mount.rs`、`allocator.rs`）

| 子格 | 判定 | 条款（kb 行号） | 代码（冻结副本行号） |
|---|---|---|---|
| 隔离的集合是 `all`、水位之上按（实例，txg）比 | 一致 | `.claude/kb/decisions/23-journal的角色与格式.md:392`：「可写挂载读阶段、重建分配器之后，把水位之上、没被施加的全部 journal 记录点名的落点隔离」；同文件 `:370`：「记录水位 (实例代号, checkpoint_txg) 按实例代号为主比」 | `crates/singlefs-core/src/mount.rs:1670`：「> (selected_version.instance, selected_version.checkpoint_txg)」；调用点 `crates/singlefs-core/src/mount.rs:2369`：「let placements_named_by_unapplied_journal_records = match shadow_ledger {」；选法不滤末条标志，变异行 `第33批T C694 的 C 选法退成只认带末条标志的记录（last）` 复跑红 |
| 解开只在这个进程写掉那条记录 txg 的根环槽、那次发布取完落点之后 | 一致 | `.claude/kb/decisions/23-journal的角色与格式.md:392`：「只在这个进程写掉那条记录所属 txg 对应的根环槽、那次发布取完落点之后解开」 | 记录归槽用的函数与这个进程记根用的是同一个：`crates/singlefs-core/src/mount.rs:1715`：「.entry(target_for_publish(record.checkpoint_txg, slots_per_region))」与 `crates/singlefs-core/src/allocator.rs:2051`：「let ring_slot = target_for_publish(checkpoint_txg, occupancy.slots_per_region);」；解开只此一处 `crates/singlefs-core/src/allocator.rs:2064`：「.remove(&ring_slot)」，写那张表的另一处只有挂载时装表 `crates/singlefs-core/src/allocator.rs:2008`：「.placements_named_by_unapplied_journal_records = named_by_ring_slot;」 |
| 进程内重读读出被抛弃根不解开（`ring`，不借 D16 那一行的重读时机） | 一致 | `.claude/kb/decisions/16-发布语义.md:37`：「由可写挂载时的预隔离护着（D23（journal 的角色与格式） 已定项 14：水位之上、没被施加的全部 journal 记录点名的落点隔离，只在这个进程写掉那条记录所属 txg 对应的根环槽、那次发布取完落点之后解开）」 | 重读那一支 `crates/singlefs-core/src/allocator.rs:2173`：「RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable => {」一带没碰 C 的表；变异行 `第33批T C694 的 C 随 D16 已定项 1 读回全零那一行的进程内重读解开（d16）` 复跑红（`out/mutations.txt` 第 5 行） |
| 抬 F 那一串按新候选集重算 | 一致 | `.claude/kb/decisions/23-journal的角色与格式.md:390`：「每次挂载与每次抬 F 按当时的候选集重算」 | `crates/singlefs-core/src/mount.rs:3683`：「.map(RootRingOccupancy::placements_named_by_unapplied_journal_records)」把还没解开的整份交给同一个 `isolate_slots_referenced_only_by_abandoned_roots`（`crates/singlefs-core/src/mount.rs:3686` 起），豁免集合按新 F 现算 |
| D28 第九项 = 分配器实际隔离的集合，同一个数 | 一致 | `.claude/kb/decisions/28-挂载期承诺量.md:27`：「取值 = 分配器实际隔离的集合的槽数」 | 读法不变 `crates/singlefs-core/src/admission.rs:585`：「isolated_for_abandoned_roots_and_unapplied_journal_records:」取 `isolated_slots()`；新用例 `the_ninth_admission_term_equals_the_slots_isolated_for_the_records_above_the_water_mark_on_every_device` 复跑绿 |
| C 跟着只供测试的影子账开关走 | 一致（主 agent 已定读法） | — | `crates/singlefs-core/src/mount.rs:2377`：「ShadowLedger::Off => BTreeMap::new(),」 |
| 隔离的落点含记录新根段四条指针的位置条目 | 规则没说 | 条款只写「点名的落点」（`.claude/kb/decisions/23-journal的角色与格式.md:392`） | `crates/singlefs-core/src/mount.rs:1673`：「(&record.new_tree_table, TransactionUnit::TreeTable),」起四条；只多不少，安全方向；实现员报告第 6 条已记。会改第九项的数（这四条指针的槽也计入） |
| 候选根的豁免遇到 F2 退回之后「比所选版本新的有效根」 | 规则没说（前提被打破，见发现 2） | `.claude/kb/decisions/28-挂载期承诺量.md:27`：「不被回退候选集里的根引用的槽」 | `crates/singlefs-core/src/mount.rs:1566`：「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」 |

推翻 T1 各「一致」格的现象：任一变异行改回之后对应用例仍绿；或造出一个水位之上没施加的记录点名的槽在这个进程写掉那一根环槽之前被取走、而分配器上读到它是空闲的盘面。

### T2 B 凭据放行

判定：一致。条款 `.claude/kb/decisions/23-journal的角色与格式.md:390`：「凭据是内存里的现行账有一条代大于那条根的 txg、罩住它树表槽（与映射根槽）的记录，只看账、不读那一槽的内容，有它就不拒，没有它照拒」。代码 `crates/singlefs-core/src/mount.rs:1625`：「fn the_current_account_records_later_units_over_the_account_slots_of(」只读 `current_records`、不碰设备；比较 `crates/singlefs-core/src/mount.rs:1644`：「&& record.generation > root.checkpoint_txg」；调用点 `crates/singlefs-core/src/mount.rs:1608`：「if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {」，放在映射现算与重读一次都不成之后。
- 凭据对树表槽与映射根槽的每条位置条目（两块盘的副本）都要罩住，`.all`；映射根是空指针时只看树表槽。
- 已释放记录的 `generation` 是释放代（`crates/singlefs-core/src/allocator.rs:42`：「仍分配时是分配代；已释放时是释放代。」），代码注释写「分配代」不准，但结论不变：根的树表槽在当时那一版的账里要么空闲要么释放代不高于上一版的 txg，现行账里罩住它又代大于那条根 txg 的记录只能是之后的合法分配（推的，读 `allocator.rs` 的回收与复用推出，没造盘面反例）。
- 瞬时读不出那一形照拒：新用例 `an_abandoned_root_whose_account_slots_only_read_back_zero_still_refuses_the_unmount_before_any_write` 复跑绿，变异 `凭据不看现行账、读不出就放` 复跑红。
- 推翻：造一个现行账里有代大于那条根 txg、罩住它树表槽的记录，而那条根的树表槽此刻其实还是它自己的（读得出、校验对）的盘面。

### T3 D28 第九项改名

判定：一致。字段 `crates/singlefs-core/src/admission.rs:487`：「pub isolated_for_abandoned_roots_and_unapplied_journal_records: BytesOnOneDevice,」，读法不变（取分配器此刻的隔离槽数，`crates/singlefs-core/src/admission.rs:585`）。仓内旧名零命中：冻结副本根下 `grep -rn 'abandoned_root_exclusive' crates research`，输出为空。三份 harness 测试里的旧名随改。推翻：任一处还在引旧字段名而编译过（不可能，编译会红）；或第九项读到的数与 `DeviceFreeMap::isolated_slots` 不同。

### T4 F1 记录有缺口时拒可写

| 子格 | 判定 | 条款 | 代码 |
|---|---|---|---|
| 「序号不连续或缺末条、而任一点名单元验不过，重读一次仍这样就拒」 | 一致 | `.claude/kb/decisions/23-journal的角色与格式.md:392`：「水位之上一次没施加的发布，读得出的那几条记录本次发布内序号不连续或缺末条、而它们点名的单元又验不过，重读一次，仍这样就拒可写挂载」 | 序号 1..N 连着：`crates/singlefs-core/src/mount.rs:6003`：「let ordinals_run_from_one_without_a_gap = ordinals」；缺末条：`crates/singlefs-core/src/mount.rs:6007`：「let carries_a_last_record = records_of_the_publish.iter().any(|record| {」；任一点名单元验不过：`crates/singlefs-core/src/mount.rs:6012`：「!crate::recovery::every_named_unit_of_the_record_verifies(reader, record)」；合取 `crates/singlefs-core/src/mount.rs:6014`：「!(ordinals_run_from_one_without_a_gap && carries_a_last_record)」。重读：`crates/singlefs-core/src/mount.rs:6196`：「if let Some(publish) = reread」。验单元那一份与施加前验点名单元同一个函数：`crates/singlefs-core/src/recovery.rs:3526`：「pub(crate) fn every_named_unit_of_the_record_verifies<Reader: PoolReader + ?Sized>(」 |
| 与 I-8.9 是不是同一个判法 | 规则没说（不是同一个函数，更严） | `.claude/kb/invariants.md:98`：「只判读得出的记录，读不出的那条不判」 | 池级 checker 的判法 `crates/singlefs-checker/src/walk.rs:8047`：「fn judge_publish_ordinals_and_last_record_flags(」按「序号之差等于计数器之差」判跳号，读不出的那条隔着照样判过（`crates/singlefs-checker/src/walk.rs:8039`：「**射程**：只判读得出的记录，读不出的那条不判（I-8.9 判据原句）——」）；F1 要的是「读得出的序号从 1 起连着」，对一条读不出的记录会判不连续。照判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 6 条，F1 的口径是对的；只是两处写着同一个编号、判的不是同一件事 |
| 正常崩溃在写记录中途分得开 | 一致 | `.claude/kb/decisions/16-发布语义.md:194`（已定项 7）：「COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽」 | 单元先于记录落盘，记录读得出时它点名的单元验得过，`some_named_unit_does_not_verify` 为假；新用例 `a_power_loss_while_writing_the_records_of_a_publish_whose_units_verify_still_mounts_writable` 复跑绿，变异 `F1 乙不看点名单元验不验得过` 复跑红 |
| 判法三种缺法的变异 | 一致 | — | 三行变异（序号缺口两行、缺末条一行）复跑都红 |
| 水位取哪一版 | 规则没说（见发现 2） | `.claude/kb/decisions/23-journal的角色与格式.md:392`：「水位之上一次没施加的发布」 | F1 用读阶段的 `effective_root`：`crates/singlefs-core/src/mount.rs:6115`：「publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(」（实参 `&effective_root`，退回之前那一版）；C 用退回之后的：`crates/singlefs-core/src/mount.rs:6414`（`rebuilt_allocator` 的实参）。两处水位在 F2 退回的盘面上不同 |
| F1 的拒在不在 D23 那张「拒可写合法」单子上 | 规则没说（见发现 3） | `.claude/kb/decisions/23-journal的角色与格式.md:451`：「重建分配器读根环两遍并起来没有有效根的拒另起拒因、单子加一行」；F1 那段没写加不加单子 | `crates/singlefs-harness/src/mount_transparency.rs:2250`：「// F1 乙（用户 2026-10-07 定）那一拒：D23 已定项 14 那张单子还没写回它（kb 写回在主 agent），这里照「不在单子上」判。」 |

推翻 T4「一致」：造一个正常崩溃（单元落了、记录只落一部分）的盘面被 F1 拒；或一个读得出的序号 1..N 连着、末条带标志、点名单元验不过的发布被 F1 拒。

### T5 F2 退回见证到的那一版

| 子格 | 判定 | 条款 | 代码 |
|---|---|---|---|
| 三处是同一份判法 | 一致 | `.claude/kb/decisions/23-journal的角色与格式.md:396`：「恢复、可写挂载读阶段与只读挂载择根三处用同一套逻辑，都退回系统配置见证到的那一版」 | 三个调用点：恢复 `crates/singlefs-core/src/recovery.rs:5494`（`&scan_journal(reader, &system_configuration),` 喂同一个函数 `crates/singlefs-core/src/recovery.rs:5317`：「pub fn version_witnessed_by_the_system_configuration_to_fall_back_to<」）、可写挂载 `crates/singlefs-core/src/mount.rs:6352`：「version_witnessed_by_the_system_configuration_to_fall_back_to(」、只读挂载 `crates/singlefs-core/src/mounted_read.rs:626`：「version_witnessed_by_the_system_configuration_to_fall_back_to(」；资格闸只此一处 `crates/singlefs-core/src/recovery.rs:5329`：「!= (chosen_root.instance, chosen_root.checkpoint_txg)」（施加过记录的不退，主 agent 已定读法）；见证判不出（R2）不退 `crates/singlefs-core/src/recovery.rs:5351`：「if system_configuration_witness_is_undecidable(&slots_of_each_device) {」 |
| 「见证过」的判法 = D23 | 一致 | `.claude/kb/decisions/23-journal的角色与格式.md:396`：「它那次发布带末条标志的记录读得出、计数器 ≤ c_见证；末条读不出时拿计数器 = c_见证 的那条记录比 (txg, 实例)，不比所选那一版旧就算见证过；两条都读不出时当见证过、不退」 | `crates/singlefs-core/src/recovery.rs:5379`：「(Some(last_record_counter), _) => last_record_counter <= witnessed_journal_counter,」；`crates/singlefs-core/src/recovery.rs:5381`：「witnessed_publish >= order_of_the_root_choice(selected_version)」；`crates/singlefs-core/src/recovery.rs:5383`：「(None, None) => true,」。次序 (txg, 实例)，与条款字面相同；变异行五条（末条支两条、c_见证支、两条都读不出就退、施加过记录的也退）复跑都红 |
| c_见证 = 0 时 | 规则没说（与 N-配置不同，见发现 4） | `.claude/kb/decisions/23-journal的角色与格式.md:394`：「不是判不出时取全部自证过的槽 journal tail 的最大值 c_见证，0 ⇒ 不判」 | N-配置有这一支 `crates/singlefs-core/src/mount.rs:6059`：「} else if witnessed_journal_counter == 0 {」；F2 的函数里没有，c_见证 = 0 时任何有记录的所选根都算没见证 |
| 退到哪一条根 | 规则没说（见发现 5） | `.claude/kb/decisions/23-journal的角色与格式.md:396`：「退回系统配置见证到的那一版」 | `crates/singlefs-core/src/recovery.rs:5399`：「order < chosen && order <= ceiling」，候选取 `readable_roots`，不按实例表滤被抛弃的根；见证到的那次发布的根读不出时（`crates/singlefs-core/src/recovery.rs:5389`：「let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen);」）退到更旧的任一条 |
| 退回之后三处报告一致 | 一致 | — | 恢复报 `outcome.root` 与 `effective_root` 都是退到那条、`journal` 仍是原所选根那一趟；只读挂载 `chosen_root`、`effective_root` 都换；可写挂载换三样并把施加数清零（`crates/singlefs-core/src/mount.rs:6380`：「prefix_applied: 0,」），资格闸保证原本就是 0，所以三处 `journal` 同值。同一盘面三处结局相同：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:33`：「PROBE5 recover: effective_root=Some((i1, t8)) outcome=FileRead root=(i1, t8)」、`:34`：「PROBE5 read_only: Ok chosen=(i1, t8) effective=(i1, t8)」、`:35`：「PROBE5 writable: Ok effective=(i1, t8)」 |
| 记录核对器读 `effective_root` | 一致 | — | `crates/singlefs-checker-tier/src/crash.rs:149`：「if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)」只判不比落点新的发布，退回之后不会把被退掉的那次发布判成「恢复自称的 txg 该有它的单元」 |
| 读恢复报告再自己重推一遍的读者 | 冲突（见发现 1） | — | `read_recovery_fields` |

推翻 T5「一致」：同一盘面上三处退到不同的版本；或一个施加过记录的版本、或见证判不出（R2）的盘面被退回；或变异行改回之后用例仍绿。

### T6 C695 映射根全零先判树表 0 条

判定：一致。条款 `.claude/kb/checks-owed.md:546`：「映射根全零时先判树表 0 条、报 `TreeTableHasNoEntries`」（原文里反引号在引文比对时去掉）。代码 `crates/singlefs-core/src/mounted_read.rs:374`：「if root.mapping_root == NodePointer::empty_root() {」，空指针就是整条指针全 0（`crates/singlefs-core/src/pointer.rs` 的 `empty_root`）；只读挂载遇树表 0 条不退（`crates/singlefs-core/src/mounted_read.rs:617`：「Err(OpenPoolForReadFailure::TreeTableHasNoEntries) => {」），与恢复对树表 0 条的版本报 `NoFile` 同向。树表有条目而映射根是空指针的，仍往下读映射判红（条款原话）。新用例复跑绿、变异 `C695 甲映射根是空指针时照旧先读映射` 复跑红。「丢盘之后该择哪一版」不在这一格（归 C683）。推翻：一条映射根全零、树表有条目的根被报成 `TreeTableHasNoEntries`，或一条树表 0 条的根仍报槽 0 读不出。

### T7 C692 每盘落点下限按几何现算

判定：一致。条款 `.claude/kb/checks-owed.md:543`：「下限改成按系统配置读出的几何现算的值」。代码 `crates/singlefs-core/src/recovery.rs:4365`：「pub fn placements_on_each_device_after_the_new_pool_file_creation(」按盘字节数与单元区起点在空分配器上照新池新建文件那一版分一遍；比较 `crates/singlefs-core/src/recovery.rs:4342`：「first_device_placements.len() >= placements_on_each_device_at_least.0」。旧常量零命中：冻结副本根下 `grep -rn 'NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE' crates`，输出为空。
复跑（`out/probes.txt`）：
- 16 个几何里 15 个建得出池，预测值与盘上实有的每盘落点数逐盘相等（15 / 15，两块盘都相等）：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:1`：「PROBE1 small384 ua_slot=1408 predicted=12 actual_at_creation={0: 12, 1: 12}」、`:11`：「PROBE1 ring1GiB_dev4GiB ua_slot=66560 predicted=16 actual_at_creation={0: 16, 1: 16}」、`:13`：「PROBE1 default_dev4GiB ua_slot=50176 predicted=14 actual_at_creation={0: 14, 1: 14}」；另有 1 GiB 盘配默认环那一格盘建不出（`:12`：「PROBE1 default_dev1GiB predicted=12 POOL_BUILD_PANIC」，是 mkfs 起池 panic，不是下限的反例）。覆盖：单元区 240 / 256 / 320 / 384 / 512 / 1024 / 4096 槽、环 2 / 3 / 64 MiB / 默认 / 1 GiB、盘 4 / 16 / 64 GiB、io_min 8192。
- 实现员报告第 9 条担心的「两条一槽记录合成一个两槽落点、记录少一条」：回收只把记录标成已回收、不删记录（`crates/singlefs-core/src/allocator.rs:1926`：`fn reclaim_released_records_up_to(` 里只 `self.reclaimed.insert(key)`），记录数只在复用合并时少一条，每次发布又新增十几条；小盘上 400 次覆盖写与顺序写混合，每盘落点数最小值 22（下限 12）：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:17`：「PROBE1B predicted_bound=12 ops_ok=400 refused=0 min_per_device_count_seen=22」；unlink 之后只增（`:18`：「PROBE1C unlink_ok=true allocator_records_per_device before={0: 42, 1: 42} after_unlink={0: 61, 1: 61} predicted_bound=12」）。没找到低于下限的合法状态。
- 一个鲁棒性旁注（发现 6）：排演函数在盘比单元区起点短时 panic。
推翻 T7「一致」：任一合法镜像某盘落点数低于它那套几何的下限（走读误判红），或某个几何上预测值与新池新建文件后实有数不等。

### T8 注释、新测试、变异行

- 变异行：diff 里追加的 18 行与替换的 20 行，与冻结副本 `crates/mutations.tsv` 逐行相同、原文在文件里恰好命中一次、必须红的测试名在点名的测试文件里是 `fn`：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/static.txt:1`：「appended 18 replaced 20」、`:2`：「problems 0」。追加的 18 行逐行打进仓副本再还原、跑点名的用例：18 行全红且红的是点名的那条（`research/prompts/m3-batch33t-code-r1-sonnet-model/out/mutations.txt:1`：「ROWS 18」；`grep -c 'RED(expected test failed)'` 得 18）。替换的 20 行里 16 行（harness 档与 core `--lib`）我另用同一个驱动（`mutrun.py` 读 `replaced20.tsv`，见模型目录）打过一遍，全红且红的是点名的用例（这一遍没进 `rerun.sh`：其中一行单条 183 秒，整套会超 4 分钟；输出只在当时终端里，没落盘；要复核：`python3 -I mutrun.py <仓副本> <主仓> @replaced20.tsv`，`ROWS_FROM`、`ROWS_TO` 取行区间，checker 档那 4 行驱动自己跳过）；4 行在 checker 档（`bad_disk_input_campaign` 与 `crash_injection_campaign`），没跑（那四行的名字、参数、测试名「照旧」只能核到静态一致；替换前的表不在仓里，`git show HEAD:crates/mutations.tsv` 是更早的版本，比不出）。
- 新测试 15 条（5 份新文件，另 1 份是共用搭建）在冻结副本的拷贝上复跑全绿，每份 1 秒以内，不需要慢档标记。文件名按领域在前、场景在后，不带里程碑、步号、增补号、并行线号、欠账号（判法 `scripts/crash-case-check.py` 的 `MILESTONE_FILE_NAME`，我只读了它的正则、没跑它）。一条函数名带来历：`crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs:180`：「fn user_accepted_cost_of_2026_10_07_a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version(」；规则 `.claude/singlefs-ai-sop/rules/test-script-discipline.md:14`：「不带里程碑、步号、批次这类来历；来历进 commit message 或项目 kb。」没列日期，按「这类」读算擦边：判「规则没说死」。
- 注释：`mount.rs` 第 1475 行一带改成今天的行为（`crates/singlefs-core/src/mount.rs:1476` 起那段文档注释，写了映射现算、重读、凭据、拒），一致。残留两处没改成今天的：`crates/singlefs-core/src/mount.rs:1790`：「读不出、解不开交回 `None`（调用方只计数，不拒绝挂载）」（调用方现在可能拒，改之前就是旧话，不在 T8 点名的那一段）；`crates/singlefs-core/src/allocator.rs:266`：「只被被抛弃根引用的槽，当前账里既不是已分配也不是 defer，分配器却不许发出去，直到被抛弃的根离开根环。」`isolated` 字段的文档只说被抛弃根，没说水位之上记录点名的那部分。都是注释，不影响行为。
- 推翻：追加的某行变异改回后点名的用例仍绿（复跑 18 / 18 红）；或替换的行里有一行原文命中不是恰好一次（静态核 0 个问题）。

## 2 发现（冲突与规则没说，两边整行并排）

### 发现 1（T5，冲突）：读恢复报告的读者自己再推一遍根，F2 退回之后断言炸

条款与实现的一边：恢复退回之后报的是退到的那版——`crates/singlefs-core/src/recovery.rs:5522`：「(outcome, witnessed_version)」那一支把 `effective_root` 换成退到的那条（`crates/singlefs-core/src/recovery.rs:5536`：「effective_root: Some((effective_root.instance, effective_root.checkpoint_txg)),」）（`.claude/kb/decisions/23-journal的角色与格式.md:396`：「恢复、可写挂载读阶段与只读挂载择根三处用同一套逻辑，都退回系统配置见证到的那一版」）。
另一边：`crates/singlefs-harness/src/mount_transparency.rs:213`：「同一份盘上再择一次根、再重放一次，要落到恢复报告里那一版」，断言在 `crates/singlefs-harness/src/mount_transparency.rs:210`（`assert_eq!(`）把「`choose_root` + `replay_journal` 落到的根」与 `report.effective_root` 比；`read_recovery_fields` 的文档写「走不到」。F2 退回之后两边不等，必 panic。
复跑：已确认的 R、较新的系统配置槽与 R 的单元读回全零（S1 形），`recover` 退回上一版，再调 `read_recovery_fields` 两种 `ComparedVersion` 都 panic：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:30`：「PROBE4 S1 recover effective_root=Some((i1, t8)) outcome_is_file_read=true」、`:31`：「PROBE4 S1 TheRecoveredVersion read_recovery_fields PANICKED: assertion `left == right` failed: 同一份盘上再择一次根、再重放一次，要落到恢复报告里那一版」。
谁在调：`crates/singlefs-checker-tier/src/crash_injection.rs:1485`：「read_recovery_fields(&image, ComparedVersion::TheRecoveredVersion);」（崩溃注入第一截每个崩溃点）与第 3391 行、harness 的挂载透明用例。
覆盖度：登记的故障模型把根槽与点名单元一起读回全零（`crates/singlefs-harness/src/history.rs` 的 `CrashRecoveryAbandoningTheNewestRoot` 文档），根槽读不出时所选根就不是 R、F2 不触发，所以随机历史大概率走不到；手造盘面（S1、`ur` 形的断电镜像）一碰就 panic。checker 档没跑，是否被登记的流碰到我判不了（复核不了：要跑 checker 档）。
推翻：`read_recovery_fields` 在任何会让 `recover` 退回的盘面上都不被调（则只是潜伏的 panic）；或那条 `assert_eq!` 改成按退回后的根比。

### 发现 2（T4、T1、T5 交叉，规则没说；实测一形暴露与一形损伤）：F1 的水位在 F2 退回之前，C 的水位在退回之后；候选根豁免对「比所选版本新的有效根」没有前提

条款一边：F1 与 C 说的是同一个水位——`.claude/kb/decisions/23-journal的角色与格式.md:392`：「水位之上一次没施加的发布」与「把水位之上、没被施加的全部 journal 记录点名的落点隔离」。D28 的第九项豁免写「不被回退候选集里的根引用的槽」（`.claude/kb/decisions/28-挂载期承诺量.md:27`），条款没写候选根不比所选那一版新；F2 之前这一点自然成立（所选根是环里最新的有效根），F2 退回之后不再成立。
代码一边：读阶段里 F1 用退回之前的 `effective_root`——`crates/singlefs-core/src/mount.rs:6115`：「publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify(」；之后 F2 才在 `crates/singlefs-core/src/mount.rs:6342`：「match rebuild_previous_version(devices, unit_area_start, &effective_root, own_record) {」失败时把 `effective_root` 换成退到的版本，C 与影子账用的是换过的（`crates/singlefs-core/src/mount.rs:6414`：「&effective_root,」传给 `rebuilt_allocator`）。F2 退回时，被退掉的 R 的记录整体落到水位之上，F1 没再判；R 的根槽还读得出、账读得出时，R 还是「有效根」，它引用的槽在 `crates/singlefs-core/src/mount.rs:1566` 被候选豁免整个跳过。
复跑（384 槽小盘、第一个文件后覆盖写 5 次，再录一次顺序写三个数据单元、三条记录的发布 R，崩在 R 的根槽之后、轮换之前；`out/probes.txt`）：
- A（现有用例那一形：R 的根槽、全部单元、中间一条记录读回全零）：F1 拒，`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:20`：「PROBE23 A_root_zero_units_zero_mid_record_zero MOUNT_REFUSED NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify」。
- B（只把根槽换成读得出，其余同 A）：F1 不拒，可写挂载落到 P（txg 8）做成，缺的那条记录点名的数据单元在分配器上空闲、两块盘都是：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:22`：「(0, 1426, 2, true), (1, 1426, 2, true)」。这一次挂载写行根之前没取到它，所以没损伤；暴露的是「缺记录的单元没人护、F1 又没拒」那一格重新打开。
- D、E（根槽读得出、只有数据单元读回全零，账读得出；E 另加中间一条记录读回全零）：R 的账读得出，是候选根，C 把它点名的槽整个跳过；挂载做成、落到 P，写行根落盘之前断电的镜像上 R 的 13 个单元里 7 个（14 份副本）已被这次挂载写掉，池级 checker 在这份镜像上判红：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:26`：「I-7.4: 最新根（txg 9）引用的单元已被复用或抹头（校验和对不上或头用不了）」；恢复在这份镜像上读回 P（F2 生效）：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:27`：「PROBE23 D_root_READABLE_only_data_units_zero_account_readable MOUNTED effective_root=(i1, t8) r_key=(i1, t9)」。用户数据（P）无损，丢的是没被确认的 R。对照（`control.py`：只把水位之上记录点名那一段循环里的 `referenced_by_candidates.contains(&key) ||` 拿掉，其余不动，跑完还原）：D 变成「空闲 0 份、写掉 0 份、恢复落到 R」——`research/prompts/m3-batch33t-code-r1-sonnet-model/out/control.txt:3`：「CONTROL D_root_READABLE_only_data_units_zero_account_readable free_copies 0 overwritten_copies 0 recover_lands_on i1 t9」；这一形的机理就是那一处候选豁免。B 与 E 的缺记录点名的那一个单元（槽 1426）拿掉豁免也仍空闲（`research/prompts/m3-batch33t-code-r1-sonnet-model/out/control.txt:1`：「CONTROL B_root_READABLE_units_zero_mid_record_zero free_copies 2 overwritten_copies 0 recover_lands_on i1 t9」），那是 F1 本该拒的那一格。
- 范围：登记的故障模型把根槽与点名单元一起读回全零（`crates/singlefs-harness/src/history.rs` 的 `CrashRecoveryAbandoningTheNewestRoot` 文档），B、D、E 都在注册范围之外；归 K2 攻方去判值不值得改。
- 推翻：F2 退回时 F1 与 C 用同一个水位且候选豁免不含被退掉的 R；或登记的流里任何一个盘面走不到 B / D / E。

### 发现 3（T4、T8，规则没说）：F1 的拒不在 D23 那张「拒可写合法」单子上，崩溃注入第二截会把一次合法的 F1 拒报红

代码一边：`crates/singlefs-harness/src/mount_transparency.rs:2647`：「/// 理由不在单子上：照崩溃注入第二截今天的写法判（单元区墙计数，别的报红）。」与 `crates/singlefs-checker-tier/src/crash_injection.rs:2662`：「Some(format!("崩溃后镜像上的可写挂载拒了：{member}"))」（`:2655` 是那条臂）。F1 被分类成不在单子上：`crates/singlefs-harness/src/mount_transparency.rs:2251`：「| StillUnreadableAfterOneReread::PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify { .. } => None,」；模型对拍也记成没理由 `crates/singlefs-harness/src/model_comparison.rs:842`。
条款一边：F1 那段只写「重读一次，仍这样就拒可写挂载」（`.claude/kb/decisions/23-journal的角色与格式.md:392`），没写加不加单子；前例是 NoValidRoot 那一拒写明「单子加一行」（`.claude/kb/decisions/23-journal的角色与格式.md:451`）。
后果（推的，checker 档没跑）：崩溃注入若在记录段中间切、同时给点名单元注入读回全零，一次按条款该拒的可写挂载会被报成「模型说该成、实现拒了」。随机历史单记录发布走不到（实现员报告第七节第 48 行同判）。
推翻：崩溃注入的登记流里没有任何状态同时满足「读得出的记录序号缺口或缺末条」与「点名单元读回全零」。

### 发现 4（T5，规则没说，推的）：F2 没有 N-配置的「c_见证 = 0 不判」那一支
见 T5 表第三行。c_见证 = 0 时 F2 把所选版本算成没见证、有退回的候选就退；N-配置对同一读数是「什么都没见证」、不拒。条款 F2 段没写 0。我没造盘面（要所选根本身走读失败、而每块盘自证过的系统配置 journal_tail 都是 0，只有第一次发布的轮换没落的池走得到），推翻：条款补一句 0 怎么处置、或代码补上那一支。

### 发现 5（T5，规则没说，推的）：F2 退到的候选不按实例表滤、也不要求是见证到的那次发布自己的根
见 T5 表第四行：`readable_roots` 全部自证过的根里取「比所选根旧、不比见证到的那一次发布新」的最大一条（`crates/singlefs-core/src/recovery.rs:5399`）；若见证到的那次发布的根读不出，会取到更旧的、可能是被实例表抛弃的时间线上的根。条款只说「退回系统配置见证到的那一版」（`.claude/kb/decisions/23-journal的角色与格式.md:396`）。没造盘面。推翻：造一个退回取到被抛弃时间线上根的盘面，或条款写明取法。

### 发现 6（T7，鲁棒性旁注，推的）：排演函数在盘比单元区起点短时 panic，而它是实参、在被比较的记录判空之前就算
代码：`crates/singlefs-core/src/recovery.rs:4557`：「if !allocation_records_are_one_per_device(」的第三个实参 `crates/singlefs-core/src/recovery.rs:4560`：「placements_on_each_device_after_the_new_pool_file_creation(」先求值。盘比单元区起点短时 panic：`research/prompts/m3-batch33t-code-r1-sonnet-model/out/probes.txt:37`：「PROBE7 device_bytes=1048576 unit_area_start_bytes=822083584 PANICKED: 建空闲图的盘末尾不在单元区起点之前」。只有可写挂载在读盘之前拒这种盘（`crates/singlefs-core/src/mount.rs:6317`：「every_device_reaches_the_unit_area_start(&*devices, unit_area_start)?;」）；恢复与只读挂载走到这里之前有 `allocation_records_fit_the_pool_geometry`（`crates/singlefs-core/src/recovery.rs:2840`、`crates/singlefs-core/src/recovery.rs:5155`）拒掉落点超出盘的记录，所以要碰到这个 panic 得是记录列表为空而盘又短的坏镜像。没造那一形。推翻：造一份恢复或只读挂载能走到排演而盘短于单元区起点的镜像，或把排演挪到比较之后。

## 3 K3 22 份每份一格（`crates-sha256.txt` 的次序）

| # | 文件 | 落在哪一格 | 判定 | 一句话 |
|---|---|---|---|---|
| 1 | `crates/singlefs-core/src/admission.rs` | T3（连带 T1 第九项读数） | 一致 | 字段改名、读 `isolated_slots()` 不变；四处旧名无残留 |
| 2 | `crates/singlefs-core/src/allocator.rs` | T1 | 一致 | C 的表与解开、补隔离都对上 D23 第 392 行；字段文档 `:266` 没写 C 那一部分（注释） |
| 3 | `crates/singlefs-core/src/mount.rs` | T1、T2、T4、T5、T8 | 一致，带发现 2、3 | 隔离、凭据、F1、F2 可写一支都对上条款；F1 与 C 的水位、候选豁免在 F2 退回上有缺口；`:1790` 一句旧注释 |
| 4 | `crates/singlefs-core/src/mounted_read.rs` | T5 只读一支、T6 | 一致 | F2 同一份判法、树表 0 条不退；映射根空指针先判树表 0 条 |
| 5 | `crates/singlefs-core/src/recovery.rs` | T5 判法与 `recover`、T7、T4 共用的验点名单元、T1 点名单元跨度 | 一致，带发现 4、5、6 | 见证判法照 D23 第 396 行；下限 15 / 15 个几何与盘上实有相等；c_见证 = 0、候选不滤实例表、排演 panic 三处条款没说 |
| 6 | `crates/singlefs-harness/src/history.rs` | T4 新成员的名字映射 | 一致 | `crates/singlefs-harness/src/history.rs:3106` 一臂，穷举 match 不留通配 |
| 7 | `crates/singlefs-harness/src/model_comparison.rs` | T4 两处臂 | 一致 | F1 记成无理由 / `None`（`:842`、`:950`）；随机历史不藏记录，模型没有这一条 |
| 8 | `crates/singlefs-harness/src/mount_transparency.rs` | T4 单子分类、T5 读恢复报告的读者 | 冲突 + 规则没说 | `:2251` F1 判不在单子上（发现 3）；`:210` 的断言在 F2 退回上 panic（发现 1，这一处不是本批改的，被 T5 打到） |
| 9 | `crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs` | T3 | 一致 | 只改名；变异两行复跑红 |
| 10 | `crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs` | T3 | 一致 | 只改名；变异 `不扣环窗口` 复跑红 |
| 11 | `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs` | T8、T1 | 一致 | 造「盲挂载」改用影子账关着的入口，靠「C 跟开关走」那条已定读法 |
| 12 | `crates/singlefs-harness/tests/common/mod.rs` | T8 | 一致 | 新入口 `…_with_the_shadow_ledger`，默认 On 与原 `mount_writable` 同义（`mount_writable` 就是 `JudgedByTheFormula` 加默认开关） |
| 13 | `crates/singlefs-harness/tests/release_checksum_quarantine.rs` | T3 | 一致 | 只改名 |
| 14 | `crates/singlefs-harness/tests/row_publish_record_without_its_root.rs` | T1（C533 改结局与名字） | 一致 | 新断言「写行取别的槽、六个落点两盘都不空闲」就是 C 的后果；两行变异复跑红 |
| 15 | `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` | T8、T1 | 一致 | M3 用关影子账的入口造「看不见 (2, 3)」那一形，随 C 跟开关走 |
| 16 | `crates/mutations.tsv` | T8 | 一致 | 追加 18 行全红；替换 20 行静态全对、16 行复跑红、4 行在 checker 档没跑 |
| 17 | `crates/singlefs-harness/tests/abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs` | T2 | 一致 | 两条用例复跑绿，对应两行变异红（拿掉凭据红、读不出就放红） |
| 18 | `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs` | T8（五份新文件的共用搭建） | 一致 | 故障模型是「读回全零、盘上字节不动」，与 `history.rs` 注入同一种；我的 B、D、E 探针直接用它造盘面 |
| 19 | `crates/singlefs-harness/tests/mounted_read_reports_an_empty_tree_table_before_reading_an_empty_mapping_root.rs` | T6 | 一致 | 环里每条树表 0 条的根报 `TreeTableHasNoEntries`，只读挂载同；变异红 |
| 20 | `crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` | T5 | 一致 | 四条用例复跑绿、四行变异红；`:180` 函数名带日期来历（规则没说死） |
| 21 | `crates/singlefs-harness/tests/writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten.rs` | T1 | 一致 | 四条用例复跑绿、四行变异红；`ring`、`all`、第九项各有一条 |
| 22 | `crates/singlefs-harness/tests/writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero.rs` | T4 | 一致 | 四条用例复跑绿、四行变异红；正常崩溃那一条守住不拒 |

## 4 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-T1 隔离集合 `all`、水位序、解开 `ring`、重读不解、抬 F 重算、第九项、影子账开关 | 一致 | 七处都对上 D23 第 392 行、D16 第 37 行、D28 第 27 行；变异行红 |
| K1-T1 新根段四条指针也隔离 | 规则没说 | 条款只写「点名的落点」；多隔离、安全方向 |
| K1-T1 / T4 / T5 候选根豁免与 F1 水位在 F2 退回上 | 规则没说（实测） | 发现 2：F1 水位在退回前、C 在退回后，候选豁免挡掉被退掉的 R 的保护；B 暴露、D / E 损伤（对照坐实机理），都在登记故障模型之外 |
| K1-T2 B 凭据 | 一致 | 只读现行账；瞬时读不出那一形照拒 |
| K1-T3 第九项改名 | 一致 | 读法不变 |
| K1-T4 判法与条款 | 一致 | 序号 1..N 连着、有末条、任一点名单元验不过、重读一次 |
| K1-T4 与 I-8.9 同一个判法 | 规则没说 | 不是同一个函数；F1 更严（对读不出的那条判不连续），checker 的 I-8.9 不判 |
| K1-T4 正常崩溃分得开 | 一致 | 单元先于记录落盘；用例与变异守住 |
| K1-T4 F1 的拒不在 D23 单子上 | 规则没说 | 发现 3 |
| K1-T5 三处同一份判法、见证过 = D23、三处报告一致、记录核对器 | 一致 | 同一函数、同一资格闸、同盘面三处结局相同 |
| K1-T5 读恢复报告的读者 `read_recovery_fields` | 冲突 | 发现 1：F2 退回上断言 panic，已复跑 |
| K1-T5 c_见证 = 0、退回候选不滤实例表 | 规则没说 | 发现 4、5，推的 |
| K1-T6 | 一致 | 映射根空指针先判树表 0 条；只读挂载不退 |
| K1-T7 | 一致 | 15 / 15 个几何预测等于实有；没找到低于下限的合法状态；发现 6 旁注 |
| K1-T8 变异行与测试 | 一致 | 追加 18 行全红、替换 20 行静态全对、16 行复跑红；4 行 checker 档没跑 |
| K1-T8 注释与名字 | 一致，带三处小残留 | `mount.rs:1790`、`allocator.rs:266` 旧注释；一条函数名带日期 |
| K3 22 份 | 22 / 22 判到 | 1 份冲突（`mount_transparency.rs`）、1 份带规则没说、其余一致 |

## 5 没做什么

- K2 不是我的格：没替攻方判「哪一格会错」，发现 2 的 B / D / E 只写了盘面与数，值不值得改归主 agent 与攻方一腿。
- checker 档一条没跑；`crash_injection.rs` 一族读 `recover` 与 `read_recovery_fields` 的用例是否在登记流里碰到发现 1、发现 3，复核不了（要跑 checker 档）。
- 替换的 20 行变异里 4 行在 checker 档（`bad_disk_input_campaign` 的 `allocation_records_released_on_the_second_device_only_are_refused_at_the_one_record_per_device_judgement_by_both_readers`；`crash_injection_campaign` 的 `units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`、`crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons`、`on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red`），没跑，归提交时那一趟。
- 「照旧」那一半（替换前的名字、参数、测试名）核不了：替换前的表不在仓里。
- 没跑 `crates-mutation-rows.py static-check`、`crash-case-check.py` 等门禁判法，只自己写了静态核（`static_rows.py`）与读了它们的正则。
- 没跑全量 harness 档（主 agent 合入验证已跑 175 个测试二进制）；只跑了新增的 5 份测试文件、与每条变异点名的用例。
- 发现 4、5、6 是读代码推的，没造盘面。
- 没读也没引攻方腿的报告、模型与目录；没写判决、没替主 agent 采纳。
