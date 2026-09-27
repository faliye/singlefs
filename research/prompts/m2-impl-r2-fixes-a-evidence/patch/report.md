# 第二轮改法 A（checker 与 litmus 那一半）实现员报告

写于 2026-09-27（JST 夜间；本机时钟 UTC）。规格 `/tmp/claude-1000/impl-r2-fixes-a/spec.md`；里程碑二收尾第四步。四件都做完，另有三处越出派发清单或要主 agent 定的，见「停下交主 agent 的」。

## 结论

| # | 关的是 | 结果 |
|---|---|---|
| 1 | Y4-a：I-7.13 与 core 收同一张表 | `geometry_of` 加四判：325 = `JOURNAL_RING_START_SLOT`、371 = `ROOT_RING_BASE_SLOT`（两个起点先于槽距判）、417 = 环起点 + ⌈环长 ÷ 16384⌉、417 是 `CLUSTER_SEGMENT_SLOTS` 的整数倍；四个新 `Verdict` 成员都归进 `value_refused_by_this_reader`（整池不作保）。常量从 `singlefs-format` 取，算式 checker 自己写（94 号绿）。「槽距上界改用同一槽自述的根环起点」：HEAD 上 `fixed_structure_slot_spacing_lies_in_the_format_range(slot_spacing, base_slot)` 已经用的是偏移 371 的字段，这一格没有要改的；现在 371 先被判成等于 64，槽距上界实际就是 1 MiB − 4096 |
| 2 | Y2-b 越格线索 | 新不变量暂立 I-7.14（`SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY`，`IMPLEMENTED_INVARIANTS` 48 → 49，改号只改 `image.rs` 那一处常量）；判法：每块盘两槽里自证过、fsid 属于本池的系统配置槽，本盘设备号 = `ImageReader::devices()` 给的这块盘的身份，每槽判一次 |
| 3 | C577 ②：litmus「轮换之后屏障」 | `litmus/publish-returns-after-the-rotation-barrier.litmus`（Never）与 `-nofence`（Sometimes）。锚点：写者 `transaction.rs::persist_the_root_then_rotate_the_system_configuration`，读者 `recovery.rs::verified_system_configuration_slots`。**herd7 没跑**（重型，归提交时 57 号；我起 `lkmm.sh --static-only` 也被重型闸拒了），静态那四条照 `lkmm.sh` 的判法手核过（见「litmus 静态核」） |
| 4 | C577 ③：零单元发布轮换后屏障报错 | 新用例钉住今天的结局：`MountError::Publish`，原因 `PublishError::BlockDevice`，写账：已落盘的发布 0 份、失败的 1 份；盘上落到盘 0 那道屏障为止（逐步钉住）；根（实例 1，txg 1）读得到；换回好设备之后再可写挂载，取号 2、择到 (1,1) |

## 写过的文件（副本 `/tmp/claude-1000/impl-r2-fixes-a/copy/` 里改的；主工作区一个字没动）

- `crates/singlefs-checker/src/lib.rs`：`Verdict` 加四个成员
- `crates/singlefs-checker/src/image.rs`：I-7.13 四判、I-7.14 常量、清单与判定函数
- `crates/singlefs-checker/src/walk.rs`：import、调用 I-7.14 判定、一行注释
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：五份几何坏镜像、I-7.14 坏镜像（也进了 `known_bad_images`，所以 `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 罩得到它）、两条新测试
- `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`：**不在派发的「要动的 crates 文件」里**，改了一格期望，见「停下交主 agent 的」第 1 条
- 新建 `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs`（两条测试）
- 新建 `litmus/publish-returns-after-the-rotation-barrier.litmus`、`litmus/publish-returns-after-the-rotation-barrier-nofence.litmus`
- `crates/mutations.tsv`：追加 10 行（`patch/mutations-append.tsv`），整行替换 1 行（`patch/mutations-replacements.tsv`，名字「A3-checker-2 第 29 条：系统配置越界值退回「这一槽不可择」」：我改动的那段 match 让它原来的锚点命中 0 次，换成新锚点并重新证过红）

追加的 10 行变异名：
1. Y4-a I-7.13：checker 不判单元区起始槽号等于环长现算的那个槽（A 格 417 +64 漏过）
2. Y4-a I-7.13：checker 不判单元区起始槽号落在聚簇段边界上（D 格环长少一槽漏过）
3. Y4-a I-7.13：checker 不判 journal 环起点等于第一版常量 1024（B 格 325 写 1023 报成单元区起点）
4. Y4-a I-7.13：checker 不判根环起点等于第一版常量 64（C 格 371 写 65 漏过、E 格报成槽距）
5. Y4-a I-7.13：几何那四个新成员不认成「读者不收的值」、退成这一槽不可择（别的槽照判、照作保）
6. Y2-b 越格线索 I-7.14：本盘设备号与盘在池里的身份不比（判定恒真，盘体对调 0 违例）
7. Y2-b 越格线索 I-7.14：池级走读不调这一判（干净镜像上 I-7.14 报不适用、不报成立）
8. I-7.14 坏镜像（checker_known_bad_images）：本盘设备号照原样写回，坏镜像退成干净镜像
9. C577 结清 ③：零单元发布落盘失败时原样重发一次（冻结再重发的形态），轮换之后那道屏障报错一次时挂载照常做成（改的是 `crates/singlefs-core/src/transaction.rs`，锚点在主工作区现版本里恰好命中 1 次，已核）
10. C577 结清 ②：litmus「轮换之后屏障」的 P0 去掉轮换之后那道屏障，与录制流里发布最后三步对不上

补丁：`/tmp/claude-1000/impl-r2-fixes-a/patch/`（`crates.patch`、`mutations-append.tsv`、`mutations-replacements.tsv`、`report.md`）。`crates.patch` 是「主工作区现版本 + 我的改动」拼出来的，因为派发之后别的会话改了主工作区的 `checker_known_bad_images.rs`（`singlefs_harness::crash` → `singlefs_harness::memory_pool`，4 行），`checker_judges_reserved_bytes…rs` 同样有这类改动。对主工作区 `git apply --check -v` 八个文件都过（下面有原样输出）。

## 补丁核对（原样输出）

`git apply --stat patch/crates.patch`（在主工作区）：
```
 crates/singlefs-checker/src/image.rs               |   90 +++++-
 crates/singlefs-checker/src/lib.rs                 |   13 +
 crates/singlefs-checker/src/walk.rs                |   17 +
 ...tation_barrier_fails_returns_the_mount_error.rs |  319 ++++++++++++++++++++
 ...es_reserved_bytes_and_widths_it_used_to_skip.rs |    4 
 .../tests/checker_known_bad_images.rs              |  201 +++++++++++++
 ...turns-after-the-rotation-barrier-nofence.litmus |   32 ++
 ...blish-returns-after-the-rotation-barrier.litmus |   41 +++
 8 files changed, 703 insertions(+), 14 deletions(-)
```

`git apply --check -v patch/crates.patch`（在主工作区）：
```
Checking patch crates/singlefs-checker/src/image.rs...
Checking patch crates/singlefs-checker/src/lib.rs...
Checking patch crates/singlefs-checker/src/walk.rs...
Checking patch crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs...
Checking patch crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs...
Checking patch crates/singlefs-harness/tests/checker_known_bad_images.rs...
Checking patch litmus/publish-returns-after-the-rotation-barrier-nofence.litmus...
Checking patch litmus/publish-returns-after-the-rotation-barrier.litmus...
exit=0
```

主工作区 `git diff --stat -- crates litmus` 末行（全是别的会话的未提交改动，我没写进主工作区；按定义附上）：
```
 118 files changed, 2204 insertions(+), 2269 deletions(-)
```

## 每条新测试的证红（`research/scripts/prove-red.sh --copy … --memory 8G singlefs-harness`，基线先跑、绿）

| 新测试 | 改坏哪一行（变异名序号） | 哪条断言红 |
|---|---|---|
| `checker_known_bad_images`：`each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_own_invariant_naming_the_refused_field` | 1–5（`image.rs` 第 262、265、281、288 行四判各拿掉一判；新成员划到 `false` 那一臂） | 1、2、4：I-7.13 该判违例的那一格判成别的（「I-7.13 要判违例，得到 …」那一句 panic）；3：B 格报成 `UnitAreaStartNotTheSlotAfterTheJournalRing`，「违例说明里要是这个成员」那条红；5：槽退成不可择、别的槽照判，I-7.13 不红 |
| `checker_known_bad_images`：`devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_number_invariant` | 6（`image.rs` 第 557 行判定写成 `true`） | 「只红 I-7.14」那条 `assert_eq!`（违例清单是空的） |
| `checker_known_bad_images`：`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`（已有测试，I-7.14 进了它的清单） | 7（`walk.rs` 第 5757 行起那一调用删掉）、8（坏镜像写回原值） | 7：干净镜像上 I-7.14 报不适用、不是成立；8：「I-7.14 那份坏镜像应当判违例」 |
| 新文件：`a_zero_unit_publish_whose_post_rotation_barrier_fails_is_not_frozen_and_the_whole_mount_returns_the_error` | 9（`transaction.rs` 零单元发布落盘失败时原样再落一次——冻结重发的形态） | 挂载做成了，走到「挂载不许做成（今天不冻结、不重发）」那句 panic |
| 新文件：`the_rotation_barrier_litmus_writer_follows_the_last_three_steps_of_every_recorded_publish` | 10（litmus 正例 P0 里轮换之后那行 `smp_wmb();` 删掉） | 「P0 要与录制流里发布最后三步逐项相同」 |

同时红了哪些：1–8 的变异行带 `-- <过滤名>`，只跑到点名那一条，二进制里其余 42 条被过滤、不知道；9、10 跑整个新二进制（2 条），各自只红点名那一条、另一条绿。证过的行：11 行全证（10 行追加 + 1 行替换），没有留给 59 号的。prove-red 原样末行：

```
✓ 点名 9 条：跑了 9 条，跳过 0 条，跑的都抓到了
✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
```
（第一次 9 行时 C577 ③ 那一行已在里面；第二次把它连同 litmus 那一行在新文件加了第二条测试之后重证；第三次是替换的第 1313 行。日志在 `prove-red-logs*/`。）

改之前的红绿（对着 HEAD 基线核过）：几何五格在改之前 A、D 只红 I-5.2、C 只红 I-7.1、B 0 违例、E 红 I-7.13 但报的是槽距成员（攻方 `logs/y4.log`，我没重跑这份攻方用例，是用变异 1–4 在同一份镜像上重现了「I-7.13 不红」）；盘体对调那份在改之前 0 违例（变异 6 的红就是这一格）。

## 第 4 步那几样（末尾原样输出）

- 测试二进制（副本、`--offline`、经 `run-with-memory-cap.sh 8G`、`capped.sh 4`）：
```
checker_known_bad_images: test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 82.35s
checker_judges_reserved_bytes_and_widths_it_used_to_skip: test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.99s
a_publish_returns_only_after_the_system_configuration_rotation_is_durable: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.33s
a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.42s
second_transaction_step_three_formatted_pool: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 72.88s
system_configuration_per_device_redundancy: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.93s
unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.32s
publish_order_matches_litmus: test result: FAILED. 0 passed; 1 failed; …（left: 35 right: 33，第一个事务步数；HEAD 上同样红，是 C577 加屏障之后那条测试没跟，规格里归改法 B）
```
后三份与 `publish_order_matches_litmus` 不是我改的文件，是 checker 判定变了之后顺手看有没有波及。
- `cargo fmt --all -- --check`：退出码 0，无输出。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 七条：`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.22s`，退出码 0。
- `cargo build --offline --all-targets`：`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 52.84s`，退出码 0。

## 登记给我的门禁阶段（在副本里跑：副本是 HEAD 的 `git archive` 加我的改动，规范副本 `.claude/singlefs-ai-sop/` 从主仓拷进去）

| 阶段 | 退出码 | 判读 |
|---|---|---|
| 33 | 1 | 剩 6 行锚点指 `crates/singlefs-checker-tier/tests/…`：那个包是别的会话在主工作区里正在挪的，HEAD 副本里没有，与这一件无关；我自己弄坏的第 1313 行已经换好锚点（`mutations-replacements.tsv`）。打补丁时 `apply-writer-patch.py` 会在主工作区跑一次 33 号，那时这 6 行的文件是在的 |
| 53 | 0 | 绿 |
| 74 | 1 | 随机历史那个二进制有 3 条红；**在不带我改动的 HEAD 副本上跑同一个二进制，红的是同样 3 条**（见下），不是这一件带进来的 |
| 89 | 77 | 本次未跑（它自己的清单问题，与这一件无关） |
| 92 | 77 | 副本不是 git 仓，本阶段跳过——**没判**，要在主工作区打上补丁之后跑 |
| 93 | 0 | 绿 |
| 94 | 0 | 绿 |

原样末行：
```
[33-mutation-tables]
  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）：
    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。
[53-format-const-placeholders]
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））
[89-closeout-row27-preconditions]
    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐
[92-layout-checker-sync]
  ! /tmp/claude-1000/impl-r2-fixes-a/copy 不是 git 仓，本阶段跳过
[93-feature-bits]
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 57 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 57 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））
[94-checker-implementation-disjoint]
  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 `singlefs_core`（别名引进来的 0 个）；共享模块 1 份源码的正文 286 行里没有分支与循环（`#[cfg(test)]` 标着的项 313 行不扫）
    这一道判不了的：两边各自手写的那份语义对不对（归模型对拍与变异表），以及 C12（增量语义共用） 另一半「运行时记账分支取反 ⇒ I-3.1（已分配统计对得上） 必须红」——那一条在 `crates/mutations.tsv` 里，门禁 59 号复跑
[74-model-differential]
test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 116.94s
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
exit=1
```

33 号列出的行（副本上）：
```
  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）：
      crates/mutations.tsv:798 实六 固定脚本：层 0 切段不带卸载入口段（C557 那一道在带卸载的流上当场断言停）：文件 crates/s
      crates/mutations.tsv:909 实审 B3b c561 计数行：评过的状态数少于闭式也打 exhaustive=true：文件 crates/singlefs-checker/tests/record_ch
      crates/mutations.tsv:910 实审 B3b c561 线程行：这一趟跑的片报得比总片数多一片（读回 + 跑的 ≠ 总片数）：文件 crates/sing
      crates/mutations.tsv:911 实审 B3b c561 计数行：闭式按 σ 两遍算（评满了也打不出 exhaustive=true）：文件 crates/singlefs-checker/te
      crates/mutations.tsv:1079 实审 B3a-3c：按位置寻址的树计数行又直接算 1u64 << |段|（第三条流 322 写的单元写段移位溢出，快
      crates/mutations.tsv:1080 实审 B3a-3c：L8 叶容器数又按写死的 5 个角色取（63 片叶容器，点名 74 项、末条 7 项）：文件 crate
    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证�
```

74 号那 3 条，在我的副本与 HEAD 副本上各跑一次 `cargo test --release -p singlefs-harness --test second_transaction_supplement_three_random_history`：
```
[带我的改动]
test seed_4000000204_raising_the_floor_on_narrow_devices_is_refused_before_any_write_instead_of_leaving_the_new_floor_on_one_device ... FAILED
test crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43 ... FAILED
test writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged ... FAILED
test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 147.45s
[HEAD，不带我的改动]
test crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43 ... FAILED
test seed_4000000204_raising_the_floor_on_narrow_devices_is_refused_before_any_write_instead_of_leaving_the_new_floor_on_one_device ... FAILED
test writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged ... FAILED
test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 145.72s
```

## litmus 静态核（`lkmm.sh --static-only` 被重型闸拒，照它的四条判法手核）

```
fence-removal True removed 1
litmus/publish-returns-after-the-rotation-barrier.litmus expect Never regs declared True
litmus/publish-returns-after-the-rotation-barrier-nofence.litmus expect Sometimes regs declared True
crates/singlefs-core/src/transaction.rs::persist_the_root_then_rotate_the_system_configuration True
crates/singlefs-core/src/recovery.rs::verified_system_configuration_slots True
named by rs: crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs
```
两个锚点函数在主工作区现版本里也各命中 1 次（`grep -c`：1、1）。对照组是正例去掉一道 `smp_wmb()`、别的逐行相同。**herd7 判 Never / Sometimes 分不分得开没有跑过**，归提交时 57 号。
P0 的形态：`WRITE_ONCE(*root)` → `WRITE_ONCE(*configuration)` → `smp_wmb()` → `WRITE_ONCE(*returned)`；P1 读 `returned` → `smp_rmb()` → 读 `configuration`；`exists (1:r0=1 /\ 1:r1=0)`。根与轮换之间不写屏障（FUA 返回才发轮换，那一段的序归 first-txn-* 两条），这样 P0 的写与屏障逐项就是录制流里发布的最后三步，绑定测试按这一点比。

## 受影响的层 0 流与崩溃枚举用例

- 判定集合变了的：I-7.13 多四条「这一槽不收」的理由；新增 I-7.14。`IMPLEMENTED_INVARIANTS` 从 48 条变 49 条。
- 合法历史（mkfs 只写 325 = 1024、371 = 64、417 = 环长现算且在段边界上，本盘设备号 = 盘身份）上，四条新理由与 I-7.14 都不会判违例；推的，依据是 74 号那一趟的计数：`I-7.13：判绿 4155 次、不适用 0 次`、`I-7.14：判绿 4155 次、不适用 0 次`。层 0 快档与全量没跑。
- 会跟着变的钉值：每条层 0 流、崩溃枚举用例打出的逐不变量计数行（`first_transaction_step_seven_layer0.rs` 的 `checker_line`、`crash.rs` 的 `checker_counts_by_invariant`、`layer0_progress.rs` 按清单解析的进度文件）都多一列 I-7.14；凡是把这一行整行钉死、或按 48 条数的，都会变。`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` 里也有这类逐不变量行（E161 装置），重跑时形状会变。
- 截断一格：断点续跑的进度文件是 48 列时写的，新清单读它不缺名字（只多一条），我没核 `layer0_progress.rs` 对缺列怎么判。
- 这一格交提交时的 `crash-verifier`：54 号快档与全量、登记的崩溃枚举用例都要重跑一遍，看上面那几行钉值。

## 停下交主 agent 的

1. **越出派发清单的一个文件**：`crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs` 的 `a_journal_ring_that_holds_fewer_than_the_safety_factor_records_or_ends_past_the_unit_area_is_refused` 有一格「环长 = 3 × 4096、417 照旧 50176 ⇒ `Ok`」。新判法下这份槽的 417 不是环长现算的 1025，报 `UnitAreaStartNotTheSlotAfterTheJournalRing`；而且 1025 不在段边界上，这个环长在新表下怎样都不被收。我把那一格期望改成这个成员（环长那一判过了、下一判拦下），4 行。不改它，打上补丁这个二进制就红。它不在「要动的 crates 文件」里，派发闸没按它判撞车：请核这一刻有没有别的实现员在改它（合入后验证一那批「几十份测试」里有没有它）。
2. **litmus 绑定测试放在哪**：`lkmm.sh` 要求 `crates/` 下有 `.rs` 写出 Never 那一条的文件名；规格把 `publish_order_matches_litmus.rs` 归给改法 B。我在这一件的新测试文件里加了第二条测试（`the_rotation_barrier_litmus_writer_follows_the_last_three_steps_of_every_recorded_publish`）来做绑定，并为此把正例写成根与轮换之间不带屏障。改法 B 要是也在 `publish_order_matches_litmus.rs` 里罩这一对，两处会各比一次；留哪一处由主 agent 定。
3. **I-7.14 原句（交 kb 下一批）**：「I-7.14（系统配置本盘设备号等于盘在池里的身份）：每块盘两个系统配置槽里，每个自证过（magic 与整槽校验和对）、fsid 与本池相同的槽，自举头里的本盘设备号等于这块盘在池里的身份。依据 D18（块里携带什么信息） 已定项 11 的系统配置字段表「本盘设备号」一行；实现一侧是可写挂载比盘表（`mount.rs` 的 `device_table_disagreeing_with`，`OwnDeviceNumberDiffersFromTheIdentityHandedIn`）。checker 实现：`crates/singlefs-checker/src/image.rs` 的 `judge_own_device_number_is_the_device_identity`。状态：已实现（编号暂立）。」另外 `invariants.md` 第 12 行写「判 48 条」要改成 49；I-7.13 那一行要补 325 / 371 / 417 那四项（与 core 收同一张表那一句今天已写，checker 这边的实现现在跟上了）。
4. **改法 B 合入之前的一格**：core 今天还不读 325 / 371（改法 B 做）。在 B 合入之前，B、C 两格（325 写 1023、371 写 65）checker 报 I-7.13「实现整池拒绝挂载」而 core 照常挂——与攻方 E 格同一个形态，只是从一格变三格；B 合入之后两边一致。补丁要是先于 B 打，中间那段时间就是这个状态。
5. **第 4 件用例钉的是文件镜像上的结局**：盘 1 的轮换写在屏障报错之前已经进了文件镜像（harness 的文件设备没有写缓存），所以之后的挂载择到 (1,1)。「屏障报错时轮换其实没持久」那一种崩溃状态这条用例不罩；它钉的只是挂载返回哪个错、录制流里落到哪一步、挂载之后在同一份镜像上怎么判。

条款没写、我没加的非分支项：没有。

## 没做什么

- 没走三方对抗；没提交；checker 档（54 号快档与全量、登记的崩溃枚举用例）、herd7（57 号）、QEMU、crates 变异整表（59 号）都没跑，归提交时的 `crash-verifier`。
- 92 号在副本里判不了（不是 git 仓），没在主工作区跑（主工作区还没打补丁，判的不是这一版）。
- 没重跑攻方的 Y4 用例 `opus_r2_y4_…rs`，只照它造镜像的写法把五格写成坏镜像。
- 74 号那 3 条红与 `publish_order_matches_litmus` 的红没有碰（HEAD 上就红，归别的件）。

## 补记：派发之后主工作区里我那几份文件被别的会话改过

交回前最后一次核（`git status`）：主工作区 `crates/singlefs-checker/src/walk.rs` 也被别的会话改了（`TailOfTheBirthIdentity::value` 改名 `birth_identity_tail_number`，5 行，第 351–501 行一带）；`checker_known_bad_images.rs` 的 4 行 import 改名同前。这两份都在派发给我的「要动的 crates 文件」里。补丁对主工作区这一刻 `git apply --check` 仍然过（退出码 0），`apply-writer-patch.py --dry-run` 原样输出：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1378 行`。打补丁之前请再核一次这两份此刻谁在改。

## 删掉的草稿

删了：`/tmp/claude-1000/impl-r2-fixes-a/copy`（仓副本，290M）、`/tmp/claude-1000/impl-r2-fixes-a/copy-head`（HEAD 基线副本，285M）、`/tmp/claude-1000/impl-r2-fixes-a/target`（编译目录，21G）、`/tmp/claude-1000/impl-r2-fixes-a/target-head`（编译目录，1.2G）、`/tmp/claude-1000/impl-r2-fixes-a/patchwork`（拼补丁用，1.5M）、`/tmp/claude-1000/impl-r2-fixes-a/baseline`（四份源文件的起点快照，1.5M）。留着：`patch/`、`report.md`、`progress.md`、各份 `*.log` 与 `prove-red-logs*/`（主 agent 核用）。
