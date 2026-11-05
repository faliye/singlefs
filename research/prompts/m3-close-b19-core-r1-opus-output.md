# m3-close-b19-core-r1 攻方腿（Opus）报告

日期：2026-10-29。轮：m3-close-b19-core-r1，第一轮。攻击面：过度外推与越位、内部矛盾、哪一条最脆弱；R3（C745）主攻。
被攻的代码与开工快照逐字节相同：`research/prompts/m3-close-b19-core-r1-snapshot/` 里五份 crates 与四份 kb 的 sha256 与主工作区此刻的相等（开工时 `sha256sum` 核过）。

这条腿没写模型、没起 cargo：下面的打中全是读码推的，没在副本或 harness 上跑过；没有模型目录与复跑命令。

## 各格判定一览

| 格 | 结论 | 依据的强度 |
|---|---|---|
| R3（C745） | 题面那两句字面都没打中（旧记录被拿去顶上：只在冻结时直接调公开的抬 F 上限函数才走得到，且只算一个数、不写盘；写根环而不经记根又不冻结：今天的写根路径都堵上了）。打中一条同机理的邻格 K3-1：冻结期间分配器退回发布之前那一份，会话的整理入口照样发 discard，能把冻结那次已落盘的根引用的单元所在整段 discard 掉 | 读码推的，没造用例 |
| R1（C773 第 0 代根） | 推荐乙，但乙的改动面被低估（GPU 走读内核也解 p；写者要把槽连同钉值一起带到写根那一刻）；丙低估了后果（写者门槛高一档是回收偏松，不只是 checker 误红）；甲若是另存钉值，checker 就失去独立现算 | 读码推的 |
| R2（C724） | 第 3987 行那处任何输入下都走不到（结构上被第 3944 行先拦）；第 3944 行在这个进程的准入路径上走不到，前提是「这个进程写的每条根都经记根」与「F 不高于现行版本的 txg」；推荐写明走不到并在本进程那一层断言，不加进单子；共享函数里保留这个错（有外部调用方） | 读码推的 |
| R4（C874） | 「观测记录一行」若挂在挂载的成功输出上，拒的那一支拿不到；原因只能随错带出。观察者「只判整写不判撕裂」那行变异在真实录制的补写流上杀不掉（两轮之间有屏障，撕裂态造不出新死盘），要另给一个合成态的必红用例；补写失败回卷那一支不在不注入故障的录制流里 | 读码推的 |

## R3（C745）：挂载期间写故障下的换类

### 攻了哪些路径

写根环槽的只有一处提交步：「CommitStep::WriteRootRecordForceUnitAccess {」那一支（`crates/singlefs-core/src/transaction.rs:541`），经 `persist_publish_writes` 被四个调用方用：带单元发布的落盘（失败冻结）、重发冻结那一次、写行（失败冻结）、零单元发布（不经分配器、不冻结）。逐个看了换类（`record_root_written_by_this_process` 改表）与分配器整份换回的每一处：

| 路径 | 写根环之后表与盘一致吗 | 结论 |
|---|---|---|
| 零单元发布（挂载里写行、暖机） | 记根写在成功才进的那个闭包里：「record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(」（`crates/singlefs-core/src/mount.rs:7756`）；失败即整个挂载返回错：「publish_failed_after_acquisition(&pool, &[], cause))?;」（`crates/singlefs-core/src/mount.rs:7784`），分配器随错丢掉 | 没打中 |
| 带单元发布、写行落盘途中失败 | 盘上那一槽可能已是新根，表退回旧类：「std::mem::replace(allocator, allocator_before_this_publish);」（`crates/singlefs-core/src/transaction.rs:1574`），同时冻结 | 表与盘不一致，靠冻结挡 |
| 冻结期间读表的入口 | 抬 F 一串第一道判冻结「if let Some(frozen) = allocator.frozen_publish() {」（`crates/singlefs-core/src/mount.rs:4771`），管理员回退同一句（`crates/singlefs-core/src/mount.rs:9704`），收改动「if let Some(frozen) = self.session.allocator.frozen_publish() {」（`crates/singlefs-core/src/mounted_session.rs:5159`），发布路径「refuse_while_a_publish_is_frozen(allocator)?;」（`crates/singlefs-core/src/transaction.rs:1835`） | 顶上那两处（「roots.push(*root_record_self_verified_on_one_rebuild_pass);」`crates/singlefs-core/src/mount.rs:3800`、「.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),」`crates/singlefs-core/src/mount.rs:4685`）在会话里冻结时走不到 |
| 公开的准入抬 F 上限（`raise_rollback_floor_to_the_admission_ceiling`、`push_one_floor_raise_within_the_admission_budget`） | 先算上限、再进抬 F 一串才判冻结：冻结那次写的槽若在挂载时记成「两遍里只有一遍自证过」、这一遍又读坏两次，算上限时顶上的是挂载时那条旧根 | 字面打中，后果只是交回的上限与停因（上限不高于 F 时交 `FloorAlreadyAtTheCeiling` 而不报冻结），不写盘；要读故障叠写故障，会话里走不到（被拒为冻结不是空间不够，`push_one_floor_raise_for_a_refusal_for_space` 先返回） |
| 整份换回分配器的各处（抬 F 被拒、回退被拒、发布落盘之前被拒） | 都在任何根写之前，或只在没冻结时换回 | 没打中 |
| 挂载期间那一份共用的读（`DeviceReadOnceInThisMount`） | 写报错就丢掉那几块记下的字节、写成就记成写下的 | 没打中 |

### 打中：K3-1 冻结期间的 discard 发到冻结那次已落盘的单元上（推的，没造用例）

不是题面那两句的字面：discard 不写根环槽，也不读根环表。是同一机理（冻结时分配器整份退回发布之前、盘上已有新根）在空闲图上的出口：C745 那一行攻方读码写的「盘上那一槽可能已是新根、表里还是旧记录」，换成空闲图同样成立，而空闲图有一个不经发布路径、冻结时不拒的写出口。

历史（每一步指到许可它的那一句）：

1. 可写挂载之后的会话，池接近满。一次用户改动被空间拒，`publish_user_change` 推一串抬 F 再判。抬 F 那一串回收、末尾放开扣住位：「allocator.release_reclaim_holds();」（`crates/singlefs-core/src/mount.rs:5096`）；回收那一刻段记进待 discard：「self.segments_reclaimed_not_yet_discarded.insert(segment);」（`crates/singlefs-core/src/allocator.rs:847`）。取一段 S 在这一串里整段回收空。
2. 同一次调用里重发这次改动，它的单元落进 S（用户数据取最低的空闲槽对、提交内生块开最低的全空段；推的：落不落进 S 看几何，S 是这一串刚放出的空间，被拒的改动正是要用它）。
3. 这次发布落盘：单元、屏障、记录、屏障、根槽 FUA 都成，系统配置槽轮换那一写报错（注入点）。分配器换回发布之前那一份（上一表第二行），那一份里 S 仍整段可再分配、仍在待 discard 表里；发布冻结。`publish_user_change` 返回错，discard 只在成功那一支发：「published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(」（`crates/singlefs-core/src/mounted_session.rs:2132`）。
4. 调用方接着调 `MountedSession::run_compaction`：入口只核盘表与有没有树，不判冻结；产品路径在批大小没定那一判就停：「CompactionRunOutcome::Stopped(CompactionRunStop::BatchSizeNotDecidedByTheClauses),」（`crates/singlefs-core/src/mounted_session.rs:3209`），回到入口之后不看结局照发 discard：「run.discards = discard_the_segments_reusable_as_a_whole(」（`crates/singlefs-core/src/mounted_session.rs:2301`）。后台入口同一句（`crates/singlefs-core/src/mounted_session.rs:2492`），只是要先判起跑。
5. discard 取段：「let segments_to_discard = allocator.take_the_segments_to_discard_on_every_device();」（`crates/singlefs-core/src/mounted_session.rs:2920`），S 在退回的那一份里「&& self.used_per_segment[segment_index] == 0」（`crates/singlefs-core/src/allocator.rs:869`），交出去。孤儿头那一判拦不住：它只认现行那一版实例表里有行的写序实例「head.is_unpublished_by_the_row(row))」（`crates/singlefs-core/src/mounted_session.rs:3022`），而这次挂载写的行只给上一个实例到要取的号之前：「(first_row_instance..instance_to_acquire.0)」（`crates/singlefs-core/src/mount.rs:5846`）——本实例自己没有行，冻结那次的单元头写序实例是本实例，查不到行，按「No」算，S 照发。
6. 重发成功之前断电。盘上冻结那次的根已 FUA 落盘、它的记录也落了；恢复择到它（盘上最新的可读有效根），它引用的单元所在的 S 收过 discard（结局三态之一：读回全零、旧内容、不定）。

可写成 harness 用例的形态：
- 注入点：用户改动那次发布的系统配置槽轮换写（任一块盘）`InjectedFault::WriteFails`；之前用小池把改动逼进「被空间拒 → 抬 F → 重发」那一支，并核这次发布取到的落点里有一个在抬 F 刚回收空的段里（核不到就换几何重造，不算通过）。
- 不必崩溃就能红的断言：冻结之后调 `run_compaction`，交回的 `discards.applied` 与 `discards.unsupported` 里不许有一段装着 `allocator.frozen_publish()` 那次 `writes().units` 的落点。今天（推的）会有。
- 崩溃那一形：不重发、丢掉会话，在镜像上把那一段按全零那一态落下，跑池级 checker 并重开读回；期望 checker 绿；今天（推的）红在冻结那次发布的单元读不出或校验和对不上。

四句：
- 分不分辨臂：C745 没给臂。这一格对 R3 的意义是「冻结期间只靠发布路径第一道拒，挡不住非发布的写」，同样打中今后任何在冻结时还会写盘的入口。
- 系统看得到判别它的东西吗：看得到，会话手里就有 `allocator.frozen_publish()`。
- 满足判据字面哪一分句：C745「每步照池级 checker 与回退结局判」那一分句（崩溃那一形上 checker 红，推的）；不是「找到一条写根环槽而不经 `record_root_written_by_this_process`、又不冻结的路径」那一分句。
- 改法在打中的格上还中不中：题面没给改法。我提两个，只在读码上推过、被攻过零轮、没实现没跑：甲，`discard_the_segments_reusable_as_a_whole` 开头判冻结、冻结着一段都不发（三处调用一并罩住）；乙，`run_compaction` 一族与后台入口第一道照 `accept` 判冻结、拒（罩住 2301、2492 两处；2132 那一处只在成功时走，冻结时本来到不了）。两者都碰得到打中的格（推的）。

### R3 没打中的形状与取样

- 题面第二句「写根环槽而不经 `record_root_written_by_this_process`、又不冻结」：四个写根调用方逐个看了（上表），零单元发布是唯一不经分配器、不冻结的，只在可写挂载取号之后那一串与 mkfs 同一个进程的暖机里用；前者失败整次挂载报错，后者不在「挂载期间」。
- 题面第一句「换类之后旧记录被拿去顶上」：顶上的两处只在抬 F 那一串与算上限里；会话里冻结时这两处前面都有冻结那一判，或前面那次发布被拒成冻结（不是空间不够）就不推抬 F。只有直接调公开的准入抬 F 上限函数时走得到（上表第四行），不写盘。
- 重发冻结那一次（`resend_the_frozen_publish`）整份换上冻结时记好的那份分配器：冻结期间还有写盘的只有 K3-1 那一个出口；冻结期间别的读也不改表。
- 进程内重读读出被抛弃根时那一槽从表里拿掉、不补隔离：那是读故障一类，挂载时读坏的被抛弃根影子账本来罩不到（allocator.rs 第 1258 行起那段注释写明的已接受缺口），不算写故障下的新形态。
- 取样：只读码，没跑。看了 `transaction.rs` 的四条落盘路径与重发、`mount.rs` 的抬 F 一串、管理员回退、取号之后那一串、`mounted_session.rs` 的收改动、整理三个入口、推环、discard；没看 `unlink.rs`、`truncate.rs` 与快照那几条发布的内部（它们都经 `publish_version` 一族，第一道判冻结）。

## R1（C773 第 0 代根那一形）

背景：mkfs 把第 0 代根种在三个区域的槽 0（`crates/singlefs-core/src/make_filesystem.rs:825` 的 `genesis_root_ring_slots`），区域 1、2 的槽 0 按落点公式最早住 txg 1、2，第 0 代根是公式之外的例外。写者算挂载时读坏的槽「可能住过的最旧 txg」只走公式：「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」（`crates/singlefs-core/src/mount.rs:2947`），所以区域 1 槽 0 钉在 1，不是 0。

### 对丙（认成已知结局）：后果被低估

实现员量到的那一对（读故障下 (t, p) = (1, 1)，无故障下 (0, 255)，线索，我没复跑）说明读故障让写者的回收门槛高了一档。池级 checker 按镜像现算：镜像上区域 1 槽 0 的第 0 代根读得出、按实例表有效、txg 0，现算门槛是 0，写者写的 t 是 1，I-7.18 红。这一红不是 checker 误判合法历史，是写者真的把释放代 1 的槽当可回收（推的）：F 为 0 时第 0 代根是恢复的候选，它引用的 mkfs 单元正是第一次写的发布（txg 1）换下的那几个，门槛 1 让它们可以被回收、再发出去，而第 0 代根可能还在区域 1 槽 0（挂载时只是这一次读坏）。要看得见后果还得另一处故障让恢复落回第 0 代根，所以是双故障下的数据损坏，不只是一条不变量红。读故障本该只让写者更保守，这里反过来，丙把它记成已知结局等于认下一处回收偏松。

### 对乙（写者钉 0、p 存公式槽、checker 把槽 0 认成钉值 0）：方向对，改动面被低估

乙的语义成立（推的）：钉值是「那一槽可能住过的最旧 txg」，三个区域的槽 0 都可能还住着 txg 0 的第 0 代根，钉值都该是 0；其余槽仍是 3·槽 + 区域。p 这一个字节存槽的编码（3·槽 + 区域），钉值由槽推（槽号为 0 的取 0，否则取编码本身），解法唯一，不用改格式。保守的代价只在 F = 0 时有：max(F, 0) 与 max(F, 1) 只在 F = 0 时不同，F 抬过一次就没差别。

题面说「写者与 checker 各改一处」，实际至少要动这几处：
1. 写者算钉值：上面 2947 那一圈之外，槽 0 另按 `genesis_root_ring_slots` 认最旧 txg 为 0；这一圈对 first_txg 为 1 的第一次可写挂载连区域 1、2 的槽 0 都不交回（年轻环里还没轮到），乙下它们也该钉。
2. 写者写 p：今天 p 的字节就是钉值，「crate::root_record::PinnedRingSlot::of_the_lowest_pin(self.lowest_pinned_txg)」（`crates/singlefs-core/src/allocator.rs:1812`），`ReclaimFloorWhenTheRootWasRecorded` 只带最小钉值、不带槽。钉值 0 写成字节 0，checker 解成区域 0 槽 0，I-7.19 去查错的槽（实现员那行变异写出 p = 0 时看到的冲突就是这一形）。乙要把「最小钉值所在的槽」一路带到写根那一刻，`of_the_lowest_pin` 的签名与 Verus 规格（`crates/singlefs-core/src/root_record.rs:151`）跟着改。
3. 池级 checker：「region: pinned_txg % geometry.regions,」（`crates/singlefs-checker/src/walk.rs:9185`）那一段解码，钉值与槽分开取。
4. GPU 走读内核也解 p，题面没算进去：钉值直接取字节「lt64(u64_of(pinned), oldest))) {」那一行（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5124`），槽取「let pinned_region = pinned % g.regions;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:5141`）。只改 CPU 一侧，两边判 I-7.18 就分叉。另有 `crash_injection.rs`、`crash_judge_conclusions.rs` 与几个实验二进制读这个字节（`grep` 命中，没逐个看它们拿它做什么）。

几个槽同时钉在 0 时（区域 0、1 的槽 0 都读坏），p 只能存其中一个，I-7.19 只查那一槽；今天也只查最小钉值那一槽，覆盖面不变。

### 对甲（p 改存槽号、与钉值脱钩）

若「脱钩」是另加一个字段存钉值：checker 的 I-7.18 就照写者写的钉值算，不再从槽独立推，写者钉错值它看不出（`.claude/rules/fs-design.md` 要运行时与 checker 用不同算法，这一处退成照抄）；要保住独立现算，checker 还是得从槽推钉值，第 0 代根那条例外照样要写进 checker——那就是乙，外加一次格式改动。若「槽号」就是 3·槽 + 区域、钉值由槽推，甲与乙是同一个改法。

### R1 结论

推荐乙，条文写成「钉值 = 那一槽可能住过的最旧 txg：三个区域的槽 0 是 0（第 0 代根），其余是 3·槽 + 区域；p 存槽的编码」。改动面按上面四处算，GPU 内核与 CPU checker 同一批改。会红的用例：实现员那一形（区域 1 槽 0 挂载时读坏或读回全零）上断言写行那次的 (t, p) = (0, 1)、池级 checker 与 GPU 判 I-7.18、I-7.19 都绿；变异两行：写者槽 0 不按第 0 代根认（t 回到 1，I-7.18 红）、写者把钉值当字节写（p = 0，I-7.19 查错槽；要让它红，用例得让区域 0 槽 0 住着本实例写的根，即环转过一圈之后的那一形）。
推翻条件：找到一个槽 0 在挂载时读坏、而它确定不可能住着第 0 代根、钉 0 会让某条不变量误红的盘面。

## R2（C724）：准入抬 F 上限里同名的 NoValidRoot

C724 那一行（`.claude/kb/checks-owed.md:439`）要「两处要么加进单子、要么写明走不到」。两处不一样：

### 第 3987 行：任何输入都走不到（结构上）

`ceiling_from_newest_and_non_empty_roots` 交 `None` 只有一条路：第 4 新的不同状态没有、且最旧有效根也没有，「.or(oldest_valid_root)?;」（`crates/singlefs-core/src/mount.rs:4044`）。调用处给的最旧有效根是「root.checkpoint_txg).min(),」那一行（`crates/singlefs-core/src/mount.rs:3985`），`valid` 不空它就是 `Some`；`valid` 为空时第 3944 行已经先返回。所以第 3987 行那个 `.ok_or` 与盘面、故障都无关，是死分支，能改成 `expect`（写明「valid 不空，第 3944 行先拦」）。这一处不该进单子：进单子就是给一个造不出来的拒因立条款。

### 第 3944 行：这个进程的准入路径上走不到，有两个前提

`valid` 是读出（或顶上）的根里「root.checkpoint_txg >= current_floor)」（`crates/singlefs-core/src/mount.rs:3918`）且不按实例表被抛弃的。这个进程的准入（`rollback_floor_ceiling_of_this_process`）传进来的 F 是现行那一版根上的 F。现行那一版的根是这个进程写的（可写挂载至少写了写行那条，之后每次发布的根也是），它的槽记成有效根「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」（`crates/singlefs-core/src/allocator.rs:2748`），于是在「知道住着根、又不是被抛弃根、也不是两遍里只有一遍自证过」的那一类里：读坏就重读，仍坏就拒成重读那一拒，走不到第 3944 行；读得出就进 `valid`（txg 不低于它自己带的 F；本实例在现行实例表里没有行，不会被判抛弃）。

两个前提都是别处的不变量，不是这个函数自己保证的：
1. 这个进程写出的每一条现行根都经记根进了表（正是 C745 要守的那一条；冻结那次的根不是现行那一版，不碍事）。
2. 根上带的 F 不高于它自己的 txg。

所以推荐「写明走不到」，写法是：共享函数里保留这个错——它是公开的，另有调用方（「let ceiling = rollback_floor_ceiling(」`crates/singlefs-checker-tier/src/on_device_modes.rs:332`，「let Ok(ceiling) = rollback_floor_ceiling(」`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:1988`，加上用例），用例里手拼的分配器与空的已知槽集合在根环全读坏时合法地走到它——只在 `rollback_floor_ceiling_of_this_process` 那一层把这个错断言成不可能（`debug_assert` 或改成内部错并写明是哪条前提破了），条文写明「这个进程的准入走不到，前提是上面两条」。加进单子的那条路给的是一个只有变异造得出的拒因。

### 变异那一行（HEAD 第 5240 行，此刻工作区第 5238 行）在推荐下怎么红

那一行把「if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {」（`crates/singlefs-core/src/mount.rs:3805`）换成 `if true`：知道住着根的槽读坏也当没有根，根环全读坏时 `valid` 为空，走到第 3944 行。今天它红在用例要的是重读那一拒、拿到的是 `Recovery(NoValidRoot)`。推荐之下：共享函数照旧交 `NoValidRoot`，用例照旧红；若在本进程那一层加了断言，变异下是断言失败（panic），用例照样红，但变异行的说明「报单子外的 Recovery(NoValidRoot)」要跟着改成断言那一句。把第 3944 行直接改成 `expect` 不可取：公开函数在外部调用方那里会 panic。

推翻条件：造出一个这个进程写过的现行根读不出、而它的槽不在重读即拒那一类里的盘面（例如有一条根没经记根就成了现行那一版）。今天没找到这样的写根路径（R3 那一节的表）。

## R4（C874）：补写那一支的可观测与崩溃流

C874 那一行在 `.claude/kb/checks-owed.md:540`。

### 可观测输出放在哪：只能随错带出

补写闸没过时整次可写挂载返回错：「Some(_gate_not_met) => return Err(device),」（`crates/singlefs-core/src/mount.rs:6023`），调用方拒成「CheckingEachDeviceBeforeTheAcquisition { device: DeviceIdentity },」（`crates/singlefs-core/src/mount.rs:374`）。拒的这一支没有挂载的成功输出，「观测记录一行」若挂在挂载交回的输出上，恰好在要看的那一支拿不到——这个候选只有在观测走一条与成败无关的通道（调用方交进来的观测者）时才成立，今天没有这条通道。所以推荐错误成员带原因：这个成员加一个字段，取值是闸的那几件，外加一件「两槽里有一槽设备报读错」（第 6009 行那一支「if slots_reporting_a_read_error > 0 {」（`crates/singlefs-core/src/mount.rs:6009`）今天拒成同一个成员，同样分不出原因）。`RewriteGateNotMet` 今天是私有的，要公开。

代价（`grep -rn 'CheckingEachDeviceBeforeTheAcquisition {' crates/ --include=*.rs` 去掉注释行数的：13 行、9 个文件，没逐处读）：core 自己 2 处（定义与 `crates/singlefs-core/src/mount.rs:7284` 构造），harness 的 `crates/singlefs-harness/src/history.rs:4712` 那一处名字映射与 `crates/singlefs-harness/src/model_comparison.rs:2067`，其余 9 行在 6 个用例文件里。第 5954 行起那段注释说「不另起错误成员」是为了不连着改这几处；加字段同样要改它们，只是不改成员表，穷举匹配写了 `{ .. }` 的不受影响。

两处报法上的限度，条文要写明：
1. 只报第一件不过的：闸按 1、五与三、四、二的次序判、第一件不过就返回（`rewrite_gate_not_met_by_a_device_without_any_self_verified_system_configuration`）。管理员按报的那一件修好再挂，可能撞上下一件。断言某一件的用例要让排在它前面的几件都过。
2. 只报第一块不过的盘：逐盘那一圈遇到第一块就整个返回。

### 崩溃流要罩住哪几个崩溃点

补写就是取号写的两轮（「for _round in ACQUISITION_WRITE_ROUNDS {」`crates/singlefs-core/src/transaction.rs:1272`），每轮逐盘写一次系统配置槽、之后一道屏障；两槽全废的那块盘第一轮落槽 1（世代号 1），第二轮落槽 0。不注入故障录下来的流里要罩的崩溃点：第一轮那一段里各盘写的每个子集（含撕裂），第一轮之后的屏障，第二轮那一段里各盘写的每个子集（含撕裂），第二轮之后的屏障；再往后是写行与暖机那几次发布里的系统配置轮换，补写的那块盘在那里第一次按「两槽都自证过」轮换，也该在同一条流里。

不在这条流里的：取号写任一写或屏障报错时的回卷（「return Err(pool.roll_back_acquisition(」`crates/singlefs-core/src/transaction.rs:1280`，与屏障那一处 `crates/singlefs-core/src/transaction.rs:1292`），这是补写闸第六件「补写失败按甲' 每盘回卷一次」。不注入故障录不到回卷那几写；C874 写的「录真实的取号两轮加补写进崩溃点重放，不手造写表」只罩不报错的那一支。要罩回卷，另录一条带写故障的流（第二块盘第一轮那一写 `WriteFails`，录下补写盘第一轮那一写与回卷那一写），在它上面同样逐态枚举。回卷之后补写盘上留的是旧代号、见证值 tail 的那一槽：它「复活」了，没再经补写闸；这一态观察者该判什么，条款没写（推的，没造）。

### 「只判整写不判撕裂」那行变异放在哪：真实流上杀不掉

观察者相对基镜像只记新出现的死盘。在不注入故障录下来的流上（推的，按代码推、没跑）：
- 有自证槽的盘，第一轮覆写较旧（或已坏）的那一槽，较新的那一槽还在，撕裂了也不死；第二轮覆写原来较新的那一槽时第一轮那一槽已经过屏障落盘，撕裂了也不死。
- 补写的那块盘在基镜像里本来就死，不算新死盘。

所以这条流上没有一个撕裂态造出新死盘，「只判整写」的观察者与正确的观察者在它上面逐态同判，变异是等价的、杀不掉。这行变异要另配一个必红的用例：给观察者喂一个合成的崩溃态（一块盘最后一个自证槽被撕裂写打坏），或一条把两轮之间的屏障去掉的核心变异录出来的流。这与 C874「不手造写表」字面冲突，条文要给观察者自己的必红用例开这个口子，或写明这行变异只在合成态上判。「不取基镜像」那一行不受影响：补写盘第一轮那一写落盘之前的几态它都会多报。

### R4 结论

推荐：错误成员加原因字段（含读错那一支），条文写明「第一件、第一块盘」；崩溃流两条（不注入故障的、第二块盘写报错带回卷的）；观察者的撕裂变异挂在合成态的必红用例上。会红的用例：每一件闸各造一个只有它不过的盘面，断言拒的成员里原因是它；读错那一支断言原因是读错。

## 推翻条件（补 R3、R4）

- K3-1：冻结之后调 `run_compaction`，交回的 discard 里一段都没有装着冻结那次发布落点的（在「抬 F 刚回收空的段被重发的改动取走」那一形上量），或恢复在冻结那次的根与它的单元之间另有一判把这条根判成不可择。
- R4 撕裂那一条：在不注入故障录下来的补写流上找到一个撕裂态，观察者按逐盘数判出一块新死盘。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| R3 | 零单元发布写根后失败、表不记 | 挂载里两处调用（写行、暖机），mkfs 进程的暖机 |
| R3 | 冻结后抬 F、回退、推环、收改动读旧类 | 四个入口的第一道判冻结；公开的准入上限函数（字面走得到，不写盘） |
| R3 | 整份换回分配器时盘上已有新根 | 抬 F 四处换回、回退一处、发布路径两处 |
| R3 | 重发换上的那份与盘不一致 | 冻结期间除 K3-1 外没有写盘的入口 |
| R3 | 挂载期间共用读把写坏的块当成写下的 | `DeviceReadOnceInThisMount` 的写与写零两支 |
| R1 | 乙在年轻环、几个槽同钉 0 上误红 | first_txg 为 1、区域 0 与 1 的槽 0 同时读坏，按代码推 |
| R2 | 第 3944 行在本进程准入路径上走到 | 现行根读坏、读得出两支；被抛弃判定；F 高于 txg |

全部是读码，没在任何装置上跑。没打中的不算「没打中」的观测（一次取样），照 `.claude/rules/three-way-inference.md` 那一条要再抽一次。

## 这条腿自己的限度

- 没写模型、没起 cargo、没造 harness 用例：K3-1 的第 2 步（重发的改动落进刚回收空的段）是不是在小池上真的发生，没量；discard 结局按三态哪一态落、恢复是否真择到冻结那次的根，按代码与条文推。
- R1 引的 (t, p) 两对数是实现员的线索，没复跑。
- R4 说撕裂态造不出新死盘，只按代码顺序推（两轮之间有屏障、每轮覆写较旧的那一槽），没跑崩溃枚举；`write_system_configuration_slot` 对两槽全废的盘落哪一槽，取的是 `transaction.rs` 第 592–594 行那段注释，没读它的实现。
- 改法（K3-1 的甲、乙，R2 的本进程断言，R4 的错误字段与第二条崩溃流）都只在读码上推过、被攻过零轮、没实现没跑。
- 没读正推腿的报告与 checker 那一簇的报告（禁读清单）。

## 没做什么

- 没跑重型测试、轻型三条、门禁阶段、cargo；没碰 GPU。
- 没建副本、编译目录与模型目录；草稿目录里只有两份文本（`cite1.txt`、`member-sites.txt`，命令输出）。
- 不判正推那几格，不替主 agent 采纳。
