# governance-defs-r2 云端攻方（Opus）报告

攻击面：G1、G2、G3 为主，G4、G5 余力（背景材料第四节分工表「云端攻方」一行）。前几轮判决：`research/prompts/governance-defs-r1-main-verification.md`，这一轮不重复它 A1–A10 的角度。
开工 2026-09-26 15:15 UTC 核过快照：`sha256sum -c research/prompts/governance-defs-r2-snapshot/sha256sums.txt` 42 行无一不符。负载：开跑前 `ps -o pid,args -u "$(id -u)"` 里没有 cargo、gate.sh、qemu、fio、vm-bench。
没跑任何重型测试；钩子的判定全用合成的 PreToolUse JSON 喂真钩子，59 号与 `mutate.sh` 的判档逻辑从今天的源文件里按锚点行现抽再喂合成输出。

## 复跑

```
bash research/prompts/governance-defs-r2-opus-model/run-all.sh <草稿目录>
```

输出原样存在 `research/prompts/governance-defs-r2-opus-model/run-all.out`（213 行；其中临时目录名每次不同，按字段比）。模型目录 41 个文件：

```
73e832d00e643b7462ee900f1ab1a7fca346af5a4b019d97557cc07992f9e42e  ./cases/g1-main-54-full-three-lines.cmd
a3aba4b97fd5b052659d23fc808419c5779d8a552829f05800bda0fa2549b666  ./cases/g1-triage-54-direct.cmd
c2944fa13c110f07e14ffea0195544a017021bafc6ce285716b54950319ae975  ./cases/g1-triage-87.cmd
15795354bd415e6463230e384341d08d485b61531d5e81958c1b995fd716dcf1  ./cases/g1-triage-gate-full.cmd
d5e62bbb32bba2233e4f8e06062c04c1923a8eab56bbac11ba1249e4b20f92c3  ./cases/g1-triage-gate-staged-logged.cmd
a23c356e59e879fef2b0290e55b3d5fe8da4bb0cdef1c9a90eba1152c542497d  ./cases/g1-triage-gate-staged-noprefix.cmd
1a1854b01604ca135f3392412ad0c1fc6318064b391082e9c8d6bfd8df133ae3  ./cases/g1-triage-gate-staged-request.cmd
d0d89ec18d5f8dc77186ce0f028b2f0b7745d8151adfaf2bc8bb7bc6e9f2a20b  ./cases/g1-triage-gate-staged.cmd
d5aefdb11766e28c511e66155a8b635f3762e37148fead3786d361904c40a356  ./cases/g2-verifier-55-qemu-first-transaction-logged.cmd
a8a2b4d62173c5be80d1b8c806972ac65de17fe7a1979180acc695cff87abb49  ./cases/g2-verifier-55-qemu-first-transaction.cmd
87aca0530bfe7a1f9ed55a14a78439f45a935fe5d8442363a848d8bc1e27a3f7  ./cases/g2-verifier-57-lkmm-logged.cmd
0c40dce73fed5a0c72ce8676cf7792ef79918f95a1cd63a6fff190080a7f968d  ./cases/g2-verifier-57-lkmm.cmd
94cf51ca564a6c5e977b32272afb797447837c546d032f14a2139d75c9e24325  ./cases/g2-verifier-59-crates-mutation-replay-logged.cmd
97450422e5cfca346832619da874fe7ff66c7ebf7805973a0cf5c8bdf960fdbb  ./cases/g2-verifier-59-crates-mutation-replay.cmd
c86b8093d2892899ab74b39fece3ee51b9760d401b4885703056cac9b401a4b6  ./cases/g2-verifier-74-model-differential.cmd
8774f86313ba8dbfb8210f83a32b86a14fc50332bdda3f0cec2966271697c6d7  ./cases/g2-verifier-77-test-environment.cmd
c254ff6d43b64b03b29f55b9bd4fa9e91497ce3e5c0ceefb85fb55667465f2d5  ./cases/g2-verifier-94-checker-implementation-disjoint.cmd
ace3c0c334b673e9cf94b4cfff7275cbec3d37d7f238a8699ae1b5ba94cf6df7  ./cases/g2-verifier-diff-quiet.cmd
7cf8dbb66947676fdc2582c3b5035b690c7417f4376b5c58765f27df8f57c41f  ./cases/g2-verifier-ls-others.cmd
429bada5cc98b941e0385397c914cee241173656e2e3e9a0452cd9db755c29a3  ./dispatch/crash-verifier--main-agent-row-copied.txt
d32a36f2f8ede72a61bcc421ac998ba3c1b52be26dbd94cb1737d6a59ec99718  ./dispatch/crash-verifier--with-diff-check.txt
fa006e2cce4a64cd1bdc8d4f36eaea9990ceabc4cc4f1603f25f51c86b7d1d36  ./dispatch/gate-triage--main-agent-row-copied.txt
1f89d7f0799139b7edc3a8956510ddbe9a5aeaf793086bcc78a3c7ac6efb9d74  ./dispatch/gate-triage--minimal.txt
281a45abaa7a9e4a7ab5b424224978a88ab460177b2b9b7312c4948d407cfdda  ./dispatch/gate-triage--only-57-clause.txt
9c212616923d5f53703483c1c5f748fde8a6679f82e29df8f55ef9b0cb652742  ./dispatch/gate-triage--pre-change-row-copied.txt
18390f1d937799bb94af11e6186e633c6278f06d8a43b1edd658148f69b6c6c2  ./feed-dispatch.sh
2fc4886fbaa54691a76978951217ab242b978f7327e86375f89ee9f5677182c2  ./feed.sh
d712ef5e214ac811da7f9773ed87fb616468f0828cd52a6b5ec9bc66507bcc0a  ./g1-exit-status-fixed.sh
ccd69c150c38433724020e763afea6820d9ddf8bd5029eedd2cc9405395b59e5  ./g1-exit-status.sh
a068cff97a9b38d651025447b4e4147f332ccd87972427118aa51e5b7e527cf4  ./g2-cargo-discovery.sh
159e2ee7ec545f9305204489b083652e8a38bf3f9c4ea7dd534576e59f67ef0d  ./g2-diff-check.sh
2dd72e0ee5c06a267d72ee77cfe64b86725f5fb46102eab6146f43cfe681bbd2  ./g2-surface-59-copyset.sh
6755336fdf0a26132d1e865df21e225ec41031ab893ba52b9571dfdd0d025df1  ./g2-surface.sh
f2878901b4e6009a1b83a20b00baefd6cc8b9acf8f76a580ccf92a93c7453bba  ./g3-classify.py
39d17634cae5e79418a5ef4a8909e7331b66265a280990e65b00e6fe14578874  ./g3-outputs/compile-error.txt
adc7ca1965a50d23e79a26bacc0d52ded25b3cf4d7bcc20d7abce2a169e29fa3  ./g3-outputs/crash-after-named-test-passed.txt
1cb30228dfd0b50e6014b63f8e13a4be24c1fb1069ec790150ff843a38e6bce0  ./g3-outputs/crash-before-named-test.txt
30d0b0c6ced7a012bbfd7352a1fde8d6d6b3d9acd34dffd78d37d9541a813eac  ./g3-outputs/named-test-failed.txt
659a816fccf1db87973e1d53783b29345c560f564a519b0c8e903c6bccf92451  ./run-all.out
737f4d071bf95f3d4a4c248c9177146fe349b69016fde6389ae7cd37ad8038b1  ./run-all.sh
2f72c37c51e9f7b95bbcdc7096e805ac9e3f90fc77d20949a91becdbcfbfaaa0  ./run-hooks.sh
```

`g2-cargo-discovery.sh` 在草稿目录里拷一份 `crates/` 跑 `cargo metadata --no-deps --offline`（不编译）；其余只跑 bash、python3 与 git。模型不碰仓里任何文件，临时仓都建在给定的草稿目录下。

## 各格判定一览

编号用 B 族（上一轮是 A1–A10）。「分辨」指这条历史让改前、改后两种写法判得不同；不分辨的照背景材料第五节末那条另记，不拿它判这一批。

| 编号 | 格 | 打中了什么 | 分辨改前改后 | 我的判定 |
|---|---|---|---|---|
| B1 | G3 | 59 号没有 `💥`、`⚠️` 两栏：测试进程没跑完的那一条落进「无效」（点名测试没打出行）或「没红」（打出了），两种都判整道红；改后两份文字说 59 号「那一行各栏都有」「`💥` 不判失败」，在 59 号那一侧都不成立 | 部分：两句新话是这一批加的；把崩溃落进「无效」是 59 号自己的行为，改前改后都没写 | 站不住（59 号那一侧） |
| B2 | G2 | 核对命令的六条路径对 57、59 过宽：别的会话动了 `research/scripts/` 下 63 个 59 号不读的已跟踪文件之一（例：执行员第 5 步自己就要改的 `replay.sh`），三道一起停；主 agent 没有接这个停的分支，共用约束又不许碰别人的改动，提交流程卡住 | 是：改前照跑 | 站不住 |
| B3 | G2 | 同一条核对命令漏了 57 号读的 `.claude/scripts/lkmm.sh`、三道各自的阶段脚本，以及 cargo 自动认作目标的未跟踪 `src/bin/*.rs` / `tests/*.rs`：退 0、照跑，判的不是要提交的那一批 | 否：改前同样判工作区；改法（这条 `git diff --quiet`）碰不到这几格 | 另记（改法碰不到打中的格） |
| B4 | G1 | 54 号打印的三行放进同一次调用后，这次调用的退出码恒等于 `git worktree remove` 的：`--full` 红、内存包装退 250 / 251 时都退 0；「全量判绿之后才往下」没有退出码可依；每次还留下一个 `mktemp -d` 目录 | 部分：吞退出码改前就在；「判绿之后才往下」是这一批加的，把它变成了承重的一步 | 站不住（措辞与打印的命令要配齐） |
| B5 | G1 | 三处复用名单写「54、55、59、74」，87 号同样调 `stage-must-run.sh`、`stage-inputs.tsv` 有它一行 | 否：改前只写 55、59 | 另记（描述不全，改法顺手补） |
| B6 | G5 | 设计员「重跑登记照抄原登记的英文名」：仓里 16 份跑前登记与重跑登记，一份都没有「英文名」这几个字 | 部分：改前是「另定一个」，改后要求照抄一个不存在的东西 | 站不住（一步要的输入没人给） |
| B7 | G4 | 「腿开工时刻」没有哪份文件要求主 agent 记；给了快照、文件不在清单、没给开工时刻、内容对不上 ⇒ 落 ✗，同一处在没给快照的轮落「分不清」 | 部分：改前这一支连开工时刻的输入都没有 | 轻，另记 |
| G1 钩子 | G1 | 门禁分诊员、崩溃验证员照定义写的每条命令，主 agent 的三行，喂三个 Bash 钩子 | — | 构造不出被拒（38 次喂钩子，拒的只有本来就该拒的 4 次） |
| N1 | G1 | 主 agent 把 main-agent.md 那一行抄进派 gate-triage 的提示，派发闸拒（当成要它跑层 0 / herd7） | 否：改前那一行抄进去同样被拒 | 不拿它判这一批 |

另有一条观察（不打分，交主 agent）：O1，崩溃验证员在主工作区跑的 55、57、59，门禁分诊员的 `gate-staged.sh` 会在暂存树上全部再跑一遍，前一次的结果不被复用。详见 G2 一节末尾。

## G3 变异计数

### B1：59 号那一侧，「各栏都有」「`💥` 不判失败」都不成立

被判的句子（两处同文）：
- `.claude/agents/mutation-triage.md:28`、`.claude/rules/mutation-sampling.md:82` 都含「59 号照它自己收尾的「计数：」行，那一行各栏都有」与「`💥` 不判失败」。

脚本今天的行为：
- `research/scripts/mutate.sh:512` `if grep -q "process didn't exit successfully" <<<"$out"; then`，`:514` `record_verdict "$row_index" crashed 0 …`：`mutate.sh` 有 💥 一档，不判失败。
- `.claude/gate.d/59-crates-mutation-replay.sh:344` `if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:` 之下判成 `invalid`；59 号没有「测试进程没跑完」这一档。
- `.claude/gate.d/59-crates-mutation-replay.sh:407` 那一行「计数：」的栏是：抓到、没红、无效、内存撞顶、超时、被总上限挤掉、排不上没跑、scope 起不来没跑，没有 💥 与 ⚠️。
- `.claude/gate.d/59-crates-mutation-replay.sh:450` `if invalid or failures or memory_hits or timeouts or squeezed or refused or unavailable:` 之下 `sys.exit(1)`。
- `mutate.sh:496` 自己写着 `` `cargo test` 在测试变红时也会打 `error: test failed`， ``——59 号拿 `^error: ` 当编译失败的判据，正撞在这一句说的坑上：测试进程被信号杀掉时 cargo 照样打 `error: test failed, to rerun pass …`。

历史（模型 `g3-classify.py`，两边的判档段都从今天的源文件按锚点行现抽，喂同一份合成输出，退出码 101）：

```
compile-error.txt	59 号：invalid（整道判红：是）	crates/mutations.tsv:7 合成变异：替换文写进源码之后编不过（退出码 101）	mutate.sh：invalid（判整轮失败：否）
crash-after-named-test-passed.txt	59 号：failure（整道判红：是）	crates/mutations.tsv:7 合成变异：inode_and_extent_lookups_from_th	mutate.sh：crashed（判整轮失败：否）
crash-before-named-test.txt	59 号：invalid（整道判红：是）	crates/mutations.tsv:7 合成变异：替换文写进源码之后编不过（退出码 101）	mutate.sh：crashed（判整轮失败：否）
named-test-failed.txt	59 号：caught（整道判红：否）	  ✓ 合成变异：inode_and_extent_lookups_from_the_root_read_the_fir	mutate.sh：caught（判整轮失败：否）
```

照定义做会怎样：变异分诊员拿到一次 59 号输出，里面有一条变异让被测代码栈溢出（`crash-before-named-test`）。照 mutation-triage.md 第 4 步「编不过的在「有变异无效」一栏」、第 6 步「没红与无效的逐条按 `mutation-sampling.md` 分类」，它把这条记成第八类（替换文编不过），出路照 59 号自己那句「改这一行替换文，不是去补用例」——而真实发生的是「破坏被看见了、但不是断言抓到的」，`mutate.sh` 那一侧会记 💥 且不判失败。同一种结局在两边一个判失败、一个不判，文字却说 `💥` 不判失败、59 号各栏都有。

合成输出的形状：进程被信号杀时 cargo 打「`error: test failed, to rerun pass …`」与「`process didn't exit successfully: … (signal: 6, …)`」，这两句正是 `mutate.sh:496`、`:512` 自己依赖的形状；libtest 在多线程默认下不先打「`test 名字 ... `」，点名测试还没结束就崩时输出里没有那一行。这是合成的，没有真跑 cargo（这个容器里内存包装退 251，按定义也不跑 59 号）。

四问：
1. 分不分辨：部分。「那一行各栏都有」「`💥` 不判失败」不加限定，是这一批新写的，在 59 号那一侧为假；「崩溃落进无效」是 59 号的行为，改前改后的文字都没说。
2. 被判的系统看不看得到判别它的东西：看得到。59 号每条失败都附输出末 8 行（`tail`），里面有 `process didn't exit successfully` 与信号号；分诊员读得到，只是定义没叫它看。
3. 满足的是判据字面哪一句：背景材料第五节「站不住」那一条的「两处文件给出相反做法」——分诊员照文字认为 💥 不判失败，59 号实际判红，而且判进了另一档。
4. 改法在这几格上中不中（下表）。

| 改法 | 修哪一格 | 在 `crash-before-named-test` 上 | 在 `crash-after-named-test-passed` 上 | 量过 / 推的 |
|---|---|---|---|---|
| 甲：两份文字把「`💥` 不判失败」限定为 `mutate.sh`；59 号一句改成「59 号没有 💥、⚠️ 两栏：测试进程没跑完的，点名测试没打出行就落进「无效」、打出了就落进「没红」，两种都判红；分类前先看那一条的尾巴里有没有 `process didn't exit successfully`，有就按 💥 报，不按第八类」 | 文字与行为对上，分诊员不再误归第八类 | 不再中 | 不再中 | 推的（没实现） |
| 乙：59 号加一档「测试进程没跑完」，判不判红由主 agent 定 | 两边同一种结局同一档 | 不再中 | 不再中 | 推的；这是改脚本行为，越出「只改描述」，交主 agent |
| 只删「那一行各栏都有」 | 去掉一句假话 | 仍中：分诊员照第 4 步仍归第八类 | 仍中 | 推的 |

推翻条件：拿一条让被测代码 `abort` 或栈溢出的变异在 59 号里真跑一次（提交时的崩溃验证员），那一条若被判成 `caught` 或另有一档，B1 撤回。

## G2 崩溃验证员

### 钩子：照定义做的每条命令都放行

`run-hooks.sh` 把下列命令以 `agent_type=crash-verifier`、前台与 `run_in_background` 各一次喂 `pattern-process-guard.sh`、`bash-command-detector.sh`、`heavy-test-guard.sh`：第 1 步的 `git diff --quiet -- …` 与 `git ls-files --others --exclude-standard -- …`；第 2 步 55、57、59 的裸写法与 `{ …; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1` 写法（带 `SINGLEFS_HEAVY_TESTS=commit`）；登记给它的轻阶段 74、77、94（不带前缀）。三个钩子全部 `rc=0`，检出记录里没有一条是崩溃验证员的。构造不出「照定义做被钩子拒」。

### B2：六条路径对 57、59 过宽，别的会话一碰 `research/scripts/` 三道全停，而主 agent 没有接这一停的分支

被判的句子：`.claude/agents/crash-verifier.md:24` 要求 55、57、59「每个阶段开跑前跑 `git diff --quiet -- crates Cargo.toml Cargo.lock litmus research/scripts research/results`」，「不为 0 就不跑那一道，停下报告「工作区与暂存区不同，判的不是要提交的那一批」交主 agent」。三道用的是同一组六条路径。

各道真读什么：
- 55：`.claude/gate.d/stage-inputs.tsv:12` 一行是 `crates/ Cargo.toml Cargo.lock research/scripts/ research/results/`；`55-qemu-first-transaction.sh:159` 还从 `research/scripts/replay.sh` 里取 E142 产物名。
- 59：`.claude/gate.d/stage-inputs.tsv:13` 一行是 `crates/ Cargo.toml Cargo.lock research/scripts/run-with-memory-cap.sh`；往宽里算，`59-crates-mutation-replay.sh:176` 拷进每一片的是 `COPIED_INTO_EACH_SHARD = ["crates", "Cargo.toml", "Cargo.lock", "litmus", "research/results"]`。
- 57：没有 `stage-inputs.tsv` 一行；`57-lkmm.sh:14` 调 `.claude/scripts/lkmm.sh`，后者读 `litmus/` 并在 `crates/` 里 grep（`lkmm.sh:281`）。

历史（`g2-diff-check.sh` 的 S1，临时仓；核对命令从定义里现抽）：主 agent 暂存了一批 `crates/` 改动、派崩溃验证员；同一时刻别的会话的实验执行员照 `experiment-runner.md:32`「在 `research/scripts/replay.sh` 里登记复跑」往 `replay.sh` 追加了一行，还没暂存。

```
S1 这一批暂存了 crates/；别的会话的执行员往 research/scripts/replay.sh 追加了一行、没暂存
  定义那条命令退 1（三道都停下报告）
  55 号自己读的路径上工作区与暂存区：不同
  57 号自己读的路径上工作区与暂存区：同
  59 号自己读的路径上工作区与暂存区：同
```

放开「别的会话改的是哪一个文件」这一步（`g2-surface.sh`、`g2-surface-59-copyset.sh`，对今天六条路径下每个已跟踪文件逐个数）：

```
六条路径下已跟踪文件 324 个
  不是 55 号输入（stage-inputs.tsv 那一行）的：6
  不是 57 号输入（litmus/、crates/）的：189
  不是 59 号输入（stage-inputs.tsv 那一行）的：192
  三道都不读、改了照样让三道全停的：0
59 号拷进每一片的：crates Cargo.toml Cargo.lock litmus research/results research/scripts/run-with-memory-cap.sh
六条路径下已跟踪文件 324 个，不在 59 号这一宽口径里的：63
```

即：动这 189 个里任一个，57 被白停；动 `research/scripts/` 下 64 个已跟踪文件里 63 个中的任一个（按最宽的口径算），59 被白停。55 那一停是对的。

停了之后：`grep -c '工作区与暂存区' .claude/main-agent.md` → `0`，main-agent.md 那一行（第 59 行）只有「派 `crash-verifier` 跑 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit`」，没有「它报工作区与暂存区不同时怎么办」。主 agent 要让工作区等于暂存区，只能动别的会话没提交的改动，而 `.claude/agent-common.md:54` 写着「看到别人没提交的改动不碰、不修」。于是提交流程停在这一步，要等别的会话自己暂存或提交。

四问：
1. 分辨：是。改前崩溃验证员不核、照跑；改后三道全停。
2. 看不看得到：看得到。`git diff --name-only -- <六条路径>` 就列出是哪个文件不同，崩溃验证员有能力按道判，只是定义让它用同一组路径、退非 0 就停。
3. 满足的字面：第五节「站不住」那条的「照改后的定义做……做不下去」与「一步要的输入没人给」（主 agent 那一侧没有分支）。
4. 改法（下表，都没实现）：

| 改法 | 修哪一格 | S1 上 | 量过 / 推的 |
|---|---|---|---|
| 丙：按道取路径：55、59 取 `stage-inputs.tsv` 各自那一行加阶段脚本自己；57 另登一行（`litmus/ crates/ .claude/scripts/lkmm.sh`）再照取 | 57、59 不再被白停 | 55 仍停（该停），57、59 照跑 | 推的；S1 的「57 / 59 号自己读的路径…同」是按这个取法算的，那一半是量过的 |
| 丁：main-agent.md 补一句崩溃验证员报「工作区与暂存区不同」时怎么办（等、或不派崩溃验证员而只靠 `gate-staged.sh` 在暂存树上跑同几道，见 O1） | 主 agent 不再卡住 | 不停在这一步 | 推的；选哪一种是设计判断，交主 agent |
| 只收窄六条路径（例：去掉 `research/scripts`） | — | 55 就会漏掉 `replay.sh`，换成 B3 那一类 | 推的 |

推翻条件：57 号或 59 号实际还读 `research/scripts/` 下别的文件（例：包装读的 `research/scripts/memory-peaks.tsv`，59 号文件头第 41 行提到它），那几个文件从「白停」里减掉；只要仍有一个 59 号不读的已跟踪文件，B2 成立。

### B3：同一条核对命令退 0、照跑，而判的不是要提交的那一批（改法碰不到这几格）

被判的句子：`.claude/agents/crash-verifier.md:24`「（退 0 才说明工作区与暂存区在这几道的输入上相同）」，以及同一处「同一批路径下的未跟踪文件（`git ls-files --others --exclude-standard -- <同一批路径>`）原样列进报告」——未跟踪文件只列、不停。

历史（`g2-diff-check.sh` 的 S2–S4，与 S1 同一套临时仓）：

```
S2 这一批暂存了 .claude/scripts/lkmm.sh；工作区里那一份又被改了、没暂存
  定义那条命令退 0（三道都照跑）
  57 号自己读的路径上工作区与暂存区：不同
S3 这一批暂存了 59 号脚本；工作区那一份不同
  定义那条命令退 0（三道都照跑）
  59 号自己读的路径上工作区与暂存区：不同
S4 这一批暂存了 crates/；别的会话在 crates/singlefs-harness/tests/ 下新建了一个没跟踪的测试文件
  定义那条命令退 0（三道都照跑）
  未跟踪（定义要求原样列进报告、不停）：crates/singlefs-harness/tests/wip_from_other_session.rs 
```

S4 的后果（`g2-cargo-discovery.sh`，草稿目录里的 `crates/` 副本，`cargo metadata` 不编译）：

```
singlefs-harness ['bin'] e999_other_session_wip
singlefs-harness ['test'] wip_from_other_session
crates/mutations.tsv 里 cargo 参数带 -p singlefs-harness 的行：631（非注释非空行共 724）
```

cargo 把没跟踪的 `src/bin/*.rs`、`tests/*.rs` 都认成目标；59 号拷进每一片的是整个工作区的 `crates/`（`:176`），跑 `-p singlefs-harness --test …` 的 631 行都会连带编译这个包的 bin。别的会话的实验执行员正好被允许在那里新建文件（`.claude/hooks/agent-write-scope.tsv:12`：`experiment-runner	crates/singlefs-harness/src/bin/e*.rs`）。那个文件还没写完、编不过时，59 号在工作区上判「无效」一大片，与这一批无关；定义只要求把它列进报告，没说列出来之后这一道的结果算不算数。

四问：
1. 分辨：否。改前崩溃验证员同样在工作区上跑、同样判到这几样；改后多了一句「退 0 才说明……相同」，在这三格上不成立。
2. 看不看得到：看得到（`git diff --cached --name-only` 与 `git ls-files --others` 都在它手里）。
3. 字面：不按第五节判这一批；照背景材料第五节末那条另记。
4. 改法：这一批给的改法（这条 `git diff --quiet`）在 S2、S3、S4 上都还中——「对这几格不起作用」。把 57 的 `lkmm.sh`、各道阶段脚本加进路径、未跟踪的 `src/bin`、`tests` 也判停，能修 S2–S4（推的）；但加得越多，B2 那种白停越多。两头都顾到的只有 O1 那条路：在暂存树上跑。

### O1（观察，不打分）：崩溃验证员的 55、57、59 与 `gate-staged.sh` 里的同几道重跑

- `gate-staged.sh` 跑 `gate.sh --staged`，后者在临时 worktree 上把 `.claude/gate.d/` 下各阶段逐个跑一遍（`.claude/singlefs-ai-sop/scripts/gate.sh:415` `GATE_D="$ROOT/.claude/gate.d"`）。
- 55、59 能不能跳过只看 `stage-must-run.sh`：`research/scripts/stage-must-run.sh:52` 没有 `SINGLEFS_STAGED_TREE` 一律要跑；有也只拿 `refs/sop/staged-green`（只有整轮全绿的 `gate-staged.sh` 才前移）比。崩溃验证员在主工作区的那一次不设这个变量、也不前移这条 ref。57 号 `stage-must-run.sh` 命中 0 次，每次现跑。
- 所以一批动了 `crates/` 的改动，55、57、59 在崩溃验证员手里跑一次（工作区），在门禁分诊员的整轮门禁里再跑一次（暂存树），前一次不被后一次复用。59 号「要几个钟头」（`.claude/gate.d/stage-owners.tsv:19` 那一行的理由列）。
- 这不是这一批的改动造成的：改前的文字「55、59 照 `stage-inputs.tsv` 复用上一次全绿判定」让人以为会复用，改后的文字（「只在这一趟里能复用上一次整轮全绿的判定」）说对了，重复因此看得见了。B2、B3 两头为难的根子也在这里：崩溃验证员在工作区上跑，核得再严也只能逼近暂存树，而同一道本来就会在暂存树上跑。要不要保留崩溃验证员这一次，是设计判断，交主 agent。

## G1 整轮门禁与复用

### 钩子：门禁分诊员与主 agent 照定义写的命令都放行

`run-hooks.sh` 以 `agent_type=gate-triage` 喂：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/gate-staged.sh`（gate-triage.md 第 2 步原样）、`=user-request` 版、写日志的 `{ …; echo "exit=$?"; } > … 2>&1` 版、全量 `bash .claude/scripts/gate.sh`、87 号，前台后台各一次，三个钩子全 `rc=0`。对照两条该拒的：不带前缀跑 `gate-staged.sh`、直接调 54 号，`heavy-test-guard.sh` 各拒两次（`run-all.out` 里原样：「gate-triage 不跑「层 0」」「要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带」）。主 agent（不带 `agent_type`）照 main-agent.md 把 54 号打印的三行放进同一次调用，前台、`run_in_background` 都三个钩子 `rc=0`、检出记录里没有它。`gate-staged.sh` 里面起的 `gate.sh --staged` 与各阶段是它的子进程，钩子只判顶层那条命令，不再判里层——钩子眼里整条算门禁分诊员的「整轮门禁」一类。

### B4：三行放进同一次调用，这次调用的退出码恒为 0（`--full` 红、内存包装 250 / 251 都一样）

被判的句子：`.claude/main-agent.md:59`「三行放进同一次 Bash 调用」与「全量判绿、全绿标记写好之后才往下，不然整轮门禁里的 54 号必红」；打印处 `.claude/gate.d/54-layer0-replay.sh:230`（「把下面三行命令放进同一次 Bash 调用」）与 `:233`，第三行以 `; git worktree remove --force "$layer0_full_base/tree"` 收尾。

历史（`g1-exit-status.sh`：三行从今天的 54 号现取，只把 54 号脚本与内存包装两个名字换成桩，在临时仓里放进同一次 `bash -c` 跑；换名是为了模型不碰层 0 与内存包装本身）：

```
green	这一次 Bash 调用的退出码：0	输出：stub-full 在 <草稿目录>/tmp.SM5RbrWFtw/tree 上跑，退 0 	剩下的 worktree 数（含主树）：1
red	这一次 Bash 调用的退出码：0	输出：stub-full 在 <草稿目录>/tmp.V5ZdrnNsAk/tree 上跑，退 1 	剩下的 worktree 数（含主树）：1
memory-cap-250	这一次 Bash 调用的退出码：0	输出：stub-memory-cap：退 250，命令一行都没跑或被杀 	剩下的 worktree 数（含主树）：1
scope-unavailable-251	这一次 Bash 调用的退出码：0	输出：stub-memory-cap：退 251，命令一行都没跑或被杀 	剩下的 worktree 数（含主树）：1
```

照定义做：主 agent 在 `run_in_background` 里起这三行，完成通知报退出码 0；它照「判绿之后才往下」派崩溃验证员，而 `--full` 其实判了红、或者内存包装撞顶退了 250（共用约束要求 250 照实报）、或者 scope 起不来退了 251（「一行都没跑」）。不会造成错误的绿：整轮门禁里 54 号快档找不到这批输入的全绿标记照样判红；代价是崩溃验证员那几个钟头白跑、250 / 251 没有被当场报出来。另外每跑一次，`mktemp -d` 建的那个目录连同 `staged.patch` 留在 `/tmp`（模型里四段历史留下四个）。

四问：
1. 分辨：部分。吞退出码改前就在（第三行的 `;` 不是这一批加的）；「全量判绿……之后才往下」是这一批新写的，让这一步变成了必须判准的一步，而给出的命令判不出来。
2. 看不看得到：输出里看得到（`--full` 的 ✓ / ✗ 行、包装的报错），退出码看不到。main-agent.md 没写「判绿」看哪一样。
3. 字面：第五节「站不住」那条的「一步要的输入没人给」（判绿的依据）。
4. 改法：

| 改法 | 修哪一格 | green / red / 250 / 251 上这一次调用的退出码 | 量过 / 推的 |
|---|---|---|---|
| 戊：打印的第三行改成 `…--full "$layer0_full_base/tree"; layer0_full_rc=$?; git worktree remove --force "$layer0_full_base/tree"; rm -rf "$layer0_full_base"; exit "$layer0_full_rc"` | 退出码带出 `--full` 的结局，临时目录不留 | 0 / 1 / 250 / 251，剩下的临时目录 0 | 量过（`g1-exit-status-fixed.sh`，原样输出见下）；只在我的模型上量过、被攻过零轮 |
| 己：main-agent.md 写明「判绿」看 `--full` 的 ✓ 行、不看退出码 | 主 agent 知道看哪 | 退出码仍 0 | 推的 |

```
green	改后那一次调用的退出码：0	剩下的临时目录：0	剩下的 worktree 数（含主树）：1
red	改后那一次调用的退出码：1	剩下的临时目录：0	剩下的 worktree 数（含主树）：1
memory-cap-250	改后那一次调用的退出码：250	剩下的临时目录：0	剩下的 worktree 数（含主树）：1
scope-unavailable-251	改后那一次调用的退出码：251	剩下的临时目录：0	剩下的 worktree 数（含主树）：1
```

戊末尾的 `exit` 在前台 Bash 调用里会结束那个 shell；换成 `( exit "$layer0_full_rc" )` 效果相同、不退出 shell（推的，没量）。改了这一行要连带改 54 号自证里对打印内容的断言（没查有没有这种断言）。

推翻条件：Claude Code 的 `run_in_background` 完成通知不报退出码、主 agent 本来就只看输出——那 B4 只剩「判绿看什么没写」一半。

### B5：复用名单漏了 87 号

`.claude/main-agent.md:59` 与 `.claude/rules/implementation-workflow.md:55` 写「54、55、59、74 号只在这一趟里能复用上一次整轮全绿的判定」，`.claude/agents/gate-triage.md:26` 写「54、55、59（还有 74）号只在 `gate-staged.sh` 那一趟里能复用」。今天调 `stage-must-run.sh` 的（`grep -c`）：54 号 2、55 号 2、57 号 0、59 号 2、74 号 3、87 号 2；`87-replay.sh:26` 就是那一行 `reuse_reason="$(bash …/research/scripts/stage-must-run.sh" "$ROOT" "$(basename "$0")")"`，`.claude/gate.d/stage-inputs.tsv:15` 有 87 号一行。门禁分诊员在 `gate-staged.sh` 那一趟里看到 87 号「本阶段跳过（复用上一次整轮全绿的判定）」退 77，文字说 87 不在复用之列。后果轻（77 本来就记「本次未跑」）。不分辨（改前只写 55、59，同样漏）；改法：三处补上 87（推的）。

### G1 其余几问：构造不出

- 派发次序：「全量判绿之后才往下」与崩溃验证员、门禁分诊员的定义都不冲突——两份定义里都没有「先于主 agent 的全量」的要求；崩溃验证员的三道不读全绿标记。
- 五处说法（main-agent.md:59、gate-triage.md:26、implementation-workflow.md:55、heavy-test-guard.sh 文件头与 POLICY、gate/SKILL.md）：除 B5 那份名单外，谁跑什么、带什么前缀、`gate-staged.sh` 跑 `gate.sh --staged`、复用只在那一趟里有，五处一致；钩子实测与文字一致（上面那一小节）。
- 另一问退 77 算不算「复用」：五处都没把 `change-touches-crates.sh` 那一问叫「复用」，门禁分诊员照共用约束把 77 记「本次未跑」，不冲突。

### N1（不分辨，另记）：main-agent.md 那一句抄进派发提示会被派发闸拒

```
gate-triage--main-agent-row-copied	rc=2	✗ 派 gate-triage 的提示要它跑「层 0」：「要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑」
gate-triage--only-57-clause	rc=2	✗ 派 gate-triage 的提示要它跑「herd7」：「57 号没有复用、每次现跑」
gate-triage--pre-change-row-copied	rc=2	✗ 派 gate-triage 的提示要它跑「层 0」：「gate-triage 带 SINGLEFS_HEAVY_TESTS=commit 跑 gate.sh --staged 并分诊（gate.sh 里 54 号跑快档并核全绿标记，55、59 �
```

改前那一行抄进去同样被拒，只写「带前缀跑 `gate-staged.sh` 并分诊」的放行（`gate-triage--minimal rc=0`）。这是派发闸对说明句的误拒（它文件头「会误拒」那一段已登记这类形状），不拿它判这一批。

## G5 其余（余力）

### B6：重跑登记「照抄原登记的英文名」，而原登记里没有这一行

被判的句子：`.claude/agents/experiment-designer.md:28`（第 2 步）「占号之后定一个英文名……写在登记 `## 一、问题` 的第一行「英文名：…」……重跑登记照抄原登记的英文名」；`.claude/agents/experiment-runner.md:27`「源文件、变异表与 bin 名一律用跑前登记 `## 一、问题` 第一行定的英文名」。

现查（2026-09-26 15:29 UTC 前后）：`ls research/prompts/e*-preregistration.md research/prompts/e*-r*-prereg.md | wc -l` → `16`；其中含「英文名」三个字的 `grep -l '英文名' … | wc -l` → `0`。已归档的更早登记同一时期写成，同样没有这一行（没逐份翻 git，推的）。

历史：主 agent 要重跑 E156（入库装置 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`），派设计员写 `e156-r4-prereg.md`。设计员照第 2 步去原登记 `research/prompts/e156-preregistration.md` 抄英文名——没有；定义没给出路。它自己起一个（例 `alloc_basis_counts`）写进第一行，执行员照第 2 步「一律用……第一行定的英文名」就会去写 `e156_alloc_basis_counts.rs`、`e156_alloc_basis_counts.tsv`，与已入库的源文件名、`replay.sh` 里 `driver_e156` 调的 `--bin e156_allocation_basis_counts` 对不上。

四问：分辨——部分（改前是「另定一个」，同样可能对不上，但不叫它抄一个不存在的东西）；看得到——看得到（原装置源文件名就在仓里）；字面——「一步要的输入没人给」；改法——「原登记没有这一行的，取原装置源文件名 `e<号>_<英文名>.rs` 里那一段」（推的，零轮），在这一格上不再中。

推翻条件：设计员的重跑登记另有一份仓内的英文名来源被定义点名（我没找到）。

### G5 其余几处：构造不出

- 书记员「N 写阿拉伯数字或汉字数字都认」：75 号的正则是 `[0-9一二两三四五六七八九十]+[项条]未定`（`.claude/gate.d/75-decision-experiment-links.sh` 里那一行），20 号只认 `——\s*(已定|半定|待定)`；今天的标题形如「—— 半定（一项未定）」「—— 半定（三项未定）」，两道都认。百以上（「一百」）不认，决策分项到不了那个数，没当成打中。
- 执行员「入库装置不写 `[[bin]]`、bin 名就是文件名」：`research/scripts/replay.sh` 的 `driver_e156` 调 `cargo run -q -p singlefs-harness --bin e156_allocation_basis_counts`，与文件名一致。
- 共用约束「写」「门禁」两处改动、执行员 7b ②：读了，没造出照做出错的历史。

## G4 核查员（余力）

### B7：「腿开工时刻」没人被要求记；少了它，「文件不在清单里」那一支落 ✗

- 被判的句子：`.claude/agents/three-way-verifier.md:22`「腿开工时刻（UTC）……没给就不查那一支，照实写没查。」与第 2 步（第 28 行）「给了快照、而这个文件不在清单里的，对主树核，并用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查……」。
- 谁给：`grep -n '开工时刻' .claude/main-agent.md .claude/rules/implementation-workflow.md .claude/rules/three-way-inference.md` 零命中；implementation-workflow.md「代码轮派腿之前记一份开工快照」只要求记 `sha256sum` 快照，不要求记时刻。
- 四种情形各走哪一支：给了快照且在清单里、对得上 ⇒ 主树；对不上 ⇒ 倒推副本，没有就「分不清」；给了快照、不在清单里 ⇒ 主树 + 现查改没改（要开工时刻）；没给快照 ⇒ 代码轮停下要、设计轮对主树核、对不上记「分不清」。每种都有一支，没有落进两支的。
- 缝在这里：给了快照、文件不在清单里、主 agent 没给开工时刻、内容与腿抄的对不上——「现查」那一半不做，剩下的是第 2 步「对不上时再找原文实际在第几行」，落 ✗；同一处引用若这一轮根本没给快照，落「分不清」。给了快照反而更严。这一轮就是例子：`.claude/scripts/lkmm.sh`、`57-lkmm.sh`、`55-qemu-first-transaction.sh`、`87-replay.sh` 我都引了，都不在这一轮的快照清单里（`grep -qF` 逐个核过）。
- 另有一处改前就有的：输入里没有「这一轮是代码轮还是设计轮」，改定义的轮（implementation-workflow.md「改它们与改 `crates/` 同规矩」）没给快照时落哪一支，文字判不出。不分辨。
- 改法：implementation-workflow.md「记一份开工快照」那一节同时记开工时刻（例：快照文件自己的提交或 mtime），核查员输入改成「快照给了就必给开工时刻」（推的，零轮）。

## 没打中的形状

| 攻击面 | 试过的形状 | 取样范围 | 结果 |
|---|---|---|---|
| G1 钩子 | 门禁分诊员 6 种命令、崩溃验证员 12 种、主 agent 三行 1 种，各前台 / 后台 | 19 条 × 2 = 38 次，每次 3 个钩子 | 该放的都放、该拒的（2 条对照）都拒 |
| G1 派发闸 | 派崩溃验证员的 2 种提示、派门禁分诊员的 4 种 | 6 条 | 只有 N1（不分辨） |
| G1 复用 | 59 号 `stage-inputs.tsv` 一行没登 `litmus/`、`research/results/`（59 号却把它俩拷进每一片）能不能让 59 号在 `gate-staged.sh` 里被错跳 | `crates/mutations.tsv` 里点名读 `litmus/` 的测试（`publish_order_matches_litmus`）0 行；crates 的测试在运行期读 `research/results/` 的 0 处（只在注释里） | 构造不出错跳；登记比拷贝窄，属 `stage-inputs.tsv` 本身，不在这一批 |
| G1 次序 | 全量判绿之前 / 之后派崩溃验证员、门禁分诊员 | 三份定义 | 不冲突 |
| G2 | 未跟踪文件落在 `research/results/`（执行员新产物）能不能改变 55 号的判定 | 55 号只按 `replay.sh` 的 E142 行取产物名 | 构造不出 |
| G3 | `mutate.sh` 一侧：「计数：只报内存撞顶与超时两栏」「几栏加起来等于表的条数」 | `mutate.sh:647` 与第 564–582 行的计数 | 与文字一致；另注意 `❌` 在 `mutate.sh` 也判整轮失败（`record_verdict … notred 1`），文字没说它不判，不算打中 |

## 这条腿自己的限度

- G3 的合成输出是照 `mutate.sh:496`、`:512` 依赖的形状写的，没有真跑 cargo 让一条变异崩溃；59 号在这个容器里跑不起来（内存包装退 251），按定义也不跑。
- B2 里 57 号的输入是我从 `57-lkmm.sh` 与 `lkmm.sh` 读出来的（`litmus/`、`crates/`、`lkmm.sh`、57 号自己），没有登记可对；59 号给了窄（登记那一行）与宽（拷贝那几样）两头。
- B4 的桩换了两个名字；`run_in_background` 的完成通知报不报退出码是 Claude Code 的行为，我没在这一轮里核。
- G4、G5 只看了背景材料点名的例子与我顺手碰到的几处，没有逐句过 diff 的删改半句（那是正推腿的活）。
- 我提的改法（丙、丁、戊、己、B5、B6、B7 的补法）都只在我的模型上量过或只是推的，被攻过零轮。

## 没做什么

- 没跑任何重型测试（54、55、57、59、87、整轮门禁、`gate-staged.sh`）；没编译 Rust（`cargo metadata` 不编译）。
- 没读禁读清单里的文件（`governance-defs-r2-sonnet-output.md`、`governance-defs-r2-sonnet-model/`）。
- 没判正推、辩方那几格；没替主 agent 采纳；副本上的数不算入库装置上的数。
- 没改仓里任何被判的文件；写的只有这份报告与 `research/prompts/governance-defs-r2-opus-model/`，草稿在 `/tmp/claude-0/-home-user-singlefs/77b4b1be-00af-5f85-8659-f6e6d51fb840/scratchpad/defs-r2-opus/`（模型的临时仓、`crates/` 副本、检出记录；不入库，复跑会重建）。
