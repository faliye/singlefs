# 55 号装置包装次序：改回 Recording 在外、FaultInjecting 在里

时刻 UTC（本机时钟）。里程碑 `.claude/kb/milestone/02-second-txn.md` 收尾批「55 号装置包装次序」。在草稿目录的副本里改，主工作区一个字节没动；交 `patch/`。

## 结论

`crates/singlefs-harness/src/bin/first_transaction_on_device.rs` 第 557 行的 `CountedDevice<Inner>` 类型别名，从 `FaultInjectingBlockDevice<RecordingBlockDevice<Inner>>`（故障注入在外、录制器在里）改成 `RecordingBlockDevice<FaultInjectingBlockDevice<Inner>>`（录制器在外、故障注入在里）。连带改了它的 4 处构造点、4 处拆包点、1 处 `.plan()` 访问点（这些都是同一个类型别名换了内层顺序之后，编译器要求跟着改的位置，不是新的行为）。新增测试 `crates/singlefs-harness/tests/a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side.rs` 直接钉住这条顺序的语义：一道被吞的屏障，调用方发过、进了录制流，转发给真设备的那份计数（`FaultDeviceCounts::barriers_forwarded`）没涨。

## 写过的文件

在 `/tmp/claude-1000/impl-gate55-wrapping/repo/` 这份副本里：

- `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`：类型别名 `CountedDevice` 换内层顺序（连同它的文档注释）；4 处构造点（`reopen_and_mount_writable`、`main()`、测试 `second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches`、测试助手 `counted_sparse_devices`，连同它的文档注释）跟着换内层套的次序；4 处拆包点（同样 4 个位置里各一次 `device.into_inner_and_operations().0.into_inner()`，原来是 `device.into_inner().into_inner_and_operations().0`）跟着换调用次序；1 处 `.plan()` 访问点改成 `.inner().plan()`（第 3062 行附近，`switched.devices[index].1`）。
- `crates/singlefs-harness/tests/a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side.rs`：新文件，见下「新测试」。
- `crates/mutations.tsv`（只在副本里追加，主表不动）：追加 1 行，见下「变异证红」；追加内容原样在 `patch/mutations-append.tsv`。

`git diff --stat -- crates litmus`（主工作区当前状态，原样；本机此刻另有会话在改 `crates/` 的别的文件，这一条命令分不出哪部分是这一轮的——本轮只落在 `first_transaction_on_device.rs` 与新测试文件两处，我的补丁 `crates.patch` 的 `git apply --stat` 单列在下面）：

```
 crates/mutations.tsv                               |    92 +-
 crates/singlefs-core/src/admission.rs              |   405 +-
 crates/singlefs-core/src/allocator.rs              |   258 +-
 crates/singlefs-core/src/mount.rs                  |   543 +-
 crates/singlefs-core/src/mounted_session.rs        |    30 +-
 crates/singlefs-core/src/transaction.rs            |   163 +-
 .../src/bin/e156_allocation_basis_counts.rs        |    10 +-
 .../src/bin/e158_root_choice_repair.rs             | 11402 ++++++++++++++++++-
 .../src/bin/first_transaction_on_device.rs         |   101 +-
 crates/singlefs-harness/src/crash_injection.rs     |    22 +-
 crates/singlefs-harness/src/history.rs             |    22 +-
 crates/singlefs-harness/src/model.rs               |   179 +-
 crates/singlefs-harness/src/model_comparison.rs    |   237 +-
 .../admission_checkpoint_cost_per_device_paths.rs  |   539 +-
 ...hecker_narrow_invariants_and_abandoned_roots.rs |     8 +-
 crates/singlefs-harness/tests/common/mod.rs        |   311 +
 .../singlefs-harness/tests/common_admission/mod.rs |     3 +-
 .../tests/publish_order_matches_litmus.rs          |    12 +-
 ...n_admission_raises_the_floor_before_refusing.rs |   276 +-
 ...inside_the_floor_raise_pushed_by_the_session.rs |    14 +-
 .../tests/second_transaction_step_five_reuse.rs    |     2 +-
 .../tests/second_transaction_step_one_overwrite.rs |    29 +-
 ...second_transaction_step_three_formatted_pool.rs |   504 +-
 ...transaction_supplement_three_crash_injection.rs |     4 +-
 ..._transaction_supplement_three_random_history.rs |    10 +-
 ...d_transaction_supplement_two_unequal_devices.rs |    56 +-
 ...upplement_two_unreadable_abandoned_root_slot.rs |    44 +-
 27 files changed, 14413 insertions(+), 863 deletions(-)
```

`git apply --stat /tmp/claude-1000/impl-gate55-wrapping/patch/crates.patch`（针对主工作区，只读校验，我的补丁自己的范围）：

```
 .../src/bin/first_transaction_on_device.rs         |   54 +++++++------
 ...e_recorded_stream_but_not_on_the_device_side.rs |   83 ++++++++++++++++++++
 2 files changed, 110 insertions(+), 27 deletions(-)
```

`git apply --check /tmp/claude-1000/impl-gate55-wrapping/patch/crates.patch`（主工作区，只读）退出码 0；另外在隔离目录做过一次真正的 dry-apply（`git apply -p1` 到只含这一个文件的隔离拷贝，与副本里的最终版本 `diff -q` 逐字节比对），两个文件都 `identical`。

## 验收第 1 条：包装次序改回

见上「结论」与「写过的文件」。改法本身很小：只是 `type CountedDevice<Inner> = A<B<Inner>>` 换成 `B<A<Inner>>`；因为整个二进制只有这一处类型定义，其余 9 处都是编译器逼着跟着换的调用形状，不是独立的行为分支（下面「变异证红」一节解释为什么这 9 处不能各自单独做一条变异）。

## 验收第 2 条：新测试

`a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side`（`crates/singlefs-harness/tests/` 下的独立集成测试；它不能引用 `CountedDevice`——那是二进制的私有类型别名，集成测试拿不到——所以直接手搭同一种两层顺序，验的是这层顺序本身的性质）：

1. 手搭 `RecordingBlockDevice::with_shared_stream(identity, FaultInjectingBlockDevice::new(identity, SparseBlockDevice::new(..), plan.clone()), stream.clone())`（与 `CountedDevice` 改后的内层次序同构）。
2. 用 `swallow_the_next_barrier_on`（与 `first_transaction_on_device.rs` 里 `swallow_the_next_barrier_on` 同一条 `FaultSchedule`：`BarrierIsSwallowed`、`FaultDeviceSelector::OnlyDevice`、`FaultOccurrence::TheNthMatchingCall(1)`）武装计划，调一次 `.barrier()`。
3. 断言：`stream.operations()` 长度为 1、种类是 `RecordedOperationKind::Barrier`（程序发的这次调用进了录制流）；`plan.counts_of_device(identity).barriers_forwarded == 0`（没有转发给它下面那块真设备）、`.barriers_swallowed == 1`。

改前（次序倒转，即 `first_transaction_on_device.rs` 今天在主工作区的样子）这条测试红：`FaultInjectingBlockDevice` 在外层直接吞掉调用、`return Ok(())`，永不 call `self.inner.barrier()`（`RecordingBlockDevice`），录制流因此保持空，第一条断言 `operations.len() == 1` 判 `left: 0, right: 1` 崩溃（下面「变异证红」一节有这次真跑出来的 panic 原文）。改后（次序改回）这条测试绿：跑过，见下方 `cargo test` 原样输出。

## 验收第 3 条：别的档录制流逐字不变

**direct 档（宿主上跑，未起虚机）**：两块叫 `nvme0n1` 的 4 GiB 稀疏文件放进 `d0/`、`d1/`（本机 `physical_block_size=512 minimum_io_size=512 write_cache=write_back`，与虚机几何相同），跑 release 二进制（经 `run-with-memory-cap.sh 8G` 与 `capped.sh 3`），`name=segments` 五行原样：

```
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[system_configuration_slot×2]
E7RESULT name=segments path=warm_up operations=18 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=transaction operations=33 segments=24+2+1+2 closed_form=16777223 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=post_mkfs_stream operations=53 segments=2+2+1+2+2+1+26+2+1+2 closed_form=67108885 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
```

逐行核对 `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs`（该文件自己搭 mkfs、取号、暖机、事务，与本二进制不共用切段入口）：mkfs = 第 329 行 `(23, "12+1+1+1+4", 4114)`；instance_acquisition = 第 341 行 `(2, "2", 4)`；warm_up = 第 353 行 `(18, "2+1+2+2+1+2", 15)`；transaction = 第 365 行 `(33, "24+2+1+2", 16_777_223)`；post_mkfs_stream = 第 377 行 `(53, "2+2+1+2+2+1+26+2+1+2", 67_108_885)`。五行逐字相同。这份 direct 档输出与主 agent 给的调查报告（`/tmp/claude-1000/investigate-gate55-segments/report.md` 第 63–73 行）当时测出的数字也逐字相同（那份报告是在改包装次序**之前**测的 direct 档，证实包装次序这次改动对 direct 档零影响）。

`cargo test`（在副本里）同样绿：

```
test recorded_paths_match_the_registered_segment_sequences ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.34s

test litmus_writer_threads_follow_the_recorded_publish_order ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.31s
```

（`first_transaction_step_five_publish.rs`、`publish_order_matches_litmus.rs` 都没被本轮改动碰到——它们走 `common::build_pool()`，只用 `RecordingBlockDevice`，不套 `FaultInjectingBlockDevice`，与 `CountedDevice` 无关；跑一遍是为了核实这条改动确实没有波及它们。）

理论依据（不只是跑了没红）：`SharedFaultPlan` 未武装（`unarmed`）时 `decide()` 恒返回 `None`，`FaultInjectingBlockDevice` 的每个方法都无条件转发给 `inner` 并计数——`direct`、`second-transaction`、`second-instance`、`raise-rollback-floor` 四档全程都是未武装的计划，两种包装顺序下发生的调用序列因此逐字相同；只有 `skip-first-transaction-barrier` 这一档会武装一次 `BarrierIsSwallowed`，这也是唯一一档次序改了会变的。

## 验收第 4 条：skip-first-transaction-barrier 档，改前改后的 `name=segments` 行原文

同样在宿主上跑（同一对稀疏盘、同一个二进制，只是先跑「改前」再切回「改后」重编）。

**改前**（主工作区今天的样子：故障注入在外、录制器在里，把副本临时换回 `orig-snapshot` 里那份原文重编出来跑的）：

```
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[system_configuration_slot×2]
E7RESULT name=segments path=warm_up operations=18 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=transaction operations=32 segments=26+1+2 closed_form=67108868 kinds=[unit_write×24,journal_record×2,barrier×3]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=post_mkfs_stream operations=52 segments=2+2+1+2+2+1+28+1+2 closed_form=268435474 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,journal_record×2,system_configuration_slot×2,barrier×3]|[root_record_fua]|[system_configuration_slot×2]
```

`transaction`（32 步、`26+1+2`）与 `post_mkfs_stream`（52 步、`…+28+…`）与 direct 档不同：被吞的那道屏障没进录制流，`unit_write×24` 与后面的 `journal_record×2,barrier×3` 因为少了一道能关段的屏障合并成一段（`26+1+2`）。这与主 agent 给的调查报告「路二：实测边界对写死边界」一节量出的数字（`operations=32 segments=26+1+2 closed_form=67108868`）逐字相同——那份报告是在「23 写死」那个 bug 已经被上一位实现员修掉、但包装次序还没修之前测的，与我这次单独复现的「改前」状态是同一个代码路径。

**改后**（这次改动之后）：

```
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[system_configuration_slot×2]
E7RESULT name=segments path=warm_up operations=18 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=transaction operations=33 segments=24+2+1+2 closed_form=16777223 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=post_mkfs_stream operations=53 segments=2+2+1+2+2+1+26+2+1+2 closed_form=67108885 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
```

改后 `skip-first-transaction-barrier` 档与 `direct` 档逐字相同（`transaction` 回到 33 步 `24+2+1+2`）：这条改动本身让「程序自己发了什么」这份记录，与「没有故障时的样子」看起来一样——因为程序确实发了那道屏障，只是它没能到真设备。改法命中的判据由此挪到了「设备侧」（`FaultInjectingBlockDevice` 的 `barriers_forwarded` / `barriers_swallowed` 计数，或真设备场景下 QEMU 的 `blklogwrites` 独立日志），不再挪到「录制流本身缺一步」——这正是 `first_transaction_on_device.rs` 文件头第 8–9 行原有的文档注释（本次没有改动那两行）已经写的口径：「第一个事务在盘 0 上漏掉…屏障，而录制器照样记下它——程序以为发了，盘上没收到，宿主那一侧必须判红」。55 号那一侧的判据要不要跟着调整（`.claude/kb/vm-harness.md`「设备侧独立录制：blklogwrites 模式」一节讲的独立日志比对），按主 agent 派发提示「55 号那一侧怎么比由 singlefs-39 改，你只交数」——本报告只交这四行数，不改 55 号脚本、不判它的比对逻辑。

## 变异证红

只加了 1 条变异，落在**新测试文件自己**的构造代码上，不落在 `first_transaction_on_device.rs` 的 9 处改动位置上。原因：

`CountedDevice<Inner>` 是全二进制唯一一处类型定义，其余 9 处（4 处构造、4 处拆包、1 处 `.plan()` 访问）全部要按这一处类型定义的形状来写，才编译得过。单独改回其中任意一处（比如只把 `main()` 里的构造改回旧顺序，其余 8 处不动），要么触发类型不匹配（`let mut devices: Vec<(DeviceIdentity, CountedDevice<Inner>)> = ...` 右边构造出来的类型与左边注解的 `CountedDevice<Inner>`——此刻仍是修好之后的顺序——对不上），要么直接找不到方法（`RecordingBlockDevice` 从来没有过 `into_inner()` 或 `.plan()` 方法，只有 `into_inner_and_operations()`；把某处拆包点单独改回 `device.into_inner().into_inner_and_operations().0`，而 `device` 的类型仍是修好后的 `RecordingBlockDevice<...>`，会是「找不到这个方法」的编译错误）。逐处验证过（不是猜的）：

- 类型别名那一行单独改回旧序：其余 8 处的构造/拆包表达式产出的类型与它们各自 `let` 绑定标注的 `CountedDevice<Inner>`（此刻还是新序）不再匹配——8 处里随便挑一处都会是这类不匹配。
- 4 处构造点里任意一处单独改回旧序：那处 `let ...: Vec<(DeviceIdentity, CountedDevice<...>)> = (...).collect()` 的右边类型与左边注解不匹配。
- 4 处拆包点里任意一处单独改回旧序（`device.into_inner().into_inner_and_operations().0`）：`device` 的类型是（未改的）`RecordingBlockDevice<FaultInjectingBlockDevice<Inner>>>`，它没有 `.into_inner()` 方法，直接编译不过（`RecordingBlockDevice` 只有 `into_inner_and_operations()`，见 `crates/singlefs-harness/src/lib.rs`）。
- `.plan()` 访问点单独改回旧序（`switched.devices[index].1.plan()`）：`.1` 的类型是（未改的）`RecordingBlockDevice<...>`，它没有 `.plan()` 方法（只有 `FaultInjectingBlockDevice` 有），直接编译不过。

`research/scripts/mutate.sh` 文件头明确写着「编译失败不等于『没抓到』…它是一条无效变异…混成一类的话，一个编译不过的变异会被报成测试盲区，把人引去改测试」——这 9 处产品代码位置，只要单独反转任意一处，**每一次都会是这一类无效变异**，不会给出任何有用的信号，也不该占 `crates/mutations.tsv` 的一行（那会让门禁 59 号往后每次复跑都在同一处报无效）。所以这 9 处不追加变异；追加的 1 条落在新测试文件自己的构造代码上（它不依赖任何共享类型别名，两种顺序都编译得过，是唯一能表达「顺序反了会怎样」又不触发编译错误的位置）。

这一件是设计层面的判断，写进这一节交主 agent：如果主 agent 认为 9 处产品代码位置仍然需要各登记一行（哪怕注定判「无效」），请另行指示，我会照办；目前的判断是不加。

追加的这一条：

- 名字：`里程碑「第二个事务」收尾批「55 号装置包装次序」：新测试自己的构造顺序改回故障注入在外、录制器在里（旧序）`
- 文件：`crates/singlefs-harness/tests/a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side.rs`
- 参数：`-p singlefs-harness --test a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side`
- 必须红的测试：`a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side`

`bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-gate55-wrapping/repo --memory 8G singlefs-harness '里程碑「第二个事务」收尾批「55 号装置包装次序」：新测试自己的构造顺序改回故障注入在外、录制器在里（旧序）'` 原样输出（fmt 之后又跑了一遍，两次都抓到）：

```
里程碑「第二个事务」收尾批「55 号装置包装次序」：新测试自己的构造顺序改回故障注入在外、录制器在里（旧序）	抓到	a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side 红了（日志 …/prove-red-logs/001.log）
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
```
退出码 0。改坏那一行时的 panic 原文（`prove-red-logs/001.log`）：

```
thread 'a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side' panicked at crates/singlefs-harness/tests/a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side.rs:63:5:
assertion `left == right` failed: 程序发的这一次屏障调用要进录制流，即使它被下面那层吞掉：[]
  left: 0
 right: 1
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

哪几行证过：以上这 1 行（表里 1169 行之后新追加的那一行）。哪几行留给 59 号：无——上面已经说明为什么另外 9 处不追加行。

## 交回前的验证（副本里，最后一遍）

### 动到的测试二进制（整个二进制）

```
$ cargo test --offline -p singlefs-harness --bin first_transaction_on_device
running 17 tests
... （17 个测试名逐行 ok，含 the_boundary_marked_before_the_first_transaction_splits_warm_up_from_the_transaction）
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.20s

$ cargo test --offline -p singlefs-harness --test a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side
running 1 test
test a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### `cargo fmt --all -- --check`

```
（无输出，退出码 0）
```

（中途 `cargo fmt --all -- --check` 先报过一次要重排的 diff：`.plan()` 那一行换行方式、新测试文件里一处 `use` 导入换行；跑了 `cargo fmt --all` 之后 `--check` 转绿，逐字重跑过两个测试目标仍然全绿，重跑过一次 `prove-red.sh` 仍然抓到。）

### `cargo clippy --all-targets --all-features -- -D warnings` + code-discipline 那 7 条

```
$ cargo clippy --offline --all-targets --all-features -- -D warnings \
    -D clippy::wildcard_enum_match_arm -D clippy::allow_attributes_without_reason \
    -D clippy::cast_possible_truncation -D clippy::cast_sign_loss -D clippy::cast_possible_wrap \
    -D clippy::undocumented_unsafe_blocks -D clippy::shadow_unrelated
    Checking singlefs-format v0.1.0 (…)
    Checking singlefs-checker v0.1.0 (…)
    Checking singlefs-core v0.1.0 (…)
    Checking singlefs-harness v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.68s
```

退出码 0。

### `cargo build --offline --all-targets`

```
   Compiling singlefs-core v0.1.0 (…)
   Compiling singlefs-checker v0.1.0 (…)
   Compiling singlefs-harness v0.1.0 (…)
   Compiling singlefs-format v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.73s
```

退出码 0。

## 门禁阶段归属表登记给 `implementation-writer` 的那几道

`awk` 现查得到 7 道：33-mutation-tables.sh、53-format-const-placeholders.sh、74-model-differential.sh、92-layout-checker-sync.sh、94-checker-implementation-disjoint.sh、93-feature-bits.sh、89-closeout-row27-preconditions.sh。逐道原样末行：

```
33-mutation-tables.sh（在副本里跑，含新追加的 1 条变异）exit=1
  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）：
      crates/mutations.tsv:659 E158 root_choice_repair session s10：attempt_mount_writable_then_raise_floor 传给 raise_rollback_floor 的目标 floor 改成远超上限（该传调用方给的 new_floor，改成 new_floor+1_000_000，必定撞 RollbackFloorAboveCeiling）：原文在 crates/singlefs-harness/src/bin/e158_root_choice_repair.rs 里命中 2 次

53-format-const-placeholders.sh（在副本里跑）exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23 已定项 19；C506；D22 已定项 1；C323）

74-model-differential.sh（在副本里跑，release 下跑 second_transaction_supplement_three_random_history.rs）exit=1
  ✗ 随机历史的测试二进制判红：crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43、random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation、rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline（21 passed; 3 failed; 2 ignored）

92-layout-checker-sync.sh（副本没有 .git，本阶段自己判「不是 git 仓」跳过 exit=77；改在主工作区读专门跑了一遍，只读、没写）exit=0
  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15 已定项 4 的登记表；这次改动比 faf255e235300d129ede6d6f85af31d686a88519，111 个格式常量里变了 0 个……

94-checker-implementation-disjoint.sh（在副本里跑）exit=0
  ✓ checker 与实现只共享常量模块 singlefs-format（……）；checker 的 4 份源码零处引 singlefs_core……

93-feature-bits.sh（在副本里跑）exit=0
  ✓ feature bit 位号在记账表、D15 已定项 4 与代码三处一致（……）

89-closeout-row27-preconditions.sh（在副本里跑）exit=77
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（……）
```

### 33 号红：与本轮改动无关（pre-existing，另一个会话正在改的文件）

红的那一行指 `crates/mutations.tsv:659`，原文要在 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 里命中 1 次，实际命中 2 次。这个文件在主 agent 派发提示的「别人在改的 crates/ 文件」清单里（「E158 第二段执行员正在改 `src/bin/e158_root_choice_repair.rs`」），我没有碰过它。核实：直接在**主工作区**（未受这一轮任何改动影响，只读）跑同一段 python 逐字核对，命中数同样是 2：

```
main workspace hit count: 2
```

即此刻主工作区本身（在我这一轮开始之前就已经是这样）就处在这个状态，与我的改动无关，不修（`agent-common.md`「门禁」一节：红了先看它点名的文件在不在这一轮的改动里；不在的不修，照写）。

### 74 号红：与本轮改动无关（隔离核实过）

在副本里把 `first_transaction_on_device.rs` 换成 `orig-snapshot`（这一轮改动之前的原样）、把新测试文件移出 `tests/` 目录，重跑 74 号：同样 3 个失败、同样的用例名（`gate-74-isolation.log`）：

```
failures:
    crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43
    random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
    rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline
test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 72.63s
```

证实这红与本轮改动无关（这几个用例名带着「known_red_form」「closeout_row_43」「abandoned_timeline」，像是收尾批别的进行中改动留下的中间状态；`second_transaction_supplement_three_random_history.rs`、`history.rs`、`model_comparison.rs` 都不在我的写范围与要动的文件清单里，也确认没有被我碰过）。核完把副本换回改后的版本，`sha256sum` 核对与 fmt 之后的最终版一致（`d33ed495b482ec9fc98a9a9a6ed5cd141265a792a561a60b547ab88dcecc40a1`）。

## 停下交主 agent 的设计问题

1. **变异覆盖范围**：`CountedDevice` 的 9 处产品代码位置（类型别名 + 4 构造 + 4 拆包 + 1 `.plan()` 访问）因共享同一处类型定义，任意单独反转都是编译错误（无效变异），不是行为分支；只给新测试文件自己的构造代码加了 1 条能编译、能证红的变异。若主 agent 希望这 9 处仍各登记一行（哪怕注定判「无效」，供 59 号如实记录），请另行指示。
2. **`fault_injection.rs` 模块头的文档注释**（第 5–7 行：「包装的位置在录制器外面（调用方与录制器之间）…录制流因此恒等于真正落到盘上的那一串」）描述的是 `history.rs::HistoryDevice`（`FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>`，本轮没有改它）那一条独立栈的性质，不是 `CountedDevice`（本轮改的这一条）的性质——这条改动之后，「包装位置在录制器外面」不再对整个 crate 成立，只对 `HistoryDevice` 成立。`fault_injection.rs` 不在我的「要动的 crates 文件」清单里，我只在 `CountedDevice` 自己的文档注释里加了一句区分（见「写过的文件」一节），没有去改 `fault_injection.rs` 那几行——是否要去把那句话改得更精确，交主 agent 判断（不影响本轮验收：`HistoryDevice` 与它依赖的 `crate::crash::MemoryPool::apply` 都没被这一轮 touch 到）。

## 没做什么

- 没走三方对抗、没提交；层 0 全量、QEMU、herd7、crates 变异整表（59 号）归 `crash-verifier`。
- 没跑 `.claude/gate.d/55-qemu-first-transaction.sh`、`vm-bench.sh`；没起虚机——55 号那一侧怎么比对（判据要不要跟着这次改动调整）按派发提示「由 singlefs-39 改，你只交数」，本报告只交上面那两组「改前/改后」的 `name=segments` 行。
- 没碰 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（另一个会话在改，见「33 号红」一节）；没碰 Z3-A 乙涉及的 `mount.rs`、`mounted_session.rs`、`lib.rs`；没碰 B3c-3 涉及的 `transaction.rs`、`model.rs`、`model_comparison.rs`、`history.rs`；没碰层 0 发现日志涉及的 `crash.rs`、`layer0_progress.rs`、`lib.rs`。
- `crates/mutations.tsv` 主表一个字节没改，只在副本里追加了 1 行做证红用；追加内容原样交在 `patch/mutations-append.tsv`，由 `apply-writer-patch.py` 合并进主表。
- 没有判「skip-first-transaction-barrier 那一档在新包装次序下，`first_transaction_device_log_check` 的『盘 0 必须有 divergence』那条判据还能不能继续用录制流本身判出来」——这正是本轮改动之后录制流不再缺步骤、判据要挪到设备侧计数或 blklogwrites 独立日志的原因，交主 agent / singlefs-39 处理。
- 交回前删掉了草稿目录里用来生成「改前」skip 档数据的临时构建产物：`repo/target/`（release + debug 都在这份 target 里，交回前删除）与 `hostrun/` 下的两块 4 GiB 稀疏文件（`d0/nvme0n1`、`d1/nvme0n1`，跑完即删）；`repo/`（不含 target，约 260 MB 源码副本）与 `orig-snapshot/`、`new-snapshot/`、`patch/`、各份 `.log` 留着交主 agent 核对，体积都在几百 KB 到几 MB 量级。
