# m3-close-b20-repairs-code-r1 攻方腿报告（第 20 批修补的代码轮）

全部用例都在副本里写好、没编译、没跑（子 agent 不交任务）。下文短写的 `mount.rs` 指 `crates/singlefs-core/src/mount.rs`，`mount_transparency.rs` 指 `crates/singlefs-verification/src/mount_transparency.rs`，`transaction.rs` 指 `crates/singlefs-core/src/transaction.rs`。下面凡是用例结局都是「推的，待主 agent 跑」；标「量的」的只有我现读源码、`grep`、`git show` 的读数。`rustfmt --check` 只证到语法解析得过。

## 复跑

```bash
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b20-repairs-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/copy --work /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/rerun-1
```

模型目录每个文件的 sha256（量的，`sha256sum`）：

```text
cd05e9074404714498347ae7f9d44cfa8f3a1e1cff6bd610d0293504d56a473f  attack_cases_appended_to_the_test_target.rs
9074cf69ce636ed4d04af50fef0069b3dc10cc39db86e2d81fe2167ab1fb6d97  attack_shape_of_the_newest_rotation_not_written.rs
2c605fdd559fa55fbb45a4d1a49e67f8bf2716814109ec4c35bca956b8944c43  mutation_row_appended_to_the_copy.tsv
e56863367481e1982c9d07dba739c015078e8ca49c2d77679da31cbc7cf60612  tools/rerun.sh
```

被判的四份文件与开工快照 `research/prompts/m3-close-b20-repairs-code-r1-snapshot/crates-sha256.txt` 逐份相同（量的：`sha256sum` 对上 aec331b3…、cb094f21…、8b4c09d1…、a4b093ef…），副本里这四份改动前也与之相同。

## 各格判定一览

| 面 / 格 | 判 | 一句话 | 量的还是推的 |
|---|---|---|---|
| 面一 删开关之后的产品路径 | **打中（H1）** | 取第一个 txg 那一遍直接读盘、不与所选那一版取 max：读阶段读对了 D、这一遍读坏，写行 txg 不大于所选那一版，违 D23 已定项 14 第 3 条；开关本身删得干净，`every_root_ring_slot_read_once` 调用点在、`MountOutput` 少字段没让断言变空 | 代码量的；结局推的，待主 agent 跑 |
| 面二 C971 靠什么拦 | **打中（H1 是拦不住的一半，没登记）** | 今天只靠读阶段 N-配置 那一判（R2、乙入判不出、计数器比）；读阶段之后那一遍读环不在任何一判里，登记只有 B2（C971 行）、B3（C975） | 同上 |
| 面三 判官按本池 fsid 筛之后 | **打中（H2、H3、H4）** | 判官吃的所选那一版仍是不筛 fsid 的恢复给的（H2）；只有一块盘带高世代别池槽时可写过、只读不过，条款没这一形（H3）；别池槽带 checker 不收的取值时判官与产品分叉（H4） | 同上 |
| 面四 新造法 | **打中（H5）** | 造的盘面不是任何一次崩溃走得到的（要两块盘都丢一次已确认的写，归 C6），注释写「就是这个样子」不实；写路径上 F2_NEWEST、丙并中间、两槽 C 从此只在条款不罩的盘面上有用例，账上没记 | 读码量的；用例待主 agent 跑 |
| 面五 18 个观察者共用 | **打中（H6）**；别的没打中 | 共用这一判把「新格式化的池上一次断电撕两写」认成合法拒：这一形可写拒、只读树表 0 条、离线出路也拒，三样都走不通，D22「判不出的盘面有一条离线出路」不成立；C492 字面也没跟上 | 旧产物量的（当参考）；用例待主 agent 跑 |
| 检查单 走不到的理由 | 成立 | 四份报告都写了没有；四份补丁 `src/` 下新加行没有 `expect`、`unreachable!` | 量的 |
| 检查单 类型与名字 | 成立（旁注） | fsid 是 `&[u8; 16]`，不是 newtype，与 core 存量同型；新加行没有 `_ =>`；造法的注释不实归 H5 | 量的 |
| 检查单 不兼容旧代码与旧格式 | **不成立（轻）** | 没有留下只为旧开关的分支；留下 4 处注释指到已删的 E205 装置（文件:行见第八节） | 量的 |
| 检查单 不新建判定库 | 成立 | 四份补丁改动行 0 命中 | 量的 |
| 检查单 所有任务多机可用 | 成立；Xid 109 那条算多机缺陷，根因推不出 | 这一批新用例用内存盘；那条用例照共用约束字面算缺陷，不归这一批 | 量的 / 推不出 |
| 检查单 测试优先多线程 | 成立 | 12 条各一个手摆小盘面，libtest 默认并行，没有写死线程 | 量的 |
| 检查单 带 --item 的命令与项 | 推不出 | 41 行变异：任务 11592 抓到 13 条、11593 抓到 9 条（都退 0），11594 我最后读时还在跑；观察者 13 行任务 11062 读过日志：抓到 13 条（副本上） | 量的 |
| 检查单 新崩溃流多罩了什么 | 成立 | 四份补丁只新建一个文件（观察者用例目标），不是新流 | 量的 |

## 一、面一：删开关之后的产品路径

**查过、没打中的部分（量的，读 diff 与现行代码）：**

- 删掉的 `first_txg_of_new_instance_under_the_test_only_switch` 在今天那一档第一步就是 `first_txg_of_new_instance`、随后 `return Ok(first_txg_today)`（`~/.local/share/singlefs/work/m3-close-b20-e205-retire/patch-without-manifest/crates.patch` 第 94–98 行），不多读盘。删开关前后产品路径的字节不变；现行调用点是 `crates/singlefs-core/src/mount.rs:9871`。
- `every_root_ring_slot_read_once`（`crates/singlefs-core/src/mount.rs:3270`）调用点 `mount.rs:3363`、`mount.rs:3383`，都在 `root_ring_read_once_or_twice_for_the_rebuild` 里，由 `rebuilt_allocator`（`mount.rs:3139`）调：不是死代码。
- `MountOutput::first_txg_of_the_new_instance_branch`、`FirstTxgOfTheNewInstance`、`RingSlotsTheFirstTxgGoesPast`、那个只供测试入口：`crates` 下 0 命中（量的，`grep -rn … | wc -l` 得 0）。读那个字段的断言只在删掉的那个测试目标里，补丁只动 4 份文件，没有别的断言跟着变空。

**打中 H1：取第一个 txg 那一遍不与所选那一版取 max。**

历史与故障（每一步指到许可它的代码）：

1. C971 最小历史：覆盖写 B、C、D（txg 4、5、6）；D 的根槽落了、两块盘的轮换没落（`crates/singlefs-core/src/transaction.rs` 发布的最后三步：根槽 FUA 在轮换之前）。c_见证 是 C 的计数器。
2. D 那次发布的记录两份都读回全零（常驻）。读阶段择根读到 D（经 `ReadStageCache`，`mount.rs:8966`），记录只扫到 C。
3. N-配置 为假：D 的末条读不出，走 `AgainstTheRecordAtTheWitnessedCounter`（`mount.rs:9162`），C 不大于 D，不拒。乙入判不出不判：四槽都自证过。
4. 所选那一版是 D。`first_txg_of_new_instance`（`mount.rs:1478`）不经读缓存、直接读盘（`highest_root_txg`，`mount.rs:1488`）：这一遍 D 的根槽瞬时读回全零，读坏的槽被跳过（`crates/singlefs-core/src/recovery.rs:1955`），`highest_ring_txg` 是 5；`records` 是读阶段那一份，最大也是 5。
5. 第 9871 行得到 first_txg = 6，等于所选那一版 D 的 txg；写行那次发布就用它（`mount.rs:8559`）。

条款：D23 已定项 14 第 3 条（`.claude/kb/decisions/23-journal的角色与格式.md` 第 373 行）写新实例第一次发布 ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1。D 这条根读阶段已经读到，它就是所选那一版。同一个函数旁边的姊妹 `tree_identifier_watermark_of_the_ring`（`mount.rs:1514`）就为这件事多取一次 max，注释写的正是「两次读环之间的瞬时读错（择根那一遍读到了它、这一遍没读到）不把水位拉低」（`mount.rs:1512–1513`），txg 这边没有。挂载透明判官的 `CheckpointTxg` 字段按同一个下界判（`crates/singlefs-verification/src/mount_transparency.rs:1885–1893`）。

限制（反推过）：只到「等于所选那一版」。要落到所选那一版之下，得 C 的记录也读不出，那时计数器比成 `Undecidable` 拒（`mount.rs:9174`）；D 已确认时 D 的记录读不出同样 `Undecidable` 拒。所以要 D 没见证、三处故障（D 的记录两份 + 这一遍 D 的根）。这一形新根与 D 同落点（`target_for_publish` 只看 txg）、新记录落回 D 的记录那一槽，不是 C971 的倒挂形；后果是新实例第一版与上一版同 txg，下游按「严格大于」写的账会不会错我没推完，用例把写完之后的池级 checker 与第二次挂载结局打出来。

用例：`attack_a_root_read_bad_only_after_the_read_stage_numbers_the_new_instance_at_or_below_the_selected_version`（k = 1..16 × 只这一次 / 从这一次起，不写死第几次读落在那一遍）。断言至少一格写行 txg 不大于所选那一版；这一条红就是我推错了。

四句：

- 分不分辨臂：分辨。甲两档那一遍另读环、读到 D 的槽读坏就越过，这一格大概率不中（推的，开关已删、量不了）；乙、丙不碰这一遍读，照中。E205 退场拿掉的正是能量这一格的开关，修它不必回到甲（改法见下）。
- 系统看不看得到：看得到，挂载手里有 `effective_root`（`mount.rs:9876` 就传给了水位那一个函数）。不是判别子观测不到。
- 字面哪一句：D23 第 373 行「根环里全部根记录的 txg」+ 1；D 这条根在根环里、这一次挂载读到过。
- 改法还中不中：跑前没有条款给过这一格的改法。我提的改法见下，只在读码上推的、被攻过零轮。

改法（被攻过零轮，没实现、没跑）：`first_txg_of_new_instance` 多收所选那一版，`highest_ring_txg.max(highest_record_txg).max(version.checkpoint_txg)`，与水位那个函数同形。

**面一的旁枝（变异缺口）：** 这一行上的变异只有三行（量的，`grep -n highest_ring_txg crates/mutations.tsv`）：第 940 行溢出、第 5974 行丢记录半、第 6824 行两半都丢。「丢根半、只看记录」没有行，退场也没有别的目标接它。用例 `attack_a_root_read_with_its_records_unreadable_takes_the_first_txg_past_it` 与副本变异表末行配成一对（命令第 3 条）。要不要进主仓交主 agent。

## 二、面二：C971 在 E205 退场之后的处置

C971 那一形今天靠什么拦（量的，读码）：只有读阶段 `newer_publish_witness`（`crates/singlefs-core/src/mount.rs:9103` 起）这一判。

| 那一形的变体 | 拦它的是 | 文件:行 | 登记 |
|---|---|---|---|
| 两条根与记录读坏，系统配置都好 | 计数器比：c_见证 大于所选那一版末条 | `mount.rs:513` | 有（D23 第 397 行起） |
| 同上，每块盘最新一槽设备报读错 | R2 | `mount.rs:9140`、`crates/singlefs-core/src/recovery.rs:5206` | 有 |
| 同上，每块盘最新一槽全零或自证不过 | 乙入判不出 | `mount.rs:9142`、`mount.rs:9190` | 有（C971 行） |
| 一块盘落后、另一块最新一槽全零（B2） | 不拦 | 用例 `a_device_holding_two_stale_self_verified_slots_lets_the_inverted_first_txg_through` | 有（C971 行，D22 第 185 行） |
| 别池又布局位不认（B3） | 不拦 | 用例 `a_slot_of_another_pool_with_unknown_incompat_bits_lets_the_inverted_first_txg_through` | 有（C975） |
| 读阶段读对、取第一个 txg 那一遍读坏（H1） | **不拦** | `mount.rs:1488` | **没有** |
| 见证读在读阶段读对、取号那一刻两块盘最新一槽瞬时读坏 | **不拦**（见下） | `crates/singlefs-core/src/transaction.rs:957`、`transaction.rs:1262` 起 | 条款写了只在读阶段判（`mount.rs:565`），后果没写 |

逐条核过的前提（量的）：系统配置每次发布都轮换（`.claude/kb/decisions/23-journal的角色与格式.md` 第 22 行「随每次发布写一次」），所以「系统配置都好而两条根藏起来」不会漏见证：我先试的「轮换有间隔」那个打法不成立。

**没登记的一半（打中，归 H1）：** E205 退场之后，甲（越过读坏的槽）那两档不再有开关，也不再量。H1 这一格恰好是甲与今天分得开的：甲那一遍另读环，读到 D 的槽读坏就越过（推的，开关已删、量不了）；今天不越过。判决第四节「退场」时看的代价表里有没有这一格，我查了 `.claude/kb/experiments/205-多处读坏倒挂三条改法的代价.md` 的小节与判决行：六格、撕裂、跳号三段都是「读阶段就读坏」，没有「读阶段读对、这一遍读坏」这一取样点（`grep -n '读阶段之后\|瞬时' ` 那份页面没有这样的格；这句只核到这一份页面）。所以退场不改变结论的前提是 H1 用 `max(所选那一版)` 单独修，不依赖甲。

**另一形（线索，没造用例）：** 取号那一刻只判 R2（`transaction.rs:975`），撕裂与全零的槽不拒，见证值取自证过的槽的最大值（`transaction.rs:1262` 的 `highest_journal_tail_of`）。读阶段四槽都好，到取号那一刻两块盘最新一槽瞬时读回全零：两轮取号写把两块盘两槽的 tail 都写成落后一次的值，P 的见证从盘上没了（不是读不到，是被覆写）。之后再崩、再藏 P 的根与记录，就是 B2 的盘面而没有丢写。要的故障：两处瞬时读 + 一次崩溃 + P 的根与记录，超单故障，后果推的。这一形把「丢写才走得到」那句话（判决 r2 B2 一行「纯崩溃走不到（推的）」）从「要丢写」放宽成「要丢写或者取号那一刻两处瞬时读坏」，只改射程句，不改代码也说得通。

**kb 现状对不上（量的）：** `.claude/kb/checks-owed.md` 第 611 行 C971 那一行的进度格还写「实现员在改（草稿目录 `~/.local/share/singlefs/work/m3-close-b20-c971-yi`）」，候选甲、丙也还写着「都被攻过零轮」，没写退场之后甲不再量。这一条归写回批。

**推翻条件：** H1 用例红（所有格写行 txg 都大于所选那一版，或 k 取遍都没做成挂载）。

## 三、面三：判官按本池 fsid 筛之后

**打中 H2：判官筛了 fsid，吃进去的所选那一版没筛。**

盘面：C971 最小历史；盘 1 较旧那一槽换成世代号大 100 的别池槽（`slot_of_another_pool`），盘 0 两槽本池自证过；C、D 的根与记录读回全零，系统配置不注故障。

1. 产品：`system_configuration_of_the_pool_named_by_the_caller`（`mount.rs:9689`）筛 fsid 择到本池。盘 0 两槽都自证过，乙入判不出不判（`mount.rs:9197`）。c_见证 是 C 的计数器，所选那一版是 B，`AgainstTheSelectedVersionsLastRecord` 为真（`mount.rs:513`），重读仍真，拒。这一拒对。
2. 判官：`judge_writable_mount_refusal` 的系统配置筛了（`crates/singlefs-verification/src/mount_transparency.rs:3304`），`selected_version` 是调用方给的，文档写「崩溃镜像上只读恢复落到的那一版」（`mount_transparency.rs:3288`），用例的调用方也这样取（用例文件 `writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:656`）。`recover` 不筛 fsid（`crates/singlefs-core/src/recovery.rs:1370` 那一格写恢复是 `AnyPool`）：盘 0 择本池、盘 1 择别池，报 `SystemConfigurationsDisagree`，`effective_root` 是 None。
3. 乙与 R2 都不成立，走到 `witnessed_counter_holds_against_the_selected_version`：`let selected_version = selected_version?;`（`mount_transparency.rs:2819`）交回 None，判据为假，判 `CriterionFalseWhenRecomputedOnTheCrashImage`。合法的拒判成不合法，与 r2 的 A1 同一个结局、不同的路。

入库用例没撞上：`the_judge_chooses_the_slots_of_this_pool_and_calls_a_claimed_refusal_illegal_…` 手填 `Some(newest_version_of_the_minimal_history())`（用例文件第 1343 行），`verdict_of_the_refusal_on` 那一路只在乙入判不出的盘面上用（乙在 selected_version 之前返回真）。

用例：`attack_the_judge_fed_the_version_recovery_lands_on_calls_a_legal_witness_refusal_illegal_when_one_slot_is_from_another_pool`（带对照：喂产品落到的 B 判合法）。

四句：

- 分不分辨臂：这一轮没有臂；它分辨「判官筛 fsid」这一改法够不够。
- 判官看不看得到：看得到，`filesystem_identifier_of_the_pool` 已在手里，只是所选那一版来自外面。
- 字面哪一句：`RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage`；`selected_version` 文档第 3288 行那句。
- 改法还中不中：r2 第五节的改法（判官加 fsid）在这一格**还中**。要么 `selected_version` 改成同口径（恢复另开一份按本池 fsid 的读，只给判官），要么文档改成「可写挂载读阶段落到的那一版」，调用方从拒因载荷取（`reread.selected_version`）。后一个把判官与被判的拿成同一个输入，有失独立，主 agent 判。都被攻过零轮。

**打中 H3：只有一块盘带高世代别池槽，条款没这一形。**

D22 已定项 8 射程（`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 185 行）只写「世代号比本池的都大、而每块盘都有这样一槽时，可写挂载拒，只读挂载与恢复择到的是别的池那一份、按它的 fsid 读不到根，报 `NoValidRoot`」。一块盘的两形：

| 盘面 | 可写 | 只读 / 恢复 | 只读的结局 |
|---|---|---|---|
| 盘 1 带高世代别池槽，盘 0 两槽好（单故障，C707 想救的那一形） | 做成（入库用例第 1420 行已断言） | 拒 | `SystemConfigurationsDisagree` |
| 同上，盘 0 再撕一槽 | 乙入判不出拒 | 拒 | `SystemConfigurationsDisagree` |

第一形可写过、只读过不去，方向与「读写都不行才叫坏」相反：想先只读挂上看一眼再决定的管理员被拒，可写反而做成。第二形和 B1 一样只剩离线，只是报的错不同，条款没写。用例 `attack_a_newest_generation_of_another_pool_on_only_one_device_refuses_read_only_while_writable_goes_through_or_refuses_too`。

四句：不分辨臂；只读挂载手里没有 fsid（`crates/singlefs-core/src/mounted_read.rs:650` 只收读者），判别子它观测不到，是「判别子观测不到」那一形，改条款写明，或给只读挂载收 fsid（改入口，零轮）；字面是 D22 第 185 行「每块盘都有这样一槽」的射程外；没有跑前改法。

**打中 H4（线索）：判官与产品读一槽的口径差，A3 作废之后另一种取值。**

产品 `system_configuration_slots_of_one_device_read_once`（`recovery.rs:5152`）用 `parse_slot`（`crates/singlefs-core/src/system_configuration.rs:714`）：查 magic、校验和、incompat、节点大小、每区槽数，不看区域数、加密类型、根环起点、槽距区间。判官的同名读法（`mount_transparency.rs:2382`）用 `parse_system_configuration`（`crates/singlefs-verification/src/verification_read.rs:88`），经 checker 的 `check_system_configuration_slot` 与 `geometry_of`（`crates/singlefs-checker/src/image.rs:845`）多判这些；解不开、又不属于自证不过（`mount_transparency.rs:2354`）的槽，两个计数都不进。

盘面：两块盘较旧那一槽换成别池的、区域数字段（`slot[361]`）改成 2、校验和重封。产品算别池的槽，乙入判不出拒；判官数不到，判据复算为假，判不合法。用例 `attack_a_slot_of_another_pool_with_a_region_count_the_checker_refuses_splits_the_judge_from_the_product`。

与 r2 作废的 A3 不是一回事：A3 查的是一槽两个毛病谁先判（判官给 Legal，作废）；这一格判官与产品对这一槽结论相反。同一口径差也在别的取值上：加密类型不是关、槽距越界（读码推的）。这一槽要别的版本或别的配置的池，第一版没有，与 C975 同一个优先级。

四句：不分辨臂；判官看得到字节，是判官自己读法窄；字面是乙入判不出「整槽校验和过而 fsid 不是本池的」那一半（`mount_transparency.rs:2404`）；改法见下。改法（零轮）：判官数别池槽照产品的五关，不走 `geometry_of`。

**查过、没打中（面三）：**

- 判官的 fsid 偏移常量 102（`verification_read.rs:134`）与 checker（`crates/singlefs-checker/src/lib.rs:279`）、core（`system_configuration.rs:45`，4 + 2 + 96）相同。
- `system_configuration_slot_bytes_belong_to_another_pool`（`verification_read.rs:138`）对短槽不越界：`TooShort` 先返回假。
- 筛 fsid 的读者只包系统配置那一读，别的判据读盘不经它（`verification_read.rs:199`）。
- `judge_writable_mount_refusal_of_a_crash_state*` 不筛 fsid（`mount_transparency.rs:3367`），文档说它判的崩溃态由一个池的写流造出。`crates/singlefs-verification/src/crash_injection.rs` 里 `grep -n 'another_pool\|filesystem_identifier\['` 0 命中，读故障只注读坏，没有换成别池字节的；它的写流只记一次 mkfs（`crates/singlefs-verification/src/crash_injection.rs:2077`）；这一句只核到这一份文件。
- 模型与实现共用错误前提：乙入判不出的两边定义一致（`mount.rs:9190`、`mount_transparency.rs:2408`）；分叉的只有上面的 H2、H4。

## 四、面四：新造法 `image_with_the_newest_rotation_of_each_device_not_written`

**查过、没打中的部分（量的，读码）：**

- 落槽对：造法写 `(g mod 2) × spacing`（`crates/singlefs-verification/tests/common_unwitnessed_publish/mod.rs:694`），与产品写槽同一式（`crates/singlefs-core/src/transaction.rs:714`）。
- 读阶段读数不变：旧造法把两块盘最新一槽读回全零，新造法把它换成 g − 2；两种盘面 c_见证 都是另一槽的值。取号的世代号算法取这块盘自证过的最大世代号加一（`transaction.rs:666`），两种盘面得同一个 g，落同一槽，往后写流相同。除乙入判不出之外的判法我读到的都只看 c_见证 或每块盘最大世代号。调查员「后面的路径不变」这一句，我独立读码同意（推的，任务 10392–10397 量过绿）。
- 全零形那半场景没丢：恢复与只读那一半仍有 `a_confirmed_root_whose_newer_system_configuration_slots_and_units_read_back_zero_reads_back_the_previous_version`（`crates/singlefs-verification/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs:321`），可写的拒在 `writable_mount_counts_…` 里。

**打中 H5：造的盘面不是这批用例说的那一形，也不是任何一次崩溃。**

用法都是两步叠：`crash_image_with_the_root_persisted_and_the_rotation_lost()`（R 的轮换没落），再 `image_with_the_newest_rotation_of_each_device_not_written`（`recovery_and_both_mounts_…witnessed.rs:510`，`fall_back_candidate_…land.rs:161`，`raise_floor_…root_ring_table.rs:1283`、`:4524`）。这时盘上世代号最大的那一次轮换不是 R 的，是 **R 前面那次已经返回的发布**（P，或丙并中间那一形的 Q）的。

1. P 的发布返回之前过了末尾屏障：`crates/singlefs-core/src/transaction.rs:1681` 那段写「根槽 FUA 写 → 系统配置槽轮换 → 屏障，屏障完成才返回」。所以 R 开写时 P 的轮换已持久。
2. R 的轮换写落的是另一槽（世代号 + 1 mod 2，`transaction.rs:714`），碰不到 P 那一槽。R 的任何崩溃态（撕裂在内）c_见证 都不小于 P 的尾。
3. 造法把 c_见证 拉到 P 的尾之下。要走到这张盘，得两块盘都丢掉一次已确认的写，且每块盘还得留下 g − 2。D22 第 185 行把丢写归 C6、写明不罩（B2 那一句）。

对不上的文字（量的）：

- 造法注释（`common_unwitnessed_publish/mod.rs:664–666`）写「一块盘世代号最大的那一次轮换没写成，就是这个样子」：单看一块盘、不叠 R 时是真话，用法里是假话。
- 同一句「世代号 g - 2、见证值与另一槽相同（那次轮换写下的就是这些）」：覆盖写历史里 g − 2 那次轮换写的是 Pm 再前一版的尾，不是另一槽的尾。c_见证 取最大，所以不改结局；只是注释不实。用例 `attack_the_fixture_needs_a_lost_write_on_both_devices_and_no_crash_state_of_r_reaches_it` 打出两个尾并断言不等，同时断言前面第 2 步。
- 用例注释写「R 没见证；P 已确认」（`recovery_and_both_mounts_…witnessed.rs:500`）而盘面 P 的轮换没写成：设备确认过又丢了，这个「已确认」不是条款说的那个。
- 实现员的改名把「whose_units_and_the_newer_system_configuration_slots_read_back_zero」换成「…newest_rotation_of_each_device_not_written」，名字现在说的是 P 的轮换没写，变异行与判决要读这个名字的人看不出它不可达。

后果（读码推的，**这是打中的实质**）：乙入判不出之后，可写路径上让 c_见证 落到所选根的前一版之下而判得出，我枚举了读坏组合走不通：P 的尾在两块盘各一份，两份都藏 ⇒ 没有一块盘两槽都自证过 ⇒ 乙拒；只藏一份 ⇒ 另一份照样见证 P。于是下面这几样写路径上的行为，有用例、有变异行，而用例的盘面都在条款不罩的那一边：

| 产品行为 | 变异行（现行号，量的） | 盘面今天怎么来 |
|---|---|---|
| `F2_NEWEST` 写路径：退到 [见证到的那一版, 所选根) 里最新的那条 | `crates/mutations.tsv:2895` | 两块盘丢 P 的轮换 |
| 丙并中间：C 那一圈并上 Q 那一槽 | 同文件「丙并中间」几行 | 两块盘丢 Q 的轮换 |
| 两槽 C、推环到较晚那一槽 | 同文件「攻方 M12 R 那一槽取最先轮到的那一槽」那行 | 同上 |

调查报告（`~/.local/share/singlefs/work/m3-close-b20-c971-reds/result.md`，「(a) 细说」）写了「丙并中间那一支从此只有 Q 的轮换写丢了才走得到…写进账给主 agent」。账上没有：`grep -n '丙并中间' .claude/kb/checks-owed.md` 与 `.claude/gate.d/dead-path-kept.tsv` 都 0 命中（量的），判决 r2 第六节去向表也没这一条。

一个能走到的旁路（线索）：第二节「另一形」，取号那一刻两处瞬时读坏把 P 的见证覆写掉。真成立的话这些分支不算全死，但那一形也没有用例。

四句：

- 分不分辨臂：分辨「换造法」与「删这几支」两条路：删了这几支的产品，在条款罩的盘面上不该有差。
- 系统看不看得到：产品看不出丢写，所以它照走；问题在用例对盘面的说法。
- 字面哪一句：D22 第 185 行「这是丢写，归 C6（块层语义假设写错），不罩」。
- 改法还中不中：没有跑前改法。两条出路都是零轮：（甲）注释与用例名写明这是丢写盘面，登一行欠账判这几支留不留；（乙）让用例回到走得到的盘面，那就要先证明有这样的盘面，第二节那一形是唯一线索。

## 五、面五：18 个观察者共用一处

**查过、没打中（量的）：**

- 19 份文件的调用点传的 fsid 与几何都与各流的池相符（逐处 `grep -A16` 看过：384 槽小盘的流传 `fixed_geometry_of_the_384_slot_small_devices`，清扫流传 `fixed_geometry_of_the_ring_hole_pools`，别的传 `geometry()`）。把 18 个观察者一起判错的那种「槽距传错」没有。
- 读集见证：录的是可写挂载与只读挂载的读（调用点的流都是 `retaining_contents_and_reads`，逐份 `grep` 过），这一判自己读的四槽（`mount_transparency.rs:2598`）可写挂载读阶段也读，都进读集；盘相同才复用，不会把结论复用给盘不同的态（`crates/singlefs-verification/src/crash_amplification.rs:23179` 起）。
- 拒因窄：两遍都要是乙入判不出（`mount_transparency.rs:2590`）。崩溃镜像不注读故障，两遍读相同，不会误红。
- 只读落到的版本只比恢复（`mount_transparency.rs:2657`），两边同一份判法，比不出恢复错；恢复对不对由恢复报告那一路判，不靠这里。
- 没改的那些挂载（`crash_points_of_the_instance_table_sweep.rs:1200` 这类 `.expect`）我只读到它们在录流那一段或写前缀切，与实现员报告相符；崩溃用例目前是缩到展开 5 的读数，全量没结局，这一句只到读码。

**打中 H6：被认下的这一拒，在新格式化的池上是死路。**

历史：mkfs，第一次可写挂载，取号写第一轮（或第二轮）两块盘各一写，断电，两写都撕裂。单故障，设备没报读错，在 D22 第 185 行「每盘一槽系统配置坏在承诺里」之内。所选那一版是第 0 代根（`(0, 0)`）。

1. 可写挂载：没有一块盘两槽都自证过，乙入判不出，拒（`mount.rs:9142`）。
2. 只读挂载：树表 0 条，`TreeTableHasNoEntries`。这一判认下这一答（`mount_transparency.rs:2667–2675`），计 `read_only_mount_found_no_file_version_after_the_refusal`。
3. 离线出路：断言第 0 代，根读得出，环里没有它的记录（mkfs 不写记录），`crates/singlefs-core/src/system_configuration_repair.rs:269` 那一支拒 `TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion`。

三样都走不通。条款 D22 第 185 行「判不出的盘面有一条离线出路」在这一形不成立。旧产物读数（量的，当参考，`git show 84b4ae23f:research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out`）：`into_undecidable=true` 且 `offline_way_out=refused` 恰好 6 行，S = 4、8、16 × 段 0、1。第 77 行原样：

```text
E7RESULT name=torn_state slots_per_region=4 segment=0 kind=acquisition state=[torn,torn] today_mount=ok system_configuration=[not_self_verified/ok,not_self_verified/ok] refuse_directly=true into_undecidable=true c596_shape=true offline_way_out=refused:TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion refusals_after_the_offline_way_out=[none]
```

E205 页把这 6 态记成「附带（不进判定）」（`.claude/kb/experiments/205-多处读坏倒挂三条改法的代价.md` 第 388 行），没进条款、没进欠账。实现员报告第 1 条问过这一支（「任务 11004 那 16 态选到 (0,0)」），判决 r2 没接。

为什么算在观察者头上：这一判一认，后面的「发第一个文件版本」就跳过，计数报的是「只读挂载找不到文件版本」，看着像常规答复。观察者注释还写着「C492：树表 0 条的一版上必须有可写的出路，重开之后发得出布」（`crates/singlefs-verification/tests/crash_enumeration_formatted_pool_remounted_without_a_file_stream.rs:159`），`.claude/kb/checks-owed.md` 第 924 行 C492 也写「挂不上或发不出当场 panic」：都是这批观察者放开之前的话。

代价的上限（推的）：盘上没有用户数据，重做 mkfs 就回来。所以不丢数据，是活性与条款不实。

用例 `attack_a_torn_acquisition_round_on_a_formatted_pool_leaves_no_way_out`（S × 段 六格，真写流、真撕裂）。

四句：

- 分不分辨臂：分辨乙入判不出与丙、甲：那几条臂这 6 态 `today_mount=ok`（旧产物同一行）。这是乙特有的代价，用户定乙时这个数只在「附带」里。
- 系统看不看得到：看得到，所选那一版就是第 0 代。
- 字面哪一句：D22 第 185 行「判不出的盘面有一条离线出路」；C492「树表 0 条的一版上…可写的出路」。
- 改法还中不中：没有跑前改法。三条零轮的路，都没实现：断言第 0 代时 tail 取 0（mkfs 写的就是 0，`crates/singlefs-core/src/make_filesystem.rs:789`，离线写出来与 mkfs 同值）；所选那一版是第 0 代时乙不判（环里没有比它新的东西，C971 的倒挂要的前提不在）；或者写进射程「这一形只能重做 mkfs」。交用户定。

## 六、改法表（都只在读码上推的、被攻过零轮）

| 改法 | 修哪一格 | 量的 / 推的 |
|---|---|---|
| `first_txg_of_new_instance` 与所选那一版取 max | H1 | 推的，待主 agent 跑；没实现 |
| 判官的 `selected_version` 改同口径 | H2；对 H3、H4 不起作用 | 推的，没实现 |
| 判官数别池槽走产品那五关 | H4；对 H2 不起作用 | 推的，没实现 |
| 只读挂载收本池 fsid，或 D22 补一块盘那两形 | H3；对 H2 不起作用（判官吃的恢复也要跟） | 推的，没实现 |
| 造法注释与用例名写明丢写，登欠账判留不留 | H5 的文字与账；产品行为不动 | 推的，没实现 |
| 断言第 0 代时 tail 取 0 / 第 0 代乙不判 / 射程写只能重做 mkfs | H6；对 H1–H5 不起作用 | 推的，没实现 |
| r2 第五节「判官加 fsid」（已入库） | A1、A2 修到了（入库用例在）；H2、H4 上**还中** | 用例已入库、任务读数归主 agent；H2、H4 推的，待主 agent 跑 |

## 七、没打中的形状（面二、三、四、五另列的之外）

| 形状 | 取样 | 结果 |
|---|---|---|
| 轮换有间隔，系统配置都好时漏见证 | 读 D23 第 22 行、`transaction.rs:1681` | 每次发布都轮换，不成立 |
| 见证计数器比法在所选那一版末条读不出时放过 | `mount.rs:9147–9175` 四支逐支推 | 放不过：末条读不出又见证那条读不出就 `Undecidable` |
| 写行 txg 落到所选那一版**之下**（不要丢写） | H1 那条路上加 C 的记录读不出 | 落不下去，`Undecidable` 拒（推的；用例只覆盖等于那一格） |
| 判官筛 fsid 的读者越界、偏移算错 | 三处常量比、`TooShort` 一支 | 没打中 |
| 删的 586 行里被别的目标依赖的逻辑 | `grep` 删掉的名字 | `crates` 下 0 命中；`scripts`、`.claude/gate.d` 下只有 `quoted-product-unreplayable.tsv` 第 13–15 行（登记的就是退场） |
| 造法写错槽、世代号算错 | 与 `transaction.rs:714` 比 | 没打中 |
| 观察者读集不全、几何传错、流没开记读 | 19 份文件逐处 | 没打中 |

没打中的那几格只抽了这一次样，照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不算结论。

## 八、检查单八格

| 格 | 判 | 依据（量的） |
|---|---|---|
| 走不到的理由二次查实 | 成立 | `~/.local/share/singlefs/work/m3-close-b20-e205-retire/result.md` 第 56 行；`m3-close-b20-crash-observers/result.md` 第 51 行起「无」；`m3-close-b20-tests-after-yi/result.md` 第 159 行起「没有」；`m3-close-b20-judge-fsid/result.md` 里 `grep -c 走不到` 得 2。四份 `crates.patch` 的 `src/` 新加行 `expect(`、`unreachable!` 都是 0（`awk` 按 `+++ b/` 分文件数的）。`image_after_the_refusal_into_the_undecidable_witness` 里的 `panic!` 是观察者判红，不是走不到 |
| 类型与名字 | 成立（旁注） | `filesystem_identifier_of_the_pool: &[u8; 16]`（`crates/singlefs-verification/src/mount_transparency.rs:3295`、`crates/singlefs-verification/src/verification_read.rs:196`）不是 newtype；core 同一个量也是裸数组（`crates/singlefs-core/src/recovery.rs:1809`），`.claude/rules/code-discipline.md` 第 132 行管的是「同一种底层类型承载不同含义」，这里没新增一种含义，记旁注不判不成立。四份补丁新加行 `_ =>` 0 处。没有缩写。名字本身不错，注释不实归 H5 |
| 不兼容旧代码与旧格式 | **不成立（轻）** | 没有留下只为旧开关的分支。留下的是指到已删装置的注释：`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:92`、`:102`、`:414`（`同 E205 装置`、`E205 装置 build_starting_history`），`:570`（`E205 装置 fault_ranges_of`）。装置已删，读的人要去 git 历史才找得到；实现员报告第 16 行自己点过第 9 行那一处，别的没点 |
| 不新建判定库 | 成立 | 四份补丁加号行 `rocksdb|sqlite|\.db\b|prune-server` 0 命中。退场还删了 E205 的逐格入库 |
| 所有任务多机可用 | 成立；Xid 109 那条算缺陷 | 新用例内存盘，不读本机状态。`the_gpu_recovery_kernel_…` 在 5060 Ti 红、32G 卡绿：`.claude/agent-common.md` 第 40 行「派到某一台就红的是缺陷，当场修，不往统一任务服务里加只派一台的名单」，照字面算多机缺陷。不是这一批的代码；是卡的显存、驱动还是用例，一次红推不出 |
| 测试优先多线程 | 成立 | `crash_points_observers_take_only_the_refusal_into_the_undecidable_witness` 12 条（`grep -c '#\[test\]'` 得 12），各一个手摆小盘面，libtest 默认并行，没有 `static`、`serial`、`test-threads`（`grep` 0 命中）。`.claude/rules/implementation-workflow.md` 第 77 行那一节管的是切片与线程数，这里没有切片可切 |
| 带 --item 的命令与项 | 推不出 | 任务 11592 日志「点名的 13 条变异都抓到了」、11593「点名的 9 条变异都抓到了」，都 `type=succeeded exit=0`；11594 我最后读 `task-server/task status` 时在跑，41 行里这一份没结局。观察者 13 行任务 11062 日志末行「点名 13 条：现判 13 条、复用 0 条、拒算 0 条；跑了 13 条，抓到 13 条」（注意：副本上） |
| 新崩溃流多罩了什么 | 成立 | 四份补丁 `--- /dev/null` 共 1 处，就是观察者用例目标；16 份 `crash_enumeration_*_stream.rs` 是改观察者，不是新流 |

## 九、这条腿自己的限度

- **一条用例都没编译、没跑。** 所有结局都是读码推的。H1 尤其要看：第几次读落在那一遍、之后的读对不对、`first_txg` 等于所选那一版时写行发布会不会自己报错或 panic，都要跑了才知道；用例把这些都打出来，不预设。
- H1 的下游后果（同 txg 的两版在账上会不会错）我没推完，只到条款字面与判官字段。
- H2–H4 都是手造盘面，与 r2 判 A1、A2、B3 同一个可达性：别池的槽要盘面被别的东西写过（`crates/singlefs-core/src/make_filesystem.rs` 清零那一段让崩溃走不到）。
- H5「读坏组合走不通」是我枚举两块盘四槽推的，没写模型穷举；取号瞬时读坏那一形只到线索。
- H6 的 6 态旧产物是早先一棵树（提交 84b4ae23f）上的读数，当参考；用例在现行代码上重造。
- 面五只读了调用点与这一判，没逐个流核跳过之后少判了什么。
- 没读禁读清单里的四份报告；r1、r2 已攻过的角度我只从判决与正文知道，有重复的话是这个缘故。

**什么现象会推翻：** H1：用例红。H2：`effective_root` 不是 None，或判官判合法。H3：只读挂载做成，或可写被拒。H4：判官判合法。H5：`tail_on_the_fixture` 不小于 `tail_of_p`，或 R 的轮换写落到 P 那一槽。H6：只读挂载不是树表 0 条，或离线出路做成。

## 要主 agent 跑的命令

全部在仓根 `/home/fy5090/code/singlefs` 起；`--cwd` 是副本 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/copy`（`.deliverables` 写成交接；用例与变异行已在里面，别的文件与开工快照相同）。

1. 攻方用例：

```bash
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors 里攻方腿追加的六条; 范围=1 个测试目标; 为什么=取第一个 txg 那一遍读坏、判官吃恢复落到的那一版、别池单盘、别池取值分叉、新格式化池死路这几格的读数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/copy -- cargo test -p singlefs-verification --test writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors attack_ -- --nocapture
```

看：`TASK_SERVER_RESULT` 的 `type=`、`exit=`；`task-server/task log <号> --all` 的 `ATTACK_` 行与 `test result:`。全绿 6 条是我全推对了，**某一条红就是我推错了那一格**，不是用例要改。`ATTACK_FIRST_TXG` 每格一行，末行 `ATTACK_FIRST_TXG_SUMMARY`。编不过的话新派同立场的腿改。

2. 造法盘面：

```bash
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 attack_shape_of_the_newest_rotation_not_written; 范围=1 个测试目标; 为什么=换造法的盘面要两块盘都丢一次已确认的写才走得到，打出尾的读数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/copy -- cargo test -p singlefs-verification --test attack_shape_of_the_newest_rotation_not_written -- --nocapture
```

看：`ATTACK_FIXTURE` 行与 `test result: ok. 1 passed`。

3. 证红「只看记录」那一行（第 1 条绿了再交）：

```bash
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=prove-red 证红副本变异表末行; 范围=点名的 1 行变异; 为什么=新实例第一个 txg 丢根半这一改坏今天表里没有行，核攻方用例抓得到' task-server/task submit -- bash scripts/mutation/prove-red.sh --copy /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/copy singlefs-verification '攻方副本：新实例第一个 txg 只看记录（环里的根不进 max）'
```

看：`抓到`，末行「跑了的都抓到了」；基线红就停。

4. 模型整份复跑（判决要引的数，主 agent 照规矩拷模型目录再跑）：

```bash
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b20-repairs-code-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/copy --work /home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/rerun-1
```

看：`task wait` 末行退 0；输出末行 `name=rerun_exits cases=0 fixture=0`。`--frozen` 给哪棵都行：脚本认得用例已追加就不再追加。

跑完要接着判读的，新派同立场的腿，带「承接：research/prompts/m3-close-b20-repairs-code-r1-opus-output.md」与任务号。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没写枚举崩溃状态的原型：这一轮的盘面都是定点手摆或单态撕裂，H5 的「R 的任何崩溃态」靠「R 的写碰不到那一槽」一句断言，不靠枚举。
- 没改主工作区任何文件；kb 与代码的改法都只写在这里。
- 没读 r1、r2 的腿报告（禁读）。
- 没删 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-attack/copy`（52M，`du -sh` 量的，建时读数）：交接给主 agent 跑上面的命令，`.deliverables` 已写。
