# 他家方案调研

**全部为文档阅读或源码逐行核对所得，未在本工程验证**，也未编译、运行过任何一个项目。
星数/提交数是页面当时显示的值（2026-08-25）。

**来源固定点与复核机制**：承重引用的源码与文献固定在本机
`~/code/fs-refs/`——`linux-6.17/fs/bcachefs/`（kernel.org 的 `linux-6.17.tar.xz`，260 个文件）、
`zfs/`（OpenZFS master，`git log -1` = `58d73c9`，2026-08-27）、
`docs/` 下 7 份文献（RFC 8439、NIST SP 800-38B / 800-38D、DJB *Cryptography in NaCl*、
OSTEP 第 45 章、BetrFS FAST 2017 / 2018、NetApp FAST'20；取回方式与 sha256 见
`scripts/kb/fetch-refs.sh`，抽取器 `scripts/kb/pdf-text.py`）。
**89 条**承重引用做成了可重跑的逐字断言（`scripts/kb/verify-citations.sh`，这个数由它自己末行报出来：`bash scripts/kb/verify-citations.sh` 的「结果：N 条命中」，别手数——源码里 `^(ck|ckn|ckdoc|ckdocn)` 的行还含 11 条只在 `--selftest` 里跑的合成断言）；其中 38 条核的是本机内核树、不是固定点树（C44（引用核到的不是固定点树）），
挂在门禁阶段「外部引用还核得动吗」（`.claude/gate.d/doc-registries.sh` 的 citations 格）——**源码树不在也判红**，
「跳过」正是让上一批文献无声蒸发的那个行为。
⚠️ **ZFS On-Disk Specification 没收**：它用 CID 双字节字体，`pdf-text.py` 抽出乱码 ⇒
按该脚本自己的规矩算**抽取失败**，不算「原文没这句」；引它的几条改从已固定的 OpenZFS 源码树核。
⚠️ 哪些引用核的不是固定点树，见 [checks-owed.md](checks-owed.md) C44（引用核到的不是固定点树）。

## 一、记账模型四选一

| 模型 | 谁 | 机制 | 代价 |
|---|---|---|---|
| extent refcount + 反向索引遍历 | btrfs qgroup | 精确算 shared/exclusive | 提交时间 +76%、事务等待 +1347%、写吞吐 −25% |
| 归属原创建者，不算全局 refcount | btrfs squota | `OWNER_REF` 内联，记账 1:1 绑 extent 生命周期，不走反向索引 | 仅比关配额慢 3.4%；算不出 shared/exclusive |
| birth txg + deadlist | ZFS | 块自带诞生代号；删除时与「下一个更老的快照」比代号 → 直接释放，或进那个快照的 deadlist；销毁快照时 deadlist 级联合并 | 记账增量维护，永不遍历 |
| accounting key 进专门的 btree | bcachefs（2024 重写） | 计数器本身是一条 bkey，tagged union 可扩展；内存侧 percpu + eytzinger 索引；更新走普通事务 + write buffer | 该重写破坏磁盘格式兼容，6.11 强制升级 |

**数字口径**：qgroup / squota 的百分比来自 btrfs 官方在 LWN 那篇 squota 文章里给的对比测试，
负载与硬件未在该文中完整披露，**不可当作本工程的性能预期**，只能当作「这条路走不通」的定性证据。

**关键信息：btrfs 自己投降了。** squota 就是官方承认精确记账做不动。
这条路走不通已经不需要本工程再验证一遍。

**三家在同一点上分胜负：记账是事务的副产品，还是事后的遍历。**
btrfs qgroup 事后遍历 → 死；ZFS 与重写后的 bcachefs 事务时增量维护 → 活。

来源：
- [btrfs: simple quotas (LWN)](https://lwn.net/Articles/944371/)
- [bcachefs disk accounting rewrite (LWN)](https://lwn.net/Articles/963570/)
- [Advanced ZFS Dataset Management — Klara](https://klarasystems.com/articles/advanced-zfs-dataset-management/)
- [Disk Space Accounting for ZFS Snapshots — Oracle](https://docs.oracle.com/cd/E19253-01/819-5461/gbcxc/index.html)

## 二、bcachefs 值得研究的几点

来源：[bcachefs Snapshots 设计文档](https://bcachefs.org/Snapshots/)、
[Principles of Operation (2026-04-16, PDF)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)。
**Principles of Operation 是最新最权威的 bcachefs 设计文档，尚未通读。**

1. **bucket + generation 号**。盘切成 bucket，每个带一个代号；指针存「期望代号」，
   代号对不上说明这块已被回收 → 数据失效不需要更新任何反向索引。对 D1（数据可移动性 / 反向索引） 是直接减负。
2. **快照 ID 进 key 低位**。快照 ID 自成一棵树（根 `U32_MAX`，向下分配，父 ID 恒大于子 ID），
   查找按祖先关系过滤，**不 clone 任何树**，创建 O(1)，可有百万级快照。
   代价：删快照要遍历所有带快照的 btree（官方提到想用 bloom filter 优化）；
   读路径有恒定祖先过滤开销；写路径要处理 whiteout 与跨快照的部分覆盖分片。
3. **指针内联多副本**。一个 extent 的指针直接列出 N 份副本位置，
   于是副本级别可按文件设——btrfs 是 per-chunk profile，做不到。
4. **B-tree 节点内部是日志结构的**（大节点，小更新追加，读时合并、后台压实）。
5. **copygc / rebalance 是常驻后台进程**，不是运维要手动跑的命令。
6. **反向索引是后来才加的**——这条是教训：证明了它补起来有多难。

**工程节奏上不要照抄**：bcachefs 磁盘格式改了很多年才稳，
2025 年被移出内核主线维护。技术可参考，节奏不可参考。

## 三、Rust 侧现有轮子

| 项目 | 语言 | 做到哪儿 | 状态 |
|---|---|---|---|
| [rustutils/btrfsutils](https://github.com/rustutils/btrfsutils) | Rust | `disk`（含 RAID5/6 Reed-Solomon）/ `transaction`（写路径）/ `fuse`（只读挂载）/ `stream` / `mkfs` / `tune` / `cli`；`btrfs check` 七阶段、`rescue chunk-recover` 已实现 | 0.13.0（2026-05-14）。13 stars、464 commits、**0 open issues、20 open PRs**。测试是 unit + `cargo insta` 快照，**页面上看不到 fuzz / xfstests / 差分测试**。`transaction` 自带警告 "Do not use it on filesystems you care about" |
| [adam900710/btrfs-fuse](https://github.com/adam900710/btrfs-fuse) | C | 只读 FUSE，全 RAID(0/1/10/5/6)、全压缩、四种校验和 | 作者是 btrfs 内核维护者 |
| [btrfs-rec](https://www.lukeshu.com/blog/btrfs-rec.html) | Go | 只读 + 重建损坏的树/chunk 映射 | 恢复领域最强的开源实现 |
| [btrfscue](https://github.com/cblichmann/btrfscue) | Go | 全 superblock 被覆盖时刨数据 | 老项目 |
| [GodTamIt/btrfs-diskformat](https://github.com/GodTamIt/btrfs-diskformat) | Rust | 只有结构体定义，读不了盘 | 22 commits，停滞 |
| [Dicklesworthstone/frankenfs](https://github.com/Dicklesworthstone/frankenfs) | Rust | 号称 ext4+btrfs 重写、22 crate、zero unsafe | 21 stars / **8213 commits** / 0 issues。**这个比例强烈暗示是 agent 批量生成**——推断，未读代码 |

**对本工程的意义**：btrfsutils 证明了「照着 btrfs 格式做 Rust 实现」三个月能出雏形，
但那是因为**格式现成、且 `btrfs inspect-internal dump-tree` 是免费的裁判**。
本工程从零设计，这两样都没有——**没有 oracle 是从头做的最大隐性成本**，
必须在预算里，见 `.claude/singlefs-ai-sop/rules/test-discipline.md`「模型对拍」。

## 四、内核 Rust 文件系统抽象层

至 2026 年，Rust 已进入 Linux 的驱动、文件系统、网络组件。fs 抽象层仍在演进，
且尚未就「通用 vs 只服务简单文件系统」达成结论。

来源：[Rust for filesystems (LWN)](https://lwn.net/Articles/978738/)。
**与本工程关系不大**——D7（是否进 Linux 主线） 已定前几年不进主线。

## 五、索引结构（D8（核心索引结构） 专项调查）

**口径**：「五、索引结构」一节来自 bcachefs *Principles of Operation* Rev 1.39.2（2026-08-25 修订）原文、
LWN、XFS 文档与论文摘要。**未在本工程验证，也未 clone/编译/运行任何一个实现。**

### 5.1 bcachefs：28 棵树，一套 btree 实现

> "The entire filesystem is built on two primitives: btrees and buckets.
> All filesystem state is a key-value pair in a btree. ... There are no separate
> inode tables, bitmap allocators, or per-inode extent trees."

它**不是**「按访问模式换数据结构」，而是**分成 28 个独立 keyspace，共用同一套 btree**，
再用两个前端解决两类难搞的访问模式：

| 前端 | 解决什么 | 用在哪 |
|---|---|---|
| **key cache** | 同一批 key 反复读写，每次全树查找太贵 | alloc btree（extent 更新要改所属 bucket 的分配信息） |
| **write buffer** | 高频小更新，批量施加更划算 | backpointers / LRU / accounting |

28 棵树里 **10 棵是 write-buffered**：`lru`、`need_discard`、`backpointers`、
`deleted_inodes`、`accounting`、`stripe_backpointers`、四棵 `reconcile_*`。
⚠️ 28 / 10 是 Principles of Operation Rev 1.39.2 文档里的数；2026-08-29 现查 linux-6.17 源码 `BCH_BTREE_IDS()` 是 21 棵、5 棵走 write buffer，以 D8（核心索引结构） 正文为准。

**write buffer 的代价（原文明写）**：更新是**无序且最终一致**的——
btree 在下次 flush 前看不到待处理更新；flush 按 key 位置排序，**丢弃时间序**；
去重后按 btree 节点顺序批量插入。
> "this sort-merge-sweep is inherently single-threaded, making its efficiency
> critical for multithreaded workloads."

即：write buffer 把写放大压下去了，但**换来一个单线程的 flush 瓶颈**。

### 5.2 日志结构的 btree 节点

节点 128K–256K（默认 256K），**内部是日志结构**：新 key 追加，不按序插入。
一个节点含多个 bset——上次全量重写留下的那个，加后续追加的若干。
查找要合并所有 bset 的结果；内存里定期重排，使一个节点**最多约 3 个 bset**
（正在写入的那个 ≤ 8×16K，其余大致成几何级数）。

三条性质值得注意：

1. **写是顺序的**（对 SSD 和机械盘都好）。
2. **节点可以在只读锁下写盘**——因为只追加。这让锁持有时间很短，
   回写不阻塞读者。**这是并发上的实惠，不只是 IO 上的。**
3. **删除只能靠 whiteout**（插入墓碑）——已写盘的 bset 里删不掉东西。
   这是日志结构节点的强制后果，会渗进所有涉及删除的代码。

key 在 bset 内是**打包**的：每个 bset 有个格式描述符记录哪些字段全体相同，
压缩存储，**省 30–50% 元数据空间**。

**为什么别人用不了大节点**：其他 COW 文件系统用 Linux page cache 存元数据，
被限制在 4K 节点；bcachefs 自管 btree 节点缓存，才能用 128K–256K 并独立调回收。
> "At petabyte scale with spinning disks, deep btrees with 4K nodes mean more
> seeks per lookup and level-2 nodes too large to stay resident under page
> cache pressure."

**对本工程的意义**：D7（是否进 Linux 主线） 已定不进主线，所以**这条约束对我们根本不存在**——
大节点是白捡的，不需要为它付出「自管缓存」以外的代价。

### 5.3 bucket 分配器：从扫描线程改成 btree

- bucket 512K–2M；bucket 与数据指针都带 **generation 号**，
  递增代号即可让旧指针全部失效，**不必去找、去删任何反向引用**。
- 原文说代号机制的价值超出了缓存失效：
  它让 journal replay 时**自举分配信息**变得可做，
  并且**让某些本来不可恢复的损坏变得可修**。← 直接呼应我们的「可重建性」设计目标。
- bucket 内**永不重写**，顺序写满即封存 → 必须有 copying GC。
  **copyGC 预留 8%（默认，可配 5%–20%）**——这是 D3（空间分配）「预留不能过大」的一个现成参照量。
- 有一棵 **fragmentation LRU btree** 记录 bucket 填充度，
  copygc 直接找最碎的，不用扫整棵 alloc btree。
- **分配器原本是内存 bucket 数组 + 专用扫描线程**维护 freelist/discard/eviction；
  后来全部换成 btree（freespace / discard / LRU 三棵），**扫描线程整个删掉**。
  原文给的理由：事务性 btree 代码「far easier to debug and reason about」，
  而且旧线程在文件系统接近满时会**吃掉大量 CPU**——换成 btree 后这些角落情况消失。
- bucket 天然映射到 SMR / zoned 设备的 zone。

### 5.4 记账（对 D5（快照 / 空间记账机制） 的独立印证）

> "Accounting counters are maintained by triggers as keys are committed,
> not by walking the btrees."

而 `check_allocations` **靠全量遍历重建记账**，用来分辨「计数器过期」和「key 真的还在」。
这与 `invariants.md` 的 I-3.1（已分配统计对得上） 是同一个设计：**运行时增量维护，checker 用遍历来验它**。

### 5.5 纠删码：write hole 的第二个已知解法

bcachefs **在后台对整个 bucket 编码**，而不是把前台写切成条带：
前台正常多副本写 → 后台攒够候选 bucket → 算 Reed-Solomon → 写 parity →
原子地更新所有指进去的 extent（丢掉多余副本、加上 parity 指针）。

> "This approach avoids the write hole entirely: parity is computed once for
> immutable data, and the extent updates are atomic btree operations."

**这是与 ZFS RAID-Z「变宽条带、每次全条带写」并列的第二个已知好答案**，
路子完全不同：一个是让部分条带写不存在，一个是让 parity 只对不可变数据算一次。
代价：stripe 里只要还有活数据，其中的 bucket 就不能复用，copygc 得整条 stripe 疏散。

### 5.6 Bε-tree / BetrFS：全路径索引的改名代价已在 2018 年被降到子树深度

Bε-tree 是 B-tree 变体，**内部节点留出 ε 比例的空间缓冲消息**，
写被批量摊销着往下刷，写 I/O 在理论上低于 B+tree。BetrFS 是第一个用它的内核文件系统。

BetrFS 用**全路径索引**（key 就是完整路径），scan 极快。
改名要替换子树里每一个 key 的前缀，**朴素实现必须重写整棵子树**——
FAST 2018 论文原文称这是全路径索引的「Achilles' heel」。

**BetrFS 0.4（FAST 2018）用 lifted Bε-tree 把这个代价从「子树大小」降到「子树深度」。**
论文原话：采用 key lifting 之后，「the number of paths that need to be modified in a
range rename also changes from being proportional to the size of the subtree to the
depth of the subtree」；复杂度一节进一步说，tree surgery 里被切分或合并的节点数
「at most proportional to the height of the tree」。

机制两步：

1. **tree surgery**——把子树左右边缘那些「混装了要搬和不要搬的 key」的节点切开
   （复用普通节点分裂的代码，只是按切分 key 而不是按中点切），然后做指针搬移。
2. **batched key update**——把子树里所有 key 的前缀整体换掉。
   这一步由 **lifting 不变量免费完成**：lifted Bε-tree 里 key 的值由「走到该节点所经过的路径」
   定义，指针搬移一落地，子树里所有 key 就已被隐式换掉前缀。

**lifting 对 key 编码的要求**：key 必须让「同一子树共享前缀」在树结构层面成立。
BetrFS 0.4 另外调整了 key 格式，使 `memcmp` 足以做 key 比较。
另有 healing 机制维持「内部节点 4–16 个孩子」的不变量，用来把树高卡住。

**论文自陈的代价**：BetrFS 0.4 相对 0.3 变差的情形是**节点切分与合并落在关键路径上**时，
lifting 的额外计算开销压过收益；全部形态里唯一被害的是**顺序写**。
改名的声明是「competitive with indirection-based file systems for a range of sizes」——
**是一段尺寸区间内竞争力相当，不是全尺寸都赢**。

口径：FAST 2018 论文 + TOS 2018 扩展版，**未在本工程验证**。
对本工程 key 布局的影响见 [decisions.md](decisions.md) D8（核心索引结构）。

来源：
- [The Full Path to Full-Path Indexing (FAST 2018)](https://www.usenix.org/system/files/conference/fast18/fast18-zhan.pdf)
- [Efficient Directory Mutations in a Full-Path-Indexed File System (TOS 2018)](https://www.cs.unc.edu/~porter/pubs/tos18.pdf)

### 5.7 XFS 反向映射：成本量级与另一个理由

rmapbt 是每个 AG 一棵 b+tree，记录 `(物理块, 属主, 偏移, 块数)`。

**空间成本（单一来源的基准，未复核）**：50TB 文件系统上，
裸 `mkfs.xfs` 约 52G 元数据（~1%）；开 `rmapbt=1` 升到 675G（~1.35%）；
再开 `reflink=1` 到 981G（~1.96%）。
⚠️ 该数字来自一篇个人博客的 mkfs 输出，**是格式化时的元数据规模，不是实际使用量**，
只能当量级参考。

**更重要的是它给了 D1（数据可移动性 / 反向索引） 第二个理由**：XFS 文档说反向映射
「is an essential feature for repairing filesystems online because the secondary
copy can be used to rebuild damaged primary metadata」。

即：**反向索引不只是为了 balance/defrag，它是在线修复得以可能的前提**——
正向索引坏了，可以用反向的重建。这直接对上避坑清单第 10 条
（修复工具没人敢用）。

来源：
- [bcachefs: Principles of Operation, Rev 1.39.2 (2026-08-25)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)
- [bcachefs Allocator](https://bcachefs.org/Allocator/)
- [xfs: add reverse mapping support (LWN)](https://lwn.net/Articles/695290/)
- [mkfs.xfs(8)](https://www.man7.org/linux/man-pages/man8/mkfs.xfs.8.html)
- [Notes on XFS: metadata size (k1024.org)](https://k1024.org/posts/2019/2019-02-08-notes-on-xfs/)
- [BetrFS: A Right-Optimized Write-Optimized File System (FAST'15)](https://www.usenix.org/system/files/conference/fast15/fast15-paper-jannen_william.pdf)
- [The Full Path to Full-Path Indexing (FAST'18)](https://oscarlab.github.io/papers/fast18-betrfs.pdf)
- [Closing the B-tree vs LSM-tree Write Amplification Gap (FAST'22)](https://arxiv.org/abs/2107.13987)

## 六、加密（D9（加密） 专项调查）

**口径**：来自 bcachefs Encryption 设计文档原文（页面标注最后编辑 2025-11-02）、
OpenZFS `module/os/linux/zfs/zio_crypt.c` 顶部设计注释（master 分支，2026-08-26 取）、
Linux fscrypt 内核文档、OpenZFS `zfs-load-key(8)` 手册页、以及 OpenZFS 公开 issue。
**全部为文档与源码阅读所得，未在本工程验证，也未编译或运行任何一个实现。**

### 6.1 三种加密边界

| 边界 | 加密什么 | 能否检测篡改 | 无 key 时能做什么 | 主要代价 |
|---|---|---|---|---|
| 块层（LUKS / dm-crypt） | 整个设备 | **不能**（无 MAC 存放处） | 文件系统的一切照常 | 无法按文件/子卷分密钥；MAC 与 nonce 没地方放 |
| 文件级（fscrypt：ext4 / f2fs / UBIFS） | 文件内容 + 文件名 | **不能**（用 XTS / CBC-CTS / Adiantum） | 全部元数据可读 | 大小、权限、时间戳、xattr、洞的位置全明文 |
| 指针内联 AEAD（ZFS / bcachefs） | 见 6.2，两家范围不同 | 能，MAC 存在指针里 | 见 6.2，两家差别极大 | 格式必须从一开始就留出 MAC / nonce 字段 |

bcachefs 文档给块层加密判死刑的理由是一句结构性的话：
> "at the block level there's nowhere to store MACs or nonces without causing painful
> alignment problems."

fscrypt 内核文档自己写明它不认证：
> "fscrypt is not guaranteed to protect confidentiality or authenticity if an attacker
> is able to manipulate the filesystem offline prior to an authorized user later
> accessing the filesystem."
理由是「ciphertext expansion 难处理」——即密文比明文长，块对齐的文件系统塞不下。

**对本工程的意义**：校验和跟着指针走（[decisions.md](decisions.md) D4（校验和位置））本身就把 MAC 和 nonce
的位置腾出来了。AEAD 对本工程是白捡的，对块对齐的文件系统不是。

### 6.2 ZFS 与 bcachefs 在同一个问题上分道扬镳：无 key 时还能不能维护

| | OpenZFS | bcachefs |
|---|---|---|
| 加密范围 | **只加密 level 0 块**；间接块（blkptr 树）不加密 | 除系统配置外全部加密 |
| 认证结构 | 间接层用 SHA512(下层 MAC 集合) 聚合；objset 层两个 256 位 MAC（portable / local） | Poly1305 MAC 逐块，chain of trust 直到系统配置 |
| 算法 | AES-CCM / AES-GCM，128/192/256 位；认证辅助用 SHA512-HMAC | ChaCha20 + Poly1305，按 RFC 7539 的路子直接用密码原语 |
| MAC 存哪 | `blk_cksum` 的后 128 位 | 元数据：明文头里的 128 位字段；数据：跟指针走 |
| nonce 存哪 | 96 位 IV：64 位在 `DVA[2]` 第二个 word，32 位在 `blk_fill` 高 32 位 | 元数据：明文头的序号；数据：key 的 96 位 version number 加派生量（见 6.3） |
| 无 key 能做 | **scrub / resilver / 重命名 / 删除 / raw send** | 基本没有——元数据本身要解密才能读 |
| 泄漏什么 | 池结构、数据集与快照名、数据集属性、文件大小、文件洞、dedup 表 | 只泄漏「这个位置是个 btree 节点/日志项」及其长度 |

**ZFS 为「无 key 可 scrub」付的价，在源码注释里写得很直白**：
`blk_cksum` 前 128 位留给**密文的截断校验和**，专供 scrub 使用，后 128 位才是 MAC。
而 MAC 树之所以不做成 Merkle 树，是因为 raw send 不传间接块，收端没有主密钥就重算不出：
> "Ideally, the cleanest solution would be to maintain a tree of authentication MACs
> going up the bp tree. However, this presents a problem for raw sends."
ZIL 与 dnode 还各有「必须留明文」的部分（claiming 与 scrubbing 要用），
这些明文部分靠 AEAD 的 AAD 机制认证。

**ZFS 的密钥层级**：主密钥不直接加密数据；64 位 salt + HKDF 派生工作密钥，
**每写 4 亿块换一次 salt**——注释按生日问题算出 96 位 IV 空间下要把碰撞概率压到 1/10^12，
上限是 398,065,730 块。主密钥用用户的 wrapping key 加密存盘，
所以换用户密钥不需要重写数据。

**bcachefs 拒绝 per-directory 加密的理由**（原文）：fsck 需要文件大小，
所以大小必须明文，否则就得把那部分 fsck 推迟到 key 提供之后；
filenames、xattrs、inline extents 各有类似麻烦。
> "With whole filesystem encryption, it's much easier to say what is and isn't encrypted."

**bcachefs 的 MAC 截断论证**（值得单独记，因为它把安全性押在一条实现约束上）：
数据 extent 的 Poly1305 MAC 默认截断到 **80 位**，省 8 字节/extent
（单副本 extent 32 字节 vs 40 字节）。论证是文件系统攻击者的伪造次数有界——
MAC 对不上就判设备故障并很快停用它，
> "attacker gets a very small (on the order of 10) attempts to forge a particular extent"
不接受这个前提的场景（例如经网络访问、已知会损坏数据的路径）要用 128 位 MAC。

**算法可换**：bcachefs 把加密做成「一种新的校验和类型」，
每个数据与元数据写都带算法字段，于是换算法不需要重写老数据。

### 6.3 nonce：加密对 COW 格式提出的真要求

**这是 bcachefs 设计文档里对本工程最有用的一段**，它记录的是一条被迫加进格式的字段。

- key 带 96 位 version number，加密开启时强制启用，每个新 extent 拿一个新号。
- extent 被部分覆盖或切分后重写时：**不能换号**（version number 有别的语义），
  **也不能沿用**（写的已经不是同一段数据）。
- 「跳 keystream 到活数据在原 extent 里的位置」这个办法遇上压缩就废：
  先丢 4K 再压缩 ≠ 先压缩再丢 4K 的输出。
- 最终解法是**给 extent 加一个「距原始 extent 起点的偏移」字段**，
  nonce 由 version number + 该偏移 + 当前大小 + 压缩算法拼出：
  > "offset + current size uniquely determines the uncompressed data, so, offset +
  > current size + compression function will uniquely determine the compressed output."

**崩溃后的 nonce 重用**：数据先用某个 version number 写盘，之后才把 extent 插进 btree，
所以不干净关机后存在「用过但 btree 里查不到」的号。bcachefs 文档说严格解法是
在 journal header 里记一个水位（小于它的号可能已被用过），**并注明尚未实现**；
当前实现是扫一遍现有最大号，把下一个号取到比它大 64k。

### 6.4 加密踩过的坑（公开记录）

| 坑 | 记录 | 口径 |
|---|---|---|
| ZFS 原生加密 + send/recv 长期损坏 | openzfs/zfs#12014 自 2021-05 开着；2024-02-12 Phoronix 报道；openzfs-docs#494 提议在文档里加警告；该 issue 下 2025-05-31 称修掉两个非 raw send 的加密 bug，进 2.2.8 / 2.3.3 | 公开 issue 与新闻报道，**未在本工程验证**。作为「加密会渗进复制/快照路径」的定性证据，不是性能或概率数据 |
| 压缩在加密之前必然泄漏长度 | Kelsey, *Compression and Information Leakage of Plaintext*, FSE 2002 | 顺序不可反（密文压不动），所以这个泄漏是选了压缩就必须接受的 |
| dedup + 加密泄漏相等性 | ZFS 用**明文的 HMAC** 当 salt+IV（前 64 位作 salt，后 96 位作 IV），保证同明文得同密文 | 源码注释自陈：攻击者能看出哪些块相同，「但 dedup 本来就泄漏这个」。另：加密下 dedup 只在 clone family 内生效 |
| 换用户密钥不等于旧密钥失效 | `zfs-load-key(8)`：旧的被包裹主密钥「accessible via forensic analysis for an indeterminate length of time」 | 手册页原文 |

来源：
- [bcachefs Encryption 设计文档](https://bcachefs.org/Encryption/)
- [OpenZFS `zio_crypt.c` 设计注释](https://github.com/openzfs/zfs/blob/master/module/os/linux/zfs/zio_crypt.c)
- [Linux fscrypt 内核文档](https://www.kernel.org/doc/html/latest/filesystems/fscrypt.html)
- [zfs-load-key(8)](https://openzfs.github.io/openzfs-docs/man/master/8/zfs-load-key.8.html)
- [OpenZFS 加密文档警告提案 openzfs-docs#494](https://github.com/openzfs/openzfs-docs/issues/494)
- [Phoronix：OpenZFS Native Encryption Use Raises Data Corruption Concerns (2024-02-12)](https://www.phoronix.com/news/OpenZFS-Encrypt-Corrupt)
- [Kelsey, Compression and Information Leakage of Plaintext (FSE 2002)](https://link.springer.com/chapter/10.1007/3-540-45661-9_21)

### 6.5 卷身份放在哪一层：ext4 与 fscrypt 都放在派生层，不放在逐单元输入里

**2026-08-29 本机内核树逐行现查，用于 D9（加密） 已定项 8 定案。**

| 实现 | 卷/文件身份进哪一层 | 逐单元输入里有什么 | 出处 |
|---|---|---|---|
| ext4 校验和 | `s_uuid` **挂载时折一次**进 `s_csum_seed` | 只有 `inum`、`gen`、块内容；`s_uuid` 一次也没出现 | `fs/ext4/super.c` 的 `ext4_init_metadata_csum`；`fs/ext4/ialloc.c` 的 `__ext4_new_inode`、`fs/ext4/bitmap.c` 的 `ext4_inode_bitmap_csum_verify`、`fs/ext4/super.c` 的 `ext4_group_desc_csum` |
| fscrypt 密钥 | 每文件 nonce **建 inode 时取一次随机数**，持久化在盘上的 context | 派生出的每文件密钥 | `fs/crypto/keysetup.c` 的 `fscrypt_prepare_new_inode` 里 `get_random_bytes(nonce, FSCRYPT_FILE_NONCE_SIZE)`；`fs/crypto/fscrypt_private.h` 的 `struct fscrypt_context_v2 { ...; u8 nonce[16]; }` |

⚠️ **fscrypt 不是「每次挂载换新盐」**——本工程曾在草稿里这么转述过，那是错的。
nonce 在**创建 inode 时**生成一次并写进盘上 context，此后每次挂载读回同一个值。
「每次挂载换新盐」这个形态如果真做了，上一次挂载写的块就永远解不开。

**对本工程的意义**：D9（加密） 把 fsid 从 AAD 移进 KDF，与这两家的分层一致——
卷/文件身份进派生层，对象身份进逐单元层。见 [decisions.md](decisions.md) D9（加密） 已定项 8。

### 6.6 明文系统配置里的水位字段可被回滚，这是有现役演示的失败模式

**2026-08-29 本机内核树现查。** dm-integrity 的 `recalc_sector` 住在明文系统配置里，
它的文档自陈：

> "legacy_recalculate — Allow recalculating of volumes with HMAC keys. This is disabled by
> default for security reasons - an attacker could modify the volume, set recalc_sector to
> zero, and the kernel would not detect the modification."
> （`Documentation/admin-guide/device-mapper/dm-integrity.rst`）

上游的处置不是「把水位算进 MAC」，而是**带密钥时默认禁掉那条路径**，
要开必须显式加 `legacy_recalculate`（`drivers/md/dm-integrity.c` 的
`dm_integrity_disable_recalculate()`）。

**对本工程的意义**：I-6.5（nonce 水位单调） 的 nonce 水位是同一个形状，而后果更硬——
回滚它不是「漏检一次篡改」，是**流密码 nonce 重用 ⇒ 两段明文直接 XOR**。
⇒ D9（加密） 已定项 8 定「水位必须被主密钥派生的 MAC 覆盖，无密钥侧不许写它」。

### 6.7 AEAD 的 tag 长度：内核允许截断到 32 位

`crypto_gcm_check_authsize()` 接受 4 / 8 / 12 / 13 / 14 / 15 / 16 字节
（`include/crypto/gcm.h` 的 `crypto_gcm_check_authsize()`，2026-08-29 现查）。
⇒ 「MAC 一律 128 位不截断」是**本工程自己的设计约束，不是密码学或内核强加的**；
它要付的空间代价（每单元 16 字节）应当按这个口径记，不能说成「没得选」。

## 七、近十年学术成果扫描（2012–2026）

**「七、近十年学术成果扫描（2012–2026）」只收「可能推翻某条既有决策前提」的成果，不做文献综述。** 每条注明它冲击哪条决策。
全部来自公开论文与项目文档，**未在本工程验证**。

### 7.1 随机小写：写优化索引把它变成非问题（冲击 D10（随机写 / 碎片化的真答案））

BetrFS 0.4 与主流文件系统在同一个随机小写微基准上的实测（FAST 2018 论文 Table 2）：

| 文件系统 | 256K 次 4 字节随机写耗时（秒，越低越好） |
|---|---|
| BetrFS 0.4 | 4.9 ± 0.3 |
| BetrFS 0.3 | 5.9 ± 0.1 |
| nilfs2 | 2013.1 ± 19.1 |
| btrfs | 2147.5 ± 7.4 |
| ext4 | 2776.0 ± 40.2 |
| xfs | 2835.7 ± 67.9 |
| zfs | 3288.9 ± 394.7 |

**口径**：256K 次 4 字节随机写、合计仅 1 MiB 写入量——这是**放到极端的微基准**，
不是数据库或虚拟机镜像的真实负载。论文自述「up to 600 times faster」。

**⚠️ 这组数字的持久化模型与 singlefs 不同，外推前必须先看这一段（2026-08-26 核实）。**
FAST 2018 论文 2.1 与 4.x 节原文：

> Crash consistency is ensured using **logical logging**, i.e., by logging the inserts,
> deletes, etc, performed on the tree. **Internal operations, such as node splits, flushes,
> etc, are not logged.** Nodes are written to disk using **copy-on-write**. At a periodic
> checkpoint (**every 5 seconds**), all dirty nodes are written to disk and the log can be trimmed.

> BetrFS ensures crash consistency by keeping a **redo log** of pending messages and applying
> messages to nodes copy-on-write. Crash recovery simply replays the redo log since the last checkpoint.

由此得到三条对本工程要紧的事实：

| 事实 | 对 singlefs 的含义 |
|---|---|
| BetrFS **是** COW 的——节点用 copy-on-write 写盘 | 「那个数字来自非 COW 系统」这个说法不成立，不能用它否掉 Bε |
| BetrFS 的持久化靠 **redo log + 每 5 秒一次 checkpoint** | 那 5 秒窗口把大量 COW 节点重写摊掉了。⚠️ **本工程并没有「每事务发布一次新根」这条承诺**——`litmus/commit-publish.litmus` 只钉一次发布事件**内部**的顺序（写块内容 → `smp_wmb` → 写 generation），对发布频率一字未提；`decisions.md` / `CLAUDE.md` / `.claude/rules/fs-design.md` 全文均无此表述（2026-08-26 逐文件 grep 核实）。发布频率是**尚未决定**的事项 |
| 全文**没有出现 checksum**，不存在「父节点存子节点校验和」的 Merkle 链 | singlefs 的 D4（校验和位置） 内联校验和 + D9（加密） 的 MAC/nonce 给每次节点重写加了一份 BetrFS 不付的固定成本，交叉点因此会移动，移多少未知 |

**所以这组数字能证明的是**：消息缓冲区能把「一个 key 要被重写几次」从 O(树高) 降到 O(树高/B)。
**它不能证明的是**：在「Merkle 校验和链 + AEAD」都在、且发布频率取某个具体值的前提下还剩多少收益。
还剩多少收益要靠 [experiments.md](experiments.md) E7（离线索引 harness） 自己量。

⚠️ **发布频率是这里的自由变量，不是常量。** 若本工程采用 checkpoint 语义
（内存里攒脏节点、定期合并写一次），那么「一次小写不必立刻重写整条从叶子到根的路径」
这个效果，**checkpoint 批量本身就免费给了**，不需要在格式里放一个持久化的消息缓冲区。
BetrFS 那个数字里有多少来自消息缓冲、多少来自 5 秒 checkpoint，**论文没有拆开过**。
E7（离线索引 harness） 的对照组因此必须区分两种「无消息缓冲」基线：完全无批量，
以及有 checkpoint 批量但无格式级消息缓冲。两者混为一谈会让消息缓冲显得比实际更必要。

机制是把小写变成 Bε-tree 的消息批量下刷，而不是原地改块，也不是关掉 COW。

对 D10（随机写 / 碎片化的真答案） 的意义：btrfs 的 `nodatacow` 是「关掉 COW 换随机写性能，代价是同时丢校验和与快照一致性」，
而这条路是「把随机写吸收进索引的消息层」，校验和与快照都不动。
**这是 D10（随机写 / 碎片化的真答案） 唯一一条有实测数字支撑的候选方向。**

**另一条顺带核实到的事实**：BetrFS 的所有版本都用**两棵** Bε-tree——
一棵放文件数据，一棵放文件系统元数据（论文 2.2 节原文：
「All versions of BetrFS use two Bε-trees: one for file data and one for file system metadata」）。
这与 D8（核心索引结构）「多个独立 keyspace」是同一个思路，只是数量少得多（bcachefs 是 28 棵）。

### 7.2 老化不是宿命，可以在结构层面设计掉（冲击 D3（空间分配）、D10（随机写 / 碎片化的真答案））

FAST 2017「File Systems Fated for Senescence? Nonsense, Says Science!」用
**连续 git checkout Linux 内核源码**把文件系统做老，测 btrfs / ext4 / F2FS / XFS / ZFS 与 BetrFS。
结论：**老化后的 BetrFS 甚至胜过其余文件系统未老化时的表现（btrfs 除外）**。
另一条被 FAST 2018 引作动机的数字：git 版本控制负载可把 ext4 的 scan 性能退化至多 **15 倍**。

对本工程：D3（空间分配） 承诺常驻自动整理，理由是「不分类 → 碎片压力集中在一个分配器」。
这条成果给出第二条路——**老化可以被索引结构本身吸收掉，不必全靠事后整理**。
两条路各要多少代价可测，判据见 [experiments.md](experiments.md) E10（老化：结构抗老化 vs 事后整理）。

### 7.3 索引结构：参数空间比「换一种树」大得多（冲击 D8（核心索引结构））

| 成果 | 结论 | 口径 |
|---|---|---|
| SplinterDB（ATC 2020） | STBε-tree = Bε-tree 加 LSM 的 size-tiering，做成「树的树」：主干 trunk 树的每个节点指向一组 B-tree 分支，每个分支带一个 quotient filter。相对 RocksDB：插入快 6–10×、点查快 2–2.6×、写放大降 2× | 论文摘要数字，面向 NVMe SSD |
| FAST 2022「Closing the B+-tree vs. LSM-tree Write Amplification Gap」 | 在带内建透明压缩的硬件上，Bε-tree 写放大 28（8KB 页）/ 36（16KB 页）；同条件 RocksDB 38、WiredTiger 268（8KB 页）/ 530（16KB 页） | 500GB 数据集、32B 记录、4 客户端线程 |
| WiscKey 式 key-value 分离 | 大 value 移出索引、只在索引里留指针，compaction 只搬 key 与元数据，写放大最多降两个数量级。工业界已落地为 RocksDB BlobDB、Titan、TerarkDB | 论文 + 工业实现 |
| HashFS（FAST 2021「Rethinking File Mapping for Persistent Memory」） | 文件映射本身可占 IO 路径的 **70%**；哈希式映射相对「页缓存里的 extent 树」把 LevelDB 上的 YCSB 吞吐提高至多 45% | **持久内存场景**；本工程若不针对 PM 只作参考 |
| TableFS（ATC 2013） | 元数据全部塞进 LSM（LevelDB），元数据密集负载相对 ext4 / XFS / btrfs 快 50%–1000% | FUSE 实现，作者自陈实现低效，即便如此仍赢 |

对 D8（核心索引结构） 的意义：D8（核心索引结构） 已定「一套 btree 实现 + 多 keyspace + write buffer / key cache 两个前端」。
这几条不否定该方向，但把**同一套结构内部的可调项**显著扩大了：
Bε 缓冲比例、size-tiering、key-value 分离、过滤器类型——
都是一套 btree 里的参数，不是「再加一种树」。
这与 D8（核心索引结构）「记账树、中央映射树、树表、实例表、inode 树仍照一套 btree 做」不冲突，反而是它的补充（分配记录树与 extent 树已按已定项 14 改成可以异构，不在此列）。

### 7.4 崩溃一致性验证：「没有 oracle」这条前提需要重估（冲击验证计划）

[CLAUDE.md](../../CLAUDE.md) 记着本工程最大的隐性成本是「从零设计没有参照实现可比对」。
近十年至少有五类现成办法，各自给的东西不同：

| 办法 | 代表 | 它给什么 | 代价 / 前提 |
|---|---|---|---|
| 按钮式形式验证 | Yggdrasil（OSDI 2016，Best Paper） | **crash refinement** 正确性定义：实现能产生的磁盘状态集合（含崩溃产生的）必须是规范允许集合的子集。Z3 自动验，**无需人工标注或证明** | 实现要写成可符号执行的形态；论文称把逻辑表示与物理表示分离后验证时间降到一分钟内 |
| 有界黑盒崩溃测试 | CrashMonkey + ACE（OSDI 2018） | 按有界参数穷举工作负载，每个崩溃点后检查恢复。65 机跑两天：复现 24 个已知 bug、找到 10 个新 bug（7 个自 2014 年就在内核里）；**并且在已形式验证的 FSCQ 上也找到一个崩溃一致性 bug** | 只覆盖有界空间；对「界外」什么也不说 |
| PM 专项同类框架 | Chipmunk（EuroSys 2023 最佳论文之一） | 同类方法用于持久内存文件系统，在 5 个 PM 文件系统里找到 23 个新 bug，后果含无法挂载、rename 原子性被破坏 | 针对 PM 语义 |
| 模型检验 + 差分对拍 | Metis（FAST 2024） | 把模型检验的状态空间穷举与差分测试结合，**不需要抽象模型**：拿另一个文件系统当参照，逐操作比对抽象状态、系统调用返回值与 errno。配套写了 **RefFS**——一个专为「加速模型检验、提高 bug 可复现性」而设计的小而快的参照文件系统 | 需要一个可信参照实现 |
| 并发 + 崩溃的机器检查证明 | GoJournal（OSDI 2021）/ DaisyNFS（OSDI 2022） | GoJournal 是验证过的并发崩溃安全日志层（Coq / Perennial 2.0），**验证过程中揪出一个大量单测都没抓到的严重并发 bug**；DaisyNFS 在 GoTxn（GoJournal 加两阶段锁与分配器）之上，用 Dafny **顺序地**验证每个 NFS 操作 | 证明成本高 |

**对本工程最直接的两条**：

1. **Metis 的 RefFS 说明「参照实现」不必是别人的成熟文件系统，可以是为对拍专门写的小实现。**
   这正是 `.claude/singlefs-ai-sop/rules/test-discipline.md` 说的模型对拍，
   但 Metis 多给了一件事：把它和状态空间穷举接起来，而不是只喂随机操作序列。
2. **DaisyNFS 的分层与 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」同构**：
   事务层一次把并发与崩溃解决掉，上层就只剩顺序正确性要验。
   这条给那条纪律加了一个本工程之外的独立理由。

### 7.5 提交协议：排序未必要靠 flush 换（冲击事务层设计）

OptFS（SOSP 2013）把 ext4 的日志提交改成乐观协议，三件事：
**事务校验和**（把数据块也纳入校验，恢复时校验和对不上就整笔丢弃）代替提交时排序；
**异步持久化通知**（asynchronous durability notification）代替阻塞式 flush；
把顺序与持久拆成两个 API（`osync()` / `dsync()`）。论文称部分负载提升达一个数量级。

NoFS（FAST 2012，「Consistency Without Ordering」）走得更远：
每个数据块带**反向指针**，读时靠反向指针判一致性，**完全不排序写**，并给了形式化模型。

对本工程：`litmus/commit-publish.litmus` 钉住的是一次发布事件内部「先写块内容、再写系统配置」这条序，
不涉及发布频率。
这两条成果说明该序有替代形态，代价各不相同。
⚠️ **NoFS 的反向指针与 D1（数据可移动性 / 反向索引） 的反向索引是两件不同的东西**：
NoFS 的是「每块自证属于谁」的一个字段，D1（数据可移动性 / 反向索引） 的是「物理块被谁引用」的可查询索引。
本工程「元数据块必须自描述」（`.claude/rules/fs-design.md`）已经接近 NoFS 那个字段。

### 7.6 硬件接口：块接口不再是唯一的对接层（冲击 D2（RAID 条带策略）、D3（空间分配）、D8（核心索引结构） 节点大小）

| 接口 | 状态 | 对本工程 |
|---|---|---|
| ZNS（Zoned Namespace） | 顺序写、由主机管 GC；因成本效率在规模部署中被采用 | zone 大小是 D8（核心索引结构）「按设备特性选节点大小」那条规则的输入 |
| NVMe FDP（Flexible Data Placement） | 合并了 Google SmartFTL 与 Meta Direct Placement Mode 两个提案，填 ZNS 与普通 SSD 之间的成本收益空档。**不要求顺序写、向后兼容，应用不改也能跑** | 比 ZNS 更可能是本工程该对接的那一层：不强制顺序写，与 COW 的分配自由度不打架 |

口径：NVM Express 与 SNIA 公开材料、2025 年 arXiv 论文，**未在本工程验证**。
两者共同点是把数据放置与 GC 的责任交给主机——**这与 COW 文件系统自己就管分配是重叠的**，
重叠部分是收益还是双重管理的代价，需要实测。

### 7.7 Rust 在内核文件系统里的现状（冲击工程路线，不冲击格式）

| 事实 | 来源与日期 |
|---|---|
| Bento（FAST 2021）：safe Rust 写内核文件系统，错误基本被沙箱在文件系统内。性能与 VFS 原生 ext4 相当，比 FUSE 版快 7×（`git clone`）到 90×（Filebench）；**支持在线升级，服务中断约 10ms**，替换运行中的文件系统对应用无中断 | 论文与项目页 |
| Greg Kroah-Hartman 2026-07-15 表态：Rust 在内核已不是实验，是永久组成部分；图形子系统今后只接受 Rust 写的新驱动 | 二手报道，**未核对原始邮件** |
| Rust binder 驱动进 Linux 6.18；PuzzleFS 列在 Rust for Linux 项目清单内 | 二手报道 |

对 D7（是否进 Linux 主线）已定的不进主线：Bento 的在线升级能力值得注意——它把「文件系统要不要在内核里」
从二选一变成了三选一（用户态 / 内核 C 模块 / 可热替换的内核 Rust 模块）。

### 7.8 bcachefs 现状更新（加强 D7（是否进 Linux 主线） 的依据，不改结论）

| 事实 | 日期 |
|---|---|
| bcachefs 在 Linux 6.17 被标记 externally maintained，**代码在 6.18 被从主线移除** | 2025–2026，公开报道 |
| 改为 DKMS 模块分发，形态与 OpenZFS on Linux 相同；开发者改向独立邮件列表投 patch | 同期 |
| bcachefs-tools v1.38.6 **去掉 experimental 标签** | 2026-06-17 |

对 D7（是否进 Linux 主线）已定的不进主线：这条把依据从「bcachefs 被移出主线是社交失败不是技术失败」
加强为「**移出主线之后项目仍在正常演进，并且在这之后才去掉实验标签**」——
主线之外是一条可行的分发路径，不只是失败后的退路。

### 7.10 bcachefs 官方文档补录（2026-08-26 现查，Principles of Operation Rev 1.39.2+2202696）

| 事实 | 原文 / 口径 | 对本工程 |
|---|---|---|
| **bucket gen 的适用范围限定为缓存数据** | §1.5：「we can reuse a bucket with **cached data** in it without finding and deleting all the data pointers by incrementing the generation number」 | [decisions.md](decisions.md) D1（数据可移动性 / 反向索引）：它只管失效不管搬运 |
| **gen 绕回机制已被反向索引取代** | 状态清单 `need_gc_gens`：「Legacy state, retained for compatibility … **now effectively unused since the invalidate worker uses backpointers instead of generation bumping**」 | 与 D1（数据可移动性 / 反向索引） 已选的「反向索引 day-1」同向 |
| **copygc 预留 8%，可配 5–20%** | §1.5 与 §9.1.10。且「Normal writes cannot dip into this reserve」，超过约 90% 容量时写延迟上升 | D3（空间分配） 的「空间预留用准入控制形态，量要小」有了一个可对比的量级：**别家是划 8% 不给用**，本工程选的是动态准入，差异要记明 |
| **加密是全有全无，且只能在 mkfs 时开** | §2.1.2：「Encryption is all-or-nothing at the filesystem level: all data and metadata except the superblock is encrypted, and all data and metadata is authenticated … **Encryption can only be enabled at format time; it cannot be added to an existing filesystem**」 | 印证 D9（加密） 的方向；但注意本工程 D9（加密） 预留了「系统配置的 KDF 标识 + 主密钥槽」，目标是**能在既有文件系统上开加密**，这是与 bcachefs 的一处有意分歧 |
| ⚠️ **`nocow` 写的数据在加密文件系统上是明文存储** | §2.1.2：「Data written with the `nocow` option is stored **unencrypted**, even on an encrypted filesystem. **This is a hard design incompatibility, not a policy choice**: ChaCha20 requires a unique nonce per (key, plaintext), and bcachefs stores the nonce externally alongside each data pointer」 | **这是 D9（加密） 与 D10（随机写 / 碎片化的真答案） 连锁那一条的实证**：本工程已定「任何绕过 COW 的快路径不许绕过加密；原地覆盖写会直接造成 nonce 重用」。bcachefs 遇到同一个约束，选择是**让 nocow 数据不加密**——本工程不接受这个交易，因此 D10（随机写 / 碎片化的真答案） 的任何候选都不许走原地覆盖写这条路 |
| **挂载前必须解锁** | §4.2.3：「the passphrase is requested once the devices have been found and **before any attempt to mount**」；FAQ：未解锁挂载报 `Required key not available` | 是 D9（加密） 已定项 5 那条推理的前提之一 |

**未找到的（明说，不补）**：没有任何一手材料用散文明说
「copygc / rebalance / scrub / fsck 在无密钥时不能跑」。
「无密钥后台整理瘫痪」是从「7.10 bcachefs 官方文档补录」表里几条推出来的，**是推论不是引文**，见 decisions.md D9（加密）。

来源：[bcachefs Principles of Operation (PDF)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)、
[bcachefs.org/Encryption](https://bcachefs.org/Encryption/)、
[bcachefs.org/FAQ](https://bcachefs.org/FAQ/)、
[The Programmer's Guide to bcache](https://bcache.evilpiepirate.org/BcacheGuide/)、
[Kent Overstreet: Bcachefs - encryption, fsck, and more (LWN 镜像)](https://lwn.net/Articles/717378/)、
[Bringing bcachefs to the mainline (LWN, LSFMM 2022)](https://lwn.net/Articles/895266/)。

### 7.11 bcachefs 的记账验证有一处自我作废（2026-08-26 源码逐字核实，Linux v6.17）

**这一节记的是一个已经发生的失效，不是推测。**

bcachefs 的 `bch2_check_allocations()` 用全量遍历重建记账、再与运行时计数器 `memcmp` 比对——
**形态与本工程 [invariants/05-空间记账.md](../../crates/singlefs-checker/invariants/05-空间记账.md) 的 I-3.1（已分配统计对得上） 相同**。但两侧共用同一段加减代码：

```c
this_cpu_add(e->v[gc][i], a.v->d[i]);        // fs/bcachefs/disk_accounting.h，bch2_accounting_mem_mod_locked()
```

`gc` 是唯一的差别——运行时写 `v[0]`，GC 重建写 `v[1]`。
上游的 `__trigger_extent()` 同样只靠一个 `bool gc = flags & BTREE_TRIGGER_gc;` 分流。
`btree_gc.c` 的注释自陈重建侧「is not idempotant; we'll calculate the wrong result
if we run it multiple times」——**重建侧本身就是一串 `+=`**。

| 它能抓到的 | 它抓不到的 |
|---|---|
| 增量**应用过程**出偏差：漏一次更新、并发丢失、崩溃后未重放 | **加减语义本身写错**：某分支符号搞反，运行时与重建一起错到同一个值，`memcmp` 全绿 |

**对本工程**：`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 写着「校验的两条路径不共享
同一段代码、同一次采样、同一个工具」。这一节是那条规则在真实系统里**已经被违反并产生盲区**的实例。
处置见 [checks-owed.md](checks-owed.md) C12（增量语义共用）。

**顺带记下的记账形态事实**（同批核实）：accounting 树在 `bcachefs_format.h` 的 `BCH_BTREE_IDS()` 里被
**格式级**绑定 `BTREE_IS_write_buffer`；`disk_accounting_format.h` 原文
「Unlike with other key types, **updates are _deltas_**」；
合并函数是 `dst->v.d[i] += src.v->d[i]`；
write buffer 对 accounting 与普通 key 在入 buffer、flush 去重、落 btree 三处显式分叉；
因为 Δ 非幂等，它必须另造一套 **bversion 全序**判断某条 Δ 有没有被应用过，
且 journal replay 未完成时 accounting key 一律不许 flush。
**「缓冲里放增量」派生出的成本，这几条就是清单。**

来源：Linux mainline v6.17 `fs/bcachefs/`（`disk_accounting.{c,h}`、`disk_accounting_format.h`、
`btree_write_buffer.{c,h}`、`btree_trans_commit.c`、`btree_gc.c`、`buckets.c`、`bcachefs_format.h`），
现拉并逐字比对。

## 八、根环槽宽与「长时间消费者怎么跟发布赛跑」这两件事，现役实现怎么做

**2026-09-08 在本机固定点 `~/code/fs-refs` 现查逐字抄出。均未在本项目验证。**
它们回答的是 D2（RAID 条带策略） 已定项 15（「生命周期不同的对象」的通用判据）
与 [checks-owed.md](checks-owed.md) C214（遍历跳的窗口余量没人守）。

### 8.1 ZFS：根环槽宽确实随设备粒度抬，但封顶 8 KiB，而且抬槽宽是减槽数不是多占地

来源 `zfs/include/sys/vdev_impl.h` 里 `VDEV_UBERBLOCK_RING` 到 `VDEV_UBERBLOCK_SIZE(vd)` 那一段宏与 `zfs/include/sys/uberblock_impl.h` 的 `UBERBLOCK_SHIFT`，逐字：

```c
#define	VDEV_UBERBLOCK_RING	(128 << 10)
/* The largest uberblock we support is 8k. */
#define	MAX_UBERBLOCK_SHIFT (13)
#define	VDEV_UBERBLOCK_SHIFT(vd)	\
	MIN(MAX((vd)->vdev_top->vdev_ashift, UBERBLOCK_SHIFT), \
	    MAX_UBERBLOCK_SHIFT)
#define	VDEV_UBERBLOCK_COUNT(vd)	\
	(VDEV_UBERBLOCK_RING >> VDEV_UBERBLOCK_SHIFT(vd))
```

`UBERBLOCK_SHIFT` 逐字 `10			/* up to 1K	*/`；`vdev_ashift` 的字段注释逐字 `/* block alignment shift	*/`。

**三件事一起读出来**：

| 读出什么 | 逐字依据 |
|---|---|
| **槽宽跟设备粒度走**，不是固定常量 | `MAX((vd)->vdev_top->vdev_ashift, UBERBLOCK_SHIFT)` |
| **但封顶 8 KiB** ——不会因为阵列的条带块大就把槽抬到那么大 | `MAX_UBERBLOCK_SHIFT (13)` 与它上方那句注释 |
| **环的总大小固定 128 KiB ⇒ 抬槽宽等于减槽数**，不是多占盘 | `VDEV_UBERBLOCK_COUNT = VDEV_UBERBLOCK_RING >> SHIFT` |

⚠️ **它用的量是 `ashift`（对齐 / 最小分配单元），不是 Linux 的 `io_min`（阵列上是条带块大小）。**
两个量在 md 阵列上差很远——E34（根环槽几何） 在真 md/raid5 上量到 128 倍。

⚠️ **与本工程的一处已知差异（`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 要求写下来）**：
ZFS 的 label 是每盘 4 份、每份 256 KiB 的**固定区域**，uberblock 环恒占其中 128 KiB；
而本工程的根环是 R 个区域按素数步长撒在盘上，**区域多大不是格式常量**——
每区槽数 S 住系统配置（D22（单元原子性怎么合成） 已定项 2 逐字「S 住系统配置。**下界 1、上界由
I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**」）。⇒ 「抬槽宽 = 减槽数」这个等式在本工程
**要先把「区域大小固定」写成条款**才成立，而仓里没有这条。

### 8.2 btrfs：靠走 commit root + 可暂停，而这条路在本工程被两道独立的闸排除

来源 `linux-6.17/fs/btrfs/`，2026-09-08 现查，逐字：

| 位置 | 逐字 |
|---|---|
| `scrub.c` 的 `scrub_setup_ctx` 里两处 | `sctx->extent_path.search_commit_root = 1;` / `sctx->csum_path.search_commit_root = 1;` |
| `scrub.c` 的 `scrub_enumerate_chunks` | `path->search_commit_root = 1;` |
| `transaction.c` 的 `switch_commit_roots` 里一前一后 | `down_write(&fs_info->commit_root_sem);` / `up_write(&fs_info->commit_root_sem);` |
| `ctree.c` 的 `finish_need_commit_sem_search` | `lockdep_assert_held_read(&lowest->fs_info->commit_root_sem);` |
| `transaction.c` 的 `btrfs_commit_transaction` | `btrfs_scrub_pause(fs_info);` |

⚠️ **本工程用不了这条路，两道排除各自独立（2026-09-08 现查坐实）**：
① **提交间隔差 8.3 万倍**——btrfs 默认提交间隔逐字
`#define BTRFS_DEFAULT_COMMIT_INTERVAL	(30)`（`linux-6.17/fs/btrfs/fs.h`，30 秒），
而本工程甲下发布间隔是 **0.36 毫秒**（2785 发布/秒）。btrfs 那套的隐含前提是
「提交间隔远大于消费者的一段扫描」；搬过来的话 scrub 每秒要被停 2785 次，永远走不完一遍。
**这条前提从没跟着那个做法一起被引用过。**
② **checker 那一侧结构上拿不到那把锁**——D13（验证路线） 已定项 5 逐字
「只共享一份从 kb 生成的常量模块；newtype、解析、校验和、遍历、记账一律各写一份」
⇒ 独立 checker 不可能持有被测实现的 `commit_root_sem`。

三件套那一段记下来是为了说清被排除的是什么，不是候选出路。

⇒ **三件套**：scrub 读的不是活树而是 **commit root**（上一个已提交事务的根）；
提交路径切换 commit root 时拿 `commit_root_sem` 的**写锁**、遍历方拿读锁，两者互斥；
且提交路径**显式把 scrub 停下来**（`btrfs_scrub_pause`），不是让它带着一个即将失效的根继续走。

**这是一条与「把保护窗口拉长到覆盖扫描时长」完全不同的路**：
它不动块重用的代数窗口，而是把消费者做成**可暂停、且与提交点同步**的。

⚠️ **与本工程的一处已知差异**：btrfs 的 commit root 只有**一代**（上一个已提交事务），
靠内存里的读写锁维持，因此它只覆盖**运行期**那一半；
而 I-7.4（近 K 代块未被复用） 逐字写着「崩溃后 defer 队列的内存态丢失，运行时的 K 由根环深度接管
（清扫准入「释放代 ≤ 环里最旧根 txg」那条闸），保护宽度随之变」——
**崩溃之后那一半，锁一个字都说不上话。**

<!-- doc-lint:not-numbers EXT4 NILFS2 -->

## 九、每对象扩展槽：五家现役实现怎么放「属于这个对象、但不在主字段里」的数据（D21（权威态与派生态的分界） 已定项 3 ③ 的外部调研，2026-09-13 现查）

D21（权威态与派生态的分界） 已定项 3 ③ 逐字「用户要求的外部调研（别家怎么做每对象扩展槽）还没做」。五家全部现查原文（URL 在表里），**未在本项目验证**；按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`，它们只能提假设、指该测的路径，不进正推 / 反推 / 校验。每家都写一处与本工程的已知差异。

| 实现 | 扩展槽住哪 | 形态与宽度 | 装不下时 | 来源（现查） | 与本工程的一处差异 |
|---|---|---|---|---|---|
| ext4 | inode 记录里固定字段之后的空间（`i_extra_isize` 记「Size of this inode - 128」，默认 256 字节 inode 里固定部分 160 字节，余下装 xattr） | 头 `ext4_xattr_ibody_header` 4 字节（magic 0xEA020000）+ 条目 `ext4_xattr_entry`（`e_name_len`、`e_name_index`、`e_value_offs`、`e_value_inum`、`e_value_size`、`e_hash`），值 4 字节对齐；内联数据是一条名为 `system.data` 的 xattr（`EXT4_INLINE_DATA_FL`） | 溢出到 `i_file_acl` 指向的一个独立块（「it is not possible for this block to contain a pointer to a second extended attribute block」）；值再大用 EA inode（`e_value_inum`，`EXT4_EA_INODE_FL`） | https://www.kernel.org/doc/html/latest/filesystems/ext4/inodes.html 、 https://www.kernel.org/doc/html/latest/filesystems/ext4/attributes.html | 槽在 inode 记录内、可按名字装任意键值；本工程的扩展点按 D1（数据可移动性 / 反向索引） 已定项 3「要么为空，要么是一个指向」，不内联数据 |
| XFS | inode 字面区（`XFS_LITINO` = inode 大小 − 核心）由数据叉与属性叉共用，`di_forkoff`（「attr fork offs, <<3 for 64b align」，单位 8 字节）定属性叉起点；`di_aformat` 取 `XFS_DINODE_FMT_LOCAL / EXTENTS / BTREE` | 短形式：`xfs_attr_sf_hdr { totsize u16, count u8, padding u8 }` + `xfs_attr_sf_entry { namelen u8, valuelen u8, flags u8, nameval[] }`，「packed as tightly as possible so as to fit into the literal area of the inode」 | 属性叉换成 extents / btree 格式指向叶块、节点块、远程值块 | https://raw.githubusercontent.com/torvalds/linux/master/fs/xfs/libxfs/xfs_format.h 、 https://raw.githubusercontent.com/torvalds/linux/master/fs/xfs/libxfs/xfs_da_format.h | 叉起点是每 inode 一个可变偏移（`di_forkoff`）；本工程的扩展点起点由系统配置声明的 N 与单元结构隐含，每单元不另存偏移 |
| btrfs | fs 树里按 objectid 挂的条目：`XATTR_ITEM`（0x18，key 里带名字哈希）、`DIR_ITEM`（0x54）、`EXTENT_DATA`（0x6c，内联时「the remaining item bytes are the data bytes」） | 叶内变长条目：「header \| item 0 \| item 1 \| … \| free space \| data N \| … \| data 0」；内联数据上限 `max_inline` 「default: min(2048, page size)」，4 KiB sectorsize 时约 3900 字节 | 同哈希的 xattr 必须装进一个叶；数据超过 `max_inline` 走外部 extent | https://btrfs.readthedocs.io/en/latest/dev/On-disk-format.html 、 https://btrfs.readthedocs.io/en/latest/Administration.html | 扩展数据是索引树里的独立条目、可变长；本工程放在单元自描述头之后的定长配额里 |
| ZFS | dnode（`DNODE_SHIFT 9`，512 字节）核心 64 字节之后的 bonus 缓冲：`DN_BONUS_SIZE(dnsize) = dnsize − DNODE_CORE_SIZE − (1 << SPA_BLKPTRSHIFT)`（512 字节 dnode 时 320），`dn_bonustype u8`、`dn_bonuslen u16` | bonus 装系统属性（SA）；类型由 `dn_bonustype` 声明 | 「Spill blocks are used to store system attribute data (i.e. file metadata) that does not fit in the dnode's bonus buffer」，`DNODE_FLAG_SPILL_BLKPTR (1 << 2)`，spill 指针占掉 bonus 末尾的一个 blkptr | https://raw.githubusercontent.com/openzfs/zfs/master/include/sys/dnode.h | 每对象槽宽是 dnode 大小减核心（可到 1 KiB 以上的 dnode）；本工程扩展点配额 N 由每条线在系统配置声明、第一版为 0 |
| APFS | inode 记录值 `j_inode_val` 末尾的可变长扩展字段：`apfs_xf_blob { xf_num_exts u16, xf_used_data u16, xf_data[] }`，每个字段 `apfs_x_field { x_type u8, x_flags u8, x_size u16 }`；类型如 `DSTREAM = 8`、`NAME = 4`、`FINDER_INFO = 7`、`SPARSE_BYTES = 13` | 每字段 4 字节元数据 + 值；xattr 值 `apfs_xattr_val { flags u16, xdata_len u16, xdata[] }` | 嵌入上限 `APFS_XATTR_MAX_EMBEDDED_SIZE = 3804`（`XATTR_DATA_EMBEDDED = 2`），更大的走数据流（`XATTR_DATA_STREAM = 1`） | https://raw.githubusercontent.com/linux-apfs/linux-apfs-rw/master/apfs_raw.h （Apple 的 APFS Reference PDF 用 CID 字体，本机 `pdf-text.py` 抽不出文字，改核 Linux 驱动头文件） | 类型 + 长度的自描述字段列表，可装数据；本工程的扩展点只装一个指向 |

**五家共有的形态**：都是「对象记录内一段有上限的空间 + 装不下时指向外部」，上限 320 字节（ZFS 512 字节 dnode）到约 3.9 KiB（btrfs 一叶、APFS 3804）。**没有一家把扩展槽做成「只许一个指向、不许内联」**——这是本工程 D1（数据可移动性 / 反向索引） 已定项 3 独有的形态，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「没有任何现役实现走 X 这条路」是最值钱的一类外部信号：举证责任在本工程这边（理由记在 D1（数据可移动性 / 反向索引） 已定项 3 与 D21（权威态与派生态的分界） 硬约束 5）。
**「为空」怎么编码，五家各自的做法**：ext4 靠 `e_value_size = 0` 与条目计数；XFS 靠 `count = 0`；btrfs 靠条目不存在；ZFS 靠 `dn_bonuslen = 0`；APFS 靠 `xf_num_exts = 0`——**都是「计数为 0」而不是「某个全零指针」**，与 D21（权威态与派生态的分界） 已定项 3 ②「全零不能直接当空用」那句同向。

## 十、快照下的死块记账、搬被共享的块与 discard 时机

取得日期 2026-10-12；每条未在本项目验证，是线索不是证据（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「别的项目怎么做，是线索不是证据」）。

**查的版本与取法**

| 对象 | 版本 / 提交 | 取法 |
|---|---|---|
| OpenZFS master | `4542a317e2fa524e24c1c28b6fbca9cd3634ba56`（2026-10-09，`git ls-remote` 与克隆 HEAD 一致） | `git clone --depth 1 --filter=blob:none --sparse`，检出 `module/zfs include/sys man cmd/zpool cmd/zfs` |
| Linux 主线 | `24a19b953e2a3d0ea5a8c4951a0ab8f32720c907`（Linux 7.3-rc7，2026-10-11） | `--depth 1 --sparse`，检出 `fs/btrfs fs/f2fs fs/nilfs2 include/uapi/linux Documentation/filesystems` |
| bcachefs | bcachefs-tools `efec39c7e8d1dda11fb16bad0e1e75d86ead3105`（2026-10-10）的 `fs/` 目录 | `--depth 1`。bcachefs 已不在主线：主线 HEAD `git ls-tree HEAD fs/ \| grep -c bcachefs` 输出 `0`；git.kernel.org 上 `fs/bcachefs/Kconfig` 在 v6.17 返回 200、v6.18-rc1 返回 404 |
| nilfs-utils | master `26efb5daff0757365101035145331b0a5a85d9d9`（ls-remote HEAD） | raw.githubusercontent 取 `lib/gc.c`、`man/nilfs_cleanerd.conf.5` |
| APFS | Apple File System Reference，PDF 元数据 CreationDate 2020-06-22，181 页 | developer.apple.com 下载，pypdf 抽文本 |
| btrfs 文档 | btrfs.readthedocs.io/en/latest 的 Trim、btrfs-quota、Qgroups、btrfs-balance、btrfs-man5 页 | curl 取 |

文件:行号一律指「查的版本与取法」表里的提交。本机原有的 `/home/fy5090/code/fs-refs/zfs`（`58d73c90`，2026-08-27）与 `/home/fy5090/linux-bug-fix/linux`（v7.3-rc1，浅克隆）比主线旧，没有拿来引。

本工程一侧拿来写差异的事实只有两条，都是现查的：`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 的「已定项 5：位置条目降为提示，中央映射是唯一入口」与「已定项 7：指针头部带出生 (树, txg)，各 8 字节」（简称「本工程走中央映射」「本工程指针带出生 txg」）。

### 10.1 ZFS：deadlist、livelist、为什么没有 BP rewrite、device removal 的 indirect mapping

| 事实 | 出处（OpenZFS `4542a317`） | 口径 | 与本工程的差异 |
|---|---|---|---|
| 内存结构：deadlist 是一棵按 `dle_mintxg` 排序的 AVL 树，每个条目带一个 bpobj；另有 `dl_cache`（按 mintxg 的字节数缓存）；盘上头 `dsl_deadlist_phys_t` 只存 used/comp/uncomp 三个总数 | `include/sys/dsl_deadlist.h` 中 `dsl_deadlist_phys_t`、`dsl_deadlist_t`、`dsl_deadlist_entry_t` 定义；`module/zfs/dsl_deadlist.c:106`（`dsl_deadlist_compare` 比 `dle_mintxg`） | 读源码 | 按出生 txg 分桶的 deadlist 这一做法：ZFS 的块指针是物理 DVA、没有中央映射；本工程走中央映射，指针带出生 txg |
| 键在盘上：deadlist 对象是一个 ZAP，key 是 mintxg（十进制串），值是 bpobj 对象号 | `dsl_deadlist.c:534`（`zap_add_int_key(... mintxg, obj ...)`）、`:895-897`（merge 时 `zfs_strtonum(za->za_name)`） | 读源码 | ZFS 的块指针是物理 DVA、没有中央映射；本工程走中央映射，指针带出生 txg |
| 入桶规则：被杀的块按 `BP_GET_BIRTH(bp)` 找 mintxg 严格小于它的最大键入桶（精确命中就退一格，「mintxg is not inclusive」） | `dsl_deadlist.c:477-491`；`:511-514` 注释 | 读源码 | ZFS 的块指针是物理 DVA、没有中央映射；本工程走中央映射，指针带出生 txg |
| 何时进 deadlist：`dsl_dataset_block_kill` 里 birth > `ds_prev_snap_txg` 的块直接释放（gang/dedup/BRT 块进池级 free bpobj 延后），否则进 head 的 deadlist；在 zio 完成回调里时先进 `ds_pending_deadlist`，由 sync 线程补插 | `module/zfs/dsl_dataset.c:304-349`；补插在 `:2397-2398` | 读源码 | ZFS 的块指针是物理 DVA、没有中央映射；本工程走中央映射，指针带出生 txg |
| 建快照：快照接走 head 原来的 deadlist 对象；head 换成一个克隆（同样的键、空 bpobj），再加一个新键 `ds_prev_snap_txg` | `dsl_dataset.c:1787`（`dsphys->ds_deadlist_obj = dsl_dataset_phys(ds)->ds_deadlist_obj`）、`:1842-1849`；`dsl_deadlist_clone` 在 `dsl_deadlist.c:700-730` | 读源码 | ZFS 的块指针是物理 DVA、没有中央映射；本工程走中央映射，指针带出生 txg |
| 并发：deadlist 只在 syncing 线程改；除 insert 外要持 `dp_config_rwlock` 写锁 | `dsl_deadlist.c:25-31` | 读注释 | 并发模型差异不明，只能当线索 |
| 删快照（新格式）：先用 `dsl_deadlist_space_range` 调 prev 的 unique 与 snapused；`dsl_deadlist_move_bpobj` 把 next 的 deadlist 里 mintxg ≥ 被删快照 `ds_prev_snap_txg` 的那些桶整桶挂到池的 `dp_free_bpobj`（这些块被删快照之后出生、死在 next 之前，没人再引用）；再 `dsl_deadlist_merge` 把被删快照自己的 deadlist 按键并进 next 的 deadlist；最后从 head 的 deadlist 删掉被删快照 creation_txg 那个键（`remove_key` 把该桶并进前一个桶） | `module/zfs/dsl_destroy.c:403-439`、`:478-487`；`dsl_deadlist.c:920-979`（move_bpobj）、`:855-918`（merge）、`:539-567`（remove_key） | 读源码 | ZFS 的块指针是物理 DVA、没有中央映射；本工程走中央映射，指针带出生 txg；本工程计划「删快照时 deadlist 并到更新的一侧」，ZFS 并入的是 next（更新的一侧） |
| merge 与 move 是整桶挂 bpobj 子对象（`bpobj_enqueue_subobj` / `dle_enqueue_subobj`），不逐块搬；预取最多 128 个子 deadlist | `dsl_deadlist.c:822`、`:954`；`:882-891`、`:942-949` | 读源码 | ZFS 的块指针是物理 DVA、没有中央映射；本工程走中央映射，指针带出生 txg |
| 旧盘上格式（deadlist 是单个 bpobj，`dl_oldfmt`）走 `process_old_deadlist`：逐块按 birth 与 `ds_prev_snap_txg` 比，留在 deadlist 或 `dsl_free_sync` | `dsl_destroy.c:127-151`、`:154-188` | 读源码 | 兼容包袱：本工程没有旧格式 |
| livelist 是什么：同样用 `dsl_deadlist_t`，挂在克隆的 dsl_dir 上，只记克隆创建之后出生的块，每条标 FREE 或 ALLOC；删克隆时倒着扫，配对抵消 FREE/ALLOC，剩下的 ALLOC 释放；嵌入式块不记 | `dsl_deadlist.c:45-63`；`dsl_dataset.c:291-302`（`birth > dd_origin_txg` 才进 `dd_pending_frees`） | 读注释与源码 | 本工程计划只做只读快照；ZFS 的 livelist 只服务可写克隆，差异不明，只能当线索 |
| livelist 何时用：pool feature `livelist` 开着时，建克隆就分配 livelist；共享比例 `(referenced-used)/referenced` ≤ `zfs_livelist_min_percent_shared`（默认 75）时在 sync 中关掉；promote 与 clone swap 时两侧都删掉 livelist；删克隆时有 livelist 走 `dsl_async_clone_destroy`，否则走 async destroy 或旧的同步遍历 | `dsl_dataset.c:1353-1365`、`:2169-2183`、`:2400-2404`、`:4019-4026`、`:4518-4524`；`dsl_deadlist.c:103`；`dsl_destroy.c:1139-1146` | 读源码 | 本工程计划只做只读快照；ZFS 的 livelist 只服务可写克隆，差异不明，只能当线索 |
| livelist 分子表：子表超过 `zfs_livelist_max_entries`（默认 500000，注释写约 128M）开新子表；按 birth txg 分组保证同一块的 ALLOC/FREE 落在同一子表，所以能一个子表一个 sync 地增量删；另有定期 condense 线程 | `dsl_deadlist.c:64-75`、`:77-86`；`module/zfs/spa.c:3486`（`spa_livelist_condense_sync`）、`:3559`（`spa_livelist_condense_cb`） | 读注释与源码 | 本工程计划只做只读快照；ZFS 的 livelist 只服务可写克隆，差异不明，只能当线索 |
| livelist 的特性页说明：克隆有大量随机/稀疏写时比传统方法删得快；克隆创建即 active，直到所有克隆删完 | `man/man7/zpool-features.7:775-782` | 读文档 | 本工程计划只做只读快照；ZFS 的 livelist 只服务可写克隆，差异不明，只能当线索 |

ZFS 第二组：没有 block pointer rewrite，device removal 走 indirect mapping。

| 事实 | 出处与日期 | 口径 | 与本工程的差异 |
|---|---|---|---|
| 源码自述的理由：indirect vdev 对应被移除的设备，「since we cannot rewrite block pointers of snapshots, etc.」，所以保留一张旧位置到新位置的映射，每次访问该 DVA 都查它 | `module/zfs/vdev_indirect.c:36-47`（`4542a317`） | 读注释 | 不改指针、在 SPA 层加一层旧→新映射 这一做法：ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 映射不按逻辑块边界建，一个 DVA 可能跨几段映射（split block），所以不是每个 DVA 都能换成一个等价的新 DVA，只能提供按段回调的 `vdev_remap` | `vdev_indirect.c:41-58` | 读注释 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 设计者公开说法（Alex Reece，2015-01-13，Delphix 博客）：按块指针建旧→新映射「quickly requires an unreasonably large data structure that must be stored both on disk and kept in memory」；「Even a technique like the now-defunct BP rewrite project would require a huge temporary table (and brings its own set of problems)」；所以在 SPA 层按旧盘上连续分配段建映射 | 原 URL https://delphix.com/blog/delphix-engineering/openzfs-device-removal 现 302 到 perforce.com/blog；取的是 web.archive.org 的 2020 快照（`/web/2020id_/…`），正文第 104-107 行（抽文本后的行号） | 读文档（存档） | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 移除流程：停分配；独立线程按 LBA 序拷贝已分配段，sync task 更新部分映射；拷贝期间的 free 同时释放旧位置与已拷贝的新位置；完成后 vdev 变 indirect | `module/zfs/vdev_removal.c:43-73` | 读注释 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 限制：普通分配类里不能有 raidz/draid 顶层 vdev（段拷贝不看块边界，算不出校验列）；所有顶层 vdev ashift 相同；加密数据集的 key 要已加载；所有 vdev 在线、健康 | `vdev_removal.c:75-92`；`man/man8/zpool-remove.8:50-52` | 读注释与文档 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 已知未解：「device removal for RAID-Z」issue #9013（ahrens，2019-07-10 开，至 2026-10-12 仍 open）；「RFC: generic device removal」#9129（2019-08-05 开，仍 open） | GitHub API `repos/openzfs/zfs/issues/9013`、`/9129`，取于 2026-10-12 | 读 issue | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 内存代价：映射整张读进内存（`vmem_alloc(map_size)` 后 `dmu_read` 整个映射对象）；`zpool remove -n` 打印移除后映射表预计占用的内存；`zpool status` 打印「memory used for removed device mappings」 | `module/zfs/vdev_indirect_mapping.c:357-361`；`zpool-remove.8:78-83`；`cmd/zpool/zpool_main.c:10607` | 读源码与文档 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 盘上每条映射 = `vimep_src`（u64，源偏移 63 位 + 1 位 mark）+ `vimep_dst`（`dva_t`）；asize 24 位，单条最多映射 8GB | `include/sys/vdev_indirect_mapping.h:29-43` | 读源码；条目字节数没有在源码里写明，按结构推是 8 + 16 = 24 字节（推的） | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 一个用户报的量：移除 6.94T 的 vdev 后「39.8M memory used for removed device mappings」 | issue #18930 正文（2026-08-15，TrueNAS，zfs-2.3.0） | 别人报的输出，未核负载与碎片度 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 缩小映射：块被 free 或 remap 后对应映射段变 obsolete，按条目计数；obsolete 比例 ≥ `zfs_condense_indirect_obsolete_pct`（默认 25%）或 obsolete spacemap 超 1GB 就 condense，重写一张只含仍被引用条目的新映射，可断点续做；`obsolete_counts` 特性一旦 active 不回退 | `vdev_indirect.c:60-124`、`:126-171`、`:175-197`；`zpool-features.7:864-878` | 读注释与文档 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| remap：写 indirect block 时把其中指向 indirect vdev 的 DVA 改成具体 DVA（能换才换，split block 换不了）；dedup 块、gang 块不 remap；被快照引用的旧 DVA 进 `ds_remap_deadlist`，删快照时按 deadlist 同一算法挪进 `dp_obsolete_bpobj` | `module/zfs/metaslab.c:5699-5730`；`module/zfs/dbuf.c:5270-5287`；`vdev_indirect.c:100-123`；`dsl_destroy.c:280-302` | 读源码 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| `zfs remap` 子命令在 master 已不存在 | `grep -n '"remap"\|remap' cmd/zfs/zfs_main.c` 无输出 | 实测 grep（只查了这一个文件） | 兼容包袱：本工程没有 |
| 读 split block 时若副本有坏，按组合重建，组合数超 `zfs_reconstruct_indirect_combinations_max`（默认 4096）时每次只随机试 100 种 | `vdev_indirect.c:207-215` | 读注释 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 近期 indirect 相关缺陷（只列标题与状态）：#18930「Kernel NULL pointer dereference in vdev_indirect_mapping_duplicate_adjacent_entries() … on a pool with an indirect vdev」open；#19120「vdev_indirect: validate split-block copies before repair」closed（2026-09-16）；#12200「VERIFY3(0 == vdev_obsolete_sm_object(...)) failed」open | GitHub search API `q=repo:openzfs/zfs "indirect vdev"`，共 292 条命中，取于 2026-10-12 | 只读标题，#18930 读了正文开头 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |
| 相近的现役做法：`zfs rewrite` 把文件块原样重写到新位置；被快照共享的块重写后旧副本仍留给快照，空间会涨，`-S` 跳过与快照共享的块、`-C` 跳过 BRT 克隆块；`-P` 保留逻辑 birth（需 `physical_rewrite` 特性） | `man/man8/zfs-rewrite.8:31-57`、`:84-91`；`zpool-features.7:880-895` | 读文档 | ZFS 平时没有中央映射，只在移除设备后对那一个 vdev 加映射；本工程走中央映射，所有指针本来就经映射 |

查过没找到的：OpenZFS 树里没有一份正面写「为什么不做 BP rewrite」的设计文档（在稀疏检出的 `module include man cmd` 上跑 `grep -rn -i 'bp rewrite\|bprewrite\|block pointer rewrite\|rewrite block pointers\|bp_rewrite'`，唯一命中是 `module/zfs/vdev_indirect.c:38` 那句注释；没检出的目录如 `tests`、`lib` 没查）；Delphix 原博客页已下线，只有存档；indirect mapping 读路径的延迟代价没找到任何公开量出来的数。

### 10.2 btrfs：共享块引用计数、删快照、balance/relocation

| 事实 | 出处（Linux `24a19b95`，7.3-rc7） | 口径 | 与本工程的差异 |
|---|---|---|---|
| 建快照：`btrfs_copy_root` 只复制根节点，对新根调一次 `btrfs_inc_ref`（给根节点的每个子指针 +1 引用），并把源树的 `last_snapshot` 设成当前 transid；除根节点之外的树块不复制 | `fs/btrfs/ctree.c:238-296`；`fs/btrfs/transaction.c:1798`、`:1827` | 读源码 | 共享靠 extent tree 里的引用计数与反向引用，COW 时惰性下推引用 这一做法：btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射 |
| 判是否可能共享：不可共享树、根节点、generation > `last_snapshot` 且不是 relocation 分配的块，判为不共享，不查 extent tree | `ctree.c:312-347` | 读源码 | btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射 |
| COW 一个可能共享的块（`update_ref_for_cow`）：refs > 1 时，若块属本树且无 FULL_BACKREF，对旧块 `btrfs_inc_ref(…, full_backref=true)` 并给旧块置 `BTRFS_BLOCK_FLAG_FULL_BACKREF`，否则对新块 inc_ref；refs == 1 且旧块是 FULL_BACKREF 时，新块 inc_ref、旧块 dec_ref | `ctree.c:349-451`（规则注释 `:362-377`，分支 `:414-449`） | 读源码 | btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射 |
| `btrfs_inc_ref`/`btrfs_dec_ref` 都进 `__btrfs_mod_ref`：逐个遍历块内的子指针（叶子里是 EXTENT_DATA 的 disk_bytenr，节点里是子块地址），对每个发一条 delayed ref；full_backref 时 `parent = buf->start`（共享反向引用按父块地址记），否则 parent = 0（按根 ID 记） | `fs/btrfs/extent-tree.c:2553-2649`（parent 取法 `:2580-2583`） | 读源码 | btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射 |
| 删快照 `btrfs_drop_snapshot`：遍历树，只被本树引用的块释放；遇到共享块只把引用减一不下探（`do_walk_down` 注释「if the block is shared and there is no need to update back, this function drops the reference」） | `extent-tree.c:6247-6259`、`:5902-5913` | 读注释 | btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射；本工程计划用 deadlist 而不是遍历引用计数 |
| 可中断续做：每轮把 `drop_progress`（key）与 `drop_level` 写进 root_item；`btrfs_should_end_transaction` 或 cleaner 需要让路时 `btrfs_update_root` 落盘后结束事务，非 relocation 情形返回 `-EAGAIN` 提前退出，root 加回 dead root 列表下次接着做；重启后从 `drop_progress` 搜回原位继续 | `extent-tree.c:6318-6373`、`:6403-6433`、`:6514-6519` | 读源码 | btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射 |
| 续做时防重复减引用：`wc->restarted` 时先 `check_ref_exists` 确认本树对该块的引用还在，找到之后才清 restarted | `extent-tree.c:5856-5869` | 读源码 | btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射 |
| qgroup 下删快照：遇到 refs > 1 的共享子树调 `btrfs_qgroup_trace_subtree`；子树根层级 ≥ `qgroup_drop_subtree_thres`（默认 3，sysfs `qgroups/drop_subtree_threshold` 可调）时不算账，直接把 qgroup 标成 inconsistent，理由写在注释里：「we are going to change ownership for quite a lot of extents, which will greatly slow down btrfs_commit_transaction()」 | `extent-tree.c:5871-5887`；`fs/btrfs/qgroup.c:2647-2658`；`fs/btrfs/qgroup.h:127`；`fs/btrfs/sysfs.c:2447-2448` | 读源码 | btrfs 的父节点直接存子块逻辑地址，没有中央映射；本工程走中央映射 |
| 官方文档对 qgroup 的说法：「The qgroup mode is considered not recommended for daily usage, unless there is no planned new snapshot/subvolume creation and deletion」；「The core design of qgroup mode and snapshot are not compatible from day one」；快照创建与子卷删除「are heavily affected and can cause a system hang」 | https://btrfs.readthedocs.io/en/latest/btrfs-quota.html 「STABILITY AND PERFORMANCE IMPLICATIONS」节（抽文本第 112-123 行） | 读文档 | 本工程没有 qgroup 这层记账，差异不明，只能当线索 |
| 同一文档的 squota 一节：qgroup「must count up the number of trees referring to an extent after its references change. This can slow down transaction commits and lead to unacceptable latencies, especially in cases where snapshots scale up」 | https://btrfs.readthedocs.io/en/latest/Qgroups.html 「Simple quotas (squota)」节（抽文本第 216-223 行） | 读文档 | 本工程没有 qgroup 这层记账，差异不明，只能当线索 |
| relocation 总流程：块组置只读；记下块组内每个 extent；建 data reloc tree（一个 inode 装新搬的数据 extent）与 reloc tree（源树的特殊快照，装搬过的树块）；把源树与 reloc tree 交换；最后清理 reloc tree 与 data reloc tree（旧 extent 仍被 reloc tree 引用，清完才能释放块组） | `fs/btrfs/relocation.c:42-87` | 读注释 | 经典 relocation：反向引用回溯、reloc tree、改写所有引用方的指针 这一做法：btrfs 要改每个引用方里的地址；本工程走中央映射 |
| 反向引用回溯：`build_backref_tree` 从一个树块的 backref 找上层引用块，再递归，直到树根或命中缓存，叶子对应引用它的各棵树的根 | `relocation.c:485-498` | 读注释 | btrfs 要改每个引用方里的地址；本工程走中央映射 |
| 数据分两段：`MOVE_DATA_EXTENTS` 拷数据，写回并等 ordered extent 后切到 `UPDATE_DATA_PTRS`，再改各树里 file extent item 指向新位置（`replace_file_extents` 只在 UPDATE_DATA_PTRS 段做事；reloc tree 一律用 full backref）；循环直到某一轮找到的 extent 数为 0 | `relocation.c:5382-5431`、`:928-959` | 读源码 | btrfs 要改每个引用方里的地址；本工程走中央映射 |
| 崩溃续做：`btrfs_recover_relocation`「resumes merging reloc trees with corresponding fs trees. this is important for keeping the sharing of tree blocks」；balance 进度存成内部状态，挂载时续做，除非 `skip_balance` | `relocation.c:5614-5620`；https://btrfs.readthedocs.io/en/latest/btrfs-balance.html（抽文本第 115-119 行） | 读注释与文档 | btrfs 要改每个引用方里的地址；本工程走中央映射 |
| balance 文档：「Extent sharing is preserved and reflinks are not broken」；不带过滤器跑「will take a lot of time as it basically move data/metadata from the whole filesystem and needs to update all block pointers」 | btrfs-balance.html（抽文本第 111-112 行、第 121-123 行） | 读文档 | btrfs 要改每个引用方里的地址；本工程走中央映射 |
| qgroup 与 relocation：交换子树时要追踪大量树块的 qgroup 归属，`qgroup_trace_extent_swap` 的注释画了 file tree 与 reloc tree 的换块例子；`btrfs_qgroup_add_swapped_blocks` 记下被换的子树根，延后追踪 | `qgroup.c:2297-2330`、`:4751-4765` | 读注释 | btrfs 要改每个引用方里的地址；本工程走中央映射 |
| backref 回溯代价的现役规避：send 对引用数 > `SEND_MAX_EXTENT_REFS`（1024）的 extent 不做 clone 改发 write，注释原文「avoid hitting limitations of the backreference walking code (taking a lot of time and using too much memory for extents with large number of references)」 | `fs/btrfs/send.c:37-43`、`:1506-1514` | 读源码 | btrfs 要改每个引用方里的地址；本工程走中央映射 |

btrfs 第二组：主线新加的 remap tree（relocation 不改引用方指针，改走一层地址重映射）。

| 事实 | 出处与日期 | 口径 | 与本工程的差异 |
|---|---|---|---|
| 引入：incompat 位 `BTRFS_FEATURE_INCOMPAT_REMAP_TREE`（1ULL << 17）、树 ID `BTRFS_REMAP_TREE_OBJECTID` 13、三种 key `IDENTITY_REMAP`(234)/`REMAP`(235)/`REMAP_BACKREF`(236)、块组标志 `REMAPPED` 与 `METADATA_REMAP` | `include/uapi/linux/btrfs.h:339`；`include/uapi/linux/btrfs_tree.h:80`、`:288-290`、`:1174-1175`、`:1348-1355` | 读源码 | 块组级旧→新地址映射，I/O 时翻译 这一做法：btrfs 只对被搬的块组加这层映射，其余地址不经映射；本工程走中央映射，所有指针都经映射 |
| 首次进主线的 tag：`include/uapi/linux/btrfs.h` 里 `INCOMPAT_REMAP_TREE` 在 v6.19 计数 0、v7.0-rc1 计数 1；引入提交 `ef6a31d035a1`「btrfs: add definitions and constants for remap-tree」，作者 Mark Harmstone 2026-01-07，提交者 David Sterba 2026-02-03 | git.kernel.org `plain/include/uapi/linux/btrfs.h?h=<tag>` 与 `commit/?id=ef6a31d…`，取于 2026-10-12 | 实测（curl + grep -c） | btrfs 只对被搬的块组加这层映射，其余地址不经映射；本工程走中央映射，所有指针都经映射 |
| 仍是实验特性：sysfs 的 `remap_tree` 特性属性包在 `#ifdef CONFIG_BTRFS_EXPERIMENTAL` 里，注释「Remove once support for remap tree is feature complete」 | `fs/btrfs/sysfs.c:297-304`、`:333-337` | 读源码 | btrfs 只对被搬的块组加这层映射，其余地址不经映射；本工程走中央映射，所有指针都经映射 |
| 哪些块组走它：开了 REMAP_TREE 且不是 SYSTEM、不是 METADATA_REMAP 块组；走它时先 `move_existing_remaps`、再 `start_block_group_remapping` 与 `do_remap_reloc`，不走旧的 `do_nonremap_reloc`；数据 extent 也不再走 `relocate_file_extent_cluster` | `fs/btrfs/relocation.h:15-24`；`relocation.c:5542-5559`、`:3747-3751`；`fs/btrfs/volumes.c:3745-3750` | 读源码 | btrfs 只对被搬的块组加这层映射，其余地址不经映射；本工程走中央映射，所有指针都经映射 |
| I/O 翻译：映射块组时若块组带 REMAPPED，`btrfs_translate_remap` 在 remap tree 里找覆盖该地址的项，IDENTITY_REMAP 表示原地未搬，REMAP 项给出新起始地址，长度截到项的末尾 | `relocation.c:5256-5304`；`volumes.c:7258-7268`（另一处调用在 `:6756`） | 读源码 | btrfs 只对被搬的块组加这层映射，其余地址不经映射；本工程走中央映射，所有指针都经映射 |
| 何时算空块组：REMAPPED 块组以 `identity_remap_count == 0` 为空，普通块组以 `used == 0 && remap_bytes == 0` 为空（discard 与删除未用块组都看这个） | `fs/btrfs/discard.c:218-235` | 读源码 | btrfs 只对被搬的块组加这层映射，其余地址不经映射；本工程走中央映射，所有指针都经映射 |
| 作者 RFC 给的动机：「Our present system involves following backrefs to rewrite addresses in the metadata trees, which can be slow」，并「has knock-on effects such as breaking nocow files」；SYSTEM 与 REMAP 块组「relocation is as it is at the moment」 | Mark Harmstone，「[RFC PATCH 00/10] btrfs: remap tree」，linux-btrfs，2025-05-15，https://lwn.net/Articles/1021452/ | 读邮件（经 WebFetch 摘要，引文是摘要给出的原句，没有逐字回对原邮件） | btrfs 只对被搬的块组加这层映射，其余地址不经映射；本工程走中央映射，所有指针都经映射 |

查过没找到的：btrfs 主线里没找到一处公开量过「qgroup 开着时 balance 慢多少」的数，只有文档里定性的说法；remap tree 在 7.x 里有没有已知缺陷清单没查（没翻 linux-btrfs 列表与 7.0–7.3 的 fixes）；remap tree 上被快照共享的 extent 搬迁时 qgroup 怎么算没读源码。

### 10.3 bcachefs：快照树与键可见性、copygc 搬共享数据、backpointers

| 事实 | 出处（bcachefs-tools `efec39c7`，路径相对 `fs/`） | 口径 | 与本工程的差异 |
|---|---|---|---|
| 四棵 btree 带快照（extents、inodes、dirents、xattrs）；每个 key 的位置里带快照 ID（`bpos.snapshot`），同一 btree 按 (inode, offset, snapshot) 排序；读某快照时，key 可见当且仅当它的快照 ID 是所求快照的祖先或本身、且没有更近的祖先覆盖它；快照内删除插 whiteout | `snapshots/snapshot.c:44-54` | 读文档注释 | 共享靠「键按快照 ID 可见」，不靠引用计数或 deadlist 这一做法：bcachefs 的共享单位是 btree key，快照树在 key 空间里；本工程计划按出生 txg 记 deadlist、走中央映射 |
| 祖先判断：每个快照节点存 128 位祖先位图（ID 差 128 以内 O(1)）加三项随机跳表 | `snapshots/snapshot.c:56-60`；`snapshots/types.h:14`（`IS_ANCESTOR_BITMAP 128`） | 读注释与源码 | bcachefs 的共享单位是 btree key，快照树在 key 空间里；本工程计划按出生 txg 记 deadlist、走中央映射 |
| 快照 ID 向下分配，子比父小，所以「祖先排在后面、最近的在前」，可见性过滤是一次前向扫描 | `snapshots/snapshot.c:62-84` | 读注释 | bcachefs 的共享单位是 btree key，快照树在 key 空间里；本工程计划按出生 txg 记 deadlist、走中央映射 |
| 建快照：在源子卷当前快照节点下分两个子节点，一个给新快照、一个替换源子卷；不复制 key；注释直说与 btrfs「clones entire COW btrees on snapshot」不同 | `snapshots/snapshot.c:86-98` | 读注释 | bcachefs 的共享单位是 btree key，快照树在 key 空间里；本工程计划按出生 txg 记 deadlist、走中央映射 |
| 不变量之一：不同快照里的两个 extent 要么范围完全相同、要么不重叠；写路径会切开祖先的 extent 让后代的写对齐 | `snapshots/snapshot.c:117-121` | 读注释 | bcachefs 的共享单位是 btree key，快照树在 key 空间里；本工程计划按出生 txg 记 deadlist、走中央映射 |
| 删快照：先标记，再由 `delete_dead_snapshots` 恢复 pass（也在线跑）分一次或多次回收；「Cost is proportional to the data in the deleted snapshot」；无活后代的节点的 key 直接丢，只有一个活子节点的中间节点把 key 下移给那个子节点（`live_child`）；有两个活子节点的中间节点留着 | `snapshots/snapshot.c:128-163` | 读注释 | bcachefs 的共享单位是 btree key，快照树在 key 空间里；本工程计划按出生 txg 记 deadlist、走中央映射；本工程计划「deadlist 并到更新的一侧」，bcachefs 是把 key 下移给唯一的活后代 |
| 删快照的中断语义：key 只往下移、inodes btree 最后处理，所以中断只会留下「内容 key 已移、inode 未移」这一种偏斜 | `snapshots/snapshot.c:215-218` | 读注释 | bcachefs 的共享单位是 btree key，快照树在 key 空间里；本工程计划按出生 txg 记 deadlist、走中央映射 |
| copygc：桶只追加写，随机覆盖留下半空桶；copygc 选最碎的桶，经 backpointers 找到活数据，写到新桶并原子更新 extent 指针；保留 `copygc_reserve`（默认 8%，可设 5–20%）只给 copygc 用 | `data/copygc.c:6-28`；`doc/bcachefs-principles-of-operation.tex:308-321` | 读文档注释 | extent 按逻辑位置键控、值里存物理指针，搬迁改那一个 key 这一做法：bcachefs 的 key 带快照 ID，被多个快照共享的数据只有祖先快照里的一个 key，搬迁只改它；本工程走中央映射，搬迁改映射条目 |
| 选桶：碎片 LRU 的索引是 `dirty_sectors / bucket_size` 按 2^31 缩放，活数据越少越靠前（即按活数据量贪心）；正在写的桶、设备不可写、backpointer 与桶计数对不上的桶不搬 | `alloc/background.h:127-145`；`data/copygc.c:98-140` | 读源码 | bcachefs 的 key 带快照 ID，被多个快照共享的数据只有祖先快照里的一个 key，搬迁只改它；本工程走中央映射，搬迁改映射条目 |
| 搬迁提交：`bch2_data_update` 在原 key 的同一快照位置写回新 key（带 `BTREE_UPDATE_internal_snapshot_node`，即可以直接改中间快照节点的 key），并在 key 被切开时调 `bch2_insert_snapshot_whiteouts` 给后代里已覆盖该位置的快照补 whiteout | `data/update.c:431`、`:473-479`；`btree/update.h:89-110`；`btree/update.c:123-159` | 读源码 | bcachefs 的 key 带快照 ID，被多个快照共享的数据只有祖先快照里的一个 key，搬迁只改它；本工程走中央映射，搬迁改映射条目 |
| 是否「因为按逻辑地址键控所以不用改指针」：源码显示搬迁仍要改 extent key 里的物理指针（同一 key 一次事务），不需要改的是「各快照各自的引用」，因为共享只有一个 key | `data/copygc.c:25-27`；`data/update.c:473-479` | 读源码；「不需要逐快照改」是从「共享只有一个 key」推的 | bcachefs 的 key 带快照 ID，被多个快照共享的数据只有祖先快照里的一个 key，搬迁只改它；本工程走中央映射，搬迁改映射条目 |
| backpointers：每段有数据的盘上扇区都有一条反向指针，存在写缓冲 btree 里，key = (device, sector offset, discriminator)，值记拥有者 key 的 btree ID、level、位置（位置含快照 ID）、数据类型与桶 generation；由 extent trigger 自动增删 | `alloc/background.c:246-263`；结构 `bcachefs_format.h:568-577` | 读注释与源码 | bcachefs 的 key 带快照 ID，被多个快照共享的数据只有祖先快照里的一个 key，搬迁只改它；本工程走中央映射，搬迁改映射条目；backpointer 指向的是唯一那个 key，不是每个快照一条（从 key 共享推的） |
| 一致性：backpointer 扇区数与桶扇区数相同就认为该桶一致，不对的才做双向全检；三个恢复 pass 互查 | `alloc/background.c:284-306` | 读注释 | bcachefs 的 key 带快照 ID，被多个快照共享的数据只有祖先快照里的一个 key，搬迁只改它；本工程走中央映射，搬迁改映射条目 |
| 用 backpointers 的操作：copygc、设备撤出、scrub、reconcile | `alloc/background.c:271-282` | 读注释 | bcachefs 的 key 带快照 ID，被多个快照共享的数据只有祖先快照里的一个 key，搬迁只改它；本工程走中央映射，搬迁改映射条目 |

查过没找到的：没找到 bcachefs 公开量过「快照很多时 copygc 或 delete_dead_snapshots 的代价」的数；`doc/bcachefs-principles-of-operation.tex` 第 722 行「Subvolumes and snapshots」一节没逐行读，只读了源码里的 `DOC_LATEX(snapshots)` 注释。

### 10.4 F2FS 与 NILFS2：段清理

F2FS（Linux `24a19b95`）：

| 事实 | 出处 | 口径 | 与本工程的差异 |
|---|---|---|---|
| 三种选段算法 `GC_CB`（cost-benefit）、`GC_GREEDY`、`GC_AT`（age-threshold）；两种 GC 类型 `BG_GC`（后台）、`FG_GC`（按需） | `fs/f2fs/segment.h:132-154` | 读源码 | 日志结构段清理 这一做法：F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |
| 缺省对应：BG_GC 用 CB（开了 atgc 用 AT），FG_GC 用 GREEDY；`gc_urgent` / `gc_idle` 各档可强制改成 CB、GREEDY 或 AT | `fs/f2fs/gc.c:248-273`；文档 `Documentation/filesystems/f2fs.rst:822-828` | 读源码与文档 | F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |
| greedy 代价 = 段（或 section）里的有效块数；CB 代价 = `UINT_MAX - (100*(100-u)*age)/(100+u)`，u 是有效块百分比，age 按段 mtime 在全盘 min/max mtime 间线性归一 | `gc.c:390-410`、`:362-388` | 读源码 | F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |
| 后台 GC 选过的 victim 记在 `victim_secmap`，前台 GC 优先拿它们；非前台、非 urgent-high 时搜索范围限 `max_victim_search` | `gc.c:343-360`、`:302-310` | 读源码 | F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |
| 前台触发：空闲 section 不够时转 FG_GC；若 prefree 段够多，先写 checkpoint 把 prefree 变 free，可能就不用 FG_GC；GC 结束后空闲不够还会再写 checkpoint 回收 prefree | `gc.c:1986-2004`、`:2068-2075` | 读源码 | F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |
| 搬迁的原子性：被清空的段先进 prefree，只有 `do_checkpoint` 成功之后 `f2fs_clear_prefree_segments` 才把它们清掉并（开 discard 时）发 discard；checkpoint 失败走 `f2fs_release_discard_addrs` | `fs/f2fs/checkpoint.c:2039-2046`；`fs/f2fs/segment.c:2209-2282` | 读源码 | F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |
| SSR 在已脏段里找空位时取 `ckpt_valid_map | cur_valid_map` 的零位，上次 checkpoint 时有效的块在下次 checkpoint 前不会被覆盖 | `segment.c:3126-3140`；`segment.h:182-185` | 读源码 | F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |
| 文档：「F2FS adopts the greedy algorithm for on-demand cleaner, while background cleaner adopts cost-benefit algorithm」 | `Documentation/filesystems/f2fs.rst:817-828` | 读文档 | F2FS 没有快照，GC 从不处理共享块；本工程的 GC 要搬被快照共享的块 |

NILFS2（Linux `24a19b95` 内核侧，nilfs-utils `26efb5da` 用户态）：

| 事实 | 出处 | 口径 | 与本工程的差异 |
|---|---|---|---|
| checkpoint 每几秒或每次同步写自动产生；用户可把选中的 checkpoint 转成 snapshot，转回 checkpoint 之前一直保留；`NILFS_IOCTL_CHANGE_CPMODE` 在两态之间切换 | `Documentation/filesystems/nilfs2.rst:14-18`、`:100-102` | 读文档 | 块号经 DAT 虚拟化、GC 改 DAT 不改树、存活按 checkpoint 区间判 这一做法：NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |
| 删 checkpoint 时跳过 snapshot：区间删除遇到 snapshot 计数，最后返回 `-EBUSY`；单个删除对 snapshot 直接 `-EBUSY` | `fs/nilfs2/cpfile.c:507-513`、`:558-559`、`:750-777` | 读源码 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |
| GC 在用户态：`nilfs_cleanerd` 选段、判存活，经 `NILFS_IOCTL_CLEAN_SEGMENTS` 一次把要删的 checkpoint 区间、要释放的虚拟块号、要搬的块、要释放的段交给内核 | `nilfs2.rst:138-141`；`fs/nilfs2/ioctl.c:850-860`（argv 五段） | 读文档与源码 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |
| 选段策略：`selection_policy` 只支持 `timestamp`（从最旧到最新回收）；`nsegments_per_clean` 默认 2；`protection_period` 默认 3600 秒，「NILFS2 GC never deletes checkpoints and their blocks whose duration time is less than the value」 | nilfs-utils `man/nilfs_cleanerd.conf.5:22-26`、`:52-58` | 读文档 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判；F2FS 用 greedy/CB，NILFS2 只按时间 |
| 存活判断（用户态）：块的生存区间 `[p_start, p_end)` 是 checkpoint 号；`p_end` 为 `NILFS_CNO_MAX` 或晚于保护期下限即活；否则对快照号数组二分，只要区间里有一个快照就活 | nilfs-utils `lib/gc.c:417-470` | 读源码 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |
| 内核侧原子性：`nilfs_clean_segments` 先把 DAT 存进 shadow map，准备失败就从 shadow 恢复；然后以 `SC_LSEG_SR`（带 super root 的完整日志）写段，段的释放在同一次段构造的 SUFILE 阶段做（`nilfs_free_segments`），构造失败用 `nilfs_sufile_cancel_freev` 撤销；成功后开 discard 时才对释放的段发 discard | `fs/nilfs2/segment.c:2546-2580`、`:1260-1265`、`:1566-1573`、`:1880-1886` | 读源码 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |
| 准备步骤可安全中止的理由写在注释里：删 checkpoint「can be removed independently」；DAT「is updated atomically using a copy-on-write technique」；标脏「nondestructive」 | `fs/nilfs2/ioctl.c:797-834` | 读注释 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |
| 搬块改的是 DAT：GC 写新位置时 `nilfs_btree_assign_gc` 调 `nilfs_dat_move` 把虚拟块号对应的物理块号改成新地址，盘上 binfo 记的仍是虚拟块号，引用该块的各 checkpoint 的树不用改 | `fs/nilfs2/btree.c:2354-2379`；`fs/nilfs2/dat.c:317-331` | 读源码 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |
| GC 读进来的块挂在 dummy inode（gcinode）上，每次拷进新日志后释放；注释写明当前代的脏块与 GC 要搬的块不会重叠 | `fs/nilfs2/gcinode.c:11-22` | 读注释 | NILFS2 的 DAT 是一层虚拟块号到物理块号的映射，本工程走中央映射，两者都有一层间接；差异在 NILFS2 的快照是整卷 checkpoint、liveness 由用户态守护进程判 |

查过没找到的：F2FS 与 NILFS2 都没找到 GC 半途崩溃后的专门续做逻辑——F2FS 靠「上次 checkpoint 之前有效的块在下次 checkpoint 前不复用」回到上次 checkpoint，NILFS2 靠「段释放与新日志同一次写」，这两句是从 F2FS 与 NILFS2 的源码推的，没有找到文档原话；NILFS2 内核里没找到 GC 选段代码：`grep -rn -i 'victim\|greedy\|cost.benefit' fs/nilfs2 | wc -l` 输出 `0`。

### 10.5 APFS：快照与空间回收的公开事实

| 事实 | 出处与日期 | 口径 | 与本工程的差异 |
|---|---|---|---|
| 建快照时卷当前的 extent-reference tree 整棵移给快照，卷换一棵新的空 extent-reference tree | Apple File System Reference（2020-06-22 版）第 55 页 `apfs_extentref_tree_oid` 条 | 读官方文档 | 每个快照一棵 extent 引用树、物理 extent 带引用计数 这一做法：APFS 的引用计数按物理 extent 记，本工程计划按出生 txg 记 deadlist；APFS 有对象映射（omap）把虚拟 oid 映到物理地址，但文件数据 extent 直接用物理地址（第 102 页 `j_phys_ext_key_t` 的 key 是物理块地址），与本工程「所有指针经中央映射」不同 |
| 快照元数据记录 `j_snap_metadata_val_t` 带 `extentref_tree_oid`、`sblock_oid` 等；文档原话「Snapshots are designed to be fast and inexpensive to create; however, deleting a snapshot involves more work」 | Apple File System Reference（2020-06-22 版）第 117-118 页 | 读官方文档 | APFS 的引用计数按物理 extent 记，本工程计划按出生 txg 记 deadlist；APFS 有对象映射（omap）把虚拟 oid 映到物理地址，但文件数据 extent 直接用物理地址（第 102 页 `j_phys_ext_key_t` 的 key 是物理块地址），与本工程「所有指针经中央映射」不同 |
| 物理 extent 记录 `j_phys_ext_val_t` 有 `refcnt`，「The extent can be deleted when its reference count reaches zero」；「For a volume that has no snapshots, the kind is always APFS_KIND_NEW」 | Apple File System Reference（2020-06-22 版）第 102-103 页 | 读官方文档 | APFS 的引用计数按物理 extent 记，本工程计划按出生 txg 记 deadlist；APFS 有对象映射（omap）把虚拟 oid 映到物理地址，但文件数据 extent 直接用物理地址（第 102 页 `j_phys_ext_key_t` 的 key 是物理块地址），与本工程「所有指针经中央映射」不同 |
| 对象映射有快照树（xid → `omap_snapshot_t`），访问时以 xid 为键 | Apple File System Reference（2020-06-22 版）第 17 页、第 47 页 | 读官方文档 | APFS 的引用计数按物理 extent 记，本工程计划按出生 txg 记 deadlist；APFS 有对象映射（omap）把虚拟 oid 映到物理地址，但文件数据 extent 直接用物理地址（第 102 页 `j_phys_ext_key_t` 的 key 是物理块地址），与本工程「所有指针经中央映射」不同 |
| reaper 负责「deletion of objects that are too large to be deleted in the time between transactions」，跨多个事务保存删除进度；删快照用 `apfs_reap_state_t`（`last_pbn`、`cur_snap_xid`、`phase`）与 omap 的两个 reap 阶段（先 map tree、后 snapshot tree） | Apple File System Reference（2020-06-22 版）第 26 页、第 50 页、第 166-167 页 | 读官方文档（`apfs_reap_state_t` 文档写「No overview available」） | APFS 的引用计数按物理 extent 记，本工程计划按出生 txg 记 deadlist；APFS 有对象映射（omap）把虚拟 oid 映到物理地址，但文件数据 extent 直接用物理地址（第 102 页 `j_phys_ext_key_t` 的 key 是物理块地址），与本工程「所有指针经中央映射」不同；本工程计划的 deadlist 合并没有 reaper 这种跨事务删除对象 |
| 空间管理器有 free queue，key 是 (xid, paddr)，结构 `spaceman_free_queue_t` 带 `sfq_oldest_xid`；文档对它的说明是「No overview available」，没有写块何时可再分配 | Apple File System Reference（2020-06-22 版）第 159-160 页 | 读官方文档 | 差异不明，只能当线索 |
| 收缩分区用 evict-mapping tree 记「where the blocks are being moved to」 | Apple File System Reference（2020-06-22 版）第 33 页、第 42-43 页 | 读官方文档 | 差异不明，只能当线索 |
| 二手说法（非 Apple）：删一个很小的快照「may take around 0.2 seconds, mainly with cleaning up the object map」，由容器唯一的 Reaper 做 | Howard Oakley，「APFS: Snapshots」，eclecticlight.co，2024-04-08（经 WebFetch 摘要，没逐字回对） | 读二手文章 | 差异不明，只能当线索 |

查过没找到的：Apple 参考手册全文 `grep -c -i trim` 命中 7 行，全是 `INODE_ACTIVE_FILE_TRIMMED`（「overprovisioning file that has been trimmed」）及其索引，没有一处讲 TRIM 何时发、与块再分配的先后；手册没讲快照删除时怎么决定哪些 extent 释放、删新快照与删旧快照释放量的差别；Hansen 与 Toolan 的逆向论文（Digital Investigation，2017）这次没取到原文；Apple 开发者站上没找到另一份讲快照空间回收的官方文档（只搜了一次网页）。

### 10.6 discard/TRIM：什么时候发、与块可再分配的先后

| 事实 | 出处 | 口径 | 与本工程的差异 |
|---|---|---|---|
| ZFS 释放的块先进 `ms_defer[txg % TXG_DEFER_SIZE]`（`TXG_DEFER_SIZE` = 2），defer 期满放回 `ms_allocatable` 的同时，autotrim 开着就加进 `ms_trim`；空闲接近 slop、vdev 正在移除或 raidz 扩展时不 defer，直接把 `ms_freed` 加进 `ms_trim` | `module/zfs/metaslab.c:4616-4667`；`include/sys/txg.h:39` | 读源码 | 先可再分配、后 trim，分配时把范围从待 trim 集合里删掉 这一做法：ZFS 的 trim 以 metaslab 为单位，本工程以段为单位产出全空段再发 discard |
| 分配时把范围从 `ms_trim` 里清掉，所以已被重新分配的范围不会再被 autotrim | `metaslab.c:4928-4929`、`:5945-5946` | 读源码 | ZFS 的 trim 以 metaslab 为单位，本工程以段为单位产出全空段再发 discard |
| autotrim 线程每个 top-level vdev 一个，处理某个 metaslab 时先 `metaslab_disable` 停它的分配，再把 `ms_trim` 换成空树、对换出来的那棵发 TRIM；同一 metaslab 至少隔 `zfs_trim_txg_batch`（默认 32）个 txg 才再访问；≤ 32KiB 的范围跳过 | `module/zfs/vdev_trim.c:1257-1305`、`:54-78`、`:96`、`:115-129`；`metaslab.c:6304-6311` | 读源码 | ZFS 的 trim 以 metaslab 为单位，本工程以段为单位产出全空段再发 discard |
| 手动 `zpool trim`：每个叶 vdev 一个线程，遍历所有 metaslab 的未分配范围，正在 trim 的 metaslab 不能分配；进度写盘，可暂停续做；autotrim 进度不写盘 | `vdev_trim.c:40-52`、`:71-72` | 读注释 | ZFS 的 trim 以 metaslab 为单位，本工程以段为单位产出全空段再发 discard |
| `autotrim` 属性文档：「Automatic TRIM does not immediately reclaim blocks after a free」，会延迟以聚合小范围；默认 off | `man/man7/zpoolprops.7:444-458` | 读文档 | ZFS 的 trim 以 metaslab 为单位，本工程以段为单位产出全空段再发 discard |
| btrfs `discard=sync`：事务提交收尾 `btrfs_finish_extent_commit` 里对每段 pinned 范围先 `btrfs_discard_extent`，再 `unpin_extent_range` 放回可分配；即同步 discard 在可再分配之前 | `fs/btrfs/extent-tree.c:2998-3013` | 读源码 | btrfs 两种 discard 与可分配的先后 这一做法：btrfs 的空闲空间按块组的内存 free space cache 管，本工程按段 |
| btrfs `discard=async`：discard 状态只在内存的 free space cache 里，不持久化，挂载时全当未 discard；块组按空闲时间排 LRU，初始延迟 `BTRFS_DISCARD_DELAY` 120 秒「to give some chance for block reuse」，全空块组延迟 10 秒；IOPS 上限 1000 | `fs/btrfs/discard.c:16-61` | 读注释与源码 | btrfs 的空闲空间按块组的内存 free space cache 管，本工程按段 |
| btrfs async 的先后：被 unpin 的范围先回到可分配，之后才由 discard worker 处理；处理一个范围时先 `unlink_free_space` 把它从 free space cache 取出（并计入 reserved），discard 完再以 TRIMMED 状态加回；分配会把 bitmap 的 trimmed 状态重置 | `extent-tree.c:3052-3054`；`fs/btrfs/free-space-cache.c:3768`、`:3794`、`:3650-3710`；`discard.c:44-52` | 读源码 | btrfs 的空闲空间按块组的内存 free space cache 管，本工程按段 |
| btrfs 文档：sync discard「could have severe performance hit and is not recommended」；async 先在内存里记，攒够或过一段时间再 trim；「Since kernel 6.2 the discard=async mount option is automatically enabled on devices that support that」 | https://btrfs.readthedocs.io/en/latest/Trim.html（抽文本第 69-91 行） | 读文档 | btrfs 的空闲空间按块组的内存 free space cache 管，本工程按段 |
| F2FS：prefree 段在 checkpoint 成功后才转 free，此时对它们发 discard；分配一个块时若有覆盖它的 discard 命令还没提交就把该块从命令里挖掉，已提交就等它完成 | `fs/f2fs/segment.c:2223-2282`、`:1874-1917`、`:3933` | 读源码 | 段式回收与本工程相近；F2FS 没有快照 |
| NILFS2：GC 释放的段在带 super root 的日志写成功后才发 discard | `fs/nilfs2/segment.c:2560-2580` | 读源码 | 段式回收与本工程相近 |
| bcachefs：桶由非空变空时 trigger 把 data_type 置 `need_discard`，进 need_discard btree；discard worker 发完 TRIM 才清标志、桶变 free 进 freespace btree；发之前要求桶变空的那条 journal 序号 ≤ 已落盘的 `flushed_seq_ondisk`，且 < `rewind_seq_ondisk`，否则推迟 | `alloc/background.c:1291-1300`、`:56-76`、`:383-391`；`alloc/discard.c:343-351` | 读源码 | bcachefs 是「先 discard、后可再分配」，与 ZFS/btrfs async 的先后相反；本工程一侧 discard 与再分配的先后没查 |
| APFS：官方手册没有 TRIM 的说明 | APFS 手册对 TRIM 的情况见 10.5 末尾「查过没找到的」 | 读官方文档 | 差异不明 |

查过没找到的：ZFS 与 btrfs 都没找到「discard 未完成时崩溃」的专门处理说明（btrfs async 注释只写了挂载时全当未 discard）；没量任何一家的 discard 延迟或吞吐。

### 10.7 反证清单

| 反证 | 查了哪几家 | 怎么查的 |
|---|---|---|
| 查过的几家里，没有一家把「按出生 txg 分桶的 deadlist」和「能搬被快照共享块的段清理」放在同一个实现里 | ZFS（有 deadlist，没有通用 GC，只有 device removal 加 indirect mapping）；btrfs（共享靠 extent 引用计数，删快照靠遍历树减引用）；bcachefs（共享靠键按快照 ID 可见，删快照靠移键）；F2FS（没有快照）；NILFS2（DAT 条目记 `de_start`/`de_end` checkpoint 区间，存活按区间与快照号判）；APFS（每个快照一棵 extent 引用树加物理 extent refcnt，GC 机制手册没写） | 读 10.1 到 10.5 所列源码与文档；「没有」只覆盖这六家与所列提交 |
| OpenZFS master 里没有改写被快照引用的块指针的机制：源码自述「we cannot rewrite block pointers of snapshots」；`zfs rewrite` 对快照共享块是另写一份、旧副本留给快照 | 只查 ZFS | `vdev_indirect.c:36-40`；`zfs-rewrite.8:52-57`、`:84-91`；稀疏检出上 grep `bp rewrite` 一类字样只命中那一句注释 |
| 查过的几家里，搬被快照共享的数据而不改各引用方的，都借了一层间接：NILFS2 改 DAT（虚拟块号→物理块号），ZFS device removal 加 indirect mapping，btrfs remap tree（实验特性）加块组级重映射，bcachefs 共享本来只有一个 key；直接改所有引用方指针的只有 btrfs 经典 relocation，作者在 remap tree RFC 里说它「can be slow」 | ZFS、btrfs、bcachefs、NILFS2；F2FS 不涉及；APFS 未知 | 10.1 到 10.4 的源码与 RFC 原句 |
| ZFS 与 btrfs async 都是「块先回到可分配、之后才 trim，分配时把范围从待 trim 集合里删掉或从 free space 里暂时取出」；bcachefs 相反，「先 discard、后进 freespace」 | ZFS、btrfs、bcachefs；F2FS 是 checkpoint 后发 discard、分配时挖掉或等待；NILFS2 是日志写成后发 | 10.6 的源码 |

口径补记：LWN 上 remap tree RFC 与 eclecticlight 的 APFS 文章的引文是 WebFetch 摘要给出的、没有逐字回对原文；Delphix 博客用的是 web.archive.org 的 2020 存档。

### 7.9 「七、近十年学术成果扫描（2012–2026）」的来源

- [The Full Path to Full-Path Indexing (FAST 2018)](https://www.usenix.org/conference/fast18/presentation/zhan)
- [Copy-on-Abundant-Write for Nimble File System Clones (TOS 2021) / How to Copy Files (FAST 2020)](https://www.usenix.org/conference/fast20/presentation/zhan)
- [File Systems Fated for Senescence? Nonsense, Says Science! (FAST 2017)](https://www.usenix.org/conference/fast17/technical-sessions/presentation/conway)
- [SplinterDB: Closing the Bandwidth Gap for NVMe Key-Value Stores (ATC 2020)](https://www.usenix.org/conference/atc20/presentation/conway)
- [Closing the B+-tree vs. LSM-tree Write Amplification Gap (FAST 2022)](https://www.usenix.org/conference/fast22/presentation/qiao)
- [Rethinking File Mapping for Persistent Memory (FAST 2021)](https://www.usenix.org/conference/fast21/presentation/neal)
- [TABLEFS: Enhancing Metadata Efficiency in the Local File System (ATC 2013)](https://www.usenix.org/conference/atc13/technical-sessions/presentation/ren)
- [Push-Button Verification of File Systems via Crash Refinement (OSDI 2016)](https://www.usenix.org/conference/osdi16/technical-sessions/presentation/sigurbjarnarson)
- [Finding Crash-Consistency Bugs with Bounded Black-Box Crash Testing (OSDI 2018)](https://www.usenix.org/conference/osdi18/presentation/mohan)
- [Chipmunk: Investigating Crash-Consistency in Persistent-Memory File Systems (EuroSys 2023)](https://dl.acm.org/doi/10.1145/3552326.3567498)
- [Metis: File System Model Checking via Versatile Input and State Exploration (FAST 2024)](https://www.usenix.org/conference/fast24/presentation/liu-yifei)
- [GoJournal: a verified, concurrent, crash-safe journaling system (OSDI 2021)](https://www.usenix.org/conference/osdi21/presentation/chajed)
- [Verifying the DaisyNFS concurrent and crash-safe file system with sequential reasoning (OSDI 2022)](https://www.usenix.org/conference/osdi22/presentation/chajed)
- [Optimistic Crash Consistency (SOSP 2013)](https://research.cs.wisc.edu/adsl/Publications/optfs-sosp13.pdf)
- [Consistency Without Ordering (FAST 2012)](https://www.usenix.org/conference/fast12/consistency-without-ordering)
- [High Velocity Kernel File Systems with Bento (FAST 2021)](https://www.usenix.org/publications/loginonline/high-velocity-kernel-file-systems-bento)
- [NVMe Flexible Data Placement 概览（NVM Express, FMS 2023）](https://nvmexpress.org/wp-content/uploads/FMS-2023-Flexible-Data-Placement-FDP-Overview.pdf)
- [Bcachefs removed from the mainline kernel (LWN)](https://lwn.net/Articles/1040120/)

---
