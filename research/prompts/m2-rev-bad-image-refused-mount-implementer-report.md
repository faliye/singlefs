# 实现员报告：I-3.10 那份坏镜像的第 ③ 步改钉「可写挂载在任何写之前被拒」

日期：2026-09-27。底座：主工作区现状，副本 `rsync -a --exclude target --exclude .git` 拷出，只改副本，交补丁。

## 结论

- `crates/singlefs-harness/tests/checker_known_bad_images.rs` 里 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount` 的 ①② 两步（checker 在挂载之前只红 I-3.10、别的判定与干净镜像逐项相同、红在分配代 5 / 诞生代号 4）一字没动；第 ③ 步从 `.expect("坏镜像照样可写挂载")` 加挂载后再判改成：
  - `mount_writable` 必须返回 `Err`，`Ok` 时 panic 并打出择根 txg、现行版 txg、施加前缀；
  - 错误成员必须是 `MountError::Recovery(RecoveryFailure::InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })`（`matches!` 逐字钉两段字符串）；
  - 挂载前后 `memory_pool_of_sparse_devices(&devices)` 整份相等（两块盘逐字节不变），录制流 `operation_count()` 不变（一个写、一道屏障都没发）。
- 「照这份文件里已有的拒绝类用例的写法核」：这份文件里没有可写挂载被拒的用例（`grep -n 'expect_err\|Err(' ` 只有第 2837 行几何读者那一条，不经挂载）。照的是同目录 `corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 的 `writable_mount_refused_before_any_write`（该文件第 382 行起：挂载前取整份镜像、`Ok` 时 panic、拒后整份镜像相等）加 `common/mod.rs` 第 295–296 行 `DiskSnapshot` 注释里的「录制流不多一步」那一半。没有直接用 `disk_snapshot`：整份镜像相等已经罩住它比的系统配置槽与根环。
- 用例名没改：「checker 在挂载之前就红 I-3.10」在新写法下仍然成立，名字不说假话；它还被 `crates/mutations.tsv` 第 441 行与 `.claude/kb/invariants.md` 引着，改名要连带改那两处。文档注释第 ②③ 两条改写（第 ③ 条原来说「坏镜像不是恢复会拒掉的样子」，已是假话）。
- 为此在 `use` 里加了 `MountError`、`RecoveryFailure` 两个名字；删掉的第 ③ 步里原用的 `CheckpointTxg(3)/(4)` 断言随之去掉（`CheckpointTxg` 在文件别处还用着，clippy 无告警）。

推翻条件：主工作区上 `rebuild_version` 第 1873 行那处调用被撤掉或挪到任何写之后，这条用例红在第 ③ 步（panic「坏镜像被可写挂载收下了」或盘上字节不等）；若主 agent 另定这份镜像应当可写挂载，则本改动方向错。

## 先红（主工作区现状，副本未改时整个测试二进制，`baseline.log`）

原样：

```
test result: FAILED. 39 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.94s
thread 'an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount' (1015196) panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:4647:63:
坏镜像照样可写挂载: Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
```

基线红集 = 只有点名这一条（它正是这一件要改的那条）；其余 39 条绿。主工作区上那一行今天在第 4647 行（派发提示写的 4572 是较早的行号，I-7.13 那份坏镜像打上之后下移）。

## 改完转绿（副本，整个测试二进制）

命令：`bash research/scripts/capped.sh 4 bash research/scripts/run-with-memory-cap.sh 8G cargo test --offline -p singlefs-harness --test checker_known_bad_images`

```
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 83.80s
exit=0
```

## 变异：改回去它就红

- 现查 `grep -n 'allocation_record_generations_and_spans_are_judged' crates/mutations.tsv`：零命中。表里没有去掉那处调用的行，故新加一条（`patch/mutations-append.tsv`，六段）：
  - 名：`实审 A3a（代码审阅第 33 条）：重建上一版不判分配记录的代 / 跨度（只在记录里的 B 那一版分配代 5 晚于根：坏镜像照样可写挂载，不是在任何写之前拒）`
  - 文件 `crates/singlefs-core/src/recovery.rs`；原文 `    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;`（在 `rebuild_version` 里，今天第 1873 行，文件里恰好一次；第 3405 行 `walk_to_file` 那处原文不同，不受影响）；替换文 `    let _ = allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root);`
  - 参数 `-p singlefs-harness --test checker_known_bad_images -- an_allocation_generation_past_its_unit_birth`；必须红的测试即点名用例。
- 证红：`bash research/scripts/capped.sh 4 bash research/scripts/prove-red.sh --copy <副本> --memory 8G singlefs-harness <上面的名>`，原样：

```
实审 A3a（代码审阅第 33 条）：重建上一版不判分配记录的代 / 跨度（只在记录里的 B 那一版分配代 5 晚于根：坏镜像照样可写挂载，不是在任何写之前拒）	抓到	an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount 红了（日志 /tmp/claude-1000/impl-bad-image-refused-mount/copy/prove-red-logs/001.log）
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
exit=0
```

  prove-red 的基线（同一参数、不改源码）：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in 5.85s`。
- 改坏哪一行 → 哪条断言红：`recovery.rs` 第 1873 行去掉 `?`（结果丢弃）→ 用例第 ③ 步 `Ok(mounted) => panic!` 那一臂红（副本内第 4652 行），原样 `坏镜像被可写挂载收下了：择根 txg CheckpointTxg(3)、现行版 txg CheckpointTxg(4)、施加记录前缀 1 条`。同时红的测试：prove-red 按这一行的参数只跑名字前缀过滤出的 1 条（39 条被过滤），过滤集里只红这一条；整二进制在这条变异下的同红集没跑，留给提交时的 59 号。
- 证过的行 = 新加的这一行；无其余行留给 59 号。第 441 行（walk.rs 改法 E）点名的也是这条用例，它红在第 ② 步，本改动没动第 ② 步，不受影响（没单独复跑，留给 59 号）。

## 第 4 步那几样（全在副本上跑；副本 = 主工作区现状 + 本补丁）

- `cargo fmt --all -- --check`：无输出，`exit=0`。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `check.sh` 第 72–80 行 `CODE_DISCIPLINE_LINTS` 七条 `-D`（现抄）：末尾原样
  ```
      Checking singlefs-harness v0.1.0 (/tmp/claude-1000/impl-bad-image-refused-mount/copy/crates/singlefs-harness)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.95s
  exit=0
  ```
- `cargo build --offline --all-targets`：末尾原样
  ```
     Compiling singlefs-harness v0.1.0 (/tmp/claude-1000/impl-bad-image-refused-mount/copy/crates/singlefs-harness)
      Finished `dev` profile [unoptimized + debuginfo] target(s) in 33.81s
  exit=0
  ```
- 登记给实现员的门禁阶段（`stage-owners.tsv` 现查 7 道），在副本里跑（副本不是 git 仓，依赖 git 的在副本里 77，另在主工作区跑一次，那一次判的是主工作区现状、不含本补丁）：
  - 33 号（副本）`exit=0`：`✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.tsv 1319 条的原文各命中源码一次；……`（新加那一行在内）
  - 53 号（副本）`exit=0`：`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：……）`
  - 92 号：副本 `exit=77`（`! …copy 不是 git 仓，本阶段跳过`）；主工作区 `exit=0`（末行 `第一条纯 SSD 布局线（……）：.claude/kb/layout/02-second-txn.md`）。本补丁不动布局与 checker。
  - 94 号（副本）`exit=0`：`✓ checker 与实现只共享常量模块 singlefs-format（……）；checker 的 4 份源码零处引 singlefs_core……`
  - 93 号（副本）`exit=0`：`✓ feature bit 位号在记账表、D15 已定项 4 与代码三处一致（……扫了 57 个 .rs……）`
  - 89 号：副本与主工作区都 `exit=77`（`⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔……）`），按没判写。
  - 74 号（副本，`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`，经 `capped.sh 4`）`exit=1`，红在 `second_transaction_supplement_three_random_history` 的 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`：
    ```
    已知红第 0 条（增补 2 收口表第 43 行）：0 段；前几个种子 []
    test result: FAILED. 23 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 93.36s
      ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）
    ```
    不在本改动里：本补丁只改 `checker_known_bad_images.rs` 这一个测试源文件，它不进 `second_transaction_supplement_three_random_history` 那个二进制；红的来自副本里主工作区现状（已知红第 43 行那一种今天 0 段，推测与 A3a 让重建先拒有关，没查）。没修，照写，交主 agent 转给管那一条的会话。

## 这一轮写过的文件

- 主工作区：一个都没写（交补丁）。
- 副本里：`crates/singlefs-harness/tests/checker_known_bad_images.rs`（→ `patch/crates.patch`）、`crates/mutations.tsv` 末尾追加 1 行（→ `patch/mutations-append.tsv`，名见上）。
- 草稿：`/tmp/claude-1000/impl-bad-image-refused-mount/` 下 `progress.md`、`report.md`、`patch/`、各 `*.log`、`mutations-row.tsv`。
- `git diff --stat -- crates litmus`（主工作区，原样末行；46 个文件全是别的会话未提交的改动，本件在主工作区零改动）：
  `46 files changed, 19834 insertions(+), 1718 deletions(-)`

## 补丁

- `patch/crates.patch`（`diff -u`，`a/` `b/` 前缀，79 行）；对主工作区 `git apply --check`：`exit=0`。
- `patch/mutations-append.tsv`：1 行六段。
- `patch/report.md`：本报告的副本。
- `apply-writer-patch.py --dry-run` 结果见文末。

## 停下交主 agent 的设计问题

- 无。第 ③ 步钉「被拒、成员、拒前盘上逐字节不变」由派发提示定，没有自定条款。
- 顺带看到（不归本件）：74 号那条随机历史用例在主工作区现状上红（上一节）。

## 没做什么

- 没走三方对抗；没跑层 0、QEMU、herd7、crates 变异整表（59 号）、全量 `cargo test`；没提交。
- 这条变异下整个测试二进制的同红集没跑（prove-red 按行参数过滤到 1 条）；第 441 行变异没复跑。均留给提交时的 59 号。
- 没用 `--release` 再跑：被测路径不经 `debug_assert` 报红，红在测试自己的 panic 臂上。
- 没改用例名（理由见结论）。
- 74 号的红没查根因、没修。

## apply-writer-patch.py --dry-run（主工作区）

```
✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1324 行
exit=0
```

## 清理

- 删了仓副本 `/tmp/claude-1000/impl-bad-image-refused-mount/copy`（含它自己的 target 与 prove-red-logs），删前 `du -sh` 见交回。
