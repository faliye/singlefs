# 核查员报告：增补 3 第 3 件（崩溃注入）代码轮第一轮

**这是观测，不是判决**：核对表里的 ✗ 不免除主 agent 对推论本身的逐条现查。

时区：本报告全部时刻为 UTC；本机时钟即 UTC，人在 JST（UTC+9）。

## 0. 开工核对

- `sha256sum -c research/prompts/m2-supp3-item3-code-r1-start-snapshot.sha256`：12 个文件全部 `OK`（与当前主树逐字节相同）——**这一轮腿跑的时候快照没被改过**，因此下面「到快照里取那一行」与「到主树里取那一行」是同一件事，报告里不重复注明。
- 三份腿报告与四份翻译/运行附件均已交齐（`m2-supp3-item3-code-r1-opus-output.md` 430 行、`-sonnet-output.md` 238 行、`-local-attack.md` 168 行 + `-translation-audit.md` 36 行 + `-runlog.md` 30 行 + 两份干净样本 + 两份作废样本）。
- 未拿到云端腿交回时给的 sha256（派发提示未给），跳过这一步、按文件当前内容核。

## 1. 判别力自证（先做，必须判 ✗）

取一条待核引用：**sonnet 报告对 `crates/mutations.tsv` 第 180 行的引用**（报告原文第 61 行：`180:增补 3 第 3 件：判崩溃点时取择根……`）。

自证操作：把行号从 180 加 1 改成 181，按第 2 步的流程核——到快照副本（`/tmp/claude-1000/m2s3i3-r1-verifier/snapshot/crates/mutations.tsv`，与主树逐字节相同）里取第 181 行。

```
$ wc -l crates/mutations.tsv
180 crates/mutations.tsv
$ awk 'NR==181' /tmp/claude-1000/m2s3i3-r1-verifier/snapshot/crates/mutations.tsv
(空，文件只有 180 行)
```

第 181 行不存在（文件恰好 180 行，末尾有换行符、无多余空行），与引用声称的内容（那一整行变异表条目）逐字不符。**判 ✗**——核查方法能分辨行号错位。自证通过，进入正式核查。


## 2. 云端攻方腿（Opus，K1、K4）核对表

产物副本仍在 `/tmp/claude-1000/m2s3i3-r1-opus/`（本机未重启，全部文件与报告表里给的 sha256 逐一比对，见下）。

### 2.1 六份日志 + 两份量具的 sha256

```
$ cd /tmp/claude-1000/m2s3i3-r1-opus && sha256sum probe1.log ci-baseline.log ci-mutated.log \
  large80-baseline.log large80-mutated.log large200-baseline.log \
  model/zz_opus_k1k4_probe.rs model/zz_opus_k1_reorder.rs
0b8cd3d946f4fb92e968a8dfe50159ed15f91a3801506e71d6c8d24d102d9c97  probe1.log
f79c92f00be311b5f23e74a72b50a12c0f54e3803b3298ad0af77da7ddbf581e  ci-baseline.log
3b14d3a9fc6305b36fced29dc8e1b69d52424e09c62dd7e58aa2059d18a6b4b5  ci-mutated.log
5f9cc79eaee9c756b713949b12bd949c735da8d4d23e00bb178b04ccd9132683  large80-baseline.log
b51035aea3e376ed2fe199bef4591dfcf9e26882679c861e0bad2d8618d71fb2  large80-mutated.log
23d6800727881adb8eba696e39337597ab669558f48d5ba2e7b421153bbaf11f  large200-baseline.log
6b7f841b545fa8fac5813d52260f4cdba6129da42c8dd7441108d467e24504b2  model/zz_opus_k1k4_probe.rs
afe0d1af7b4d47e34f0816ccaf4162c4c46c9150417d559402e3dfb316987068  model/zz_opus_k1_reorder.rs
```
**全部 8 个值与报告第二节的表逐字相同** ⇒ ✓（产物未被事后改动）。

### 2.2 代码行号引用

| 引用 | 核的结果 | 命令/依据 |
|---|---|---|
| `milestone/02-second-txn.md:424`（验收标准整行） | ✓ 整行逐字相同 | `awk 'NR==424' .claude/kb/milestone/02-second-txn.md` |
| `crash_injection.rs:600`（屏障 filter） | ✓ 逐字相同 | `awk 'NR==600' crash_injection.rs` |
| `crash.rs:172`（`MemoryPool::apply` 跳过屏障） | ✓ 逐字相同 | `awk 'NR==172' crash.rs` |
| `crash_injection.rs:474`（`newly_persisted` 前缀切片） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:598-599`（候选区间下端） | ✓ 逐字相同（含 600 行的 filter 一并对上） | 同上 |
| `crash_injection.rs:427`（`after_the_last_finished_step` 赋值） | ✓ 逐字相同 | 同上 |
| `history.rs:2464`（`apply_cold_start_recover` 只读） | ✓ 逐字相同 | 同上 |
| `.../second_transaction_supplement_three_crash_injection.rs:246`（固定脚本注释，报告写成 `crash_injection.rs:246`，实指测试文件） | ✓ 逐字相同（行号真实指向测试文件而非同名 crate 文件，与报告脚注一致，未见混淆） | `awk 'NR==246' second_transaction_supplement_three_crash_injection.rs` |
| 同文件 `76-161`（`assert_every_crash_injection_path_was_exercised` 整段） | ✓ 表里 9 行断言逐行核，行号与断言内容全部对上（80-84/93-110/111-125/126-129/130-133/134-142/143-146/147-154/155-160） | 逐段 `sed -n` 核对 |
| 同文件 `28-31`（`UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES`） | ✓ 逐字相同 | 同上 |
| 同文件 `314-317`（`SINGLEFS_CRASH_INJECTION_DEVICES` 分支） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:499`（`recover(&image, Consult)`） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:511-519`（`crash_recovery_disagreement` 与两个计数分支） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:521`（`checker_violations_on`） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:657-659`（`crash_state_observation` 文档注） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:676`（`raised_floor_lands_only_on_abandoned_roots: None,`） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:7`（模块文档「冷启动都在流里」） | ✓ 逐字相同 | 同上 |
| `crash.rs:438-443`（`RecordCheck` 结构体，整段抄） | ✓ 逐字逐行相同 | 同上 |
| `crash.rs:793-794`（Consult/Ignore 各一遍） | ✓ 逐字相同 | 同上 |
| `crash.rs:813-844`（oracle_violation_for_versions 判定块） | ✓ 起止两端核对，内容对得上 | 同上 |
| `crash.rs:845-865`（`check_pool_image` 循环） | ✓ 起止两端核对 | 同上 |
| `crash.rs:866-872`（`check_records` 调用与两个计数） | ✓ 逐字相同 | 同上 |
| `crash.rs:1035-1052`（`persisted_writes_of_state`） | ✓ 整个函数体起止与引用范围恰好吻合 | 同上 |
| `crash.rs:306-343`（`writes_and_segments`） | ✓ 整个函数体起止与引用范围恰好吻合 | 同上 |
| `crash.rs:245-282`（`CrashImage` 一带，未逐字引用、只作机制指认） | ✓ 范围内确有 `CrashImage` 结构体与相关方法，无逐字引用因此不适用「整行抄」判据 | 同上 |
| `crash.rs:333-335`（FUA 段关闭，报告称「先 `current.push(writes.len()-1)` 再关段」） | **✗ 行号偏移一行**：`current.push(writes.len() - 1)` 实际在第 **332** 行，引用区间 333-335 只覆盖 `if...{ segments.push(...) }` 那三行，不含被引用的 push 语句本身 | `awk 'NR==328,NR==337' crash.rs`（见下方原样输出） |
| `segments.rs:73`（FUA 注释） | ✓ 逐字相同 | 同上 |
| `root_ring.rs:35`（`region = txg % 3` 的字面代码） | ✓ 逐字相同（`region: checkpoint_txg.0 % ROOT_RING_REGIONS`，`ROOT_RING_REGIONS=3`） | 同上 |
| `.claude/gate.d/54-layer0-replay.sh:2`（gate-stage 注释） | ✓ 逐字相同 | 同上 |
| `.claude/gate.d/54-layer0-replay.sh:87`（整行抄） | ✓ 逐字相同 | 同上 |
| `.../second_transaction_step_zero_layer0.rs:353-356`（钉住两条记录核对器判据为 0） | ✓ 范围对应 `assert_eq!` 整个元组，报告用「钉着 `record_root_without_record == 0`」做paraphrase、非逐字引用，语义相符 | 同上 |
| `crates/mutations.tsv` 第 67 行 | ✓ 存在，内容与「步 3：零单元发布在记录与根之间少一道屏障」逐字对应 | `awk 'NR==67' crates/mutations.tsv` |
| `D13（验证路线） 已定项 4`（`decisions/13-验证路线.md:71`，69-81 整段抄） | ✓ 整段逐字相同 | `sed -n '69,81p' decisions/13-验证路线.md` |
| `D23（journal的角色与格式） 已定项 14`（`decisions/23-journal的角色与格式.md:354`，整行抄） | ✓ 整行逐字相同 | `awk 'NR==354'` |

`crash.rs:333-335` 原样输出（证 ✗）：
```
$ awk 'NR==328,NR==337' crates/singlefs-harness/src/crash.rs
328:                         .contents
...
332:                 current.push(writes.len() - 1);
333:                 if retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess {
334:                     segments.push(std::mem::take(&mut current));
335:                 }
336:             }
337:         }
```

### 2.3 数字与复跑（主 agent 点名重点核的几样）

| 数字 | 核的结果 | 命令/依据 |
|---|---|---|
| K1-a：屏障变异下「5 条用例全绿」 | ✓ `ci-mutated.log` 末行 `test result: ok. 5 passed; 0 failed; 1 ignored`，sha256 已核对上 | `tail -3 ci-mutated.log` |
| K1-a：报告称「与基线的输出逐字比只差两行里的两个流下标」 | ✓ **不是「逐字相同」**：`diff ci-baseline.log ci-mutated.log` 除去构建耗时与 `CRASH_INJECTION_PROGRESS` 到达顺序两类非确定性行外，正文只有两处 `(种子,流下标)` 数值不同，与报告原文「只差两行里的两个流下标」一致（主 agent 派发提示里的转述「报告与基线逐字相同」比 opus 原文更强，是派发提示自己的简化，不是 opus 报告的问题） | `diff ci-baseline.log ci-mutated.log`（见下方原样输出） |
| K1-a：diff 片段里的具体数值 `[(11, 519), (16, 493)]` / `[(11, 517), (16, 491)]` | ✓ 内容逐字相同 | 同上 |
| K1-a：diff 头「`51c51`」 | **✗ 行号对不上**：对这两份已存盘、sha256 已核对的日志重新 `diff`，该处差异实际在 **`79c80`**，不是 `51c51`（`ci-mutated.log` 比 `ci-baseline.log` 多一行 `Compiling singlefs-core …`，从第 2 行起整体错位 1 行，`51` 与实际的 `79` 差距远超这 1 行的错位，来源不明） | `diff ci-baseline.log ci-mutated.log`；`grep -n "2 个崩溃点；前几个 (种子, 流下标) \[(11" ci-baseline.log ci-mutated.log` → `79`/`80` |
| K1-b：同段镜像对「2782 对（journal_record 429、unit_write 2353）」 | ✓ **独立复现**：报告未把这段「原样输出」存进任何一份带 sha256 的日志文件，`/tmp/claude-1000/m2s3i3-r1-opus/` 下找不到这段文本；把两份量具拷进本核查员自己的草稿副本（`/tmp/claude-1000/m2s3i3-r1-verifier/repo`，`rsync` 自主工作区，非 opus 的原目录）重新编译重跑，得到逐字相同的输出 | 见下方原样输出（`nice -n 19 cargo test --release -p singlefs-harness --test zz_opus_k1_reorder -- --nocapture`，本核查员自己的副本） |
| K1-c：层 0 枚举域 46 090 683 / 前缀 7243 / 排除 627(7219 中) | ✓ `probe1.log` 末行 `TOTALS writes=7219 candidates=6592 excluded_by_start=627 excluded_by_failed_step=0 layer0_states=46090683 prefix_states=7243`，与 sha256 核对过的文件逐字相同 | `tail -3 probe1.log` |
| K1-c：实抽「92 个（截在发布中间 77、截回过去 85）」 | ✓ `ci-baseline.log:35`：`崩溃点 92 个：截在发布中间（根还没落盘）的 77 个、截回到最后一版之前的 85 个`，逐字相同 | `grep -n "个崩溃点" ci-baseline.log` |
| K1-c：种子 22「candidates=0」 | ✓ `probe1.log` 第 22 行数据列第 8 列（`candidates`）确为 0 | `awk 'NR==22' probe1.log` |
| K1-d：大档起点段「5533/101820」+「EXTRA」整行 | ✓ **独立复现**（这段同样没有存进任何带 sha256 的日志）：本核查员在自己的副本上以报告给的环境变量重跑，输出逐字相同 | `PROBE_SEEDS=200 PROBE_STEPS=40 PROBE_POINTS=8 nice -n 19 cargo test --release -p singlefs-harness --test zz_opus_k1k4_probe -- --nocapture k1k4_probe`（见下方原样输出） |
| K1-f/g：大档 200 段「新发现 3」、种子 80、`1310720 − 1212416 = 98304 = 6×16384` | ✓ `large200-baseline.log` 逐行核对：`崩溃状态：以已知红收尾 {0: 261}、新发现 3`；`崩溃点上的新发现 … 种子 80，截在录制流第 386 步`；`记账的已分配 Some(1310720)，遍历全部有效根得到 1212416`，算术 1310720−1212416=98304、98304/16384=6，均核对无误 | `grep -n "新发现\|种子 80\|1310720" large200-baseline.log` |
| K1-f：与收口表第 43 行「差 6 × 16384」「快档种子 80」逐项对得上 | ✓ `.claude/kb/milestone/02-second-txn.md` 第 360 行原文含「差 6 × 16384」与「快档种子 80」两处字样，逐字吻合 | `grep -n "^| 43 " .claude/kb/milestone/02-second-txn.md` |
| K4-4：层 0 每状态跑法（`gate.d/54-layer0-replay.sh:87` 整行） | ✓ 已在 2.2 节核过 | 同上 |

原样输出（K1-a 的 diff，本核查员重新对 opus 已存盘的日志跑的，未改动这两份文件）：
```
$ diff ci-baseline.log ci-mutated.log | sed -n '1,3p'
0a1
>    Compiling singlefs-core v0.1.0 (/tmp/claude-1000/m2s3i3-r1-opus/repo/crates/singlefs-core)
2c3
$ grep -n "2 个崩溃点；前几个 (种子, 流下标) \[(11" ci-baseline.log ci-mutated.log
ci-baseline.log:79:崩溃状态上的已知红第 0 条（…）：2 个崩溃点；前几个 (种子, 流下标) [(11, 519), (16, 493)]
ci-mutated.log:80:崩溃状态上的已知红第 0 条（…）：2 个崩溃点；前几个 (种子, 流下标) [(11, 517), (16, 491)]
```

原样输出（K1-b，本核查员在自己的副本 `/tmp/claude-1000/m2s3i3-r1-verifier/repo` 独立重跑）：
```
$ nice -n 19 cargo test --release -p singlefs-harness --test zz_opus_k1_reorder -- --nocapture
MIRROR_PAIR segment=6 segment_len=2 first_copy=write#13(dev DeviceIdentity(0)) second_copy=write#14(dev DeviceIdentity(1))
NOT_A_PREFIX last_persisted=write#14 holes=1 first_hole=write#13
RECOVER_ON_REORDERED_STATE outcome="NoFile" effective_root=Some((InstanceGeneration(0), CheckpointTxg(0))) prefix_applied=0
DOMAINS writes=129 segments=31 layer0_states=789588 prefix_states=130
test a_mirror_pair_out_of_order_is_a_layer0_state_no_prefix_can_reach ... ok
FAST_TIER_MIRROR_PAIRS total=2782 by_kind={"journal_record": 429, "unit_write": 2353}
test how_many_mirror_pairs_the_fast_tier_leaves_unreachable ... ok
```

原样输出（K1-d，本核查员独立重跑）：
```
$ PROBE_SEEDS=200 PROBE_STEPS=40 PROBE_POINTS=8 nice -n 19 cargo test --release -p singlefs-harness \
  --test zz_opus_k1k4_probe -- --nocapture k1k4_probe
EXTRA zero_candidate_histories=0 not_completed=0 just_before_root_candidates=5829 points_per_history=8 expected_just_before_root_draws=0.48
TOTALS writes=101820 candidates=96287 excluded_by_start=5533 excluded_by_failed_step=0 layer0_states=594980354 prefix_states=102020
test k1k4_probe ... ok
```

**观测（不算 ✗，单列）**：报告第二节的产物 sha256 表只登记了 6 份日志 + 2 份量具，但报告正文引用的「原样输出」里有两段（K1-b 的 `MIRROR_PAIR`/`FAST_TIER_MIRROR_PAIRS`，K1-d 的大档 `EXTRA`/`TOTALS`）在 `/tmp/claude-1000/m2s3i3-r1-opus/` 下找不到对应的落盘文件——报告自己的证据表没有完全覆盖它引用的全部「已跑过」的输出。本核查员改用独立重跑核实了这两段内容为真（见上方原样输出），但这是核查员另起的验证路径，不是「回读 opus 自己留的产物」。

### 2.4 计数

- 核了 43 处（代码行号引用 33 处 + 数字/复跑 10 处，含 sha256 校验、diff 复核、两次独立重跑）。
- ✓ 40 处（含 2 处独立重跑坐实、1 处「与基线逐字相同」的转述澄清）。
- ✗ 2 处（`crash.rs:333-335` 行号偏移一行；K1-a diff 头 `51c51` 与实际 `79c80` 不符，内容不受影响）。
- 不算 ✓ 也不算 ✗ 单列 1 处（K1-b/K1-d 两段输出未存进 sha256 覆盖的产物清单，已用独立重跑补齐）。

## 3. 云端正推腿（Sonnet，K2、K3）核对表

| 引用 | 核的结果 | 命令/依据 |
|---|---|---|
| `model_comparison.rs:295-312`（`observed_read_back_after_a_crash` 整段） | ✓ 内容逐字相同，但报告把源码里跨行的结构体字面量（`ObservedReadBack::Failed { what: ... }` 等）压成单行——语义与全部 token 未丢，只是排版重排，非摘句 | `awk 'NR==293,NR==320' model_comparison.rs` |
| `recovery.rs:451-457`（`rollback_high_water_of_root` 整段） | ✓ 逐字逐行相同 | 同上 |
| `recovery.rs:1336`（`recover` 签名）、`:1337`（`mapping_fallbacks`）、`:1378`（`walk_to_file` 调用） | ✓ 均逐字相同 | 同上 |
| `recovery.rs:422-429`（`instance_table_of_root`） | ✓ 逐字相同 | 同上 |
| `recovery.rs:1363`（`root_key` 赋值） | ✓ 逐字相同 | 同上 |
| `D13（验证路线） 已定项 7`（`decisions/13-验证路线.md:130`，整行抄） | ✓ 整行逐字相同 | `awk 'NR==130'` |
| `mount.rs:952-976`（`instance_rows_to_write`） | ✓ 引用片段（`let first_row_instance = ...` 三行）逐字相同 | `awk 'NR==950,NR==978' mount.rs` |
| `mount.rs:1287-1292`（`PreviousInstanceRow` 构造） | **✗ 引用块混入未标注的自撰批注**：源码 1287-1292 行没有任何行内注释，报告在 `instance: target.instance,` 后面加了 `// = R_old 的实例号，不是新实例自己的号`，未注明这是引用者自己加的说明，容易被读成源码自带的注释 | `awk 'NR==1287,NR==1292' mount.rs`（见下方原样输出） |
| `mount.rs:923-948`（`warm_up_publish_txgs`） | ✓ 函数体起止与引用范围恰好吻合，`931` 行 `vec![device_of_txg(row_publish_txg)]` 指认准确 | 同上 |
| `mount.rs:676-706`（`raise_rollback_floor` 空发布循环） | ✓ 用 `...` 显式省略了 `PublishPlan` 结构体字面量，其余逐字相同 | 同上 |
| `mount.rs:609`（`raise_rollback_floor` 签名） | ✓ 逐字相同 | 同上 |
| `history.rs:268-284`（`HistoryOperation` 枚举，报告称含全部 7 个成员含 `ColdStartRecover`） | **✗ 行号范围少了 2-3 行**：`ColdStartRecover` 实际在第 **286** 行、枚举闭合的 `}` 在第 **287** 行，均在引用区间 268-284 之外；区间起点 268 也落在上一个结构体的收尾 `}` 上，不是这个枚举的开头（枚举本体是 271-287） | `awk 'NR==266,NR==287' history.rs`（见下方原样输出） |
| `model.rs:665-680`（`committed_versions` 文档注 + 实现） | ✓ 逐字逐行相同（主 agent 派发提示写的引用点是 `665-668`，是派发提示自己的缩写，sonnet 实际引用范围是 665-680，与源码完整对应） | `awk 'NR==663,NR==682' model.rs` |
| `crates/singlefs-format/src/lib.rs:185-186`（`ROOT_RING_REGIONS=3`、`ROOT_RING_SLOTS_PER_REGION=8`） | ✓ 逐字相同 | `awk 'NR==185,NR==186' crates/singlefs-format/src/lib.rs` |
| `history.rs:2635`（`execute_history_with` 签名） | ✓ 逐字相同 | 同上 |
| `history.rs:2750-2762`（失败提前 `return`、`observer` 调用） | ✓ 逐字相同 | 同上 |
| `history.rs:2756`（`observer(&StepObservation{...})`） | ✓ 逐字相同 | 同上 |
| `crash_injection.rs:418-421`（`committed_versions.insert` 回调） | ✓ 逐字相同 | 同上 |
| `D23（journal的角色与格式） 已定项 14`（`decisions/23-journal的角色与格式.md:376`） | **软 ✗（技术性，非实质性）**：报告用「……」省略了原文中段（`research/prompts/m2-step45-code-r1-main-verification.md` 脚注、C340 的 P1/P2 两种取法的完整描述），不算整行抄；但被省略段落里的关键短语「比 R_old 新的根全坏时落回 R_old、同实例连号的被抛弃记录整段重放」在同一份报告第 111 行又被整段逐字引用了一次，未见内容失真 | `awk 'NR==376' decisions/23-journal的角色与格式.md` 对比报告第 105-111 行 |
| `crates/mutations.tsv:180`（整行抄，判别力自证已用过） | ✓ 逐字相同 | 见第 1 节 |

原样输出（`mount.rs:1287-1292`，证 ✗）：
```
$ awk 'NR==1287,NR==1292' crates/singlefs-core/src/mount.rs
    let previous_row = PreviousInstanceRow {
        instance: target.instance,
        selected_root_txg: target.checkpoint_txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    };
```
（源码这几行没有任何行内注释；报告原文在 `instance: target.instance,` 后接了 `// = R_old 的实例号，不是新实例自己的号`。）

原样输出（`history.rs:268-284`，证 ✗）：
```
$ awk 'NR==266,NR==287' crates/singlefs-harness/src/history.rs
266:    pub steps_above_current_floor: u64,
267:}
268:
269:
270:/// 历史里的一步。
271:#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
272:pub enum HistoryOperation {
273:    /// ...
274:    PublishFirstFile(ContentChoice),
...
284:    RaiseRollbackFloor(FloorTargetChoice),
285:    /// 关掉这个进程的可写会话，冷启动 `recovery::recover`（看 journal）。
286:    ColdStartRecover,
287:}
```

### 3.1 计数

- 核了 20 处。
- ✓ 17 处（含 1 处排版重排、1 处显式省略号，均不影响内容）。
- ✗ 2 处（`mount.rs:1287-1292` 混入未标注批注；`history.rs:268-284` 范围少了 `ColdStartRecover` 与闭合括号）。
- 软 ✗ 单列 1 处（`decisions/23-...md:376` 用省略号截断整行抄，但关键内容在报告别处已整段引用，未实质失真）。

## 4. 本地攻方腿（K5、K6）核对表

### 4.1 翻译核对表逐条现查（原文文件:行 vs 提示英文）

| 英文项 | 声称的原文文件:行 | 核的结果 |
|---|---|---|
| Q1 | `crash_injection.rs:732-736` | ✓ 行号与内容都对上；「in the order slices actually finish」这一处多出来的限定词，核对表已自陈来源（同段末句「按到达次序打」），未见凭空添加 |
| Q2 | `crash_injection.rs:657-659` | ✓（与 2.2 节 opus 引用的同一处逐字核对一致） |
| Q3 | `history.rs:1229` | ✓ 逐字相同 |
| Q4 | `history.rs:1262` | ✓ 逐字相同（`R × S = 24` 等具体数字与括注全部保留） |
| Q5 | `history.rs:2531` | ✓ 逐字相同 |
| Q6 | `history.rs:1267` | ✓ 逐字相同 |
| Q7 | `.claude/kb/milestone/02-second-txn.md:311` | ✓ 该行确为收口表第 ② 行，内嵌句「代码三方第一轮攻方腿……种子 61、81……13 个槽……2 个……只数了槽、没核机理」与英文译文逐句对应；「（`research/prompts/m2-supp3-item1-implementer-report.md`第十三节）」脚注被删，核对表已自陈理由 |
| Q8 | `history.rs:1257-1258` | ✓ 逐字相同；「②」圆圈数字记号被译成阿拉伯数字「2」，核对表已自陈理由 |
| F12 | `history.rs:1184`（字段声明在 1185） | ✓ doc 注原文「与它报的第一处」确有歧义，核对表把首稿「together with the first detail message it reported for each」（隐含去重）改成不含 first 的中性译法，并现查 `history.rs:2516-2528`、`walk.rs:1500-1501` 佐证「可能推入不止一条」——现查属实（见下） |
| F14 | `history.rs:1187` | ✓ 逐字相同 |
| F15 | `history.rs:1189` | ✓ 逐字相同 |
| F16 | `history.rs:1191` | ✓ 原文仅两个名词，译文加了 "mismatch" 两次，核对表已自陈并给出 `HarnessJudgement` 两个成员名字面依据（`AllocationGenerationIsNotThePublishTxg`、`ColdStartRecoveryFailedOnCheckerGreenImage`），现查属实 |
| F17 | `history.rs:1193` | ✓ 逐字相同；「（第 2 件）」脚注被删，核对表已自陈理由 |
| F18 | `history.rs:1195-1196` | ✓ 逐字相同 |
| F1/F2（`SLICES_PER_WORKER_THREAD`、`seed_slices`、`spawned_worker_threads` 算式） | `crash_injection.rs:46,853-865,751` | ✓ 现查 `wanted = worker_threads.max(1) * 4`、`slice_count = wanted.min(seed_count)`、`spawned_worker_threads = worker_threads.count().min(slices.len().max(1))`，与 F1/F2 文字描述逐项对应 |
| F19-F24（`root_ring_has_turned`、两条匹配函数、`classify_failure`） | `history.rs:1230-1256,2533-2543` | ✓ 五个函数体逐字对照，条件顺序、`&&` 关系、`.position()` 语义全部对应 |
| F25（I-3.1 detail 模板三个空） | `crates/singlefs-checker/src/walk.rs:1500-1501` | ✓ `format!("盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}")`，三个空（device/allocated/walked）与事实描述一致 |

F12 佐证的现查命令：
```
$ sed -n '2516,2528p' crates/singlefs-harness/src/history.rs | grep -n "violations.push\|invariant"
$ sed -n '1498,1502p' crates/singlefs-checker/src/walk.rs
```
（结果与核对表引用一致：`violations_on` 按不变量逐条 `push`，不去重；I-3.1 的判定在 `walk.rs` 按设备各调用一次 `judge`。）

### 4.2 干净样本判定复核

| 检查点 | 声称 | 核的结果 |
|---|---|---|
| void1 判红原因 | `corruption-check.py` 判红：英文复读 7 处，样本 `not checked | not checked | ...` | ✓ 独立重跑 `corruption-check.py` 于 void1 拷贝：`红 ... 英文复读=7(9.16/千)`，样本文字逐字相同 |
| void2 判红原因 | 判的是**提示文件自己**的 `::` 粘连=8，模型输出本身判绿 | ✓ 独立重跑：`corruption-check.py` 对 void2 输出本身判绿（`words=1189 ... 全 0`），对当时的提示文件判红；当前提示文件（已改掉全部 `::`）`grep -c '::' local-attack.md` = 0，与「已改掉 8 处」的说法一致 |
| s1/s2 `oov-check.py` 判红 | 生词=36、拼接=1（`FailureObservation(=failure+observation)`） | ✓ 独立重跑，s1、s2、（以及 void2 的模型输出部分）三次结果逐字相同：`生词=36 拼接=1`，生词表 `overturned RaiseRollbackFloor FailureObservation` |
| 「`FailureObservation` 是提示自带的类型名，不算损坏」 | 人工归类为干净 | ✓ 现查：`FailureObservation` 一词在提示正文出现 5 次（Section 6 标题、F24、Section 8、Section 9 各一次），且与 `crates/singlefs-harness/src/history.rs:1181` 的 `pub struct FailureObservation` 字面拼写逐字节相同，不缺头不多字母；`RaiseRollbackFloor` 同样是提示 Fact F11 给出的真实枚举成员名，`overturned` 是提示自己要求的收尾短语用词。三个词均现查过来源，判定不带损坏成立 |
| 「四次调用已达停止条件」 | 两次判红作废、两次退出码 0 且人工复核过 | ✓ 运行记录汇总表与本核查员独立重跑的判定一致（void1/void2 作废、s1/s2 干净），四次记录完整、无遗漏 |

## 5. 主 agent 点名的第 4 项：背景材料「58 次」对不对

背景材料 `_m2-supp3-item3-code-r1-background.md:19`：「崩溃点上 checker 跑了 92 次、问模型 92 次（比过内容 **58** 次、树表 0 条对上 31 次）」。

现查两条独立依据：
1. `.claude/kb/milestone/02-second-txn.md:388`（同一段现状叙述）原文写「恢复读回文件 **61** 次、没有文件 31 次、失败 0 次……checker 跑 92 次、问模型 92 次」——kb 自己的记述就是 61，不是 58。
2. `crash_injection.rs:512-519` 的计数逻辑：`disagreement.is_none()` 时按 `read_back` 成员分别给 `model_contents_compared`（对应「比过内容」）与 `model_versions_without_a_file_matched`（对应「树表 0 条对上」）各加一次；opus 在自己副本上重跑 `ci-baseline.log` 得到的原始计数是 `recoveries_reading_a_file=61`、`recoveries_without_a_file=31`，零违例。

61 + 31 = 92，58 + 31 = 89 ≠ 92。**结论：61 对，背景材料的「58」是错的**——两条独立依据（kb 现状叙述、opus 副本重跑）互相印证，都指向 61，不是 58。这处应改的是背景材料，不是 kb 或代码。

## 6. 没做什么

- 不判 K1-K6 各条推论打中成不成立、该不该采纳，也不判「大档 200 段新发现是不是真 bug」——那是主 agent 与三方判决的事，本报告只核观测。
- 不核 opus/sonnet/本地腿的攻击面分工划得对不对、是否有遗漏，只核已经写出来的引用与数字。
- 未拿到云端腿交回时给的 sha256（派发提示未提供），本报告全程按文件当前内容核对，未核对「文件是否与交回时刻完全一致」这一层。
- 未重编译验证 opus 报告里「四条改法」的可行性与代价（报告自己也标注这些是「推的」「被攻过零轮」），不在核查员职责内。
- 未复跑 `crates/mutations.tsv` 第 67 行、第 180 行两条变异本身是否真的会让对应测试判红（两条腿的报告都已注明「未实际复跑 cargo mutants」），本报告同样未跑——这类改动 `crates/` 源码的复跑不在核查员写范围内。
- s1、s2 两份本地攻方样本在 Q16 上给出相反答案（s1「no」、s2「yes」，而两份样本给出的 S5/S6 具体数值本身完全一致），这是内容/推理层面的不一致，不属于本报告核查的引用/产物/复跑范畴，仅在此提示主 agent 留意，不作为 ✗ 计入任何一条腿的核对表。
- 未核对 D13、D23 之外别的决策文件是否也被这一轮引用（背景材料与附录里是否还有别处摘句），只核了主 agent点名与报告正文实际出现的引用。

## 7. 全局计数

- Opus 腿：核 43 处，✓ 40，✗ 2，单列（未存入产物但已独立复现）1。
- Sonnet 腿：核 20 处，✓ 17，✗ 2，软✗单列 1。
- 本地攻方腿（含翻译核对表、样本判定）：核 22 处，✓ 22，✗ 0。
- 背景材料数字争议：现查坐实「61 对，58 错」。
- 判别力自证：1 次，按要求判 ✗，方法有效。

（全文完）
