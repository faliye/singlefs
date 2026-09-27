# 实审 C11 交回：码 2 头宽改成三方各算一份 + 交叉断言（代码审阅第 11 条）

时刻：2026-09-26 22:52 UTC 起做（JST 2026-09-27 07:52 起）。交法：**交补丁**。改动都在仓副本 `/tmp/claude-1000/impl-rev-c11/repo` 里，主工作区一个字没碰；补丁目录 `/tmp/claude-1000/impl-rev-c11/patch/`。

## 结论

1. `singlefs-format` 里删了 `index_node_header_bytes` 这个 `const fn`（主工作区 `crates/singlefs-format/src/lib.rs:50`），加了一个标量 `INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2`（key 区间里的 key 个数）；`INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE`（86）、`NONCE_MAC_ALGORITHM_RESERVED_BYTES`（29）原样留着。
2. 三方各写一份，三份都叫 `index_node_header_bytes`，同一个概念用同一个名字，靠 crate 路径区分。三份互不调用，也不调同一个 helper：
   - core：`crates/singlefs-core/src/unit.rs:140`，参数 `key_width: usize`。三个调用点都改成用 core 自己的式子：明文头末尾走同文件 `:126` 的私有函数 `index_node_plain_header_end`（在 `:171` 建节点、`:365` 解节点时用），条目区起点在 `:147` 用 `index_node_header_bytes`。
   - checker：`crates/singlefs-checker/src/lib.rs:301`，参数是盘上偏移 51 那 1 字节原样（`u8`），沿用 `check_unit` 今天直接从 `unit[51]` 取 key 宽的读法；`check_unit` 在 `:331` 调它，再减 29。
   - 模型：`crates/singlefs-harness/src/model.rs:267`，参数 `u64`；`index_node_entry_capacity`（`:274`）改调它。
   - format 自己的单测（`crates/singlefs-format/src/lib.rs:466`）里另写了一份只给测试用的私有式子，用来核那几个字面量，这样原有 `crates/mutations.tsv` 第 337–342 行（这 6 行按名字点的都是 format 的这条单测）原样有效。
   （以上行号都是副本里终版文件的行号。）
3. 交叉断言：新测试文件 `crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`，里面两条：
   - `core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte`：key 宽从 0 扫到 255 共 256 个值，每个值上 `assert_eq!(core, checker)`、`assert_eq!(checker, model)`；再核「头宽 + 一个只有 key 的条目 ≤ 16384」；最后断言确实扫了 256 个。
   - `each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width`：kb 里带 `format-const` 标记的 6 个头宽（inode 树根 131 / 树表单元 131，key 宽 8；分配记录树 135，key 宽 10；记账树 159，key 宽 22；extent 树 163，key 宽 24；中央映射树 169，key 宽 27）逐个先与格式常量比，再与三份在对应 key 宽上的结果比。
4. 取值域上界怎么来的（测试文件头注释里也写了）：
   - ① key 宽字段在码 2 头偏移 51、宽 1 字节（D18 已定项 18 的偏移表），所以上界是 `u8::MAX` = 255；
   - ② 节点至少要装得下一个条目。条目以 key 打头，条目宽不小于 key 宽，所以要满足 115 + 3k ≤ 16384，得 k ≤ 5423。② 比 ① 松，上界取 255；第一条测试在每个 k 上顺带核 ②，② 哪天变得比 ① 紧，它会先红；
   - 下界 0：字段里能写 0，core 的 `parse_index_node` 和 checker 的 `index_node_view` 都照读、不拒。
5. 这次加了 14 行变异，全部用 `prove-red.sh` 在副本上证过，14 行都抓到（core 那一组 2 行，harness 那一组 12 行）。其中 4 行（checker 只在 k ≥ 128 时歪、模型只在 k = 0 时歪、两个共享标量各改一次）另外跑了一遍新测试的整个二进制，看同一个二进制里另一条测试红不红。已有的第 337 行（format 单测）在改后的代码上也重证了一遍，抓到。
6. 同类的 `const fn` 找齐了：能被 core、checker、模型三方中两方以上共用的 `const fn`，只能住在 `singlefs-format` 里，而那里只有 2 个：`index_node_header_bytes`（这次改了）和 `journal_in_flight_record_limit`（没改，理由和用在哪见下文「同类 const fn」一节）。
7. 门禁（都在副本上跑）：27、53、94、93 号绿；92、89 号退 77，本次没跑成；33 号红，红的 6 行都是别人文件里的锚点，不是我的行；74 号红两条，把我的四份换回原件的基线上照样红这两条，与这次改动无关。各道门禁的原样末行见下文「验证输出」一节。

什么现象会推翻这些结论：
- 把补丁打进主工作区后，新测试二进制或下文列出的任何一个测试二进制红了；
- 三份式子里有一份在某个 k 上和另外两份不等，而新测试没红；
- 主工作区里另有一份被 core、checker、模型中两方以上调用的带算术 `const fn` 或 `fn`，却不在 `singlefs-format` 里。按依赖关系走不到这种情况：checker 只依赖 `singlefs-format`，`crates/singlefs-checker/src` 里提到 `singlefs_core` 的只有 `lib.rs:126` 一行文档注释；`model.rs` 的 `use` 只有 `std` 与 `singlefs_format`。

## 这一轮写过的文件

补丁里的（主工作区还没打；`git apply --check` 对主工作区此刻的状态能过）：
- `crates/singlefs-format/src/lib.rs`
- `crates/singlefs-core/src/unit.rs`
- `crates/singlefs-checker/src/lib.rs`
- `crates/singlefs-harness/src/model.rs`
- `crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`（新建）
- `crates/mutations.tsv`：没直接改，要追加的 14 行放在 `patch/mutations-append.tsv`，变异名见下文「变异」一节。

这四份原件取副本时的 sha256，与交回前主工作区现查的完全一致，说明取副本之后主工作区里这四份没被别人动过：
`crates/singlefs-format/src/lib.rs` b4f79a7c…、`crates/singlefs-core/src/unit.rs` ae7f272d…、`crates/singlefs-checker/src/lib.rs` 5601e2fb…、`crates/singlefs-harness/src/model.rs` 5068b1f7…（全值在 `/tmp/claude-1000/impl-rev-c11/copy-sha256.txt`）。
取副本时别的会话在改的文件，sha256 也记在同一份文件里。那一刻副本整体编得过，没有拿 A2b、A2c 开工前的原件去顶。

`git apply --numstat patch/crates.patch`（原样）：
```
17	7	crates/singlefs-format/src/lib.rs
26	17	crates/singlefs-core/src/unit.rs
24	8	crates/singlefs-checker/src/lib.rs
17	4	crates/singlefs-harness/src/model.rs
139	0	crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs
```
主工作区的 `git diff --stat -- crates litmus` 里全是别的会话还没提交的改动，这一轮一行都不是我的（我没碰主工作区），所以不贴。

## 受影响的层 0 流与崩溃枚举用例

**没有。** checker 这边只动了 `check_unit` 算码 2 明文头末尾的写法，判定集合没变：
- 改前：`format::index_node_header_bytes(unit[51]) − 29`，即 `86 + 2 × unit[51] + 29 − 29`；
- 改后：`checker::index_node_header_bytes(unit[51]) − 29`，是同一条式子，只是写在 checker 里。

新测试在 key 宽 0..=255 上逐个核过 checker 那份与 core、模型相等，又在 6 个登记值上核过它与原 format 那份的字面量相等。所以任何一个镜像上，`check_unit` 的头校验和覆盖范围和载荷 CRC 的偏移都与改前逐字节相同。层 0 各流、崩溃枚举用例的钉值都不会变，不用排快档。core 的建节点、解节点和节点容量同理：取值没变，只是换了写法。

## 同类 const fn

找法：`grep -rn 'const fn' crates/*/src` 命中 76 处，57 个不同的名字。对每个名字数它在 core `src`、checker `src`、`model.rs` 里出现的文件数，两方以上都出现的有 14 个名字。除了 `index_node_header_bytes`，其余 13 个（`bytes`、`contains`、`kind`、`name`、`of`、`placement`、`position`、`record_flags_byte`、`root`、`slots`、`tree`、`unit_class`、`unmount_marker`）都是同名、各方各自声明的方法，不是同一个函数。因为 checker 只依赖 `singlefs-format`，`model.rs` 只 `use` 了 `std` 与 `singlefs_format`，三方之间要共用一个 `const fn`，只能经由 `singlefs-format`。而 `grep -rn 'const fn' crates/singlefs-format/src` 在主工作区只命中两处：

| const fn | 式子 | 谁在用 | 这一件改没改 |
|---|---|---|---|
| `index_node_header_bytes`（format `lib.rs:50`） | `86 + 2 × key 宽 + 29` | core `unit.rs`（三处）、checker `lib.rs`（`check_unit`）、模型 `model.rs`（`index_node_entry_capacity`） | 改了：format 只留标量，三方各算一份，另加交叉断言 |
| `journal_in_flight_record_limit`（format `lib.rs:218`） | `环字节数 ÷ 4096 ÷ 3` | core `make_filesystem.rs:277`、`recovery.rs:2147`、`system_configuration.rs:368`；harness 实验装置 `src/bin/e158_root_choice_repair.rs:8067`。checker 与模型都不用 | 没改：`make_filesystem.rs`、`recovery.rs` 正被实审 A2b 改着；照规格只列不改，等 A2b 交回另派 |

`journal_in_flight_record_limit` 今天只有 core 一方在用，另有一个实验装置在用，还不构成「两方以上共用」。它违反的是 D13 已定项 5 的另一半：发射物里不许有算术。等 A2b 交回另派时要主 agent 定两件事：搬进 core 自己写一份，还是 checker 或模型也要各算一份；以及 e158 装置那一处怎么办。

顺带看到、不属于这一件的：format 里有 8 个 `pub const` 的值本身是算式，D13 已定项 5 的判据是「发射物里有没有分支与算术」，这 8 个也算在「算术」里：
- `DATA_UNIT_PAYLOAD_OFFSET`（`:36`）
- `PACKED_UNIT_RECORDS_OFFSET`（`:43`）
- `JOURNAL_NEW_ROOT_SEGMENT_BYTES`（`:195`）
- `JOURNAL_NAMED_ENTRY_BYTES`（`:201`）
- `JOURNAL_NAMED_ENTRIES_PER_RECORD`（`:205`）
- `JOURNAL_RING_DEFAULT_BYTES`（`:212`）
- `ROOT_RING_CHUNK_BYTES`（`:258`）
- `TEST_IMAGE_DEFAULT_BYTES`（`:282`）

（行号都是主工作区现行 format `lib.rs` 的。）其中 `DATA_UNIT_PAYLOAD_OFFSET` 与 `JOURNAL_NAMED_ENTRY_BYTES` 被两方用着：前者 checker 与模型，后者 core 与 checker。算不算、要不要一起改，交主 agent 定（见下文「设计问题」一节）。

## 变异（追加 14 行，`patch/mutations-append.tsv`，六段；和主表比过，名字都不重）

每条新测试「改坏哪一行 → 哪条断言红」（行号是副本终版文件的；红的那句断言消息照日志原样抄）：

| 变异名 | 改坏哪一行 | 哪条测试的哪条断言红 |
|---|---|---|
| core 那份 key 区间只算一个 key | `unit.rs:127–129` 的 `* key_width` 改成 `* key_width / 2` | 交叉那条：`key 宽 1：core 算的头宽与 checker 算的对不上`（116 对 117） |
| core 那份漏了预留位 29 | `unit.rs:141` 去掉 `+ reserved_bytes()` | 交叉那条：`key 宽 0：core 算的头宽与 checker 算的对不上`（86 对 115） |
| core 建码 2 节点时明文头末尾错取含预留位的头宽 | `unit.rs:171` | 已有的 `unit::tests::empty_tree_table_node_has_key_width_at_51_and_a_sealed_header`（`码 2 头 的偏移与字段表不符` 102 对 131），同一个二进制里 `the_three_parsers_round_trip_…` 也红 |
| core 解码 2 节点时明文头末尾错取含预留位的头宽 | `unit.rs:365` | 已有的 `unit::tests::the_three_parsers_round_trip_their_builders_and_refuse_a_damaged_header`（`码 2: HeaderChecksumMismatch`） |
| core 算节点容量时条目区起点少了预留位（记账树叶多装一行） | `unit.rs:147` | 已有的 `eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing`（`[240, 243]` 对 `[239, 244]`） |
| checker 那份 key 区间只算一个 key | `lib.rs:302–303` | 交叉那条：`key 宽 1：core 算的头宽与 checker 算的对不上`（117 对 116） |
| checker 那份只在 key 宽 128 及以上歪 | `lib.rs:303` 取值前 `& 0x7f` | 交叉那条：`key 宽 128：core 算的头宽与 checker 算的对不上`（371 对 115）；钉绝对值那条仍绿（6 个登记的 key 宽都小于 128），说明这一条靠的是扫遍取值域 |
| checker 那份漏了预留位 29 | `lib.rs:306–307` | 交叉那条：`key 宽 0：core 算的头宽与 checker 算的对不上`（115 对 86） |
| checker 判单元头时明文头末尾没减预留位 | `lib.rs:331–332` | 已有的 `every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold`（`树表单元: ChecksumMismatch`） |
| 模型那份 key 区间只算一个 key | `model.rs:268` | 交叉那条：`key 宽 1：checker 算的头宽与理想模型算的对不上`（117 对 116） |
| 模型那份只在 key 宽 0 上歪 | `model.rs:268` 的 key 宽改取 `.max(1)` | 交叉那条：`key 宽 0：checker 算的头宽与理想模型算的对不上`（115 对 117）；钉绝对值那条仍绿，说明这一条靠的是扫到下界 |
| 模型那份漏了预留位 29 | `model.rs:269` | 交叉那条：`key 宽 0：checker 算的头宽与理想模型算的对不上`（115 对 86） |
| 共享标量 key 区间的 key 数写成 3 | format `lib.rs:56` | 钉绝对值那条：`inode 树根（INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES）：core 在 key 宽 8 上算的头宽不是登记的 131`（139）；交叉那条仍绿，三份一起歪时只有钉绝对值看得出来 |
| 共享标量 key 区间之外的头宽写成 87 | format `lib.rs:47` | 钉绝对值那条，同上一行的消息（132）；交叉那条仍绿 |

变异名在表里的全称都以「实审 C11 三方各算头宽：」开头，全文见 `patch/mutations-append.tsv`。14 行都用 `prove-red.sh` 证过，没有留给 59 号才证的。

`prove-red.sh` 原样输出（core、format（重证已有的第 337 行）、harness 三次，线程上限 5，内存上限取默认 8G）：
```
实审 C11 三方各算头宽：core 建码 2 节点时明文头末尾错取含预留位的头宽	抓到	empty_tree_table_node_has_key_width_at_51_and_a_sealed_header 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/core/001.log）
实审 C11 三方各算头宽：core 解码 2 节点时明文头末尾错取含预留位的头宽	抓到	the_three_parsers_round_trip_their_builders_and_refuse_a_damaged_header 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/core/002.log）
✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了
exit=0
格式常量字面量改掉一个数：INODE_TREE_ROOT_INDEX_NODE_HEADER_BYTES	抓到	tree_index_node_header_literals_equal_the_header_formula_at_each_tree_key_width 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/format/001.log）
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
exit=0
实审 C11 三方各算头宽：core 那份 key 区间只算一个 key	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/001.log）
实审 C11 三方各算头宽：core 那份漏了预留位 29	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/002.log）
实审 C11 三方各算头宽：core 算节点容量时条目区起点少了预留位（记账树叶多装一行）	抓到	eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/003.log）
实审 C11 三方各算头宽：checker 那份 key 区间只算一个 key	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/004.log）
实审 C11 三方各算头宽：checker 那份只在 key 宽 128 及以上歪（盘上那一字节丢了最高位，只有扫遍取值域才看得见）	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/005.log）
实审 C11 三方各算头宽：checker 那份漏了预留位 29	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/006.log）
实审 C11 三方各算头宽：checker 判单元头时明文头末尾没减预留位	抓到	every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/007.log）
实审 C11 三方各算头宽：模型那份 key 区间只算一个 key	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/008.log）
实审 C11 三方各算头宽：模型那份只在 key 宽 0 上歪（按至少 1 字节算，只有扫到下界才看得见）	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/009.log）
实审 C11 三方各算头宽：模型那份漏了预留位 29	抓到	core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/010.log）
实审 C11 三方各算头宽：共享标量 key 区间的 key 数写成 3（三份一起歪，连起来比看不出，靠钉绝对值）	抓到	each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/011.log）
实审 C11 三方各算头宽：共享标量 key 区间之外的头宽写成 87（三份一起歪，连起来比看不出，靠钉绝对值）	抓到	each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width 红了（日志 /tmp/claude-1000/impl-rev-c11/prove-logs/harness/012.log）
✓ 点名 12 条：跑了 12 条，跳过 0 条，跑的都抓到了
exit=0
```
「同时红了哪些」：用 `--lib -- unit::tests` 过滤跑的那两行，`core/001.log` 里红两条（`empty_tree_table_…` 与 `the_three_parsers_round_trip_…`），`core/002.log` 里只红点名的那条。另挑 4 行跑了新测试的整个二进制（不带过滤），每行只红表里写的那一条，另一条绿：
```
row 6 cargo exit=101
row 10 cargo exit=101
row 12 cargo exit=101
row 13 cargo exit=101
whole-binary/row-6.log
test each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width ... ok
test core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte ... FAILED
whole-binary/row-10.log
test each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width ... ok
test core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte ... FAILED
whole-binary/row-12.log
test core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte ... ok
test each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width ... FAILED
whole-binary/row-13.log
test core_checker_and_model_compute_the_same_index_node_header_width_for_every_key_width_byte ... ok
test each_tree_registered_index_node_header_width_equals_all_three_formulas_at_that_tree_key_width ... FAILED
```
（row 6、10、12、13 依次是上表的 checker 只在 k ≥ 128 时歪、模型只在 k = 0 时歪、key 数写成 3、写成 87。跑之前副本四份源码的 sha256 在 `whole-binary-pre.sha256`，跑完 `sha256sum -c` 全 OK。）

基线红集：`prove-red.sh` 每组参数都先在不改源码的副本上跑一次，都没红（红了它会退 2）。新测试不在任何名字带 layer0 的二进制里，不是崩溃枚举用例，不调 `enumerate_layer0`。

## 验证输出（第 4 步那几样；都在仓副本上跑，线程上限 5，内存上限没给、按 `replay.sh` 的默认 8G）

动到的测试二进制（副本换回终版之后，各跑整个二进制，末行原样）：
```
singlefs-format --lib exit=0
singlefs-core --lib exit=0
singlefs-checker --lib exit=0
singlefs-harness --lib exit=0
singlefs-harness --test index_node_header_width_computed_three_ways_agrees_for_every_key_width exit=0
singlefs-checker_--lib.log: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
singlefs-core_--lib.log: test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
singlefs-format_--lib.log: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
singlefs-harness_--lib.log: test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 98.35s
singlefs-harness_--test_index_node_header_width_computed_three_ways_agrees_for_every_key_width.log: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
`cargo fmt --check`（整个工作区）退 1。有差异的只有别人的文件：`crates/singlefs-core/src/allocator.rs`（A2c 在改）、`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-harness/tests/core_review_unit_area_start_and_publish_limits.rs`。对我的五份单跑 `rustfmt --edition 2021 --check`，退 0。
`cargo clippy`（`check.sh` 那一套 lint，整个工作区 `--all-targets --all-features`）退 101，末行原样：`error: could not compile \`singlefs-core\` (lib test) due to 5 previous errors`。红的地方都不在我的文件里：
- `crates/singlefs-core/src/admission.rs:670`、`:671`（manual_div_ceil）；
- `admission.rs:1107`、`:1142`（doc_lazy_continuation）；
- `crates/singlefs-core/src/allocator.rs:216`（manual_is_multiple_of）；
- 放行这三条之后，接着红在 `tests/core_review_unit_area_start_and_publish_limits.rs:256`、`:321` 和 `src/bin/e156_allocation_basis_counts.rs`、`src/bin/e158_root_choice_repair.rs` 的 shadow_unrelated。

所以改成只查动到的目标，加上那三条 `-A` 才编得过 core。结果：`-p singlefs-format -p singlefs-checker -p singlefs-core --all-targets` 退 0；`-p singlefs-harness --lib --test <新测试> --test first_transaction_step_five_publish --test second_transaction_supplement_two_accounting_node_full` 退 0。
`cargo build --offline --all-targets`：`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 37.54s`，退 0。

登记给实现员的门禁阶段（`stage-owners.tsv` 里 7 道；带根参数的，根指到副本；33 号没有根参数，在副本里跑）：
- 27 号，退 0：`  ✓ 格式常量同步（41 个已登记，41 个在源码里被钉住）`
- 53 号，退 0：`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
- 94 号，退 0。绿行原样：`  ✓ checker 与实现只共享常量模块 \`singlefs-format\`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 \`singlefs_core\`（别名引进来的 0 个）；共享模块 1 份源码的正文 285 行里没有分支与循环（\`#[cfg(test)]\` 标着的项 271 行不扫）`
- 93 号，退 0：`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））`
- 92 号，退 77，本次没跑成：`  ! /tmp/claude-1000/impl-rev-c11/repo 不是 git 仓，本阶段跳过`
- 89 号，退 77，本次没跑成。首行：`  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`；末行原样：`    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`
- 33 号，退 1。锚点红在第 430、436、974、975、977、978 行，都是 `admission.rs`、`allocator.rs` 里的锚点，我追加的第 1031–1044 行不在里面。之后在主工作区单跑 33 号，只剩第 430、436 两行红（A4b 那几行已经被修好）。
- 74 号，退 1。`second_transaction_supplement_three_random_history` 红两条：`crash_recovery_abandoning_the_newest_root_…_closeout_row_43`（I-7.4 被抛弃根，「NewFinding」）与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`（「新发现 28」）。把我的四份换回原件、删掉新测试后跑基线，**同样红这两条**（`gate-74-baseline.log`，末行 `== 74 baseline exit=1`），所以不是这次改动引起的，照写不修。

## 门禁 94 号该加的一项（我不改门禁，交主 agent 派工具线）

现在的 ③ 只查「format 正文里没有分支与循环」，允许「纯算术」，所以删掉的那个 `const fn` 过得了 ③。建议加两条：

- **④ format 正文不许有函数**：`crates/singlefs-format/src/**/*.rs` 里除了 `#[cfg(test)]` 标着的那一项，正文不许出现 `fn` 项（带不带 `const`、`pub` 都算），只许 `pub const` / `pub static` 标量与数组。
  - 红样本：把这次删掉的 `pub const fn index_node_header_bytes` 加回去；
  - 绿样本：测试模块里的私有 `fn index_node_header_bytes`（`#[cfg(test)]` 项，不扫）；
  - 今天会红在 `journal_in_flight_record_limit`（format `lib.rs:218`）。要么等 A2b 交回另派、把它搬走之后再上这一条，要么先把它登记成一个带到期条件的例外（点名欠账与 A2b）。
  - `pub const` 的初始化式里带算术的那 8 个（见上文「同类 const fn」一节）要不要也算，取决于下一节第 1 条怎么定。
- **⑤ 三方各算的式子要有交叉断言**：登记一张表（门禁文件里或 `.claude/gate.d/` 下一份 tsv），一行一个式子：式子名、core 路径、checker 路径、模型路径、交叉断言测试文件。今天只有一行：`index_node_header_bytes`、`crates/singlefs-core/src/unit.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/tests/index_node_header_width_computed_three_ways_agrees_for_every_key_width.rs`。逐行判：
  - 三个文件各有一处 `fn <式子名>`；
  - `model.rs` 里不出现 `singlefs_core::`、`singlefs_checker::`（core 与 checker 之间已经由 ①② 从依赖上挡住）；
  - 交叉断言测试文件在，并且三条路径 `singlefs_core::…::<式子名>`、`singlefs_checker::<式子名>`、`singlefs_harness::model::<式子名>` 都出现；
  - `crates/mutations.tsv` 里至少各有一行变异改 core、checker、模型那一份，并点名这个测试文件里的测试。
  - 「扫遍取值域」静态判不了，这一半靠这次那两行只在取值域一端变歪的变异，由 59 号复跑来护。

## 设计问题（停下交主 agent）

1. **format 里 `pub const` 的初始化式带算术，算不算 D13 已定项 5 不许的「算术」**。D13 已定项 5 原文是「判据是发射物里有没有分支与算术」，门禁 94 号 ③ 写的却是「它只许发射标量与纯算术」，两处口径不一。按原文，上文那 8 个都得改写成字面量（门禁 27 号绑值，就像 `DATA_POINTER_BYTES` 那样，format 单测里留原来的算式）；按 94 号 ③，可以留着。这件活只管 `const fn`，这 8 个我没动。
2. **`journal_in_flight_record_limit` 怎么拆**：今天只有 core 一方在用，另有一个实验装置（e158）在用。等 A2b 交回另派时要定两件事：式子只搬进 core，还是 checker 或模型也要各算一份；以及 e158 装置那一处怎么办。
3. **新标量没有 kb 落点**：`INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2`，以及原有的 `INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86`、`NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29`，在 kb 里都没有 `format-const` 标记（`grep -rn 'format-const: INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE\|format-const: NONCE_MAC' .claude/kb` 零命中），门禁 27 号绑不住它们的值。今天能兜住的是新测试钉的 6 个绝对值，以及 format 单测。要不要让书记员在 `.claude/kb/layout/01-first-txn.md` 补标记，由主 agent 定；我不写 kb。
4. **kb 里引着被删掉的那个函数**：`.claude/kb/experiments/142-第一个事务的干跑.md:191` 写着「`crates/singlefs-format/src/lib.rs:50-54` 与这个装置的 `index_node_header_bytes` 逐项核过」，`.claude/kb/experiments/157-并行线一两条条款的计数模型.md:12` 写着 `index_node_header_bytes(24)=163`。这两处是当时跑的记录，要不要加注由主 agent 或书记员定。D13 已定项 5 的「实现与它的差距」一段，也可以补一句「码 2 头宽改为三方各算、交叉断言护」。
5. **三份用同一个名字**：我照「一个语义概念全仓一个名字」让三份都叫 `index_node_header_bytes`，靠 crate 路径区分；format 单测里那份私有的也叫这个名字。主 agent 要是更想让名字带上是哪一方的（比如 `checker_…`），改起来是机械替换，变异行的原文也要跟着改。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7、crates 变异整表（59 号）、全量 `cargo test --all`，这些归提交时的 `crash-verifier` 与整轮门禁，都没跑。
- 没提交。补丁也没打进主工作区：规格要我在仓副本里做，补丁留给主 agent 用 `research/scripts/apply-writer-patch.py` 打。
- `journal_in_flight_record_limit` 没改（A2b 在改它的调用方）；format 里那 8 个带算术的 `pub const` 没改（不在这件活里，见「设计问题」第 1 条）。
- 没改门禁 94 号（归工具线），没写 kb。
- 整个工作区的 clippy 与 fmt 都红在别人的文件上，没修；只证了我动到的目标干净。
- 92 号、89 号退 77，本次没判；74 号红，但基线同样红，照写。
- 草稿收尾：删了仓副本 `/tmp/claude-1000/impl-rev-c11/repo`（15G，几乎都是它的 target），也删了从 HEAD 拷出来的 `capped-head.sh`。没删、留给主 agent 核的，都在 `/tmp/claude-1000/impl-rev-c11/` 下，合计 1.6M：补丁目录 `patch/`，原件 `orig/`，终版 `final/`（sha256 记在 `final.sha256`），各次日志，证红日志 `prove-logs/`，整二进制变异跑的 `whole-binary/`，`progress.md`。
- 这一轮用过的 capped.sh：主工作区的 `research/scripts/capped.sh` 22:52Z 前后有 bash 语法错（别的会话插坏了）。副本的第一次 build、五个二进制首跑、prove-red 这三批，是用 HEAD 版拷到草稿目录的那份设线程上限跑的；收到主 agent 消息说修好之后，改回用仓里那份。
