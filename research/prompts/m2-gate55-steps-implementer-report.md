# 55 号装置步数：两处写死的 23 改成量出来的边界

里程碑 `.claude/kb/milestone/02-second-txn.md` 收尾批「55 号装置步数」；输入是 `/tmp/claude-1000/investigate-gate55-segments/report.md`（sha256 前 8 位 30b8ddef）。

## 结论

两处都不再写死步数，从「暖机 `warm_up()` 已调完、`publish_first_file()` 还没调」这一处量出来，逐字与今天的写法核对过：

1. **`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`**：新增 `operation_count_before_first_transaction`（在 `ScenarioPoint::BeforeFirstTransaction` 回调里用 `stream.operation_count()` 量），替掉 `operations.len() - 23`；把切五段的逻辑抽成 `first_transaction_paths(operations, mkfs_operation_count, operation_count_before_first_transaction)`（`main()` 与新测试共用同一份定义，不是各自重复一遍字面量）。
2. **`crates/singlefs-harness/tests/publish_order_matches_litmus.rs`**：`&operations[operations.len() - 23..]` 改成 `&operations[pool.warm_up_operation_count..]`。
3. **`crates/singlefs-harness/tests/common/mod.rs`**（第三份必改文件，见下「为什么多改一个文件」）：`BuiltPool` 加字段 `warm_up_operation_count`，在 `build_pool()` 里紧跟 `warm_up()` 调用之后用 `stream.operations().len()` 量出来，与 `first_transaction_step_five_publish.rs` 自己那份 `warm_up_operation_count`（该文件第 187–188 行）量的是同一个边界——都是「调完 `warm_up`、还没调 `publish_first_file`」那一刻。

推翻条件：把这两处改回写死的步数（`crates/mutations.tsv` 新增的两条变异），点名的测试必须变红；本报告「变异证红」一节贴了原样输出，两条都抓到了。

## 为什么多改一个文件（超出派发给的两个文件）

`publish_order_matches_litmus.rs` 用的是 `crates/singlefs-harness/tests/common/mod.rs` 的 `build_pool()`，那个 `BuiltPool` 只有 `mkfs_operation_count`，没有暖机边界。要在litmus 测试里不写字面量地拿到这个边界，只能：(a) 改 `common/mod.rs` 加一个字段（选了这条，纯加字段，向后兼容）；或 (b) 在 litmus 测试里另用 `StepKind` 分类去找「post-mkfs 里第一个 UnitWrite」的位置（不改 common，但那是重新发明一种判定，不是「引用同一个边界」）。选 (a) 是因为它与 `first_transaction_step_five_publish.rs` 那份私有 `build_pool` 的写法（`warm_up_operation_count = stream.operations().len()`，紧跟在 `warm_up()` 调用之后）完全同构，是「同一个定义处」最直接的落地。`common/mod.rs` 被 68 个测试文件 `mod common;` 引用，但只有它自己的 `build_pool()` 构造 `BuiltPool { .. }` 字面量（`grep -rn "BuiltPool {" crates/singlefs-harness/` 核过），新增字段是纯加法，`cargo build --offline --all-targets` 全绿证实没有连带破坏。

## 中途插入：C554 乙补丁落主工作区，改了 `first_transaction_on_device.rs`

主 agent 2026-09-27 通知：C554 乙给 `MountError` 加了新成员，同一份文件里两处穷举 `match` 补了新分支（`MountError::NewerStateStillUnreadableAfterOneReread(_)`，约在原文件的 `reopen_and_mount_writable` 与 `raise_the_rollback_floor_and_describe` 两处 match，行号约 837 / 1250）。同一批还改了 `crates/singlefs-harness/tests/common/mod.rs`（加 `UnreadableRange` 一整套读故障装置）与 `crates/mutations.tsv`（追加它自己的变异行）。核实：这几处改动与我这两处的文本锚点不重叠（C554 的两处 match 分支在我的 `first_transaction_paths` 与 `main()` 改动之前；`common/mod.rs` 的新增代码在 `build_pool()` 之后的下一个函数里）。

处理：把草稿目录的仓副本全量重新 `rsync -a --delete --exclude target <主工作区>/crates/ /litmus/`（覆盖掉副本里旧的、C554 之前的版本），在这份新底座上把我这三份文件的改动逐条重放（Edit 的 old_string / new_string 与之前一致，因为文本锚点没被 C554 碰到），`crates/mutations.tsv` 重新追加同样两行。之后完整重跑：`cargo build --offline --all-targets`、两个测试二进制、`cargo fmt --all -- --check`、clippy、`prove-red.sh`（见下面各节），全部绿/抓到。

**底座时刻与 sha256**（主工作区）：
- 底座时刻：2026-09-27 起（那次整仓 rsync 的时刻），到本报告落盘前最后一次核对 2026-09-27，这三个文件与 `crates/mutations.tsv` 的 sha256 一直没变（每次核对都重跑了 `sha256sum`，见下）：
  - `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`：`3f84693ff6cd9c2200e92e63f38f8733fe3e99c64a9f0f7afcfdc703974738bb`
  - `crates/singlefs-harness/tests/common/mod.rs`：`84ef4a64987b2d9a32838ced7671a9bb3cdaec0c2c465d66728fe48657843ff3`
  - `crates/singlefs-harness/tests/publish_order_matches_litmus.rs`（未被 C554 碰过，从会话开始就没变过）：`a861c32621f98d376dfcc83ca344e924e0bf188982866e5161e66af1af2d76c0`
  - `crates/mutations.tsv`：`643afbf6e443492383aeeac54c277f23606a817f2512f3a9d4900b582da296dd`
- `git apply --check /tmp/claude-1000/impl-gate55-steps/patch/crates.patch`（在主工作区当前 HEAD + 未提交改动上）：退出码 0，最后一次核对同上时刻。另外做了一次真正的 dry-apply（拷主工作区这三份文件到隔离目录、`git apply -p1`、与副本里的最终版本 `diff -q` 逐字节比对，三份都相同）不仅是 `--check`。

## 写过的文件（草稿目录 `/tmp/claude-1000/impl-gate55-steps/repo/` 里，不在主工作区）

- `crates/singlefs-harness/src/bin/first_transaction_on_device.rs`：加 `operation_count_before_first_transaction`、在 `BeforeFirstTransaction` 回调里量它、抽出 `first_transaction_paths` 函数、`main()` 改成调它、`mod tests` 的 `use super::{..}` 加 `first_transaction_paths`、新增测试 `the_boundary_marked_before_the_first_transaction_splits_warm_up_from_the_transaction`。
- `crates/singlefs-harness/tests/common/mod.rs`：`BuiltPool` 加字段 `warm_up_operation_count`，`build_pool()` 里量它并带进构造。
- `crates/singlefs-harness/tests/publish_order_matches_litmus.rs`：`transaction` 切片改用 `pool.warm_up_operation_count`；新增一条钉死 `transaction.len() == 33` 的断言。
- `crates/mutations.tsv`：追加两行（草稿目录另有 `patch/mutations-append.tsv` 同样两行，供主 agent 合并进主表）。

`git apply --stat` 给出的等价 `git diff --stat`（副本没有 `.git`，用同一份 `crates.patch` 让 `git apply` 在主工作区侧算出来，逐字）：

```
 .../src/bin/first_transaction_on_device.rs         |   93 +++++++++++++++++---
 crates/singlefs-harness/tests/common/mod.rs        |    8 ++
 .../tests/publish_order_matches_litmus.rs          |   12 ++-
 3 files changed, 97 insertions(+), 16 deletions(-)
```

`crates/mutations.tsv` 的改动没有算进这份 diff（补丁按格式要求把它拆到 `mutations-append.tsv`，`crates.patch` 用 `':!crates/mutations.tsv'` 排除掉）。

## 验收第 1 条：direct 五行与 `first_transaction_step_five_publish.rs` 逐字相同（宿主上跑，没起虚机）

按投资告报告的路一同样做法：两个都叫 `nvme0n1` 的 4 GiB 稀疏文件放在 `d0/`、`d1/`（宿主 `physical_block_size=512 minimum_io=512 write_cache=write_back`，与 `/sys/class/block/nvme0n1/queue/*` 逐字相同），跑 `target/release/first_transaction_on_device <d0/nvme0n1> <d1/nvme0n1> direct`（经 `run-with-memory-cap.sh 8G` 与 `capped.sh 3`），只截 `name=segments` 五行原样：

```
E7RESULT name=segments path=mkfs operations=23 segments=12+1+1+1+4 closed_form=4114 kinds=[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]|[root_record_fua]|[root_record_fua]|[system_configuration_slot×4,barrier×2]
E7RESULT name=segments path=instance_acquisition operations=2 segments=2 closed_form=4 kinds=[system_configuration_slot×2]
E7RESULT name=segments path=warm_up operations=18 segments=2+1+2+2+1+2 closed_form=15 kinds=[journal_record×2,barrier×4]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=transaction operations=33 segments=24+2+1+2 closed_form=16777223 kinds=[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
E7RESULT name=segments path=post_mkfs_stream operations=53 segments=2+2+1+2+2+1+26+2+1+2 closed_form=67108885 kinds=[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[unit_write×24,system_configuration_slot×2,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2]
```

逐行核对 `first_transaction_step_five_publish.rs`：mkfs 行 = 第 329/331–333 行断言 `(23, "12+1+1+1+4", 4114)` 与 `kinds` 串；instance_acquisition = 第 341/343–345 行 `(2, "2", 4)`；warm_up = 第 353/355–357 行 `(18, "2+1+2+2+1+2", 15)`；transaction = 第 365/367–369 行 `(33, "24+2+1+2", 16_777_223)`；post_mkfs_stream = 第 377/379–381 行 `(53, "2+2+1+2+2+1+26+2+1+2", 67_108_885)`。五行逐字相同，`name=done emitted=23`、退出码 0。这一步跑了两次（C554 落主工作区前后各一次，用的都是重编译出来的 release 二进制），两次结果逐字相同。跑完删了两块 4 GiB 稀疏文件。

## 每条新测试：改坏哪一行 → 哪条断言红（`show-me-test.md`）

### 测试 1：`the_boundary_marked_before_the_first_transaction_splits_warm_up_from_the_transaction`（新增在 `first_transaction_on_device.rs` 的 `mod tests` 里）

改坏：把 `first_transaction_paths` 函数体里 `&operations[mkfs_operation_count + 2..operation_count_before_first_transaction]` 与 `&operations[operation_count_before_first_transaction..]` 两处换回 `&operations[mkfs_operation_count + 2..operations.len() - 23]` 与 `&operations[operations.len() - 23..]`（即 `crates/mutations.tsv` 新增第一条变异）。哪条断言红：

```
thread 'tests::the_boundary_marked_before_the_first_transaction_splits_warm_up_from_the_transaction' panicked at crates/singlefs-harness/src/bin/first_transaction_on_device.rs:2428:9:
assertion `left == right` failed: mkfs / instance_acquisition / warm_up / transaction / post_mkfs_stream 五段的操作数：与 first_transaction_step_five_publish.rs 断言的五个数（该文件第 329、341、353、365、377 行）逐字相同
  left: [23, 2, 28, 23, 53]
 right: [23, 2, 18, 33, 53]
```

`left` 里 `warm_up=28、transaction=23` 正是投资告报告里查到的旧 bug 形态（事务前 10 次单元写被划进 warm_up）。

### 测试 2：`litmus_writer_threads_follow_the_recorded_publish_order` 新增的断言（原有测试，新加一条断言）

改坏：把 `&operations[pool.warm_up_operation_count..]` 换回 `&operations[operations.len() - 23..]`（`crates/mutations.tsv` 新增第二条变异）。哪条断言红：

```
thread 'litmus_writer_threads_follow_the_recorded_publish_order' panicked at crates/singlefs-harness/tests/publish_order_matches_litmus.rs:51:5:
assertion `left == right` failed: 第一个事务的步数写死错了就在这里先红：24 次单元写 + 2 道屏障 + 2 条 journal 记录 + 2 道屏障 + 1 次根槽 FUA + 2 次系统配置槽写，与 first_transaction_step_five_publish.rs 的断言（第 365 行 `(33, "24+2+1+2", 16_777_223)`）同一个数
  left: 23
 right: 33
```

这条新断言正是主 agent 验收要求的「补一条『步数写错就红』的断言」：没有它，`collapsed`（对相邻同类去重）看不出旧 bug——把 23 改错成任何数，只要落在 mkfs/warm_up 边界之后，`collapsed` 序列的种类形状不变（`unit_write→barrier→journal_record→barrier→root_record_fua`），这条测试原本会假绿（这也是它一直没被这次步数漂移抓住的原因）。

## 变异证红（`research/scripts/prove-red.sh`，不自己写证红脚本）

两条变异都写进了副本 `crates/mutations.tsv`（末尾追加，主表不动；追加内容见 `patch/mutations-append.tsv`）。命令与原样输出（在 C554 落主工作区、重新整仓 rsync 之后跑的最后一次，即最终状态）：

```
$ bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-gate55-steps/repo --memory 8G singlefs-harness \
    '里程碑「第二个事务」收尾批 55 号装置步数：first_transaction_paths 的暖机 / 事务分界又靠流尾往回数 23 步' \
    '里程碑「第二个事务」收尾批 55 号装置步数：litmus 绑定测试又从流尾往回数 23 步取事务窗口'
里程碑「第二个事务」收尾批 55 号装置步数：first_transaction_paths 的暖机 / 事务分界又靠流尾往回数 23 步	抓到	the_boundary_marked_before_the_first_transaction_splits_warm_up_from_the_transaction 红了（日志 …/prove-red-logs/001.log）
里程碑「第二个事务」收尾批 55 号装置步数：litmus 绑定测试又从流尾往回数 23 步取事务窗口	抓到	litmus_writer_threads_follow_the_recorded_publish_order 红了（日志 …/prove-red-logs/002.log）
✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了
```

退出码 0。这一对变异跑过 3 次（同一份命令）：C554 落地前的底座上跑了 2 次（第一次改完立即证红、修完 clippy 的生命周期问题之后又证了一次），整仓重新 `rsync` 到 C554 落地后的底座、重放全部编辑之后再跑了 1 次——3 次都两条「抓到」。每次跑完 `prove-red.sh` 自己把源文件写回原样并 `touch`，跑完逐次用 `grep -c "operations.len() - 23"` 核过两份文件里都是 0（还原干净）。

`crates/mutations.tsv` 里两处哪几行改坏对应哪条断言，与上一节「改坏哪一行 → 哪条断言红」一致（`prove-red.sh` 本质就是自动做那一步再判红）。33 号（`.claude/gate.d/33-mutation-tables.sh`）在追加之后跑过，见下「门禁阶段」一节，锚点各命中一次、没有重复行。

## 交回前的验证（在最终底座上跑的最后一遍）

### 动到的测试二进制（整个二进制）

```
$ cargo test --offline -p singlefs-harness --bin first_transaction_on_device
running 17 tests
... (17 个测试名逐行 ok)
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.08s

$ cargo test --offline -p singlefs-harness --test publish_order_matches_litmus
running 1 test
test litmus_writer_threads_follow_the_recorded_publish_order ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
```

`tests/common/mod.rs` 被 68 个测试文件 `mod common;` 引用，新增字段纯加法；`cargo build --offline --all-targets` 全绿（见下）覆盖了它们的编译，没有另外逐个跑这 68 个二进制——理由与代价在上面「为什么多改一个文件」一节。

### `cargo fmt --all -- --check`

```
$ cargo fmt --all -- --check
（无输出，退出码 0）
```

（中途 `cargo fmt --all -- --check` 先报过一次要重排的 diff，是我写的 `run_first_transaction` 闭包换行方式不合 rustfmt 默认宽度；跑了 `cargo fmt --all` 之后 `--check` 转绿，语义没变——闭包体逐字相同，只是缩进换行。）

### `cargo clippy --all-targets --all-features -- -D warnings` + code-discipline 那 7 条

```
$ cargo clippy --offline --all-targets --all-features -- -D warnings \
    -D clippy::wildcard_enum_match_arm -D clippy::allow_attributes_without_reason \
    -D clippy::cast_possible_truncation -D clippy::cast_sign_loss -D clippy::cast_possible_wrap \
    -D clippy::undocumented_unsafe_blocks -D clippy::shadow_unrelated
    Checking singlefs-core v0.1.0 (…)
    Checking singlefs-harness v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.40s
```

退出码 0。（中途 clippy 报过一次 `needless_lifetimes`：`first_transaction_paths<'operations>(...)` 的显式生命周期可以省略，改成不写生命周期参数之后转绿——这也是为什么最终代码没有显式 `'operations`，编译器自己推得出。）

### `cargo build --offline --all-targets`

```
$ cargo build --offline --all-targets
   Compiling singlefs-core v0.1.0 (…)
   Compiling singlefs-harness v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.20s
```

退出码 0。

## 门禁阶段归属表登记给 `implementation-writer` 的那几道

`awk` 现查得到 7 道：33-mutation-tables.sh、53-format-const-placeholders.sh、74-model-differential.sh、92-layout-checker-sync.sh、94-checker-implementation-disjoint.sh、93-feature-bits.sh、89-closeout-row27-preconditions.sh。逐道原样末行：

```
33-mutation-tables.sh（在副本里跑，含新追加的 2 条变异）exit=0
  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1155 条的原文各命中源码一次；……

53-format-const-placeholders.sh（在副本里跑）exit=0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23 已定项 19；C506；D22 已定项 1；C323）

92-layout-checker-sync.sh（副本没有 .git，本阶段自己判「不是 git 仓」跳过；在主工作区读专门跑了一遍）exit=0
  ✓ ……没抽到常量的格式定义路径 1 条……

94-checker-implementation-disjoint.sh（在副本里跑）exit=0
  ✓ checker 与实现只共享常量模块 singlefs-format（……）

93-feature-bits.sh（在副本里跑）exit=0
  ✓ feature bit 位号在记账表、D15 已定项 4 与代码三处一致（……）

89-closeout-row27-preconditions.sh（主工作区与副本各跑一遍，结果相同）exit=77
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（……）
```

74-model-differential.sh 见下一节单独说明（红，但坐实与本轮改动无关）。

## 74 号：红，但坐实与本轮改动无关

在最终底座上跑 `74-model-differential.sh`（release 下跑 `second_transaction_supplement_three_random_history.rs`），退出码 1：

```
failures:
    crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43
    random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
    rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline
test result: FAILED. 21 passed; 3 failed; 2 ignored; 0 measured; 0 filtered out; finished in 77.50s
```

隔离核实：把副本里 `first_transaction_on_device.rs` 与 `tests/common/mod.rs` 换成主工作区当前未改动的原样（即完全不带我这两处修改），再跑一次 74 号——同样 3 个失败、同样的用例名（`gate-74-isolation.log`）。这证明红与我这一轮改动无关：这两个测试文件我都没碰过，`second_transaction_supplement_three_random_history.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs` 都不在我的写范围与要动的文件清单里。核完把副本换回我的版本（`diff -q` 核对逐字节相同）。这几个用例名本身带着「known_red_form」「closeout_row_43」「abandoned_timeline」，像是 C554 乙那批还在收尾的改动留下的中间状态，不归我修，也没有去动。

## 没做什么

- 没走三方对抗、没提交；层 0 全量、QEMU、herd7、crates 变异整表（59 号）归 `crash-verifier`。
- 没跑 `.claude/gate.d/55-qemu-first-transaction.sh`、`vm-bench.sh`；没起虚机。
- 74 号红了但坐实与本轮无关（上一节），没有去改 `history.rs`/`model_comparison.rs`/`second_transaction_supplement_three_random_history.rs`。
- 投资告报告里「skip-first-transaction-barrier 那一档在新包装次序下 55 号那条判据还成不成立」那一条，仍然只列不改，留给主 agent 定（本报告没有新证据，只有投资告报告已给的两种次序推演）。
- `publish_order_matches_litmus.rs` 里没有再去动别的过期常量或断言（例如它自己 `writer_steps_of` 那一段读 litmus 文件的逻辑），因为验收范围只是这两处步数。
- 没有跑 68 个引用 `tests/common/mod.rs` 的测试二进制逐一验证；用 `cargo build --offline --all-targets` 全绿加「新字段纯加法、唯一构造点在 `build_pool()` 自己」的静态论证代替（见「为什么多改一个文件」一节）。
- 交回前删掉了草稿目录里的仓副本（`/tmp/claude-1000/impl-gate55-steps/repo/`，含它自己的 `target/`，约 18G）与所有临时文件；只留 `report.md`、`patch/`、几份门禁与 prove-red 的日志（`gate-*.log`、`fmt-*.log`、`clippy-*.log`、`build-all-targets*.log`）。

