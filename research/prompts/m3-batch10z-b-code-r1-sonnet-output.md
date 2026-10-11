# m3-batch10z-b-code-r1 正推腿报告（Sonnet）

日期 2026-10-08。轮名 m3-batch10z-b-code-r1，立场：正推。分到的格：正文第三节 K1（1、2、3）、K2（1、2、3）、K3。

## 约定与量法（先读这一节）

- 代码行号一律按冻结副本 `~/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/`（被判文件 sha256 与 `research/prompts/m3-batch10z-b-code-r1-snapshot/crates-sha256.txt` 的 mount.rs 一行 `667a3374…` 对得上）。主工作区的 `crates/singlefs-core/src/mount.rs` 在我开工之后被别的会话改过（sha256 `7bf5f538…`），行号与冻结副本不同；allocator.rs、新用例文件、e158 装置主工作区与冻结副本逐字节相同。写 `crates/…` 路径的引文，核对时要加 `--root` 指到冻结副本。
- kb 条款写 kb 文件名加那份文件自己的行号（现查）：`16-发布语义.md:37`、`23-journal的角色与格式.md:390`、`28-挂载期承诺量.md:27`。
- 我的副本：`~/.local/share/singlefs/work/m3-batch10z-b-code-r1-sonnet/repo`（K2-2 用，带 C 臂开关，只加在副本里）与 `repo-k3`（K3 变异用，每次施加后还原，结束时 `diff -r` 与冻结副本相同）。编译目录 `target`、`target-k3`。入库的量法与脚本在 `research/prompts/m3-batch10z-b-code-r1-sonnet-model/`。
- 编译与测试都经 `run-with-memory-cap.sh 12G` 加 `capped.sh`，线程数 8（开工时线程上限是 10，我读背景材料的中途收到主 agent 的消息改成 8，那时我还没起任何编译，所有编译与测试都按 8）。只跑 harness 档（`singlefs-harness` 的 `--test` 目标、`singlefs-core --lib`）；`singlefs-checker-tier` 的测试没跑。池级 checker 在 harness 档用例里作为库调用（用例自己的 `checker_violations_on`），不是 checker 档测试。
- 开工时 `ps` 看到别的会话的一条 `cargo test --offline --no-fail-fast -p singlefs-format -p singlefs-core -p singlefs-harness -p singlefs-checker`（pid 1377308，内存包装 8G、线程 10）一直在跑；没有性能测量在跑。我的编译与测试照常加 `nice -n 19`，cargo 排队拿锁没有等过。

## K1　定案做到没有

条款（整行里的相关句，现查的行号）：「不论住在哪一类槽，一律照挂载时算出、记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒」（`.claude/kb/decisions/16-发布语义.md:37`）；同一行末句「那一道照 B 拒，不变」（`.claude/kb/decisions/16-发布语义.md:37`）；「抬 F 那一串（准入与卸载）不走这一道：被抛弃根照挂载时记在根环表里的落点算影子账，不从盘上重读它的账、读不出也不拒，这一拒只留在再可写挂载那一道」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）；「各条根引用的槽从它的分配记录树读、不加盘上字段；每次挂载与每次抬 F 时算出、只住内存、按设备算」（`.claude/kb/decisions/28-挂载期承诺量.md:27`）。

### K1-1　抬 F 那一串的每个入口

判定：**做到，但有一形漏（表里没记着的四形仍读盘、读不出照 B 拒）**。

依据（读码 + 复跑）：

1. 入口都汇到一处。`raise_rollback_floor` 调 `raise_the_floor_through`：「    let raised = raise_the_floor_through(」（`crates/singlefs-core/src/mount.rs:3208`）；`unmount` 同：「    let raised = raise_the_floor_through(」（`crates/singlefs-core/src/mount.rs:3259`）；`raise_rollback_floor_to_the_admission_ceiling` 下面那一层调 `raise_rollback_floor`：「    raise_rollback_floor(」（`crates/singlefs-core/src/mount.rs:3418`）；写行之后的准入推空间是 `push_floor_raises_after_the_row_publish`（`crates/singlefs-core/src/mount.rs:5937`）走同一族。会话写那一路用新用例第二条真跑过（K3 表里基线绿）。
2. 影子账函数只有两处调用，别的出现都是注释：「        ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(」（`crates/singlefs-core/src/mount.rs:2508`，可写挂载重建分配器，传读盘那一臂）与「        match isolate_slots_referenced_only_by_abandoned_roots(」（`crates/singlefs-core/src/mount.rs:3842`，抬 F 那一串，传「表里记着的，否则读盘」那一臂：「            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(」（`crates/singlefs-core/src/mount.rs:3851`））。命令：`grep -n 'isolate_slots_referenced_only_by_abandoned_roots' crates/singlefs-core/src/mount.rs`，非注释命中就这两处。
3. 漏的一形：循环里先查表、没查到就读盘并按 B 处置：「            .recorded_in_the_root_ring_table(root)」（`crates/singlefs-core/src/mount.rs:1580`）、「            .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))」（`crates/singlefs-core/src/mount.rs:1581`）。读不出就计数并走 B 的函数：「                unreadable += 1;」（`crates/singlefs-core/src/mount.rs:1585`），拒在「    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {」（`crates/singlefs-core/src/mount.rs:1659`）之后。条款字面是「一律」：「不论住在哪一类槽，一律照挂载时算出、记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒」（`.claude/kb/decisions/16-发布语义.md:37`）。四形（实现员报告「停下交主 agent」第 1 条的编号，我都核了代码）：
   - 形 1，可写挂载自己写行之后才判成被抛弃的根（K4-1 的 R）：重建分配器判被抛弃用写行之前的表：「    let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);」（`crates/singlefs-core/src/mount.rs:2405`），R 记成有效根，取落点的方法对它交 `None`：「            RootRingOccupant::ValidRoot { .. }」（`crates/singlefs-core/src/allocator.rs:1135`）。**我在入库装置上造出来量了**（K2-2）：形 1 的读盘臂，挂载之后 R 的账读坏，抬 F 与卸载都拒成 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`。
   - 形 2，挂载那一刻读坏或读回全零的槽，抬 F 那一遍读出被抛弃根：「            | RootRingOccupant::BadAtMount(_)」（`crates/singlefs-core/src/allocator.rs:1136`）、「            | RootRingOccupant::AllZeroAtMount { .. }」（`crates/singlefs-core/src/allocator.rs:1137`）交 `None`。既有用例把这一形钉成「照拒」：`an_abandoned_root_whose_account_slots_only_read_back_zero_still_refuses_the_unmount_before_any_write`（`crates/singlefs-harness/tests/abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs:174`）；K3 变异 m6 证明这条会红。
   - 形 3，E2 重读后从表里拿掉的槽：「                        occupancy.occupants.remove(&ring_slot);」（`crates/singlefs-core/src/allocator.rs:2223`）。
   - 形 4，那一槽记着的是别的根：见 K1-2（交 `None` 是设计）。
   另有一条实现员没列的：这个进程写掉那一槽就换成有效根：「            .insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」（`crates/singlefs-core/src/allocator.rs:2102`），那条被抛弃根也已离开根环、盘上读不到它，不是漏。

推翻它的现象：找到一条走 `raise_the_floor_through` 之外去重算影子账的抬 F 路径（grep 零命中，所以不太可能）；或表里已记着落点的被抛弃根在抬 F 时仍去读盘（K3 的变异 m1 到 m4、m14 与基线的对照说明没有）。

### K1-2　按 (实例代号, txg) 认同一条根

判定：**做到；对上了却不是同一条根的，没找到；是同一条根却对不上的，就是 K1-1 的形 1 到形 3**。

依据：
- 取法按槽找、再比整对：`placements_recorded_for_the_abandoned_root`（`crates/singlefs-core/src/allocator.rs:1111`），`AbandonedRoot` 臂比 `(*recorded_instance, *recorded_checkpoint_txg) == (instance, checkpoint_txg)`，「两遍里只有一遍自证过」的被抛弃那一半比那一遍读出的整条根记录的 `(instance, checkpoint_txg)`；库单测 `placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root_not_for_another_root_on_its_ring_slot` 钉同槽差一整圈、实例代号不同、有效根三种交 `None`。
- 我另造的变异里，把比较削成只比 txg 或只比实例代号的四条（m9 到 m12）、把槽号偏一格的 m13、把构造器里存的实例代号写成 0 的 m3、把 txg 加一的 m4、把抬 F 那一遍的键写成 `(实例, 0)` 的 m14，都被抓到（表在 K3）。
- 为什么没有「对上了不是同一条」：一个进程写的根 txg 逐个加一，写出的根都带本进程的实例代号；被抛弃根的实例代号是前一个或更早的进程的。进程内不会再写出一条与被抛弃根同 (实例代号, txg) 的根（推的，依据 `record_root_written_by_this_process` 里 txg 逐个加一的断言）。
- 弱点（没反例，写下来）：取法只靠「槽号 ← txg」定位，若一条根被错放在不是它 txg 对应的槽上，查不到、落回读盘，是保守方向。

推翻它的现象：造出一条 (实例代号, txg) 与表里记着的相同、而内容（引用的落点）不同的被抛弃根；本轮没造到，也没有机制让它出现。

### K1-3　可写挂载那一道（含崩了再可写挂载）的 B、C、F1、F2 一个字没动

判定：**做到**。

依据：
1. 读改动全文（`research/prompts/_m3-batch10z-b-code-r1-diff.md`）里 mount.rs 的 hunk 头：`@@ -1473`、`-1484`、`-1491`、`-1505`、`-1536`、`-2475`、`-2537`、`-3011`、`-3767` 九段。1536 之后到 2475 之间没有 hunk：B 的函数 `placements_of_an_abandoned_root_whose_account_is_unreadable`（冻结副本 `crates/singlefs-core/src/mount.rs:1639`）、凭据函数、C 的两个函数（`placements_named_by_unapplied_journal_records`、`placements_in_the_accounts_of_unapplied_publishes`）都在这一段里，一个字没动。落在可写挂载上的只有两处：传读盘那一臂：「            PlacementsOfAbandonedRoots::ReadFromTheirAccounts,」（`crates/singlefs-core/src/mount.rs:2521`），与构造 `AbandonedRoot` 时多存两个字段：「                ) => RootRingOccupant::AbandonedRoot {」（`crates/singlefs-core/src/mount.rs:2583`）。
2. 读盘那一臂行为与改前相同：`ReadFromTheirAccounts => None`（`recorded_in_the_root_ring_table` 的第一臂），`.or_else` 接的是改前那一句 `placements_referenced_by_root(...)`，拒的位置不动（`crates/singlefs-core/src/mount.rs:1659`）。
3. 复跑：基线上 `credential`（2 条）、`corrupt`（`an_abandoned_root` 过滤 1 条）、`slotbad`（3 条）全绿，新用例第三条「崩了再可写挂载没有凭据照拒」绿（K3 基线表）。把 B 的凭据改成恒成立（变异 m15）这四处全红：新用例第三条、`an_abandoned_root_whose_instance_table_pointer_sits_below_the_unit_area_refuses_the_writable_mount_instead_of_panicking`、`…only_read_back_zero_still_refuses_the_unmount_before_any_write`、LR3 那条。
4. F1、F2：F1（记录有缺口时拒可写）与预隔离 C 在 hunk 之外；我没有单独对 F1、F2 造变异，只靠「hunk 之外 + 既有用例基线绿」判（`writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten` 一族我没跑）。

推翻它的现象：把 `rebuilt_allocator` 那一臂换成别的臂后新用例第三条仍绿（m15 说明不会），或 hunk 之外的行与冻结前不同（我没有改前的树，hunk 清单靠改动全文，没独立 diff 过）。

## K2　I-7.4（近 K 代块未被复用）与 I-4.8（近 K 代根校验和自洽）

### K2-1　记着的落点是挂载那一刻算的，抬 F 照它算，有没有一形掉出影子账

判定：**表里记着的被抛弃根，没找到掉出的一形；两处读码推出的缺口（没造）；表里没记着的四形不在这一问里（读盘照今天，K2-2 量了形 1）**。

依据：

1. 被抛弃根引用的落点不随时间变：一条根引用的单元在它自己的树表、分配记录树里，单元是写一次不动的；挂载之后能变的是那几个单元所在的槽能不能读（K4-2 那一形，定案要接住的），或被重新分配（被抛弃根的落点在根离开根环之前隔离着，`allocator.rs` 的 `AbandonedRoot` 文档：「/// 抬 F 那一串（准入与卸载）重算影子账时，这一槽这一遍读出的被抛弃根与这里记着的是同一条（实例代号与 txg 都对上）就照这一串算、」（`crates/singlefs-core/src/allocator.rs:1029`））。「挂载之后那条根的引用变了」我没找到机制。
2. 记着的是整条根的全部落点，没滤掉被候选根豁免的：循环里先 `insert(...)` 记全部，再滤候选与已隔离的（`crates/singlefs-core/src/mount.rs:1573` 起的循环：先取 `placements`，再 `for (device, slot, span_slots) in placements.iter().copied()` 过滤）。所以挂载那一刻被候选根豁免、之后抬 F 越过候选根的槽，抬 F 那一遍照记着的补隔离。T1 甲那一类（两遍里只有一遍自证过）：被抛弃那一半的落点同样记在 `RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses` 里，取法按那一遍读出的整条根认；库单测钉了取法，既有 `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root` 37 条在副本基线上绿；变异 m21（那一臂交空）库单测与既有两条红（K3）。
3. 实测（K2-2 的装置）：形 1 的两臂，两臂都做成首次抬 F 的例子里，首次推抬 F 之后每块盘的隔离槽集合逐槽相同（集合哈希一致，OW 为 14 槽、SQ 为 19 槽；见 K2-2），逐步 checker 没红过 I-7.4、I-4.8。这些例子里 R 的落点都不在「表里记着」的那一类，所以这条实测量的是形 1，不是「表里记着」那一类本身；「表里记着」那一类的实测是新用例的两条（卸载、会话写）与 K3 的基线。
4. 读码推出的两处缺口，**没造**：
   - 挂载时凭据放行的被抛弃根，表里记的是空串（`AbandonedRoot` 文档：「/// 账读不出（凭据放行）的、影子账关着（只供测试）的，这一串是空的：它什么都没隔离，离开根环时也什么都不清。」（`crates/singlefs-core/src/allocator.rs:1028`））；抬 F 这一遍照空串算。改前抬 F 会重新读盘、映射现算，瞬时读不出的那种可能多出落点。凭据的意思是那几槽已被之后的合法写占了，我没造出「挂载时凭据放行、之后又读得出且有该隔离的单元」。
   - 影子账关着（只供测试的开关）挂载，之后开着抬 F：表里记的是空串（`crates/singlefs-core/src/mount.rs:2525` 附近 `ShadowLedger::Off` 臂交空 map），抬 F 不再从盘上补算。产品路径挂载与抬 F 同开同关（`mount_writable` 传 `ShadowLedger::On`，`crates/singlefs-core/src/mount.rs:6036`），只在测试里混用才露。

推翻它的现象：造出一条表里记着落点的被抛弃根，抬 F 做成之后它引用的某个单元在 `isolated` 集合之外、且被分配器发出去（或池级 checker 红 I-7.4 / I-4.8）。我的 K2-2 探测在今天读盘臂与 C 臂各 40 个形 1 的例子上没造出来。

### K2-2　表里没记着的四形：第 1 形两臂都量

判定：**两臂都没有掉出影子账；今天读盘臂在「挂载之后 R 的账读坏」上白拒（19 例里 19 例），C 臂这一形不拒（SQ 10 例 10 例做成；OW 9 例抬 F 做成，第 32 次覆盖写的拒是装置假象，见下）**。以下只覆盖我造的这一档（384 槽小盘、之前覆盖写 1 次、R 两种形状、60 次会话覆盖写、每三步与每次写完/卸载后跑一次池级 checker），不是全空间。

造法（入库装置，在我副本上）：`crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs` 的 `record_one_publish_after_overwrites(1, kind)`，`kind` 取 `Overwrite`（OW，R 的单元写 20 份，10 个偏移 × 两盘）与 `SequentialWriteOfThreeDataUnits`（SQ，26 份，13 个偏移）；崩在 R 的根槽落盘之后、系统配置轮换之前（`crash_image_with_the_root_persisted_and_the_rotation_lost`）。挂载 M1（空间准入判着、影子账开着）时让 R 的某一个单元（逐个换，OW 10 个偏移、SQ 13 个偏移）读回全零，恢复择到 P，R 的根槽与账读得出，被当成候选根（K4-1 的 R）。M1 写行之后 R 按实例表被抛弃，根环表里是有效根：核了 `placements_recorded_for_the_abandoned_root(R) = None`。C 圈里 R 那一槽确有记着的落点：OW 为 26 或 48 个、SQ 为 44 或 72 个（`C_for_valid_R`）。
故障三档（只动 R 的树表、中央映射树根的位置条目，设备报读错）：账无故障；仅 M1 时读错；M1 之后读错（挂着时坏）。M1 之后会话覆盖写到 60 次，写满前准入推抬 F，最后正常卸载。
**C 臂**：在副本里加的开关（`SINGLEFS_ARM_C`，diff 在模型目录 `k2/c-arm-mount.rs.diff`、`k2/c-arm-allocator.rs.diff`），抬 F 那一遍对表里没记着、又是「挂载那一刻记成有效根（txg 对得上）、同槽上 C 圈有按槽记的落点」的被抛弃根，取那一圈的落点；别的照旧。我给它多加了一条「那一槽记的是有效根且 txg 相同」的认同条件，因为 C 圈只按槽号存、不带 (实例, txg)。
复跑（线程 8，内存 12G，`nice -n 19`）：
`SWEEP_MOD=3 SWEEP_REM=<0|1|2> [SINGLEFS_ARM_C=1] CARGO_TARGET_DIR=…/target bash scripts/run-with-memory-cap.sh 12G bash scripts/capped.sh 8 cargo test --offline -p singlefs-harness --test k2_arms_probe -- --nocapture --test-threads 8 s_ [m_]`，每臂三次，合并后 50 个例子（46 个读零 R 单元的例子 + 4 个无读零的 M 例子）。统计脚本 `k2/an.py`，原样输出在 `k2/an-output.txt`，逐例原始行在 `k2/sweep-today-arm.txt`、`k2/sweep-c-arm.txt`。`an.py` 输出里与判定有关的行（原样）：

```
cases today 50 c 50 same labels True
== 今天读盘臂
('OW', '挂后账读错', 'selP', 'Cvalid') n=9 拒9 {(17, 'NewerStateStillUnreadableAfterOneReread'): 9} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'拒': 9}
('OW', '账无故障', 'selP', 'Cvalid') n=9 拒0 {} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'做成': 9}
('SQ', '挂后账读错', 'selP', 'Cvalid') n=10 拒10 {(17, 'NewerStateStillUnreadableAfterOneReread'): 10} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'拒': 10}
('SQ', '账无故障', 'selP', 'Cvalid') n=10 拒0 {} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'做成': 10}
== C 臂
('OW', '挂后账读错', 'selP', 'Cvalid') n=9 拒9 {(32, 'RollbackFloorCeilingNeedsUnreadableValidRootTreeTable'): 9} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'做成': 9}
('OW', '账无故障', 'selP', 'Cvalid') n=9 拒0 {} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'做成': 9}
('SQ', '挂后账读错', 'selP', 'Cvalid') n=10 拒0 {} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'做成': 10}
('SQ', '账无故障', 'selP', 'Cvalid') n=10 拒0 {} 违例(逐步/写后)0 违例(卸载后)0 卸载 {'做成': 10}
隔离集合 两臂都做成抬F的例子：相同 25 不同 0
一臂做成一臂没做成的例子数 25
两臂都做成但集合不同 []
{'both None': 6, 'today None / C Some': 19}
```

（另有「仅挂载时读错」两臂各 2 例：都不拒、都做成、0 违例；R 被择为现行版本、不是被抛弃根的 10 例（`notP`）两臂同样，拒因都是算上限读有效根的树表，不在这一问内。）

读法：
- 掉出：今天读盘臂 40 个形 1 例子、C 臂 40 个，逐步 checker 与卸载之后的 I-7.4、I-4.8 都是 0 违例；两臂都做成首次抬 F 的 25 例（形 1 的 21 例：OW 10 例每块盘 14 槽、SQ 11 例每块盘 19 槽；另 4 例是 R 被择为现行版本、隔离 0 槽），隔离槽集合逐槽相同（集合哈希一致，`ISOSET` 行）。没有量出掉出。
- 白拒：今天读盘臂在「挂着时 R 的账读坏」19 例全部拒：抬 F 在第 17 次覆盖写被拒成 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`，点名 R（OW 例子里 R 是 (实例 1, txg 5)），卸载同样拒，盘上一个写都没发。条款说的是不拒：「读不出也不拒」（`.claude/kb/decisions/16-发布语义.md:37`）。C 臂：SQ 的 10 例整串做成；OW 的 9 例第 17 次抬 F 做成，第 32 次覆盖写被拒，**拒因点名的根是 (实例 2, txg 32)，不是 R**：我注入的读错区间按物理槽记（R 的树表在槽 1502，探测原样输出 `ACCOUNTSLOTS Overwrite R=(InstanceGeneration(1),CheckpointTxg(5)) tree_table=[(0, 1502), (1, 1502)] mapping_root=[(0, 1501), (1, 1501)] alloc_tree=[(0, 0), (0, 0)]`，拒因里写的是 `UnitUnreadable { slot: SlotNumber(1502) }`），抬 F 回收 R 的单元后新根的树表落到同一个槽上，装置的故障还开着；这是装置的假象，不是 C 臂的拒。
- 两臂的差只在「读不出」上：读得出时两臂交出同一个集合。C 臂依据的是 M1 那一刻 C 圈记下的落点（包括 M1 时账读不出的例子里，records 点名的单元），没有量到它比读盘少的。
- C 臂的认同条件是我加的，因为 C 圈按槽存；不加的话 C 圈只能按槽号对，可能把同槽上差一整圈的另一条被抛弃根当成 R（推的，没造）。

推翻它的现象：造出一个例子，今天读盘臂或 C 臂在抬 F 做成之后池级 checker 红 I-7.4 / I-4.8，或首次抬 F 之后某一臂的隔离槽集合是另一臂的真子集。我的 40 + 40 例没有。我没有把档开大（4 GiB 盘、k 取别的值、R 引用 P 没有的共享单元）；X 的发现（删掉 `protected.extend` 在 4 GiB 池上证红）说明大盘上抬 F 那一路更要紧，那一档本轮没量。

### K2-3　`RaisedFloor::abandoned_roots_unreadable` 不再数记着落点的根：有没有调用方靠这个数做决定

判定：**core 内没有；研究装置 e158（checker 档，我没跑）靠它，读码推出会红**。

依据：
1. core 内只有搬运：`raise_rollback_floor` 与 `unmount` 把 `raised.abandoned_roots_unreadable` 抄进结果（`crates/singlefs-core/src/mount.rs:3225`、`crates/singlefs-core/src/mount.rs:3275`），`raise_the_floor_through` 从影子账交回里取：「            Ok(computed) => computed.abandoned_roots_unreadable,」（`crates/singlefs-core/src/mount.rs:3856`）。没有 `if`、`match` 在它上面分叉（`grep -rn '\.abandoned_roots_unreadable' crates --include=*.rs`：core 里命中就这几处；`MountOutput` 那个同名字段是挂载的，不变）。
2. harness 档断言它的：新用例卸载那条断言 0（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:389`），不记着的那一形仍数 1：`assert_eq!(raised.abandoned_roots_unreadable, 1, "C 的账读不出");`（`crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:1117`，慢档 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable`；我在 K3 的 m16 那一轮里跑过一次，`1 passed`，说明那一形的根没被记着、仍数 1）。变异 m16（把记着的也数进去）新用例卸载那条红。
3. 研究装置：e158 的 op1 第三种变体把它当「触发」统计：`abandoned_roots_unreadable: Some(raised.abandoned_roots_unreadable),`（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:2077`）、`if let Some(count) = attempt.abandoned_roots_unreadable {`（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:3026`）。它的单元测试 `mount_writable_then_raise_floor_reaches_the_injected_fault`（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:22856`）先不注入地可写挂载、再对被抛弃根的分配记录树节点注入读错、再抬 F，并断言：「                "抬 F 重算影子账要数到这条读不出账的被抛弃根，得到 {:?}（不注入时 {:?}）",」（`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:22914`）要求计数大于 0。这一批之后，那条被抛弃根挂载时已记着落点，抬 F 不读盘、计数是 0，断言应当红（读码推出；它在 checker 档，**我没跑**）。同族的 E158 触发计数（`raise_floor_trigger_count`）也会掉。实现员报告没有提这一条（他只跑了 core lib 与 13 份 harness 档目标）。

推翻它的现象：在 checker 档装置上跑 `mount_writable_then_raise_floor_reaches_the_injected_fault` 仍绿（那说明那条被抛弃根在装置的池里没被记成被抛弃根，比如恢复落到的与我读的不同）。

## K3　测试判别力

判定：**新用例与库单测拦得住这一批改动里「传哪一臂」「认同一条根」「构造器存的字段」「计数口径」「B 不动」这五处；拦不住「记着的落点内容被削」（m1、m2，只有旧用例拦）和「取法对四类槽交空」（m5、m8、m20）。LR3 改完之后原来的判别力还在，但它对这一批已无判别力；抬 F 那一遍的映射现算没有任何用例钉着（这一批之前也没有）**。

量法：在 `repo-k3`（冻结副本拷贝，编译目录 `target-k3`）上对源码逐条施加变异、跑下面几个 harness 档目标、还原，每条用 `python3 k3/mut.py <名>` 施加（原文须恰好命中一次）、还原后 `diff -r` 与冻结副本相同（最后一次核：`CLEAN`）。脚本 `k3/mut.py`、`k3/run.sh`、`k3/all.sh`、`k3/all2.sh`、`k3/all3.sh` 与每条的结果摘要 `k3/out/*.txt` 在模型目录。目标简称：`新` = 新用例文件三条；`库` = `singlefs-core --lib`（200 条）；`LR3` = LR3 文件里 `a_floor_raise_whose_shadow_ledger…` 与 `an_abandoned_root_with_a_two_page…` 两条；`凭据` = `abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots`（2 条）；`坏槽` = `unreadable_abandoned_root_slot`（3 条）；`复用` = `reuse_after_raising_the_floor`（3 条，m16 另跑慢档 1 条）；`一遍` = `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root`（37 条）；`坏盘` = `corrupt_on_disk_content_is_refused_instead_of_panicking` 里 `an_abandoned_root` 过滤的 1 条。基线（未变异）：库 200 绿、新 3 绿、LR3 2 绿、凭据 2 绿、坏槽 3 绿、复用 3 绿、一遍 37 绿（`k3/out/baseline.txt`）。

| 变异 | 改在哪 | 红的 | 照绿的 |
|---|---|---|---|
| m1 记着的落点交空（`.take(0)`） | `mount.rs` `recorded_in_the_root_ring_table` | LR3 两片 1 条、一遍 2 条 | **新 3 条、凭据、坏槽、复用** |
| m2 只交第一个落点 | 同上 | 同 m1 | **新 3 条** |
| m3 构造器里存的实例代号写 0 | `mount.rs:2583` 那一臂 | 新 2 条（卸载、会话写）、LR3 1 条 | 凭据、一遍 |
| m4 构造器里存的 txg 加一 | 同上 | 同 m3 | 一遍 |
| m5 取法对「挂载时读坏」交 `Some(空)` | `allocator.rs:1136` 那一行 | 无 | 库、新、凭据、坏槽、复用 |
| m6 取法对「读回全零」交 `Some(空)` | `allocator.rs:1137` | 凭据 1 条（`…only_read_back_zero_still_refuses_the_unmount_before_any_write`） | 库、新、坏槽、复用 |
| m7 取法对有效根交 `Some(空)` | `allocator.rs:1135` | 库 1 条 | 新 |
| m8 取法对「一遍自证过的有效半边」交 `Some(空)` | `allocator.rs` 取法最后一臂 | 无 | 库、新、坏槽、一遍 |
| m9 / m10 被抛弃根那一臂只比 txg / 只比实例代号 | `allocator.rs` | 库 1 条（各） | 新 |
| m11 / m12 一遍自证过的被抛弃半边只比 txg / 只比实例代号 | 同 | 库 1 条（各） | 新 |
| m13 取法按 txg 加一找槽 | `allocator.rs` | 库 1 条、新 2 条 | — |
| m14 抬 F 那一遍的键写成 (实例, 0) | `mount.rs:3829` 之后的 map | 新 2 条 | — |
| m15 B 的凭据恒成立 | `mount.rs:1659` | 新第三条、坏盘 1 条、凭据 1 条、LR3 1 条 | — |
| m16 记着落点的根也计入「读不出」 | `mount.rs:1585` 之前 | 新卸载条 | 复用、复用慢档 |
| m17 重建分配器存进表里的落点清空 | `mount.rs:2583` 那一臂 | 新 2 条、LR3 两片 1 条 | 一遍 |
| m18 抬 F 去掉 `.filter(abandoned_by_table)` | `mount.rs:3829` | 无 | 新、LR3 |
| m19 抬 F 那一遍关掉映射现算 | `mount.rs:1639` 起的函数 + 线程局部标志 | 无 | 新、LR3 整份 13 条、凭据、坏槽、复用、坏盘、一遍 |
| m20 取法对「读坏后重读读出有效根」交 `Some(空)` | `allocator.rs` | 无 | 库、新、凭据、坏槽、一遍 |
| m21 一遍自证过的被抛弃半边交空串 | `allocator.rs` | 库 1 条、一遍 2 条 | 新 |
| m22 改回批前（抬 F 读盘）并关掉抬 F 的映射现算 | 两处 | 无 | LR3 整份 13 条 |
| m23 改回批前（抬 F 传读盘那一臂） | `mount.rs:3851` | 新 2 条（卸载、会话写） | LR3 整份 13 条 |
| m24 LR3 的原变异（影子账交 Err 不换回分配器） | `mount.rs:3856` 之后 | LR3 `…restores_the_allocator` 1 条 | 其余 12 条 |

数：m1 至 m21 共 21 条变异，16 条有红的；其中被这一批自己的新用例或库单测抓到的 13 条（m3、m4、m7、m9 至 m17 共九条、m21），只被旧用例抓到的 3 条（m1、m2、m6），5 条全绿（m5、m8、m18、m19、m20）；13 + 3 + 5 = 21。m18 我判等价（表里记成 `AbandonedRoot` 的根按现行表一定也被抛弃，实例表只增不减，推的）。m22、m23、m24 是对照，不计入 21。

要点：
1. **新用例只断言「没拒、计数 0、checker 不红」，不断言隔离了什么**。卸载那条断言计数 0：「            raised.abandoned_roots_unreadable, 0,」（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:389`），checker 违例的过滤只取 I-7.4、I-4.8：「        .filter(|violation| violation.starts_with("I-7.4") || violation.starts_with("I-4.8"))」（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:366`）。384 槽小盘上这几十次写里没有用到被抛弃根的单元，所以把记着的落点削成空（m1）或一个（m2）它仍绿；抓住它们的是旧用例（LR3 两片、一遍）。想让新用例自己抓：加一句断言抬 F 之后分配器的隔离槽集合包含根环表里记着的那些落点。
2. **取法的 `None` 臂只有一条库单测，且只含有效根、被抛弃根两类**。读坏、读回全零、读坏后重读读出有效根、一遍自证过的有效半边这四类槽，库单测一个都没放（`placements_recorded_for_an_abandoned_root_answer_only_for_the_same_root_not_for_another_root_on_its_ring_slot` 用的是 `AbandonedRoot`、一遍自证过的被抛弃、有效根三类）。m5、m8、m20 把其中三类改成交空落点，所选 7 个目标全绿；m6 只被一条既有用例抓到。这几条变异若真发生，抬 F 会把读到的被抛弃根当成「引用 0 个落点」，不隔离也不读盘，方向是掉出影子账（推的，我没把它造成 checker 红）。
3. **LR3**：原来要钉的是「影子账交 `Err` 时分配器整个换回」，m24（原变异）仍让它红，所以那一件钉着。若问的是「被抛弃根的账读不出时从中央映射现算」：LR3 本来就不钉它；它由 c393 一族与两片实例表那条用例钉在**挂载那一遍**；**抬 F 那一遍的映射现算**这一批之后只在表里没记着的四形上走到，m19 全绿；而且 m22（把这一批改回、再关掉抬 F 的映射现算）下 LR3 整份 13 条也全绿，说明改前也没人钉，不是这一批弄丢的。LR3 对这一批已无判别力：m23（改回批前）LR3 整份 13 条仍绿（新用例才抓得住，上表 2 条红）。
4. 改过的 LR3 把「表里没记着时照读盘、读不出照拒」钉成了期望：其文档写着：「抬 F 那一遍读出它、照盘上读它的账（这一形条款没写，照改之前，交主 agent）。」（`crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:656`）。主 agent 若按条款字面（「一律」）改，这条要翻。
5. 我没有重跑实现员追加的四行变异；m23 与实现员第 1 行（抬 F 改回读盘）同形，新卸载、会话写两条红，对得上他的记录。

推翻它的现象：对表里 m5、m8、m20 任一条，找到在 harness 档里照红的既有用例（我只跑了上面列的 7 个目标，没跑整个 harness 档，可能有我没选到的目标抓住它们）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 | 做到，但有一形漏 | 四个入口汇到一处、影子账只有两个调用点；表里没记着的四形仍读盘、读不出照 B 拒（形 1 实测白拒 19/19），字面「一律」没做到 |
| K1-2 | 做到 | 按槽找再比 (实例, txg)，库单测与 m9 至 m14 抓住；对上了不是同一条的没找到，同一条却对不上的是 K1-1 的形 1 到 3 |
| K1-3 | 做到 | mount.rs 九段 hunk 里可写挂载那一道只有传读盘臂与构造器加字段；B、C 函数所在段无 hunk；m15 与基线对得上 |
| K2-1 | 表里记着的没找到掉出 | 两臂首次抬 F 后隔离集合逐槽相同、0 违例；挂载时凭据放行记空串、影子账关着挂载记空串两处读码缺口没造 |
| K2-2 | 两臂都没掉出；今天读盘臂白拒，C 臂不白拒 | 形 1 共 40 例：读盘臂「挂后 R 账读错」19/19 抬 F 与卸载被拒，C 臂 SQ 10/10 做成、OW 9/9 抬 F 做成（第 32 次的拒是装置假象）；0 违例 |
| K2-3 | core 内没有调用方；e158 有 | 研究装置 e158 的 `mount_writable_then_raise_floor_reaches_the_injected_fault` 靠它大于 0，读码推出这一批后会红，checker 档没跑 |
| K3 | 部分拦住 | 21 条变异 16 条有红、13 条被新用例/库单测抓；漏 m1、m2（只旧用例抓）与 m5、m8、m20、m19（全绿）；LR3 原判别力还在，对本批无判别力 |

## 没做什么

- 没跑 checker 档（`singlefs-checker-tier` 的测试，含 e158 那条单元测试）、`gate.sh`、QEMU、变异整表、层 0 快档；没提交、没改仓内文件，只写了本报告与模型目录。
- 没跑整个 harness 档；K3 只跑了上面列的 7 个目标，K2 只跑了自己的探测。没单独对 F1（记录有缺口时拒可写）、F2 造变异。
- K2-2 只量了 384 槽小盘、k = 1、R 两种形状、60 次覆盖写、读错三档；没量 4 GiB 档，也没量 R 与 P 共享单元在抬 F 越过 P 之后的那一形。
- 没替主 agent选 C 臂或读盘臂；表里没记着的四形里，形 2 到形 4 我只读码，没造。
- 起过一条 `./all.sh … &`（K3 变异串，约 50 分钟）与一条 `./sweep_all.sh … &`（K2 扫描串），都在前台 `proc.py wait` 等完，没有留下在跑的进程；这与「长活交回」那条不完全一致，写在这里。

## 要主 agent 跑的命令

- checker 档的 e158 单元测试（K2-3 的读码推断要它来证）：`cd <仓> && SINGLEFS_HEAVY_TESTS=commit bash scripts/run-with-memory-cap.sh 12G bash scripts/capped.sh 8 cargo test -p singlefs-checker-tier --bin e158_root_choice_repair mount_writable_then_raise_floor_reaches_the_injected_fault`，看这一条红不红、红的话断言文字是不是 `抬 F 重算影子账要数到这条读不出账的被抛弃根`。
- K2-2 的数要在入库装置上重做才能引（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节）：把 `research/prompts/m3-batch10z-b-code-r1-sonnet-model/k2/k2_arms_probe.rs` 与两份 `c-arm-*.diff` 做成入库装置的新一次跑，再引。

## 草稿目录与模型目录

模型目录 `research/prompts/m3-batch10z-b-code-r1-sonnet-model/`：`k2/`（探测、C 臂 diff、统计脚本与输出、逐例原始行）、`k3/`（变异表、驱动脚本、结果摘要）、`README.txt`。草稿目录 `~/.local/share/singlefs/work/m3-batch10z-b-code-r1-sonnet/`：已删掉 `repo`、`repo-k3`（各 21 MB）、`target`（1.3 GB）、`target-k3`（3.1 GB）四个，留日志（`sweep/`、`k3/out/`、`probe-*.log`、`progress.md`）。
