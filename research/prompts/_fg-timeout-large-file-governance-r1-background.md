# 前台 timeout 过长与提交前大文件两道执行前拒：治理文件轮第一轮（fg-timeout-large-file-governance-r1，2026-11-18）

这一轮判的是检出钩子 `.claude/hooks/bash-command-detector.sh` 新加的 ㉑、㉒ 两道执行前拒，`scripts/gate/large-file-check.py` 新加的 `--guard`，以及 `.claude/agent-common.md` 跟着改的几句。三份都已换上主工作区。

用户原话有两句。第一句是「那条前台命令的超时设成了 600000 毫秒 这个问题 出现了很多次 拦截这个写法」，门槛由用户弹窗定为「超过 240000 就拒」。第二句是「大文件压缩的钩子在提交时候没有生效」：超过 100 MiB 的产物两次进了没推的提交，只能重做那几笔提交。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 换上的实现做的是不是文件头 ㉑、㉒ 两段与 `large-file-check.py` 文件头 `--guard` 那一段说的事（正推腿）。
2. 哪一格会错：该拒的放过、不该拒的误拒、把钩子拖慢或弄挂、拒绝说明给的出路照做不通、文档与代码对不上（攻方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；任务号后面是任务日志里的原句）：

- 这一轮没有改 `crates/` 下的代码，新加的代码也不读 `crates/` 下的任何文件。
- **㉑**：文件头第 418 到 423 行。判法是 `foreground_timeout_refusal`（第 3085 行），门槛是常量 `FOREGROUND_TIMEOUT_MILLISECONDS = 240000`（第 3083 行），拒绝说明由 `foreground_timeout_messages`（第 3098 行）写，出路按主 agent、子 agent 分开写。原来 ③ 只记子 agent 的检出、不拒，那一支连同 `subagent_timeout_findings` 已删。
- **㉒**：文件头第 424 到 443 行。
  - 参数怎么认：`git_commit_form`（第 3128 行）、`git_push_judged`（第 3169 行）、`git_add_form`（第 3186 行）。
  - 起判法：`large_file_check_run`（第 3229 行）起 `<根>/scripts/gate/large-file-check.py --guard <模式>`，时限 `LARGE_FILE_CHECK_SECONDS = 25`（第 3114 行）。
  - 汇总与拒绝说明：`large_file_findings`（第 3254 行）汇总，`large_file_messages`（第 3358 行）写拒绝说明。
  - 自证压小门限：自证经 `selftest_limit_bytes`（第 3219 行）读环境变量 `BASH_COMMAND_DETECTOR_LARGE_FILE_LIMIT_BYTES`，只在自证标记对得上时认。
- **`--guard`**：`scripts/gate/large-file-check.py` 文件头第 26 到 39 行说用法与退出码。
  - 入口是 `guard`（第 377 行）与 `main_guard`（第 395 行）。
  - 三种模式：commit-index（`guard_staged`，第 325 行，跟 `GIT_INDEX_FILE`）、commit-worktree（`guard_worktree`，第 343 行）、push（`guard_unpushed`，第 367 行）。
  - 输出与上限：标准输出只打一行 JSON，上限常量 `SINGLE_FILE_SIZE_LIMIT_BYTES` 与不带 `--guard` 时是同一份。
- **共用约束**：`.claude/agent-common.md` 第 70 行改成主 agent 与子 agent 的前台 timeout 都不超过 240000 毫秒、超过的执行前拒；第 75 行检出钩子那一条的括号里补了「② ㉑ 只在前台拒」；第 96、97 行新加 ㉑、㉒ 两项。
- **中途改过的两处**：
  - 第一处：`large_file_check_run` 原来收的是脚本路径形参，算键的库折不出，开关复跑「140 条里拒算 140 条」（任务 20578）。现在改成收根，起命令那一行写字面尾巴 `os.path.join(script_root, "scripts", "gate", "large-file-check.py")`；在快照树上现算闭包，拒算 0 行（任务 20594）。
  - 第二处：修第一处时，自证一格的格名改了、文件头点名没跟着改，开关复跑「2 处不对」（任务 20600），格名已改回。
- **验过的**：
  - 换上：`large-file-check.py` 是任务 20477；检出钩子是任务 20481、20597、20620，最后一版 sha256 b7627f14…。
  - 门禁与样本档：门禁 code-tooling 的 committed-file-size 格「甲 查了 14496 个跟踪的文件」绿（任务 20478）；code-tooling 样本档「点名的 1 道阶段判得都对（239 个用例）」（任务 20480）。
  - 自证：检出钩子自证「✓ 自检通过（查了 1420 种）」（任务 20599）。
  - 开关复跑：任务 20623「复跑了 139 条开关（1 份），自证红且点名的格与实际红的格逐格相同 139 条」「现判 140、复用 0、拒算 0」。
- **工具员报告里写明的局限**（交腿判该不该算缺口）：
  - git push 只判当前分支相对上游。
  - merge、cherry-pick、rebase、commit-tree 不判。
  - `-a` 取暂存区与工作区的并集，只会多拒。
  - LFS 与 autocrlf 转换后的大小不算。
  - 只含数字的字符串 timeout 当数拒。
  - 工具员报告在 `~/.local/share/singlefs/work/fg-timeout-large-file-guards/result.md`。
- 工具层的背景（任务服务、换上工具怎么用）在 `.claude/kb/tooling.md`。

## 范围

这一轮被判的文件（都读换上之后的那一版）：

- `.claude/hooks/bash-command-detector.sh`：只判文件头 ㉑、㉒ 两段（第 418 到 443 行）、开关表里 ㉑、㉒ 那九条（第 871 到 901 行）、第 3083 到 3370 行的函数、`main()` 里调这两道的地方、自证里「前台超时:」与「提交前大文件:」两组格（第 6530 行、第 8454 行、第 8566 行起）。这份文件里别的会话这几天改的 ⑫、⑭ 与新加的 ㉓ 不在这一轮判。
- `scripts/gate/large-file-check.py`：`--guard` 那一半（文件头第 26 到 39 行，函数第 299 到 425 行）与它和不带 `--guard` 的两半共用的上限、`exceeds`。
- `.claude/agent-common.md`：第 70、75、96、97 行。

这一轮的改动相对 HEAD 的样子：`git diff HEAD -- scripts/gate/large-file-check.py .claude/agent-common.md`；检出钩子那一份的 diff 里混着别的会话的改动，只看「范围」点名的行。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（一条 hook 输入 JSON、一串 git 操作、一条命令）或 文件:行。

正推腿：

1. ㉑ 那一段每一句，逐句对 `foreground_timeout_refusal` 与 `main()` 里的调用：哪些放行、哪些拒，主 agent 与子 agent 的出路各写什么，和别的拒绝一起列的次序。
2. ㉒ 那一段每一句，逐句对 `git_commit_form`、`git_push_judged`、`git_add_form`、`large_file_findings`：-a、路径、-o / -i、排在前面的 git add、`GIT_INDEX_FILE`、不判的那几种、判不了的那几种、出路的字面。
3. `--guard` 三种模式与退出码，逐句对文件头第 26 到 39 行；上限是不是只有 `SINGLE_FILE_SIZE_LIMIT_BYTES` 一份、`exceeds` 是不是同一个函数。
4. `.claude/agent-common.md` 那四行，与钩子今天的行为逐句对得上没有；用户要的两件各落在哪一行，有没有多做的。

攻方腿：

5. ㉑ 漏拒与误拒：timeout 写成浮点、负数、科学计数、带空格或单位的字符串、超大整数时各判成什么；`run_in_background` 写成字符串 "true" 时算前台还是后台；主 agent 照拒绝说明改成 run_in_background 之后，会不会撞上别的道（例：子 agent 的 ⑩、④ 的单独 `&`）。
6. ㉒ 漏拒：造得出「超过上限的文件进了提交或推了出去，㉒ 却放行、也不记检出」的写法吗？至少试这几形：`git commit --amend`、`git commit -F`、`git -c … commit`、`command git commit`、别名（`git ci`）、`git stash` 之后 `git commit`、`git add -u && git commit`、`git push origin HEAD:other`、`git push --all`、`git push --mirror`、子模块、`xargs git commit`。
7. ㉒ 误拒与拖慢：工作区改大了而没暂存的跟踪文件、被忽略的大文件、带空格与非 ASCII 的路径、软链接指到仓外大文件的时候各判成什么。一条命令里有几条 git commit / push 时，判法起几次、合计最长多久；`.claude/settings.json` 里这个钩子注册的时限够不够。对象库很大时 `guard_staged` 与 `guard_unpushed` 要多久。
8. 判法出错与钩子挂掉：`large-file-check.py` 抛 Python 异常、输出多行、退 3、git 不在 PATH 时，钩子是放行加记检出，还是自己抛异常、退别的码。
9. 出路照做通不通：拒绝说明给的 `gzip -n -9 -k`、`git rm --cached`、`git add <文件>.gz` 照抄执行，会不会被项目里管 Bash 的那几道闸拒（⑮ 清索引、⑱ 撤改动、⑰ 字面内容）；做完之后再 `git commit` 是不是放行。
10. 自证与开关：九条开关里有没有两条弄坏同一处、或者点名的格靠别的开关也会红；自证里真 git 仓的格过不过 `scripts/gate/selftest-process-guard.py` 盯的那几样；门限压小的环境变量在平常跑时真的不认吗。
11. 多机：钩子起判法时用的 `repository_root_of` 在仓副本、git worktree、第二台上各读哪一份 `large-file-check.py`。

## 分工

正推腿做第 1 到 4 问，攻方腿做第 5 到 11 问。两条腿都读「范围」里的三份文件；只有一条腿读到的范围，在报告里写明。要跑的探针（临时小仓里的 git 操作序列、喂钩子的 JSON）不自己跑，照「要主 agent 跑的命令」写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条。

### 小节清单：`.claude/kb/tooling.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| # 工具与环境事实 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （工具与环境事实 标题之下、第一个下级标题之前的正文：第 2-7 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ## 本地 LLM（三方论证不派它当腿） | 抄 | 默认全抄（脚本生成） |
| ## 仓里的性能数都只在本机成立，引用要带机器口径 | 抄 | 默认全抄（脚本生成） |
| ## 跑项目代码的命令一律经 task-server 交 | 抄 | 默认全抄（脚本生成） |
| ## 变异工具：并发不超过核数，不限时，卡住由统一任务服务判 | 抄 | 默认全抄（脚本生成） |
| ## 变异条数会随源码漂，不许在 kb 里钉死 | 抄 | 默认全抄（脚本生成） |
| ## 改了引用方，要 grep 全仓改被引用方 | 抄 | 默认全抄（脚本生成） |
| ## 一个数不许从「它自己判为非法」的配置里算出来 | 抄 | 默认全抄（脚本生成） |
| ## 未定项一律写成编号条目，标题一律声明条数 | 抄 | 默认全抄（脚本生成） |
| ## 验证产物放哪：四个位置，各管一段 | 抄 | 默认全抄（脚本生成） |
| ## 内核源码：本机有完整树，不必 WebFetch | 抄 | 默认全抄（脚本生成） |
| ## 到点自己开工：用 remind.sh 登记 | 抄 | 默认全抄（脚本生成） |
| ## Rust 工具链 | 抄 | 默认全抄（脚本生成） |
| ## RocksDB 与 snappy 的预编库：每台主机编一次，取值只走进程环境 | 抄 | 默认全抄（脚本生成） |
| ## 虚机 benchmark harness（E7（离线索引 harness） 用） | 抄 | 默认全抄（脚本生成） |
| ## 造几何量不同的设备：不需要真硬件 | 抄 | 默认全抄（脚本生成） |
| ### QEMU 路线（推荐：无需 root，直接接现有 harness） | 抄 | 随父节带到 |
| ### 宿主侧路线（要 root，胜在不用启虚机） | 抄 | 随父节带到 |
| ## `scripts/mutation/mutate.sh` 的结局分类 | 抄 | 默认全抄（脚本生成） |
| ## 几个会话共用一个仓时怎么暂存与提交 | 抄 | 默认全抄（脚本生成） |
| ## 复跑与变异脚本要在 `scripts/experiment-apparatus/` 下跑 | 抄 | 默认全抄（脚本生成） |
| ## 批量立 / 定分项时门禁会红的形态 | 抄 | 默认全抄（脚本生成） |
| ## 本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个 | 抄 | 默认全抄（脚本生成） |

**出处 `.claude/kb/tooling.md:1-7`（整段抄，未转述）**

```markdown
# 工具与环境事实

**中文** · [English](tooling.en.md) · [日本語](tooling.ja.md)

**tooling.md 放「本工程用到的工具现在是什么状态、用的时候怎么做」**。
规则不进 tooling.md，进 [CLAUDE.md](../../CLAUDE.md) 或 `.claude/rules/`。

```

**出处 `.claude/kb/tooling.md:8-26`（整段抄，未转述）**

```markdown
## 本地 LLM（三方论证不派它当腿）

三方论证只派两条云端腿加主 agent 判决，不派本地模型的腿。仓里调网关的脚本与字词损坏检测器（`ask-local.sh`、`ask-local-selftest.sh`、`local-repro.sh`、`local-repro-tuned.sh`、`corruption-check.py`、`corruption-summary.py`、`oov-check.py`、`en-words.txt`）2026-10-11 删掉了；要再用，`git log --diff-filter=D -- scripts/agents/ask-local.sh` 找到删它的那次提交，从它的上一版取回。

| 项 | 值 |
|---|---|
| 入口 | 本机的本地模型网关（OpenAI 兼容，地址与所在目录不进仓） |
| 鉴权 | 请求头 `X-Api-Key`，取值在本机私有配置里，不进仓 |
| 模型 | `local` → `Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`，`max_model_len` 262144 |
| 调用脚本 | 仓里没有了（取回办法见本节开头那一段） |
| 预算 | 不传 `max_tokens` 与 `thinking_token_budget` |

用它的时候：

- 提示一律用英文，不用 markdown 强调（`**粗体**:`）；不靠 `presence_penalty`、`repetition_penalty` 修中文复读。
- 它给出的具体字段名、结构名、机制描述一律当待核线索；核不动整条丢掉。
- 提示里把最容易上钩的类别错误写出来并禁止：「同一系统在两种介质上」不许答成「两个不同系统」；「宽度在别处定」写明「推迟的是宽度，不是字段本身」。
- 它答「举不出来」时跑阳性对照：同一份提示只把被问对象换成已知答案是「有」的对象，这次举得出来，那个「举不出来」才算数。

```

**出处 `.claude/kb/tooling.md:27-38`（整段抄，未转述）**

```markdown
## 仓里的性能数都只在本机成立，引用要带机器口径

**射程**：挂钟、吞吐、每次发布多少毫秒、门禁跑多久、冷挂载多少秒、并行加速比，凡是时间与速率。

**怎么做**：

- 引一个性能数时把**机器口径**一起带（哪台机、什么盘、什么负载、空不空闲），不带口径的数不许进判据。
- 拿本机的数去定**阈值、上界、超时**时，写明「这是本机值，别的机器上要重量」。
- 需要一个跨机器成立的数时，要么量两台以上，要么写成条件式（「按 X 算是 Y，那一档本机测不到，仍是假设」）。

门禁那一档整环清零的耗时、冷重开恢复的耗时、崩溃点重放两条流的挂钟、E152（按里程碑对比六家文件系统的文件性能） 与六家的全部对比数都是本机数（C486（整环清零的 55 秒与冷重开的 17 秒都没人管））。

```

**出处 `.claude/kb/tooling.md:39-45`（整段抄，未转述）**

```markdown
## 跑项目代码的命令一律经 task-server 交

- 统一任务服务（`task-server/`）是起跑项目代码的任务（编译、测试、门禁、实验、自证；CPU、GPU 都算）的唯一入口：挑机器、内存额度、线程、排队次序都由服务定，提交时不收。服务只调配 CPU 与内存，不分卡、不领卡、不清场：卡归 GPU 批处理验证服务（`gpu-verification-server/`）。用法、判法与拒绝清单在 `task-server/README.md`，不抄第二份。
- 调用方用客户端 `task-server/task` 的七个子命令：`submit [--cwd <目录>] -- <命令…>` 交、打出任务号；`confirm <号> --items <N> --tasks <M>` 核对确认单之后确认（交了的任务先待确认，确认了才进队，不确认的满 30 分钟服务取消），`confirm <号> --why '<…>'` 确认一条跑满两小时的任务确是要跑这么久；`wait <号>` 等到结局（末行 `TASK_SERVER_RESULT …`，退出码是任务的）；`status [<号>]` 看在排、在跑与排队原因；`log <号>` 看最后一次尝试的日志；`cancel <号>` 停；`clean <号>` 删这条任务留下的编译目录（第二台那一趟的编译目录由 `multi-host-run.sh` 跑完自己删，`clean` 不管）；任务自己在跑的那一次尝试里按需领卡、还卡用 `lease take`、`lease return`（只认文件头声明了 `# task-server-card-lease: on-demand` 的入口），调用方不起。命令里不套包装、不写资源参数，写了当场拒（`task-server/README.md`「提交写法」）。
- 子 agent 一律不交任务：`submit`、`wait`、`cancel`、`clean` 只由主 agent 起，子 agent 要跑的写进报告交主 agent（用户 2026-11-01 定，`records/2026-11-17-subagent账本自上而下清欠.md` 第一百七十四节）。
- 出了问题看哪里：结局看 `wait` 末行的 `type=`（错误类型与退出码的表在 `task-server/task_server_lib.py` 文件头「错误类型」）；排着不起看 `task-server/task status <号>` 的「在等：…」；跑挂了看 `task-server/task log <号>`（`--all` 整份）；`wait` 退 78 是连不上服务，先 `task-server/task status <号>` 看任务还在不在，不重交。

```

**出处 `.claude/kb/tooling.md:46-50`（整段抄，未转述）**

```markdown
## 变异工具：并发不超过核数，不限时，卡住由统一任务服务判

- 并发度乘以一条被测单位自己占的核数，不超过核数。cargo-mutants 跑本仓用 `--jobs 1`。
- `scripts/mutation/mutate.sh`、`scripts/mutation/broad-mutation.py`、`scripts/mutation/mutation-prune.py` 不给一条变异或一步设限时，也没有超时这一种结局：整条命令是统一任务服务（`task-server/`）的一个任务，卡住与死循环由服务判、停掉，记 `stalled`、打回提交它的会话（判法在 `task-server/README.md`）。

```

**出处 `.claude/kb/tooling.md:51-61`（整段抄，未转述）**

````markdown
## 变异条数会随源码漂，不许在 kb 里钉死

正文里写数它的命令，不只写数：

```
cargo mutants --list --package singlefs-checker | wc -l
cargo mutants --list --package singlefs-core | wc -l
```

要引一个具体的数，连同量它的日期写成事件句（「<日期> 数出 N 条」），不写成现状句。

````

**出处 `.claude/kb/tooling.md:62-65`（整段抄，未转述）**

```markdown
## 改了引用方，要 grep 全仓改被引用方

改正一处措辞之后，说「已全部改完」之前，先 `grep -rn` 全仓数一遍。范围含 `kb/` 全部子目录、`research/prompts/`（喂给三方论证的背景材料）、`scripts/` 与 `research/` 下的源码注释、`records/`、门禁脚本。数不清就写「改了这几处，没有全仓核过」。

```

**出处 `.claude/kb/tooling.md:66-70`（整段抄，未转述）**

```markdown
## 一个数不许从「它自己判为非法」的配置里算出来

- 算一个上限或下限之前，先确认那个配置在本工程的判定下是合法的。
- 撤回一个数时，作废理由写成真实的那一条（缺输入、配置非法、口径错），不写成它保留下来的别的档同样有的性质。

```

**出处 `.claude/kb/tooling.md:71-74`（整段抄，未转述）**

```markdown
## 未定项一律写成编号条目，标题一律声明条数

按 `^\d+\.` 提取的脚本只认编号条目；`.claude/gate.d/doc-decisions.sh` 的 kb-shape 格的条数检查只在标题声明了「N 项未定」时启动。

```

**出处 `.claude/kb/tooling.md:75-87`（整段抄，未转述）**

```markdown
## 验证产物放哪：四个位置，各管一段

| 位置 | 放什么 | 谁读 | 进仓库 |
|---|---|---|---|
| `.claude/kb/experiments.md` | 结论、口径、判据、复现命令、**判决性的那几行数字** | 模型检索 | 进 |
| `records/` | 每一轮的叙述：定了什么、推翻了什么、依据 | 人通读 | 进 |
| `scripts/` | 实验代码与脚本 | 跑的人 | 进（`.gitignore` 只排除 `target/` 与个别超大文件） |
| `research/results/<实验>-<日期>.out` | 原始输出全文 | 下一轮复跑时逐行比对 | 进；早于最近一轮的在下一次提交时归档进 git 历史（门禁 doc-experiments 的 archive-past-rounds 格） |

- 判决性的数字贴进 kb 正文。
- 每份原始输出要能自证是产出它的那一次运行留下的：收尾行 `E7RESULT name=done emitted=N` 里的 N 等于文件里 `E7RESULT` 的行数，对不上整轮作废。归档时核一遍：
  `for f in research/results/*.out; do echo "$(grep -c '^E7RESULT' $f) vs $(grep 'name=done' $f)"; done`

```

**出处 `.claude/kb/tooling.md:88-100`（整段抄，未转述）**

```markdown
## 内核源码：本机有完整树，不必 WebFetch

| 项 | 值 |
|---|---|
| 路径 | `~/kbuild/linux-om`（1.8 GB） |
| 版本 | **Linux 7.2.0**（`Makefile` 的 `VERSION = 7` / `PATCHLEVEL = 2` / `SUBLEVEL = 0`） |
| 有什么 | 完整 `.c`/`.h` 正文，含 `fs/jbd2/`、`fs/xfs/`、`fs/btrfs/`、`fs/erofs/`、`fs/ext4/`、`fs/f2fs/`、`drivers/nvme/`、`drivers/md/`、`crypto/`、`Documentation/` |
| **没有什么** | **`fs/bcachefs/` 不在这棵树里**（见 [decisions.md](decisions.md) D7（是否进 Linux 主线））。`/usr/src/linux-headers-6.17.0-1028-oem/fs/bcachefs/` 只有 `Kconfig` + `Makefile`，**没有源码** |

- 查源码用 `~/kbuild/linux-om`，不用 `/usr/src/linux-*-headers-*`（那里只有 `Kconfig` 与 `Makefile`）。
- 引哪棵树就标哪个版本号，两个版本的结论不混着引。kb 里标「现查 Linux 6.17」的引用来自 GitHub 的 `v6.17` tag，本机没有那个版本。
- OpenZFS 本机没有树，现取 `https://raw.githubusercontent.com/openzfs/zfs/master/...`，引用时记下取回日期，行号不是永久的。

```

**出处 `.claude/kb/tooling.md:101-106`（整段抄，未转述）**

```markdown
## 到点自己开工：用 remind.sh 登记

用户要会话到点自己开工时，只用 `bash scripts/agents/remind.sh start --every <分钟> --message <提醒内容> --once`（Bash 的 `run_in_background` 起，到点退出叫醒会话；规则 `.claude/rules/timer-only-remind.md`，写法 `.claude/skills/remind/SKILL.md`）；不用 CronCreate，也不在 Bash 里只为定时而写 `sleep`。提醒内容写开工指令所在的路径，第一步用 `set -o noclobber` 排他建「已开工」标记，标记已在就什么都不做（`remind.sh` 重起时可能重复提醒）。

- 提醒只活在当前会话里，告诉用户会话不能关、机器不能休眠。

```

**出处 `.claude/kb/tooling.md:107-117`（整段抄，未转述）**

```markdown
## Rust 工具链

| 项 | 值 |
|---|---|
| 版本 | rustc 1.98.0 / cargo 1.98.0 |
| 安装方式 | rustup，`--profile minimal --default-toolchain stable`，装在宿主 `~/.cargo` |
| 额外目标 | `x86_64-unknown-linux-musl`，实验二进制（`scripts/experiment-apparatus/` 下的包）编成静态的在虚机的 busybox initramfs 里跑；验证包的真设备二进制 `new_pool_file_creation_on_device` 不走它（验证包默认编 RocksDB 的 C++，本机没有 musl 的 C/C++ 编译器），改用 `--target x86_64-unknown-linux-gnu` 加 `+crt-static` 编成 static-pie，命令在 `.claude/gate.d/qemu-device-streams.sh` |
| 组件 | `rustfmt` + `clippy`（`--profile minimal` 不含它们，要另装） |
| **PATH** | rustup 的 shim 软链在 PATH 上的用户级可执行文件目录里（cargo / rustc / rustup / rustdoc / rustfmt / cargo-fmt / cargo-clippy）；门禁与脚本用 `bash -c` 起，不读 shell 的启动文件，PATH 不靠改 dotfile |
| release 溢出检查 | 树根 `Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml` 的 `[profile.release]` 都设了 `overflow-checks = true`；profile 只认工作区根，写在成员 manifest 里不生效 |

```

**出处 `.claude/kb/tooling.md:118-145`（整段抄，未转述）**

```markdown
## RocksDB 与 snappy 的预编库：每台主机编一次，取值只走进程环境

librocksdb-sys 的 C++（`librocksdb.a` 与 `libsnappy.a`）每台主机只编一次，各编译目录经环境变量链这一份，不在每个编译目录里现编。工具是 `scripts/hosts/rocksdb-prebuilt.py`，用法写在它的文件头（它不认 `--help`，退出码 2）。子命令都在仓根跑，`--root R` 不给就是这份脚本所在的仓。

| 子命令 | 做什么 |
|---|---|
| `key` | 打这台主机此刻的键（各组成）与预编库目录，不编 |
| `env` | 预编库在、对得上时打四行 `export`（给 `eval` 用）；不在退 1、标准输出为空 |
| `build` | 预编库不在就编一次，持锁：几个进程同时来只编一次，后来的等锁之后看到编好了直接退 0；起真 cargo，是重活，经 `task-server/task submit -- python3 scripts/hosts/rocksdb-prebuilt.py build` 交（`task-server/README.md`） |
| `install` | `build`，再写这台主机的三处环境来源、改用户级 systemd 管理器此刻的环境，最后跑 `check`；同样是重活 |
| `check` | 判这台主机：两个工作区（`crates/singlefs-verification/Cargo.toml` 与 `scripts/experiment-apparatus/Cargo.toml`）的依赖对得上、预编库在且对得上、三处环境来源与此刻进程环境的取值都与预编库一致、cargo 配置里没写这四个变量 |
| `audit-target <编译目录>…` | 判编译目录：librocksdb-sys 构建脚本那一次拿到的取值是不是这台的预编库、`out/` 里有没有现编过的 C++、同一把键下见过几种取值；一份构建脚本记录都没找到退 77（本次未判） |
| `entry [--cargo C]` | 起 cargo 的入口开头调的轻判（谁调、取值缺了怎么办写在「起 cargo 的入口」那一段） |

**预编库放哪**：`${SINGLEFS_ROCKSDB_PREBUILT_STORE:-~/.local/share/singlefs/rocksdb-prebuilt}/librocksdb-sys-<版本>/<键的前 16 位>/lib/{librocksdb.a,libsnappy.a}`；同目录 `manifest.json` 记键的各组成与两份库的 sha256，`complete` 最后写（有它才算编好）；不放开机清空的目录。`SINGLEFS_ROCKSDB_PREBUILT_STORE` 只给自证换临时目录用。键由两个工作区的 rocksdb 依赖写法、两份 `Cargo.lock` 里相关包的版本与 checksum、`rustc -vV`、c++ / cc / ar 的真实路径与版本、交给 C++ 的指令集、编法组成，哪一样变了键就变、要重编。

**取值从哪来**：四个变量 `ROCKSDB_LIB_DIR`、`ROCKSDB_STATIC`、`SNAPPY_LIB_DIR`、`SNAPPY_STATIC` 的取值写在三处环境来源里，都由 `install` 写：

| 位置 | 作用 |
|---|---|
| `~/.config/singlefs/rocksdb-prebuilt.env` | 唯一的取值来源：四行 `export`，外加 `unset ROCKSDB_COMPILE`、`unset SNAPPY_COMPILE`；别手改，改了重跑 `install` |
| `~/.bashrc` 最前面一段（三行，`# >>> singlefs rocksdb-prebuilt >>>` 到 `# <<< singlefs rocksdb-prebuilt <<<`） | `source` 那份 env 文件；放在交互判断之前，所以交互 shell、从它起的会话与 ssh 起的非交互命令都读得到 |
| `~/.config/environment.d/60-singlefs-rocksdb-prebuilt.conf` | 用户级 systemd 管理器起的单元读（task-server 为每一次尝试起的单元，`task-server/README.md`）；`install` 另跑 `systemctl --user set-environment`，让此刻在跑的管理器当场换上 |

**四个变量只走进程环境**：不写进 `.cargo/config.toml` 或别的哪一层 cargo 配置的 `[env]`（写了 `check` 判红；理由在 `.cargo/config.toml` 开头的注释）。`ROCKSDB_COMPILE`、`SNAPPY_COMPILE` 一律不设，设成 1 就在编译目录里现编。

**起 cargo 的入口**：门禁阶段、崩溃与变异驱动、`merge-and-verify.sh`、`admission.py` 交的命令、第二台那一侧，开头都调 `entry`。它只判三样：这台装了（预编库在、对得上，不算两份库的 sha256）、此刻进程环境的四个取值与预编库一致、没设 `ROCKSDB_COMPILE` 与 `SNAPPY_COMPILE`；不判三处环境来源（那是 `check` 的事）。取值缺了或对不上，入口当场拒、一个 cargo 都不起，逐项写出出路。出路：此刻进程没有取值的，开新会话（`install` 之后起的登录 shell 读 `~/.bashrc` 最前面那一段），或在命令前 `eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"`；这台还没装的先交 `task-server/task submit -- python3 scripts/hosts/rocksdb-prebuilt.py install`（`task-server/README.md`）。要编的那棵树里没有依赖 rocksdb 的工作区时（编不到 librocksdb-sys，门禁样本与自证的小仓多是这样），`entry` 不判、退 0。

```

**出处 `.claude/kb/tooling.md:146-160`（整段抄，未转述）**

```markdown
## 虚机 benchmark harness（E7（离线索引 harness） 用）

| 项 | 值 |
|---|---|
| 脚本 | `bash scripts/devices/vm-bench.sh <静态二进制>` / `--selftest` |
| 它是什么 | 项目自己的虚机装置：给虚机挂 virtio 盘（稀疏文件落 `TMPDIR`）并把二进制塞进 initramfs |
| 完整性闸 | 被测程序收尾行报 `name=done emitted=N`，宿主抓到的条数必须等于 N，否则整轮作废 |
| 默认规格 | 2048M 内存 / 4 vCPU / 4096M 盘 / 900s 超时，均可用环境变量覆盖 |

- 抓结果行不锚定行首（控制台首行前面有 BIOS 的转义序列），条数靠完整性闸核。
- 虚机里裸设备的顺序写吞吐不稳定，不作为任何判断的输入。
- 跑批脚本作废一轮时，把该轮输出的尾部一并记下。

---

```

**出处 `.claude/kb/tooling.md:161-197`（整段抄，未转述）**

````markdown
## 造几何量不同的设备：不需要真硬件

本机具备造异构设备池的全部手段，[decisions.md](decisions.md) D12（目标介质） 已定项 3「设备级几何量该放哪、异构池支不支持」可以实测。

### QEMU 路线（推荐：无需 root，直接接现有 harness）

`zoned.zone_size` 与 `logical_block_size` 挂在 **`nvme-ns`（命名空间）** 上，不是挂在控制器上，同一个控制器下的多个命名空间可以有不同的几何量。

已实测通过（QEMU 8.2.2，退出码 0，三个设备全部构造成功）：

```
-device nvme,serial=A,id=nv0
-device nvme-ns,drive=d1,bus=nv0,nsid=1,zoned=true,zoned.zone_size=64M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d2,bus=nv0,nsid=2,zoned=true,zoned.zone_size=128M,logical_block_size=4096,physical_block_size=4096
-device nvme-ns,drive=d3,bus=nv0,nsid=3,logical_block_size=512,physical_block_size=512
```

一条命令同时造出：**异构 zoned 池**（64M vs 128M zone）、**512e/4Kn 混用**、**zoned 与常规混用**。

- `logical_block_size` 必须配 `physical_block_size`，只给前者会被拒：`logical_block_size > physical_block_size not supported`。
- 来宾内核看不看得到三份不同的几何没有验过：要进来宾读 `/sys/block/*/queue/{logical_block_size,chunk_sectors}` 与 `blkzone report`。

### 宿主侧路线（要 root，胜在不用启虚机）

| 手段 | 造什么 | 关键参数 |
|---|---|---|
| `null_blk` | zoned 设备，内存后端 | `zoned=1 zone_size=<MB> zone_capacity zone_nr_conv zone_max_open`。模块参数是全局的，**每设备各自的几何要走 configfs**（`/sys/kernel/config` 本机已挂载） |
| `zloop` | **文件后端**的 zoned 设备 | 模块在 `drivers/block/zloop.ko.zst`，本机有 |
| `scsi_debug` | 任意扇区大小 + ZBC | `sector_size=<bytes> zbc=host-managed zone_cap_mb zone_nr_conv` |

运行内核 6.17.0-1028-oem 的相关配置：`CONFIG_BLK_DEV_ZONED=y`、
`CONFIG_BLK_DEV_NULL_BLK=m`、`CONFIG_BLK_DEV_ZONED_LOOP=m`、`CONFIG_SCSI_DEBUG=m`。

⚠️ 本机唯一的真实块设备是 `nvme0n1`（3.6T，512/512，非旋转）——
**没有真的旋转盘、没有真的 4Kn 盘、没有真的 zoned 盘**，所有相关数字都只能是模拟出来的，
引用时必须带这个口径。

````

**出处 `.claude/kb/tooling.md:198-203`（整段抄，未转述）**

```markdown
## `scripts/mutation/mutate.sh` 的结局分类

- 结局：`✅ [抓到]`（红了）/ `❌ [没红]`（测试盲区）/ `⏭ 变异导致编译失败（无效变异，不计入盲区也不算命中）`；另有 `💥`（测试进程没跑完）、`⚠️`（测试进程报了失败，却一个失败的测试名都没抓到：抓名字的规则有盲区）与锚点失效（`⚓ [锚点腐化]`，原文命中不是恰好一次的那一条不跑、不入判定库，别的条照跑，整轮退 3）几类，各自单列；收尾「计数：」一行报抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏（全部结局见 `scripts/mutation/mutate.sh` 文件头与 `--selftest`）。
- 上限与限时：每次 cargo test（基线、每条变异、还原之后那一次）直接起，不设内存上限、不限时；整条 `mutate.sh` 交统一任务服务（`task-server/`），一整条一个任务：内存上限由服务按记录给、撞了放大重排；死循环的变异由服务停掉整条任务（用时远超记录而日志又满 10 分钟不涨就停掉、打回）（`scripts/mutation/mutate.sh` 文件头「上限、限时与卡住」）。
- 改了判定工具，拿已知答案的整套语料回归一遍，不只跑新加的那几条。

```

**出处 `.claude/kb/tooling.md:204-217`（整段抄，未转述）**

```markdown
## 几个会话共用一个仓时怎么暂存与提交

门禁红了先分清红的是谁的改动，再动手；不为了让门禁变绿去改别人在飞的文件。规矩在上游 `session-wrapup.md` 第 4 节，本仓的做法：

| 事 | 怎么做 |
|---|---|
| 只暂存自己的块 | `scripts/gate/stage-mine.py --match 我的标记 --foreign 别人的标记 文件…`，先 `--dry-run` 看它挑了哪几块 |
| 暂存区也是共用的 | 暂存前先确认 `git diff --cached --name-only` 为空；暂存完、提交前再核一遍名单与逐文件 cached diff，然后不带路径地 `git commit` |
| 不要 `git commit -- <路径>` | 带路径时 git 提交的是这些路径在工作区里的内容，不是暂存区 |
| 同一行两边都改了 | stage-mine 报「进暂存区的一块里有命中 --foreign 的行」时一行切不开：对方在那份文件里只做了可逆替换（改名）的，拿工作区版本把对方的替换反做一遍、与 `git show HEAD:文件` 比 diff 确认只剩自己的，再 `git hash-object -w` + `git update-index --cacheinfo 100644,<sha>,<路径>` 放进暂存区，工作区不动 |
| 在 worktree 上提交 | 在 worktree（HEAD 上）用脚本重放这次要提交的全部改动 → 与主工作区逐文件比对（不同的部分应当全是对方的；比对时 `git -c core.quotePath=false`，否则中文路径被转义、`cmp` 全报不同）→ 在 worktree 跑整道门禁并提交 → 主仓里 `git update-ref refs/heads/master <新提交> <旧 HEAD>` 再 `git read-tree <新提交>`，工作区不动、对方的改动原样留着 |
| 看清哪些不是这次要提交的 | `git -c core.quotepath=false status --short`，中文路径默认被转义成八进制，grep 中文关键词会漏 |
| 两边改动交织在同一文件 | 一个提交带全部，消息里写明（拆开提交会让门禁 doc-registries 的 experiment-refs 格与门禁 doc-experiments 的 experiment-orphans 格判红） |

```

**出处 `.claude/kb/tooling.md:218-223`（整段抄，未转述）**

```markdown
## 复跑与变异脚本要在 `scripts/experiment-apparatus/` 下跑

- `scripts/mutation/mutate.sh` 在 `scripts/experiment-apparatus/` 下跑：`cd scripts/experiment-apparatus && bash ../mutation/mutate.sh <bin 名> <源文件> ../mutation/tables/<变异表>`。
- `replay.sh` 只认带 E 的实验号（`E` 加数字），裸数字退出码 2。
- `replay.sh` 每一趟编进它在 `${TMPDIR:-/tmp}/singlefs-replay-build-<进程号>-XXXXXX/target/` 自己建的编译目录、跑完删（调用方设了 `CARGO_TARGET_DIR` 的照它编、不删），研究二进制从 `$REPLAY_RELEASE_DIRECTORY`（`$CARGO_TARGET_DIR/release/`）起；`scripts/experiment-apparatus/target/` 不再有复跑编出来的二进制。

```

**出处 `.claude/kb/tooling.md:224-243`（整段抄，未转述）**

```markdown
## 批量立 / 定分项时门禁会红的形态

一次立或定一批分项之后，提交前门禁里这几格会按下列形态红，写的时候照「写法」一列写：

| 门禁 | 会红的形态 | 写法 |
|---|---|---|
| doc-registries 的 first-txn-hooks 格（三份文件挂钩） | 按 `\| N \|` 行切未定项条目，最后一行把后面所有 `#### 未定项 N` 正文吞进自己那一条；正文里写「改新池新建文件的字节：是（」，判「否」的最后一行也被当成「是」 | 索引行用规范形态 `改新池新建文件的字节：是（日期，依据：…）`，正文小节写成 `**改新池新建文件的字节**：是（…）` |
| doc-decisions 的 item-ref-status 格（引用归属） | 行内裸的「已定项 k」归属给同一行最近的 `D<n>（简称）` | 新立分项之后跑 doc-decisions 的 item-ref-status 格，把翻掉的裸引用补上决策名；决策正文里引别的决策的分项，同一行里要重复写自己的决策名 |
| doc-lint | `F` 加数字这类审核编号被当成没有登记位的编号；简称后面加逗号注判「名字不符」 | 写「发现 1」「提议 2」；括注另起一对括号，写成「编号（简称）（条款已写）」，不塞进简称的括号里 |
| doc-registries 的 table-shape 格（欠账表形状） | 新的六列欠账行接在文件末尾，而那里是「已还清」那张 4 列表；欠着那张里一格开头写「已还（…）」或「销账」；欠着那张中间夹一个空行 | 接在第一张表最后一行后面：第一个五竖线的 `\|---\|` 之前、最后一行以 `\| C` 开头且竖线 ≥ 7 的行；还清了就按四列挪进已还清表，只还一部分的开头写「已还一半」「决策已定、检查仍欠」「前置已还」；表中间不留空行 |
| doc-registries 的 segment-registry 格（段序列登记表） | `--selftest` 要「mkfs 种根」那一行的段序列串在整份 `layout/01-first-txn.md` 里只出现一次，同一串在别处再写一遍就红 | 别处引用写成「4 + 1 + 1 + 1 + 4」带空格的写法 |
| 门禁 doc-decisions 的变更史三格（entry-added、shape、history-loss） | 「（其N）」、快查两行与节顶现状行 | 写法与判法在 `.claude/rules/changelog-format.md`；数与编号有没有丢由 history-loss 格判，整理变更史时暂存前自己先跑 `scripts/kb/history-consolidate.py loss-check` |
| code-source-discipline 的 format-constants 格（格式常量） | kb 标记改值后，`scripts/experiment-apparatus/**/*.rs` 与 `crates/**/*.rs` 里每一份同名 `const` 都要跟（`research/` 下只剩冻结证据，扫它但由 `.claude/format-constants-exclude` 逐份放行）；`stale=` 列的旧值串不许再出现在 kb 正文 | 别把仍在别处合法出现的串（`base = 95`）加进 `stale=` |
| clippy `shadow_unrelated` | 同一个测试函数里第二次 `let reader = …` / `let segments = …` 判红，rustfmt 重排之后用正则替换还会漏改后半段 | 每个读者 / 段 / 报告起不同的名字（`extent_reader`、`warm_up_segments`、`control_report`） |
| naming-lint | 「字母 + 数字」当单字母：`read_u48` 判红；结尾的 `a` 与只有一段的 `a` 判红（`side_a`），打头与中间的 `a` 后面还有别的段时当英语冠词、不判（`a_mount_that_stays_short`）；它扫全仓的 .rs，连 `research/prompts` 里云端腿写的模型源码也扫 | 写 `read_six_byte_unsigned`；`research/prompts` 在 `.claude/naming-lint-exclude`（证据目录，不回收），腿的模型写到别的目录照查 |
| crash-point-replay（崩溃点重放） | 崩溃枚举用例住 `crates/singlefs-verification/tests/`、登记成 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行、不标 `#[ignore]`（D13（验证路线） 已定项 15） | 交门禁 crash-point-replay：按条那一趟经 `--skip` 排掉登记的用例，登记的用例逐条按崩溃状态块的单元键判（块键相同的复用、缺的当场判；登记行第三列带 `separate-run=` 的用例没被点名时不跑、报本次未跑）；在跑时换 `CARGO_TARGET_DIR` 就能同时编别的测试 |
| 变异证明 | rustfmt 之后多行调用的整串匹配「没命中」 | 按「找到含关键串的行、删到以 `);` 收尾的那一行」做；改完复原并 `assert` 文件逐字节等于原文 |
| checker 新判定 | 只在对象已经过了同一关之后才判的判定永远判不红 | 每条新判定配一份坏镜像（`checker_known_bad_images.rs` 的形态），不许只看干净镜像全绿 |
| 批量定点替换 | 逐条写盘会在一处锚点不中时留下半套改动 | 用 `scripts/kb/replace-batch.py`：全部替换先在内存里核「恰好命中 1 次」，全过了才写盘 |

```

**出处 `.claude/kb/tooling.md:244-254`（整段抄，未转述）**

```markdown
## 本机 Claude Code 会话里的 `grep` 与 `find` 不是 GNU 那两个

`type grep`、`type find` 显示两个都是 shell 函数：`grep` 底下是 ugrep，`find` 底下是 bfs（报错分别以 `ugrep: error`、`bfs: error` 开头）。两处行为与 GNU 不同：

| 写法 | 结果 | 改成 |
|---|---|---|
| `grep -o` 带 `.\{0,90\}` 这种有界重复，作用在中文行上 | `exceeds complexity limits`，退出码 2，一行都不输出 | 用 python 按下标切：`line[max(0, i - 90):i + 90]` |
| `find 目录 -newermt '-40 minutes'` | `Invalid timestamp`，不认相对时间 | 写 ISO 日期：`-newermt '<YYYY-MM-DD>'` |

两种失败的样子都是「这条命令没给出结果」，接在管道或 `||` 后面时不要读成「没有命中」。

```
