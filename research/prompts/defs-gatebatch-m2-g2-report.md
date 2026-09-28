# G2 交回：门禁批第二轮判决的余项与实分一接入（tooling-writer）

写于 2026-09-27。规格 `/tmp/claude-1000/gate-batch-m2-g2/spec.md`；途中收到主 agent 四条追加（见「途中收到的追加」一节）。

## 结论

- 规格第 1–9 条都做完，第 9 条（54 号）最后套。途中追加的四件（admission 自证临时仓链 `.claude/singlefs-ai-sop` 与 `.claude/scripts`、仓根模板改放草稿目录、新脚本改 source 垫片并写准入声明、驱动自证默认改假 cargo）也做完。
- 出口各项的原样末行都绿，只有两处红不在这一批：47 号红在 `research/scripts/check-segment-registry.py --selftest`；doc-lint 红在 O1–O3、F23 等编号与两份 kb 文件（这一批改过的文件里 grep 零命中）。
- 仓根 `multi-host.env.example` 不在我的写范围，写闸拒了。成品在 `/tmp/claude-1000/gate-batch-m2-g2/multi-host.env.example`，sha256 `72ae03d4a01a49ccc069dc27c62a03e31d42dc420e52a879fa67bd4c2c458bd4`，与 deliver 那份逐字节相同（没改）；由主 agent 拷进仓根。
- 第 4 条选「把驱动脚本按内容算进输入」，理由见第 4 条那一段。
- 推翻条件：真 `--full` 在分片开着时，驱动脚本在 HEAD + 暂存区的临时树里算出的指纹与 54 号给的对不上（这一批只在假 cargo 与临时仓上证过）；或者别的会话再改 `stage-must-run.sh` 这类 helper 的 source 写法，admission 自证的临时仓又缺文件。

## 改过的文件与 git diff --stat

逐文件相对开工时备份（`backup/`）的增删行数，由 `diff backup/<名> <仓里的文件>` 数 `>` / `<` 得出：admission.py +339 −23，stage-inputs.tsv +10 −7，lib_heavy_tests.py +34 −7，implementation-workflow.md +1 −1，54 号 +46 −4。54 号的数里含 singlefs-39 做的第 63 行替换（source 垫片，+1 −1），那一行不是我改的。新建三份：layer0-shard-run.sh 280 行，layer0-shard-run-selftest.sh 207 行，layer0-shard-configuration-check.sh 98 行，权限 775。八份都与测过的 dev 副本逐字节相同（`cmp` 现核）。合在一起的 diff：`/tmp/claude-1000/gate-batch-m2-g2/g2-changes.diff`（1498 行）。

`git diff --stat -- <这几份>` 原样。它比的是 HEAD，所以混着 G1 与别的会话没提交的改动；admission.py 与三份新脚本没进 git，不在 stat 里：

```
 .claude/gate.d/54-layer0-replay.sh       | 617 +++++++++++++------------
 .claude/gate.d/stage-inputs.tsv          |  45 +-
 .claude/hooks/lib_heavy_tests.py         | 745 +++++++++++++++++++++++++++++--
 .claude/rules/implementation-workflow.md |  16 +-
 4 files changed, 1074 insertions(+), 349 deletions(-)
?? research/scripts/admission.py
?? research/scripts/layer0-shard-configuration-check.sh
?? research/scripts/layer0-shard-run-selftest.sh
?? research/scripts/layer0-shard-run.sh
```

仓里的改动都是定点替换：开发在草稿里的仓副本上做，再由 `apply_hunks.py` 把每个 hunk 交给 `research/scripts/replace-once.py` 套进仓里。每个 hunk 在仓里恰好命中一次，套完回读与 dev 相同。日志在 `logs/repo-apply*-*.log`。三份新脚本用 Write 新建，之后的改动同样走 replace-once。第一轮 hunk 数：admission 35、lib 12、54 号 4、tsv 3、规则 1。第二轮：admission 1、驱动 1、自证 8、配置判法 1。

## 各条改法与证红

### 第 1 条：实分一三份 diff 重打（admission.py、stage-inputs.tsv、lib_heavy_tests.py）
- `patch --dry-run` 在今天的文件上：admission.py 19 个 hunk 有 5 个打不上（文件头第三列、弄坏开关表、自证成功句、线程格之后的 merge 格、main 的用法句），全在草稿副本里照 diff 的意思按新代码重写。lib 的 4 个全套上。tsv 那一份按第 7 条合并着改。
- 按 G1 之后的接口改的地方：merge 格接在 G1 新加的 `started_worker_threads=` 两格之后；54 号分片格里假驱动的参数照旧；`CrashCase` 多一个成员 `is_shardable_across_machines`。
- 原有自证：`admission.py --selftest` 实分一的格全在，现 232 格全过（原样末行见「出口各项」）；`lib_heavy_tests.py --selftest` 141 种全过，其中实分一的 4 格也在。

### 第 2 条：三份新脚本（来源 deliver，照 G1 之后的 admission.py 核过）
- deliver 版调 `crash-case-manifest` 用的是 `--extra-file <54 号> --extra-file <admission.py>`。G1 之后 54 号给的是 `--judging-digest --toolchain --build-environment`，照原样调，两边指纹永远对不上。驱动本机、第二台、跑完三处都改成 G1 的参数，自证里算指纹那一处也跟着改。
- deliver 版调 `crash-case-record` 用 `--threads-text`，G1 已经删掉这个选项，改成 `--machine-cores "$local_cores" --threads "$local_threads" --threads-origin "$local_threads_origin"`。
- 追加指示之后：三份都改成 `source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"`（垫片）。驱动的 `--selftest` 分支挪到 preflight 那一行之后，同时删掉 `run-condition: check …configuration-check.sh`：preflight 的 check 读不到参数，留着它的话，没有配置时 `--selftest` 会被拒 78。配置由驱动开跑后第一步的配置判法拒，退 1，原因与出路照旧，自证 ⑤ 相应改成期望退 1。
- 三份相对 deliver 的差：`driver-vs-deliver.diff`（70 行）、`selftest-vs-deliver.diff`（175 行）、`check-vs-deliver.diff`（14 行）。

### 第 3 条：`crash-case-shardable` 进 `CRASH_CASE_JUDGING_SUBCOMMANDS`
- 改法：admission.py 第 208–209 行把它加进元组。判法摘要（第 1894–1895 行）从分派表取这几项；弄坏开关 `shardable-outside-judging-digest` 把它摘掉。
- 该红的输入：把 `command_crash_case_shardable` 里「没登记」那一支的 `return 1` 改成 `return 0`，看判法摘要变不变（`probe_g2.py`）。
- 改前（G1 代码加实分一 hunk）→ 改后，原样：
```
改前	第3条	command_crash_case_shardable 把没登记答成登记了（源码里命中 1 处）⇒ 判法摘要不变
改后	第3条	command_crash_case_shardable 把没登记答成登记了（源码里命中 1 处）⇒ 判法摘要变
```
- 自证：`JUDGING_DIGEST_VARIANTS` 加一格（第 3416 行 `SHARDABLE_ANSWER_UNREGISTERED`）。开关格在第 3471 行。在草稿副本上用 `ADMISSION_BREAK=shardable-outside-judging-digest` 跑：
```
  ✗ 判法摘要：单机跑还是双机分片：command_crash_case_shardable 把「没登记 shard=」也答成登记了 ⇒ 摘要变：摘要没变
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

### 第 4 条：双机驱动进两条层 0 流用例的指纹（选「按内容算进输入」）
- 为什么不走 `crash-case-command`：驱动决定判绿的不只是起 cargo 那一条命令。两片的退出码与 `LAYER0_SHARD mode=run` 行、账本拷回与数份、merge 那一趟的环境、merge 行在不在，都是它判的。改成经 `crash-case-command` 起，进摘要的只有命令，这些判法照样在指纹之外；`crash-case-command` 还得另收第二台的进度目录与线程数。按内容算进去，整份驱动都在指纹里。驱动 eval 配置判法打出来的赋值，所以配置判法 `layer0-shard-configuration-check.sh` 也一起进。
- 改法：admission.py 第 187 行 `SHARD_DRIVER_FILES`；第 1551 行 `shard_driver_lines`（登记了 `shard=across-machines` 的，这两份按内容排在登记行之前，不在的记「找不到 <路径>」）；第 1547 行接进清单。stage-inputs.tsv 第 28 行 54 号那一行的路径加上这两份：只改驱动时，快档不会在「范围那一问」退 77，照样核标记。代价：分片关着、照单机跑时驱动也在指纹里，改它这两条会多跑一趟（文件头「管不到的」写明，接受）。
- 该红的输入：临时 git 仓里放一条 `shard=across-machines` 的用例，先放进驱动脚本，再改它，看指纹。原样：
```
改前	第4条	登记了 shard=across-machines 的用例：放进驱动脚本 ⇒ 指纹不变；改驱动脚本 ⇒ 指纹不变（7ee48c0c812a → 7ee48c0c812a）
改后	第4条	登记了 shard=across-machines 的用例：放进驱动脚本 ⇒ 指纹变；改驱动脚本 ⇒ 指纹变（03e9308a66e2 → 3f3cc1939518）
```
- 自证：`run_layer0_stage_shard_cells` 开头两格（第 3824 行起）。`ADMISSION_BREAK=shard-driver-outside-manifest` 下：
```
  ✗ 输入清单：放进双机分片的驱动脚本与配置判法 ⇒ 登记了 shard=across-machines 的 stream-a 指纹变、没登记的 stream-b 不变：stream-a 6fd93cbbda19bb0d → 6fd93cbbda19bb0d，stream-b 96ad622302455cf8 → 96ad622302455cf8
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

### 第 5 条：`strace -E VAR=VAL` / `--env=VAR=VAL`
- `strace -h` 现跑，第 15 行原样 `  -E VAR=VAL, --env=VAR=VAL`，第 17 行 `  -E VAR, --env=VAR`（只清变量）。
- 改法：lib_heavy_tests.py 第 606 行 `LAUNCHER_ENVIRONMENT_OPTIONS` 加 `"strace": {"-E", "--env"}`；第 597 行 strace 带值的选项加 `--env`；第 651 行接弄坏开关 `strace-drops-env`。`-E VAR` 不带 `=` 的不带进里面那条命令（沿用原有的 `"=" in value` 判法）。文件头第 15、40、54 行跟着改。
- 改前（仓里原来那份 lib）→ 改后，`probe_strace.py` 在 lib 自己的样本工作区里判，原样：
```
改前	第5条	strace -f -E <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改前	第5条	strace --env=<runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改前	第5条	strace --env <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改前	第5条	strace -fE <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ 不重型，放行
改后	第5条	strace -f -E <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace --env=<runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace --env <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
改后	第5条	strace -fE <runner> cargo test -p singlefs-harness --test <登记的崩溃枚举目标>（不带 --ignored）⇒ crash-case-cargo
```
  改前 `--env <值>` 分开写那一种碰巧判成重型：`--env` 不在表里，被当成不带值的选项，赋值那个词被当成要起的命令剥过去。这是巧合，不是判对了。
- 自证：第 967 行起加 5 格（最后一格 `-E VAR` 只清变量，判不重型）。`LIB_HEAVY_TESTS_BREAK=strace-drops-env` 下，lib 自检与 heavy-test-guard.sh 自检都红：
```
  ✗ lib_heavy_tests 自检：strace -E 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace --env= 设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace --env 值另起一个词、设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ lib_heavy_tests 自检：strace -fE 合写设 runner、点名崩溃枚举用例不带 --ignored 应当是 crash-case-cargo，实际 None
  ✗ 自检：lib_heavy_tests.py 的自检 应当是 0，实际 1
```
  heavy-test-guard.sh 本身没改。

### 第 6 条：`crash-case-command` 起用例时清掉调用方的 `SINGLEFS_LAYER0_SHARD`
- 改法：admission.py 第 1856–1857 行，`env -u SINGLEFS_LAYER0_SHARD` 放在第一个 `NAME=VALUE` 之前。第一版写在 `SINGLEFS_LAYER0_START_OVER=1` 之后，54 号 `--start-over` 那一格当场红了：env 把后面的 `-u` 当成要起的命令。
- 该红的输入：调用方设着 `SINGLEFS_LAYER0_SHARD=0/2`，照交出的命令起一趟，把 cargo 往后换成打那个变量的 sh。原样：
```
改前	第6条	调用方设着 SINGLEFS_LAYER0_SHARD=0/2，照 crash-case-command 的命令起的用例看到「0/2」（命令前段：env -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… …）
改后	第6条	调用方设着 SINGLEFS_LAYER0_SHARD=0/2，照 crash-case-command 的命令起的用例看到「unset」（命令前段：env -u SINGLEFS_LAYER0_SHARD -u SINGLEFS_LAYER0_START_OVER SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… …）
改后	第6条	带 --start-over 时起的用例看到「unset/1」、退 0
```
  （命令前段里的临时路径我换成了 `…`；整行原文在 `logs/probe-before.log` 与 `logs/probe-after.log`。）
- 自证：第 3804、3813 行两个函数。`ADMISSION_BREAK=keep-caller-shard-switch` 下：
```
  ✗ crash-case-command：调用方环境里设着 SINGLEFS_LAYER0_SHARD=0/2 ⇒ 起的用例看不到它：用例看到的是「0/2」
  ✗ admission.py 自证没过：1 格判错（共 232 格）
```

### 第 7 条：stage-inputs.tsv
- 第 16–20 行（原第 16–18 行）：「再加判它的 54 号、准入模块 admission.py」改成现状：进指纹的是准入模块里崩溃枚举用例的判法摘要；登记了 `shard=across-machines` 的另按内容加驱动与配置判法；54 号不进。第三列的说明加上 `shard=across-machines`。
- 第 28 行（54 号那一行）：路径加 `research/scripts/layer0-shard-run.sh research/scripts/layer0-shard-configuration-check.sh`（第 4 条）；注释「（它也进每条用例的指纹）」改成「进每条用例指纹的是它里面崩溃枚举用例的判法摘要，不是整份」，另加一句驱动与配置判法。
- 第 36、37 行：实分一的 `shard=across-machines` 与注释。
- 第 39 行 c561：第三列与注释逐字照实审 B3b 报告第 116 行起那一段（`exhaustive=C561_SIGMA_FULL threads=C561_SIGMA_FULL`）。我核过用例确实打这两行：`crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs` 第 656 行打计数行，第 675 行打 `LAYER0_PARALLEL_FINISHED` 同形行。
- 第 40 行新加 `crash-case:crash-injection-fast-tier`：原文是实审 B3c-1 报告第 85 行整行抽出来追加的（`awk 'NR==85'`），追加之后与抽出来的那行 `cmp` 相同，4 列。用例在 `second_transaction_supplement_three_crash_injection.rs` 第 131 行标 `#[ignore]`，第 132 行是用例函数。
- 该红的输入：拿改前 / 改后的表判合成日志（`probe_g2.py`）。原样：
```
改前	第7条	crash-case:c561-sigma-full：计数行 exhaustive=false ⇒ 判绿
改前	第7条	crash-case:c561-sigma-full：1 个线程跑了 64 片（本机 32 核、没显式设） ⇒ 判绿
改前	第7条	crash-case:c561-sigma-full：两行都对 ⇒ 判绿
改前	第7条	crash-case:crash-injection-fast-tier：计数行打了两行 ⇒ 登记表里读不出这一条（.claude/gate.d/stage-inputs.tsv 里崩溃枚举用例 crash-case:crash-injection-fast-tier 要恰好一行，实际 0 行）
改后	第7条	crash-case:c561-sigma-full：计数行 exhaustive=false ⇒ 判红
改后	第7条	crash-case:c561-sigma-full：1 个线程跑了 64 片（本机 32 核、没显式设） ⇒ 判红
改后	第7条	crash-case:c561-sigma-full：两行都对 ⇒ 判绿
改后	第7条	crash-case:crash-injection-fast-tier：计数行打了两行 ⇒ 判红
```
- 自证：admission.py 第 3783 行 `run_real_crash_case_row_cells`，拿真仓登记表的这两行判 5 格。登记表是数据，没配弄坏开关；红的样子就是上面「改前」那几行，谁把这两行改回去，这 5 格就红。

### 第 8 条：`.claude/rules/implementation-workflow.md`（第二种活，改的规则）
- 第 58 行「崩溃枚举用例」一条末尾加一句：「双机分片的驱动脚本 `research/scripts/layer0-shard-run.sh` 带什么参数都算（`--merged-log` 也算），参数里有 `--selftest` 的不算。」这与 lib 第 706 行的判法一致：`return None if "--selftest" in arguments else heavy_test("crash-case-cargo", …)`。
- 同一行 runner 那一句「`systemd-run -E` / `--setenv`」后面加「、`strace -E` / `--env`」，跟第 5 条的闸对齐（规格没点名这一处，但规则要与闸逐类相同）。
- 改过的定义、共用约束与规则只有这一份，交主 agent 开定义三方（门禁 72 号）。

### 第 9 条：54 号（最后套）
- 动手前读了一遍今天的样子，与备份相同（singlefs-39 的准入声明与 preflight 调用已在）。实分一 4 个 hunk 有 2 个打不上（文件头、循环那一段），照 G1 的 `read_crash_case_command` / `run_crash_case <日志> <命令…>` 重写。
- 改后字面：第 25–29 行文件头的双机分片一段；第 237–244 行 `run_crash_case_in_two_shards`；第 350–358 行判开不开（配置判法判得过才开，打一行开或关）；第 394–418 行逐条用例：先 `crash-case-shardable` 定走哪一路，分片的 `case_threads_note` 写明第二台取它自己的核数，退非 0 时分片那一支有自己的 ✗ 与出路。merge 那一趟的日志照单机的判法判、写同一格（线程逐片判在 admission.py 第 1608、1629 行）。
- gate-lint 在第 413 行报过一次红：出路句里写了「✗」字，被当成下一处拒绝。改成「驱动脚本输出里判红的那一句」，重跑转绿（见「出口各项」）。
- 自证：admission.py 里 54 号那几格与实分一的两格分片格（第 3824 行起）全过，含「双机分片：关」「开：stream-a 交给驱动 --merged-log、另两条单机跑」「驱动退非 0 判红、不写标记」。

### 途中收到的追加
1. 之后，在做第 1–7 条时收到：admission 自证在 singlefs-39 补声明之后有 replay.sh 两格、54 号若干格红，要在两处临时仓链真仓的 `.claude/singlefs-ai-sop` 并写进 .gitignore。做法：第 2706 行 `LINKED_INTO_SELFTEST_REPOSITORIES`、第 2709 行 `link_sop_copy`，在 replay（`run_replay_cells`）与 54 号（`run_layer0_stage_cells`）两处调。
   - 改前原样（开工时在仓里跑）：`  ✗ admission.py 自证没过：17 格判错（共 207 格）`；草稿副本上 replay.sh 也换成带 preflight 那一版之后：`  ✗ admission.py 自证没过：21 格判错（共 207 格）`。点名的是 54 号若干格与「replay.sh：老产物没有产物头…」两格，原因原样 `…/research/scripts/../../.claude/singlefs-ai-sop/scripts/preflight.sh: No such file or directory`。
   - 改后见「出口各项」232 格全过。
2. 在新建三份脚本时收到：仓根模板超写范围，改放草稿目录（见「结论」）。
3. 在跑出口检查时收到：singlefs-39 的垫片 `.claude/scripts/preflight.sh` 出来了，要新脚本改 source 它、补声明、`--selftest` 挪到 preflight 之后，admission 自证临时仓再链 `.claude/scripts`。都做了（第 2 条与第 1 项）。这期间仓里 admission 自证又因 helper 改 source 垫片红了 21 格，原样 `…/research/scripts/../../.claude/scripts/preflight.sh: No such file or directory`，加链之后转绿。
4. 看门狗报：驱动自证里真起了 `cargo test … --exact the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts`，被判成崩溃枚举用例。
   - ① 主工作区的 stage-inputs.tsv 没有这一行：它是自证在临时仓副本的登记表里追加的（`crash-case:sharded-selftest`）。真仓登记只点名标了 `#[ignore]` 的用例；新加的 fast-tier 那条第 131 行标了。
   - ② 自证改成默认用假 cargo：临时仓里另写一条标 `#[ignore]` 的替身用例，三趟都不带 `--include-ignored`，不算重型。只有带 `SINGLEFS_HEAVY_TESTS` 时才用真 cargo 跑那条小流用例，不带时成功行写明真 cargo 那一趟本次未跑（自证第 48–89 行）。
   - 那一次真 cargo 跑了 7 分钟（临时目录 1.2G，自己删了），10 格全过：`  ✓ layer0-shard-run.sh 自证通过：10 格都对（第二台是本机上的另一个目录，没碰真的第二台）`。它经 `capped.sh 4` 与 `run-with-memory-cap.sh 8G` 起。
   - 假 cargo 版的证红：草稿副本里把驱动比工具链与数账本两处换成 `true`，原样 `  ✗ layer0-shard-run.sh 自证没过：2 格判错（共 10 格）`（③ ④ 两格红），换回之后 10 格全过。

## 出口各项（在仓里现跑，2026-09-27；末行原样）

| 命令 | 退出码 | 末行 / 判红的行 |
|---|---|---|
| `python3 research/scripts/admission.py --selftest` | 0 | `  ✓ admission.py 自证通过：232 格都对（含弄坏开关 skip-unchanged、…、single-worker-threads-field、threads-ignore-shards、shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch 下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）`（中间一段开关名是我略的；全文在 `logs/final-admission.log`；改完 54 号出路句之后又跑了一次，同样 232 格全过） |
| `python3 .claude/hooks/lib_heavy_tests.py --selftest` | 0 | `  ✓ lib_heavy_tests 自检通过（查了 141 种：cargo 与包装过的命令行 55 种、/usr/bin/time、flock、systemd-run 这一类包在外面的 33 种、直接执行的测试二进制 20 种、按名字判的脚本 6 种、跑不跑编译出来的代码 26 种、导入不了 admission.py 1 种）` |
| `bash .claude/hooks/heavy-test-guard.sh --selftest` | 0 | `  ✓ 自检通过（查了 658 种，其中该拒 121 种）：…` |
| `bash research/scripts/layer0-shard-run.sh --selftest` | 0 | `  ✓ layer0-shard-run.sh 自证通过：10 格都对（假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）；第二台是本机上的另一个目录，没碰真的第二台）`（31 秒；直接跑 `layer0-shard-run-selftest.sh` 被重型闸要求经内存包装，经 `run-with-memory-cap.sh 8G` 跑同样 10 格全过） |
| 47 号 | 1 | 只有一处红：`  ✗ python3 research/scripts/check-segment-registry.py --selftest 没过（退出码 1）：`（段序列登记表与 E142 产物对不上 2 处）。不在这一批里，没修。admission.py 与驱动自证在这一趟里没被列出 |
| 62 号 | 0 | `  ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）` |
| 63 号 | 0 | `  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸与弹窗断言闸注册着、自证通过，…` |
| 73 号 | 0 | `    没扫的目录 0 个（不在）：（没有）`；它的汇总行 `  ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 3 个目录（research/scripts .claude/hooks .claude/scripts），拒绝都带出路、shell 纪律守住；…` |
| `doc-lint.sh .` | 1 | `  ✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、11 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 514，跳过 0）`：点名 O1–O3、F23、F24、M32、M34、U11、U13 与 `.claude/kb/experiments-history.md`、`.claude/kb/experiments/158-择根与修复四岔路.md`。在这一批改过的八份里 `grep -nwE` 这几个编号零命中，两份 kb 是别的会话的 M |
| rules-lint（项目本地） | 0 | `  ✓ 规则只写怎么做（扫了 29 份文件 1958 行；没扫 0 个；记录小节 0、论证小节 0、…命中 0；…）` |
| `GATE_LINT_DIR=.claude/gate.d` gate-lint | 0 | `  ✓ 门禁自检通过：84 个脚本（.sh 与 .py）、305 条拒绝都带了出路`（改 54 号第 414 行出路句之前红 1 处，见第 9 条） |
| `SHELL_LINT_DIR=.claude/gate.d` shell-lint | 0 | `  ✓ shell 纪律检查通过（共 76 个脚本）` |
| gate-lint / shell-lint 对 research/scripts 与 .claude/hooks | 0 | research gate-lint `  ✓ 门禁自检通过：73 个脚本（.sh 与 .py）、237 条拒绝都带了出路`；research shell-lint `  ✓ shell 纪律检查通过（共 39 个脚本）`；hooks gate-lint `  ✓ 门禁自检通过：12 个脚本（.sh 与 .py）、18 条拒绝都带了出路` |
| `preflight-lint.py` | 0 | `  ✓ 准入与运行条件：判了 137 个脚本（登记目录 49、脚本 3、钩子 9、门禁阶段 76），都在开头写明了条件并先判；…` |
| `gate-overlap.py` | 0 | `  ✓ 相对 97f5904b44cd：新加的门禁与钩子 2 个（.claude/gate.d/84-verdict-false-named.sh、.claude/hooks/handback-guard.sh）都写明了比过谁，改过的 92 份脚本对照已有的 133 份没有整段相同` |
| 阶段归属表登记给 tooling-writer 的阶段 | — | awk 列出 0 个，没有要额外跑的 |

`admission.py` 在 `.claude/preflight-exclude` 第 17 行登记成「还没改完：别的会话（singlefs-99 的 G2）正在改它，改完之后补声明」。它的准入声明我没写（规格说归 singlefs-39），现在交回，这一行要由那边收尾。

## 新写的 `# gate-similar:` / `# hook-events:` 行
没有：这一批没新建门禁阶段或钩子（54 号是改，lib_heavy_tests.py 是库）。新建的三份是研究脚本，准入声明原样：
- layer0-shard-run.sh 第 33–34 行：`# admission: always 每次调都跑这一刻这棵树上的一条崩溃枚举用例；复用判定在门禁 54 号的全绿标记里，不在这里`／`# run-condition: command cargo rustc python3 rsync ssh nproc git`
- layer0-shard-run-selftest.sh 第 17–18 行：`# admission: always 自证判的是这一刻的驱动脚本与仓，每次调都要现跑`／`# run-condition: command cargo rustc python3 rsync git nproc`
- layer0-shard-configuration-check.sh 第 17–18 行：`# admission: always 判的是这一刻配置文件与第二台的样子，每次调都要现判`／`# run-condition: command git bash`

三份的自证没挂进 47 号的 runner 表：47 号不在规格的改动清单里，交主 agent 定。

## 第三轮要攻的改后字面（判决第四节口径）
- admission.py：第 182–189 行（`shard=` 写法、`SHARD_DRIVER_FILES`、`LAYER0_SHARD_VARIABLE`）；第 208–209 行（判法入口加 `crash-case-shardable`）；第 1163 行起 `parse_crash_case` 的 `shard=` 分支；第 1547–1563 行（驱动按内容进清单）；第 1608 行与第 1625–1657 行（merge 那一行逐片判线程）；第 1856–1857 行（清分片开关）；第 1894–1895 行（摘要里的子命令表）；第 2240 行 `command_crash_case_shardable`；自证第 2706–2718 行、第 3365 行、第 3416 行、第 3471 行、第 3783–3822 行、第 3824 行起。
- 54 号：第 25–29 行、第 237–244 行、第 350–358 行、第 394–418 行。
- stage-inputs.tsv：第 16–20 行、第 28 行、第 36–37 行、第 39–40 行。
- lib_heavy_tests.py：第 95 行、第 597 行、第 606 行、第 651 行、第 706–708 行，自证第 967 行起。
- 规则 implementation-workflow.md：第 58 行。
- 三份新脚本全文，重点是驱动第 33–39 行（preflight 在 `--selftest` 之前）、第 149–163 行（两台指纹）、第 263–279 行（判与写标记），以及自证的假 cargo（第 48–89 行）。

## 看到但没做
1. 54 号快档的 `run_layer0_test_binary` 起两条流的快用例时，没清调用方的 `SINGLEFS_LAYER0_SHARD`。第 6 条只点名 `crash-case-command`，这一处照旧（假红方向）。
2. 分片 merge 那一趟，54 号交给 `crash-case-record` 的 `--machine-cores/--threads` 是本机那一片的，标记里 `configured_worker_threads=` 只记本机。第二台那一片配的数只在 `parallel_finished=` 行的 `shard_configured_worker_threads=` 里（推的：标记字面对分片的描述不全，没改）。
3. strace 的别的长选项（`--output=`、`--user=` 等）不在 `LAUNCHER_OPTIONS_WITH_VALUE` 里：带值另起一个词的写法会被当成命令，剥错。只补了 `--env`。
4. 仓根 `multi-host.env.example` 由主 agent 拷；实分一 deliver 里的 kb 第六节 diff 没碰（归书记员）。

## 没做什么
- 没跑重型测试：54、55、57、59、87 号本身、`gate.sh`、全量 `cargo test`、`check.sh`、名字带 layer0 的目标、驱动的非 `--selftest` 形态都没跑。驱动自证在追加 ④ 之前用真 cargo 跑过一次小流用例（`crash_enumeration_sharded_across_processes`，不带 layer0、不带 `--include-ignored`，经内存包装与 capped 4）。看门狗把它记成崩溃枚举用例：临时仓登记表里那一行让它看起来像登记用例。之后改成默认假 cargo。
- 没走定义三方，没提交，没做 git 写操作。
- heavy-test-guard.sh 没改；47 号没改（新脚本的自证没挂进 runner 表）。
- 47 号与 doc-lint 的红不在这一批，没修。

## 草稿目录与清理
- 删了：`/tmp/claude-1000/gate-batch-m2-g2/dev`（215M，开发用的仓副本）、`sim`（516K，套 hunk 的演练副本）、`try`（1.1M，patch 试打）。自证、探针建的临时仓都由它们自己删掉了（`ls -d /tmp/admission-selftest-* /tmp/g2-probe-*` 零个）。
- 留着：`backup/`（五份改前原件）、`g2-changes.diff`、`*-vs-deliver.diff`、`hunks-*.diff`、`round2-*.diff`、`apply_hunks.py`、`probe_g2.py`、`probe_strace.py`、`logs/`、`progress.md`、`multi-host.env.example`（待拷）、`final-sha256.txt`（八份仓内文件交回时的 sha256）。

## 补记
仓根 `multi-host.env.example` 已由主 agent（README 那件）放进仓根，与草稿那份逐字节相同（`cmp` 现核相同，sha256 `72ae03d4a01a49ccc069dc27c62a03e31d42dc420e52a879fa67bd4c2c458bd4`），不用再拷。
