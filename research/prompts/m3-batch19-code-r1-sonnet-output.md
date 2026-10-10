# 删除批第 19 批代码轮第一轮：云端正推腿报告（K1、K3）

轮：m3-batch19-code-r1。腿：云端正推（Sonnet）。日期：2026-10-05。方案按 `crates/` 今天的实现来谈。
立场：条款、判决与用户定案 → 冻结副本里的行 → 钉它的测试与变异行；能复跑的在冻结副本的拷贝上复跑，只跑 harness 档和 `singlefs-core --lib`，没跑 checker 档。
禁读清单里的两处（攻方腿报告与模型）一个字没读。

## 0　口径与复跑

- 冻结副本 8 份文件的 sha256，交回前现核，原样输出（8 行 OK，非 OK 行数 0）：
  `crates/mutations.tsv: OK`、`crates/singlefs-core/src/transaction.rs: OK`、`container_trees_grow_a_level_when_the_root_is_full.rs: OK`、`inode_number_watermark_pressed_back_reissues_a_number.rs: OK`、`the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs: OK`、`unlink_mount_continuation_and_the_emptied_version.rs: OK`、`unlink_single_file_publishes.rs: OK`、`unlink_writes_per_transaction_do_not_grow_with_the_file.rs: OK`。
- 复跑在冻结副本的拷贝上（草稿 `/tmp/claude-1000/m3-batch19-code-r1-sonnet/`），每条 cargo 经 `capped.sh 6` 与 `run-with-memory-cap.sh`；会话里同时有攻方腿的 cargo 在跑，设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（两个 agent 各跑各的）。
- 模型目录 `research/prompts/m3-batch19-code-r1-sonnet-model/`：`rerun.sh` 分阶段（setup、base、rows:2670-2676、rows:2677-2682、variants、probes），每阶段单调不超过 240 秒；交回前在空目录 `/tmp/claude-1000/m3-batch19-code-r1-sonnet/rerun1` 里从头各阶段跑通一遍，摘要抄在模型目录 `rerun-summary.txt`（28 行；交回前在第二个空目录 `rerun2` 里 setup、base、rows 两段、variants、probes 各阶段从头又跑了一遍，摘要取这一遍）。
- 基线（未改的冻结副本）：七个目标全绿，结果行为 22、6、9、4、1、1 条过，`singlefs-core --lib -- pressed_back_inode_number_watermark` 2 条过。
- 变异行的原文命中：`mutations.tsv` 全表 2677 行（扣掉冻结副本里不存在的那 1 个 litmus 文件）逐行数原文在被改文件里的命中，非 1 的 0 行；新增 13 行（`mutations.tsv` 第 2670–2682 行）各命中恰好 1 次，行里点名的测试函数名（第 6 段）在目标文件里都有同名函数。

## 1　K1-N1：Q-F 三条判别形与三行变异

### 1.1 条款到实现

- 条款：「unlink 的意图那次写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值」（`.claude/kb/decisions/08-核心索引结构.md:195`）；续做同一口径：「意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间」（`.claude/kb/decisions/08-核心索引结构.md:470`）；用户 Q-F 乙的原话在变更史：「意图那次写进三个时间秒；各批照抄意图那次的值，保住 ①b。」（`.claude/kb/decisions-history.md:3258`）。
- 实现：意图那条记录的时间字段直接取参数：「write_time_seconds,」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:5975`）；各批照抄上一版这条记录：「write_time_seconds: record.write_time_seconds,」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:6038`）；记录序列化把一个值写三格：「writer.put_u64(self.write_time_seconds);」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/records.rs:78`）。会话那一层把调用方的时间原样交给意图：「write_time_seconds,」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:257`）。判定：**一致**。
- 推翻它的现象：任何一条写路径（会话、续做、回退）在意图或批里改了这个值而盘上 64/72/80 三格不等于给的值。本轮复跑里没有。

### 1.2 每条用例断言的是不是盘上三格等于给的值

- 读法读的是盘不是内存：按 `version.units` 里 InodeLeafContainer 的单元槽位去两块盘上读字节，在码 3 记录区找 inode 号是第一个文件的那一条，取偏移 64、72、80：「field_at(record, 64),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:539`）、「field_at(record, 72),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:540`）、「field_at(record, 80),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:541`）。偏移与 D8 字段表一致：「| 64 | atime / mtime / ctime 秒，各 8（unlink 的意图那次三个共写调用方给的一个值，各批照抄） | 24 |」（`.claude/kb/decisions/08-核心索引结构.md:188`）。
- 断言形都是「每版 × 两块盘 × 三格」等于给的值：进程内直发 `vec![vec![[observed.given_time; 3]; 2]; 2]`（第 612 行，意图加一批），会话 `...; 3]`（第 746、766、786、811、834 行，意图加两批），续做 `...; 2]`（第 879、897 行）。三格在写入端本来就是同一个字段写三遍（records.rs:78 的循环），所以「三格相等」这一点用例能钉的是读回，钉不了「某一格单独写错」；这一格不是 Q-F 要求的另一件事，不算缺。
- 判定：**一致**。给的时间取样点现在是：晚 600 秒（第 737 行起那条）、早 600 秒（757）、先建更晚 inode（777）、2^32 + 7（795，常量在第 625 行）、上一版加 7 秒（819）；进程内直发 Later（554）、Earlier（604）；续做 Later（872）、Earlier（889）。

### 1.3 三行变异改坏的是不是名字说的那一处

复跑（整个目标 22 条，红的是哪几条；`rerun-summary.txt` 的 `row-2670` 至 `row-2673`）：

| 行 | 名字说的 | 改的位置 | 红的 |
|---|---|---|---|
| 2670 qfm | 会话里时间只进不退 | `mounted_session.rs` 的 Unlink 一臂 | 2 条：777（先建更晚 inode）和 757（会话早于上一版） |
| 2671 qfu | 意图按 32 位存 | `transaction.rs` 意图记录 | 1 条：795 |
| 2672 qfr | 意图取整到分钟 | 同上 | 2 条：795（2^32 + 7 也不是整分钟）和 819 |
| 2673 n7-clamp | 意图取给的与上一版里较大的 | 同上 | 3 条：604（本批新加）、757、889 |

- qfu、qfr、n7-clamp 三行：改的就是名字说的那一处（意图记录的 `write_time_seconds` 一格，锚在 5974–5975 行，只命中意图不命中批），判定 **一致**。

### 1.4 qfm：单处写法与攻方原来那种是不是同一类错

**冲突**。两边整行并排：

| 一边 | 原文 |
|---|---|
| 攻方批 17 的 qfm（`research/prompts/m3-batch17-code-r1-opus-output.md:304`） | 「qfm：会话里时间只进不退（会话记下给过的最晚时间，unlink 与建 inode 的时间取 max(给的, 它)）」 |
| 本批 qfm 变异行（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/mutations.tsv:2670`） | 「会话那一层 unlink 给意图的时间取给的与现行那一版里各条 inode 记录最晚的写入时间里较大的那个」 |
| 新用例上方的判别力注释（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:775`） | 「会话里只有 unlink 那几条照旧绿」 |

- 机理不同：原版 qfm 要会话里多一个字段，记下**会话自己给过的**最晚时间；本批变异行是在 Unlink 一臂上读**上一版盘上所有 inode 记录**的最晚写入时间再取 max，不需要会话记忆。后者与 n7-clamp 同属「拿上一版的记录当下界」，只是范围从「这一条记录」放宽到「所有记录」。
- 复跑量出三个版本的红集（整个目标 22 条）：

| 版本 | 红的 |
|---|---|
| 本批变异行 2670 | 757 与 777，共 2 条 |
| 批 17 攻方原版（`patches/qfm-session-field-batch17.patch` 原样：`mounted_session.rs` 加字段与改 `change`，`history.rs` 三处构造补字段；另补本轮多出的第四处 `session_that_loses_power` 构造） | 只有 777，共 1 条 |
| 我写的单处会话记忆（`patches/qfm-thread-local-insert.rs.txt`：`publish_user_change` 开头用 thread_local 记最晚时间，Unlink 与 NewInodes 两臂各 max；零轮，只在我的副本上量过） | 只有 777，共 1 条 |

- 结论（正推）：①新用例 777 能杀攻方原来的 qfm，原版只有它红，覆盖没有漏；②变异行 2670 **不是**原来那种错：它同时被早于上一版的会话用例 757 杀掉（757 本批补丁只给搭建函数多加了一个 `NoOtherChange` 参数，断言没动），所以注释里「会话里只有 unlink 那几条照旧绿」对这一行不成立；③后果：变异表里没有任何一行只能被 777 杀，把 777 删掉或削弱，2670 照样红，表上看不出来。改法（零轮，我的副本上量过）：把 2670 换成「`publish_user_change` 开头用会话级记忆」的单处写法，或照批 17 补丁的形式写成带字段的补丁；只要它只被 777 杀，这条用例才有变异在背后。
- 推翻它的现象：若有一个会话记忆写法同时被 757 杀，则 ② 不成立。我量到的两个写法都只被 777 杀。

## 2　K1-N2：「早于上一版」一形（进程内直发）

- 拆法：原用例 554 改用带参数的搭建 574，断言与改之前同值（意图一版、中间一批，两块盘三格都等于给的值：第 559–569 行），新用例 604 用同一个搭建、给上一版减 600 秒（第 612 行 `vec![vec![[observed.given_time; 3]; 2]; 2],`）。判定：**一致**，拆开前后 Later 一形判的是同一件事。
- 它新增的判别力：n7-clamp（行 2673）下 604 红，晚于上一版那条 554 照旧绿（复跑：554 不在红集里）。**但 604 不是唯一杀 n7-clamp 的**：同一行下会话早于上一版的 757 和续做早于上一版的 889 也红，红集共 3 条。原因：会话与续做都经 `publish_unlink_intent`（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mounted_session.rs:252` 的 `} => publish_unlink_intent(`），变异锚在 transaction.rs 的意图记录里，三条路都过它。604 注释里「会话与续做两条早于上一版的用例不经直接调 `publish_unlink_intent` 这一路」说的是用例怎么调，不是变异能不能被它们杀。
- 结论：604 无害、不错，对「变异只改在 `publish_unlink_intent` 之外的直发路径」这类错多一层；但在今天的变异表上不构成额外覆盖（冗余，不是缺口）。推翻它的现象：找到一个只在直发路径出错、会话与续做路径不出错的实现。我没有找到，也没试着造。

## 3　K1-N3：「放开挪到暖机之前」的钉

### 3.1 条款到实现

- 条款：「卸载半途崩溃（系统配置已带新 F、带新 F 的根没落满每块盘）之后的下一次可写挂载同样只放开一截」（`.claude/kb/decisions/16-发布语义.md:41`），同一行：「其余扣到这次挂载的暖机在每块盘上都落了根（用户 2026-10-04 定，零轮）」（`.claude/kb/decisions/16-发布语义.md:41`）。N3 要钉的「放开挪到暖机之前」**不在批 17 判决的 K1-N4 行里**：那一行写的是「没钉用户看得到的结局；`admission.rs` 第 741 行给了一条可走的用户可见线，没造」（`research/prompts/m3-batch17-code-r1-main-verification.md:35`）；这一形的出处是批 17 正文的已排除项「已进第 18 批清单」（`research/prompts/_m3-batch17-code-r1-body.md:27`）与那一轮 Sonnet 腿报告「（背景材料第一节已排除，进第 18 批）。」（`research/prompts/m3-batch17-code-r1-sonnet-output.md:108`）。
- 出处写错（注释层）：新用例的注释写「放开挪到暖机之前那一刀」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:615`），括号里把「钉不住放开挪到暖机之前」归给判决第二节 K1-N4 行，判决那一行里没有这句（grep `挪到暖机之前` 在批 17 判决里 0 命中，在 Sonnet 腿报告里命中第 108 行）。不影响用例判什么，只是注释的出处该改成批 17 正文第 27 行；背景材料这一格的「压着的条款」栏沿用了同一个说法。
- 实现：放开在暖机循环之后：「// 扣住的那一截不交给任何会话。」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4887`）接「allocator.release_reclaim_holds();」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4888`），循环起点是「let mut current = row_publish.clone();」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4863`）。判定：**一致**。

### 3.2 断言判的是什么，变异下红在哪

- 用例（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:622` 起）：窄盘 176 槽，卸载之前现行那一版换下的槽 `released_before_the_unmount`，卸载只落一个写就崩，再可写挂载，断言写行那次与每次暖机写下的单元的槽与这些槽的交集为空：「.filter(|slot| released_before_the_unmount.contains(slot))」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:656`）。这判的是**落点**（盘面层），不是用户读回；理由是只读挂载丢盘的几条在写行那次就崩，暖机没跑到（用例注释自己写了）。判定：**一致**，范围比「用户看得到」窄，窄在没有读回。
- 复跑（行 2674，整个目标 6 条）：红 2 条：N3 这条与 N4 那条（`row-2674`）。**两条都是红在核心里的 panic，不是红在用例自己的断言上**：`mount.rs:4906` 的预演对真发那一判，信息是「取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/mount.rs:4907`，该行是 panic 信息的头一行，`assert_eq!(&placements_taken, planned, …)` 在 4905–4906 行）。
- 把那一判在副本里关掉（我的做法：把 `if let (Some(planned), false)` 改成多一个恒假的条件，只改副本，驱动里 `variant-jia-without-the-core-assert`）再跑：红 1 条，是 N3 自己的断言，`left: [1410]`、`right: []`，与用例注释写的「换下的槽 1410」同值；N4 那条在这个变体下绿。
- 结论：N3 对甲有两层，核心里那一判先红，用例自己的断言是第二层且复跑坐实；N4 对甲的红完全靠核心里那一判。若将来有人把预演对真发那一判去掉（或预演也跟着挪），N3 自己的断言接得住，N4 接不住——这不是缺口（N4 的活是 3.1 的另一件），但要知道 N4 那条在甲下不是独立的证据。
- 推翻它的现象：关掉核心那一判之后甲下 N3 仍绿；我量到的是红（`[1410]`）。

## 4　K1-N4：用户看得到的结局

- 判决要求：「N4 那一条补用户看得到的结局（挂载交回之后第一次写要不要多推一串空发布、F_生效 被多抬多少），造得出就补用例，造不出写明。」（`research/prompts/m3-batch17-code-r1-main-verification.md:54`）。今天造出来了：窄盘 176 槽（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:683`）、文件长到 36 个单元（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:666`）再缩成 1 个、卸载崩在第一个写之后、再挂、第一次写把文件写回 36 个单元。
- 判的是用户看得到的结局：第一次写成功还是被拒、推没推抬 F、F 变没变。写成功是靠 `unwrap_or_else` 里的 panic（第 717 行：「panic!("挂载交回之后第一次写把文件顺序写回 36 个单元：{refusal:?}")」，`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:717`），之后断言「(0, 1, floor_of_the_mount),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:725`）。
- 复跑（行 2675，删掉暖机之后的放开）：红 2 条，N4 与 4 GiB 那条老用例（`row-2675`）。N4 红在第 717 行的 panic，原样：`NoSpaceAfterRaisingTheFloor { last_refusal: PlacementRefused { unit: Data(DataUnitIndexInFile(31)), refusal: NoFreeSlotOnAnyDevice }, floor_raises: [], … stop: FloorAlreadyAtTheCeiling { floor: CheckpointTxg(5), ceiling: CheckpointTxg(5) }, versions_published: [] }`——与背景材料 N4 一行写的 `NoSpaceAfterRaisingTheFloor`（`FloorAlreadyAtTheCeiling`）一致。
- 没有任何变异红在第 725 行那句断言上（`floor_raises` 为 0、发成一版、F 不动）：红的路径在到它之前就 panic 了。它钉的是「别的实现也许写成功但先推一串抬 F」，变异表里没有一行背着它；不是错，是没有证据说明它会红。
- 范围：**这个结局只在准入关着的配置里可达**。我把 N4 用例里的 `SpaceAdmission::SkippedByTheTestOnlySwitch`（第 700 行）换成 `JudgedByTheFormula` 在**基线**上跑（驱动 `probe-n4-admission-judged-baseline`），第一次写在准入处就被拒：`SpaceAdmissionRefused(… available: AvailableBytesOnOneDevice(-10321961), demand: BytesOnOneDevice(1359872) …)`，两块盘都是负数，与放开没关系。也就是说这块 176 槽的窄盘按式子判，可用本来就是负的，产品配置下 N4 要的那一格走不到；用例借准入开关才把结局推到落点那一判上。这是 `SkippedByTheTestOnlySwitch` 的本意（只供测试，进分支），但「用户看得到」要加上「准入关着」这个前提。
- 误红与漏红：用户决定的几步（槽数 176、覆盖写次数、36、崩在第 1 个写之后）在用例里是写死的常量；用例注释写「单元区 176 槽上文件长到 32–40 个单元的每一点都是这个结局」，**我没有复核这个扫描**（归攻方腿的 K2），所以对 35、37 这类相邻点我不判。写死的 36 落在该区间内，今天的基线绿、变异红，两头我只量了这一点。
- 判定：**一致**，附一条范围说明（准入关着才可达）与一条空缺（725 行断言无变异背着）。

## 5　K1-N5：两份私有搭建换成共用模块

- 前后对照（补丁原样见 `research/prompts/_m3-batch19-code-r1-diff.md`，我用 `git show HEAD:…` 取改之前的文件与冻结副本 `diff`）：`unlink_single_file_publishes.rs` 变动 70 行（补丁 13 增 57 删），`container_trees_grow_a_level_when_the_root_is_full.rs` 变动 40 行；两份的 `#[test]` 个数改前改后一样（9 对 9、4 对 4）；测试函数体不在变动之列，变动只在 `mod`/`use` 头与被删掉的私有函数。
- 搭建函数逐个对：三个单元的内容与写法、批大小 1、下一步、新池建 inode 的函数体，补丁里被删的与共用模块里的逐行相同（共用模块：「pub fn pool_with_a_three_unit_file(tag: &str) -> BuiltPool {」在 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:96`，「pub fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {」在同文件第 163 行，建 inode 的写入时间常量「pub const WRITE_TIME_SECONDS_OF_THE_NEW_INODES: u64 = 1_788_000_100;」在同文件第 44 行，值与被删的那份相同）。
- 唯一没换的一个：`publish_the_intent` 在 `unlink_single_file_publishes.rs` 里保留私有版（「fn publish_the_intent(pool: &mut BuiltPool) -> Result<TransactionOutput, PublishError> {」，`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_single_file_publishes.rs:33`），因为那份交回 `Result`、有「已删完」那一格要看被拒；共用版被拒就 panic（`common_deletion_in_progress/mod.rs:107`）。该文件里 `publish_the_intent(&mut pool).expect("意图")` 的调用点（第 92、157、207、263 行）用的是本地这份，不是共用的。
- 共用模块头写明用它的文件要同时声明两个 `mod`：「用这一份的测试文件同时声明 `mod common;` 与 `mod common_pointer_head_corruption;`」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs:5`），两份都照办了。
- 复跑：`unlink_single_file_publishes` 9 条过、`container_trees_grow_a_level_when_the_root_is_full` 4 条过。判定：**一致**（行为不变）。推翻它的现象：改前改后任一条测试的函数体不同，或某个搭建的写入时间/批大小对不上；我对的是静态文本加两个目标全绿，没有做「改前的二进制与改后的二进制逐字节对盘面」那一层。

## 6　K1-N6：C2 的 unlink 斜率检查（部分）

- 条款：C2 的判法是「斜率不为 0 判红」（`.claude/kb/checks-owed.md:18`），同一行写明对象补上 unlink 之后才有东西可量：「对象补上 unlink 之后这一道有东西可量」（`.claude/kb/checks-owed.md:18`）。用例头的题目是「unlink 单次发布的写入量不随文件长」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:1`），测试函数名收在「and_its_largest_publish_does_not_grow_with_the_file(」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:133`）；而下面 ②③ 自己放行并钉住的恰是「最大那次发布随文件长而长」（量数见下）。
- 用例的三层断言：①每次发布换下的数据单元不超过一批：「<= EXTENT_TREE_LOWER_LEAF_DATA_UNITS),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:152`）；②每份文件里写调用最多的那次发布不超过一个单元那份的最大值加 6：「const WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE: u64 = 6;」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:29`）；③四份文件每次发布的写调用钉绝对值，最后一份：「vec![33, 37, 35, 33],」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:178`）。写的字节数收了、没断言：「written_bytes: total.written_bytes,」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:46`）只进了打印。
- **「每次发布换下的单元 ≤ 144」与 C2 的「斜率为 0」不是同一件事**：①是分批规则（D8 已定项 16 第 2 条「每批释放尾部 K 个单元」，`.claude/kb/decisions/08-核心索引结构.md:465`）的上界，是「写入量不随规模长」的一个必要条件（单位是换下的单元，不是写入量）；C2 要的斜率是写入量对规模。我跑 `--nocapture` 取用例自己打印的四行（原样，一行一份文件）：

```
1 个数据单元的文件，每一次发布（意图、各批）：[WritesOfOnePublish { write_calls: 31, written_bytes: 508416, data_units_released: 0 }, WritesOfOnePublish { write_calls: 25, written_bytes: 344576, data_units_released: 1 }]
144 个数据单元的文件，每一次发布（意图、各批）：[WritesOfOnePublish { write_calls: 31, written_bytes: 508416, data_units_released: 0 }, WritesOfOnePublish { write_calls: 29, written_bytes: 410112, data_units_released: 144 }]
145 个数据单元的文件，每一次发布（意图、各批）：[WritesOfOnePublish { write_calls: 31, written_bytes: 508416, data_units_released: 0 }, WritesOfOnePublish { write_calls: 31, written_bytes: 475648, data_units_released: 144 }, WritesOfOnePublish { write_calls: 29, written_bytes: 410112, data_units_released: 1 }]
432 个数据单元的文件，每一次发布（意图、各批）：[WritesOfOnePublish { write_calls: 33, written_bytes: 541184, data_units_released: 0 }, WritesOfOnePublish { write_calls: 37, written_bytes: 573952, data_units_released: 144 }, WritesOfOnePublish { write_calls: 35, written_bytes: 541184, data_units_released: 144 }, WritesOfOnePublish { write_calls: 33, written_bytes: 475648, data_units_released: 144 }]
```

- 读这四行：每份文件里写调用最多的那次发布是 31、31、31、37，字节最大是 508416、508416、508416、573952——写入量随文件长度是**长**的（意图那次也从 31 变 33），只是长得慢、有台阶。①对这四份都成立，不说明斜率为 0；②的上界 31 + 6 = 37，**恰好等于** 432 个单元那份的最大值，没有余量，是拟合到这一点的数；③钉死四份的当前值。所以这条用例钉的是「每批有上界 + 今天这四点的绝对值」，不是斜率。背景材料 N6 一行写「C2 的 unlink 斜率检查（部分）」与「斜率那一项四点上分不出，留欠账」，与我量到的一致；但用例头与函数名没带「部分」二字，读用例的人看到的是「不随文件长」。
- 注释与数不合（并排）：用例注释写「不随文件的单元数长：432 个单元的文件与 145 个单元的文件，一批走的路径一样长、罩的叶一样多。」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs:28`）；同一文件第 178 行钉的数是 145 个单元那份最大 31、432 个单元那份最大 37（上面打印）。注释这一句与自己钉的数矛盾（差 6 恰是常量 6）；不影响断言，影响读者对常量 6 的理解。
- 变异（行 2676，批大小改成整个文件一批）：红 1 条，红在 ①，第一处是 145 个单元那份「换下 145 个」；与行里「① ③ 红、② 判不出」的说法一致（② 判不出这一点我没有单独验，只看到红在 ①）。
- 判定：**部分一致**。①②③ 都做了；与 C2「斜率为 0」不是同一件事，用例也没这么声称。推翻它的现象：若再加一份大于 432 的文件、写调用最大值继续长过 37，②③ 红——这是今天四点之外的范围，我没跑。

## 7　K1-N7：C479 的只供测试开关与用例一

### 7.1 条款到实现

- 欠账：C479 要的是「只供测试的开关把水位压回一个已发过的号，再建一个 inode，checker 必须判红」（`.claude/kb/checks-owed.md:384`），并要「拦不住就要另立一条不变量。判别力自证：把开关关掉，检查必须由红转绿」（`.claude/kb/checks-owed.md:384`）。I-9.6 判的两半：「水位 > 树内最大 key；墓碑那一半（2026-10-05 起）水位 >」（`.claude/kb/invariants.md:293`）。
- 类型：压回的号是带私有字段的新类型，`new` 拒小于第一个 inode 号（含 0）：「pub struct InodeNumberWatermarkPressedBackForTests(u64);」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:215`）、「(next_inode_number >= FIRST_INODE_NUMBER).then_some(Self(next_inode_number))」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:221`）。写入口默认是产品臂：「InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:321`）。
- **开关的类型没有让产品路径走不到它**：设置函数是 `pub`、没有 `cfg` 把守：「pub fn set_inode_number_watermark_source(&mut self, source: InodeNumberWatermarkSource) {」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:392`），与它的兄弟同一写法（「pub fn set_container_tree_capacities(&mut self, capacities: ContainerTreeCapacities) {」，`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:379`）。让产品路径碰不到它的是调用点：`grep -rn 'set_inode_number_watermark_source' crates --include=*.rs`（冻结副本里）命中 3 行：定义（transaction.rs:392）、新用例的 `writer.set_inode_number_watermark_source(source);`（用例文件第 33 行）、用例文件头的一句注释（第 2 行）。挂载、续做、管理员回退（`mount.rs` 第 6526 行 `version_to_publish_the_rollback_after` 构造上一版、第 6575 行 `publish_version_appending_a_lineage_record` 再进 `publish_version_of_trees`，同一个发布路径）都用 `PoolWriter::new` 的默认臂，零处调用设置函数。判定：**一致**（靠命名与零调用点，不靠类型；与兄弟开关一致）。
- 两处读点读同一份：起号：「.watermark_this_publish_starts_from(first_new_inode_number);」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:5889`），记账：「.watermark_this_publish_starts_from(previous_version.inode_number_watermark()),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:8010`）。同一个开关：两处都读 `pool.inode_number_watermark_source()`，起号的 `pool` 是 `publish_new_inodes` 的参数，记账的 `pool` 是 `publish_admitted` 的 `&PoolWriter`，中间经 `publish_version`、`publish_version_with_unmount_marker`、`publish_version_of_trees` 原样传下去；同一份上一版：起号读 `previous`，记账读的 `previous_version` 来自 `Some(previous),`（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:5927`）一路传到 `publish_admitted(`（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:6521`）。
- 「两处读点在任何落盘之前」：成立。起号那处在 `publish_new_inodes` 开头，记账那处在 `publish_admitted` 里，而它拿的是不可变的 `pool: &PoolWriter<'_, Device>`（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:7780`），返回的是要写的东西，落盘在调用者那一头的 `persist_the_publish_or_freeze_it(`（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs:6359`）。注意：断言（压的号不小于上一版水位）在记账那处触发时，分配器的内存状态已被这次发布的释放与取落点动过（失败路径只在 `Err` 时换回，panic 不换）；盘上没动，进程本来就 panic，不构成问题。
- 一条没写的行为：开关装上之后，**这个写入口发的每一次有上一版的发布**（不只建 inode）记账里都写压回去的号加这次新建数（读点在 `/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-core/src/transaction.rs` 第 8008–8010 行），设置函数的文档注释自己写明了这一点；今天用例只用一次，没碰到。

### 7.2 用例一两边各断言了什么，六行变异红在哪

用例（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:73` 起）：新池新建文件后建 inode 2、3、4（水位 5），再建一个 inode，开关把水位压回 3。

| 边 | 断言（行） | 内容 |
|---|---|---|
| 压回去 | 37 | 写入口上装的臂的名字就是装进去的那一个：「"运行时看得出走的是哪一臂"」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:37`） |
| 压回去 | 86 | 记账里的水位写成 4：「"从压回去的 3 起发一个，水位写成 4"」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:86`） |
| 压回去 | 91 | 3 号的记录换成这次发布建的新对象：「"3 号被重发：树里那条记录换成这次发布建的新对象"」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:91`） |
| 压回去 | 95 | I-9.6 判红，说明含水位与树内最大 key：「detail.contains("水位 4 不大于 inode 树内最大 key 4"),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:95`） |
| 压回去 | 106 | 只红这一条：「vec!["I-9.6"],」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:106`） |
| 关开关（同一段流） | 118 | 发 5 号，水位 6：「"开关关着：发 5 号，水位 6"」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:118`） |
| 关开关 | 122 | I-9.6 成立：「InvariantVerdict::Holds,」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:122`） |
| 关开关 | 127 | 零违例：「Vec::<&str>::new(),」（`/tmp/claude-1000/m3-batch19-code-r1/tree/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs:127`） |

对 C479 要的两件：开关压回去、checker 判红（95/106）与关开关由红转绿（122/127）都有；判别力自证是同一个测试里同一段流两边比，满足「把开关关掉，检查必须由红转绿」。**第二件没做**：C479 还要「压回去但树里那条记录已被换掉」的镜像，看 I-9.6 拦不拦；用例头第 5–6 行明说不钉、交主 agent，背景材料也把用例二划出这一轮。判定：**部分一致**（第一件一致，第二件按划分不归这一轮）。

复跑（驱动 `rows:2677-2682`，原样结果行见模型目录 `rerun-summary.txt` 的 `row-2677` 至 `row-2682`）：

| 行 | 变异 | 红的 | 红在哪 |
|---|---|---|---|
| 2677 | 设置函数不生效 | 用例一 | 第 37 行：`left: "inode_number_watermark=from_the_previous_versions_accounting"`、`right: "inode_number_watermark=pressed_back_for_tests"` |
| 2678 | 起号不读开关 | 用例一 | 第 91 行：`left: Some(4)`、`right: Some(5)`；此时水位仍写成 4，第 86 行过 |
| 2679 | 记账不读开关 | 用例一 | 第 86 行：`left: 6`、`right: 4` |
| 2680 | 压回臂交回上一版水位 | 用例一 | 第 86 行：`left: 6`、`right: 4` |
| 2681 | `new` 收 0 | core 单测 `the_pressed_back_inode_number_watermark_refuses_the_invalid_inode_number_zero` | 单测自己的断言 |
| 2682 | 不小于上一版水位也不拦 | core 单测 `a_pressed_back_inode_number_watermark_not_below_the_previous_watermark_is_a_misused_switch` | `should panic` 没 panic |

- **六行里没有一行红在 I-9.6 那几句（95/106/127）上**：2677、2678、2679、2680 都红在前面的直接断言上。理由不是用例弱，是这几刀改的是开关，不是 checker；但也意味着「I-9.6 只红这一条、关开关零违例」这几句的判别力，今天变异表里没有行背着（它们的判别对象是 checker，另在 `checker_known_bad_images.rs` 那一族里）。
- **探针（我加的，在模型目录 `probe/zz_probe_n7_body.rs`，驱动 `probes`）**：行 2679 的变异下跑同一段流，打印：

```
  [probe-n7-baseline] PROBE watermark=4 birth3=Some(5) txg=5 verdict_I96=Violated("记账里的 inode 号水位 4 不大于 inode 树内最大 key 4") violated=["I-9.6"]
  [probe-n7-under-row-2679] PROBE watermark=6 birth3=Some(5) txg=5 verdict_I96=Holds violated=[]
```

  第二行是「3 号已经被重发（3 号记录的对象出生代 = 这次发布的 txg 5）、记账里的水位仍诚实地写 6」：池级 checker 的 I-9.6 判成立、违例清单为空。这是「号被重发而 I-9.6 看不出」的一个实例（只压了起号、记账里的水位没压）。C479 第二件问的是另一形（水位压回去、压回的正是发过的最大号，记录被换掉），两形不同，但同属 I-9.6 只比水位与树内最大 key、墓碑最大号、看不到重发；背景材料划出这一轮的用例二是同一类。我不判要不要另立不变量（划给设计轮），只把量到的写在这里。用例一的 86 与 91 两句直接断言，正是 2678 与 2679 这两种「只压一处」的变异的唯一抓手。
- 判定：**一致**（对开关的六个变异都有点名的红；对 checker 的判别由同一段流两边比给出，未被变异单独背着）。推翻它的现象：找到一种「只压起号、不压记账」或反之的实现而用例一仍绿——2678、2679 就是这两种，量到的都红。

## 8　K3：8 份被判文件各一格

| # | 文件（冻结副本 `crates/` 下） | 判到的 | 判定 | 一句话 |
|---|---|---|---|---|
| 1 | `mutations.tsv` | 新增 13 行（第 2670–2682 行）：原文各命中恰好 1 次；全表 2677 行（扣 1 个冻结副本里不存在的 litmus 文件）命中非 1 的 0 行；13 行各红在点名的测试上（见 1.3、3.2、4、6、7.2 各表） | **一致**，附一条：行 2670（qfm）不是名字说的那一种错 | 它同时被 757 杀，注释「会话里只有 unlink 那几条照旧绿」对这行不成立；没有行只被 777 杀 |
| 2 | `singlefs-core/src/transaction.rs` | 开关的新类型与枚举（215–290 行一带）、写入口字段与默认臂（301、321）、设置与读取函数（392–400）、两处读点（5885–5889、8006–8010）、两条核心单测（9411、9427）；意图与批的时间字段（5975、6038）本批没动，作 N1 的依据读过 | **一致** | 产品路径零调用点；两处读点同开关同上一版同在落盘之前；设置函数 `pub` 无 `cfg` 把守，靠命名与零调用点 |
| 3 | `singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` | N5：删私有的 `pool_with_new_inodes` 与两个常量，改用共用模块；`#[test]` 4 对 4，4 条过 | **一致** | 搭建函数体与共用模块逐行相同 |
| 4 | `singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs` | N7 用例一：压回 3、两边 8 句断言；六行变异里它被四行点名 | **部分一致** | 第一件（红与由红转绿）一致；第二件（压回去但记录已被换掉）按划分不归这一轮；探针量到一个同类实例（只压起号）I-9.6 判成立 |
| 5 | `singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | N3、N4 两条新用例、`slots_released_by_the_current_version` 与一处文档改写；6 条过；行 2674、2675 各红 2 条 | **一致**，附三条说明 | ①N3 注释把出处归给判决 K1-N4 行，判决里没有这句（批 17 正文第 27 行才有）；②甲下两条都先红在核心 `mount.rs:4906` 的预演对真发那一判，N3 自己的断言要关掉那一判才红；③N4 的结局只在准入关着时可达，725 行断言无变异背着 |
| 6 | `singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | N1、N2：三条判别形加早于上一版一形；22 条过；行 2670–2673 复跑 | **一致**，附二条 | ①qfm 行与原版 qfm 不同类（同 1）；②N2 的 604 在今天变异表上冗余（n7-clamp 红 3 条） |
| 7 | `singlefs-harness/tests/unlink_single_file_publishes.rs` | N5：删私有的三单元搭建、批大小、下一步、两个常量，保留返回 `Result` 的 `publish_the_intent`；`#[test]` 9 对 9，9 条过 | **一致** | 保留的那一个理由成立（有被拒那一格要看） |
| 8 | `singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs` | N6：四点三层断言；1 条过；行 2676 红在 ① | **部分一致** | 钉的是「每批有上界 + 四点绝对值」，不是斜率；写调用最大值 31/31/31/37 随文件长而长；②的常量 6 恰好等于拟合点的差；注释第 28 行与自己钉的数矛盾；`written_bytes` 收了没断言 |

## 9　各格的推翻条件（汇总，细处见各节）

- N1：任何一条写路径的意图或批里改了时间值而盘上 64/72/80 三格不等于给的值；或找到一个会话记忆写法同时被 757 杀（则 qfm 行与原版同类的说法站得住）。
- N2：找到一个只在直发路径出错、会话与续做路径不出错的实现（则 604 不冗余）。
- N3：关掉核心里预演对真发那一判后，甲下 N3 自己的断言仍绿（我量到红，`[1410]`）。
- N4：删掉暖机之后那一句放开时，第一次写仍写成功（我量到被拒）；或准入照式子判时这块窄盘上基线第一次写能做成（我量到在准入处被拒）。
- N5：改前改后任一条测试函数体不同，或搭建的写入时间、批大小对不上。
- N6：加一份大于 432 的文件，写调用最大值继续长过 37（则 ②③ 红，说明今天的钉只在四点内成立）；或找到一个每批换下 ≤ 144 个、写入量却随文件长的实现而四点绝对值恰好不变。
- N7：找到「只压起号、不压记账」或反之的实现而用例一仍绿（2678、2679 就是这两种，量到都红）；或产品路径里有一处调用设置函数（grep 命中 3 行，无产品调用点）。

## 10　判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-N1 | 一致，附一条 | 条款、代码、盘上三格一致；qfu、qfr、n7-clamp 三行改的是名字说的那一处；qfm 行是「读上一版所有记录取 max」，不是攻方原来的「会话记下给过的最晚时间」，且同时被 757 杀 |
| K1-N2 | 一致（冗余） | 拆法保住了原断言；604 在 n7-clamp 下红，但 757、889 也红，今天变异表上不是唯一抓手 |
| K1-N3 | 一致，附出处写错 | 断言判落点不判读回；甲下先红在核心 4906，关掉那一判才红在自己的断言（`[1410]`）；注释把出处归给判决 K1-N4 行，判决里没有 |
| K1-N4 | 一致，附范围与空缺 | 红在第 717 行的拒绝（`FloorAlreadyAtTheCeiling`），没红在 725 行；结局只在准入关着时可达；725 行断言无变异背着 |
| K1-N5 | 一致 | 搭建函数体逐行相同，测试个数与函数体不变，两个目标全绿 |
| K1-N6 | 部分一致 | 「每次发布换下 ≤ 144」不是「斜率为 0」；四点上写调用最大值 31、31、31、37，字节也在长；②的 6 恰是拟合差；注释第 28 行与自己钉的数矛盾 |
| K1-N7 | 一致（C479 第二件不归本轮） | 开关类型靠命名与零调用点，不靠类型；两处读点同开关同上一版同在落盘之前；六行变异都红；2678、2679 红在直接断言上；探针量到「只压起号」时 I-9.6 判成立 |
| K3-1 `mutations.tsv` | 一致，附一条 | 13 行原文各命中 1、各红在点名的测试上；2670 不是名字说的那一种 |
| K3-2 `transaction.rs` | 一致 | 开关、两处读点、两条核心单测都读过、复跑过 |
| K3-3 `container_trees_grow_a_level_when_the_root_is_full.rs` | 一致 | N5，4 条过 |
| K3-4 `inode_number_watermark_pressed_back_reissues_a_number.rs` | 部分一致 | 第一件一致，第二件按划分不归本轮 |
| K3-5 `the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs` | 一致，附三条 | N3 出处、甲下红在核心、N4 范围与空缺 |
| K3-6 `unlink_mount_continuation_and_the_emptied_version.rs` | 一致，附二条 | qfm 行不同类；604 冗余 |
| K3-7 `unlink_single_file_publishes.rs` | 一致 | N5，9 条过 |
| K3-8 `unlink_writes_per_transaction_do_not_grow_with_the_file.rs` | 部分一致 | 钉的是上界加四点绝对值，不是斜率 |

我提的改法（都是零轮，只在我的副本上量过，没被攻过）：①把行 2670 换成只被 777 杀的会话记忆写法（`patches/qfm-thread-local-insert.rs.txt`，`publish_user_change` 开头单处锚）；②N3 用例注释的出处改成批 17 正文第 27 行；③N6 注释第 28 行改成与钉的数一致；④N4 若要把 725 行断言当证据，补一行只红在它上面的变异。

## 11　没做什么

- K2（哪一格会错）不归这条腿：没有把用户决定的几步（单元区槽数、覆盖写次数、文件长到多少再缩、卸载崩在哪个写之后）放开扫；N3/N4 的合法实现误红与错实现漏红、N7 压回的号与水位、墓碑、回退的组合、N1/N6 有没有错实现让用例照样过，都没系统攻。N4 用例注释写的「176 槽上 32–40 个单元每一点都是这个结局」我没有复核。
- 没复核主 agent 的观测：合入验证「163 个测试二进制」、fmt 与 clippy 退 0，复核不了（没跑，不归这条腿）；没跑 checker 档，没跑门禁。
- N6 用例注释说的「2304 个单元以上整个文件一批的写调用才超过分批」没复跑，也没有大于 432 的点。
- N5 只做了静态对照加两个目标全绿，没有「改前、改后二进制对盘面」逐字节那一层。
- 管理员回退、挂载、续做对开关的可达性我只做了 grep 与调用链阅读，没造运行时反例。
- 不替主 agent 采纳或出判决；没改 `crates/`，没改被判文件。

## 12　草稿、模型与清理

- 模型目录：`research/prompts/m3-batch19-code-r1-sonnet-model/`（`rerun.sh`、`drive.py`、`patches/`、`probe/`、`rerun-summary.txt`）。`rerun.sh` 在空目录里分阶段各跑通一遍，摘要即 `rerun-summary.txt`（28 行）。
- 删了（交回前，自己建的编译目录与仓副本）：`/tmp/claude-1000/m3-batch19-code-r1-sonnet/work`（18M）、`/tmp/claude-1000/m3-batch19-code-r1-sonnet/work2`（19M）、`/tmp/claude-1000/m3-batch19-code-r1-sonnet/target`（2.5G）、`/tmp/claude-1000/m3-batch19-code-r1-sonnet/rerun1`（2.4G）、`/tmp/claude-1000/m3-batch19-code-r1-sonnet/rerun2`（2.4G）。
- 没删：`/tmp/claude-1000/m3-batch19-code-r1-sonnet/` 下的日志、`mut.py`、`mut2.py`、`keep/`（第二遍复跑的输出与摘要）、`draft/`（本报告的分段草稿）：都是小文件、是本报告的复核材料，没入库是因为模型目录里的 `rerun.sh` 已能原样重出，这些只是第一遍的临时日志。
- 要主 agent 跑的命令：无。
