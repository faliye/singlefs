# 池级 checker 在途改动代码轮第一轮：正推腿报告

日期：2026-11-16。腿：three-way-forward。草稿目录：`/home/fy5090/.local/share/singlefs/work/checker-src-backfill-code-r1-forward`。

## 0. 开工核对与方法

- `sha256sum -c research/prompts/checker-src-backfill-code-r1-snapshot/crates-sha256.txt`：五份全 OK。`kb-sha256.txt` 的十三份（D23、01-first-txn、invariants 01 到 09 与 README）也全 OK。`git diff HEAD -- <五份>` 的 sha256 与 `changes.patch` 相同（1e19a827b215aff7b2546ccaae652d18f335c2f443cfb89d02d91db33639d515），patch 是现行树对 HEAD 的完整差。
- 条款现读：D23 已定项 18、19 整节，`01-first-txn.md` 一整节，I-7.13、I-8.1、I-3.5、I-9.16、I-1.8 各自整行，D8 已定项 9 整节。
- 改名核对用我自己的脚本（草稿目录 `rename_check.py`、`rename_count.py`）：把每一段连续的 `-`/`+` 行切成词（标识符、数、字符串、`//` 注释行、标点，空白不计），两侧词序列逐词比，只许标识符对标识符。rustfmt 的折行、合行因此不算差。
- 没写模型，没有 `rerun.sh`。要量的都写在第 8 节。
- **条款与代码同批在途**：`git diff HEAD --stat -- crates/singlefs-checker/invariants` 有四份改动（01、02、04、05）。I-7.13 里「这个商装得进……4 字节一格」那一句、I-3.5 里「走到没有条款的形状」两段，都是这一批的 `+` 行，不是先于代码的条款。这一点在下面逐件处标出，这几处「对得上」只说明代码与同批文字一致，独立的依据另列。

## 1. 五件改动逐件

### 1.1 环长判据加「在飞上限装得进 4 字节」：对得上

条款原文（整行读过）：

- 「在飞记录数上限 = 环槽数 ÷ F」（`.claude/kb/decisions/23-journal的角色与格式.md:525`）；同一行「记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`」，环槽就是 4096 字节，环槽数 = 环长 ÷ 4096。
- 「| 几何 | journal 在飞记录数上限 | 4 | 65536（= 196608 ÷ F） | D23（journal 的角色与格式） 已定项 18 |」（`.claude/kb/layout/01-first-txn.md:136`），宽 4 字节；偏移见「journal 在飞记录数上限 345」（`.claude/kb/layout/01-first-txn.md:160`）。HEAD 里这两行就在，不是这一批改的。
- 「journal 在飞记录数上限 = ⌊环长 ÷ 4096 ÷ F⌋」（`crates/singlefs-checker/invariants/03-journal.md:8`）。
- 「journal 环长 ÷ 4096 ÷ F（`JOURNAL_SAFETY_FACTOR`）≥ 1、这个商装得进系统配置里它那 4 字节一格（偏移 345，装不进时不走到 I-8.1（环几何够大） 的等式）且环末端不越过单元区起始槽号」（`crates/singlefs-checker/invariants/02-根环.md:20`，同批在途）。
- 「I-7.13（系统配置池级字段在读者收的范围里） 与这一条同时不成立时走 I-7.13（系统配置池级字段在读者收的范围里） 那一支」（`crates/singlefs-checker/invariants/03-journal.md:8`）：两条同时不成立时只红 I-7.13。

代码逐句对：

- 商：「let in_flight_record_limit = journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR;」（`crates/singlefs-checker/src/image.rs:970`），常量「pub const JOURNAL_RECORD_BYTES: u64 = 4096;」（`crates/singlefs-format/src/lib.rs:241`）、「pub const JOURNAL_SAFETY_FACTOR: u64 = 3;」（`crates/singlefs-format/src/lib.rs:273`）。两次整除向下取整与 ⌊环长 ÷ 4096 ÷ F⌋ 同值。
- 「in_flight_record_limit >= 1」（`crates/singlefs-checker/src/image.rs:974`）加「&& u32::try_from(in_flight_record_limit).is_ok()」（`crates/singlefs-checker/src/image.rs:975`）加环末端不越过单元区起点（crates/singlefs-checker/src/image.rs:976 起）：三项与 I-7.13 那一句逐项一致。
- 槽里那一格读成 u32：「declared_in_flight_record_limit: u64::from(read_u32(slot, 345)),」（`crates/singlefs-checker/src/image.rs:917`）。商 ≥ 2³² 时，槽里最大只能写 2³²−1，I-8.1 第二格的等式「必不成立」（注释这句成立），所以不先拒就会红在 I-8.1；先拒归 I-7.13 与 I-8.1 那条「同时不成立走 I-7.13」一致。
- 报的成员：`geometry_of` 里 `journal_ring_bytes_lie_in_the_supported_range` 为假报 `JournalRingBytesOutsideTheSupportedRange`（crates/singlefs-checker/src/image.rs:895-900），与 core 同名成员。
- core 同判：「|| !journal_in_flight_record_limit_fits_its_four_byte_field(sizes.journal_ring_bytes)」（`crates/singlefs-core/src/recovery.rs:1478`），函数体 `u32::try_from(journal_in_flight_record_limit(journal_ring_bytes)).is_ok()`（`crates/singlefs-core/src/system_configuration.rs:125-127`）。两侧收同一张表，一格不差；核的另一侧在 core 对 `==0`、4 字节、环末端三项用 `||` 串，checker 用 `&&` 串取反，等价。
- 边界：新测试的取样点 `3 << 44`：3·2⁴⁴ ÷ 4096 ÷ 3 = 2³²，「u64::from(u32::MAX) + 1,」（`crates/singlefs-checker/src/image.rs:2473`）；减 12288 后是 2³²−1。单元区起点「JOURNAL_RING_START_SLOT + (3 << 30),」（`crates/singlefs-checker/src/image.rs:2484`）：⌈3·2⁴⁴ ÷ 16384⌉ = 3·2³⁰，两个取样点上环末端都不越过它，只有 4 字节一格分得开。读代码核过，没跑。

两处没对到独立条款（不改判定，写明）：

1. 「装不进就拒」这句是 I-7.13 同批在途的文字。独立的依据是 `.claude/kb/layout/01-first-txn.md:136` 的宽 4 加 D23 已定项 18 的取值式：值写不进它的字段，I-8.1 等式必不成立，两侧都得有一处先拒。
2. 代码注释写「D23（journal 的角色与格式） 已定项 18」，已定项 18 的措辞是「环槽数 ÷ F」、没有 ⌊环长 ÷ 4096 ÷ F⌋ 这个写法；后者在 I-8.1。注释指到的两处合起来才推得出式子，不矛盾。

推翻条件：在 `3 << 44` 与 `(3 << 44) − 12288` 两个环长上，core 与池级 checker 的结局不同（第 8 节命令 C3 的第六格）；或存在环长 ≥ 3·2⁴⁴ 而槽里在飞上限字段「等于」4 字节截断值、I-8.1 却报成立（等式右边是 u64 不截断，推不出会有）。

### 1.2 CRC 按内容复用：没有条款可对；与现算同值的论证在第 3 节，推得出

`lib.rs` 的新注释没点名任何编号。`crates/singlefs-checker/src/lib.rs` 头注释「CRC-32C、解析、校验各写一份」只管不引 core，不管同值。能对的只有「改前后返回值不变」这一条，第 3 节逐步推。结论：对得上（对的是自己的同值性质，不是条款）。

### 1.3 I-3.5 加「走到没有条款的形状」一路

条款原文（整行读过）：

- 「R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状：树表里种类没登记的条目算，用户 2026-11-09 定）时整对报不适用」（`crates/singlefs-checker/invariants/05-空间记账.md:11`）。用户定的只有「种类没登记」。
- 同一行状态格（同批在途）：「种类没登记的不管根指针；livelist、稀疏旁表根指针非全零同走这一支，走读同样不往下走、引用同样没收全」；「参与的那一对同样报部分没判，共享那片树表的版本同记，不进走读失败（I-7.2、I-4.8、I-7.4 不跟着红，I-9.16 ④ 判红」。
- 登记表：「| 0 | 无（树表条目不用它） | — |」（`.claude/kb/decisions/08-核心索引结构.md:322`）到「| 15 | 快照 extent 根 | 已定项 8 |」（`.claude/kb/decisions/08-核心索引结构.md:337`），码 0..15；「| 5 | 中央映射树 | D19（块指针的结构与宽度预算） 已定项 5 |」（`.claude/kb/decisions/08-核心索引结构.md:327`）。
- 第一版 livelist、稀疏旁表根指针为零：「八棵根指针为零」（`.claude/kb/layout/01-first-txn.md:80`）。

分格对代码（`crates/singlefs-checker/src/walk.rs`）：

| 格 | 代码 | 对条款 |
|---|---|---|
| 码 14、15 不算 | 「TREE_KIND_SNAPSHOT_EXTENT_ROOT => false,」（crates/singlefs-checker/src/walk.rs:823，臂首是码 14 与码 15 两个常量用或连起来）；走读先把这两种交 `walk_the_tree_of_a_snapshot_entry`（crates/singlefs-checker/src/walk.rs:3353-3354），不到 key 形态那一步 | 对得上（走读照活头树往下走，不是没有条款的形状） |
| 码 6、7 根指针非全零才算 | 「=> !root_pointer_is_all_zero,」（crates/singlefs-checker/src/walk.rs:824，臂首是 TREE_KIND_LIVELIST 与 TREE_KIND_SPARSE_SIDE_TABLE 用或连起来）；走读对没有 key 形态的种类直接返回：「let Some(schema) = key_schema_for_tree_kind(kind) else {」（crates/singlefs-checker/src/walk.rs:3359） | 与走读一致（根非零而走读不往下走，引用数不全）；条款里只有同批在途的状态格这样写，用户 2026-11-09 定的字面只到「种类没登记」。推不出独立依据，写明 |
| 其余种类看有没有 key 形态 | 「other_kind => key_schema_for_tree_kind(other_kind).is_none(),」（crates/singlefs-checker/src/walk.rs:825） | 码 0 与 ≥16 没登记，`key_schema_for_tree_kind` 对它们返回 `None`（`crates/singlefs-checker/src/lib.rs:887`），算；码 1..4、8..13 有 key 形态，不算；码 5 登记了且有 key 形态，不算（条款没把它列为没登记）。8..13 与测试里「走读走得下去」的清单一致，5 不在测试清单里 |
| 根指针全零的没登记种类也算 | 臂不读 `root_pointer_is_all_zero` | 对得上（「种类没登记的不管根指针」） |

流程对条款：

- 记：「self.note_the_first_tree_table_entry_without_a_clause(&entries_of_the_walk);」（crates/singlefs-checker/src/walk.rs:2939，先走这片树表的那一版）与「self.note_the_first_tree_table_entry_without_a_clause(&entries);」（crates/singlefs-checker/src/walk.rs:3776，共享的版本）：两处都经同一个函数，对得上「共享那片树表的版本同记」。函数体 `find_map` 取第一条，不 `push` 进 `walk_failures`（crates/singlefs-checker/src/walk.rs:3748 起整段读过），对得上「不进走读失败」。
- 清：「self.kind_of_a_tree_table_entry_without_a_clause_in_this_version = None;」（crates/singlefs-checker/src/walk.rs:2183，在 `begin_walking_a_version` 里），与 `user_visible_tree_roots_of_this_version.clear()` 同处同步；每一版从没有记起。
- 带：走完一版记进 `VersionPlacementsForTheAppearanceRule`（crates/singlefs-checker/src/walk.rs:2221-2222），再转 `VersionReferencesForTheAppearanceRule`（crates/singlefs-checker/src/walk.rs:2270-2271）。树表读不到的版本，字段保持 `None`。
- 判：相邻一对里「.chain(std::iter::once(version))」（crates/singlefs-checker/src/walk.rs:1092，前面接 previous_versions.iter()）任一版有种类就把这一对记进「不判」并 `continue`（crates/singlefs-checker/src/walk.rs:1090-1100），与旧的「走读记过失败」那一支（crates/singlefs-checker/src/walk.rs:1083）同形同归处；R⁻ 取并、R 逐条的口径与条款「R⁻ 侧引用集合取并，R 侧逐条各判一次」一致。条款写「整对报不适用」，两个分支都进同一个 `unjudged…` 向量，最终是部分没判，与状态格「报部分没判」一致。
- 短条目：宽不够的条目 `tree_table_entry_of_the_walk` 返回 `None`，`filter_map` 丢掉，它们本来就由 `walk_tree_table_entry` 记走读失败，不会既不判又不报。

**一处注释与条款同样宽的话，对不上**：注释（crates/singlefs-checker/src/walk.rs:3747）与状态格都写「这一条由 I-9.16 ④ 判红」。读 I-9.16：「③ 每一种种类码（登记过的与没登记的都算）至多一条」「④ 树表非空时进树表的十二种（② 那一串）各恰好一条」（`crates/singlefs-checker/invariants/09-inode树结构.md:28`）。代码 `judge_tree_table_entry_kind_counts`（crates/singlefs-checker/src/walk.rs:6102 起）与 `judge_tree_table_entries_ordering`（crates/singlefs-checker/src/walk.rs:6033 起）读过：

- 没登记的条目**顶掉**十二种之一（测试里把 `deleted_inodes` 那一条的种类改成没登记的）：④ 缺一种，红。测试 `an_intermediate_version_without_a_deleted_inodes_entry_leaves_the_deletion_queue_invariants_not_judged_while_the_newest_root_is_clean` 钉的就是这一形，断言 `violated_invariants == ["I-9.16"]`。
- 没登记的条目在十二种之外**多出一条**（十二种各一条加一条码 16）：③ 码 16 只一条，成立；④ 十二种都在，成立；② 只比十二种的次序，码 16 被跳过。三道读下来都不红。
- 码 6、7 根指针非全零：种类仍是十二种之一、各一条，I-9.16 不红。

所以这两形下 I-9.16 不红（别的不变量红不红我没逐条查，I-9.27 头 ID 那一判尤其没读），I-3.5 那一对报部分没判。是否放行由条款定（条款没说这两形是坏盘），但「由 I-9.16 ④ 判红」对这两形不真。读码推的，没跑。要主 agent 现查的话，照 `crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里「十三条」那份测试的写法（复制一条已有条目追加成第十三条）追加一条种类码 16 的条目：那份测试复制的是稀疏旁表条目，所以走 ③ 红，不是这一形。

推翻条件：任何一个树表带码 ≥16（或码 0）的条目、该版其余十二种齐全的镜像上，池级 checker 报了 I-9.16 红，则上面「不红」的读法被推翻。

### 1.4 扫描重建三个错误成员改名：对得上

三个成员去掉名字里的 `I18`，改成条款的名字：I-1.8 的标题是「归并后版本全序」，① 是「同一组的成员载荷相同」、② 是「同 key 的各组两两全序键不等」（`crates/singlefs-checker/invariants/01-块自描述.md:14`）。

- `DataUnitVersionsTiedOnTheTotalOrderKeyOfI18` → `DataUnitVersionsTiedOnTheMergedVersionTotalOrderKey`，`PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18` → `PackedRecordVersionsTiedOnTheMergedVersionTotalOrderKey`：② 那一格（同键两组），名字对。
- `CopiesOfOneVersionCarryDifferentPayloadsUnderI18` → `CopiesOfOneVersionCarryDifferentPayloadsUnderTheMergedVersionTotalOrder`：① 那一格（载荷不同），名字对。
- 三个成员在 `rebuild.rs` 的定义、`reason()` 三臂、`latest_of_the_ordered_versions` 两臂、`choose_current_versions` 一处，加三处测试断言，词序列核过：只有这三个标识符变（各 4 处出现），`reason()` 的字符串 `…I-1.8 ② 不成立…` 与 `…I-1.8 ① 判损坏…` 没动。变异行 4（第 4 节）把 `PackedRecord` 臂换成 `Data` 那个成员，测试断言 `PackedRecord` 那个成员，改回去才绿，说明两个成员各自被一条断言钉着。

推翻条件：`grep -rnE 'TotalOrderKeyOfI18|TotalOrderOfI18|UnderI18' crates scripts`（`.rs`、`.tsv`、`.py`、`.wgsl`、`.sh`）有命中。我读过的结果零命中；不在这个射程里的一处旧名见第 7 节。

### 1.5 改名批：见第 2 节

## 2. 改名批逐块核对

- **块数**：`changes.patch` 五份共 106 个 `@@` 块（量的：`grep -c '^@@'` 对 `git diff HEAD` 得 106；image 11、lib 6、position_addressed 2、rebuild 27、walk 60）。按连续的增删行再切细是 120 段。
- **纯改名**：90 段词序列逐词比，只差标识符（或 `///` 变 `//` 一处），没有第二种差别；其中 89 段在 81 个只含改名的块里（image 8、lib 3、position_addressed 2、rebuild 27、walk 41），1 段夹在真改动块里（lib 的 `-1227` 块里 `system_configuration_slot_whose_node_size_is_not_the_format_constant_is_refused_by_name` 的改名）；余下 30 段不是纯改名，全在下面那 25 个块里（`hunk_count.py` 量的）。
- **不是纯改名**：25 个块（image 3：`-955`、`-969`、`-2454`；lib 3：`-1`、`-54`、`-1227`；walk 19：`-715`、`-778`、`-802`、`-809`、`-823`、`-849`、`-1066`、`-1849`、`-2145`、`-2182`、`-2229`、`-2896`、`-3313`、`-3700`、`-3714`、`-28870`、`-28879`、`-28893`、`-28920`）。这 25 块我逐块读过，都是第 1 节那几件的真改动或它们的测试，没有哪一处是「看上去在改名、实际改了行为」。lib 的 `-1` 块只改头部 `//!` 注释（删「步 1 能判的」「步 3 / 步 4 / 步 5 加的」两处进度措辞，列的判项没变）。
- **标识符改名共 76 个**（量的，`old_names.txt`）：image 8、lib 4、position_addressed 2、rebuild 23（三个成员加 20 个测试）、walk 39。walk 的 39 个里：34 个 `a_` 测试函数；3 个局部变量（`a_copy_of_the_class_the_entry_type_names_verifies`→`some_…`、`a_later_root_is_incomplete`→`some_…`、`a_record_sits_before_the_first_member`→`some_…`）；1 个**生产函数** `a_truncation_intent_names_the_record`→`truncation_intent_names_the_record`（定义加两处调用，crates/singlefs-checker/src/walk.rs:15081、15137、15211）；1 个不是去前缀的测试改名 `quarantined_unit_tail_over_a_record_released_between_t_and_theta_high_reddens_the_unit_class_reading` 把 `t` 展开成 `the_root_recorded_reclaim_threshold`（crates/singlefs-checker/src/walk.rs:27327）。
- **碰撞与残留**：76 个新名在 HEAD 的同一份文件里一次都没出现（`git show HEAD:<文件> | grep -cw` 逐个 0），76 个旧名在现行同一份文件里一次都没剩。局部变量的三处都在同一作用域里一起改了，没有影子。
- **`///`→`//`**：「// 待读的一个节点：上段 (层级, 序号)，或某个 inode 下段 (层级, 序号)；根那一个层级取它自己头里的。」（`crates/singlefs-checker/src/walk.rs:7273`），后面紧跟函数体里的局部 `enum Pending`；局部项上的文档注释改普通注释不改代码。
- **其它文件引到旧测试名**：`crates/mutations.tsv` 第六段每一行的测试名我都按 `fn <名>` 在 `crates/` 下找过（量的：8003 行，找不到的 50 行，全在 `prune-server/src/` 下、不在 `crates/` 里，与这一批无关；五份检查器文件指到的行一条都不缺）。
- **不是纯改名的两点要主 agent 知道**：① 生产函数 `truncation_intent_names_the_record` 被改名（派发正文第 5 项只说「测试函数与局部变量」）；② `quarantined_…` 那一条是展开缩写，不是去 `a_`。两处都只改了名字。

推翻条件：任一新名与同作用域里别的名冲突（编译会报），或任一旧测试名仍被别处按名字引用而没改（`rg -F` 旧名，除 `crash_judge_checker_scan.wgsl` 里同形的 GPU 内核局部名外应零命中；那个 `.wgsl` 里的 `a_later_root_is_incomplete`、`a_truncation_intent_names_the_record` 是内核自己的名字，不是对 Rust 名字的引用）。

## 3. 按内容复用与现算同值的论证（`crc32_castagnoli_table`）

记 F(b) 为不带复用的查表 CRC：初值 `!0`，逐字节 `remainder = table[(remainder & 0xFF) ^ byte] ^ (remainder >> 8)`，末了取反（`crates/singlefs-checker/src/lib.rs:172-182`）。F 只读 `b`、表是 `OnceLock` 里一次算好的常量，F 是纯函数。要证：复用版 G(b) = F(b) 对每个输入 b。

**不变量 I**：线程记录里每个候选 c 满足 `c.checksum == F(c.content)`。

- 建立：`candidates` 只在一处被加入元素，「content: bytes.into(),」（`crates/singlefs-checker/src/lib.rs:202`）与同一次调用里「let checksum = !remainder;」（`crates/singlefs-checker/src/lib.rs:181`）算的是同一个 `bytes`（共享借用 `&[u8]`，两句之间没有写；`#![forbid(unsafe_code)]` 在 `crates/singlefs-checker/src/lib.rs:7`）。`content` 是 `bytes` 的拷贝，`checksum` 是对 `bytes` 现算的值，所以 `checksum == F(content)`。
- 保持：`candidates_by_fingerprint` 与 `remembered_content_bytes` 的全部写点是 `clear()`（crates/singlefs-checker/src/lib.rs:188）、`entry(…)` 取向量并 `push`（crates/singlefs-checker/src/lib.rs:192-201）、`remove(0)`（crates/singlefs-checker/src/lib.rs:197）、总数的重算（crates/singlefs-checker/src/lib.rs:206）；没有任何地方改一个已记下的 `content` 或 `checksum`（读的点只有 crates/singlefs-checker/src/lib.rs:146、149、157）。删除不会破坏 I。

**第一步，指纹相同再逐字节比**：命中路径是「.position(|candidate| *candidate.content == *bytes);」（`crates/singlefs-checker/src/lib.rs:149`）找到之后「return candidates[position].checksum;」（`crates/singlefs-checker/src/lib.rs:157`）。切片相等包括长度与每个字节，所以 `content == bytes` ⇒ `F(content) == F(bytes)`，由 I 返回的就是 F(bytes)。这一步**不依赖指纹**：指纹只决定去哪个桶里找（`HashMap` 的键相等用 `ContentFingerprint` 的 `Eq`，长度加 16 个字全等），取样位置选得再差、哈希撞了也只是多比几段或比不中而回到现算，不会取错值。`(bytes.len() - WORD_BYTES) * word_index / 15` 不会越界：`len > 8` 才走这个分支，起点 ≤ `len − 8`，终点 `min(len, 起点+8)`（crates/singlefs-checker/src/lib.rs:133-138），`len ≤ 8` 与 `len = 0` 都是起点 0。64 位下 `len × 15` 不溢出。推得出，没有缺口。

**第二步，清空**：条件是「if remembered.remembered_content_bytes + bytes.len() > remembered_content_bytes_limit {」（`crates/singlefs-checker/src/lib.rs:187`），体内 `clear()` 并把总数置 0。这一步发生在现算之后、记下之前，本次返回的是刚算的 `checksum`；清空之后表是空的，I 对空表平凡成立；之后的调用要么命中新记下的、要么现算。值不受影响。推得出。前面还有一道「if bytes.len() > remembered_content_bytes_limit {」（crates/singlefs-checker/src/lib.rs:184）：内容比整个上限还大就不记、直接返回现算值，也不影响值。

**第三步，候选上限**：「if candidates.len() >= REMEMBERED_CANDIDATES_PER_FINGERPRINT_LIMIT」（`crates/singlefs-checker/src/lib.rs:196`）时 `remove(0)` 挤掉最早的一个再 `push`。被挤掉的只是少了一个可命中的候选，查找结果只会是「命中某个与输入逐字节相同的候选」或「没命中、现算」，两样返回值都是 F(b)。推得出。

**其余会 panic 或改值的点，逐个看过**：`candidates[position]` 的下标来自对同一个向量的 `.position`（crates/singlefs-checker/src/lib.rs:149、157），在界内；总数更新 `remembered_content_bytes - evicted_content_bytes + bytes.len()`（crates/singlefs-checker/src/lib.rs:206）先减后加，被减的 `evicted` 是总数的一个加数，不会下溢，并且 `bytes.len() ≤ 上限` 已由 crates/singlefs-checker/src/lib.rs:184 保证，加法不溢出；`RefCell::borrow_mut`（crates/singlefs-checker/src/lib.rs:145）闭包里没有再调 `crc32_castagnoli_table`，不会重入；`HashMap` 用默认的随机哈希，只用 `get`、`entry`、`clear`，没有依赖迭代次序（测试里 `values()` 求和与取最大也与次序无关）。

**调用次序与线程**：返回值只由本次的 `bytes` 决定（上面三步），所以判定不依赖调用次序、也不依赖哪条线程调用；线程局部记录只影响耗时与内存。

**推不出的两处**：

1. **耗时与内存的上界没证**。注释说 8 MiB 乘线程数「压在几百 MiB」（`crates/singlefs-checker/src/lib.rs:60`），按登记的每台默认线程数 31（`.claude/heavy-thread-default:7`）是 248 MiB，这只算了内容字节。每条记下的内容另带约一个 `ContentFingerprint`（136 字节）、一个 `Vec`、一个 `Box` 头；内容几十到一百多字节时（单元头的 `unit[..107]` 那类调用）开销与内容同量级，极小内容（几字节）开销是内容的几十倍。调用点里有没有大量极小而互异的输入我没量；这是内存上界的问题，不是值的问题，交攻方腿与主 agent 量。
2. **线程退出时的访问**：`thread_local!` 里带析构的值在线程退出析构之后再被 `with` 访问会 panic。本箱没有 `impl Drop`（`grep -n 'impl Drop'` 在 `crates/singlefs-checker/src/*.rs` 零命中），没有别的线程局部析构去调 CRC；箱外调用者我没查。

**`#[cfg(test)]` 会不会漏进非测试构建**：限额覆盖、两个计数是 `#[cfg(test)] thread_local!`（crates/singlefs-checker/src/lib.rs:99）；用它们的三处语句各自带 `#[cfg(test)]`（crates/singlefs-checker/src/lib.rs:126、150、182 起），非测试构建里限额取常量「let remembered_content_bytes_limit = REMEMBERED_CONTENT_BYTES_LIMIT_PER_THREAD;」（`crates/singlefs-checker/src/lib.rs:125`）。`singlefs-verification` 等别的箱链接这个库时 `cfg(test)` 对库本身不开，走的是非测试那一支。测试构建与生产构建的唯一代码差别是限额的来源与计数。

推翻条件：找到一个输入 b，使得同一线程上先调过某个别的输入 c 之后 `crc32_castagnoli_table(b)` 与全新线程上第一次调的值不同。具体到上面三步，就是造出 `content == bytes` 而 `checksum != F(bytes)` 的候选，或造出非 I 的写点；读源码没有。

## 4. 新加的 21 行变异（`mutation-rows-new.tsv`）

先核形式（量的，脚本 `anchors.py`、`red_names.py`）：21 行的「原文」在各自文件里都恰好命中一次（`\n` 展开成换行后数）；21 行的替换文都与原文不同；第六段的 16 个测试名（有几行共用一个）在 `crates/` 下各恰有一个 `fn`。现行 `crates/mutations.tsv` 里这 21 行的行号是 481、1593、2688、3912、4355、7072 到 7080、7642、7986 到 7991（量的，整行比对，`crates/mutations.tsv` 共 8008 行）。

**行 1 到 5 不是新逻辑**：它们是 HEAD 里已有的五行，原文里的局部变量名或成员名随改名变了，键（文件、原文、替换文）因此出现为「新」（量的：HEAD 的 `crates/mutations.tsv` 里 `if a_record_sits_before_the_first_member`、`if a_later_root_is_incomplete {`、`if a_copy_of_the_class_the_entry_type_names_verifies`、`PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18`、`|| a_truncation_intent_names_the_record` 各 1 行）。真正新增的断点是行 6 到 21，共 16 行（9 + 1 + 6）。行 1、2、3、5 的点名测试我没读（逻辑没改，沿用 HEAD 的判，没重判）；行 4 读过：改坏把 `PackedRecord` 臂换成 `Data` 那个成员，测试断言「Some(ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheMergedVersionTotalOrderKey),」（`crates/singlefs-checker/src/rebuild.rs:1521`），断在那一处。

**行 6 到 21，改坏的那一处与点名测试（读码推的，一条都没跑）**：

| 行 | 改坏哪一处 | 点名测试怎么断 | 断在那一处？ |
|---|---|---|---|
| 6 | crates/singlefs-checker/src/walk.rs:2939 先走这片树表的那一版不记 | `…intermediate_version_without_a_deleted_inodes_entry_…_clean`：中间那一版树表只有它自己引用（测试里 `assert_ne!` 钉了），它是先走的；不记则这一对照判，测试自己的注释写「照判的话……就红了」，`violated_invariants == ["I-9.16"]` 先断 | 是 |
| 7 | crates/singlefs-checker/src/walk.rs:2221-2222 走完一版时不把记下的种类带进 `VersionPlacements…` | 同一个测试，同一断言 | 是 |
| 8 | crates/singlefs-checker/src/walk.rs:3776 共享树表的版本不记 | 库单测 `version_sharing_a_tree_table_walked_earlier_…`：先往 `tree_table_entries_of_a_walked_tree_table` 放一条码 16 的条目，调 `judge_the_tree_table_entries_an_earlier_version_walked`，断言 `Some(16)`；不记则 `None` | 是 |
| 9 | crates/singlefs-checker/src/walk.rs:2183 开始走下一版不清 | 同一个库单测随后调 `begin_walking_a_version()` 断言 `None`；自赋值则仍是 `Some(16)` | 是 |
| 10 | crates/singlefs-checker/src/walk.rs:1090 起前一侧加 `.take(0)`，前一版的种类不算 | `previous_version_with_…_leaves_the_pair_not_judged`：前一版带码 16、后一版 `None`；前一侧不看则这一对被判，谱系树里没有担保，判红，辅助函数 `assert_the_pair_reports_…` 的 `other @ (Holds \| Violated \| NotApplicable)` 那一臂 panic | 是 |
| 11 | crates/singlefs-checker/src/walk.rs:1092 去掉 `.chain(std::iter::once(version))`，后一版不算 | `later_version_with_…_leaves_the_pair_not_judged`：后一版带码 16、前一版 `None`；同上 | 是 |
| 12 | crates/singlefs-checker/src/walk.rs:824 码 6、7 一律 `false` | `only_unregistered_kinds_and_rooted_livelist_or_sparse_side_table_entries_are_shapes_without_a_clause` 对 6、7 断言「根指针非全零时为真」 | 是 |
| 13 | crates/singlefs-checker/src/walk.rs:823 码 14、15 改 `true` | 同一个测试里「走读走得下去」那串种类含 14、15，断言为假（两种根指针取值各判） | 是 |
| 14 | crates/singlefs-checker/src/walk.rs:825 没登记的不算（`is_some() && other_kind == 0`） | 同一个测试对 0、16、`u16::MAX` 断言为真；0 那一支 `key_schema_for_tree_kind(0)` 是 `None`，替换后为假 | 是 |
| 15 | crates/singlefs-checker/src/image.rs:975 去掉 `u32::try_from(…).is_ok()` | 新测试 `journal_ring_whose_in_flight_record_limit_overflows_…`：`3 << 44` 一点商 = 2³²，没有这一格则判据为真，与期望 `journal_ring_bytes == largest_fitting` 不符 | 是 |
| 16 | crates/singlefs-checker/src/lib.rs:149 比对改成恒真 | `table_crc32c_reusing_remembered_content_…_on_every_pass`：中间 8192 号字节只差 0x5A 的两段指纹相同、内容不同；第二段会取第一段的值，与按位那一份不同，第 0 遍就断 | 是 |
| 17 | crates/singlefs-checker/src/lib.rs:157 取第 0 个候选 | 同一个测试：两段在同一指纹下排 0、1 位；第 1 遍第二段位置是 1，取到第一段的值，断 | 是 |
| 18 | crates/singlefs-checker/src/lib.rs:188 `clear()` 换成 `shrink_to_fit()` | `clearing_the_remembered_content_at_the_limit_…`：限额 3 个节点、8 段互异内容，第 4 段起清空；不清则表里 4 段（65536 字节）超过限额 49152，`content_bytes_summed_over_the_table <= shrunk_limit` 断 | 是 |
| 19 | crates/singlefs-checker/src/lib.rs:201 `push` 换成 `drop` | `the_same_content_computed_again_on_the_same_thread_is_reused_…`：第二次调用应不现算；不记则现算次数 +1，断 | 是 |
| 20 | crates/singlefs-checker/src/lib.rs:196 `>=` 换成 `>` | `a_fingerprint_keeps_at_most_the_candidate_limit_…`：5 段同指纹，期望留 4；`>` 留 5，`most_candidates_under_a_single_fingerprint == 4` 断 | 是 |
| 21 | crates/singlefs-checker/src/lib.rs:218 `checksum_field_holds` 改走 `crc32_castagnoli_bitwise` | `checksum_field_holds_on_the_same_slot_twice_…`：头一次判之后现算次数应 +1；按位那一份不走计数，次数不动，断 | 是 |

两个变异的写法本身我核过会编过：行 16 的闭包参数写成 `_candidate`；行 19 的 `drop(RememberedChecksum { … })` 只换了开头，结尾的括号与分号照旧配上（`drop` 吃一个值，`candidates` 之后不再用）。行 6 与 8 用 `let _x = &entries…;` 保留借用，不引新未用警告。没编，推的。

**哪些判红分支没有任何一行变异罩着**（行 6 到 21 之外，读码推的）：

- crates/singlefs-checker/src/walk.rs:809「root_pointer_is_all_zero: root_pointer.all_zero,」：把它改成恒 `true`，码 6、7 带非零根的条目永远不算没有条款的形状。我在 `crates/singlefs-verification/tests/` 与箱内测试里找过：没有一条把码 6、7 的根指针写成非零再看 I-3.5 的测试（`TREE_KIND_LIVELIST` 的几处命中都是换种类、排次序、做空根条目）；库单测直接调纯函数，共享那条测试用字面构造的结构体，都绕过这一行。所以这个方向的变异会活下来（推的，没跑）。改成恒 `false` 的方向会让每个合法镜像丢掉 I-3.5 的判，别的测试会先红。
- crates/singlefs-checker/src/walk.rs:2270-2271 把版本的种类从「落点」结构带到「引用」结构：没有一行变异；`…intermediate_version…` 那条测试走真实的整条路，会断（推）。
- 记下的是「第一条」还是「最后一条」：没有测试有两条以上没有条款的条目，没有变异，也没有断言钉（推）。
- 记下的种类不进 `walk_failures`：没有变异；那条测试断言 `violated_invariants == ["I-9.16"]`，进了 `walk_failures` 会让 I-7.2 等也红，会断。
- crates/singlefs-checker/src/lib.rs:184 内容比限额还大就不记：没有变异，也没有测试用大于限额的内容（测试限额最小是 49152，输入最大 16384），这一道没有任何断言钉住。值不受影响，内存上界受影响。
- crates/singlefs-checker/src/lib.rs:187 触发清空的比较、crates/singlefs-checker/src/lib.rs:206 总数里减挤掉的那一段、crates/singlefs-checker/src/lib.rs:133-138 取样位置：没有变异；对应测试（`clearing_…` 的 `sum <= limit`、`a_fingerprint_keeps_…` 的 `recorded == 4 × 16384`、几条靠「同指纹异内容」造出来的断言）会断，推的。

推翻条件：任一行在改坏之后点名的测试仍绿，该行就不罩那一处。要判的话经门禁 `crates-mutation-replay.sh --item` 现判（第 8 节 C5）；上表是读码的预测，不是现判。

## 5. 检查单八格

| 格 | 判 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 见下 |
| 类型与名字 | 不成立（一点：`crates/singlefs-checker/src/walk.rs:825` 通配臂） | 见下 |
| 不兼容旧代码与旧格式 | 成立 | 见下 |
| 不新建判定库 | 成立 | 见下 |
| 所有任务多机可用 | 成立 | 见下 |
| 测试优先多线程 | 成立 | 见下 |
| 带 --item 的命令与项 | 成立（`--list-items` 能否列出待主 agent 跑） | 见下 |
| 新崩溃流多罩了什么 | 成立 | 见下 |

**走不到的理由二次查实：成立。** 我自己对 `changes.patch` 的加行逐行数（`awk` 跟着 `@@` 头算新文件行号，匹配 `unreachable!`、`panic!`、`.expect(`、`.unwrap(`、`assert*!`、`todo!`、`unimplemented!`、`as <整数类型>`、`_ =>`）：`image.rs` 在 `mod tests`（crates/singlefs-checker/src/image.rs:2083）之前、`rebuild.rs` 在 `mod tests`（crates/singlefs-checker/src/rebuild.rs:841）之前、`position_addressed.rs` 在 `mod tests`（crates/singlefs-checker/src/position_addressed.rs:356）之前、`walk.rs` 在 `mod tests`（crates/singlefs-checker/src/walk.rs:23208）之前没有一行命中；`lib.rs` 在 `mod tests`（crates/singlefs-checker/src/lib.rs:1294）之前命中三行。逐行：

- 「.set(comparisons.get() + u64::try_from(compared).expect("候选数装得进 u64"));」（`crates/singlefs-checker/src/lib.rs:154`）：上面一行 `#[cfg(test)]`（crates/singlefs-checker/src/lib.rs:150），只在测试构建里。`compared` 是 `candidates.len()` 或某个位置，≤ 候选上限 4，usize→u64 在 64 位上不会失败。**真正新加的只有这一处。**
- 「let mut remainder = u32::try_from(index).expect("小于 256");」（`crates/singlefs-checker/src/lib.rs:163`）与「[usize::from(u8::try_from(remainder & 0xFF).expect("低 8 位") ^ *byte)]」（`crates/singlefs-checker/src/lib.rs:178`）：**不是新加的**。HEAD 的同一个文件里原本就有这两句：`git show HEAD:crates/singlefs-checker/src/lib.rs | grep -n 'expect("小于 256")\|expect("低 8 位")'` 打出 `67:            let mut remainder = u32::try_from(index).expect("小于 256");` 与 `81:        remainder = table[usize::from(u8::try_from(remainder & 0xFF).expect("低 8 位") ^ *byte)]`（另有 1219 行是测试里的）。这次只是挪进闭包、缩进变了，diff 里显示成 `+`。依赖的不变量：前一句的 `index` 来自对 256 项数组的 `enumerate()`，< 256；后一句 `remainder & 0xFF` ≤ 255。两句成立。
- 其余可能 panic 的点（不是 `expect`，但我也逐个看过，第 3 节）：下标、切片、减法、`borrow_mut`、`candidates[position]`，没有走得到的越界或重入。

**类型与名字：不成立，只一点。**

- 新名全是说出内容的完整名字，没有缩写，没有序数代称（`REMEMBERED_CONTENT_BYTES_LIMIT_PER_THREAD`、`REMEMBERED_CANDIDATES_PER_FINGERPRINT_LIMIT`、`FINGERPRINT_SAMPLED_WORD_COUNT`、`ContentFingerprint`、`RememberedChecksum`、`RememberedChecksumsOfThisThread`、`WORD_BYTES`、`tree_table_entry_is_a_shape_without_a_clause`、`note_the_first_tree_table_entry_without_a_clause`、`kind_of_a_tree_table_entry_without_a_clause_in_this_version`、`root_pointer_is_all_zero`、rebuild 三个成员）。种类用 `Option<u16>`，与同一份文件里全部 `TREE_KIND_*` 同型，不是新引入的裸整数。
- 通配臂：「other_kind => key_schema_for_tree_kind(other_kind).is_none(),」（`crates/singlefs-checker/src/walk.rs:825`）。**它算通配臂**：有名字的捕获，语义上等于 `_ =>`，接住 `u16` 里没列出的全部值（0、5、8 到 13、16 以上）。`kind` 是盘上读来的编码，`code-discipline.md` 对这种值的写法是：「**语义上允许未知值的**（盘上读到的、协议里收到的、以后还会加新值的编码），把「未知」做成一个显式成员」（`.claude/singlefs-ai-sop/rules/code-discipline.md:101`），检查单这一格的判据是「不缩写、不写 `_ =>`、`expect` 写清依赖哪条不变量」（`.claude/skills/three-way/SKILL.md:83`），所以按字面算打中。减轻的几点：① 这个箱里树种类没有枚举，全是 `const u16`，不是新引入的写法；同一文件在 HEAD 里已有 11 处 `_ =>` 与几处 `_outside_the_three_unit_classes =>` 这类具名通配；② 臂体不是丢弃，而是查 `key_schema_for_tree_kind`，它自己的末臂是 `_ => None`（`crates/singlefs-checker/src/lib.rs:887`），所以以后登记新种类而没加 key 形态时，这里默认「没有条款」，方向是多不判、不是多红，是安全方向；③ 改成「显式未知成员」要先给树种类立一个枚举，波及整个 `walk.rs`，超出本轮。这一点是不是打中、要不要立枚举，交主 agent 判。
- `expect` 写清不变量：crates/singlefs-checker/src/lib.rs:154 的「候选数装得进 u64」只写了期望、没写依赖「候选数 ≤ 上限 4 且 usize 不宽于 u64」这条不变量，措辞偏弱；它在 `#[cfg(test)]` 里。163、178 是旧句。

**不兼容旧代码与旧格式：成立。** 我对加行再扫一遍关键词（旧版、旧布局、旧格式、legacy、migrat、迁移、兼容旧、backward、此前、原先、起初、最初、后来、改成了、换成了、修过），命中为零（命中的只有两行注释里的「崩溃」二字，与本格无关）。没有认旧版本的分支、旧常量、迁移路径、造旧形态输入的测试。改名只去掉成员名里的编号 `I18`，不留旧名别名。

**不新建判定库：成立。** 按内容复用是 `thread_local!` 里的 `HashMap`，只存「内容 → CRC」，进程内、不落盘、线程退出即丢。`.claude/rules/verification.md` 对判定库的定义是「库里只放剪枝按单元键复用时要读的键」（`.claude/rules/verification.md:54`）：存的是判定，经 prune-server 那一份。这里存的是纯函数 CRC 的值，不是任何一条不变量的判定，也不是剪枝键。不属「判定复用」，不受那一条管。加行里 `rocksdb`、`sqlite`、`prune` 三个关键词零命中。

**所有任务多机可用：成立。** 加行里没有 `hostname`、`/proc/`、`/sys/`、`env::var`、`std::env`，新测试只用内存里的字节、`std::thread::spawn`、不读文件。`crates/singlefs-checker/src/lib.rs:60` 的 `.claude/heavy-thread-default` 只在注释里，不读。

**测试优先多线程：成立。** 新测试都是单场景小用例，没有可切片的大批量；lib.rs 那五条里的 `std::thread::spawn`（`crates/singlefs-checker/src/lib.rs:1359`）是为让线程局部记录从空的起，helper 的文档注释（crates/singlefs-checker/src/lib.rs:1354 起）写明了；没有 `--test-threads`、`available_parallelism`、线程数常量。

**带 --item 的命令与项：成立。** `grep -c 'test=singlefs-checker' .claude/gate.d/stage-inputs.tsv` 得 0，没有崩溃用例登记行指到池级 checker 的测试目标（`crash-case:` 行里带 `checker` 字样的 26 行是验证包里测试名的一部分，不是这个箱的目标）；新测试随 `cargo test -p singlefs-checker --lib` 与验证包已有目标点名。变异行 16 条新的、5 条换键的，经 `crates-mutation-replay.sh --item <行号或变异名>` 点名，`--list-items` 能否都列出来要主 agent 跑（第 8 节 C4），我没跑。
- 小注：`a_fingerprint_keeps_at_most_the_candidate_limit_and_evicts_the_earliest_remembered_content` 是这一批新写的测试，名字以 `a_` 起头，而这一批正把 76 个名字的 `a_`、`an_` 去掉。`naming-lint.sh` 对打头的 `a` 加别的段按英语冠词放行（`.claude/singlefs-ai-sop/scripts/naming-lint.sh:12`），不判红，只是与这一批的方向不一致。

**新崩溃流多罩了什么：成立（本轮没有新崩溃流）。** 加行非注释行里没有 `crash`、`崩溃`；没有新加 `crash-case:` 登记，没有新加崩溃枚举用例。行为上，CRC 复用跑在崩溃点重放每个崩溃状态的判里（这是它的用途，注释 crates/singlefs-checker/src/lib.rs:57-60、crates/singlefs-checker/src/lib.rs:115 写了），不改变任何一个状态的判定结局（第 3 节）。

推翻条件：走不到格被推翻，需要在生产段新加行里找到一个能 panic 的点；类型格，要在新加的名字里找到缩写或序数；判定库格，要找到把 CRC 值或判定写到进程外的路径。

## 6. 事实表（`facts.md`）与正文写错的地方

每条都是我自己对 `changes.patch` 或源文件现查的，输出见该条。

1. **`lib.rs` 生产段「新加 3 处 `expect`」写错，真新加的只有 1 处**，正文检查单第一格同样。`crates/singlefs-checker/src/lib.rs:163`「小于 256」与 `crates/singlefs-checker/src/lib.rs:178`「低 8 位」是 HEAD 里就有的两句（`git show HEAD:crates/singlefs-checker/src/lib.rs | grep -n 'expect("小于 256")\|expect("低 8 位")'` 打出 `67:` 与 `81:` 两行，原样见第 5 节），这次挪进闭包、缩进变了，diff 里显示成 `+`。`crates/singlefs-checker/src/lib.rs:154` 真新加，且在 `#[cfg(test)]` 里（crates/singlefs-checker/src/lib.rs:150）。
2. **`lib.rs` 新测试「6 条」写错，是 5 条**（正文「新测试 6 条」、`facts.md` 的「6 个 #[test]」）：`sed -n 340,665p changes.patch | grep -c '^+    #\[test\]'` 得 5。6 是 `lib.rs` 新变异行的条数。
3. **`facts.md` `image.rs` 一节第 4 样把「改到的函数」写成 `physical_block_size_fits_a_root_slot`**：那是 hunk 头借来的上文，没改。改到的是 `journal_ring_bytes_lie_in_the_supported_range`（`crates/singlefs-checker/src/image.rs:965`），它被 `geometry_of`（crates/singlefs-checker/src/image.rs:895）与新测试（crates/singlefs-checker/src/image.rs:2488）引到。「两处都没引到的 1 个」不成立。
4. **`facts.md` `walk.rs` 一节「三个 fn 名去 a_」写错，是一个生产函数、三处出现**：`a_truncation_intent_names_the_record`→`truncation_intent_names_the_record`（定义 crates/singlefs-checker/src/walk.rs:15081，调用 15137、15211）。它还被第 3 样记成「生产段新加的条目名」，其实是改名。正文第 5 项只提「测试函数与局部变量」，漏了这个生产函数。
5. **数目不齐**：`image.rs`「七个测试去 a_」，块清单与我数的都是 8；`lib.rs`「三个测试去 a_」，实际 4（第四个在 +315 行那一大块里）；`walk.rs`「约 40 个」，量的是 39 个标识符改名，其中 34 个 `a_` 测试函数、1 个展开缩写的测试（`quarantined_…_between_t_and_theta_high_…`，crates/singlefs-checker/src/walk.rs:27327）、3 个局部变量、1 个生产函数。
6. **变异「新增 21 行」里 5 行是旧行换键**（第 4 节）：真新断点 16 行。`facts.md` 写「新增 walk 13、rebuild 1」，其中 walk 4 行、rebuild 1 行是 HEAD 已有的行因改名而键变，不是新覆盖。
7. **正文行号**：「`crates/singlefs-checker/src/walk.rs:823` 的臂写成 `other_kind => …`」，该臂在 `crates/singlefs-checker/src/walk.rs:825`（823 是码 14、15 那一臂）；「`crates/singlefs-checker/src/walk.rs:7270` 一处文档注释改成普通注释」，现行行号是 `crates/singlefs-checker/src/walk.rs:7273`（7270 是 hunk 起点）。
8. **`facts.md` 线索两条我已核，不必再留**：登记表到 15 为止（`.claude/kb/decisions/08-核心索引结构.md:337` 的「| 15 | 快照 extent 根 | 已定项 8 |」，码 16 是第一个没登记的，测试常量对）；「I-9.16 ④ 判红」只对顶掉十二种之一的形态真，见第 1.3 节。

## 7. 本轮之外、主 agent 要知道的

1. **kb 里一处旧成员名**：「改名 `DataUnitVersionsTiedOnTheTotalOrderKeyOfI18`」（`.claude/kb/checks-owed.md:961`，同一行的 `…TieOnTheTotalOrderKeyOfI18` 是同一个旧名），三语副本 `.en.md`、`.ja.md` 同有（`grep -rIlF` 在 `.claude/kb/` 里命中三份）。旧名在代码里已不存在，这一行是对着已删名字的叙述。不在这五份文件里，不归我改。
2. **实验装置对 CRC 查表函数计时**：`crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs` 的 `replay_unit_check` 对每个单元调「black_box(singlefs_checker::crc32_castagnoli_table(bytes));」（`crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs:1722`）。复用之后，同一进程里重复算同一段内容的那一遍测到的是记录命中的代价（指纹加逐字节比对），不是查表 CRC 的代价；E161 以前的读数是没有复用时量的。这个装置的结论依赖哪些读数我没查，推的：装置的读数与实现对不上时两边都查（`.claude/rules/implementation-first.md` 第 4 条）。要不要为它给这个调用换成不经复用的写法，主 agent 判。
3. **复用的耗时收益没量**：`crates/singlefs-checker/src/lib.rs:57-60` 与 `crates/singlefs-checker/src/lib.rs:115` 的注释写「几十到几百个 16 KiB 的单元」「相邻崩溃状态只差一次写，绝大多数单元逐字节相同」，没有读数；复用收益与内存上界都没量，推的。

## 8. 要主 agent 跑的命令

没有写模型，没有 `rerun.sh`，没有复跑脚本。下面是预测的核对，条条在仓根起（`--cwd` 缺省即仓根）。读数我都没有，上面所有「红」「绿」是读码推的。

- **C1** 这一批新增与相关用例，11 条：
  `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker 库测试点名 11 条新增用例; 范围=1 个测试目标（--lib）、11 条用例; 为什么=核正推腿读码推出的断言在现行未弄坏的代码上都绿' task-server/task submit -- cargo test -p singlefs-checker --lib -- table_crc32c_reusing_remembered_content_agrees_with_the_bitwise_crc32c_on_every_pass clearing_the_remembered_content_at_the_limit_keeps_every_checksum_equal_to_the_bitwise_crc32c the_same_content_computed_again_on_the_same_thread_is_reused_and_another_thread_computes_its_own a_fingerprint_keeps_at_most_the_candidate_limit_and_evicts_the_earliest_remembered_content checksum_field_holds_on_the_same_slot_twice_reuses_the_remembered_checksum_the_second_time journal_ring_whose_in_flight_record_limit_overflows_its_four_byte_field_lies_outside_the_supported_range later_version_with_a_tree_table_entry_of_an_unregistered_kind_leaves_the_pair_not_judged previous_version_with_a_tree_table_entry_of_an_unregistered_kind_leaves_the_pair_not_judged only_unregistered_kinds_and_rooted_livelist_or_sparse_side_table_entries_are_shapes_without_a_clause version_sharing_a_tree_table_walked_earlier_takes_its_entry_without_a_clause_until_the_next_version two_packed_versions_with_the_same_total_order_key_are_not_decided`
  看：`TASK_SERVER_RESULT` 行 `type=` 与 `exit=0`；`task-server/task log <号>` 里 `test result: ok. 11 passed; 0 failed`（11 是推的，数一遍点名的名字）。同一趟把检查器箱的测试代码整个编了一遍，七十六处改名的编译错在这里出。
- **C2** 环长新一格 core 与池级 checker 同拒（核第 1.1 节的边界）：
  `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 system_configuration_fields_core_and_checker_refuse_the_same_table; 范围=1 个测试目标; 为什么=核 3<<44 一格 core 的只读与可写挂载都拒、池级 checker 报 I-7.13 违例' task-server/task submit -- cargo test -p singlefs-verification --test system_configuration_fields_core_and_checker_refuse_the_same_table`
  看：`exit=0`，日志里 `test result: ok` 且 `0 failed`。
- **C3** I-3.5 整条路走真镜像（变异行 6、7 的测试）：
  `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 pool_checker_judges_the_deletion_state_of_every_walked_version; 范围=1 个测试目标; 为什么=核中间一版树表有没登记种类的镜像上 I-3.5 报部分没判而 I-7.2 等不红' task-server/task submit -- cargo test -p singlefs-verification --test pool_checker_judges_the_deletion_state_of_every_walked_version`
  看：`exit=0`，`0 failed`。
- **C4** 列项（不起 cargo），核 21 行都能被点名、锚点没失效：
  `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 的 --list-items; 范围=列项、不起 cargo; 为什么=核这一批 21 行新变异都能被点名、锚点没失效' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items`
  看：末尾「锚点失效」一栏为 0 行；列出的变异名里含第 4 节那 21 行（行号 481、1593、2688、3912、4355、7072 到 7080、7642、7986 到 7991）。
- **C5** 21 行逐行现判（第 4 节表的预测对不对）：
  `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 带 --item 点名这一批 21 行新变异; 范围=21 行; 为什么=核每行改坏之后点名的测试真红，读码预测是否成立' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item 481 --item 1593 --item 2688 --item 3912 --item 4355 --item 7072 --item 7073 --item 7074 --item 7075 --item 7076 --item 7077 --item 7078 --item 7079 --item 7080 --item 7642 --item 7986 --item 7987 --item 7988 --item 7989 --item 7990 --item 7991`
  看：成功行写明 21 行全抓到；有行没红，那一行就不罩那一处。点名的四个验证包目标都不读崩溃缩域变量（`grep -l` 对 `checker_known_bad_images.rs`、`checker_truncation_holes_and_intent_must_red_worlds.rs`、`pool_checker_judges_the_deletion_state_of_every_walked_version.rs` 零命中），范围声明闸若仍要全域理由，照它的拒绝说明补。行号是此刻 `crates/mutations.tsv`（sha256 2c1b068f…）的行号，表变了就换成变异名。
- **C6** 两条命名检查我没起：本轮我只读不改，没有改 `crates/` 下的 `.rs`，命名检查归写这些改动的人。

## 9. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1.1 环长一格 | 对得上 | 式子、4 字节、`3 << 44` 边界、core 同判都对；「装不进就拒」那句条款是同批在途文字，独立依据是 `.claude/kb/layout/01-first-txn.md:136` 的宽 4 |
| 1.2 CRC 按内容复用 | 对得上（无条款，对的是同值性质） | 三步都推得出同值；耗时与内存上界推不出，没量 |
| 1.3 I-3.5 没有条款的形状 | 对得上，一处注释与条款同样过宽 | 种类没登记、码 14/15、共享、清空、「不进走读失败」都对；码 6、7 非零根只有同批在途状态格作据；「I-9.16 ④ 判红」只对顶掉十二种之一的形态真 |
| 1.4 三个成员改名 | 对得上 | 名字取自 I-1.8 的标题与 ①②，只改标识符 |
| 1.5 改名批 | 纯改名 | 106 块、120 段，90 段纯改名，25 个块是真改动；76 个标识符改名无碰撞无残留；多出一个生产函数改名与一个缩写展开 |
| 第 3 节 复用同值 | 推得出（三步） | 不变量 I 加三步；内存上界与线程退出析构那两处推不出 |
| 第 4 节 21 行变异 | 16 行新断点，读码都断在那一处；5 行旧行换键 | 没罩的：`crates/singlefs-checker/src/walk.rs:809` 非零根接线（恒 true 方向）、第一条还是最后一条、`crates/singlefs-checker/src/lib.rs:184` 超限不记 |
| 检查单：走不到的理由 | 成立 | 真新加的只有 `crates/singlefs-checker/src/lib.rs:154`，在 cfg(test) 里；163、178 是旧句 |
| 检查单：类型与名字 | 不成立 | 一点：`crates/singlefs-checker/src/walk.rs:825` 是通配臂（安全方向，树种类没有枚举）；交主 agent 判要不要立枚举 |
| 检查单：不兼容旧 | 成立 | 关键词与结构两样都零命中 |
| 检查单：不新建判定库 | 成立 | 进程内线程局部、不落盘、存的是 CRC 值不是判定 |
| 检查单：多机 | 成立 | 无本机状态 |
| 检查单：多线程 | 成立 | 小用例；那一条 `thread::spawn` 为的是线程局部记录从空起 |
| 检查单：带 --item | 成立（`--list-items` 待跑 C4） | 没有 stage-inputs 行指到这个箱 |
| 检查单：新崩溃流 | 成立 | 本轮没有新崩溃流 |
| 事实表写错 | 7 条 | 见第 6 节 |

## 没做什么

- 没跑任何测试、变异、门禁、编译，没往任务服务交任务；第 4 节「断在那一处」与第 3 节同值性都是读码推的，C1 到 C5 的读数没有。
- 没替主 agent 判采纳；`crates/singlefs-checker/src/walk.rs:825` 通配臂算不算打中、码 6、7 非零根要不要有独立条款、`e161` 装置要不要改，都留给主 agent。
- 没读禁读清单里的另一条腿的报告；没读别的轮的判决与腿报告。
- 没量内存上界（每条记下的内容的额外开销）与复用的耗时收益；没查 GPU 判器那一侧（`.wgsl` 内核）与 CPU 判器在 I-3.5 这一路是不是同判，状态格说同判、对拍在验证包里，我没读。
- 没核 I-9.27 对「十二种之外多一条码 16」的树表红不红；只说了 I-9.16 不红。
- 没把五份源文件以外的 `crates/`、`scripts/` 当判对象；第 7 节三条是顺带读到的。
- 草稿目录里只有我的核对脚本与输出，没有仓副本、编译目录。
