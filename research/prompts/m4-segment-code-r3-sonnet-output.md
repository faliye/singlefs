# m4-segment-code-r3 辩方腿报告（Sonnet，2026-10-16）

立场：辩方，正文 `research/prompts/_m4-segment-code-r3-body.md` 第四节辩方那一行。复核前两轮判决 `research/prompts/m4-segment-code-r1-main-verification.md`、`research/prompts/m4-segment-code-r2-main-verification.md` 的结论与判定过程，回答 Q1、Q3、Q4。不替主 agent 撤销或维持判决。
冻结副本：`~/.local/share/singlefs/work/m4-segment-code-r3/repo/`（第三轮），前两轮的 `~/.local/share/singlefs/work/m4-segment-code-r1/repo/`、`~/.local/share/singlefs/work/m4-segment-code-r2/repo/`。下文代码引文一律写绝对路径；kb 条款写 kb 文件自己的行号；产物行写 `research/prompts/…` 下的文件。
草稿目录：`~/.local/share/singlefs/work/m4-segment-code-r3-defense/`（交回前删编译目录与仓拷贝，见末节）。模型目录：`research/prompts/m4-segment-code-r3-sonnet-model/`。

## 0　开工核对

命令（在第三轮冻结副本根上）：`sha256sum -c ~/.local/share/singlefs/work/m4-predev/research/prompts/m4-segment-code-r3-snapshot/crates-sha256.txt`，原样输出：

```
crates/mutations.tsv: OK
crates/singlefs-core/src/admission.rs: OK
crates/singlefs-core/src/allocator.rs: OK
crates/singlefs-core/src/block_device.rs: OK
crates/singlefs-core/src/code_two_tree.rs: OK
crates/singlefs-core/src/compaction_plan.rs: OK
crates/singlefs-core/src/lib.rs: OK
crates/singlefs-core/src/logged_operations.rs: OK
crates/singlefs-core/src/mounted_read.rs: OK
crates/singlefs-core/src/mounted_session.rs: OK
crates/singlefs-core/src/mount.rs: OK
crates/singlefs-core/src/records.rs: OK
crates/singlefs-core/src/recovery.rs: OK
crates/singlefs-core/src/region_scan.rs: OK
crates/singlefs-core/src/transaction.rs: OK
crates/singlefs-core/src/unlink.rs: OK
crates/singlefs-core/src/write_accounting.rs: OK
crates/singlefs-format/src/lib.rs: OK
crates/singlefs-reference-filesystem/src/lib.rs: OK
crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs: OK
crates/singlefs-replay-prefix/src/lib.rs: OK
```

（21 行全 OK。）开跑前 `ps` 看到的别的会话负载：一趟整轮门禁在跑 `checker-tier-crash-point-replay` 与 `cargo test --release -p singlefs-checker-tier`（别的会话，原样前缀见 `research/prompts/m4-segment-code-r3-sonnet-model/out/ps-before.txt`），没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；本腿每条命令加 `nice -n 19`、编译与测试经 `capped.sh 31` 与 `run-with-memory-cap.sh`，同一时刻只起一条 cargo，没有等锁。

## 进度（报告分段落盘，重起先读这里已写完的段）

- 段 0：开工核对（本段）。
- 段 1：Q4 复跑——四条腿的复跑脚本在新草稿目录的结果。
- 段 2：Q4 数——两份判决第二节引的数现算一遍，与脚本缺的几步。
- 段 3、4：Q1 逐条复核与替出局一方的辩护。
- 段 5：Q3 清单全不全、条款读窄。
- 段 6：判决条目表。
- 段 7：要主 agent 跑的命令、没做什么、模型目录、草稿目录清理、引文核对。

## 总览（一句话版）

- Q4：四条腿的 `rerun.sh` 在新目录各跑一遍，全部与腿交的输出逐行相同（r1 正推 `result=same`，r1 攻方 7/7，r2 辩方 10/10，r2 攻方 15/15）；两份判决第二节引的数现算 59 项对上、2 项对不上（「24 格红」口径、950093 的 9 与 11 不在腿交的 out 文件里）；脚本缺的步我补跑了三处（310 段、309 条、既有用例），结果同腿报告。
- Q1：站得住的结论都还站得住；够不着的是依据，不是结论：T9 的 Verus 一半、T7 的产品路径后果、T10 的读法、F1「与 D23 相容」的字面一层，「两次抽样没打中」够不到它们。
- Q3：有 12 条两腿或单腿碰到、没落进改法或欠账的形状（3.1；其中 4 条判决正文写过、去向栏却没有对应的改法或欠账行：#1、#7、#8、#12）；"站得住"所依的条款有 4 处读法偏窄（T7、T8、T10、F1，3.3），第二轮判决的 H5 一句与 U2 冲突（主 agent 已在第三轮正文改了）；第一轮 T10 的读法两轮都没被攻过。

## Q4　判决引用的数能不能核（段 1：四条腿的复跑）

### 4.1　怎么跑的

- 四条腿的 `tools/rerun.sh` 原样起：`--repo` 仓根，`--frozen` 各自那一轮的冻结副本（r1 两条用第一轮，r2 两条用第二轮），`--work` 新目录（`~/.local/share/singlefs/work/m4-segment-code-r3-defense/runs/<腿>`），`--model` 腿的模型目录；整条经 `nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G`，线程 31。
- 前台命令不超过 4 分钟，所以起的是各脚本的复制品 `staged/rerun-<腿>.sh`，与原脚本只差一行（让同一个 `--work` 能分阶段再起；四份 diff 相同），原样：

```
$ diff m4-segment-code-r1-sonnet-model/tools/rerun.sh staged/rerun-r1-sonnet.sh
64c64
<   if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
---
>   if [[ -z "${RERUN_ALLOW_NONEMPTY:-}" && -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
```

- 编译目录种子：拷主 agent 上两轮复跑留下的 `target`（只读拷，原目录没动）：r1 正推 5.0G、r1 攻方 897M、r2 辩方 2.7G、r2 攻方 1.5G；省的是首次全量编译，被编代码仍是各轮冻结副本（`find … -exec touch` 后重编，`Compiling singlefs-core …` 在各 `logs/` 里）。
- r2 攻方 `RERUN_ONLY` 分五次起（prepare+build+五个小阶段 / rollback-shapes+mount-reads 撞了 230 秒时限没跑完 / mount-reads / rollback-shapes / model-fix），每个阶段各有一次完整跑完的输出，比对只用那几次；撞时限那一次（`rc=124`，之后无残留进程）留下的半截日志已被后两次完整跑覆盖。

### 4.2　结果（原样；`out/legs-rerun-lines.txt` 是同一份，在模型目录里）

| 腿 | 脚本末行 | 挂钟 |
|---|---|---|
| r1 正推（Sonnet） | `name=sonnet_forward_rerun_matches_expected result=same` | 90 秒 |
| r1 攻方（Opus） | `name=rerun_total same=7 differ=0` | 90 秒 |
| r2 辩方（Sonnet） | `name=rerun_total same=10 differ=0` | 205 秒 |
| r2 攻方（Opus） | 四次完整跑合计 `same=11`、`1`、`2`、`1`（共 15）、`differ=0`（另有一次撞时限，见 4.1） | 100 + 74 + 178 + 160 秒 |

```
## r1-sonnet-full
name=sonnet_forward_rerun_matches_expected result=same
## r1-opus-full
name=rerun_compare file=remount-sweep.txt identical=true lines=16
name=rerun_compare file=seed126-prefix-suffixes.txt identical=true lines=10
name=rerun_compare file=rollback-minimal-trace.txt identical=true lines=12
name=rerun_compare file=i35-seed2022.txt identical=true lines=9
name=rerun_compare file=compaction-seed1055-trace.txt identical=true lines=15
name=rerun_compare file=t11-original.txt identical=true lines=4
name=rerun_compare file=t11-mutated.txt identical=true lines=6
name=rerun_total same=7 differ=0
## r2-sonnet-full
name=rerun_compare file=t4-r2-checker.txt identical=true
name=rerun_compare file=f1-r2-core.txt identical=true
name=rerun_compare file=f1-r2-core-with-f1-neutered.txt identical=true
name=rerun_compare file=f1small-r2-core.txt identical=true
name=rerun_compare file=f5-r2-core.txt identical=true
name=rerun_compare file=f6-reference-filesystem-snapshot-sets.txt identical=true
name=rerun_compare file=arithmetic.txt identical=true
name=rerun_compare file=t10.txt identical=true
name=rerun_compare file=seeds.txt identical=true
name=rerun_compare file=t11.txt identical=true
name=rerun_total same=10 differ=0
## r2-opus-1
name=rerun_compare file=reference-filesystem-probe.txt identical=true（两边去掉计时那一行）
name=rerun_compare file=trace476.txt identical=true
name=rerun_compare file=dump535.txt identical=true
name=rerun_compare file=dump640.txt identical=true
name=rerun_compare file=dump3395.txt identical=true
name=rerun_compare file=trace297.txt identical=true
name=rerun_compare file=transient-completing.txt identical=true
name=rerun_compare file=transient-1055.txt identical=true
name=rerun_compare file=transient-issue-pairs.txt identical=true
name=rerun_compare file=checkpoint-cost-margins.txt identical=true
name=rerun_compare file=empty-file-set-snapshots.txt identical=true
name=rerun_total same=11 differ=0
## r2-opus-3
name=rerun_compare file=mount-transient-reads.txt identical=true
name=rerun_total same=1 differ=0
## r2-opus-4
name=rerun_compare file=rollback-shapes-sweep.txt identical=true
name=rerun_compare file=rollback-shapes-samples.txt identical=true
name=rerun_total same=2 differ=0
## r2-opus-5
name=rerun_compare file=model-rollback-relocations-before-and-after.txt identical=true
name=rerun_total same=1 differ=0
```

r1 正推脚本没有逐文件比对行，它比的是汇总：新跑出的 `summary.txt` 与腿交的 `out/expected.txt` 的 sha256 相同（`2b3ff31dc36c165005dce8248fb33e219902c1b912c62162a94a49b4c3427dd4`，两个文件）。

### 4.3　判决第一节的核查数能不能重现

判决写「cite-check（以工作树为根）」，工作树的 `crates/` 此后打了修复批二三，今天照那句话跑会大片对不上；换成「该轮冻结副本 + 仓根」两个根就与判决的数一字不差。同一份报告两种根的原样末行：

```
$ python3 scripts/agents/cite-check.py research/prompts/m4-segment-code-r1-sonnet-output.md          （默认根，即今天的工作树）
  ✗ 核了 110 处引文、25 处裸行号，66 处对不上，25 处没判（其中裸行号没有引文可比 25 处；逐处列在上面）
$ python3 scripts/agents/cite-check.py research/prompts/m4-segment-code-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m4-segment-code-r1/repo --root .
  ✓ 核了 110 处引文、25 处裸行号，对上 110 处，没判 25 处（其中裸行号没有引文可比 25 处；逐处列在上面）
```

另三份报告同样（`research/prompts/m4-segment-code-r3-sonnet-model/out/cite-<腿>.txt` 是完整输出）：

| 报告 | 默认根（今天的工作树） | 该轮冻结副本 + 仓根 | 判决第一节写的数 |
|---|---|---|---|
| r1 攻方 | 24 处对不上 | `✓ 核了 31 处引文、5 处裸行号，对上 31 处，没判 5 处` | 31 / 5，对上 31 |
| r2 辩方 | 30 处对不上 | `✓ 核了 65 处引文、29 处裸行号，对上 65 处，没判 29 处` | 65 / 29，对上 65 |
| r2 攻方 | 23 处对不上 | `✓ 核了 37 处引文、6 处裸行号，对上 37 处，没判 6 处` | 37 / 6，对上 37 |

结论：数对得上，但判决里「以工作树为根」那句话今天复现不了，要写「以该轮冻结副本与仓根为根」。推翻条件：今天的工作树上照判决那句话（不加 `--root`）跑，仍得「对上 110 处」。

### 4.4　模型目录完整性与腿的自跑

- `sha256sum -c SHA256SUMS`：r1 攻方模型目录 15 个文件 OK、0 个不 OK；r2 攻方 32 个 OK、0 个不 OK；r2 辩方 26 个 OK、0 个不 OK（r1 正推模型目录没有 `SHA256SUMS`，13 个文件）。
- 三份报告 sha256 前缀与调度记录里写的一致：`8b61036e`（r1 攻方）、`3b94c40f`（r2 攻方）、`04fbd6f3`（r2 辩方）。
- 四个 `rerun.sh` 的 `--selftest` 今天全红（原样首行：`✗ 自检：拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 1：cp: cannot stat '/tmp/rerun-templat…`）：模板自证期望 `rerun_steps` 是模板自带的那一小段，腿换成自己的步骤之后自证不适用；它判的是模板，不判腿的复跑；哪一道闸会跑它我没去查，写成待查。

## Q4　（段 2：判决第二节引的每个数现算一遍，与复跑脚本缺的几步）

### 4.5　数对数

做法：`python3 -I research/prompts/m4-segment-code-r3-sonnet-model/tools/check-numbers.py --prompts research/prompts --frozen-r1 <第一轮冻结副本> --frozen-r2 <第二轮冻结副本> --repo .`，只读腿交的 `out/` 文件、两轮冻结副本与 kb，不起任何编译。每项一行 `name=check id=… expected=<判决或报告里的数> actual=<现算> result=ok|differ`，原样在模型目录 `out/check-numbers.txt`（62 行）。末行与两处对不上，原样：

```
name=check id=r2-R1-cells-with-reds source=r2-sonnet out/f1-r1-core.txt 有违例的格数（判决写「24 格红」，按有池级 checker 违例的格读） expected=24 actual=34 result=differ
name=check id=r2-R1-seed950093-9-vs-11 source=r2-sonnet out/seeds.txt 950093 一行含「上限 Some(9)」与「实现报上限 11」 expected=True actual=False result=differ
name=check_total ok=59 differ=2
```

| 判决引的数（所在判决第二节的格） | 现算 | 判定 |
|---|---|---|
| r1 T4：种子下标 2022 缩到 5 步 | `tools/rerun.sh` 的 `ATTACK_KEEP` 5 个，输出含 `RED Operation(4) I-3.5` 与 `HIT true` | 对上 |
| r1 T5：16 格中 6 格打中 | `remount-sweep.txt` 16 行，`ending=NewFinding` 6 行 | 对上 |
| r1 T6：种子下标 1055 缩到 13 步；`segs0` 3310、记账 3309 | `ATTACK_KEEP` 13 个；`expected.txt` 同时含 `segs0_written_in_the_intent=3310` 与 `accounting_row_11_written_by_the_issue_publish=3309` | 对上 |
| r1 T7：32000 次比较 | `expected.txt`：`checked_comparisons=32000` | 对上 |
| r1 T8：第 4453 行；三条 harness 用例红 | 第一轮冻结副本 `crates/mutations.tsv` 第 4453 行含「回收记下的段不论整段可不可再分配都发 discard」；`expected.txt` s03 `failed_tests=3` | 对上 |
| r1 T9：`/home/fy5090/.local/share/singlefs/work/m4-segment-code-r1/repo/crates/singlefs-core/src/compaction_plan.rs:902`：「span_slots: 1,」 | 该行在 | 对上；「78 verified」本机无 verus，没核 |
| r1 T1、T5、T10 的锚点（`mounted_session.rs` 第 1390 行、`mount.rs` 第 8187 行、`recovery.rs` 第 2968 行，第一轮冻结副本） | 三处都在（`check-numbers.txt` 的 `r1-anchor-*` 各项） | 对上 |
| r1 T2：`SnapshotLifetime` 只命中枚举与两个臂 | 核心源码含该词 11 行：枚举、两个臂、判定函数本体、它自己的单元测试；没有任何开关 | 结论对上，「只命中…两个臂」口径松（判决点名 3 处，实际 11 行） |
| r1 T11：第 4346 行；原样绿、改坏红 | `t11-original.txt` 2 passed，`t11-mutated.txt` 1 failed | 对上 |
| r2 R1：49 个场景；第一轮核心 24 格红、108 条违例；修后 0 格 | `f1-r1-core.txt` 49 格、没走完 24 格、违例 108 条；有违例的格 **34** 个；`f1-r2-core.txt` 没走完 0、违例 0 | 对上，但「24 格红」是「没走完的 24 格」，违例分布在 34 格里（10 格走完了但池级 checker 红） |
| r2 R1：写死形状 240 格 | `rollback-shapes-sweep.txt` `CASE` 240 行，`hit=true` 0 行 | 对上 |
| r2 R1：随机历史 2924 段 | 两个 `random-summaries*.txt` 的 `seeds=` 之和 2924；`problems=` 之和 17（报告：原样模型 15 + R7 一形 2） | 对上；判决没写其中 1891 段是在改过模型的副本上跑的（报告 R8 一节写了） |
| r2 R1：4 GiB 盘 310 段中 3 段 | `random-summaries.txt` 里比重 with-rollback 种子下标 [400,555)∪[600,755) 的 `seeds=` 之和 310、`problems=` 之和 3、`steps_checked=` 之和 12636；文件里没有批标签，310 要靠种子区间反推 | 对上；**脚本没这一步，我补跑了，见 4.6** |
| r2 R1：小盘 496 段中 12 段，改模型后 0 | `model-rollback-relocations-before-and-after.txt`：原样模型 `seeds` 之和 496、`problems` 之和 12；改后 496 / 0；复跑脚本含这一步，逐字节相同 | 对上 |
| r2 R1：种子 950093 第 18 步模型 9、实现 11 | `out/seeds.txt` 的 950093 那行在 330 字处截断（331 字节），**不含 9 与 11**；新跑的日志（`research/prompts/m4-segment-code-r3-sonnet-model/out/seed-rollback-950093.log`）含 `上限 Some(9)（D16（发布语义） 已定项 1）` 与 `实现报上限 11` | 数是对的，但腿交的 out 文件盖不住它，复跑脚本比的也是截断后的行 |
| r2 R2：种子 940015、940105、960004、950093；297、6318 | `seeds.txt` 四个 `ending=NewFinding`；`trace297.txt`、`seed6318-small-device.txt` 含 `NewFinding` | 对上 |
| r2 R3：`the_completed_intent_yielded(3310, 3310)=false`、`(3310, 3309)=true` | `arithmetic.txt` 含 `(3310 vs 3310)=false` 与 `(3310 vs 3309)=true` | 对上 |
| r2 R5：115 格；6/6 | `empty-file-set-snapshots.txt` 两条 `SWEEP` 的 `cases` 99 + 16 = 115、`hits` 0；`f5-r1-core.txt` 没走完 6、`f5-r2-core.txt` 0 | 对上 |
| r2 R6：20 行探针 | `f6-reference-filesystem-snapshot-sets.txt` 20 行 | 对上 |
| r2 R7：`ckpt_cost` 余量 528–534 槽 | `checkpoint-cost-margins.txt` 22 个 `ckpt_cost − fixed_point_slots_written`，最小 528 最大 534 | 对上；全是 5 片叶的小池上的数 |
| r2 复核：F9 的 309 条全绿；六个必红 | 腿报告 `logs/checker-tests.log` 的 266 + 11 + 12 + 15 + 4 + 1 = 309（另 2 条 ignored）；**复跑脚本没这一步，我补跑了，见 4.6** | 对上 |

### 4.6　复跑脚本缺的几步，我补跑的原样输出

**（一）r2 攻方：with-rollback 310 段、3 段不合。** `tools/rerun.sh` 只含小盘 496 段那一步，不含 `random-summaries*.txt`；腿的 `src/run_random_batch.sh` 把草稿路径写死（第 5 行 `W=/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2-attack`、第 8 行从 `$W/repo/target/release/deps/` 找二进制），腿交回时已删，新目录里起不来。我写了不依赖路径的 `tools/random-batch.sh`（同一写法：每件 2G 内存包装、每件一个 rc 文件、一个不带参数的 `wait`），在原样模型的第二轮冻结副本拷贝上 release 重编后跑种子下标 [400,555) 与 [600,755)（每件 5 段，62 件）：

```
[400,555)：seeds=155 problems=2 steps_checked=6297   ATTACK_PROBLEM 种子偏移 476、535
[600,755)：seeds=155 problems=1 steps_checked=6339   ATTACK_PROBLEM 种子偏移 640
合计 seeds=310 problems=3 steps_checked=12636        （out/with-rollback-310.txt 末行：total seeds=310 steps_checked=12636 problems=3）
```

62 件逐件（`seed_start`、`seeds`、`steps_checked`、`problems`）与腿的 `out/random-summaries.txt` 里同种子起点的行全部相同（我的脚本比出 `62 62 True`，差异列表空）。前台时限下第一批 27 件跑完、4 件被截断，那 4 件（起点 420、445、460、470）单独补跑，`problems=0`，与腿的行相同。判决「310 段中 3 段」成立。

**（二）r2 辩方：必红与已知坏镜像 309 条全绿；既有用例绿。** 在第二轮冻结副本的新拷贝上 `cargo test --offline -p singlefs-harness` 六个目标（`out/f9-checker-tests.txt`）：

```
running 12 tests / test result: ok. 12 passed; 0 failed; 0 ignored
running 15 tests / test result: ok. 15 passed; 0 failed; 0 ignored
running 268 tests / test result: ok. 266 passed; 0 failed; 2 ignored
running 11 tests / test result: ok. 11 passed; 0 failed; 0 ignored
running 4 tests / test result: ok. 4 passed; 0 failed; 0 ignored
running 1 test / test result: ok. 1 passed; 0 failed; 0 ignored
```

（每组原样是 `running N tests` 与 `test result` 各一行，这里并成一行写；逐行原文在 `out/f9-checker-tests.txt`。）12 + 15 + 266 + 11 + 4 + 1 = 309。F2、F3、F4、合取 7 的四个既有用例文件（`out/existing-fix-tests.txt`）：8、2、2、3 条全 `ok`，含 `the_intent_deletion_refused_after_a_batch_emptying_the_region_publishes_the_batch_carrying_the_intent_and_the_next_publish_deletes_it`、`an_issue_opening_a_new_cluster_segment_writes_the_empty_segments_of_its_accounting_not_the_count_before_it`、`forcing_the_deadlist_arm_keeps_every_replaced_user_visible_unit_without_any_snapshot`、`an_overwrite_on_a_pool_without_an_empty_cluster_segment_is_refused_by_the_empty_cluster_segments_floor_by_default`。

**（三）没补、也没法补的。** 2924 段里除上面 310 段与脚本已含的 model-fix 一步（小盘 496 段，原样与改模型各一遍）之外的其余段（4 GiB 的 `r2-mix` 155 段、with-rollback 早期 72 段、384/256/240 槽三档小盘的其余批）没重跑：脚本没有这些批，文件里没有批标签，「1033 段原样模型 / 1891 段改过模型」的分法无从核；重跑要主 agent 派（见末节）。r1 判决的「78 verified」「零轮读法 17 条」没法复跑（本机无 verus；17 条是正推报告里的人工表）。

## Q1　前两轮「站得住」「打中」的依据够不够得着（段 3）

每条写：结论、依据现查了什么、够不着的地方、是不是同样打中别的方案、什么现象会推翻。「站得住」指判决的结论，「够不着」指判决用来支撑它的那一句依据，两件事分开判。

### 1.1　F1（回退映射只重写位置变了的路径）与它背后的 T5 打中

- **打中的依据够得着。** 16 格里 6 格（`research/prompts/m4-segment-code-r1-opus-model/out/remount-sweep.txt`，4.5 的数对上）；49 个场景第一轮核心 24 格没走完、108 条违例（`research/prompts/m4-segment-code-r2-sonnet-model/out/f1-r1-core.txt`）；把 F1 的函数改成返回空表之后第二轮核心的输出与第一轮核心的输出逐字节相同（`f1-r2-core-with-f1-neutered.txt` 与 `f1-r1-core.txt` 同 sha256 `3955116cf1cd9fa839a692c883f9bc86468b2aec62fee8a15ea183d7a87ae787`，腿报告 13 节），所以 F1 就是让这 49 格从红到绿的那一处，不是别的改动。两份输出我在新目录复跑都逐行相同（4.2）。
- **F1 站得住。** 修后 49 格 0 红、60 格单叶映射树 0 红（`f1small-r2-core.txt`）、240 格写死形状 0 命中、310 + 496 段随机历史除模型一侧的 3 + 12 段外 0 红（4.5、4.6）。
- **「F1 与 D23 相容」这句依据只够到检查器与模型一层，够不到字面一层。** D23 那一段把中央映射树明列在「按这次发布照调整之后的分配器现算」那一串里：`.claude/kb/decisions/23-journal的角色与格式.md:378`：「按这次发布照调整之后的分配器现算，不从 R_old 那棵账载入」。F1 的做法是照现行版 COW 整棵树、再对位置与目标版不同的 key 重写（第二轮冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2/repo/crates/singlefs-core/src/mount.rs:8191`：「central_mapping_tree: current.central_mapping_tree.clone(),」；key 集合来自 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2/repo/crates/singlefs-core/src/mount.rs:8235`：「fn mapping_keys_whose_target_locations_differ_from_the_current_mapping(」，位置取目标版的 `mapped_units`）。这介于两者之间：节点与没变的条目照现行版，变了位置的条目取目标版的位置；后一半不是「分配器现算」。两种读法在每一步上给出同一棵树只被间接证明（池级 checker 的 I-3.9、I-3.15 与模型逐版比不红），没有一格把 F1 之后那棵映射树与「分配器现算」出来的树逐字节比。所以判决写「F1 与 D23 相容」不如写「F1 在所有量过的历史上与 D23 的现算给出同样的可观测结果」。
- **同样打中别的方案吗。** 判决 F1 字面（整棵取目标版）：是，它要把映射树节点纳入复活、验盘、释放集，而 `is_a_user_visible_unit` 对这两类是 false（腿报告 1.2 节，我没重验），也与 D23 同一行冲突，出局的理由站得住。「把所有用户可见 key 所在叶全标重写」：`ckpt_cost` 余量只在 5 片叶的小池上量过（528–534 槽，4.5），默认 4 GiB 盘只有算术（叶至多 721 片 < 1058，腿报告 `research/prompts/m4-segment-code-r2-sonnet-output.md:100`：「限度：这是算术，不是量出来的」），全标重写比 F1 多写，两者在大池上的上界同样没有量过。
- **推翻条件**：对一次回退，F1 之后的映射树与「分配器现算」的树有一个 key 或位置不同；或一段合法历史上 I-3.9 / I-3.15 在 F1 之后红。

### 1.2　F4（只供测试的强制开关）

- 站得住。第一轮的缺项成立：第一轮冻结副本核心源码含 `SnapshotLifetime` 的 11 行里没有开关（4.5）；F4 之后三个既有用例在第二轮冻结副本拷贝上 `ok`（4.6（二））；条款在 `research/prompts/m4-snapshot-r1-main-verification.md:32`：「只供测试的开关：强制「立即释放 / 进 deadlist」」。
- 够不着的是覆盖面：开关挂在写入口上（腿报告 `research/prompts/m4-segment-code-r2-sonnet-output.md:200` 同一节），会话与随机历史造的写入口每次新开，强制不了；「两臂各一条用例」只在直接造写入口的测试里满足。
- 同样打中别的方案：开关挂分配器能让会话也强制，但攻方指的纪元 0 条目隐患（`research/prompts/m4-segment-code-r2-opus-output.md:238`：「纪元 0 的条目以后建的任何快照都放不掉」，推的、没造用例）出在「强制进 deadlist 那一臂」的语义上，挂哪里都在，所以不分高下。这一条两份判决都没写（进 Q3）。
- 推翻条件：产品路径出现一处 `set_snapshot_lifetime_judgement` 调用；或 F4 两臂的某个副作用不经 `verdict_of`。

### 1.3　F5（空文件集合上建快照放开）

- 站得住。第一轮核心 6/6 没走完、第二轮 0/6（4.5，复跑 `f5-r2-core.txt` 逐行相同）；核心与模型 115 格对上（4.5）；D16 把建快照写成「算非空」：`.claude/kb/decisions/16-发布语义.md:46`：「建快照那次发布算非空」，与 D23 候选集一行一致：`.claude/kb/decisions/23-journal的角色与格式.md:382`：「删完之后有树、文件集合为空的那一版写过文件，算带文件」。
- 够不着的是 RefFS 一侧：翻译到 unlink 就停，所以「空文件集合上的快照」从没进过 RefFS 的判定（腿报告 `research/prompts/m4-segment-code-r2-sonnet-output.md:218` 的「F5 在 RefFS 一侧是文字上的放开，没有取样点」）。判决 R5 一行写「核心与模型 115 格对上」，没说 RefFS 一侧只有单测。
- `history.rs` 里 `CreateSnapshot` 的文档注释还写着拒绝，F5 之后过时，第三轮冻结副本仍在：`/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/singlefs-harness/src/history.rs:543`：「现行那一版有树时调（有树而文件集合为空的一版也调：会话拒成 `NoFileVersionToChange`）」（进 Q3）。
- 推翻条件：入库装置上 6 格任一格红；或空文件集合上建快照之后冷启动 / 可写挂载读不出快照。

### 1.4　F6（RefFS 恢复之后的快照集合）

- 站得住：20 行翻译层探针（4.5）；`allowed_snapshot_sets` 在翻译里只有两处用法。够不着的是「核心能落到而 RefFS 判不允许」这一半：翻译今天喂不到被抛弃的根（攻方 `research/prompts/m4-segment-code-r2-opus-output.md:271` 一段），所以 F6 的整个读法只在 RefFS 单元层被量过。
- 这一处与 H5 的关系见 Q3：判决 H5 一句与 `research/prompts/m3-formal-r6-main-verification.md:28` 的 U2 冲突。

### 1.5　第一轮 T7（合取 7，用户定「维持，记后果」）

- **站得住，是因为用户定过两次。** 条款：`.claude/kb/decisions/03-空间分配.md:146`：「**只拦前台写入类发布**：删除类、整理批、挂载与回退的发布免这一条」；用户 2026-10-16 定「维持，记后果」（`research/prompts/m4-segment-code-r1-main-verification.md:29`）；更早 gc 轮用户已经收过「合取 7 冻结」这一类：`research/prompts/m4-gc-r1-main-verification.md:42`：「H-G4-2（合取 7 冻结）用户定案收」。
- **依据够不着的两处。**
  - 「出路」一句：判决写「前台写入类发布全拒、不推抬 F」，第二轮辩方报告写成「用户的出路是删文件或删快照，不是写」（`research/prompts/m4-segment-code-r2-sonnet-output.md:285`：「出路：删除类发布免」那一条）。第一轮正推自己带着条件：`research/prompts/m4-segment-code-r1-sonnet-output.md:221`：「删掉的东西没占满一整段时放不开」。两轮没有一次造出「删除之后全空聚簇段数回到 N₀ 以上」的取样点，所以「出路」只是「删除类发布不被合取 7 拦」（读码），不是「删了就能再写」。
  - 条款没连着读：D3 自己有 `.claude/kb/decisions/03-空间分配.md:193`：「只要 `df` 报出的空闲 ≥ s，写 s 字节就必须成功」，与 `.claude/kb/decisions/03-空间分配.md:279`：「就等于承诺后台整理是**常驻的、自动的、用户无需知道的**」。产品配置下整理一个意图都不签（第二轮冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2/repo/crates/singlefs-core/src/compaction_plan.rs:346`：「batch_size: CompactionBatchSize::NotDecidedByTheClauses,」，第三轮冻结副本该行在 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/singlefs-core/src/compaction_plan.rs:399`），合取 7 一旦拒写，`df` 与拒绝的关系就是这两条在碰。gc 轮把同类形状（暂停期间 R 排他造假性 ENOSPC）也点过名、收成零轮和用户定案（`research/prompts/m4-gc-r1-main-verification.md:42`：「H-G1-1（暂停期间 R 排他造假性 ENOSPC，与 D3 已定项 9 第 1 条）」）；本轮判决没提这条先例，也没把「产品路径永不起整理」写进用户当时看到的选项，只有一句「记后果」。
- **R8 的「两次没打中」够不着它。** 第二次抽样的 T7 一行（腿报告 `research/prompts/m4-segment-code-r2-opus-output.md:346`）答的是 R 排他、瞬时读错、零个全空段造不出；随机历史里的整理靠测试开关起跑（`/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2/repo/crates/singlefs-harness/src/history.rs:553`：「产品路径的批大小条款没写、走不进去」），不是产品配置。「产品配置 + 零个全空段 + 前台写」这件事两次抽样都没造出（第一轮正推：「这一格是推的」，同一行 221；第二轮：`research/prompts/m4-segment-code-r2-opus-output.md:324`：「造不出。带 TRACE 的随机历史里记了每次签发之前那一版按记录口径的全空段数」）。照"几份样本按每一格答的是哪件事判"，这件事只有读码一份，没有观测。
- 同样打中别的方案：把 `N₀` 改 0（关）违 D3 登记表第 7 行，让产品路径起整理要等 E10 的批大小与水位，判决已列；这两条路在"产品配置下合取 7 拒写"这件事上没有差别，选哪条是用户的事。
- 欠账没落：`.claude/kb/checks-owed.md` 里 `合取 7` 命中 C621（整理落地时要动的删除条款没有清单外的检查）与 C826（三条改过的用户定案句要复核），`N₀` 命中 C139（整理停摆之后释放也发不出去）与 C820（N₀ 算式没有计数模型）；四行都不写"产品路径整理不起跑、默认 N₀ 开着、前台写被拒而 df 有空闲"这个后果（3.1 #8）。
- 推翻条件：造出产品配置下的合法历史，合取 7 拒写而 `df` 空闲 ≥ s，且之后只靠删除类发布写回成功——那「出路」成立；或相反，删光也回不到 N₀。

### 1.6　第一轮 T8（discard 少发，不改代码，记欠账）

- 站得住。机理读码对：待发集只在回收那一刻记、整交即摘（第二轮冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2/repo/crates/singlefs-core/src/allocator.rs:791`：「self.segments_reclaimed_not_yet_discarded.insert(segment);」，`/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2/repo/crates/singlefs-core/src/allocator.rs:830`：「segments_to_discard.push(self.segment_start_slot(segment_index));」）。安全一侧：设备日志判器判每个 discard 落在可发区间里，没造出 discard 打到仍被引用的段（27 段 0 红，腿报告）。
- 第二轮判决（`research/prompts/m4-segment-code-r2-main-verification.md:26`）写「G5 字面不要求补发」，依据的是 `research/prompts/m4-gc-r1-main-verification.md:36`：「按 64 槽对齐的整段发」。这一句管时机（发布返回之后）、谓词和粒度，对"谓词后来才变真、该段此前被扣下"没有话；沉默不等于排除，也不等于要求。判决落在"少发不破坏一致性、记一行欠账"上，这个落点两种读法下都不错；但"字面不要求"这个说法过强，写成"字面没说"才对。
- 没有检查判完整性：判器只判"发出去的合法"，不判"该发的发了"。
- 同样打中别的方案：补发（把扣下 / 报错的段放回待发集，一行）会对 `Unsupported` 设备每次重试，腿报告 `research/prompts/m4-segment-code-r2-sonnet-output.md:294` 已写；不补发则永远少发。两条在"哪一读法对"上没有观测能分。
- 欠账行没落（判决写的是「阶段同步时取号写进 `checks-owed.md`」，`research/prompts/m4-segment-code-r1-main-verification.md:30`）：`.claude/kb/checks-owed.md` 里 `少发`、`补发` 命中 0 行；第三轮正文列了这条是"欠账"，没说在哪。
- 判决里用来说"变异第 4453 行可杀"的是三条 harness 用例（4.5 对上）；但这一行自己点名的杀手仍是慢的那条：第三轮冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/mutations.tsv` 的 4452 行末列是 `every_crash_state_of_the_lifecycle_closure_recovers_reads_through_the_mapping_and_the_mount_continues_every_intent`，调度记录里这一行是超时（`records/2026-10-13-里程碑四预开发.md:219`），改点名的建议（正推第 8 条）没进任何改法。

### 1.7　第一轮 T10（坏 logged_ops 条目当损坏，可写挂载拒）

- 站得住的是行为：解码函数对补零非零、保留种类码、槽数不是 64、两条意图升序、条目降序五种坏条目都返回 `Err`（r1 正推 `out/expected.txt` s14 一组），不对齐 R 的坏条目在可写挂载拒成 `UnitMalformed`、只读挂载与冷走读收下（同文件 s13 一组），冷启动读得出同一条根（r2 辩方 `t10.txt`，复跑相同）。
- 读法是零轮：D8 `.claude/kb/decisions/08-核心索引结构.md:108`：「**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载」。判决（`research/prompts/m4-segment-code-r1-main-verification.md:32`）读成"说的是续做不挡挂载，不管盘上条目字节坏了"。字面还有一读："不挡挂载"挂在"提示类"这个类上。已有先例把提示类当可丢：`research/prompts/m4-gc-r1-main-verification.md:33` 的 G2 一行写着签发之后 R 里出现隔离槽就丢掉这条意图连同 R 排他（K-c，零轮，提示类可丢）。两轮里这个读法只被辩方复核过，攻方只打了行为（瞬时读错注入），没有人攻过读法本身。
- 同样打中别的方案："丢坏条目照挂"对 R 不对齐 / 槽数不是 64 的条目走不通（不知道 R，排他无从装），对补零非零、保留种类码的条目走得通（R 可解析）；这五种在解码层是不同的错误值（`EntryRefused(…)`、`MoreThanOneCompactionIntent`、`EntriesNotInStrictlyAscendingKeyOrder`），挂载层只量过 R 不对齐那一种（拒成 `UnitMalformed`），其余几种在可写挂载上的结局没量过；判决没分。保留种类码拒收还可能有前向兼容的理由（我的论证，条款没写）。判决的"一致处理别的坏结构"不是条款，是类比。
- 推翻条件：用户或条款把"提示类的坏条目"定成可丢；或冷启动路径也该拒。

### 1.8　第二轮 R8：六格两次抽样没打中

- T1、T3：两次抽样答的是同一件事（建快照 / 删快照的随机历史与回退），第二次 4028 + 3610 次建快照、3337 次删快照，站得住。
- T2：第一轮正推的缺项（强制开关）靠 F4 解了；第二次抽样答的是"释放前读盘核隔离"，不是开关。"没打中"对的是前一件事，不是后一件，后一件由 F4 的用例守。
- **T9 够不着 Verus 一半。** 第一轮 T9 的打中是对拍输入域的缺口（`compaction_plan.rs` 第 902 行记录跨度全是 1，F8）；两次抽样没打中的是区域扫描拒绝（第二次 915 次签发 0 次拒绝，`research/prompts/m4-segment-code-r2-opus-output.md:347`），Verus 的 78 verified 两腿两轮都没复核（本机无 verus：`research/prompts/m4-segment-code-r1-main-verification.md:31`）。「T9 两次抽样都没打中」不覆盖证明本身；这一块以形式化轻型为准，第三轮正文的"已知"里写了主仓那份门禁点不中 `compaction_plan_proofs`，副本里整理核 20 项红（调度记录），没人在工作树上看到绿。
- T7：见 1.5。T10：见 1.7。
- 另：这六格里 T2、T7、T9、T10 在第一轮正推那边各有一条打中（F4、后果、F8、张力）；"没打中"只指攻方那一路。

### 1.9　其余条目（依据够得着的，短写）

- **第一轮 T4 / F9（池级 checker 的 I-3.5 标签）**：站得住。新目录里 `t4-r2-checker.txt` 逐行相同；四个快照种子第一轮的 `walk.rs` 红 13、0、8、2、第二轮 0、0、0、0（`out/t4-snapshot-seeds-*.txt`，4.5）；六个必红 / 已知坏镜像目标 309 条在新拷贝上全绿（4.6（二））。注意种子 7002035 在第一轮 `walk.rs` 上本来就 0 红，攻方报告里它不是打中的一个，判决没拿它当打中。
- **第二轮 R1 的模型一侧（H1）**：站得住（310 段 3 段、496 段 12 段、改模型后 0，全部复跑，4.5、4.6）。判决写「随机历史 2924 段没红」，报告 R8 一节写明其中 1891 段是在改过模型回退那一步的副本上跑的，判决没带这句；对"映射、释放、复活没红"这个结论不影响（池级 checker 的红与模型对拍的不合是两路），但读判决的人会以为 2924 段全在入库模型上。
- **第二轮 R2（F2 站不全，用户定「规划时补足」→ H4）**：打中的依据够得着（`seeds.txt` 940015、940105、960004，`trace297.txt`、`seed6318-small-device.txt`，4.2 逐行相同；第一轮冻结副本上同样复现）。
- **第二轮 R3（`segs0`，用户定「取签发之前」→ H3）**：`(3310 vs 3310)=false`、`(3310 vs 3309)=true`（`arithmetic.txt`）复跑相同。用户的选择与 D26 字段表那一行相反：`.claude/kb/decisions/26-后台整理与放置回收.md:63`：「签发那次发布写下的记账第 11 项（全空聚簇段数，D5（快照 / 空间记账机制） 已定项 4）里 `R` 所在盘的值，只作观测量」；判决写"字段表那一句归阶段同步"，第三轮正文"已知"里有，没有漏。
- **第二轮 R4、R5**：站得住（F4 见 1.2，F5 见 1.3）。
- **第二轮 R6（→ H5）**：判决那一句与 U2 冲突，见 3.3 最后一行。
- **第一轮 T1、T2、T3、T11**：T1、T3 站得住；T2 的缺项由 F4 补；T11 的第 4346 行 `t11-original.txt` 2 passed、`t11-mutated.txt` 1 failed 复跑相同。

### 1.10　替被判出局的一方辩护

1. **判决 F1 字面（中央映射树整棵取目标版）**。它最强的理由：回退那一版的映射 = 目标版的映射，没有"现行版 COW + 差量重写"的中间态，也不依赖"位置不同"的比较是否漏形（第二轮辩方报告 1.6 节的替代表自己写了：担心 `!=` 比较漏形时，全标重写是更稳的退路）。它出局的理由是 D23 同一行（`.claude/kb/decisions/23-journal的角色与格式.md:378`）把映射树列在"现算"那一串、且要扩复活 / 验盘 / 释放集到映射树节点。我替它说一句：这两条理由对"整棵取目标版"成立，对"目标版映射里用户可见 key 的那一部分取目标版"不成立——而 F1 取的正是后者，于是 F1 其实已经把出局方的一半吃进来了（位置取目标版），区别只剩"其余叶是否照现行版"。不翻案，但"出局"一词比实际更重。
2. **T10 的"丢坏条目照挂"**。理由见 1.7：D8 的字面可读成"提示类不挡挂载"，gc 轮已有"提示类可丢"的先例，补零非零与保留种类码两种坏条目里 R 可解析、排他可装。它对 R 不对齐的坏条目走不通。要让这条路和判决分出高下，要用户或条款把"不挡挂载"的射程说清；没有观测能分。
3. **T8 的补发**。理由：G5 描述的是"发布返回之后谓词为真的槽"这个集合，字面读法偏向完整；改动一行。代价：`Unsupported` 设备每次重试。没有观测能分，记欠账是保守落点。
4. **R2 的"模型加一臂"（替代 H4）**。理由（`research/prompts/m4-segment-code-r2-sonnet-output.md:170` 的替代 2、攻方 R7 一节）：不动产品代码、不碰 G3「一批只搬照抄的单元」、也没有 H4 在"只剩本来就重写的节点时全带上"那一支可能超过批大小的问题（第三轮正文写成"推的"）。出局理由：产品里仍会发一次没搬单元、照抄意图的发布，吃一个根槽与一格 `B_gc`（后果轻，`research/prompts/m4-segment-code-r1-opus-output.md:206`）。用户已定"规划时补足"，这是用户的事；两个后果谁重，没有数。

## Q3　开着的清单全不全；「站得住」所依的条款有没有读窄（段 5）

### 3.1　两腿或单腿碰到、没进任何一张表的形状

"表"指：第一轮判决的改法 F1–F10 与第二节、第二轮判决的 H1–H6 与第二节、两份"交用户"表、第三轮正文第二节的"实现员交下来的"和"已知"两段（`research/prompts/_m4-segment-code-r3-body.md:22`、`:23`），再加 `.claude/kb/checks-owed.md`。每一行我都现查了"在不在"，grep 命中数写在最后一列。

| # | 形状 | 谁碰到（出处） | 在不在表里 |
|---|---|---|---|
| 1 | `publish_relocation_batch` 的文档注释与行为不符 | 第一轮正推 `research/prompts/m4-segment-code-r1-sonnet-output.md:360`；写进了第一轮判决 T5 一行（`research/prompts/m4-segment-code-r1-main-verification.md:27`：「另：`publish_relocation_batch` 的注释与行为不符（正推）」） | **判决正文有，去向栏没有**：F1–F10、H1–H6 都不管它；第三轮冻结副本仍在，且同一文件里两处注释互相矛盾：`/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/singlefs-core/src/transaction.rs:7505`：「准入同删除类（式子那一判照判，合取 7 免）」对 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/singlefs-core/src/transaction.rs:8791`：「删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判」；实际是否判看 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/singlefs-core/src/admission.rs:1526` 的 `publish_has_an_ordinary_allocation` |
| 2 | 测试代码里 `CreateSnapshot` 的文档注释在 F5 之后过时 | 第二轮辩方 `research/prompts/m4-segment-code-r2-sonnet-output.md:219` | 没有；第三轮冻结副本仍在 `/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/singlefs-harness/src/history.rs:543` |
| 3 | `batches_published` 只数搬了单元的发布，"用户改动之后至多一批"因此可能一趟发三次 | 三条腿：第一轮攻方 `research/prompts/m4-segment-code-r1-opus-output.md:206`、第二轮辩方 `research/prompts/m4-segment-code-r2-sonnet-output.md:147`、第二轮攻方 `research/prompts/m4-segment-code-r2-opus-output.md:208`（推的、没造） | 第一轮判决 F2 采了"不把没搬单元的发布记成一批"（`research/prompts/m4-segment-code-r1-main-verification.md:42`），这个选择带出的后果（一趟三发）没进任何表（`checks-owed.md`、第二轮判决、第三轮正文里 `batches_published` 命中 0 行）；H4 之后该计数条件未变：`/home/fy5090/.local/share/singlefs/work/m4-segment-code-r3/repo/crates/singlefs-core/src/mounted_session.rs:2071`、`:2042` |
| 4 | "已经过去的发布数"不数挂载自己的写行与暖机（规则没说） | 第一轮正推 `research/prompts/m4-segment-code-r1-sonnet-output.md:362`（第 9 条）；第一轮攻方攻过预算没打中（`research/prompts/m4-segment-code-r1-opus-output.md:212`） | 没有（`已经过去的发布数` 命中 0 行） |
| 5 | D5 两行对"哪一份"写法不同：`.claude/kb/decisions/05-快照-空间记账机制.md:277`：「快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg) 的条目」，`.claude/kb/decisions/05-快照-空间记账机制.md:278`：「之后按批放掉 [lo, hi) 里 birth > P 的条目」 | 第一轮正推 `research/prompts/m4-segment-code-r1-sonnet-output.md:363`（第 10 条） | 没有；两份判决都不提 |
| 6 | 强制进 deadlist 那一臂造出纪元 0、以后没有快照放得掉的条目，回退又照抄现行版的 deadlist | 第二轮攻方 `research/prompts/m4-segment-code-r2-opus-output.md:238`（推的、没造用例） | 没有；判决 R4 一行写"站得住（两腿）" |
| 7 | R7 的后两形（零个全空段、合法镜像触发 `TwoRecordsMovableUnderOneMappingKey`）与更大池的 `ckpt_cost` 上界 | 第二轮判决自己写了去向 `research/prompts/m4-segment-code-r2-main-verification.md:24`：「后两形与更大池的 `ckpt_cost` 上界记欠账」 | **去向写了，欠账行没落**：`TwoRecordsMovableUnderOneMappingKey` 在 `checks-owed.md` 命中 0 行，`ckpt_cost` 命中的 11 行都是更早的项；第三轮正文"已知"里也没有 |
| 8 | 合取 7 的后果（产品路径整理不起跑、默认 `N₀` 开着、前台写被拒而 `df` 有空闲） | 第一轮正推（`research/prompts/m4-segment-code-r1-sonnet-output.md:221`）、判决 T7 一行 | **去向写了"记后果"，欠账行没落**（1.5） |
| 9 | 整理搬根单元会让根指针变，按 D16 的比法就是一个新的"非空"状态，会把更旧的状态挤出"4 个不同状态" | 第二轮攻方 `research/prompts/m4-segment-code-r2-opus-output.md:122`：「实现按字节去重，从新到旧的不同状态是 12、11、10、9」；第一轮正推 `research/prompts/m4-segment-code-r1-sonnet-model/out/expected.txt` 第 39、40 行 `user_visible_trees_changed=true`（搬快照根） | 没有；判决把这一处只当模型错（H1）关了。D16 的比法：`.claude/kb/decisions/16-发布语义.md:46`：「它的 (inode 树根指针, extent 树根指针, 全部快照树的根指针) 与每一条比它新、已计入的根都不同时才计入」；而 G8 的预算前提是 `research/prompts/m4-gc-r1-main-verification.md:39`：「GC 发布不改变用户可见状态」。搬根的一批是否改变，没有条款也没有判决说 |
| 10 | 暂停的意图（空间不够）之后再起 | 第二轮辩方 `research/prompts/m4-segment-code-r2-sonnet-output.md:95`（"今天没有取样点，不造"）、攻方 `research/prompts/m4-segment-code-r2-opus-output.md:160`：「暂停的意图（被空间拒）没造出来」 | 没有；判决 R7 只列四形，不含这一形 |
| 11 | 两块盘不对称（释放集 `any` 与 `all`） | 第二轮辩方 `research/prompts/m4-segment-code-r2-sonnet-output.md:93`（"造不出"）、攻方 `research/prompts/m4-segment-code-r2-opus-output.md:161`（"没造，推的"） | 没有 |
| 12 | 变异表第 4453 行（第二、三轮冻结副本里是 4452 行）的杀手是慢的那条，调度记录里超时；正推建议换成三条 harness 用例 | 第一轮正推第 8 条、攻方 `research/prompts/m4-segment-code-r1-opus-output.md:262` | **判决正文提了可换，F10 没点这一行**；第三轮冻结副本仍点名慢的那条（1.6）；可能归"变异轻型点名重跑"（验证段），但"超时的四行"（683、4371、4449、4453）没在第三轮正文"已知"里 |

另有一条不是形状而是读法：**T10 的读法两轮都只被辩方复核、没被攻过**（1.7），判决标"零轮"，第三轮正文的 Q1 把"T10 三处不改代码的判定"列成"前两轮判站得住的"，其实读法这一层是零轮。

### 3.2　交用户表

两份判决的"交用户"表（第一轮 §六、第二轮 §五）各两题，都标了用户已定，没有空缺；但表只列了题和答案，用户当时看到的选项与后果描述不在判决里（我核不了）。T7 的"记后果"和第二轮 H3 的"取签发之前"（与 D26 字段表相反）都落在这一处：后果写在判决一行里，条款与欠账没有跟上（3.1 第 8 条；H3 的字段表一句第三轮正文已列"归阶段同步"）。

### 3.3　「站得住」所依的条款有没有读窄

| 判决里的"站得住" | 它引的条款 | 读窄在哪 |
|---|---|---|
| T7 维持合取 7 | `.claude/kb/decisions/03-空间分配.md:146` | 只读了第 7 行"只拦前台写入类发布"；同文件已定项 9 第 1 条（:193）与已定项 14（:279）没连着读；gc 轮的先例（`research/prompts/m4-gc-r1-main-verification.md:42`）也没引。结论有用户定案撑着，依据一栏读窄 |
| T8 不补发 | `research/prompts/m4-gc-r1-main-verification.md:36`：「按 64 槽对齐的整段发」 | 沉默读成"不要求"（1.6） |
| T10 当损坏 | `.claude/kb/decisions/08-核心索引结构.md:108` | 一读取续做，另一读取类（1.7）；零轮 |
| F1 与 D23 相容 | `.claude/kb/decisions/23-journal的角色与格式.md:378` | 相容只证到可观测一层，不到"现算"字面（1.1） |
| H5（第二轮判决 §三） | **没引条款** | 判决写"恢复读出的内容对不上任何允许状态时……按恢复不被接受处置"（`research/prompts/m4-segment-code-r2-main-verification.md:36`），与 `research/prompts/m3-formal-r6-main-verification.md:28`：「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」直接相反。实现员照判决字面做就把 U2 整个吃掉并要删变异第 1883 行（调度记录 `records/2026-10-13-里程碑四预开发.md:262`），主 agent 没打那一段，第三轮正文第 21 行已改成照 U2。**这条是判决层面的漏引，已被第三轮正文补上，不再开着**——写在这里是因为它说明：两条腿都读了 RefFS 的注释「读出的内容一个都对不上（抛弃最新根的那次恢复不判读出的内容）时照取 S_0 那一刻的」（`/home/fy5090/.local/share/singlefs/work/m4-segment-code-r2/repo/crates/singlefs-reference-filesystem/src/lib.rs:724`），没有往上查这句括号的来历 |

没读窄的（我现查了条款）：F5 的"建快照算非空"（`.claude/kb/decisions/16-发布语义.md:46`）、F4 的 S3 末句（`research/prompts/m4-snapshot-r1-main-verification.md:32`）、F3/H3 的字段表两处（`.claude/kb/decisions/26-后台整理与放置回收.md:26`、`:63`，辩方报告逐字引了，判决摘要里提了字段表与停机谓词）、T5 的 G3（`research/prompts/m4-gc-r1-main-verification.md:34`）。

## 判决条目表（段 6）

判定用三个词：**站得住**（结论与依据都在）、**够不着**（结论可能对，但判决用的那句依据够不到）、**同样打中别的臂**（替代方案受同一条反例）。"证据"栏的 4.x、1.x、3.x 指本报告对应小节。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| r1 T1 建快照放开 → F5 | 站得住；RefFS 一半只有单测 | 6/6 → 0（4.5）；115 格；RefFS 翻译到 unlink 就停（1.3） |
| r1 T2 缺强制开关 → F4 | 站得住；覆盖面限于直接造写入口的测试 | grep 11 行无开关；3 个既有用例 ok（4.6）；纪元 0 隐患未进表（3.1 #6） |
| r1 T3 删快照 | 站得住 | 两次抽样同一件事，3337 次删快照（1.8） |
| r1 T4 打中（I-3.5 判据一侧）→ F9 | 站得住 | `t4-r2-checker` 复跑相同；四个种子 13/0/8/2 → 0；309 条绿（4.5、4.6） |
| r1 T5 打中（回退映射）→ F1 | 打中站得住；F1 站得住；"与 D23 相容"**够不着字面** | 16 格 6 格、49 格 24/108/34（4.5）；F1 变异输出与第一轮核心逐字节同（1.1） |
| r1 T6 打中（`segs0` 差一；腾空 R 不删意图）→ F2、F3 | 站得住 | 3310/3309、1055 的 13 步复跑相同（4.5）；F2 够不着同形另一支，第二轮已打中 |
| r1 T7 维持合取 7、记后果 | 结论站得住（用户定两次）；"出路"一句够不着；R8"没打中"够不着产品路径后果；欠账行没落 | 1.5；3.1 #8 |
| r1 T8 少发 discard、记欠账 | 站得住；"字面不要求补发"过强（字面没说）；欠账行没落；第 4453 行杀手仍是慢的 | 1.6；3.1 #12 |
| r1 T9 → F8 | F8 站得住；"两次没打中"够不着 Verus 一半 | `compaction_plan.rs` 第 902 行对上；78 verified 没人核（1.8） |
| r1 T10 不改 | 行为站得住；读法零轮，同样打中"丢坏条目"这条出局的路 | 1.7；3.3 |
| r1 T11 打中 → F6、F7 | 站得住 | 第 4346 行输出复跑相同；20 行（4.5） |
| r2 R1 F1 站得住；模型一侧打中 → H1 | 站得住；判决没带"1891 段在改过模型的副本上跑" | 310/3、496/12/0 复跑，62 件逐件相同（4.6）；`24 格红` 口径松（4.5） |
| r2 R2 F2 站不全 → H4、H2 | 打中站得住；"模型加一臂"是被替代方案 | 四个种子复跑相同；1.10 第 4 条 |
| r2 R3 `segs0` → H3 | 站得住；用户选择与 D26 :63 相反，归阶段同步 | `arithmetic.txt` 复跑相同（4.2） |
| r2 R4 强制开关 | 站得住 | 1.2 |
| r2 R5 空池快照 | 站得住 | 115 格、6/6（4.5） |
| r2 R6 RefFS 集合 → H5 | F6 站得住；**H5 一句够不着 U2**（已被第三轮正文改了） | 3.3 最后一行 |
| r2 R7 开着的形状 | 够不着：四形外还有 3.1 #10、#11 两形；"记欠账"没落 | 3.1 #7、#10、#11 |
| r2 R8 六格"两次没打中" | T1、T3 站得住；T2 答的不是同一件事；T7、T9、T10 够不着（产品路径后果 / Verus / 读法） | 1.8 |
| r2 复核：T4/F9、T7/T8/T10、F4–F7 | T4/F9、F4–F7 站得住；T7/T8/T10 结论站得住、依据各有读窄 | 1.2–1.7、3.3 |
| 两份判决第一节的核查数 | 数对得上；"以工作树为根"那句今天复现不了 | 4.3 |
| 两份判决第二节的数 | 59 项对上，2 项口径或出处有缺 | 4.5 |

## 要主 agent 跑的命令（段 7）

整串复跑（四条腿的 `rerun.sh` 各从零跑一遍 + 判决数 + 补的三步）。工作目录取仓根；线程 `RERUN_THREADS` 默认 31；内存上限 8G（脚本里每一步再套各自的包装）；`--work` 给一个新的空目录；`--frozen` 是第二轮冻结副本，第一轮冻结副本默认取它旁边的 `m4-segment-code-r1/repo`（`RERUN_FROZEN_R1` 可改）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-segment-code-r3-sonnet-model/tools/rerun.sh \
  --repo ~/.local/share/singlefs/work/m4-predev \
  --frozen ~/.local/share/singlefs/work/m4-segment-code-r2/repo \
  --work <新的空草稿目录>
```

跑完看：四行 `name=rerun_leg leg=… name=rerun_total same=7 differ=0` 一类（r1 正推那行是 `…matches_expected result=same`，r1 攻方 7、r2 辩方 10、r2 攻方 15），再看末行 `name=rerun_total same=5 differ=0`（`seed950093-ceiling.txt`、`check-numbers.txt`、`with-rollback-310.txt`、`f9-checker-tests.txt`、`existing-fix-tests.txt` 五份与模型目录 `out/` 逐字节相同）。时间：四条腿从零编（r1 正推与 r2 辩方 debug、r1 攻方与 r2 攻方 release）加两批 31 件随机历史，推的 40 到 90 分钟，我没从零量过（我这边的编译目录都是拷的种子，各部分分开跑过，见 4.1）。只想跑其中一步：`RERUN_ONLY="numbers"`（秒级）、`RERUN_ONLY="tests"`（约 2 分钟，可给 `RERUN_TARGET_SEED_DEBUG=<已编过的 debug 编译目录>`）、`RERUN_ONLY="legs"`、`RERUN_ONLY="batches"`。

要补跑、我没跑的（脚本里没有，也没有批标签）：2924 段随机历史里除 310 段外的其余批。要重跑就按 `research/prompts/m4-segment-code-r2-opus-model/out/random-summaries-small-devices-and-later.txt` 每行的 `weights=`、`seed_start=`、`seeds=` 逐批用 `tools/random-batch.sh`（盘宽用 `ATTACK_DEVICE_WIDTH`），原样模型与改过模型的批要分两棵树，批标签要自己加；这一块我判"值不值得"留给主 agent。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没读第三轮攻方报告与模型目录（禁读清单）。
- 没做 Q2（修复批三的修补本身）：那是攻方的格。3.1 里 #1、#2、#3 去第三轮冻结副本里 grep 过"还在不在"，没验过 H1–H5 的行为。
- 没跑 checker 档测试、崩溃点重放、GPU、QEMU、门禁阶段、变异整表；没起 `crates/singlefs-checker-tier` 的任何测试（补跑的是 `singlefs-harness` 的六个池级 checker 测试目标与四个既有用例目标）。
- 没复核 Verus 的 78 verified（本机无 verus），没复核第一轮判决"零轮读法 17 条"（人工表，不是数）。
- 没造"产品配置 + 零个全空段 + 前台写"的合法历史，没验"删除之后合取 7 放开"（1.5）；没造 T10 补零非零 / 保留种类码两种坏条目在可写挂载上的结局（1.7）；没拿"分配器现算"的映射树与 F1 之后的映射树逐字节比（1.1）。
- 没重跑 2924 段随机历史里除 310 段与 model-fix 一步之外的批，没核"1033 段原样 / 1891 段改过模型"的分法（4.6（三））。
- `tools/rerun.sh` 的 `legs`、`batches` 两步没有整串从零跑过：`legs` 四条腿的脚本我分阶段在带种子的编译目录上都跑通了；`batches` 一步跑到随机历史开跑（编译已完成），前台 235 秒时限截断，批本身我用 `random-batch.sh` 分四次跑完（400 起一批、补跑 4 件、600 起两批；310 段，62 件）；`tests`、`numbers` 两步经 `rerun.sh` 整条跑通（`name=rerun_total same=2 differ=0`；`numbers` 一步 `same=1 differ=0`）。
- 四个腿的 `rerun.sh --selftest` 今天全红，我没去判模板自证对腿的脚本适不适用（4.4）。

## 模型目录

`research/prompts/m4-segment-code-r3-sonnet-model/`：`tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps` 一段）、`tools/check-numbers.py`（判决数现算，只读）、`tools/random-batch.sh`（不依赖草稿路径的随机历史分件脚本）、`out/`（`check-numbers.txt`、`with-rollback-310.txt`、`f9-checker-tests.txt`、`existing-fix-tests.txt`、`seed950093-ceiling.txt`、`seed-rollback-950093.log`、`legs-rerun-lines.txt`、`ps-before.txt`、`cite-<腿>.txt` 四份）、`SHA256SUMS`。`bash -n` 过；`rerun.sh --selftest` 同四条腿一样红（模板自证只认模板自带的 `rerun_steps`）。

## 草稿目录清理

交回前删掉的（删前 `du -sh`，都在 `~/.local/share/singlefs/work/m4-segment-code-r3-defense/runs/` 下）：

| 目录 | 删前大小 | 是什么 |
|---|---|---|
| `r1-sonnet` | 5.0G | r1 正推复跑的工作目录（编译目录 5.0G + 仓拷贝 28M） |
| `r1-opus` | 925M | r1 攻方复跑（release 编译目录 897M + 仓拷贝 28M） |
| `r2-sonnet` | 3.6G | r2 辩方复跑的冒烟（`RERUN_STOP_AFTER=t4`，编译目录 3.0G、探针二进制 513M、仓拷贝 29M） |
| `r2-sonnet-full` | 3.7G | r2 辩方复跑整串（编译目录 2.7G、探针二进制 1.1G、仓拷贝 29M） |
| `r2-opus` | 1.5G | r2 攻方复跑（含 release 编译目录的仓拷贝，模型回退补丁已打在上面） |
| `r2-opus-orig` | 1.5G | r2 攻方原样模型的 release 拷贝（跑 310 段用） |
| `r2-clean` | 5.1G | 第二轮冻结副本的新拷贝（debug 编译目录 5.0G，跑 309 条与既有用例） |
| `smoke-batches` | 1.5G | `rerun.sh` 的 `batches` 步冒烟（235 秒时限截断） |
| `smoke-tests` | 5.4G | `rerun.sh` 的 `tests` 步冒烟 |
| `smoke-numbers` | 28K | `rerun.sh` 的 `numbers` 步冒烟 |

合计约 28G；删后草稿目录剩 356K（`du -sh`）：各步日志 `runs/*.log` 与 `runs/batch-*.out`、`staged/`（四份复制品脚本）、`build/`、`scratch/`、开工核对与 cite-check 的完整输出、`ps-before.txt`。没有留下任何仓拷贝或编译目录。第一、二、三轮冻结副本和主 agent 上两轮复跑留下的目录都没动过。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m4-segment-code-r3-sonnet-output.md` 末行（默认根，即仓根；代码引文都写了冻结副本的绝对路径）：

```
  ✓ 核了 39 处引文、47 处裸行号，对上 39 处，没判 47 处（其中裸行号没有引文可比 47 处；逐处列在上面）
```

没判的 47 处全是裸行号（脚本只核行号在不在文件里）。我对其中指向产物、判决、报告、kb 与代码的每一处，都另用 `sed -n <行>p` 打出那一行的开头，核过内容与我的说法对得上，才写进来。
