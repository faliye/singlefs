# 转述核对表：m2-emptypool-nonempty-r1-local-attack

核对本轮提示文件里每一处把中文原文译成英文转述的地方。格式：英文项 / 原文文件:行 / 首稿缺的 / 定稿。
「首稿缺的」记的是逐句核对时发现英文比原文窄、或英文加了原文没有的限定词的地方；没有发现缺漏或多加的，写「无」。

## 一、十条变异的英文标签（Section 4 的 Label 行）

| 英文项（定稿） | 原文文件:行 | 首稿缺的 / 多的 |
|---|---|---|
| Step 5: fall back the non-empty test to the journal ring's transaction number (...) | crates/mutations.tsv:62 | 无 |
| Step 5: the previous-root lookup for non-empty takes any readable root, not just valid ones (...) | crates/mutations.tsv:63 | 多：「not just valid ones」原文没有这半句，是从紧邻的中文注释（mount.rs:495「前一条」只在有效根里找）与被替换的原文变量名 valid 推出来的，为了让模型看得出这条变异把取值范围从「valid」扩大到「readable」，加了这半句并在提示里同一条旁边给出 valid/readable 两个变量名的原文对照 |
| Step 5: the non-empty test compares only the inode-tree root pointer, not the extent-tree root pointer | crates/mutations.tsv:64 | 无 |
| Step 3: after remounting a pool that has only been through mkfs, the allocator does not mark mkfs's two units in the unit region as allocated | crates/mutations.tsv:65 | 无（「不认」译成「不标记为已分配」，依据是该行替换文把 mark_format_time_units 调用换成了 let _ = (...)，语义上就是不再标记为已分配，不是漏译） |
| Step 3: on a pool that has only been through mkfs, the zero-unit warm-up publish writes back_chain as 0 instead of computing it from the previous record | crates/mutations.tsv:66 | 无 |
| Step 3: the zero-unit publish is missing one barrier between the journal record write and the root write | crates/mutations.tsv:67 | 无 |
| Step 3: after remounting a pool that has only been through mkfs, the mkfs instance table's placement is computed using the tree table's span (1 slot) instead of the instance table's own span (2 slots) | crates/mutations.tsv:68 | 多：原文只说「按 1 槽记」，括注「(1 slot) instead of the instance table's own span (2 slots)」是从 T4 用例里 assert_eq!((allocated_slots(), deferred_slots()), (3, 0), "只有 mkfs 的实例表 2 槽与树表 1 槽") 这条断言反推出实例表跨度 2、树表跨度 1，为了让模型不用自己去比对两个 span_slots() 就能核这条变异而加的，提示里把这条断言原文也附在 Section 3.6 |
| Step 3: when a version whose tree table is empty needs to write instance-table rows, the refusal is moved to after acquiring the instance (it now returns only after the superblock has already been written with the new instance number) | crates/mutations.tsv:69 | 无 |
| Step 4: rolling back to a root whose tree table is empty is no longer refused before any write (the check is replaced by a condition that is always false, so the refusal branch can never run) | crates/mutations.tsv:70 | 多：括注是从替换文字面 if false && ... 直接推出来的机制说明，原文标签只说结果（不在写之前拒绝），不说机制；括注不会改变结果，只是把「怎么做到的」说清楚，供模型核对替换文与标签是不是同一件事 |
| Step 5: when computing the ceiling for raising F, an unreadable tree table on a valid root is guessed as containing neither tree, instead of being refused | crates/mutations.tsv:71 | 无 |

## 二、九个测试的文档注释（Section 3.2/3.3/3.5/3.6/3.8/3.9）

| 英文项（定稿） | 原文文件:行 | 首稿缺的 / 多的 |
|---|---|---|
| T1: Non-empty is recognized from disk (...) txg 14 would become an empty root, the fourth-newest non-empty root would drop to A at txg 3, and the ceiling would drop to 3. | crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:317-319 | 无 |
| T2: The previous root is looked up only among valid roots (...) raising to 4 is refused. | crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:364-366 | 多：「which sits between A and D in txg order」里的「in txg order」原文「夹在 A 与 D 之间」没有明说按 txg 排序，是从上下文（A 是 txg 3、D 是 txg 9、C 是 txg 8）补的限定词，避免模型误读成任意顺序 |
| T3: If either tree's root pointer changed, it counts as non-empty (...) is compared against a value meaning the tree table contains neither tree. | crates/singlefs-core/src/mount.rs:1100-1101 | 多：「The oldest valid root, which has no previous root」里的「which has no previous root」原文这一句没有重复说，是从同一函数的 rollback_floor_ceiling 与 T1 的用法补的限定词，说明为什么最旧的有效根要与「两棵树都没有」比 |
| T4: Acceptance: instance number acquired is 1, no rows written (...) after the first file version, all 26 hold. | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:113-116 | 无 |
| T5: Segment sequence: ten segments, closed-form count 262165 (...) identical to the stream from running the first transaction in the same process right after mkfs. | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs:118 | 无 |
| T6: The version that runs normally: the 18-write segment is not expanded (...) every other segment is expanded over any subset. | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs:126 | 多：「either fully persisted or not at all」里的「or not at all」原文「只以整段持久进入后面的状态」没有单独说反面，是为了把「整段持久」说成一个二选一（要么整段已持久、要么还没到这一段）而补的半句，不改变原意 |
| T7: Writing instance-table rows on a version whose tree table is empty is unsupported (...) the recorded-operation stream does not gain a single step. | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:58-60 | 无（「没有文件版本的一版」译成「a version whose tree table is empty」，两者在全篇背景材料与本提示里通用，见 D16 已定项 9「树表 0 条 ⇒ 还没发布过文件版本」） |
| T8: Rolling back to a root whose tree table is empty (...) is no longer refused before any write. | crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs:84-85 | 无 |
| T9: When computing the ceiling, one valid root's tree table cannot be read (...) raising F would succeed. | crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:423-425 | 无 |

## 三、DiskSnapshot 与录制流的文档注释（Section 6.1/6.2）

| 英文项（定稿） | 原文文件:行 | 首稿缺的 / 多的 |
|---|---|---|
| A comparable on-disk snapshot: the raw bytes of both superblock slots (...) not a single write and not a single barrier was issued. | crates/singlefs-harness/tests/common/mod.rs:242-243 | 无 |
| A recorder wrapped around a block device (...) Reads are not recorded: crash states are cut only at writes and barriers. | crates/singlefs-harness/src/lib.rs:1,3 | 少：原文第 1 行括注「（里程碑步 0）」与第 3 行括注「（D13（验证路线） 已定项 4）」两处出处引注都没有译出。两处都是给人看的立项 / 决策出处，不是 Z6 要核的技术限定词（本轮判据不依赖「这是哪个里程碑」或「哪条决策定的」），故意略去；提示第 0 节已要求模型只依据文档给的事实作答，出处引注对分辨变异与用例的覆盖没有影响 |

## 四、其它就地译出的代码注释（Section 4 的两处上下文函数注释）

| 英文项（定稿） | 原文文件:行 | 首稿缺的 / 多的 |
|---|---|---|
| this is what mutation M10 (defined below) changes: if the tree table cannot be read or cannot be parsed, refuse instead of guessing: it cannot be decided whether this root is non-empty or what the next root should be compared against (only comparing tree table root pointers is decided). | crates/singlefs-core/src/mount.rs:482 | 少：原文「读不出要走修复」（unreadable must go through repair）这半句没有译进这条代码内注释。这半句的完整译法已经出现在同一份提示 Section 3.2 对 T9 文档注释的翻译里（「an unreadable tree table must go through repair, it may not be skipped or guessed」），代码内注释只保留了与该函数字面逻辑直接对应的那半句，避免两处重复；判据表 Question 3/6 用到的正是 Section 6.6 与 T9 docstring 那份完整表述，不依赖这条代码内注释 |
| the previous root is looked up only among valid roots: an abandoned-timeline root and a root below F do not count. | crates/singlefs-core/src/mount.rs:495 | 无 |

## 五、prepare() 里段形状断言的消息串译文（Section 3.7）

| 英文项（定稿） | 原文文件:行 | 首稿缺的 / 多的 |
|---|---|---|
| acquiring the instance is two writes, then the row-writing record is two writes, then a root, then the superblock is two writes, then the warm-up record is two writes, then a root, then the superblock is two writes together with the first file version's sixteen unit writes, then a record is two writes, then a root, then the superblock is two writes | crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs:57 | 无（「写行发布的记录两写」「暖机的记录两写」译成「the row-writing record」「the warm-up record」，比原文短一个「发布的」/「的」，但同一份提示 Section 3.7 紧邻的说明段落里已把完整关系讲清楚：写行那次发布、暖机那次发布各自的记录各两写，断言消息本身只是把十段顺序串起来，不影响模型据此判断哪一段对应哪条变异） |

## 六、检查方法

逐条对照用的是原文文件里现查到的中文原文（本表「原文文件:行」一列的行号，都是 2026-09-17 现读 crates/ 与 crates/mutations.tsv 得到的，不是抄第三方转述）；每一条英文先摘出对应的中文原句，逐词核对有没有漏译的限定词，再看英文有没有多出原文没有的限定词或括注——多出来的一律在「首稿缺的 / 多的」一列写明为什么加、加了会不会改变判据表要用到的事实。全表十条变异标签、九条测试文档注释、两条 DiskSnapshot / 录制流文档注释、两条上下文函数内注释、一条断言消息串译文，逐条核过，无遗漏未列出的翻译段落。
