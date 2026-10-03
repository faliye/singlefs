# m3-blanks-r1 攻方腿（Opus）报告（2026-10-01）

<!-- doc-lint:not-numbers B1 B2 B3 B4 B5 B6 B7 B8 K-R3-1 K-R3-2 K-R3-3 K-U5 W1 W2 W3 W4 OWN -->

攻击面：正文第三节 B1–B8 全部；① 过度外推与越位、② 内部矛盾与循环依赖、③ 最脆弱的一格。冻结副本 `/tmp/claude-1000/m3-blanks-r1/tree/` 只读，代码引文写那份副本的绝对路径；kb 引文写 kb 文件自己的行号（现查）。

## 零、复跑与模型

四个纯 Python 模型（不读盘、不编译、不碰崩溃枚举；没有一处用到 `singlefs-checker-tier`），复跑：

```
bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 15 bash research/prompts/m3-blanks-r1-opus-model/rerun.sh <输出目录>
```

本腿两次跑（`/tmp/claude-1000/m3-blanks-r1-opus/run1`、`run2`）末行逐字相同：「model-output 相同」，`model-output.txt` sha256 `f0d829daf41628e2e02b81787c3acca20f12cf939bb4d1af265a78420db65031`；整条挂钟 62.45 s、峰值 838932 KB（`/usr/bin/time` 量的）。

| 文件 | 行数 | sha256 | 答哪一格 |
|---|---|---|---|
| `queue_bound.py` | 114 | `cce4edd70207695e6e38bd2bb51300f7513f0e6bc0d1ce17e633e1f2d751c1fc` | B4 / B8 × D23 已定项 14 |
| `tomb_reserve.py` | 97 | `8907a898e4311e8bb66553e87317a8cd15e6e87664028b6d42a34d1e0157b184` | B7 × B5 |
| `kind_alias.py` | 74 | `6d0ea0ddfa0b73871c5c4c8f3b590c0b844437ba03fb7a50b19b8f0d96c818b3` | B1 |
| `ring_window.py` | 47 | `2db6fe023b8279e33c7aec35e448ac842fc5a0d16f7c709f29b640de482372c7` | B6 |
| `rerun.sh` | 18 | `d47d2151430593e7e1977ba1ba35a4fe68d14ed6bd2e59af02bef57d5459ae68` | — |
| `model-output.txt` | 52 | `f0d829daf41628e2e02b81787c3acca20f12cf939bb4d1af265a78420db65031` | — |

取样的缩法（第 3b 条）：`queue_bound.py` 是穷举用户动作的广度搜索，状态数随 inode 数与深度指数长。先跑一小段估：`3 6 4` 0.07 s、`4 8 4` 0.53 s、`5 9 4` 5.48 s、`4 8 12`（没有 write 动作的那一版）155.34 s 峰值 4694568 KB；加了 write 动作之后 `4 7 12` 40.88 s、峰值 838892 KB。正式跑取 `5 9 4`（根环 4 条，盘紧时的候选集）与 `4 7 12`（根环 12 条，S = 4 的整环），没跑 `4 8 12` 加 write 那一格（按无 write 时 155 s / 4.7 GB 乘分支数推，内存要越 8G 上限）。这里没有崩溃状态，不涉及层 0 枚举。

## 一、各格判定一览

「打中」= 造出了历史、输入或算式，每一步指到条款；「没打中」= 试过的形状写在第八节。四句的回答在第六节。

| 格 | 判 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| B1 | 没打中「错开」的读者；打中候选丙与「永远不撞」的说法 | 「错开」唯一的读者是钉字节的测试（2026-09-13 的来历句），甲照样保住它；甲只多出两处「按树 ID 找种类」的查找错会静默读成别的树；乙「那段永远不撞」不成立；丙改 D8 已定项 11 的用户定案、反而让「种类写成树 ID 的低 2 字节」那一类写错多出一格静默 | 量过（`kind_alias.py`） |
| B2 | 打中候选「0」与一条已定句冲突 | D18 已定项 11 已写「克隆头的类型 1 容器号从 1 起」；空树（零根指针）没有「最大号」，按同一句取 1；回退撞身份那一问在管理员回退下没有对象（墓碑树随现行），崩溃恢复那一形是 C113 同组的老问题，不是墓碑独有 | 推的（读条款） |
| B3 | 打中「显然」的 0 与 D18 已定项 11、D5 已定项 9 两处 | 实现员代填的四个 0 与「类型 1 容器归写者树独占」相冲（墓碑树是每头一棵，头 ID 该写 12）；D5 已定项 9 的定义句「创建这棵树的那次发布所服务的可写头」按字面给全部新树 12，与它自己列的五个 0 互相矛盾，判不了新树；今天没有一处读者读头 ID，checker 判不出任何取值 | 推的（读代码 + 读条款） |
| B4 | **打中，最脆弱**（第五节） | 「unlink 一次跑完 ⇒ 队列 ≤ 1」不成立：回退取 R_old 的 `deleted_inodes`、会话里不续做，「unlink → 回退到意图那一版」每转一圈队列多一个号；只看 B8 定不出上界 | 量过（`queue_bound.py`） |
| B5 | 打中候选「第一版不实现」 | 「不实现、满时拒」改三条已定（D3 已定项 17 射程、D8 已定项 6、D18 已定项 24 第 1 条）与用户定案 K-R3-3；拒成空间错是假性 ENOSPC，拒成非空间错是「因结构拒删除」，两种都违条款，没有合法的拒法 | 推的（读条款与 `mounted_session.rs` 分流） |
| B6 | 读法乙打中，读法甲没打中 | 两种都在准入那一刻算得出；乙在「根还少时写满、再删一个大文件」的历史上罩不住顶点，删除要靠抬 F 把回退深度砍到 4；甲 S = 4 / 8 / 16 是 13 W / 25 W / 49 W | 粗模型量过（`ring_window.py`） |
| B7 | **打中** | 「每活对象 41 字节 + 常驻 3 槽」在墓碑树根长高那一步差约 1 槽 / 盘（m = 110 025、n = 1 时差 16 343 字节），每多长一层多差 1 槽；41 是每份副本的字节、标签 6 按副本之和记，写错单位就少扣一半；「41 × n」与「总量向上取整」从 n = 3 起不等，而 D8 已定项 16 判据 ④ 要逐字节相等 | 量过（`tomb_reserve.py`） |
| B8 | 打中「这一格不改格式、只为 B4 给依据」那句 | B8 的接口臂直接决定队列有没有界与 D3 已定项 9 第 2 条（删掉的空间有界步数内可用）在回退之后守不守得住；六个臂里只有「回退之后续做」与「会话里每次用户改动先续做」两个两样都守 | 量过（`queue_bound.py`） |

## 二、攻击面①：过度外推与越位

### 2.1 B1：「错开」的读者是谁，甲乙丙各动了什么

「错开」是怎么来的：2026-09-13 总审核把它写成「树 ID 与种类码错开：第一版树 ID 从 11 起」，理由一栏「错开树 ID 让「种类写成树 ID」在新池新建文件就红」（`records/2026-09-13-总审核.md:270`：「在新池新建文件就红」）。它防的是写者把两个量写反，读者是钉字节的测试，不是格式本身。条款里留下的三处说法：「树 ID 是另一个量、与种类码错开（取值在已定项 11）」（`.claude/kb/decisions/08-核心索引结构.md:298`）、「与树的种类码 1..8 错开」（`.claude/kb/decisions/08-核心索引结构.md:320`），代码注释两处「树 ID，从 11 起编号、与树的种类码错开」（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/address.rs:39`）、「树 ID 从 11 起、与树的种类码错开」（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-format/src/lib.rs:175`）。

`kind_alias.py` 对新池新建文件那一版的树表量了四种写错（W1 单条种类写成自己树 ID 的低 2 字节、W2 全部这样写、W3 读者按「树 ID == 种类常量」找条目、W4 单条树 ID 写成自己的种类码），读者的拒法照冻结副本：种类没登记拒（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/recovery.rs:2848`：「what: "树的种类没登记",」），同一种类两次拒（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/recovery.rs:2176`：「return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);」）。原样输出：

```
E name=kind_alias cand=old_8_trees entries=7 own_kind_equals_own_id=[] W1=[('refused_unregistered', 7)] W2=refused_unregistered W3_lookup_returns_other_tree=[] W4=[('refused_order', 6), ('silent', 1)]
E name=kind_alias cand=A_append_9_12 entries=11 own_kind_equals_own_id=[] W1=[('refused_kind_twice', 2), ('refused_unregistered', 9)] W2=refused_unregistered W3_lookup_returns_other_tree=['lineage->extent', 'logged_ops->inode'] W4=[('refused_order', 10), ('silent', 1)]
E name=kind_alias cand=B_segment_0x8000 entries=11 own_kind_equals_own_id=[] W1=[('refused_unregistered', 11)] W2=refused_unregistered W3_lookup_returns_other_tree=[] W4=[('refused_order', 9), ('silent', 2)]
E name=kind_alias cand=C_ids_from_65536 entries=11 own_kind_equals_own_id=[] W1=[('refused_kind_twice', 9), ('refused_unregistered', 1), ('silent', 1)] W2=refused_unregistered W3_lookup_returns_other_tree=[] W4=[('refused_order', 10), ('silent', 1)]
E name=kind_segment_reached segment_start=32768 first_file_reissues_until_a_tree_id_reaches_it=2729 or_snapshots=32746
```

读法：
- 2026-09-13 那条理由要的是「每棵树自己的种类码 ≠ 自己的树 ID」，这样 W1 / W2 / W4 在新池新建文件里逐字节都变、钉字节的测试红。三个候选 `own_kind_equals_own_id=[]` 全满足——甲没有丢掉「错开」的读者。
- 甲独有的代价是 W3：按树 ID 找 11、12 会找到 extent 树与 inode 树（`lineage->extent`、`logged_ops->inode`），旧表里这种写错找不到条目、当场报错。core 里树 ID 是 `TreeIdentifier(u64)`、种类是 `u16`，W3 要显式写出 `.0 == u64::from(kind)` 才写得错；checker 按裸字节读两段（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-checker/src/walk.rs:1312`：「let tree = read_u64(entry, 0);」，`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-checker/src/walk.rs:1313`：「let kind = read_u16(entry, 10);」），W3 在 checker 里比在 core 里好写错。
- 乙「另取一段、永远不撞」不成立：树 ID 水位只增，第一个文件版本每重发一次占 12 个号、每个快照占 1 个，0x8000 那一段在重发 2729 次或 32746 个快照之后就被树 ID 追上（上面最后一行）。乙只能写成「在树 ID 到达那一段之前不撞」。
- 丙越位：它改 D8 已定项 11 的用户定案「第一版树 ID 从 11 起编」，连带改新池新建文件里每个单元头的出生树、记账行 (12, 12, …) 的树段、头 ID、全部 `TREE_IDENTIFIER_*` 常量与钉死的镜像 CRC；而且从 65536 起编时 W1 多出一格静默（livelist 的树 ID 65541 低 2 字节是 5，正好是不进树表的中央映射树的码），正是「错开」要防的那一类写错变得看不见。要不出这一格得把起点挑成 65536 + 11（`C_ids_from_65547` 那一行与甲同形），那就是平移，什么都没买到。

改法（推的）：取甲；三处「错开」的说法改写成「新池新建文件那一版每棵树自己的种类码不等于自己的树 ID」（这是 2026-09-13 那条理由真正要的），不写成「只对 1..8 成立」——后一种写法把一条测试灵敏度的要求说成了值域性质，下一次加树的人会以为可以随便撞。

### 2.2 B2：容器号取几、空树怎么读

- 已定的一句管着「第一个号」：「克隆头的类型 1 容器号从 1 起按树内单调分配，身份由出生树分开」（`.claude/kb/decisions/18-块里携带什么信息.md:300`）。取 0 要改这一句（或让 origin 与克隆头两套起点不同），属于越位；取 1 不动任何已定。
- 空树：墓碑树零根注册（树表条目根指针全零），「最大号」读不出来不是 0，是「没有」。写成「零根指针 ⇒ 下一个号 = 1；否则 = 最右叶序容器号 + 1」。谱系树同一条（它也是零根注册、只增不删）。
- 容器号 0 还撞一处读法：码 2 内部条目里类型段 0 的那一支两段恒零（`.claude/kb/decisions/08-核心索引结构.md:164`：「容器号、出生代两段恒 0」；墓碑树同形，`.claude/kb/invariants.md:297`：「0 ⇒ 子单元是码 2（根长高之后」），而 inode 树空根的 key 区间写 [0, 0]。墓碑树根只装一个 0 号容器时根的 key 区间也是 [0, 0]，按区间而不是按条目数认「空根」的读者会认错（D8 已定项 16 第 5 条的第三成员判定点名不看墓碑树，所以今天没有这样的读者；推的）。取 1 两处都躲开。
- 回退撞身份那一问：管理员回退时墓碑树随现行那一版，回退之前写过的墓碑容器全都还挂在树上、一个都不会「不可达」，这一问在管理员回退下没有对象。崩溃恢复落到旧根时墓碑树随所选根，新号与被抛弃时间线上的容器同号、出生代不同（恢复之后新实例的 txg 大于环里读得出的每一条根与记录），撞不上；两条都读不出的双重故障下同号同代，那是 C113 同组的「孤儿与活版共用身份」老问题，不分辨墓碑、不归 B2。

### 2.3 B3：头 ID，「随 R_old 就照 extent 树写 12」是拿错了轴

- 题面给的判别轴是「回退时随 R_old 还是随现行」。这一轴与「归不归某个头」是两根轴：墓碑树回退时随现行（`.claude/kb/decisions/23-journal的角色与格式.md:376`：「的根指针取 R_old 那一版的」那一句的后半写墓碑树、谱系树随现行），却是每头独占的——「类型 1 容器归写者树独占」（`.claude/kb/decisions/18-块里携带什么信息.md:300`），同一句往下写克隆不收 origin 的类型 1 容器、克隆头的容器号自己从 1 起。按回退那根轴推，墓碑树会写 0；按归属那根轴，它属于 inode 树 12 那个头。
- 实现员代填的四个 0 的依据是「D5（快照 / 空间记账机制） 已定项 9 只点名了 inode 树与 extent 树写头」（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/transaction.rs:6572`），钉在 `vec![12, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0]`（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1223`）。D5 已定项 9 自己的定义句写值「从「创建这棵树的那次发布所服务的可写头」来」——这一句字面上给第一个文件版本那次发布建的每一棵树都是 12（那次发布服务 inode 树 12 那个头），可同一句又列「分配记录树 / 记账树 / livelist 共享树 / 稀疏旁表树 / deadlist 树写 0（无归属）」（`.claude/kb/decisions/05-快照-空间记账机制.md:227`），这五棵也是同一次发布建的。定义句与它自己列的例子互相矛盾，拿它判四棵新树判不出来：真正起作用的是「每头一份的结构写头、全池一份的写 0」，而这句没写下来。
- 按「每头一份 / 全池一份」逐棵判（推的）：`deleted_inodes` 存的是一个头的 inode 号、克隆头要自己的队列 ⇒ 12；墓碑树按上面 D18 那句每头独占 ⇒ 12；谱系树记的是池的根时间线（回退是对池根的一次向前发布）⇒ 0；logged_ops 要装的整理意图是全池的（D26），而承诺类意图（将来的 truncate）是每头的——一棵树装两种归属，头 ID 一个字段装不下，留给 C364。
- 谁会读错：今天一个都没有。core 只解析、照抄（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/mount.rs:5168`：「head_identifier: entry.head_identifier,」），checker 不读偏移 116 那 8 字节，`.claude/kb/invariants.md` 里没有一条不变量提到头 ID。四棵新树的头 ID 改成任何值，只有上面那一条钉字节的测试会红。读者在快照 / 克隆落地那天出现（D5 已定项 9「非 inode 树靠树表条目里的「头 ID」找到它归哪个头」那一半）：全写 0 时克隆头与 origin 共用一棵 `deleted_inodes` 与一棵墓碑树，克隆头的挂载续做会去删 origin 的号。

### 2.4 B5：「第一版不实现根长高」改三条已定

- 已定的三处都写了长高：「根满时长高，照 D8（核心索引结构） 已定项 6」（`.claude/kb/decisions/18-块里携带什么信息.md:565`）；D8 已定项 6「inode 树与墓碑树（D18（块里携带什么信息） 已定项 24）共用这一条」（`.claude/kb/decisions/08-核心索引结构.md:155`）；D3 已定项 17 射程「墓碑树根满时长高（D8（核心索引结构） 已定项 6），不因结构拒删除」（`.claude/kb/decisions/03-空间分配.md:308`）；用户定案 K-R3-3「写根长高（推荐）」。正文 B5 问「第一版要不要实现」，「不要」这一支不是一个可选的实现形态，是推翻这三条与一条用户定案。
- 不实现时拒成什么：今天 inode 树的同一种拒是 `MoreLeafContainersThanOneRootNodeHolds`（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/inode_tree.rs:142`），会话把它分到「不是空间不够」那一边（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/mounted_session.rs:164`：「| PublishError::InodeTreeWriteRefused(_)」）。照抄这一分流，unlink 的意图那次发布返回一个非空间错——这就是「因结构拒删除」，D3 已定项 17 射程逐字禁的。改分到空间那一边，会话推抬 F、推满 B = 8 次仍不够、报 ENOSPC，而 `df` 照样报着空闲——D3 已定项 9 第 1 条的假性 ENOSPC。两条出口都违条款，「不实现」没有合法的拒法。删除路径确实被挡：意图那次要往开放墓碑容器追加一条，开放容器满且根满时那一条无处可放；挂载续做不受影响（续做的号在意图那次已经写过墓碑）。
- 「11 万次才满」不是推迟的理由：第一版不回收墓碑、回退不撤墓碑，110 025 是池一生的上限（`research/prompts/m3-delete-r3-main-verification.md:54`：「根装 135 个容器 × 815 条 = 110 025 次 unlink」）；用只供测试的容量旋钮把一个容器压到 1 条，136 次 unlink 就到（C619 的复现条件就是这么写的）。

### 2.5 B6：两种读法，哪种罩得住顶点

- 两种都在准入那一刻算得出、不遍历：甲是系统配置常量 3 S（`ROOT_RING_SLOTS_PER_REGION_MINIMUM` 4 到 `MAXIMUM` 16）；乙是分配器内存里那张「哪些环槽住着根」的表的大小（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/mount.rs:2164`：「pub fn ring_slots_known_to_hold_a_root_by(allocator: &PoolAllocator) -> BTreeSet<RootRingSlot> {」）。「算得出」分不开两者。
- 顶点：删除第 b 批之后还被候选集里的根钉着的删除新写是 W · min(b, 3 S) + W，在第 3 S 批附近到顶（判决「顶点在第 3S 次发布附近」，`research/prompts/m3-delete-r3-main-verification.md:42`）。顶点只看「一段删除能攒下几批被钉住的旧槽」，与「准入那一刻环里有几条根」无关；乙把一个随删除推进而长的量换成了写满那一刻的快照值。
- 乙的反例历史（粗模型，`ring_window.py`）：新池新建文件之后用很少几次大的顺序写把池写满（环里只有 r 条根，r ≪ 3 S），再删一个大文件。W 取 368（辩方第三轮「对 K = 144 在 4 GiB 盘上是 368 槽」，`research/prompts/m3-delete-r3-sonnet-output.md:26`，算的）：

```
E name=ring_window S=8 ring_slots=24 r_at_fill=4 reading=A W=368 window_slots=9200 window_in_W=25 batches=240 short_batches=0 rollback_depth_lost=0
E name=ring_window S=8 ring_slots=24 r_at_fill=4 reading=B W=368 window_slots=1840 window_in_W=5 batches=240 short_batches=118 rollback_depth_lost=20
E name=ring_window S=16 ring_slots=48 r_at_fill=4 reading=B W=368 window_slots=1840 window_in_W=5 batches=480 short_batches=238 rollback_depth_lost=44
```

  乙下删除不会卡死（删除类发布只判落点，落点不够时会话推抬 F），代价是回退候选从整环被砍到最近 4 条——删除这件事悄悄吃掉了 20 / 44 条回退深度，而窗口本来就是为了不吃它才放进式子的。乙还随介质故障变：一个根槽读不出，r 少一，窗口跟着缩、可用凭空多出 W。
- 甲 S = 4 / 8 / 16：13 W / 25 W / 49 W，W = 368 时 4784 / 9200 / 18032 槽 / 盘（同一份输出 `window_slots`）。乙随 r 在 2 W 到 (3 S + 1) W 之间，新池新建文件那一刻 r 很小。

## 三、攻击面②：内部矛盾与循环依赖

### 3.1 B4 ⇄ B8 ⇄ D23 已定项 14 ⇄ D8 已定项 16 第 6 条：队列上界不是 B8 一格定得出的

正文写 B8「判它是为了让 B4 的队列上界有依据」，隐含的推理是「unlink 一次调用把意图与全部批做完 ⇒ 任何一版里队列至多 1 个号」。这条推理漏了回退：

1. 管理员回退的新根「与 `deleted_inodes` 的根指针取 R_old 那一版的」（`.claude/kb/decisions/23-journal的角色与格式.md:376`）；
2. 候选集收任何树表不是 0 条的根（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/mount.rs:4943`：「match tree_table_has_no_entries(reader, &target_root) {」），删除中间那一版在内——第一轮判决已量到「回退进删除中间一版、nlink = 0 的记录带着 6 个单元回来」（`research/prompts/m3-delete-r1-main-verification.md:37`）；
3. 续做只有一个挂点：`mount_writable`「在写行与暖机之后、交回之前把队列走到空」（`.claude/kb/decisions/08-核心索引结构.md:456`）；回退那次发布不续做，会话里也没有别的续做点；
4. 第一版能建任意多个 size 0 的 inode（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/mounted_session.rs:58`：「NewInodes { count: u64, write_time_seconds: u64 },」），unlink 按 inode 号。

于是在一次可写挂载里：`unlink(1)`（意图那一版队列 {1}，之后删完）→ 回退到意图那一版（队列 {1} 回来，没人续做）→ `unlink(2)`（意图那一版 {1, 2}）→ 回退到它 → `unlink(3)` → …… 每转一圈队列多一个号，上界只剩「池里有几个 inode」。

`queue_bound.py` 只固定前缀（N 个活的 size 0 inode、队列空），用户动作（对每个活 inode 的 unlink、回退到候选集里每一条根、write、remount）全部放开、广度优先穷举到给定深度；B8 的六个接口臂见文件头。原样输出（去掉见证串以外照抄；见证串在 `model-output.txt` 第 1–12 行）：

```
E name=queue_bound arm=OWN inodes=5 depth=9 ring=4 candidates=4 states=320595 max_queue=5 max_writes_with_queue_nonempty=7
E name=queue_bound arm=DRAIN_AFTER inodes=5 depth=9 ring=4 candidates=4 states=91225 max_queue=3 max_writes_with_queue_nonempty=7
E name=queue_bound arm=DRAIN_BEFORE inodes=5 depth=9 ring=4 candidates=4 states=70075 max_queue=1 max_writes_with_queue_nonempty=7
E name=queue_bound arm=OWN_RBCONT inodes=5 depth=9 ring=4 candidates=4 states=20235 max_queue=1 max_writes_with_queue_nonempty=0
E name=queue_bound arm=DRAIN_ANY inodes=5 depth=9 ring=4 candidates=4 states=57915 max_queue=1 max_writes_with_queue_nonempty=0
E name=queue_bound arm=ASYNC inodes=5 depth=9 ring=4 candidates=4 states=627323 max_queue=5 max_writes_with_queue_nonempty=8
E name=queue_bound arm=DRAIN_AFTER inodes=4 depth=7 ring=12 candidates=12 states=1044314 max_queue=4 max_writes_with_queue_nonempty=5
```

OWN 的见证（`model-output.txt` 第 1 行）：「witness=[unlink(1) rollback(1) unlink(2) rollback(1) unlink(3) rollback(1) unlink(4) rollback(1) unlink(5)]」——五个 inode 全进了队列。DRAIN_AFTER 在根环 4 条时停在 3、根环 12 条时到 4（= inode 数），差别只是意图那一版被续做批挤出候选集的快慢。

读法：
- 「一次调用跑完」（OWN）与「意图之后把整条队列做完」（DRAIN_AFTER）都没有界——上界随回退次数长，B4 问的「多于一个根就拒」在 2031 圈之后真会被走到，而那一拒是「因结构拒删除」（同 2.4 节）。
- 只有三个臂把队列钉在 1：意图之前先把现行那一版的队列做完（DRAIN_BEFORE），回退那次发布之后接着续做（OWN_RBCONT），会话里任何一次用户改动都先续做（DRAIN_ANY）。
- 第二个量 `max_writes_with_queue_nonempty` 数的是回退进删除中间之后、用户只做写的那几步里队列一直不空：DRAIN_BEFORE 也是 7（= 深度 − 2，随深度长）。被回退带回来的那个号已经是「没有文件」（nlink = 0），它没释放的单元在用户下一次 unlink 或下一次挂载之前一直占着；D3 已定项 9 第 2 条「删掉一个 s 字节的对象之后，同样大小的写必须在有界步数内成功」（`.claude/kb/decisions/03-空间分配.md:187`）在这一段上没有界。六个臂里只有 OWN_RBCONT 与 DRAIN_ANY 两个量都是 0 / 1。
- 所以 B4 的上界要 B8 与 D23 已定项 14、D8 已定项 16 第 6 条一起定：B8 选 OWN / DRAIN_AFTER / ASYNC，就要给 D8 已定项 16 第 6 条加一个会话里的续做点（回退之后，或每次用户改动之前），这是改已定项；B8 选 DRAIN_ANY，本身就是第 6 条的增补。两种都越出「B8 不改格式、只是实现形态」的说法。

### 3.2 B7 ⇄ B5 ⇄ D3 已定项 2：常驻那一份是在 K-R3-3 之前算的

D3 已定项 2 写「另加全池常驻一份（一个开放墓碑容器 + 根）」（`.claude/kb/decisions/03-空间分配.md:52`），数出自第三轮判决「全池一份常驻余量（一个开放墓碑容器 + 根，辩方算 3 槽）」（`research/prompts/m3-delete-r3-main-verification.md:44`）与辩方的势函数「每对象 ⌈2 × (1 + 1/135) × 16 384 ÷ 815⌉ = 41 字节」（`research/prompts/m3-delete-r3-sonnet-output.md:172`）。那份势函数把内部节点按「每 135 个容器多一个」摊，没有算根长高那一步一次要两个新节点（新根 + 分出的右半，D8 已定项 6）。K-R3-3 定了长高之后，这一常驻量与已定的长高互相不配。

预付是现行那一版的函数（标签 6 按活对象数现算，不是余额），所以要对每个可达的 (已写墓碑数 m, 活对象数 n) 都罩得住。`tomb_reserve.py` 原样输出：

```
E name=tomb_reserve rule=G1_41B_plus_3slots cap=815 fanout=135 growth_levels_scanned=1 n_max=1700 min_margin_bytes=-16343 at_m=110025 at_n=1 need_slots=4 height_before=1
E name=tomb_reserve rule=G1_41B_plus_3slots cap=815 fanout=135 growth_levels_scanned=2 n_max=1700 min_margin_bytes=-32727 at_m=14853375 at_n=1 need_slots=5 height_before=2
E name=tomb_reserve_large_n rule=G1_41B_plus_3slots n_samples=1e4,1e5,1e6,1e7,3e7 min_margin_bytes=-32368 at_m=2005203995 at_n=10000
E name=tomb_root_full m=110025 containers=135 perm_slots=271 next_one_need_slots=4 reserve_g1_bytes_n1=49193
E name=tomb_reserve rule=41B_plus_2_plus_height_plus_1 cap=815 fanout=135 growth_levels_scanned=3 n_max=1700 min_margin_bytes=41 at_m=0 at_n=1 need_slots=3 height_before=0
E name=tomb_reserve_large_n rule=41B_plus_2_plus_height_plus_1 n_samples=1e4,1e5,1e6,1e7,3e7 min_margin_bytes=16784 at_m=0 at_n=10000
```

反例镜像：墓碑树根装满 135 个容器、每个 815 条（m = 110 025），池里只剩 1 个活对象。它下一次 unlink 要永久新占 4 槽 / 盘（新容器 2 + 新根 1 + 右半 1），预付给它的是 3 槽 + 41 字节 = 49 193 字节，差 16 343 字节，约 1 槽。树高每多一层差多 1 槽；n 到 1 万也补不回来（第三行）。活对象少于约 400 个时这一格都中。可达性：第一版能建任意多个 size 0 的 inode、按号 unlink；用只供测试的容量旋钮把容器压小，几百步就到（`tomb_reserve_knob` 三行，旋钮下 41 字节的摊法离需要差得更远，因为摊法写死的是产品容量 815）。

它为什么在今天的判据里看不见：删除类发布不判式子，可用被吃到负数也不拒；D8 已定项 16 判据 ④「每个被判状态的记账标签 4、6 等于从这一版镜像现算的值」（`.claude/kb/decisions/08-核心索引结构.md:457`）拿同一个式子现算，式子写短了两边一样短。只有「标签 6 罩着的量 vs 墓碑树将来真要占的槽」那种对拍看得见（C617 的复现条件就是这一类，`.claude/kb/checks-owed.md:486`：「删后回退留下的墓碑没有预付罩着」）。

### 3.3 B7 ⇄ D28 已定项 1 ⇄ D8 已定项 16 判据 ④：单位与取整

- 单位：41 是**每份副本**的字节（上面那条势函数用的是每盘 2 槽一个容器），而已承诺预留「按全部副本之和记字节」（`.claude/kb/decisions/28-挂载期承诺量.md:24`），每块盘扣「和 ÷ 副本数」向上取整（`/tmp/claude-1000/m3-blanks-r1/tree/crates/singlefs-core/src/admission.rs:99`：「BytesOnOneDevice(self.0.div_ceil(replicas.replicas()))」）。照「41 字节 / 对象」字面把 41·n 写进标签 6，每盘只扣一半：

```
E name=tomb_units replicas=2 n=1000000 label6_sum_bytes=82098304 per_device_deduct=41049152 if_label_written_one_copy_per_device_deduct=20524576 shortfall_per_device=20524576
```

  而判据 ④ 拿同一个式子现算，core 与 checker 照同一句话写错时两边一致、一格都不红。
- 取整：「每对象向上取整成 41 再乘 n」与「n 个对象的总量一次向上取整」从 n = 3 起就不等：

```
E name=tomb_rounding per_object_exact=40.5040 ceil_per_object=41 first_n_where_41n_ne_ceil_total=3 at_1e6_41n=41000000 ceil_total=40503959
```

  两种都偏多扣、都安全；但判据 ④ 要逐字节相等，core 与 checker 各按一种写，在第三个活对象那一版就红在合法状态上。B7 要交的「精确式子」必须把取整写在哪一步钉死。
- 标签 6 在各次发布里的值（按 G1 式子，两盘，推的）：L6 = 2 × (41 n + 3 × 16384) = 82 n + 98 304，n = 现行那一版 inode 树里 nlink ≥ 1 的记录数。新池新建文件（n = 1）98 386；建两个空 inode 之后 98 550；unlink 其中一个的意图那一版 98 468，各批与最后一批不变（n 不变）；回退到意图之前那一版，inode 树取 R_old、n 回到 3，98 550。新池新建文件改的字节：记账树叶第 10 行（`(6, 0, 0xFFFF_FFFF, 3)`）的 value 8 字节从 0 改成 98 386（行内偏移 22，`walk.rs` 的记账行读法同此），连带记账树单元的载荷 CRC 与钉死的整份镜像 CRC；第一个文件版本那次写入类发布的需求每盘多 49 193 字节。
- 回退：D23 已定项 14 写「记账标签 4、6 按新根这一版现算（inode 树与 `deleted_inodes` 取 R_old 的，所以被带回来的对象的墓碑份额照抬）」。墓碑树随现行，回退之前那次删除写下的墓碑留着；带回来的对象重新被预付一份，它旧的那条墓碑没人罩——这就是 C617，回退那次不判式子，可用被抬下去不拒。本腿不重复量它。

### 3.4 其余几处互相要靠彼此才成立的

| 两边 | 怎么缠在一起 | 出处 |
|---|---|---|
| B1 甲 ⇄ D8 已定项 9 射程、D8 已定项 11、两处代码注释 | 甲成立就要同时改写四处「错开」的说法（2.1 节），否则条款与常量互相说假话 | 2.1 节 |
| B2 ⇄ D18 已定项 11 第 300 行 | 「第一个号」已经被克隆头那一句定成 1，B2 不是空白，是没被引到 | 2.2 节 |
| B3 ⇄ D5 已定项 9 ⇄ D18 已定项 11 | 定义句与它自己的例子矛盾；判墓碑树的头 ID 要靠 D18 的「写者树独占」 | 2.3 节 |
| B5 ⇄ B7 | B5 选长高，B7 的常驻量就要随树高长（3.2 节）；B5 选不长高，B7 的 3 槽对、而删除在第 110 026 次被挡（2.4 节）。两格必须一起定 | 3.2 节 |
| B4 ⇄ B8 ⇄ D23 已定项 14 ⇄ D8 已定项 16 第 6 条 | 队列上界与「删掉的空间有界步数内可用」要靠一个会话里的续做点，而第 6 条只给了挂载那一个 | 3.1 节 |
| B6 ⇄ D28 已定项 1 的 W | W 的式子只在「删除批只改尾叶」时成立（K-R3-1），甲的窗口 (3 S + 1) W 跟着 W 一起成立或一起不成立；B6 的两种读法都不检这一前提 | `.claude/kb/decisions/28-挂载期承诺量.md:29`：「环窗口 = (根环里的根数 + 1) × W」 |

## 四、改法与它们各修哪一格

全部是本腿提的改法：**只在我的模型上量过、被攻过零轮**。「量过」贴的是 `model-output.txt` 里的原样行；「推的」是按条款或代码推、没实现没跑。

| 改法 | 修的格 | 在打中的那几格上 | 量过 / 推的 |
|---|---|---|---|
| B8 取 DRAIN_ANY：会话里每一次用户改动（含 unlink）先把现行那一版的队列做完，再做它自己；挂载续做照旧 | B4 队列上界、回退之后 D3 已定项 9 第 2 条的步数 | 两格都不中：`arm=DRAIN_ANY … max_queue=1 max_writes_with_queue_nonempty=0`（根环 4 与 12 两种都是） | 量过 |
| B8 取 OWN、D23 已定项 14 加「回退那次发布之后接着续做」 | 同上 | 两格都不中：`arm=OWN_RBCONT … max_queue=1 max_writes_with_queue_nonempty=0`；代价是改 D23 已定项 14 回退「一次普通发布」的字面 | 量过 |
| B8 取 DRAIN_BEFORE：unlink 写意图之前先做完队列 | B4 队列上界 | 队列那一格不中（`max_queue=1`）；**步数那一格照样中**（`max_writes_with_queue_nonempty=7`，随深度长）——对 D3 已定项 9 第 2 条那一格不起作用 | 量过 |
| B8 取 OWN 或 DRAIN_AFTER、不加续做点 | — | 两格都中（`max_queue=5` / `4`） | 量过 |
| B7 常驻量改成 2 + H + 1 槽（H = 现行那一版墓碑树的内部层数，空树 H = 0）：L6 = 副本数 × (41 n + (3 + H) × 16384) | B7 根长高那一步 | 不中：`rule=41B_plus_2_plus_height_plus_1 … min_margin_bytes=41`，大 n 取样 `min_margin_bytes=16784`；新池新建文件那一版 H = 0，字节与 G1 式子相同（98 386） | 量过 |
| 同上的副作用 | — | H 在一次删除类发布里加一，标签 6 跟着涨 2 × 16384，而删除类发布不判式子：可用在那一刻被抬下去不拒（不卡删除，只让下一次写入类准入多扣一槽 / 盘） | 推的 |
| B7 把单位写成「标签 6 = 副本数 × 每份副本的量」、取整写成「每对象先取 41 再乘 n」 | B7 单位格、取整格 | 不中（两格都是写法问题，钉一种就消失）；对根长高那一格不起作用 | 推的 |
| B5 照 K-R3-3 实现长高 | B5 | 「不实现」那一支的两条出口都违条款（2.4 节），长高之后不拒 | 推的 |
| B1 取甲，四处「错开」改写成「每棵树自己的种类码 ≠ 自己的树 ID」 | B1 | 钉字节的灵敏度保住（`cand=A_append_9_12 … own_kind_equals_own_id=[]`）；W3 两处静默查找错（`lineage->extent`、`logged_ops->inode`）这一改法不修，要修只能取乙 | 量过 |
| B2 第一个号取 1，零根指针 ⇒ 下一个号 1；谱系同 | B2 | 与 D18 已定项 11 第 300 行一致，不撞 [0, 0] 读法 | 推的 |
| B3 `deleted_inodes` 12、墓碑树 12、谱系树 0、logged_ops 留 C364；D5 已定项 9 的定义句改成「每头一份的结构写它的头，全池一份的写 0」 | B3 | 今天没有读者，任何取值都不红；这一改法让条款自己判得出四棵树 | 推的 |
| B6 取读法甲 | B6 | 不中：甲在三个 S 上 `short_batches=0 rollback_depth_lost=0` | 粗模型量过 |

## 五、攻击面③：最脆弱的一格

按「最可能让实现写错、或让 checker 判不出」排：

1. **B4（连同 B8）**。显然的推理「unlink 一次跑完 ⇒ 队列 ≤ 1」是错的，九步历史（3.1 节见证）就打穿；实现员照 OWN 或 DRAIN_AFTER 写，崩溃流、checker 全绿：历史上每一个状态里，队列里的号都是 nlink = 0、仍在 inode 树里、单元都在账上的记录，I-9.5、I-4.7 这类单镜像判据一条都不红（推的：照它们的字面逐条对过，没跑 checker）。坏处只在跨状态看得到——同一次挂载里队列随回退次数长，被回退带回来的号的空间在下一次 unlink 或挂载之前一直不回来。判别它的东西只在历史里，不在任何一张镜像里。
2. **B7**。式子里三处各自能写错（常驻量缺长高那一步、单位按份还是按和、取整在哪一步），而判据 ④ 是「两边拿同一个式子各算一遍」，同一句话读错时两边一起错；只有「预付 vs 墓碑树将来真占的槽」那种对拍看得见，今天没有（C617 那一行要的检查是同一类）。根长高那一格要 11 万次 unlink 才在产品容量下走到，日常测试走不到。
3. **B3**。完全判不出：四棵新树的头 ID 写成什么，core 与 checker 都不读（2.3 节），只有一条钉字节的测试会因为改值而红；它定的是快照 / 克隆落地那天才有读者的语义。

## 六、打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的是哪一句的字面 | 跑前条款给的改法还中不中 |
|---|---|---|---|---|
| B4 队列无界（3.1） | 分辨：六个臂里 OWN、DRAIN_AFTER、ASYNC 中，DRAIN_BEFORE、OWN_RBCONT、DRAIN_ANY 不中 | 看得到：会话手里有现行那一版的 `deleted_inodes`（第二轮 V3 已定「会话里的队列每次从现行那一版的 `deleted_inodes` 读，不缓存」，`research/prompts/m3-delete-r2-main-verification.md:35`：「读，不缓存」）；单镜像 checker 看不到 | 正文 B4「第一版队列的长度上界是多少」与「多于一个根就拒」两句 | 正文 B4 给的「多于一个根就拒」在打中的历史上会被走到，且那一拒违 D3 已定项 17 射程；不起修的作用 |
| 回退之后空间不回来（3.1） | 分辨：只有 OWN_RBCONT、DRAIN_ANY 不中，DRAIN_BEFORE 也中 | 同上 | D3 已定项 9 第 2 条「删掉一个 s 字节的对象之后，同样大小的写必须在有界步数内成功」（`.claude/kb/decisions/03-空间分配.md:187`） | 正文没给改法 |
| B7 根长高一步预付不够（3.2） | 不分辨 B7 的取整、单位两种写法（三种写法都短）；分辨常驻量的两种定义 | 看得到：发布时墓碑树的层数在手（根节点头里的层级） | D3 已定项 2「另加全池常驻一份（一个开放墓碑容器 + 根）」 | 正文 B7 没给改法；第三轮判决给的 41 字节 + 3 槽就是被打中的那一式 |
| B7 单位、取整（3.3） | 分辨写法，不分辨常驻量 | 看得到（式子的输入全在镜像里） | 正文 B7「单位是字节还是槽、按副本之和还是每盘、向上取整在哪一步」 | — |
| B5「不实现」（2.4） | 分辨：只打「不实现」那一支 | 看得到（根满在写意图之前就知道） | D3 已定项 17 射程「墓碑树根满时长高（D8（核心索引结构） 已定项 6），不因结构拒删除」（`.claude/kb/decisions/03-空间分配.md:308`）；D3 已定项 9 第 1 条 | 「拒」那一改法本身就是被打中的 |
| B3 定义句自相矛盾（2.3） | 不分辨四棵树的任何取值（今天没有读者） | **看不到**：今天没有一个读者读头 ID，这是「判别子观测不到」那一形，要等快照 / 克隆的读者 | D5 已定项 9「创建这棵树的那次发布所服务的可写头」（`.claude/kb/decisions/05-快照-空间记账机制.md:227`） | — |
| B1 丙、乙（2.1） | 分辨候选 | 看得到（新池新建文件的字节） | 正文 B1「为什么那段永远不撞」（乙）；D8 已定项 11 的用户定案（丙） | — |
| B2 取 0（2.2） | 分辨取值 | — | D18 已定项 11「克隆头的类型 1 容器号从 1 起按树内单调分配，身份由出生树分开」（`.claude/kb/decisions/18-块里携带什么信息.md:300`） | — |
| B6 读法乙（2.5） | 分辨读法 | 看得到 | D28 已定项 1「环窗口 = (根环里的根数 + 1) × W」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）与判决 G1-窗口系数的顶点 | — |

## 七、八格以外冒出来的格式空白

1. **谱系树根满**：谱系树也只增不删，根装 135 个容器 × 2039 条 = 275 265 次回退之后要长高；D18 已定项 24 第 5 条没写谱系树长高，I-9.17 写「类型段只许 5」（`.claude/kb/invariants.md:291`），没有类型段 0 那一支——长高之后的合法镜像被它判坏，不长高则回退在那一刻因结构被拒。与 B5 同形，正文没列。
2. **I-9.21 / I-9.22「墓碑树里容器号沿条目严格递增」**（`.claude/kb/invariants.md:295`）：根长高之后根的条目是类型段 0、容器号恒 0（I-9.23），按「沿根的条目」读当场为假，要写成「沿叶序」。
3. **类型段的取值按树分**：D8 已定项 6 写内部条目类型段「2 ⇒ 子节点是码 3 容器（读两个 16 KiB 槽）；其余值判条目损坏」（`.claude/kb/decisions/08-核心索引结构.md:164`），墓碑树「条目照 inode 树内部节点的 120 字节形态」而孩子是类型 1，谱系树是类型 5。照 D8 已定项 6 共用一份解码时，「其余值判条目损坏」要改成按树的种类查合法集合（I-9.2 {0, 2}、I-9.23 {0, 1}、I-9.17 {5}）；条款只在不变量里分开写了，D8 已定项 6 的正文没改。
4. **会话里的续做点**（3.1 节）：D8 已定项 16 第 6 条只有挂载一个续做点，回退进删除中间之后没有条款让队列在会话里走到空。它是 B4 / B8 的前提，单列在这里是因为它改的是 D8 已定项 16，不是 B8 的接口形态。
5. **墓碑预付的常驻量随树高**（3.2 节）：改的是 D3 已定项 2 的「另加全池常驻一份（一个开放墓碑容器 + 根）」那半句。

## 八、没打中的形状

| 格 | 试过的形状 | 取样范围 | 结果 |
|---|---|---|---|
| B1 | 找一个靠「种类码与树 ID 值域不相交」做判断的读者：`.claude/kb` 与 `.claude/rules` 里 grep「错开」，只有 D8 已定项 9、11 与 `layout/01-first-txn.md` 三处复述；冻结副本里 core 按种类找条目（`recovery.rs` 的 `tree_of`、`mount.rs:5162` 回退挑树），checker 按种类分支，没有一处拿树 ID 比种类常量 | 全仓 grep、冻结副本 grep | 没找到运行时读者；W1 / W2 / W4 在三个候选上全都改字节 |
| B2 | 崩溃恢复后同号同代：要求被抛弃根与它的记录都读不出（双重故障），与 inode 叶容器、数据单元同一暴露；管理员回退下找「不可达的旧墓碑容器」 | 推的 | 不分辨墓碑；管理员回退下没有对象 |
| B4 / B8 | 想打穿 DRAIN_ANY 与 OWN_RBCONT：放开 unlink、回退到候选集里每一条根、write、remount | 5 个 inode 深度 9 根环 4（57 915 / 20 235 个状态）；4 个 inode 深度 7 根环 12（944 932 / 749 848 个状态） | `max_queue=1`、`max_writes_with_queue_nonempty=0`，没打中 |
| B6 | 想打穿读法甲：根少时写满、盘紧抬 F 后再删、S 取 4 / 8 / 16 | 粗模型，每个 S 三种 r，删除 30 S 批 | 甲 `short_batches=0` 全部 |
| B7 | 想打穿「2 + H + 1」：每层长高的临界 m（F^k − 1、F^k、F^k + 1 个容器，各 ± 1 条、± 一个容器）× n = 1..1700，另取 n = 1e4、1e5、1e6、1e7、3e7 | 三层长高以内 | 最小余量 41 字节（m = 0、n = 1），没有负数 |
| B5 | 长高那次发布本身的崩溃安全 | 没做（C619 已记，要崩溃流） | — |

## 九、这条腿自己的限度

- 四个模型都是条款的抽象，不是 `crates/` 的原型：
  - `queue_bound.py` 只有 size 0 的 inode（一次 unlink = 意图 + 一次最后一批），不建新 inode、不算空间、不在一次 unlink 中间崩（remount 只从最新那一版续做）；候选集只取根环最近 4 或 12 条，不判实例表与 F 的细节；同一个根环内容去重；深度只到 9（第零节写了为什么）。「队列随回退次数长」是归纳出来的，模型只给到 inode 数那么长。
  - `tomb_reserve.py` 假定末尾分裂、右侧路径全满时才长高，只数永久占用，暂态 COW 归窗口、不在这里；没量窗口能不能把差的那一槽兜住。
  - `ring_window.py` 是粗模型：候选集一开始就按整环算、不随 r 起步，每批固定 W，不模拟分配器的落点与聚簇段。它只用来说明乙把「随删除推进长的量」换成了「写满那一刻的快照」，数不进 kb。
  - `kind_alias.py` 只看新池新建文件那一版的树表，读者的三种拒法是照冻结副本重写的，没调 core。
- 抽样一次：这一轮「没打中」的那几格（第八节）按 `.claude/rules/three-way-inference.md` 要再抽一次才算。
- B3 与 B2 的结论都是读条款推的，没有任何读者可以跑。

## 十、没做什么

- 没编译 Rust、没跑任何 cargo 测试、没跑 checker 档与门禁；没有崩溃状态（层 0 枚举 0 次，本腿占用的原型状态预算 0）。
- 没读禁读清单里的文件（`research/prompts/m3-blanks-r1-sonnet-output.md`、`research/prompts/m3-blanks-r1-sonnet-model/`、正推腿提示）。
- 没判正推、辩方的格，没替主 agent 采纳；本腿的数全在自己的模型上量，不算入库装置上的数。
- 进程：第一次试 `queue_bound.py 4 8 4`（旧版本、根环 12）超过前台两分钟被转后台，我按进程号停掉了它（`proc.py stop 2454039`，停后 `ps` 查无残留）；那一次的输出不算结果。开跑前 `ps` 看到别的会话的 `cargo test` 与 `e168_delete_ring_window_borrow`（`run-with-memory-cap.sh 24G`）在跑，没有 qemu / fio 类性能测量；Python 模型不等 cargo 锁。
- 草稿目录 `/tmp/claude-1000/m3-blanks-r1-opus/` 只有 `run1/`、`run2/` 两份 `model-output.txt`，没有编译目录与仓副本。
