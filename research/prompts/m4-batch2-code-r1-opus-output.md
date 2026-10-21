# m4-batch2-code-r1 攻方腿报告（Opus，2026-10-13）

轮名 m4-batch2-code-r1（里程碑四第 2 批代码轮第一轮），攻击面正文第三节 T1–T9。被判对象是冻结副本 `~/.local/share/singlefs/work/m4-batch2-code-r1/repo/`（开工时按 `research/prompts/m4-batch2-code-r1-snapshot/crates-sha256.txt` 16 行逐个核过，0 行不符）；一切改动与编译都在草稿目录的拷贝 `~/.local/share/singlefs/work/m4-batch2-code-r1-attack/repo/` 上做，下文「副本」都指它，副本上量出的数不是入库装置上的数。

## 复跑命令与模型文件

整条经内存包装（`tools/rerun.sh` 里每条 cargo 经 `scripts/hosts/capped.sh 8`），`--work` 给一个新目录：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-batch2-code-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-batch2-code-r1/repo --work <新草稿目录>
```

六步（`RERUN_ONLY="<步名> …"` 只跑其中几步；`RERUN_TARGET_DIR` 给编译目录）：s1 快照树表与区域扫描探针、s2 随机历史上 I-1.13 的判定计数（4 批）、s3 白名单绕过变异 W2、s4 配对等号变异、s5 区域扫描四个变异、s6 两行没证红的 discard 变异。末行 `name=rerun_summary same=… differ=… missing_in_model_out=…`。`out/` 是草稿目录里各次真跑的日志按 `rerun.sh` 的 normalize 同一规则规整出来的（`tools/` 外的草稿脚本 `make_out.py`）；本腿用 `rerun.sh` 在新目录里复跑过 s1、s3、s4、s6（各自 `same=1`，见下），s2、s5 前台 4 分钟跑不完，没经 `rerun.sh` 复跑，列在「要主 agent 跑的命令」。

```
name=rerun_out file=s1-snapshot-and-region-scan.txt same=1
name=rerun_out file=s3-whitelist-bypass.txt same=1
name=rerun_out file=s4-pairing-equality-probe.txt same=1
name=rerun_out file=s4-pairing-equality-six-tests.txt same=1
name=rerun_out file=s6-row-4037.txt same=1
name=rerun_out file=s6-row-4040.txt same=1
```

模型文件 sha256（26 个）：

```
43f4d7e0019c5bb0404423a759d00e3936eb97be4da485f540555f96f99a2950  out/s1-snapshot-and-region-scan.txt
08ed20a7ffb053699c2889eda7ad5ef49549d4de9de19044e7c49480754a2fbe  out/s2-i113-b1.txt
013052265461566b3b4823081b425943c73f7d024f7c64632f8daf75f04b8563  out/s2-i113-b2.txt
222c896824e84c72d746e8b5e560d25f2b471736a014c7a356b81b2f34a91597  out/s2-i113-b3.txt
7be41fc1dbb42e15daf3ab23490bab8cd7c907c4cdb4c7b64cdefa3363584c64  out/s2-i113-b4.txt
048e9d3c53cad7d02d2b9c09661d3da03cc9ec174171b67874082fb6bb474bdc  out/s3-whitelist-bypass.txt
83f1b698ff92280f151c31060a58e0ed515cb90f89560b578250e0a4419551fe  out/s4-pairing-equality-probe.txt
ea0ecb1faf9cb9fdcd95a6c31f30675df34cdb158298507e7afa5c8e48ea7138  out/s4-pairing-equality-six-tests.txt
3e77c77eba8fafa9d1d93d563a1efcd4b97d13306eec9ab5b8382676cdb6fc95  out/s5-R1-quarantine-wins-over-mapping-core.txt
77d091a8c0009be4221a4e2d40cdcbd72a17a7d22d74a6934e70072e7effc4f1  out/s5-R1-quarantine-wins-over-mapping-harness.txt
3e77c77eba8fafa9d1d93d563a1efcd4b97d13306eec9ab5b8382676cdb6fc95  out/s5-R2-any-instead-of-all-core.txt
77d091a8c0009be4221a4e2d40cdcbd72a17a7d22d74a6934e70072e7effc4f1  out/s5-R2-any-instead-of-all-harness.txt
3e77c77eba8fafa9d1d93d563a1efcd4b97d13306eec9ab5b8382676cdb6fc95  out/s5-R3-points-at-ignores-the-device-core.txt
77d091a8c0009be4221a4e2d40cdcbd72a17a7d22d74a6934e70072e7effc4f1  out/s5-R3-points-at-ignores-the-device-harness.txt
3e77c77eba8fafa9d1d93d563a1efcd4b97d13306eec9ab5b8382676cdb6fc95  out/s5-R4-pinned-only-at-the-start-slot-core.txt
77d091a8c0009be4221a4e2d40cdcbd72a17a7d22d74a6934e70072e7effc4f1  out/s5-R4-pinned-only-at-the-start-slot-harness.txt
d42f8f7bfb44b9f088d151450f8ddba8818cdeff2e703cbef9332cf1757968b5  out/s6-row-4037.txt
f557ffed2e5daac04eed17ba7e8fe98eaa52c27d96c272fbb90be435f4eed522  out/s6-row-4040.txt
8bf1f6ca29142f489abe078655c42c1a137470e83016e728bfb5fd943e2e1733  proto/attack_i113_reasons.rs
ec0a28d7231b4a523983883ec8faa47050aecca7970de23fc9759a96e0a70007  proto/attack_snapshot_and_region_scan.rs
0cca6cc6e36e43322bd599f2fdeece85191925a7a916497a0c3885a752d1049d  tools/apply_row.py
6d98af55d551de1a171fceeb7f50ba1323bff30c3030e8d99d17eef4f57e2614  tools/mutate_text.py
775d5507e28f5a379d06cdf609e83f3199aa4fa4aa3079832a9eab03e0499e29  tools/mutate_whitelist.py
d2a3f8af7bca774b1b9d4c24fe12ac7e1e5a0d0c8b9d4f786b5df2e1867f9e74  tools/region-scan-mutations.tsv
ebf7e01d3b13aff6b76daeab865305f4171dc258e6a691ffc90f7a101654431a  tools/rerun.sh
719846a48cf86c4fcdc989a59f4d2d40ee5572ce4286af1b525a0267d8aabda4  tools/summarize_i113.py
```

## 各格判定一览

| 格 | 判定 | 打中的东西（一句） | 已知清单里有没有 |
|---|---|---|---|
| T1 | **打中** | 配对判定里「码 15 头 ID 等于码 14 树 ID」改成「不大于」，6 条用例全绿，而一张两条码 15 都指第二个码 14 的坏表被收下、第一条码 15 被配给第一个码 14（量过） | 无 |
| T2 | **打中（两处）** | ① 合法快照表（S1 字段取值）冷启动走读 `walk_to_file` 与 `user_visible_tree_root_pointers` 拒成「树的种类没登记」，池级 checker 判 I-9.16 ③ 红、带前驱时再红 I-9.25（量过）；② 同一张表可写挂载照收，挂载那几次发布把 4 条码 14 / 15 一声不响全丢了（量过） | 已知清单只记了字段与构造点未加；①② 都不在清单里 |
| T3 | 没打中 | 字段偏移、369、三条拒绝与「零格式位」都对得上；用例钉了绝对字节 | — |
| T4 | **打中（不分辨读法）** | 自定读法「隔离槽仍有映射条目指着按普通单元判」翻成相反读法，12 条用例全绿；另三处判定（all→any、位置条目不比盘、钉住只看起点）各翻一次也全绿（量过） | 自定读法被攻过零轮，不在已知清单 |
| T5 | **打中（条件：盘上坏记录）** | 一条跨段的坏记录：选段把它的槽算进两段、选中第 0 段，扫描整次拒，选段输入里没有「被拒的 R」，再选还是它——整理卡死在这一段（量过）；资格按记录算这一形在第 3 批改成「至少一个能搬」之后仍在 | 资格写错在清单里；跨段卡死不在 |
| T6 | 没打中 | 默认实现先核对齐与越界再交 Unsupported，与写零同一判据；两行没证红的 harness 变异在副本上各自红在自己点名的用例（量过） | — |
| T7 | 部分（推的） | `zero_fill` 同名让段种类多重集那一道分不出 discard 与整段清零；两态之外「一次 discard 段内部分落地」没有枚举，今天两个后端都到不了 | — |
| T8 | **打中（两处）** | ① 必红世界 ③ 能被绕过：非测试代码中段一行带 `#[cfg(test)]` 字样的注释之后 `use crate::walk::check_pool_image;` 并指名它，两条白名单单测都绿（量过）；② C827 说 11 个成员里「这两格」真实写路径走得到，实测 C615（谱系）与 C768（退役）两格也走得到、且在随机历史上占不适用的大头，删过一次文件之后的 241 个状态 0 个成立（量过） | ② 的后果超出 C827 写的范围 |
| T9 | 打中（推的） | I-4.4 先例在 CPU 一侧恒不适用、判到位恒 0；I-1.13 在 CPU 一侧判得到，对拍逐字比判到位，GPU 判到位恒 0 ⇒ 不是只在 I-1.13 违例的状态上红，而是在 CPU 判 I-1.13 成立的每个状态上都红 | 「不比没接上」在清单里；红的范围比「预期红那几条」宽，是推的、没跑 |

## T1 快照条目的配对判定

**打中：「头 ID 恰是一条码 14 的树 ID」那个等号，6 条用例钉不住。** 配对那一行是「inode_root_entry.tree.0 == extent_root_entry.head_identifier」（`crates/singlefs-core/src/recovery.rs:2434`）。6 条用例里码 15 的头 ID 只取过三种值：恰等于自己那条码 14（「extent_root.head_identifier = snapshot_inode_root_tree;」，`crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs:188`）、活头 inode 树号（比所有码 14 都小，「version.pairs[1].extent_root.head_identifier = live_inode_tree;」，`crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs:350`）、第一个码 14 的号（「version.pairs[1].extent_root.head_identifier = version.pairs[0].inode_root.tree.0;」，`crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs:368`）——没有一条让码 15 的头 ID 比它该配的码 14 大。

- 变异 M-T1（副本上，模型目录 tools 下的 mutate_text.py）：把 `==` 换成 `<=`。6 条用例原样输出（`out/s4-pairing-equality-six-tests.txt`，排过序）：

```
cargo_rc=0
test a_snapshot_extent_root_whose_head_is_not_any_snapshot_inode_root_is_refused_by_every_reader ... ok
test a_snapshot_inode_root_whose_head_is_not_the_live_inode_tree_is_refused_by_every_reader ... ok
test a_tree_table_carrying_two_snapshot_pairs_is_rebuilt_and_hands_back_both_pairs ... ok
test a_tree_table_with_one_more_snapshot_extent_root_than_snapshot_inode_roots_is_refused_by_every_reader ... ok
test an_eleventh_snapshot_is_refused_by_every_reader_while_ten_are_rebuilt ... ok
test two_snapshot_extent_roots_claiming_the_same_snapshot_inode_root_are_refused_by_every_reader ... ok
```

- 同一变异下的坏表（探针 `two_extent_roots_both_naming_the_second_inode_root`：两对快照，码 14 树号 65549、65551，两条码 15 的头 ID 都写 65551）被收下、配错（`out/s4-pairing-equality-probe.txt`）：

```
ATTACK_PAIR 收下：码 14 树 65549 ↔ 码 15 树 65550（码 15 头 ID 65551）
ATTACK_PAIR 收下：码 14 树 65551 ↔ 码 15 树 65552（码 15 头 ID 65551）
```

  冻结副本原样代码上同一张表被拒（草稿日志 mt1-probe-orig.log 那一行）：

```
ATTACK_PAIR 拒：Walk(UnitMalformed { what: "树表里有一条快照 extent 根配不上一条只属于它的快照 inode 根" })
```

- 四句：不涉及候选与臂，是判定与它的用例之间的缺口；判别它的东西系统看得到（码 15 的头 ID 与每条码 14 的树号在同一张树表里）；满足的是 T1「8 行变异是不是各自只靠一条用例红」那一问的反面——有一种写错法一条用例都不红；跑前条款没给改法。
- 改法（只在我的模型上量过、被攻过零轮）：加一条用例，码 15 的头 ID 取「比它该配的码 14 大、等于另一条码 14」的值（上面探针的形），期望拒成同一句文案。「`==` → `<=`」那一格它红不红——推的（没把用例写进副本跑）。

**其余各问（没打中）**：S1 另写了码 15「前驱 0」与码 14「flags 位 0 留给销毁中」（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1 那一行）。前驱字段今天解完即丢（下一节）；flags 位 0 今天读者拒，探针输出「ATTACK_PARSE flags 位 0 的码 14：TreeTableEntry::parse = None」，来源是 `crates/singlefs-core/src/records.rs:504` 那一行 flags 非 0 即拒；销毁中归 S4 那一批，不算打中。两条码 15 指同一条码 14、码 14 比码 15 多一条、第 11 条、头 ID 指别的头，四种镜像各落在用例预期的那一条文案上（6 条用例在副本上原样绿）。码 14 / 15 的放行只跳过这两种，别的种类照旧进 `kinds_seen`。

## T2 读侧兜底与还没加的字段

**打中 ①：合法的快照树表，冷启动走读与池级 checker 不收。** 探针 `snapshot_tree_tables_on_every_reader_and_the_pool_checker`：第一个文件那一版（txg 3）的十二条之后照 S1 / S2 的字段取值接两对码 14 / 15（树号从水位连发、码 14 头 ID = 活头 inode 树号、码 15 头 ID = 配对码 14 的树号），重建树表单元、补校验和与水位、写回根槽、照抄进 journal 记录新根段；对照「原样重写」那一份 checker 违例 0 条，改法本身不带红。原样输出（`out/s1-snapshot-and-region-scan.txt` 里这一份的行）：

```
ATTACK_READER 两对快照（前驱全 0） rebuild_version: Ok(())
ATTACK_READER 两对快照（前驱全 0） allocation_records_under_root: Ok(())
ATTACK_READER 两对快照（前驱全 0） walk_to_file: Err(UnitMalformed { what: "树的种类没登记" })
ATTACK_READER 两对快照（前驱全 0） user_visible_tree_root_pointers: Err(UnitMalformed { what: "树的种类没登记" })
ATTACK_READER 两对快照（前驱全 0） open_pool_for_read: Ok(())
ATTACK_CHECKER 两对快照（前驱全 0） I-9.16 违例：根（实例 1、txg 3）的树表：种类 14 有 2 条（③ 每种至多一条）
ATTACK_CHECKER 两对快照（S1 字段取值） I-9.25 违例：树 65551（种类 14）的树表条目 previous_snapshot_txg 2：不是 inode 树的条目恒 0，inode 树的不大于这一版的 txg 3
```

- 来源：`user_visible_tree_root_pointers` 的兜底臂报「树的种类没登记」（`crates/singlefs-core/src/recovery.rs:3475`）；`walk_to_file` 那一路「let key_width = key_width_for_kind(entry.kind).ok_or(RecoveryFailure::UnitMalformed」（`crates/singlefs-core/src/recovery.rs:5504`）。用例对合法表只核了 `rebuild_version` 一个读者（用例文件第一条），五个读者一起核的只有四张坏表。
- 结果：同一份盘面，可写挂载那一路（`rebuild_version`）收、冷启动恢复那一路（`walk_to_file`）拒，只读挂载（`open_pool_for_read`）收，checker 判红。
- 四句：不涉及候选；系统看得到（种类码就在条目里）；满足 T2「码 14 / 15 在挂载读路径上被当成什么」那一问；跑前条款没给改法。判决 S1 那一行写了「I-9.16 ③、I-9.27、I-9.25 与 core 的 `tree_table_entries_each_kind_at_most_once` 各加 14、15 一格」，这一批只做了 core 那一格；checker 三格与读侧两处兜底臂归哪一批，正文第二节没写（实现员报告提过读侧两处，正文「已知」清单里没有）。

**打中 ②：可写挂载收下带快照的池，挂载自己的发布就把快照条目一声不响丢掉。** 探针 `a_writable_mount_of_a_pool_carrying_snapshot_entries`：同一张两对快照的表，内存稀疏盘上 `mount_writable` 一次，原样输出：

```
ATTACK_MOUNT 挂载之前最新根（txg 3）树表里码 14 / 15 共 4 条
ATTACK_MOUNT 现行那一版（内存里）树表 12 条，码 14 / 15 0 条
ATTACK_MOUNT 挂载之后盘上最新根（txg 5）树表里码 14 / 15 共 0 条
```

- 来源：重建上一版那一个入口的文档写着快照条目「这一个丢掉它们」（`crates/singlefs-core/src/recovery.rs:2649`）；写者每次发布照十二个种类现装树表（「let tree_table_entries = vec![」，`crates/singlefs-core/src/transaction.rs:8649`）；`RebuiltVersionWithItsSnapshotEntries` 在 `crates/` 里除那份用例外没有消费者（grep 只命中 recovery.rs 自己与用例文件）。
- 后果超出已知清单：清单写的是「`previous_snapshot_txg` 字段与四处构造点还没加（第 3 批）」，没写「可写挂载放行之后头一次发布就删掉快照」。今天没有写者造得出快照，射程是手造镜像与「建快照接上、读侧补齐之前」那一段；判决 S8 对回退写了「有码 14 / 15 条目就拒」，可写挂载没有对应的一格。
- 改法（零轮，推的，没实现没跑）：`RebuiltVersionWithItsSnapshotEntries` 有消费者之前，可写挂载对「快照条目非空」拒（与 S8 的回退同形）；或写者把上一版的码 14 / 15 原样带进新树表。

**`previous_snapshot_txg` 那 8 字节**：读者解出来就丢（「let _previous_snapshot_txg = reader.get_u64();」，`crates/singlefs-core/src/records.rs:512`），写者恒写 0（「writer.put_u64(0); // previous_snapshot_txg：0 = 不适用」，`crates/singlefs-core/src/records.rs:473`）；核心层没有别的判定把它读成别的量；池级 checker 的 I-9.25 读它，码 14 上照 S1 填非 0 就红（上面那一行）。

## T3 logged_ops 条目的格式（没打中）

- 字段偏移：`to_bytes` 每段之后都有 `assert_position`（14、20、22、30、38、44），与判决 G1 的「整理 = R 的设备 4 + 起始槽 6 + 槽数 2、`c0` 8、`segs0` 8、补零 6」逐段对上（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 那一行）；用例按手写字节钉了绝对值（`crates/singlefs-harness/tests/logged_operations_intent_record_round_trips.rs` 第一条，期望字节不从写者反推），不是只比往返。
- 一节点 369：(16384 − (86 + 2 × 10 + 29)) ÷ 44 = 16249 ÷ 44 = 369.29，取整 369；用例拿 core 的 `index_node_entry_capacity` 现算比。
- 三条拒绝（宽 ≠ 44、种类 0 / 2 保留、R 不对齐或槽数 ≠ 64）与 D26 已定项 2 的「零格式位」不冲突：那一句说的是谓词的输入不新增盘上字段、`c0` 住意图记录（`.claude/kb/decisions/26-后台整理与放置回收.md:51`），意图记录本身的字段表正是 G1 定的那一份。
- 没判的一格：解码只核 64 对齐，不核「R 在单元区内」——起点槽 0（落在固定结构里）的 R 解得开；G1 把它写进 I-4.4 的判法，而 I-4.4 今天在池级 checker 上恒报不适用（见 T9），今天没有写者，不算打中。

## T4 区域扫描的四类判定

**打中：自定读法「隔离槽仍有映射条目指着按普通单元判」没有任何用例钉，翻成相反读法 12 条用例全绿；同类的还有三处。** 副本上逐个变异（模型目录 tools 下 region-scan-mutations.tsv），每个变异跑 core 的 5 条区域扫描用例与 harness 的 7 条选段 / 计划用例：

| 变异 | 改哪一行 | core 5 条 | harness 7 条 |
|---|---|---|---|
| R1 隔离优先于映射 | `crates/singlefs-core/src/region_scan.rs:244` 去掉「映射里没人指着」那半个条件 | 全绿 | 全绿 |
| R2 all → any | `crates/singlefs-core/src/region_scan.rs:274`（指着这一槽的条目有一条 key 对就判可搬） | 全绿 | 全绿 |
| R3 位置条目不比盘 | `crates/singlefs-core/src/region_scan.rs:39`（只比槽号） | 全绿 | 全绿 |
| R4 钉住只看起点槽 | `crates/singlefs-core/src/region_scan.rs:237`（不看记录罩住的后一槽） | 全绿 | 全绿 |

R1 下 core 那 5 条的原样输出（`out/s5-R1-quarantine-wins-over-mapping-core.txt`；R2–R4 的 core 与 harness 两份与 R1 逐字节相同，sha256 见文首）：

```
cargo_rc=0
test region_scan::tests::a_record_straddling_the_region_boundary_is_refused_before_any_classification ... ok
test region_scan::tests::a_unit_whose_header_key_differs_from_the_mapping_entry_pointing_at_it_is_inconsistent ... ok
test region_scan::tests::a_unit_whose_mapping_entry_points_at_another_slot_is_inconsistent_not_dead ... ok
test region_scan::tests::an_unreadable_header_or_a_class_tag_unlike_the_mapping_key_is_inconsistent ... ok
test region_scan::tests::every_class_of_the_region_scan_is_pinned_down_slot_by_slot ... ok
```

- 为什么都绿：用例里隔离的那一槽从来不被映射指着（「pool.place(slot_in_region(6), 2, Some(&quarantined_data));」，`crates/singlefs-core/src/region_scan.rs:594`，没有对它 `map`），两盘的位置条目恒同槽，钉住的槽只给过跨度 1 的节点，每个槽至多一条映射条目指着。T4 问的「优先级的每一次交叉」用例只罩了一部分。
- R1 那一格的后果（推的）：判决 G9 定义「已死 / 隔离」都写的是「映射里没人指向它」，G2 的 K-c 要「签发之后 R 里出现隔离槽就丢意图」。今天的读法下，一个在隔离集里、映射又指着的单元会读头往下走，头读得出、key 对得上就是可搬——计划会去搬一个「释放前核出校验和对不上」的落点。普通发布走不到「映射指着隔离槽」（隔离发生在释放时，映射此时已改指新落点）；挂着时的回退（向前发布一版旧根）会把映射改回旧版本的落点，旧落点里有一份当初释放时被隔离的副本，映射就指着隔离槽——这一形可达不可达、回退是不是先判隔离，我没量，是推的。两种读法哪个对，判决没写；这一格只说「哪个读法都没有用例拦着」。
- 四句：不分辨臂（两种读法共用全部用例而全绿）；系统看得到（映射条目与隔离集都是输入）；满足 T4「隔离槽仍有映射条目指着时按普通单元往下判……7 条用例与变异覆没覆盖优先级的每一次交叉」那一句的否定分支；跑前条款没给改法。改法（零轮、推的）：先在判决里定读法，再各加一条「隔离 ∧ 被指着」「两条条目指同一槽 key 不同」「位置条目两盘不同槽」「钉住的是跨度 2 单元的后一槽」用例。
- 真写者写出的四版上（探针 `region_scan_classes_on_versions_written_by_the_writer`：第一个文件 → 建 300 个空 inode → 顺序写 200 KiB → 覆盖写 5000 字节，内存稀疏盘），每一条映射条目对应的单元都判可搬，豁免的三类（映射树节点、树表、实例表第一片）不在钉住集里时一律判已死。原样输出（摘最后一版两行）：

```
ATTACK_SCAN 版本「第一个文件 + 建 300 个 inode + 顺序写 200 KiB + 覆盖写 5000 字节」 盘 0 段数 2 映射条目 11：{"可搬": 11, "已死｜MappingTree｜槽 50291": 1, "已死｜TreeTable｜槽 50292": 1, "已死｜（这一版没有这个槽上的角色）｜槽 50176": 1}
ATTACK_CHOOSE 版本「第一个文件 + 建 300 个 inode + 顺序写 200 KiB + 覆盖写 5000 字节」 钉住 2 槽：选中 R 起点 50176，R 里各项 ["已死", "可搬"]，一批 3 交 1 项
```

  槽 50176 是实例表第一片（同一探针打出的「实例表指针」两条位置条目都是槽 50176）。这说明码 1 / 码 2 / 码 3 三条 key 规则（含自定的「码 3 实例代号取写序里那一个」）在单实例的真写者镜像上与写者一致；跨实例（重挂之后照抄的容器）没造，见限度。豁免单元不靠头里的东西认、全靠调用方给的钉住集，钉住集漏一类就落进「已死」——这是判决 G2 把钉住集放进输入的直接后果，不算打中。

## T5 选段与计划

**打中（条件：盘上有一条跨段的坏记录）：整理会卡死在同一段。** 选段按记录罩住的每个槽计数，不管记录跨不跨段（「for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {」，`crates/singlefs-core/src/compaction_plan.rs:63`）；区域扫描遇到跨段记录整次拒（「if reaches_in_from_below || reaches_out_above {」，`crates/singlefs-core/src/region_scan.rs:189`）；选段的输入只有开放段、活意图的 R 与钉住集，没有「上一次被扫描拒了的段」。探针 `a_straddling_record_makes_greedy_choose_the_refused_region_every_time`（单元区第 0 段末槽一条跨 2 的记录，第 2 段 10 个正常单元）原样输出：

```
ATTACK_STRADDLE 第 1 次：选中 Some(0)，扫描 Some(Err(RecordStraddlesTheRegionBoundaryUnsupported { device: DeviceIdentity(0), slot: SlotNumber(50239), span_slots: 2 }))
```

（第 0 次那一行同形。）跨段的那条记录把第 0、1 两段都算成 1 个活槽，两段都会被扫描拒；greedy 每次取第 0 段。资格改成判决 G2 的「R 里至少一个能搬的单元」（已知清单，第 3 批）之后，这一形仍在：被拒的扫描交不出分类，资格要另写「扫描拒了算不合格」才排得开。坏记录来自盘上（C411 的域，跨度与单元类别恢复不判），写者造不出。四句：分辨臂——不涉及候选；系统看得到（扫描交回了拒绝）；满足 T5「资格……会不会选出……让 plan_batch 交空批而 R 还排他着」那一问的另一形：不是空批而是每次都扫不了；改法（零轮、推的）：选段的资格加一条「扫得了」，或者扫描对跨段那一条单列一类而不整次拒。

其余（没打中）：排开钉住段、开放段、活意图 R 三条在 `crates/singlefs-core/src/compaction_plan.rs:85` 起那三行，各有一行变异证过；greedy 分数按记录罩住的槽数算，不按扫描分类（所以已死 / 不一致的段也算活槽，这正是已知清单那一条）；同分取起点小的有变异证过；`plan_batch` 只交（映射 key, 源落点），没有目的落点。真写者四版上选段都选中实例表所在的第一段、按可搬数交批（上面那一行），批里同一个 key 至多一项（每条映射条目只指 R 那块盘上的一个槽）。

## T6 discard 口子的 trait 与两个后端（没打中）

- 默认实现先过「check_aligned_and_in_range」再交 Unsupported（`crates/singlefs-core/src/block_device.rs:193` 起那一段，交回那一行在 `crates/singlefs-core/src/block_device.rs:204`）：越界、没对齐交错误，不吞成 Unsupported；判据与写零、两个后端同一份。文件后端打洞报不支持交 Unsupported、不退回写 0，报别的错照常交回；`EINTR` 原样重打、没有轮数上限（上界是信号数，与写零那一路同一口径）；长度 0 交 Applied 并计一次（「if discarded_length == 0 {」，`crates/singlefs-core/src/block_device.rs:656`）。`O_DIRECT` 后端核过请求交 Unsupported 并计数（`crates/singlefs-core/src/block_device.rs:948`）。测试开关是 `with_test_only_discard_landing` 换进去的，平常路径的初值是打洞那一臂。
- 没证红的 harness 两行，副本上各自红在自己点名的那条用例（模型 out/s6-row-4037.txt、out/s6-row-4040.txt 原样，排过序）：

```
  left: DiscardOutcomeCounts { applied_calls: 0, unsupported_calls: 0 }
 right: DiscardOutcomeCounts { applied_calls: 0, unsupported_calls: 1 }
cargo_rc=101
test a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step ... FAILED
```

```
  left: "[unit_write×2,barrier]"
 right: "[zero_fill,unit_write,barrier]"
cargo_rc=101
test segments_count_a_discard_as_one_plain_write_with_two_crash_states ... FAILED
```

  分别是 `crates/mutations.tsv:4037`（录制器不数结局）与 `crates/mutations.tsv:4040`（切段归进写单元）。checker 档那 3 行没跑（重型）。
- 零长度那一格的小差（推的，不算打中）：同一个零长度请求，文件后端与内存盘交 Applied、`O_DIRECT` 后端与默认实现交 Unsupported；录制器把 Applied 的零长度 discard 记成一步，进了写表就是段里一个空写（状态数多一倍、内容相同），设备日志期望侧多一条长度 0 的 Discard。今天没有产品路径发 discard（`grep` 里 `.discard_at(` 只在两个包装里转发），走不到。

## T7 录制、两态与设备日志（部分，推的）

- 「不支持就一个字节没动」只在后端那一层保证：录制器照后端交回的结局记不记（`crates/singlefs-harness/src/lib.rs:459` 起），故障注入层照转；没有哪一层核「交 Unsupported 之后盘面真的没变」。今天三个后端都满足（默认实现与 `O_DIRECT` 不发系统调用，文件后端 `EOPNOTSUPP` 不改文件），不算打中。
- `zero_fill` 同名分不出的那一道：分段与种类多重集（「RecordedOperationKind::Discard => StepKind::ZeroFill,」，`crates/singlefs-harness/src/segments.rs:68`）。登记表比的是每段的种类多重集，一次 discard 与一次整段清零在里面都写成 `zero_fill`：产品路径哪天把该发 discard 的地方发成 `write_zeroes_at`（或反过来），段序列与种类串一个字不变，这一道判不出；设备日志判器分得出（期望侧 WriteZeroes 投成整段写、Discard 投成 Discard 条目），崩溃点重放两样都按「范围写零」两态，同样分不出。今天没有产品路径发 discard，射程是第 3 批。
- 两态是不是 G5 的两态：是（G5「崩溃点重放当「范围写零」枚举两态」；写表那一处「discard 按「范围写零」进写表（D13（验证路线） 已定项 4）」，`crates/singlefs-harness/src/memory_pool.rs:611`）。第三态（推的）：一次 discard 罩一整段 64 槽（1 MiB），掉电时段内一部分扇区已清、一部分还是旧字节，这一态两态枚举不出；它与「两态」的差只在扫描方向读被 discard 的槽时（I-1.8 / I-1.13 扫候选槽），被 discard 的槽按 G5 只取「可再分配」的，上面没有被引用的单元，我想不出哪条不变量会因此翻面，没量。真盘「Applied 之后读回既非 0 也非旧字节」归真盘线。
- 宿主重跑与虚机今天都是 0 条：`crates/` 里除两个包装的转发外没有一处调 `discard_at`；下一批产品路径发了之后，宿主重跑那一侧（内存盘交 Applied，录进流、期望侧投 Discard）与虚机那一侧（`O_DIRECT` 交 Unsupported，盘上没有 Discard）会在设备日志逐项比对那一步对不上——已知清单里那一条。

## T8 扫描重建与 I-1.13 的不适用口径

**打中 ①：必红世界 ③（「往重建模块里加一行调 `walk_*` 的代码，白名单单测红」，`.claude/kb/milestone/04-fourth-txn.md:109`）能被绕过。** 两条白名单单测都只读「测试模块之前那一段」，切点是源码里第一处 `#[cfg(test)]` 字样（「.split_once("#[cfg(test)]")」，`crates/singlefs-checker/src/rebuild.rs:595`），第二条按字面 `crate::` 找路径（「for (index, _) in source.match_indices("crate::") {」，`crates/singlefs-checker/src/rebuild.rs:664`）。变异 W2（模型目录 tools 下 mutate_whitelist.py）：在 `rebuild_index_tables` 的文档之前加一行普通注释「// 下面这个函数的用例在文件末尾 #[cfg(test)] 模块里」，其后一行 `use crate::walk::check_pool_image;`，函数体里 `let _walk_entry_named_by_the_rebuild = check_pool_image;`。编译 0 条警告，两条单测原样输出（`out/s3-whitelist-bypass.txt`，本腿另在新目录经 rerun.sh 复跑 same=1）：

```
cargo_rc=0
test rebuild::tests::the_scan_rebuild_names_no_crate_item_outside_the_whitelist_by_a_path ... ok
test rebuild::tests::the_use_list_of_the_scan_rebuild_is_exactly_the_format_parsing_whitelist ... ok
```

  对照：把那行注释放在 `use` 清单之前（变异 W1），第一条单测红在「白名单里没用上的」那一半（草稿日志 w1.log）——切点前移会把合法的 use 一起切掉，所以绕过要把注释放在 use 清单之后。四句：不涉及候选；系统看得到（切点就是源码里的字样）；满足四·1 题面必红世界 ③ 那一句的否定；改法（零轮、推的）：切点改成按 `mod tests` 那一行的语法位置切，或者白名单单测改读 `cargo` 产出的依赖信息而不读源码文本。

**打中 ②：C827 只说 11 个成员里「这两格」真实写路径走得到，实测 C615（谱系）与 C768（退役）两格也走得到，而且在随机历史上占不适用的大头。** C827 那一行原文是「`ScanRebuildNotDecided` 11 个成员里这两格真实写路径走得到（unlink 意图到最后一批之间、覆盖写变短留下的单元）」（`.claude/kb/checks-owed.md:594`）。探针 `tally_i113_verdicts_over_random_histories`（随机历史生成器与执行器原样，每一步之后对镜像跑 `check_pool_image`、按 I-1.13 的判定计数；两块单元区 384 槽的小盘、空间准入判着、每段 30 步）四批的汇总（模型目录 tools 下 summarize_i113.py 对四份日志的原样输出）：

```
i113-b1.log  ATTACK_SUMMARY weights=broad seeds=8 first_seed=1000 ops=30 elapsed_s=56.2
  状态 248：不适用·C615 谱系 128、不适用·C768 退役 17、成立 103
  上一步 CreateInode 的状态：不适用·C615 谱系 26、不适用·C768 退役 4
i113-b2.log  ATTACK_SUMMARY weights=delete seeds=16 first_seed=2000 ops=30 elapsed_s=85.8
  状态 496：不适用·C615 谱系 340、不适用·C768 退役 96、不适用·C827 变短 4、成立 56
  上一步 CreateInode 的状态：不适用·C615 谱系 124、不适用·C768 退役 28
i113-b3.log  ATTACK_SUMMARY weights=reuse seeds=16 first_seed=3000 ops=30 elapsed_s=50.7
  状态 496：不适用·C615 谱系 61、不适用·C768 退役 130、成立 305
  上一步 CreateInode 的状态：不适用·C615 谱系 6、不适用·C768 退役 53
i113-b4.log  ATTACK_SUMMARY weights=orphans seeds=16 first_seed=4000 ops=30 elapsed_s=60.1
  状态 496：成立 496
  上一步 CreateInode 的状态：无
```

- 四批 1736 个状态，I-1.13 违例 0 个、成立 960 个、不适用 776 个，其中 C615 谱系 529、C768 退役 243、C827 变短 4、C827 删除中 0（随机历史里 unlink 一次调用就推完全部批，中间态只在崩溃状态上出现）。「上一步是建 inode」只发生在删完第一个文件之后：这 241 个状态 0 个成立——删过一次文件的池，之后每一版 I-1.13 都不适用；回退过一次的池同理（谱系记录只增不删，`crates/singlefs-checker/src/rebuild.rs:121`、`:123` 那两个成员）。
- 后果超出 C827 写的范围：四·1 题面的射程写了「回退过的镜像上这一条报 NotApplicable」，退役那一格题面写的是「退役记录读侧第一版不接」；两样都没写「一次 unlink 之后永久不适用」，C827 写「这两格」。四·2 验收「固定脚本流与随机历史的每个镜像上比对」在今天的比重下近半数镜像比不了。
- 四句：不分辨臂（是覆盖面的缺口，不是哪条读法错）；系统看得到（退役树、谱系树里有没有记录）；满足 T8「`ScanRebuildNotDecided` 11 个成员里哪几格真实写路径走得到」那一问；跑前条款没给改法。改法（零轮）：C827 的范围改成四格并写明「删过 / 回退过之后永久不适用」；或先接 C768 的退役读侧（退役记录压住对应容器身份），删除线的镜像就回到可比。

**没打中的几问**：
- 常量表按整份 judgements 查会不会把合法镜像判成不适用：上面 1736 个合法状态里，不适用理由没有一个来自常量表（四条「遍历方向……已判损坏」的理由 0 次）。合法镜像上那四条本来就不该有违例，有了整张图已经红，常量表只决定 I-1.13 报不报重复的红。
- 输入禁区：`rebuild.rs` 里只有纯函数，读盘都在 `walk.rs` 的调用方；`judge_scan_rebuild_agrees_with_the_walked_index` 只拿扫描方向那一份结果与实例表行喂重建（`crates/singlefs-checker/src/walk.rs:12515` 常量表之后那一段），没看到绕过——绕过白名单的唯一一形是上面 ①。
- 「映射条目改指一份没分配的副本」那张镜像上 I-1.13 的红：扫描在原槽与副本槽都扫到同一版（同归并键），重建的落点集合比映射多两份、两边各留读得出的之后仍不同，是扫描重建自己真的对不上，不是 I-3.1 / I-3.11 的连带（推的，按 `differences_in_the_central_mapping` 的读法推，没单独跑那张镜像）；同一机理就是已知清单里「G3 落地后合法搬迁镜像上 I-1.13 会红」那一条。

## T9 GPU 覆盖表进、内核不置位（打中，推的，没跑）

- I-4.4 先例怎么落的：CPU 一侧 I-4.4 恒报不适用（「// I-4.4（意图只描述目标态）：第一版的意图载体没有可装增量的字段，报不适用并写理由（用户 2026-10-05 定）。」，`crates/singlefs-checker/src/walk.rs:13417`），所以违例位、判到位两边都恒 0，「进覆盖表、内核不置位」在对拍上逐位相同。I-1.13 在 CPU 一侧判得到：成立时判到位置 1（「InvariantVerdict::Holds => (violated, judged | bit),」，`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:420`）。对拍除了违例位还比判到位（「|| gpu_cross_check_words.pool_checker_judged_bits」，`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2608`），GPU 判到位来自内核写的字，内核不置位就恒 0。
- 推出来的红的范围：不是「I-1.13 违例的那几个状态」，而是「CPU 判 I-1.13 成立或违例的每一个状态」。随机历史上成立的比例是 960 / 1736（上一节），新池新建文件、固定脚本这类没有回退、没有删除的流上几乎每个状态都成立——每条对拍流都会在第一个这样的状态上进 `checker_mismatches`。照抄 I-4.4 抄错的那一格就是这个：I-4.4 的 CPU 一侧是常量，I-1.13 不是。已知清单写了「不比没接进 `crash_judge_conclusions.rs`」，这一条只把「哪几条会红」说宽了，是推的（没起 GPU、不跑 checker 档）。
- 覆盖表的编译期断言（「POOL_CHECKER_COVERED_INVARIANT_INDEXES[position] == position,」，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:171`）证明的是：表的长度等于 `IMPLEMENTED_INVARIANTS` 的条数、第 k 项是 k。表本身因此恒等于 0..93，内核判不判哪一位它一概不知；「新立一条而内核没跟上时编译不过」只拦「忘了往表里加下标」，下标 92 加进去（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:158`）就过了，带 `gpu` 特性编过证明的也只是这一点。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| T1 | 两条码 15 指同一码 14、码 14 多一条、第 11 条、头 ID 指别的头、同树号两条码 14 | 6 条既有用例在副本上原样跑；同树号那一形只读代码（I-9.16 ① 在 `rebuild_version` 与 `walk_to_file` 拒、另三个读者不判，是这一批之前就有的口径） |
| T3 | 偏移、369、三条拒绝、「零格式位」、R 不在单元区 | 读代码与既有用例；R 不在单元区那一格记在 T3 |
| T4 | 真写者四版上的分类（码 1 / 2 / 3 三条 key 规则） | 单实例、内存盘、4 种前缀；跨实例照抄的容器没造 |
| T6 | 默认实现越界 / 没对齐、文件后端三条落地路径、两行没证红的 harness 变异 | 读代码；两行变异副本上各跑一次 |
| T8 | 合法镜像上 I-1.13 误红 | 1736 个随机历史状态 0 个违例；常量表造成的不适用 0 个 |
| T8 | 崩溃状态上的 I-1.13 误红 | 没跑（见限度） |
| T9 | 带 `gpu` 编、对拍 | 没跑（重型、不起 GPU） |

## 这条腿自己的限度

- 崩溃状态一个没枚举。派发要求的枚举函数（`enumerate_layer0` 一族）今天在 checker 档包里（`crates/singlefs-checker-tier/src/layer0_enumeration.rs`，不在 `memory_pool.rs`），那个包默认特性要现编 RocksDB、`--no-default-features` 编不过，前台 4 分钟编不完，原型就没走崩溃状态；I-1.13 在崩溃状态上会不会误红（例如发布中途、实例表清扫中途）这一问本腿没答。全量状态数因此记 0 个、累计 0 个（约 10⁷ 的额度一个没用）。
- 随机历史在 debug 下跑（release 下 core 一个包前台 4 分钟编不完，试了两次都在编 `singlefs-harness` 时被 `timeout` 截断，草稿日志 build2.log），盘取单元区 384 槽的小盘（4 GiB 盘上 2 段 × 30 步 235 秒没跑完，草稿日志 rh-probe.log）。批的段数按先跑一小段估：1 段 × 10 步 9.1 秒、8 段 × 30 步 8 线程 56.2 秒，之后每批 16 段、各 50–86 秒，四批共 56 段、1736 个状态、挂钟约 4.3 分钟，没设批的限时；缩的只有盘宽（4 GiB → 384 槽）与段数（随机历史大档默认 1000 段 → 56 段）。
- T4 / T5 的探针都在内存里造输入，没经发布入口；隔离集、钉住集是我按判决 G2 的列举手给的。
- T2 ② 的镜像是手造的（写者今天造不出快照），树表单元重建、根槽与 journal 新根段照抄都是我写的；对照组（原样重写）checker 0 条违例，排除了改法本身带红。
- 所有数都是副本上量的，不是入库装置上的数；改法全部零轮。

## 要主 agent 跑的命令

1. 复跑本腿模型的 s2、s5 两步（本腿在前台跑不完）：工作目录仓根，内存上限 8G，线程 8（rerun.sh 里每条 cargo 经 capped.sh 8，随机历史二进制 `ATTACK_THREADS=8`），预计 20 分钟内：

```
RERUN_ONLY="s2-i113-random-histories s5-region-scan-unpinned" bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-batch2-code-r1-opus-model/tools/rerun.sh --repo . --frozen ~/.local/share/singlefs/work/m4-batch2-code-r1/repo --work <新草稿目录>
```

   看末尾 `name=rerun_out file=… same=1` 共 12 行与 `name=rerun_summary same=12 differ=0 missing_in_model_out=0`。s2 的四份比的是排过序的计数行，`elapsed_s` 已去掉。
2. 要回答「I-1.13 在崩溃状态上会不会误红」（本腿没答）：崩溃点重放轻型照常跑，看 I-1.13 那一列有没有违例——`bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick`（轻型，归主 agent）。

## 没做什么

- 没跑任何门禁阶段、checker 档测试、GPU、重型测试；checker 档那 3 行 discard 变异没证；带 `gpu` 特性的编译没做。
- 没读、没碰 WGSL 与 `crash_amplification*`；T9 关于内核判到位恒 0 的那一句取自 `crash_judge_gpu.rs` 文档「内核不置位」，没读内核源码。
- 没读禁读清单里的文件；实现员报告只用来核事实（没证红的两行是哪两行、读侧两处兜底臂有人提过）。
- 没在冻结副本里改任何东西；副本上的改动都还原并逐字节比过（`cmp` 与冻结副本相同）。
- 不判正推、辩方那几格，不替主 agent 采纳；改法全部「只在我的模型上量过、被攻过零轮」或「推的」。
