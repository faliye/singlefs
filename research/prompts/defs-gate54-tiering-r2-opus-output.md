# defs-gate54-tiering-r2 云端攻方（Opus）报告：V1、V2

<!-- doc-lint:not-numbers V1 V2 V3 V4 V5 K1 K2 K3 S1 S2 S3 S4 S5 T0 T1 T2 T3 H-A H-B H-B2 H-C g1 g2 g4 g6 g124 f2 f3 f4a f4b f5 -->

2026-09-24，01:05–01:45 UTC（10:05–10:45 JST）。攻击面：正文第三节 V1、V2。禁读清单照派发提示，没读。
被判的 54 号：工作区那一份，sha256 `232c8c0de11bf6561f9ca1736652745f387d05832ad0a5c99a08bc80c0386bf7`，与 `defs-gate54-tiering-r2-snapshot/sha256sums.txt` 同（`sha256sum -c` OK）。
`.claude/main-agent.md` 在我干活的途中被别的会话改过（01:16 UTC 现查 sha256 `66e18e65…`，快照记的是 `14a43f86…`，`-c` FAILED），我引的那一行字面与背景材料里的逐字相同（`grep -c` 两边各 1 次），行号从我开工时的第 44 行挪到了第 49 行；下文写第 49 行。

## 复跑

```
bash research/prompts/defs-gate54-tiering-r2-opus-model/run-all.sh /tmp/claude-1000/defs54-r2-attack/runs
```

只在给的草稿根下建合成仓（`git init` 的临时仓，每格一个）、跑假 cargo（`fake-bin/cargo`，打合成日志，行为由调用那一刻 `crash.rs` 里的记号与仓外的假工具链版本决定）；真仓一处 git 写都没做，真仓 common-dir 里 `singlefs-layer0*` 现查 0 个。整趟 01:35:24–01:43:30 UTC，8 分钟。入库的 `outputs/` 就是这一趟的原样。

模型文件（`sha256sums.txt` 原样，`run-all.sh` 开跑先 `-c`）：

| sha256 | 文件 |
|---|---|
| 7a1124f4f8a8f4777efb9520f3574070c2a2b921cc784177698e6f29aba3cf02 | fake-bin/cargo |
| 3bb6f284ce1cb93427c6a184be2dabc074064bd60bef3c4fb03f3e2b2ae6f376 | fix-arms.sh |
| 191f90430b39a7731a78160684b3d9006a6339469b397bcea2d66d8f4f68443a | fixes/54-g1.sh |
| 2924db515c1c92b5d2932a71494d53b0752595f74df8eeebb8f565674444c312 | fixes/54-g124.sh |
| e0dc1da7f1d3e73aef74049ce1aa71c0d0f9162506439b05278f6908308c79ab | fixes/54-g2.sh |
| dde733423606253339d0aaca26c2d232d0b86bcf95375cecb292091867276ebd | fixes/54-g4.sh |
| 37a8d4e6486d251c25fc1c943b905ff7a1d13f5c67ea4c147443f1b86f27f0ed | h-a-late-stage-54.sh |
| 5ebc735b0cb37fb1386f12484cbff49b610c9df2c83979a6082d5bcba6318b98 | h-b-revert-stale-cell.sh |
| e38c712c90583b83e76e82aceb31db5b39936786ec7a6dacb49d9f62188079b6 | h-b2-forced-recheck.sh |
| d9751b64fb7c0b98bbdb252184c7b90ef7b9a0332f07e21c15bdbdf481b6d65b | h-c-foreign-run-deletes.sh |
| 232c8c0de11bf6561f9ca1736652745f387d05832ad0a5c99a08bc80c0386bf7 | inputs/54-post.sh（工作区的 54 号） |
| 726dd25419c096d5562627c8c431ac48b7da7060d5082e3450506c970c171521 | inputs/54-pre.sh（`git show HEAD:` 那一份，分档之前） |
| f0c64f9f470c9f478be0ab864d5386e1f40f3e85df92b8a78e5e4a00fcb514f2 | inputs/change-touches-crates.sh（与 HEAD 同，两臂共用） |
| f0c64f9f470c9f478be0ab864d5386e1f40f3e85df92b8a78e5e4a00fcb514f2 | inputs/change-touches-crates-pre.sh |
| 71ea1c7e18f959a5aefd120ae7f2843bfeb4da007a51148e5b2ba2d45fcdeac1 | inputs/stage-inputs-pre.tsv（HEAD） |
| 03314d49bf746b584c170722aaef29a1271120894c3dc952473d46509aab5acb | inputs/stage-inputs.tsv（工作区） |
| 32f576aa8f9de118cc418fef0b8a36f663af418d87e1b50daf7fd6169fc2468f | inputs/stage-must-run-pre.sh（HEAD） |
| 5c75fa0eef04bb105330a935032c3f34eb5759c29918caf0f1506a4d1bf323db | inputs/stage-must-run.sh（工作区） |
| 672a21a0a249c9efb561dce2fa8b44946d02a7ebd6dd37b7b9d3f29aa668fa72 | lib.sh |
| 0cf441639bf5ceab4042a7446f35a940aea7c263099a12a2abef64985051d74c | run-all.sh |
| 4da6a50113ec66c922bca5e41e6ddee45299c045e3375b7dbc15088e6f1c05bc | v1-outpath-scenes.sh |

产物（`outputs/`）：

| sha256 | 文件 |
|---|---|
| 982718651b4c3b8857437be4c6149c79d3c5d84e3f1831bbbe3be185bb3c94f0 | h-a-late-stage-54.out |
| e57b2e6d80b6e127c2c556c45fa6f19a2e850e7dbd6699d34c627a8546724f12 | h-b-revert-stale-cell.out |
| e38fd72c814d75bf93f9715cb8831f94aaaa847e54d12441e68570f744658753 | h-b2-forced-recheck.out |
| 416caa18bcf821e643dd4bcd946da94b5ed9d33e0bd12a325d11daf28dc4361f | h-c-foreign-run-deletes.out |
| b89452f8124f4306453064943422ceb25a58576a8ed6d9f10eaa8eb06407108d | v1-outpath-scenes.out |
| 4b87697957d82203e13984e360598ac2da73314eb0c80fd2952c6cfc555b418a | fix-arms-g1.out |
| 7cbfb4615390a0fd29fd6f433447750f2eb9df5cd3a77c77632b15b76aa3160c | fix-arms-g2.out |
| c2b10d0311864496cba24a5c5cf4cdd435fcdb67eb8d714915c5b12594d2f2b9 | fix-arms-g4.out |
| 4a30cb806ba3b2f04933fc59ba0e5e662d468f92c5ef0042582f76d4af052e07 | fix-arms-g124.out |

怎么判对错：每一格末尾有一行「真值」——把门禁那一刻的 HEAD + 暂存区 clone 到另一个仓（另一个 common-dir，碰不到这一格的标记），用那棵树里的 54 号跑一次全量（分档之前那一份不认 `--full`，照它的默认带 `SINGLEFS_GATE_FULL=1` 跑）。门禁退出码与真值不同就是判错。「pre 臂」是同一段历史换成分档之前的 54 号（HEAD 那一份，门禁自己在临时 worktree 里跑全量）。

## 各格判定一览

| 格 | 历史（场景） | post 臂结果（用户动作放开扫） | pre 臂 | 打中哪一类（正文第 61 行的字面） | 分辨 |
|---|---|---|---|---|---|
| V1 H-A | X 暂存 `crates/` 改动、建 worktree 跑 `--full`；Y 往同一个暂存区放一个**只改 54 号**的改动（判得更严）。扫 Y 暂存的时刻 T0–T3 | **打中**：T1、T2 两格 X 的门禁绿、真值红；T3 X 门禁绿（对）、Y 门禁绿而真值红。4 个时刻里 3 个出了一处误绿；T0 对 | 4 个时刻都对 | 该红的没红（也是「该跑全量的没跑」：暂存区那份 54 号的全量一次没跑） | **分辨** |
| V1 对照 | 同上，Y 放的是登记的输入（`crates/`） | X 门禁红「没有这一格」，真值绿：要按出路再跑一趟。是设计给的红，不是误判 | — | — | — |
| V1-a | 暂存区为空：代码轮刚改完、没暂存，崩溃验证员照定义在主工作区跑快档 | 快档必红；照红句的三行跑完，写下的是 HEAD 那一格，验证员再跑仍红；`gate.sh --staged` 退 77 | 分档前验证员跑的就是全量，照内容判 | 不是误判；照定义做必红、出路三行在这个现场罩不到报红的那一份（与第一轮 S1 同类：次序问题） | 分辨（pre 不必红） |
| V1-b | 暂存区里只有别的会话的 `crates/` 改动 | 红 → 照出路跑 → 绿，真值绿 | — | 没打中 | — |
| V1-c | 出路第 1、2 行之间 HEAD 变了（别的会话提交整个暂存区 / 只提交它自己的路径） | `apply` 两种都失败、报错；第 3 行照跑，全量跑在没套上补丁的树上；门禁 77 / 红，都对 | — | 没有误判；白跑一趟全量，建法与 `gate.sh --staged` 不同（后者套不上就停） | — |
| V1-d | `$TMPDIR` 里留下的目录 | 两次 `mktemp -d` 各一个新目录，第二次不读第一次的 | — | 构造不出 | — |
| V2 H-B | `crates/` 逐字节退回到一份有旧绿格的内容，旧格是另一版 54 号（漂移=54）或另一版工具链（漂移=toolchain）写的；这一批照定义起 `--full`。扫 5 种用户动作 | **打中**：「`--full` 还在跑时起门禁」「54 号进程被 SIGKILL 之后起门禁」两种门禁绿、真值红；OOM（杀的是测试进程）、SIGTERM、等跑完三种门禁红（对）。两种漂移各 5 格里 2 格 | 两种漂移都红（对） | 该红的没红 | **分辨** |
| V2 H-B2 | 工具链升级之后，照 `change-touches-crates.sh` 第 27 行拿 `SINGLEFS_GATE_FULL=1` 在干净工作树上重新验 HEAD | **打中**：带不带 `--staged` 都绿（0、0），真值红 | 两种都红 | 该红的没红（环境变了而旧格照认） | **分辨** |
| V2 H-C | 同一批输入上另一趟 `--full` 在主工作区里判红：Z 拿没暂存的半成品 54 号试跑（wip），或 Z 用入库的 54 号跑、途中主工作区的 `crates/` 被改（edit）。扫两趟的先后 4 种 | **打中**：Z 后跑完（Zlast、overlapZlast）两格，EXIT trap 删掉 X 在 worktree 里写下的那一格，X 门禁红、真值绿；两种 Z 各 4 格里 2 格 | 都绿（对） | 不该红的红了 | **分辨** |
| V2 trap 覆盖面 | SIGKILL / OOM / 掉电 | SIGKILL 54 号本身：同一批输入前一趟那一格留着（H-B kill9 那一格）；OOM 通常杀的是占内存的测试进程：cargo 判红 → 54 号自己判红 → trap 照删（H-B oom 那一格，量过）；掉电：推的，见第二节 | — | 只有与 H-B 的漂移合起来才是误判 | — |
| V2 线程数 | 标记里记的线程数快档不核 | 没构造出按定义可达的触发（没人被定义要求设 `SINGLEFS_LAYER0_THREADS`） | — | 没打中（推的） | — |

打中三处（H-A、H-B / H-B2、H-C），病根是同一个：**一格的键只含登记的三条路径的内容**，判它的 54 号、工具链、写它的那一趟是不是在 HEAD + 暂存区的 worktree 里跑的，都不在键里；而「同一批输入的结果是确定的」那条前提（54 号第 11 行）在这三样变了时不成立。

## 一、V1　f5 的 worktree 与暂存区

### H-A　建完 worktree 之后，别的会话往暂存区放了一个只改 54 号的改动

**历史**（`h-a-late-stage-54.sh`，每格一个合成仓）。HEAD 的 54 号是 old：把「多核只起 1 个工作线程判红」那一判换成 `if false`，仿 2026-09-19 `fc7942f` 之前的 54 号（`git log -S'只起了 1 个工作线程'` 找到的就是这个提交，它同时把这一判加进 54 号、把 `crates/` 改成多线程）。new 就是今天入库的那一份。

1. X 改 `crash.rs`，内容里的线程数没传进去（假 cargo 打 `worker_threads=1`），暂存。许可它的：`.claude/main-agent.md` 第 49 行「主 agent 用 `research/scripts/stage-mine.py` 暂存」。
2. X 这一批改了登记的输入，照同一行在 HEAD + 暂存区的 worktree 里后台跑那棵树里的 `--full`；命令取 54 号出路函数第 199–201 行打出来的三行原样（V1-a 那一格核过：红句里打出的三行与函数里的三行 `cmp` 为 0）。
3. **Y** 在 T0（X 建 worktree 之前）/ T1（X 的全量跑的途中）/ T2（X 的全量跑完、X 起门禁之前）/ T3（X 的门禁之后）把 54 号换成 new 并暂存。Y 这一批只改 54 号，54 号不在它自己登记的输入里（`stage-inputs.tsv` 第 11 行），照第 49 行 Y 不起 `--full`。
4. X 的全量用 worktree 里那份（old）54 号判绿，按输入哈希写格。输入哈希只算登记的路径（54 号第 20、21 行），第 300 行取键。
5. X 起 `gate-triage`（模型里只跑 `gate.sh --staged` 那一段建法加 54 号）：临时 worktree 里是 HEAD + 暂存区，54 号已是 new；快档第 249 行按同一个哈希找格，第 253、261 行都过，判绿。T3 那一格再让 Y 起一次门禁。

逐行引文：

```
.claude/main-agent.md:49
| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里后台跑那棵树里的 `bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记）→ `gate-triage` |
.claude/gate.d/stage-inputs.tsv:11
54-layer0-replay.sh	crates/ Cargo.toml Cargo.lock	# 跑 crates 的崩溃点重放；两条流的用例编译期与运行期都不读 .claude/kb/layout/ 与 research/results/（段序列与布局表的比对归 52 号）
.claude/gate.d/54-layer0-replay.sh:20
# 输入：`.claude/gate.d/stage-inputs.tsv` 里登记给本阶段的路径（唯一登记位，复用判定读的也是它）。文件集是 git 眼里这些路径下
.claude/gate.d/54-layer0-replay.sh:249
  full_green_marker_path="$full_green_marker_prefix.$layer0_input_hash"
.claude/gate.d/54-layer0-replay.sh:300
full_green_marker_path="$full_green_marker_prefix.$input_hash_at_start"
```

post 臂 T1 原样（`outputs/h-a-late-stage-54.out`）：

```
════ H-A [post] Y 在 T1 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED
  ── X 的 --full（出路句三行原样，worktree 建于 Y 暂存之前）：
  ✓ 全绿标记写进 /tmp/claude-1000/defs54-r2-attack/runs/main/h-a-post-T1-SINGLE_THREADED-54/.git/singlefs-layer0-full-green.b42259802ccae17c3287fa23d8be8486
      [格] b42259802cca worker_threads=1 judged_root=…/tmp.poU65Fs6mr/tree
  ── X 起门禁（gate.sh --staged 的建法，只跑 54 号）；暂存区：.claude/gate.d/54-layer0-replay.sh crates/singlefs-harness/src/crash.rs 
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（b42259802ccae17c…，4 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T01:35:33Z，标记里的计数行原样
    [gate --staged 里 54 号退出码 0]
  ── 真值（X 门禁那一刻的 HEAD + 暂存区）
  ✗ 第一个事务那条流：本机 32 核、SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），全量枚举却只起了 1 个工作线程
    [真值：这份 HEAD + 暂存区用它自己的 54 号跑 --full，退出码 1]
```

（长行是模型打印时自己截在 150 字的，产物文件里就是这样。）pre 臂同一格原样：

```
════ H-A [pre] Y 在 T1 暂存 54；X 的 crash.rs 记号=SINGLE_THREADED
  ── X 起门禁（gate.sh --staged 的建法，只跑 54 号）；暂存区：.claude/gate.d/54-layer0-replay.sh crates/singlefs-harness/src/crash.rs 
  ✗ 第一个事务那条流：本机 32 核、SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），全量枚举却只起了 1 个工作线程
    [gate --staged 里 54 号退出码 1]
```

全部格（门禁退出码 / 真值退出码；T3 两次门禁先 X 后 Y）：

| 臂 | T0 | T1 | T2 | T3 |
|---|---|---|---|---|
| post，X 的内容单线程 | 1 / 1 | **0 / 1** | **0 / 1** | X 0 / 0，Y **0 / 1** |
| pre，同上 | 1 / 1 | 1 / 1 | 1 / 1 | X 0 / 0，Y 1 / 1 |
| post，X 的内容多线程（对照） | — | 0 / 0 | — | X 0 / 0，Y 0 / 0 |
| post，Y 放 `crates/`（对照） | — | 1 / 0 | 1 / 0 | — |

**四句**：
1. 分不分辨：分辨。pre 4 个时刻都对，post 3 个时刻各出一处误绿。
2. 被判的系统当时看不看得到：今天看不到。标记里没有写它的 54 号是哪一份，快档手里只有输入哈希；但快档自己的脚本就在手边（`$0`），记一个 sha256 进键就看得到，所以不是「判别子观测不到」那一种，是没记。
3. 满足判据字面的哪一个分句：正文第 61 行「该红的没红」；同一格也满足「该跑全量的没跑」（暂存区那份 54 号的全量一次没跑，第 49 行的触发条件不含 54 号自己）。
4. 跑前条款给的改法：正文第 61 行只写「改 54 号或定义」，没列具体改法。我提的几个见第三节：g1（判它的 54 号进键）在 T1、T2、T3 三格上都转成红（量过）。

**可达性要打的折扣**：要有两个会话共用一个暂存区，一个改 `crates/`，另一个同时改 54 号、改得更严，而前一个的内容恰好不满足更严的那一判。T3 那一格不要并发：Y 在 X 起过门禁之后才暂存，Y 自己的门禁照样绿。54 号的 `--full` 判定确实会变：`fc7942f` 加线程那一判，这一轮又动了一次。

**什么会推翻它**：主 agent 的派法里另有一句要求「这一批改了 54 号也要跑 `--full`」；或者快档另外核了写那一格的 54 号是不是自己。

### 出路句那三行在三种现场下给出什么（`v1-outpath-scenes.sh`）

出路函数原文（54 号第 199–201 行）：

```
  echo '                在项目根、暂存之后：layer0_full_base="$(mktemp -d)"; git diff --cached --binary > "$layer0_full_base/staged.patch"'
  echo '                git worktree add --detach "$layer0_full_base/tree" HEAD && { [ ! -s "$layer0_full_base/staged.patch" ] || git -C "$layer0_full_base/tree" apply --index "$layer0_full_base/staged.patch"; }'
  echo '                bash "$layer0_full_base/tree/.claude/gate.d/54-layer0-replay.sh" --full "$layer0_full_base/tree"; git worktree remove --force "$layer0_full_base/tree"'
```

**V1-a 暂存区为空。** 最常见的来路是崩溃验证员：`.claude/main-agent.md` 第 45 行把它排在代码轮之后、暂存之前，定义第 2 步 54 号默认快档、在主工作区跑。

```
.claude/main-agent.md:45
| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
```

原样（`outputs/v1-outpath-scenes.out` 里的整行，挑了几行；长行是模型打印时自己截在 150 字的）：

```
  ── 崩溃验证员的快档，退出码 1
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 74f88a29d4fc1e39…，4 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-attack/runs/m
  ── 红句里打出的三行与 54 号出路函数的三行逐字相同（cmp 0）
  ── 照红句三行原样跑完
  · --full 开跑（2026-09-24T01:38:32Z）：不删任何一格，这一趟判红才删这批输入那一格；这一道的输入哈希 ccec9defeddec16d…（4 个文件，登记路径 crates/ Cargo.toml Cargo.lock）
  ── 崩溃验证员再跑快档，退出码 1
  ── gate --staged（暂存区为空）
  ! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间一个改动都没有（干净工作树；要重新验 HEAD 用 SINGLEFS_GATE_FULL=1）
    [gate --staged 里 54 号退出码 77]
```

判法：验证员的红字面是对的（这一份内容确实没跑过全量），而照定义它在那个时刻只能红；出路三行套的是空暂存区，全量跑在 HEAD 上（`ccec9def…`），报红的是工作区那一份（`74f88a29…`），跑完照样红。这是一趟白跑的全量（真仓上是几个小时），不是误判。分档之前验证员跑的就是全量、按内容判，所以这一格分辨，但按正文第 61 行的字面三类都不是；与第一轮 S1 同类（定义给的次序），交主 agent 判要不要算。验证员定义第 24 行没说这一红在暂存之前是必然的，它会照第 3 步原样抄「✗」交回。

**V1-b 暂存区里只有别的会话的东西。** X 这一批只改文档，Y 暂存了 `crates/`、还没跑它的全量。X 的门禁红 → 照出路跑 → 再起门禁绿，真值绿。判对了；代价是 X 替 Y 跑了一趟全量，第 49 行的触发条件（「这一批改了」）不让 X 跑，出路句又让它跑，两处说法不一，没造成误判。

**V1-c HEAD 变了**（出路第 1 行跑完、第 2 行之前，别的会话提交）：

```
════ V1-c HEAD 变了：出路第 1 行跑完、第 2 行之前，别的会话提交（own-paths）
  ── 暂存区：crates/singlefs-harness/src/crash.rs crates/singlefs-harness/src/lib.rs 
  ── 插进来的提交之后 HEAD=94dda68，暂存区：crates/singlefs-harness/src/crash.rs 
error: patch failed: crates/singlefs-harness/src/lib.rs:1
error: crates/singlefs-harness/src/lib.rs: patch does not apply
```

补丁是对旧 HEAD 算的，套到新 HEAD 上两种提交法都失败；第 2 行的 `&& { …; }` 只挡住 `apply`，第 3 行照跑，全量跑在没套上补丁的新 HEAD 上、写下那一格。之后的门禁：别的会话提交了整个暂存区时 77（对），只提交它自己的路径时红「没有这一格」（对）。没有误判，是一趟白跑；与第 8 行「worktree 的建法与 `gate.sh --staged` 相同」字面不符：共享 `gate.sh` 第 75、76 行套不上就 `die`。

```
.claude/gate.d/54-layer0-replay.sh:8
#     两条流的全量枚举（「全量」「多线程」两段说的就是它），不问复用、不问改动范围，照跑。worktree 的建法与 `gate.sh --staged` 相同，命令在快档的出路句里。
.claude/singlefs-ai-sop/scripts/gate.sh:75
  if [[ -s "$staged_base/staged.patch" ]] && ! git -C "$staged_tree" apply --index "$staged_base/staged.patch"; then
```

**V1-d `$TMPDIR` 留下的目录。** 同一条出路连跑两次：留下 `tmp.eF1YXiiBLP/staged.patch` 与 `tmp.7zfUrReqzk/staged.patch` 两个目录，两份补丁不同，两格的 `judged_root` 各指各的目录，worktree 登记只剩主工作树。**构造不出**「下一次同名命令把上一次的目录当现场读进来」：卡在第 199 行每次都 `mktemp -d` 取新名字，第 2、3 行只经 `$layer0_full_base` 找目录。唯一的路是把三行拆开、在不同的 shell 里跑（变量丢了）：那时第 1 行写 `/staged.patch`、第 2 行建 `/tree`，非 root 都当场失败，响的（推的，没跑：不在根目录下造文件）。

## 二、V2　一批输入一格、失败才删

54 号头部把「开跑不删」立在一条前提上（第 11、12 行）：

```
.claude/gate.d/54-layer0-replay.sh:11
#     开跑一格都不删：同一批输入的结果是确定的，前一趟写下的那一格在这一趟跑的过程中照样算数。这一趟没写成标记就退出（判红、跑的过程中输入变了、
.claude/gate.d/54-layer0-replay.sh:12
#     被 TERM / INT / HUP 打断）时，退出前删这批输入那一格；被 SIGKILL 杀掉来不及删，前一趟那一格留着。别的格不动；全绿才写这一格。
```

「同一批输入」在代码里只指登记的三条路径的内容（第 20、21 行，第 300 行取键）。这一轮同一批里新改的 `stage-must-run.sh` 对复用判定立的规矩正好相反：

```
research/scripts/stage-must-run.sh:64
# 阶段脚本自己进比对：判据本身变了，上一次的判定就不再作数。
research/scripts/stage-must-run.sh:27
#   **环境变了而内容没变**——rustc 升级、系统库换了，树一模一样而结论已经不作数。
research/scripts/stage-must-run.sh:29
#   这里用**复用上限**替代：绿判定超过 SINGLEFS_REUSE_HOURS 小时（默认 24）就强制跑一趟并重新前移。
```

54 号的格既不含判它的 54 号，也不含工具链，也没有上限，「旧格只增不减」。前提不成立的三种情形各造了一条历史。

### H-B　退回到一份有旧格的内容（`h-b-revert-stale-cell.sh`）

**历史**：
1. 提交 A 这一批把 `crash.rs` 改成 C0，照第 49 行在 worktree 里跑 `--full`、写格 `f56ecab4…`（漂移=54：那时的 54 号是 old；漂移=toolchain：C0 带 `TOOLCHAIN_SENSITIVE`，那时的假工具链是 1），门禁绿，提交。
2. 提交 B：漂移=54 时把 54 号换成 new；漂移=toolchain 时把仓外的假工具链升到 2（仿 `rustup update`，不在仓里）。`crash.rs` 改成 C1，跑 `--full`、门禁绿，提交。
3. 这一批：`git checkout A -- crates/`（把 B 的 `crates/` 改动退掉），暂存。这一批改了登记的输入，照第 49 行后台起 `--full`（出路三行原样）。
4. **放开扫的用户动作**：`during` 全量还在跑时起 `gate-triage`（第 49 行写的是「后台跑 … → `gate-triage`」，没写等它跑完；第 11 行明写前一趟那一格在这一趟跑的过程中照样算数）；`kill9` 54 号那个 bash 被 SIGKILL（会话被整组杀掉、机器重启一类）之后起门禁；`oom` 占内存的测试进程被 SIGKILL（模型里杀的是假 cargo 进程）之后起门禁；`term` 54 号被 SIGTERM 之后起门禁；`finish` 等全量跑完再起门禁。

post 臂、漂移=54、`during` 原样：

```
  ── 这一批：git checkout A -- crates/，暂存区：crates/singlefs-harness/src/crash.rs 
      [格] 74f88a29d4fc worker_threads=32 judged_root=…/tmp.sDFcW7mxhC/tree
      [格] f56ecab4287c worker_threads=1 judged_root=…/tmp.mW0CMJ6bPb/tree
  ── --full 还在跑（54 号 pid 2198424），起门禁
  ✓ 层 0 快档跑完（release，只跑不标 ignored 的用例，全量那条留给 --full）：第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（f56ecab4287c8f55…，4 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T01:35:56Z，标记里的计数行原样
    [gate --staged 里 54 号退出码 0]
  ── 这一趟 --full 的判定行：
  ✗ 第一个事务那条流：本机 32 核、SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），全量枚举却只起了 1 个工作线程
      [格] 74f88a29d4fc worker_threads=32 judged_root=…/tmp.sDFcW7mxhC/tree
```

门禁绿在前，这一趟全量判红、trap 删掉 `f56ecab4…` 在后；门禁已经放行。post 臂、漂移=toolchain、`kill9` 原样：

```
  ── 54 号 pid 2204210 被 SIGKILL 之后起门禁
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（63342786f92e3d2d…，4 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T01:36:44Z，标记里的计数行原样
    [gate --staged 里 54 号退出码 0]
  ── 这一趟 --full 的判定行：
      （没打出判定行）
  ── 真值
  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）
    [真值：这份 HEAD + 暂存区用它自己的 54 号跑 --full，退出码 1]
```

pre 臂、漂移=54 原样：

```
  ── 起门禁（分档之前：门禁自己在临时 worktree 里跑全量）
  ✗ 第一个事务那条流：本机 32 核、SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），全量枚举却只起了 1 个工作线程
    [gate --staged 里 54 号退出码 1]
```

这一批的门禁退出码 / 真值（前两次门禁是提交 A、B 之前的，都是 0，略）：

| 漂移 | during | kill9 | oom | term | finish | pre |
|---|---|---|---|---|---|---|
| 54 | **0 / 1** | **0 / 1** | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 |
| toolchain | **0 / 1** | **0 / 1** | 1 / 1 | 1 / 1 | 1 / 1 | 1 / 1 |

**四句**：
1. 分不分辨：分辨（pre 两种漂移都红）。
2. 系统当时看不看得到：今天看不到（格里不记 54 号与工具链）；能记，见第三节 g1、g2。
3. 字面分句：正文第 61 行「该红的没红」。
4. 改法：g1 修漂移=54 的 during、kill9（量过），修不了漂移=toolchain（量过）；g2 反过来（量过）；g4 两样都修不了（量过）。

**可达性折扣**：`crates/` 与两个 Cargo 文件要逐字节回到一份写过格的旧内容（退掉一个坏提交就是）；中间 54 号的全量判定或工具链变过；再加「全量没跑完就起门禁」或「全量被整组杀掉之后照样起门禁」。第 49 行没写「等全量判绿再派 `gate-triage`」，第 11 行明写跑的过程中旧格算数。

**什么会推翻它**：第 49 行被读成「全量跑完且判绿之后才派 `gate-triage`」，而且被整组杀掉的全量算作「没跑完」要重跑——那样 `during`、`kill9` 两格都走不到门禁；H-B2 与 H-A 照样中。

### H-B2　工具链升级之后照「重新验 HEAD」的办法验，旧格照认（`h-b2-forced-recheck.sh`）

不要退回、不要并发。提交 A 的全量在假工具链 1 下绿、写格 → 工具链升到 2 → 工作树干净 → 照 `change-touches-crates.sh` 第 27 行重新验 HEAD：

```
research/scripts/change-touches-crates.sh:27
# `SINGLEFS_GATE_FULL=1` 强制当成碰了，用来在干净工作树上重新验一遍 HEAD。
```

post 臂原样（挑的整行）：

```
  ── 工具链升到 2；工作树干净：0 行 status
      [格] 63342786f92e worker_threads=32 judged_root=…/tmp.48W3DX9UXb/tree
  ── SINGLEFS_GATE_FULL=1，带 --staged 的建法
  ✓ 这批输入那一格全绿标记与这批输入的内容哈希相同（63342786f92e3d2d…，4 个文件，登记路径 crates/ Cargo.toml Cargo.lock），两条流都是 exhaustive=true：层 0 全量跑完于 2026-09-24T01:37:45Z，标记里的计数行原样
    [gate --staged 里 54 号退出码 0]
  ── SINGLEFS_GATE_FULL=1，主工作区直接跑 54 号
    [退出码 0]
  ── 真值（工具链 2 下这份 HEAD 用它自己的 54 号跑全量）
  ✗ 层 0 崩溃点重放的用例判红（上面是 cargo test 的尾部）
    [真值：这份 HEAD + 暂存区用它自己的 54 号跑 --full，退出码 1]
```

pre 臂两种都红（`[gate --staged 里 54 号退出码 1]`、`[退出码 1]`）。`stage-must-run.sh` 的 24 小时上限在这里也不起作用：上限到了，它让 54 号「要跑」，跑的是快档，快档照认旧格。

**四句**：分辨（pre 红）；系统看不到（格里不记工具链）；「该红的没红」；g2 修得了（量过：带 `--staged` 与直接跑都转成 1），g1、g4 修不了（量过）。
**可达性折扣**：假工具链只是一个开关；真的 rustc 升级让同一份代码的全量结论翻面，要靠未定义行为、编译器缺陷或依赖在 `Cargo.lock` 之外变了一类，稀少。它量的是「54 号有没有把这一格挡住」，不是「这事常不常见」。另一处打折：`SINGLEFS_GATE_FULL=1` 在分档之后的本意也许就只是「强制跑快档」（54 号第 91 行「要强制跑：SINGLEFS_GATE_FULL=1 再跑一次」），那样这一格就只剩「环境变了没有任何一道会重跑全量」这一句。
**什么会推翻它**：有一处定义要求工具链升级后重跑层 0 全量，或格里已经记了工具链版本。

### H-C　同一批输入上另一趟 `--full` 判红，trap 删掉别人的绿格（`h-c-foreign-run-deletes.sh`）

**历史**：
1. HEAD 的 54 号是 new。X 改 `crash.rs`（多线程，入库 54 号的全量该绿），暂存；主工作区的 `crates/` 与暂存区相同（X 刚把自己的改动全暂存了）。
2. X 照第 49 行在 worktree 里跑 `--full`，判绿，写格 `74f88a29…`。
3. Z 在主工作区里跑 `bash .claude/gate.d/54-layer0-replay.sh --full`（不带根，第 53 行取主工作区），输入哈希与 X 那一格相同。两种 Z：
   - wip：Z 是正在改 54 号的会话，主工作区里的 54 号是它没暂存的半成品（模型里多了一判「`LAYER0` 行要 `journal_differing=0`」，合成日志里是 3），拿半成品试跑。
   - edit：Z 用入库的 54 号，例如崩溃验证员被派发提示点名跑全量（定义第 24 行「派发提示点名要层 0 全量时才带」，第 2 步在主工作区起）；跑的途中主工作区的 `crates/` 被人接着改（没暂存）。
4. Z 判红（wip：半成品那一判；edit：第 367、368 行「跑的过程中输入变了」），EXIT trap（第 303 行）删掉第 300 行那个键——就是 X 那一格。
5. X 起 `gate-triage`，快档红「没有这一格」。
6. **放开扫的**：两趟的先后，Zfirst / Zlast / overlapZlast / overlapXlast。

```
.claude/gate.d/54-layer0-replay.sh:303
trap 'if [[ "$full_marker_written" != 1 ]]; then rm -f -- "${full_green_marker_path:?}"; fi; rm -rf -- "${layer0_scratch_directory:?}"' EXIT
.claude/gate.d/54-layer0-replay.sh:23
# 主工作区里跑的 --full 读的是工作区（连同别的会话没暂存的改动、工作区那一份 54 号），罩不到这一批暂存内容；所以 --full 在 HEAD + 暂存区的 worktree 里跑。
```

第 23 行说主工作区的全量罩不到暂存内容，但没挡它跑、也没挡它的 trap 删格。wip、Zlast 原样：

```
  ── X 那一趟（worktree，入库的 54 号）：
  ✓ 全绿标记写进 /tmp/claude-1000/defs54-r2-attack/runs/main/h-c-post-wip-Zlast/.git/singlefs-layer0-full-green.74f88a29d4fc1e39c65611dd687a300f06693668396f
  ── Z 那一趟（主工作区）：
  ✗ （别的会话改到一半的 54 号）LAYER0 行 journal_differing 不是 0
      [格] 一格都没有
  ── X 起门禁
  ✗ 快档绿了（第一个事务那条流 5 条通过、1 条 ignored；两次发布那条流 5 条通过、1 条 ignored），但这批输入（哈希 74f88a29d4fc1e39…，4 个文件）没有层 0 全量的全绿标记（/tmp/claude-1000/defs54-r2-attack/runs/m
    [gate --staged 里 54 号退出码 1]
  ── 真值
    [真值：这份 HEAD + 暂存区用它自己的 54 号跑 --full，退出码 0]
```

edit、overlapZlast 里 Z 的红句原样：`  ✗ 全量跑的过程中这一道的输入变了（开跑 74f88a29d4fc1e39…，跑完 bed1948cbf756b05…）：两条流读到的不一定是同一版，不写全绿标记`，之后 X 的门禁同样 1、真值 0。

| Z | Zfirst | Zlast | overlapZlast | overlapXlast | pre（Zlast） |
|---|---|---|---|---|---|
| wip | 0 / 0 | **1 / 0** | **1 / 0** | 0 / 0 | 0 / 0 |
| edit | 0 / 0 | **1 / 0** | **1 / 0** | 0 / 0 | 0 / 0 |

**四句**：
1. 分辨：分辨（pre 门禁自己跑全量，Z 碰不到它）。
2. 系统当时看不看得到：删格的那一趟看得到自己的根是主工作区、不是 HEAD + 暂存区的 worktree，也看得到自己的红是「输入变了」；今天的 trap 不看这些。
3. 字面分句：正文第 61 行「不该红的红了」。
4. 改法：g1 修 wip 两格（Z 的半成品 54 号算出另一个键，删的是它自己那一格；量过），修不了 edit（同一份 54 号，同一个键）；g4 修 edit 两格（量过），修不了 wip；g124 四格都修（量过）。

**可达性折扣**：要有人在主工作区对同一份内容跑 `--full`。定义里点得到的是崩溃验证员被点名跑全量（edit 那一种还要途中有人改 `crates/`），与正在改 54 号的会话试跑（wip）。崩溃验证员定义第 23 行挡的是「另有 `--full` 在跑时不起」，挡不住 Zlast：X 那一趟已经跑完。代价只是多一趟全量（误红，不是误绿），但全量在真仓上是几个小时。

### trap 在 SIGKILL、OOM、掉电下留下哪一格

- **SIGKILL 54 号那个 bash**（量过，H-B `kill9`）：trap 不跑，同一个键上前一趟写下的格留着；那一格只要还对，这一步不出错，对不上（H-B 的漂移）就误绿。
- **OOM**（量过，H-B `oom`）：内核挑的多半是占内存最多的测试进程，不是 54 号的 bash；测试进程死了 → cargo 判红 → 54 号走第 307–313 行判红退出 → trap 照删。门禁红，对。
- **SIGTERM**（量过，H-B `term`）：trap 跑，删，门禁红。
- **掉电**（推的，没跑）：标记先写 `.partial.<pid>` 再 `mv -f`（第 375、389 行），没有 fsync；`.git` 在 ext4 上（`df -T` 现查）。新名字的 rename 不触发 ext4 的 `auto_da_alloc`，掉电后可能剩一个零长的格：快档第 261 行读到空哈希判红，红句说「那一格被改过或拷错了」，说错了原因，但不误绿。前一趟的格不受影响。
- **线程数**（推的，没跑）：格里记了线程数（第 384 行），快档不核；一趟 `SINGLEFS_LAYER0_THREADS=1` 的全量写下的格照样被认。要误判得有一份在多线程下才错的内容，而且有人显式设了 1——54 号第 147 行教人这么设，但没有哪份定义要求主 agent 这么跑，按定义可达的触发我构造不出。

## 三、改法各修哪一格

改法全部是我提的，**只在我的模型上量过、被攻过零轮**。正文第 61 行的跑前条款只写「改 54 号或定义」，没列具体改法，所以没有「条款给的改法」可逐个核；下表核的是我提的这几个。「量过」的格贴的是 `outputs/fix-arms-<改法>.out` 里的门禁退出码 / 真值退出码（副本上的数，不是入库装置上的数）；「推的」是按代码推、没实现没跑。

- **g1**：判这一格的 54 号自己进输入哈希（`$0` 的 sha256 当一行清单）。`fixes/54-g1.sh`，比原文多两行。
- **g2**：工具链进输入哈希（`cargo -V` 的原样当一行清单）。`fixes/54-g2.sh`。
- **g4**：「跑的过程中输入变了」那一支判红时不删开跑那一格（trap 只在 `full_marker_written` 为 0 时删）。`fixes/54-g4.sh`。
- **g124**：三样一起。
- **f4b**（第一轮攻方提、第一轮判决没采纳）：标记里记写它的 54 号的 sha256，快档与自己比。键不变。
- **g6**：改定义，不改脚本：第 49 行写明「`--full` 判绿之后才派 `gate-triage`；没正常跑完（被杀、被打断）就重跑」。

| 改法 | H-A T1 | H-A T2 | H-A T3（Y 的门禁） | H-B 54 during | H-B 54 kill9 | H-B 工具链 during | H-B 工具链 kill9 | H-B2 | H-C wip Zlast | H-C wip overlapZlast | H-C edit Zlast | H-C edit overlapZlast |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 不改（第一、二节） | 0 / 1 中 | 0 / 1 中 | 0 / 1 中 | 0 / 1 中 | 0 / 1 中 | 0 / 1 中 | 0 / 1 中 | 0 / 1 中 | 1 / 0 中 | 1 / 0 中 | 1 / 0 中 | 1 / 0 中 |
| g1 | 修（量过 1 / 1） | 修（量过 1 / 1） | 修（量过 1 / 1） | 修（量过 1 / 1） | 修（量过 1 / 1） | 还中（量过 0 / 1） | 还中（量过 0 / 1） | 还中（量过 0 / 1） | 修（量过 0 / 0） | 修（量过 0 / 0） | 还中（量过 1 / 0） | 还中（量过 1 / 0） |
| g2 | 还中（量过 0 / 1） | 还中（量过） | 还中（量过） | 还中（量过 0 / 1） | 还中（量过） | 修（量过 1 / 1） | 修（量过 1 / 1） | 修（量过 1 / 1） | 还中（量过 1 / 0） | 还中（量过） | 还中（量过） | 还中（量过） |
| g4 | 还中（量过） | 还中（量过） | 还中（量过） | 还中（量过） | 还中（量过） | 还中（量过） | 还中（量过） | 还中（量过） | 还中（量过 1 / 0） | 还中（量过） | 修（量过 0 / 0） | 修（量过 0 / 0） |
| g124 | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 修（量过） |
| f4b | 修（推的） | 修（推的） | 修（推的） | 修（推的） | 修（推的） | 还中（推的） | 还中（推的） | 还中（推的） | 还中（推的：键没变，Z 的 trap 照删 X 那一格） | 还中（推的） | 还中（推的） | 还中（推的） |
| g6 | 还中（推的：X 的全量是跑完判绿的） | 还中（推的） | 还中（推的） | 修（推的，要照做） | 修（推的，要照做） | 修（推的） | 修（推的） | 还中（推的） | 还中（推的） | 还中（推的） | 还中（推的） | 还中（推的） |

对照格（改法不该弄坏的）：
- H-B 漂移=54 `finish`（该红）：g1、g2、g4、g124 都是 1 / 1（量过）。
- H-A X 的内容多线程、Y 在 T1 暂存 54 号（真值绿）：g2、g4 仍 0 / 0；**g1 与 g124 转成 1 / 0**（量过）——X 那一趟是旧 54 号跑的，新 54 号没跑过，g1 让门禁要求重跑一趟。这是 g1 的代价，不是它修错了：它把「该跑全量的没跑」当红。

各改法的代价（都是推的）：
- g1：54 号每改一个字节，所有格作废；下一批碰 `crates/` 的照第 49 行本来就跑全量，多出来的只有「别人在我跑完之后才暂存了 54 号」那一种（上面对照格），多一趟全量。第 49 行的触发条件要不要加上「这一批改了 54 号自己」，要另判（不加时只改 54 号的一批在快档第 87–93 行（「这次改动没碰它判的东西」）退 77，新 54 号的全量拖到下一批碰 `crates/` 时才跑，与分档之前一样不分辨）。
- g2：工具链一升级，所有格作废；只在碰 `crates/` 的那一批或强制跑时要求重跑。
- g4：输入变了的那一趟不删格，只要开跑那一格是同一份 54 号写的就没害处；与 g1 合用才完整。
- f4b 与 g1 的差别：f4b 只在读格时比，删格仍按内容哈希删，所以挡不住 H-C wip。
- g6：靠主 agent 照做，门禁本身不判；H-B2（不经第 49 行）碰不到。

没有哪一个单独修得了全部格：g1 修 H-A、H-B 漂移=54、H-C wip；g2 修 H-B 漂移=工具链、H-B2；g4 修 H-C edit。

## 没打中的形状

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| 建完 worktree 之后别人往暂存区放**登记的输入**（`crates/`） | H-A 对照，T1、T2 各一格 | 门禁红「没有这一格」，真值绿：哈希对不上、要按出路再跑。这是设计给的红（「这一份没跑过全量」），不算误判 |
| 建完 worktree 之后别人往暂存区放不影响全量结论的改动（多线程内容 + 54 号） | H-A 对照，T1、T3 | 门禁绿、真值绿，判对 |
| 同一份 54 号、同一工具链下，退回到有旧格的内容，全量没跑完就起门禁 | 没单独跑；H-B 的 `during` 去掉漂移就是这一格 | 前提（确定性）成立时旧格就是对的答案，构造不出误判 |
| 两趟同一份 54 号的 `--full` 在同一批输入上撞车、都绿 | 读代码；实现员第三轮报告自证过「同一批输入两趟 --full 撞车不误红」 | 构造不出 |
| SIGKILL 留下的 `.partial.<pid>` 被当成格 | 读代码：快档按确切文件名找（第 253 行），列最近一格时 `find ! -name '*.partial.*'`（第 208 行） | 构造不出 |
| 出路三行拆到不同的 shell 里跑（变量丢了） | 推的，没跑 | 第 1、2 行在根目录下写文件、建目录，非 root 当场失败，响；不误判 |
| `$TMPDIR` 里上一次的 `staged.patch` 被下一次读进来 | V1-d，连跑两次 | 构造不出（每次 `mktemp -d` 新名） |
| 暂存区为空 / 只有别人的东西 / HEAD 在第 1、2 行之间变了 | V1-a、V1-b、V1-c 各一或两格 | 没有误判；V1-a 必红、V1-c 白跑一趟全量，见第一节 |
| 标记里的线程数不核 | 推的 | 没有按定义可达的触发 |
| 掉电 | 推的 | 最坏零长格 → 快档红（原因说错），不误绿 |

扫过的用户动作：H-A 扫 Y 暂存的 4 个时刻 × Y 放什么 2 种 × X 的内容 2 种（跑了 12 格，含 pre 臂）；H-B 扫全量起来之后 5 种动作 × 2 种漂移（加 pre 共 12 格）；H-B2 扫带不带 `--staged`；H-C 扫两趟先后 4 种 × Z 的 2 种（加 pre 共 10 格）。每格只跑一次：脚本是确定性的（开发时的一趟与入库这一趟，H-A、H-B、H-C、V1 的每格门禁与真值退出码相同；H-B2 是后加的，它的 pre 臂真值在第一次整趟 `run-all.sh`（01:25–01:34 UTC）里退 77，是我的真值函数没越过「没碰输入」那一问，改了 `lib.sh` 之后整趟重跑，入库的是重跑的）。

## 这条腿自己的限度

- 假 cargo 把「编译并跑」简化成「读调用那一刻 `crash.rs` 里的记号」；真 cargo 的增量编译、文件锁都没模拟；时间窗用 hold 文件（`FAKE_CARGO_HOLD`）卡住，不是几个小时。
- 只仿了 `gate.sh --staged` 里 54 号那一段（建法照共享 `gate.sh` 第 55–76 行：`diff --cached --binary`、`worktree add --detach HEAD`、`apply --index`，再设 `SINGLEFS_STAGED_TREE`），没跑整轮；`refs/sop/staged-green` 在合成仓里不存在，复用那一问恒答「要跑」。
- H-A、H-B 的「更严的 54 号」是照 `fc7942f` 的形状倒推的（把线程那一判拿掉当旧版），不是一次真的未来改动；H-B、H-B2 的工具链漂移是一个开关，真实的 rustc 升级让结论翻面很少见。
- H-C 的 wip 半成品是我造的一判（`journal_differing=0`）。
- 合成仓的 `crates/` 只有两个文件；「真值」用的是同一套假 cargo，真值本身不是层 0 真跑。
- 开发时有一趟改法对比，我在它跑的途中改了它的驱动脚本，那一趟的尾巴报了语法错；那一趟整个作废，入库的 `outputs/` 全部来自 01:35–01:43 UTC 那一趟干净的 `run-all.sh`。
- 报告里贴的块是产物里挑出来的整行（`grep -cxF` 核过几条在产物里整行命中），不是整段；整段在 `outputs/` 里。

## 没做什么

- 没判 V3、V4、V5（归本地攻方与辩方），没替主 agent 采纳改法；g1、g2、g4、g6 都是被攻过零轮的提议。
- 没编译 Rust，没跑真全量，没跑 `gate.sh` 全量，没在真仓里做任何 git 写；真仓的 git common-dir 一格标记都没写（`singlefs-layer0*` 现查 0 个）。副本与合成仓上的数不算入库装置上的数，要引得在入库装置上重做。
- 阶段归属表里没有登记给 three-way-attack 的阶段（`stage-owners.tsv` 现查 0 行），没跑门禁阶段。
- 草稿目录 `/tmp/claude-1000/defs54-r2-attack/`：`runs/` 是合成仓（可删），`model/` 是开发时的模型副本，入库那一份以 `research/prompts/defs-gate54-tiering-r2-opus-model/` 为准；开发时的 `model/outputs/` 是早先那几趟，不作依据。

## 附：上文按行号点到、没在正文整行抄的原文

```
.claude/agents/crash-verifier.md:23
1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
.claude/agents/crash-verifier.md:24
2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
.claude/gate.d/54-layer0-replay.sh:91
    echo "     → 要强制跑：SINGLEFS_GATE_FULL=1 再跑一次；前缀是 stage-inputs.tsv 登记给本阶段的 ${layer0_input_paths_text}，判据见 research/scripts/change-touches-crates.sh。"
.claude/gate.d/54-layer0-replay.sh:147
    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_LAYER0_THREADS=1 bash .claude/gate.d/54-layer0-replay.sh。"
research/prompts/_defs-gate54-tiering-r2-body.md:61
| 一格判「站不住」⇒ 改 54 号或定义；按「第三轮之后停」，第三轮只攻前两轮都站住的形态 | 攻方给出一条可达历史：该跑全量的没跑、不该红的红了、该红的没红，或 agent 照定义做会覆盖别人的东西 |
```
