# 根环实现代码轮第三轮：云端正推腿报告（Sonnet，2026-10-05）

轮名 m3-root-ring-code-r3。立场：正推（条款 → 冻结副本里的行 → 钉它的测试与变异行）。格 K2、K3、K4、K5、K6（正文第三节）。K1 不归我。前两轮站住的（E2、B2、RR4、K3 R2 谓词、七类读者等）不重推。

## 零、读法与约定

- 读的是冻结副本 `/tmp/claude-1000/m3-root-ring-code-r3/tree/`。它与主工作区 `crates/` 逐文件比（脚本 `research/prompts/m3-root-ring-code-r3-sonnet-model/cmp_tree_vs_main.sh`，命令与输出原样）：

```text
compared=281 differ=0
```

  所以 `crates/…` 的行号冻结副本与主仓相同，cite-check 取默认根就能核。kb 行号一律现查（`grep -nF`），不从背景材料里数。
- **我没编译、没跑任何测试。** 开工前 `ps` 看到主 agent 的 checker 档快档在跑（`cargo test --release --no-fail-fast -p singlefs-checker-tier --tests`，经 32G 内存包装）、别的会话还有编译；实现员在改钉簇里留的整仓副本连 `target` 就是 34G（改钉簇报告第十节 `du -sh` 原样），我不去抢。凡写「抓到」「红了」的，都是实现员报告里 `prove-red.sh` 的输出（在 `/tmp/claude-1000/m3-r1/…`，没入库），不是我跑的；我只在写明处核了日志原文。跑得动的静态核我做了：段序列算闭式、变异表锚点、`match` 审计、随机历史输出点数。
- 判定三档：一致 / 冲突 / 规则没说；线索另标「线索」，由主 agent 现查坐实。每格末写「什么现象会推翻它」。
- 方案按 `crates/` 今天的实现来谈。
- 模型与脚本（都只读冻结副本、不编译）放 `research/prompts/m3-root-ring-code-r3-sonnet-model/`：`cmp_tree_vs_main.sh`、`k5_closed_form.py`、`k5_by_publish.py`、`anchors_check.py`、`targets_check.py`、`k3_match_audit.py`、`k5_arith.py`、`build_report.py`（后者把报告草稿里的引文逐条在被引文件里 `grep` 一次，命中恰好一行才写成引文）；上面各命令的原样输出另存在同目录 `results/`（`cmp.out`、`k5a.out`、`k5b.out`、`k5c.out`、`anchors.out`、`targets.out`、`k3audit.out`、`k3-fuzz.out`、`k2-calls.out`、`k4-lineage.out`）。


## 一、K2　RS1–RS3 与 R2（A）

### 1.1 RS1：进程内重读的三种结局

条款：「读得出而自证不过的，与仍读不出同一处置，按记的最大 txg 算」（`.claude/kb/decisions/16-发布语义.md:35`）；同一行还写了「读得出、自证过而按实例表被抛弃的，当没有根」（`.claude/kb/decisions/16-发布语义.md:35`）。

代码（`crates/singlefs-core/src/mount.rs`，重读函数 `reread_of_a_root_ring_slot_unreadable_at_mount`）：

- 读得出、自证过、按表有效：「RootRingSlotReading::SelfVerified(root) if !abandoned_by_table(&root, table) => {」（`crates/singlefs-core/src/mount.rs:1302`） → `ValidRoot`；
- 读得出、自证过、按表被抛弃：「RootRingSlotReading::SelfVerified(_) => {」（`crates/singlefs-core/src/mount.rs:1307`） → `AbandonedByTheInstanceTable`；
- 读不出与自证不过并成一支：「) => RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified,」（`crates/singlefs-core/src/mount.rs:1312`）。

根环表的改法在 `allocator.rs` 的 `oldest_txg_in_the_window_rereading_slots_unreadable_at_mount`：第三种结局 「return (Some(highest_txg_it_can_hold), rereads);」（`crates/singlefs-core/src/allocator.rs:1171`）（按那一槽能住的最大 txg 记着、门槛取它），第二种在记根那一处 「occupancy.occupants.remove(&ring_slot);」（`crates/singlefs-core/src/allocator.rs:1913`）（拿掉，当没有根）。逐项与 D16 第 35 行三句对得上：**一致**。

钉它的测试（`crates/singlefs-harness/tests/` 下改名后的文件，7 条，文件头一句话说明三种结局与按哪张表判）：「fn a_valid_root_read_bad_once_on_the_reread_is_counted_at_the_highest_txg_its_slot_can_hold() {」（`crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs:410`）（自证不过一格）、「fn a_valid_root_read_on_the_reread_inside_the_window_holds_the_reclaim_threshold_at_its_txg() {」（`crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs:440`）（读得出有效一格）、「fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {」（`crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs:611`）（被抛弃一格）。变异行（`crates/mutations.tsv`）：「第9批 RS1：进程内重读读得出而自证不过改回当没有根」（`crates/mutations.tsv:2310`）、「第9批 RS2 探针 p1：进程内重读读得出、自证过、按表有效的根也当没有」（`crates/mutations.tsv:2313`）、「第9批 RS2 探针 p2：进程内重读不按实例表判有效」（`crates/mutations.tsv:2314`）。三格各有一行改坏就红的变异，实现员 `prove-red.sh` 全抓到（核心簇报告第五节表前三行与 p1、p2 行；我没重跑）。

不安全方向（F 越过一条真住着非空有效根的槽）在 RS1 上只有一条路：把一条真有效的根当「被抛弃」拿掉。拿掉的判据是 「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:1540`）：实例 i 的行 (i, T_i) 在新实例起头写下、此后实例 i 不再发布，所以 i 的 txg > T_i 的根按定义就是被抛弃的（已有行在这些历史里不被改，我没逐处验写行规则）；这条路走不通。判有效用旧表或空表走的是反方向：没有行的实例的根全算有效，被抛弃的根被多留在根环表里，回收门槛更保守（安全方向），代价是记账比池级 checker 遍历（按写行之后最新那条根的表判有效）多、I-3.1 红——这才是 RS2 要钉「用写行之后那一张」的缘由（下节）。没找到不安全方向的路。

**什么现象会推翻**：任一段合法历史上，重读读出的根按「写行之后的表」判被抛弃，而它盘上其实还在回退候选集里（那时根环表该留它）；或重读读到自证不过的槽，实际记账比遍历多/少与条款的「按最大 txg 算」不符（K1 的安全方向代价）。

### 1.2 RS2：按哪张实例表判、记根的四处

- 条款字面：D16 第 35 行「按实例表仍然有效」；代码里表住在分配器上 「装上根环表时是空表；可写挂载在写行那次发布的表一定下来、第一次记根之前换成它」（`crates/singlefs-core/src/allocator.rs:1099`），换表只有一处 「allocator.note_the_instance_table_judging_the_rereads_of_root_ring_slots(」（`crates/singlefs-core/src/mount.rs:4486`）（`establish_instance` 里，写行那次发布的表）。`rebuilt_allocator` 里原来那句预设最新根那张表的赋值已删：`grep -n "note_the_instance_table_judging_the_rereads_of_root_ring_slots(" crates/singlefs-core/src/*.rs` 只剩定义与这一处调用。
- 记根的四个调用点（命令与原样输出）：

```text
$ grep -n "record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(" crates/singlefs-core/src/mount.rs crates/singlefs-core/src/transaction.rs
crates/singlefs-core/src/transaction.rs:1757:    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
crates/singlefs-core/src/transaction.rs:7754:    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
crates/singlefs-core/src/mount.rs:3452:            record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
crates/singlefs-core/src/mount.rs:4686:                record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
$ grep -n "rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(\|rollback_floor_ceiling(" crates/singlefs-core/src/*.rs
crates/singlefs-core/src/mount.rs:2188:    rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
crates/singlefs-core/src/mount.rs:2692:    let ceiling = rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
crates/singlefs-core/src/mount.rs:2973:            let ceiling = rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
$ grep -rn "rollback_floor_ceiling(" crates --include=*.rs | grep -v "singlefs-core/src/mount.rs"
crates/singlefs-checker-tier/src/on_device_modes.rs:279:    let ceiling = rollback_floor_ceiling(
crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:1987:    let Ok(ceiling) = rollback_floor_ceiling(
crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:2263:    rollback_floor_ceiling(
crates/singlefs-harness/src/model.rs:1291:    pub fn rollback_floor_ceiling(&self) -> Option<ModelCheckpointTxg> {
crates/singlefs-harness/src/model.rs:1364:        view_without_the_roots.rollback_floor_ceiling()
crates/singlefs-harness/src/model.rs:2422:        let ceiling = self.rollback_floor_ceiling();
crates/singlefs-harness/src/model.rs:3426:        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
crates/singlefs-harness/src/model.rs:3541:        assert_eq!(model.rollback_floor_ceiling(), Some(ModelCheckpointTxg(6)));
```

  上面前四行就是四处：`mount.rs:3452`（`publish_empty_after`，零单元发布）、`mount.rs:4686`（`establish_instance` 里零单元写行那一支）、`transaction.rs:1757`（`prepare_the_row_publish_on_a_version_without_file`）、`transaction.rs:7754`（`publish_admitted`，带单元的发布）。
- 钉：「按写行之后那张表」「fn the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote() {」（`crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs:711`） + 变异行 「第9批 RS2：写行那次发布的表定下来之后不换进程内重读判有效用的表」（`crates/mutations.tsv:2315`）；`mount.rs:3452` 一处 「第9批 RS2 探针 q1：树表 0 条那一版上的暖机零单元发布记根不重读」（`crates/mutations.tsv:2316`） 钉着（用例 「fn the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount() {」（`crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs:822`））；`transaction.rs:7754` 由 RS1 那几条覆盖写发布的用例带着。`mount.rs:4686` 与 `transaction.rs:1757` 两处**没有会红的用例**（核心簇报告第七节第 1、2 条：走不到 / 做不出）：变异表里没有它们的行（`grep -c "探针 q2\|探针 q3" crates/mutations.tsv` 为 0）。第二处在 `checks-owed.md` 有 C702，第一处没有行（见 K6 表第 1 行）。
- 结论：**一致**；两处记根点没有钉，已知，不是新缺口。

**推翻**：有一条路径在 `rebuilt_allocator` 交回之后、`establish_instance` 换表之前记根（空表读到，被抛弃的根被当有效，安全方向多记）；或有人去掉换表那句、回到旧表（被抛弃的根被当有效，安全方向，但记账比 checker 遍历多、I-3.1 红；变异行 「第9批 RS2：写行那次发布的表定下来之后不换进程内重读判有效用的表」（`crates/mutations.tsv:2315`） 会红）。

### 1.3 RS3：准入抬 F 的上限并进「挂载那一刻读不出」的槽

条款两行同写一件事：「窗口里挂载那一刻读不出、之后这个进程没写过的槽，当作可能是一条非空有效根，F 不越过它」（`.claude/kb/decisions/16-发布语义.md:37`）（抬 F 的上限一行）与「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，判回退候选与算 F_生效 时当没有根」（`.claude/kb/decisions/16-发布语义.md:38`）（「根槽这一次读坏」一行：准入抬 F 那一半只并读不出的，判回退候选与 F_生效 并读不出或自证不过的当没有根）。9-5 报告第七节第 5 条问的「两行字面不一样」在今天的 kb 里已经对齐（用户 2026-10-04 定 Q5，写回丙落盘），实现取「只并读不出的」正是它。

代码：新函数 `rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount` 里，这一遍读坏、又不是「知道住着根」的槽才取封顶 「.filter(|highest_txg_it_can_hold| is_inside_the_window(*highest_txg_it_can_hold))」（`crates/singlefs-core/src/mount.rs:2324`），结果 「ceiling_from_the_roots_read.min(highest_txg_it_can_hold)」（`crates/singlefs-core/src/mount.rs:2329`）。准入抬 F 的两处产品调用点都换成它（上面 `k2-calls.out` 后两段：`mount.rs:2692`、`mount.rs:2973`）。

我从三个方向找反例，都没找到：

1. 「F 越过一条真住着非空有效根的槽」：封顶对象是这一遍还读坏的、窗口里（锚 = 读得出且按表有效的最新根）、txg 不低于 F 的槽；封顶值取那一槽能住的**最大** txg，真住在那里的任何根 txg 都不高于它，F 取 min 就不越过。窗口外的槽按定义不并（窗口已许可它被复用）。「挂载时读得出、自证不过」的槽不在根环表里（核心簇报告第七节第 5 条），所以一条读得出但位被翻坏的有效根不挡 F——这是用户 Q5 认下的，不是实现的洞。
2. 「抬 F 被一个从没住过根的槽永远挡住」：封顶只在槽的 txg 还在窗口里时有效；窗口锚随这个进程每发一次布往前走，3S 次发布之后 `highest_txg_it_can_hold` 掉出窗口、封顶自己消失；那一槽被这个进程写到时也从表里转成 `ValidRoot`。所以挡的时间有上界（至多一圈窗口的发布数），不是永远。代价是读故障时抬得少一点（用例 「fn root_slot_already_unreadable_at_mount_is_taken_as_a_possible_root_and_holds_the_admission_ceiling_at_its_txg(」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:2363`） 钉 `FloorAlreadyAtTheCeiling { floor: 0, ceiling: 0 }`）。
3. 旧签名仍有调用者不带封顶：`rollback_floor_ceiling` 保留，转手传空表。除 `mount.rs` 自己外还有三处调用（`k2-calls.out` 末段）：`crates/singlefs-checker-tier/src/on_device_modes.rs:279`（checker 档在盘模式里真抬 F）、`e158_root_choice_repair.rs:1987`、一份 harness 用例 `admission_raises_the_floor_before_refusing.rs:2263`。核心簇报告第一节写明「签名不动」。**线索**：在盘模式里挂载读不出槽的盘上抬 F，不会像产品路径那样被封顶（推的，没造盘面）。另 `crates/singlefs-harness/src/model.rs:1291` 的参考模型 `rollback_floor_ceiling` 里没有「挂载以来读不出」这个概念，持久读错的历史上模型与实现的上限会分开；今天随机历史的读错都是瞬时的（改钉簇报告第三节末），所以对拍不出，要给模型加持久读错故障时先改模型（推的）。

钉：两条用例 「fn admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount() {」（`crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs:180`）（X 读不出，上限 21）与对照 「fn admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x() {」（`crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs:217`）；变异行 「第9批 RS3：准入抬 F 的上限不按挂载那一刻读不出、之后没写过的槽封顶」（`crates/mutations.tsv:2317`） 与合入后改钉的那一条 「第9批改钉 RS3（合入后改钉的那一条）：挂载那一刻读不出的槽不封顶准入抬 F 的上限」（`crates/mutations.tsv:2371`）。改钉簇 `prove-red` 日志（`/tmp/claude-1000/m3-r1/batch9-repin/prove/round2-p4.out`）里那一行「抓到」，红在 `admission_raises_the_floor_before_refusing.rs:2391`（我核了日志原文，没重跑）。

结论：**一致**；线索一条（旧签名三处调用者、参考模型无封顶）。

**推翻**：有一段合法历史，挂载那一刻读不出的槽后来经重读以外的路径被当作无根、而准入抬 F 把 F 抬过了它盘上那条非空有效根（池级 checker 的 I-3.1 / I-7.9 红）；或封顶在一圈窗口之后仍挡着（`FloorAlreadyAtTheCeiling` 在锚过了 `highest + 3S` 之后还报）。

### 1.4 R2：`mount_transparency.rs` 与 core 是不是同一个口径

- 条款：「某块盘一份自证过的系统配置都没有时，取号那一步也拒」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）；R2 本身（有槽设备报读错、且没有一块盘两槽都自证过）在同一行写「用户定，被攻过零轮」。
- core：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。」（`crates/singlefs-core/src/recovery.rs:3318`），实现 `.any(|slots| slots.slots_reporting_a_read_error > 0)` 加 `!…both_slots_self_verified()`；读槽 「for offset in [0, slot_spacing_in_bytes] {」（`crates/singlefs-core/src/recovery.rs:3296`），自证过要整槽校验和过且 fsid 与本池相同。
- harness 判法另写的一份：「不调实现的 `recovery::system_configuration_witness_is_undecidable`：判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。」（`crates/singlefs-harness/src/mount_transparency.rs:1359`）；读槽 「let offset = DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes);」（`crates/singlefs-harness/src/mount_transparency.rs:1341`） 两槽、同一个 fsid 过滤；「某块盘一份自证过的都没有」另起一函数 「fn some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(」（`crates/singlefs-harness/src/mount_transparency.rs:1378`），在 `newer_publish_witnessed_on_the_crash_image` 里与判不出并成一个「按真」。
- 差一处结构、没有差口径：core 把「某块盘一份都没有」做成单列成员 「读阶段判完之后，逐盘核或取号那一遍读到某块盘两槽里一份本池自证过的系统配置都没有」（`crates/singlefs-core/src/mount.rs:452`）（只在读阶段之后逐盘核或取号那一遍报，C664 欠着两种拒因分开报），崩溃镜像是静态的，harness 在同一个函数里判，布尔结果相同。撕裂轮换（读得出、自证不过、设备没报错）两边都不算判不出。
- 钉：「fn a_witness_refusal_on_a_torn_system_configuration_rotation_is_a_refusal_for_the_wrong_reason() {」（`crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs:104`）、「fn a_witness_refusal_on_a_device_without_any_self_verified_system_configuration_slot_is_legal() {」（`crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs:132`）；变异行 「第9批 RR8：harness 判法复算判据 N-配置 回到取甲」（`crates/mutations.tsv:2318`）、「第9批 RR8：harness 判法复算判据 N-配置 不认某块盘一份自证过的系统配置都没有」（`crates/mutations.tsv:2319`）（实现员 `prove-red` 抓到）。
- 结论：**一致**。fsid 不符落哪一边条款没说，两边都当「读得出而自证不过」处理，欠账 C707 记着。

**推翻**：一张崩溃镜像上 harness 判「判据按真（拒是合法的）」而 core 的可写挂载并不拒，或相反，且不是 C664 那种两种拒因报成一个的差别。


## 二、K3　I-7.9 两窗口、I-9.28 与「部分没判」（C）

### 2.1 I-7.9 两支各取两个窗口的交、交空不判

条款：「「之前的根」（两支都取）取两个一圈窗口的交里 txg 比它小的那些」（`.claude/kb/invariants.md:63`）（I-7.9 行）。代码（准入支）：「&& window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)」（`crates/singlefs-checker/src/walk.rs:5248`）；带卸载记号支：「&& window_anchored_at_the_newest_root_of_the_image.contains(*txg)」（`crates/singlefs-checker/src/walk.rs:5393`），交空时这条根记进「判不了」，不适用理由写 「或带卸载记号、两个一圈窗口的交里没有它之前的根」（`crates/singlefs-checker/src/walk.rs:5448`）。镜像一侧的锚取最新根：「一圈窗口（D16（发布语义） 已定项 1）：N 取最新根（它按自己的实例表恒有效），3S 是根环槽数。」（`crates/singlefs-checker/src/walk.rs:8080`）。**一致。**

问「有没有一张坏镜像（F 越过了上限）因为交为空而漏红」，我推到了一个小结论：根槽位置只由 txg 定（「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:114`）、「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:115`）），连续 3S 个 txg 各占一个槽，没有洞时环里恰是 (N − 3S, N] 里的每个 txg，所以环里每一条根都满足「txg > N − 3S」，**第二个窗口在没有洞的环上什么也不滤**；交为空只会在被判的根 r 恰是环里最旧的一条（r = N − 3S + 1，它之前的根早被盖掉）时出现，那时它连同实例的前一条根都不在环里，在 `judge_rollback_floor_raises_against_their_ceilings` 开头（「let Some(previous_root_of_the_same_instance) = roots」（`crates/singlefs-checker/src/walk.rs:5360`） 那个 `else { continue; }`）就被当作「不是抬 F 的根」跳过，新旧写法一样。第二个窗口真起作用的只有洞：洞那一槽留着上一圈的根（txg ≤ N − 3S），旧写法把它当「之前的根」才误红。所以新增的漏红只有「洞里留下的上一圈的根恰是 r 真正的前一条」这一类——证据已不在盘上，判不了，没有别的路。没找到「本来能红、因为交空而漏掉」的新类。

钉与证据：四条用例 + 两条改钉用例（`pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs`：「fn an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole(」（`crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs:551`） 与 「fn an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant(」（`crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs:593`）），变异行 「第9批改钉 I-7.9 带卸载记号那一支：只取锚在被判那条根上的窗口，不取锚在镜像最新有效根上的那个」（`crates/mutations.tsv:2364`）、「第9批改钉 I-7.9 带卸载记号那一支：两个窗口的交里恒判成立」（`crates/mutations.tsv:2365`） 与准入支三行 「第9批 I-7.9 一圈窗口：只取锚在被判那条根上的窗口（K1 类洞，txg 2 顶上来又红）」（`crates/mutations.tsv:2354`） 等；改钉簇 `prove-red` 抓到（核日志原文，没重跑）。随机历史两批（仓里有产物 `research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/`，我按 `summary.txt` 现数）：

```text
$ awk 汇总 research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/summary.txt（每行 out-文件 seeds= … checker_red_seeds=）
R230 seeds=230 checker_red_seeds=0
R150 seeds=150 checker_red_seeds=0
```

9-6 报告里甲版（条款字面，只锚在被判根上）同两批红 21 段、9 段，乙版各剩 1 段（种子 31045、1037，「乙剩下的两段：种子 31045、1037」（`/tmp/claude-1000/m3-r1/batch9-checker/patch/report.md:177`）），改钉后这两颗种子在仓里的输出是 「FUZZ seed=31045 s=8 arm=window faults=false journal=1048576 lives=3」（`research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/out-R230-r11.out:46`） 与 「FUZZ seed=1037 s=4 arm=window faults=false journal=805306368 lives=4」（`research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/out-R150-a2.out:13`）（`checker_reds=[]`）。同一装置在 9-6 的甲、乙两版上能红（21 / 9 与 1 / 1），所以这个 0 不是空过。

弱处（不是冲突）：带记号那一支「洞上再挂」那条用例在挂载之后只断言「不许 Violated」（上面第一条用例末尾 `!matches!(… Violated(_))`），没有钉成「报不适用」，改成恒报成立的变异不会让它红；「两个窗口的交里恒判成立」那一行靠的是第二条用例（F 抬到 v + 1 仍红）。

**推翻**：一张环无洞的镜像，被判根 r 之前有两条以上的根都在两个窗口里，而 I-7.9 判不出已抬过头的 F；或层 0 / 随机历史上出现 I-7.9 在合法状态上红（量「误红」的两批 0 段红是改钉簇的一次抽样，不是我跑的）。

### 2.2 I-9.28 四种结局的边界

条款四种：「判定四种：有一处诞生 txg 对不上 → 不成立」（`.claude/kb/invariants.md:315`）；「或条目比字段表短 → 部分没判，列出这些根（盘号、实例、txg、为什么没判）」（`.claude/kb/invariants.md:315`）；「没有一棵树出现在两个不同的树表单元里（几条根共用一个树表单元时拿同一份字节比自己，是恒真判定）→ 不适用」（`.claude/kb/invariants.md:315`）。代码：读树表那一遍记下三种读法（`TreeTableReading`）：「references.tree_table_reading = TreeTableReading::Unreadable;」（`crates/singlefs-checker/src/walk.rs:3776`）、「references.tree_table_reading = TreeTableReading::SomeEntryShorterThanTheFieldTable;」（`crates/singlefs-checker/src/walk.rs:3785`），判法里次序是违例（`judge` 里照比）、部分没判 「if !roots_whose_tree_table_was_not_read_in_full.is_empty() {」（`crates/singlefs-checker/src/walk.rs:4683`）、不适用 「} else if compared_trees == 0 {」（`crates/singlefs-checker/src/walk.rs:4691`）、其余成立；`into_report` 里次序 「} else if let Some(unjudged) = self.partially_judged.get(invariant) {」（`crates/singlefs-checker/src/image.rs:282`） 排在违例之后、成立与不适用之前，与细则（records 第 9 批「用户 2026-10-04 弹窗定」那条）「比出不一致报违反；都一致而有…报部分没判；都没有、也没比到…报不适用；其余成立」同序。逐边界：

| 边界 | 代码 | 钉 | 判 |
|---|---|---|---|
| 树表读不出（指针非零） | `Unreadable` → 部分没判，说明列「盘 N 上的根（实例 i、txg t）：树表读不出」 | 「fn a_readable_root_whose_tree_table_is_unreadable_while_the_others_agree_is_reported_as_partially_judged_naming_that_root」（`crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs:560`）；变异行三条（报成成立 / 报成不适用 / 不列进没判到） | 一致 |
| 条目比字段表短 | `SomeEntryShorterThanTheFieldTable` → 部分没判 | **没有用例、没有变异行**：`grep -rn "比字段表短\|SomeEntryShorter" crates --include=*.rs` 在 `walk.rs` 之外只命中 `image.rs` 的一句文档，`crates/mutations.tsv` 里 0 行 | 条款有、代码有、**钉没有**。只能由坏镜像（树表头里条目宽改窄且校验和重封）打到，合法历史走不到 |
| 没有一棵树出现在两个单元里 | `compared_trees == 0` → 不适用 | 「fn no_tree_seen_in_two_different_tree_table_units_is_reported_as_not_applicable() {」（`crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs:588`）；变异行 「第9批改钉 I-9.28：只出现在一片树表里的树也拿来比」（`crates/mutations.tsv:2370`） | 一致 |
| 违例与部分没判同时 | 违例压过部分没判 | 「fn a_birth_txg_disagreement_is_reported_as_violated_even_when_another_roots_tree_table_is_unreadable」（`crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs:539`）；变异行 「第9批改钉 第四种判定：部分没判压过违例」（`crates/mutations.tsv:2369`） | 一致 |

一处读法要摆出来：`roots` 是根环里每条自证过的根，不按窗口滤、含被抛弃时间线的根。被抛弃的旧根的树表单元可以被合法复用（窗口已许可），读出来校验和对不上就是「读不出」。所以合法镜像上 I-9.28 会因此常报「部分没判」而不是「成立」——它不红、不掩盖违例（违例排在前面），只是判到的那部分被标成没判全。这是细则的字面结果，不是错；记下来是因为层 0 计数把它并进「不适用」（下节），那种状态在计数上看不出判到了多少棵树。

**推翻**：一张合法镜像上 I-9.28 红；或一张树表条目被改窄的坏镜像上 I-9.28 报成立 / 不适用（缺的那一条钉打到才知道）。

### 2.3 「部分没判」并进层 0 的「不适用」，全仓穷举 `match`

代码：「InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {」（`crates/singlefs-checker-tier/src/crash.rs:1493`） 计进不适用那一格，上一行注释写明理由（`评估过 + 不适用 = 状态数` 照旧成立、进度文件格式不多一格）。方向是保守的：部分没判**不会**被算进「评估过」，也不会被算成违例；代价是「评估过」的计数在这类状态上偏小。我查了两头：

- 「评估过必须 > 0」的名单（各条流的 `must_evaluate`）里没有 I-9.28（`grep -n "I-9.28" crates/singlefs-checker-tier/tests/*.rs` 只命中 GPU 对拍那一份），所以并进去不会让这类钉红（另一类逐条精确值的钉见 K4 3.7）。
- 全仓对 `InvariantVerdict` 的穷举 `match`：我写了审计脚本（`k3_match_audit.py`，只读，原样输出）：

```text
files mentioning InvariantVerdict members: 83
files with 0 PartiallyJudged: 22
   ('crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs', 1, 0, 1, 0)
   ('crates/singlefs-checker-tier/tests/crash_points_of_the_rollback_publish.rs', 0, 0, 1, 0)
   ('crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs', 1, 0, 1, 0)
   ('crates/singlefs-harness/tests/an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor.rs', 0, 1, 0, 0)
   ('crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs', 1, 0, 1, 0)
   ('crates/singlefs-harness/tests/instance_table_page_full.rs', 0, 0, 1, 0)
   ('crates/singlefs-harness/tests/instance_table_second_page_write.rs', 2, 0, 1, 0)
   ('crates/singlefs-harness/tests/journal_last_record_flag_of_a_multi_record_publish.rs', 0, 0, 3, 0)
   ('crates/singlefs-harness/tests/many_inodes_write_and_read.rs', 2, 0, 3, 0)
   ('crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs', 0, 0, 2, 0)
   ('crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs', 0, 0, 1, 0)
   ('crates/singlefs-harness/tests/overwrite_in_one_instance.rs', 2, 0, 1, 0)
   ('crates/singlefs-harness/tests/publish_failure_resent_unchanged.rs', 1, 0, 1, 0)
   ('crates/singlefs-harness/tests/release_checksum_quarantine.rs', 3, 0, 2, 0)
   ('crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs', 1, 0, 2, 0)
   ('crates/singlefs-harness/tests/reused_allocation_record_overlap.rs', 1, 0, 1, 0)
   ('crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs', 1, 0, 0, 0)
   ('crates/singlefs-harness/tests/root_ring_turn_in_one_mount.rs', 0, 0, 1, 0)
   ('crates/singlefs-harness/tests/system_configuration_per_device_redundancy.rs', 0, 4, 0, 0)
   ('crates/singlefs-harness/tests/test_only_switches_force_every_branch.rs', 0, 0, 1, 0)
   ('crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs', 1, 0, 4, 0)
   ('crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs', 1, 0, 2, 0)
--- matches with a wildcard arm after InvariantVerdict arms
crates/singlefs-harness/tests/checker_known_bad_images.rs 1837 _ => InvariantVerdict::Holds,
crates/singlefs-harness/tests/checker_known_bad_images.rs 2951 _ => InvariantVerdict::Holds,
```

  逐行读它：22 份没提到 `PartiallyJudged` 的文件里，`=>` 臂带 `InvariantVerdict::` 的一处都没有（用的是 `==`、`matches!`、`let … else`）；两处通配臂 `_ => InvariantVerdict::Holds`（`checker_known_bad_images.rs:1837`、`:2951`）在**构造期望值**的 `match` 里，不是对判定做 `match`。生产代码里的每个对判定的 `match`（`crash.rs`、`crash_injection.rs`、`bad_disk_input.rs`、`history.rs`、`fault_injection.rs`、六个装置二进制）都有 `PartiallyJudged` 一臂，处置是「不红、不算判绿」：例 「| InvariantVerdict::PartiallyJudged(_) => false,」（`crates/singlefs-checker-tier/src/bad_disk_input.rs:1324`）（「checker 一条都没判」那一格按判过算）。实现员报告里的 clippy 命令都带 `-D clippy::wildcard_enum_match_arm` 那七条（例：核心簇报告第六节、改钉簇报告第五节），合入验证的 clippy `--all-targets --all-features` 过了（records 第 9 批「改钉簇与 9-9 合入」那条），所以穷举 `match` 没有通配臂吞掉新成员是编译期保证的，不靠我读。
- `!matches!(found, InvariantVerdict::NotApplicable(_))` 这一类（`checker_known_bad_images.rs:3449`、`:7105`、`:7193`）把部分没判算成「判过」，方向是让这些断言更严（遇到就红），不会放过。

结论：**一致**；没有一条流把「部分没判」的状态算成判过。

**推翻**：某个层 0 流的 `must_evaluate` 之外有逐条 `expected_evaluated` 精确值（见 K4 末「线索二」），在 I-9.28 变成「部分没判」的状态上与计数对不上。


## 三、K4　GPU 与 CPU 同口径（D）

读法：同一个判法在两侧各取一两行定案的关键语句并排；偏移与常量我自己算了一遍。**GPU 对拍我一条都没跑**（要 `gpu` 特性、checker 档，主 agent 的快档不带它；这台机器有两张卡，但跑它是重型测试）；新加的 13 条用例与 20 行变异全是实现员「推的、没跑」。所以下面的「一致」都是读码层面的一致。

### 3.1 I-3.5（GPU 新判）逐要素

- R⁻ 取 txg 比 R 小的最大一组：CPU 按相邻两组 「for adjacent in groups.windows(2) {」（`crates/singlefs-checker/src/walk.rs:534`）；GPU 「if (lt64(other_txg, version_txg) && (!has_previous || lt64(previous_txg, other_txg))) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3090`）。**一致**。
- gap-row 接得上：CPU 「row.instance == previous_instance && row.published_checkpoint_txg == previous_txg」（`crates/singlefs-checker/src/walk.rs:476`）、GPU 「if (instance == previous_instance && eq64(row_txg, previous_txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3029`）；中间实例每个都有行、都是 (0, 0)：CPU 「== u64::from(version_instance - previous_instance - 1);」（`crates/singlefs-checker/src/walk.rs:486`）、GPU 「if (row_of_the_previous_version && instances_between == version_instance - previous_instance - 1u && every_row_between_published_nothing) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3050`）。**一致**。
- N3（洞接不上时用户可见树里的单元不判）：CPU 「if hole == HoleInTheCandidateSet::HoleNotBridgedByTheInstanceTableRows」（`crates/singlefs-checker/src/walk.rs:577`）、GPU 「if (hole == APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS && in_a_user_visible_tree) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3127`）。**一致**。
- (b) 整棵继承：CPU 「root.root_pointer_birth_txg < previous_txg」（`crates/singlefs-checker/src/walk.rs:563`）、GPU 「if (!lt64(root_pointer_birth_txg, previous_txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3064`）；(a)：CPU 「let born_after_the_previous_version = birth_txg > previous_txg;」（`crates/singlefs-checker/src/walk.rs:586`）、GPU 「let born_after_the_previous_version = lt64(previous_txg, appearance_unit_birth_txg(u));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3133`）。**一致**。
- 引用不全跳过那一对：CPU 「if !previous_references_are_complete || !version.references_are_complete {」（`crates/singlefs-checker/src/walk.rs:549`）、GPU 两处 「if (!has_previous || sw(row + 3u) != 1u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3095`） 与 「if (!previous_references_are_complete) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3104`）。**一致**。
- 钉：对拍用例 2 条（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）+ 变异行 「第 9 批 GPU 判器：I-3.5 出现规则那一判恒成立」（`crates/mutations.tsv:2298`）、「第 9 批 GPU 判器：I-3.5 不认整棵继承下来的用户可见树（(b) 那一支丢了）」（`crates/mutations.tsv:2299`），没跑。gap-row 与 N3 在 GPU 侧**没有流、没有变异行**（9-3 报告第四节 S7 自己写了），CPU 侧有（`pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs` 五条 + 变异行 bb、bc、bd、be、bf、bg）。不是冲突，是 GPU 这一半没钉。

### 3.2 I-9.28

- 归并键与判违例：CPU `if distinct_tree_tables.len() < 2 { continue; }` 之后 「let birth_txg_is_the_same_in_every_readable_root = sightings」（`crates/singlefs-checker/src/walk.rs:4658`）；GPU 「if (distinct_tables >= 2u && !same_birth) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:761`）。
- 条目偏移：CPU 「const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;」（`crates/singlefs-checker/src/walk.rs:3405`） = 108；GPU 「let birth = rd64(tree_table_base, o + 108u);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:482`）。
- 树表读不出 / 条目比字段表短：GPU 的注释写明 「树表指针全零 ⇒ 没有落点；树表读不出、条目比字段表短 ⇒ 有落点、没有条目」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:769`）（代码里 `!tt.ok || tt.entry_width < 200u` 就直接返回），与 CPU「记 `Unreadable` / `SomeEntry…`、条目不进归并」同。
- 部分没判与不适用 GPU 不置位：「不适用（没比到两个不同的树表单元）与部分没判（有根的树表指针非零而读不出、或条目比字段表短）都不置位，与成立一样」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:792`）。对拍只比违例位：CPU 侧收红的是 「if let InvariantVerdict::Violated(_) = verdict {」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1799`），收成立的 「if let InvariantVerdict::Holds = verdict {」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1814`） 只用在「CPU 判过而成立」的几条断言里，部分没判与不适用两样都不进那个集合。**两形都是 0 位，没有一形两边对不上**（读码）。
- 钉：流 2 条 + 变异行 「第 9 批 9-9 GPU 判器：I-9.28 恒成立（扫描内核不归并根表里每一条根的树表）」（`crates/mutations.tsv:2372`）、「第 9 批 9-9 GPU 判器：I-9.28 把树表读不出的根（部分没判）报成违例」（`crates/mutations.tsv:2374`）（没跑）。9-9 报告 Q3 说的缺口在：「fn the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant(」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:3461`） 只断言 CPU 一个状态都不红 I-9.28，没有断言 CPU 真报了「部分没判」；GPU 也报「成立」时这条断言照样绿。

### 3.3 I-7.8 认领

认领之后按整单元跳：CPU 「first_slot_not_claimed = slot + claimed_slots;」（`crates/singlefs-checker/src/walk.rs:3216`） / GPU 「first_slot_not_claimed = add64(slot, u64_of(claimed_slots));」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3664`）；码 1、码 3 两槽码 2 一槽：CPU 「1 => (105, DATA_UNIT_BYTES / SLOT_BYTES),」（`crates/singlefs-checker/src/walk.rs:3168`） / GPU 「码 1、码 3 两槽，码 2 一槽；解不出 ⇒ 0，这一槽不认领。不看 fsid」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:750`）；没发布过的孤儿滤掉：CPU 「&& birth_txg > row.published_checkpoint_txg」（`crates/singlefs-checker/src/walk.rs:3154`） / GPU 「if (!is_zero64(row_txg) && lt64(row_txg, birth_txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3679`）。起点槽集合两侧不同（CPU 扫基镜像写过的扇区加持久写的起点槽，GPU 扫落点表里每个被碰过的槽，9-9 报告第三节第 2 条自己写了），我推了一遍：多出来的槽只有被认领单元的第二槽与没持久的写碰过的槽，前者被跳过、后者读出来是基镜像（零），数到的码 2 节点相同。钉：对拍 1 条 + 变异行 「第 9 批 9-9 GPU 判器：I-7.8 扫描方向认领之后一律只跳一槽（数据单元第二槽上的伪头又被数进出现过的树 ID）」（`crates/mutations.tsv:2375`）（没跑）。**一致**。

### 3.4 I-7.9 两个窗口、RS5、RS6

- 窗口：CPU 「checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.anchor_txg」（`crates/singlefs-checker/src/walk.rs:7027`） / GPU 「return lt64(anchor_txg, add64_saturating(txg, u64_of(root_ring_slot_count)));」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1200`）；3S：GPU 「let root_ring_slot_count = sw(H_BASE + H_REGIONS) * sw(H_BASE + H_SLOTS_PER_REGION);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1207`） 对 CPU `geometry.regions * geometry.slots_per_region`；两个窗口：GPU 「&& in_the_one_revolution_window(txg, newest_txg_of_the_image, root_ring_slot_count)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1234`）（带记号支）与 「|| !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1254`）（准入支）；窗口里一条都没有不判：GPU 「if (have_newest_before && lt64(newest_before, raising.floor)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1240`） 对 CPU 交空 `continue`。**一致**。钉：对拍 3 条 + 变异行 「第 9 批 9-9 GPU 判器：I-7.9 准入那一支不取一圈窗口（改回旧判法，之前的根取根环里全部）」（`crates/mutations.tsv:2377`） 等三行（没跑；9-9 报告 Q2 自己写了「带记号那一支那条用例要等 CPU 那一份合入才会绿」，现在两份都合入了）。「锚在被判根上」那一道在今天的定义下是多余的（锚在镜像最新根上的窗口已经包含它，9-9 报告第三节第 3 条），没变异行，这是等价变异，不是缺口。
- RS5 卸载记号其余位：core 「UnmountMarker::of_unmount_marker_field_of_a_new_root_segment(reader.get_u32())?;」（`crates/singlefs-core/src/journal.rs:346`） / GPU 「if ((rd32(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER) & 0xFFFFFFFEu) != 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:998`）；指针头五样：GPU 「fn record_pointer_head_is_the_first_version(base: u32, o: u32) -> bool {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:961`） 对 core 的 `judged()`（MAC / nonce、算法类型、压缩码、压缩长度、extent 偏移五样）。
- RS5 / RS6 七段取记录：core 「unmount_marker: record.new_unmount_marker,」（`crates/singlefs-core/src/recovery.rs:3601`） / GPU 「next.unmount = (rd32(r.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER) & 1u) != 0u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1304`）；偏移我按 core 的读序自己加：记录头前 4+2+1+1+4+4+4+6+8+44+8+1+4+4+4 = 99 → 树表指针 99、映射根 185（99 + 86）、水位 271（185 + 86）、F 279、实例表指针与分配记录树根在新根段里再往后，GPU 「next.watermark = rd64(r.base, 271u);」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1308`） 与 「next.floor = rd64(r.base, 279u);」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1309`） 对得上。走读内核施加版四条指针都取记录：GPU 「WALK_KIND_VERSION：四条指针都取自那条记录的新根段」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2871`） 对 CPU 「fn walk_version_applied_only_by_records(&mut self, version: &VersionAppliedOnlyByRecords) {」（`crates/singlefs-checker/src/walk.rs:1349`）（树表、映射根、实例表、分配记录树根四条都从 `new_root_segment` 切）。**一致**。变异行 12 条（9-3）没跑；9-3 报告 S4 把四条指针头都判全算在「与 core 同口径」里，超出派发两处，主 agent 在 records 第 9 批只描述未判收退，代码里在。

### 3.5 覆盖表补第 68 号之后，覆盖断言能不能被新立而 GPU 没判的不变量绕过

覆盖断言只有两道编译期断言：长度等于 `IMPLEMENTED_INVARIANTS` 的长度 「"池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表"」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:79`），与下标升序 「POOL_CHECKER_COVERED_INVARIANT_INDEXES[position] == position,」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:84`）。它只证明「登记了」，不证明内核里真有置位的代码：有人新立一条不变量、把下标补进表而内核不置位，两道断言都过；这时只有对拍里「CPU 红而 GPU 不红」才抓得到，前提是对拍的某条流上 CPU 真红过那一条。68 号这一条有 2 条流 + 2 条变异行钉（3.2），补表那一步本身没有变异行（9-9 报告「没写变异行的」一节：少了它编译期断言编不过，不是测试红）。**结论：能绕过**；绕过后的防线是对拍流 + 变异行，不是这两道断言。这是设计，不是这一批的退步（同一形在改回全覆盖之前就是这样）。

### 3.6 线索一：GPU 走读不走种类 9、10、11，而回退那一发布今天写了种类 11 的单元（冲突，推的，没跑）

- GPU：「// key 形态：1 extent、2 inode、3 分配记录、4 记账、5 映射；别的种类不走」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2745`），之后 「if (schema == NONE) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2754`） 直接返回，根指针不读、不记引用；GPU 记引用的入口一共四处（`w_note_pointer_references` 的四个调用：索引节点 「w_note_pointer_references(p, 1u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1477`）、实例表链、数据单元、inode 树根下的容器），没有一处是按种类 9–11 的根去调的。仓里 `wgsl` 里与谱系有关的只有两行（命令原样输出）：

```text
$ grep -n -i "lineage\|谱系" crates/singlefs-checker-tier/src/crash_judge_checker.wgsl crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl crates/singlefs-checker-tier/src/crash_judge_common.wgsl crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:156:const TREE_KIND_LINEAGE: u32 = 11u;
crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1719:            || kind == TREE_KIND_SPARSE_SIDE_TABLE || kind == TREE_KIND_DEADLIST || kind == TREE_KIND_LINEAGE
$ grep -n "TREE_KIND_LINEAGE\|TREE_KIND_TOMBSTONE\|TREE_KIND_DELETED_INODES" crates/singlefs-checker/src/walk.rs | head -12
71:const TREE_KIND_DELETED_INODES: u16 = 9;
72:const TREE_KIND_TOMBSTONE: u16 = 10;
73:const TREE_KIND_LINEAGE: u16 = 11;
86:    TREE_KIND_DELETED_INODES,
87:    TREE_KIND_TOMBSTONE,
88:    TREE_KIND_LINEAGE,
213:        TREE_KIND_DELETED_INODES => Some(DELETED_INODES_ENTRY_BYTES),
215:        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE => Some(inode_internal_entry_bytes()),
386:    [TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_DELETED_INODES];
1933:        if kind == TREE_KIND_TOMBSTONE {
1942:        if kind == TREE_KIND_LINEAGE {
3433:            TREE_KIND_EXTENT | TREE_KIND_DELETED_INODES | TREE_KIND_TOMBSTONE => {
```

- CPU：种类 9、10、11 都有形态 「10 | 11 => Some(KEY_SCHEMA_CONTAINER_NUMBER),」（`crates/singlefs-checker/src/lib.rs:601`），走读里 「if kind == TREE_KIND_LINEAGE {」（`crates/singlefs-checker/src/walk.rs:1942`） 走到每片容器；读单元那一步记引用 「self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, what);」（`crates/singlefs-checker/src/walk.rs:1253`）。
- 为什么今天会走到：回退那次发布追加谱系记录，第一次追加在零根的谱系树上写出容器与谱系树根，D18 「5. 谱系记录」（`.claude/kb/decisions/18-块里携带什么信息.md:573`） 第 5 条；9-8 二分实测只退掉这一处 D 那一段就从 481 回到 477（`/tmp/claude-1000/m3-r1/batch9-9-8/patch/report.md` 第三节，主 agent 现量也是 481，records 第 9 批「9-8 现量」）；钉回退那一发布共 29 次写的用例里 「("roll_back_by_a_forward_publish", 29)」（`crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs:153`）。GPU 对拍的回退流 `record_fixed_script_through_rollback` 就调这条回退：「roll_back_step(」（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:423`）。
- 推到的差别：I-3.1 的「遍历」数的是走读记下的引用 「let walked = shl64(add64(add64(c_slots_referenced(device, w_reference_count[0]), u64_of(exempted_all[d])), u64_of(c_deferred_per_device[d])), SLOT_SHIFT);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1620`），CPU 同一格 「judgements.judge("I-3.1", allocated == Some(walked), || {」（`crates/singlefs-checker/src/walk.rs:8514`） 的 `walked` 含谱系单元；记账的已分配含它们（它们是真分配）。所以 D 的根落盘之后的状态上，GPU 数少三个槽（谱系树根 1 + 容器 2，每块盘各一份）、报 I-3.1 违例，CPU 判成立。I-3.11（已分配减 defer 等于最新根走读）同一个量。I-5.1、I-3.5 上 GPU 只会少判（走不到的树里的单元不进引用），不会误红，所以对拍里露出来的会是 I-3.1 / I-3.11 这类按数量比的条。9-3 报告 S2 写的「今天的对拍流里没有这三种树（推的，没逐条流查）」：「今天的对拍流里没有这三种树（推的，没逐条流查）」（`/tmp/claude-1000/m3-r1/batch9-gpu/patch/report.md:82`），与上面三点对不上——回退流一直带着 D，而 D 今天写谱系。这条在 records 第 9 批里的去向是「进第 9 批收拢表判去向」（records 第 9 批「9-3 GPU 簇交回」），而第 9 批收拢表不存在（见 K6）；`checks-owed.md` 里没有行（C676 只管 I-9.16 ② 的七种）。
- 复核法（主 agent）：带 `gpu` 特性跑 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 的 `the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows`，看回退流那一行 `mismatches=` 是不是 0。主 agent 眼下那趟快档不带 `gpu`，看不到它。
- **推翻**：回退流 `mismatches=0`（那就是我漏了某处把种类 11 的单元记进引用）；或 `w_note_pointer_references` 另有一条不经 `w_read_index_node` 的路读到了谱系树根。

### 3.7 线索二：层 0 新池新建文件那条流的逐条核对没有 I-9.28 的项（推的，没跑）

`crash_enumeration_new_pool_file_creation_stream.rs` 的 `assert_checker_counts` 对 `IMPLEMENTED_INVARIANTS` 每条算 `expected_evaluated`：「let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8", "I-7.9"];」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:157`） 里的是 0，别的归 `only_under_the_new_root` 等几类，**都不在的落到最后一臂 `every_state`（每个状态都评估）**。I-9.28 不在任何一类里（`grep -n "I-9.28" crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 零命中，9 份补丁里这份文件没有改动）。而同一条路径（mkfs、挂载、第一个文件）上 I-9.28 恒报不适用：harness 的登记表 「["I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28"];」（`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:666`），理由是别的根都指着空树表、树表条目只出现在一片树表里。于是这条流上 I-9.28 的「评估过」为 0、预期是全部状态：快档用例 `layer0_quick_tier_matches_the_full_tally_shape` 会红在 `{invariant} 评估过的状态数` 那一句（推的）。9-6 报告第八节写过「I-9.28 多半与它同报不适用或同成立」，「I-9.28 多半与它同报不适用或同成立」（`/tmp/claude-1000/m3-r1/batch9-checker/patch/report.md:207`），没有人去改这份文件，records 与 checks-owed 里也没有去向。
**但要注意**：同一个循环里 I-3.12、I-5.5、I-9.27 这些更早新立的不变量同样没有项（文件头注释还写「其余 23 条每个状态都评估」：「其余 23 条每个状态都评估。」（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:106`）），所以这条用例也可能早在这一批之前就红着。两种情况的判别：主 agent 的快档结果里这一条红了，红的消息点名哪几条不变量。**推翻**：快档里这一条绿（那我对 I-9.28 在这条流上恒不适用的读法错了）。


## 四、K5　数能不能核（E）

算术与段序列我用脚本自己算了一遍（脚本在模型目录，只读冻结副本，不编译、不跑被测代码）。每个数从哪一段代码推出、钉它的测试与变异行、改回旧值红不红：

### 4.1 481 次写（固定脚本到 E 再卸载）

- 来历：回退 D 那一段的单元写 20 → 24，多出的 4 次是回退追加谱系记录写的两个单元（谱系容器 + 谱系树根）各两块盘一份。用例注释把 24 拆开：「回退 D 24（分配记录树 7 个节点 + 记账 + 映射 + 树表 = 10 个固定点单元，加谱系容器与谱系树根」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:395`）；总数式 「195 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:455`） = 195 + 165 + 44 + 33 + 44。我从用例里 78 段的段长数组（第 445–450 行）直接加：

```text
segments 78 writes 481
two_state_closed_form 1678377089
segments_of_size_2 42
three_state_if_all_size2_are_sysconf_pairs 1678377299
three_state_if_23_pairs 1678377204
to_D_segments 36 writes 195 closed 67502143
```

- 为什么是谱系：D18 「5. 谱系记录」（`.claude/kb/decisions/18-块里携带什么信息.md:573`）；实现员二分（只把 HEAD 上追加谱系记录那一处换成不追加就回到 477，`/tmp/claude-1000/m3-r1/batch9-9-8/patch/report.md` 第三节表）：「HEAD 上只退掉谱系记录这一处就回到 477」（`/tmp/claude-1000/m3-r1/batch9-9-8/patch/report.md:45`）；改回不追加的探针（9-8b 报告第三节）段序列退回 20、写数退回 477。
- 钉：`prepare` 里按脚本臂的段序列与写数断言（快档、不带 `ignore`）；变异行 「回退那次发布不追加谱系记录（D 那一段的单元写退回 20，到 D 的段序列断言红）」（`crates/mutations.tsv:2471`） 点名 「fn the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_sequence() {」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:640`），改回旧值（20 / 477）它在 `prepare` 的段序列断言上红。变异行锚点我逐行核过（`anchors_check.py`，下面）。
- **仓里没有东西钉的一块**：谱系两个单元的槽号 50352、50354 与「容器 32768 字节 / 树根 16384 字节」只出自实现员的探针（9-8 报告第二节表），records 第 9 批「写回戊交回」已写「仓内没有钉」；这不影响 481（481 钉的是写数与段长，不是槽号）。

### 4.2 两态闭式 1678377089、三态全量 1678377204、按发布分

- 两态闭式 = 1 + Σ(2^n − 1)：脚本 `k5_closed_form.py` 对 78 段数组算出 1678377089（上面输出第 3 行）= 旧 1662648449 + 2^24 − 2^20（用例注释 「D 的单元写 20 → 24，那一段 2^20 − 1 → 2^24 − 1、多 15728640：1662648449 → 1678377089，段数不变。」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:97`））。
- 三态 = 两态 + 23 × 5：用例注释 「每段 3² − 1 比两态时的 2² − 1 多 5：1678377089 + 23 × 5 = 1678377204」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:101`）。23 这个数我另核：78 段里大小为 2 的有 42 段，按「每次发布一个记录段 + 系统配置原地写两两一段」拆，19 次发布 + 取号 2 + 抬 F 1 + 卸载 1 = 23 对系统配置写（`k5_arith.py` 末两行，推的、与注释一致）。
- 按发布分：脚本 `k5_by_publish.py` 把 `FULL_STATES_BY_PUBLISH_THROUGH_THE_UNMOUNT` 21 格相加 = 1678377204，`QUICK_TIER_…` 21 格相加 = 278，`2^n + 11`（n = 24）= 16777227 = `instance2_txg9` 那一格：

```text
full by-publish keys 21 sum 1678377204
quick by-publish keys 21 sum 278
16 65547
24 16777227
28 268435467
txg9 expected 16777227 got 16777227
```

- **钉的强弱**：到 D 为止那一份在快档里钉着 「assert_eq!(closed_form_state_count(&prepared.segments), 67_502_143);」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:643`） 与 「(67_502_198, 132)」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:663`），改回旧值（51_773_503 / 51_773_558）快档就红。到 E 再卸载那一份的 1678377089 与 1678377204 **只在 `#[ignore]` 的全量用例里比**（「const FULL_STATES_WITH_TWO_STATES_PER_WRITE_THROUGH_THE_UNMOUNT: u64 = 1_678_377_089;」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:98`）、「const FULL_STATES_THROUGH_THE_UNMOUNT: u64 = 1_678_377_204;」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:103`） 只被那条用例引用）：改回旧值，快档不红，只有门禁 checker-tier-layer0-replay 的全量那一趟红；快档里能守住它们的只有 `prepare` 的段长数组（两态闭式是它的函数，上面脚本为证）。三态数没有快档下的第二条路；`e170`、`e171` 里同值的常量只是互相抄（`assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_678_377_089)` 是常量对字面量）。
- 同一条 `#[ignore]` 的理由串还写着 「#[ignore = "全量五十多亿个状态、每个两遍恢复 + checker；门禁 checker-tier-layer0-replay 在 release 下跑，带断点续跑"]」（`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:805`），今天是 16.8 亿；9-8 报告第六节第 5 条报了，没人改，records 与 checks-owed 里没去向（K6）。
- 装置里的旧值：「const SECOND_STREAM_WRITES: usize = 477;」（`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:578`） 没跟（records 第 9 批「9-8 交回」写了归 E161 主人）；它不钉 481，是装置自己的常量。
- 变异表锚点（静态）：`anchors_check.py` 把 `crates/mutations.tsv` 2466 行的原文逐行在文件里数命中次数：

```text
rows 2466 bad 1
(1355, 'nofile', 'C577 结清 ②：litmus「轮换之后屏障」的 P0 去掉轮换之后那道屏障，与录制流里发布最后三步对不上')
```

  唯一一行是 litmus 目录下的行（不在 `crates/` 副本里，不是这一批）。`targets_check.py` 再核每行 `--test` 目标文件与「必须红的测试名」在不在：2466 行里 11 行说「必须红的测试名不在目标文件里」（行 86–88、112、113、192、295、480、1240、1241、2014），**没有一行落在第 9 批的 2298 号之后**（原样输出在 `research/prompts/m3-root-ring-code-r3-sonnet-model/results/targets.out`）。

### 4.3 点名项上限 64 与叶罩 144 的取样点

- 来历：「pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;」（`crates/singlefs-format/src/lib.rs:229`），注释写 「/// 一条 4 KiB 记录装几个点名项：(4096 − 487) ÷ 56 的整数部分」（`crates/singlefs-format/src/lib.rs:228`）；格式库自己的单测把它与式子绑着 「(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES,」（`crates/singlefs-format/src/lib.rs:548`）。我算：

```text
(4096-487)//56 = 64  487+64*56 = 4071  487+65*56 = 4127 (>4096)
(16384-163)//112 = 144
scratch bytes per state = 1160256  2147483644//that = 1850  12GiB//that = 11105
segments 78 size-2 segments 42  = 19 record segments + 23 system-configuration pairs -> 42
system configuration writes: acquire x2, raise-F x1, unmount x1, rotation after each of 19 publishes -> 46
```

  （前两行是本节的算式；第三行 4.5 用；后两行 4.2 用。）
- 取样点：「for expected_unit_count in [63u64, 64, 65, 143, 144, 145] {」（`crates/singlefs-core/src/write_request_split.rs:220`）；用例前的两条 `assert_eq!` 把取样点与两个常量绑住：「singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD,」（`crates/singlefs-core/src/write_request_split.rs:210`）；顺序写那份 「JOURNAL_NAMED_ENTRIES_PER_RECORD, 64,」（`crates/singlefs-harness/tests/sequential_write_one_record_per_data_unit.rs:171`） 与 「JOURNAL_NAMED_ENTRIES_PER_RECORD + 1,」（`crates/singlefs-harness/tests/sequential_write_one_record_per_data_unit.rs:181`）。被测代码不读这个常量，所以这几条断言是「钉取样点」不是「钉行为」：实现员自己写明（点名项门槛簇报告第一节第 2 点），并给了删法。行为那一半仍由 `transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction` 这类用例带着：该簇 `prove-red` 表第一行「core lib 182 过 2 红」把常量改成 63 时它也红（`/tmp/claude-1000/m3-r1/batch9-named-limit/patch-without-e171/report.md` 第三节表）。
- 变异行 「切分取样点钉门槛：一条 journal 记录的点名项上限改成 63（取样点 63 / 64 / 65 不再压在上限上）」（`crates/mutations.tsv:2466`）、「顺序写钉门槛：一条 journal 记录的点名项上限改成 65（64 个单元那条用例不再压在上限上）」（`crates/mutations.tsv:2468`） 等五行，改回旧值（取样点贴 67）该簇 14 行 `prove-red` 全抓到；我没重跑。冻结副本里旧取样点已清：`grep -rn "sixty_seven\|\[66u64\|66, 67, 68" crates --include=*.rs` 零命中（只剩 `admission.rs` 里一个名字带 368 的用例和 GPU 覆盖表的下标 67、68，不相干）。
- 一处不是钉的注释：`multi_record_publishes_of_transaction_zero.rs` 第 5–8 行的算式 487 + 64 × 56 = 4071、第 65 项落在 4071..4127 越过 4096，我算对（上面输出第一行）。

### 4.4 `root_slot_already_unreadable_at_mount…` 改钉后的值

- 值：「(CheckpointTxg(0), CheckpointTxg(0)),」（`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:2395`）（`FloorAlreadyAtTheCeiling { floor: 0, ceiling: 0 }`）。推导（用例文档注释写了，我按代码对）：mkfs txg 0、暖机 1、2、第一个文件 3，进程退出后 (区域 0, 槽 0) 读不出；新实例 first_txg = 4。那一槽是 txg 0 的槽（`region: checkpoint_txg.0 % ROOT_RING_REGIONS`、`slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % S`，K2 引过），窗口里只能住 txg 0（下一次轮到它是 txg 3S），所以 `highest_txg_it_can_hold` = 0；读得出的根里最旧的有效根是暖机那条 txg 1，上限 1；`min(1, 0) = 0`，不高于现行 F 0 ⇒ `FloorAlreadyAtTheCeiling { 0, 0 }`（K2 引的 「ceiling_from_the_roots_read.min(highest_txg_it_can_hold)」（`crates/singlefs-core/src/mount.rs:2329`））。
- 钉：这条用例本身；变异行 「第9批改钉 RS3（合入后改钉的那一条）：挂载那一刻读不出的槽不封顶准入抬 F 的上限（上限取读得出的根算出的 1、F 抬到 1）」（`crates/mutations.tsv:2371`）：改回旧值（不封顶，上限 1、F 抬到 1）这条红在 `admission_raises_the_floor_before_refusing.rs:2391`，原样日志 `/tmp/claude-1000/m3-r1/batch9-repin/prove/round2-p4.out`、`round2-p4-logs/001.log`（我读了，没重跑）。
- 另一个旧值同族：改之前那条用例（读不出当没有根）的老变异行名 `实五 445：挂载那一刻就读不出的槽也重读、仍坏就拒（挡住了条款说当没有根的那一格）` 只换了点名的用例名，`targets_check.py` 核过目标文件与测试名都在。

### 4.5 只在临时探针上核过、仓里没东西钉的数（逐个）

| 数 | 出处 | 仓里钉着吗 |
|---|---|---|
| 谱系两单元槽号 50352、50354 | 9-8 探针 | 没有；不影响 481 |
| 1678377204（三态全量） | 实现员 harness 探针按 3^m·2^(n−m)−1 自算 | 只在 `#[ignore]` 全量里（4.2） |
| 派活状态数 1850、11105 | 9-3 现算、9-9 复核 | 钉着：「assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1850);」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:882`）、「assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 11_105);」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:889`）；我算 2147483644 ÷ (290064 × 4) 取整 = 1850、12 GiB ÷ 同 = 11105（上面 `k5_arith.py` 输出第三行），`SCRATCH_WORDS` 是 「pub const SCRATCH_WORDS: usize = 290_064;」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:116`） |
| 三个内核展开后的表达式数（290 085 / 268 616 / 342 906 对 40 万预算） | 9-9 的 naga 小程序 | 钉着预算而不是这几个值（`every_kernel_stays_within_the_inlined_size_budget`，checker 档，没跑） |
| 随机历史 230 / 150 段 0 红 | 改钉簇 | 产物在仓里（K3 2.1），装置不在改钉簇之后的仓里（它交回前删了 `fz-proto/`），复跑要照报告重建 |
| 全部「新用例证红」 | 实现员 `prove-red` | 日志在 `/tmp`，没入库；checker 档与 GPU 的 20 行一行没证 |


## 五、K6　欠账与未定项清单全不全

读法：九份实现员报告各自的「取了读法的」「做不出会红用例的」「要主 agent 判的」每一条，逐条去 `.claude/kb/checks-owed.md`、`records/2026-09-30-里程碑三开工第一轮.md`、kb 条款里现查去向。记法：`R:n` = records 那份第 n 行，`C…` = checks-owed 的欠账号；报告路径简写成 `/tmp/…/batchX`（都在 `/tmp/claude-1000/m3-r1/` 下，节号写 §）。判：✓ 有去向且去向对得上；△ 有去向但不全（缺 kb / C 行、或指向的容器不存在）；✗ 无去向；— 不需要去向。

### 5.1 核心簇、GPU 簇、池级 checker 簇

| 号 | 来源 | 条 | 去向（现查） | 判 |
|---|---|---|---|---|
| 01 | core2 §7.1 | `mount.rs` 写行零单元记根（q2）走不到重读，没写用例、没挂变异行 | R:541「走不到、收下」；C 行无 | △ 收下、没登行（走不到所以没有可钉） |
| 02 | core2 §7.2 | `transaction.rs` 写行记根（q3）做不出会红的用例 | R:541 + C702 | ✓ |
| 03 | core2 §7.3 | `rebuilt_allocator` 里没人读的那句赋值 | R:541「照代码纪律删」→ 改钉簇删（R:550）；冻结副本里 `note_the_instance_table_judging_the_rereads_of_root_ring_slots(` 只剩 `mount.rs:4486` 一处调用（K2 1.2 的 grep） | ✓ 已做 |
| 04 | core2 §7.4 | R2「某块盘一份自证过的都没有 ⇒ 按真」照实现已写的成员 | R:541 收下；D22 第 178 行、C664、C707 | ✓ |
| 05 | core2 §7.5 | RS3 并不并「自证不过」，D16 两行字面不一样 | R:541 交用户 → R:544 用户 Q5「只并读不出的槽」；D16 第 37、38 行已对齐（K2 1.3） | ✓ |
| 06 | core2 §7.6 | 清扫在树表 0 条那一版上「第一版不做」（成员 `SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion`，一个头都不抹） | R:541（第 6–8 条并说「收下」）；D18 清扫那一条（第 314 行）没有这一格；C 行无 | △ 无 kb / C 登记 |
| 07 | core2 §7.7 | 清扫的块设备错两支没写条款、没用例 | C703 | ✓ |
| 08 | core2 §7.8 | 清扫自取两格：根环里可读根的账读不出就不删；池里有写序实例大于挂载根的单元就不删 | R:541 收下。前一格落在 D18 第 314 行「那次清扫里没有任何一次读失败」的字面里；后一格来自判决 `m3-sweep-and-tree-id-r1-main-verification.md` 第 44 行「条款没写，正推给保守读法」，D18 与 C 行里都没有 | △（后一格条款缺、无 C 行；K1 要攻的正是它会不会让一行永远删不掉） |
| 09 | core2 §7.9 | 被抛弃的根所在的槽挂载那一刻读不出 ⇒ I-7.4 红，已知一形 | R:541 + C554（长行末段写明） | ✓ |
| 10 | core2 §7.10 | 挂载透明不认清扫发布 | C704 | ✓ |
| 11 | core2 §7.11 | RS2 用例放在文件名只说 RS1 的文件里 | 改钉簇改名（R:550）；冻结副本里文件已是 `root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs` | ✓ |
| 12 | core2 §5 | core 库单测 3 条（`instance_table_sweep.rs`）没有自己的变异行 | 无（`grep -c "库单测"` records 0 / checks-owed 0） | ✗ |
| 13 | core2 §5 | checker 档 3 行变异与整行替换的 5 行没证红 | R:541 只写「证红 20 行抓 19」与两条 `crash-case:` 登记行，没提这 8 行；它们只能留给提交时 `checker-tier-crates-mutation-replay`，没有记录点名 | △ |
| 14 | core2 §3 | C670 的注释改了、断言没动，措辞要形式化线改 | R:541「已发形式化接入那条线」 | ✓ |
| 15 | core2 §4 | 阈值占位 369；后台那一支只给了入口、没调度 | C690、C597（后者写明「仓里没有后台调度器」） | ✓ |
| 16 | core2 §8 | 两条 `crash-case:` 登记行 | `.claude/gate.d/stage-inputs.tsv` 第 47、48 行（R:545） | ✓ |
| 17 | gpu §4 S1 | 两处说法变旧：`image.rs` 注释与 `invariants.md` 第 16 行 | **两处都没改**：`invariants.md:16` 仍写 「下标 65 的 I-3.5（引用区间的精确性） GPU 内核还没判、那一位恒 0」（`.claude/kb/invariants.md:16`） 与「列 68 个下标」；冻结副本 `image.rs:76`、`:82` 仍写「GPU 判器内核还没判它」，而 `crash_judge_gpu.rs` 已写「没有「内核还没判」那一张表」、`POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET` 在 `crates/` 里零命中。R:538 写「进第 9 批收拢表」 | ✗ |
| 18 | gpu §4 S2 | GPU 走读不走种类 9、10、11 | R:538「进第 9 批收拢表判去向」，该表不存在（5.3）；C 行无（C676 只管 I-9.16 ② 的七种）。本轮 K4 3.6 给了它今天会露出来的形 | ✗ |
| 19 | gpu §4 S3 | 共享树表那一支 GPU 不逐版再判 I-9.25 | 同 18 | ✗ |
| 20 | gpu §4 S4 | `parse_record` 四条指针头都判全 extent 偏移，超出派发两处，报告写了退法 | R:538 只描述「S4 写了退法」，没写收退；代码里留着，与 core `judged()` 同口径（K4 3.4） | △ 隐含收下 |
| 21 | gpu §4 S6 | 派活单测钉的状态数本来就过时 | 已在 9-3 补丁里改为 1850 / 11105（K5 4.5） | ✓ |
| 22 | gpu §4 S7 | gap-row、N3、共享树表那一支 GPU 侧没有会红的流 | R:538「进收拢表」；同 18 | ✗ |
| 23 | gpu §4 S5、S8 | 变异写法照实说明；恢复内核里新取的两个字段没读者 | 报告内已写明是等价变异，不需要去向 | — |
| 24 | gpu §5、§8 | 12 行变异没证红、对拍没跑 | R:538「留给提交前的 GPU 对拍与变异行逐个 --item」 | ✓ |
| 25 | checker §6 Q1 | I-7.9 窗口取法 | 用户定乙（R:540）→ 改钉簇（R:550）→ kb I-7.9 行（K3 2.1） | ✓ |
| 26 | checker §6 Q2 | I-9.28 什么时候报不适用 | R:540 用户定 + 第四种判定；kb I-9.28 行写四种 | ✓ |
| 27 | checker §6 Q3 | 认领本身判红那半句没做 | R:539 主 agent 判「算做成」 | ✓ |
| 28 | checker §6 Q4 | 同 txg 并组造不出合法前提下的坏镜像 | R:539「收下不钉」；C 行无 | △ 决定了不钉 |
| 29 | checker §3 第 2 条 | 认领规则留的缝：释放了的两槽数据单元，第一槽被一槽的码 2 节点拿走、第二槽旧载荷里的伪头照数；反向缝只会漏红 | 判决 「剩下那条缝记进 C89（单元头收口的遗留重算） 第 ② 笔」（`research/prompts/m3-sweep-and-tree-id-r1-main-verification.md:75`），代码注释也指向它（「认领规则留的缝（判决同一行「F4 只修一半」，C89（单元头收口的遗留重算） 第 ② 笔）」（`crates/singlefs-checker/src/walk.rs:3186`））；C89 ② 现文里没有这条缝（`grep` 「旧载荷」「释放了的」「一槽的码 2」在 C89 行零命中） | ✗ 指针悬空 |
| 30 | checker §3 第 3 条 | `…reddens_only_its_own_invariant…` 那条用例的期望已改成两条，名字没改，交主 agent 定 | 无；冻结副本 `checker_known_bad_images.rs:2579` 名字原样 | ✗ |
| 31 | checker §10 | kb 一处没写（I-9.28 状态、I-7.8 认领、I-3.5 钉、判 69 条） | 写回丙（R:554）：`invariants.md` 第 12 行写 69、I-9.28 行写「已实现」 | ✓ |
| 32 | checker §8 | I-9.28 新立后层 0 各条流的逐条登记要补一格（十余份 `crash_enumeration_*`） | 无。9 份补丁里只有 `crash_enumeration_fixed_script_stream.rs`（9-8）动了；`new_pool_file_creation_stream` 没有 I-9.28 项（K4 3.7） | ✗ |
| 33 | checker §4、gpu2 §1 | clippy 没带 `--all-features`；69 条下 GPU 覆盖断言编不过 | R:543、R:545、R:551（9-9 补 68，合入验证过） | ✓ |


### 5.2 改钉簇、9-9、点名项门槛簇、9-8、9-8b

| 号 | 来源 | 条 | 去向（现查） | 判 |
|---|---|---|---|---|
| 34 | repin §6 读法 2 | 层 0 计数把「部分没判」并进「不适用」那一格 | R:550「它取的读法，进代码轮第三轮攻」= 本轮（K3 2.3） | ✓ |
| 35 | repin §6 读法 1、3、4、5、7、8 | `into_report` 次序（部分没判压过「射程有一部分判不了」）；一棵树都没比到仍报部分没判；说明里的盘号取根槽区域的设备；带记号支只取窗口、不加「按实例表有效」与「txg ≥ 抬之前的 F」；`bad_disk_input` 按判过；`e156` 加标签 | 报告里；R:550 只点了读法 2；kb I-9.28 行写了「各处都对得上，但…→ 部分没判」（读法 3 的字面）；其余在 /tmp 报告里 | △ |
| 36 | repin §6 读法 6 | 带记号支判红的说明文字没改（仍写「根环里 txg 比它小的最新那条根」，现在是两个窗口的交里最新那条） | 无；旧文字被一条用例逐字钉着：「实例 1 txg 7 那条带卸载记号的根把回退下界 F」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:4543`） | ✗（小） |
| 37 | repin §7 | I-3.5 整对不判、I-9.14 读不出树表的根要不要也改报部分没判 | C701（R:540 写「下一轮判」） | ✓ |
| 38 | repin §3 末 | history / fault_injection / crash_injection 的 `invariant_partially_judged`、层 0 并进不适用、`bad_disk_input` 判过、`e156` 标签，这几臂没有会红的用例 | 无（records、checks-owed 里 grep「没有会红的用例」与这几处的名字零命中） | ✗ |
| 39 | repin §4 | `crash_identity.rs` 判法版本要不要动、库里旧结论要不要重判 | **不需要登记**：CPU 判法摘要由闭包源码现算（「判法改了每个点的复用键都变、树不动」（`crates/singlefs-checker-tier/src/crash_amplification.rs:645`）），GPU 的 `gpu_judge_digest` 把五份内核源码都收进摘要（「message.extend_from_slice(GPU_JUDGE_CHECKER_SCAN_SOURCE.as_bytes());」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:173`））。（我读码核的；CPU 闭包是否一路收到 `singlefs-checker` 的 `walk.rs` 没逐文件核） | ✓ |
| 40 | repin §3 | 7 行改名变异（RS1 / RS2）只换了 `--test`、没重证 | 无；我静态核了目标文件与测试名都在（K5 4.2 的 `targets_check.py`） | △ |
| 41 | repin §5 | 带 gpu 特性的两份 `gpu_judge_*.rs` 改后没编过 | R:551「clippy `--all-targets --all-features` 过」 | ✓ |
| 42 | repin §1 | 随机历史两批输出不在仓里 | R:554 拷进 `research/prompts/m3-r1-batch9-repin-evidence/window-fuzz/`（K3 2.1 我按它现数） | ✓ |
| 43 | repin §7 | invariants.md 的 I-7.9、I-9.28、文件头第四种判定要写回 | 写回丙（R:554）；文件头已写「部分没判」（`invariants.md` 第 6 行） | ✓ |
| 44 | gpu2 §4 Q1 | GPU 三处没有一圈窗口 | C705；R:549 ① | ✓ |
| 45 | gpu2 §4 Q2 | 带记号支窗口里没有之前的根 ⇒ GPU 取「不判」 | R:549 ②；CPU 同取（K3 2.1、K4 3.4） | ✓ |
| 46 | gpu2 §4 Q3 | 没写「CPU 真报了部分没判」那句断言 | R:549 ③「进代码轮第三轮」= 本轮（K4 3.2）；断言仍没写 | ✓ 去向对，事没做 |
| 47 | gpu2 §4 Q4 | 手造根的流只比逐位相同 | R:549 ④ 收下 | ✓ |
| 48 | gpu2 §4 Q5 | naga 小程序一次没经 `capped.sh` | R:549 ⑤ 记这一条 | ✓ |
| 49 | gpu2 §2 | 派发清单外多改 `crash_judge_checker_scan.wgsl` | R:549 收下 | ✓ |
| 50 | named §1 | 取样点钉值断言是新加的（被测代码不读常量） | R:567 收下 | ✓ |
| 51 | named §6 | `scripts/hosts/memory-peaks.tsv` 里记着旧用例名作键 | 无（records / checks-owed 里 `memory-peaks` 零命中）；该表在 `research/`，实现员没权改 | ✗（小） |
| 52 | named §4 | clippy 红在 `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:181-182`（`shadow_unrelated`），不在改动里 | R:567「提交时分诊」 | ✓ |
| 53 | named §1 | 六行改名走「删旧名 + 追加新整行」 | R:567 | ✓ |
| 54 | 9-8 §1 | 多出的 4 次写来自回退追加谱系记录，合条款 | R:568、R:560 | ✓ |
| 55 | 9-8 §6、9-8b | 两份崩溃放量用例同一原因会红 | 9-8b 做（R:570） | ✓ |
| 56 | 9-8 §6.3 | e161 / e170 / e171 里的 477 | R:568（e161「归 E161 主人，进下一轮清单」）、R:558（e170、e171 归形式化线）；冻结副本 `e161…rs:578` 仍是 477 | △ 「下一轮清单」不存在（5.3） |
| 57 | 9-8 §6.4 | `layout/01-first-txn.md` 第 412、420 行、segment-registry 格 | 写回戊（R:582） | ✓ |
| 58 | 9-8 §6.5 | `#[ignore]` 理由「全量五十多亿个状态」已不对（今天 16.8 亿） | 无；冻结副本里原样（K5 4.2） | ✗（小） |
| 59 | 9-8 §6.6、9-8b | 变异行「崩溃放量 步数不按连续同种段计」的红不能作数 | 9-8b 做；R:570 | ✓ |
| 60 | 9-8b §3 | 新变异落在 `transaction.rs`，不落在 `mount.rs` | R:570 收下 | ✓ |
| 61 | 9-8b §5 | 只开 `verdict-store` 不开 gpu 时 `crash_amplification.rs:41`、`:2384` 的 dead-code 报错 | R:570「提交时分诊」 | ✓ |
| 62 | 9-8b §2、§7 | 探针里的崩溃点命名是照抄的一份，不是 `into_crash_points` 本身 | 无；9-8b 报告第六节要主 agent 跑那两条用例才见分晓 | △ |
| 63 | 9-8、R:582 | 谱系两单元槽号只出自探针、仓里没有钉 | R:582「留下一轮的」 | △ 「下一轮清单」不存在 |

### 5.3 两处共同的底

- **「第 9 批收拢表」「下一轮清单」不存在。** R:538 对 S1、S2、S3、S6、S7 写「进第 9 批收拢表判去向」，R:568、R:582、R:549 等多处写「进下一轮清单」。`grep -n '^### ' records/2026-09-30-里程碑三开工第一轮.md | tail -8` 的后四行依次是第 6 批收拢（第 416 行）、第 7 批（第 434 行）、第 8 批（第 472 行）、第 9 批截止批次（第 518 行）：第 7、8 批都没有收拢小节，第 9 批也没有，而第 9 批又是截止批次。所以这些条的去向今天只是一句散文，没有一张能被核「清不清完」的表。正文第二节「已知没做」把 C705 那一类写进了，但下面 12 条「✗」没有去向：12、17、18、19、22、29、30、32、36、38、51、58（其中 18、19、22 就是 R:538 的 S2、S3、S7）。
- **九份实现员报告的存档。** 它们都在 `/tmp/claude-1000/m3-r1/…`；仓里有拷贝的只有三份（9-5、9-3、9-6，在别的一轮的快照目录里：`research/prompts/m3-batch14-code-r1-snapshot/peer-root-ring-batch9-core2-patch-report.md`、`…-gpu-patch-report.md`、`…-checker-patch-report.md`）。改钉簇、9-9、点名项门槛簇、9-8、9-8b 与 `patch-item1` 的报告没有任何仓内副本（我用报告标题 `grep -rlF` 全仓现查，上面三份以外零命中）。C702–C704 的「出处」栏已经写了「会话草稿，没入库」；/tmp 一清，5.2 里 35、36、38、40、62 这些「只在报告里」的读法就没有原文可查了。
- 统计（脚本数 `out-5.md`、`out-6.md` 两张表的末列）：63 行，✓ 39、△ 11、✗ 12、— 1。


## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2 RS1 三种结局 | 一致 | D16 第 35 行三句对 `mount.rs` 重读分类与 `allocator.rs` 改表逐项相同；不安全方向只有「拿掉真有效根」一条路，按定义走不通 |
| K2 RS2 按哪张表、四处记根 | 一致 | 表在写行那次发布后换上；旧表只会多留被抛弃的根（安全方向，但记账比 checker 遍历多、I-3.1 红）；`mount.rs:4686` 与 `transaction.rs:1757` 两处记根点没有会红的用例，已知（C702 只登了第二处） |
| K2 RS3 只并读不出的槽 | 一致，线索一条 | 没找到 F 越过真有效根、也没找到永久挡住（挡不过一圈窗口）；旧签名三处调用者与参考模型都不带封顶 |
| K2 `mount_transparency.rs` 照 R2 | 一致 | 与 core 同谓词、同读槽、同 fsid 过滤；只差「某块盘一份都没有」在 core 是单列成员、harness 并在一个函数里 |
| K3 I-7.9 两支两窗口、交空不判 | 一致 | 第二个窗口在无洞环上什么也不滤，没找到新增漏红类；带记号那条用例只断言「不许红」，钉得弱 |
| K3 I-9.28 四种结局 | 一致，一格钉缺 | 「条目比字段表短」条款有、代码有，用例与变异行都没有；合法历史上因旧根树表被复用会常报部分没判（不红、不掩盖违例） |
| K3 部分没判并进层 0「不适用」、全仓穷举 `match` | 一致 | 保守方向，没有一条流把它算成判过；穷举 `match` 编译期保证，22 份没提到它的文件都不是对判定的 `match` |
| K4 I-3.5、I-9.28、I-7.8 认领、I-7.9 窗口、RS5、RS6 | 一致（读码，没跑） | 逐要素并排对得上；GPU 侧 I-3.5 的 gap-row 与 N3 没有流、没有变异行 |
| K4 部分没判 / 不适用在 GPU 不置成立位 | 一致 | 对拍只比违例位；没有一形两边对不上 |
| K4 覆盖断言能否被绕过 | 能绕过（设计如此） | 两道编译期断言只证「登记了」；防线是对拍流 + 变异行 |
| K4 线索一：GPU 不走种类 9–11 | 冲突（线索，推的） | 回退发布今天写种类 11 的单元，GPU 数不到，I-3.1 / I-3.11 在 D 之后的状态上应与 CPU 分叉；要 gpu 特性跑对拍才见分晓 |
| K3 / K4 线索二：层 0 新池新建文件流没有 I-9.28 的项 | 线索（推的） | I-9.28 在该流上恒不适用，逐条核对循环却默认「每个状态评估」；这条用例也可能早已红 |
| K5 481 次写 | 一致 | 段长数组加出 481；谱系槽号只在探针里 |
| K5 两态闭式 1678377089 / 三态 1678377204 / 按发布分 | 一致，钉强弱不同 | 我算出同值；到 D 的数在快档钉着，到 E 再卸载的两个数只在 `#[ignore]` 全量里比 |
| K5 点名项上限 64 的取样点 | 一致 | 算式对；断言是「钉取样点」不是「钉行为」，行为由别的用例带着；旧取样点已清 |
| K5 `root_slot_already_unreadable…` 改钉值 { 0, 0 } | 一致 | `min(1, 0)`；改回不封顶红（改钉簇 `prove-red` 日志） |
| K6 | 63 行：✓ 39、△ 11、✗ 12、— 1 | 12 条无去向（含 GPU 不走种类 9–11、`invariants.md:16` 仍写 I-3.5 内核没判、认领缝的 C89 指针悬空）；两处散文去向（第 9 批收拢表、下一轮清单）不存在；六份实现员报告无仓内副本 |

## 七、留给主 agent 的复核（我复核不了的）

1. 线索一：带 `gpu` 特性跑 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 的 `the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows`，看回退流（`crash-judge-gpu-rollback`）`mismatches=`。复核不了的原因：要 GPU 与 checker 档特性，是重型测试，子 agent 不跑；主 agent 眼下那趟快档不带 `gpu`。
2. 线索二：主 agent 那趟快档里 `crash_enumeration_new_pool_file_creation_stream` 的 `layer0_quick_tier_matches_the_full_tally_shape` 红不红、红的话消息点名哪几条不变量。复核不了的原因：结果还没出，我不引。
3. 5.1 的 12 条 ✗ 要不要进 checks-owed、记在哪里。
4. K2 线索（`on_device_modes.rs:279` 旧签名、`model.rs:1291` 无封顶）：要不要登一条欠账；复核不了的原因：要造「挂载时读不出的槽 + 在盘抬 F」的盘面，没造。

## 八、没做什么

- 没编译、没跑任何测试（原因在第零节）；所有「抓到」「红了」都是实现员 `prove-red` 的输出或日志原文，我只读了几份（RS3 改钉那一行、I-9.28 那几行），没重跑。
- K1（清扫器）不归我，没推；K2 里与清扫有关的只到 K6 第 06、08 行为止。
- 没替主 agent 判采纳，也没出判决。第 6 节「判定」是我这一条正推腿的读法，不是判决。
- 没重推前两轮站住的 E2、B2、RR4、K3 R2 谓词、七类读者等；没再抽 K2、K3 第二次样（「没打中」要再抽一次才记，这一条属于主 agent 判）。
- 没验的：GPU 内核真卡上的行为、层 0 / 崩溃放量任何计数、变异表整表（我只静态核了锚点与目标文件）、第 9 批没证红的 30 行有没有红（GPU 簇 13 行含换掉的 1 行、9-9 的 8 行、核心簇 checker 档 3 行与整行替换 5 行、9-8b 的 1 行，都留给提交时 `checker-tier-crates-mutation-replay`）。
- 没读禁读清单里的文件：`research/prompts/m3-root-ring-code-r3-opus-output.md` 与 `…-opus-model/`、`research/prompts/m3-reclaim-window-r1-*-output.md` 与模型目录。
