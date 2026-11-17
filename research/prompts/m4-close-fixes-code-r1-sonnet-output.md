# m4-close-fixes-code-r1 正推腿报告（Sonnet，2026-11-09）

批次：records/2026-10-13-里程碑四预开发.md#收口修补二批。立场：从 D4（校验和位置） 已定项 4、C946（只坏一份的读没有计数也没有搬迁修复）与 `crates/` 今天的实现推「应该是什么」，再与改动逐格比。
只读；没往统一任务服务交任务；没起脚本、没跑测试。判定词：一致 / 冲突 / 规则没说。数标「量的」（命令数出来）或「推的」。
行号：kb 行号是 kb 文件自己的行号（`grep -nF` 现取）；代码行号是工作区今天的文件行号（`grep -nF` 现取），与 diff 全文里的块对应，不是 diff 全文的行号。

## 条款原文与出处（整行抄的只有 C946 一行以外都是片段，片段在所在行里 `grep -nF` 命中一次）

- D4（校验和位置） 已定项 4，`.claude/kb/decisions/04-校验和位置.md` 第 71 行（定案）与第 73 行（射程）。用到的片段：
  - 71：读到只坏一份时计数；可写挂载里读到的，再经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离，走同一个事务层，不开第二条写路径
  - 71：挂载这一串里读到的，在挂载之后第一次用户改动发成之后搬，挂载之后一直没有用户改动就不搬；冷恢复读与只读挂载只计数、不搬
  - 71：两份以最新那一版里指着这个单元的指针为准
  - 73：可写挂载里的读都计数并搬，包括部分写与截断的读回、可写挂载时建上一版那一读、可写挂载期间别的读
  - 73：挂载这一串里读到的（可写挂载时建上一版那一读），挂载交回之前记下
  - 73：并进那次改动之后另发的那一批搬迁
  - 73：被空间拒时同样推抬 F 再试一遍
  - 73：那一批搬没搬成都只试这一次，结局随那次改动报出，之后的用户改动不再接着搬
  - 73：豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，不进计划
  - 73：冷恢复读与只读挂载只计数、不搬，只读挂载读到只坏一份不修
- C946，`.claude/kb/checks-owed.md` 第 589 行（一行里的三个片段）：
  - 读到一份坏之后没有代码把它修掉
  - 部分写读回、截断尾单元读回、挂载态 `read_at`、冷恢复读各跑一次，读出的都是原内容、计数各加一，之后这个单元落在新槽、两份新写、旧落点隔离
  - 计数的落点与取数接口；已有搬迁路径（`crates/singlefs-core/src/transaction.rs` 的 `relocation_checked_against_the_version` 起那一段）接上只坏一份的触发
- D19（块指针的结构与宽度预算） 已定项 5，`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 第 97 行标题：#### 已定项 5：位置条目降为提示，中央映射是唯一入口
- I-9.4、I-9.12，`crates/singlefs-checker/invariants/09-inode树结构.md` 第 16 行与第 24 行：
  - 16：打包记录类型 2 的容器内每条记录的 inode 号 ≥ 容器号
  - 24：第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界

## 一、格 1：diff 里的每一块做的是不是 D4 已定项 4 与 C946 说的

diff 全文是 6 个文件块（量的：`grep -c '^diff --git' research/prompts/_m4-close-fixes-code-r1-diff.md`，见末尾「命令与输出」）。下面按块列：文件:行（工作区今天的行）、对着的条款行、判定。

### 1.1 compaction_plan.rs（会话这一侧的挂载期状态）

| 块 | 文件:行 | 条款行 | 判定 |
|---|---|---|---|
| 字段与初值：挂载时读到只坏一份、到挂载交回那一版仍在映射或留存条目里的单元的映射 key | compaction_plan.rs:858、:885 | D4 73「挂载交回之前记下」 | 一致 |
| 读 / 记 / 取走三个方法 | compaction_plan.rs:918、:925、:941 | D4 73「之后的用户改动不再接着搬」：取走即清空（`std::mem::take`） | 一致 |

注：记的方法里「已有同一把 key 就不再记」那一支（compaction_plan.rs:930 起），调用点只有 mount.rs:7910 一处、一次挂载只调一次、传进来的 key 已由认 key 的函数去重（transaction.rs:11741 起），这一支今天走不到（推的：只读了调用点，没跑）。无害，不违条款。

### 1.2 mount.rs（挂载把读到只坏一份的位置条目带到挂载交回）

| 块 | 文件:行 | 条款行 | 判定 |
|---|---|---|---|
| `PreviousVersion` 三个成员各带 `units_read_with_one_copy_failing` 与读、记两个方法 | mount.rs:1250–1311 | D4 73「挂载这一串里读到的（可写挂载时建上一版那一读），挂载交回之前记下」 | 一致 |
| `rebuild_previous_version` 三个成员初值为空 | mount.rs:1448–1458 | 同上 | 一致（管线，不含条款内容） |
| 建完那一版之后把套子交出的位置条目记进 `previous` | mount.rs:9843–9848 | 同上 | 一致 |
| 挂载交回之前按挂载之后那一版认成映射 key、记进挂载期状态 | mount.rs:7904–7917 | D4 73「豁免映射的三类…认不出映射 key，不进计划」 | 一致：认 key 只遍历 `mapped_units` 与留存条目，三类豁免不在其中（transaction.rs:11751、:11764）。挂载之后那一版不是有树的一版时不记（mount.rs:7907 的 `if let Some(...)`）：条款没写这一形，那一形没有数据单元与映射，合理，归「规则没说」 |
| import 一行 | mount.rs:92 | — | 管线 |

### 1.3 mounted_session.rs（首次用户改动发成之后搬）

| 块 | 文件:行 | 条款行 | 判定 |
|---|---|---|---|
| 首次用户改动发成之后取走挂载时记的 key | mounted_session.rs:2596（`let mut published = published?;`）、:2603（`.take_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting();`） | D4 71「挂载之后第一次用户改动发成之后搬」；73「只试这一次」 | 一致：取走在 `published?` 之后，改动被拒不取，下一次发成的改动仍搬；取走后清空，之后的改动不再搬。入口只有 `publish_user_change` 一条（mounted_session.rs 里 `publish_the_pending_changes`、两个 `publish_user_change` 包装都落到它，量的：`grep -n 'self.session.publish_user_change\|self.publish_user_change'` 同上命令段） |
| 另发那一批的计划并进挂载时的 key | mounted_session.rs:2062–2150（并入处 :2112） | D4 73「并进那次改动之后另发的那一批搬迁」「被空间拒时同样推抬 F 再试一遍」「结局随那次改动报出」 | 一致：同一个函数、同一套推抬 F、结局报在同一个返回成员里 |
| 结局成员的名字与文档只写「快照」 | mounted_session.rs:207、:211–219 | D4 73「结局随那次改动报出」 | 一致于条款；名字与文档对不上现状（见 2.2） |
| 公开方法：看挂载时记着、还没交出去的 key | mounted_session.rs:2478–2490 | C946 589「计数的落点与取数接口」 | 一致 |
| 一次改动内同一把 key 只计一次（`add_the_reads_of_the_writer` 与 `keep_the_mapping_keys_not_read_back_before_in_this_change`） | mounted_session.rs:257–316 | D4 71「读到只坏一份时计数」；C946 589 同 | 规则没说：条款不分「一次改动内重读同一份坏副本」。依据是 r3 判决第三节第 6 条（判决不是条款）；kb 写回随这一批收尾（判决第五节） |
| 库单测一条 | mounted_session.rs:6790 起 | — | 测试 |

### 1.4 recovery.rs

| 块 | 文件:行 | 条款行 | 判定 |
|---|---|---|---|
| 套着数的读者按位置条目去重 + 交出计过的位置条目 | recovery.rs:200–231、:299–315 | D4 71「读到只坏一份时计数」；C946 589「取数接口」 | 去重：规则没说（同一份坏副本一次挂载只计一次，依据判决第三节第 3 条）；取数接口：一致 |
| 新套子 `PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`：按提示读到只坏一份、提示不是映射里那两条就不往里报 | recovery.rs:323–420（判在 :414、:417） | D4 71「两份以最新那一版里指着这个单元的指针为准」；D19 已定项 5（97 行）「位置条目降为提示，中央映射是唯一入口」 | 规则没说：D4 不写提示过期时的计数；由 D19 已定项 5 推出（推的）。与 D4 71 的字面「指针为准」措辞有张力，见 1.7 |
| 两处套用：冷走读 / 重建经提示再经映射读树节点与数据单元 | recovery.rs:1018、:1077 | 同上；D4 73「冷恢复读…只计数、不搬」 | 例外同上；「只计数、不搬」一致 |
| 分隔 key：`inode_record_is_not_below_the_separator_key_and_the_container`、叶序判多一个分隔 key 参数、四个读者的调用点 | recovery.rs:6173、:6203、:7010、:3977、:7613、:7799 | I-9.4（16 行）、I-9.12（24 行） | 不归 D4 / C946；与 I-9.4、I-9.12 的字面一致（`largest_before >= separator_key` 报错对应「> 第 i−1 个孩子的 key 区间上界」） |
| 库单测（叶序模块两条、计数去重一条） | recovery.rs:9048 起、:9171 起 | — | 测试 |

### 1.5 mounted_read.rs（只读挂载）

| 块 | 文件:行 | 条款行 | 判定 |
|---|---|---|---|
| `read_at` 拆出 `read_the_data_units_of_the_range`，报错与否都把这次观测并进打开之后的累计 | mounted_read.rs:959–1004、:1014–1085（并入处 :999） | D4 73「只读挂载只计数、不搬」；71「读到只坏一份时计数」 | 一致（只读挂载仍只计数）；报错也计：规则没说，不与「读到就计」冲突 |
| `dereference_data_unit` 按提示读套上新套子，被丢弃的那一报把这一读的计数清零 | mounted_read.rs:1100–1125（清零 :1123） | 同 recovery.rs 套子一行 | 例外：规则没说，同上 |
| `ReadPathObservation` 文档补两句 | mounted_read.rs:86–91 | — | 文档，随例外 |
| `open_pool_for_read` 逐条记录判下界、叶序判交分隔 key | mounted_read.rs:509、:572–585 | I-9.4、I-9.12 | 不归 D4 / C946；与 I-9.4/I-9.12 字面一致 |

### 1.6 transaction.rs

| 块 | 文件:行 | 条款行 | 判定 |
|---|---|---|---|
| `mapping_keys_of_the_units_at_these_locations`：按挂载之后那一版认 key | transaction.rs:11741 | D4 73「豁免映射的三类…认不出映射 key，不进计划」 | 一致 |
| `relocations_of_the_units_read_with_one_copy_failing_while_mounting`：计划 | transaction.rs:11778 | D4 71「再经已有的搬迁路径…整个搬到新槽」；C946 589「已有搬迁路径…接上只坏一份的触发」 | 一致；见 1.7 (e)：只核 key 还在，不核落点还是读坏的那一对 |
| `CopiesOfAUnitRead::WithoutAFailingCopy` 文档补一句 | transaction.rs:11826–11827 | — | 文档，随例外 |
| 部分写读回：按提示读到只坏一份、提示不是上一版映射落点就改记没坏 | transaction.rs:11898–11912（判在 :11909） | D4 73「包括部分写与截断的读回」都计数并搬 | 例外：规则没说，同上；产品路径上走到的是另一形，见 1.7 (b) |

### 1.7 格 1 的结论：哪几处与条款对不上、哪几处条款没说

一致的（条款写了、代码照做）：挂载交回之前记下建上一版读到只坏一份的单元（mount.rs:7904–7917）；挂载之后第一次发成的用户改动之后取走、并进同一批搬迁、同一套推抬 F、结局随那次改动报出、只试一次（mounted_session.rs:2596–2616、:2062–2150）；被拒的改动不取；冷恢复读与只读挂载只计数、不搬（diff 里没有一块给它们加搬迁）；三类豁免认不出 key、不进计划。

对不上的与要交主 agent 的：

**(a) 条款「可写挂载期间别的读」没有对应的实现，这批改动也没碰。** 冲突（对着 D4 73「可写挂载里的读都计数并搬，包括…可写挂载期间别的读」）。实现里可写会话从盘上读单元的另一处是另发搬迁时读「只被快照钉着、映射条目留存」的单元：transaction.rs:11177（`RelocatedUnitHolder::KeptForTheSnapshots => {`）起，:11182 `.find_map(|location| {`，取先核得上校验和的那一份，坏的那一份不计数。这一读读到只坏一份时：单元被搬走（有「搬」），没有计数（缺「计数」）。第一批报告把它归成「会话里别的读不经数的那一层」（`m4-close-code-r3-fix-writer/result.md` 第 193 行，片段「会话里今天数「只坏一份」的只有读—改—写读回旧单元那一处」）。这一块不是这批 diff 的块，是 diff 没罩住的条款句。推翻它：证明留存单元那一读在到达 :11182 之前已经被别的套子数过（我没找到，推的）。

**(b) 部分写读回那一判（transaction.rs:11909）在产品路径上走不到的是「提示过期」，走得到的是「映射里查不到」，走到时少计一次。** 冲突（对着 D4 71「读到只坏一份时计数」，窄角）。追法：
- 第二批报告说 11909 在产品路径上走不到提示过期那一支（`m4-close-code-r3-fix-writer-2/result.md` 第 180 行，片段「部分写读回那一判在产品路径上走不到「提示过期」那一支」，推的）。我追了：从盘上重建一版后，重建出来的指针一律被换成映射位置（recovery.rs:3714 `take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {`；整理搬迁那一批同样，transaction.rs:15121）；换的写法是映射里有这把 key 才换（转述；transaction.rs:3881 `if let Some(locations) = position_of_key(&mapping_key_for_data(`），没有这把 key 的指针保持盘上读来的提示。会话自己发的版本里指针与映射条目同一次写出。所以「提示≠映射落点」只在 key 不在映射里时成立，此时 `locations_of_role(role)` 返回 `NodeMalformedOrNoEntryWithThisKey`，11909 走进去、把这一读改记「没坏」。
- 这样的版本能挂载：冷走读 / 重建对「提示读不出且映射里没有」报的是内容为空、挂载照常（recovery.rs:3855 起的注释「提示与映射都读不出时字节是空的、不报错」）；提示读得出而映射里没有这把 key 的更不拦。
- 后果：这个单元一份坏、一份好、映射里没有它的 key：部分写读回读到好的那份，计数 0（D4 71 要计 1），也不进搬迁计划；但部分写本身整个重写这个单元，修复照做。相邻的 `None` 分支（提示两份都读不出）在同一缺口上返错（`UnreadableAtTheHintsAndMissingFromTheMapping`），两条路对「映射里没有」的态度不一致。
- 同形的丢弃还在 recovery.rs:417（`Ok(Some(_) | None) | Err(_) => self.one_copy_failing_note_dropped.set(true),`）与挂载态 mounted_read.rs:1123：映射里没有这把 key 或查映射判红时一律不计。
- 这几处是不是要计，取决于「提示是唯一已知位置时算不算权威」：D4 71 的「指针为准」字面说算；D19 已定项 5（97 行）说位置条目只是提示。条款没分，交用户 / 主 agent 定。
- 推翻它：找到一条路径让重建出来的一版里「有 key 却没被换成映射位置」，或证明 key 不在映射里的版本到不了部分写（我没找到，推的）。

**(c) 例外本身（提示过期的读到只坏一份不计）D4 / C946 没写。** 规则没说。五处：mounted_read.rs:1100–1125、recovery.rs:323–420（套用 :1018、:1077）、transaction.rs:11898–11912，加两处文档（mounted_read.rs:86–91、transaction.rs:11826–11827）。基础是 D19 已定项 5（97 行）与 r3 判决第三节第 4 条（判决，非条款）。代码注释引的是 D19（块指针的结构与宽度预算） 已定项 8（mounted_read.rs:1100 的注释），权威的条款是已定项 5，已定项 8 是「哪些指针进映射」（19-…md:181 标题）：引错了编号（量的：两处标题行都 `grep -n '#### 已定项' ` 过）。

**(d) 去重三处（一次挂载同一份坏副本一次、一次改动同一把 key 一次、报错的 read_at 也并入累计）D4 / C946 没写。** 规则没说。依据只有 r3 判决（第三节第 3、5、6 条）。三处的口径彼此不同：只读挂载每次 `read_at` 都计（不去重，mounted_read.rs:999）；挂载建上一版按位置条目去重（recovery.rs:299–315）；一次改动按映射 key 去重（mounted_session.rs:304–316），改动被拒后调用方原样重试会再计一次（`OldDataUnitReadsBackWithOneCopyFailingInThisChange` 每次 `publish_user_change` 新建，mounted_session.rs:2581–2582）。这是口径不统一，不是矛盾：三处各是独立的计数器，不会在一个数上互相扣减（推的：读了三处的累加点）。

**(e) 挂载记的 key 到首次改动之间不核落点。** 规则没说（推的）。`relocations_of_the_units_read_with_one_copy_failing_while_mounting`（transaction.rs:11778）只问 key 在不在这一版映射里，不核映射里的落点还是不是挂载时读坏的那一对。挂载与首次用户改动之间若整理（后台入口，定时叫起不依赖用户改动）已把这个单元搬到好的新槽，首次改动之后仍会再搬一次（多一次无害的搬迁，不丢数据）。没有代码比对：`grep -n 'locations' transaction.rs` 在 11778–11821 一段只有取源落点的用法（量的，见末尾）。

**(f) C946（checks-owed.md:589）这一行与 D4 71 / 73 互相冲突，且描述的是改前的实现。** C946 的用例写「挂载态 `read_at`、冷恢复读各跑一次…之后这个单元落在新槽、两份新写、旧落点隔离」，D4 71 / 73 写冷恢复读与只读挂载「只计数、不搬」；diff 照 D4 做（没有给只读挂载、冷恢复读加搬迁）。C946 行里「实现里…一份读错就读另一份，不计数」「读到一份坏之后没有代码把它修掉」也已不是现状。kb 写回在判决第五节，随这一批收尾，不在我的写范围。

**(g) 分隔 key 的块（recovery.rs 6143–6232 与三个调用点、mounted_read.rs:509、:572–585）不归 D4 / C946**，归 I-9.4（16 行）与 I-9.12（24 行）；两条的字面与代码一致。顺带两处观察，不在这两格里判：
- 返回的 `invariant: "I-9.4"` 同时罩「记录号小于分隔 key」，而分隔 key 那一半写在 I-9.12 第二句（recovery.rs:6173 起的文档自己写明）；挪出来的判法沿用旧标号。
- `inode_records_of_the_leaf_ascend_after_the_leaves_before` 的文档说「没有记录的叶由调用方另拒」，并据此说「条目按分隔 key 严格递增由这两句合出来（叶都不空）」。调用方里只有重建那一处拒空叶（recovery.rs:3969 `        if records.is_empty() {`）；冷走读（:7613）、快照冷读（:7799）、只读挂载（mounted_read.rs:582）三处我没找到空叶的拒绝（`grep -n 'records.is_empty()' recovery.rs mounted_read.rs`，量的，见末尾）。空叶时叶序判在函数头就返回，分隔 key 那一句不判。推的：四个读者对「空叶」不是一个结局。不在两格范围，交主 agent。

### 1.8 格 1 对攻击面第 2、3 条的回答

- 攻击面 2（`hints_are_the_mapping_position`）：走得到的是「映射里没有 key」那一形（上文 (b)），不是「提示过期」；两种非 `Found` 的结果记 false，走到时计数少一次，对着 D4 71 不对。`EntryNarrowerThanItsFieldTable` 一形：重建路径遇条目窄于字段表会在读映射时判红拒挂载（recovery.rs 的 `EntryWidthDiffersFromItsFieldTable`），所以会话里的版本到不了（推的）。
- 攻击面 3（「可写挂载期间别的读」那一句）：写法与实现对不上，见 (a)。另外这一句的前提「可写挂载里的读」：`MountedSession` 对用户没有文件读接口（`grep -n 'pub fn' mounted_session.rs` 里没有读文件的方法，推的），所以「别的读」只能是会话自己读盘的几处：另发搬迁读留存单元（(a)）、释放之前读盘核（transaction.rs 里 `copies_of_a_released_unit_failing_the_release_checksum_check`，它记隔离，不记「只坏一份」的次数）。

补注（格 1.3）：`publish_the_change_raising_the_floor_for_space` 除了真发布之外还在推环预演里被调一次（mounted_session.rs:1775 起，传的是一个新建的空账 `OldDataUnitReadsBackWithOneCopyFailingInThisChange::nothing_read_yet()`，注释写预演不并进累计）。预演不经 `publish_user_change`，不取走挂载时记的 key：与 D4 71「第一次用户改动发成之后」一致（量的：该处只有这一个调用点，见末尾「真发布的入口」一条）。

## 二、格 2：第四节检查单八格，逐格回「成立」「不成立」「推不出」

### 2.1 走不到的理由二次查实 —— 不成立

- 两份实现员报告都没有「走不到的理由」一节（量的：两份各 0 命中，见末尾）。检查单要求的「报告里列全每一处」这个前提不成立。
- 第二批报告在「停下交主 agent」第 1 条写了一处走不到：transaction.rs:11909（`if !hints_are_the_mapping_position {`）「提示过期那一支」产品路径走不到（第二批报告第 180 行）。逐个追调用点：
  - 部分写、截断、变长读回的上一版有两个来源：会话自己发出的版本（指针与映射条目同一次写出，提示 = 映射位置）；从盘上重建的版本（recovery.rs:3714 把指针盖成映射位置，盖的前提是映射里有 key，transaction.rs:3881）。所以「提示过期」确实走不到。成立（推的，读了三处，没造镜像跑）。
  - 但这一判不是死代码：映射里没有 key 时指针保持盘上的提示，`locations_of_role` 交回 `NodeMalformedOrNoEntryWithThisKey`，11909 走进去、把一份真的「只坏一份」改记没坏（见 1.7 (b)）。走到时计数少一次，且那一刻没有错误成员、没有落盘之前返回的动作，只是把计数悄悄压成 0；判定（`copies_read`）与后面的写读的是同一份输入（`previous`），所以不会和后面的写矛盾，但少计是条款字面（D4 71「读到只坏一份时计数」）不允许的。判：「走不到」这个理由对「提示过期」成立，对这一判整体不成立。
  - 文档把这一支写成「提示过期」（transaction.rs:11826–11827），实际可达的触发是「映射里没有」，文档与可达性不符。
- 实现员报告「改法逐处」里没有别的写成走不到的分支。diff 里的 `expect` 与断言见 2.2。

推翻「不成立」：产出一个部分写会话版本，在 key 不在映射的前提下读回一份坏一份好的单元仍计 1（即另有地方补计），或证明 key 不在映射的版本不可挂载。

### 2.2 类型与名字 —— 不成立

逐项（新加的名字在 diff 全文，行号为工作区今天的行）：
- `inode_records_of_the_leaf_ascend_after_the_leaves_before` 多的分隔 key 参数（recovery.rs:6203 起，参数名 `separator_key_of_the_leaf`）：四个调用点（recovery.rs:3977、:7613、:7799，mounted_read.rs:582）传的都是同一轮循环里刚解出的那条内部条目的 `separator_key`（读代码确认，没有跨条目串号）。名字说出内容，成立。
- `inode_record_is_not_below_the_separator_key_and_the_container(record_inode: u64, separator_key: u64, container: u64)`（recovery.rs:6173）：三个不同语义的量都是裸 `u64`、相邻排列。仓里有 `InodeNumber` 新类型（address.rs:52），本函数不用；`separator_key` 与 `container` 在判式里对称（`record < separator || record < container`），两者传反不会被任何用例看出（推的：单测传 (234,235,234)、(234,234,235)，两种传反结果相同）。检查单要求「同一个量不同语义不共用裸整数」：打中一处（轻）。
- `PoolReaderNotingOneCopyFailingOnlyAtTheMappingPosition`（recovery.rs:323）：名字说得出做什么；`pub(crate)`；转发全部九个报告方法（recovery.rs:350–420，与 `PoolReader` 的九个方法逐个对过，量的：末尾「套子转发的方法数」一条）。成立。
- `PoolReaderCountingUnitReadsWithOneCopyFailing` 的去重（recovery.rs:299–315）：调用顺序是先转发给里面一层、再去重计数；临时借用在 `if` 条件结束时放掉，再 `borrow_mut`，不会重入（读代码，推的）。成立。
- `OldDataUnitReadsBackWithOneCopyFailingInThisChange::keep_the_mapping_keys_not_read_back_before_in_this_change`（mounted_session.rs:304）：名字说「记上没读回过的」，实际还交回一个 `u64`（读回过的个数），名字没说交回什么。轻度名实不符，打中一处（轻）。
- `read_the_data_units_of_the_range`（mounted_read.rs:1014）：成立；`# Errors` 与 `# Panics` 写清。
- `expect` 与断言依赖的不变量，逐处：
  - mounted_session.rs 的 `.expect("一次发布读回的单元数装得进 u64")`：依赖 usize→u64 无损，成立。
  - mounted_session.rs 的 `assert_eq!(len(keys), count)`（「写入口上只坏一份的读回次数与映射 key 一次一把」）：依赖写入口两份账只在 `read_back_the_old_payload_counting_a_failing_copy` 一处同时加（transaction.rs:11672–11682；字段只在 transaction.rs:371–376、:414–415 声明与初始化）。我读了字段的全部写点，只这一处。成立（推的）。这是 `assert!` 不是 `debug_assert!`：账目将来分家时产品路径 panic 而不是少计，条款没定，写明即可。
  - 紧接着的 `checked_sub`：依赖「已读回过的个数 ≤ 本次发布的个数」，由上一条的相等断言保证；`.expect("一次改动里只坏一份的读回次数装得进 u64")` 的文字只说装得进，没提减法的依赖。轻。
  - `locations_of_role` 里 `.expect("进映射的单元每个一把 key…")`（transaction.rs:5045）：11909 的新走法让数据角色在「只坏一份」时也走到这里，不只是 `None` 分支；前提「数据指针对应的角色在 `mapped_units` 里」在 `None` 分支已经承担，成立。
  - 新的两个计划函数没有 `expect`/`unwrap`（transaction.rs:11741–11821，读全函数，量的：用 `let ... else`、`continue`）。成立。
  - 11741 的 `central_mapping.locations_of_key(mapping_key)` 可 panic：key 走到的叶角色不在 `units` 里（transaction.rs:5150 起的 `# Panics`）。mount.rs:7907 起对挂载之后每一版新调一次（位置条目非空时），对重建 + 暖机之后的版本该前提已被整理读版本的同一函数承担（mounted_session.rs:3968 `central_mapping_as_the_compaction_reads_it`）。成立（推的）。
- `UserChangePublished::relocation_of_the_old_data_units_kept_for_the_snapshots` 与枚举 `RelocationOfTheOldDataUnitsKeptForTheSnapshots`（mounted_session.rs:207、:218）：名字、字段文档（:205–206）、枚举文档（:211–216）、`NotNeeded` 文档（:219 `/// 不另发：这次改动读回旧单元时没有只坏一份的，或只坏一份的那几个在改动里已经释放（没有活快照钉着）。`）都只说快照；现在同一个成员也报挂载时读到只坏一份的那一批。第一批报告「停下交主 agent」第 1 条已知（`mutations.tsv` 锚点原因没改名）。名字没说对，打中一处。

补一条（不在八格里）：diff 的注释里有 20 行带日期、定案人或判决出处（量的：`grep -nE '^\+.*(用户 2026-|判决 ` 开头的 `research/prompts|此前)'` 命中数，见末尾）；`.claude/agent-common.md`「代码与脚本的注释只写现状」归门禁 doc-text 的 comment-history 格判，不在检查单里。

### 2.3 不兼容旧代码与旧格式 —— 成立

diff 全文里 `legacy|old_format|migrat` 命中 0（量的）；新增行里 `旧版|旧格式|旧布局|旧库|旧镜像|兼容|legacy_ok` 命中 0（量的）。没有认旧版本的分支、旧布局号、迁移路径、只为旧数据放宽的读法。`PreviousVersion` 多的字段是运行期内存，不落盘。推翻：在 diff 之外的改动里找到这类分支（本格只查了 diff 全文）。

### 2.4 不新建判定库 —— 成立

diff 全文里 `sqlite|rocksdb|RocksDB|\.tsv|prune|File::create|fs::write|OpenOptions` 命中 0（量的）。新增存放都是进程内内存：`Vec<Vec<u8>>`（compaction_plan.rs:858）、`RefCell<Vec<[LocationEntry; 2]>>`（recovery.rs:206）、`Vec<[LocationEntry; 2]>`（mount.rs:1258 起）。第二批两份测试文件里的新增（进度线程、`/proc/self/status`）不写判定。推翻：这些内存状态被序列化进某个判定存放处（没找到）。

### 2.5 所有任务多机可用 —— 成立

查了两份测试文件：`crates/singlefs-verification/tests/data_unit_read_with_one_copy_failing_is_counted.rs` 与 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`。
- 前一份：池、盘都是内存里的 `SparseBlockDevice`（该文件 :75、:122），没有文件、环境变量、主机名、地址（量的：IP/主机名/`std::process::Command` 样式命中 0，见末尾）。
- 后一份：这批新增读本机状态的只有 `/proc/self/status` 的常驻内存与峰值（:3833 `    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {`），读不出写「读不出」，只进进度行、不进判定（:3830 起的文档）。该文件里命中「主机/进程」样式的另一处是 `nvidia-smi`（:5658），在量显存的辅助函数里，`git log -S` 查到的引入提交早于这批（baefc645a），不在这批改动里（推的）。
- GPU 对拍测试没上卡时报「本次未跑」、不是通过也不是红，这是该文件的既有约定，不是这批新加。
推翻：新增的进度线程或 `/proc` 读取的结果被用进了判定（没有，`resident_and_peak_mebibytes_of_this_process` 只在 `eprintln!` 里用）。

### 2.6 测试优先多线程 —— 成立

- GPU 对拍：CPU 判器线程数取登记值（该文件 :399–424，没设变量取 `.claude/heavy-thread-default`，既有）；新加的进度线程每 5 分钟打一行（`PROGRESS_INTERVAL_OF_THE_SLICES` = `Duration::from_secs(5 * 60)`，:4839；`recv_timeout` 在 :3813），行里有区间与 CPU / GPU 各判了几个（:3819–:3826）；进度线程放在 `std::thread::scope` 里，判完或中途 panic 都放下发的一头收手（:3735–:3792）；计数器 `AtomicU64`、逐状态回调里 `fetch_add`（:3886），跨线程安全。
- GPU 那一侧按块依次判（:3914–:3915），一张租来的卡一次一块，这是显存额度排队的既有结构（`lease_card_or_report_not_run`），不是漏并行（推的：卡内并行在内核里）。
- `data_unit_read_with_one_copy_failing_is_counted` 各条单池单场景、互相独立，由 cargo 按服务给的线程数并行。
推翻：进度线程在判完之后不收手（会让测试挂住）——`drop(stop_the_progress_lines)` 之后 `join`（:3788–:3789）已覆盖；本格没跑，待主 agent 的 GPU 对拍整目标。

### 2.7 带 --item 的命令与项 —— 不成立

- 第一批报告第 7 条（`m4-close-code-r3-fix-writer/result.md` 第 176 行起）写：项名「我拿不准，先列项再点名」，变异那条的 `--item` 写成占位 `--item '<把 patch/mutations-append.tsv 第一段 23 个名字逐个写成 --item>'`（第 179 行），不是能交的命令；脚本名 `checker-tier-crates-mutation-replay.sh`、`checker-tier-formal-verification.sh` 今天不在 `.claude/gate.d/`（量的：`ls .claude/gate.d | grep -c checker-tier` = 0），crate 名 `singlefs-harness`、`singlefs-checker-tier` 今天不在 `crates/`（量的：`ls crates`，见末尾）。命令缺项：打中。
- 第二批报告的第 6 条按完整变异名逐项 `--item`，带 `SINGLEFS_RUN_SCOPE`，起在 `<根>`；四个名字今天在 `crates/mutations.tsv` 里都有（6910–6913，量的）。
- 表里这一批共 27 行（量的：`grep -c m4-close-code-r3 crates/mutations.tsv` = 27 = 23 + 4），行号今天是 6801–6823 与 6910–6913；派发材料写的「第 6822–6824、6921–6924 行」今天对不上（GPU 恢复内核三行是 6821–6823，第二批四行是 6910–6913），表在被别的批改，行号会漂，按名字点。
- 「新项能被各条的 `--list-items` 列出」：我不跑门禁（共用约束），待主 agent 跑（见「要主 agent 跑的命令」第 1 条）。
推翻：那条 `--list-items` 把 27 个名字全列出且第一批的变异已按名字交过（派发材料说 6822–6824 已在任务 9518、9632 跑过，我没读那两份日志，推的）。

### 2.8 新崩溃流多罩了什么 —— 成立（有一条弱点）

第二批两条判退坏叶流（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`:21155 起、:21181 起）：
- 比已有的两条判退并集流（:20950 的「建上一版拒收」、:20975 的「打开拒收」）多罩的：同样「并集里只有一样读者拒收内容」，但拒收的原因换成这批新加的分隔 key 判（I-9.12 第三句），所以 GPU 两样读者（打开、建上一版）里新加的那一判各被单独罩住；改坏任一判，并集只剩字节丢了，GPU 退到 V，与 CPU 分家。写得出，成立。
- 弱点：「states_staying_on_r」的实值只有任务 9535（缩域到 17 时 9）一个量的数（派发材料给的，我没读日志）；全量域没跑，对应的变异行（6912、6913）没证红。

其余三条流，我按「这一批新加的、能叫流或端到端场景的」取这三条：
- 第一批 GPU 对拍流 `two-inode-leaves-separator-key-of-the-right-leaf-lowered`（该文件 :12610 起测试、:12629 流）：与已有的 `new-pool-new-inode-leaf-records-swapped`（:10729、:12593）同一个建池前言（234 个 inode 两片叶），多罩的是「根上右片分隔 key 改低、记录本身照叶序严格升序」这一形——已有那一条违反的是片内次序，不碰分隔 key 的判；落在最后一次发布的根上，三样 GPU 读者读到右片都应拒在 I-9.12。成立。
- 端到端 `a_partial_write_refused_for_space_and_published_after_raising_the_floor_counts_the_unit_read_back_with_one_copy_damaged_once`（`data_unit_read_with_one_copy_failing_is_counted.rs`:1889）：已有的「被空间拒」用例（:887、:937）都是快照下另发那一批被拒，没有「用户改动自己被拒、推抬 F、重发」又读回同一份坏副本这一形。多跑的是推抬 F 重发那一步的重读。成立。不是崩溃枚举，一个手造的盘面。
- 端到端 `a_writable_mount_falling_back_to_a_version_it_cannot_build_counts_the_damaged_copy_read_by_both_builds_once`（同文件 :1951）：多跑的是挂载里「判退到的那一版建不出、留在所选那一版、两趟共用同一个套着数的读者」这一步。成立。弱点：第二批报告自己写了没有一行变异能只钉「两趟」而不连带关掉单趟去重（`m4-close-code-r3-fix-writer-2/result.md` 的「证红期望」表下一段），所以它比单趟的 `a_writable_mount_counts_one_damaged_copy_of_the_instance_table_once`（:1477）多罩的那一半没有变异兜着。
推翻：把「三条」换成别的三条之后，其中一条写不出比已有流多罩什么——主 agent 若指的是另三条请告诉我，这一格要重判。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 格 1 逐块：挂载记 key、首次发成的改动之后取走并进同一批、只试一次、豁免三类 | 一致 | mount.rs:7904–7917、mounted_session.rs:2596–2616、:2062–2150、transaction.rs:11741、:11778 对着 D4 71 / 73 |
| 格 1 逐块：冷恢复读与只读挂载只计数、不搬 | 一致 | diff 里没有给它们加搬迁；C946（589 行）的用例要求它们也搬，与 D4 冲突，kb 行待写回 |
| 格 1：「可写挂载期间别的读」（D4 73） | 冲突 | 另发搬迁读留存单元（transaction.rs:11177–11190）读到只坏一份时搬了、没计数；diff 没碰、没罩 |
| 格 1：部分写读回 transaction.rs:11909 与同形的两处丢弃 | 冲突（窄角） | 走得到的不是「提示过期」而是「映射里没有 key」，走到时少计一次，对着 D4 71 |
| 格 1：提示过期不计、去重三处、报错的 read_at 并入累计 | 规则没说 | D4 / C946 没写；依据是 D19 已定项 5 与 r3 判决，kb 写回随这一批 |
| 格 1：挂载记的 key 不核落点 | 规则没说 | 挂载与首次改动之间整理已搬走的，会被再搬一次（推的，无害） |
| 格 1：分隔 key 的块（recovery.rs、mounted_read.rs） | 不归 D4 / C946 | 归 I-9.4、I-9.12，与字面一致；另见 1.7 (g) 的空叶观察 |
| 格 2.1 走不到的理由二次查实 | 不成立 | 两份报告没有该节；11909 对「提示过期」走不到成立，但那一判可经「映射里没有」走到并少计 |
| 格 2.2 类型与名字 | 不成立 | 三个裸 u64 同形参数、`keep_…` 名没说交回个数、结局成员名与文档只说快照；其余 `expect` 与断言依赖成立 |
| 格 2.3 不兼容旧代码与旧格式 | 成立 | diff 内两条 grep 命中 0 |
| 格 2.4 不新建判定库 | 成立 | diff 内一条 grep 命中 0，新增存放都是进程内内存 |
| 格 2.5 所有任务多机可用 | 成立 | 新增只读 `/proc/self/status` 进进度行，不进判定 |
| 格 2.6 测试优先多线程 | 成立 | 进度线程 5 分钟一行、线程数取登记值；整目标待主 agent 跑 |
| 格 2.7 带 --item 的命令与项 | 不成立 | 第一批报告命令是占位、指向今天不存在的脚本与包；第二批完整；`--list-items` 待主 agent 跑 |
| 格 2.8 新崩溃流多罩了什么 | 成立 | 写得出各自多罩什么；弱点是挂载退版那条没有只钉「两趟」的变异，全量域没跑 |
| 攻击面 2（11909 一判走不走得到） | 打中 | 见 1.7 (b)、2.1 |
| 攻击面 3（「可写挂载期间别的读」） | 打中 | 见 1.7 (a)、1.8 |
| 攻击面 4（只试一次、结局报在名字只说快照的成员里、豁免三类） | 前两项一致、名字不对 | 一次与豁免成立；成员名与文档只说快照（2.2 末项） |
| 攻击面 5（read_at 报错并入累计与同一把 key 只计一次） | 口径不统一、不矛盾 | 三个计数器各管各的（1.7 (d)） |

## 四、要主 agent 跑的命令

1. 核这一批追加的变异行能被 `--list-items` 列出（格 2.7）。命令：

```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=crates-mutation-replay.sh 的 --list-items 列项; 范围=只列项、不起 cargo; 为什么=核这一批追加的二十七行变异（第一批二十三行、第二批四行）能被 --list-items 按名字列出，检查单「带 --item 的命令与项」要这一条' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs
```

在哪个目录起：仓根 `/home/fy5090/code/singlefs`。看什么：`TASK_SERVER_RESULT` 那一行 `exit=0`；`task-server/task log <号> --all` 里含 `m4-close-code-r3` 的行数是 27（`task-server/task log <号> --all | grep -c 'm4-close-code-r3'`），行号是 6801–6823 与 6910–6913；末尾点名的「键对不上」「锚点失效」行里没有这 27 个名字。这一格待主 agent 跑。

2. 坐实 1.7 (b) 的用例（需要实现员写，不是命令）：在 `data_unit_read_with_one_copy_failing_is_counted.rs` 加一条——三单元池，把第 1 个数据单元的映射条目从中央映射里去掉（重封到根），再弄坏它头一份，挂载后经会话部分写这个单元，断言这次挂载的累计是 1。今天的代码预期得 0（transaction.rs:11909 把它改记没坏）。若得 1，我的 (b) 就错了。

## 五、命令与输出（这一次真跑出来的原样）

均在仓根 `/home/fy5090/code/singlefs` 直接起（只读命令）；输出也存在草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r1-sonnet/commands-output.txt`、`commands-output-2.txt`。

```
$ grep -c '^diff --git' research/prompts/_m4-close-fixes-code-r1-diff.md
6
$ grep -c 走不到的理由 <第一批报告>   （一份一条命令，输出只有计数）
0
$ grep -c 走不到的理由 <第二批报告>
0
$ grep -cE 'legacy|old_format|migrat' research/prompts/_m4-close-fixes-code-r1-diff.md
0
$ grep -E '^\+' <diff> | grep -cE '旧版|旧格式|旧布局|旧库|旧镜像|兼容|legacy_ok'
0
$ grep -cE 'sqlite|rocksdb|RocksDB|\.tsv|prune|File::create|fs::write|OpenOptions' <diff>
0
$ grep -n 'self.session.publish_user_change\|self.publish_user_change' crates/singlefs-core/src/mounted_session.rs
5688:        match self.session.publish_user_change(
5917:        match self.session.publish_user_change(devices, change) {
5999:        match self.session.publish_user_change(devices, change) {
$ grep -n 'records.is_empty()' crates/singlefs-core/src/recovery.rs crates/singlefs-core/src/mounted_read.rs
crates/singlefs-core/src/recovery.rs:3969:        if records.is_empty() {
crates/singlefs-core/src/recovery.rs:5993:            || container.records.is_empty()
crates/singlefs-core/src/recovery.rs:7310:        || instance_table.records.is_empty()
$ awk 命中 locations 的行 transaction.rs 11778–11821
11778: pub fn relocations_of_the_units_read_with_one_copy_failing_while_mounting(
11800:         let source_locations = match central_mapping.locations_of_key(mapping_key) {
11801:             MappingLookup::Found(mapped_locations) => Some(mapped_locations),
11807:                 .map(|(_, kept_locations)| *kept_locations),
11809:         let Some(source_locations) = source_locations else {
11815:                 slot: source_locations[0].slot,
$ ls crates; ls .claude/gate.d | grep -c checker-tier
mutations.tsv
singlefs-background-gpu
singlefs-checker
singlefs-core
singlefs-device-library
singlefs-format
singlefs-reference-filesystem
singlefs-replay-prefix
singlefs-unit-graph
singlefs-verification
0
$ grep -c m4-close-code-r3 crates/mutations.tsv
27
$ grep -n 变异名行号 (GPU 与端到端的 7 行)
6821 6822 6823 6910 6911 6912 6913 
$ grep -n '#### 已定项 [58]' D19
97:#### 已定项 5：位置条目降为提示，中央映射是唯一入口
181:#### 已定项 8：全部指针进映射，引用侧按类携带映射 key 的各段
$ TZ=Asia/Tokyo date +%F
2026-11-09
$ 两份测试文件里 IP/主机名/子进程 样式命中数
data_unit_read_with_one_copy_failing_is_counted.rs: 0
crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs: 1
$ 套子转发的方法数（trait 的 fn 行 127–194，套子 impl 的 fn 行 350–421）
trait: 9  套子 impl: 9
$ grep -cE '^\+.*(用户 2026-|判决 `research/prompts|此前)' <diff>
20
$ ls .claude/gate.d | grep checker-tier（无输出即 0 行）
rc=1
```

## 六、没做什么

- 没往统一任务服务交任务，没起脚本、测试、模型、门禁；上面所有「走不到」「少计」的判断都是读代码推的（标了「推的」），没有造镜像跑。
- 没替主 agent 采纳或出判决；攻击面第 1 条（T3、T4 照攻）只在 2.2 与 1.3 里碰到被拒推抬 F 的同路径，没有逐条再攻。
- 没读禁读清单里的文件；没读两份实现员的 `patch/` 目录里的补丁，只读了两份 `result.md`。
- 没核第一批 GPU 恢复内核（`.wgsl`）的改动本身（diff 全文不含 `.wgsl`）。
- 没判格 2.6 的「GPU 对拍整目标」是否按时收手、格 2.8 的 `states_staying_on_r` 实值：这两样要上卡跑，数是派发材料给的（任务 9535），我没读那份日志。
- 没写 kb（D4 已定项 4、C946、I-9.12 的实现状态写回在 r3 判决第五节，不在我的写范围）。
- 没建仓副本、编译目录、模型；草稿目录里只有两份命令输出文本。

## 七、什么现象会推翻这些结论

- 格 1 (a)：证明留存单元那一读在 transaction.rs:11182 之前已被别的套子数过。
- 格 1 (b)、格 2.1：4 号里的用例得 1，或证明 key 不在映射的版本到不了部分写。
- 格 1 (c)(d)(e)：条款里另有一句把提示过期不计、去重写成已定项（我只在 D4 71 / 73 与 C946 589 里找，没有）。
- 格 2.2：把三个裸 `u64` 参数换成新类型、改名后仍被门禁 naming-lint 或 code-source-discipline 报红（我没跑），则这一格要重判。
- 格 2.7：`--list-items` 列不全 27 行，则「不成立」加重。
- 格 2.3 / 2.4：在 diff 之外的改动里找到旧格式分支或新的判定存放处（本轮只查了 diff 全文与两份测试文件）。
