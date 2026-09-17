# 崩溃一致性验证员 —— 重新试跑报告（第 6 步：跨阶段指纹）

试跑范围：`crates/`、`litmus/` 工作区现状（别的会话未提交的改动）。只跑 55、57、59 号，
不跑 54 号（上一次试跑已量过约 47 分钟）。全程只读，未改仓里任何文件。

## 开跑前状态

`ps aux | grep -E "cargo|qemu-system|herd7|gate.sh" | grep -v grep`（2026-09-17 06:33 UTC 左右）：

```
fy5090    231695  0.0  0.0 240852 24656 ?        SNl  05:52   0:00 /home/fy5090/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo test --release -p singlefs-harness --test second_transaction_step_zero_layer0 -- --include-ignored --nocapture
```

按主 agent 给的例外（定义第 1 步）：本机有 cargo 在跑，允许照常跑（加 `nice -n 19`），
不停下。之后每次开跑一个阶段之前又各查了一次 `ps`，见下面「阶段记录」逐阶段的记录——
定义里没写清「开跑前 ps」是整个任务一次还是每阶段一次，这次按更保守的「每阶段一次」做，
详见文末「试跑观察」。

`git diff --stat -- crates litmus`（开跑时，2026-09-17 06:33 UTC）：

```
 crates/mutations.tsv                               |  42 +-
 crates/singlefs-checker/src/image.rs               |  14 +-
 crates/singlefs-checker/src/walk.rs                |  99 ++-
 crates/singlefs-core/src/allocator.rs              | 341 ++++++++-
 crates/singlefs-core/src/lib.rs                    |   2 +
 crates/singlefs-core/src/recovery.rs               | 380 +++++++++-
 crates/singlefs-core/src/transaction.rs            | 835 +++++++++++++--------
 .../src/bin/first_transaction_on_device.rs         | 164 +++-
 crates/singlefs-harness/src/scenario.rs            |   4 +-
 .../tests/checker_known_bad_images.rs              |  46 ++
 crates/singlefs-harness/tests/common/mod.rs        |  26 +-
 .../tests/first_transaction_step_five_publish.rs   |  29 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   6 +-
 .../tests/first_transaction_step_six_recovery.rs   |   5 +-
 .../tests/first_transaction_step_two_data_unit.rs  |   4 +-
 .../tests/second_transaction_step_one_overwrite.rs |  40 +-
 .../tests/second_transaction_step_zero_layer0.rs   | 286 ++++++-
 17 files changed, 1918 insertions(+), 405 deletions(-)
```

## 环境（供参考，本轮三个阶段都没有判「环境」）

```
$ command -v herd7; echo "exit=$?"
exit=1
$ ls -l /dev/kvm; echo "exit=$?"
crw-rw---- 1 root kvm 10, 232 Sep 17 05:41 /dev/kvm
exit=0
$ command -v qemu-system-x86_64; echo "exit=$?"
/usr/bin/qemu-system-x86_64
exit=0
```

`herd7` 不在默认 `PATH` 里（装在 `/home/fy5090/.opam/default/bin/herd7`），
但 `.claude/gate.d/57-lkmm.sh` 调用的 `.claude/scripts/lkmm.sh` 自己有兜底：
`command -v herd7` 找不到时会 `eval "$(opam env ...)"` 再探测一次，实测找得到、跑得通。
**只凭 `command -v herd7` 的原样输出会误判「缺 herd7」**——这次没有这样判，是先读了阶段脚本源码才知道有兜底逻辑，
细节见文末「试跑观察」第 6 条。

## 阶段记录

### 57 号 —— 内存序（herd7 + litmus/）

`ps` 开跑前（06:34 左右）：仍只有上面那条 cargo test，没有 qemu-system / herd7 / gate.sh。

指纹（开跑前，2026-09-17T06:34:37Z）：
```
head=5f9e449d4b6b90c17309a35483082133e570dfe8
diff_sha256=1eccd94684b20f3a71917429fd035d7ed00e1c82c6280b3964f9168bd70b4f53
untracked_sha256=bf8023cc0c2d3c6d9687b6133e6550df0fd8f8431c8c0416031c554117b6ec7b
```

开跑：`nice -n 19 bash .claude/gate.d/57-lkmm.sh`，前台跑，`$?` 直接拿到。
开始 2026-09-17T06:34:52Z，结束 2026-09-17T06:34:52Z（同一秒内完成），**退出码 0**。

原样输出（全文）：
```
══ LKMM（herd7） ══
  ✓ 内核树 /home/fy5090/linux-bug-fix/linux
  ✓ herd7  7.58, Rev: exported

  ✓ litmus/commit-publish-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/commit-publish.litmus  Never（符合声明）
  ✓ litmus/first-txn-journal-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-journal-implies-units.litmus  Never（符合声明）
  ✓ litmus/first-txn-root-implies-units-nofence.litmus  Sometimes（符合声明）
  ✓ litmus/first-txn-root-implies-units.litmus  Never（符合声明）

  ✓ LKMM 通过（3 条 Never：每条都有内容对得上的对照组，2 条绑到代码、1 条声明不对应代码；共 3 条 Sometimes）
  ✓ 内存序：litmus/ 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符
```

计数行：`✓ LKMM 通过（3 条 Never：每条都有内容对得上的对照组，2 条绑到代码、1 条声明不对应代码；共 3 条 Sometimes）`。

指纹（结束后，2026-09-17T06:35:00Z）：
```
head=5f9e449d4b6b90c17309a35483082133e570dfe8
diff_sha256=1eccd94684b20f3a71917429fd035d7ed00e1c82c6280b3964f9168bd70b4f53
untracked_sha256=bf8023cc0c2d3c6d9687b6133e6550df0fd8f8431c8c0416031c554117b6ec7b
```
与开跑前一致，未变。

### 59 号 —— crates 变异表复跑

`ps` 开跑前（06:35 左右）：新增一条别的会话起的 `cargo build --release --manifest-path e7-index-bench/Cargo.toml`，
仍没有 qemu-system / herd7 / gate.sh；按例外照常跑。

指纹（开跑前，取指纹的那条命令本身在 2026-09-17T06:35:10Z，但实际启动阶段的命令是紧接着的
下一次 Bash 调用，`date -u` 显示真正 `nohup` 起来的时刻是 2026-09-17T06:35:16Z——
两条命令之间隔了约 6 秒，这个缺口后来被证明不是空等，见下面「与 55/57 对比」）：
```
head=5f9e449d4b6b90c17309a35483082133e570dfe8
diff_sha256=1eccd94684b20f3a71917429fd035d7ed00e1c82c6280b3964f9168bd70b4f53
untracked_sha256=bf8023cc0c2d3c6d9687b6133e6550df0fd8f8431c8c0416031c554117b6ec7b
```

开跑：`nohup nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh > 59-output.log 2>&1 &`
（后台跑，用写死的 pid `kill -0` 等待收尾，不用 `pgrep -f`）。
开始 2026-09-17T06:35:16Z，结束 2026-09-17T06:36:16Z（约 1 分钟），
**退出码 0（推断，非直接捕获，见「试跑观察」第 4 条）**：输出以脚本自己的成功行收尾，
脚本里唯一的失败出口都会先打印 `✗ ...` 再 `sys.exit(1)`，本轮输出里一处都没有。

原样输出（56 条逐条，全文见 `59-output.log`；这里抄最后一行计数与前 3 行示例）：
```
  ✓ 逐盘核退回只核总数：placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even 红了
  ✓ 同盘槽号不核唯一：two_records_for_the_same_slot_on_the_same_device_are_rejected 红了
  ✓ 空闲不随分配减（I-5.2 要红）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ……（共 56 条，每条都是「✓ <变异名>：<测试名> 红了」）
  ✓ crates 变异表复跑：56 条变异各自红在点名的测试上（原文都恰好命中一次）
```

计数行：`✓ crates 变异表复跑：56 条变异各自红在点名的测试上（原文都恰好命中一次）`。

指纹（结束后，2026-09-17T06:36:58Z）：
```
head=5f9e449d4b6b90c17309a35483082133e570dfe8
diff_sha256=e7f0987db4310fe823e155f161c7d0c447aef91a1076b53564cb5082e164030e
untracked_sha256=bf8023cc0c2d3c6d9687b6133e6550df0fd8f8431c8c0416031c554117b6ec7b
```

**`diff_sha256` 变了**（`1eccd9…` → `e7f0987…`），`head` 与 `untracked_sha256` 没变。
`find crates litmus -newer 57-output.log -type f` 定位到三个文件改动，mtime 都是 06:35:11 UTC：
`crates/mutations.tsv`、`crates/singlefs-checker/src/walk.rs`、
`crates/singlefs-harness/tests/checker_known_bad_images.rs`。
`git diff --stat` 前后对比：`mutations.tsv` 42 行 → 43 行、`checker_known_bad_images.rs` 46 行 → 66 行、
`walk.rs` 总行数不变但内容变了（mtime 变了）。这是**另一个会话在这段时间里改了 `crates/`**，
不是本次试跑动的。

**⇒ 这几个绿对应的不是同一份源码**：57 号的判定对应的是 06:35:11 UTC 之前的 `crates/`；
59 号（以及下面的 55 号）对应的是 06:35:11 UTC 之后、含那三个文件改动的 `crates/`。
57 号与 59/55 号之间不是同一次源码快照上跑出来的结果。

### 55 号 —— QEMU 真设备上的第一个事务

`ps` 开跑前（06:37 左右）：别的会话又换了一条 `cargo test --release -p singlefs-harness
--test first_transaction_step_seven_layer0`，仍没有 qemu-system / herd7 / gate.sh 本身在跑；按例外照常跑。

指纹（开跑前，2026-09-17T06:37:34Z）：
```
head=5f9e449d4b6b90c17309a35483082133e570dfe8
diff_sha256=e7f0987db4310fe823e155f161c7d0c447aef91a1076b53564cb5082e164030e
untracked_sha256=bf8023cc0c2d3c6d9687b6133e6550df0fd8f8431c8c0416031c554117b6ec7b
```
与 59 号结束后的指纹一致——59 号与 55 号之间源码没有再变。

开跑：`nohup nice -n 19 bash .claude/gate.d/55-qemu-first-transaction.sh > 55-output.log 2>&1 &`，
后台跑，写死 pid `kill -0` 等待。开始 2026-09-17T06:37:41Z，结束 2026-09-17T06:37:56Z（约 15 秒），
**退出码 0（推断，理由同 59 号：脚本唯一的失败出口 `fail()` 会先 `echo "  ✗ …"` 再 `exit 1`，
本轮一次都没触发，脚本以成功行收尾、无显式 `exit`，走的是 `echo` 的返回码 0）**。

原样输出（全文）：
```
  ✓ QEMU 真设备上的第一个事务：3 次虚机跑、18 项检查全过；direct 设备侧逐项对得上、两个对照都红在该红的地方
```

计数行：`✓ QEMU 真设备上的第一个事务：3 次虚机跑、18 项检查全过；direct 设备侧逐项对得上、两个对照都红在该红的地方`。

指纹（结束后，2026-09-17T06:38:08Z）：
```
head=5f9e449d4b6b90c17309a35483082133e570dfe8
diff_sha256=e7f0987db4310fe823e155f161c7d0c447aef91a1076b53564cb5082e164030e
untracked_sha256=bf8023cc0c2d3c6d9687b6133e6550df0fd8f8431c8c0416031c554117b6ec7b
```
与开跑前一致，未变。

## 汇总表

| 阶段 | 退出码 | 耗时 | ✓ / ✗+→ | 计数行 |
|---|---|---|---|---|
| 57 内存序（herd7 + litmus/） | 0（直接捕获） | <1 秒（06:34:52–06:34:52 UTC） | 全 ✓，见上 | 3 条 Never（2 条绑代码、1 条不对应代码）、3 条 Sometimes |
| 59 crates 变异表复跑 | 0（推断） | 约 1 分钟（06:35:16–06:36:16 UTC） | 全 ✓，见上 | 56 条变异各自红在点名的测试上 |
| 55 QEMU 真设备上的第一个事务 | 0（推断） | 约 15 秒（06:37:41–06:37:56 UTC） | 全 ✓，见上 | 3 次虚机跑、18 项检查全过 |

三阶段都不是 77（本次未跑），也没有一个是「作废」（判定行都读到了）。
**但 57 号与 59/55 号不是同一份源码上跑出来的**——见上面「⇒ 这几个绿对应的不是同一份源码」。

## 没做什么

- 没跑 54 号（层 0 崩溃点重放）：这次试跑的例外明确排除它，上一次试跑已单独量过（约 47 分钟）。
- 没修任何一处红（本轮三个阶段都没有红）；也没有判红是不是这一轮改动造成的，因为没有红需要判。
- 没跑 `gate.sh` 全量、没跑别的门禁阶段（不归这次试跑）。
- 没有判定「环境缺失」：herd7 / KVM / qemu-system-x86_64 三样命令输出已贴在开头，但三个阶段都成功，不构成「环境」判定的场合。
- 全绿只说明这三道的证据要求满足；层 0 覆盖到哪几条流、哪些没进来，看 54 号头部，这次没有重新核实（沿用上一次试跑对 54 号的记录，本轮不重复）。
- 没有对比这次三阶段的判定内容与上一次试跑（若有）是否一致——这次任务只要求验证第 6 步指纹机制，不是复核判定结论本身。

## 试跑观察

**1. 第 6 步的指纹命令能照写跑通。** `git rev-parse HEAD`、`git diff HEAD -- crates litmus | sha256sum`、
`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`
三条命令这次每次都顺利跑完，输出稳定（64 位十六进制），没有报错，也没有因为 `-z`/`xargs -0` 的空输入情形
（本轮 untracked 一直是空集）出问题——`xargs -0 -r` 的 `-r` 在没有输入时不会去跑 `sha256sum`（避免读 stdin 卡住），
外层再套一次 `sha256sum` 对空输入也能稳定给出同一个哈希（`bf8023cc…`，全程未变）。

**2. 三份指纹在阶段之间确实变了一次，机制被验证有效。** 57 号（开跑前 06:34:37Z、结束后 06:35:00Z）
的 `diff_sha256` 一直是 `1eccd9…`；59 号开跑前 06:35:10Z 读到的还是 `1eccd9…`，
但 59 号结束后 06:36:58Z 已经是 `e7f0987…`；55 号开跑前 06:37:34Z 与结束后 06:38:08Z 都是 `e7f0987…`。
`head` 与 `untracked_sha256` 全程没变。定点排查（`find crates litmus -newer <文件> -type f`
配合各文件 mtime）确认是另一个会话在 06:35:11 UTC 左右改了 `crates/mutations.tsv`、
`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`
三个文件——这正是定义第 6 步要拦的场景：57 号的判定对应改动前的源码，59/55 号对应改动后的源码，
按定义要求写明「这几个绿对应的不是同一份源码」（已写进上面「阶段记录」与「汇总表」）。

**3. 指纹命令与实际启动阶段命令之间如果隔成两次 Bash 调用，中间的空窗会被别的会话钻进去。**
这次给 59 号取「开跑前」指纹（06:35:10Z）与真正 `nohup` 起 59 号脚本（06:35:16Z，从这条命令自己
打的 `date -u` 读出）隔了两次独立的 Bash 工具调用、约 6 秒；恰好那三个文件的改动落在 06:35:11Z，
正好卡在这 6 秒缺口里。结果是：59 号「开跑前」记下的指纹（`1eccd9…`）其实已经不是 59 号脚本
真正读到的源码状态（脚本内部 `shutil.copytree` 发生在它自己启动之后，此时源码已经是 `e7f0987…`）。
本轮巧合的是「开跑前」指纹与「上一阶段结束后」指纹一样（都是 `1eccd9…`），所以看起来像是在
59 号执行期间变的；但严格说，这次记录的「59 号开跑前」指纹本身就已经不准，只是误差方向没有
影响到本轮要验证的结论（源码确实在 57 号与 59/55 号之间变了）。
**⇒ 定义没写清楚「开跑」指纹要不要和启动阶段的命令合并成一条**；建议把取指纹的命令和
`nohup nice -n19 bash .claude/gate.d/<文件> & echo $!` 写进同一次 Bash 调用，减小这个窗口
（哪怕合并了还是有窗口，但比拆成两次调用小得多）。

**4. 后台跑的阶段拿不到真实退出码。** 59 号与 55 号都用了 `nohup … & ; echo bg_pid=$!` 起在后台、
再用写死 pid 的 `kill -0` 轮询等待收尾——这是为了不在一次 Bash 调用里空等超过上限、也不用
`pgrep -f`（会匹配到自己的命令行）。但这样一来，`nohup … &` 那条外层命令本身的 `$?` 只是
「后台任务启动成功」这件事的退出码，不是阶段脚本真正跑完的退出码；等它结束之后已经没有
任何 shell 变量能读到那个 `$?` 了。这次是靠读输出内容反推：两个阶段的脚本都只有两种收尾——
触发 `fail()`（或等价的 `✗ …` + 退出非 0）、或者走到最后一行的成功 `echo`（隐含退出码 0，因为脚本
里再没有别的语句）——本轮两次都读到了成功行、没有 `✗`，所以退出码是 0。但这是**推断，不是直接
读数**，与 57 号（前台跑、`echo "exit_code=$?"` 直接拿到）不是同一种确定性。
**⇒ 定义写「后台跑、结束后读输出」，没说清「读输出」是否等价于「记退出码」**；建议后台起的阶段
改成 `{ nice -n19 bash .claude/gate.d/<文件>; echo $? > <某个文件>; } &`，结束后 `cat` 那个文件拿
真实退出码，不必靠「有没有 ✗」反推。

**5. herd7 不在默认 `PATH`，但这不构成「环境」判定。** `command -v herd7` 单独跑退出码是 1，
如果只按定义第 5 条字面「缺 herd7……判环境」去看这一条命令的输出，会显得像是环境缺失；
但 `.claude/scripts/lkmm.sh` 自己在探测不到时会 `eval "$(opam env …)"` 再探测一次，实测能找到
（`/home/fy5090/.opam/default/bin/herd7`），阶段本身跑得通、判定也正常给出。这次没有误判成
「环境」，是因为动手前先读了阶段脚本源码；**如果只按定义第 5 条列的三条命令的原样输出下判断，
会把「阶段自己有兜底、能跑通」的情形误报成「缺 herd7」**。定义第 5 条其实已经要求同时贴
「阶段自己的报错」，只是没有明说「三条命令都不代表最终结论，要以阶段实际跑的结果为准、
三条命令只是辅助材料」——这次的做法是三条命令原样贴出来但不据此下判断，等阶段真正跑完再看。

**6. `ps` 检查的粒度定义没写清楚。** 定义第 1 步只写「开跑前 `ps` 看……有就报……停下」，没说是
「整个任务开工前查一次」还是「每个要跑的阶段各查一次」。这次因为主 agent 给了例外（允许 cargo
跑），停不停已经不是问题，但为了不漏看 qemu-system / herd7 / gate.sh 这三样真正会撞车的进程，
这次选了更保守的做法：任务开工前查一次，之后每个阶段开跑前又各查一次（一共 4 次），
每次都只看到别的会话在跑不同的 `cargo` 命令，没有 qemu-system / herd7 / gate.sh；额外的三次
`ps` 调用耗时都在一秒以内，没有额外成本。

**7. `一次只跑一个` 与写死 pid 等待可以两条一起满足，没有冲突。** 三个阶段全部严格串行——
每一个都等到它的 `kill -0 <pid>` 循环退出、确认进程真正结束之后，才取「结束后」指纹、
再进入下一个阶段的「开跑前」指纹与启动。用的是 `command-safety.md` 允许的写死 pid 形式，
没有用 `pgrep -f`（那条规则明令会匹配到自己的命令行、可能永远不退出）。

## 附录：59 号完整原样输出（未截断，共 57 行）

```
  ✓ 逐盘核退回只核总数：placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even 红了
  ✓ 同盘槽号不核唯一：two_records_for_the_same_slot_on_the_same_device_are_rejected 红了
  ✓ 空闲不随分配减（I-5.2 要红）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ✓ 释放时清位图（立即复用）：released_placements_are_not_handed_out_again_before_reclaim_exists 红了
  ✓ 分配记录树装不下不判：repeated_overwrites_report_a_full_allocation_node_instead_of_panicking 红了
  ✓ 释放退回按提示：release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released 红了
  ✓ 映射查到 key 就算（落点指错不核记录）：release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking 红了
  ✓ 失败的发布不退回分配器：publish_running_out_of_space_midway_leaves_the_allocator_as_it_was 红了
  ✓ 内容超长不判：content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched 红了
  ✓ oracle 只比 txg 不比实例：landing_on_the_lower_instance_of_the_same_txg_is_a_violation 红了
  ✓ oracle 只按 txg 找版本：the_same_txg_from_two_instances_are_two_versions 红了
  ✓ oracle 放过没版本的更新根：newer_root_without_any_version_reporting_no_file_is_violation 红了
  ✓ Ignore 那一遍恢复不过 oracle：targeted_controls_on_the_second_publish_go_red_where_they_should 红了
  ✓ 步 1 变异：不释放旧落点：release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue 红了
  ✓ 步 1 变异：defer 行写 0：release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue 红了
  ✓ 步 1 变异：事务号不加一：overwrite_publishes_the_second_version_through_the_same_commit_shape 红了
  ✓ 步 1 变异：改动计数留 1：overwrite_publishes_the_second_version_through_the_same_commit_shape 红了
  ✓ 步 2 变异：已分配行不随分配更新（I-3.1 要红）：cold_start_reads_the_second_content_and_the_pool_checker_stays_green 红了
  ✓ 步 2 变异：忘了改写分配记录：release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue 红了
  ✓ 步 3：不写行（实例表照抄）：remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version 红了
  ✓ 步 3：暖机只推一次：damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device 红了
  ✓ 步 3：前缀跨实例边界（把别的实例的记录也接上）：layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape 红了
  ✓ 步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）：stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row 红了
  ✓ 步 3：I-3.8 判定恒真：checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root 红了
  ✓ 步 3：重建分配器时不把已释放的放进 defer 队列：remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version 红了
  ✓ 步 4：回退行不带回退位：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：只写被退回的实例那一行、不写中间实例行：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1，盖掉被抛弃发布的记录槽）：rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation 红了
  ✓ 步 4：影子账一个槽都不隔离：without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit 红了
  ✓ 步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）：rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused 红了
  ✓ 步 4：前缀第五条不判（回退行的 W 不封顶）：the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water 红了
  ✓ 步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 5：回收不看释放代（复用窗口置 0）：remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version 红了
  ✓ 步 5：抬 F 的上限不看第 4 新的非空根：raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused 红了
  ✓ 步 5：抬 F 的空发布只推一次（F 只落在一块盘上）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：复用时追加记录而不改写（同盘同槽两条）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：checker 的候选集不按 F 收（F 之下的根照走）：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：F_生效 取各盘 F 最大值的最大值而不是最小值：one_device_carrying_the_floor_alone_does_not_take_effect_on_remount 红了
  ✓ 步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条：torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg 红了
  ✓ 步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）：rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation 红了
  ✓ 步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效：roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates 红了
  ✓ 步 5：回收门槛不看环里最旧有效根：reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root 红了
  ✓ 步 4：影子账把被抛弃根账里已释放的落点也隔离：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：重建分配器时不认第 0 版树表单元（另一个进程里的第一个文件版本漏释放它）：rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released 红了
  ✓ 步 4：影子账不豁免候选根引用的槽（保守读法）：rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content 红了
  ✓ 步 4：被抛弃根的树表读不出时不计数：torn_tree_table_of_an_abandoned_root_is_counted_and_does_not_fail_the_mount 红了
  ✓ 步 5：抬 F 之后不按新候选集重算影子账：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：影子账的豁免不看候选根的 txg 是否 ≥ F：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：抬 F 回收的槽不扣住、生效之前就能发出去：slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect 红了
  ✓ 步 5：抬 F 生效之后不放开扣住的槽：raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it 红了
  ✓ 步 5：没做过可写挂载的进程抬 F 时 panic 而不是报错：raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking 红了
  ✓ 步 5：抬 F 重算影子账时读不出账的被抛弃根不计数：raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable 红了
  ✓ 步 5：提交内生块的 bump 游标不绕开开段之后才隔离或扣住的槽：commit_generated_bump_skips_a_slot_isolated_after_the_segment_was_opened 红了
  ✓ 步 6：I-7.4 不判候选根引用的单元被复用或抹头：the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target 红了
  ✓ 步 6：I-4.8 只看最新根、不看别的候选根：the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target 红了
  ✓ 步 6：I-7.4 不在最新根上判、只看更早的候选根：reused_unit_referenced_only_by_the_newest_root_makes_the_reuse_invariant_red 红了
  ✓ crates 变异表复跑：56 条变异各自红在点名的测试上（原文都恰好命中一次）
```
