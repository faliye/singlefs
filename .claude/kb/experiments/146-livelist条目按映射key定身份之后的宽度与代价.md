## E146 livelist 条目按映射 key 定身份之后的宽度与代价 —— 已跑（2026-09-13，计数模型，8 单测 / 12 条变异全抓）

问的是 D6（快照实现模型） 已定项 2 ①：livelist 条目的 key 与宽度。2026-09-13 从已定条款推出的约束是身份必须用中央映射的 key（D19（块指针的结构与宽度预算） 已定项 5 规则 1「释放一律经映射，不经提示」），
共享树的 key 以头的树 ID 8 字节打头（D6（快照实现模型） 已定项 3），后接映射 key（D19（块指针的结构与宽度预算） 已定项 6：码 1 27 字节、码 2 / 码 3 25 字节）；
E130（每头一份 livelist 过不过有界销毁） 用的 24 字节形态随 C266（livelist 条目的身份用了会过期的位置提示） 作废，C260（livelist 条目的预付没算进解开自己那个量） 的每次 FREE 预付量也跟着定不下。
这里把「按映射 key 定身份」的几种装法逐个算成字节、扇出、树高、预付量与第一个事务的字节差，**只报数不判输赢**；取哪一种是 D6（快照实现模型） 已定项 2 的定案。

**结果一句话**：补齐到最宽、事件类型折进类标签的形态条目 35 字节（叶扇出 463、内部 137），每次 FREE 预付 35 字节 = 一个 32 KiB 单元的 10 个万分点；
带事件字节 36（叶 450）；分两棵树码 2 / 3 那棵 33（叶 491）但与 D6（快照实现模型） 已定项 3「一棵共享树」字面不合；value 带 14 字节位置提示则 49（叶 330）。
E132（livelist 载体·按真实树数与内部扇出重算） 的 21 个规模格上，35 字节形态只在两格比作废的 24 字节形态高一层（总条目 65536 那两格：高 3 对 2）。day-1 注册共享树在第一个事务上多 200 字节（树表一条条目，装进第 1 版树表单元），空树若要根节点再加 16384 + 56 + 53 = 16693；惰性创建 0 字节。

**复跑命令**（`exact` 模式，与留存产物逐字节比对）：

```bash
cd research && bash scripts/replay.sh E146
# 直接跑装置：
cd research && cargo run --release --bin e146-livelist-entry-width
```

代码 `research/e7-index-bench/src/bin/e146_livelist_entry_width.rs`，原始输出 `research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out`（130 行，末行 `emitted=130`；树表条目按 200 算的那一版，上一版 `research/results/e146-livelist-entry-width-2026-09-14-round2.out` 按 148 算、与它只差 6 行（`name=config` 1 行与 5 行 `name=first_transaction`），再上一版 `research/results/e146-livelist-entry-width-2026-09-13.out` 按 145 算，两版都原样保留；变异复跑 `research/results/e146_livelist_entry_width-mutate-2026-09-16-tree-table-200.log`，12 条全抓），
跑前登记 `research/prompts/e146-preregistration.md`，变异表 `research/mutations/e146_livelist_entry_width.tsv`（12 条全抓，
`bash research/scripts/mutate.sh e146-livelist-entry-width research/e7-index-bench/src/bin/e146_livelist_entry_width.rs research/mutations/e146_livelist_entry_width.tsv`）。

### 这是计数模型，不是实现

没有 I/O、没有随机源 ⇒ 跑 N 遍必然逐字节一致；证据强度来自 8 个单测 + 12 条变异全部被抓，其中扇出、头宽、树高、预付、第一个事务字节都各有一条按跑前登记的手算钉死绝对值的断言。

### 臂（条目形态）

| 臂 | key | value | 树 |
|---|---|---|---|
| `legacy_e130` | 24（类型标签 2 + 位置条目 14 + birth txg 8，作废形态，只做对照） | 0 | 1 |
| `pad_event_in_tag` | 头树 ID 8 + 映射 key 27（码 2 / 3 补齐到 27），ALLOC / FREE 折进类标签字节的高位 | 0 | 1 |
| `pad_event_byte` | 同上 + 事件类型 1 字节 | 0 | 1 |
| `two_trees_event_in_tag` | 码 1 一棵 35、码 2 / 码 3 一棵 33 | 0 | 2 |
| `pad_event_in_tag_with_hint` | 35 | 位置条目 14（非权威的提示） | 1 |

### 口径

- 节点几何逐字取 E145（码 2 自描述头与映射树 key 宽的代价）：节点 16384、码 2 头 81 + 2 × key 宽 + 预留 28（D18（块里携带什么信息） 已定项 7 / 已定项 14）、内部条目 = key + 83 字节子指针（D19（块指针的结构与宽度预算） 已定项 7）、满装。
  E132（livelist 载体·按真实树数与内部扇出重算） 用的头 64 与子指针 59 是旧口径，同一个量不再按它算。
- 规模网格照抄 E132（livelist 载体·按真实树数与内部扇出重算）：头数 {1, 4, 16, 64, 256, 1024, 4096} × 每头条目数 {1024, 65536, 1048576}；两棵树那一臂按码 1 : 码 2/3 = 8 : 1 分条目（E145（码 2 自描述头与映射树 key 宽的代价） 的每 8 个数据单元一个节点）。
- 每次 FREE 的预付 = 叶条目宽（追加一条 FREE 就是追加一条叶条目），占 32 KiB 单元的万分点向下取整。
- 第一个事务：day-1 注册 = 树表多一条 200 字节（D8（核心索引结构） 已定项 8），第 1 版树表 5 条 + 1 ≤ 每单元 81 条（C157（树表容量在两处按不同条目宽算） 口径；2026-09-13 用户定案 livelist 与稀疏旁表都 day-1 注册后是 7 条，同样装得进）⇒ 不多占单元；实现若要求空树也有根节点，再加一个节点 16384 + journal 点名项 56 + 码 2 映射条目 53；第一次克隆时再建 = 0 字节。
- 变异 12 条全抓：头树 ID 宽改 4、码 1 映射 key 改 25、头宽漏乘 2、预留位改 0、叶扇出向上取整、内部条目漏加指针、树高每层加 2、两棵树不分条目、预付按节点大小算万分点、空树要根时漏算映射条目、事件字节不加、树表第 1 版条目数改 112。

### 判决表整行抄自产物

```text
E7RESULT name=config node_bytes=16384 header_without_key_range=81 reserved=28 node_pointer=83 head_tree_identifier=8 mapping_key_data=27 mapping_key_node=25 unit_bytes=32768 tree_table_entry=200 data_units_per_node=8
E7RESULT name=node arm=legacy_e130 tree=shared key_bytes=24 entry_bytes=24 header_bytes=157 payload_bytes=16227 leaf_fanout=676 internal_fanout=151
E7RESULT name=node arm=pad_event_in_tag tree=shared key_bytes=35 entry_bytes=35 header_bytes=179 payload_bytes=16205 leaf_fanout=463 internal_fanout=137
E7RESULT name=node arm=pad_event_byte tree=shared key_bytes=36 entry_bytes=36 header_bytes=181 payload_bytes=16203 leaf_fanout=450 internal_fanout=136
E7RESULT name=node arm=two_trees_event_in_tag tree=data key_bytes=35 entry_bytes=35 header_bytes=179 payload_bytes=16205 leaf_fanout=463 internal_fanout=137
E7RESULT name=node arm=two_trees_event_in_tag tree=node key_bytes=33 entry_bytes=33 header_bytes=175 payload_bytes=16209 leaf_fanout=491 internal_fanout=139
E7RESULT name=node arm=pad_event_in_tag_with_hint tree=shared key_bytes=35 entry_bytes=49 header_bytes=179 payload_bytes=16205 leaf_fanout=330 internal_fanout=137
E7RESULT name=prepaid arm=legacy_e130 bytes_per_free=24 unit_share_basis_points=7
E7RESULT name=prepaid arm=pad_event_in_tag bytes_per_free=35 unit_share_basis_points=10
E7RESULT name=prepaid arm=pad_event_byte bytes_per_free=36 unit_share_basis_points=10
E7RESULT name=prepaid arm=two_trees_event_in_tag bytes_per_free=35 unit_share_basis_points=10
E7RESULT name=prepaid arm=pad_event_in_tag_with_hint bytes_per_free=49 unit_share_basis_points=14
E7RESULT name=first_transaction arm=pad_event_in_tag tree_table_entries=1 day1_null_root_bytes=200 day1_root_required_bytes=16693 lazy_bytes=0 tree_table_fits_first_unit=true
E7RESULT name=first_transaction arm=two_trees_event_in_tag tree_table_entries=2 day1_null_root_bytes=400 day1_root_required_bytes=33386 lazy_bytes=0 tree_table_fits_first_unit=true
E7RESULT name=sensitive entries=460 pad_event_in_tag_height=1 pad_event_byte_height=2
E7RESULT name=verdict pad_entry_bytes=35 legacy_entry_bytes=24 pad_leaf_fanout=463 legacy_leaf_fanout=676 grid_cells=21 cells_where_pad_taller_than_legacy=2 prepaid_pad=35 prepaid_legacy=24
```

### 树高（21 个规模格，两棵树那一臂括号里是两棵各自的高）

| 头数 | 每头条目 | 总条目 | `legacy_e130` | `pad_event_in_tag` | `pad_event_byte` | `two_trees_event_in_tag` | `pad_event_in_tag_with_hint` |
|---|---|---|---|---|---|---|---|
| 1 | 1024 | 1024 | 2 | 2 | 2 | 2（码 1 2，码 2/3 1） | 2 |
| 1 | 65536 | 65536 | 2 | 3 | 3 | 2（码 1 2，码 2/3 2） | 3 |
| 1 | 1048576 | 1048576 | 3 | 3 | 3 | 3（码 1 3，码 2/3 3） | 3 |
| 4 | 1024 | 4096 | 2 | 2 | 2 | 2（码 1 2，码 2/3 1） | 2 |
| 4 | 65536 | 262144 | 3 | 3 | 3 | 3（码 1 3，码 2/3 2） | 3 |
| 4 | 1048576 | 4194304 | 3 | 3 | 3 | 3（码 1 3，码 2/3 3） | 3 |
| 16 | 1024 | 16384 | 2 | 2 | 2 | 2（码 1 2，码 2/3 2） | 2 |
| 16 | 65536 | 1048576 | 3 | 3 | 3 | 3（码 1 3，码 2/3 3） | 3 |
| 16 | 1048576 | 16777216 | 4 | 4 | 4 | 4（码 1 4，码 2/3 3） | 4 |
| 64 | 1024 | 65536 | 2 | 3 | 3 | 2（码 1 2，码 2/3 2） | 3 |
| 64 | 65536 | 4194304 | 3 | 3 | 3 | 3（码 1 3，码 2/3 3） | 3 |
| 64 | 1048576 | 67108864 | 4 | 4 | 4 | 4（码 1 4，码 2/3 3） | 4 |
| 256 | 1024 | 262144 | 3 | 3 | 3 | 3（码 1 3，码 2/3 2） | 3 |
| 256 | 65536 | 16777216 | 4 | 4 | 4 | 4（码 1 4，码 2/3 3） | 4 |
| 256 | 1048576 | 268435456 | 4 | 4 | 4 | 4（码 1 4，码 2/3 4） | 4 |
| 1024 | 1024 | 1048576 | 3 | 3 | 3 | 3（码 1 3，码 2/3 3） | 3 |
| 1024 | 65536 | 67108864 | 4 | 4 | 4 | 4（码 1 4，码 2/3 3） | 4 |
| 1024 | 1048576 | 1073741824 | 4 | 4 | 4 | 4（码 1 4，码 2/3 4） | 5 |
| 4096 | 1024 | 4194304 | 3 | 3 | 3 | 3（码 1 3，码 2/3 3） | 3 |
| 4096 | 65536 | 268435456 | 4 | 4 | 4 | 4（码 1 4，码 2/3 4） | 4 |
| 4096 | 1048576 | 4294967296 | 5 | 5 | 5 | 5（码 1 5，码 2/3 4） | 5 |

### 这几个数说明什么

1. **按映射 key 定身份把条目从 24 字节抬到 35 字节**（+46%），叶扇出 676 → 463；但树高在 21 格里只有两格多一层（总条目 65536 时 3 对 2），其余 19 格同高 ⇒ 点查与插入的层数几乎不变，代价主要在字节。
2. **C260（livelist 条目的预付没算进解开自己那个量） 要的数**：每次 FREE 预付 35 字节（带事件字节 36、带提示 49），占一个 32 KiB 单元的 10 / 10 / 14 个万分点；E130（每头一份 livelist 过不过有界销毁） 记的 24 字节（7 个万分点）作废。这个量还没进「解开自己需要多少」那条 2026-08-31 用户定案的条款。
3. **事件类型折不折进类标签**：类标签 1 字节里的码只有 1 / 2 / 3，高位空着，折进去省 1 字节、叶扇出 463 对 450；两种在 21 格上树高全同。这一格是编码取舍，不是代价取舍。
4. **分两棵树**只让码 2 / 3 那棵窄 2 字节（叶 491 对 463），21 格里最大那格节点树矮一层（4 对 5）；代价是树表多一条、与 D6（快照实现模型） 已定项 3「载体取一棵共享树」的字面不合，改它要重开已定项 3。
5. **value 带 14 字节位置提示**是把 C266（livelist 条目的身份用了会过期的位置提示） 作废的那个字段挪进 value：叶扇出掉到 330、预付 49，1024 头 × 1048576 那格多一层（5 对 4），而 D19（块指针的结构与宽度预算） 已定项 5 规则 1 下释放不许按提示走，它买不到任何路径 ⇒ 只报不推荐。
6. **第一个事务上 day-1 与惰性创建差 200 字节**（空树根指针为零的读法）或 16693 字节（空树也要一个根节点的读法）；两棵树翻倍。惰性创建换到的 0 字节要付「第一次克隆时建树」这条机制。
7. 头树 ID 前缀 8 字节与映射 key 里的出生树 8 字节可能冗余（一棵树若只属于一个头，出生树就指得到头）；这是 D6（快照实现模型） 已定项 2 / C267（一个可写头在树表里没有一个能被指到的对象） 的事，这里没建模、没省。

### 它答不了的

1. 每头计数器住哪（C267（一个可写头在树表里没有一个能被指到的对象））、condense 的挂钟代价、节点填充率（按满装算，真实 B 树 50%–100%）。
2. 头树 ID 前缀能不能省、事件类型要不要进 key（ALLOC 与 FREE 同一块各一条时它必须进 key，否则两条同 key）——后一句是 key 唯一性的要求，装置按「进 key」建模。
3. 取哪一种：这是 D6（快照实现模型） 已定项 2 的定案。

## 历史版本

### 2026-09-16
- 树表条目 148 → 200（D8（核心索引结构） 已定项 8，2026-09-16 用户定案加宽），装置的 `TREE_TABLE_ENTRY_BYTES` 跟着改，产物从 `research/results/e146-livelist-entry-width-2026-09-14-round2.out` 换成 `research/results/e146-livelist-entry-width-2026-09-16-tree-table-200.out`；哪几行变了见 [experiments-history.md](../experiments-history.md) 2026-09-16（其一）。

### 2026-09-14
- 树表条目 145 → 148（D8（核心索引结构） 已定项 8 的树表条目 2026-09-14 用户定案重排并加头 ID 8、根指针从 83 到 86，`format-const: TREE_TABLE_ENTRY_BYTES = 148`，门禁阶段「格式常量在 kb 与实验源码之间同步」当天判红）重跑：
  产物换成 `research/results/e146-livelist-entry-width-2026-09-14-round2.out`，130 行里 **6 行**变——`name=config` 的 `tree_table_entry` 145→148，5 行 `name=first_transaction` 的 `day1_null_root_bytes` 145→148（两棵树那臂 290→296）与 `day1_root_required_bytes` 16638→16641（两棵树 33276→33282）；
  条目宽、预付字节、规模网格上的树高、判别取样点各行逐字未变。8 个单测全绿。
  ⚠️ 变异表 M12 的原文停在 `TREE_TABLE_FIRST_VERSION_ENTRIES = 5`，而装置 2026-09-13 已按定案改成 7 ⇒ 这一条从那天起就替换不上、整轮变异中断（退出码 3）。原文改成 7 之后 12 条全抓。
  曾经：树表条目 145，day-1 注册 145 / 16638 字节（两棵树 290 / 33276）。

### 2026-09-13
- 新建。跑前登记写于装置之前，判据与失败条款没有改过。
- 用户同日定案取带事件字节的 36 字节形态、day-1 注册（D6（快照实现模型） 已定项 2）；装置里第 1 版树表条目数常量按定案改成 7，产物不含它、逐字节不变。
