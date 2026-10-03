# 删除（新事务）的设计：第一轮正文（2026-09-30）

<!-- doc-lint:not-numbers U1 U2 U3 U4 U5 U6 U7 -->

## 一、题面

里程碑三三·1（`.claude/kb/milestone/03-third-txn.md`「三　删除（新事务）」三·1 那一行）。第一版只做单文件 unlink，没有目录，按 inode 号删，不带 truncate（用户 2026-09-28 定）。这一轮要定格式与机制的条款，写回 D8（核心索引结构） 新分项、还清 C118（`deleted_inodes` 树的形态无落点）、给 `layout/03-third-txn.md` 建档；每条岔路带代价数交用户。

已定的概念层（整节原文在附录）：D8 已定项 5（意图写进 logged_ops 再分批推进、崩溃后续做；删除中的 inode 单独进 `deleted_inodes`、由后台回收；意图只声明目标态；续做是每次恢复都跑的常规动作；每个中间态都合法；删除意图与最后一批同一事务提交）与已定项 6（回收完成前 inode 记录留在 inode 树里、nlink = 0；已发布的 (inode 号, 对象出生代) 不复用）；D18（块里携带什么信息） 已定项 8（删除要写墓碑单元）与已定项 10（墓碑取区间记录 + 打包共享单元）。

## 二、实现今天的样子（主 agent 的观测，2026-09-30，HEAD fcc4d340）

事实表 `research/prompts/m3-delete-r1-facts.md`（事务员 2026-09-30 交回，sha256 5a22673508fddfbcc43375ada09b602ecc41be46c652cd2029847f6ffcfd9313）九张表逐格带 `文件:行`。主 agent 现查了第一条，与代码逐字对得上。

- **实现今天只认一个带数据的文件（inode 1）**：恢复重建一版时 `crates/singlefs-core/src/recovery.rs:3347-3360` `extents_of_the_first_file` 要 extent 树上段恰好一条 inode 1 的叶条目，`recovery.rs:2499-2506` 要 inode 树里有 inode 1 那条记录，不满足报 `UnitMalformed`；「没有文件」今天只有树表 0 条那一种。所以删掉 inode 1 之后的那一版会被恢复当坏镜像拒（事务员推的）。`transaction.rs:4640-4644` `publish_first_file` 要那一版树表 0 条，所以「删了之后再建第一个文件」今天走不通。
- **mkfs 一棵树都不建**：树表第 0 版 0 条（`make_filesystem.rs:457-469`）；七条树表条目（含 livelist、稀疏旁表、deadlist 三棵零根）是第一个文件版本那次发布写的（`transaction.rs:6482-6524`）。
- nlink 写死 1、读时跳过（`records.rs:56`、`:85`），`InodeRecord` 没有 nlink 字段，checker 不读。
- 「待删占用」那一行记账已在写、恒 0（标签 4，`transaction.rs:6283-6285`，值是 `admission.rs:151-152` 的常量），准入式子已扣它，checker 不读；事务员推的口径问题：要删的 extent 在释放之前还算在「已分配」里，「待删占用」若也记这些字节，同一批会被扣两次（C375（待删占用与已承诺预留按份还是按和记没有条款））。
- 释放按角色点名（`roles_replaced_via_mapping` → 读盘核 → `release_leaving_the_record_allocated_on`，进 defer），重复释放同一落点报 `ReleaseTargetAlreadyReleased`（`transaction.rs:3219-3220`）。
- 一次发布的记录上限 65 536（默认环，`transaction.rs:5387`），一条记录 67 个点名项；只供测试的点名项容量开关 `JournalRecordNamedEntryCapacity::CappedForTests` 是「只供测试的旋钮」的先例。
- 可写挂载的写序列与续做挂点候选：事务员推在暖机循环（`mount.rs:3955`）之后、`Ok(Mounted)`（`:4018`）之前，取号前的预演（`:3767`）与准入要把续做那几批算进去。
- 崩溃点名字按写种类自动生成（`crash_amplification.rs:2389`，`<入口>.<写种类>.<第几段>`）；固定脚本流在 `crates/singlefs-checker-tier/tests/common_crash_points/mod.rs`，加一个 unlink 步不需要新编码。
- `grep -rn '没有删除' crates/ research/ .claude/kb/` 45 行（事实表 (h) 节全文），其中 `inode_tree.rs:28`、`admission.rs:154` 两处注释的前提会随删除失效。

## 三、格

### U1　格式：`deleted_inodes` 与意图记录住哪（C118）
`deleted_inodes` 的 key / value / 树 ID、何时建（mkfs、第一个文件版本那次、还是第一次 unlink 那次；与「mkfs 一棵树都不建」的今天怎么对上）；意图记录住单独一棵 logged_ops 树，还是记账或树表里一条条目；单文件 unlink 的意图目标态怎么写（「inode N：无 extent、记录删除」这类是推的）。新树落 D15（格式冻结政策） 已定项 7 的哪一层、态别（登记进 `.claude/kb/freeze-layer-membership.md`）。每个候选给多写几个单元、几条记录、几字节。

### U2　墓碑（D18 已定项 8 / 10，C84（墓碑单元的粒度没人定）、C109（哪些树的 key 死亡要写墓碑））
第一版 unlink 死掉的 key 有哪些（extent 叶条目、inode 记录在回收完成时、映射条目），哪些要写墓碑、墓碑的粒度；不写墓碑的那几类凭什么（D18 已定项 8 原文）。

### U3　分批与续做（C1（意图续做做成常规路径）、C2（无界操作做成单事务））
分批的粒度（按 extent 数、按单元数、按记录条数）；**分批必须能被测试强制进入**（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：第一版单文件的 extent 数有界、按字面可能一批做完，要一个只供测试的批大小旋钮，固定脚本的删除段至少两批；续做入口挂在可写挂载的哪一步（事务员推的挂点要核），取号前预演与准入怎么把续做算进去；只读挂载看到进行中的删除时读到什么（inode 记录 nlink = 0 留在树里：「文件在不在」对读者的答案）。

### U4　空间：可用空间扣待删占用（`.claude/kb/invariants.md` 的 I-3.4（可用空间扣待删占用），C375）
待删占用记什么（未释放的 extent 字节，还是别的）、与「已分配」会不会扣两次、每批之后怎么变；给一段三批的删除里每批之后的可用空间逐格值。

### U5　恢复与模型的前置
恢复今天只认 inode 1：删掉之后的一版（inode 记录 nlink = 0 留树、extent 树空、树表条目非 0）按什么形态认「没有文件」；删完之后能不能再建文件、`publish_first_file` 的前置怎么改（inode 号不复用，新文件拿 inode 2）；理想模型（`crates/singlefs-harness/src/model.rs`）的删除操作与读回答案。

### U6　崩溃点与流
崩在回收中间的崩溃点怎么进放量的节点（自动命名够不够分辨「第几批」）；固定脚本到 E 之后加 unlink 段、还是另起一条流，各自多少崩溃状态（推的，给算式）；删除类发布崩在中间（安全轮第三轮没跑的截断成 0 那一格，`research/prompts/m2-safety-r3-main-verification.md` 第四节）并不并进来。

### U7　与稀疏旁表、整理共用意图机制
D5（快照 / 空间记账机制） 已定项 6 的稀疏旁表与 D26（后台整理与放置回收） 的整理也用意图机制：第一版 unlink 定下的意图记录形态要给它们留什么（字段、种类码），不留的话以后加要不要改格式。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | U1–U7 | 从已定条款与冻结副本推每格的形态与代价数，给推荐 |
| 云端攻方（Opus） | U1、U3、U4、U5 为主，U2、U6、U7 找反例 | 找每个候选在崩溃中间态、续做两次、只读挂载、准入上的反例；U4 在副本上把「扣两次」那一格量出来；U5 在副本上核「删掉 inode 1 之后恢复拒」成不成立 |

第一轮，不派辩方。

## 五、跑前写死的判据

- 一个候选让某个中间态不是合法状态（D8 已定项 5）⇒ 出局。
- 续做同一批两次盘上字节不同 ⇒ 那个形态出局。
- 分批进不了测试（没有只供测试的旋钮、或固定脚本里只有一批）⇒ 那个形态不收。
- U4：量出扣两次 ⇒ 待删占用的口径必须改，写明改成什么。
- U5：攻方核实「删掉 inode 1 之后恢复拒」⇒ 恢复的形态判定改动进三·2 的实现清单。
- 各候选的代价数（多写的单元、记录、字节、崩溃状态数）进交用户的岔路表；腿提的新形态算被攻过零轮。

交付：腿的报告写 `research/prompts/m3-delete-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；攻方的副本与用例放 `research/prompts/m3-delete-r1-opus-model/`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。

## 六、冻结副本

腿读代码读 `/tmp/claude-1000/m3-ring-r1/tree/crates/`（与 m3-ring-r1 那一轮共用一份，sha256 在 `research/prompts/m3-ring-r1-snapshot/crates-sha256.txt`，这一轮快照目录里另存一份同内容的清单）。
