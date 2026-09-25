# 门禁 69 号补落产物：交回报告（2026-09-25）

范围：问题单 `research/prompts/m2-header311-rerun-questions.md` 第 3、4、5、6 行。不改装置、不改结论——本轮只补落产物、补跑变异整表、复核门禁 69 号。

## 1. E155 / E157 / E159：用今天的装置重跑，产物落库

跑前登记（沿用，未改一字）：`research/prompts/e155-r5-prereg.md`、`e157-r1-prereg.md`、`e159-r1-prereg.md`。

命令：
```
$ REPLAY_OUT=/tmp/claude-1000/gate69-refresh/replay-out nice -n 19 bash research/scripts/capped.sh 6 bash research/scripts/replay.sh E155 E155R2 E155R3 E155R4 E157 E159
实验 二进制                判定     说明
-------------------------------------------------------------------------
E155  e155-fsync-write-volume  字节一致 e155-fsync-write-volume-2026-09-24-rename.out
E155R2 e155-second-run-fsync-write-volume 字节一致 e155-second-run-fsync-write-volume-2026-09-24-rename.out
E155R3 e155-third-run-release-cascade 字节一致 e155-third-run-release-cascade-2026-09-20-stage1.out
E155R4 e155-fourth-run-group-commit-concurrency 字节一致 e155-fourth-run-group-commit-concurrency-2026-09-21.out
E157  e157-parallel-line-one-clauses 字节一致 e157-parallel-line-one-clauses-2026-09-22.out
E159  e159-fsync-wait-group-commit 字节一致 e159-fsync-wait-group-commit-2026-09-24-anchors.out
-------------------------------------------------------------------------
字节一致 6 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0
```
全文见 `research/results/gate69-refresh-replay-e155-e157-e159-2026-09-25.log`。

六份都与 `replay.sh` 当时登记的产物逐字节一致（`diff -q` 无输出）。这六份登记产物的 mtime（Sep 23–24）早于对应装置源码改到 311 的 mtime（Sep 24 13:35–14:09），门禁 69 号判据一因此判红——重跑得出的字节与登记产物相同，但没有落进 `research/results/` 的新产物，`replay.sh` 也没有指到一份不比源码旧的文件。

处置（登记里「相同：把 replay.sh 的登记行指到新文件，旧文件留着」）：把 `$OUT_DIR` 里六份现跑产物逐一 `cmp` 确认与旧登记产物逐字节相同后，另存新文件（旧文件原样留着，未删）：

| 实验 | 新产物（`research/results/`） | 与旧产物 `cmp` |
|---|---|---|
| E155 | `e155-fsync-write-volume-2026-09-25-h311-replay.out` | 一致 |
| E155R2 | `e155-second-run-fsync-write-volume-2026-09-25-h311-replay.out` | 一致 |
| E155R3 | `e155-third-run-release-cascade-2026-09-25-h311-replay.out` | 一致 |
| E155R4 | `e155-fourth-run-group-commit-concurrency-2026-09-25-h311-replay.out` | 一致 |
| E157 | `e157-parallel-line-one-clauses-2026-09-25-h311-replay.out` | 一致 |
| E159 | `e159-fsync-wait-group-commit-2026-09-25-h311-replay.out` | 一致 |

`research/scripts/replay.sh` 第 171–174、176、192 行改指这六份新文件（旧文件名一个字没删）。改完复跑一次确认登记自洽：

```
$ nice -n 19 bash research/scripts/capped.sh 6 bash research/scripts/replay.sh E155 E155R2 E155R3 E155R4 E157 E159
字节一致 6 ／ 仅计时不同 0 ／ 对不上 0 ／ 跑不了 0 ／ 结论断言不中 0 ／ 产物已归档 0
```
全文见 `research/results/gate69-refresh-replay-e155-e157-e159-postupdate-2026-09-25.log`。

单测数（一条命令数出来，本轮未改任何测试，数字与各自跑前登记记的一致，用于确认没有退化）：

```
e155-fsync-write-volume: 43 passed
e155-second-run-fsync-write-volume: 53 passed
e155-third-run-release-cascade: 23 passed
e155-fourth-run-group-commit-concurrency: 76 passed
e157-parallel-line-one-clauses: 21 passed
e159-fsync-wait-group-commit: 55 passed
```
（命令：`cd research && nice -n 19 bash scripts/capped.sh 6 cargo test --release --manifest-path e7-index-bench/Cargo.toml --bin <bin>`，逐个跑出，`test result: ok` 行原样如上。）

本轮没有变异改动（变异表本轮未碰这六份对应的表），沿用各自跑前登记里记的数（E155 四份改后 18/37/17/9 抓到、各 1 等价、0 无效、0 没红；E157 改后 10/10/10 全抓；E159 改后 17/17/17 全抓，均见对应跑前登记「九、变异」与实验页历史节，本轮未重跑）。

实验页历史节各补一条（`.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`、`157-并行线一两条条款的计数模型.md`、`159-fsync等待时间随并发数组提交与wal两臂.md`），形态照页里已有条目：「2026-09-25 为门禁 69 号补落产物，逐字节相同」，写明补存的新文件名与改指的登记行。没有写任何新结论、没有改判据。

## 2. E142 变异表整表复跑

`research/mutations/e142_first_transaction_dry_run.tsv` 最后一次改动在 2026-09-24 17:32:40 UTC（`git diff` 确认：M23 锚点从 307 改到 311、新增 M80–M93 共 13 条），晚于此前两份变异日志（`research/results/e142_first_transaction_dry_run-mutate-2026-09-25-header311-last-flag.log`、`…-header311-last-flag-reverify5.log`，均 17:29:22 UTC），后者只覆盖 87 条 + 单独复核 5 条，两次都不是今天这份 92 行 tsv 的整表一次性重跑。

装置源码 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` 本轮未改（mtime 17:17:45，早于两份日志与 tsv 最后一次改动，确认基线未变）。

跑前登记 `research/prompts/e142-r14-prereg.md`（沿用，未改一字，本轮不涉及它写死的判据）；实验页登记的变异命令（`Cargo.toml:377-378` 的 bin 名 `e142-first-txn-dry-run`）：

```
$ cd research && nice -n 19 bash scripts/mutate.sh e142-first-txn-dry-run e7-index-bench/src/bin/e142_first_transaction_dry_run.rs mutations/e142_first_transaction_dry_run.tsv
基线：全绿
… （92 条 ✅ [Mxx…] 红：… ，逐条见日志）
已还原，基线仍全绿
```
（这条命令通过 `run_in_background` 起时误加了一个多余的结尾 `&`，导致工具层的完成通知在真正跑完之前就发了；发现后没有重起这条 mutate.sh——它当时仍在正常跑——另起一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait 2102147 --timeout 3600` 等它真正退出，退出码 0 后才读日志。）

三个数（一条命令数出来）：

```
$ grep -c '^✅ \[' /tmp/claude-1000/gate69-refresh/e142-mutate-full-2026-09-25.log
92
$ grep -c '⏭' /tmp/claude-1000/gate69-refresh/e142-mutate-full-2026-09-25.log   # 无效
0
$ grep -c '❌' /tmp/claude-1000/gate69-refresh/e142-mutate-full-2026-09-25.log   # 没红
0
$ tail -1 /tmp/claude-1000/gate69-refresh/e142-mutate-full-2026-09-25.log
已还原，基线仍全绿
```

**抓到 92 条、无效 0 条、没红 0 条**——tsv 里全部 92 行（不含登记原文点名跳过、不进表的 M89）都在今天这份源码上一次性重跑抓到，没有需要逐条列出的无效或没红条目，不需要改表。

单测（本轮未改代码，跑出数确认与实验页记的一致）：
```
$ cd research && nice -n 19 bash scripts/capped.sh 6 cargo test --release --manifest-path e7-index-bench/Cargo.toml --bin e142-first-txn-dry-run
test result: ok. 57 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.40s
```

产物落库：`research/results/e142_first_transaction_dry_run-mutate-2026-09-25-full.log`（sha256 `dd8492c96094cee32a857a00fc13e3a767535480f5289841dcbda19349aa7423`，与草稿目录里的原始输出逐字节相同）。实验页 `.claude/kb/experiments/142-第一个事务的干跑.md` 历史节补一条「2026-09-25（为门禁 69 号补落产物）」：「变异 92 条全抓」，形态照页里已有条目，只记这一件事，没有改判决、没有改字节清单结论。

## 3. 门禁 69 号复核

```
$ nice -n 19 bash .claude/gate.d/69-evidence-in-repo.sh
  ✓ 产物与依据都落在仓里（判了 23 份这一轮改过的实验装置与变异表、290 份 .md，扫到 210 处 /tmp 路径、没有一处是引依据；基准 e980a219f1834c638cd2fae18b25a60525a6bd52）
    没判 432 份 research/prompts 下这一轮没新写的 .md（登记在 .claude/doc-lint-exclude 的原样保存证据，改不得）
退出码：0
```

## 登记修订了什么

四份跑前登记（`e155-r5-prereg.md`、`e157-r1-prereg.md`、`e159-r1-prereg.md`、`e142-r14-prereg.md`）**没有**任何修订——本轮没有发现需要收严判据或补臂的情况，只是把已经判出的结论对应的产物补落进仓里、把已经改过的变异表补跑一遍整表。

## 实验页路径

- `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md`
- `.claude/kb/experiments/157-并行线一两条条款的计数模型.md`
- `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md`
- `.claude/kb/experiments/142-第一个事务的干跑.md`

## 写了/改了哪些文件

- `research/scripts/replay.sh`（第 171–174、176、192 行改指新产物文件名）
- `research/results/e155-fsync-write-volume-2026-09-25-h311-replay.out`（新增）
- `research/results/e155-second-run-fsync-write-volume-2026-09-25-h311-replay.out`（新增）
- `research/results/e155-third-run-release-cascade-2026-09-25-h311-replay.out`（新增）
- `research/results/e155-fourth-run-group-commit-concurrency-2026-09-25-h311-replay.out`（新增）
- `research/results/e157-parallel-line-one-clauses-2026-09-25-h311-replay.out`（新增）
- `research/results/e159-fsync-wait-group-commit-2026-09-25-h311-replay.out`（新增）
- `research/results/e142_first_transaction_dry_run-mutate-2026-09-25-full.log`（新增）
- `.claude/kb/experiments/155-…md`、`157-…md`、`159-…md`、`142-…md`（历史节各补一条）

未改：`research/mutations/e142_first_transaction_dry_run.tsv`（本轮只读，未追加、未改任何一行）；四份跑前登记；任何装置源码。

## 岔路表

| 问题单行 | 已够判 / 还差什么 | 登记里剩下的量能不能让它翻面 | 指到的命令 |
|---|---|---|---|
| 第 3 行（E155 四份装置） | 已够判——上一轮「不变」的结论本轮未动；本轮只补落产物、更新 `replay.sh` 登记，门禁 69 号从红转绿 | 不能：`e155-r5-prereg.md` 第六节的判据（Q155.1=67、Q155.2 逐字节相同）本轮用今天的装置重新核过，四份都仍相同；登记里没有剩下未跑的量（第六节「够判点」已在上一轮触发） | `bash research/scripts/replay.sh E155 E155R2 E155R3 E155R4`（本报告第 1 节，`replay-e155-e157-e159-postupdate.log`） |
| 第 4 行（E157 第一段） | 已够判——上一轮「不变」的结论本轮未动；本轮只补落产物、更新 `replay.sh` 登记 | 不能：第一段判据本轮用今天的装置重新核过，逐字节相同；第二段（C491）按 D23 已定项 17 定案，登记里明写「不因这一次而续跑」，不算这一次的欠账 | `bash research/scripts/replay.sh E157`（本报告第 1 节） |
| 第 5 行（E159 锚点与冒烟） | 部分够判——写量锚点与冒烟这一半本轮用今天的装置重新核过、逐字节相同；等待时间的计时段仍按登记明写「不在这一次」，留给机器空闲时另派 | 写量与冒烟这一半不能翻面（逐字节相同）；计时段完全没跑，不构成本轮的欠账（登记与问题单第 5 行都写明这一段延后） | `bash research/scripts/replay.sh E159`（本报告第 1 节）；计时段命令未跑，见 `e159-r1-prereg.md`「五、5.7」 |
| 第 6 行（E142 变异表） | 已够判——字节清单与判决的结论在第十四次跑已判出「不变」，本轮只补跑变异整表；92 条全抓、0 无效、0 没红 | 不能：变异整表复跑不影响 Q142.1–Q142.6 与 P1–P6 的判定（那些用的是产物比对，不是变异表）；登记第二段（Q142.5、整张变异表两次比对附带）仍标「够判后未跑」，与本轮无关，非欠账 | `cd research && bash scripts/mutate.sh e142-first-txn-dry-run e7-index-bench/src/bin/e142_first_transaction_dry_run.rs mutations/e142_first_transaction_dry_run.tsv`（本报告第 2 节，`e142-mutate-full-2026-09-25.log`） |

## 没做什么

- 没重判这四行问题单的「不变 / 变」结论——沿用上一轮已经判出的结论，本轮只补齐门禁 69 号要求的证据。
- 没跑 E159 的计时段（问题单第 5 行明写「机器空闲时另跑」，不在本轮范围）。
- 没跑 E142 登记第二段（Q142.5 附带、整张变异表两次比对——这条与本轮任务不同，本轮任务是「把变异表在今天源码上整表跑一遍」，已完成；登记里写的「两次比对」指的是改常量前后两份日志的比对，那件事在第十四次跑已经做过）。
- 没跑门禁全量、没跑 15/87 号（不归本轮）、没提交。
- 没改任何装置源码、没改任何跑前登记的判据段。
