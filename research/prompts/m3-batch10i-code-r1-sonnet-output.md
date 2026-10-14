# m3-batch10i-code-r1 云端正推腿（Sonnet）报告：K1、K3、K4（2026-10-06）

立场：从条款与定案原文推到冻结副本里的行，再推到钉它的测试与变异行；能复跑的在草稿副本上复跑。引代码写冻结副本 `/tmp/claude-1000/m3-batch10i-code-r1/tree/` 里的行号（下面简写 `crates/…` 即该副本下的路径），引 kb 写 kb 文件自己的行号。方案按 `crates/` 今天的实现来谈。

## 0 前提与复跑环境

- 冻结副本五份被判文件的 sha256 在快照 `research/prompts/m3-batch10i-code-r1-snapshot/crates-sha256.txt` 里各命中 1 次（命令：对 `mount.rs`、`allocator.rs`、`mutations.tsv`、`rollback_by_a_forward_publish.rs`、`rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 各 `sha256sum` 后 `grep -c` 快照，五行输出都是 1）。D16 与 checks-owed 的 sha256 同快照 `kb-sha256.txt`（`16-发布语义.md` 为 b2cd52de…，同值）。
- 复跑只在草稿副本 `/tmp/claude-1000/m3-batch10i-code-r1-sonnet/work/`（冻结副本的 `crates/`、`Cargo.*`、`.cargo/`），自己的 target `/tmp/claude-1000/m3-batch10i-code-r1-sonnet/target/`；每条命令 `nice -n 19 bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/hosts/capped.sh 15 cargo test --offline …`。变异一律用模型目录里的 `mutrun.py`（在副本上套一行、跑整个测试二进制、还原；全部跑完后 `diff -rq` 副本 `crates/` 与冻结副本 `crates/` 无输出）。
- 包装结局：第一次起 rollback 那一条退 252（同会话已有一条在跑：攻方腿的 `cargo build --offline --release -p opus-i1-attack`，`ps` 看到的），那一次的输出不算结果；之后设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（派出去的两条腿各跑各的）照跑。另有五次退 251（四次是上限超过峰值表实测的 2 倍、一次是 `auto` 而表里没有这条命令的键），一次退 250（变异 2871 按 `auto` 的 918M 撞顶，改 3G 加理由重跑，结果用重跑那一次），退 250/251/252 的输出都没当结果用。
- 可复现物：`research/prompts/m3-batch10i-code-r1-sonnet-model/`（`mutrun.py`、`p/` 下每个自造变异的原文与替换文、`logs/` 下 37 份原始输出、`results.txt` 每份的 `test result` 行）。

## 1 K1　新类撑不撑得住

### 1.1 从条款与定案推出的「应该是什么」

| 编号 | 来源 | 应然 |
|---|---|---|
| E1 | S1 定案：`research/prompts/m3-batch10-code-r3-main-verification.md:90` 「两遍里有一遍读坏的自证根在根环表里另记一类，抬 F 重读它仍坏时照」（行内接「挂载那一刻读坏」钉住、不拒，那几个字带书名号，这里不进引号） | 这一类抬 F 那一遍读坏重读仍坏：不拒；回收门槛与准入上限照 `BadAtMount` 的两条钉法处置 |
| E2 | D16 「环里最旧有效根」行（`.claude/kb/decisions/16-发布语义.md:34`）「门槛按最小的算，根环表记最大的」 | 门槛取可能住过的最小 txg、上限取能住的最大 txg；这两个量对一个槽各是一个数 |
| E3 | D16 「抬 F 的上限」行（`.claude/kb/decisions/16-发布语义.md:36`）「当作可能是一条非空有效根，F 不越过它」 | 读坏并进上限的槽，F 不越过「它可能是的那条根」 |
| E4 | D23 已定项 14（`.claude/kb/decisions/23-journal的角色与格式.md:390`）「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」，同行「每次挂载与每次抬 F 按当时的候选集重算」 | 被抛弃的根还在环里时，它引用的单元在每一次抬 F 之后也仍被隔离 |
| E5 | D16 「根槽这一次读坏」行（`.claude/kb/decisions/16-发布语义.md:37`）「这个进程写掉那一槽、或进程内记根时重读读出有效根或被抛弃的根才解开」 | 这一句写的是 `BadAtMount` / `AllZeroAtMount` 两类；新类怎么解开条款没写（第 37 行今天没有 S1 这一类，见 `m3-batch10-code-r3-main-verification.md:90` 里「D16 已定项 1 第 37 行补这一类」） |

E1 是定案，E2、E3 管数，E4 管影子账，E5 管解开。E1 之外 kb 今天没有 S1 的字，E5 对新类属「规则没说」。

### 1.2 代码今天的样子（推到的行）

| 步 | 行 | 原文 |
|---|---|---|
| 新类只在整环重读那一支里算 | 「ring_slots_self_verified_on_only_one_of_the_two_passes =」（`crates/singlefs-core/src/mount.rs:2092`） | 没重读（第一遍每块盘都读出有效根）这个集合保持空 |
| 新类能住的 txg：与读坏的槽同一个函数 | 「fn txgs_ring_slots_bad_at_mount_can_hold(」（`crates/singlefs-core/src/mount.rs:1887`） | 窗口锚取并出来那份读数里最新的有效根（`window_anchor`），所以与 `BadAtMount` 同一个 N |
| 并进读出来那条根 | 「.min(root.checkpoint_txg),」（`crates/singlefs-core/src/mount.rs:2263`）、「.max(root.checkpoint_txg),」（`crates/singlefs-core/src/mount.rs:2266`） | 最旧取 min、最大取 max；公式给不出这个槽（年轻的环、第 0 代根）时 `map_or_else` 退到「|| TxgsARingSlotBadAtMountCanHold::new(root.checkpoint_txg, root.checkpoint_txg),」（`crates/singlefs-core/src/mount.rs:2258`） |
| 抬 F 算上限：新类读坏重读一次、仍坏不拒 | 「if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {」（`crates/singlefs-core/src/mount.rs:2520`） | 仍坏放进 `ring_slots_bad_on_this_read_taken_as_holding_no_root` |
| 上限并最大 txg | 「.chain(ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes)」（`crates/singlefs-core/src/mount.rs:2643`）、「.filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)」（`crates/singlefs-core/src/mount.rs:2724`）、「Some(highest_txg_it_can_hold) => ceiling_from_the_roots_read.min(highest_txg_it_can_hold),」（`crates/singlefs-core/src/mount.rs:2727`） | 上限 = min(读得出的根算出的上限, 仍坏的新类槽里最小的「能住的最大 txg」)，该 txg 不低于今天的 F 才算 |
| 回收门槛读根环：新类先从「读坏就拒」那一份里拿掉 | 「let ring_slots_whose_reread_still_bad_refuses_the_raise: BTreeSet<RootRingSlot> =」（`crates/singlefs-core/src/mount.rs:3338`） | 拿掉之后新类这一遍读坏当没有根，再在 3364 起的循环里自己重读一次 |
| 仍坏就钉最旧 txg | 「.insert(*ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held());」（`crates/singlefs-core/src/mount.rs:3382`） | 局部映射，只活在这一次抬 F 里；`oldest_valid_root_in_the_ring_for_raising_the_floor` 再并进去 |
| 影子账重算只看这一遍读出来的根 | 「let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {」（`crates/singlefs-core/src/mount.rs:3576`）下面第 3581 行「&roots,」 | 仍坏的新类槽不在 `roots` 里 |
| 重算怎么用 `roots` | 「let mut referenced_by_candidates: BTreeSet<(u32, u64)> = current_records」（`crates/singlefs-core/src/mount.rs:1493`）、「for root in roots.iter().filter(|root| is_abandoned(root)) {」（`crates/singlefs-core/src/mount.rs:1515`） | 被抛弃根的账只从 `roots` 里取；候选根的豁免也只给 `roots` 里的 |
| 隔离只加不撤 | 「/// 隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；」（`crates/singlefs-core/src/mount.rs:1476`） | 读不到的被抛弃根不会让已有的隔离消失，只是不新增 |
| 进程内写掉那一槽 | 「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」（`crates/singlefs-core/src/allocator.rs:2010`）、「) => self.clear_isolation_no_abandoned_root_in_the_ring_backs(」（`crates/singlefs-core/src/allocator.rs:2030`） | 新类被换下时按它带的落点清隔离（被抛弃那一支） |
| 进程内重读那一遍看哪些槽 | 「occupancy.ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held();」（`crates/singlefs-core/src/allocator.rs:2101`） | 只有 `BadAtMount` 与 `AllZeroAtMount`，不含新类（「| RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,」，`crates/singlefs-core/src/allocator.rs:1207`、`crates/singlefs-core/src/allocator.rs:1241` 两处都把新类排在外） |

### 1.3 逐问

**K1-a　回收门槛有没有一形越过盘上真住着、可回退到的有效根：一致（没推出这样的形）。**
- 新类的内容是第一遍或重读那一遍自证过的那条根 R（并出来的就是它，`better_reading_of_a_root_ring_slot_over_the_two_passes`）；这个进程之后写掉那一槽才会变，写掉就换占位（见 K1-f）。
- 抬 F 那一遍仍读坏时钉的是 `oldest_txg_it_can_have_held()`（`crates/singlefs-core/src/mount.rs:3382`），它是 `min(公式最旧 txg, R.txg)`（`crates/singlefs-core/src/mount.rs:2263`），所以对「R 有效」这一支恒 ≤ R.txg；对「R 被抛弃」那一支 `oldest_valid_root_in_the_ring` 不算它（`crates/singlefs-core/src/allocator.rs:1144` 起那个 match 把它归 `None`），钉着只会让门槛更低。
- 进程内（不抬 F 时）新类有效那一支取 R.txg 本身（`crates/singlefs-core/src/allocator.rs:1144` 同一处 `Some(*checkpoint_txg)`），不钉更旧的值：内容已知，这是对的。
- 能推翻它的现象：造出一个新类槽，盘上真住着的有效根比 `min(公式最旧, R.txg)` 更旧（要求那一槽的内容不是被读出的 R，而这个进程没写过它）；或抬 F 那一遍的门槛 `floor_of_the_reclaim`（`crates/singlefs-core/src/mount.rs:3598`）高过钉着的 txg。

**K1-b　被抛弃的根会不会掉出影子账：冲突（一形，推出来的，没造出反例）。**
- 条文：「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（`.claude/kb/decisions/23-journal的角色与格式.md:390`），同行「每次挂载与每次抬 F 按当时的候选集重算」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）。
- 代码：抬 F 那一遍重算影子账的输入是这一遍读出来的 `roots`（`&roots,`，`crates/singlefs-core/src/mount.rs:3581`），被抛弃根的账与候选根的豁免都只从它里面取（`crates/singlefs-core/src/mount.rs:1515`、`crates/singlefs-core/src/mount.rs:1493` 起的 `referenced_by_candidates`）。新类的被抛弃变体（`RootReadOnOneRebuildPass::AbandonedRoot`，带着 `referenced_placements`，`crates/singlefs-core/src/mount.rs:2301` 起）读坏、重读仍坏时不在 `roots` 里，占位上记着的落点没人用来重算。
- 并排：条文要求「每次抬 F 按当时的候选集重算」且被抛弃根离环之前不许重新分配；代码在这一形里抬 F 照样推过去（不拒），隔离只加不撤（`crates/singlefs-core/src/mount.rs:1476`），所以已有的隔离不丢，丢的是「抬 F 之后新变成只被被抛弃根引用」的那几个槽——一个槽此前靠候选集里一条 txg 低于新 F 的有效根豁免，新 F 一过那条根就不再是候选，该隔离而没隔离，随后回收、发出去。
- 这条路在改前被关着：`crates/singlefs-core/src/mount.rs:3557` 的注释「不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根；重建分配器两遍里只有一遍自证过的槽读坏也重读一次，仍坏不拒、」前一行写的就是 C554 乙报告 Q6 那一形（知道住着根的槽读坏按没有根走，被抛弃根引用的槽不隔离），改前靠「仍坏就拒」挡住。S1 把新类从「知道住着根」挪到「照挂载那一刻读坏」，被抛弃变体在这一处等于重开了 Q6。
- 与 S1 定案字面一致（「挂载那一刻读坏」那一类本来就当没有根），所以这是「定案字面」与 D23 第 390 行主句之间的冲突，不是实现员读错。新类的被抛弃变体与 `BadAtMount` 的区别是占位上记着它引用的落点（内容已知），用得上而没用。
- 可达条件（推的）：新类槽装的是被抛弃根 C；抬 F 那一遍读它两次都坏；C 与某条有效根 V 共享一个落点，V 的 txg 在旧 F 与新 F 之间，且该落点已不在现行账里。能推翻它的现象：造出这一形、抬 F 之后 `isolated_slots_per_device` 与没注读故障的对照相同，且 C 的落点在 C 离环之前没被改写。我没造：现有的 R1 用例（`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:1767` 起的 `while abandoned_root_still_in_the_ring(`）挂载之后换成不注故障的盘继续写，抬 F 读那一槽读得出，走不到这一形；S1 两条用例的新类槽是有效变体。
- 这条是线索，由主 agent 现查；造反例归攻方腿。

**K1-c　「能住的 txg」集合算得对不对：一致。**
- 与读坏的槽是同一个函数、同一个窗口锚（`crates/singlefs-core/src/mount.rs:1887`），那个函数这一批没改；新类集合与 `BadAtMount` 集合不相交（并出来的新类槽是自证过的，`bad_ring_slots_not_all_zero` 只从并出来仍读坏的槽取）。
- 公式给不出的槽（年轻的环里还没轮到的、第 0 代根住的非公式槽）退到 `(R.txg, R.txg)`；公式给得出的槽再 `min` / `max` 并上 R.txg（`crates/singlefs-core/src/mount.rs:2263`、`crates/singlefs-core/src/mount.rs:2266`）。
- 能推翻它的现象：一个新类槽的 `oldest` 高过 R.txg，或 `highest` 低于 R.txg。两个都由 `min` / `max` 排除；我用变异 P4（把整个集合换成 `(R.txg, R.txg)`）跑 31 条全绿，说明没有测试钉这个集合，见 K3-d。

**K1-d　并进 R.txg 之后有没有一形让 F 越过它：规则没说（有一形可构造，推的）。**
- 上限 = `min(读得出的根算出的上限, highest)`，`highest = max(窗口里落到这一槽的最大 txg, R.txg)`（`crates/singlefs-core/src/mount.rs:2266`、`crates/singlefs-core/src/mount.rs:2727`）。取 `max` 只会把封顶抬高，所以新类的封顶从不比「只取 R.txg」更紧。
- 一形：R 是留在洞里的旧根（txg 低于窗口下沿），窗口里同槽有一个更大的 txg h；抬 F 那一遍 R 读坏两次、读得出的有效根不足 4 条非空，上限落在读得出的最旧有效根（高于 R.txg）上；封顶取 h > R.txg，F 可以越过 R。条文 E3「F 不越过它」对已知 txg 的 R 没写封顶该取哪个，E2 的「根环表记最大的」是为内容未知的槽定的。
- 能推翻它的现象：这样一个盘面上抬 F 被封在 R.txg；或证明窗口里同槽有更大 txg 时 R 不可能被读出（我没证）。

**K1-e　卡住上限多于必要：一致（没推出多卡的形）。**
- 封顶只取 `max`，不会比 E3 更紧。上限偏低的另一个来源是仍坏的 R 不在 `valid` 里（`crates/singlefs-core/src/mount.rs:2646` 起，`readable` 只含读得出的），这不是封顶造成的：我把起点的覆盖写次数从 20 改成 21（`for overwrite_index in 0..20 {`，`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:495`），S1 持续读坏那条的「盘面与只在重读那一遍报读错的再挂逐字节相同」变 false（其余七项不变，`logs/mut-narrow21.log`）；再叠变异 P1（去掉上限并最大 txg）、P2（去掉回收门槛的钉）、P3（不记钉），仍同一项 false（`logs/mut-n21-p1.log`、`logs/mut-n21-p2.log`、`logs/mut-n21-p3.log`），所以差异来自「R 读不出、上限少算它」，与封顶无关。这是读不出就判不了的代价，不是条款外的卡。
- 能推翻它的现象：去掉封顶（P1）后某个起点的盘面与有封顶时不同。

**K1-f　它什么时候解开：写掉那一槽摆得出；重读读出有效根或被抛弃的根摆不出（没有占位转换）；条款没写。**
- 写掉：「.insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });」（`crates/singlefs-core/src/allocator.rs:2010`）换成 `ValidRoot`，被抛弃变体按「) => self.clear_isolation_no_abandoned_root_in_the_ring_backs(」（`crates/singlefs-core/src/allocator.rs:2030`）清隔离。路径在，但没有测试钉它（变异 A3：新类被抛弃变体被换下时不清隔离，31 条全绿，见 K3-d）。
- 进程内重读读出有效根或被抛弃的根：进程内记根那一处只重读 `BadAtMount` 与 `AllZeroAtMount`（`crates/singlefs-core/src/allocator.rs:2101`），抬 F 那一遍的重读只读盘、不改占位（`oldest_valid_root_in_the_ring_for_raising_the_floor` 注释写「只读盘，不改分配器」）。所以新类读出来之后还是新类，占位一直留到这个进程写掉那一槽。钉本身只活在一次抬 F 里（局部映射，`crates/singlefs-core/src/mount.rs:3382`），下一次抬 F 读得出就自然不钉。
- 后果：新类留在 `ring_slots_known_to_hold_a_root` 里（`crates/singlefs-core/src/allocator.rs:1102`），管理员回退判候选与算 F_生效 读它仍坏照拒（K2，攻方腿的格），不因抬 F 那一遍读出来而变。条款 E5 只写了 `BadAtMount` 那两类的解开，对新类「规则没说」。
- 能推翻它的现象：找到进程内某条路径把新类占位换成别的成员（我 grep 过 `occupants.insert` 与 `occupants.remove`，只有 `record_root_written_by_this_process*` 两处）。

**K1-g　顺带核了：`on_device_modes.rs` 的 `raise_the_rollback_floor_to_its_ceiling` 不构成打中。** 它先用空的新类表调 `rollback_floor_ceiling`（`crates/singlefs-checker-tier/src/on_device_modes.rs:336` 起），新类槽读坏两次会在那里拒；但新类只由 `rebuilt_allocator` 产生，这个模式的分配器来自 mkfs 同一个进程，不会有新类。能推翻它的现象：有一条路径让 checker 档装置在重建过的分配器上调它。

## 2 K3　补强的用例

### K3-a　rollback 三条慢档每一步核方向：一致（罩住了中间每一步）

- 推：钉住的那一槽是 txg 28 的槽（「root_slot_position_of(28),」，`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:234`），环 24 槽，下一次写它是 txg 52；循环「while pool.output.root.checkpoint_txg.0 + 1 < txg_writing_the_pinned_slot {」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:239`）每写一次调一次只核方向的函数（调用在 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:246`），它要求「只许 I-3.1 红」且「allocated > walked,」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:201`）；最后一次覆盖写（txg 52）之后要求一条违例都没有（「"{step}：池级 checker 一条违例都没有：{violated:?}"」，`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:167`）。
- 复跑基线：`logs/base-rb.log` 末行 `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 51.95s`。
- 复跑变异（`crates/mutations.tsv:2927` 那一行，加我自造的 32 / 51 / 52 三个同形变体，替换文只改 `checkpoint_txg.0 == 40` 里的数）：

| 变异 | 结果 | 红在哪一步（`logs/` 里 panic 行） |
|---|---|---|
| txg 40（表里的行） | `1 passed; 3 failed`（`refused_before_writing` 绿，其余三条红） | 「第 9 次覆盖写之后（txg 40），那一槽还没被写掉：I-3.1 红在记账多的一边」，已分配 Some(294912)、遍历 5963776（plain）；Some(344064) / 6012928（torn、zeroed）——与实现员报告抄的数相同 |
| txg 32（第一步） | 同上 1 / 3 | 第 1 次覆盖写之后（txg 32），三条各一次 |
| txg 51（钉住那一槽之前最后一步） | 同上 1 / 3 | 第 20 次覆盖写之后（txg 51），三条各一次 |
| txg 52（写掉那一槽那一次） | 同上 1 / 3 | 「写掉钉住的那一槽（txg 52）之后：池级 checker 一条违例都没有」 |

- 所以覆盖写 1 到 20（txg 32 到 51）与最后一次（txg 52）每一步都罩住「红在记账少的一边」；没罩的只有大小（设计如此，只核方向）。「overwrite > 0」的哨兵在 `crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:256`（「"{step}：挂载交回的那一版到钉住那一槽之间至少有一次覆盖写"」）。
- 能推翻它的现象：某个 txg（32 到 52）上把回收门槛改坏而这三条仍绿；我测了四个点，一个都没出现。

### K3-b　R1 那 16 条：一致（各自红在自己要罩的那件事上），变异表那 14 行偏钝

- 31 条是怎么数的：基线 `logs/base-rr.log` 的 `test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.86s`，`grep -c '^test .* ok$'` 数出 31。
- 变异表里 R1 两维那 12 行与 rr2 补的 2 行（`crates/mutations.tsv:2913` 到 `crates/mutations.tsv:2926`）用的是同一个替换文（只留最新那条根，与 10h 的 `crates/mutations.tsv:2872` 同文），我套 `crates/mutations.tsv:2917` 跑整个二进制：`0 passed; 31 failed`（与实现员报告的「31 条全红」一致）。整个二进制全红，所以「点名的那条红了」对每一行都自动成立，分不出哪条测试守哪件事。
- 真正分得出的是 10h 留下的几行，我各套一遍，看这 16 条里谁红：

| 变异（行号） | 改回的样子 | 红的条数 | 16 条里红的 |
|---|---|---|---|
| `crates/mutations.tsv:2864` | 整份换成重读的读数（rr2 臂） | 14 | 「只坏重读」12 条（3 种读坏 × 4 组：384 槽 / 4 GiB × 覆盖写 1 / 3 次）；反方向 4 条绿 |
| `crates/mutations.tsv:2866` | 第一遍自证压不过重读读坏 | 14 | 同上那 12 条；反方向 4 条绿 |
| `crates/mutations.tsv:2868` | 第一遍读坏压过重读自证过的根 | 16 | 反方向 4 条（4 组）；「只坏重读」12 条绿 |
| `crates/mutations.tsv:2871` 的文（我把参数换成整个 harness 二进制，`logs/mut-r1-2871h.log`） | 逐槽并时只取第一遍 | 16 | 反方向 4 条；「只坏重读」12 条绿 |

- 所以 16 条一分为二：12 条守「重读读数不能整份替换第一遍」，4 条守「并的时候第一遍读坏不能压过重读读出的根」；每一条在改回对应旧样子时红、在另一种旧样子下绿，都不空转。10h 之前的另两个旧样子（只读第一遍、只读重读）各自被其中一类罩住。
- 缺口：10i 这 14 行没有一行对应上表这种分得开的变异；点名的测试在门禁里「必须红」只验到「整个二进制全红」。新增那 12 条的守法靠我这次复跑，表里没有对应行。
- 能推翻它的现象：把 2864 或 2868 的变异套在新增的某一组（如 4 GiB、3 次）上而它不红。我套的是整个二进制，新增 12 条都在红的名单里（`logs/mut-r1-2864.log`、`logs/mut-r1-2868.log`）。

### K3-c　丢盘整读新起点（全零与乱字节两条）：一致

- 两条是 `on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before` 与 `…device_zero_reading_back_garbage_in_the_rebuild_pass_…`。在 2871h（只取第一遍）与 2868 下红；在 2864、2866（只读重读，重读那一遍整环读得出）下绿，所以它们罩的是「整环重读发生」，不是 R1 的并法。基线绿。
- 能推翻它的现象：删掉整环重读（`crates/mutations.tsv:2646` 那一行「第10e批 重建分配器读根环那一遍一条有效根都没读出时不重读、当场拒」的文）后这两条仍绿；该行的点名测试在另一个二进制里，我没套。

### K3-d　S1 两条与新类里没人钉的几处：打中（缺口，不是用例红错地方）

| 变异 | 改什么 | 结果 |
|---|---|---|
| `crates/mutations.tsv:2909`、`2910`、`2911`、`2912`（表里 4 行） | S1 四处 | 各自 30 过 1 红（2909、2911、2912：持续读坏那条）、29 过 2 红（2910：两条都红），与实现员报告一致 |
| P1（`p/p1.*`） | 去掉上限并新类最大 txg 那一步（`crates/singlefs-core/src/mount.rs:2643` 那一行 `.chain(…)`） | 31 条全绿 |
| P2（`p/p2.*`） | 去掉回收门槛里并上仍坏新类槽的钉（`crates/singlefs-core/src/mount.rs:3431` 起的 `.chain(…)`） | 31 条全绿 |
| P3（`p/p3.*`） | 不往仍坏映射里记钉（`crates/singlefs-core/src/mount.rs:3382` 那一句换成空） | 31 条全绿 |
| P4（`p/p4.*`） | 新类「能住的 txg」换成 `(R.txg, R.txg)`（`crates/singlefs-core/src/mount.rs:2260` 起） | 31 条全绿 |
| A2（`p/a2.*`） | 新类被抛弃变体不算进环里还在的被抛弃根引用的落点那一份（`crates/singlefs-core/src/allocator.rs:1171` 起） | 31 条全绿，core 库 195 条全绿（`logs/mut-alloc-a2lib.log`） |
| A3（`p/a3.*`） | 新类被抛弃变体被换下时不清隔离（`crates/singlefs-core/src/allocator.rs:2018` 起那个 match） | 31 条全绿 |

- S1 的两条用例都只造「新类里装的是有效根」，只验「不拒」这半条；钉最旧 txg、并最大 txg、被抛弃变体在抬 F 那一遍的处置、离环清隔离、算进还被引用的落点，没有一条用例钉，变异表也没有行。实现员报告里「证红」表没声称罩住它们。E2、E3、E4 对应的行为在 S1 这批里是「写了、没钉」。
- 能推翻它的现象：别的二进制里有一条测试在 P1 到 P4、A2、A3 任一条下红（我只套了点名的 `rebuilding_the_allocator…one_device_reads_no_valid_root` 二进制与 core 库）。我没跑全量 harness（不归子 agent）。

### K3-e　S1 瞬时那条的读序号 16：数对，但脆

- 常量「const READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE: u64 = 16;」（`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:1228`）。我把它改成 15 或 17 再套 2910：S1 瞬时那条不红（`30 passed; 1 failed`，只剩持续读坏那条红，`logs/mut-const15-2910.log`、`logs/mut-const17-2910.log`），只有 16 能让 2910 杀掉它。所以 16 就是抬 F 第一次算上限读那一槽的序号，数是对的。
- 脆在：读序一变，这条用例照样绿（读坏的是别的一次读），只有 2910 那一行在门禁复跑时报出来。能推翻它的现象：15 或 17 下 2910 仍杀掉瞬时那条（没出现）。

## 3 K4　数能不能核

每一格写：数从哪段代码推出、复跑看到什么、钉它的变异行改回旧值时红不红。

| 数 | 从哪推出 | 复跑 | 判 |
|---|---|---|---|
| `narrow-384-20-2` 起点 | 配方抄自攻方原型：「"narrow-384-20-2" => narrow_start(384, 20, 2),」（`research/prompts/m3-batch10-code-r3-opus-model/main.rs:350`），`narrow_start` 的形参是「fn narrow_start(slots: u64, overwrites: usize, units: usize) -> StartPoint {」（`research/prompts/m3-batch10-code-r3-opus-model/main.rs:329`）；用例里 384 槽与 20 次覆盖写、顺序写到 2 个单元分别在「for overwrite_index in 0..20 {」（`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:495`）与「pool.sequential_write(data_unit_payload_capacity() + 100)」（`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:500`），起点函数是「pub fn start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(」（`crates/singlefs-harness/tests/common_admission/mod.rs:85`） | 三个数与原型逐个相同。基线 31 条绿，S1 持续读坏那条断言挂载写行、暖机之后推了抬 F（「floor_raises_pushed_by_the_mount: true,」，`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:1431`）。我把 20 改成 10、19：31 条全绿（`logs/mut-narrow10.log`、`logs/mut-narrow19.log`），并且在这两个起点上套 2909 仍杀掉 S1 持续读坏那条（`logs/mut-n10-2909.log`、`logs/mut-n19-2909.log`）；改成 21：S1 持续读坏那条的「盘面与只在重读那一遍报读错的再挂逐字节相同」变 false，其余七项不变（`logs/mut-narrow21.log`） | 数能核。「20」不是下界，10 到 20 都推得出抬 F；它是我测的 10、19、20、21 里能让「盘面相同」那一项成立的最大值，贴着断崖（21 起不成立，原因见 K1-e：读不出的 R 少算上限）。用例注释里「抬 F 抬到的上限不受那一槽封顶」是这个起点的性质，不是代码的性质 |
| `(5, 3)` 对 `(4, 2)` | 盘 0 那份实例表读次数 = 重建分配器之前 2（「const DEVICE_ZERO_READS_OF_THE_NEWEST_INSTANCE_TABLE_BEFORE_THE_ALLOCATOR_IS_REBUILT: u64 = 2;」，`crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:593`）+ 影子账第一遍与重读各 1（`crates/singlefs-core/src/mount.rs:1980` 起的 `instance_table_of_the_newest_root_read_at_most_twice`）= 4；盘 1 那份只在盘 0 读坏时读，2 次；断言写在「DEVICE_ZERO_READS_OF_THE_NEWEST_INSTANCE_TABLE_BEFORE_THE_ALLOCATOR_IS_REBUILT + 2,」（`crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:636`）。生效 F 不另读（`crates/singlefs-core/src/mount.rs:1985` 的 `effective_rollback_floor_under_the_newest_roots_table` 用现成的表）。改回旧值（变异行在 `crates/mutations.tsv:1314`，现在写 `crate::recovery::effective_rollback_floor`）多读一遍：盘 0 + 1、盘 1 + 1 = (5, 3) | 套 1314：`7 passed; 1 failed`，只有 `a_newest_instance_table_unreadable_when_the_shadow_ledger_reads_it_is_reread_once_then_refuses_writable_instead_of_turning_the_isolation_off` 红，输出 `left: (5, 3)`、`right: (4, 2)`（`logs/mut-a3b-1314.log` 第 18、19 行）；替换文这次编得过 | 数能核，行有效 |
| R1 31 条 14 秒 | 31 = 原 15 + S1 2 + R1 两维 12 + rr2 补 2（本节 K3 与实现员报告；`logs/base-rr.log` 数出 31 条 ok） | 末行 `test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.86s`（实现员 14.06 s，机器不同）。整份 14.86 s 里并行跑，任何单条不超过它，「单条都不到 60 秒」成立 | 数能核；时长依机器，只作量级 |
| txg 40 那一次证红 | 钉住的槽是 txg 28 的槽，环 24 槽，下一次写它是 txg 52；挂载交回的版本 txg 31，覆盖写 20 次得 txg 32 到 51；表里 40 是其中第 9 次（输出「第 9 次覆盖写之后（txg 40）」） | 套 2927：三条红、`refused_before_writing` 绿，读数与报告相同；另测 32、51、52 都红（K3-a） | 数能核；40 不是特殊点 |
| 16（读序号） | 推不出来（读次序由挂载里读根环的各步决定）；实现员是打调用栈现量的 | 15、17 下 2910 杀不掉瞬时那条，只有 16 杀得掉（K3-e） | 数对，但是现量的，随读序变 |
| 追加 21 行、替换 7 行 | `crates/mutations.tsv` 里以「第10i批」起头的行 `grep -c '^第10i批'` 数出 21，分布 2909 到 2912（S1，4）、2913 到 2924（R1 两维，12）、2925 到 2926（rr2 补，2）、2927 到 2929（慢档，3） | 与报告相符；替换 7 行我没逐行核 | 追加数能核 |
| core 库 195 条 | — | `logs/base-corelib.log`：`test result: ok. 195 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` | 能核 |
| S1 两条的证红「只这一条」与「同一次一起红的」 | — | 2909：只持续读坏那条；2910：两条；2911、2912：只持续读坏那条（K3-d） | 能核 |

能推翻「数能核」的现象：在同一份冻结副本上按同一条命令复跑得出不同的数。我复跑了上表每个能跑的数，没出现。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-a 回收门槛越过真有效根 | 一致 | 新类钉的是 `min(公式最旧, R.txg)`，有效变体恒不高于 R.txg；没推出越过的形 |
| K1-b 被抛弃的根掉出影子账 | 冲突（推的，没造反例） | 抬 F 重算只用这一遍读出的根，被抛弃变体读坏两次不在其内，条文 D23 第 390 行要求每次抬 F 重算且离环前不许重新分配；改前靠「仍坏就拒」挡的 C554 乙 Q6 在这一形上重开 |
| K1-c 「能住的 txg」集合 | 一致 | 同一个函数、同一个窗口锚，与 `BadAtMount` 不相交；公式给不出时退到 R.txg |
| K1-d 并进 R.txg 后 F 越过它 | 规则没说（有一形可构造，推的） | 封顶取 `max(窗口最大, R.txg)` 只会比「只取 R.txg」松；R 是洞里旧根且读得出的非空根不足 4 条时 F 可越过 R |
| K1-e 卡住上限多于必要 | 一致 | 封顶不比 E3 更紧；上限偏低来自读不出的 R 不在有效根里，与封顶无关（P1、P2、P3 在 21 次覆盖写的起点上不改结果） |
| K1-f 解开路径 | 规则没说 | 写掉那一槽摆得出；重读读出有效根或被抛弃根没有占位转换，占位留到写掉为止，钉只活在一次抬 F 里 |
| K3-a rollback 三条每步核方向 | 一致 | txg 32、40、51、52 四个点各套一遍，三条都在那一步红 |
| K3-b R1 16 条 | 一致，表偏钝 | 10h 的 2864、2866、2868、2871 四种旧样子下 16 条各红在对应一类；10i 那 14 行用的变异让整个二进制 31 / 31 全红，分不出归属 |
| K3-c 丢盘整读新起点两条 | 一致 | 只取第一遍时红，只读重读时绿，罩的是整环重读发生 |
| K3-d S1 两条与新类里没人钉的几处 | 打中（缺口） | 钉最旧 txg、并最大 txg、被抛弃变体离环清隔离与算进被引用落点，P1 到 P4、A2、A3 六个变异都 31 条全绿，表里没有行 |
| K3-e 读序号 16 | 一致，脆 | 15、17 下瞬时那条不再被 2910 杀掉 |
| K4 各数 | 一致，一处贴断崖 | 起点数 20 贴着「盘面相同」成立的上界；其余数逐个复跑对上 |

## 没做什么

- 没造 K1-b 的反例（要同时造「被抛弃根与一条低于新 F 的有效根共享落点、抬 F 读那一槽两次都坏」的盘面与能保留读故障的设备，现有 R1 用例挂载之后换成无故障的盘），交攻方腿与主 agent 现查。
- 没判 K2（回退判候选与算 F_生效 那两处），不归这条腿；K1-f 末段只写了它与新类占位不转换的关系。
- 没跑全量 `cargo test -p singlefs-harness`、checker 档、门禁、变异整表（不归子 agent）；变异只套了点名的二进制与 core 库，别的二进制里是否有测试在 P1 到 P4、A2、A3 下红，核不了。
- 变异表另外 7 行替换（第 2299 行等）没逐行复跑。
- 没在入库装置上重做数：本报告的变异与读序数是草稿副本上的复跑，按 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」那一节，进判决前由主 agent 在入库装置上重做；模型目录里的 `mutrun.py` 与 `p/` 可拷去重跑。
- 包装在每次跑的时候会往 `~/.local/share/singlefs/memory-peaks.tsv` 记峰值（它自己的行为），我没动那份文件。
- 草稿目录 `/tmp/claude-1000/m3-batch10i-code-r1-sonnet/` 里的 `work/`（冻结副本拷贝，19M）与 `target/`（编译目录，26G）：交回前已删（`rm -rf`，之后 `ls` 只剩日志与 `model/`、`progress.md`），不再有；`mut-*.log` 已拷进模型目录 `logs/`，草稿目录里留着原件。
