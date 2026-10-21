# m4-trunc-r1 攻方腿（Opus）报告

日期 2026-10-16。立场：假设各格的候选都错，造历史去打穿。代码行号按工作树 `~/.local/share/singlefs/work/m4-predev`；kb 行号是 kb 文件自己的。没读禁读清单里的两样。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-trunc-r1-opus-model/`。复跑（整条经内存包装，工作目录随意，`--work` 给一个新目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-trunc-r1-opus-model/tools/rerun.sh \
  --repo ~/.local/share/singlefs/work/m4-predev --frozen ~/.local/share/singlefs/work/m4-predev --work <新的草稿目录>
```

它把冻结树的 `crates/`、`Cargo.*`、`.cargo`、`.claude/heavy-thread-default` 拷进草稿目录，放进原型测试、打两份副本补丁，跑 harness 一个测试目标（三种读法各一次）与 core 一个 lib 单测，各自的 `M4TRUNC` 行与 `out/` 里同名文件逐行比。我在草稿目录里复跑一遍的原样末四行：

```
name=rerun_arm arm=extent-probe lines=5 same_as_model_out=yes
name=rerun_arm arm=z-not-applicable lines=13 same_as_model_out=yes
name=rerun_arm arm=z-unclaimed lines=14 same_as_model_out=yes
name=rerun_arm arm=z-write-order lines=13 same_as_model_out=yes
```

```
81efdce88ef8601b602687c5781d00216fa60e963d6332d1e01b8fff85d08207  ./out/extent-probe.txt
bec5d5830d47ee2a2de44d0ce5248d2f0112c798e616543052a8bc5bcf6da2d7  ./out/z-not-applicable.txt
9840437f0ffed7a1e2847c4f1d4b00300f2a7a58429ede48767d03c062e25d8a  ./out/z-unclaimed.txt
aa4c6b8b4fd49d1866a85226df64f2327e72fe56474e3e53ca30782b4524cc9d  ./out/z-write-order.txt
82662f7a29c69c02aefe8ef4fe7fe1c0baca90c7cc46f299f0567b9cafddda49  ./prototype/m4_trunc_r1_opus_prototype.rs
ddbe1c46ec3929a8d6a7534bc782ab0f2d7bb9ef11f5a3cfd7fad0b9a83f52a8  ./prototype/rebuild.patch
2787336381a5f8fb34790a9bc1b4f0b52600c6a82efa9ec5dc7a74194ced4c80  ./prototype/unlink.patch
0b1f7c768f5b916005cbe9706db480b842d3401233c961011d76ed0ec42a47c4  ./tools/rerun.sh
```

原型是什么：截断与变长今天零代码，原型不实现它们，而是在两块内存稀疏盘（`common_admission::start_plain`，4 GiB 档）上用真写路径造出前缀，再按字节改最新根那一版 inode 记录的 size 与 blocks（`common_deletion_in_progress::change_the_inode_record`，重封叶、补校验和到根槽），造出「只改 size 的变长」与「意图那次就写 size = N」两种候选落地后的那一版，交池级 checker（`check_pool_image`）判。`rebuild.patch` 在副本的扫描重建里加两个环境变量开关：`M4_TRUNC_R1_Z_UNCLAIMED`（读法 Z 落成「size 之外不认领」）、`M4_TRUNC_R1_Z_WRITE_ORDER`（我自提的「读法 Z 加写序」，被攻过零轮）。`unlink.patch` 加一个纯函数探针，看删除批的 extent 计划在叶边界上照抄哪片叶。副本上的数不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 打中的候选与历史（一句） | 量过 / 推的 |
|---|---|---|---|
| W1 | 打中 | 甲（logged_ops 新种类码）：管理员回退时 logged_ops 随现行那一版、inode 树与 extent 树取 R_old，回退到截断做了一半的一版，意图丢、目标没达成；反方向现行那一版挂着的截断意图被带到 R_old 的内容上。乙（记录本身是载体）回退站得住，但它逼出 W2 甲、并让 W7 的「有账」无从判红 | 推的（回退那一段读码，`mount.rs` 现行实现） |
| W2 | 打中 | 甲（意图那次 size = N）：截断中间态上读法 Z 落成「不认领」时 I-1.13 判违例（今天报不适用）；乙（每批 size = 留下单元数 × 净荷）：只读挂载读到的 size 既不是截前也不是截后。W2 与 W3 交叉配错的两格（甲配 W3 乙、乙配 W3 甲）每个中间态读者都判声明长度不符 | W2 甲 × Z 量过；其余推的 |
| W3 | 打中 | 复用 `roles_replaced_via_mapping` 的「含 Data(0) 就是整份重写」判别：截断点在单元 0 与在单元 k ≥ 1 两边，一边重复释放、一边漏放；复用删除批 extent 计划：留下单元数恰是 144 的倍数时罩着被重写单元 k 的那片叶被照抄、指针指向已换下的旧单元；丙（不重写）配 W5 甲：截短再变长读出被截掉的旧字节 | extent 计划照抄那一半量过；其余推的 |
| W4 | 部分打中 | 理想模型把一版里全部数据单元记成同一个出生 txg：截断只重写单元 k 之后再建快照、再覆盖写，模型对单元 k 与其余单元判同一个生死，必有一边判错；「截断意图跨过建快照」「删快照与截断意图同在队列」在会话走空纪律下走不到 | 推的 |
| W5 | 打中 | 甲（只改 size，留洞）：三单元文件覆盖写成一个单元（后两个单元已放、盘上还在、同一对象出生代）→ 变长到三个单元：扫描重建把两个旧单元认领回来、I-1.13 判违例；乙（补零重写旧尾单元再留洞）同样中（洞里那几格照样有旧单元）。另：今天三条写入口下洞只可能是后缀，`data_pointers: Vec` 不必改成映射 | 甲量过；乙与后缀论断推的 |
| W6 | 打中 | 读法 Z 原文「按 size 判死活」在两处同时错：W2 甲的截断中间态（size 之外的单元是活的）与变长留洞（size 之内的旧单元是死的）。我提的「Z 加写序」修住后一处、修不住前一处 | 量过（副本） |
| W7 | 打中 | W1 乙下「size 之外还有单元 ⇒ 有截断意图」恒真、必红世界造不出；W2 乙 × W3 乙下「最后一批没和删意图同事务」留下的一版是一个尺寸为留下单元数 × 净荷的合法文件，池级 checker 看不出；W3 丙 × W5 甲两条一起把尾单元声明长度对 size 的核对放宽到什么都不判 | 推的 |
| W8 | 打中 | `units_replaced_by_the_publish` 把换下的数据单元一律记成 `role_written_at[Data]` 那一个 txg（同 W4）；W2 乙下 RefFS 的允许集合要收「任一批之后的前缀」，W2 甲下只要截前、截后两样 | 推的 |
| W9 | 打中 | 接在 3 单元流后面的截断流够不着 144 单元的叶边界（W3 那一格只在 ≥ 145 单元的文件上出现）；「强制尾单元重写在意图那次还是最后一批」的开关若不连着 W2 一起切，强制出来的那一臂每个中间态读者都判不符；回退不在计划里的任何一条流上，W1 那一格崩溃流抓不到 | 推的 |

## W1　意图载体与目标态

### 打中 1：甲（logged_ops 新种类码）过不了管理员回退——两个方向

回退那次发布挑树按种类：只有 extent、inode、`deleted_inodes` 三种取 R_old 的，logged_ops 随现行那一版。

- `crates/singlefs-core/src/mount.rs:8163`：「|| entry.kind == TREE_KIND_DELETED_INODES」
- `crates/singlefs-core/src/mount.rs:8205`：「deleted_inodes: target_version.deleted_inodes.clone(),」
- `crates/singlefs-core/src/mount.rs:8207`：「logged_operations: current.logged_operations.clone(),」
- `crates/singlefs-core/src/transaction.rs:2971`：「随现行那一版：管理员回退不取 R_old 的」

这条注释引的是 D23 已定项 14，但 D23 那一行只写了墓碑树、谱系树（`grep -c '墓碑树、谱系树、logged_ops 随现行那一版' .claude/kb/decisions/23-journal的角色与格式.md` 输出 `0`，同一份文件里 `logged_ops` 零命中）：

- `.claude/kb/decisions/23-journal的角色与格式.md:378`：「新根的用户可见树（inode 树、extent 树）与 deleted_inodes 的根指针取 R_old 那一版的；墓碑树、谱系树随现行那一版」
- D8 已定项 5 把这一问留给第一个写者：`.claude/kb/decisions/08-核心索引结构.md:108`：「字段表、回退时取哪一版、态别留到第一个写者」
- 上一轮判决已经点过截断这一格：`research/prompts/m3-intent-r1-main-verification.md:39`：「且将来 truncate 等承诺类意图也住 logged_ops、只能随 R_old」
- C364 那一行把意图记录写成随根回退：`.claude/kb/checks-owed.md:268`：「而同一项要求这几样住在一条随根回退的盘上记录里」

**历史 H1（R_old 那一版有没做完的截断，现行那一版没有）**，批大小只供测试的开关取 1：

1. 新池新建文件，会话里顺序写三个单元（size = 2.5 个净荷，`UserChange::SequentialWrite`）。
2. 截断到 N = 0.5 个净荷，走甲：意图那次往 logged_ops 签一条截断条目（inode 1、对象出生代、N），一批放单元 2，最后一批放单元 1、删条目。三版记作 v意图、v一批、v最后。走空纪律让这一串一口气做完（会话里每次用户改动先走空，`.claude/kb/decisions/08-核心索引结构.md:486`：「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 deleted_inodes 队列走空」）。
3. 同一次可写挂载里管理员回退到 v一批（候选集只要求根环里有效、txg ≥ F_生效、带文件，`mount.rs` 的 `rollback_candidate`；不带快照）。
4. 新的一版：inode 树与 extent 树是 v一批 的，logged_ops 是 v最后 的（空）。

结局按 W2 的两个候选分：

- W2 甲（意图那次 size 就是 N）：新的一版 size = 0.5 净荷、extent 树里还有单元 0、1，没有意图。挂载态读者打开文件时条数对不上就拒（`crates/singlefs-core/src/mounted_read.rs:815`：「if extent_records != data_units_implied_by_the_file_size {」）；单元 1 再没有谁去放，永久占槽。
- W2 乙（每批 size = 留下单元数 × 净荷）：新的一版是一个 size = 2 净荷、两个满单元的合法文件，没有意图。截断做了一半永远停在那里。

两种结局都是「意图没了、目标没达成」，正是 D8 已定项 5 要排除的那一格：`.claude/kb/decisions/08-核心索引结构.md:120`：「删除意图必须与最后一批工作在同一个事务里提交」。unlink 的同一段历史不中：`deleted_inodes` 取 R_old 的，号跟着回来，下一次用户改动或挂载续做把它走完。

**历史 H2（现行那一版挂着截断，R_old 是另一份内容）**：

1. 顺序写 A（三个单元）；再顺序写 B（三个单元，对象出生代不变，`crates/singlefs-core/src/transaction.rs:7196`：「inode_object_birth: first_file_record.object_birth,」）。
2. 截断 B 到 0.5 净荷：意图那次发成之后崩。
3. 重开可写挂载，续做的删除类发布被空间拒、推满仍放不下：`.claude/kb/decisions/08-核心索引结构.md:486`：「推满仍放不下时挂载照样交回可写、队列留着」（截断是承诺类，照 unlink 同一条续做纪律）。
4. 管理员回退到 A 那一版。logged_ops 随现行，截断条目（inode 1、同一个对象出生代、N）原样带过来，对象出生代对得上。
5. 下一次用户改动先走空：A 被截到 0.5 净荷。用户回退到 A，拿到的是一份没人要求过的截短的 A。

这与 `m3-intent-r1` 判决第 2 行攻方量到的「写 → unlink → 回退到 unlink 之前 → 续做一批」同形，只是把 unlink 换成截断。

**四句**：① 分辨臂：分。H1、H2 只打甲（以及任何「意图住在随现行那一版的结构里」的载体），乙与「意图住在随 R_old 回退的结构里」的丙不中。② 系统当时看得到判别它的东西吗：回退那次发布手里同时有 R_old 与现行那一版，看得到；是挑树规则把它扔了，不是观测不到。③ 满足的分句：D8 已定项 5「删除意图必须与最后一批工作在同一个事务里提交」那一句的推论「不可能出现意图没了但目标状态没达成」（H1）；H2 是续做做错（同 `m3-intent-r1` 判决第 2 行的判据「某段合法历史上续做判不出或做错 ⇒ 不收」）。④ 改法在这几格上：见下。

**改法（只在我的模型上量过——这一格其实没量、是推的；被攻过零轮）**：

| 改法 | H1 | H2 | 代价 |
|---|---|---|---|
| 甲一：logged_ops 按种类分来源，截断条目取 R_old、整理条目随现行 | 修（推的） | 修（推的） | 回退那次发布要重建一个混合来源的 logged_ops 根：回退多一种新写法，要进崩溃点重放；`mount.rs` 回退构造与 `is_a_unit_taken_from_the_rollback_target` 都要按条目分 |
| 甲二：截断意图另立一棵随 R_old 回退的树（形同 `deleted_inodes`，key = inode、value = 对象出生代 + N），加进回退取 R_old 的种类表 | 修（推的） | 修（推的） | 新树种类码、零根注册、checker 走读；回退取 R_old 的种类表多一行 |
| 乙：意图载体就是记录本身 | 修（推的：意图跟 inode 树一起取 R_old） | 修（推的） | 见打中 2 与 W2、W6、W7 |

### 打中 2：乙（记录本身是载体）站得住回退，但它逼出 W2 甲、让「有账」失去必红世界

乙的定义里「意图那次就把记录的 size 写成 N」就是 W2 甲；W2 甲在 W6 上量出来的结局见 W6。W7 那一格：乙下「size 之外还有单元」本身就是意图，所以「size 之外还有单元的 inode 要么有截断意图、要么一个不剩」恒真，一个漏放尾部单元的写路径缺陷会被下一次挂载当成截断续做悄悄放掉、checker 一次都不红（推的，判别子观测不到那一形：checker 看得到的只有 size 与单元数，两种情形逐条相同）。乙另有一样代价：没有队列，续做入口要按 inode 逐个比单元数与 size，每次用户改动先走空就是每次扫一遍全部 inode（第一版只有 inode 1 写数据，今天便宜；推的）。

### 没打中（W1 其余几问）

- 一个 inode 同时两条截断意图、截到更短的第二次落在第一次没做完时、对删除中的 inode 截断：会话里每次用户改动先走空（`mounted_session.rs` 的 `UserChange` 文档，`crates/singlefs-core/src/mounted_session.rs:113`：「先把现行那一版的 deleted_inodes 走空」），走空被空间拒时这次用户改动本身也拒，所以第二次截断开始时第一次已经做完；删除中的 inode 在走空之后已不在树里。我找到的唯一绕过是上面的回退（H2 的反面：现行那一版挂着 N₂、R_old 挂着 N₁，甲一取 R_old 的 N₁、丢 N₂，回退语义下合理）。试过的形状：会话内交错 7 种用户改动（覆盖写、顺序写、建 inode、unlink、建快照、删快照、截断）两两接在一条没做完的截断后面，加崩溃后挂载续做被空间拒；都先被走空挡住（推的，没跑）。
- 一次发布能做完的截断（N 落在最后一个单元里、不放任何单元）要不要意图：单次发布不存在批间中间态，崩溃只有前后两态；不写意图不产生打得中的历史。它与 D8 已定项 16「只有这一条路」（`.claude/kb/decisions/08-核心索引结构.md:482`：「只有这一条路，不设一次做完的快路」）的字面冲突是条款问题，不是反例；切换判据「⌈N ÷ 净荷⌉ = 现有单元数」在操作期间稳定。但见 W3：这条快路一旦只重写单元 k，同样要过 `roles_replaced_via_mapping` 那一格。

## W2　分批与最后一批同事务

### 打中 1：W2 与 W3 只有两种配法读得通，交叉配的两格每个中间态都判声明长度不符

两条读者都要求每个单元的声明长度恰等于 min(size − 起点, 净荷)：

- `crates/singlefs-core/src/recovery.rs:5440`：「if u64::from(header.declared_length) != expected_payload_length {」
- `crates/singlefs-core/src/mounted_read.rs:1090`：「if u64::from(header.declared_length) != declared_length_expected {」

截断点 N 落在单元 k 中间（k × 净荷 < N < (k + 1) × 净荷）时：

| W2 \ W3 | 甲（意图那次重写单元 k） | 乙（最后一批重写单元 k） |
|---|---|---|
| 甲（意图那次 size = N） | 每个中间态：size = N、单元 k 声明长度 = N − k × 净荷，相符 | 每个中间态：size = N、单元 k 还是满的，不符 |
| 乙（每批 size = 留下单元数 × 净荷） | 每个中间态：size ≥ (k + 1) × 净荷、单元 k 已短，不符 | 每个中间态：留下的全是满单元，相符 |

历史：顺序写三个单元 → 截断到 0.5 净荷（k = 0）→ 批大小 1、在第一批之后崩 → 只读挂载打开文件。交叉两格在这一步就被读者拒，违反 `.claude/kb/decisions/08-核心索引结构.md:124`：「中间状态必须每一个都是合法的文件系统状态」。（推的：按两条读者的判式与三格的字节推，没造镜像。）所以 W2、W3 实际是两条联动的路：（甲, 甲）与（乙, 乙）。下面两条打中各打一条。

### 打中 2：（甲, 甲）的中间态与读法 Z 不相容——量过

W2 甲的中间态里 size 之外的单元是活的（extent 树还引用它们、等后面的批去放）；读法 Z 的原文是「按 size 判死活」：`.claude/kb/decisions/18-块里携带什么信息.md:235`：「文件变短不写新记录，扫描重建时数据单元按重建出的 inode 记录的对象出生代与 size 判死活」。

原型 `truncation_intermediate_with_the_target_size_written_at_the_intent`：顺序写三个单元，把最新那一版的 size 改成恰好一个净荷（截断点在单元边界，不必重写尾单元，正是（甲, 甲）意图那次之后、第一批之前的样子），交池级 checker。三种读法的原样输出（`out/` 里那三份）：

- 今天的读法（size 之外报不适用）：`research/prompts/m4-trunc-r1-opus-model/out/z-not-applicable.txt:12`：「M4TRUNC case=truncation-intermediate-size-written-at-intent I-1.13 = NotApplicable」
- Z 落成「不认领」：`research/prompts/m4-trunc-r1-opus-model/out/z-unclaimed.txt:13`：「M4TRUNC case=truncation-intermediate-size-written-at-intent I-1.13 = Violated」
- Z 加写序（我自提的，见 W6）：`research/prompts/m4-trunc-r1-opus-model/out/z-write-order.txt:12`：「M4TRUNC case=truncation-intermediate-size-written-at-intent I-1.13 = Violated」

违例那一行的差异原样（`z-unclaimed.txt` 第 13 行的括号里）：

```
扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 2 处，第一处 key （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）：扫描重建没有这一条、遍历方向 落点 {盘 0 槽 50184、盘 1 槽 50184}；「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 1、偏移 32634）：扫描重建 没有这一条、遍历方向 （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）
```

同一前缀不改 size 时成立：`research/prompts/m4-trunc-r1-opus-model/out/z-unclaimed.txt:10`：「M4TRUNC case=three-units-before-truncation I-1.13 = Holds」。所以（甲, 甲）落地之后，截断中间态在 I-1.13 上只有两个出路：继续报不适用（C827 那一格永不还清，而且 W6 题面问的「截断落地后改成不认领」不成立），或者扫描重建去读截断意图（意图住 logged_ops 码 2 树、扫描方向只读码 1 / 码 3 单元头，要越过 D13 已定项 12「两个方向共享的只有格式解析」那条界，I-1.13 那一行的原文是「扫描方向与遍历方向共享的只有格式解析（D13（验证路线） 已定项 12）」）。

四句：① 分辨臂：分，（乙, 乙）每个中间态留下的单元都在 size 之内，不中（推的）。② 看得到吗：扫描重建看得到的只有单元头与 inode 记录，「size 之外的单元是等着放的活单元」与「size 之外的单元是覆盖写变短留下的死单元」两种情形在它那里逐条相同（同一对象出生代、同一偏移、都没有更新的版本）——这是判别子观测不到那一形：Z 没写错，是（甲, 甲）造出了 Z 分不开的两种情形。③ 分句：I-1.13「三张派生表逐项相同」。④ 改法：在（甲, 甲）这一格上「Z 落成不认领」与「Z 加写序」都还中（上面两行 Violated），只有「报不适用」不红、但也不判。

### 打中 3：（乙, 乙）的中间态在用户眼里是第三种尺寸（推的）

W2 乙每批之后 size = 留下单元数 × 净荷。从意图提交起，崩溃镜像上的只读挂载、冷走读、快照之外的任何读者，读到的 size 既不是截前的 S、也不是截后的 N，而是 (k + 1) × 净荷 到 S 之间按批步进的一串值；回退候选集也收这些版本。unlink 走同一形状不出这个问题，因为从意图提交起读者就答「没有文件」（D8 已定项 16 第 4 条）。要么 RefFS 的允许集合收「任一批之后的前缀」（判据放宽，W8），要么读者按意图里的 N 答 size（读者去读 logged_ops，W8 的代价）。历史：顺序写四个单元 → 截断到 0.5 净荷、批大小 1 → 第一批之后崩 → 只读挂载读 size：答 3 净荷。

### 没打中

- 一批写入量不随文件大小增长：两条路的每一批都只放 K 个单元、重写被截那一片下段叶与祖先（加最多一个数据单元），树高是对数；没找到让一批写入量随 S 涨的形状。
- ①b 两份盘逐字节相同：两条路的每一批都由现行那一版与意图里的 N 定，不读时钟（意图那次时间秒照抄的纪律照搬即可）；没找到分叉。

## W3　截断点落在单元中间：尾单元怎么 COW

### 打中 1：「含 Data(0) 就是整份重写」的判别在 k = 0 与 k ≥ 1 两边各错一边（推的）

- `crates/singlefs-core/src/transaction.rs:3644`：「roles.contains(&TransactionUnit::Data(DataUnitIndexInFile::FIRST));」
- `crates/singlefs-core/src/transaction.rs:3646`：「if rewritten_data_units == 0 || !file_content_is_rewritten {」
- 不写内容的发布另由 `previous_roles_without_a_successor` 补尾部：`crates/singlefs-core/src/transaction.rs:6091`：「(self.carried_data_units..previous.data_pointers.len())」，而写内容的发布那一支返回空（`:6087`–`6089`）。

截断那一批「只重写单元 k、放掉 k 之后的单元」今天没有发布形状。新加时它必须二选一：

| 新发布算哪一种 | k = 0（截到 0.3 净荷、文件原有 3 个单元） | k ≥ 1（截到 1.5 净荷） |
|---|---|---|
| 算「写内容」 | `roles_replaced_via_mapping` 见到 Data(0)、按「重写了 1 个」补 Data(1..3)，对 | 见不到 Data(0)、不补；`previous_roles_without_a_successor` 那一支也空：Data(2) 没人放，漏 |
| 算「不写内容」、留下 k + 1 个 | `previous_roles_without_a_successor` 补 Data(1..3)，`roles_replaced_via_mapping` 又因 Data(0) 补一遍 Data(1..3)：同一落点进两次换下清单 | 对 |

两次换下在快照下撞「一个单元只被杀一次」的断言（`crates/singlefs-core/src/transaction.rs:4971`：「一个单元只被杀一次」），不在快照下进两次释放。所以「复用 `deadlist_changes_of_this_publish`」那一步要先把这条判别换成按发布种类给，不能按「角色里有没有 Data(0)」猜。四句：① 不分辨 W3 甲 / 乙（两者都要只重写 Data(k) 的发布），分辨的是「复用这条判别」与「换掉它」。② 判别子：发布路径看得到自己是哪一种发布，是判别式选错了输入，不是观测不到。③ 分句：W3 题面「`roles_replaced_via_mapping` 那条「不含 Data(0) 就是尾部」的判别怎么改」。④ 改法：按发布种类交「留下几个」（我推的，零轮）。

### 打中 2：（乙, 乙）的最后一批若复用删除批的 extent 计划，留下单元数是 144 的倍数时被重写的单元 k 没有节点指它

删除批计划把「罩的单元全在留下那一段里」的下段节点照抄（`crates/singlefs-core/src/unlink.rs:193`：「last_data_unit < data_units_kept」，照抄那一支 `crates/singlefs-core/src/unlink.rs:206`：「ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,」）。删除批不改留下的单元，照抄是对的；截断的最后一批要重写单元 k（留下 k + 1 个），当 k + 1 恰是一片下段叶罩的单元数 144 的倍数时，罩着单元 k 的那片叶整个落在留下那一段里、被照抄，叶里单元 k 那一条还指着旧单元，新写的单元 k 没有任何节点指它。

量过（副本里 `unlink.rs` 测试模块加的纯函数探针，`out/extent-probe.txt` 全文）：

```
M4TRUNC extent before=200 kept=144 leaf_holding_unit=143 position=(0, 0) origin=CarriedFromThePreviousVersion
M4TRUNC extent before=200 kept=143 leaf_holding_unit=142 position=(0, 0) origin=RewrittenThisPublish
M4TRUNC extent before=200 kept=145 leaf_holding_unit=144 position=(0, 1) origin=RewrittenThisPublish
M4TRUNC extent before=300 kept=288 leaf_holding_unit=287 position=(0, 1) origin=CarriedFromThePreviousVersion
M4TRUNC extent before=3 kept=2 leaf_holding_unit=1 position=(0, 0) origin=RewrittenThisPublish
```

历史：顺序写 200 个单元 → 截断到 143.5 净荷（k = 143）→ 最后一批（产品批大小 144 时就是第一批）。结局（推的，计划之后那一步没跑）：旧单元 143 经换下清单释放，叶 0 仍指它；读单元 143 读到已释放的槽；新单元 143 占槽而没有引用。四句：① 不分辨 W3 甲 / 乙本身，打的是「最后一批照搬删除批计划」这一实现选择；（甲, 甲）在意图那次就重写单元 k，那一次要另写 extent 计划，同样要把罩着 k 的叶并进重写集。② 判别子看得到（计划函数手里有 k）。③ 分句：W3 题面「各答：哪一种不让任何一个中间态出现读者判声明长度不符」之外的那一格——指针指向已换下的单元，I-4.2 / I-1.12 这一族会红。④ 改法：截断的计划把「罩着单元 k 的那片叶到根」恒并进重写集（推的，零轮）。

### 打中 3：丙（不重写、读者读到 min(声明, size − 起点)）截短再变长读出被截掉的字节（推的）

历史：顺序写两个单元（字节全是 0xAA）→ 截断到 0.5 净荷，丙：单元 0 不重写，声明长度仍是满的 → 放掉单元 1 → 变长到 2 净荷，走 W5 甲（只改 size）。读 [0.5, 1) 净荷那一段：丙的读法取 min(声明 = 净荷, size − 起点 = 2 净荷) = 净荷，读出 0xAA；POSIX 要求截断之后再变长、那一段读出 0。丙只在「变长时按 size 判尾单元不满就重写补零」（W5 乙、而且判据是 size 不是声明长度）时不中。四句：① 分辨：只打丙。② 看得到：读者手里有 size 与声明长度，但两种情形（「声明长度之后到原 size 是截掉的旧字节」与「是写进去的真字节」）在单元上逐字节相同，丙的读者分不开——判别子观测不到。③ 分句：W3 题面「哪一种不让任何一个中间态出现读者判声明长度不符」丙是答得上的，打的是 W5「声明长度之后、size 之内读出 0 是洞还是坏单元」。④ 改法：丙只能配 W5 乙；配了之后 W7 的「尾单元声明长度对 size」核对只剩「≥」一个方向（见 W7）。

### 没打中（W3）

- 截到 0：（甲, 甲）意图那次把单元 0 重写成声明长度 0、size 0；（乙, 乙）最后一批同样做。两者都满足「长度 0 的内容也写一个声明长度 0 的数据单元」（`crates/singlefs-core/src/extent_tree.rs:11`：「长度 0 的内容也写一个声明长度 0 的数据单元」）与 I-4.6 对 size 0 的「0 或 1 个单元」。「一个单元都不留」那一种被冷走读拒（`crates/singlefs-core/src/recovery.rs:4726-4742` 拒「一个单元都没有」，事实表第二节），不算反例，是条款没放开。
- 内联指针只罩单元 0：截到一个单元以内时留下的恰是单元 0，内联照样表达；没找到「只剩一个而不是单元 0」的历史（截断恒从尾部放）。
- D16 已定项 5「一个事务最多写一个单元的用户数据」：两条路的每一次发布至多重写一个数据单元，不中。

## W4　快照下截断

### 打中：理想模型把换下的数据单元一律记成同一个出生 txg（推的）

`crates/singlefs-harness/src/model.rs:3020`：「.get(&ModelUnitRole::Data)」，`crates/singlefs-harness/src/model.rs:3026`：「birth_txg: data_written_at,」——一版里全部数据单元共一个「写于」，换下几个就重复几份。截断只重写单元 k 之后，一版里的数据单元有两个出生 txg。

历史：顺序写三个单元（t1）→ 建快照（快照纪元 t2）→ 截断到 1.5 净荷，单元 1 重写（t3 > t2），单元 2 放（进 deadlist）→ 覆盖写整份（t4，快照仍活）。覆盖写换下单元 0（出生 t1 ≤ 纪元 t2，该进 deadlist）与单元 1（出生 t3 > t2，该释放）。实现按每个映射 key 判（`snapshot_lifetime_verdict` 读映射 key 里的出生 txg），两个结局不同；模型只有一个 `data_written_at`，要么全记 t1（单元 1 该放而模型说留）、要么全记 t3（单元 0 该留而模型说放），模型与实现对拍必有一边不符。W5 乙（变长重写旧尾单元）同样造出两个出生 txg。四句：① 不分辨 W3 甲 / 乙（都只重写 Data(k)），分辨「模型照旧」与「模型按单元记出生 txg」。② 模型手里有每一步，看得到，是表示太粗。③ 分句：W8「`units_replaced_by_the_publish` 怎么算」。④ 改法：`role_written_at` 对数据单元改成按单元号记（推的，零轮）。

### 没打中（W4）

- 截断意图跨过建快照、删快照与截断意图同在队列：建快照、删快照都是 `UserChange`，先把队列走空（`crates/singlefs-core/src/mounted_session.rs:113`：「先把现行那一版的 deleted_inodes 走空」），走空被空间拒时它们自己也拒；崩在中间重开时挂载续做先于交回。我试过的形状：截断批间崩 → 重开时续做被空间拒（队列留着）→ 建快照：建快照这次用户改动先走空、走不动就拒，快照建不出来。没找到让快照落在意图之后、最后一批之前的历史（推的，没跑）。前提是截断意图进同一个走空（甲一、甲二、乙都要把它接进 `walk_the_deletion_queue_until_it_is_empty`；今天它只走 `deleted_inodes` 与销毁中的快照，`crates/singlefs-core/src/transaction.rs:8245`：「deleted_inodes 先走空，再走销毁中的快照（树号升序）」）。若截断意图不进这个走空，这一格就打开了。
- 回退与快照：回退那次在带快照的版本上先拒（`mount.rs` 的 `SnapshotsInAVersionOfTheRollbackWhichTheFirstVersionRefuses`），W1 的 H1、H2 都不带快照。
- 被重写的单元 k 的旧版进 deadlist：映射 key 取上一版的、按 key 判，单个单元的判定本身没找到错的形状；错在 W3 打中 1 的「换下清单里重复」与上面的模型。

## W5　变长与洞

### 打中 1：只改 size 的变长让扫描重建把已放掉的旧单元认领回来——量过

覆盖写、顺序写变短时，上一版多出来的尾部单元当场释放，但槽没被盖之前单元头还在盘上，同一对象出生代、同一偏移。今天 size 没盖住它们，扫描重建报不适用（C827 那一格）。变长把 size 抬回去之后，它们落进 size 之内，扫描重建按「有同出生代记录、在 size 之内」照认领（`crates/singlefs-checker/src/rebuild.rs:733`：「.filter(|record| read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET) == object_birth);」，size 那一判只拦 size 之外：`crates/singlefs-checker/src/rebuild.rs:746`：「if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {」），而遍历方向那里是洞。

原型 `extend_by_size_only_after_a_shorter_overwrite`：两块内存稀疏盘，第一个文件之后会话里顺序写 2.5 净荷（三个单元）→ 覆盖写 1000 字节（一个单元，单元 1、2 放掉）→ 把最新那一版的 size 改成 3 净荷 − 1（blocks 一起改）。

- 变长之前：`research/prompts/m4-trunc-r1-opus-model/out/z-not-applicable.txt:1`：「M4TRUNC case=shorter-overwrite-before-extend I-1.13 = NotApplicable」
- 变长之后：`research/prompts/m4-trunc-r1-opus-model/out/z-not-applicable.txt:4`：「M4TRUNC case=extend-after-shorter-overwrite I-1.13 = Violated」
- 换成 Z 不认领，变长之后照样违例：`research/prompts/m4-trunc-r1-opus-model/out/z-unclaimed.txt:4`：「M4TRUNC case=extend-after-shorter-overwrite I-1.13 = Violated」
- 对照（没有留下旧单元，同样只改 size）：`research/prompts/m4-trunc-r1-opus-model/out/z-not-applicable.txt:8`：「M4TRUNC case=extend-without-left-units I-1.13 = Holds」

违例原样（`z-not-applicable.txt` 第 4 行括号里）：

```
扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 2 处，第一处 key （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）：扫描重建 落点 {盘 0 槽 50184、盘 1 槽 50184}、遍历方向没有这一条；「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 1、偏移 32634）：扫描重建 （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）、遍历方向 没有这一条
```

扫描重建一侧多出来的两条就是被放掉的单元 1、2（出生 txg 4 是那次顺序写）。I-1.13 之外这张镜像上别的不变量 0 条违例（`z-not-applicable.txt:5`：「M4TRUNC case=extend-after-shorter-overwrite violated_count = 1」，那一条就是 I-1.13）。今天的挂载态读者会拒这张镜像（条数对不上，`mounted_read.rs:815`），所以这不是「今天就读错」，是「W5 甲落地、读者放开洞之后，扫描重建这条降级路径会把旧字节当成洞里的内容交出去」：D18 已定项 5 把重建定为降级路径，一次重建就让用户在洞里读到截掉之前的字节。

历史要的东西都现成：覆盖写变短（今天就有）、变长（W5 甲）。截断也一样造得出（截断放掉的尾部单元同样留在盘上），所以截断 + 变长同中。

四句：① 分辨臂：W5 甲中；W5 乙（只重写旧尾单元补零到单元末尾）照样中——单元 1、2 的位置在乙下也是洞（推的，没造：乙改的是单元 0，单元 1、2 的头不变）；只有「变长时把 [s, N) 显式写成 0 单元」不中，而它的写入量随 N − s 涨、要走意图。② 判别子：扫描重建看得到的是单元头（同对象出生代、同偏移、没有更新的版本）与 size，「洞里的旧单元」与「size 之内的活单元」在这两样上逐条相同——判别子观测不到，改的是判据（读法 Z）。③ 分句：I-1.13「三张派生表逐项相同」，「每个 inode 的数据单元集合」与「中央映射」两张都差 2 处。④ 改法：「Z 不认领」不修这一格（上面 `z-unclaimed.txt:4` 仍违例）；「Z 加写序」修（W6）。

### 打中 2：今天三条写入口下洞只可能是后缀，`data_pointers: Vec` 不必改成映射（推的，反向发现）

三条写入口都从偏移 0 写整份内容（`FileVersionPlan` 文档，事实表第三节），截断只从尾部放，变长只改 size（甲）或只重写最后一个单元（乙）。在这几种操作组成的任意历史里，extent 树里的单元号恒是 0..m 连号、洞只在 m 之后到 size 之间。所以 W5 题面「`TransactionOutput::data_pointers` 按下标当单元号的内存态怎么表达洞（改成按单元号索引的映射，还是别的）」在这一轮的操作集合上答案是「不用改」：`Vec` 的长度就是现有单元数、size 另记。要改的只是「单元数 = ⌈size ÷ 净荷⌉」那几处判式（冷走读 `crates/singlefs-core/src/recovery.rs:5376`、挂载态读 `crates/singlefs-core/src/mounted_read.rs:815`、I-4.6、C711）换成「≤」。推翻条件：这一轮加任何「写到偏移 o」的入口（pwrite），后缀论断立刻失效。

### 没打中（W5）

- 洞之后整份覆盖写：整份写从 0 起连号写满，洞被写成真字节的单元（含写进去的 0），没有「保持洞」那一种可能，也就没有反例。
- 变长之后再缩回洞里（N 落在 m 之后）：没有单元可放、可重写，一次发布改 size；若 N 落进最后一个单元的声明长度之内，等同普通截断。没找到额外的形状。
- 变长要不要意图：甲不放、不写任何单元，乙只写一个单元，都一次发布做完，没有批间中间态。
- I-9.15：blocks 按 size 算，与分到多少空间无关，洞不影响（`records.rs` 的 `to_bytes` 现算）。

## W6　扫描重建与读法 Z

### 打中：Z 原文同时在两处错——量过；我提的「Z 加写序」只修一处

W2 打中 2 与 W5 打中 1 合起来：

| 镜像 | 今天（size 之外报不适用） | Z 不认领 | Z 不认领 + 写序（我提的，零轮） |
|---|---|---|---|
| 覆盖写变短留下的单元（C827 那一格） | NotApplicable（`z-not-applicable.txt:1`） | Holds（`z-unclaimed.txt:1`） | Holds（`z-write-order.txt:1`） |
| 变短之后只改 size 变长 | Violated（`z-not-applicable.txt:4`） | Violated（`z-unclaimed.txt:4`） | Holds（`z-write-order.txt:3`） |
| （甲, 甲）截断中间态 | NotApplicable（`z-not-applicable.txt:12`） | Violated（`z-unclaimed.txt:13`） | Violated（`z-write-order.txt:12`） |
| 对照：没留下旧单元的变长 | Holds（`z-not-applicable.txt:8`） | Holds（`z-unclaimed.txt:8`） | Holds（`z-write-order.txt:7`） |

每一格都是量过（副本，`out/` 三份原样，行号即上表括号里的）。逐格的原样行：

- `research/prompts/m4-trunc-r1-opus-model/out/z-write-order.txt:3`：「M4TRUNC case=extend-after-shorter-overwrite I-1.13 = Holds」
- `research/prompts/m4-trunc-r1-opus-model/out/z-write-order.txt:1`：「M4TRUNC case=shorter-overwrite-before-extend I-1.13 = Holds」
- `research/prompts/m4-trunc-r1-opus-model/out/z-unclaimed.txt:1`：「M4TRUNC case=shorter-overwrite-before-extend I-1.13 = Holds」

「Z 加写序」的定义（`rebuild.patch` 里那一段）：同一 (inode, 对象出生代) 的当前版本数据单元里，若有偏移更小、出生 txg 更大的一个，这一个判死、不认领。依据（推的）：这一轮的操作集合里，活单元的出生 txg 随偏移不降——整份写同一个 txg，截断只重写最后留下的单元 k、k 之后全放，变长乙只重写最后一个单元；被放掉而还在盘上的单元出生 txg 一定小于后来重写的那个更低偏移的单元。

它在已有用例上的误伤：副本里带 `M4_TRUNC_R1_Z_WRITE_ORDER=1` 跑 `scan_rebuild_agrees_with_the_walked_index` 整个目标，原样末行 `test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.43s`，挂的那一条是 `a_data_unit_left_beyond_the_size_by_a_shorter_overwrite_is_not_applicable_to_the_scan_rebuild`（它钉的就是「报不适用」，加写序之后那两个单元先被写序判死、走不到 size 那一判，变成成立）——这是改法要改的那一格，不是误伤；其余 7 条照过。

四句：① 分辨臂：分。「Z 加写序」在（乙, 乙）+ 变长甲 / 乙的组合上三格全成立（前两格量过，（乙, 乙）中间态推的：留下的全在 size 之内、写序不降）；（甲, 甲）的中间态谁都修不住。② 判别子：写序那一条用的是单元头里已有的出生 txg，扫描方向看得到。③ 分句：I-1.13 逐项相同；D18 已定项 10 读法 Z 那一句的字面「按……size 判死活」。④ 改法与限度：「Z 加写序」只在我的模型上量过、被攻过零轮；推翻条件是出现「写到偏移 o 而不重写 o 之后的单元」的入口（pwrite、punch）——那时活单元的写序不再随偏移单调，这条规则会把活单元判死。它也改了 D18 已定项 10 的读法本身，要走三方。

### 没打中（W6）

- I-1.9（清扫水位内无可读垃圾）未实现：它管的是「水位之下不许有可读垃圾」，覆盖写变短留下的单元本来就是它的对象；截断落地不改变这一格的性质，只是多一个来源（截断放掉的尾部）。没造出截断特有的形状。
- 截断墓碑：D18 已定项 10 定了部分删除不写区间墓碑，扫描重建只认整对象墓碑（`crates/singlefs-checker/src/rebuild.rs:658-675`）；截断不写墓碑就不碰这一支，没找到反例。

## W7　checker 多判哪几条

### 打中 1：W1 乙下「截断中有账」造不出必红世界（推的）

照 I-4.7 的写法立「size 之外还有单元的 inode 要么有截断意图、要么一个不剩」。乙的意图就是「size 之外还有单元」，这条恒真。必红世界「最后一批放少了一个单元」在乙下是一个合法的、等着续做的中间态；下一次挂载续做把它放掉，checker 没有一版会红。判别子观测不到：checker 能看到的（size、单元数）在「等着放」与「漏放」两种情形下逐条相同。甲一 / 甲二下这条有对象（看 logged_ops 或截断树里有没有那条），必红世界造得出（删掉意图条目、留着尾部单元）。

### 打中 2：（乙, 乙）下「最后一批与删意图同事务」改坏之后池级 checker 看不出（推的）

W9 题面要的必红世界：改坏「最后一批与截断意图同事务」。在（乙, 乙）下改坏的样子是「倒数第二批顺手删了意图」，崩在那之后，留下的一版是：size = 留下单元数 × 净荷、单元全满、没有意图——一个完全合法的文件。池级 checker 每条不变量照过；只有拿 RefFS / 模型比内容（期望 N）的那一层看得出。（甲, 甲）下同一改坏留下「size = N、size 之外有单元、没有意图」，「截断中有账」那条红。所以这一格的必红世界在（乙, 乙）下只能挂在对拍上，不能挂在池级 checker 上；挂在对拍上就要求 RefFS 的允许集合不收「任一批之后的前缀」作终态（W8）。

### 打中 3：W3 丙 × W5 甲一起把尾单元声明长度核对放到什么都不判（推的）

W5 甲要读者容忍声明长度 < min(size − 起点, 净荷)，W3 丙要容忍 >。两样都上之后，「尾单元声明长度对 size」那条新判据剩下的只有「≤ 净荷」，写路径把声明长度写错一个字节的变异在 checker 上不红（内容对拍仍可能红）。W3 甲 / 乙 + W5 乙时等式对每个现存单元都保得住，checker 那条可以写成「现存单元的声明长度 = min(size − 起点, 净荷)」原样。

### 没打中（W7）

- I-4.4 白名单加截断种类：value 34 字节装 inode 8 + 对象出生代 8 + N 8 + 补零 10，「再截 N 个」这样的增量没有地方放；没找到能塞进增量而不出白名单的形状。前提是 N 是绝对字节数而不是「再截多少」。
- 活 inode 的单元数对 size（C711）：（乙, 乙）+ 变长下可以统一写成「单元数 ≤ ⌈size ÷ 净荷⌉.max(1)」；（甲, 甲）下要另带「有截断意图时可以多」。这是代价，不是反例。
- GPU 判器：新条目照 I-4.4、I-1.13 的先例进覆盖表、内核不置位；对拍不比那张表的登记是流程，没有可造的反例。

## W8　模型、RefFS、翻译层、随机历史

### 打中 1：`units_replaced_by_the_publish` 的单一出生 txg——同 W4 打中，推的

不再重复；那一段的历史同样是 W8 的反例：模型的 `ModelPublishKind` 加几个成员都不够，`role_written_at` 对数据单元要按单元号记。

### 打中 2：RefFS 允许集合在（乙, 乙）下要放宽、在（甲, 甲）下不用（推的）

崩溃落在意图与最后一批之间，读者在崩溃镜像上（只读挂载、冷走读、回退目标）答什么：

| 路 | 崩在批间的崩溃镜像上读者答的 size | RefFS 允许集合要收的 |
|---|---|---|
| （甲, 甲） | N | 截前 S（崩在意图之前）、截后 N |
| （乙, 乙），读者只读记录 | (k + 1) × 净荷 .. S 之间按批步进的某一个 | 截前、截后、加每一批之后的前缀 |
| （乙, 乙），读者按意图里的 N 答 | N | 截前、截后；代价是冷走读、只读挂载、模型读都去读截断意图 |

第二行把允许集合放宽成「任一前缀」之后，一个把文件截错到某个批边界上就停下的实现缺陷（W7 打中 2 那一形）在 RefFS 一侧也合法——对拍那一层也看不出了，这格的必红世界就两头都没有。

### 没打中（W8）

- 理想模型整份字节补 0 能不能区分洞与写过的 0：读者答案相同，W5 打中 2 说明这一轮洞只是后缀，模型记「现有单元数 + size」两样就够，不用逐字节标洞；没找到模型非知道不可的形状（W6 的扫描重建一侧除外，那归 checker）。
- `UserChange` 拒删除中的 inode、没有文件、N 超上限：走空之后删除中的 inode 已不在树里，前两样合成「没有文件版本可改」；N 超上限只是参数检查。没找到反例。
- 随机历史 `simpler_variants` 怎么缩：缩 N 与缩批大小两维；没找到会把一条打中的历史缩成不中的方向（没跑）。

## W9　崩溃流、必红世界与只供测试的开关

### 打中 1：接在 3 单元流后面的截断流够不着叶边界那一格（推的）

W3 打中 2 只在留下单元数是 144 的倍数时出现，要 ≥ 145 个单元的文件（下段叶罩的单元数是格式常量 144，不随批大小开关变）。D8 已定项 16 射程里的崩溃流用 3 单元文件，压小的是批大小、不是叶宽，那条流上的每个崩溃态都碰不到这一格。要么另立一条 145 单元起的流（每个单元一个写，崩溃态数随单元数涨，要先用 `crash_state_count_with_torn_in_place_overwrites` 估），要么用 harness 层纯函数用例钉计划（我量的那一种）。

### 打中 2：「强制尾单元重写在意图那次还是最后一批」的开关单独切会造出读者判不符的一臂（推的）

W2 打中 1 那张表：W3 甲配 W2 乙、W3 乙配 W2 甲两格每个中间态都判声明长度不符。这个开关若只切尾单元重写的时机、不连着切 size 的写法，强制出来的那一臂本身不是合法分支，跑出来的红不说明产品分支有错。`.claude/rules/fs-design.md` 五条硬要求第 2 条要的「能强制走指定分支的测试开关」，这里要切的是（甲, 甲）/（乙, 乙）这一对，不是 W3 单独一维。

### 打中 3：W1 的回退那一格不在任何一条计划的流上（推的）

题面列的流（缩到单元中间、缩到 0、变长留洞、快照下缩）都不含管理员回退。W1 H1 要「截断做了一半的一版」作回退目标、H2 要「挂载续做被空间拒」再回退，两样都不在崩溃点重放的写流里——回退是一次普通的向前发布，崩溃枚举它的写不会让甲的错误显形；要的是一条「截断批间 → 回退到批间那一版 → 续做」的确定性用例与它的必红世界（把 logged_ops 改回随现行）。

### 没打中（W9）

- 「最后一批与截断意图同事务」改坏之后（甲, 甲）的流判器红不红：红（「截断中有账」那条，W7 打中 2 的反面），没找到让它不红的形状。
- 批大小开关：与 unlink 共用 `UnlinkBatchSize`（1..=144）够用，没找到要另立开关的历史。

## 自提改法与它们各修哪一格

全部只在我的模型上量过或只是推的，被攻过零轮。

| 改法 | W1 H1 / H2 | W2 交叉两格 | W2（甲, 甲）× I-1.13 | W5 变长 × I-1.13 | W3 k = 0 / k ≥ 1 | W3 叶边界 | W4 / W8 模型出生 txg |
|---|---|---|---|---|---|---|---|
| 甲一或甲二（截断意图随 R_old 回退） | 修（推的） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 只走（甲, 甲）或（乙, 乙）两条联动的路 | 不碰 | 修（推的） | （甲, 甲）不修、（乙, 乙）修（推的） | 不碰 | 不碰 | 不碰 | 不碰 |
| 读法 Z 不认领 | 不碰 | 不碰 | 不修（量过，`z-unclaimed.txt:13`） | 不修（量过，`z-unclaimed.txt:4`） | 不碰 | 不碰 | 不碰 |
| 读法 Z 不认领 + 写序 | 不碰 | 不碰 | 不修（量过，`z-write-order.txt:12`） | 修（量过，`z-write-order.txt:3`） | 不碰 | 不碰 | 不碰 |
| 按发布种类交「留下几个」，不按 Data(0) 猜 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 不碰 | 不碰 |
| 截断计划恒把罩着单元 k 的叶到根并进重写集 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的；照抄那一半量过，`out/extent-probe.txt` 第 1、4 行） | 不碰 |
| 模型对数据单元按单元号记出生 txg | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） |

要交用户的岔路（候选、翻面观测、够判条件，不带倾向）：

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| W2 × W3 的联动 | （甲, 甲）：意图那次写 size = N 并重写单元 k；（乙, 乙）：每批 size = 留下单元数 × 净荷，最后一批重写单元 k、写 N | （甲, 甲）的中间态上 I-1.13 是否有不越过「两个方向只共享格式解析」的判法；（乙, 乙）下读者按意图答 size 的代价（冷走读、只读挂载、模型各多读几样） | 两条各在入库装置上造一张批间镜像，I-1.13 与读者的答案各量一次 |
| 截断意图住哪 | 甲一 logged_ops 按种类分回退来源；甲二 另立随 R_old 的截断树；乙 记录本身 | 回退那次发布多出的写与崩溃态数；乙的「有账」必红世界造不造得出 | 甲一、甲二各写出回退那次的重写角色集合并数崩溃态；乙给出一个池级 checker 红的必红世界，给不出就判「造不出」 |
| 读法 Z 在洞上的修法 | Z 加写序；变长时显式写 0 单元（走意图）；扫描重建对有洞的文件报不适用 | 出现「写到偏移 o 而不重写 o 之后的单元」的入口时写序是否还单调 | 这一轮的操作集合定下来（有没有 pwrite / punch） |

## 没打中的形状（汇总）

- 会话内两条意图并存（截断与截断、截断与 unlink、截断与删快照）：7 种用户改动两两接在一条没做完的截断后面、外加挂载续做被空间拒，共 49 + 7 种排列，全部被「每次用户改动先走空、走不动就拒」挡住（推的，没跑）。唯一的绕过是管理员回退（W1）。
- 截断意图跨过建快照：同上，被走空挡住（推的）。前提是截断意图接进 `walk_the_deletion_queue_until_it_is_empty`。
- 一批写入量随文件大小增长：两条路都不随 S 涨（推的）。
- 截到 0、内联指针、一个事务至多一个数据单元：没找到反例（推的）。
- 原型取样：harness 三个用例（三单元顺序写后覆盖写成一单元再变长、第一个文件直接变长、三单元文件截断中间态）× 读法三臂（今天、不认领、不认领加写序），加 extent 计划探针五个几何（200→144、200→143、200→145、300→288、3→2）。每次跑不到 2 秒（`finished in 1.62s` / `1.63s` / `1.72s`，探针 `0.00s`），没有缩取样。

## 这条腿自己的限度

- 截断与变长没实现：原型用改记录字节模拟「候选落地之后那一版」，不是候选真写出来的那一版。改的只有 size 与 blocks，单元、extent 树、意图载体都是前缀历史留下的样子；（甲, 甲）中间态用的是截断点恰在单元边界、不用重写尾单元的那一格。真实现之后的镜像可能还有别的差异。
- 崩溃枚举没跑：崩溃状态枚举函数在 checker 档包里（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:243`：「pub fn enumerate_crash_states(」），不在定义写的 `memory_pool.rs`；harness 档的测试目标依赖它就按重型拒，所以我的原型一个崩溃态也没枚举（全量状态数没算，跑过的崩溃态数 0）。打中的几格都在不崩溃的终态镜像上，不靠崩溃态；崩溃态上会不会多出别的形状没看。
- W1、W3 打中 1、W3 打中 3、W4、W7、W8、W9 全是读码推的，没造镜像。W1 那两段历史要回退那次发布真跑一次才算量过（`common_deletion_in_progress::roll_back_to` 现成，但截断没实现，造不出 v一批）。
- 「Z 加写序」只在副本里三张镜像与已有的 8 条扫描重建用例上看过；随机历史与回退、快照下的镜像没跑。
- 一次抽样：「没打中」的各条按规则不算没打中，要第二次抽样。

## 没做什么

- 没判正推、辩方那几格，没替主 agent 采纳；副本上的数不是入库装置上的数。
- 没跑门禁阶段、没跑 checker 档测试、没起 GPU、没跑重型测试。没有要主 agent 跑的命令。
- 跑之前看负载：别的会话有一条 `cargo test --release -p singlefs-checker-tier … crash_enumeration_unlink_after_the_multi_record_publish_stream …` 在跑（pid 2368099），没有性能测量；我的每条 cargo 加 `nice -n 19`、经 8G 内存包装、`capped.sh 31`，没等锁。
- 草稿目录 `~/.local/share/singlefs/work/m4-trunc-r1-opus/` 里留日志（`run-*.log`、`build*.log`、`rerun-check.out`）与两份补丁原件；仓副本 `repo/`、编译目录 `target/`、复跑检查的 `rerun-check/` 交回前删掉。
