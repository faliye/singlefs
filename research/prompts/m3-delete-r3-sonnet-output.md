# m3-delete-r3 辩方腿报告（Sonnet，2026-10-01）

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 V6 V7 S1 S2 R3 L4 X1 F1 T1 M1 M3 Z2 E-b E-c W1 W2 W3 W4 W5 W6 W7 G1 G2 G3 G4 G5 G6 G7 G8 N1 N2 N3 N4 N5 N6 K1 O1 -->

批次：records/2026-10-01-删除与并发事务.md#3；轮名 m3-delete-r3（删除第三轮、墓碑第二轮，最终轮）；立场：云端辩方。读的是冻结副本 `/tmp/claude-1000/m3-delete-r2/tree/crates/`（`sha256sum -c` 对清单 `research/prompts/m3-delete-r3-snapshot/crates-sha256.txt` 0 项不 OK）；引代码的行号对它，与主仓 `crates/` 逐字节相同（主 agent 开工前现查）；kb 行号是主仓此刻的。我提的形态与收严都被攻过零轮，逐处标「零轮」；数分三类：「量过」（副本原型上跑出来的，原样行在模型目录）、「算的」（纯算术，`ring_window_cells.py` 复现）、「推的」（读代码或按条款推，没跑）。

## 零、模型、复跑、中途收到的消息

- 模型目录 `research/prompts/m3-delete-r3-sonnet-model/`：`m3_delete_r3_defense_cells_part.rs`（接在删除第二轮攻方原型文件后面的 5 个用例，只用 harness 档）、`ring_window_cells.py`（算术与对拍，每个断言钉绝对值）、`run-final-r-lines.txt`（32 行 `R name=`）、`cells.out`（`C name=` 行）、`rerun.sh`。
- 文件 sha256（`sha256sum`）：
  ```
  d498d1ff57d2fd8486bcb793b2bc3acd8b502dd74cf401f1543c8011ae1e3120  cells.out
  7f10229668c02b94b27c34ffa1b3996e04ce6a663c3853954f444e1e11427e18  m3_delete_r3_defense_cells_part.rs
  cc0c642e47b1fc77aef86f8186022cc917bac70e00fe138010322cff16dc5a1b  rerun.sh
  fd493cae6eeff8c07e5c2fb6881973ef49a59276c91159861276c31fd5c5c97a  ring_window_cells.py
  23d0290bdc49f2ef05de4186093c0e5e095f25ad91b8285a32273834605da5f1  run-final-r-lines.txt
  ```
- 复跑（从仓根；草稿目录必须还不存在，用完自己删）：`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=3600 RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由> RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON=<理由> bash research/scripts/run-with-memory-cap.sh 6G bash research/scripts/capped.sh 5 bash research/prompts/m3-delete-r3-sonnet-model/rerun.sh <仓根> /tmp/claude-1000/m3-delete-r2/tree <草稿目录>`。我从模型目录从头跑过一次：输出末三行是「R name= 行相同」「cells.out 相同」「exit=0」。
- 原型没有意图记录、`deleted_inodes`、墓碑、谱系记录、nlink 字段；账房是攻方原型的 `LEDGER_FOR_TESTS`（标签 4、6 两个按全部副本之和记的字节，准入读数与记账行都读它）。S2 比原型多的槽数按角色表另加，标「算的」：意图那次 +4 槽 / 盘（`deleted_inodes` 根 1 + 墓碑容器 2 + 墓碑树根 1），最后一批 −2 槽 / 盘（inode 叶容器不再写）。
- 中途收到主 agent 的消息四条（线程上限改 10、8、6、5）：第一条在我跑完 `g1_w_by_batch…` 第一版（线程 16）之后，其后的命令照新值；最终入库的 `run-final-r-lines.txt` 与复跑都在线程 5 下出的（线程数不改这些数）。
- 开工 `ps` 看到别的会话的 `cargo nextest` 与一个远端变异在跑，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；每条都加了 `nice -n 19`，没有等锁；包装第一次报「这个会话已经有一条日常的在跑」（同一会话里攻方腿的用例），设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 照理由并行跑。

## 一、五条的结论（逐条依据在后面各节）

1. **「一轮」各行**（第二节表）：删除第二轮 V1 的 ①②③①b、V2「不论选哪支都要的」里的 (a)(c)(d)(e)、V5 第三成员，墓碑第一轮的 W1、W2、W3、W4，以及把 D23:417 判成被打穿那一处，在 S2 上都**站得住**。三处要改：① V2 (b)「删除类发布按发布之后的标签抵扣」对环窗口**不成立**——窗口账下发布前后的标签相等，抵扣额为零；量过：可用 16 343 字节的池上，按朴素读数判，意图那次被拒，关掉式子那一判四次发布全过（第三节 3.3）；这一条只在**写入类**发布上还要（收标签的增量）。② V5 里「inode 树零条目根的 key 区间取 `[0, u64::MAX]`」与 D8（核心索引结构） 已定项 6 与 I-9.1 逐字写的 `[0, 0]` 冲突，读者都不判空节点的区间，没有功能后果，改回 `[0, 0]` 不用改 kb。③ V1 ④ 原来抓的那一族（待删占用减两次、不减）在 S2 里**不存在**（标签 4 恒 0、标签 6 不随批变），④ 留着，靶子换成墓碑份额与窗口行。
2. **G1**（第三节）：标签 4 恒 0，意图那次不转、各批不抵；标签 6（每盘槽）= (L + 1) × W_ub + 3 + 41 B × 活对象数 ÷ 16384，L = 3 × 根环每区槽数（系统不可变配置 `root_ring_slots_per_region`，4 到 16，所以 L 是 12 到 48），W_ub 是一次删除类发布每盘新占槽数的上界，对 K = 144 在 4 GiB 盘上是 368 槽（量到的最大值 307 到 312，一单元一叶的散落造法），窗口在 4 GiB、L = 24 是 9 200 槽 / 盘（单元区的 4.34%）。删除类发布**不判式子那一判、判落点那一判**。窗口从第一个文件版本起计。环窗口量出来的顶点 258 / 304 / 712 与第二轮逐字相同，窗口在账里时删除全程没动到保留量，不在账里时借走 258 / 304 / 712 槽。
3. **G2**（第四节）：五条判据在 S2 上判得住**逻辑上的**非幂等，前提是逻辑内容写明含死亡 txg 与谱系记录（删除流里恒空）；判不住的两类：开放容器在续做里多写（逻辑内容不变）与走共用写路径的重复墓碑（③ 两边一起错）。收严两条，都零轮：⑤ 每种发布的重写角色集合钉绝对值（harness 层）；checker 加「谱系上每个（号，出生代）至多一条墓碑」。①b 由 1 个样本补到 6 个，全部逐字节相同。
4. **G5**（第五节）：三棵树的空形态在今天的读者上**只有冷启动只读挂载与池级 checker 认**，恢复重建（`recovery.rs:2453`、`:2506`、`:3298` 与 `extent_tree.rs:664`）全拒；零根指针对 inode、extent 两棵连只读挂载都拒（`mounted_read.rs:414`），所以这两棵只能取零条目根，`deleted_inodes` 可取零根指针（`recovery.rs:3910` 已经跳过零根的树表条目）。墓碑树不空对第三成员的判定**没有影响**，前提是判定写成「树表有条目、inode 树零记录、extent 树零条目、队列空」，不看墓碑树。
5. **G8**（第六节）：F1 状态数在 S2 上重算：意图那次 13 个单元，合计 92 274 732（E-c）/ 89 129 004（E-b），全域 1 451 229 366 / 1 448 083 638（单文件池上的单元数算的，流上没量）；815 条一容器算对；10⁶ 次 unlink 的 76.69 MiB 算对，但单层根只装 135 个容器 = 110 025 条，十万次以上要先有多层墓碑树；环窗口顶点三个数在冻结副本上逐字复现；回退那次发布多写 2 个单元（谱系容器 + 墓碑树根，推的），闭式 1 048 583。

## 二、两份判决里标「一轮」的各行，在 S2 上还站不站得住

「站得住」= S2 上照样成立；「打穿」= S2 上不成立，写明改成什么；「够不着」= 它攻的那一族在 S2 里不存在。最后一列写：同一击是不是同样落在每一个替代方案上（不分辨臂），与什么现象会推翻。

| 判决条目 | 辩方判定 | 依据（产物整行或代码行号） | 不分辨臂？/ 推翻条件 |
|---|---|---|---|
| 删除 V1 ① 计划相等；② 删完再续做一个写都不发 | 站得住（弱，只看计划）；② 在 S2 上多一个前提：删完那一版要认得是「完成」，靠第三成员（V5 行） | 续做计划只读现行那一版；核心里没有无序集合：`grep -c 'HashMap\|HashSet'` 对 `transaction.rs allocator.rs extent_tree.rs inode_tree.rs mount.rs recovery.rs` 六个文件输出全是 0 | 不分辨。推翻：续做计划读了会话内存、时钟或无序迭代 |
| 删除 V1 ③ 每个写点截断、重开续做到底，终态逻辑内容相同、checker 全绿 | 站得住，S2 补两处：(i) 逻辑内容写明含墓碑的死亡 txg（意图那次发布写，崩溃重放保留同一个 txg，两条路径相等）与谱系记录（删除流里恒空）；(ii) 崩在意图发布里的写点，恢复落到意图之前或之后一版，终态取多版本 oracle：删完，或删之前那一版的内容 | 崩溃流本来就跑多版本 oracle：「每个状态跑恢复 + 多版本 oracle」（`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:5`）；「逻辑内容」的表在删除第二轮辩方报告 `research/prompts/m3-delete-r2-sonnet-output.md` 的 4.4 节（第 243 行起） | 不分辨。推翻：意图那次发布经恢复之后的 txg 与不崩溃的不同（那时死亡 txg 不能进逻辑内容） |
| 删除 V1 ④ 每个被判状态的标签 4、6 等于镜像现算值 | 站得住，但它原来抓的那一族**够不着 S2**：攻方的非幂等续做是待删占用减两次 / 不减，S2 里没有逐批的计数器，标签 4 恒 0，标签 6 在四次删除发布里是同一个值 | 原靶子：「E name=v1_criteria rule=SubtractedTwice」（`research/prompts/m3-delete-r2-opus-model/run-final-e-lines.txt:18`）；S2 上量过：「label4_row=0 label6_row=301563904」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:5-8`）；④ 的新靶子是墓碑份额（每活对象 41 字节）与窗口行（第三节） | 不分辨。推翻：S2 又引入逐批增减的标签（例如把窗口按在飞的删除个数记） |
| 删除 V1 ①b 同镜像重挂载续做逐字节相同（零轮，一个样本） | 站得住；样本由 1 个补到 6 个，全部逐字节相同 | 六种镜像（3 / 300 / 144 / 240 单元，K = 1 / 144，散落或连续，停在意图之后或第一批之后）各自重挂载两次续做到底：「images_byte_equal=true outputs_equal=true」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:27-32`）；原型没有墓碑，S2 的墓碑写路径没量 | 不分辨。推翻：S2 的续做取到时钟、随机数或无序迭代 |
| 删除 V2「不论选哪支都要的」(a) 标签 4、6 做成版本的函数 | 站得住；S2 上更简单：标签 4 恒 0，标签 6 只依赖盘几何、系统配置、K 的产品常量、活对象数，镜像上全算得出（第三节 3.6） | 记账行取值今天与准入读数读同一个常量：「这两行与准入读数读的是同一个常量」（`crates/singlefs-core/src/transaction.rs:6282`） | 不分辨。推翻：窗口里要用盘上没有的量（例如 K 旋钮） |
| V2 (b) 删除类发布按「发布之后的标签」抵扣，不按发布前读数判 | **打穿（对 S2）**：窗口账下删除各批发布前后标签相等，抵扣额为零。量过：写完之后可用 16 343 字节的 4 GiB 池上，按朴素读数判，意图那次被拒；式子那一判关、落点那一判开，四次发布全过。S2 改成「删除类发布不判式子那一判」。这一条在**写入类**发布上仍要（收标签增量：建 inode 抬标签 6，第一个文件版本抬一整份窗口） | 「result=Err(intent:SpaceAdmissionRefused[(16384, 180224), (16384, 180224)])」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:10`）；「variant=formula_skipped_landing_judged result=Ok」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:11`），四次发布之后每盘可用 −163 840、−376 832、−573 440、−770 048 字节（原型值；S2 另加 +4 / −2 槽之后见第三节表） | 只打中环窗口这一支：R3 那一支标签随发布转，抵扣有意义（`research/prompts/m3-delete-r2-sonnet-model/cells.out:13-17` 的紧池表）。所以它不是「不论选哪支都要的」。推翻：窗口改成随在飞删除转进标签 4 |
| V2 (c) W 连固定点一起算 | 站得住：固定点在大批里压倒普通分配 | K = 144、一单元一叶：新占 307 槽，其中固定点 303：「fixed_point=[13, 303]」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:21`）；连续落点时固定点 8 到 15（删除第二轮 `E name=w_row`） | 不分辨。推翻：固定点被别的预算（保留池）单列，不进 W |
| V2 (d) 写明式子那一判与落点那一判各关不关 | 站得住，S2 定：式子那一判**关**，落点那一判**开** | 落点那一判是唯一管物理空槽的：「if units_landing_is_judged {」（`crates/singlefs-core/src/transaction.rs:5089`）；它拒了之后会话推抬 F：「PublishError::PlacementRefused { .. } => true,」（`crates/singlefs-core/src/mounted_session.rs:149`）——窗口的用处就是不走到这一步（保回退深度，第三节 3.5） | 不分辨。推翻：落点那一判在窗口够用的池上拒了 |
| V2 (e) 墓碑单元算进同一份 | 站得住，细化：意图那次的 4 槽（墓碑容器 2、墓碑树根 1、`deleted_inodes` 根 1）是**暂态**，落在每批 W 里由窗口兜；墓碑记录本身**永久**（第一版不回收），每活对象预付 41 字节（按记录摊）加全池 3 槽整容器余量（第三节 3.6） | 与 W3 行同源：不回收，所以按对象预付永久份额才有意义 | 不分辨。推翻：第一版开始回收墓碑 |

| 删除 V5 第三成员：「有树无文件」是 `RebuiltVersion` / `PreviousVersion` / `PoolVersion` 的第三个成员，不借 `WithoutFile` | 站得住；S2 上没有第三条路：`WithoutFile` 的定义就是树表 0 条，而 S2 删完之后树表仍 9 条 | 定义：「/// 树表 0 条：所选根下面还没有发布过文件版本」（`crates/singlefs-core/src/recovery.rs:2271`）。借它要撞的六处都还在：会话「PoolVersion::WithoutFile(_) => return Err(UserChangeRefused::NoFileVersionToChange),」（`crates/singlefs-core/src/mounted_session.rs:261`）；卸载「return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {」（`crates/singlefs-core/src/mount.rs:2080`）；挂载前那一判「PreviousVersion::WithoutFile { .. } => {」（`crates/singlefs-core/src/mount.rs:3752`）下面一行直接拒挂载；回退候选「match tree_table_has_no_entries(reader, &target_root) {」（`crates/singlefs-core/src/mount.rs:4753`）；回退入口「回退的调用方要带文件的现行版本：cur 树表 0 条时它的时间线上还没有带文件的版本，」（`crates/singlefs-core/src/mount.rs:5042`）；以及第六处（两腿都没点名）：只读挂载找 inode、extent 两棵树要根指针非零「entry.kind == kind && entry.root != NodePointer::empty_root()」（`crates/singlefs-core/src/mounted_read.rs:414`），借 `WithoutFile` 的版本没有这两棵 | 不分辨：只要最后一批删掉 inode 记录而树表不退回，每个替代方案都要这个成员。推翻：删完之后树表退回 0 条（水位要另找地方放，格式改动） |
| V5 里 inode 树零条目根的 key 区间「取常量 `[0, u64::MAX]`」 | **打穿（子句）**：kb 已逐字写 `[0, 0]`，改回 `[0, 0]`，不用改 kb | D8 已定项 6：「哪怕树是空的（零条目，key 区间写 [0, 0]，0 是无效 inode 号）」（`.claude/kb/decisions/08-核心索引结构.md:159`）；I-9.1：「哪怕树只有一片叶或是空的（零条目、key 区间 [0, 0]）」（`.claude/kb/invariants.md:275`）；两个读者都不判空节点的区间：「if let (Some(first), Some(last)) = (node.entries.first(), node.entries.last()) {」（`crates/singlefs-core/src/recovery.rs:3253`）只在有条目时核；「区间贴紧首末条目（空节点区间任意）」（`crates/singlefs-checker/src/lib.rs:597`）。所以 `[0, u64::MAX]` 没有功能后果，只是与两条已定文字相左；①b 要的只是「一个常量」，`[0, 0]` 满足 | 只打 inode 一棵。推翻：用户要把 inode 树空根改成罩整个 key 空间（那要改 D8 与 I-9.1 两处文字） |
| 墓碑 §2：D23:417 判成被打穿 | 站得住（打穿成立） | 那句依据：「撑的是回退行，回退行删了，那一半不再承重」（`.claude/kb/decisions/23-journal的角色与格式.md:417`）；它指的 E104 那一半恰是「不写回退行时全规则臂也复活 3」（`.claude/kb/experiments/104-扫描重建的现行版本判定.md:76`），而 E104 的回退世界定义是「写回退行 (r_old, T_old, ∞)」（`.claude/kb/experiments/104-扫描重建的现行版本判定.md:31`）——「不写回退行」那一臂**就是今天的设计**：「不新增实例、不取号、不写实例表行」（`.claude/kb/decisions/23-journal的角色与格式.md:376`），不是一个可以不再承重的旁支；2026-09-26 删回退行的依据只答了「回退实例永远不给自己写回退行、截断走不到是结构事实」（`.claude/kb/decisions-history.md:8036`），没答扫描重建怎么认挂载根是一次回退；已发布谓词对同实例只问诞生代号：「i == i_now ⇒ b ≤ 挂载根的 txg」（`.claude/kb/decisions/18-块里携带什么信息.md:309`），所以被回退掉的那版（诞生代号 ≤ 回退那次的 txg）照样是已发布；攻方在副本上量到同身份三版、择新键选中被回退掉的那版：「p3_newest_birth=5 p3_newest_is_referenced_by_d=false d_references_birth=Some(3)」（`research/prompts/m3-tomb-r1-opus-model/run-e-lines.txt:5`） | 不分辨：墓碑写不写、回退重写还是谱系记录，扫描重建都对「挂着时的回退」没有输入；用户已按这个读法定 K-W5b（`research/prompts/m3-tomb-r1-forks.md:10`）。够得着吗：这一击打的是「那一半不再承重」这句话，不是扫描重建本身；`crates/` 里没有扫描重建，两边的扫描规则都是按 C113 定案 P2 / P3 字面写的（墓碑第一轮判决第一节），所以「复活」是规则字面的后果，不是实现里量到的。推翻：D18 已定项 11 另有一句定义让「对该根可见」排除被回退抛弃的单元（我只读到上面那一句，没找到） |
| 墓碑 W1 载体：一棵专用墓碑树，根恒码 2，孩子是码 3 容器，容器与根进中央映射 | 站得住；S2 上两处不用改代码：墓碑树随现行那一版（回退只换 extent、inode 两种树的根），`deleted_inodes` 要加进那一支 | 回退换根的条件：「root: if entry.kind == TREE_KIND_EXTENT」（`crates/singlefs-core/src/mount.rs:4972`）接着是 inode 那一种，别的种类取现行根，墓碑树落在 else 支；inode 叶容器今天就进映射：「mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),」（`crates/singlefs-core/src/transaction.rs:6409`），所以「唯一持有者」那句要改（D18 已定项 11，`.claude/kb/decisions/18-块里携带什么信息.md:294`）；D 出局（住根记录）：根记录 457 + 86 = 543 > 512，「pub const ROOT_RECORD_BYTES: u64 = 457;」（`crates/singlefs-format/src/lib.rs:188`）、「if u64::from(geometry.physical_block_size) < ROOT_RECORD_BYTES {」（`crates/singlefs-core/src/make_filesystem.rs:313`） | 不分辨（D 出局是格式常量撞墙，对每个臂一样）。S2 删掉了「孩子类型 3 退役」，谱系容器住哪归 G4。推翻：第一版要求 4 KiB 扇区（根槽 ≥ 543） |
| 墓碑 W2 字段表 40 字节（加死亡 txg）、815 条一容器 | 站得住；「死亡 txg = 意图那次发布的 checkpoint_txg」在崩溃重放下成立（推的）：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根，txg 取记录头里的 checkpoint_txg | 算的：`ring_window_cells.py` 的 `C name=tombstone_capacity record_bytes=40 per_container=815`（`research/prompts/m3-delete-r3-sonnet-model/cells.out:67`）；重建根的条款：「施加一条记录 = 把所选根的这四个字段换成记录里的」（`.claude/kb/decisions/23-journal的角色与格式.md:443`），记录水位按 (实例代号, checkpoint_txg) 比（同一份 D23 已定项 14 的前缀口径） | 不分辨（死亡 txg 只服务谱系臂；回退重写臂不要它）。推翻：恢复重建的根换了 txg；跨实例重发同一个 txg 时 8 字节分不开，靠容器版本的实例判（P2），表读不出时要 4 字节实例代号（攻方第一轮提的，零轮） |
| 墓碑 W3 第一版不回收；统计量 13、14 不登记 | 站得住；S2 上多一条支撑：墓碑永久，所以每活对象预付一份永久份额才讲得通（第三节 3.6） | 条款：「有过一次推进之后才随每次发布重写」（`.claude/kb/decisions/05-快照-空间记账机制.md:134`）；`grep -rn '清扫水位' crates --include=*.rs` 在冻结副本输出 0 行；增长：10⁶ 次 unlink 算得 1227 个容器、80 412 672 字节（76.69 MiB，两份）（`research/prompts/m3-delete-r3-sonnet-model/cells.out:68`） | 不分辨（放宽回收门会把对象判活，攻方 5.1 节反例对每个臂一样）。推翻：第一版里有人读 13 / 14，或清扫落地 |
| 墓碑 W4 开放容器有新记录才重写，满了才关 | 站得住；S2 上它是「不删除的发布不重写墓碑树」这个**计划形状**，现在没有会红的检查，要钉（第四节 ⑤） | 条款与账：「1000 文件、每 10 个 fsync 一次 ⇒ 100 × 32 KiB × 2 ≈ 6.25 MiB」数的是带删除的发布（`.claude/kb/checks-owed.md:100`）；照抄机制：「fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {」（`crates/singlefs-core/src/transaction.rs:5435`）；同类先例（计划形状钉成断言）：「E name=v6 op=writable_mount row_publish_rewrites_inode_leaf=false」（`research/prompts/m3-delete-r2-opus-model/run-final-e-lines.txt:251`） | 不分辨。推翻：D18 作者的原意是每次发布都重写（那时每次发布 +2 单元、闭式 ×4） |

同类先例（第二节各行）：V1 ③、④ 把幂等落成「逻辑内容相等」「标签等于镜像现算」的已落地检查，找不到（I-3.4、I-4.7 在 `.claude/kb/invariants.md:139`、`:172` 都是「未实现」）；V5 零条目根的先例：`crates/` 里没有任何代码写出空根（`grep` 不到），最近的是三棵 day-1 零根指针的树（livelist、稀疏旁表、deadlist；deadlist 的种类码：「pub const TREE_KIND_DEADLIST: u16 = 8;」（`crates/singlefs-core/src/records.rs:267`））；D23:417 与 W1 到 W4：这一轮没有找到更早的同类复核。


## 三、G1　环窗口落成式子（零轮：这一节的形态都是我提的，只在副本原型上量过）

### 3.1 式子

```
L     = 3 × root_ring_slots_per_region            环里的根数；系统不可变配置，4..16 → L = 12..48，mkfs 默认 8 → 24
K     = 144                                         产品批大小（常量，测试旋钮只压分批、不改这个）
N_ℓ   = 分配记录树第 ℓ 层两块盘合计罩到的位置数      层 0 是叶（812 槽一片），内部节点扇出 169；ℓ 取 0 … R−1，R 是根层级
W_ub  = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))               一次删除类发布每盘新占槽数的上界（槽）；24 = 普通分配 16 + 固定点根 4 + 映射树根以下 4
Ω     = (L + 1) × W_ub                              环窗口（每盘槽）
标签 4 = 0
标签 6 = 副本数 × ( (Ω + 3) × 16384 + 41 × n_live )  字节，按全部副本之和记；n_live = nlink ≥ 1 的 inode 记录数；3 = 一个开放墓碑容器 + 根的整容器余量
         树表里没有 deleted_inodes 条目的版本（第一个文件版本之前）标签 6 = 0
```

- 「环里的根数」读的字段：「pub root_ring_slots_per_region: RootRingSlotsPerRegion,」（`crates/singlefs-core/src/system_configuration.rs:150`），乘三个区「pub const ROOT_RING_REGIONS: u64 = 3;」（`crates/singlefs-format/src/lib.rs:238`）；取值区间「pub const ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = 4;」（`crates/singlefs-format/src/lib.rs:247`）到「pub const ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16;」（`crates/singlefs-format/src/lib.rs:252`）。它在系统不可变配置里：「都属于系统不可变配置——改它们要重建文件系统。」（`crates/singlefs-core/src/system_configuration.rs:136`），所以「环长随每区槽数变」在一个池的寿命里不发生，标签 6 不会因它漂。
- 回收的时机就是窗口的来源：释放代 ≤ 下界才回收，「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」（`crates/singlefs-core/src/mount.rs:1158`），发布里在取完落点之后才记根、回收（`crates/singlefs-core/src/allocator.rs:1655`）。一次释放要等 L − 1 次发布之后才还回，所以连续 L 次删除发布最坏每次新占 W_ub 而一槽都没还回；亏空 ≤ L × W_ub（引理：释放 ≥ 新写时，算的，2000 条随机序列最坏取到上界的 0.791；`research/prompts/m3-delete-r3-sonnet-model/cells.out:62`）。用户定案里的「+ 1」多出一格 W_ub（368 槽 / 盘）的余量，我不砍它。
- 引理的前提「释放 ≥ 新写」被**永久增长**破坏：意图那次的墓碑容器与根是新增（没有被换下的前身），`deleted_inodes` 根同理。这部分不进窗口、进每对象预付（3.6）。16 槽 / 100 次、每次少还 4 槽的序列亏空线性涨（`research/prompts/m3-delete-r3-sonnet-model/cells.out:63`）。
- 单位：「BytesOnOneDevice(self.0.div_ceil(replicas.replicas()))」（`crates/singlefs-core/src/admission.rs:99`）：每盘扣 ⌈和 ÷ 副本数⌉；第一版每个单元落每块盘，所以和 = 每盘 × 副本数。量过：4 GiB × 2 盘、一个活对象，标签 6 = 301 563 986 = 2 × (9 203 × 16 384 + 41)（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:1`）。

### 3.2 W_ub 对量到的每批新占槽数

一单元一叶的造法（把每块盘的空闲槽标成已分配、只留相隔 816 槽的空洞，数据单元各落一片叶；只动分配器、盘上不写），K = 144 一批新占 307 到 312 槽，其中分配记录树节点 297 到 299 个。式子那一判关、落点那一判开。

| 盘 | 文件单元数 | 批大小 K | 量到的最大一批新占（槽） | W_ub | 余量 | 出处（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt`） |
|---|---|---|---|---|---|---|
| 4 GiB | 16 | 16 | 46 | 112 | 66 | :19 |
| 4 GiB | 64 | 64 | 144 | 208 | 64 | :20 |
| 4 GiB | 144 | 144 | 307 | 368 | 61 | :21 |
| 4 GiB | 240 | 144 | 310 | 368 | 58 | :22 |
| 4 GiB | 144 | 16 | 52 | 112 | 60 | :23 |
| 4 GiB | 144 | 1 | 26 | 82 | 56 | :24 |
| 16 GiB | 144 | 144 | 309 | 380 | 71 | :25 |
| 16 GiB | 240 | 144 | 312 | 380 | 68 | :26 |

- 连续落点时这一批只新占 12 到 24 槽（删除第二轮 `E name=w_state` 与 `w_row`），W_ub 是它的 15 到 30 倍；散落造法逼到 81% 到 84%。第二轮辩方给的字面最坏 `2×min(K+26,262)+5` 在 K = 144 上是 345，量到 307 到 312 已经贴近，所以我取 2·(K + 26) 一层加四个固定点根与映射树的 4，没有再收紧。
- W_ub 只随批大小 K 与分配记录树的层数、每层位置数走：与文件单元数 N 无关（K 封顶，N = 1 000 时一批也只动 K 个单元）、与 extent 树形无关（量到的普通分配只有 3 到 5 槽，`ordinary=[3, 5, 4]` 在 `run-final-r-lines.txt:22`；一批最多动 3 片下段叶加根，含在 16 里）、与「写完之后用户又做了什么」无关（删除第二轮攻方的 `E name=w_state` 八种状态里变的只有固定点，上界取分配记录树整层 min，罩得住）。
- 没验的：(i) 分配记录树层数 ≥ 3 的盘（模型算：约 256 GiB 起，各层位置数 [41202, 246, 2]）上逐层取 min 的上界没造出对应的散落世界，推的；(ii) 映射树根以下取 4 是量到 0 到 2 之后推的；今天写入口造不出「一个文件的单元分属多次发布」（`publish_overwrite` 只收一个数据单元的文件，顺序写整个文件重写），映射 key 按（类、树、诞生 txg、实例、序号）排，一个文件的单元在映射里挨着；局部覆写进来之后这一项要重估；(iii) 意图那次的 +4 槽是按角色表另加的，原型没写墓碑。


### 3.3 3 单元文件、K = 1 删除：每次发布之后的标签 4、6 与每盘可用（逐格值）

4 GiB × 2 盘、2 副本、每槽 16 384 字节；标签是全部副本之和的字节，可用是每盘字节。窗口账 Ω = 9 200 槽 / 盘（K = 144、L = 24）。「每盘新占」是原型量到的，S2 另加按角色表算：意图那次 +4（`deleted_inodes` 根 1、墓碑容器 2、墓碑树根 1），最后一批 −2（inode 叶容器不再写；`deleted_inodes` 取根指针置零，E-b，不写根）。可用 = 原型量到的真读数（账房带标签）再扣 S2 另加的槽（算的）。

**表 A：名义池**（空 4 GiB 池上写这一个文件；删除各批不判式子那一判）

| 时刻 | 每盘新占槽 | 标签 4 | 标签 6 | 每盘可用 | R3 逐文件对照（标签 4 / 标签 6 / 每盘可用） |
|---|---|---|---|---|---|
| mkfs 与暖机（树表 0 条） | — | 0 | 0 | — | — |
| 第一个文件版本（1 单元） | 14 | 0 | 301 563 986 | 3 207 315 415 | — |
| 顺序写成 3 单元之后 | 19 | 0 | 301 563 986 | 3 207 004 119 | 0 / 1 638 400 / 3 356 966 912 |
| 意图之后 | 11 + 4 = 15 | 0 | 301 563 904 | 3 206 758 400 | 1 179 648 / 0 / 3 356 966 912 |
| 批 1（留 2）之后 | 13 | 0 | 301 563 904 | 3 206 545 408 | 753 664 / 0 / 3 356 966 912 |
| 批 2（留 1）之后 | 12 | 0 | 301 563 904 | 3 206 348 800 | 360 448 / 0 / 3 356 966 912 |
| 批 3（留 0，删记录）之后 | 12 − 2 = 10 | 0 | 301 563 904 | 3 206 184 960 | 0 / 0 / 3 356 966 912 |

出处：窗口列 `research/prompts/m3-delete-r3-sonnet-model/cells.out:26-32`（原样行 `run-final-r-lines.txt:2-8`）；第一个文件与顺序写的新占槽 14、19 取自辩方第二轮 `research/prompts/m3-delete-r2-sonnet-model/run-final-d-lines.txt:12-13`，那份的意图与各批 11 / 13 / 12 / 12（`:14-17`）与我这一轮的 `new_slots` 逐个相同；R3 列 `research/prompts/m3-delete-r2-sonnet-model/cells.out:2-6`（辩方第二轮策略 X）。两点：① 标签 6 在四次删除发布里恒为 301 563 904，意图那次比写入时少的 82 字节就是该对象的墓碑预付（2 × 41）转成了真分配；② 窗口账下可用在删除各批里**下降**（每次降这一批的新占槽），因为被换下的槽与被删的数据单元在 defer 里仍计「已分配」，要等环转完才还；R3 把这几笔提前扣了，可用恒定。代价对比：这个池上一个 3 单元文件，R3 常驻 50 槽（819 200 字节），窗口 9 203 槽（150 781 993 字节）。

**表 B：写完之后可用恰为 0 的池**（窗口在账里，池填到可用 = 0；`run-final-r-lines.txt:9-11`）

| 时刻 | 每盘可用（字节，S2） | 窗口剩余（空闲槽 − 保留量 6 989，S2） | 式子那一判开（朴素） | 式子那一判关、落点那一判开 |
|---|---|---|---|---|
| 删除之前 | 16 343 | 9 204 | — | — |
| 意图之后 | −229 376 | 9 189 | 拒：「(16384, 180224)」可用 16 384 字节对需求 11 槽 | 放行 |
| 批 1 之后 | −442 368 | 9 176 | （前一次已拒，没有往下） | 放行 |
| 批 2 之后 | −638 976 | 9 164 | — | 放行 |
| 批 3 之后 | −802 816 | 9 154 | — | 放行 |

R3 对照：同一个池上写入时预付，紧池表里四次发布朴素都拒、抵扣都放行、发布后可用都是 0（`research/prompts/m3-delete-r2-sonnet-model/cells.out:13-17`）。窗口账下可用降到负数是对的：窗口那 9 200 槽就是留给这几笔暂态的，保留量一槽没动（窗口剩余全程 ≥ 9 154）。

### 3.4 窗口多大；保留量已经罩住的那部分

「保留量」= 13 × ckpt_cost + 8（挂载期承诺量 + checkpoint 保留池，每盘槽）：4 GiB 上量到 6 989（`run-final-r-lines.txt:9` 的 `reserve_slots_dev0=6989`）；ckpt_cost 4 GiB 量到 537 到 543（`run-final-r-lines.txt:19-22`），16 GiB 量到 2 504 到 2 508（`:25-26`），模型取 537 / 2 487，别的盘大小是推的。落点那一判要「普通 + ckpt_cost」空闲，所以删除各批能借保留量的是 12 × ckpt_cost + 8 − 普通槽数。最后一列 = max(0, L × W_ub + 16 − (12c + 8))：窗口之外保留量罩不住的亏空。

| 盘（单元区槽 / 盘） | L | 保留量 | Ω | Ω ÷ 单元区 | Ω ÷ 保留量 | 保留量罩不住的亏空 |
|---|---|---|---|---|---|---|
| 3 GiB（146 432，默认 journal 环 768 MiB 的最小盘） | 24 | 4 909 | 9 200 | 6.28% | 1.87 | 4 316 |
| 4 GiB（211 968） | 12 | 6 989 | 4 784 | 2.26% | 0.68 | 0 |
| 4 GiB | 24 | 6 989 | 9 200 | 4.34% | 1.32 | 2 396 |
| 4 GiB | 48 | 6 989 | 18 032 | 8.51% | 2.58 | 11 228 |
| 8 GiB | 24 / 48 | 15 439 | 9 300 / 18 228 | 1.96% / 3.84% | 0.60 / 1.18 | 0 / 3 620 |
| 16 GiB | 24 / 48 | 32 339 | 9 500 / 18 620 | 0.95% / 1.86% | 0.29 / 0.58 | 0 / 0 |
| 64 GiB | 24 | 133 661 | 10 650 | 0.26% | 0.08 | 0 |
| 1 TiB | 24 | 2 160 179 | 17 750 | 0.03% | 0.01 | 0 |

出处：`research/prompts/m3-delete-r3-sonnet-model/cells.out:1-21`（算的；4 GiB 的 ckpt_cost 与 W 量过，其余盘大小的 ckpt_cost、层数是推的）。最小盘 3 GiB 来自：「if ring_bytes > smallest / 4 {」（`crates/singlefs-core/src/make_filesystem.rs:346`）与「pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;」（`crates/singlefs-format/src/lib.rs:213`），自定 journal 环的小盘我没算（那是另一个同批实验的格）。读法：

- 窗口的量与盘大小几乎无关（W_ub 只随分配记录树层数缓慢涨），保留量随盘线性涨；所以窗口在小盘上最贵（4 GiB 的 4.34%）、在大盘上可忽略（1 TiB 的 0.03%）；L 从 24 到 48，窗口翻倍。
- 保留量自己能罩住最坏亏空的盘：L = 12 在最小盘上就罩得住，L = 24 约 5.3 GiB 起，L = 48 约 9.9 GiB 起（`research/prompts/m3-delete-r3-sonnet-model/cells.out:74-76`，按分配记录树叶数线性外推、ckpt_cost 取模型值，推的）；在那些盘上窗口的真实作用只剩「删除不吃保留量」。量过的对照：窗口不在账里时，K = 1 删 100 / 200 单元借走保留量 258 / 304 槽（L = 24）、712 槽（L = 48）；窗口在账里时同一段历史空闲槽比保留量多 8 946 / 8 900 / 17 324 槽，保留量没动（`run-final-r-lines.txt:12-17`，三个顶点与删除第二轮攻方 `research/prompts/m3-delete-r2-opus-model/run-final-e-lines.txt:1` 的 `max_borrowed_slots=258` 同数）。
- 窗口随 K 线性涨（4 GiB、L = 24）：K = 1 / 16 / 48 / 144 的 Ω = 2 050 / 2 800 / 4 400 / 9 200 槽，占单元区 0.97% / 1.32% / 2.08% / 4.34%（`cells.out:22-25`）。K 是用户定的自由选取（144），这一列只说明它的价。


### 3.5 替用户推翻的 R3 逐文件预付：它在 G1 的哪一格上比环窗口强

只给数，不替主 agent 判。窗口取 4 GiB、L = 24、K = 144（Ω = 9 200 槽 / 盘）；R3 的每文件槽数是意图加各批（量过，S2 另加）。

| 格 | 谁强 | 数 | 出处 |
|---|---|---|---|
| 池里文件少时的常驻成本 | **R3 强** | R3 在文件数小于下面这几个数时比窗口省：3 单元 K = 1（50 槽 / 个）184 个；300 单元连续（61 槽）151 个；144 单元一单元一叶、K = 144（325 槽，量到）29 个；size 0 的空 inode（22 槽，攻方代用量）419 个。没有文件的池 R3 常驻 0，窗口从第一个文件版本起就是 4.34% | `research/prompts/m3-delete-r3-sonnet-model/cells.out:42-45`（算的） |
| 崩在删除中间、池近满、重挂载 | **R3 强** | R3 下可用恒 0，挂载前那一判不拒；窗口账下可用降到 −229 376 … −802 816 字节（表 B），判 Short，写行之后推抬 F 的空发布（回退候选缩到最近 4 个状态）。读代码：拒的条件「(available.0 < i128::from(demand.0)).then_some(DeviceShortOfDemand {」（`crates/singlefs-core/src/admission.rs:447`），带文件的上一版走「PreviousVersion::WithFile { .. } => {」（`crates/singlefs-core/src/mount.rs:3747`）进推抬 F，树表 0 条的才直接拒（`:3752`）。推的：没造真满盘的重挂载（满池是分配器拷贝上标的，不能重挂） | 零轮修法：队列非空时，挂载前那一判把窗口算作可动用（可用 + Ω × 16384 ≥ 0），续做自己要的就是它 |
| 与环长无关 | R3 强 | R3 每文件的量只随 ⌈N ÷ K⌉ × W；窗口随 L 线性涨，4 GiB 上 L = 48 占单元区 8.51% | 3.4 表 |
| 池里文件多、空 inode 多 | **窗口强** | R3 的空 inode 每个 22 槽，4 GiB 单元区只装得下 9 634 个可删的空 inode（211 968 ÷ 22，代用量；删除第二轮 `E name=empty_inode_prepay`，`research/prompts/m3-delete-r2-opus-model/run-final-e-lines.txt:13`）；窗口每对象只收 41 字节 | `cells.out:46` |
| 回退到更大的文件版本 | **窗口强** | 量过：两版活对象数同为 1，标签 6 回退前后同值 301 563 986，回退之后每盘可用 −606 249 字节，全是回退那次自己的新占（38 槽，窗口兜）；R3 下同一段历史 −1 212 416，其中 R3 抬的那份每盘 589 824 | 「label6_row_before=301563986 rollback=Ok units_after=300 label6_row_after=301563986」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:18`）；R3 版：「available_after=[-1212416, -1212416]」（`research/prompts/m3-delete-r2-opus-model/run-final-e-lines.txt:15`） |
| checker 从镜像重算 | **窗口强** | 窗口只依赖盘几何、系统配置、产品常量、活对象数；R3 逐文件要取纯函数上界，是量到值的 6 到 25 倍（K = 144 连续落点 W 约 15 到 18 对 W_ub 368，K = 1 的 13 对 82）；用户定案里「再大 5 到 20 倍」成立 | 算的：368 ÷ 15 = 24.5，368 ÷ 18 = 20.4，82 ÷ 13 = 6.3 |
| 满池删得动 / 删除不吃保留量 | 平手 | 窗口：表 B 全过、保留量一槽没动；R3：紧池表抵扣放行 | 3.3、3.4 |

### 3.6 题面其余问句，逐条答

1. **标签 4 记什么**：恒 0（零轮）。三个候选：(a) 恒 0；(b) 记删除中 inode 还没释放的 extent 字节（口径 A）——这些单元在被删前、释放后都计在「已分配」里（释放只进 defer：「self.deferred_slots += span;」，`crates/singlefs-core/src/allocator.rs:556`），再扣一次就是重复扣，第一轮已出局，S2 上同样；(c) 队列非空时把窗口从标签 6 挪到标签 4——可用不变，多一条重算规则，只让 I-3.4 不空洞，没有新保证。取 (a)。I-3.4 的字面「可用空间统计已扣除」（`.claude/kb/invariants.md:139`）靠「已分配」满足，不靠标签 4；D5 已定项 20 写明待删除结构算活引用（`.claude/kb/decisions/05-快照-空间记账机制.md:425`）。要改的条款：D5 统计量表第 4 行（「| 4 | 待删除但意图未完成占用 | 准入（I-3.4（可用空间扣待删占用）） | 不带（全池承诺量） | 不带 |」，`.claude/kb/decisions/05-快照-空间记账机制.md:118`）与 D28 已定项 1 式子里的待删占用项改成「恒 0 的保留位」，I-3.4 改写成「待删中 inode 的未释放单元仍计在已分配里，标签 4 = 0」。
2. **意图那次怎么转、各批怎么抵**：意图那次标签 6 少 2 × 41 字节（该对象的墓碑预付转成真分配），标签 4 不动；各批不抵（没有预付可抵），空间由窗口兜。写入类发布（建文件、建 inode、第一个文件版本）**按发布之后的标签判**：放行 ⟺ 需求 ≤ 可用（发布前）−（标签 6 之后 − 标签 6 之前）。
3. **式子那一判与落点那一判**：删除类发布（意图、各批、挂载里的续做批）式子那一判**关**（`publish_has_an_ordinary_allocation` 判成真也不判，要在 `crates/singlefs-core/src/transaction.rs:5131` 那处加这一类的例外）、落点那一判**开**；D28 已定项 1 接线那一条要加一句：「判不判照旧：没有普通分配的发布（空发布、写行、暖机、抬 F）不判」（`.claude/kb/decisions/28-挂载期承诺量.md:34`）之外，删除类发布也不判。依据：D3 已定项 17「凡是释放路径要写的字节（墓碑单元、livelist 的 FREE 条目），空间在写入那一刻就按已定项 15 的准入控制预付掉（已定项 2）。」（`.claude/kb/decisions/03-空间分配.md:304`）——预付落成窗口，释放路径就不再申请。
4. **窗口兜不住时**：落点那一判拒，会话照「PublishError::PlacementRefused { .. } => true,」（`crates/singlefs-core/src/mounted_session.rs:149`）推抬 F 的空发布再判；抬到上限 = min(每块盘最新的有效根, 第 4 新的不同状态的非空有效根)（`crates/singlefs-core/src/mount.rs:1933`），回退深度缩到 4。窗口的作用是不走到这一步，不是没有这一步。
5. **「(环里根数 + 1)」**：读 `root_ring_slots_per_region` × 3（3.1）；一个池寿命里不变。
6. **单位**：全部副本之和记，每盘扣 ⌈和 ÷ 副本数⌉（3.1）。
7. **建 inode 的墓碑预付，按记录还是按整容器**：**按记录（每活对象 41 字节）加全池一份 3 槽整容器余量**，不是每对象一整容器。势函数：m 个墓碑之后永久占 2⌈m ÷ 815⌉ 槽容器 + 1 槽根（每 135 个容器多一个内部节点）；还有 n 个活对象要死时最多再长 2⌈n ÷ 815⌉ + 3 槽 ≤ 2n ÷ 815 + 3 槽 ⇒ 每对象 ⌈2 × (1 + 1/135) × 16 384 ÷ 815⌉ = 41 字节，加常量 3 槽。只按记录摊不够写第一个容器（n < 815 时摊得不到一个容器，墓碑第一轮 W6 那条）；每对象一整容器 3 槽，1 000 个对象就是 3 000 槽（46.9 MiB / 盘）。暂态（每次发布重写开放容器、旧版进 defer）在窗口里，不在这一项。算的，没量（原型没有墓碑）。
8. **窗口从哪一次发布起计**：第一个文件版本（树表 0 → 9 条，`publish_first_file`，`crates/singlefs-core/src/transaction.rs:4624` 起）。之前的版本（mkfs、暖机、第一次挂载）标签 6 = 0，字节不变；第一个文件版本按发布之后的标签判，一个放不下窗口的池在这里拒写第一个文件（ENOSPC，不是挂载失败）。
9. **checker 只从镜像重算**：能。输入：系统配置 `root_ring_slots_per_region`、各盘单元区槽数、格式常量（K = 144：「pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;」，`crates/singlefs-format/src/lib.rs:156`；叶 812、扇出 169，`:140`、`:146`）、n_live（inode 树按叶走一遍，checker 本来就逐叶走：「self.judgements.judge("I-9.13", !records.is_empty(), \|\| {」，`crates/singlefs-checker/src/walk.rs:1596`）、树表里有没有 `deleted_inodes` 条目。不需要 K 旋钮。常量要跟统计量登记表一起冻结（「统计量登记表立成独立冻结组件」，`.claude/kb/decisions/08-核心索引结构.md:173`），否则换常量之后候选集里旧根的行与新代码的重算对不上。审计与被审计不许同一段代码：checker 另写一份 W_ub 与 n_live。
10. **窗口会不会被别的发布吃掉**（推的）：写入类发布按发布之后的标签判，窗口已在可用里扣掉，吃不到；空发布、暖机、抬 F 的固定点走保留池，不走窗口；只有删除类与回退吃窗口。没有对「写入类发布按发布之后的标签判」跑过用例（原型的判在发布前的读数上）。

同类先例：全池一份、不随对象长的常驻预留，在仓里找不到（`grep -rli slop .claude/kb` 无输出）。最近的形状是 checkpoint 保留池：全池量、「每次发布重算、不设常数上限、只住内存」（`.claude/kb/decisions/28-挂载期承诺量.md:26`）；标签 6 比它多一步，要落进记账行、从版本重算。

### 3.7 什么现象会推翻 G1

- 入库装置上 K = 144 的某一批新占超过 W_ub（4 GiB 为 368），或层数 ≥ 3 的盘上超过逐层 min 之和 ⇒ 式子不收。
- 某个（盘大小、L）上窗口小于量到的借保留量顶点 ⇒ 不收（我量了 4 GiB 的三个点，顶点 258 / 304 / 712 都小于窗口；小盘长环那一格由同批实验量）。
- 某个删除类发布释放的槽少于它新占的（除意图那次的墓碑与 `deleted_inodes` 根）⇒ 引理不成立，亏空线性涨。
- 入库装置上写入类发布按发布之后的标签判之后，仍有写入类发布吃进窗口 ⇒ 3.6 第 10 条不成立。
- 真满盘上崩在删除中间的重挂载不推抬 F、或推了也读不到删之前的候选 ⇒ 3.5 第二行的判断要改。


## 四、G2　五条幂等判据够不够（删除第二轮 V1 的复核）

### 4.1 逐条在 S2 上判得住什么

| 判据 | 判得住 | 判不住 | S2 上的状态 |
|---|---|---|---|
| ① 计划相等 | 计划依赖隐藏状态 | 计划之外的副作用，记账行的值 | 站得住（弱） |
| ①b 同镜像重挂载续做逐字节相同 | 续做里引入时钟、随机、无序迭代 | 同一份错写被两边一起写出来 | 站得住；样本 1 → 6，全部相同（第二节） |
| ② 删完再续做一个写都不发 | 完成态上还在写 | 中间态上的重复施加 | 站得住，前提是第三成员（第五节） |
| ③ 终态逻辑内容相同 + 每个崩溃态 checker 全绿 | 内容级的重复与错位 | 终态会抹平的量；两边共用的写路径错；不改内容的写放大 | 站得住，逻辑内容要按 4.3 改 |
| ④ 每个被判状态的标签 4、6 等于镜像现算值 | 记账行写错 | 它原来的靶子（待删占用减两次 / 不减）在 S2 里没有 | 前提成立（3.6 第 9 条）；靶子换成墓碑份额与窗口行 |

### 4.2 找「五条都过」的非幂等续做：七个形状

「过」= ①–④、①b、② 全部不红。

| # | 形状 | 五条过不过 | 判 |
|---|---|---|---|
| 1 | 续做路径多写一条墓碑（续做重走了意图那一步） | **不过**：终态墓碑多重集 2 ≠ 不崩溃的 1（③） | 抓得住 |
| 2 | 意图与续做共用的写路径把同一对象的墓碑写两条，两条落在**不同容器** | **过**：③ 两边都是 2；墓碑第一轮的 N4 只判「同一容器内 (号, 出生代) 不重复」 | **漏**。补：checker 加「谱系上每个 (号, 出生代) 至多一条墓碑」（零轮）；删 → 回退 → 再删合法产生的两条在谱系过滤之后只剩一条，不误红 |
| 3 | 开放容器在续做里被重写两次：同一发布内同身份两个落点 | 落点按身份入表：「let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();」（`crates/singlefs-core/src/transaction.rs:6108`），第二次分配会盖掉表项、第一个落点留成已分配而没有根引用，I-3.11 红（推的）；要「过」得让第一个落点同时进 defer，现有代码没有这条路 | 抓得住（I-3.11 已实现）；分成两次发布重写见形状 4 |
| 4 | 非删除的批（只改 extent 与 inode 的批）也重写墓碑树 | **过**：逻辑内容不变、标签 4、6 不变；中间版本 checker 全绿——被换下的容器进 defer，I-3.11（已分配减 defer 等于最新根走读，`.claude/kb/invariants.md:144`）不红 | **漏**，但是写放大（每次发布 +2 单元、闭式 ×4），不是逻辑非幂等。补：⑤ 每种发布的重写角色集合钉绝对值（下） |
| 5 | 续做提前删 `deleted_inodes` 的 key（最后一批之前） | 逐态：I-4.7「要么在删除中队列里有条目，要么其 extent 已全部释放」（`.claude/kb/invariants.md:172`）红 | 抓得住，**前提是 I-4.7 落地**（今天「未实现」，三·3 要实现的就是它） |
| 6 | 续做把 size 写回、不跟着截；或把 nlink 写回 1 | 恢复拒：`crates/singlefs-core/src/recovery.rs:3672` 的 `data_unit_count_matches_the_inode_size` 对每个崩溃态都判；nlink 写回由读者入口拒 nlink = 0 与 `parse` 读 nlink 兜 | 抓得住（量过：删除第二轮 `E name=v6 op=publish_new_inodes_1 rewrites_inode_leaf=true raw_nlink_after=1 units_after=4`，`research/prompts/m3-delete-r2-opus-model/run-final-e-lines.txt:252`） |
| 7 | 续做把墓碑的死亡 txg 换成续做那次发布的 txg（替换、不是追加） | 多重集若不含死亡 txg 则**过**；含死亡 txg 则不过 | 所以逻辑内容必须含死亡 txg（4.3） |

⑤（零轮，harness 层，不是 checker 层，因为镜像上看不到写了几次）：意图那次发布的重写角色 = 固定点 + inode 叶与根 + `deleted_inodes` 根 + 墓碑容器 + 墓碑树根，不含 extent；各批的重写角色 = 固定点 + extent 节点 + inode 叶与根，**不含**墓碑树任何单元与 `deleted_inodes` 根（最后一批除外：根指针置零）。每种钉绝对值，在崩溃流里每次发布核。先例：计划形状钉成断言的形式在删除第二轮已有（「E name=v6 op=writable_mount row_publish_rewrites_inode_leaf=false」，`research/prompts/m3-delete-r2-opus-model/run-final-e-lines.txt:251`）；同类先例找不到更早的。

### 4.3 「逻辑内容」该不该含墓碑的死亡 txg、谱系记录

- **含死亡 txg**：它是语义字段（谱系过滤靠它判这条墓碑在不在谱系上），不含就逃过形状 7。两条路径相等：死亡 txg = 意图那次发布的 checkpoint_txg，崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根，txg 取记录头里的（推的）；崩在意图发布里、恢复落到意图之前的状态，终态取多版本 oracle（第二节 V1 ③ 行），不比死亡 txg。
- **含谱系记录**（多重集）：删除流里恒空，不增加比对负担；删除中间回退再续做的历史里它必须在，否则「续做多写一条谱系记录」逃过。
- **仍不含**：实例号、除死亡 txg 之外的 txg、改动计数、落点、单元头的诞生代号与写序、逐盘标签 1 / 2 / 5 / 10 / 11（回收时机随环变）、容器头里的 txg 与写序。
- 标签 12 / 4 / 6：含。标签 4 恒 0，标签 6 在删除流里只在意图那次变 82 字节。

### 4.4 同类先例，与什么现象会推翻

- 先例：D8 已定项 5 第 4 条把幂等写成「一批里每个动作施加时先核它要删的东西还在」（`.claude/kb/decisions/08-核心索引结构.md:124`），代码里已兑现：「return Err(PublishError::ReleaseTargetAlreadyReleased {」（`crates/singlefs-core/src/transaction.rs:3220`）。把「幂等」落成「逻辑内容相等」的已落地检查：找不到。
- 推翻 4.2 的「漏」：入库装置上形状 2 或形状 4 被现有检查抓到（那时它们不是漏）；推翻「站得住」：找到形状 8，五条加 ⑤ 与新增的谱系上唯一性都过而逻辑内容不同。
- 没做：S2 的续做写路径没实现，4.2 的七个形状是按判据字面与代码构造推的，只有 ①b 在原型上量过。


## 五、G5　删空与恢复（删除第二轮 V5 的零轮项）

结论：三棵树的空形态在**今天**的读者里，冷启动只读挂载与池级 checker 认，恢复重建（可写挂载）全拒；认的程度因树而异（下表）。墓碑树不空对第三成员的判定没有影响。全部是读代码，没有造出带空形态的盘（`crates/` 里没有写出空根的代码），逐条标出处。

### 5.1 三棵树的空形态 × 四个读者

| 空形态 | 恢复重建一版（可写挂载） | 冷启动只读挂载 | 池级 checker | I-9.13 |
|---|---|---|---|---|
| inode 树：零条目根（指针非零、层级 1、key 区间 `[0, 0]`） | **拒**：「what: "inode 树根没有条目",」（`crates/singlefs-core/src/recovery.rs:2453`），再往下「what: "inode 树里没有第一个文件那条记录",」（`crates/singlefs-core/src/recovery.rs:2506`） | **认**：根的区间只在有条目时核（`crates/singlefs-core/src/recovery.rs:3253`），内部条目循环零次，`open_file` 对任何 inode 答「.ok_or(OpenFileFailure::NoSuchInode { inode })?;」（`crates/singlefs-core/src/mounted_read.rs:627`）——「没有文件」的答案今天就有 | **认**：只有层级 0 的根被拒「if root.level == 0 {」（`crates/singlefs-checker/src/walk.rs:1411`），层级 1 零条目走零次；区间「区间贴紧首末条目（空节点区间任意）」（`crates/singlefs-checker/src/lib.rs:597`）；I-9.6 报不适用：「judgements.not_applicable("I-9.6", "最新的根下面走不到 inode 树里的任何一条记录")」（`crates/singlefs-checker/src/walk.rs:6153`） | 无可判（没有叶）；kb 自己就这么写：「唯一的叶被删空时是不落盘、根变成零条目」（`.claude/kb/decisions/08-核心索引结构.md:169`）|
| extent 树：零条目根（层级 0、第 0 格，key 区间 = 位置规定 `[(0, 0, 0), (0, 142, u64::MAX)]`，头里自述条目宽 113） | **拒**：「"extent 树节点一条条目都没有（缺席的一段不写节点）",」（`crates/singlefs-core/src/extent_tree.rs:664`）连根一起拒；再往下「what: "extent 树里第一个文件一个数据单元都没有",」（`crates/singlefs-core/src/recovery.rs:3298`） | **认**：打开池时 extent 树只记根指针、不读（`crates/singlefs-core/src/mounted_read.rs:429`），打开文件前先在 inode 记录里找不到就返回 | **认，但头要对**：层级 = 位置规定的层、key 区间 = 位置规定、条目宽 = 113：「view.smallest_key == prescribed_range.0 && view.largest_key == prescribed_range.1,」（`crates/singlefs-checker/src/walk.rs:2025`），零条目不豁免 I-1.1 与 I-1.10 的头判，所以 S2 的「按位置规定」正是 checker 要的 | 不适用 |
| `deleted_inodes`：根指针置零（E-b） | **认**：「if entry.root == NodePointer::empty_root() {」后 `continue`（`crates/singlefs-core/src/recovery.rs:3910`）；要加的只是种类表长度「const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 7] = [」（`crates/singlefs-core/src/recovery.rs:2188`）加到 9 | **认**：没有读者去找它（`entry_of_kind` 只用于 inode 与 extent） | **认，零改动**：引用集合对任何种类的零根都跳过：「if tree_root.all_zero {」（`crates/singlefs-checker/src/walk.rs:2864`），带注释「day-1 注册、还没有根节点的树（livelist 6、稀疏旁表 7、deadlist 8）：不占落点。」（`crates/singlefs-checker/src/walk.rs:2865`）；队列**非空**（根非零）时要给这个种类写走读，否则「TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => references.is_complete = false,」（`crates/singlefs-checker/src/walk.rs:3228`）让 I-3.1 变成「数不全」 | 不适用 |

### 5.2 为什么 inode、extent 只能取零条目根，`deleted_inodes` 才能取零根指针

- inode、extent 若取零根指针：冷启动只读挂载拒：「entry.kind == kind && entry.root != NodePointer::empty_root()」（`crates/singlefs-core/src/mounted_read.rs:414`）、`entry_of_kind(TREE_KIND_EXTENT)?` 与 `entry_of_kind(TREE_KIND_INODE)?` 报 `TreeTableHasNoTreeOfKind`（`crates/singlefs-core/src/mounted_read.rs:429`、`:431`）；恢复重建里 inode 树的根指针取自树表条目，直接拿去读节点（`crates/singlefs-core/src/recovery.rs:2352`），零指针读不出一个 inode 根，推的（没造盘）。checker 对零指针静默跳过（`crates/singlefs-checker/src/walk.rs:800`），所以 checker 不挡，是读者挡。这是对 V5「inode、extent 零条目根；`deleted_inodes` 零根指针」的**读者侧依据**，不是整齐性的取舍。
- `deleted_inodes` 是新种类，没有旧读者要求它非零；空队列在盘上只有一种样子（建树后到第一次 unlink 前、删空之后都是零根）；最后一批少写 1 个单元（F1 的 n 22 → 20，第六节）。先例：livelist、稀疏旁表、deadlist 三棵就是零根指针（`crates/singlefs-core/src/records.rs:267` 登记 deadlist 的种类码）。
- 要改的读者代码（实现清单，读代码的数）：恢复 `rebuild_version` 三处（`recovery.rs:2453`、`:2506`、`:3298`）加第三成员那一支，`extent_tree.rs:664` 对根放行；`PoolVersion` 等三个枚举的每个 `match` 逐处加臂（四处已列：`mounted_session.rs:261`、`mount.rs:2080`、`:3752`、`:4753`）。

### 5.3 墓碑树不空对第三成员判定的影响

- 判定写成：树表有条目（重建的第一个分流：「if tree_table.entries.is_empty() {」之后直接返回 `WithoutFile`，`crates/singlefs-core/src/recovery.rs:2328`；`tree_table_has_no_entries` 是同一个判，`crates/singlefs-core/src/recovery.rs:2107`；第三成员的分流放在读完 inode 根之后）∧ inode 树零条目根 ∧ extent 树零条目根 ∧ `deleted_inodes` 零根 ∧ 水位行在。**不看墓碑树**。
- 墓碑树在最后一个对象删完之后是非空的（第一版不回收，W3），谱系容器同理。判定若写成「全部树为空」，删完的版本永远判不成第三成员，落回「有文件」那一支去撞 `recovery.rs:2453`。
- 反过来，第三成员**不能是一个裸的「空」值**：它要带墓碑树、谱系容器、记账树、映射树的单元，发布路径靠它照抄没重写的角色：「fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {」（`crates/singlefs-core/src/transaction.rs:5435`）。所以改的是 `TransactionOutput` 里文件那两个字段变 `Option`（删除第二轮攻方第七节），不是造一个新的空类型。
- 删完之后能做的写：`publish_new_inodes` 对空树建第一片叶容器今天就支持：「树是空的（新池新建文件那一档）时第一条记录建第一片容器，容器号 = 它自己的 inode 号。」（`crates/singlefs-core/src/inode_tree.rs:162`），号取水位；回退入口只收带文件的现行版本（`mount.rs:5042`），第三成员要放进「带文件」这一类才不会让删完之后回退不回删之前。

### 5.4 什么现象会推翻

- 恢复重建的第三成员那一支做出来之后，三棵空树的某一棵在池级 checker 上红（尤其是 extent 零条目根的头自述宽度不是 113）。
- 墓碑树被允许回收到零条目：那时「不看墓碑树」仍成立，但第三成员判定与「树表有条目」要重核。
- 有人要把 inode 树的零条目根 key 区间改成 `[0, u64::MAX]`：要同时改 D8 已定项 6 与 I-9.1 两处文字。


## 六、G8　两轮判决里报出的代价数，在 S2 上重算或标「推的」

| 报出的数 | 出处 | S2 上的值 | 口径 |
|---|---|---|---|
| F1 状态数 41 943 084（E-c）/ 38 797 356（E-b），未含墓碑树多出的 2 个单元 | 删除第二轮判决 V7 F1 行 | 一次带单元的发布增量是 2^n + 11（n = 2 × 单元数）：「上一次轮换段 3² − 1 = 8 + 单元段 2^n − 1 + 记录段 3 + 根槽段 1 = 2^n + 11」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:102`）。3 单元文件、K = 1 的单元数：意图 10（量过：第二轮 `run-final-d-lines.txt:14` 的角色表）+ `deleted_inodes` 根 1 + 墓碑容器 1 + 墓碑树根 1 = 13；批 1 是 12；批 2 是 11；批 3 是 11（E-c，零条目根）/ 10（E-b，根指针置零）。各发布 67 108 875 + 16 777 227 + 4 194 315 + {4 194 315 \| 1 048 587}，合计 **92 274 732（E-c）/ 89 129 004（E-b）**；接在多记录流后面（全域「层 0 的枚举域 1358954634 个状态」，`crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:21`）：**1 451 229 366 / 1 448 083 638** | 算的；单元数在单文件池上量、S2 另加按角色表；流上没量（流里 A、B、C 之后的分配器状态不同，分配记录树节点数要在流上量一次才能钉） |
| 墓碑记录 815 条一容器 | 墓碑第一轮判决 W2 行 | ⌊(32768 − 136) ÷ 40⌋ = 815（32 字节时 1 019；旧头 91 字节 ÷ 32 是 1 021）；136 = 含 29 字节预留位的头：「声明长度 ≤ 32768 − 136」（`.claude/kb/invariants.md:30`） | 算的（`research/prompts/m3-delete-r3-sonnet-model/cells.out:67`） |
| 10⁶ 次 unlink 的墓碑增长（32 字节 61.4 MiB / 40 字节 76.7 MiB） | 墓碑第一轮判决 W3 行 | 40 字节：1 227 个容器 × 32 768 × 2 份 = 80 412 672 字节（76.69 MiB），加墓碑树节点 10 个内部 + 1 个根 × 16 KiB × 2 = 360 448 字节；每次 unlink 约 80.4 字节（两份）。**但这个量今天走不到**：单层根装 135 个容器 = 110 025 条，再往上要多层墓碑树，而 `MultiLevelCodeTwoTree` 今天只有记账与映射两个成员（「pub enum MultiLevelCodeTwoTree {」，`crates/singlefs-core/src/transaction.rs:2053`）；inode 树同形的拒绝已在代码里（「MoreLeafContainersThanOneRootNodeHolds { containers: usize, capacity: usize },」，`crates/singlefs-core/src/inode_tree.rs:142`）。第六格（墓碑树分裂）归攻方 | 算的（`cells.out:68`）；多层是读代码 |
| 环窗口顶点 258 / 304 / 712 槽 | 删除第二轮判决第五节 | 冻结副本上同一段历史逐字复现：「apex_deficit_slots=258」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:12`）、「apex_deficit_slots=304」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:14`）、「apex_deficit_slots=712」（`research/prompts/m3-delete-r3-sonnet-model/run-final-r-lines.txt:16`）。S2 比原型只多意图那次 +4 槽，顶点最多再加 4 | 量过（副本原型，不是 S2）；同批小盘长环那一格不在这里 |
| 回退那次发布多写的单元数 | 墓碑第一轮判决第五节表（谱系：1 个码 3 容器 × 2 份） | 谱系容器住墓碑树下时，容器重写 + 墓碑树根重写 = 2 个单元 = 4 次写，该段两态闭式 65 543 → 1 048 583（×16）；只重写容器、根不动则 262 151（×4）。原来的回退发布段形 16 + 2 + 1 + 2 的闭式 65 543 是攻方量过的（`research/prompts/m3-tomb-r1-opus-model/run-e-lines.txt` 的 `measured_two_state_closed_form=65543`） | 推的（谱系住处是 G4，没实现）；闭式是算的（`cells.out:72`） |
| R3 逐文件的代价：3 单元文件 50 槽对数据 6 槽；300 单元连续 10%；散落约 98%；再大 5 到 20 倍；空 inode 22 槽、4 GiB 只够 9 634 个 | 删除第二轮判决第五节表 | 50 槽 / 6 槽 = 8.3 倍（S2 另加 +4 / −2 后仍 50）；300 单元连续 61 ÷ 600 = 10.2%；一单元一叶 K = 144 本轮量到 325 ÷ 288 = **113%**（旧的 281 ÷ 288 = 97.6% 是 W = 263，这一轮造法逼出的最大一批是 307 到 312）；取纯函数上界的倍数 6.3 到 24.5；空 inode 211 968 ÷ 22 = 9 634 | 算的（`cells.out:42-46`）；22 槽是攻方代用量，推的 |
| 保留量 13 × ckpt_cost + 8 | 删除第二轮攻方 2.5 节 | 4 GiB：13 × 537 + 8 = 6 989，与量到的 `reserve_slots_dev0=6989`（`run-final-r-lines.txt:9`）同数 | 量过 |
| 落点那一判要「普通 + ckpt_cost」约 541 槽 | 删除第二轮判决 V2 行 | 量到 ckpt_cost 537 / 541 / 543（4 GiB，`run-final-r-lines.txt:19-22`），16 GiB 2 504 / 2 508（`:25-26`）；随盘线性涨，所以落点那一判在大盘上要的是几千槽，保留量同步涨 | 量过 |
| 「约为数据的 4.5%」 | 删除第一轮判决 | 已被第二轮判决推翻（13 槽是 K = 2 一个点），这一轮不再引 | — |

结论：F1 的数涨到约 2.2 倍（意图那次从 12 个单元变 13 个，2^24 → 2^26 把意图一项乘了 4），是这一轮代价数里变化最大的一项；窗口与 R3 的对比数都在第三节。


## 七、这一轮我提的新形态（都算被攻过零轮，交用户表里标「零轮」）

| # | 形态 | 代价数 | 量过还是推的 |
|---|---|---|---|
| 1 | 标签 4 恒 0；标签 6 = 副本数 × ((Ω + 3) × 16384 + 41 × 活对象数)，Ω = (L + 1) × W_ub | 窗口 Ω 每盘 9 200 槽（4 GiB、L = 24、K = 144，单元区的 4.34%；L = 48 为 18 032，8.51%）；窗口不写盘，多写的单元 0 | 窗口顶点与 W 量过（副本原型），式子是算的 |
| 2 | W_ub = 24 + Σ min(N_ℓ, 2·(K + 26)) | 4 GiB 368，16 GiB 380（量到的最大一批 307 到 312） | 量过 8 个点；层数 ≥ 3 的盘与映射树根以下取 4 是推的 |
| 3 | 删除类发布不判式子那一判、判落点那一判；写入类按发布之后的标签判 | 0 个单元 | 删除类量过（朴素拒、关式子那一判全过）；写入类是推的 |
| 4 | 窗口从第一个文件版本起计 | 第一个文件版本多收 9 203 槽 / 盘（4 GiB） | 算的 |
| 5 | 每活对象 41 字节 + 全池 3 槽整容器余量（墓碑预付按记录摊加余量） | 1 000 个对象 = 41 000 字节 + 49 152 字节，对「每对象一整容器」的 46.9 MiB | 算的 |
| 6 | 队列非空时挂载前那一判把窗口算作可动用（零轮修法，补 3.5 第二行 R3 强的格） | 0 个单元 | 推的 |
| 7 | inode 树零条目根 key 区间取 `[0, 0]`（回到 D8 与 I-9.1 已写的） | 不改 kb；`[0, u64::MAX]` 要改两处文字 | 读代码 |
| 8 | 逻辑内容含死亡 txg 与谱系记录；⑤ 每种发布的重写角色集合钉绝对值；checker 加「谱系上每个（号，出生代）至多一条墓碑」 | 断言与检查，0 个单元 | 推的；形状 2、4 是按判据字面构造的 |
| 9 | 第三成员的判定不看墓碑树；`TransactionOutput` 的文件那两个字段变 `Option` | 0 个单元 | 读代码 |

## 八、没做什么

- 不替主 agent 撤销或维持任何判决，不替用户在 R3 逐文件与环窗口之间选；3.5 只给数。
- G3、G4、G6、G7 归攻方，没碰；谱系记录的住处、格式、来回回退，我只在 G8 里写了回退那次发布的单元数（推的）。
- 没实现 S2：原型没有意图记录、`deleted_inodes`、墓碑、谱系记录、nlink 字段；意图那次的 +4 槽、最后一批的 −2 槽、F1 的单元数都是按角色表加的，没落盘量过。
- 没量：写入类发布按发布之后的标签判（原型的判在发布前的读数上）；真满盘上崩在删除中间的重挂载（分配器拷贝上标的满池不能重挂）；分配记录树层数 ≥ 3 的盘；映射树根以下取 4 在局部覆写进来之后；自定 journal 环的小盘与小盘长环（同批另有实验量，我没读它的登记与产物）；F1 在多记录流上的真实单元数。
- 没在入库装置上量任何东西：全部量数在冻结副本 + 删除第二轮攻方与辩方的补丁 + 我的追加用例上，进 kb 之前要在入库装置上重做。
- 没读禁读清单里的文件（别的腿的提示与产出、同批小盘长环实验的登记与产物、实验页）；读了问题单 `research/prompts/m3-ring-window-questions.md`（允许）。
- 只跑 harness 档的一个测试目标 `m3_delete_r3_defense_cells`（经 `run-with-memory-cap.sh` 与 `capped.sh`），没碰 `singlefs-checker-tier`，没跑门禁，没改 `crates/` 与 kb。
- 清理：草稿目录 `/tmp/claude-1000/m3-delete-r3-sonnet/` 里的仓副本与编译目录 `work/`（`tree/`、`target/`）、复跑用的 `rerun-work/`（`tree/`、`target/`）交回前删了；留下的 `dev/` 只有日志与草稿，几十 KB。

## 九、什么现象会推翻这份报告的结论（汇总）

- 第二节：S2 的续做取到时钟或无序迭代（①b）；D18 已定项 11 另有一句定义让扫描排除被回退抛弃的单元（D23:417 那一行）；第一版开始回收墓碑（W3）。
- 第三节：见 3.7 五条。
- 第四节：见 4.4。
- 第五节：见 5.4。
- 第六节：流上量到的单元数与单文件池不同（F1 的数要换，式子不变）；谱系容器与墓碑树根分开写（回退闭式变）。

## 十、交回前的 cite-check 末行

命令：`python3 research/scripts/cite-check.py research/prompts/m3-delete-r3-sonnet-output.md`。末行原样：

```
  ✓ 核了 81 处引文，对上 81 处，没判 0 处（逐处列在上面）
```
