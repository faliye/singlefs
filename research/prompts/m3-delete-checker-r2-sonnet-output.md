# 删除线 checker 并进三·3：T2–T15 正推腿报告（云端正推，Sonnet，2026-10-05）

立场：从 D8 已定项 6 / 16、D18 已定项 24 与 `invariants.md` 那十一行推每格「应该是什么」，再与冻结副本 `/tmp/claude-1000/m3-delete-checker-r2/tree/` 今天的代码逐格比。调查报告（prep-report）只当线索，下面每一条都在条款与代码上现查过；它与我读的不一致处单列。
冻结副本 sha 已核（10 份 `sha256sum -c` 全 OK）。重型测试一概没跑；没有编译 Rust。

## 〇　方法与两个小模型（复跑口径）

- 路径缩写：下文不带目录的 `walk.rs`、`image.rs`、`mount.rs`、`transaction.rs`、`inode_tree.rs`、`crash_judge_gpu.rs`、`crash_judge_checker*.wgsl` 行号，都在 `/tmp/claude-1000/m3-delete-checker-r2/tree/crates/` 下（`walk.rs`、`image.rs` 在 `singlefs-checker/src/`；`mount.rs`、`transaction.rs`、`inode_tree.rs` 在 `singlefs-core/src/`；`crash_judge_*` 在 `singlefs-checker-tier/src/`）；带引号的引文都写了全路径。
- 模型目录：`research/prompts/m3-delete-checker-r2-sonnet-model/`，纯 Python 计数模型（`legal_history_model.py`、`bad_history_model.py`、`subsets.py`、`bing_naive.py`、`t12_newest_only.py`），不读盘、不跑 crates；`rerun.sh <还不存在的工作目录>` 在空目录里从头跑五个脚本并与 `expected/` 逐字节 diff，我交回前跑过一次，末行见下一条。
  - `legal_history_model.py`：随机合法历史（建、unlink 意图 + 最后一批、管理员回退到任一未被抛弃的根、崩溃恢复落到较旧根并重发号），每个前缀当一个「镜像」，候选集 = 全部未被抛弃的根，评估各读法有没有误红。
  - `bad_history_model.py`：加一个「水位压回」动作（C479 的开关），数各候选对「同一时间线上复用号」的检出，窗口 K = 候选集只剩最新 K 条根。
- `rerun.sh` 在空目录里跑的末行（这一次真跑的原样）：「diff 相同（5 个输出与 expected/ 逐字节一致）」。
- 模型的前提都落在条款或代码上：回退取树见 `/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-core/src/mount.rs:6280` 与 `:6298`（inode 树 / extent 树 / `deleted_inodes` 取 R_old，墓碑树与谱系树取现行，水位取 max）；候选集见 `walk.rs:8992` 与 `:9049`。
- 复跑（我这一次真跑的输出，原样）：

```
$ nice -n 19 python3 research/prompts/m3-delete-checker-r2-sonnet-model/legal_history_model.py 3000 10
评估时刻 21128（seeds=3000, length=10）
误红 T2a_ver = 0
误红 T2a_x = 0
误红 T3a_I9.18 = 0
误红 T3a_I9.19 = 0
误红 T3a_I9.20 = 0
误红 T3b_I9.18 = 2149
误红 T3b_I9.19 = 94
误红 T3b_I9.20 = 0
误红 N4lit_I9.20 = 430
误红 C479_yi = 0
误红 C479_bing = 0
误红 T12 = 0
最短反例 T3b_I9.18: seed=0 ops=['create', 'unlink', 'rollback']
最短反例 T3b_I9.19: seed=356 ops=['create', 'unlink', 'rollback', 'rollback']
最短反例 N4lit_I9.20: seed=47 ops=['create', 'unlink', 'rollback', 'unlink']
```

- 一个我自己踩过的模型毛病：第一版模型里崩溃恢复只抛弃「同实例」的根，跑出 T2a_x 79、T3a_I9.18 13、C479_yi 340、T12 617 的误红；那是模型错（第二次崩溃落到第一次恢复之前时，第一次恢复之后的新实例根也在被抛弃之列），改成「txg 大于 R_k 的根全抛弃」后全归零。含义：乙、丙、T12 的无误红依赖「候选集按最新根的实例表滤掉被抛弃的根」这一条在多次崩溃恢复下确实成立；这一条我只读了代码（`walk.rs:8992`），没有核「实例表里对中间实例有没有行」，见「没做什么」。

## 一　几条公共事实（现查）

- F1 选根与候选集：选根按 (txg, 实例) 取最大（`walk.rs:8934`），候选集 = 最新根 + 按最新根实例表没被抛弃且不低于回退下界 F 的根（`walk.rs:9049`），再加由记录施加出来的几版（`walk.rs:9103`）。
- F2 删除态扫描按树表单元去重只扫一次（「if !tree_tables_scanned.insert((」，`walk.rs:8227`），版本清单里没有 txg（`VersionToScanTheDeletionStateOf` 只有 name 与树表指针，`walk.rs:7922`）；GPU 扫描内核不去重（「CPU 按树表单元去重只扫一次，违例位是并集，这里不去重」，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2060-2066`）。
- F3 墓碑记录与谱系记录今天只有一处读：I-9.6 墓碑那一半只取对象 ID、只读所选根（「fn largest_published_tombstone_object_identifier(」，`walk.rs:8594`）；主走读走容器树时把记录丢掉（「let _records = self.judge_packed_container(&unit, record_type, &container_name);」，`walk.rs:2149`）。
- F4 inode 记录的出生代今天只在主走读里进一张全版本共用的表，后走的盖前走的（「self.inode_object_birth.insert(inode, object_birth);」，`walk.rs:2390`）；删除态扫描的记录只留 nlink 与 size（「struct InodeRecordOfTheDeletionScan {」，`walk.rs:7929`）。
- F5 容器树条目身份不符今天只记走读失败（「if header_identity != identity {」，`walk.rs:2142`），走读失败让 I-7.2、I-4.8、I-7.4 三条一起红（「judgements.judge("I-7.2", newest_failures.is_empty(), || {」，`walk.rs:9213`；`walk.rs:8975` 是 I-4.8、I-7.4 的口径）——prep 只写了 I-4.8、I-7.4，漏了 I-7.2。
- F6 判不到对象的默认报法：一条不变量一次都没评估、也没登记理由，报「不适用」，理由取默认的「这个镜像上没有它判的对象」（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/image.rs:320`）。
- F7 GPU 覆盖表要求全覆盖（「改回要求全覆盖」，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:71`），违例位容量 128（`crash_judge_gpu.rs:42`，容量断言 `:54`），今天 74 条（`image.rs:88`），加 7 到 10 条仍在容量内。

## 二　T2：I-9.10「该对象的墓碑记录」怎么认、比哪几版

条款（`.claude/kb/invariants.md:297`）：「该对象的墓碑记录里的对象出生代（若有；」「只对 inode 记录还在的对象判」。D8 已定项 6 同时规定「对外暴露的对象身份 = (inode 号, 对象出生代)」与「凡按 inode 号认对象的结构都要带上出生代或随根回退」（`.claude/kb/decisions/08-核心索引结构.md:173`）。

逐候选：
- T2-b 按 (号, 出生代) 找墓碑再比出生代：照条款字面「对象 = (号, 出生代)」最忠实，但比较两边来自同一把钥匙，恒真（推的，算术）。站得住字面，没有判别力，不能当这一格的实现。
- T2-a 按号：同一版墓碑树里对象 ID == N 的每条墓碑，同一版 inode 树里有 N 的记录时，出生代相等。它读「号」当对象身份，与 D8:173 的「按 inode 号认对象要带出生代」字面有张力；能成立靠的是 D8 已定项 6 的前提「inode 号在每一条时间线上单调」（`.claude/kb/decisions/08-核心索引结构.md:171`）——同一条时间线上一个号只对应一个对象，按号找就等于按对象找。
- T2-c 按号且只取谱系上的：合法历史上与 a 同判（模型里 a 与 c 的差只在谱系外墓碑，谱系外墓碑的号若在记录里，出生代必相同，因为同一条时间线上号只创建一次）；坏镜像上 c 看不到谱系外那几条里改坏的出生代，比 a 弱；还要先有「在谱系上」那套（T3），不能先于它落地。

范围：
- 每版各比自己的（墓碑树经头 ID 配 inode 树，I-9.27 同一口径）：模型里 0 误红（`T2a_ver = 0`，含崩溃恢复重发、管理员回退、发意图之后回退到意图之前三形）。记录与它的墓碑同在的合法窗口有两个：意图那次到最后一批之间（nlink 0 的记录加刚追加的墓碑），和管理员回退把记录带回来而墓碑留在现行树里；这两个窗口里按号比出生代都相等（同一时间线上一个号只创建一次）。
- 所选根墓碑对跨版本记录：模型里同样 0 误红（`T2a_x = 0`）；但它靠 `walk.rs:2390` 那张全版本共用、后盖前的表，一旦候选集里混进被抛弃时间线的根（实例表读不出时一条都不滤）就会在崩溃恢复重发上误红，并且一个号两个出生代时这张表只记住一个，比较对象不明；不取。

取：T2-a + 每版各比自己的。依据是条款字面加「号在时间线上单调」前提（D8:171）；T2-b 是字面但恒真；T2-c 弱。它不需要新决策分项，但它读「号」的合法性站在 D8:171 / :184 上（见 T15）。
GPU：每版要多存墓碑 (对象 ID, 出生代) 与记录出生代（记录行 5 个字 → 7 个字），是 `d_note_inode_record` 那张表加两个字（`crash_judge_checker_scan.wgsl:2094`），跟得上。
坏镜像只红自己（推的，没造）：造法见 prep 第二节；本腿只按条款推了它的前提：墓碑对象 ID 2 / 出生代 b3 的镜像在 I-9.6（2 小于水位）、I-9.19（没有同对记录）上都不红。
推翻条件：造得出一份合法镜像（从入口走得到），同一版里某号的记录出生代与同号墓碑出生代不同；或 T2-a 在 `crash-injection-fast-tier` 的随机历史（`history.rs:473` 一带有 Unlink 与 RollBackWhileMounted）上红。

## 三　T3：「在谱系上」从哪条根起算；T4：要不要按已发布谓词再滤

条款：「墓碑记录的死亡 txg 落在挂载根的父链上——从挂载根的 txg 起往回走，有谱系记录 (T_rb, T_old) 的那个 txg（T_rb）的父是 T_old，其余 txg 的父是 txg − 1」（`.claude/kb/invariants.md:305`）。I-9.19 写「照 I-9.18（删除队列里的号有谱系上的墓碑） 的定义」（`.claude/kb/invariants.md:306`），I-9.20 同（`.claude/kb/invariants.md:307`）。

「挂载根」字面是一条根（选根）；条款没有写「每版」。要判候选集里的每一版，就要给每一版指定起算的根：
- T3-a 每版用自己的 txg 与自己树表里的谱系树：把每个已发布过的根当成它自己发布那一刻的挂载根，这个状态在历史上真的是一个挂载镜像，所以它是对条款字面的逐版推广。模型里 I-9.18 / 19 / 20 都 0 误红（`T3a_I9.18 = 0`、`T3a_I9.19 = 0`、`T3a_I9.20 = 0`）。
- T3-b 一律用所选根的 txg 与谱系树：模型里 I-9.18 误红 2149、I-9.19 误红 94（最短：建、unlink、回退到意图之前；I-9.19 的最短：建、unlink、回退、再回退到删完之后）。机理：位于被回退抛弃那一段里的版本，自己的墓碑在所选根的父链外，I-9.18 的队列号找不到链上墓碑；再次回退到删完之后时，被抛弃段里还原回来的活记录与链上的墓碑同对，I-9.19 红。prep 只预言了 I-9.18，I-9.19 这一形是模型多出来的，已用 seed=356 的 6 个根逐个核过（根 5 记录 (1,2,1) 活着、墓碑 (1,2,3) 在所选根链上、在根 5 自己的链上不在）。
- T3-c（新候选，零轮）只判所选根（字面「挂载根」）：不会误红，但把候选集里的版本全放过；与 I-9.5 的「任一版」共用判法（`.claude/kb/invariants.md:292`「判走读到的任一版（候选集里每条根，含由记录施加出来的那几版），不只判最新根」）不一致，GPU 的覆盖也退到一个状态一版。可行，弱于 a。

取 T3-a。两处接口的事实：
- 版本清单要加 `checkpoint_txg`：根环里的根取 `root.checkpoint_txg`，由记录施加出来的取 `version.checkpoint_txg`（后者 `walk.rs:9103` 一带已有这个字段，`walk.rs:9121` 在用）；这是对 `walk.rs:7922` 那个结构的加字段。
- 按树表单元去重（F2）对 T3-a 是安全的（推的，已想过）：共用同一树表单元的几版，树表单元里墓碑的死亡 txg 与谱系记录的 T_rb 都不大于产生那个树表单元的发布的 txg，它们的链在 (t_X, txg] 上没有谱系记录跳跃，所以链上成员关系相同；CPU 去重、GPU 不去重不会让违例位分叉。想推翻它：造一份两个 txg 不同的版本共用树表单元、而墓碑死亡 txg 大于其中较小那个 txg 的镜像（我想不出入口，没造）。
- 谱系记录内容没有不变量（C616，`.claude/kb/checks-owed.md:477`「以后改了其中一条」那一行），链算法对 T_old 不小于 T_rb、读不全一律记「判不了」，不判红。

T4 已发布谓词：不滤。I-9.6 的条款自己写了「（按实例表判）」，I-9.18–I-9.20 的条款没有写；两处并列、一处写一处不写，字面取不滤。合法状态上两种都不误红（所选根可达的墓碑容器按定义都是已发布的）；滤与不滤只改变「所选根可达一片未发布的墓碑容器」这种坏镜像（它本来就红 I-4.2）会不会再连带 I-9.18–I-9.20。不滤也省掉在 GPU 新读墓碑的路径上再带一份实例表行的接口。
推翻 T3-a 的现象：随机历史（建、unlink、回退、崩溃恢复）里某一版在自己的 txg 与谱系树下 I-9.18 / I-9.19 / I-9.20 红而该状态是合法的。
推翻 T4「不滤」的现象：合法历史上所选根可达一片按实例表判不是已发布的墓碑容器，且其中的墓碑让 I-9.18 / I-9.19 / I-9.20 红（那时要滤）。

## 四　T5：旋钮信号从哪来；T6：I-9.21 / I-9.22 判哪几版，干净镜像报什么

条款：I-9.21 只对「没有用只供测试的容量旋钮的镜像」判，非最右容器记录数 == 815（`.claude/kb/invariants.md:308`「没有用只供测试的容量旋钮的镜像上」）；I-9.22 判「旋钮不落盘，checker 从镜像外拿旋钮值」（`.claude/kb/invariants.md:309`），并写明「按产品容量判这类镜像会在合法状态上红」。D18 已定项 24 第 3 条：旋钮不落盘（`.claude/kb/decisions/18-块里携带什么信息.md:572`「关闭路径加只供测试的容量旋钮，旋钮不落盘」）。

今天的事实（现查）：
- 旋钮只住内存（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-core/src/tombstone_tree.rs:53` 一带），容量三样（墓碑容器、谱系容器、码 2 节点条数）各自独立（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-core/src/container_tree.rs:114-127` 的 `ContainerTreeCapacities`）。只有墓碑容器那一样影响 I-9.21 / I-9.22；只压节点条数（树更早长高）不改变容器装满的事实，对 I-9.21 仍算「没用」。
- 池级入口 `check_pool_image(reader: &dyn ImageReader)`（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:8683`）与 `ImageReader`（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/image.rs:19`）里都没有旋钮；全仓 233 处调用（`grep -rn 'check_pool_image(' --include=*.rs` 去掉 fn 定义行，冻结副本 crates/ 下），6 个 `impl ImageReader for`。
- 压旋钮造镜像的只有 `container_trees_grow_a_level_when_the_root_is_full.rs`（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:45` 墓碑容器 1 条），四处断言零违例（`:206`、`:236`、`:259`、`:338`，行内是「assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");」）；层 0 与三·6 的删除历史取产品容量（「tombstone_container_capacity: DeletionSwitches::PRODUCT.tombstone_container_capacity,」，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-harness/src/scenario.rs:264`）。

逐候选：
- T5-K1 新入口 `check_pool_image_with_…(reader, Option<u64>)`，旧入口 = 没装旋钮：照条款字面「从镜像外拿」（参数在镜像外）；233 处调用不动，只有那一个用例文件的 `violations()` 改调新入口；层 0 与 GPU 只走产品那一支（I-9.22 在那两处恒不适用）。站得住。
- T5-K2 `ImageReader` 加一个默认交 `None` 的方法：条款也容得下（读口子算「镜像外」的一种读法）；6 个 impl 继承默认值，调用点不动；但旋钮值得由测试镜像那一侧的读口子带，harness 的 `MemoryPool` 并不知道自己是用什么容量建出来的（容量是写入口上每次发布带的开关，`transaction.rs:4114`），要在建池的测试代码里另行设置，等于也改那个用例文件；blast radius 比 K1 大（trait 面）。站得住。
- T5-K3 从镜像推（有不满的非最右容器就当测试镜像）：I-9.21 永远判不红——它要抓的恰是「非最右容器不满」；站不住。
- T5-只判产品、I-9.22 恒不适用：I-9.21 站得住；I-9.22 的条款是「未实现（三·3 实现）」，要它恒不适用就进 `INVARIANTS_WITHOUT_A_BAD_IMAGE`（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:2924`，今天只有 I-4.4）并改 `invariants.md:309` 的状态列与 `.claude/kb/milestone/03-third-txn.md:85` 的措辞，I-9.22 实质上就不存在了。字面站得住，改的是决策分项以下的 kb 措辞与「每条至少一份坏镜像」的豁免：**标出来，归用户**。

合法镜像上会不会误红：K1 / K2 下，产品镜像（没装旋钮）上 I-9.21 判「非最右 == 815」：产品路径「满了才关」所以任一版的墓碑树都是前缀形（除最右都满）——模型没有建这一格，是条款推的；崩溃恢复落到旧根、管理员回退都只改「当前是哪一版的墓碑树」，不改前缀形。装了旋钮值 K 的镜像按 K 判 I-9.22（`≤ K`，条款只写「≤」，我不加成「非最右 == K」，因为旋钮可以在两次挂载之间换值，换值的合法镜像上非最右容器可以是旧值）。误红只出现在「没装旋钮却用了旋钮」这一形，那是调用方的错，不是 checker 的。
GPU：层 0 恒产品，I-9.21 内核照产品容量判（`TOMBSTONE_CONTAINER_RECORDS`，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-format/src/lib.rs:116`）；I-9.22 内核恒不置位，同 I-4.4 的先例（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_gpu.rs:69` 写明下标 73 的 I-4.4 恒报不适用、内核不置位）。

取：K1（改动最小，条款字面最贴「镜像外」）；K2 也站得住。这一格不触决策分项，只有「只判产品」那一支要交用户。
推翻 K1 的现象：层 0 或 GPU 的合法流里出现装了旋钮的镜像（则产品那一支会误红）；或用旋钮的用例不止一份、每份都得手传旋钮值而传错了没人发现。

T6 判哪几版：
- 每版：墓碑树按叶序列出 (容器号, 记录数)，相邻比容器号严格递增，除最右每片 == 815。合法版本都是前缀形，不误红；扫描要的「按叶序读每片容器的记录数与记录」本来 T2 / T3 就得有（F3），所以 I-9.21 / I-9.22 在 CPU 几乎不加成本；GPU 要多一张按叶序的 (容器号, 记录数) 表。
- 只所选根：也站得住，GPU 便宜，但只看一版，与 I-9.5 / I-9.24 的「每版」口径不一致，漏判候选集里中间版本的坏法。
取每版。
干净镜像：墓碑树零根或只有一片容器时，相邻比较与「非最右」都没有对象；照 F6 报「不适用」带默认理由，不报成立。I-9.5 的「干净的一版也真比过」不适用于这里——它比的是两个集合，这里没有任何一对可比。I-9.24 不同，见 T8。
推翻条件：造得出合法历史（产品容量、从入口走得到）某一版非最右墓碑容器记录数不是 815；或装了旋钮的镜像在换值后合法地超过新值。

## 五　T7：I-9.17 / I-9.23 身份不符怎么报；重合小句

条款：I-9.17、I-9.23 都写「与 I-9.2（条目身份与子头相符） 同形」（`.claude/kb/invariants.md:304`、`:310`），都写「其余判该条目损坏」（`.claude/kb/invariants.md:304`）；D8 已定项 6：「三棵树的内部条目同一形态」，类型段合法集合按树分（`.claude/kb/decisions/08-核心索引结构.md:164`）。

今天（现查）：
- inode 树一侧（I-9.2）：身份不符走 `judgements.judge("I-9.2", …)`，类型段不认得也是判定，并且先登记引用（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:2265-2267` 的 `entry_holds`，`:2278-2280` 类型段 0 那一支，`:2236` 未登记类型按 span 1 登记）。
- 容器树一侧（墓碑树、谱系树共用）：类型段不在 {0, record_type} 记走读失败并 `continue`，在登记引用之前（「if entry_type != record_type {」`walk.rs:2105`，登记在 `:2111`）；身份四段不符只记走读失败（`walk.rs:2142`）；类型段 0 那一支（`:2066-2104`）三段为 0、子单元是码 2、子指针出生树，一条都不判；子指针出生树 == 身份引用出生树没有任何一处判（I-1.12 的调用点不含容器树，`walk.rs:956` 一带的注释写明 inode 那一类归 I-9.2）。
- GPU 同形（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2955` 类型段不对、`:2973` 身份不符，后面都是 `w_failures + 1`）。

逐候选：
- T7-a 单独判定，不再记走读失败：照「与 I-9.2 同形」的字面，也是 I-9.2 在 inode 树一侧已经走的路。身份不符的镜像只红 I-9.17（或 I-9.23），不再连带 I-7.2、I-4.8、I-7.4（F5）。站得住，取它。
- T7-b 两样都记：条款没禁，字面站得住；代价是每个坏镜像必连带 I-7.2、I-4.8、I-7.4，三条的语义是「根走不完 / 单元被复用或抹头」，身份引用对不上不属于这两件事，I-9.2 那边就没有这样记；做不到只红自己，出局（按「只红自己」口径，不是按字面）。
实现要点（来自 inode 一侧的先例，不是新方案）：类型段不认得时先登记引用再判红，不 `continue` 在登记之前，否则 I-3.1 连带红（prep 在这一点上的判断我同意，但是用 `:2105` 与 `:2111` 的先后读出来的，不是跑出来的）。类型段 0 且层级 ≤ 1 那一条走读失败（「这两棵树里没有条款定的形状」）是层级形状，不是身份，保留。

重合小句（T7-c）：
- 「类型段 0 ⇒ 子单元是码 2」与 I-1.6（类标签，`walk.rs:1129` 的 `judge_unit_header`）、「子容器打包记录类型 == 5 / 1」与 I-1.7（`walk.rs:2702`）同红，「子单元偏移 6 = 3」同理。I-9.2 本来就是这样合取的（`walk.rs:2265-2267` 一个 `entry_holds` 里并着 `unit[6] == 3`、身份四段相等、子指针出生树），所以留作合取，沿用先例；其余小句（身份里容器号 / 出生代 / 出生树不符、子指针出生树不等于身份引用出生树、类型段 0 的三段为 0）各自只红自己，这几份坏镜像才是 I-9.17 / I-9.23 的「只红自己」载体。
- 类型段 0 那一支的坏镜像要把树长高：谱系树 135 条一个码 2 节点装满后第 136 片才长高，墓碑树 135 片 × 815 条，都只能靠旋钮（连到 T5-K1 / K2 的旋钮值）。
GPU：把两处 `w_failures + 1` 换成 `w_judge(INVARIANT_I_9_17 或 I_9_23, …)`，两条新下标，不加表，跟得上。
推翻条件：T7-a 改完后某份只改身份的坏镜像仍连带 I-7.2；或一份合法镜像上 I-9.17 / I-9.23 判红。

## 六　T8：I-9.24；T9：I-9.18 并不并

T8 条款：「任何一版里 `deleted_inodes`（D8（核心索引结构） 已定项 16）至多一条 key；每个被判的版本各判一次，不遍历版本之间的历史」（`.claude/kb/invariants.md:311`），D8 已定项 16 第 6 条末句同（`.claude/kb/decisions/08-核心索引结构.md:469`「任何一版里 `deleted_inodes` 至多一条 key」）。
- 条款只谈队列，不谈 inode 树：T8-a 每条 `deleted_inodes` 条目都判，不管头 ID 配不配得上，是字面。今天队列只在配上 inode 树的那一支里才读（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:8367-8385`，`and_then` 串着记录），GPU 在 inode 树条目数不是 1 时直接 `return`、不读队列（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2299-2330` 的 `d_judge_deletion_queue_of_an_entry`）。T8-a 要把读队列提到配对之前，CPU 一处、GPU 一处。
- T8-b 只在配上的组里判：站得住（那一版别的条也判不了），但让「头 ID 配不上 + 队列两条」这一形漏判，是漏判，不是误红。
- 取 T8-a。队列根读不出、或不是层级 0 的码 2 根（`walk.rs:8046` 起）今天记「判不了」，I-9.24 沿用，不把「层级 ≥ 1」判红（D8 没有一句「`deleted_inodes` 恒层级 0」的不变量，这个形状归别处）。
- 干净镜像：条款写「每个被判的版本各判一次」，所以队列空（根指针全零）也判一次、报成立；这与 I-9.5「干净的一版也真比过」同口径。树表里没有 `deleted_inodes` 条目的版本（第一个文件版本之前）没有对象，报默认不适用。
- 会连带的现有用例：「队列 {1} → {1, 7}，只红 I-9.5」那一条（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:105`）在 T8-a 下两条 key，I-9.24 红，要换形。
推翻条件：队列只有一条 key 的合法版本上 I-9.24 红；或两条 key 的镜像在头 ID 配不上时 I-9.24 不红。

T9：I-9.18 的状态列没有「三·3 实现」几个字（`.claude/kb/invariants.md:305` 写的是「未实现（今天没有实现；…第 18 批没做它）」），`.claude/kb/milestone/03-third-txn.md:85` 的九格也不含它。但「在谱系上」的定义就住在 I-9.18 这一行里，I-9.19、I-9.20 都写「照 I-9.18 … 的定义」（`.claude/kb/invariants.md:306`、`:307`）；模型显示 T3 读错时 I-9.18 是最灵敏的探针（T3-b 下 I-9.18 误红 2149，I-9.19 误红 94）。取并：定义行与引用它的两行同批实现，三条互相验读法。代价：「意图之后墓碑对象 ID 改成 2，只红 I-9.6」那一条（`pool_checker_judges_the_deletion_state_of_every_walked_version.rs:266`）里队列 {1} 找不到 (1, b1) 的墓碑，I-9.18 连带红，要换形；:847 那条（意图之后 1 号 nlink 0 → 1）与 T9 无关，I-9.19 一立就得改断言；它的注释写「这一形必与 I-9.18（删除队列里的号有谱系上的墓碑）、I-9.19（谱系上的墓碑不指活对象）」同红（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs:844`），我按条款读 I-9.18 在这一形上成立（队列 {1} 的 1 号有链上墓碑 (1, b1)），只有 I-9.19 红，断言应是 {I-9.5, I-9.19}；注释与我的读法差一条，实现之后跑出来才分得出，不是条款问题。不并也站得住（条款状态列没点它）：代价是「在谱系上」的定义没有实现它的那一条自己的判定。这一格是范围问题，不是条款问题，**交主 agent / 用户定**。

## 七　T10：C479 立什么；T12：I-9.6「树内最大 key」跨根取；T15：D8 已定项 6 要不要写明文

条款：
- C479 自己的检查列：「的镜像，看 I-9.6（水位大于两处最大号） 还拦不拦得住，拦不住就要另立一条不变量」（`.claude/kb/checks-owed.md:384`），并且逐字写「checker 必须判红」（同一行）。S 量过 I-9.6 成立、I-9.19 按 (号, 出生代) 认也拦不住（题面），所以 C479 自己的话是「要另立」。
- D8 已定项 6：「inode 号在每一条时间线上单调」是分裂规则的前提（`.claude/kb/decisions/08-核心索引结构.md:171`）；禁令落在一对上（`.claude/kb/decisions/08-核心索引结构.md:173` 「已发布的 (inode 号, 对象出生代) 不复用」），回收窗口里同号三段逐字相同那一句限定「说的是同一条时间线里的复用」（同一行）；射程里写「号在崩溃恢复落到较旧的根之后可以重发」（`.claude/kb/decisions/08-核心索引结构.md:207`）；字段表写「inode 号不复用后它不再承担复用判别」（`.claude/kb/decisions/08-核心索引结构.md:184`）。C143 的定案是「不搬水位，把唯一性收窄到」一对（`.claude/kb/checks-owed.md:125`）。
- 结论：条款里有三处把「号在每一条时间线上不复用」当事实写（:171、:184、:173 的窗口句），没有一处把它写成禁令；S 那一形（删完、压回水位、再发同号、新出生代）按「一对不复用」的字面不违反，按「时间线上单调」违反。prep 的这个读法我同意（逐行现查过 :171 / :173 / :184 / :207）。

逐候选（数是模型数，合法历史上全是 0 误红，坏历史是「水位压回 + 写路径先拒中间没有记录的号」口径，窗口 K = 候选集只剩最新 K 条根，3000 个种子 × 10 步，有复用的时刻 775 个）：
- 甲 I-9.10 按号（T2-a）：合法历史 0 误红；复用时刻检出 K=3 272、K=6 281、K=1000 281（约 35%）。它靠墓碑，墓碑第一版不回收（`.claude/kb/decisions/18-块里携带什么信息.md:573`「第一版不回收墓碑」），所以不受根环深度影响，但只看得到「删完再发」，看不到「活记录被原地换」（没有墓碑）。C479 检查列要的第二形正是后者，甲单独不够。不需要新决策分项（读法落在 D8:171）。
- 乙 候选集里同号只许一个出生代（记录 + 各版自己谱系上的墓碑）：合法历史 0 误红；检出 K=3 537、K=6 722、K=1000 775（69% / 93% / 100%）；两形都能红，但「活记录被原地换」只在被换之前那一版还在候选集里时红，根环转过去就看不见。依赖候选集按最新根的实例表滤掉被抛弃的根（「let abandoned_by_the_newest_roots_table = |root: &crate::RootView| {」，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:8992`）；那个闭包对 `instance_table_rows` 逐行 `any`（`walk.rs:8993`），实例表一行都没读到时一条都不滤（推的，我没造这一形），这时乙要记「判不了」，否则崩溃恢复重发会误红。需要 T15。
- 丙 相邻版本新号不小于父版水位（父按谱系配）：按谱系配父时合法历史 0 误红；父一律取 txg − 1 时误红 2165 个时刻（`research/prompts/m3-delete-checker-r2-sonnet-model/expected/bing_naive.out`，最短 create、rollback、rollback）。检出 K=3 273、K=6 405、K=1000 434；模型里丙红的时刻都被乙覆盖（`bing_not_yi = 0`），乙红丙不红的 264 / 317 / 341 个。在模型里丙被乙支配，还要多一份「按谱系配父」的算法和 GPU 的每版水位；不取。
- 丁 不立单镜像不变量，改在层 0 / 录制流上跨根比：跨根判器今天没有；C479 的检查列写「checker 必须判红」，丁等于改写那一格，**标出来，归用户**。
- 现有 I-9.6 对照（同口径）：K=3 269、K=6 297、K=1000 300，约 35–39%；合法历史上不误红，坏历史（只压水位不再建）也红，是我口径里「没有复用却红」的 1 322–1 366 个时刻，那是水位压回本身，不是误红。
- 甲乙并用后仍漏的：K=3 时 150 / 775（19%），K=6 时 28 / 775（3.6%），K=1000 时 0（`research/prompts/m3-delete-checker-r2-sonnet-model/expected/subsets.out`）。含义：乙的检出随窗口涨，窗口小时有一成多的坏历史谁都不红（活记录被原地换、被换之前那一版已不在窗口里、墓碑也没有）。**漏判，不是误红**，要在 C479 那一行写明。

取：甲 + 乙（甲不受窗口影响，乙补活记录被换）。乙要先有 T15 的明文，否则乙判红的状态（同时间线复用）在 D8 里只有「前提」没有「禁令」，立不变量没有出处。T15 不写时：只做甲，并在 C479 那一行写明「活记录被原地换」这一形靠 I-9.6 与（窗口内）别的版本，单镜像判不了的部分仍欠。

现有用例连带：`inode_number_watermark_pressed_back_reissues_a_number.rs` 用例一断言「只红 I-9.6」（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:106-107` 的 `vec!["I-9.6"]`）：乙下更早那一版有 (3, 旧出生代)，要连带红，断言要改；甲下不变。
GPU：甲不新增下标；乙新增一个下标和一张跨版本的「号 → 出生代」表（草稿区加一张，表满记判不了，同 `d_note_inode_record` 先例）。
推翻条件：甲 / 乙在合法历史（含 `history.rs:473` 一带的随机历史：Unlink、RollBackWhileMounted、CrashRecoveryAbandoningTheNewestRoot）上红；或造得出「时间线上复用」而甲乙在窗口足够大时都不红。

T15：要不要把「同一条时间线上 inode 号不复用」写进 D8 已定项 6。
- 写：给乙一个出处，也给甲的「按号认」一个出处；改的是决策分项的正文（射程那一段）和变更史；措辞要带「时间线」的定义（所选根的祖先链，含管理员回退抛弃的那一段；崩溃恢复抛弃的时间线不算），否则会与 :207 的「崩溃恢复之后可以重发」打架。**要改决策分项，交用户。**
- 不写：只做甲（和丁）；甲仍然成立（条款 :171 / :184 已经把单调当前提），但其合法性站在前提而不是禁令上。
这一格的推翻条件：用户或另一条腿找出 D8 另一处写了「恢复之后重发的号可以与同一时间线上已有的号相同」。

T12：I-9.6「树内最大 key」跨根取。
- 条款（`.claude/kb/invariants.md:293`）只写「该树 inode 树内最大 key」，没写取哪一版。代码在主走读每片叶上取最大（「self.largest_inode_number_in_the_inode_tree = Some(」，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:2395`），候选集全部版本累积，最后与最新根的水位比（「judgements.judge("I-9.6", watermark > largest_inode, || {」，`walk.rs:9336`）；GPU 同形（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2665` 的运行最大值）。
- 合法历史上跨根取不误红：模型 `T12 = 0`（回退取 max、恢复后候选集滤掉被抛弃的根，所以跨根最大 key 不超过最新根水位）。含水位压回的坏历史上，跨根取红 1666 个时刻、只取最新根红 1012 个、只有跨根红 654 个，合法历史两边都是 0（`research/prompts/m3-delete-checker-r2-sonnet-model/expected/t12_newest_only.out`，同一个水位压回口径的模型数）。
- 改 kb 字面成「候选集里任一版」：只改一行字，代码与 GPU 都已经是这样，跨根还多检出上面那 654 个时刻（最新根的树里没有高位 key、水位却回退）。改代码只取最新根：要改 CPU 和 GPU 内核两处，还少一形检出。取改 kb 字面。
- 同一前提：候选集的滤法对（见公共事实 F1 与上面「方法」里那条未核的前提）。
推翻条件：合法历史上某一版 inode 树内最大 key 不小于最新根水位。

## 八　T11：S4a「同一片叶里同号两条」立什么

条款里没有一句写容器内记录次序（`grep` 在 D8 已定项 6、D18 已定项 24、`invariants.md` 里搜「记录…升序」只命中树表条目与 journal 的，没有 inode 叶的）；能推出它的是两句：
- D8 已定项 6：「插入永远落在最右的叶」「分裂点取末尾」（`.claude/kb/decisions/08-核心索引结构.md:171`）加上「唯一性范围 (树 ID, inode)」（`.claude/kb/decisions/08-核心索引结构.md:173`）：新号恒大于树里已有的每一个 key、只在尾部追加，所以一片叶里的记录必按 inode 号严格升序；合并只许左吸收右 / 左从右借（同一行 :171），次序不变。
- 核心层的读写都靠这条性质：「记录按 inode 号升序」（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-core/src/inode_tree.rs:51`）、二分查找（`inode_tree.rs:298`）、写入口拒「同一次写里的记录没有按 inode 号严格升序排」（`inode_tree.rs:158`）。
I-9.4 今天管跨容器：记录号 ≥ 容器号、容器号沿叶序严格递增、左容器最大 key < 右容器号（`.claude/kb/invariants.md:291`）。三句合起来只缺容器内那一段。扫描一侧的代码注释已经把唯一性当成别处判过的（「同一个号出现两条时后读到的那条盖掉先读到的（号的唯一性由主走读的 I-9.4、I-1.1 判）」，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:8086`），这句注释今天不真。

逐候选：
- 甲 并进 I-9.4，加第四句「同一片类型 2 容器里记录号沿记录序严格递增」：站得住。合法镜像上不误红（上面两句推出的性质；核心层写入口拒非升序是它的第二条独立支撑）。实现是 `walk_inode_leaf_container` 的记录循环里留上一条的号（「for record in records {」，`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:2356`），GPU 是叶记录循环里同样留一个局部变量（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2635` 紧挨着 I-9.4 第一句），不加下标、不加表。S4a（[1 nlink 0, 1 nlink 1]、队列空）：扫描里后者盖前者（`walk.rs:8112` 的 `records.insert(`），I-9.5 成立，所以今天零违例，加甲之后只红 I-9.4（推的）。S4b（[1 nlink 1, 1 nlink 0]）今天红 I-9.5、I-4.7，加甲之后再多红 I-9.4；这一形的连带红是重号本身造成的，不算「不能只红自己」。
- 乙 另立一条：同样站得住，判法同甲；多一个下标、GPU 覆盖表一项、`writable_mount_of_a_formatted_pool.rs` 的不适用表与条数注释、干净镜像用例的清单（prep 第十五节列的文件）。多出来的只有登记成本，没有判别力上的好处。
- 丙 并进 I-1.7 的逐条记录那一步：I-1.7 是「判定顺序」，失败即中止、容器的记录不再交出（「任一步失败即中止」，`.claude/kb/invariants.md:32`），必须限定类型 2——墓碑记录按死亡先后追加（D18 已定项 24 第 3 条「有新记录的那次发布才重写」，`.claude/kb/decisions/18-块里携带什么信息.md:572`），先删 3 再删 2 就是 [3, 2]，合法；实例表（类型 4）有链指针记录。而且读者（核心层）也得按同一步拒，否则 checker 的中止链与读者的中止链不是同一套。站得住，但改读者规则、改格式冻结面上的判定顺序，超出这一批。
- X2（扫描收记录时同号第二次出现交回判不了）：与甲并用时没必要（甲已经在主走读里红了）；单做它，S4a 由「成立」变「判不了」、零违例，洞还在。只改 CPU 不改 GPU 会分叉：GPU 的 `d_note_inode_record` 同号盖掉（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2093` 的注释「同一个号已有一行就盖掉」），CPU 若判不了而 GPU 置违例位，对拍不一致（GPU 对拍只跑合法流，看不出）。不取。
取甲。推翻条件：造得出合法镜像（从入口走得到）某片叶里记录号不严格升序；或甲加上之后某份现有「只红 X」的用例连带红 I-9.4。

## 九　T13：新条在 `IMPLEMENTED_INVARIANTS` 里的次序；T14：I-9.20 的两个问

T13：`image.rs` 的文档把规则写死：下标 i ↔ 层 0 违例位 i，「只往末尾追加」（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/image.rs:75`），「前 49 条的下标不动」（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/image.rs:76`），今天 74 条（`image.rs:88`）；GPU 覆盖表要求 0..N−1 一条不缺不重（`crash_judge_gpu.rs:79-93` 的常量断言），违例位容量 128（`crash_judge_gpu.rs:42`）。
- 按我的取法要新增的下标：I-9.17、I-9.18（T9 并）、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、C479-乙，共 9 条，总数 83，在 128 以内；I-1.7、I-9.10、I-9.4 都是改已有行，不加下标；S4a 取甲不加。
- 次序：规则只要求追加，次序本身不影响判定；prep 的建议次序（I-9.17、I-9.18、I-9.19…I-9.24、C479-乙）站得住，按 kb 编号升序、新立的接最后，最好读。I-9.22 的下标要占（同 I-4.4 占着下标 73、内核不置位）。
- 同批必须同步的数：`.claude/kb/invariants.md:12`「判 74 条」及文件头同步纪律、`image.rs:88` 的 `[&str; 74]`、`image.rs:85-87` 那段文档、GPU 覆盖表与常量 `INVARIANT_I_*`（`crash_judge_checker_common.wgsl:89` 起）。
推翻条件：无（这是登记次序，不是判定）。

T14-a「每条记录」管谁：I-9.20 的句式是「墓碑树里在谱系上的墓碑记录…按 (号, 对象出生代) 不重复；每条记录区间起 < 区间止，第一版只许整对象 [0, 2⁶⁴ − 1]」（`.claude/kb/invariants.md:307`），标题是「谱系上的墓碑不重号且记录合法」，「每条记录」指回「在谱系上的墓碑记录」。字面：只谱系上的。判全部也站得住（谱系外的墓碑由同一条追加路径写出，形状相同，合法镜像上两种同绿），更严，但超出字面；取字面。
T14-b 「记录数 ≤ 815」：`(32768 − 136) ÷ 40` 取整 = 815（`python3 -c` 现算：32632 ÷ 40 = 815.8）；I-1.7 的 `count * width <= declared && declared <= data_unit_bytes() - 136`（`/tmp/claude-1000/m3-delete-checker-r2/tree/crates/singlefs-checker/src/walk.rs:2702-2705`）已把它推出来，且 I-1.7 不合法时容器的记录不交出（`walk.rs:2707`）。这一小句永远不会单独红，没有「只红它」的坏镜像。条款字面带着它（`.claude/kb/invariants.md:307`「每个打包记录类型 1 容器的记录数 ≤ 815」），那一小句不受「在谱系上」约束（它说的是容器）。取留作合取，判法读头里的记录数（`walk.rs:2699`），不要求坏镜像；删掉也站得住，那是改 kb 一行。
GPU：无新表，容器头记录数已在读。
推翻 T14 的取法：合法历史上谱系外的墓碑记录区间不是整对象（则「全部」会误红，字面取法不受影响）；或一份坏镜像只改谱系外那条记录而无人察觉，要用户认为这也该红（则改取「全部」）。

## 十　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T2 | 取 a + 每版 | 按号 + 每版各比自己的；b 恒真，c 弱；合法历史 0 误红；前提落在 D8:171（号在时间线上单调） |
| T3 | 取 a；b 出局 | 每版用自己的 txg 与谱系树；b 在 I-9.18 误红 2149、I-9.19 误红 94（模型）；新候选 c 只判所选根，弱 |
| T4 | 不滤 | I-9.6 写了「按实例表判」、I-9.18–20 没写；合法状态两种都不误红 |
| T5 | 取 K1，K2 也成立；「只判产品」交用户 | K1 233 处调用不动；K3 让 I-9.21 判不红，出局；「只判产品」要改 `invariants.md:309` 与 milestone 措辞、进豁免表，归用户 |
| T6 | 每版；干净镜像报不适用 | 合法版本都是前缀形；零根 / 一片没有可比的对象，走默认不适用 |
| T7 | 取 a；小句留作合取 | 条款写「与 I-9.2 同形」；b 必连带 I-7.2 / I-4.8 / I-7.4（F5）；重合小句沿 I-9.2 的合取先例 |
| T8 | 取 a；干净报成立 | 条款只谈队列；GPU 读队列要提到配对之前；:105 那条用例要换形 |
| T9 | 交主 agent / 用户（我倾向并） | 范围问题；「在谱系上」的定义住在 I-9.18，且它是 T3 读错时最灵敏的探针；代价 :266 一条用例换形 |
| T10 | 取甲 + 乙；丙被乙支配；丁交用户 | 甲不受根环深度影响但看不到活记录被换；乙补它，窗口小时仍漏一成多；乙需要 T15；丁改写 C479 检查列 |
| T11 | 取甲 | 并进 I-9.4 第四句；无新下标；扫描注释 `walk.rs:8086` 的前提由它兑现；丙改读者规则，X2 不需要 |
| T12 | 改 kb 字面成「候选集里任一版」 | 代码与 GPU 已是这样，模型 0 误红；改代码要动两处且少一形检出 |
| T13 | 按 prep 次序追加 | 次序不影响判定；我的取法新增 9 条，总 83，在 128 内；同批同步 kb 的 74 |
| T14 | a 取字面（只谱系上）；b 留作合取 | ≤ 815 由 I-1.7 推出、永不单独红 |
| T15 | 写（乙的出处）/ 不写（只做甲） | 要改决策分项，交用户；不写则 C479 那一行写明活记录被换这一形单镜像判不了的部分仍欠 |

交用户的（要改决策分项或出口措辞）：T5「只判产品」、T15「写」、T10 丁（改 C479 检查列）。被攻过零轮的：我新提的 T3-c；T14-a 取字面不是新提；模型里的 T10 检出数是我口径（水位压回动作 + 窗口 K），不是实现上量的。

## 十一　没做什么

- 重型测试一概没跑；没跑 cargo、harness 档、checker 档、GPU 判器；没编译 Rust。凡写「只红自己」「照旧全绿」的，是按代码与条款推的，没造过镜像、没跑过（T2、T7、T8、T11 的坏镜像都是这样）。
- 两个模型是条款与代码读法的计数模型，不是 crates 的运行：没有根环容量与回退下界 F（候选集取全部未被抛弃的根，窗口 K 只用在 C479 的检出数上）、没有分批、没有多头、没有实例表读不出、没有 journal 施加出来的版本。模型数不进 kb，要引得在入库装置上重做一遍。
- 没核「多次崩溃恢复、第二次落到第一次恢复之前」时最新根的实例表对中间实例有没有行（模型按「txg 大于 R_k 的根全部抛弃」处理）：这是乙、丙、T12、T2 跨版本无误红的共同前提，只读了 `walk.rs:8992` 的闭包。
- 没读 prep 引的攻方产物（`n7.txt`、`m3-batch19-code-r1-opus-model/` 等）、没复跑攻方量数；没读本轮攻方的报告与模型（禁读清单）。prep 与我不一致的地方：F5（I-7.2 也连带）、T3-b 在 I-9.19 上也误红（它只预言了 I-9.18）。
- 没有改任何仓内文件，除了写本报告和模型目录 `research/prompts/m3-delete-checker-r2-sonnet-model/`；草稿在 `/tmp/claude-1000/m3-delete-checker-r2-sonnet/`（只有模型的几份跑出输出，没有编译目录、没有仓副本；rerun 的工作目录已删）。
- GPU 的「跟得上」都是结构判断（读了覆盖表断言、违例位容量、扫描内核的草稿区与 `d_note_inode_record`），没编、没跑；草稿区 `SCRATCH_WORDS`（`crash_judge_gpu.rs:120`，今天 298 736 个字）每版多存墓碑与谱系后要涨多少，没估。
- 不替主 agent 采纳或出判决；T9、T5「只判产品」、T10 丁、T15 都是交出去的。

## 交回前 cite-check 末行

  ✓ 核了 30 处引文，对上 30 处，没判 0 处（逐处列在上面）
