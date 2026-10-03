# 里程碑三七·1 实现侧事实表（2026-09-30）

批次：records/2026-09-30-里程碑三开工第一轮.md#1。只读，没改仓内文件，没跑 cargo build / test。

口径（全表通用）：
- 「行数」= 从 `fn` 那一行到收尾 `}`，不含文档注释与属性；「码行」= 其中去掉空行与纯注释行。由草稿脚本 `/tmp/claude-1000/m3-r1/formal-facts/fns.py` 算（跳过 `#[cfg(test)] mod`），全表在 `fns.tsv`（849 个带体的 fn）。
- 「闭包」= 从根函数出发按名字跟调用：自由函数按名、`Type::f` 按 owner、`.f(` 方法按同名全收；停在 `PoolReader` / `BlockDevice` 的接口方法（read、device_identities、device_size_in_bytes、journal_record_offsets_hint、read_at、write_at、write_zeroes_at、barrier、size_in_bytes、probe_physical_block_size）和 from / fmt / clone / default。**闭包合计是推的上界**：同名方法全收会多算（K 闭包里混进了 `ByteWriter::skip/position`、`InodeLeafContainerIndexInTree::position` 共 9 行），trait 对象与闭包参数不跟会少算。每组闭包的逐函数清单在草稿目录 `closure_<组名>.tsv`。
- `singlefs-format` 只有常量：`grep -c "fn " crates/singlefs-format/src/lib.rs` 输出 `7`，7 处都在第 287 行 `#[cfg(test)]` 之后，所以各组合计里 format 贡献 0 行函数。

## (a) `Recover` 在 `crates/` 里是哪几个函数

已定项 1 写的是「一次发布 + 一次崩溃 + 一次恢复」里的 `Recover`，`crates/` 里有两条入口：只读冷启动 `recovery::recover`，和可写挂载 `mount::mount_writable*` 取号之前那一段。派发提示里的「回卷」在代码里是取号失败的回卷写 `PoolWriter::roll_back_acquisition`（`transaction.rs:437` 文档注释「取号全或无失败：已写出的那几份回卷成旧代号」），它在取号之后，是写，列在 a3 里。

### a1 只读冷启动 `recover` 的调用图（直接调用，由脚本列出、逐个对过源码）

```
recover                                   recovery.rs:4222
├─ choose_system_configuration            recovery.rs:964
│  ├─ mountable_slot_or_refusal           recovery.rs:838
│  │  ├─ SystemConfiguration::parse_slot  system_configuration.rs:494
│  │  └─ system_configuration_values_this_reader_accepts  recovery.rs:706
│  └─ slot_one_found_by_trying_every_slot_spacing_the_format_allows  recovery.rs:893
├─ choose_root                            recovery.rs:1149
│  └─ visit_valid_roots → visit_valid_roots_with_ring_slots   recovery.rs:1055 / 1073
│     ├─ every_root_ring_slot             recovery.rs:1097
│     └─ read_root_ring_slot → RootRecord::parse_slot   recovery.rs:1128 / root_record.rs:110
├─ scan_journal                           recovery.rs:2891
│  └─ JournalRecord::parse                journal.rs:272
├─ replay_journal                         recovery.rs:3057
│  ├─ counter_of_the_last_record_the_root_covers  recovery.rs:2962
│  ├─ record_ends_its_publish             recovery.rs:2932
│  ├─ readable_record_of_the_same_publish_follows recovery.rs:2941
│  ├─ judge_back_chain_against_the_previous_record_on_disk  recovery.rs:3005
│  │  ├─ record_offset / back_chain_of    journal.rs:196 / journal.rs:168
│  │  └─ JournalRecord::parse
│  ├─ journal_in_flight_record_limit      system_configuration.rs:88
│  ├─ unit_bytes_for_class                recovery.rs:330
│  └─ crc32_castagnoli                    checksum.rs:43
├─ unit_area_start_of_the_chosen_system_configuration  recovery.rs:814
└─ walk_to_file_in_the_unit_area_starting_at            recovery.rs:3909（走树读文件，按名字匹配 39 个直接调用）
```

| 函数 | 文件:行 | 行数 | 码行 |
|---|---|---|---|
| `recover` | recovery.rs:4222–4301 | 80 | 79 |
| `choose_system_configuration` | recovery.rs:964–1052 | 89 | 84 |
| `mountable_slot_or_refusal` | recovery.rs:838–869 | 32 | 32 |
| `slot_one_found_by_trying_every_slot_spacing_the_format_allows` | recovery.rs:893–948 | 56 | 54 |
| `system_configuration_values_this_reader_accepts` | recovery.rs:706–806 | 101 | 100 |
| `SystemConfiguration::parse_slot` | system_configuration.rs:494–581 | 88 | 84 |
| `choose_root` | recovery.rs:1149–1168 | 20 | 20 |
| `visit_valid_roots` | recovery.rs:1055–1069 | 15 | 15 |
| `visit_valid_roots_with_ring_slots` | recovery.rs:1073–1093 | 21 | 21 |
| `every_root_ring_slot` | recovery.rs:1097–1104 | 8 | 8 |
| `read_root_ring_slot` | recovery.rs:1128–1145 | 18 | 18 |
| `RootRecord::parse_slot` | root_record.rs:110–154 | 45 | 45 |
| `scan_journal` | recovery.rs:2891–2928 | 38 | 38 |
| `JournalRecord::parse` | journal.rs:272–355 | 84 | 79 |
| `replay_journal` | recovery.rs:3057–3229 | 173 | 131 |
| `counter_of_the_last_record_the_root_covers` | recovery.rs:2962–2987 | 26 | 24 |
| `record_ends_its_publish` | recovery.rs:2932–2937 | 6 | 6 |
| `readable_record_of_the_same_publish_follows` | recovery.rs:2941–2953 | 13 | 13 |
| `judge_back_chain_against_the_previous_record_on_disk` | recovery.rs:3005–3037 | 33 | 32 |
| `back_chain_of` | journal.rs:168–175 | 8 | 8 |
| `record_offset` | journal.rs:196–202 | 7 | 7 |
| `journal_in_flight_record_limit` | system_configuration.rs:88–90 | 3 | 3 |
| `unit_bytes_for_class` | recovery.rs:330–338 | 9 | 9 |
| `crc32_castagnoli` | checksum.rs:43–63 | 21 | 21 |
| `unit_area_start_of_the_chosen_system_configuration` | recovery.rs:814–819 | 6 | 6 |
| `walk_to_file_in_the_unit_area_starting_at` | recovery.rs:3909–4182 | 274 | 256 |

### a2 可写挂载取号之前的那一段（`mount_writable` → … → 取号）

入口链：`mount_writable`（mount.rs:4165）→ `mount_writable_with_space_admission`（4177）→ `mount_writable_with_test_only_switches`（4191）→ `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`（4430），后者依次调：

| 次序 | 函数 | 文件:行 | 行数 | 做什么 |
|---|---|---|---|---|
| 1 | `admit_the_writable_device_count` | mount.rs:3675–3685 | 11 | 可写设备数准入 |
| 2 | `refuse_device_identities_handed_in_more_than_once` | mount.rs:3435–3455 | 21 | 盘身份重复就拒 |
| 3 | `choose_system_configuration` | recovery.rs:964–1052 | 89 | 同 a1 |
| 4 | `parameters_of_the_selected_system_configuration_agreeing_with_the_caller_parameters_and_device_table` | mount.rs:3518–3596 | 79 | 调用方参数与盘表对盘上的 |
| 5 | `every_device_reaches_the_unit_area_start` | recovery.rs:1844–1860 | 17 | 盘够不够长 |
| 6 | `read_stage_settled_at_most_on_the_one_reread` | mount.rs:4391–4422 | 32 | 读阶段一遍，见证判真再重读一遍（C554 乙） |
| 6.1 | `read_stage` | mount.rs:4353–4382 | 30 | `choose_root` → `scan_journal` → `replay_journal(…, true)`（mount.rs:4358–4367），都经读缓存 |
| 6.2 | `newer_publish_witness` | mount.rs:4273–4347 | 75 | 系统配置见证的 jsn 对所选那一版 |
| 6.3 | `ReadStageCache::read` | mount.rs:4233–4250 | 18 | 挂载内读缓存（`RefCell<BTreeMap>`，mount.rs:4212） |
| 7 | `journal_position_of_the_selected_version` | mount.rs:3152–3166 | 15 | |
| 8 | `rebuild_previous_version` → `rebuild_version_in_the_unit_area_starting_at` | mount.rs:1033–1052 → recovery.rs:2309–2785 | 20 + 477 | 从盘上重建所选那一版 |
| 9 | `first_txg_of_new_instance` | mount.rs:1097–1115 | 19 | |
| 10 | `tree_identifier_watermark_of_the_ring` | mount.rs:1133–1149 | 17 | |
| 11 | `rebuilt_allocator` | mount.rs:1489–1606 | 118 | 分配器、F 生效值、被抛弃根隔离（下挂 `effective_rollback_floor_under_the_newest_roots_table` recovery.rs:1453、`isolate_slots_referenced_only_by_abandoned_roots` mount.rs:1230、`instance_table_of_the_newest_root_read_at_most_twice` mount.rs:1181） |
| 12 | `establish_instance`（取号之前那段 mount.rs:3694–3837） | mount.rs:3688–4043 | 356（整函数） | 取号之前：`PreviousVersion::units`（mount.rs:957）、`warm_up_publish_txgs`（3054）、`next_instance_generation_to_acquire`（transaction.rs:695）、空间准入 `admit_on_every_device`（admission.rs:421）、`dry_run_of_the_publishes_after_acquisition`（mount.rs:2874–2995，122 行）、逐盘核 `devices_without_the_selected_version`（mount.rs:3182–3221，40 行） |

### a3 取号与回卷（写路径，取号之后）

| 函数 | 文件:行 | 行数 |
|---|---|---|
| `acquire_expected_instance`（establish_instance 在 mount.rs:3838 调） | transaction.rs:797–821 | 25 |
| `write_acquired_instance` | transaction.rs:827–877 | 51 |
| `PoolWriter::roll_back_acquisition`（回卷写） | transaction.rs:440–483 | 44 |

### a4 合计行数（只算 core + format；format 为 0；闭包是推的上界，见口径）

| 组 | 根 | 函数数 | 行数 | 码行 |
|---|---|---|---|---|
| K：前缀判定与施加（M2 的核） | `replay_journal` | 36 | 577 | 527 |
| S1：择系统配置 | `choose_system_configuration` | 31 | 562 | 550 |
| S2：择根 | `choose_root` | 32 | 301 | 301 |
| S3：扫环 | `scan_journal` | 28 | 337 | 332 |
| W：走树读文件 | `walk_to_file_in_the_unit_area_starting_at` | 174 | 3121 | 3081 |
| A：只读冷启动整条 | `recover` | 217 | 4244 | 4141 |
| B：挂载读阶段与前置核 | a2 第 1–6 行 | 86 | 1657 | 1591 |
| C：B 之后、取号之前（不含预演） | a2 第 8–12 行里除预演外的根 | 285 | 5513 | 5407 |
| DRY：取号前预演 | `dry_run_of_the_publishes_after_acquisition` | 454 | 9401 | 9100 |
| PRE：取号之前整段 | B ∪ C ∪ DRY | 591 | 13502 | 13071 |
| D：取号与回卷 | `acquire_expected_instance` | 104 | 1634 | 1591 |
| ALL：PRE ∪ D ∪ A | | 600 | 14049 | 13586 |
| M：整个可写挂载（含取号后写行、暖机、抬 F） | `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread` | 667 | 15686 | 15112 |

算法：`python3 fns.py closure <根…> --stop read,device_identities,device_size_in_bytes,journal_record_offsets_hint,read_at,write_at,write_zeroes_at,size_in_bytes,probe_physical_block_size,barrier,fua_write_at,from,fmt,clone,default`，输出末行 `TOTAL fns=… lines=… code=…` 抄进上表。K 组逐函数（36 个）：recovery.rs 6 个 260 行，journal.rs 5 个 126 行，pointer.rs 6 个 74 行，checksum.rs 5 个 68 行，bytes.rs 11 个 40 行，其余 3 个各 3 行。

另算两个并集：S2 ∪ S3 ∪ K（择根 + 扫环 + 取前缀）`TOTAL fns=50 lines=772 code=722`；再并上 S1（择系统配置）`TOTAL fns=68 lines=1235 code=1173`。

读法（推的）：M1 / M2 要的「择根 + 扫环 + 取前缀」约 770 行，连择系统配置约 1 240 行；`recover` 里四分之三（W 的 3 121 行）是走树读文件，M1 里「输出过 I-7.1 与 I-8.3」那一半用不到它，「有定义、终止」那一半要不要罩住走树，看七·1 把 `Recover` 的边界划到哪。可写挂载取号之前的 1.35 万行里，预演（DRY）一项 9 401 行，它把整条发布路径在分配器拷贝上走一遍。

## (b) 读写盘的抽象、全局状态、线程、unsafe、循环上界

| 项 | 事实 | 出处 |
|---|---|---|
| 读盘抽象 | trait `PoolReader`，四个方法：`device_identities()`、`device_size_in_bytes(device)`、`read(device, offset, length) -> Option<Vec<u8>>`（读不到交 None）、`journal_record_offsets_hint(device, ring_start, ring_bytes) -> Option<Vec<…>>`（None 就全环扫）。没有写方法 | recovery.rs:81–101 |
| `recover` 的签名 | `pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport`：只读，动态分派 | recovery.rs:4222 |
| `PoolReader` 的实现者（core 里） | `[(DeviceIdentity, Device)]` 与 `Vec<(DeviceIdentity, Device)>`（经 `BlockDevice::read_at`，hint 恒 None）；挂载读缓存 `ReadStageCache` | recovery.rs:103、133；mount.rs:4226 |
| `PoolReader` 的实现者（harness） | 层 0 的 `MemoryPool` 与 `CrashImage`（基线叠持久了的写；hint 只给写过的记录槽） | crates/singlefs-harness/src/memory_pool.rs:384、437 |
| 写盘抽象 | trait `BlockDevice`：`read_at`、`write_at`、`write_zeroes_at`、`barrier`、`probe_physical_block_size`、`size_in_bytes`；取号与回卷经 `PoolWriter`（包着 `&mut [(DeviceIdentity, Device)]`）写 | block_device.rs:106–135；mount.rs:3704 |
| 施加记录写不写盘 | 不写。`replay_journal` 只改本地 `rebuilt`（`let mut rebuilt = *root;` 与末条施加处），返回 `Ok((report, rebuilt))` | recovery.rs:3073、3210–3226、3228 |
| unsafe | core 与 format 都 `#![forbid(unsafe_code)]`；`grep -rn "unsafe"` 另一处命中是注释「不写 unsafe」 | core lib.rs:7；format lib.rs:11；block_device.rs:390 |
| 线程 | `grep -rn "std::thread\|thread::spawn\|rayon\|\.spawn(" crates/singlefs-core/src crates/singlefs-format/src \| wc -l` 输出 `0` | 命令输出见 `evidence-greps.txt` |
| 全局状态 | 非测试代码里只有一个：CRC-32C 查表 `static TABLES: std::sync::OnceLock<[[u32; 256]; 8]>`（懒初始化、之后只读）。另一个 `static IMAGE_COUNTER: AtomicU64` 在 `#[cfg(test)] mod tests` 里 | checksum.rs:11；block_device.rs:554（`#[cfg(test)]`）、559 |
| 局部内部可变 | `OnceCell<CodeTwoTreeReadFromDisk>`（映射树首次用到才读）；挂载读缓存 `RefCell<BTreeMap<…>>` | recovery.rs:530、3474；mount.rs:4212 |
| 闭包里的 panic 点（K / A，推的上界） | K：`.expect(` 24、`assert*!` 1、`panic!` 类 0、`.unwrap()` 0；A：`.expect(` 117、`assert*!` 8、`panic!` 类 1、`.unwrap()` 0 | 草稿脚本逐行正则数，范围是 closure_K / closure_A 的函数体 |
| 堆容器（K / A） | K 里 `BTreeMap/BTreeSet` 4 处、`Vec` 4 处；A 里 29 处、96 处。`replay_journal` 的记录入参就是 `&BTreeMap<(InstanceGeneration, u64), JournalRecord>` | recovery.rs:3061 |

### 循环上界

| 循环 | 上界来自 | 出处 |
|---|---|---|
| 择系统配置：逐盘读槽 0、槽 1 | 盘数（`reader.device_identities()`） | recovery.rs:974、984 |
| 全池没有可择的槽 0 时找槽 1 | 槽距档数：代码注释写「上下界之间 … 2033 档，每档读一次」 | recovery.rs:882、901–905 |
| 择根：逐个根环槽 | `ROOT_RING_REGIONS` × 每区槽数 S = 3 × S，S ∈ [4, 16]（mkfs 取 8） | recovery.rs:1097–1104；format lib.rs:238、247、252、256 |
| 扫环：每盘每个记录槽 | 盘数 × min(环长, 盘长 − 环起点) ÷ 4096；有 hint 时是 hint 的条数。默认环长 805 306 368 字节，即每盘 196 608 个记录槽（算术） | recovery.rs:2901–2916；format lib.rs:191、213 |
| 取前缀主循环 | `above.into_iter().take(in_flight_limit)`：在飞上限 = 环长 ÷ 4096 ÷ F，F = 3；默认环下 65 536 条（算术）。候选只收与所选根同实例、水位之上的记录 | recovery.rs:3076–3085、3109；system_configuration.rs:88–90；format lib.rs:221 |
| 点名单元逐个验 | 每条记录的点名项数 × 2 条位置：点名项数受 `payload_end > record_bytes` 拒，≤ (4096 − 311) ÷ 56 = 67（格式常量 `JOURNAL_NAMED_ENTRIES_PER_RECORD = 67`） | recovery.rs:3174–3187；journal.rs:329–333 的 `payload_end` 判；format lib.rs:200、203、206 |
| 反向链重读前一条 | 盘数（注释「迭代次数的上界是池里的盘数」） | recovery.rs:3023–3035 |
| 锚点、同发布后继 | `records` 条数（BTreeMap 的 values / range） | recovery.rs:2941–2953、2962–2987 |
| 读实例表链 `loop` | 链的片数：每片身份带片序号，注释写「轮数不超过一块盘装得下的 32 KiB 单元数」，成环在身份判上断 | recovery.rs:2029–2031（注释）、2040 |
| 树的递归读 | 树高：层级字段是 u8，孩子层级必须等于父层级减一，`units_seen` 拒同一节点被两条父条目引用；扇出受 16 KiB 节点装得下的条目数 | code_two_tree.rs:956–1010（`read_node_and_its_subtree`）；allocation_record_tree.rs:681；extent_tree.rs:822、981（递归调用点） |
| A 闭包里的 `while` / `loop` | 只有上面那一个 `loop`（脚本扫 A 闭包全部函数体：`== A while/loop 1`，`recovery.rs:2040 read_instance_table_chain: loop {`） | 草稿脚本输出 |

## (c) 哈希、CRC、AEAD：函数与调用点；有界模型检查要桩掉什么

### c1 函数

| 函数 | 文件:行 | 是什么 |
|---|---|---|
| `crc32_castagnoli(bytes) -> u32` | checksum.rs:43–63 | CRC-32C，slicing-by-8，查表来自 `castagnoli_tables`（checksum.rs:10–35，`OnceLock` 懒建 8 × 256 表） |
| `wide_checksum_with_field_zeroed` | checksum.rs:68–80 | 32 字节校验和字段：字段按 0 参与算 CRC-32C，结果写前 4 字节 |
| `wide_checksum_field_holds(bytes, cover_end, field_offset) -> bool` | checksum.rs:84–88 | 判上面那个字段对不对 |
| `back_chain_of(previous_record_bytes) -> u32` | journal.rs:168–175 | 反向链 = CRC-32C(前一条的 311 字节头，header_csum 按 0) |
| 哈希 / Merkle / AEAD | 没有。`grep -rn 'aead\|Aead\|AEAD\|chacha\|aes\|blake\|sha2\|sha256\|xxh\|siphash\|Hasher' crates/singlefs-core/src crates/singlefs-format/src | wc -l` 输出 `0`（连注释都没有）。指针头部的 MAC 16 / nonce 12 在第一版恒 0，读路径判它们全 0，非 0 当损坏 | pointer.rs:98、107、136、155 |

文件头注释写明「checker 另写一份按位的，两份对同一段字节必须给同一个数」（checksum.rs:3）。

### c2 调用点（在 A 闭包 `recover` 里，脚本逐行扫，12 处）

| 调用点 | 所在函数 | 调的 |
|---|---|---|
| checksum.rs:78 | `wide_checksum_with_field_zeroed` | `crc32_castagnoli` |
| checksum.rs:87 | `wide_checksum_field_holds` | `wide_checksum_with_field_zeroed` |
| journal.rs:174 | `back_chain_of` | `crc32_castagnoli` |
| journal.rs:277 | `JournalRecord::parse`（记录头校验和） | `wide_checksum_field_holds` |
| journal.rs:332 | `JournalRecord::parse`（点名载荷校验和） | `crc32_castagnoli` |
| recovery.rs:355 | `read_unit_via_locations`（整单元校验和对位置条目） | `crc32_castagnoli` |
| recovery.rs:3029 | `judge_back_chain_against_the_previous_record_on_disk` | `back_chain_of` |
| recovery.rs:3185 | `replay_journal`（点名单元逐个验） | `crc32_castagnoli` |
| root_record.rs:116 | `RootRecord::parse_slot` | `wide_checksum_field_holds` |
| system_configuration.rs:498 | `SystemConfiguration::parse_slot` | `wide_checksum_field_holds` |
| unit.rs:350 | `check_common_prefix`（单元头） | `wide_checksum_field_holds` |
| unit.rs:362 | `check_payload_checksum`（单元载荷） | `crc32_castagnoli` |

PRE 闭包（取号之前整段）里 26 处，多出来的是预演走发布路径时的写侧（`JournalRecord::to_bytes` journal.rs:255、261，`RootRecord::to_slot` root_record.rs:102，`build_*` unit.rs:114、213、276 等）；D 闭包（取号与回卷）9 处。清单在草稿脚本输出里，没全抄。

### c3 做有界模型检查（Kani 这一类）要桩掉的（全是推的，本机没有 Kani，一样都没试）

| 要桩的 | 为什么（推的） | 桩成什么（推的） |
|---|---|---|
| `crc32_castagnoli` | 对 4 096 / 16 384 / 32 768 字节的符号缓冲区逐字节查表，SAT 规模随字节数线性涨；还经 `OnceLock` 静态表 | 未解释函数：同输入同输出、输出任意 u32；已定项 14 的「不许读成验了完整性」照这个桩成立 |
| `wide_checksum_field_holds` | 同上，且它是自证判的唯一入口（根槽、系统配置槽、记录头、单元头） | 任意 bool，或「过 / 不过」当成镜像生成器的一个自由位 |
| `back_chain_of` | 311 字节 CRC | 随 `crc32_castagnoli` 一起成未解释函数 |
| `PoolReader` | trait 对象；真实现要拷 4 KiB–32 KiB 的 `Vec<u8>` | 写一个有界符号读者：盘数、每区槽数 S、hint 给的记录槽数都取小常数；`read` 交符号字节或 None |
| `JournalRecord::parse` / `RootRecord::parse_slot` | 逐字段解 4 096 字节 | `replay_journal` 是 `pub`、记录入参直接是 `&BTreeMap<(InstanceGeneration, u64), JournalRecord>`（recovery.rs:3057–3063），M2 的检查可以绕开解析、直接造符号记录表；但 `judge_back_chain_against_the_previous_record_on_disk` 在里面还会经 `reader.read` 重读前一条并 `JournalRecord::parse`（recovery.rs:3024–3028），这一步要么随读者一起桩、要么让符号读者交回能解出同一条记录的字节 |
| `BTreeMap` / `Vec` | 堆分配与平衡树在 CBMC 里展开很贵 | 不桩，靠把记录条数限到很小（在飞上限在 harness 里另设）；这一格要量了才知道 |
| AEAD | 实现里没有 | 不用桩；D9 加密进来之后按已定项 14 当未解释函数 |

## (d) M1 / M2 / M3 在实现里落在哪几行；checker 判 I-7.1、I-8.3 的地方；I-8.4

### d1 实现侧

| 义务 | 判定 | 文件:行 |
|---|---|---|
| M1「有定义」 | `recover` 对任何读者都交回 `RecoveryReport`，走不下去的写成 `RecoveryOutcome::Failed { root, failure }`，不以 `Result` 外抛 | recovery.rs:4222–4301；结局类型 recovery.rs:281–293 |
| M1「终止」 | 循环与递归的上界见 (b) 的上界表；A 闭包里唯一的 `loop` 是实例表链 | recovery.rs:2040 |
| M1「不 panic」这一格 | 没有判定，靠逐处 `expect` 的前置条件：例如 `record_offset` 的 `checked_rem(...).expect(...)` 靠择系统配置拒短环；`JournalRecord::parse` 里 `assert_eq!(reader.position(), 311)` | journal.rs:196–202；journal.rs:325（`assert_eq!` 那句） |
| M1 输出过 I-7.1 | 择根：遍历根环全部槽，只收自证过的，取 `(checkpoint_txg, instance)` 最大；一条都没有交 `None` | recovery.rs:1149–1168 |
| | `recover` 把 `None` 报成 `Failed { root: None, failure: NoValidRoot }` | recovery.rs:4238–4248 |
| | 可写挂载读阶段同样：`choose_root(...).ok_or(RecoveryFailure::NoValidRoot)?` | mount.rs:4358–4359 |
| M1 输出过 I-8.3 | 候选只收与所选根同实例、`(instance, txg)` 严格大于所选根的记录，按 `(instance, counter)` 排 | recovery.rs:3076–3083 |
| | 链首锚点 = 所选根那次发布里带末条标志的那一条的 jsn + 1；带标志的多于一条报错、一条不施加 | recovery.rs:3094–3096；2962–2987 |
| | 锚点读不出时链首要 `txg = 根 txg + 1` 且本次发布内序号 1；根 txg 加一溢出报错 | recovery.rs:3099–3106、3114–3118 |
| | 前缀最多在飞上限条 | recovery.rs:3084–3085、3109 |
| | jsn 严格连续：`(record.instance, record.counter) != expected_key` 就 `break` | recovery.rs:3110–3113 |
| | 反向链不等就断（前一条不在盘上不判） | recovery.rs:3121–3126 |
| | 下一个期望 jsn | recovery.rs:3158 |
| M2 前缀不切发布 | 本次发布已过关、末条还没到的记录先攒在 `records_of_the_open_publish`，不施加 | recovery.rs:3108、3198 |
| | 末条没到就 `continue`；任何 `break` 都在施加之前，攒着的那几条随循环结束丢掉 | recovery.rs:3199–3202 |
| | 施加只发生在带末条标志的那一条过关之后：计数、W、换根四个字段 | recovery.rs:3203–3226 |
| | 新发布首条序号必须是 1 | recovery.rs:3130–3134 |
| | 末条没到而下一条换了 txg、发布内序号跳号、上一条没提交标记而换了事务号，都断 | recovery.rs:3135–3157 |
| | 末条不带提交标记，断 | recovery.rs:3161–3163 |
| | 带末条标志之后同一 `(instance, txg)` 还有读得出的记录，断在带标志那一条 | recovery.rs:3168–3172；2941–2953 |
| | 点名单元有一个校验和不对，断 | recovery.rs:3173–3191 |
| | 末条标志怎么认：记录标志字节位 0 | recovery.rs:2932–2937；journal.rs:55–63（`from_record_flags_byte`） |
| M3 幂等 I-8.4 | 实现里没有判定。`grep -rn "幂等\|idempot" crates --include=*.rs \| wc -l` 输出 `0` | evidence-greps.txt |
| | 推的：冷启动那条路只读（`PoolReader` 没有写方法，recovery.rs:81–101；施加只改本地 `rebuilt`，recovery.rs:3073、3228），同一份镜像上 `recover` 跑两次读的是同一份字节；盘上状态会变的是可写挂载取号之后（mount.rs:3838 取号、3870 起写行与暖机），I-8.4 在那一段才有内容 | |
| M3 单调（后半） | 没有判定，也没有落点（已定项 1 原文「后半全仓无落点」） | |

### d2 checker 与别的判定侧

| 项 | 事实 | 文件:行 |
|---|---|---|
| I-7.1 在池级 checker | `root_ring_judgements.judge("I-7.1", !roots.is_empty(), …)`；S 空时之后的根判报不适用并返回 | crates/singlefs-checker/src/walk.rs:5852–5861（5852–5854 那一判，5855–5861 空集时报 I-7.3 不适用并返回） |
| | 列在已实现清单 `IMPLEMENTED_INVARIANTS` 里 | crates/singlefs-checker/src/image.rs:68、92 |
| | kb 状态「已实现（2026-09-14 … 层 0 每个崩溃状态都判）」 | .claude/kb/invariants.md:52 |
| I-8.3 在池级 checker | 没有。不在 `IMPLEMENTED_INVARIANTS`；`grep -rn "I-8\.3" crates --include=*.rs \| wc -l` 输出 `0` | image.rs:68–119；evidence-greps.txt |
| | kb 状态「未实现」，并写着「这一条不是前缀纪律的全部」与两句判据补句（前缀不得停在一次发布中间；在飞上限就是前缀最多 N 条） | .claude/kb/invariants.md:86 |
| 与 I-8.3 相邻、checker 已判的 | I-8.6 反向链、I-8.7 事务号、I-8.8 提交标记、I-8.9 发布内序号与末条标志：判的是环里全部自证过的记录，不是恢复取到的那段前缀 | walk.rs:5097（`judge_journal_back_chain`）、5253（`judge_commit_markers_per_transaction`）、5386（`judge_publish_ordinals_and_last_record_flags`）；调用处 walk.rs:5838–5850 |
| I-8.4 | 没有判定。kb 状态「未实现」；crates 里「幂等」「idempot」零命中 | .claude/kb/invariants.md:87；evidence-greps.txt |
| 记录核对器（D13 已定项 7，不编号） | 两条判据：「根在盘上而那次发布的记录一条都不在」；「恢复自称的 txg ≥ 某次发布而那次发布某个单元全部副本缺席」。入参里有恢复报的 `effective_root`，没有前缀本身，也没有第二次恢复的结果 | crates/singlefs-checker-tier/src/crash.rs:48–60、105–164 |
| 崩溃恢复对模型的判（crash injection / 故障注入） | `crash_recovery_disagreement`：走读不许失败；择到的根是模型提交过的某一版；不许比盘上已持久的最新根槽旧；内容逐字节同。不看前缀、不看 jsn | crates/singlefs-harness/src/model.rs:2246；调用 crates/singlefs-checker-tier/src/crash_injection.rs:858、1615，crates/singlefs-harness/src/fault_injection.rs:2352、2359 |
| 层 0 的 oracle | `oracle_violation_for_versions`（版本表由调用方按固定脚本手写） | crates/singlefs-checker-tier/src/crash.rs:951 |
| crash refinement 代码 | `grep -rln refinement crates research --include=*.rs --include=*.py --include=*.sh \| wc -l` 输出 `0` | evidence-greps.txt |

读法：I-7.1 有 checker 判定且层 0 全量跑；I-8.3 与 I-8.4 在 `crates/` 里零判定，M2 那一句（前缀不切发布）今天只住在实现的 `replay_journal` 里，没有独立的一侧判它。

## (e) 理想模型 `model.rs` 里有没有恢复的语义

| 项 | 事实 | 文件:行 |
|---|---|---|
| 文件头 | 「不建崩溃：第 1 件的历史里没有断电、没有设备错，每一步的写都落完，所以环里最新的那条根就是最后写出的那一条、journal 里没有要施加的记录」「第 3 件（崩溃注入）接上时要加『这个崩溃状态恢复到的版本在不在允许集合里』」 | crates/singlefs-harness/src/model.rs:8–9 |
| 冷启动读回 | `answer_cold_start_recover`：所选根 = 环里 `(txg, 实例)` 最大那条，读回它那一版的文件；没有施加记录这一步 | model.rs:1625；类型注释 model.rs:363 |
| 由记录重建（最接近重放的一处） | `answer_mount_writable_with_the_roots_rebuilt_from_their_records`：给一份「根槽没落盘的根」名单，择根落到名单之外最大的那条，再把与它同实例、更新的那几条根整次接上（换实例就停），W 取它们的最大事务号。粒度是整次发布，没有记录、jsn、反向链、撕裂与「前缀停在发布中间」 | model.rs:1309–1354 |
| 最新根读不出 | `answer_mount_writable_with_the_newest_root_unreadable`：取号之前拒，模型状态不变 | model.rs:1276 |
| 崩溃之后允许什么 | `crash_recovery_disagreement` 四条判据（见 d2）；提交过的版本目录由 `committed_versions` 攒 | model.rs:2232–2246、811 |
| 结论 | 模型里没有「崩溃状态 → 恢复读到什么」的逐记录语义：它答的是「落到哪一版允许」，以整次发布与根为单位。M2 要的「任一 jsn 连续前缀的重放结果 = 整数个发布」在模型里没有对应的函数 | |

## (f) 本机工具链（只查不装，2026-09-30）

```
$ cargo --version
cargo 1.98.0 (797e8a9bc 2026-08-05)
$ rustc --version
rustc 1.98.0 (88d9e12ae 2026-08-18)
$ cargo install --list
cargo-fuzz v0.13.2:
    cargo-fuzz
cargo-mutants v27.1.0:
    cargo-mutants
sashiko v0.2.5:
    benchmark
    review
    sashiko
    sashiko-cli
$ which kani cargo-kani z3 java tlc apalache-mc verus creusot 2>&1; echo "which rc=$?"
which rc=1
$ for t in kani cargo-kani z3 java tlc apalache-mc verus creusot; do printf '%s: ' $t; command -v $t || echo 'not found'; done
kani: not found
cargo-kani: not found
z3: not found
java: not found
tlc: not found
apalache-mc: not found
verus: not found
creusot: not found
$ python3 -c 'import z3'
Traceback (most recent call last):
  File "<string>", line 1, in <module>
ModuleNotFoundError: No module named 'z3'
$ pip list 2>/dev/null | grep -i z3; echo "pip grep rc=$?"
pip grep rc=1
$ pip --version
/bin/bash: line 1: pip: command not found
$ python3 --version
Python 3.12.3
$ python3 -m pip --version 2>&1; echo "rc=$?"
/usr/bin/python3: No module named pip
rc=1
$ ls ~/.cargo/bin
benchmark cargo cargo-clippy cargo-fmt cargo-fuzz cargo-miri cargo-mutants clippy-driver review rls rust-analyzer rustc rustdoc rustfmt rust-gdb rust-gdbgui rust-lldb rustup sashiko sashiko-cli
$ grep -n -i 'kani\|verus\|prusti\|creusot\|loom' Cargo.toml crates/*/Cargo.toml; echo "cargo.toml grep rc=$?"
cargo.toml grep rc=1
```

| 项 | 结果 |
|---|---|
| Rust | cargo 1.98.0 / rustc 1.98.0；仓根没有 `rust-toolchain.toml` 也没有 `rust-toolchain`（`cat` 报 No such file） |
| cargo 子命令 | 装了 cargo-fuzz、cargo-mutants、sashiko；`~/.cargo/bin` 里另有 `cargo-miri` |
| Kani / Verus / Creusot | 都不在 PATH |
| Z3 | 可执行文件不在 PATH；python3 3.12.3 里 `import z3` 失败；`pip` 命令不存在、`python3 -m pip` 也没有，所以 `pip list \| grep -i z3` 那条是空的，不代表查过包列表 |
| TLA+（TLC）/ Apalache | 不在 PATH；`java` 也不在 PATH |
| Cargo.toml（工作区根 + 五个 crate） | kani / verus / prusti / creusot / loom 零命中（grep 退出码 1） |

系统 python3 连 pip 模块都没有（`python3 -m pip --version` 报 No module named pip）；别的 venv 里装没装 z3 没查。

## 结论

1. M2 的全部判定在 `replay_journal` 一个函数里（recovery.rs:3057–3229，173 行；按名字跟下去的闭包 36 个函数、577 行），它是 `pub`、单线程、无 unsafe，记录表直接作入参，读盘只为反向链重读与点名单元验校验和，经 `PoolReader`。择根 + 扫环 + 取前缀的并集约 770 行，连择系统配置约 1 240 行；`recover` 整条 4 244 行，其中走树读文件 3 121 行。可写挂载取号之前约 1.35 万行，其中取号前预演 9 401 行。
2. 校验和只有 CRC-32C 一种（`crc32_castagnoli` 与两个 32 字节字段包装、`back_chain_of`），A 闭包里 12 个调用点；没有密码学哈希，没有 AEAD，指针里的 MAC / nonce 第一版恒 0。
3. I-7.1 有池级 checker 判定（walk.rs:5852）；I-8.3、I-8.4 在 `crates/` 里零判定，kb 状态都是「未实现」（invariants.md:86、87）。模型不建崩溃、没有逐记录的重放语义（model.rs:8–9），最近的一处是整次发布粒度的「由记录重建」（model.rs:1309）。
4. 本机 Kani、Z3（可执行与 python 模块）、TLC、Apalache、Java、Verus、Creusot 都没有；`pip` 也没有；Cargo.toml 里没有任何形式化验证依赖。三条候选路在本机都要先装工具才能量代价。

什么现象会推翻：
- 第 1 条：有别的函数在 `replay_journal` 之外决定施加哪几条记录（例如挂载路径另写一份前缀），`grep -rn "replay_journal\|scan_journal" crates/singlefs-core/src` 会多出调用者之外的前缀实现；闭包行数是按名字跟的上界，换成按类型解析（rust-analyzer 调用图）会变小。
- 第 3 条：`crates/` 里出现 `"I-8.3"` 或 `"I-8.4"` 字面量的判定，或 `IMPLEMENTED_INVARIANTS` 收进它们。
- 第 4 条：工具装在 PATH 之外（例如 `~/.opam`、某个 venv、容器里），这一轮只查了 PATH、系统 python3 与 Cargo.toml。

## 没做什么

- 没改仓内任何文件；没跑 cargo build / test / miri，没跑重型测试，没跑门禁；没提交。
- 闭包是脚本按名字匹配的上界，没用编译器级的调用图核对；没逐个读 W（走树）与 DRY（预演）闭包里的函数。
- 没查 PATH 之外的安装位置（venv、opam、容器）；`herd7` 在 `~/.opam/default/bin/`（里程碑文件七节写的），这一轮没问、也没核。
- 桩法（c3）全是推的，本机没有 Kani，一样都没试。
- 没为七·1 写岔路单或代价数，那是主 agent 的活。

## 草稿文件（都在 `/tmp/claude-1000/m3-r1/formal-facts/`，不是编译目录也不是仓副本）

- `fns.py`：函数提取与闭包脚本；`fns.tsv`：core + format 全部带体 fn 的起止行与行数。
- `closure_{K,S1,S2,S3,S23K,S123K,W,A,B,C,DRY,PRE,D,ALL,M}.tsv`：各组闭包的逐函数清单与合计。
- `evidence-greps.txt`：线程、unsafe、static、I-8.3、幂等、refinement、AEAD 几条 grep 的原样输出。
- `bd_fns.txt`：block_device.rs 的 fn 行（查接口方法名用）。
