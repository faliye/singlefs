# 实审 A4e：C545 准入先拒之后，清单外 3 条用例改期望

## 一、结论

`second_transaction_step_one_overwrite.rs`（1 条用例）与 `second_transaction_supplement_two_unequal_devices.rs`（2 条用例）改完期望之后，两份测试目标全绿：

- `cargo test -p singlefs-harness --test second_transaction_step_one_overwrite`：12 passed; 0 failed
- `cargo test -p singlefs-harness --test second_transaction_supplement_two_unequal_devices`：3 passed; 0 failed（含不受影响、原样保留的 `user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written`）

三条改期望的用例仍钉住它原来要钉的事：`SpaceAdmissionRefused` 都在 `try_overwrite` / 发布路径返回，前面 `records_before` / `fingerprint(&pool.allocator)` / `pool.stream.operations().len()` 的比对没有改动、断言原样留着，钉的仍是「拒在动分配器与任何写之前，盘上（录制流）逐字节不变、分配器不动」。

主表第 12、76、77、78、80、145、149 行逐条在副本上重证：**5 条仍红**（76、77、78、80、149），**2 条证不了红**（12、145）——原因见第四节，是 C545 准入先拒把这两条变异原来靠的代码路径挡在了更早的返回之前，不是这次改期望引入的新问题（同一件事在 A4d 报告第六节第 4 条已经点名交主 agent）。

一律没写主工作区：全部改动在副本 `/tmp/claude-1000/impl-rev-a4e/copy` 里做完，补丁在 `/tmp/claude-1000/impl-rev-a4e/patch/crates.patch`，`git apply --check` 在主工作区核过（见第五节）。

## 二、改了什么（副本里）

只改了两个文件（都在派发提示「要动的 crates 文件」清单里）：

- `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`
- `crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs`

`git diff --stat`（副本，只看这两个文件）：

```
 .../tests/second_transaction_step_one_overwrite.rs | 29 ++++++-----
 ...d_transaction_supplement_two_unequal_devices.rs | 56 ++++++++++++----------
 2 files changed, 48 insertions(+), 37 deletions(-)
```

`crates/mutations.tsv` 没有追加、没有替换：这次是重看已有 7 行的锚点是否仍红，不是新写变异（第四节）。没有新建 `admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs`（那是 A4d 已经落进主工作区、当前未提交的文件，`git status` 里显示 `??`，不是我写的，我没碰它）。

派发提示指名「别的会话正在改」的文件（`recovery.rs`、`mount.rs`、`lib.rs`、两个 `second_transaction_supplement_two_*` 之外的用例、`e158_root_choice_repair.rs`、`first_transaction_on_device.rs`、`publish_order_matches_litmus.rs`）一个都没碰，`git status --short` 里也没有出现在我的两处编辑之外。

## 三、每条改期望的用例：推导与钉值

三条用例的 `SpaceAdmissionRefused` 载荷都是从副本里实跑一次原样输出里读出来的（改期望之前先跑一次改坏前——也就是 A4d 落地之后、我改期望之前——的 `{result:?}`），不是推算的：

### 3.1 `publish_running_out_of_space_midway_leaves_the_allocator_as_it_was`

盘面：两块盘（`DeviceIdentity(0)`、`DeviceIdentity(1)`），除 50182–50183 外单元区全占。跑出来的原样：

```
Err(SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [
  DeviceShortOfDemand { device: DeviceIdentity(0), available: AvailableBytesOnOneDevice(0), demand: BytesOnOneDevice(8863744) },
  DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(0), demand: BytesOnOneDevice(8863744) }
] }))
```

两块盘对称（测试对两块盘做的是同一种占用），both `available=0`、`demand=8_863_744` 字节 = 541 槽（extent 根 1 + inode 叶容器 2 + inode 根 1 + ckpt_cost 537）。改成 `assert_eq!(refusal.short_devices, [...].to_vec(), ...)`。

### 3.2 `filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking`

小盘（`DeviceIdentity(1)`）单元区填满：

```
Some(SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [
  DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(0), demand: BytesOnOneDevice(32768) }
] }))
```

只报盘 1（大盘段外有富余，不短）；32768 字节 = 2 槽 = 这次的数据单元。

### 3.3 `devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written`

小盘数据单元那一对够、提交内生块那一判短：

```
Some(SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices { short_devices: [
  DeviceShortOfDemand { device: DeviceIdentity(1), available: AvailableBytesOnOneDevice(16384), demand: BytesOnOneDevice(7553024) }
] }))
```

16384 字节 = 1 槽（没挡的槽在数据单元取走那一对之后剩 1）；7553024 字节 = 461 槽（extent 根 1 + inode 叶容器 2 + inode 根 1 + ckpt_cost 457）。

三处都改成 `let Err(PublishError::SpaceAdmissionRefused(refusal)) = &result/&refused else { panic!(...) }` 加 `assert_eq!(refusal.short_devices, ...)`，与已有的 `admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` 同一写法（那份文件是 A4d 加的，我没改它，只是照它的风格写）。模块文档与每条测试上面的文档注释照实改（原文与新文一并见补丁）。

## 四、主表第 12、76、77、78、80、145、149 行：变异逐条重看

用 `bash research/scripts/prove-red.sh --copy /tmp/claude-1000/impl-rev-a4e/copy --memory 8G singlefs-harness <7 个变异名>`（经 `capped.sh 4`；先跑基线绿，逐条施加变异、经内存包装跑、判红、还原）。原样输出（`/tmp/claude-1000/impl-rev-a4e/prove-red-a4e.log`）。日志原本落在 `<副本>/prove-red-logs/`，交回前副本要删（第九节），已整份挪到 `/tmp/claude-1000/impl-rev-a4e/prove-red-logs-a4e/`，下面消息里的 `.../00N.log` 都在那里：

```
✗ 失败的发布不退回分配器	没红	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was 没判红（日志 .../prove-red-logs/001.log）
增补 2：用户数据落点只取盘 0 的答案（C368：小盘写满之后越界 panic）	抓到	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking 红了（日志 .../002.log）
增补 2：提交内生块开段与回落只取盘 0 的答案（C368：小盘越界 panic）	抓到	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking 红了（日志 .../003.log）
增补 2：各盘给提交内生块的去处不同也不拒（D3 已定项 8 待办 ① 没条款的分支被定成取盘 0）	抓到	devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written 红了（日志 .../004.log）
增补 2：提交内生块拒绝之前先关开放段（拒绝动了分配器）	抓到	devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written 红了（日志 .../005.log）
✗ 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，C368 仍欠的那一半）：发布层把分配器的落点拒绝一律报成「每块盘上都没有」	没红	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking 没判红（日志 .../006.log）
增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2i）：分配器用户数据那一处把「小盘写满」报成「每块盘上都没有」；等大的小盘走不到，不等盘那条用例判出	抓到	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking 红了（日志 .../007.log）
✗ 点名 7 条：跑了 7 条，跳过 0 条，有没红或无效的（逐条列在上面）
```

| # | 名（简称） | 结局 |
|---|---|---|
| 12 | 失败的发布不退回分配器 | **没红** |
| 76 | 增补 2：用户数据落点只取盘 0 | 抓到 |
| 77 | 增补 2：提交内生块开段回落只取盘 0 | 抓到 |
| 78 | 增补 2：各盘去处不同也不拒 | 抓到 |
| 80 | 增补 2：提交内生块拒绝前先关开放段 | 抓到 |
| 145 | 增补 3 第 2 件：落点拒绝一律报「每块盘上都没有」 | **没红** |
| 149 | 增补 3 第 2 件：小盘写满报成「每块盘上都没有」 | 抓到 |

**证不了红的两条，原因（读 001/006 号日志，两次都是 `... ok`，测试没有变红）：**

- **第 12 行**：变异删的是 `transaction.rs` 里「落盘失败时把 `allocator` 换回改之前那份拷贝」那一句（`*allocator = allocator_before_this_publish;`）。改期望之后，`publish_running_out_of_space_midway_leaves_the_allocator_as_it_was` 走的是准入先拒（`admit_the_units_landing_on_every_device`），这一判在 `settle_the_allocation_record_tree`（会真的改分配器拷贝）**之前**返回，分配器在这条路径上从来没被改过、也就没有「换回去」这一步要执行——删掉那一句这条用例看不出差别。这不是我这次改坏的：C545 准入先拒把「这次的单元落不落得下」挪到了动分配器之前，这条变异原来靠的正是「B 分配到数据单元、A 释放判定已经写进分配器拷贝」那一步，现在这条用例走不到那一步。A4d 报告第六节第 4 条已经把「这几条变异要不要换靶子」列成交主 agent 的问题，我没有在我的两个文件范围内找到能顶上这条变异的替代锚点（这条变异改的是 `transaction.rs` 通用的错误处理路径，不是这两个测试文件的内容，换靶子超出「要动的 crates 文件」清单）。
- **第 145 行**：变异改的是发布层把分配器的 `PlacementRefusal` 一律报成 `NoFreeSlotOnAnyDevice` 那一段代码（`refuse_a_publish_of_the_placement...` 附近，具体在锚点原文里）。`filling_the_smaller_device...` 改期望之后已经在准入先拒那一步交回 `SpaceAdmissionRefused`，根本不经过「把 `PlacementRefusal` 包成 `PlacementRefused` 那一段」，这段代码这条用例走不到，变异自然不改变行为。同一原因：C545 让这条用例的 `try_overwrite` 调用不再触达 145 行变异的靶子代码。

其余 5 行（76、77、78、80、149）改期望之后仍然抓到——它们的变异都在分配器层（`allocator.rs`）的 `try_allocate_user_data` / `try_allocate_commit_generated` 直接调用路径上，这两条测试里直接调分配器的那几段断言没有改（第二节没有触及），仍然是原来的锚点、原来的判据。

基线（不改源码，同一组过滤参数）都绿，没有基线红。


## 五、第 4 步那几样（副本里；线程上限 4，`capped.sh 4`；跑编出来的代码经 `run-with-memory-cap.sh 8G`）

开跑前 `ps` 看到别的会话的 cargo（`e158_root_choice_repair` 的 release 测试），没有 qemu / fio / vm-bench / e152，`nice -n 19` 照跑。

**动到的测试二进制（整个二进制）：**

```
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.45s   # second_transaction_step_one_overwrite
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s     # second_transaction_supplement_two_unequal_devices
```

**第 3 步证红**：见第四节 prove-red.sh 原样输出（5 抓到、2 证不了红并写明原因）。

**`cargo fmt --all -- --check`**：exit 0，无输出。

**`cargo clippy --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS`**（`-D clippy::wildcard_enum_match_arm -D clippy::allow_attributes_without_reason -D clippy::cast_possible_truncation -D clippy::cast_sign_loss -D clippy::cast_possible_wrap -D clippy::undocumented_unsafe_blocks -D clippy::shadow_unrelated`，从 `.claude/singlefs-ai-sop/scripts/check.sh` 现抄，没跑 `check.sh` 本身）：

```
    Checking singlefs-core v0.1.0 (.../copy/crates/singlefs-core)
    Checking singlefs-harness v0.1.0 (.../copy/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.19s
CLIPPY_EXIT=0
```

**`cargo build --offline --all-targets`**：

```
   Compiling singlefs-core v0.1.0
   Compiling singlefs-harness v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.82s
BUILD_EXIT=0
```

**阶段归属表登记给 `implementation-writer` 的门禁阶段**（`awk` 现查，见下）：`33-mutation-tables.sh`、`53-format-const-placeholders.sh`、`74-model-differential.sh`、`92-layout-checker-sync.sh`、`94-checker-implementation-disjoint.sh`、`93-feature-bits.sh`、`89-closeout-row27-preconditions.sh`。都在副本里跑（副本临时接了一份 `.git`，只为了让 92/74 号能算出 git diff 范围；两个重活相关文件位置见「没做什么」一节的说明）：

```
  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1139 条的原文各命中源码一次；……                    # 33，exit 0
  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位……）                                                                                    # 53，exit 0
  ✓ checker 与实现只共享常量模块 singlefs-format（……）                                                                                     # 94，exit 0
  ✓ feature bit 位号在记账表、D15 已定项 4 与代码三处一致（……）                                                                             # 93，exit 0
  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（……）                                                                     # 89，exit 77
  ✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15 已定项 4 的登记表；这次改动比 faf255e2……，111 个格式常量里变了 0 个……                              # 92，exit 0
```

**74-model-differential.sh**（release 跑 `second_transaction_supplement_three_random_history`，exit 非 0）：

```
test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 38.12s
  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
     红的两条：crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43
              random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation
```

**这两条红不是这一轮带来的**：这个测试文件我一个字都没碰（不在我改的两个文件里），A4d 报告第五节已经记过同名的基线红（"74 号红，不是这一轮带来的...红的同样是 crash_recovery_abandoning...closeout_row_43 与 random_histories_fast_tier_end_only..."），是 A4d 落地之后就已经存在、尚未提交的已知红（A4c 报告第七节也记过）。我没有去修它——按共用约束「红了先看它点名的文件与行在不在这一轮的改动里；不在就照实说『红了，但不是这一轮的』，别顺手去修」，这条不归我这次的验收范围。


`awk` 现查命令与原样输出：

```
$ awk -F'\t' -v me="implementation-writer" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv
33-mutation-tables.sh
53-format-const-placeholders.sh
74-model-differential.sh
92-layout-checker-sync.sh
94-checker-implementation-disjoint.sh
93-feature-bits.sh
89-closeout-row27-preconditions.sh
```

## 六、4a：改了 checker 的不变量判定集合吗

没有。这一轮只改了两个测试文件的期望值与文档注释，没有碰 `crates/singlefs-checker/src/` 下任何文件，不用列「受影响的层 0 流与崩溃枚举用例」这一节。

## 七、补丁交付

`/tmp/claude-1000/impl-rev-a4e/patch/crates.patch`：`git diff -- crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs`（副本里执行）。**只含这两个文件**，不含主工作区里其它会话尚未提交的改动（A4d 的 `admission.rs`/`allocator.rs`/`transaction.rs`/`mutations.tsv` 等改动已经落进主工作区，不重复打进这份补丁）。

在主工作区核过：

```
$ git apply --check /tmp/claude-1000/impl-rev-a4e/patch/crates.patch
APPLY_CHECK_OK
```

没有 `mutations-append.tsv`、`mutations-replacements.tsv`、`mutations-delete.txt`：这一轮没有新增变异、没有改主表任何一行（主表第 12、76、77、78、80、145、149 行原样保留，重看结果见第四节），也没有要删的变异名，所以这三份补丁文件都没建（用不上的不建）。

## 八、这一轮写过的文件

- `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`（副本里）
- `crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs`（副本里）
- `/tmp/claude-1000/impl-rev-a4e/patch/crates.patch`
- `/tmp/claude-1000/impl-rev-a4e/report.md`（本文件）

`git diff --stat -- crates litmus`（副本，全量，含别的会话未提交的改动，分不出谁改的，原样贴）：

```
 crates/mutations.tsv                               |   40 +-
 crates/singlefs-core/src/admission.rs              |  405 +-
 crates/singlefs-core/src/allocator.rs              |  258 +-
 crates/singlefs-core/src/transaction.rs            |  125 +-
 .../src/bin/e156_allocation_basis_counts.rs        |   10 +-
 .../src/bin/e158_root_choice_repair.rs             | 6676 +++++++++++++++++++-
 .../admission_checkpoint_cost_per_device_paths.rs  |  539 +-
 ...hecker_narrow_invariants_and_abandoned_roots.rs |    8 +-
 ...n_admission_raises_the_floor_before_refusing.rs |  276 +-
 ...inside_the_floor_raise_pushed_by_the_session.rs |   14 +-
 .../tests/second_transaction_step_five_reuse.rs    |    2 +-
 .../tests/second_transaction_step_one_overwrite.rs |   29 +-
 ...d_transaction_supplement_two_unequal_devices.rs |   56 +-
 13 files changed, 7909 insertions(+), 529 deletions(-)
```

只有最后两行（`second_transaction_step_one_overwrite.rs` +29/-、`second_transaction_supplement_two_unequal_devices.rs` +56/-）是这一轮写的；其余是主工作区已有、还没提交的别人的改动（A4d 已落地的补丁与别的会话在 `e158_root_choice_repair.rs` 上的活）。


## 九、没做什么

- 没走三方对抗；没提交；没碰 `.claude/agents/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`。
- 没有新增测试，因此没有第 3 步「改坏代码证红」那种新测试证红流程；三条改期望的用例走的是第四节的变异重看（`prove-red.sh`）。
- 层 0 全量、QEMU、herd7、crates 变异整表复跑、全量 `cargo test --all`、`check.sh`/`gate.sh` 整轮都没跑，留给提交时的 `crash-verifier` 与整轮门禁。
- 门禁 74 号（`second_transaction_supplement_three_random_history`）报的两条红，不是这一轮引入、我也没有修（第五节）。
- 主表第 12、145 两行变异证不了红，原因见第四节；我没有在 mutations.tsv 里改这两行、也没有新找靶子顶替——换靶子涉及别的测试文件或 `transaction.rs` 的通用错误路径，超出这一轮「要动的 crates 文件」清单，交主 agent／`mutation-triage` 定。
- 92-layout-checker-sync.sh、74-model-differential.sh 依赖 git 记录来算改动范围，副本本来是 `rsync -a --exclude target --exclude .git`（92 号在纯净副本上报 `不是 git 仓，本阶段跳过`，exit 77）；为了能真的跑这两道，我又把主工作区当前的 `.git`（115M）整份 rsync 进了副本——这样副本上的 git 历史与主工作区一致，`git diff` 对得上主工作区当前的未提交状态。这一步不算「写主工作区」（只读 `.git` 内容拷了一份到副本），但记在这里，因为它和「不写主工作区」的字面意图有一点距离，供主 agent 核。

## 十、清理（交回前）

副本 `/tmp/claude-1000/impl-rev-a4e/copy`（含后来补进去的 `.git`、`target` 由 `CARGO_TARGET_DIR` 另指到 `/tmp/claude-1000/impl-rev-a4e/target-copy`）与编译目录在交回前删除；`prove-red-logs` 已整份挪到 `/tmp/claude-1000/impl-rev-a4e/prove-red-logs-a4e/`（第四节），`gate-*.log`、`prove-red-a4e.log`、`patch/crates.patch`、`report.md` 都在副本之外，留着。
