# 代码轮第三轮（最终轮）正推腿（Sonnet）：W1 判决对实现、数能不能核

日期 2026-10-01。轮名 m3-code-r3。立场：正推。只读与推导，没编没跑测试；跑过的非读命令是 sha256sum、cite-check、grep / awk 计数、
草稿目录里的一段 python ctypes 调 `fallocate`（第五节）。被判代码读冻结副本 `/tmp/claude-1000/m3-code-r3/tree/`，下文代码行号都是那里的行号。
草稿 `/tmp/claude-1000/m3-code-r3-sonnet/`，模型目录 `research/prompts/m3-code-r3-sonnet-model/`（只放变异行锚点核对脚本与它的输出，不是攻方模型）。
禁读清单（opus 腿的产出、模型目录、草稿目录）一个没读。

## 一、结论（逐条依据在后面各节）

1. J1–J5 全部做到，没有一条没做；字面偏离一处（J3 的 AC，见第二节 J3），偏离有据，多做 3 条用例。
2. 13 行变异的锚点在冻结副本里各恰好命中一次，点名的测试各恰好一条。
3. 单处等价写法与攻方原样在这几条新用例上结局相同：RG、Z0、E1 两边文本相同；RP、EC 在新用例上结局相同；AC 在新用例上相同、在别的盘面上不同（第三节）。
4. 前两轮判决引的产物行、复跑数全部在各自的日志里逐字找到（第四节）；两处「数」今天复现不出原数：r1 的「主仓为根 88 / 88」与 r2 的「3 处对不上」，原因是被引的 kb 与规则文件在判决之后又改过，行号漂了。
5. 判决或实现里没被判决点名、但同类的残留两处：`new_pool_file_creation_on_device.rs` 第 834、1251 行旧注释；J2 只钉了抬 F 路径上两次核盘里的第一次（第二节）。都不算 J 没做到，写出来交主 agent 定。

## 二、W1-① 逐条核 J1–J5：做到、没做到、多做

材料先核（命令原样）：

```
$ cd /tmp/claude-1000/m3-code-r3/tree && sha256sum -c research/prompts/m3-code-r3-snapshot/crates-sha256.txt | grep -c ": OK"   （路径取仓内）
17
$ cd /tmp/claude-1000/m3-r1/merge-wt && sha256sum -c …/m3-code-r3-snapshot/crates-sha256.txt | grep -c ": OK"
17
$ grep -c '^diff -ruN' research/prompts/m3-code-r3-diff.patch ; wc -l research/prompts/m3-code-r3-diff.patch
5
698 research/prompts/m3-code-r3-diff.patch
$ grep -c '^#\[test\]' <r2 树与 r3 树>/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs
r2: 21  r3: 31
$ grep -c '#\[test\]' <r2 树与 r3 树>/crates/singlefs-core/src/block_device.rs
r2: 14  r3: 15
```

17 份与登记一致；merge-wt（跑合入验证的那份）与冻结副本逐份相同，所以验证的就是被判的。

### J1：做到

判决原文：「H2 加一条 txg 8（区域 2）的用例。RG 变异必须红。」（`research/prompts/m3-code-r2-main-verification.md:36`）

- 做了：「const PUBLISH_TXG_WITH_ITS_ROOT_IN_REGION_TWO: CheckpointTxg = CheckpointTxg(8);」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:760`），
  用例 `a_session_publish_onto_device_zero_holding_the_publish_root_in_region_two_taken_before_its_root_slot_write_is_refused_before_any_write`（同文件第 1135 行）走会话发布、切在根槽 FUA 之前，
  断言拒成缺根并点名盘 0（`assert_refused_as_behind_without_the_root`，两次读到的都是 `NEVER_WRITTEN_RING_SLOT`）。
- 钉的是判决要钉的那句：txg 8 → 区域 2（`target_for_publish` 里 `txg % 3`）、`region_devices` 是 [盘 0, 盘 1, 盘 0]，区域 2 归盘 0。RG 把区域号当盘号，区域 2 变成盘表里没有的盘 2，
  `「if device_of_the_ring_slot != device {」`（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3418`）对盘 0 取 `None`，缺根不核。区域 0、1 的号恰好等于盘号（0、1），所以 txg 6、7、27 放得过 RG，只有 txg 8 分得出——判决选对了点。
- 没做到：无。多做：无。
- 推翻它的现象：在 RG 变异下整份 entries 二进制跑出来这一条不红。实现员的证红日志（`/tmp/claude-1000/m3-r1/code-r2-fixes/prove-red-logs-harness/001.log`）是「30 passed; 1 failed」，失败的就是这一条（我核了日志里的失败名单）。

### J2：做到

判决原文：「H1 那一形（只让根槽第一次读坏）在正常卸载、准入抬 F、管理员回退三个入口上各加一格。」（`research/prompts/m3-code-r2-main-verification.md:37`）

- 做了：一个共用函数 `fn one_transient_read_error_of_the_publish_root_slot_through(entry: EntryAfterTheMount) {`（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:1383`）加三条入口用例；
  注入位置是 `const ROOT_SLOT_READ_OF_THE_BEHIND_CHECK_IN_THE_OTHER_THREE_ENTRIES: u64 = 1;`（同文件第 1162 行）。三条都断言：注入真发生 1 次（`plan.fired_count()`）、入口做成（`refusal.is_none()`）、做完池级 checker 无违例。
- 钉的是判决要钉的那句：缺根那一半第一次读坏、重读读得出 P 时不拒。第 1 次读就是那一判的第一次读，有攻方日志为证：RP 原样下「Y2R txg=6 entry=NormalUnmount occ=once n=1 fired=1 writes=0 outcome=behind-root(DeviceIdentity(0),Bad(Unreadable),Bad(Unreadable))」（`research/prompts/m3-code-r2-opus-model/logs/wrong/RP_no_reread_in_the_three_other_entries/y2r-6.log:11`），
  原样代码同一格「Y2R txg=6 entry=NormalUnmount occ=once n=1 fired=1 writes=44 outcome=ok checker=0」（`research/prompts/m3-code-r2-opus-model/logs/orig/y2r-6.log:11`）。
- 没做到：无。多做：断言里多了「池级 checker 不红」（H1 原有两条没有这一句）；与判决不冲突。
- 观察（不算没做到）：抬 F 那条路在同一入口里核两遍盘表，攻方 RP 日志里第 3 次读也是缺根那一判：「Y2R txg=6 entry=FloorRaiseToTheAdmissionCeiling occ=once n=3 fired=1 writes=0 outcome=behind-root(DeviceIdentity(0),Bad(Unreadable),Bad(Unreadable))」（`research/prompts/m3-code-r2-opus-model/logs/wrong/RP_no_reread_in_the_three_other_entries/y2r-6.log:33`）。
  J2 的用例只注入第 1 次读。只在第二遍核盘上去掉重读的错实现，J2 三条放得过（推的，没跑；判决 J2 字面是「第一次读坏」，不要求这一格）。
- 推翻它的现象：RP 变异下 J2 三条里有一条不红（实现员日志 `prove-red-logs-harness/002.log`：「25 passed; 6 failed」，名单里有 J2 三条加 H1 会话发布、H1 可写挂载、H3 一条）。

（更正上一段：第一个命令块里的路径是简写，实跑用的是 `/home/fy5090/code/singlefs/research/prompts/m3-code-r3-snapshot/crates-sha256.txt` 的绝对路径；输出的 17 是 `grep -c ": OK"` 的原样结果。）

### J3：做到，字面偏离一处，多做 3 条

判决原文：「三个入口各加一条 txg 27 转圈的用例」（`research/prompts/m3-code-r2-main-verification.md:38`）与「做目标；EC、AC 两种错实现必须红。」（同文件第 38 行，中间一句是回退取前一版）。

- 做了：共用函数 `fn root_device_whose_ring_slot_holds_the_root_of_the_previous_lap_through(`（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:1456`）加六条入口用例：
  txg 27 三条（常量 `const PUBLISH_TXG_ONE_LAP_AFTER_THE_FIRST_FILE: CheckpointTxg = CheckpointTxg(27);` 在同文件第 749 行，住盘 0）、txg 25 三条（常量 `CheckpointTxg(25);` 在第 763 行，住盘 1）。
  回退那条取前一版：`RollbackTargetOfTheEntry::VersionBeforeTheCurrent,`（同文件第 1465 行）经 `let target = match rollback_target {`（第 268 行）算成现行版 txg 减 1。
  断言走原有的 `assert_refused_as_behind_without_the_root`：点名 P 的根住的那块盘、缺的是根、两次读到的都是上一圈那条根，加两块盘逐字节不变、会话现行版本不动。
- 字面偏离：判决要 AC 在 txg 27 上红，做不到。txg 27 → 区域 0 → 盘 0 = `region_devices[0]`，AC（只查 `region_devices[0]`）查的恰是对的那块盘。
  攻方自己的日志证实：AC 下 txg 27 照拒「Y3 txg=27 region=0 slot=1 root_dev=DeviceIdentity(0) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=behind-root(DeviceIdentity(0),AnotherRoot { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }」
  （`research/prompts/m3-code-r2-opus-model/logs/wrong/AC_only_region_zero_device_in_the_three_other_entries/y3-24..32-first.log:16`），
  txg 25 才做成「Y3 txg=25 region=1 slot=0 root_dev=DeviceIdentity(1) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=ok」（同日志第 8 行）。
  攻方报告当时就写了这一点：「区域 1 且转圈的 txg 25、28、31 上卸载、准入抬 F、回退做成、checker 红」（`research/prompts/m3-code-r2-opus-output.md:244`），判决第三节 J3 把「txg 27」和「AC 必须红」写在了一起，是判决自己的前提没对上攻方的表。
  实现员发现后另加 txg 25 三条，做法对；判决 J3 字面该改成「EC 在 txg 27 与 25 红，AC 只在 txg 25 红」。
- txg 25 的槽里确是上一圈的 txg 1：攻方原样代码的一格「Y3 txg=25 region=1 slot=0 root_dev=DeviceIdentity(1) cut=BeforeItsRootSlotWrite entry=SessionPublish outcome=behind-root(DeviceIdentity(1),AnotherRoot { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(1) }」（`research/prompts/m3-code-r2-opus-model/logs/orig/y3-c.log:11`），与 `ROOT_OF_THE_FIRST_WARM_UP_PUBLISH_A_LAP_BEFORE` 的值一致。
- EC 在 txg 27、txg 25 两种上都做成：「Y3 txg=27 region=0 slot=1 root_dev=DeviceIdentity(0) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=ok」（`research/prompts/m3-code-r2-opus-model/logs/wrong/EC_any_self_verified_root_in_the_three_other_entries/y3-24..32-first.log:16`）、
  「Y3 txg=25 region=1 slot=0 root_dev=DeviceIdentity(1) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=ok」（同日志第 8 行）；所以这六条对 EC 都是有效的探测。
- 多做：txg 25 三条（判决没要，理由如上）。没做到：无（AC 的 txg 27 是判决写错，不是实现漏）。
- 推翻它的现象：AC 变异下 txg 25 那三条里有一条不红，或 EC 变异下六条里有一条不红。实现员日志：EC「25 passed; 6 failed」、AC「28 passed; 3 failed」，失败名单我逐条核过——EC 红六条（txg 27 三、txg 25 三），AC 红的正是 txg 25 三条（`prove-red-logs-harness/003.log`、`004.log`）。

### J4：四点都做到；Z0 稀疏盘那一行没证红（推得出必红）；同类旧注释另有残留

判决原文（`research/prompts/m3-code-r2-main-verification.md:40-43`）四句。

- H5 单测：偏移从 3 处到 5 处，新加「offset_one_block_past_the_end,」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/block_device.rs:1226`）和半块处 256，长度都是 0；期望里这两格写成 `refused(OutOfRange{…})`、`refused(Unaligned{…})`。
  文件后端对期望：「file_outcomes, expected,」（同文件第 1312 行），两臂 `O_DIRECT` 对文件后端：「direct_outcomes, file_outcomes,」（同文件第 1331 行）。三个结局逐项相同且都是拒，判决「三个后端都拒」里文件与 `O_DIRECT` 两个在这条单测里，稀疏盘在下一条。Z0 变异下文件后端对这两格交 `Ok`，第 1312 行红；实现员日志红在 `block_device.rs:1314`（`prove-red-logs-core/001.log`，「138 passed; 1 failed」）。
- 稀疏盘那份：`requests()` 加「(DeviceOffsetInBytes(2 * DEVICE_BYTES), 0),」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-harness/tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs:37`）与「(DeviceOffsetInBytes(256), 0),」（同文件第 38 行）；
  两处计数跟着改成 8 与 26：「refused_writes, 8,」（第 173 行）、「(4, 26),」（第 275 行）。我数了一遍：写被拒 = 4 个越界（含溢出一个）+ 新增越界 1 + 没对齐 2 + 新增没对齐 1 = 8；读不出 = 两块盘各 8 + 第 7 块盘 10 = 26，与注释一致。
- SAFETY 句：「跨过文件末尾的打洞，文件内那一半照常打成 0，文件外的部分内核不做，都不碰内存。返回值在下面查了。」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/block_device.rs:392`）与判决给的句子逐字相同；描述符、偏移、长度三类词在前几行还在（我读了第 385–391 行）。
  这句内核行为我在本机 ext4 上复现：长度 0 返回 -1 errno 22，跨末尾的打洞返回 0、文件内那一半被打成 0、文件长度不变（第五节命令输出）。
- 旧注释：「有盘落后于现行那一版又缺它的单元或它的根」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-harness/src/model_comparison.rs:780`）已改。
- 残留（判决没点名）：同一句的旧写法还在 checker-tier 的 `new_pool_file_creation_on_device.rs`：
  「有盘落后于现行那一版又缺它的单元：挂着之后的入口在任何写之前拒，这个二进制不换盘，走不到。」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:834`，同样一句在第 1251 行）。
  判决写的是「model_comparison.rs:780」一处，所以这不是 J4 没做到；是同类的漏网，由主 agent 定改不改（那个文件在重型档，我不动）。
- Z0 稀疏盘那一行（`research/prompts/m3-code-r3-diff.patch` 里第二条 J4 行）没证红，实现员报告自己写了。推导：稀疏盘那份测试基线绿（`/tmp/claude-1000/m3-r1/code-r2-fixes/prove-red-logs-harness/baseline.log:51` 「test result: ok. 2 passed」），
  其中清零一格是 `verdict_of(sparse_device.write_zeroes_at(offset, length))` 与「verdict_of(file_device.write_zeroes_at(offset, length)),」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-harness/tests/sparse_devices_refuse_requests_outside_the_device_like_the_file_backend.rs:160`）逐个相等；
  新增两项在基线下文件后端拒（偏移越界、没对齐），所以稀疏盘也拒（否则基线红）。Z0 只动文件后端，让它对这两项交 `Ok`、稀疏盘不动，第 160 行的 assert_eq 红。前提只有两个：基线绿、Z0 不碰稀疏盘。推的，没跑；W2 要在副本上跑一次。
- 推翻它的现象：Z0 变异下稀疏盘那份测试仍绿。

### J5：做到

判决原文：「H8 的测试开关改成」…（`research/prompts/m3-code-r2-main-verification.md:44`，原句里有嵌套引号，这里只引前半句）；单测钉 k = 1、2 都照常打成洞，E1 变异必须红。

- 做了：开关改成带数的变体，循环里的闸门「} if punch_attempts_interrupted < punches_interrupted_before_the_real_one => {」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/block_device.rs:457`）；
  断言抽成 `fn assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error(`（同文件第 1126 行），k = 1 沿用原名（调用在第 1193 行 `assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error(1);`）、
  k = 2 新加（第 1201 行 `assert_punched_again_after_interruptions_without_falling_back_or_reporting_an_error(2);`）。旧变体名全仓只剩这一个文件里的新名（`grep -rn ActAsIfTheFirstPunch` 命中四处，都在 `block_device.rs`）。
- 钉的是判决要钉的那句：E1 在第二次被打断时报错。E1 的插入点是「punch_attempts_interrupted += 1;」（同文件第 494 行）之后：k = 1 时计数到 1 就真打、不触发；k = 2 时第二次被打断计数到 2，`> 1` 成立，报 `InputOutput`，k = 2 红、k = 1 绿。
  实现员日志：E1 红在 k = 2 那条，k = 1 没红（`prove-red-logs-core/002.log`，「138 passed; 1 failed」，失败名单只有 `…twice_in_a_row…` 一条）。
- 没做到：无。多做：无（生产路径没动：开关只在测试变体的分支里生效，`PunchHoleOrWriteZeroBytesIfUnsupported` 那条臂仍直接打洞）。
- 钉的强度（推的）：k = 2 排除「重打只给一次」，但「重打至多给 N 次，N ≥ 2」的错实现两条都放得过。判决要的是 k = 1、2，不要更多；这个留给 W2 去量。
- 推翻它的现象：E1 变异下 k = 2 绿，或 k = 1 红。

### 变异行：13 行，锚点与点名测试都对得上

命令：`python3 research/prompts/m3-code-r3-sonnet-model/check_rows.py`（读冻结副本 `crates/mutations.tsv`，把每行的原文按 `\n` 还原成换行后数它在目标文件里的命中次数，
再数点名的测试名在目标文件里作为 `fn` 出现几次；脚本 sha256 前 8 位 `b85fdaf2`，输出 `check_rows.out` 前 8 位 `59999937`）。输出头两行与逐行汇总原样：

```
total lines 1511
m3-code-r2 rows 13
（其后 13 行，每行都是：1 anchor_hits; fn_hits=1; fields=6; m3-code-r2 J…）
```

- 13 行 = J1 一、J2 三、J3 六（EC 三、AC 三）、J4 二、J5 一，与实现员报告的「追加 13 行」一致；`diff` 里 `^+m3-code-r2` 数出来也是 13，表从 1498 行到 1511 行。
- 每行的锚点命中 1 次、点名测试 `fn_hits=1`，没有一行是静默不命中的假行。
- 这一步核的是行本身合不合格；行在编译之后红不红，只有实现员的证红日志（上文各节引过）和 W2 的重跑能证，我不编不跑。

## 三、W1-③ 单处等价写法与攻方原样：在这几条用例上结局是否真相同

攻方原样在 `research/prompts/m3-code-r2-opus-model/wrongs.py`。共同前提：`devices_behind_the_current_version` 有两个调用点——挂着之后三个入口经 `caller_inputs_agreeing_with_the_disk` 的「devices_behind_the_current_version(devices, &parameters_of_the_pool, current);」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3834`），
和会话发布的「let devices_behind = devices_behind_the_current_version(」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mounted_session.rs:287`）。变异行里 EC、AC 的过滤只改前一处，原样的 thread_local 也只在前一处置位，两边的作用域相同。

| 错实现 | 攻方原样 | 变异行 | 新用例上结局 | 两边差在哪 |
|---|---|---|---|---|
| RG | 区域号当盘号 | 同一替换（`expect` 的提示字不同） | 相同：红 J1 | 无 |
| Z0（核心单测、稀疏盘各一行） | 长度 0 早退挪到 `check_request` 前 | 同一替换 | 相同：红 H5 单测；稀疏盘那份推得出红（J4 一节） | 无 |
| E1 | 第二次被打断就报错 | 同一替换 | 相同：红 k = 2、k = 1 绿 | 无 |
| RP | thread_local，只在三个入口那一处调用前后置位，那期间 `reread` 取 `first_reading` | 全局：`let reread = first_reading;`（锚点「let reread = reading_other_than_the_version_root()?;」，`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3444`） | J2 三条相同：第 1 次读坏时 `first_reading` 是 `Bad`，不重读就把盘拒成缺根，两边都红 | 全局行连会话发布与可写挂载也不重读：实现员日志里红 6 条（J2 三、H1 会话发布、H1 可写挂载、H3 一），原样只会红 J2 三条（推的；原样下 H1 各条仍绿是 r2 量过的：entries 21 条全绿，这正是 r2 打中的缺口）。所以这一行红，说明不了「J2 是唯一防线」，只说明 J2 三条对它红 |
| EC | thread_local，那一处里守卫 `true \|\|` | 在 3834 那一处后过滤掉「两次读里有一次是 AnotherRoot」的缺根项 | 相同（见下） | 读盘次数：原样第一次读到别的根就返回，只读 1 次；过滤版读 2 次。J3 六条不数读次数 |
| AC | thread_local，那一处里归属盘一律取 `region_devices[0]` | 在 3834 那一处后过滤掉盘不是 `region_devices[0]` 的缺根项 | J3 六条相同（见下） | 盘面不同时结局不同（见下） |

EC 的逐情形推导。守卫在「if root.instance == instance && root.checkpoint_txg == checkpoint_txg =>」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3429`），
第一次读与重读在第 3443、3444 行。两次读结果 (第一次, 重读) 取遍 {P 自己, 别的根, 读坏}：

| (第一次, 重读) | 原样 EC | 过滤版 EC |
|---|---|---|
| (P 自己, —) | 不拒 | 不拒（无缺根项） |
| (别的根, 任意) | 第一次就返回 `None`，不拒 | 项里有一次是别的根，滤掉，不拒 |
| (读坏, 别的根) | 重读返回 `None`，不拒 | 同上，滤掉，不拒 |
| (读坏, P 自己) | 不拒 | 不拒 |
| (读坏, 读坏) | 拒 | 保留，拒 |

五行结局逐行相同；读盘是确定的，所以对任一盘面两边的「拒 / 不拒」集合相同。J3 六条的盘面是 (别的根, 别的根)：两边都不拒、入口做成、测试红。

AC 的推导。原样：那一处里 `device_of_the_ring_slot` 恒为 `region_devices[0]`，再走「if device_of_the_ring_slot != device {」（`/tmp/claude-1000/m3-code-r3/tree/crates/singlefs-core/src/mount.rs:3418`）；
对每块落后的盘 d：d 不是盘 0 ⇒ 原样 `None`，过滤版算出缺根项后因盘不是盘 0 被滤掉，两边都不拒；d 是盘 0 且 P 的根槽确实住盘 0 ⇒ 两边都按真实读数判，相同；
d 是盘 0 而 P 的根槽住盘 1（区域 1）⇒ 原样会去读盘 1 的那一槽、读不出 P 的根就把盘 0 拒了，过滤版不拒。只有最后一种两边不同，要求盘 0 落后、而 P 的根槽在盘 1。
J3 六条里被换的只有一块盘：txg 27 换盘 0（根槽住盘 0）、txg 25 换盘 1（根槽住盘 1）；另一块盘是 P 做完的样子，不落后，走不到最后一种。所以六条上两边相同：txg 27 三条 AC 下都绿、txg 25 三条 AC 下都红。

- 实现员报告自己推的结局（「RP 原样红 J2 三条，EC 原样红 J3 六条，AC 原样红 J3 txg 25 三条」）与我的推导一致；它说没量过，我也没量过，W2 量。
- 攻方日志里能对上的：AC 原样 txg 25 做成、27 不做成（J3 一节引的两行）；RP 原样 n = 1 缺根、原样代码 n = 1 做成（J2 一节引的两行）；EC 原样 txg 27、25 做成（J3 一节引的两行）。
- 推翻它的现象：W2 在副本上套攻方原样，J2 三条或 J3 六条里有一条与变异行的红 / 绿不同。差在盘面上的那一格（盘 0 落后、根槽住盘 1）：新用例没有，今天没有哪条用例能分出 AC 原样与过滤版。

## 四、W1-④ 前两轮判决引的数：能不能在各自的日志里逐字找到

命令与原样输出（每条这一次真跑；r1、r2 的冻结副本与攻方模型目录今天都还在）：

```
$ cite-check m3-code-r1-{sonnet,opus}-output.md --root /tmp/claude-1000/m3-code-r1/tree
  ✓ 核了 88 处引文，对上 42 处，没判 46 处（逐处列在上面）
$ cite-check m3-code-r1-{sonnet,opus}-output.md --root .
  ✗ 核了 88 处引文，23 处对不上，0 处没判（逐处列在上面）
$ cite-check m3-code-r2-{sonnet,opus}-output.md --root /tmp/claude-1000/m3-code-r2/tree
  ✓ 核了 34 处引文，对上 16 处，没判 18 处（逐处列在上面）
$ cite-check m3-code-r2-{sonnet,opus}-output.md --root .
  ✗ 核了 34 处引文，5 处对不上，0 处没判（逐处列在上面）
$ sha256sum -c m3-code-r1-snapshot/crates-sha256.txt（在 r1 冻结副本里）| grep -c ": OK"   →  12
$ sha256sum -c m3-code-r2-snapshot/crates-sha256.txt（在 r2 冻结副本里）| grep -c ": OK"   →  17
$ （r1 攻方模型目录）sha256sum -c SHA256SUMS-logs.txt | grep -c ": OK"; wc -l < SHA256SUMS-logs.txt   →  84 / 84
$ （r2 攻方模型目录）sha256sum -c SHA256SUMS-logs.txt | grep -c ": OK"; wc -l < SHA256SUMS-logs.txt   →  291 / 291
```

### r1 判决（`research/prompts/m3-code-r1-main-verification.md`）

- 12 份冻结副本 sha 相符（判决第 14 行）；`SHA256SUMS-logs.txt` 逐份 OK（判决第 16 行，今天 84 份全 OK）。
- 四条 MUT 行逐字在：「MUT A_only_device_zero	GREEN	rc=0	failed=[]	results=94」（`research/prompts/m3-code-r1-opus-model/logs5/full-suite.out:1`）、
  「MUT D_no_reread	GREEN	rc=0	failed=[]	results=94」（同文件第 2 行）、「MUT E_any_self_verified_root	GREEN	rc=0	failed=[]	results=94」（同文件第 3 行）、
  「MUT BASE	GREEN	rc=0	failed=[]	results=94」（`research/prompts/m3-code-r1-opus-model/logs/full-suite.out:1`）。
  同名的 A、D、E 三行另有 `results=3` 的一组在 `logs/wrong.out`（只跑原型三条），判决引的是 `results=94` 那组，在 `logs5`。
- 会话发布那一行：「SwappedForTheCutAfterTheRootSlotWrite session_publish fail_read=1 outcome=DevicesBehindTheCurrentVersionAndMissingItsUnits」（`research/prompts/m3-code-r1-opus-model/logs/reread-no-reread.log:10`，原样代码那一格是同日志之外的 `reread-orig.log` 第 7 行 `outcome=Ok`）。
- 判决第 2、3 行的 168、226：`logs/stale-m1-A_only_device_zero.log` 第 778 行的 SUMMARY 末尾「flagged=168」、`logs/stale-wrap-E.log` 第 175 行「flagged=226」逐字在（判决写的「格」就是这里的 flagged）。
- 第 8 行的 364 911：日志里没有现成的合计行，是 `logs5/crash-part0.log`、`crash-part1.log`、`crash-part2.log` 里每个 `LAYER0_PARALLEL_START states=` 的和，我加了一遍：106613 + 82073 + 176225 = 364911，三份里分别 5、8、4 个这样的行，共 17 条流，与「17 条流」一致。
- 第 4、6 行的实测：本机 ext4 上 `fallocate(PUNCH_HOLE|KEEP_SIZE)` 长度 0 返回 -1 errno 22、越过文件末尾与跨文件末尾都返回 0——我在草稿目录重跑了（第五节命令输出），与判决相同。第 5 行引的「第 128 行 `remove_file`」在 r1 冻结副本里是「std::fs::remove_file(path).expect("清理镜像");」（`/tmp/claude-1000/m3-code-r1/tree/crates/singlefs-harness/tests/mkfs_on_image_files_punches_the_zeroed_ranges.rs:128`）。
- 「以主仓为根，对上 88 处」今天复现不出：23 处对不上，每一处我都核了，原文都还在，只是行号挪了（D18 314 → 317 共 7 处、323 → 326 共 2 处、338 → 342 共 2 处、340 → 344 共 1 处、184 → 157 或 185 共 1 处；`checks-owed.md` 397/459/460/477 各 2 处、各前移 3 行；D23 389 → 390 共 2 处）。
  D18 的修改时间是今天、晚于 r1 判决文件的修改时间，kb 在判决之后被写回过；这是证据冻结、kb 往前走带来的漂移，不是判决写错。
- 判决第 5 行的「`/tmp` 下现成两份 9 月 30 日留下的镜像，各实占 769M，主 agent 看清后删了」：删了，复核不了。

### r2 判决（`research/prompts/m3-code-r2-main-verification.md`）

- 17 份与 291 份今天都 OK。
- 「WRONG RG_region_number_as_device entries rc=0 test result: ok. 21 passed; 0 failed」（`research/prompts/m3-code-r2-opus-model/logs/wrongs-batch1b.out:29`）、
  「WRONG RP_no_reread_in_the_three_other_entries entries rc=0 test result: ok. 21 passed; 0 failed」（同文件第 47 行），`sweep-1` 到 `sweep-4` 各 `rc=0` 在同文件第 48–51 行；
  「WRONG EC_any_self_verified_root_in_the_three_other_entries y3-1 rc=0」（`research/prompts/m3-code-r2-opus-model/logs/chain3.out:12`）；
  「WRONG AC_only_region_zero_device_in_the_three_other_entries entries rc=0 test result: ok. 21 passed; 0 failed」（`research/prompts/m3-code-r2-opus-model/logs/chain4.out:5`）。都逐字在。
- 「格 Z0 h5、AC entries 用时 255 秒」（`research/prompts/m3-code-r2-opus-model/logs/progress.md:2`）、「格 COMBO_RG_RP_Z0_E1 全包 用时 915 秒」（同文件第 3 行）逐字在。组合「全绿」我数了：`logs/full/COMBO_RG_RP_Z0_E1/` 91 个 `test result`、728 passed、0 failed，里面有 `core-lib.log`（核心层单测也在）；`COMBO_EC_R3` 同为 91 / 728 / 0。
- 「✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了」（`research/prompts/m3-code-r2-opus-model/logs/prove-red.out:3`）逐字在。
- 第 7 行「27 个 txg × 2 种切法 × 4 个入口，结局相同」：`logs/orig/y3-a.log`、`y3-b.log`、`y3-c.log` 里含「Y3 txg=」的行共 216、不同 txg 27 个、切在根槽之前 108 行，108 行全是 `behind-root` 且 `unchanged=true checker=0`，与攻方报告第 16、221 行的 108 相同。
- 第 9 行的「2 304 次小段加 16 次整文件打洞 0 次」：辩方模型目录里小段两份日志 `punches_ok=1536` 与 `punches_ok=768`（和 2304）、整文件两份各 `big_punches=8`（和 16），四份都 `EINTR=0`，逐字在（`research/prompts/m3-code-r2-sonnet-model/logs/eintr-small-punches-ext4.log`、`eintr-small-punches-tmpfs.log`、`eintr-big-punch-ext4.log`、`eintr-big-punch-tmpfs.log`）。
- 「3 处对不上」今天是 5 处：D18 的 314 → 317、323 → 326、184 → 185（与判决写的 317、326、185 相同，第 184 行那句今天在 157 与 185 两处），多出的 2 处在 `.claude/singlefs-ai-sop/rules/command-safety.md`（辩方引的 147 → 150、攻方引的 155 → 158）。
  该规则文件的修改时间早于 r2 判决文件、晚于辩方报告，我不知道判决当时跑 cite-check 的实际输出，所以不能判是判决漏记还是之后才挪的：**复核不了**；两处引的句子都在、只是行号后移 3，不影响判决的结论。
- 判决写的「快照里 sha256 前缀 `e09ab08b`，现在 `9781aed2`」：`e09ab08b` 是 `research/prompts/m3-code-r2-snapshot/kb-sha256.txt` 第 1 行的值（逐字在）；`9781aed2` 当时没存档，今天 D18 的 sha 已是 `a83e6430`（r3 快照 `kb-sha256.txt` 记的也是它），**复核不了**。
- 一条依赖要写明：判决第 5 行「E1 放得过全部用例」只有组合那一次跑（`wrong/` 下没有 E1 单独的目录）。推得出单独也放得过：E1 只在同一次写零连着被打断两次时才不同，基线里造得出这一形的只有「第一次被打断」的开关；但那是推的，不是单跑量的。

## 五、本机复现的内核事实（命令与原样输出）

`research/prompts/m3-code-r3-sonnet-model/fallocate_len0_probe.py`（sha256 前 8 位 `27a3944d`），镜像在草稿目录（`df -T` 是 ext4），跑完删除；输出存 `fallocate_len0_probe.out`（前 8 位 `a557ea16`），原样：

```
off=0 len=0 -> rc=-1 errno=22
off=512 len=0 -> rc=-1 errno=22
off=0 len=512 -> rc=0 errno=0
off=65536 len=512 -> rc=0 errno=0
off=0 len=131072 -> rc=0 errno=0
straddle(off=65024,len=1024) rc 0 size 65536 in-file half zeroed True head untouched True
```

长度 0 报 `EINVAL`（22）；起点在文件末尾、整段在文件外都交回 0；跨文件末尾的打洞交回 0、文件内那 512 字节被打成 0、文件长度不变、前面的字节不动。
这三条就是 r1 判决第 4、6 行和 J4 的 SAFETY 句（`block_device.rs` 第 392 行）所依赖的事实。本机 ext4 之外的文件系统没测。

## 六、冲突两句并排（整行）

只有三处两边不一样，其余各格一致。

**冲突 1：J3 判决字面与攻方 AC 的日志**

判决这一行（`research/prompts/m3-code-r2-main-verification.md` 第 38 行）：

```
- **J3**：三个入口各加一条 txg 27 转圈的用例，回退那条取「前一版」做目标；EC、AC 两种错实现必须红。
```

AC 原样下 txg 27 的那一行（攻方日志 `research/prompts/m3-code-r2-opus-model/logs/wrong/AC_only_region_zero_device_in_the_three_other_entries/y3-24..32-first.log` 第 16 行，整行）：

```
Y3 txg=27 region=0 slot=1 root_dev=DeviceIdentity(0) cut=BeforeItsRootSlotWrite entry=NormalUnmount outcome=behind-root(DeviceIdentity(0),AnotherRoot { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) },AnotherRoot { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }) unchanged=true checker=0 remount=Ok(InstanceGeneration(1),CheckpointTxg(27)) checker_after_remount=0 ms=5110
```

AC 下 txg 27 仍然拒成缺根，所以「AC 必须红」的用例在 txg 27 上写不出来。结论：判决字面要改，实现的做法（加 txg 25）对。

**冲突 2：r2 判决的 cite-check 计数与今天的计数**

判决这一行（`research/prompts/m3-code-r2-main-verification.md` 第 15 行，整行）：

```
- `cite-check.py`（主仓为根）：「核了 34 处引文，3 处对不上，0 处没判」。3 处都是辩方腿引 `.claude/kb/decisions/18-块里携带什么信息.md` 第 314、323、184 行，原文现在在第 317、326、185 行。这份 kb 在这一轮开工之后被别的会话改过：快照里 sha256 前缀 `e09ab08b`，现在 `9781aed2`，是删除定案的写回。引的句子逐字都在，只是行号挪了，不算引错。
```

今天同一条命令的末行（原样）：`  ✗ 核了 34 处引文，5 处对不上，0 处没判（逐处列在上面）`。多出的 2 处是 `command-safety.md` 的行号后移 3 行（第四节）。判决说的 3 处 D18 今天同样复现；说「3 处都是」今天不成立，原因复核不了。

**冲突 3：材料里的「98 个测试二进制」与日志**

材料这一行（`research/prompts/_m3-code-r3-body.md` 第 9 行，整行）：

```
合入后验证：harness 档 98 个测试二进制全绿、clippy 过，fmt 只红在原有 5 份没动过的文件（日志 `/tmp/claude-1000/merge-and-verify/2026-10-01-1042109/`）。
```

日志 `test.log` 的计数（第七节，原样）：`grep -c "^test result"` 得 98；`grep -c "Running "` 得 94、`grep -c "Doc-tests"` 得 4；passed 之和 807、failed 0。
98 是 94 个测试目标加 4 个文档测试的 `test result` 行数，不是 98 个二进制。数没错，名称不准；r1 判决里的 94 与这里的 94 是同一口径（`results=94`）。

## 七、合入验证日志的计数与实现员报告里的几处小数

命令与原样输出（日志 `/tmp/claude-1000/merge-and-verify/2026-10-01-1042109/`）：

```
$ grep -c "^test result" test.log
98
$ grep -c "Running " test.log; grep -c "Doc-tests" test.log
94
4
$ grep "^test result" test.log | awk passed/failed 之和
passed 807 failed 0
$ grep "^Diff in" fmt.log | 取文件名 | sort -u | wc -l
5
$ tail -1 clippy.log
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.08s
```

- 新用例都在这次合入验证里绿：`test.log` 里 `running 31 tests`（entries）、`running 139 tests`（核心层单测，与实现员基线的 139 条相同）、稀疏盘那份 `running 2 tests`；
  新增的 11 条（J1 一、J2 三、J3 六、k = 2 一）与改过的 H5 单测，名字各有 `... ok` 行（J3 六条的名字片段在 `test.log` 里命中 7 行，多的 1 行是 H2 原有的 txg 27 会话发布一条）。
- fmt 5 份就是实现员报告点名的那 5 份（报告正文写「4 份」而名单列了 5 份，是笔误）；clippy 末行 `Finished`，无错误行。
- 实现员报告的证红数：RG「30 passed; 1 failed」、全局不重读「25 passed; 6 failed」、EC「25 passed; 6 failed」、AC「28 passed; 3 failed」、Z0「138 passed; 1 failed」、E1「138 passed; 1 failed」，
  在 `/tmp/claude-1000/m3-r1/code-r2-fixes/prove-red-logs-harness/`、`prove-red-logs-core/` 的日志里都逐字找到，失败名单与报告所写一致。
  报告里「红 6 条，J2 三条加上已有的 H1 会话发布、H1 可写挂载、H3 三条」的「H3 三条」应读成「这三条」：红的 6 条是 J2 三条加 H1 会话发布、H1 可写挂载、H3 一条。
  另：`prove-red-logs-core/003.log`、`004.log` 与 `prove-red-logs-harness/003.log`、`004.log` 逐字节相同（`cmp` 无输出），是拷贝，核心层那一跑只有 `001`、`002` 两份真日志。
- 这些数全部来自实现员与主 agent 的跑，我没重跑编译与测试（任务要求只读与推导）。

## 八、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W1-① J1 | 一致（做到） | txg 8 会话发布一条，钉的是区域 2 归盘 0；RG 下只红这一条（日志 30 过 1 红） |
| W1-① J2 | 一致（做到）；观察一处 | 三个入口各一条、注入第 1 次读；抬 F 路径第二遍核盘（第 3 次读）没钉，判决字面不要求 |
| W1-① J3 | 一致（做到）；判决字面冲突一处 | AC 在 txg 27 红不了，判决字面写错；实现员加的 txg 25 三条是对的，多做 3 条 |
| W1-① J4 | 一致（做到）；Z0 稀疏盘行没证红；同类旧注释有残留 | H5 单测、稀疏盘 requests、SAFETY 句、model_comparison 第 780 行都对得上；`new_pool_file_creation_on_device.rs` 第 834、1251 行旧注释在判决点名之外 |
| W1-① J5 | 一致（做到） | 开关改成前 k 次，k = 1、2 各一条；E1 红 k = 2；重打至多 N（N ≥ 2）的错实现放得过（推的） |
| W1-① 变异 13 行 | 一致 | 锚点各命中 1 次、点名测试各 1 条 `fn`；RP、EC、AC 三种用单处等价写法，不是攻方原样 |
| W1-② 新用例钉的句子 | 一致 | J1–J5 每条用例钉的就是判决要钉的那句；钉的强度边界在上面各节「推的」处标了 |
| W1-③ RG、Z0、E1 | 一致（两边相同） | 变异行与攻方原样同一替换 |
| W1-③ RP、EC | 一致（新用例上结局相同） | RP 行是全局的，比原样宽；EC 行读盘次数不同；J2、J3 用例不数读次数，结局逐情形推过 |
| W1-③ AC | 新用例上相同；别的盘面不同 | 只在「盘 0 落后而 P 的根槽住盘 1」那一格原样多拒、过滤版不拒；今天没有用例分得出，推翻条件是 W2 在副本上跑出不同 |
| W1-④ r1 判决的数 | 一致；一处今天复现不出 | MUT 行、SwappedFor… 行、168、226、364 911（三份日志的和）、12 份 sha、84 份日志都对上；「主仓为根 88 对上 88」今天是 23 处行号漂移 |
| W1-④ r2 判决的数 | 一致；两处复核不了 | WRONG 各行、255 秒、915 秒、2 条抓到、216 / 108 格、2 304 / 16 次、17 份 sha、291 份日志都对上；「3 处对不上」今天是 5 处（多的 2 处在 command-safety.md），`9781aed2` 没存档 |
| 背景材料的数 | 数一致，名称不准 | 「98 个测试二进制」是 94 个目标加 4 个文档测试的 98 行 `test result` |

## 九、没做什么

- 没编译、没跑任何测试与变异（任务要求只读与推导）；所有「红 / 绿」的说法来自实现员与攻方的既有日志，或我对代码的推导，推导处都标了「推的」。
- 没套攻方原样的错实现、没写新的错实现、没在副本上证 Z0 稀疏盘那一行红：这是 W2 的事。W3（两块盘以外的几何、取样敏感度）按正文不归我。
- 没读 `research/prompts/m3-code-r3-opus-output.md`、`m3-code-r3-opus-model/`、`/tmp/claude-1000/m3-code-r3-opus/`。
- 没核 D18 写回后的内容（r2 判决第 10 行的「写回 D18 时一并写明」与 `/tmp` 镜像残留那笔欠账）：不在 J1–J5 里。
- 没核 r1、r2 判决里 kb 条款的实际文字（只核了产物行、复跑数与行号漂移）。
- 没替主 agent 采纳或出判决：冲突 1 的判决字面该怎么改、旧注释残留改不改、「3 处 / 5 处」怎么记，由主 agent 定。
- 打洞的内核行为我只在 ext4 上复现，没测 tmpfs 等别的文件系统。
