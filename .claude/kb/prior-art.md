# 他家方案调研

**全部为文档阅读所得，未在本工程验证**，也未 clone、编译、运行过任何一个项目。
星数/提交数是页面当时显示的值（2026-08-25）。

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
**后者是最新最权威的 bcachefs 设计文档，尚未通读。**

1. **bucket + generation 号**。盘切成 bucket，每个带一个代号；指针存「期望代号」，
   代号对不上说明这块已被回收 → 数据失效不需要更新任何反向索引。对 D1 是直接减负。
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
必须在预算里，见 `singlefs-ai-sop/rules/test-discipline.md`「模型对拍」。

## 四、内核 Rust 文件系统抽象层

至 2026 年，Rust 已进入 Linux 的驱动、文件系统、网络组件。fs 抽象层仍在演进，
且尚未就「通用 vs 只服务简单文件系统」达成结论。

来源：[Rust for filesystems (LWN)](https://lwn.net/Articles/978738/)。
**与本工程当前关系不大**——D7 已定前几年不进主线。

## 五、索引结构（D8 专项调查）

**口径**：以下来自 bcachefs *Principles of Operation* Rev 1.39.2（2026-08-25 修订）原文、
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

**对本工程的意义**：D7 已定不进主线，所以**这条约束对我们根本不存在**——
大节点是白捡的，不需要为它付出「自管缓存」以外的代价。

### 5.3 bucket 分配器：从扫描线程改成 btree

- bucket 512K–2M；bucket 与数据指针都带 **generation 号**，
  递增代号即可让旧指针全部失效，**不必去找、去删任何反向引用**。
- 原文说代号机制的价值超出了缓存失效：
  它让 journal replay 时**自举分配信息**变得可做，
  并且**让某些本来不可恢复的损坏变得可修**。← 直接呼应我们的「可重建性」设计目标。
- bucket 内**永不重写**，顺序写满即封存 → 必须有 copying GC。
  **copyGC 预留 8%（默认，可配 5%–20%）**——这是 D3「预留不能过大」的一个现成参照量。
- 有一棵 **fragmentation LRU btree** 记录 bucket 填充度，
  copygc 直接找最碎的，不用扫整棵 alloc btree。
- **分配器原本是内存 bucket 数组 + 专用扫描线程**维护 freelist/discard/eviction；
  后来全部换成 btree（freespace / discard / LRU 三棵），**扫描线程整个删掉**。
  原文给的理由：事务性 btree 代码「far easier to debug and reason about」，
  而且旧线程在文件系统接近满时会**吃掉大量 CPU**——换成 btree 后这些角落情况消失。
- bucket 天然映射到 SMR / zoned 设备的 zone。

### 5.4 记账（对 D5 的独立印证）

> "Accounting counters are maintained by triggers as keys are committed,
> not by walking the btrees."

而 `check_allocations` **靠全量遍历重建记账**，用来分辨「计数器过期」和「key 真的还在」。
这与 `invariants.md` 的 I-3.1 是同一个设计：**运行时增量维护，checker 用遍历来验它**。

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
对本工程 key 布局的影响见 [decisions.md](decisions.md) D8。

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

**更重要的是它给了 D1 第二个理由**：XFS 文档说反向映射
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

## 六、加密（D9 专项调查）

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

**对本工程的意义**：校验和跟着指针走（[decisions.md](decisions.md) D4）本身就把 MAC 和 nonce
的位置腾出来了。AEAD 对本工程是白捡的，对块对齐的文件系统不是。

### 6.2 ZFS 与 bcachefs 在同一个问题上分道扬镳：无 key 时还能不能维护

| | OpenZFS | bcachefs |
|---|---|---|
| 加密范围 | **只加密 level 0 块**；间接块（blkptr 树）不加密 | 除超级块外全部加密 |
| 认证结构 | 间接层用 SHA512(下层 MAC 集合) 聚合；objset 层两个 256 位 MAC（portable / local） | Poly1305 MAC 逐块，chain of trust 直到超级块 |
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

## 七、近十年学术成果扫描（2012–2026）

**本节只收「可能推翻某条既有决策前提」的成果，不做文献综述。** 每条注明它冲击哪条决策。
全部来自公开论文与项目文档，**未在本工程验证**。

### 7.1 随机小写：写优化索引把它变成非问题（冲击 D10）

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
| 全文**没有出现 checksum**，不存在「父节点存子节点校验和」的 Merkle 链 | singlefs 的 D4 内联校验和 + D9 的 MAC/nonce 给每次节点重写加了一份 BetrFS 不付的固定成本，交叉点因此会移动，移多少未知 |

**所以这组数字能证明的是**：消息缓冲区能把「一个 key 要被重写几次」从 O(树高) 降到 O(树高/B)。
**它不能证明的是**：在「Merkle 校验和链 + AEAD」都在、且发布频率取某个具体值的前提下还剩多少收益。
后者要靠 [experiments.md](experiments.md) E7 自己量。

⚠️ **发布频率是这里的自由变量，不是常量。** 若本工程采用 checkpoint 语义
（内存里攒脏节点、定期合并写一次），那么「一次小写不必立刻重写整条从叶子到根的路径」
这个效果，**checkpoint 批量本身就免费给了**，不需要在格式里放一个持久化的消息缓冲区。
BetrFS 那个数字里有多少来自消息缓冲、多少来自 5 秒 checkpoint，**论文没有拆开过**。
E7 的对照组因此必须区分两种「无消息缓冲」基线：完全无批量，
以及有 checkpoint 批量但无格式级消息缓冲。两者混为一谈会让消息缓冲显得比实际更必要。

机制是把小写变成 Bε-tree 的消息批量下刷，而不是原地改块，也不是关掉 COW。

对 D10 的意义：btrfs 的 `nodatacow` 是「关掉 COW 换随机写性能，代价是同时丢校验和与快照一致性」，
而这条路是「把随机写吸收进索引的消息层」，校验和与快照都不动。
**这是 D10 目前唯一一条有实测数字支撑的候选方向。**

**另一条顺带核实到的事实**：BetrFS 的所有版本都用**两棵** Bε-tree——
一棵放文件数据，一棵放文件系统元数据（论文 2.2 节原文：
「All versions of BetrFS use two Bε-trees: one for file data and one for file system metadata」）。
这与 D8「多个独立 keyspace」是同一个思路，只是数量少得多（bcachefs 是 28 棵）。

### 7.2 老化不是宿命，可以在结构层面设计掉（冲击 D3、D10）

FAST 2017「File Systems Fated for Senescence? Nonsense, Says Science!」用
**连续 git checkout Linux 内核源码**把文件系统做老，测 btrfs / ext4 / F2FS / XFS / ZFS 与 BetrFS。
结论：**老化后的 BetrFS 甚至胜过其余文件系统未老化时的表现（btrfs 除外）**。
另一条被 FAST 2018 引作动机的数字：git 版本控制负载可把 ext4 的 scan 性能退化至多 **15 倍**。

对本工程：D3 承诺常驻自动整理，理由是「不分类 → 碎片压力集中在一个分配器」。
这条成果给出第二条路——**老化可以被索引结构本身吸收掉，不必全靠事后整理**。
两条路各要多少代价可测，判据见 [experiments.md](experiments.md) E10。

### 7.3 索引结构：参数空间比「换一种树」大得多（冲击 D8）

| 成果 | 结论 | 口径 |
|---|---|---|
| SplinterDB（ATC 2020） | STBε-tree = Bε-tree 加 LSM 的 size-tiering，做成「树的树」：主干 trunk 树的每个节点指向一组 B-tree 分支，每个分支带一个 quotient filter。相对 RocksDB：插入快 6–10×、点查快 2–2.6×、写放大降 2× | 论文摘要数字，面向 NVMe SSD |
| FAST 2022「Closing the B+-tree vs. LSM-tree Write Amplification Gap」 | 在带内建透明压缩的硬件上，Bε-tree 写放大 28（8KB 页）/ 36（16KB 页）；同条件 RocksDB 38、WiredTiger 268（8KB 页）/ 530（16KB 页） | 500GB 数据集、32B 记录、4 客户端线程 |
| WiscKey 式 key-value 分离 | 大 value 移出索引、只在索引里留指针，compaction 只搬 key 与元数据，写放大最多降两个数量级。工业界已落地为 RocksDB BlobDB、Titan、TerarkDB | 论文 + 工业实现 |
| HashFS（FAST 2021「Rethinking File Mapping for Persistent Memory」） | 文件映射本身可占 IO 路径的 **70%**；哈希式映射相对「页缓存里的 extent 树」把 LevelDB 上的 YCSB 吞吐提高至多 45% | **持久内存场景**；本工程若不针对 PM 只作参考 |
| TableFS（ATC 2013） | 元数据全部塞进 LSM（LevelDB），元数据密集负载相对 ext4 / XFS / btrfs 快 50%–1000% | FUSE 实现，作者自陈实现低效，即便如此仍赢 |

对 D8 的意义：D8 已定「一套 btree 实现 + 多 keyspace + write buffer / key cache 两个前端」。
这几条不否定该方向，但把**同一套结构内部的可调项**显著扩大了：
Bε 缓冲比例、size-tiering、key-value 分离、过滤器类型——
都是一套 btree 里的参数，不是「再加一种树」。
这与 D8「不做异构结构」不冲突，反而是它的补充。

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
   这正是 `singlefs-ai-sop/rules/test-discipline.md` 说的模型对拍，
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

对本工程：`litmus/commit-publish.litmus` 钉住的是一次发布事件内部「先写块内容、再写超级块」这条序，
不涉及发布频率。
这两条成果说明该序有替代形态，代价各不相同。
⚠️ **NoFS 的反向指针与 D1 的反向索引是两件不同的东西**：
NoFS 的是「每块自证属于谁」的一个字段，D1 的是「物理块被谁引用」的可查询索引。
本工程「元数据块必须自描述」（`.claude/rules/fs-design.md`）已经接近 NoFS 那个字段。

### 7.6 硬件接口：块接口不再是唯一的对接层（冲击 D2、D3、D8 节点大小）

| 接口 | 状态 | 对本工程 |
|---|---|---|
| ZNS（Zoned Namespace） | 顺序写、由主机管 GC；因成本效率在规模部署中被采用 | zone 大小是 D8「按设备特性选节点大小」那条规则的输入 |
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

对 D7（不进主线）：Bento 的在线升级能力值得注意——它把「文件系统要不要在内核里」
从二选一变成了三选一（用户态 / 内核 C 模块 / 可热替换的内核 Rust 模块）。

### 7.8 bcachefs 现状更新（加强 D7 的依据，不改结论）

| 事实 | 日期 |
|---|---|
| bcachefs 在 Linux 6.17 被标记 externally maintained，**代码在 6.18 被从主线移除** | 2025–2026，公开报道 |
| 改为 DKMS 模块分发，形态与 OpenZFS on Linux 相同；开发者改向独立邮件列表投 patch | 同期 |
| bcachefs-tools v1.38.6 **去掉 experimental 标签** | 2026-06-17 |

对 D7：这条把依据从「bcachefs 被移出主线是社交失败不是技术失败」
加强为「**移出主线之后项目仍在正常演进，并且在这之后才去掉实验标签**」——
主线之外是一条可行的分发路径，不只是失败后的退路。

### 7.10 bcachefs 官方文档补录（2026-08-26 现查，Principles of Operation Rev 1.39.2+2202696）

| 事实 | 原文 / 口径 | 对本工程 |
|---|---|---|
| **bucket gen 的适用范围限定为缓存数据** | §1.5：「we can reuse a bucket with **cached data** in it without finding and deleting all the data pointers by incrementing the generation number」 | [decisions.md](decisions.md) D1：它只管失效不管搬运 |
| **gen 绕回机制已被反向索引取代** | 状态清单 `need_gc_gens`：「Legacy state, retained for compatibility … **now effectively unused since the invalidate worker uses backpointers instead of generation bumping**」 | 与 D1 已选的「反向索引 day-1」同向 |
| **copygc 预留 8%，可配 5–20%** | §1.5 与 §9.1.10。且「Normal writes cannot dip into this reserve」，超过约 90% 容量时写延迟上升 | D3 的「空间预留用准入控制形态，量要小」有了一个可对比的量级：**别家是划 8% 不给用**，本工程选的是动态准入，差异要记明 |
| **加密是全有全无，且只能在 mkfs 时开** | §2.1.2：「Encryption is all-or-nothing at the filesystem level: all data and metadata except the superblock is encrypted, and all data and metadata is authenticated … **Encryption can only be enabled at format time; it cannot be added to an existing filesystem**」 | 印证 D9 的方向；但注意本工程 D9 预留了「超级块的 KDF 标识 + 主密钥槽」，目标是**能在既有文件系统上开加密**，这是与 bcachefs 的一处有意分歧 |
| ⚠️ **`nocow` 写的数据在加密文件系统上是明文存储** | §2.1.2：「Data written with the `nocow` option is stored **unencrypted**, even on an encrypted filesystem. **This is a hard design incompatibility, not a policy choice**: ChaCha20 requires a unique nonce per (key, plaintext), and bcachefs stores the nonce externally alongside each data pointer」 | **这是 D9 与 D10 连锁那一条的实证**：本工程已定「任何绕过 COW 的快路径不许绕过加密；原地覆盖写会直接造成 nonce 重用」。bcachefs 遇到同一个约束，选择是**让 nocow 数据不加密**——本工程不接受这个交易，因此 D10 的任何候选都不许走原地覆盖写这条路 |
| **挂载前必须解锁** | §4.2.3：「the passphrase is requested once the devices have been found and **before any attempt to mount**」；FAQ：未解锁挂载报 `Required key not available` | 是 D9 未答项 5 那条推理的前提之一 |

**未找到的（明说，不补）**：没有任何一手材料用散文明说
「copygc / rebalance / scrub / fsck 在无密钥时不能跑」。
「无密钥后台整理瘫痪」是从上面几条推出来的，**是推论不是引文**，见 decisions.md D9。

来源：[bcachefs Principles of Operation (PDF)](https://bcachefs.org/bcachefs-principles-of-operation.pdf)、
[bcachefs.org/Encryption](https://bcachefs.org/Encryption/)、
[bcachefs.org/FAQ](https://bcachefs.org/FAQ/)、
[The Programmer's Guide to bcache](https://bcache.evilpiepirate.org/BcacheGuide/)、
[Kent Overstreet: Bcachefs - encryption, fsck, and more (LWN 镜像)](https://lwn.net/Articles/717378/)、
[Bringing bcachefs to the mainline (LWN, LSFMM 2022)](https://lwn.net/Articles/895266/)。

### 7.9 本节来源

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

## 历史版本

### 2026-08-26（其六）
- 补 7.10：bcachefs 官方文档现查补录六条，含 bucket gen 限定为缓存数据、
  gen 绕回机制已被反向索引取代（`need_gc_gens` 标为 legacy）、copygc 预留 8%、
  以及 `nocow` 数据在加密文件系统上明文存储这条硬不兼容。
  同时明说一条**未找到**：没有一手材料用散文说「无密钥时后台任务不能跑」，那是推论。

### 2026-08-26（其五）
- 7.1 与 7.5 删掉「每事务发布新根」这个说法。它曾经被当成本工程的既有承诺写进正文，
  实际上项目里从来没有这条：litmus 只钉一次发布事件内部的顺序，
  decisions.md / CLAUDE.md / rules/fs-design.md 全文无此表述（逐文件 grep 核实）。
  改动依据：它是一个未经审查的默认假设，而基于它做出的「Bε 收益在本工程会缩水」的推理
  因此站不住——发布频率是自由变量，checkpoint 语义下批量是免费的。

### 2026-08-26（其四）
- 7.1 补核实结果：BetrFS 的持久化是 redo log + 每 5 秒 checkpoint、节点 copy-on-write、
  全文无 checksum。核实经过：本地提取 FAST 2018 论文 PDF 文本后逐词检索。
  据此给那组随机写数字加了「能证明什么 / 不能证明什么」的分界。
  起因是一次三方论证里有一条腿断言「那个数字来自没有 COW 的系统」，核实后该断言不成立。

### 2026-08-26（其三）
- 补第七节：2012–2026 学术成果扫描，只收「可能推翻既有决策前提」的条目，共 8 小节。
- 5.6 改写：曾经写作「BetrFS 用全路径索引，改名代价高到不可接受……按路径编 key 就会继承这个坑」，
  现在写作「BetrFS 0.4（FAST 2018）用 lifted Bε-tree 把改名代价从子树大小降到子树深度」。
  改动依据：读了 FAST 2018 论文原文，复杂度一节明写被切分或合并的节点数
  「at most proportional to the height of the tree」。旧写法把 BetrFS 0.3 的状态当成了现状。

### 2026-08-26（其二）
- 补第六节：D9 加密专项调查。读了 bcachefs Encryption 设计文档与 OpenZFS
  `zio_crypt.c` 设计注释原文，记下三种加密边界的取舍、ZFS 与 bcachefs 在
  「无 key 能否维护」上的分岔、nonce 被迫进格式的那个字段，以及四条公开的坑。

### 2026-08-26
- 补第五节：D8 索引结构专项调查。读了 bcachefs Principles of Operation
  Rev 1.39.2（2026-08-25）原文，含 28 棵树的清单、write buffer 的代价、
  日志结构节点的三条性质、bucket 分配器的演进；另加 XFS rmapbt 成本量级
  与 BetrFS 的改名教训。

### 2026-08-25
- 建档。记账四模型、bcachefs 六点、Rust 轮子六个、内核 Rust fs 现状。
