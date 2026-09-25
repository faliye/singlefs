# m2-final-code-r4 云端攻方（Opus）：Z19、Z21、Z22 找反例（2026-09-25 03:29 UTC / 12:29 JST）

读的代码是冻结副本 `/tmp/claude-1000/m2-final-code-r4/tree/crates/`（改后）与 `/tmp/claude-1000/m2-final-code-r3/tree/crates/`（改前）；kb 引文与行号取 `/tmp/claude-1000/m2-final-code-r4/kb-snapshot/.claude/kb/` 里那份文件自己的行号（D18 不在快照里，取主工作区那份，已注明）。代码行号取冻结副本。副本上跑出来的数一律是副本上的数，不是入库装置上的数。

**复跑**：`bash research/prompts/m2-final-code-r4-opus-model/rerun.sh <空的工作目录>`（线程上限 8；拷两份冻结树、改后那份打 `copy-only-core.patch`——只加三个环境变量开关，不设变量时与冻结副本同行为——放用例、逐条跑，日志落 `<工作目录>/logs/`）。我这一次的原样日志在 `research/prompts/m2-final-code-r4-opus-model/logs/`。日志是在用例文件逐步加函数的过程中跑出来的；之后改过的只有 `z19_b_…` 加了一个环境变量分支（`R4_OPUS_Z19_SKIP_AFTER_THE_PROBE`，不设时与跑 `z19-b.log`、`z19-b-no-defer-term.log` 那一版逐行同行为），别的函数一字未改；副本源码的三个开关也是逐个加的，每条日志跑的时候没设变量的开关都不起作用。

**模型目录每个文件的 sha256**（`SHA256SUMS` 原样）：

```
daa8a74b46a3cd2ff5cc5ed6f1f1df3913fecfc5c6d786d14069387cbca38514  ./copy-only-core.patch
8c7a020ae4426f178f8ac00bf9cd692a832fc2775f116cc5a178ec9c07790869  ./logs/z19-b.log
24f122809896d7ee1d437b5a8a51061db07310fe9b40c0f261a4bbf935e52a40  ./logs/z19-b-no-defer-term.log
fb5ff6843c9759c5e9d5cc6fbd99065d226e2cd03fa811a97e8f3b6c8fc6eb4c  ./logs/z19-b-skip-after-probe.log
09c641797125a4be95f7d9e253afa5bee76d328e89b579f5720455375774c5f3  ./logs/z19-c.log
86ba4ff7d831bc716d2ba812fd9cbb99265f98b94f1581c9d832f36f69e75671  ./logs/z19-r4-sample1.log
220bd229d085152f6ed3b15d6eff1b91a9eaaf93d6b38331e2b46469ea01ccb7  ./logs/z19-r4-sample2.log
4efbeb8092e0df18c2f0ef2ae0994e04c789dc81ff87509532151096fa476769  ./logs/z21-r3-control.log
7da892642d00c9f4ce6845d5403b431a4010f4c5e25d2124895df5701b83a6d0  ./logs/z21-r4-b2-c.log
72401511e456c1aad727560ae219028eab4e39a05b0596fef1fa24df8a1be272  ./logs/z21-r4-c-sample2.log
9cfb87d5fe1acfa13857e13f50566a52be3317d950e7d69ffcab45ec25864bbf  ./logs/z21-r4.log
bf541f79605a437e2d169972b0e18e40a41cdb22aa8e0c241baa80ae240ac468  ./logs/z21-r4-restore-chosen-instance-only.log
f8a06a1158487d841e7957748a16bb020038068731b0d35f162c9626e787d290  ./logs/z22-sweep.log
6f0c5fcac6ca4a743e46068629b8f183fcbe526bcdc06ce8076e52c91b0c6aad  ./logs/z22-sweep-mutation-skip-recompute.log
d8373c15993944883d0849737366a59aed2caafa4c1b58d4208c02bcd650bbc9  ./rerun.sh
d59b2b8c33222f8dbf765eb58a81b7ef26095cc6c7c5de1d6a72f00e4e7741b8  ./tests/r4_opus_z19.rs
eafdaaf49e6b52dc91c6d6503ac274c2b402d539deab9630bf0cffec473d00a5  ./tests/r4_opus_z21.rs
e00edfc85fca202d32eea868e23a0497410b50184e4de7f11f8208e38b5cbb5f  ./tests/r4_opus_z22.rs
```

## 各格判定一览

| 格 | 判定 | 一句话 | 改前（第三轮冻结树）同一历史 |
|---|---|---|---|
| Z21-B | **打中**（替没写的条款做了选择） | ⑥ 补写把实例表里每一条回退行都补回来，删除规则早已合法删掉的也补；实例表行不回收，池一生做过 24 次回退、之后任一个根槽读不出，普通可写挂载与每一条回退都在取号前报见证表写满 | 做成（量过） |
| Z21-A | **打中**（替没写的条款做了选择） | 回退到 mkfs 的第 0 代根、崩在 post 窗口：没有回退行，补写之后每一次挂载都补不上，3 / 5 / 8 个根故障就撤销回退、读回被抛弃的 C | 相同（量过，逐字） |
| Z21-C | 没打中（两次抽样） | 随机连续回退（跨实例、随机崩在 post 窗口）240 段：补写没补出错的条目、择根不落错、checker 全绿 | 79 / 120 段缺条目（量过） |
| Z19-B | **打中**（替没写的条款做了选择） | 一次会话里每一次覆盖写都被准入放行，下一次（或下下一次、之间一个字节都不写）可写挂载被准入拒；普通挂载与回退到最新根都拒，只有丢版本的回退能回到可写。三种小盘、两种内容长度同形；删掉「− defer 待释放」只把边界往后推 | 做成（量过，同一批 48 格改前全做成） |
| Z19-A | 没打中（两次抽样） | 准入放行、取号之后才被落点拒：1152 段随机历史里 0 次 | — |
| Z19 第 3 问 | 不算打中（已知岔路） | defer 里的槽扣两次，是 alloc-basis 岔路 2 的现状臂；顺带量出删项臂的代价数（副本） | — |
| Z22 | 没打中 | 没有为转绿放宽的断言；formatted_pool 第 7 读是 12 个注入点里唯一测到原性质的一个，变异下转红；一条断言成了恒真、四条改测测试开关那一臂，都有理由 | — |

攻方试的改法：Z21 只补 N = 所选根实例那一条（副本上量过，Z21-B / B2 / C 三组都过；被攻过零轮）。Z19 没有量过能修的改法。

## Z21　见证表的两处修补

### Z21-B　补写把删过的条目全补回来：一个根槽坏了，健康回退过 24 次的池再也挂不上可写（打中，改前不中）

**历史**（`r4_opus_z21.rs` 的 `z21_b_rollback_rows_resurrect_deleted_entries_when_one_ring_slot_goes_bad`，日志 `logs/z21-r4.log`）：

1. A（txg 3）→ B → C（txg 5），实例 1。
2. 连着 K 次管理员回退，每次目标取最新的根（空回退，用户定的一步），每次整次写完、一次都不崩，根环始终全读得出；两次回退之间插 w 次普通可写挂载（w = 0、1 放开扫）。
3. 最旧那条可读根的根环槽翻坏一个字节（`flip_byte`，持续读不出的一种）。
4. 普通可写挂载一次。

**许可每一步的字面**：

- 第 2 步：回退是管理员的正常操作，删除规则把能删的删掉——`23-journal的角色与格式.md:394` 原行：
> - ① **删除规则**：条目 (N, r_old, T_old) 删掉 ⟺ 根环每一个槽都读得出、都自证过，且其中没有一条根的实例代号落在 [r_old, N) 里。另外，被别的条目罩住的删：条目 (Na, ra, Ta) 被 (Nb, rb, Tb) 罩住 ⟺ Nb ≥ Na 且 (rb, Tb) ≤ (ra, Ta)（实例代号为主比）。被罩住的那条抛弃的每一条根，罩住它的那条也抛弃，删掉之后并集判法不变（主 agent 2026-09-25 定，被攻过零轮，实二五已实现）。「没抛弃任何可读的根或自证记录就删」不采纳：暂时读不出的根或记录之后又读得出时会被放回候选。
- 第 4 步挂载先补、再删：`23-journal的角色与格式.md:399` 原行：
> - ⑥ **补写**：每次可写挂载在删除规则之前，按所选根实例表里的回退行，把见证表缺的条目补回来。N 取实例表里回退行之后第一条 T ≠ 0 的行（照字面取第一个有行的实例，回退跨过实例时会取到中间实例，补出的条目罩不住那个实例的根，实二五有用例区分两种读法）；没有就是所选根自己的实例。回退挂载崩在写行根落了、轮换还没落的那一格（下一次挂载之前）补不到，那是 post 写序认的窗口（代码轮第二轮判决第四节；主 agent 2026-09-25 定，被攻过零轮，实二五已实现）。
- 代码照这句做：`mount.rs:1594` 取实例表里**每一条**回退行（`.filter(|row| row.is_rollback)`），`mount.rs:1643` 把推出来的条目并进盘上那张表，再过删除规则；根环有一个槽读不出时第一条删除规则一条都不删（`mount.rs:1545` `None => true,`），装不下就在取号之前报写满（`mount.rs:1668`、`mount.rs:1675` `.map_err(full)?;`）。
- 实例表里的行不会少：行回收（主工作区 `.claude/kb/decisions/18-块里携带什么信息.md:311`「回收」那一条）在冻结树里没有实现（`grep -n '行回收' crates/singlefs-core/src/*.rs` 只命中两行注释，`mount.rs:1535`、`recovery.rs:730`），日志里 K 次回退之后最新根的实例表恰有 K 条回退行。

**原样输出**（改后，`logs/z21-r4.log`，w = 0 那五行）：

```
Z21-B writable_between=0 rollbacks=22 ring_all_readable=true witness_entries_on_disk=8 rollback_rows_in_newest_table=22 rows=22 healthy_next_mount=Ok("Ok") | corrupted_root=(16,47) ring_all_readable_after=false -> writable_mount=Ok("Ok") disk_unchanged=false
Z21-B writable_between=0 rollbacks=23 ring_all_readable=true witness_entries_on_disk=8 rollback_rows_in_newest_table=23 rows=23 healthy_next_mount=Ok("Ok") | corrupted_root=(17,50) ring_all_readable_after=false -> writable_mount=Ok("Ok") disk_unchanged=false
Z21-B writable_between=0 rollbacks=24 ring_all_readable=true witness_entries_on_disk=8 rollback_rows_in_newest_table=24 rows=24 healthy_next_mount=Ok("Ok") | corrupted_root=(18,53) ring_all_readable_after=false -> writable_mount=Err(Recovery(RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 })) disk_unchanged=true
Z21-B writable_between=0 rollbacks=25 ring_all_readable=true witness_entries_on_disk=8 rollback_rows_in_newest_table=25 rows=25 healthy_next_mount=Ok("Ok") | corrupted_root=(19,56) ring_all_readable_after=false -> writable_mount=Err(Recovery(RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 25, capacity: 23 })) disk_unchanged=true
Z21-B writable_between=0 rollbacks=30 ring_all_readable=true witness_entries_on_disk=8 rollback_rows_in_newest_table=30 rows=30 healthy_next_mount=Ok("Ok") | corrupted_root=(24,71) ring_all_readable_after=false -> writable_mount=Err(Recovery(RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 30, capacity: 23 })) disk_unchanged=true
```

w = 1 那五行同形（K = 24、25、30 拒，22、23 做成；盘上见证只有 4 条、实例表 48 / 50 / 60 行）。改前（第三轮冻结树，同一份用例，`logs/z21-r3-control.log`）这十格全部 `writable_mount=Ok("Ok")`。

**之后还剩什么路**（`z21_b2_…`，`logs/z21-r4-b2-c.log`，K = 24）：

```
Z21-B2 writable_attempt=1 -> Err(Recovery(RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 })) disk_unchanged=true
Z21-B2 writable_attempt=2 -> Err(Recovery(RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 })) disk_unchanged=true
Z21-B2 rollback_targets_refused_with_witness_full=23 others=[]
Z21-B2 read_only_recovery -> FileRead root=(25,76) content_is_A=false content_len=2602
Z21-B2 slot_restored_same_pool -> writable_mount=Ok("Ok") witness_after=7
```

根环里 23 条可读根当回退目标，23 次全报写满；只读恢复照常。坏槽复原（没翻那个字节的同一个池）就挂得上。改前同一段：两次可写挂载、23 次回退全做成。

**四问**：

1. **分不分辨臂**：分辨。臂是「⑥ 补哪几条」：今天的「所选根实例表里每一条回退行都补」中；改前「不补」不中（但它在 Z21-C 上 120 段里 79 段中，那是第二轮 Z9-B 打中的那一格）；攻方的收窄臂「只补 N = 所选根实例那一条」不中（见下面改法表）。
2. **系统当时看不看得到判别它的东西**：看得到一个代理。盘上缺一条条目有两种来路：被删除规则合法删了，或回退那次挂载崩在 post 窗口里没写上。后一种只能是所选根自己的实例 N（N 的写行根落了、N 的挂载没走到轮换）；N 小于所选根实例的，之后那个实例取号的两次写（`mount.rs:1668` 那张 `before_the_row_publish`）已经带过它，盘上没有只能是删的。这一条是推的，Z21-C 的随机段（下一节）量过它不漏补。
3. **满足的是哪一句的哪一个分句**：代码照 ⑥ 的字面做；「缺的条目」没说是「从没写上的」还是「删过的」——属**替没写的条款做了选择**。结果落在 ② 的字面里（`23-journal的角色与格式.md:395` 原行见下），盘上不变；写满之后怎么办没定，见 C547（回退见证表的删除规则与写满没有条款）。`），但前提变了：② 与 C547 数的是「槽坏着的时候发生的回退」，这里数的是**池一生里做过的回退**——槽好好的时候早已删掉的条目，槽一坏全部回来。第 380 行（原行见下）把条数上限写成只由根环几何定，补写之后决定条数的是实例表里的回退行数，行不回收就没有上界。

   `23-journal的角色与格式.md:395` 原行：
> - ② **写满**：有槽持续读不出、或每次回退都崩在写行轮换之后暖机之前时，条目删不掉，写满时在取号之前拒绝挂载（`RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided`），盘上不变；写满之后怎么办没定，见 C547（回退见证表的删除规则与写满没有条款）。

   `23-journal的角色与格式.md:380` 原行：
> **回退见证**（C332（回退实例两个根都读不出时回退被撤销） 的修法）：回退在系统配置槽里记一张见证表，放在系统配置字段表之后、越过 512 字节，靠系统配置的整槽校验和与两槽轮换认撕裂（越过 512 撕开的那一槽作废，等于这次写没持久）。一个条目记一次回退：新实例代号 4 字节、R_old 的实例代号 4 字节与 txg 8 字节。根 (i, T) 被条目 (N, r_old, T_old) 抛弃 ⟺ (r_old, T_old) < (i, T)（实例代号为主比）且 i < N；择根先跳过被任一条目抛弃的根，再照 D22（单元原子性怎么合成） 已定项 7 择新。见证随回退那一次挂载写行之后的第一次系统配置轮换写，之后每一次系统配置写都带着整张表。条数上限 = 根环槽数减 1（R × S − 1，只由根环几何定）。每次回退都崩在写行轮换之后、暖机之前，连着 R × S 次，根环全读得出也写得满（代码轮第二轮判决第三节）；写满时的处置见「回退见证的实现取法」②。见证表读不出就是系统配置槽读不出，挂载报错，不静默择根。字段落点随实现写进 D22（单元原子性怎么合成） 已定项 9 的字段表与 [layout/02-second-txn.md](../layout/02-second-txn.md)；条目什么时候删随实现，交代码三方。

4. **跑前条款给的改法在这几格上还中不中**：第二轮判决第四节第 3 条（补写）就是引入它的那一改，没有给别的改法；C547 只写「写满之后怎么办仍没定」。

**攻方试的改法**（只在我的副本上量过、被攻过零轮）：⑥ 只补 N = 所选根实例的那一条（副本开关 `R4_OPUS_RESTORE=chosen-instance-only`，补丁 `copy-only-core.patch`）。

| 格 | 今天（改后） | 改前 | 只补 N = 所选根实例 |
|---|---|---|---|
| Z21-B 十格（K = 22..30，w = 0、1） | K ≥ 24 的六格拒（量过） | 十格全做成（量过） | 十格全做成（量过，`logs/z21-r4-restore-chosen-instance-only.log`） |
| Z21-B2（坏槽之后两次挂载、23 条回退） | 全拒（量过） | 全做成（量过） | 全做成（量过） |
| Z21-C 随机回退 120 段（种子基 91000） | 0 段错（量过） | 79 段错（量过） | 0 段错（量过） |
| 第二轮 Z9-B 那一格（回退挂载崩在 post 窗口，之后 m = 1..3 次挂载） | 修好（库里用例钉着） | 中 | 推的：修好——那一格 N 就是所选根的实例；Z21-C 里 225 次崩在这一窗口的回退都覆盖到了 |
| Z21-A（回退到第 0 代根） | 中 | 中 | 中（量过） |

**推翻条件**：在冻结副本上，同一段历史（24 次健康回退、一个根槽坏）普通可写挂载做成——说明写满不是补写造出来的，本节作废。

### Z21-A　回退到 mkfs 的第 0 代根、崩在 post 窗口：补写永远补不上（打中，改前改后相同）

**历史**（`z21_a_rollback_to_the_generation_zero_root_crashing_in_the_post_window_is_never_restored`）：A → B → C（txg 5），实例 1；管理员回退到 (0, 0)（库里有这一格的正常用例 `a_rollback_to_the_make_filesystem_root_holds_the_witness_table_invariant_without_a_row_for_instance_zero`）；回退挂载崩在写行那次的根 FUA 落了、轮换没落；之后普通可写挂载 m 次（m = 0..=3 放开扫）；最后让比 C 新的根全读不出，恢复。对照：回退挂载整次写完。

**许可每一步的字面**：回退目标 (0, 0) 在候选集里（库里的用例）；崩的那一格是 ⑥ 认下的 post 窗口；之后的挂载是用户照常做的事。
代码自己在 `mount.rs:1587` 写着推不出来（原行：`/// 回退到 mkfs 的第 0 代根（r_old = 0）不写行（D18（块里携带什么信息） 已定项 11：实例 0 不写行），实例表里没有回退行，这一条推不出来。`）；回退那一次写的行是中间实例行 (1, 0, 0)，不是回退行（`mount.rs:1470` `let first_row_instance = previous_row.instance.0.max(1);`，实例 0 不进行区间）。

**原样输出**（改后，`logs/z21-r4.log`）：

```
Z21-A m=0 crash_between_row_root_and_rotation=true witness=[] newest=(2,6) rows=[(1,0,-)] unreadable_root_txgs=[6] faults=1 -> FileRead root=(1,5) content_is_A=false content_len=2602
Z21-A m=1 crash_between_row_root_and_rotation=true witness=[] newest=(3,8) rows=[(1,0,-) (2,6,-)] unreadable_root_txgs=[6, 7, 8] faults=3 -> FileRead root=(1,5) content_is_A=false content_len=2602
Z21-A m=2 crash_between_row_root_and_rotation=true witness=[] newest=(4,10) rows=[(1,0,-) (2,6,-) (3,8,-)] unreadable_root_txgs=[6, 7, 8, 9, 10] faults=5 -> FileRead root=(1,5) content_is_A=false content_len=2602
Z21-A m=3 crash_between_row_root_and_rotation=true witness=[] newest=(5,13) rows=[(1,0,-) (2,6,-) (3,8,-) (4,10,-)] unreadable_root_txgs=[6, 7, 8, 9, 10, 11, 12, 13] faults=8 -> FileRead root=(1,5) content_is_A=false content_len=2602
Z21-A m=3 crash_between_row_root_and_rotation=false witness=[(2, 0, 0)] newest=(5,16) rows=[(1,0,-) (2,7,-) (3,10,-) (4,13,-)] unreadable_root_txgs=[6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] faults=11 -> NoFile { root: (InstanceGeneration(0), CheckpointTxg(0)) }
```

m ≥ 1 之后见证表一直是空的，3 / 5 / 8 个根故障就把回退静默撤销、读回被抛弃的 C（2602 字节）；不崩的对照故障更多也落在 (0, 0)。改前树上这八行逐字相同（两份日志的 `Z21-A` 行排序后 md5 同为 `c0b26d22dcf452b83a3c5438a9270851`）。这正是第二轮 Z9-B 打中的形态，补写修好了 r_old ≥ 1 的那几格，r_old = 0 这一格照旧。

**四问**：

1. **分不分辨臂**：不分辨「补写 / 不补写 / 只补所选根实例」三臂（三份日志这一格同为中）。分辨的是「补写从哪推」：只认回退行的推法都中；从行形推（见下）的不中（推的）。
2. **系统看不看得到**：看得到一部分。所选根实例表开头是一串 (i, 0, 0) 行、前面没有任何实例的行，就是「上一次择到的是实例 0 的根」；它与「实例 1 取号之后烧掉、第 0 代根上再挂」写出的行形相同，但两种情形推出的条目 (N, 0, 0) 抛弃的都是实例表本来就判为被抛弃的根（(i, 0, 0) 行的实例的每一条 T > 0 的根），补上它不改变实例表那一侧的候选集。推的，没实现。
3. **满足的是哪一句**：⑥ 的最后一句只认「下一次挂载之前」那一格补不到；这一格在之后每一次挂载都补不到，代码注释自认、kb 没写——属**替没写的条款做了选择**（条款没说 r_old = 0 时补写从哪推）。
4. **跑前条款给的改法**：第二轮判决第四节第 3 条的补写在这一格上照样中（量过）。

**推翻条件**：冻结副本上 m ≥ 1 那几格见证表里出现 (2, 0, 0)，或恢复落到 (0, 0)。

### Z21-C　随机回退段：补写没有补出错的条目（没打中，两次抽样）

**做法**（`z21_c_random_rollback_series_with_crashes_in_the_post_window_restore_exactly_the_intended_entries`）：每段先覆盖写到 txg 5..10；随后 1..6 次回退，目标在根环里随机挑（不挑第 0 代根，那一格归 Z21-A；不是候选的换下一条），每次一半概率崩在 post 窗口；两次回退之间随机插 0..2 次普通可写挂载；最后整次写完一次普通可写挂载。
判：盘上每一条条目都是某次回退本来要写的那一条（没有错的 N、没有错的目标）；本来要写而盘上没有的，要么被盘上一条罩住，要么按第一条删除规则删得掉；池级 checker 一条不红；最后那次挂载做成。

| 抽样 | 树 | 段数 | 回退次数 | 崩在 post 窗口 | 不合格段 |
|---|---|---|---|---|---|
| 种子基 91000 | 改后 | 120 | 426 | 225 | 0 |
| 种子基 92000 | 改后 | 120 | 418 | 207 | 0 |
| 种子基 91000 | 改前（对照） | 120 | 426 | 225 | 79（全是「缺条目、删不掉」） |
| 种子基 91000 | 改后 + 只补所选根实例（副本） | 120 | 426 | 225 | 0 |

原样汇总行：

```
Z21-C summary series=120 base=91000 rollbacks=426 crashed_in_post_window=225 intended_entries_present_at_end=213 bad_series=0
Z21-C summary series=120 base=92000 rollbacks=418 crashed_in_post_window=207 intended_entries_present_at_end=220 bad_series=0
Z21-C summary series=120 base=91000 rollbacks=426 crashed_in_post_window=225 intended_entries_present_at_end=182 bad_series=79
Z21-C summary series=120 base=91000 rollbacks=426 crashed_in_post_window=225 intended_entries_present_at_end=213 bad_series=0
```

（第 1、3、4 行出自 `logs/z21-r4-b2-c.log`、`logs/z21-r3-control.log`、`logs/z21-r4-restore-chosen-instance-only.log`，第 2 行出自 `logs/z21-r4-c-sample2.log`。）

这一格回答了正文 Z21 的第 2、3 问：在连续回退、回退跨实例、每次崩或不崩的这些段上，补写取 N 的读法没补出错的条目，择根没落错（checker I-7.10 / I-7.11 / I-3.1 全绿）。第 1 问（被罩住的删会不会删掉仍护着根的条目）：罩住的删在这 844 次回退上没让任何一段不合格；它的并集判法不变是等式（`mount.rs` 那段注释的证明），我没另造反例。

## Z19　空间准入接线与需求的读法

### Z19-B　一次会话里每一次覆盖写都被准入放行，下一次可写挂载却被准入拒；之后只有丢掉版本的回退能回到可写（打中）

**历史**（`r4_opus_z19.rs` 的 `z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount`，准入判着，也就是产品路径；每一步之后跑池级 checker）：

1. mkfs 同一个进程里发完第一个文件，关掉、可写挂载。
2. 覆盖写 k 次（k 由用户定，从 0 扫到这一次会话里准入一直放行到的那一次 k_max；内容 3000 字节与 100 字节两种）。
3. 关掉、可写挂载；再关掉、可写挂载（中间一个写都没有）。
4. 逐个回退到根环里第 0..23 新的根；冷启动恢复；再可写挂载。

**许可每一步的字面**：第 2 步每一次覆盖写都是准入放行的（`session_all_admitted=true`）；挂载只要逐盘可用 ≥ 0（`28-挂载期承诺量.md:32` 原行见下）；写行、暖机不判（`28-挂载期承诺量.md:34` 原行见下），代码 `transaction.rs:4564` 起那一段只在有普通分配时判，`mount.rs:1723` 挂载判需求 0。

`28-挂载期承诺量.md:32` 原行：
> - 可写挂载在 `mount::establish_instance` 里判：取号与预演之前，这一刻的需求逐盘记 0。

`28-挂载期承诺量.md:34` 原行：
> - 需求只算普通分配（数据单元、extent 树与 inode 树的节点）；没有普通分配的发布（空发布、写行、暖机、抬 F）不判。读数取分配器此刻的计数。

**原样输出**（`logs/z19-b.log`，每种盘宽 3000 字节那一组的最后两格；100 字节那一组逐格同形）：

```
Z19-B width=240 selector=2999 first_mount=Applied overwrites_admitted_in_one_session=5 next_step=Some("PublishError::SpaceAdmissionRefused")
Z19-B width=256 selector=2999 first_mount=Applied overwrites_admitted_in_one_session=6 next_step=Some("PublishError::SpaceAdmissionRefused")
Z19-B width=384 selector=2999 first_mount=Applied overwrites_admitted_in_one_session=10 next_step=Some("PublishError::SpaceAdmissionRefused")
```

| 盘宽（单元区槽） | k | 这一次会话每次覆盖写都放行 | 下一次可写挂载 | 再下一次 | 回退 24 次里做成几次 | 改前（准入关掉）下一次 / 再下一次 |
|---|---|---|---|---|---|---|
| 240 | 3 | true | 做成 | 做成 | 5/24 | 做成 / 做成 |
| 240 | 4 | true | 做成 | 准入拒 | 6/24 | 做成 / 做成 |
| 240 | 5 | true | 准入拒 | 准入拒 | 6/24 | 做成 / 做成 |
| 256 | 4 | true | 做成 | 做成 | 5/24 | 做成 / 做成 |
| 256 | 5 | true | 做成 | 准入拒 | 5/24 | 做成 / 做成 |
| 256 | 6 | true | 准入拒 | 准入拒 | 6/24 | 做成 / 做成 |
| 384 | 8 | true | 做成 | 做成 | 5/24 | 做成 / 做成 |
| 384 | 9 | true | 做成 | 准入拒 | 6/24 | 做成 / 做成 |
| 384 | 10 | true | 准入拒 | 准入拒 | 7/24 | 做成 / 做成 |

（表只列 3000 字节那一组；两组合计 48 格，准入判着时下一次或再下一次挂载被拒的 12 格，全在 k = k_max − 1、k_max 上；改前那一臂 48 格里 0 格被拒。）

逐步（`z19_c_step_by_step_after_the_last_admitted_overwrite`，`logs/z19-c.log`，240 槽、k = 4 那一段；回退序号 0 就是最新那条根，回退到它不丢版本）：

```
Z19-C width=240 k=4 step=5 op=CloseAndMountWritable -> Applied newest_root_after=(3,11)
Z19-C width=240 k=4 step=6 op=CloseAndMountWritable -> MountError::SpaceAdmissionRefusedBeforeAcquisition newest_root_after=(3,11)
Z19-C width=240 k=4 step=7 op=CloseAndMountWritable -> MountError::SpaceAdmissionRefusedBeforeAcquisition newest_root_after=(3,11)
Z19-C width=240 k=4 step=8 op=CloseAndMountRollback(RingRoot { index_from_newest: 0 }) -> MountError::SpaceAdmissionRefusedBeforeAcquisition newest_root_after=(3,11)
Z19-C width=240 k=4 step=9 op=CloseAndMountRollback(RingRoot { index_from_newest: 1 }) -> MountError::SpaceAdmissionRefusedBeforeAcquisition newest_root_after=(3,11)
Z19-C width=240 k=4 step=10 op=CloseAndMountRollback(RingRoot { index_from_newest: 2 }) -> Applied newest_root_after=(4,13)
```

k = 4 那一段：第一次重挂做成（取号 3、写行、暖机），紧接着第二次重挂——之间用户一个字节都没写——被拒，之后普通挂载与回退到最新根都被拒，只有回退到更旧的根（丢掉至少一版）做成。k = k_max 那一段（240 槽 k = 5、256 槽 k = 6）第一次重挂就被拒，同样要回退到更旧的根才可写。三种盘宽、两种内容长度都是这一形：k_max 那一格下一次挂载拒，k_max − 1 那一格下下一次挂载拒。

**改前**：同一批 k 的历史把扫 k 那几段的准入关掉（只供测试的开关，改前两处都不判准入），48 格里下一次与再下一次可写挂载全部做成、每一步之后 checker 判绿（`logs/z19-b-skip-after-probe.log`）。

**四问**：

1. **分不分辨臂**：不分辨正文点名的那几个读法，也不分辨 alloc-basis 岔路单第 2 行的两臂（`research/prompts/alloc-basis-forks.md:10`，式子删不删「− defer 待释放」）。删掉那一项之后（副本开关 `R4_OPUS_ADMISSION=no-defer-term`，`logs/z19-b-no-defer-term.log`）一次会话放行到的次数从 5 / 6 / 10 涨到 11 / 11 / 21，形状照旧：k_max 那一格下一次挂载拒，k_max − 1 那一格下下一次挂载拒（量过）。两臂共用的前提是：挂载那一刻逐盘要求「连这次挂载的切换预留、保留池都扣完之后 ≥ 0」，而上一次会话自己的固定点、上一次挂载自己的写行与暖机都实占了槽，占完之后下一次照样再扣一整份——一次放行的会话不保证下一次挂载放行。按「判据自己也会写错」那一条，这一格要另立一笔账，不拿它判岔路 2。
2. **系统看不看得到**：看得到。最后一次覆盖写准入的那一刻，分配器上的计数加上这次发布自己的固定点与写行、暖机的量，就能算出下一次挂载的读数；准入不做这一步。
3. **满足的是哪一句**：代码照 `28-挂载期承诺量.md:32`、`:34` 的字面做。条款没写「一次放行的会话之后，下一次挂载必须放行」；已定项 3 说多的那一份给写行那次发布（`28-挂载期承诺量.md:85` 原行见下），射程只在「近满盘 + 坏扇区」「根槽持续读不出」两种情形下提到只读吸收态（`28-挂载期承诺量.md:89` 原行见下）。这里没有坏扇区、根环全读得出，接连两次什么都不写的重挂就把池翻成只读，只有丢版本的回退能出来——属**替没写的条款做了选择**，射程漏了这一形。它不是 C542 那一族（没有一次取号之后的落点拒，两次抽样的随机历史里 `PlacementRefused` 与取号之后的 `MountError::Publish` 都是 0 次，见 Z19-A）。
4. **跑前条款给的改法**：没有。岔路 2 的删项臂量过，不修（只把边界往后推）。攻方没有实现别的改法；一个推的方向：覆盖写的准入需求里加上这次发布自己的固定点数，使「放行 ⇒ 发布之后可用仍 ≥ 0」成立（推的，没实现、没跑，被攻过零轮）。

`28-挂载期承诺量.md:85` 原行：
> - **N_switch + 1 份**：N_switch = 3（D23（journal 的角色与格式） 已定项 14）；多的一份给写行那次发布的元数据——写行那次是新实例的第一次发布，之前不推抬 F 的空发布，同一次发布里的用户数据重做照走准入、不够返回 ENOSPC。N_switch 那几份是逐次累加的上界：连续切换的前提是上一次的根没发布、重建态从同一个根来，上一次的链页在重建态里不存在，实际不累加。

`28-挂载期承诺量.md:89` 原行：
> **射程**：式子按「两份副本各落一块盘」写，只对两块盘的池没有歧义（D2（RAID 条带策略） 已定项 9 的第一个可运行目标）；**三块盘以上时两份落哪两块盘没写**，挂载判定该要求「任意两块」「空闲最多的两块」还是「分配器会选的那两块」都没定，空发布落哪块盘同样没写。第一版不跑 zoned，zoned 上 rows0 无界（C121（zoned 上实例表链长无上界））；非 zoned 上 rows0 由行回收压住，但回收要「无读失败 ∧ 全盘在线」且删行要可写挂载，近满盘 + 坏扇区的池上「预留拿不到 ⇒ 只读」可能成吸收态（推的，每 369 行才多一片）；每次可写挂载都写行、行回收要根环每个槽都读成功之后，一个根槽持续读不出时行只增不减，这个吸收态不再要求近满盘（C335（根槽持续读不出时实例表只增不减））。D16（发布语义） 已定项 5 只写 T_dirty 是触发条件、没写重做期间开放的 checkpoint 会不会堵住写者，那是 D16（发布语义） 的口径空白。

**推翻条件**：冻结副本上 240 槽、k = 4 那段历史的第二次重挂做成；或者条款里找到一句明写「挂载接连被准入拒、只能靠丢版本回退出来」是认下的代价。

### Z19-A　准入放行、取号之后才被落点拒（没打中，两次抽样）

**做法**（`z19_small_pool_campaigns_with_the_space_admission_judged`）：库里的随机历史执行器，准入判着、每一步之后跑池级 checker 与模型对拍；盘宽 240 / 256 / 384 × 比重四种（`BROAD`、`TOWARD_THE_UNIT_AREA_WALL`、`REUSE_AFTER_RAISING_THE_FLOOR`、`ROLLBACK_AFTER_RAISING_THE_FLOOR`）= 12 格，每格 48 段 × 120 步；两个种子基 7040000000、7050000000（与库里的 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE` 不同），共 1152 段。

两次抽样合计的拒绝成员（`logs/z19-r4-sample1.log`、`-sample2.log` 逐格相加）：

| 成员 | 次数 |
|---|---|
| `MountError::SpaceAdmissionRefusedBeforeAcquisition` | 12044 |
| `PublishError::SpaceAdmissionRefused` | 6852 |
| `MountError::RollbackFloorAboveCeiling` | 2832 |
| `MountError::RollbackTargetNotACandidate(NotInRing)` | 1744 |
| `MountError::RollbackTargetNotACandidate(OnAbandonedTimeline)` | 1498 |
| `PublishError::ContentExceedsDataUnit` | 2300 |
| `PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile` | 406 |
| `MountError::RollbackTargetNotACandidate(BelowEffectiveFloor)` | 91 |

没出现的：任何 `PlacementRefused`、`MountError::Publish(…)`（取号之后发布失败）、`RaiseFloorSequencePublishFailed`、`RowPublishAdmissionRefusedBeforeAcquisition`、`WarmUpAdmissionRefusedBeforeAcquisition`、`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`。新发现 0；以已知红收尾的 10 段全是「已知红」清单第 0 条（增补 2 收口表第 43 行，与准入无关）。每格一行的原样输出在两份日志的 `Z19 width=` 行。

普通可写挂载一共试了 16277 次、被拒 10333 次（`operation` 行逐格相加：`CloseAndMountWritable applied=5944 refused=10333`；准入在普通挂载与回退挂载两处合计拒 12044 次）——小盘上池大半时间挂不上可写，与 Z19-B 同一形。

**读法**：两次抽样都没打中，按三方规则记「没打中」。它说明的只是：在这几种盘宽与比重上，准入判着时式子总是先于落点拒；C545 那一格（准入放行而固定点仍取不到）一次都没走到。

### Z19 的第 3 问：读数的口径

- 「已分配」取分配器的 `allocated_slots()`（`admission.rs:323`），它在释放进 defer 时不减（`allocator.rs:301` 只加 `deferred_slots`），式子又减一次「defer 待释放」（`admission.rs:369`）：defer 里的槽扣两次。这是 alloc-basis 岔路单第 2 行的「第一轮双扣」，岔路开着、条款式子里这一项还在（`28-挂载期承诺量.md:19`），实现照字面扣，不算新打中。`second_transaction_supplement_two_root_ring_turn_in_one_mount.rs` 那段注释自己也写了「defer 按读法甲扣两次」。
- 顺带量出这条岔路的一个代价数（副本上，量过）：删掉那一项，单元区 240 / 256 / 384 槽的小盘上一次会话里准入放行的覆盖写从 5 / 6 / 10 次变成 11 / 11 / 21 次（`logs/z19-b.log` 与 `logs/z19-b-no-defer-term.log` 的 `overwrites_admitted_in_one_session`）。只在我的副本上量过。
- 被抛弃根独占量取 `isolated_slots()`、切换预留按分配器上记的 rows0、ckpt_cost 按 Σ 名单，这三项我逐行读过、与 `28-挂载期承诺量.md` 已定项 1、3、4 的口径一致；算术那几格归本地攻方，我没算数。

## Z22　三处红转绿与改过的测试（逐条对改前改后）

改前取 `/tmp/claude-1000/m2-final-code-r3/tree/crates/`，改后取冻结副本。结论：**没有一条是为了转绿放宽断言**；有两处值得记：一条的最后一个断言变成了恒真（判别力搬到了新加的断言上，不算放宽），四条用例测的对象从产品路径换成了测试开关那一臂（有理由，产品路径在小盘上走不到那一格）。

| 用例 | 改了什么 | 原性质还测不测 | 依据 |
|---|---|---|---|
| `formatted_pool` 的 `transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write` | 注入点第 2 读 → 第 7 读（改前 `…formatted_pool.rs:87` 写 `(2)`，改后 `:73` 常量 7） | **测**（量过） | 下面的扫描 |
| `checker_known_bad_images` | 补 I-7.10、I-7.11 两份坏镜像与一条「各自只红哪几条」的用例；补空叶挂在根下只红 I-1.1 的用例 | 测，只增不减 | 改前那几份坏镜像与断言一行没删（diff 里这份文件只有 `+` 行与一处 `use` 合并） |
| `step_four_rollback` 两条改名 | 恢复那一步改断言落 R_old (1, 3) 读回 A；影子账那一半改成沿 C 的根直接走读（`walk_to_the_file_under_the_abandoned_root_c`，改后 `:870`） | 测影子账那条性质；「恢复会退到 C」那一格不再在这里测 | 见下 |
| 加 `SpaceAdmission` 开关的四条（随机历史的单元区墙取样、240 槽挂载预演取不到落点、384 槽回退到最旧根、根环一次挂载转一圈的小盘段） | 关掉准入 | 测的仍是「准入放行之后落点那一道兜底」，但走的是测试开关那一臂 | 见下 |
| `commit_generated_fallback` 抬 F 那一条改名 | 断言从「扣住位留在进程里、空闲 > 0」反过来改成「分配器与抬 F 之前逐项相同」 | 性质本身按 C546 的修法改了方向，新断言更严 | 见下 |
| `rollback_witness` 写满那一条 | 23 条造出来的条目从 (1, 0, k) 换成 (k + 3, 1, k + 5)（改后 `:423`） | 测 | 旧的 23 条被新的罩住规则删得只剩一条，表写不满，只能换；新的 23 条彼此罩不住、第一条删除规则也删不掉，这一点我按条目逐条对过 |
| `release_checksum_quarantine` | 加用例 12、13；旧用例只改注释与一个帮手函数的参数 | 测，只增不减 | diff 里旧断言没有 `-` 行 |
| `bad_disk_input`、`crash_injection`、`fault_injection` | `HistoryExecution` 多一个字段，填「判着」 | 测（4 GiB 盘上准入不会拒） | — |

### formatted_pool：注入点扫 1..12，加一条变异对照

`r4_opus_z22.rs` 原样照这条用例的历史（mkfs 之后取号 1 就停，重开时每块盘第 n 次读系统配置槽 0 报一次读错），n 从 1 扫到 12；副本变异 `R4_OPUS_MUTATION=skip-recompute`（`acquire_expected_instance` 不重算、照判定的号写，即 mutations.tsv 里「取号不核判定的号」那一类）下再扫一遍。原样输出（`logs/z22-sweep.log`、`logs/z22-sweep-mutation-skip-recompute.log`，只贴 n = 2、7、9）：

```
Z22 mutation="" n=2 -> Ok(instance=2) disk_snapshot_unchanged=false original_test_assertions_hold=false
Z22 mutation="" n=7 -> InstanceGenerationChangedBeforeAcquisition{expected:1,recomputed:2} disk_snapshot_unchanged=true original_test_assertions_hold=true
Z22 mutation="" n=9 -> InstanceGenerationChangedBeforeAcquisition{expected:2,recomputed:1} disk_snapshot_unchanged=true original_test_assertions_hold=false
Z22 mutation="skip-recompute" n=7 -> Ok(instance=1) disk_snapshot_unchanged=false original_test_assertions_hold=false
```

12 个 n 里只有 n = 7 让原用例的断言成立（其余 10 个 `Ok(instance=2)`，n = 9 是反方向的拒）；变异下 n = 7 变成 `Ok(instance=1)`——取号写下了一个已经烧过的号，原用例在这里红。所以改后的用例测的仍是「判定与重算不一致就不写」，注入点换了没有放宽；它是一个脆的数（再多一读就要重数），但数错时是红、不是悄悄绿（1..6、8、10..12 都让原断言不成立）。

### step_four_rollback 两条

改前断言「四条根读不出 ⇒ 恢复落 C (2, 8)：影子账开着读回第三次内容、关着读不回」（改前 `…step_four_rollback.rs:935` 那一句）。回退见证之后恢复不再落 C，改后先断言落 R_old，再沿 C 的根直接走读。

- 影子账那条性质照样测：开着 `assert_eq!(…, Ok(Some(third_content())))`，关着 `assert_ne!`，与改前同一种强度（改前关着那一臂也只是 `assert_ne!`）；mutations.tsv 里「影子账一个槽都不隔离」那一行跟着改了名，仍点这条（我没跑库里的这个二进制，这一句是读出来的）。
- 少了的是「恢复真的会落到 C」这一端到端的一格；它现在只在 post 窗口里出现（第二轮 Z9-B 的 m = 0，与本报告 Z21-A 的 r_old = 0 那几格）。Z21-A 那几格恰好就是端到端的「恢复落 C、读回 C 的 2602 字节」，影子账开着（`logs/z21-r4.log`）——读回的是完整的 C，说明影子账在那一格上护住了 C 的单元（只量了开着那一臂）。

### 加 SpaceAdmission 开关的四条

四条都改成 `SpaceAdmission::SkippedByTheTestOnlySwitch`（随机历史 `:384`、`:595`、`:723`，根环一圈 `:209`）。改前没有准入，它们测的是产品路径上的落点兜底；改后产品路径在这几块小盘上式子先拒、走不到落点那一道，于是换成测试开关那一臂。不算放宽：被测的代码路径（落点拒绝在任何写之前返回）与断言都没变；另为单元区墙那一条加了一条准入判着的对照（`unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_…`）。另外三条没有准入判着的对照：240 槽预演、384 槽回退到最旧根、根环一圈，在产品路径上的对应格只能靠 4 GiB 的用例与随机历史。

### commit_generated_fallback 那一条

改后 `:428` 的最后一个断言（`try_allocate_commit_generated` 报 `NoFreeSlotOnAnyDevice`）在改后已经恒真：抬 F 之前用例自己把每块盘的空槽占光（改后新加的 `free_slots_before_the_raise == [0, 0]`），换回与不换回分配器两种实现都取不到落点——注释也改成了「每块盘上本来就一个空槽都没有」。判别力搬到了新加的两条断言上（空闲槽数与抬 F 之前相同、`Debug` 逐项相同，改后 `:417`），mutations.tsv 里「C546：不换回」那一行点的就是这条用例。不是放宽；那一句断言可以删掉或改成判别得出的形式（推的）。

**推翻条件**：`formatted_pool` 那一格在变异下仍绿；或在 `checker_known_bad_images`、`release_checksum_quarantine`、`step_four_rollback` 的改前改后 diff 里找到一条被删掉或放宽的旧断言。

## 没打中的形状

- **Z21 被罩住的删删掉仍护着根的条目**：Z21-C 两个种子基 240 段、844 次回退（432 次崩在 post 窗口）里没有一段择根落错、checker 没红。没另造「各盘各槽一新一旧」的逐槽撕裂枚举（崩点只取 post 窗口一处）。
- **Z21 补写取 N 在跨实例、连续回退时补错**：同上，0 条错的条目；中间实例、回退到回退实例自己的根、回退到被别的回退抛弃过的根都在随机段里出现过（日志 `steps=` 列）。
- **Z19 准入放行之后在取号之后被落点拒（C542 之外）**：12 格 × 48 段 × 120 步 × 两个种子基；落点拒一次都没出现。取样只在三种小盘上，4 GiB 没扫。
- **Z19 空发布、写行、暖机、抬 F 不判准入 ⇒ 它们自己取不到落点**：同一批随机历史里 `RaiseFloorSequencePublishFailed`、`WarmUpAdmissionRefusedBeforeAcquisition`、`RowPublishAdmissionRefusedBeforeAcquisition` 都是 0。
- **Z22 各条**：见 Z22 一节的表；formatted_pool 那一格量过，其余是读 diff 得出的。

## 这条腿自己的限度

- 副本上的数不算入库装置上的数；两个开关（`R4_OPUS_RESTORE`、`R4_OPUS_ADMISSION`）与一个变异开关（`R4_OPUS_MUTATION`）只在副本里，补丁 `copy-only-core.patch`，不设变量时与冻结副本同行为（Z21 / Z22 / Z19 的「改后」数都是不设变量跑的）。
- Z21-B 的故障只用了「根环槽翻一个字节、持续读不出」；一次挂载里瞬时读不出一个根槽会不会同样让那一次挂载报写满，是推的（删除规则读根环用的是同一个 `every_root_ring_slot_holds_a_root`），没量。
- Z21-A 之后用户那几步只放开了「普通可写挂载 m 次」，没扫「写第一个文件、覆盖写」这类之后的操作。
- Z21 的「只补所选根实例」为什么不漏补，是推的（取号那两次写带着补过的表）；量过的只有 Z21-C 种子基 91000 那 120 段与 Z21-B / B2。
- Z19-B 只扫了单文件、单内容长度的覆盖写（3000 与 100 字节）与三种小盘；4 GiB 上同一形要写满几十万槽，没跑。「放行 ⇒ 下一次挂载放行」这条是我立的判据，条款里没有，主 agent 判它该不该立。
- Z19-B 的「改前」用的是改后树上关掉准入的开关（改前两处都不判），不是第三轮冻结树本身（第三轮树的执行器没有这个字段，用例编不过去）。
- 本地攻方的算术几格我没碰；C542 那一格没攻。

## 没做什么

- 没跑库里任何测试二进制（只跑了自己写的 `r4_opus_z19`、`r4_opus_z21`、`r4_opus_z22`）；mutations.tsv 里点到的那几行我没在副本上证红，Z22 里关于它们的句子是读出来的。
- 没判 Z20、Z23 与算术几格；没替主 agent 采纳改法。
- 没把副本上的数做成入库装置的一次跑。
