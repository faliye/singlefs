# 发布 B（覆盖写 + 释放）代码三方第一轮：攻方腿（Opus）报告

**日期**：2026-09-16。**分工**：背景材料第三节的 X1（释放的口径）、X2（记账口径）、X4（层 0 多版本 oracle 的假阴性）、X5（对照够不够）。

**方法与口径**：
- 原仓 `/home/fy5090/code/singlefs` 全程只读，未改任何已有文件、未做任何 git 写操作。
- 改代码跑用例在副本 `/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/m2-code-r1-opus-copy/`（`rsync -a --exclude target --exclude .git`）。
  **下面凡带「副本」二字的数都是在这个副本上跑出来的**，不是入库产物；要进 kb 得按 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」重做。
- 副本基线：`cargo test --workspace` 退出码 0（debug；全量层 0 那条 `#[ignore]` 没跑）。变异一次改一处、跑完还原，`--no-fail-fast`。
- 两个只在副本里的探针文件：`crates/singlefs-harness/tests/zz_opus_probe.rs`（把池级 checker 的全部判定原样打印）、`zz_opus_probe2.rs`（连续覆盖写 N 次；直接喂 oracle 两条同 txg 不同实例的版本）。它们不做断言，是观测不是验收。

## 一、判决摘要

| 判据 | 判决 | 打中的是什么 |
|---|---|---|
| **X1 释放的口径** | **打中（两条）** | ① 已释放的槽**永远回不来**：`deferred_slots` 只增不减，全仓没有一处实现 D16 已定项 1 的「可再分配」谓词 ⇒ D3 已定项 9 第 2 条的「有界步数」界是 ∞，没有任何会红的检查；② 由 ① 直接推出一段**按条款走不通的历史**：同一个 4100 字节文件连续覆盖写，第 49 轮还绿，**第 50 轮 `crates/singlefs-core/src/unit.rs:151` panic「条目装不进一个节点」** |
| **X2 记账口径** | **打中（一条）** | **I-5.2 在这批代码上是恒真式，判别力为零**——D5 已定项 4 第 398–401 行逐字禁的就是这个写法，而 `crates/singlefs-core/src/transaction.rs:882` 那行注释声称遵守了它。变异证据：`allocated_slots` 写错 163840 字节，**I-3.1 判违例、I-5.2 判 Holds** |
| **X4 层 0 oracle 的假阴性** | **打中（一条）** | `oracle_violation_for_versions` 与 `newest_persisted_root_txg` **都把实例维丢了**：`PublishedVersion` 只按 `checkpoint_txg` 索引、`newest_persisted` 只取 txg 的 max。D22 已定项 7 择新是 (txg, 实例) 字典序。副本实测：恢复落在实例 1 的第 7 代根上、而实例 2 的第 7 代根也持久，oracle 判 `None`（绿）。B 这条流全是一个实例 ⇒ 今天不可达；**步 0 的脚本含切换与回退，那时它承重** |
| **X5 对照够不够** | **打中（两条）** | ① 「释放代写错」与「树表诞生 txg 改了」**各只有一条断言红，而且都是「写者自己写的值对不对」那一型**——池级 checker 零违例、冷启动走读零差别、层 0 oracle 零违例；② 「映射条目没删」这条变异**造不出来**：映射树每次发布从六条现成条目整棵重建，「删掉 A 的六条」不是一条代码路径，那条断言是对重建的同义反复 |
| （X3 / X6 / X7 不是我这条腿的格，只在第六节记两条顺手核出来的） | — | — |

**反向接受条款对照**：X1 / X2 / X4 打中 ⇒ 按跑前条款要「改代码、补一条会红的用例、再攻一轮」；X5 打中 ⇒ 补对照。第七节按这条给每一条写了改哪、补哪条会红的用例。

## 二、X2：I-5.2 是恒真式，而代码注释声称它不是

### 条款（整行抄，`.claude/kb/decisions/05-快照-空间记账机制.md:398-401`）

```markdown
⚠️ **第 2 项必须独立维护，不许由 `容量 − 已分配` 现算。**
I-5.2（空闲统计对得上） 逐字就是「空闲统计 == 总空间 − 已分配空间」——若空闲就是这么算出来的，
**那条不变量是恒真式，判别力为零**（`.claude/singlefs-ai-sop/rules/test-discipline.md`
「写完一个检查，问：如果被测对象真的坏了，它会不会红」）。两个数要走两条不共享的加减路径。
```

`.claude/kb/invariants.md:167` 那一行逐字：

```markdown
| I-5.2 | 空闲统计对得上 | 空闲空间统计 == 总空间 − 已分配空间 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） |
```

### 代码：三处都走同一条减法

| 处 | 路径与行 | 干了什么 |
|---|---|---|
| 运行时的量 | `crates/singlefs-core/src/allocator.rs:149-153` | `free_slots()` 的整个实现是 `self.unit_area_slots - self.allocated_slots`，doc 注释自陈「空闲 = 容量 − 占着的（I-5.2（空闲统计对得上））」 |
| 写进记账行 | `crates/singlefs-core/src/transaction.rs:882-886` | 第 882 行注释写「第 2 项独立维护、不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）」，第 885 行调的正是 `device_map.free_slots()` |
| checker 判 | `crates/singlefs-checker/src/walk.rs:779-785` | `capacity` 取 `device_bytes / SLOT_BYTES - geometry.unit_area_start_slot`，判 `free + allocated == capacity`；`free` 与 `allocated` 两个数都从盘上那棵记账树读 |
| 验收断言 | `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs:238-242` | 断言 `value(STATISTIC_FREE_BYTES) == (unit_area_slots - occupied_slots) * SLOT_BYTES`，消息写「空闲 + 已分配 = 单元区（I-5.2）」——第三次走同一条减法 |

**`DeviceFreeMap::new`（allocator.rs:117）算 `unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT`，与 checker 的 `capacity` 是同一个式子。**
把这两处代进 I-5.2 的判据：`(capacity − allocated) + allocated == capacity`。
⇒ **I-5.2 今天判的是「运行时的单元区几何常量等不等于 checker 的单元区几何常量」，对分配器的记账一个字都没判。**

### 变异证据（副本）

变异 **M-X2**：`allocator.rs` 的 `mark_released` 里加一行 `self.allocated_slots -= span;`（milestone 步 2 现状自陈「此前这里的预想「已分配减、空闲不变」两个都错」的那个错读法）。
副本上 `zz_opus_probe.rs` 打印的池级 checker 判定，逐行抄：

```text
PROBE accounting statistic=1 device=0 value=212992
PROBE accounting statistic=2 device=0 value=3472670720
PROBE checker I-3.1 Violated("盘 0：记账的已分配 Some(212992)，遍历全部有效根得到 376832")
PROBE checker I-5.1 Holds
PROBE checker I-5.2 Holds
```

已分配少报了 163840 字节（10 槽），**I-3.1 红、I-5.2 绿**，因为 3472670720 + 212992 = 3472883712 = 211968 × 16384 恰好还是单元区。
**`allocated_slots` 取任何值 I-5.2 都绿**——这不是「这次恰好没抓到」，是这条不变量在这批代码上没有能红的输入（除非有人去改 `free_slots()` 那条减法本身，或者两个几何常量打架）。

⚠️ 这一条**不是「又一条没抓到的变异」**：D5 已定项 4:398 明令不许这么写，`transaction.rs:882` 的注释说自己没这么写，而代码这么写了。**注释与代码不符这件事本身也没有任何检查在看。**

## 三、X1：释放的口径——已释放的槽永远回不来，第 50 轮覆盖写 panic

### 条款（整行抄）

`.claude/kb/decisions/16-发布语义.md:371`：

```markdown
| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |
```

`.claude/kb/decisions/03-空间分配.md:365-366`：

```markdown
2. **删掉的空间在有界步数内可用**：删掉一个 s 字节的对象之后，同样大小的写必须在有界步数内成功。
   这个界要写成算得出的数，不许是「最终会」。
```

`.claude/kb/decisions/03-空间分配.md:180` 里那一句（同一行里，整行太长，只指出它在哪一行、不另做短版本）：条目数上界「从「已分配落点数」变成「曾分配过、尚未被覆盖的落点数」，仍以容量 / 16 KiB 为界」。

### 现查：可再分配谓词全仓没有实现

`grep -rn "F_生效\|环里最旧有效根\|可再分配\|reusable" crates/ --include=*.rs` 在原仓只命中一处，而那一处是**注释**：

- `crates/singlefs-core/src/allocator.rs:308` 的 doc 注释「条目不删；槽仍占着，要等释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配（D16（发布语义） 已定项 1）。」

`deferred_slots` 的全部出现（`crates/singlefs-core/src/allocator.rs` 第 110 / 126 / 155-156 / 167 行，加一处单测）里，**只有第 167 行 `self.deferred_slots += span;` 写它，没有任何一处减它**；`allocated[]` 位图在 `mark_released`（第 160-168 行）里一位都不动。
⇒ **一个落点一旦进 defer 队列，这个进程里再也不会被发出去，也没有任何路径在别的进程里把它发出去**（分配器每次挂载从头建，见第六节 X6 那条）。

### 一段按条款走不通的历史（副本实测）

装置：`zz_opus_probe2.rs`，`build_pool` 之后对**同一个 4100 字节文件**反复调 `publish_overwrite`，每轮把 `pool.output` 换成新的一版（即每轮释放上一轮的八个单元）。逐行抄产物：

```text
PROBE2 round=1 txg=4 data_slot=50182 allocated=376832 free=3472506880 defer=163840 records=36
PROBE2 round=2 txg=5 data_slot=50184 allocated=540672 free=3472343040 defer=327680 records=52
PROBE2 round=12 txg=15 data_slot=50204 allocated=2179072 free=3470704640 defer=1966080 records=212
PROBE2 round=49 txg=52 data_slot=50534 allocated=8241152 free=3464642560 defer=8028160 records=804
thread 'repeated_overwrites_never_give_a_released_slot_back' panicked at crates/singlefs-core/src/unit.rs:151:5:
条目装不进一个节点
```

（十二轮那一版另跑过一次，`PROBE2 十二次覆盖写之后 checker 违例：[]`——池级 checker 全程零违例。）

**读数**：
1. 文件从头到尾只有**一个 32 KiB 数据单元**是活的，而 49 轮之后每盘 `已分配` = 8241152 字节（503 槽）、`defer` = 8028160 字节（490 槽）、`空闲` 单调下降 8028160 字节。**删掉的空间一个字节都没回来**，界不是「算得出的数」，是 ∞ ⇒ D3 已定项 9 第 2 条在这批代码上不成立。
2. 分配记录条数每轮 +16（8 个单元 × 2 盘），49 轮 804 条。`publish_file_version`（`crates/singlefs-core/src/transaction.rs:829-844`）把**全部**分配记录塞进**一个** `build_index_node`，没有分裂；`crates/singlefs-core/src/unit.rs:151` 的 `assert!` 在装不下时 panic。
   第 50 轮 820 条 × 20 字节 = 16400 > 16384 − 头 ⇒ **进程 panic，不是 `PublishError`**。
3. ⚠️ **这一格今天在门禁里是全绿的**：层 0 两条流只跑到 B（两次发布），验收用例只跑一次覆盖写，池级 checker 在 49 轮的镜像上零违例。**没有任何检查看得见「第 50 轮会 panic」。**

### `release` 的两盘同槽假设与 `placements` 的跨度推法：核过，今天成立，但两处都是没有检查的隐式约定

- `crates/singlefs-core/src/allocator.rs:309-326` 的 `release` 按 `(device, slot)` 用 `.find()` 取第一条记录。今天 `(device, slot)` 在 `records` 里唯一，因为 `record()`（第 292-305 行）每个落点每盘只 push 一次、`mark_allocated` 断言整跨度此前空闲。
  ⚠️ **但 `record()` 是 push，不是「覆盖」**，而 D3 已定项 7 逐字要求「条目留到该落点被重新分配时覆盖」。一旦第七节那条「可再分配」补上，同一个 `(device, slot)` 会出现**两条**记录，key 在分配记录树里不再唯一，而 `release` 的 `.find()` 会永远改到旧的那条。**今天不可达只是因为可再分配没实现**，两个洞互相遮掩。
- `crates/singlefs-core/src/transaction.rs:516-528` 的 `placements()` 按 `unit.identity.placement()` 的**种类**推跨度，不记实际分配了几个槽。今天两边一致（`UserData` 与 `TwoSlotsAligned` 都是 2、`OneSlot` 是 1）。把 `TwoSlotsAligned => 2` 改成 `=> 1`（变异 M7）后副本上七个测试红，所以这一格有对照。⚠️ 但它抓的是「推错了」，抓不到「分配器实际给的跨度与种类不一致」——那要等码 3 容器换档或跨度 > 2 的单元出现，今天没有这种单元。

### 我没能打中的（照实报）

- **「已释放的槽被重新发出去」构造不出来**：`lowest_user_data_slot`（allocator.rs:214-230）读的是 `allocated[]` 位图，`mark_released` 不清位 ⇒ 结构上发不出来。这是上面第 1 条的另一面：错的方向不是「发早了」，是「永远不发」。
- **`release` 在两盘落点不同时会出错**这条：今天 `Placement` 只有一个 `slot` 字段、`record()` 给两盘写同一个槽号（D2 已定项 10 的两盘同构），构造不出两盘不同槽的历史，除非先改数据结构。记成 X6 那一格的隐式写死，不算打中。

## 四、X4：层 0 多版本 oracle 丢了实例维

### 条款（整行抄，`.claude/kb/decisions/22-单元原子性怎么合成.md:488`）

```markdown
| **实例代号** | **4** | D16（发布语义）已定项 6 骑手 1：与 D23（journal 的角色与格式）已定项 9 的 journal 实例代号**同一个计数**；择新在 checkpoint_txg 平局时按它高者赢（设备失而复得会造出两条同 txg 的合法根，checker 欠账见 [checks-owed.md](../checks-owed.md) C88（根环的时间线判别未实现）） |
```

`.claude/kb/decisions/23-journal的角色与格式.md:1237` 同一句的另一半：

```markdown
2. **记录水位 `(实例代号, checkpoint_txg)` 按实例代号为主比**，与 jsn 同序；择根照旧 txg 为主、实例代号破平局（D22（单元原子性怎么合成） 已定项 7），两处比的东西不同，各守各的序。按 txg 为主时，回退新根取「根环最大 + 1」，而被抛弃实例开放 checkpoint 里已提交的记录 txg 可以更大，会被当成水位之后施加到回退根上（第二轮反推腿 R-TXG，零故障）。
```

### 代码：两处都只拿 txg

- `crates/singlefs-harness/src/crash.rs:593-602` `newest_persisted_root_txg` 把每次发布映射成 `CheckpointTxg(publish.checkpoint_txg)` 再取 `.max()`——**实例代号从来没被读出来过**（`root_txg_of_write` 只读根记录偏移 28 的 8 字节 txg，crash.rs:604-615）。
- `crates/singlefs-harness/src/crash.rs:573-575` `versions.iter().find(|version| version.checkpoint_txg == effective_txg)`——`PublishedVersion`（crash.rs:548-552）只有 `checkpoint_txg` 与 `content` 两个字段，没有实例；`.find()` 取第一条命中。
- `crash.rs:566` 的回退闸判的是 `effective_txg < newest`，**两个都是纯 txg**。

`oracle_violation_for_versions` 拿到的 `effective_root` 是 `(InstanceGeneration, CheckpointTxg)` 二元组，第 563 行 `let Some((_, effective_txg)) = effective_root` 把实例那一半**用 `_` 丢掉**。

### 副本实测

`zz_opus_probe2.rs` 直接喂 `oracle_violation_for_versions` 两条同 txg 不同实例的版本，逐行抄：

```text
PROBE2 oracle 对「落在低实例的同代根」判：None
PROBE2 oracle 对「实例 2 的内容」判：Some("读回的内容不对（走的是第 7 代根）")
```

第一行：盘上实例 2 写的第 7 代根持久着（`newest_persisted = 7`），恢复落在**实例 1** 的第 7 代根上并读出实例 1 的内容——按 D22 已定项 7，择新该选实例 2 那条，**这是一次真的退代**；oracle 判 `None`，绿。
第二行更糟：同一个 `versions` 里两条同 txg，`.find()` 只认第一条 ⇒ 读出实例 2 的正确内容反而被判违例。**oracle 在这一格上两个方向都错，一个假阴性一个假阳性。**

### 射程（说清它今天碍不碍事）

- **B 这条流不可达**：`second_transaction_step_zero_layer0.rs:82-91` 的 `versions` 只有 txg 3 与 txg 4 两条，整条流全是 `InstanceGeneration(1)`，四条根槽写 txg 各不相同 ⇒ 今天 524312 个状态里**一个都踩不到**。所以这不是一条「今天判错了」的账。
- **步 0 会踩到**：`.claude/kb/milestone/02-second-txn.md:60` 逐字「harness 能跑「固定脚本（预想）」那一段列的脚本（mkfs、A、B、**切换**、C、**回退**与 D、攒根、抬 F、E）」。切换与回退各造一个新实例；D23 已定项 14 的回退新根取「根环最大 + 1」让根的 txg 仍然互不相同，**但被抛弃时间线的根与回退实例的根同时在环里**，而 `newest_persisted_root_txg` 对「被抛弃」这件事一无所知——它会把被抛弃实例那条已持久的根算进 max，于是**恢复正确地不选被抛弃的根反而会被 oracle 判成「根槽已持久而恢复到旧态」**（假阳性），除非步 0 另给它实例表输入。
- 这两件事是同一个病根：**oracle 把「哪条根该赢」压成了一个全序的 txg**，而条款给的是 (txg, 实例) 加实例表有效性。

⇒ **打中的是 oracle 的形态，不是今天的某个数**。按跑前的反向接受条款要「改代码、补一条会红的用例、再攻一轮」：这条的会红用例就是上面那两行探针（把它们做成断言：同 txg 不同实例时 oracle 必须判违例 / 必须分得开两条版本）。

## 五、X5：变异表（副本实测，一次一处、跑完还原）

跑法：`cargo test --workspace --no-fail-fast`，记「红了哪几个测试」。基线全绿。

| # | 变异 | 改哪 | 红在哪 |
|---|---|---|---|
| M-X2 | 已释放的从「已分配」里减掉 | `allocator.rs` `mark_released` 加 `self.allocated_slots -= span;` | 4 个：`allocator::tests::releasing_a_placement...`、`release_rewrites...`（断言「已分配 = mkfs 3 槽 + A 10 槽 + B 10 槽」）、`cold_start...`、层 0 的 `every_crash_state...`（I-3.1 违例数 ≠ 0）。**checker 侧只有 I-3.1 红，I-5.2 绿**（第二节） |
| M1 | **释放代写错**（写 3 不写 4） | `transaction.rs:704` `release(*placement, CheckpointTxg(txg.0 - 1))` | **只有 1 个**：`second_transaction_step_one_overwrite.rs:201` 的 `assert_eq!(record.generation, CheckpointTxg(4), "释放代 = B 的 txg")`。池级 checker 零违例、冷启动走读零差别、层 0 oracle 零违例 |
| M3 | **树表诞生 txg 改了**（跟着这次 txg 走） | `transaction.rs:672` `tree_birth_txg: txg` | **只有 1 个**：`second_transaction_step_one_overwrite.rs:276` 的 `assert_eq!(entry.birth_txg, CheckpointTxg(3), "树的诞生 txg 不随重写变")`。I-9.2 绿——容器头与条目身份引用**一起**变了，四段仍逐字相等 |
| M4 | **超级块世代号不加一** | `transaction.rs:184` `+ 1` 改 `+ 0` | 6 个（取号两条、根槽/超级块那条、覆盖写形态、checker 坏镜像那条）。**有对照** |
| M5 | **反向链算成空** | `transaction.rs:676` `&previous.record_bytes` 改 `&[]` | 7 个（含层 0 两条与靶向对照）。**有对照** |
| M6 | **`release` 只做盘 0** | `allocator.rs:324` 后加 `break;` | 2 个：allocator 单测 + `release_rewrites...`。**有对照** |
| M7 | **`placements()` 跨度推成 1** | `transaction.rs:523` `TwoSlotsAligned => 1` | 7 个。**有对照** |

### X5 打中的第一条：M1 与 M3 的对照是「写者自己写的值对不对」，不是独立的一路

M1 与 M3 各只红一条断言，而那条断言读的是 `publish_overwrite` 返回的**内存态 `TransactionOutput`**，拿它跟同一个测试里写死的常量比。三条独立的路——池级 checker、冷启动走读、层 0 oracle——**对这两条变异一个字都不说**（M1 / M3 的副本产物里 `PROBE checker ... Violated` 一行都没有）。

- **释放代**：milestone 步 2 自陈「「释放代写成 3」这条变异没有会红的检查——三方第一轮三条腿都核过 [invariants.md](../invariants.md) 没有「释放代不小于分配代」这条」。核下来这句**对不变量那一半是准的**，但整句写宽了：验收用例第 201 行确实会红。⇒ 应改成「没有**会红的不变量**，只有一条验收断言」，并按那句自己列的两条出路（立一条不变量 / 不写这条变异）定一个。
- **树表诞生 txg**：milestone 步 2 没提这一格。它与释放代同型且更隐蔽——`birth_txg` 变了之后，I-9.2 判的「子头的四段与条目的身份引用逐字相等」两边**一起**变，恒真。

### X5 打中的第二条：「映射条目没删」这条变异造不出来，那条断言是同义反复

`crates/singlefs-core/src/transaction.rs:958-983` 的 `mapping_entries` 每次发布都是一个**新建的六元 `vec![]`**，条目从这次发布的六个指针现算。**代码里没有「保留上一版条目」这条路径，所以也没有「删掉它」这个动作。**
`second_transaction_step_one_overwrite.rs:262-268` 那条「A 的映射条目一条都不留：释放走映射、条目删掉」的断言，判的是「B 的六个 key 与 A 的六个不相等」——而 key 由 `mapping_key_for_data / _for_node` 从这次的出生 (树, txg) 与写序算出，B 的 txg 是 4、A 是 3 ⇒ **不相等是算术上的必然**，不是删除动作的证据。要造这条变异得先**加**一段把旧条目拼进来的代码。

**同一格更重的一条**：D19 已定项 5 随定案生效的三条硬规则第 1 条逐字是（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:227`）

```markdown
1. **释放一律经映射，不经提示**（E96（混合架构的一致性） 实测：按提示解落点会误放约 5000 次 / 200 轮）。
```

而 `crates/singlefs-core/src/transaction.rs:702-705` 的释放入参是 `previous.placements()`（transaction.rs:677），也就是**上一版 `TransactionOutput` 里记着的槽号**——不经映射。映射在代码里只出现在解引用的**回落**支路（`crates/singlefs-core/src/recovery.rs:772-790`：位置提示读不出来才查映射）。
⇒ **「释放判定路径」这个东西今天在代码里不存在**，milestone 步 2 验收标准里那两条（「释放判定路径对它报「不在映射」」「删掉映射条目，释放判定路径报「不在映射」而不是按提示释放」）**没有被测过**，而步 2 的现状写着「验收四条全过」。今天不出错只因为释放的入参是同一个进程里刚写出来的内存态；**冷启动之后再覆盖写一次就没有这个入参了**（见第六节）。

## 六、顺手核出来的两条（不在我这条腿的格里，交给对应的腿）

1. **（X6）分配器每次挂载从零重建，没有从分配记录树恢复的路径。** `PoolAllocator::new`（`crates/singlefs-core/src/allocator.rs:268-277`）把 `records` 初始化成空、`open_segment` 置 `None`、位图全 false；`recovery.rs` 全文没有一处构造 `PoolAllocator` 或 `DeviceFreeMap`（`grep -rn "PoolAllocator::new\|DeviceFreeMap::new" crates/ --include=*.rs` 只命中 `allocator.rs` 自己与 harness / 测试四处，恢复与挂载路径一处都没有）。
   ⇒ **今天「冷启动之后再写一次」这条路在代码里走不通**：整批覆盖写用例都在同一个进程、同一个 `pool.allocator` 上跑（`second_transaction_step_one_overwrite.rs` 的 `overwrite` 直接用 `&mut pool.allocator`）。
   这不是错，是第一版的范围；但它让第三节那条「已释放的槽永远回不来」在**单进程内**才观测得到，而真正的形态（挂载后位图怎么建、defer 队列的内存态崩溃后丢了谁接手）一格都还没有代码。D3 已定项 7:180 逐字点名了接手的那条闸（「崩溃后 defer 队列的内存态丢失，接手的是「释放代 ≤ max(F_生效, 环里最旧有效根)」这条闸」），实现是空的。
2. **（X7）milestone 步 2 现状那句「空闲 = 单元区 − 已分配（I-5.2（空闲统计对得上））」与代码相符，而 `transaction.rs:882` 的注释与代码不符。** 两处说的是同一件事，一处诚实一处不诚实——要改的是代码（第七节），不是那句现状。

## 七、按跑前的反向接受条款，每一条该怎么处置

| 打中的 | 反向接受条款要的 | 具体到哪一处 |
|---|---|---|
| **X2** I-5.2 恒真 | 改代码 + 补一条会红的用例 + 再攻一轮 | 把 `DeviceFreeMap` 的空闲做成**独立维护的计数**：`new` 里置 `unit_area_slots`，`mark_allocated` 减 span，（将来可再分配时）加回；`free_slots()` 直接返回它，不再做减法。会红的用例：**去掉 `mark_allocated` 里的那次减法，I-5.2 必须由绿转红**（今天做这个变异 I-5.2 纹丝不动）。⚠️ 顺带删掉 `transaction.rs:882` 那句不实的注释，并把 `second_transaction_step_one_overwrite.rs:238-242` 的期望值改成不经「单元区 − 已分配」算出来的常数 |
| **X1-①** 已释放的槽永不回来 | 改代码 + 补一条会红的用例 | 第一版可以不实现可再分配（它要 F_生效 与环里最旧有效根，两样都还没有），**但不许留成静默的空白**：给 `checks-owed.md` 立一笔，题面是「defer 队列只增不减，D3 已定项 9 第 2 条的界是 ∞」，并在 milestone 步 2 的现状里写明。会红的用例可以先做成弱形态：断言 `deferred_slots` 与 `df` 能报的空闲之间的关系今天是什么（`I-5.3（报出的空闲都兑现得了）` 那一条 `.claude/kb/invariants.md:168` 写着「未实现」，正是这一格） |
| **X1-②** 第 50 轮 panic | 改代码 + 补一条会红的用例 | 两条路：(a) 分配记录树支持分裂（真前置，步 3 之后）；(b) 第一版把这条写死成**显式的 `PublishError`** 而不是 `assert!` panic——`crates/singlefs-core/src/unit.rs:151` 今天是 panic。会红的用例：连续覆盖写到条目装不下，必须拿到 `Err`，不许 panic。⚠️ 这一条也可以选「记成欠账、第一版不管」，但那要**写下来**：今天它既没实现也没记账 |
| **X4** oracle 丢实例维 | 改代码 + 补一条会红的用例 + 再攻一轮 | `PublishedVersion` 加实例字段、`newest_persisted_root_txg` 返回 `(实例, txg)` 并按 (txg, 实例) 字典序取 max、`oracle_violation_for_versions` 第 563 行别再用 `_` 丢掉实例。会红的用例就是第四节那两行探针做成断言。**改之前这一条不进步 0**：步 0 的脚本含切换与回退，用今天这个 oracle 跑出来的「零违例」证明不了什么 |
| **X5-①** M1 / M3 只有验收断言 | 补对照 | 释放代：按 milestone 自己列的两条出路二选一（立一条「释放代不小于同落点的分配代、且不晚于所选根」的空间记账不变量，或删掉这条变异并写明为什么）。树表诞生 txg：同型，一并定 |
| **X5-②** 映射那一格 | 补对照 | 先把「释放判定路径」建出来（哪怕第一版只是一个查映射的函数），再谈它的对照；在此之前 milestone 步 2 验收标准里那两条**不许记成「全过」** |

## 八、这条腿自己的口径与它答不了的

- **全部数字都在副本上、debug 构建**。层 0 全量那条 524312 状态的用例带 `#[ignore]`，我没跑；我的结论没有一条依赖它。
- **变异一次只改一处**，改完跑 `--no-fail-fast`、跑完还原；每条变异的完整输出留在 scratchpad 的 `out-<标签>.txt`，随会话目录一起消失，要入库得按跑前登记在入库装置上重做。
- **我没有攻 X3 / X6 / X7**（分工里不是我的格），第六节那两条是核 X1 / X5 时顺手撞见的，交给正推腿复核。
- **第四节那条的射程我自己收窄过**：它在 B 这条流上**不可达**，我没有构造出一个今天就判错的状态；打中的是 oracle 的形态与它在步 0 上的承重，不是今天的某个数。
- **第三节第 2 条（第 50 轮 panic）我只在单进程、同一个 `PoolAllocator` 上跑到**。挂载后分配器怎么重建今天没有代码（第六节第 1 条），所以「跨挂载也是第 50 轮」这句我说不出来，没说。
