# prior-art：Btrfs / Bcachefs / OpenZFS 一次 fsync 各写了什么

## 来源与版本（现查，2026-09-17）

- Linux 内核源码：`/home/fy5090/code/fs-refs/linux-6.17`（只解出 `Documentation/` 与 `fs/`，没有 `.git`，版本靠目录名与同目录 `linux-6.17.tar.xz`、`fetch.log` 认定；`fetch.log` 记 `[12:17:45] linux tarball 147M` `[12:17:50] all done`，文件 mtime `2026-08-29 12:17:50 UTC`）。Btrfs 在 `fs/btrfs`（65 个 `.c`），bcachefs 在 `fs/bcachefs`（98 个 `.c`）。
- OpenZFS 源码：`/home/fy5090/code/fs-refs/zfs`，`git log -1`：`58d73c90dcdd77cdd07fc0b53d12d1b7339d2fe7 2026-08-27 09:51:25 -0700`；`META` 里 `Branch: 1.0`、`Version: 2.4.99`——是开发分支，不是打了 tag 的正式版本（`git describe --tags` 报 `fatal: No names found`）。
- btrfs-progs：本机安装的 `mkfs.btrfs, part of btrfs-progs v6.6.3`（`/usr/sbin/mkfs.btrfs`），只用它的 man page 取 `nodesize` 默认值（mkfs 是用户态工具，不在上面那棵内核源码树里）。
- 三家都只读源码/文档，没有插桩、没有编译跑真实负载；下面每条事实的口径都写「读源码」或「读文档」，不是实测。

## 一、Btrfs：fsync 写日志树，不碰主树祖先与根，但每次都重写超级块

| 事实 | 出处 | 口径 | 与本工程的差异 |
|---|---|---|---|
| `btrfs_sync_file` 的主路径调用 `btrfs_log_dentry_safe` 把改动写进 `root->log_root`（子卷自己的日志树），这棵树与主子卷树 `root->node` 是两个不同的 CoW 树 | `fs/btrfs/file.c:1726-1733` | 读源码 | singlefs 没有独立的日志树，一条 journal 记录代替它；D23（journal 的角色与格式） 已定项 1 逐字「每次 fsync 写脏叶 + 全部祖先 + 根槽 + 一条记录，祖先不延后」——btrfs 恰好在「祖先要不要延后」这一点走的是相反的路：日志树自己也是 CoW 树，有自己的祖先链，但那条链只属于日志树，不牵动主树 |
| `btrfs_sync_log`（真正把日志落盘的函数）写完日志树与 `log_root_tree`（登记各子卷日志树 root 的一棵小树）之后，调 `write_all_supers(fs_info, 1)` 把改过的超级块写到全部镜像槽 | `fs/btrfs/tree-log.c:3182-3222` | 读源码 | singlefs 的根槽与超级块槽也是每次发布都写（D16（发布语义） 已定项 1），这一点两边一致；差异在 btrfs 的「超级块指向的是日志树根」而不是主树根——主树根这次发布完全没变 |
| 主子卷树的 `commit_root` 只在**全量事务提交**（`commit_transaction`，不是 `btrfs_sync_log`）里才被切换成 `root->node`；这段代码在 `transaction.c`，与 fsync 走的 `btrfs_sync_file → btrfs_sync_log` 是两条不同函数链 | `fs/btrfs/transaction.c:1508-1513` | 读源码，逐行核对这段代码不在 `tree-log.c` 的调用链里 | singlefs 目前把「fsync」定义成「一次全量发布」（D16（发布语义）「连带定死的三条」第 1 条），即每次 fsync 都做 btrfs 只在全量提交时才做的那件事；这是 fsync 语义粒度上的差异，不是格式差异 |
| 全量事务提交时，`btrfs_free_log(trans, root)` 把刚刚提交进主树的那些子卷的日志树整棵释放 | `fs/btrfs/transaction.c:1499`（在 `commit_transaction` 的根切换循环里） | 读源码 | singlefs 的 journal 是定长环形结构（D23），回收靠环内位置轮转；btrfs 的日志树是普通 CoW 树、按需分配树块，回收靠「下次全量提交整棵扔掉」，不是环形覆盖——这是盘上格式上的差异 |
| 全量提交的默认周期常量 `BTRFS_DEFAULT_COMMIT_INTERVAL = 30`（秒），可挂载参数 `commit=` 改 | `fs/btrfs/fs.h:302` | 读源码（默认值常量），未实测触发频率 | singlefs 没有「周期性提交」这个概念，每次 fsync 都是全量发布；btrfs 有两级：日志（每次 fsync）与全量提交（30 秒或被日志过多等条件提前触发），这是并发/持久化模型上的差异 |
| 挂载时若超级块的 `log_root` 非零，调用 `btrfs_recover_log_trees`（内部 `walk_log_tree` 两阶段：先 `LOG_WALK_PIN_ONLY` 钉住块，再重放） | `fs/btrfs/disk-io.c:2089` 调用；函数定义 `fs/btrfs/tree-log.c:7268` | 读源码 | 差异不明，只能当线索：两边都是「挂载时按一个指针决定要不要重放」，但 btrfs 重放的是一棵完整的日志树（可能有多层内部节点），singlefs 重放的是环形记录序列，遍历方式不同，本机没有比较两者重放代价的数据 |
| 默认树节点（`nodesize`，叶子与内部节点同一个尺寸）= 16 KiB 或页大小取较大者，此处两者都是 16 KiB；`leafsize` 恒等于 `nodesize` | 本机 `mkfs.btrfs --version`（v6.6.3）+ `man mkfs.btrfs`「default value is 16KiB (16384) or the page size, whichever is bigger」 | 读文档（用户态工具的默认值，不在这棵内核源码树里），未在内核源码里找到这个默认值本身，只在 `fs/btrfs/disk-io.c:3395` 看到 `fs_info->nodesize = nodesize` 这个运行期赋值点，说明它是一个格式化时定下、运行期只读的常量 | 与 singlefs 的两级尺寸（D8/D11：索引节点 16 KiB；D4 已定项 5/7：叶容器 32 KiB）不同，btrfs 节点和叶子共用同一个尺寸，不区分层级 |
| 超级块单份大小 `BTRFS_SUPER_INFO_SIZE = 4096`，`static_assert` 钉死这个结构体大小；一台盘最多写 `BTRFS_SUPER_MIRROR_MAX = 3` 份镜像超级块（按盘大小决定实际写几份，16 GiB 盘上不到第三份的偏移阈值） | `fs/btrfs/fs.h:71-72`；`fs/btrfs/disk-io.h:25-26` | 读源码 | 这是 201 195 字节里「超级块部分」的构成单元，但没有拆到「这一格 5 轮平均具体写了几份超级块、几个日志树节点」——见下面「负载口径对照」 |


## 二、Bcachefs：fsync 只等日志刷到某个序号，不强制写树节点，也不碰超级块

| 事实 | 出处 | 口径 | 与本工程的差异 |
|---|---|---|---|
| `bch2_fsync` 依次做：等页缓存写完、`sync_inode_metadata`、再调 `bch2_flush_inode` | `fs/bcachefs/fs-io.c:237-262` | 读源码 | — |
| `bch2_flush_inode` 里真正的持久化调用是 `bch2_journal_flush_seq(&c->journal, seq, ...)`——把日志刷到「这个 inode 最后一次被记日志时的序号」，函数名和调用点里都没有出现任何「写某棵 btree 节点」或「写超级块」的调用 | `fs/bcachefs/fs-io.c:219-235` | 读源码；`journal_io.c` 全文 `grep -c bch2_write_super` 命中 0 次（负向检查，只查了这一个文件） | singlefs 没有「先落日志、树节点稍后再补写」这种分层——D11（索引节点要不要留消息缓冲区） 已定项 7「第一版全部直落叶」；bcachefs 的 fsync 快，正是因为它把「持久」和「进主树、能被树查询到」拆成两个不同的落盘时间点，singlefs 目前是同一个时间点 |
| 日志条目的内存缓冲区大小上下限：`JOURNAL_ENTRY_SIZE_MIN = 64 KiB`，`JOURNAL_ENTRY_SIZE_MAX = 16 MiB`；但这只是内存缓冲区的分配尺寸，不是每次落盘一定写这么多 | `fs/bcachefs/journal_types.h:114-115` | 读源码 | — |
| 实际落盘字节数按内容取整到设备块大小：4 处写路径都调用 `vstruct_sectors(数据, c->block_bits)`，格式化时块大小默认 `BCH_SB_BLOCK_SIZE = 4 << 10`（4096 字节） | `fs/bcachefs/journal_io.c:1618,1855,2029,2081`；默认块大小 `fs/bcachefs/opts.h:129-133` | 读源码 | 与 singlefs 恒定 32 768 字节单元（D4（校验和位置） 已定项 5）比，bcachefs 的日志写是「按内容变长、只对齐到块边界」，不是恒定容器——这是盘上格式上的差异 |
| 树节点被日志钉住之后何时回写：`journal_reclaim`（把最老日志钉住的树节点写回、腾出日志空间）默认延迟 `journal_reclaim_delay = 100` 毫秒；日志自身独立于 fsync 的后台整块提交默认延迟 `journal_flush_delay = 1000` 毫秒 | `fs/bcachefs/opts.h:341-345`；`fs/bcachefs/opts.h:329-333` | 读源码（选项默认值），未实测这两个延迟在真实负载下的实际生效周期 | 与 btrfs 单一的 30 秒提交周期相比，bcachefs 拆成两个独立周期，对应两件不同的事（树节点回写 vs 日志本身落盘）；singlefs 目前只有「一次 fsync = 一次全量发布」，没有这种两级延迟 |
| 挂载时 `bch2_journal_read` 读日志，`bch2_journal_replay`（在 `recovery.c`）把日志项重放进树 | `fs/bcachefs/journal_io.c:1336`；`fs/bcachefs/recovery.c:342` 定义，`:860` 调用 | 读源码 | 差异不明，只能当线索：两边都靠一个「挂载时先读日志再重放」的路径，具体重放算法（按序号范围扫、还是按树遍历）没有比较 |

## 三、OpenZFS：fsync 只提交 ZIL，不碰 DMU 对象树或 uberblock

| 事实 | 出处 | 口径 | 与本工程的差异 |
|---|---|---|---|
| `zfs_fsync` 唯一的持久化调用是 `zil_commit(zfsvfs->z_log, zp->z_id)` | `module/zfs/zfs_vnops.c:104-116` | 读源码 | singlefs 的 fsync 语义（D16「连带定死的三条」第 1 条：fsync=一次全量发布）与这里正相反：ZFS 的 fsync 明确只做 ZIL 提交，不触发全量发布（那是 TXG 的事） |
| `zil.c` 全文没有直接调用 `vdev_uberblock_sync`；文中出现的 9 处 `spa_sync` 全部在注释里（描述与后台 TXG 提交的竞争关系），逐行核对不是函数调用 | `module/zfs/zil.c`（`grep -n "vdev_uberblock_sync\|spa_sync(" zil.c`，9 处命中，逐条核对行号 886/1325/1327/2369/2613/2618/3616/4098 均为注释） | 读源码 + 命中逐行核对（负向检查，只查了这一个文件） | — |
| ZIL 日志块最小尺寸 `ZIL_MIN_BLKSZ = 4096ULL`；实际写入尺寸按用量取整到这个边界：`wsz = P2ROUNDUP_TYPED(lwb->lwb_nused, ZIL_MIN_BLKSZ, uint64_t)` | `include/sys/zil.h:89`；`module/zfs/zil.c:1993` | 读源码 | 与 singlefs 恒定 32 768 字节数据单元比，ZIL 块最小 4 KiB、按需增长（上限由 `zil_max_log_data` 决定），不是恒定容器 |
| DMU 对象树（含间接块）与 uberblock 只在 TXG 提交（`spa_sync`）时落盘，默认最长间隔 `zfs_txg_timeout = 5`（秒） | `module/zfs/txg.c:106`（定义）`:523`（使用处：`timeout = zfs_txg_timeout * hz`） | 读源码（默认值常量），未实测触发频率 | 与 btrfs 的 30 秒相比，ZFS 默认周期更短（5 秒）；两家都比 singlefs「每次 fsync 都全量发布」松得多 |
| TXG 提交时 `zil_sync` 处理已经进了主树的日志记录（推进可丢弃点）；挂载 / 数据集打开时 `zil_replay` 重放尚未落进主树的日志记录 | `module/zfs/zil.c:4135`（`zil_sync` 定义）；`:4784`（`zil_replay` 定义） | 读源码；`zil_replay` 具体被谁在挂载哪一步调用，没有继续往下查（不在 `zil.c` 里，跨文件调用链没跟完） | — |


## 四、负载口径对照：能把机制对到 201 195 / 14 592 / 8 192 这三个数上吗

**先说清这三个数量的是哪个负载，和任务描述的负载不是同一个。** `research/perf-by-milestone.md` 第 231-232 行（`二·二、两盘镜像` 表）与 `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md` 第 160 行、37 行一致：六家这一维的 fio 作业是「刚格式化的空文件系统上 **100 次**『新建 + 写 3000 字节 + fsync 文件 + fsync 目录』」，表里的「每次写字节」是**去掉第一次之后 99 次的平均**（这份文件第 224 行「六家是 100 次的平均或中位（含第一次）」）；每次迭代里有**两次 fsync**（文件一次、目录一次），而且每次迭代都在**新建一个文件**（不是同一个文件反复覆盖写）。任务描述的「一个小文件覆盖写 4100 字节、然后 fsync」是 singlefs 自己「发布 B」那一格的负载（同一份文件第 21 行），六家没有跑过这个负载（第 223 行「六家覆盖写一次写多少没量」）。下面的映射只能就着「新建+写+两次 fsync」这个真实负载去对，对不上的按任务要求写明对不上。

- **OpenZFS mirror，8 192 字节：对得上，但只对上总字节数，对不上次数拆分。** `ZIL_MIN_BLKSZ = 4096`，两块盘镜像 ⇒ `4096 × 2 = 8192`，与表里的数逐字相同；这一格 5 轮之间没有标 ⚠（离散小），与「每次迭代稳定产出同一种最小尺寸的日志写」这个机制吻合。但表里同一行「每次写请求」= 4.00、「每次 FLUSH」= 2.00：如果每次迭代真有两次独立的 `zil_commit`（文件 fsync 一次、目录 fsync 一次），且都各自写一个 4096 字节块，两块盘上应该是 4 次写（2 次 commit × 2 disk）——这一步能通过读源码解释「1 次 commit = 1 个 4096 字节块」，但解释不了「两次 fsync 只算出一次 commit 的数据量」；本机没有插桩验证第二次（目录）fsync 是不是命中了 ZIL 里的「没有新数据、直接返回」的空转路径。这一半算「对不上」，标「没有拆到 bio 级别，只是构成要素吻合」。
- **Bcachefs 双副本，14 592 字节：对不上。** `14 592 ÷ 2 = 7 296`，不是 `block_size`（4096）的整数倍（7296/4096 = 1.78），也不是「每次迭代固定写 N 个块」能直接除尽的数。源码给出的是「按内容变长、取整到块边界」的机制（`vstruct_sectors`），能解释「为什么不是一个固定常量」，解释不了「这个具体平均值背后每次迭代各写了多少」——这本来就是一个跨 100 次不同迭代（每次新建不同的文件，涉及分配位图、目录项、inode 等不同大小的增量）的**平均值**，要拆开得有逐次的字节数，源码读不出来，`.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md` 第 179 行自己也写「它们这 8–15 KiB 写的是什么，E152 到 2026-09-17 为止没量」——这条线索没有新证据推翻。另外这一行「每次写请求」= 6.30（不是整数，两块盘不对称或每次迭代请求数不一致），比同一次跑单盘表（Bcachefs 单盘 2.15，见第 98 行）的两倍（4.30）还多，源码没有解释这个差异，标「对不上」。
- **Btrfs raid1，201 195 字节：对不上，只有构成要素。** 能确认的构成单元：`nodesize = 16384`（日志树节点）、`BTRFS_SUPER_INFO_SIZE = 4096`（每份超级块，`write_all_supers` 每次日志同步都写、raid1 下两块盘各写、每块盘最多 `BTRFS_SUPER_MIRROR_MAX = 3` 份）。`201195 ÷ 16384 ≈ 12.28`，不是整数倍；就算减掉超级块（按两块盘各写 2 份 4096 算，`201195 − 2×2×4096 = 184779`，`184779 ÷ 16384 ≈ 11.28`）依然不是整数。这与 bcachefs 同理：这是 100 次不完全相同的「新建文件」操作的平均值，源码只给出「日志树节点是 16 KiB、超级块是 4 KiB」这两个常量，给不出「平均每次迭代精确碰了几个脏节点」。这一格离散没有标 ⚠，说明 5 轮之间稳定，但稳定不等于能从源码反推出具体拆解，标「对不上，只有构成要素」。

**共性结论（推论，没有走三方论证）**：三家在这个负载上都不是「树写日志一样恒定」，它们的每次持久化字节量是「日志/意图日志按内容变长 + 取整到一个小得多的边界（4 KiB 或 16 KiB 量级）」的产物，而不是像 singlefs 那样「每次都以 32 KiB 单元恒定重写全部脏节点」。这条推论解释了「为什么六家的每次持久化字节量比 singlefs 小得多」这个方向，但解释不了三个具体数字本身——精确拆解需要给这几家插桩或者找它们自己的 trace 工具（比如 btrfs 的 `trace_btrfs_sync_file`，本机没有跑）。


## 五、反证单列：有没有现役写时复制文件系统每次 fsync 都写「脏叶 + 全部祖先 + 根」

**查了哪几家、怎么查的**：本机有源码的三家写时复制文件系统——Btrfs、Bcachefs、OpenZFS。每家都读了它自己的 fsync 系统调用入口函数（`btrfs_sync_file` / `bch2_fsync` / `zfs_fsync`），顺着它的调用链走到真正落盘的那个函数（`btrfs_sync_log` / `bch2_journal_flush_seq` / `zil_commit`），并在那个落盘函数所在的文件里对「有没有调用主树提交、根切换或超级块写」做了一次文件内 grep 核对：
- Btrfs：`btrfs_sync_log` 所在的 `tree-log.c` 里，主树根切换（`commit_root` 赋值）代码在 `transaction.c`，两个文件分开，逐行确认 `tree-log.c` 里没有这段代码；`write_all_supers` 确实被调用（超级块被写，但指向的是日志树根，不是主树根）。
- Bcachefs：`bch2_journal_flush_seq` 所在的 `journal.c` 与它调用的 `journal_io.c` 里，`grep -c bch2_write_super journal_io.c` 命中 0 次。
- OpenZFS：`zil_commit` 所在的 `zil.c` 里，`grep -n "vdev_uberblock_sync\|spa_sync("` 命中 9 处，逐条读上下文确认全部是注释，没有一处是函数调用。

**结论（反证，不是正证）**：这三家没有一家在 fsync 路径上重写主结构的脏叶、全部祖先、和根（或超级块 / uberblock 里指向主结构根的那个字段）；全部走一条独立的、按内容变长的日志 / 意图日志结构，把主结构的落盘挪到周期性（Btrfs 30 秒、ZFS 5 秒）或达到某个回收水位（Bcachefs 日志回收 100 毫秒一轮）时才做。这只说明「三家现役实现都没有走 D23（journal 的角色与格式） 已定项 1 那条『每次 fsync 写全部祖先 + 根』的路」，不能证明那条已定项是错的——`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「没有任何现役实现走 X 这条路」一节：这只把举证责任压过去，不是判决。

**没有查、也不计入这条反证的**：本机没有 Nilfs2、WAFL 或别的写时复制 / 日志结构文件系统的源码（`find /home/fy5090/code/fs-refs` 只有 `linux-6.17` 与 `zfs` 两个目录，`linux-6.17/fs/` 下只解出 `bcachefs` 与 `btrfs` 两个子目录），没有第四家的独立观测。

## 没做什么

- 没有编译、没有跑任何一家的文件系统、没有插桩验证「一次 fsync 到底触发几次块层写请求、大小各是多少」——上面每条都是读源码得出的控制流事实，不是实测的字节流水账。
- 没有拆开 201 195 / 14 592 / 8 192 这三个数到「逐次迭代」的粒度：那需要给 btrfs/bcachefs/zfs 挂 `blktrace` 或它们自带的 tracepoint（`trace_btrfs_sync_file` 等）单独跑一遍 E152 那条负载，这一轮没做。
- 没有查 ZFS `zil_replay` 具体在挂载的哪一步、由哪个函数调用（只确认了它存在、在 `zil.c` 定义），跨文件的调用链没有跟完。
- 没有论证 singlefs 该不该改、改哪一条已定项——那是设计推论，要走 `.claude/rules/three-way-inference.md` 的三方论证，不是这份报告的范围。
- 没有查 Nilfs2、WAFL 或任何第四家写时复制 / 日志结构文件系统，本机没有它们的源码。
- 没有验证 `.claude/singlefs-ai-sop/rules/`、`kb/decisions.md` 里被这份报告引用的各条已定项今天是不是仍是「已定」状态——只按 `.claude/kb/milestone/02-second-txn.md` 第 264-300 行给出的编号原样引用，没有逐条重新去 `kb/decisions.md` 核对状态列。
