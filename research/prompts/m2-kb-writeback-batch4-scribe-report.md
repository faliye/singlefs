# kb 写回执行报告（第四批，甲/乙两份出口）

## 规格文件

- 甲：`/tmp/claude-1000/kb-batch4-drafter/甲/spec.json`（8 条）
- 乙：`/tmp/claude-1000/kb-batch4-drafter/乙/spec.json`（7 条）
- 起草员报告：`research/prompts/m2-kb-writeback-batch4-drafter-report.md`（sha256 `372050a08c8bec91418e966f0ed5cdd82f81bdd52f429c4bf30a5d6285fc8483`，已核对一致）

## 开工前 sha256（施加规格之前）

```
0b843b2b39e9bec49271213e63f7624544ba6a0773bd6fc68e074dceeae9e80c  .claude/kb/decisions/28-挂载期承诺量.md
7524ce0703b91c1e876aa1456e5c06d9cad26a5c4853f18527c0877c2460811f  .claude/kb/decisions-history/2026-09.md
2f96e8dd81c25d4ef0db5ab02cb9a499f0a68af2d7d876a62caee309123b3dd8  .claude/kb/checks-owed.md
80782e1c5da5004ccc63de7beb73c433d7780662783f22900f96884ebe055e04  .claude/kb/decisions/23-journal的角色与格式.md
070d60ad0a2f3b92f07c41cc0c7a766b1627e003649ad15d1f25eb88ebbfdb16  .claude/kb/decisions/16-发布语义.md
0c8f3fb88b14ded45d2475cebaa07ec0f4fcca05561251c2527aa88b28362f6b  .claude/kb/decisions/18-块里携带什么信息.md
0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895  .claude/kb/decisions.md
10d1b2a3d013471560f7f5972af90fa43c229f557341c6249dc2417055e86e9d  .claude/kb/decisions-history.md
```

开工时这几个文件已带别的会话未提交的改动（`git status --porcelain` 里已显示 `M`），按定义照记，不是这一轮造成的。

## 写范围闸预检（施加前，逐文件）

对甲/乙两份规格里出现的全部 6 个文件路径分别喂 `write-guard.sh` 预检（`AGENT_HOOK_DETECTIONS=/tmp/claude-1000/kb-batch4-scribe/precheck.jsonl`），6 个全部 `exit=0`：

```
.claude/kb/decisions/28-挂载期承诺量.md            exit=0
.claude/kb/decisions-history/2026-09.md            exit=0
.claude/kb/checks-owed.md                          exit=0
.claude/kb/decisions/23-journal的角色与格式.md      exit=0
.claude/kb/decisions/16-发布语义.md                 exit=0
.claude/kb/decisions/18-块里携带什么信息.md         exit=0
```

一条都没被拒，8+7 条规格全部照写。

## 施加次序与 kb-spec-check.py

按规格给定次序：先甲、核过、落地；再对落地后的仓核乙、转绿；再落乙。

```
$ python3 research/scripts/replace-batch.py --dry-run /tmp/claude-1000/kb-batch4-drafter/甲/spec.json
✓ 8 处都命中得对（4 个文件），--dry-run 没写
$ python3 research/scripts/replace-batch.py /tmp/claude-1000/kb-batch4-drafter/甲/spec.json
✓ 8 处替换，写了 4 个文件（改名换上新 inode），回读一致
$ python3 research/scripts/kb-spec-check.py /tmp/claude-1000/kb-batch4-drafter/乙/spec.json
  ✓ 规格 7 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 7 条）
$ python3 research/scripts/replace-batch.py --dry-run /tmp/claude-1000/kb-batch4-drafter/乙/spec.json
✓ 7 处都命中得对（5 个文件），--dry-run 没写
$ python3 research/scripts/replace-batch.py /tmp/claude-1000/kb-batch4-drafter/乙/spec.json
✓ 7 处替换，写了 5 个文件（改名换上新 inode），回读一致
```

甲落地后乙的 `kb-spec-check.py` 转绿，与起草员报告预告的一致（乙第 5、6 条锚在甲第 3、6 条新写的文本上，甲落地之前会报「出现 0 次」，落地之后就命中）。

## 主 agent 已判的五点（照给定判断落地，不重判）

1. 第 1 条（E158（择根与修复四岔路） 实验页「影响的决策」那一对）：这一批不补，等写 E158 实验页的执行员补——规格里已按此写法（依据先指两份 runner 报告，正文与欠账行都写「门禁 75 号那一对等实验页写好再补」）。
2. 第 2 条出处：两份规格里已经是 `research/prompts/e158-r3-prereg.md` 第 348 行与 `research/prompts/e158-r4-prereg.md` 第 413 行（乙第 1 条 basis 与正文原样如此），未再改动。
3. 第 3 条：C554 乙-配置续的判据段照规格插在 D23（journal 的角色与格式） 已定项 14 里（甲第 8 条），没有单独立一条已定项。
4. 第 4 条：Q2 没有新立欠账（乙没有对应 C57x），只在 checks-owed.md 的 C554 行正文里点了一句「模型跟上乙在做，记欠见 Q2」，与规格一致。
5. 第 5 条（欠账号顺延）：写第一行之前现查 `.claude/kb/checks-owed.md` 当前最大号：

```
$ grep -oE '^\| C[0-9]+' .claude/kb/checks-owed.md | grep -oE '[0-9]+' | sort -n | tail -5
568
569
570
571
572
```

最大号是 C572，与规格假设一致（C573–C580 顺号可用），**不需要改号**，无对照表。

## 决策变更史（49 号）

甲写「其七」，乙在其七之前插「其八」「其九」，标题里的「（其N）」在写之前照当月文件现取，与规格给定一致（未改）。

跑 `bash .claude/gate.d/49-history-brief.sh`（不带 `--write`）两次（甲落地后一次、乙落地后一次），均只报「生成块与原文不一致」（因还没重生成），未发现「（待补）」占位或不属于这一轮的条目；确认安全后各跑一次 `--write`：

```
甲落地后：✓ 按原文重新生成了 .claude/kb/decisions-history.md：566 条条目，这次补了 0 条「（待补）」快查
乙落地后：✓ 按原文重新生成了 .claude/kb/decisions-history.md：568 条条目，这次补了 0 条「（待补）」快查
```

两次 `--write` 前后各对 `git diff -- .claude/kb/decisions-history.md` 核对新增行：甲那次唯一新增的表行是 D28（挂载期承诺量） 已定项 4 那条卡片（「共改过 23 次」→「24 次」）；乙那次新增两行，分别是 D23（journal 的角色与格式）/D16（发布语义） 合记的「其八」与 D18（块里携带什么信息） 的「其九」（对应卡片各「+1 次」）。被挤出「只留最近 3 次」窗口的行是设计内的旧行，不是新增；两次新增里都没有不属于这一批的条目。

## kb 门禁（doc-lint、20、21、30、43、49、75、78）原样末行

```
$ bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .（末行）
  ✗ 文档铁律检查失败：1 个文件违规、0 处编号引用无定义、1 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 521，跳过 0）

$ bash .claude/gate.d/20-kb-shape.sh（末行）
  ✓ kb 形状检查通过（kb 文件 209 份、规则 7 份）

$ bash .claude/gate.d/21-decision-items-sync.sh（末行）
  ✓ 决策分项清单与正文同步（350 个分项）

$ bash .claude/gate.d/30-decision-history.sh（末行）
  ✓ 决策正文改了 30 行，变更史新增 7 条条目

$ bash .claude/gate.d/43-owed-table-shape.sh（末行）
  ✓ 欠账表两张登记表的行形状都对，欠着的那张里没有写着已还的行（检查了 560 行，2 张表）

$ bash .claude/gate.d/49-history-brief.sh（末行，乙落地并 --write 之后再跑一次）
  ✓ 决策变更史的快查与原文同步：查了 568 条条目、1196 格快查，decisions-history.md 与原文一致

$ bash .claude/gate.d/75-decision-experiment-links.sh（末行，退出码 1）
      [不对称] .claude/kb/decisions/23-journal的角色与格式.md：D23 已定项 14 的依据引了 E158，.claude/kb/experiments/158-择根与修复四岔路.md 的影响的决策表里没有「支撑 / 推翻」这一行

$ bash .claude/gate.d/78-owed-cited-tests.sh（末行）
  ✓ 已还清的行引的测试名都还在（83 行，查了 45 个标识符：……）
```

**75 号红**：正是主 agent 已判「这一批不补」的 E158（择根与修复四岔路） 那一对，照原样报，没有去补实验页的「影响的决策」表。

**doc-lint 红（这一轮写出的句子，按定义停下不自改）**：

1. `.claude/kb/checks-owed.md:490`（本轮新插的 C577 行，规格 乙第 6 条原文）：句子「实现落地后按上面三条用例断言不再判……红」里的「上面三条」命中 kb 正文位置指代的判据（数量词紧跟在「上面」之后）。
2. `.claude/kb/decisions-history/2026-09.md`（本轮新写的「其七」条目，规格 甲第 3 条原文）第 35 行：「另记成 C573、C574、C575、C576 四笔新欠账」里 C573、C574、C575 三处是裸编号引用（没带简称），只有末尾的 C576 后面另有信息但同样没写成「编号（简称）」形态。

两处文本都是规格给定的原文，我逐字照写，没有加字也没有自己改句子；按定义「红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子」，这两处交主 agent 判要不要改规格重写。

## 阶段归属表登记给 kb-scribe 的阶段（共 41 个，跑毕并贴结果）

```
$ awk -F'\t' -v me=kb-scribe '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv | wc -l
41
```

逐个 `nice -n 19 bash .claude/gate.d/<文件>`，退出码汇总：34 绿（0）、2 本次未跑（77：`35-user-verdict-owed.sh`、`36-invariant-count-cross-file.sh`；1 本次未跑：`61-settled-same-file.sh`）、7 红：`10-kb-rot.sh`、`22-item-ref-status.sh`、`27-format-constants.sh`、`33-mutation-tables.sh`、`52-segment-registry.sh`、`69-evidence-in-repo.sh`、`75-decision-experiment-links.sh`。

**归属判断（逐条看点名的文件/对象在不在这一轮改动里）**：

| 阶段 | 点名的对象 | 在不在这一批规格里 | 判断 |
|---|---|---|---|
| 10-kb-rot.sh | `E161 被引用但 experiments/ 下没有它`（另有两处「没判」是 `.claude/rules/format-evolution.md`、`.claude/skills/decide/SKILL.md` 里的占位路径说明，不是缺陷） | 我这一批没碰 E161、没碰这两个文件 | 不是这一轮的，不修 |
| 22-item-ref-status.sh | `crates/singlefs-harness/tests/a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side.rs:5` 引 `.claude/kb/decisions/13-验证路线.md` 已定项 4/7 | D13 不在我这一批规格里；`git status` 显示该决策文件已有别的会话未提交改动 | 不是这一轮的，不修 |
| 27-format-constants.sh | `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`、`e157_parallel_line_one_clauses.rs` | 都不是我这一批写的文件 | 不是这一轮的，不修 |
| 33-mutation-tables.sh | `research/mutations/e162_crash_verdict_block_store.tsv`、`crates/mutations.tsv:659`（E158 root_choice_repair） | 不在我这一批规格里 | 不是这一轮的，不修 |
| 52-segment-registry.sh | `.claude/kb/layout/01-first-txn.md`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` | 都不是我这一批写的文件 | 不是这一轮的，不修 |
| 69-evidence-in-repo.sh | `research/prompts/m2-kb-writeback-batch2-scribe-report.md`、`…batch3-scribe-report.md` 里引用 `/tmp` 路径 | 是更早批次（batch2、batch3）的报告，不是这一轮 | 不是这一轮的，不修 |
| 75-decision-experiment-links.sh | D23（journal 的角色与格式） 已定项 14 依据引 E158（择根与修复四岔路） | **是**这一批写出的引用（乙第 1 条）；主 agent 已判「这一批不补，等 E158 实验页的执行员补」 | 是这一轮的，但按主 agent 判断不在这一批的写范围要求内，照报不修 |

除 75 号（已由主 agent 判定这一批不处理）之外，7 处红都不是这一轮改动引入的，未做任何修改。

## 改过的全部文件（开工前 → 收尾时 sha256）

| 文件 | 开工前 | 收尾时 |
|---|---|---|
| `.claude/kb/decisions/28-挂载期承诺量.md` | `0b843b2b39e9bec49271213e63f7624544ba6a0773bd6fc68e074dceeae9e80c` | `5b2f486eded720b1121e04814f451cd28774ab600e685a1b979045ab08cbe062` |
| `.claude/kb/decisions-history/2026-09.md` | `7524ce0703b91c1e876aa1456e5c06d9cad26a5c4853f18527c0877c2460811f` | `5f46ba55c82615cfd6c6069814a240de0e8b2ac93bbbf9a220dca26c19807f1a` |
| `.claude/kb/checks-owed.md` | `2f96e8dd81c25d4ef0db5ab02cb9a499f0a68af2d7d876a62caee309123b3dd8` | `e3d26ebdb7ccd3b0098f75139e478683cec1dbd5916f31fc2d0e7102c586dbd5` |
| `.claude/kb/decisions/23-journal的角色与格式.md` | `80782e1c5da5004ccc63de7beb73c433d7780662783f22900f96884ebe055e04` | `10988a041e8f6b63b0dfc3b43630a7bd27fd37450d30ae422ded6ca12290f1b2` |
| `.claude/kb/decisions/16-发布语义.md` | `070d60ad0a2f3b92f07c41cc0c7a766b1627e003649ad15d1f25eb88ebbfdb16` | `2344da7ebb42db316a89a01cd885f3b95b2b79cbe3201ac9391cdb12cdab709f` |
| `.claude/kb/decisions/18-块里携带什么信息.md` | `0c8f3fb88b14ded45d2475cebaa07ec0f4fcca05561251c2527aa88b28362f6b` | `2ef1055cb9f1ad2bd813777e890254d4dc3bed48ab4d94b3f0aae37b3b329027` |
| `.claude/kb/decisions-history.md`（由 49 号 `--write` 重生成，两次） | `10d1b2a3d013471560f7f5972af90fa43c229f557341c6249dc2417055e86e9d` | `0a18bc4144686ef8d696a471a0a89cb60b66a385966c5a897a536026b5b2adce` |
| `.claude/kb/decisions.md` | `0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895` | `0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895`（未改，只记录用） |

## 没做什么（固定会有的）

- 没判用户定案对不对，没判「乙-配置续」这个改法本身站不站得住，规格没写的句子一个字没加。
- 没跑重型测试（`cargo test`、层 0、QEMU 等）。
- 没提交（git add / commit）。
- 分项翻状态：这一批规格里没有任何一处是分项从「未定」翻「已定」（或反向），没有调用 `research/scripts/relabel-item.py`，也没有因此需要单独跑一次 `bash .claude/gate.d/21-decision-items-sync.sh --write`（该阶段在阶段归属表里正常跑过一次，是绿的）。
- doc-lint 报的两处红（`.claude/kb/checks-owed.md:490` 的「上面三条」、`.claude/kb/decisions-history/2026-09.md` 「其七」条目里 C573/C574/C575 裸引用）是这一轮写出的句子，按定义停下交主 agent 改规格，没有自己改句子。
- 75 号红（E158（择根与修复四岔路） 那一对不对称）是主 agent 已经判过「这一批不补」的缺口，照原样报，没有去补 `.claude/kb/experiments/158-择根与修复四岔路.md` 的「影响的决策」表。
- 阶段归属表里另外 6 处红（10、22、27、33、52、69 号）点名的文件都不在这一批规格里，判定为别的会话在飞的改动，没有修改。
