# governance-defs-r1 云端攻方（Opus）报告

写于 2026-09-26 14:10 UTC（东京 23:10）。被判对象：工作区里 15 份定义相对 HEAD 的改动（`research/prompts/_governance-defs-r1-diff.md`）。攻击面：G1、G2、G3、G5 为主，G4、G6 余力。本地腿缺席（正文第四节），攻击面不与它分。

## 复跑

```
cd /home/user/singlefs/research/prompts/governance-defs-r1-opus-model
nice -n 19 bash run-all.sh <草稿目录>      # 草稿目录下没有 copy/ 时它先用 tar 拷一份仓副本（排除 target、.git、research/target）
```

`run-all.sh` 依次跑：`probes.sh`（全部钩子判定，合成 PreToolUse JSON，检出记录写进 `<草稿目录>/detections.jsonl`）、`run-with-memory-cap.sh --check 4G`、四个模型 `g1-hash-divergence.sh`、`g3-sample-numbering.sh`、`g5-mutate-markers.sh`、`g6-relabel-crates-anchor.sh`，最后用 `cite.sh < citations.tsv` 数引文条数。这一次的原样输出在模型目录的 `run-all.out`。模型不跑 cargo test、不连网关、不碰主仓的 git：G1 模型的 git 仓、G6 模型改的仓副本都在草稿目录里。

| sha256 | 文件 |
|---|---|
| cc3c3aa468540182a722a3f10cdd1393d6a0d0d055580d66c9f5131f09c3ec95 | probe.sh |
| f9ec92d84fc7b6859f3af0cea62af44f04130c310b1022d4bfd537eed5235c5b | probes.sh |
| dbf7fd6415ea37f395a1c0278c75d202c950b114d83b481be71a6476cae44590 | run-all.sh |
| a938b1f484b931fefafe1a10d26f6109467d26d0463d9b95dbbeee92fe845d51 | g1-hash-divergence.sh |
| e33c9b8dae919301a3a6499b656923d009d44fc0f449cfb8dc7920655a3178f8 | g3-sample-numbering.sh |
| 00bc378acd68705d9a41a83478c28ef3c2dd63977ce65b9f4ba52d00f0edb1c3 | g5-mutate-markers.sh |
| 9b78815370a97daccf9fe604eab0480d123073e38ecdfed946a54e7120e64885 | g6-relabel-crates-anchor.sh |
| ee1f3670c6709d82604ab1a54bf4eb4c03d9351745e8c7a520a8ee44814ddf52 | cite.sh |
| 766bf100c4dd516e899f4930df7f6964ae3cdf40d892af6a1f319c72452b89c9 | citations.tsv |
| b706c6ea47b2a63ad01bdaba3fb8712ccf6be08fca742c5ec78d36262db235c8 | run-all.out |

引文取法：报告里每条「文件:行」都经 `cite.sh` 在被引文件里 `grep -nF` 恰好命中一次取出（`citations.tsv` 列了要找的串）；命中两处的两条（检出钩子的提示句、⑦ 的文件头）另用 `grep -nF` 逐行现取，行号写在正文里。

**这个容器与定义假定的机器不同的四处**（下文凡是受它影响的判定都标「环境」）：
1. 没有用户级 systemd scope：`bash research/scripts/run-with-memory-cap.sh --check 4G` 退 251（`run-all.out`「# run-with-memory-cap --check 4G」一段）。
2. 没装 `rsync`：`nice -n 19 rsync …` 报 `nice: 'rsync': No such file or directory`，`mutate.sh` 自己拷副本那一步同样报 `rsync: command not found`。仓副本改用 tar 拷，G5 模型给 `mutate.sh` 垫了一个只认那一种调用的 rsync 壳。
3. 以 root 跑，草稿在 `/tmp/claude-0/…`；钩子里写死的是 `/tmp/claude-1000/`（写范围表、⑦ 的射程）。
4. 仓的 git common-dir 里一格层 0 全绿标记都没有（见 A2）。

## 各格判定一览

「分辨」一列回答「这条历史让改前、改后两种写法一起出局吗」（正文第五节 ⚠️）：「分辨」＝只有改后的写法中；「不分辨」＝改前也中，按那一条另记一笔，不拿它判这一批。

| 编号 | 格 | 判定 | 分辨 | 一句话 | 量过 / 推的 |
|---|---|---|---|---|---|
| A1 | G1 | 打中：结果错 | 分辨 | crash-verifier 照改后的第 2 步在主工作区跑 54 号快档，主工作区在 54 号登记路径下与暂存区有任何不同，它就找不到主 agent 在 worktree 里写的那一格全绿标记、判红；同一批的 `gate.sh --staged` 判绿 | 量过（模型，7 种主工作区状态里 6 种中） |
| A2 | G1 | 打中：做不下去 | 不分辨 | 标记的键还含 54 号自己的 sha256 与 `cargo -V`/`rustc -V`，而主 agent 起全量的条件只看登记路径；只换了工具链或 54 号的一批永远等不来全量。这个容器里一格标记都没有 | 量过（键的构成、标记数）；「永远等不来」是推的 |
| A3 | G2 | 打中：被钩子拒 | 不分辨 | 实验执行员第 5 步的字面命令 `bash research/scripts/replay.sh E<号>` 被 heavy-test-guard 拒（replay.sh 正文里有不经内存包装的 cargo run 与直接执行）；G2 那批改动没改到这一句 | 量过（钩子真跑） |
| A4 | G2 | 打中：一步要的东西没人给（环境） | 分辨 | 共用约束只写了包装退 250 怎么办；退 251（scope 起不来）、252（排不上）、254（被总上限挤掉）没写。这个容器里包装恒退 251，攻方腿新得的「副本里 cargo test / run」一条也跑不成 | 量过（--check 退 251）；其余推的 |
| A5 | G3 | 打中：结果错 | 不分辨 | 样本号已被占时：照第 3 步字面写 `>` 把一份干净样本整份盖掉；照共用约束加 noclobber 则重定向失败退 1，第 4 步把它判成「本地腿缺席」 | 量过（ask-local.sh 自己的测试缝） |
| A6 | G3 | 描述与行为不符 | 分辨 | 第 4 步括注「共用约束开着 `noclobber`，`>` 写不进已存在的文件」：每次 Bash 调用开头 noclobber 都是关的，`>` 照样写进那份空文件 | 量过 |
| A7 | G5 | 打中：结果错 | 不分辨（改前也数不出，另记一笔） | mutation-triage 第 5 步让照 `✅ [抓到]`、`⏭  [无效]`、`❌ [没红]` 数：真实输出方括号里是变异名，这三个串是 `mutate.sh --selftest` 里那几条变异的名字；照字面数得 0/0/0。另有 💥（没跑完）、⚠️（抓名字盲区）两种结局哪一栏都不归 | 量过（假 cargo 跑真 mutate.sh） |
| A8 | G4 | 打中：结果错（标签错） | 部分分辨 | 核查员第 2 步只有「对得上 / 对不上 / 两样都没有」三支，没有「这个文件不在快照清单里」一支；本报告引的 28 个文件里 8 个不在这一轮的 31 行清单里，照字面落进「分不清：文件在腿开工之后被改过」 | 量过（清单对比）；核查员会怎么判是推的 |
| A9 | G6 | 打中：两处文件给出相反做法 | 分辨（改后才写「走代码轮」） | 分项翻状态时 relabel-item.py 改写 crates/**/*.rs，会把 crates/mutations.tsv 的锚点改坏、33 号红；kb-scribe 新写「实现的地盘，要走代码轮」，main-agent 那一行仍写「33 号红 → experiment-runner 只修锚点」，而 experiment-runner 对 crates/mutations.tsv 只许追加 | 量过（仓副本里翻 D23 第 4 条） |
| A10 | G6 | 两处文件说法相反 | 分辨 | 共用约束新写「`&` 只配逐个 `wait "$pid"`，不带参数的 `wait` 恒返回 0」；检出钩子拒绝时给的出路仍是「`a & b & wait`，用不带参数的 `wait` 等齐」 | 量过（钩子真跑） |
| — | G1 | 构造不出 | — | 主 agent 那条全量命令带前缀放行、不带拒；worktree 三行命令两个钩子都放行；gate.sh 整轮跑全部 `.claude/gate.d/*.sh`、`refs/sop/gate-ok` 与定义一致；`watch.sh --processes` 在 | 量过 |
| — | G2 | 构造不出 | — | 共用约束的包装写法对实现员、攻方腿、相对路径、仓副本、套 capped.sh 都放行；副本里有包装脚本、不依赖 .git；15、74 号按名字判、不读正文，与「按轻阶段」不矛盾；实现员抄的 clippy 行与 check.sh 一致 | 量过 |
| — | G5 | 构造不出 | — | 59 号「计数：」行、「没跑到」归没红、编不过归无效，与 mutation-triage 第 4、5 步一致；crates/mutations.tsv 表头六段、724 行全是六段、执行员写范围有它；英文名与 claim-experiment.sh、各门禁只按号认不冲突 | 量过 |
| — | G6 | 构造不出 | — | kb-scribe 预检带 `AGENT_HOOK_DETECTIONS` 时默认检出文件不变；relabel-item 确实扫 `crates/**/*.rs`；quote-rust-items.py、kb-scribe-followup.sh 在；决策标题行的状态写法对得上 | 量过 |

## G1 层 0 全量归主 agent

### A1 crash-verifier 在主工作区跑 54 号快档，找不到 worktree 里写的那一格标记

改后的定义让 crash-verifier 在提交流程里跑 54 号快档：

- `.claude/agents/crash-verifier.md:25:2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号只跑快档（不带 `--full`，快档加核全绿标记）；层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里自己跑，不归你；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。`
- `.claude/main-agent.md:59:| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，主 agent 自己建 HEAD + 暂存区的 worktree（建法照 54 号出路），在后台跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash <它的根>/.claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记），看门狗用 `--processes` 盯 → 派 `crash-verifier` 跑 54 号快档与 55、57、59，命令带 `SINGLEFS_HEAVY_TESTS=commit` → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（`gate.sh` 里 54 号跑快档并核全绿标记，55、59 照 `stage-inputs.tsv` 复用上一次全绿判定，57 号没有复用、每次现跑）；用户要求时两处都换成 `=user-request` |`
- 规则同样这么写：`.claude/rules/implementation-workflow.md:55:| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：层 0 全量由主 agent 在 HEAD + 暂存区的 worktree 里跑；崩溃验证员跑 54 号快档与 55、57、59 号；门禁分诊员跑 `gate.sh --staged` 并分诊，其中 54 号跑快档并核全绿标记，55、59 靠「输入没变就复用上一次全绿判定」不重跑，57 号没有复用、照跑 |`
- 阶段归属：`.claude/gate.d/stage-owners.tsv:37:54-layer0-replay.sh	crash-verifier	层 0 崩溃点重放`

54 号快档怎么找标记（`.claude/gate.d/54-layer0-replay.sh`）：

- 不带根参数时根取脚本所在的树：`.claude/gate.d/54-layer0-replay.sh:61:ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"`；crash-verifier 的命令是 `bash .claude/gate.d/<文件>`，根就是主工作区。
- 键按磁盘上的内容算，含未跟踪文件：`.claude/gate.d/54-layer0-replay.sh:182:  git -C "$ROOT" ls-files -z --cached --others --exclude-standard -- "${layer0_registered_input_paths[@]}" > "$manifest_file.listing" || return 1`，还有 `:190:  stage_script_hash="$(sha256sum < "$layer0_stage_script_path" | cut -d' ' -f1)" || return 1` 与 `:191:  toolchain_versions="$(cd "$ROOT" && cargo -V && rustc -V)" || return 1`。
- 快档用这个键找格：`.claude/gate.d/54-layer0-replay.sh:279:  write_layer0_input_manifest "$quick_manifest" || fail_without_input_manifest`、`:284:  if [[ ! -f "$full_green_marker_path" ]]; then`（找不到就判红，出路是再去 worktree 跑一次全量）。
- 54 号自己的文件头说了主工作区罩不到暂存内容：`.claude/gate.d/54-layer0-replay.sh:29:# 主工作区里跑的 --full 读的是工作区（连同别的会话没暂存的改动、工作区那一份 54 号），罩不到这一批暂存内容；所以 --full 在 HEAD + 暂存区的 worktree 里跑。`
- 登记路径：`.claude/gate.d/stage-inputs.tsv:11:54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）`

**历史**（每一步指到许可它的那一句）：
1. 主 agent 照 main-agent.md:59 用 stage-mine.py 暂存这一轮的 crates/ 改动。
2. 主 agent 照同一行建 HEAD + 暂存区的 worktree、在里面跑 `--full`，判绿，写下键为 H_staged 的那一格。
3. 主工作区里 54 号登记路径下有一处与暂存区不同（下表 C1–C4、C6、C7；agent-common.md 写明「本机常有别的会话在同一个仓里干活」）。
4. 主 agent 照同一行派 crash-verifier；crash-verifier 照 crash-verifier.md:25 在主工作区跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`：算出的键是 H_disk ≠ H_staged，第 284 行判红。
5. gate-triage 照 main-agent.md:59 跑 `gate.sh --staged`，54 号在临时 worktree 里算出 H_staged，判绿。

同一批暂存内容，一处红、一处绿。照红句的出路再跑一次全量也没用：全量在 worktree 里跑，写的还是 H_staged；只要第 3 步那处不同还在，crash-verifier 的快档就一直红。

**模型**（`g1-hash-divergence.sh`，函数原样取自 54 号，只跑 `cargo -V`/`rustc -V`）。前缀固定成「一个临时仓，暂存了一处 crates/ 改动」；放开扫的是用户决定的那一步：暂存之后、crash-verifier 开跑之前主工作区里还有什么。三列分别是主 agent 全量写的键、crash-verifier 快档找的键、`gate.sh --staged` 快档找的键（`run-all.out`「# g1」一段原样）：

```
C0 对照：主工作区与暂存区一致 full=9bfbc9fc24ccc662 files=4 | cv=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
C1 别的会话改了已跟踪文件、未暂存 full=9bfbc9fc24ccc662 files=4 | cv=ae18243e4b64448f files=4 → 找不到那一格(红) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
C2 别的会话新建未跟踪文件 full=9bfbc9fc24ccc662 files=4 | cv=5ebea91ecb67397e files=5 → 找不到那一格(红) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
C3 自己一处改动漏暂存（Cargo.lock） full=9bfbc9fc24ccc662 files=4 | cv=3da9b1db50a16888 files=4 → 找不到那一格(红) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
C4 删了已跟踪文件、删除未暂存 full=9bfbc9fc24ccc662 files=4 | cv=efbd46b77af695f8 files=3 → 找不到那一格(红) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
C5 被忽略的文件（crates/target/） full=9bfbc9fc24ccc662 files=4 | cv=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
C6 暂存之后又改了已暂存的文件 full=9bfbc9fc24ccc662 files=4 | cv=ad0f9c852b98fbad files=4 → 找不到那一格(红) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
C7 主工作区的 54 号有未暂存改动 full=9bfbc9fc24ccc662 files=4 | cv=4c8c22d3be65bc08 files=4 → 找不到那一格(红) | staged=9bfbc9fc24ccc662 files=4 → 找得到那一格(绿)
```

模型第一次跑时 `staged_tree` 忘了建基目录，补丁没套上，C0 对照三列不等、当场露出来；修掉之后才是上面这份（修法：`mkdir -p "$2"`，经 replace-once.py 换 inode）。今天主仓 `git status --porcelain -- crates Cargo.toml Cargo.lock | wc -l` → `0`，所以这一刻的主仓落在 C0；这条历史要的是第 3 步那种状态，不是今天的状态。

**打中之后的四句**：
1. 分不分辨：分辨。改前 main-agent 那一行让 crash-verifier 在 worktree 里跑 `--full`（附录二 diff 第 366 行的删除行），不在主工作区跑快档，C1–C7 不会有这一红；改后才有。规则 implementation-workflow.md:55（提交 802fcc1，不归这一轮判）写法与改后相同，它一起中。
2. 判别它的东西 crash-verifier 当时看不看得到：看得到。`git diff -- crates Cargo.toml Cargo.lock` 与 `git status --porcelain` 就分得出主工作区与暂存区；定义没让它看。它第 6 步的指纹 `git diff HEAD -- crates litmus` 把暂存与没暂存的混在一起，分不出。
3. 满足判据字面的哪一句：派发提示的「照改后定义做……结果错」。正文第五节第二行列的四种字面形态（被钩子拒、两处文件相反、输入没人给、越出定案）里没有「结果错」这一种，最近的是「两处文件给出相反做法」（crash-verifier.md:25 让它在主工作区跑，54 号第 29 行说主工作区罩不到暂存内容），但第 29 行说的是 `--full`。归哪一条由主 agent 定，我不替它归。
4. 改法在打中的格上还中不中：跑前条款只有一个改法「改那一处定义」，没写改成什么。下面三种各自在 C1–C7 上的结果：

| 改法 | C1–C4、C6、C7 | 代价与副作用 | 量过 / 推的 |
|---|---|---|---|
| F1 crash-verifier 不跑 54 号（快档与核标记由 `gate.sh --staged` 管） | 不中（没有主工作区那一次） | 要同时改 crash-verifier.md:25、main-agent.md:59、stage-owners.tsv:37，规则 implementation-workflow.md:55 也要改 | 推的 |
| F2 crash-verifier 在 HEAD + 暂存区 worktree 里跑快档（`bash <树>/.claude/gate.d/54-layer0-replay.sh <树>`） | 不中：键等于模型的 staged 列，8 行全等 | crash-verifier 不许自己做 git 写（agent-common「不做」第一条，含 worktree），树要由主 agent 给；而 54 号出路那三行在全量跑完后就 `git worktree remove`。快档与 `gate.sh --staged` 重复跑一遍 | 键相等量过；流程推的 |
| F3 照旧在主工作区跑，另加一条分诊「主工作区在登记路径下与暂存区不同 ⇒ 不是这一批」 | 照样红，只是不算这一批的账 | 红还在，每次提交都要多看一次 | 推的 |

**什么现象会推翻 A1**：crash-verifier 实际在 HEAD + 暂存区的树里跑 54 号（比如派发提示给它树根、它照带根参数跑），或 54 号快档改成按暂存区内容（`git ls-files -s` 的对象）算键而不按磁盘。

### A2 标记的键比「要不要跑全量」的条件宽（不分辨，另记一笔）

主 agent 起全量的条件是「这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入」（main-agent.md:59），登记的只有 `crates/ Cargo.toml Cargo.lock`（stage-inputs.tsv:11）；键还多两样：54 号自己的 sha256（54 号:190）与工具链版本（54 号:191）。历史：rustup 升了工具链，或这一批只改了 54 号脚本 → 这一批不碰登记路径 → 主 agent 不起全量 → `gate.sh --staged` 的 54 号找不到那一格判红 → gate-triage 照它第 3 步「一条都没碰 ⇒ 不是这一轮」分诊 → 没有哪一步叫人去补全量。改前 main-agent 的触发条件同一句，改前也中，所以不拿它判这一批。

这个容器里现成的实例：`git rev-parse --git-common-dir` 下 `ls -la | grep -i layer0` 一行都没有，而今天的键是 `239462859551948e07c4cc8f22bc36721490fcf77cf8310ba5d84ae094ff0b16`（131 个文件）。这一轮只改定义、不碰登记路径，照 main-agent.md:59 不起全量；在这个容器里跑 `gate.sh --staged`，54 号会判红（推的：我没跑 54 号，它是重型）。在用户的机器上有没有这一格标记，我看不到。
推翻 A2 的现象：main-agent 或 gate-triage 的定义里另有一句「54 号报没有这一格、而这一批不碰登记路径时，主 agent 照出路补跑全量」，我没找到。

## G2 内存包装与重型清单

### A3 实验执行员第 5 步的字面命令被 heavy-test-guard 拒（不分辨，G2 没改到这一句）

- `.claude/agents/experiment-runner.md:32:5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152（按里程碑对比六家文件系统的文件性能） 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；共享 `gate.sh` 的「转发计时」阶段查读子进程输出的循环里一边取时间一边输出。`
- 钩子：`.claude/hooks/heavy-test-guard.sh:74:# 另一道，与重型不重型无关：子 agent 跑编译出来的代码——cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算）、`；replay.sh 不在按名字判的名单里，钩子读它正文：`research/scripts/replay.sh:383:      ./target/release/e9-keylayout "$REPLAY_DEV" "$s" interleave 8 "$r" || return 1`、`research/scripts/replay.sh:414:  (cd .. && cargo run -q -p singlefs-harness --bin e142_first_transaction_write_dump) >"$impl_snapshot" || return 1`。

钩子真跑（`run-all.out`「## G2 实验执行员第 5 步」原样）：

```
exit=2 agent=experiment-runner hook=heavy-test-guard.sh
  cmd: bash research/scripts/replay.sh E158
  | ✗ 不经内存包装跑编译出来的代码被拒：脚本 /home/user/singlefs/research/scripts/replay.sh:383 里的 直接执行 cargo 编出来的二进制 ./target/release/e9-keylayout（experiment-runner 不经 run-with-memory-cap.sh）
  |   另有：脚本 /home/user/singlefs/research/scripts/replay.sh:414 里的 cargo run（experiment-runner 不经 run-with-memory-cap.sh）
  |   另有：脚本 /home/user/singlefs/research/scripts/replay.sh:415 里的 直接执行 cargo 编出来的二进制 ./target/release/e142-first-txn-dry-run（experiment-runner 不经 run-with-memory-cap.sh）
  |   另有：脚本 /home/user/singlefs/research/scripts/replay.sh:456 里的 cargo run（experiment-runner 不经 run-with-memory-cap.sh）
exit=0 agent=experiment-runner hook=heavy-test-guard.sh
  cmd: bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E158
```

钩子按整份脚本判，与复跑哪一个号无关：`bash research/scripts/replay.sh --list` 同样被拒（草稿里另跑过一次，同样四处）。钩子这一道由提交 `97f5904`（2026-09-25）加入，早于这一批，改前第 5 步同一句，所以不分辨；但它正落在 G2「子 agent 跑 cargo test / run 一律经包装」的射程里，这一批改了共用约束与实现员、攻方腿，没改执行员第 4、5 步（第 4 步生成产物直接执行 `research/target/` 下的二进制，同一道闸）。读了共用约束的执行员能自己改写成经包装的那一句，那一句放行（上面第二条）；照第 5 步字面做就被拒。
- 四句：不分辨；执行员看得到拒绝句；满足第五节第二行的「被钩子拒」；改法「第 5 步写成 `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E<号>`」在这一格上放行（量过），4G 够不够 replay.sh 里 `cargo run --release -p singlefs-harness` 的编译是推的，这个容器量不了（A4）。
- 推翻它的现象：heavy-test-guard 把 `research/scripts/replay.sh` 列进按名字判的名单（`lib_heavy_tests.py` 的 KNOWN_SCRIPT_LOCATIONS），或执行员定义第 5 步另有经包装的写法而我没读到。

### A4 包装退 251 / 252 / 254 怎么办，共用约束没写（环境）

共用约束只写了 250：`.claude/agent-common.md:46:- 重型测试（哪些算重型以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头那一句为准，逐条的判法在 `.claude/hooks/heavy-test-guard.sh` 文件头）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段（15、74 号虽然也跑 cargo test，按轻阶段对待）。子 agent 跑 `cargo test` / `cargo run` 与直接执行编出来的二进制，一律经内存包装：`bash research/scripts/run-with-memory-cap.sh <上限> <命令>`（上限照 systemd 写法，派发提示没给的用 `4G`；退出码 250 是撞了这一条的上限，照实报主 agent，不自己调大重跑），不经它的由 `heavy-test-guard.sh` 拒。重型测试里 `crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。`包装自己的文件头还有：`research/scripts/run-with-memory-cap.sh:21:#   251    slice 的总上限设不上、systemd-run 起不来（没有用户级 systemd、D-Bus 连不上、上限写法 systemd 不认）、账与锁的目录建不了，或查不到 scope 的结局：`、`research/scripts/run-with-memory-cap.sh:25:#   254    被总上限挤掉：scope 的 Result 是 oom-kill，而 scope 自己的 oom 计数是 0、oom_kill 不为 0——整个 slice 满了，内核在 slice 里挑了这一条杀，`（它接着一行写「它的结果不算数，重跑它」）。

这个容器里：

```
$ nice -n 19 bash research/scripts/run-with-memory-cap.sh --check 4G; echo "exit=$?"
  ✗ 带内存上限的 scope 起不来：slice singlefs-heavy.slice 的总上限设不上：总上限算不出来：/proc/meminfo 的 MemTotal 读不到，或整机内存减余量 20G 已不剩（RUN_WITH_MEMORY_CAP_SLICE_TOTAL=没设）；命令一行都没跑
  → 怎么办：照上面那句修（systemctl --user status、cat /sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/cgroup.subtree_control 里要有 memory）；修好之前别跑这一步，不许退回无总上限去跑
exit=251
```

历史：云端攻方（本腿）照改后 `.claude/agents/three-way-attack.md:37:- 报告文件、模型目录、草稿目录。除此之外不写。要改代码试的，把仓拷到草稿目录（`rsync -a --exclude target --exclude .git`），只在副本上改；副本与自己的模型可以编译、跑（`cargo test -p <crate> --test <目标>`、`cargo run`，经内存包装、照共用约束看负载与线程上限；重型测试照样不跑），副本上的数注明是副本。` 在副本里经包装跑 cargo test → 退 251、一行没跑；不经包装 → 钩子拒（`run-all.out` 里 `cargo test -p singlefs-core --test core_contract` 那一例退 2）。这一格在这个容器里做不下去；同一句里的 `rsync` 这个容器也没有（开头「环境」第 2 条）。在有用户级 systemd 的机器上 251 不出现，推的。
- 四句：分辨（改前攻方腿定义没许它在副本里跑 cargo test / run，这一格是新许的）；包装的 stderr 写了「命令一行都没跑」，看得到；满足「一步要的东西没人给」（251、252、254 各该怎么办）；改法「共用约束补一句：251、252 是一行没跑，照实报主 agent、不换不经包装的写法；254 照包装文件头重跑一次」是推的，没实现。
- 推翻它的现象：用户的机器与云端腿跑的容器是同一台、`--check` 退 0；或共用约束别处已写了 251 的处置而我没找到（我查的是 `grep -n '251' .claude/agent-common.md`，0 行）。

### G2 构造不出的部分（量过，钩子真跑，`run-all.out`「## G2 子 agent 经内存包装」）

- 共用约束的写法对子 agent 放行：实现员、攻方腿在仓副本里的 `bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract` 都退 0；`cd research` 后的 `bash scripts/run-with-memory-cap.sh 4G cargo run --release --bin e7-foo` 退 0。草稿里另试过 `--release`、`--lib`、`capped.sh 4` 套在包装里外两种次序、`cd 副本 && …`、主仓绝对路径的包装，全部退 0。包装按基本名认（`.claude/hooks/lib_shell_words.py:53:MEMORY_CAP_WRAPPER = "run-with-memory-cap.sh"`），副本里的相对路径一样认。
- 副本里有 `research/scripts/run-with-memory-cap.sh`（tar 拷的副本里 `ls` 得到）；它找峰值表时 git 找不到就退回脚本旁边那一份（`default_peak_table`），副本不带 `.git` 也跑得起来（这个容器里只到 251 那一步，推的）。
- 「15、74 号按轻阶段」与钩子不矛盾：钩子按名字判门禁阶段、不读正文，实现员跑 `nice -n 19 bash .claude/gate.d/74-model-differential.sh` 退 0。74 号红了时它给的出路是裸的 `cargo test`（`.claude/gate.d/74-model-differential.sh:46:    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"`），照抄会被拒（量过，退 2）；共用约束已写了要经包装，读了共用约束的实现员会改写，不算打中。15 号的 `.claude/gate.d/15-research-build.sh:25:out="$(cd "$R" && cargo test --release 2>&1)"` 按钩子的分类本是「在工作区根裸跑的全量测试」，但它包在按名字判的阶段里，钩子看不到；规则与共用约束都明写 15 号按轻阶段，两边说法一致。
- 实现员抄的 lint 与 check.sh 一致：check.sh 第 22 行起 `CODE_DISCIPLINE_LINTS=(` 七条，第 31 行 `cargo clippy --all-targets --all-features -- -D warnings "${CODE_DISCIPLINE_LINTS[@]}" \`，与 `.claude/agents/implementation-writer.md:28` 的写法相同。

## G3 本地腿

被判的两句：
- `.claude/agents/three-way-local-attack.md:27:3. 用 Bash 的 `run_in_background` 起 `bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-s<n>.md`（单次请求最长 `ASK_LOCAL_TIMEOUT` 默认 900 秒，超过前台上限），起完结束本轮等完成通知，退出码取通知里的；命令里不加 `setsid`、`&`、`disown`。`
- `.claude/agents/three-way-local-attack.md:28:4. 退出码 5：作废副本由脚本留成 `-output-void<n>.md`，这一份不算；判红那次重定向建出的 `s<n>` 是空文件，先确认它大小为 0，再用 `>|` 重定向到同一个号重跑（共用约束开着 `noclobber`，`>` 写不进已存在的文件）；退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏。退出码非 0 非 5（包括 6：损坏检测器没跑成或找不到，这一份没验过）、或网关不通：停下，报「本地腿缺席」。`
- 共用约束：`.claude/agent-common.md:35:- tools 只有 Read / Bash 的：新建文件一律排他，`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告；之后 `>>` 追加。`（本地攻方的 tools 是 `Read, Bash`）

钩子：第 3 步命令、加 noclobber 的写法、第 4 步 `>|` 的写法，以 `run_in_background` 喂 bash-command-detector.sh 都退 0（`run-all.out`「## G3」一段）。④（后台里单独的 `&`）与第 3 步的写法不相干。

### A6 「共用约束开着 noclobber」这句前提不成立（量过）

两次独立的 Bash 调用：

```
$ set -o noclobber; set -o | grep noclobber
noclobber      	on
$ set -o | grep noclobber; printf 'clean sample\n' > $W/g3-s1.md; printf 'second write\n' > $W/g3-s1.md; echo "rc=$?"; cat $W/g3-s1.md
noclobber      	off
rc=0
second write
```

每次 Bash 调用开头 noclobber 是关的；共用约束 35 行是要 agent 自己在建新文件的那一条命令里打开，不是替它开着。模型 H3（`g3-sample-numbering.sh`，用 ask-local.sh 自己的 `ASK_LOCAL_FAKE_TEXT` 测试缝）：判红留下的空 s2 直接用 `>` 重跑，退 0、s2 写进 72 字节。所以 `>|` 不是必需，括注的理由是错的；照第 4 步做结果不错，是描述与行为不符。
推翻它的现象：Claude Code 的 Bash 工具在某个版本里跨调用保留 `set -o` 状态（这个容器里不保留，上面两条是连着的两次调用）。

### A5 样本号已被占：`>` 盖掉干净样本，或 noclobber 下被判成「本地腿缺席」（不分辨）

第 3 步没说 `n` 怎么取（第 4 步只说退出码 0 的每次调用占一个号）。放开扫的是这一步用户（腿）决定的动作：`s<n>` 事先存不存在、存在时是空的还是干净样本、命令前加没加 noclobber。模型原样（`run-all.out`「# g3」，路径换成 `$SCRATCH`）：

```
H1 s1 已是一份干净样本，再用第 3 步字面写法落进 s1
  退出码=0；s1=bf1683f0826e129e；stderr 首行：
H2 同上，但照共用约束「新建文件一律排他」加 noclobber
  退出码=1；s1=bf1683f0826e129e；stderr 首行：bash: line 1: prompts/rX-local-attack-output-s1.md: cannot overwrite existing file
H4 检测器找不到（退出码 6 那一支）
  退出码=6；s1=bf1683f0826e129e；stderr 首行：ask-local: 找不到 nochecker/corruption-check.py —— **没做**字词损坏检查，不是通过了
  s3 大小：0 字节；作废副本：1 份
```

（H0 先写出 s1＝clean1，sha256 前 16 位 `81507b49f13c5ed3`；H1 之后成了 clean2 的 `bf1683f0826e129e`：第一份干净样本没了，退出码 0，没有任何提示。）

- H1：照第 3 步字面写 `>`，一份干净样本被整份盖掉。
- H2：照共用约束 35 行加 noclobber，重定向本身失败，ask-local.sh 一行没跑，退 1；第 4 步「退出码非 0 非 5……停下，报『本地腿缺席』」，把网关好好的一条腿报成缺席。
- 号什么时候会被占：同一轮重派一条接着做的本地腿（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」一节写了撞限额后新开一条接着做），它从 s1 数起；或同一前缀给了两条腿。这一步由腿与主 agent 决定，所以这是一条可达的历史，不是必然。
- 四句：不分辨（改前第 3 步同一个 `>`、第 4 步同一句「非 0 非 5 → 缺席」，改前也中）；腿看得到 stderr 那一行 `cannot overwrite existing file`，定义没让它看；满足派发提示的「结果错」（盖掉证据、误报缺席），第五节第二行的四种字面形态里没有；改法「第 3 步号取 `ls <前缀>-output-s*.md` 之后下一个没用过的，命令前加 `set -o noclobber;`；第 4 步补一句『重定向失败（stderr 是 cannot overwrite existing file、退 1）不是缺席，换下一个号』」只在我的模型上看过 stderr 那一行，改法本身推的、被攻过零轮。
- 退出码 6 与「非 0 非 5 → 缺席」对得上：H4 退 6、s3 空、留一份作废副本，第 4 步括注把 6 归进缺席那一支。ask-local.sh 自己带提示文件调 oov-check（`research/scripts/ask-local.sh:88:  VERDICT="$(python3 "$CHECK" "$TXT" "$1" 2>&1)"; crc=$?`），第 5 步手跑 `oov-check.py <样本> <提示文件>` 与它同口径；生词表截 300 字符属实（`research/scripts/oov-check.py:249:        print("     生词: " + ' '.join(dict.fromkeys(oov))[:300])`）。
- 推翻 A5 的现象：本地腿定义另有「号从哪取」的规定而我没读到；或 ask-local.sh 自己检查 stdout 指到的文件已非空并拒绝（今天没有：H1 退 0）。

## G5 变异计数

### A7 照字面数 `✅ [抓到]` / `⏭  [无效]` / `❌ [没红]` 得 0/0/0（不分辨，另记一笔）

- `.claude/agents/mutation-triage.md:28:5. 报抓到 / 无效 / 没红三个数（`mutate.sh` 没有三数汇总行，照每条的 `✅ [抓到]`、`⏭  [无效]`、`❌ [没红]` 标记数；59 号照「计数：」行），超时、内存撞顶等其余几栏另列、不并进三个数，不为 0 的整轮已判失败；与改动前的数比，「无效」变多要单列。`
- mutate.sh 真正打的：`research/scripts/mutate.sh:538:    record_verdict "$row_index" caught 0 "✅ [$name] 红：$red" ""`、`research/scripts/mutate.sh:536:    record_verdict "$row_index" notred 1 "❌ [$name] 一个测试都没红 —— 这条破坏没有被任何检查看见" ""`、`research/scripts/mutate.sh:504:    record_verdict "$row_index" invalid 0 "⏭  [$name] 变异导致编译失败，本条无效（不计入盲区，也不算命中）" ""`；另两种：`research/scripts/mutate.sh:514:    record_verdict "$row_index" crashed 0 "💥 [$name] 测试进程没跑完（signal ${sig:-?}）——破坏被看见了，但不是断言抓到的" ""`、`research/scripts/mutate.sh:534:    record_verdict "$row_index" nameblind 1 "⚠️  [$name] 测试进程报了失败，但一个失败的测试名都没抓到 —— mutate.sh 抓名字的规则有盲区，这一条不算盲区也不算命中" ""`。
- 定义里那三个串出自自检：`research/scripts/mutate.sh:111:  for wanted in "✅ [抓到] 红：tests::adds" "❌ [没红]" "⏭  [无效]" "⏱  [超时]" "🧱 [撞内存] 撞了内存上限 64M（内存撞顶）" \`——自检变异表里的变异就叫「抓到」「没红」「无效」。正文第二节第 4 条（主 agent 的观测）同样写成了这三个串。
- 收尾只有：`research/scripts/mutate.sh:647:echo "计数：内存撞顶 $memory_hit_count 条（上限 $MEMORY_MAX）、超时 $timeout_count 条"`。

模型（`g5-mutate-markers.sh`：假 cargo 仿自检写法，五条变异取实验表常见的英文名；这个容器没有 scope，设 `RUN_WITH_MEMORY_CAP_BREAK=nocap` 让包装直接跑，另垫一个 rsync 壳，两样都不影响判结局）。`run-all.out`「# g5」原样：

```
✅ [e999_drop_bound_check] 红：tests::adds
✅ [e999_flip_comparison] 红：tests::adds
❌ [e999_touch_comment] 一个测试都没红 —— 这条破坏没有被任何检查看见
⏭  [e999_break_syntax] 变异导致编译失败，本条无效（不计入盲区，也不算命中）
💥 [e999_endless_loop] 测试进程没跑完（signal 9）——破坏被看见了，但不是断言抓到的
计数：内存撞顶 0 条（上限 16G）、超时 0 条
已还原，基线仍全绿
── 照定义字面数（grep -cF）──
✅ [抓到] → 0
⏭  [无效] → 0
❌ [没红] → 0
── 按行首符号数（grep -c '^符号'）──
✅ → 2
⏭ → 1
❌ → 1
💥 → 1
变异表条数：5
```

- 四句：不分辨：改前写「原样贴 `mutate.sh` 的汇总行」，那一行不存在，改前也数不出（改后 5 行里自己承认「没有三数汇总行」）。所以它不支持退回改前，要的是把改后的写法改对。分诊员看得到每一行的变异名。满足派发提示的「结果错」。改法「按行首符号数：`grep -c '^✅ \['`、`grep -c '^⏭  \['`、`grep -c '^❌ \['`，另数 `'^💥 \['`、`'^⚠️  \['` 并写明它们归哪一栏」在模型上 2＋1＋1＋1 ＝ 5 ＝ 表的条数（量过，只在这一张表上）；💥、⚠️ 该归哪一栏（照 `mutation-sampling.md` 分类，还是算进三个数之一）我没定，交主 agent。
- 推翻 A7 的现象：mutate.sh 在某条路径上打字面的 `✅ [抓到]`（只有变异名恰好叫「抓到」时才会）；或分诊员把 `✅ [抓到]` 读成「✅ 那一类」而不是字面串——那样数得对，但定义写的是反引号里的字面串。

### G5 构造不出的部分

- 59 号：「计数：」行在 `.claude/gate.d/59-crates-mutation-replay.sh` 第 407–408 行，八栏加「共 N 条」；「点名的测试没跑到」在第 349 行归进 `failure`（没红），编不过归无效，与 mutation-triage 第 4 步「编不过的在『有变异无效』一栏，『没跑到』算进没红」一致。scope 一开始就起不来时（第 212 行）一条都不跑，不打「计数：」行，第 4 步「对不上就是中途退出，这一次的数不算」接得住。
- 入库装置那一支：`crates/mutations.tsv:1:# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。`，非注释行 724 行全是六段（`awk -F'\t' 'NR>1 && $0 !~ /^#/ {print NF}' crates/mutations.tsv | sort | uniq -c` → `724 6`）；写范围表有 `experiment-runner	crates/mutations.tsv` 一行；33 号在未改动的副本上判绿（`✓ 148 个实验二进制都有成形的变异表，1653 条变异的原文各命中源码一次；crates/mutations.tsv 724 条的原文各命中源码一次`）。
- 英文名：`claim-experiment.sh` 只按号查重（`used_numbers` 从各处文件名里取号），34、85、86、96 号按号或按目录认文件，不按简称配对；照英文名起源文件与变异表，这几道都认得到。harness 里的 bin 没有 `[[bin]]`，名字就是蛇形的文件名（replay.sh 里是 `--bin e158_root_choice_repair`），第 2 步「`name` 写连字符」只对 research 那一支的 `[[bin]]` 成立；执行员照 `replay.sh` 已有的 E156、E158 两行照抄就对，我没构造出照做会错的历史，记作措辞上的歧义。

## G4 核查员与三方调度（余力）

### A8 快照清单外的文件，核查员第 2 步没有那一支（部分分辨）

- `.claude/agents/three-way-verifier.md:21:- 腿开工那一刻的快照路径：`sha256sum` 清单，罩这一轮被判的文件与材料点名的 kb 文件（`.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」；代码轮必给），腿跑着的时候被别的会话改过的，主 agent 另给倒推出的原样副本（`*.at-snapshot`）。没给快照的代码轮，停下要，不对主树核。没给快照的轮（设计轮）对主树核，腿引的行号与主树对不上时记「分不清：文件可能在腿交回之后被改过」（与第 6 步那一类一样单列），不记 ✗。`
- `.claude/agents/three-way-verifier.md:27:2. 每处「文件:行号 + 抄的原文」：先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件在腿开工之后被改过」，不记 ✗。对不上时再找原文实际在第几行；若那个行号在背景材料里正好是这句，写「误写成背景材料第 N 行，原文件实为第 M 行」，不只打 ✗。`

这一轮的快照 `research/prompts/governance-defs-r1-snapshot/sha256sums.txt` 31 行，`sha256sum -c` 此刻 31 行全 OK。本报告引了 28 个文件（`cite.sh` 取出的「文件:行」去重），其中 8 个不在清单里：

```
$ comm -23 cited-files.txt snap-files.txt
.claude/gate.d/15-research-build.sh
.claude/gate.d/74-model-differential.sh
.claude/gate.d/lib-item-ref-status.py
.claude/hooks/lib_shell_words.py
.claude/singlefs-ai-sop/scripts/gate.sh
crates/mutations.tsv
research/scripts/replay.sh
research/scripts/watch.sh
```

这 8 个文件的引文，照第 2 步字面：清单里没有它，谈不上「对得上」；主 agent 也不会为一个不在清单里的文件给倒推副本 ⇒ 落进「两样都没有的记『分不清：文件在腿开工之后被改过』」。这个标签是错的：文件未必被改过，只是清单没罩到。腿引什么文件，开工前没人知道，主 agent 列清单时也罩不全。
- 四句：部分分辨。改前写「快照路径（`crates/` 与 `.claude/kb/` 至少这两样）」加「到快照里取那一行」（附录二 diff 第 307、314 行的删除行）；改前的快照罩整个 crates/ 与 kb/，代码轮里腿引的多半在这两处，改后只罩被判文件与材料点名的 kb。定义轮两种写法都罩不到脚本与钩子，这一格不分辨。核查员看得到清单里有没有这个文件（`grep -F` 一下就知道）。满足派发提示的「结果错」（标签错），第五节第二行的字面形态里最近的是「一步要的输入没人给」。改法「第 2 步补一支：文件不在清单里，对主树核，行号对不上记『分不清：文件不在快照里』」推的，被攻过零轮。
- 推翻它的现象：核查员定义别处有「清单外的文件怎么核」而我没读到；或规则要求快照罩腿可能引的全部文件（implementation-workflow.md「代码轮派腿之前记一份开工快照」只要求被判文件与材料点名的 kb 文件）。

### G4 其余

- main-agent 那一句与三方规则：`.claude/rules/three-way-inference.md:163:云端腿同样适用——这条管的是「模型答复」这类观测，不是本机那条腿特有的：云端腿报「没打中」而要拿它支撑结论时，主 agent 用同一份提示再派一条同立场腿，报告带 `-s2` 后缀。`；main-agent.md:54 写到「主 agent 用同一份提示再派一条同立场腿」为止，少了「报告带 `-s2` 后缀」。不逐字同义，只少一个命名细节；第二条腿的报告路径由派发提示给，丢了这半句不会让谁做不下去。记为措辞差，不算打中。
- 与这一轮直接相关：本报告里标「构造不出」的格（G1、G2、G5、G6 各一行），照这一条要支撑「站得住」之前得再抽一次同立场腿；正文第五节第一行没写这一步。

## G6 其余描述修正（余力）

### A9 分项翻状态改写 crates/ 里的锚点：kb-scribe 与 main-agent 给出相反的去向（分辨）

- `.claude/agents/kb-scribe.md:30:3. 分项翻状态：`python3 research/scripts/relabel-item.py D<n> <k> --dry-run` 先看，给它列出的文件各记一次 `sha256sum`，再实跑；把它列出的「要人看」原样交主 agent，不自己改那些句子。它改写了 `research/**/*.rs` 的，加跑 33 号，并把改到的实验源码逐个列给主 agent；改写了 `crates/**/*.rs` 的（实现的地盘，要走代码轮），同样逐个列给主 agent；预演列出的文件里带别的会话没提交改动的（`git status --porcelain` 里有它），也逐个列给主 agent。然后 `bash .claude/gate.d/21-decision-items-sync.sh --write`。`
- `.claude/main-agent.md:61:| 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |`
- relabel-item 确实扫 crates：`.claude/gate.d/lib-item-ref-status.py:74:         'research/**/*.rs', 'crates/**/*.rs', '.claude/rules/*.md')], [])))`。
- 执行员对 crates/mutations.tsv 只许追加（experiment-runner.md:27 那一行里「变异表改追加进 `crates/mutations.tsv` 末尾」与「只追加，不改别人的」）。

**历史**（`g6-relabel-crates-anchor.sh`，全在草稿里的仓副本上）：把 D23 第 4 条从已定项表挪进新立的「### 未定项」一节 → 调 relabel-item.py 自己的 `relabel()`（kb-scribe 第 3 步实跑那一步）→ crates/ 下 13 个文件被改写 → `crates/mutations.tsv` 第 463 行（「P6 后一半：空发布记录的本次发布内序号写成 2」）的锚点在 `crates/singlefs-core/src/transaction.rs` 里命中从 1 变 0 → 33 号判红：

```
✓ D23（journal 的角色与格式） 第 4 条是未定：改写 282 处、67 个文件；要人看的句子 0 处
翻状态之后锚点命中次数：0
  ✗ crates/mutations.tsv 这些行的「原文」在源码里不是恰好命中一次（门禁 59 号预扫会整张表退出，一条都不跑）：
      crates/mutations.tsv:463 P6 后一半：空发布记录的本次发布内序号写成 2：原文在 crates/singlefs-core/src/transaction.rs 里命中 0 次
33 号退出码=1
```

同一次还坏了一条 research 的锚点（`research/mutations/e142_first_transaction_dry_run.tsv:85 M87_offset_table_reverts_to_307_layout`），那一条归执行员修是对的。未改动的副本上 33 号退 0（对照）。
接下来：main-agent.md:61 说「33 号红 → experiment-runner 只修锚点」；执行员修 crates/mutations.tsv 第 463 行等于改别人的行，它的定义不许；kb-scribe.md:30 又说 crates 的改写「是实现的地盘，要走代码轮」。三份定义给出两个去向。
- 四句：分辨（改前 kb-scribe 对 crates 一句没写，只有 main-agent 一个去向；改后才出现第二个去向）。主 agent 看得到 33 号点名的是 `crates/mutations.tsv` 还是 `research/mutations/`。满足第五节第二行的「两处文件给出相反做法」。改法「main-agent.md:61 按表分：research/ 的锚点 → experiment-runner，crates/mutations.tsv 的 → implementation-writer（代码轮）」推的，被攻过零轮。
- 取样范围：crates/mutations.tsv 里原文带「已定项 / 未定项」字样的锚点只有 2 条（`awk -F'\t' 'NR>2 && $0 !~ /^#/ && $3 ~ /[已未]定项/' crates/mutations.tsv | wc -l` → `2`，第 459、463 行），我只翻了 D23 第 4 条那一条；这条历史只在翻到这两条锚点引的分项时出现。
- 推翻它的现象：relabel-item.py 跳过 crates/mutations.tsv 锚点所在的行；或 main-agent 别处写了 crates 锚点的去向。

### A10 并行 `&` 配哪种 `wait`：共用约束与检出钩子说法相反（分辨）

- `.claude/agent-common.md:55:- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后逐个 `wait "$pid"` 取回每个作业的退出码时用（不带参数的 `wait` 恒返回 0，会把失败吞掉，`.claude/singlefs-ai-sop/rules/command-safety.md`「并行不许把失败吃掉」）。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。`
- 检出钩子拒绝时的出路（`grep -nF` 命中两处，逐行现取）：`.claude/hooks/bash-command-detector.sh:2366:                  "几条活要并行就在同一条命令里 `a & b & wait`，用不带参数的 `wait` 等齐", file=sys.stderr)`、`.claude/hooks/bash-command-detector.sh:2377:                  "（几条活要并行就在同一条命令里 `a & b & wait`，用 `wait` 等齐）；"`。

钩子两种写法都放行（`run-all.out`「## G6」：逐个 `wait "$first"` 退 0，`sleep 1 & sleep 2 & wait` 退 0，单独的 `sleep 1 &` 退 2 并给出上面第 2377 行那句出路）。历史：子 agent 在 run_in_background 里写了一个没有 `wait` 的 `&` → 被拒 → 照拒绝句改成 `a & b & wait` → 放行，失败被吞（共用约束说的那种）。钩子不在这一轮 15 份里，也不许子 agent 改；照改后的共用约束做不会错，照钩子的出路做会错。
- 四句：分辨（改前共用约束写的正是「用不带参数的 `wait` 等齐」，与钩子一致）；agent 看得到两句；满足「两处文件给出相反做法」（一处是钩子）；改法「钩子的出路句改成逐个 `wait "$pid"`」推的，不归这一轮的 15 份，要另立一笔。
- 规则原文站在共用约束这边：`.claude/singlefs-ai-sop/rules/command-safety.md:100:**不带参数的 `wait` 退出码恒为 0。** 后台那一批里红了几个，它一个字都不说。`（同一节还写「命令位置上不带参数的 `wait` 判红」由 shell-lint S6 判）。推翻它的现象：检出钩子的出路句已改成逐个 `wait "$pid"`（今天第 2366、2377 行不是）。

## 构造不出的格：依据

**G1**（`run-all.out`「## G1」）：
- 主 agent 的命令带 `SINGLEFS_HEAVY_TESTS=commit` 退 0，不带退 2（`✗ 重型测试被拒：门禁 54 号（54-layer0-replay.sh）（层 0）：主 agent 跑「层 0」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带`）。54 号出路那三行（`print_staged_worktree_full_commands` 打出来的原样，存在草稿 `g1-worktree-cmd.txt`）以主 agent、`run_in_background` 喂两个钩子都退 0。那三行里 54 号的路径是 `"$layer0_full_base/tree/…"`，钩子看不见变量拼出的路径，放行与前缀无关；主 agent 照 main-agent.md:59 带前缀，不受影响。
- gate-triage 不再单跑登记给它的阶段：`gate.sh` 按文件名跑全部本地阶段（`.claude/singlefs-ai-sop/scripts/gate.sh:419:    < <(find "$GATE_D" -maxdepth 1 -name '*.sh' -type f | sort)`），`--staged` 在临时树里跑同一份循环，登记给它的阶段都在里面。ref 名：`.claude/singlefs-ai-sop/scripts/gate.sh:535:  git -C "$ROOT" update-ref refs/sop/gate-ok "$START_HEAD" 2>/dev/null || true`，与 gate-triage 写范围一句一致。
- 看门狗：`research/scripts/watch.sh:7:#   bash research/scripts/watch.sh --processes                           只盯这个 Claude 实例底下已经在跑的长进程（主 agent 自己起的长命令）；`
- crash-verifier 等全量：它第 1 步「有 `54-layer0-replay.sh --full` 在跑时，不起 54 号」，主 agent 的全量在后台，`ps` 看得到那条命令行，接得上（推的，没起真进程）。

**G6**：
- kb-scribe 预检（改后第 1 步写法）真跑三条路径：`.claude/kb/decisions.md` 退 0，`crates/singlefs-core/src/lib.rs` 与 `research/prompts/x.md` 退 2；`/tmp/claude-1000/agent-hook-detections.jsonl` 前后都是 598 字节，草稿里的 `precheck.jsonl` 2 行。
- ⑦ 的射程：`.claude/hooks/bash-command-detector.sh:116:# ⑦ 在同一个 inode 上改一个已经存在的脚本（`.sh`、`.py`，或带执行位的文件；在仓里或 /tmp/claude-1000/ 下）：正在跑它的 bash 按文件偏移往下读，`（`grep -nF` 命中两处，这一行逐行现取）。共用约束写「草稿目录里……改一个已存在的 `.sh` / `.py` 照 ⑦ 换 inode」与它一致；这个容器的草稿在 `/tmp/claude-0/`，`printf 'x' > 已存在的.sh` 在那里退 0，是环境（开头第 3 条），不是定义的错。
- `research/scripts/quote-rust-items.py`、`.claude/hooks/kb-scribe-followup.sh`、`.claude/hooks/kb-scribe-followups.tsv` 都在（`ls` 列得出）。决策标题行状态：`grep -h '^## D[0-9]' .claude/kb/decisions/*.md | sed 's/.*—— //' | sort | uniq -c` → `2 半定（一项未定）`、`1 半定（三项未定）`、`25 已定`，与 kb-scribe 新写的「已定 / 半定（N 项未定）」一致。
- sweep 组号：`research/scripts/stale-candidates.py` 的 `parse_group_ranges` 文档串写「F1-F12,F15」，用 ASCII 连字符，与 sweep 的新写法一致（没喂 en dash 看它拒不拒，推的）。
- main-agent 第 7 步点名的三处（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文）与第 2 步列的逐字相同（main-agent.md 第 19 行）。

## 没打中的形状与取样范围

| 格 | 试过的形状 | 取样范围 | 结果 |
|---|---|---|---|
| G1 | 主 agent 全量命令带 / 不带前缀；crash-verifier 快档、带 `--full`（钩子放行，定义不许）；gate-triage `gate.sh --staged`、单跑 15 号；worktree 三行 | 7 条命令（worktree 三行喂了两个钩子，其余只喂 heavy-test-guard） | 只有「不带前缀」被拒，与定义一致 |
| G1 | 主工作区相对暂存区的 8 种状态（A1 模型 C0–C7） | 一个临时仓、一处暂存改动；没扫「别的会话改的是登记路径之外的文件」（那种不进键，推的：不中） | 6 种中 |
| G2 | 实现员、攻方腿、执行员经包装 / 不经包装的 cargo test、run、`--lib`、`--release`、套 capped.sh、`cd 副本`、主仓绝对路径包装、74 号、replay.sh、mutate.sh | 钩子 1 个 × 16 条命令 | 不经包装的与 replay.sh 字面写法被拒 |
| G3 | 第 3、4 步三种重定向 × 后台；样本号新 / 已有干净 / 已有空 / 加没加 noclobber；检测器缺失 | 检出钩子 4 条（`run-all.out` 里 3 条，另一条 `>|` 到 research/results/ 下不存在的文件只在草稿里跑过）；ask-local.sh 测试缝 5 次调用 | A5、A6；没试网关真返回、没试 exit 5 的真实损坏文本（用的是空文件那一格直接构造） |
| G5 | mutate.sh 真跑 5 条变异（抓到 2、没红 1、无效 1、没跑完 1）；59 号只读代码没跑 | 1 张表；超时、撞内存两种结局没造（nocap 下撞内存造不出，超时我没造） | A7 |
| G6 | relabel 翻 D23 第 4 条；kb-scribe 预检 3 条路径；`&` 三种写法 | 翻状态只试 1 条分项（2 条易腐锚点里的 1 条） | A9、A10 |

没攻的：G4 的「本地腿派哪条由主 agent 定」逐字对照（只看了一眼 main-agent.md:54 与 three-way-inference.md 第 21 行的意思相同，没逐字比）；G6 里 experiment-designer 的排除写法（`--exclude=experiments.md --exclude=experiments-history.md`）没真跑 grep 去看排干净没有；sweep、materials 的其余改动只核了文件在不在。

## 这条腿自己的限度

- 我自己提的改法（F1–F3、A3–A10 各条里写的改法）只在我的模型上量过或只是推的，**被攻过零轮**。「几个改法各修哪一格」的表只在 A1 一处，每格已标量过 / 推的。
- A1、A2 里凡是说「54 号快档会红」的，都是用 54 号自己的键函数算出键不等、推出第 284 行那一支，没有真跑 54 号（它是重型，子 agent 不跑）。
- G5 模型的 mutate.sh 在 `RUN_WITH_MEMORY_CAP_BREAK=nocap` 下跑，还垫了 rsync 壳；判结局那段代码没动，内存上限那一支不在模型里。
- G6 模型改的是草稿里的仓副本，副本上的数不是入库装置上的数；33 号是在副本里跑的。
- 这个容器的环境与用户的机器不同（开头四条）；A2、A4 里「这个容器」那半句只对这个容器成立。
- 「构造不出」的格是一次抽样；照 three-way-inference.md:163，要拿它支撑「站得住」得再派一条同立场腿。
- 本地攻方缺席，没有第二条攻方腿与我分攻击面，G4、G6 我只是余力扫过。

## 没做什么

- 没跑重型测试（54、55、57、59、87 号、`gate.sh`、全量 cargo test），没跑 `cargo test` / `cargo run`（这个容器包装退 251）。门禁阶段只在草稿的仓副本里跑了 33 号（轻阶段）。
- 没读禁读清单里的 `governance-defs-r1-sonnet-output.md` 与 `governance-defs-r1-sonnet-model/`。
- 没改仓里任何定义、钩子、规则、脚本；写的只有这份报告、模型目录与草稿目录。G1 模型的 `git init`、`git worktree` 都在草稿目录里自建的临时仓上，没碰主仓的 git。
- 不判正推那几格（改动对不对得上核对结论与用户定案、删掉的原文丢没丢判据）；不替主 agent 采纳；副本上的数不算入库装置上的数。
- 草稿目录 `/tmp/claude-0/-home-user-singlefs/77b4b1be-00af-5f85-8659-f6e6d51fb840/scratchpad/leg-opus/` 里还有：仓副本 `copy/`、各模型的临时目录 `g1*/tmp.*`、`g3.*`、`g5.*`、`g6.*`、检出记录 `detections.jsonl`、`precheck.jsonl`、引文取出 `cites.txt`、`cited-files.txt`、`snap-files.txt`、`g1-worktree-cmd.txt`、`g6-baseline-33.out`。承重的原样输出都已进模型目录的 `run-all.out`，草稿里的不作依据。
