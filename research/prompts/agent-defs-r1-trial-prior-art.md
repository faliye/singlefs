# 调研报告：btrfs `btrfs_is_zoned` 在事务日志与 ENOSPC 准入路径的渗透（试跑）

任务来源：主 agent 点名派发的调研员（prior-art）试跑。
要查：Linux btrfs 里 `btrfs_is_zoned` 出现在 `fs/btrfs/` 下多少个文件、一共多少处；
其中 `tree-log.c`（事务日志路径）与 `space-info.c`（ENOSPC 准入）各多少处。
要比：本机两棵源码树 `/home/fy5090/code/fs-refs/linux-6.17` 与
`/home/fy5090/kbuild/linux-om`，以及两棵树之间的差异。

这份调研核实的是本仓已有决策（D17，`.claude/kb/decisions/17-实现分层与第三方管道.md`
第 29–31 行）与项目本地规则（`.claude/rules/fs-design.md` 第 123–133 行）里已经登记的一组数字
今天（2026-09-17）还站不站得住，不产出新决策。

## 第 0 步：70 号门禁阶段（本轮登记给 prior-art 的阶段）

先按 `.claude/agent-common.md`「门禁」一节查登记表：

```
$ awk -F'\t' -v me="prior-art" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv
70-citations.sh
```

`.claude/gate.d/stage-owners.tsv` 第 70 行原文：
`70-citations.sh	kb-scribe,prior-art	外部引用还核得动；写进 kb 的是书记员，查源码树的是调研员`

跑它（`nice -n 19 bash .claude/gate.d/70-citations.sh`），原样末行与退出码：

```
══ 结果：89 条命中，0 条未命中 ══
  ✓ 全部命中
EXIT_CODE=0
```

这一阶段实际调用 `research/scripts/verify-citations.sh`，其中与本轮问题直接相关的四行
（`research/scripts/verify-citations.sh:79-82`）：

```
ckn D17 "btrfs_is_zoned 的文件数"        22 "grep -rn 'btrfs_is_zoned' '$BTR' | awk -F: '{print \$1}' | sort -u | wc -l"
ckn D17 "btrfs_is_zoned 的总处数"        93 "grep -rn 'btrfs_is_zoned' '$BTR' | wc -l"
ckn D17 "其中落在事务日志路径的"          5 "grep -c 'btrfs_is_zoned' '$BTR/tree-log.c'"
ckn D17 "其中落在 ENOSPC 准入的"          6 "grep -c 'btrfs_is_zoned' '$BTR/space-info.c'"
```

其中 `$BTR="$REFS/linux-6.17/fs/btrfs"`、`$REFS` 默认 `/home/fy5090/code/fs-refs`
（`research/scripts/verify-citations.sh:15-17`），即本轮任务给的第一棵树；门禁跑时
四条全部命中（`✓ D17 btrfs_is_zoned 的文件数（22）` 等，见上面 70 号阶段的完整输出）。
这四行只核了树一，没有核树二（本机 7.2.0 树）；树二那组数字（25/105/5/7）没有登记进
`verify-citations.sh` 的断言表，是 `fs-design.md:130-133` 里的一段叙述性文字，70 号阶段
够不着它——下面第二节我自己独立现查补上。

## 一、两棵源码树的出处与版本

| 树 | 路径 | 版本 | 出处 | 核实方式 |
|---|---|---|---|---|
| 树一 | `/home/fy5090/code/fs-refs/linux-6.17` | Linux 6.17（基础版，非 point release） | `/home/fy5090/code/fs-refs/fetch.log`：`[12:17:45] linux tarball 147M`；本地留有 `linux-6.17.tar.xz` | sha256 比对：本地 `9b607166a1c999d8326098121222feb080a20a3253975fcdfa2de96ba7f757a7`，与 `https://cdn.kernel.org/pub/linux/kernel/v6.x/sha256sums.asc` 里 `linux-6.17.tar.xz` 一行逐字相同（2026-09-17 现查，见下方命令） |
| 树二 | `/home/fy5090/kbuild/linux-om` | 7.2.0（`VERSION=7 PATCHLEVEL=2 SUBLEVEL=0 EXTRAVERSION=` 空，代号 `Baby Opossum Posse`，见 `Makefile:2-6`） | 无 `.git`、无补丁目录（`find . -maxdepth 1 -iname "*.patch"` 零命中）、`README` 是通用内核 README（非发行版定制文案）；`fs/btrfs/*.c` 的 mtime 均为 `2026-08-22 23:40:23` | 树本身没有版本控制记录，只能从 `Makefile` 顶部三行与 `README` 反推「像一份未打补丁的上游快照」，**未能进一步核实它的确切取得来源**（这一条记「未在本项目验证」） |

`sha256` 核对命令与结果：

```
$ sha256sum /home/fy5090/code/fs-refs/linux-6.17.tar.xz
9b607166a1c999d8326098121222feb080a20a3253975fcdfa2de96ba7f757a7  /home/fy5090/code/fs-refs/linux-6.17.tar.xz

$ curl -sSL https://cdn.kernel.org/pub/linux/kernel/v6.x/sha256sums.asc | grep -E "linux-6\.17\.tar\.xz"
9b607166a1c999d8326098121222feb080a20a3253975fcdfa2de96ba7f757a7  linux-6.17.tar.xz
```

两者逐字相同（2026-09-17 现查）。⚠️ **口径**：这只核实了树一那份 tarball 与 kernel.org
官方发布的字节一致；`fs-refs/linux-6.17` 目录本身是从这份 tarball**稀疏解出**的
（只留了 `Documentation/` 与 `fs/bcachefs`、`fs/btrfs` 两个子目录，`find` 现查确认，
树里没有顶层 `Makefile`），所以「树一是干净的 6.17」这句话核实到的是 tarball 级别，
不是目录级别的逐文件核对（tarball 本身没有再解压比对）。

树二没有同等强度的出处证据——**这一条本机核不动**，只能记「据 Makefile 自陈是 7.2.0，
出处未知」。

## 二、独立现查的计数（2026-09-17，与 kb 已登记值对照）

统一口径：`grep -rn 'btrfs_is_zoned' <目录>`；文件数取 `awk -F: '{print $1}' | sort -u | wc -l`，
总处数取整条命令 `wc -l`；单文件计数取 `grep -c 'btrfs_is_zoned' <文件>`。命令与输出不截断，
完整贴在下面「原始命令与输出」小节。

| 事实 | 树一 6.17（实测） | kb 登记值（6.17） | 树二 7.2.0（实测） | kb 登记值（7.2.0，叙述性） |
|---|---|---|---|---|
| 命中的文件数 | 22 | 22（D17 分项，`17-实现分层与第三方管道.md:30`） | 25 | 25（`fs-design.md:132`） |
| 总处数 | 93 | 93（`17-实现分层与第三方管道.md:30`） | 105 | 105（`fs-design.md:132`） |
| `tree-log.c` 处数 | 5 | 5（`17-实现分层与第三方管道.md:30`） | 5 | 未单独登记（kb 只登记了 6.17 那一档的 5） |
| `space-info.c` 处数 | 6 | 6（`17-实现分层与第三方管道.md:31`） | 7 | 7（`fs-design.md:132`：「`space-info.c` 从 6 处涨到 7 处」） |

**四项与 kb 里已登记的数字逐字相同**，包括 `fs-design.md:130-133` 那句叙述性的「25 文件
105 处，space-info.c 从 6 处涨到 7 处」——这句话在 70 号门禁的断言表里没有对应条目（见第 0 节），
是我这次单独用同一个口径重新跑出来核对的，不是门禁帮我核的。

口径与出处：本条是**实测**（我自己在两棵树上跑的 grep），不是读文档；`fs-design.md` 与
`17-实现分层与第三方管道.md` 里的原始数字标注是「2026-08-27 现查 Linux 6.17 源码树」
（`fs-design.md:123`）与「2026-08-29 同时现查两棵树」（`fs-design.md:130`）。**未在本项目
（singlefs）里验证**——这组数字只说明 btrfs 今天长什么样，不说明 singlefs 该怎么做。

### 逐文件差异（两棵树都命中的文件，计数有变化的）

| 文件 | 树一 6.17 | 树二 7.2.0 | 差 |
|---|---|---|---|
| `block-group.c` | 11 | 14 | +3 |
| `space-info.c` | 6 | 7 | +1 |
| `inode.c` | 6 | 5 | −1 |
| `volumes.c` | 3 | 4 | +1 |
| `super.c` | 2 | 3 | +1 |
| `extent_io.c` | 1 | 4 | +3 |
| `delalloc-space.c` | 1 | 2 | +1 |

其余 15 个共有文件（`zoned.c` `free-space-cache.c` `zoned.h` `scrub.c` `tree-log.c`
`extent-tree.c` `ioctl.c` `file.c` `dev-replace.c` `bio.c` `sysfs.c` `fs.h`
`disk-io.c` `direct-io.c` `block-rsv.c`）计数不变。

树二独有的 3 个文件（树一没有命中）：`transaction.c`（1 处）、`relocation.c`（1 处）、
`delayed-ref.c`（1 处）。逐项相加：93 + 3(block-group) + 1(space-info) − 1(inode)
+ 1(volumes) + 1(super) + 3(extent_io) + 1(delalloc-space) + 3(三个新文件) = 105，
与总处数对上。

### `space-info.c` 新增的那一处：真实新增，不是行号漂移

`tree-log.c` 两棵树都是 5 处，逐行核对（见下方原始输出）是**同一组调用点**，只是文件里别处
加了代码，行号整体下移（如 174→282、258→353），命中的 5 行本身没变。
`space-info.c` 不同：树二比树一多出一行（`918:		if (btrfs_is_zoned(fs_info) {`），
这一行在树一里不存在。读上下文（`fs/btrfs/space-info.c:911-918`，树二）：

```
	case RECLAIM_ZONES:
		if (btrfs_is_zoned(fs_info)) {
			btrfs_reclaim_sweep(fs_info);
			btrfs_delete_unused_bgs(fs_info);
			btrfs_reclaim_block_groups(fs_info,
						   BTRFS_ZONED_SYNC_RECLAIM_BATCH);
			ASSERT(current->journal_info == NULL);
			ret = btrfs_commit_current_transaction(root);
		} else {
```

这是 ENOSPC flush 状态机（`btrfs_flush_space` 的 `switch (state)`）里新增的一个分支
`RECLAIM_ZONES`，本身就是一次新的、专门为 zoned 介质加的 ENOSPC 准入分支，而不是既有代码
挪了位置。这条实测支持 `fs-design.md:133`「这不是一次性的历史包袱，是持续渗透——每一版都有人
『只是加个 zoned 判断』」这句话，是**实测**，不是读文档得出的。

## 三、已知差异（本工程 vs btrfs 这一处做法）

| 差异维度 | 内容 |
|---|---|
| **盘上格式** | `btrfs_is_zoned()` 定义是 `IS_ENABLED(CONFIG_BLK_DEV_ZONED) && fs_info->zone_size > 0`（`fs/btrfs/fs.h:975-978`，树一 6.17 现查），是一个**挂载时探测底层设备拓扑得到的运行期字段**，不是超级块上的 incompat feature bit；这解释了为什么它要在 93 处调用点各自查一遍，而不是在挂载入口判一次就够——判据本身就分散在每个调用点里。本工程 `.claude/kb/decisions/12-目标介质.md`（D12）与 `.claude/rules/fs-design.md`「格式层的『让非法状态无法表示』」一节的设计意图，是把介质布局身份编码进 incompat 位，让「拿错布局读」在挂载那一刻就失败，而不是散在运行期判断里；**这是设计意图，我没有去查 `crates/` 里今天是不是已经这样实现了，这一条只对照 kb 里记的意图，不对照实现现状** |
| **兼容包袱** | btrfs 要同时挂载「建盘时就是 zoned」与「后来在 zoned 设备上挂载的老镜像」，`zone_size` 只能在挂载时现场探测，不能假设它在格式历史上一直存在；本工程 CLAUDE.md 明写「磁盘格式仍是软的」「第一个外部用户出现前可以随时拆了重做」（`format-evolution.md`），没有这层历史包袱，因此把介质身份收进格式层的代价（一次性改一次格式）远低于 btrfs 今天要付的代价（要在数以百计的调用点上各自判断，因为格式历史锁死了） |

⚠️ **这两条差异不构成「所以本工程该怎么做」的论证**——按 `evidence-discipline.md`「别的项目怎么做，
是线索不是证据」，它们只说明 btrfs 这个具体渗透实例背后有一个可以指出的原因（运行期探测 +
历史兼容），不判断 D12/D17 里选的路径是不是对的；那两条决策的依据是决策文件自己给的，不是这份
调研给的。

## 四、原始命令与输出（不截断）

### 树一（`/home/fy5090/code/fs-refs/linux-6.17/fs/btrfs`）

```
$ grep -rn 'btrfs_is_zoned' "$BTR1" | awk -F: '{print $1}' | sort | uniq -c | sort -rn
     16 zoned.c
     11 block-group.c
      9 free-space-cache.c
      8 zoned.h
      7 scrub.c
      6 space-info.c
      6 inode.c
      5 tree-log.c
      5 extent-tree.c
      3 volumes.c
      2 super.c
      2 ioctl.c
      2 file.c
      2 dev-replace.c
      2 bio.c
      1 sysfs.c
      1 fs.h
      1 extent_io.c
      1 disk-io.c
      1 direct-io.c
      1 delalloc-space.c
      1 block-rsv.c
（文件数 22；总处数 93；tree-log.c=5；space-info.c=6）
```

### 树二（`/home/fy5090/kbuild/linux-om/fs/btrfs`）

```
$ grep -rn 'btrfs_is_zoned' "$BTR2" | awk -F: '{print $1}' | sort | uniq -c | sort -rn
     16 zoned.c
     14 block-group.c
      9 free-space-cache.c
      8 zoned.h
      7 space-info.c
      7 scrub.c
      5 tree-log.c
      5 inode.c
      5 extent-tree.c
      4 volumes.c
      4 extent_io.c
      3 super.c
      2 ioctl.c
      2 file.c
      2 dev-replace.c
      2 delalloc-space.c
      2 bio.c
      1 transaction.c
      1 sysfs.c
      1 relocation.c
      1 fs.h
      1 disk-io.c
      1 direct-io.c
      1 delayed-ref.c
      1 block-rsv.c
（文件数 25；总处数 105；tree-log.c=5；space-info.c=7）
```

### `tree-log.c` 逐行核对（两棵树的 5 处调用点一一对应，只是行号因文件其余部分改动而漂移）

```
-- 树一 --
174:	const bool zoned = btrfs_is_zoned(fs_info);
258:	const bool zoned = btrfs_is_zoned(root->fs_info);
3040:	if (ret == -EAGAIN && btrfs_is_zoned(fs_info))
3075:	if (btrfs_is_zoned(fs_info)) {
3162:	if (ret == -EAGAIN && btrfs_is_zoned(fs_info)) {

-- 树二 --
282:	const bool zoned = btrfs_is_zoned(fs_info);
353:	const bool zoned = btrfs_is_zoned(root->fs_info);
3363:	if (ret == -EAGAIN && btrfs_is_zoned(fs_info))
3398:	if (btrfs_is_zoned(fs_info)) {
3489:	if (ret == -EAGAIN && btrfs_is_zoned(fs_info)) {
```

五对调用点的代码形态（每行去掉行号后）逐一相同，判定为「同一组调用点，行号漂移」。

### `space-info.c` 逐行核对（树二比树一多一处）

```
-- 树一（6 处）--
211:	if (btrfs_is_zoned(fs_info))
254:	if (btrfs_is_zoned(info))
297:	if (btrfs_is_zoned(info)) {
417:	if (btrfs_is_zoned(fs_info))
487:	if (btrfs_is_zoned(fs_info))
1141:	if (btrfs_is_zoned(fs_info))

-- 树二（7 处）--
222:	if (btrfs_is_zoned(fs_info))
265:	if (btrfs_is_zoned(info))
307:	if (btrfs_is_zoned(info)) {
445:	if (btrfs_is_zoned(fs_info))
518:	if (btrfs_is_zoned(fs_info))
918:		if (btrfs_is_zoned(fs_info)) {    <- 新增，见上一节 RECLAIM_ZONES 分支
1194:	if (btrfs_is_zoned(fs_info))
```

前 6 行两棵树逐条对应（去行号后代码形态相同，只是行号从 211/254/297/417/487/1141
漂移到 222/265/307/445/518/1194）；树二多出的第 918 行是新分支，已在上一节贴出上下文。

### `btrfs_is_zoned` 的定义（树一，`fs/btrfs/fs.h:975-978`）

```
static inline bool btrfs_is_zoned(const struct btrfs_fs_info *fs_info)
{
	return IS_ENABLED(CONFIG_BLK_DEV_ZONED) && fs_info->zone_size > 0;
}
```

## 五、反证清单（「没有任何现役实现这样做」这类）

**本轮不产出这类反证。** 任务给的问题范围只是「btrfs 一家、两棵版本树」，不构成对
「有没有现役实现把介质分支挡在格式层、不散进事务日志与 ENOSPC 准入」这个问题的普查——
要下「没有任何现役实现这样做」的判断，至少要看 bcachefs、OpenZFS、F2FS 等本机已有源码树
的对应机制（比如 bcachefs 的多设备/durability 分支、ZFS 的 vdev 分支）是怎么组织的，
这次没有做。`.claude/kb/decisions/17-实现分层与第三方管道.md` 第 1–3 条本身已经查过
OpenZFS 的 vdev 结构与本地模型的正推腿，但那是另一轮已经做完的工作，不在本次任务范围内，
我也没有重新核实那几条。

## 六、没做什么

- 没有查 btrfs 之外的任何文件系统（bcachefs、OpenZFS、F2FS、XFS 等）在这同一个问题上
  怎么做，因此第五节的反证清单是空的，不是查过之后确认没有。
- 没有论证「把介质身份编进 incompat 位」这条设计意图适不适合本工程，也没有去查
  `crates/` 里现在的介质分支处理今天实际长什么样、是不是真的做到了「挂不上而不是静默误读」——
  第三节那条差异只对照 kb 里记的设计意图，不对照 `crates/` 源码。按
  `.claude/rules/implementation-first.md` 的要求，真要把这条差异写进任何方案讨论，
  应该先读 `crates/singlefs-core` 里介质分支相关的代码路径，这次没有做，因为任务范围
  是查源码树，不是查本仓实现。
- 没有在本项目里验证这份调研的任何一条结论——按调研员角色定义与
  `evidence-discipline.md`「别的项目怎么做，是线索不是证据」，这些数字只是关于 btrfs
  的线索，不是关于 singlefs 该怎么做的证据。
- 树二（`/home/fy5090/kbuild/linux-om`）的确切取得来源没有查清楚（无 git、无下载日志），
  这一条本机核不动，第一节已标注。
- 没有验证两棵树里除 `btrfs_is_zoned` 之外的其余介质分支写法（比如 bcachefs 那一侧、
  或者 btrfs 里另一个类似的运行期开关）是不是也是同样的渗透模式——任务只问了这一个符号。
- 没有跑 `gate.sh` 全量（按 `.claude/agent-common.md`「不做」一节，未经主 agent 明写允许
  不跑全量门禁；本轮只跑了登记给 prior-art 的单个阶段 70 号）。

## 七、试跑观察（调研员定义哪里不清楚、做不下去、与实际对不上）

1. **「阶段归属表登记给你的阶段」这句指代的是哪张表，定义里没写清楚，靠猜。**
   调研员定义原文是「先跑阶段归属表登记给你的阶段（共用约束『门禁』一节）」，
   而实际登记表 `.claude/gate.d/stage-owners.tsv` 与查询命令都写在 `agent-common.md`
   里，不在调研员自己的定义文件里。这次因为读过 `agent-common.md` 才知道去哪查、
   命令怎么写；如果调研员定义要单独发给一个没读过 `agent-common.md` 的模型，
   这一步会卡住。**建议**：调研员定义里直接给出那条 `awk` 命令，或者至少给出
   `stage-owners.tsv` 的路径，不要只说「共用约束『门禁』一节」。

2. **「70 号会报哪些承重引用指向的源码树或文献不在本机了」这句，与本轮实际情况不完全对得上。**
   70 号阶段（`verify-citations.sh`）在源码树缺失时的行为，是**逐条**报告某条断言未命中
   （因为 grep 在一个不存在的路径上会返回空，与预期值不等就判 `✗`），退出码非零；但它
   打印的失败信息形态是「某条 D 编号的某句话」+「实测值 vs 预期值」，不是一句直接说
   「这棵树不在本机」的话。这次两棵树都在，没有踩到这条，**没能验证**调研员定义这句话
   描述的失败信息形态是不是准确——这是「试跑观察」里唯一一条我没法用今天这轮实测去坐实
   或推翻的。

3. **调研员定义要求的「事实 / 出处与日期 / 口径 / 与本工程的差异」四列表，与「每条写一处
   它与本工程的已知差异」这句自然语言指示之间，落到具体写法时有一点模糊**：一条事实
   （比如「文件数是 22」）本身不天然带有一个「与本工程的差异」——差异是**做法层面**的
   （btrfs 怎么组织这个分支 vs 本工程打算怎么组织），不是**数字层面**的。这次处理方式是把
   四个数字合并成「一条渗透模式」的事实，再对这一条事实整体写差异，而不是给「文件数」
   「总处数」「tree-log.c 处数」「space-info.c 处数」四个数字各写一条差异——四个数字各写
   一条差异会变成同一句话抄四遍。**建议**：定义里说清楚「差异」是挂在「一组数字背后的
   做法」上，还是挂在「表格的每一行」上；这次我按前者理解，如果不对，第三节的写法需要拆开。

4. **「差异不明，只能当线索」这句话在本轮没有用上**——两条差异（盘上格式、并发模型/
   兼容包袱其中之一）都想得出来，写得也算站得住，所以没有触发这条兜底。不确定这是因为
   本轮问题恰好好写差异，还是这条兜底本身描述的场景（差异想不出来）在多数调研任务里
   都不常见——这条**没有反面案例**，无法判断这句兜底写得够不够用。

5. **草稿目录 `draft/` 在本轮没有用上。** 因为这次任务的证据链条不长（两次 grep +
   一次 sha256 核对），中间结果不需要落盘草稿就能直接写进报告；`draft/` 目录建了但是空的。
   不确定调研员定义预期的「草稿」是指什么阶段的产物——如果是指「先写一份不完整的报告草稿
   再定稿」，这次没有走这个流程，直接一次成文；如果指的是「中间取数用的临时文件」，
   这次的中间取数（grep 输出）都直接内联进了报告，也没有落草稿。**建议**：定义里如果
   要求必须使用草稿目录，需要说清楚草稿放什么、报告与草稿的关系是什么。

6. **70 号阶段的输出没有单独标出「树二」这组数字**（见第 0 节结尾），这不是阶段本身的
   缺陷（它的断言表本来就只登记了树一），但如果调研员的任务恰好是「核对 kb 里两棵树的
   数字」，跑一遍门禁并不能覆盖任务的全部范围，还是要自己现查——这次这一点在定义里
   没有被提醒，是我读了 `verify-citations.sh` 源码才发现的。
