# m3-close-b19-r3fix-code-r1 攻方第二样本（opus-s2）报告（2026-11-05）

立场：攻。攻击面是抬 F 那一遍三处按槽认盘、认槽（`RootsReadForRaisingTheFloor` 组装、`ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass`、`roots_on_the_devices_of_their_ring_slots`）、三条单测与三行变异，对照 checker 按槽认盘那几处。没读禁读清单里的任何文件。

引的代码行都是主工作区此刻的行。开工快照只存了 sha256，这次现量：`crates/singlefs-core/src/mount.rs` cb00d920…、`crates/singlefs-checker/src/walk.rs` 685475d0…、`crates/singlefs-core/src/root_ring.rs` 92402c9c…，三份都与 `research/prompts/m3-close-b19-r3fix-code-r1-snapshot/crates-sha256.txt` 里那一行逐字相同（量的：`sha256sum` 与 `cat` 那份清单比过）。

## 模型：复跑命令与文件

模型目录 `research/prompts/m3-close-b19-r3fix-code-r1-opus-s2-model/`，两份文件：

| 文件 | sha256 |
|---|---|
| `src/genesis_slot_attribution_equivalence_model.rs` | b66eb106271d84461ac88d7771d322c4e2fe672431ecf98382a049ba803a2096 |
| `tools/rerun.sh` | 64378f5f8171cb325bf9237ba7ebb3deababb5914f83c27aa7e5908c345f0c22 |

复跑：`bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>`，整条交统一任务服务（命令原样在「要主 agent 跑的命令」一节）。它把冻结副本的 `crates/`、`prune-server/`、`Cargo.toml`、`Cargo.lock`、`.cargo/`、`.claude/heavy-thread-default` 拷进 `<work>/tree`，核 `mount.rs` 的 sha256 是开工快照那一份（对不上退 3、不拼），把模型源码拼到副本 `mount.rs` 末尾（模型是 `mount` 的子模块，能调私有函数），在副本里跑 `cargo test -p singlefs-core --lib -- genesis_slot_attribution_equivalence_model --nocapture`，打出 `E1ESTIMATE`、`E1RESULT`、`E1SELFPROOF`、`E1EXAMPLE` 几行，跑完删 `<work>/tree`。模型目录里没有冻结输出（这条腿不跑），末行写 `result=no_reference_in_the_model_directory`。

模型是什么：抬 F 那一遍三处的「按槽」一侧调仓里的函数本身（`ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass`、`roots_on_the_devices_of_their_ring_slots`、`newest_valid_root_txg_on_every_device`、`allocator::oldest_txg_in_the_ring_rereading_slots_bad_at_mount`、`reclaim_floor`），「按 txg」一侧照补丁 `~/.local/share/singlefs/work/m3-close-b19-r3-impl-core/patch/crates.patch` 删掉的减号行重写（读出根的槽取 `target_for_publish(txg)`，根归盘取 `region_devices[target_for_publish(txg).region]`）。两侧喂同一遍读数，逐格比中间量（钉表、每块盘上最新有效根）与结局量（环里最旧有效根、回收门槛、卸载当场放开的释放代集合）。取样：区域归属 4 种（0/1/0、0/1/2、0/0/0、0/1/1）× 池里有没有一块不住区域的盘 × F_生效 {0, 5} × 三个槽 0 各 4 种读数（第 0 代根、下一圈有效根、下一圈被抛弃根、没读出根）× 三个槽 1 有没有有效根 × 四个槽钉不钉（三个槽 0 钉在 0、区域 0 槽 2 钉在 6）× 每个钉着的槽 4 到 5 种重读结局，全组合 17,694,720 格（推的：按循环界算，跑出来以 `E1RESULT` 的 `configurations=` 为准）。跑前写下的判据在源码末尾：结局量三格必须为 0；中间量两格必须大于 0（取样走到了两种认法不同的格）；两条自证（释放代从 0 起、第 0 代根当被抛弃）必须比出不同，证明比法分得出。

这个模型不造历史，不跑崩溃点枚举：它比的是抬 F 那一遍读完根环之后的三步，读数取的是这一遍可能读到的全部组合（含重读、读坏、全零三种），不问这些读数由哪段历史造出。读数「可达」与否由下文逐格读码论证。

## 各格判定一览

| 格 | 结论 | 依据的性质 |
|---|---|---|
| Q2.1 写者与 checker 在准入、卸载上结局不同 | 没打中 | 读码坐实三处与按 txg 结局恒等（下面 Q2.3 的 P-a、P-b、P-c）；checker 归盘也按槽；模型上待量 |
| Q2.2 补读那几槽带的槽与真住的槽不同 | 没打中（结局层） | 只有第 0 代根能带错槽（C743 落点核给第 0 代根留的口子）；带错时三处结局不变；被抛弃根那一半由回收那一步的表兜住；读码坐实，模型上待量 |
| Q2.3 「真写者上两种认法算出的数相同」的前提 | **字面打中一处，结局层没打中** | 实现员给的前提「暖机已在每块盘落了本实例的根」在被吞的根槽写之下不成立，`newest_valid_root_txg_on_every_device` 两种认法一个交 `Some(0)` 一个交 `None`（推的，读码）；放开的释放代集合仍相同，靠的是另两条前提（释放代不小于 1、第 0 代根永不被抛弃），不是暖机 |
| Q2.4 单测与变异 | 三行变异各有单测红（推的，读码）；三行都是结局层的等价变异；组装那一步（补读、首读推进去的槽）没有变异罩着 | 结局恒等，任何整池用例都杀不掉这三行 |
| Q1（辅）与 D16 已定项 1 逐句对 | 对得上 | 「正常卸载」那一行（`.claude/kb/decisions/16-发布语义.md:43`）、「环里最旧有效根」那一行（`.claude/kb/decisions/16-发布语义.md:36`） |
| Q3 mount.rs 还按 txg 推槽、推盘的地方 | 有，逐处判过都不该改 | 只差第 0 代根，那几处都碰不到第 0 代根或只碰将来的 txg |

## Q2.3　「真写者上两种认法算出的数相同」要靠的前提

先把两种认法能不同的地方缩到第 0 代根上（P-c），再分三处看。

**P-c：非第 0 代根读出来只会住在落点公式给的那一槽。** 抬 F 那一遍首读、重读都走 `read_root_ring_slot`，自证得过而不住在公式槽的根照读坏记：`crates/singlefs-core/src/recovery.rs:1867`：「if !root_sits_in_the_ring_slot_the_formula_gives(」。顶上用的那条根（两遍里只有一遍自证过的槽）是重建分配器那一遍经 `read_root_ring_slot_telling_never_written_apart` 读出、同样核过公式的（`crates/singlefs-core/src/mount.rs:2808` 一带）。公式只给第 0 代根开口子，三个区域的槽 0 都认：`crates/singlefs-core/src/root_ring.rs:188`：「if checkpoint_txg == crate::make_filesystem::GENESIS_CHECKPOINT_TXG {」。所以对每一条带着槽的根，「它带的槽」与「按 txg 推的槽」只在 txg 0 上可能不同（读码坐实）。P-c 满足。

**P-a：第 0 代根永远不被实例表抛弃。** 抛弃的判法是 txg 严格大于那一行的所选 txg：`crates/singlefs-core/src/recovery.rs:2230`：「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」。txg 0 大于不了任何 u64。P-a 满足（读码坐实）。

**P-b：没有一个落点的释放代是 0。** 释放代是释放它的那次发布的 txg；txg 0 是 mkfs 种第 0 代根那一次，它不释放任何东西（推的：没逐行核 mkfs 那一次的释放路径，只核了 `allocator.rs` 释放写的是这次发布的 txg）。回收谓词是释放代不超过门槛（`crates/singlefs-core/src/allocator.rs:2610` 调 `released_record_can_be_reclaimed`），门槛是 0 时一个都回收不到。

**P-d（实现员给的那一条）：暖机已在每块盘落了本实例的根。** 条款是 `.claude/kb/decisions/16-发布语义.md:232`：「推到本实例的根覆盖每块盘为止」。

三处逐个看：

1. **环里最旧有效根（第 5041 行那一处，钉表按槽滤）。** 两种认法的钉表只在「这一遍在某个槽 0 读出了第 0 代根」时不同。读出了第 0 代根，它就按 P-a 进有效根的 txg 集合，最小值是 0；而三个槽 0 的钉值也都是 0（`crates/singlefs-core/src/mount.rs:3001`：「crate::make_filesystem::GENESIS_CHECKPOINT_TXG」）。于是 `oldest_txg_in_the_ring_rereading_slots_bad_at_mount` 两边都交 `Some(0)`：有效根最小值 0 不小于钉值 0，会先重读那一槽（`crates/singlefs-core/src/allocator.rs:1791`：「if oldest_valid_root.is_some_and(|oldest| oldest < pinned_txg) {」），重读读出什么结局都还是 0。两边只差多不多重读一次（按槽认的一侧在「区域 0 槽 0 钉着、别的区域读出第 0 代根」时多重读区域 0 槽 0 一次；按 txg 认的一侧在「区域 1 或 2 槽 0 钉着、那一槽读出第 0 代根」时多重读那一槽一次）。只靠 P-a 与 P-c，不靠 P-d。
2. **回收门槛。** 取 `reclaim_floor(new_floor, 上一格)`（`crates/singlefs-core/src/mount.rs:5300`：「let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);」），上一格相同它就相同。
3. **卸载那一格的「每块盘上最新有效根」（第 5311 行那一处）。** 第 0 代根只给它所在那块盘添一个 0；一块盘上有别的有效根时 max 不变，没有时从 `None` 变 `Some(0)`。任何一块盘交 `None`，整个取小交 `None`（`crates/singlefs-core/src/mount.rs:4284`：「.collect::<Option<Vec<CheckpointTxg>>>()?」）。所以两种认法的结果要么相同，要么一个 `Some(0)`、一个 `None`。`Some(0)` 时当场放开到 `min(门槛, 0)`（`crates/singlefs-core/src/mount.rs:5324`：「floor_of_the_reclaim.min(newest_valid_root_on_every_device),」），按 P-b 一个都放不开；`None` 时这一截为空。两者放开的集合相同，剩下那一截两边都扣到生效。靠 P-b 与 P-c，不靠 P-d。

**结论：三处的结局（回收门槛、当场放开的释放代集合、扣到生效的那一截）在任何一遍读数上都与按 txg 认相同，靠的是 P-a、P-b、P-c，不是 P-d。** 中间量（钉表、`newest_valid_root_txg_on_every_device` 的返回值）能不同。

**P-d 不满足的反例历史（字面打中，推的，读码，没造盘）**：被吞的根槽写（故障注入的「说谎的盘」，`crates/singlefs-harness/src/fault_injection.rs:2636` 的 `SwallowedWriteByteRange`）。

1. mkfs 那个进程：第 0 代根写进三个区域的槽 0（区域归属 0/1/0）；暖机 txg 1 落区域 1 槽 0（盘 1），这一次根槽写 FUA 返回成功而被盘吞掉，那一槽仍是第 0 代根；txg 2 落区域 2 槽 0（盘 0）；新建文件 txg 3 落区域 0 槽 1（盘 0）。
2. 同一个进程接着正常卸载。抬 F 那一遍读根环：区域 0 槽 0 第 0 代根、区域 1 槽 0 第 0 代根（进程以为是 txg 1；它在「知道住着根的槽」里，读出来自证得过就不重读，不核读出的是不是自己写的那一条）、区域 2 槽 0 txg 2、区域 0 槽 1 txg 3。
3. 按槽：盘 0 最新 3，盘 1 最新 0，交 `Some(0)`。按 txg：四条都归盘 0，盘 1 一条都没有，交 `None`。中间量不同；当场放开的都是空集（P-b）。

这一段历史上 P-d 字面不成立：本实例写成（FUA 返回）的根覆盖了盘 1，盘上却没有。实现员那句话要是指 `newest_valid_root_txg_on_every_device` 的返回值，在这段历史上不成立；要是指放开的集合，成立，但理由该写 P-b，不是暖机。

打中之后四句：
- 分不分辨臂：不分辨。按槽与按 txg 两条臂在这一格上放开的集合相同，差的只是中间量。
- 被判的系统当时看不看得到：写者看不到那一槽的第 0 代根是被吞写留下的（字节自证得过、槽在第 0 代根的口子里）；但这一格的结局不依赖它看不看得到。
- 满足的是哪个分句：Q2.3 的「有一个不满足就给出反例历史」——不满足的是实现员写的前提 P-d；被打中的是「算出的数相同」的字面，不是结局。
- 跑前条款给的改法在这一格上还中不中：第三轮判决定的改法就是「三处改成按槽」，这一格两条臂结局相同，改与不改都不中。要改的只有实现员报告里那句理由（`~/.local/share/singlefs/work/m3-close-b19-r3-impl-core/result.md`「第 1 条的说明」，主 agent 在背景材料第二节转述）：把「结局相同」的依据换成 P-a、P-b。同一段历史顺带碰到 `newest_valid_root_txg_on_every_device` 文档注释里另一句（`crates/singlefs-core/src/mount.rs:4256` 一带，「卸载那一格滤不滤（被抛弃的根）结局相同」，理由也是暖机先在每块盘落了根）：暖机那一条根被吞、那块盘上留着更早实例的被抛弃根时，不滤会放开得更多，代码是滤的，所以只是注释的理由不成立、代码不错（推的，没造盘）。两处都是措辞改法，只在我读码上推过、被攻过零轮。

什么现象会推翻这一节：模型 `E1RESULT` 里 `oldest_differs`、`reclaim_floor_differs`、`immediate_reclaim_differs` 任一格大于 0；或者找到一处释放代为 0 的释放（P-b 破）；或者找到一条非第 0 代根绕过落点核进了抬 F 那一遍的读数（P-c 破）。

## Q2.1　写者（按槽认盘之后）与 checker 在准入或卸载上结局不同

**没打中。** 依据分两层（读码坐实，模型上待量）：

1. 三处按槽与按 txg 结局恒等（上一节）。所以「按槽认盘之后」写者在准入、卸载上的结局与改之前逐格相同；若有写者与 checker 的分歧，它在改之前就在，不是这一改造出来的。
2. checker 那一侧对第 0 代根也按槽归盘：读根环不核落点公式、每条根带它住的区域与槽（`crates/singlefs-checker/src/image.rs:1760`：「.and_then(|bytes| check_root_slot(&bytes, &geometry.filesystem_identifier).ok())」）；算 F_生效 时每块盘的最新有效根按区域取盘（`crates/singlefs-checker/src/walk.rs:21115`：「let device = geometry.region_devices[usize::try_from(*region).expect("区域号")];」）；I-7.9 按 journal 重建抬 F 那一刻的环时，槽 0 认作第 0 代根、归那一槽所在区域的盘（`crates/singlefs-checker/src/walk.rs:9065`：「if slot == RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION」，同一函数开头按区域取 `device`）；按 txg 推槽的那一处把 txg 0 排除在外（`crates/singlefs-checker/src/walk.rs:9421`：「&& record.checkpoint_txg > 0」）。两边对第 0 代根的口径一致。

checker 不直接判「卸载当场放开到哪儿」：它判的是放开之后被复用的单元有没有被候选根或丢盘之后择到的根引用（I-7.4、I-2.1 一族）。写者当场放开得只会比条款少（上一节第 3 条：中间量不同时两边都放开空集），checker 看到的复用不会因为这一改多出来。

试过的形状（都是读码推演，没造盘）：
- 环里只剩第 0 代根时卸载（mkfs 进程里新建文件之前就卸载、或暖机根被吞）：两边都放开空集。
- 区域 1 槽 0 挂载时读坏、这一遍读出第 0 代根，准入抬 F：两边回收门槛都是 `max(new_floor, 0)`。
- 区域 0 槽 0 挂载时读坏、这一遍又读坏，区域 1、2 槽 0 读出第 0 代根：两边门槛相同，按槽的一侧多重读区域 0 槽 0 一次。
- 区域归属不是 0/1/0（三块盘、单盘、0/1/1）：上一节的论证不依赖归属，模型扫了四种。
- 池里有一块不住任何区域的盘：`all_devices` 带上它时两种认法都交 `None`（那块盘一条根都没有），卸载当场一截都不放开。这与条款「有一块盘一条有效根都读不出时这一截为空」（「正常卸载」那一行，`.claude/kb/decisions/16-发布语义.md:43`）字面一致；第一版两块盘都住区域，走不到（推的，没核 mkfs 收不收第三块盘）。

什么现象会推翻：模型 `E1RESULT` 结局三格大于 0；或者造出一张镜像，checker 对第 0 代根按区域 0 归盘（找到一处 checker 把 txg 0 推到区域 0 而写者按槽的判定）。

## Q2.2　补读坏那几槽带的槽与它真正住的槽可能不同吗

**结局层没打中。** 逐种看 `readable_roots_for_raising_the_floor` 的组装（第 4962–5007 行）推进 `roots_with_ring_slots` 的每一条：

| 来路 | 带的槽 | 能不能与真住的槽不同 |
|---|---|---|
| 首读自证得过（`crates/singlefs-core/src/recovery.rs:2016` 那个函数的首读） | 读的那一槽 | 只有第 0 代根能：C743 落点核给第 0 代根三槽都开口子，错位读把同一块盘上另一个槽 0 的第 0 代根读回来时照收。第一版归属下区域 0、2 同在盘 0，读区域 2 槽 0 错位读回区域 0 槽 0 的字节，带的是区域 2 槽 0，真住的是区域 0 槽 0 |
| 知道住着根的槽首读坏、重读自证得过 | 读的那一槽 | 同上，只有第 0 代根 |
| 两遍里只有一遍自证过的槽，这一遍重读自证得过（第 4993–4995 行） | 读的那一槽 | 同上 |
| 同上，重读仍坏（读不出、读坏不是全零、读回全零三种都走 `Bad`），拿重建那一遍记着的根顶上：`crates/singlefs-core/src/mount.rs:4997`：「.push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),」 | 记着它的那一槽 | 记着的那条根是重建那一遍在这一槽读出、核过公式的；这个进程写掉那一槽就不在这一类里。只有重建那一遍本身错位读进了第 0 代根时才不同 |

带错槽时三处各算成什么：带错的只可能是第 0 代根（txg 0、永不被抛弃）。按 Q2.3 第 1、3 条，它让最旧有效根变成 0（只会压低门槛）、让一块盘多一个 0（只会让当场放开那一截变空）；结局都是少放，不是多放。错位读读回第 0 代根时，那一槽真住着的根（比如 txg 2 的有效根）这一遍漏读：它是有效根时，漏掉它只让门槛更低、那块盘的最新根取别的槽（更旧或相同），仍是少放。

会不会放开被引用的单元——查了最可能出事的一形：那一槽在根环表里记成被抛弃根（H1）。错位读读回第 0 代根之后，那一槽算「这一遍读出了根」，`crates/singlefs-core/src/mount.rs:5003`：「.filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))」 把它记着的落点从这一遍的影子账输入里拿掉，被抛弃根本身也不在 `roots` 里。但回收那一步另按分配器根环表里全部被抛弃根记着的落点补隔离，不看这一遍读到什么：`crates/singlefs-core/src/allocator.rs:2583`：「.map(RootRingOccupancy::placements_the_isolation_still_protects)」。所以那几个单元回收出来照样被隔离、发不出去（读码坐实到这一行；`isolate_reclaimed_slots_the_isolation_still_protects` 的本体没逐行读，推的）。这一形与按槽、按 txg 无关：改之前第 5003 行那一滤就在（补丁里没有这一行的改动）。

什么现象会推翻：造出一遍读数，带错槽的不是第 0 代根（要先绕过 `crates/singlefs-core/src/recovery.rs:1867` 那一核）；或者 H1 那一形里回收那一步不补隔离（根环表里那一项在抬 F 之前已被换掉）。

## Q2.4　三条单测与三行变异测到了什么

三行变异（`crates/mutations.tsv:6742`、`crates/mutations.tsv:6743`、`crates/mutations.tsv:6744`）改回按 txg 时三条单测（模块 `crates/singlefs-core/src/mount.rs:11162`）各红在哪（推的，按代码逐行推；这条腿没跑，背景材料转述副本上任务 6526 三行都抓到）：

| 变异 | 改了什么 | 单测一（`crates/singlefs-core/src/mount.rs:11206`） | 单测二（`crates/singlefs-core/src/mount.rs:11247`） | 单测三（`crates/singlefs-core/src/mount.rs:11276`） |
|---|---|---|---|---|
| 6742 按 txg 认盘 | 第 5096 行区域取 `txg % 3` | 红：三份第 0 代根与 txg 3 都归盘 0，盘 1 没有根，交 `None`，断言要 `Some(0)`（第 11236 行） | 不调这个函数，绿 | 不调，绿 |
| 6743 按 txg 认读出根的槽 | 第 5076 行槽取 `target_for_publish(txg, AT_MAKE_FILESYSTEM)` | 不调，绿 | 红：两份第 0 代根都认成区域 0 槽 0，区域 1 槽 0 留在钉里 | 红：区域 1、2 的第 0 代根认成区域 0 槽 0，把区域 0 槽 0 从钉里拿掉 |
| 6744 读出根的槽不从钉里滤掉 | 第 5078–5080 行滤子恒真 | 不调，绿 | 红：区域 1 槽 0 留在钉里 | 绿：期望本来就是照旧钉着 |

测到的是什么：三条都只在函数这一级比中间量（钉表、`newest_valid_root_txg_on_every_device` 的返回值），按 Q2.3 这三行变异在结局层是等价变异——任何整池用例、崩溃点重放都杀不掉它们，能杀的只有函数级单测。这不算漏测，但要知道：这三条单测一删，这一改就没有任何东西罩着；而罩不罩着也不改任何结局。单测一的场景（卸载那一遍盘 1 上只读出第 0 代根）在没有故障的写者上走不到（暖机在盘 1 落了本实例的根），要被吞的根槽写才走得到（Q2.3 的反例历史就是这一形的近亲）；它判出的 `Some(0)` 与变异的 `None` 放开的都是空集。

没被变异罩着的改法（推的）：
- 组装那一步推进 `roots_with_ring_slots` 的槽。补读那两处（`crates/singlefs-core/src/mount.rs:4994`、`crates/singlefs-core/src/mount.rs:4997`）与首读（`recovery.rs` 里那个函数，补丁之前就带槽）推的是「读的那一槽」；把它们换成 `target_for_publish(root.checkpoint_txg, …)`，三条单测都不经过 `readable_roots_for_raising_the_floor`，照绿。结局仍等价（同一个证明），所以这也是等价变异，补不补一行由主 agent 定。
- 变异 6743 的替换文用 `RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM`，不是池里读出的 S；补丁删掉的原文用的是 `parameters_of_the_pool.geometry.root_ring_slots_per_region`。三条单测只放第 0 代根，S 取多少都推到区域 0 槽 0，所以不影响判别力；但它不是逐字的「改回」。

## Q1（辅）　三处与 D16 已定项 1 逐句对

- 「环里最旧有效根」那一行（`.claude/kb/decisions/16-发布语义.md:36`）：抬 F 那一遍「根环表里挂载时读不出的那几项同样按这一句并进最旧有效根」——代码按槽滤掉这一遍在那一槽读出了根的钉（第 5041 行），其余并进、成了最小的先重读。对得上。按槽是字面读法：钉的是槽，「这一遍读出了那一槽」本来就按槽说。
- 「正常卸载」那一行（`.claude/kb/decisions/16-发布语义.md:43`）：当场可用的是「释放代不高于『每块盘上最新有效根的 txg』最小者的那一截」，代码 `min(回收门槛, 每块盘最新)`；「有一块盘一条有效根都读不出时这一截为空、整截扣到生效」，代码 `None` 那一支。对得上。「每块盘上」按根所在的槽归盘是字面读法；按 txg 推会把区域 1、2 槽 0 的第 0 代根算到盘 0 上，与「盘上」不合。
- 字面有张力的一处（与按槽、按 txg 无关，补丁之前就在，推的）：同一行末句「这一遍读不出、自证不过的槽当没有根」，而代码把「两遍里只有一遍自证过、这一遍重读仍坏」的槽顶上的那条根也算进每块盘上最新有效根（它就在 `roots_with_ring_slots` 里，第 4997 行推进去）。「根槽这一次读坏」那一行（`.claude/kb/decisions/16-发布语义.md:39`）给这类槽的处置写的是「抬 F 那一串（准入与卸载）重算回收门槛与影子账时」照记着的根算、「有效根照它本身进上限与候选集」，没写进卸载当场放开那一截的「每块盘上最新有效根」。两句怎么拼是条款没写清的一格。走到它要两类故障叠起来：那块盘上本实例写的根全被吞（否则本实例的根比顶上那一条新，顶上那一条当不了最新），且顶上那一槽这一遍仍读坏。最小形（没造盘）：归属 0/1/0；挂载时重建两遍里区域 1 某槽 X（盘 1）只有一遍读出 txg t1；本实例暖机落盘 1 的根被吞；正常卸载时 X 重读仍坏。代码按 t1 算盘 1 的最新根，按末句算是盘 1 上读得出的更旧的那一条 t0 < t1，释放代在 (t0, t1] 的那一截代码当场放开、末句不放。之后卸载半途崩、盘 0 掉、X 仍读不出时，恢复在盘 1 上择 t0，它引用的、释放代在 (t0, t1] 里的单元可能已被这一串的空发布写掉。这一格交主 agent 定是不是条款要补一句；不算这一轮的打中（射程是三处按槽，它不分辨按槽与按 txg）。

## Q3　mount.rs 里还按 txg 推槽、推盘的地方

有，逐处判过都不该改成按槽（读码坐实到行，结论推的）：只有第 0 代根两种认法不同，下面这几处要么碰不到 txg 0，要么推的是将来的 txg（将来的根只按公式写）。
- 抬 F 那一串规划这一串空发布落哪块盘、准入预算里数下一串要几次（`push_one_floor_raise_within_the_admission_budget`）（`crates/singlefs-core/src/mount.rs:4790`、`crates/singlefs-core/src/mount.rs:5187`）、暖机规划（`crates/singlefs-core/src/mount.rs:6219`）：都是将来要写的 txg。
- 水位之上没施加的记录、较新可读根的账、中间各次发布按 txg 归槽（`crates/singlefs-core/src/mount.rs:2033`、`crates/singlefs-core/src/mount.rs:2111`、`crates/singlefs-core/src/mount.rs:2251`、`crates/singlefs-core/src/mount.rs:2363`）：都是比所选那一版新的发布，txg 至少 1。
- 挂载时读坏的槽能住哪些 txg（`crates/singlefs-core/src/mount.rs:2953`、`crates/singlefs-core/src/mount.rs:2961`）：第 0 代根另由 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held` 给三个槽 0 都钉 0，已按槽处理。
- 落后支缺根那一判（`crates/singlefs-core/src/mount.rs:6982` 起，`crates/singlefs-core/src/mount.rs:6987`：「区域号是 txg 对区域数取模，小于 3」）：现行那一版是第 0 代根时只核区域 0 槽 0（盘 0），盘 1 上区域 1 槽 0 那一份不核。它前面先要「系统配置的 journal tail 落后于那一版末条记录的位置」才走到这一判，第 0 代根那一版没有 journal 末条记录可落后（推的，没核第 0 代那一版的 journal 位置怎么取）。
- `allocator.rs` 里按 txg 找根环表那一槽的两处（被抛弃根的落点、写行之后才判成被抛弃的根借 C 那一圈）不在 mount.rs，不在 Q3 射程；它们查的根都不是第 0 代根（第 0 代根永不被抛弃，P-a）。

## 没打中的形状

都是读码推演，没造盘、没跑；模型扫的是读数组合，不是历史。

- 第 0 代根在三个槽 0 的每一种「读出 / 下一圈有效根 / 下一圈被抛弃根 / 没读出」组合，配三个槽 0 与一个非 0 槽的钉、每个钉的五种重读结局（有效第 0 代根、有效下一圈根、被抛弃、全零、仍坏），四种区域归属，池里多一块不住区域的盘，F_生效 0 与 5：结局恒等（证明在 Q2.3，模型待量，约 1.77×10⁷ 格）。
- 错位读把同一块盘上另一个槽 0 的第 0 代根读回来（第一版归属下区域 0、2 同在盘 0）：带错的槽只影响中间量；那一槽是被抛弃根（H1）时影子账输入漏掉它，回收那一步按根环表补隔离兜住（Q2.2）。
- 被吞的根槽写让暖机落不到某块盘（Q2.3 的反例历史）：中间量不同，放开的集合相同。
- 两遍里只有一遍自证过的槽这一遍重读三种坏法（读不出、读坏不是全零、读回全零）：都走 `Bad`、都顶上记着的那条根，带的槽是记着它的那一槽，与真住的槽一致。
- 写者与 checker 对第 0 代根归盘：checker 读根环不核公式、按读出的槽归盘，I-7.9 重建抬 F 那一刻的环时槽 0 认作第 0 代根、归那一槽的盘；与写者一致（Q2.1）。
- 管理员回退能不能让本实例的暖机根变成被抛弃（那样盘 1 的最新根会退回第 0 代根）：不能，回退「不新增实例、不取号、不写实例表行」（`crates/singlefs-core/src/mount.rs:10319`）。

## 这条腿自己的限度

- 没造任何一段历史、没跑任何东西：全部结论是读码推的；模型源码这条腿没编过（照共用约束不交任务），有编译错的话由承接的腿改。
- 模型不枚举崩溃状态：它比的是抬 F 那一遍读完根环之后的三步，读数组合取的是超集（重读结局不受「那一槽真住着什么」约束），不说明哪些组合由合法历史造得出。
- 模型的「按 txg」一侧是照补丁减号行重写的，不是编进去的旧代码；与旧代码逐字不同的地方：用 `AT_MAKE_FILESYSTEM` 的 S（模型里的根都按这个 S 摆，两边一致）。
- P-b（没有释放代为 0 的释放）只核了释放写的是这次发布的 txg，没逐行核 mkfs 那一次。
- Q1 那一格字面张力的最小形要两类故障叠起来，没造、没量；它不在这一轮的射程里，只列给主 agent。
- 前三轮攻过的全零并、推空、准入那一处按槽认盘都没再攻；r3 判决第二节 Q4 第 3 问那一格的「算出来的数相同」这一轮按实现员的说法重新查了前提。

## 要主 agent 跑的命令

1. 模型复跑（harness 档，非重型；singlefs-core 的 lib 测试，模型由 rerun.sh 拼进副本）。在草稿目录起：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试里由 rerun.sh 拼进副本的 genesis_slot_attribution_equivalence_model 一条; 范围=1 个测试; 为什么=三方代码轮攻方第二样本的模型，比第 0 代根按槽与按 txg 认的三处中间量与结局量' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus-s2 -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-r3fix-code-r1-opus-s2-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus-s2/rerun-model-output
```

   - `--frozen` 写的是仓根：rerun.sh 先核副本里 `mount.rs` 的 sha256 是开工快照那一份（cb00d920…），对不上退 3、不拼。主 agent 手里有 `snapshot-tree.py` 倒推出的开工树的，`--frozen` 换成那棵树。
   - 判结果：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 `E1RESULT` 一行 `oldest_differs=0 reclaim_floor_differs=0 immediate_reclaim_differs=0`，且 `pinned_sets_differ`、`newest_on_every_device_differs` 都大于 0；两行 `E1SELFPROOF` 的数都大于 0；`test result: ok. 1 passed`。`exit=101` 是断言红或编不过，看 `<work>/model-output.txt`：断言红的话 `panicked at` 那一行点名哪一格，`E1EXAMPLE` 给例子，Q2.1–Q2.3 的「没打中」撤回。
   - 估时（推的）：编 singlefs-core 的 lib 测试几分钟；1.77×10⁷ 格在调试构建下估几分钟，`E1ESTIMATE` 一行打出扫完第一种区域归属之后按它估的全量秒数，超过 40 分钟就停、交回来缩取样。
   - 模型目录里没有冻结输出，末行是 `result=no_reference_in_the_model_directory`；判决要引的数以主 agent 这次复跑为准。

2. 交回之前的引文核对已跑（见下一节末行），不用另交。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；没往统一任务服务交任务；判决要引的数没复跑（命令在上一节）。
- 没写进仓里任何 `crates/` 文件；模型只在 rerun.sh 建的副本里拼进 `mount.rs`。
- 没读禁读清单里的文件；实现员报告 `result.md` 也没读，只引背景材料对它的转述。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-r3fix-code-r1-opus-s2` 里只建过两份拼 rerun.sh 用的文本（交回前删掉）；没建仓副本、没建编译目录。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-r3fix-code-r1-opus-s2-output.md` 末行（这一次跑出的原样）：

```
  ✓ 核了 19 处引文、30 处裸行号，对上 19 处，没判 30 处（其中裸行号没有引文可比 30 处；逐处列在上面）
```
