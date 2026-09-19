## 二、这一轮写过的文件

主工作区一个字没写。全在 `/tmp/claude-1000/m2-layer0-parallel/` 下：

- 副本里改的（补丁的来源）：`copy/crates/singlefs-harness/src/crash.rs`、`copy/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`、`copy/crates/mutations.tsv`（末尾追加 8 行，变异名见第六节）、`copy/.claude/gate.d/54-layer0-replay.sh`。
- 补丁：`crates-layer0-parallel.patch`（`crates/` 三个文件）、`gate54-layer0-parallel.patch`（54 号），都是 `-p1`、`a/` `b/` 前缀，由 `draft/make-patches.sh` 拿 `original/`（开工 15:51 从主工作区拷的快照）对 `copy/` 生成。
- 草稿：`draft/`（`run-layer0.sh`、`run-mutations.sh`、`mutations-local.tsv`、`summarize-mutation.py`、`gate54-functions-check.sh`、`make-patches.sh`、报告分段）；变异副本 `mutant/`；59 号复跑用的临时根 `gate59-root/` 与 target `gate59-target/`；日志 `logs/`；`progress.md`；本报告 `report.md`。

补丁对主工作区的改动量（`git apply --stat`，在主工作区跑）：

```
 crates/mutations.tsv                               |    8 
 crates/singlefs-harness/src/crash.rs               |  795 +++++++++++++++++++-
 .../tests/second_transaction_step_zero_layer0.rs   |   71 ++
 3 files changed, 829 insertions(+), 45 deletions(-)
 .claude/gate.d/54-layer0-replay.sh |   63 +++++++++++++++++++++++++++++++++---
 1 file changed, 58 insertions(+), 5 deletions(-)
```

主工作区 `git diff --stat -- crates litmus` 原样（都是别的会话的未提交改动，我没碰）：

```
 crates/mutations.tsv                 |  2 ++
 crates/singlefs-harness/src/crash.rs | 25 ++++++++++++++++---------
 crates/singlefs-harness/src/lib.rs   |  6 ++++++
 3 files changed, 24 insertions(+), 9 deletions(-)
```

补丁是对主工作区今天的内容做的：快照里的 `crash.rs` 已经带着上面那 25 行未提交改动（`SparseDevice::read_into`），`mutations.tsv` 已经带着那 2 行；`history.rs` 与随机历史用例没进补丁。19:38 主工作区这四个文件与快照逐字节相同（`cmp`），两份补丁 `patch -p1 --dry-run` 都干净（`checking file …` 各行、退出码 0）。
