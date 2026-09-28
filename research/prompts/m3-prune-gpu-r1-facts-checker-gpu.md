# m3-prune-gpu-r1 事实调查：池级 checker 流水线、可 GPU 化部分、E163 / E161 可复用件

调度记录 `records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 任务乙。只读代码、kb、产物；没编译、没跑任何测试。行号都是 `grep -n` / `sed -n` 现取的文件自己的行号。标「推的」的是读代码推出来、没量过的。

## 1　池级 checker 与记录核对器依次做什么、分三类

### 1.0 每个崩溃状态上的流水线（层 0）

`crates/singlefs-checker-tier/src/crash.rs:1131` `evaluate_state_recording_findings` 对一个状态依次做：

| 次序 | 调用 | 行 | 读什么 |
|---|---|---|---|
| 1 | `newest_persisted_root(writes, &persisted)` | crash.rs:1141 | 写表 + 持久集合 |
| 2 | 组 `CrashImage { base, writes, persisted }` | crash.rs:1142-1146 | 不拷字节，读时现叠 |
| 3 | `recover(&image, JournalPolicy::Consult)`、`recover(&image, JournalPolicy::Ignore)` | crash.rs:1147-1148 | singlefs-core 的恢复，两遍 |
| 4 | `classified_oracle_violation_for_versions` 两遍（看 / 不看 journal） | crash.rs:1168、1190；函数在 crash.rs:968 | 恢复结果 vs 录制时发布过的版本（oracle） |
| 5 | `check_pool_image(&image)` | crash.rs:1210 | 池级 checker，一元谓词 |
| 6 | `check_records(&image, consulted.effective_root)` | crash.rs:1247；函数在 crash.rs:48 | 记录核对器，四元入参 |

`CrashImage` 的读法：每次 `read` 先从基线池读、再把写表里每一条持久了的写逐条叠上去（`crates/singlefs-harness/src/memory_pool.rs:445-469`，循环 `for (write, is_persisted) in self.writes.iter().zip(&self.persisted)`），即每次读是 O(写表长 W)（推的：按代码结构，没量过）。扫描方向的候选槽 = 基线写过的扇区所在槽 ∪ 持久了的 `UnitWrite` 所在槽（memory_pool.rs:741-751），journal 候选同理（memory_pool.rs:752-770）。

### 1.1 池级 checker `check_pool_image`（`crates/singlefs-checker/src/walk.rs:5656-6255`）依次做的事

报 49 条不变量（`crates/singlefs-checker/src/image.rs:68` `IMPLEMENTED_INVARIANTS: [&str; 49]`）。按函数里的次序：

| # | 步骤（行） | 调用 | 判的不变量 | 类 | 每状态数据量（出处） |
|---|---|---|---|---|---|
| 1 | walk.rs:5657 | `chosen_system_configurations` → `verified_system_configuration_slots` → `system_configuration_slot_readings`（image.rs:571 / 487 / 456）→ `check_system_configuration_slot`（lib.rs:245）+ `geometry_of`（image.rs:241） | 择槽；槽内 magic、整槽 CRC、格式版本、incompat、加密类型、几何上下界 | a | 每盘 2 槽 × 4096 字节；整槽 CRC 用按位的 `crc32_castagnoli_bitwise`（lib.rs:89 经 `checksum_field_holds`） |
| 2 | walk.rs:5676-5689 | `judge_system_configuration_values_the_reader_accepts`（image.rs:510），再调一次 `system_configuration_slot_readings` | I-7.13 | a（逐槽） | 同上，再读一遍 |
| 3 | walk.rs:5701-5712 | 各盘择到的 fsid 逐盘比 | I-1.4 | b（跨盘） | 标量 |
| 4 | walk.rs:5728-5732 | 根环区域设备分布 | I-7.6 | b | 标量 |
| 5 | walk.rs:5738 | `valid_roots`（image.rs:613）→ `check_root_slot`（lib.rs:318） | 根槽 magic、整槽 CRC（按位）、fsid、flags | a | R × S 个根槽 × `physical_block_size` 字节（root_slot_positions，image.rs:593） |
| 6 | walk.rs:5744-5756 | `verified_system_configuration_slots` 第三次读系统配置槽 | — | a | 同 1 |
| 7 | walk.rs:5757 | `judge_own_device_number_is_the_device_identity`（image.rs:549） | I-7.14 | a（逐槽比盘身份） | 标量 |
| 8 | walk.rs:5761 | `judge_system_configuration_floor_against_the_roots_on_each_device`（walk.rs:4118） | I-7.12 | b（系统配置 F vs 同盘根） | 标量 |
| 9 | walk.rs:5767 | `judge_instance_carriers`（walk.rs:2529）→ 第四次读系统配置槽（walk.rs:2535）+ `instance_carriers`（walk.rs:2470）：重读全部根槽、整条 journal 候选槽逐条 `check_journal_record`、单元区候选槽逐槽读 4096 字节头 `unit_write_order_instance`（walk.rs:2440） | I-7.7 | b（跨全盘取最大号） | journal 每槽 4096 字节（整条 CRC 按位）；单元头每槽 4096 字节（`UNIT_HEADER_SCAN_BYTES`，walk.rs:2436），头 CRC 按位 |
| 10 | walk.rs:5769-5770 | `scanned_journal_records_by_device`（walk.rs:4996）→ `scanned_journal_records_of_device`（walk.rs:4848）：每条候选记录 `check_journal_record`（lib.rs:733）+ `back_chain_of_record_header`（lib.rs:726） | 记录自身：magic、`header_csum` 罩整条 4096（按位）、类型、长度、fsid、载荷 CRC（按位）、点名项 flags | a（逐条） | 每条 4096 字节，两次按位 CRC（整条 + 载荷）+ 311 字节头一次按位 CRC |
| 11 | walk.rs:5771-5781 | `judge_journal_back_chain`（5027）、`judge_location_order_of_journal_named_entries`（4901）、`judge_transaction_numbers_per_instance`（5099）、`judge_commit_markers_per_transaction`（5183）、`judge_publish_ordinals_and_last_record_flags`（5316） | I-8.6、I-2.5（点名项那一半）、I-8.7、I-8.8、I-8.9 | b（记录之间；I-2.5 那一半逐条，a） | 记录解出来的标量 |
| 12 | walk.rs:5782-5792 | 根环非空；`judge_root_ring_health`（4772） | I-7.1、I-7.3 | b | 标量 |
| 13 | walk.rs:5800-5808 | `Walk::starting_with` + `walk_root`（walk.rs:854）走最新根 | 见 1.2 | b（主体） | 见 1.2 |
| 14 | walk.rs:5820-5830 | 最新根走读有没有失败、有没有 I-2.1 违例 | I-4.8、I-7.4（最新根那一格） | c（候选集里的一条） | 标量 |
| 15 | walk.rs:5837-5897 | 按最新根指着的实例表剔被抛弃的根、按 F 生效值剔 F 之下的根，定候选集 `candidate_indexes`；F 生效值取各盘最新有效根的 F 与系统配置 F 的最大值（5866-5874） | — | c（定回退候选集） | 标量 |
| 16 | walk.rs:5898-5921 | 候选集里其余每条根 `walk_root(.., false)`，每条各判一格 | I-7.4、I-4.8（每条候选根），走读内部照判 1.2 那些 | c | 每条候选根一遍走读（`visited_units` 共享，走过的单元不重走，walk.rs:262-265 注释） |
| 17 | walk.rs:5925-5951 | `versions_applied_only_by_records`（5507）：由 journal 记录施加出来、根槽没落盘的那几版，每版 `walk_version_applied_only_by_records`（890） | I-7.4、I-4.8（每版） | c | 同 16 |
| 18 | walk.rs:5952-5960 | `judge_blocks_referenced_by_abandoned_roots`（5582）：重读全部根槽判读不读得出；每条被抛弃的根另起一个 `Walk` 走一遍 | I-7.4（被抛弃时间线那一半） | c | 每条被抛弃根一遍完整走读（不共享 visited） |
| 19 | walk.rs:5979-5991 | 最新根走读断没断；被引用单元数为 0 时报不适用 | I-7.2；I-1.2、I-4.2 的不适用 | b | 标量 |
| 20 | walk.rs:5994-6006 | `scanned_content_units`（4625）：单元区候选槽逐槽读 4096 字节头 `content_unit_of_header`（4572，头 CRC 按位 + fsid + 已发布谓词）；`judge_merged_version_total_order`（4672）按类身份段归并成组，组内载荷校验和字段不同时才 `payload_checksum_holds_on_disk`（4654，整单元查表 CRC） | I-1.8 | b（跨单元归并） | 单元头每槽 4096 字节；整单元只在组内不一致时读 |
| 21 | walk.rs:6008-6048 | `IndexNodeCache`（`BTreeMap<(u32, u64, u32), Option<IndexNodeView>>`，walk.rs:2762）共用；`judge_release_generation_and_tree_table_birth`（3395）、`judge_allocation_records_disjoint`（3457）、`allocation_record_node_pointers_of_the_candidate_versions`（3586）、`judge_allocation_generations_against_unit_births`（3634）、`judge_rollback_floor_raises_against_their_ceilings`（4024） | I-3.9、I-9.14、I-5.4、I-3.10、I-7.9 | c（按候选集里每条根读树表与树节点） | 按候选根读节点，缓存按 (盘, 槽, 校验和) 去重；读节点用 `read_index_node_without_judging`（2764）→ `index_node_view`（lib.rs:447），CRC 按位 |
| 22 | walk.rs:6053-6074 | 记账里的 inode 号水位 vs 遍历侧最大 inode key | I-9.6 | b | 标量 |
| 23 | walk.rs:6075-6089 | inode 树走没走到；数据单元对象出生代 vs inode 记录 | I-9.12 不适用、I-9.10 | b | 标量表 |
| 24 | walk.rs:6091-6107 | `references` 按盘排序、相邻两段不重叠 | I-5.1 | b | 引用条数 |
| 25 | walk.rs:6131-6229 | `slots_referenced_per_device`（5644）；`quarantined_slots_exempted_per_device`（4244）两遍（全部走过的版本 / 只最新根）；记账行 vs 遍历和、vs 单元区容量、减 defer | I-3.1、I-5.2、I-3.11 | c（I-3.1 对候选集并集；I-3.11 对最新根）；I-5.2 是 b | 记账行 + 引用表 |
| 26 | walk.rs:6232-6253 | `scanned_tree_identifiers`（2396）：单元区候选槽逐槽读 **16384 字节**（`node_bytes()`，walk.rs:2410）判码 2 头 CRC（按位）+ 诞生代 + 实例表；与走过的树表树 ID 并起来取最大，对根环水位 | I-7.8 | b | 单元区候选槽每槽 16384 字节 |

⚠️ 单元区候选槽整轮扫三遍（第 9、20、26 步），系统配置槽读四遍（第 1、2、6、9 步），根槽读三遍（第 5、9、18 步），journal 候选槽读两遍（第 9、10 步）。「几遍」是数代码里的调用点，推的，没在跑的时候数过。

### 1.2 走读（`Walk`，walk.rs:253）里每读一个被引用单元做的事

每跟一条指针（例：码 2 节点 `read_index_node`，walk.rs:790-852）：

| 次序 | 调用（行） | 判的 | 类 | 数据量 |
|---|---|---|---|---|
| 1 | `parse_node_pointer`（image.rs:669）；`judge_location_order`（image.rs:710）；`judge_pointer_mac_and_nonce_are_zero`（image.rs:699） | I-2.5、I-2.4（指针头 MAC 16 + nonce 12 恒 0） | a（只看父单元里那 86 字节指针） | 每指针 86 字节 |
| 2 | `note_references_of_a_pointer_followed_by_the_walk`（walk.rs:633）→ `note_reference`（561）、`judge_a_placement_referenced_in_this_version`（588） | I-5.1（同一版里同一落点被引两次） | b | 引用表 `BTreeMap<(u32, u64, u64), String>` |
| 3 | `read_referenced_unit`（image.rs:734）：两条位置条目各读一份整单元，`crc32_castagnoli_table` 比位置条目里的校验和（image.rs:746） | I-2.1 | a（一份单元字节 + 父指针里的 4 字节校验和） | 每指针 2 × 16384 或 2 × 32768 字节，查表 CRC 两次 |
| 4 | `visited_units.insert`（walk.rs:818 `if !self.visited_units.insert(placement)`）：走过的不再往下走 | — | b | — |
| 5 | `judge_unit_header`（walk.rs:663）：类标签 / flags / 类身份段首字节（I-1.6）；头校验和罩 [0, 头末)（I-2.4，`checksum_field_holds` 按位，walk.rs:700）；格式版本（I-2.4）；声明长度之后补齐为 0 + 载荷 CRC（I-2.3，查表，walk.rs:725）；头里 fsid vs 池 fsid（I-1.4）；29 字节加密预留位恒 0（I-2.4） | I-1.6、I-2.4、I-2.3、I-1.4 | a（池 fsid 是全池一个标量入参） | 头 ≤ 625 字节按位 CRC + 载荷整段查表 CRC |
| 6 | `judge_birth_identity_of_a_referenced_unit`（walk.rs:482） | I-1.2、I-4.2（按挂载根与实例表的已发布谓词） | b（要最新根的实例表） | 头里几个字段 |
| 7 | `index_node_view_judging_a_zero_entry_width`（walk.rs:761）→ `index_node_view`（lib.rs:447）→ `check_unit`（lib.rs:374）：**再判一遍**头校验和（按位）与载荷 CRC（**按位**，lib.rs:409），再核声明长度 = 条目数 × 条目宽、条目宽 ≥ key 宽、条目宽 0 时条目数 0；切出 `entries: Vec<Vec<u8>>` | I-1.10（条目宽 0 那一格） | a | 同一单元整段再按位 CRC 一次 |
| 8 | 头里树 ID vs 引用它的树（walk.rs:832） | I-1.3 | b（期望值来自父） | 标量 |
| 9 | `check_index_node_keys`（lib.rs:567）或 `check_internal_node_separators`（lib.rs:590）：key 宽 = 形态宽、条目 key 按字段序严格递增、首末条目贴紧头里 key 区间 | I-1.1 | a（形态由树的种类定，是父传下来的参数） | 条目数 × key 宽 |
| 10 | 子树覆盖区间、分隔 key 与孩子区间（`walk_code_two_subtree` 1107、`judge_separators_against_the_inode_children` 1746）、条目宽 = 字段表宽（`entry_width_holds` 1253、`position_and_entry_width_hold` 2003）、树表条目次序（`judge_tree_table_entries_ordering` 2611）、实例表行（`judge_instance_table_rows` 1874）、inode 树（1405-1780）、打包容器（`judge_packed_container` 1913）、中央映射 key vs 单元头（`walk_central_mapping_entries` 987，判在 1000、1025、1052、1066、1082）、extent 与分配记录树（2041-2376） | I-1.1、I-1.7、I-1.10、I-1.11、I-3.8、I-9.1、I-9.2、I-9.4、I-9.7、I-9.12、I-9.13、I-9.15、I-9.16 | b | 父子两层或整棵树 |

码 2 节点一次被引用，整单元 CRC 共算三遍：第 3 步查表（整单元）、第 5 步查表（载荷）、第 7 步按位（载荷，经 `check_unit`）；头校验和按位算两遍（第 5、7 步）。推的：数的是代码路径，没计时。

### 1.3 按不变量归类（49 条）

归法：a = 只看单个单元（或单个槽、单条记录、单条指针）的字节，外加全池一两个标量（池 fsid）就能判；b = 要跨单元、走树、跨盘、跨记录；c = 要按回退候选集、记账对遍历并集这类「集合对集合」判（池级 checker 里没有 oracle，oracle 在 crash.rs 那一侧）。一条不变量在几处判的，按它最重的那一处归。

| 类 | 不变量（判它的函数） |
|---|---|
| a | I-2.1（`read_referenced_unit`，但「对哪几条根判」是 c）、I-2.3、I-2.4、I-1.6（`judge_unit_header`）、I-2.5（`judge_location_order` / `judge_location_entries_order` / `judge_location_order_of_journal_named_entries`）、I-7.13（`judge_system_configuration_values_the_reader_accepts`）、I-7.14（`judge_own_device_number_is_the_device_identity`）；外加不单列编号、走读前提的「单元自证」：`check_unit`、`check_root_slot`、`check_system_configuration_slot`、`check_journal_record` |
| b | I-1.1、I-1.2、I-1.3、I-1.4、I-1.7、I-1.8、I-1.10、I-1.11、I-3.8、I-4.2、I-5.1、I-5.2、I-7.1、I-7.2、I-7.3、I-7.6、I-7.7、I-7.8、I-7.12、I-8.6、I-8.7、I-8.8、I-8.9、I-9.1、I-9.2、I-9.4、I-9.6、I-9.7、I-9.10、I-9.12、I-9.13、I-9.15、I-9.16 |
| c | I-3.1、I-3.9、I-3.10、I-3.11、I-4.8、I-5.4、I-7.4、I-7.9、I-9.14（第 15-18、21、25 步） |

按函数名 grep 各不变量字符串所在函数（`awk` 取每个 `"I-x.y"` 所在的 `fn`）现查过；I-1.11、I-7.13、I-7.14 用的是常量名（image.rs:40、50、57），按常量名查。

### 1.4 记录核对器（`crates/singlefs-checker-tier/src/crash.rs`）

入参（crash.rs:41 文档注释，那一行的前半逐字）：「记录核对器（D13（验证路线） 已定项 7，故意不给它编号；入参 (崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)）：崩溃前镜像是 `image.base`、记录流是 `image.writes`、崩溃后镜像是 `image` 本身、持久集合是 `image.persisted`」。D13 已定项 7 原文在 `.claude/kb/decisions/13-验证路线.md:133`（「需要第二个输入的核对归记录核对器，入参 `(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)`」起那一行），射程在 :135（「按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）」在那一行里）。

| 次序 | 调用（行） | 判什么 | 类 | 数据量 |
|---|---|---|---|---|
| 1 | `check_records`（crash.rs:48）→ `check_records_against`（crash.rs:105），层 0 两份崩溃后镜像传同一份 | — | — | — |
| 2 | `instance_table_of_the_effective_root`（crash.rs:172）：从写表里倒着找写出恢复所落根身份的那次根槽写，拿它的字节 `InstanceTableOfRootRecord::read`（walk.rs:3834）沿链读实例表 | 恢复落到的那一版哪些发布被抛弃 | c | 一条根记录 + 实例表链 |
| 3 | `publishes_in(writes, persisted, continuity)`（singlefs_harness::memory_pool）逐次发布 | — | c | 写表 W 条 |
| 4 | 第一条判据：`in_place(publish.root) && !publish.records.iter().any(in_place)` → `root_without_record`（crash.rs:139-143）；`in_place` 在崩溃态镜像上读写的区间、`write.contents.still_on_disk(&bytes)` 逐字节比（crash.rs:121-126） | 根在而记录一条都不在 | c（对记录流） | 每次发布：根槽 + 各记录的区间 |
| 5 | 第二条判据：恢复自称的 txg ≥ 这次发布且没被抛弃时，按偏移把单元副本分组，某组每份副本都缺席 → `claimed_state_missing_unit`（crash.rs:145-159） | 声称的状态缺单元 | c | 每份副本 |
| 6 | 缺席判定 `unit_copy_is_missing_under_the_persisted_set`（crash.rs:191）：按 512 字节扇区逐个看还是不是这份副本的字节；不是的，在 `copy + 1..writes.len()` 里找持久了、落在这个扇区上、过了回收谓词的更晚写（crash.rs:220-239） | 同上 | c | 每份副本 长度 ÷ 512 个扇区 × 最多 W 次后写 |
| 7 | 回收谓词 `reuse_is_not_proven_illegal_by_the_reclaim_predicate`（crash.rs:293）：扫 `writes[..later_index]` 里的根槽写与系统配置槽写取三个界（系统配置槽写要 `SystemConfiguration::parse_slot`，singlefs-core） | C513 那一格 | c | 每个后写扫一遍写表前缀，按后写下标缓存（crash.rs:207） |

记录核对器 `use` 了 `singlefs_core::system_configuration::SystemConfiguration`（crash.rs:33）、`singlefs_core::recovery`（crash.rs:19）与 `singlefs_harness::memory_pool`（crash.rs:34）：它住在 checker 档，不是池级 checker，不受门禁 94 号 ①② 约束（见第 6 节）。

## 2　(a) 类在 GPU 上算要怎么重映射；今天实现里没有 GPU 对应物的东西

### 2.1 逐项重映射（推的：读代码与 E163 装置得出的形态，没写、没量）

| (a) 项 | 今天的入参 / 出参 | GPU 上要映射成 | 着色器里要算的 |
|---|---|---|---|
| 整单元校验和 vs 位置条目（I-2.1，image.rs:746） | `reader.read(..)` 交回 `Option<Vec<u8>>`；比父指针里的 u32 | 去重后单元内容的扁平缓冲 `array<u32>`，每单元占定长槽（E163 取 32768 字节 = 8192 字，rs:236）+ 每单元长度数组（E163 `unit_lengths`，rs:231）；每状态一张「(盘, 槽) → 内容下标」表 | 整单元 CRC-32C，出一个 u32；比较留在 CPU 或另传期望值数组 |
| 单元头自证 `check_unit`（lib.rs:374） | `&[u8]` → `Result<u8, Verdict>` | 同一扁平缓冲；出参改成每单元一个 u32 判定码（`Verdict` 27 个成员都不带载荷，lib.rs:96-165，`awk 'NR>=96&&NR<=165 && /^    [A-Z][A-Za-z]+,$/' … | wc -l` 数得 27，可编成整数） | 头宽按类标签与 `unit[51]` 现算（码 2 头末 = 86 + 2k，lib.rs:354-370）；头 CRC 要把偏移 10 起的 32 字节当 0 参与（`checksum_field_holds`，lib.rs:82-92 今天是 `to_vec()` 再 `fill(0)`）；载荷 CRC；格式版本；29 字节预留位全 0 |
| `judge_unit_header` 的 I-1.6 / I-2.3 / I-2.4 / I-1.4（walk.rs:663） | `&mut self` 上累加 `Judgements`，失败时 push `String` 进 `walk_failures` | 每单元一组位旗标（每条不变量一位）+ 期望类标签（来自父指针的种类，要 CPU 先给）+ 池 fsid 低 8 字节（uniform） | 同上，再加声明长度之后补齐全 0 |
| `index_node_view` + `check_index_node_keys` / `check_internal_node_separators`（lib.rs:447、567、590） | 产出 `IndexNodeView { entries: Vec<Vec<u8>>, smallest_key: Vec<u8>, .. }`（lib.rs:432-444）；`KeySchema::fields` 每条 key 现算一个 `Vec<u64>` 再按 `Vec` 字典序比（lib.rs:533-550、573） | 不切条目：按 (条目区起点 = 115 + 2k, 条目宽, 条目数) 在缓冲里现算偏移；key 形态表（5 种，lib.rs:511-524）编成常量数组 + 树种类参数 | 逐条相邻 key 按字段比；字段宽 1/2/4/6/8 字节，6、8 字节字段在 WGSL 里要拆成两个 u32 比（见 2.2） |
| `packed_unit_view`（lib.rs:622） | `records: Vec<Vec<u8>>` | 同上按 (136, 记录宽, 记录数) 现算 | 声明长度 = 记录数 × 记录宽、记录宽 0 那一格 |
| `check_system_configuration_slot` + `geometry_of`（lib.rs:245、image.rs:241） | 4096 字节槽 → `Result<SystemConfigurationView, Verdict>` | 每盘 2 槽，量极小（E161 `g5_contents` 报 fixed_structure 总共 35328–106496 字节，见第 4 节） | 不值得单独上 GPU（推的） |
| `check_root_slot`（lib.rs:318） | R × S 槽 | 同上 | 同上 |
| `check_journal_record`（lib.rs:733）、`back_chain_of_record_header`（lib.rs:726） | 4096 字节 → `Result<JournalRecordView, Verdict>`（含 `named: Vec<NamedEntryView>`、`new_root_segment: Vec<u8>`） | 每条 4096 字节定长槽；出判定码 + 反向链要用的 311 字节头 CRC | 整条 CRC（偏移 46 那 32 字节当 0）、载荷 CRC（311 到 311 + 56 × 点名项数）、点名项 flags；E161 报 journal 去重后只有 2–7 种内容（第 4 节），量极小 |
| 扫描方向的单元头（walk.rs:2440、4572、2396） | 每候选槽读 4096 或 16384 字节，只用头 | 与整单元同一个缓冲；只算头 CRC | 头 CRC + fsid + 几个字段抽出来交回 CPU（已发布谓词要实例表，是 b） |

### 2.2 今天的实现里在 GPU 上没有对应物的东西

| 东西 | 在哪 | GPU 上的处境 |
|---|---|---|
| `&dyn ImageReader` 按需读、交回 `Option<Vec<u8>>` | image.rs:19-28；每次读经 `CrashImage` 叠写表（memory_pool.rs:445-469） | 没有 trait 对象、没有按需读盘：要在 CPU 侧先把要判的单元字节聚成扁平缓冲再派发 |
| `Vec<u8>`、`Vec<Vec<u8>>`、`to_vec()` 拷贝 | lib.rs:87、350、443、453、487、619、647；walk.rs 里普遍 | 着色器里没有堆分配；定长槽 + 偏移现算 |
| `BTreeMap` / `BTreeSet`（引用表、`visited_units`、`IndexNodeCache`、记账行、`Judgements` 本身） | walk.rs:258-302、2762；image.rs:122-128 | 没有有序映射；这些都属 b / c 类，留在 CPU（推的） |
| `Judgements::judge(.., detail: impl FnOnce() -> String)` 生成违例说明文字 | image.rs:132 | 没有 `String`、没有闭包：GPU 只能交判定码 / 位旗标 + 第一处违例的单元下标，文字由 CPU 事后按下标重算 |
| `Result<_, Verdict>` 与 `?` 早退 | lib.rs:245-289、374-427、447-503 | 没有 `Result`；改成判定码，分支按码走（WGSL 有 `if`/`switch`/`loop`） |
| `assert_eq!` / `expect` / `panic!` | lib.rs:468（`assert_eq!(offset, 86 + 2 * key_width, ..)`）、lib.rs:544（未登记字段宽 `panic!`）、各处 `try_from(..).expect` | 着色器里不能 panic；越界读的行为由 WGSL 规范定（不在这次核的范围内，未核） |
| `u64` 标量（fsid 低 8 字节、txg、树 ID、key 字段）与 6 字节整数 `read_six_byte_unsigned`（lib.rs:231） | lib.rs:228-241 | WGSL 核心没有 64 位整数；wgpu 有 native-only 特性 `SHADER_INT64`（`wgpu-types-30.0.1/src/features.rs:1125` `const SHADER_INT64 = 1 << 37;`，文档注释「Allows shaders to use i64 and u64」「Supported platforms: - Vulkan …」「This is a native only feature.」）；E163 请求的是 `wgpu::Features::empty()`（e163 rs:294），要 u64 就得开这个特性或拆成两个 u32 |
| CRC 查表 `OnceLock<[u32; 256]>` | lib.rs:58 | 可放 storage / uniform 缓冲或着色器常量数组；E163 的着色器用的是按位（rs:238-273），没有查表 |
| `#![forbid(unsafe_code)]` | 五个 crate 的 lib.rs 都有（checker lib.rs:7、core lib.rs:7、harness lib.rs:5、checker-tier lib.rs:4、format lib.rs:11） | wgpu 的 API 是安全的；E163 把 `&[u32]` 转字节用的是逐个 `to_le_bytes`（rs:326-332），没用 `unsafe` 或 bytemuck |
| wgpu 默认上限 | `wgpu-types-30.0.1/src/limits.rs:441` `max_storage_buffer_binding_size: 128 << 20, // (128 MiB)`、:443 `max_buffer_size: 256 << 20, // (256 MiB)`（`Limits::default()` 调 `defaults()`，limits.rs:344-347） | E163 用 `wgpu::Limits::default()`（rs:295）：一次绑定的单元缓冲超过 128 MiB（32768 字节槽约 4096 个单元，推的算术）就要提高 `required_limits` 或分批派发 |

## 3　E163 装置里能直接复用的件

装置 `research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs`（981 行，`wc -l`），下文「rs:」指这份文件的行号。

| 件 | 位置 | 做什么 | 搬进 `crates/` 时要改的（推的，没改没编） |
|---|---|---|---|
| WGSL 着色器 `COMPUTE_SHADER_SOURCE` | rs:216-275 | 4 个绑定：`params`（uniform，unit_count + 3 个预留）、`unit_words: array<u32>`、`unit_lengths: array<u32>`、`results: array<ResultEntry{crc, local_offset}>`（rs:229-232）；`@workgroup_size(64)`（rs:238）；一个调用算一个单元，按位反射 CRC-32C，字节按 `(word >> 8*(k%4)) & 0xFF` 取 | 槽宽写死 `WORDS_PER_UNIT_SLOT = 8192u`（rs:236，32768 字节），16384 字节的码 2 节点要么占半个空槽、要么另开一个槽宽参数；从槽首算到 `length_in_bytes`，不支持「从偏移 X 起算」（载荷 CRC 要从头末起）和「某 32 字节按 0 参与」（头校验和） |
| CPU 参照 `crc32c_reference` | rs:30-44 | 按位反射 CRC-32C，与着色器各持一份常量（rs:24、26 与 rs:234-235） | 池级 checker 已有两份（lib.rs:38 按位、lib.rs:57 查表），不用搬 |
| 锚点 `EXPECTED_ANCHOR_VALUES` / `anchor_vectors` / `anchor_mismatches` | rs:47-88 | 8 个锚点的绝对值，GPU 与 CPU 各自比绝对值（页面「路径与结论登记」表下那段：两份同改错时只有绝对值检查抓得到） | 可原样当阳性对照 |
| 适配器枚举 `new_vulkan_instance`、`nvidia_discrete_adapters` | rs:183-195 | 只开 Vulkan 后端；过滤 vendor 0x10DE 且 `DeviceType::DiscreteGpu` | 可原样 |
| 上下文 `build_gpu_context` | rs:290-324 | `request_device`（`Features::empty()`、`Limits::default()`、`MemoryHints::Performance`），再分配一块 4 字节 storage 缓冲当「起得来」的探针，建管线 | 要 u64 就加 `SHADER_INT64`；批大于 128 MiB 要调 `required_limits`（第 2.2 节） |
| 派发 `run_gpu_crc` | rs:341-421 | 建 params / input / lengths / results（整片先填哨兵 0xFF）/ staging 五块缓冲，一次 `dispatch_workgroups`，拷到 staging，`map_async` + `device.poll(PollType::Wait { timeout: None })` 回读 | `poll` 不设超时（rs:408）；每次派发都新建全部缓冲 |
| 覆盖断言 `assert_full_coverage_no_sentinel` | rs:425-430 | 每条结果的 `local_offset` = 数组下标、没有哨兵残留 | 可原样 |
| 多卡切片 `command_run_shard` | rs:656 起 | 一张卡一个进程：`--input 批文件 --start --count --adapter-index --output`，另可注入翻位 `--inject-flip` 与丢结果 `--inject-drop` | 跨机不在装置里：文件里 `ssh`/`scp` 零命中（`grep -c 'ssh\|scp'` 得 0），跨机拷贝与起进程由跑的人在外面做 |
| 结果文件 `write_shard_result_file` / `read_shard_result_file` | rs:450、504 | 每片一个文本结果文件（带全局单元号、CRC、计时字段） | 计时字段让每片文件两次跑不同，页面「复跑」一节只登记 merge 输出 |
| 合并与比对 `command_merge` → `compare_reference_and_shards` | rs:753、rs:538-561 | 按全局单元号 0..N 查：缺的、重复的、与 CPU 参照不同的各列一张表 | 可原样当「GPU 结果 vs CPU 参照逐格比」的骨架 |
| 测试互斥锁 `GPU_TEST_MUTEX` | rs:853（用在 rs:942、959、971） | 进程内 `static Mutex<()>` 序列化三条 GPU 单测 | 只管同一进程里的线程；几个测试进程同时抢同一张卡不在它射程里（推的，没量过） |

依赖与版本（`research/e7-index-bench/Cargo.toml`）：

- :466-468 `[features]` 里 `e163-gpu = ["dep:wgpu", "dep:pollster"]`；:478-482 `[dependencies.wgpu]` `version = "=30.0.1"`、`default-features = false`、`features = ["vulkan", "wgsl", "std"]`、`optional = true`；:484-486 `[dependencies.pollster]` `version = "=1.0.1"`、`optional = true`；:488-491 bin 挂 `required-features = ["e163-gpu"]`。
- `research/` 是独立 workspace（根 `Cargo.toml:12` `exclude = ["research"]`），锁在 `research/Cargo.lock`：`grep -c '^name = "wgpu' research/Cargo.lock` 得 6、根 `Cargo.lock` 得 0；`grep -rln wgpu crates/ | wc -l` 得 0。
- 装置代码不照 `crates/` 的 clippy 口径写：`grep -cE '\bas (u32|u64|usize|i64)\b'` 得 24 处 `as` 转换（如 rs:347 `count as u32`、rs:397），而 `crates/` 走的 `check.sh` 带 `-D clippy::cast_possible_truncation`（`.claude/singlefs-ai-sop/scripts/check.sh:75`）。搬进 `crates/` 要改成 `try_from`。

实验页 `.claude/kb/experiments/163-GPU多卡算单元校验和.md` 写的限制：

- M2（双机五卡）判「不能」，原因是显存（以下两处是那两行里的一句，整行太长没抄，整行看原文）：:31「**另一台第三张卡（RTX 5080，片 2，单元 1024–1535）两次都建不起 GPU 上下文**：`request_device` 失败，`RequestDeviceError { inner: Core(Device(OutOfMemory)) }`；当时该卡 `nvidia-smi` 显存余量仅 141 MiB（被本地服务占满，登记「不停本地模型」）。」；:38 结论「翻面条件是那张卡腾出显存」。
- 变异（:62 那一行的头一句）：「**`research/scripts/mutate.sh` 跑不了这个表**：它固定用 `cargo test --release --bin "$BIN"`，不带 `--features`……」。⚠️ 这句与 HEAD 里的 `mutate.sh` 已经对不上：HEAD 版（`git show HEAD:research/scripts/mutate.sh | grep -c required-features` 得 23）文件头 :16-19 写「bin 在 Cargo.toml 的 [[bin]] 里挂了 required-features 的：每次 cargo test 带 `--features <包名>/<feature>,…`」，引入它的提交是 `7a3c8e1e`（2026-09-27）。这张表在新 `mutate.sh` 下跑没跑过、几个 worker 进程同时占卡会不会重现页面 :84 那种挂起，没查（推的风险）。
- 它答不了的（:107-111）：显存充裕时五卡是否全成、速度与吞吐（登记不设速度门槛）、单元长度 / 派发粒度 / 批大小敏感性（只跑过 N = 2560、32768 字节）。
- 影响的决策（:119）：D24（后台重活能不能卸给 GPU） 记「不影响」，理由是 D24 射程只管三个后台重活。

## 4　E161 可行性档的分段计时与去重计数（整行抄）

产物 `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out`（171 行）。取样域与线程数（:1）：

```
E7RESULT name=config experiment=E161 mode=feasibility threads=10 registration=research/prompts/e161-preregistration.md domain=first_small_37+quick_54_84+strided_segment_heads_4096
```

去重计数（`name=g1`，`grep -n 'name=g1 '`，5 行）：

```
28:E7RESULT name=g1 cell=first_small states=37 distinct_unit_contents=18 per_state_unit_checks=644 r1=0.027950 r1_at_or_above_half=false unit_read_events=1610 states_without_unit_reads=0 header_scan_events=1380 header_scan_per_state_positions=534 header_scan_distinct_contents=15 ring_record_events=212 ring_record_per_state_positions=106 ring_record_distinct_contents=3 ring_record_r=0.028301 content_collisions=0 q1a_peak_after_cold_start=12 q1a_positive_intervals=3/10 q1a_final=18 q1a_last_four=[12,0,2,0]
39:E7RESULT name=g1 cell=first_segment_head states=4096 distinct_unit_contents=9 per_state_unit_checks=50324 r1=0.000178 r1_at_or_above_half=false unit_read_events=99476 states_without_unit_reads=0 header_scan_events=84264 header_scan_per_state_positions=42132 header_scan_distinct_contents=8 ring_record_events=32768 ring_record_per_state_positions=16384 ring_record_distinct_contents=2 ring_record_r=0.000122 content_collisions=0 q1a_peak_after_cold_start=0 q1a_positive_intervals=1/1 q1a_final=9 q1a_last_four=[9]
50:E7RESULT name=g1 cell=first_quick states=54 distinct_unit_contents=18 per_state_unit_checks=972 r1=0.018518 r1_at_or_above_half=false unit_read_events=2142 states_without_unit_reads=0 header_scan_events=1968 header_scan_per_state_positions=828 header_scan_distinct_contents=15 ring_record_events=348 ring_record_per_state_positions=174 ring_record_distinct_contents=3 ring_record_r=0.017241 content_collisions=0 q1a_peak_after_cold_start=12 q1a_positive_intervals=3/11 q1a_final=18 q1a_last_four=[0,0,2,0]
138:E7RESULT name=g1 cell=second_segment_head states=4096 distinct_unit_contents=23 per_state_unit_checks=165012 r1=0.000139 r1_at_or_above_half=false unit_read_events=443540 states_without_unit_reads=0 header_scan_events=387368 header_scan_per_state_positions=140436 header_scan_distinct_contents=20 ring_record_events=49152 ring_record_per_state_positions=24576 ring_record_distinct_contents=3 ring_record_r=0.000122 content_collisions=0 q1a_peak_after_cold_start=0 q1a_positive_intervals=1/1 q1a_final=23 q1a_last_four=[23]
149:E7RESULT name=g1 cell=second_quick states=84 distinct_unit_contents=58 per_state_unit_checks=6708 r1=0.008646 r1_at_or_above_half=false unit_read_events=19068 states_without_unit_reads=0 header_scan_events=16692 header_scan_per_state_positions=5868 header_scan_distinct_contents=52 ring_record_events=1544 ring_record_per_state_positions=772 ring_record_distinct_contents=7 ring_record_r=0.009067 content_collisions=0 q1a_peak_after_cold_start=9 q1a_positive_intervals=5/14 q1a_final=58 q1a_last_four=[0,8,0,0]
```

分段计时（`name=g3`，`grep -n 'name=g3 '`，5 行）：

```
37:E7RESULT name=g3 cell=first_small states=37 state_elapsed_ns=77898270 recovery_consult_elapsed_ns=6218674 recovery_consult_share_ratio=0.079830 recovery_ignore_elapsed_ns=4220050 recovery_ignore_share_ratio=0.054173 oracle_elapsed_ns=57120 oracle_share_ratio=0.000733 pool_checker_elapsed_ns=64510703 pool_checker_share_ratio=0.828140 record_checker_elapsed_ns=2880753 record_checker_share_ratio=0.036980 unit_check_replay_elapsed_ns=35012682 unit_check_replay_share_ratio=0.449466 unit_events_replay_elapsed_ns=111501157 unit_events_share_ratio=1.431368 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=29498021 walk_checker_elapsed_ns=69131624 walk_checker_of_checker_share_ratio=1.071630 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.108108
48:E7RESULT name=g3 cell=first_segment_head states=4096 state_elapsed_ns=5361069162 recovery_consult_elapsed_ns=493604274 recovery_consult_share_ratio=0.092071 recovery_ignore_elapsed_ns=441067394 recovery_ignore_share_ratio=0.082272 oracle_elapsed_ns=2454094 oracle_share_ratio=0.000457 pool_checker_elapsed_ns=3967962616 pool_checker_share_ratio=0.740143 record_checker_elapsed_ns=455165874 record_checker_share_ratio=0.084902 unit_check_replay_elapsed_ns=2512991021 unit_check_replay_share_ratio=0.468748 unit_events_replay_elapsed_ns=7079574037 unit_events_share_ratio=1.320552 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=1454971595 walk_checker_elapsed_ns=4302373569 walk_checker_of_checker_share_ratio=1.084277 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.000000
59:E7RESULT name=g3 cell=first_quick states=54 state_elapsed_ns=100154281 recovery_consult_elapsed_ns=7983871 recovery_consult_share_ratio=0.079715 recovery_ignore_elapsed_ns=6029949 recovery_ignore_share_ratio=0.060206 oracle_elapsed_ns=50789 oracle_share_ratio=0.000507 pool_checker_elapsed_ns=81293816 pool_checker_share_ratio=0.811685 record_checker_elapsed_ns=4782447 record_checker_share_ratio=0.047750 unit_check_replay_elapsed_ns=51945813 unit_check_replay_share_ratio=0.518657 unit_events_replay_elapsed_ns=149278874 unit_events_share_ratio=1.490489 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=29348003 walk_checker_elapsed_ns=86900991 walk_checker_of_checker_share_ratio=1.068974 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.222222
147:E7RESULT name=g3 cell=second_segment_head states=4096 state_elapsed_ns=19150712953 recovery_consult_elapsed_ns=1114083742 recovery_consult_share_ratio=0.058174 recovery_ignore_elapsed_ns=988126998 recovery_ignore_share_ratio=0.051597 oracle_elapsed_ns=39248714 oracle_share_ratio=0.002049 pool_checker_elapsed_ns=16482335779 pool_checker_share_ratio=0.860664 record_checker_elapsed_ns=525887270 record_checker_share_ratio=0.027460 unit_check_replay_elapsed_ns=8635287703 unit_check_replay_share_ratio=0.450912 unit_events_replay_elapsed_ns=28538927000 unit_events_share_ratio=1.490227 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=7847048076 walk_checker_elapsed_ns=17651600700 walk_checker_of_checker_share_ratio=1.070940 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.000000
158:E7RESULT name=g3 cell=second_quick states=84 state_elapsed_ns=753022700 recovery_consult_elapsed_ns=28987879 recovery_consult_share_ratio=0.038495 recovery_ignore_elapsed_ns=23935482 recovery_ignore_share_ratio=0.031785 oracle_elapsed_ns=821199 oracle_share_ratio=0.001090 pool_checker_elapsed_ns=686332409 pool_checker_share_ratio=0.911436 record_checker_elapsed_ns=12921261 record_checker_share_ratio=0.017159 unit_check_replay_elapsed_ns=363700795 unit_check_replay_share_ratio=0.482987 unit_events_replay_elapsed_ns=1220581625 unit_events_share_ratio=1.620909 walk_and_judge_lower_elapsed_ns=0 walk_and_judge_upper_elapsed_ns=322631614 walk_checker_elapsed_ns=728438333 walk_checker_of_checker_share_ratio=1.061349 state_shorter_than_its_segments_share_ratio=0.000000 replaying_longer_than_the_checker_share_ratio=0.000000
```

G5 的内容量（`name=g5_contents`，5 行）与显存阳性对照（`name=positive_control_device_memory`，2 行）：

```
36:E7RESULT name=g5_contents cell=first_small states=37 fixed_structure_bytes=72192 fixed_structure_distinct=22 journal_ring_bytes=12288 journal_ring_distinct=3 whole_unit_bytes=344064 whole_unit_distinct=18 unit_header_scan_bytes=61440 unit_header_scan_distinct=15 p1_bytes=489984 p1_peak_after_cold_start=324096 p1_positive_intervals=6/10 p1_final=489984 p1_last_four=[324096,0,16896,0]
47:E7RESULT name=g5_contents cell=first_segment_head states=4096 fixed_structure_bytes=38912 fixed_structure_distinct=13 journal_ring_bytes=8192 journal_ring_distinct=2 whole_unit_bytes=163840 whole_unit_distinct=9 unit_header_scan_bytes=32768 unit_header_scan_distinct=8 p1_bytes=243712 p1_peak_after_cold_start=0 p1_positive_intervals=1/1 p1_final=243712 p1_last_four=[243712]
49:E7RESULT name=positive_control_device_memory cell=first_segment_head p1_grew=32768 p3_grew=147 p2_grew=7 p4_per_state_grew=48 pc5_holds=true
58:E7RESULT name=g5_contents cell=first_quick states=54 fixed_structure_bytes=80384 fixed_structure_distinct=24 journal_ring_bytes=12288 journal_ring_distinct=3 whole_unit_bytes=344064 whole_unit_distinct=18 unit_header_scan_bytes=61440 unit_header_scan_distinct=15 p1_bytes=498176 p1_peak_after_cold_start=262656 p1_positive_intervals=7/11 p1_final=498176 p1_last_four=[69632,0,16896,0]
146:E7RESULT name=g5_contents cell=second_segment_head states=4096 fixed_structure_bytes=35328 fixed_structure_distinct=13 journal_ring_bytes=12288 journal_ring_distinct=3 whole_unit_bytes=425984 whole_unit_distinct=23 unit_header_scan_bytes=81920 unit_header_scan_distinct=20 p1_bytes=555520 p1_peak_after_cold_start=0 p1_positive_intervals=1/1 p1_final=555520 p1_last_four=[555520]
148:E7RESULT name=positive_control_device_memory cell=second_segment_head p1_grew=32768 p3_grew=147 p2_grew=66 p4_per_state_grew=48 pc5_holds=true
157:E7RESULT name=g5_contents cell=second_quick states=84 fixed_structure_bytes=106496 fixed_structure_distinct=33 journal_ring_bytes=28672 journal_ring_distinct=7 whole_unit_bytes=1048576 whole_unit_distinct=58 unit_header_scan_bytes=212992 unit_header_scan_distinct=52 p1_bytes=1396736 p1_peak_after_cold_start=213504 p1_positive_intervals=9/14 p1_final=1396736 p1_last_four=[0,180736,4096,0]
```

去重键（`name=reuse_arm`，K2-consult / K2-ignore / K2-pair / K4-walk / K4-units / K4-full 六臂 × 五格 = 30 行）在产物 :29-34、:40-45、:51-56、:139-144、:150-155，每行三百字以上，这里没抄，要引时整行取。判定行 :170 末尾：`g1=not_judged_feasibility_only g2=not_judged_feasibility_only g3=not_judged_feasibility_only g4=not_judged_feasibility_only g5=not_judged_feasibility_only`（这一段是 `grep -o` 取的片段，不是整行）——可行性档 G1–G5 一行都没判。

按上面各行算的每状态平均（算术，`python3` 从这几行解析字段相除；只是可行性档取样域上的平均，`threads=10`、同机有别的负载，量级参考）：

| 格 | 每状态逐单元检查 | 每状态单元读 | 每状态头扫描读 | 每状态环记录读 | 每状态总耗时 µs | 其中池级 checker µs | 记录核对器 µs | 恢复（看 journal）µs |
|---|---|---|---|---|---|---|---|---|
| first_small | 17.4 | 43.5 | 37.3 | 5.7 | 2105.4 | 1743.5 | 77.9 | 168.1 |
| first_segment_head | 12.3 | 24.3 | 20.6 | 8.0 | 1308.9 | 968.7 | 111.1 | 120.5 |
| first_quick | 18.0 | 39.7 | 36.4 | 6.4 | 1854.7 | 1505.4 | 88.6 | 147.8 |
| second_segment_head | 40.3 | 108.3 | 94.6 | 12.0 | 4675.5 | 4024.0 | 128.4 | 272.0 |
| second_quick | 79.9 | 227.0 | 198.7 | 18.4 | 8964.6 | 8170.6 | 153.8 | 345.1 |

读这两张表要知道的口径（出自跑前登记 `research/prompts/e161-preregistration.md` 第六节 G3，:295-308，Q3f 在 :304、Q3g 在 :305）：`unit_check_replay`（Q3f，s_lo）是「每个状态对它每个不同单元位置重放一次 U」，`unit_events_replay`（Q3g，s_hi）是「每一次单元读重放一次 U，另对每一次头扫描读做一次 `crc32_castagnoli_bitwise`（整段读到的字节），对每一次环内 4096 字节读做一次 `check_journal_record`」；U(字节) = `crc32_castagnoli_table` + 按类标签 `check_unit` / `index_node_view` + key 检查 / `packed_unit_view`（:299）。U 里的 `check_unit` 用的是按位 CRC（lib.rs:409），走读里 `judge_unit_header` 的载荷 CRC 用查表（walk.rs:725），所以 s_hi 大于 1（1.32–1.62）不说明单元级检查比整个 checker 还长，只说明重放的这份按位 CRC 比真走读里的那几道贵（推的，没拆开量）。

跑前登记第四节 4.8、4.9 原文（`research/prompts/e161-preregistration.md:174-175`，整行）：

```
| 4.8 | 显存预算：10¹⁰ 个状态时单卡每状态 3.419 字节、五卡每状态 10.26 字节 | 同 4.7 | **G5 的判定在跑之前已被算术大半定了**：凡按状态存下来的部分（子集掩码、每状态的键），每状态超过 10.26 字节，一次装入在五卡上也装不下 |
| 4.9 | 子集掩码按写表位图算（推的：W = 录制流写数 + 每次取三态的写一份撕裂镜像，同段没有与它重叠的更晚写、没有重放）：第一条流 W = 49、7 字节/状态，10¹⁰ 时 7×10¹⁰ 字节（66 757 MiB），单卡装不下、五卡装得下；第二条流整条 W = 523、66 字节/状态，10¹⁰ 时 6.6×10¹¹ 字节，五卡装不下；第二条流最大一段 2 415 919 103 个状态 × 66 字节 = 159.45 GB，按段分批也装不下 | 同 4.7 | 同 4.8：显式存每状态掩码的「一次装入」按第二条流的 W 已装不下；跑出来的数补的是 P1（去重后的内容）、P3（读集）的实际大小与每状态平均。问题单的两个候选都假定掩码要存；「掩码不存、由序号现算」不在候选里，交回时报给主 agent |
```

4.8、4.9 的「同 4.7」指 :173 那一行的出处（第十三节 13.2 的 `anchors.py`）；4.6（:172）是容量读数：本机 5090 total 32607 MiB、5060 Ti total 16311 MiB，五卡合计按 97843 MiB 算，另一台的读数那一行写「是主 agent 转述的读数，我没现查」。

## 5　E162 现状

第 5 问改派，不答。

## 6　依赖约束：门禁 94 号判什么、GPU 代码住哪、`Cargo.lock` 牵动哪些行

### 6.1 门禁 94 号（`.claude/gate.d/94-checker-implementation-disjoint.sh`）

| 条 | 行 | 判什么 | 对 GPU 代码意味着什么（推的，没造输入跑过） |
|---|---|---|---|
| ① | :9-14；实现 :146-171 | 按 `crates/*/Cargo.toml` 建内部依赖图，`singlefs-checker` 与 `singlefs-core` 各取传递闭包求交，减去 `singlefs-format` 必须为空；:148「外部 crate 在图上是叶子（本仓没有它的 Cargo.toml），仍然进闭包、仍然参与求交」 | wgpu 只进池级 checker 一侧：交集仍只有 `singlefs-format`，不红；core 一侧也引 wgpu（或引同一个 GPU 内部 crate）就红。wgpu 自己的传递依赖不在图上（:13-14「罩不到的」） |
| ② | :15-16 | 池级 checker 源码零处引 `singlefs_core` | GPU 代码写在池级 checker 里就归这一条管 |
| ③ | :17-19 | `singlefs-format` 正文不许有分支与循环 | GPU 代码不能放进 format |
| ④ | :20-22；实现 :259-271 | harness 档包的闭包里没有 `singlefs-checker-tier`，源码零处引 `singlefs_checker_tier` | GPU 代码放 checker 档不碰这一条；放池级 checker 时 harness 依赖池级 checker（`crates/singlefs-harness/Cargo.toml:9`），非可选依赖会让 harness 档每次编都带上 wgpu（编译代价，不是违规） |
| 射程 | :40 | 「五个 crate 的名字写死在这里」（python 里 :56-61 的 `CHECKER`、`CORE`、`SHARED`、`HARNESS`、`CHECKER_TIER`） | 新建第六个 crate 放 GPU 代码，它自己引不引 `singlefs_core` 不归 ② 管；只有被池级 checker 依赖时进 ① 的闭包 |

`.claude/rules/verification.md` 里相关的三句：:14 池级 checker 那一行「只依赖 `singlefs-format`」；:17「依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档」；:45「池级 checker 与实现的依赖闭包（dev-dependencies 也算）除 `singlefs-format` 之外不相交……它今天只依赖 `singlefs-format`、没有 `[dev-dependencies]`，这一句 94 号不单判」。即「池级 checker 只依赖 format」这一句今天没有门禁单判，94 号判的是交集。

现查的依赖（`[dependencies]` 段）：池级 checker 只有 `singlefs-format`（`crates/singlefs-checker/Cargo.toml:9`）；core 只有 `singlefs-format`（`crates/singlefs-core/Cargo.toml:9`）；harness 有 checker、core、format（`crates/singlefs-harness/Cargo.toml:9-11`）；checker 档有 checker、core、format、harness（`crates/singlefs-checker-tier/Cargo.toml:9-12`）。五个 crate 都 `#![forbid(unsafe_code)]`。

### 6.2 GPU 代码住哪（三个候选，推的，都没试）

| 候选 | 94 号 | 独立性（D13（验证路线） 已定项 5：checker 与实现只共享常量模块） | 别的代价 |
|---|---|---|---|
| 放池级 checker `crates/singlefs-checker`，wgpu 挂可选特性 | ①②④ 都不红（前提：core 不引 wgpu） | 有机器守：② 查源码不引 core | 用户原话要的是「和 core 无关」「checker 的部分」（`.claude/kb/milestone/03-third-txn.md:51`）；harness 依赖池级 checker，特性不开就不编 wgpu；工作区一起编时特性合并由 Cargo 定（推的，没核 resolver 2 在这个仓里的行为） |
| 放 checker 档 `crates/singlefs-checker-tier` | 不红（它不是 ① 比的两侧之一） | 没机器守：checker 档本来就依赖 core（Cargo.toml:10），GPU 判定代码若调了 core 的东西，94 号看不见 | checker 档默认只在提交时跑（`.claude/rules/verification.md:13`） |
| 新建第六个 crate，只依赖 `singlefs-format` + wgpu，被池级 checker 或 checker 档依赖 | 被池级 checker 依赖时进 ① 闭包；它自己不在 ② 的扫描范围 | 要给 94 号加第六个路径才有 ② 那种守 | 根 `Cargo.toml:5-11` 的 `members` 要加一行；94 号写死五个路径（:40） |

### 6.3 给 `crates/` 加 wgpu 会动根 `Cargo.lock`；`stage-inputs.tsv` 里登记了 `Cargo.lock` 的行

根 `Cargo.lock` 今天没有 wgpu（`grep -c '^name = "wgpu' Cargo.lock` 得 0）。`.claude/gate.d/stage-inputs.tsv` 里第二列有 `Cargo.lock` 的行（`awk -F'\t'` 按空格切第二列逐词比，14 行；非注释且有第二列的行共 16 行，另两行是 :30 `57-lkmm.sh` 与 :35 `E142/layer0`）：

| 行 | 键 | 根 `Cargo.lock` | `research/Cargo.lock` |
|---|---|---|---|
| 28 | 54-layer0-replay.sh | 有 | — |
| 29 | 55-qemu-device-streams.sh | 有 | — |
| 31 | 59-crates-mutation-replay.sh | 有 | — |
| 32 | 74-model-differential.sh | 有 | — |
| 33 | 87-replay.sh | 有 | — |
| 34 | E142 | 有 | 有 |
| 36 | crash-case:layer0-first-stream | 有 | — |
| 37 | crash-case:layer0-second-stream | 有 | — |
| 38 | crash-case:floor-raise-pushed-by-the-session | 有 | — |
| 39 | crash-case:c561-sigma-full | 有 | — |
| 40 | crash-case:layer0-multi-record-publish-stream | 有 | — |
| 41 | crash-case:layer0-tree-split-streams | 有 | — |
| 42 | crash-case:layer0-position-addressed-tree-streams | 有 | — |
| 43 | crash-case:crash-injection-fast-tier | 有 | — |

另 :35 `E142/layer0` 的路径列是 `@E142`，按表头 :12「路径写成 @<别的键> 就是那个键登记的全部路径」也含两份锁。`research/scripts/admission.py` 里 `grep -n Cargo.lock` 零命中：算崩溃枚举用例输入时减掉的只有 `crates/mutations.tsv` 与没人读的 `src/bin/`（admission.py:259、261），不减锁。推的后果：根 `Cargo.lock` 一变，上表 8 条崩溃枚举用例的全绿标记全部作废，下一次 54 号 `--full` 要全部重跑；54、55、59、74、87 号的复用判定也都失效。

## 7　D24 与 E21：定案、射程、管不管崩溃验证

D24（后台重活能不能卸给 GPU）（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md`）射程（:3，整行）：

> D24（后台重活能不能卸给 GPU） 管一件事：三个后台重活（批量压缩、全盘 scrub、大规模纠删码重建）要不要卸给 GPU。不管的：CPU 侧算法怎么选在 D9（加密） 已定项 3；scrub 本身的形态在 D9（加密） 已定项 5；后台整理与回收的队列与预算在 D26（后台整理与放置回收）；三格划分本身是 `.claude/rules/fs-design.md` 的记账纪律，D24（后台重活能不能卸给 GPU） 只引用它。

已定项索引（:9-13）：1 暂缓不排期、GPU 这条线不再继续测；2 三条硬约束；3 判据形式；4 重开的四个闸 G24.1–G24.4；5 合法性来自 fs-design.md 第三格。各分项定案原文在 :17（已定项 1）、:30-34（已定项 2 三条）、:47（已定项 3）、:60-71（已定项 4）、:84（已定项 5）。已定项 2 第 3 条（:34，整行）：

> 3. **它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核。** GPU 算错（驱动 bug、显存 ECC 缺失、算法实现分歧）不能变成静默的数据损坏 ⇒ 与 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构：**GPU 路径与 CPU 路径必须是两份实现**，抽样复核。

E21（GPU 卸载的净收益）（`.claude/kb/experiments/21-GPU卸载的净收益.md`）标题（:1，整行）：

> ## E21 GPU 卸载的净收益 —— 三段全部已测（2026-08-28）；纯扫描那格判负收益，候选三格**暂缓不排期**

它量的是 CPU 内存带宽 64.91 GB/s、GPU 单程 56.69 GB/s、来回 28.5 GB/s、纯扫描端到端 50.3 GB/s 净收益 0.78×（:56-66）；影响的决策表（:214-220）三行都只指 D24。

管不管崩溃验证这一侧（现查到的四处文字，各说各的）：

| 出处 | 原文（整行或指路） | 说了什么 |
|---|---|---|
| D24 :3 | 见上 | 射程逐字限定三个后台重活，崩溃验证不在里面 |
| `.claude/rules/verification.md:34` | checker 档全量那一行里有一句「GPU 只接校验和那一截，要不要接归 D24（后台重活能不能卸给 GPU）」 | **把崩溃验证的 GPU 那一截归到 D24**，与 D24 自己的射程对不上 |
| E163 页 :119、E161 页（`161-崩溃放量的去重与分段耗时.md`）:94 | E163 页「D24（后台重活能不能卸给 GPU） 管的是批量压缩、scrub、纠删码重建三个后台重活（已定项 3 的射程逐字限定）……与那三个候选场景不是同一件事（与 E161（崩溃放量的去重与分段耗时） 对 D24（后台重活能不能卸给 GPU） 的回看同一口径）」（:119 那一行的一段） | 实验页按「不影响」记 |
| `.claude/kb/milestone/03-third-txn.md:51` | 用户原话「这是崩溃放量的实现 是checker的部分 和core无关」「这个肯定是要实现的」（那一行里的两句） | 里程碑三把 GPU checker 定为必做 |

推的：D24 已定项 1「GPU 这条线不再继续测」与已定项 2 的三条硬约束，按 D24 自己的射程不罩崩溃验证；而 verification.md:34 那句把它指回 D24。两处要不要对齐、由谁定，是这一轮三方或用户的事；这里只报对不上。D24 已定项 2 第 3 条「GPU 路径与 CPU 路径必须是两份实现，抽样复核」与里程碑三第四项「GPU 上的 checker 与 CPU 上的 checker 怎么对拍（抽样逐格相同，不同就判红）」（03-third-txn.md:53）内容同向，但前者按射程不管后者。

## 8　几条要带走的事实，与什么会推翻它们

| 事实 | 依据 | 什么现象会推翻 |
|---|---|---|
| 每状态耗时里池级 checker 占大头：可行性档五格 `pool_checker_share_ratio` 0.740–0.911，记录核对器 0.017–0.085，两遍恢复合计 0.07–0.17 | 第 4 节 g3 五行 | 全域按登记 6.0 加权跑完之后占比落到 0.5 以下 |
| 去重之后单元内容极少：五格 `distinct_unit_contents` 9–58，`r1` 0.000139–0.027950 | 第 4 节 g1 五行 | 更大的取样域上 `q1a_final` 随状态数线性涨 |
| (a) 类在今天的走读里是「按指针边走边判」，单元字节经 `&dyn ImageReader` 按需读（image.rs:19-28），每次读经 `CrashImage` 按写表叠一遍（memory_pool.rs:445-469） | 第 1.0、2.2 节 | 找到一条不经 `CrashImage::read` 的读路径 |
| 同一个码 2 节点整单元 CRC 算三遍，其中一遍按位（`check_unit` 里 lib.rs:409） | 第 1.2 节代码路径 | 计时拆分显示 `index_node_view` 不在走读热路径上 |
| 根 `Cargo.lock` 一变，8 条崩溃枚举用例的全绿标记全部失效 | 第 6.3 节 | admission.py 实际会把锁从指纹里减掉（今天 `grep -n Cargo.lock` 零命中） |
| E163 页说 `mutate.sh` 跑不了带 `required-features` 的表，与 HEAD 的 `mutate.sh` 对不上 | 第 3 节 | HEAD 的 `mutate.sh` 实际仍不带 `--features`（它文件头与代码不一致） |
| verification.md:34 把崩溃验证的 GPU 那一截归 D24，D24 :3 射程不含它 | 第 7 节 | D24 另有一处分项把崩溃验证写进射程（全文 97 行已通读，没找到） |

## 没做什么

- 没编译、没跑任何测试或装置（派发写明「重型测试：不跑」，另一会话的层 0 全量占着两台机器）；上文所有「推的」都没量过。
- 第 5 问（E162 现状）按主 agent 改派不答。
- 没读 `walk.rs` 里每个走读子函数的全文（inode 树、extent、分配记录树、实例表链），它们的归类按函数名、判的不变量与调用关系定；「一个单元读几遍」只数到 `read_index_node` 那条路径，别的子走读没逐个数。
- 没核 WGSL 规范里越界访问的语义、没核 Cargo resolver 2 在这个工作区里的特性合并行为。
- 没核 E163 的变异表在 HEAD 的 `mutate.sh` 下跑不跑得起来。
- 没现查两台机器此刻的显存（E161 登记 4.6 的读数是 2026-09-27 的）。
- 草稿目录 `/tmp/claude-1000/m3-facts-checker-gpu/` 只放了比对用的几份文本（`quoted.txt`、`orig.txt`、`quoted_sorted.txt`、`q2.txt`、`o2.txt`），没建编译目录与仓副本。
