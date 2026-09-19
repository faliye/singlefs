# 层 0 并行第一轮三方判决第二节第 3 行（Y5m2）——实现员报告

时刻 UTC（2026-09-19）。这一回直接改主工作区。依据：`research/prompts/m2-supp3-item1-code-r2-main-verification.md` 第二节第 3 行；变异原文取自 `research/prompts/m2-supp3-item1-code-r2-opus-model/y5-mutants.tsv` 的 Y5m2 那一行。

## 一、结论

- 次序照派发：00:05:41 先重读 `crates/mutations.tsv`（141 行，sha256 3f1007c5…，最后改于 09-18 20:10:29），只在末尾追加 1 行 Y5m2；之后才改 `crash.rs`。
- 新单测 `absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one`（`crates/singlefs-harness/src/crash.rs` 第 1718 行；搭建函数 `slice_with_only_ignored_violations` 在第 1688 行）：前两片都带「不看 journal 那一遍的第一处违例」，并完取第一片的；再并第三片也不换；计数相加。没有改原来那条合并单测，另起一条，名字里写明「两片都有」这个场景。
- Y5m2 施加之后新单测红，其余 26 条 lib 测试不红；照 59 号的判法复跑新加的这一行，红在点名的测试上。
- `check.sh`：主工作区上跑了一次，绿；但跑的途中别的会话在改 `history.rs`（见第四节），所以又在 00:06 的快照副本上跑了一次，也绿。

**什么现象会推翻**：把第 602 行 `if self.first_ignored_violation.is_none() {` 换成 `if first_ignored_violation.is_some() {`，新单测还绿；或者门禁 59 号整表复跑时这一行报「没跑到」。

## 二、写过的文件（主工作区）与 sha256（00:09:38 取）

```
77766ab8ad442dcec756e52561c9e8886536bb171cab8dc86041d8f7793fa3ec  crates/singlefs-harness/src/crash.rs
ed28ab1cba0289e1f392af8c3416239eb7c05e1f448b3868c96a5e90a4ed42b5  crates/mutations.tsv
```

- `crates/mutations.tsv`：141 → 142 行。追加的那一行，变异名 `增补 2 第 41 行 层 0 并行（代码三方第一轮 Y5m2）：并片时「不看 journal 那一遍的第一处」取后面那一片的`；原文与替换文与攻方腿 Y5m2 那一行逐字相同；cargo 参数 `-p singlefs-harness --lib -- absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation`；点名测试 `absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one`。原文在 `crash.rs` 里命中 1 次（Python 按 59 号的判法数）。
- `crates/singlefs-harness/src/crash.rs`：只在 `mod tests` 末尾加了一个搭建函数和一条测试（都用 Edit 写）；非测试代码没动。
- 没碰 `history.rs` 和随机历史的用例文件。

主工作区 `git diff --stat -- crates litmus` 原样（这里面还有上一轮打进去的并行化补丁、别的会话的改动，分不出谁改的；这一回我只改了上面两个文件）：

```
 crates/mutations.tsv                               |  14 +
 crates/singlefs-harness/src/crash.rs               | 874 +++++++++++++++++++--
 crates/singlefs-harness/src/lib.rs                 |   6 +
 .../tests/second_transaction_step_zero_layer0.rs   |  71 +-
 4 files changed, 911 insertions(+), 54 deletions(-)
```

## 三、证明会红（改坏哪一行 → 哪条断言红）

副本 `/tmp/claude-1000/m2-layer0-parallel/r2-copy/`（00:06 从主工作区 `rsync -a --exclude target --exclude .git` 拷来，自己的 target）。

- 基线（不改动）：`cargo test --offline -p singlefs-harness --lib` → `test result: ok. 27 passed; 0 failed`，基线红集为空（`logs/r2-baseline-lib.log`）。
- 施加 Y5m2（`crash.rs` 第 602–604 行 `if self.first_ignored_violation.is_none() { … }` → `if first_ignored_violation.is_some() { … }`），同一个命令（`logs/r2-y5m2-lib.log`）。原样：

```
test crash::tests::absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one ... FAILED
thread 'crash::tests::absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one' (3627456) panicked at crates/singlefs-harness/src/crash.rs:1727:9:
assertion `left == right` failed: 两片都有时取前面那一片的
  left: Some("第二片的 Ignore")
 right: Some("第一片的 Ignore")
test result: FAILED. 26 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

  同时红的只有这一条。原来那条合并单测照旧绿（它前面那一片这一项是空的，就是攻方说的空当）。改完从主工作区拷回原件并 `touch`，`cmp` 相同。被测代码里没有 `debug_assert`。

## 四、`check.sh` 与照 59 号的判法复跑

**主工作区** `nice -n 19 bash .claude/scripts/check.sh`（`logs/r2-check-sh.log`，00:06:58 → 00:08:03，退出码 0），末尾原样：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
```

四道：`✓ 格式通过`、`✓ clippy 通过`、`✓ 构建通过`、`✓ 单测通过`。**但要知道**：这一趟跑的时候 `crates/singlefs-harness/src/history.rs` 在主工作区被改了两次（mtime 00:07:10、00:07:56；别的实现员在改），这一趟编的是哪一版 `history.rs` 说不清。00:07:10 那一版在我第一次照 59 号复跑时编不过（`history.rs:1426` 附近 `judged_by_outcome_only`，9 个错误，E0061、E0277、E0308、E0425、E0559、E0599）。那是改到一半的文件，不是我改的。

所以在 **00:06 的快照副本** `r2-copy/` 上又跑了一次 `check.sh`（`logs/r2-check-sh-snapshot.log`，00:08 起 → 00:09:33，退出码 0）。这个副本的 `crash.rs`、`mutations.tsv` 与主工作区逐字节相同（`cmp`），`history.rs` 是 09-18 20:11 那一版。结果 `✓ 格式通过`、`✓ clippy 通过`、`✓ 构建通过`、`✓ 单测通过`，末行 `  ✓ 单测通过`。

**照 59 号的判法复跑新加的这一行**：临时根 `gate59-root-r2/`，内容是 `r2-copy/` 的 `crates`、`Cargo.toml`、`Cargo.lock`、`litmus`、`research/results`，表里只留注释和这一行；跑的是主工作区的 `bash .claude/gate.d/59-crates-mutation-replay.sh <临时根>`，`GATE_MUTATION_TARGET_DIR=/tmp/claude-1000/m2-layer0-parallel/gate59-target`（`logs/r2-gate59-y5m2.log`，00:07:28 → 00:07:30，退出码 0）。末尾原样：

```
  ✓ 增补 2 第 41 行 层 0 并行（代码三方第一轮 Y5m2）：并片时「不看 journal 那一遍的第一处」取后面那一片的：absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one 红了
  ✓ crates 变异表复跑：1 条变异各自红在点名的测试上（原文都恰好命中一次）
```

第一次拿 00:07:10 的主工作区当临时根，报「点名的测试 … 没跑到（退出码 101）」，原因就是上面那个编不过的 `history.rs`，不是这一行的问题；那份输出被第二次覆盖掉了，这里照记。

## 五、没做什么

- 没走三方对抗；门禁 59 号整表复跑、54 号归 `crash-verifier`；没提交，没做 git 写操作。
- 整张 142 行的变异表没复跑，只复跑了新加的这一行。
- 主工作区的 `check.sh` 是在 `history.rs` 被改的途中跑的，只能算参考；干净的那一次是在快照副本上跑的。`history.rs` 那一版编不过，没修也没碰（别的会话的文件）。
- 登记给我的阶段只有 53 号：在主工作区跑，末行原样 `  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））`，退出码 0。
