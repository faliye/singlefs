# E112 对抗腿（攻击面：臂集完不完整 / 有没有稻草人对照臂）

**日期**：2026-09-06　**腿**：云端 B（Opus），攻击立场
**现查范围**：本仓 kb 与 E112 源码 / 产物；本机内核树 `/home/fy5090/code/fs-refs/linux-6.17/`（含 `fs/btrfs`、`fs/bcachefs`）；
参数敏感性在仓外副本 `scratchpad/e112-copy` 上跑（工作区未动，`sed` 每次断言命中并回读）。

---

## 一、臂集完不完整

### （a）打的是什么

`flags` 轴只有 `Ignore / Skip / Refuse` 三个值，`记账` 轴只有 `Carry / NoCarry` 两个值。
我要证明的是：**这两条轴上都缺了本仓自己已经定过、或现役实现正在用的那一档**，
而缺掉的那几档不是"想得出来但没人会用"，恰恰是最强的对手。

### （b）现查到的原文

**① 本工程对"未知 flags 位"已经定过两次，两次都不是跳过，是拒收：**

`.claude/kb/decisions/18-块里携带什么信息.md:632` 逐字——

> **偏移 7 是 8 个 flag 位，第一版恒 0，读者遇到非 0 位一律拒收**——这是前向兼容政策，不是损坏检出（头校验和已经罩着它）：本工程不给这 8 位留「忽略陌生位」的通道 ⇒ 将来给任一位赋义按 D15（格式冻结政策）「改变已有字段含义」判 incompat；

`.claude/kb/decisions/08-核心索引结构.md:396` 逐字——

> **flags / 填充 / 预留三段同一政策：恒 0，读者遇到非 0 ⇒ 该记录 EIO（对象级，不拒整个容器）**——前向兼容政策不是损坏检出（载荷校验和照样过），落成 I-9.7（记录三段恒零），在 I-1.7（打包容器合法与判定顺序） 的中止链之外。

`.claude/kb/invariants.md:259` 逐字——

> | I-9.7 | 记录三段恒零 | 打包记录类型 2 的记录偏移 108 / 112 / 120 三段（填充 4 / flags 8 / 预留 20）恒 0；非 0 判该记录 EIO，容器其余记录照读（前向兼容政策，D15（格式冻结政策） 判定表；不是损坏检出，载荷校验和照样过） | 未实现 |

⇒ 缺的第一条臂是 **`RejectEntry`：未知位 ⇒ 该条目 EIO，挂得上、别的树照读**。
它在 E112 的坐标里既不是 `Skip`（跳过是无声的，EIO 是有声的），也不是 `Refuse`（不拒整个挂载）。
**E112 的「被引用条款」表里一条 D18 都没有**——它把本工程对同一个问题已有的两处定案整个漏在外面。

**② D15 的判定表自己就有第三档，而且明写这一档要分开判：**

`.claude/kb/decisions/15-格式冻结政策.md:90` 逐字——

> | 新增记录类型（新 key 类型） | 只读遍历可能误判；整理 / 写可能误删看不懂的记录 | compat_ro | 是 | **「只读安全」与「可写安全」是两条独立保证**，必须分开判。⚠️ 打包记录单元内的打包记录类型**不走这一行**（D18（块里携带什么信息） 已定项 11）：跳过一个墓碑类或 inode 类记录会改变只读答案（E59（缓冲消息能不能只从单元重算）），默认档是 incompat；只有登记时能证明「跳过它不改变任何只读答案」的类型才降到 compat_ro，且 compat_ro 读者不许跑扫描重建 |

同一张表的下一行才是 E112 用来杀 `Refuse` 的那条：

`.claude/kb/decisions/15-格式冻结政策.md:91` 逐字——

> | 新增一棵 keyspace | 若超级块的树表 day-1 就是开放列表：只是看不到新树；若树数量硬编码：读不出树表 | **compat**（已定：树表 day-1 做成开放列表，见已定项 2）。留了开放树表 → compat；没留 → incompat | 是 | …… |

⇒ 缺的第二条臂是 **`ReadOnly`：未知位 ⇒ 只读挂载（compat_ro）**。
E112 的实验正文第 16 行**抄了这三档**（"三档：incompat 不认识不许挂 / compat_ro 不认识只读挂 / compat 不认识随便"），
**抄进了「被引用条款」表，却没在模型里建成一条臂**。

**③ 同构场景的 compat_ro 要求本仓已经欠着了：**

`.claude/kb/checks-owed.md:81` C71（不认识的统计量标签静默忽略）逐字——

> **它若静默忽略，自己的 ENOSPC 不等式就少一项而毫无察觉**——那不是 compat，至少是 compat_ro。……运行期的形态是挂载时遇到不认识的统计量标签必须按 compat_ro 拒绝可写挂载，并做成能被测试强制进入的分支

⇒ 「旧读者碰到自己不认识的东西，而那东西进了它的记账」，本仓已经判过一次，判的是 **compat_ro 拒绝可写挂载**。
E112 对一个结构完全同型的问题给出了相反的答案，**而两个答案没有在任何地方对质过**。

**④ 记账轴缺的那一档，是唯一比 `Carry` 便宜的那一档：**

`.claude/kb/decisions/05-快照-空间记账机制.md`（D5 已定项 2，E112 自己引的那条）说没人重写的行过 K 代消失。
于是"不搬"还有第三条出路：**豁免这批行的 K 代点删**（写代价 0，比 `Carry` 的每代 2 行更省）。
它要改 D5 已定项 2，所以**不是免费的**，但 E112 连量都没量就没有它——
`Carry` 是在只有一个对手（`NoCarry`，明显更差）的情况下赢的。

**⑤ 模型里 `Carry` 搬的东西，不是写者非搬不可的那样东西：**

`.claude/kb/decisions/22-单元原子性怎么合成.md:381` 逐字——

> 间接层的代价是每次发布多 COW 一个树表单元，目标负载上 **+7.1% 块**，知情接受。

⇒ **树表每次发布整个 COW 重写**。所以一个 v1 写者每发布一代，都必须把它不认识的**树表条目本身**原样写回去，
否则那棵树当场从池子里消失。E112 的 `Carry` 逐字只定义在**记账行**上
（源码 `:59` 逐字「把不认识的记账行原样带到新代。」），**条目本身的搬运一个字都没有**。
丢一条记账行是丢一个统计量，丢一条树表条目是丢一整棵树——两者差着几个数量级，而模型只有前者。

**⑥ 逐条判"撞不撞已定条款"**（判据：撞了就说明它本来就出局、不算漏）：

| 提出的臂 | 撞不撞已定条款 | 判 |
|---|---|---|
| `RejectEntry`（未知位 ⇒ 该条目 EIO） | **不撞**，反而是 D18:632 / D8:396 对另外两个 flags 字段已定的政策 | **算漏，要补** |
| `ReadOnly`（未知位 ⇒ 只读挂载） | **不撞**。E112 用来杀 `Refuse` 的 D15 已定项 2 逐字只讲**旧读者**（"对旧读者只是看不到新树"），而 D15:90 逐字要求"只读安全与可写安全……必须分开判" | **算漏，要补** |
| 按位分档（16 位里切 compat / compat_ro 两段） | **不撞**。D15:90 对打包记录类型用的正是这个举证结构（"只有登记时能证明……才降到 compat_ro"） | **算漏，要补** |
| `Skip + 记一条警告` | 不撞，但在 E112 的度量下与 `Skip` 同格：`silent()` 只看 `caught_today`，日志里一行警告不是任何 checker 会红的东西 | 记一笔账（要先定义"警告算不算被抓"） |
| 记账侧：显式墓碑 | 不撞，但**它自己也会被 K 代点删**（墓碑同样要每代重写才活得下来）⇒ 与 `NoCarry` 同归 | 记一笔账，可判出局，但**理由要写下来** |
| 记账侧：豁免那棵树的行不点删 | **撞 D5 已定项 2**（只保留最近 K 代），要重开 D5 | 记一笔账；它是 `Carry` 唯一的真对手，写代价 0 |
| 记账侧：只搬 checker 认得的统计量 | **撞 C71**——那正是 C71 点名要禁的静默忽略 | **本来就出局，不算漏** |

### （c）打中到什么程度

**臂集要补并重跑。** 缺的是三条，其中 `ReadOnly`（compat_ro）那一条**足以推翻"唯一"这个结论**：
它 `mount_ok = 1`（只读挂得上）⇒ 进得了 `main()` 第 201 行 `if o.mount_ok` 那道筛子；
只读 ⇒ 一代不发 ⇒ `rows_lost = 0`、`misread_records = 0` ⇒ `silent() = 0`。
于是 `winners` 变成 `ReadOnly + Skip_Carry`，`absolute_verdict` 里
`assert_eq!(winners, vec!["Skip_Carry".to_string()])` 当场变红。

---

## 二、`Ignore` 是不是稻草人

### （a）打的是什么

按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 逐字的判据——
**"对面那条臂的样子，支持那条路的人认不认"**。我去现役实现里找：
真有人对**一条根 / 树表条目上的 flags 字段**实现"忽略未知位、继续按已知语义解释"吗？

### （b）现查到的原文

**btrfs 对它的树表条目（`btrfs_root_item`）上的未知 flags 位——拒收，不是忽略：**

`fs/btrfs/tree-checker.c:1191-1192`——

```c
	const u64 valid_root_flags = BTRFS_ROOT_SUBVOL_RDONLY |
				     BTRFS_ROOT_SUBVOL_DEAD;
```

`fs/btrfs/tree-checker.c:1263-1269`——

```c
	/* Flags check */
	if (unlikely(btrfs_root_flags(&ri) & ~valid_root_flags)) {
		generic_err(leaf, slot,
			    "invalid root flags, have 0x%llx expect mask 0x%llx",
			    btrfs_root_flags(&ri), valid_root_flags);
		return -EUCLEAN;
	}
```

**btrfs 对 inode 上的未知 flags 位——按位分档，高低两段各判各的：**

`fs/btrfs/inode-item.h:75-80`——

```c
static inline void btrfs_inode_split_flags(u64 inode_item_flags,
					   u32 *flags, u32 *ro_flags)
{
	*flags = (u32)inode_item_flags;
	*ro_flags = (u32)(inode_item_flags >> 32);
}
```

`fs/btrfs/tree-checker.c:1170-1182`——

```c
	btrfs_inode_split_flags(btrfs_inode_flags(leaf, iitem), &flags, &ro_flags);
	if (unlikely(flags & ~BTRFS_INODE_FLAG_MASK)) {
		inode_item_err(leaf, slot,
			       "unknown incompat flags detected: 0x%x", flags);
		return -EUCLEAN;
	}
	if (unlikely(!sb_rdonly(fs_info->sb) &&
		     (ro_flags & ~BTRFS_INODE_RO_FLAG_MASK))) {
		inode_item_err(leaf, slot,
			"unknown ro-compat flags detected on writeable mount: 0x%x",
			ro_flags);
		return -EUCLEAN;
	}
```

⇒ **未知 incompat 位一律拒；未知 ro-compat 位只在可写挂载时拒。** 这就是"按位分档 + compat_ro"，
一个字不差地长在一个**每条目的 flags 字段**上，不是超级块。

**btrfs 在超级块那一层对未知 compat_ro 位——只读挂，且连日志重放都不许：**

`fs/btrfs/disk-io.c:3226-3231`——

```c
	if (compat_ro_unsupp && is_rw_mount) {
		btrfs_err(fs_info,
	"cannot mount read-write because of unknown compat_ro features (0x%llx)",
		       compat_ro);
		return -EINVAL;
	}
```

`fs/btrfs/disk-io.c:3233-3244`——

```c
	/*
	 * We have unsupported RO compat features, although RO mounted, we
	 * should not cause any metadata writes, including log replay.
	 * Or we could screw up whatever the new feature requires.
	 */
	if (compat_ro_unsupp && btrfs_super_log_root(disk_super) &&
	    !btrfs_test_opt(fs_info, NOLOGREPLAY)) {
		btrfs_err(fs_info,
"cannot replay dirty log with unsupported compat_ro features (0x%llx), try rescue=nologreplay",
			  compat_ro);
		return -EINVAL;
	}
```

**bcachefs 对不认识的树——留着、不校验、每次写 journal 原样发回去（这是 `Carry` 的现役形态）：**

`fs/bcachefs/recovery.c:503-521`——

```c
		if (fsck_err_on(entry->btree_id >= BTREE_ID_NR_MAX,
				c, invalid_btree_id,
				"invalid btree id %u (max %u)",
				entry->btree_id, BTREE_ID_NR_MAX))
			return 0;

		while (entry->btree_id >= c->btree_roots_extra.nr + BTREE_ID_NR) {
			ret = darray_push(&c->btree_roots_extra, (struct btree_root) { NULL });
```

`fs/bcachefs/btree_cache.h:120-123`——

```c
static inline unsigned btree_id_nr_alive(struct bch_fs *c)
{
	return BTREE_ID_NR + c->btree_roots_extra.nr;
}
```

`fs/bcachefs/btree_update_interior.c:2775-2782`——

```c
	for (i = 0; i < btree_id_nr_alive(c); i++) {
		struct btree_root *r = bch2_btree_id_root(c, i);

		if (r->alive && !test_bit(i, &skip)) {
			journal_entry_set(end, BCH_JSET_ENTRY_btree_root,
					  i, r->level, &r->key, r->key.k.u64s);
```

`fs/bcachefs/bkey_methods.c:183-184`——

```c
	if (type >= BKEY_TYPE_NR)
		return 0;
```

⇒ bcachefs 对不认识的 btree**根**：收进 `btree_roots_extra`、每次写 journal 随 `btree_id_nr_alive` 一起发回去、
键值一概不校验（`return 0`）。**与本工程一处差异**：bcachefs 的 `bkey` 自带 `u64s` 长度，
它敢不校验是因为解得开；本工程的索引节点条目宽按树不同而节点头不带条目形态
（D8 未定项 8 自己写下的那条反证）⇒ 这条做法**不能整条搬过来**，只能引它证明"每次发布把不认识的根原样写回去"是现役做法。

### （c）打中到什么程度

**`Ignore` 确实是稻草人，但打法和预想的相反——`Skip` 也没有现役先例。**

- 我在两棵现役树里**一处也没找到**"对一条根 / 树表条目上的未知 flags 位，忽略它、继续按已知语义解释"。
  btrfs 在那个位置上返回 `-EUCLEAN`。⇒ `Ignore` 那 4000 条误读，量的是一条没人会实现的路。
- 但同一份现查也说明：**`Skip`（无声跳过整条条目）同样没有先例**。
  现役实现在这个位置上分两档——**拒收**（`RejectEntry`，btrfs root/inode incompat 位）
  与**只读挂**（`ReadOnly`，btrfs inode ro-compat 位 + 超级块 compat_ro）。
  E112 的六个格子里，这两档**一个都没有**。
- 所以这一条不是"打赢了一个稻草人所以结论存疑"这么轻。它是：**天平的两边同时被换掉了**——
  赢的那条臂没人用，输的那条臂也没人用，而现役的两条路一条都没上称。

⇒ **臂集要补一条并重跑**（与第一节同一结论，此处是外部证据侧的独立支持）。

---

## 三、模型的参数取值是不是被挑过

### （a）打的是什么

`t_new=4 h_new=2 k_keep=4 g_gens=10 records_per_new_tree=1000` 五个数，
一个都没有 kb 出处（源码 `:29-45` 的注释只解释含义，不给依据）。
我在**仓外副本**上逐个换值重跑，看结论翻不翻。

### （b）现查到的原文（本轮实测，副本 `scratchpad/e112-copy`，基线先复现过与 `research/results/e112-old-writer-unknown-tree-2026-09-06.out` 逐行相同的输出）

**变体 A：`H_NEW = 0`（不认识的树里一个可写头都没有）**

```
name=cell arm=Skip_Carry   mount_ok=1 misread_records=0 rows_lost=0 rows_written_per_gen=17 silent_today=0 silent_by_spec=0
name=cell arm=Skip_NoCarry mount_ok=1 misread_records=0 rows_lost=0 rows_written_per_gen=17 silent_today=0 silent_by_spec=0
name=carry_cost extra_rows_per_gen=0 own_rows_per_gen=17 threshold_rows=144 over_threshold=0
name=verdict min_silent_today=0 winners=Skip_Carry+Skip_NoCarry
--- test result: FAILED. 5 passed; 6 failed
```

这正是变异表里的 M9（`research/mutations/e112_old_writer_unknown_tree.tsv` 第 9 行
`M9_不认识的可写头数归零	const H_NEW: u64 = 2;	const H_NEW: u64 = 0;`），
`research/results/e112-mutate-2026-09-06.log` 逐字——

> ✅ [M9_不认识的可写头数归零] 红：absolute_generation_boundary,absolute_carry_cost,absolute_rows_per_gen,absolute_silent_counts,absolute_verdict,positive_control_skip_nocarry_loses_rows

⇒ **M9 红的不只是"常量被改了"这件事。** 它红在 `absolute_verdict` 上，
意思是 `H_NEW = 0` 时 **`Carry` 与 `NoCarry` 完全打平**：多写 0 行、少丢 0 行。
㊀ 那条定案（"写者必须把不认识的记账行原样带到新代"）在这个点上**没有任何证据支持**——
它既不错也不对，是**无对象**。

**变体 C：`K_KEEP = 20`（G < K，代际丢弃看不见）** ⇒ 同样 `winners=Skip_Carry+Skip_NoCarry`。

**变体 E：`RECORDS_PER_NEW_TREE = 0`（未知位不改记录编码）**

```
name=verdict min_silent_today=0 winners=Ignore_Carry+Ignore_NoCarry+Skip_Carry
   arm=Ignore_Carry   misread_records=0 rows_lost=0 rows_written_per_gen=19 silent_today=0 silent_by_spec=0
   arm=Skip_Carry     misread_records=0 rows_lost=0 rows_written_per_gen=19 silent_today=0 silent_by_spec=0
```

**变体 F：`T_NEW = 1, H_NEW = 1`** ⇒ `winners=Skip_Carry`，结论不变（只是量变小）。

### （c）打中到什么程度

**记两笔账，其中第二笔接近"结论要改"。**

1. **㊀ 的证据强度只活在 `H_NEW > 0 ∧ G > K` 这个区间里**，两条边界一碰就打平。
   而 `H_NEW = 2 / T_NEW = 4`（不认识的树里一半是可写头）这个比例**全仓没有出处**。
   实验正文自己也写着"暴露面很窄……不是可写头就一行都没有"——
   那句话是对的，但它同时意味着**这个实验的判别力全靠一个没有出处的参数撑着**。
   按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「我是不是只在一个点上取了值」，这一条中招。

2. **`Ignore` 输掉的那 4000 条，来源是一个手打的常量加一个没写下来的假设**：
   假设"未来某个 flags 位会改记录编码"。源码 `:119` 逐字 `misread_records: T_NEW * RECORDS_PER_NEW_TREE`——
   这不是从任何机制推出来的，是直接赋值。
   而**现查到的两个真实 root flag 位是 `BTRFS_ROOT_SUBVOL_RDONLY` 和 `BTRFS_ROOT_SUBVOL_DEAD`**
   （`fs/btrfs/tree-checker.c:1191-1192`）——**都不改记录编码**。
   把常量改成 0（变体 E），`Ignore` 立刻与 `Skip_Carry` 并列第一。
   ⇒ 忽略一个 `RDONLY` 位的真实代价是"往只读快照里写"，忽略一个 `DEAD` 位的真实代价是
   "把一个删了一半的子卷救活"——**两样都很严重，而模型里一个量纲都没有**。
   模型给 `Ignore` 记的是一笔它多半不会犯的错，漏掉的是它真会犯的那两笔。

---

## 四、「静默」这个判据本身

### （a）打的是什么

`silent()`（源码 `:79-82`）逐字 `let wrong = self.misread_records > 0 || self.rows_lost > 0; u64::from(wrong && !self.caught_today)`。
我打三处：**这个向量漏了谁的代价**、**"唯一"是怎么来的**、**"今天"这一列有没有信息**。

### （b）现查到的原文

**① "唯一"这个词与产物直接冲突。** `research/results/e112-old-writer-unknown-tree-2026-09-06.out` 第 6、7 行逐字——

```
E7RESULT name=cell arm=Refuse_Carry mount_ok=0 misread_records=0 rows_lost=0 rows_lost_closed_form=0 rows_written_per_gen=0 caught_by_spec=0 caught_today=0 silent_today=0 silent_by_spec=0
E7RESULT name=cell arm=Refuse_NoCarry mount_ok=0 misread_records=0 rows_lost=0 rows_lost_closed_form=0 rows_written_per_gen=0 caught_by_spec=0 caught_today=0 silent_today=0 silent_by_spec=0
```

而 `.claude/kb/decisions/08-核心索引结构.md:432` 逐字写的是——

> **六个格子里 `Skip + Carry` 是**唯一今天与按规范都零静默结局**的一个**

**六个格子里有三个 `silent_today=0 silent_by_spec=0`。** 那个"唯一"是 `main()` 第 201 行
`if o.mount_ok {` 这道筛子造出来的，不是比出来的——`Refuse` 从来没上过称。

**② 把 `Refuse` 踢出称的那个数是手打的字面量，没有任何断言盯着它。**
源码 `:232` 逐字——

```rust
            "name=refuse_cost unmountable_gens={G_GENS} compat_with_d15_item2=0"
```

全文件 `compat_with_d15_item2` 只出现这一次（`grep -c` = 1），**11 个单测里一条都没提它，9 条变异里一条都没碰它**。
判据 5 要的"是否相容（0/1）"就是这里手打的那个 `0`。
按 `.claude/singlefs-ai-sop/rules/test-discipline.md` 逐字"这条结论的算术是从哪来的，那部分有断言盯着吗"，这一格是空的。
⚠️ 顺带：`.claude/kb/experiments/112-旧写者遇到不认识的树.md:127` 写着"9 条变异全抓、0 盲区"，
而同一份规则逐字"**不许写「零盲区」**，只能写「N 条变异全部被抓」"。

**③ "今天"那一列不含信息。** 产物六行 `caught_today` 全是 `0`
⇒ `silent_today ≡ wrong` ⇒ 四个指标里进排名的只剩"这条臂到底出没出错"这一个布尔。
"今天 / 按规范"这条轴只在 C156 那笔顺带的账上有用，对排名一点作用没有。

**④ 最要命的一处：`Skip` 的失败模式根本没进 outcome 向量。**

同一天、同一个未定项的定案 ① 逐字（`.claude/kb/decisions/08-核心索引结构.md:432`）——

> **① 快照进树表**（每个快照一条条目），条目加**诞生 txg 8 字节**——它给 I-3.7（活快照可枚举） 一个载体。

`.claude/kb/invariants.md:123` 逐字——

> | I-3.7 | 活快照可枚举 | 盘上存在一个可枚举的持久结构，其元素恰为当前全部活快照（既不多也不少） | 未实现 |

`.claude/kb/decisions/05-快照-空间记账机制.md:5-10` 逐字——

> - 删除时把块的代号与**活头记录的 `prev_snap_txg`**（上一个快照的代号）比：
>   `birth > prev_snap_txg` → **直接释放**；否则 → 追加进**活头自己的 deadlist**。
> - 建快照时把**活头的 deadlist 整个交接给新快照**，活头换一个空的。
> - 销毁快照时把它的 deadlist **合并到下一个更新的那一侧**，并按 `birth > prev(S).txg`
>   过滤出可释放的条目。不做全盘扫描。

`.claude/kb/invariants.md:120`（I-3.5 引用区间的精确性）逐字——

> 可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：**按 txg 升序枚举全部快照根与活头**得 `Reach(b)`……

（顺带：`.claude/kb/decisions/08-核心索引结构.md:432` 引这条时写的是 `invariants.md:119`，本轮现查它在 **:120**，行号漂了一行。）

而 E112 的 `T_NEW` 定义逐字（源码 `:32-33`）——

> /// v1 不认识的树总数（新种类，或**已知种类但置了 v1 不认识的 flags 位**）。

⇒ **把这几条串起来**：一个普通的、活着的快照，只要将来被置上一个 v1 不认识的 flags 位，
v1 就"跳过该条目"⇒ 它从 v1 眼里的快照集合里消失 ⇒
v1 销毁相邻快照时，"下一个更新的那一侧"认成了错的邻居、`prev(S).txg` 取成了错的值 ⇒
**按 `birth > prev(S).txg` 释放掉仍被那个隐形快照引用的块**。
这是**数据没了**，不是"少了一行统计"。
而 E112 给 `Skip` 的这一格记的是 `misread_records: 0`（源码 `:141`）、`rows_lost` 在 `Carry` 下取 0（源码 `:130`），
`silent_by_spec = 0`。**它给对手的最坏情况记了 4000，给自己这条臂的最坏情况记了 0，而两边都没有机制推导。**

⚠️ 这一格 checker 也接不住：v1 的 checker 同样跳过那棵树，两侧一起少一项，
I-3.1（已分配统计 == 遍历和）**照样平**。

**⑤ D15 判定表已经把这个失败模式写下来了**，就在第 90 行那一行的第二列——
"**整理 / 写可能误删看不懂的记录**"，档次 **compat_ro**，理由列逐字"**「只读安全」与「可写安全」是两条独立保证**，必须分开判"。
E112 用来杀 `Refuse` 的是同一张表的第 91 行，而那一行的现象列逐字只讲**旧读者**："只是看不到新树"。

### （c）打中到什么程度

**结论要改。** "按静默最少选"这个选法本身有三个洞，第三个是致命的：

1. 它把 `Refuse`（可用性代价 10 代全挂不上）用一道 `mount_ok` 筛子请出了称，
   而不是把那个代价放到同一个天平上——于是"唯一"这个词是筛子造的。
   补上 `ReadOnly` 那条臂之后这个洞立刻显形：它 `mount_ok = 1`、`silent = 0`，
   **进得了称、还打平**，而它的真实代价（旧写者不能写）与 `Skip` 的真实代价（旧写者写瞎了）
   **一个都不在这个判据里**。
2. 它是布尔，4000 条误读与 2 行丢失同权；`caught_today` 全 0 使"今天"这条轴退化。
3. **它的 `wrong` 只认两种错（误读、丢行），而 `Skip` 犯的是第三种错**：
   在不完整的根集合上做生死判定。这一种既不在 `misread_records` 里，也不在 `rows_lost` 里，
   ⇒ 按这个判据 `Skip` 恒等于"零静默"，**不是因为它对，是因为它错的那一格没有量纲**。
   这与 C156 记下的那笔账是同一个形状，而 C156 只记了 `Ignore` 那一半。

---

## 五、如果结论要改，改成哪一条

**要改。改的是 ㊁，㊀ 跟着降级；两条都要重跑。**

### 改法

**㊁ 改成：flags 其余 15 位按位分档，默认档是 compat_ro，不是 compat。**

| 位段 | 旧读者 | 旧写者 | 举证责任 |
|---|---|---|---|
| **默认（未登记的位）** | 跳过该条目，且**不许跑扫描重建、不许做任何依赖「全部根」的判定**（I-3.5 / I-3.7 / D18 代际回收） | **拒绝可写挂载（compat_ro）** | 无——这是默认 |
| **登记为 compat 的位** | 跳过该条目 | 照常写 | 登记时必须证明「**看不见这棵树，不改变任何写侧判定**」——分配、回收、deadlist 合并与过滤、快照枚举，逐项 |

### 依据

1. **D15:90 逐字「「只读安全」与「可写安全」是两条独立保证，必须分开判」**，
   而 ㊁ 现在的全部依据是 D15:91（那一行的现象列逐字只说**旧读者**"只是看不到新树"）。
   **写者那一侧从来没有被判过。** 这不是我推的，是 D15 自己那张表要求的动作没做。
2. **D15:90 的现象列逐字「整理 / 写可能误删看不懂的记录」，档次 compat_ro。**
   本工程把这一档给了"新增记录类型"；而"一棵整树对写者隐形"比"一类记录对写者隐形"严格更宽。
   同一张表不能对更宽的那个给更松的档。
3. **C71 已经为结构同型的问题判过 compat_ro**（`checks-owed.md:81` 逐字"必须按 compat_ro 拒绝可写挂载"）。
   两处给相反答案，按 `kb-discipline.md`「矛盾比空白更糟」必须并成一条。
4. **现役实现在这个位置上就是这么分的**：btrfs 的 inode flags 低 32 位未知即拒、
   高 32 位未知**只在可写挂载时**拒（`tree-checker.c:1170-1182`）；超级块未知 compat_ro 位
   只读挂且**连日志重放都不许**（`disk-io.c:3226`、`3238-3243`）。
   ——按 `evidence-discipline.md`，这只当**机制**与**该测哪条路径**用，不当正证；
   与本工程的一处已知差异：btrfs 的 root item 是可以整条拒的独立条目，
   本工程的树表条目住在每次发布都要整个 COW 的树表单元里（D22:381），拒一条的下游代价不同，这一条必须自己量。
5. **零静默这个招牌保不住**：`Skip` 在不完整根集合上做 D5 的 deadlist 合并与过滤，
   会释放仍被引用的块，而 v1 的 checker 同样看不见那棵树 ⇒ I-3.1 照样平。
   ⇒ `Skip` 的 `silent_by_spec` 不是 0。

**㊀（搬运）降级为「compat 位那一档才用得上的一条规则」，并把搬运对象补全。**
理由：默认档取 compat_ro 之后，旧写者根本不写 ⇒ 一行都不会丢 ⇒ ㊀ 在默认档上**无对象**。
它只在"登记为 compat 的位"那一档还有意义，而那一档的写代价与收益 E112 都没量对——
`H_NEW = 0` 时收益归零（本轮实测变体 A），且搬运对象漏了**树表条目本身**
（D22:381 每次发布 COW 整个树表单元 ⇒ 不原样写回去那棵树当场消失；
bcachefs 的 `btree_roots_extra` + `btree_id_nr_alive` 是这件事的现役形态）。

### 重跑要补的东西

| 轴 | 补什么 |
|---|---|
| flags 臂 | 加 `RejectEntry`（该条目 EIO）、`ReadOnly`（compat_ro 只读挂）、`PerBit`（分档） |
| 记账臂 | 加「豁免 K 代点删」（写代价 0，要改 D5 已定项 2）；`Carry` 拆成「搬记账行」与「搬树表条目」两项 |
| outcome 向量 | 加 `blocks_freed_while_referenced`（或至少 `roots_missing_from_liveness_set`）与 `writable_gens_lost`；**`Refuse` / `ReadOnly` 的代价必须进同一个天平，不许再用 `mount_ok` 筛子请出去** |
| 参数 | `H_NEW`、`RECORDS_PER_NEW_TREE` 要么给出仓内出处，要么**扫一条曲线**，不许只取一个点 |
| 断言 | `compat_with_d15_item2` 不许再是手打字面量；它是杀掉一整条臂的那个数，必须由被引条款算出来并配变异 |
| 措辞 | 「六个格子里唯一零静默」当场作废——产物第 6、7 行就是反例。要写成「在挂得上的四个格子里」，而补臂之后连这句也不成立 |

### 我没打中的地方（如实写）

- **`Ignore` 该输，这一点我没能推翻。** 我只推翻了"它输了 4000 条"这个量与它的量纲，
  没推翻"它该出局"。忽略 `RDONLY` / `DEAD` 这类位的后果同样严重，只是模型没有那个量纲。
- **`Carry` 在 compat 那一档上大概率仍然是对的**（D8 已定项 1 的幂等完整值让它成立）。
  我打的是"它赢得太轻松"——对手只有一个明显更差的 `NoCarry`，而更省的那条（豁免点删）没上称。
- **树表条目宽、扇出、18 字节那批数**我一个字没碰，那是另一个攻击面。
