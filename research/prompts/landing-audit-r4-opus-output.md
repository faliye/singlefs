# landing-audit-r4 攻方腿（Opus）：K2 新句子与已有的矛盾、K3 哪一条最脆（2026-10-23）

攻击面：正文 `research/prompts/_landing-audit-r4-body.md` 节「三、格」的 K2、K3。被判对象按主工作区今天的文件读（开工时 `sha256sum -c research/prompts/landing-audit-r4-snapshot/judged-sha256.txt` 没有一行对不上）。没读禁读清单里的正推腿报告。

## 复跑

模型目录 `research/prompts/landing-audit-r4-opus-model/`。四个探针都不编译、不起 cargo，只拷被判的钩子与脚本进草稿目录喂造出来的输入：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/landing-audit-r4-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

这一腿开工时没有主 agent 建的冻结树，`--frozen` 我给的是仓根本身（被判文件与快照哈希逐个相同）。`--repo` 只读：派发闸 `--check-prompt` 的 `--root`、骨架 `dispatch-prompt.py`、merge 判法读的真树 `Cargo.toml`。末尾四行 `name=<探针> 与 out/ 逐行相同` 是比对结果；这一腿跑了两趟（`rerun1`、`rerun2`），第二趟四行都是「逐行相同」。

文件与 sha256（`find … -type f | sort | xargs sha256sum` 原样）：

```
9cc40a8fc41e0cea181dc3edee3a051ffd4249bd51b197be5ed5a9b5068a3dc6  research/prompts/landing-audit-r4-opus-model/out/merge_select_probe.out
45d101023e6484dcc606bb00c2a2e6b5f435f52912f41c928e44e2474083cc8f  research/prompts/landing-audit-r4-opus-model/out/naming_probe.out
0d43dbc8a39e9d5555599830f4bf8249be67adb24b647269cd4404c51355b2f3  research/prompts/landing-audit-r4-opus-model/out/scratch_prompts.out
e20d11f8a03f0fa525b965437c5d0b27559954151b4262eee9b37ec4a42505bf  research/prompts/landing-audit-r4-opus-model/out/threshold_probe.out
6f7cd681695eb47a4856bcbd3cc5ab33265435eedd22f3e5c78a5a9a6fdb60fc  research/prompts/landing-audit-r4-opus-model/probes/merge_select_probe.sh
09f90dd4c44d09cb1ea5896ba6fb2a27bd32caeabe7d15e4898a12308bb5c11b  research/prompts/landing-audit-r4-opus-model/probes/naming_probe.py
3c295e84335f25a70d919ae994ee030035cd50ce3c9264885bb93e8bbf28abd1  research/prompts/landing-audit-r4-opus-model/probes/scratch_prompts.sh
598a59a3f49e99948db351c19b136bdda11b62e7526c5ca957133f3b801e0427  research/prompts/landing-audit-r4-opus-model/probes/threshold_probe.py
70a932b78b058604d4dd760ce7ec4c41b152c6e88a9b599b75d70e42d0ba245d  research/prompts/landing-audit-r4-opus-model/tools/rerun.sh
```

## 各格判定一览

| 格 | 编号 | 一句话 | 判定 |
|---|---|---|---|
| K2 定义之间 | K2-1 | 合入后验证带 `=user-request` 跑 checker 档快档、不弹窗：主 agent 定义第 3 条、规则 implementation-workflow 的场合表、重型闸的出路三处都还写「提交之外先弹窗」「其余任何时候重型不跑」 | 打中 |
| K2 定义之间 | K2-2 | 主 agent 定义调度表里 merge-and-verify 那一行的命令没带前缀，照抄会让新那一步拒、整条退非 0 | 打中 |
| K2 定义之间 | K2-3 | 主 agent 定义调度表写写回员在 `crates/` 下的例外只有 `invariants/*.md`；写范围表与写回员定义已放到 `invariants/**/*.md` | 打中 |
| K2 定义之间 | — | 结局账 `.txt` 放 `records/`：没有规则或门禁拒它，doc-text 只判 `records/*.md` | 没打中 |
| K2 定义之间 | — | 共用约束交回正文闸那一条补的两样与 `handback-guard.sh` 文件头 ⑤ ⑥ | 没打中 |
| K2 钩子之间 | K2-4 | 「这一份提示的草稿目录」有四处认法：派发闸 ⑰（冒号可无、不限行首）、派发闸 ⑯ 写者锁、完工清理闸、交回闸 ⑥（三处都要行首加冒号）；同一行提示四处判得不一样 | 打中 |
| K2 钩子之间 | K2-5 | 「三方腿报告路径」三处认法：派发闸报告行那一支认 `research/prompts/<轮>-opus-output.md` 这种没填的形态写法，同一函数里新加的整份提示那一支不认；交回闸 ② 连子目录与 `<轮>` 都认 | 打中（轻） |
| K2 钩子之间 | — | 交回闸新 ⑤ 与共用约束「命名检查交回前跑」、rsync 副本交主 agent 那一支 | 没打中 |
| K2 门禁之间 | K2-6 | `replay-comparison-units.tsv` 文件头五行「不登记的比对对象」把 C879 要登记的五行写成了合法的不登记，C879 还写「把这几条登记进」，两处对同一件事判法相反 | 打中 |
| K2 门禁之间 | — | `experiment-number-directories.tsv` 放 `.claude/gate.d/`、`claim-experiment.sh` 也读它 | 没打中（字面不矛盾，只与先例不一致） |
| K2 门禁之间 | — | `lib-forbidden-notations.py` 从放行变成被拒 | 不报（九十节 ㉖ 已记） |
| K3 ⑥ | K3-1 | 交回闸 ⑤ 认「真跑过命名检查」只看命令位置与退出码形状：`false && …`、`true || …`、`if false; then …; fi`、只定义不调用的函数、`timeout 0.1 …` 被杀退 124，五种都没跑完而放行 | 打中 |
| K3 ⑥ | — | 被钩子拒的、跑在别的根上的算不算 | 没打中（两样都照拒） |
| K3 ⑳ | K3-2 | ⑰ 认草稿目录「冒号可无、不限行首」：转述别的子 agent 时写「，草稿目录 <别处>」或列表项「- 草稿目录 <别处>」，那份别处的 report.md 被当成本提示的报告拒；自己的写成「草稿目录在 <路径>」又认不出、放过本提示的 report.md | 打中 |
| K3 ⑬ | — | 下限之后真大的命令照报（阈值是峰值的 1.5 倍）；只多看出峰值正好为 0 的条目阈值是 8 GiB、峰值 1 字节的是 512 MiB，不单调 | 没打中（不单调那一点记作观察） |
| K3 ⑭ | — | 照登记表认 light 自证、按 TMPDIR 与名字形态认临时目录 | 没打中（读代码推的，没起真进程） |
| K3 merge-and-verify | — | 判「补丁没碰 checker 档闭包」靠成员 `Cargo.toml` 三张依赖表；本仓 8 个成员全在闭包里，闭包外的文件不会让这一步误报本次未跑；「本次未跑」只剩 `scripts/experiment-apparatus` 下的包与空 `crates.patch` 两种 | 没打中（闭包判法在本仓上落空，记作观察） |

K2 打中 6 条（K2-1 到 K2-6），K3 打中 2 条（K3-1、K3-2）。下面每一条的改法都只在我的探针上量过或是推的，都算被攻过零轮。

## K2 新句子与已有的有没有矛盾

### K2-1　合入后验证跑 checker 档快档，与「提交之外先弹窗」「其余任何时候重型不跑」三处

并排：

- 新句子：`.claude/main-agent.md:29`：「编译与测试之后脚本另跑一步 checker 档快档：命令带 `SINGLEFS_HEAVY_TESTS=user-request`」；`.claude/main-agent.md:52`：「两条命令开头都带 `SINGLEFS_HEAVY_TESTS=user-request`」。
- 已有的一（同一份定义）：`.claude/main-agent.md:22`：「提交之外任务确实要跑，先弹窗问用户，同意了才跑」。
- 已有的二（规则，比定义高一级）：`.claude/rules/implementation-workflow.md:45`：「任务确实要跑时主 agent 先弹窗问用户」；`.claude/rules/implementation-workflow.md:47`：「其余任何时候 | 重型不跑」。场合表里没有「合入后验证」这一行，合入后验证落在「其余任何时候」。
- 已有的三（钩子拒绝说明）：`.claude/hooks/heavy-test-guard.sh:857`：「提交之外任务确实要跑的：主 agent 先弹窗问用户」。
- 已有的四（共用约束）：`.claude/agent-common.md:58`：「重型测试只由主 agent 在提交时或用户要求时跑」。

矛盾在哪：合入后验证不是提交，也不是那一刻的用户要求；新那一步按用户 2026-10-23 的定案每趟都带 `=user-request` 起 checker 档包的测试、不弹窗。四处已有的句子都没写这个例外，照字面读，主 agent 每趟合入后验证之前都该弹窗，或者干脆不该跑（规则那一张表）。`b9-defs-verify.md` 「没做什么」自己点过第 3 条，规则 implementation-workflow 场合表与重型闸那句拒绝说明它没点。`=user-request` 这个值在 `.claude/skills/crash-test/commit-stages.md` 与 `.claude/skills/gate/staged-gate.md` 里说的都是「用户要求时跑」，现在多了一种「常设定案下的合入后验证」，一个值两种意思。

四句：分辨改法（只改定义第 3 条修不了规则那一张表）；读这几句的是主 agent，跑的那一刻看得到四处原文；满足 K2 字面「新句子与已有的规则、定义、钩子之间有没有矛盾」；九十节 ㉓ 的改法（定义第 10 条与验证段第 1 步加一步）恰好是造出这个矛盾的那一处，对这一格不起作用。

改法（推的，被攻过零轮）：规则 `.claude/rules/implementation-workflow.md` 场合表加一行「合入后验证 | checker 档快档里单元键对不上的测试函数，经 `scripts/gate/merge-and-verify.sh`，命令带 `SINGLEFS_HEAVY_TESTS=user-request`，不弹窗（用户 2026-10-23 定案）」；定义第 3 条加「合入后验证那一步除外，见第 10 条」；重型闸出路那一句补「合入后验证照 main-agent 第 10 条带 =user-request」。规则要走规则的改法，归主 agent 定。

推翻条件：仓里另有一处规则或定义已经写了「合入后验证的 checker 档快档不弹窗」这个例外（`grep -rn '合入后验证' .claude/rules/ .claude/skills/` 这一次只命中 `.claude/rules/verification.md` 第 49 行「合入后验证三条的全部由主 agent 跑」（轻型三条）与第 59 行的范围声明，两处都不是这个例外）。

### K2-2　调度表那一行的 merge-and-verify 命令没带前缀

- 调度表「一批实现员交回到齐、合入之前」那一行，`.claude/main-agent.md:96`：「主 agent 跑 `bash scripts/gate/merge-and-verify.sh [--memory 上限] [--threads N] <补丁目录…>`：打全部补丁」，后面只列编译、测试、fmt、clippy、build，没有前缀，也没有 checker 档快档那一步。
- 同一份定义 `.claude/main-agent.md:52`：「两条命令开头都带 `SINGLEFS_HEAVY_TESTS=user-request`」。
- 脚本：`scripts/gate/merge-and-verify.sh:696`：「checker 档快档那一步要起它的命令带 SINGLEFS_HEAVY_TESTS=user-request（提交时 =commit）」，没带就记红、整条退 1（自证那一格在 `scripts/gate/merge-and-verify.sh:1366` 判同一句出路）。

照调度表那一行原样起，补丁只要碰了 `crates/` 下任何一个成员（K3 那一节量过：本仓 8 个成员全在 checker 档包的依赖闭包里），这一步就拒、整条退非 0。调度表是主 agent 查「这一步跑什么」最常落到的那一处，第 10 条与验证段第 1 步改了，它没跟。

四句：分辨改法（改调度表那一行就修）；主 agent 看得到；满足「两处判同一件事而判法不同」——同一条命令，两处写法一处带前缀一处不带；九十节 ㉓ 的改法没碰这一行。

改法（推的，被攻过零轮）：调度表那一行的命令前面加 `SINGLEFS_HEAVY_TESTS=user-request`，列举里补「checker 档快档（单元键对不上的测试函数）」，或整段改成指到验证段第 1 步、不在调度表里另写一份。

推翻条件：merge-and-verify 不带前缀、补丁碰了 checker 档闭包时退 0（自证「补丁改到 checker 档包依赖的 demo、起它的命令没带 SINGLEFS_HEAVY_TESTS」那一格判的正是退 1）。

### K2-3　写回员在 `crates/` 下的例外，调度表比写范围表窄

- 调度表「定案之后写回 kb」那一行，`.claude/main-agent.md:102`：「例外是 `crates/singlefs-checker/invariants/*.md`（不变量清单，只限这些 `.md`，写回员可以写）」。
- `.claude/agents/kb-scribe.md:42`：「不变量清单，下一层与各层子目录里的都算，只限这些」。
- `.claude/hooks/agent-write-scope.tsv:13`：「crates/singlefs-checker/invariants/**/*.md」。

主 agent 照调度表判「不变量清单子目录里的 .md 该交谁」会判成交实现员（`crates/` 别处并进第 4 条下一批），写回员定义与写范围表却已放行写回员写。门禁 code-tooling 的 agent-write-scope 格判的是写范围表与定义，调度表不在它判的范围里（b9-gates-a 的说法），所以这一处没有会红的检查。正文里点名的「`CLAUDE.md`「什么时候派哪个 agent」那张表」在今天的仓里是 `.claude/main-agent.md` 节「什么时候派哪个 agent」，`CLAUDE.md` 里没有 kb-scribe 与「只限这些」（`grep -n 'kb-scribe\|只限这些' CLAUDE.md` 零命中）。

四句：分辨改法（只改调度表那一句）；主 agent 读得到三处；满足「两处判同一件事而判法不同」；九十节 ⑧ 的改法改了表与定义，没碰调度表，对这一格不起作用。

改法（推的，被攻过零轮）：调度表那一句改成「例外是 `crates/singlefs-checker/invariants/` 下各层的 `.md`（`*.md` 与 `**/*.md` 两行，只限这些 `.md`）」，或只写「写范围照 `.claude/agents/kb-scribe.md`「写范围」」不再列模式。

推翻条件：今天 `invariants/` 下没有子目录（b9-gates-a 写了），所以这一处眼下不会让谁真的派错；要是用户定「调度表只写下一层、子目录另由定义管」，这一条就是措辞，不是矛盾。

### 没打中：结局账 `--out records/<轮>-outcome-<这一天>.txt`

查了什么：`.claude/rules/directory-layout.md` 四节没有一句管 `records/`（`grep -n records .claude/rules/directory-layout.md` 零命中）；全仓门禁与钩子里按目录认 `records/` 的只有 doc-text：`.claude/gate.d/doc-text.sh:49`：「这次改动碰到的（含未跟踪的）records/*.md」，`.claude/gate.d/doc-text.sh:749` 的 case 也只收 `records/*.md`；`.claude/hooks/session-start.sh` 列「最近动过的记录」只取 `.md`。没有哪一道拒 `records/` 下的 `.txt`，也就不矛盾。

顺带看到的一点（不算打中）：`.txt` 不进 doc-text 的 private-values、clock-times、out-of-project-names 几格。我在草稿目录跑了一次 `python3 scripts/agents/agent-cost.py outcome --since 2026-10-22 --until 2026-10-22 --summary`（没带 `--out`），57 行输出里 `/home/` 0 处、`时:分` 形态 0 处，今天的输出不带这几样；以后 outcome 的输出改了写法（例如带上会话目录），这几格判不到它。推翻条件：doc-text 或别的门禁其实另判了 `records/` 下的非 `.md`。

### 没打中：共用约束交回正文闸那一条与 `handback-guard.sh` 文件头

逐句并排比了共用约束「执行前拒绝的写法」交回正文闸那一条新补的两样，与 `.claude/hooks/handback-guard.sh:26` 到第 38 行的 ⑤ ⑥：类型（实现员、执行员、工具员；写回员）、工具（Edit / Write / MultiEdit）、树（主工作区或带 `.git` 的副本，rsync 副本不判）、「只看会话记录里的命令，不看报告」、「找不到的目标」一节三处找（交回正文、交回点名的报告、派发提示「报告：」那一份）都对得上。共用约束没写文件头里的「被钩子拒的、超时的、没有结果的不算」与「认不出的：Bash 里写 .rs」，是少写了射程，不是说反。共用约束「门禁」一节「命名检查交回前跑」写在 rsync 副本里改的「不在副本上跑」、两条命令交主 agent，交回闸照不判，三处一致。

### K2-4　「这一份提示的草稿目录」四处认法不同

同一件事——派发提示里哪一行点名了这个子 agent 的草稿目录——今天有四处各认各的：

| 处 | 认法 | 出处 |
|---|---|---|
| 派发闸 ⑰（这一批新加） | 「草稿目录」前面不连着字、不在引号里，冒号可有可无，路径可带反引号，不限行首，只认工作根或旧根底下的 | `.claude/hooks/runner-dispatch-guard.sh:1310`：「REPORT_SCRATCH_MENTION = re.compile」 |
| 派发闸 ⑯ 写者锁 | 行首（`LINE_START`）加「草稿目录」加冒号（冒号必须有） | `.claude/hooks/runner-dispatch-guard.sh:1507`：「WRITER_LOCK_DIRECTORY_LINE = re.compile(LINE_START」 |
| 完工清理闸 | 行首加「草稿目录」紧跟冒号，路径任意 | `.claude/hooks/completion-cleanup-guard.sh:169`：「SCRATCH_LINE_PATTERN = re.compile」 |
| 交回闸 ⑥（这一批新加，找写回员自己起草的 spec.json） | 行首加「草稿目录」加冒号 | `.claude/hooks/handback-guard.sh:459`：「for match in re.finditer(PROMPT_LABEL_LINE_START + re.escape(label)」，调用在 `.claude/hooks/handback-guard.sh:475` |

后果：一行「草稿目录 ~/.local/share/singlefs/work/x/」（不写冒号）或句中「，草稿目录 <路径>」，⑰ 当本提示的草稿目录判报告名，⑯ 不在那里建写者锁，完工清理闸不查那里留没留副本，写回员在那里起草的 spec.json 交回闸 ⑥ 找不到。工具员自己知道两处不同，还拿它当用：`research/prompts/subagent-landing-audit-2026-10-22/b9-dispatch-guard.md:72`：「草稿目录一行不写冒号，这样 stdin 那几格不会在真工作根里建写者锁」。⑰ 宽出来的那一截正是 K3-2 误拦误放的来源。

四句：分辨改法（四处收成一份认法就修，只改 ⑰ 修一半）；四个钩子判的时候看到的是同一份提示原文；满足「两处判同一件事而判法不同」；九十节 ⑳ 给的改法（「⑰ 与报告形态判只看报告行里落在本提示草稿目录下的路径」）没说草稿目录怎么认，工具员按 K2-2 那份提示的写法放宽，与另三处拉开了。

改法（推的，被攻过零轮）：四处共用一份认法，放进 `.claude/hooks/lib_work_root.py` 或 `lib_writer_lock.py`，取「行首、冒号必须有」那一种（骨架 `scripts/agents/dispatch-prompt.py` 打的就是「草稿目录：」），⑰ 改调它；骨架普及之前那 79 份写法（b9-dispatch-guard 第 1 节）照九十节 ㉗ 已记的「没有草稿目录一行时只判裸文件名」处置。自证加一格：同一份提示交四处，认出的草稿目录集合逐个相同。

推翻条件：四处其实调的是同一个函数（今天 `grep -n '草稿目录' .claude/hooks/*.py .claude/hooks/*.sh` 里四个 re.compile 各在各的文件与行）；或者用户定「⑰ 有意比另三处宽」。

### K2-5　「三方腿报告路径」三处认法不同（轻）

- 派发闸报告行那一支：`.claude/hooks/runner-dispatch-guard.sh:1461`：「THREE_WAY_REPORT_FORM = re.compile」，名字部分是「除 `/`、空白、引号外任意字」，`<轮>` 这类没填的形态写法也算。这一支是第九批之前就有的（`landing-audit-b9-dispatch-guard/orig/` 那份开工原样第 1319 行同一句）。
- 同一函数里这一批新加的整份提示那一支：`.claude/hooks/runner-dispatch-guard.sh:1462`：「（不认 <轮> 这类形态写法）」，名字部分只认 ASCII 字母数字与 `_.-`。
- 交回闸 ②：`.claude/hooks/handback-guard.sh:125`：「LEG_REPORT_MENTION = re.compile」，名字部分里 `/` 与 `<`、`>` 都认。

喂的（`out/scratch_prompts.out` 原样）：

```
leg-form-placeholder（three-way-attack）：退 0；  ✓ 派发闸照这份提示放行（three-way-attack，查了 21 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者�
leg-subdir（three-way-attack）：退 2；✗ 派 three-way-attack 的提示整份提示里没有一处 research/prompts/<名>-output.md（报告行点名的，或不在「」『』“”与反引号里的）；报告行点名的「research/p
交回闸 ② 的 LEG_REPORT_MENTION 认出： ['research/prompts/x-r1/x-r1-opus-output.md']
交回闸 ② 的 LEG_REPORT_MENTION 认出： ['research/prompts/<轮>-opus-output.md']
```

也就是：报告行写「报告：research/prompts/<轮>-opus-output.md」（形态写法没填），派发闸放行，同一函数的新那一支却明写不认这种；报告写进 `research/prompts/<子目录>/…-output.md`，派发闸拒，交回闸收。前一种是误放：腿照这一行写不出真文件名。

四句：分辨改法；派发闸与交回闸看到的是同一个路径字符串；满足「两处判同一件事而判法不同」；九十节 ⑳ 与 ㉗ 的改法都没碰形态正则本身。

改法（推的，被攻过零轮）：三处共用一条形态正则（建议取新那一支的写法，并在报告行那一支同样不认 `<`、`>`），放进 `.claude/hooks/lib_three_way_legs.py`（三方腿的事已收在那里）；三处自证各加「`<轮>` 形态写法拒」「子目录拒」两格。

推翻条件：骨架之外没有人会写出 `<轮>` 这种没填的报告行（骨架打的是 `<填 轮>`，⑬ 管）；那这一条只剩三处不一致、没有真提示会撞上。

### K2-6　比对对象登记表的「不登记的比对对象」五行与 C879 判法相反

- 欠账 C879，`.claude/kb/checks-owed.md:560`：「E165 根选择与杀伤界正式跑没有单元」，现状句写 r5 重跑登记比对对象表第 15 至 19 行「在单元表里没有单元」、登记表「也没登记它们」，要做的那一列：「把这几条登记进 `.claude/gate.d/replay-comparison-units.tsv`」，门禁在单元名找不到时红。
- 这一批（任务 28）：`.claude/gate.d/replay-comparison-units.tsv:16`：「或在这里点名不登记」；`.claude/gate.d/replay-comparison-units.tsv:17`：「不登记的比对对象：research/prompts/e165-r5-prereg.md #15：r3 档择根，单元表 formal_run_units 里没有这条正式跑的单元」，第 18 到 21 行同形写 #16 到 #19。

是不是同一件事：是。C879 点名的就是 #15 到 #19 这五行，理由也是同一个事实（单元表里没有单元）。两处判法相反：C879 把它当欠账、要登记、要红；登记表文件头把它当合法的不登记，格读到这五行就算对上（b9-gates-b 的真仓读数「登记表对上 14 行、文件头点名不登记 5 行」，判绿）。登记表与格里都没有一处指到 C879（`grep -c C879` 在 `replay-comparison-units.tsv`、`doc-experiments.sh` 里都是 0）。

C879 的现状句这一批之后还对不对：字面还对（这五行确实没登记成数据行），但不全——它没写「格已经认这五行为不登记、判绿」，读 C879 的人会以为门禁在这五行上是红的或还没判。C879 要做的那一列「登记进……门禁红」与今天的格判法也接不上：照今天的判法，要让它红，得先删掉文件头那五行，而删掉之后格会因为「表里一行没有登记、也没点名不登记」判红——这才是 C879 要的红。

九十节 ㉖ 已记「点名不登记的五行没立只缩不涨」，那是「以后能不能随手加不登记行」；这里报的是「这五行本身与 C879 说反了」，不重复。

四句：分辨改法（只立只缩不涨不修这一格，五行照样判绿）；格判的那一刻看得到文件头的理由文字，看不到 C879；满足 K2 正文点名的那一问（「说的是不是同一件事，C879 的现状句这一批之后还对不对」）；九十节 ⑮ 的改法（固定形态、点名不登记）正是造出这五行的那一处。

改法（推的，被攻过零轮）：任选其一，交主 agent 定——（甲）五行的理由改成「C879 欠账：……，C879 还清时删这一行」，格加判：不登记行的理由要带一个在 `.claude/kb/checks-owed.md` 里开着的 C 号，C 号挪进「已还清」后这一行判红；C879 现状句补「登记表文件头把这五行点名为不登记，格判绿」。（乙）不留点名不登记这一条退路，五行照 C879 原样判红，挂在 C879 下直到装置补单元。

推翻条件：E165 那一侧已经判定这五行本来就不该有单元（C879「前提」那一列写「先判这几条正式跑该不该有单元」），那时 C879 应挪进已还清，文件头五行就是对的。

### 没打中：`experiment-number-directories.tsv` 放在门禁旁边

`.claude/rules/directory-layout.md:22`：「只被门禁自己读的配置（格名、阶段归属、排除表）留在门禁旁边」。这份登记被门禁 doc-experiments 与 `scripts/kb/claim-experiment.sh` 两处读，不是「只被门禁自己读」；同一节上一句管的是「只被某一组测试读的」定义与输入，也不是它。规则对「门禁与脚本合读」的配置没说放哪，字面上不矛盾。与先例不一致：同样被门禁与脚本合读的 `.claude/kb-directories`（`scripts/kb/quote-kb.py` 与 doc-registries 读）放在 `.claude/` 顶层；任务 28 交回自己写了放 `.claude/gate.d/` 是因为写范围到不了 `.claude/` 顶层。推翻条件：directory-layout 另有一句管合读的配置（我读全了这份文件四节，没有）。

### 没打中：交回闸新 ⑤ 与共用约束「命名检查交回前跑」、rsync 副本那一支

见上面「共用约束交回正文闸那一条」一段。另核了一处：共用约束要实现员「在主工作区（写仓根）或 git worktree 建的副本（写副本根）上跑」，交回闸按改动所在的树判根；实现员定义第 3 步让仓副本用 `rsync -a --exclude target --exclude .git` 建，没有 `.git`，交回闸照不判，两边一致。K3-1 里跑过「worktree 里改、根写主工作区」，交回闸照拒，也一致。

## K3 哪一条最脆

### K3-1　交回闸 ⑤ 把没跑完、根本没跑的命名检查认成「真跑过」

哪一处：`.claude/hooks/handback-guard.sh:30`：「被钩子拒的、超时的、没有结果的不算」。认法是两样：命令位置上出现那两条命令（`.claude/hooks/handback-guard.sh:359`：「commands_at_command_position(text, current)」，`&&`、`||`、`if` 分支、函数体里的都算命令位置），加上工具结果不是错误、或错误以「Exit code」开头（`.claude/hooks/handback-guard.sh:327`：「text.lstrip().startswith("Exit code")」）。它不看那两条命令有没有真的执行到、有没有跑完。

怎么复出来：`probes/naming_probe.py` 造实现员的会话记录（Edit 主工作区 `crates/core/src/lib.rs`，再一两条 Bash），喂草稿目录里拷的 `handback-guard.sh`（`CLAUDE_PROJECT_DIR` 指到一个带 `.git` 的假仓）。`out/naming_probe.out` 原样：

```
对照：改了 .rs 没跑：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
对照：两条都真跑：退 0
false && 两条（一条都没执行，退 1）：退 0
true || 两条（一条都没执行，退 0）：退 0
if false; then 两条; fi：退 0
bash -n 只查语法：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
timeout 0.1 包着、被杀退 124：退 0
两条写进函数定义不调用：退 0
根写成 "$CLAUDE_PROJECT_DIR"（真跑了两条）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
worktree 里改、根写主工作区（应拒）：退 2；✗ implementation-writer 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：
```

五种误放：`false && <两条>`、`true || <两条>`、`if false; then <两条>; fi`、`f() { <两条>; }` 只定义不调用、`timeout 0.1 <两条>` 被杀退 124。前四种是「命令位置上有、却一条都没执行」；最后一种最像真会发生的：命名检查扫全仓要一会儿，`timeout` 包着被杀，工具结果是「Exit code 124」，与「命名检查判红」同一个形状，被当成跑过。文件头写的「超时的不算」只管 Bash 工具自己的超时，不管命令里自带的 `timeout`。

正向核过、不算误判的三种：`bash -n`（只查语法）照拒；改动在 git worktree、根写成主工作区照拒（被问的「跑在别的根上的算不算」：不算，对）；根写成 `"$CLAUDE_PROJECT_DIR"` 照拒——子 agent 的 Bash 环境里这个变量没设（`echo "${CLAUDE_PROJECT_DIR-unset}"` 打 `unset`），那条命令真跑也不是在那棵树上跑，拒得对。被钩子拒的那一种自证里已有一格（「命名检查被钩子拒了不算跑过」），这里没重测。

四句：分辨改法（下面的改法修这五种，「只认工具结果不是错误」那种收严修不了 `true ||` 与函数定义两种）；交回闸判的那一刻手里有工具结果全文，看得到两条命令自己打的汇总行；满足九十节 ⑥「要拦什么」字面「挡不住「报告里说自查了、其实没跑」」——这五种都是会话记录里有命令、其实没跑；九十节 ⑥ 的改法（核会话记录里跑没跑过那两条命令）就是今天这一版，对这五种不起作用。

改法（只在我的探针上量过误放、改法本身是推的，被攻过零轮）：「跑过」改成认工具结果里两条命令各自的收尾行——code-source-discipline.sh 四格各自的格头或汇总行、naming-lint.sh 的「查了 N 个 .rs 文件」那一句——而不是只认退出码形状；退出码 124、137、143 一律不算跑过。自证加五格（上面五种各一格，应得 2）与一个弄坏开关（退回只看退出码形状）。

推翻条件：交回闸其实另核了工具结果正文（今天 `tool_calls` 只交「跑过没有」一个布尔，正文只用来判开头是不是「Exit code」）；或者实现员、执行员、工具员的真会话记录里，这五种形状一次都没出现、以后也不会出现（那这一条只是射程外，不是误放的现实风险；我没扫真会话记录）。

### K3-2　⑰ 的「草稿目录」认法：转述的别处被当成自己的、自己的写法稍变就认不出

哪一处：`.claude/hooks/runner-dispatch-guard.sh:1310`：「REPORT_SCRATCH_MENTION = re.compile」——「草稿目录」前面不连着字（中文字算字，标点与行首不算）、冒号可有可无、不限行首、不在引号里，后面接工作根底下的路径，就当本提示的草稿目录。判报告名时只判落在这些目录下的与裸文件名。

怎么复出来：`probes/scratch_prompts.sh` 照 clerk 骨架造五份提示（本提示的草稿目录行与报告行都照骨架写，`草稿目录：…/k3-probe-mine`、`报告：…/k3-probe-mine/result.md`），只改一句正文，喂草稿目录里拷的派发闸 `--check-prompt --type clerk`。`out/scratch_prompts.out` 原样前五行：

```
relay-comma（clerk）：退 2；✗ 派 clerk 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：「/home/fy5090/.local/share/singlefs/work/k3-probe-previous/report.md」。
relay-control（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 18 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）、⑯ 草稿目录写者锁（派的
relay-list（clerk）：退 2；✗ 派 clerk 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：「/home/fy5090/.local/share/singlefs/work/k3-probe-previous/report.md」。
own-zai（clerk）：退 0；  ✓ 派发闸照这份提示放行（clerk，查了 17 行提示）；派的那一刻另判：⑦ 并发与限额窗口（派的那一刻按主会话记录判）
own-control（clerk）：退 2；✗ 派 clerk 的提示报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：「/home/fy5090/.local/share/singlefs/work/k3-probe-mine/report.md」。
```

五份各改了什么：

| 提示 | 正文那一句 | 该得 | 实得 |
|---|---|---|---|
| relay-control | 「前一段交回在先，那一段的草稿在 <别处>/ 里，报告 <别处>/report.md 是那一段写的，只读不改。」 | 放（转述） | 放 |
| relay-comma | 同上，只把「那一段的草稿在」换成「，草稿目录 <别处>/ 里有它的探针」 | 放（转述） | 拒（误拦） |
| relay-list | 「要读的：」下列表项「- 草稿目录 <别处>/ 下的 logs/」「- 报告 <别处>/report.md（前一段的）」 | 放（转述） | 拒（误拦） |
| own-control | 骨架原样，只把自己的报告改成 `<本草稿目录>/report.md` | 拒 | 拒 |
| own-zai | 同上，再把自己那一行写成「草稿目录在 <本草稿目录>（自己建）」 | 拒 | 放（误放） |

误拦：用户定 ⑳「按草稿目录判」，为的就是转述别的子 agent 的报告不判；可转述时只要顺手写出那一段的「草稿目录 <路径>」（逗号后、列表项里），那个别处就被当成本提示的草稿目录，它下面的 report.md 被当成这一份的报告拒。误放：本提示自己的草稿目录写成「草稿目录在」「草稿目录为」「草稿目录（自己建）：」这类前面没毛病、后面插了字的，⑰ 认不出，自己的 report.md 就放过去；九十节 ㉗ 记的是「提示里没有「草稿目录」一行时」，这里是有一行、认不出，形状不同。这两种同时说明被问的那一句：会认到别处引的草稿目录，会，而且只差一个冒号或一个「在」字。

四句：分辨改法（K2-4 的「行首、冒号必须有」修 relay-comma 与 relay-list 两种误拦；own-zai 那种误放要另加一条「本提示一行草稿目录都认不出就按 ㉗ 的处置」）；派发闸判的那一刻看到的就是这几行原文；满足九十节 ⑳「要拦什么」字面「只看报告行里落在本提示「草稿目录：」下的路径」——字面就带冒号，今天的认法比字面宽；⑳ 给的改法正是今天这一版，对这三格不起作用。

改法（只在我的探针上量过误判、改法是推的，被攻过零轮）：见 K2-4；自证加三格：relay-comma、relay-list 两种应得 0，own-zai 那种按 ㉗ 的定法判。

推翻条件：真派发提示里转述别的子 agent 时从不写「草稿目录 <路径>」——b9-dispatch-guard 的语料比法（3317 份首条提示）可以直接数这一形状；我没跑语料。

### 没打中：⑬ 阈值取下限之后，峰值表里真大的命令还报不报

查了什么：`probes/threshold_probe.py` 按文件路径导入草稿目录里拷的 `agent-watch.py`，直接调 `memory_threshold`（包装那一支换成没有、峰值换成造出来的数），`--process-rss-gibibytes 8`、`--process-rss-floor-mebibytes 512`，峰值从 0 扫到 20 GiB。`out/threshold_probe.out` 原样：

```
峰值      0.000 MiB → 阈值     8192.0 MiB（fixed）
峰值      0.000 MiB → 阈值      512.0 MiB（peakfloor）
峰值      0.004 MiB → 阈值      512.0 MiB（peakfloor）
峰值    100.000 MiB → 阈值      512.0 MiB（peakfloor）
峰值    300.000 MiB → 阈值      512.0 MiB（peakfloor）
峰值    341.000 MiB → 阈值      512.0 MiB（peakfloor）
峰值    400.000 MiB → 阈值      600.0 MiB（peak）
峰值   2048.000 MiB → 阈值     3072.0 MiB（peak）
峰值   5120.000 MiB → 阈值     7680.0 MiB（peak）
峰值   6144.000 MiB → 阈值     9216.0 MiB（peak）
峰值  20480.000 MiB → 阈值    30720.0 MiB（peak）
```

（第二行是峰值 1 字节，按 MiB 打出来是 0.000。）真大的命令照报，阈值是峰值的 1.5 倍；下限 `scripts/agents/agent-watch.py:2676`：「floor = min(floor_mebibytes * BYTES_PER_MEBIBYTE, fixed_bytes)」只抬不到下限的那一截，碰不到大命令。峰值过了 8 GiB 的三分之二（约 5.3 GiB）的命令阈值高过固定线 8 GiB，这是第九批之前就有的（峰值那一支本来就不封顶），不算这一批的。

记作观察、不算打中：峰值正好是 0 的条目走 `scripts/agents/agent-watch.py:2671`：「if peak:」之外的固定线 8 GiB，峰值 1 字节的取下限 512 MiB，阈值随峰值不单调。峰值表按 MiB 向上取整记（表文件头写的口径），正好是 0 的条目很少；要堵就把 `if peak:` 改成 `if peak is not None:`，让 0 也走下限（推的，被攻过零轮）。推翻条件：峰值表里确有成批的 0 条目、且这些命令真会涨到几 GiB。

### 没打中：⑭ 看门狗怎么认自证临时目录（读代码推的，没起真进程）

查了什么：`light_selftest_scratch_owner` 从写文件的进程往上找登记成 light 的自证（`registered_light_selftest`，bash -c、python -c / -m 起的不认），找到才看路径；`private_temporary_base` 往上跳过参数以它的参数收尾的包装层，到第一层不是包装的、比 TMPDIR（`scripts/agents/agent-watch.py:2219`：「return launcher != base」）；共用 TMPDIR 时只认下一层名字像 mktemp 的（`scripts/agents/agent-watch.py:2329`：「SELFTEST_TEMPORARY_NAME = re.compile」）或两个探查前缀。试的形状（都是推的）：真长跑写在 `/tmp/claude-1000/…/tasks/*.output`、父进程没设 TMPDIR——不是自证的后代，照报；light 自证真红写进 Bash 工具的输出文件——那份文件不在它的临时目录下一层、父子 TMPDIR 相同，照报；子 agent 用 `TMPDIR=<自己的草稿目录> bash x.sh --selftest` 起——整个草稿目录算它的，但只豁免它自己的后代写的文件，误放不到别的长跑；heavy-if-set 的自证带变量起照算 heavy、不豁免。没找到能让一条真长跑的 ✗ 被豁免的形状。和上层共用 TMPDIR、自己起名 mktemp 的照报，九十节 ⑭ 去向列已记，不重复。推翻条件：一条不是 light 自证后代的进程写的 ✗ 被写成「✗ 不报」。

### 没打中：`merge-and-verify.sh` 判「补丁没碰 checker 档闭包」

靠什么：`scripts/gate/merge-and-verify.sh:477`：「closure, frontier = {checker}, [checker]」起，沿成员 `Cargo.toml` 的 dependencies、dev-dependencies、build-dependencies（连同 `[target.*]` 下的）往下收；补丁里每个路径按 `scripts/gate/lib_rebuilt_packages.py` 的 `package_of`（往上第一份带 `[package]` 的 `Cargo.toml`）归包；不落在包里的、改了工作区根的、追加了变异行的都照跑。

`probes/merge_select_probe.sh` 把 `PY_SELECT` 那段 python 原样截出，在真树上（只读）喂四份造出来的补丁目录。`out/merge_select_probe.out` 原样：

```
unit-graph → CHECKER	affected	补丁改到 checker 档包或它依赖的包：singlefs-unit-graph
experiments → CHECKER	unaffected	补丁只改到 e7-index-bench，都不是 checker 档包 singlefs-checker-tier、也不在它的依赖闭包里，快档测试函数的单元键不会因这一批变
empty → CHECKER	unaffected	这一批补丁目录里没有改到文件的 crates.patch
harness-tests → CHECKER	affected	补丁改到 checker 档包或它依赖的包：singlefs-harness
工作区成员 8 ；checker 档包依赖闭包 8 ；闭包外的成员 []
```

本仓 8 个成员全在闭包里（checker 档包直接依赖其余 7 个，`crates/singlefs-checker-tier/Cargo.toml:17`：「singlefs-unit-graph = { path = "../singlefs-unit-graph" }」是其中一行），所以「补丁只改到闭包外的包」这一支在 `crates/` 下永远不会走到。被问的「闭包外被测试读到的文件」：checker 档测试与构建脚本读到的 `crates/` 之外的数据文件（`crates/singlefs-checker-tier/src/crash_identity.rs:950`：「!path.starts_with("crates/")」那一段取的 watched_files，今天至少有 `.claude/heavy-thread-default`）不进 `crates.patch`；真进了补丁、不落在包里的，判「affected」照跑。所以误报本次未跑只剩两种：补丁只改 `scripts/experiment-apparatus` 下的包（独立的 cargo 包，checker 档源码里提到它的都在注释里，我只 grep 了提及、没逐行核），或补丁目录里 `crates.patch` 是空的。后一种之下，同一批里主 agent 自己直接写进主工作区的 `crates/` 改动（定义第 10 条「主 agent 自己写的 `crates/` 改动算进同一批」）补丁里看不见，会报「本次未跑」（推的，没跑整条 merge-and-verify）。

记作观察：闭包判法在本仓上落空，这一步除 `--copy`、`--peer`（九十节 ㉓ 已记）与上面两种之外每趟都起。改法（推的，被攻过零轮）：补丁目录里一个改到文件的 `crates.patch` 都没有时不判「本次未跑」，改判「照跑、由那一道按键复用」，省下的只是一次键对比。推翻条件：成员依赖表改了，出现闭包外的成员。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| K2 结局账 | `records/` 下 `.txt`：directory-layout 四节、全部门禁阶段与钩子里按目录认 `records/` 的写法、一份真跑的 `outcome --summary` 输出 | `grep` 全部 `.claude/gate.d/`、`.claude/hooks/`、`scripts/doc-lint.sh`；一天窗口 197 个子 agent 的输出 |
| K2 共用约束与交回闸文件头 | ⑤ ⑥ 两样逐句并排 | 共用约束那一条与 `handback-guard.sh` 第 26 到 38 行 |
| K2 ⑤ 与 rsync 那一支 | 共用约束「命名检查交回前跑」、实现员定义建副本的写法、交回闸判树的写法 | 三处原文；探针一格（worktree 里改、根写主工作区） |
| K2 实验号目录登记的位置 | directory-layout「测试的定义与输入跟着测试放」两句；同类登记 `.claude/kb-directories` 的放法 | 一份规则、一处先例 |
| K3 ⑥ | 被钩子拒、根在别的树、`bash -n`、`"$CLAUDE_PROJECT_DIR"` 作根 | 探针 10 份会话记录 |
| K3 ⑬ | 峰值 0、1 字节、4 KiB 到 20 GiB 共 11 个点 | 只调 `memory_threshold`，没起真进程 |
| K3 ⑭ | 非自证后代写的 ✗、自证真红写进 Bash 输出文件、子 agent 自设 TMPDIR、heavy-if-set 带变量 | 读代码推的 |
| K3 merge-and-verify | 补丁改 unit-graph、harness 测试文件、`scripts/experiment-apparatus`、空 `crates.patch` 四份 | 真树 8 个成员 |

## 这条腿自己的限度

- 探针都是喂拷进草稿目录的钩子与脚本，不是真派发、真交回：派发闸用 `--check-prompt`（不建锁、不判 ⑦ ⑯），交回闸喂造出来的 hook 输入与会话记录（`CLAUDE_PROJECT_DIR` 指到假仓），merge 那一步只截了挑包那段 python、没起整条 merge-and-verify（它是 whole，子 agent 不起）。
- K3-1、K3-2 的误判只在我造的输入上量过；真会话记录与真派发提示里这几种形状出现过几次，我没扫（b9 两份交回的语料比法能直接数）。
- ⑭ 没起真进程，只读代码推；⑬ 没起真进程，只调了阈值函数。
- 别的会话的改动（⑫ 收严、根分区告警、merge-and-verify 每趟自己的编译目录、主工作区 cargo 那一句、定义第 11 条）没判。
- 一条腿只抽一次样：上面每个「没打中」都不能单独支撑「没有问题」（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。

## 没做什么

- 没判 K1（正推那一格）；没替主 agent 采纳任何改法；上面每个改法都是这一腿提的，被攻过零轮，量过的只有误判本身（`out/` 四份），改法都没实现、没跑。
- 没编译、没起 cargo、没跑门禁阶段与各类 lint、没跑任何重型测试；没跑被判钩子与脚本的 `--selftest`。
- 没改仓里除报告与模型目录之外的任何文件。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/landing-audit-r4-opus/`：拷进去的钩子树 `tree/`（11M）、两趟复跑的工作目录 `rerun1/`、`rerun2/`（各 11M）、假仓与假会话目录（共约 116K）交回前已删；留下的只有探针草稿、`outcome-probe.out` 与两份复跑日志（共 172K），没有仓副本、编译目录与库拷贝。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/landing-audit-r4-opus-output.md --background research/prompts/_landing-audit-r4-background.md` 末行原样：

```
  ✓ 核了 37 处引文、4 处裸行号，对上 37 处，没判 4 处（其中裸行号没有引文可比 4 处；逐处列在上面）
```
