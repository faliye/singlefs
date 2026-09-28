# gate-shrink-r1 云端攻方（Opus）报告

日期 2026-09-28。攻击面：第 3、4、6 问与方案 A 里的 doc-kb（正文「四、分工」云端攻方那一行）。第 1 问的逐格映射归本地攻方，这里不做那张表。

## 复跑

- 模型目录：`research/prompts/gate-shrink-r1-opus-model/`；逐个文件的 sha256 在同目录 `SHA256SUMS`（下面整份贴出）。
- 整套：`bash research/prompts/gate-shrink-r1-opus-model/rerun.sh <仓根> <草稿目录>`。各件单跑的用法写在各自文件头；M6、M6b、M9 各要十几分钟到二十来分钟。
- B4 的 74 号实跑（要编 release 的 singlefs-harness，不在 rerun.sh 里）：
  `SINGLEFS_GATE_FULL=1 GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G CARGO_TARGET_DIR=<草稿目录>/target nice -n 19 bash research/scripts/capped.sh 8 bash .claude/gate.d/74-model-differential.sh`
  （74 号在阶段里自己经内存包装，外面不再包一层；原样输出 `outputs/stage74-run1.log`）。
- 各件是什么：M1 doc-kb 五道的样本在别的四道上判什么；M2 「声称有 --selftest」的四种认法；M3 `--staged` 的链接 worktree 上 evidence-in-repo 判什么；M4 去编号后的阶段次序；M5 治理文档里按全名指门禁；M6 term-renames 的掩码正则（原样与加左边界）；M6b 照 `sweep-term.py --check` 的走法分三段计时；M7 B1、B2、B3、B8 的违规输入放进仓副本；M8 按暂存树现算 batch-scope；M9 research 脚本自证逐条计时与分批并行。

模型文件的 sha256（`outputs/` 下 35 份原样输出的 sha256 在 `SHA256SUMS` 里）：

```
566dd7013a3852ae8243f3f15055c86fb4857f5291af0cc22647a8ded30c939b  ./m1_want_cross_hits.sh
cad7e5a230dcaf0f55dbeaf5c851481b73dd924b9e1629d094d49a2da50f831b  ./m2_selftest_claim_recognizers.py
86579227a73105a510d7f056d9a293ec5b34ac1c170ad5f650dfe8018883df96  ./m3_staged_worktree_evidence.sh
ee4db8315d3454dff9e797caabd275140b4585fe6d824237924eaddfc19b78c4  ./m4_stage_order.sh
96221799098becc3fd37e7d9898e35c01bfee8d7ff6f6e2e11b99fe48bac8bd1  ./m5_gate_name_references.sh
80288271ee5ec03741a6d534c72c7fd3baea420bebf8e4c00b5b715e738ffa10  ./m6b_sweep_term_phase_totals.py
c491e70158272bae6d19a13c5f319ca5de41b61a6c5890320245c31a6a5937fa  ./m6_term_renames_mask_regex.py
70b3c62677f89b9282274907ea1564e5da52110167b60eefa7cb319c3d267807  ./m7_copy_injections.sh
f1f67df9a521ad59fef97e94deb32c1063535ad0ef4dce0a86c79eb71d04da02  ./m8_batch_scope_on_staged.py
e133be7d30225dd4e93a6138d65d783a18fd7c3c1192f0b383a54720be1a42a1  ./m9_research_selftest_timing.sh
e13df3a694abdef51d82527a644edcd5e19272c5ebdf34db3a894aa37d8d3a86  ./rerun.sh
```

## 各格判定一览

「量过」= 在模型或副本上真跑出来、原样输出落在模型目录；「推的」= 按代码推、没实现没跑。副本上的数不算入库装置上的数。

| 格 | 判定 | 证据 | 一句话 |
|---|---|---|---|
| B1 删 row27-preconditions | 打中 | 量过（M7 V1） | 删后收口表第 27 行那几笔（C333 行回收等）的前置进了 `crates/` 也没人判；那一行正文还写着交门禁 89 号盯着 |
| B2 删 agent-write-scope 里与 hooks-registered 重复的段 | 条件打中 | 量过（M7 V2） | 只删八个钩子的自证调用不丢东西；照字面「只留表与定义双向一致」连注册判据一起删，三种 matcher 收窄 hooks-registered 判绿、今天这一格判红 |
| B3 删 table-shape 与 doc-lint F 重叠的一半 | 没打中 | 量过（M7 V3） | 编号后面接全角空格的重复登记，doc-lint 仍红（G 报「裸引用」），只是没报成「登记了两处」 |
| B4 74 号与「构建与单测」 | 打中，不分辨两臂 | 量过（74 号实跑一次） | 六段模型对拍都在 `#[ignore]` 用例里；check.sh 与 74 号都不带 `--ignored`，74 号今天判红；两臂都没人跑模型对拍 |
| B5 research-script-selftests 的认法 | 条件打中 | 量过（M2） | 按「入口写法」列举的认法漏新写法（本仓今天六种写法的并集对五种新写法只认得一种）；按「去掉注释与出路文字」认的办法五种全认、只提名的两种都不认 |
| B6 77 号的次序 | 部分打中 | 量过（M4）+ 推的 | 依赖次序的只有与时刻无关的几类（残留设备、宿主盘剩余空间）；类名序下 77 号前面跑完 checker-tier，后面是 code、doc、harness；起名 `harness-test-environment` 会排在 `harness-tests` 前面 |
| B7 两格超过 5 分钟 | 定位了，给出不砍格的收法 | 量过（M6、M9） | term-renames 慢在已归档产物名的掩码正则二次回溯（一份 2 MB 的 E142 产物里有一段 65536 字的连续字符，单份要 60–82 秒），加左边界后同一份输出、快三个量级；research-script-selftests 45 条里两条占 68%，分批并行两次：400.9 秒到 178 秒、401.7 秒到 137 秒，45 条都过 |
| B8 删 experiment-refs、decision-refs | 打中 | 量过（M7 V8） | 规则文件里的悬空号、kb 里出现不到 3 次的悬空号只有这两格判；doc-lint H 要 ≥3 次且只扫 kb，number-name-sync 跳过没登记的号 |
| B9 三格长期红 | 判断（见正文） | 算过（M8）+ 推的 | batch-scope：判据对，提交时的暂存树上照样红（188 个触发文件 151 个没登记），缺一种成批登记的写法；knowledge-sync：判据对，是流程欠账，删了丢的是回扫；archive-past-rounds：「这一轮」按全仓算、没有在跑的轮的保护，多会话下判据本身不合用（推的） |
| 第 2 问 doc-kb「一格红淹掉别格」 | 打中 | 量过（M1、五道实跑） | 28/28 份红样本在别的四道上也退 1，8/8 份绿样本在别的道上判红，5/328 条 want 由别的道替打出来；分诊表按道一行、归属表按道一格，doc-kb 的归属与派修从此不分格 |
| 第 4 问 改名之后按全名指门禁 | 打中 | 量过（M5） | governance-refs 只认「两位数加号」与反引号里的路径；按全名写的指向（方案要的写法）悬空了也不红 |
| 第 4 问 47 号两处 `[0-9][0-9]-*.sh` | 推的 | 读码 | 全部改名时两格都响亮地红；只改一部分时 fixture-claims 静默跳过没编号的阶段 |
| 第 4 问 20 号 --write 先于 30 号 --write | 没打中（只多一轮） | 推的 | 并进 doc-kb 后次序由格表定；写反了下一次判定照样红，不会漏判 |
| 第 4 问 locale 改变次序 | 没打中 | 量过（M4） | 方案里的名字在 C.UTF-8 与 en_US.UTF-8 下排出同一个次序 |
| 第 6 问 evidence-in-repo 在提交时从不判 | 打中 | 量过（M3） | gate-staged.sh 的链接 worktree 里这一格开头就退 77；「扫仓的门禁只在提交前由 gate-staged.sh 跑」之后，kb 里拿 /tmp 当依据、装置改了没新产物两件事默认没人判 |
| 第 6 问 写入后钩子被删 | 事实 | git | 第一轮删了书记员的写入后钩子 kb-scribe-followup.sh 与它的表（暂存区 D），它每次写 kb 跑 31 道快阶段；该留在写入那一刻的列在第四节 |

## 一、B1–B9：删了、改了之后什么违规没人判

### B1 删 43 号 row27-preconditions

- 违规输入（M7 V1，副本上造）：在 `crates/singlefs-core/src/instance_table.rs` 末尾加一个用 `rows.retain(|row| *row != 0)` 的函数，也就是 C333（删行那次发布被重放）那一笔「行回收」的前置进来了，而没有补会红的用例。
- 今天谁判：43 号那一格的探针表里写着这一条：「C333（删行那次发布被重放）：行回收	crates/singlefs-core/src/instance_table.rs	rows.retain(	0」（`.claude/gate.d/43-checks-owed-and-closeout.sh:760`）。副本上加之前这一格退 77，加之后退 1（`m7-v1-before.log`、`m7-v1-row27.log`）。
- 删了之后为什么没人判：全部门禁脚本里读 `instance_table.rs` 的只有 43 号（M7 输出「v1 其他读 instance_table.rs 的门禁脚本：43-checks-owed-and-closeout.sh」）。C333 仍在欠着的那张表里（`.claude/kb/checks-owed.md:279`），而里程碑收口表第 27 行把「余下仍不可达的由门禁 89 号按前置立逐字探针（前置没进来退 77，动了就红）」（`.claude/kb/milestone/02-second-txn.md:361`）写成了那几笔的去向。删了这一格，这句话变成假话，governance-refs 只扫治理文档「CARRIER_PATTERNS = ["CLAUDE.md", ".claude/main-agent.md", ".claude/agent-common.md",」（`.claude/gate.d/lib-governance-refs.py:34`），不扫 kb，也没人报它悬空。
- 方案写的依据「里程碑二提交之后删」够不着这件事：那几笔（行回收、挂载内回收等）今天不可达，它们的前置要等以后的实现进来，恰恰落在里程碑二提交之后（推的：我没查里程碑三的范围）。要删，先把这几笔的去向改到里程碑三的收口表或一条会红的用例上。

### B2 删 47 号 agent-write-scope 里与 hooks-registered 重复的段

- 重复的只有「跑八个项目钩子的 --selftest、看退出码」这一段：hooks-registered 对每个带 --selftest 字样的钩子跑自证「grep -q -- '--selftest' "$hook" || continue」（`.claude/singlefs-ai-sop/scripts/hooks-registered.sh:86`），射程还更宽。只删这一段，没找到丢的东西。
- 方案原话是「只留表与定义双向一致」，照字面就连注册判据一起删了。hooks-registered 只核「有一条注册指向它」和文件头 `# hook-events:` 里写了工具名的那几个 matcher；write-guard.sh 没有 `# hook-events:` 行，heavy-test-guard.sh 与 session-start.sh 的那一行不带工具名或 matcher 词。违规输入（M7 V2，副本上造，三样一起放）：
  1. write-guard 的 matcher 从 `Write|Edit` 收成 `Write`（Edit 从此不过写范围闸与整份覆盖闸）；
  2. heavy-test-guard 从 matcher `Bash` 那一条挪到一条 matcher `Edit` 上（重型测试闸再也不在 Bash 上触发）；
  3. session-start 的 matcher 从 `startup|resume|compact` 收成 `startup`（压缩之后不再补回提示）。
- 结果：hooks-registered 退 0，末行「✓ 11 个钩子都注册着，其中 10 个的自检通过」；47 号 `--check agent-write-scope` 退 1，四行 ✗ 逐条点名这三样（`m7-v2-hooks-registered.log`、`m7-v2-agent-write-scope.log`）。今天判它的是这一格的「① `.claude/settings.json` 的 PreToolUse 里有一条 matcher 同时覆盖 Write 与 Edit」（`.claude/gate.d/47-code-tooling-selftests-and-registries.sh:445`）以及 ⑩⑫ 两条。⑧ omitClaudeMd、⑬ 共用模块不另抄、⑮ model 取值这三条 hooks-registered 根本不判。

### B3 删 43 号 table-shape 与 doc-lint F 重叠的一半

- 试的形状：同一个编号在已还清那张表里再登记一次，编号后面接全角空格（`| C333　| …`，输入法常打出来的）。43 号的认法「m = re.match(r'\|\s*(C\d+)\s*\|', s)」（`.claude/gate.d/43-checks-owed-and-closeout.sh:159`）里的 `\s` 认 Unicode 空白，报「C333 在第 279、604 行各登记一次」；doc-lint 取编号时「id = c[2]; gsub(/^[ \t]+|[ \t]+$/, "", id)」（`.claude/singlefs-ai-sop/scripts/doc-lint.sh:787`）只去半角空白，那一行登记不上，F 不报重复。
- 但 doc-lint 的 G 仍把那一行报成「C333 裸引用」，整道照样红（`m7-v3-v8-doc-lint.log`）。所以删掉重叠的一半，这个输入不会漏判，只是红行指的方向从「登记了两处」变成「引用没带简称」，出路指错了一步。算没打中。行形状、写着已还、被空行断开、少于两张登记表这四样 doc-lint 不判，要留。

### B4 74 号 model-differential 与「构建与单测」

- 事实：74 号要的六段报告全由标了 `#[ignore]` 的用例打出。`random_histories.rs` 里打「── 随机历史快档 ──」那一段的用例（`crates/singlefs-harness/tests/random_histories.rs:281`）头上是「#[ignore = "harness 耗时用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]」（`crates/singlefs-harness/tests/random_histories.rs:267`），另五段的用例同样标着（第 297、335、373、416、477 行一带，逐条列表见模型目录 `outputs/b4-ignored-tests.txt`）。
- 74 号跑的是「if bash "$MEMORY_CAP_RUNNER" "$MODEL_DIFFERENTIAL_MEMORY_MAX" cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then」（`.claude/gate.d/74-model-differential.sh:48`），不带 `--ignored`；共享「构建与单测」跑的是「if run_cargo test --all 2>&1 | tee "$test_output"; then」（`.claude/singlefs-ai-sop/scripts/check.sh:97`），也不带。
- 量过：在工作区上照阶段原样跑一次 74 号（命令在报告开头），退 1，原样末几行在 `stage74-run1.log`：「✗ 测试跑过了，这几段却没有「模型对拍 N 步」那一行：」后面列了全部六段。也就是说 74 号今天就是红的；盘点那一趟把它记成「不跑（要编译）」（`research/prompts/gate-shrink-r1-inventory/cells.tsv` 第 81 行）。
- 所以盘点第 7 列「74 号的用例已在 cargo test --all 里跑过一遍」对这六段不成立：那六条用例两边都不跑。方案给的两臂——留 74 号、靠「构建与单测」——在这件事上一起落空：留 74 号，它照今天的写法永远红（或者被人把 SECTIONS 删到绿）；靠「构建与单测」，模型对拍从此一步都不跑，违规输入就是让执行器绕开 `judge_by_model`（模型一步不判）——测试照过、没人红。
- 牵连：`crates/singlefs-harness/tests/` 下 9 份文件共 68 处 `#[ignore]`（harness 耗时用例），门禁里没有一道带 `--ignored` 跑它们；规则说轻用例是「- 轻：每次改完跑，`cargo test -p singlefs-harness` 不带 `--ignored` 跑到的全部。」（`.claude/rules/verification.md:23`），第 24 行写「完整的一遍归提交时的整轮门禁」（`.claude/rules/verification.md:24`），而整轮门禁里没有这一遍。方案里的 harness-tests（14 + 74）两格都不跑它们。

### B5 research-script-selftests 只认自己实现了入口的脚本

- 今天的认法把全文出现 `--selftest` 的文件都算「声称有」，多认了三份（两份只在注释里提别人、一份是数据文件），少认的没有；这是「宁宽」的一侧。
- 改成「只认自己实现了入口」，实现者得挑一套入口写法。M2 在 research/scripts/ 今天的 53 份上量了三种认法：
  - R1 只认最常见的两种写法（python 的 `== "--selftest"`、shell 的 case 分支）：认下 37 份，真有自证的 13 份掉出去（archive-past-rounds.py、gate-staged.sh、sweep-term.py、test-environment-check.py 等）。
  - R2 本仓今天六种写法的并集：认下 50 份，只去掉那三份误认，今天看是对的；但对临时造的五份新脚本（getopts 式的 `--selftest|-s)`、`match` 语句、集合求交、先赋常量再比、`test "$1" = --selftest`）只认得第一份。
  - R3 不列写法：只认 .sh / .py，先去掉 `#` 注释行与出路文字（与这一格认「有阶段在跑它」时去掉的是同一套），剩下的代码里还有 `--selftest` 才算：今天认下 50 份（同 R2），五份新写法全认，只提名的两份都不认。
- 违规输入：新写一份 `research/scripts/foo.py`，用 `match sys.argv[1:]: case ["--selftest"]:` 实现自证，不进 runner 表。今天判红（MISSING）；照 R1、R2 这类认法改完，它不在被扫集合里，没人要求它有阶段在跑，也没人报。
- 所以打中与否取决于实现：列写法的打中，R3 这种不打中。

### B6 77 号 test-environment 的次序

见第三节「去编号之后的次序」第 1 条。

### B7 两格超过 5 分钟：慢在哪，怎么收而不砍格

- term-renames：在 research/scripts/sweep-term.py 里，时间几乎全花在「已归档产物名」的掩码上，不在改名检测上。掩码正则是「PRODUCT_FILE_NAME_PATTERN = re.compile(r"[A-Za-z0-9_.-]+\.(?:out|log|txt|tsv|json|csv)\b")」（`research/scripts/sweep-term.py:42`），没有左边界，遇到很长的一段 `[A-Za-z0-9_.-]` 字符（E142 的干跑产物里有 65536 字一段的十六进制）会在每个起点重扫整段，二次回溯。M6 在 research/results/ 上逐份量：逐份的数在模型目录 `outputs/m6-all-results.tsv`，汇总在第七节；同一个正则只加左边界 `(?<![A-Za-z0-9_.-])`，换出来的文本与被换下的原串逐份相同，时间从几十秒降到百分之一秒量级。改名检测本身 0.05 秒/MB 上下，全仓 217 MB（2145 份不豁免的文件，research/results/ 占 185 MB）十来秒。
  - 要防的「收」法：把 research/results/ 豁免掉（sweep-term 要求源码与产物一起换，产物漏换 replay 的逐字节复跑就对不上）；只扫改动范围（往 `.claude/kb/term-renames.md` 加一行新改名时，要扫的恰恰是没改动的旧文件）。这两种都会让「全仓不再出现旧名」变窄；加左边界不变。
- research-script-selftests：runner 表 45 条逐条顺序跑合计 400.9 秒，其中 layer0-shard-run.sh --selftest 150.3 秒、agent-watch.py --selftest 121.6 秒，两条占 68%；再往下是 admission.py 41.9 秒、run-with-memory-cap.sh 35.5 秒、mutate.sh 20.4 秒（`outputs/runner-times-first.tsv`）。按耗时从长到短每批 8 条并行，整批挂钟 178 秒，45 条退出码都是 0（`outputs/parallel-first.txt`）；M9 再跑一次：顺序 401.7 秒，分批并行 137 秒，45 条退 0。要防的「收」法：只跑脚本本身改过的那几条自证——被测脚本 import 或 source 的共用模块（admission.py、lib_shell_words.py、changed-paths.sh 这类）改了，依赖它的自证不重跑，坏了没人知道。
- 两格都有不砍格、不缩射程的收法，量过的只在我的副本与模型上，被攻过零轮。

### B8 删 10 号 experiment-refs、decision-refs

- 违规输入（M7 V8）：在 `.claude/rules/verification.md` 末尾加一行引「E996（臆造的实验）」；在 `.claude/kb/pitfalls.md` 正文里把 E996 与 D89（臆造的决策）各引两次。两个号在仓里一处都没有。
- 今天：experiment-refs 报「E996 被引用但 experiments/ 下没有它」、decision-refs 报「D89 被引用但 decisions/ 下没有它」，各退 1。这两格扫 kb 全部 .md 加 `.claude/rules/*.md`，一次就算：「raw_matches="$(grep -ohE "(^|[^A-Za-z0-9/-])${letter}[0-9]{1,3}([^A-Za-z0-9-]|\$)" "${scanned_md_files[@]}" 2>"$grep_error_file")" \」（`.claude/gate.d/10-references-and-invariants.sh:110`）。
- 删了之后：doc-lint 的 H 只扫 kb、要出现 ≥3 次「ID_MIN=3」（`.claude/singlefs-ai-sop/scripts/doc-lint.sh:763`），G 只比对有登记位的号；number-name-sync 对没登记的号直接跳过「if num in truth and truth[num] != nm:」（`.claude/singlefs-ai-sop/scripts/number-name-sync.sh:101`）。副本上 doc-lint 与 number-name-sync 的输出里 E996、D89 一次都没出现（M7 末段）。能删的只有「kb 里出现 ≥3 次的悬空号」那一小片重叠；规则文件里的、kb 里一两次的，只有这两格判。
- 顺带：第一次试用的 E999 被 doc-lint H 报了「出现 3 次」，第三次出现在 `.claude/gate.d/fixtures/10-references-and-invariants.sh/red/` 的样本 kb 里——H 把样本目录也当 kb 数，样本的悬空号会替真 kb 凑数（`m7-first/` 那一趟，留在草稿目录，没入库）。

### B9 三格长期红

- batch-scope：盘点那一趟在工作区上跑，这一格文件头自己写了「# ⚠️ 几个会话共写一个仓时，不带 --staged 跑它会把**别的会话未提交的改动**也算进这一批，报出一堆不是你的触发文件。」（`.claude/gate.d/11-review-and-sync-records.sh:60`）。我按提交时暂存树的判据现算（M8）：暂存 2277 条路径里触发文件 188 个，151 个没登记，登记了而不在这一批的 25 条——提交时照样红。原因是这一批本身太大（推的：第一轮把 79 道合成 23 道，门禁目录整片是触发文件），而登记表只认逐个路径，没有「这一批整片改了 `.claude/gate.d/`，理由是……」这种成批写法。判据对；缺的是成批写法。不补，大家会拿暂存清单机械生成登记表，这一格就空转了。删掉它丢的是成本信号（顺手修的脚本把同步记录撑大），不丢正确性：knowledge-sync 照样要求同步记录点名每个触发文件。
- knowledge-sync：判据对（改了规则、agent、钩子之后要回扫旧说法），红是流程欠账：合并那一批的同步记录没写。删了丢的是正确性——规则改了而别处还说旧话，没人查。
- archive-past-rounds：删「在 HEAD 里有、且这一轮没碰过」的 research/results 与 research/prompts 文件（`research/scripts/archive-past-rounds.py:80`），保留的只有判决与登记表两类「KEEP = re.compile(r"-main-verification\.md$|abandoned-rounds\.tsv$")」（`research/scripts/archive-past-rounds.py:36`）与被代码当输入的。「这一轮」按全仓的基准算，不按会话算：会话甲上一次提交带进去、它的轮还在跑（腿还在读背景材料）的文件，会话乙提交时跑 --apply 就删掉。多会话同写一个仓时，这张红单谁也不敢照着删，1816 份就是这么攒出来的（推的：我没实跑 --apply）。判据本身在多会话下不合用，要先加「还在跑的轮」的保护（例：有 `_<轮>-body.md` 而没有 `<轮>-main-verification.md`、也没登记撂下的轮，它的文件不删），再谈欠账。

## 二、doc-kb 37 格：一格红把别的格的出路淹掉

五道（10、20、30、32、43）并成一道之后，「一格红」在三处把别的格淹掉。

### 1. 判别力样本：exit=1 与绿样本在并起来之后不再分辨格（量过，M1）

照 stage-selftest.sh 的办法，把五道里每一道的每一份样本（36 份、328 条 want）拷进临时目录、跑 setup.sh，在同一棵树上跑另外四道（`outputs/m1-rows.tsv`、`outputs/m1-per-sample.tsv`）：

| 量 | 数 |
|---|---|
| 红样本在别的四道里至少一道也退 1 | 28 / 28 |
| 绿样本在别的道上判红（至少三道退 1） | 8 / 8 |
| 某一道的 want 由别的道替它打出来 | 5 / 328 条，落在 4 份样本上 |

替打的五条：10 号 red-unreadable 的路径 `.claude/kb/dangling-link.md`（20、30、32 号也打）、32 号 red-field-refs 的「已定项 7」、red-first-txn-hooks 的「未定项 1」「未定项 2」（20 号也打）、43 号 green 的「本次未跑」（20 号也打）。

含义：并成 doc-kb 之后，stage-selftest 只能喂整道（47 号文件头写着「# 每份样本八格全跑（stage-selftest.sh 只给项目根与 --force，给不了 --check）：绿样本里另外七格要么无对象可判、要么判绿，」，`.claude/gate.d/47-code-tooling-selftests-and-registries.sh:45`），每份红样本的 `exit=1` 由别的格白送，判别力只剩 want；而 want 里有 2–5 个字的短句（「自指」「挂错节」「未定项 1」），别的格一打就替它通过。用弄坏开关（`<道>_BREAK=<格名>`）证明「这一格坏了样本会判错」时，这 5 条在 doc-kb 里证不出来。八份绿样本全要重做成一棵「37 格都绿或都退 77」的树。

### 2. 分诊与派修按道算，不按格算（读码 + 五道实跑）

- 五道在今天的工作区上各跑一次（`outputs/realrun-*.log`）：10 号 204 行退 1、20 号 113 行退 1、30 号 7 行退 0、32 号 40 行退 0、43 号 24 行退 1，并起来 388 行。红的四格是 governance-refs（29 行 ✗）、invariant-anchors、item-ref-status（133 处）、paid-cited-tests，都不是哪一个书记员这一次写出来的。
- 场景：书记员这一次写错一处决策状态，item-ref-status 之外又红一格（比如 cross-decision-status）。整轮门禁里 doc-kb 这一道本来就红，多一格红不改变道的判定；分诊员的交付是「一张表：阶段 / 绿·红·77 / 归属（这一轮 · 不是这一轮 · 环境 · 并发 · 分不清）/ 门禁原样的 ✗ 与 → ；然后未实现清单原样。」（`.claude/agents/gate-triage.md:38`），按道一行，一行只有一个归属。同一道里四格「不是这一轮」、一格「这一轮」，这一行怎么写都丢一半。今天 20 号 13 格已经是这样，doc-kb 把它放大到 37 格，而且把欠账最多的几格（item-ref-status、governance-refs、paid-cited-tests）与书记员最常碰的几格放进同一行。
- 派修按 stage-owners.tsv 第二列、一道一格。doc-kb 的五道原来的归属合起来是 kb-scribe、implementation-writer、experiment-runner、prior-art、gate-triage；row27-preconditions 那一格原来归「89-closeout-row27-preconditions.sh	implementation-writer」（`.claude/gate.d/stage-owners.tsv:71`），修它要写 `crates/` 的测试，而书记员的写范围只有「kb-scribe	.claude/kb/**」（`.claude/hooks/agent-write-scope.tsv:9`）。doc-kb 红了先派给谁，这一列答不出按格的答案；派错了，写范围闸当场拒，再回主 agent 转派。
- 汇总行的出路是通用的一句「按上面那一格自己的出路改；改完只重跑那一格」（`.claude/gate.d/43-checks-owed-and-closeout.sh:902`，20 号同形），指到格名，不指到规则。388 行里 → 行只有 9 行（10 号 3、20 号 4、43 号 2），10 号 governance-refs 的 29 行 ✗ 共用两行 →，第三行是整道的通用出路。分诊员照定义抄「✗ 与紧跟的 →」，这一格抄出来是 29 行 ✗ 配两句出路。

### 3. `--check <格名>` 够不够

格名在五道之间不撞（37 个名字各不相同）。够用来单跑一格；不够的是上面两件：样本不给 `--check`，分诊与派修不按格。要拆回来的判据我给一条：同一道里的格，修它们的是不是同一个 agent、同一个写范围——row27-preconditions、format-const-placeholders（要改 crates 常量文件）、citations（要调研员查外部源码树）三格不满足。

## 三、去编号之后的次序与改名（第 4 问）

共享 gate.sh 按文件名排序跑本地阶段：「< <(find "$GATE_D" -maxdepth 1 -name '*.sh' \( -type f -o -type l \) | sort)」（`.claude/singlefs-ai-sop/scripts/gate.sh:636`）。

### 1. 77 号 test-environment（B6）

- 它依赖次序的只有一半。临时目录那一类用祖先 gate.sh 的开跑时刻当分界，晚于它的条目不判红「if produced_after and latest_modification_epoch >= produced_after:」（`research/scripts/test-environment-check.py:328`），所以这一类放在哪儿判得都一样：这一趟留下的，下一趟门禁判。与时刻无关的几类（残留 loop / dm 设备、挂载、带 singlefs 的 qemu 进程、宿主盘只读、剩余空间、ext4 错误计数、内核日志）判的是 77 号开跑那一刻的机器，排在它后面的阶段干的事这一趟看不见。
- 按 `<类>-<内容>.sh` 排（M4，名字取方案 A 表里的，checker-tier 那几道按「checker-tier-<原名去编号>」补）：77 号叫 `checker-tier-test-environment.sh` 时排第 7 / 15，后面还有 code-research-build、code-source-discipline、code-tooling、doc-experiments、doc-kb、doc-process-records、doc-text、harness-tests 八道。其中 code-research-build（research 的 `cargo test --release`）与 harness-tests（74 号的 `cargo test --release`）是整轮里两次最大的编译；`--staged` 的临时 worktree 里构建从零开始（`.claude/singlefs-ai-sop/scripts/gate.sh:55`），宿主盘剩余空间这一类在它们之前就判完了。残留设备那几类，排在后面的这八道今天不建 loop、dm、挂载，也不起 qemu（推的：按脚本 grep，没逐条跑）。
- 反过来，今天的 87 号（replay）排在 77 号后面，已经违反「排在最后」；改名成 `checker-tier-replay.sh` 之后它排到 77 号前面，这一处是改名修好的。
- 起名的坑：要让它排最后而又守类名，只能放进排在最后的 harness 类；而 `harness-test-environment.sh` 排第 14、在 `harness-tests.sh` 之前（`-` 小于 `s`，两种 locale 一样）。不靠字母序的办法是让与时刻无关的几类也按「这一趟之前就有」判：qemu 进程有启动时刻、loop 背后的文件有修改时刻，可以沿用临时目录那一类的分界；剩余空间这一类本来就是快照，写明「判的是开跑时的盘」即可。这样 77 号放在哪儿都一样（推的，没实现）。

### 2. 20 号 --write 与 30 号 --write

- 20 号的 --write 只重新生成 decisions.md 的分项清单与索引表「状态」列，30 号 status-sync 的 --write「# --write 只重写这一行（缺了就在标题下补上），不碰历史条目；节或索引行缺了，--write 补不了，照样判红。」（`.claude/gate.d/30-decision-history-entries.sh:60`），写的是从索引表抄到变更史节顶的现状行。修红时反过来跑，30 号抄进去的是旧状态；但下一次判定 status-sync 照样红，不会漏判，只多修一轮。并进 doc-kb 以后次序由格表定，要是 doc-kb 另给一个「全部 --write」的入口，格表里 decision-items-sync 要排在 status-sync 前面。没打中。

### 3. 按文件名记下的跨趟状态

- 55、57、59、74、87 号的「能不能复用上一次整轮全绿」按文件名查 stage-inputs.tsv，查不到「return EXIT_GATE_MUST_RUN, f"{REGISTRATION_TABLE} 里没有 {stage} 这一行，按要跑处理"」（`research/scripts/admission.py:1149`），阶段文件自己也进比对（`research/scripts/admission.py:1153`）；全绿标记名里带阶段文件名。改名之后第一趟全部重跑、旧标记作废，方向是安全的，代价是一趟重阶段。没打中。

### 4. 认「两位数开头」的两处

- 47 号 research-script-selftests 认「有阶段在跑它」用「for stage in sorted(glob.glob(".claude/gate.d/[0-9][0-9]-*.sh")):」（`.claude/gate.d/47-code-tooling-selftests-and-registries.sh:244`），fixture-claims 认阶段用「stages = sorted(glob.glob(os.path.join(STAGE_DIR, "[0-9][0-9]-*.sh")))」（`.claude/gate.d/47-code-tooling-selftests-and-registries.sh:1149`）。全部去编号时，前者把每份声称有自证的脚本都报成没人跑、后者报「一个 NN-*.sh 都没有」，都是响亮的红。危险的是只改一部分（例：54、59 号在里程碑三重设计，先留着旧名）：fixture-claims 只看还带编号的那几道，改了名的阶段头部声称有样本而目录不在，它一声不吭（推的，没造）。

### 5. 按全名指门禁，从此没人判（量过，M5）

- governance-refs 认门禁指向只有两种：「两位数加号」「GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.\-–])((?:[0-9]{2}\s*[、/]\s*)*[0-9]{2})\s*号")」（`.claude/gate.d/lib-governance-refs.py:36`）与反引号里带 `/` 的路径。M5 在临时仓的 CLAUDE.md 里写三行指向同一道不存在的门禁：「门禁 98 号」判红、「`.claude/gate.d/doc-decisions.sh`」判红、「doc-decisions 那一道的 decision-items-sync 格」没判红，也没进「没判的」清单。用户定「以后记录门禁按照全名记录」，按全名写的指向正是第三种；改名之后第一种一处都不会再有，这一格对门禁指向就只剩写全路径的那一种。格名（「xx 那一格」）今天也没人判。
- 违规输入：`.claude/agents/kb-scribe.md` 里写「写完跑 doc-kb 的 decision-items-sync 格」，之后 doc-kb 拆回两道、格改名，这句悬空，门禁全绿。

### 6. locale（没打中）

lib.sh 先试 C.UTF-8、C.utf8，都没有才用 en_US.UTF-8（`.claude/singlefs-ai-sop/scripts/lib.sh:12`）。M4 把方案里的名字在两种 locale 下各排一遍，次序相同。

## 四、「只在提交时跑」的代价：该留在写入那一刻就拦的（第 6 问）

### 先说今天写入那一刻还剩什么

- 第一轮删了书记员的写入后钩子：HEAD 里 settings.json 的 PostToolUse 在 `Write|Edit|Bash` 上注册 `kb-scribe-followup.sh`，它按 `kb-scribe-followups.tsv` 每次写 kb 跑 31 道快阶段、把 ✗ 与 → 交回书记员；这两个文件在暂存区是 D（`git status --short .claude/hooks/`、`git show HEAD:.claude/hooks/kb-scribe-followups.tsv`）。方案 C 节的改动清单里没有这一条，只在 kb-scribe.md 的 diff 里少了一行。
- 现在书记员写入时照跑的只剩两样：「4. 门禁阶段交回前不跑，由提交前 `gate-triage` 跑的 `research/scripts/gate-staged.sh`（它起的整轮 `gate.sh`）判」之后那半句「第 2、3 步里的 `--check shape` 与 `--write` 是写回流程的一步，照跑。」（`.claude/agents/kb-scribe.md:35`）。写入那一刻当场拒的，只有 write-guard 的撇号角标与钟点两道。

### 判据：哪一类不能等到提交

1. 提交时那一格根本不跑；
2. 修它要的信息只在写的那一刻有（/tmp 里的原件、改这一句的理由），到提交时已经没了；
3. 提交之前错的东西已经被下游抄走——write-guard 第三道立的理由就是这一种：「为什么：门禁 12 号只在收尾跑整轮门禁时判，2026-09-24 实验设计员照样把这类名字写进三份重跑登记、执行员又抄进源码。」（`.claude/hooks/write-guard.sh:16`）；
4. 几个会话共用的编号撞了：提交时才发现，两边都已经把号抄进了代码注释、测试名、记录与冻结的三方证据，冻结证据改不得，撞号永久留下。

### 清单

| # | 该在写入时拦的 | 今天谁判、什么时候 | 为什么不能等（上面第几条） | 拦法（都是推的，没实现） |
|---|---|---|---|---|
| W1 | kb 与这一轮新写的提示里拿 `/tmp` 路径当依据 | 34 号 evidence-in-repo 判据二；但在 gate-staged.sh 的链接 worktree 里整格先退 77（量过，M3：同一份红样本，样本仓上退 1、照 `gate.sh --staged` 建的 worktree 上退 77，而那一行 `/tmp` 依据就在 worktree 里） | 1、2：提交时不判；/tmp 的原件会被 77 号的 clean、重启或草稿目录清理删掉，事后补不回来 | write-guard 加一道：写 `.claude/kb/**.md` 与 `research/prompts/` 下新文件时，写进去的内容命中判据二那个形状就拒；形状只写一份，门禁与钩子共用（照 lib-forbidden-notations.py 的办法） |
| W2 | 新立的欠账号、变更史「其 N」撞号 | 共享 doc-lint F、「历史条目编号」，提交时 | 4：几个会话写的是同一个工作区里的同一份文件，写入那一刻就看得见别人刚登记的号；等到提交，号已经散出去了 | Write/Edit 往登记表里新加一行 `\| C<n> \|` 或 `（其 N）` 时，同一份文件里已有同号就拒；实验号已经有 `claim-experiment.sh` 排他占号 |
| W3 | 决策正文改了没留变更史条目 | 30 号 entry-added，提交时 | 2：条目里写的是为什么改，只有写的人知道；提交时红了派给别人补，补出来的理由是重建的 | PreToolUse 拦不了（正文与条目是两次写，先写哪次都会被拦）；要么恢复写入后报告（删掉的那个钩子的形态，只报 entry-added 这一格），要么在「交回前不跑门禁」里给书记员开 `--check entry-added` 这一格的口子 |
| W4 | 装置或变异表改了而 research/results/ 里没有不比它旧的产物 | 34 号 evidence-in-repo 判据一；同 W1，提交时退 77 | 1：提交时不判；补它要重跑实验，执行员交回时最便宜 | 不是写入钩子能判的（要看两份文件的新旧与指纹）；执行员交回前跑这一格，或整轮门禁另在工作区上跑它一次 |

W1、W4 的根子是同一件：agent-common.md 第 99 行还写着「门禁 69 号判这一条的形式：装置或变异表改了而 `research/results/` 里没有一份不比它旧的产物、kb 与这一轮新写的提示里把 `/tmp` 路径当依据引用，都红。」（`.claude/agent-common.md:99`），而「扫仓的门禁只在提交前由 gate-staged.sh 跑」之后，这一格在默认的提交路径上一次都不判；那一格自己的出路写的是「不带 --staged 在工作区里跑一遍 bash .claude/scripts/gate.sh 才判得了这一格。」（`.claude/gate.d/34-doc-experiment-pages-and-products.sh:528`），分诊员只在主 agent 明写全量时才这样跑。

### 不必挪到写入时的

- 撇号角标、钟点：已经在 write-guard 里。
- 欠账表行形状、分项引用状态、状态说两遍、决策分项清单同步、现状行同步、字段表合计、doc-lint 的 G / H：提交时红了，照门禁给的出路机械地改得回来（清单同步那两格还有 `--write`），不丢信息，也不散到下游。

## 五、打中之后的四句

「臂」取方案在那一条上给的候选；方案只给一个做法的，另一臂是「不做」。

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到判别它的东西 | 满足的是判据字面哪一分句 | 方案给的改法在打中的那几格上还中不中 |
|---|---|---|---|---|
| B1 | 分辨：留这一格判红，删了没人判 | 看得到：门禁跑时读得到 `instance_table.rs` | 第 3 问「删了或改了之后，什么违规会从此没人判？举一个具体的输入」 | 方案只给「里程碑二提交之后删」；在这一格上照样中。先把那几笔的去向挪到里程碑三的收口表或一条会红的用例上，才不中 |
| B2 | 分辨：只删自证调用不中，删到「只留表与定义双向一致」中 | 看得到：settings.json 在门禁跑时就在 | 第 3 问同一句 | 方案原话那个做法中；窄的那个做法不中 |
| B4 | 不分辨：留 74 号与靠「构建与单测」两臂一起落空（今天 74 号红、check.sh 不跑那六条） | 看得到：`#[ignore]` 与两条命令行都在仓里 | 第 3 问「B4、B8、B9 给出判断与依据」 | 两个改法都中。病根是两臂共用的前提「那批用例有人跑」，另立一笔：让 74 号带 `--include-ignored`（或 `-- --ignored --exact <六条>`），再判留不留 74 号 |
| B5 | 分辨：按写法列举的认法中，去注释与出路文字的认法不中 | 看得到：认法读的就是脚本全文 | 第 3 问同一句 | 方案原话「只认自己实现了入口」两种实现都算；列写法那种中 |
| B8 | 分辨：删了中，留着不中 | 看得到 | 第 3 问「B4、B8、B9 给出判断与依据」 | 方案没给改法（待腿判）；判断是不能删 |
| 第 2 问 doc-kb | 部分分辨：分诊一行、派修一格这两件，doc-kb 比拆开的五道重；样本 exit=1 白送这件，第一轮已经并成的 20 号（13 格）今天就有，doc-kb 只是放大 | 看得到：格名、归属表、写范围表都在仓里 | 第 2 问「出路还能不能指到那一格、那条规则？…有没有该拆回来的」 | 方案没给拆的判据；我给的「修它的是不是同一个 agent、同一个写范围」在 row27-preconditions、format-const-placeholders、citations 三格上要拆 |
| 第 4 问 全名指门禁 | 分辨：留编号时有人判，去编号后没人判 | 看得到：治理文档就是文本 | 第 4 问之外、属于「改名之后从此没人判」；照派发给我的攻击面（专找合了、砍了之后没人判）记 | 方案没给；要 governance-refs 认门禁目录里的文件名（去掉 .sh）与各道格名 |
| 第 6 问 evidence-in-repo | 分辨：只跑 gate-staged.sh 时不判，另在工作区跑时判 | 判据一在 worktree 里确实看不到（修改时刻是检出那一刻）；判据二不需要修改时刻，是整格一起退 77 连累的 | 第 6 问「有没有该留在写入那一刻就拦的」 | 方案没给；W1、W4 |

## 六、我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| research-script-selftests 的「声称有」改成：只认 .sh / .py，去掉 # 注释行与出路文字后代码里还有 `--selftest`（M2 的 R3） | B5 | 量过（M2：今天 53 份认 50 份、三份误认去掉；五份新写法全认、两份只提名的都不认，`outputs/m2.txt`） |
| sweep-term.py 的 PRODUCT_FILE_NAME_PATTERN 前面加 `(?<![A-Za-z0-9_.-])` | B7 term-renames | 量过（M6：research/results/ 逐份比，换出的文本与换下的原串都相同；时间见第七节） |
| research-script-selftests 的 runner 表按耗时从长到短分批并行，每条写自己的退出码文件、数份数 | B7 research-script-selftests | 量过（草稿那一次 400.9 秒到 178 秒、M9 那一次 401.7 秒到 137 秒，两次 45 条都退 0）；只两次观测，不算稳定 |
| 74 号的 cargo test 带 `--include-ignored` | B4 | 推的（没跑：那六条是 harness 耗时用例，单条 debug 下 60 秒以上） |
| 77 号把与时刻无关的几类也按「开跑之前就有」分界（qemu 进程的启动时刻、loop 背后文件的修改时刻），剩余空间写明是开跑时的快照 | B6 | 推的 |
| evidence-in-repo 的判据二不跟着链接 worktree 一起退 77；判据一改由执行员交回前跑，或整轮另在工作区跑一次 | 第 6 问 W1、W4 | 推的 |
| write-guard 加 /tmp 依据形状与登记表撞号两道 | 第 6 问 W1、W2 | 推的 |
| governance-refs 认门禁全名与格名 | 第 4 问 5 | 推的 |
| archive-past-rounds 保护还在跑的轮（有正文、没判决、没登记撂下的） | B9 | 推的 |
| batch-scope 认一种成批登记（目录前缀加一条理由） | B9 | 推的 |

## 七、B7 的量

### term-renames（M6、M6b）

| 量 | 数 | 出处 |
|---|---|---|
| 照 `sweep-term.py --check` 走一遍：扫的文件 | 2148 份（跳过 target、.git，豁免表里的不扫） | `outputs/m6b-sweep-term-phase-totals.txt` 末行 |
| 其中三段合计：读文件 / 掩码 / 改名检测 | 0.2 秒 / 1029.3 秒 / 9.9 秒（总 1039.4 秒） | 同上 |
| research/results/ 299 份上原样掩码的合计 | 1041.0 秒；超过 1 秒的 20 份，每份都含一段 65536 字的 `[A-Za-z0-9_.-]` 连续字符 | `outputs/m6-all-results.tsv` |
| 最慢的一份 | 82.36 秒（e142-new-pool-file-creation-dry-run-2026-09-25-q142-1v-combined.out，2059438 字） | 同上 |
| 同一批加左边界之后的合计 | 1.76 秒 | 同上 |
| 两种掩码换出来的文本与换下的原串 | 299 份逐份相同 | 同上 |

两次跑都与别的活同时跑、nice 19，挂钟偏大；比例不受影响。改名检测正则本身不慢。

### research-script-selftests（M9，另有草稿那一次）

| 量 | 草稿那一次 | M9 |
|---|---|---|
| 45 条顺序跑合计 | 400.9 秒 | 401.7 秒 |
| 最慢两条 | layer0-shard-run.sh 150.3、agent-watch.py 121.6 | layer0-shard-run.sh 145.7、agent-watch.py 121.3 |
| 退出码非 0 | 0 | 0 |
| 按耗时分批、每批 8 条并行的挂钟 | 178 秒，45 条退 0 | 137 秒，45 条退 0 |

出处：`outputs/runner-times-first.tsv`、`outputs/parallel-first.txt`、`outputs/m9-sequential.tsv`、`outputs/m9-parallel.tsv`、`outputs/m9-summary.txt`。

## 没打中的形状

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| B2 只删八个钩子的自证调用：找一个 47 号跑得到而 hooks-registered 跑不到的自证 | 八个项目钩子；hooks-registered 对全部 11 个钩子里带 `--selftest` 字样的 10 个跑自证 | 没找到：hooks-registered 射程更宽 |
| B3 找一种重复登记 table-shape 红而 doc-lint 绿 | 全角空格、半角空格两种；只在副本上试了全角 | doc-lint 以 G 的「裸引用」红，没漏判 |
| B6 locale 让两台机器排出不同的次序 | 方案里 14 个名字加 77 号的四个候选名，C.UTF-8 与 en_US.UTF-8 | 次序相同 |
| 第 4 问 20 号 --write 与 30 号 --write 写反 | 读码 | 下一次判定照样红，只多一轮 |
| 第 4 问 改名让跨趟复用漏跑重阶段 | admission.py 的 gate_reuse 与全绿标记的键 | 查不到登记行按要跑，旧标记作废；只多跑一趟 |
| B9 batch-scope 的红只是工作区里别的会话的改动 | 按提交时暂存树的判据现算（M8） | 暂存树上照样红，不是跑法造的 |
| 第 2 问 两道并起来格名撞车，`--check` 选错格 | 37 个格名 | 不撞 |

## 这条腿自己的限度

- 74 号只在工作区上实跑了一次（别的会话有未提交的 crates 改动在里面）；HEAD 版 74 号第 48 行是同一条命令、HEAD 版测试文件同样 19 处 `#[ignore]`，HEAD 上没实跑。
- 副本（M7）不带 research/results 与 research/prompts，也没有 git 历史，所以副本上的 doc-lint、number-name-sync 本来就有与注入无关的红；我只看输出里有没有点名注入的那几个编号，没比基线。
- M1 只比「同一棵样本树上别的道打出什么」，没真把五道并成一个脚本；并起来以后格的次序、共用的前置失败会不会再多淹一层，没量。
- B9 里 archive-past-rounds 删掉别的会话在跑的轮的材料，是按代码推的，没实跑 --apply。
- B7 的并行只量了两次（草稿一次、M9 一次），与别的活同时跑，挂钟偏大；没核各条自证之间有没有共用可写状态，只看了退出码。
- 第 6 问的四条拦法都没实现；W2 的「写入时看得见别人刚登记的号」依赖几个会话共用一个工作区，各自开 worktree 时不成立。
- 我跑的是今天的工作区；别的会话在改，同一格重跑可能不同。

## 没做什么

- 不判正推、辩方那几格，不做第 1 问的逐格映射表（本地攻方的面），不核第 5 问的定义 diff。
- 没跑 54、55、57、59、87 号，没跑 gate.sh 整轮与 gate-staged.sh；74 号经阶段自己的内存包装跑（`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=8G`），外面没再包一层。
- 没改仓里除报告与模型目录之外的任何文件，没做 git 写（M3 的 worktree 与 M5、M7 的 git init 都在草稿目录里的样本仓）。
- 没读禁读清单里的文件；背景材料里抄进来的调度记录两节（第 2575 行起）也没读，理由同禁读清单。
- 我工作期间别的会话改过 `.claude/gate.d/20-doc-decision-documents.sh`（与开工快照 `research/prompts/gate-shrink-r1-snapshot/gates-sha256.txt` 对不上）与 `.claude/singlefs-ai-sop/scripts/doc-lint.sh`（行号挪了 23 行，报告里的行号按交回前现查的改过）；M1 与五道实跑用的是跑那一刻的版本。

## 草稿目录

- 删了：`/tmp/claude-1000/gate-shrink-r1-opus/target`（267M，74 号实跑的 CARGO_TARGET_DIR）、`/tmp/claude-1000/gate-shrink-r1-opus/repo-copy`（51M，B2、B3、B8 第一趟用的仓副本）；M7 的仓副本 `m7-copy` 与 M3、M5 的临时样本仓由脚本自己跑完删掉。
- 留着（主 agent 核用，都是日志与小脚本）：`/tmp/claude-1000/gate-shrink-r1-opus/` 下的 `m1/`、`m3/`、`m5/`、`m7/`、`m7-first/`（用 E999 的那一趟，doc-lint H 被样本目录凑够三次）、`m9/`、`prof/`（草稿计时脚本与输出）、`realrun/`、`seg/`（报告分段草稿）。要入库的已拷进模型目录 `outputs/`。
