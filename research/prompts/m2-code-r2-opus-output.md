# 发布 B 代码三方对抗第二轮：攻方腿报告（Opus，2026-09-16）

攻击面：**只攻第一轮判决之后那四处改法本身**（Y1 释放经映射、Y2 空闲独立维护、Y3 装不下报错、Y4 oracle 带实例），
加顺带一项（`allocation_records_are_one_per_device`）。不重复第一轮攻过的角度（原代码错在哪）。

**副本**：`/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/m2-code-r2-opus-copy/`
（`rsync -a --exclude target --exclude .git`）。下面凡标「副本读数」的都是在那份副本上跑出来的，**不入库**。
副本基线：`cargo test --workspace` 全绿（`cargo clippy --workspace --all-targets -- -D warnings` 也全绿）。

## 结论一览

| 处 | 打中 / 没打中 | 一句话 |
|---|---|---|
| **Y1 释放经映射** | **打中，两处** | ① 映射条目在、但落点指向别处时，释放路径 **panic**（`.expect("释放的落点要有分配记录")` / `assert_eq!("释放的跨度与分配记录不符")`），而 Y3 的改法刚刚宣布「不许走到断言」；② 跨度不经映射、按单元种类现推，映射只供槽号——两条「互相对」的路共用同一个跨度来源 |
| **Y2 空闲独立维护** | **打中** | 把 `free_slots()` 整个改回 `self.unit_area_slots - self.allocated_slots`（D5 已定项 4 ⚠️ 明令不许的那一式），**全仓零测试判红、零警告、clippy 全绿**。这处改法没有任何会红的东西守着 |
| **Y3 装不下报错** | **没打中（算式与时点都核过）**，但有两处写死要记账 | 头 135 / 容量 812 逐项核对无误、报错确在动分配器之前；`NoSpaceFor` 那条路**不是**——它在释放之后、分配到一半才返回，留下一个半新的池，而且没有任何用例 |
| **Y4 oracle 带实例** | **打中** | 「落在一个从没发布过的更新的根上、报『没有文件』」被 oracle 判为无违例（构造出来了，副本上跑绿）；另外 `JournalPolicy::Ignore` 那一遍恢复从头到尾不过 oracle，而发布 B 之后它与 `Consult` 读出的是**两个不同版本** |
| **顺带 逐盘核** | **打中（放过一个本该拒的）** | 判据是**集合**相等，而走读自己不判 key 严格递增 ⇒ 同一盘同一个槽两条记录（代不同）照样过 |

## Y1 释放经映射：改法把一个 panic 换成了另一个 panic

### 打中 ①：映射条目在、落点指错 ⇒ panic，不是 `ReleaseNotInMapping`

`crates/singlefs-core/src/transaction.rs:556-581` 的 `placements_to_release_via_mapping` 只在**查不到 key** 时报错：

```
mapping_locations_for_key(mapping_node_bytes, key)
    .ok_or(PublishError::ReleaseNotInMapping { unit: identity })?
```

查得到就把 `locations[0].slot` 原样交给 `PoolAllocator::release`。而 `crates/singlefs-core/src/allocator.rs:312-329` 的 `release` 对这个槽号只有断言，没有错误路径：

```
.expect("释放的落点要有分配记录");
assert!(!record.is_released, "同一个落点释放了两次");
assert_eq!(u64::from(record.span_slots), placement.span, "释放的跨度与分配记录不符");
```

⇒ **映射节点里那把 key 还在、落点被改过**，两种结局都是 panic，而不是一次干净的拒绝。

副本上两个探针（加在 `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs` 末尾，
把 A 的映射节点里码 1 那一条的 `locations[0..2].slot` 改写之后再覆盖写）：

```
test probe_mapping_entry_pointing_at_an_unallocated_slot_panics - should panic ... ok   # 指向 60000（无分配记录）
test probe_mapping_entry_pointing_at_a_one_slot_unit_panics     - should panic ... ok   # 指向 A 的 extent 根（记录跨度 1）
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out
```

**该红而没红的用例**：`release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released`
只造了「把码 1 那一条整条删掉」这一种坏法（附录二那份用例正文里 `assert_eq!(kept.len(), 5, "删掉码 1 那一条，剩五条")`），
**没有任何用例造「条目在、落点变了」**。而这正是第一轮 Y3 定下的口径要拦的形态——附录二 `unit.rs` 的新断言消息逐字写着
「条目装不进一个节点：写者要先按 index_node_entry_capacity 判、报错，**不许走到这里**」。
同一轮判决里一边把 `build_index_node` 的 panic 换成 `PublishError`，一边新开了两条从映射内容直通 panic 的路。

⚠️ 射程：今天 `previous` 是同进程内存态，坏不了。但 `placements_to_release_via_mapping` 的入参是
`&TransactionOutput`、**一个 reader 都不带**（`transaction.rs:556-560`），它读的是 `previous.unit(MappingTree).bytes`
与 `previous.mapped_units` 两份内存字节，跟盘上那份映射节点没有任何一条比对。
步 3「第二个可写实例要重开」一旦要从盘上重建 `previous`，这两条 panic 就是盘上字节直接够得着的。

### 打中 ②：跨度不经映射，而验收那条「两条路互相对」在跨度这一维上是恒等式

`transaction.rs:574-579`：映射查出来的 `locations` 只用了 `locations[0].slot`，跨度来自
`identity.span_slots()`（`transaction.rs:408-416`，按单元种类查表：`UserData` 与 `TwoSlotsAligned` 给 2、`OneSlot` 给 1）。
而 `TransactionOutput::placements()`（`transaction.rs:530-540`）的跨度**也是** `unit.identity.span_slots()`。

⇒ 验收里那句

```
assert_eq!(via_mapping, first.placements(), "经映射取的八个落点与写者自己记的槽号一致（两条路各自算）");
```

在 `slot` 这一维上确实是两条路，在 `span` 这一维上**两边调的是同一个函数**——
这一半是恒等式，按 `.claude/rules/mutation-sampling.md` 的「第五类：两个口径算同一个量」，
它对「跨度取错」这件事一个字都说不出来。真正钉住跨度的是 `allocator.rs:322-326` 那条 `assert_eq!`，
而那是一条 panic，不是一条判据（见打中 ①）。

D19 已定项 5 第 1 条要的是**释放一律经映射**。今天经映射的只有槽号；跨度仍然「经提示」——
按写者自己那张种类表推出来，盘上那条分配记录的跨度只被拿来做断言比对。

### 「报 `ReleaseNotInMapping` 之后调用方拿着一个半新的池」——这一问的答案分两半

- `ReleaseNotInMapping`：**池确实一点没动**。`publish_overwrite`（`transaction.rs:748-773`）在进
  `publish_file_version` 之前先算 `placements_to_release_via_mapping(previous)?`，错了当场返回，
  分配器没被碰过；验收里 `assert_eq!(fresh_pool.allocator.records(), &records_before[..])` 钉住了这一条。✔
- `AllocationRecordsExceedOneNode`：同样干净（判定在 `transaction.rs:800-813`，`release` 循环在它之后）。✔
- **`NoSpaceFor`：不干净，而且没有任何用例。** `transaction.rs:795-810` 的次序是
  「先 `for placement in release { allocator.release(*placement, txg); }`，再八个落点逐个 `allocate_*`，
  某一个拿不到就 `?` 返回 `NoSpaceFor`」。返回时上一版八个落点**已经全部改写成已释放 + 释放代 = 这次的 txg**、
  `deferred_slots` 已经加上去了、这次已经拿到的那几个落点也已经进了 `records` 与位图，
  而**没有任何一次发布成立**。调用方拿同一个 `previous` 重试 ⇒ 撞 `assert!(!record.is_released, "同一个落点释放了两次")`。

副本探针（同一个文件）：

```
test probe_publishing_twice_from_the_same_previous_panics - should panic ... ok
```

它造的是「拿同一个 `previous` 发两次」，与「`NoSpaceFor` 之后重试」是同一条断言；
`NoSpaceFor` 自己要把 4 GiB 镜像填满才走得到，这一轮没在副本上跑出来，**记成推理**。

**该有而没有的用例**：三条错误路径里只有两条被钉过「池没动」，`NoSpaceFor` 那条既没有「池没动」也没有
「池动了多少、怎么退回去」。按 `.claude/rules/fs-design.md` 的「释放空间这个操作本身不需要申请空间」，
一次失败的发布把上一版整个标成已释放、而新版本没出生，是能把池推进不可退状态的形态。

## Y2 空闲独立维护：这处改法没有任何会红的东西守着

`.claude/kb/decisions/05-快照-空间记账机制.md:398-401` 逐字：

```
⚠️ **第 2 项必须独立维护，不许由 `容量 − 已分配` 现算。**
I-5.2（空闲统计对得上） 逐字就是「空闲统计 == 总空间 − 已分配空间」——若空闲就是这么算出来的，
**那条不变量是恒真式，判别力为零**（`.claude/singlefs-ai-sop/rules/test-discipline.md`
「写完一个检查，问：如果被测对象真的坏了，它会不会红」）。两个数要走两条不共享的加减路径。
```

### 打中：把改法整个撤回，全仓零判红

副本上做的变异（三处，等于回到 HEAD 的形态）：

1. `DeviceFreeMap::free_slots` 这个字段删掉（`allocator.rs:110`）；
2. `new` 里 `free_slots: unit_area_slots,` 那一行删掉（`allocator.rs:129`）；
3. `mark_allocated` 里 `self.free_slots -= span;` 那一行删掉（`allocator.rs:214`）；
4. `free_slots()` 的函数体改回 `self.unit_area_slots - self.allocated_slots`（`allocator.rs:155`）。

副本读数：

```
cargo test --workspace      → 全部 test result: ok，0 failed；grep -cE 'warning|^error' = 0
cargo clippy --workspace --all-targets -- -D warnings → Finished（零 warning）
```

**一条用例都没红、一条 clippy 都没响。** 这处改法今天没有任何东西在证明它还在。

为什么两条验收断言拦不住：
- `second_transaction_step_one_overwrite.rs` 里 `assert_eq!(value(STATISTIC_FREE_BYTES), 3_472_506_880, "空闲钉绝对值…")`——
  撤回之后 `211968 − 23 = 211945`，`211945 × 16384 = 3 472 506 880`，**同一个数**；
- `allocator.rs` 单测里 `assert_eq!(pool.devices[0].free_slots(), 211_968 - 5, "空闲不变")`——
  这句本身就把期望写成了「容量 − 已分配」的形状，撤回之后当然还相等。

第一轮判决第三节那一格给的会红变异是「把 `mark_allocated` 里的减法摘掉」。那条**只摘一半**（留下字段，
让两个计数器脱钩）确实会红——但它证明的是「两个计数器要同步」，不是 D5 要的那件事。
D5 禁的是**第二个数由第一个数现算**，而那条路今天一个字都没被测过。
按 `.claude/singlefs-ai-sop/rules/show-me-test.md`「每个『验证』都得能失败」，这一格是空的。

### 顺带：I-5.2 今天到底在判什么，与 I-3.1 能不能被同一条变异一起骗过

`crates/singlefs-checker/src/walk.rs:783` 判的是 `free + allocated == capacity`，
两个数都取自**最新根下面的记账树**，都是 `DeviceFreeMap` 的字段投影；
而 `free_slots` 与 `allocated_slots` 全仓只有一处写（`allocator.rs:213-214`，同一对语句 `+= span` / `-= span`），
`mark_released` 不碰它们（`allocator.rs:163-172`）。⇒ 它们的和是**代码构造上的常量**，
I-5.2 唯一能抓的是「那一对语句里改了一条没改另一条」，以及「几何常量与镜像实际大小对不上」。
**它抓不到「空闲是从已分配算出来的」这件事本身**——这正是上面那条变异全绿的机理。

I-3.1（`walk.rs:776`）判的是 `allocated == walked`，`walked` 走的是根环全部有效根的引用并集，
与记账值不共享代码路径 ⇒ 它是真判据。⇒ **两条被同一条变异一起骗过的形态是有的**：
凡是**同时**改 `allocated_slots` 与 `free_slots`（例如 `allocated_slots += 2 * span; free_slots -= 2 * span`）
的变异，I-5.2 恒绿（和仍等于容量吗——不等，这条会红）；
真正一起骗过的是**只改记账条目那一侧的口径**：把 `STATISTIC_FREE_BYTES` 与 `STATISTIC_ALLOCATED_BYTES`
两行同时按同一个偏移平移，`free + allocated` 不变 ⇒ I-5.2 绿，而 I-3.1 会红。
⇒ 结论：**I-3.1 罩得住的那一格，I-5.2 罩不住任何额外的东西**；I-5.2 今天的独立判别力仅限于
「几何常量 vs 镜像大小」。撤回变异全绿这一条已经把这句话坐实了。

## Y3 装不下报错：算式与时点都核过了，是对的；漏的是「同一类 panic 只换了一个」

### 逐项核，全对

| 问 | 核 | 判 |
|---|---|---|
| 容量算式 `(16384 − 头) / 20 = 812` | `index_node_entry_capacity`（`unit.rs:124-132`）用 `index_node_header_bytes(key_width)`；`crates/singlefs-format/src/lib.rs:227` 逐字 `assert_eq!(index_node_header_bytes(10), 135, "分配记录树的码 2 头");`；`(16384 − 135) / 20 = 16249 / 20 = 812` | ✔ |
| 头 135 里 `reserved_bytes` 算不算 | 算。`unit.rs:33-35` 的 `reserved_bytes()` 就是 `NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29`（`format/src/lib.rs:30`），而 `index_node_header_bytes` 的定义里已经含着它（`format/src/lib.rs:50-53`）。改前 `build_index_node` 用的 `entries_start = header_end + reserved_bytes() = (135 − 29) + 29 = 135` ⇒ **新旧两条断言逐字等价**，改法没有悄悄放宽或收紧 | ✔ |
| 「这次之后的记录数 = 现有 + 8 × 盘数」 | `transaction.rs:781-782` 是 `allocator.records().len() + TransactionUnit::IN_BUMP_ORDER.len() * allocator.devices.len()`；`PoolAllocator::record`（`allocator.rs:296-309`）对每个落点逐盘各 push 一条 ⇒ 每次发布每盘恰好 8 条。暖机与取号不走 `publish_file_version`（`transaction.rs` 里只有 `publish_first_file` / `publish_overwrite` 调它），**没有「空发布」这一格**；`release` 只改写不 push ⇒ 不影响计数。三盘时是 `+24`，式子照样对 | ✔ 三种情形都不成立反例 |
| 报错在动分配器之前 | 判定在 `transaction.rs:781-792`，`for placement in release` 在 `transaction.rs:795`，落点循环在 `transaction.rs:801` ⇒ **判定确在两者之前**，验收里 `assert_eq!(pool.allocator.records().len(), 804)` 与「最后一版的八个落点没被释放」钉住了 | ✔ |

⇒ **Y3 这一处没打中**：四个问题逐个核下来，改法是对的。

### 但同一个公开入口里还有三条「装不下 ⇒ panic」没换，其中一条一行代码就走得到

第一轮判决给 Y3 定的口径是「不许走到 `build_index_node` 的断言」。改法只覆盖了分配记录树那一棵：

| 路 | 装不下时 | 门槛 | 走得到吗 |
|---|---|---|---|
| 分配记录树 | `PublishError::AllocationRecordsExceedOneNode` ✔ | 812 条 | 第 50 次覆盖写 |
| **数据单元载荷** | **panic**「载荷装不进一个单元」（`unit.rs:228-231`） | 32768 − 105 − 29 = **32 634 字节** | **`FirstFile.content` 传一个大一点的切片就到了** |
| 记账树 | panic（`unit.rs:159-162`） | 条目 34 字节、头 159 ⇒ 477 条；条目数 = 3 + 6 × 盘数 ⇒ **80 块盘** | 远 |
| 树表 | panic（同上） | 条目 200 字节、头 131（key 宽 8）⇒ 81 条；今天 7 条 | 步 3 之后建到第 82 棵树 |
| 映射树 / extent 根 / inode 根 / inode 叶 | panic（同上） | 映射条目恒 6 条、extent 恒 1 条 | 单文件形态下走不到 |

副本探针（同一个测试文件）：

```
test probe_content_larger_than_the_data_unit_payload_still_panics - should panic ... ok   # content 32 640 字节
```

**该有而没有的用例**：`publish_overwrite` 的公开签名收 `FirstFile { content: &[u8] }`，
对 `content` 的长度**一个前置检查都没有**，而它比「第 50 次覆盖写」好走得多。
按 `.claude/singlefs-ai-sop/rules/code-discipline.md` 的「能恢复的失败走 `Result`；不变量被破坏是 bug，用断言」，
「用户给的内容太长」是调用方能恢复的失败，不是不变量被破坏 ⇒ 它该是 `PublishError` 的一个成员，
和 `AllocationRecordsExceedOneNode` 同一格。记账树与树表那两条门槛远，可以只记账不改。

## Y4 oracle 带实例：择新序核过是对的，打中的是 oracle 自己的两处假阴性

### 择新序：与两条条款一致，没打中

`.claude/kb/decisions/22-单元原子性怎么合成.md:488` 逐字：

```
| **实例代号** | **4** | D16（发布语义）已定项 6 骑手 1：与 D23（journal 的角色与格式）已定项 9 的 journal 实例代号**同一个计数**；择新在 checkpoint_txg 平局时按它高者赢（设备失而复得会造出两条同 txg 的合法根，checker 欠账见 [checks-owed.md](../checks-owed.md) C88（根环的时间线判别未实现）） |
```

- `newest_persisted_root`（`crash.rs:596-611`）`.max()` 在 `(CheckpointTxg, InstanceGeneration)` 上 ⇒ txg 为主、实例平局高者赢 ✔
- `choose_root`（`recovery.rs:281-297`）比的是 `(candidate.checkpoint_txg, candidate.instance)` ⇒ 同序 ✔
- 池级 checker（`walk.rs:704-708`）`max_by_key(|(_, _, root)| (root.checkpoint_txg, root.instance))` ⇒ 同序 ✔
- 根记录偏移：`root_record.rs:12` 逐字「magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + …」⇒ 实例在 24、txg 在 28，`root_identity_of_write`（`crash.rs:614-631`）读的两个偏移**都对** ✔
- 暖机空发布的根算不算一版：不算，`versions` 里只有 txg 3 / 4，落在暖机根上恢复报 `NoFile`、`version` 查不到 ⇒ 走 `(NoFile, None) => None` ✔

⇒ 这四问都没打中。

### 打中 ①：落在一个从没发布过的更新的根上、报「没有文件」，oracle 判无违例

`crash.rs:576-585` 的 `match (outcome, version)` 里 `(RecoveryOutcome::NoFile { .. }, None) => None`：
只要 `(effective_txg, effective_instance)` 不比 `newest_persisted` 旧、又不在 `versions` 里，
报「没有文件」就一律放过。而**恢复走到一个比最新持久根还新的根是合法的**（journal 前缀施加，靶向对照 ③ 就是这一格），
所以这条路不是不可达的：一次重放把根推到一个没人发布过的代、而那个代下面的树表是空的，
表现就是「文件没了」，oracle 全绿。

副本读数（探针加在 `crates/singlefs-harness/src/crash.rs` 的 `r2_probe_tests`，直接喂 `oracle_violation_for_versions`）：

```
test crash::r2_probe_tests::a_fabricated_newer_root_reporting_no_file_is_blessed ... ok
test crash::r2_probe_tests::a_fabricated_instance_reporting_no_file_is_blessed   ... ok
```

前者：`effective_root = (实例 1, 第 5 代)`、`newest_persisted = (第 4 代, 实例 1)`、
versions 里只有第 3 / 4 代、outcome 是 `NoFile` ⇒ **返回 `None`**，
而第 4 代根下面文件是在的。后者把实例换成从没存在过的 9，结果一样。

**该有而没有的用例**：`oracle_instance_tests` 两条只测了「落在低实例要判违例」「同 txg 两实例是两条版本」，
没有一条问「落在一个没有版本的根上、而更旧的根下面有文件」。
按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」，
oracle 是层 0 唯一的判据，它的假阴性就是整条 524 312 状态流的盲区。

### 打中 ②：`JournalPolicy::Ignore` 那一遍恢复从头到尾不过 oracle

`crash.rs:668-669` 两遍恢复都跑了，而 `ignored` 只进了一个计数器（`crash.rs:674-676`
`if consulted.outcome != ignored.outcome { tally.journal_differing_states += 1; }`），
**再也没有别的地方读它**；oracle 只判 `consulted`（`crash.rs:688-693`）。

发布 B 之后这不再是一句形式话：两遍恢复现在会读出**两个不同的版本**——
`second_transaction_step_one_overwrite.rs` 里那条 `damage_probes…` 用例逐字写着
`recover(&root_damaged, JournalPolicy::Ignore).outcome` 是第 3 代根 + `file_content()`，
而 `Consult` 是第 4 代根 + `second_content()`。⇒ Ignore 这一遍读错版本，没有任何东西会说话。

而 Y4 的改法**刚好把判它所需的机械备齐了**：`oracle_violation_for_versions` 是按「实际走的根」查版本的，
对 `ignored` 照样成立。副本上把它接上（同一个函数、同一份 `versions` 与 `newest_persisted`，
另计一个数不混进 `tally.violations`）：

```
test every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims ... ok
test targeted_controls_on_the_second_publish_go_red_where_they_should ... ok
test result: ok. 2 passed; 0 failed; 1 ignored
```

26 个状态**零新增违例**，靶向对照五格照过 ⇒ 接上去不要钱。
唯一要注意的是别直接 `tally.violations += 1`：那样靶向对照 ② 会从 1 变 2
（那一格两遍恢复都违例，副本上跑过一次，`assert_eq!(root_without_units_tally.violations, 1)` 判红）。

### 顺带：实例这一维在枚举里被走到过零次

副本探针读出这条流四条根槽写的实例代号：

```
PROBE root_instances=[1, 1, 1, 1]
```

⇒ Y4 加进去的实例维，在全量 524 312 个状态里**一个状态都没走到**；
它今天的全部证据就是 `oracle_instance_tests` 那两条手喂的单测。
这不是改法的错（步 4 才有第二个实例），但里程碑现状里不该写成「层 0 判了实例维」。

### 顺带二：两个「择新序」在仓里并存，各自对应一条条款，oracle 只跟了其中一条

`recovery.rs:390` 的 journal 水线是 `let water = (root.instance, root.checkpoint_txg);`，
`recovery.rs:394` 按 `(record.instance, record.checkpoint_txg) > water` 取前缀——**实例为主**；
而根环择新（`recovery.rs:290`）与 oracle（`crash.rs:604-609`）都是 **txg 为主**。
两者与各自的条款一致（`recovery.rs:7` 的文件头逐字「前缀 = `(实例代号, checkpoint_txg)` 严格大于所选根」，
D22 已定项 7 管根环），所以**不是代码与条款不符**；但它意味着「一条实例低、txg 高的记录」
在根环序里在所选根之上、在 journal 水线之下，两条路会给出相反的判定。
单实例流上走不到，步 4 一开就走得到；oracle 只跟了根环那一条。记一笔，不当打中。

## 顺带：`allocation_records_are_one_per_device` 逐盘核

### 打中：放过一个本该拒的——同一盘同一个槽两条记录

`recovery.rs:517-547` 把每盘的记录收成 `BTreeSet<(u64, u16, u64, bool)>`（槽, 跨度, 代, 已释放），
再要求各盘集合相同、第一盘不少于 10 个落点。函数的文档逐字说的是
「分配记录「每个落点每盘各一条」」，而**集合里没有任何东西要求一个槽只出现一次**：
同一盘同一个槽的两条记录（代不同）在集合里就是两个元素，两盘对称就过。

这在走读这一层是真盲区：`parse_index_node`（`unit.rs:340-380`）只判声明长度、条目宽、条目区边界，
**不判 key 严格递增**；判 key 严格递增的 `check_index_node_keys` 全仓只在池级 checker 里被调
（`crates/singlefs-checker/src/walk.rs:187`），而 `walk_to_file` / `recover` 那条路根本不跑池级 checker。
⇒ 分配记录 key 是 (设备 4, 槽号 6)，同盘同槽两条记录 key 相同，**恢复这条路上没有任何一处会红**。

副本读数（探针加在 `recovery.rs` 的 `allocation_records_per_device_tests`）：

```
test recovery::allocation_records_per_device_tests::probe_two_records_for_the_same_slot_on_the_same_device_are_accepted ... ok
```

造的是第一个事务那 10 个落点之外，两盘各多一条 `(50176, 2, 第 9 代, 未释放)`——
槽 50176 在两盘各已有一条第 3 代已释放的记录 ⇒ 每盘集合 11 个元素、两盘相同 ⇒ **返回 true**。

**为什么这一格要紧**：`allocator.rs:305-309` 的注释逐字「释放时条目不删、不点删：改写成已释放 + 释放代，
**留到该落点被重新分配时覆盖**」，而 `PoolAllocator::record`（`allocator.rs:296-309`）是无条件 `push`，
一条「覆盖」的代码都没有——步 5 的回收一接上，重新分配一个已释放的槽就会**追加**第二条记录，
正好落进这个盲区。而且 `release` 的 `.find()`（`allocator.rs:315-318`）取的是第一条命中的，
那条是已释放的 ⇒ 下一次释放该槽撞 `assert!(!record.is_released, "同一个落点释放了两次")` panic。
⇒ 这条盲区与 Y1 那两条 panic 是同一条链。

### 没找到「拒而本该过」的

逐个试过：记录在盘上的顺序（集合无序，不影响）、三盘（各盘集合相同即过）、
一条已释放一条没释放（`a_release_rewritten_on_one_device_only_is_rejected` 正是这一格，拒得对）、
落点数多于 10（下限，不是等号，过得对）。唯一一处形式上会误拒的是 `device_identities` 自己带重复项
（`placements_per_device.len() != device_identities.len()` 会红），而那意味着超级块本身坏了，不算误拒。
⇒ 这一半没打中。

## 判决与欠账

| 编号 | 打中什么 | 建议 |
|---|---|---|
| **Z1** | 映射条目在、落点指错 ⇒ `release` panic（两条：无记录 / 跨度不符），而 Y3 刚立了「不许走到断言」的口径 | 把 `PoolAllocator::release` 的两条断言改成 `PublishError` 的两个成员（落点无记录 / 跨度与记录不符），各配一条造「条目在、落点变了」的用例 |
| **Z2** | 跨度不经映射、按单元种类现推；验收那句「两条路互相对」在跨度维上是恒等式 | 跨度改从上一版的分配记录取（映射给槽、记录给跨度），验收改成拿记录的跨度对种类表 |
| **Z3** | `NoSpaceFor` 在释放之后、分配到一半返回，留下半新的池；重试撞 panic | 要么把释放挪到八个落点全拿到之后，要么给失败路径一条回退；补一条「失败之后池没动」的用例 |
| **Z4** | **Y2 整处改法撤回，全仓零判红零警告** | 立一条会红的检查：拿一份「空闲 = 容量 − 已分配」的坏镜像喂池级 checker，要求 I-5.2 判红；或在验收里把空闲与已分配钉成**两个各自算出来的绝对值**，而不是一个数与它的补 |
| **Z5** | `FirstFile.content` 超长 ⇒ panic（32 634 字节，一行就走得到），与 Y3 修的是同一类 | 加 `PublishError::ContentExceedsDataUnit { bytes, capacity }`；记账树 477 条 / 树表 81 条那两条门槛远，记账不改 |
| **Z6** | oracle 假阴性：落在无版本的更新的根上报「没有文件」，一律放过 | `(NoFile, None)` 那一臂加一条：`versions` 里存在不比 `effective_root` 旧的版本时判违例 |
| **Z7** | `JournalPolicy::Ignore` 那一遍恢复不过 oracle，而发布 B 之后两遍会读出两个版本 | 接上（副本上 26 个状态零新增违例），另计一个 `ignored_violations`，别混进 `tally.violations` |
| **Z8** | 逐盘核用集合比，放过「同盘同槽两条记录」；走读不判 key 严格递增 | 逐盘先核「槽号不重复」再比集合；`record()` 的「重新分配时覆盖」要么实现、要么把注释改成实况 |
| 记一笔 | 实例维在全量 524 312 个状态里零覆盖；根环择新（txg 为主）与 journal 水线（实例为主）两序并存，oracle 只跟了前者 | 里程碑现状别写成「层 0 判了实例维」；两序的分歧记进步 4 决策点 |

**没打中的**：Y3 的容量算式（812）、头 135 含不含 `reserved_bytes`、`+8 × 盘数` 在空发布 / 多对象 / 跨盘数上、
报错时点在动分配器之前；Y4 的择新序（三处同序）、根记录偏移 24 / 28、暖机根不算一版；
逐盘核的「拒而本该过」。这七问逐个核下来改法都是对的，依据写在各自那一节。

**副本上动过的文件**（都不入库，原仓一个字节没改、无任何 git 写操作）：
`crates/singlefs-core/src/allocator.rs`（Y2 撤回变异，跑完已还原）、
`crates/singlefs-core/src/recovery.rs`（一条探针单测）、
`crates/singlefs-harness/src/crash.rs`（两条探针单测 + Ignore oracle 试接）、
`crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`（四条探针）、
`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`（一条读数探针）。
