# m3-close-b19-e165-r1 攻方腿报告（Opus）

日期：2026-11-05。立场：假设「两种根环在合法写者下造不出、三条变体全部合法状态同值」是错的，造历史去打穿它。攻击面：正文第三节 Q2 三问。
模型的读数一个都还没有：子 agent 不交任务，模型与复跑命令在「要主 agent 跑的命令」一节；本报告里凡是写「推的」的，都等主 agent 跑出来再核。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-close-b19-e165-r1-opus-model/`，两份文件：

| 文件 | sha256 | 是什么 |
|---|---|---|
| `e165_attack_form_b.rs` | `1364e3ba1884396b1e4a05cc75bf6d752dcbfe03930c4672e2054e398e7a5170` | harness 档测试目标源码（复跑时拷进副本的 `crates/singlefs-harness/tests/`），两个用例：`e165_attack_probe`（对照 + 一格全量 + 各格数状态、打估时）、`e165_attack_full`（`#[ignore]`，全量 25 格） |
| `tools/rerun.sh` | `e7086f9caa1195b31f4cdb47d97f50dcba76cb289f5027e36e082b29b79496ca` | 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`：拷冻结副本、放进测试目标、先跑估时，估出全量不超过 2400 秒才跑全量 |

复跑：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus/tree --work <新的草稿目录>`。
冻结副本：本轮没有主 agent 建的 `<轮>-frozen/`，我把主工作区的 `crates/`、`prune-server/`、`Cargo.toml`、`Cargo.lock`、`.cargo/`、`.claude/heavy-thread-default` 拷进 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus/tree`（48M，已在该目录 `.deliverables` 写成「交接」），被判的五份源码 sha256 与 `research/prompts/m3-close-b19-e165-r1-snapshot/crates-sha256.txt` 逐份相同（`recovery.rs`、`mount.rs`、`transaction.rs`、`root_ring.rs`、`singlefs-replay-prefix/src/lib.rs`，我用 `sha256sum` 现比过）。

## 各格判定一览

| 格 | 判定 | 依据的种类 |
|---|---|---|
| Q2.1 形甲（同 txg、不同实例同进择根） | **造不出**，与故障、实例代号、写者次序都无关：两条同 txg（非 0）的根只能住同一槽，一次择根每槽读一次 | 读码（落点公式 + C743 错位读 + 每槽一读），推的；模型里另有一个错位写者对照，待跑 |
| Q2.1 形乙，正文的前提原样（只读坏根槽与记录、系统配置两槽都自证过） | **造不出**：挡它的不是 R2，是判据 N-配置（系统配置见证的 tail 比所选那一版新就重读一次、仍新就拒）；N-配置 把「读坏的根」压到至多一条（崩在轮换之前那一次），新实例第一次发布的 txg 恰好等于它，落进同一槽、把它盖掉 | 读码，推的；模型格 `roots_hidden=1/2 × newest_system_configuration_fault=none` 待跑 |
| Q2.1 形乙，系统配置最新那一槽读回全零或读得出而自证不过（R2 不数这两种） | **打中（推的，待模型）**：最小历史见第一节；盘上出现 (实例 1, txg 6) 与 (实例 2, txg 5) 同在，基线、v1823、v1825 择 (1,6)，v1826 择 (2,5) | 读码推出的历史；模型格 `roots_hidden=2 × newest_system_configuration_fault=all_zero/not_self_verified` 待跑 |
| Q2.1 系统配置那一槽设备报读错 | 造不出：R2 拒 | 读码，推的 |
| Q2.1 根槽三种读坏（设备报错、全零、不是全零而自证不过） | 三种同一结局：`highest_root_txg` 与择根对三种一样跳过 | 读码，推的；模型三种各一列 |
| Q2.2 回退、清扫、被抛弃判在择根之前筛不筛 | **不筛**：`choose_root` 不按实例表滤（背景材料第二节「自证过、未被抛弃的摊成择新键」与代码对不上，见第四节）；被抛弃判只在择根之后按所选那条根的表用 | 读码 |
| Q2.3 v1823、v1825 | **全部经 `choose_root` 走得到的状态上同值，不只 E165 界内**：它们只在同 txg 异实例时与基线不同，而形甲在择根入参里不出现；只剩第 0 代根三份同 txg，那三份实例代号都是 0 | 读码，推的；该记等价变异、把等价性写成一条测试（第三节） |
| Q2.3 v1826 | **界外不同值**：在第一节那段带故障的合法历史上与基线择得不同 | 推的，待模型 |
| 附带打中（不在三问里，线索） | 同一段故障下可写挂载丢掉已确认的 C（txg 5）：C331（择根倒挂压过已确认的写） 那一形绕过 R2 | 推的，模型 `cell_mount` 行 `h2_mount=ok … effective=(1,4)` 待跑 |

## 一、Q2.1：读坏的根环槽能不能让新实例第一次发布不大于那条根、读回来成形甲或形乙

### 形甲：结构上造不出（推的，读码）

- 一条根只写进一槽，槽由 txg 定：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:172`）、「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:173`）。
- 读一槽时自证得过、却不住在公式那一槽的根当读坏：「if !root_sits_in_the_ring_slot_the_formula_gives(」（`crates/singlefs-core/src/recovery.rs:1867`），判法「target_for_publish(checkpoint_txg, slots_per_region) == ring_slot」（`crates/singlefs-core/src/root_ring.rs:192`）。
- 择根每槽读一次、每槽至多一条根交给核：「RootRingSlotReading::SelfVerified(root) => Some(root),」（`crates/singlefs-core/src/recovery.rs:1904`）。
- 所以同一次择根里两条 txg 相同（非 0）的根必须住同一槽，不可能；txg 0 只认 mkfs 种的三槽，三份都是 mkfs 写的实例 0。实例代号、读故障、回退、崩溃都改不了这一点。模型的 `writer_variant_same_txg_in_another_slot` 对照把 (2,6) 写进 txg 7 那一槽，预期读成坏槽、形甲不出现（待跑）。
- 唯一「同 txg 不同实例」在盘上并存的样子是一条根与一条记录（新实例第一次发布崩在记录之后、根槽之前，记录落回被盖住的那个计数器），它不经择根核，三条变体管不到（第六节「没打中的形状」）。

### 形乙：正文前提下被 N-配置 挡住（推的，读码）

正文第二节把挡法写成 R2，漏了判据 N-配置。它在 D23（journal 的角色与格式） 已定项 14：「不是判不出时取全部自证过的槽 journal tail 的最大值 c_见证」（`.claude/kb/decisions/23-journal的角色与格式.md:397`），代码「} => self.witness.witnessed_journal_counter > selected_version_last_record_counter,」（`crates/singlefs-core/src/mount.rs:506`），重读一次仍为真就拒「StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {」（`crates/singlefs-core/src/mount.rs:9023`）。每次发布末尾都在每块盘轮换一槽系统配置、tail 写这次发布的末条计数器：「journal_tail: last_counter_of_this_publish,」（`crates/singlefs-core/src/transaction.rs:2370`）、「for index in 0..self.devices.len() {」（`crates/singlefs-core/src/transaction.rs:596`）。

推法：设最后一次完整发布是 L（轮换与末尾屏障都持久），盘上的根至多到 L + 1（在飞那一次崩在根槽 FUA 之后、轮换之前）。系统配置两槽都自证过时 c_见证 ≥ L 的末条计数器，所选那一版 M < L 就判真、拒；于是 M ≥ L，`first_txg_of_new_instance` = 「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1419`）≥ L + 1 ≥ 盘上每条根。读坏的若是 L + 1 那一槽，新实例第一次发布恰好 txg = L + 1，按落点公式落进同一槽、把旧根盖掉。正文那个前提（只坏根槽与记录、R2 不触发）下，新实例第一次发布可以**等于**那条根的 txg，但盘上不会出现形甲或形乙。
这与 W3 第三轮判决里「同 txg 的新根落在被抛弃的根同一槽、把它盖掉，撞号 0」一致（那是当时的读数，没重跑，线索）。

根槽三种读坏在这里同一结局：读不出走「RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => continue,」（`crates/singlefs-core/src/recovery.rs:1809`），全零与不是全零而自证不过都走「RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {}」（`crates/singlefs-core/src/recovery.rs:1810`），`highest_root_txg` 都不数。

### 形乙：N-配置 被系统配置那一槽的「全零 / 自证不过」读坏绕过（打中，推的，待模型）

R2 只数设备报读错：「None => slots_reporting_a_read_error += 1,」（`crates/singlefs-core/src/recovery.rs:5004`）、「.any(|slots| slots.slots_reporting_a_read_error > 0)」（`crates/singlefs-core/src/recovery.rs:5048`）；读得出而自证不过的槽只是丢掉，c_见证 也只取自证过的槽：「.flat_map(|slots_of_this_device| slots_of_this_device.self_verified.iter())」（`crates/singlefs-core/src/mount.rs:8887`）。这一种是有意放过的：「撕裂的系统配置轮换（读得出、自证不过）不拒：取号写重写坏的那一槽。」（`crates/singlefs-core/src/transaction.rs:931`）。最新那一槽一次读回全零或读得出而自证不过，c_见证 就退回上一次发布的 tail，N-配置 多放过一条读坏的根。

**最小历史**（两块盘，区域归属 [盘 0, 盘 1, 盘 0]，与 harness 共用搭建同几何；模型 `build_starting_history` 与 `fault_ranges_of` 逐步照它造）：

| 步 | 做什么 | 盘上此刻 |
|---|---|---|
| 1 | mkfs；实例 1 取号、暖机 txg 1、2；新池新建文件 txg 3；覆盖写 B（txg 4）、C（txg 5），都完整发布、已确认 | 根 (1,1)…(1,5)；每块盘两槽系统配置 tail = C、B 的末条计数器 |
| 2 | 覆盖写 D（txg 6）：单元、记录、根槽 FUA 持久，系统配置轮换没持久就崩（D 没确认） | 多一条根 (1,6) 在 txg 6 那一槽、多一条 D 的记录；系统配置仍是 {C, B} |
| 3 | 可写挂载，这一次挂载里读故障：txg 5、6 两槽根读坏，C、D 的记录在两块盘都读坏，两块盘最新那一槽系统配置（tail = C）读回全零或读得出而自证不过；三类根读坏任取一种 | 择根 (1,4)；c_见证 = B 的计数器 = 所选那一版末条 ⇒ N-配置 为假；R2 为假（没有设备报读错）；取号得 2；`first_txg` = max(4, B 的记录 txg 4) + 1 = 5 |
| 4 | 实例 2 写行那一次发布 txg 5：根 (2,5) 按公式落 txg 5 那一槽（盖掉 (1,5)）、记录落回 C 那个计数器的槽、系统配置轮换 | 根环里 (1,6) 还在，(2,5) 在 |
| 5 | 暖机要让新实例的根落遍两块盘：txg 5 落盘 0，下一次 txg 6 也落盘 0（会盖掉 (1,6)），txg 7 才落盘 1。崩在 txg 6 的根槽 FUA 之前（写行那一次完整持久之后或之间都算） | **形乙**：(1,6) 与 (2,5) 同时自证过 |
| 6 | 读故障消失，下一次恢复择根 | 见下表 |

| 择根序 | 择哪条 | 已确认的发布 |
|---|---|---|
| 基线（`choose_root`，txg 为主） | (1,6) | 推的：C 在这条时间线上（D 叠在 C 之上），C 不丢；实例 2 什么都没确认过（第 5 步崩在挂载返回之前），也不丢确认过的东西。但 (1,6) 那棵树的单元可能已被实例 2 写行那一次重用（实例 2 按 B 那一版的账分配，C、D 的单元在它看来是空的），那样基线这次挂载会拒或退回——模型 `mount_after` 一列给真结局 |
| v1823（不看实例代号） | (1,6) | 同基线 |
| v1825（平局取低） | (1,6) | 同基线 |
| v1826（实例代号为主） | (2,5) | 推的：(2,5) 是建在 B 上的写行那一版，**丢已确认的 C**（第 3 步那次挂载已经把它丢了，v1826 把丢失坐实；基线反而把它找回来） |

能不能更深（实例 2 带着用户已确认的写坐在 (1,6) 之下）：要读坏三条根（txg 4、5、6），c_见证 就得退到 txg 3 的计数器，系统配置两槽 {C, B} 都得读坏，那块盘一槽自证过的都没有，择系统配置或取号那一道拒（`self_verified_system_configurations_of_every_device` 的「某块盘一份都读不出」那一拒）。所以这一机制下形乙里实例代号高的那条至多是写行那一次（推的）。单盘池没有暖机把 txg 6 写掉那一步，挂载可以带着形乙返回，之后第一次用户发布 txg 6 才盖掉 (1,6)（推的，模型只跑了两盘几何）。

## 二、Q2.2：回退、清扫、被抛弃判会不会在择根之前把那一形筛掉

不会（读码）。

- `choose_root` 读三个区域全部槽，自证过的一律摊成择新键交核，没有一处看实例表：「RootRingSlotReading::SelfVerified(root) => Some(root),」（`crates/singlefs-core/src/recovery.rs:1904`）、「choose_root_slot(&choice_keys_in_ring_order).map(|chosen_slot| {」（`crates/singlefs-core/src/recovery.rs:1921`）。核自己也只比 (txg, 实例代号)：「|| (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0」（`crates/singlefs-replay-prefix/src/lib.rs:4015`）。
- 被抛弃判「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:2230`）用在回退候选集、影子账、抬 F 上限与 F 生效值，用的是择出来的那条根指着的表。第一节的形乙里实例 2 的表有行 (1, 4, W)（「selected_root_txg: effective_root.checkpoint_txg,」，`crates/singlefs-core/src/mount.rs:9547`），按它 (1,6) 是被抛弃的；可是基线择到的是 (1,6) 自己，它的表里没有实例 2 的行，这张表永远轮不到判 (1,6)。所以被抛弃判在这一形上是**择根之后**才用、而且用的是错的那张表，不是择根之前的筛子。
- 清扫（`instance_table_sweep`）只删实例表的行、不碰根环；管理员回退只在候选集里挑目标、新实例照「环里最大 + 1」取 txg，都不改择根的入参。
- 重放候选只认与所选根同实例、txg 更大的记录：「record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0」（`crates/singlefs-replay-prefix/src/lib.rs:154`），所以基线择 (1,6) 之后实例 2 那条 txg 5 的记录不施加、也不把 (1,6) 换掉。

## 三、Q2.3：三条变体是只在 E165 界内同值，还是全部合法状态同值

**v1823、v1825：全部经 `choose_root` 走得到的状态上同值（不止界内），推的。** 两条只在「txg 相同、实例代号不同」时与基线不同；按第一节形甲那一段，核的入参里同 txg 的两个 Some 只能都是第 0 代根，三份都是实例 0、键逐字相同，三条序都留读根环次序里第一份。核的产品调用方只有 `recovery.rs:1921` 那一处（`grep -rn 'choose_root_slot(' crates` 现查：其余四处在 `singlefs-replay-prefix/src/lib.rs` 自己的用例里）。所以按 `.claude/rules/mutation-sampling.md` 那张表，这两条在系统层是第二类（等价变异），不是第三类：KA / KL 与 Verus 必红世界直接给核喂同 txg 异实例的键，那是 `choose_root` 交不出的入参，它们让变体红，证明的是「核的序写对了」，不是「系统会撞上这一形」。
该留档的等价性测试（推的，没写）：一条 Verus 引理或池级断言，形如「入参里每个 Some 键的槽下标等于 `target_for_publish(txg)` 的下标（txg 0 时在三个第 0 代槽之一且实例 0），则基线、v1823、v1825 交回同一下标」；判别力是把 `read_root_ring_slot` 的错位读那一判（C743）拿掉，用错位写者造两槽同 txg 异实例，三条就分开。

**v1826：界外不同值（推的，待模型）。** 第一节那段历史上它与基线择得不同。它的前提是 E165 合法一族没有生成的东西：系统配置最新那一槽这一次读坏（全零或自证不过）、同一次挂载里两条根与它们的记录读坏、崩在新实例暖机之前。E165 实验页自己写过界：「只在界内、合法、不带故障的 K 族」（`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:608`）。所以 v1826 是第三类（取样点不敏感）：要补的是这个取样点，不是留档。

## 四、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨：三条变体里只有 v1826 在这一形上与基线不同，v1823、v1825 不动 |
| 被判的系统当时看不看得到判别它的东西 | 第 3 步那次挂载看不到：读回全零 / 自证不过的最新那一槽，与一次撕裂的轮换在挂载那一刻逐字节分不开（撕裂的是被覆写的旧槽，读坏的是最新槽，挂载读不出槽里本来是什么），这正是 R2 有意放过的那一种。第 6 步的择根看得到：两条根都自证过，(实例, txg) 都在键里 |
| 满足的是判据字面的哪一个分句 | C650 那一句「合法历史上会让三种改坏择根序红的这一族状态」里的「合法历史」：历史里每一步都是今天的写者与 D16（发布语义）、D23（journal 的角色与格式）、R2 口径内的读故障与崩溃；它只让三种里的一种红 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 正文没有给改法；我提的改法见下一节，逐格标「推的」 |

## 五、改法（只在我的推理上过、被攻过零轮；模型也还没跑）

| 改法 | 修第一节形乙那几格 | 修附带打中（C 丢） | 代价 | 标 |
|---|---|---|---|---|
| 甲：`first_txg_of_new_instance` 把挂载那一刻读坏的根槽当作住着它能住的最大 txg（同 D16（发布语义） 已定项 1「抬 F 的上限」把读坏槽当可能非空有效根的口径，`.claude/kb/decisions/16-发布语义.md:38` 那一行），新实例第一次发布越过它 | 中（推的）：(2,·) 全都大于 (1,6)，形乙不出现，v1826 与基线又同值 | 不中：C 照旧被那次挂载丢掉 | txg 跳号至多一圈（3S）；I-8.14 那条「射程外」可以收回 | 推的 |
| 乙：读阶段见到一块盘有一槽系统配置读得出而自证不过或全零，就照 R2 重读一次、仍坏按判不出拒可写 | 中（推的）：第 3 步那次挂载拒，盘上逐字节不变 | 中（推的） | 与 C596（见证判不出的系统配置盘面只剩只读挂载） 定案冲突：一次普通断电撕裂轮换之后也拒（虽然取号写本可重写那一槽），等于把 R2 收严回取甲 | 推的 |
| 丙：择根之前按实例代号最高那条根的表滤被抛弃的根 | 让基线择 (2,5)，与 v1826 同值 | 不中，且把 C 的丢失坐实（与 v1826 同害） | — | 推的，不推荐 |

## 六、背景材料与代码对不上的地方（核过，读码）

1. 第二节「自证过、未被抛弃的摊成择新键」：`choose_root` 不滤被抛弃的根（第二节引的 `recovery.rs:1904`、`recovery.rs:1921`）。按它写的 Q2.2 前提不成立。
2. 第二节、第三节把「R2 不触发」当成唯一的门：真正挡正文前提的是 N-配置（第一节）。正推腿若按 R2 推「造得出」，会和读码对不上；若按 R2 推「造不出」，理由错了。
3. 第二节「C331（择根倒挂压过已确认的写） 由它挡」：附带打中那一形（第一节第 1–3 步，c_见证 被一次「全零 / 自证不过」读坏压低）里，可写挂载丢掉已确认的 C、R2 不响。I-8.14（新实例第一个 txg 高于更早实例的根与记录） 的登记行自己承认这一族存在：「射程外：挂载那一刻一时读不出、今天读得出的根或记录（瞬时读故障不留在盘上），镜像上与改坏的 f_k 逐字节相同，判不了」（`crates/singlefs-checker/invariants/03-journal.md:21`）；harness 共用搭建造被抛弃根时就是清零最新那一槽系统配置：「系统配置没见证到它（清零、不写回；见证在时 C554 乙拒可写，造不出被抛弃的根）」（`crates/singlefs-harness/tests/common/mod.rs:827`）。C764 那一行记过产品上的同类镜像：「在产品上误报 1 037 260 张（根与记录在挂载时都读不出那一形）」（`.claude/kb/checks-owed.md:471`，当时的数，没重跑，线索）。
4. D23（journal 的角色与格式） 已定项 14 注 3 要的是「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`.claude/kb/decisions/23-journal的角色与格式.md:373`）；代码只数这一刻自证过的根（`recovery.rs:1809`、`recovery.rs:1810`），条款字面在读坏的槽上做不到，条款没写读坏的槽怎么算。

## 七、没打中的形状（试过哪些、范围多大）

全部是读码推的，模型里对应格待跑：

- 只读坏最高一条根与它的记录（模型 `roots_hidden=1`，三种根读坏 × 四种系统配置读坏 = 12 格）：新实例第一次发布 = 那条根的 txg，落同一槽盖掉，形甲形乙都不出现。
- 读坏两条根、系统配置不读坏或设备报读错（6 格）：N-配置 或 R2 在取号之前拒，一个字节不写。
- 两盘里一盘系统配置设备报读错、另一盘两槽都自证过：R2 为假，但轮换每次写每块盘，另一盘的 tail 一样新，N-配置 照样拒。
- 轮换写到一半崩（盘 0 写了、盘 1 没写）：c_见证 取各盘最大，不降。
- 崩溃撕裂轮换：撕的是被覆写的旧槽，最新那一槽完好，c_见证 不降；所以纯崩溃（不带读故障）造不出形乙。
- 读阶段经读缓存择根、`first_txg` 直接读盘（`crates/singlefs-core/src/mount.rs:1419` 那一句的输入在 `crates/singlefs-core/src/mount.rs:9511` 读 `devices`）：一槽择根时读得出、算 `first_txg` 时读不出，`first_txg` = 所选那条根的 txg，新根落同一槽盖掉它，不成形。
- 管理员回退：新实例照环里最大 + 1，不成形。
- 吞写（C600）代替瞬时读坏：吞掉 C 那次轮换写能把 c_见证 持久地压低，应当造出同一个形乙；没建模型。
- 用户动作：形乙只活在实例 2 挂载里暖机 txg 6 那一笔根槽写之前；模型把这次挂载写流的崩溃状态逐个列（每段真子集 + 全部持久），挂载返回之后的用户动作只会更早盖掉 (1,6)，不放开扫也不改「不中」那几格的结论（推的）。

## 八、这条腿自己的限度

- 模型一次都没编过（子 agent 不交任务）；照 harness 共用搭建与 `warm_up_journal_counter.rs` 的调法写的，编不过要承接的腿修。
- 没调 `enumerate_crash_states`：它经判定库与来历表、只挂固定观察者，接不出「崩溃之后再挂载一次」。我在模型里照同一个枚举域自己列（`for_each_crash_state`，个数等于 `closed_form_state_count`），**没有**原地覆写的撕裂第三态；也没调 `crash_state_count_with_torn_in_place_overwrites`（在 checker 档包里）。起点历史 D 那一步只取一个崩溃状态（根槽持久、轮换没持久），没枚举 D 的写流。
- 读故障整段、整次挂载都在，被写过的段才撤；没扫「只坏第一次」那种瞬时读错（那样 N-配置 的重读会读出新状态、挂载照常择到新根，推的）。
- 只跑两盘几何，单盘没跑。变体是在模型里照 `crates/mutations.tsv` 三行的改法另写的择根序，没打变异补丁；真 `choose_root` 只当基线对拍（`core_disagrees_with_local_baseline` 一列）。
- 每个状态跑池级 checker 与一次基线可写挂载，估时由 `e165_attack_probe` 打；超过 2400 秒 rerun.sh 不跑全量。

## 要主 agent 跑的命令

副本 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus/tree` 里已放好测试目标（`crates/singlefs-harness/tests/e165_attack_form_b.rs`，与模型目录那一份逐字节相同），`.deliverables` 写了「交接」。三条都是 harness 档，不是重型。

1. 估时（先跑）。起点目录：上面那个副本。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 e165_attack_form_b 的估时用例; 范围=1 个测试目标; 为什么=攻方腿模型先跑对照与一格全量估全量挂钟，超过四十分钟就不跑全量' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus/tree -- cargo test --release --offline -p singlefs-harness --test e165_attack_form_b -- --nocapture --test-threads=1 e165_attack_probe`
   看：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `OPUSMODEL name=control kind=writer_variant_first_txg_not_above_the_unreadable_root … verdict=red_as_expected`（对照会红，判形乙那一段是好的；写 `MODEL_BROKEN` 或用例 panic 就是模型坏了）、`OPUSMODEL name=control kind=writer_variant_same_txg_in_another_slot … form_a=false`、`OPUSMODEL name=cell roots_hidden=2 root_and_record_fault=all_zero newest_system_configuration_fault=all_zero … form_b=<数>`、`OPUSMODEL name=estimate … projected_full_seconds=<秒>`，以及 25 行 `OPUSMODEL name=probe_count …`（每格这次挂载的结局与状态数）。编不过时 `exit` 非 0，日志里是 `error[…]` 行。
2. 全量（第 1 条的 `projected_full_seconds` 不超过 2400 才交）。起点目录同上。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 e165_attack_form_b 的全量用例; 范围=1 个测试目标; 为什么=攻方腿模型全量二十五格每格挂载写流的崩溃状态一个不落' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus/tree -- cargo test --release --offline -p singlefs-harness --test e165_attack_form_b -- --nocapture --test-threads=1 --ignored e165_attack_full`
   看：`exit=0`；日志里 25 行 `OPUSMODEL name=cell …`、至多 40 行 `OPUSMODEL name=form_state …`、末行 `OPUSMODEL name=done cells=25 cells_with_form_b=<数>`。本报告的推断要核的几列：`roots_hidden=2` 且系统配置读坏为 `all_zero` / `not_self_verified` 的 6 格 `form_b` 大于 0、`v1826_differs` 大于 0、`v1823_differs=0`、`v1825_differs=0`；其余 19 格 `form_a=0 form_b=0`；所有格 `form_a=0`、`core_disagrees_with_local_baseline=0`；那 6 格 `cell_mount` 行 `h2_mount=ok … effective=(1,4)`（附带打中：C 被丢）；`mount_after` 一列看基线在形乙镜像上择 (1,6) 之后挂得上还是拒。
3. 复跑脚本整份（判决要引的数由它出）。起点目录：仓根。`--work` 给一个不在或空的新目录。
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus/tree --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus-rerun`
   看：`TASK_SERVER_RESULT` 的 `exit=0`；日志里 `name=frozen_source` 六行、`name=model_source … sha256=1364e3ba1884396b1e4a05cc75bf6d752dcbfe03930c4672e2054e398e7a5170`、`name=probe exit=0`，之后要么 `name=full_skipped`（估时超过 2400 秒），要么全量那几行与 `name=rerun_roots … full_exit=0`。退非 0 的件数为 0 算过。

跑完之后新派同立场的腿（派发提示写「承接：research/prompts/m3-close-b19-e165-r1-opus-output.md」与任务号），读 `task-server/task log <号> --all`，把读数填回第一节、第三节，把「推的」改成「量过」或撤回。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；Q3（判据怎么改）不在我的攻击面，只在第三节给了 v1823、v1825 该留档的等价性测试长什么样。
- 没往统一任务服务交任务；模型没编、没跑，判决要引的数一个都没有，命令在上一节。副本上将来量出的数不算入库装置上的数。
- 没读禁读清单里的正推腿产出。
- 没为单盘几何、吞写、只坏第一次的瞬时读错建格（第七、八节）。
- 改法甲、乙、丙都没实现、没跑，被攻过零轮。

## 交回前的引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r1-opus-output.md` 末行（这一段写进来之前那一次）：`✓ 核了 27 处引文、9 处裸行号，对上 27 处，没判 9 处（其中裸行号没有引文可比 9 处；逐处列在上面）`。

没删的副本：`/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus/tree`（48M，仓副本，不带 target）：主 agent 要在里面交上面三条命令、并当 rerun.sh 的 `--frozen`，已在该目录 `.deliverables` 写成「交接」。
