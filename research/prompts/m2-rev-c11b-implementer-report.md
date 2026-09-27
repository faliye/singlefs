# 实审 C11b 交回：format 里带算术的 8 个 `pub const` 改字面量；`journal_in_flight_record_limit` 搬出 format，core 与 E158 各算一份 + 交叉断言

时刻：2026-09-26 23:20 UTC 前后开工（JST 2026-09-27 08:20；进度记录 `progress.md` 第一行 23:22Z）。交法：**交补丁**。改动都在仓副本 `/tmp/claude-1000/impl-rev-c11b/repo` 里，主工作区一个字没碰；补丁目录 `/tmp/claude-1000/impl-rev-c11b/patch/`。
第一次取副本之后收到主 agent 消息（A2c 已打进主工作区、`transaction.rs` 加进清单、底座换成新的主工作区）：删了第一份副本，按当时的主工作区重取（主表 1093 行），之后全部在新副本上做。

## 结论

1. **format 只剩标量，没有函数。** `crates/singlefs-format/src/lib.rs` 里 8 个初始化式带算术的 `pub const` 改成整数字面量，值不变；`pub const fn journal_in_flight_record_limit` 删掉。
   - 8 个名字、值与终版行号（副本终版）：`DATA_UNIT_PAYLOAD_OFFSET` 134（:38）、`PACKED_UNIT_RECORDS_OFFSET` 136（:44）、`JOURNAL_NEW_ROOT_SEGMENT_BYTES` 188（:197）、`JOURNAL_NAMED_ENTRY_BYTES` 56（:203）、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 67（:206）、`JOURNAL_RING_DEFAULT_BYTES` 805_306_368（:213）、`ROOT_RING_CHUNK_BYTES` 1_048_576（:260）、`TEST_IMAGE_DEFAULT_BYTES` 4_294_967_296（:285）。
   - 每个的文档注释末尾加了 `format-const: 名字`（照 `DATA_POINTER_BYTES` 的写法），新老三个标量 `NONCE_MAC_ALGORITHM_RESERVED_BYTES`（:32）、`INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE`（:47）、`INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT`（:56）也加了。kb 那一侧 11 个名字**一个标记都没有**，要加的行见下文「kb 要加的 format-const 标记」一节。
   - 原来的 8 条算式原样留进 format 单测新加的 `arithmetic_initializer_literals_equal_the_expressions_they_replaced`（:468），与字面量逐个 `assert_eq!`。
   - 模块头注释加了两行：这里只放标量、不写算式不放函数、算式留在单测里（D13（验证路线） 已定项 5「判据是发射物里有没有分支与算术」）。
2. **`journal_in_flight_record_limit` 两方各算。**
   - core：`crates/singlefs-core/src/system_configuration.rs:73` 新加 `pub fn journal_in_flight_record_limit(journal_ring_bytes: u64) -> u64`，式子 `环长 ÷ JOURNAL_RECORD_BYTES ÷ JOURNAL_SAFETY_FACTOR`。core 的四个调用方都改成 `use crate::system_configuration::journal_in_flight_record_limit`，调用那一行一个字没改：`system_configuration.rs:380`、`make_filesystem.rs:277`、`recovery.rs:2148`、`transaction.rs:5110`（`transaction.rs` 只改了 import 两处，照主 agent 消息）。所以主表第 1028、1029、1075 行（锚在这几行调用上）原样有效，33 号在副本上核过。
   - E158 装置：`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:8041` 自写一份私有 `fn journal_in_flight_record_limit`（先算环槽数、再除 F，只取 format 的 `JOURNAL_RECORD_BYTES` 与 `JOURNAL_SAFETY_FACTOR` 两个标量），`device_replay` 里那一处（:8217）改调它。E158 别的地方没动。
   - checker：不算在飞上限，不另造（`grep -rn '在飞\|in_flight\|SAFETY_FACTOR' crates/singlefs-checker/src` 只命中 `walk.rs:4283` 一行文档注释里的「在飞垃圾」，与在飞上限无关）。`crates/singlefs-checker/src/lib.rs` 没有常量改名，一个字没动。
   - 理想模型 `model.rs` 不用在飞上限（`grep -n '在飞\|in_flight\|SAFETY_FACTOR' crates/singlefs-harness/src/model.rs` 零命中），不在清单里，没加。
3. **交叉断言放在 E158 这个 bin 自己的测试模块里，不在 `tests/` 下新开文件**（派发清单里那份 `crates/singlefs-harness/tests/journal_in_flight_record_limit_computed_independently_agrees_for_every_ring_size.rs` 没建）。理由：`tests/` 下的集成测试链接不到 bin 目标里的函数，装置那份私有 `fn` 只有同一个 bin 的 `#[cfg(test)]` 模块看得见；用 `#[path]` 把 1.3 万行的装置整个编进一份新测试二进制，会连它自己那 68 条单测一起在新二进制里再跑一遍。模块 `journal_in_flight_record_limit_cross_check_tests`（:8056）紧挨着装置那份 `fn`，两条：
   - `core_and_apparatus_compute_the_same_in_flight_record_limit_at_every_enumerated_ring_size`（:8113）：环长 0 到 2^20 逐个比，再比每个 2 的幂附近、每个数量级上 12288 的倍数附近的 4096 边界、u64 顶端、在飞上限越过 u32 的那一处、默认 768 MiB；另断言 ① 那一段恰好比了 2^20 + 1 个、u64::MAX / 2^60 / 在飞上限跨 u32 那一处 / 默认环四个点都在枚举集合里。
   - `the_in_flight_record_limit_is_pinned_at_values_worked_out_from_the_clauses`（:8151）：7 个按条款手算的绝对值（默认环 → 65536，12287 → 0，12288 → 1，2^60 → 93824992236885，在飞上限跨 u32 的两边，u64::MAX → 1501199875790165），两份各比一次。「改共享标量、两份一起歪」靠它抓。
4. **取值域与上界从哪来**（也写在那个测试模块的文档注释里）：环长是系统配置 8 字节的「环长度（字节）」字段，恢复照读盘上的值，所以两份要在整个 u64 上相等；按条款 mkfs 收的环长在 [3 × 4096, 设备容量 ÷ 4]（D23（journal 的角色与格式） 已定项 19 ③），容量被 6 字节槽号 × 16 KiB 槽（D19（块指针的结构与宽度预算） 已定项 4）顶在 2^62，所以不过 2^60。2^64 个值扫不遍，只取枚举点；① 那一段把模 12288（两份式子里仅有的除数 4096 × 3）的每个余数走了 85 遍，②–⑤ 罩每个数量级与两端。**罩不住的**：只在某个不落在枚举点上的大环长处才歪的写法。
5. 变异：追加 13 行（`patch/mutations-append.tsv`），13 行逐条用 `prove-red.sh` 在副本上证过，**13 行都抓到**；另外 3 行临时变异按整个测试二进制跑过一遍，看同一个二进制里还红了谁（见下文「每条新测试的证红」）。
6. 门禁（副本上跑）：27、53、94 绿；登记给我的另几道 33、93 绿，92、89 退 77（本次未跑），74 见下文「验证输出」。27 号绿是因为 11 个名字 kb 里都还没登记——27 号只绑 kb 里登记了的名字，Rust 注释里的 `format-const:` 它不看，所以 kb 没加标记时它不红，也就没在绑它们。

什么现象会推翻这些结论：
- 补丁打进主工作区后，format `--lib`、core `--lib`、E158 bin 的测试，或 `core_review_tree_table_duplicates_and_slot_one_search`、`core_review_unit_area_start_and_publish_limits` 两个测试二进制红了；
- `grep -n 'const fn\|pub fn\|^fn ' crates/singlefs-format/src/lib.rs` 在 `#[cfg(test)]` 之前有命中，或 format 里有 `pub const` 的初始化式带 `+ - * /`；
- core 与装置两份在某个环长上不等，而交叉断言没红；
- 书记员照下文加了 kb 标记之后，27 号红在 format 这 11 个名字上（值与我写的字面量不等）。

## 这一轮写过的文件

补丁里的（主工作区还没打；`git apply --check` 对主工作区此刻的状态能过）：
- `crates/singlefs-format/src/lib.rs`
- `crates/singlefs-core/src/system_configuration.rs`
- `crates/singlefs-core/src/make_filesystem.rs`（只改 import）
- `crates/singlefs-core/src/recovery.rs`（只改 import）
- `crates/singlefs-core/src/transaction.rs`（只改 import，照主 agent 消息）
- `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（加一个私有 `fn`、一个 `#[cfg(test)]` 模块，改 `device_replay` 里调用的那一行）
- `crates/mutations.tsv`：没直接改，要追加的 13 行放在 `patch/mutations-append.tsv`，变异名见下文「变异」一节。
- 清单里没动的两份：`crates/singlefs-checker/src/lib.rs`（没有常量改名）、`crates/singlefs-harness/tests/journal_in_flight_record_limit_computed_independently_agrees_for_every_ring_size.rs`（没建，理由见「结论」第 3 条）。

取副本时这八份（上面六份、checker `lib.rs`、`crates/mutations.tsv`）的 sha256 记在 `/tmp/claude-1000/impl-rev-c11b/copy-sha256.txt`，交回前在主工作区逐个重算，**八份都与取副本时相同**；取副本时别的会话在改的文件（A2c、B3c-2、编号落定那几份）的 sha256 也记在同一份文件里。第一次取、后来丢掉的那份副本的 sha256 在 `copy-sha256-first-take-discarded.txt`。

`git apply --numstat patch/crates.patch`（原样）：
```
70	27	crates/singlefs-format/src/lib.rs
7	7	crates/singlefs-core/src/make_filesystem.rs
6	5	crates/singlefs-core/src/recovery.rs
16	4	crates/singlefs-core/src/system_configuration.rs
5	5	crates/singlefs-core/src/transaction.rs
151	2	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
```
主工作区 `git diff --stat -- crates litmus` 的末行（原样）是 `94 files changed, 30874 insertions(+), 11385 deletions(-)`，全是别的会话还没提交的改动，这一轮一行都不是我的（我没碰主工作区），所以不贴全文。

终版六份的 sha256 在 `/tmp/claude-1000/impl-rev-c11b/final.sha256`（终版文件本身在 `final/`，原件在 `orig/`）。

## 受影响的层 0 流与崩溃枚举用例

**没有。** checker（`crates/singlefs-checker/src/`）一个字没动。core 那边在飞上限的值在任何环长上都与改前逐个相等（式子原样从 format 搬到 core，四个调用方的调用行不变），format 那 8 个常量的值也不变，所以 mkfs 写出的字节、恢复取的前缀、一次发布的记录条数上限都与改前相同，层 0 各流与崩溃枚举用例的钉值不会变。

## 同类 const fn 与 `pub const` 算术，找齐了没有

- `grep -n 'pub const fn\|const fn' crates/singlefs-format/src/lib.rs` 在副本终版上**零命中**（退出码 1）。format 单测模块里 C11 留下的私有 `fn index_node_header_bytes` 不是 `const fn`、在 `#[cfg(test)]` 里，不算。
- 按 `#[cfg(test)]` 之前的正文找 `fn`：`awk '/^#\[cfg\(test\)\]/{exit} /\bfn\b/{print NR": "$0}' crates/singlefs-format/src/lib.rs` 零输出。
- 初始化式带算术的 `pub const`：`grep -n '^pub const' crates/singlefs-format/src/lib.rs | grep -E '[-+*/]'` 只剩第 77 行 `LOC_ENTRY = 14`，命中的是它行尾的 `// naming-lint:external` 注释里的 `//`，值本身是字面量；`grep -n '^pub const' … | grep -v ';'` 零输出（没有跨行的初始化式）。
- `crates/singlefs-format/src` 下只有 `lib.rs` 一份源码（94 号报「共享模块 1 份源码」）。

## kb 要加的 format-const 标记（我不写 kb，交主 agent 派书记员）

`grep -rn "format-const: <名字>\b" .claude/kb` 对下表 11 个名字逐个跑，**全部零命中**。每行给出标记原文、建议落在哪一行（主工作区现查的行号）、值的出处分项。标记文法照 `.claude/gate.d/lib-format-const.py` 文件头：`<!-- format-const: 名字 = 整数 -->`，不带 stale（没有要清的旧字面串）。

| 名字 | 要加的标记 | 建议落点（现查行号） | 出处分项 |
|---|---|---|---|
| `NONCE_MAC_ALGORITHM_RESERVED_BYTES` | `<!-- format-const: NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29 -->` | `.claude/kb/layout/01-first-txn.md:186`（预留位那一行，宽写着 `12 + 16 + 1`） | D18（块里携带什么信息） 已定项 14 / 已定项 16；算法类型 1 字节 2026-09-14 用户定案 |
| `DATA_UNIT_PAYLOAD_OFFSET` | `<!-- format-const: DATA_UNIT_PAYLOAD_OFFSET = 134 -->` | `.claude/kb/layout/01-first-txn.md:190`（「含 29 字节预留位的头是 **134**，载荷从 134 起」） | D18（块里携带什么信息） 已定项 16 |
| `PACKED_UNIT_RECORDS_OFFSET` | `<!-- format-const: PACKED_UNIT_RECORDS_OFFSET = 136 -->` | `.claude/kb/layout/01-first-txn.md:377`（实例表单元那一行「含预留位的头 136，记录区从 136 起」） | D18（块里携带什么信息） 已定项 11 / 已定项 16 |
| `INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE` | `<!-- format-const: INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86 -->` | `.claude/kb/decisions/18-块里携带什么信息.md:446`（码 2 偏移表那一行，末尾「明文头 86 + 2k」） | D18（块里携带什么信息） 已定项 18；D8（核心索引结构） 已定项 11 |
| `INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT` | `<!-- format-const: INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2 -->` | 同一行（「52 key 区间 2k」） | D18（块里携带什么信息） 已定项 18；D18 已定项 2（全部码 2 节点带 key 区间） |
| `JOURNAL_NEW_ROOT_SEGMENT_BYTES` | `<!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->` | `.claude/kb/decisions/23-journal的角色与格式.md:435`（已定项 15 定案「… = 188 字节」）；或 `.claude/kb/layout/01-first-txn.md:319` | D23（journal 的角色与格式） 已定项 15 |
| `JOURNAL_NAMED_ENTRY_BYTES` | `<!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->` | `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 17（标题在第 462 行「点名项 56 字节的字段表」，定案句在第 464 行） | D23（journal 的角色与格式） 已定项 17 |
| `JOURNAL_NAMED_ENTRIES_PER_RECORD` | `<!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 67 -->` | `.claude/kb/decisions/23-journal的角色与格式.md:315`（已定项 12 那一节「4 KiB 记录装 67 个点名项（已定项 4）」） | D23（journal 的角色与格式） 已定项 4 / 已定项 12 / 已定项 17 |
| `JOURNAL_RING_DEFAULT_BYTES` | `<!-- format-const: JOURNAL_RING_DEFAULT_BYTES = 805306368 -->` | `.claude/kb/layout/01-first-txn.md:132`（系统配置字段表「journal 环长度（字节） \| 8 \| 805 306 368」） | D23（journal 的角色与格式） 已定项 19 ③（默认 768 MiB） |
| `ROOT_RING_CHUNK_BYTES` | `<!-- format-const: ROOT_RING_CHUNK_BYTES = 1048576 -->` | `.claude/kb/layout/01-first-txn.md:140`（「根环 chunk \| 4 \| 1 048 576」） | D22（单元原子性怎么合成） 已定项 2 |
| `TEST_IMAGE_DEFAULT_BYTES` | `<!-- format-const: TEST_IMAGE_DEFAULT_BYTES = 4294967296 -->` | **没有条款落点**，只有 `.claude/kb/layout/01-first-txn.md:427`「仍标预想的取值」那一段写着 4 GiB | 无分项；欠账 C323（镜像大小全仓没有条款）。见「设计问题」第 2 条 |

书记员落位时要一起看的两件事：
1. **加标记之后 27 号会连带核仓里同名的别的 const**（它扫 `research/**/*.rs` 与 `crates/**/*.rs`，同名、类型是单个标识符的 const 值必须是等于登记值的整数字面量）。主工作区现查（`grep -rnE 'const (…11 个名字…)\b' research crates --include=*.rs`，去掉 format 自己）：
   - `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:106` `const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 2 * NODE_POINTER_BYTES + 8 + 8;` ——值是算式，登记 `JOURNAL_NEW_ROOT_SEGMENT_BYTES` 之后 27 号当场红（「值写成了…门禁读不出它等于几」）；
   - `research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:74` `const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = (JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES;` ——同上，登记 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 之后红；
   - 其余同名的都是等于登记值的字面量，不红：e157 `:71`、e142 `:108`、e146 `:39` 的 `JOURNAL_NAMED_ENTRY_BYTES = 56`，e142 `:122` 的 `INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86`，harness 两份测试 `checker_narrow_invariants_and_abandoned_roots.rs:113`、`checker_known_bad_images.rs:4430` 的 `JOURNAL_NAMED_ENTRY_BYTES: usize = 56`。
   这两处在 `research/` 下，我写不了；登不登记那两个名字、要不要改实验源码并重跑，归主 agent 定（见「设计问题」第 3 条）。
2. 标记落进决策正文（D18 已定项 18、D23 已定项 15 / 12 / 17）时，39 号「一条已定项里恰好一个 format-const 标记，值要等于该分项下字段表的合计」那一条会开始核它们；D18 第 446 行一行放两个标记时，那一格按 39 号的判据怎么走我没验。落位之后跑一次 27、39 号。

## 变异（追加 13 行，`patch/mutations-append.tsv`，六段；和主表比过，名字都不重）

13 行逐条用 `bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-c11b/repo --memory 8G <crate> <名…>` 在副本上证过（format 9 行、harness 4 行），**13 行都抓到**，没有留给 59 号「只追加不证」的行。

| # | 变异名 | 改哪一处 | 必须红的测试 | 证红结局（改坏 → 哪条断言红） |
|---|---|---|---|---|
| 1 | 实审 C11b 算式改字面量：码 1 载荷起点写成 135（…） | format `DATA_UNIT_PAYLOAD_OFFSET` 134 → 135 | `arithmetic_initializer_literals_equal_the_expressions_they_replaced` | 抓到；`lib.rs:469` 的 `assert_eq!` left 135 / right 134 |
| 2 | …码 3 记录区起点写成 137（…） | `PACKED_UNIT_RECORDS_OFFSET` 136 → 137 | 同上 | 抓到；`:474` left 137 / right 136 |
| 3 | …journal 新根段写成 189（…） | `JOURNAL_NEW_ROOT_SEGMENT_BYTES` 188 → 189 | 同上 | 抓到；`:479` left 189 / right 188 |
| 4 | …journal 点名项写成 57（…） | `JOURNAL_NAMED_ENTRY_BYTES` 56 → 57 | 同上 | 抓到；`:484` left 57 / right 56 |
| 5 | …一条记录装的点名项写成 66（…） | `JOURNAL_NAMED_ENTRIES_PER_RECORD` 67 → 66 | 同上 | 抓到；`:489` left 66 / right 67 |
| 6 | …journal 默认环长多写 1 字节（单元区起点那条 ÷ 16384 的断言看不出…） | `JOURNAL_RING_DEFAULT_BYTES` 805_306_368 → 805_306_369 | 同上 | 抓到；`:494` left 805306369 / right 805306368 |
| 7 | …根环 chunk 多写 1 字节（…） | `ROOT_RING_CHUNK_BYTES` 1_048_576 → 1_048_577 | 同上 | 抓到；`:499` left 1048577 / right 1048576 |
| 8 | …测试镜像默认大小多写 1 字节（…） | `TEST_IMAGE_DEFAULT_BYTES` 4_294_967_296 → 4_294_967_297 | 同上 | 抓到；`:500` left 4294967297 / right 4294967296 |
| 9 | 实审 C11b 补 format-const 的共享标量：预留位写成 30（…） | `NONCE_MAC_ALGORITHM_RESERVED_BYTES` 29 → 30 | 同上 | 抓到；`:469` left 134 / right 135（码 1 那条先红） |
| 10 | 实审 C11b 在飞上限两份各算：core 那份把 F 多算 1（…） | core `system_configuration.rs` 式子 `/ JOURNAL_SAFETY_FACTOR` → `/ (JOURNAL_SAFETY_FACTOR + 1)` | `core_and_apparatus_compute_the_same_in_flight_record_limit_at_every_enumerated_ring_size` | 抓到；`e158…rs:8116` 「环长 12288 字节：core 与装置的在飞上限不等」left 0 / right 1；同一次钉绝对值那条也红（`:8174` left 49152 / right 65536） |
| 11 | …core 那份只看环长低 32 位（2^32 以下逐个扫看不出，只有每个数量级的枚举点抓得到） | core 式子里环长先 `& u64::from(u32::MAX)` | 同上 | 抓到；`:8130`（枚举点那一段）「环长 4294967296 字节…」left 0 / right 349525——① 逐个扫的那一段没红，是 ② 抓的；钉绝对值那条同时红（2^60 那一格 left 0 / right 93824992236885） |
| 12 | …装置那份环槽数少算一格（环长恰是 12288 的倍数时少 1） | E158 `let ring_record_slots = ring_bytes / JOURNAL_RECORD_BYTES;` → `ring_bytes.saturating_sub(1) / …` | 同上 | 抓到；`:8116` 「环长 12288 字节…」left 1 / right 0；钉绝对值那条同时红（`:8179`，装置那一句，left 65535 / right 65536） |
| 13 | …共享标量 F 写成 4（两份一起歪，互比看不出，靠钉绝对值） | format `JOURNAL_SAFETY_FACTOR` 3 → 4 | `the_in_flight_record_limit_is_pinned_at_values_worked_out_from_the_clauses` | 抓到；`:8174` left 49152 / right 65536；互比那条**没红**（按过滤跑 1 passed / 1 failed），正是要它说明的那一格 |

prove-red 的原样判定行在 `/tmp/claude-1000/impl-rev-c11b/prove-red-run-1.log`（13 行首跑；format 那 9 行的日志文件 001–009 被 harness 那一组同名日志覆盖了前 4 个，所以另把 format 9 行换目录重跑一遍，判定行在 `prove-red-run-2.log`、日志在 `prove-logs-format/`，9/9 抓到）。每组参数 prove-red 先跑一次不改的基线，两组基线都绿（`prove-logs-format/baseline.log`、`prove-logs-first-pass/baseline.log`）。

**已有的行**：锚在这次动过的文件上的主表行，33 号在副本上逐条核过原文各命中一次（下文「验证输出」）。锚在在飞上限调用上的第 1028、1029（`make_filesystem.rs`）与第 1075 行（`transaction.rs`）调用行原样没动，照旧有效；它们点名的两个测试二进制 `core_review_tree_table_duplicates_and_slot_one_search`（12 passed）、`core_review_unit_area_start_and_publish_limits`（20 passed）在终版上都绿。没有需要替换或删除的行。

## 每条新测试的证红（整个测试二进制）

先跑不改的副本：五个动到的测试二进制全绿（见下文「验证输出」），**基线红集是空的**。然后三行临时变异（不进补丁，跑完从副本表里删了）各按整个测试二进制跑一次，看同时红了谁：

| 新测试 | 挑的变异 | 整个二进制的结局（原样） | 同时红的 |
|---|---|---|---|
| format `arithmetic_initializer_literals_equal_the_expressions_they_replaced` | `JOURNAL_RING_DEFAULT_BYTES` 多 1 字节 | `test result: FAILED. 5 passed; 1 failed; …`（`prove-logs-whole-format/001.log`） | 只有它自己；`widths_match_the_first_transaction_byte_table` 里「单元区紧接 journal 环之后」那条按 ÷ 16384 算，看不出多 1 字节 |
| E158 `core_and_apparatus_…_at_every_enumerated_ring_size` | core 那份 F 多算 1 | `test result: FAILED. 68 passed; 2 failed; …`（`prove-logs-whole-e158/001.log`） | 钉绝对值那条（core 那一句先红） |
| E158 `the_in_flight_record_limit_is_pinned_…` | 共享标量 F 写成 4 | `test result: FAILED. 69 passed; 1 failed; …`（`prove-logs-whole-e158/002.log`） | 没有：互比那条照绿，E158 自己那 68 条单测也都绿 |

没有 `debug_assert` 夹在这几条路径上（被测的都是一行整数除法），没另跑 `--release`。

## 验证输出（第 4 步那几样；都在仓副本上跑，线程上限 5，内存上限没给、按 `replay.sh` 的默认 8G）

开跑前 `ps` 看负载：23:21 UTC 看到别的会话的 `cargo test`（层 0 分片自检、`checker_known_bad_images`）、`cargo clippy` 在跑，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；23:36 UTC 编译前再看，没有别的 cargo，也没有性能测量。收到主 agent 消息是在读完条款、正要动手改的时候（23:27Z），改了什么见文件头第二段。副本用它自己的 `target`，没有等锁。

**动到的测试二进制**（经 `run-with-memory-cap.sh 8G`，整个二进制、不按名字挑；末行原样）：
```
format-lib.log: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
core-lib.log: test result: ok. 127 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.01s
e158-bin.log: test result: ok. 70 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.58s
core-review-tree-table.log: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
core-review-unit-area.log: test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s
```
（E158 bin 70 条 = 原有 68 条 + 新加 2 条。）

**`cargo build --offline --all-targets`**：退出码 0，末行 `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 7.92s`；第一次全量编译 27.02s，0 条 warning。

**`cargo fmt --check`**：退出码 1，67 处 `Diff in`，**全部在 `e158_root_choice_repair.rs`、全部是原件就有的**：把原件单独拿去 `rustfmt --check` 也是 67 处；我加的那一段（终版第 8038–8187 行）一处都不在里面。我改的另五份 `rustfmt --check` 各自退 0。E158 那 67 处我没顺手格式化（规格要求 E158 只动在飞上限那一处）。

**`check.sh` 那套 clippy**（`-D warnings` 加 `code-discipline.md` 那七条，逐 crate `--no-deps --all-targets --all-features`）：
```
clippy singlefs-format exit=0
clippy singlefs-core exit=0
clippy singlefs-checker exit=0
clippy singlefs-harness exit=101
```
harness 红的是 E158 里 5 处 `shadow_unrelated`（`ranges`、`run`、`geometry`，终版行号 7244、7245、9950，测试编法下另两处 12973、13126）与 `tests/checker_narrow_invariants_and_abandoned_roots.rs:188`、`:618` 两处 `wildcard_enum_match_arm`。**全是原件就有的**：六份换回原件、只对 E158 这个 bin 跑，同样那 5 处（原件行号 7244、7245、9801、12824、12977，差的 149 行就是我插进去的那一段）：
```
clippy e158 bin (orig) exit=101
clippy e158 bin test-profile (orig) exit=101
clippy e158 bin (final) exit=101
clippy e158 bin test-profile (final) exit=101
```
两边 `error:` 行逐条相同，我加的那一段没有 clippy 告警。

**登记给我的门禁阶段**（`stage-owners.tsv` 里 implementation-writer 名下 7 道：33、53、74、92、94、93、89；规格另点 27）：
- 27 号，退 0：`  ✓ 格式常量同步（41 个已登记，41 个在源码里被钉住）`。这次那 11 个名字 kb 里都没登记，27 号只绑登记了的名字，所以它不红，也没在绑它们；「27 号因为 kb 还没加标记而红的名字」：没有。
- 53 号，退 0：`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
- 94 号，退 0：`  ✓ checker 与实现只共享常量模块 \`singlefs-format\`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 \`singlefs_core\`（别名引进来的 0 个）；共享模块 1 份源码的正文 286 行里没有分支与循环（\`#[cfg(test)]\` 标着的项 313 行不扫）`
- 33 号，退 0：`  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1101 条的原文各命中源码一次；…`（副本表 = 主表 1093 行 + 13 行，1101 是去掉表头注释之后的变异条数）
- 93 号，退 0：`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））`
- 92 号，退 77，本次没判：`  ! /tmp/claude-1000/impl-rev-c11b/repo 不是 git 仓，本阶段跳过`。打进主工作区之后它会看到 format 里 8 个常量的值文本变了（它按空白归一的原文比，算式改字面量算「改值」），要求 checker 判定路径在同一次改动里被碰过；主工作区里 checker 的 `lib.rs`、`walk.rs`、`image.rs` 都已有别的会话的改动，按它的判法会过，但那不是这次改动碰的，我没验。
- 89 号，退 77，本次没判。首行 `  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`；末行 `    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`（与这一件无关）。
- 74 号，退 1：`second_transaction_supplement_three_random_history` 红两条 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，末行 `test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 48.48s`。**六份换回原件跑基线，同样红这两条**（`gate-74-baseline-originals.log`，`test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 44.54s`，`== 74 baseline exit=1`），不是这次改动引起的，照写不修。

## 设计问题与要主 agent 定的事（停下交回）

1. **交叉断言没按派发清单放进 `tests/` 下的新文件，放在了 E158 bin 自己的 `#[cfg(test)]` 模块里。** 原因见「结论」第 3 条：`tests/` 下的集成测试链接不到 bin 目标的私有函数。这样做的代价：
   - E158 净多 149 行（`git apply --numstat` 记 +151 / −2：一个私有 `fn`、一个测试模块、调用处改成一行），超出「只改 `journal_in_flight_record_limit` 那一处」的字面；E158 的运行行为没变：装置那份与改前调的 format 那份是同一串运算（先 ÷ `JOURNAL_RECORD_BYTES` 再 ÷ `JOURNAL_SAFETY_FACTOR`，都取整数部分），`device_replay` 取到的上限在每个环长上都与改前相等。
   - 主 agent 要是更想让交叉断言住 `tests/`，出路是把装置那份搬进 harness 的库（例如 `model.rs`，C11 的模型那份就住那里），E158 与测试都调库里那份——那要动不在这次清单里的 harness 库文件，我没做。
   - 装置 `device_replay` 调的是不是它自己那份，交叉断言看不出来（改成调 core 那份，值一样、什么都不红，是等价变异）。这一格只能靠 C11 报告里建议的 94 号 ⑤ 那种静态检查（装置源码里不许出现 `singlefs_core::system_configuration::journal_in_flight_record_limit` 在测试模块之外）。
2. **`TEST_IMAGE_DEFAULT_BYTES` 没有条款落点。** 全仓只有欠账 C323（镜像大小全仓没有条款）与 `layout/01-first-txn.md:427` 那段「仍标预想的取值」。D13（验证路线） 已定项 5 写「生成器拒绝发射没有 kb 落点的常量」；它连同 `JOURNAL_RING_DEFAULT_BYTES`（mkfs 参数的默认值）都不是盘上格式承诺的量，要不要继续住 format、要不要登记 `format-const`，由主 agent 定。我照规格把两者都写成了字面量、注释里加了 `format-const:`，kb 行照给（上文表里最后一行标了「没有条款落点」）。
3. **登记 `JOURNAL_NEW_ROOT_SEGMENT_BYTES`、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 两个名字，27 号会红在 `research/` 下两份实验源码上**（`e142_first_transaction_dry_run.rs:106`、`e157_parallel_line_one_clauses.rs:74`，值是算式）。要么先不登记这两个名字，要么改那两份实验源码（写成字面量）并按 27 号的出路重跑产物；我写不了 `research/`。
4. **E158 第 4 次跑的跑前登记与装置对不上了。** `research/prompts/e158-r4-prereg.md:403` 写「重放的在飞上限调 `singlefs_format::journal_in_flight_record_limit`」，这次改成调装置自己那份（式子相同）；同一份登记第 703 行记着装置的 `wc -l`（13020）与 sha256，打上这个补丁之后是 13169 行、sha256 也变。第 4 次跑的执行员开跑前核这两处会对不上，要主 agent 在登记里记一笔修订（冻结证据不回改，按 `evidence-discipline.md`「跑产物之前修订臂或判据，是合法的，但要留下记录」那一条另写）。
5. **顺带看到、不在这一件里：环长 ≥ 2^32 × 12288 字节（约 48 TiB）时 mkfs 会 panic，而不是拒绝。** 在飞上限写进系统配置的是 4 字节（`layout/01-first-txn.md:134`「journal 在飞记录数上限 \| 4 \| 65536」），`system_configuration.rs:381` 用 `u32::try_from(in_flight_limit).expect("在飞上限 4 字节")`；`make_filesystem.rs` 的 `check_geometry`（:255 起）只拒「环 < 一条记录」「在飞上限 = 0」「环 > 最小盘 ÷ 4」「根环越界」，没有拒在飞上限装不进 4 字节的那一格。要一块 ≥ 192 TiB 的盘才走得到。按条款（D23（journal 的角色与格式） 已定项 19 ③ 与 6 字节槽号）mkfs 收得下的环到 2^60，所以这条分支按条款走得到；该拒还是该改字段宽，条款没写。我没改（不在这件活的文件与条款里），交叉断言里特意放了这一处的两个枚举点与两个钉值。

## 没做什么

- 没走三方对抗；层 0、QEMU、herd7、crates 变异整表（59 号）、全量 `cargo test`、`check.sh`、`gate.sh` 整轮都没跑，归提交时的 `crash-verifier` 与整轮门禁。
- 没提交，补丁没打进主工作区（规格要我在副本里做，交补丁由主 agent 用 `research/scripts/apply-writer-patch.py` 打）。
- 没写 kb（11 个 `format-const` 标记只给了原文与落点），没改门禁 94 号（C11 报告建议的 ④⑤ 归工具线），没碰 `research/`（上文第 3、4 条）。
- `crates/singlefs-checker/src/lib.rs` 与清单里那份新测试文件没动、没建（理由见上）。
- E158 里原有的 67 处 rustfmt 差异、5 处 `shadow_unrelated`，`checker_narrow_invariants_and_abandoned_roots.rs` 的 2 处 `wildcard_enum_match_arm`，都是原件就有的，没修。
- 74 号红的两条，原件基线同样红，没修。92、89 号退 77，本次没判。
- 草稿收尾：删了仓副本 `/tmp/claude-1000/impl-rev-c11b/repo`（`du -sh` 16G，几乎全是它自己的 `target`），以及试格式化用的两份 E158 草稿 `scratch-e158-rustfmt-full.rs`（596K）、`scratch-e158-block-formatted.rs`（8.0K）。第一次取、后来丢掉的那份副本在收到主 agent 消息时就删了（没编译过）。草稿目录现在合计 3.7M。留给主 agent 核的都在 `/tmp/claude-1000/impl-rev-c11b/` 下：补丁目录 `patch/`，原件 `orig/`，终版 `final/`（sha256 在 `final.sha256`），取副本时的 sha256 `copy-sha256.txt`，各次日志（`build-1.log`、`touched-logs-1/`、`lint-logs/`、`gate-*.log`、`gate-74-baseline-originals.log`），证红日志 `prove-red-run-1.log`、`prove-red-run-2.log`、`prove-logs-*/`，变异草稿 `mutations-append-draft.tsv`，跑用的几份小脚本，`progress.md`。
