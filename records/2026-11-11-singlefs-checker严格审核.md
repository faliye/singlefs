# singlefs-checker 严格审核

对象：`crates/singlefs-checker` 当前工作区，按树里的现行源码读，不是相对某次提交的差异。本轮只新增这份记录，没有改产品实现、测试、欠账表、不变量正文或决策正文，也没有提交。

`records/2026-11-01-crates逐函数深度代码审核.md` 里这个包的 F-43、F-44、F-45、F-46、F-62，对照现行源码不成立，下面「对过、不单列」写了理由，不重复编号。`records/2026-10-24-singlefs-checker-审计复核.md` 的 C01、C02 在现行 `walk.rs` 里已经按那份记录改过，同样不重复编号。

## 结论

读码确认两条。K01 是挂载与池级 checker 对同一槽的拒绝归类不一致：在飞记录数上限装不进 4 字节时，core 在环长那一格整池拒，checker 的几何入口放行，随后 I-8.1 判红并把 I-7.13 报成不适用。池仍然不会被判成可挂。K02 是不变量正文里谱系记录宽还写着 16，已定项、格式常量和 checker 都是 24。没有把「这个包没有别的缺陷」当作结论。`walk.rs` 约 29226 行没有逐函数读完，库测试没有在本轮跑过。

| 编号 | 优先级 | 位置 | 读到的行为 | 成立条件 |
| --- | --- | --- | --- | --- |
| K01 | P2 | `image.rs` 的 `journal_ring_bytes_lie_in_the_supported_range`、`geometry_of`；`walk.rs` 的 `judge_pool_image_given_the_container_capacity_knobs`；core `recovery.rs` 的 `system_configuration_values_this_reader_accepts` | 环长除以 4096 再除以 F 得到的在飞上限超过 `u32` 能装下的范围时，core 报 `JournalRingBytesOutsideTheSupportedRange`，挂载在比三个字段等式更早的地方停下。checker 的环长判据没有这一格，几何收成可用，I-8.1 因为槽里那 4 字节对不上现算的 `u64` 而违例，并让其余不变量（含已经能判成立的 I-7.13）报不适用 | 自证过的系统配置槽里，环长至少 `2³² × 12288` 字节，单元区起始槽号等于这条环末尾的下一槽且落在 64 槽段边界上，其余几何字段仍是这一版收的值，整槽校验和重封。mkfs 在写盘之前就拒这个环长，所以要手改槽 |
| K02 | P3 | `invariants/01-块自描述.md` 的 I-1.7 那一行；`walk.rs` 的 `registered_record_width` | 这一行写「类型 5（谱系）是 16」。D18（块里携带什么信息）已定项 24 第 5 条、`LINEAGE_RECORD_BYTES`、`registered_record_width` 都是 24。按 24 判的坏镜像已经在测试里 | 读这一行的人会把登记宽当成 16。现行判定按 24，不会把宽 24 的谱系容器判坏 |

没有 P0 或 P1。K01 不把一份挂载会拒的池判成成立。它改的是崩溃位图上红的是哪一条：core 这一格归 I-7.13，checker 红 I-8.1、I-7.13 不适用。I-8.1 正文写明两条同时不成立时只红 I-7.13、I-8.1 报不适用；这一形走不到那一支，因为 checker 的 I-7.13 根本不看这 4 字节。

## 范围和这次实际做了什么

包只路径依赖 `singlefs-format`。`src/lib.rs` 是 `#![forbid(unsafe_code)]`。入口是 `walk::check_pool_image`。不变量清单在 `invariants/`，README 写池级 checker 判 114 条。

这次用 `wc -l` 点过的现行行数：

| 文件 | 行数 |
| --- | --- |
| `src/image.rs` | 2457 |
| `src/lib.rs` | 1365 |
| `src/position_addressed.rs` | 433 |
| `src/rebuild.rs` | 2188 |
| `src/walk.rs` | 29226 |
| `Cargo.toml` | 9 |
| `invariants/` 下 10 份 | 30、37、44、11、43、20、25、50、55、46 |

上面六份 `src/*.rs` 与 `Cargo.toml`，按路径排序后各自做 SHA-256，再对 `sha256sum` 那种「摘要、两个空格、路径」的清单做一次 SHA-256，结果是 `64b13bee9a7861920143e148556bd105a8430bbf2ceebc3dbe6062e24071f3fd`。这是读码当时的树，不是发布物摘要。`invariants/` 对照过下面点名的行，没有算进这份摘要。

本轮执行过的只有读文件、检索和上面的行数与摘要。没有跑 `cargo test`。

逐段读过的：`lib.rs` 的校验和、系统配置槽与根槽、单元头与打包视图、journal 记录头；`image.rs` 的 `geometry_of`、环长、`Judgements::into_report`、`value_refused_by_this_reader`、I-8.1 三个等式；`position_addressed.rs` 的扇出与跨度；`walk.rs` 的入口早退、打包容器 I-1.7、登记记录宽、journal 环按 `(计数器, 实例)` 留第一份、缓存命中后再判出生身份与树 ID。`rebuild.rs` 只核对了 `choose_current_versions` 对短头的 `assert` 是调用方契约。core 对照了 `recovery.rs` 的 `system_configuration_values_this_reader_accepts`、`system_configuration.rs` 的在飞上限与写槽偏移、`make_filesystem.rs` 对装不进 4 字节的拒绝。同表测试读的是 `system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 全文（五行场景，文件到第 220 行结束）。

没有逐行读完的：`walk.rs` 里 I-3.1、I-3.5 的显现规则、删除扫描、记账、extent 与分配走读的大部分；`rebuild.rs` 的认领规则；每个字段与 `layout/01-first-txn.md` 的逐字节对照。物理块大小与本盘设备号的偏移是对照 core 写槽核对的，不是对照布局表逐行核对的。

## K01：装不进 4 字节的在飞上限，checker 红 I-8.1，挂载红环长

core 把三件事并进 `JournalRingBytesOutsideTheSupportedRange`（`recovery.rs` 该成员的注释，以及 `system_configuration_values_this_reader_accepts` 里环长那一判）：在飞上限（环长 ÷ 4096 ÷ F，F 取格式常量）是 0；同一个数装不进系统配置偏移 345 的 4 字节；环末端越过同一槽的单元区起点。装不进 4 字节的判据是 `journal_in_flight_record_limit_fits_its_four_byte_field`。`system_configuration.rs` 写明：环长 ≥ `2³² × 12288` 字节时装不下，mkfs 在写之前拒，读者择系统配置时也按环长越界拒；`to_slot` 写这 4 字节的 expect 靠的就是这两道。三个字段等式 `journal_ring_geometry_fields_recompute_from_the_ring_length` 排在这一判之后，环长已经越界时走不到。

checker 的 `journal_ring_bytes_lie_in_the_supported_range` 只做两件：用格式常量 F 算在飞上限 ≥ 1，以及环末端不超过单元区起点。注释也只写这两件。`geometry_of` 在这一判通过、单元区起点等于环末下一槽、并且起点在 `CLUSTER_SEGMENT_SLOTS`（64）边界上时，把槽收成 `PoolGeometry`。槽里偏移 345 的在飞上限用 `read_u32` 收成 `u64`，所以几何里这份声明值至多是 `u32::MAX`。

最小的这样一环：环长 `3 << 44` 字节（等于 `2³² × 12288`）。在飞上限现算是 `2³²`，`u32::try_from` 失败。环起点取第一版常量 1024 时，环长能被 16384 整除，槽数是 `3 << 30`，单元区起始槽号是 `1024 + (3 << 30)`。1024 与 `3 << 30` 都能被 64 整除，起点落在段边界上，环末端与这个起点的字节偏移相等，环长那两判都过。

`judge_pool_image_given_the_container_capacity_knobs` 先判 I-7.13。这一槽几何收成可用，I-7.13 不违例。随后 I-8.1 在一份单独的 `Judgements` 上比三个等式。F 仍是 3 时，第一格过；第二格要求现算的在飞上限等于槽里那 4 字节。现算是 `2³²`，槽里至多 `u32::MAX`，这一格必不等。违例次数大于 0 时，函数把这份单独判定上的每条已实现不变量都标成不适用并返回。`not_applicable` 用 `or_insert`，不擦掉已经记下的 I-8.1 违例。`into_report` 先看违例，所以报告是 I-8.1 违例，I-7.13 不适用。早先在 `root_ring_judgements` 上记下的 I-7.13 成立随那份对象丢掉。

I-7.13 正文写「判的字段与 core 收同一张表」，环长一格列的是「÷ 4096 ÷ F ≥ 1 且环末端不越过单元区起始槽号」，与 checker 的两个函数一致，没有装进 4 字节这一句。I-8.1 正文写：三格任一不等时整池不作保；I-7.13 与 I-8.1 同时不成立时只红 I-7.13，I-8.1 报不适用。这一形在 core 上两条理由都在（环长越界，并且 4 字节对不上现算值），checker 只看见第二条，所以「同时不成立走 I-7.13」没有发生。

同表测试 `system_configuration_fields_core_and_checker_refuse_the_same_table.rs` 覆盖单元区起点、journal 环起点、根环起点 65、段边界、根环起点 0。没有覆盖装不进 4 字节的环长。`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` 只钉 mkfs 对 48 TiB 环的 `JournalInFlightRecordLimitWiderThanItsFourByteField`，两块盘一个扇区都没写。

触发要同时满足：槽自证过（magic 与整槽校验和重封过），环长至少 `2³² × 12288` 字节，单元区起始槽号改成这条环的下一槽并仍在 64 槽边界上，R、S、P、chunk、两个起点、槽距、根槽宽仍是这一版收的值。只把环长改大、单元区起点留着默认值时，环末端越过单元区起点，checker 与 core 都报环长越界，归 I-7.13，K01 不出现。本轮没有造这只镜像跑 checker。

## K02：I-1.7 正文把谱系记录宽写成 16

`invariants/01-块自描述.md` 的 I-1.7 写：记录宽等于登记表，类型 2 是 140，类型 1（墓碑）是 40，类型 5（谱系）是 16，出处写 D18（块里携带什么信息）已定项 24。同一条已定项的第 5 条写谱系记录是 T_rb 8 + T_old 8 + H_cur 8，共 24 字节。已定项 11 的字段表把类型 5 的宽写成 24，并据此算一个容器 1359 条。`crates/singlefs-format/src/lib.rs` 的 `LINEAGE_RECORD_BYTES` 是 24。`walk.rs` 的 `registered_record_width` 对 `PACKED_TYPE_LINEAGE` 返回 24，注释点已定项 24 第 5 条。

同一行 I-1.7 的不等式「记录数 × 记录宽 ≤ 声明长度且声明长度 ≤ 32768 − 136」与 `judge_packed_container` 一致。登记宽不等才判损坏。状态列写类型 1 与类型 5 的记录宽在 2026-10-05 补了坏镜像。`checker_known_bad_images.rs` 里类型 5 的注释写「登记 24」，改的是头里的 24 → 15 或 24 → 23。按 24 判的代码不会把合法谱系容器判坏；正文那一句「是 16」与已定项、格式常量、判定、坏镜像注释都不一致。

## 对过、不单列成问题

**根环起点 0：core 先报槽距，checker 先报起点。** `geometry_of` 在槽距之前判两个起点等于第一版常量，注释写起点不对时报起点。core 先用同一槽自述的根环起点做槽距上界，起点落在槽 1 里时报 `FixedStructureSlotSpacingOutsideTheFormatRange`，然后才比起点等不等常量。两边都进 I-7.13。同表测试的 E 格把根环起点改成 0，期望就是这个分叉：core 的成员是槽距，checker 说明里点名 `RootRingBaseSlotNotTheFirstVersionConstant`，并且只红 I-7.13。这是已有测试钉住的次序，不另编号。core 在 `system_configuration_values_this_reader_accepts` 上的注释写「与 checker 的 I-7.13 同一格」；对起点 0 这一形，同一格指的是不变量编号，拒绝成员并不相同，E 格已经把成员分开写了。

**2026-11-01 的 F-45、F-46。** `position_addressed.rs` 的扇出在层级 ≥ 8 时饱和。任一装得进 `u64` 字节数的设备，槽数不超过 `2⁵⁰`，根层级到不了 8。`sum::<u64>` 要在设备块数大到不现实时才会溢出。现行源码上看不到一份盘镜像能走进这两条。

**2026-11-01 的 F-43、F-44。** `rebuild.rs` 的 `choose_current_versions` 对短于头尾的缓冲 `assert`。注释把这写成调用方契约：扫描路径声称只交头校验和已核过的单元。没有从 `check_pool_image` 走到一份会让这里 panic 的盘。分配体积是开销，不是判错。

**2026-11-01 的 F-62。** `check_pool_image` 在 I-1.4 比过 fsid、I-7.15 比过设备数之后，几何取 `chosen[0]`。core 的 `choose_system_configuration` 同样留下第一块挂得上的盘的整份配置，后面的盘只比 fsid 与设备数。I-7.15 的正文射程就是设备数。这是挂载与 checker 共用的读法，不单记在 checker 上。

**2026-10-24 的 C01。** `read_index_node` 在 `visited_units` 命中时调用 `note_a_unit_an_earlier_version_walked_referenced_again`，再判 I-1.6、I-1.2、I-1.3、I-1.1、I-1.12。打包容器与数据单元的命中走同一钩子。早退不再跳过出生身份与树 ID。

**2026-10-24 的 C02。** `scanned_journal_ring_of_device` 的键是 `(计数器, 实例)`，`or_insert` 留先扫到的那一份。

**journal 点名项条数相乘。** `check_journal_record` 先把条数从 `u32` 转成 `usize`，再拿头长加条数乘 56 去比 4096。64 位上 `(u32::MAX) * 56` 不回绕，超长条数是 `DeclaredLengthMismatch`。flags 非 0 的重封循环在长度已经比过之后才跑。

**I-7.13 或 I-8.1 早退不会擦掉刚记下的违例。** `into_report` 的次序是违例、部分没判、射程里判不了、已评估则成立、否则不适用。`not_applicable` 不覆盖已有违例。

**系统配置与根槽的校验和覆盖。** 系统配置校验和盖 4096 字节整槽。根槽校验和盖按 `physical_block_size` 读入的那一缓冲。偏移与 core 写槽一致的有：fsid 102、校验和 155、本盘设备号 139、设备数 143、世代 147、加密类型 219、节点大小 301、物理块大小 317、journal 在飞上限 345、槽尾 469、根校验和 138。

## 明确不声称的事

- 没有声称 `walk.rs` 与 `rebuild.rs` 里没读完的部分没有别的缺陷。
- 没有跑库测试，没有造 K01 那只手改镜像。K01 的依据是两边函数的判据、早退用的那份 `Judgements`，以及 `2³² × 12288` 这条环长在单元区起点跟着改、并且落在 64 槽边界上时能通过 checker 的环长两判。
- 没有改 I-1.7 或 I-7.13 的正文，也没有新开 C 号。代码注释里仍点名的 C307、C461、C476、C748、C896、C897 本轮没有拿去重编号。
