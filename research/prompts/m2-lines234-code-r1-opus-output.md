# m2-lines234-code-r1　云端攻方（Opus）：X1 / X2 / X6

立场：找代码与条款的反话，以及没被钉住的选择。攻击面 X1、X2、X6（与本地攻方的「22 个文件的测试覆盖」不重叠）。
时刻：2026-09-22 23:5x–2026-09-23 0x:xx UTC（= JST +9）。前几轮判决：无（第一轮）。

## 开工快照核对

`sha256sum -c research/prompts/m2-lines234-code-r1-snapshot/opening.sha256` → **22 行全 OK**，
所以本报告里 `crates/` 的行号一律是**主树行号**，不用 diff 里的形态。

## 各格判定一览

| 格 | 判定 | 打中的那一句 | 拿什么钉着（今天） |
|---|---|---|---|
| **X1** 挂载态整片读中央映射 | **替没写的条款做了选择**（两处），其中一处**守的是走不到的那一侧** | ① 「映射树根是根兼叶」这条前提，写侧**没有任何准入**钉着：同一个函数 `admission_of_one_publish` 给分配记录树、记账树各写了一条容量准入，映射树没有；② 读侧那条 `level != 0` 的拒绝**从今天的写侧走不到**——写侧把 level 写死 0、没有分裂逻辑，条目超了走到的是 `build_index_node` 的断言 panic，不是「长出第二层」 | 只有一条用例钉「伪造的多层映射根被拒」；余量 294 − 140 = 154 是两个互不相干的常量的巧合，没有任何东西钉住这个关系 |
| **X2** 树节点那一侧没有经映射的回退 | **替没写的条款做了选择**，而且这个选择让**同一份字节**在数据单元与树节点上得出相反的结局 | 同一种损伤（位置提示指一个空槽、映射里是真落点）：数据单元 ⇒ 多跳一次、读回正确字节；extent 树根 ⇒ `open_pool_for_read` 返回 `Walk(UnitUnreadable { slot: SlotNumber(54286) })`，**整个池挂不上**。D19 已定项 5 的字面对两者不加区分 | 无。C483 只登记了「没有条款」，没有会红的东西；checker 那一侧同样把过期提示当损坏（推的） |
| **X6** mkfs 整环清零走的块设备动作与段序列的步骤种类 | **和条款说反话**（第六种那一格）＋ **替没写的条款做了选择**（「按动作归类」与「清零算一个整写」两处） | D17 已定项 2 第 1 条逐字写「步骤种类**集合**相同（写单元 / 写 journal 记录 / 根槽 FUA 写 / 系统配置槽原地覆写 / 屏障**五种**）」；活代码与 `layout/01-first-txn.md` 八都写**六种**。两种读法在**门禁 52 号逐字比的那一串上**差出来，我量到了两串（见 X6 第 2 节） | 第六种本身有用户 2026-09-19 定案撑着；**没写回 D17 已定项 2**。「按动作归类不看落点」与「清零算一次整写」两处一条条款都没有 |

**判据自检（`evidence-discipline.md`「判据自己也会写错」四句）**：见本报告末尾「四句自检」一节，三格逐格答。

## 复跑：装置与命令

仓副本：`/tmp/claude-1000/lines234-opus/repo`（`rsync -a --exclude target --exclude .git`），
`CARGO_TARGET_DIR=/tmp/claude-1000/lines234-opus/target`。**副本上量出来的数不算入库装置上的数**。

开跑前负载（2026-09-22 23:53 UTC 现查）：`ps -o pid,args -u "$(id -u)"` 里没有 `qemu-system` / `vm-bench.sh` /
`e152-file-system-benchmark` / `fio` / `cargo` / `gate.sh`；`uptime` 报 `load average: 1.58, 10.32, 14.25`，`nproc` 32。
全部 `cargo` 加 `nice -n 19 -j 6`，没等过锁。

两份改动（副本上的，未入主树）：

| 文件 | 作用 | sha256 |
|---|---|---|
| `/tmp/claude-1000/lines234-opus/artifacts/x1-x2-experiment.patch`（144 行） | 给合成镜像补上「extent 树根也进中央映射」（真发布路径本来就这么写），加一个损伤档与两条用例 | `9eee1e6cb2bcaba8e414f1240b1ff73aa2607d8018df1142563e50e0f214c287` |
| `/tmp/claude-1000/lines234-opus/artifacts/x6-five-kind-reading.patch`（9 行） | 把 `classify` 改成 D17 已定项 2 第 1 条那五种的读法（清零按落点归类），量另一串 | `f192441c904938e9fdf48ef5207de97080a473de8b473578ca5d4307652a54cd` |

```
cd /tmp/claude-1000/lines234-opus/repo
patch -p1 < ../artifacts/x1-x2-experiment.patch
CARGO_TARGET_DIR=/tmp/claude-1000/lines234-opus/target nice -n 19 cargo test -j 6 -p singlefs-harness \
  --test second_transaction_parallel_line_two_mounted_read -- --nocapture opus_
# X6 第 1 串（活代码）
CARGO_TARGET_DIR=... nice -n 19 cargo test -j 6 -p singlefs-harness --test first_transaction_step_one_mkfs \
  -- --nocapture recorded_stream_matches_the_registered_mkfs_segment_sequence
# X6 第 2 串（五种读法）
patch -p1 < ../artifacts/x6-five-kind-reading.patch && 同上一条
```

⚠️ 这两份改动**只在副本上**，本轮没有入库产物；主 agent 要留就从上面两个 `/tmp` 路径拷。
现在不入库的理由：它们是攻方腿自己的实验装置，按 `three-way-inference.md`「判决由主 agent 做」一节，
副本上的数要主 agent 在入库装置上重做一次才能引。

---

## X1　挂载态整片读中央映射

### 1. 被判的行为，逐处现查

- `crates/singlefs-core/src/mounted_read.rs:347–354`：`open_pool_for_read` 读一次映射树根，`for entry_bytes in &mapping_root.entries` 把**整片**解进 `central_mapping_entries`。
- 同文件 `614`：`fn central_mapping_lookup(&self, key: &[u8]) -> Option<[LocationEntry; 2]>` —— 签名里没有 `reader`，查映射发不出设备读。
- 同文件 `338–344`：`if mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL { return Err(...TreeLevelUnexpected...) }`，常量在 `60`：`const CENTRAL_MAPPING_TREE_ROOT_LEVEL: u8 = 0;`。
- 读数 3 的出处：`crates/singlefs-harness/tests/second_transaction_parallel_line_two_mounted_read.rs:897–909` 断言 `device_reads_issued: 3`。

### 2. 这一格我打中的：那条拒绝守的是**走不到的那一侧**

主 agent 在正文第三节写「今天由『映射树根不是根兼叶就拒绝打开挂载态』挡着」。**挡不住，因为它走不到。**

**(a) 写侧把层级写死 0，没有分裂那条路。** `crates/singlefs-core/src/transaction.rs:2438–2452`，
`build_index_node(TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING), 0, ...)` —— 第二个实参是层级，字面常量 `0`。
全仓只有这一处写映射树根（`grep -n 'TREE_IDENTIFIER_CENTRAL_MAPPING' crates/singlefs-core/src/transaction.rs` 命中 749 / 976 / 2434 / 2439 / 2455，只有 2439 那一处建节点）。
⇒ **今天的写侧造不出层级 1 的映射根**，读侧那条拒绝只对伪造镜像成立（用例
`a_central_mapping_root_that_is_an_internal_node_is_refused_instead_of_being_read_as_entries` 喂的正是伪造镜像）。

**(b) 真正该有的那道准入没有。** `crates/singlefs-core/src/transaction.rs:1782–1815` 的 `admission_of_one_publish`
逐条判了两棵「第一版只有一个节点」的树：
`1791` `let allocation_node_capacity = index_node_entry_capacity(...ALLOCATION_RECORD_KEY_BYTES..., ...ALLOCATION_RECORD_BYTES...)`，
`1804` `let accounting_node_capacity = index_node_entry_capacity(...ACCOUNTING_KEY_BYTES..., ...ACCOUNTING_ENTRY_BYTES...)`。
**映射树没有这一条**（`grep -rn 'index_node_entry_capacity' crates/` 七处命中里没有映射）。
装不下时走到的是 `crates/singlefs-core/src/unit.rs:160–163` 的断言，它的原样文字是
「条目装不进一个节点：写者要先按 index_node_entry_capacity 判、报错，不许走到这里」——
这句话逐字承认「写者要先判」，而映射的写者没判。

**(c) 量出来的数（副本，`opus_x1_...` 用例的 `--nocapture` 原样输出）**：

```
X1 容量：映射(27,55)=294 inode根(8,120)=135 分配记录(10,20)=812 记账(22,34)=477
X1 映射条目上界 = 5 + K，K ≤ inode 根扇出 135 ⇒ 上界 140，余量 154
thread 'opus_x1_the_central_mapping_node_has_no_capacity_admission_while_two_sister_trees_do' (1248909) panicked at crates/singlefs-core/src/unit.rs:160:5:
条目装不进一个节点：写者要先按 index_node_entry_capacity 判、报错，不许走到这里
X1：映射条目到第 295 条时 build_index_node 断言 panic
```

「5 + K」的出处是 `crates/singlefs-core/src/transaction.rs:2385–2423`：一次发布的映射条目 =
数据单元 1 + extent 根 1 + inode 叶容器 K 片 + inode 根 1 + 分配记录树 1 + 记账树 1。
K 的上界是 inode 根的扇出，`crates/singlefs-core/src/inode_tree.rs:100–103` 的
`pub fn leaf_containers_one_root_node_holds() -> usize { index_node_entry_capacity(8, ...INODE_INTERNAL_ENTRY...) }` = 135。

⇒ **今天不翻车，靠的是 140 ≤ 294 这个数字巧合**，而这两个数分属两棵没有关系的树，没有任何检查、断言或不变量钉住它们的关系。
D8 已定项 6 改 inode 内部条目宽、D19 已定项 10 改映射条目宽、或者 D16 已定项 5 那句
「一个事务最多写一个单元的用户数据」被放开（D19 已定项 6 的射程第 127 行自己写着那句「最好写进那一句」，说明它还在动），
三样里任何一样动了，余量就变；而变到 0 的那天，红的不是门禁，是发布路径里的一次 panic。

### 3. 能看出差异的历史与读数（能跑的形态）

**历史 H1（今天就跑得了，跑过）**：写侧的溢出落点。
写序列：不需要盘——直接对写侧那一步取样。`build_index_node(映射树, level=0, key宽 27, 条目宽 55, 295 条条目)`。
故障点：无（不是崩溃，是写路径自己）。期望读数：**进程 panic**，消息逐字为 `unit.rs:160` 那一句。
实测：见上面的 `--nocapture` 输出。**这条打中的是「拒绝够不够」那一问的前提**——不够，因为在它生效之前写侧先挂了。

**历史 H2（今天跑不了，缺装置）**：真正走到「映射长成多层」。
写序列：连发 K 次发布，让 inode 叶容器数超过 289 片（= 294 − 5）。
缺哪个装置：① 写侧没有映射树的分裂逻辑（写死 level 0）；② inode 根在 K = 136 片时先拒
（`inode_tree.rs` 的 `InodeTreeWriteRefusal`），**够不到 289**。
⇒ 这条历史在今天的代码上**不可达**，我照实记「没打中」，不拿它撑结论。

### 4. 特别要答的那一问：映射长成多层之后怎么办，今天那条拒绝够不够

**不够，而且不够的方式和正文预想的不一样。** 分三句：

1. **它今天走不到**（上面 2(a)）。它是给伪造镜像准备的一道解析纪律（不把内部条目当 55 字节映射条目解），
   那个作用它做到了；把它当成「整片读这条前提的守卫」是记错了账——守卫在写侧，而写侧没有守卫。
2. **等它走得到的那天，它的语义是「整个池挂不上」**，不是「退回到现场读」。
   `mounted_read.rs:338` 返回 `OpenPoolForReadFailure::TreeLevelUnexpected` ⇒ `mount_read_only` 整个失败。
   D19 已定项 5 定案表里「中央映射」那一格逐字是「**解引用与释放判定的唯一入口**。搬迁只改这里一条条目，
   『谁引用我』根本不用知道」——唯一入口在长大之后变成「一个文件都读不了」，是把可用性悬崖装在唯一入口上。
3. **`device_reads_issued = 3` 这个读数今天没有任何东西标注它的适用域。**
   D17 已定项 5 射程 ① 要的是「一次 API 调用发了几个设备级操作」的真值。多层之后真值是 3 + 映射树高，
   而代码里连那个分支都没有；用例 `second_transaction_parallel_line_two_mounted_read.rs:905–908` 的断言消息
   里写着「映射长到一个节点装不下、要按需走树的那天这个数会涨」——**这句话写在注释里，不在任何会红的东西里**。

（2 的 H2 里那句「inode 根在 K = 136 片时先拒」现查坐实：`crates/singlefs-core/src/inode_tree.rs:236–240`
的 `let capacity = leaf_containers_one_root_node_holds();` 加
`InodeTreeWriteRefusal::MoreLeafContainersThanOneRootNodeHolds`，同文件 `545` / `550` 两行单测钉住「扇出 135」「第 136 片容器今天装不下」。）

---

## X2　树节点那一侧没有经映射的回退

### 1. 被判的行为

`crates/singlefs-core/src/mounted_read.rs:310–311` 自陈（原样整行抄）：

> /// **树节点今天只按位置提示读，没有经映射的回退**：与冷启动走读（`recovery::walk_to_file`）今天同款，

代码上：`330`（映射根）、`358`（树表）、`384`（extent 根）、`412`（inode 根）、`435`（inode 叶容器）
五处走的都是 `read_tree_root` / `read_unit_via_locations`，一次 `central_mapping_lookup` 都不叫；
只有 `746–797` 的 `dereference_data_unit` 那一条路有回退。

### 2. 打中：同一份损伤，数据单元与树节点结局相反（量出来的）

正文问的是「判它是哪条分项的后果，还是两处都在替一条没写的条款做选择」。
两处都在替一条没写的条款做选择，而且**这个选择的代价能在字节上量出来**。

**装置**：`second_transaction_parallel_line_two_mounted_read.rs` 那份合成镜像。它原来漏了一件真发布路径会做的事——
把 extent 树根也写进中央映射（`transaction.rs:2394–2398`：
`(TransactionUnit::ExtentRoot, mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer), extent_pointer.locations)`）。
我在副本上按真写侧补上这一条，然后把**树表条目里 extent 根的位置提示**换成那个从来没写过的空槽
（与已有用例 `PointTheHintOfThisUnitAtAnEmptySlot` 对数据单元做的是同一种损伤、同一个空槽号）。

**写序列 + 故障点 + 期望读数**（能跑，跑过）：

| | 数据单元（已有用例，2026-09-22 就在仓里） | extent 树根（这一轮加的） |
|---|---|---|
| 字节状态 | 父指针的两条位置条目指空槽；映射条目指真落点 | 树表条目的两条位置条目指空槽；映射条目指真落点 |
| 结局 | `read_at` **成功**，字节与写入逐字节相同 | `open_pool_for_read` **失败** |
| 读数 | `ReadPathObservation { data_units_read: 1, data_unit_dereferences: 1, stale_location_hint_hops: 1, device_reads_issued: 3 }` | `Walk(UnitUnreadable { slot: SlotNumber(54286) })` |

副本上 `--nocapture` 的原样输出：

```
X2 open_pool_for_read 的返回：Walk(UnitUnreadable { slot: SlotNumber(54286) })
test opus_x2_a_stale_hint_on_a_tree_node_makes_the_whole_mount_fail_although_the_mapping_is_right ... ok
```

整份用例文件补上那条映射条目之后 **13 条全绿**（含原有 11 条），所以这份镜像除了那一处提示之外是自洽的：

```
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s
```

### 3. 它与条款的关系

D19 已定项 5 的定案表两行（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:93–94`，原样整行抄）：

> | 位置条目（已定项 4 的 14 字节） | **降为提示**。宽度与段划分不变，变的是它的地位：读到它可以直接去那个落点试，但它可能过期 |
> | 中央映射 | **解引用与释放判定的唯一入口**。搬迁只改这里一条条目，「谁引用我」根本不用知道 |

不进映射的**只有三类**，同文件 `165` 逐字：

> **不进映射的三类（自举豁免）**：映射树自己的节点、树表单元、实例表单元。它们的父指针（映射树的根、根记录里的树表指针与实例表指针）里的位置条目是权威，不是提示，已定项 5 的「唯一入口」与硬规则 1 对这三类不适用；三类都是池级单份、引用者只有一个。

⇒ 代码对**映射根与树表**不回退，与这一句**相符**（它们就在豁免三类里）。
对 **extent 树根、inode 树根、inode 叶容器**不回退，条款里找不到许可它的那一句：
这三类不在豁免表里，按定案表第一行它们的位置条目是「可能过期」的提示，按第二行映射是唯一入口。
不过这还不算「和条款说反话」（判据第 2 条的门槛）：条款没有一句写「解引用不到要去查映射」这个**动作**，
只写了地位。地位与动作之间少的那一句，正是 C483 登记的第 ② 条。
⇒ 判 **替没写的条款做了选择**，而不是说反话。两个都说得通的选择、它们分得开：
「树节点也回退」⇒ 上表右列变成读得到；「树节点不回退」⇒ 上表右列是整池挂不上。分得开，按判据第 3 条该立条款。

### 4. 今天有没有会红的东西钉着（没有）

- 用例：`grep -n 'LocationHintDamage::' crates/singlefs-harness/tests/second_transaction_parallel_line_two_mounted_read.rs`
  在主树上命中 4 行（239、287、288、873），损伤档只有 `None` 与 `PointTheHintOfThisUnitAtAnEmptySlot`，
  后者的参数是 `DataUnitIndexInFile` ⇒ **没有一条**用例喂得出「树节点提示过期」这个形状。
- checker：`crates/singlefs-checker/src/walk.rs:458` 的 `fn walk_central_mapping_entries` 只逐条读「映射条目指的单元」，
  **不与父指针里的提示比**；而 `fn walk_tree_table_entry`（同文件 `484`）按提示读 extent / inode 根。
  ⇒ 我这份镜像在 checker 眼里**照样判红**，红在「按提示读不到」这一侧（**推的**：没跑——
  合成镜像的根记录只住内存、不进根环，`check_pool_image` 吃的是整块盘镜像，这一条没有装置）。
  这不是好消息：按 D19 已定项 5 的字面，提示过期**不是损坏**，而 checker 与挂载路径今天把它一起当损坏。
- C483（`.claude/kb/checks-owed.md:428`）登记了这一格没有条款，**「怎么拦」那一格写的是「条款定下之后」才有检查**，
  即今天一个会红的东西都没有。

### 5. 可达性，照实说

上表右列那份字节状态**不是今天的写路径造得出来的**：真发布路径每次把提示与映射条目写同一个落点，
提示要过期得有搬迁（D26 后台整理），仓里没实现。
⇒ 我打中的是**字节**那一侧（判据第 2、3 条都写「哪个字节**或**哪条可达历史」），不是可达历史那一侧。
这一格的结论因此是「今天不咬人、条款一写就咬人」：等 D26 的搬迁落地，这一格从「没有条款」直接变成「数据读不到」。

---

## X6　mkfs 整环清零走的块设备动作与段序列的步骤种类

### 1. 今天怎么落的（现查）

- 块设备抽象：`crates/singlefs-core/src/block_device.rs:105` 的注释逐字是「/// **六个动作，只有这六个**。介质是什么这里看不见。」
  （正文第三节写「今天五个动作」——那是改动前的数；trait 今天的六个成员是
  `read_at` `107`、`write_at` `112`、`write_zeroes_at` `127`、`barrier` `133`、`probe_physical_block_size` `134`、`size_in_bytes` `135`。
  FUA 不是第七个动作，它是 `write_at` 的 `durability` 参数。）
- mkfs 的调用点：`crates/singlefs-core/src/make_filesystem.rs:258–263`，**每块盘四次**——根环三个区域各一次
  （`259–260` 的 `for region_to_clear in 0..ROOT_RING_REGIONS { device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?; }`）
  加 journal 环一次（`262`）。两块盘 ⇒ 八次。
  ⚠️ `block_device.rs:125` 那句「唯一的调用方是 mkfs 的整环清零」与这四个调用点**不符**：四个里只有一个是环，三个是根环区域。
- 登记表那一侧：`crates/singlefs-harness/src/segments.rs:23` 加了 `ZeroFill` 进枚举，`57` 的
  `RecordedOperationKind::WriteZeroes => StepKind::ZeroFill` 按**动作**归类，`106` 把它并进「普通写那一档」（不关段、段里算一个写）。

### 2. 打中：D17 已定项 2 第 1 条逐字说五种，活代码与登记表说六种，两种读法量出两串不同的字节

`.claude/kb/decisions/17-实现分层与第三方管道.md:38`（原样整行抄，就是已定项 2 第 1 条）：

> 1. 结构等价类按录制流的段序列分：两条布局线同类 ⟺ 对每一条根槽写路径（发布、空发布、mkfs 种子、实例切换 / 管理员回退——D13（验证路线） 已定项 1 列的三种写者加空发布），两条线录制流的段序列同构：段边界（屏障与 FUA 写切出的段，D13（验证路线） 已定项 4）的位置相同、每段里出现的步骤种类**集合**相同（写单元 / 写 journal 记录 / 根槽 FUA 写 / 系统配置槽原地覆写 / 屏障五种）；一步重复几次（设备数、单元数、副本数）是参数不进判据（D13（验证路线） 已定项 10）。比较只在同一路径角色、跨线之间做，一条线内部各路径不互比、不计数。

`.claude/kb/layout/01-first-txn.md:384`（同一件事的另一处登记位；D17 已定项 2 第 3 条逐字写「登记位只有一处：layout/01-first-txn.md 八」）里那一句是：

> 步骤种类**六种**：写单元（含码 3 容器、实例表单元）、写 journal 记录、根槽 FUA 写、系统配置槽原地覆写、屏障、**整段清零**（`zero_fill`，2026-09-22 随 mkfs 清根环与 journal 环加，用户 2026-09-19 定「录制流登记成一种新步骤、层 0 认它」）。

**两处登记位一个说五种、一个说六种。** 判据第 2 条要「具体是哪个字节或哪条可达历史上看得出差异」，我量了：
同一次 mkfs 的录制流，两种读法下门禁 52 号逐字比的那一串 `kinds=`**不同**（副本上跑 `recorded_stream_matches_the_registered_mkfs_segment_sequence`，原样输出）：

```
  left: "[unit_write×4,journal_record×2,root_record_fua×6,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]"
 right: "[zero_fill×8,unit_write×4,barrier]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier]"
```

`right` 是活代码今天报的（六种读法，与 `layout/01-first-txn.md:390` 登记的逐字相同，主树上跑这条用例是绿的）；
`left` 是把 `classify` 改成「五种、清零按落点归类」之后同一次 mkfs 报出来的。
第一段的**步骤种类集合**因此从 `{zero_fill, unit_write, barrier}` 变成
`{unit_write, journal_record, root_record_fua, barrier}`——而那个集合正是 D17 已定项 2 第 1 条**判等价类同构用的东西**。
⇒ **判「和条款说反话」**：满足的是判据第 2 条，差异落在门禁 52 号逐字比对的那一串字节上。

归错判据的可能性我核过（见末尾「四句自检」）：这一格也可以读成「条款那个括号是当时的描述、不是规范」。
两种读法我都写出来了，**但无论取哪一种，D17 已定项 2 的正文今天都没有被改**，而改动本身有用户定案撑着
（`records/2026-09-19-里程碑二遗留收拢.md:148`，原样整行抄：
`| 4 | 块设备抽象加不加「写零」 | 加 | 并行线四：块设备抽象加写零动作，录制流登记成一种新步骤、层 0 认它 |`）。
⇒ 第六种**不是**「又一次替条款做选择」，它是一条**已定而没写回 D17 已定项 2** 的决定。
主 agent 要还的是「写回」，不是「重判」。

### 3. 第六种里面，还有两个没人定的选择

**(a)「整段清零按动作归类、不看落点」**（`segments.rs:8–9` 自陈：
「整段清零按**动作**归类、不看落点：它在块层就是另一个命令（真设备上映射成 WRITE ZEROES），不是『写在某处』的一次写」）。
用户答复那一行只说「登记成一种新步骤」，没说按动作还是按落点。两个选择都说得通、分得开：
上面 2 节那两串就是它们分出来的。今天的代码在同一张表里**混了两套归类法**——
普通写按落点（`segments.rs:64–74`，按落点分的四支），清零按动作（`57`）。
后果在等价类数上：以后开第二条布局线，它的 mkfs 若拿普通写清环（后端没有 WRITE ZEROES），
六种表下第一段种类集合不同 ⇒ 不同构 ⇒ 等价类 +1 ⇒ 门禁挂钟按 D17 已定项 2 第 2 条乘 2；
五种表下同构 ⇒ 仍是 1。这正是 C8（门禁范围判不出来，D17 已定项 2 的「欠」那一行）在乘的那个数。
**今天有没有会红的东西钉着这个选择：没有。**

**(b)「清零算一次整写」**（`segments.rs:105–108`：「清零是普通写那一档：不做持久、不关段，段里多算一个写」）。
它把 mkfs 第一段从 4 个写变成 12 个写，闭式崩溃状态数
`1 + (2^12 − 1) + 3×(2^1 − 1) + (2^4 − 1) = 4114`，正是 `layout/01-first-txn.md:390` 登记的数。
D13 已定项 4（`.claude/kb/decisions/13-验证路线.md:71`，原样整行抄）：

> **定案**：崩溃点重放要枚举的崩溃状态集合定义为：屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写也是段边界：它自成一段，它之后的写不与它同段。撕裂子集**不枚举**，一次写的任何撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像要真的生成出来喂给 checker，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。

它把撕裂并进「没持久」，理由写在同一格的射程（同文件 `73`）里：「撕裂态与『没持久』的差别只有 CRC32C 的碰撞概率」。
**这条理由对清零不成立**：一次 768 MiB 的清零底下拆成 192 次 4 MiB 写
（`block_device.rs:103` 的 `pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;`，`313` / `533` 两个后端各按它循环），
而被清的那一段**没有任何校验和罩着**——半清的 journal 环里剩下的那一半是上一个池**真合法**的记录。
这句话不是我推的，是 mkfs 自己的注释说的（`make_filesystem.rs:238–239`，两行原样整行抄）：

> // 为什么 mkfs 自己做：一条 journal 记录、一条根记录合不合法只看它自己的魔数与校验和，盘上的旧字节里
> // 凑出一条合法记录的概率不是 0（上一次 mkfs 留下的就是真合法的记录），而「这块盘是新建的全零镜像」

⇒ 把清零当「一次整写」，等于宣称「清了 100/192 块」与「一块没清」在判定上同一件事，而 mkfs 之所以要清，
正是因为这两件事不同。这一格今天**不咬人**，因为 mkfs 的崩溃状态不进层 0
（D17 已定项 2 射程同文件 `44`「mkfs 的崩溃状态仍不在层 0 枚举里」，`layout/01-first-txn.md:390` 那一行「层 0 枚举」格写「**不在**」）。
但它与用户答复里那句「**层 0 认它**」是两个读法，而 4114 这个数已经登记进 kb 了。

---

## 没打中的形状（试过、取样范围、为什么不算）

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| X1：造一条历史让映射真的长成两层 | 写侧全部建映射根的位置（`transaction.rs` 里 `TREE_IDENTIFIER_CENTRAL_MAPPING` 五处命中：749 / 976 / 2434 / 2439 / 2455，只有 2439 建节点） | **走不到**：层级是字面常量 0，没有分裂那条路；条目要到 295 条才溢出，而上界是 140 |
| X1：靠堆 inode 叶容器把映射撑爆 | K 从 1 扫到 136 | **走不到**：`inode_tree.rs:236–240` 在 K = 136 时先拒（`MoreLeafContainersThanOneRootNodeHolds`），够不到 289 |
| X6：让「设备数进判据」在盘上现形（D17 已定项 2 第 1 条逐字写「设备数……是参数不进判据」，而登记串里写着 `zero_fill×8`、`unit_write×4`，都是设备数 × 常数） | 把 `run_mkfs` 的盘数从 2 改成 3，跑同一条用例 | **走不到**：`crates/singlefs-core/src/make_filesystem.rs:188` 的断言当场拦下，原样输出 `assertion `left == right` failed: 第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）  left: 3  right: 2`。⇒ 今天那几个 `×N` 是常数，这条不算打中 |
| X2：让树节点的过期提示由**真发布路径**造出来 | 翻了 `transaction.rs` 全部写映射条目的地方（`2385–2423`） | **走不到**：每次发布把提示与映射条目写同一个落点；要搬迁（D26），仓里没实现。我只打中了字节那一侧 |
| X2：拿 checker 坐实「同一份字节，checker 绿 / 挂载红」 | — | **没跑**：合成镜像的根记录只住内存、不进根环，`check_pool_image` 吃的是整块盘镜像，缺这个装置。报告里那一句标了「推的」 |
| X1：映射条目 key 重复 / 乱序时 `central_mapping_lookup` 取第一条（线性扫，`mounted_read.rs:615–618`） | 看了写侧（`transaction.rs:2432` 的 `sort_by_key(mapping_key_sort_key)`）与读侧（`open_pool_for_read` 对映射根**不判 key 升序、不判重**） | **不算打中**：要造出重复 key 得伪造镜像，而 checker 那一侧有 `check_index_node_keys`（`walk.rs:383`）判 key 区间与条目；二分与线性两种实现只在 checker 判红的镜像上分得开。按判据第 3 条「分不开的不立条款，登记成实现细节」处理 |

## 这条腿自己的限度

1. **一次抽样。** `three-way-inference.md`「一条腿只抽一次样不算一次观测」那一节：上面四条「没打中」
   **一次不算**，要再抽一次才记得住；「打中」那三条是线索，真伪由主 agent 现查坐实。
2. **副本上的数。** X1 的容量、X2 的 `Walk(UnitUnreadable { slot: SlotNumber(54286) })`、X6 的两串 `kinds`
   都在 `/tmp/claude-1000/lines234-opus/repo` 上量的，**不进 kb**；要引，主 agent 按跑前登记在入库装置上重做一次。
3. **我自己提的改法被攻过零轮**（下一节标了每个改法修哪一格）。
4. **没判的**：X3 / X4 / X5 归云端正推腿，22 个文件的测试覆盖归本地攻方腿，我一个字都没判。
   `bad_disk_input.rs`（2168 行）、`read_tally.rs`、三个 `bin/` 我只在 X1–X2 用得到的地方读过，没有整份判。
5. **门禁**：`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-attack` 的阶段
   （`awk -F'\t' -v me=three-way-attack '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv` 无输出），所以一个都没跑。
6. **一行主树代码都没改**，一次 `git` 写操作都没做。

## 我提的改法（每个标明修哪一格；**只在我的模型上量过、被攻过零轮**）

| 改法 | 修哪一格 | 在打中的格上还中不中 |
|---|---|---|
| A. 给 `admission_of_one_publish` 加第三条：映射条目数 > `index_node_entry_capacity(27, 55)` 时返回 `PublishError::MappingEntriesExceedOneNode`（与已有两条同形） | X1(b) | 修掉「写侧 panic」那一半；**X1(a)「整片读没有条款」照样中**——准入只是把 panic 换成错误，没说整片读对不对 |
| B. 把 `device_reads_issued` 的适用域写进条款：「映射根兼叶时该数 = 提示试的次数 + 映射落点试的次数」，并给它一条会红的检查（层级 ≠ 0 的镜像上断言拒绝而不是断言 3） | X1 第 4 节第 3 句 | 不修 X1(b)，也不修 X2 |
| C. 树节点解引用加经映射的回退（与 `dereference_data_unit` 同形），自举豁免三类除外 | X2 | 修 X2；**对 X1 与 X6 不起作用** |
| D. 把第六种写回 D17 已定项 2 第 1 条的括号，并在那一句里写明「一种步骤按动作定还是按落点定」 | X6 第 2 节与 3(a) | 不修 3(b)「清零算一次整写」 |
| E. 给 D13 已定项 4 的射程补一句：「没有校验和罩着的整写（今天只有 `zero_fill`），撕裂态不并进『没持久』」，或者反过来写明为什么可以并 | X6 3(b) | 不修 X6 第 2 节那两串的差异 |

## 四句自检（`evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）

| 问 | X1 | X2 | X6 |
|---|---|---|---|
| **分不分辨臂？** | 分。「整片读 / 现场读」「有准入 / 无准入」是两组独立的臂，打中的只压「无准入」那一侧，不把两组一起打掉 | 分。「树节点回退 / 不回退」两臂，打中只压「不回退」 | **第 2 节那一格不分辨臂**：五种与六种都被打中（一个是条款、一个是代码，两边一起错在「没写回」）。⇒ 按那一节的处置，**不拿它判臂**，把「写回 D17」另立一笔账；判臂只看 3(a)、3(b) 那两格 |
| **被判的系统当时看得到判别它的东西吗？** | 看得到：条目数在发布装节点之前就算出来了（`transaction.rs:2428`），准入判得了 | 看得到：映射整片已经在挂载态里（`open_pool_for_read` 解完才走树节点），回退要的 key 与 `reader` 当场都在手上 | 看得到：`RecordedOperationKind` 与落点在 `classify` 的入参里，两种归类法都判得出来 |
| **满足的是判据字面的哪一个分句？** | 第 3 条（两个都说得通的选择、在可达历史上分得开：一边发布报错、一边进程 panic） | 第 3 条（两个选择在**字节**上分得开：同一份镜像一边读得到、一边挂不上） | 第 2 条（「具体是哪个字节」：门禁 52 号逐字比的那一串，两种读法量出两串）。**不是**第 3 条——第六种有用户定案，不是没人定的选择 |
| **跑前条款给的每个改法在打中的格上还中不中？** | 见上表：A 修 X1(b)，X1(a) 照样中 | C 修 X2，对 X1 / X6 不起作用 | D 修第 2 节与 3(a)，3(b) 照样中；E 修 3(b)，第 2 节照样中 |
