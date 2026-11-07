# m4-gpugc-shared-bytes-code-r1 正推腿（Sonnet）报告

立场：正推。核「代码今天做的是不是改法声称的那件事」。T1、T4 主攻。方案按 `crates/` 今天的实现来谈。

记号：`R/` = 冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo/`；下文代码引文都写 `R/` 之下的路径与行号。
引补丁块只写补丁目录名：第 86 批 = `m4-gpugc-shared-unit-bytes-r4/patch/`，第 88 批 = `m4-gpugc-name-collisions-writer/patch/`。

状态：分段落盘，本段是开头与格 T1；T2、T3、T4、判定一览、要主 agent 跑的命令、没做什么在后面各段。

## T1　过度外推与越位：改的真只是「持有方式」吗

### T1.0　先答「有没有一处读的是『是不是同一份』」

结论：产品代码里没有。命令与输出（在冻结副本里跑，范围是被判的 13 份文件加整个 `crates/`）：

```
grep -rnE 'ptr_eq|strong_count|weak_count|Arc::as_ptr|as_ptr\(\)' crates --include=*.rs | grep -v e202_gpu_cpu_gc
```

命中里和 `Arc<TransactionOutput>`、`Arc<[u8]>` 有关的只有测试：`crates/singlefs-core/src/mount.rs:14286`、`:14296`、`:14335`、`:14339`、`:14368`、`:14378`、`:14410`、`:14419`（本批新加的测试模块，判的是「指着同一份」，正是测试要钉的）与
`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:2868`。
其余命中与这批无关：`crates/singlefs-verification/src/crash_amplification.rs:15172`、`:15177`（桶表里的见证索引）、`crates/singlefs-verification/src/gpu_unit_checks.rs:2667`、`crates/singlefs-core/src/background_compute.rs:476`、`crates/singlefs-core/src/container_tree.rs:4161`、`:4176`（一块缓冲的地址）、`crates/singlefs-core/src/block_device.rs:1062`（对齐）。
`Arc::get_mut`、`Arc::downgrade`、`Weak<`、`unsafe` 在 `transaction.rs`、`mount.rs`、`mounted_session.rs`、`recovery.rs`、`instance_table_sweep.rs` 里零命中（`get_mut(` 的三处命中是 `instance_table_sweep.rs:1062` 的 map 取值与 `mounted_session.rs:1239`、`:1251` 的单元格，与这批无关）。

两个读身份的隐含处，写明、不放过：
1. 标准库给 `Arc<T: Eq>` 的 `==` 有「同一份就直接判相等」的快路（`Arc::ptr_eq(a, b) || **a == **b`）。`TransactionOutput`（`crates/singlefs-core/src/transaction.rs:3250` 的 `#[derive(Clone, Debug, PartialEq, Eq)]`）与 `PoolVersion` 都是派生 `Eq`，自反，所以快路只省时间，不会把「内容不等」判成相等。这一条是**推的，没量过**：本机没有 rust-src（`ls ~/.rustup/toolchains/*/lib/rustlib/src/rust/library/alloc/src/sync.rs` 无输出），标准库源码核不了；推翻它的现象：某个含 `f64` 之类非自反成员的类型被塞进 `TransactionOutput`（派生 `Eq` 会编译不过，所以不会发生）。
2. `Arc<TransactionOutput>`、`Arc<[u8]>` 的 `Debug`、`PartialEq`、`Hash` 都按内容走，不带地址（`PublishedUnit.bytes` 的文档注释 `crates/singlefs-core/src/transaction.rs:3237`–`:3239` 自己写了「相等照字节比，与共不共享无关」）。

字节比较处：`crates/singlefs-core/src/mount.rs:6877`、`:7093` 把 `Vec<u8> == Vec<u8>` 改成 `bytes_on_this_device[..] == unit.bytes[..]`，是内容比较，不是地址比较。

**TransactionOutput 里没有内部可变性**（共享才安全的另一半）：`grep -nE '(Cell|RefCell|Mutex|RwLock|Atomic[A-Z][a-z0-9]*)<?' crates/singlefs-core/src/transaction.rs` 的命中全在 `PoolWriter`（`transaction.rs:369`、`:372`、`:376`）与几个局部量，不在 `TransactionOutput`（`:3251` 起）的字段里；`TransactionOutput` 的字段类型来自的模块（`allocation_record_tree`、`code_two_tree`、`container_tree`、`extent_tree`、`inode_tree`、`root_record` 等）不在 `grep -rlE '\b(Cell|RefCell|OnceCell|Mutex|RwLock)<' crates/singlefs-core/src` 的命中里（命中是 `region_scan`、`mounted_read`、`recovery`、`instance_table_sweep`、`background_compute`、`transaction`、`mount`、`mounted_session`，`recovery.rs` 里的 `OnceCell`/`Cell` 在读盘用的读入器上，`recovery.rs:204`、`:206`、`:327`、`:1147`、`:6620`，不在 `TransactionOutput` 字段里）。这一条是按模块 grep 推的，没有逐字段类型核；推翻它的现象：`TransactionOutput` 的某个嵌套字段类型里含 `Cell`。

### T1.1　五处 `Arc::make_mut`（验收：每处一行表，有结论与依据）

总论（读码推的）：改法之前，每个持有者（会话 / 挂载交回值 / 版本列表 / 整理）手里都是各自一份深拷；改法之后持有者可能指着同一份，但 `Arc::make_mut` 在还有别人指着时先拷一份、改自己的那一份，所以「别人看不见这次改动」在每一处都由 `make_mut` 的语义给出，与改前的「各拷一份」逐持有者等价；要紧的是**改动落在哪一份上**——`make_mut(&mut x)` 把拷出来的新份换进 `x` 自己，所以该看到改动的那一方（拿 `&mut Arc` 的那一方）看得到。每处的持有者数由读码得出，没有运行时量过（见「要主 agent 跑的命令」第 1 条）。

| # | 位置（冻结副本） | 改的是谁、那一刻的持有者 | 拷完之后 | 钉它的 | 结论 |
|---|---|---|---|---|---|
| 1 | `crates/singlefs-core/src/mount.rs:4610`（`unmount`） | 改的是调用方 `&mut PoolVersion` 里的那一份（`current`）。持有者：`current` 自己；另有可能指着同一份的：可写挂载交回的 `MountOutput.warm_up_publishes` 最后一项（`mount.rs:8628` 起 `current = row_publish.clone()`、`:8644` `warm_up_publishes.push(next.clone())`，本批测试 `writable_remount_hands_back_its_last_warm_up_publish_as_the_current_version` 钉「最后一次暖机与现行那一版指着同一份」），或会话交回的 `versions_published` 最后一项。`unmount` 在 `singlefs-core/src` 里没有调用方（`grep -rnE 'unmount\(' crates/singlefs-core/src` 只命中定义 `mount.rs:4584`），调用方都在 `singlefs-verification`（`src/scenario.rs:792`、`src/bin/e180_*.rs:945`、`src/bin/e181_*.rs:2136`、各 `tests/`）。刚挂载完就卸载：引用数 ≥ 2 | 拷一份、抬 F 就地推进拷出来的那一份并留在 `current` 里；`warm_up_publishes` 里那一份还是抬 F 之前的。改前这两处各是独立深拷，抬 F 同样只推进 `current` | 没有变异行点名 `make_mut`（`grep -c make_mut crates/mutations.tsv` 为 0）；间接钉的是「卸载之后读 `current` 的 floor」的各用例（推的，没逐个核） | 照旧 |
| 2 | `crates/singlefs-core/src/mount.rs:8335`（`continue_the_deletion_queue_until_it_is_empty`，`walked`） | 改的是局部 `walked`。`walked = Arc::clone(current_version_of_trees)`（`:8281`）起手；每轮 `walk_the_deletion_queue_until_it_is_empty` 发成一步就 `*current = Arc::new(next); steps_published.push(Arc::clone(current))`（`transaction.rs:12319`–`:12320`），所以被空间拒的那一刻持有者：`walked`、`continuation.steps_published` 最后一项（发成过步时），或 `walked` 与 `*current` 原来那一份（一步没发成时）；抬 F 之后 `walked` 已是自己的一份，下一轮没发成步而再被拒时引用数为 1，原地改 | 拷一份后 `walked` 指向新份，抬 F 推进它；续做交回的 `steps_published` 最后一步仍是抬 F 之前的（与 `Mounted.current` 不再是同一份，字段注释 `mount.rs:1102`–`:1103` 写的「走到了空时最后一个就是 `Mounted::current`」只在没抬 F 时成立，改前也是这样） | `crates/mutations.tsv` 第 6917 行（暖机）管的是 `warm_up_publishes`，不是这一处；续做里抬 F 的用例是 `crash_given_states_of_the_images_harness_scenarios_hand_out` 的 `a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_and_continues_to_the_emptied_version`（第 86 批替换行 `三·2 后续 2` 点名它，看的是 `*current =` 那一行，不是 `make_mut`） | 照旧 |
| 3 | `crates/singlefs-core/src/mount.rs:8708`（`acquire_then_publish_until_the_continuation`） | 改的是局部 `current`（`PoolVersion`）。`current` 起手是 `row_publish.clone()`（`mount.rs:8628`），每次暖机 `warm_up_publishes.push(next.clone()); current = next;`（`:8644`–`:8645`），所以走到这一处 `current` 与 `row_publish`（无暖机时）或 `warm_up_publishes` 最后一项**一定**共享：引用数 ≥ 2，一定拷一份 | 抬 F 推进 `current` 这一份；`row_publish`、`warm_up_publishes` 随 `PublishedBetweenAcquisitionAndTheContinuation` 交回，仍是各自发布时的样子。改前 `.clone()` 是深拷，同样不受抬 F 影响 | 行 6917 钉共享（不是钉 `make_mut`）；抬 F 之后 `current` 不等于最后一次暖机的用例同上推的 | 照旧 |
| 4 | `crates/singlefs-core/src/mounted_session.rs:5079`（`push_one_floor_raise_for_a_refusal_for_space`） | 改的是 `&mut Arc<TransactionOutput>`。调用方两处：`:5169`（走空里被拒，`current` 是 `walked`）与 `:3163`（改动本身被拒，`version_to_change` 直接指进会话 `current` 的内层 `Arc`）。持有者：走空发成过步时，版本列表（`published_in_this_change.versions_published`）里的最后一项与它共享；`walked` 刚 `Arc::clone` 自会话 `current` 时（`:3066`）一步没发成则与会话 `current` 共享；一轮抬 F 之后自己的一份引用数为 1 | 拷一份、抬 F 就地推进它；版本列表里那一份是抬 F 之前的，改前也是 | 行 6914、6915（`:3132`、`transaction.rs:12320` 的共享）；没有行点名 `make_mut` | 照旧 |
| 5 | `crates/singlefs-core/src/transaction.rs:2561`（`PoolVersion::with_writes`） | 改的是刚落盘（或重发）的那一版的 `writes` 字段。两个调用方，见 T2 | 见 T2 | 间接：凡是读发布交回版本 `writes` 的用例（`write_accounting_per_structure` 等，推的） | 照旧（引用数见 T2） |

「照旧」的依据（每一行共用）：(a) 所有改 `TransactionOutput` 的写位置（`&mut TransactionOutput` 形参，`grep -rnE '&mut (TransactionOutput|PoolVersion|Arc<TransactionOutput>)' crates/singlefs-core/src` 共 30 处）要么拿到的是局部独有的一份（`version_to_relocate_on = version_after_the_change.clone()`，`mounted_session.rs:2116`，深拷、之后 `*current =` 换进去），要么经这五处 `make_mut`；没有 `get_mut`、`unsafe`、`as_ptr`（T1.0）；(b) 因此没有任何一方能看到另一方「还指着自己时」的改动。这一条是读码推的；推翻它的现象：找到一处不经 `make_mut` 却改了共享 `TransactionOutput` 内容的写（需要内部可变性或 `unsafe`，T1.0 两处 grep 零命中）。

### T1.2　`Arc::clone`、`Arc::new`、`Arc::unwrap_or_clone`（逐处，产品代码）

| 位置（冻结副本） | 动作 | 那一刻的持有者 | 钉它的 |
|---|---|---|---|
| `crates/singlefs-core/src/transaction.rs:12319`–`:12320` | 走空每发成一步：`*current = Arc::new(next)` 后 `steps_published.push(Arc::clone(current))` | `current`（调用方的 `walked` / 会话 `current`）与列表最后一项；前一步只剩列表一份 | 变异行 `crates/mutations.tsv` 第 6915 行（`an_unlink_through_the_session_hands_back_the_last_step_of_the_walk_as_the_session_current`） |
| `crates/singlefs-core/src/mounted_session.rs:3066`、`:3132` | 走空起手 `walked = Arc::clone(...)`；unlink 意图那一版 `Arc::new(published)` 后 `.push(Arc::clone(&published))` | 起手：会话 `current` 与 `walked`；意图那一版：列表与 `published`，之后 `*current = ...` 换进会话 | 行 6914（`user_change_hands_back_the_version_it_published_as_the_one_the_session_holds_as_current`）；`三·2 第 8 条` 替换行（`the_session_hands_in_the_intent_and_every_step_of_an_unlink_in_publish_order`，替换文把 `.push(Arc::clone(&published))` 换成 `drop(Arc::clone(&published))`） |
| `crates/singlefs-core/src/mounted_session.rs:2135` | 搬迁另发那一批：`Arc::new(*relocated)`、`.push(Arc::clone(&relocated))`、`*current = ...shared...` | 列表与会话 `current` | 没有点名这一行的变异行（`grep -nF 'Arc::clone(&relocated)' crates/mutations.tsv` 零命中）；`m4-close-code-r2 快照下另发搬迁` 替换行钉的是 `*current =` 那一句（推的：不钉共享） |
| `crates/singlefs-core/src/mounted_session.rs:3338`、`:3371`、`:3394`、`:3406` | 推环：`start = Arc::clone(current 的内层)`；预演 `rehearsed = Arc::clone(&start)`；真发 `last_published = start`；发成一步 `versions_published.push(Arc::clone(&next))` | 预演只读；真发：列表最后一项与 `last_published`（之后 `*current =` 换进会话） | 行 6916（`a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through`，断言在 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:2868`） |
| `crates/singlefs-core/src/mount.rs:8281`、`:8628`、`:8644` | 挂载续做 `walked = Arc::clone(...)`；暖机 `current = row_publish.clone()`、`warm_up_publishes.push(next.clone())`（`PoolVersion` 的 `clone` 现在是 `Arc` 增计数） | 见 T1.1 第 2、3 行 | 行 6917（`writable_remount_hands_back_its_last_warm_up_publish_as_the_current_version`） |
| `crates/singlefs-core/src/mounted_session.rs:3774`–`:3788`（整理那一趟的走空） | `walked = Arc::clone(current.shared_version_of_trees())`；`*current = ...shared...(walked)`；`walk.steps_published.into_iter().map(Arc::unwrap_or_clone)` | 3786 之后：最后一步引用数 2（列表 + 会话 `current`）→ 拷一份；前面各步引用数 1 → 原样取出 | 没有（`grep -c unwrap_or_clone crates/mutations.tsv` 为 0）；T3 再谈 |
| `crates/singlefs-core/src/transaction.rs:2498`、`:2525`（`into_file_version`、`into_version_of_trees`） | `Arc::unwrap_or_clone` | 产品里 `into_version_of_trees` 只有一个调用方，`transaction.rs:12674`，接的是刚由 `PoolVersion::of_a_version_of_trees(output)`（`:12670`）新装的一份，经 `with_writes` 之后引用数为 1 → 不拷；`into_file_version` 在 `singlefs-core/src` 里只有测试 `instance_table_sweep.rs:1200` 用 | 无 |
| `crates/singlefs-core/src/transaction.rs:16341`、`:16355` 及补丁里其余 `Arc::from(...)` | 字节 `Arc<[u8]>` 的照抄 / 新装 | 照抄：上一版与这一版共享同一份不可变字节；新装：只此一份 | 行 6912、6913（`an_empty_publish_shares_the_bytes_of_every_unit_it_carries_with_the_version_before`，带「重写的树表不共用」的对照） |

`unwrap_or_clone` 交回时，是否有一方仍以为自己拿着被交回的那一份：没有。产品里只有两种结果——引用数 1 时原样搬出（原持有者随之消失，没有第二方），引用数 >1 时拷一份交出、其余持有者留着自己的；两种结果之后，没有任何一方还保留「指向被交回那份」的句柄去读内容或身份（T1.0）。推的；推翻它的现象：找到 `unwrap_or_clone` 之后还用「指针相等」判断两方是否同源的代码（T1.0 零命中）。

### T1 小结

- 五处 `make_mut`：全部「照旧」（价值语义不变），依据 = T1.0 的零命中 + 每处 `&mut Arc` 的拥有者拿到自己的一份。
- 没有变异行钉 `make_mut` 本身（0 行）：这五处的「该看到的看得到」靠间接用例；是否真会红，要主 agent 跑 `要主 agent 跑的命令` 第 2 条（把 `make_mut` 换成「改一份临时拷贝」）。
- 推翻 T1 的现象：任何一处读到指针身份或引用数；任何一处在 `make_mut` 之前别的持有者通过内部可变性改了内容；任何一个「该看到抬 F 的一方（持有者拿 `&mut Arc` 的那个）」看不到。

记号补充：下文「行 NNNN」指 `crates/mutations.tsv`（冻结副本里）的第 NNNN 行：6912、6913 = 甲档照抄两行；6914 = 用户改动列表；6915 = 走空列表；6916 = 推环列表；6917 = 暖机列表。
这六行的变异名、文件与点名的测试已在冻结副本里读过（`sed -n 6912,6917p crates/mutations.tsv`）。

### T1 附：一处文档与代码不一致（发现，不影响行为）

`crates/singlefs-core/src/mount.rs:1102`–`:1103` 的字段注释写「（两处指着同一份，不拷一整版；写行那一版与暖机各版同样与 `Mounted::current` 共享）」。读码：`mount.rs:8628` 起 `current = row_publish.clone()`，每次暖机 `next` 是 `publish_empty_after` 新装的一份（`:8644`–`:8645`），所以与 `Mounted::current` 同一份的只有**最后一次**暖机（无暖机时是写行那一版）；本批的测试自己就在断言反面：`crates/singlefs-core/src/mount.rs:14427` 的断言消息是「对照：写行那一版与现行那一版不是同一份」。注释把「各版」都说成共享，说多了。推的，没量过；`publish_empty_after`（`mount.rs:5910`）经 `.map(PoolVersion::of_a_version_of_trees)`（`mount.rs:5924`）每次新装一份 `Arc::new`，与上一次暖机不共享；推翻它的现象：某次暖机返回的 `Arc` 指着上一次暖机同一份（读 `mount.rs:5910`–`:5924` 即可看出不是）。

### T1 附：变异行名（`crates/mutations.tsv` 第一栏，原样）

| 行 | 变异名 | 点名的测试 |
|---|---|---|
| 6912 | 甲档照抄单元共用字节：照抄（carried_unit）退回整份复制字节（空发布照抄的单元与上一版不再指着同一份） | `an_empty_publish_shares_the_bytes_of_every_unit_it_carries_with_the_version_before` |
| 6913 | 甲档照抄单元共用字节：照抄的 extent 树节点退回整份复制字节（空发布照抄的 extent 树根与上一版不再指着同一份） | 同上 |
| 6914 | Arc<TransactionOutput>：用户改动发成的那一版放进版本列表时退回整版深拷（交回的那一版与会话的现行那一版不再是同一份） | `user_change_hands_back_the_version_it_published_as_the_one_the_session_holds_as_current` |
| 6915 | Arc<TransactionOutput>：走空每发成一步放进交回的列表时退回整版深拷（走空的最后一步与会话的现行那一版不再是同一份） | `an_unlink_through_the_session_hands_back_the_last_step_of_the_walk_as_the_session_current` |
| 6916 | Arc<TransactionOutput>：推环发成的每一版放进版本列表时退回整版深拷（推环的最后一版与会话的现行那一版不再是同一份） | `a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through` |
| 6917 | Arc<TransactionOutput>：可写挂载暖机发成的那一版放进挂载交回值时退回整版深拷（最后一次暖机与挂载交回的现行那一版不再是同一份） | `writable_remount_hands_back_its_last_warm_up_publish_as_the_current_version` |
| 1865 | 三·2 会话：建 inode 之后仍归删空的一版（第 86 批替换行，原文里是 `*current = PoolVersion::of_a_shared_version_of_trees(published);`） | `new_inodes_on_the_emptied_version_are_published_and_the_version_holds_files_again` |
| 1894 | 三·2 第 8 条：会话交回的那几版不记 unlink 的意图那次（第 86 批替换行，原文里是 `Arc::clone(&published)`） | `the_session_hands_in_the_intent_and_every_step_of_an_unlink_in_publish_order` |

以上名字与测试名是我读冻结副本 `crates/mutations.tsv` 第 6912–6917、1865、1894 行第一栏与第六栏抄的。这些行里没有一行的原文含 `make_mut` 或 `unwrap_or_clone`（`grep -c make_mut crates/mutations.tsv` 为 0，`grep -c unwrap_or_clone` 同）。

## T2　内部矛盾：冻结重发与 D16 已定项 17

### T2.1　`with_writes` 的调用方与那一刻的引用数（验收：哪一行代码、几个引用、结论）

`PoolVersion::with_writes`（`crates/singlefs-core/src/transaction.rs:2561` 的 `Arc::make_mut(version).writes = writes;`）是私有函数，全仓只有两个调用点（`grep -rnE 'with_writes\(' crates --include=*.rs` 命中 `transaction.rs:1605`、`:1637` 与定义 `:2557`，没有别处）：

| 调用点 | 路径 | 那一刻 `version` 的引用数 | 依据 | 拷不拷 |
|---|---|---|---|---|
| `crates/singlefs-core/src/transaction.rs:1605`（`persist_the_publish_or_freeze_it` 成功分支） | 带文件 / 有树的一版落盘成功，把写账填进去 | 1 | 唯一能走进这一支 `make_mut` 的调用者是 `publish_version_of_trees_relocating_units_after_a_check`（`transaction.rs:12619` 起，落盘调用在 `:12665`–`:12670`）：传进来的是刚由 `PoolVersion::of_a_version_of_trees(output)` 新装的一份，`output` 是函数里刚建好的 `TransactionOutput`，没有别的句柄；另一个调用者 `transaction.rs:1870`（树表 0 条上写行）装的是 `PoolVersion::WithoutFile(output)`，不走 `make_mut`（`:2557` 的 `WithoutFile` 臂是直接赋值） | 不拷，就地改；随后 `:12674` 的 `.into_version_of_trees()`（`Arc::unwrap_or_clone`）引用数仍为 1，不拷 |
| `crates/singlefs-core/src/transaction.rs:1637`（`resend_the_frozen_publish`，**冻结重发那一支**） | 取走分配器上冻结的那次发布，重发成功后填写账交回 | 1（读码推的；有一条例外路径下为 2，见下） | `allocator.take_frozen_publish()`（`crates/singlefs-core/src/allocator.rs:2092`）把 `Box<FrozenPublish>` 整个搬出来，`version` 是 `FrozenPublish` 的私有字段（`transaction.rs:1536`–`:1543` 的结构定义，`version` 在 `:1539`）。冻结那一刻 `FrozenPublish { writes, version, allocator_after_the_publish }`（`transaction.rs:1598`–`:1602`）里的 `version` 是 `persist_the_publish_or_freeze_it` 的形参，失败分支直接返回 `Err(PublishError::BlockDevice(cause))`（`:1603`），没有把 `version` 交回调用方，所以冻结期间唯一的持有者是分配器上的 `Box<FrozenPublish>` | 不拷，就地改 |

例外路径（引用数 2）：`FrozenPublish` 与 `PoolAllocator` 都派生 `Clone`。若在冻结着的时候有人 `allocator.clone()`，副本里的 `FrozenPublish.version` 与原件指着同一份 `Arc`；之后任一边 `resend` 时 `make_mut` 看到引用数 2，会拷一份再改，另一边那份冻结记录里的 `writes` 仍是「空账」。改前这是深拷，两边同样互不相干，所以价值语义没变。我查了产品代码里全部 `allocator.clone()` 命中（`grep -nE 'allocator[a-z_]*\.clone\(\)' crates/singlefs-core/src/*.rs`，15 处：`instance_table_sweep.rs:670`；`mount.rs:5356`、`:5630`、`:5783`、`:5983`、`:6181`、`:8769`、`:10839`；`mounted_session.rs:1748`、`:3370`；`recovery.rs:6566`；`transaction.rs:1860`、`:1965`、`:12639`、`:13423`）：`transaction.rs:1860`、`:12639` 是发布之前的快照（此时 `frozen_publish` 为空，否则第一道 `refuse_while_a_publish_is_frozen` 已返回，`transaction.rs:1569` 附近）；`mount.rs:5356`、`mounted_session.rs:3370` 之前有 `if let Some(frozen) = allocator.frozen_publish()` 的拒（`mount.rs:5341`、`mounted_session.rs:3328`）；`mount.rs:10839` 之前有同样的拒（`mount.rs:10640`）。**没有逐个追到调用链的**：`mount.rs:5783`、`:5983`、`:6181`、`:8769`、`mounted_session.rs:1748`、`recovery.rs:6566`、`instance_table_sweep.rs:670`——它们是预演 / 挂载期拷贝，读码判断挂载期分配器是新重建的（`allocator.rs:1908` 的字段注释写「只住内存：重开之后走恢复」），推的。推翻它的现象：某条产品路径在 `frozen_publish().is_some()` 时拷分配器并继续 `resend`（本版没有）。

`resend_the_frozen_publish` 在 `crates/singlefs-core/src` 里**没有产品调用方**（`grep -rnE 'resend_the_frozen_publish\(' crates --include=*.rs` 只命中八个验证测试：`write_accounting_per_structure.rs:608`、`discard_after_the_frozen_publish_is_resent_reaches_the_segments_reclaimed_before_the_freeze.rs:293`、`common_images_for_the_given_states_engine/fsync_drop_and_devices_without_the_selected_version.rs:239`、`publish_failure_resent_unchanged.rs:225`、`instance_table_sweep_stops_on_a_block_device_error.rs:432`、`fault_injection_campaign.rs:304`、`rollback_by_a_forward_publish.rs:1010`、`a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs:499`），这八处都没有 `allocator.clone()`（逐个 grep 过，零命中）。调用方拿到 `Option<PoolVersion>` 后是它唯一的持有者。

### T2.2　与 D16 已定项 17 的关系

条款原文（`.claude/kb/decisions/16-发布语义.md`，已读 372–396 行，整节从 `#### 已定项 17` 到下一个 `##`）：

- 第 377 行：「**报错之后不许假设可以照旧复用同一份内存态重试。**（`close(2)` 的 NOTES 段明写「Retrying the close() after a failure return is the wrong thing to do」；本工程取同一条语义。）」
- 射程（同文件第 383 行）：「**只在「转只读」与「重做又失败」那几格上生效。** `D23`（journal 的角色与格式） 已定项 14 失败表判「切换」的那几格，调用方根本收不到错误（实例切换会把在飞 checkpoint 重发掉），这一条对那几格无话可说。」
- 依据里那句 ⚠️（同文件第 391 行）：定案的后两句「被攻过零轮」，用户 2026-09-21 知情接受。

读法：这句管的是**调用方**（fsync 的收到错的一方）能假设什么；冻结重发是发布路径自己的内部机制，条款来源是 `.claude/kb/decisions/23-journal的角色与格式.md:409`（「发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它……重发成功才建下一次发布」）。冻结重发这一支今天的实现，按定义**就是**复用同一份内存态——`FrozenPublish` 里存着发布之后的分配器与装好的版本，重发成功时 `*allocator = allocator_after_the_publish`（`transaction.rs:1635`）、把冻结的 `version` 交出去——这是 D23 已定项 14 要的「原样重发」，不是调用方「假设」。D16-17 的第二句有没有把它也禁掉，要看射程那一句：这一支属不属于「转只读」与「重做又失败」两格，kb 没有逐格写（欠在 C460「失败表的射程没有逐个失败点列表」，`.claude/kb/checks-owed.md:308`）。**这是 kb 本身没定的一格，不是这批代码引进来的。**

本批的改动对这一格的作用，按引用数分两种：
1. 引用数 1（T2.1 两个调用点的常态）：`Arc::make_mut` 不拷，等价于改前的 `version.writes = writes`（改前字段赋值，`transaction.rs` 补丁块里 `-                version.writes = writes;`），重发用的就是冻结时那一份内存态——行为与改前逐字相同。
2. 引用数 2（只有分配器被拷过的例外路径）：先拷一份再改。改前这条路径同样是互不相干的两份深拷，所以「另一边看不到这边写账的更新」改前改后一样；这不是「照旧复用同一份内存态」的新形态——新拷出来的那一份是内容相同的副本，用的仍是同一份冻结内容，没有变成「重新构造一份内存态再发」。

所以不构成矛盾，也没有让一方看到另一方的半截改动：`make_mut` 要么原地（无他人持有）要么拷（有人持有，那人持有的不动）。推翻这一条的现象：存在一条路径在引用数 ≥ 2 时 `with_writes` 之后，另一个持有者**需要**看到 `writes` 更新（那一方改前靠的是共享可变，而改前没有共享，所以不可能）。

### T2.3　与 C381 那一行（`.claude/kb/checks-owed.md:265`）的关系

C381 行末尾新加的一段（第 265 行，grep 命中「分配器整份拷贝换回滚日志那一样挂在这里」）：判决 `research/prompts/m4-gpugc-publish-growth-r1-main-verification.md` 第二节 Q2.3 ③ 判「按调查员的写法做不到——落盘失败、冻结等原样重发那一支（`crates/singlefs-core/src/transaction.rs:1579`–`:1596`）要的是发布之后那一份分配器，回滚日志只给得出发布之前那一份」；用户 2026-11-06 定「和 C381 一起另开一题」，「开那一题时先定冻结重发那一支的分配器从哪来，再判回滚日志能不能做」。

对照本批：
- **分配器那一半，本批一行没动。** `allocator_before_this_publish`、`allocator_after_the_publish` 两个名字在第 86 批改动全文里零命中（`grep -n 'allocator_before_this_publish\|allocator_after_the_publish' research/prompts/_m4-gpugc-shared-bytes-code-r1-diff.md` 无输出）；`transaction.rs:1860`、`:12639` 的整份拷贝（`let allocator_before_this_publish = allocator.clone();`）与 `FrozenPublish.allocator_after_the_publish`（`transaction.rs:1542`）原样在。C381 欠的「分配器从哪来」这一题，本批既没有还，也没有加深。
- **版本那一半**（`FrozenPublish.version`）本批从深拷改成可共享，但冻结那一刻引用数为 1，所以与 C381 要的「发布之后那一份」的取法无关。
- **C381 行里的行号引用 `transaction.rs:1579`–`:1596` 被本批挪了一行**：本批在 `transaction.rs` 开头加了一行 `use std::sync::Arc;`（补丁块 `@@ -11,6 +11,7 @@`），同一段现在是 `crates/singlefs-core/src/transaction.rs:1580` 起（文档注释开头）到 `:1612` 附近；C381 行里的引用是改动前的号。这是 kb 引用漂了一行，不影响结论；列出供主 agent 在落 kb 时一并更新（我不改 kb）。

结论 T2：`with_writes` 冻结重发那一支在 `transaction.rs:1637` 引用数为 1（推的，见「要主 agent 跑的命令」第 1 条的量法），就地改，行为与改前相同；多于一个时拷一份再改，也与改前的深拷等价，不会变成 D16 已定项 17 禁的「照旧复用同一份内存态重试」（那是调用方假设，不是这一支的内部机制），也不会让一方看到另一方的半截改动。与 C381 的关系：互不相干（分配器那一半没动）；唯一的牵连是 C381 行里一个行号引用被挪了一行。

## T3　没改的深拷与共享并存

### T3.1　哪些写位置能改一份 `TransactionOutput`（先把「分叉」要走的路列全）

`grep -rnE '\*current = ' crates/singlefs-core/src/mount.rs` 与对 `raise_the_floor_through`（`crates/singlefs-core/src/mount.rs:5330`）循环体的核读：抬 F 那一串对 `&mut TransactionOutput` 只做**整份替换**（`crates/singlefs-core/src/mount.rs:5651` 的 `*current = next;`，空发布不抬 F 那一支 `mount.rs:5138` 的 `*current = published.clone();`，回退 `mount.rs:10909` 的 `*current = published;`），中间没有对 `current` 的字段赋值（我对 5335–5780 行按 `current.xxx =`、`.push`、`.insert`、`.extend`、`.clear`、`.retain` 扫过，命中只有 5651 一行）。所以一份 `TransactionOutput` 被「就地改」的唯一形态是：被换成新装的下一版。这一点对 T3 要紧：不存在「A 方半截改了一个字段、B 方没跟上」的中间态。

### T3.2　逐处：保留的深拷 × 共享的并存

| 出处（冻结副本） | 保留的深拷 | 与它并存的共享 | 内容能不能分叉 | 结论 |
|---|---|---|---|---|
| `crates/singlefs-core/src/mounted_session.rs:3788`（整理那一趟 `CompactionRun::versions_published`，类型 `Vec<TransactionOutput>`，`:346`） | 走空各步经 `Arc::unwrap_or_clone`：最后一步（引用数 2：列表 + `*current = ...shared...(walked)` 在 `:3786`）拷一份，前面各步（引用数 1）原样取出 | 会话 `current` 是共享 `Arc`；`:4657`、`:4778` 整理各步 `*current = PoolVersion::of_a_version_of_trees(published.clone())` 是另一份深拷（列表里装的是 `*version`，`:3918`） | 整理这一趟里没有对 `current` 的原地改（`*current =` 整份换，抬 F 不在整理的步里：被空间拒就 `Stopped`，不推抬 F）；跑完之后会话再抬 F 经 `make_mut`，改的是 `current` 自己，`CompactionRun` 里按值那一份不动。文档写「最后一个就是会话的 `current`」（`mounted_session.rs:345`）指的是内容相等，今天仍成立，不是同一份 | 无分叉路径；与改前一样（改前也是各自独立的深拷） |
| `crates/singlefs-core/src/mount.rs:5650`–`:5651`（抬 F 那一串 `publishes.push(next.clone()); *current = next;`）与 `:5138`（不抬 F 的空发布） | `RaisedFloor.publishes: Vec<TransactionOutput>` 是按值的深拷 | `current` 是调用方经 `make_mut`（T1.1 五处）拿到的那一份；同一版的另一个持有者可能是版本列表 / 挂载交回值（共享） | 抬 F 之后 `current` 的内容 = `publishes.last()`（两份深拷，内容相等）；别的持有者（版本列表里的共享那份）停在抬 F 之前。两类持有者「该不同」，且改前也不同 | 无新分叉；不存在「一方以为与另一方相同」的读者（T1.0：没有身份比较，内容比较时抬 F 前后的两版本来就不等） |
| `crates/singlefs-core/src/mounted_session.rs:2116`–`:2128`（`version_to_relocate_on = version_after_the_change.clone()`） | 搬迁另发那一批前先深拷现行那一版、在拷贝上推抬 F | 版本列表（`versions_published`）里与旧 `current` 共享的最后一项 | `*current = of_a_version_of_trees(version_to_relocate_on)`（`:2128`）把会话换成拷贝（可能抬过 F）；列表里那一项仍是抬 F 之前的；Published 分支再 `Arc::new(*relocated)` 放进列表并换进会话（`:2134`–`:2136`） | 无分叉；这一处没有用 `make_mut`，用的是显式深拷再整份换，语义与 `make_mut` 同 |
| `crates/singlefs-core/src/mounted_session.rs:1749`、`transaction.rs:12231`、`:15120`、`mount.rs:5784` | 预演用的私有深拷（`rehearsed_version`、`version_rehearsed`、`relocated_version`、`rehearsed_current`） | 无（局部，用完丢；预演的分配器也是拷贝） | 预演不写回会话 | 无分叉 |
| `crates/singlefs-core/src/instance_table_sweep.rs:558`–`:559`（`version.clone()`，现在是 `Arc::clone`，`PoolVersion` 的 `&mut` 臂里 `version: &mut Arc<TransactionOutput>`） | 无（这一处本批变成共享） | 与会话 `current` 同一份 | 只读到发布结束，`*current = ...(sweep_publish)` 整份换（`:691`） | 无分叉。这一处的 `version.clone()` 是隐式的 `Arc::clone`（`grep -n 'Arc::clone'` 不显示），T1.2 表里没有这一行，补在这里 |
| 验证包里为推进 `current` 而 `Arc::make_mut` 的帮手（`crates/singlefs-verification/src/history.rs:8050`、`:8189`；`src/bin/e180_admission_reading_versus_writable_bytes.rs:700`–`:707` 的 `current_version_mut`；`src/bin/e158_root_choice_repair.rs` 补丁块里的三处；多个 `tests/`） | 同上 | 这些帮手手里的 `&mut Arc` 与调用方留着的版本列表 | `make_mut` 拷一份、改自己的；列表不动，等于改前「各拷一份」 | 无分叉 |

### T3.3　单元字节被当 `Vec<u8>` 就地改坏

`PublishedUnit.bytes: Arc<[u8]>` 编译期不给就地改：`Arc<[u8]>` 没有 `DerefMut`，全仓没有对 `bytes` 用 `Arc::make_mut` / `Arc::get_mut`（`grep -rnE 'make_mut\(&mut [a-z_.\[\]0-9]*bytes|get_mut\(&mut [a-z_.\[\]0-9]*bytes|Arc::get_mut' crates --include=*.rs` 零命中）。第 89、90 批改的就地改处，逐处读过，改的都只是被测的那一份：
- `crates/singlefs-verification/tests/container_tree_root_units_read_at_their_recorded_positions.rs:25`–`:28`：`tampered = version.clone()`（单元向量深拷、字节 `Arc` 共享），`decoy.bytes = Arc::from(vec![0; ...])` 是换 `tampered.units[0]` 自己的指针，`version` 不受影响。
- `crates/singlefs-verification/tests/common_images_for_the_given_states_engine/publish_lookup_by_identity.rs:942`、`:1028`：`rebuilt = version.clone()`、`malformed = version.clone()`，各自换自己那一份单元的 `bytes` 指针；`:1027` 那一行注释写明原因（「就地清零会连 version 一起改坏」）。
- `crates/singlefs-verification/tests/model_comparison_judges_the_instance_table_and_allocation_generations_of_a_version_without_a_file.rs:194`：改的是树表 0 条那一版的 `VersionWithoutFilePublishOutput`（`row_publish_of` 取 `PoolVersion::WithoutFile`，`:90`–`:97`），这一成员不是 `Arc`，没有共享。
- `crates/singlefs-verification/tests/zero_unit_publishes_are_compared_with_what_the_row_publish_of_the_same_mount_handed_in.rs:241`：`row_publish_with_a_wrong_row = row_publish_of(...).clone()`，同上两种之一，改自己的拷贝。
- `crates/singlefs-verification/tests/checker_narrow_invariants_and_abandoned_roots.rs:996`、`:1508`：`leaf.bytes.to_vec()`、`data_unit.bytes.to_vec()` 先取出 `Vec<u8>` 拷贝再改拷贝，原单元不动。
- `crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs:714`、`:727`：`std::mem::take(&mut unit.bytes)` 摘掉会话 `current` 里数据单元的字节（先经 `current_version_mut` 的 `Arc::make_mut` 拷版本，`:700`–`:707`），之后原样放回（`:727`）；摘掉的是这个 `Arc` 句柄，不是共享的字节，别的持有者的字节不动。注意 `Arc<[u8]>` 的 `take` 要 `Default`，留下的是空切片 `Arc`；只在摘到放回之间被探针读到。

推翻 T3 的现象：任何一处在 `make_mut` 之后另一个持有者「该变而没变」（或反过来）；任何一处 `PublishedUnit.bytes` 被换成 `Arc<[u8]>` 之后，被测用例读到的是别的用例改过的字节。

### T3 附：一处成本说明（不影响正确性）

T3.1 说明抬 F 那一串对 `current` 只做整份替换，所以 `make_mut` 在引用数 > 1 时拷出来的整份深拷（T1.1 第 1–4 行）随即被 `*current = next` 盖掉；也就是说这四处在引用数 > 1 时多拷了一次没用的整版（抬 F 被拒在第一次发布之前时更是白拷）。这与「改成共享省拷贝」的目的有出入，但没有语义影响；推的，没量过，量法见「要主 agent 跑的命令」第 1 条（`strong` 读数 ≥ 2 的就是白拷的次数）。

## T4　第 88 批：去重与改名改没改行为

补丁目录 `m4-gpugc-name-collisions-writer/patch/`，改前原文在它的 `baseline/`（8 份，sha256 与 `apply-outcome.tsv` 里「改前」一栏逐份相同：`adapter_inventory.rs` e6063ba0…、`background_gpu_compute.rs` 2f47a523…、`checksum_kernels.rs` 1cef5190…、`gpus_of_this_host.rs` a7b358fe…、`live_slot_kernels.rs` b1a283b0…、`region_record_kernels.rs` f74d1602…、`two_kernel_reconciliation.rs` da66718b…、测试 `…cpu_backed_test_double.rs` 2833a97f…，我 `sha256sum` 过）。冻结副本里被判的 13 份文件的 sha256 与主工作区逐份相同（我 `sha256sum` 比过，13 份都 same）。

### T4.1　去重项逐项（验收：每项一行）

| # | 去重项 | 两份原文逐字相同吗 | 去重之后 | 结论 |
|---|---|---|---|---|
| 1 | `low_and_high_words_of_the_slot`（产品代码） | **是。** 改前 `live_slot_kernels.rs:72`–`:77` 与 `region_record_kernels.rs:234`–`:239`，从 `fn` 到收尾 `}` 取出来逐字节比（python 取正则 `fn low_and_high_words_of_the_slot\(.*?\n\}\n`）：长度都是 224，sha256 都是 `ae97dc4a1d4fef9069fb42a149ed48b70eed5268062e4d83017e3775376eedea`；两边 `SlotNumber` 同一个类型（`use singlefs_core::address::SlotNumber`，`live_slot_kernels.rs:24` 与 `region_record_kernels.rs:53` 的 `use singlefs_core::address::{CheckpointTxg, SlotNumber};`） | 留 `crates/singlefs-background-gpu/src/live_slot_kernels.rs:73` 一份，改成 `pub(crate)` 并加一行文档注释；`region_record_kernels.rs:66` `use crate::live_slot_kernels::low_and_high_words_of_the_slot;`，删掉自己那一份。三处调用（`region_record_kernels.rs:261`、`:354`、`:395`）名字没动 | 行为不变（函数体逐字相同、类型相同） |
| 2 | `GIBIBYTE`（仅测试） | **是。** 改前 `adapter_inventory.rs:339` 与 `gpus_of_this_host.rs:961` 都是 `    const GIBIBYTE: u64 = 1 << 30;`（逐字相同） | `adapter_inventory.rs:335`–`:337` 新加 `#[cfg(test)] pub(crate) const GIBIBYTE: u64 = 1 << 30;`；`gpus_of_this_host.rs:956` 的 `use` 引它。值与类型不变，所有用处（`32 * GIBIBYTE` 等）取值相同 | 值逐个相同 |
| 3 | `EIGHT_GIBIBYTES`（测试目标 `background_gpu_compute_agrees_with_the_cpu_backed_test_double`） | **是。** 改前该测试文件 `const EIGHT_GIBIBYTES: u64 = 8 << 30;` 与 `crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs:34` 的 `pub const EIGHT_GIBIBYTES: u64 = 8 << 30;`，值与类型都相同（`pub` 只差可见性） | 测试文件 `:18` 改成 `use common_background_gpu_compute::{CpuBackedBackgroundGpuComputeForTests, EIGHT_GIBIBYTES};`，删本地那一行；该共用模块是经 `#[path]` 引进来的（测试文件 `:10`–`:11`），共用模块本批没改（sha256 61b16ade…） | 用处（`:91`、`:753`、`:1019`、`:1064`、`:1112`）取值相同。顺带：`crates/singlefs-background-gpu/src/background_gpu.rs:2677` 还有一份私有的 `const EIGHT_GIBIBYTES: u64 = 8 << 30;`，在 lib 的测试模块里，不同编译目标，不是本批动的；撞名格按目标判，不在我这一格 |

其余 7 个名字是**改名**（丙类），不是去重：两份内容本来不同，各自换了名字，没有合并。我把 8 份文件改前 / 改后逐 hunk 比，按各文件的改名表把改前文本套上改名后与改后文本去空白再去尾逗号比较（`difflib.SequenceMatcher` 取 hunk；改名表来自 `patch/result.md`「改了什么」一表）：`background_gpu_compute.rs`、`checksum_kernels.rs` 零个解释不了的 hunk；`gpus_of_this_host.rs` 只剩 `use` 里加 `GIBIBYTE` 与删本地常量两处（上面第 2 项）；`live_slot_kernels.rs` 剩第 72 行那一处（`pub(crate)` 与注释，产品代码，见上）；`region_record_kernels.rs` 剩 `use` 一处插入与旧函数一处删除；`adapter_inventory.rs` 剩新增 `GIBIBYTE` 那一处与 `WAIT_LIMIT_…` 重折行；`two_kernel_reconciliation.rs` 剩三处 `|_entry_position| { panic!(…) }` 被 rustfmt 去掉花括号（`panic!` 作闭包体，语义同）。**所以改名没带出别的内容变化。**

### T4.2　WGSL 源串与入口名

- 补丁 8 个文件里没有 `.wgsl`（`grep -c wgsl patch/crates.patch patch/apply-outcome.tsv` 两份都是 0）；两份核的 WGSL 来自 `include_str!("….wgsl")`（`live_slot_kernels.rs:44`–`:50`；`region_record_kernels.rs:76`–`:82`），`include_str!` 的文件名字面串与 `*_ENTRY_POINT` 常量那几行，改前（`baseline/…/live_slot_kernels.rs:44`–`:50`、`region_record_kernels.rs:75`–`:81`）与改后逐行相同，只是 `region_record_kernels.rs` 因为多了一行 `use`、少了 7 行函数而移了行号。
- 非测试代码里这一批只动了三处（我拿每份文件改前 / 改后 `diff` 的 hunk 起止行与第一个 `#[cfg(test)]` 的行号比）：`live_slot_kernels.rs:72`–`:73`、`region_record_kernels.rs:66`（加 `use`）与删 `:234`–`:240`（旧函数）；`adapter_inventory.rs:335`–`:338` 是 `#[cfg(test)]` 常量。别的 hunk 全落在各文件第一个 `#[cfg(test)]` 之后。WGSL 是经 `include_str!` 读进来的，`low_and_high_words_of_the_slot` 是主机侧打包槽号的函数，不在 `.wgsl` 里；所以 WGSL 源串与入口名不变。
- 单元图的闭包（判法单元键）：单元图按**函数单元**收闭包（`crates/singlefs-unit-graph/src/closure.rs` 文件头「R-裸名」「R-路径」），不按文件。`low_and_high_words_of_the_slot` 那个单元改后的词法单元串多了 `pub(crate)`，所以以它为成员的闭包（两份核的主机侧打包入口）键会变，被去掉的那份不再进任何闭包；这是 `pub(crate)` 与删除带来的**预期**键变化，不是行为变化。键具体怎么变我没算（跑 `unit-graph` 要编）：推的，没量过，命令见「要主 agent 跑的命令」第 4 条。
- 可见性扩大：`pub(crate)` 只在 `singlefs-background-gpu` 这一个包内可见；`grep -rn 'low_and_high_words_of_the_slot' --include=*.rs crates` 在两份核之外零命中（上面第一块命令，输出为空）；`live_slot_kernels` 本身是 `pub mod`（`lib.rs:29`），而该函数是 `pub(crate)`，包外调不到。包内两个 `use`/调用点都是本批要的。没有别的模块误用。

推翻 T4 的现象：(a) `patch/baseline/` 里两份 `low_and_high_words_of_the_slot` 取出来 sha256 不同（我取到相同）；(b) 任何一处改后 WGSL 的 `include_str!` 文件名或入口名字面串与改前不同（我比到相同）；(c) 第一个 `#[cfg(test)]` 之前出现了我列的三处之外的 hunk（没有）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 读身份 | 一致（声称的对） | 产品代码里没有 `ptr_eq`、`strong_count`、`get_mut`、`unsafe`；`ptr_eq` 只在本批新加的测试里；`Arc<TransactionOutput>` 的 `==` 对 `Eq` 类型走标准库「同一份直接相等」快路，`Eq` 自反，只省时间（std 源码本机核不了，推的） |
| T1 `make_mut` 五处 | 五处「照旧」 | 每处拿 `&mut Arc` 的一方改自己的一份，别的持有者（版本列表 / 挂载交回值）不动，与改前「各拷一份」等价；没有变异行点名 `make_mut`（0 行），间接钉住与否待量（命令 2） |
| T1 `Arc::clone` / `unwrap_or_clone` | 一致 | 共享处有 6 行变异行钉（`crates/mutations.tsv` 6912–6917）；整理那一趟的 `unwrap_or_clone` 与 `with_writes` 后 `into_version_of_trees` 无变异行钉 |
| T1 发现 | 文档与代码不一致（不影响行为） | `mount.rs:1102`–`:1103` 的字段注释把「写行那一版与暖机各版」都说成与现行那一版共享，只有最后一个共享，本批自己的测试 `mount.rs:14427` 断言了反面 |
| T2 引用数 | 一致 | `with_writes` 两个调用点：`transaction.rs:1605`（新装的一份，引用数 1）、`:1637`（冻结重发，引用数 1，分配器被拷过时为 2，产品里没找到这条路）；产品里无 `resend_the_frozen_publish` 调用方，八个调用都在验证测试 |
| T2 与 D16 已定项 17 | 一致（不矛盾） | 已定项 17 管调用方的假设，冻结重发是 D23 已定项 14 的内部机制；引用数 1 时与改前逐字相同，2 时拷一份与改前的深拷等价 |
| T2 与 C381 | 无关（分配器那一半没动） | 改动全文里 `allocator_before_this_publish`、`allocator_after_the_publish` 零命中；唯一牵连是 C381 行里引的 `transaction.rs:1579`–`:1596` 因多一行 `use` 漂了一行 |
| T3 | 一致 | 保留的深拷（整理那一趟按值、抬 F 的 `publishes`、搬迁的 `version_to_relocate_on`、预演拷贝）与共享并存时，对 `TransactionOutput` 只有整份替换、没有字段级原地改，也没有身份比较，找不到内容分叉路径；`Arc<[u8]>` 的就地改处都是换自己那份的指针或 `to_vec` 拷贝 |
| T3 成本 | 发现 | 抬 F 只整份替换 `current`，所以 `make_mut` 在引用数大于 1 时拷的整版随即被盖掉（白拷一次），不影响语义（推的，命令 1 量） |
| T4 去重三项 | 一致 | `low_and_high_words_of_the_slot`、`GIBIBYTE`、`EIGHT_GIBIBYTES` 两份原文逐字相同（前者 sha256 同为 `ae97dc4a…`） |
| T4 WGSL | 一致 | 补丁里没有 `.wgsl`；`include_str!` 文件名与入口名字面串改前后逐行相同；第一个 `#[cfg(test)]` 之前只动了三处 |
| T4 闭包键 | 待主 agent 跑 | 单元图按函数单元收闭包，`low_and_high_words_of_the_slot` 那个单元多了 `pub(crate)`，相关键预期会变；具体怎么变没算（命令 4） |

## 要主 agent 跑的命令

我没往统一任务服务交任务，没起 cargo；写了两个只做拷贝与文本替换的草稿脚本（不是模型，没有 `rerun.sh`），sha256：
- `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/tools/make_copies.py`　`bed382472fb9628bfcc49709d923face24bed64b5ef16a318904effbd7b7f0fc`
- `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/tools/make_pre_post_88.py`　`58e6787d2bf7270627a8b2b0a871374446d8dce954af1c4b191d757198b4c737`

`FROZEN` = `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo`，`D` = `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet`。冻结副本只读，下面所有写和编译都在 `D` 下的副本里。

**0. 先造副本（只拷贝与替换文本，不编）**。起在仓根：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet -- python3 /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/tools/make_copies.py --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo --out /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet -- python3 /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/tools/make_pre_post_88.py --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1/repo --baseline /home/fy5090/.local/share/singlefs/work/m4-gpugc-name-collisions-writer/patch/baseline --out /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies
```

看：两条都 `exit=0`；第一条日志有 6 行 `ok  命中 1 次`、`mut-1` … `mut-5` 与 `probe-copy: 五处探针`；第二条日志是 `pre-88：盖回 8 份改前原文`。（我已在冻结副本上跑过 `--check`：6 行都是 `ok  命中 1 次`。）用完 `task-server/task clean <号>`，`/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies` 由我认领的草稿目录清理时一并删（约 8 份 × 50 MB）。

**1. 量五处 `make_mut`（与冻结重发那一刻）的引用数**（T1.1、T2.1、T3 附）。起在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/probe-copy`，`--cwd` 指那里；每条前面加范围声明 `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 <名>（引用数探针副本）; 范围=1 个测试目标; 为什么=核 Arc::make_mut 那一刻的引用数'`：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/probe-copy -- cargo test -p singlefs-verification --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount -- --nocapture
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/probe-copy -- cargo test -p singlefs-verification --test admission_raises_the_floor_before_refusing -- --nocapture
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/probe-copy -- cargo test -p singlefs-verification --test write_accounting_per_structure -- publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up --exact --nocapture
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/probe-copy -- cargo test -p singlefs-verification --test a_publish_returns_only_after_the_system_configuration_rotation_is_durable -- --nocapture
```

看：`task-server/task log <号> --all` 里 `REFCOUNT site=…` 行。我推的读数：`mount.rs:4610`（刚挂载完就卸载时）≥ 2；`mount.rs:8708` ≥ 2（一定与写行或最后一次暖机共享）；`mount.rs:8335`、`mounted_session.rs:5079` 发成过步之后 ≥ 2，一轮抬 F 之后紧接着又被拒时为 1；`transaction.rs:2561` 在 `:1605` 那条路上为 1；`transaction.rs:1637 resend` 那行（`resend before with_writes strong=`）为 1。任一读数与此不符就是我的读法错了，以量的为准。推翻 T2 的读数：`transaction.rs:1637 resend` 那行 `strong` ≥ 2。测试都过（`test result: ok`）才算探针没改变行为；探针只加 `eprintln!`。

**2. 五处 `make_mut` 有没有被现有测试钉住**（T1 小结）。起在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/mut-N`（N = 1…5，对应 `mount.rs:4610`、`mount.rs:8335`、`mount.rs:8708`、`mounted_session.rs:5079`、`transaction.rs:2561`）。每个副本把对应一处 `Arc::make_mut(x)` 换成「改一份临时拷贝」（`&mut TransactionOutput::clone(&…)`，抬 F / 写账改在临时拷贝上、现行那一份不动）。五条：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/mut-1 -- cargo test -p singlefs-verification --no-fail-fast --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount --test admission_raises_the_floor_before_refusing --test write_accounting_per_structure --test a_publish_returns_only_after_the_system_configuration_rotation_is_durable
（mut-2 … mut-5 同上，只换 --cwd）
```

范围声明同上，`跑什么` 写「变异副本 mut-N」。看：`exit=0` = 这个变异被放过（没有任何一条用例钉住那一处 `make_mut` 落在 `current` 自己身上），`exit≠0` 并且日志里有 `FAILED` 行 = 被钉住，记下是哪条用例。放过的变异说明 T1 的「照旧」只由读码支撑、没有测试保护；这时按判决自己的口径决定要不要补一行变异（我不替主 agent判）。编译错不算红：看日志是 `error[E…]` 还是用例 `FAILED`。

**3. 保留的深拷（T3）**：无单独命令；T3 的结论靠读码 + 命令 2 的放过 / 抓到。

**4. 第 88 批单元图闭包键前后对比**（T4.2）。起在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/pre-88` 与 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/post-88`：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/pre-88 -- cargo run -p singlefs-unit-graph --bin unit-graph -- dump . --judgment-kind product-behavior
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-shared-bytes-code-r1-sonnet/copies/post-88 -- cargo run -p singlefs-unit-graph --bin unit-graph -- dump . --judgment-kind product-behavior
```

范围声明：`种类=target; 跑什么=singlefs-unit-graph 的 dump（第 88 批改前 / 改后树）; 范围=1 个二进制; 为什么=核 low_and_high_words_of_the_slot 去重后单元图单元与指纹怎么变`。看：两份日志 `task-server/task log <号> --all` 里 `grep low_and_high_words_of_the_slot` 的行：改前应有两行（分属 `live_slot_kernels.rs` 与 `region_record_kernels.rs`），改后一行且指纹变了（`pub(crate)`）；其余单元的指纹行除这两份文件里的测试改名之外应逐行相同（`diff` 两份日志，差异只落在 `live_slot_kernels.rs`、`region_record_kernels.rs`、`adapter_inventory.rs`、`gpus_of_this_host.rs`、`background_gpu_compute.rs`、`checksum_kernels.rs`、`two_kernel_reconciliation.rs` 与那个集成测试文件）。`dump` 的输出格式我没读过（`crates/singlefs-unit-graph/src/output.rs`），行形若与我写的不同，以实际输出为准。

另外：交回前我跑了 `python3 scripts/agents/cite-check.py`，结果见下一节。

## 没做什么

- 没替主 agent 采纳或出判决；没往统一任务服务交任务，没起 cargo，没跑任何门禁阶段；判决要引的数（引用数读数、变异放过 / 抓到、单元图键）都在「要主 agent 跑的命令」里，主 agent 复跑之前一律当「推的」。
- 没读禁读清单里的任何一份（另一条腿的报告与草稿目录、主 agent 的判决）。
- 没量运行时引用数；T1.1、T2.1 的持有者数是读码得出的，标了「推的」。
- 没核 `Arc<T: Eq>` 的 `==` 快路（本机无 rust-src），没核 `dump` 的输出行形。
- 没逐个追 15 处 `allocator.clone()` 的调用链（T2.1 里点名了没追的 7 处）。
- 没读 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」与「记账是事务的副产品，不是事后的遍历」两节，也没读 `milestone/04-fourth-txn.md` 第七节（背景材料点名的压着的条款里，我只核了 D16 已定项 17、C381、D23 已定项 14 的失败处置那一句）；T1–T4 四格的结论没有用到那几节。
- 没碰 kb；C381 行里漂了一行的 `transaction.rs:1579`–`:1596` 引用只在报告里指出，没改。
- 没写模型，所以没有 `rerun.sh`。
