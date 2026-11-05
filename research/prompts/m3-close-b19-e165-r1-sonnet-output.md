# m3-close-b19-e165-r1 正推腿报告（Sonnet，2026-11-05）

立场：正推。从已定条款与 `crates/` 今天的写者推「合法写者下盘上能出现什么根环」，再与 C650 剩下那一句逐格比。
没有写模型，没有起任何命令跑代码；下面每一格标「读码推的」或「待主 agent 跑」。引文都在被引文件里 `grep -nF` 过，行号是那份文件自己的行号。

## 一、结论先行

| 问 | 结论 | 标 |
|---|---|---|
| Q1.1 形甲（同 txg 不同实例） | **造不出**，与读坏多少个槽、有没有系统配置见证都无关：挡它的是「一个 txg 只有一个落点槽」，写者和读者两侧各一道 | 读码推的 |
| Q1.1 形乙（实例代号更高而 txg 更低） | **单故障承诺内造不出**；同一次挂载里至少 5 处同时读不到（1 个根槽加两条记录各两份镜像）才造得出，造出之后下一次择根里只有 v1826 与基线分开 | 读码推的 |
| Q1.2 最小历史 | 形乙：三次挂载加两次崩溃，见第四节；形甲无 | 读码推的 |
| Q1.3 等价性留档 | v1823、v1825：合法域内只有第 0 代根这一处能平局、且那三份同字节，能留成等价变异；v1826 不是等价变异，不能留档 | 读码推的 |
| Q3 判据 | 不是表里四种的原样；最近的是「判别子观测不到」；改成按变异分开写 | 读码推的 |

先说两处与背景材料不同的读码结果，主 agent 在判决里要现查：

1. 背景材料第一节写「自证过、未被抛弃的摊成择新键」。`choose_root` 不按实例表滤抛弃：`recovery.rs:1888` 起的函数体里只有读槽、自证、落点核，没有 `abandoned`；
   可写挂载读阶段调的也是它（`mount.rs:8938` `choose_root(cache, system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;`）。
   按实例表判抛弃的只在回退候选、影子账、抬 F 和 F 生效值那几处（`recovery.rs:2222` `pub fn publish_is_abandoned_by_the_instance_table(`）。
   后果：被抛弃的根（txg 高于所选版本的旧时间线的根）仍参与择根，它之所以不压过新实例，靠的只是新实例的第一个 txg 大于环里一切可见的根（`mount.rs:1419`）。
   这一点把形乙的挡法全压在「第一个 txg 够大」上，见第三节。
2. 新实例第一个 txg 里「根那一半」不是读阶段那一遍读出来的，是另读一遍盘：`mount.rs:1412` `    let highest_ring_txg = highest_root_txg(` 传的是 `devices`（直读），
   而择根走读阶段的读缓存（`mount.rs:9007` `let cache = ReadStageCache::empty_over(devices);`）。「记录那一半」是读阶段扫出来的 `records`（`mount.rs:9511` 的调用）。
   所以同一次挂载里择根看到的根和算第一个 txg 看到的根可以不同；E165 实验页自己也写了这一点没造（`165-恢复重放有界穷举与变异杀伤界.md:1449`
   「这一族的读故障不碰记录槽，造不出两者不同的态」）。

## 二、读到的实现（文件:行，读码）

写者侧（根怎么上盘）：

- 根槽只有两个写者：发布路径 `transaction.rs:570` `let target = target_for_publish(`，和 mkfs 种第 0 代根 `make_filesystem.rs:776`
  `device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;`（槽集合 `make_filesystem.rs:821` `pub fn genesis_root_ring_slots()`，三个区域各自的槽 0）。
  查法：`grep -rn 'WrittenStructureKind::RootSlot' crates/*/src` 在 core 里只有计账，写根的 `write_at` 只有这两处；`slot_offset(` 的调用点在 core 里还有读根环的几处，都是读。
- 落点只由 txg 定：`root_ring.rs:172` `region: checkpoint_txg.0 % ROOT_RING_REGIONS,`，槽号 `(txg / 3) % S`（`root_ring.rs:167` 起的 `pub fn target_for_publish(`）。
  一次发布只写这一个槽，根不镜像（D16「发布的持久顺序」那一行写「根槽（FUA）」单写）。
- 同一实例内 txg 逐次加一：`transaction.rs:5839` `pub fn checkpoint_txg_of_the_publish_after(` 是「它加一」。
- 新实例第一个 txg：`mount.rs:1402` `fn first_txg_of_new_instance<` 取 `checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))`（`mount.rs:1419`），
  `highest_root_txg` 经 `visit_valid_roots`，读不出的槽 `recovery.rs:1809` `RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => continue,`，
  读得出而自证不过的也跳过，所以三种「读坏」（设备报读错、读得出非全零自证不过、读回全零）对 `highest_root_txg` 与对 `choose_root` 一样：都是「看不见这个根」。
  记录那一半：`scan_journal`（`recovery.rs:5059`）读每块盘的环，两份镜像任一份自证即在（D23 `23-journal的角色与格式.md:367` 「任一份自证校验和过即在」）。
- 每条记录写到每块盘同一偏移：`transaction.rs:558` `CommitStep::WriteJournalRecordToEveryDevice { counter, record } => {` 与 `transaction.rs:559` 的 `record_offset(counter, …)`。
  偏移只由计数器定。
- 一次发布的持久顺序：`transaction.rs:1652` `fn persist_the_root_then_rotate_the_system_configuration<`（根槽 FUA、系统配置槽轮换、屏障），
  条款 D16 `16-发布语义.md:215`「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」。
- 取号：`transaction.rs:1029` 的 `let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);`，
  取号写把新号写进每块盘的系统配置槽（`transaction.rs:1014` 的注释：「取号会取到的号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的实例代号) + 1」），所以新实例代号恒大于盘上任一自证过的实例代号。
- 取号的调用点：`grep -rln 'next_instance_generation_to_acquire\|acquire_instance(' crates/*/src` 在 core 里只有 `mount.rs` 与 `transaction.rs`；
  `mounted_session.rs` 里没有实例切换的取号（grep 零命中），所以「会话内实例切换」今天没有写者，不在这一轮的对象里。

读者侧：

- 读根环核落点：`recovery.rs:1852` `pub fn read_root_ring_slot<` 在自证过的根不住在公式槽时记成 `NotSelfVerified`（`recovery.rs:1873`），判法 `root_ring.rs:183`
  `pub fn root_sits_in_the_ring_slot_the_formula_gives(`，第 0 代根（txg 0）照 mkfs 种根的三个槽另认。
- 择根核 `replay-prefix/src/lib.rs:3989` `pub fn choose_root_slot(`：换的条件在 `:4015`—`:4016`
  `|| (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0` / `&& candidate_key.instance.0 > chosen_key.instance.0),`。
- 可写挂载的「更新状态读不出」闸（N-配置）：`mount.rs:8860` `fn newer_publish_witness<`，比较三支 `mount.rs:8905` `WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {`、
  记录读不出的一支、`mount.rs:8920` `WitnessedCounterComparison::Undecidable`；判真就在同一个读缓存上重做读阶段一遍（`mount.rs:9002` `fn read_stage_settled_at_most_on_the_one_reread<`），
  仍真则 `mount.rs:9021` `return Err(MountError::NewerStateStillUnreadableAfterOneReread(`。R2 判法 `recovery.rs:5043` `pub fn system_configuration_witness_is_undecidable(`。

## 三、Q1 第 1 问：形甲（同 txg 不同实例）造不出

推导，每步一个出处（读码推的）：

1. 一个根槽装一条根，一次发布只写一个槽：`transaction.rs:570` 取一个 `target`，随后只对这一个槽 `write_at`。根环不镜像。
2. 槽只由 txg 定：`root_ring.rs:172` 区域 = txg mod 3，槽号 = (txg / 3) mod S。写者只有 `transaction.rs:570` 与 `make_filesystem.rs:776` 两处（第二节查法）。
3. 读者只收住在公式槽里的根：`recovery.rs:1852` 的 `read_root_ring_slot` 把住错槽的根记成 `NotSelfVerified`（`recovery.rs:1873`），
   `root_ring.rs:183` 的判法对 txg ≥ 1 的根只认 `target_for_publish(txg) == ring_slot`。所以两条被读者接受的、txg 相同且不为 0 的根必在同一个槽，即读根环次序里的同一项，进不了同一次择根的两个位置。
4. txg 为 0 的根只有 mkfs 写：新实例的第一个 txg 是「某个非负数加一」（`mount.rs:1426` 起的 `checkpoint_txg_after_the_version` 用 `checked_add(1)`），恒 ≥ 1；
   发布路径同实例内加一（`transaction.rs:5839`）。mkfs 把同一个 `root_slot` 字节写进三个区域的槽 0（`make_filesystem.rs:768` 到 `:777` 的循环），三份同字节，实例代号都是 0。
   所以平局只剩这三份同键的根，v1823 与基线、v1825 与基线在它们上同值（v1825 要「实例代号严格低」才换，三份实例相同）。
   这与 D16 `16-发布语义.md:195` 一致：「一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根」。
5. 池级 checker 也有同一条：`crates/singlefs-checker/invariants/02-根环.md:24` 的 I-7.17「根槽位置等于 txg 算出的位置」。

挂载那一刻读坏的根环槽下，这条挡法还成立，而且根本不经过 `highest_root_txg` 和 R2：

- 最坏的一格：txg 最高的那条根 X（txg a）的槽在算第一个 txg 那一读里读坏，它那次发布的记录两份也读不到。
  `highest_root_txg` 不数这个槽，`mount.rs:1419` 取到 a−1 或更小的值；若 X 之前的根与记录在读，第一个 txg = a，新实例的第一次发布写 `target_for_publish(a)`，即 X 自己的槽，把 X **盖掉**。盘上仍是一槽一根。
- 读坏的三种（报读错、非全零自证不过、读回全零）对这一步没有区别：三种在 `highest_root_txg` 和 `choose_root` 里都是「看不见」（第二节），区别只在回收门槛怎么钉（D16 `16-发布语义.md:36`、`:39` 两行），与形甲无关。
- R2 只看系统配置槽，不看根环槽（`recovery.rs:5043`），这没有留缝：形甲的挡法不靠 R2。R2 与 N-配置闸挡的是另一件事——X 已确认时不让新实例在看不到它的情况下往下写，见第四节。
- 盖掉 X 是不是丢已确认的发布，是另一问：X 已确认 ⇒ 系统配置已见证它（D16 `:215` 的顺序里系统配置轮换在根槽 FUA 之后、fsync 等它持久），
  读阶段判 N-配置为真，重读一遍仍真则取号之前拒可写（`mount.rs:9021`）；重读读到 X 的根或记录就不会有「第一个 txg = a」。X 未确认则本来就不在承诺里。

形甲唯一的缝是读者放行的第 0 代例外被手工改字节利用：`crates/singlefs-harness/tests/root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs:5`
写明「同 txg 两条根能并存的只剩第 0 代根照 mkfs 种根的槽另认那几槽」，那个用例把区域 1、2 的槽 0 改写成 txg 0、实例代号 1 与 2。没有任何写者写得出这个形状（第 4 步）；它是造出来测择根核的。

### 冲突一处：C88 的前提与 D16 的条款

并排整行抄（行号是各自文件的行号）：

- `.claude/kb/checks-owed.md:80`（C88）：「设备失而复得会让根环里并存两条同 checkpoint_txg 的自证合法根」
- `.claude/kb/decisions/16-发布语义.md:195`：「一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根」

读码推：C88 立账时（落点核还没进读者）这句是推的读法；今天写者一槽一根，读者又拒错位根，回退之后新时间线复用旧 txg 时写的仍是同一个槽，盖掉旧根而不是并存。
这一格我判 C88 前提在今天的实现下不成立，但它是另一笔账的题面，由主 agent 定要不要改，我不改。

### 什么现象会推翻「形甲造不出」

- 任一条能在盘上留下两条自证过、txg 相同且不为 0、落点公式都认的根的写者或读者行为。具体可查：第二节两处写者之外，`grep -rn 'write_at(' crates/singlefs-core/src` 里出现第三处写根槽；
  或 `root_sits_in_the_ring_slot_the_formula_gives` 对 txg ≥ 1 的根放行了不止一个槽；或第 0 代根被发布路径写过（第一个 txg 出现 0）。
- 一个设备把「读槽 s 返回槽 s' 的内容」做成常态且这个返回通过落点核（落点核本身挡错位读，`recovery.rs:1873`）。

## 四、Q1 第 1、2 问：形乙（实例代号更高而 txg 更低）

### 4.1 盘上要出现的是什么

两条被读者接受的根 X（txg a，实例 i）与 Y（txg b，实例 j），i < j，b < a，且不在同一个槽（a、b 同余 3S 的话 Y 盖掉 X，见第三节）。
Y 是新实例 j 的发布，它的 txg 不小于 j 的第一个 txg（同实例内逐次加一，`transaction.rs:5839`）。
所以必要条件：实例 j 挂载时算出的第一个 txg ≤ b < a，也就是 `mount.rs:1419` 那个 max 把 X 漏掉了：
X 的根槽在 `highest_root_txg` 那一读里看不见，**并且** X 那次发布的每一条记录都不在 `records` 里（记录的 `checkpoint_txg` 同样进 max），
并且 txg 落在 [第一个 txg, a] 之间的其余根与记录也都看不见。

### 4.2 逐个挡法，在「读坏」下还成不成立（读码推的）

1. **没有读故障**：`highest_root_txg` 看得见 X，第一个 txg ≥ a + 1。成立。
2. **一个根槽读坏**（三种读坏任一）：X 的记录写在根之前，屏障隔开（D16 `16-发布语义.md:215`），两块盘同偏移各一份（`transaction.rs:558`、`:559`），记录那一半的 max 仍把 a 带进来，
   或者可写挂载读阶段用记录把 X 重建出来当所选版本（D23 `23-journal的角色与格式.md:471`「崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根」）。
   `highest_root_txg` 不数读坏的槽，这一条留的缝由记录那一半补上；R2 在这里不起作用，它看的是系统配置槽（`recovery.rs:5043`）。
3. **一块盘整块读不出**：可写挂载在读盘之前就拒（D2 `02-RAID条带策略.md:226` 一行里「独占打开池中过半的设备」、可写设备数下限），E165 的合法域也这样摆（`165-…md:627` 一带「历史里的可写挂载在读盘之前就拒」）。
   即使放行：记录镜像在另一块盘，`scan_journal` 逐盘读（`recovery.rs:5059` 起的循环），另一份仍在。
4. **已确认的 X 另有一道闸**：N-配置（`mount.rs:8860`）把系统配置里各盘自证槽的 tail 最大值 c_见证 与所选版本末条记录的计数器比，c_见证 大就判真、重读、仍真则拒（`mount.rs:9021`）。
   每次发布的系统配置轮换写每块盘（`transaction.rs:596` `for index in 0..self.devices.len() {`），所以每块盘最新那一槽都带最新 tail。
   只有读错的故障模型里，c_见证 要么是真最新值，要么每块盘最新槽都没读到，此时「有槽设备报读错且没有一块盘两槽都自证过」，R2 判真（`recovery.rs:5043`）。
   所以漏掉的 X 只可能是**没确认**的发布（根槽 FUA 之后、系统配置轮换持久之前崩了，D16 `:215`）。
5. **没确认的发布一个实例至多一次**：同一实例的发布串行，每次以屏障结尾（`transaction.rs:1652`），下一次发布在上一次返回之后才开始。要叠出两个没确认的发布，只能靠连续崩溃再挂载：每次挂载把上一个没确认的发布当所选版本接着往下写（它的根或记录读得到），写行发布再崩一次。

### 4.3 单故障内造不出，多处同时读坏造得出

形乙的第一个 txg 要被压到 b < a，上面各条一起被绕过：至少要两个 txg 的发布（txg ≥ b 的那些，其中有 a 和 b 之后的一个）在同一次挂载里看不见，
而每个发布的根（不在记录重建的那一例里）、记录的两份镜像都要读不到。**下界 5 处**（1 个根槽 + 两条记录各 2 份）：

| 看不见的东西 | 处数 |
|---|---|
| 最高那条未确认根 Z2 的根槽（择根那一读和算第一个 txg 那一读都读不到） | 1 |
| Z1 的记录（两块盘各一份） | 2 |
| Z2 的记录（两块盘各一份） | 2 |

D22 `22-单元原子性怎么合成.md:185`：「容错承诺是单故障」；E165 实验页 `165-…md:1446`：「journal 记录槽、同一次挂载里两项以上的读故障（SB、RI、RNSB 本身除外）、故障跨两次以上挂载不恢复，都没造」。
所以形乙在承诺内造不出，在承诺外（≥ 5 处读坏）造得出；E165 的界内 0 读数不构成对它的任何证据（它的读故障不碰记录槽，一次挂载至多一项）。

### 4.4 最小历史 H_B（读码推的，逐步标盘上状态）

记号：C = 最后一次已确认的发布，txg c，实例 1，末条记录计数器 k，系统配置 tail = k。S 取默认几何。暖机计划至少还有一次空发布（两块盘、`mount.rs:6225` `let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish_txg)];` 起只覆盖一块盘）。

| 步 | 发生什么 | 盘上之后 |
|---|---|---|
| 0 | 起点 | 根 C(c,1)；记录到 k；tail = k |
| 1 | 挂载 2（实例 2）无故障，写一次发布 Z1（txg c+1，记录计数器 k+1），记录两份落盘、屏障过，**崩在根槽 FUA 之前** | txg c+1 的槽仍空；记录 k+1（实例 2）两份；tail = k |
| 2 | 挂载 3（实例 3）无故障：择根得 C，扫环见记录 k+1，N-配置 c_见证 = k 不大于所选版本的计数器，按 D23 `:471` 由记录重建 Z1 当所选版本；第一个 txg = c+2（`mount.rs:1419`，记录的 txg c+1 + 1）；写行发布 Z2（txg c+2，实例 3，记录 k+2），**根槽 FUA 后、系统配置轮换前崩** | 根 C(c,1)、Z2(c+2,3)；记录 k+1、k+2；tail = k |
| 3 | 挂载 4（实例 4）：这次挂载里下面 5 处每一次读都读不出：Z2 根槽，记录 k+1 两份，记录 k+2 两份。择根得 C；N-配置 c_见证 = k，所选版本 C 的末条记录读得出，k > k 假，不重读不拒；第一个 txg = max(c, c) + 1 = **c+1**；取号 = max(系统配置里的 3, 环里看得见的根的实例) + 1 = 4（`transaction.rs:1029`）；写行发布 Y1：txg c+1，记录计数器 = 看得见的最大计数器 + 1 = k+1（`mount.rs:9505`），**写到记录 k+1 同一偏移，盖掉 Z1 的记录**（`journal.rs:509` `record_offset` 只看计数器）；根写进 txg c+1 的槽。Y1 的根 FUA 落盘后，**暖机第二次发布（txg c+2）的根写之前崩** | 根 C(c,1)、**Y1(c+1,4)**、**Z2(c+2,3)**：实例 4 > 3 而 txg c+1 < c+2——**形乙** |
| 4 | 挂载 5 无故障，择根 | 见 4.5 |

形乙到步 3 的崩溃点那一刻才出现；暖机第二次发布的根一落盘，它写 txg c+2 的槽，把 Z2 盖掉，形乙消失。形乙在盘上存在的窗口 = Y1 的根 FUA 到暖机第二次发布的根 FUA。
若要窗口越过暖机、越过用户的已确认发布：Z 的链再叠长（每多叠一个没确认的发布，坏读多 3 处：根槽 1 + 记录 2），见 4.6。

### 4.5 步 4：下一次择根，三条变体与基线各择哪条（读码推的）

盘上两条相关的根：Z2 = (txg c+2, 实例 3)，Y1 = (txg c+1, 实例 4)；比较式在 `replay-prefix/src/lib.rs:4015`—`:4016`。

| 择根核 | 择谁 | 为什么 |
|---|---|---|
| 基线 | Z2 | txg 为主，c+2 > c+1 |
| v1823 比较不看实例代号 | Z2 | 只在 txg 平局时与基线分开；这里没有平局 |
| v1825 平局取低 | Z2 | 同上 |
| v1826 实例代号为主 | Y1 | 4 > 3 |

所以形乙把 v1826 与基线分开，v1823、v1825 与基线同值。这与 E165 的 KL 读数一致（`165-…md:1296` 的 LQ3 行：KL 上 v1823、v1825 为 0、v1826 为 6；`:1286`：同 txg 两条根「没有对象」）。

已确认的完整发布丢不丢：H_B 里 Z1、Z2、Y1 都没确认，C 是它们共同的祖先，基线择 Z2 或 v1826 择 Y1 都带着 C，**不丢已确认的发布**（推的）。
基线择 Z2 之后能不能走通是另一件事：挂载 4 的分配器按 C 建，不知道 Z1、Z2 的落点（它们的根与记录那一遍都读不到，影子账与预隔离都没有它们），
Y1 写的单元可能盖掉 Z2 引用的单元，择到 Z2 之后走读失败再靠 F2 退回（D23 `:395`「所选根走读失败、系统配置没见证它时退回见证到的那一版」）。这一步没读透，标「推的，没量」。

### 4.6 已确认的发布丢不丢：要更长的链（读码推的）

让形乙越过暖机与用户的已确认发布，要 Z 链更长。设 Y 在崩之前确认了 u 次用户发布（Y 的 txg 依次是 c+1 行发布、c+2 暖机、c+3 … c+2+u 用户发布），
要让最高的没确认根 Z 的 txg a ≥ c+3+u（否则被 Y 自己的发布盖掉），链长 L ≥ u+3（Z1 无根，Z2 到 Z_L 各一条根），同一次挂载里要读不到的处数 = 3L − 1 = 8 + 3u：

| u | 链长 L | 坏读处数 | 盘上出现形乙后的下一次择根 |
|---|---|---|---|
| 0（暖机完成，没有用户发布） | 3 | 8 | 基线择 Z_L（没确认），v1826 择 Y；Y 里没有用户数据，不算丢已确认 |
| 1 | 4 | 11 | 基线择 Z_L，**Y 里 fsync 已返回的用户发布丢**；v1826 择 Y，不丢 |

u ≥ 1 时 N-配置也拦不住：Y 的记录从计数器 k+1 起写，盖掉 Z1 到 Z_{L−1} 的记录，Z_L 的记录（计数器 k+L）留下，k+L 大于 Y 最后一条的 k+2+u，
下一次恢复里 c_见证（= Y 最后一条的计数器）不大于基线所选的 Z_L 的末条计数器，判假，不重读不拒（`mount.rs:8905` 那一支的比较方向）。计数器被新实例接着用，使 tail 对旧时间线的版本失明。
这一格是推的：没有任何已有产物量过，E165 的生成器写行 txg 取条款允许的最小值、读故障不碰记录槽（`165-…md:1449`）。

### 4.7 什么现象会推翻形乙这一节

- 推翻「单故障内造不出」：任一单故障（一个根槽、一条记录的一份、一块盘、一个系统配置槽之一）的历史，让 `mount.rs:1419` 的 max 比盘上某条根的 txg 小至少 2。
  具体可查：读一条根与它的两份记录三者之一的单点故障，第一个 txg 都不会比该根小——除非有第三处不经记录进 max 的 txg 来源，`grep -n 'first_txg' crates/singlefs-core/src/mount.rs` 里 `:9511` 的调用是唯一赋值点。
- 推翻 H_B：步 3 的挂载在读阶段就拒（N-配置、F1、F2 任一道），或写行发布前的某道闸（取号前逐盘核，`mount.rs:5912` 一带）读到记录或根而拒。这三道我只读到比较方向，没有逐支走通，**待实测**。
- 推翻「Z 链可叠」：步 2 的写行发布在 Z1 只有记录没有根的所选版本上被拒（例如 `FirstFileVersionDoesNotFollowTheVersionItBuildsOn` 一类，`transaction.rs`）；D23 `:399` 只对「第一个文件版本」说这条（实现 `transaction.rs:5641` 一带），写行发布不是文件版本，我读到的是不拒，但没跑。

### 4.8 贴原样输出：根槽的写者只有两处（我跑的只读命令）

```
$ grep -rn 'CommitStep::WriteRootRecordForceUnitAccess' crates/*/src | cut -c1-170
crates/singlefs-core/src/transaction.rs:566:            CommitStep::WriteRootRecordForceUnitAccess {
crates/singlefs-core/src/transaction.rs:1659:    writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {
$ grep -rn 'root_sits_in_the_ring_slot_the_formula_gives(' crates/singlefs-core/src | grep -v 'root_ring.rs:3[0-9][0-9]'
crates/singlefs-core/src/root_ring.rs:183:pub fn root_sits_in_the_ring_slot_the_formula_gives(
crates/singlefs-core/src/recovery.rs:1867:            if !root_sits_in_the_ring_slot_the_formula_gives(
crates/singlefs-core/src/mount.rs:2808:            if !root_sits_in_the_ring_slot_the_formula_gives(
```

另两处写盘的 `write_at`：`instance_table_sweep.rs:604` 写的是单元头（`to_erase.placement.slot`），`make_filesystem.rs:804` 写的是系统配置槽，都不是根槽。
`mount.rs:2808` 是重建分配器读根环那一遍的落点核（与 `recovery.rs:1867` 同判法），所以两条读根环的路径都核落点。

## 五、Q1 第 3 问：等价性写成哪一条测试，C650 剩下那一句关成什么

### 5.1 先纠正一处前提：「三条现由 Verus 必红世界与 KL 罩着」对 v1823、v1825 不成立

E165 实验页第 7 次重跑的 LQ3 行（`165-…md:1296`）：「r3 档五种：今天的序 0、平局取低 0、实例代号为主 6、比较不看实例 0、取第一个有效者 4」。
KL 上 v1823（比较不看实例）、v1825（平局取低）都是 0，只有 v1826（实例代号为主）红 6。KA 换成照落点公式摆根的 KL 之后，动态一侧不再罩 v1823、v1825；
它们今天只被 Verus 一侧杀：`verus-must-red-worlds.tsv:155`（「择根·改判定·比较不看实例代号」）与 `:157`（「C650·改判定·择根 txg 平局时实例代号低者赢」），
`mutations.tsv:1616`、`:1618` 是它们在变异表里的行；v1826 是 `verus-must-red-worlds.tsv:158` 与 `mutations.tsv:1619`。
Verus 的 spec 对所有输入（含平局）判，所以三条按 `mutation-sampling.md:16` 的字面「在所有输入上与原式同值」都**不是**等价变异，它们是被形式化一侧杀掉的变异；
缺的只是「合法历史上动态地红」的那一族（C650 剩下那一句）。

### 5.2 形甲造不出 ⇒ v1823、v1825 在合法状态上同值；留档的测试是三件，缺一不可

单独任何一件都不够：核引理只证「前提下同值」，前提要由读者侧和写者侧两件给。

1. **读者侧（singlefs-core 的性质测试，轻档）**：对每区槽数 S 取 mkfs 允许的各值、txg 取 [1, 6·3S]，`root_sits_in_the_ring_slot_the_formula_gives(txg, 槽)` 为真的槽恰好一个，
   就是 `target_for_publish(txg)`；txg 为 0 时为真的槽恰好是 `genesis_root_ring_slots()` 的三个。这钉「读者接受的根里，txg ≥ 1 的同 txg 必同槽」。
   已有的相邻用例在 `root_ring.rs:300` 起（只取了 S = 8、txg 4 一个点加第 0 代根），不够；要穷举。
2. **写者侧（harness 的池级断言）**：在一段含多次挂载、崩溃、写行与暖机的合法历史上，记下每一次对根槽的写（txg、槽、实例代号），断言
   ① 写入的 txg 都 ≥ 1 且槽 = `target_for_publish(txg)`；② txg 为 0 的根只有 mkfs 写（实例代号 0、三份同字节）。第 0 代例外是读者放行的，
   `root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot.rs:5` 就是手改成「txg 0、实例 1 与 2」才得到平局，写者侧断言要把这一洞补上。
3. **核侧（Verus 引理，进 `crates/singlefs-replay-prefix`）**：前提 P =「任两个自证过的槽，若择新键的 txg 相同则键完全相同」，结论 = `choose_root_slot` 与 v1823、v1825 的比较式交回同一个槽。
   必红：去掉 P 里「键完全相同」一句，引理在 txg 平局而实例不同的例子上必须红（这一格就是 v1823、v1825 与基线分开的地方）。

这三件合起来才是「v1823、v1825 在合法域上与基线同值」；记法写「合法域上等价、全输入域上被 Verus 必红世界杀」，不写成无条件的等价变异（`mutation-sampling.md:16` 的字面不满足）。

### 5.3 v1826 不能留档为等价

v1826 在 KL 上红 6，在 Verus 里红，而形乙在单故障承诺内造不出、承诺外（≥ 5 处读坏）造得出（第四节）。所以：

- **单故障承诺内**：没有一条合法历史让 v1826 与基线分开，对象为空。
- **承诺外**：H_B 就是分开它们的取样点（`mutation-sampling.md:17`「取样点不敏感」要补的那个敏感取样点），E165 今天没有这一族（读故障不碰记录槽、一次挂载一项）。

### 5.4 C650 剩下那一句该关成什么

C650 原句（`checks-owed.md:394`）：「还欠的是合法历史上会让三种改坏择根序红的这一族状态」。读码推的结论：

| 变异 | 关成 | 依据 |
|---|---|---|
| v1823 比较不看实例、v1825 平局取低 | 「合法历史里无对象」：动态一侧不再欠；形式化一侧由必红世界 `:155`、`:157` 杀；5.2 的三件测试留档 | 第三节、5.2 |
| v1826 实例代号为主 | 「单故障承诺内无对象，动态一侧由 KL（非合法域，明标）与必红世界 `:158` 罩；承诺外形另立一笔」 | 第四节、5.3 |

「另立一笔」的内容：H_B 的最小历史（4.4）、坏读处数 3L−1、E165 要补的一族（一次挂载里允许记录槽与根槽同时读坏、写行 txg 取实现的取法而不是条款允许的最小值）。
这一笔要不要立、立不立进承诺，由主 agent 与用户定（D22 `:185` 对 C583 那一形就是写成「不罩」的条款）；我不替它选。

## 六、Q3：判据属「判据自己也会写错」哪一种，改成什么

C650 的判据（`checks-owed.md:394`「会红的检查」一格）：「改坏择根序（同 `checkpoint_txg` 平局取低、实例代号为主这类）这一族要红」。
`evidence-discipline.md:122` 起的四种形态，我逐种核（读码推的）：

| 形态 | 对得上吗 |
|---|---|
| 打中不分辨臂 | 否。这里没有「一格打中让每条臂一起出局」，是一族状态根本造不出 |
| 判别子观测不到（`evidence-discipline.md:125`） | **最近，但不是原样**。判据要被判的系统（择根核）要区分「基线」与「变体」，而它在合法历史上做决定那一刻看到的东西，两者逐条相同：v1823、v1825 只在平局时与基线分开，平局在合法写者下不出现。差别是：表里这一种说的是「被判系统区分两种情形」，这里是「判据要求的输入不存在」 |
| 打中归错了判据 | 否 |
| 改法碰不到打中的格 | 否 |

所以答：**不是四种里任何一种的原样，最近的是第二种「判别子观测不到」**；它在 C650 里的具体形状是**一条存在性前提被当成已知**：
C650 同一行把这三条记成「取样点不敏感」（`mutation-sampling.md:17`），那一类的处方是「补一个敏感的取样点」，默认敏感点在合法域里存在。
对 v1823、v1825 它不存在，应落在 `mutation-sampling.md:16` 那一类（域限定后的同值）；对 v1826，承诺内同样不存在、承诺外存在（H_B）。

改成（按变异分开写，每一格写明它修的是哪一格，`evidence-discipline.md:127` 那一条的要求）：

1. v1823、v1825：判据改为「形式化一侧由必红世界杀（`:155`、`:157`），动态一侧在合法历史上无对象；无对象由 5.2 的三件测试证」。修的是 v1823、v1825 那两格。
2. v1826：判据改为「承诺内：KL 与必红世界罩，合法域无对象；承诺外：H_B 另立一笔」。修的是 v1826 那一格。
3. 总句「这一族要红」删去，换成对每条变异各写一格的杀伤来源；不再有一句话同时要三条红。

这是改判据，`evidence-discipline.md:125` 那一行的处方是「改完是新一轮」：这一轮（Q1 的正推）就是在判据改写前先证对象存不存在，改写本身要经攻方与判决再走一遍，我没替主 agent 判。

什么现象会推翻这一节：
- 推翻「v1823、v1825 无对象」：第三节的任一推翻现象成立。
- 推翻「v1826 承诺内无对象」：第 4.7 节的单故障反例。
- 推翻「不是原样的四种」：有人能把「合法域里没有取样点」归进四种之一并说明打中的格是哪一格；我没找到。

## 七、要主 agent 跑的命令

我没有往统一任务服务交任何任务。下面三项是只核已有读者侧挡法，都不重型。范围声明按 `.claude/hooks/run-scope-guard.sh` 文件头的 target 写法，主 agent 核一遍再交。

1. 读者侧落点核（形甲的读者侧挡法还在）。起：仓根 `/home/fy5090/code/singlefs`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 root_ring_readers_take_a_root_outside_its_formula_slot_for_a_bad_read; 范围=1 个测试目标; 为什么=核形甲的读者侧挡法：错位根按读坏处理' task-server/task submit -- cargo test -p singlefs-harness --test root_ring_readers_take_a_root_outside_its_formula_slot_for_a_bad_read`
   看：`TASK_SERVER_RESULT` 行 `exit=0`；`task-server/task log <号>` 里 `test result: ok`、`0 failed`。
2. 第 0 代例外是手改字节才得到平局。起：同上。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot; 范围=1 个测试目标; 为什么=核同 txg 平局只在第 0 代手改的根环上' task-server/task submit -- cargo test -p singlefs-harness --test root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot`
   看：同上。
3. 落点核单测：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-core 的 lib 测试里名字含 a_root_sits_only_in_the_ring_slot_its_txg_targets 的一条; 范围=1 个测试目标; 为什么=核落点公式的读者判法' task-server/task submit -- cargo test -p singlefs-core --lib a_root_sits_only_in_the_ring_slot_its_txg_targets`
   看：`test result: ok. 1 passed`。

**待做、我做不了的**（等主 agent 派实现员或实验设计员，不是命令）：
- H_B 的实测：在 `crates/singlefs-harness/tests/` 加一条用例，用内存池和读故障包装（相邻先例 `warm_up_journal_counter.rs`），按 4.4 的步 0 到步 4 摆；
  步 3 的 5 处读不出（Z2 根槽、记录两个计数器各两份）。判据：步 3 的挂载成功、盘上出现 Y1(c+1,4) 与 Z2(c+2,3)，`choose_root` 与三条变体各择谁与 4.5 的表相同。
  在这条用例出结果之前，形乙「造得出」只是读码推的。
- 5.2 的三件测试是新写的，不是现成的。

## 八、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1.1 形甲，一般情形 | 造不出（读码推的） | 一槽一根、槽只由 txg 定、读者拒错位根；平局只剩第 0 代三份同字节的根（`transaction.rs:570`、`root_ring.rs:172`、`root_ring.rs:183`、`make_filesystem.rs:776`） |
| Q1.1 形甲，挂载那一刻读坏的根环槽 | 挡法照样成立（读码推的） | 挡法不经 `highest_root_txg` 与 R2：最坏是新根写进读坏的那个槽、盖掉它 |
| Q1.1 形乙，无故障或单故障 | 造不出（读码推的） | 记录与根双重进 `mount.rs:1419` 的 max，记录两份镜像，已确认的发布另有 N-配置闸与 R2 |
| Q1.1 形乙，同一次挂载 ≥ 5 处读坏 | 造得出（读码推的，没实测） | 最小历史 H_B：三次挂载、两次崩溃；盘上 Y1(c+1,实例4) 与 Z2(c+2,实例3) 并存 |
| Q1.2 下一次择根 | 基线、v1823、v1825 择 Z2，v1826 择 Y1（读码推的） | 比较式在 `lib.rs:4015`—`:4016` |
| Q1.2 已确认的完整发布丢不丢 | H_B 不丢；Y 再确认 u ≥ 1 次用户发布时，基线择没确认的 Z_L，Y 里已确认的丢，坏读 ≥ 11 处（推的，没量） | 计数器被新实例接着用，使 N-配置对旧时间线失明 |
| Q1.3 v1823、v1825 | 合法域上与基线同值，留档三件测试；全输入域上不是等价变异（被 Verus `:155`、`:157` 杀） | 5.2 |
| Q1.3 v1826 | 不是等价变异；承诺内无对象，承诺外 H_B 是敏感取样点 | 5.3 |
| Q1.3 C650 剩下那一句 | v1823、v1825 关成「合法历史无对象」；v1826 关成「承诺内无对象、承诺外另立一笔」 | 5.4 |
| Q3 判据 | 不是四种原样，最近「判别子观测不到」，具体是存在性前提被当成已知；改成按变异分开写的杀伤来源 | 第六节 |
| 背景材料前提 | 两处与读码不合：`choose_root` 不按实例表滤抛弃；KL 不罩 v1823、v1825 | 第一节 |
| C88 前提 | 与 D16 `:195` 冲突，今天实现下前提不成立 | 第三节，我不改，交主 agent |

### 本轮没读透的、会影响结论的

- 步 3 之后基线择 Z2 走不走得通（Y1 的分配是否盖掉 Z2 的单元）：没读到底。
- 步 3 的挂载在 F1、F2 与取号前逐盘核三道闸上是否放行：只读到比较方向，没逐支走通。
- 4.6 的 u ≥ 1 一格：没有任何产物量过。
- 读阶段的读缓存（`mount.rs:9007`）与算第一个 txg 的直读（`mount.rs:1412`）之间能不能只一边读到：4.4 的 H_B 不依赖它（假设「这次挂载里每一次读都读不出」），但它是比 H_B 更省坏读处数的路径候选（根的直读那一遍坏而读缓存那一遍好，所选版本是 Z2），我没展开，交攻方腿。

## 九、没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决。
- 没往统一任务服务交任务，判决要引的数没复跑；第七节三条命令由主 agent 交。
- 没写模型。理由：形甲的推导是结构论证（槽由 txg 定），拿我自己写的小模型验它只会复述我的前提；形乙需要真代码上的读故障，只能是入库用例（第七节「待做」），不是我写的副本。
- 没读禁读清单里的文件（攻方腿这一轮的产出）。
- 没读 E165 产物原文（`research/results/` 与 `git show` 的 `.out`），E165 的数都引实验页，行号是实验页自己的。
