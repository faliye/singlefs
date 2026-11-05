# m4-close-code-r1 正推腿报告（Sonnet，2026-10-30）

立场：正推。核「代码做的是不是判决与条款说的那件事」。冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-close-code-r1/repo/`（下文行号都指它），kb 条款写 kb 文件自己的行号。

开工核：冻结副本 30 份被判文件对 `research/prompts/m4-close-code-r1-snapshot/crates-sha256.txt` 逐份 `sha256sum -c`，非 OK 的 0 份。

模型目录 `research/prompts/m4-close-code-r1-sonnet-model/`（只读脚本，不起 cargo）。复跑命令（仓根、冻结副本、新的草稿目录三个路径都从参数给）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m4-close-code-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r1/repo --work <新的草稿目录>
```

跑完末尾打出 `name=partial_write_plan_model same_as_out=yes`、`name=panic_sites same_as_out=yes`、`name=mutation_checks same_as_out=yes` 三行。文件与 sha256（在模型目录里 `sha256sum` 取的）：

```
9f65c12b798876217e48d136155fe3b0e3af95f598398e646f3e9dd9727b22b2  partial_write_plan_model.py
6a1d1dd7f87c6d1b82fbced6497ba2da2dbe446f56788dfece217542cf94499a  panic_sites.py
2db7caa082d345a4b67a8154ebe9176f214a76dd37e230ab983815371413c358  mutation_checks.py
5147ed5737483d24db6587fb57aa3556643b968f910946ac873a1e3ecbcd7bf5  tools/rerun.sh
35f22c4ed24144aed6d471e11eac8f35bacdce5498b07ae05cf937553a357969  out/partial_write_plan_model.out
1ba160b2a9fa3ef9c56347e634e929adc9e856b62a850671e026483687a6c8b2  out/panic_sites.out
0b17fcae7c10a9df9467a98f4712cfbfe867140c16363c9bfed1c61f5f38750d  out/mutation_checks.out
59dc27835816e2a4e6139de423f0a46eff92184d52a2163a9f7311fec987bb3e  probe_partial_write_split.rs
93e342ebeac71f28f272ea43e490d987f1946b59797589236a49340bd35bf78d  out/probe_partial_write_split.run.txt
d1c1d5b37e366453c72e693d3616e66ad0150d655daf5b7b57e1ac742a24f2bd  out/probe_partial_write_split.first-run-failed.txt
```

后三个文件是 T2.3 的真 Rust 探针（源码、通过的一次输出、第一次红的输出）；它们要 cargo，不在 `rerun.sh` 里，跑法见 T2.3。`rerun.sh` 只复跑前三份脚本并与 `out/` 里同名输出逐行比。

`panic_sites.py` 取「改前」用 `git show 59ce5a86550de2c2c9bc6b51b34b4f881313682c:<路径>`（开工时的 HEAD）；本报告里「HEAD」「改前」指这一个提交。T1.1 至 T1.3 三节是上一条正推腿写的；我抽查了其中 22 处冻结副本行号（对着 `sed -n <行>p | grep -F <函数名或标识>`），20 处直接对上，`recovery.rs:3856` 那一处调用其实起于 :3854（`data_unit_count_matches_the_inode_size` 在 :3854 开头，:3856 是参数 `file.tail`），`unlink.rs:123` 对上的是 `Batch` 变体的文档行；其余结论文字我没改。

约定：下文「冻结:路径:行」指冻结副本 `crates/` 下的文件与行；「kb:文件:行」指仓里 `.claude/kb/` 下那份文件自己的行号（已现查）。判定三选一：成立 / 不成立 / 条款没说。

## T1 多 inode 改 size

### T1.1 第一个文件与别的 inode 是不是同一条路径、同一份判法

判定：成立（有两处各自的分流点，分流点之后的判法同一份）。

证据（冻结副本，读码）：

- 单元数按号取：`冻结:singlefs-core/src/transaction.rs:11134` `data_units_of_the_file_in_the_version`，第一个文件取 `version.data_pointers.len()`，别的取 `file_other_than_the_first(inode)` 那一项（没有条目取 0）；`publish_size_change`（:11164，取数在 :11192）、`publish_truncation_step`（:11643，取数在 :11668）、unlink 的 `next_unlink_step_after`（:11077）三处共用这一个函数。
- 续做按号认领：`冻结:transaction.rs:11655-11662` `previous.inode_record_of(intent.inode)` 比对象出生代、nlink != 0、size == 目标；`TruncationBatch` / `TruncationFinalBatch` 带 `inode`（`冻结:transaction.rs:5920` 附近的 `data_units_after_this_publish` 与 :9344 的 `size_change_of_a_file`）。
- 下段计划同一份：`冻结:singlefs-core/src/truncate.rs:300` `plan_the_lower_segment_after_a_size_change` 入参是该文件自己的下段位置表；第一个文件走 `冻结:truncate.rs:383` `plan_the_extent_tree_after_a_size_change`（它第一步就调 :300 那个，:394 起才是上段），别的文件在 `冻结:transaction.rs:7627` 起的 `SizeChange` 臂里调同一个。
- 标签与单元数核对同一份：写侧 `TailOfAFile::of_data_units_and_size`（`冻结:singlefs-core/src/extent_tree.rs:334`）对两类文件相同；读侧冷恢复 `冻结:recovery.rs:3856` 别的文件按 `file.tail` 调与第一个文件相同的 `data_unit_count_matches_the_inode_size`（:6702）；挂载态读者 `data_units_read_within_the_file_size`（`冻结:write_request_split.rs:95`）按标签分 `NoneWritten` / `SomeWritten` / `HoleAfterTheLastDataUnit`，不分 inode。
- 粘住的标签 0：`冻结:transaction.rs:7672-7674`（`has_an_entry_after_this_publish = previous_file.is_some() || change.data_units_after > 0 || size_after_in_bytes > 0`）；写树时 `冻结:transaction.rs:13456` `(None, None) => ExtentUpperLeafTarget::NoDataUnit`，`with_the_tail` 对 `NoDataUnit` 不改（`冻结:extent_tree.rs:403`）。
- 拒绝成员删净：`grep -rn 'SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses\|is_the_first_file\|names_the_live_first_file' crates/singlefs-core/src` 在冻结副本零命中（我跑的，原样零行输出）。

条款：kb:08-核心索引结构.md:166（截断入口边界一句已写回「对任意活 inode」「截断的两种批带 inode 号」「续做认领全按条目点名的 inode」），判决 `research/prompts/m4-multi-inode-size-r1-main-verification.md` 第 21、24 行（S3、S6 两格）。

什么现象会推翻：任一读者或写者在别的 inode 上取了第一个文件才有的常量（`FIRST_INODE_NUMBER`）而没分流；本节没找到这样的点，但我没逐个读完 `transaction.rs` 全部 17244 行，只读了上列函数与它们的调用链（读过的范围见各行号）。

### T1.2 「供三方核」的四处写法

| 处 | 实现写法（冻结副本） | 条款依据（kb 行） | 判定 |
|---|---|---|---|
| ① unlink 批截带洞的别的文件，尾部换回标签 1、2 | `冻结:transaction.rs:7608-7615` `DeletionBatch` 臂 `tail: TailOfAFile::NoHoleAfterTheLastDataUnit` | kb:08-核心索引结构.md:507 第 2 条「size 改成「剩下单元数 × 净荷容量」」，加 `冻结:unlink.rs:123` `UnlinkStep::Batch` 文档「恒 ≥ 1」。剩下 k 个单元、size 恰为 k × 净荷，`of_data_units_and_size(k, k×净荷)` 为 `k < k` 假，即不是有洞 | 成立：推得出（算术：`冻结:extent_tree.rs:335` 的式子在 size = k×净荷 时给「没有洞」；读侧 `冻结:write_request_split.rs:115-117` 对标签 3、4 要求单元数严格少于 size 除出来的数，所以不改回 1、2 反而会被读者拒） |
| ② N = S = 0、上段本来没有它：只改记录、不写条目 | `冻结:transaction.rs:7672-7674` 三个条件都假（`previous_file` 为 None、单元 0、size 0）则不 push | kb:08-核心索引结构.md:452「号 ≥ 2 的 inode 变长到 size > 0 而一个单元都没有时写它」「`publish_new_inodes` 建出的 size 0 inode 不写」（kb:08-核心索引结构.md:452 同一行）；判决别的 inode 的粘住形见 kb:08-核心索引结构.md:480 | 成立：推得出。注意判决第 20 行括注写「用户 2026-10-24「一律写标签 0」推到任意 inode」，字面读会让 size 0 的别的 inode 也写标签 0；但同一格的主句是「采正推的形」（粘住：变长到 size > 0 才写），kb:452 已把粘住形写成条款，两处合起来不冲突，括注只指标签 0 的含义；如果主 agent 要排除字面读法，需要在判决里一句话说清 |
| ③ 冷恢复对 nlink = 0 的记录同判「有记录、没有条目、size > 0」 | `冻结:recovery.rs:3887-3897`：`record.inode != FIRST_INODE_NUMBER && !inodes_with_an_entry.contains(&record.inode) && record.size != 0`，没有 nlink 过滤 | 判决第 20 行「「有记录、没有条目、size > 0」在挂载态读者、三处冷读者、…给同一个判法」未分 nlink；kb:08-核心索引结构.md:466 同句 | 成立（判决字面没分 nlink，实现没加例外）。另核这样判会不会拒合法历史：nlink = 0 的记录只来自 unlink 意图，意图那次不动 extent 条目（上段原样），中间批 `data_units_kept ≥ 1`（`冻结:unlink.rs:123`）条目留着，最后一批同一次发布删记录与条目（kb:08-核心索引结构.md:507）；所以合法历史里不存在「nlink = 0、有 size、没条目」。没有反例，也没有现成用例我能复核（见 T10） |
| ④ 第一个文件走 `plan_the_extent_tree_after_a_size_change`，别的文件走统一计划，号集合可多出粘住的标签 0 | `冻结:transaction.rs:8311`（第一个文件，传 `FIRST_INODE_NUMBER`）与 :8328 起的 `plan_the_extent_tree_after_the_size_change_of_a_file_other_than_the_first`（:9384，:9406 `entry_changed = had_an_entry != has_an_entry || rewrites_the_upper_leaf_entry()`） | kb:08-核心索引结构.md:507 第 8 条（上段形状只由有条目 inode 的号集合定）；标签 0 是条目（kb:08-核心索引结构.md:452）所以号集合多出它是同一条规则的结果 | 成立：推得出。第一个文件那一支不重算号集合的前提是「第一个文件改 size 时条目一直在」（`冻结:truncate.rs:377` 文档），由用户 2026-10-24 的「一律写标签 0」保证（kb:08-核心索引结构.md:479「用户定案 2026-10-24：第一个文件没有数据单元时 size 0 也写标签 0」；kb:08-核心索引结构.md:452 同义，现查改过：上一条腿原写 :466，那一行讲的是别的），所以集合确实不变 |

什么现象会推翻：① 若存在合法历史让 unlink 批之后剩下的单元数 × 净荷 ≠ 新 size（即 `next_unlink_step` 给出的 size 不是 k × 净荷），那标签 1、2 会错；`冻结:unlink.rs:142-165` 的 size 就是 `size_after_an_unlink_batch(kept, 32634)`，我没发现例外。③ 若出现 nlink = 0 的 inode 在最后一批之前丢了条目。

### T1.3 C145 第三条的 `assert!(released_entry.is_some())`

判定：成立（合法历史里不会触发，坏盘输入也到不了）。

- 断言位置：`冻结:transaction.rs:6537-6546`（`entries_after.remove(...)` 的结果）。放掉的条目来自 `deadlist_entries_the_destruction_of_a_snapshot_releases`（`冻结:transaction.rs:6330-6357`），它对 `version.deadlist_entries` 做 `filter` 再 `cloned`；`entries_after`（:6515 起）用同一份 `previous_version.deadlist_entries` 建 `BTreeMap`。所以只有「两条 deadlist 条目键字节完全相同」时，第二次 `remove` 才会是 None。
- 这条在冷恢复就被拒：`冻结:recovery.rs:4336-4340`（`deadlist_mapping_keys.len() != deadlist_entries.len()` 则 `UnitMalformed`，文字「两条 deadlist 条目同一把映射 key」）；deadlist 条目键含映射 key，键全同必然映射 key 同。
- 只供测试的强制进 deadlist 那一臂另有 :6450的 `assert_ne!`，与本条无关。
- kb 依据：kb:checks-owed.md:920（C145 行「放块时要删的那条条目必须还在」照用户定加断言，等价性留档成用例）。

什么现象会推翻：构造一版 `deadlist_entries` 里有两条完全同键的条目而冷恢复没拒。我没造，只读了 :4336 的拒绝；该检查针对「映射 key」，我认为覆盖，但没有跑过。

（接手说明：以下 T2 起由接手的同立场腿续写；T1 三节为上一条所写，我复核了其中引用的几处行号仍对应冻结副本，见 T1 补记。）

## T2 部分写 core 一侧

约定补充：「冻结:replay-prefix:」指冻结副本 `crates/singlefs-replay-prefix/src/`（部分写的纯计划核在那里，由 core 直接调用，这几个函数不在这一轮 14 份补丁的 diff 里，是被调用的既有核，我只读来判 core 的调用对不对）。判决 `research/prompts/m4-partial-write-r1-main-verification.md` 的格在其第二节表里：R2 第 20 行、R3 第 21 行、R4 第 22 行、R5 第 23 行、R6 第 24 行、R7 第 25 行；kb `04-校验和位置.md` 已定项 3 定案第 52 行、射程第 55–58 行，已定项 4 定案第 71 行（行号去文件里 `grep -nF` 现查过）。

### T2.1 读旧净荷改成读盘（含截断尾单元）

判定：成立。
- 部分写：`冻结:singlefs-core/src/transaction.rs:11553-11570` 对 `reads_the_old_unit_back()` 的单元调 `read_back_the_old_payload_of_a_data_unit`（:11381 起，读 `pool.devices`，不取内存里的版本字节）；截断意图与变长的尾单元：`冻结:transaction.rs:11261-11292` 同一个函数，`payload_of_the_rewritten_tail_data_unit` 已不存在（`grep -n payload_of_the_rewritten_tail_data_unit` 冻结副本 `crates/singlefs-core/src` 零命中，我跑的）。
- 对判决 R3 第 21 行「读盘……截断与变长重写尾单元那条路一起改成读盘」与 kb:04-校验和位置.md:57「截断尾单元的专用读改写不能代替通用部分写」：两条路现在是同一个读函数，符合。
- 读之后的核对：整单元 CRC-32C 对位置条目（`冻结:transaction.rs:11410`），单元头对查找路径（:11432-11455：树、对象、对象出生代、fsid、出生 txg 与写序、锚点偏移、声明长度）、补齐为零（:11456-11457，I-2.3，kb:04-校验和位置.md:71 末句「补齐字节恒为 0」）。D4 已定项 3 定案句（kb:04-校验和位置.md:52）说「验 MAC」；未加密路径只有 CRC（kb:04-校验和位置.md:57 射程「未加密路径执行 CRC，不执行 MAC」），实现只做 CRC，与条款一致；加密开启时的 MAC 路径这一版没有，条款也说无密钥侧不做部分写（kb:04-校验和位置.md:73），不冲突。
- 读失败在分配与写之前：部分写 `:11550-11599` 循环整个在 `publish_deletion_step`（:11624）之前，`?` 提前出口；截断 `:11264-11280` 同理在 `publish_deletion_step`（:11300）之前。错误成员 `OldDataUnitToRewriteNotReadBack` 在 `冻结:transaction.rs:5598`。

什么现象会推翻：任何一个调用点在读盘前先分配或写，或读失败被 `unwrap_or` 吞成零（上面两处都是 `map_err(...)?`，没有）。

### T2.2 两份副本一份读错时的判法

判定：成立（实现是「任一份读到且整单元 CRC 对上就算读到」）；该读法条款没有直说，属「条款没说、推得出」。
- 实现：`冻结:transaction.rs:11402-11412` `read_by_the_locations` 对 `pointer.locations` 两条位置逐条 `find_map`，第一条读不出或 CRC 对不上就试第二条；两条都不算数才走上一版中央映射再读两条（:11413-11429）；全不算数才报 `UnreadableAtTheHintsAndAtTheMappedLocations` 或 `...MissingFromTheMapping`（:11361-11364）。
- 条款：kb:04-校验和位置.md:71「旧单元读不回来、或读回来 MAC 验不过时」没有说单元有两份副本时一份坏算不算「读不回来」。把「单元」读作「两份副本合起来的那个逻辑单元」是推得出的（同一单元的两份内容相同、各有位置条目里的校验和；一份读错而另一份对得上，这个单元读得回，且不会得到零填充）；`冻结:mounted_read.rs:443` `read_unit_via_locations` 挂载态读者同样读法。
- 一处条款没说：kb:19-块指针的结构与宽度预算.md:115 对「释放之前的读盘核」要求先重读一次再下结论，那条只管释放核；读回旧单元这里没有重读（`冻结:transaction.rs:11402-11412` 每条位置只读一次）。一次瞬时坏读会让部分写失败。D4 已定项 4 只说「读不回来」就失败，所以不重读不算违反条款，但条款没有说要不要重读：条款没说。

什么现象会推翻「推得出」：D4 作者本意是「任一份读错就失败」（那样实现比条款宽）。kb 里我没找到支持这个读法的句子。

### T2.3 部分写的切分（`write_request_split.rs`）

判定：成立。
- 冻结:`singlefs-core/src/write_request_split.rs:238-391` 的 `PartialWriteOfAFile`：终点 `checked_write_end`（:256）、写后 size = max(写前, 终点)（:277-279）、写后单元数 `units_after_partial_write`（:301-307）、重写起点 `first_unit_to_rewrite`（:311-323，起点越过末尾且旧尾单元声明长度不满一个净荷时从旧尾单元起）、每个单元来源 `classify_partial_write_unit`（:342-368）。纯核在 `冻结:replay-prefix:partial_write_proofs.rs:54-75`（分类）、:109-119（起点）、:122-138（单元数），`冻结:replay-prefix:file_range_proofs.rs:98-113`（声明长度）、:203-247（单元内字节段）；这些都带 `ensures`，Verus 证过（不由我复验）。
- 对判决：R2 第 20 行「读回次数 = 触及单元里净荷没被整单元盖满的个数；有旧单元读旧单元；没有旧单元读新槽残留一次；起点越过末尾的空隙补整单元零、不读」：`classify_partial_write_unit` 五格与这句逐格相同（OldReadBack / OldCoveredWhole / NewReadResidue / NewCoveredWhole / ZeroGap）。R4 第 22 行「写到 o 时把 o 之后的单元一起重写」：重写段 `[first_rewritten, data_units_after)`，段里 o 之后既有而这次没写到的旧单元分类为 `OldReadBack`（既有、没盖满），照读回再写。R5 第 23 行「单元从 0 起连号，洞只在尾部；空隙补整单元零；T、N_w 不碰」：`transaction.rs:11611-11612` 截断字段照抄。
- 取样点自己量：我写了独立暴力参照 `research/prompts/m4-close-code-r1-sonnet-model/partial_write_plan_model.py`（纯 Python，不编译 Rust；把上面几个纯核的式子移植进去，与逐字节暴力参照比）。参照是字节数组上「在偏移处覆盖、必要时延长补零」；被判是「只重写计划给的那一段、旧单元读回（用内存表代替读盘）、新单元残留丢弃」。取样：单元净荷容量 1、2、3、4、7（用小容量把净荷边界、跨单元、写到尾都走满），旧单元数 0 到 4，旧 size 取各单元数下所有合法值（含尾部有洞），偏移 0 到 6×容量+1，长度 1 到 3×容量+1。检查：重写后还原出的文件内容与参照逐字节相同；段之外的单元与改前逐字节相同且声明长度本来就不该变；单元数连号。命令与输出：
```
nice -n 19 python3 -I research/prompts/m4-close-code-r1-sonnet-model/partial_write_plan_model.py
cap=1 比了 416 例，全相同
cap=2 比了 2058 例，全相同
cap=3 比了 5000 例，全相同
cap=4 比了 9802 例，全相同
cap=7 比了 39688 例，全相同
合计 56964 例，0 例不同
```
  净荷边界取样点（容量 c：偏移 c-1、c、c+1，长度 c、c+1，写到尾 off+len == size_before）都在上面的穷举里（c = 4 时偏移 3、4、5，长度 4、5）。真容量 32634 没在这个模型里跑（算术同式；真值上的边界取样由 `冻结:write_request_split.rs:444-` 的库内单测 `partial_write_read_decisions_match_overwritten_bytes_at_unit_boundaries` 取容量 32634、偏移 {0, c-1, c, c+1, 3c}、长度 {1, c, c+1}，我读了源码，没跑它，因为那条要 cargo）。
- 真 Rust 代码上的对拍（接手时上一条腿的探针 `probe_partial_write_split.rs`，源码已拷进模型目录）：冻结副本拷贝（`singlefs-core` 加两个路径依赖，`copy/`）里把 `PartialWriteOfAFile` 用真容量 32634 对暴力字节数组参照比。上一条腿排队的那条 `cargo test --no-run` 在我接手后自己轮到了，编成（`Finished test profile in 28.95s`）；我对它排队中的进程想停掉，权限被拒，所以它照编完了。第一次跑探针红在 `内容对不上 n=0 size=1 off=1 len=1`（`out/probe_partial_write_split.first-run-failed.txt`）：这是探针自己的参照错——它把没有单元的洞（n < ⌈size ÷ 净荷⌉ 的那一段）当成有字节的旧内容，真实现里洞读出 0。我在拷贝里改成「前 n 个单元之外的旧字节取 0」，重编（`cargo test --offline -p singlefs-core --test probe_partial_write_split --no-run`，经 `capped.sh 31` 与内存包装 4G，`Finished … in 5.83s`），再经内存包装 2G 跑那个测试二进制，输出 `name=probe_partial_write_split cases=2205 classes={…}`、`test result: ok. 1 passed`（`out/probe_partial_write_split.run.txt`）。五种来源各至少 201 次（旧单元盖满 201、旧单元读回 1052、新单元盖满 1006、新单元读残留 2367、空隙补零 1953）。探针比的是：写后 size、写后单元数、重写起点、不重写的单元声明长度不变且没被写碰、每个单元的来源分类与「是否盖满、是否碰到」逐字节条件、重写后重组的文件内容。结论：T2.3 在真代码上也成立，不只是我的 Python 移植。cargo 不进 `rerun.sh`（要内存队列与编译）；命令在上面，源码在模型目录。

什么现象会推翻：真实代码里的计划函数与我移植的式子不同（我对着源码逐式移植，但没把移植版与 Rust 版跑在一起对拍）。

### T2.4 新错误成员在落盘之前交回

判定：成立。
- 成员：`OldDataUnitToRewriteNotReadBack { inode, unit, read_back }` 在 `冻结:transaction.rs:5598`，外层 `PublishError::TruncationRefused`（:11562-11570、:11272-11280）。其余入口拒绝都在读之前：未知 inode（:11495）、首文件删后出生的 inode（:11499-11510）、nlink 0（:11511）、终点溢出（:11519-11538）、记录数超上限（:11541-11545），全在 `publish_deletion_step` 之前，不写盘。
- 外层成员名「改 size 的拒绝」装部分写的读不回：`冻结:transaction.rs:11474-11475` 文档自己写明；判决没规定外层名，实现员也把「另立 PublishError 外层成员要不要」交主 agent（事实表 m4-partial-write-model-writer 一节，result.md 第 105 行）。条款没说。
- 一处不是「交回」的：`冻结:transaction.rs:11514-11518` 和 :11238-11241 对「承诺类意图队列不空」用 `assert!`（panic）而不是类型化错误；文档写明调用方（会话）先走空（`# Panics`，:11477-11479），D8 已定项 16 第 6 条是依据。挂载后队列里有续做的意图时会话先走空，所以合法历史不触发；直接调入口的调用方才会触发。这不算「新错误成员没交回」，列在这里是因为它是同一个入口上的 panic 面。

什么现象会推翻：任何一个在 :11624 之前就已调用 `allocator` 或 `pool.write` 的路径；我读到的范围内没有。

### T2 残留读（R2 第 20 行、kb:04-校验和位置.md:56）

判定：成立，附一处文字对不上的小点。
- 读一次、字节丢弃、读不出记数不拒写：`冻结:transaction.rs:14387-14425` 每个 `ReadOnceAndDiscardTheBytes` 的单元读一次整单元，`.is_some()` 记 `note_a_residue_read`（:419-425）；新净荷在入口按零装（:11575-11579），读到的字节没有进 payload 的路径。
- 小点：文档（:14385）写「第一条位置项那块盘」，代码取的是 `reader.first()`（池里第一块盘，:14397），不是 `pointer.locations[0].device`。两者一致的前提是池里设备次序与位置条目的设备身份升序（kb:19-块指针的结构与宽度预算.md:255「位置条目按设备身份升序」）相同；我没核池里设备数组一定按身份升序排。条款只说读「新槽的残留一次」（kb:04-校验和位置.md:56），不挑盘，所以不影响判定。

## T3 部分写进模型与 RefFS

### T3.1 F2 的读法（一份读错仍成功、两份都读不回才失败）是不是 D4 已定项 4 推得出

判定：条款没说（字面没有）；在判决已采的故障格下推得出，不冲突。
- 实现读法：同 T2.2，`冻结:singlefs-core/src/transaction.rs:11402-11429`，任一份整单元 CRC 对上就读到；`冻结:transaction.rs:11361-11364` 三种失败都是「两份（含映射落点）都没读到」或「读到了与查找路径不符」。
- 条款字面：kb:04-校验和位置.md:71「旧单元读不回来、或读回来 MAC 验不过时，那个部分写失败并返错」。「读不回来」没写单元有两份副本时的一份。判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第 25 行（R7 一格）写「故障格照正推 R7.1」，正推 R7.1 的 F2 行是「读坏（一份）……成功（读第二份）」（`research/prompts/m4-partial-write-r1-sonnet-output.md:352`），所以 F2 的成功是判决已采的格，不是实现员自己加的读法；正推 R7.4 的推翻条件（`research/prompts/m4-partial-write-r1-sonnet-output.md:385`）写明「读不回」指两份都读不回，是「推的，要用户确认」。我没找到用户确认的记录；判决第四节「交用户」表（`research/prompts/m4-partial-write-r1-main-verification.md` 第 34 行起）里没有这一项单列，末尾「回看决策」表第 58 行写 D4 已定项 4「不受影响：读坏必失败照旧」，没有分一份与两份。
- 结论：实现的读法 = 正推 R7.4 的读法 = 判决 R7 采的 F2 格；D4 已定项 4 字面没有分一份与两份。可以说「推得出」的依据只有一条：同一单元两份内容相同，另一份对得上就没有数据丢，不会出现 kb:04-校验和位置.md:71 禁的那一形：不许拿零填充凑出一个新单元（kb 原话：那会把「一段数据丢了」变成「一段合法的零」）。
- 另一个读法的后果：若用户把「读不回」读成「任一份读错就失败」，实现要改 `冻结:transaction.rs:11402-11412` 成「两份都要对上」，并且现有的 F2 成对用例翻成失败。

什么现象会推翻「不冲突」：用户的原意是任一份读错就失败。记录里没有，所以保持「条款没说」，需要用户一句话确认（这一条本来就在正推 R7.4 的待确认里，实现员只是沿用）。

### T3.2 RefFS 允许集合里部分写的中间态与实现、理想模型是不是同一件事

判定：成立（RefFS 一侧）；实现一侧与理想模型一侧的崩溃面复核不了。
- RefFS：`冻结:singlefs-reference-filesystem/src/lib.rs:63-66` `Operation::WriteAtOffset` 与 `OverwriteTheFile`、`Truncate` 同归 `EnteredKind::ContentChange`（:665-690 的 `submit` 里 :631、:640、:658、:686 同一个种类；:241 `ContentChange => false` 说明它不是发布点）。所以崩溃的允许集合与整份写、截断同一套：写前、写后两样，没有写到一半的内容。用例 `冻结:lib.rs:2093-2130`（崩在按偏移写那次发布里，断言恰是写前 "abcdef" 与写后 "abcdXYZ" 两样）。
- 内容算法：`冻结:lib.rs:665-690`：偏移与终点之间补 0、终点越过末尾则长到终点、盖住那一段。对照 T2.3 的参照（暴力字节数组），同一个定义，我的模型与它相同（`research/prompts/m4-close-code-r1-sonnet-model/partial_write_plan_model.py` 里的参照就是这三行语义）。
- 与实现是不是同一件事：实现 `publish_partial_write` 把整次写做成一次发布（一次 `publish_deletion_step`，`冻结:transaction.rs:11624`），重写的单元全在这一次里，没有「先写前一半单元、另发一次后一半」的分段；所以崩溃可见的状态只有写前根与写后根两样，与 RefFS 的 `ContentChange` 假设一致。判决 R4（第 22 行）选「写到 o 时把 o 之后一起重写」，也是为了留在一次发布里。
- 复核不了的部分：这个「一次发布里崩溃只读出写前、写后两样」的实现一侧证据是崩溃点枚举（`冻结:singlefs-checker-tier/tests/crash_enumeration_partial_write_stream.rs`），属 checker 档，本轮不许跑；实现员报告也写「只编、没跑」（事实表 m4-partial-write-writer 一节，result.md 第 149 行）。RefFS 翻译（`冻结:singlefs-harness/src/reference_filesystem_translation.rs:524-544`）在 harness 里，不在这一轮的补丁射程内，我只确认它构造 `Operation::WriteAtOffset`。
- 两处模型的小点：（一）RefFS 只收第一个文件：`冻结:lib.rs:659-690` 的状态只有 `NoFile` / `OneFile`，别的 inode 的部分写没有进 RefFS（实现员报告写 RefFS 翻译只翻第一个文件，`冻结:reference_filesystem_translation.rs:524` 注释）；别的 inode 的部分写由理想模型对拍，不由 RefFS 判。（二）长度 0 的写：RefFS 在 `冻结:lib.rs:665-690` 不拒，实现类型上写不出（`冻结:transaction.rs:11321-11323`，`PartialWriteBytes::new` 拒空）；判决没判（见 `冻结:transaction.rs:11311-11312` 文档），条款没说。

什么现象会推翻：实现的部分写在一次调用里发了两次根（例如大写分段发布）；`publish_partial_write` 只调一次 `publish_deletion_step`，我读到的范围内没有第二次。

## T4 发布触发

约定：判决 = `research/prompts/m4-publish-trigger-r1-main-verification.md`，P1–P7 在它第二节表的第 20–26 行；kb = `16-发布语义.md`（已定项 2 定案第 107 行、已定项 5 定案第 164 行与 166 行、已定项 10 定案第 270 行、已定项 17 定案第 374 行，行号现查）。代码都在 `冻结:singlefs-core/src/mounted_session.rs`（下称 ms）、`system_configuration.rs`、`transaction.rs`。

### T4.1 触发判据只在操作期间不变的输入上分支

判定：成立。
- 判据：`ms:4880-4897` `publish_trigger_cause(pending_is_empty, time_since_the_last_publish, journal_bytes_of_the_pending_changes, thresholds)`，与 kb:16-发布语义.md:107 的式子「触发 ⟺ 待发非空 ∧ ((now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty))」逐项相同：空则 None（:4886-4888），时间支 `>= time_threshold`（:4889），脏量支 `>= dirty_threshold`（:4890），三种结果 `TimeThreshold` / `DirtyThreshold` / `BothThresholds`（:4891-4896，分支可观测，fs-design 五条硬要求第 4 条）。
- 分支变量是两个阈值：`PublishTriggerThresholds` 只有 `of_the_mount`（挂载求一次，`冻结:system_configuration.rs:458-466`）与特性后的测试来源两个构造；会话字段 `thresholds` 私有、全文件无第二处赋值（`grep -n 'thresholds =' ms` 只有 :5042 的挂载取值）。`now`、待发的 journal 字节是被比较的量，不是分支变量，符合 `.claude/rules/fs-design.md:106-107`（分支三条约束第 1 条）与 kb:16-发布语义.md:109（「已知边角」写明「`now` 与脏字节数是被比较的量、会变，不是分支变量」）。
- 读数倒退：`ms:5113-5117` 用 `saturating_sub`，倒退时间隔 0；`ms:5104-5110` `note_a_publish` 取 `before.max(now)`，上次读数只往前走。对判决 P1（第 20 行）「读数倒退时当作没过时间（不回拨）」与 kb:16-发布语义.md:166，成立。
- 触发只在两处判：`accept` 收下之后（`ms:5214-5225`）与 `publish_when_due`（`ms:5391-5402`）；一个请求（如一次多单元顺序写）是一次 `accept`，不被切开（P5，第 24 行）。
- 一处小点：`accept` 在被拒绝（冻结、盘表不符）时也先执行了 `note_the_reading_seen`（`ms:5174` 在 :5175 的冻结判之前），第一次见到的读数当基线会因此被一次被拒的调用设定。它不碰盘、不碰待发；基线取「第一次见到的读数」本身是实现员的选择（kb:16-发布语义.md:164 说「上次 checkpoint 的时刻」，没说还没发布过时取什么；实现员报告 R:73 已列），条款没说。

什么现象会推翻：任何一处在 `accept` 调用中途重新读钟或重新求阈值。`accept` 只用入参 `now`；`run_compaction_in_the_background` 里 `caller.clock_reading()` 读一次（`ms:5480`）后传下去，没有再读。

### T4.2 只供测试的强制开关（P6，判决第 25 行）

判定：成立（阈值开关）；已有的整理、删除、空间准入开关按判决不在这一行。
- 两个阈值的测试来源 `of_test_only_values_clamped_by_the_journal_ring` 在 `#[cfg(feature = "test-only-publish-thresholds")]` 后面（`冻结:system_configuration.rs:471-472`），特性在 `冻结:singlefs-core/Cargo.toml:37` 登记、没有 `default` 条目（文件里 `[features]` 一节只有 `lookup-work-counters` 与它两项，我读到第 37 行为止）；唯一打开它的是 harness 的 dev 依赖（`冻结:singlefs-harness/Cargo.toml:19`，在 `[dev-dependencies]` 下，:16 是该节标题）；全仓别的 Cargo.toml 没有打开它（`grep -rn test-only-publish-thresholds --include=Cargo.toml` 只命中这一处加 core 自己那一行）。
- 类型文档里的文档测试用 `cfg_attr` 在特性关时写成 `compile_fail`（`冻结:system_configuration.rs:438-439`），是「发布构建够不着」的编译期钉子；但它是文档测试，我没跑它（要 cargo）。实现员报告也写「P6 的编译期检查是文档测试，没跑」（事实表 m4-publish-trigger-writer-2 一节）。
- 每条分支能被强制进入（fs-design 五条硬要求第 2 条）：时间支靠 `time_threshold`、脏量支靠 `dirty_threshold`，测试来源可把任一个压到几毫秒或几条记录；夹取照常做（`冻结:system_configuration.rs:486-495`：给的是夹之前的值，有效值 = min(值, 环长 ÷ F)，与 kb:16-发布语义.md:164 一致）。
- 默认有效值：时间 5000 ms、脏量 2 GiB（`冻结:system_configuration.rs:102-103`），默认环下夹到 256 MiB（kb:16-发布语义.md:164「默认环下有效值 = min(2 GiB, 256 MiB) = **256 MiB**」）；文档里「无视 5 s 与 256 MiB 的判据」（`冻结:system_configuration.rs:467` 注释）指有效值，不是矛盾。
- 挂载选项通道没有（C422）：产品路径上两个阈值只来自常量，盘上偏移 433 的 `T_time` 字段并没有被读成有效值——与 kb:16-发布语义.md:162 的「实现边界」一致（那一段自己写「运行配置没有挂载可设入口」）。

### T4.3 屏障与进门预判的次序

判定：成立；其中三处「第一版不收」是条款没说。
- 次序（`ms:5168-5232` 的 `accept`）：记读数 → 冻结则拒（:5175-5182）→ 盘表不符则拒（:5183-5196）→ 按类分流（:5197-5208）：屏障（unlink、截断、建删快照，`ms:4853-4856`）走 `publish_a_barrier_alone`（:5350-5384）：先排空待发、再单发，两次都报；数据写类先做进门预判 `judge_a_data_change_before_any_read_or_write`（:5240-5347）、过了才 `take_in`（:5211），再判触发（:5214）。这与判决 P3（第 22 行）「unlink、截断、建删快照、整理、回退、卸载是屏障，进门先排空待发、自己单发；accept 进门先做这一批合起来的准入与条数预判，过不了就在任何读写之前拒」、kb:16-发布语义.md:270 逐句对上。回退与卸载只经 `into_session_after_publishing_the_pending_changes` 交出会话（`ms:5541-5561`），类型上先排空；整理只在 `run_compaction_in_the_background` 里判过起跑才排空（`ms:5471-5534`，排空失败时把「该跑了」放回去，:5506-5510）。
- 进门预判在任何读写之前：`ms:5308-5315` 与 :5331-5345 调的 `journal_bytes_of_data_changes_published_together_judged_before_any_read_or_write`（`冻结:transaction.rs:10093`）先 `refuse_while_a_publish_is_frozen`（:10094）再拼同一份计划、走 `judge_the_publish_before_any_read_or_write`（:12307，注释自己写「分配器只读、一个盘都不读、一个写都不发」）。
- 判决 P3 的待核前提「accept 进门能否做不拷分配器的只读预判」：实现做的是「固定点在分配器的拷贝上走」（`冻结:transaction.rs:12296` 文档与 :12317 的注释），也就是拷了；判决说做不到要停下报主 agent，实现员照做得到的那一半做了并在「停下交主 agent」第 1 条报了（事实表该节，result.md 第 212 行）。我的判定：预判没有写、没有读盘，满足「在任何读写之前拒」；「不拷分配器」那一半没做到，是否算违反由主 agent 定——判决与条款都没有说拷贝本身不许。
- 三处「第一版不收」，条款没说（`ms:4968-4981`、:4973-4979）：按偏移写不进待发（`WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion`，kb:16-发布语义.md:270 只列了整份写、建 inode 与屏障）；承诺类意图队列没走空时数据写类 `accept` 拒（:5254-5256，走空是写，预判要在任何写之前，所以不能在 accept 里走空，调用方先经 `publish_user_change` 发一次）；写待发里建的那个号（:5274-5280）。都在任何读写之前拒、待发不动，不违反「返回成功之后不拒」，只是调用方要处理这三种拒。

什么现象会推翻：某条路径先改了待发或先写盘、再拒。三种拒之前 `self.pending` 都没动（`take_in` 在 :5211，所有拒绝在它之前）。

### T4.4 报错之后并进冻结的那一次

判定：成立，附一处窄的角（条款没说分开处理）。
- 处理：`ms:5128-5153` `publish_the_pending_changes`：先 `std::mem::replace` 取走待发，发布失败时 `frozen_publish().is_none()` 才把待发放回（:5147-5149）；冻结了就不放回，等于并进冻结的那一次。对 kb:16-发布语义.md:374「fsync 报错之后，这次发布的数据可以出现也可以不出现」与 :270「发布写失败冻结之后 accept 在任何读写之前拒」：冻结后 `accept` 拒（:5175-5182），待发已空，交出会话（`ms:5541`）排空得 `None`，调用方原样重发冻结的那一次。成立。
- 窄的角（实现员「停下交主 agent」第 8 条，result.md 第 212 行）：被拒绝的发布若先推了抬 F（空间不够，`冻结:mounted_session.rs:2030-2034` 文档「被拒是空间不够就推一串（至少一次空发布）」），那个空发布的写失败而冻结时，冻结的是那个空发布，批本身根本没进冻结的那一次，仍被丢（`ms:5147` 只看有没有冻结，不看冻结的是谁）。可复现的最小形：池空间紧到排空待发被准入拒 → 抬 F 的空发布 → 该空发布写失败。结果：调用方被告知「已收下」的改动确定丢失（不是「可能出现」）。kb:16-发布语义.md:374 允许「不出现」，所以不违反条款；但判决 P3（第 22 行）说的新不变量「已 accept 的改动必须发布得出去」在这个角上不成立，调用方也无法重发（内存里没有副本）。我没能实际造出这个态（要把池压满再注入写失败，属 harness/checker 档），属读码推出，复核不了。
- 另一小点：`publish_the_pending_changes` 失败时不更新上次读数（`ms:5146-5150` 没有 `note_a_publish`），即使抬 F 或走空的几步已发成；效果只是下一次触发判得更早，不丢改动，无条款可对。

### T4.5 「D16 已定项 10『同一 inode 的写折叠』管不到两个号」

判定：条款没说。
- 今天的做法：待发里已有另一个 inode 的整份写时，`accept` 拒成 `WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion`（`ms:4973-4979` 成员，:5258-5267 判），调用方要先 `publish_pending_now`。理由写在成员文档：一次发布写一个文件以内是这一版发布计划的形状（`冻结:transaction.rs` 的 `PublishPlan::file` 是 `Option`，`冻结:transaction.rs:5825`，用法见 :12283 `plan.file.is_some()`）。
- 条款：kb:16-发布语义.md:270 写「数据写类可并：同一 inode 的写折叠、建 inode 求和」，判决 P3（第 22 行）同。这句只说了同号折叠与建 inode 求和，既没说不同号的写可以并进一次发布，也没说不可以；「可并」是允许，不是要求。所以两个号的写攒进同一次发布，条款既推不出也推不翻；今天的「拒并」是更窄的实现，不违反条款，但让多文件交替写的负载每换一个号就发布一次，和「发布粒度取攒批」的收益（kb:16-发布语义.md:270 射程第二点）有落差。

什么现象会推翻：另有一句条款或用户定案说两个号的写必须能并（我没找到）。

## T5 块设备信任边界

约定：`冻结:bd` = `冻结:singlefs-core/src/block_device.rs`。改前版本取自 `git show HEAD:crates/singlefs-core/src/block_device.rs`（HEAD 里这份文件没有别的会话的改动，14 份补丁里只有 block-device 补丁动它），存在草稿目录 `bd-head.rs`（2194 行）；我对两份做 `diff -u -I '^\s*//'`（忽略注释行），产物在草稿目录 `bd.diff`（1796 行），读的是它去掉空行和文档行之后的 + / − 行。信任清单 = `冻结:.claude/gate.d/verus-trust-list.tsv`，block_device 的条目在第 21–34 行。

### T5.1 执行代码改写前后是不是同一个行为

判定：成立，附两处不严格同义但不可达的点。
- 对齐与越界：`bd:635-672` `check_aligned_and_in_range` 的函数体与改前（`bd-head.rs:502-524`）逐字相同，只加了 `ensures`；先判对齐（`Unaligned`）再判越界（`OutOfRange`，`checked_add` 溢出也归越界），错误种类与次序不变。规格 `request_is_aligned_and_in_range`（`bd:230-240`）把物理块 0 取成「只有 0 算整数倍」，与 `u64::is_multiple_of(0)` 的取法相同，注释自己写明（`bd:221-227`）。
- 零填充分块长度：`bd:683-688` `zero_fill_chunk_length` 函数体不变（`length.min(ZERO_FILL_CHUNK_BYTES)`），`ensures` 与之同式；`bd:697-728` `write_zeroes_in_chunks` 的循环体逐行不变，只把参数 `FnMut` 换成 `Fn`（`mut write_chunk` 去掉）并加了 `invariant` / `decreases`。两个后端的回调原先是 `|…| self.file.write_all_at(…)`（闭包借 `self`），现在先 `let file = &self.file` 再借 `file`（`bd:905`、`bd:1199`），调的是同一个 `write_all_at`。
- 三个系统调用外壳 `read_exact_at_the_offset` / `write_all_at_the_offset` / `sync_data_of_the_file`（`bd:768-790`）函数体就是原来那一句 `file.read_exact_at` / `write_all_at` / `sync_data`；调用点把 `.map_err(InputOutput)?` 写成 `match`，错误成员与出口次序相同（对 `?` 与 `return Err` 逐处读过 `bd-head.rs` 对应行）。
- 写零 / discard 里「模拟 errno」的三处 `Err(io::Error::from_raw_os_error(libc::EINTR / EOPNOTSUPP / EIO))` 换成 `hole_punch_error_the_test_switch_stands_for(…)`（`bd:580-587`），函数体按 `HolePunchFailure` 三个成员返回同样三个 errno；再经 `classify_hole_punch_failure`（`bd:568-577`）分回原路：这是一处间接，不是同一个表达式，行为同。
- 计数：`+= 1` 全换成 `saturating_add(1)`（`bd:108-124` 的 `DiscardOutcomeCounts::note`，写零 / discard 的几个计数）。到 `u64::MAX` 之前结果相同；之后改前是 debug panic / release 回绕，现在停住。这一点不严格同义；不可达（要 2^64 次调用）。
- 日志：`Ghost` 日志的 push 全是幽灵代码，产品构建里擦掉，不改运行行为。位置（`bd:821-825` 写、`bd:849-853` 写零、`bd:1017` 屏障）都在真正发系统调用之前，与特质规格「`Err` 时日志不变或多这一条」相容。
- 长度 0 的写零 / discard：`bd:854` 与 `bd:942` 先 `check_request` 再 `return Ok`，不调 `fallocate`，与改前相同；写零的日志在 `length == 0` 的早退（`bd:854`）之前就 push 了（`bd:849-853`），discard 的日志只在 `length > 0` 且打洞成功时 push（`bd:985-991`），分别对上特质里两个 `ensures`（`bd:344-366`、`bd:382-409`）。

什么现象会推翻：任何一条错误分支换序，或 `Fn` 回调里有状态（`FnMut` 才写得出的计数）。回调体只有一次 `write_all_at`，没有状态。

### T5.2 信任清单里的几样，差分测试罩不罩得住它们声称的规格

判定：罩得住「它们声称的」；下面三处是规格没声称、靠别的东西撑着的，单列。
- 逐条对：清单第 31 行 `read_exact_at_the_offset` 声称「缓冲长度不变」（`bd:768-775` 的 `ensures`），差分测试 `the_file_backend_keeps_the_block_device_contract_on_every_request`（`bd:2810`）实际比的是整块盘读回与模型逐字节相同（`bd:2697-2718`），比声称的强；第 32、33 行 `write_all_at_the_offset`、`sync_data_of_the_file` 没有 `ensures`，不声称什么，测试比字节、比屏障交回 `Ok`；第 28、29 行 `classify_hole_punch_failure` / `hole_punch_error_the_test_switch_stands_for` 的差分测试 `only_unsupported_errnos_send_a_failed_hole_punch_to_the_zero_byte_fallback`（`bd:1921`）与 `every_simulated_hole_punch_failure_carries_its_errno_and_is_sorted_back_onto_its_own_path`（`bd:2927`）都在文件里；第 30 行打洞段的差分测试 `punching_ranges_off_the_filesystem_block_boundaries_reads_back_zero_keeps_the_neighbours_and_the_file_length`（`bd:1715`）用真 `fallocate` 在 ext4 临时文件上跑。清单里点名的测试名我逐个 `grep -n 'fn <名>'` 都在 `bd` 里，没有悬空的。
- 规格没声称的点一：`AlignedScratch::new`（清单第 34 行，`bd:1055-1072`）只声称「窗口装得下、`length` 等于要的」，没有声称「起点按 4096 对齐」也没有声称「内容全 0」。后者是 `O_DIRECT` 后端写零的正确性前提（`bd:1198` 起，用这块暂存区当全 0 源）。对齐由测试 `aligned_scratch_starts_on_a_4096_boundary_for_every_length`（`bd:2398`）取 {512, 4096, 16384, 32768, 1<<20} 五个长度核，长度 0 与非 4096 倍数的长度不在这五个里；「全 0」靠 `the_direct_backend_keeps_the_block_device_contract_on_every_request_on_both_arms`（`bd:2824`）的「写 → 写零 → 读回比模型」间接罩住。证明的结论（日志里记了 `WriteZeroes`）不依赖这两样，所以不是证明的洞；是 `O_DIRECT` 路径正确性依赖测试而非证明。
- 规格没声称的点二：discard 的规格（`bd:382-409`）声称「交回 `Err` 时日志不变」，即失败的 discard 没有动介质。对应的差分测试 `a_discard_whose_hole_punch_fails_with_an_input_output_error_is_reported_and_changes_nothing`（`bd:2513`）用的是只供测试的开关 `ActAsIfPunchingFailedWithAnInputOutputError`（`bd:978`），这个开关根本不调 `fallocate`（直接造一个 EIO），所以它证的是「造出来的 EIO 走到报错那一路、没写零」，罩不住「真 `fallocate` 在中途 EIO、介质已部分被打洞」。写与写零的规格对 `Err` 留了「日志不变或多这一条」两可（`bd:257-269` `issued_operations_after_the_call`），discard 没留。discard 本来按 D13 已定项 4 的两态建模（读回 0 还是旧字节由设备定），真 EIO 后的介质状态在模型里没有位置。判定：这一格规格比实际能保证的强了一档，罩不住，建议主 agent 把 discard 对 `Err` 也改成两可（不改行为，只放宽规格），或在清单里登记这是个信任假设。我没有改动建议的更多细节。
- 规格没声称的点三：被信任的外壳里没有一个声称「持久」。`sync_data_of_the_file` 的清单条目自己写「持久与否测试看不见，归崩溃点重放与真设备那一侧」（清单第 33 行），与此一致；属于清单已承认的缺口，不是我新发现的。

### T5.3 证明对 SMT 的负担有没有异常

没有从源码读出异常，判定：条款没说、推测。`check_aligned_and_in_range` 的 `ensures` 涉及 `int` 上对符号模数（`physical_block_size.0` 这个 u32 变量）的 `%` 与 `==>` 的多路组合（`bd:644-654` 三条），符号模数的取余是 Z3 的非线性算术，这是三个函数里最可能慢的一个。`length_of`（`bd:673-681`）与 `zero_fill_chunk_length`（`bd:683-688`）函数体是一句，`ensures` 是等式或一个 `if`，本身不该重；它们在负载 80 时超过 600 秒、负载 1 时几十秒，更像是 vstd 的 `try_from` / `expect` / `Ord::min` 规格在整个 `verus!` 模块的上下文（本模块有近二十个 `open spec fn` 与一个 `ghost enum`）里被 Z3 反复实例化，或 rlimit 之外的墙钟超时；我没有跑 Verus（不许起），没法区分。

什么现象会推翻 T5.1：改前后对任一请求（偏移、长度、物理块、设备大小）错误种类不同。穷举对拍我没做（需要把改前改后两版都编出来跑），`the_file_backend_keeps_the_block_device_contract...` 里的 `CONTRACT_REQUESTS`（`bd:2673-2684`，含 `u64::MAX - 511` 溢出）是现成的取样，它跑的是改后版，改前版没有对照跑。

## T6 编解码与记录格式的执行代码改写

约定：`冻结:core` = `冻结:singlefs-core/src`。改前版本取自 `git show HEAD:crates/singlefs-core/src/<文件>.rs`，存草稿目录 `<文件>-head.rs`（`bytes`、`checksum`、`pointer`、`unit`、`root_record`、`records`、`logged_operations`、`journal` 都取了）。我自己写了两个只读小脚本放在草稿目录：`panic_sites.py`（逐文件抽出非测试代码里的 `assert!` / `assert_eq!` / `.expect(` / `unreachable!` / `panic!` / `unwrap()` 的行，连同所在 fn，改前改后按多重集比）与 `trunc_sites.py`（找 `min` / `saturating` / `take` / `as u*` 一类的静默截断；这一个的匹配把 Verus 规格里的 `.take(`、`.last()` 也算进来，噪音太大，T7 没用它的结果，只当线索）。

### T6.1 每一处是不是同义

判定：成立（抽样 + 机械比对）；全部 146 个 fn 我没有逐个读完，下面写读了什么、没读什么。
- 机械比对（我跑的）：`python3 -I panic_sites.py bytes checksum root_record journal unit records logged_operations pointer`，非测试代码里能 panic 的写法，改前改后多重集差只有三类：（一）`assert_eq!` / `assert_ne!` 换成同一条件的 `assert!`（`pointer` 的 `write_to` 6 处、`unit` 的 `build_packed_unit` / `build_index_node` / `parse_index_node` / `parse_packed_unit`、`records` 的 `build_mapping_entry` / `key_bytes` / `written_by_the_truncation_intent_of`、`journal` 的 `write_to` / `to_bytes` / `parse`、`root_record` 的 `parse_slot` 末尾位置断言）；（二）`try_into().expect("切了 N 字节")` 换成 `ByteReader::get_*` / `copied_into_an_array`（`bytes::get_u16` / `get_u32` 里多了一个 `u16::try_from(wide_value).expect(..)`，`wide_value` 是 2 或 4 个字节拼出来的，装得进，不可达）；（三）`checksum::wide_checksum_field_of_the_crc` 多出四个 `u8::try_from(crc % 256 ...).expect(..)`，取余后装得进一个字节，不可达。没有新增 `unreachable!` / `panic!`（改前改后都是 0，`logged_operations` 里那一处两边都有）。
- 改前改后 panic 面不变，错误分支不变：我逐字对读了 `冻结:root_record.rs:406-480`（对 `bd-head` 同款的 `root_record-head.rs:174-224`）`parse_slot`：魔数 → 整槽校验和 → fsid → flags → 四条指针，次序相同；唯一的不同是返回结果里的 fsid 取 `*expected_filesystem_identifier` 而不是读出来的那 16 字节，前一步已判过两者相等（`:426`），结果里取期望的那份（`:480`），同值。`冻结:records.rs:406-432` `InodeRecord::parse` 把 `chain().any()` 拆成两个 `every_byte_is_zero`，仍写在 `||` 的短路之后（长度不是 140 就不切片），长度错的输入不会 panic，与改前同。`冻结:records.rs:1069-1081` `data_key_tail` 取 `transaction % 2^48` 写 6 字节小端，等于改前 `to_le_bytes()[..6]` 的低 6 字节。`冻结:journal.rs:509-525` `record_offset` 的函数体与改前（`journal-head.rs:238-247`）逐字相同，只加了 `requires`。
- 码 2/3 读者拒收的集合：`冻结:journal.rs:947-950` 把 `496 + 56n > 4096` 改成先比 `n > (4096 − 496) / 56`（= 64）：n = 64 时 496 + 3584 = 4080 ≤ 4096 收下，n = 65 时 4136 > 4096 拒，两种写法的收下集合相同（我算了边界），拒的方式都是 `None`，次序对结果不可见（失败都是同一个 `None`）。
- 没读到的：`bytes.rs`（+728 行）、`checksum.rs`、`pointer.rs`、`unit.rs`（+1858 行）、`records.rs`（+905 行）、`logged_operations.rs`、`journal.rs` 其余 fn 的改写，我只靠上面的机械比对、实现员的同义改写表（codec-writer-2 报告第二节表，result.md 第 46–73 行；codec-writer 报告第二节表，result.md 第 43–57 行）和 core 库测试（实现员报告：codec 第一批 362 条、第二批 366 条、tighten 367 条、journal 预分配 372 条全绿，都是实现员自己报的，我没复跑，cargo 要进内存队列）。

什么现象会推翻：任何一个读者对同一串字节，改前交 `Some`、改后交 `None`（或反过来）。我没有造改前改后对拍的装置（需要把 HEAD 版与冻结版各编一遍再喂同一批字节）；T10 里能看到哪些变异行罩着这些读者。

### T6.2 往返契约在格式常量边界上取不取得到

判定：条款没说「边界取不取得到」；证明层面成立，运行层面靠调用点的常量。
- 证明层面：`lemma_packed_unit_round_trip` / `lemma_index_node_round_trip` 的 `requires` 就是写者的 `requires`（`冻结:core/unit.rs:689` `spec_packed_unit_writer_accepts`、`:920` `spec_index_node_writer_accepts`），收紧之后两条引理不再多要前提（tighten 报告 result.md 第 20–22 行）。写者能接受的输入就是往返引理说读者能收的输入。
- 边界：写者的 `requires` 里记录数 ≤ 65535、记录区装进一个单元、写序事务号装得进 6 字节（`冻结:core/unit.rs:689-710` 的谓词）。调用点（见 T6.3）全部传格式常量，没有一个会压到这些边界；边界值本身（比如恰好 65535 条）没有调用点。
- 读者比写者多拒的两种（空节点而条目宽窄于 key 宽；记录宽 0 而记录数非 0）：收紧之后写者的 `requires` 把它们排除，但函数体不判（`冻结:core/unit.rs:703` 文档自己写「函数体不判它、不为它 panic」），所以经没验的调用方（verus! 之外的所有调用方）仍能写出读者拒收的节点；这正是 `checker_known_bad_images` 等测试造坏镜像要用的口子，不是缺陷。

### T6.3 码 2、码 3 写者新收严的前提在每一个执行调用点是否成立

判定：成立（产品路径全部成立）；函数体没有新增判断，所以即使不成立也不会新 panic。
- 前提：`build_index_node` 要 `key_width <= entry_width`（空节点也要）；`build_packed_unit` 要 `record_width > 0`。两个函数体没改判法（tighten 报告 result.md 第 6 行：「函数体一处运行时判法都没改」；我对读 `冻结:core/unit.rs:947-1060` 里仍只在有条目时逐条断言 `entry.len() >= key_width`）。
- 我把冻结副本 `crates/singlefs-core/src` 里非测试调用点一个个对了 key 宽与条目宽（`grep -n 'build_index_node(\|build_packed_unit('`，再读参数）：extent 树四处 key 宽 24（`冻结:core/extent_tree.rs:57-59`），条目宽 112 / 110 / 113 / 110（`extent_tree.rs:642`、`:675`、`:712`、`:747`）；树表 key 8 对条目 200（`make_filesystem.rs:650`、`transaction.rs:15744`）；inode 树 key 8 对 120（`transaction.rs:13804`）；容器号 key 8 对 120（`transaction.rs:14023`）；已删 inode key 8 对 8（相等，`transaction.rs:14136`）；已记操作 key 10 对 44（`transaction.rs:14177`）；分配记录树 key 10 对条目宽（`allocation_record_tree.rs:562`）；多层码 2 树 `transaction.rs:16645` 的 key 宽取各树字段宽之和：记账 2+8+4+8 = 22 对叶条目 34，映射 1+8+8+4+6 = 27 对 55，deadlist 8+8+1+8+8+4+6 = 43 对叶条目宽 `DEADLIST_ENTRY_KEY_BYTES` = 43（`冻结:core/code_two_tree.rs:37-44`、`冻结:core/records.rs:1171`、`transaction.rs:2648-2658`），相等，内部条目宽 = key 宽 + 86（`code_two_tree.rs:122-124`）。全部满足。码 3 记录宽：实例表行 88、inode 记录 140、`record_width_in_bytes` 40 / 24 / 34（`transaction.rs:2018`、`:13737`、`:13932`），非 0。
- `region_scan.rs:989,1011`（合成扫描输入）传 key 24 / 条目宽 112、记录宽非 0 的字面常量。
- 测试里的调用方（`checker_known_bad_images` 一类）刻意造「条目比 key 宽还窄」的坏节点，是否因此红？函数体没加判，所以不会因这次改动新 panic；也没有读到它们依赖的 `assert_eq!` 被换成别的行为。

什么现象会推翻：一条合法写路径传了比 key 窄的条目宽、或记录宽 0。上面全部调用点是常量或常量之和，没有。

## T7 容器小树的执行代码改写

### T7.1 改写前后行为同不同

判定：成立，附一处实现员已报告的行为差异（更严，不可达）。
- 机械比对：`python3 -I panic_sites.py container_tree deleted_inodes inode_tree lineage_tree retirement_tree tombstone_tree`：非测试代码里能 panic 的写法只有 `assert_eq!` → `assert!`、`.expect(..)` 搬了位置（`container_tree` 的 `positions_in_bump_order`、`ancestors_of_containers`、`append_records_to_the_open_container`：`last_mut().expect()` 换成 `pop().expect()` 再 push 回去，`冻结:core/container_tree.rs:3096`）、`deleted_inodes` 一处 `.expect("条目宽在读者那里核过等于 8")` 去掉（换成 vstd 的 `u64_from_le_bytes`）、`inode_tree` 里 `INODE_LEAF_RECORDS.div_ceil(2)` 换成 `(INODE_LEAF_RECORDS + 1) / 2`（`冻结:core/inode_tree.rs:264`；233 时都是 117，实现员报告写了；我算 (233+1)/2 = 117、⌈233/2⌉ = 117，相同）。没有新增 `unreachable!` / `panic!`（六份文件改前改后都是 0）。
- 唯一行为差异：`冻结:core/container_tree.rs:1624-1651` `from_children_of_each_node_at_each_level` 多拒一种输入——层级 1 的孩子数之和越过 `usize`（`checked_children_count_sum`，`:1454-1487`）交 `None`；改前收下、之后求和时 debug panic / release 回绕。文档自己写「盘上一个节点的条目数是两字节，读不出这样的数」（`:1622`），即读盘来的形状里每个节点条目数 ≤ 65535，要和越过 2^64 需要 2^48 以上个节点，物理上放不下，不可达。它还附带收掉实现员报告里的「255 层容器树再长一层」：255 层且最右路径每层都满，层级 1 的容器数至少 `entries_per_node^254`，远大于 `usize::MAX`，构造时就被 `containers_fit_usize`（`:1638`）拒掉，所以 `root_level` 的 `.expect`（`:1664-1666`）与 `ancestors_of_containers` 里那个 `u8::try_from(level_index + 1).expect(..)` 在合法或坏盘形状上都走不到。
- 实现员报告里列的其余改写（`sort_by_key` 去掉、改成逐片插入位置；`binary_search` 换手写二分；`BTreeMap` 换 `while` 循环）：我读了 `inode_tree.rs` 里 `search_records_by_inode`（`冻结:core/inode_tree.rs:660`）只确认它是下标二分，没有对它做改前改后的穷举对拍，退役清单排序「同序号排在已有的之后 = 稳定排序」这一条我没验。

什么现象会推翻：同一批输入（叶容器序列、要写的记录）改前改后交出的叶容器序列不同。库内有对应用例（实现员报 363 条全绿），我没复跑。

### T7.2 「走得到的分支在任何落盘之前交回类型化错误」：有没有改成 assert! / unreachable! / 静默截断

判定：成立（没有发现新增）；有两处原来就有、实现员已提出的 panic 面，不是这次引入。
- 没有新增 `unreachable!`、`panic!`（上面机械比对）；`assert!` 全是 `assert_eq!` 的同条件替身，没有新增条件。`冻结:core/container_tree.rs:2311`（`carried`）、`:3035`、`:3039`、`:3174`（`append_records_to_the_open_container`）都是 `shape.containers() == containers.len()` 与 `records_per_container >= 1` 这两种，改前也是 `assert_eq!` / 同条件，且各有 `requires` 对应（`:2304`、`:3018`）。
- 静默截断：我没有找到把错误输入悄悄截成合法值的新写法；`from_children…` 的新拒是交回 `None`，不是截断。
- 原来就有的 panic 面（不是这次引入）：`root_level` / `ancestors_of_containers` 的 `u8::try_from(层数).expect(..)`（上面 T7.1 已说明走不到）。
- 新 `None` 的读盘调用方：`grep -n from_children_of_each_node_at_each_level` 在 container_tree.rs 之外只有一处，`冻结:core/recovery.rs:5673-5679`，接 `.ok_or_else(malformed)?`，即转成类型化的读盘错误，没有 `expect`。T7.2 的这一问成立。

## T9 journal 读者预分配

判定：成立（两样都成立）；证明一侧只读源码和 vstd 规格，没有复验。
- `named_count` 在调用到分配那一行之前已被约束：`冻结:core/journal.rs:878` 读出 `named_count`（盘上 4 字节）后，要先过 `:948` `if named_count > (record_bytes - header_bytes) / named_entry_bytes { return None }`（上界 (4096 − 496) / 56 = 64），再过 `:964` 载荷 CRC 比对，才到 `:971` `Vec::with_capacity(named_count)`。所以坏盘给一个大项数时（比如 2^32 − 1）在 `:948` 就 `None`，不会按它分配；最大一次分配是 64 个 `NamedUnit`（结构体含两个 `LocationEntry`、几个整数和 10 字节数组，几十到一百字节量级，不是我量的）。改前的 `Vec::new()` 加逐项 push 同样先过这两关。
- 循环不变式与 `with_capacity` 的 vstd 规格：`~/.cargo/registry` 里 vstd 0.0.0-2026-09-20-0158 的 `std_specs/vec.rs:115-118` 给 `Vec::<T>::with_capacity` 的规格是 `ensures v@ == Seq::<T>::empty()`（我 `sed` 读到的原文），与 `Vec::new()` 的规格（`:110-112` `new_in` 同样 `v@ == Seq::empty()`）相同，所以循环不变式 `named@.len() == named_index`（`冻结:core/journal.rs:975-989`）进循环前成立的论证不变，同一份结论证得出。我没起 Verus（不许）；实现员报告写 verus-proof 格 711 verified、0 errors、没增减，与主 agent 在背景材料里记的合入后数相同。
- 两点小话：一，实现员单测 `a_parsed_record_holds_its_named_units_in_a_vector_of_exactly_the_named_count`（`冻结:core/journal.rs:1708`）断言 `capacity() == 1`；Rust 文档对 `with_capacity` 只承诺「至少」这个容量，当前分配器下是恰好，所以这条钉的是实现细节，换分配器策略可能无故红。二，实现员报告写 `lemma_parse_round_trip#1` 在空真检查里红（函数体开头的 `hide` 挡住了判法插的 `assert(false)`），与本次改动无关；空真由门禁判，这一格不重复，只在这里记一句。

## T8 只动规格与证明的五份：规格说的是不是条款

范围说明：这一格我读了 `gc_ring_interval_proofs.rs` 的模块文档（`冻结:singlefs-replay-prefix/src/gc_ring_interval_proofs.rs:1-30`）、G7 定理（:2200-2262）、`compaction_relation`（:1446-1500）、`compaction_publish_is_accepted`（:1303-1306）、`gc_crash_proofs.rs` 的门槛函数与引理（:70-140）、`verified_model.rs` 的文件头与槽区间部分（:1-60）、codec-tighten 的两处 `hide`（`冻结:core/journal.rs:545`、`:1370-1372`）；`gc_ring_interval_proofs.rs` 其余约 2000 行、`compaction_plan_proofs.rs` 的 P4 一段（只读了计划核的签名与不变式，那是「一字没动」的可执行核）、`address.rs` 的 newtype 我没有逐条读。判决 = `research/prompts/m4-exit5-relocation-proof-r3-main-verification.md`，Z1 三行在其第二节表里（本文件第 19、20、21 行），第三节「抽象状态」「G7」在第 31、32 行。

### T8.1 有没有一条 ensures 比条款弱

判定：G7 成立；门槛引理成立但带两条没证的前提（单列）。
- G7：判决 Z1 第二行（第 20 行）要的是「整理不改现行内容；回退候选集是不整理那份的后缀、至多少 k 个，抬 F 的上限恰多 k，按版本号点名」。`theorem_compaction_keeps_what_the_user_sees`（`冻结:replay-prefix:gc_ring_interval_proofs.rs:2200`）的 `ensures` 是 `compaction_keeps_what_the_user_sees(...)`，展开成 `compaction_relation`（:1446）：现行内容相同（:1458）、最后一个根内容相同（:1460，这是用户 2026-10-26 加的「连崩溃」）、窗口与下一个用户版本号相同、`raise_floor_ceiling(with) == raise_floor_ceiling(without) + compaction_publishes`（:1465，恰多 k）、不整理那份的根按原次序同内容同版本号出现在整理那份里且整理那份里带版本号的根都是这样来的（:1467-1476，后缀）、内存 F 之上与持久 F 之上两层都用 `floors_correspond`（:1478-1492，至多少 k 个）。逐句对上，没有比判决弱的一句。
- 用户 2026-10-26 定「整理的发布只带持久 F」：`observation_after_a_compaction_publish`（:1291-1300）用 `observed_in_memory_floor_after_a_compaction_publish` 与 `observed_persisted_floor_after_a_compaction_publish` 两个函数分开算；core 一侧 `冻结:singlefs-core/src/transaction.rs:10697` 整理的根带 `rollback_floor: previous.root.rollback_floor`（上一版的根带的 F，也就是持久 F），与协议文档（:11-14）自己指的是同一处。
- 门槛引理：判决 Z1 第三行（第 21 行）要「门槛蕴涵没被引用」写成引理。`lemma_the_reclaim_threshold_leaves_the_released_run_unreferenced`（`冻结:replay-prefix:gc_crash_proofs.rs:79`）证出了，没有 `assume`。但它的两条前提 `the_release_generation_hides_the_run`（释放代的含义）与 `the_floor_is_at_most_every_protected_version`（floor 不大于现行、持久 F 之上每个根、每个读者的版本）没有调用点：全仓没有一个地方把它们喂给这条引理（它是接口引理）。模块文档自己写「这是分配器与 floor 的来历要另证的义务，不在这里证」（`gc_crash_proofs.rs:9-15`）。所以条款的原句「门槛蕴涵没被引用」被证的是一个有条件的蕴涵：条件在 core 里是否成立没人证。对判决第 20 行的字面（把它写成引理），成立；对它的意图（core 回收不碰被引用的槽），还差这两条前提，条款没说前提要不要在这一轮证。
- 「读者」那一维在 core 里没有对应物：`grep -n reader allocator.rs` 只命中 `ByteReader`（字节读取器）；分配器不记读者、floor 的算法（`mount.rs` 抬 F 序列）也不看读者的版本。协议里的读者是为并发读建的；第一版串行（会话持 `&mut`），这一维在 core 里恒为空，引理的读者前提空真地成立。这一点不是缺陷，但「floor 不大于每个读者的版本」这条前提的证明义务在 core 一侧是空的，里程碑六引入并发读时会变成实的。

什么现象会推翻 G7 成立：`compaction_relation` 里有一条说成了存在但取不到的值（如 `position` 被取成空序列让后缀条件空真）。`position.len() == without_compaction.roots.len()`（:1467）把它钉成与根数等长，所以空真取不到。

### T8.2 有没有一条 requires 比实现的调用点强

判定：有一条，在 G7 上；core 一侧靠 T4 的屏障恰好满足，但协议文档没有把这个依赖写出来。
- 前提：`event_is_accepted_by_both` 对整理的发布要 `compaction_publish_is_accepted`（`冻结:replay-prefix:gc_ring_interval_proofs.rs:1303-1306`）：根环至少有一个根，且现行内容等于最后一个根的内容（「整理不把用户还没落成根的改动先落盘」）。这是实现员报告里说的「条款没写的前提」（事实表 m4-exit5-compaction-floor-writer-2 一节，result.md 第 315 行）：判决 Z1 第二行没有这个前提。
- core 里满足它的东西：整理只在 `MountedSessionWithPendingChanges::run_compaction_in_the_background` 里跑，判过起跑后先排空待发（`冻结:singlefs-core/src/mounted_session.rs:5499-5503`，见 T4.3），排空发成一次用户发布，之后现行内容就是最后一个根的内容；该攒批的会话把 `MountedSession` 设成私有字段、对外只给 `&MountedSession`（`ms:5069-5071`），调不到不排空的 `MountedSession::run_compaction_in_the_background`。所以在今天的 core 里这条 `requires` 成立，但依赖 T4.3 的屏障，而不是协议或 core 的某条证明。如果日后有一条整理入口绕开攒批会话（比如挂载选项通道、里程碑十），G7 的前提在那一条上不成立，证明照样绿。
- 门槛引理的两条前提：见 T8.1，同类问题（前提无调用点）。

### T8.3 int 与 u64 之间的桥（判决 Y5）缺着，哪一条结论在实现上不成立

判定：条款没说结论会不成立；我推不出一条因此在 core 上实际不成立的结论，但有一条「接上之前不能引」的结论。
- 协议里槽区间是 `int`（`SlotRun { start: int, span: int }`，`冻结:replay-prefix:gc_ring_interval_proofs.rs:30-33`），`runs_overlap` 在数学整数上算（:39-41）。core 一侧 `verified_model.rs` 已经把桥的义务写成了谓词 `slot_run_fits_in_u64`（`冻结:core/verified_model.rs:44`：`span_slots >= 1 && start + span <= u64::MAX`），文档写「接协议核时要另证」（:37-40），但没有任何一条引理把它接到 `gc_ring_interval_proofs`。
- 缺着的后果：协议证出的「写与回收的门放行的槽没被任何受保护视图的区间罩住」是 `int` 上的结论；core 的分配器按 `SlotNumber`（u64）、跨度（u16 一类）做 `slot + span` 的比较。在 core 的真实取值范围里（槽号是 6 字节整数，`冻结:core/records.rs` 里分配记录用 `get_six_byte_unsigned`；跨度 2 字节）起点加跨度不可能越过 u64，所以 `int` 与 `u64` 的结果在合法取值上相同；但「合法取值」的这一层限制证明里没有，坏盘给出的槽号与跨度（读者在读分配记录时只判 6 字节、2 字节的宽度）会不会越过，是桥该证的东西。能引的结论：G7 与门槛引理在协议层成立；不能引的：「core 的回收在 u64 上不碰被引用的槽」，要等桥。
- 门槛函数本身没有这个问题：`released_record_can_be_reclaimed`（`冻结:replay-prefix:gc_crash_proofs.rs:69` 的 `ensures answer == (released && generation <= floor)`）的 generation 与 floor 在规格里就是 `u64`，没有 `int`。

### T8.4 其余几份

- `verified_model.rs`：文件头写明只放 spec、不接协议核、不被产品调用（:1-10）；格式常量用 `assume_specification` 交回值（`SLOT_BYTES` 16384、`MAPPING_KEY_BYTES` 27，对 `冻结:singlefs-format/src/lib.rs:14`、`:92` 同值，我 `grep -n` 核过），跨文件一致性由库内测试 `every_format_constant_specification_returns_the_value_singlefs_format_defines` 判（`verified_model.rs:244` 起的测试模块）；core 里共 42 条这样的规格（`grep -c 'pub assume_specification\[ singlefs_format'` 我数的）。它们是信任清单里的可信声明，值错了 Verus 照样绿，所以那条测试是唯一的钉子。
- codec-tighten 用 `hide` 换的两行必红世界：落点在 `冻结:core/journal.rs:545`、`:1370-1372`；「空真与必红世界由门禁判」，这一格不重复。我只确认 `hide` 只出现在 proof fn 的开头（Verus 编时有、普通构建擦掉），不影响执行代码。
- `compaction_plan_proofs.rs`：P4 一段换成区间版，选段核、计划核两个可执行核补丁没动（实现员报告 result.md 第 281 行的说法，我读了 `plan_compaction_batch` 的签名与循环不变式，没对改前改后逐行比）；我没有核 P4 区间版 `requires` 对判决 Z1 第一行（第 19 行）「变体二：目的地压住回退目标两槽单元的第二槽」（第 19 行）的覆盖，这一点留给门禁的必红世界（实现员报该变体有必红世界行）。
- `gc_ring_interval_proofs.rs`、`address.rs` newtype：见范围说明，没有逐条读。

## T2 补记：冻结副本里 `write_request_split.rs` 的部分写计划不全是这一轮 14 份补丁改的

我在 T10 里查变异表锚点时发现：冻结副本 `crates/singlefs-core/src/write_request_split.rs`（sha256 `a849ae35…`，与开工快照相同）里 `PartialWriteOfAFile` 的计划已经改成调 `singlefs_replay_prefix::partial_write_proofs` 与 `file_range_proofs` 的 Verus 核（`冻结:core/write_request_split.rs:284-366`），而本轮 diff 里同一个类型是内联算术（`research/prompts/_m4-close-code-r1-diff.md:3773-3787`，`first_rewritten_unit` 用 `checked_sub` 与 `match`）。`git diff HEAD --stat` 看，这份文件相对 HEAD 还有 176 行改动（`+135 −41`），`partial_write_proofs.rs`、`file_range_proofs.rs` 两份在 git 里是新增（`A`）、不在本轮 14 份补丁里。结论：T2.3 读的计划不在这一轮的 diff 里；两版我按公式对过——重写起点、写后单元数、来源分类三式两边相同（diff 版 `first_rewritten_unit` 的「首个被写单元 < 写前单元数则取它，否则旧尾单元不满则取旧尾单元，否则取写前单元数」等于 `first_unit_to_rewrite` 的三支），所以 T2.3 的判定对两版都成立；但被这次改写弄丢的变异行见 T10。

## T10 测试与变异钉得住这一段的执行代码改动吗

方法与命令（只读，不起 cargo）：`research/prompts/m4-close-code-r1-sonnet-model/mutation_checks.py` 读 `冻结:crates/mutations.tsv`（6311 行），（一）每行「原文」在冻结副本里是否恰好命中一次，（二）各主题变异行点名的测试 fn 在仓里找不找得到，（三）T6/T7 各文件里每个非测试 exec fn 有没有变异行的原文落在它的函数体里。我对每条结论另写了定点读法，在各格下面。「改回去会不会红」我没有现跑（要 cargo 且是变异整表的活），下面说的「会红」是按行里写的「必须红的测试名」存在、原文替换文确实改了那一处推的，现跑留给门禁 checker-tier-crates-mutation-replay。

总体数：变异表 6311 行里原文不恰好命中一次的有 14 行（锚点失效；门禁表头写「不是的那一行记锚点失效、不跑，别的行照判，整道判红」）；其中点名被判 30 份文件的 8 行：`write_request_split.rs` 6 行（表第 5894、5896、5901、5902、5903、5904 行）与 `truncate.rs` 2 行（第 4387、4404 行），都是 HEAD 上命中 1 次、冻结副本上命中 0 次（我用 `git show HEAD:<文件>` 对过）。另 6 行在 `unlink.rs`（4 行）、litmus（1 行）、wgsl（1 行），不在本轮射程。

### T10.1 T1（多 inode 改 size）

判定：大体钉住，三处缺口。
- 30 行（表第 5814–5830、5832–5835、5876–5884 行；5831 是 C145 那一行，单列在下面）：单元数按号取（5815）、续做按号认领（5816）、两种批的 inode（5817）、下段计划的叶边界（5818）、尾单元头里的对象与旧净荷（5819、5820）、标签 3/4（5821）、粘住的标签 0（5822）、快照下 deadlist（5823）、冷恢复三处（5824、5825、5826）、池级 checker（5827）、胶水（5828）、模型（5829、5876–5884）、GPU 恢复/检查内核（5832–5835）。点名的测试 fn 全在仓里（我的检查输出「点名测试在仓里找不到 fn 的 0 行」）。
- 缺口一：unlink 批截带洞的别的文件、尾部换回标签 1/2（T1.2 ①，`冻结:core/transaction.rs:7608-7615`）。表里没有原文落在这一臂的行（`NoHoleAfterTheLastDataUnit` 只在第 5821 行出现，那是改 size 的一臂，`transaction.rs:13458`）。改成保持标签 3/4 的错法：读者对标签 3/4 要求单元数严格少于 size 算出的数（`冻结:core/write_request_split.rs:115-117`），所以会被冷恢复或读者拒，多半有测试会红，但没有一行变异点名它。补哪一个取样点：一个带洞的别的文件（先变长造洞再 unlink 批截到恰好 size = 单元数 × 净荷），断言最后一批之后上段叶标签是 1/2、冷恢复收下。
- 缺口二：N = S = 0 不写条目（T1.2 ②，`transaction.rs:7672-7674`）。第 5822 行只覆盖「该写而不写」；反向「不该写而写」（把条件改成恒真）无行。取样点：`publish_new_inodes` 建出的 size 0 inode 在上段叶里没有条目。
- 缺口三：冷恢复对 nlink = 0 的记录是否豁免（T1.2 ③，`冻结:core/recovery.rs:3887-3897`）。第 5826 行把 `record.size != 0` 换成 `== u64::MAX`，覆盖「不判 size」；「加一句 nlink != 0 的豁免」无行。在合法历史里这两种改法不可区分（nlink = 0 且 size > 0 且无条目不会出现，T1.2 ③），所以只能用手造的镜像钉：一份 nlink = 0、size > 0、上段没有条目的记录，断言冷恢复拒。
- C145 第三条的 `assert!(released_entry.is_some())`（T1.3）在合法历史里不触发，没有变异能让它红（改成不断言行为不变），不需要行；表第 5831 行钉的是它上面的 `remove`（改成 `get` 则不删条目）。

### T10.2 T2（部分写 core 一侧）

判定：读盘与故障格钉得很全；计划算术那一块现在没有一行钉着。
- 39 行（表第 5885–5905、5913–5929、6221 行）：读盘（5885、5929）、CRC（5886）、一份坏读第二份（5887、5923）、都坏拒（5888、5890）、映射落点（5924、5925）、短读（5926）、声明长度（5927）、补齐非零（5928）、残留读与记数（5891–5893）、空隙不读（5894）、跨末尾变长（5895）、洞里（5896）、R4 退路（5897）、别的 inode（5898）、K4 deadlist（5899）、会话入口（5900）、计划四行（5901–5904）、下段计划（5905）、模型与 RefFS（5913–5922）、崩溃流（6221）。
- 缺口一（上面总体数里的 6 行）：5894、5896、5901–5904 的原文都落在 `write_request_split.rs` 的内联算术上，现在冻结副本里这些算术已搬进 `singlefs-replay-prefix` 的核（T2 补记），所以 6 行锚点失效；对应的计划那一块（`classify_partial_write_unit`、`first_unit_to_rewrite`、`units_after_partial_write`、`bytes_in_unit`）本身是 Verus 证过的，表里没有行点名 `partial_write_proofs.rs` / `file_range_proofs.rs`（`grep` 全表无）。留在 `write_request_split.rs` 里的胶水有四处没有行点名：`has_old` 实参取 `unit.0 < self.data_units_before`（`:356`）、`tail_is_partial` 的算式（`:313-317`）、`end_exclusive` 的来路（`:256-259`）、`declared_length_after_the_write` 取写后 size（`:334`）。取样点：把 `has_old` 的比较改成 `<=`、把 `tail_is_partial` 恒取 false、把 `data_units_after` 的 `end_exclusive` 换成 `offset`，各配一个 T2.3 里模型的边界例（偏移 = 单元边界 ± 1，旧尾单元满与不满）。我的 `partial_write_plan_model.py` 可当这些行的参照用例。
- 缺口二：truncate.rs 的 K2、K12 两行失效（4387、4404），是别的会话改 `truncate.rs` 时挪走的，不在这一轮的补丁里（`truncate.rs` 本轮的补丁只在 `DataUnitsRewrittenInOnePublish` 与下段计划上），列在这里是因为 T2 的尾单元改写走这个文件。
- 缺口三：残留读取盘的选择（T2.4 小点，`冻结:core/transaction.rs:14397` `reader.first()`）无行。只有两块盘的池里才分得出，取样点：两盘池、第一块盘残留读坏而第二块好，断言记数。
- 缺口四：读回旧单元没有重读（T2.2），也没有行；条款也没要，只是写在这里。

### T10.3 T4（发布触发）

判定：触发式、屏障、预判钉得多，冻结后的处理与「第一版不收」三拒无行。
- 21 行（表第 5943–5963 行，另 2380、2463 两行钉 T_time 默认值）：`≥` 与 `>`（5943、5944）、两支都没到（5945）、待发非空合取（5946）、读数倒退（5947）、只计数据字节/折叠（5948、5950）、夹取（5949、5951）、冻结后 `accept` 拒（5952）、屏障先排空与单发（5953、5954）、整理排空（5955、5956）、进门判并成的批（5957）、原入口排空（5958）、RefFS 翻译（5959）、判触发在收下之后（5960）、折叠（5961）、两条崩溃流（5962、5963）。点名的测试 fn 全找得到。
- 缺口一：`publish_the_pending_changes` 在发布失败时的处理（`冻结:core/mounted_session.rs:5146-5150`，`frozen_publish().is_none()` 才放回待发）。表里没有原文落在 `frozen_publish().is_none()` 或 `self.pending = pending` 上的行。错法两种：无论冻结与否都放回（冻结的那次重发后，待发再发一次，写重复、建 inode 重复）、无论如何都不放回（没冻结也丢改动）。取样点：写失败注入两次——一次发生在读与分配之前（没冻结，待发应还在）、一次发生在写到一半（冻结，待发应空）；再加我在 T4.4 写的那个角：空间被拒、抬 F 的空发布写失败而冻结，批被丢（这一形现在行为是丢，是否改要主 agent 定，行要钉的是定下的那个行为）。
- 缺口二：`WriteToASecondInodeInOneBatchNotSupportedInTheFirstVersion`（`ms:5258-5267`）、`QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`（`ms:5254-5256`）、`WriteAtOffsetIntoThePendingChangesNotSupportedInTheFirstVersion`（`ms:5199-5203`）三个「第一版不收」的拒，全表没有行（我 grep 了成员名）。删掉任一拒，`accept` 会在返回成功之后才在发布时出错，违反「过不了就在任何读写之前拒」。取样点：各配一个造出那一态的用例（两个号交替写；挂载后带着未走完的截断意图交数据写；按偏移写交 `accept`）。
- 缺口三：阈值测试来源的编译期封闭（P6）是文档测试 `compile_fail`，不是变异行；把 `#[cfg(feature …)]` 去掉的错法由文档测试红（`冻结:core/system_configuration.rs:438-439`），要跑 `cargo test -p singlefs-core --doc`，不在 `--lib` 里，门禁的变异整表（`--lib` 或点名目标）看不到；我没有找到哪一格门禁在默认特性下跑这条文档测试，实现员报告也写「没跑」。这是一个需要主 agent 查的点：编译期封闭靠谁钉。

### T10.4 T6（编解码）与 T7（容器小树）

判定：钉得住的只是原来就有行的那一批；这一轮改写的执行代码，多数 fn 没有变异行点名，靠 Verus 证明兜底，这一点是设计（实现员报告以必红世界 160 + 49 行代替），但「改成另一种错法」落在规格同样错的地方时只有测试能红。
- 我的检查（三）输出每文件「有变异行落在函数体里的 exec fn 数 / 非测试 exec fn 总数」：`bytes.rs` 1/26、`checksum.rs` 3/18、`pointer.rs` 5/22、`root_record.rs` 3/6、`journal.rs` 5/13、`unit.rs` 8/21、`records.rs` 22/41、`logged_operations.rs` 7/24、`container_tree.rs` 3/40、`inode_tree.rs` 10/29、`deleted_inodes.rs` 1/3、`lineage_tree.rs` 0/7、`retirement_tree.rs` 2/8、`tombstone_tree.rs` 1/7（完整输出在复跑的 `mutation_checks.out`）。这个数偏低估：变异行也可以锚在调用者里，函数被别的行间接钉住。
- 点名没有行的、又是 T6/T7 改写重点的 fn：`bytes.rs` 全部 ByteReader/ByteWriter 的 `put_*`/`get_*`（逐字节写、成块拷贝）、`unit.rs` 的 `pieces_of_one_width`（`chunks().take().map()` 改下标循环，T6 的重点）、`seal_payload_checksum`、`write_common_prefix`、`build_packed_unit`、`records.rs` 的 `data_key_tail`（`% 2^48`）、`journal.rs` 的 `back_chain_of`（496 字节拼法）、`container_tree.rs` 的 `from_children_of_each_node_at_each_level`/`checked_children_count_sum`（唯一行为差异）/`root_level`/`ancestors_of_containers`/`positions_in_bump_order`、`inode_tree.rs` 的 `search_records_by_inode`（手写二分）、`note_rewritten`/`index_is_noted`/`retire`（`sort_by_key` 改逐片插入，实现员写「同序号的排在已有的之后」）、`records_below_which_a_leaf_borrows_from_its_right_neighbour`（`div_ceil(2)` 改 `(N+1)/2`）。
- 每一处的取样点（改回去或换成另一种错法要有一行会红）：
  - `data_key_tail`：事务号取 `% 2^48` 与取低 48 位以外的位；取样 `transaction = 2^48` 与 `2^48 + 1`，断言尾段字节为 0 与 1。
  - `pieces_of_one_width`：`n × w == d` 的边界、`w == 0 ⇒ n == 0`；取样 w = 1、n = 0；n = 64、w = 56 的 journal 恰好满记录。
  - `from_children…` 的溢出拒：取样一层孩子数之和 = `usize::MAX`（需要 `Vec` 里放两个大计数，构造得出，因为只是 `usize` 数，不是真节点）。
  - `search_records_by_inode`：二分的「不在、插入点」二值，取样首、尾、中间不在、重复号（读者应拒重复，别的在上游）。
  - `(INODE_LEAF_RECORDS + 1) / 2`：取样叶容量 233 恰好 117 的借位门槛（行 3358 钉了合并门槛，没有钉借位门槛）。
  - 退役清单排序：同序号两片退役的先后（稳定）。
  - ByteWriter/ByteReader：取样 u16/u32/u64/六字节的大端混淆（字节 0x01 0x02 → 0x0201），写后读回，Verus 的 `ensures` 钉小端，但如果 `ensures` 本身写成大端而函数体也写大端则证明绿；这时只有对着 `singlefs-format` 字节表的测试才红（我推的，现有 core 库测试有这样的用例，没有按行核）。
- 码 2/3 收严的前提（T6.3）：函数体没变，无变异可挂。

### T10.5 T9（预分配）

实现员已配 2 行（表第 6298、6299 行）：第 6299 行（`Vec::with_capacity(named_count + 1)`）点名 core 库单测，实现员现证红；第 6298 行（改回 `Vec::new()`）点名 checker 档 E197 的测试，属重型，没证红（实现员报告写明留给主 agent）。这一格不算在 T10 要问的 T1、T2、T4、T6、T7 里，附在这里。

## 判定一览

判定三选一：成立 / 不成立 / 条款没说。同一格里有几个子问的，每个子问各一行；括注里写的是附加的发现。

| 格 | 判定 | 一句话 |
|---|---|---|
| T1.1 第一个文件与别的 inode 同路径同判法 | 成立 | 单元数按号取、续做按号认领、下段计划与标签核对同一份；没读完 `transaction.rs` 全文，只读调用链 |
| T1.2 ① unlink 批截带洞文件换回标签 1、2 | 成立 | 剩 k 个单元、size = k × 净荷，算术上无洞，读者对 3/4 反要求单元数更少 |
| T1.2 ② N = S = 0 不写条目 | 成立 | kb 已写粘住形；判决括注字面读法有歧义，建议判决补一句 |
| T1.2 ③ 冷恢复对 nlink = 0 同判 | 成立 | 判决字面没分 nlink，实现无例外；合法历史里这一形不出现，无反例 |
| T1.2 ④ 第一个文件与别的文件两条计划 | 成立 | 第一个文件的条目恒在，号集合不变，推得出 |
| T1.3 C145 第三条断言 | 成立 | 触发需要两条 deadlist 条目同键，冷恢复已拒 |
| T2.1 读旧净荷改读盘 | 成立 | 部分写与截断尾单元同一读函数，读失败在任何分配与写之前 |
| T2.2 两份副本一份读错 | 成立（读法推得出，条款没直说；重读条款没说） | 任一份 CRC 对上就算读到；D4 字面没分一份与两份，一次瞬时坏读不重读 |
| T2.3 部分写的切分 | 成立 | Python 暴力参照 5 种容量 56964 例全同；真 Rust 探针 2205 例（真容量 32634）通过；计划本体在本轮 diff 之外（见 T2 补记） |
| T2.4 新错误成员在落盘之前交回 | 成立 | 全在 `publish_deletion_step` 之前；外层成员名条款没说 |
| T3.1 F2 读法是否 D4 已定项 4 推得出 | 条款没说 | 判决已采 R7.1 的 F2 格；字面「读不回来」没分一份两份，用户未确认 |
| T3.2 RefFS 中间态与实现、模型 | 成立 | 内容改动型只允许写前写后两样，与实现单次发布一致；崩溃枚举属 checker 档，复核不了 |
| T4.1 触发判据只在不变输入上分支 | 成立 | 阈值挂载求值一次，`now` 与待发字节是被比较的量 |
| T4.2 只供测试的强制开关 | 成立 | 阈值测试来源在默认关的特性后；已有开关按判决另立欠账 |
| T4.3 屏障与进门预判次序 | 成立（三处「第一版不收」条款没说） | 屏障先排空再单发，预判在任何读写之前；预判拷了分配器，判决说做不到要报主 agent |
| T4.4 报错后并进冻结的那一次 | 成立（附一个窄角） | 抬 F 空发布冻结时批仍被丢，不违 D16 已定项 17 但违「已 accept 必发得出去」 |
| T4.5 两个号的写攒进一次发布 | 条款没说 | 「同一 inode 折叠」不涉两个号，今天拒并是更窄的实现 |
| T5.1 块设备改写前后同一行为 | 成立 | 判法体逐字相同；仅计数改饱和加，不可达 |
| T5.2 差分测试罩不罩得住信任清单 | 成立（附 discard 对 `Err` 的规格比能保证的强一档） | 声称的都罩住；discard 的失败用例只是造出来的 EIO |
| T5.3 证明对 SMT 的负担 | 条款没说 | 源码读不出异常，推测是 vstd 规格实例化，未复验 |
| T6.1 编解码改写同义 | 成立（抽样） | panic 面多重集比对只有 assert_eq→assert 等替身；146 个 fn 未逐个读完 |
| T6.2 往返契约在边界上取不取得到 | 条款没说 | 证明层面写者前提即引理前提；调用点全是常量，边界值没有调用点 |
| T6.3 码 2、码 3 新前提在调用点成立 | 成立 | 所有产品调用点 key 宽 ≤ 条目宽、记录宽 ≠ 0；函数体没加判 |
| T7.1 容器小树改写前后行为 | 成立（一处更严、不可达） | `from_children…` 多拒层级 1 孩子数之和越过 usize |
| T7.2 无新增 assert/unreachable/静默截断 | 成立 | 没有新增 `unreachable!`/`panic!`；`from_children…` 的 `None` 在读盘处转类型化错误 |
| T8.1 ensures 不比条款弱 | 成立（门槛引理带两条无调用点的前提） | G7 展开逐句对上判决；门槛引理前提在 core 一侧没人证 |
| T8.2 requires 不比调用点强 | 不成立 | G7 要「整理发布只在现行内容与最后一个根相同时发」，判决没写；core 靠 T4 屏障满足 |
| T8.3 int 与 u64 的桥 | 条款没说 | 桥的谓词写了没接；合法取值下起点加跨度不越界，但证明里没有 |
| T9 journal 预分配 | 成立 | `named_count` 先过 ≤ 64 与 CRC 再分配；vstd 规格对 `with_capacity` 同 `new`，未复验 Verus |
| T10 变异钉得住吗 | 不成立 | 14 行锚点失效（本轮射程内 8 行）；T1 三处、T2 计划胶水、T4 冻结处理与三拒、T6/T7 多数改写 fn 无行 |

## 没做什么

- 不判别的腿的格（攻方的反例构造），不替主 agent 采纳或出判决。
- 没起 Verus、GPU、门禁阶段、checker 档测试。cargo 只起过一条：冻结副本拷贝里 `singlefs-core` 的一个自写探针测试目标（T2.3），编一次（接手前上一条腿排队的那条，加我改探针之后重编一次）、跑一次，每条都经内存包装；所以「改前改后穷举对拍」「变异现跑证红」「库内测试复跑」这几类都没做，写在各格的「什么现象会推翻」里。
- 没逐行读完：`transaction.rs`（17244 行）全文、`unit.rs` / `records.rs` / `bytes.rs` / `container_tree.rs` / `inode_tree.rs` 的全部改写、`gc_ring_interval_proofs.rs` 其余约 2000 行、`compaction_plan_proofs.rs` P4、`address.rs`。
- 复核不了：崩溃枚举（checker 档）、文档测试 `compile_fail`、实现员报告里的测试条数与 Verus 条数（都要 cargo 或 Verus）。

## 删了什么（草稿目录 `~/.local/share/singlefs/work/m4-close-code-r1-forward/`）

交回前删掉的，括号里是删之前 `du -sh` 量的大小：上一条腿留下的冻结副本拷贝 `copy/`（5.9M，我在里面只改了一个探针文件 `tests/probe_partial_write_split.rs`，改动已拷进模型目录）；编译目录 `target/`（541M，上一条腿排队的那次编译加我重编一次的产物）；`git show` 取的改前文件 15 份 `*-head.rs`（合计约 0.5M，其中 `bd-head.rs` 104K、`inode_tree-head.rs` 80K、`records-head.rs` 72K）；`bd.diff`（84K）；`build1.txt`（40K，大半是排队提示）、`build2.txt`（4K）；两个复跑输出目录 `rerun-test1/`、`rerun-final/`（各 32K）。留下的是交接摘要、cite-check 输出、探针两次运行的原始输出和我早先的草稿脚本，共 92K。模型目录 `research/prompts/m4-close-code-r1-sonnet-model/` 在仓里，不删。

## 要主 agent 处理的

- 一条权限问题：我想停上一条腿遗留的、还在排队的 cargo（pid 3360610，它的编译目录在上面那个草稿目录），权限分类器拒了；它之后自己编完退出了，现在没有进程留着（`ps` 核过）。无需再处理，写在这里是因为那一次「停进程被拒」的原文是「Permission for this action was denied by the Claude Code auto mode classifier. Reason: [Interfere With Workloads]」。
- 判定里要主 agent 定的事：T1.2 ② 判决括注的歧义（要不要补一句）；T3.1 F2 读法要不要问用户确认；T4.4 抬 F 空发布冻结时批被丢要不要改；T4.5 两个号的写要不要并；T4.2 默认特性下 `compile_fail` 文档测试由谁钉；T5.2 discard 对 `Err` 的规格要不要放宽成两可；T8.2 G7 的「整理发布只在现行内容与最后一个根相同时发」要不要写进判决/条款；T10 里 8 条失效锚点（`write_request_split.rs` 6 条、`truncate.rs` 2 条）要有人换锚点。
