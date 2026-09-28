# 门禁逐格盘点（第二轮第 1 步）

调度记录：records/2026-09-28-门禁59号提速与双机分片.md「第二轮：门禁收缩、提炼、三方与测试」第 1 步。日期 2026-09-28。
只交事实，不交「该合、该砍」的结论。

## 交付

- 逐格表：`/tmp/claude-1000/gate-inventory/cells.tsv`，84 行数据 + 1 行表头，10 列，制表符分隔（`awk -F'\t' '{print NF}'` 85 行都是 10）。
- 表里「本文件:N」指第 1 列那份门禁文件的第 N 行；「NN 号:N」指 `.claude/gate.d/` 下以 NN 开头的那份的第 N 行；别的写全路径加行号。所有「文件:行」都是这一次现查的（一段脚本核过 506 处引用的行号不越界，余下 2 处是抄自记录的装置源码行号 `new_pool_file_creation_on_device.rs:1475`、`publish_order_matches_litmus.rs:47`，照原记录写）。
- 各格原样日志：`/tmp/claude-1000/gate-inventory/logs/<门禁>__<格>.log`；计时：`timings.tsv`（门禁、格、退出码、毫秒）；原 47 号那一趟：`old47-run.log`、`old47-run-attempt1.log`。

## 一、总数

```
$ wc -l .claude/gate.d/*.sh | tail -1
  15276 total
$ ls .claude/gate.d/*.sh | wc -l
23
```

格数按每份文件的格名数组数（`CELLS=`、`CELL_NAMES=`、`CELL_ORDER=`、`GRID_NAMES=`；没有数组的算整道一格）：

```
10-references-and-invariants.sh	7
11-review-and-sync-records.sh	7
12-doc-forbidden-notations-and-old-terms.sh	3
13-code-vague-names-and-test-file-names.sh	2
14-harness-one-scenario-per-test.sh	1
15-research-build.sh	1
20-doc-decision-documents.sh	13
27-code-constants-enums-bits-match-kb.sh	3
30-decision-history-entries.sh	3
32-doc-field-and-layout-registry.sh	9
33-code-experiment-and-mutation-source-discipline.sh	3
34-doc-experiment-pages-and-products.sh	10
43-checks-owed-and-closeout.sh	5
47-code-tooling-selftests-and-registries.sh	8
54-layer0-replay.sh	1
55-qemu-device-streams.sh	1
57-lkmm.sh	1
59-crates-mutation-replay.sh	1
74-model-differential.sh	1
77-test-environment.sh	1
87-replay.sh	1
92-layout-checker-sync.sh	1
94-checker-implementation-disjoint.sh	1
总格数 84
```

其中 74 格在 14 道带格名数组的门禁里，10 道是整道一格。没写 `# gate-category:` 的 13 道（10、11、15、30、43、55、57、59、74、77、87、92、94）第 3 列按 records 那份的五类表填、标「推」（records/2026-09-28-门禁59号提速与双机分片.md:63–69）。

## 二、怎么跑的

- 逐格顺序跑，一次一格：`nice -n 19 bash research/scripts/capped.sh 8 timeout -k 10 300 bash .claude/gate.d/<门禁> --check <格>`，在仓根上跑、不给项目根参数（默认取脚本往上两级）。整道的三份（77、92、94）不带 `--check`。脚本在 `/tmp/claude-1000/gate-inventory/run-cells.sh`，清单 `cell-list.tsv` 77 行。
- 开跑前看负载：没有 qemu、vm-bench、e152、fio；load average 2.46（32 核）；本机另有 vllm 本地模型与几个 claude 会话在跑。跑的是**工作区**（不是 `--staged` 的临时树），工作区里有几个会话的未提交改动，所以取改动范围的格（11 号几格、34 号几格、20 号 settled-same-file、30 号几格）判的是这些改动合在一起的样子。
- 没跑：54、55、57、59、87（重型）与 15、74（要编译），第 6、10 列写「不跑」。
- 两格撞了 300 秒停掉：12 号 term-renames（日志只有格名一行）、47 号 research-script-selftests。
- 跑的过程中门禁文件没变（`find .claude/gate.d -maxdepth 1 -name '*.sh' -newer cell-list.tsv` 零命中）；20 号的暂存状态开工时是 `AM`、收尾时是 `A `（有人在这期间暂存了它，文件内容没变）。别的会话在这期间改了 `.claude/rules/changelog-format.md`（「门禁管哪一半」从第 51 行挪到第 52 行），表里引它的行号按收尾时的文件改过。

## 三、今天的判定

按 cells.tsv 第 10 列数：绿 54、红 17、77 有 4、不跑 7、不跑完 2（合计 84）。

```
$ awk -F'\t' 'NR>1 {v=$10; sub(/[（：(].*/,"",v); print v}' cells.tsv | sort | uniq -c
      4 77
      7 不跑
      2 不跑完
     17 红
     54 绿
```

红的 17 格（第一处点名见 cells.tsv）：10 governance-refs；11 batch-scope、crates-adversarial-review、abandoned-rounds、knowledge-sync、agent-def-adversarial-review；12 prime-marks；13 vague-names；20 item-ref-status；34 results-cited、evidence-in-repo、decision-links、archive-past-rounds；43 paid-cited-tests；47 stage-owners、research-gate-lint；77 整道。

其中几处红的对象是这一轮合并本身带出来的（事实，逐条可核）：
- 10 governance-refs 列了 28 处指不到的指向（`grep -c '✗ \.claude/\|✗ CLAUDE.md'` 数得 28：旧门禁号 20 处、不存在的路径 8 处），全是旧门禁号与旧路径：例 `.claude/agent-common.md:28` 的 `.claude/gate.d/91-archive-past-rounds.sh`、`.claude/agent-common.md:99`「69 号」、`.claude/main-agent.md:65`「75 号」、`.claude/rules/path-moves.md:33`「21 号」、`.claude/rules/verification.md:81`「62 号」、`CLAUDE.md:79–81`「93 / 90 / 16 号」、`.claude/agents/kb-scribe.md:32` 的 `21-decision-items-sync.sh`（全表在 `logs/10-references-and-invariants__governance-refs.log`）。
- 47 stage-owners 点名合并出来的 10、11、12… 号在 `stage-owners.tsv` 里没登记。
- governance-refs 只扫治理文档，kb 里按旧文件名称呼门禁的它看不见：`.claude/kb/decisions.md:14`（`37-decision-summary-width.sh`）、`.claude/kb/decisions.md:83`（`22-item-ref-status.sh`）。

## 四、47 号 research-script-selftests 的四份

这一格先跑 runner 表 45 条（`.claude/gate.d/47-code-tooling-selftests-and-registries.sh:190–217`），全过了才进覆盖段（:230–286）。逐格计时那一趟在 runner 表里撞了 300 秒，没走到覆盖段。覆盖段是一段内嵌 python，我把它原样抽出来（`/tmp/claude-1000/gate-inventory/coverage.py`，与新 47 号 :237–263 那一段同字节）在真仓根上跑，传同一条 NOT_RUN_HERE：

```
CLAIMED	53	RUN	48
MISSING	research/scripts/layer0-shard-configuration-check.sh
MISSING	research/scripts/layer0-shard-run-selftest.sh
MISSING	research/scripts/memory-peaks.tsv
MISSING	research/scripts/rerun-failed-tests.py
EXEMPT	research/scripts/vm-bench.sh	自证要连起三次虚机，挂钟太重，不进每轮门禁
```

「声称有 --selftest」的认法是文件全文里出现 `--selftest` 这串字（:241–242 `"--selftest" in open(path…).read()`），不看它是不是自己的开关。逐份现查：

| 文件 | 自己有没有 `--selftest` | 依据（`grep -n -- --selftest`） |
|---|---|---|
| research/scripts/layer0-shard-configuration-check.sh | 没有，只在注释里提别人的 | :24「research/scripts/layer0-shard-run.sh --selftest 里对应那一格判错」、:27「layer0-shard-run.sh --selftest 用」 |
| research/scripts/layer0-shard-run-selftest.sh | 没有，它是别人的自证本体 | :2「research/scripts/layer0-shard-run.sh --selftest 的本体」；`layer0-shard-run.sh --selftest` 在 runner 表里（47 号:208） |
| research/scripts/memory-peaks.tsv | 没有，是数据文件 | 内存包装写的峰值日志，第 248、249、598、743、1012、1081、2200、2207 行记的是别的命令行里的 `--selftest`；它被 .gitignore 挡着（`git check-ignore -v` → `.gitignore:14:/research/scripts/memory-peaks.tsv`），`--staged` 的临时树里没有它 |
| research/scripts/rerun-failed-tests.py | 有 | :8 用法「rerun-failed-tests.py --selftest」、:154 `if arguments == ["--selftest"]`；runner 表与 15 号都没调它 |

所以四份里三份是认法认错（文本里提到而已），一份（rerun-failed-tests.py）是真有自证、没人跑。

合并前的原 47 号在同一份仓上判不判它们：

- 覆盖段两份逐字相同：`diff <(sed -n '/^NOT_RUN_HERE=(/,/^PY_COVERAGE/p' old47.sh) <(sed -n '/^NOT_RUN_HERE=(/,/^PY_COVERAGE/p' 新 47 号)` → 「覆盖段逐字相同」。它按被判仓的 `.claude/gate.d/[0-9][0-9]-*.sh` 认「有阶段在跑」，不看自己那张 runner 表，所以在同一棵树上两份判得一样。
- 实跑：把 `git show HEAD:.claude/gate.d/47-research-script-selftests.sh` 拷进 `/tmp/claude-1000/gate-inventory/old47-tree/.claude/gate.d/`（sha256 d1b39662…121b，与 HEAD 那一版相同），`.claude/scripts` 与 `.claude/singlefs-ai-sop` 两个符号链接指回真仓，给真仓根当参数跑。第一次少了 `.claude/singlefs-ai-sop` 链接，preflight 当场拒（`old47-run-attempt1.log`，rc=1，0 秒）；补链接之后第二次原样输出（`old47-run.log`）：

```
  ✗ 这些 research 脚本声称有 --selftest，却没有任何门禁阶段在跑它：
      research/scripts/layer0-shard-configuration-check.sh
      research/scripts/layer0-shard-run-selftest.sh
      research/scripts/memory-peaks.tsv
      research/scripts/rerun-failed-tests.py
  → 怎么办：把它加进上面的 runner 表；确实不该每轮跑的，登记进 NOT_RUN_HERE 并写明为什么——没人跑的自证只在写它的那天跑过一次。
rc=1 用时 344 秒
```

  原 47 号的 runner 表 41 条（新的 45 条，多了 mutation-shard-run.sh、multi-host-run.sh、crates-mutation-rows.py、migrate-changelog-format.py 四条）在今天的工作区上全过（日志里没有一行 runner 的 ✗），然后覆盖段报同样四份，退 1。
- 另在 HEAD 那棵树上现算（`git show HEAD:` 取出 research/scripts/ 78 份与 `.claude/gate.d/[0-9][0-9]-*.sh` 79 份放进 `head-coverage-tree/`，跑同一段覆盖代码）：`CLAIMED 49 RUN 44`，MISSING 是 harness-test-timing.py、layer0-shard-configuration-check.sh、layer0-shard-run-selftest.sh、rerun-failed-tests.py。也就是说三份在合并之前的 HEAD 上就判红；memory-peaks.tsv 只在工作区出现（没进 git）；harness-test-timing.py 今天在暂存区里是删除（`git status` 为 `D `）。

## 五、跑的先后有依赖的

1. **77 号要在所有跑测试的阶段之后，而按文件名排它不在最后。** 共享 gate.sh 按文件名排序逐个跑本地阶段（`.claude/singlefs-ai-sop/scripts/gate.sh:636` `find "$GATE_D" -maxdepth 1 -name '*.sh' … | sort`）；77 号自己的分界写的是「这一阶段排在最后」（`.claude/gate.d/77-test-environment.sh:40`），取祖先 gate.sh 的启动时刻当分界、晚于它的临时条目不判红（:38–41）。按文件名 87、92、94 排在 77 后面；87 号起 research/scripts/replay.sh，它的输出目录是 `${TMPDIR:-/tmp}/singlefs-replay-$$`（replay.sh:30），名字落在 77 号扫的 singlefs-* 里（77 号今天的日志「共查 206 项 singlefs-*」）。推的，没量过：87 留下的东西这一趟 77 看不见，下一趟门禁开跑之前留下的会被下一趟 77 判红（与 C474 同形，`.claude/kb/checks-owed.md:396`）。92、94 只读文本。
2. **共享阶段先于本地阶段**：「本地阶段判别力」stage-selftest（gate.sh:404）与「构建与单测」check.sh（gate.sh:539）都在「项目本地阶段」（gate.sh:631）之前跑；check.sh 的 `cargo test --all` 已在 debug 下跑过 54 号快档与 74 号那批用例（见 cells.tsv 第 7 列）。
3. **跨趟的依赖**：55、57、59、74、87 号先问 research/scripts/stage-must-run.sh，比 `refs/sop/staged-green` 那棵树与这次暂存树（stage-must-run.sh:16）；那条 ref 只在 research/scripts/gate-staged.sh 起的整轮全绿之后才前移（gate-staged.sh:4、:47）。所以这几道「这一次要不要跑」取决于上一次整轮门禁有没有全绿。54 号快档核的全绿标记由此前的 `--full` 写（54 号:21），同样跨趟。
4. **写回的次序**：30 号 status-sync 的 `--write` 按 decisions.md 索引表的「状态」「结论（简报）」两格重写史册节顶的现状行（30 号:57–60）；20 号 decision-items-sync 的 `--write` 重写的正是 decisions.md 索引表的「状态」列（20 号:26、:35–36）。修红时 20 号的 `--write` 要先于 30 号的 `--write`，反过来 30 号写进去的是旧状态。门禁里的判定次序（20 在 30 前）与这个一致。
5. **归档与同步记录打架**：34 号 archive-past-rounds 要求删掉上一轮的记录，11 号 knowledge-sync 要求改动范围里的同步记录还在；C450 仍欠（`.claude/kb/checks-owed.md:376`），34 号的出路里写了「不给基准…删完门禁 11 号的 knowledge-sync 格当场红」（`logs/34-doc-experiment-pages-and-products__archive-past-rounds.log`）。这是 `--apply` 与判定之间的先后，不是门禁里的次序。
6. 10 号 citations 在整道七格全跑时先在后台起、轮到它时再 wait（10 号:23–24），是道内的并行，与别的阶段无关。

## 六、重复与重叠（cells.tsv 第 7 列的汇总，只列事实）

第 7 列不写「无」的 29 行（`awk -F'\t' 'NR>1 && $7 !~ /^无/' cells.tsv | wc -l` → 29）。按性质分：

- **同一份判法跑两遍**：33 mutation-tables 的 crates 那一半与 59 号派活前判的是同一份 `row_problems`（59 号:8–10、:14；33 号:106–108）；47 agent-write-scope 的「注册着、自证过」与共享「工具层的闸」hooks-registered.sh ①②（hooks-registered.sh:12–14）判同一件事，整轮里 .claude/hooks/ 下这些钩子的 `--selftest` 各跑两遍；54 号快档、74 号那批用例先被共享 check.sh 的 `cargo test --all`（check.sh:97）在 debug 下跑一遍。
- **同一判据落在两处**：43 table-shape「同一个编号在两张表里各登记一次」（43 号:103）与共享 doc-lint F「一个编号只许有一处登记位」（doc-lint.sh:770–777），checks-owed.md 两张表都带 registry 标记（:13、:484）；12 prime-marks 与 43 closeout-collects-open ④ 在收口表首列的 U+2032、U+2033 上重叠（43 号:223）；20 kb-shape 第 7 段与 20 decision-items-sync 判同一个量（20 号:1765–1768 写明是故意留的两条路）。
- **同一件事分两道**：研究脚本自证 5 份在 15 号（15 号:85–139）、45 条在 47 号 runner 表。
- **同对象不同判据 / 同数据源**：10 experiment-refs、decision-refs 与 doc-lint H（≥3 次、只 kb）；13 vague-names 与 naming-lint；30 shape 与 history-ordinal；30 status-sync 与 34 index-sync（都读 experiments.md 索引）；27 feature-bits 与 92 ③（都读 D15 已定项 4 登记表）；32 segment-registry 与 55 direct 档（都比 E142 产物）；20 status-redundancy、kb-shape 第 5 段与 34 decision-links ⑦（决策首行与分项索引，format-evolution.md:27）；32 format-const-placeholders 与 10 decision-refs；47 fixture-claims ④ 与 doc-lint M（同一个 date_out_of_range）；47 research-gate-lint 与共享 gate-lint、shell-lint、script-modes（同工具、目录不重叠，只有 .claude/gate.d/ 上的终止进程扫描与 shell-lint S2 重叠）；47 rules-manifest 与 doc-lint K（同判据、对象不重叠）；77 与 gate.sh「跑完没留下临时文件」；92 与 27 format-constants（同库）。

## 七、依附的对象过期或已关的（第 8 列里挑出来的事实）

- 依附已关里程碑一「新池新建文件」（出口已满足，CLAUDE.md:8）的格：32 号 field-refs、field-projection、first-txn-hooks、format-const-placeholders。对象文件都在；first-txn-hooks 今天判「是」的未定项 0 条。
- 依附里程碑二收口表具体一行的格：11 abandoned-rounds（第 33 行，02-second-txn.md:367）、32 second-txn-hooks（第 31 行，:365）、32 tree-table-reserve（第 17 行，:348）、43 closeout-collects-open（第 34 行，:368）、43 row27-preconditions（第 27 行，:361，那一行今天写「这一行已有去向，不再等前置」）。几行都还在。
- 依附的欠账已还清、格还在盯的：C4（47 rules-manifest）、C11（27 feature-bits）、C12（94）、C14（92）、C31（20 stale-open-items）、C38（10 citations）、C45（20 freeze-layer-membership）、C94（32 field-table-sums）、C316、C485（32 segment-registry）、C327（33 mutation-tables）、C338（32 tree-table-reserve）、C358（43 audit-contradictions）、C382、C384（47 research-gate-lint）、C396（77）、C468（11 knowledge-sync）、C472（11 batch-scope）。行号见 cells.tsv。
- 没有任何规则或定义点名的格（只靠欠账或记录撑着）：11 batch-scope、11 abandoned-rounds（`grep -rn` 零命中，命令在 cells.tsv 第 8 列）、27 clause-enums、34 verdict-false-named、repro-command、experiment-orphans、43 paid-cited-tests、15 整道。
- 史册改成按决策分节之后：原 48 号（条目住在日期所在月的那一份）与原 49 号（快查与原文同步）判的月份文件已不存在（`.claude/kb/decisions-history/` 不在），30 号的 shape、status-sync 两格判的是新形态（.claude/rules/changelog-format.md:7–43）；20 item-ref-status 今天的 131 处红列出来的都在 decisions-history.md。

## 八、什么现象会推翻这份表

- 某格计时：同一条命令在负载更低时重跑，时间差一个量级以上（这一次是 nice 19、单格串行、load 约 2–3 / 32 核）。
- 「抓到过没有」写「没找到记录」的格：在 records/、.claude/kb/、research/prompts/ 里找到一条它判红而是真问题的记录（我按旧号、`.claude/gate.d/<旧文件名>`、判红原句搜，另搜了强信号词「转绿、改回、抓到、查出」，逐条读的只是命中的那一部分，见「没做什么」）。
- 「今天」那一列：工作区在变（别的会话在改），同一格重跑判定可能不同；红的多数点名的是别的会话在途的改动。
- 47 号四份的结论：rerun-failed-tests.py 之外的三份，只要在文件里出现一处真的 `--selftest` 开关处理，就不是认法认错。

## 九、没做什么

- 54、55、57、59、87 号（重型）与 15、74 号（要编译）没跑，第 6、10 列写「不跑」；它们的「判什么、读什么」取自文件头与 stage-inputs.tsv。
- 12 号 term-renames、47 号 research-script-selftests 撞 300 秒停掉，没有完整判定；47 号那一格的覆盖段另外现算了（第四节）。term-renames 没有另算。
- 「抓到过没有」一列不是穷举：我按旧号（「门禁 N 号」「N 号」、旧文件名）在 records/、.claude/kb/、research/prompts/ 里搜到 3716 行命中（`hits.txt`，`grep -vc '^=== ' hits.txt` 数），只逐条读了 records/ 与 kb 里的那一部分（`hits-kb-records.txt`）和带强信号词的一部分（`strong-hits.txt`），research/prompts/ 里的大半没逐条读；写「没找到记录」的格可能有漏的。也没搜 git 历史里已归档的文件。
- 「与谁重复」只比了共享 gate.sh 第 380–560 行那批阶段与这 23 道之间；没比钩子（除 write-guard 共用库那一处）、没比 research/scripts 里不属于门禁的检查。
- 各格的红没有修，也没判归属是这一轮的还是别的会话的（只按日志写了点名的文件）。
- 没跑 `gate.sh` 整轮与 `gate-staged.sh`，没做任何 git 写操作，没改仓里任何文件。
- 删了自己在草稿目录建的两份仓副本：`/tmp/claude-1000/gate-inventory/head-coverage-tree`（2.9M，HEAD 的 research/scripts 与门禁阶段）、`/tmp/claude-1000/gate-inventory/old47-tree`（24K，原 47 号的拷贝与两个指回真仓的符号链接）。没删：`logs/`（428K，各格原样日志）、`old47-run.log`、`old47-run-attempt1.log`、`old47.sh`、`coverage.py`、`timings.tsv`、`hits*.txt`、`gen_part*.py` 与 `assemble.py`（cells.tsv 的生成器）——都是这份报告的依据，主 agent 要核。
