# m3-close-b19-code-r2 辩方腿报告（Sonnet，辩方）

日期：2026-10-30。轮名 m3-close-b19-code-r2。复核对象：第一轮判决 `research/prompts/m3-close-b19-code-r1-main-verification.md`；
被辩护的是它采了的 T4（C874）改法一、T2（C724）改法与判「相符」的 T1、T3、T5、T6，以及 C929（U5 设计判断）两个候选。
没读禁读清单里的任何一份（另一条腿的报告与模型、主 agent 核对稿）。

## 复跑与核对（报告开头）

- 开工快照：在仓根跑 `sha256sum -c research/prompts/m3-close-b19-code-r2-snapshot/crates-sha256.txt` 得 11 行 OK，`kb-sha256.txt` 得 5 行 OK（量的）。
  报告里的代码行号都在这个状态的文件里 `grep -nF` 现取。D18（块里携带什么信息）的 kb 文件不在 `kb-sha256.txt` 清单里，它的行号是此刻现读，
  `git diff HEAD` 对它为空（量的）。
- 模型目录 `research/prompts/m3-close-b19-code-r2-sonnet-model/`：`tests/` 五份 harness 探针（U1、U2、U4、U5、U6）、`patches/` 一份反向补丁
  （把第二轮修补还原成第一轮状态）与三份变异补丁、`out/` 每一步筛过的输出（PROBE、test、test result、panicked 行，去耗时与线程号、按行排序）、`tools/rerun.sh`。
- 复跑命令（整条经内存包装；`--frozen` 给开工快照那一刻的树：`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`）：
  `bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-b19-code-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>`
  每一步末尾打 `SAME name=…`（与 `out/` 里存的筛过的输出逐行相同）或 `DIFF name=…`。`--selftest` 也要经包装（`bash scripts/hosts/run-with-memory-cap.sh 4G bash …/tools/rerun.sh --selftest`），
  我跑了，末行「自检通过（查了 5 格）」。
- 本腿两棵树：`new` = 开工快照（第二轮修补之后，逐盘核在续做预演之前）；`old` = `new` 反向打上 `patches/r2-fix-as-given.diff`（第二轮修补之前）。
  `old` 的 `mount.rs` blob 是 `f7b756303fe1…`，与 `_m3-close-b19-code-r2-diff.md` 里 `index f7b7563..6e21c1f` 的前像相同；`new` 的是 `6e21c1fa38c6…`（`git hash-object`，量的）。
- 一次完整复跑（4G 内存包装、从开工快照的干净副本起、含两棵树各自的编译）：末尾 `SAME` 13 行、`DIFF` 0 行；`new` 树的 `mount.rs` 在三条变异还原之后 sha256 前 16 位仍是
  `ba5453bdb9b1c31d`（量的，复跑日志）。`out/` 里存的行是我先在副本上逐条跑出来的，复跑与它们逐行相同，所以不是复跑自己写的。
- 这些探针全部是副本上的探针，不进 kb；入库形态要判决后另立会红用例（下面每格写了哪一条探针可以直接升格）。

### 文件 sha256

- `tools/rerun.sh` df534f9382a6371c160304a2df93ac9cb24c5d492b7988bcea271471b9175703
- `tests/r2_probe_u1_check_before_rehearsal.rs` 11f95c8a332b943c9046baebf0a4416f82e41801fc6f33a41cbd940150fc1814
- `tests/r2_probe_u2_zeroed_slots_with_read_faults_in_the_rehearsal.rs` 7b5ebab71674da4a7e4594d4c054d06bf93c44c0964125f9a79155bf11c165a6
- `tests/r2_probe_u4_failed_resend_then_resend.rs` a8d3ea35f098faf9525c2d90d347494cfe4cbe996981f761ec7a01eff0f33a89
- `tests/r2_probe_u5_state_after_the_rollback_and_after_a_crash.rs` 7938983e1e7f542d70b83296b7a965900a54b39d6a8955e565c4f3b16e80cdbf
- `tests/r2_probe_u6_instance_table_sweep_threshold_by_mounting_many_times.rs` 400e55d5efdbd383878f1bc3c30632f24b3c2cac8deca7551ff92973af86e791
- `patches/mutation-6144-tie-broken-backwards.patch` 68ba0009b5ec89d7b01281d9d67de196d05a232b53307686ea81b7bcd2795f14
- `patches/mutation-6155-ceiling-rereads-skipped.patch` 4a02438ea7e0456bac1d88eef6b5bc87921ff984186b0042493c0653bc3dcb29
- `patches/mutation-6248-public-ceiling-entry-panics.patch` c6080b07ca06cccefd6f0edfd57b58d8a0fb6599cbb768738532f12f87d68df1
- `patches/r2-fix-as-given.diff` 7f447a86f67772cc8bc94b378130c78d50fbc94d3cc196f5f15ca60b89a164c0
- `out/new-core-lib-root-record.txt` 0da1f1a0a01191897409e6c5067b340fa68b69034438764ffbf196d07052a12c
- `out/new-pinned-and-new-tests.txt` 8fa7a2f0eeac7c6294925dfda21a7c856278cdd3df0f6b6dd6dd66902030ec0a
- `out/new-root-record-target.txt` fbc381420ac299231bfc169829d7f2e0398bbde23a54ae4ef7abaf97d3ca96ca
- `out/new-t1-mutation-6144.txt` a77dd3430e6daf6242256f38e62e29cfe87ee9d83c48649c6292140d3263f57e
- `out/new-t2-mutation-6155.txt` 3112528c193d721d2f27e65e007261a2bda6b62f7e133cd88582ed9dec0b428b
- `out/new-t2-mutation-6248.txt` d754733976ac55ad18db18047bf9ba2b822ab8818c6c61f320d7ee638feb4789
- `out/new-u1.txt` 270df18b5fb9b24e4bf0989238da065949a96fe6f62f0406c6c649c71d60c9b4
- `out/new-u2.txt` dbb03c28fa12c81ed2b1d1dfd157486160b0c2cbfa1d59ebcc0bf8fdea2658b9
- `out/new-u4.txt` 1ad05eeebd928e170a9426b3cec420bf9fd009f36f13b21ae6bc18c01005242f
- `out/new-u5.txt` 76567217e18a01c32fe54116c935b9667b22b0af48395ab182b4c8ad59e9ca3d
- `out/new-u6.txt` 54f53506c2748750d4441a5c91810ed3ba834398eca944a5e91122665e04682d
- `out/old-pinned-and-t4.txt` 9e09491dbb2b836f6ef61ce38871ed236fe3096b117c98862bcaba4442dbd18c
- `out/old-u1.txt` bc643e93759cf9b78141837fa1703d02f3965f88416d33289b014ddd1f84391e

## 一览表

| 格 | 判决条目 | 辩方判定 | 判决该不该改 |
|---|---|---|---|
| U1 | T4 改法一（逐盘核挪到预演之前） | **站得住**，且比不挪的替代方案够得着的多；一处理由要更正：「钉读序数的用例没跑到新次序那一支」不成立，真因是它们对这两段的次序不敏感（量的） | 采纳不变；补一条条款与用例钉「两样同时成立时报逐盘核那一拒」；更正那句理由 |
| U2 | 改法二（补写盘两槽读坏不记账）没加 | **站得住（不加）**：判决写的触发条件「预演是否仍因补写盘两槽读坏重走到上限」没有触发；剩一处拒因标签被盖（量的）与一类没造出的停法（推的） | 不加改法二；把「标签被盖」记一笔，要修另立零轮收严 |
| U3 | T2 断言的放处 | **站得住**；本进程准入路径只有一个入口，断言在那；`raise_rollback_floor` 一并变是同一个共享函数的必然后果，不是越出 T2 | 不改；补记 `mutations.tsv` 第 6155 行名字陈旧（行本身仍红，量的） |
| U4 | T3「相符」与新用例 | **站得住，用例够得着一个调用点**：三个调用点共用一个被守住的函数；重发先失败再成功的形补了（量的）；冻结期间再冻结造不出 | T3 维持相符；新用例留着；可再补一条用户改动那个点的用例（要有一段不被这次改动占去） |
| U5 | C929 回卷后过不过补写闸 | 推荐 **照今天不判**；回卷后下一次仍判三、四、五，会误拒「取号成功后崩溃」的正常盘面（量的） | 条款补一句，回卷用例补断言（补写盘表为空） |
| U6 | T1、T3、T5、T6「相符」靠读码 | T1、T3、T5 各用会红的探针或变异核了一次，**没被冤枉**；T6 里 `root_record.rs` 的 Verus 化只有读码加既有用例，没做新旧字节对比 | T1、T3、T5 维持；T6 的 Verus 化那一块列进「核不动」 |

## U1　逐盘核挪到预演之前

### 够不够得着（正推）

- 次序：`crates/singlefs-core/src/mount.rs:7113`「let devices_rewritten_by_the_acquisition =」在 `crates/singlefs-core/src/mount.rs:7133`
  「refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(」之前；`crates/singlefs-core/src/mount.rs:7127`
  「devices_rewritten_by_the_acquisition: devices_rewritten_by_the_acquisition.clone(),」把同一份表交给预演和真做。
- 打中的那一格：窄盘镜像盘 1 两槽写 0 加续做缺空间。`out/new-pinned-and-new-tests.txt` 里这一条新用例过，`out/old-pinned-and-t4.txt` 里（还原成旧次序的树）同一条
  `FAILED`，红在「挂载做成」（量的，复跑 `SAME`）。
- 比判决多一格：续做最后一批报 C841 的镜像加盘 1 两槽写 0。新次序报 C841，旧次序报
  `ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition`（探针 `u1_c841_image_with_both_slots_of_device_one_zeroed_returns_c841_when_the_check_comes_first`，
  `out/new-u1.txt` 与 `out/old-u1.txt` 逐行比，量的）。旧次序下 T4 的洞不只拒错了「该做成的挂载」，还把「该报 C841 的拒」报成了读坏。
- 再多一格：盘 1 整块写 0（补写闸第一件过不了）加 C841 镜像。新次序报逐盘核那一拒 `SystemConfigurationUnreadableAfterTheReadStage`，旧次序报预演读坏
  （探针 `u1_c841_image_with_device_one_blank_reports_the_check_not_c841_when_the_check_comes_first`，量的）；同一块盘在不带 C841 的窄盘镜像上，新树报同一个逐盘核那一拒
  （`out/new-u1.txt` 的 `plain_with_blank_device_one` 行），旧树报预演读坏。旧次序下这一拒因被预演吃掉，补写闸的拒因载荷走不到，正是 C874 要运行时看得见的那一格。

### 是不是同样打中所有替代方案

- 替代 1：只加改法二、不挪次序。预演仍按空表走，那一趟取号读盘 1 一份自证过的都没有、停在取号；若读坏账里没有盘 1 两槽的条目，
  `crates/singlefs-core/src/mount.rs:5708`「(None, None, _) => return Ok(()),」把预演判成「留给续做」，挂载能做成，但续做最后一批的 C841 不再在取号之前拒
  （预演根本没走到续做）。**推的**，没造：这条替代方案修得了「该做成的挂载被拒」，修不了「该在取号之前拒的 C841 盘面」。
- 替代 2：预演里把补写盘表硬塞成「全部两槽全废的盘」，不经逐盘核。会绕过补写闸，等于拿掉闸，不是同一档的收严。
- 所以挪次序是三者里唯一让预演、闸、拒因三处同时走到的。

### 逐盘核本身读盘（读坏、读错）会不会让预演看到的盘面与真做不同

- 逐盘核在 `crates/singlefs-core/src/mount.rs:7375`「} = with_reads_straight_to_the_disk(read_route, || {」里读，走 `StraightToTheDisk` 路：
  `crates/singlefs-core/src/mounted_session.rs:1015`「ReadRouteOfThisMount::StraightToTheDisk => return self.disk.read_at(offset, buffer),」
  既不取也不改这次挂载记下的那一份，读错也不进读错账。读坏也不进：
  `crates/singlefs-core/src/mounted_session.rs:1170`「ReadRouteOfThisMount::StraightToTheDisk => {}」（记读坏的函数在这一路什么都不做），记账那一句在
  `crates/singlefs-core/src/mounted_session.rs:1177`「self.reads_failing_self_verification.borrow_mut().push(」，只在共用路与重读路上到。
  两本账每一趟预演开头清空：`crates/singlefs-core/src/mount.rs:5674`「reads_failed_on_the_disk.borrow_mut().clear();」。
  所以逐盘核不论排在预演前后，都碰不到预演看到的字节与两本账（读代码得的，**推的**）。
- 盘是确定的内存盘时前后读到同一份；真盘上逐盘核与预演之间多隔了一趟预演的 CPU 时间，没有写（预演的写只进内存）。**没造**「逐盘核读时瞬时读错」的探针，
  这一问只有读代码的依据。

### 条款对先后有没有约束

- D18：`.claude/kb/decisions/18-块里携带什么信息.md:321`「这次挂载要发的写行与暖机在分配器的副本上预演取得到全部落点」，
  `.claude/kb/decisions/18-块里携带什么信息.md:322`「交进来的盘有一块不带就在取号之前、任何写之前拒可写」；
  D22：`.claude/kb/decisions/22-单元原子性怎么合成.md:185`「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽」
  与「补写失败照取号每块盘回卷一次」；C874：`.claude/kb/checks-owed.md:540`「C874 | 补写那一支进不了崩溃测试、运行时看不出」。
  这几条只定「都在取号之前、任何写之前」，没有一条给逐盘核与「续做预演」排先后。D18:321 说的预演是写行与暖机那一趟（`publishes_after_acquisition_rehearsed`，
  仍排在逐盘核之前，`crates/singlefs-core/src/mount.rs:7099` 一带），续做预演的来历是岔路单（`crates/singlefs-core/src/mount.rs:7129` 起的注释），不在 kb 条款里。
  先后没被条款约束，**也没被条款钉住**；两样同时成立时报哪一个，今天只有我这三条探针在钉。

### 钉读序数的用例：是不是「没跑到新次序那一支」

- 判决（`research/prompts/m3-close-b19-code-r1-main-verification.md:22`）写「钉读序数的用例按新次序重核」；本轮问的是它们一个没改是不是因为没跑到新次序。
  **不是**：每一次可写挂载都走新次序，这些用例都跑到了。它们没红是因为对这两段的次序不敏感：
  `crates/singlefs-harness/tests/common_reads_shared_by_the_rehearsal_and_the_real_run/mod.rs:74`「pub const READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS: u64 = 308;」
  是判号那一读，在 `crates/singlefs-core/src/mount.rs:7057`「let instance_to_acquire = next_instance_generation_to_acquire(&pool).map_err(」，排在逐盘核与续做预演两段之前；
  别的用例的窗口都由不带故障那一遍的录制现取（同一个 `mod.rs` 里的 `ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy`），
  逐盘核读的是 4096 字节的系统配置槽，窗口筛的是 16 KiB 的树表读，选不中它。
- 量的：四个钉读序数的目标（`writable_mount_acquisition_write_reads_the_system_configuration_slots_from_the_disk`、
  `writable_mount_rehearsal_drops_bytes_failing_self_verification_from_the_shared_reads`、`writable_mount_rehearsal_and_the_real_run_read_each_block_once`、
  `writable_mount_reread_after_a_bad_read_reaches_the_disk_past_the_reads_shared_with_the_rehearsal`）在新树与旧树里都过
  （`out/new-pinned-and-new-tests.txt`、`out/old-pinned-and-t4.txt`，复跑 `SAME`）。所以「不用改」的结论对，写进判决的理由要换成「对次序不敏感，两棵树都过」。
- 顺带：`crates/singlefs-core/src/mount.rs:7348`「/// 核在它之前、读的是紧接着要写的这批盘」的「紧接着」现在中间隔着一趟预演（只进内存、不写盘），读的盘面没变，注释字面不再准。小事。

### 推翻 U1 的现象

- 把逐盘核的读从 `StraightToTheDisk` 路改成共用路（读坏账会记进盘 1 两槽之外的东西），或让预演写盘：则「碰不到预演看到的字节」不成立。
- 出现一份条款要求两样同时成立时先报预演那一拒：则新次序要改回。
- 造出一个盘面，新次序下挂载被拒、旧次序下做成：则「新次序只改变了拒因、没改变结局」不成立（我的三条探针里没有，量的）。

## U2　改法二要不要

- 判决的触发条件：`research/prompts/m3-close-b19-code-r1-main-verification.md:22`「改完量预演是否仍因补写盘两槽读坏重走到上限」。修补员推「这一形预演只走一趟、读坏账没被看」。
- 机制（读代码）：预演走到续做的结局直接返回，不看两本账：`crates/singlefs-core/src/mount.rs:5686`「ContinuationRehearsedBeforeAcquisition::WalkedToItsEnd => return Ok(()),」；
  两本账空时留给续做：`crates/singlefs-core/src/mount.rs:5708`「(None, None, _) => return Ok(()),」；到上限且只有读坏时交最先那一条：
  `crates/singlefs-core/src/mount.rs:5714`「(None, Some(read_failing_self_verification), true) => {」。补写盘两槽读坏进账的来源：预演里那一趟取号读盘 1 两槽，
  读成而自证不过的槽报读坏（`recovery::system_configuration_slots_of_one_device_read_once`）。所以只要预演走到底，账里有盘 1 两槽也不被看。
- 量的（`out/new-u2.txt`）：
  - 盘 1 两槽写 0、不带故障：`MountMade devices_rewritten=[DeviceIdentity(1)]`。
  - 加上「只进内存那一趟头一次读的树表两份各读回坏字节一次」：仍 `MountMade devices_rewritten=[DeviceIdentity(1)]`（重走一趟走到底，账里的盘 1 两槽没挡）。
  - 加上「同一份树表每一次读都读回坏字节」：拒，交的读坏是 `ReadBackFailingSelfVerification device=DeviceIdentity(1) offset=DeviceOffsetInBytes(0) length=4096`，
    即盘 1 槽 0；同一故障在没改坏的镜像上交的是 `device=DeviceIdentity(0) offset=DeviceOffsetInBytes(25067520) length=16384`，即真出故障的那份树表。
    拒的结局没变，拒因标签被盘 1 两槽的读坏盖住了（这是改法二唯一量得到的好处）。
- 预演停在别的非 C841 错上、又有补写盘：要的条件三个同时成立：预演要进只进内存那一趟（缺空间，`crates/singlefs-core/src/mount.rs:5761`「Err(refusal) if refusal_is_short_of_space(&refusal) => {」）；
  那一趟停在 `crates/singlefs-core/src/mount.rs:5788` 那张表里判 `false` 的错（或 `StoppedBeforeTheEndOfTheContinuation`）；那一趟没有读错读坏。
  **没造出来**：这张表里的成员（txg 到顶、树 ID 水位、删除队列错……）窄盘镜像上没有现成的造法，读错读坏造成的停是账本来就管的那一类。
  造出来的话，今天的行为是：账里有盘 1 两槽的读坏 → 重走到上限 → 在取号之前拒、盘不变、拒因写成读坏；改法二则留给续做。方向是「更早拒、标签不准」，
  不是「该做成的拒了」，除非那一停是预演自己的假象；`crates/singlefs-core/src/mount.rs:5636`起的文档列的假象类（当场重读那几读、取号写之前那几次读、取号写算世代号那一读）都是读类，
  不是非读类的停。**推的**。
- 判定：不加改法二。要修标签就另立零轮收严：交拒因时丢掉落在补写盘表里那几块盘的两个系统配置槽范围的条目（取号会覆写它们）。

### 推翻 U2 的现象

- 造出一个补写盘加非读类停法的盘面，旧（修补前）次序挂载做成、新次序被拒，且真做走得通：则改法二要加。
- 造出补写盘两槽读坏条目让一次本来走得到底的预演去重走：与 `WalkedToItsEnd => return Ok(())` 矛盾，会推翻上面的机制。

## U3　T2 断言的放处

- 判决要求（`research/prompts/m3-close-b19-code-r1-main-verification.md:20`）「改回：公开入口交回错误，`expect` 只留在本进程准入路径那一处」。
- 本进程准入路径只有一个入口：算上限的函数 `rollback_floor_ceiling_of_this_process`（`crates/singlefs-core/src/mount.rs:4405`）有两个直接调用方，
  `raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk`（`crates/singlefs-core/src/mount.rs:4507`「let ceiling = rollback_floor_ceiling_of_this_process(」，
  公开入口和推抬 F 共用）、`raise_the_floor_through` 的准入那一支（`crates/singlefs-core/src/mount.rs:4902`「let ceiling = rollback_floor_ceiling_of_this_process(」，只被只供测试的
  `raise_rollback_floor` 与卸载走；卸载那一支不判上限）。本进程里推抬 F 的三处都经 `push_one_floor_raise_within_the_admission_budget`：会话
  `crates/singlefs-core/src/mounted_session.rs:4533`「match push_one_floor_raise_within_the_admission_budget(」、可写挂载续做
  `crates/singlefs-core/src/mount.rs:7605`「match push_one_floor_raise_within_the_admission_budget(」、写行之后
  `crates/singlefs-core/src/mount.rs:8145`「match push_one_floor_raise_within_the_admission_budget(」，断言在它里面
  （`crates/singlefs-core/src/mount.rs:4660`「let raised_to_the_ceiling =」起）。所以本进程准入路径上没有一处在没有有效根时 panic 而不是被这一处拦下的（读代码，**推的**）。
- 网变宽了：断言现在罩住整个 `raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk` 的结果，不只算上限那一步。能不能多罩：产出 `RecoveryFailure::NoValidRoot` 的位置，
  `crates/singlefs-core/src/` 下 grep 得 `crates/singlefs-core/src/mount.rs:3978`（算上限，`.ok_or(RecoveryFailure::NoValidRoot)?;`）、`crates/singlefs-core/src/mount.rs:8455`（读阶段择根）、
  `crates/singlefs-core/src/mounted_read.rs:662`（只读挂载）、`crates/singlefs-core/src/recovery.rs:8001`；抬 F 这一串里只有 `crates/singlefs-core/src/mount.rs:3978` 在调用图上（读代码，**推的**），所以断言的触发集合与以前相同。
- `raise_rollback_floor` 一并变不是越出 T2：它与公开入口共用 `raise_the_floor_through` 的准入那一支，断言原来在被共用的 `rollback_floor_ceiling_of_this_process` 里，
  要把断言挪走就必然两个入口一起变；想只改一个得把共用函数拆成两份，那才是越出。它的文档 `# Errors` 本来就列「算上限的错」。
- 新用例红绿：新树过（`out/new-pinned-and-new-tests.txt`）；把公开入口改回断言（`patches/mutation-6248-public-ceiling-entry-panics.patch`，即 `crates/mutations.tsv` 第 6248 行的替换）
  两条都红于 `变异：公开入口在一条有效根都没有时断言失败`（`out/new-t2-mutation-6248.txt`，复跑 `SAME`）。
- 连带：`crates/mutations.tsv` 第 6155 行的名字写「红在 rollback_floor_ceiling_of_this_process「现行根都经记根」那条断言」，那条断言已挪走。行本身仍红：
  施加它（`patches/mutation-6155-ceiling-rereads-skipped.patch`）后用例 `the_admission_ceiling_refuses_with_the_reread_refusal_when_every_root_ring_slot_reads_bad_only_on_its_own_read`
  红在 `走不到单子外的同名拒因 Recovery(NoValidRoot)`（`out/new-t2-mutation-6155.txt`，量的）。只是名字陈旧，下一批改名字。
- push 站点那一个断言本身没有会红的用例（走不到：公开 API 造不出「这个进程挂载建的分配器没有根环表」）；这与修补前一样，不是新缺口。

### 推翻 U3 的现象

- 本进程准入路径上另有一处直接调 `rollback_floor_ceiling_of_this_process` 或 `raise_the_floor_through` 的准入支而不经 `push_one_floor_raise_within_the_admission_budget`：则那一处没被断言罩住。
- 抬 F 这一串里另有产出 `Recovery(NoValidRoot)` 的位置：则断言多罩了一类。

## U4　C928 解冻用例钉的是不是条款

- 条款：判决 T3（`research/prompts/m3-close-b19-code-r1-main-verification.md:21`）「补一条用例钉「解冻后下一次进 discard 调用点就发、不丢」」。
- 新用例走的是哪一个点：只走整理显式入口。`crates/singlefs-harness/tests/discard_after_the_frozen_publish_is_resent_reaches_the_segments_reclaimed_before_the_freeze.rs:274` 与 `:301` 各调一次
  `.run_compaction(&mut devices)`（冻结着一次、重发成功之后一次），重发在 `:291`。三个调用点是 `crates/singlefs-core/src/mounted_session.rs:2136`
  「published.discards_after_the_change = discard_the_segments_reusable_as_a_whole(」（用户改动发成之后）、`:2307`「run.discards = discard_the_segments_reusable_as_a_whole(」（整理显式入口）、
  `:2498`「run.discards = discard_the_segments_reusable_as_a_whole(」（后台入口）。三处共用 `crates/singlefs-core/src/mounted_session.rs:2924` 的函数，
  冻结判在它里面：`crates/singlefs-core/src/mounted_session.rs:2931`「if let Some(frozen) = allocator.frozen_publish() {」，所以一个点走到就罩住三个点的「冻结着不发」；
  「解冻后发」靠的是待发表在冻结期间没被摘、重发换回的分配器里还在（`crates/singlefs-core/src/transaction.rs:1612`「*allocator = allocator_after_the_publish;」），也在共用状态里，不在某个调用点里。
  所以：钉的是条款（解冻后下一次进调用点就发），走的是三个点里的一个，**够得着一个点**；另两个点用例没走（读代码，**推的**同一函数）。
- 冻结期间再冻结：造不出。发布路径第一道在任何读写之前拒：`crates/singlefs-core/src/transaction.rs:1545`
  「fn refuse_while_a_publish_is_frozen(allocator: &PoolAllocator) -> Result<(), PublishError> {」；落盘失败冻结时断言没有冻结着的：
  `crates/singlefs-core/src/allocator.rs:2057`「pub(crate) fn freeze_publish(&mut self, frozen: FrozenPublish) {」里的 `assert!`。
- 重发失败后：重发失败把那一次放回去，分配器没动：`crates/singlefs-core/src/transaction.rs:1604`「allocator.freeze_publish(frozen);」。我补了这一形（探针
  `u4_a_a_resend_that_fails_once_keeps_the_pending_discards_for_the_compaction_run_after_the_good_resend`，历史搭建逐字取自既有用例）：冻结 → 再武装一次盘 0 系统配置槽写错 → 重发报
  `BlockDevice(InputOutput(… 注入的写错 …))`、仍冻结 → 冻结着调整理入口一段不发 → 撤故障重发成功 → 整理显式入口：`PROBE name=u4a_after_failed_then_good_resend discarded=2 still_reusable=1`，
  断言「装冻结那次落点的段不发、冻结之前回收且仍整段可再分配的段每块盘都发了」过（量的，`out/new-u4.txt`，复跑 `SAME`）。**没丢**。
- 用户改动那个点我也试了（探针 `u4_b_the_user_change_after_the_resend_discards_the_segments_reclaimed_before_the_freeze`）：这一条**没有分辨力**：重发之后的 `NewInodes` 自己的落点占去了
  冻结之前回收的最后一个候选段（`out/new-u4.txt`：`u4b_segment … segment=50432 reusable_after_the_change=false discarded_by_the_change=false`，`reusable_now_but_not_discarded=[]`），
  所以该改动没有可发的段、什么都没发，且没有「仍整段可再分配而没发」的段。结论：用户改动点没被我这条探针钉住（不是证明它丢了），要钉得先造出至少两段冻结之前回收、其中一段不被这次改动占去。
  后台入口没跑。
- 判定：T3「相符」维持。新用例是对的条款、够着一个点；重发失败形补上了；冻结期间再冻结造不出；用户改动点与后台点是同一函数，没独立钉。

### 推翻 U4 的现象

- 在重发失败之后再重发成功的历史上，整理显式入口没发冻结之前回收的段（我的 u4a 没出现）。
- 用户改动点或后台点上，有一段「仍整段可再分配、没被这次发布占去、没发 discard」（u4b 没造出这样的段，所以没出现也没排除）。
- 能在冻结着时造出第二次冻结：与 `crates/singlefs-core/src/allocator.rs:2057` 的断言和第一道矛盾。

## U5　C929 回卷之后过不过补写闸（设计判断）

候选 A「照今天不判」：回卷之后那块盘有自证槽，下一次挂载按有自证槽的盘走（落后支），不再过补写闸三、四、五。
候选 B「回卷后的下一次挂载仍判三、四、五」。条款没写：`.claude/kb/checks-owed.md:592`「先定条款：回卷之后这块盘算补写过还是算落后」。

### 今天的行为（量的，`out/new-u5.txt`）

- 回卷之后盘 1 的槽：第二轮盘 1 写报错（第一轮补写那一写落了槽 1、号 2），回卷写落槽 0、带回卷之前的号 1：`slots=[(1, InstanceGeneration(2)), (2, InstanceGeneration(1))]`。
  取号失败那次挂载只写了系统配置槽：`crates/singlefs-harness/tests/acquisition_rewriting_a_device_without_a_self_verified_system_configuration_rolls_back_on_a_write_failure.rs:156`
  「.all(|write| write.kind == StepKind::SystemConfigurationSlot),」。
- 下一次挂载：`MountMade instance=InstanceGeneration(3) devices_rewritten=[]`（探针 `u5_a_…`）。既有回卷用例把这个补写盘表丢了：同一文件 `:219`
  「let (instance_of_the_next_mount, _devices_rewritten_by_the_next_mount) =」。也就是说候选 A 今天成立，而且没有一条断言钉它。

### 替候选 A 辩一次

- 补写闸三、四、五判的是这块盘此刻的盘面（根环、journal 环、单元区、环槽读错）。回卷只写系统配置槽，盘上其余字节没动，所以下一次挂载重判与取号那一刻同值：
  没有「回卷让盘面变了」这回事，重判在无外力时只会重复首次的结论（读代码加上一条的断言，**推的**）。
- `devices_without_the_selected_version`（`crates/singlefs-core/src/mount.rs:6077`）里有自证槽的那一支不扫单元区、不看环槽读错（读这一个函数得的）：那是两槽全废这一档的额外戒心，因为全废的盘没有任何见证。回卷后的盘有两个自证槽，按落后支判
  （`.claude/kb/decisions/18-块里携带什么信息.md:322`「交进来的盘有一块不带就在取号之前、任何写之前拒可写」那一整条里「不带」的两种：两槽一份自证过的都没有；世代号最大那份落后且缺单元或缺根槽）。
- A 会错的盘面（拼出来的，量的）：回卷之后的盘 1 两槽，配上盘 1 单元区里有写序实例大于所选那一版的单元。探针 `u5_c_post_rollback_slots_on_a_unit_area_holding_units_of_a_later_instance_are_not_gated_again`：
  把「取号成功那次挂载崩在第一条记录之前」的盘面（盘 1 带号 2 写下的单元）里盘 1 两槽换成回卷之后那两槽，下一次挂载 `MountMade instance=InstanceGeneration(3) devices_rewritten=[]`。
  同一份单元区、盘 1 两槽仍是全废时，补写闸四拒（`u5_b_…` 第二条：`RewriteGateNotMet(UnitWrittenByALaterInstanceInItsUnitArea)`）。
  走到这个盘面要「回卷之后、下一次挂载之前，盘 1 的单元区被带外换成别的内容」；对池里任何一块有自证槽的盘，这样的带外换内容今天同样没有对应的闸。

### 替候选 B 辩一次

- 回卷之后的两槽是这个进程凭取号那一刻一次准入判断写下的见证；B 的意思是：这个见证不该让以后的挂载少判一次。上面 A 会错的盘面里，同一份单元区在槽全废时被拒、在回卷槽下被收，
  是「回卷把盘洗白」；`.claude/kb/decisions/18-块里携带什么信息.md:346` 起记过同族的「同池旧快照的盘经入口洗白」，那一条用户定案（2026-09-27）是在「可见」之后再核本盘设备号与落后支，取向是收严；同一条也定了落后支取「落后且缺单元」、不取「落后就拒」（`.claude/kb/decisions/18-块里携带什么信息.md:322`「落后支取「落后且缺单元」、不取「落后就拒」」），即收严的同时避开了对正常崩溃盘面的误拒。
- B 会错的盘面（量的）：取号成功那次挂载崩在第一条记录之前（盘 1 带自证槽、带号 2 写下的单元），下一次挂载本来该做成，探针 `u5_b_…` 第一条：
  `MountMade instance=InstanceGeneration(3) devices_rewritten=[]`。若 B 对这块盘重判闸四（盘 1 有写序实例大于所选那一版的单元），就是 `UnitWrittenByALaterInstanceInItsUnitArea`
  那一拒（同一份单元区、槽写 0 的那条探针量到的）：把一个正常的崩溃盘面永远拒成不能可写。回卷之后、再挂一次做成、再崩，同样产生这个盘面，所以 B 在「回卷」之后的自然历史上会误拒。
- B 要避开它就得认出「这块盘是回卷留下的」：回卷之后的盘与别的有自证槽的盘在盘上没有区别（上面第一条量的槽：两槽都自证、号不同），要认就得在系统配置里加字段（D22 已定项 9、26 的字段表
  与格式冻结的取舍，不是这一轮能定的），或者让回卷把零槽盘写回「不自证」。后一种（候选 C，不在被问的两个里）无标记也幂等，但它改了回卷的语义——
  `.claude/kb/decisions/18-块里携带什么信息.md:321`「先把已经写出的盘回卷成旧代号、每块盘只写一次回卷」——而且回卷写不成时留下比今天更坏的盘面。不推荐。

### 推荐

**A**。条款补一句：补写闸是取号那一刻对盘面的一次准入判断；取号失败回卷之后这块盘有自证槽，下一次挂载按有自证槽的盘走落后支，不再过补写闸（回卷只写系统配置槽，盘上其余字节没动）。
回卷用例（上面那个文件的 `:219` 一带）补断言：下一次挂载 `devices_rewritten_by_the_acquisition` 为空；变异：回卷之后把这块盘再交补写闸 / 回卷之后拒这块盘，两条必红。
A 的残余风险是带外换内容，对所有有自证槽的盘同形，不是回卷独有；若用户要把「回卷不洗白」当成要求，才走 C 那条改回卷语义的路，那是用户的判断，我不替他定。

### 推翻 U5 的现象

- 造出回卷之后的盘面，不经带外改动，下一次挂载按 A 收下、而按补写闸三、四、五该拒：则 A 会错在自然历史上。
- 造出取号成功后崩溃的正常盘面，B 的重判不拒：则 B 没有误拒的盘面。

## U6　第一轮判「相符」靠读码：核一次

读码依据最薄的是 T1（跨写者、池级 checker、GPU 内核三处还原，平局规则只在代码注释里）与 T5（阈值、每片行数、「链在一片之内」三个数合起来的外推）；T3 已在 U4 用探针核了；
T6 的 `root_record.rs` 搬进 `verus!` 是体量最大的一块。各核一次：

- **T5（探针 `u6_the_default_threshold_starts_the_sweep_at_366_rows_and_the_chain_stays_in_one_page`）**：S = 4 的 12 槽小环池上，产品路径（开关关着、阈值 365）反复可写挂载加卸载 420 次，
  `out/new-u6.txt`：`u6_first_sweep mount_number=366 last_not_swept_rows=365`；该次清扫 `RowsDeleted deleted=364`、挂载后行数 2；
  `max_rows_at_a_not_swept_decision=365 max_rows_after_a_mount=365 mounts_with_rows_over_one_page=0`（量的）。代码对得上：
  `crates/singlefs-core/src/instance_table_sweep.rs:809`「if rows <= configuration.row_count_threshold.0 {」（366 行起扫）；每片 369：`crates/singlefs-core/src/admission.rs:530`
  「pub const INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE: u64 = INSTANCE_TABLE_PAGE_RECORDS - 1;」；预留里的片数 `crates/singlefs-core/src/admission.rs:566`
  「.div_ceil(INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE)」取 (rows0 + 3)：下一次挂载读到 365 行、写 1 行，rows0 = 366，366 + 3 = 369，刚好一片。修补后的文档（`_m3-close-b19-code-r2-diff.md`）
  说的「366 行起」「每片仍 369」「清扫做成时链留在一片之内」在这个池上都成立；「一行不删、一次写几行、开关交给后台时链照样长出第二片」那几形我没造（只读了代码）。
  **没被冤枉**。口径：这是一个池（12 槽环，旧实例很快轮出环）的数，别的环长上清扫能删几行不同。
- **T1（变异 `crates/mutations.tsv` 第 6144 行，`patches/mutation-6144-tie-broken-backwards.patch`）**：把平局倒着破后，
  `two_genesis_ring_slots_pinned_at_txg_zero_carry_the_lower_ring_slot_as_the_pinned_ring_slot` 红（`out/new-t1-mutation-6144.txt`），不变异时同一目标 12 条全过
  （`out/new-root-record-target.txt`），所以钉着平局规则的用例真会红。读码依据对得上：`crates/singlefs-core/src/allocator.rs:2793`「.min_by_key(|pin| (pin.pinned_txg, pin.ring_slot));」，
  `crates/singlefs-core/src/root_ring.rs:12`「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]」（区域在前、槽号在后）。**没被冤枉**。
  GPU 内核与池级 checker 的还原我没跑（题面禁碰 GPU，没有现成的会红用例我可复用），仍是读码。
- **T3**：见 U4，**没被冤枉**（加了重发失败形；用户改动点与后台点没有分辨力）。
- **T6**：`root_record.rs` 的 `verus!` 化。只做到：核心层写出的根字节被池级 checker 的独立解析器读回，同一目标 12 条、`singlefs-core` 库内 `root_record` 7 条都过
  （`out/new-root-record-target.txt`、`out/new-core-lib-root-record.txt`）。没做的是新旧 `to_slot` 的字节对比：开工快照之前的版本（`git show HEAD:` 那份）的
  `of_the_lowest_pin` 收的是 `Option<CheckpointTxg>`，现在收 `Option<u64>`，不能直接放回去编，对比要另搭模块改名并排编。这一块**核不动**，不判冤枉与否。

### 推翻 U6 的现象

- T5：任一环长上默认阈值 365 的产品路径，在挂载末尾行数不超过 365 时下一次挂载的准入按两片留，或清扫第一次触发的行数不是 366：推翻「366 行起」与「链留在一片之内」两句。
- T1：造出平局的钉槽盘面，写者交出的 p 与池级 checker 或 GPU 内核还原出的槽不同。
- T6：新旧 `to_slot` 对同一条根交出不同字节。

## 另外看到的（不在六格里，但判决可能用到）

- `crates/mutations.tsv` 第 6155 行的名字写红在已挪走的断言（见 U3），行本身仍红；下一批改名字。
- 判决 T4 一行写「钉读序数的用例按新次序重核」，真因见 U1：它们对次序不敏感。
- 两样同时成立（逐盘核拒加预演拒）时报哪一个，今天没有条款也没有既有用例，只有本腿的三条探针；升格入库的形态：`u1_c841_image_with_device_one_blank_reports_the_check_not_c841_when_the_check_comes_first`
  （旧次序红、新次序绿，量的）。

## 没做什么

- 没读禁读清单里的文件；没跑 checker 档、重型测试、GPU、整包 `cargo test`；没做 git 写；没改主工作区任何文件（所有编译在草稿目录的两棵副本里，探针与变异都施加在副本上）。
- U1：「逐盘核读时瞬时读错」没造探针，只有读代码的依据。
- U2：预演停在非读类错上又有补写盘的盘面没造出来（窄盘镜像上没有现成造法），结论里这一段是推的。
- U4：用户改动点与后台点我的探针没有分辨力（u4b 没造出可发的段），后台入口没跑。
- U5：候选 B 没有实现，只量了它该拒与不该拒的盘面；候选 C（回卷把零槽盘写回不自证）没做。拼出的盘面（u5c）是组合出来的，不是一条自然历史。
- U6：T1 的 GPU 内核一致性没跑；T6 的 `root_record.rs` 新旧字节对比没做；T5 只在一个池（12 槽环）上量。
- 没替主 agent撤销或维持判决，只交复核结果。
- 没有「要主 agent 跑的命令」：复跑命令在文件头，我已跑通。
- 清理：交回前删了草稿目录里的仓副本与编译目录：`new` 44M、`old` 44M、`frozen` 44M、`target-new` 6.5G、`target-old` 3.2G、复跑工作目录 `rerun-3` 9.8G（`du -sh`，量的，删之前记的）。
  草稿目录里只留日志与补丁文本（`run-*.txt`、`rerun-3.log`、`r2-fix.diff` 等）。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-code-r2-sonnet-output.md --background research/prompts/_m3-close-b19-code-r2-background.md` 末行：
「✓ 核了 36 处引文、25 处裸行号，对上 36 处，没判 25 处（其中裸行号没有引文可比 25 处；逐处列在上面）」。裸行号我在写之前逐个 `grep -nF` 现取过，它们没有引文可比所以列进「没判」。
