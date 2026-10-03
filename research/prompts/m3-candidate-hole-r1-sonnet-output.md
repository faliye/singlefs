# m3-candidate-hole-r1 正推腿（Sonnet）报告：H1、H2、H3（2026-10-04）

<!-- doc-lint:not-numbers H1 H2 H3 E1 E2 B2 RS3 K1 K2 P1 J1 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 D1 D2 D3 D4 D5 D6 G1 G2 G3 G4 G5 G6 E-D1 E-D2 E-D3 E-D4 E-D5 E-D6 T-D1 T-D2 T-D3 T-D4 H1-F1 H1-F2 H1-F3 H1-F4 H1-F5 H1-F6 H1-F7 H1-r H1-j H1-b H1-a H1-c H1-d H1-e H2-a H2-b H2-c H2-d H2-e H3-a H3-b H3-c -->

读的是冻结副本 `/tmp/claude-1000/m3-candidate-hole-r1/tree/`（`crates/…` 的路径都指它）；kb 引文的行号是现查的 kb 文件自己的行号。
方案按 `crates/` 今天的实现来谈。立场：从已定条款与今天的实现逐句推，推得出的写候选，推不出的写明；不采纳、不判决。
引文写法：「原文」（`文件:行`）；代码引文是那一行的原文，kb 引文是那一行里的整句。调查员报告（`/tmp/claude-1000/m3-r1/batch8-investigate/report.md`，仓外）与主 agent 的观测只作转述，标「转述」。

## 〇、先说结论（候选与推不出的）

| 格 | 推得出的候选 | 推不出的 |
|---|---|---|
| H1 | 「取号跳过」认实例表行证据（今天 gap-row 的写法）；journal 里「已落、没施加」的记录不单独当证据（字面读法在当前实例上会把存在过的版本当成跳过）；行证据有一处没盖到的口子（R 所在实例自己更早的版本），可用 R 的根里实例表指针的出生代号补上（零轮，推的） | 「出生代号补口子」依赖「写行之后每一版照抄同一张实例表」，kb 自己写「没量」；journal 环被盖掉那一形下 journal 证据不可读，两个候选在那一形上的差别没造盘面量过 |
| H2 | 候选三个：整条不适用；E1 门槛区间；区间只加在「存在过却走不到」的那几代的单元上（γ，零轮，推的）。E1 按条文只动丁-defer 门槛，盖不住「走不到的那一版自己独占的单元」 | 区间上沿取哪个量在条款里没有定义；γ 的上沿没造盘面量过 |
| H3 | 三条判法共用的是读根环那一层（逐槽读数、位置公式、锚与窗口的算法），不是一张物化的表：三条要的锚、实例表、F 下界各不相同；至少 4 格口径对不上（锚、实例表、F 下界、有效根里算不算由记录施加的版本） | 用户已定的 RS3（抬 F 上限把读不出的槽算作可能的非空有效根）里「读不出」含不含「自证不过」，条款没写 |

## 一、读到的事实（F 系：实现与条款的原句）

| 号 | 事实 | 出处（原句） |
|---|---|---|
| F1 | 新实例第一次发布的 txg = 根环里最大 txg 与环里记录最大 txg 取大再加一 | 「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1237`）；条款「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`.claude/kb/decisions/23-journal的角色与格式.md:371`） |
| F2 | 写行那一次给 [上一个实例, 新实例) 里每个实例写一行：上一个实例 (i, T, W)，中间实例 (i, 0, 0)；T = 恢复择定那一版的 txg；行只追加 | 「let first_row_instance = previous_row.instance.0.max(1);」（`crates/singlefs-core/src/mount.rs:3589`）、「selected_root_txg: CheckpointTxg(0),」（`crates/singlefs-core/src/mount.rs:3601`）、「selected_root_txg: effective_root.checkpoint_txg,」（`crates/singlefs-core/src/mount.rs:5470`）、「rows.extend_from_slice(rows_written);」（`crates/singlefs-core/src/mount.rs:1139`） |
| F3 | 重放前缀只认与所选根同实例、txg 更大的记录；别的实例的记录从不施加 | 「record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0」（`crates/singlefs-replay-prefix/src/lib.rs:145`）；条款「前缀规则不跨实例边界」（`.claude/kb/decisions/23-journal的角色与格式.md:369`） |
| F4 | 同一实例里每次发布接在上一版后面，txg 加一；发布失败就冻结、原样重发同一个 txg，重发成功前不建下一次发布 | 「pub fn checkpoint_txg_of_the_publish_after(」（`crates/singlefs-core/src/transaction.rs:3786`）、「.checked_add(1)」（`crates/singlefs-core/src/transaction.rs:3791`）、「PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {」（`crates/singlefs-core/src/transaction.rs:1205`） |
| F5 | 「根槽写失败推进 txg」今天没有实现，条款自己写在后面的里程碑 | 「原样重发管这一版的串行提交；探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项，随失败处置的代码在后面的里程碑做」（`.claude/kb/decisions/23-journal的角色与格式.md:402`） |
| F6 | I-3.5 的射程把「根槽写失败重发跳一格」列为缺口来源之一（与 F4、F5 对读：今天的 crates 走不到这一源） | 「另有恢复丢掉尾巴记录、根槽写失败重发跳一格、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）」（`.claude/kb/invariants.md:146`） |
| F7 | gap-row 今天的写法：R 的实例大于 R⁻ 的实例；最新根的实例表里有 (R⁻ 的实例, T = R⁻.txg)；中间实例的行都是 (j, 0, 0)；不看 R 所在实例自己的更早版本 | 「if version_instance <= previous_instance {」（`crates/singlefs-checker/src/walk.rs:470`）、「row.instance == previous_instance && row.published_checkpoint_txg == previous_txg」（`crates/singlefs-checker/src/walk.rs:475`）、「row.published_checkpoint_txg == 0 && row.applied_transaction_high_water_mark == 0」（`crates/singlefs-checker/src/walk.rs:487`）；条款「最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判」（`.claude/kb/invariants.md:146`） |
| F8 | 「由记录施加」今天要三样：记录带提交标记并进得了重放前缀、最新根的实例表有这个实例的行且 txg ≤ T、根槽读不出 | 「let applied_by_a_recovery = record.commit_marker == CommitMarker::Present」（`crates/singlefs-checker/src/walk.rs:6774`）、「&& record.checkpoint_txg <= row.published_checkpoint_txg」（`crates/singlefs-checker/src/walk.rs:6782`）、「let root_slot_is_readable = roots.iter().any(」（`crates/singlefs-checker/src/walk.rs:6784`） |
| F9 | checker 自己有一份读 journal 的与一份「下一次挂载会施加哪一段」的复刻，依赖里只有 `singlefs-format` | 「fn records_the_replay_reads(」（`crates/singlefs-checker/src/walk.rs:7137`）、「fn replay_the_next_mount_would_do<'records>(」（`crates/singlefs-checker/src/walk.rs:7227`）、「singlefs-format = { path = "../singlefs-format" }」（`crates/singlefs-checker/Cargo.toml:9`）；D13 已定项 5：「其余一律不共享」（`.claude/kb/decisions/13-验证路线.md:139`） |
| F10 | 实例表只在写行那次发布重写，别的发布（含管理员回退、卸载、抬 F）都把上一版的实例表指针原样带过去；checker 从根记录 170..256 读这条指针 | 「instance_table: InstanceTablePlan::Rewrite(instance_table),」（`crates/singlefs-core/src/mount.rs:3236`）、「instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),」（`crates/singlefs-core/src/transaction.rs:5648`）、「let mut pointer = parse_node_pointer(&record[170..256]);」（`crates/singlefs-checker/src/walk.rs:4788`）；`grep -n 'InstanceTablePlan::Rewrite(' crates/singlefs-core/src/*.rs` 原样输出六行，只有 `mount.rs:3236` 是构造，`transaction.rs` 的 5158、6479、7844、7857、8414 都是 `if let` / `match` 臂 |
| F11 | 根槽位置由 txg 定；一圈窗口 (N − 3S, N] 的 3S 个 txg 各占一个不同的槽（所以窗口里每个槽恰好对应一个位置 t） | 「r == txg mod R 且 s == (txg div R) mod S」（`.claude/kb/invariants.md:71`）；`python3 /tmp/claude-1000/m3-candidate-hole-r1-sonnet/slotcheck.py` 原样输出：`cases 1200 collisions 0`（S ∈ {1,2,3,4,8,16} × N ∈ [0,200)） |
| F12 | 窗口与回收门槛 | 「窗口 = txg > N − 3S（一圈，3S 是环长）」（`.claude/kb/decisions/16-发布语义.md:35`）；「checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.newest_valid_root_txg」（`crates/singlefs-checker/src/walk.rs:6707`） |

## 二、H1：「取号跳过」认什么证据

### 2.1 推出来的（D 系）

- **D1　行证据能证明什么。** 由 F1：新实例的第一个 txg 比环里与记录里见过的最大 txg 大一；由 F3：别的实例的记录从不施加；由 F2：写行那一次给每个中间实例 j 写 (j, 0, 0)、给上一个实例写 (i, T = 所选那一版的 txg)。所以「最新根的实例表里有 (R⁻ 的实例, T = R⁻.txg)、中间实例的行都是 (j, 0, 0)」（F7 的条款原句）合起来证明：**R⁻ 与 R 所在实例的第一版之间，没有任何别的实例的已施加版本**。落在这一段里的 txg 只能是：被抛弃的根、已落没施加的记录、或根本没发出来的号。这一步从 F1–F3、F7 逐句推出，不需要 journal。
- **D2　行证据没盖到的口子。** F7 的代码与条款都只比「R⁻ 的实例」「R 的实例」「两者之间的实例」，不看 R 所在实例 v 自己在 R 之前的版本。由 F4，同一实例里 txg 逐次加一、没有跳号（F5、F6：今天走不到跳号那一源），所以 v 的第一版 f_v 小于 R.txg 时，[f_v, R.txg) 里每个 txg 都是存在过的版本；它们之中根槽读不出、记录又被盖掉的就是「存在过却走不到」。这一类被 F7 当成「接在 R⁻ 后面」：R 相对 R⁻ 的判法照常判，而 N3（只放过用户可见树里的单元）本来是为「洞里可能有一次回退」留的。
- **D3　f_v 能不能从盘上读到。** f_v 是写行那次发布的 txg（`.claude/kb/decisions/16-发布语义.md:43`：「它是新实例的第一次发布」）。由 F10：写行那次发布重写实例表，之后每一版把同一条实例表指针带过去，所以 v 的任一条根（从 170..256 读）里的实例表指针的出生代号 = f_v。这是按代码推的；kb 自己对「照抄同一张实例表」写的是「没量」：「隔一条也是同一张实例表」（`.claude/kb/invariants.md:143`），我也没量（没跑 harness）。
- **D4　journal 里「已落、没施加」的记录各是哪一种。** 由 F3，恢复只把「与所选根同实例、txg 更大」的记录当候选。盘上一条记录 (j, t) 落了、而 t 位置上没有可读的根，有三种：① j ≠ 现行实例，行里 (j, Tj) 且 t > Tj（或 (j, 0, 0)）：没施加，号被占了；② j 是现行实例（最新根的实例），没有行：这条记录若 t 小于最新根，它就是被最新根「覆盖」的一版，**存在过**，只是根槽读不出；③ j ≠ 现行实例、t ≤ Tj：已施加的版本（F8 的由记录施加那一格）。把三种都当成「没施加所以是跳过」，②③ 就是误认。要区分，②③ 靠的是行（①③）与最新根的 txg（②）——也就是行证据本身，journal 只提供「这个 txg 被哪个实例占着」。
- **D5　journal 证据在「环被盖掉」那一形上读不到。** F9 的读法读的是环里还在的记录；环被后来的记录盖掉（kb 把它列为缺口来源：「根槽被环盖掉而记录也被盖掉（短 journal 环）」，`.claude/kb/invariants.md:146`）之后，被盖掉的记录不在，证据只能退回行证据。出生代号（D3）不住在 journal 环里，不受这一形影响。

### 2.2 三个候选

| 候选 | 「取号跳过」的认法 | 在哪些合法历史上把真洞误认成跳过 | 在哪些坏镜像上漏红 | checker 读不读得到 |
|---|---|---|---|---|
| H1-r　只认行 | 今天的 gap-row（F7）：行证据齐就是跳过 | D2 那一类：R 所在实例有 f_v < R.txg 的不可达版本、其中有回退发布。需要一次读故障（根槽读坏）加一次回退，两个不同的前提；转述背景材料：1000 段随机合法历史里带 I-3.5 计数的 22696 个盘面 I-3.5 全成立（`research/prompts/m3-root-ring-code-r2-opus-output.md` 第 136–142 行）；这批盘面里有没有 D2 那一类、含不含读故障，我没读产物，不知道；条款没写「合法历史」含不含读故障 | 接不上的对里，用户可见树里的新写单元的诞生代号被改小的坏镜像判不出（kb 自己写的盲区）；接得上的对不放过 | 行已读（F7 用的就是 `instance_table_rows_of_root_without_judging`）；不碰 journal |
| H1-j　行 ∪ journal 里已落未施加的记录（字面读法） | 有 (j, t) 记录而 t 位置上没有可读根、且这条记录不在「已施加」集合里，就当跳过 | D4 ②：现行实例 1 连发 5..9，txg 7 的根槽读坏，记录 (1, 7) 在环里、没有现行实例的行；字面读法把 7 当跳过；若桥接改成「中间每个 t 都是跳过就算接上」（F7 的实例比较 `version_instance <= previous_instance` 本来挡着同实例），7 若是回退发布，R=(1, 8) 相对 R⁻=(1, 6) 就按接上了全判，老出生的单元判红；7 的独占单元在 defer 里未回收时，I-3.1 也把「一代该存在的版本」当成没存在过 | 同 H1-r；另外把同实例的洞也当「接得上」，盲区变小但靠的是误认 | journal 环盖掉那一形读不到记录，退回 H1-r（不比 H1-r 更糟，但在那一形上没有它声称的增益）；有独立读法（F9）；把「没施加」收成「行证明没施加」就退回 H1-r |
| H1-b　行 + R 的实例表指针出生代号 | 行证据齐且 R 的实例表指针出生代号 = R.txg（R 就是 v 的第一版）才算接上；出生代号 < R.txg 时 [f_v, R.txg) 里不在 R⁻ 之后的 txg 标「存在过」，这一对按接不上（N3）；出生代号读不出也按接不上 | D2 那一类不再误认（推的：D3 成立才成立）；残余：出生代号读不出的单元（实例表单元坏）按接不上，判得更松（漏红）而不是更严（误红） | 比 H1-r 多一类进 N3：R 所在实例有更早不可达版本的对；这些对本来就判不了「洞里是不是回退」（转述：`research/prompts/m3-birth-floor-r2-opus-output.md` 的 J1-a 打中之后第 2 句，大意是「这一对之间有洞」可判、「洞里是不是回退」不可判） | 实例表指针出生代号是 checker 已读的字段（`parse_node_pointer(&record[170..256])`，F10）；与 journal 无关，D5 那一形不影响 |

### 2.3 逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| H1 第一问：「取号跳过」是否认实例表行证据 | 一致 | F7 的条款句与代码同写行证据；D1 从 F1–F3 逐句推出行证据证明了「R⁻ 与 R 所在实例第一版之间没有别的实例的已施加版本」 |
| H1 第一问的补口：R 所在实例自己的更早版本 | 规则没说（条款与代码都没覆盖，D2 推出这一口） | F7 的代码、条款都不看 R 所在实例；D3 给出一条盘上读得到的补法（H1-b），推的，没量 |
| H1 第二问：是否也认 journal 里已落未施加的记录 | 规则没说；字面读法推出冲突（D4 ②） | 条款只写了跨实例边界的记录不施加：「前缀规则不跨实例边界」（`.claude/kb/decisions/23-journal的角色与格式.md:369`）；「同实例、txg 不高于所选根的记录也不施加（被所选根覆盖）」是 F3 代码的口径，条款里没有单独一句；这两种「没施加」对「是不是跳过」的含义相反（转述我的读法） |
| H1 第三问：checker 读不读得到那条未施加的记录 | 一致（读得到，有自己一份） | F9：`records_the_replay_reads`、`replay_the_next_mount_would_do`；环被盖掉那一形读不到（D5） |
| H1 第四问：D13 已定项 5 | 一致 | F9：checker 依赖里只有格式常量；H1-b 读的是已读的指针字段，不加共用 |
| 「缺口来源」六样里今天走得到的 | 冲突（一句） | 条款「根槽写失败重发跳一格」（F6，`.claude/kb/invariants.md:146`）对 F4/F5：今天发布失败就冻结、原样重发同一个 txg，跳号那一源「仍在本项，随失败处置的代码在后面的里程碑做」（`.claude/kb/decisions/23-journal的角色与格式.md:402`）；今天的 crates 走不到 |

### 2.4 什么现象会推翻

- 「行证据对今天 crates 能走出的缺口全覆盖」：有一段由今天 crates 走出的历史（不含读故障、不含 F5 的跳号），出现 txg 缺口而 F7 的行证据接不上。
- D3（出生代号 = f_v）：任一条 v 的根（不是写行那条）里的实例表指针出生代号 ≠ 写行那次发布的 txg。查法：`grep -n 'InstanceTablePlan::' crates/singlefs-core/src/*.rs` 里 `Rewrite(` 的构造不止 `mount.rs:3236` 一处，或 harness 里读两条根的指针对不上。
- D4 ②：现行实例的行不存在时恢复也施加了 t 小于最新根的记录（读 F3 的 `above_the_chosen_root` 只认 txg 更大，所以今天不会）。

## 三、H2：I-3.1 在「存在过却走不到」的那一代上判什么

### 3.1 读到的事实（G 系）

| 号 | 事实 | 出处（原句） |
|---|---|---|
| G1 | I-3.1 今天是逐盘一个等式：记账的「已分配」对遍历得到的槽数加隔离豁免加 F 之下仍在 defer 的槽；没有「一对」这个粒度 | 「judgements.judge("I-3.1", allocated == Some(walked),」（`crates/singlefs-checker/src/walk.rs:8186`）；条款「已分配空间统计 == 实际遍历所有引用得到的和」（`.claude/kb/invariants.md:143`） |
| G2 | 丁-defer 并进的是「F 之下、读得到的根引用着」的槽；记账里还在 defer（已释放、释放代高于门槛）的算已分配 | 「let roots_below_the_floor: Vec<&crate::RootView> = roots」（`crates/singlefs-checker/src/walk.rs:8120`）、「记账照算已分配」（`crates/singlefs-checker/src/walk.rs:5256`）、「record.is_released && record.generation > reclaim_threshold」（`crates/singlefs-checker/src/walk.rs:5380`） |
| G3 | I-3.1 今天唯一的「不适用」是整条：最新根带的 F 低于 F 生效值 | 「最新根带的 F 低于 F 生效值（系统配置里的新 F 已落、带它的根还没落或全读不出）」（`crates/singlefs-checker/src/walk.rs:8232`） |
| G4 | 每次发布连空发布都重写树表单元，所以任何一个存在过的版本至少独占一个单元 | 「每一次发布（连空发布）都重写树表单元」（`.claude/kb/decisions/16-发布语义.md:47`） |
| G5 | 分配器在挂载那一刻只对「设备报读错」的窗口内槽另记一条（按它能住的最大 txg 进进程内回收门槛），自证不过的槽当没有根 | 「RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => Some(*ring_slot),」（`crates/singlefs-core/src/mount.rs:1685`）、「挂载那一刻设备报读错的槽，落在挂载时的一圈窗口里」（`crates/singlefs-core/src/allocator.rs:950`）；条款「读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的」（`.claude/kb/decisions/16-发布语义.md:35`） |
| G6 | 这一形今天已有一次旁证：窗口之前，短环上的洞让池级 checker 红 I-3.1，机理是洞里的版本新写的单元放进 defer、回收不了仍算已分配，候选集里没有根引用它们；kb 把它与 I-3.5 的「候选集有洞」判法并成一件 | 「洞里那几版新写的单元已释放进 defer，环里最旧有效根是旧根 8、回收不了仍算已分配」（`research/prompts/m3-birth-floor-r2-main-verification.md:32`）；「I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查」（`.claude/kb/invariants.md:146`） |

### 3.2 推出来的

- **E-D1　走不到的那一代怎么从盘上分。** 窗口里（F11：每个槽对应一个位置 t）t ≥ F_生效、t ≤ N 的位置，扣掉「槽里是 txg = t 的根且读得到」与「由记录施加（F8）」之后，剩下的按 H1 的证据分三种：**不存在**（行证据接上的那一对之间、t < f_v，或槽里 txg = t 的根被判抛弃）、**存在过却走不到·已证**（实例 i 有可见版本——根或由记录施加——而 t 在它们之间或之后、至 Ti（现行实例至最新根）；由 F4 同实例连续）、**未知**（实例没有任何可见版本，或实例表读不出）。短 journal 环那一形（F6 末一样）是「已证」：根槽没写过、记录被盖掉、行里 Ti ≥ t。根槽读坏的那一形，坏的是 t 所在的槽，t 在同实例两条可见根之间，也是「已证」。
- **E-D2　不平衡的方向。** 由 G2、G4 与「一个版本独占的单元在下一版放进 defer、释放代高于门槛时回收不了」：走不到的存在版本使「已分配」**大于**遍历，盘上的记账里没有任何一处会让走不到的版本使「已分配」比遍历小（这一句是推的，没造盘面）。所以不安全方向（真坏）是「已分配 < 遍历」，安全方向（不是坏）是「已分配 > 遍历」，且多出来的部分只来自走不到的那几版独占的单元。E1 针对的 K2-e 那一形是反方向的（见 E-D4）。
- **E-D3　E1 按条文能不能盖住走不到的那一版自己的单元。** E1 的条文（背景材料正文第一节，转述）：丁-defer 门槛从一个值改成区间，下沿取窗口里那一槽没住着它该住的 txg 的根的位置的最小 txg，记账落在区间两端的遍历之间就算对得上。丁-defer 并进的只有 G2 里 `roots_below_the_floor` 的引用——F 之下、**读得到**的根。走不到的那一版在窗口里、txg ≥ F、根读不到，不在这个集合里，改门槛改不了它。所以**按条文，E1 盖不住这一类**；盖得住的只有门槛不确定带里的槽（释放代在 (门槛下沿, 门槛上沿] 里、被 F 之下的读得到的根引用着的）。
- **E-D4　E1 的区间两端的方向与代价。** 下沿压低门槛 ⇒ 并进更多槽 ⇒ 遍历变大；区间的下端是「按较高门槛的遍历」，所以它**接受「已分配 < 遍历（取今天的门槛）」**。这正是不安全方向，所以 E1 会放过一类真的记账少：被 F 之下的读得到的根引用、释放代落在 (今天门槛, 下沿推高的那一端] 里的槽，记账少记了的那一格。带宽 = 这类槽数，有界。再者位置集合「槽里没住着它该住的 txg 的根」含「槽里是上一圈的根」那些位置——它们的版本从没存在过，对回收门槛没有贡献（G5：分配器的根环表只收读得到的根与设备报读错的槽），并进下沿只会把区间拉宽。
- **E-D5　整条不适用的粒度。** G1、G3：I-3.1 没有「一对」，最细的不适用是整条（或逐盘一格）。「这一代所在的那一对判不适用、整条照判其余」按今天的代码落不成；落得成的是「表里有已证或未知的走不到的那一代，I-3.1 整条报不适用」，沿 G3 那一处的写法。后果：这类盘面上 I-3.1 一个字节的错也看不出。
- **E-D6　候选 γ（我推的，零轮）。** 表里没有「已证」「未知」位置时，等式照旧，一字不改（今天调查员量到的 190 个有洞状态都是由记录施加、取号跳过两类，I-3.1 全成立，转述）。有时，逐盘判 `walked ≤ allocated ≤ walked + U`：下沿就是今天的 `walked`（真的记账少照红）；上沿 U = 最新根分配记录树里「已释放、释放代 g 高于门槛、g 落在某个走不到的 t 与 t 之后下一个存在版本的 txg 之间（不可证时取到 N）、且不在遍历到的槽里」的槽数乘槽宽。U 用的记录是 `released_above_the_floor` 同一份读法已经读出来的（`crates/singlefs-checker/src/walk.rs:5276`），不新读盘。E-D1 的分类与 E-D2 的方向是它的前提。

### 3.3 两种来源上各候选判成什么

| 来源 | 维持等式（不改） | 整条不适用 | E1（门槛区间） | γ（只加走不到那几版的上沿） |
|---|---|---|---|---|
| 根槽读坏：t 在窗口里、≥ F、同实例两条可见根之间，槽读不出 / 自证不过 | 红（误红：G6 同机理，E-D2 方向） | 不适用 | 仍红（E-D3：独占单元不在丁-defer 集合里，G4 保证至少有一个） | 落在 [walked, walked + U] 里 ⇒ 成立 |
| 短 journal 环：根槽没写过、记录被盖、行里 Ti ≥ t | 红（G6 的旧形） | 不适用 | 仍红（同上） | 成立（同上） |
| 位置不存在（行证据接上、t < f_v）与被抛弃的根 | 等式成立（今天） | 不变，等式成立 | 不变 | 不变，等式成立 |
| 门槛不确定带（K2-e 的形：整条命读不出的窗口内槽，分配器以最大 txg 记） | 红（遍历多于记账） | 不适用 | 成立（它为这一形设计） | 不成立：γ 的下沿是今天的 walked，这一形是「已分配 < walked」，γ 照红 |

### 3.4 逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| 候选「这一代所在的那一对判不适用」 | 冲突（粒度） | 正文把它写成按「一对」；G1、G3：今天 I-3.1 是逐盘等式，最细是整条不适用（E-D5）；候选改写成整条不适用 |
| 候选「区间形 E1」盖不盖两种来源 | 冲突 | 条文只动丁-defer 门槛（E-D3）；两种来源的差额来自走不到的版本独占单元（G4），不在那个集合里；仍红 |
| E1 放不放过真的记账少 | 会（有界） | E-D4：区间下端低于今天的遍历 |
| 区间上下沿取哪个量 | 规则没说 | kb 对「走不到的那一代」只有 F6、G6 的机理描述，没有区间定义；γ 是推的，没造盘面量过 |
| 哪一边是安全方向 | 一致（与背景材料同方向） | 记账多于遍历是安全方向：E-D2 从 G2、G4 推出，走不到的版本只会让「已分配」更大 |
| 候选不止两个 | 加一个 γ（零轮，推的） | E-D6 |

### 3.5 什么现象会推翻

- E-D2（走不到只会让已分配更大）：造出一张走不到的存在版本使遍历大于已分配的盘面（例如该版本独占的单元在窗口内已被回收，而遍历还数着它们）。
- E-D3（E1 盖不住）：在副本上造一张「根槽读坏、t 在窗口里、≥ F」的盘面，实现 E1 的区间后 I-3.1 转绿。
- γ：造一张走不到版本盘面，`allocated − walked` 不等于 U（上沿算多或算少）；或一张真泄漏（记账里一条释放代落在带内、没人引用的已释放记录）被 γ 放过，且这一形在合法历史上不出现。
- E-D1 的「已证」：同实例两条可见根之间的 t 存在一个从没发布过的号（F4、F5 反过来：今天跳号走得到）。

## 四、H3：三条判法共用一张洞表

### 4.1 读到的事实（H 系）

| 号 | 事实 | 出处（原句） |
|---|---|---|
| H1-F1 | I-3.5 读的是「被走过的相邻版本」（候选根加由记录施加）与最新根的实例表行；候选集按最新根的锚 N、F_生效、最新根的实例表过滤 | 「judge_the_appearance_rule_of_adjacent_candidate_versions(」（`crates/singlefs-checker/src/walk.rs:7860`）、「let abandoned_by_the_newest_roots_table」（`crates/singlefs-checker/src/walk.rs:7712`）、「let window = OneRevolutionWindow {」（`crates/singlefs-checker/src/walk.rs:7761`）、「let outside_the_window = !window.contains(root.checkpoint_txg);」（`crates/singlefs-checker/src/walk.rs:7777`） |
| H1-F2 | I-3.1 的候选、丁-defer 门槛用同一套最新根的锚、实例表与 F_生效 | 「let deferred_threshold = deferred_slots_reclaim_threshold(」（`crates/singlefs-checker/src/walk.rs:8131`）、「let effective_rollback_floor = verified_system_configurations_of_this_pool」（`crates/singlefs-checker/src/walk.rs:7753`） |
| H1-F3 | I-7.9 今天按**抬 F 的那一条根自己**的实例表判「之前的根」是否抛弃，下界是**抬之前**的 F，不分窗口；每条抬 F 的根各取一份 | 「root.checkpoint_txg < raising_root.checkpoint_txg」（`crates/singlefs-checker/src/walk.rs:4952`）、「&& root.checkpoint_txg >= floor_before_the_raise」（`crates/singlefs-checker/src/walk.rs:4953`）、「&& !abandoned_by_instance_table_rows(&instance_table_rows_of_the_raising_root, root)」（`crates/singlefs-checker/src/walk.rs:4954`）、「let floor_before_the_raise = previous_root_of_the_same_instance.rollback_floor;」（`crates/singlefs-checker/src/walk.rs:5067`） |
| H1-F4 | 实现里的抬 F 上限已经按窗口取（B2）：锚 = 读得出、按传入实例表不被抛弃的最新根；有效 = txg ≥ 今天的 F、未被抛弃、在窗口里；只读根环里读得出的根，不数由记录施加的版本 | 「let window_anchor = readable」（`crates/singlefs-core/src/mount.rs:2145`）、「root.checkpoint_txg >= current_floor」（`crates/singlefs-core/src/mount.rs:2158`）、「is_inside_the_window(root.checkpoint_txg)」（`crates/singlefs-core/src/mount.rs:2160`）、「let readable = roots_of_the_ring_for_the_ceiling(」（`crates/singlefs-core/src/mount.rs:2136`） |
| H1-F5 | checker 的上限算法已经是区间：空不空判不了的根在上沿一律按「计入」；上沿、下沿之间这条根不判 | 「always_counted: &[(u64, u32)],」（`crates/singlefs-checker/src/walk.rs:4894`）、「if raised_floor > ceiling.lowest_possible && raised_floor <= ceiling.highest_possible {」（`crates/singlefs-checker/src/walk.rs:5108`）；条款「算上限要比的有效根树表读不出时，它空不空判不了、不猜：F 不高于把这些根全算空得出的上限就成立，高于全算非空得出的上限就违例，落在两者之间这条根不判」（`.claude/kb/invariants.md:63`） |
| H1-F6 | 用户已定（2026-10-04）：I-7.9 的「之前的根」只取一圈窗口；抬 F 的上限把窗口里挂载那一刻读不出、之后没写过的槽当作可能是一条非空有效根 | 转述：`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS3 一行；条款已写「都只在一圈窗口里的根上取（用户 2026-10-03 定，零轮：窗口外的旧根，窗口已许可它的树表被复用，读不出不再拒抬 F）」（`.claude/kb/decisions/16-发布语义.md:37`） |
| H1-F7 | 逐槽读数与根的位置：`root_slot_positions` 排出每个槽的位置，`valid_roots` 只交回自证过的；位置公式条款与 checker 各一份、I-7.17 判两边对上 | 「pub fn root_slot_positions(geometry: &PoolGeometry) -> Vec<(u64, u64, u32, u64)> {」（`crates/singlefs-checker/src/image.rs:872`）、「pub fn valid_roots(reader: &dyn ImageReader, geometry: &PoolGeometry) -> Vec<(u64, u64, RootView)> {」（`crates/singlefs-checker/src/image.rs:1158`） |

### 4.2 推出来的

- **T-D1　三条各要洞表的哪几列。** 洞表按调查员的两列存：L-槽（逐槽读数：自证过的根 (txg, 实例) / 读不出 / 自证不过；「槽里是上一圈的根」不是第四种读数，是「读得到、txg ≠ 位置 t、且 txg ≡ t (mod 3S)」，由 F11 与 I-7.17 定）与 L-版（t 这一版存在不存在：根在位 / 由记录施加 / 被抛弃 / 不存在·行证 / 存在过却走不到·已证 / 未知，E-D1）。

| 判法 | 要的列 | 不要的列 |
|---|---|---|
| I-3.5 | L-版：被走过的版本（根在位、由记录施加）、「不存在·行证」（对之间每个 t）、H1-b 的出生代号；不要每个 t 的逐槽读数——桥接由行证据一次定，不逐位置 | L-槽 |
| I-3.1 | L-版里「存在过却走不到·已证」「未知」（γ 的 P 与 U）；E1 型的门槛区间要 L-槽里「设备报读错」的位置 | 「被抛弃」「根在位」的细节（已在候选过滤里） |
| I-7.9 | L-槽里「设备报读错」的位置（RS3 的幽灵根，进 H1-F5 的 `always_counted`，txg 取位置 t）；位置所在的窗口按**这条抬 F 的根**算 | L-版整列：由记录施加的版本没有槽、没有盘（H1-F4：实现只数读得出的根），读它只会数多 |

- **T-D2　三个上下文的参数不同，所以共用的是读槽那一层，不是一张物化的表。** 同一个「窗口里、F 之上、没被抛弃」要三套参数：

| 参数 | I-3.5、I-3.1（H1-F1、H1-F2） | I-7.9（H1-F3、H1-F4） |
|---|---|---|
| 锚 N | 镜像里最新根的 txg，一个 | 每条抬 F 的根 r 各一个：txg < r、按 r 的实例表未被抛弃的最新读得出的根 |
| 抛弃判的实例表 | 最新根的 | r 自己的 |
| F 下界 | F_生效（镜像最终值） | 抬之前的 F（同实例前一条根带的） |
| t 的范围 | (N − 3S, N] 且 ≥ F_生效 | (N_r − 3S, N_r] 且 ≥ 抬之前的 F；t < r |
| 由记录施加的版本 | 算（走读、桥接都用） | 不算 |

- **T-D3　口径对不上的格（把用户已定的 I-7.9 读法放进一张按最终镜像物化的表）。** 每一格都是推的，没造盘面：
  1. **锚。** 按最终镜像的 N 取窗口，窗口比实现在抬 F 那一刻的窗口晚，checker 的集合是实现集合的子集，上限不低于实现的（子集只删最旧的根，「第 4 新」或「最旧有效根」只会更新），不会误红，但漏掉 (实现上限, checker 上限] 这一带里的过头抬 F。按 r 前的 N_r 取就与实现一致，没有这一带。
  2. **实例表。** 用最新根的表，r 之前、后来被抛弃的同实例根（都是 r 之前最新的几条）会被删掉，上限不变或更低，实现抬对了的 F 判红。r 自己的表没有后来才写的行。
  3. **F 下界。** 用 F_生效 截掉 [抬之前的 F, F_生效) 的位置，截掉的是 r 之前最旧的那几条、恰好是这次抬 F 越过去的根：「第 4 新的非空」不足 4 个时退到「最旧有效根」，最旧的被删，上限不变或偏高，漏掉一带过头抬 F。
  4. **由记录施加的版本。** 列里有，I-7.9 不能数（无槽无盘，H1-F4）。
  5. **RS3 的「读不出」。** 分配器只对设备报读错的槽另记一条，自证不过的槽当没有根（G5：`mount.rs:1685`）；条款 `.claude/kb/decisions/16-发布语义.md` 现行的「根槽这一次读坏」一行写的是「读不出或自证不过」，用户 2026-10-04 定的 RS3 只写「读不出」。含不含「自证不过」条款没写（推不出）；含进去，每个撕了根槽的崩溃状态的上限都变宽，不含就与 G5 一致。
  6. **幽灵根的 txg。** 实现挂载时一个读不出的槽在 (N − 3S, first_txg − 1] 里能住一两个 txg（G5），checker 在静态镜像的窗口里每个槽只有一个位置（F11）；RS3 要的是上沿，取位置 t 够用，但与实现「记最大」的值可能差一圈，推不出两者逐位相等。
- **T-D4　共用读槽层，错会不会连带。** 会，方向分两种：① 把一个读不出的槽读成「上一圈的根」或把存在过的版本读成不存在：I-7.9 的幽灵根少一条（上限偏低）、I-3.1 的等式偏严、I-3.5 对这一对判得更全——三条同时**偏红**，看得见；② 把没写过的槽读成坏槽：I-7.9 上限偏宽、I-3.1 走区间或不适用、I-3.5 落进 N3——三条同时**偏松**，没有一条红，谁也看不到。所以口径层的检查要立在这三条之外（`.claude/singlefs-ai-sop/rules/show-me-test.md` 的「但要先问这个坑在哪一层：装置里，还是口径上」：同一个量由两套装置各算一遍，对不上判红）：位置公式已有 I-7.17 钉（`.claude/kb/invariants.md:71`），读数种类（读不出 / 自证不过 / 上一圈的根）与「存在不存在」没有钉；harness 的历史生成器知道每个位置实际发生了什么，可以逐位置对拍。这一条是推的，今天没有这样的检查（表还不存在）。

### 4.3 逐格判定

| 格 | 判定 | 依据 |
|---|---|---|
| 三条各要哪几列 | 一致（调查员第十节第三段） | T-D1：I-3.5 要「存在·不存在」两版列，I-3.1 要走不到的那几代，I-7.9 只要设备报读错的槽；比调查员写的多一条：I-7.9 不要「槽里是上一圈的根」之外的列，而 I-3.5 不是逐位置读槽，是行证据 |
| 共用一张表会不会让一条的读法错连带另两条 | 会（共用读槽层时） | T-D4；派生层各自参数化可隔开，读槽层隔不开 |
| 用户已定的 I-7.9 只取窗口、抬 F 上限把读不出的槽算进去，放进洞表之后有没有对不上的格 | 有，六格（T-D3） | 锚、实例表、F 下界、由记录施加的版本四格推得出；「读不出」含不含自证不过、幽灵根 txg 两格推不出 |
| 候选「三条各算各的」 | 没被排除，但推出的是：读槽层共用、派生层分开 | T-D2 的参数表 |

### 4.4 什么现象会推翻

- T-D2（实现的锚是抬 F 那一刻的）：`crates/singlefs-core/src/mount.rs:2145` 的锚改成别的量（例如镜像最终最新根）。
- T-D3 第 2 格：造一段合法历史（抬 F 之后崩溃、恢复落到抬 F 之前的根），用最新根的表算上限时实现抬对了的 F 并不判红。
- T-D4 ②：造一张把没写过的槽读成坏槽的表，I-7.9、I-3.1、I-3.5 里有一条红。

## 五、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| H1-a「取号跳过」认实例表行证据 | 一致 | F7 条款句与代码同写行证据；D1 从 F1–F3 逐句推出行证据证明「R⁻ 与 R 所在实例第一版之间没有别的实例的已施加版本」 |
| H1-b 行证据没盖到 R 所在实例自己的更早版本 | 规则没说 | D2 推出这一口；H1-b 用 R 的实例表指针出生代号补，依赖「写行之后每一版照抄同一张实例表」（kb 写「没量」），推的 |
| H1-c 也认 journal 里已落未施加的记录 | 规则没说；字面读法冲突 | D4 ②：现行实例被最新根覆盖的记录是存在过的版本；「没施加」要靠行才分得出，收成行证据就退回 H1-r；journal 环被盖掉那一形读不到 |
| H1-d checker 自己有没有一份读法（D13 已定项 5） | 一致 | F9：自有 journal 读法与「下一次挂载会施加哪一段」的复刻，依赖里只有格式常量 |
| H1-e 缺口来源里「根槽写失败重发跳一格」 | 冲突（条款对今天的实现） | F4、F5、F6：今天的 crates 发布失败就冻结、原样重发同一个 txg，这一源走不到 |
| H2-a「这一对判不适用、整条照判其余」 | 冲突（粒度） | G1、G3：I-3.1 是逐盘等式，最细的不适用是整条；候选改成整条不适用 |
| H2-b E1（丁-defer 门槛区间） | 冲突（盖不住） | E-D3、E-D4：条文只动 F 之下读得到的根的门槛，走不到的版本独占的单元不在其中（G4 保证至少一个）；区间下端低于今天的遍历，会放过有界的一带记账少 |
| H2-c 区间上下沿取哪个量 | 规则没说 | kb 只有机理描述（F6、G6）没有定义；γ 的上沿 U 是推的，没造盘面 |
| H2-d 哪一边是安全方向 | 一致 | E-D2：走不到的版本只会让记账比遍历多；记账少是不安全方向 |
| H2-e 候选不止两个 | γ（零轮，推的） | 无「已证 / 未知」位置时等式照旧；有时判 walked ≤ allocated ≤ walked + U |
| H3-a 三条各要洞表哪几列 | 一致 | T-D1：I-3.5 要版本列与行证据（不逐位置读槽）、I-3.1 要走不到的那几代、I-7.9 只要设备报读错的槽 |
| H3-b 共用一张表会不会连带错 | 会 | T-D4：读槽层的错同时偏红或同时偏松，偏松没有一条红；派生层按上下文参数化可以隔开 |
| H3-c 用户已定的 I-7.9 读法放进洞表后有没有口径对不上的格 | 有（六格；四格推得出） | 锚、实例表、F 下界、由记录施加的版本推得出；「读不出」含不含自证不过、幽灵根 txg 推不出 |

## 六、复跑的命令

- 窗口里每个槽恰好一个位置（F11）：`python3 /tmp/claude-1000/m3-candidate-hole-r1-sonnet/slotcheck.py`，原样输出 `cases 1200 collisions 0`。
- `Rewrite` 的构造点（F10）：在冻结副本根下 `grep -n 'InstanceTablePlan::Rewrite(' crates/singlefs-core/src/*.rs`，原样输出：
  `crates/singlefs-core/src/mount.rs:3236:        instance_table: InstanceTablePlan::Rewrite(instance_table),`
  `crates/singlefs-core/src/transaction.rs:5158:        if let InstanceTablePlan::Rewrite(rewrite) = &self.instance_table {`
  `crates/singlefs-core/src/transaction.rs:6479:        InstanceTablePlan::Rewrite(rewrite) => {`
  `crates/singlefs-core/src/transaction.rs:7844:        InstanceTablePlan::Rewrite(rewrite) => build_instance_table_chain(`
  `crates/singlefs-core/src/transaction.rs:7857:        InstanceTablePlan::Rewrite(_) => {`
  `crates/singlefs-core/src/transaction.rs:8414:        (InstanceTablePlan::Rewrite(_), _) => {`
- 其余事实都是读代码与 kb 现查的行（引文逐处过了 `cite-check.py`，见文末）。

## 七、没做什么

- 没编译、没跑任何 harness 档或 checker 档测试，没写原型。D3（出生代号 = 写行那次发布的 txg）、E-D2、E-D3、γ 的上沿 U、T-D3 各格都是读代码推的，没在盘面上量；它们的推翻条件在各节末。
- 调查员报告（`/tmp/claude-1000/m3-r1/batch8-investigate/report.md`，在仓外）与主 agent 的观测（190 个有洞状态、22696 个盘面、2181 个候选集有洞）只当转述用，没复核；cite-check 对仓外路径判不了。
- 没读也没推「实现里的内存分配器门槛」与 checker 门槛逐位相等的那一格（`mount.rs` 挂载重建时读不出的槽按最小 txg、进程内记最大 txg，G5）；γ 的下沿「就是今天的 walked」默认两边这一格对得上，没证明。
- 没判别的腿是攻方的 H1、H2 对每个候选造误红与漏红的盘面；这里写的「误红的历史」「漏红的坏镜像」是推出来的形，不是造出来的。
- 不替主 agent 采纳候选或出判决；不写岔路单。
