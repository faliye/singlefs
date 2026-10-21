# m3-multi-file-unlink-r1 攻方腿（Opus）报告（2026-10-19）

立场：假设正文第三节 U1–U5 里把 D8（核心索引结构） 已定项 16 原样套到多文件上的读法是错的，造可达的历史去打穿它。代码一律读那棵打上补丁的树 `~/.local/share/singlefs/work/m3-multi-file-unlink-r1-tree`（下面写 `crates/…` 的行号，与主工作区此刻同名文件逐字节相同，`cmp` 核过 11 份被引文件）；kb 条款写 kb 文件自己的行号。

## 复跑命令与文件

模型目录 `research/prompts/m3-multi-file-unlink-r1-opus-model/`：一份 harness 档原型用例、一份给冻结树加变体开关的补丁、三份观测输出、复跑脚本。复跑（工作目录随意，新草稿目录必须不存在或为空）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-multi-file-unlink-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-multi-file-unlink-r1-tree --work <新草稿目录>
```

它把冻结树拷进草稿目录、打 `core-variant.patch`、放进原型用例，三种读法各跑一遍 `cargo test -p singlefs-harness --test opus_multi_file_unlink`，把 `OPUS|` 观测行排序后与 `out/` 里那一份逐行比。本腿在副本上自跑一次的原样输出：

```
name=rerun_compare variant=today same_as_model_out=yes lines=18
name=rerun_compare variant=lift same_as_model_out=yes lines=42
name=rerun_compare variant=lift_count same_as_model_out=yes lines=26
```

三种读法（`OPUS_VARIANT`）：`today` = 那棵树今天的代码；`lift` = 只把 `refuse_an_unlink_the_deletion_steps_do_not_cover` 撤成放行；`lift_count` = 再让 `next_unlink_step_after` 对 inode N 取它自己的剩余单元数（第 1、2 条原样套到 N 的最小改法）。补丁只加这两处开关，不设变量时行为与冻结树相同。盘是两块 4 GiB 内存稀疏盘（`common_admission` 的 `PoolUnderTest::start_after_the_first_file`，`SparseBlockDevice`），起点是 mkfs、取号、暖机、第一个文件（3000 字节、1 个单元）。

```
1b477285d727b418c9fd369e41b8270d42cbdcd4102363853c8635c9c68ce83c  research/prompts/m3-multi-file-unlink-r1-opus-model/core-variant.patch
674376995ba72935c87269f4379bace5f3de35c3a6c7cd6338432f595a244303  research/prompts/m3-multi-file-unlink-r1-opus-model/opus_multi_file_unlink.rs
a356cb7edf0f30e359573e7b56db348aa563f11df322bc842a6e905ab66eede0  research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift_count.txt
1a97d9577692e9c494397d6c4377881f08b408a5c1cf509cc8460881244870a1  research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt
9dc641021cf64b8366ebfc6637a39a5e7cf619340c64670607cb8f2d24ba3880  research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-today.txt
1ca5de43ce5478910ea67004acc052b3d79ff3e5f2b1835f97653d5c81385b3b  research/prompts/m3-multi-file-unlink-r1-opus-model/tools/rerun.sh
```

`tools/rerun.sh` 是 `scripts/agents/three-way-rerun-template.sh` 的拷贝、只改了 `rerun_steps`；模板自带的 `--selftest` 拿模板原来那一段 `rerun_steps` 当被测物，改过之后它在临时目录里找不到补丁而红，这是改 `rerun_steps` 的必然结果，不是脚本坏了。

## 各格判定一览

| 格 | 判定 | 打中的形状（一句） | 量过 / 推的 |
|---|---|---|---|
| U1 | 打中 | 只撤拒绝：删 inode 2 的最后一批删了记录、没动 extent 条目与 3 个单元，池级 checker I-4.7、I-1.13 红，下一次可写挂载被恢复拒 | 量过（`lift` 的 B 段） |
| U1 | 打中 | 第 1、2 条原样套到 N：inode 2 的第一批截的是第一个文件，丢掉的单元被写者改标成「之后到 size 是洞」，读者读出零、删除做完之前没有一条判法报 inode 1 | 量过（`lift_count` 的 C 段） |
| U1 | 打中 | 最后一批写标签 0：与 D8 已定项 14「写者不写标签 0」直接冲突；恢复对「第一个文件之外、没有单元」拒，池级 checker 对没有记录的标签 0 条目不判，两边判得不一致 | 推的 |
| U2 | 打中 | 删除留下零条目非根叶、或层级 ≥ 1 的零条目根：池级 checker I-1.1 判红，核心读者照收 | 推的 |
| U2 | 打中 | 不降层：上段长到层 1 之后删第一个文件，第一个文件最后一批的断言 panic，之后每次可写挂载的续做都 panic | 量过（`lift` 的 E 段） |
| U2 | 没打中 | 判据 ①b（同一份崩溃镜像各自续做逐字节相同）：形状随历史不同不破 ①b | 推的 |
| U3 | 打中 | K-U5「拦」今天不拦：删空之后建 inode、按 inode 号写 3 个单元做成，checker 全绿、读回相等；写进去的文件删不掉 | 量过（`today` 的 A1 段） |
| U3 | 打中 | 判别子观测不到：「删空之后建的 inode」与「删第一个文件之前就在的 inode」两段历史走到同一个成员、同一份摘要 | 量过（`today` 的 A1、A2 段）；「只有墓碑死亡 txg 对出生代分得开」是推的 |
| U3 | 打中 | 正文要的「次序不同的两段历史终态相同」按第 7 条自己的「逻辑内容」定义就不可能成立（含墓碑死亡 txg） | 推的 |
| U3 | 没打中 | 只撤拒绝、inode 2 与第一个文件同在第 0 片叶时删第一个文件：做成、inode 2 读回相等、重挂照常 | 量过（`lift` 的 D 段） |
| U4 | 打中 | 任何一个对象删除中，扫描重建 I-1.13 对整份镜像报不适用，别的文件的索引一格都不比；C 段里第一个文件丢的单元要等删除做完才被 I-1.13 抓到 | 量过（`lift` 的 F 段、`lift_count` 的 C 段） |
| U4 | 没打中 | `deleted_inodes` 至多一条 key：会话先走空与意图入口的断言在多文件下照样守住 | 推的 |
| U5 | 打中 | 正文第二节没点名、改成多文件就不成立的前提 14 处（含 5 处直接决定删除各步走哪条分支的代码） | grep 现查 |

「量过」的数都是攻方副本上的数，不是入库装置上的数。

## U1　删一个第一个文件之外、带数据单元的文件

### U1-甲　只撤拒绝：最后一批删了记录、没删条目与单元（量过）

历史（`lift`，用例 `lift_only_unlinks_inode_two_holding_three_data_units`）：mkfs → 第一个文件 → 建 inode 2 → 按 inode 号顺序写 inode 2 三个单元（97,895 字节以内）→ 会话 unlink 2。每一步许可它的那一句：

1. 意图之后下一步只看第一个文件有没有单元：「第一版只有第一个文件写数据：别的 inode 一个数据单元都没有，意图之后直接是最后一批。」（`crates/singlefs-core/src/transaction.rs:9296`）——inode 2 剩余单元数取 0，直接走最后一批。
2. 最后一批之后的单元数只对 inode 1 归零：「DeletionStepOfThePublish::FinalBatch { inode } if inode.0 == FIRST_INODE_NUMBER => 0,」（`crates/singlefs-core/src/transaction.rs:4986`）；inode 2 的最后一批落到别的那一支，extent 计划照抄上一版。
3. 第一个文件之外各文件的计划只认「这次写的那一个」整份重写，别的照抄：`plan_the_files_other_than_the_first_after_this_publish` 由 `crates/singlefs-core/src/transaction.rs:6971` 那一处以「这次写的文件」= 无调用，inode 2 那一项连同 3 个数据单元原样留下。
4. inode 树那一侧照第 2 条删了记录（`remove_record_from_leaf_containers`）。

原样输出（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt`）：

- 「records(inode,nlink,size)=[(1, 1, 3000)] first_file_units=1 others(inode,units)=[(2, 3)]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:20`）
- 「I-4.7: 根（实例 1、txg 7）：inode 2 处于删除中（inode 树里没有它的记录），extent 树里还有 3 个数据单元，号也不在 deleted_inodes 里」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:21`），同一行 I-1.13 也红（中央映射与每个 inode 的数据单元集合各差 3 处）。
- 「mount_writable=Err(Recovery(UnitMalformed { what: "inode 树里没有第一个文件之外那个文件的记录，extent 树里却有它的数据单元" }))」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:19`），拒在 `crates/singlefs-core/src/recovery.rs:3325`。

结局：3 个单元成孤儿，池再也挂不成可写。四句：① 不分辨臂——标签 0 与「从叶里去掉」两个候选都要先改这三处分派，这是候选共用的前提，另立一笔账；② 系统看得到判别它的东西（`previous.file_other_than_the_first(inode)` 在做决定那一刻就在内存里）；③ 满足的是第 2 条「最后一批释放剩下全部单元、删 inode 记录、删 `deleted_inodes` 那条 key，同一次发布」（`.claude/kb/decisions/08-核心索引结构.md:482`）里「释放剩下全部单元」那一分句的否定；④ 正文没有跑前改法条款，不适用。

### U1-乙　第 1、2 条原样套到 N：第一批截的是第一个文件，丢的单元被洗成洞（量过）

历史（`lift_count`，用例 `lift_count_first_batch_of_inode_two_with_a_three_unit_first_file`，批大小压成 1）：第一个文件顺序写成 3 个单元 → 建 inode 2 → inode 2 写 3 个单元 → `publish_unlink_intent(2)` → `publish_unlink_step(2)` 一次。

1. `next_unlink_step_after` 对 N 取 N 的单元数（只改这一处），交回 `Batch { data_units_kept: 2 }`。
2. 发布路径的 `Batch` 那一支把第一个文件写死在参数里：「(DeletionStepOfThePublish::Batch { .. }, Some(previous_version)) => {」（`crates/singlefs-core/src/transaction.rs:6989`），下一行起传 `FIRST_INODE_NUMBER`（`crates/singlefs-core/src/transaction.rs:6992`），批之前的单元数取「let data_units_before = previous.map_or(0, |version| version.data_pointers.len());」（`crates/singlefs-core/src/transaction.rs:6965`）——第一个文件的。
3. 即使把第 2 步的 `FIRST_INODE_NUMBER` 换成 N 也不够：`ExtentTreeVersion` 的下段只有第一个文件那一棵——「/// 一版 extent 树在内存里的样子（第一版只有第一个文件有内容，下段只有它那一棵）」（`crates/singlefs-core/src/extent_tree.rs:664`），`plan_the_extent_tree_after_a_deletion_batch` 的下段照抄 / 换下全按 `previous.lower_nodes` 算（`crates/singlefs-core/src/unlink.rs:203-212`），`inode` 参数只进上段那一串（推的：这一半没跑）。
4. 写者每次发布按（单元数, size）重取第一个文件的尾部标签：「TailOfAFile::of_data_units_and_size(」（`crates/singlefs-core/src/transaction.rs:12387`）。第一个文件剩 2 个单元、记录 size 仍是 3 个单元的量 ⇒ 标签取「下段根、之后到 size 是洞」（K5 的标签 3），丢掉的那一个单元成了合法的洞。

原样输出（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift_count.txt`）：

- 「records(inode,nlink,size)=[(1, 1, 97895), (2, 0, 65268)] first_file_units=2 others(inode,units)=[(2, 3)]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift_count.txt:19`）：inode 2 的 size 按批改了，单元一个没少；第一个文件少了一个单元、size 没变。
- 「OPUS|C-read-back-first-file|len=97895 equal=false first-two-units-equal=true third-unit-all-zero=true third-unit-was-all-zero=false」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift_count.txt:25`）：挂载态读者照常读出 97,895 字节，第三个单元读成全零。
- 这一版池级 checker 只报 inode 2：「inode 2 的记录 nlink = 0、size 65268，按净荷容量 32634 要 2 个数据单元，extent 树里有 3 个」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift_count.txt:24`），同一行另一条是 inode 2 的 I-4.10；inode 1 一条都没有（标签 3 许「至少 1 个、少于 3 个」）。
- 重挂续做走完之后 I-1.13 才抓到第一个文件：「第一处 key （inode 1、偏移 65268）」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift_count.txt:22`）。

四句：① 分辨——它只打「照抄第 2 条、只改取单元数那一处」的读法，任何把第一个文件那一份 `ExtentTreePlan` 与 N 的下段分开算的候选都不中；② 系统看得到（N 的下段在 `files_other_than_the_first` 里）；③ 满足的是 D8 已定项 5 第 3 条「中间状态必须每一个都是合法的文件系统状态」（`.claude/kb/decisions/08-核心索引结构.md:124`）的否定，且是静默的：判法这一侧被 K5 的洞标签接住；④ 不适用。

### U1-丙　最后一批写标签 0（推的）

D8 已定项 14 写「标签 0 全零，只有读者认、写者不写」（`.claude/kb/decisions/08-核心索引结构.md:431`），这一候选先要改已定项 14。改了之后：恢复对第一个文件之外「没有单元」的条目拒（「what: "extent 树里第一个文件之外的文件没有单元、单元有洞或尾部有洞（按 inode 号写数据写不出来，第一版不支持）",」，`crates/singlefs-core/src/recovery.rs:5233`）；池级 checker 的走读对任何 inode 收标签 0，单元数对 size 那一判只按记录逐条判（「///    标签 0 与没有上段叶条目的号一个单元都没有。」，`crates/singlefs-checker/src/walk.rs:11528`），记录已在同一次发布里删掉，这一条标签 0 条目没有判法——checker 绿、恢复红。再往下：删空判定要「extent 树零条目根（key 区间按位置规定）」（`.claude/kb/decisions/08-核心索引结构.md:485`），全删完之后上段还挂着标签 0 条目，这一版从盘上重建时永远到不了删空那个成员（内存里的判定 `crates/singlefs-core/src/transaction.rs:3470` 只看两张单元表，与盘上读回的分岔）。⇒ 「从叶里去掉」是唯一不碰已定项 14 的候选，代价落到 U2。

## U2　上段形状随删除收缩

### U2-甲　零条目非根叶、层级 ≥ 1 的零条目根：池级 checker 与核心读者判得不一致（推的）

池级 checker 读按位置寻址的节点「读节点这一步只判条目 key 严格递增、节点不空」（`crates/singlefs-checker/src/walk.rs:361`），只给 extent 上段层级 0 的根开例外：「/// 同上，读的是 extent 树上段的根：层级 0 的根兼叶可以一条条目都没有」（`crates/singlefs-checker/src/walk.rs:363`）；别的节点空着走到「if view.entries.is_empty() {」（`crates/singlefs-checker/src/lib.rs:701`）报 I-1.1。核心这一侧 `read_upper_node_and_its_subtree`（`crates/singlefs-core/src/extent_tree.rs:1130` 起）对空叶、没有孩子的内部节点不判，恢复照收。两个历史各打一格：

- 「留成零条目叶」：mkfs → 第一个文件 → 建 inode 2…150 → 写 inode 150（上段长到层 1：叶 0、叶 1、根）→ 删 150、叶 1 留成零条目叶 ⇒ checker 红、恢复绿。
- 「根不降层」再删到全空：同一前缀，删 150、删 1 ⇒ 根是层 1、零个孩子的内部节点 ⇒ checker 红；同时 D8 已定项 16 第 5 条的删空判定写「extent 树零条目根（key 区间按位置规定）」（`.claude/kb/decisions/08-核心索引结构.md:485`），没说层级，这一版算不算删空两边各读各的。

四句：① 分辨臂（「换下」与「降层」两个候选不中）；② 看得到；③ 满足第 5 条删空判定那一分句与 I-1.1 的分歧；④ 不适用。

### U2-乙　不降层 + 第一个文件那一支的断言：panic 进挂载续做（量过）

今天能走到的最近一形（`lift`，用例 `lift_only_unlinks_the_first_file_while_another_file_holds_data_units` 的 E 段）：建 inode 2…150、写 inode 150 两个单元，上段变成「others(inode,units)=[(150, 2)] upper(level,index)=[(0, 0), (0, 1), (1, 0)]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:32`）→ 删第一个文件：

- 「OPUS|E-level1-unlink-1|PANIC(第一版只有第一个文件在 extent 树里有条目：上段只有根兼叶一个节点)」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:35`），断言在「upper_root.level == 0 && previous.upper_nodes.len() == 1,」（`crates/singlefs-core/src/unlink.rs:239`）。
- 意图那次已落盘，之后可写挂载续做同一批：「OPUS|E-level1-after-remount|PANIC（可写挂载的续做里）」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:31`）——每次挂载都 panic，池卡死在可写挂载之前。

「删完 150 再删 1」若取「上段根的层级不变」（「/// 上段根的层级不变（这一批不改 inode 号的集合）。」，`crates/singlefs-core/src/unlink.rs:152` 那一句搬到最后一批）是同一个断言（推的，删 150 的最后一批今天写不出来）。四句：① 分辨（降层的候选到第一个文件那一支时上段已回到根兼叶，不中）；② 看得到；③ 第 6 条「挂载里续做到空」的否定；④ 不适用。

### U2-丙　同一个「有条目的 inode 集合」两种形状（推的；①b 没打中）

写路径取层级「upper_root_level_for(largest_inode_with_an_entry).max(previous_upper_root_level);」（`crates/singlefs-core/src/transaction.rs:6445`）——只升不降。删除若不降层，有条目集合 {1, 2} 经「写 2 → 写 150 → 删 150」到的是「层 1 根 + 叶 0」，经「写 2」直接到的是「根兼叶」。两种形状：

- 判据 ①b 不破：两份续做从同一份崩溃镜像读同一个上段、算同一份计划，形状随历史不同不进 ①b。③ 也不破：第 7 条的「逻辑内容」不含逐盘标签 1 / 2 与落点。
- 破的是第 7 条末尾那一句的可钉性：「另加 harness 层每种发布的重写角色集合钉绝对值（零轮）」（`.claude/kb/decisions/08-核心索引结构.md:487`）——同一个改动（写 inode 2）在两种形状上重写的角色集合是 {叶 0, 根} 与 {根}，绝对值跟历史走。
- 写法建议（只在我的推导里成立、被攻过零轮）：一版的上段节点集 = 每个有条目 inode 那片叶到根那一串的并，根层级 = 罩得住有条目最大号的最低一层；有条目集合为空时是层级 0 的零条目根。这一句让写、删两边同一个形状，写路径那个 `.max(previous_upper_root_level)` 在每一版都合这一句时是空操作（推的）。降层时旧根之下照抄的节点角色要按旧层级点名换下（写路径就这么写：「.map(|(position, _)| role_of_extent_upper_node(*position, previous_upper_root_level))」，`crates/singlefs-core/src/transaction.rs:6483`）；照 `plan_the_extent_tree_after_a_deletion_batch` 的写法（「let upper_root_level = previous.upper_root().level;」，`crates/singlefs-core/src/unlink.rs:189`）新旧层级不分，降层时 `ExtentRoot` 这个角色名在两版里指不同节点，旧根漏进换下清单的风险在这里（推的）。

三种终态（照上面那一句推的）：只剩第一个文件 = 层级 0 根兼叶一条条目；只剩别的一个文件 N = N 那片叶到「罩得住 N 的最低一层」根那一串（N ≥ 143 时第 0 片叶不在里面）；全删空 = 层级 0 的零条目根（与今天第一个文件删完那一形同一份字节形态）。

## U3　第一个文件之外还有文件时删第一个文件

### U3-甲　K-U5「拦」今天在「删空 → 建 inode」那一版上不拦（量过，今天的代码）

历史（`today`，用例 `today_writes_data_to_an_inode_created_after_the_pool_was_emptied`）：mkfs → 第一个文件 → 会话 unlink 1 → 建 1 个 inode → 按 inode 号顺序写它 3 个单元。

1. unlink 1 之后是删空那个成员：「OPUS|A1-after-unlink-1|member=WithTreesAndNoFile|records(inode,nlink,size)=[] first_file_units=0」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-today.txt:3`）。
2. 删空那一版放行建 inode：「UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {」（`crates/singlefs-core/src/mounted_session.rs:2052`）。建完 inode 树有一条记录，成员判定（「self.data_pointers.is_empty() && self.files_other_than_the_first.is_empty();」，`crates/singlefs-core/src/transaction.rs:3470` 与上面一行合取「inode 树零条目根」）不再成立，成员换成 `WithFile`：「OPUS|A1-after-new-inode|member=WithFile|records(inode,nlink,size)=[(2, 1, 0)]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-today.txt:1`）。
3. 会话只在删空那个成员上拒按 inode 号写（`crates/singlefs-core/src/mounted_session.rs:2058` 那一句的理由是「删空之后的一版上 inode 树一条记录都没有」，这一版已经有了）；按 inode 号写的入口只判记录在不在、是不是删除中（`crates/singlefs-core/src/transaction.rs:8337-8343`）。写做成：「member=WithFile|records(inode,nlink,size)=[(2, 1, 97895)] first_file_units=0 others(inode,units)=[(2, 3)]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-today.txt:4`），池级 checker 「OPUS|A1-checker-violations|[]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-today.txt:5`），读回相等（同文件第 7 行）。
4. 之后删它被拒：「UnlinkWhileAFileOtherThanTheFirstHoldsDataUnitsIsNotDecidedByTheClauses { inode: InodeNumber(2)」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-today.txt:9`）——这个文件在条款补上之前删不掉。

射程那一句「删完之后不再建带数据的文件（K-U5，记欠账）」（`.claude/kb/decisions/08-核心索引结构.md:489`）与 C618 的「删完之后（有树、文件集合为空那一版）只能建 size 0 的新 inode」（`.claude/kb/checks-owed.md:407`）在今天的代码上是空话：只有删空那一版本身拦，往后走一步就出了射程。

### U3-乙　判别子观测不到：两段历史同一个成员、同一份摘要（量过）

对照历史（用例 `today_writes_data_to_an_inode_that_outlived_the_first_file`）：mkfs → 第一个文件 → 建 inode 2 → unlink 1（inode 2 没有单元，拒绝那一道放行）→ 写 inode 2 三个单元。写完那一版的摘要与 U3-甲 逐字相同（`diff` 两行去掉标签之后的部分，原样输出 `identical`）：「OPUS|A2-after-write|member=WithFile|records(inode,nlink,size)=[(2, 1, 97895)] first_file_units=0 others(inode,units)=[(2, 3)]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-today.txt:12`）。

用户定的「拦」管的是「删完之后再建的 inode」。系统在写那一刻看得到的东西：成员（两段都是 `WithFile`）、inode 树（两段都是 inode 2、nlink 1、size 0）、extent 树（两段都没有条目）、队列（都空）——逐条相同；分得开的只有墓碑树里 inode 1 那条墓碑的死亡 txg 与 inode 2 记录的对象出生代谁先谁后（推的，没逐字节比两份镜像），而删空判定明写「不看墓碑树、谱系树与退役树」（`.claude/kb/decisions/08-核心索引结构.md:485`）。这是「判别子观测不到」那一形：「拦」要么改写成看得到的判据（例：「第一个文件不在树里的那几版上，按 inode 号写拒」——它连 U3-乙 这段一起拦，代价是删第一个文件之后别的文件全成只读；或「写之前比墓碑死亡 txg 与对象出生代」——要改删空判定那一句），要么承认今天的代码已经放开了 K-U5、C618 改去向。四句：① 分辨（「按成员拦」不中，上面两种改写中）；② 看不到，正是这一格；③ 满足射程那一句；④ 不适用。

### U3-丙　删第一个文件的最后一批（量过两形）

- 第一个文件与 inode 2 同在第 0 片叶（`lift` 的 D 段）：撤掉拒绝就做成，函数文档说的「零条目根」实际是「把第一个文件那一条拿掉、别的照装」（装叶按 `files_other_than_the_first` 装）：「OPUS|D-leaf0-after-remount|member=WithFile|records(inode,nlink,size)=[(2, 1, 65263)] first_file_units=0 others(inode,units)=[(2, 2)]」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:24`），checker 空、读回相等。这一形没打中；打中的是文档那一句「/// 零条目根（key 区间照旧按位置规定）。第一版只有第一个文件在 extent 树里有条目、它在第 0 片叶里，上段因此只有根兼叶那一个节点。」（`crates/singlefs-core/src/unlink.rs:229`）与行为不符。
- 上段长到层 1（E 段）：见 U2-乙，panic 并卡死挂载。

成员：删完第一个文件、别的文件还在的那一版落进 `WithFile`，`inode_record` 是空的。第一版的三个成员里没有「有树、第一个文件不在、别的文件在」这一形；今天的代码靠 `WithFile` 装它（崩溃注入那一侧已经点了这一形：`crates/singlefs-checker-tier/src/crash_injection.rs:3096`）。成员不改也行，前提是 U3-乙 的「拦」不按成员判。

### U3-丁　次序不同的两段历史（推的）

「删 1 再删 2」与「删 2 再删 1」走到删空。按第 7 条的「逻辑内容」定义，「墓碑记录多重集（含死亡 txg）」（`.claude/kb/decisions/08-核心索引结构.md:487`）两段不同（inode 1 与 inode 2 的死亡 txg 互换了先后），各条记录的改动计数也不同；逐字节相同与「逻辑内容」相同都不成立。正文那一问要的只能是去掉死亡 txg 之后的一个更弱的等价（例：活对象集合、单元集合与墓碑的 (号, 出生代) 集合），要立就得另写一个判据名，不能借第 7 条的「逻辑内容」。

## U4　续做、队列、墓碑与 checker

### U4-甲　任何一个对象删除中，扫描重建对整份镜像不判（量过）

历史（`lift`，用例 `lift_only_scan_rebuild_verdict_while_the_first_file_is_half_deleted_and_inode_two_holds_data`，批大小 1）：第一个文件 3 个单元 → 建 inode 2 → 写 inode 2 两个单元（这一版 I-1.13 成立）→ 意图删 1 → 第一批。这一版（每次发布都是一个可崩溃落到的版本）：

- 「OPUS|F-half-deleted-I-1.13|I-1.13: NotApplicable("扫描重建：墓碑压着的对象还有 inode 记录（删除中），墓碑判 key 死亡（D11 已定项 4）而遍历方向照样读得到没删完的数据单元，中间态上认不认领条款没写")」（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift.txt:39`）
- 判法在「return Err(ScanRebuildNotDecided::DeletionInProgressNotWrittenForTheScanRebuild);」（`crates/singlefs-checker/src/rebuild.rs:771`）：一个对象删除中就整张表交回不判，inode 2 的映射、单元集合、记录一格都不比。

单文件时「删除中」与「整份镜像只有被删的那个文件」是一回事，多文件之后它让别的文件在整段删除期间失去 I-1.13 这一道。U1-乙 就落在这个缝里：第一个文件丢单元的那几版里 inode 2 处于删除中，I-1.13 报不适用，别的判法被 K5 的洞标签接住，只有删除走完之后 I-1.13 才红（`research/prompts/m3-multi-file-unlink-r1-opus-model/out/opus-lift_count.txt:22`）。崩溃点重放的每个崩溃状态都跑池级 checker，停在删除中途的那些状态上这一形是绿的（推的，没跑崩溃枚举）。四句：① 不分辨候选（每个多文件删除候选都在中途经过「有对象删除中」的版本），是 I-1.13 判法本身的前提，另立一笔账：按对象不判、别的对象照判；② 看得到（被墓碑压着的只有那一个 (号, 出生代)）；③ 满足 I-1.13 报不适用那一列「删除中（墓碑压着的对象还有记录）」；④ 不适用。

### U4-乙　其余几样（推的为主）

- `deleted_inodes` 至多一条 key：没打中。守它的是会话先走空与意图入口的断言（「"发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key（D8 已定项 16 第 6 条）"」，`crates/singlefs-core/src/transaction.rs:9200`），与文件个数无关；回退到意图那一版带回的也只有那一个号。打中的是理由写错：理想模型对拍那一侧写「`deleted_inodes` 至多一条 key（只有 inode 1）」（`crates/singlefs-harness/src/model_comparison.rs:123`），多文件之后号不再只有 1，这句理由要换成「先走空」那一条。
- 挂载续做：取号前预演与真续做共用 `next_unlink_step_after`（预演在 `crates/singlefs-core/src/transaction.rs:9565`），所以 U1 的三形在续做里同样复现；E 段已量到续做 panic。
- 墓碑：意图那次按记录的对象出生代写整对象墓碑（`crates/singlefs-core/src/transaction.rs:9225`），对别的文件照样成立，没打中。U3-乙 那一判反倒要用它。
- 快照 deadlist：快照读只读第一个文件（`crates/singlefs-core/src/recovery.rs:6654`），快照下删 inode N 之后「快照里的 N 还读得回」没有读者可判；deadlist 的释放分流按角色走，`DataOfAFileOtherThanTheFirst` 角色进不进 deadlist 我没跑（推的：要一段「建快照 → 删 N → 读快照里的 N」的用例才判得了，今天连读快照里的 N 的入口都没有）。
- I-4.7：判法按号、对象取「nlink = 0 的号，加 extent 有单元而没有活记录的号」，多文件下照判，B 段量到它红；I-4.6 / I-4.10 同样按号逐条判（C 段量到它们对 inode 2 红）。I-3.4 是恒 0 的保留位，与文件个数无关。I-9.5 不读 extent 树。这几条不用改判法，要补的是取样点。
- 幂等判据取样点（推的）：① 至少一段「两个带数据文件、交错写与删」的历史；② 删 N 的每一批之后读回第一个文件逐字节相同（只看 checker 抓不到 U1-乙）；③ 上段跨层：有条目最大号在第 0 片叶之外、删它与删第一个文件两种次序；④ 删除走完之后的一版单独判 I-1.13（中途的版本它不判）。

## U5　把「只有第一个文件写数据」当前提、正文第二节没点名的地方

取法：在那棵树的 `crates/` 下 grep「只有第一个文件 / 第一版只有第一个 / 只有 inode 1 / 根兼叶」与 `FIRST_INODE_NUMBER`（core 里 48 处：transaction.rs 37、recovery.rs 5、truncate.rs 3、unlink.rs 3），逐处读，去掉正文第二节已点名的（extent_tree.rs 模块文档、transaction.rs 6426 / 9279 / 9712–9740、unlink.rs 157 / 234、恢复、截断拒绝、快照读、删空判定、扫描重建墓碑那一判）。剩下这些，多文件删除一落地就不成立：

决定删除各步走哪条分支的（改条款时必须一起改，正文没点）：
1. 「DeletionStepOfThePublish::FinalBatch { inode } if inode.0 == FIRST_INODE_NUMBER => 0,」（`crates/singlefs-core/src/transaction.rs:4986`）——只有 inode 1 的最后一批放单元（U1-甲 的病根）。
2. 「let data_units_before = previous.map_or(0, |version| version.data_pointers.len());」（`crates/singlefs-core/src/transaction.rs:6965`）与 `Batch` 那一支传 `FIRST_INODE_NUMBER`（`crates/singlefs-core/src/transaction.rs:6992`）——批截谁写死成第一个文件（U1-乙 的病根）。
3. 「/// 一版 extent 树在内存里的样子（第一版只有第一个文件有内容，下段只有它那一棵）」（`crates/singlefs-core/src/extent_tree.rs:664`）——`plan_the_extent_tree_after_a_deletion_batch` 的下段全按这一棵算，传别的 inode 号也截第一个文件的下段。
4. 「TailOfAFile::of_data_units_and_size(」（`crates/singlefs-core/src/transaction.rs:12387`）——写者按「第一个文件的单元数 × 第一个文件的记录 size」重取尾部标签；它不是错的前提，但它让任何误截第一个文件的批被洗成合法的洞（U1-乙）。
5. 「/// 这次发布之后 extent 树长什么样（第一版只有第一个文件有内容，D8（核心索引结构） 已定项 14 的两段）：」（`crates/singlefs-core/src/transaction.rs:6120`）——删除类发布的 extent 计划默认走这一支（U1-甲 第 2 步落到这里）。

已经不成立、留着会误导的前提句：
6. 「/// 第一版只有第一个文件在 extent 树里有条目：别的 inode 要改成别的 size，就要给它在 extent 树上段写一条标签 0 的条目，」（`crates/singlefs-core/src/transaction.rs:4686`）——补丁之后别的 inode 已有条目；它给的拒绝理由「要写标签 0」与 U1-丙 同一个冲突。
7. 「`deleted_inodes` 至多一条 key（只有 inode 1）」（`crates/singlefs-harness/src/model_comparison.rs:123`）——见 U4-乙。

判法与装置只罩第一个文件的（正文第五格点了文件、没点这几处）：
8. 理想模型对拍丢掉别的文件的角色：「TransactionUnit::ExtentLowerNodeOfAFileOtherThanTheFirst { .. } => return None,」（`crates/singlefs-harness/src/model_comparison.rs:92`，上一行是数据单元那一个）；模型自己「模型罩的版本里只有第一个文件有下段。」（`crates/singlefs-harness/src/model.rs:1202`），最后一批「删 inode 记录（树里只有 inode 1，叶容器一片都不剩）」（`crates/singlefs-harness/src/model.rs:188`）。
9. 崩溃注入的覆盖写只写第一个文件：「覆盖写只写第一个文件，写入口拒删完与正在删的」（`crates/singlefs-checker-tier/src/crash_injection.rs:3160`）；随机历史与崩溃枚举用例里没有一处调按 inode 号写的两个入口（grep `SequentialWriteOfAnInode|OverwriteOfAnInode|publish_sequential_write_of_an_inode` 在 `crates/singlefs-checker-tier` 与 `crates/singlefs-harness/src` 下命中 0 个文件）——多文件的写与删今天一条崩溃流都没有。
10. 坏盘输入：「第一版只有 inode 1 有 extent，extent 树上段只有根兼叶」（`crates/singlefs-checker-tier/src/bad_disk_input.rs:959`）——坏法按「上段只有根兼叶」挑节点。
11. GPU 判器与 CPU 同一条「只有层级 0 的根许空」：「// 零条目只许上段的根兼叶：第一个文件删完之后的零条目根」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2755`）与「// extent 树上段的根兼叶删完第一个文件之后可以一条条目都没有」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1891`）——U2-甲 的任何一个形状候选，GPU 两份内核要与池级 checker 一起改。
12. 场景脚手架「/// 只有第一个文件（H1、H2）。」（`crates/singlefs-harness/src/scenario.rs:290`）。
13. 会话拒按 inode 号写的理由「// 按 inode 号写同写第一个文件：删空之后的一版上 inode 树一条记录都没有，没有可写的文件。」（`crates/singlefs-core/src/mounted_session.rs:2058`）——它只罩删空那一版本身，见 U3-甲。
14. 池级 checker 只给层级 0 的根许空：「/// 同上，读的是 extent 树上段的根：层级 0 的根兼叶可以一条条目都没有」（`crates/singlefs-checker/src/walk.rs:363`）——见 U2-甲。

## 我提的改法各修哪一格（只在我的模型上量过、被攻过零轮）

| 改法 | U1-甲 | U1-乙 | U1-丙 | U2-甲 | U2-乙 | U3-甲/乙 | U4-甲 |
|---|---|---|---|---|---|---|---|
| 批与最后一批按 inode 分派：N 的单元与下段只从 `files_other_than_the_first` 那一项算，第一个文件那一份 `ExtentTreePlan` 照抄 | 推的：修 | 推的：修 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 最后一批把 N 的条目从叶里去掉（不写标签 0） | 推的：修（与上一行合用） | 不碰 | 推的：修 | 引出这一格 | 不碰 | 不碰 | 不碰 |
| 上段规范形：节点集 = 有条目号各自那一串的并，层级 = 罩得住最大号的最低一层，空集 = 层级 0 零条目根；写删同一句 | 不碰 | 不碰 | 不碰 | 推的：修 | 推的：修（第一个文件那一支改成按规范形算，断言删掉） | 不碰 | 不碰 |
| 「拦」改写成「第一个文件不在树里的版本上按 inode 号写拒」 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 推的：修甲，也拦乙 | 不碰 |
| 扫描重建按对象不判：删除中那一个 (号, 出生代) 不比，别的照比 | 不碰 | 推的：让中途那几版也抓得到 | 不碰 | 不碰 | 不碰 | 不碰 | 推的：修 |

表里没有一格是「量过」：这几个改法我都没在副本上实现。量过的只有打中那一侧（`lift`、`lift_count`、`today` 三份输出）。

## 最脆弱的三处（按排序）

1. **U1-乙：第 2 条原样套到 N，静默截第一个文件。** 只改「剩余单元数取谁」这一处（看上去就是条款要的那一句），批就截到第一个文件，丢掉的单元被写者的尾部标签规则洗成合法的洞：读者读出零、池级 checker 与恢复在删除中途都不报，只有删除走完之后 I-1.13 在旧单元还没被复用时才红。它同时踩中 U4-甲 的缝。量过（`lift_count`）。推翻它的观测：在冻结树上打同一补丁，`OPUS|C-read-back-first-file` 那一行给出 `equal=true`。
2. **U3-甲/乙：K-U5「拦」今天不拦，且照成员判不出来。** 删空 → 建 inode → 按 inode 号写，今天的代码做成、全绿，写进去的文件又删不掉；对照历史（先建 inode 再删第一个文件）走到逐字相同的摘要，判别只剩墓碑死亡 txg 对出生代，而删空判定明写不看墓碑树。这是用户刚定的那一格，最可能直接被照字面落地成「按成员拦」而白拦。量过（`today`）。推翻它的观测：`OPUS|A1-write-inode-2` 那一行不是 `Ok`。
3. **U2-乙 + U2-甲：上段收缩没有规范形，第一个文件那一支的断言与两份 checker（CPU、GPU）都只认「层级 0 根兼叶许空」。** 上段长到层 1 再删第一个文件，意图落盘之后每次可写挂载都 panic；任何「留空叶」「不降层」的候选在池级 checker 与核心读者之间分岔。量了 panic 那一形（`lift` 的 E 段），分岔那一形是推的。推翻它的观测：E 段两行不是 `PANIC`；或在核心读者里找到对非根空节点的拒绝。

## 没打中的形状

- `lift` 的 D 段：只撤拒绝，第一个文件与 inode 2 同在第 0 片叶时删第一个文件——做成、inode 2 读回相等、重挂照常、checker 空。取样：1 个起点、1 段历史、inode 2 两个单元。
- 判据 ①b：形状随历史不同不破 ①b（推的，没跑两份续做逐字节比）。
- `deleted_inodes` 至多一条 key：会话先走空与意图入口的断言在多文件下守住（推的；没造回退到别的文件意图那一版的历史）。
- 墓碑出生代：别的文件照记录写，没找到错的那一形（推的）。
- 取样总量：3 种副本读法 × 6 条用例，每条 1 段确定历史；单元数取 1–3、批大小取 1 与产品值、上段层级取 0 与 1、inode 号取 2 与 150。没放开的用户动作：删的次序（U3-丁 只推了）、删之间插写、快照、回退。

## 这条腿自己的限度

- 没跑崩溃枚举（`enumerate_crash_states`）：多文件删除今天在意图之前就拒，能枚举的只有我副本上的错误读法，枚举它们的崩溃状态只会复现上面已确定的错（B 段的孤儿、E 段的 panic）；中途版本我只取了「意图 + 一批」那一个已发布的版本（F 段、C 段）当可崩溃落到的状态。⇒ 「停在删除中途的崩溃状态上 U1-乙 全绿」是推的。
- 三种副本读法是我造的最小改法，不是正推腿或实现员会写的那一版；打中的是「原样套用」，不是任何一个认真实现的候选。
- 跑法：编一次 14 秒（只编 harness 这一个测试目标），每种读法一遍 1–3 秒（`test result` 行），不分批、没缩取样；内存包装上限 8G，前两次跑在包装的准入队列里排不进（slice 被别的会话的测试占着），停掉重排，结果以后来三次跑与 `rerun.sh` 自跑为准。
- 「量过」的数都在攻方副本上，入库装置没跑过。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没读禁读清单里的正推腿报告与模型。
- 没量：U1-丙（标签 0）、U2-甲（空叶 / 不降层的分岔）、U3-丁（次序）、快照下删 N、回退。
- 没跑任何重型测试、门禁阶段与轻型三条。没有要主 agent 跑的命令；要复核数就照开头的复跑命令跑。
- 草稿目录 `~/.local/share/singlefs/work/m3-multi-file-unlink-r1-opus/` 里的仓副本、编译目录与 `rerun.sh` 自跑目录交回前已删；留着三种读法的原始日志（`run-*.log`）与跑法脚本，复核照开头的复跑命令从模型目录重建。
