# I-7.13 坏镜像补齐（checker_known_bad_images）实现员报告

写于 2026-09-27（时刻 UTC；JST = UTC + 9）。底座是主工作区 2026-09-27T08:08:28Z 的现状（`snapshot-sha256.txt`：`checker_known_bad_images.rs` e3734906…、checker `image.rs` 3e780e8b…、`walk.rs` b32f0b02…、`crates/mutations.tsv` fae2eee2…）。改动在副本 `work/` 里做，交的是 `patch/`。线程上限 4，内存上限 8G。

## 一、结论

- `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 由红转绿。做法：在 `known_bad_images` 里给 I-7.13 补了一份坏镜像：盘 1 槽 0 的加密类型改成 1，再重算整槽校验和（写法跟 A3-checker-2 那条用例一样）。
- 新加用例 `a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`，断言三件事：这份坏镜像只红 I-7.13；违例说明点名「盘 1 偏移 0」；其余每一条不变量都报不适用。
- 两行新变异经 `prove-red.sh` 都抓到了（第四节）。
- 整个测试二进制改完之后是 39 绿 1 红。红的那一条在改动之前的基线里就红（第三节），不是这一件带来的。
- **什么现象会推翻**：补丁打进主工作区之后，这两条用例里有一条红；或者门禁 59 号复跑这两行变异时没红；或者主 agent 改了处置（比如一槽越界时拿别的槽照判），这时第二条用例会红，得跟着改。

## 二、这一轮写过的文件

- `crates/singlefs-harness/tests/checker_known_bad_images.rs`（副本里改的；`patch/crates.patch`）：
  - 第 12–14 行：`use` 加进 `SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`。
  - 第 413–418 行：四个常量。`SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET = 219`，由 kb 字段表算出：155 + 32 + 16 + 12 + 4，和 checker `lib.rs:211` 的写法一样；`ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ = 1`；`DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT = 1`；`OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT = 0`。
  - 第 812–828 行：`known_bad_images` 里紧跟 I-7.6 那一份，加了一份 I-7.13 坏镜像，键直接用 checker 的常量。
  - 第 2322 行起：新用例（上面第一节第二条）。
- `crates/mutations.tsv`：只在末尾追加 2 行（`patch/mutations-append.tsv`），见第四节。
- 草稿目录下另有 `report.md`、`progress.md`、各条日志与 `patch/`。

`git apply --stat`（在主工作区上跑；副本里没有 `.git`，给不出 `git diff --stat -- crates litmus`，下面是等价的统计）：

```
 .../tests/checker_known_bad_images.rs              |   77 ++++++++++++++++++++
 1 file changed, 76 insertions(+), 1 deletion(-)
```

## 三、先红与转绿（整个测试二进制，都经 `capped.sh 4` 与 `run-with-memory-cap.sh 8G`）

**先红**：在主工作区原样副本 `base/` 上跑（08:08:28Z 拷的，四份文件的 sha256 与底座逐一核过都是 `OK`）。`cargo test --offline -p singlefs-harness --test checker_known_bad_images`，日志 `baseline-kept.log`。原样摘录：

```
test an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount ... FAILED
test the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target ... FAILED
thread 'the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target' (924570) panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:2344:5:
assertion `left == right` failed: 第一版每条不变量都至少有一份坏镜像，也没有清单之外的
  left: {"I-1.1", "I-1.10", "I-1.11", "I-1.2", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-1.8", "I-2.1", "I-2.3", "I-2.4", "I-2.5", "I-3.1", "I-3.10", "I-3.11", "I-3.8", "I-3.9", "I-4.2", "I-4.8", "I-5.1", "I-5.2", "I-5.4", "I-7.1", "I-7.12", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-7.9", "I-8.6", "I-8.7", "I-8.8", "I-8.9", "I-9.1", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15", "I-9.2", "I-9.4", "I-9.6", "I-9.7"}
 right: {"I-1.1", "I-1.10", "I-1.11", "I-1.2", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-1.8", "I-2.1", "I-2.3", "I-2.4", "I-2.5", "I-3.1", "I-3.10", "I-3.11", "I-3.8", "I-3.9", "I-4.2", "I-4.8", "I-5.1", "I-5.2", "I-5.4", "I-7.1", "I-7.12", "I-7.13", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-7.9", "I-8.6", "I-8.7", "I-8.8", "I-8.9", "I-9.1", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15", "I-9.2", "I-9.4", "I-9.6", "I-9.7"}
test result: FAILED. 37 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 76.95s
```

**基线红集**有两条：上面那条，加上 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount`。后一条的红在原样副本上就有，不在这一件里，原样摘录：

```
thread 'an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount' (923418) panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:4572:63:
坏镜像照样可写挂载: Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "分配记录跨度为 0，或分配代 / 释放代晚于根" })
```

推测它来自别的会话在 `singlefs-core` 里还没提交的改动，没查。按第 5 条不修，交主 agent。

**转绿**：改动之后的副本 `work/`，同一条命令，日志 `after.log`。原样：

```
test a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable ... ok
test the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target ... ok
test an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount ... FAILED
thread 'an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount' (930279) panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:4647:63:
test result: FAILED. 39 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 297.71s
error: test failed, to rerun pass `-p singlefs-harness --test checker_known_bad_images`
exit 101
```

剩下的那一条就是基线红集里那条：行号 4572 挪到 4647，是因为上面插了 75 行。

## 四、证红（`research/scripts/prove-red.sh --copy work/ --memory 8G singlefs-harness <两个名字>`，经 `capped.sh 4`）

两行都亲自证过，没有留给门禁 59 号去证的。原样输出（`prove-red.log`）：

```
I-7.13 坏镜像（checker_known_bad_images）：盘 1 槽 0 的加密类型改回 0，坏镜像退成干净镜像	抓到	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target 红了（日志 /tmp/claude-1000/impl-i713-bad-image/prove-red-logs/001.log）
I-7.13 坏镜像（checker_known_bad_images）：一槽越界时拿别的槽照判、照作保	抓到	a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable 红了（日志 /tmp/claude-1000/impl-i713-bad-image/prove-red-logs/002.log）
✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了
exit 0
```

| 变异 | 改坏哪一行 | 哪条断言红（日志原样） |
|---|---|---|
| 加密类型改回 0 | 测试文件第 415 行：`ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ` 由 `1` 改成 `0`。这一槽重封之后和原样逐字节相同，坏镜像退成了干净镜像 | `checker_known_bad_images.rs:2427:9`：`I-7.13 那份坏镜像应当判违例，实际 Holds` |
| 一槽越界时拿别的槽照判 | checker `walk.rs` 里 `    if any_system_configuration_slot_carries_a_refused_value {` 前面加 `false &&`（原文和 A3-checker-2 那一行相同，锚点在 `walk.rs` 里恰好命中 1 次） | `checker_known_bad_images.rs:2361:17`：`这个池挂不上，I-1.1 要报不适用，得到 Holds` |

- 每行的参数里都带 `-- <测试名>`，所以证红只跑了点名的那一条（`39 filtered out`），同一个二进制里还有哪些测试会跟着红，这次没跑出来。推测：第一行变异也会让新用例红，红在第 2361 行的「要报不适用」那一条（镜像退成干净的之后，判定清单里排在前面的 I-1.1 报成立，比 I-7.13 那一臂先走到）。这一点没跑过。
- prove-red 自带的基线：`prove-red-logs/baseline.log` 里是 `test result: ok. 1 passed; 0 failed; ... 39 filtered out`。
- 跑完之后核过还原：`walk.rs` 与主工作区那份 `cmp` 相同；第 415 行的原文照旧命中 1 次。
- 被测代码里没有 `debug_assert` 挡在断言前面：两条红都红在测试自己的断言上，所以没有再跑 `--release`。

## 五、第 4 步那几样的末尾原样输出（都在副本 `work/` 里跑）

### fmt（`cargo fmt --all -- --check`，先对副本跑过一次 `cargo fmt --all`，只动了本文件）

```
fmt exit 0
```

### clippy（`--all-targets --all-features -- -D warnings`，加上 `check.sh:72-80` 的 7 条 `-D`）

```
    Checking singlefs-harness v0.1.0 (/tmp/claude-1000/impl-i713-bad-image/work/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.22s
clippy exit 0
```

### build（`cargo build --offline --all-targets`）

```
   Compiling singlefs-harness v0.1.0 (/tmp/claude-1000/impl-i713-bad-image/work/crates/singlefs-harness)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 35.49s
build exit 0
```

### 动到的测试二进制

见第三节「转绿」。

## 六、登记给实现员的门禁阶段（`stage-owners.tsv`，都在副本 `work/` 里跑，`nice -n 19`）

登记给实现员的有 7 个：33、53、74、92、94、93、89。末行原样（太长的截到 200 字，全文在草稿目录 `gate-<阶段>.log`）：

| 阶段 | 退出码 | 末行 |
|---|---|---|
| 33 | 0 | `✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.tsv 1318 条的原文各命中源码一次；……`（1318 条里有新追加的 2 行） |
| 53 | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：……）` |
| 92 | 77（没跑） | `! /tmp/claude-1000/impl-i713-bad-image/work 不是 git 仓，本阶段跳过` |
| 94 | 0 | `✓ checker 与实现只共享常量模块 singlefs-format（……checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）……` |
| 93 | 0 | `✓ feature bit 位号在记账表、D15 已定项 4 与代码三处一致（……扫了 57 个 .rs，认出 4 处 feature bit 常量……）` |
| 89 | 77（没跑） | `「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐` |
| 74 | 1（红） | `✗ 随机历史的测试二进制判红`：`second_transaction_supplement_three_random_history` 里 `crash_recovery_abandoning_a_newest_root_the_system_configuration_never_witnessed_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 红了（`test result: FAILED. 23 passed; 1 failed; 2 ignored`） |

- 74 号的红和这一件没有关系。它只编、只跑 `second_transaction_supplement_three_random_history` 这一个测试二进制；这一件只改了 `checker_known_bad_images.rs`，那是另一个测试二进制，不进它的编译。副本里其余 `crates/` 与主工作区相同（`diff -rq` 只报这一份文件），所以按推断主工作区上 74 号也是同一个红。没在主工作区上复跑。照第 5 条不修，交主 agent。
- 92 号在副本上跑不了（要 git）。它判的是布局与 checker 的同步，这一件没动 checker 也没动 kb。没在主工作区上跑：主工作区还没有这份改动，跑出来不说明这一件。

## 七、停下交主 agent 的

1. **条款**：I-7.13 的原句还没进 `.claude/kb/invariants.md`，照派发提示由主 agent 另派。这一件引编号只经 checker 的常量，没写原句。
2. **基线红**：`checker_known_bad_images` 里的 `an_allocation_generation_past_its_unit_birth_…_before_the_mount`（第三节）与 74 号那一条（第六节）都红在主工作区现状上，不是这一件带来的，要派给改那两处的会话。
3. 没有要自己拍板的设计判断。「只红 I-7.13、其余报不适用」照的是主 agent 已经认下的处置。

条款没写、也不是分支的项：没有。

## 八、受影响的层 0 流与崩溃枚举用例

不适用：没动 `crates/singlefs-checker/src/`，只改了一个测试文件，新加的变异行里有一行会临时改坏 `walk.rs`，但只在证红时动。没有新加层 0 流，也没有崩溃枚举用例。

## 九、补丁

`patch/` 里有三份：
- `crates.patch`：1 个文件。对主工作区 `git apply --check` 退 0，打补丁之前主工作区那份文件的 sha256 仍是底座的 e3734906…。
- `mutations-append.tsv`：2 行，六段，名字在表里没有。
- `report.md`：本文的拷贝。

没有 replacements，也没有 delete。

## 十、没做什么

- 没走三方对抗，没提交。层 0 全量、QEMU、herd7 和变异整表归 crash-verifier；门禁 59 号也没跑。
- 没跑全量 `cargo test`，也没跑别的测试二进制。
- 没在主工作区上跑任何东西：先红与转绿都在副本上跑，副本和主工作区的差别见第六节。
- 没改 `invariants.md`，也没改文件头「第一版判的 36 条不变量」那句过时的注释：条款归主 agent 另派，注释不在这一件的范围里。
- 第三节与第六节那两条基线红没查原因。

## 十一、草稿与删掉的东西

- 删了 `/tmp/claude-1000/impl-i713-bad-image/base`（原样副本，删前 `du -sh` 2.2G），基线日志拷到了 `baseline-kept.log`。
- 删了 `/tmp/claude-1000/impl-i713-bad-image/work`（改动副本，删前 `du -sh` 18G），改动都在 `patch/` 里。
- 留着：`report.md`、`progress.md`、`patch/`、`snapshot-sha256.txt`，以及日志 `baseline*.log`、`after.log`、`prove-red.log`、`prove-red-logs/`、`clippy.log`、`build.log`、`gate-*.log`、`draft.diff`，都是给主 agent 核的材料。

`apply-writer-patch.py --dry-run` 原样输出（在主工作区上，08:2x UTC）：

```
✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1323 行
dry-run exit 0
```
