# 代码审阅报告：singlefs-format 与 singlefs-core 的格式 / 编解码层（13 个文件）

只读审阅，没跑 cargo、没跑脚本。结论全部来自读代码与手算偏移；「已确认」指读代码能复现推理链。

### 1. journal 环长是 mkfs 参数，单元区起点却是写死的 784 MiB；check_geometry 按另一条式子判单元区越界
- 位置：crates/singlefs-core/src/make_filesystem.rs:152-175、:47-48；crates/singlefs-format/src/lib.rs:211-212、:266；crates/singlefs-core/src/system_configuration.rs:390；crates/singlefs-core/src/journal.rs:178-183
- 严重度：高
- 置信度：已确认
- 代码做了什么：
  ```
  if ring_bytes > smallest / 4 { return Err(JournalRingTooLargeForDevice{..}) }            // make_filesystem.rs:153
  let unit_area_start = (JOURNAL_RING_START_SLOT + ring_bytes / SLOT_BYTES) * SLOT_BYTES;   // make_filesystem.rs:169
  pub const INSTANCE_TABLE_SLOT: SlotNumber = SlotNumber(UNIT_AREA_START_SLOT);             // make_filesystem.rs:47（恒 50176 = 784 MiB）
  writer.put_u64(UNIT_AREA_START_SLOT);                                                     // system_configuration.rs:390
  ```
- 应该做什么 / 为什么错：环长只受「≤ 最小盘容量 / 4」一条约束，没有上界 768 MiB，也不要求是 16 KiB（或 4 KiB）的整数倍；而单元区起点、实例表与树表第 0 版的落点、分配器的单元区起点（allocator.rs:236 减的也是 `UNIT_AREA_START_SLOT`）全是常量 50176。check_geometry 第三判用的是「16 MiB + 环长」这条式子，与真正用的常量不是一个数。要么把环长钉死在 `UNIT_AREA_START_SLOT − JOURNAL_RING_START_SLOT` 个槽（拒掉别的值），要么让单元区起点随环长走（系统配置里那 8 字节写真实值、各处读它）。另外环长 < 4096 时 `record_offset` 里 `ring_bytes / JOURNAL_RECORD_BYTES` = 0，`% ring_slots` 除零 panic；< 12288 时在飞上限为 0。
- 失败场景：
  - 环长 > 768 MiB：4 GiB 盘、`journal_ring_bytes = 1 GiB`（≤ 4 GiB / 4，过闸）。journal 环占 [16 MiB, 1040 MiB)，实例表单元（784 MiB）与树表单元（784 MiB + 32 KiB）都在环里；计数器 196609 的那条记录（`(196609−1) × 4096 + 16 MiB = 784 MiB`）写下去就盖掉实例表第 0 片，分配器也会把 784 MiB 起的槽发给数据单元，与环重叠。静默损坏，没有任何检查会红。
  - 环长 < 768 MiB：100 MiB 盘、环长 16 MiB。三判全过（16 ≤ 25；根环末端 7 MiB+32 KiB；算出来的「单元区起点」32 MiB + 48 KiB ≤ 100 MiB），mkfs 先把根环与 journal 环清零（已经改了盘），再往 784 MiB 写实例表时得到 `BlockDevice(OutOfRange)`——报错成员不对，且拒绝发生在破坏性写之后，而 `UnitAreaBeyondDevice` 本该在任何写之前拦下它。
  - 环长不是 16 KiB 整数倍（例 768 MiB + 4096）：`ring_bytes / SLOT_BYTES` 截断，第三判按 784 MiB 算，环实际盖到实例表单元的头 4 KiB。

### 2. 固定结构的几何没有一条互不重叠的检查：槽距或 physical_block_size 取大 / 取小时，系统配置槽、三个根环区域、journal 环互相覆盖
- 位置：crates/singlefs-core/src/make_filesystem.rs:147-190、:259-261、:327-328、:359-363；crates/singlefs-core/src/root_ring.rs:94-137；crates/singlefs-format/src/lib.rs:513-516
- 严重度：高（触发要求槽距 > 6 MiB ÷ S（S = 8 时 > 768 KiB），或系统配置槽 1 碰根环区域 0 要槽距 > 1 MiB − 4 KiB，或调用方给出不一致的槽距 / pbs；条件满足时静默覆盖根记录）
- 置信度：已确认（手算偏移）
- 代码做了什么：
  ```
  let root_ring_end = ring_end(spacing, S); if root_ring_end > smallest { Err(RootRingBeyondDevice) }   // make_filesystem.rs:159-168，只比设备末尾
  DeviceOffsetInBytes(ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES)  // root_ring.rs:96，区域间距恒 3 MiB
  slots_per_region.count() * u64::from(fixed_structure_slot_spacing)                                // root_ring.rs:126，区域长 = S × 槽距，无上界
  let offset = DeviceOffsetInBytes(slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing)); // make_filesystem.rs:360-362
  let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");   // make_filesystem.rs:327
  ```
- 应该做什么 / 为什么错：`MakeFilesystemParameters.geometry` 里的槽距、pbs、S 都是调用方直接给的，mkfs 不核：(a) 槽距 = `slot_spacing_for(io_min)`、且 ≥ 4096；(b) 两个系统配置槽 `2 × 槽距 ≤ 根环基址 1 MiB`；(c) 区域长 `S × 槽距` 不越过下一个同盘区域：区域归属写死 [0, 1, 0]，同在盘 0 上的区域 0 与区域 2 起点相距 6 MiB，区域长超过 6 MiB 就重叠（make_filesystem.rs:259-261 注释说的「互不重叠」要的是 ≤ P × chunk = 3 MiB）；(d) 根环末端 `7 MiB + S × 槽距 ≤ journal 起点 16 MiB`；(e) `457 ≤ pbs ≤ 槽距`。format 的测试 lib.rs:513-516 断言「槽宽不超过槽距下限，两槽永不重叠」，但这个性质只在槽距 ≥ 4096 时成立，mkfs 从不判。make_filesystem.rs:259-261 的注释说几段「互不重叠」，同样没有检查撑着。
- 失败场景：
  - `io_min = 1 MiB`（mdraid chunk 1 MiB、dm-thin 大块都给得出；D2 已定项 19 依据的 E129 自己就记了 io_min 65536 的真设备），`slot_spacing_for` 给 1 MiB，S = 8：系统配置槽 1 在 [1 MiB, 1 MiB + 4 KiB)，恰好是区域 0 槽 0；区域 0 占 [1, 9) MiB、区域 2 占 [7, 15) MiB，二者都在盘 0 上，区域 0 槽 6 / 7 与区域 2 槽 0 / 1 同址。txg 18（区域 0 槽 6）盖掉 txg 2 的根，之后 txg 26（区域 2 槽 0）又盖掉 txg 18；每次写系统配置槽 1 都毁掉区域 0 槽 0 上的根（txg 0、24、48…），反过来也毁掉系统配置槽 1。分配器的 `RootRingOccupancy` 以为这些根都在，回退候选集与择根都建在不存在的根上。
  - 槽距 2 MiB、S = 8：区域 2 末端 23 MiB，盖进 journal 环头 7 MiB。
  - 调用方给槽距 1024（没走 `slot_spacing_for`）：系统配置两槽 [0, 4096) 与 [1024, 5120) 重叠，写槽 1 撕掉槽 0。
  - pbs = 256：`root.to_slot(256)` 里 `assert!(slot_bytes >= 457)` 在 mkfs 中途 panic（此时已清零根环、journal 环并写了两个单元），而不是在 check_geometry 里返回错误。pbs = 8192、槽距 4096：每条根写 8 KiB，盖到同区域下一个槽。

### 3. 系统配置读者不看 compat_ro 位图：带不认识的 compat_ro 位的池照样可写挂载
- 位置：crates/singlefs-core/src/system_configuration.rs:434-438、:537-544
- 严重度：中
- 置信度：已确认（`grep -rn -i "compat_ro"` 在 crates 下 0 命中；recovery.rs / mount.rs 里只有 incompat 的分流）
- 代码做了什么：
  ```
  let incompat = &slot[FEATURE_BITS_OFFSET..FEATURE_BITS_OFFSET + FEATURE_BITMAP_BYTES];   // 只切了 [6, 38)
  unknown_in_first_byte == 0 && !unknown_in_rest && has_layout_identity
  ```
- 应该做什么 / 为什么错：feature bits 96 字节是 incompat / compat_ro / compat 各 256 位（layout/01-first-txn.md 自举头那一行）。`.claude/rules/fs-design.md`「feature bit 分三档」写明 compat_ro =「不认识只读挂」。第一版没有登记任何 compat_ro 位，所以 [38, 70) 里任何一位非 0 都是不认识的位，读者至少要把它报出来让挂载降成只读；现在 `parse_slot` 把整段跳过，`SystemConfiguration` 里也没有字段带出去。
- 失败场景：新版本写了一个 compat_ro 位（它的语义正是「旧读者可以读、不能写」），这版读者按可写挂载接着发布，改坏新版本维护的结构，没有任何告警。

### 4. 单元与指针读者不守「预留位恒 0、非 0 判损坏」：算法类型、压缩码、压后长度、extent 偏移、29 字节预留位、码 2 预留 2、码 2 / 码 3 的补齐区都整段跳过
- 位置：crates/singlefs-core/src/unit.rs:362-391、:425-458、:486-513；crates/singlefs-core/src/pointer.rs:96-102；crates/singlefs-core/src/root_record.rs:137（之后的算法类型 / nonce / MAC 不读）
- 严重度：中
- 置信度：已确认（core 与 checker 里都没有对这几段的判断：`grep -rn "NONCE_MAC_ALGORITHM_RESERVED_BYTES"` 在 core 只用来算偏移，checker lib.rs:314 / :382 / :534 同样只算偏移）
- 代码做了什么：
  ```
  reader.skip(16 + 12 + 1 + 1 + 2 + 2);        // pointer.rs:97：MAC、nonce、算法类型、压缩算法码、压后长度、extent 偏移
  reader.skip(4 + 2);                          // unit.rs:373：载荷 CRC 与码 2 预留 2
  let entries_start = header_end + reserved_bytes();   // unit.rs:383：29 字节预留位不看
  let records_start = header_end + reserved_bytes();   // unit.rs:450：码 3 同；记录区之后的补齐也不看
  ```
- 应该做什么 / 为什么错：invariants.md I-2.4 逐字：「那 29 字节由『恒 0、读者遇到非 0 一律判该结构损坏』这条规则守（已定项 17）」；`.claude/rules/fs-design.md` 把「每个写的算法类型字段」列为成立的格式分支，理由是「遇到不认识的算法类型只能显式失败，不会静默按错算法校验」。D19 已定项 11 的压缩算法码「0 = 不压缩，第一版写 0」。现在这些字节被合法写成非 0（新版本开了加密或压缩）时，本版读者按未加密、未压缩、extent 偏移 0 去解，得到的是看起来合法的错字节——正是 fs-design 说的「静默误读」。29 字节预留位落在载荷 CRC 里，随机翻位会被 CRC 抓到，所以这一条挡的是「合法写成非 0」而不是随机损坏。码 2 / 码 3 的补齐区（I-2.3 说恒 0）在 core 读者里也不判，只有码 1 由 `data_unit_payload`（unit.rs:520）判。
- 失败场景：一个 DataPointer 头部偏移 29 的压缩算法码 = 1、压后长度 = 9000：`DataPointer::read_from` 交回的结构里根本没有这两个字段，读路径按声明长度把压缩后的字节当文件内容返回。一个码 1 单元 29 字节预留位里的算法类型那一字节 = 1（载荷 CRC 按改后的字节重算）：`parse_data_unit` 照常 Ok，读者把密文当明文交出。

### 5. 自证结构的读者跳过多个有登记值的字段与补齐区：格式版本、校验和算法标识、加密类型、结构常量、补齐恒 0
- 位置：crates/singlefs-core/src/system_configuration.rs:427-498；crates/singlefs-core/src/root_record.rs:110-154；crates/singlefs-core/src/journal.rs:252-329
- 严重度：中（格式版本、加密类型）/ 低（其余）
- 置信度：已确认
- 代码做了什么：
  ```
  if bytes.len() < slot_bytes() || bytes[..4] != SYSTEM_CONFIGURATION_MAGIC { ... }   // system_configuration.rs:428，偏移 4 的格式版本从不读
  reader.skip(16 + 4 + 1);                                                              // :445，偏移 138 的校验和算法标识跳过
  reader.skip(32 + 16 + 12 + 4 + 1 + 1 + 80 + 4 + 4 + 4 + 4);                           // :449，加密类型（偏移 219）、节点大小、单元大小、落点粒度、位置条目宽跳过
  reader.skip(1); // 算法类型                                                            // journal.rs:264
  ```
- 应该做什么 / 为什么错：
  - 格式版本：单元头的读者判 `!= FORMAT_VERSION` 拒收（unit.rs:300），系统配置写了 `FORMAT_VERSION`（:325）读者却不看，两边不一致。
  - 校验和算法标识：I-2.2 写「读到 0 或未登记的码一律拒绝挂载」，D18 已定项 17 写「挂载时按这个字节选算法」。invariants.md 把 I-2.2 登记为「未实现（条款已写、检查仍欠）」，这里只说明 core 读者这一侧同样没有。
  - 加密类型（偏移 219）：写 0 = 关；读者不看，加密开着的池会被当成不加密的池挂上。
  - 单元大小 / 落点粒度 / 位置条目宽 / 节点大小 / 记录尺寸 / R / P / chunk / 根环基址 / 单元区起点：写的时候照格式常量写，读的时候不比对，一律用编译期常量。与第 1 条连着：单元区起点在盘上有一格，读者从不读它。
  - 补齐恒 0：D18 已定项 17 写自证结构（根记录、系统配置槽、journal 记录头）「补齐恒为 0」。三个读者都只核校验和，[457, pbs)、[489, 4096)、记录里点名项之后的补齐区不判 0。
  - 盘上读来的 `journal_ring_bytes` 与槽距不做任何范围判：recovery.rs:1833 起在 `journal_record_offsets_hint` 给不出时按它生成 `0..ring_bytes / 4096` 个偏移，一份自证得过而环长字段离谱的槽会让扫描分配巨量偏移。
- 失败场景：别的实现写了格式版本 2、布局身份位照旧的系统配置槽，这版读者照收，按版本 1 的字段表解后面所有字段。

### 6. 位置条目的升序断言写成 `<=`，I-2.5 要的是严格升序；读者一侧不判
- 位置：crates/singlefs-core/src/pointer.rs:136-139、:177-180、:150-162、:191-207；crates/singlefs-core/src/make_filesystem.rs:125-145
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```
  assert!(self.locations[0].device <= self.locations[1].device, "位置条目按设备身份升序（I-2.5）");
  ```
- 应该做什么 / 为什么错：I-2.5：「按设备身份严格升序……两条位置条目设备身份相同或逆序，一律判损坏」，只有整条 86 字节全零的指针豁免。写者的断言放过了两条都是同一块盘的非零指针；`location_entries` 按调用方给的身份表排序取前两个，身份表里有重复时就写出这种指针。读者 `read_from` 对顺序一概不判（checker 判，core 读者不判）。
- 失败场景：`location_entries(&[DeviceIdentity(0), DeviceIdentity(0)], slot, unit)` 返回两条设备 0 的条目，`NodePointer::write_to` 不报错，写出一条违反 I-2.5 的指针。mkfs 今天由 check_geometry 的区域归属判顺带挡住，别的调用方没有这层保护。

### 7. 条目宽 / 记录宽为 0 而条目数 / 记录数非 0 的节点照收，交回的条目比头里自述的少
- 位置：crates/singlefs-core/src/unit.rs:377-391、:447-458
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```
  if entry_count * entry_width != declared_length { return Err(..) }
  let entries = bytes[entries_start..entries_start + declared_length].chunks(entry_width.max(1)).take(entry_count)...
  ```
- 应该做什么 / 为什么错：宽为 0 时 `N × 0 = 0 = 声明长度` 过得去，`chunks` 在空切片上一块都不出，交回 0 条，而头里写的是 N 条；`IndexNodeHeader` / `PackedUnitHeader` 里不带条目数，调用方无从发现。码 2 这一支要 key 宽也是 0 才走得到（`entry_width < key_width` 先拦），码 3 没有这层。应在宽为 0 且数非 0 时报 `Structure`。另外 checker 的 `index_node_view` / 码 3 视图（crates/singlefs-checker/src/lib.rs:379-390、:531-544）是同一段写法（`chunks(width.max(1)).take(count)`），D13 已定项 5 要的两份独立解析器在这一格上一起漏，互比不会红。
- 失败场景：码 3 单元 记录数 = 5、记录宽 = 0、声明长度 = 0，头校验和与载荷 CRC 都按改后的字节封好：`parse_packed_unit` 返回 `records` 为空的 Ok，读者把一个自述 5 条记录的容器当成空容器。

### 8. `mapping_key_sort_key` 生产代码里没人调用，它那条测试证明的不是映射树真正用的序
- 位置：crates/singlefs-core/src/records.rs:214-228、:593-611；真正定序的在 crates/singlefs-core/src/code_two_tree.rs:37-39
- 严重度：低
- 置信度：已确认（`grep -rn mapping_key_sort_key crates` 只有 records.rs 两行：定义与测试）
- 代码做了什么：
  ```
  pub fn mapping_key_sort_key(key: &[u8]) -> (u8, u64, u64, u32, u64) {   // records.rs:216
  pub const CENTRAL_MAPPING: Self = Self(&[1, 8, 8, 4, 6]);                // code_two_tree.rs:39，映射树实际按它比
  ```
- 应该做什么 / 为什么错：同一个「映射 key 的全序」手写了两份，一份（records.rs）只被自己的测试用，另一份（code_two_tree.rs）才是映射树建树时用的。`mapping_keys_order_by_field_not_by_bytes` 测的是没人用的那份，给人的印象却是「映射树不按 memcmp 排」已经有测试盯着。两份在补零那 2 字节上语义还不同（records.rs 忽略，code_two_tree.rs 当 6 字节整数的高 2 字节比）。删掉 records.rs 那份、把测试挪到 `CodeTwoKeyFieldWidths::CENTRAL_MAPPING` 上。
- 失败场景：有人把 `CENTRAL_MAPPING` 改成 `&[1, 8, 8, 4]` 之类错的字段表，records.rs 的这条测试照样绿。

### 9. 同一段 10 字节 key 尾段有两个编码器，事务号越过 48 位时一个断言、一个静默截断
- 位置：crates/singlefs-core/src/records.rs:245-250 与 :190-199（经 crates/singlefs-core/src/bytes.rs:155-158）
- 严重度：低
- 置信度：已确认
- 代码做了什么：
  ```
  tail[4..].copy_from_slice(&write_order.transaction.to_le_bytes()[..6]);   // data_key_tail：高 16 位直接丢
  writer.put_six_byte_unsigned(write_order.transaction);                    // mapping_key_for_data：assert!(value < 1 << 48)
  ```
- 应该做什么 / 为什么错：journal 点名项的尾段（`data_key_tail`）与映射 key 的尾段（`mapping_key_for_data`）应当逐字节相同（records.rs:490-494 的测试就这么断言），但在 ≥ 2⁴⁸ 的输入上一个 panic、一个截断成别的值。`WriteOrder.transaction` 是 `u64`，transaction.rs:4062 / :5058 直接拿完整的事务号填，没有掩码。`data_key_tail` 应当同样断言 < 2⁴⁸，或者直接调 `put_six_byte_unsigned`。
- 失败场景：理论上限，今天的负载走不到；一旦走到，点名项凑出来的映射 key（`NamedUnit::mapping_key`）与树里的映射 key 不同，重放查不到该单元。

### 10. 死代码与只为压警告留下的引用
- 位置：crates/singlefs-format/src/lib.rs:164、:170；crates/singlefs-core/src/address.rs:88-91；crates/singlefs-core/src/make_filesystem.rs:13、:369
- 严重度：低
- 置信度：已确认（`grep -rn` 全 crates：`FIRST_TRANSACTION_ACCOUNTING_ROWS`、`FIRST_TRANSACTION_TREE_TABLE_ENTRIES` 各 1 行，只有定义；`COUNTER_LIMIT` 2 行，定义与自己的测试）
- 代码做了什么：
  ```
  pub const FIRST_TRANSACTION_ACCOUNTING_ROWS: u64 = 15;      // lib.rs:164，无引用，也没有 format-const 标记
  pub const FIRST_TRANSACTION_TREE_TABLE_ENTRIES: u64 = 7;    // lib.rs:170，同上
  pub const COUNTER_LIMIT: u64 = 1 << 48;                     // address.rs:90，只有自己的测试读；真正拦 48 位的是 bytes.rs:156 的另一处字面量
  let _ = JOURNAL_RECORD_BYTES;                               // make_filesystem.rs:369，import 了却不用，靠这一行压 unused 警告
  ```
- 应该做什么 / 为什么错：`code-discipline.md`「用不到的代码删掉」「编译器报的 never used 当错误处理」。make_filesystem.rs:369 这一行把编译器的 unused-import 信号关掉了，应删掉 import 与这一行。`COUNTER_LIMIT` 与 bytes.rs:156 的 `1u64 << 48` 是同一个数的两处定义。
- 失败场景：不影响行为；影响的是「一个数只许有一处定义」与未使用告警的判别力。

## 读过的文件与行数

13 个文件全部逐行读完，共 5353 行（`wc -l`）：

| 文件 | 行数 |
|---|---|
| crates/singlefs-format/src/lib.rs | 546 |
| crates/singlefs-core/src/address.rs | 124 |
| crates/singlefs-core/src/bytes.rs | 124 |
| crates/singlefs-core/src/checksum.rs | 120 |
| crates/singlefs-core/src/pointer.rs | 256 |
| crates/singlefs-core/src/unit.rs | 711 |
| crates/singlefs-core/src/records.rs | 612 |
| crates/singlefs-core/src/root_record.rs | 271 |
| crates/singlefs-core/src/root_ring.rs | 265 |
| crates/singlefs-core/src/journal.rs | 579 |
| crates/singlefs-core/src/system_configuration.rs | 833 |
| crates/singlefs-core/src/instance_table.rs | 473 |
| crates/singlefs-core/src/make_filesystem.rs | 439 |

为核对推理另读了片段（不在审阅范围、没逐行审）：code_two_tree.rs:20-110、recovery.rs:1201-1249 / :1825-1845 / :2722-2773、mount.rs:2830-2850、transaction.rs:270-290、block_device.rs 的 grep 结果、invariants.md I-2.1..I-2.5 行、D18 已定项 16 / 17、D19 已定项 7 / 11、D2 已定项 19、D23 已定项 17、checks-owed.md C307 行。

逐项核过、没问题的（避免重复查）：
- 编解码对称：码 1 / 码 2 / 码 3 的写者与读者字段顺序、宽度、偏移逐项一致（码 1：42/43/51/59/67/75/83/91/101/105；码 3：69/71/73/81/89/93/103/107；码 2：86 + 2k）；指针 50 + 28 + 10 / 8；根记录 138 / 457；journal 头 46 / 78 / 87 / 91 / 95 / 99 / 287 / 295 / 311；系统配置 102 / 147 / 155 / 219 / 301 / 317 / 333 / 361 / 362 / 379 / 425 / 433 / 469 / 481 / 489；树表条目 14 / 100 / 200；inode 记录 40 / 48 / 64 / 88 / 96 / 108 / 140。
- 校验和覆盖：头校验和先于载荷 CRC 算、罩住载荷 CRC 字段；码 2 载荷 CRC 偏移 = header_end − 10 = 76 + 2k；三类自证结构整槽覆盖、字段按 0 参与；与 checker（image.rs:254 按 physical_block_size 取根槽宽）一致。
- CRC-32C slicing-by-8：表生成与每轮 8 字节的表下标对应关系与标准实现一致，公开校验值 0xE3069283 的测试覆盖 1 个整块 + 1 个尾字节。
- 盘上字节的 panic 面：各 parse 入口先判长度，之后的切片与 `ByteReader` 读取都落在判过的范围内；`SlotNumber::to_device_offset` 的输入来自 6 字节槽号（< 2⁴⁸），乘 16384 < 2⁶²，不溢出；`first_file_byte` 的调用点单元序号来自内存里的下标。
- format 常量：各式子与字面量逐个手算过（457、311、56、67、65536、233、370、169、147、144、143、812、131 / 135 / 159 / 163 / 169）。

## 注释与代码不一致清单

| # | 位置 | 注释说 | 代码 / 条款实际 |
|---|---|---|---|
| 1 | crates/singlefs-format/src/lib.rs:229 | 「槽距 = max(4096, mkfs 时探测的 io_min)」 | D2 已定项 19 与 system_configuration.rs:90 / :101-109：4096 向上取整到 io_min 的整数倍；io_min = 3072 时前者给 4096、后者给 6144 |
| 2 | crates/singlefs-format/src/lib.rs:288、:417 | 测试「写成加法、不写减法（减法在变异下会编译期溢出）」 | 同一测试体在 :307、:314、:325、:330、:340、:350、:355 用了 `NODE_BYTES - …`、`DATA_UNIT_BYTES - …` |
| 3 | crates/singlefs-format/src/lib.rs:513-516 | 断言消息「槽宽不超过槽距下限，两槽永不重叠」 | 只在槽距 ≥ 4096 时成立，mkfs 不判槽距（第 2 条） |
| 4 | crates/singlefs-core/src/system_configuration.rs:118 | 「这 37 行里只有下面 5 个字段（`sizes` 里 4 个值，合起来 8 个值）」 | `SystemImmutableSizes` 有 5 个字段（:85-98，:79 自己写「那五个」），合起来 9 个值 |
| 5 | crates/singlefs-core/src/journal.rs:248 | 「magic、类型、整条校验和、fsid、载荷校验和四关」 | 列了五样，且漏了 :267 的记录长度判 |
| 6 | crates/singlefs-core/src/make_filesystem.rs:112 | 链指针记录「kind 1 + flags 0 + 指向下一片的指针 86」 | 第 2 字节是「有无下一片」（instance_table.rs:70-77、D18 已定项 11），不是 flags |
| 7 | crates/singlefs-core/src/make_filesystem.rs:259-261 | 根环三段与 journal 环「互不重叠」，所以不另加屏障 | 不重叠依赖未检查的几何（第 2 条） |
| 8 | crates/singlefs-core/src/records.rs:377-379 | C307「那一条还开着（码 1 给 27、码 2 / 码 3 给 25，补不补零没定）」 | 同文件 :187-188 写「中央映射 key 一律 27……末尾补零 2」，format lib.rs:89 写「2026-09-14 用户定案一宽」，code_two_tree.rs:37-38 按补零 27 定序；checks-owed.md C307 行未标还清。三处说法互相矛盾，没判哪边对 |
| 9 | crates/singlefs-core/src/pointer.rs:3、:138 | 「按设备身份升序，I-2.5」 | I-2.5 要严格升序，断言是 `<=`（第 6 条） |
