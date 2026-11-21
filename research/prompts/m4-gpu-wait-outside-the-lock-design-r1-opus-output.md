# m4-gpu-wait-outside-the-lock-design-r1 攻方腿（Opus）报告（2026-11-18）

立场：假设「锁里取输入、锁外等 GPU、回锁复核再发布」与「任何前台等锁都让路」是错的，造可达历史打穿（Q2、Q4、Q5 主攻，兼 Q1、Q3、Q6；第一轮角度：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱）。
这一腿一条项目代码都没跑（子 agent 不交任务）：文中「量的」只有 grep 与读文件；别的都是读码推的。探针写好了、没编没跑，等主 agent 跑「要主 agent 跑的命令」。
没读禁读清单里的三处。2026-11-09 那一轮已打中的 C1（锁内等 GPU，前台等的就是 GPU）、C2（只有 fsync 让得了路）与反向饿死不重报。
引代码写主工作区路径与行号（`git rev-parse HEAD` 是 4319aa2b035b2d8238d64aeb68b3ceb7674cc723，`crates/` 另有工作区改动，行号取此刻工作区，逐条 `grep -nF` 过）。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/`；设计轮没有冻结副本，探针拼在这一腿用 `scripts/hosts/minimal-copy.py` 建的副本上（`~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/copy`，此刻工作区的 `crates/`，`.deliverables` 写成交接）。

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model
```

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `a155a3aae76bc30e696c985510e0b1da14a784160006ec602c83760dcff7f247` |
| `probe/src/lock_order.rs` | `24b811e607519d9520154a85f604f81b2e3ac090e87f3f3052efabdfa445193f` |
| `probe/src/main.rs` | `1b26b3b5610ba531a145ec797654279341c801980bd5ac328362bdd7a67fe6dc` |
| `probe/src/pool.rs` | `d536c3f27449573c4a92e40f4f5cf5692122165c151cb6504805d339fcb114ac` |
| `probe/src/stale_base.rs` | `f7d0f2c7ee77096e39d0869e19bfc55b546b45b5694469b5683fda5eb232da9d` |
| `tools/patch_the_copy.py` | `84f0cd1188b696dc10d33c96a4289e7fc7c12098f1f59dcf08cb9ffd22f6eeba` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `tools/rerun.sh` | `bb60bed8e786683becdad422b9f809f7eb9678abf825fa25c4becec270695a17` |

`out/` 是空的。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps`（`bash -n` 过）；`patch_the_copy.py` 只往拷贝的工作区成员加一行、拷入探针包，核心层与运行库一行不改（`py_compile` 过；成员那一行在副本 `Cargo.toml` 第 17 行，命中 1 次）。探针两条子命令：
- `stale-base`：起点池（两块 4 GiB 内存稀疏盘 `SparseBlockDevice`，mkfs → 新池新建文件 → 顺序写 40 单元 → 隔一个部分写一个 → 建 30 次 inode）只建一次，每段历史从它拷。一段历史：可写挂载 → 克隆 `current`（第一段取输入）→ 前台发 k 次部分写（各自发成、答了成功）→ 四臂之一 → 整理之后再发 m 次 → 丢会话（写全落盘的断电）→ 只读挂载读回两个字节。四臂：`stale_base_live_intent_check`（把 `current` 换回克隆的那一版再走显式整理入口，即回锁复核只比活意图时第三段的样子）、`current_base`、`txg_guard`（txg 不等就作废）、`planted_rollback`（判法自证：盘换回发之前那一份，k > 0 的每段都该读出丢了）。由用户决定的几步放开扫：k ∈ {0,1,2,3,5,8}、m ∈ {0,1,3}、前台写的单元 ∈ {1,2,39}，共 216 段；先跑最费时的三段估全量，超过 40 分钟退 3 不跑全量。
- `lock-order`：同一起点挂运行库（不起维护线程），一条线程持 GPU 实现那把锁 `hold`（代替锁外那一段在卡上等），5 毫秒后前台切换、15 毫秒后前台读一次；三臂见第五节。
- 崩溃状态：没取。两条子命令判的是逻辑丢更新与锁序，丢的是写全落盘之后那一态，不靠撕裂写；不调枚举函数，这一腿与接手的腿合计 0 个状态。

## 各格判定一览

| 格 | 判定 | 打中的形（节） | 分辨候选吗 | 量的还是推的 |
|---|---|---|---|---|
| Q2 弱判据 | 打中 | 只比活意图（或 R 排他、分配器、隔离集，凡不含「现行那一版没换」的）放行锁外那一批：第三段拿第一段克隆的那一版当 `previous` 发，txg 照 `previous` 加一、根槽照 txg 算，盖掉前台已答成功的那一次发布（一节） | 分辨：txg 判据臂不中 | 推的，待主 agent 跑 `stale-base` |
| Q2 txg 判据 | 打中 | 只比 txg 漏掉待发：锁外期间 `accept` 收进待发不发布、txg 不变；回锁照发就破了「整理是屏障」，先排空就让每一次写都作废这一批（二节） | 分辨 txg 与「txg 加待发空」 | 推的（读码） |
| Q2/Q3 活锁 | 打中（有条件） | 作废重算时前台发布率 λ、一批锁外 W：一批留得下的概率 e^(−λW)；λ = 1000/s、W ≥ 10.6 ms 时约 2.5 × 10⁻⁵；全空段破例那一批在「作废」这一闸上没有豁免（三节） | 不分辨 txg 与「txg 加待发空」 | 推的（算术，W 取 E185 的 5.3 ms） |
| Q3 让路问在哪 | 打中 | 「任何前台等锁都让路」若在第三段（发布前）再问一次，锁外那几十毫秒里来一次读就作废；只在第一段问则第三段照常持锁发一次（三节） | 分辨两种问法 | 推的 |
| Q4 崩溃语义 | 没打中（严判据下） | txg 相等且待发空时第三段算出的计划与锁内算的一样，写序列是今天就有的；回锁之后照新一版重做（不作废）才出新形：计划对不上新一版拒成计划错、整个调用交回错误（四节） | — | 推的 |
| Q5 锁序 | 打中 | 前台切到 gpu 先取会话锁、在校验里再取 GPU 实现那把锁；锁外那一段持着 GPU 锁时前台切换持会话锁等它，读与 `accept` 排在切换后面；第三段持着 GPU 锁回会话锁则死锁（五节） | 分辨「回锁前放 GPU 锁」与否 | 推的，待主 agent 跑 `lock-order` |
| Q5 卸载 | 打中 | 维护线程整个生命期持 `Arc<Shared>`，卸载不 join 就在 `Arc::try_unwrap` 那一处 panic：「卸载时不等还在跑的提交」做不到，卸载照旧等锁外那一批（五节） | 不分辨（拆不拆都等） | 推的（读码） |
| Q5 panic / 中毒 | 部分 | 锁外 panic 不再毒会话锁而毒 GPU 实现那把锁，之后在会话锁里取它就 panic；结局同今天（坏、要重挂），共用把手的另一次挂载连带坏（五节） | — | 推的 |
| Q1/Q6 最脆弱 | 打中 | 三样只靠调用方守约：发布路径不核 `previous` 是不是现行那一版；「一批在途」不在会话状态里；GPU 锁与会话锁的次序没有类型拦（六节） | — | 推的 |
| 越位 | 打中 | `06-sixth-txn.md` 一·4 带过来的条件是「整体独占」：锁罩住从读 `previous` 到换 current 的整段；拆法把整理一批拆成两段持锁，要先说明它怎么还守这一句（六节） | — | 读条款 |

## 一、Q2：弱判据放行，锁外那一批建在旧一版上发出去（主攻）

**形**。今天一批进门就克隆现行那一版，整批都拿这一份当底：`crates/singlefs-core/src/mounted_session.rs:5370`：「整理只在有树的一版上推」（第 5368 到 5371 行 `let version = current … .clone()`），签发时拿它的 txg 当 `c0`：`crates/singlefs-core/src/mounted_session.rs:5591`：「allocation_generation_at_issue: version.root.checkpoint_txg,」，发布时把 `&version` 当上一版交下去（第 5603、5686 行），发成就换现行那一版：`crates/singlefs-core/src/mounted_session.rs:5610`：「*current = PoolVersion::of_a_version_of_trees(published.clone());」。拆开之后这份克隆自然跨过放锁：第一段取它、第三段拿它发，类型照收。

发布路径不核「上一版就是现行那一版」：新一版的 txg 只从上一版加一，`crates/singlefs-core/src/transaction.rs:11170`：「let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;」（整理一批走的 `publish_compaction_step_after_its_check`，第 11161 行起）；根写到哪一槽只看 txg，`crates/singlefs-core/src/root_ring.rs:172`：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」，`crates/singlefs-core/src/root_ring.rs:173`：「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」。我在 `transaction.rs` 里搜过拿上一版与分配器、与现行那一版比的核：只有新池新建文件那一条路径核「上一条记录的 txg 等于要建在上面的那一版」（`FirstFileVersionDoesNotFollowTheVersionItBuildsOn`，第 5808 行），整理与用户改动的发布路径没有（量的：`grep -c "checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)" crates/singlefs-core/src/transaction.rs` 打出 17，都直接从上一版加一；「搜过」只到 `transaction.rs` 这一份，别的文件没搜）。

**历史**（前缀固定、用户动作放开）：
1. 维护线程一拍，第一段在锁里：克隆 V0（txg t），没有活意图，交 GPU 之前放锁。
2. 锁外等 GPU 期间，前台 `publish_pending_now`（fsync）或 `publish_user_change` 发成 k ≥ 1 次（V1…Vk，txg t+1…t+k），各自答了成功。今天这一步许可：前台入口都只取会话锁（`crates/singlefs-mount-runtime/src/lib.rs:1046`：「pub fn switch_background_compute(」一类，正文二节列了五个），锁外那一段不持会话锁。
3. 第三段回锁，复核只比活意图：V0 与 Vk 都「没有活意图」，相等，放行（只比 R 排他、只比分配器读数同样放行：签发之前 R 不排他，分配器在 k 次之后的读数第三段本来就要重读）。
4. 第三段照今天的代码签发：`crates/singlefs-core/src/mounted_session.rs:5598`：「allocator.compaction_mut().issue(issued);」，以 V0 为底发，txg = t+1，写进 V1 的那一槽；之后一趟里的各批都接在这条线上（`current` 已换成它）。
5. 断电（或接着发 m 次再断电）：恢复按 txg 择根，读回的那一版里没有 V1 那次改动（k = 1 时 V1 的根被盖掉；k ≥ 2 时 Vk 的根还在而内存里的会话走在 V0 那条线上，之后 m 次发布的 txg 与 V2… 撞槽）。

量法：`stale-base` 的 `stale_base_live_intent_check` 臂就是第 3、4 步；读数等主 agent 跑。我推的结局：k ≥ 1 的段里 `lost_foreground=1` 或 `lost_after=1`、或可写重挂失败、或发布路径别处 panic（分配器装着 V1…Vk 的分配，分配记录树是 V0 的）；四种都算打中（答了成功的发布读不回，或坏得要重挂）。`current_base` 与 `txg_guard` 两臂应全 0，`planted_rollback` 是判法自证。翻面观测：`stale_base_live_intent_check` 臂 k ≥ 1 的全部段 `lost_foreground=0 lost_after=0 panicked=0 writable_remount_failed=0`，即发布路径别处另有我没找到的核拦下旧底，这一节撤回。

**四句**：
- 分不分辨候选：分辨。txg（加实例代号）相等这一判把第 2 步之后的每一段都拦下；只比活意图、只比 R 排他、只比分配器的都放行。
- 系统看不看得到判别它的东西：看得到。第三段在锁里，现行那一版的 txg 与克隆那一版的 txg 都在手里，比一次就够。
- 满足的是 Q2 题面哪一分句：「锁外算出的 GPU 结果已经过期（前台在锁外期间发布）却通过了候选的复核判据……结果被用来签发或搬迁」，签发那一支。
- 改法在这几格上还中不中：甲（第三段比 txg 与实例代号，不等就作废）不中；乙（发布路径核「上一版的 txg 等于分配器记下的最近一次发成的 txg」，不等就在任何写之前拒）不中，且对甲漏写的调用方也兜底；只加「回锁重扫 R」不够，选段签发那一支没有 R 可重扫。

## 二、Q2：只比 txg 漏掉待发，屏障与作废两头顶（主攻）

**形**。整理是屏障：`crates/singlefs-core/src/mounted_session.rs:7528`：「整理是屏障——待发里不并进整理的发布」，每次调用起跑之前先排空待发，`crates/singlefs-core/src/mounted_session.rs:7555`：「两次调用之间前台收下的待发在下一次调用先排空」。`accept` 收数据写类改动只进待发、多数时候不发布，txg 不变（第 7134 到 7135 行收进待发，触发才发）；准入判的是收的那一刻的现行那一版与分配器（第 7290 行文档「判的是此刻的现行那一版与分配器」）。今天收进待发与把它发出去之间不会插进整理的发布：别的发布入口都先排空。

**历史**：
1. 第一段在锁里排空待发（今天的第 7664 行），取输入，放锁，txg = t。
2. 锁外期间前台 `accept` 一次数据写（今天 `MountedRuntime::accept` 只取会话锁，`crates/singlefs-mount-runtime/src/lib.rs:961` 那一处取锁）：准入照 t 那一版与此刻分配器判过、收进待发，不触发，txg 仍是 t。
3. 第三段回锁，只比 txg：相等，放行。

此后两条路都破一样已定的东西：
- 照发不排空：整理一批插在「准入判过」与「待发发出去」之间，那一批取目的落点要占 R 外的空洞、搬走的落点进 defer 不当场空出（D16 已定项 15），待发发出去时判的分配器已不是准入那一刻的。准入答过「收下」的改动在 fsync 时被空间拒、或多推几串抬 F，今天走不到（推的；要量就在近满的池上摆，探针没做）。它也直接违背屏障那一句。
- 先排空再发：排空是一次用户改动的发布，txg 变了，这一批照 txg 判据作废。于是锁外期间任何一次 `accept`（不必 fsync）都作废这一批：写多的负载下比三节算的 λ 还大。

**四句**：分辨「只比 txg」与「比 txg 且待发空」两种候选；系统看得到（第三段在锁里读得到待发空不空）；满足 Q2 题面「前台在锁外期间……改了分配器」一句（待发本身不在分配器里，但准入判的就是分配器）；改法甲（只比 txg）在这一格还中，要加「待发空」，加了之后与三节的活锁同一个代价。

## 三、Q2、Q3：作废重算下后台发不出；让路问在哪

**活锁的算术**（推的，不是量的）。照一、二两节，回锁复核要么严（txg 加实例代号相等、待发空），对不上整批作废重算；要么照新一版重做（四节）。取作废：锁外那一段长 W、前台改动（发布或 `accept`）按率 λ 的泊松流到达时，一批留得下的概率 e^(−λW)。W 的下沿：有活意图的一批对 R 重扫一次、甲乙两段各一次调用（`mounted_session.rs` 第 5634 行），每次调用至少一次提交；E185 第 4 次重跑一次提交约 5.3 ms（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:40`：「一次 GPU 提交的挂钟约 5.3 ms」），W ≥ 10.6 ms；没有活意图的一批还加丙段一次与每扫一段两次。λ = 10/s 时 e^(−0.106) ≈ 0.90；λ = 100/s 时 e^(−1.06) ≈ 0.35；λ = 1000/s 时 e^(−10.6) ≈ 2.5 × 10⁻⁵。前台每秒上千次小写加 fsync 的负载下后台几乎一批都发不出，作废的那几批 GPU 与准备输入的主机 CPU 白花（D26 已定项 13 第 7 条按省下的主机 CPU 时间算值不值，作废批全记在赔的一边）。泊松、W 的取法都是推的；量法写进「这条腿自己的限度」。

**底额破例兜不兜得住**。破例只排在让路之前：`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「破例排在第 6 条的让路之前，前台有 fsync 在等也照发」。拆开之后多了一道「回锁作废」，条款里破例在它前后没有位置。全空段低于底额、前台写不进时 txg 还会动：`accept` 被空间拒当场推抬 F，`crates/singlefs-core/src/mounted_session.rs:7080`：「于是这一次调用可能写盘，至多推 B − 1 次空发布」；unlink 是屏障、照发。所以破例那一批照样会被作废，「每趟至少发 1 次」兜不住（要发生，得前台在写不进时还在重试写或删文件；只读的前台不动 txg，兜得住）。能兜住的两个候选，代价都不是零：破例那一批在锁里等 GPU（这一批回到 C1 的形，用户 2026-11-09 接受过）；破例那一批走 CPU（与 D26 已定项 13 第 1 条「开了 gpu 就照开着跑，不按……自动切回 CPU」顶，要改条款）。

**「任何前台等锁都让路」问在哪一段**。今天核心层在每一批之前问一次：`crates/singlefs-core/src/mounted_session.rs:4732`：「match caller.answer_before_the_next_batch() {」，那一刻在第一段（锁里）。拆开之后若在第三段发布之前再问一次，锁外那十几毫秒里来一次读就让路：读不改 txg，却让这一批白算（让路之后留着结果下一次再用，又回到一节的旧底问题）。只在第一段问，第三段照常持锁发一次，前台在第三段等的是一次发布（今天每次发布之间放锁，同样等一次），不比今天差。正推若写「每一批之前问」，要写明是第一段，不是发布之前。

## 四、Q4：崩溃语义与一个事务层

**严判据下没打中**（推的）。第三段核过「txg 与实例代号相等、待发空」之后，现行那一版就是第一段克隆的那一版；GPU 三段的输入是这一版的记录与映射、已分配落点上的字节（写时复制，落点被放掉之前不会被重写，放掉要经一次发布）、常驻隔离集与被拒段表（隔离集只经发布记下，第一段进门取走：`crates/singlefs-core/src/mounted_session.rs:5367`：「compaction.take_the_quarantines_noted_by_the_publishes(allocator.compaction_mut());」；被拒段表只有整理自己写）。所以第三段算出的计划、发的那一次，与在锁里一口气算出的相同，写序列是今天的写序列之一，不用新的崩溃流。签发与 R 排他照旧在第三段锁里做（第 5598 行），锁外那一段没有意图、R 不排他：前台这时可以往候选段里分配，这些改动都是发布，txg 判据拦得下。整理照旧经 `publish_compaction_step_after_its_check` 那一条，不是第二条发布路径（`.claude/rules/fs-design.md:54`：「绕开状态机的第二条发布路径不允许」）。

**回锁照新一版重做（不作废）才出新形**（推的，没量）。计划照第一段那一版算、照新一版发：发布路径逐条核计划，前台在锁外期间改写过的单元源落点已不是映射位置，`crates/singlefs-core/src/transaction.rs:11457`：「PlannedSourceIsNotTheMappingPosition {」；这一种不算空间不够，`crates/singlefs-core/src/mounted_session.rs:5966`：「if !compaction_publish_refused_short_of_space(&refused) {」，整个调用交回错误，运行库送 `CompactionCallRefused` 并停这一拍（`crates/singlefs-mount-runtime/src/lib.rs:630`）。今天这几种计划错只在代码有错时出现，拆开之后成了平常的并发，报警口径要跟着分。选段签发那一支更糟：签发前那一判「R 外的空洞够不够装下 R 里要搬的」（第 5574 到 5588 行）用的是第一段扫出的 R，锁外期间前台分进 R 的单元没数进去，`c0` 若仍取第一段那一版的 txg，这些单元分配代大于 `c0`，意图做完 R 也不空（C140 那一类）；`c0` 若改取新一版的 txg，要搬的就比判过的多（攻方 K-g 那一判照旧数算的死局）。所以「只重算变了的那一部分」要整套重判签发前那几判，等于在锁里重扫 R——锁外省下的就只剩丙段。

## 五、Q5：卸载、切换、出错、panic 撞上锁外那一批（主攻）

### 五·1　GPU 实现那把锁与会话锁的次序

**形**。前台切换取会话锁（`crates/singlefs-mount-runtime/src/lib.rs:1046`：「pub fn switch_background_compute(」，第 1052 行取锁），往下 `crates/singlefs-core/src/mounted_session.rs:3173`：「.request_switch_from_the_next_batch(background_compute_settings, gpu_compute)」，再进校验 `crates/singlefs-core/src/background_compute.rs:1956`：「admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(」，选 `gpu` 那一支在会话锁里取 GPU 实现那把锁：`crates/singlefs-core/src/background_compute.rs:1176`：「let implementation = gpu_compute.lock();」。选 `cpu` 不取（第 1171 行那一支）。今天一批取 GPU 锁之后一直持到整批发完：`crates/singlefs-core/src/mounted_session.rs:5160`：「let mut implementation = gpu_compute.lock();」到 `crates/singlefs-core/src/mounted_session.rs:5178`：「implementation.take_the_submissions_of_the_batch();」，中间就是发布。今天两把锁的次序处处是「会话锁 → GPU 锁」。

**历史 A（拆开、第三段之前放 GPU 锁）**：维护线程锁外持 GPU 锁在卡上等（卡挂住时一段至多约 4 × 等的上限，正文二节读码算的约 40 秒）；前台改卡的切换（`gpu` 换一组卡、改显存上限或等的上限）取到会话锁、在校验里等 GPU 锁；之后来的读（`with_the_session_and_devices`）、`accept`、fsync 都排在会话锁上。前台读等的就是 GPU：C1 换了一个入口回来。卡挂住正是管理员要切换的时刻；切到 `cpu` 不取 GPU 锁、不挡（这一条是「管理员救场走得通」的唯一一条路）。
**历史 B（拆开、第三段持着 GPU 锁回会话锁）**：沿用今天第 5160 到 5178 行的守卫范围（为了第三段读 `take_the_submissions_of_the_batch`），次序在维护线程上变成「GPU 锁 → 会话锁」，与前台切换的「会话锁 → GPU 锁」成环：死锁。不靠卡挂住，一次正常的锁外等与一次切换撞上就成；死锁之后卸载 join 维护线程（`crates/singlefs-mount-runtime/src/lib.rs:1113`），卸载也挂住。

量法：`lock-order` 三臂。`released_before_the_session_lock` 在 hold = 100、400、1600 毫秒各跑切到 gpu 与切到 cpu：我推的读数是切到 gpu 时 `switch_waited_ms ≈ hold − 5`、`read_waited_ms ≈ hold − 15`，切到 cpu 时两样都接近 0；`held_into_the_session_lock_switch_to_cpu` 应 `all_done=1`；`held_into_the_session_lock_switch_to_gpu` 应 `deadlocked=1`（10 秒没齐）。翻面观测：切到 gpu 时 `read_waited_ms` 接近 0（校验那一处其实不在会话锁里取 GPU 锁，我读错了），或第三臂 `deadlocked=0`。

**四句**：分辨「回锁之前放 GPU 锁、且切换校验不在会话锁里取 GPU 锁」与其余形态；系统看得到（谁持哪把锁是确定的）；满足 Q5 题面「GPU 实现那把锁在锁外等的期间被切换……会不会死锁或锁序反转」；改法丙（第三段之前放 GPU 锁，`take_the_submissions_of_the_batch` 挪到锁外那一段末尾）治 B 不治 A；改法丁（切换的卡清单校验挪到会话锁外：先取 GPU 锁读卡清单、放掉，再取会话锁记成待生效）治 A，丁要说清卡清单在两次取锁之间变了怎么办（C943 那一类窗）。

### 五·2　卸载与丢弃

卸载与 Drop 都 join 维护线程（`crates/singlefs-mount-runtime/src/lib.rs:1109`：「一批之内 GPU 还在等时要等到那一批出来」）。拆开之后线程停在锁外等 GPU 时，卸载想「不等还在跑的提交」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`：「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」）就不能 join；可维护线程整个生命期持着 `Arc<Shared>`（线程体是 `Shared` 上的方法，第 691 行），卸载拿盘要 `Arc::try_unwrap`，不 join 就当场 panic：`crates/singlefs-mount-runtime/src/lib.rs:1136`：「panic!("线程 join 过了、前台入口都借着 self：运行库是这一份的唯一持有者")」。所以拆开不改变「卸载等锁外那一批」（至多约 40 秒，推的），条款那一句照旧只在实现层（卡的账）成立、在运行库这一层不成立。要让卸载不等，锁外那一段只能持 GPU 把手与自己那份输入的拷贝，不持 `Arc<Shared>`——线程体要拆，正推若没写这一层就是越位。Drop 同理（第 1205 行 join）。

### 五·3　出错、超时重交、等的那一处 panic、锁中毒

- 出错与超时重交 3 次：GPU 交回出错在第三段锁里照今天的规矩处置（停下、删意图、报警），推的没有新形；超时重交的 40 秒挪到锁外，前台不再等它（除了五·1 的切换），这是拆法真买到的东西。
- 等的那一处 panic（C1030）：今天外壳接住；接不住时，拆开之前毒的是会话锁（坏、要重挂），拆开之后线程体的 `catch_unwind`（第 778 行）记坏、会话锁不毒，而 GPU 实现那把锁中毒：之后谁取它都 panic，`crates/singlefs-core/src/background_compute.rs:746`：「.expect("GPU 实现上一次借出时 panic 过：那一次已经往上报过，不带着可能半改的连续不确定计数再用它")」。前台切到 gpu 在会话锁里取它（第 1176 行）就 panic、毒会话锁；把手几处共用时（`of_a_shared_implementation`，第 735 行；C943），另一次挂载下一批开头取它也 panic。结局同今天（坏、重挂），多了「坏传到共用这只把手的别的挂载」，产品路径一个把手只给一个会话（C943 登记行原话）时不出。
- 前台锁里 panic、会话锁中毒：第三段 `lock_the_session` 报坏（第 491 行），锁外那一批丢掉，GPU 锁随守卫放掉；第一段在会话状态上改过的（`begin_a_batch` 记下的补跑批数、开卡答案）随坏掉的会话一起丢，无害。
- 切到 `cpu` 或换卡在锁外期间到：只记成待生效（第 1961 行），这一批照旧、下一批开头换上，与条款一致；条款第 4 条「出错之前已经收到、还没生效的切换请求……在停下那一刻也算数」今天只能经只供测试的钩子撞到，拆开之后成了平常路径（`replace_the_tasks_stopped(Vec::new())`，第 1962 行，在锁外期间就清了停下表），要有一条真线程用例。
- C943 的窗：拆开之后「交设置」（第一段开头）与「读失败」（第三段）之间夹着整段 GPU 等待，窗从几行代码变成几十毫秒到几十秒（推的）。

## 六、Q1、Q6：哪一条最脆弱；越位

**最脆弱：拆法的正确性靠三样调用方守约，类型一样都不拦**（按坏的后果排）。
1. 「第三段发布的底就是现行那一版」只靠第三段那一次比较。发布路径收 `&TransactionOutput` 当上一版、txg 从它加一（一节），旧底照样发、盖根槽。一处漏写（比如将来打包搬迁照抄这套拆法、只核自己关心的那几样）就是答了成功的改动丢掉。这是一节改法乙要放进发布路径的原因：比较放在每次发布必经的那一处，不放在每个调用方。
2. 「一批在途」不在会话状态里。`begin_a_batch` 交回的 `BackgroundBatchUnderWay` 是个值，第一段拿走、第三段还回 `end_the_batch`；其间别的锁里代码再开一批（`crates/singlefs-core/src/background_compute.rs:2064`：「self.put_the_switch_requested_from_the_next_batch_in_effect();」），或两个整理入口的开头（`crates/singlefs-core/src/mounted_session.rs` 第 3419、4516 行）把待生效的设置换上，第三段就撞断言 `crates/singlefs-core/src/background_compute.rs:2111`：「后台计算的设置一批之内不换：挂着的切换只记成待生效，下一批开头才换上」，在会话锁里 panic、运行库坏。今天不会：一批在一次持锁里开、关。拆开之后能并上的入口：`MountedRuntime::run_one_maintenance_pass` 是公开的（`crates/singlefs-mount-runtime/src/lib.rs:1087`：「pub fn run_one_maintenance_pass(&self)」，文档写给「不起线程的确定性测试直接调」），起了维护线程的运行库上再调它，两拍并行，各开一批；类型不拦。产品路径今天没人这样调（推的），但「每一批之前问」「一批之内不换」两条已定条款都靠它不发生。改法戊：会话里记一个「在途的批」，第二次 `begin_a_batch` 与第一段之外的设置换上都先判它。
3. 「前台」与「后台」取的是同一个 `lock_the_session`（`crates/singlefs-mount-runtime/src/lib.rs:491`：「fn lock_the_session(&self)」）。「任何前台等锁都让路」要在每个前台入口登记计数；登记放进 `lock_the_session` 最省事、也最不会漏将来的入口，可维护线程自己的 `publish_when_due_once`、重发冻结、第一段、第三段都走它，计数里就有了自己：今天的答法 `crates/singlefs-mount-runtime/src/lib.rs:428`：「fn answer_before_the_next_batch(&self)」读到计数大于 0 就让路，后台每一批都给自己让路。登记只能逐个入口写（像 `crates/singlefs-mount-runtime/src/lib.rs:987`：「let _waiting = self.foreground_fsync_is_waiting();」），新加一个前台入口忘了写就不让路，类型不拦。
4. 两把锁的次序（五·1）同样只靠写代码的人记得。`.claude/rules/fs-design.md`「并发交错由工具穷举判定」一节要的是写成形式模型、由工具穷举：两把锁、三类线程（维护、前台读写、前台切换）这一组交错，Q6 的验收里该有一份锁序的模型（门禁 lkmm 那一套或别的穷举工具），不只真线程用例。

**越位：里程碑六一·4 带过来的条件**。那一行写的是「整体独占」：`.claude/kb/milestone/06-sixth-txn.md:59`：「锁罩住从读 `previous` 到换 current 的整段」。拆法把整理一批拆成两段持锁：第一段读 `previous`（克隆），第三段换 current。用户把「里程碑六再拆」提前，没有同时撤这一句。拆法要么写明第三段在锁里重新读 `previous`、核它与第一段那一份是同一版（一节改法甲；这样从读到换仍在一次持锁里，锁外那一段只当提示），要么交用户改一·4 那一句；正文没提这一句，是这一轮的缺口。

## 七、改法各修哪一格（都只在我的推理上过、被攻过零轮；量过的一格都没有）

| 改法 | 一 弱判据旧底 | 二 待发 | 三 活锁 | 三 破例 | 五·1 A 切换挡读 | 五·1 B 死锁 | 五·2 卸载不等 | 六·2 在途批 | 六·3 自让路 |
|---|---|---|---|---|---|---|---|---|---|
| 甲 第三段比 txg 与实例代号，不等作废 | 修（推的，待主 agent 跑） | 不修 | 加重 | 不修 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 甲加 第三段另判待发空 | 修（推的） | 修（推的） | 加重 | 不修 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 乙 发布路径核上一版 txg 等于分配器记的最近发成那一次 | 修，兜住漏写的调用方（推的） | 不修 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 丙 第三段之前放 GPU 锁 | 不碰 | 不碰 | 不碰 | 不碰 | 不修 | 修（推的，待主 agent 跑） | 不碰 | 不碰 | 不碰 |
| 丁 切换的卡清单校验挪出会话锁 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 修（推的） | 不碰 | 不碰 | 不碰 |
| 戊 会话记在途的批 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 不碰 |
| 己 破例那一批在锁里等 GPU | 不碰 | 不碰 | 不碰 | 修（推的），代价是这一批回到 C1 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 庚 让路只在第一段问、计数逐个前台入口登记 | 不碰 | 不碰 | 减轻读引起的作废 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） |
| 辛 锁外那一段不持 `Arc<Shared>` | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修的前提（推的） | 不碰 | 不碰 |

「三 活锁」没有一个改法修：作废重算本身就是它的来源。要下限只能在「连着作废 N 批之后下一批在锁里等」（回到 C1 的形，N 取什么要量）与「照新一版重做」（四节，锁里重扫 R）之间选，两条都要代价数，交用户之前先量（见「这条腿自己的限度」）。

## 八、Q6：验收清单要补的几条（推的）

- 一节：旧底那一格要有一条会红的用例——第三段比较删掉（或只比活意图）时必须红：读回答过成功的那一次。`stale-base` 的 `stale_base_live_intent_check` 臂就是这条用例的骨架；`planted_rollback` 证判法会红。
- 二节：锁外期间 `accept` 一次（不 fsync），断言第三段要么作废、要么排空在先；照发不排空的那一版必须红。
- 三节：作废率要量，不是推：真线程用例在替身卡上按固定间隔（1、10、100 毫秒）发小写加 fsync，数一分钟里第三段发成几批、作废几批；W 用真卡在 5090 与 5060 Ti 上量。
- 五·1：两把锁的次序要有一份穷举模型（`.claude/rules/fs-design.md`「并发交错由工具穷举判定」），加 `lock-order` 那种真线程用例当绑到代码上的锚点。
- 五·2：卸载在替身卡挂住时等多久，断言不超过 4 × 等的上限（拆不拆都要，今天就没量）。
- 六·2：两拍并行（维护线程加一次公开的 `run_one_maintenance_pass`）、锁外期间请求切换，断言不 panic。
- 「替身卡验得了、哪些非真卡不可」：一、二、三的作废计数、五·1、五·2、六·2 都只用替身卡（`lock-order` 的假实现那一种就够）；W 的分布、C1030 那条 panic 路非真卡不可。

## 没打中的形状

读码试过、判不出毛病的（都是推的）：
- 锁外期间发布冻结（前台一次写失败）：txg 不变、只比 txg 放行，但整理那一次发布走 `publish_version_of_trees_relocating_units_after_a_check`，冻结着在任何读写之前拒（`transaction.rs` 第 13211 行 `refuse_while_a_publish_is_frozen`），交回错误、不写盘；下一拍重发冻结的那一次。不是漏判。
- 锁外期间切到 `cpu`：只记成待生效，这一批照旧，`end_the_batch` 的断言不撞（第 2109 行比的是生效的那一组，没换）。
- 锁外期间 Drop：join 等这一批出来，与今天同；第三段在 `stopping` 已设时照发一次，与今天一批发完再查 `stopping` 同。
- 有活意图那一支锁外期间前台往 R 里分配：R 已排他，分不进；前台删文件放掉 R 里的落点是一次发布，txg 判据拦。
- 连续不确定计数（第 8 条）：作废的那一批在卡上已算过一轮，照「一批放弃时……已经交回的段照算一轮」记，不是新形。
- 第二条发布路径：只要搬单元写目的地仍在第三段的发布里做，就没有。若有人为缩短第三段把「原样字节写目的地」挪到锁外，那才是绕开状态机，正文没这么写。
- 取样范围：一、二、四节按 `advance_compaction_by_one_publish_computing_on`（第 5355 到 5732 行）每一个 `return` 与发布点逐个看过；五节看了切换、开卡答案、开批、关批四条路上每一处 `.lock()`（`grep -n "\.lock()" background_compute.rs` 非测试的那几处：第 745、1103、1105、1124、1176 行）；打包搬迁那一族没看（它今天没有 GPU 路径）。

## 这条腿自己的限度

- 探针一次没编、没跑：Rust 是照公开接口手写的，编不过是这一腿的错，承接的腿照 `$WORK/out/build.log` 修。`stale-base` 的读数我只推了方向；推错的翻面条件写在一节。
- `stale-base` 只摆了「签发」那一支的旧底（起点池没有活意图、显式入口一趟从选段签发起）；有活意图那一支（照抄意图的一批建在旧底上）没摆，推的结局相同。
- 起点池只有一种（带洞、两块盘、默认根环几何 S = 8）；k、m、单元三样放开扫，几何与盘数没扫。
- 估时（推的）：编译要编验证包的库（连 `prune-server` 拖进来的 `librocksdb-sys`），是大头；`stale-base` 216 段，每段一次可写挂载、至多一趟整理（至多 3S − 4 = 20 次发布）、一次只读挂载、一次可写重挂，估每段 1 到 5 秒、全量 4 到 18 分钟；`lock-order` 一次挂载加约 10 秒的死锁等待。内存推的：两块 4 GiB 稀疏盘只存写过的扇区，起点池一份加每段一份拷贝，估 1 到 3 GiB。探针先跑三段打出 `ESTIMATE`，超过 40 分钟退 3。
- 三节的作废率是泊松假设下的算术，W 只取了下沿；破例被作废要前台在写不进时还在重试写或删文件，这样的负载有多常见没查。
- 二节「准入答过收下、fsync 时被空间拒」只推了方向，没在近满的池上摆。

## 要主 agent 跑的命令

1. 探针整份复跑（`tools/rerun.sh` 经包装起；包装先删掉运行目录，服务撞内存上限重排同一条命令时照样跑得通）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model
```

   - 在哪起：仓根（命令里的路径都是绝对路径）；在新 shell 里起。`--frozen` 指的副本在这一腿认领的草稿目录下，`.deliverables` 写成交接；运行目录 `probe-run` 也在那下面，名字以 `-run` 收尾，包装只删它。范围声明闸要它时照 `.claude/hooks/run-scope-guard.sh` 文件头补。
   - 不经包装的等价写法（运行目录要不在或是空的）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/copy --work /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/probe-run`。
   - 跑完看：`TASK_SERVER_RESULT` 那一行 `exit=0` 算跑完、退非 0 的件数为 0。`task-server/task log <号> --all` 里依次有 `name=patched …`、`name=build done …`、`name=exit_codes lock_order=0 stale_base=0`、`LOCK name=lock_order …` 八行、`RESULT name=lock_order deadlocked_when_held_into_the_session_lock=…`、`SETUP`、`ESTIMATE`、四行 `RESULT name=stale_base arm=…`、两行 `WALL_SECONDS`、`name=self_checks …`、`name=rerun_done …`。判这一腿的话：一节看 `arm=stale_base_live_intent_check` 那一行的 `lost_foreground`、`lost_after`、`panicked`、`writable_remount_failed` 是否有一样大于 0（都 0 则一节撤回）；五·1 看 `deadlocked_when_held_into_the_session_lock=1` 与 `released_before_the_session_lock switch_to=gpu` 三行的 `read_waited_ms` 是否随 `hold_ms` 涨。
   - 退 1 而日志尾巴是 `cargo build` 的报错：这一腿没编过探针，编不过是它的错，交承接的腿照 `probe-run/out/build.log` 修。退 1 而写「判法自证没红」或「对照臂丢了」：探针判不出它要判的，`stale-base` 的数作废。`stale_base=3` 是估时超过 40 分钟，交承接的腿缩 k 与单元的取样。
   - 内存与时间：推的，见「这条腿自己的限度」。
   - 承接：新派同立场的腿，派发提示写「承接：research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-output.md」与任务号，它读 `task-server/task log <号> --all` 与 `probe-run/out/` 下两份 `.out`，把一节、五·1 的「推的」换成读数，按翻面条件改判定。

2. 引文核对（文本工具，交回前这一腿自己跑过，末行贴在下面）：`python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-output.md`。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；七节的改法都是零轮，进产品代码之前要照 `.claude/rules/three-way-inference.md` 做 A/B。
- 没往统一任务服务交任务，探针没编没跑，判决要引的数都没复跑（命令在上一节）。
- 没在近满的池上摆二节的「准入之后被空间拒」，没量三节的作废率与 W，没造五·3 那条接不住的 panic。
- 草稿目录里留着：`copy/`（56M，交接，主 agent 跑完第 1 条之后删）、`minimal-copy.txt`（建副本的输出）。没建编译目录。

## 引文核对末行

`python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-output.md` 退 0，末行：

```text
  ✓ 核了 36 处引文、3 处裸行号，对上 36 处，没判 3 处（其中裸行号没有引文可比 3 处；逐处列在上面）
```
