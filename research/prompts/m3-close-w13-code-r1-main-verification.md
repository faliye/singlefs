# m3-close-w13-code-r1 判决（代码轮第一轮，2026-10-21）

正文 `research/prompts/_m3-close-w13-code-r1-body.md`，背景材料 `research/prompts/_m3-close-w13-code-r1-background.md`（sha256 eef66d116800a9127173b94f675b3c1530840ea99a78c28c769d34de409be69c）。开工快照 `research/prompts/m3-close-w13-code-r1-snapshot/`（`code-sha256.txt`、`kb-sha256.txt`、`product-code.diff`）。

被判的 7 份文件是：
- `crates/singlefs-core/src/make_filesystem.rs`
- `crates/singlefs-core/src/extent_tree.rs`
- `crates/singlefs-core/src/allocator.rs`
- `crates/singlefs-core/src/allocation_record_tree.rs`
- `crates/singlefs-core/src/records.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/mounted_read.rs`

两条腿：
- 正推（Sonnet）：`research/prompts/m3-close-w13-code-r1-sonnet-output.md`，sha256 3e8b24e140ee8f28199c4c18e4c80d1dbc38c8388c978658dbc42feb6b1b9d7e。
- 攻方（Opus）：`research/prompts/m3-close-w13-code-r1-opus-output.md`，sha256 928b746926ae7fcaccaa6303eaf7559fecd45d377535ab861d2f8230c655b396。模型目录 `research/prompts/m3-close-w13-code-r1-opus-model/`。

## 一、核查

1. 快照：腿交齐之后 `sha256sum -c research/prompts/m3-close-w13-code-r1-snapshot/code-sha256.txt` 七份都 OK；`kb-sha256.txt` 全部 OK。腿跑着时这七份没被改过，腿引的行就是主工作区现行的行。
2. 引文：`python3 scripts/agents/cite-check.py <两份报告> --root . --background <背景材料>` 末行「✓ 核了 112 处引文、71 处裸行号，对上 112 处，没判 71 处（其中裸行号没有引文可比 71 处；逐处列在上面）」。判决用到的裸行号我逐处读过：
   - `.claude/kb/decisions/22-单元原子性怎么合成.md:179` 写「段序列与步数以 [layout/01-first-txn.md](../layout/01-first-txn.md) 八「mkfs 种根」那一行为准」；`.claude/kb/layout/01-first-txn.md:410` 仍是「… = 4 单元写，共 14] 屏障 [m3 根 FUA 区域 0] …」那一版。
   - `crates/singlefs-core/src/extent_tree.rs:515`：`if bytes.len() < usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113")`。
   - `crates/singlefs-core/src/records.rs:853`：`if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55")`。
   - `crates/singlefs-core/src/allocation_record_tree.rs:887-888`：`|| header.entry_width < usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")` 之后 `continue`。
   - `crates/singlefs-checker/src/walk.rs` 的 deadlist 那一支注释原话「条目宽同中央映射树只守走读要几个字节（I-1.10 的射程里没有它）」；`field_table_width_of_tree_kind` 里没有 `TREE_KIND_DEADLIST` 那一臂。
   - `crates/singlefs-checker/invariants/01-块自描述.md:16` I-1.10 列的树是 extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108，没有 deadlist。
   - `.claude/kb/decisions/22-单元原子性怎么合成.md:181` 射程那一句原话：「作废持久之后、最后一道屏障之前崩溃：下一次挂载只有两种结局——整池拒（`NoValidSystemConfiguration`），或恢复读回 mkfs 的第 0 代空池（根为实例 0、txg 0，没有文件）」。
3. 攻方在副本上量的数（K2 的拒因、K5 的 deadlist 宽 44 镜像）判决里不引，K2、K5 都按上面读过的代码与条文判；所以没复跑它的模型。

## 二、逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| K1 | 代码照条文，没打中代码 | 两腿都核过写序与分段逐条对得上。两腿都报 kb 内部矛盾：D22 已定项 8 指到的 layout 那一行是旧段序（14+1+1+1+4、25 步），代码与钉住它的用例是 2+12+1+1+1+4、27 步。这是 kb 写回欠的，已在这一段 kb 写回清单（`layout/01-first-txn.md:162,410`），提交前门禁 segment-registry 格会红，写回前不提交 |
| K2 | 代码照条文，没打中代码；条文措辞有缝，交写回 | 随机源打不开、读不满都在任何写之前拒，不退回别的来源（两腿读码一致）。攻方说的「调用方拿旧池 fsid 去挂重做出来的空池报 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`」：那是调用方断言的参数与盘不符，不是盘面的结局；条文那一句说的是盘面的两种结局，判决 `research/prompts/m3-close-c656-r3-main-verification.md` S3 已删挂载两个分句、用例按恢复结局判。不算代码错。射程句「下一次挂载」改成说恢复结局、并写明调用方断言参数那一拒不算盘面结局，交 kb 写回，零轮 |
| K3 | 没打中，不拿它支撑结论 | 攻方只抽了一次、全是推的；照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」不记「没打中」。mkfs 中途崩溃的结局由 checker 档 `crash_enumeration_make_filesystem_over_a_used_pool_stream` 快档与全量判，快档这一趟撞内存没跑完，修完再跑 |
| K4 | 六处照条文；漏改的三处判去向 | 六处比较两边都是条目宽对字段表宽（字节），两腿一致。`extent_tree.rs:515` 上段叶 113：真漏，已在根环一簇（C867 第七处）。`records.rs:853` 映射条目 55：I-1.10 原文明写不判，账在 C307，不动。`allocation_record_tree.rs:888` 影子账走读：有意宽松，条款没要求它拒，不动 |
| K5 | 改名照条文；打中一处跨 oracle 不一致（不在被判文件里） | 每个调用方的处置与改名前相同（两腿一致）。打中：实现照 C867 拒宽 deadlist 条目，池级 checker 明写 deadlist 不在 I-1.10 射程里，宽 44 的 deadlist 镜像上恢复拒、checker 不报。改法：I-1.10 射程加 deadlist 叶与内部条目、`walk.rs` 的 deadlist 那一支按字段表宽判。`walk.rs` 在 m4-i315-r3 的开工快照里，等那一轮判完进下一批。名实不符（`RecoveryFailure::EntryNarrowerThanItsFieldTable` 与 `recovery.rs` 文档注释「窄于」）已在根环一簇 |

## 三、改法（被攻过零轮）

被判的 7 份产品代码这一轮不改。打中与漏的去向：

| 项 | 改什么 | 批 |
|---|---|---|
| K1 layout 段序 | `layout/01-first-txn.md:162,410` 三语与 `scripts/crash/check-segment-registry-product-sample.txt`、e142 装置的旧数 | kb 写回批（已在清单） |
| K2 射程措辞 | D22 已定项 8 射程「下一次挂载」那一句 | kb 写回批，零轮 |
| K2 失败分支没有用例 | 随机源读不出、读不满那一支补用例；`RANDOM_SOURCE_PATH` 是常量，要能换源才写得出用例，动产品代码 | 根环一簇那一批（同动 `crates/singlefs-core/`），代码轮再攻 |
| K4 上段叶 113 | 已在根环一簇 | 根环一簇 |
| K5 deadlist 跨 oracle | I-1.10 射程加 deadlist，`walk.rs` deadlist 那一支按字段表宽判，造宽 44 的坏镜像用例 | m4-i315-r3 判完之后的下一批（与 C773 同批） |

## 四、交给实现员

这一轮没有要并进批 14 改红批的条目。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-10-21 改了：射程「下一次挂载」那一句交写回，改成说恢复结局；layout 段序跟着写回 |
