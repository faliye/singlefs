# E162 路径搬迁：layer0-shard.env → multi-host.env

## 改了什么（装置源码，只两行）

`research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs`：

```
2898c2898
< /// P_B：`ssh -o BatchMode=yes <PEER_SSH_HOST> nproc`，主机名取仓根 `layer0-shard.env`（与 layer0-shard-run.sh:155 同一个取法）。
---
> /// P_B：`ssh -o BatchMode=yes <PEER_SSH_HOST> nproc`，主机名取仓根 `multi-host.env`（与 layer0-shard-run.sh 的 run_on_peer 函数同一个取法）。
2901c2901
<     let configuration = fs::read_to_string(repository_root.join("layer0-shard.env")).unwrap_or_default();
---
>     let configuration = fs::read_to_string(repository_root.join("multi-host.env")).unwrap_or_default();
```

`layer0-shard-run.sh:155` 现查已经指不到取主机名那段代码（今天那一行是「复原之后回读没过」的错误提示），取主机名/跑 `nproc` 的实际代码在 `run_on_peer` 函数（`research/scripts/layer0-shard-run.sh:219`，`ssh -o BatchMode=yes "$PEER_SSH_HOST" ...` 在 225 行）：注释按定义要求改指函数名。`multi-host.env` 已确认 git 忽略（`.gitignore:19`），键名 `PEER_SSH_HOST=` 与旧文件格式一致（`multi-host.env.example` 第 6–7 行）。别的字一个没动。

`peer_logical_processors` 只在 `throughput_context`（S2/几何敏感性）里被调用（`grep -n peer_logical_processors` 只两处命中：定义处 2899、调用处 3094），不在 anchors 驱动路径上——这是产物逐字节不变的机制解释，不是靠它代替实测。

## 单测

命令（照实验页「S1 够判档档（第二段）」的复跑一节，线程上限按本轮派发改成 8）：

```
cd research && BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include bash scripts/capped.sh 8 bash scripts/run-with-memory-cap.sh 8G cargo test --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store
```

`test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`（原样输出见 `research/prompts/e162-config-rename-evidence/unit-test.log`，退出码 0）。与改名前实验页记的 23 passed / 0 failed 一致。

## 产物：新起文件名、cmp 逐字节比对

`ps` 现查：另一会话在跑 `cargo build --release -p singlefs-checker-tier --features verdict-store`（编 librocksdb-sys，非性能测量类），照共用约束加 `nice -n 19` 照常跑。

编译（`cargo build -q --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store` 报 `Finished ... in 0.03s`，说明用的就是上面单测那次已经编好的、含这次改动的二进制，`research/target/release/`，未设 `CARGO_TARGET_DIR`）之后跑：

```
bash research/scripts/capped.sh 8 bash research/scripts/run-with-memory-cap.sh 8G ./target/release/e162-crash-verdict-block-store anchors > research/results/e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out
```

排他新建（`set -o noclobber`），退出码 0，27 行。

```
$ cmp research/results/e162-crash-verdict-block-store-2026-09-27-anchors.out research/results/e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out
（无输出，退出码 0，逐字节一致）
```

旧产物 `e162-crash-verdict-block-store-2026-09-27-anchors.out` 一个字节没改、没删。

判决行点名（第 4c 步）：新产物只有一条 `name=verdict` 行——`E7RESULT name=verdict part=anchors anchor_mismatches=0`（第 26 行）。`anchor_mismatches` 是名字表示不匹配的整数计数，取值 0，不点名；收尾 `E7RESULT name=done emitted=27`（27 = 行数，等于 `wc -l`）。判决行 1 条，没有 `false` / `not_run` 字段，违例、不匹配、歧义、失败类的整数计数都是 0。

## replay.sh

`research/scripts/replay.sh` 第 211 行 E162 anchors 登记行改指新文件名：

```
E162|@driver_e162_anchors||e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out|exact
```

`bash research/scripts/replay.sh E162`（改登记前、改登记后各跑一次）两次都报：

```
E162  @driver_e162_anchors     字节一致 e162-crash-verdict-block-store-2026-09-27-anchors[-2026-09-28].out
E162  @driver_e162_power_cut_selftest 字节一致 e162-verdict-store-power-cut-2026-09-27-selftest-r2.out
字节一致 2 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0 ／ 输入没变没跑 0
✓ 复跑判过的 2 行都对得上（字节一致 2、仅计时不同 0），结论断言全中
```

第二行 `driver_e162_power_cut_selftest` 登记未动（那份装置 `e162_verdict_store_power_cut.rs` 没有引用 `layer0-shard.env`/`multi-host.env`，`grep` 零命中），本轮没有改它的产物，只是顺带在同一次 `replay.sh E162` 里一起复跑、一起确认字节一致，不属于本轮搬迁范围。

## 变异表：碰不到，没动

`research/mutations/e162_crash_verdict_block_store.tsv`（13 条，全文已读）逐条「原文」子串核对，没有一条命中改动的这两行（都是 `WritePath`/`redb`/`RocksDB`/核对/读路径/杀点/计时/块值/键编码相关的业务逻辑行，与配置文件名注释、`fs::read_to_string` 那一行无关）。按定义第 5 步「碰不到就不动」，没有改这张表，也没有重跑变异。

## kb 改动

- 实验页 `.claude/kb/experiments/162-崩溃放量判定块存储选型.md`：
  - 「anchors 判决」「复跑」两节里三处产物文件名改成新名字，并各加一句指向今天的路径搬迁与 `cmp` 结果；
  - 「影响的决策」D13（验证路线） 已定项 4 那一行回看改写「2026-09-28 不受影响：只改配置文件名，结论、数与产物不变」，`grep -n 'E162（崩溃放量判定块存储选型）' .claude/kb/decisions/13-验证路线.md` 今天现查仍是零命中（依据段没引这个实验），这一行仍按备料记，**要不要升成支撑交主 agent 判**（本轮没有改决策文件，写范围也不许）；旧的两层回看原样折进这一行的括注里，没有丢内容；
  - 「历史版本」新增一条「2026-09-28：路径搬迁」。
- `.claude/kb/experiments-history.md`：新增一条「2026-09-28：E162（崩溃放量判定块存储选型） 路径搬迁——配置文件改名」，放在历史版本最上面（该文件按新到旧排）。

## 写范围与副作用

只写了：装置源码那两行、`research/scripts/replay.sh` 那一行登记、新产物文件、实验页、`experiments-history.md`、本报告。没碰 `crates/`、没碰别的实验、没跑门禁、没提交。

`git status` 现查：仓里有另一会话正在进行的大规模 SOP/gate.d 重排（提交前 `git status` 快照与本轮开始时一致），我只核对了上面列出的这几个文件相对 HEAD 的 diff，确认里面只有本轮这些改动，没有碰任何不属于本轮的文件；这几个文件 `git status` 显示 `MM`（同时有暂存与未暂存差异），是另一会话在这几个文件已有的改动之上叠加了我这一轮的编辑，我没有做任何 git 操作（add/commit 都没跑），暂存区那一半不归我处理。

草稿目录 `/tmp/claude-1000/e162-config-rename/`（16K，只有本轮的报告与两份日志，没有仓副本、没有编译目录，不需要交回前清理）。

## 没做什么

- 没跑门禁（提交前由 gate-triage 跑）。
- 没跑 `driver_e162_power_cut_selftest` 之外/之内的任何 S1/S2/S3/S4 重跑（不在本轮范围，任务只要求锚点产物准入判 + 复跑）。
- 没判「D13（验证路线） 已定项 4 这一行该不该从备料升成支撑」——依据段没引这个实验，交主 agent。
- 没有暂存或提交任何改动。
