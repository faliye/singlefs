# m3-admission-r1 云端正推腿（Sonnet）报告

日期：2026-10-12。轮名 m3-admission-r1，格 A1–A6，立场照正文「四、分工」表云端正推那一行：从条款原文与 `crates/` 今天的代码推「这一行还缺哪一句」。

## 0　读法与取证

- kb 条款写 kb 文件名加那份文件自己的行号（行号是写报告时对 kb 文件现查的）；kb 的长行（D16 已定项 1 的表格行有几千字）只抄与结论有关的整句，句内用「…」切开，不是整行抄；整行原文在被引文件里按行号可得。
- 代码引文对冻结副本 `~/.local/share/singlefs/work/m3-admission-r1/repo/`（引用处写 `crates/…` 路径与副本里的行号）。点名的 7 份文件 sha256 核过：

```
$ cd ~/.local/share/singlefs/work/m3-admission-r1/repo && sha256sum -c /home/fy5090/code/singlefs/research/prompts/m3-admission-r1-snapshot/frozen-copy-sha256.txt
crates/singlefs-core/src/admission.rs: OK
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-core/src/allocator.rs: OK
crates/singlefs-core/src/make_filesystem.rs: OK
crates/singlefs-harness/tests/unequal_devices.rs: OK
crates/mutations.tsv: OK
```

- 点名之外的文件（`mounted_session.rs`、`recovery.rs`、`root_ring.rs`、`system_configuration.rs`、`singlefs-format`、别的 harness 用例）是 2026-10-12 暂存区那一版，引到时写明。
- 腿自己量的数：在草稿目录 `~/.local/share/singlefs/work/m3-admission-r1-sonnet/` 的冻结副本拷贝上量（debug 编译，`run-with-memory-cap.sh 16G` + `capped.sh 15`，同一时刻一条 cargo，没改冻结副本）。探针源码、变异脚本、原样输出与 `tools/rerun.sh` 在模型目录 `research/prompts/m3-admission-r1-sonnet-model/`（`out/` 里每个 `.txt` 是对应命令的 `name=` 行、`test` 行与 `test result` 行）。
- 判定词照正文「五、交付要求」：还缺 / 已被吃掉 / 部分。

### 0.1　先看一个贯穿 A3、A5 的结构事实（读码，后面各格引用）

发布路径上有三道闸，次序是：落得下（`admit_the_units_landing_on_every_device`，`crates/singlefs-core/src/transaction.rs:6709`）→ 式子（`admit_on_every_device`，`crates/singlefs-core/src/transaction.rs:6763` 起，读数 `admission_reading_before_a_publish_on_a_ring_of`）→ 落点那一道（分配器真取落点）。kb 里写下来的只有后两道：`.claude/kb/decisions/03-空间分配.md:257`「是两道串联的闸：先过 D28（挂载期承诺量） 的…再过 D16（发布语义） 的抬 F 逻辑」与 `.claude/kb/decisions/03-空间分配.md:136`「共七条，全部为真才许分配」（七条合取里没有「这次的单元落得下」）；落点那一道的条款是 `.claude/kb/decisions/03-空间分配.md:98`「提交内生块在固定点分配时找不到空槽（NoFreeSlotOnAnyDevice），在任何写之前拒绝这次发布，盘上不变」。第一道（用户 2026-09-27 定「准入先拒」）只在 `.claude/kb/checks-owed.md:718`（已还清表 C545 行）与代码文档里有；D28 / D3 的条款正文里没有写它这一道（D28 已定项 4 末尾的「欠」行 `.claude/kb/decisions/28-挂载期承诺量.md:137` 只提了 C573–C576 的名字，D28 `…:36` 的「删除类发布的落点那一判照判」字面分不出指哪一道，见 A5.1）。会话把两种拒绝同样处理：`crates/singlefs-core/src/mounted_session.rs:912`（暂存区版）`PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,`，都推抬 F 再重判。

## A1　单位：需求、可用、df 与活元数据（C370、C371）

**判定：部分。** 需求的单位已被实现吃掉（取盘字节），但条款没写；`df` 与「活元数据」两样 `crates/` 里没有，条款里的两处也没定义。

### A1.1　今天条款与实现各说什么（并排）

| 项 | 条款 | 实现 |
|---|---|---|
| 「已分配」的口径 | `.claude/kb/decisions/28-挂载期承诺量.md:24`「物理字节，与 D5（快照 / 空间记账机制） 已定项 7 的」；D5 已定项 7 见 `.claude/kb/decisions/05-快照-空间记账机制.md:191`「固定结构（系统配置槽、根环、journal 环）既不在容量里也不算已分配」 | `crates/singlefs-core/src/admission.rs:687` `allocated: BytesOnOneDevice::of_slots(device_map.allocated_slots()),`（逐盘槽数 × 16384） |
| 「需求」摊到每块盘 | `.claude/kb/decisions/28-挂载期承诺量.md:24`「摊到每块盘按什么记，仍是 C370（需求、可用与 df 没有共同单位） 那一格。」；`.claude/kb/decisions/28-挂载期承诺量.md:36`「需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，不加这次换下的槽）」（单位写的是「槽」，没说按盘还是按副本之和） | `crates/singlefs-core/src/admission.rs:1478` `let slots_of_the_demand: u64 = rewritten_roles.iter().map(|role| role.span_slots()).sum();` 之后每块盘一条同一个字节数；模块文档自陈 `crates/singlefs-core/src/admission.rs:25`「实现员取的读法（条款没写，交主 agent）」 |
| 副本数 | 条款：全池三项「按全部副本之和记字节」再「摊到每块盘各扣副本之和 ÷ 副本数」 | `crates/singlefs-core/src/admission.rs:127`「⇒ 副本数 = 池里的盘数」（第一版每个单元落每一块盘） |
| `df` | `.claude/kb/decisions/28-挂载期承诺量.md:96`「可用已经扣掉这一项，df 报可用（D3（空间分配） 已定项 9 第 1 条）」；`.claude/kb/decisions/16-发布语义.md:43`「没用过的 + 已释放的 + 活元数据 − 保留池 − 推空最坏残留 − 滞后量」 | 没有（命令见 A1.2） |
| 「可分配」 | `.claude/kb/decisions/16-发布语义.md:42`「可分配 = min(可再分配 + 活元数据 − 保留池, df)」 | 没有按这个式子算的量；准入只算 D28 的九项式子 |
| 「活元数据」 | 只出现在上面两个 D16 式子里；`.claude/kb/checks-owed.md:280`「全仓只在式子里出现（2026-09-17 grep）」 | 没有 |

### A1.2　复跑的零命中（本轮在冻结副本上现跑）

```
$ cd ~/.local/share/singlefs/work/m3-admission-r1/repo && grep -rn 'statfs\|fn df_\|df_available' crates | wc -l
0
$ grep -rn '活元数据\|live_metadata' crates | wc -l
0
```

### A1.3　今天缺的是哪几句

1. 缺一句「需求(d) = 这次发布新写的全部槽 × 16384，每块盘一份、每块盘相同」。事实上 D28 已把后三项钉成按全部副本之和记，「已分配」是逐盘物理字节，需求要与「已分配」同口径就只能读成盘字节（C370 行自己在 `.claude/kb/checks-owed.md:279` 写了「真正缺的是 D16（发布语义） 已定项 1 那一套…」，即需求那一半 2026-09-17 已收窄）。这句今天能被一条会红的用例钉住：我在复跑副本里把 `demand_of_the_roles_on_each_device` 的字节数乘以盘数，`admission` 的 21 条单测里恰好 1 条转红：

```
$ （基线）admission 单测：test result: ok. 21 passed; 0 failed
$ （改动：let bytes = BytesOnOneDevice::of_slots(slots_of_the_demand * u64::try_from(devices.len()).unwrap());）
test admission::tests::demand_counts_every_rewritten_role_and_only_a_publish_with_an_ordinary_allocation_is_judged ... FAILED
test result: FAILED. 20 passed; 1 failed
```
（原样输出在 `research/prompts/m3-admission-r1-sonnet-model/out/demand-times-devices.txt`。）
2. 缺一句 `df` 是不是逐设备取小、有没有设备维（`df` 没实现，所以这一句今天只能先写条款，检查等 `df` 落地——C370 行「检查」列自己就带这个前提）。
3. 缺「活元数据」是哪个集合，或这一项是否该删。

### A1.4　候选写法、代价、要动的文件

`df` 的设备维（多报 / 少报用 D28 式子里只留 `容量 − 已分配` 的读数量，其余八项取 0；几何取 `crates/singlefs-harness/tests/unequal_devices.rs:300` 里的两块不等大的盘：单元区 211 968 槽与 146 465 槽，等量几何取两块 4 GiB 盘各 211 968 槽）：

| 候选 | 定义 | 要动的 `crates/` 文件 | 两盘等量 | 两盘不等量（211 968 / 146 465 槽） |
|---|---|---|---|---|
| 甲 逐设备取小 | `df` = min_d 可用(d)（再钳 0） | `crates/singlefs-core/src/admission.rs` 加一个函数读 `AdmissionReading::available_on_each_device`（`crates/singlefs-core/src/admission.rs:721`）；调用点（statfs 一类）`crates/` 里没有，要新加 | 211 968 槽，不多报不少报 | 146 465 槽；大盘多出的 65 503 槽之上的空间每个单元要落每块盘、同槽，不可用，不算少报 |
| 乙 全池之和 | `df` = Σ_d 可用(d) | 同上 | 423 936 槽，多报 211 968 槽（约 3.2 GiB），按 `.claude/kb/decisions/03-空间分配.md:192`「只要 df 报出的空闲 ≥ s，写 s 字节就必须成功」就是假性 ENOSPC | 358 433 槽，多报 211 968 槽 |
| 丙 之和 ÷ 副本数 | `df` = Σ_d 可用(d) ÷ 盘数（与全池三项摊到每块盘的做法同形） | 同上 | 211 968 槽，与甲相等 | 179 216.5 槽，多报约 32 751.5 槽（约 0.5 GiB） |

（槽数 × 16 384 = 字节；211 968 × 16 384 = 3 472 883 712，与探针 `out/a3-a2-a4-a5-probe.txt` 里 `d0_available_bytes=3472883712` 一致。）C370 行 `.claude/kb/checks-owed.md:279` 说第二轮攻方腿的模型里只有「两边都逐设备取小、且不预付」这一种 `df` 写法在三条臂上都不出假性 ENOSPC——甲就是它；「预付」（写入时要先留的解开自己）：C370 行 `.claude/kb/checks-owed.md:279` 转述 D3 已定项 2 自陈「量还没算」，这句已过时——今天的实现把墓碑与退役预付算成记账标签 6「已承诺预留」，按发布之后那一版的标签进式子（`crates/singlefs-core/src/admission.rs:333` `committed_reservation_for_tombstones_and_retirement_records`，副本数 × ((常驻槽数) × 16384 + 41 × n_live + 35 × n_记录)；`crates/singlefs-core/src/admission.rs:326` 文档里是这个式子），所以写入量 s 对预付的影响只剩新增 inode 数 × (41 + 35) × 副本数，`df` 可以取「现行标签下的单个数」，也可以另加这一项做成 s 的函数——甲里还要一并定的第二句。

「活元数据」：

| 候选 | 定义 | 要动的文件 | 代价 |
|---|---|---|---|
| 子 删掉这一项 | D16 已定项 1 的 `准入` 与 `df` 两行里去掉它；D28 式子本来就没有 | 不动 `crates/`；E139 装置的 `pool.metadata` 取法要跟着改（C371 行的检查列） | `可分配`、`df` 少加一截，保守 |
| 丑 现行版本引用的元数据块 | 现行版本的树节点（inode / extent / 记账 / 中央映射 / 分配记录 / 谱系 / 退役各树）加树表加实例表链的块数 × 16 384，按盘记 | `crates/singlefs-core/src/admission.rs`（`checkpoint_cost_of_the_version_to_build_on_with_node_capacities`，`crates/singlefs-core/src/admission.rs:1053` 已经在数这几棵树的节点） | 新量，D5 已定项 4 的统计量清单要跟着判完备 |
| 寅 别的集合 | 条款没写它为什么要加进可分配；这一格我从条款与代码推不出原意 | — | 见岔路 |

### A1.5　要交用户的岔路（不带倾向）

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 需求的单位 | 盘字节、每盘相同（实现今天取的）/ 全部副本之和再摊（w ≥ 3 条带才与前者分开） | 第一版 w = 2 全镜像：两种在每盘上数相等，翻面要等 w ≥ 3 | 无需再量；定了就写进 D28 已定项 1 的需求那一句，用上面那条乘盘数变异当会红用例 |
| `df` 的设备维 | 甲 / 乙 / 丙 | 两块不等大的盘上 `df` 报值与「写 s 字节是否成功」对照：乙、丙在 unequal 几何上出假性 ENOSPC，甲不出 | `df` 实现出来后在 `unequal_devices` 几何上跑一次；今天只够判条款文字 |
| `df` 是不是写入量的函数（预付） | 单个数（现行标签）/ 随新增 inode 数变 | 一次写创建 k 个新 inode 时，预付增量 k × (41 + 35) × 副本数 字节是否会让「df 报 ≥ s 而写 s 失败」出现（k 大、盘近满才会） | 在近满盘上造一次「创建 k 个文件」的写，量 `df` 单个数版本与随 k 版本谁出假性 ENOSPC；今天没量 |
| 活元数据 | 子 / 丑 / 寅 | C371 行检查列：把活元数据改成全部活着的元数据块，C355 第一轮那一格要由不中翻成中 | 要先有 E139 里 `pool.metadata` 的取法与条款定义逐字对照 |

**什么现象会推翻 A1 的判定：** (a) 找到 `crates/` 里按盘数乘需求的一处调用（`grep -n 'demand_of_the_roles_on_each_device\|admit_on_every_device' crates` 全看一遍）；(b) `grep` 出一处 `df` / `statfs` 的实现；(c) 乘盘数的变异在 21 条单测里一条都不红。

## A2　第九项的口径（C356）

**判定：已被吃掉（符号与口径两半）；剩下的「不被高估」是 A3 的 `扣住` 与已知的双扣，不是第九项本身的问题。**

### A2.1　被吃掉的是哪一条定案、哪一处代码

- 条款：`.claude/kb/decisions/28-挂载期承诺量.md:27`「取值 = 分配器实际隔离的集合的槽数」，同一行末尾「与分配器实际隔离的集合是同一个数」。C356 行自己也写了 `.claude/kb/checks-owed.md:271`「不小于 0 已由定义保证」。
- 代码：`crates/singlefs-core/src/allocator.rs:639` `pub fn isolated_slots(&self) -> u64 {`（返回 `u64` 计数，类型上不可能为负）；`crates/singlefs-core/src/admission.rs:691` 读数取 `BytesOnOneDevice::of_slots(device_map.isolated_slots()),`，整个当第九项。没有「统计量之差」的路径。
- 口径分歧：`.claude/kb/checks-owed.md:271`「2026-10-11 现查全 kb 只剩本行还留着这个旧口径」。我对 `.claude/kb/` 重查：
```
$ cd /home/fy5090/code/singlefs/.claude/kb && grep -rn 'R_old 起' --include=*.md . | cut -c1-140
checks-owed.md:271:| C356 | 第九项按统计量之差算，净释放时为负且没有钳位 | D28（挂载期承诺量） 已定项 1 第
```
  只剩 C356 行自己（把旧口径当历史引用）。已还清的 C318 行 `.claude/kb/checks-owed.md:606` 写的是「被抛弃根独占量」，与 D28 一致。所以「怎么拦」列的两个口径今天对得上，那一对矛盾已经没有了。

### A2.2　「可用不被高估」这半句今天成不成问题：拿分配器能发的槽数对拍

C356 行「检查」列要的等式是「分配器发得出的槽数与准入算出的可用逐格相等」。我在副本里拿 `AdmissionReading::of_allocator`（其余八项取 0）的可用槽数对 `DeviceFreeMap::unblocked_slots()`（三种挡位都没置的槽数，分配器真发得出的）逐阶段对拍（探针 `research/prompts/m3-admission-r1-sonnet-model/probes/a3_probe.rs`，用例 `identity_between_the_reading_and_what_the_allocator_can_issue`；原样输出在 `out/a3-a2-a4-a5-probe.txt`）：

```
name=a2a3-identity stage=fresh reading_available_slots=211968 allocator_issuable_unblocked_slots=211968 reading_minus_issuable=0
name=a2a3-identity stage=two_data_units_allocated reading_available_slots=211964 allocator_issuable_unblocked_slots=211964 reading_minus_issuable=0
name=a2a3-identity stage=isolate_two_free_slots reading_available_slots=211962 allocator_issuable_unblocked_slots=211962 reading_minus_issuable=0
name=a2a3-identity stage=isolate_two_allocated_slots_double_deducted reading_available_slots=211960 allocator_issuable_unblocked_slots=211962 reading_minus_issuable=-2
name=a2a3-identity stage=two_slots_reclaimed_and_held reading_available_slots=211962 allocator_issuable_unblocked_slots=211962 reading_minus_issuable=0
name=a2a3-identity stage=holds_released reading_available_slots=211962 allocator_issuable_unblocked_slots=211964 reading_minus_issuable=-2
```

读法（差 = 读数可用 − 分配器可发）：隔离空槽那一阶段差 0，第九项没有高估；隔离到已分配的槽时差 −2（读数比分配器少 2 槽，方向是少报，安全）——这就是 D28 `.claude/kb/decisions/28-挂载期承诺量.md:27` 括号里 C762 点名的「与「已分配」双扣」，`crates/singlefs-core/src/allocator.rs:582` `pub fn isolate(` 不跳过分配位，`crates/singlefs-core/src/allocator.rs:598` `pub fn isolate_the_slots_not_allocated(` 跳过；抬 F 那一串读盘得来、表里没记着的被抛弃根仍走前者（`crates/singlefs-core/src/allocator.rs:2236` 的文档注释写了这一支）。「回收但扣住」那一阶段的 0 是 −2（双扣）与 +2（扣住）恰好抵消，不是等式成立——拆开看见 A3。

### A2.3　这一行该还清、改写还是留着

三种处置都是记账动作，我不选。各自的事实：
- 还清：符号与口径两半已由 `.claude/kb/decisions/28-挂载期承诺量.md:27` 与 `crates/singlefs-core/src/allocator.rs:639` 吃掉；C318 的旧口径冲突不再存在。留下的「净释放历史下读数 = 分配器可发」没有会红的用例——我的对拍是分配器层的合成历史，不是「崩溃恢复落到旧根、被抛弃时间线净释放」那一类真历史（没造，见「没做什么」）。
- 改写：缩成「净释放的被抛弃时间线下，对拍等式的会红用例」一句欠账，把双扣挪给 C762、扣住挪给 A3 的新行。
- 留着：C356 行文字里的「改后的口径…可用是否不被高估」仍是未验的问句。

### A2.4　岔路与推翻条件

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| C356 行去向 | 还清 / 改写 / 留着 | 在「崩溃恢复落旧根且被抛弃时间线净释放」的真历史上，读数可用 − 分配器可发 > 0 且扣住位为 0 | 造这一类历史跑一次对拍（harness 里 `admission_formula_on_the_mounted_state.rs` 已有「被抛弃一条根」的建池函数 `remounted_after_a_recovery_abandoned_the_third_version`，要加删除类历史）；跑出差 > 0 → 留着；差 ≤ 0 → 可还清 |

**什么现象会推翻「已被吃掉」：** 上面那类真历史上读数可用 − 分配器可发 > 0 且 `free_slots − unblocked_slots` 为 0（即不是扣住造成的）。

## A3　抬 F 扣住的槽（C570、C546 剩下两格）

**判定：部分。** 「式子里没有一项装扣住」这半句今天仍成立（已量）；「落点那一判兜得住」只对用户发布成立，兜不住的那一格有机制、各段已量，整条历史没造；C546 剩的两格（落盘途中写错、读盘核对不上）里扣住位留在进程里的事实已量到。

### A3.1　已量：读数把扣住的槽算空闲，落点读数把它们算挡着

代码（对冻结副本）：回收后的槽先 `crates/singlefs-core/src/allocator.rs:726` `self.allocated_slots -= span;`（`mark_reclaimed`，`crates/singlefs-core/src/allocator.rs:710`），再 `crates/singlefs-core/src/allocator.rs:665` `pub fn hold_until_floor_takes_effect(` 只置「抬 F 扣住」位、不动已分配数与空闲数。准入读数的「已分配」读 `allocated_slots()`（`crates/singlefs-core/src/admission.rs:687`），落得下那一判读 `unblocked_slots`（三种位都没置，`crates/singlefs-core/src/admission.rs:833` 一带）——两边说的不是同一个集合。`crates/singlefs-core/src/admission.rs:672-673` 的文档 ⚠️ 自陈：「那两段里准入读哪一个，条款没写。」C570 行 `.claude/kb/checks-owed.md:402`「那一刻的准入判断比分配器真会给的更松」。

量（探针 `a3_probe.rs` 用例 `probe`，两块 4 GiB 盘的分配器，分配两个数据单元、释放、回收并扣住，再放开；原样输出 `out/a3-a2-a4-a5-probe.txt`）：

```
name=a3-probe stage=both_released_in_defer_window d0_allocated_slots=4 d0_free_slots=211964 d0_available_bytes=3472818176 d0_unblocked_slots=211964 d0_pairs_for_data=105982
reclaimed_placements=2
name=a3-probe stage=reclaimed_but_held_until_floor_takes_effect d0_allocated_slots=0 d0_free_slots=211968 d0_available_bytes=3472883712 d0_unblocked_slots=211964 d0_pairs_for_data=105982
name=a3-probe stage=holds_released d0_allocated_slots=0 d0_free_slots=211968 d0_available_bytes=3472883712 d0_unblocked_slots=211968 d0_pairs_for_data=105984
```
扣住期读数可用比窗口内多 65 536 字节（4 槽），分配器可发的槽数与数据槽对数一个没涨；放开后两边才对上。多出的量 = 被扣住的槽数。

### A3.2　成功的抬 F 里准入读不到扣住位；失败的抬 F 把它留在进程里（已量）

- 成功：`crates/singlefs-core/src/mount.rs:4677` `allocator.release_reclaim_holds();` 在整串空发布做成之后、同一次调用内放开；整串里的空发布没有普通分配，不判准入。所以成功路径上用户发布的准入从来读不到扣住位。
- 失败：`crates/singlefs-core/src/mount.rs:4639` `return Err(MountError::RaiseFloorSequencePublishFailed(` 在第二次起落盘途中失败时直接返回，不放开。`release_reclaim_holds` 的调用点只有两个（`crates/singlefs-core/src/mount.rs:4677`、`crates/singlefs-core/src/mount.rs:6256`，后者是可写挂载暖机之后放开重建时扣住的那一截）。D16 写了这一格没有条款：`.claude/kb/decisions/16-发布语义.md:40`「那一格里扣住的槽怎么放没有条款，见 C546（抬 F 被拒时扣住的槽不退回）」；C546 行 `.claude/kb/checks-owed.md:387`「这两格里扣住位留在进程里、下一次抬 F 做成才放，怎么放条款没写」。

量（探针 `c546` 用例，harness 里 `commit_generated_units_fall_back_when_the_segment_runs_out.rs` 的注入写错装置，抬 F 到 4，第二次空发布的第一个写报块设备错；`out/c546-probe.txt`）：

```
name=c546-probe success before(free,unblocked)=(211881, 211881) after(free,unblocked)=(211880, 211880) reclaimed=13
name=c546-probe write_failed_after_one_persisted d0_free_slots=211888 d0_unblocked_slots=211873 d0_free_minus_unblocked=15 d0_isolated=0 frozen_publish_present=true
```
做成时扣住数 0（空闲 = 可发）；写错之后盘 0 上 15 个槽计数上空闲、分配器不发（`free_slots − unblocked_slots = 15`，隔离数 0，所以是扣住位）。

### A3.3　「兜不住的那一格」是什么样的历史（机制已量，整条没造）

推：用户发布的准入先过落得下那一判（只数没挡的槽，不数保留项），再过式子；式子把扣住槽当可用，所以式子放行的上限比分配器真能给的多出「扣住数」。一次准入放行并落地之后，剩下的真可发槽数可能小于式子里为下一次空发布留的保留池（`checkpoint_reserve_pool` 一项），这时用户发布成功，但下一串抬 F 的整串预演（`crates/singlefs-core/src/mount.rs:4577` 的 `rehearse_the_publishes_raising_the_floor`）取不到落点，报 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`；会话把它当「推满仍不够」报 `NoSpaceAfterRaisingTheFloor`。而扣住的槽只有一次成功的抬 F 才放开，抬 F 又做不成——这是 D3 已定项 9 第 1 条（`.claude/kb/decisions/03-空间分配.md:192`「只要 df 报出的空闲 ≥ s，写 s 字节就必须成功」）的反面：若 `df` 照式子读，`df` 报有空间、写入 ENOSPC。造这条历史需要：抬 F 写错并重发冻结的那一次（扣住位留着）→ 用户发布把可发槽数压到小于保留池 → 再抬 F。我没造（要同时布置注入、重发和近满盘）；机制的前两段（读数多算扣住数；写错后扣住数 > 0）都有上面的原样输出。

### A3.4　候选写法、代价、要动的文件

| 候选 | 定义 | 要动的 `crates/` 文件 | 代价 / 注意 |
|---|---|---|---|
| 甲 式子加一项「扣住」 | 第十项 = 扣住的槽数 × 16384（逐盘） | `crates/singlefs-core/src/allocator.rs`（`DeviceFreeMap` 加一个随 `set_blocking_bit`（`crates/singlefs-core/src/allocator.rs:450`）增量维护的扣住数）；`crates/singlefs-core/src/admission.rs`（`DeviceAdmissionTerms`、`AdmissionReading::of_allocator`、`available_on_each_device`） | D28 已定项 1 的式子行带 `gate:admission-formula` 标记，改式子要连 D5 已定项 4 的「式子逐项对照」表与门禁里那条式子核对一起改；平时扣住数为 0，只在抬 F 的两个窗口与写错残留里不为 0 |
| 乙 读数把扣住算成已占 | `of_allocator` 里 `allocated` 取 `allocated_slots + 扣住数`，式子字面不变 | `crates/singlefs-core/src/admission.rs:687` 一处 + 同样要一个扣住数读口 | 读数的「已分配」与记账第 1 项「已分配字节」（I-3.1 对账的统计量）从此不是同一个数；`crates/singlefs-core/src/admission.rs:579-582` 文档把它定义成统计量的镜像 |
| 丙 落得下那一判也数保留项 | 落得下 = 没挡的槽 ≥ 单元 + 固定点 + 下一次空发布的保留 | `crates/singlefs-core/src/admission.rs`（`UnitsOfAPublishToLand`、`admit_the_units_landing_on_every_device`）与 `crates/singlefs-core/src/transaction.rs:6709` 调用点 | 保留项要换成槽数口径；式子那一判不变；和甲不同的是扣住期兜底在第一道而不在式子 |
| 丁 不改，条款写「扣住期读数偏松，由落得下那一判兜；兜不住的这一格是已知限度」 | 只改 D28 / D16 文字 | 无 | 不再让 `df` 与准入分叉要靠别的办法（A1 甲） |

C546 剩的两格（扣住位怎么放）：

| 候选 | 定义 | 要动的文件 | 注意 |
|---|---|---|---|
| 戊 留到下一次抬 F 做成才放（今天） | 写成条款 | 无 | 机制见 A3.3：扣住期里的准入偏松 |
| 己 写错之后检查「带新 F 的根是否已落满每块盘」，满了就放 | 重发冻结的那一次成功后（或失败返回处）按落盘的 txg 判 | `crates/singlefs-core/src/mount.rs:4639` 一带、重发入口 `resend_the_frozen_publish`（`crates/singlefs-core/src/transaction.rs:1366`，重发处要能调到 `release_reclaim_holds`） | 判「落满每块盘」要用根环落点公式 `device_of_txg`，重发的那一次本身可能就是补满的那一次 |
| 庚 失败返回时把分配器整个换回抬 F 之前 | 同第一次发布失败的做法 | `crates/singlefs-core/src/mount.rs:4635` 那一支（今天只对「第一次就失败且无冻结」换回） | 前面几次已落盘，换回会让分配器忘掉它们已占的槽，危险；今天的注释就是因此只换第一次 |

### A3.5　岔路与推翻条件

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 式子要不要装扣住 | 甲 / 乙 / 丙 / 丁 | A3.3 那条整历史：式子放行、用户发布落地、再抬 F 被预演拒。造出来 → 丁不够；造不出（预演总能取到落点）→ 丁够 | 在注入装置里造这条历史跑一次 |
| 写错残留的扣住位 | 戊 / 己 / 庚 | 己的判据「根落满每块盘」在重发之后是否恒可从分配器内存里读出 | 读一次 `raise_rollback_floor` 的 `planned_txgs` 与重发路径，看是否有口 |

**什么现象会推翻 A3 判定：** (a) 扣住期里读数可用与分配器可发相等（探针里 `d0_available_bytes` 不涨）；(b) 写错之后 `free_slots − unblocked_slots` 为 0。

## A4　df 与段内关着的空槽（C571）

**判定：部分。** 「段内关着的空槽计数」这个读数的前置今天已经在分配器里（吃掉了前置），`df` 与条款没有（还缺）。

### A4.1　读数前置今天已有（已量）

- 条款：`.claude/kb/decisions/03-空间分配.md:165`「聚簇段在挂载期间对用户数据关着：这次挂载开过的聚簇段（当前开放的那一段，以及回落之前开过的段）都不给…」（原句续到「用户数据」）。
- 代码：`crates/singlefs-core/src/allocator.rs:493` `pub fn unblocked_slot_pairs_outside_the_cluster_segments(` = 总的没挡槽对数 − 这次挂载开过的各段里的槽对数；`crates/singlefs-core/src/admission.rs:830` 一带 `PlacementRoomOnOneDevice.slot_pairs_for_data_units` 把它交给准入。C571 行 `.claude/kb/checks-owed.md:403` 的前置「先要有…这个读数」就是这个按段增量维护的槽对数（C571 行引的 A4c 报告第三节第 3 条）。
- 缺的一半：它数的是槽对，段内孤立的单个空槽不在其中。要得到「段内关着的空槽数」本身需要一个按段的没挡槽数（镜像 `used_per_segment`），在 `crates/singlefs-core/src/allocator.rs:450` `set_blocking_bit` 里同时维护；今天没有。

量（探针用例 `slots_closed_to_user_data_inside_the_mounts_cluster_segments_grow_with_churn`，核心层分配器：先分配 300 个一槽提交内生块，再全部释放并回收；`out/a3-a2-a4-a5-probe.txt`）：

```
name=a4-closed-pairs stage=fresh cluster_segments=0 unblocked_slots=211968 pairs_for_data_outside_segments=105984 pairs_for_data_per_room=105984 slots_unblocked_inside_cluster_segments_upper_bound=0
name=a4-closed-pairs stage=300_one_slot_commit_generated_blocks cluster_segments=5 unblocked_slots=211668 pairs_for_data_outside_segments=105824 pairs_for_data_per_room=105824 slots_unblocked_inside_cluster_segments_upper_bound=20
name=a4-closed-pairs stage=all_300_released_and_reclaimed_inside_closed_segments cluster_segments=5 unblocked_slots=211968 pairs_for_data_outside_segments=105824 pairs_for_data_per_room=105824 slots_unblocked_inside_cluster_segments_upper_bound=320
name=a4-closed-pairs next_user_data_slot=50496 (单元区起点 50176)
```
（`…_upper_bound` 是 没挡槽数 − 2 × 段外槽对数；这个史里段外没有孤立单槽，所以它等于段内关着的空槽数。）这一史里 5 个段、320 个空槽（5 242 880 字节）全部对用户数据关着，用户数据落点跳到 50496 = 起点 + 320。关着的量随这次挂载开过的段数涨、重挂载之后才对用户数据开放（`.claude/kb/decisions/03-空间分配.md:165` 同一条后半）。

### A4.2　假性 ENOSPC 的见证：用例名已漂移，我补了核心层一条（已量）

C545 行 `.claude/kb/checks-owed.md:718` 登记的那条「字节式子放行、写入被拒」的见证用例名 `a_write_whose_free_slot_pairs_are_all_inside_this_mounts_cluster_segments_is_refused_by_the_space_admission_before_any_write_while_the_byte_formula_admits`，在冻结副本的 `crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` 里已经没有：该文件现存 5 条用例（`out/c545-existing-5-tests.txt`，全绿），最接近的 `crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs:350` 那条（文档在该文件 343–348 行）造盘面（段外槽全标已分配）去撞落得下那一判，不断言式子放行；该文件头注释（第 5–6 行）写明原因：小盘为环窗口补足之后普通写被式子先拒。所以「df 若照式子读会出假性 ENOSPC」这一格今天没有现成的见证用例。我补了一条核心层的见证（探针用例 `the_byte_formula_admits_a_data_unit_while_no_slot_pair_outside_the_cluster_segments_is_left`，史同上、再把段外空槽全标已分配）：

```
name=a4-witness d0_formula_available_slots=320 formula_admits_one_data_unit=true d0_pairs_for_data_outside_segments=0 landing_check_admits_one_data_unit=false user_data_alloc=None
```
式子读数可用 320 槽、放行一个 2 槽的数据单元；段外一对都没有，落得下那一判拒，分配器也取不到用户数据落点（`user_data_alloc=None`）。`df` 若等于这个可用，就是 D3 已定项 9 第 1 条的假性 ENOSPC。

### A4.3　候选写法（条款该怎么写）与后果

D3 已定项 9 第 1 条（`.claude/kb/decisions/03-空间分配.md:192`「只要 df 报出的空闲 ≥ s，写 s 字节就必须成功」）是判据；D3 已定项 13（`.claude/kb/decisions/03-空间分配.md:267`「不分 data / metadata 两类空间，全局统一分配器。」）限制了第三种写法。

| 候选 | 定义 | 假性 ENOSPC | 代价 / 要动的文件 |
|---|---|---|---|
| 甲 报（`df` 照式子，含关着的槽） | `df` = D28 式子可用（A1 甲） | 有：上面的史里 `df` 比用户数据真能拿的多 5 242 880 字节；写超过段外容量的数据时 `df ≥ s` 而 ENOSPC，违反 D3 已定项 9 第 1 条 | 不用新读数；条款上要把「关着」写成已知违反并改 D3 已定项 9，或否掉这一格 |
| 乙 不报（`df` 扣掉段内关着的空槽） | `df` = 式子可用 − 段内关着的空槽 × 16384 | 数据写不出假性 ENOSPC；提交内生块仍用得上这些槽，所以 `df` 偏少的那部分在提交路径上不是浪费 | 要新读数：按段的没挡槽数（`crates/singlefs-core/src/allocator.rs:450` 一带加字段并增量维护）+ `admission.rs` 一个读口；偏少量随挂载内开段数涨，重挂载归零 |
| 丙 另报一项（`df` 报数据可用，另报元数据可用） | 两个数 | 数据那一项不出；元数据那一项是提交内生块可用 | 与 D3 已定项 13「不分 data / metadata 两类空间」字面冲突（分配器没有第二个分类轴，这里是读数的第二个轴，不是分配轴，是否算冲突要用户判）；FUSE 一侧 `statfs` 要能报两项 |

### A4.4　最小前置、岔路、推翻条件

把这一格做成会红的用例的最小前置：(1) `df` 的实现（A1 甲）；(2) 按段的没挡槽数读数（乙需要；甲、丙不需要）；(3) 用例：造一段「字节式子放行、段外没有成对空槽」的历史（现成：上面补的核心层那条探针，要升成入库用例）+ 断言 `df` 报值 < 一个数据单元所需（乙）/ `df` ≥ 且写入被拒（甲，把见证钉成「已知违反」）。

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| `df` 报不报段内关着的空槽 | 甲 / 乙 / 丙 | 关着的槽数占式子可用的比例：本轮合成史是 5 个段 320 槽（0.15%，4 GiB 盘）；真负载（提交内生块的释放与回收频繁）没量 | 量一次长挂载里 `段内关着的空槽 / 式子可用`；要 `df` 与按段读数先有 |

**什么现象会推翻 A4 判定：** (a) `grep` 出 `df` 实现或「段内关着的空槽」按段读数（今天零命中：A1.2 与 allocator.rs 里 `used_per_segment` 之外无按段没挡槽数）；(b) 上面那个 300 块的史里 `…_upper_bound` 不是 320。

## A5　落得下那一判的射程（C573、C574、C576）

**判定：** C573 部分（普通分配节点已算，删插没算）；C574 还缺（两半都没写、实现也只在落点那一道拒）；C576 部分（「改法待选」今天事实上是改钉 + 保留直调 + 开关用例三者并存，登记行还欠第 12 行的靶子）。

### A5.1　这一判今天判什么、不判什么（读码）

- 判：数据单元个数 ≤ 段外成对的没挡槽对（`crates/singlefs-core/src/admission.rs:926` `if room_on_the_device.slot_pairs_for_data_units < units.data_units {`）；没挡的槽数减去数据单元取走的槽 ≥ 提交内生块槽数（`crates/singlefs-core/src/admission.rs:937` `let slots_left_after_the_data_units = room_on_the_device`），后者 = 普通分配那一半各角色的槽数之和 + `ckpt_cost`（`crates/singlefs-core/src/admission.rs:893` `commit_generated_slots: ordinary_commit_generated_slots`）。
- 不判：`crates/singlefs-core/src/admission.rs:904` 的文档 ⚠️ 自陈「② 只数槽：两槽的 inode 叶容器要成对」，两槽对齐与各盘同槽这一判不判；读数是每块盘各自一条（`PlacementRoomOnOneDevice`），没有跨盘比较。
- 次序与范围：`crates/singlefs-core/src/transaction.rs:6709` 排在 settle 之前；删除类发布（意图、各批）不判式子、落得下照判——D28 `.claude/kb/decisions/28-挂载期承诺量.md:36` 写的是「删除类发布的落点那一判照判」，字面没分清这个「落点那一判」是落得下这一判还是分配器真取落点那一道（现存用例两种都有：`an_unlink_intent_whose_commit_generated_units_find_no_unblocked_slot_is_refused_by_the_landing_judgement_before_any_write_while_the_formula_is_not_judged`，`out/c545-existing-5-tests.txt`）。
- 两道拒绝会话同样处理：`crates/singlefs-core/src/mounted_session.rs:912`（暂存区版）把 `SpaceAdmissionRefused` 与 `PlacementRefused` 都当「空间不够」，都推一串抬 F 再重判，预算同一个 `PUBLISHES_PER_ADMISSION_AT_MOST`（`crates/singlefs-core/src/mount.rs:3973` `pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;`）。所以两种写法对「挂着的会话多推还是少推」没有差别：都推；差别只在交回的错误类型与载荷。

### A5.2　C573：固定点上界

- 第一半（要不要算普通分配里的 extent / inode 节点）：**已被吃掉**。`crates/singlefs-core/src/admission.rs:872` `pub fn units_of_a_publish_to_land(` 把普通分配那一半的角色按槽数加进去（C573 行 `.claude/kb/checks-owed.md:405` 也写了实审 A4d 取了算），条款没写。
- 第二半（中央映射树因这次普通分配多出的删插）：**还缺**。`.claude/kb/checks-owed.md:405`「ckpt_cost 仍按空发布上界（K1）算」。代码：`crates/singlefs-core/src/admission.rs:1082` 的注释 「一次空发布在中央映射树里改的条目是这两棵树的节点」 + `crates/singlefs-core/src/admission.rs:1112` `deleted_at_most: mapped_changes_with_accounting_nodes(` 只数分配记录树、记账树与谱系 / 退役那几把；而有普通分配的发布还在映射里为每个重写的数据单元、extent 节点、inode 叶容器与根各插一把新 key（`crates/singlefs-core/src/transaction.rs:5692` 起的 `mapped_units`）。
- 量：现存的量表测试 `admission_checkpoint_cost_per_device_paths`（暂存区版）每格有一次覆盖写，我在副本里重跑了（`out/a4c-ckpt-cost-existing.txt`，460 行 `name=a4-checkpoint-cost`）。覆盖写那 8 行的 `ordinary_slots=6`，`difference`（实写固定点槽数 − 之前现算的 ckpt_cost）从 −140（1 GiB 盘）到 −167306（1 TiB 盘），460 行（含空发布）里没有一行大于 0（`grep -o difference=… | awk` 数得 0 行）。它们只有 6 个普通槽，**没有测到第二半**——没有哪一格有「普通分配多、中央映射树靠近分裂」的发布。
- 推（没量）：这一半只通过分裂数起作用：中央映射树叶容量 294（`crates/singlefs-harness/tests/mapping_tree_capacity_admission.rs`，暂存区版，首段注释「叶 294 / 内部 143」），`crates/singlefs-core/src/admission.rs:1336` `let leaf_additions_per_split_of_a_fresh_node = leaf_entries.div_ceil(2);` = 147，所以 m 个多出来的普通单元最多让叶层多切 ⌈m / 147⌉ 次、每 72 次叶切再让内部层多切一次；被删的路径数已被 K1 的巨大 `changed_paths`（量表里覆盖写的 ckpt_cost 是实写固定点的 24 倍（1 GiB：146 对 6）到 16 732 倍（1 TiB：167316 对 10））罩着。窄盘（K1 小到 5）上才可能在路径数上也不够。
- 后果：不够时先拒放行、落点那一道在任何写之前拒（`.claude/kb/decisions/03-空间分配.md:98`「提交内生块在固定点分配时找不到空槽（NoFreeSlotOnAnyDevice），在任何写之前拒绝这次发布，盘上不变」），交回 `PlacementRefused`，会话照样推抬 F。

### A5.3　C574：两槽对齐与各盘同槽

量（探针，冻结副本拷贝；`out/a3-a2-a4-a5-probe.txt` 与 `out/c576-switch-off-probe.txt`）：

```
name=a5-c574-first-half d0_unblocked_slots=105984 d0_pairs_for_data=0
name=a5-c574-first-half pre_check_for_one_two_slot_container=true
name=a5-c574-first-half landing_one_slot_ok=true landing_two_slots_aligned=Err(NoFreeSlotOnAnyDevice)
name=c576-probe-pair-fragmented result=Some("PlacementRefused { unit: InodeLeafContainer(InodeLeafContainerIndexInTree(0)), refusal: NoFreeSlotOnAnyDevice }")
```
前三行：每块盘单元区里每个奇数槽标已分配，没挡的槽 105 984 个、一对相邻的都没有；落得下那一判放行一个两槽容器，分配器取不到。第四行是整条发布路径：数据单元拿走唯一的一对、剩下 700 个互不相邻的奇数槽（够槽数、不成对）——落得下放行，`PlacementRefused` 落在 inode 叶容器上。这就是 C574 前半（两槽对齐）：**还缺**，今天只在落点那一道拒。
后半（各盘同槽）：落得下那一判不比较各盘（读数逐盘独立）；落点那一道比（`crates/singlefs-core/src/allocator.rs:162` `fn agreement_across_devices<Answer: Copy + PartialEq>(`）。现存两条钉子：用户数据各盘落点不同 → 发布层 `PlacementRefused`（`crates/singlefs-harness/tests/unequal_devices.rs:464` `Err(PublishError::PlacementRefused {`）；提交内生块各盘去处不同 → 直调分配器得 `…DifferAcrossDevicesSegmentAlignmentUndefined`（`crates/singlefs-harness/tests/unequal_devices.rs:378`），发布层在那一史里被落得下先拒成 `SpaceAdmissionRefused`（`crates/singlefs-harness/tests/unequal_devices.rs:401`）。D3 `.claude/kb/decisions/03-空间分配.md:170`「小盘没有全空段而大盘有时，在动任何状态之前拒绝（发布层统一报假性 ENOSPC）」两种写法都满足「动任何状态之前」；「统一报」在会话层满足（A5.1），在 `PublishError` 类型层不满足（两个成员）。

### A5.4　每格两种写法的后果

| 格 | 写法 | 盘上动没动 | 交回的错误 | 会话多推 / 少推 | 要动的 `crates/` 文件 |
|---|---|---|---|---|---|
| C573 第二半 | 先拒：把普通单元的删插并进「落得下」用的上界（不并进保留池用的 ckpt_cost，否则 `checkpoint_reserve_pool`（`crates/singlefs-core/src/admission.rs:1538` 一带）与切换预留的 c_max 一起变大，每次空发布多留） | 都没动 | `SpaceAdmissionRefused`（载荷是短的那一种单元的字节，C575 的混用照旧） | 同样推；先拒偏严时可能多推几串后才报 ENOSPC，而按落点那一道本可落下（上界高估） | `admission.rs`（`checkpoint_cost_of_the_version_to_build_on_with_node_capacities` 旁加一个只给落得下用的变体）、`transaction.rs:6711-6718` 调用点 |
| C573 第二半 | 留给落点：条款写「落得下的固定点上界按空发布 ckpt_cost 计，超出的在落点那一道拒」 | 都没动 | `PlacementRefused{unit, refusal}` | 同样推 | 无 |
| C574 两槽对齐 | 先拒：单元里加「容器个数」，对 `unblocked_slot_pairs`（`crates/singlefs-core/src/allocator.rs:289`，已有）减去数据单元取走的对，必要条件、不充分（单槽节点与容器抢槽对） | 都没动 | 同上 | 同上 | `admission.rs`（`PlacementRoomOnOneDevice` 加一个字段、`UnitsOfAPublishToLand` 加容器数）、`transaction.rs:6709` |
| C574 两槽对齐 | 留给落点 | 都没动 | `PlacementRefused{InodeLeafContainer…, NoFreeSlotOnAnyDevice}`（已量） | 同样推 | 无（条款加一句） |
| C574 各盘同槽 | 先拒：落得下读数按盘比「最低答案」 | 都没动 | 同上 | 同上 | `allocator.rs` 要在准入路径上算各盘最低落点；`lowest_user_data_slot`（`crates/singlefs-core/src/allocator.rs:785`）是从单元区起点的线性扫，与 `crates/singlefs-core/src/admission.rs:824` 文档里「读它们 O(这次挂载开过的聚簇段数)，不扫单元区」的取向冲突 |
| C574 各盘同槽 | 留给落点（今天） | 都没动 | `PlacementRefused{…Differ…}` | 同样推 | 无 |

### A5.5　C576：用例与变异靶子今天各是什么

`crates/mutations.tsv` 点名 C576 的第 12、76、77、78、80、145、149 行。我把每行的改动施在副本上、跑它点名的测试（基线先跑；改动写入后 `touch` 还原；脚本 `probes/mutrow.py`，原样输出 `out/mutrow-<行>-baseline.txt` / `-mutated.txt`）：

| 行 | 基线 | 施变异后 | 读法 |
|---|---|---|---|
| 76 | 绿 | 红 | panic 在 `crates/singlefs-core/src/allocator.rs:685`（小盘位图下标越界：变异让「各盘去处不同」也照盘 0 的答案取），发生在用例直调分配器的那一段，改钉没动它 |
| 77 | 绿 | 红 | 该用例本来就报 `PlacementRefused`，没改钉 |
| 78 | 绿 | 红 | 红在 `fingerprint == before`（`crates/singlefs-harness/tests/unequal_devices.rs:392`）——直调之后分配器要不变 |
| 80 | 绿 | 红 | |
| 145 | 绿 | 红 | 该用例用只供测试的开关（`crates/singlefs-core/src/allocator.rs:1658`）关掉准入，所以「装开关」这条路今天已经有一条在用 |
| **12** | 绿 | **绿（没杀死）** | 见下 |
| 149 | 没跑 | 没跑 | 靶子在 `singlefs-checker-tier`（checker 档），复核不了 |

第 12 行（`crates/mutations.tsv:12`「失败的发布不退回分配器」，锚点在 `crates/singlefs-core/src/transaction.rs:6632` `*allocator = allocator_before_this_publish;`，`publish_version_of_trees` 的 `Err` 臂）：点名的 `publish_running_out_of_space_midway_leaves_the_allocator_as_it_was` 在变异下仍绿；我另跑了整个 `overwrite_in_one_instance`、`admission_refuses_before_units_…`、`unequal_devices` 三个文件（5 + 12 + 3 条），全绿；又试了两种史——关开关让落点那一道去拒（`out/c576-switch-off-probe.txt`，`PlacementRefused{ExtentRoot…}`）、对缺失让落得下放行而落点拒（上节第四行）——变异下都绿。原因（推 + 一处文档佐证）：取落点是在分配器的**拷贝**上走完一遍才动真分配器（`crates/singlefs-core/src/transaction.rs:6885`「真发时同一个落点同样取不到」；`transaction.rs:6872-6885` 的文档），落点拒绝在拷贝上就返回，真分配器没被改过，所以回退那一句在这一类拒绝上几乎是死码。它是不是等价变异我没判完（`publish_admitted` 里 `allocate_placement_for_role(...)?` 在真分配器上，若拷贝与真发分叉才走到）。**这一行的靶子今天失效**，登记行「要不要换靶子没定」对第 12 行的答案是：要换，或判等价。

现存用例今天的结局：`devices_answering_different_places…`（`crates/singlefs-harness/tests/unequal_devices.rs:359` 起）直调分配器的断言保留（`:375-391`），发布层的断言已改钉 `SpaceAdmissionRefused`（`:401`）；`user_data_slots_that_differ…`（`:425` 起）发布层仍是 `PlacementRefused`（`:464`），没改钉。登记行说的两条改法：开关那条已有用例（行 145）、改钉那条已用在第一条用例上。

### A5.6　岔路与推翻条件

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| C573 第二半 | 先拒（并删插）/ 留给落点 | 有普通分配、中央映射树靠近分裂的发布上，实写固定点槽数 > 现算 ckpt_cost（今天量表没有这一格） | 在 `admission_checkpoint_cost_per_device_paths` 的量表里加一格「多单元写」跑一次；> 0 则两种写法的后果才分得开，≤ 0 则条款写「留给落点」即够 |
| C574 两槽对齐 | 先拒 / 留给落点 | 近满盘上「先拒」多拒的比例（落点能落下而先拒拒了）；会话多推的串数 | 按近满历史重放，数先拒与落点各自拒的次数 |
| C574 各盘同槽 | 先拒 / 留给落点 | 先拒要线性扫，成本与单元区大小成比例 | 定不定接受准入路径上的 O(单元区) 扫 |
| C576 第 12 行 | 换靶子 / 判等价 / 留着 | 能否造出让真分配器在中途被改后才拒的史（`publish_admitted` 里的 `?`） | 实现员读 `publish_admitted` 找一条会让拷贝与真发分叉的史，或判死码 |

**什么现象会推翻 A5 的判定：** (a) 近满盘上落得下放行的发布在落点那一道被拒后，会话少推或多推一串（两种拒绝在 `mounted_session.rs:912` 不同处理）；(b) 第 12 行在某个史下转红；(c) 量表里出现 `difference > 0` 的覆盖写格（推翻「第二半今天不会在窄盘以外出事」那句推论）。

## A6　几何下界（C568、C582）

### A6.1　C568：io_min 1 MiB 的盘建不了卷

**判定：部分——事实比登记行写的更严，「接受」那一条写法不够。**

登记行（`.claude/kb/checks-owed.md:400`）说拒因是区域长 S × 槽距越过区域间距 3 MiB，并说「只比同一块盘上真背根的区域（区域 0 与区域 2 同在盘 0、间距 6 MiB）能放宽到 S × 槽距 ≤ 6 MiB」。但 io_min = 1 MiB 时第一对重叠的不是区域与区域，而是**系统配置槽 1 与根环区域 0**：

- 槽距 = 4096 向上取整到 io_min 的整数倍（`.claude/kb/decisions/02-RAID条带策略.md:313`「槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍」；`crates/singlefs-core/src/system_configuration.rs:208` `floor.div_ceil(minimum_input_output_bytes) * minimum_input_output_bytes`，暂存区版），io_min 1 MiB 则槽距 1 MiB。
- 系统配置槽 i 在 i × 槽距（`crates/singlefs-core/src/make_filesystem.rs:295` 一带 `fixed_structure_extents`），槽 1 因此落在 [1 MiB, 1 MiB + 4 KiB)；根环区域 0 起点是 `crates/singlefs-core/src/root_ring.rs:96` `ROOT_RING_BASE_SLOT * SLOT_BYTES + region * ROOT_RING_PRIME_STEP * ROOT_RING_CHUNK_BYTES,`（暂存区版），`crates/singlefs-format/src/lib.rs:298` `pub const ROOT_RING_BASE_SLOT: u64 = 64;`（暂存区版）× 16 384 = 1 MiB——正是同一个字节。
- 现存用例已钉这一格（暂存区版，我跑过，绿）：`crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:404`「槽距 1 MiB（io_min 1 MiB）：系统配置槽 1 落在 [1 MiB, 1 MiB + 4 KiB)，正是根环区域 0 的槽 0。」（`out/c568-existing-test.txt`）。

我把 `fixed_structure_extents` 的式子写成一个小模型（`probes/geom.py`，不是代码，只复述式子；两个落点由上面那两条现存用例对着真代码核过：1 MiB → 系统配置槽 1 对区域 0，384 KiB 放行 / 384 KiB + 512 字节 → 区域 0 对区域 1），枚举 io_min × S，列「现行判据（六段两两比）」与「只比同一块盘上真背根的段」两种下第一对重叠（原样输出 `out/geom.txt`）：

```
io_min  S  现行 all-pairs        只比同盘（盘 0：系统配置、区域 0、区域 2、journal；盘 1：系统配置、区域 1、journal）
256K   S=16  ('reg0','reg1')      盘 0 None、盘 1 None
384K   S=16  ('reg0','reg1')      盘 0 None、盘 1 None
512K   S= 8  ('reg0','reg1')      盘 0 None、盘 1 None
512K   S=16  ('reg0','reg1')      盘 0 ('reg0','reg2')、盘 1 None
1M     S= 4  ('sys1','reg0')      盘 0 ('sys1','reg0')、盘 1 None
1M     S= 8  ('sys1','reg0')      盘 0 ('sys1','reg0')、盘 1 None
1M     S=16  ('sys1','reg0')      盘 0 ('sys1','reg0')、盘 1 ('reg1','journal')
（其余 io_min ≤ 512K 且 S ≤ 4、io_min ≤ 256K 且 S ≤ 8 的格两种判据都 None）
```
读法：放宽成「只比同盘」能多收的是 io_min 256 KiB–512 KiB 里 S 偏大的那些格（含 mkfs 默认 S = 8 配 io_min 512 KiB 这一格，现行拒、放宽后收）；对 io_min 1 MiB 无效——盘 0 上系统配置槽 1 与区域 0 仍重叠。**所以 C568 标题那种设备（io_min 1 MiB）要接受，除了「只比同盘 + 改 mkfs 清零」还得动格式常量**（`ROOT_RING_BASE_SLOT`、`ROOT_RING_PRIME_STEP` / `ROOT_RING_CHUNK_BYTES` 或系统配置槽 1 的位置；`JOURNAL_RING_START_SLOT = 1024` 即 16 MiB 要不被区域 2 的末端 7 MiB + S × 槽距顶到），即改新池新建文件的字节与 D22 已定项 16 第 2 句。

另一处与「接受」相关的现成约束：`crates/singlefs-core/src/make_filesystem.rs:533` 的注释「三个区域在每块盘上都清，不按 `region_devices` 只清本盘要读的那几段」，写明这是有意的（别的读者以后改成扫全部盘时，不留下「本池 fsid、自证得过、这一版读不到」的根记录）；放宽成只比同盘等于要连这条一起改，并要求读者（`recovery`、checker 的 `image.rs`）永远只读本盘背的区域。

候选写法：

| 候选 | 定义 | 要动的 `crates/` 文件 | 要的用例 |
|---|---|---|---|
| 甲 不接受，把拒的那一格写进 D2 已定项 19 射程 | 写明「拒 `sp + 4096 > 1 MiB` 或 `S × sp > 3 MiB` 的盘」（`sp` 为槽距；另有判定宽度对齐那一判 `make_filesystem.rs:444`，按 io_min 与区域起点 1 / 4 / 7 MiB 的整除关系再拒一批，模型没复述它）；S = 8 时 sp ≤ 384 KiB 收、512 KiB 起拒（现存用例钉了 384 KiB 收、384 KiB + 512 字节拒） | 不动 `crates/`；`make_filesystem.rs:432-443` 的判据已是这条 | 现存 `core_review_…:404`（1 MiB）与 `core_review_…:438`（384 KiB / +512）；补：512 KiB 配 S = 4 收、S = 8 拒的一对，和 1 MiB 配 S = 4 / 8 / 16 都拒的三格 |
| 乙 接受：只比同盘 + 改清零，io_min 1 MiB 仍拒 | 判据改成按 `parameters.region_devices` 分盘比；清零只清本盘背的区域 | `make_filesystem.rs`（`check_geometry` 的两两比、`make_filesystem` 里 `write_zeroes_at(region_start(…))` 的清零循环，`crates/singlefs-core/src/make_filesystem.rs:559`）；读者侧 `recovery.rs` / checker `image.rs` 要核「只读本盘背的区域」 | 分盘判据的单测（盘 0 / 盘 1 各一组）；清零不互相盖住的镜像测试；256–512 KiB 的几何各一条 mkfs → 挂载 → 发布的往返；读者不读未背根区域的用例 |
| 丙 接受 io_min 1 MiB（动格式常量） | 乙 + 挪根环基址或系统配置槽 1 | 乙的全部 + `singlefs-format/src/lib.rs`、`root_ring.rs`、`system_configuration.rs`、layout 字节表（`layout/01-first-txn.md`） | 乙的全部 + 新常量的字节表对账 + 往返 |

### A6.2　C582：挂载一侧盘容量下界

**判定：部分。** 实现里「挂载走到哪一步」已经有定论（已量，下表）；条款该写哪个下界仍没定。

条款：`.claude/kb/checks-owed.md:410`「挂载一侧的实现取的是前者，条款没写该取哪个」（前者 = 完整槽数 ≥ 单元区起始槽号，`crates/singlefs-core/src/recovery.rs:2019` `pub fn every_device_reaches_the_unit_area_start(`，暂存区版）；后者 = mkfs 的下界（`crates/singlefs-core/src/make_filesystem.rs:418` `if end_of_the_units_written_by_make_filesystem > smallest {`，实例表 2 槽 + 树表 1 槽共 U + 3 槽）。

量（探针 `probes/c582_probe.rs`：只做过 mkfs 的池，4 GiB 盘 mkfs 之后把盘换成较短的稀疏盘并带上原镜像，容量 = (U + extra) × 16 KiB，U = 50176；可写与只读挂载各一次；`out/c582-probe.txt` 共 84 行）：

| extra（槽，相对单元区起点 U） | 只缩盘 1（盘 0 完整） | 两盘都缩 |
|---|---|---|
| 0、1、2（低于 mkfs 下界 U + 3） | 可写挂载 `Err(Recovery(AllocationRecordOutsideThePoolGeometry{ what: "分配记录的跨度越过单元区末尾" }))` | `Err(Recovery(UnitUnreadable{ slot: 50176 或 50178 }))` |
| 3 … 23 | `Err(SpaceAdmissionRefusedBeforeAcquisition{… available: 负数 …})`，extra = 3 时盘 1 可用 −344 064 字节，每多一槽少 16 384，到 extra = 23 时 −16 384 | 同左 |
| ≥ 24 | `Ok`（可写挂载成功） | 同左 |

只读挂载在所有 extra ≥ 3 都报 `Open(TreeTableHasNoEntries)`（mkfs 之后一个文件都没有，与容量无关）。没有 panic，没有走到比拒绝更深的坏结果。读法：低于 U 的由 `every_device_reaches_the_unit_area_start` 拒（既有用例 `crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:631`，暂存区版）；落在 [U, U + 3) 的盘过得了那一道，被后面的分配记录几何检查 / 单元读不出拒；[U + 3, U + 24) 被准入式拒（扣掉的合计 24 槽：extra = 3 时容量 3 槽、可用 −21 槽；拆成已分配 3 槽加切换预留与 checkpoint 保留池，是我的推，没单独量；这个池没有文件版本，环窗口不扣，`crates/singlefs-core/src/admission.rs:1552` 的文档「从第一个文件版本那次发布起计」）；≥ U + 24 才可写挂载。登记行「判别力自证：把挂载一侧的下界改回不检查，…必须由拒绝变成走到更深处的坏结果」只对低于 U 的盘成立（`crates/singlefs-core/src/allocator.rs:377` 的 `.expect("建空闲图的盘末尾不在单元区起点之前…` 会 panic）；对 [U, U + 3) 这个中间区，那条检查本来就放行，改回不检查什么都不变，所以这一格的自证对中间区无法成立。

候选写法：

| 候选 | 定义 | 要动的文件 | 代价 |
|---|---|---|---|
| 甲 保持「完整槽数 ≥ 单元区起点」，条款写成前置条件 | 写「这一判只是建空闲图的前置；更严的容量由后面的记录几何检查与准入式判」 | 不动 `crates/` | 中间区的拒绝类型是三种之一，不是专门的「盘太短」 |
| 乙 取 mkfs 的下界（U + 3 槽） | 挂载一侧新判一道，与 mkfs 共用同一个式子 | `recovery.rs:2019` 一带加判；`make_filesystem.rs:412-423` 里的式子抽成共用函数；新的 `RecoveryFailure` 成员 | 中间区 [U, U + 3) 得到专门的拒绝；多一个成员与用例 |
| 丙 取「可写挂载走得通」的下界 | 不写常数，写成「准入式可用 ≥ 0」 | 无（今天就是这样，U + 24 是这个池的数，随版本涨） | 下界不是格式常量，条款只能写关系 |

### A6.3　岔路与推翻条件

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| C568 接不接受 | 甲 / 乙 / 丙 | 真设备 io_min 的分布（mdraid chunk 1 MiB 之外还有谁）；放宽成只比同盘后读者是否真的从不读未背根区域 | 先查 recovery / checker 的读法；用户定要不要动格式常量 |
| C582 下界取谁 | 甲 / 乙 / 丙 | 有文件版本的池上中间区的结局（我只量了 mkfs 之后无文件的池） | 在带文件的池上重量同一张表 |

**什么现象会推翻 A6：** (a) 对 `fixed_structure_extents` 的式子枚举时发现 io_min 1 MiB 在只比同盘的判据下盘 0 上没有重叠（模型与真代码对不上）——两条现存用例已对过 2 个点；(b) 中间区 [U, U + 3) 在某个池（带文件）上被挂载成功并写出坏结果。

## 岔路汇总（候选与翻面观测都在各格里；这里只列编号，不带倾向，状态都是「开着」）

| 编号 | 问题 | 候选 | 所在 |
|---|---|---|---|
| F1 | 需求的单位写不写、写哪个 | 盘字节每盘相同 / 全部副本之和再摊 | A1.5 |
| F2 | `df` 的设备维 | 逐设备取小 / 全池之和 / 之和 ÷ 盘数 | A1.4–A1.5 |
| F3 | `df` 是不是随新增 inode 数变的函数 | 单个数 / 随新增 inode 数变 | A1.5 |
| F4 | 「活元数据」 | 删掉 / 现行版本引用的元数据块 / 别的集合 | A1.4–A1.5 |
| F5 | C356 行去向 | 还清 / 改写 / 留着 | A2.3–A2.4 |
| F6 | 式子要不要装扣住 | 第十项 / 读数的已分配含扣住 / 落得下那一判数保留项 / 不改写成已知限度 | A3.4–A3.5 |
| F7 | 写错残留的扣住位怎么放 | 留到下一次抬 F / 重发后按落满与否放 / 失败返回时换回分配器 | A3.4–A3.5 |
| F8 | `df` 报不报段内关着的空槽 | 报 / 不报 / 另报一项 | A4.3–A4.4 |
| F9 | C573 第二半 | 先拒并删插 / 留给落点 | A5.4、A5.6 |
| F10 | C574 两槽对齐 | 先拒 / 留给落点 | A5.4、A5.6 |
| F11 | C574 各盘同槽 | 先拒 / 留给落点 | A5.4、A5.6 |
| F12 | C576 第 12 行（与 149 行） | 换靶子 / 判等价 / 留着；149 行我没法复核 | A5.5–A5.6 |
| F13 | C568 | 不接受并写射程 / 接受（只比同盘 + 改清零，1 MiB 仍拒）/ 接受 1 MiB（动格式常量） | A6.1、A6.3 |
| F14 | C582 | 保持完整槽数 ≥ 单元区起点 / 取 mkfs 的 U + 3 / 取准入式可用 ≥ 0 | A6.2–A6.3 |

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A1（C370、C371） | 部分 | 需求取盘字节已在实现里（`admission.rs:1478`，乘盘数变异使 1/21 条单测转红），条款没写；`df`、「活元数据」在 `crates/` 零命中、条款里也没定义 |
| A2（C356） | 已被吃掉（符号与口径）；残余是 A3 的扣住与已知双扣 | `isolated_slots()` 是 `u64` 计数，D28 第九项已改成实际隔离的集合；对拍：隔离空槽差 0，隔离已分配槽差 −2（少报，安全），真「净释放历史」没造 |
| A3（C570、C546 两格） | 部分 | 读数把扣住的槽算空闲（4 槽 → 读数多 65 536 字节），落点读数算挡着；成功的抬 F 里准入读不到扣住位，写错之后残留 15 槽（已量）；「式子放行、用户发布落地、抬 F 被拒」整条历史没造 |
| A4（C571） | 部分 | 段外槽对读数已在分配器里；300 块提交内生块的合成史里 5 段 320 槽对用户数据关着；C545 行点名的见证用例已不在，补了核心层一条（式子可用 320 槽放行、段外 0 对） |
| A5（C573） | 部分 | 普通分配节点已算；删插没算（量表无多单元写格，推：只经分裂数起作用） |
| A5（C574） | 还缺 | 两槽对齐、各盘同槽都只在落点那一道拒；已量落得下放行而落点拒（整条发布路径） |
| A5（C576） | 部分 | 第一条用例已改钉、第二条本来是 `PlacementRefused`、开关用例已有；变异行 76、77、78、80、145 在现码上红，第 12 行不红（三种史），149 行复核不了 |
| A6（C568） | 部分 | 1 MiB 的第一对重叠是系统配置槽 1 对区域 0；「只比同盘」放不宽 1 MiB，只放宽 256–512 KiB 里 S 大的格；要接受 1 MiB 须动格式常量 |
| A6（C582） | 部分 | 两个下界之间的盘今天挂载一律被后面的门拒（记录几何 / 单元读不出 / 准入式），没有更深的坏结果；可写挂载的实际下界是 U + 24 槽（mkfs 之后无文件的池） |

## 没做什么

- 不判别：不替主 agent 采纳任何候选、不出判决、不写倾向；岔路单（`<轮>-forks.md`）留给主 agent。
- 没复核：`crates/mutations.tsv` 第 149 行（靶子在 `singlefs-checker-tier`，checker 档，按定义不跑）。没跑任何 `.claude/gate.d/` 下的脚本、没跑 checker 档测试、没起 GPU、没跑全表变异；变异行是我把单行改动施在草稿副本上跑它点名的 harness 测试（harness 档）。
- 没造的历史：A2 的「崩溃恢复落旧根、被抛弃时间线净释放」真历史；A3.3 的整条「式子放行 → 用户发布落地 → 抬 F 被预演拒」历史；A5 C573 的「多单元写、中央映射树靠近分裂」量表格；A4 的长挂载里「段内关着的空槽 / 式子可用」比例；C582 在带文件的池上的中间区结局（只量了 mkfs 之后无文件的池）。第 12 行变异没判完是不是等价变异。
- 模型的边界：`probes/geom.py` 是对 `fixed_structure_extents` 式子的复述，不是代码，只被两条现存用例对过两个点；其余探针都是对冻结副本拷贝上的真代码调用，探针源码不入 `crates/`。
- kb 长行没整行抄，抄的是与结论有关的整句并用「…」切开（cite-check 只认这样的写法）；整行原文按行号可取。
- 一次前台命令的 `timeout` 我设了 480000 毫秒（共性约束上限 240000），实际耗时约 100 秒，已跑完；之后的命令都不超过 240000。
- 引文行号里，`mounted_session.rs`、`recovery.rs`、`root_ring.rs`、`system_configuration.rs`、`singlefs-format`、`core_review_geometry_back_chain_and_empty_inode.rs`、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`mapping_tree_capacity_admission.rs`、`admission_checkpoint_cost_per_device_paths.rs` 是 2026-10-12 暂存区那一版（非 sha256 点名的七份），已在引用处标注。

## 要主 agent 跑的命令

没有。需要核我的数时：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m3-admission-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-admission-r1/repo --work <新的草稿目录>
```
（环境变量 `RERUN_GROUPS=probes` / `mutrows,demand` 可分两段跑；`RERUN_TARGET_DIR` 指一个可复用的编译目录。末尾 `name=rerun_compare` 行与 `out/` 逐文件比。）

## 模型目录

`research/prompts/m3-admission-r1-sonnet-model/`：`probes/`（`a3_probe.rs`、`c582_probe.rs`、两个追加进现存用例文件的探针片段、`geom.py`、`mutrow.py`、`mutgen.py`）、`out/`（每一步的原样输出）、`tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps` 与加了 `RERUN_GROUPS` / `RERUN_TARGET_DIR` 两个环境变量）。

## 草稿目录

`~/.local/share/singlefs/work/m3-admission-r1-sonnet/`：交回前我已删掉自己建的编译目录（`target`、`target-rerun`，各约 2.6 GB）与仓副本（`work`、`rerun-A`、`rerun-B`、`rerun-C`），都能由模型目录里的 `tools/rerun.sh` 重建；留下的只有各步的 `.log`、辅助脚本（约 416 KB），不入库（原样输出已挑进模型目录 `out/`）。
