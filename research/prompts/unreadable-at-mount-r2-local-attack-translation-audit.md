# unreadable-at-mount-r2 本地攻方：提示转述核对表（2026-09-28）

提示：`research/prompts/unreadable-at-mount-r2-local-attack.md`（105 行，sha256 `0f5c4a4caa2a618f2271925382c5c9aec4ad459189aabddd0601af8a2cd0224e`），攻击面是正文「Y4　轨迹网格」。「英文项」列写提示里的事实号（F01…）与提示文件自己的行号。

- kb 行号是这一次现查的；kb 12 份与 `research/prompts/unreadable-at-mount-r2-snapshot/kb-sha256.txt` 逐份对得上（`sha256sum -c`：12 OK）。
- 代码行号取自冻结副本 `/tmp/claude-1000/unreadable-at-mount-r2/tree/crates/`（`crates-sha256.txt` 174 OK）；下表写成仓内相对路径，因为用到的 `mount.rs`、`transaction.rs`、`recovery.rs`、`root_ring.rs`、`mounted_read.rs`、`block_device.rs` 与 `crates/singlefs-harness/tests/common/mod.rs` 今天与主工作区逐字节相同（`cmp`）。
- 缩写：`正文` = `research/prompts/_unreadable-at-mount-r2-body.md`；`D16` = `.claude/kb/decisions/16-发布语义.md`；`D22` = `.claude/kb/decisions/22-单元原子性怎么合成.md`；`D23` = `.claude/kb/decisions/23-journal的角色与格式.md`；`INV` = `.claude/kb/invariants.md`；`core/` = `crates/singlefs-core/src/`；`common` = `crates/singlefs-harness/tests/common/mod.rs`；`M1` = `research/prompts/unreadable-at-mount-r1-opus-model/`（第一轮攻方模型目录，不在这一轮禁读清单里）；`网格` = `M1/c331_candidates_grid.rs`；`base日志` = `M1/runs/base-c331_candidates_grid.log`；`jia日志` = `M1/runs/jia-c331_candidates_grid.log`；`c393汇总` = `M1/c393-summary.txt`；`甲diff` = `M1/candidate-jia.diff`；`(a)diff` = `M1/candidate-a.diff`；`C393报告` = `research/prompts/closeout-recheck-2026-09-28/c393-investigator-report.md`；`一轮判决` = `research/prompts/unreadable-at-mount-r1-main-verification.md`。
- 「首稿」= 草稿目录 `/tmp/claude-1000/unreadable-at-mount-r2/local-attack/prompt-draft1.md`；「定稿」= 仓里的提示（草稿目录 `prompt-final.md` 是它的副本）。定稿与英文项相同的写「同」。
- ⚠️ 「只读挂载不写盘」kb 里没有条款，提示里只写成代码事实 F21（`mount_read_only` 的注释与签名），不写成规则。

## 一、提问框架与行、列

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 「The mount sequence」：之后连挂 mount 1、2、3（可能更多），每次先试可写、被拒就只读（第 15 行） | 正文:25「之后连挂 N 次（N ≥ 3，每次先试可写、不成就只读）」；正文:34「格里填第 1、2、3 次挂载各是「可写 / 只读」，以及第 3 次之后是否还会变」 | 无 | 同 |
| 标签 W / R（第 17 行） | 正文:34「可写 / 只读」 | 无 | 同 |
| Q1 行：torn_both、torn_d0、torn_d0_d1_persisted、tree table、allocation record tree；列：Today、Jia、Jia-read-error、(a)、(b)-from-tree-table（第 95 行） | 正文:34「行是 {撕裂轮换三形, 树表持续读不出, 分配记录树持续读不出}，列是 {今天, 甲, 甲-读错, (a), (b)-从树表现算}」 | 无 | 同（撕裂轮换三形拆成三行，见第五节） |
| Q1 每格「facts」写事实号（第 99 行） | 正文:34「每格写推导依据的事实表行号」 | 无 | 同 |
| Q1「after mount 3: STAYS or CAN CHANGE」（第 97 行） | 正文:34「第 3 次之后是否还会变」 | 无 | 同 |
| Q1「disk between mounts」（第 98 行） | 正文:25「逐次记：可写成没成、盘上变没变、撕裂那一槽或读不出的账有没有被重写 / 离开根环」 | 无 | 同（Y1 的逐次记录项，放进 Y4 让两边可对格，见第五节） |

## 二、行的历史（事实 A 部）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| F01 实例 1 暖机 txg 1、2，A 在 txg 3（第 23 行） | D22:336「暖机空发布 2 次、新池新建文件 txg 3 是格式常量」；网格:98 | 无 | 同 |
| F01 重开取号 2，写行 4、暖机 5，B 6、C 7，全部确认返回 | 网格:98「A（实例 1，txg 3）→ 重开，实例 2 写行 4、暖机 5，再发 … 次覆盖写（txg 6 起），全部确认返回」；网格:103-105 | 首稿写「Reopen with no fault as instance 2 (that mount selected A)」，括注「选了 A」是我推的，网格里没写 | 删括注，改「Reopen as instance 2, with no fault injected」；选根由 F22 的 `choose_root` 给 |
| F01 D (2, 8)：根 FUA 落盘、记录落盘，轮换系统配置那一写（每块盘一写）落到（D 之前这块盘自证过的最大世代号 + 1）mod 2 那一槽，按行撕裂或没持久，之后崩溃，D 没确认返回 | 网格:180-181「标准历史之后实例 2 再发 D（txg 8）：根 FUA 落盘、记录落盘，轮换系统配置那一写（每块盘一写，同一段）按 `rotation` 撕裂或没持久，之后崩溃——D 没确认返回」；网格:198-208（落槽 = (D 之前最大世代号 + 1) % 2） | 缺「系统配置槽写是原地覆写、长于一个扇区」 | 补「The system configuration slot write is an in-place overwrite longer than one sector.」（网格:181「系统配置槽写是原地覆写、长于一个扇区，取第三态」）；「同一段」不译：一次写一段，与「one write per device」同义 |
| F01「In the torn rows no read fault is injected anywhere」 | 网格:181「零读故障就是层 0 枚举得到的合法崩溃状态」；base日志:97 `slots=None … hidden=None` | 无（「合法崩溃状态」这一判断不译：Y3 在复核它，写进提示就是替模型下了判断） | 同 |
| F02 撕裂字节：不同的那一截前一半新、后一半旧；读得出字节、整槽校验和不过 | 网格:156-157「同一次写撕裂时「新旧都读不出」的字节（…不同的那一截前一半新、后一半旧）」；网格:158-168；D22:199 | 缺「新旧都读不出」 | 补「so neither the old nor the new content can be read from it」 |
| F03 torn_both：两块盘都撕裂；自证槽 (0, 10, 7)、(1, 10, 7) | 网格:174-175、215；base日志:26「prefix torn_both: tails after the torn rotation [(0, 10, 7), (1, 10, 7)]」；三元组含义 网格:93（盘、槽世代号、journal_tail） | 无 | 同 |
| F04 torn_d0：盘 0 撕裂、盘 1 没持久（留旧字节）；(0, 10, 7)、(1, 10, 7)、(1, 9, 6) | 网格:172-173、213-214；base日志:9 | 无 | 同 |
| F05 torn_d0_d1_persisted：盘 0 撕裂、盘 1 持久；(0, 10, 7)、(1, 10, 7)、(1, 11, 8) | 网格:176-177、216；base日志:8 | 无 | 同 |
| F06 C393 历史：A、B → 取号 2 → C (2, 8) → 崩溃恢复抛弃 C（C 的根槽与数据单元暂时读不出、每块盘世代号最大那一槽被清零，落到 (2, 7)，实例 3 写行 9、暖机 10）→ C 又读得出；C 是还在根环里的被抛弃根 | C393报告:8；common:427-432；C393报告:141「txg 14 的时候 C 还在根环里」 | 无（「见证 C 的系统配置槽坏掉」按 common:429、common:491 写成「最大世代号那一槽清零」，那是它的做法；「C 写回」译成「becomes readable again」：撤掉暂存、原样写回字节） | 同 |
| F07 清零那一槽被恢复那次挂载的取号写落回 | common:491「清零、不写回：这次挂载的取号写正好落回这一槽（世代号取读得出的最大 + 1）」 | 无 | 同 |
| F08 树表那一行：从 mount 1 起 C 的树表两份每次读都读不出、每次挂载都是；别处无读故障 | 正文:25「(a)、(b)：被抛弃根的树表 / 分配记录树持续读不出」；C393报告:19「树表（槽 50342）｜一直读不出」；c393汇总:72 | 无 | 同（「on every read, in every mount」是对「持续」的展开，见第五节） |
| F09 分配记录树那一行：C 的分配记录树根节点两份持续读不出，C 的树表读得出 | 正文:25；C393报告:16「分配记录树（根节点，槽 50339）｜一直读不出」；c393汇总:51 | 无 | 同（「C's tree table stays readable」见第五节） |

## 三、规则（事实 B 部，kb）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| F10 发布持久顺序：COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障；fsync 等轮换持久才返回；根槽持久之后再更新系统配置、每个 checkpoint 一次（第 43 行） | D16:179 | 无（不译的：重放施加前逐项验校验和、进层 0 枚举、抬 F 那一串多一步、新实例根覆盖两块盘之前 fsync 不返回、管理员回退的确认条件、实现待派；都不碰「第几次挂载可写」这件事） | 同 |
| F11 每盘恒 2 槽，世代号从 1 起、每写一次 +1，下一次写的槽 = 世代号 mod 2；择槽取校验和过且世代号最大 | D22:330 | 首稿把「逐盘计」挂在择槽上（「counted per device」接在择槽句末），原文「逐盘计」管的是三种写 | 改成「Counted per device, all three kinds of system configuration write … write the largest self-verified generation on this device + 1」 |
| F11 取号、发布末尾的轮换、取号失败的回卷三种写都写最大世代号 + 1 | D22:330「取号、发布末尾的轮换、取号全或无失败时的回卷三种系统配置写都写这块盘上自证过的槽里最大的世代号 + 1」 | 缺「全或无」 | 补「after an all-or-nothing acquisition failure」；槽 i 的偏移 = i × 槽距、根环区域公式不译（地址，不碰轨迹） |
| F12 跨过之后一次写可能落一半，整槽校验和判出，择槽规则自动回退到另一个槽；可检测、可恢复；撕裂那次挂载拿到上一代配置（第 47 行） | D22:199-200 | 缺三处：「内容允许跨过 512（探测到的 physical_block_size）」这一前件；「覆盖整槽 4096 含补齐」；同一分项的 ⚠️「今天 489 没跨 512，这条是允许不是已经跨」 | 三处都补上（D22:199、D22:202）；「层 0 要多枚举一个撕裂维度」不译（验证代价，不碰轨迹） |
| F13 读阶段读完，取池里每块盘两槽里全部自证过的系统配置槽 journal tail 最大值 c_witness；0 不判；末条记录读得出比计数器；读不出而计数器等于 c_witness 的记录读得出比 (实例代号, checkpoint_txg)；两条都读不出按真（第 49 行） | D23:389 | 缺「每块盘两槽」 | 补「among the two slots of every device in the pool」 |
| F13 为真就重做读阶段一遍，重做为假就用它往下走，仍为真就在取号之前拒可写、盘上逐字节不变，只读照常 | D23:389 | 缺「在这次挂载内有效的读缓存（只收读成且不是全零的落点）上」与「R = 1，取自 D16 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等」 | 补「on the read cache that is valid within this mount (…); R = 1, with no waiting between the two reads」；「D16 已定项 1 那一行」不译（模型读不到 D16） |
| F13 重建分配器判抛弃用的最新那条根的实例表读不出，同样重读一次、仍读不出拒可写 | D23:389 | 首稿整句缺 | 补上；「代码审阅第 22 条」出处不译 |
| F13「续」：取号那一写不再写 tail 0，写取号那一刻按同一取法读到的见证值（每块盘两槽里全部自证过、fsid 与本池相同的槽 journal_tail 取最大，一份都没有时 0，第一道屏障之后、第一个取号写之前读一次）；回卷写带同一个见证值 | D23:389 | 首稿整段缺 | 补上；「不退回 0」不单译（与「带同一个见证值」同义）；函数名 `write_acquired_instance`、`highest_system_configuration_journal_tail` 与用户定案、实现员交回的出处不译 |
| F13 取号那一刻某块盘一份自证过的系统配置槽都没有：拒；实现在做 | D23:389「取号那一刻某块盘一份自证过的系统配置槽都没有（见证值那一遍读错，逐盘核那一遍读得出）：拒（主 agent 2026-09-27 定），实现在做（实审 A3c）」 | 首稿整句缺；第二稿补了主句仍缺括注 | 补主句、括注「(the pass that reads the witness value fails while the per-device check pass before it could read)」与「the rule text says this refusal is being implemented」；「主 agent 2026-09-27 定」「实审 A3c」不译 |
| F13 系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到；记欠 | D23:389 | 首稿与第二稿都缺，定稿之前补的（torn_both、torn_d0 里的 D 正是这一形） | 补「A newest root that the system configuration did not witness: … recorded as an open item」；C554 与 Q1 编号不译 |
| F13「mkfs 写 tail 0；mkfs 之后第一次取号读到 0、写 0」 | D23:389 | 不译 | 不译：这几行历史里没有 mkfs 之后的第一次取号 |
| F14 被抛弃时间线今天只由崩溃恢复造出；它们引用的块在离开根环之前同样不许重新分配、不许抹头（第 51 行） | INV:55 | 缺括注「管理员回退是挂着时的一次向前发布，不抛弃任何根」与「同样」 | 补括注与「likewise」；I-7.4 的回退候选集定义、F_生效、checker 那一半不译（F27 已给 checker 红的实测，判抛弃的定义由 F22 按代码给） |

## 四、代码、实测与候选（事实 C、D、E 部）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| F15 读不出的槽与自证不过的槽同样掉出 c_witness；一槽都不剩时 0（NothingWitnessed，判假）；Undecidable 判真（第 55 行） | core/mount.rs:4146-4159、core/mount.rs:4160-4161、core/mount.rs:384-399；core/recovery.rs:2861-2871 | 无 | 同 |
| F16 每一遍见证读都直接从盘读系统配置槽；第一遍假就用它；真就调钩子、重做；重做仍真拒 `NewerStateStillUnreadableAfterOneReread`（取号之前、盘上不变）；重做假用重做那一遍（第 57 行） | core/mount.rs:4195「再判 N-配置（系统配置槽经 `devices` 直接读）」；core/mount.rs:4242-4266；core/mount.rs:4275 | 无 | 同 |
| F17 挂载内次序：读阶段与它的一次重读 → 重建上一版 → `rebuilt_allocator`（读最新根的实例表、算影子账）→ `establish_instance`（取号写与新实例的发布）（第 59 行） | core/mount.rs:4311、core/mount.rs:4333、core/mount.rs:4348、core/mount.rs:4367；core/mount.rs:1417-1438 | 无 | 同 |
| F17 `establish_instance` 之前各步拿到的都是盘的共享引用，块设备的写要可变引用；在它之前交回的拒盘上逐字节不变 | core/mount.rs:1026-1027、core/mount.rs:1090-1091、core/mount.rs:1126-1127、core/mount.rs:1378-1379（`&Vec` / `&[..]`）；core/block_device.rs:112-113（`fn write_at(&mut self, …)`）；core/mount.rs:1173「可写挂载在取号之前调它，拒的时候盘上逐字节不变」；core/mount.rs:4275 | 首稿只有「A refusal returned before establish_instance leaves the disks unchanged」一句，没有它凭的代码事实 | 补「Every step before establish_instance receives the devices by shared reference, and the block device write operation needs a mutable reference.」 |
| F18 写的世代号 = 这块盘自证槽最大世代号 + 1；落槽 = 世代号 mod 2（第 61 行） | core/transaction.rs:344「世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2」；core/transaction.rs:361-366、core/transaction.rs:407-410 | 缺「两槽都读不出时从 1 起」 | 补「or 1 when no slot of that device self-verifies」 |
| F19 取号时逐盘再读两槽，某块盘自证槽 0 份就失败，至少 1 份就过；取号写每块盘写一槽、带这一刻读到的最大 journal tail（第 63 行） | core/transaction.rs:630-664（拒在 654-660）；core/transaction.rs:840-850 | 无 | 同 |
| F20 取号之后新实例写行发布与暖机发布，每次以每块盘一次系统配置轮换收尾；F06 那段历史实例 3 写行 9、暖机 10；撕裂那几行今天 mount 1 成了实例 3，之后三次覆盖写落在 txg 11、12、13（第 65 行） | D16:179（每次发布末尾轮换系统配置）；core/mount.rs:2936、core/mount.rs:2968（`warm_up_publish_txgs`、`instance_rows_to_write`）；C393报告:8；base日志:97「k1=(3,11)->kept(3,11) k2=(3,12)->kept(3,12) k3=(3,13)->kept(3,13)」 | 无 | 同 |
| F21 只读挂载：择系统配置、择根、扫 journal、重放、沿施加之后的根打开；一个字节都不写；给它的读者只有读操作（第 67 行） | core/mounted_read.rs:544「只读挂载：择系统配置 → 择根 → 全环扫描 journal → 按前缀口径施加 → 沿施加之后的根打开挂载态。」、core/mounted_read.rs:545「一个字节都不写（并行线二「这条线不写盘」）。」、core/mounted_read.rs:553（`reader: &dyn PoolReader`）；core/recovery.rs:81-101（`PoolReader` 只有 `device_identities`、`device_size_in_bytes`、`read`、`journal_record_offsets_hint`） | 无 | 同（这一行是代码事实，不是 kb 条款） |
| F22 `choose_root` 取根环里全部自证过的根中 (checkpoint_txg, 实例代号) 最大的（第 69 行） | core/recovery.rs:1147-1168 | 首稿与第二稿都没有；没有它模型推不出撕裂那几行的实例 2 行是 (1, 3)、D 不被判抛弃 | 定稿之前补上 |
| F22 (i, T) 被抛弃 ⟺ 最新根的实例表有行 (i, Ti, …) 且 T > Ti；新实例取号写它接续的那个实例的行（那个实例、所选根的 txg、…） | core/recovery.rs:1392-1404；core/mount.rs:4362-4366 | 无 | 同 |
| F23 影子账每次可写挂载都跑，对象是根环里读得出的根；每条被抛弃根调一次 `placements_referenced_by_root`（读它的树表与分配记录树）；None 就计数 +1、continue：不隔离、不重读、不报错、不拒（第 71 行） | core/mount.rs:1417、core/mount.rs:1438；core/mount.rs:1251-1255；core/mount.rs:1215「树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载」；core/mount.rs:1296 | 无 | 同 |
| F23 隔离位在那次挂载的分配器里、独立于分配位；每次挂载与每次抬 F 现算；被抛弃根的隔离只在它离开根环的那一次发布里清 | core/mount.rs:1216「隔离位独立于分配位（`DeviceFreeMap::isolate`）」；core/mount.rs:2352「窄读法要按每次挂载与每次抬 F 的候选集现算」；core/mount.rs:1217「清只在被抛弃的根离开根环的那一次发布里清」 | 首稿写「a bit in the in-memory allocator」，「in-memory」是我推的，引到的行没写 | 删「in-memory」，改「a bit in the allocator of that mount, separate from the allocation bit」 |
| F24 发布落哪个根槽只由 txg 定；根只在后来的发布盖掉它的槽时离开根环；F06 那段历史 C 的根槽在 txg 32 被盖掉（第 73 行） | core/root_ring.rs:107-117；D22:330「区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定」；C393报告:31「txg 32 的根会盖掉 C 的根槽」、C393报告:141 | 无 | 同 |
| Part D 标题「Measured results of mount 1 only (no second mount was measured)」（第 75 行） | 一轮判决:48「攻方没在副本上连挂两次」 | 无（同一行里的推断句不译：那是这一轮要验的结论） | 同 |
| F25 今天、撕裂三行 mount 1：可写不丢；所选 D (2, 8)，末条记录 8；第一遍 c_witness 7（torn_both、torn_d0）/ 8（torn_d0_d1_persisted）；第一遍就判完（第 77 行） | base日志:97「prefix=torn_both … verdict=writable_no_loss effective=(2,8) … read_stage=first[sel(2,8)w7:last_record=8]」；base日志:98（torn_d0_d1_persisted，`w8`）；base日志:553（torn_d0，`w7`） | 无 | 同 |
| F26 甲、撕裂三行 mount 1：拒可写，`NewerStateStillUnreadableAfterOneReread`（第 79 行） | jia日志:94、jia日志:104、jia日志:483 | 无 | 同 |
| F27 今天、树表行与分配记录树行 mount 1：可写；计数 1；隔离 0 / 0；挂载之后第一次覆盖写（txg 14）复用 C 的数据单元；I-7.4 红；读阶段第一遍判完，所选 (3, 10) 就是系统配置见证到的那次（jsn 10）（第 81 行） | C393报告:16、C393报告:19、C393报告:25；c393汇总:51、c393汇总:72（今天那一栏 `ok iso=0/0 unread=1 REUSE@14(abandoned#0) I74red`） | 无 | 同 |
| F28 (a)、两行 mount 1：拒可写，理由点名被抛弃根的账（第 83 行） | c393汇总:51、c393汇总:72（(a) 那一栏 `refused(AccountOfAnAbandonedRoot)`） | 无 | 同 |
| 候选 Today：Part C 的代码（第 87 行） | 正文:54「方案按 `crates/` 今天的实现来谈」 | 无 | 同 |
| 候选 Jia：见证读里有一槽读不出或自证不过就当 Undecidable（真），重做一遍，重做仍有就拒可写（第 88 行） | 正文:9「甲（见证读里有一个系统配置槽读不出或自证不过就当判不出，走重读，仍有就拒可写）」；甲diff:29-33 | 无 | 同 |
| 候选 Jia：量过的那一版是「某块盘自证槽少于 2 份」；取号扩展：取号时某块盘自证槽少于 2 份就拒 | 甲diff:30-32（`slots_of_this_device.len() < 2`）；甲diff:47-49；一轮判决:54「甲 的取号扩展（取号写之前见证读有槽读不出同样判不出）」 | 无 | 同 |
| 候选 Jia-read-error：只有设备报读错（EIO）的槽当判不出；自证不过的槽（撕裂、全零、内容坏）照今天不进 max；取号扩展照同一判不出定义（第 89 行） | 正文:9「只有设备报读错（EIO）的槽当判不出；自证不过的槽（撕裂、全零、内容坏）照今天不进 max」；正文:28「取号扩展照甲-读错 的判不出定义」 | 无（正文:9「它挡 `DeviceError` 形、放 `Zeros` 形」不译：那是 Y2 网格上的结论，写进来就替模型答了） | 同 |
| 候选 (a)：影子账读一条被抛弃根的账（树表或分配记录树）读不出，重读一次，仍这样就拒可写、只读照常；量过的那一版在 `rebuilt_allocator` 里、`establish_instance` 之前拒（第 90 行） | 正文:11「(a) 重读一次仍读不出就拒可写」；(a)diff:8「影子账读一条被抛弃根的账（树表或分配记录树）读不出、解不开，重读一次仍是这样：拒可写」；(a)diff:30「仍读不出就拒可写、只读照常」；(a)diff:39、(a)diff:71-72 | 缺「解不开」 | 补「cannot read or cannot decode」，「still unreadable」改「still so」 |
| 候选 (b)-from-tree-table：分配记录树读不出而同一条根的树表读得出时，从树表走读现算这条根引用的槽，只在内存里隔离、不写盘；树表读不出时照 (a)（第 91 行） | 正文:11 | 无 | 同 |
| 正文:9、正文:11 两处「推断会永远只读——推的，没量」 | 正文:9、正文:11 | 不译 | 不译：那是这一轮 Y1、Y4 要验的推断，写进提示就是替模型下结论 |

## 五、英文比原文多出来的

| 多出来的 | 提示里在哪 | 为什么加 |
|---|---|---|
| 撕裂轮换三形拆成三行（torn_both、torn_d0、torn_d0_d1_persisted），网格 25 格 | 第 95 行 | 派发把三形点名列出；三形的自证槽不同（F03–F05），答案可能不同，合成一行就藏住了差别 |
| 「Each mount ends before the next one starts. The fault named by the row stays in place for all mounts; no other fault is added and nothing lifts the fault. Nothing other than these mounts writes to the disks.」 | 第 15 行 | 正文只写「连挂 N 次」「持续读不出」；不写死这几条，「永远只读」与「故障被撤掉」「外人写盘」分不开，每格都会变成开放题 |
| 「The question does not fix whether the user writes files during a writable mount.」 | 第 15 行 | 正文没定可写挂载里用户写不写；写明没定，让模型在要用到时写进 assumption，不自己默认 |
| F08、F09「on every read, in every mount」 | 第 37、39 行 | 「持续读不出」的展开；第一轮攻方的「一直读不出」臂（C393报告:16、C393报告:19）就是每次读都坏，与「只坏第一次读」那几臂分开 |
| F09「C's tree table stays readable」 | 第 39 行 | 分配记录树那一行只坏分配记录树根（c393汇总:51 `AllocationRecordTreeRoot`）；(b) 要看树表读不读得出，不写死这一格就开着 |
| 候选各条末尾「Everything else as today」 | 第 88–91 行 | 正文的候选定义只写改了什么；不写这句，模型会在没改的那一半替候选补行为 |
| Notation 里「A slot "self-verifies" when …」 | 第 13 行 | 把「自证过」钉成可读、整槽校验和过、fsid 相同三件事（core/recovery.rs:2861-2871、core/transaction.rs:630），F15 与候选甲、甲-读错都靠它分「读不出」与「自证不过」 |
| F13 括注「the per-device check pass before it」里的「before it」 | 第 49 行 | D23:389 只写「逐盘核那一遍」；先后取自 core/transaction.rs:843「屏障前那一核与这一次读之间又有一块盘读不出」 |
| F21「(list devices, device size, read, journal offset hint)」 | 第 67 行 | 把「只有读操作」落成 `PoolReader` 的四个方法（core/recovery.rs:81-101），模型能按名核 |
| 每格要答「disk between mounts」 | 第 98 行 | 这是 Y1 逐次要记的项（正文:25）；本地腿不跑，但要写出推的「盘上变没变」，主 agent 才能与 Y1 逐格对（正文:52「Y4 与 Y1 同一格不同的，主 agent 现查坐实哪一边」） |
| 规则第 3 条「不写代码行号与文件行号，按事实号与函数名指」 | 第 8 行 | 定义要求；样本里自给的行号一律标「模型自给、未核」 |
