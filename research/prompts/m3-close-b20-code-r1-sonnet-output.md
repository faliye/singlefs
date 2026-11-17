# m3-close-b20-code-r1 正推腿报告（Sonnet）

立场：正推。从已定条款与 `crates/` 今天的实现推出「应该是什么」，再与被判的 C971、C973 两处代码逐格比。没读攻方腿的报告，没引本轮之外的 `*-main-verification.md` 判决。
引文都在被引文件里 `grep -nF` 过一次；行号是被引文件自己的行号。读数里没标「量的」的都是读码推的。

## 要主 agent 跑的命令

我没往统一任务服务交任务，没起测试与装置。下面三条要跑才核得了，各对应一格「待主 agent 跑」。

1. 新测试整份（核拒在第一个写之前盘面逐字节不变、11 条用例）：
   `task-server/task submit -- cargo test -p singlefs-verification --test writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors`
2. 清扫第 0 代根判据的单测（核四条判据用例与两条函数单测）：
   `task-server/task submit -- cargo test -p singlefs-core --lib instance_table_sweep::misplaced_ring_reads_tests`
3. 新项能被列出（核「带 --item 的命令与项」一格的后半句）：
   `task-server/task submit -- bash scripts/crash/crash-point-replay.sh --list-items`

我自己做的只读核查：变异表 29 行（`crates/mutations.tsv` 第 1948、2284、2285、7045–7063 行，加 C973 的 6900、6901、6992–6997、7001、7002）的「原文」在各自文件里逐条恰好命中一次。脚本在 `/home/fy5090/code/singlefs` 之外的草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r1-legs/forward/anchors.py`，输出 29 行命中数列都是 1。这只证锚点没失效，不证变异会红（会红是任务 10223 的读数，我没复跑）。

## 一、C971：乙入判不出（`mount.rs`）

### 应该是什么（条款原文）

`research/prompts/e205-preregistration.md` 第 46 行：「乙入判不出：重读之后仍是这两种之一的槽，并进 R2 的「有槽设备报读错」那一半当读错计，R2 的另一半（没有一块盘两槽都自证过）照旧；判不出就拒可写。」
同一份第 47 行：「两种读法共用：「自证过」与 `system_configuration_slots_of_one_device_read_once` 同口径（整槽校验和过、fsid 与本池相同）；布局号不认、每区槽数越界的槽照实现不算坏（不进乙）；乙只加一道拒，不拒时其余每一步与今天相同。」
kb 落点 `.claude/kb/decisions/22-单元原子性怎么合成.md` 第 185 行（已定项 8 射程）有同一句的定稿：「读阶段读回全零或读得出而自证不过的槽重读那一槽一次，重读后仍是这两种之一的槽并进 R2「有槽设备报读错」那一半当读错计，R2 的另一半照旧，判不出就拒可写」。

从这两处推出的应有形：
- 坏槽 = 读得出且（读回全零 或 自证不过）的槽；设备报读错的槽不属于它（那是 R2 本身）；
- 判不出 = （有槽设备报读错 或 有这样的坏槽）且没有一块盘两槽都自证过；
- 判不出按判据 N-配置 为真，重读一次，仍判不出就拒可写，拒在第一个写之前；
- 取号、F2 退回、逐盘核不动。

### 实现今天的样子，逐项对

**1. 认的是不是恰好「读得出而自证不过」与「读回全零」：成立，另有两处边要主 agent 看。**
- 一槽读得出而 `SystemConfiguration::parse_slot` 拒成 `NotSelfDescribing`（magic 不对或整槽校验和不过）就记坏：`crates/singlefs-core/src/system_configuration.rs:715`、`:718`。读回全零 magic 不对，落在 `:715` 那一支。
- `recovery::system_configuration_slots_of_one_device_read_once` 对这两样与 `NodeBytesNotTheFormatConstant` 调 `reader.note_that_the_bytes_read_fail_self_verification`：`crates/singlefs-core/src/recovery.rs:5160`–`:5167`；套在读者外面的那一层把这一读的结论改记 `ReadFailingSelfVerification`：`crates/singlefs-core/src/mount.rs:6692`–`:6717`。
- 布局位不认、每区槽数越界不报坏：`crates/singlefs-core/src/recovery.rs:5168`–`:5172`，与第 47 行一致。
- 设备报读错的槽记 `ReportingAReadError`（`crates/singlefs-core/src/mount.rs:6645`），不进新判法，由 R2 在前面判（`crates/singlefs-core/src/mount.rs:9186`）。
- 边一：`NodeBytesNotTheFormatConstant`（节点大小字段不是格式常量，`crates/singlefs-core/src/system_configuration.rs:732`）也被并进坏槽。第 46、47 行没点名它；它由第 47 行的 read_once 同口径带进来，`system_configuration.rs:828` 注释写它与 `NotSelfDescribing` 同一个处置（转述）。我判这一处与条款相容，写出来供核。
- 边二（可能打中，见「判定一览」F1）：fsid 与本池不同、整槽校验和过的槽不并进坏槽。实现里它既不进 `self_verified`，也不调 `note_that_..._fail_self_verification`：`crates/singlefs-core/src/recovery.rs:5151`–`:5157`（`if` 没有 `else`）；`crates/singlefs-core/src/mount.rs:9229`–`:9231` 注释明写「别的池的槽读的一方不报读坏，不算」；独立判官也这样写，`crates/singlefs-verification/src/mount_transparency.rs:2331`「别的池的槽不算」。
  条款字面：第 47 行把「自证过」定成「整槽校验和过、fsid 与本池相同」，不并进的只点名布局号不认与每区槽数越界两样；`.claude/kb/checks-owed.md` 第 424 行（C707）写「fsid 不是本池的槽按自证不过处置」。按字面，fsid 不符的槽是「读得出而自证不过」，该并进。实现与判官一致地不并进，两处同出一手，不构成互相独立的印证。
  后果面（推的）：只有没有一块盘两槽都自证过、没有撕裂 / 全零 / 读错的槽、至少一块盘有一槽是别池的这一形，实现不拒而字面该拒，例如两块盘各一槽自证过、另一槽都是别池的镜像（双故障）。单个别池槽配另一块盘两槽都自证过，两边都不拒（`crates/singlefs-verification/tests/writable_mount_passes_over_a_system_configuration_slot_of_another_pool.rs:24` 那一条就是这一形）。
  什么现象会推翻「该并进」这一读法：用户或 C707 的原话说的「按自证不过处置」只管择系统配置那一步（`choose_system_configuration_of_the_pool`），不管见证读。C707 行去向栏写「只接在可写挂载与离线出路上」，读法偏向后者，我没法单凭读码排除前者。
  同一份代码里的旁证（读法分歧的线索）：逐盘核那一处的注释把「别的池的」和撕裂、清零并列写进「读得出、自证不过」：`crates/singlefs-core/src/mount.rs:6795`；而同一个读者的分类枚举注释写别的池的槽「读的一方没报自证不过」：`crates/singlefs-core/src/mount.rs:6598`–`:6600`。同一份文件里两处对别的池的槽算不算自证不过说法不一。

**2. 重读是不是恰好一次、读的是系统配置槽不是根：一次成立；读的对象比条款粗一档。**
- 恰好一次：`read_stage_settled_at_most_on_the_one_reread` 先读一遍（`crates/singlefs-core/src/mount.rs:9324`），判据为真且缺记录那一项为空时才重读，只重读一遍（`:9333`–`:9334`），没有循环；重读那一遍判假就往下走，判真拒（`:9336`–`:9345`）。
- 系统配置槽从盘上重读：`newer_publish_witness` 的读者是 `devices`（`crates/singlefs-core/src/mount.rs:9167`–`:9177`），不是读缓存 `cache`；`read_stage` 把 `devices` 和 `cache` 分着传（`:9248`–`:9263`）。
- 但重读的是整个读阶段，不是「那一槽」：重读那一遍还是 `read_stage`（`crates/singlefs-core/src/mount.rs:9334`），择根、扫 journal、重放都在读缓存上重做，系统配置槽只是其中一步。条款（D22 第 185 行）写「重读那一槽一次」。
- 触发条件也是整个判据：只有判据 N-配置 为真才重读（`mount.rs:9326`）。条款写的是每个坏槽都重读。
- 两处差别的外延（推的）：
  - 一块盘两槽都自证过时判据可以为假，坏槽不会被重读；条款字面会重读它。重读读好了会抬 c_见证，但第 47 行写「乙只加一道拒，不拒时其余每一步与今天相同」，实现这样处理与第 47 行一致，与第 185 行「重读那一槽」字面不同。
  - 判据为真时重读的是每个槽，包括第一遍读好了的槽。一块盘第一遍一槽好一槽坏，重读时那个好槽碰到瞬时读坏、坏槽读好，实现算仍没有一块盘两槽都自证过而拒，条款字面（只重读坏槽）算那块盘两槽都自证过、不拒。实现更严，方向是多拒。
- 什么现象会推翻粗一档没有结果上的后果这一判：构造上面第二种盘面（一槽好一槽坏的盘，重读时好槽读坏、坏槽读好，另一块盘也没有两槽都好）并注入读故障序列；没做，要建盘面。

**3. 「没有一块盘两槽都自证过」是不是照 R2 原有的那一半：成立。**
- 新函数的后半句 `!slots_of_each_device.iter().any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)`：`crates/singlefs-core/src/mount.rs:9239`–`:9241`。
- R2 的原式 `.iter().any(...slots_reporting_a_read_error > 0) && !....any(both_slots_self_verified)`：`crates/singlefs-core/src/recovery.rs:5191`–`:5196`。两处后半句是同一个函数同一个量，且 `both_slots_self_verified` 是同一个方法（`crates/singlefs-core/src/recovery.rs:5129`）。
- 次序：R2 先判（`crates/singlefs-core/src/mount.rs:9186`），乙入判不出第二判（`:9188`），然后才是「没见证」（`:9193`）。乙入判不出压在 `witnessed_journal_counter == 0` 之前，与 R2 同位；刚 mkfs 的池两槽都写过（`recovery.rs:5158`–`:5159` 注释），不会误触。
- 量化范围是 `devices.device_identities()` 的每一块盘（`mount.rs:9167`），盘数在函数前面已核等于系统配置的盘数（`mount.rs:7473`），所以对三块盘以上同样成立；`STRIPE_WIDTH_LOWER_BOUND` 是 2（`mount.rs:824`），可写挂载没有上界。随机历史的池固定两块盘（`crates/singlefs-verification/src/history.rs:2791`），新测试也是两块盘，三块盘以上没有用例罩。

**4. 拒是不是在第一个写之前：成立（读码），盘面不变的断言待主 agent 跑。**
- 拒发生在 `read_stage_settled_at_most_on_the_one_reread` 里，调用点 `crates/singlefs-core/src/mount.rs:9720`。它之前的步骤（`admit_the_writable_device_count`、`refuse_device_identities_handed_in_more_than_once`、`system_configuration_of_the_pool_named_by_the_caller`、`parameters_of_the_selected_system_configuration...`、`every_device_reaches_the_unit_area_start`，`mount.rs:9690`–`:9708`）的盘参数都是共享引用（`mount.rs:9658`、`:7510`、`:7424`），写不了盘；取号在 `establish_instance` 里（`mount.rs:9884`）。
- 断言：`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:631` 的 `image_after == history.image`（拒在取号之前，盘上逐字节不变）。待主 agent 跑（命令 1）。

**5. 拒因沿用已有成员有没有让别处走错：核心层没发现，判官 / 工具层待主 agent 看一眼。**
- 核心层对 `comparison` 的读者只有 `witnesses_a_publish_newer_than_the_selected_version`（`crates/singlefs-core/src/mount.rs:503`–`:525`）。新成员在那里和 R2 成员并列落 `true`（`:516`–`:523`）。我 grep 了 `WitnessedCounterComparison`、`NewerPublishWitness`、`SelectedVersionAgainstTheWitness`，核心层只有 `mount.rs` 用，没有别的 `match` 要补臂；`match` 里没有通配臂。
- 只读挂载：`read_stage_settled_at_most_on_the_one_reread` 只有 `mount.rs:9720` 一个调用点，只读挂载在 `crates/singlefs-core/src/mounted_read.rs`，这一批没改（`git status --short crates` 没列它）。测试里也断言了撕裂轮换那一态只读挂载照常：`writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:946`。
- 取号那一刻读见证值：`crates/singlefs-core/src/transaction.rs:964` 仍只调 R2；F2 退回那一处：`crates/singlefs-core/src/recovery.rs:8139` 仍只调 R2；取号之前逐盘核：`crates/singlefs-core/src/mount.rs:6784` 仍只用 `slots_reporting_a_read_error`。三处都与 `mount.rs:559` 注释写的「取号那一刻读见证值与择根退回那一处照 R2 本身判」一致。
- C596 那一格：离线出路 `crates/singlefs-core/src/system_configuration_repair.rs` 这一批没动。乙入判不出拒下的盘面，离线出路仍是 D22 第 185 行写的那一条。
- 拒因成员的文档没跟上（可能打中，轻）：`StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion` 的注释写「系统配置见证过一次比所选那一版新的发布」（`crates/singlefs-core/src/mount.rs:399`–`:401`），而它现在也载着 R2 与乙入判不出两种「判不出」。R2 那一种本来就这样，乙入判不出只是又添一种；区分靠载荷里的 `comparison`（`mount.rs:403`–`:404`）。读这个成员的人只看成员名会读成见证到了更新的发布。
- 非拒的挂载在这一形上也变了（可能打中，轻）：判据第一遍为真而重读为假时，往下走用的是重读那一遍的读阶段，`rereads.read_stage` 记 `OnTheOneReread`（`crates/singlefs-core/src/mount.rs:9358`–`:9364`），之前同一盘面记 `OnTheFirstRead`。第 47 行写「不拒时其余每一步与今天相同」，这一条是「多一次重读」的可观测痕迹，不是多一道拒；测试已断言它（`...as_read_errors.rs:969` 那一条）。

## 二、C973：第 0 代根判据（`instance_table_sweep.rs`）

### 应该是什么（条款原文）

`.claude/kb/checks-owed.md` 第 613 行（C973 一行，「怎么拦」一栏与「去向」一栏），逐字摘三句，整行在那一行里：
- 「落点核认第 0 代根住三个区域的槽 0 任一槽，核不出错位读回的第 0 代根，所以清扫另加第 0 代根判据」
- 「错位读算这一槽自证不过，走原有的重读一次，仍不过这一次不删」
- 去向栏复述判据：「区域 r 的槽 0 读到第 0 代根、而这一遍读出的环里已有落点公式会写进这一槽且 txg 更大的根时，算这一槽自证不过，走原有的重读一次，仍不过这一次不删」
上位条款：D16 `.claude/kb/decisions/16-发布语义.md` 第 195 行（骑手条款 1，落点核「恢复、挂载与实例表清扫读根环时都调」，第 0 代根照 mkfs 种根的槽另认）；D18 `.claude/kb/decisions/18-块里携带什么信息.md` 第 318 行「其余的槽须自证过，否则重读一次，仍不过就这次不删」。

从这几句推出的应有形：
1. 清扫读根环每槽调读者同一个落点核；错位读等于这一槽这一次自证不过，重读一次，仍不过就整轮不删；
2. 第 0 代根只住三个区域的槽 0（`crates/singlefs-core/src/root_ring.rs:188`–`:191`、`crates/singlefs-core/src/make_filesystem.rs:816`–`:818`），落点核对它不分哪个槽 0，所以要再有一条判据：槽 0 读到第 0 代根，而环里别处已经有证据说发布路径写过这一槽，就算这一槽自证不过；
3. 这条判据走和 1 同一条「重读一次」的路，不另开出口。

### 实现今天的样子，逐项对

**1. 清扫调读者同一个落点核：成立。**
- 清扫：`crates/singlefs-core/src/instance_table_sweep.rs:583`–`:589` 调 `root_sits_in_the_ring_slot_the_formula_gives`；读者：`crates/singlefs-core/src/recovery.rs:2010` 与 `crates/singlefs-core/src/mount.rs:3041` 调的是同一个函数（`crates/singlefs-core/src/root_ring.rs:183`）。
- 落点核不过 → 返回 `Ok(None)`（`instance_table_sweep.rs:582`–`:590`），与「自证不过」同路。

**2. 错位读 → 重读一次 → 仍不过不删：成立。**
- 每槽至多读两次，两次都 `None` 就 `Err(RootRingSlotIsNotSelfVerifiedAfterOneReread)`：`crates/singlefs-core/src/instance_table_sweep.rs:480`–`:500`；读不出与读回全零直接 `Err`，不重读：`:573`–`:578`。
- 用例（行里点名的名字都在）：`misplaced_read_replacing_the_only_ring_root_of_an_instance_on_both_reads_deletes_no_row`（`instance_table_sweep.rs:1445`）。

**3. 第 0 代根判据：成立，措辞有一处差，后果只多不删。**
- 条件「槽 0 读到第 0 代根」：`instance_table_sweep.rs:608`–`:610` 只判 `checkpoint_txg == GENESIS_CHECKPOINT_TXG`（`make_filesystem.rs:53`，值 0），不再判槽号是 0。这一步多余但无害：第 0 代根只会在槽 0 通过落点核（`root_ring.rs:188`–`:191`），走到判据时槽号已经是三个区域的槽 0 之一。
- 条件「这一遍读出的环里」：最大 txg 取第一遍每槽读出的根（`instance_table_sweep.rs:507`–`:511`），重读读出的不改它（`:522` 注释、`:541` 的闭包仍用同一个 `highest_txg_of_the_pass`）。
- 条件「已有落点公式会写进这一槽且 txg 更大的根」：实现是 `(GENESIS+1..=最大 txg).take(环长)` 里有没有一个 txg 的 `target_for_publish` 等于这一槽（`instance_table_sweep.rs:611`–`:613`）。这是「最大 txg 已到发布路径写进这一槽的 txg」的上界式，不要求那个 txg 真有一条根在环里。
  - 行里字面是「环里已有落点公式会写进这一槽且 txg 更大的根」。按字面读它永远不成立（推的）：过了落点核的根都住在自己 txg 的落点槽里，环里不会有第二条落点是这一槽的根，这一槽自己住的是第 0 代根。所以实现取上界式是唯一有操作意义的读法；`instance_table_sweep.rs:593`–`:597` 的注释也这样写。
  - 单测把这一点钉死了：判据函数只吃（txg、槽、最大 txg、每区槽数、环长）五个量，不吃根集合（`instance_table_sweep.rs:601`–`:607`）；`instance_table_sweep.rs:1859`–`:1862` 断言最大 txg 到 1 时区域 1 槽 0 就算已写到。
  - 遍历长度：连续 3S 个 txg 每槽恰好落一个（`root_ring.rs:167`–`:174`），`take(环长)` 从 txg 1 数到 3S，区域 0 槽 0 的第一个落点是 3S 而不是 0，单测钉了这一格（`instance_table_sweep.rs:1863`–`:1865`，最大 txg 11 不算、12 算，S = 4）。
- 差的后果（推的）：发布 txg 有跳号时，上界式会把一个其实没人写过的槽也算成「已写到」。跳号来自 `first_txg_of_new_instance` 取 max(根最大 txg, 记录最大 txg) + 1（`crates/singlefs-core/src/mount.rs:931`）：记录写了、根没写的那次发布崩了，下一个实例跳过它的 txg。只有跳过的恰好是 1、2 或 3S（落在第 0 代根所在槽的三个 txg）才碰到。碰到时那一槽的第 0 代根是真的，却被算成自证不过，清扫每一轮都停在 `RootRingSlotIsNotSelfVerifiedAfterOneReread`，直到环转到把那一槽写掉（至多 3S 个 txg）。方向是多不删，不会错删行；`.claude/kb/decisions/18-块里携带什么信息.md` 第 318 行本来就写了「一个根槽持续读不出时行永远删不掉」这笔代价。
  - 什么现象会推翻：造一份 txg 1 的根没写过、之后至少 txg 2 已写的池，让清扫在区域 1 槽 0 仍是第 0 代根时跑；若清扫照删行，上面这条推论就错。没做，要建盘面。

**4. 重读 / 仍不过不删走原有的路：成立。**
- 判据命中的槽：第一遍就不是一次读成的（已重读过一次）→ 直接 `Err`，不读第三次：`instance_table_sweep.rs:527`–`:533`；用例 `ring_slot_reread_after_one_misplaced_read_and_read_back_as_the_genesis_root_is_not_read_third_time_and_deletes_no_row`（`:1924`）。
- 第一遍一次读成的 → 重读一次，重读读出的仍是已写到的第 0 代根或读不出 → `Err`：`:534`–`:548`；读对了就换成重读读出的根：`:549`；用例 `...reread_right_keeps_its_row`（`:1730`）、`...on_both_reads_deletes_no_row`（`:1703`）。
- 发布还没写到那一槽时第 0 代根照认：`the_genesis_root_left_in_a_ring_slot_no_publish_has_reached_yet_is_self_verified_and_no_row_is_a_candidate`（`:1748`）。
- 阳性对照（错位读回非第 0 代根时落点核拦下）：`misplaced_read_returning_another_root_than_the_genesis_root_on_the_same_pool_deletes_no_row`（`:1763`）。
- 变异：C973 的 8 行（`crates/mutations.tsv` 第 6992–6997、7001、7002 行）锚点各命中一次。

**5. 判据对清扫够不够用（正推，推的）。** 判据放过一个错位读回的第 0 代根，当且仅当没有 txg 落在这一槽的 t ≤ 第一遍最大 txg（`take(3S)` 那一段）。区域 1、2 的槽 0 对应 t = 1、2，区域 0 的槽 0 对应 t = 3S。清扫要删的候选行属于实例 j，它在环里唯一的根在被错位读的槽 X，实例 k > j 是当前挂载的实例，它的根在环里且 txg 严格大于 X 里那条根的 txg（`first_txg_of_new_instance` 取 max + 1，`mount.rs:931`）。所以第一遍最大 txg ≥ X 里那条根的 txg + 1，而 X 的第一个落点 t0 ≤ min(那条根的 txg, 3S) 在遍历范围里，判据命中。这条推导依赖当前实例的根比候选行的根新，不依赖环里有没有跳号。破这一前提的要么是 `first_txg_of_new_instance` 的只供测试的甲开关（只会抬高，不降低），要么是读者没把当前实例的根读进第一遍最大 txg（它自己被错位读）；后一种我没展开，留给攻方腿。

读者（恢复、挂载）没有这条第 0 代根判据（`recovery.rs:2010`、`mount.rs:3041` 只有落点核）；C973 一行只写清扫，不在本格里评。

## 三、八格「检查单」逐格回

**1. 走不到的理由二次查实：成立；另记一条旁注。**
- 句子「产品代码里没有新加 `todo!`、`unreachable!`、`assert!`，也没有条款没定的错误成员」：`git diff HEAD -- crates/singlefs-core/src/mount.rs` 的新增行里 `todo!` / `unreachable!` / `unimplemented!` / `expect(` / `unwrap(` / `_ =>` / `panic!` / `assert` 零命中；`git show b14b6fe92 -- crates/singlefs-core/src/instance_table_sweep.rs` 产品那一半新增行里 `todo!` / `unreachable!` / `unimplemented!` 零命中。新错误成员：清扫文件只有 5 个改名（`crates/singlefs-core/src/instance_table_sweep.rs:283`、`:287`、`:298`、`:300`、`:308`），没有新成员；`mount.rs` 的新比法成员（`crates/singlefs-core/src/mount.rs:560`）是比法枚举成员，不是错误成员，条款已定（`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 185 行）。
- 旁注：C973 那一段（`b14b6fe92`，已进 HEAD）新加了一条 `.expect`：`crates/singlefs-core/src/instance_table_sweep.rs:507`–`:511`（「根环至少三个区域各一槽，每槽都交回了一条根」）。这一条不在句子点名的三种里，实现员报告没列它；它是一条走不到的理由，我核了成立：`ROOT_RING_REGIONS = 3`（`crates/singlefs-format/src/lib.rs:290`），每区槽数下界 4（`crates/singlefs-format/src/lib.rs:299`，`RootRingSlotsPerRegion::from_system_configuration_field` 判区间 `crates/singlefs-core/src/root_ring.rs:123`–`:124`），`every_root_ring_slot` 交回 3S ≥ 12 个槽（`crates/singlefs-core/src/recovery.rs:1960`–`:1967`），第一遍每槽要么 push 要么 `return Err`（`instance_table_sweep.rs:496`–`:506`），所以 `roots_read` 到 `:507` 非空。理由成立，只是「没有」这句话该多写一条。
- 同一次提交里 `:570`、`:571` 的两个 `.expect`（根槽宽、区域号）是从旧的内联读法搬进新函数 `root_read_once_from_the_ring_slot` 的，不是新理由（`git log -S` 它们早在 `f8a9ddaab`）。

**2. 类型与名字：成立。**
- 新名：`mount.rs:560`、`mount.rs:9233`、`instance_table_sweep.rs:601` 三处与正文一致；名字开头没有单字母冠词；`git grep` 旧名（`\bARootRingSlot`、`\bAReadOfAPlacement`、`\bAUnitWrittenBy`、`\bAHeadRereadBefore`）在 `crates` 下零命中。
- `git diff HEAD -- crates/singlefs-core/src/mount.rs` 新增行没有 `_ =>`、`expect(`、`unwrap()`（上一格的命令）。`witnesses_a_publish_newer_than_the_selected_version` 的 `match` 新增一臂，没有通配（`crates/singlefs-core/src/mount.rs:504`–`:524`）。
- txg、槽号、区域号：判据函数签名里 txg 是 `CheckpointTxg`、槽是 `RootRingSlot`、每区槽数是 `RootRingSlotsPerRegion`（`instance_table_sweep.rs:601`–`:607`）；函数里 `GENESIS_CHECKPOINT_TXG.0 + 1..=highest_txg_of_the_pass.0` 把 u64 拆出来做区间、再包回 `CheckpointTxg(txg)`（`:611`–`:613`），局部且 +1 不会溢出（起点 0）。`ring_length_in_slots: usize` 是个裸计数，来源 `every_root_ring_slot(...).len()`（`:476`–`:477`），没有同类裸整数混用。`RootRingSlot` 自己的两个字段是裸 u64（`crates/singlefs-core/src/root_ring.rs:13`–`:16`），是这一批之前的形，不是本批新增。

**3. 不兼容旧代码与旧格式：成立。** 新增行里 `旧版 / 旧布局 / 迁移 / 兼容 / legacy / migrat / compat / 版本号 / layout` 零命中（两个文件的新增行）。「布局位不认」是读到的一槽按现格式判不认，不是认旧版本的分支（`crates/singlefs-core/src/recovery.rs:5168`–`:5172`，这一批没动）。

**4. 不新建判定库：成立。** 两个核心文件新增行与新测试文件里 `rocksdb / sqlite / .tsv / prune / std::fs / File:: / Command` 零命中；新判法只返回 `bool`，不存东西。

**5. 所有任务多机可用：成立。** 同上两处新增内容里 `hostname / std::env / /home / nfs / std::fs / File::` 零命中。新测试用 `SparseBlockDevice` 内存盘（`...as_read_errors.rs` 第 60 行起的 import 与 `DEVICE_BYTES` 常量），没读本机状态。新测试文件头注释里提到的 `git show HEAD:research/results/...` 是文档注释，不被执行。

**6. 测试优先多线程：成立（不需要加并行）。** 新测试文件 11 条 `#[test]`（`grep -c '^#\[test\]'` 得 11），每条一个盘面一次挂载（六格 6 条、手造盘面 3 条、撕裂轮换 1 条、重读 1 条）；文件里没有 `static`、`thread_local`、`Mutex`、`OnceLock`（grep 零命中），各条互不共享状态，`cargo test` 默认的并行线程就够，没有可切的片。随机历史与崩溃注入沿用已有跑法，本轮没动。

**7. 带 --item 的命令与项：部分成立，后半待主 agent 跑。**
- 22 行变异：锚点各命中一次（本报告开头，29 行里含这 22 行）；会不会红是任务 10223 的读数，我没复跑。
- 崩溃点重放两项在登记表里：`.claude/gate.d/crash-point-replay-items.tsv` 第 63、64 行（`read-faulted-states-never-hold-the-witness-criterion`、`random-histories-and-fault-injection-given-states`），`.claude/gate.d/stage-inputs.tsv` 第 93、94 行各点名一个测试函数，两个函数都在：`crates/singlefs-verification/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs:1003`、`crates/singlefs-verification/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs:1329`。
- 「能被 `--list-items` 列出」：待主 agent 跑（命令 3）。

**8. 新崩溃流多罩了什么：成立（读码），数是主 agent 的读数。**
- 新步 `WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros` 只藏按盘次序排第一的那块盘最新那一槽（`crates/singlefs-verification/src/history.rs:7961`–`:7963` 注释，`:8001`–`:8004` 的 `hidden.truncate(1)` 与模型答 `answer_mount_writable`）。随机历史的池是两块盘（`history.rs:2791`），另一块盘两槽都自证过，R2 与乙入判不出都判得出，可写挂载往下走。
- 它多罩的是乙入判不出放行的那一支：每块盘都藏那一步现在必拒（`history.rs:7958`–`:7960` 注释，模型答在 `:7998`–`:8000`），拒了盘面不变，后面没有挂载之后的状态可走；只有新步把最新一槽读回全 0 但挂载做成之后的逐盘核（`mount.rs:6776`–`:6810`）、取号补写、随后的崩溃枚举与给定状态引擎都继续走起来。变异 `crates/mutations.tsv` 第 7061、7063 行（把只藏一块盘改成每块盘都藏、抽样改格）钉的就是这一点。
- 没罩的：三块盘以上（池固定两块）；读回的是撕裂字节而不是全 0（这一组比重只造全 0）；两块盘各藏一槽但不是同一槽的组合。
- 取样点读数（每块盘都藏 Ok 0 / Err 6，只藏一块盘 Ok 23 / Err 0）是正文写的任务 10211 的读数，我没复跑，当主 agent 的读数。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| C971 认的槽（读得出而自证不过、读回全零） | 成立，一处边要看 | 读回全零与自证不过都经 `parse_slot` 拒成 `NotSelfDescribing` 记坏；节点大小字段不对也记坏（与条款相容）；fsid 不符的槽不记（F1） |
| C971 fsid 不符的槽 | 打中？（`crates/singlefs-core/src/recovery.rs:5151`–`:5157`） | 条款字面「自证过」含 fsid 相同、不并进只列两样，实现与判官都不并进；是读法分歧，主 agent 定 |
| C971 重读恰好一次 | 成立 | 先读一遍、至多重读一遍（`mount.rs:9324`、`:9334`） |
| C971 读的是系统配置槽不是根 | 成立，粒度粗一档（F2） | 系统配置槽直接从盘上重读，但重读的是整个读阶段，触发条件是整个判据，条款写「重读那一槽」 |
| C971 R2 的另一半照旧 | 成立 | `mount.rs:9239`–`:9241` 与 `recovery.rs:5194`–`:5196` 同一个量，乙压在「没见证」之前与 R2 同位 |
| C971 拒在第一个写之前 | 成立（读码），盘面不变待主 agent 跑 | 调用点 `mount.rs:9720` 在取号之前，之前各步只持共享引用；用例断言 `image_after == history.image` |
| C971 拒因沿用已有成员 | 成立，两条轻 | 核心层只有一个读者，只读挂载、取号、F2 退回、逐盘核、离线出路都没动；成员文档（F3）与非拒挂载记 `OnTheOneReread`（F4）两处没跟上 |
| C973 调读者同一个落点核 | 成立 | `instance_table_sweep.rs:583`–`:589` 与读者同一个函数 `root_ring.rs:183` |
| C973 错位读重读一次、仍不过不删 | 成立 | `:480`–`:500`，用例 `:1445` |
| C973 第 0 代根判据 | 成立，措辞差一处（F5） | 实现是「最大 txg 已到」的上界式，行里字面「已有根」按字面永不成立；txg 跳号时多不删，不错删 |
| C973 判据走原有重读路 | 成立 | `:523`–`:550`，四条用例 `:1703`、`:1730`、`:1748`、`:1924` |
| 检查单 1 走不到的理由二次查实 | 成立，旁注（F6） | 三种宏与错误成员零新增；清扫新加了一条 `.expect`（`:507`–`:511`），理由我核了成立，实现员报告没列 |
| 检查单 2 类型与名字 | 成立 | txg / 槽 / 每区槽数都是 newtype，局部拆 u64 做区间；无通配臂，旧名零残留 |
| 检查单 3 不兼容旧代码与旧格式 | 成立 | 新增行零命中旧版 / 迁移 / 兼容字样 |
| 检查单 4 不新建判定库 | 成立 | 新增行与新测试零命中存储字样 |
| 检查单 5 所有任务多机可用 | 成立 | 零命中主机 / 环境 / 文件系统读取，用内存盘 |
| 检查单 6 测试优先多线程 | 成立 | 11 条单场景用例，无共享状态，默认并行即可，没有可切的片 |
| 检查单 7 带 --item 的命令与项 | 部分成立，待主 agent 跑 | 22 行变异锚点各命中一次、两个崩溃项登记在案；`--list-items` 待跑（命令 3） |
| 检查单 8 新崩溃流多罩了什么 | 成立（读码） | 多罩乙入判不出放行的那一支及其后续状态；没罩三块盘以上与撕裂字节 |

发现（供主 agent 逐条现查，不是我的判决）：
- F1 fsid 不符槽：见第一节。推翻条件：C707 / 用户原话「按自证不过处置」只管择系统配置那一步；或找到乙入判不出登记里说明 fsid 不符不进乙的话。
- F2 重读粒度：见第一节第 2 点。推翻条件：构造一块盘一槽好一槽坏、重读时好槽读坏坏槽读好，且另一块盘没有两槽都好的盘面，两种读法的结局相同。
- F3 成员文档：`crates/singlefs-core/src/mount.rs:399`–`:401`。
- F4 `rereads.read_stage`：`crates/singlefs-core/src/mount.rs:9358`–`:9364`。
- F5 上界式：见第二节第 3 点。推翻条件：造 txg 1 没写过的池，清扫仍照删。
- F6 旁注：`crates/singlefs-core/src/instance_table_sweep.rs:507`–`:511`。
- F7 三块盘以上没有用例罩：随机历史（`crates/singlefs-verification/src/history.rs:2791`）与新测试都是两块盘；判法本身对盘数是通用的（`mount.rs:9167`、`:9239`–`:9241`）。

## 没做什么

- 没替主 agent 采纳或出判决；上面「发现」都是线索，由主 agent 现查。
- 没往统一任务服务交任务，三条命令写在开头「要主 agent 跑的命令」，那三格写「待主 agent 跑」；判决要引的数没复跑。
- 没写模型、没跑装置，也就没有 `rerun.sh`。
- 没攻：重读那一次本身读故障的走向、错位读回别槽非第 0 代根时两条判据叠加、落点公式 txg 相等等，是攻方腿的面；我只在第二节第 5 点给了判据够用的正推。
- 没读也没引：攻方腿的报告、本轮之外的 `*-main-verification.md`。
- 没核：`crates/singlefs-verification/src/mount_transparency.rs`、`model.rs`、`history.rs` 的改动里除引用到的几处之外的部分；`research/results/` 下 E205 产物的读数。
