# m3-formal-code-b17-r1 云端正推（Sonnet）报告（2026-10-04）

轮：代码轮 m3-formal-code-b17-r1 第一轮。批次：`records/2026-10-01-里程碑三形式化接入.md#17`。我的格：K1、K3、K4（正文第四节）。
立场：逐句核代码做的是不是条款说的。代码引文都读冻结副本 `/tmp/claude-1000/m3-formal-code-b17-r1/tree/`（快照清单核过：`sha256sum -c` 对
`research/prompts/m3-formal-code-b17-r1-snapshot/crates-sha256.txt` 在副本里没有一行不是 OK），报告里 `crates/…` 的行号都是那棵树里的行号。
方案按 `crates/` 今天的实现来谈；没读过禁读清单里的任何文件。

## 0　怎么量的（所有「我量的」都在草稿副本上，不在仓里）

- 副本：`/tmp/claude-1000/m3-formal-code-b17-r1-sonnet/ws/`（冻结树的 `crates/` + 仓根的 `Cargo.toml`、`Cargo.lock`、`.cargo/`）。
- harness 档测试都经 `research/scripts/run-with-memory-cap.sh` 与 `research/scripts/capped.sh 8` 跑；变异驱动 `research/prompts/m3-formal-code-b17-r1-sonnet-model/run-mutation-row.py`
  （在副本里按 crates 变异表的六段格式逐行改、跑、还原；还原后把文件修改时间置成现在，否则 cargo 沿用变异那次的编译产物——我第一遍没置，第 2 行之后的结果作废重跑，下面的数都是置过之后的）。
- 基线（副本、未变异）：`cargo test -p singlefs-harness` 四个目标——
  `checker_known_bad_images` 69 过、`pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement` 5 过、
  `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker` 8 过、`writable_mount_of_a_formatted_pool` 15 过（`research/prompts/m3-formal-code-b17-r1-sonnet-model/logs/base-all.log` 里的 `test result` 行，退出码 0）。
- 开跑前 `ps` 只看到别的会话的 cargo / 测试在跑，没有 qemu、fio、vm-bench、e152；包装拒过一次「同一会话同时只许一条」，设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 与
  `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`（理由写在驱动脚本里）后照跑，上限 4G。
- 没跑：checker 档任何测试二进制（GPU 对拍、`--lib` 的 naga 三条）；没碰仓里的任何文件，除报告与模型目录外。

## K1　I-9.27 的 CPU 与 GPU 判法对不对

### K1.1　条款逐句对代码（CPU：`crates/singlefs-checker/src/walk.rs`）

条款一行在 `.claude/kb/invariants.md`；D8 的句末关系判据在 `.claude/kb/decisions/08-核心索引结构.md`。

| 条款句 | 代码 | 判定 |
|---|---|---|
| 「走读到的任一版（候选集里每条根，含由记录施加出来的那几版）的树表有条目时」（`.claude/kb/invariants.md:314`） | 根环里的候选根走 `walk_root`、由记录施加出来的那几版走 `walk_version_applied_only_by_records`，两处都调 `walk_tree_table_and_central_mapping_root`（`crates/singlefs-checker/src/walk.rs:1337`、`crates/singlefs-checker/src/walk.rs:1370`），I-9.27 挂在它里面第一个走到树表那一版的分支（`crates/singlefs-checker/src/walk.rs:1451`） | 一致 |
| 「逐条判条目偏移 116 的头 ID」（`.claude/kb/invariants.md:314`） | 偏移常量 `const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;`（`crates/singlefs-checker/src/walk.rs:3369`）= 116，逐条读它 | 一致 |
| 「种类码 2（inode 树）的条目等于它自己的树 ID（条目偏移 0）」（`.claude/kb/invariants.md:314`） | 种类 2 映射到 `TREE_KIND_INODE => Self::ItsOwnTreeIdentifier,`（`crates/singlefs-checker/src/walk.rs:3392`），判 `HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => head_identifier == tree,`（`crates/singlefs-checker/src/walk.rs:3515`）；树 ID 取 `read_u64(entry, 0)` | 一致 |
| 「种类 1（extent 树）、9（deleted_inodes）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID」（`.claude/kb/invariants.md:314`） | `TREE_KIND_EXTENT \| TREE_KIND_DELETED_INODES \| TREE_KIND_TOMBSTONE => {`（`crates/singlefs-checker/src/walk.rs:3393`），判 `inode_tree_identifiers.contains(&head_identifier)`（`crates/singlefs-checker/src/walk.rs:3517`） | 一致 |
| 「表里有多于一条种类 2 条目时放宽成」等于其中某一条的树 ID | `contains` 对全部种类 2 条目的树 ID 取「某一条」，没有取第一条 | 一致 |
| 「表里没有种类 2 条目时这三种判红」（`.claude/kb/invariants.md:314`） | 表里没有种类 2 条目时 `inode_tree_identifiers` 为空，`contains` 恒假 ⇒ 种类 1 / 9 / 10 的每一条判红 | 一致 |
| 「种类 3、4、6、7、8、11、12 的条目等于 0；别的种类码不判。」（`.claude/kb/invariants.md:314`） | 七个码并成 `\| TREE_KIND_LOGGED_OPERATIONS => Self::Zero,`（`crates/singlefs-checker/src/walk.rs:3402`），判 `HeadIdentifierOfATreeTableEntry::Zero => head_identifier == 0,`（`crates/singlefs-checker/src/walk.rs:3519`）；`_ => Self::NotJudged,`（`crates/singlefs-checker/src/walk.rs:3403`）对应 `HeadIdentifierOfATreeTableEntry::NotJudged => continue,`（`crates/singlefs-checker/src/walk.rs:3520`）。码值我对了 `crates/singlefs-checker/src/walk.rs` 的 `TREE_KIND_*` 常量表（3、4、6、7、8 与 9..12，种类 5 中央映射不进树表、落在 `_`） | 一致 |
| 「树表 0 条的版本一条也不判；条目宽不到 124 字节（头 ID 那 8 字节读不全）的树表整张不判。」（`.claude/kb/invariants.md:314`） | 0 条时 `for` 循环不进，一格也不记，`into_report` 里没评估过、没不适用理由的条目报「不适用」；宽度这半句见 K1.2 | 一致 |
| 「不拿字面 65537 比」（`.claude/kb/invariants.md:314`） | `grep -nF 65537 crates/singlefs-checker/src/walk.rs` 只命中一行，是 `/// 句末的关系判据：判的是关系，不拿字面 65537 比）。`（`crates/singlefs-checker/src/walk.rs:3375`）；判据代码里没有字面 65537 | 一致 |
| D8 句末：「判据是关系、不是字面值：inode 树条目的头 ID 等于它自己的树 ID；extent 树、deleted_inodes、墓碑树条目的头 ID 等于同一张树表里 inode 树条目的树 ID；谱系树、logged_ops、分配记录树、记账树、livelist 共享树、稀疏旁表树、deadlist 树写 0」（`.claude/kb/decisions/08-核心索引结构.md:254`） | 同上三支。谱系 11、logged_ops 12、分配记录 3、记账 4、livelist 6、稀疏旁表 7、deadlist 8，与 `Zero` 那一支的七个码一一对上 | 一致 |

### K1.2　条款后来补的两句：宽度与「每张树表判一次」

我读到的 `.claude/kb/invariants.md` 第 314 行，开头那次 `grep -nF` 命中的版本没有这两句；写报告时再读，这一行已被改：条款现在含
「条目宽不到 124 字节（头 ID 那 8 字节读不全）的树表整张不判」，状态列含「共享这张树表的别的版本不重判」与「条目宽不到 124 字节时这张树表报射程里判不了」
（`.claude/kb/invariants.md:314`，工作区有未提交改动：`git diff --stat -- .claude/kb/invariants.md` 报 6 行改动）。下面按现在的条款对：

| 条款句 | 代码 | 判定 |
|---|---|---|
| 「条目宽不到 124 字节（头 ID 那 8 字节读不全）的树表整张不判」（`.claude/kb/invariants.md:314`） | `.all(\|entry\| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8);`（`crates/singlefs-checker/src/walk.rs:3494`，116 + 8 = 124）不成立就 `return`，并记 `part_of_the_range_not_judged`（理由串 `crates/singlefs-checker/src/walk.rs:3498`）。同一个节点里所有条目同宽：`index_node_view` 拒绝 `declared_length != entry_count * entry_width`（`crates/singlefs-checker/src/lib.rs:505`）、按 `.chunks(entry_width)` 切条（`crates/singlefs-checker/src/lib.rs:521`），所以 `all` 与「看第一条」等价，不会出现一张表里一部分条目判了、一部分没判 | 一致 |
| 「共享这张树表的别的版本不重判」（`.claude/kb/invariants.md:314`） | 调用点在 `walk_the_entries_of_a_tree_table_walked_first_by_this_version` 里，注释写明 `I-9.27 只看同一张表里条目之间的关系、与哪一版无关：共享这片树表的别的版本不重判（与 I-9.16 同）。`（`crates/singlefs-checker/src/walk.rs:1450`）；别的版本走 `None if tree_table_walked_by_an_earlier_version => {`（`crates/singlefs-checker/src/walk.rs:1412`）。同一落点在一份镜像里字节相同，判法只看表内条目，每版各判一遍的结果与判一遍相同；差别只在第一处违例的话写的是先走到那一版的名字 | 一致 |
| 条款没说、代码做了：条目宽不到 124 字节时整条在没有违例时报「不适用」带理由，而不是「成立」 | `part_of_the_range_not_judged` 的语义：没有违例时整条报不适用（`crates/singlefs-checker/src/image.rs:237`）；与 I-9.16 读不出种类那一格同一做法 | 规则没说（现在条款状态列写了「报射程里判不了」，对得上） |

### K1.3　GPU 与 CPU 是不是同判

GPU 判法在走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`，CPU 判法在上面那些行。逐项并排：

| 项 | CPU | GPU | 判定 |
|---|---|---|---|
| 偏移 116 | `8 + 2 + 2 + 2 + 86 + 8 + 8`（`crates/singlefs-checker/src/walk.rs:3369`） | `const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: u32 = 116u;`（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:155`） | 同 |
| 条目宽门槛 124 | `.all(\|entry\| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8);`（`crates/singlefs-checker/src/walk.rs:3494`） | `if (n.entry_width < TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8u) {`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1466`），后接 `return` | 同 |
| 种类字段偏移 10 | `const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 8 + 2;`（`crates/singlefs-checker/src/walk.rs:3366`） | `let kind = rd16(n.base, entry + 10u);`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1472`） | 同 |
| 种类 2 | `head_identifier == tree`（`crates/singlefs-checker/src/walk.rs:3515`） | `w_judge(INVARIANT_I_9_27, eq64(head, tree));`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1475`） | 同 |
| 种类 1 / 9 / 10 | `.contains(&head_identifier)`（`crates/singlefs-checker/src/walk.rs:3517`） | 种类判 `} else if (kind == TREE_KIND_EXTENT \|\| kind == TREE_KIND_DELETED_INODES \|\| kind == TREE_KIND_TOMBSTONE) {`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1476`），内层对表里每个种类 2 条目比树 ID，结果 `w_judge(INVARIANT_I_9_27, names_an_inode_tree_of_this_table);`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1484`） | 同（多条放宽、没有种类 2 就判红，两边都是「存在」量词） |
| 写 0 的七个种类 | 3、4、6、7、8、11、12（`crates/singlefs-checker/src/walk.rs:3402`） | `\|\| kind == TREE_KIND_LOGGED_OPERATIONS) {`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1487`）前一行是 SPARSE / DEADLIST / LINEAGE（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1486`），判 `w_judge(INVARIANT_I_9_27, is_zero64(head));`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1488`）；1485 行是 ALLOCATION / ACCOUNTING / LIVELIST | 同 |
| 种类码值 | `TREE_KIND_*` 1、2、3、4、6、7、8 与 9..12（`crates/singlefs-checker/src/walk.rs:61`–`crates/singlefs-checker/src/walk.rs:73`） | `const TREE_KIND_EXTENT: u32 = 1u;`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:108`）到 `const TREE_KIND_DEADLIST: u32 = 8u;`（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:114`）；9、10、11、12 在 `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:146`、`:148`、`:150`、`:152` | 同 |
| 违例位 | `IMPLEMENTED_INVARIANTS` 里 `HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,`（`crates/singlefs-checker/src/image.rs:143`），数组从 `crates/singlefs-checker/src/image.rs:76` 起、下标 = 143 − 77 = 66 | `const INVARIANT_I_9_27: u32 = 66u;`（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:141`） | 同 |
| 挂的位置 | 每张树表判一次，挨着 I-9.16 | 在 `w_walk_tree_table_and_mapping` 里，紧接 I-9.16：`w_judge_tree_table_entries_ordering(tt);`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2605`）后一行 `w_judge_tree_table_entry_head_identifiers(tt);`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2606`）；根与由记录施加的版本都进同一处（`if (kind == WALK_KIND_VERSION) {`，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2639`）。GPU 不按表去重，每版各判一遍；违例位是粘的，结果与按表判一遍相同 | 同（结果相同，路径不同） |
| 只在主走读那一份进违例位 | — | `if (w_judging && !holds) {`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:776`）；`w_start(0u, true);`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2925`），别的走读 `w_start(1u, false);` | 同 |
| 「判不了」 | 宽度不够记 `part_of_the_range_not_judged`（判定为「不适用」） | 不置位 | 同（对拍比的是违例位，「不适用」与「成立」都是 0 位） |

我自己加的核验（草稿副本 `/tmp/claude-1000/m3-formal-code-b17-r1-sonnet/ws/`，模型目录 `research/prompts/m3-formal-code-b17-r1-sonnet-model/naga-check/`）：
- `cargo check --offline -p singlefs-checker-tier --lib --features gpu` 退出码 0；不带 `--features gpu` 时 `crash_judge_gpu` 模块根本不编（`#[cfg(feature = "gpu")]` 在 `crates/singlefs-checker-tier/src/lib.rs:18`，下面一行是 `pub mod crash_judge_gpu;`），所以 K3 的编译期断言只有开 gpu 特性才算过。
- 一个单独的小程序（不是 checker 档测试二进制）把 `recovery_kernel_source()`、`checker_walk_kernel_source()`、`checker_scan_kernel_source()` 交给 naga 解析加校验（与 `crash_judge_gpu.rs` 里 `parse_and_validate` 同一套调用），经内存包装跑，原样输出（`naga-run.out`）：
  ```
  恢复内核 解析与校验通过（134091 字节源码）
  池级 checker 走读内核 解析与校验通过（309160 字节源码）
  池级 checker 扫描内核 解析与校验通过（217664 字节源码）
  ```
  自证会红：把走读内核里 `is_zero64(head)` 改成 `is_zero64x(head)` 重编再跑（`naga-red.out`），走读内核报 `error: no definition in scope for identifier: is_zero64x`（`wgsl:5390`），另两个仍通过；还原后 `cmp` 与冻结树逐字节相同。
  ⇒ 「WGSL 的 naga 校验没跑」这一格我补上了：走读内核与扫描内核（含 I-5.5 的扫描内核改动）在冻结树上过 naga 解析与校验。这只证 WGSL 合法，不证 GPU 判得对。

### K1.4　高水位用例与 M1 坏镜像证不证得到它

我量的（副本、经包装；每次都是还原后重编再跑，见第 0 节）：

- 高水位用例 `a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation`（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:150`）
  基线过：它断言「树 ID 从根环水位发的合法镜像：池级 checker 一条都不红」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:278`）与
  「I-9.27 在头 ID 65549 的树表上真被评估过且成立」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:283`）。
- M1（`crates/mutations.tsv` 第 2267 行）施加后：该用例红，`checker_known_bad_images` 69 个仍过。红的断言是「池级 checker 一条都不红」，`left` 只有一条：
  `("I-9.27", "根（实例 3、txg 29）的树表：第 0 条（树 ID 65548、种类 1）的头 ID 是 65537，该等于同一张树表里种类 2 条目的树 ID（这张表里的种类 2 条目：[65549]）")`
  （`research/prompts/m3-formal-code-b17-r1-sonnet-model/logs/mut/row1.log`，摘要在 `research/prompts/m3-formal-code-b17-r1-sonnet-model/mutation-results.txt`）。只红 I-9.27，没有第二条。
- 反面：把判法里「某一条种类 2 的树 ID」换成字面 65537（我自加 G1：种类 1/9/10 那一支改成 `head_identifier == 65537`；G2：种类 2 那一支改成 `head_identifier == 65537`），
  两个都是该用例红、`checker_known_bad_images` 69 过（`research/prompts/m3-formal-code-b17-r1-sonnet-model/logs/mutown/row7.log`、`research/prompts/m3-formal-code-b17-r1-sonnet-model/logs/mutown/row8.log`）。⇒ 高水位用例证得到「别拿字面 65537 比」。
- 判定：高水位用例 + M1 证得到 I-9.27 的「关系、不是字面」这一半，也证得到写者写死 65537 的坏实现。

### K1.5　I-9.27 判法的几个分支没有任何测试证过（不是代码错，是证据缺）

我对 `walk.rs` 里的判法自加了六个变异（A 到 F；C 见 K4.3）（`research/prompts/m3-formal-code-b17-r1-sonnet-model/mutations-own.tsv` 前 6 行，G1、G2 是后两行），都在副本里跑 `pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement` 加 `checker_known_bad_images`（基线 5 + 69 全过）：

| 自加变异（改的是 `crates/singlefs-checker/src/walk.rs` 里的一处） | 结果 | 含义 |
|---|---|---|
| A：`HeadIdentifierOfATreeTableEntry::Zero => head_identifier == 0,`（`crates/singlefs-checker/src/walk.rs:3519`）改成 `=> true,` | 两个目标都过（69 过、5 过），没红 | 种类 3、4、6、7、8、11、12 写 0 那一支没有任何坏镜像证过 |
| B：`HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => head_identifier == tree,`（`crates/singlefs-checker/src/walk.rs:3515`）改成 `=> true,` | 两个目标都过，没红 | 种类 2 等于自己树 ID 那一支没有坏镜像证过 |
| E：`inode_tree_identifiers.contains(&head_identifier)`（`crates/singlefs-checker/src/walk.rs:3517`）改成 `inode_tree_identifiers.first() == Some(&head_identifier)` | 两个目标都过，没红 | 「多于一条种类 2 时放宽」没有任何用例（没有多头镜像） |
| F：种类 1 / 9 / 10 那一支只留种类 1（`TREE_KIND_TOMBSTONE => {`，`crates/singlefs-checker/src/walk.rs:3393`，去掉 9、10） | 两个目标都过，没红 | 种类 9 与 10 在判法里是不是被判，没有坏镜像证过（坏镜像只写了种类 1） |
| D：认法 ① 不看 extent 树的树 ID（那一行 `== TREE_IDENTIFIER_WATERMARK_AT_MKFS` 改成 `== read_u64(extent_entry, 0)`，属 I-5.5，顺手量） | 高水位用例红 | 认法 ① 被高水位用例证住 |
| G1、G2 | 见 K1.4，红 | |

我只跑了这两个目标。别的 harness 目标里有没有谁红 A、B、E、F，我只 `grep` 过：除这两个目标之外，提到 I-9.27 或头 ID 的 harness 文件只有 `tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs`（17-4 那份，只断言头 ID 合种类时 I-9.27 不红）与 `writable_mount_of_a_formatted_pool.rs`（三张不适用表），
没有一份写过「种类 2 头 ID 错」「种类 3..12 头 ID 非 0」的坏镜像 ⇒ 推：A、B、E、F 在整个 harness 档都不红。GPU 对拍也只有一条 I-9.27 坏流（extent 树条目头 ID 写 0：`bytes[entry + 116..entry + 124].fill(0);`，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1183`）。
⇒ GPU 的种类 2、写 0 七个种类、种类 9 / 10 三支，没有一条对拍坏流（推；对拍没跑）。

### K1 判定

- CPU 判法对条款（现在的 `.claude/kb/invariants.md:314` 一行与 D8 句末）：**一致**，逐句无冲突。
- GPU 与 CPU 同判（种类表、条目宽 124、偏移 116、位 66、挂的位置）：**一致**（读代码并排；GPU 对拍没跑，所以是「推的」）。
- 高水位用例与 M1 证得到它：**证得到**（量过：M1 红且只红 I-9.27；字面 65537 的两种读法红）。
- 补充：A、B、E、F 四处判法分支没有测试证过（我量的两个目标里不红；整个 harness 档与 GPU 对拍里没有对应坏镜像是推的）。
- 推翻它的现象：主仓里 `checker_known_bad_images` 或任一别的 harness 目标在 A、B、E、F 之一下红了（我的结论是它们都不红）；M1 在主仓上不红；GPU 对拍里种类 2 / 写 0 的坏流红了又不一致（那就是 GPU 与 CPU 不同判）。

## K3　GPU 覆盖表断言的改法

### K3.1　断言怎么写的

代码（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`，整块是 `const _: () = {` 里的四道）：

1. 条数：两张表条数之和等于 `IMPLEMENTED_INVARIANTS.len()`（`== singlefs_checker::image::IMPLEMENTED_INVARIANTS.len(),`，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:82`）。
2. 覆盖表升序：`"覆盖表按下标升序"`（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:90`）；还没判表升序：`"「内核还没判」那张表按下标升序"`（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:99`）。
3. 每个下标至少在一张表里：`while invariant_index < singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() {`（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:105`），失败信息 `"池级 checker 的每一条要么在覆盖表里、要么在「内核还没判」那张表里"`（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:127`）。
4. 注释自述：`// 每个下标恰好在一张表里：两张表都升序、合起来条数等于不变量条数，再要求每个下标至少在一张表里，就一条不缺、一条不重。`（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:103`）。

鸽巢推导（推的，下面用编译实测支撑）：第 3 道要求 0..N−1 每个下标至少出现一次，要 N 个互不相同的在范围内的条目；第 1 道把两张表的条目总数钉在 N ⇒ 每个条目都在范围内、彼此不同，也就是每个下标恰好出现一次。出范围的下标、跨表重复、同表重复都过不了第 1 或第 3 道；第 2 道是额外的形式约束。

kb 对得上：`.claude/kb/invariants.md:16` 写「编译期断言两张表合起来是 `IMPLEMENTED_INVARIANTS` 的每一条、每个下标恰好在其中一张表里、两张表各按下标升序」。

### K3.2　编译实测（副本，`cargo check --offline -p singlefs-checker-tier --lib --features gpu`，不跑测试）

驱动 `research/prompts/m3-formal-code-b17-r1-sonnet-model/const-assert-variants.py`，每个变体只改两张表，跑完还原；输出原样（`k3-const-assert.out`）：

```
V0 原样（今天的两张表）: cargo check 退出码 0
V1 覆盖表少 67（I-5.5 没登记）: cargo check 退出码 101
    error[E0080]: evaluation panicked: 池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表（暂不补内核的，点名进「内核还没判」那张表）
V2 66 同时在两张表里: cargo check 退出码 101
    error[E0080]: evaluation panicked: 池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表（暂不补内核的，点名进「内核还没判」那张表）
V3 覆盖表 66 写两遍、67 缺（条数对、升序破）: cargo check 退出码 101
    error[E0080]: evaluation panicked: 覆盖表按下标升序
V4 覆盖表 65 与还没判表重复、66 缺（条数对）: cargo check 退出码 101
    error[E0080]: evaluation panicked: 池级 checker 的每一条要么在覆盖表里、要么在「内核还没判」那张表里
V5 覆盖表 67 写成 68（条数对、67 缺）: cargo check 退出码 101
    error[E0080]: evaluation panicked: 池级 checker 的每一条要么在覆盖表里、要么在「内核还没判」那张表里
V6 还没判表点名 67、覆盖表不含 67（把 I-5.5 登记成内核没判，应当编得过）: cargo check 退出码 0
V7 还没判表空、覆盖表 0..=67 全列（根环线第 9 批删表之后的形态，应当编得过）: cargo check 退出码 0
V8 还没判表空、覆盖表 0..=67 少 65（删表而 I-3.5 没补，应当编不过）: cargo check 退出码 101
    error[E0080]: evaluation panicked: 池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表（暂不补内核的，点名进「内核还没判」那张表）
```

### K3.3　判定理由

- 漏判（一条既不在覆盖表也不在还没判表）：V1、V5、V8 编不过。重复登记：V2（跨表）、V3（同表）、V4（跨表加缺一条）编不过。⇒ 编不过的门挡得住漏与重。
- 它不挡的：把一条内核其实判了的不变量登记成「内核还没判」（V6 编得过）。这是主动点名、不是漏；覆盖表里每一条内核真判了，靠每条各有一条对拍坏流（K1.5 说了 I-9.27 的坏流只有一条），不靠这道断言。旧断言（`覆盖表按下标次序、一条不缺`）同样不挡这个方向。
- 错误信息：V2（重复）走的是第一道，报的是「新立的不变量 GPU 内核还没判」，说的是「漏」，与实际（重复）不符；断言红得对，话不对（小毛病，不影响编不过）。
- 与根环那条线接得上：V7（还没判表空、覆盖表 0..=67）编得过，等价于旧的「全覆盖」断言；V8（删表而 I-3.5 没补进覆盖表）编不过，所以删表之后断言仍然顶着「全覆盖」。道理：还没判表空时，第 1 道钉条数 = N、第 3 道钉每个下标至少一次、第 2 道钉升序，覆盖表只能是 0..N−1（推）。
  真删 `POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET` 这个常量本身时，断言块里引用它的三处（第 1 道、还没判表的升序循环、第 3 道里的内层循环）要一起删，我没试这一步（V7 保留的是空表）。
- 合并面：根环那条线的 `records/2026-10-01-里程碑三形式化接入.md:743` 说「第 9 批 GPU 实现 I-3.5 时删「内核还没判」表、断言回到全覆盖，以这条线合完的版本为底」；b17 重写的恰是同一个 `const _` 块，那一步要在这块上做，不是在旧文本上做。
- `#[cfg(feature = "gpu")]` 门：断言只在开 gpu 特性的编译里求值（`crates/singlefs-checker-tier/src/lib.rs:19`）。主 agent 说 clippy 用了 `--all-targets --all-features`，那一趟会求值；我的 V0 退出码 0 与此一致。

### K3 判定

**一致**：断言挡漏、挡重、挡升序破、挡出范围，实测 V1–V5、V8 红，V0、V6、V7 过。依据是我在副本上开 gpu 特性的 `cargo check`，不是读出来的。
推翻它的现象：主仓 `--features gpu` 的编译在 V1–V5 或 V8 的任一种表上编得过；或 V0 的今天的表编不过。

## K4　变异行与测试的判别力

### K4.1　8 行变异（`crates/mutations.tsv` 第 2267–2274 行，与 diff 全文里的 `mutations-append.tsv` 8 行逐行相同：`diff` 输出 SAME）

我的量：副本，每行用驱动按表里的原文 / 替换文 / cargo 参数施加（原文在文件里恰好命中一次，驱动打印 `anchor hits 1`），还原后重编。

| 行 | 名（引自表） | 改的是哪一处 | 它点名要证的 | 副本上的结果 | 判定 |
|---|---|---|---|---|---|
| 2267 | 「C673 ⑤ I-9.27 判别力（攻方 M1）」（`crates/mutations.tsv:2267`） | `crates/singlefs-core/src/transaction.rs` 写树表条目处 `head_identifier,` 改成「非 0 一律写 `TREE_IDENTIFIER_INODE`」 | 写者写死 65537，在树 ID 从根环水位发的那一版上被查出 | 高水位用例红；`left` 只一条 I-9.27；`checker_known_bad_images` 69 过 | 改对了处、红对了测试 |
| 2268 | 「C673 ⑥ I-5.5 判别力（实审 A2c 第 15 条那一形）：用户数据落点不从单元区起点起扫，新池新建文件的数据单元不在 U + 4（默认环）」（`crates/mutations.tsv:2268`） | `crates/singlefs-core/src/allocator.rs` 里 `lowest_user_data_slot`（`crates/singlefs-core/src/allocator.rs:737`）的起点加 `CLUSTER_SEGMENT_SLOTS` | I-5.5 在默认环新池新建文件上判得到落点偏 | 默认环用例红：`盘 0 上落点最小的数据单元在槽 50240，不是单元区起始槽号 50176 + 4 = 50180` | 改对了处、红对了测试 |
| 2269 | 「C673 ⑥ I-5.5 判别力（实审 A2c 第 15 条那一形）：用户数据落点不从单元区起点起扫，128 MiB 环的新池新建文件数据单元不在 U + 4」（`crates/mutations.tsv:2269`） | 与 2268 同一处、同一替换文（文件、原文、替换文三段相同，只有点名要红的测试不同） | 128 MiB 环 U 现算 | 在 2268 那次跑里同时红：`槽 9280，不是单元区起始槽号 9216 + 4 = 9220` | 红对了测试 |
| 2270 | 「C673 ⑥ I-5.5 判别力（实审 A2c 第 15 条那一形）：用户数据落点不从单元区起点起扫，只暖机过的实例 2 写第一个文件那一版数据单元不在 U + 4」（`crates/mutations.tsv:2270`） | 同 2268 | 暖机后重挂、从 mkfs 水位建树的版本也被认出 | 在 2268 那次跑里同时红：`根（实例 2、txg 5）… 槽 50304，不是单元区起始槽号 50176 + 4 = 50180` | 红对了测试 |
| 2271 | 「C673 ⑥ I-5.5 判法：认法 ② 不看诞生 txg（覆盖写那几版也被当成从 mkfs 水位第一次建树的那一版来判）」（`crates/mutations.tsv:2271`） | `crates/singlefs-checker/src/walk.rs` 认法 ② 那一行末尾加 `\|\| true` | 覆盖写那几版不被认成从 mkfs 水位建树的那一版 | 只红 `once_the_first_file_version_leaves_the_root_ring_…`（该文件 4 过 1 红）。注意被它红的断言是「根环里只剩覆盖写那几版…：I-5.5 一版都不判」，不是落点断言 | 改对了处、红对了测试 |
| 2272 | 「C673 ⑥ I-5.5 判法：落点判成恒成立（数据单元搬走的坏镜像不红）」（`crates/mutations.tsv:2272`） | `crates/singlefs-checker/src/walk.rs` 里 `judgements.judge(invariant, smallest_slot == expected_slot, …)` 的条件改成 `true` | 坏镜像 `move_the_data_unit_of_the_first_file_two_slots_further`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1318`）要判违例 | `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 红：`I-5.5 那份坏镜像应当判违例，实际 Holds`。只跑了那一个名字（`-- the_clean_image_…`），其余 68 个被过滤 | 改对了处、红对了测试 |
| 2273 | 「C673 ⑤ I-9.27 判法：extent 树、deleted_inodes、墓碑树条目的头 ID 不比 inode 树的树 ID（头 ID 写 0 的坏镜像不红）」（`crates/mutations.tsv:2273`） | `crates/singlefs-checker/src/walk.rs:3517` 的 `inode_tree_identifiers.contains(&head_identifier)` 改成 `true` | 头 ID 写 0 的坏镜像要红 I-9.27 | `each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant` 红：`判红的该只有 I-9.27 … left: [] right: ["I-9.27"]` | 改对了处、红对了测试 |
| 2274 | 「C673 ⑤ I-9.27 判法：extent 树、deleted_inodes、墓碑树条目的头 ID 拿条目自己的树 ID 比（树 ID 从根环水位发的合法镜像上误红）」（`crates/mutations.tsv:2274`） | 同一处改成 `head_identifier == tree` | 高水位合法镜像上不误红 | 高水位用例红；同一变异下新文件的另外四个用例也红（新池里 extent 树条目 树 ID 65536、头 ID 65537，`left` 为 `("I-9.27", "根（实例 1、txg 3）的树表：第 0 条（树 ID 65536、种类 1）的头 ID 是 65537，…[65537]")`） | 红对了测试，还多红四个（它比点名的更狠，不是问题） |

要点：
- 第 3、4、6、7、8 行（正文问「没证的那几行推不推得出红」）我都量了、不是推：3、4 随 2 一次跑红；6、7、8 单独跑红。8 行都红。
- 表里 2268–2270 三行是同一处变异：这三行证的是同一个改动在三个用例上都红，不是三个不同的改动。
- 摘要与每次的原始输出：`research/prompts/m3-formal-code-b17-r1-sonnet-model/mutation-results.txt`、`research/prompts/m3-formal-code-b17-r1-sonnet-model/logs/mut/`。

### K4.2　17-4 之后点名那条用例的两行旧变异

基线：`tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker` 8 过（17-4 改后的 `swap_the_kinds_of_the_tombstone_and_lineage_entries`，`crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs:129`）。

| 行 | 名 | 副本结果 | 判定 |
|---|---|---|---|
| 1605 | 「三·2 第 7 条：池级 checker 判 I-9.16 ② 的发号次序不认四棵新树（退回七棵）」（`crates/mutations.tsv:1605`） | 点名用例 `the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds`（`crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs:403`）红：`判红的该只有 I-9.16 … left: [] right: ["I-9.16"]` | 仍红 |
| 1809 | 「第 4 批合入后 I-8.10：树表排序探针改了根不把根照抄进 journal 记录的新根段」（`crates/mutations.tsv:1809`） | 同一用例红（该文件 6 过 2 红，另一条是 `the_checker_reddens_the_tree_table_ordering_and_the_key_order_when_two_entries_swapped_their_places`）；点名用例的 `left` 为 `["I-9.16", "I-8.10"]` | 仍红 |

两行原文在副本里恰好命中一次（驱动 `anchor hits 1`）。17-4 的改法让互换后的头 ID 合新种类（`crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs:172` 断言「互换之后第 8 条是种类 11（谱系树），头 ID 合它的新种类：0」）。

### K4.3　顺手量到的一处弱点（归 K2，攻方腿的格；我只报量到的）

变异 C：I-5.5 判法里每盘取最小槽号那一步（`(*smallest_slot).min(location.slot)`）改成 `.max(...)`：`pool_checker_judges_…` 5 过、`checker_known_bad_images` 69 过，不红（`research/prompts/m3-formal-code-b17-r1-sonnet-model/logs/mutown/row3.log`）。
原因（推）：I-5.5 的夹具都只有一个数据单元（`pub const FILE_BYTES: usize = 3000;`，`crates/singlefs-harness/tests/common/mod.rs:38`，覆盖写那几版也是 1000 多字节），每盘上「最小」「最大」「唯一」是同一个槽。
⇒ 条款「每块盘上落点槽号最小的那一个」里的「最小」没有被任何夹具区分；t1 与一个更靠后的单元同在时，取法对不对没人证。

### K4 判定

**一致**（正文问的三件事都有答）：
- 8 行变异各自改的是它点名的那一处：是（2268–2270 是同一处）。
- 3、4、6、7、8 行：推得出红，而且我量了，红。
- 17-4 之后的 1605、1809：仍红。
- 补充：没有被变异证到的分支（K1.5 的 A、B、E、F，K4.3 的 C）不属于这 8 行点名范围，是证据缺口，不是变异表错。
推翻它的现象：主仓上这 8 行或 1605、1809 里有一行不红；2268–2270 在主仓上不是同红；或 A、B、C、E、F 在主仓某个 harness 目标里红了（那就是我的「没证」不成立）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 CPU 判法对条款 | 一致 | 种类 2 自己、1/9/10 对同表种类 2（多条放宽、无则红）、3/4/6/7/8/11/12 写 0、别的不判、0 条不判、宽不到 124 整表不判，逐句对上；条款一行在我读的过程中被改过，按改后的对 |
| K1 GPU 与 CPU 同判 | 一致（推） | 种类表、条目宽 124、偏移 116、种类偏移 10、位 66、挂的位置都并排对上；GPU 对拍没跑；naga 解析与校验三个内核全过（我补量） |
| K1 高水位用例与 M1 | 证得到（量） | M1 红且只红 I-9.27；判法换成字面 65537 的两种读法也被高水位用例红 |
| K1 补充：判法分支证据缺口 | 规则没说、证据缺 | 种类 2 自等、写 0 七个种类、种类 9/10、多条种类 2 放宽这四处，harness 两个目标里自加变异都不红；GPU 对拍只有 extent 写 0 一条坏流 |
| K3 覆盖表断言 | 一致（量） | 开 gpu 特性的 cargo check：漏、重、升序破、出范围的表都编不过，今天的表与删表后的全覆盖形态编得过；与根环线接得上（V7、V8） |
| K4 8 行变异 | 一致（量） | 8 行都改在点名的一处、副本上都红；3、4 与 2 是同一处变异；3、4、6、7、8 我量了不是推 |
| K4 17-4 后的 1605、1809 | 一致（量） | 两行仍红 |
| K4 补充：I-5.5「最小」取法 | 证据缺口（归 K2） | `min` 改 `max` 在 harness 两个目标里不红；夹具每盘只有一个数据单元（推） |

## 没做什么

- 没跑 checker 档任何测试（GPU 对拍两条新用例、`crash_judge_gpu` 的 `--lib` 里 naga 三条、`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 整个文件），按提示归提交时由主 agent 跑。K1 里「GPU 与 CPU 同判」因此是读代码得来的，标「推」。naga 解析校验我用一个单独的小程序做了（不是 checker 档测试二进制），它只证 WGSL 合法。
- 没判 K2（认法与判据误红、漏判）：那是攻方腿的格。K4.3 与 K1.5 里量到的变异 C 是顺手报的事实，不是对 K2 的判定。
- 没把整个 harness 档跑一遍去看 A、B、E、F 是否在别处红：只跑了 `pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement` 与 `checker_known_bad_images` 两个目标，别处只 `grep` 过，所以「整个 harness 档都不红」是推的。
- 没验「真删『内核还没判』常量本身」这一步（V7 保留了空表）。
- 没替主 agent采纳或出判决。
- 没动仓里 `crates/` 与 kb 的任何文件；kb 里 `.claude/kb/invariants.md:314` 的状态列这时已写「已实现」，不归我改。

## 要主 agent 跑的命令

没有放后台的活。K1 要提升到「量」的两处都在 checker 档：GPU 对拍里那两条新用例（I-9.27 的 `the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_identifier_other_than_the_inode_tree`，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2633`；I-5.5 那条紧随其后）与 `crash_judge_gpu` 的 `--lib` naga 三条，归提交时那一趟，我不起。

## 留下的东西与已删的东西

- 入库：`research/prompts/m3-formal-code-b17-r1-sonnet-model/`：`run-mutation-row.py`、`const-assert-variants.py`、`mutations-append.tsv`（17-1 那 8 行）、`mutations-own.tsv`（我自加的 8 行）、`mutation-results.txt`、`k3-const-assert.out`、`naga-run.out`、`naga-red.out`、`naga-check/`（小程序源码，`Cargo.toml` 改名成 `Cargo.toml.draft`，免得被当成 cargo 包）、`logs/`（各次 cargo 原始输出）。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b17-r1-sonnet/` 里留着日志与小文件；我建的编译目录与仓副本 `/tmp/claude-1000/m3-formal-code-b17-r1-sonnet/ws/`（3.7G，含 `target/`）已 `rm -rf` 删掉，交回时不在。要复现：把冻结树的 `crates/` 与仓根 `Cargo.toml`、`Cargo.lock`、`.cargo/` 拷一份，用模型目录里的驱动。

## cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b17-r1-sonnet-output.md --root /tmp/claude-1000/m3-formal-code-b17-r1/tree --root . --background research/prompts/_m3-formal-code-b17-r1-background.md` 末行：

```
  ✓ 核了 23 处引文，对上 23 处，没判 0 处（逐处列在上面）
```
