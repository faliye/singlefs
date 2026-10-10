# m3-batch37-43-code-r1 云端正推腿报告（Sonnet，立场：正推；格 K1、K3）

2026-10-08 · 冻结副本根 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree`（下文 TREE 指它；引代码写绝对路径:行号）· 模型目录 `research/prompts/m3-batch37-43-code-r1-sonnet-model/` · 草稿 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1-sonnet`

## 0 现场

- 23 份 sha256：`cd TREE && sha256sum -c research/prompts/m3-batch37-43-code-r1-snapshot/crates-sha256.txt` 的 OK 数：
  `23`（命令在 TREE 里跑，`grep -c ': OK$'`；没有一份不 OK）。
- 开跑前 `ps -o pid,args -u "$(id -u)"`：没有 qemu-system、vm-bench、fio、e152 在跑；有另一条会话的 `cargo test`（`run-with-memory-cap.sh 24G … cargo test --offline --no-fail-fast -p singlefs-format -p singlefs-core -p singlefs-harness …`）和一条 e174 装置在跑，占着内存包装的 slice（32.3 GiB / 40.1 GiB）；我的命令都带 `nice -n 19`。
- 我起过一次 8G 的循环（8 个小测试二进制排在一条 `for` 里），第一条在包装里排队 200 秒被 `timeout` 杀（退 124，不是测试结果，输出不算）；我把那串自己起的进程逐个 `proc.py stop` 掉，列了一遍 `ps` 确认我的一个不剩，改成每个小测试二进制 4G、逐个跑。小测试二进制每个不到 1 秒跑完。
- 只用一份仓副本（草稿目录 `repo`）、一个编译目录（同目录 `target`），一次一条 cargo；编译经 `run-with-memory-cap.sh 6G` + `capped.sh 15`，跑经 `4G`，线程 15。
- 方法：先从 D23 已定项 14 的条款推「应该是什么」，再读 TREE 今天的代码，再复跑 harness 档用例与变异行；kb 条款写 kb 文件自己的行号。
- checker 档（`-p singlefs-checker-tier` 的测试与变异行）与 GPU 不跑；GPU 一侧读 wgsl，与 CPU 逐支对。

## 1 基线（harness 档，副本上现跑）

编译：`cargo test --offline --no-run -p singlefs-core --lib -p singlefs-harness` 加 10 个 `--test`，退 0。下面每行是 `run-<目标>.log` 里 `test result` 的原样（去掉耗时字段）：

| 目标 | 结果行 |
|---|---|
| recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed | `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;` |
| reuse_after_raising_the_floor（默认档，13 条 ignored 是慢档） | `test result: ok. 3 passed; 0 failed; 13 ignored; 0 measured; 0 filtered out;` |

另跑：`random_histories` 默认档 `test result: ok. 23 passed; 0 failed; 19 ignored; 0 measured; 0 filtered out;`；core lib `test result: ok. 200 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;`；慢档（`#[ignore]`）由第 5 节变异行 ROW 11（`random_histories`）与 ROW 37（`reuse_after_raising_the_floor`）带 `--include-ignored --exact` 跑到。两条慢档用例在干净副本上各自 `--include-ignored --exact` 跑：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out;`（`random_histories`）、`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out;`（`reuse_after_raising_the_floor`）。

推翻这一节的现象：同一份 TREE 在空目录重编后任一行不是 ok。

## 2 K1 逐格（上）：X1–X5

约定：「应该是」从 D23 已定项 14（`.claude/kb/decisions/23-journal的角色与格式.md`）与判决 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 推；「今天是」读 TREE 代码；判定三选一（一致 / 冲突 / 规则没说）。

### X1　变异表第 3075、3116、3117 行重取锚点

- 应该是：判决第二节 K1 W7、V7 行（三行原文被 V 改没，重取锚点）。
- 今天是：`static_anchor.py` 在副本上扫全表：`ROWS 3306 ANCHOR-BAD 0`、`NAMES-UNIQUE True`（输出见模型目录 `expected/static.txt`）；三行都在 checker 档（`crash_judge_common.wgsl` ×2、`crash_judge_checker.wgsl` ×1），点名的测试是 checker 档的，**我不跑**，只核锚点命中恰好一次。
- 判定：**一致**（锚点命中 1 次；红不红归主 agent 经门禁跑）。
- 推翻：门禁 59 号在这三行报「原文不在文件里」，或这三行点名的测试不红。

### X2　双故障慢档用例改名、期望改成抬 F 之前池级 checker 一条都不红

- 应该是：判决第三节第 1 条「期望改成不红」；D23 预隔离那一段说 `C` 护住被抛弃那条根的单元。
- 今天是：用例名 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_leaves_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units` 在 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/tests/random_histories.rs:1330`，断言在同文件 1349 行 `violations.is_empty(),`；旧名与旧的「恰好 I-7.4 红」断言都删了（diff 里 `-` 行）。
- 判别力（现跑）：变异行 3258（`for placement in placements_named_by_unapplied_journal_records {` 换成 `.iter().take(0)`）在副本上 **红**：`ROW 11 rc=101 RED(expected test failed) nfailed=1`（`expected/mutations.txt` 的 `ROW 11`）。第一次 ROW 11 打了 `TIMEOUT`：那一次是内存包装 8G 在排队（别的会话占着 slice），不是结果；重跑 2 分 19 秒红。
- 判定：**一致**。
- 观察（小）：行 3258 / 3284 的名字说「不隔离水位之上没施加的记录点名的落点」，替换文 `.iter().take(0)` 把传给 `isolate_slots_referenced_only_by_abandoned_roots` 的**整个 C**（记录点名的 + `C_ACCOUNT` 的账 + `C_NEWER_ROOTS` 的账，已在 `rebuilt_allocator` 里并成一份）都清空了。我另做了三个窄版（`adhoc-cases.tsv` A1 去掉并之前「记录点名的」那一份、A2 去掉 `C_ACCOUNT` 并进来的账、A3 去掉 `C_NEWER_ROOTS` 并进来的账）：这两条慢档用例在三个窄版上**都不红**（`adhoc.txt` 里 `ADHOC A1 0/1`、`ADHOC A2 0/1`、`ADHOC A3 0/1` 全是 GREEN）。双故障那一形里三份各自都护得住 R 的单元，只有整个 C 清空这两条才红；点名那一份另由 `the_units_named_by_the_first_two_records_…_are_isolated` 单独钉（`ADHOC A1 2` RED），`C_NEWER_ROOTS`、`C_ACCOUNT` 各有自己的变异行。所以这两条用例钉的是「C 整体在」，不是三份各自；名字照 `isolate_slots_referenced_only_by_abandoned_roots` 的参数名 `placements_named_by_unapplied_journal_records`（并好的整个 C），不算错，只是读成「只有记录点名的那一份」会读窄。
- 推翻：窄版 A1 在这两条慢档用例上红了，或整个 C 清空它们不红。

### X3　F1 的拒归进 D23 单子；崩溃注入第三截补两臂、一格计数

- 应该是：records 第 1234 行（主 agent 定：在镜像上复算、不加 `RefusalConditionNotMet` 成员）。
- 今天是：
  - `RefusalReasonOnTheD23List` 加成员 `PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`；`refusal_reason_on_the_d23_list` 七个 `StillUnreadableAfterOneReread` 成员逐个对过（TREE `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2276` 起）：`PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion`→单子；`InstanceTableOfTheNewestRootForTheShadowLedger`→单子；`PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`→单子（X3）；`AccountOfAnUnappliedPublishWithIncompleteReadableRecords`→单子（C1）；`InstanceTableOfTheNewestRootForTheEffectiveFloor`、`RootRingSlotKnownToHoldARoot`、`AccountOfAnAbandonedRoot`→`None`（不在单子上）。七个都有一臂、没有通配。
  - 复算 `publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify_on_the_crash_image` 只用 `choose_system_configuration`、`scan_journal`、自己写的 `named_unit_has_a_copy_that_verifies_on_the_crash_image`（读崩溃镜像、比整单元 CRC-32C），没有调实现里判拒的 `mount::publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`（grep 见下）。`RefusalConditionNotMet` 枚举没动。
  - 崩溃注入：`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs` 加了一格计数 `remount_refusals_for_a_publish_above_the_water_mark_missing_records_whose_named_units_do_not_verify` 与 `Legal` / `Illegal` 两臂（checker 档，**不跑**，读码核）。
- 判定：**一致**。
- 推翻：另有成员在 `refusal_reason_on_the_d23_list` 里归错；或复算函数里出现对实现里同名判拒函数的调用。

## 3 K1 逐格（中）：Y1–Y6

### Y1　`C_NEWER_ROOTS`：认哪几条根

- 应该是：D23「再并上根环里比所选那一版新（按择根的序）、按最新那条根的实例表有效、读得出的根自己的账里的落点」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）；按那条根的 txg 对应的根环槽归组。
- 今天是：筛根三个条件在一处：「!is_abandoned(root)」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1926`，按最新那条根的实例表有效）、「&& (root.checkpoint_txg, root.instance)」与「> (selected_version.checkpoint_txg, selected_version.instance)」（同文件 1927–1928，择根的序、txg 为主实例破平局，比所选那一版新）；`roots` 是重建分配器读根环那一遍读出的根（读不出的槽不在其内）。账按 `target_for_publish(root.checkpoint_txg, slots_per_region)` 归组（同函数末尾 `.entry(target_for_publish(root.checkpoint_txg, slots_per_region))`）。调用点在 `rebuilt_allocator`，传的 `selected_version` 是退过 F2 之后的 `effective_root`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6852`），所以 F2 退回之后被退掉的那条根满足「比所选那一版新」。
- 账读不出时三分：记录一条都读不出——映射现算得出就并、现算不成「None => continue,」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1952`，不拒、不重读）；记录不齐——走 `placements_of_an_unapplied_publish_whose_account_is_unreadable`（可拒）；记录齐——`continue`（交 F1 乙）。与 D23 两句逐句对上：「那次发布读得出的记录齐（本次发布内序号 1..N 连着、带末条）就不加」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）、「不齐（缺序号或缺末条）就照被抛弃根的账读不出那一份办」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。
- 判定：**一致**。
- 变异（现跑）：行 3263–3266 四行（`.extend(placements_in_the_newer_roots_account.into_iter().take(0))`）都 `RED(expected test failed)`（`ROW 16–19`）。
- 推翻：一盘面上比所选那一版新、读得出、按表有效的根的账没被并进 C，或一条被抛弃的根被并进来。

### Y2　`C_MAPPING`（收窄）：什么时候现算、什么时候拒

- 应该是：D23「不齐（缺序号或缺末条）就照被抛弃根的账读不出那一份办」「较新可读根那次发布的记录一条都读不出、账也读不出时，映射现算得出就隔离，现算不成不隔离、不拒」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）；被抛弃根那一份的步骤是 D23 :390 一带写的（转述）映射现算、重读一次、凭据、拒（`.claude/kb/decisions/23-journal的角色与格式.md:390` 同一行：「从中央映射现算这条根引用的落点，只在内存里隔离、不写盘」）。
- 今天是：共用函数 `placements_of_a_root_whose_account_is_unreadable_unless_still_unreadable_after_one_reread`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1682`）：先映射现算；不成就 `before_the_one_reread.before_rereading();`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1696`）、账或映射再读一次；再不成看凭据「if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1704`）→ `Some(Vec::new())`；否则 `None`。被抛弃根那一份（`placements_of_an_abandoned_root_whose_account_is_unreadable`，`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1648`）与 C 的没施加的发布那一份（`placements_of_an_unapplied_publish_whose_account_is_unreadable`）都只是这一份加各自的拒因。三条判「不齐」的调用都用同一个 `readable_records_of_the_publish_leave_a_gap_or_miss_the_last`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6395`，F1 乙也用它）：序号排好不是从 1 逐个加一、或没有一条带末条标志（空集也算不齐）。
- 「记录一条都读不出」那一支（H1 的 `urJ` 形）：不重读、不拒（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:1942` `if readable_records_of_its_publish.is_empty() {`），照 records 第 1266 行 ①（主 agent 定）。
- 判定：**一致**。
- 变异（现跑）：`ROW 20–26` 七行 C_MAPPING 变异（不现算 ×2、现算不成也不拒、什么都不交、不收窄 ×2、记录一条都读不出也拒）全 `RED(expected test failed)`；第 43 批替换的 `ROW 22`、`ROW 23` 两行同红。其中 `ROW 26`（行 3273）的替换文拒的时候用的是旧成员 `AccountOfAnAbandonedRoot`，不是新成员——它模拟「也拒」，红不红不受成员影响；名字与替换文对得上；成员沿用 C393 的那个是遗留，不影响红。
- 推翻：记录齐的发布账读不出时拒了可写挂载；或不齐、映射现算不成、重读仍不成、凭据也没有而没拒。

### Y3　`F1_AFTER_F2`：退到的那一版当水位

- 应该是：D23「按退到的那一版当水位再判一次」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）：重读一次，仍这样就拒，拒因同一个。
- 今天是：`mount_writable_with_every_test_only_switch` 里退回分支「let Some(version_fallen_back_to) =」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6787`）之后、建上一版成功的臂 `Ok(previous) => {`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6811`）里调 `refuse_publishes_above_the_version_fallen_back_to_missing_records_whose_named_units_do_not_verify`，传的是 `&version_fallen_back_to` 当水位（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6813` 起）；该函数先判一遍，空就过，非空就 `before_the_one_reread.before_rereading();` 再判一遍，仍非空 `Err(NewerStateStillUnreadableAfterOneReread(PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify))`，点名 `first()`（按 (实例, txg) 最小）。点名单元直接读盘，只读不写。之后 `rebuilt_allocator` 用的也是退到的那一版（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6852`）。
- 判定：**一致**。
- 变异（现跑）：行 3274、3275（`)?;` → `).ok();`）`ROW 27、28` 红；行 3276（`|| true`）`ROW 29` 红；行 3307（去掉重读前的钩子调用）`ROW 43` 红。
- 观察（小）：水位判的是退之前的 `records`（读阶段扫出的那一份），退回之后不重扫环；两次扫之间盘上没写，读故障下 `scan_journal` 的结果可以与再扫不同，但 D23 写的就是「记录取读阶段扫出的那一份」（函数文档同句）。规则没说读故障下再扫一遍。
- 推翻：退回之后按所选那一版而不是退到的那一版判；或退到的那一版之上有缺口发布、点名单元两份都验不过、读故障撤掉后仍验不过却没拒。

### Y4　`F2_NEWEST`（含 A1）：区间、过滤、各不退

逐句 D23 :396 的 F2 一段对 `recovery.rs` 的 `newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5429`）：

| D23 :396 的句子（逐字） | 代码 | 一致？ |
|---|---|---|
| 「退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根」 | 「witnessed_publish <= order」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5527`）、同块 `order < chosen`、`readable_roots(` 取读得出的根（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5513`）、「Some(table) => !root_is_abandoned_by_the_instance_table(root, table),」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5524`）、「.max_by_key(order_of_the_root_choice)」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5531`；并列取最后，GPU 同） | 是 |
| 「所选根的实例表读不出时只认与见证到的那一版恰好相等、读得出的那条根」 | 「None => order == witnessed_publish,」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5525`），表来自 `instance_table_chain_of_root(reader, chosen_root).ok()` | 是 |
| 「那条记录读不出、c_见证 = 0、那一段里没有读得出又不被抛弃的根都不退」 | 「let witnessed_publish = record_at_the_witnessed_counter?;」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5506`）、「if witnessed_journal_counter == 0 {」后 `return None;`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5474`–5475）、`.max_by_key` 空集 `None` | 是 |
| 「c_见证 落在所选那次发布自己的某条记录上时那一段是空的，所选那一版算见证过、不退」 | 那条记录的 (txg, 实例) = 所选根的 ⇒ `witnessed_publish <= order && order < chosen` 无解 ⇒ `None`；所选那一版末条读不出时「witnessed_publish >= order_of_the_root_choice(selected_version)」算见证过 | 是 |
| 「两条都读不出时当见证过、不退」 | 「(None, None) => true,」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5500`） | 是 |
| （D23 之外，前几轮已留）施加过记录的那一版不退；见证判不出不退 | 「if system_configuration_witness_is_undecidable(&slots_of_each_device) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5463`）；函数头 `(selected_version.instance, selected_version.checkpoint_txg)` 与 `chosen_root` 不等就 `None`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5440`） | 规则没说（沿旧例，函数文档已写） |

- A1（c_见证 = 0 不退）：CPU 在 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5474`；GPU 在 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:208`「if (is_witnessed || is_zero64(witnessed_counter) || !has_witnessed_record) {」；两边都是「环里读得出计数器 0 的记录也一样不退」。
- 三处同一份判法：恢复 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5619`、只读挂载 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mounted_read.rs:626`、可写挂载 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6788` 起，调的是同一个函数名；每处传 `&chosen_root`、`&effective_root`（施加记录之后的那一版）与各自的 `records`。
- 变异（现跑）：`ROW 30–33`（`<=` → `==`，四个采样点）、`ROW 34`（见证记录读不出照退）、`ROW 35`（不按表滤）、`ROW 36`（表读不出照收最新）、`ROW 41`（c_见证 = 0 守卫 `if false`）全 `RED(expected test failed)`；`ROW 9`（第 35 批 F2_EXACT 取「见证上限之下任一条可读根」的替换文）也红。
- 判定：**一致**。
- 推翻：见下一节 GPU 逐支对上之后，任一盘面 CPU 与 GPU 退到不同的根；或 D23 :396 一句在代码里找不到对应分支。

### Y5　第 3163 行换取样点

- 应该是：判决 K1 W7、V7 行：第 3163 行（第 33 批 T「C 选法退成只认带末条标志的记录」）在 `C_ACCOUNT` 之下存活，补一个分得开的取样点（`C_ACCOUNT` 读不到账、末条读不出的那一形）。
- 今天是：该行点名的测试换成 `the_units_named_by_the_first_two_records_of_a_publish_whose_root_slot_and_tree_table_read_back_zero_are_isolated`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs:140`）：R 的三条记录齐、R 的根槽与树表读回全零，`C_ACCOUNT` 读不到账（树表读不出）、也不从映射现算（记录齐），前两条记录点名的落点只有 C 的「记录点名」那一份护着；断言这些落点 M1 之后「!device_map.is_free(location.slot)」。
- 现跑：`ROW 4 rc=101 RED(expected test failed) nfailed=1`（单独施加这一行、不带 `C_ACCOUNT` 变异就红）。基线 `writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero` 4 条过。
- 判定：**一致**（取样点现在单独就分得开）。
- 推翻：把这一行的变异与别的 C 变异同时去掉后它不红；或测试名与该行第六段不一致。

### Y6　`reuse_after_raising_the_floor.rs` 同形用例改名、改期望

- 应该是：判决第三节第 1 条（期望改成不红）。
- 今天是：`four_overwrites_after_crash_recovery_abandoned_the_third_version_leave_the_pool_checker_green_before_any_floor_raise_since_the_writable_mount_isolates_its_units` 在 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:1164`，断言「violations.is_empty(),」在 1178 行；慢档（`#[ignore]`）。
- 现跑：变异行 3284 `ROW 37 rc=101 RED(expected test failed)`；默认档 `reuse_after_raising_the_floor` 3 passed、13 ignored。慢档单条在窄版 A1/A2/A3 下都 GREEN（见 X2 的观察，同一原因）。
- 判定：**一致**。
- 推翻：同 X2。

## 4 K1 逐格（下）：Z1、Z2、A1–A4、B1、C1

### Z1　GPU 恢复内核的 F2 与 CPU 逐支同判；实例表沿链读的上限

读码（GPU `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`，CPU 同 Y4 表）逐支：

| 支 | CPU | GPU | 同？ |
|---|---|---|---|
| 所选 ≠ 择到的 ⇒ 不退 | `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5440` | 「if (selected.instance != chosen.instance || !eq64(selected.txg, chosen.txg)) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:143`） | 同 |
| 见证判不出 ⇒ 不退 | `system_configuration_witness_is_undecidable`（有槽读错且没有一块盘两槽都自证过） | 「if (some_slot_reports_a_read_error && !some_device_has_both_slots_self_verified) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:177`） | 同（读错只建模槽距未对齐，注释论证盘内，前一轮已采） |
| c_见证 = 取各盘自证槽 journal tail 最大；0 不退 | `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5474` | `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:208` 的 `is_zero64(witnessed_counter)` | 同 |
| 见证过判法三支 | `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5500` 一带 `match` | 「is_witnessed = le64(last_record_counter, witnessed_counter);」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:204`）、「is_witnessed = !witnessed_record_is_older_than_the_selected_version;」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:206`）、缺省 `true` | 同 |
| 见证记录读不出 ⇒ 不退 | `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5506` | 同一个 `if` 里的 `!has_witnessed_record`（208 行） | 同 |
| 区间下界含见证那一版、上界不含所选根 | `witnessed_publish <= order && order < chosen` | `!precedes_in_the_root_choice_order(root.txg, root.instance, witnessed_record_txg, witnessed_record_instance)` 与 `precedes_…(root…, chosen…)`（`is_in_the_range`） | 同 |
| 实例表读得出：按表滤；读不出：只认恰好相等 | `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5524`–5525 | 「is_admissible = !abandoned_by_rows(root.instance, root.txg);」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:222`）、「var is_admissible = is_the_witnessed_version;」（220 行）；`abandoned_by_rows` = 行实例相同且 `lt64(row_txg, txg)`，与 `root_is_abandoned_by_the_instance_table` 同 | 同 |
| 取最新、并列取最后 | `max_by_key` | 「&& (!out.found || !precedes_in_the_root_choice_order(root.txg, root.instance, out.root.txg, out.root.instance))) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:227`）：新 ≥ 旧就换 | 同 |

- 实例表沿链读：`load_rows_of_the_instance_table_chain`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:96`）：一片读不出 / 解不开 ⇒ 表不可读（`false`）；行数到 `MAX_ROWS`（`crash_judge_common.wgsl` 里 512u）还要放行 ⇒ 「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES;」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:112`）；链长走满 256 片 ⇒ 「undecidable = UNDECIDABLE_TOO_DEEP;」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:128`）。这两个 `undecidable` 记号不是新立的：`crash_judge_common.wgsl` 里别的限额早已在用同名常量（如 1815、1979、2012、2275 行），状态被标判不了后落回 CPU；CPU 不设这两个上限。
- 我的差分模型（`f2_differential.py`，两份各自转写、读码转写不是调原代码）：随机 400000 个小状态（实例 0–2、txg 1–5、至多 7 条根、至多 6 条记录、1–2 块盘各两槽、实例表读不出 / 0–3 行），`MISMATCH 0`、`FALLBACK-FOUND 28288`；对 GPU 转写依次打进「去掉 c_见证 = 0 守卫」「忽略实例表」「下界改成严格大于」「并列取最先」四个弄坏开关，四个都 `MISMATCH>0`（自证会红）。**限度**：转写是我读码得来的，与我的读法共用前提；行数超 512 / 链超 256 那一支模型里没造（`GPU-UNDECIDABLE-SKIPPED 0`），只读码核。
- 判定：**一致**。
- 推翻：真 GPU 对拍文件（我不跑）在某条流上红；或差分模型换种子出现 `MISMATCH>0`。

### Z2　建模差两处对齐；恢复内核钉值

- 建模差两处（前一轮判决 K1 V1 行）：①读错：GPU 只在槽 1 的偏移不是 512 的倍数时记读错（`SECTOR_BYTES_OF_THE_MEMORY_POOL`，`crash_judge_recovery.wgsl` 里 `(offset % SECTOR_BYTES_OF_THE_MEMORY_POOL) != 0u`），对照 `memory_pool.rs` 的 `check_request_against_the_device_contract`：偏移或长度不是物理块的倍数 `Unaligned`、越盘尾 `OutOfRange`；越盘尾那一支注释论证走不到（读者收下的槽距 ≤ 根环基址 − 4096，两槽恒在盘内，`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:156` 起一整段）。②c_见证 = 0 守卫：与 CPU 同形（Y4 / A1）。
- 钉值：A2 把恢复内核钉值 328934 → 328970：「("recovery", recovery_kernel_source(), 328_970, 71_030),」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:1348`），余量 71 030 = `LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:246`，400_000）− 328 970；算术对。**钉值本身（naga 展开后的表达式数）是 checker 档的 `each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget` 量的，我复核不了**：要主 agent 在 checker 档跑那一条，红的话看实际数。
- 判定：建模差两处 **一致**；钉值 **复核不了**（算术对，数没量）。
- 推翻：checker 档那一条红，或量出来的数不是 328 970。

### A1　c_见证 = 0 不退（CPU、GPU 两边加守卫）

- 应该是：D23「那条记录读不出、c_见证 = 0、那一段里没有读得出又不被抛弃的根都不退」（`.claude/kb/decisions/23-journal的角色与格式.md:396`）；records 第 1264 行：CPU 与 GPU 都加守卫。
- 今天是：CPU `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5474`（函数里在算出 c_见证 之后、判见证过之前）；GPU `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:208`；测试 `a_witnessed_counter_of_zero_does_not_fall_back_even_with_a_record_carrying_counter_zero_in_recovery_and_both_mounts`（harness，恢复、只读挂载、可写挂载三处）与 GPU 对拍文件里 `the_gpu_judge_agrees_with_the_cpu_that_a_witnessed_counter_of_zero_does_not_fall_back_even_to_the_version_of_a_record_carrying_counter_zero`（checker 档，不跑）。
- 现跑：基线 `recovery_and_both_mounts_fall_back_…` 15 passed 含这一条；`ROW 41`（`if false {`）`RED`。GPU 一侧变异 `ROW 42` 是 checker 档，`SKIPPED`。
- 判定：**一致**。
- 推翻：伪造的计数器 0 记录上 CPU 或 GPU 任一处退了。

### A3　F2 函数改名

- 应该是：records 第 1266 行 ③：名字不准了（交回的是区间里最新那条），改名动 `mounted_read.rs`、`mount_transparency.rs`。
- 今天是：新名 `newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to` 在 `/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5429`（定义）、`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/recovery.rs:5619`、`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mounted_read.rs:626`、`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:6788` 起的调用、`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:229` 起的调用、wgsl 58 / 139 行。旧名在 TREE 的 `crates` 下一处不剩：
  ```
  $ grep -rn "version_witnessed_by_the_system_configuration_to_fall_back_to" crates | wc -l
  0
  ```
  harness 与 core 编得过（基线编译退 0）。
- 判定：**一致**。
- 推翻：别处（kb、research）有按旧名的引用而门禁没拦——那不在这一轮范围，我没扫 kb。

### A4　`F1_AFTER_F2`「重读之后故障撤掉」那一形的用例

- 应该是：records 第 1266 行 ④：补用例，钉重读之前的钩子撤故障后不拒。
- 今天是：`a_root_falling_back_with_its_first_record_and_tree_table_reading_back_zero_mounts_writable_when_the_reread_clears_the_fault`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/tests/writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault.rs`，76 行一条用例）：R 的第 1 条记录与树表读回全零，钩子 `lift()` 撤故障；断言可写挂载做成、钩子只调一次、读阶段 `OnTheFirstRead`（那一次重读是 `F1_AFTER_F2` 的）、落在 P。
- 现跑：1 passed；`ROW 43`（去掉重读前的 `before_the_one_reread.before_rereading();`）`RED`。
- 判定：**一致**。
- 推翻：钩子不撤故障时（同盘面、`BeforeTheOneReread::RereadImmediately`）这条盘面可写挂载做成了——那说明 F1 没判。（该盘面在 `writable_mount_rejudges_missing_records_…` 第一条里钉了拒，基线 3 passed。）

### B1　Z 那条「所选根的实例表抛弃 V」对拍流的新造法

- 应该是：只让 R 认的那份实例表多一行 (V⁻ 的实例, V⁻ 的 txg)，V⁻、V 的根记录与原来那张表一个字节都不动。
- 今天是（读码，checker 档不跑）：`point_the_chosen_root_alone_at_an_instance_table_abandoning_v`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，11862 行起）：从基镜像读出 R 指着的那张共享表（先断言流里没有对它的写、V 与 R 的根记录指着同一张表、CRC 与指针里的相同）→ 拷一份、在最前面插一行、重封 → 写进基镜像里流从没写过的槽（先 `assert_nothing_in_the_flow_wrote_the_slot`）两块盘 → 只重写 R 的根记录里实例表指针（`rewrite_root_record_and_reseal(flow, root_of_r, …)`）与 R 那次发布每条 journal 记录新根段里的实例表指针（`rewrite_journal_record_and_reseal`，记录先断言原来的指针等于共享表的）。V⁻、V 的根记录没有任何重写调用；原来那张表所在的槽没写。
- 判定：**一致**（读码）。真 GPU / CPU 对拍红不红复核不了（要主 agent 上卡）。
- 推翻：对拍文件里这条在 V 的根上也看到被抛弃（说明改动漏到 V），或 R 的记录新根段与根记录指着不同的表。

### X4　`read_at` 注释

- 应该是：根环线建议：说清拼读切片后 16 KiB 没有字节。
- 今天是：只加了两行注释，没动代码（diff 里 `crash_judge_common.wgsl` 一个 hunk、只有 `+//` 行）：「拼读切片后 16 KiB 没有字节（base 指的是全零 16 KiB 那一条版本字节，再往后不是下一个落点的字节），调用方只能比 CRC 或读前 16 KiB」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:465`）。对 `read_at`（同文件 469 行起）的拼读分支：`version` 取 `ZERO_UNIT_VERSION_OF_SIXTEEN_KIBIBYTES`、`s.length = DATA_UNIT_BYTES`、`s.c1` 放拼出的整段 CRC、`s.c2` 带 `SLICE_SPLICED_ACROSS_THE_END_OF_A_LOCATION`：`base` 指的字节确实只有 16 KiB。注释说得对。
- 判定：**一致**。
- 推翻：有调用方对拼读切片读了第 16 KiB 之后的字节（注释说「只能比 CRC 或读前 16 KiB」，我没逐个扫调用方）。

### X5　`allocator.rs` 里那句 `protected.extend` 的用例

- 应该是：判决第二节 K1 / K2 W1 行：那一句没有用例也没有变异行，补。
- 今天是：新用例 `a_slot_of_the_current_account_named_by_an_unapplied_record_stays_isolated_after_its_reclaim_until_the_root_ring_slot_of_that_record_is_overwritten`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/tests/writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten.rs:97`）；变异行 3259 删掉「protected.extend(self.placements_named_by_unapplied_journal_records());」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/allocator.rs:1150`）。
- 现跑：用例 1 passed；`ROW 12 rc=101 RED(expected test failed) nfailed=1`。
- 判定：**一致**。
- 推翻：那一句删掉后这条用例过。

### C1　`C_MAPPING` 那一拒另起成员；镜像复算

**成员**：`StillUnreadableAfterOneReread::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { unapplied_publish: RollbackTarget }`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-core/src/mount.rs:403`），`placements_of_an_unapplied_publish_whose_account_is_unreadable` 用它、`placements_of_an_abandoned_root_whose_account_is_unreadable` 仍用 `AccountOfAnAbandonedRoot`（C393）：两拒共用同一个 `..._unless_still_unreadable_after_one_reread` 判法（`mount.rs:1682`）、各报各的成员。`history.rs`、`model_comparison.rs`（两个 `match`：`refusal_reason_of_mount_error` 映 `Unexplained`，`root_ring_slot_still_bad_after_one_reread_of_rollback_error` 映 `None`）、`mount_transparency.rs`（`refusal_reason_on_the_d23_list` 新一臂，见 X3）都补了臂；七个 `StillUnreadableAfterOneReread` 成员无通配。

**D23 单子**：`RefusalReasonOnTheD23List::AccountOfAnUnappliedPublishWithIncompleteReadableRecords { unapplied_publish: (InstanceGeneration, CheckpointTxg) }`；读故障下取法：「取不放宽的一边」，与 F1 那一拒同列（`judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount` 里那一格：复算为假照判不合法）。

**镜像复算逐条核正文写的「三样」**（函数 `account_of_an_unapplied_publish_with_incomplete_readable_records_on_the_crash_image`，`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2717`）：

| 正文三样 | 代码 | 一致？ |
|---|---|---|
| ① 点名的是水位之上读得出记录的发布，或根环里比所选那一版新、按最新那条根的实例表有效、读得出的根 | 「unapplied_publish > selected_version && !readable_records_of_the_publish.is_empty();」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2734`），元组 (实例, txg) 实例为主，与实现 `C_ACCOUNT` 筛记录同序；较新根：`newer_valid_readable_root_on_the_crash_image`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2835`）：同 (实例, txg)、「&& (root.checkpoint_txg, root.instance) > (selected_version.1, selected_version.0)」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2858`，txg 为主，与实现 `C_NEWER_ROOTS` 同序）、被抛弃按「row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2854`，自己写了一份，没调 `root_is_abandoned_by_the_instance_table`；最新那条根的表读不出时不滤）；两样都不是就「if !is_an_unapplied_publish_above_the_water_mark && newer_valid_readable_root.is_none() {」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2741`）交回假 | 是 |
| ② 读得出的记录不齐（序号从 1 起逐个加一不成立，或没有带末条标志的；一条都读不出算没有末条） | 「if ordinals_run_from_one_without_a_gap && carries_the_last_record_of_the_publish {」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2757`）后 `return false;`；空集时前一半为真、后一半为假 ⇒ 不返假 ⇒ 算不齐 | 是 |
| ③ 账读不出：顺树表指针找到的账根两份都验不过 | 对读得出的每条记录的 (`new_tree_table`, `new_allocation_record_tree_root`) 与较新根的 (`tree_table`, `allocation_record_tree_root`) 取 `.all(account_unreadable_on_the_crash_image)`；`account_unreadable_on_the_crash_image`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2792`）：树表两份都验不过 ⇒ 读不到账根 ⇒ 算读不出；树表 0 条 ⇒ 账根是新根段里那条分配记录树根；有条目 ⇒ `TREE_KIND_ALLOCATION` 那条的根指针（没有那一条算读得出，「None => return false,」`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2825`）；「account_root != NodePointer::empty_root()」（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2828`）且 `!node_has_a_copy_that_verifies_on_the_crash_image` 才算读不出 | 是 |
| （不复算）映射现算那一半 | 函数文档写了「这一格只判必要条件」；代码里确实没有读中央映射树的调用 | 与正文同 |

**有没有调实现里判拒的函数**：复算整条链（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs` 的 2627–2870 行）只调 `choose_system_configuration`、`scan_journal`、`choose_root`、`instance_table_of_root`、`readable_roots`、`parse_index_node`、`crc32_castagnoli`、`NodePointer::read_from` 与文件里自己的 `TreeTableEntryRootPointersByKind::of_the_entries`、`node_has_a_copy_that_verifies_on_the_crash_image`；没有 `placements_in_the_accounts_of_*`、`placements_referenced_by_root`、`readable_records_of_the_publish_leave_a_gap_or_miss_the_last`、`publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify`。命令与原样输出：
```
$ grep -n "placements_in_the_accounts_of\|isolate_slots_referenced_only_by_abandoned_roots\|publishes_above_the_water_mark_missing_records_whose_named_units_do_not_verify\|placements_referenced_by_root\|placements_of_an_unapplied_publish_whose_account_is_unreadable\|readable_records_of_the_publish_leave_a_gap" crates/singlefs-harness/src/mount_transparency.rs | grep -v "^[0-9]*:\s*///"
（零行；第二个 grep 去掉文档注释行，实际代码里零命中）
```
对 `mount` 模块只 `use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread}`（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:27`），四个都是类型。

- 现跑：3 条用例（`..._judged_on_the_d23_list`）1 条拒因成员、1 条合法、1 条非法（三条记录都在的镜像上拒判成非法，`conditions_not_met == [CriterionFalseWhenRecomputedOnTheCrashImage]`）；3 passed。变异 `ROW 45`（成员照旧 C393）、`ROW 46`（映成不在单子上）、`ROW 47`（复算不看记录齐不齐）全 `RED`；`ROW 45` 红的两条是「自己的成员」那条与「合法」那条；`ROW 46` 红的两条是「非法」那条与「合法」那条（`mutations.tsv` 第六段只点名一条，另一条是同一处改动连带红的，不是点名不准）。
- 偏离与限度（读码，照实列，不当冲突）：
  1. 账根按「新根段里的树表指针找到的账根」读，不是主 agent 先写的「新根段里的分配记录树根」：带文件的一版新根段里那一条恒是空指针，按字面永远判不成读不出；实现员已在函数文档里写明并交主 agent 定（第 43 批报告），正文第一节 C1 行的三样已是树表指针这一读法。
  2. 账只看账根一个节点：账根以下的节点读不出时实现照拒、复算判读得出 ⇒ 多报红一侧（正文「已知」已列）。
  3. 树表有条目但没有 `TREE_KIND_ALLOCATION` 那一条时复算判读得出（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2825`）；实现读这种树表时是否 `None` 我没造盘面核——合法写者每一版都写分配记录树条目，没造出这样的盘面（推的）。
  4. **较新可读根那一臂（①后半、③的 `newer_valid_readable_root`）没有任何 harness 用例**：3 条用例都是「水位之上没施加的发布」那一臂；变异 `ROW 47` 也只钉「记录齐」。较新根那一臂只由读码核（上表）；崩溃注入第三截（checker 档）会在随机状态上碰到它，我跑不了。
- 判定：**一致**（三样都照；没调实现判拒函数）；限度 2、3、4 如上。
- 推翻：一个合法的 C_MAPPING 拒（水位之上有记录的发布、记录不齐、树表读不出、映射现算不成）被复算判假；或复算为真而实现不拒时被当成多余（复算只判必要条件，这不算）；或第 4 条那一臂在崩溃注入里出现红。

## 5 K1 变异行（新行与改过锚点的行）

范围：变异表里这一轮的 48 行 = 37 条新行（第 37 批 5、第 38 批 22、第 39 批 3、第 41 批 3、第 42 批 1、第 43 批 3；其中第 38 批的 C_MAPPING 2 行被第 43 批换了替换文）加 11 条被改过锚点或替换文的老行（第 3075、3116、3117 行 3 条（X）；第 33 批 T 5 条与第 35 批 F2_EXACT 1 条（Y 替换）；第 36 批 V 1 条（Z 替换）；LR4 1 条（第 43 批替换））。`static.txt`：`ROWS 3306 ANCHOR-BAD 0`、`NAMES-UNIQUE True`，被删的两行（第 35 批 F2_EXACT 退回取法、第 39 批 Z c_见证 = 0 守卫那一行）都不在了。

| 档 | 行数 | 现跑结果 |
|---|---|---|
| harness 档 | 39 | 39 行全 `RED(expected test failed)`：点名的测试在红的里面；`nfailed=1` 的 37 行，`nfailed=2` 的 2 行（`ROW 45`、`ROW 46`，见 C1）；没有一行 `GREEN`、`COMPILE-ERROR`、`ANCHOR-HITS` |
| checker 档 | 9 | `SKIPPED`：第 3075、3116、3117 行（X）、第 36 批 V 1 行、第 39 批 Z 3 行、第 41 批 GPU 1 行、第 42 批 F2_NEWEST 1 行（它改的是 CPU 的 `recovery.rs`、点名的是 GPU 对拍文件，仍属 checker 档）。锚点各命中恰好一次；红不红归主 agent 经门禁 59 号 / checker 档跑 |

输出原样在 `research/prompts/m3-batch37-43-code-r1-sonnet-model/expected/mutations.txt`（48 行 `ROW`）；两次从头跑（`rerun-out`、`rerun-out2`）`compare.sh` 全同。

逐条看替换文与名字：

1. **名字与替换文对得上**：除下面 2、3 两点外，harness 档每一行的替换文就是名字说的那一处（第 37 批 F1 的三行、`C_NEWER_ROOTS` 四行 `.take(0)`、`C_MAPPING` 七行、`F1_AFTER_F2` 三行、`F2_NEWEST` 七行、第 43 批三行、c_见证 = 0 守卫一行、LR4、第 33 批 T 五行与第 35 批 F2_EXACT 一行）；点名的测试名与该行第五段的参数一致（`mutrun.py` 的判法是「第六段的测试名在红的里面」，39 行全部成立）。
2. **名字比替换文窄**：第 3258、3284 行（见 X2），替换文清空整个 C。
3. **成员遗留**：第 3273 行替换文用 C393 的 `AccountOfAnAbandonedRoot` 拒，第 43 批之后 C_MAPPING 的拒是另一个成员；这行模拟的是「也拒」，红不受影响，名字也没说成员。
4. **同一替换文挂两条**：第 3282 行（CPU 侧 `recovery_and_both_mounts…` 里那条）与第 3308 行（GPU 对拍侧）原文、替换文逐字相同，各点名各自的测试；表里名字不同（`NAMES-UNIQUE True`）。这是有意的：GPU 侧那条要上卡才能证红。
5. **第 3163 行（Y5）**：单独施加就红（`ROW 4`），不再依赖别的变异。
6. 第 3075、3116、3117 行（X1）：锚点命中恰好一次；替换文的意思我读了：3116 的替换文把「slots_left_in_the_gap < slots_read」换成 32 位加法比较（模拟旧写法回绕），3117 把 32 KiB 另读换成拿 16 KiB 切片去比，3075 把 `select(0u, 1u, !whole_data_unit_fits)` 换成 `NONE`；都是原名字说的那个旧写法。红不红没跑。

**窄版变异（我加的，`adhoc-cases.tsv`）**：A1 去掉并之前「记录点名的」那一份、A2 去掉 `C_ACCOUNT` 并进来的账、A3 去掉 `C_NEWER_ROOTS` 并进来的账。结果（`expected/adhoc.txt`）：对 `random_histories` 与 `reuse_after_raising_the_floor` 两条双故障慢档用例，三个窄版都 GREEN；对 `writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero`（A1 第 2 组）RED、对 `writable_mount_isolates_newer_readable_roots…`（A1 第 3 组）GREEN。A1 在 `..._newer_readable_roots_…` 那份 8 条用例上不红，是因为那份用例钉的是 `C_NEWER_ROOTS` 与 `C_MAPPING`，不是「记录点名的」那一份；记录点名那一份由 Y5 的用例钉（A1 第 2 组 RED）。

- 推翻：在 harness 档任一行把替换文打回副本后点名的测试不红；或者两次从头跑 compare 不同。

## 6 K3　23 份文件，每份一格

sha256：23 份全 OK（第 0 节）。列：文件 / 这一轮对应的格 / 做了什么 / 判定。「读码」= 没编译没跑（checker 档），其余 = 读码 + 现跑（基线 + 变异行）。

| # | 文件（TREE 下） | 格 | 做了什么 | 判定 |
|---|---|---|---|---|
| 1 | `crates/singlefs-checker-tier/src/crash_injection.rs` | X3、C1 | 第三截补 F1、C_MAPPING 两个成员的 `Legal` / `Illegal` 臂与两格计数（`/home/fy5090/.local/share/singlefs/work/m3-batch37-43-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs` 3753 行起的 `match` 无通配）；`CriterionFalse` 之类的 `Illegal` 不在这一函数里报红，只记一次判过（函数文档同句） | 读码一致；未跑 |
| 2 | `crates/singlefs-checker-tier/src/crash_judge_common.wgsl` | X4 | 只加两行注释，对 `read_at` 拼读分支说得对（X4） | 一致 |
| 3 | `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs` | A2 | 钉值 328_970 / 余量 71_030，算术对；注释记了 328 934 → 328 970 | 钉值复核不了（要 naga 量，checker 档） |
| 4 | `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl` | Z1、Z2、A1 | F2_NEWEST 逐支与 CPU 同（第 4 节表）；实例表沿链读、超限记判不了；c_见证 = 0 守卫 | 读码一致；差分模型 `MISMATCH 0`；未上卡 |
| 5 | `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | Z、A1、B1 | Z 的 4 条流（第 4 条是 c_见证 = 0 那条，第 41 批改了名字和期望）、B1 新造法 `point_the_chosen_root_alone_at_an_instance_table_abandoning_v`（11862 行起）；B1 只动 R 的根记录与 R 的记录新根段，V⁻、V 不动 | 读码一致；红不红要上卡 |
| 6 | `crates/singlefs-core/src/mounted_read.rs` | A3、Y4 | 调用点改新名（626 行），传 `&chosen_root`、`&effective_root` | 一致（core lib 200 passed；`recovery_and_both_mounts…` 15 passed） |
| 7 | `crates/singlefs-core/src/mount.rs` | Y1–Y3、C1、A3 | `C_NEWER_ROOTS`、`C_MAPPING` 收窄、`F1_AFTER_F2`、新错误成员 | 一致；打在它上面的 24 条变异行（ROW 0、4–8、11、16–29、37、43、45）全红 |
| 8 | `crates/singlefs-core/src/recovery.rs` | Y4、A1、A3 | `F2_NEWEST` 与 c_见证 = 0 守卫、改名 | 一致；打在它上面的 10 条变异行里 9 条（ROW 9、30–36、41）红，ROW 44 是 checker 档不跑 |
| 9 | `crates/singlefs-harness/src/history.rs` | C1 | `still_unreadable_after_one_reread_member` 加一臂（3106 行） | 一致（编译过；成员无通配） |
| 10 | `crates/singlefs-harness/src/model_comparison.rs` | C1 | 两个 `match` 各加一臂（842、952 行），映 `Unexplained` / `None` | 一致（编译过） |
| 11 | `crates/singlefs-harness/src/mount_transparency.rs` | X3、C1、A3 | 两个新成员、两条镜像复算、改名 | 一致；打在它上面的 5 条变异行（ROW 13–15、46、47）全红 |
| 12 | `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs` | X3、C1、Y 各测试的底 | 新增 `RecordedPublish::root / unit_writes_at / unit_writes_grouped_by_unit`、`root_is_in_the_ring`、`mount_writable_and_assert_the_units_of_r_stay_unwritten_through_three_user_actions` 等；11 个目标共用 | 一致：11 个用到它的目标全 ok（7 个在快照里、4 个不在：`abandoned_root_whose_account_…` 2、`read_recovery_fields_follow_…` 4、`writable_mount_isolates_the_units_named_by_unapplied_journal_records_…` 5、`writable_mount_refuses_a_publish_above_the_water_mark_…` 6 条，我另跑了这 4 个） |
| 13 | `crates/singlefs-harness/tests/random_histories.rs` | X2 | 慢档用例改名、期望改空 | 一致；默认档 23 passed、19 ignored；`ROW 11` 红 |
| 14 | `crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` | Y4、A1 | F2_NEWEST 的用例与 c_见证 = 0 | 一致；15 passed；`ROW 9、30–36、41` 红 |
| 15 | `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` | Y6 | 同形慢档用例改名改期望 | 一致；3 passed、13 ignored；`ROW 37` 红 |
| 16 | `crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs` | Y1、Y2 | `C_NEWER_ROOTS`、`C_MAPPING` 的 8 条 | 一致；8 passed；`ROW 16–26` 红 |
| 17 | `crates/singlefs-harness/tests/writable_mount_isolates_the_account_of_an_unapplied_publish_whose_record_reads_back_zero.rs` | Y5 | 第 3163 行的新取样点（140 行起） | 一致；4 passed；`ROW 4` 红 |
| 18 | `crates/singlefs-harness/tests/writable_mount_keeps_a_released_slot_named_by_an_unapplied_record_isolated_until_its_root_ring_slot_is_overwritten.rs` | X5 | `protected.extend` 的用例 | 一致；1 passed；`ROW 12` 红 |
| 19 | `crates/singlefs-harness/tests/writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list.rs` | C1 | 新成员、合法、非法三条 | 一致；3 passed；`ROW 45–47` 红 |
| 20 | `crates/singlefs-harness/tests/writable_mount_refusal_for_a_publish_missing_records_recomputed_on_the_crash_image.rs` | X3 | F1 的拒归单子、复算三条 | 一致；3 passed；`ROW 13–15` 红 |
| 21 | `crates/singlefs-harness/tests/writable_mount_rejudges_f1_after_falling_back_and_accepts_when_the_reread_clears_the_fault.rs` | A4 | 重读之后故障撤掉那一形 | 一致；1 passed；`ROW 43` 红 |
| 22 | `crates/singlefs-harness/tests/writable_mount_rejudges_missing_records_after_falling_back_to_the_witnessed_version.rs` | Y3 | `F1_AFTER_F2` 的用例 | 一致；3 passed；`ROW 27–29` 红 |
| 23 | `crates/mutations.tsv` | X1、Y5、新行 | 全表 3306 行、48 行这一轮的行 | 一致；锚点全命中一次；harness 档 39 行全红 |

（第 7、8、11 行的变异行数是数出来的：把 `rows.txt` 的 48 行按变异表第二段的文件分，`mount.rs` 24 行全红、`recovery.rs` 10 行 9 红 1 跳过、`mount_transparency.rs` 5 行全红、`allocator.rs` 1 行红；`crash_judge_common.wgsl` 2、`crash_judge_checker.wgsl` 1、`crash_judge_recovery.wgsl` 5 共 8 行跳过，加上 `recovery.rs` 的 ROW 44，一共 9 行跳过。）

## 7 prove-red.sh 复证与一次意外

- 用 `research/scripts/prove-red.sh --copy <仓副本> --memory 6G singlefs-harness <变异名…>` 把 harness 档 39 行再证一遍（与 `mutrun.py` 是两个驱动）：三段，`✓ 点名 11 条：跑了 11 条，跳过 0 条，跑的都抓到了`、`✓ 点名 15 条：跑了 15 条，跳过 0 条，跑的都抓到了`、`✓ 点名 13 条：跑了 13 条，跳过 0 条，跑的都抓到了`，合计 39 条全抓到（原样在 `research/prompts/m3-batch37-43-code-r1-sonnet-model/prove-red-run/chunk-*.txt`）。
- 意外（照实写）：第一次我把 20 个名字放进一段，前台 `timeout 235` 把它杀了；其中第 3259 行改的是 `crates/singlefs-core/src/allocator.rs`（不在 23 份快照里），被杀时还没还原，我的工作副本里 `allocator.rs` 缺了 `protected.extend(...)` 一行。我是靠随后同一个用例的「基线就红」发现的（`diff -rq` 对 TREE 只列出这一个文件），把 TREE 里的原文件拷回、`touch` 之后 `diff -rq crates <TREE>/crates` 无输出。影响面：这之前跑的 `rerun-out`、`rerun-out2` 两个空目录各用各自的副本（事后 `diff -rq` 都与 TREE 相同），基线与变异结果都出在它们里面；工作副本里在意外之前跑的基线（各 harness 目标）是在干净副本上跑的，意外之后没有再用工作副本的结果当依据。TREE 本身没动（23 份 sha256 全 OK，事后复核一次 `sha256sum -c` 无不 OK）。

## 8 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X1 | 一致 | 全表 3306 行锚点各命中恰好一次，三行都在 checker 档，红不红没跑 |
| X2 | 一致 | 慢档用例期望改成 `violations.is_empty()`；变异 3258 红；名字读成「只有记录点名的那一份」会读窄（替换文清空整个 C） |
| X3 | 一致 | F1 的拒归单子，七个 `StillUnreadableAfterOneReread` 成员逐个对过无通配；镜像复算不调实现判拒函数；崩溃注入两臂读码核，没跑 |
| X4 | 一致 | 只加注释，说对了拼读切片后 16 KiB 没有字节 |
| X5 | 一致 | 新用例单独钉 `protected.extend`，变异 3259 红 |
| Y1 | 一致 | 比所选那一版新、按实例表有效、读得出，三个条件都在；记录一条都读不出 / 不齐 / 齐三分与 D23 对上 |
| Y2 | 一致 | 现算 → 重读 → 凭据 → 拒，被抛弃根与 C 共用一份、各报各的成员；记录齐不走；一条记录都读不出不拒 |
| Y3 | 一致 | 退到的那一版当水位再判一次，重读前调钩子；读故障下不重扫环（规则没说） |
| Y4 | 一致 | D23 :396 逐句对上代码；三处调同一个函数；规则没说的两处沿旧例 |
| Y5 | 一致 | 第 3163 行单独就红 |
| Y6 | 一致 | 同 X2 |
| Z1 | 一致 | GPU 与 CPU 逐支同判（读码 + 差分模型 400000 态 0 不一致，四个弄坏开关都红）；512 行 / 256 片记判不了回落 CPU，读码核，模型没造 |
| Z2 | 建模差一致；钉值复核不了 | 328970 / 71030 算术对，naga 展开数要 checker 档量 |
| A1 | 一致 | CPU 5474 行、GPU 208 行同守卫；harness 用例与变异 ROW 41 红 |
| A2 | 复核不了 | 同 Z2 钉值 |
| A3 | 一致 | 旧名在 crates 下零命中，编得过 |
| A4 | 一致 | 重读前钩子撤故障的用例；变异 ROW 43 红 |
| B1 | 一致（读码） | 只改 R 的根记录与 R 记录新根段里的实例表指针，V⁻、V 不动；对拍红不红要上卡 |
| C1 | 一致 | 三样都照、没调实现判拒函数；限度：账根以下不复算（已知）、较新根那一臂没有 harness 用例、树表有条目无分配树条目的一形没造盘面 |
| K1 变异行 | 一致 | harness 档 39 行全红（两个驱动各一遍）；checker 档 9 行锚点命中一次、没跑 |
| K3 | 23 份：harness 与 core 的 17 份（src 6、测试 11）读码 + 现跑一致；checker 档 5 份读码一致、未跑（其中钉值复核不了）；`mutations.tsv` 1 份一致 | 逐份见第 6 节 |

没有一格判「冲突」。小项（不改判定，请主 agent 看一眼）：①X2 / Y6 的慢档用例名字窄于替换文；②第 3273 行替换文用 C393 的成员；③C1 较新根那一臂无 harness 用例；④F1_AFTER_F2 在读故障下不重扫环（D23 没写）。

## 9 没做什么

- checker 档与 GPU 一律没跑：对拍文件的 Z / A1 / B1 流、`crash_injection.rs` 第三截的两臂与计数、恢复内核钉值 328970、9 条 checker 档变异行（锚点各命中一次，红不红没证）。GPU 一侧只读 wgsl、用 CPU 判法比。
- 没有替主 agent 采纳或出判决；X3 那一拒该不该在读故障下放宽、C1 账根读法、较新根那一臂要不要补用例，都只列事实。
- 没扫 kb / research 里按旧函数名的引用（A3 的范围只到 `crates/`）。
- 没有造「树表有条目但没有分配记录树条目」那一形的盘面（C1 限度 3）。
- 没有对每个 `read_at` 调用方核「拼读切片只读前 16 KiB」（X4）。
- 差分模型是我读码转写的两份，与我的读法共用前提；没有拿真 GPU 核。
- 没有重新核实现员报告里的数（只当线索），没读禁读清单里的文件。

## 10 要主 agent 跑的命令

（我不跑 checker 档；命令照 `crates/mutations.tsv` 第五段与 `crates/singlefs-checker-tier` 的测试名，工作目录仓根，`SINGLEFS_HEAVY_TESTS=user-request`，内存上限与线程数按你的现场定。）

1. 恢复内核钉值：`cargo test -p singlefs-checker-tier --lib each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget`，看 `recovery` 一项是不是 `328_970`、余量 `71_030`。
2. GPU 对拍文件（带 `--nocapture`，看 `CRASH_JUDGE_GPU_MISMATCH` 行数与「本次未跑」条数）：`cargo test -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --nocapture`；重点看第 11809、11980、11999、12110 行起的四条流和 B1 的 `the_gpu_judge_agrees_with_the_cpu_that_a_recovery_does_not_fall_back_to_a_version_the_instance_table_of_the_chosen_root_abandons`。
3. checker 档变异 9 行：名字前缀在 `research/prompts/m3-batch37-43-code-r1-sonnet-model/rows.txt`，序号是 `expected/mutations.txt` 里 `SKIPPED` 的 `ROW 1、2、3、10、38、39、40、42、44`；经门禁 checker-tier-crates-mutation-replay 或 `research/scripts/mutate.sh` 跑。
4. 崩溃注入第三截（`crash_injection.rs` 的两臂与两格计数）：随你提交时的 checker 档整轮。

复跑我的模型：`SINGLEFS_REPO=<冻结副本根> SINGLEFS_MAIN_REPO=<主仓根> bash research/prompts/m3-batch37-43-code-r1-sonnet-model/rerun.sh <空目录> static f2diff`，再按模型目录 `README.md` 的分段顺序接着跑，最后 `bash …/compare.sh <那个目录> …/expected` 应见五个「全同」。
