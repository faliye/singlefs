# m2-supp3-item2 阶段同步：事实表（段一，回扫员 sweep）

阶段名：`m2-supp3-item2`。基准提交 `cc3e8ba`，结束在工作区（当前 `HEAD` 是 `95fd301`，比基准多两次已提交的提交：`c0b9c3b`「决策瘦身 28 条……」、`95fd301`「立 C420」，
两者都不属于这一阶段；工作区里另混着别的会话未提交的改动——`.claude/rules-rationale/`、`CLAUDE.md`、`.claude/agents/{experiment-runner,kb-scribe}.md`、
`.claude/rules/{implementation-first,implementation-workflow,mutation-sampling,path-moves,three-way-inference}.md`、`.claude/gate.d/55-qemu-first-transaction.sh`、
两份 `records/2026-09-13-*.md`、`records/2026-09-20-rules只写怎么做.md`、`research/e7-index-bench/src/bin/*.rs`、`research/scripts/e125-zoned-wp-probe.sh`、
`research/scripts/rules-rationale-audit.py`——按派发提示要求一律没碰。

## 方法：工作区混着两段已提交的无关历史和另一个会话的未提交改动，用「仓副本 + 只叠触发文件」隔离

直接对真实工作区跑 `--changes --base cc3e8ba` 会把 `c0b9c3b`（决策瘦身，touch 了 `.claude/kb/checks-owed.md` 223 行、新增 6 条与 C368/C415 无关的
「## 历史版本」标题；`.claude/kb/milestone/02-second-txn.md` 里另外 3 处 D16 编号重排）与另一个会话的 kb/rules 改动一起算成这一阶段的变更，
把毫不相干的 H 编号塞进事实表。做法与 `m2-closeout-tooling-sync-facts.md` 相同：`git clone --local` 到
`/tmp/claude-1000/m2s3i2-sweep/scoped-repo`，`git checkout cc3e8ba`，只把派发提示点名的触发文件当前内容原样叠上去（`git status --porcelain`
核对结果与触发文件清单逐一对应，见下）；`crates/`、`.claude/gate.d/74-model-differential.sh` 及其 fixtures 在 base..HEAD 之间没有任何提交改动过
（`git diff --stat cc3e8ba HEAD -- crates/ .claude/gate.d/74-model-differential.sh` 为空），可以直接叠当前工作区内容。

`.claude/kb/checks-owed.md` 与 `.claude/kb/milestone/02-second-txn.md` 两个文件被 `c0b9c3b` 的决策瘦身与这一阶段的改动共同触碰，不能整份覆盖：

- `checks-owed.md`：只手工拼出「C368 那一行整行换成 HEAD 的新版本 + 紧接着插入 HEAD 的 C415 整行」，其余 925 行原样保留 cc3e8ba 的内容
  （`diff` 核对：与 cc3e8ba 相比只多这两处，见 `/tmp/claude-1000/m2s3i2-sweep/checks-owed.base.md` 与 `checks-owed.scoped.md`）。
- `milestone/02-second-txn.md`：只把第 166 行（`mount_rollback` 现状段落里那一句）换成 HEAD 的版本，其余 600 行原样保留 cc3e8ba 的内容；
  HEAD 相对 cc3e8ba 在这个文件里还有 5 处别的改动（D16 编号重排 3 处、增补 3 现状段落的 r1 进度叙述 2 处），均未被这一阶段的派发列为
  「做成的事」，不叠。

叠完之后 `git status --porcelain`（scoped-repo）与派发提示的触发文件清单逐一对应（14 个 `M`/`??` 条目，见下）。对这份副本跑
`--changes --base cc3e8ba`：0 条 H（两个 kb 文件里我保留的两处改动都不落在「## 历史版本」小节里，是现状段落内的整句替换/插入，
`changed_segments()` 对纯插入（`insert` 操作码）不产出片段，纯换行的 C368 那一处也是插入形态，因此「变更清单」的两节都是空的
——见 `/tmp/claude-1000/m2s3i2-sweep/changes.md`），事实表 10 行的「出处」因此全部写「派发提示「这一阶段做成的事N」」，不引 H 编号。

事实表与候选表都基于这份副本产出：`--facts` 搜的「现状载体」语料库因此排除了另一个会话尚未提交的改法与 `c0b9c3b`/`95fd301` 两个提交的
无关改动，候选表只罩这一阶段触发文件对应的事实。

scoped-repo `git status --porcelain`（与触发文件清单核对）：

```
?? .claude/gate.d/74-model-differential.sh
?? .claude/gate.d/fixtures/74-model-differential.sh/
?? crates/singlefs-harness/src/model_comparison.rs
?? crates/singlefs-harness/src/model.rs
 M .claude/kb/checks-owed.md
 M .claude/kb/milestone/02-second-txn.md
 M crates/mutations.tsv
 M crates/singlefs-checker/src/walk.rs
 M crates/singlefs-core/src/allocator.rs
 M crates/singlefs-core/src/mount.rs
 M crates/singlefs-core/src/transaction.rs
 M crates/singlefs-harness/src/history.rs
 M crates/singlefs-harness/src/lib.rs
 M crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs
 M crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
 M crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
 M crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs
 M crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs
```

`research/prompts/` 下这一轮的三方与验证证据（`_m2-supp3-item2-code-r1-*`、`-r2-*`、`m2-supp3-item2-*`）不是 `CARRIER_PATTERNS` 里的
现状载体（`.claude/kb/`、`.claude/rules/`、`records/` 等才是），不影响 `--changes`/`--facts` 的语料库，未叠入副本；出处引用它们时按
「派发提示」间接指过去，事实表本身不直接读它们。

## 事实表（10 行，`--check-facts` 已通过）

原文件：`/tmp/claude-1000/m2s3i2-sweep/facts.tsv`（制表符分隔，下方原样抄一份）。

```tsv
编号	旧事实	新事实	检索词	出处
F1	「与理想模型对拍」没有：`.claude/singlefs-ai-sop/scripts/gate.sh` 的未实现清单里「模型对拍」一直挂着，`.claude/gate.d/` 里没有任何阶段声明 `# gate-covers: 模型对拍`，层 0 的多版本 oracle 只答「这个崩溃状态恢复到的版本在不在允许集合里」	理想模型 `crates/singlefs-harness/src/model.rs`（只用 `singlefs_format` 的常量，D13（验证路线） 已定项 5）与模型对拍胶水 `model_comparison.rs` 接好；随机历史每一步拿实现的结局与只住内存的模型比	模型对拍	派发提示「这一阶段做成的事①」
F2	增补 3 第 2 件（模型对拍）在这一阶段开工前还没有随机历史的取样点	随机历史的取样点扩到五段：快档、偏向抬 F 之后复用的取样点、偏向抬 F 之后回退的取样点、逼近分配记录墙的取样点、新增的小盘上逼近单元区墙的取样点（两块单元区 384 槽的小盘，比重 `TOWARD_THE_UNIT_AREA_WALL`，每一步跑池级 checker）	取样点	派发提示「这一阶段做成的事②」
F3	新立：门禁 74 号 `74-model-differential.sh`	门禁 74 号（`# gate-covers: 模型对拍`）：release 下单跑 `second_transaction_supplement_three_random_history` 测试二进制，要求五段都打出「模型对拍 N 步」且步数大于 0；红绿样本按真输出录；`stage-owners.tsv` 登记给 `implementation-writer,crash-verifier`	74-model-differential	派发提示「这一阶段做成的事③」
F4	新立：随机历史「逼近分配记录墙的取样点」每一步之后跑一次池级 checker	第四段「逼近分配记录墙的取样点」最初不跑池级 checker，现在每一步之后都跑一次；已知红第 0 条那一形只记不停	逼近分配记录墙	派发提示「这一阶段做成的事④」
F5	59 号（crates 变异表）跑到 140 条变异全部红在点名的测试上（增补 3 第 1 件收口时的产物）	`crates/mutations.tsv` 新增第 165–178 行（14 条，增补 3 第 2 件第二轮），整表 173 条；59 号换干净编译目录重跑，173 条全红在点名的测试上	条变异全部红在点名的测试上	派发提示「这一阶段做成的事⑤」
F6	新立：增补 3 第 2 件代码三方两轮打中之后的改法	代码三方两轮打中之后的改法接好（判决 `research/prompts/m2-supp3-item2-code-r2-main-verification.md` 第三节 1–5 条）：候选排除「低于 F」不再报成「树表 0 条」等五处遮蔽	代码三方两轮打中之后的改法	派发提示「这一阶段做成的事⑥」
F7	新立：增补 3 第 2 件的四道重验证	四道重验证过：层 0 崩溃点重放全量两条流零违例、QEMU 真设备 18 项、herd7 三条 Never、59 号整表	四道重验证	派发提示「这一阶段做成的事⑦」
F8	新立：C415（变异表复跑共用编译缓存，陈旧产物能把变异记成被抓）	C415：门禁 59 号拷仓跑变异，`shutil.copytree` 保留源文件 mtime，`GATE_MUTATION_TARGET_DIR` 跨轮复用时拷进来的源码比缓存里的产物旧、cargo 判它新、不重编，会把陈旧产物记成变异被抓；判别力自证：造一份缓存产物比拷进来的源码新的局面，不修必须报「没跑到」	C415	派发提示「这一阶段做成的事⑧」
F9	C368（分配器落点只看盘 0，盘不等大时断言失败） 仍欠的写清只有「发布层把三种拒绝统一报成 `NoSpaceFor`，调用方分不出」，没提模型对拍的取样点缺口	C368 补上一段：模型对拍没有不等盘的取样点——增补 3 第 2 件的小盘段两块盘等大，把「每块盘上都没有」报成「小盘写满」、只换原因的那条变异在五段上一段都不红（2026-09-19 实现员实测，代码三方第二轮判决第 44 行第 6 条）；判别力自证新增一句：随机历史另加一个两块盘不等大的取样点，该变异必须红	C368	派发提示「这一阶段做成的事⑧」
F10	`.claude/kb/milestone/02-second-txn.md` 步 4 现状写「目标根不在环里报 `RollbackTargetNotInRing`」	改写成「目标根不在环里报 `RollbackTargetNotACandidate`（排除原因 `NotInRing`）」——源码里那个成员早就改名了，kb 原文这才跟上	RollbackTargetNotInRing	派发提示「这一阶段做成的事⑨」
```

## 命令与输出末行

```
$ cd /tmp/claude-1000/m2s3i2-sweep/scoped-repo
$ python3 research/scripts/stale-candidates.py --check-facts /tmp/claude-1000/m2s3i2-sweep/facts.tsv --base cc3e8ba
  ✓ 事实表罩全了：0 条变更记录都有出处，10 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）
$ python3 research/scripts/stale-candidates.py --facts /tmp/claude-1000/m2s3i2-sweep/facts.tsv --base cc3e8ba --out /tmp/claude-1000/m2s3i2-sweep/candidates.tsv
  ✓ 候选表 /tmp/claude-1000/m2s3i2-sweep/candidates.tsv：184 行（180 个不同的行）
```

退出码都是 0。

## 候选表

单独存在 `research/prompts/m2-supp3-item2-sync-candidates.tsv`（185 行，含表头，184 条候选；六个「新立：」事实——F3、F4、F6、F7、F8——
不参与候选搜索，按工具设计跳过）。逐 fact 候选行数：F1 52、F2 124、F5 1、F9 5、F10 2；F10（`RollbackTargetNotInRing`）的两条命中分别是
`.claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:8` 与 `records/2026-09-17-已分配口径三方与两个实验.md:80`——这两处仍在用被
`crates/singlefs-core/src/mount.rs` 删掉的旧枚举成员名说话（是「那一次发生的事」的引述——`E154 装置的实测记录，实现当时确实报过这个
名字——还是需要更新成现状句，留给逐行判的组去判）。

## 每行检索词（--check-facts / --facts 要求的清单）

| 编号 | 检索词 | 是否「新立」 |
|---|---|---|
| F1 | 模型对拍 | 否 |
| F2 | 取样点 | 否 |
| F3 | 74-model-differential | 是 |
| F4 | 逼近分配记录墙 | 是 |
| F5 | 条变异全部红在点名的测试上 | 否 |
| F6 | 代码三方两轮打中之后的改法 | 是 |
| F7 | 四道重验证 | 是 |
| F8 | C415 | 是 |
| F9 | C368 | 否 |
| F10 | RollbackTargetNotInRing | 否 |

## 没做什么

- 只做了阶段同步的第一段（写事实表）；候选表已产出交给下一段的逐行判，但逐行判本身不归这一轮。
- 没有跑 `git diff --stat cc3e8ba HEAD` 之外，对 `crates/` 之外别的目录逐提交核对——只核了 `crates/`、`.claude/gate.d/74-model-differential.sh`
  与两个 kb 文件三处的边界，够定出 14 个触发文件与两处手工拼接，没有再往深核 `c0b9c3b`/`95fd301` 两个提交里其余改动的细节（与这一阶段无关，
  按派发提示不用管）。
- 没有碰、没有读派发提示点名要求不碰的别的会话的改动内容（只在 `git status`/`git diff --stat` 层面确认了它们的路径不在触发文件清单内）。
- 候选表的语料库基于 scoped-repo（其余文件冻结在 `cc3e8ba`），如果另一个会话的改动或 `c0b9c3b`/`95fd301` 两个提交里恰好也改了同一批
  现状载体里、与本阶段检索词撞词的句子，这份候选表看不到——它只保真本阶段触发文件对应的事实，这是有意的隔离，不是遗漏。
