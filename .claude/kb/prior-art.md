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

### 5.6 Bε-tree / BetrFS：写优化索引的教训

Bε-tree 是 B-tree 变体，**内部节点留出 ε 比例的空间缓冲消息**，
写被批量摊销着往下刷，写 I/O 在理论上低于 B+tree。BetrFS 是第一个用它的内核文件系统。

**教训比结构本身更值钱**：BetrFS 用**全路径索引**（key 就是完整路径），
于是 scan 极快，但**改名代价高到不可接受**。直到 BetrFS 0.4 引入 `range rename`
才把改名拉回可用区间。

> 对本工程：**写优化索引 + 全路径索引 = 改名问题**。
> 按 inode 编 key 就不继承这个坑；按路径编 key 就会。这是选 key 布局时的一条硬约束。

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

---

## 历史版本

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
