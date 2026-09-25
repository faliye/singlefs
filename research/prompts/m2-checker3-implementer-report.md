# impl-m2-checker3 报告（收口表第 26 行 I-7.9、第 46 行 C480 跟着要做的 I-9.15、实六报告第 ② 件映射回退、第 13 行 N2）

时刻一律 UTC。副本与草稿目录 `/tmp/claude-1000/impl-m2-checker3/`；主工作区一个字节都没改（改在副本里、交补丁）。

## 一、结论

| 件 | 结果 |
|---|---|
| 1 I-7.9 checker 判定 | 做完。只判「抬 F 的根」，拿它之前的根、抬之前的 F、它自己的实例表算上限。**「抬 F 的根」按同一实例里它前一条根认**（实二第八节原文，也是用户 09-24 选的「实二建议」）；按书记员规格的字面（「根环里 txg 比它小的那条有效根」，不限同一实例）实现，固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史两档判红（变异第 5 行实测，第四节）。**kb 那一句要加「同一实例」，交主 agent（第十二节第 1 条）** |
| 2 I-9.15 blocks = ⌈size ÷ 512⌉ | 做完。编号取 I-9.15（I-9 类现用到 I-9.14）。坏镜像只红它；写路径写回 64 的变异让干净镜像红 |
| 3 `rebuild_version` 与 `allocation_records_under_root` 经映射回退 | 做完。`allocation_records_of_version_without_file` **没改**：那一版的映射根恒为空指针、分配记录树根从不进映射，回退无对象（第十二节第 2 条） |
| 4 N2 | 做完。今天红（`Walk(UnitUnreadable { slot: 50180 })`，整池挂不上）、改后绿：重建照抄位置项、那一项字节为空，可写挂载照常，冷走读报 `MappingStillUnreadable`、挂载态读报 `DataUnitChecksumMismatchEverywhere` |

四件的每条新测试都证明过会红（第八节），`crates/mutations.tsv` 要追加的 12 行变异逐行在副本里跑过、点名的测试都红（第九节）。`check.sh` 在补丁打上之后的主工作区现状上停在 clippy，红的 5 处全在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（不是这一轮改的）；其余 clippy、`cargo build --all-targets` 分开补跑都过；全量 `cargo test --all` 按主 agent 指示停掉、留给最后统一跑，停之前跑完的 25 个二进制全绿（第十一节）。

## 二、交付物

- 补丁：`/tmp/claude-1000/impl-m2-checker3/impl-m2-checker3.patch`，只含 `crates/`，打在主工作区 10:48 UTC 的现状上（HEAD `e980a21`，`crates/` 里只有 `e158_root_choice_repair.rs` 一处没提交、不是我的；快照 `main-crates-now/`）；`git apply --check /tmp/claude-1000/impl-m2-checker3/impl-m2-checker3.patch` 在主工作区里跑过、无输出（第三节）。
- 变异行：`/tmp/claude-1000/impl-m2-checker3/mutations-append.tsv`，12 行，六段制表符分隔，追加到 `crates/mutations.tsv` 末尾即可（原文在补丁打上之后的文件里各恰好命中一次，第九节）。
- 报告：本文件。
- 旧底座上的同一份改动（05:26 快照）：`impl-m2-checker3-on-0526-base.patch`，只作记录；变异证明与第一轮全量测试是在它上面跑的（第十节）。
- 续会话（主 agent 10:4x 的消息，说旧会话进程退出、`bqf4dan1a` 的通知没送到）：那一次 `cargo test --all` 的日志 `logs/check-steps-after-clippy.log` 停在「== cargo test --all」、之后没有输出也没有退出码——没跑完，没有结果可用；补丁对主工作区现状重新生成（`e980a21` 之后 `crates/` 与我 08:54 的底座只差 `mutations.tsv` 与 `e158_root_choice_repair.rs` 两个文件，都不在补丁里），`check.sh` 在新底座上重跑（第十一节）。

## 三、这一轮写过的文件

副本 `rebased/`（主工作区现状 + 补丁）里改过的 `crates/` 文件，12 个：
- `crates/singlefs-checker/src/image.rs`（`IMPLEMENTED_INVARIANTS` 41 → 43）
- `crates/singlefs-checker/src/walk.rs`（I-7.9 判定与它用的读者、I-9.15 判定、实例表行解析抽成 `parse_instance_table_row` 两处共用）
- `crates/singlefs-core/src/recovery.rs`（数据单元回退抽成函数、重建与读分配记录经映射回退、N2）
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`
- `crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`
- `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`
- `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`
- `crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs`
- `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs`
- 新建 `crates/singlefs-harness/tests/second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit.rs`（测试目标自动发现，不用改 `Cargo.toml`）

`crates/mutations.tsv` 没改，要追加的 12 行在 `mutations-append.tsv`（变异名见第九节）。草稿目录里另有 `repo/`（05:26 底座上的工作副本）、`mutant/`（变异证明用，自己的 target）、`baseline/`（没打补丁的 05:26 底座，拿来定基线红集与「今天红」）、`rebased/`、`tools/`、`logs/`。

主工作区里 `git diff --stat -- crates litmus` 原样（10:48 UTC；这一轮一个字节都没写进主工作区，这一行是别的会话的）：

```
 crates/singlefs-harness/src/bin/e158_root_choice_repair.rs | 8 ++++++--
 1 file changed, 6 insertions(+), 2 deletions(-)
```

补丁自己的 `git apply --stat` 原样：

```
 crates/singlefs-checker/src/image.rs               |    8 
 crates/singlefs-checker/src/walk.rs                |  362 +++++++++++++++++++-
 crates/singlefs-core/src/recovery.rs               |  257 ++++++++++++--
 .../tests/checker_known_bad_images.rs              |  253 ++++++++++++++
 .../tests/first_transaction_step_seven_layer0.rs   |   12 -
 .../tests/second_transaction_step_four_rollback.rs |    5 
 ...second_transaction_step_three_formatted_pool.rs |   29 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +
 ..._transaction_supplement_three_random_history.rs |    5 
 ...ent_two_rebuild_with_an_unreadable_data_unit.rs |  111 ++++++
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  276 ++++++++++++---
 12 files changed, 1211 insertions(+), 138 deletions(-)
```

## 四、I-7.9 判定怎么读的

代码：`crates/singlefs-checker/src/walk.rs` 第 2144 行 `judge_rollback_floor_raises_against_their_ceilings`（第 3487 行在 `check_pool_image` 里调）；上限第 2052 行 `rollback_floor_ceiling_before_the_raise`、第 2022 行 `rollback_floor_ceiling_from`；读一条根自己的实例表第 1960 行 `instance_table_rows_of_root_without_judging`（只读不判，判定归主走读）；「非空」比的两条根指针第 1910 行。行号是补丁打上之后的。

逐条根 r：
1. **抬 F 的根**：同一实例里 txg 比 r 小的最新那条根 p（第 2158 行）带的 F 低于 r 带的。没有同实例的 p（回退那次发布、新实例的第一条根，或 p 已被环盖掉）不算；一次抬 F 推的几次空发布只有第一条算。
2. **上限**：只用根环里 txg < r 的根；有效 = 按 r 自己指着的实例表不被抛弃 ∧ txg ≥ p 带的 F（抬之前的 F）；每块盘上最新的有效根取小，与第 4 新的非空有效根（不足 4 个取最旧有效根）再取小；非空 = 按 (txg, 实例) 排、跟前一条有效根比 inode 树与 extent 树的根指针，最旧那条跟「两条都没有」比。与 `crates/singlefs-core/src/mount.rs` 的 `rollback_floor_ceiling` 同一套口径，另写一份、不共用代码。
3. **树表读不出的有效根**（F 抬过之后 F 之下的单元被合法回收复用，后来的镜像上常见）：它自己空不空判不了，它后一条也判不了；这几条全算空得上限下沿、全算非空得上沿（上限对非空集合单调）。F ≤ 下沿判成立，F > 上沿判违例，落在两沿之间这条根不判——不按空或非空猜。
4. 一条根都没判到 ⇒ 报不适用，两种理由分开写（没有抬 F 的根 / 有但判不了）。

为什么是「同一实例」：回退那条根带的是恢复算出的 F_生效，按它自己的实例表，当初撑起那个 F 的非空根在被抛弃的时间线上；拿「txg 比它小的有效根」认前一条，回退根就成了「抬 F 的根」，上限算出来低于 F。实测（变异第 5 行把第 1 步换成字面读法，`logs/mutation-05-whole.log`）：固定用例在 Operation(6) 判红，原话「实例 3 txg 12 那条根把回退下界 F 从 0 抬到 6，高于它之前的根算出的抬 F 上限 0：每块盘上最新的有效根 {0: 6, 1: 4}，非空有效根 [6]，空不空判不了的有效根 [3, 4]，最旧有效根 txg 0」；同一个变异还让随机历史快档 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation` 与 `rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms` 红。

环转过之后：抬 F 那一刻算上限用过的最旧几条根会被盖掉；盖掉的是最旧的，重算出的上限只会更宽（抓不到，不误红）。例外是被盖掉那条的后一条原本非空、单独比「两条都没有」时变空——要求那条根的树表里两棵树都没有条目，今天带文件之后的根都有，没造出过。坏盘把中间某条根弄坏的镜像上可能误红，没造过这样的镜像（第十四节）。

坏镜像与阳性对照：`crates/singlefs-harness/tests/checker_known_bad_images.rs` 第 2857 行 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`（从实二草稿改过来）。步 5 那段历史（第 2768 行 `pool_before_raising_the_floor`，上限 11）：
- 抬到 11：一条都不红，I-7.9 判成立；
- 再发一版 E：E 的数据单元落回 mkfs 那片第 0 版树表的槽，txg 0–2 几条根的树表读不出；它们在抬之前的 F（0）之上，算上限要用——两沿都是 11，照样判成立；
- 抬到 12（入口被骗过：交给入口的现行版本里 F 先改成 12，入口按 txg ≥ 12 取有效根，上限恒不低于目标）：**只**红 I-7.9，原话含「实例 3 txg 15 那条根把回退下界 F 从 0 抬到 12，高于它之前的根算出的抬 F 上限 11」与「非空有效根 [3, 11, 12, 13, 14]」。

登记：`crates/singlefs-checker/src/image.rs` 第 37 行 `IMPLEMENTED_INVARIANTS` 41 → 43（加 I-7.9、I-9.15）。层 0 名单：
- `second_transaction_step_zero_layer0.rs` 第 424 行新加 `assert_the_rollback_floor_raise_was_judged`，抬过 F 的三条用例（快档 108 个状态、八线程切片合并、陈旧 tail）调它：I-7.9 至少在一个状态上真被评估过。共用的 `must_evaluate` 名单不放它：同一个函数也给没抬过 F 的脚本用（残留记录那条流）。
- `first_transaction_step_seven_layer0.rs`：I-7.9 进「这条流上一个状态都评估不到」（没抬过 F）。
- `second_transaction_step_three_formatted_pool_layer0.rs`：I-7.9 不进（没抬过 F，注释写明）。
- 随机历史快档 `second_transaction_supplement_three_random_history.rs`：I-7.9 进「至少判绿一次」名单。实测（`logs/modified-test-1.log`，旧底座）随机历史五个取样点 I-7.9 判绿 21 / 467 / 344 / 1962 / 1420 次，崩溃注入四个取样点 2 / 13 / 0 / 13 次，各段「新发现 0」。
- 各处写死不适用名单的断言照改：`second_transaction_step_three_formatted_pool.rs` 四张表、`checker_known_bad_images.rs` 的干净镜像与 `expected_verdict_with_one_record_per_transaction`、`second_transaction_step_four_rollback.rs` 那条回退用例（没抬过 F，报不适用）。

## 五、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）

- 判定：`walk.rs` 第 714 行，走 inode 叶容器时逐条记录判 `read_u64(记录, 48) == read_u64(记录, 40).div_ceil(512)`；偏移与 512 在 checker 里按 D8（核心索引结构） 已定项 6 的字段表另写一份（第 45–51 行），不引实现的 `INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES`。与 I-9.7 同一处、同一批记录，走不到 inode 叶的镜像上报不适用。
- 粒度：只判不等就红，没写「该记录 EIO」那一半——读者（`InodeRecord::parse`）今天不看偏移 48，要不要也判是实六报告第七节 Q1(b) 那一问，条款没写（第十二节第 3 条）。
- 坏镜像：`checker_known_bad_images.rs` 的 `known_bad_images` 加一格（blocks 写 64）；另加第 1908 行 `inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant`，64 / 5 / 7 三个取样值各**只**红 I-9.15，原话逐字比「inode 1 的记录 blocks {值} 不等于 ⌈size 3000 ÷ 512⌉ = 6」。干净镜像上判成立。
- 层 0：`first_transaction_step_seven_layer0.rs` 进「只在新根下面评估」那一类（与 I-9.7 同）；`second_transaction_step_zero_layer0.rs` 与 `second_transaction_step_three_formatted_pool_layer0.rs` 进「必须评估」名单；随机历史快档进「至少判绿一次」。

## 六、`rebuild_version` 与读分配记录经映射回退（实六报告第 ② 件）

依据 D19（块指针的结构与宽度预算） 已定项 8 末句「豁免三类之外的单元（码 1 数据单元与码 2 树节点——extent 树根、inode 树根、inode 叶容器）位置提示读不出时一律经映射回退；豁免三类（映射树根、树表、根记录那条链）不回退。」（kb 正在改，以派发给的规格为准）。

- `crates/singlefs-core/src/recovery.rs` 第 1120 行 `rebuild_version`：extent 树根、inode 树根、分配记录树根、记账树根走 `read_mapped_tree_node_via_hint_then_central_mapping`（索引节点类），inode 叶容器走同一函数（打包记录类），数据单元走新抽出来的第 370 行 `read_data_unit_via_hint_then_central_mapping`。实例表、树表、映射树根照旧只按根记录里的位置条目读。
- 映射树根第一次要回退时才读、读过就留着（第 437 行 `CentralMappingRootWithBytesReadOnFirstUse`，存字节与解开的节点，重建要把字节照抄进上一版）；提示都读得出的镜像上读序与报错次序不变。读法照重建原来读映射树根那一步：只解节点、不核自描述（冷走读那一份 `CentralMappingRootReadOnFirstUse` 另核树 ID 等，没动它）。
- 第 862 行 `allocation_records_under_root`：分配记录树根同一条回退，映射取这条根自己的。
- 冷走读 `walk_to_file` 里原来内联的数据单元回退换成调第 370 行那个函数（第 2308 行），报错成员与计数不变。
- 回退次数在这两个读者里不交出去（`RebuiltVersion`、影子账今天没有接多跳观测点的口子），D19 已定项 5 硬规则 3 在这两处没有观测点（第十二节第 4 条）。
- 用例（`second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs`，原来搬三样的装置扩成六样：再加分配记录树根、记账树根、数据单元）：
  - 第 495 行 `rebuilding_the_previous_version_reads_every_relocated_mapped_unit_through_the_central_mapping`：六样各搬一次（提示指空槽、映射改指新落点），重建做成、那个角色的字节等于搬之前那一份、位置项照抄父指针、分配记录与数据指针与搬之前逐条相等。
  - 第 554 行 `reading_the_allocation_records_of_a_root_goes_through_the_central_mapping_when_the_hint_is_stale`：映射跟着改 ⇒ 读得回、逐条相等；映射没改 ⇒ `MappingStillUnreadable { slot: 提示那个槽 }`。
  - 第 587 行 `rebuilding_with_a_relocated_extent_tree_root_that_the_mapping_does_not_follow_fails_after_the_hop`：树节点不在 N2 的容下之列，重建报 `MappingStillUnreadable`。

## 七、N2：重建时读不出的数据单元

- `rebuild_version` 第 1231 行：数据单元经提示、再经映射都读不出（映射里没有它的 key，或映射落点也读不出）⇒ 不报错，数据指针照抄，`units` 里那一项的字节为空 `Vec`；查映射本身出错（映射树根读不出、条目窄于字段表）照旧报错。第 403 行 `DataUnitReadThroughTheCentralMapping` 把「读到了 / 映射里没有 / 映射落点也读不出」三种交给调用方，冷走读按原来的两个成员报错，重建按 N2 容下。
- 为什么空字节可以：带文件的上一版里数据单元的字节只被照抄进下一版的 `units`（`transaction.rs` 的 `carried_file_version_units` / `carried_unit`），落盘只写这次重写的角色（`written_units`），释放核验按映射条目读盘（硬规则 1），都不读这份字节。这是现查代码得出的，没有类型挡着「以后有人拿它当内容用」（第十二节第 5 条）。
- 用例：新文件 `crates/singlefs-harness/tests/second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit.rs` 的 `data_unit_unreadable_on_both_devices_is_carried_by_its_locations_and_the_writable_mount_goes_on_until_the_file_is_read`。第一个事务那一版上把数据单元两盘都写零（提示与映射指同一个槽）：重建做成、位置项照抄、那一项字节为空；`mount_writable` 照常取号 2、新实例这一版照抄那个数据指针；之后冷走读报 `MappingStillUnreadable { slot: 50180 }`，只读挂载打得开、文件打得开，`read_at` 报 `DataUnitChecksumMismatchEverywhere { 第 0 个单元, 槽 50180 }`。
- 原型对照：攻方原型 `research/prompts/m2-newq-r1-opus-model/patch/apply2.py` 用的是填零的 32768 字节占位；这里用空字节，不装作读到了内容。

## 八、每条新测试「改坏哪一行 → 哪条断言红」

证明分两路：①「今天」——把新测试文件拷进一份没打补丁的主工作区副本（05:26 快照 `baseline/`）直接跑（`logs/new-tests-on-today-code.log`，跑完已还原）；② 变异——在 `mutant/` 副本里按 `mutations-append.tsv` 逐行改坏一处，跑点名测试所在的**整个**测试二进制，拷回原件并 `touch`（`tools/prove_red.py`，日志 `logs/mutation-NN-whole.log`）。`mutant/` 用自己的 target。基线红集：`baseline/` 全量 `cargo test --workspace --no-fail-fast` 只红 `layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape`（`first_transaction_step_seven_layer0.rs` 第 130 行，I-3.10 评估状态数 7 ≠ 4，不在这一轮的改动里）；`mutant/` 不改动时 `checker_known_bad_images` 29 条全过，其余点名二进制在 `repo/` 全量里全过。下表只列基线红集之外的。

| 新测试 | 改坏哪一处 | 哪条断言红 | 同时红的 |
|---|---|---|---|
| `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant` | 变异 1：`walk.rs` 第 2187 行判定改 `true` | `checker_known_bad_images.rs` 那条用例「F 抬到 12（上限 11）：判红的该只有 I-7.9」（全 Holds） | 无 |
| 同上 | 变异 2：第 2033 行 `.min` 改 `.max`（上限改宽） | 同上 | 无 |
| 同上 | 变异 3：第 4 新改第 3 新 | 同上 | 无 |
| 同上 | 变异 4：有效根按抬之后的 F 取（恒真读法） | 同上 | 无 |
| `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`（已有，这一轮要求它不红） | 变异 5：抬 F 的根按字面读（不限同一实例） | `second_transaction_supplement_three_random_history.rs` 第 853 行 `NewFinding { … I-7.9 … }` | 快档、偏回退档两条 |
| `assert_the_rollback_floor_raise_was_judged`（层 0 名单） | 变异 6：一条抬 F 的根都不认 | `second_transaction_step_zero_layer0.rs` 第 425 行「I-7.9 至少在一个状态上真被评估过」 | `stale_tail_…` 同一处 |
| `inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant` | 变异 7：`walk.rs` 第 714 行判定改 `true` | 该用例 + 干净镜像用例第 1887 行「I-9.15 那份坏镜像应当判违例，实际 Holds」 | `the_clean_image_…` |
| 同上 | 变异 8：`div_ceil` 改 `/`（向下取整） | 干净镜像 I-9.15 判红（第 1860 行「干净镜像上 I-9.15 的判定」） | 这个二进制里 24 条（凡是在真镜像上要求零违例的都红） |
| 干净镜像上 I-9.15 成立（`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`） | 变异 9：`records.rs` 写路径写回 64 | 第 1860 行「干净镜像上 I-9.15 的判定」，实际 `Violated("inode 1 的记录 blocks 64 不等于 ⌈size 3000 ÷ 512⌉ = 6")` | 同上 24 条 |
| `rebuilding_the_previous_version_reads_every_relocated_mapped_unit_through_the_central_mapping` | 今天的代码 | 第 525 行「ExtentTreeRoot 提示指空槽、映射里是真落点：重建要做成，实际 Walk(UnitUnreadable { slot: 50240 })」 | — |
| 同上 | 变异 10：重建查映射恒查不到 | 同一行，`MappingMiss { slot: 50240 }` | `rebuilding_with_a_relocated_extent_tree_root_…` |
| `reading_the_allocation_records_of_a_root_goes_through_the_central_mapping_when_the_hint_is_stale` | 今天的代码 | 第 565 行「经映射读回来，逐条相等」，实际 `Err(UnitUnreadable { slot: 50245 })` | — |
| 同上 | 变异 11：读分配记录查映射恒查不到 | 同一行 | 无 |
| `rebuilding_with_a_relocated_extent_tree_root_that_the_mapping_does_not_follow_fails_after_the_hop` | 今天的代码 | 第 595 行，实际 `UnitUnreadable { slot: 50240 }`、期望 `MappingStillUnreadable` | — |
| `data_unit_unreadable_on_both_devices_…_until_the_file_is_read` | 今天的代码 | 第 47 行「重建要照抄它的位置项、照常做成，实际 Walk(UnitUnreadable { slot: 50180 })」 | — |
| 同上 | 变异 12：重建时读不出的数据单元照旧报错 | 同一行，`MappingStillUnreadable { slot: 50180 }` | 无 |

「今天的代码」那几行：同一次运行里这个文件原有的 5 条用例全过（`logs/new-tests-on-today-code.log`）。变异的行号是 05:26 底座上的；补丁打到新底座上 `walk.rs` 行号不变，测试文件行号有偏移。N2 那条用例证明会红时名字还带前缀 `a_`，之后为过命名检查改了名，内容没动。

## 九、`crates/mutations.tsv` 要追加的 12 行（`mutations-append.tsv`）

变异名（按行序；第 5–6 列是 cargo test 参数与必须红的测试）：

1. 收口表第 26 行 I-7.9：判法拿掉（抬 F 的根带的 F 高于上限也判成立） —— `crates/singlefs-checker/src/walk.rs` → `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`
2. 收口表第 26 行 I-7.9：上限算法改宽（每块盘上最新的有效根与第 4 新的非空有效根取大，不取小） —— `crates/singlefs-checker/src/walk.rs` → `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`
3. 收口表第 26 行 I-7.9：上限算法改宽（第 4 新的非空有效根改成第 3 新的） —— `crates/singlefs-checker/src/walk.rs` → `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`
4. 收口表第 26 行 I-7.9：有效根按抬之后的 F 取（上限的「有效」带 txg ≥ 新 F，检查恒真的那一种读法） —— `crates/singlefs-checker/src/walk.rs` → `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`
5. 收口表第 26 行 I-7.9：抬 F 的根不认同一实例的前一条，按它自己的实例表认 txg 比它小的有效根（回退那条根被当成抬 F，合法历史上误红） —— `crates/singlefs-checker/src/walk.rs` → `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`
6. 收口表第 26 行 I-7.9：一条抬 F 的根都不认（层 0 那条抬过 F 的流上 I-7.9 一个状态都评估不到） —— `crates/singlefs-checker/src/walk.rs` → `every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims`
7. 收口表第 46 行 C480：I-9.15 判定恒真（blocks 与 ⌈size ÷ 512⌉ 不等也判成立） —— `crates/singlefs-checker/src/walk.rs` → `inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant`
8. 收口表第 46 行 C480：I-9.15 按 ⌊size ÷ 512⌋ 比（向下取整，写路径写的 6 在 3000 字节上被判红） —— `crates/singlefs-checker/src/walk.rs` → `inode_record_blocks_other_than_the_logical_length_in_512_byte_blocks_reddens_only_the_blocks_invariant`
9. 收口表第 46 行 C480：写路径把 blocks 写回按分到一个 32 KiB 单元算的 64，checker 的 I-9.15 在真镜像上红 —— `crates/singlefs-core/src/records.rs` → `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`
10. 实六报告第 ② 件：rebuild_version 的树节点与数据单元不经映射回退（查映射恒查不到） —— `crates/singlefs-core/src/recovery.rs` → `rebuilding_the_previous_version_reads_every_relocated_mapped_unit_through_the_central_mapping`
11. 实六报告第 ② 件：allocation_records_under_root 的分配记录树根不经映射回退（查映射恒查不到） —— `crates/singlefs-core/src/recovery.rs` → `reading_the_allocation_records_of_a_root_goes_through_the_central_mapping_when_the_hint_is_stale`
12. 收口表第 13 行 N2：重建时读不出的数据单元不容下（照今天整次挂载失败） —— `crates/singlefs-core/src/recovery.rs` → `data_unit_unreadable_on_both_devices_is_carried_by_its_locations_and_the_writable_mount_goes_on_until_the_file_is_read`

锚点：每行原文在补丁打上之后的文件里恰好命中一次（`tools/mutation_rows_rebased.py` 逐行核过，输出「12 行，原文都恰好命中一次」）；`crates/mutations.tsv`（主工作区 13:09 UTC 那一版，520 行；第 520 行是别的会话新加的 E158 那一条，锚在没提交的 `e158_root_choice_repair.rs` 上）的锚点在补丁打上之后也都恰好一次（`anchor_check.py`：「不是恰好一次的：0」）。第 395–398 行（实六加的 C480 那几条，锚在 `records.rs`）不动；第 9 行是同一个写路径变异点名到 checker 这一条，实六报告第七节 Q1 说的「把它的点名测试改成 checker 那一条」按追加做、没改原行。第 5、6、9 行点名的测试不是这一轮新加的那条，它们钉的是「这一轮的改动会被悄悄撤回」那一格：第 5 行撤回「同一实例」，固定用例红。

## 十、受影响的测试在旧底座上的结果

- `baseline/`（05:26 主工作区快照，没打补丁）全量 `cargo test --offline --workspace --no-fail-fast`（05:36–08:12，`baseline-test.log`，退出 101）：红的只有 `layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape`（`first_transaction_step_seven_layer0.rs` 第 130 行「I-3.10 评估过的状态数」left 7 / right 4）。这就是基线红集。
- `repo/`（同一底座 + 改动）全量（06:09–08:37，`logs/modified-test-1.log`，退出 101）：红的是基线那一条，加 4 条写死了「每条不变量都成立」的旧断言（`checker_known_bad_images.rs` 3 条、`second_transaction_step_four_rollback.rs` 1 条，I-7.9 在没抬过 F 的镜像上报不适用）。这 4 处断言随后按「没抬过 F ⇒ I-7.9 不适用」改了，两个二进制重跑全过（`logs/rerun-fixed-binaries.log`：29 过、12 过）。同一次全量里随机历史快档与四个取样点、崩溃注入各档、层 0 快档都过，I-7.9 与 I-9.15 的判绿次数见第四、五节。

## 十一、`check.sh`

在 `rebased/`（主工作区 `e980a21` 现状 + 补丁）上跑，`tools/check_rebased.sh`，日志 `logs/check-rebased-2.log`。

`check.sh` 末尾原样（10:49 UTC）：

```
  ✓ 格式通过
  ✗ clippy 有告警（按 -D warnings 视为错误），或者踩了编码纪律的某一条
```

退出码 1。clippy 报的 5 处全在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（`mounted` / `mut allocator` 同名遮蔽、一处通配臂；这个文件在主工作区里有别人没提交的改动，不在这一轮的补丁里）。之后分开补跑、各记退出码：
- clippy（check.sh 那一组 `-D` 全带上），`singlefs-format`、`singlefs-core`、`singlefs-checker` 全部目标：0；
- clippy，`singlefs-harness` 的 lib、108 个测试目标参数（54 个测试文件）、e158 之外四个 bin（构建与测试两种各一遍）：0、0；
- `cargo build --all-targets`：0；
- `cargo test --all --no-fail-fast`：**没跑完。全量测试留给最后统一跑**（主 agent 13:0x UTC 的指示：用户定全量验证在全部代码落定之后统一跑一次）；按主 agent 给的进程号用 `proc.py stop` 停掉了 `check_rebased.sh`（pid 2060768）、它底下的 `cargo test`（pid 2061409）与正在跑的 `second_transaction_step_five_reuse` 二进制（pid 3893799），13:08 UTC。

已跑完的 25 个二进制结果如下（全绿，0 条红）：

| 二进制 | 过 | 红 | 忽略 |
|---|---|---|---|
| unittests `singlefs_checker` | 3 | 0 | 0 |
| unittests `singlefs_core` | 94 | 0 | 0 |
| unittests `singlefs_format` | 5 | 0 | 0 |
| unittests `singlefs_harness` | 67 | 0 | 0 |
| unittests bin `e156_allocation_basis_counts` | 6 | 0 | 0 |
| unittests bin `e158_root_choice_repair` | 28 | 0 | 0 |
| unittests bin `first_transaction_device_log_check` | 4 | 0 | 0 |
| unittests bin `first_transaction_on_device` | 12 | 0 | 0 |
| unittests bin `first_transaction_region_bytes` | 0 | 0 | 0 |
| `checker_known_bad_images` | 29 | 0 | 0 |
| `first_transaction_region_bytes` | 4 | 0 | 0 |
| `first_transaction_step_five_publish` | 8 | 0 | 0 |
| `first_transaction_step_one_mkfs` | 9 | 0 | 0 |
| `first_transaction_step_seven_layer0` | 5 | 0 | 1 |
| `first_transaction_step_six_recovery` | 6 | 0 | 0 |
| `first_transaction_step_two_data_unit` | 3 | 0 | 0 |
| `instance_acquisition` | 4 | 0 | 0 |
| `parallel_line_one_sequential_write` | 3 | 0 | 0 |
| `publish_order_matches_litmus` | 1 | 0 | 0 |
| `second_transaction_mapping_node_admission` | 4 | 0 | 0 |
| `second_transaction_parallel_line_one_layer0` | 1 | 0 | 1 |
| `second_transaction_parallel_line_one_multi_unit_file` | 7 | 0 | 0 |
| `second_transaction_parallel_line_one_sequential_write` | 3 | 0 | 0 |
| `second_transaction_parallel_line_three_many_inodes` | 4 | 0 | 0 |
| `second_transaction_parallel_line_two_mounted_read` | 11 | 0 | 0 |

我动到的 9 个测试二进制里，只有 `checker_known_bad_images` 与 `first_transaction_step_seven_layer0` 在新底座上跑过（上表；后者在旧底座上的基线红，在新底座上随主工作区的「I-3.10 计数」那份补丁绿了）。另外 7 个——`second_transaction_step_four_rollback`、`second_transaction_step_three_formatted_pool`、`second_transaction_step_three_formatted_pool_layer0`、`second_transaction_step_zero_layer0`、`second_transaction_supplement_three_random_history`（含随机历史快档）、`second_transaction_supplement_two_tree_nodes_and_the_central_mapping`、`second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit`——只在 05:26 底座上绿过（第十节；最后两个在 `repo/` 单独跑过，`logs/new-tests-1.log` 全过；`second_transaction_step_four_rollback` 08:5x 在 `repo/` 重跑过、12 过），**没在 `e980a21` 上重跑**。两个底座的差别（`diff -rq base-crates main-crates-now`）：这 7 个测试文件里只有 `second_transaction_step_three_formatted_pool.rs`（78 行差）与 `second_transaction_supplement_three_random_history.rs`（318 行差）变了；它们调的代码里变了 `singlefs-core` 的 `allocator.rs`、`journal.rs`、`make_filesystem.rs`、`mount.rs`、`recovery.rs`、`transaction.rs`，`singlefs-format`，`singlefs-checker` 的 `lib.rs`、`walk.rs`，`singlefs-harness` 的 `history.rs`、`model_comparison.rs`、`scenario.rs`（其中能看到的一处是实十三把 journal 记录头从 307 改到 311、`recovery.rs` 链首判定多一条「本次发布内序号」）。补丁都打得上，行为上的交互没验。

## 十二、停下交主 agent 的设计问题

1. **I-7.9 里「抬 F 的那一条根」怎么认，kb 那一句要改。** 书记员规格 `/tmp/claude-1000/kb-writeback-0924/spec.md` 第三节第 1 条写「它带的 F 比根环里 txg 比它小的那条有效根带的高」，有效又是「按那条根自己指着的实例表判」——照字面，回退那条根（带恢复算出的 F_生效）的前一条有效根是回退目标，F 低，回退根就成了「抬 F 的根」，合法历史判红（第四节，变异第 5 行实测：固定用例、随机历史快档、偏回退档三条红）。用户 09-24 选的是「实二建议」（`records/2026-09-24-里程碑二收尾调度.md` 第二、三节），实二报告第八节的原文是「r 带的 F 高于同一实例里按 txg 排在 r 前面、仍有效的那条根 p 带的 F；回退或新实例的第一条根没有同实例的 p，不算抬」。我照实二原文写了代码；kb 那一句要不要改成「同一实例」由主 agent 定、交书记员。推翻条件：用户原意确是不限实例——那样固定用例必红，要另定回退根怎么算。
2. **`allocation_records_of_version_without_file` 没做映射回退。** 它读的是树表 0 条那一版根记录直接持有的那片分配记录树节点（C512）。这一版的中央映射根恒是空指针（`crates/singlefs-core/src/make_filesystem.rs` 第 318 行 mkfs 写 `mapping_root: NodePointer::empty_root()`；零单元发布与树表 0 条那一版的写行发布照抄上一版的，`crates/singlefs-core/src/transaction.rs` 第 629 行与第 975 行 `mapping_root: previous_root.mapping_root,`，行号是主工作区 `e980a21` 那一版的），写者也从不把这片节点写进映射——回退没有可查的东西。D19（块指针的结构与宽度预算） 已定项 8 的豁免三类没点它，「一律经映射回退」按字面只能得到 `MappingMiss`。两条路：把「根记录直接持有的节点」列进豁免（条款改一句），或者照字面把报错成员从 `UnitUnreadable` 换成 `MappingMiss`。条款没定，我没改，今天的行为照旧（提示读不出报 `UnitUnreadable`，在挂载的任何写之前）。
3. **I-9.15 读者那一半。** checker 只判「不等就红」；`InodeRecord::parse` 今天不看偏移 48，读到与 size 对不上的 blocks 照常交出记录。I-9 类的通例是「该记录 EIO」（I-9.7、I-9.8），条款（D8（核心索引结构） 已定项 6、C480 用户 09-23 定案）只写了 checker 加一条。要不要读者也判、判了报哪个成员，没条款（实六报告第七节 Q1(b) 同一问）。
4. **`rebuild_version` 与 `allocation_records_under_root` 的多跳次数没交出去。** D19 已定项 5 硬规则 3 要多跳率有运行时观测点；这两处要交出去得改 `RebuiltVersion` 的形状或返回值，调用方在 `mount.rs`（实十五在改），没碰。
5. **N2 那一项的字节用空 `Vec` 表示「没读内容」。** 今天没有路径读照抄的数据单元的字节（第七节），但没有类型挡着。做成类型（例如 `PublishedUnit` 的字节换成「读到的 / 照抄而没读的」两成员）要改 `transaction.rs`（实十六在改）与各处构造，没做。
6. **I-7.9 的两处读法是我定的，请主 agent 过目**：① 树表读不出的有效根不按空或非空猜，取上限的两沿、落在中间不判（沿用 checker 里「判不了，不按空或非空猜」的写法）；② 根环里每一条抬 F 的根都判，不只最新那一次。

## 十三、`invariants.md` 建议的原文（我不改 kb）

**I-7.9 定义列**：在书记员规格第三节第 1 条的新串上只改括号里那一句——
旧：`**抬 F 的那一条根**（它带的 F 比根环里 txg 比它小的那条有效根带的高）`
新：`**抬 F 的那一条根**（它带的 F 比同一实例里 txg 比它小的最新那条根带的高；回退那次发布与新实例的第一条根没有同实例的前一条，不算抬——它们带的是恢复算出的 F_生效；一次抬 F 推的几次空发布只有第一条算）`
后半句「有效 = 按那条根自己指着的实例表判仍然有效 ∧ txg ≥ 抬之前的 F；「之前的根」取根环里 txg 比它小的那些」不动。可再补一句读不出的处置：`算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判`。

**I-7.9 状态列**：
> 已实现（2026-09-24，池级 checker `walk::check_pool_image` 的 `judge_rollback_floor_raises_against_their_ceilings`，根环里每条抬 F 的根各判一格；坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`：步 5 那段历史上抬到上限 11 与之后再发一版都成立、抬到 12 只红它；固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version` 与随机历史各档不红；层 0 `second_transaction_step_zero_layer0.rs` 抬过 F 的三条流要求真被评估过；`crates/mutations.tsv` 6 条变异（判法拿掉、上限取大、第 4 新改第 3 新、有效按新 F 取、不限同一实例、一条都不认）；用户 2026-09-24 定义，里程碑「第二个事务」收口表第 26 行）

**新行 I-9.15**（放在 I-9.14 那一行之后）：
> `| I-9.15 | inode 记录的 blocks 等于 ⌈size ÷ 512⌉ | 打包记录类型 2 的每条记录，偏移 48 的 blocks == ⌈偏移 40 的 size ÷ 512⌉（D8（核心索引结构） 已定项 6：blocks 是逻辑长度的 512 字节块数，不表示分到的空间；C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）；不等判红。读者要不要也判、判了报什么没有条款 | 已实现（2026-09-24，池级 checker walk::check_pool_image 走 inode 叶容器时逐条判，与 I-9.7 同一处；坏镜像在 crates/singlefs-harness/tests/checker_known_bad_images.rs：blocks 写成 64 / 5 / 7 各只红它；层 0 每个走得到 inode 叶的崩溃状态都判；变异：判定恒真、按向下取整比、写路径写回 64；里程碑「第二个事务」收口表第 46 行） |`

**条数两处**：第 12 行「池级 checker（`walk::check_pool_image`）判 40 条」——那句按 `grep -c '^| I-.*已实现' .claude/kb/invariants.md` 数，今天主工作区数出来是 40，而 `IMPLEMENTED_INVARIANTS` 是 41（两边已经差一条，不是这一轮造成的）；补丁打上之后代码是 43，kb 这边 I-7.9 与 I-9.15 改成已实现之后是 42，差的那一条是 I-3.11（代码里有、kb 那一行还没写已实现；实二报告第八节给过它的状态列原文）。第 17 行「现共 75 条在用（编号至 I-9.14…）」→ 76 条、编号至 I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）。

## 十四、什么现象会推翻这些结论

- I-7.9「不在合法历史上误红」：层 0 全量（`full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean`，门禁 54 号，我没跑）、随机历史大档、或坏盘输入 / 故障注入的镜像上出现一条只红 I-7.9、历史本身合法的镜像。已知的风险格：坏盘把抬 F 那一刻用过的中间某条有效根弄坏，重算的非空集合少一条，上限可能变低（第四节）。
- I-7.9「抓得到抬过上限」：把 `raise_rollback_floor` 的上限判定（`mount.rs` 的 `new_floor > ceiling`）拿掉之后，步 5 那段历史抬到 12 的镜像上 I-7.9 不红。我只用「骗入口」造了这一格，没直接改实现跑。
- I-9.15：有写路径写出的 inode 记录 blocks ≠ ⌈size ÷ 512⌉ 而 checker 判成立，或 size 为 0 的记录在别处被要求 blocks ≠ 0。
- 映射回退：搬过树节点的镜像上可写挂载仍报 `UnitUnreadable`；或者拿掉 `rebuild_version` 里的回退之后第六节那三条用例仍然绿。
- N2：某条路径把照抄数据单元那一项的字节当内容用（写盘、算校验和、比对）——那时空字节会变成静默错；或者可写挂载之后的发布把那个读不出的数据单元释放回空闲池而没核（归释放核验那一侧，实十六在改）。

## 十五、没做什么

- 没走三方对抗；层 0 全量、QEMU、herd7、`crates/mutations.tsv` 整表复跑（门禁 59 号）都没跑，层 0 与 crates 变异表归 `crash-verifier`；没提交。
- 登记给我的七个门禁阶段（`33-mutation-tables.sh`、`53-format-const-placeholders.sh`、`74-model-differential.sh`、`92-layout-checker-sync.sh`、`94-checker-implementation-disjoint.sh`、`93-feature-bits.sh`、`89-closeout-row27-preconditions.sh`）按派发没跑。
- `invariants.md`、`checks-owed.md`、里程碑收口表都没改（不写 kb）；`crates/mutations.tsv` 没改，要追加的行在 `mutations-append.tsv`。
- 12 行变异是在 05:26 底座上证明会红的；补丁挪到 `e980a21` 底座之后只核了锚点仍恰好命中一次，没在新底座上重跑变异。
- 全量 `cargo test --all` 没跑完，按主 agent 指示留给最后统一跑；我动到的 9 个测试二进制里有 7 个没在新底座上重跑（第十一节末段）。
- `allocation_records_of_version_without_file` 没做回退（第十二节第 2 条）；两处读者的多跳观测点没接（第 4 条）；读者侧的 blocks 判定没做（第 3 条）。
- 没碰的别的会话的文件：`journal.rs`、`mount.rs`、`allocator.rs`、`transaction.rs` 一个字节都没改。
- 负载：开工时（05:25）`ps` 看到别的会话在跑 `cargo test --offline --all --no-fail-fast` 与随机历史那个二进制，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；机器负载一直在 50–70，每个测试二进制比平时慢几倍，没有等锁（副本各用自己的 target）。

## 十六、中途收到的消息

1. 10:4x UTC，主 agent：旧会话进程退出、续上；核现场、对主工作区现状 `git apply --check`、不加 `&` 等长活。照做：旧的 `cargo test --all` 没跑完（第二节），补丁对 `e980a21` 重新生成、`git apply --check` 过。
2. 之后，主 agent：线程上限 4，新起的编译 / 测试 / 变异命令用 `research/scripts/capped.sh 4`。收到时只有 `check_rebased.sh` 那一串在跑（照指示不停）；之后我没有再起编译、测试或变异命令。
3. 13:0x UTC，主 agent 例行询问：回了在等 pid 2061409（`cargo test --offline --all --no-fail-fast`）、已跑完 25 个二进制、预计还要两三个小时。
4. 13:0x UTC，主 agent：全量测试不用等，停掉 `check_rebased.sh` 那一串、照实列已跑完的、交回。照做（第十一节）。
