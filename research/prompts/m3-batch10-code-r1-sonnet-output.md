# 根环实现第 10 批：代码轮第一轮，云端正推腿（Sonnet）报告（2026-10-05）

分到的格：K1、K3、K5、K6、K7。立场：从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行。冻结副本 `/tmp/claude-1000/m3-batch10-code-r1/tree/`，下文代码引文的行号都是那份副本里的行号（`cite-check.py` 用 `--root` 指它判）；kb 引文的行号是仓里 kb 文件自己的行号。

## 0　这一轮没做的事（先说清，免得读成「验过」）

- 没编译、没跑任何测试与变异（定义没写要编；这几格又多半要 harness 或 checker 档）。凡写「推的」都是读码推的；凡写「量过」只指我这一轮真跑的命令（只有 grep、算术脚本、sha256）。
- 实现员报告在 `/tmp` 下他们各自的目录里（不在仓里），我只当线索读，下文引到时只写「某批报告第几节」，不当依据。
- 用户 2026-10-04、2026-10-05 的定案原话在调度记录 `records/2026-09-30-里程碑三开工第一轮.md` 第 611、624 行；条文还没写回 kb 的几处下文逐处标「kb 没写回」。

## K1　回收门槛（A、B）

### K1-1　六处是不是都退回「环里最旧有效根」

条款原文（D16 已定项 1「环里最旧有效根」那一行）：「六处都不再按一圈窗口滤根，窗口之外的有效根」（`.claude/kb/decisions/16-发布语义.md:34`）；同一行末尾另写：「一圈窗口这个量现在只剩这一处在用，给读不出的槽划出它能住的 txg 的范围」（`.claude/kb/decisions/16-发布语义.md:34`）。

逐处推到代码（都是冻结副本）：

| 处 | 代码 | 判定 |
|---|---|---|
| ① 分配器回收门槛，进程内 | `RootRingOccupancy::oldest_valid_root_in_the_ring` 对表里全部有效根与两类钉着的槽取 `.min()`，不滤窗口（`crates/singlefs-core/src/allocator.rs:1084`，调用点 `crates/singlefs-core/src/allocator.rs:1933`） | 一致 |
| ① 挂载时 | 「let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_ring);」（`crates/singlefs-core/src/mount.rs:2030`），`m` 取不到那一支「None => oldest_valid_root_in_the_ring,」（`crates/singlefs-core/src/mount.rs:2041`） | 一致 |
| ① 抬 F 那一遍 | 「let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_ring);」（`crates/singlefs-core/src/mount.rs:3212`） | 一致 |
| ② checker 候选过滤 | 「let walked = *index == newest_index || (!abandoned && !below_floor);」（`crates/singlefs-checker/src/walk.rs:8997`），没有窗口项 | 一致 |
| ③ checker I-3.1 丁-defer 门槛 | 「oldest_valid_root_in_the_ring.max(effective_rollback_floor)」（`crates/singlefs-checker/src/walk.rs:5837`），对读得出的有效根取 `.min()` | 一致 |
| ④ 影子账候选过滤 | 「.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)」（`crates/singlefs-core/src/mount.rs:1489`） | 一致 |
| ⑤ 回退候选集 | `roll_back_by_a_forward_publish` 的四个排除成员里没有窗口那一格（`crates/singlefs-core/src/mount.rs:6049`、`:6054`、`:6059`）；`OutsideTheOneRevolutionWindow` 全仓 grep 只剩一条用例注释 | 一致 |
| ⑥ 抬 F 的上限 | 取读得出的有效根「.filter(|root| root.checkpoint_txg >= current_floor)」（`crates/singlefs-core/src/mount.rs:2400`），不滤窗口 | 一致 |

**窗口今天还在用的地方（grep 数出来的）**：

- 池级 checker 的 I-7.9：「&& window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)」（`crates/singlefs-checker/src/walk.rs:5402`，准入支算上限的「之前的根」）、「&& window_anchored_at_the_newest_root_of_the_image.contains(*txg)」（`crates/singlefs-checker/src/walk.rs:5544`，带卸载记号支）。
- GPU 扫描内核的 I-7.9 同口径：「&& in_the_one_revolution_window(txg, newest_txg_of_the_image, root_ring_slot_count)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1287`）、「|| !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1306`）。
- 挂载时给读坏的槽划「能住的最大 txg」的区间：「let lowest_txg_inside_the_window = window_anchor」（`crates/singlefs-core/src/mount.rs:1781`）。这一处就是 D16 说的「只剩这一处」。
- 死代码：`txg_is_inside_the_one_revolution_window`（`crates/singlefs-core/src/root_ring.rs:123`）全仓 grep 零调用方。
- 不算六处：D28 准入式子的 `ring_window_on_one_device`（`crates/singlefs-core/src/admission.rs:383`），它是准入公式里的环窗口开销，不是回收门槛。

**判定（K1-1）**：六处实现一致。**与 D16 第 34 行字面冲突一处**：条款说抬 F 的上限「含判非空那一遍」六处都不滤窗口、窗口量只剩给读坏的槽划范围；代码里 checker 与 GPU 的 I-7.9 算上限时仍按锚在镜像最新根上的窗口滤（上面四处）。代码注释自己写明理由（后来的镜像上重算上限，洞里留下的旧根会把上限算低、在合法镜像上误红，`crates/singlefs-checker/src/walk.rs:5480-5500` 一带），所以是 checker 侧重算特有的、有意的偏离；条款没写这一处。invariants.md 第 63 行还写两个窗口、第 143 行 I-3.1 还写「且落在一圈窗口里（txg 大于最新根的 txg 减 3S」（`.claude/kb/invariants.md:143`）：这两行是调度记录第 601 行点名的「范围外残留进下一份写回」（`records/2026-09-30-里程碑三开工第一轮.md:601`），不是无声丢的。
另有一处过时注释：「某块盘一条有效根都读不出（m 取不到）时当场只放到一圈窗口里最旧有效根」（`crates/singlefs-core/src/mount.rs:2020`），代码是「None => oldest_valid_root_in_the_ring,」（`crates/singlefs-core/src/mount.rs:2041`），行为已退回整环，只是注释没跟。
推翻它的现象：有一份读码证据显示六处里某处的过滤谓词里还含窗口量（我 grep 了 `OneRevolutionWindow`、`one_revolution`、`txg_is_inside`，上面是全部命中）。

### K1-2　钉在可能住过的最旧 txg 那三处：有没有一段合法历史越过一条盘上真住着、可回退到的有效根（不安全方向）

推导（读码）：

1. 钉的数取法：「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」（`crates/singlefs-core/src/mount.rs:1797`）从 0 起走、每个槽第一次碰到的 txg 就是它在 [0, first_txg) 里能住的最小 txg。区域内槽位只由 txg 定，所以这个数不高于这个槽上任何一条真实住过的根的 txg。钉着的槽因此只会让门槛更低，不会更高。
2. 三处都取这同一个数：挂载时 `rebuilt_allocator` 把两类槽的最旧 txg `chain` 进 `oldest_valid_root_in_the_ring`（`crates/singlefs-core/src/mount.rs:1999-2013`）；进程内 `RootRingOccupancy::oldest_valid_root_in_the_ring`（`crates/singlefs-core/src/allocator.rs:1084`）对两类槽取 `oldest_txg_it_can_have_held`；抬 F 那一遍并上 `ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held`（`crates/singlefs-core/src/mount.rs:3044`）。
3. 解开钉的只有三种事：这个进程写到那一槽（`insert` 盖掉，`crates/singlefs-core/src/allocator.rs:1929`）、E2 重读读出有效根或被抛弃的根（`crates/singlefs-core/src/allocator.rs:2005`、`:2013`）、抬 F 那一遍这一遍自己读出了根（`crates/singlefs-core/src/mount.rs:3047`）。三种里：被盖掉的槽里原来的根确已不在盘上；重读读出有效根是照读出来的 txg 算；读出被抛弃的根当没有根，与 checker 按最新根的实例表判抛弃同一个口径。
4. 没有被钉住也没有被记进表的槽：「年轻的环里还没轮到」的槽（一个 txg 都住不了），它不可能住根。

我没读出不安全形。**读到的几个旁证**：E2 的循环在「最小的有效根比钉着的最小 txg 还旧」时返回，不重读（`crates/singlefs-core/src/allocator.rs:1238`），这是节省读次数，不影响安全（钉着的槽只会更低）。

两处覆盖缺口（这一格里「改回窗口也不红」的地方）：

- ④ 影子账候选过滤与 ③ I-3.1 丁-defer 门槛，改回一圈窗口没有会红的用例，也没有变异行。`crates/mutations.tsv` 里 RGall 的改回窗口行覆盖了进程内门槛（`:2468`-`:2470`）、候选过滤（`crates/mutations.tsv:2474`）、回退候选集（`crates/mutations.tsv:2476`）、抬 F 的上限（`crates/mutations.tsv:2477`）、由记录施加那一版的第 ④ 条，没有一行点名 `isolate_slots_referenced_only_by_abandoned_roots` 的候选过滤或 `deferred_slots_reclaim_threshold`；第 10 批 core 报告第八节第 6 条自己写了「没有会红的用例」。调度记录里没找到这一条的去向（K7 记）。
- `mount.rs:2020` 那条过时注释说明没人拿 m 取不到那一支的回收放开范围做过独立断言之外的核对（该支由删除与并发那条线第 17 批的用例钉，`crates/mutations.tsv` 里「删除批第 17 批 甲」那一行）。

**判定（K1-2）**：读码没找到不安全形；两处改回窗口无变异覆盖。推翻它的现象：造出一段合法历史，在某个读坏或读回全零的槽里住着一条有效根 R，而这个进程在 R 所在槽被写掉之前回收了 R 引用的单元（那就是钉的数高于 R 的 txg，只可能是 `first_txg` 取错或槽公式 `target_for_publish` 与写者不一致）。

### K1-3　全零槽另算：上限不并、回收门槛钉住，两边是不是同一份读数、同一个分类

- 分类只在挂载那一刻做一次：读报错或读得出不是全零而自证不过 ⇒ 读坏，读回全零 ⇒ 全零，自证过 ⇒ 根（`read_root_ring_slot_telling_never_written_apart`，`crates/singlefs-core/src/mount.rs:1737`；根环表、回收门槛、窗口锚用的是同一个 `ring_slot_readings`，不是各读一遍）。
- 回收门槛三处都看两类：进程内 `oldest_valid_root_in_the_ring` 里「RootRingOccupant::AllZeroAtMount {」（`crates/singlefs-core/src/allocator.rs:1095`）；挂载时 `chain` 了两类（`crates/singlefs-core/src/mount.rs:2003-2013`）；抬 F 那一遍并 `ring_slots_pinned_since_the_mount…`，它把两类并成一张表（「let mut pinned = self.ring_slots_bad_at_mount_pinned_at_the_oldest_txg_they_can_have_held();」，`crates/singlefs-core/src/allocator.rs:1179`）。
- 上限只看读坏那一类：`ring_slots_bad_at_mount_by` 调的 `ring_slots_bad_at_mount` 里只有 `BadAtMount` 一个臂返回 `Some`（`crates/singlefs-core/src/allocator.rs:1129`），全零落进 `=> None`；调用点 `crates/singlefs-core/src/mount.rs:2852`、`:3133`。
- 重读不改类：E2 重读读回全零或仍读坏，循环 match 里那两个结局的臂是空的（`StillUnreadableOrNotSelfVerified | AllZero => {}`，`crates/singlefs-core/src/allocator.rs:2015-2016`），表里的项不动；10c 报告第七节第 1 条也这样自报。所以两边按「挂载那一刻读到的」分类，之后一直不变，直到写掉那一槽或重读读出有效根（读出被抛弃的根就 `remove`，`crates/singlefs-core/src/allocator.rs:2013`）。
- **D16 第 36 行与代码冲突一处（kb 没写回）**：条款原文「挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它」（`.claude/kb/decisions/16-发布语义.md:36`），而代码里读回全零（它在这套分类里属「自证不过」的一种）不并进上限、只钉回收门槛：见 `ring_slots_bad_at_mount_by` 的文档「读回全零的槽不在其内」（`crates/singlefs-core/src/mount.rs:2763`）。这是用户 2026-10-04 先定、2026-10-05 又定乙的结果（调度记录第 624 行），实现员 10c 报告第八节第 2 条已提「要写回」，今天 D16 还没写（第 601 行的残留清单里也没列这一格，只列了 D23 第 382 行、invariants 两行、C660/C705）。

**判定（K1-3）**：实现一致；条款写回缺一格（上限那一行与「根槽这一次读坏」那一行要写明全零另算）。推翻它的现象：造出一段历史，同一个槽在挂载那一刻读回全零、之后抬 F 那一遍里它一次进上限一次不进（我读出的结构里不会，因为两处都查同一张根环表）。

### K1-4　I-3.1 多记账：只红在记账多的一边？时长到那一槽被写掉为止？有没有红在记账少的一边

推导（读码，推的）：

1. 设分配器门槛 T_a = max(F, min(读得出的有效根 txg, 各钉着的槽的最旧 txg))，checker 门槛 T_c = max(F_生效, 盘上读得出的有效根最小 txg)。钉着的槽读出来的内容要么是有效根（它的 txg ≥ 钉的数）、要么是全零或坏、要么被抛弃（拿掉）。钉的数不高于槽里真实 txg（K1-2 第 1 点），所以 T_a ≤ T_c：分配器回收得更少，记账只会比 checker 遍历得到的多。
2. 反方向要 T_a > T_c：要有一条盘上有效根没被分配器看见又没被钉。两类来源——槽没进表（年轻的环里还没轮到的槽，不能住根）、表里记着别的数而盘上换了内容（这个进程写过的槽按 `ValidRoot` 记，`crates/singlefs-core/src/allocator.rs:1929`；失败的发布整个分配器退回，表一起退回）。读码没找到第三条。
3. 抬 F 之后（k2-zeros 没洞那一形）：F 越过 X = 17 抬到 18，X 那一槽被抬 F 那一串第一次空发布写掉；半途崩溃态里系统配置已带 18，checker 的 F_生效 取系统配置里读得出的 F，X 已不在候选集，门槛两边都按 max(18, …)。所以也不红在记账少的一边（推的，没造半途崩溃盘面；10b 报告第八节第 2 条自己也写「推的，没造」）。

钉它的测试：方向由 `allocated_statistic_counts_more_than_the_walk` 判，它解析说明里的「记账的已分配 Some(a)，遍历全部有效根得到 b」，a > b 才真（`crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs:207`）；用例断言每条红都必须是 I-3.1 且这一边：「"{when}（txg {txg}）：只许 I-3.1 红在记账多的一边：{violations:?}"」（同文件:286），写掉那一槽之后必须一条都不红：「"{when}（txg {txg}）：X 那一槽写掉了，不再钉"」（同文件:292）。前洞那一形断言红的版本集合恰好是 txg 20 一个：「"只有重读读回全零、照旧钉着的 txg 20 那一版红：{violations_of_each_version:?}"」（`crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs:636`）。
两处弱点：① 解析失败时 `matches!` 给 false、断言红，不会静默放过，但**这两条用例都不断言真的红过**（没红也过），所以「红一版」那个代价是不是真发生，靠的是前洞那一条断言恰好集合 = [20]，另一条（X = 21 洞里那一形）只禁方向、不要求红；② 「最长一圈」的上界没有用例：被钉的槽被写掉最迟隔 3S 个 txg，是槽公式推出的，要是那一槽的写一直失败（磁盘卡住那一形），时长无界，这正是第二节列的 E139 F10、F11（已知没做，不算打中）。

**判定（K1-4）**：只红在记账多的一边——两条腿的读码都对上、两个用例的断言方向对上；没找到记账少的一形。数 `Some(3145728)`、`1441792` 见 K6。推翻它的现象：任何一段合法历史上池级 checker 报「记账的已分配 Some(a)，遍历全部有效根得到 b」而 a < b。

## K3　I-7.9 与 I-8.5（C、F 的 I-7.9 一半）

### K3-1　I-7.9 两支只留锚在镜像最新根上的窗口：有没有一张坏镜像（F 越过上限）因此漏红

条款今天的字面（invariants.md 第 63 行，10 批前的写法）：「落在两个一圈窗口的交里的最新那条根的 txg」（`.claude/kb/invariants.md:63`）；代码只留锚在镜像最新根上的那一个（K1-1 列的四处）。**这是条款与代码的冲突**，调度记录第 611 行记了「主 agent 认它取的 I-7.9 读法」、第 601 行记了 invariants 第 63 行等写回，所以去向是有的；没有的是下面这个盲带。

推导（推的，没造盘面）：窗口按镜像最新根 N 取，锚只会比抬 F 那一刻的锚新，所以窗口只会去掉「环上有洞时洞里留下的旧根 X（txg ≤ N − 3S）」，去掉后 checker 算的上限 ≥ 分配器今天（取整环，`crates/singlefs-core/src/mount.rs:2400`）算的上限。差只在非空有效根不足 4 个、上限落到「最旧有效根」时（`rollback_floor_ceiling_from` 的 `.unwrap_or(oldest_valid_root_txg)`，`crates/singlefs-checker/src/walk.rs:5327`）：整环口径取 X，窗口口径取窗口里最旧的那条 W（> X）。于是一张镜像上 F ∈ (X, W]、其它条件相同：整环口径下违例，checker 看不见。这一带正是 RGall 要保护的东西（F 越过洞里留着、可回退到的有效根 X，X 独占的单元被回收）。检查是不是有别的网：F 越过 X 之后 X 低于 F_生效，不再是候选，I-3.1 的候选集与丁-defer 门槛（`crates/singlefs-checker/src/walk.rs:5837`）都按 max(F, …) 算，自洽、不红；I-7.4、I-4.8 只对候选根判。所以这一带只有实现一侧的用例与变异在盯（抬 F 的上限改回窗口：`crates/mutations.tsv:2477` 与 `:2497`，后者 10b 报告第四节写「没证」）。
钉 checker 这一侧的用例只有三条正向：窗口里红「`a_floor_raised_above_the_ceiling_of_the_roots_inside_the_window_still_reddens_the_rollback_floor_invariant`」（`crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs:407`）、洞里旧根不许把带记号支判红（同文件:553，断言在 :587）、窗口内仍红（同文件:595 一带）；没有一条钉「F 越过洞里旧根时 checker 不判」这一边，也就没有人知道这条边是有意的。
**判定（K3-1）**：条款与代码冲突（invariants 第 63 行）有写回去向；「F 越过洞里有效根 X 时 I-7.9 看不见」是代码注释自己说明的代价（`crates/singlefs-checker/src/walk.rs:5480-5500`），条款与检查欠账里都没写这一带。推翻它的现象：造一张镜像，环上有一个洞留着 X、窗口里非空有效根不足 4 个，F 抬到 (X, W] 之间，池级 checker 判 I-7.9 红（那就是我推错了窗口去掉的集合）。

### K3-2　`192:9:3:1` n = 8 `w:all/but1:z` 池级 checker 判 I-7.9 红 4116：实现越了上限，还是 checker 判错

读到的事实（grep 与读码，没复跑——复跑要编译、要起装置）：

- 那份拷贝（`m3b17-today/tree`）是 10b 之后、10c 之前的树：`allocator.rs` 里 `AllZeroAtMount` 零命中、`mount.rs` 里读回全零的结局叫 `AllZeroNeverWritten` 且两处都 `=> None`（`mount.rs` 1889、1897 行），注释写「从没写过、不并进上限、回收门槛不钉它」。`z` 臂是读回全零（零填充读），所以那一形是：盘上槽里住着完好的有效根、挂载那一刻读回全零，分配器当它「从没写过」，上限不并、门槛不钉。
- 今天的树里读回全零的槽在上限里**照旧不并**（K1-3 第三点），只是回收门槛钉住了：`ring_slots_bad_at_mount` 只返回 `BadAtMount`（`crates/singlefs-core/src/allocator.rs:1129`）。
- 池级 checker 判 I-7.9 的定义是：F 不高于「锚在镜像最新根上的窗口里、它之前的有效根」算出的上限。镜像上两盘都在，盘上那些有效根读得出，checker 把它们算进上限；分配器看不见（读回全零），F 越过它们，红是对 I-7.9 字面判对了的。「它自己的判法 0 坏」说的是攻方装置的冷读判法只看回退后读不读得出数据（F 越过候选根不等于数据读坏），两个判法的问句不同，不矛盾。

**判定（K3-2）**：是实现越了上限，不是 checker 判错；越法就是用户 2026-10-04 认下的 Q3 代价「读坏成全零的槽 F 可能越过住在里面的有效根」（调度记录第 624 行；第二节也列为已认的代价）。10c 把全零钉进回收门槛之后，那两段改成取号之前被拒：触发处是取号前整串预演里落点取不到（`crates/singlefs-core/src/mount.rs:4675` 起那个 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`，文档自己写「条款没定」，`crates/singlefs-core/src/mount.rs:228`）；为什么取不到——钉住之后回收得更少，窄盘上放不下写行与暖机——是我读码推的，没量。K2 归攻方腿，我不判该不该拒。
推翻它的现象：在今天的树上跑 `rerun.sh` 的 l0-5，若 I-7.9 红的状态里 F 没有越过任何一条盘上读得出的有效根的 txg，则是 checker 判错（我推的口径里不会有这种状态）。

### K3-3　I-8.5 改报不适用：边界对不对

条款（invariants.md 第 94 行，没跟这一批）：「自证过而不可择的槽（R / S 越界、incompat 位不认得）与带越界值的槽都算自证过」（`.claude/kb/invariants.md:94`）。代码把一条分成三格（`crates/singlefs-checker/src/walk.rs:8687` 起）：

| 盘面 | 判 | 代码 |
|---|---|---|
| 一槽都没自证过（`chosen` 空、没有 `SelfVerified*`） | 红 | `first_violation_of` 取到违例（`crates/singlefs-checker/src/walk.rs:8694`） |
| 有槽自证过、一块盘都择不出 | 不适用，带理由 | 「池里有系统配置槽自证过，而没有一块盘交得出可择的系统配置：这不是一个挂得上的镜像」（`crates/singlefs-checker/src/walk.rs:8706`），连同别的每条一起不适用 |
| 至少一块盘择得出 | 照常判（成立；单盘全废走旁路报告） | 同函数后段 |

边界推导：

1. 「自证过而不可择」是 `SelfVerifiedButNotChoosable`（incompat 位不认得，或 R / S 越界，`crates/singlefs-checker/src/image.rs:533` 上面的文档写明）；池级字段带读者不收的值的槽是另一类（`CarriesAValueThisReaderRefuses`），整池报不适用，在更早的一支就返回了（`any_system_configuration_slot_carries_a_refused_value` 那个 `if`，`crates/singlefs-checker/src/walk.rs:8649`），到不了第二格；所以第二格接住的是前一类。用例造的是 R = 4（`RegionCountPastTheRegionDeviceFields`）：它走的是哪一支我读出来是 `SelfVerifiedButNotChoosable`（用例名：「`self_verified_system_configuration_slots_no_reader_can_choose_leave_every_invariant_not_applicable`」，`crates/singlefs-harness/tests/checker_known_bad_images.rs:7839`），断言每一条都不适用；
2. 条款与代码的差：条款只写「算自证过」与两处整池不适用，没写「全池择不出时报不适用」。这第三种不适用是 10b 加的、`InvariantVerdict`「不适用必须带理由，不许静默算成立」撑着；旧用例原来钉「I-8.5 成立」，已改钉不适用。所以是**条款没跟（冲突）**，去向是写回批（调度记录第 601 行的残留清单没单列 I-8.5 这一格，只列了 invariants 第 63、143 行；10b 报告第七节第 5 条与调度记录第 624 行记了读法，主 agent 认）。
3. 一个边界风险（推的）：`chosen` 是「择得出系统配置」，别的池的旧盘（fsid 不同）槽也算自证过（`image.rs` 注释写「本池自证不过那一半这里不分」，`crates/singlefs-checker/src/image.rs:1180`），全池都是别的池的槽时这一条报不适用而不是红；条款没说，旧行为。

钉它的变异：`crates/mutations.tsv:2494`「第10b批 I-8.5 没有可择的系统配置时照旧判成成立」，10b 报告第四节列它为证过。
**判定（K3-3）**：边界对（红只给「一槽都没自证过」，这一格 `fault_injection_fast_tier` 钉着，10b 报告第七节第 5 条）；条款没写第三种不适用。推翻它的现象：造一张镜像，一块盘都择不出、有槽自证过，而 checker 报了 I-8.5 红或成立。

## K5　GPU 与 CPU 同口径（F）

### K5-1　树数上限 11 的守门用例能不能被一条写得更长的树表绕过

- 内核一侧：一张树表第 12 棵不同的树起「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (28u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:527`）：不是静默少记，是判不了、交 CPU；对拍断言一个状态都不许判不了：「一个状态都不许判不了」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:5`）。所以任何一条被对拍的流写出 12 棵不同的树，在有卡的机器上都红在对拍里。
- 守门用例是取卡之前的早红，只看两条流（新池新建文件、固定脚本到回退 D），读内核源码里的上限与这两条流真写下的树表单元条数比：函数名「`every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_count_the_gpu_scan_kernel_records`」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:3843`），断言文本「"流 {} 写下的树表最多 {largest} 条，GPU 扫描内核一张树表只记 {capacity} 棵树：先把 C_MAX_TREES_PER_TABLE 跟上（草稿区顺移、SCRATCH_WORDS 重算）",」（同文件:3867）。
- 绕过的几种形（推的，没造）：① 别的流（带坏镜像的那几十条）里手工改出更长的树表：守门不看，但对拍在有卡机器上红（上一条）；没卡的机器用卡的用例打 `NOT_RUN_NO_GPU_ADAPTER` 返回成功，那时守门是唯一的网，而它只罩这两条流。② 树表条目宽变了（不是 200）：`is_a_tree_table` 不认，`assert!(!entry_counts.is_empty())` 红，不会静默。③ 树表长成多层（根层级 > 1，每片叶各自 ≤ 11、合起来 > 11）：守门只数每一个叶单元的头里条数，不加总；内核只记一张表里的树（`r_note_tree_table_entry_birth_txg` 逐条记）。核心层今天的树表是单个码 2 节点（`crates/singlefs-core/src/recovery.rs` 里读树表是 `read_unit_via_locations` 读一个单元，`:4836` 一带），这一形走不到。④ 只改 `C_MAX_TREES_PER_TABLE` 不改每候选根的字数 `C_CANDIDATE_WORDS = 50`（= 6 + 4 × 11，「const C_CANDIDATE_WORDS: u32 = 50u;」，`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:378`）与它后面 27 个手写偏移：全仓没有一条「偏移不重叠」的断言（grep `overlap|不重叠|layout` 在 `crash_judge_gpu.rs` 只命中生成结论字号的几处），`scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels`（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:285`）只核 `SCRATCH_END` 与 `SCRATCH_WORDS` 相等。我用脚本把链上 23 处「起点 + 条数 × 字数 = 下一个起点」逐个算过（脚本与输出见下），今天全对；但要是有人把上限抬到 12 而漏改偏移，对拍只会在有一条流恰好读到被压的那一片时才红。

脚本输出（`/tmp/claude-1000/m3-batch10-code-r1-sonnet/chain.py`，读冻结副本里 `crash_judge_checker_common.wgsl`；草稿，不入库，因为是一次性核算）末两行：

```
tree words 50 == C_CANDIDATE_WORDS 50
BAD count 0 of 23
```

**判定（K5-1）**：守门用例被「别的流上手工写更长的树表」绕过时，由对拍的「判不了」挡住（需要卡）；偏移不重叠没有断言，是已有的盲区（推的）。推翻它的现象：造一张 12 棵树的树表放进任一对拍流，GPU 没有报 `UNDECIDABLE_TOO_MANY_ENTRIES`、也没有判不了。

### K5-2　种类 9–11 的走读、映射条目数、I-9.16 ② 的 11 种：与 CPU 逐要素

| 要素 | CPU | GPU | 判 |
|---|---|---|---|
| 十一种次序表 | `recovery.rs` 与 `walk.rs` 各有一份同名常量，同序：extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑、谱系、logged_ops（`crates/singlefs-core/src/recovery.rs:2351`、`crates/singlefs-checker/src/walk.rs:84`），种类码 1,2,3,4,6,7,8,9,10,11,12（`crates/singlefs-core/src/records.rs:385` 是最后一个） | `tree_kind_in_the_issuing_order`：位置 < 4 取位置 + 1，否则位置 + 2（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2777`），个数「const TREE_KINDS_OF_THE_TREE_TABLE: u32 = 11u;」（同文件:126）：位置 0..10 ⇒ 1,2,3,4,6,7,8,9,10,11,12 | 一致（我逐位算过） |
| I-9.16 ② 相邻两棵严格升序、同一种取先出现的 | 「TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER」被 `recovery.rs:2410` 与 `walk.rs:3526` 取 `.find` 第一条 | 恢复内核（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2963`）与走读内核（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1712`）各一份，同样取第一条后 `break`、前一棵不小于后一棵即红 | 一致；差一处：GPU checker 内核 ③ 红时 `return` 先走，不再判 ②（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1686` 起），CPU 两条都判；两边红的是同一个不变量位 I-9.16，位相同 |
| 键宽、条目宽 | 种类 9 键宽 8，10、11 键宽 8（`crates/singlefs-core/src/recovery.rs:3687` 一带），checker 条目宽 8 / 120（`walk.rs:232`、`:234`） | 常量 `DELETED_INODES_KEY_WIDTH` 8、`CONTAINER_NUMBER_KEY_WIDTH` 8（`crash_judge_common.wgsl:128`、`:129`），`CONTAINER_TREE_ENTRY_BYTES` 120、子指针偏移 34（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:132`） | 一致 |
| 引用集合扫描里种类 9–12 | `DeletedInodes => {}`（`crates/singlefs-checker/src/walk.rs:4264`）；容器树层级 1 取条目偏移 34 的子指针（「references.note(&parse_node_pointer(&entry[34..120]));」，`crates/singlefs-checker/src/walk.rs:4268`），层级 > 1 记数不全；6、7、8、12 与不认识的种类记数不全 | 同形：种类 9 空、10 与 11 层级 1 取偏移 34 的子指针、其余 `r_complete = false`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:612-621`） | 一致 |
| 映射条目数 | `deletion_tree_nodes = deleted_inodes 根有就 1 + 墓碑树各节点与容器 + 谱系树各节点与容器`（「let deletion_tree_nodes = usize::from(by_kind.contains_key(&TREE_KIND_DELETED_INODES))」，`crates/singlefs-core/src/recovery.rs:4986`） | 「let deletion_tree_nodes = select(0u, 1u, have_deleted_inodes) + container_tree_units;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3138`），`container_tree_units` 对墓碑、谱系两种累加 | 一致 |
| 种类 6、7、8、12 有根 | `key_width_for_kind` 给 `None` ⇒ `UnitMalformed`（`recovery.rs:3687-3693`） | `tree_kind_key_width` 给 `NONE` ⇒ `walk_failed(34u)`（`crash_judge_common.wgsl` 里 `if (key_width == NONE)` 那一支） | 一致 |

**判定（K5-2）**：逐要素对上；仅 ③ 红后 ② 要不要判是一处过程差异，位相同、不算分叉。推翻它的现象：对拍里某一状态 I-9.16 以外的位在 ③ 红的盘面上因此不同（我没找到把别的位挂在 ② 判完之后的写法）。

### K5-3　种类 9、10 与容器树长高：对拍等于没测这几支？

正文第三节 K5 的说法有一半已经过时，要分开写：

- 对拍文件今天有删除线的流：文件头写「删除中间态（unlink 意图与第一批）一条流钉四条判过而成立」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:11`），流里 `deleted_inodes` 根是 {1}、墓碑树开放容器有一条（`record_new_pool_through_the_first_deletion_batch`，同文件:4641）。所以种类 9、10 **不是**没有流；10b GPU 实现员报告第四节「对拍文件的流都不删文件」是删除与并发那条线第 18 批之前的说法。
- 但这几条流**只比池级 checker 那一半**：「恢复那一半这里不比：CPU 恢复从意图那一版起把 nlink = 0 的第一个文件答成没有文件」（同文件:4874），原因是 GPU 恢复内核不读 nlink（CPU 判在「if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」，`crates/singlefs-core/src/recovery.rs:5077`）。于是恢复内核里只在种类 9、10 非零根时才走的几行（`have_deleted_inodes` 与 `container_tree_units` 的累加、映射条目数的删除项）在这些流上没有逐状态对拍；它们只在回退流上对种类 11 被对拍到。这一格有去向：C712「GPU 恢复内核不判 nlink = 0 的文件」（`.claude/kb/checks-owed.md:567`）。
- 容器树层级 > 1（长高）：没有流；GPU 报告第六节第 2 条自己说明用 12 层显式栈、超过判不了交 CPU、CPU 一侧的形状核对四项「在 CPU 上走不到红」（推的）；去向在调度记录第 621 行。

**判定（K5-3）**：种类 9、10 的**恢复内核**那几行没有对拍覆盖（C712 已登）；**checker 内核**那一半有流。容器树长高没有流（已登）。推翻它的现象：对拍流名单里有一条在恢复内核上逐状态比过 `deleted_inodes` 非零根（我 grep 了删除线的几条流，都经 `compare_the_checker_on_a_deletion_flow`，只比 checker 位）。

## K6　数能不能核

| 数 | 从哪段代码推出来 | 钉它的测试 | 变异行改回旧值红不红 | 判 |
|---|---|---|---|---|
| I-3.1 `Some(3145728)` 与 `1441792` | 钉在 9 的槽使分配器门槛 9，盘上完好的 X = 21 使 checker 门槛 21（K1-4）；3145728 = 3 × 2^20，1441792 = 11 × 2^17，差 13 × 2^17（整除；单位是字节，槽宽我没在这里核） | **没有测试钉这两个数**：全仓 grep `3145728`、`1441792` 只有一处无关的 `e168_delete_ring_window_borrow.rs:4613`；用例只钉方向（K1-4 引的 `:207`、`:286`） | 钉最旧 txg 的是 `crates/mutations.tsv:2490`「第10b批 读坏的槽在回收门槛里钉在能住的最大 txg」，10b 报告第四节列它为证过（红在钉最旧 txg 的用例上）；钉「只红记账多一边」这个方向的变异行我没找到，方向断言只在用例里 | 只在实现员探针上核过 |
| 前洞那条只红 txg 20 | 前洞槽钉在 9、环里最旧有效根 10，txg 20 那一版它成了表里最小、重读读回全零照旧钉 | 集合恰好 = [20]（`...that_reread.rs:636`，K1-4 引） | `crates/mutations.tsv:2583`「第10c批 进程内重读读回全零当确认为空」，10c 报告第四节 R4 抓到（红在同文件 604 行那一句） | 核得动；注意用例造的是「挂载读报错、之后读回全零」，不是无读故障的全零槽 |
| `k2-zeros` 有洞 F 不越过 21 | RGall 之后洞里 txg 6、7、8 把上限压在 6（`CEILING_OVER_THE_HOLES` = 6，`X_TXG_AFTER_THE_HOLES` = 21） | 「"上限与抬到的 F 都是洞里旧根定的 6，不越过 X 的 txg 21"」（`crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs:367`） | `crates/mutations.tsv:2497` 改回窗口取（上限 22），10b 报告写「没证」 | 测试钉了；变异没证 |
| `k2-zeros` 没洞 F 抬到 18 | 上限照读得出的根取 18，X = 17 读回全零不并 | 「"定案的代价：上限与抬到的 F 都是读得出的最旧有效根 18，越过 X 的 txg 17"」（同文件:447） | `crates/mutations.tsv:2493`「全零改回当读坏记、并进准入抬 F 的上限」（F 停在 17）10b 报告列为证过；10c 的 `:2587` 抓到的是「抬 F 之前 X 那一版引用的单元一个都不空闲」（报告 R9） | 钉了；2493 证过，2497（有洞那一份）没证 |
| 草稿区 `SCRATCH_WORDS` = 298 736 | 内核里 `SCRATCH_END` 手写常量；链上 23 处起点 + 条数 × 字数我全算过（K5-1 脚本）；每候选 50 字 = 6 + 4 × 11 | `scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels` 只核两个常量相等（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:285`） | `crates/mutations.tsv:1940` 把 `SCRATCH_WORDS` 换回 247 792（它的替换文与原文点名同一行），checker 档，第 5 批的老行、这一批改了替换文，10b GPU 报告写全部没证 | 相等有测试；各偏移没有测试，只靠我这一次算术 |
| 展开大小 307 685 / 289 232 / 345 909 | 调度记录第 621 行转述 10b GPU 报告的打印行；今天的值要在编译后用 `kernel_inlined_size` 现量 | 只钉上界：「pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:208`），`every_kernel_stays_within_the_inlined_size_budget`（同文件:1070）打印行但不断言具体数 | 无 | 具体数只在实现员探针上量过，我核不了（要编译） |
| 第 927 行「等价」 | 见下 | 留档用例两半断言（`crash_injection_campaign.rs:1710`、`:1715`） | 删行加两行（`:2654` 一带），没证 | **冲突** |

第 927 行等价论证的冲突，两边整行并排：

- 规则（`.claude/rules/mutation-sampling.md:16`）：「| 在**所有**输入上与原式同值 | **等价变异** | 把等价性写成一条测试留档，再换一个真会改行为的变异 |」；判据（`.claude/rules/mutation-sampling.md:90`）：「- 确认在所有输入上同值 ⇒ 才是等价变异，按 `test-discipline.md` 留档。」
- 留档用例自己的文档：「/// 所以在崩溃注入摆得出的每个崩溃状态上两种交法同值，把第一截的写表退回前缀那一条变异是等价变异，这一条给它留档」（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1615`）；而它的断言展示了一个两种交法不同值的输入：整条流判「恢复自称新态而单元缺席」（「"整条流里有 (1, 5) 的根槽写，那次发布归得出来，它的数据单元两份都不在：判「恢复自称新态而单元缺席」"」，同文件:1710）、前缀不判（同文件:1715）。

推导：这条变异在「崩溃注入摆得出的状态」上同值，在「恢复策略 `ConsultWithoutNamedVerification`」这个输入上不同值，是用例自己造出来的；规则的「所有输入」在这里被缩成「崩溃注入的取样域」。实现员的理由是不改被测代码就没有换策略的开关（报告「停下交主 agent」第 2 条，调度记录第 638 行记了「要让这类变异真红得给崩溃注入加换恢复策略的开关」），所以按规则字面该归「取样点不敏感」、补敏感取样点，不是留档；它被当等价处置，主 agent 认了（第 638 行）。已知没做的第二节列了「崩溃注入加换恢复策略的开关」，所以这一格按第二节原去向写，不算打中。
推翻它的现象：崩溃注入的恢复入口里找到一处能产出 `ConsultWithoutNamedVerification` 结局（我 grep `JournalPolicy::` 在 `crash_injection.rs` 只命中 `Consult`）。

## K7　欠账与读法清单全不全

方法：把第 10 批到第 10f 批、第 927 行、锚点那几份实现员报告里「取了读法的」「做不出会红用例的」「交主 agent 的／停下交主 agent 的」每一条列出来（报告在各批目录的 `report.md`，不在仓里；十一份我都读了对应小节），逐条去 `records/2026-09-30-里程碑三开工第一轮.md`、`.claude/kb/checks-owed.md`、第二节与判决里找去向。去向的判据是找得到那一条的内容，不是找得到那一批的名字。

先说两个容器：

1. **`.claude/kb/checks-owed.md` 里第 10 批一行都没有**：`grep -c` 数「第 10 批、第 10b 批……第 10f 批」得 0。调度记录第 601 行写明 invariants 与 checks-owed 的写回等 `m3-delete-checker-r1` 与 `m3-formal-c655-c687-r1` 两轮判完另派：「范围外残留进下一份写回（等 m3-delete-checker-r1 与 m3-formal-c655-c687-r1 两轮判完）：D23 第 382 行回退候选集还写一圈窗口、invariants 第 63、143 行、checks-owed C660 标题与 C705、实现注释里引旧行名、E139 第 2 次落地后 D16 定案那句与 C660 再改。」（`records/2026-09-30-里程碑三开工第一轮.md:601`）——所以「欠账没进 checks-owed」是已知的待办，不是无声丢；但那张残留清单也没把下面「无去向」几条列进去。
2. **「下一轮清单」不是一个容器**：调度记录里「下一轮清单」出现 15 次（`grep -c`），但 `### 第 N 批` 标题只到第 9 批，第 7、8、9 批没有对应的「收拢」小节（`grep -n '^### 第'` 输出：271 第 4 批、311 第 4 批收拢、327 第 5 批、373 第 5 批收拢、416 第 6 批收拢、434 第 7 批、472 第 8 批、518 第 9 批；第 10 批各条写在第 9 批小节后面）。所以凡去向只写「进下一轮清单」的，今天没有一张表接住它们；这与代码轮第三轮判决 K6「两个去向容器今天不存在」是同一个现象，没有改善（那一轮判决要求第 9 批收拢表在这一轮收尾时写，`.claude/main-agent.md` 第 8 条）。

逐条（「有去向」写调度记录行号与原话片段；「容器空」指只写了进下一轮清单；「无」指我没找到）：

| 报告与条 | 内容 | 去向 | 判 |
|---|---|---|---|
| 10 批 core 七.1 | I-7.9 两支只留锚在镜像最新根上的窗口 | 「主 agent 认它取的 I-7.9 读法（两支只留锚在镜像最新根上的窗口，锚在被判那条根上的被罩住，零轮）」（`records/2026-09-30-里程碑三开工第一轮.md:611`） | 有去向 |
| 10 批 core 七.2、七.3、七.7、七.8、七.9 | 读不出槽的区间、Q3 两类、改钉的几条用例 | 被 10b 取代：「验收那一形 txg 0 的单元挂载之后与 txg 28–48 每一版都不空闲、不变量全绿，「只在 txg 48 之后判」已退」（`records/2026-09-30-里程碑三开工第一轮.md:624`） | 有去向 |
| 10 批 core 七.4、七.5 | A2「现行 F」取清扫交进来那条根的 F、低于它的账读得出也不读；C 先把头全重读一遍再抹 | 定案字面（调度记录第 596 行 Q1 A2、Q2 C）一致，读法细节没有单独去向 | 无独立去向（与定案字面一致，不算丢） |
| 10 批 core 八.1、八.2、八.3 | 合法历史回收、读坏槽区间、Q3 代价 | 第 611 行「它第八节第 1 条是安全问题……补丁不合，进第 10b 批重做」＋第 624 行用户两条定案 | 有去向 |
| 10 批 core 八.4、八.5 | RR4 与 q1 造不出会红的盘面，变异拿掉 | 10b 把两行补回：RR4 那行 `crates/mutations.tsv:2492`「第10b批 RR4 抬 F 那一遍的回收门槛不并根环表里挂载时读坏的槽」，10b 报告列为证过；q1 那行 `crates/mutations.tsv:2489`「第10b批 E2 树表 0 条那一版上的写行发布记根之前不重读挂载时读坏的槽」，没证 | 有去向 |
| 10 批 core 八.6 | 影子账候选过滤、checker 丁-defer 门槛改回窗口没有会红的用例，没加变异行 | **没找到**：第 611 行只转述了第 1 条；checks-owed 没有；10b 的追加行里也没有这两处（K1-2 已列） | **无** |
| 10 批 core 八.9 | 要写回条款 | D16、D18 已写回（第 601 行）；invariants、checks-owed、D23 第 382 行在残留清单里；**D16「抬 F 的上限」「根槽这一次读坏」两行写明全零另算不并，不在残留清单里**（K1-3） | 部分 |
| 10 批 checker-tier 三·1 | 抹头段摆不到二次崩溃 | 「取甲：`history.rs` 加一种留下孤儿的操作」（`records/2026-09-30-里程碑三开工第一轮.md:605`），10b 落地（第 619 行） | 有去向 |
| 10 批 checker-tier 三·2 | `crash_identity` 要搬模块 | 第 605 行「照做」＋第 619、620 行 | 有去向 |
| 10 批 checker-tier 三·3 | `DeletedRowsStillPresent` 没有用例也没有变异行 | **没找到**：`DeletedRowsStillPresent` 全仓只在 `crash_injection.rs` 的三处定义与使用（2768、2802、2837），`tests/` 与 `crates/mutations.tsv` 零命中（grep 数过）；调度记录零命中 | **无** |
| 10 批 checker-tier 三·3 | C704 harness 判法不认清扫发布，状态由主 agent 定 | C704 行在 checks-owed 第 559 行，调度记录第 605 行「C704 在崩溃注入这边处理」；那一行的状态今天没改 | 有去向（状态没更新） |
| 10b-core 七.1–七.3、七.5、八.1–八.3 | 全零读法、最旧 txg、E2 时机、I-8.5、I-3.1 记账多、k2-zeros 更正 | 「读法，主 agent 认」与用户两条定案（`records/2026-09-30-里程碑三开工第一轮.md:624`） | 有去向 |
| 10b-core 七.4 | 读不出与自证不过合成一类 `BadAtMount` | 第 624 行写「读坏的槽（读不出或不是全零而自证不过）」，没有单列这一读法 | 有去向（隐含） |
| 10b-core 八.4 | 要写回：「门槛按最小的算，根环表记最大的」「读得出而自证不过的……按记的最大 txg 算」两句改写；D18 已定项 11 清扫那一句全零与读不出同待遇没改 | 前两句 D16 已写回（第 601 行「变更史含 Q1–Q3 原话」）；**D18 那一句全零同待遇没找到去向** | 部分 |
| 10c-core 七.1、七.3 | 重读读回全零不改类（挂载读坏、重读读回全零仍记读坏；反过来仍记全零）；全零钉着之后每次记根都重读一次 | 无调度记录条目（第 625 行只写派 10c、第 630-631 行只写合入）；今天由 K1-3 判 | 无独立去向 |
| 10c-core 八.1 | 无读故障的全零槽也红一版，算不算用户 ① 里的代价 | 正文第二节写了「没有读故障时崩在记录之后、根槽之前、环没转过来的全零槽红一版」是用户已认的代价（主 agent 的观测，不是调度记录条目） | 有去向（正文第二节） |
| 10c-core 八.2 | 要写回：全零钉住、不并进上限 | 第 601 行的残留清单没有；D16 今天还写「读不出或自证不过」都并（K1-3） | **无**（K1-3 已列） |
| 10b-checker-tier 五.1–五.3、五.5、五.6 | 清单外 4 份、`TIER_MODULES`、模型把掉电步当可写挂载、`crash_identity`、越权跑门禁 | 第 619 行「主 agent 判」①②③⑤⑥ 逐条；第 620 行模块表 | 有去向 |
| 10b-checker-tier 五.4 | 快档与随机崩溃注入战役不抽这一步 | 「④ 快档与随机崩溃注入战役不抽这一步：不动已钉种子，进下一轮清单」（`records/2026-09-30-里程碑三开工第一轮.md:619`） | 容器空 |
| 10b-gpu 六.1（七.3） | 上限不从 Rust 生成，要核心层导出种类表 | 「核心层导出种类表让编译期就红，进下一轮清单」（`records/2026-09-30-里程碑三开工第一轮.md:621`） | 容器空 |
| 10b-gpu 六.2、七.2 | 容器树长高与种类 9、10 没有流 | 「② 种类 9、10 与容器树长高没有走得到的流，加一条带 unlink 的录制流进下一轮清单」（同第 621 行）；种类 9、10 的流删除线后来已有（K5-3），恢复那一半归 C712 | 容器空；一半已被后来的批次补上 |
| 10b-gpu 六.3、六.5、七.1、七.5 | I-7.9 窗口不变、extent 偏移既有差别、kb 要改 | 第 621 行 ③ ④、第 622 行（归 26-5）；invariants 第 63 行写回在残留清单（第 601 行） | 有去向 |
| 10b-gpu 七.4 | clippy 一处不是我的红 | 第 625 行「fmt、clippy 过」之后不再红 | 有去向（已消） |
| 10d-record-seam 二.2 | 接缝不在段首时不交，那种状态可能照旧假红 | 「② 只交写表下标不超过崩溃段第一个写的接缝，接缝不在段首的状态照旧会假红（随机历史里有没有没量，合入后崩溃注入跑一并看）」（`records/2026-09-30-里程碑三开工第一轮.md:629`）；正文第三节 K4 | 有去向 |
| 10d-record-seam 二.3 | 掉电步无论有没有真掉电都记接缝 | 「③ 掉电步不管真掉没掉电都记接缝」（`records/2026-09-30-里程碑三开工第一轮.md:629`） | 有去向 |
| 10d-record-seam 二.4 | 条款里 `claimed_state_missing_unit` 零命中 | 第 629 行记「记欠账随下一份写回」，checks-owed 今天没有这一行；正文第三节 K4 同一问 | 有去向（欠账登记还没落） |
| 10d-record-seam 四.1 | GPU 判的流里没有掉电步；以后把随机历史交给 GPU 判，发布表要照接缝分 | 第 629 行只记了「层 0 与 GPU 发布表照旧 `OneRecording`」（同上行号）；没记「以后把随机历史交给 GPU 判时发布表要照接缝分」那一半；checks-owed 的「接缝」一次命中是无关的 C64 | 部分 |
| 10d-anchors 验收 2 | 8 行重定锚都没证红 | 「8 行重定锚变异在 checker 档副本上 `prove-red.sh`……7 行抓到」（`records/2026-09-30-里程碑三开工第一轮.md:636`） | 有去向 |
| 927 报 停下 1、2 | 注释过时；崩溃注入换恢复策略的开关 | 「下一批清单：`crash_injection.rs` 第一截上方注释已过时；要让这类变异真红得给崩溃注入加换恢复策略的开关（改被测代码）」（`records/2026-09-30-里程碑三开工第一轮.md:638`），正文第二节「已知没做」 | 有去向（容器空，但第二节认了） |
| 10e 报 一格 | 重读只坏一部分槽、与择根不一致 | 正文第三节 K2；第 634 行「进第 10 批代码轮攻方」 | 有去向 |
| 10e 报 条款没写 1–3 | 观测成员、钩子、拒因 | 第 637 行 ① ② ③ | 有去向 |
| 10e 报 条款没写 4 | 只读挂载没碰（定案只管可写挂载） | **没找到** | **无** |
| 10e 报 一格 3、4 | 重读读出比择根那一遍新的根时判被抛弃用择根时那条根的实例表；重读里读坏的槽住着被抛弃的根，影子账看不到、之后读出被抛弃根只拿掉不补隔离 | **没找到**（第 637 行只转述了「重读只坏一部分槽」一条；报告说「没造」） | **无** |
| 10f 报 停下 1 | `BeforeTheOneReread` 文档改成列四个调用点 | 「主 agent 留」（`records/2026-09-30-里程碑三开工第一轮.md:644`） | 有去向 |
| 10f 报 停下 2 | 钩子要不要传「哪一处调的」 | 「下一轮清单：钩子要不要传「哪一处调的」（实例表与根环共用一个钩子，用例分不出）」（同第 644 行） | 容器空 |

统计（用 `awk` 数上面表的最后一列，一行可含几条）：共 35 行；有去向 20 行（含「已消」「正文」「隐含」「状态没更新」「欠账登记还没落」几种写法）、容器空 5 行（含第 927 行那一行，第二节认过）、部分 3 行、无 7 行——其中「无独立去向」2 行（10 批 core 七.4、七.5 与定案字面一致；10c 七.1、七.3 读法）不算丢，其余 5 行是真无去向。**真无声丢的 5 行**（既不在记录、不在 checks-owed、第二节也没认）：10 批 core 八.6、10 批 checker-tier 三·3 的 `DeletedRowsStillPresent`、10c 八.2（D16 全零另算要写回，同一格在 10 批 core 八.9、10b-core 八.4 里各出现一次）、10e 条款没写 4、10e 一格 3、4；另加「部分」3 行里缺的那半（D16 全零另算、D18 清扫那句全零同待遇、10d 四.1 的以后接缝）。

**判定（K7）**：欠账不全——五行找不到去向（上一段列的），另五行去向是「进下一轮清单」而那张表不存在。推翻它的现象：在调度记录或 checks-owed 里 grep 到这几条的内容（我用过 `DeletedRowsStillPresent`、`发布表`、`只读挂载`、`影子账候选`、`丁-defer`、`不隔离` 各 grep 一遍）。

## 冲突并排（两边各抄一句；kb 的长表格行只抄冲突的那一分句，行号是那一行）

| # | kb 或规则一边 | 代码或用例一边 |
|---|---|---|
| 1 | 「六处都不再按一圈窗口滤根，窗口之外的有效根」（`.claude/kb/decisions/16-发布语义.md:34`） | 「&& window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)」（`crates/singlefs-checker/src/walk.rs:5402`） |
| 2 | 「一圈窗口这个量现在只剩这一处在用，给读不出的槽划出它能住的 txg 的范围」（`.claude/kb/decisions/16-发布语义.md:34`） | 「|| !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1306`） |
| 3 | 「挂载那一刻读不出或自证不过、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它」（`.claude/kb/decisions/16-发布语义.md:36`） | 「读回全零的槽不在其内」（`crates/singlefs-core/src/mount.rs:2763`） |
| 4 | 「且落在一圈窗口里（txg 大于最新根的 txg 减 3S」（`.claude/kb/invariants.md:143`） | 「let walked = *index == newest_index || (!abandoned && !below_floor);」（`crates/singlefs-checker/src/walk.rs:8997`） |
| 5 | 「落在两个一圈窗口的交里的最新那条根的 txg」（`.claude/kb/invariants.md:63`） | 「&& window_anchored_at_the_newest_root_of_the_image.contains(*txg)」（`crates/singlefs-checker/src/walk.rs:5544`） |
| 6 | 「自证过而不可择的槽（R / S 越界、incompat 位不认得）与带越界值的槽都算自证过」（`.claude/kb/invariants.md:94`） | 「池里有系统配置槽自证过，而没有一块盘交得出可择的系统配置：这不是一个挂得上的镜像」（`crates/singlefs-checker/src/walk.rs:8706`） |
| 7 | 「确认在所有输入上同值 ⇒ 才是等价变异」（`.claude/rules/mutation-sampling.md:90`） | 「所以在崩溃注入摆得出的每个崩溃状态上两种交法同值」（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:1615`） |
| 8（代码对代码） | 「某块盘一条有效根都读不出（m 取不到）时当场只放到一圈窗口里最旧有效根」（`crates/singlefs-core/src/mount.rs:2020`，注释） | 「None => oldest_valid_root_in_the_ring,」（`crates/singlefs-core/src/mount.rs:2041`，行为已是整环） |

第 1、2、4、5 行是调度记录第 601 行点名的写回残留（去向有）；第 3 行不在那张残留清单里（K7 记）；第 6 行 10b 报告第七节第 5 条与调度记录第 624 行记了读法（条款没跟）；第 7 行见 K6；第 8 行是过时注释。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 六处退窗口 | 一致；与 D16 第 34 行字面冲突一处 | 六处全退；checker 与 GPU 的 I-7.9 重算上限仍滤窗口，有代码注释的理由，条款没写 |
| K1-2 钉最旧 txg 三处的安全方向 | 读码没找到不安全形；两处改回窗口无变异覆盖 | 影子账候选过滤、丁-defer 门槛没有会红的用例也没有变异行，调度记录里没找到去向 |
| K1-3 全零槽同一份读数同一个分类 | 实现一致；D16 第 36 行条款没写回 | 上限只并 `BadAtMount`，门槛并两类，分类挂载那一刻定、之后不改 |
| K1-4 I-3.1 多记账方向与时长 | 只红记账多的一边（读码推 + 用例断言方向） | 「最长一圈」无用例，磁盘卡住那一形无界属已知没做 |
| K3-1 I-7.9 只留一个窗口漏红 | 有盲带：F ∈ (洞里旧根 X, 窗口最旧根 W] 时 checker 看不见 | 推的，没造盘面；只有实现一侧的用例与变异在盯 |
| K3-2 `192:9:3:1` I-7.9 红 4116 | 实现越了上限，不是 checker 判错 | 那份拷贝是 10c 之前的树，全零不钉不并；Q3 代价；没复跑 |
| K3-3 I-8.5 不适用的边界 | 边界对；条款没写第三种不适用 | 一槽都没自证过仍红，有槽自证过而全池择不出报不适用 |
| K5-1 树数上限守门 | 守门只罩两条流；别的流由对拍的「判不了」挡；偏移不重叠无断言 | 链上 23 处算术我现算全对 |
| K5-2 种类 9–11 与 I-9.16 ② 逐要素 | 一致 | 仅 ③ 红后 GPU 不再判 ②，位相同 |
| K5-3 种类 9、10 与容器树长高没流 | 说法过时一半：checker 内核有删除线的流，恢复内核那几行没有逐状态对拍 | 归 C712；容器树长高仍没有流 |
| K6 数能不能核 | 3145728、1441792 与展开大小只在实现员探针上量过；第 927 行「等价」与规则字面冲突 | SCRATCH_WORDS 与 23 处偏移我现算全对；前洞 txg 20、k2-zeros 两个值有断言钉着 |
| K7 欠账与读法 | 不全：5 行真无去向、3 行部分、5 行去向是不存在的「下一轮清单」 | checks-owed 里第 10 批一行都没有，是已知待办（第 601 行） |

## 没做什么

- 没编译、没跑任何测试或变异（定义没写要编）。凡「推的」都是读码推的；凡写「量过」的只有 grep、`chain.py` 算术、`awk` 数表。K6 里钉数的变异「改回旧值红不红」除了实现员报告里写「证过」的行，我都没跑。
- 没复跑 `rerun.sh` 的 l0-5（要编译、起装置；今天的树上它也不再复现 4116）。K3-2 的结论是从两棵树的 grep 差（10b 树 `AllZeroAtMount` 零命中、冻结副本里有）与代码读法推的。
- 没读攻方腿这一轮的任何文件；没读 E139 的实验页；K2、K4 不归我，K4 只在 K7 里引了它出现的几条去向。
- K7 只读了十一份实现员报告里「取了读法的」「交主 agent 的／停下交主 agent 的」「条款没写」那几节，没通读；每份报告在各批目录下（不在仓里），所以 K7 的「无去向」只能说「我没在调度记录与 checks-owed 里找到」，不排除去向在我没读的会话记录里。
- 不判别、不采纳、不出判决；第二节列的已知没做（E139 F10、F11，钩子不区分两处，崩溃注入换恢复策略的开关，`crash_injection.rs` 第一截注释过时）与用户已认的代价，正文里只核了「是不是只有这么大」。
- 草稿：`/tmp/claude-1000/m3-batch10-code-r1-sonnet/chain.py`（读冻结副本里 `crash_judge_checker_common.wgsl`、核 23 处偏移链的一次性算术脚本，输出已贴在 K5-1）。现在不入库：它是一次性核算、没有被任何结论当成会被复跑的装置；要入库得先按 `research/` 的装置规矩登记。没有编译目录与仓副本需要删。

## 交回前核对

`python3 scripts/cite-check.py <本报告> --root <冻结副本> --root . --background research/prompts/_m3-batch10-code-r1-background.md` 末行：

```
  ✓ 核了 68 处引文，对上 68 处，没判 0 处（逐处列在上面）
```
