# 子 agent 落地审计：「看起来没产生效果」那几桶的 40 个抽样，逐个判定（2026-10-22）

批次：records/2026-10-22-子agent结果落地审计.md#4
输入：`/home/fy5090/.local/share/singlefs/work/subagent-landing-audit-2026-10-22/inputs/4-no-effect.json`（40 个）；花费取同目录 `all-agents-classified.json` 的 `cost` 字段。
草稿：逐条笔记 `notes.md`（sha256 e2dc0c75a72f93cccaf013e0d11167cb6c9d6f5a04a1e9be46cc6d3f16887969）；辅助脚本在 `tools/` 下：
- `tl.py`：主会话时间线；
- `sub.py`：子 agent 最后几步；
- `succ.py`：找后继者；
- `verdicts.py`：汇总。

写法说明：第一次用 Write 写这份报告被工具拒了，原话是「Subagents should return findings as text, not write report files」。随后交回钩子 handback-guard.sh 要求「全文写进报告文件」，所以改用共用约束里给没有 Write 的 agent 的写法：`set -o noclobber` 后用 heredoc 分段写。

## 结论

40 个里：
- 27 个其实产生了效果，是机械层漏判；
- 10 个被后继者重做；
- 2 个是主 agent 有意弃用；
- 1 个丢了；
- 判不了的 0 个。

按折算花费，「其实产生了效果」占这 40 个的 84.9%，「被重做」占 14.6%，后两类合计不到 0.5%。

交回了东西的四桶（report-unreferenced、report-used-by-later-sub、written-overwritten、written-unused）27 个全部其实产生了效果。没交回的两桶（stopped-abandoned、cut-by-usage-limit）13 个里，10 个被重做、2 个有意弃用、1 个丢了。

外推到 240 个（推的，按桶分层）：约 83% 的花费其实产生了效果，约 16% 是被重做，弃用加丢了约 1%。换算到全部 2230 个的折算，这几桶原来算 6.81%，真正没产生效果的（重做、弃用、丢了）约 1.1%。重做的那部分里，前任的半成品大半被接手者用上了。

推翻条件：
- 再抽一批，如果 report-unreferenced 桶里「其实产生了效果」不到一半，或者 written-overwritten 桶里出现交回后主 agent 一步没核、也没合入的，上面的外推就不成立。
- 第 11 个：如果别处已经立了「登记里的正式跑在单测单元表与 replay.sh 都找得到」这一格，它要从「丢了」改判成「被重做」。我只查了 doc-experiments.sh 的 12 个 gate-cell、checker-tier-research-build-and-replay.sh 的 4 个 gate-cell，以及 `grep -rln -E '单元表|正式跑' .claude/gate.d/*.sh scripts/`，都没有这一格。

## 汇总

以下是 `python3 -I tools/verdicts.py` 的原样数，样本折算合计 91.34 百万。

| 判定 | 个数 | 折算（百万） | 占 40 个的花费 |
|---|---|---|---|
| 其实产生了效果 | 27 | 77.53 | 84.9% |
| 被后继者重做 | 10 | 13.37 | 14.6% |
| 主 agent 有意弃用 | 2 | 0.41 | 0.4% |
| 丢了 | 1 | 0.03 | 0.0% |
| 判不了 | 0 | 0 | 0 |

桶与判定交叉：

| 桶 | 判定 | 个数 |
|---|---|---|
| report-unreferenced | 其实产生了效果 | 19 |
| report-used-by-later-sub | 其实产生了效果 | 1 |
| written-overwritten | 其实产生了效果 | 6 |
| written-unused | 其实产生了效果 | 1 |
| cut-by-usage-limit | 被后继者重做 | 5 |
| stopped-abandoned | 被后继者重做 | 5 |
| stopped-abandoned | 主 agent 有意弃用 | 2 |
| stopped-abandoned | 丢了 | 1 |

**外推（推的）。** 做法是：某判定在该桶样本花费里的占比，乘该桶的总体花费；个数同理。总体是 240 个、505.97 百万，占全部 2230 个（7430.75 百万）的 6.81%。

| 判定 | 外推个数 | 外推折算（百万） | 占 240 个 | 占全部折算 |
|---|---|---|---|---|
| 其实产生了效果 | 约 162 | 约 421.9 | 83% | 5.68% |
| 被后继者重做 | 约 62 | 约 81.0 | 16% | 1.09% |
| 主 agent 有意弃用 | 约 10 | 约 2.8 | 1% | 0.04% |
| 丢了 | 约 5 | 约 0.2 | 0% | 0.00% |

外推的限制：各桶抽到的样本数不齐：

| 桶 | 抽到 / 总数 |
|---|---|
| report-unreferenced | 19/101 |
| written-overwritten | 6/40 |
| cut-by-usage-limit | 5/36 |
| stopped-abandoned | 8/42 |
| report-used-by-later-sub | 1/17 |
| written-unused | 1/4 |

最后两桶各只有 1 个样本，外推到这两桶的数只能看个量级。

## 逐条判定表

花费单位是百万（折算）；时刻取自主会话 jsonl 的时间戳。依据见下一节。

| # | agent | 桶 | 花费 | 判定 |
|---|---|---|---|---|
| 0 | a950f6e8fb02144ff | stopped-abandoned | 0.35 | 主 agent 有意弃用 |
| 1 | ae490c0c0ff890bd7 | stopped-abandoned | 0.23 | 被后继者重做 |
| 2 | ae430399da15b7a05 | report-unreferenced | 0.67 | 其实产生了效果 |
| 3 | a5002dbe54e9c1153 | report-unreferenced | 8.44 | 其实产生了效果 |
| 4 | ae90c3bc3015d8c1b | report-unreferenced | 1.10 | 其实产生了效果 |
| 5 | a7e17b11bc907b4b8 | report-unreferenced | 2.48 | 其实产生了效果 |
| 6 | a7ac87df74763888c | report-unreferenced | 2.80 | 其实产生了效果 |
| 7 | a58c6818c7130a2a8 | stopped-abandoned | 0.05 | 被后继者重做 |
| 8 | a319676002ef8a7f4 | written-unused | 6.40 | 其实产生了效果 |
| 9 | a4098c52b80224334 | cut-by-usage-limit | 1.18 | 被后继者重做 |
| 10 | a08c8603129598c6c | stopped-abandoned | 0.04 | 被后继者重做 |
| 11 | a72995f39b43a7098 | stopped-abandoned | 0.03 | 丢了 |
| 12 | a8f1c481095d9c421 | report-unreferenced | 0.74 | 其实产生了效果 |
| 13 | a6c0573382086ed76 | stopped-abandoned | 0.06 | 主 agent 有意弃用 |
| 14 | a193efc2c02e1be13 | cut-by-usage-limit | 0.73 | 被后继者重做 |
| 15 | a40113fd9d20d3f2f | report-unreferenced | 0.82 | 其实产生了效果 |
| 16 | a1fde38dd319b793e | written-overwritten | 4.15 | 其实产生了效果 |
| 17 | a7551ab925195552a | report-unreferenced | 0.73 | 其实产生了效果 |
| 18 | a35b9deaf54a80347 | report-unreferenced | 0.16 | 其实产生了效果 |
| 19 | aa00cd04d77659bdc | report-unreferenced | 1.95 | 其实产生了效果 |
| 20 | ab24d506152c12130 | report-unreferenced | 1.61 | 其实产生了效果 |
| 21 | a09d2bd77e868ff10 | written-overwritten | 3.04 | 其实产生了效果 |
| 22 | aaca396545ea5e03f | report-unreferenced | 4.26 | 其实产生了效果 |
| 23 | ad1d6d6c1f71c9039 | report-unreferenced | 0.96 | 其实产生了效果 |
| 24 | aacf5334684d7e101 | written-overwritten | 1.81 | 其实产生了效果 |
| 25 | adb95b74ec6d6e13f | written-overwritten | 0.15 | 其实产生了效果 |
| 26 | ae86ff3db80c14cc0 | report-unreferenced | 1.57 | 其实产生了效果 |
| 27 | a1e3d5eb602c8d239 | stopped-abandoned | 0.82 | 被后继者重做 |
| 28 | ae8522b4752eaf275 | report-used-by-later-sub | 2.83 | 其实产生了效果 |
| 29 | a42951f16d8842a19 | report-unreferenced | 0.48 | 其实产生了效果 |
| 30 | a3c5d1c6719964344 | report-unreferenced | 0.34 | 其实产生了效果 |
| 31 | addd61bc5af1a41f7 | stopped-abandoned | 0.91 | 被后继者重做 |
| 32 | a6d19229fd9229c82 | report-unreferenced | 0.16 | 其实产生了效果 |
| 33 | a7f035b5f37a55a32 | cut-by-usage-limit | 6.47 | 被后继者重做 |
| 34 | ab15fce1b4de481fc | report-unreferenced | 0.89 | 其实产生了效果 |
| 35 | ab85b77d63e4ab98e | written-overwritten | 19.07 | 其实产生了效果 |
| 36 | ab6a6ff713da2d1cf | written-overwritten | 8.81 | 其实产生了效果 |
| 37 | a3281c2f3764c6e43 | cut-by-usage-limit | 0.82 | 被后继者重做 |
| 38 | a57f0b0e08d99dcbf | cut-by-usage-limit | 2.12 | 被后继者重做 |
| 39 | afe99ce4c3ae0e40a | report-unreferenced | 1.10 | 其实产生了效果 |

## 依据：主 agent 有意弃用（2）

**0**（kb 三语门禁）：用户 14:39:07 停了它，接着说「门禁加上就好了 其他的你不管了」。主 agent 14:39:34 回「好，只把门禁加上，滞后表、别的门禁跳译文、上游 doc-lint 那几样都不做了」，随后自己改了 lib-trilingual.py 和 .claude/trilingual-documents。被停之前它调了 50 次工具，全是读。

**13**（Survey judge steps still on CPU）：用户 10:26:00 说「你这个观点是错的 我们所有的流程 cpu和gpu是分离的。 cpu跑 录入数据 gpu跑对比运算数据。 严格来说开启了gpu的时候，所有的对比都需要用gpu对比。」主 agent 10:26:25 回「明白了，我前面两次都没说对」，然后 TaskStop（它只跑了约 1 分钟）。之后按新口径又派了两次：a5c752dcf2500c12f 又被用户纠正后停；a391e1ba4d86466c5「Survey run/compare split via RocksDB」最后用上了，桶是 report-used-by-main。

## 依据：丢了（1）

**11**（乙：对照逐格对得上的门禁格）：04:25:51 和 10 号一起被停，停之前只调了 3 次工具。停的原因是另一个会话 singlefs-3b 正在把 scripts/ 搬进子目录。

它要做的是：给 doc-experiments 加一格，判「登记里的正式跑在单测单元表与 replay.sh 都找得到」，对应待优化表第 3、4 行。

为什么判丢了：
- 收尾表把它记成下轮做。`records/2026-10-11-里程碑三待优化上下游协调.md` 第 104 行一带原文是「| 3、4 | 上游关；下游门禁格下轮开 | doc-experiments 加格等门禁搬进 `scripts/` |」。
- 之后没人派。今天的 doc-experiments.sh 共 12 个 gate-cell（第 15–26 行），里面没有这一格。
- `.claude/kb/milestone/03-third-txn.md` 第 185 行的现状列至今写着「跨实验的检查没立」。

丢的是：跨实验比对正式跑、单测单元表、replay.sh 复跑行三处的检查。

## 依据：被后继者重做（10）

**1**：另一个会话（8adc3bc8）11:12:40 清盘，`rm -rf` 删了工作根下 506 项，冻结副本也在里面。主 agent 11:14:11 TaskStop 了三条腿，重建输入以后 11:17:18 说「Inputs are rebuilt and logged. Redispatching…」，11:17:37 派 a2429f91b33384514「准入条款簇三方攻方腿（重派）」，桶是 verdict-archived。前任只读了 1.5 分钟，白做了，但量很小。

**7**：派出 25 秒后被 TaskStop。主 agent 12:28:30 说「先带着边界证据重派调查员」，12:28:48 派 a9eb9cedfce23b6b1「Investigate GPU too-many-records after E6」（report-used-by-main，结论是 MAX_RECORDS=512）。白做约等于 0。

**9**：01:36:31 撞了周限额。16:03:26 主 agent 说「腿报告都没落盘，只有模型目录。照规矩用交接摘要新派同立场的腿接着做」，16:07:36 派 ade3e757be6052695「b24 code round forward leg (resume)」（verdict-archived）。提示里写着「第一步先核现场、已落盘的不重做」，前任落盘的 sonnet-model 目录被沿用了。

**10**：被停的原因同 11。06:15:47 的记录写着「照新路径重派甲…丙一（只做第 11 行）…乙与丙的 lkmm 那一半等门禁搬完」。后续分成两件：
- 第 11 行由 a4d08372532bdb658「丙一：新命令按预测记账」做了，桶是 report-used-by-main。
- lkmm 那一半由另一个会话的 a53c20c1ec7d3d9f6「工具实现员：QEMU LKMM Verus 单元键」做了（10-14，written-landed）。`milestone/03-third-txn.md` 第 188 行的现状列写着「2026-10-15 批4 起七道重型都按判定的单元键入库，LKMM 每份 litmus 一条判定」。

前任只调了 4 次工具，白做约等于 0。

**14**：17:07:57 撞了会话限额。用户 17:30:36 说「限制恢复了 开始吧」。17:30:58 主 agent 说「四件各新派一个接手，派发提示里把原来的输入照抄，另指到交接摘要与已落的现场」，17:32:28 派 aac7c2e7f8bb98dca「7-2b 接手：M3 在核上证」（written-landed）。前任写的 1200 行里有 1189 行进了 git（机械层自己数的），半成品基本全被用上。

**27**：用户 09:59:17 说「比较糟糕的一件事 我关闭了客户端 看看任务还在吗」，同时收到通知「4 background agents didn't finish before the previous session ended」。09:59:36 主 agent 说「四件活都停了…补丁一份也没交出来…换新草稿目录重派」，10:00:03 派 ae3e53f5c35f0d660「批 12 变异三行修红（重派）」（written-landed）。提示里写「只有认领文件，没有可接的产物，从头做」。前任调了 90 次工具，花了 0.82，全部白做。

**31**：和 27 是同一次客户端关闭。10:00:23 派 aa4553b92cde40eed「批 12 工具批五条（重派）」（written-partial）。提示里写前任的「plan.tsv、pairs.tsv、apply-numbers.py 可以读来参考（照下面第 2 条的验收现核，不照抄）」，前任的东西部分被用上。

**33**：14:37:22 撞了周限额。14:41:34 主 agent 说「五个子 agent 全部撞了周限额…逐个抽交接摘要，新派同类 agent 接着做」，然后派 a6b123c951891791c「实验执行：E181 第一段（接续）」（written-landed）。提示指到 handover-a7f035b5f37a55a32.md 和前任的草稿目录，原话是「编好的 release 二进制可以直接执行，不再编」。接手者的报告写「在上一位草稿上改了四处写法」，大部分被用上。

**37**：17:05:03 撞了会话限额（它本身就是一次重派）。17:30:26 主 agent 说「The usage limit hit all nine running agents at once…」，17:31:17 派 aadd40d58df7b03de「m3-ring-r2 攻方（接手）」（verdict-archived）。提示里写「报告还没落盘…已读的不必重读全文」。前任手上只有读过的东西，白做了大半。

**38**：和 14 是同一次撞顶。17:31:54 派 a477254ab7d199fa5「7-5b 接手：脚本收尾」（report-used-by-main）。前任写的 183 行里有 176 行进了 git，基本被用上：e165-pin-red-examples.py 91 行进了 89 行，quote-kb.py 39 行全进。

## 依据：其实产生了效果（27）

**2、15**（阶段同步逐行判）：主会话 01:58:28 用 `for a in "1 F1-F4" … "4 F21-F23,F25-F26"` 逐份读 `m5-sync-judge-$i`，输出里有「3:   ✓ 报告判全了：201 行候选都有逐行判定」和「4:   ✓ 报告判全了：222 行候选都有逐行判定」。接着汇总出 fixes.tsv，02:00:40 派 kb-scribe「阶段同步改动写回」。

**3**（决策变更史第二份译文）：02:28:59 的拼接命令是 `cat $S/dh-part1/… $S/dh-part2/decisions-history.part2.$l.md $S/dh-part3/… > $S/decisions-history.$l.md`。`git status --short` 显示 `A  .claude/kb/decisions-history.en.md` 和 `.ja.md`。

**4**（核证己组）：10:58:56 主 agent 把十笔判定 `cat >> rot-verify-main/verdicts.tsv`。11:02 据 verdicts.tsv 从 checks-owed.md 挪出 41 行，dry-run 输出「42 处都命中得对」。

**5**（26-1r）：01:04:46 主 agent 说「26-1r 交回，审过收下」。01:05:09 用 merge-and-verify.sh 打 /tmp/claude-1000/m3-formal-26-1r/patch。01:05:17 告诉 singlefs-30「26-1r 已经打进主工作区」。

**6**（E156 第 6 次登记）：13:19:31 主 agent 说「六项认定我都接受，追加到登记末尾」，13:19:37 `cat >> research/prompts/e156-r6-prereg.md`。`git log --all -- research/prompts/e156-r6-prereg.md` 能找到 04ceaa904。

**8**（E187 档表上限）：13:34:46 主 agent 核了产物 sha256，13:35:56 拿这些数弹窗问用户。用户选了「T16b：末档 16287 (推荐)」，13:43:09 写回 D27 档表。漏判的原因：m5-dev 工作树和主仓各自取了 E187，撞了号，后来 m5-dev 把它改成 E194。m5-dev 里现在有 194-档表上限收益.md 和 e194-pack-tier-ceiling-*.out，git 里在 db4f75b5e「backup: m5-dev worktree before sync 2026-10-22」。

**12、17**（门禁样本修复）：两件都直接改在主工作区。
- 06:00:26 主 agent 说「313 份里原先判错 48 份，五个工具实现员都改完交回了」；06:07:29 输出「✓ 有样本的阶段判得都对（313 个样本）」。
- `code-source-discipline.sh:468` 现在是 `scanned_roots = ["scripts/experiment-apparatus", "research", "crates"]`。
- experiment-orphans-red/expect 现在含「E162  scripts/experiment-apparatus/e162-sample.tsv」。
- 两件的改动都在 021b4db09。

**16**（五·10 拿不准 10 处）：03:44:09 主 agent 说「十条都有答案。逐条定」。03:44:36 记进 records/2026-10-13-代码欠账批.md，原文是「主 agent 定：① `pointer.rs` 第 325、400 行：`<=` 是给 `NodePointer::empty_root()`…」。它写的行是草稿副本里的探针，本来就不该进仓。

**18**（三方腿定义加一句）：09:39:43 主 agent 用 git diff --stat 和 sha256 核过。调度记录写「定义批交回（m3-pending-b3-leg-defs/report.md 09b526a2…7ccd）：主 agent 现查 git diff --stat 4 个文件、6 加 3 删」。`.claude/agents/three-way-attack.md:48` 现在有「从 `scripts/agents/three-way-rerun-template.sh` 拷」这一句。

**19、39**（C439 存量事实表）：00:47:15 的记录是「11-1b～e 交回（`~/.local/share/singlefs/work/c439-stock-{b,c,d,e}/report.md`，sha256 依次 e0c872ed…、a9cebfdb…、59d8b481…、dbe873df…）。五组合计 361 行」。00:47:57 拿合计弹窗，用户据此改定了存量的处置。

**20**（三·6 摸底）：16:54:59 `sha256sum /tmp/claude-1000/m3-prep-3-6/report.md` 后记进调度记录，16:55:28 据此派 kb-scribe「写回 7：layout/03 补登与过时句」。

**21**（E192 第二、三段）：12:02:03 主 agent 说「E192 交回了。照你说的只核、只记」，核了末行、26 条变异和关键数。12:02:22 记进 m5-dev 的调度记录，12:02:33 把 C4F5 交用户定。

**22**（A8 盘点）：07:33 主 agent 把 A8 的 30 行分进 slice2-用户定.tsv。09:47 的写回提示写着「Sources: slice2-用户定.tsv (30 rows from A8, reports in rot2-A8/report.md)」。09:51 的记录是「写进 9 个文件…mutation-triage 第 5 步另补…（A8 报告的建议）」。

**23、30**（kb 写回甲、D13 夜间档）：写回员直接改了 kb。主 agent 09:03:17 说「写回员甲交回。它点出的几处旧说法我自己改」；10:08:55 说「它留了四个点要我判」，逐条采纳，并补了两行用户定案。D8 里的 K-R3-3 在 0c065788f。

**24**（E185 修订）：13:38:14 主 agent 直接读它的装置草稿 m4-gpugc-runner-2/e185_gc_batch_wall_clock_split.rs，13:38:29 说「配对用的挂钟换成 `CLOCK_MONOTONIC_RAW`」，在它的草稿上接着改。

**25**（工具 T2 定义）：08:13:59 主 agent 拿它留的改前副本对过 diff，08:14:08–10 给三个在跑的子 agent 发「定义改了」的通知，08:14:33 弹窗请用户豁免，08:15:08 写进 agent-def-review-exempt。这些行后来被更新的定义改写了。

**26**（S31 逐行判）：05:55:58 跑 `--check-report … $(ls ~/.local/share/singlefs/work/rot-judge-S*/report.md | sort)`；05:56:06 用 `for seg in ['S31',…]: open(f'{W}/rot-judge-{seg}/report.md')` 收逐行判定；05:58:37 写回员的提示里写「判决：回扫逐行判的 35 份报告」。

**28**（复用键审计路子三）：17:29:20 主 agent 跑 `W=~/.local/share/singlefs/work; sha256sum … $W/audit-reuse-keys-by-logs/report.md`，17:29:24 读它的不合理项清单。17:30:40 记进 records/2026-10-13-复用键与全仓改名.md「2026-10-15 重型门禁复用键三路审计对账」。

**29**（第 17 批 Q）：16:13:46 `sha256sum /tmp/claude-1000/m3-batch17/q/patch/*`；16:44:46 约定 merge-and-verify 依次打 m3-batch17/p/patch、m3-batch17/q/patch。今天 unlink_mount_continuation_and_the_emptied_version.rs 里「given_a_time_earlier_than_the_previous_version」命中 3 处。

**32**（Explore：调度脚本现状）：22:53:36 主 agent 照它的脚本一览写了「现有脚本现状」，比如「host-dispatch.py：选机器 + 判长活…」，22:53:45 据此弹窗问排队语义。之后用户把这件事搁置了，但调研本身被用了。

**34**（写回起草 2）：12:04:00 主 agent 说「43 项全部命中…照收」，12:04:03 用 `for i in (1,2,3,4): json.load(open(f'{W}/rot-wb-{i}/spec.json'))` 查四份规格有没有撞行，12:04:22 在副本上合并施加。

**35**（E195）：06:20:49 主 agent 说「K3–K7 五行都够判…先核」，核了修订时刻和产物，06:21:37 记进 m5-dev 的 records/2026-10-22-里程碑五第二轮.md。

**36**（批 10 工具 P1–P7）：13:27:10 在集成副本上 `git apply …/m3-tooling-b10/patch.diff && git commit -qm '批 10：第二题第一轮改法 P1–P7' && git tag after-b10`，13:28:04 补了两处，打 after-b10-fixup。这些行后来被之后的批次改写了。

## 原因归纳

### 被重做与丢了，一共 11 个

| 原因 | 个数 | 哪几个 |
|---|---|---|
| 用量撞顶（会话限额或周限额），恢复后抽交接摘要新派同类接手 | 5 | 9、14、33、37、38 |
| 客户端关闭、会话断，没交回，重派 | 2 | 27、31 |
| 别的会话动了它要用的东西（清盘删了冻结副本、同时整目录搬 scripts/），主 agent 停掉，之后重派 | 2 | 1、10 |
| 别的会话搬目录时停掉，记成「下轮开」，之后没人接 | 1（丢了） | 11 |
| 派出后马上拿到新证据，补了输入再派 | 1 | 7 |

题里列的几种原因：「派发提示缺输入导致白跑」「交回内容主 agent 没读到」「同一件事被重复派」，这 40 个里一个都没有。

重做里前任白做了多少（折算，百万）：
- 全白：1、7、10、27、37，合计 1.96；
- 部分用上：9（模型目录）、31（计划表），合计 2.09；
- 基本用上：14、33、38，合计 9.32。

也就是说，「被重做」那 13.37 里，真正白做的约 2 到 4，大头是撞顶以后接手者照交接摘要接着做的。

### 主 agent 有意弃用，2 个

两个都是用户的话改了方向：0 是用户缩小范围，13 是用户纠正口径。两个都只跑了几分钟。

### 机械层为什么把这 27 个漏判

| 漏判原因 | 个数 | 哪几个 |
|---|---|---|
| 主 agent 点它的路径是用变量、循环、通配或花括号拼出来的（`$S`、`$W`、`m5-sync-judge-$i`、`rot-judge-S*`、`c439-stock-{b,c,d,e}`、`rot-wb-{i}`） | 8 | 2、3、15、19、26、28、34、39 |
| 从交回里抽出的产物路径是坏的：全角括号和后面的字粘进了路径，例如「report.md（298」「report.md（sha256」 | 4 | 20、22、23、30 |
| 交回正文本身就是结论，主 agent 照它记账或定案，不再点报告 | 3 | 4、16、32 |
| 直接改在主工作区，主 agent 直接验改动（跑样本自证、git diff），不点报告 | 3 | 12、17、18 |
| 主 agent 用的是它的补丁目录或仓内登记文件，不是报告路径 | 3 | 5、6、29 |
| 写在 m5-dev 工作树（还没合回主仓），机械层只比主仓和 git；8 号另外还改过号 | 3 | 8、21、35 |
| 当时用上了，之后被后续修订改写，所以出现在 git 里的行不到 30% | 3 | 24、25、36 |

对机械层的建议（不在这次的活里，供主 agent 参考）：
- 抽路径时在全角括号处截断；
- 匹配主会话命令时，先把 `~`、`$W`、`$S` 这类在同一条命令里赋过值的变量展开，再展开花括号和通配；
- written-* 那几桶把 m5-dev 这类工作树和它们的 backup 提交也算进去。

## 没做什么

- 没改仓内任何文件，没做 git 写操作，没编译，没跑测试和门禁，没提交。
- 后继者的结局只看到桶名、交回开头几百字，以及上面写到的那几处仓内现查，没有逐个读完后继者的会话。
- 「白做多少」是按提示原文和机械层的 lines_found 粗分成三档，没有逐行比前后两任写的东西。
- 外推只按桶分层，没有按 agent 类型或会话再分；report-used-by-later-sub 和 written-unused 两桶各只有 1 个样本。
