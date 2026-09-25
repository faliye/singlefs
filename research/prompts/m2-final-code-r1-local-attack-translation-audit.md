# m2-final-code-r1 本地攻方提示：核对表

代码读冻结副本 `/tmp/claude-1000/m2-final-code-r1/tree/crates/`，行号在该副本里现查。

## 表一：写死的事实（数与公式）与来源文件:行

| 提示文件里的事实编号 | 数/公式 | 来源文件:行 |
|---|---|---|
| Fact 1a | INSTANCE_TABLE_PAGE_RECORDS = 370 | singlefs-format/src/lib.rs:117 |
| Fact 1b | 370 条记录，末条链指针，369 条数据行；0 行也是一片 | singlefs-core/src/instance_table.rs:123-124 |
| Fact 1c | instance_table_pages_for_rows：rows_per_page=369，pages=ceil(rows/369).max(1) | singlefs-core/src/instance_table.rs:126-129 |
| Fact 1d | 测试断言 0→1，369→1，370→2，738→2，739→3 | singlefs-core/src/instance_table.rs:391,393-400（`pages_for_rows_is_at_least_one_and_opens_a_page_every_three_hundred_sixty_nine_rows`，断言行 393-400） |
| Fact 2a | JOURNAL_RECORD_BYTES = 4096 | singlefs-format/src/lib.rs:156 |
| Fact 2b | JOURNAL_HEADER_BYTES = 311 | singlefs-format/src/lib.rs:165 |
| Fact 2c | JOURNAL_NAMED_ENTRY_BYTES = 56 | singlefs-format/src/lib.rs:168-169（数值由 324 行 `assert_eq!(JOURNAL_NAMED_ENTRY_BYTES, 56, ...)` 核) |
| Fact 2d | JOURNAL_NAMED_ENTRIES_PER_RECORD = (4096-311)/56，u64 整除截尾 | singlefs-format/src/lib.rs:172-173 |
| Fact 3a | instance_table_page_roles_in_bump_order(pages) 恰返回 pages 项 | singlefs-core/src/transaction.rs:2259-2270（`(0..pages).rev()...collect()`） |
| Fact 3b | 写行记录 named = P 个实例表片角色 + 1 个分配记录树角色 | singlefs-core/src/transaction.rs:970-971（取 P）、1038-1063（`.chain([NamedUnit{...}])`，链接恰一项） |
| Fact 3c 前半 | ordinal_within_publish 恒 FIRST，place_in_publish 恒 LastRecordOfThePublish | singlefs-core/src/transaction.rs:1028-1031 |
| Fact 3c 后半（结构观测，非中文原文转述） | 这条路径不调 roles_named_by_each_record_of_the_publish，to_bytes 不核 named.len() 上限 | singlefs-core/src/transaction.rs:947-1064（全函数体，未见调用）；singlefs-core/src/journal.rs:231-233（`to_bytes` 里 `for named in &self.named { named.write_to(&mut writer); }`，无长度核） |
| Fact 4a-4e | roles_named_by_each_record_of_the_publish 与 transaction_offset_of_each_record_of_the_publish 两份文档注释 | singlefs-core/src/transaction.rs:3118-3126（前者文档）、3130-3167（前者函数体，`chunks(named_unit_capacity)` 在 3159-3164）、3169-3172（后者文档）、3174-3186（后者函数体，`min(...)` 在 3182） |
| Fact 5a | 记录字节写入次序与各字段宽度 | singlefs-core/src/journal.rs:190-230（`to_bytes` 逐字段 `put_*`/`skip`/`assert_position` 调用） |
| Fact 5c | header_csum 罩 [0,4096)，字段自身 32 字节按 0 参与 | singlefs-core/src/journal.rs:1-3（模块顶注）、241-244（`to_bytes` 里 `wide_checksum_with_field_zeroed(&bytes, record_bytes, ...)`，`record_bytes`=4096）；singlefs-core/src/checksum.rs:65-80（`wide_checksum_with_field_zeroed` 定义与其文档注释） |
| 偏移 7 / 87 结构观测 | JOURNAL_RECORD_FLAGS_OFFSET=7，JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET=46+32+8+1=87（来自 JOURNAL_HEADER_CHECKSUM_OFFSET=46） | singlefs-core/src/journal.rs:21-22（偏移46公式）、24（偏移7）、63-65（偏移87公式） |
| 偏移 91 / 95 结构观测（非常量，取自运行时 writer 位置，非中文原文转述） | back_chain 起 91，payload_checksum 起 95：由 87 之后 `put_u32(ordinal)`(4)+`put_u32(back_chain)`(4) 累加，`writer.position()` 记为 payload_checksum_offset | singlefs-core/src/journal.rs:215-217 |
| 报告口径「片数 ≥ 67、行数 > 24354」 | 这一句是主 agent 交办的既有报告结论，不是我在冻结副本里查到的注释；提示 Question 3d 把它作为背景事实交给模型自己核算，不由我先行判断真假 | 派发提示原文（本轮任务描述），非仓内文件 |

## 表二：转述自中文原文的句子——逐句核对

首稿：直接写成提示文件里现在这句之前、`cat >> ... <<'EOF'` 第一次落盘时的原文，不另存草稿文件。

| 英文项（提示文件行号定位） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact 1b/1c（instance_table_pages_for_rows 整段文档） | singlefs-core/src/instance_table.rs:123-124 | 无遗漏：370/369/D18 已定项11/0行也是一片/与D28已定项3同一个数，五点都在 | 未改动 |
| Fact 1d（测试断言） | singlefs-core/src/instance_table.rs:391,393-400 | 无遗漏：五组输入输出都抄全 | 未改动 |
| Fact 3b 引用的行内注释「点名项…这次重写的角色各一项」 | singlefs-core/src/transaction.rs:1038 | 原文「按 bump 次序——实例表链的各片（尾片先），分配记录树」这半句排序细节首稿就没收；这是有意简化，见下方「未逐句核对」一节，不算遗漏未察觉 | 未改动（有意简化，已在下方说明） |
| Fact 3c 前半「ordinal_within_publish 恒 FIRST」 | singlefs-core/src/transaction.rs:1028-1029 | 无遗漏 | 未改动 |
| Fact 3c 前半「place_in_publish 恒末条」 | singlefs-core/src/transaction.rs:1030-1031 | 无遗漏 | 未改动 |
| Fact 4a 整段（roles_named_by_each_record_of_the_publish 文档第一段） | singlefs-core/src/transaction.rs:3118-3122 | 首稿把「C310（事务切分纪律与记录数口径打架）」简化成「a 2026-09-16 user decision」，漏了 C310 这个编号标签本身——与本提示 Background 一节自己承诺的「tags kept as opaque labels」矛盾 | 用 `replace-once.py` 补回「C310」标签，改成「(C310, a 2026-09-16 user decision, fixed that …)」 |
| Fact 4a「C491 未定→2026-09-23 定」那半句 | singlefs-core/src/transaction.rs:3118-3119 | 首稿保留了「used to be an unsettled question…settled on 2026-09-23」的实质内容，但同样把「C491」这个编号标签本身简化掉了，只保留了它的中文括注内容（「多条记录时共享内生块在哪条点名」被意译成「which record names the roles…that are not data units」） | 未改：编号「C491」仍未补回提示文件（只有 D23/C310 补了，C491 这处漏网）——记入下方「仍未核平」一节，不隐瞒 |
| Fact 4b（roles_named 文档第二段，写1个或0个数据单元） | singlefs-core/src/transaction.rs:3121-3122 | 无遗漏：「写了一个数据单元或一个都没写」「一个事务点名全部重写角色」「第一个事务今天的形态」三点都在 | 未改动 |
| Fact 4d（末条再跨记录） | singlefs-core/src/transaction.rs:3124-3126 | 无遗漏：67 项门槛、bump 次序、2026-09-24 用户定案、只有真正最后一条带标志，四点都在 | 未改动 |
| Fact 4e（transaction_offset 文档整段） | singlefs-core/src/transaction.rs:3169-3172 | 无遗漏：前 N−1 条各自事务、其余归最后一个事务、共享事务号（已定项7）三点都在；「已定项7」这个编号首稿就没抄（原文「共享它的事务号（已定项7「同一事务的全部记录共享它」）」） | 未改：「已定项 7」这个编号同样漏收——记入下方「仍未核平」一节 |
| Fact 5c（header_csum 覆盖范围） | singlefs-core/src/journal.rs:1-3；checksum.rs:65-66 | 无遗漏：[0,4096)、含补齐、字段自身按0参与、I-2.4 四点都在 | 未改动 |

## 提示文件里比原文多出来的限定词/括注

| 多出来的地方 | 多的是什么 | 为什么加 |
|---|---|---|
| Fact 1c 结尾 | 「with a floor of 1 page even when rows = 0」重复用「floor」这个词，原文只说「0 行也是一片」 | 数学上 max(1, …) 就是取下限，用 floor 一词让不熟悉中文原文写法的模型更明确这是下限而非四舍五入，不改变原意 |
| Fact 2d | 「it never rounds up」 | Rust 整数除法截尾这件事原文只写「(4096 − 311) ÷ 56」没有显式说明取整方向；补这半句是为了不让模型默认按数学除法四舍五入或向上取整算出 68，这是本题算术正确性的关键限定词，原文虽未逐字写出但由「⌊…⌋」符号与 u64 整除隐含 |
| Fact 3c 后半（结构观测） | 整句「The code path that builds this record never calls roles_named_by_each_record_of_the_publish…and never checks the named-item list's length against any per-record maximum」 | 这不是对某句中文原文的转述，是我读 `transaction.rs:947-1064` 全函数体与 `journal.rs:231-233` 之后的结构观测（该路径确实没有调用点名项拆分函数，`to_bytes` 也确实没有长度核对），用来支撑 Question 3 让模型自己判断「装不装得下」，不登记进表二 |

## 未逐句核对的部分（结构事实，不是对某句中文原文的转述）

Fact 3a（instance_table_page_roles_in_bump_order 返回 P 项）、Fact 5a（记录字节写入次序与宽度）、偏移 91/95 的推算，都是我自己读 `transaction.rs:2259-2270`、`journal.rs:190-230` 代码本身（字段调用顺序、`assert_position` 断言值）之后转写的结构事实，不是对某一句中文 doc 注释的逐句转述，因此不适用「转述都要对着原文核」那条、不登记进表二（同 `m2-wave3-code-r1-local-attack-translation-audit.md:3-4` 的先例记法）。

Fact 3b、3a 里为了只保留「返回几项」「加几项」这个计数事实，有意去掉了 `transaction.rs:2259-2260`（尾片先、D3 已定项10⑤、D19 已定项9）与 `transaction.rs:1038`（按bump次序、尾片先）里的落点顺序细节——这些细节只影响哪一片对应哪个指针，不影响「一共几项」这个算术问题，故不转述、也不算遗漏未察觉。

## 仍未核平（本轮交回时诚实登记，未回填）

- Fact 4a 里的编号「C491」（原文见 `transaction.rs:3118`）在提示文件里被意译掉、没有像 C310 那样补回标签本身。
- Fact 4e 里的编号「已定项 7」（原文见 `transaction.rs:3172`）在提示文件里同样只译了内容（「共享事务号」）没有补回编号。
两处都不影响算术内容本身（模型不需要知道 C491 或已定项7的编号就能做 Question 4 的计算），但按 Background 一节「tags kept as opaque labels」的自我承诺，这两处严格说没有做到；已如实记录，未回填，因为提示文件已进入跑样阶段、不再改动（改动会使已跑的样本与提示文件不同版本）。
