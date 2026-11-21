# 看门狗「知识同步提醒」治理文件轮第二轮：攻方腿报告（watchdog-knowledge-sync-governance-r2，2026-11-18）

攻击面：背景材料「要腿判的」第 1 到 7 问。被判文件读主工作区此刻的那一版：`scripts/agents/agent-watch.py` 的 sha256 是 `5b6030270788db16981d3e0813584cf7aef278200eeda03ca5ae9fc4ab86555c`，与开工快照 `research/prompts/watchdog-knowledge-sync-governance-r2-snapshot/judged-sha256.txt` 那一行相同（`sha256sum` 现查）。第一轮攻过、没打中的角度（改名两向、起来时改着之后提交又改、stash、换分支、新提交删文件、只改 mtime、行尾、索引锁、三级取法本身）不重复。

**这份报告里的读数全是「推的」**：探针写好了，没跑（派发提示写明腿不自己跑）；凡写「探针预期」的，都等主 agent 复跑之后以产物为准。判定里写「读码」的是照代码行推的。

## 复跑命令与模型文件

模型目录 `research/prompts/watchdog-knowledge-sync-governance-r2-opus-model/`：

| 文件 | sha256 |
|---|---|
| `probe/knowledge_sync_r2_probe.py` | `e4da7c22075467f418203dad1e0f354a2adbfeff7d723c6a313fb32e8f0f646a` |
| `tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`；`rerun_steps` 之外与模板逐字相同，`diff` 现核） | `2e4df9f4672a28a05ea3e6b29437b598c3d1c6dc11e7114593d14f59836f0dcb` |
| `tools/rerun-from-a-clean-work-directory.sh`（先删那一个运行目录，再调 `rerun.sh`；只删工作根底下直接一层、名字以 `-run` 收尾、不带 `.`/`..` 段的，别的退 2） | `3843564d2e51165cfef94f984d0c67e79a7868f62055db8119edbedbe6d28edc` |

复跑命令（「要主 agent 跑的命令」一节第一条，原样）：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-opus-run /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r2-opus-model
```

- `--frozen` 填的是主工作区本身：探针只导入其中的 `scripts/agents/agent-watch.py`（带 `-B` 与 `PYTHONDONTWRITEBYTECODE=1`，不落字节码），不对真仓跑 git；第一行 `PROBE name=watchdog … sha256=…` 打出被导入那份的 sha256，对不上 `5b603027…` 就是被判对象在开工之后变了，这一趟不算。主 agent 有冻结树的，换成它。
- 探针做什么：在 `<运行目录>/repositories` 底下逐格建小仓与临时会话目录；HOME、`AGENT_WATCH_PROJECTS_ROOT`、`AGENT_WATCH_STATE_ROOT` 都换到那里，去掉 `TASK_SERVER_SOURCE_REPOSITORY`，git 带 `GIT_CEILING_DIRECTORIES=<运行目录>`、不读用户与系统配置（配置那一格给自造的）。导入被判的 `agent-watch.py`，直接调 `take_repository_baseline`、`knowledge_sync_section`、`repository_status_section`、`repository_snapshot`、`knowledge_state_path`；模块里的 `time` 换成假时钟（只换 `time.time`，算摘要时限用的 `time.monotonic` 照真的走），一串起来与退出的时刻由探针喂。起的子进程（`--child`、照抄跑的 `bash`）都 `start_new_session=True`，到点 `os.killpg` 只停那一组。
- 完整性闸：末行 `PROBE name=done emitted=<N> setup_failures=<K>`，`rerun.sh` 核 N 等于前面 `PROBE` 行数，对不上退 3。
- 探针的时间与内存（推的，没量）：最重的三格是 5 万份被忽略文件（建文件约 10 秒、算摘要几秒）、8 GiB 稀疏文件（本机 ext4 上 sha256 几秒；工作根 `findmnt` 现查是 `ext4`）、弄坏开关下命名管道等满 20 秒；整趟估几分钟、远不到 40 分钟，常驻内存估几十 MB。没有崩溃状态，第 3b 步的崩溃枚举与状态数不适用。

## 各格判定一览

| 问 | 判定 | 打中的（各节逐条给能照喂的输入与 文件:行） |
|---|---|---|
| 1 落盘与按时带 | 推不出（打中） | 提醒间隔最长约两倍 `--max-minutes`；起来时就查不成时不落盘、上次提醒的时刻每只重置，告警比 `--max-minutes` 密时这一节永远不出；两只读同一份时都打、后一只写的「离上次提醒」是错的；落盘在打印之前；`reminded_at` 被改成 NaN 或将来的时刻就再也不提醒；`--agents` 起、子 agent 还没会话记录时不落盘（文件头没写这一种） |
| 2 被忽略的条目 | 推不出（打中） | 「没查成」下面给的两条命令看不见被忽略的条目，而超时最可能就出在它们身上；只改忽略规则、内容没变也标「内容摘要变了」；时限到了：起来时到的同第 1 问那一条，之后到的每次退出都带、基线不换 |
| 3 porcelain 加 -z | 推不出（打中） | 同一路径出两条（`git rm --cached` 之后「D 」与「??」）合成一个键，已暂存少数一个、「D」那一码丢了；带换行的路径键对，但输出里一份点名拆成两行，能造出第二行「→ 同步：」；非 UTF-8 文件名被 `errors="replace"` 换掉，摘要恒「不在」，改了不点名；R、C 两段与 `git diff --name-only -z` 读得对；三形配置不改读数（自证有格），另几种配置交探针 |
| 4 摘要没变不点名 | 主体成立；边角推不出（打中，低） | 索引里改了、工作区原样不点名，判它不该点名（理由见那一节）；改过又改回去不点名，成立；状态码那一支这一轮没改判法；边角：未跟踪目录里指到目录的符号链接改指向、读不出的文件改了内容，都不点名 |
| 5 给人抄的命令 | 推不出（打中，低） | 照抄能跑这一格只在不带特殊字符的仓路径上跑（`knowledge-reading`），把 `shlex.quote` 换成别的引法自证照绿；`repositorymanualcommandssemicolon` 只红点名的那一格，成立（登记表那一行记的任务 20379 读数）；`who-touched.py` 那一行过得了主 agent 的闸，成立（读码） |
| 6 文档与开关表 | 推不出（打中） | `--processes-only` 告警退 3 没有「→ 接着盯」那一行，而「→ 同步：」与技能段都叫照它；文件头「谁先满谁打」与两只都打不符；文件头「重新记一份…落盘」在查不成时不成立，且 git 起不来被写成「HEAD 找不到了」；文件头漏了 `--agents` 取不到会话目录那一种；开关表这两批的每一条与代码对得上，成立 |
| 7 第一轮按线索记的两处 | 成立（命名管道读码与登记的复跑读数；时序读码），剩下的固定时限只会误红、推的 | 命名管道格的形状真能在弄坏开关下卡死（探针另跑一遍）；r1 的时序窗口没了，剩 60 秒告警余量、60 秒命名管道、120 秒收集、60 秒照抄四处固定时限 |

## 第 1 问：落盘与按时带这一节

### 打中：提醒间隔最长约两倍 `--max-minutes`，「满一小时就出」不兑现

用户定案的原话是「记到文件、满一小时就出」（`research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md:73`）。代码只在退出那一刻判满没满：「due = now - baseline.reminded_at >= arguments.max_minutes * 60」（`scripts/agents/agent-watch.py:2048`）；而一只看门狗什么时候定时退出，只看它自己起来的时刻：「report_deadline = watch_started + arguments.max_minutes * 60」（`scripts/agents/agent-watch.py:2988`，`--processes-only` 那一路同一句在 `scripts/agents/agent-watch.py:2490`）。在跑的看门狗到了「上次提醒 + 60 分钟」不退，要等它自己的 60 分钟满、或下一次告警。

能照喂的一串退出时刻（`max-minutes` 60，watch.conf 没改；分钟）：

| 步 | 时刻 | 事 | 带不带这一节 |
|---|---|---|---|
| 1 | 0 | 一只定时回报退出、带了这一节、落盘 `reminded_at=0` | 带 |
| 2 | 0.1 | 主 agent 起下一只（读到 `reminded_at=0`） | — |
| 3 | 59.9 | 告警退 3：59.9 < 60 | 不带 |
| 4 | 60.0 | 主 agent 起下一只（此刻已满，但起来时不判） | — |
| 5 | 120.0 | 这一只一直没告警，定时回报退 4 | 带：离上次 120.0 分钟 |

推一般式：提醒只在某次退出 e 满足 e − R ≥ M 时出；前一次退出 e₀ < R + M（没满），下一只起在 e₀ + d、最晚退在 e₀ + d + M，于是间隔上界是 2M + d（d 是主 agent 重起的间隔），e₀ 越贴近 R + M 越接近。探针 `exit-chain` 把告警时刻表（无告警、59.9 一次、每 59.9、每 30、每 10、1 与 61）与重起间隔（0.1、1、5 分钟）都扫了一遍（第 3 步「由用户决定的动作」放开扫，只固定 `max-minutes` 与「第一只起来时没有落盘文件」），每格打 `max_gap_minutes`，末尾 `exit-chain-worst arm=as-is`；探针预期 as-is 一臂最坏约 120、零轮改法一臂（`exit-when-due`，下面「改法」）不超过 60 + d。都是推的，待主 agent 跑。

### 打中：起来时就查不成时不落盘，上次提醒的时刻每只重置；告警比 `--max-minutes` 密时这一节一次都不出

起来时查不成（不是仓、git 起不来、某条 git 超过 30 秒、算摘要超过 30 秒），`take_repository_baseline` 直接返回、不落盘：「return RepositoryBaseline(root, None, str(error), state_path, started_at, note)」（`scripts/agents/agent-watch.py:1849`）——上次提醒的时刻取的是这一只起来的时刻 `started_at`。读到了落盘那一份、但核起点 HEAD 那一条 git 出错时也走到这一句：先写 note「note = f"落盘的基线里起点 HEAD {snapshot.head_short_hash} 在仓里找不到了（{error}），这一只起来时重新记了"」（`scripts/agents/agent-watch.py:1842`），再去取快照、取不成就同样用 `started_at`，落盘那一份记的 `reminded_at` 被这一只丢掉。

能照喂的输入：会话目录里没有落盘文件；`git` 一直起不来（自证「git 起不来」那一格的法子：`AGENT_WATCH_GIT_EXECUTABLE` 指到不存在的路径）；告警每 30 分钟一次、主 agent 每次 0.1 分钟后重起。第 0 只起在 0、30 告警退出（离它自己起来 30 分钟，没满）；第 1 只起在 30.1、记成 30.1，60 告警退出（29.9，没满）……每一只都从自己起来时重新计，180 分钟里这一节一次都不出，主 agent 也一次都不知道「没查成」。探针 `unreadable-from-start`（告警每 30、每 59.9、每 10 三种）预期 `sections_printed=0`；`unreadable-later`（先查成过一次、落了盘，之后 git 起不来）走的是上面 `:1842` 那一句，预期同样是 0。零轮改法 `exit-when-due` 碰不到这一格（`reminded_at` 每只都被重置），探针同一行的 `sections_printed_with_exit_when_due` 预期也是 0。

另一种查不成（算摘要超时，git 本身好的）结局相反：起来时读得到落盘的基线（不用算摘要），到点算此刻的快照超时、写「没查成」、不换基线（`scripts/agents/agent-watch.py:2056` 那一句的 `now_snapshot is not None` 不成立），下一只读回同一份旧的 `reminded_at`，于是满了之后**每一次**退出都带「没查成」这一节。探针 `digest-timeout-chain kind=later` 预期每 10 分钟一次告警的 180 分钟里打 18 次左右；`kind=from-start` 预期 0 次（同上一段）。

### 打中：两只看门狗读同一份时都打，后一只写的「离上次提醒多少分钟」是错的

文件头写：「盯同一个会话时读写同一份：谁先满谁打、打了就换。」（`scripts/agents/agent-watch.py:132`）。代码在起来时读一次落盘的基线，之后只用内存里那一份（`scripts/agents/agent-watch.py:2992` 起的 `repository_baseline`），退出时不重读文件。`--discover` 与 `--processes` 两只同时在跑（`--discover` 一个会话只一只，`--processes` 另算）：

能照喂的输入：落盘那一份 `reminded_at` 在 −120 分钟；`--discover` 那只起在 −1、`--processes` 那只起在 −0.5；前者在 0 告警退出、带这一节、把落盘的换成 0；后者在 0.5 定时回报退出——它内存里还是 −120，照样带这一节，提醒句写「到此刻这 120.5 分钟里」，实际离上一次提醒 0.5 分钟。探针 `two-watchers` 预期 `first_printed=True second_printed=True second_claims_minutes_since_last_reminder=120.5`。没丢东西（后一只比的是更早的基线、点名只多不少），是重复与那一句写错；文件头的「谁先满谁打」读成「只打一次」就对不上。

### 打中：落盘在打印之前

「problem = save_knowledge_state(baseline.state_path, baseline.root, now_snapshot, now)」（`scripts/agents/agent-watch.py:2057`）在 `knowledge_sync_section` 里，返回之后调用方才「print_lines(knowledge_sync_section(repository_baseline, arguments, on_alert=True))」（`scripts/agents/agent-watch.py:3027`）。这一只在两句之间被停（主 agent 停看门狗、标准输出写失败），落盘那一份已是「此刻、刚提醒过」，这一节没送到；下一只接着它比，这一节点名过的文件再也不点。探针 `replaced-before-print` 预期：没打出去的那几行点名了 `.claude/kb/k.md`，落盘的 `reminded_at` 已是 60，下一只到 120.5 点名 `[]`。窗口很窄（毫秒级），推的。

在 `mkstemp` 之后、`os.replace` 之前被 SIGKILL：「descriptor, temporary = tempfile.mkstemp(prefix=f".{KNOWLEDGE_STATE_FILE_NAME}.", suffix=".tmp", dir=os.path.dirname(state_path))」（`scripts/agents/agent-watch.py:1776`）建的临时文件留在会话目录，没有谁清（`except OSError` 那一支只管 OSError）。落盘那一份不受影响（`os.replace` 原子）。探针 `killed-inside-replace` 用 `os.replace` 抛 `SystemExit` 顶替 SIGKILL，预期 `leftover_temporary_files=1`、下一只照常、`leftover_after_next_watchdog=1`。只是会话目录里多一个小文件，不算缺口，记在这里。

### 打中：`reminded_at` 被改成 NaN 或将来的时刻就再也不提醒

「reminded_at, recorded_at = float(record["reminded_at"]), float(record["recorded_at"])」（`scripts/agents/agent-watch.py:1812`）不核有限、不核不晚于此刻；`json.load` 认 `NaN`、`Infinity`。`now - nan >= …` 恒假，这一节不再带，带了才换的落盘那一份也就一直不换：这个会话往后一直不提醒，报告里一个字都不写（没带这一节，「基线：」那一行也不出）。能照喂的输入：把会话目录里 `agent-watch-knowledge-sync.json` 的 `"reminded_at"` 改成 `NaN`、`Infinity` 或 `4102444800`（2100 年）。探针 `hand-edited-reminded-at` 六个值逐个喂，预期 NaN、Infinity、2100 年三个 `printed_after_ten_hours=False`，`null` 走「读不出、重新记」，字符串 `"0"` 与 `-Infinity` 照常带。只有手改或写坏才到得了（看门狗自己只写 `time.time()`），按「被手改」那一问记。

### 其余几种（成立或只是写明的结局）

| 情形 | 结局（读码；探针格） |
|---|---|
| 第一次起、没有落盘文件 | 上次提醒的时刻取这一只的 `watch_started`、当场落盘；第一次提醒最早在它起来之后满 60 分钟的那次退出，最晚照上面那一条（`first-start`：预期 59.9 不带、60 带、落盘的 `reminded_at` 是 0） |
| 会话目录取不到 | `--processes-only` 没给 `--session-dir`：只记在这一只里（文件头写了）。另有一种文件头没写：`watch.sh <agent 号>` 起的 `--agents`（`scripts/agents/watch.sh:329` 不带 `--session-dir`）、被看的子 agent 都还没有会话记录时，`knowledge_state_path` 交 None，note 写「取不到会话目录，基线只记在这一只看门狗里，下一只接不上」（`scripts/agents/agent-watch.py:1832`）（`agents-without-transcript` 预期 `state_path=null`）。技能段的起法不带 agent 号，这一种是旁路 |
| 落盘文件被截断 | 「读不出、重新记」，上次提醒的时刻重置成这一只起来的时刻；落盘那一份基线与此刻之间的变动丢了，「基线：」那一行写明（`truncated-state`：预期点名 `[]`） |
| 换了仓（同一会话目录、不同 `--repository-root`，例：从仓副本里的 `watch.sh` 起） | 「记的是别的仓、重新记」，并**覆盖**落盘那一份；回到原仓的下一只又判「别的仓」、再覆盖，两边来回时各自的变动都丢、计时都重置（`other-repository-shared-session`：预期点名 `[]`）。旁路，推的 |
| 换了会话 | 新会话目录里没有落盘文件，当成第一次起；上一个会话最后一次提醒之后的变动不报。与用户定的范围（「没派过子 agent 的对话收不到」那一行同一类）一致，不算缺口 |
| 两只同时到点、同时 `os.replace` | 后换上的那一份留下；两份都是各自「此刻」的快照，下一只比的是较早的那一份，只会多点、不会漏 |

## 第 2 问：范围里被 git 忽略的条目

### 打中：「没查成」下面给的两条命令看不见被忽略的条目

被忽略的条目由「listed = run_git(root, ["ls-files", "-z", "--others", "--ignored", "--exclude-standard", "--", *KNOWLEDGE_SCOPE])」（`scripts/agents/agent-watch.py:1718`）列出、逐个算摘要。查不成时给人看的只有两条：「[git_command_line(root, "status", "--short"), git_command_line(root, "log", "-5", "--format=%h %s")]」（`scripts/agents/agent-watch.py:1994`）。`git status --short` 不列被忽略的文件，`git log` 更不相干；而算摘要超时（30 秒）最可能就出在范围里的大文件或很多文件上，本仓范围里被忽略的正是 `.claude/singlefs-ai-sop/rules/` 那 11 份（`git ls-files -z --others --ignored --exclude-standard -- <范围八项>` 现查 11 行）。照抄那两条看不到它们改没改。

能照喂的输入：小仓 `.gitignore` 写 `/.claude/singlefs-ai-sop/`，`.claude/singlefs-ai-sop/rules/rule.md` 在；记基线；改 `rule.md`；到点时 `AGENT_WATCH_DIGEST_TIMEOUT_SECONDS=0`。这一节写「没查成：算内容摘要超过 0 秒…」，两条命令照抄退 0、输出里没有 `rule.md`。探针 `timeout-commands-miss-ignored` 预期 `exits=[0, 0] any_output_mentions_rule=False`。技能段说「写着「没查成」的，照它下面一行一条的命令自己看仓」，照做漏掉这一类。

### 打中（低）：只改忽略规则、内容没变，也标「被 git 忽略、内容摘要变了」

`knowledge_files_changed` 的被忽略那一半按「基线有、此刻没有」或反过来算变了，标签次序里排在「已不在」前面。两种只改规则的输入：

- 把 `.gitignore` 里 `/.claude/singlefs-ai-sop/` 那一行删掉：底下的规则一字没动，全部点名「被 git 忽略、内容摘要变了」，整个目录另在「新出现 [??] .claude/singlefs-ai-sop/」里（范围外的上层目录，不记摘要）。探针 `ignored-to-untracked`。
- 基线时未跟踪的 `.claude/kb/notes.md`，之后写进 `.git/info/exclude`：它从 `git status` 里消失、进了被忽略的表，标「被 git 忽略、内容摘要变了」，而内容没变。探针 `untracked-to-ignored`。

多点不漏，标签写错了「为什么点它」；按读码记，低。

### 时限到了，这一节写什么、基线换不换

- 写：提醒句照写「…哪些知识类文件变了这一趟没查成…」，一行「仓的现状：没查成：算内容摘要超过 30 秒，停下了（范围里的文件太大或太多）」，再两条命令（上面那一条打中）。没有「→ 同步：」那一行。
- 换不换：不换（`scripts/agents/agent-watch.py:2056` 那一句要 `now_snapshot` 不是 None）。起来时就超时的不落盘、上次提醒的时刻每只重置，之后超时的每次退出都带——两种结局见第 1 问那一条。

### 大文件、很多文件、符号链接、命名管道（读码成立，耗时待探针）

| 形状 | 读码的结局 | 探针格（预期，推的） |
|---|---|---|
| 很多文件 | 每份一次 `lstat`+`open`；`git ls-files` 那一条有自己的 30 秒；JSON 里每份一条摘要，落盘随之变大 | `ignored-many files=5000`、`files=50000`：打 `listing_seconds`、`baseline_seconds`、`state_bytes`；预期都在时限内 |
| 大文件 | 每读 1 MiB 核一次截止时刻，超过 30 秒走「没查成」 | `ignored-sparse bytes=8589934592`：打秒数与 GiB/秒；按吞吐外推多大会超时 |
| 符号链接（指到 `/dev/zero`） | `lstat` 认出是链接，记「链接 /dev/zero」，不打开 | `ignored-special-files`：子进程里记基线，预期 `hung=False` |
| 命名管道、套接字 | 只记类型，不打开；`git ls-files` 列不列它们也一并看 | 同上一格 |
| 删掉、新加 | 删掉的此刻取「不在」、新加的基线取「不在」，都与另一头不同，点名、标签是「被 git 忽略、内容摘要变了」；来源对 | 自证「范围里被 git 忽略的条目」那一格已覆盖，探针不重复 |

## 第 3 问：porcelain 加 -z 之后

### 打中：同一路径出两条时合成一个键

`git rm --cached` 一份被跟踪的规则（同目录另有被跟踪的文件，`git status` 才逐份列）之后，`--porcelain=v1 -z` 给两条：`D  .claude/rules/r.md` 与 `?? .claude/rules/r.md`（推的，探针打 `raw_entries`）。读法按路径元组作键、后一条盖前一条：「status_by_path[(path,)] = code」（`scripts/agents/agent-watch.py:1595`）。结局：

- 计数少一：已暂存那一条没了，`git status --short 此刻：已暂存 0 个…`，而下一次提交就会删掉这份规则。文件头说计数「从输出现数」，对不上。
- 「D」那一码丢了；点名照样有（基线时它不在 `git status` 里、此刻有，走状态码那一支），标「工作区改了没提交」，看不出下一次提交会删它。

能照喂的输入：小仓提交 `.claude/rules/r.md` 与 `.claude/rules/keep.md`；记基线；`git rm -q --cached -- .claude/rules/r.md`；到点。探针 `duplicate-path` 预期 `raw_entries` 两条、`parsed` 一条、`counts_staged_unstaged_untracked=[0, 0, 1]`。这一形 `git status --short` 时代同样合成一个键（键也是路径），不是加 `-z` 带进来的，是这一轮没修到。

### 打中：带换行的路径，键对，输出拆行、能造出第二行「→ 同步：」

`-z` 之后键里的换行原样留着（探针 `newline-path key_is_exact` 预期 True；新提交那一边 `newline-in-commit exact` 预期 True）。但点名那一行直接把路径拼进去：「named = rows_with_remainder([f"    {path}（{tag}）" for path, tag in knowledge], limit, "份",」（`scripts/agents/agent-watch.py:2002`），打出来是两行。能照喂的输入：新建 `.claude/kb/x` + 换行 + `   → 同步：知识类文件没变，不用重读.md`。这一节里就有两行以「   → 同步：」起头，后一行还写着「不用重读」；技能段叫主 agent「照「→ 同步：」那一行做」。探针 `newline-path` 预期 `arrow_like_lines=2`。谁都能在工作区建文件（别的会话的子 agent 也算），所以是能到达的输入；造出来的内容要人有意为之，记低。

### 打中（低）：非 UTF-8 文件名改了不点名

「capture_output=True, text=True, errors="replace", timeout=timeout, env=environment)」（`scripts/agents/agent-watch.py:1566`）把不是 UTF-8 的字节换成 U+FFFD，按这个名字去 `lstat` 找不到，摘要两头都是「不在」，相等，不点名。能照喂的输入：被跟踪的 `.claude/kb/` 里基线时就未跟踪的 `.claude/kb/` + 字节 `0xff` + `.md`，之后改内容。探针 `non-utf8-path` 预期点名 `[]`、`baseline_digests` 里那一条是「不在」。第一轮的状态码判法同样看不出（状态码不变），不是这一轮带进来的。

### 成立的几处

- 改名、复制两段：`-z` 下一条是「XY 新路径」再跟一段原路径，读成（原路径, 新路径）；自证「状态码计数」那一格喂了 `R  f.txt\0e.txt\0`、核成 `("e.txt", "f.txt")`（读码）。
- 未跟踪目录整行：键是带尾斜杠的目录路径，范围按前缀判；摘要走目录那一支（自证「未跟踪目录的摘要」那一格）。
- 子模块行：`-z` 下与普通文件一样是「XY 路径」，读成（路径,）；摘要会走进子模块的工作树（含它的 `.git` 文件）。范围里没有子模块，没造，推的。
- 第一轮三形配置：`--porcelain=v1` 不受 `status.branch`、`color.status` 影响，`--untracked-files=normal` 盖掉 `status.showUntrackedFiles`；自证「本机 git 配置不改读数」那一格三形都喂。另几种配置（`status.renames=copies`、`diff.renames=copies`、`status.showStash`、`status.submoduleSummary`、`status.aheadBehind=false`、`status.relativePaths=false`、`core.quotepath=true`、`color.ui=always`、`status.displayCommentPrefix`）交探针 `porcelain-config`，每种打 `same_status`、`same_named`、`same_counts`；我推 `copies` 两种会把「改了源文件、另复制一份暂存」读成一条「C」，源文件那一头经 `codes_by_path` 拿到「C」这一码——源文件本来就改了，点名不变，计数可能少一，待跑。
- `git diff --name-only -z` 那一边：按 NUL 切、`--no-renames`，带双引号与换行的路径都原样（自证「新提交文件名带双引号」与探针 `newline-in-commit`）。

## 第 4 问：「摘要没变就不点名」

摘要只取工作区里的内容；判法是「两头都记了摘要的只比摘要，有一头没记的比状态码」（`worktree_path_changed`，`scripts/agents/agent-watch.py:1870` 起），对此刻 `git status` 里的每个路径逐个判（「for path, code in now_codes.items():」（`scripts/agents/agent-watch.py:1895`））。

| 形状 | 结局 | 判 |
|---|---|---|
| 索引里换了内容、工作区原样（基线时「M 」、之后 `git update-index --cacheinfo` 往索引里放别的内容，状态「MM」） | 不点名（探针 `index-only` 预期 `now_code="MM"`、点名 `[]`） | 成立：这一节叫主 agent 重读的是工作区里的文件，内容没变、重读也读到同一份；索引里那一份一提交就由「新提交动到」点名。只暂存的不点名正是第一轮判决第 6 条要的 |
| 改过又改回基线时的内容 | 不点名（`changed-and-back` 预期 `[]`） | 成立：「相对基线变了」的定义下没变；中间那一版没被提交就没人读得到 |
| 改过、提交、又改回（新提交里动到了） | 「新提交动到」那一半照点 | 成立（读码） |
| 基线时没摘要的路径（基线时不在 `git status` 里） | 比状态码；这一轮把「按路径元组比」改成「按路径比」（`codes_by_path`），判法本身没变：基线时干净、之后只改执行位的照点，工具员写明没改 | 与第一轮判决「`reset --soft` 那一形不改，按线索记」同一类，已知 |
| 未跟踪目录里指到**目录**的符号链接改指向 | 不点名：目录的摘要只走 `os.walk` 给的文件名，「for file_name in sorted(file_names):」（`scripts/agents/agent-watch.py:1702`）；指到目录的链接在子目录表里、不跟随也不记，文件头「取底下每个目录项的相对路径与摘要合起来…链接记指向的路径」对它不成立 | 打中（低）。能照喂：被跟踪的 `.claude/kb/tracked.md` 在；基线时未跟踪的 `.claude/kb/drafts/` 里有 `notes.md` 与 `ref -> ../../../first`；之后把 `ref` 改指 `../../../second`。探针 `links-in-untracked-directory` 预期 `directory_link_named=[]`、`file_link_named` 点名 `.claude/kb/drafts/`。仓里不放软链接（`.claude/singlefs-ai-sop/rules/command-safety.md@仓里不放软链接`），所以只到得了违规的工作区 |
| 读不出的文件（权限 000）改了内容 | 摘要两头都是「读不出 Permission denied」，相等，不点名 | 打中（低）。能照喂：被跟踪的 `.claude/kb/` 里新建 `locked.md`、`chmod 000`，记基线；`chmod 600`、改内容、`chmod 000`。探针 `unreadable-file` 预期点名 `[]`（以 root 跑时不成立，探针打 `running_as_root`） |
| 非 UTF-8 文件名 | 见第 3 问那一条 | 打中（低） |

## 第 5 问：给人抄的命令

### 打中（低）：自证的「照抄能跑」只在不带特殊字符的仓路径上跑

自证那一格的小仓建在「root = os.path.join(work, "knowledge-reading")」（`scripts/agents/agent-watch.py:4125`）底下，`work` 是自证的临时目录，路径里没有 `$`、反引号、单引号、空格。取命令那一句「commands = [line[len(COMMAND_INDENT):] for line in unread_lines + capped_lines if line.startswith(COMMAND_INDENT + "git ")]」（`scripts/agents/agent-watch.py:4230`）之后逐条写进 `manual-command.sh` 交 `bash`。所以：

- 写进临时脚本交 `bash` 这一改本身没削弱判法：`bash 脚本` 与粘进 shell 的同一行切词相同（非交互 shell 不做历史展开；`shlex.quote` 用单引号，交互 shell 里 `!` 在单引号里也不展开）。
- 但这一格核不出引法的错：把 `git_command_line` 里的 `shlex.quote` 换成 `"'%s'" % part` 或直接不加引号，路径里没有特殊字符，六条照样退 0，自证照绿。没有一条开关弄坏引法（开关表里 `repositorymanualcommands` 起头的两条弄坏的是分号与连成一行）。
- 被判的代码在特殊字符路径上对不对：探针 `special-root-commands` 把仓建在 `q'uote $HOME `+反引号+`echo injected`+反引号+` dir/repo` 底下，取同样的六条照抄跑，预期 `exits` 全 0、`injected_marker_seen=False`（读码：`shlex.quote` 对 POSIX shell 是对的）。

### 成立的两处

- `repositorymanualcommandssemicolon` 只红点名的那一格：登记表那一行记着复跑读数「任务 20379 复跑这一百一十八条：点名的格与实际红的格逐格相同 118 条」（`.claude/gate.d/break-switch-replay-files.tsv:73`）。读码也对：行尾全角分号贴在最后一个引号外面，`git … status --short；` 与 `diff … HEAD；`、`log --oneline …..HEAD；` 退非 0，`log -5 '--format=%h %s'；` 把分号并进格式串、退 0——开关说明只点了前三种，对得上。这是主 agent 那一趟的读数，判决要引的话由主 agent 复跑。
- `who-touched.py` 那一行过不过管 Bash 的闸：项目代码闸放主 agent 直接跑的表里有「"scripts/agents/who-touched.py": "any"」（`.claude/hooks/lib_project_code_runs.py:1120`，表头 `MAIN_AGENT_TOOLS_RUN_DIRECTLY` 在 `.claude/hooks/lib_project_code_runs.py:1114`），参数不限；读看门狗输出的是主 agent，所以照抄（把 `<路径>…` 换成真路径）过得了。没喂钩子验，读码。`<路径>…` 不替换就交 shell 会被读成重定向——它是模板、文件头与技能段都写了要换，不算缺口。

## 第 6 问：文件头、技能段、`watch.conf` 与开关表

### 打中：`--processes-only` 告警退 3 没有「→ 接着盯」，「→ 同步：」与技能段都叫照它

「→ 同步：」那一行开头是「restart_first = "还有子 agent 在跑的，先照这份报告最后一行「→ 接着盯」起下一只看门狗"」（`scripts/agents/agent-watch.py:2029`），技能段同样写「先照报告最后一行「→ 接着盯」起下一只看门狗」（`.claude/skills/dispatch/SKILL.md:39`）。`--processes-only` 那一路告警时是 `print_report`、这一节、`return 3`（`scripts/agents/agent-watch.py:2523` 到 `scripts/agents/agent-watch.py:2525`），没有重起那一行；只有定时回报退 4 打「print("   → 接着盯就用 run_in_background 再起一个看门狗："」（`scripts/agents/agent-watch.py:2539`）。主 agent 同时有子 agent 在跑（由 `--discover` 那一只盯着）、而 `--processes` 那一只按「根分区余量过低」退 3 且离上次提醒满了：这一份报告的最后一行就是「→ 同步：」自己，它要找的「→ 接着盯」不存在。能照喂的输入：`watch.sh --processes`，`AGENT_WATCH_ROOT_FREE_BYTES` 设成低于阈值（自证「alert-processes」那一只的法子），落盘的 `reminded_at` 在两小时前。读码；文件头「都在重起看门狗的出路句之前」这一句对退 3 那一路是空的。

`--discover`、`--agents` 两路告警退 3 时，`print_active_list` 在这一节之后打「→ 接着盯」，有子 agent 在跑时它是最后一行，成立；一个子 agent 都没有时那一行不打，「→ 同步：」开头那一句的「还有子 agent 在跑的」不成立、跳过，也成立。

### 打中：文件头的落盘几句与行为不符

| 文件头原句 | 行为 |
|---|---|
| 「盯同一个会话时读写同一份：谁先满谁打、打了就换。」（`scripts/agents/agent-watch.py:132`） | 两只都打，后一只「离上次提醒」写错（第 1 问那一条） |
| 「取不到会话目录（--processes-only 没给 --session-dir）的只记在这一只里、下一只接不上」（`scripts/agents/agent-watch.py:131`） | 另有 `--agents` 而子 agent 都还没有会话记录那一种（第 1 问表里那一行） |
| 「没有、读不出、对不上仓、HEAD 找不到了，就此刻重新记一份、上次提醒的时刻记成这一只起来的时刻、」（`scripts/agents/agent-watch.py:129`）接下一行「落盘」 | 此刻那一份取不成时不落盘；核起点 HEAD 那一条 git 起不来、超时也写成起点 HEAD「在仓里找不到了」（`scripts/agents/agent-watch.py:1842`），把 git 的毛病说成基线的毛病（第 1 问那一条） |
| 「git status 写成一条的整个未跟踪目录取底下每个目录项的相对路径与摘要合起来」（`scripts/agents/agent-watch.py:142`） | 指到目录的符号链接不算（第 4 问那一条） |

### 成立

- 五种标签的字面：文件头第 144 行、技能段（`.claude/skills/dispatch/SKILL.md:39`）与代码里 `COMMITTED_KNOWLEDGE_TAG` 到 `IGNORED_KNOWLEDGE_TAG` 五个常量（`scripts/agents/agent-watch.py:625` 到 `scripts/agents/agent-watch.py:629`）逐字相同（读码）。
- 哪些退出带：退 3、退 4，满了才带；退 0 与 `report` 不带（`--agents` 那一路 `scripts/agents/agent-watch.py:3031` 起的分支、`report` 在 `:3022` 先返回）。文件头、技能段、`watch.conf` 那几行注释三处说法一致。
- `watch.conf` 的 `repository-report-lines` 注释四句（上限、写 0 不打也不读不记、落盘位置、读哪个仓）与代码对得上。
- 开关表这两批新加与改过点名的每一条（`knowledgecheckcommandomitted` 到 `repositorystatenotread`，`switches-r1.txt` 与 `switches-now.txt` 的 `diff` 列出这两批新加 18 条，两份清单在草稿目录 `~/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-opus/`，用 `grep -nE` 从第一轮基底与此刻两版的开关表抽出）逐条对着代码读过：「弄坏哪一处」写的函数与改法都是代码里那一处的 `BROKEN_DETECTION ==` 分支所做的事；点名的格对不对由任务 20379 的逐条复跑判（登记表那一行），读码不另判。

## 第 7 问：第一轮按线索记的两处

### 命名管道：成立（读码，另交探针复核）

自证那一格的形状：「os.mkfifo(os.path.join(repositories["fifo"], ".claude/kb/pipes/channel"))」那一句在 `scripts/agents/agent-watch.py:4425`，同目录先写一份 `notes.md`，小仓里只有 `.claude/main-agent.md` 被跟踪，所以 `git status` 给的是「?? .claude/kb/」、在范围里、走目录摘要、`os.walk` 走到管道。照常时 `file_digest` 先 `lstat`、不是普通文件只记类型；弄坏开关 `knowledgefiletypeunchecked` 下 `open(path, "rb")` 阻塞，看门狗记基线就卡住、60 秒到点被停、那一格红。第一轮探针走不到，是因为它小仓里 `.claude/` 整个未跟踪、状态行是范围外的 `.claude/`；这一格把 `.claude/` 跟踪住，绕开了。探针 `fifo-under-tracked-claude` 用同一形状、照常与开关下各起一次子进程记基线，预期照常 `hung=False`、开关下 `hung=True`（20 秒到点停那一组）。

### 时序窗口：第一轮那一处没了（读码）；剩下四处固定时限只会误红（推的，没量）

「先落盘基线、再改仓、改完才起」之后，要改仓的那几只（`changed`、`rules`、`worktree`、`capped`、`processes`、`due-alert`）退出时一定满（落盘的 `reminded_at` 在两小时前），改动也一定在它们起来之前做完；`gap-second` 在 `gap-first` 退出之后才改仓、才起。第一轮那个「回报周期 0.4 分钟、改动没做完就到点」的窗口不在了。

剩下的固定时限：

| 时限 | 在哪 | 超了会怎样 |
|---|---|---|
| 告警退出要在起来之后 1 分钟以内 | 「SELFTEST_ALERT_WATCH_MINUTES = "1"」（`scripts/agents/agent-watch.py:3995`）；`alert`、`alert-processes`、`gap-first`、`due-alert` 四只 | 机器忙到第一次整查超过 1 分钟：「告警退出而没满不带」那三只满了、带这一节，判红；`gap-first` 那只带了就把基线换成此刻，`gap-second` 跟着可能红 |
| 命名管道那一只要在全体起来之后 60 秒内退出 | 「SELFTEST_FIFO_WATCH_SECONDS = 60」（`scripts/agents/agent-watch.py:3996`）与「collect("fifo", launched_at + SELFTEST_FIFO_WATCH_SECONDS - time.time())」（`scripts/agents/agent-watch.py:4478`） | 判红 |
| 别的每只 120 秒、照抄每条 60 秒 | `collect(key, 120)`、`SELFTEST_SHELL_SECONDS` | 判红 |

四处超了都是误红、不会误绿：开关下该红的格不靠这些时限变红（例：`repositoryonalert` 让告警退出一律带，与时刻无关）。按弄坏开关复跑那一趟 118 条逐条相同（登记表那一行），说明那一趟没撞上；忙时会不会撞上没量过。

## 打中之后的四句

照 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`：「打中之后先问四句：它分不分辨臂；系统当时看得到要被区分的东西吗；它满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的格上还中不中。」（`.claude/rules/evidence-discipline.md:125`）。这里的「臂」是第一轮判决「三、改法」那几行（已换上的做法）与下面我提的零轮改法。

| 打中 | 分辨臂 | 系统当时看得到 | 满足的分句 | 第一轮改法还中不中 |
|---|---|---|---|---|
| 间隔约两倍 | 分辨：只在退出时判满的做法中，起来时就知道到期时刻的 F1 不中 | 看得到（起来时就读到 `reminded_at`） | 用户定案「满一小时就出」的「就出」 | 「落盘…满 `--max-minutes` 就带」那一行照中（它就是现状）；F1 修 |
| 起来时查不成、每只重置 | 不分辨 F1（F1 照中）；分辨 F2 | 看得到（它知道这一趟没查成） | 同上；另与文件头「没查成的那一次不换」不冲突，冲突的是「满了就带」 | 「算摘要带时限，超时走没查成」那一行照中（它造出这一格）；F1 照中；F2 修 |
| 两只都打 | 不分辨用户定的臂；是文档与实现之间 | 看得到（退出时可以重读文件） | 文件头「谁先满谁打」 | 照中；F3 修（或只改文件头一句） |
| 落盘在打印之前 | 不分辨 | 看得到 | 用户定案「满一小时就出」（出的东西没送到） | 照中；F4 修 |
| `reminded_at` NaN / 将来 | 不分辨 | 看得到 | 第 1 问「落盘文件被手改」 | 照中；F5 修 |
| 「没查成」命令漏被忽略的 | 不分辨 | 看得到（被忽略的表就是它自己列的） | 文件头「再一行一条手动看的命令」与技能段「照它下面一行一条的命令自己看仓」 | 「手动命令拆成两行…」那一行照中；F6 修 |
| 同一路径两条 | 不分辨 | 看得到（输出里两条都在） | 文件头「从输出现数」 | 「改用 `--porcelain=v1 -z`」那一行照中；F7 修 |
| 带换行的路径拆行 | 不分辨 | 看得到 | 技能段「照「→ 同步：」那一行做」只该有一行 | 照中；F7 修 |
| `--processes-only` 退 3 没有「→ 接着盯」 | 不分辨 | 看得到（它自己知道没打那一行） | 「→ 同步：」那一行「先照这份报告最后一行「→ 接着盯」」 | 「文件头、技能段…照改完的行为重写」那一行照中；F8 修 |
| 自证核不出引法 | 不分辨 | 不适用（自证的事） | 第 5 问「自证还核不核得出照抄能跑」 | 「自证补格」那一行照中；F9 修 |

低的几条（只改忽略规则的标签、非 UTF-8、指到目录的链接、权限 000、`--agents` 不落盘、换仓覆盖、临时文件留下）四句同上一类：都不分辨臂、系统都看得到，不另列。

## 改法（只在我的模型上量过、被攻过零轮；下表每格都没跑过）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| F1：`report_deadline` 取「起来时刻 + M」与「落盘的上次提醒 + M」里早的那个，已满就第一次整查后当场退 4 | 间隔约两倍 | 推的，待主 agent 跑：探针 `exit-chain arm=exit-when-due` 是这一臂，与 `arm=as-is` 喂同一批时刻表（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 里零轮改法先在探针里做 A/B 那一条，转述）；预期最坏间隔从约 2M + d 降到 M + d |
| F2：起来时查不成也落盘一份「只有 `reminded_at`、快照为空」的，或带了「没查成」这一节也把 `reminded_at` 记成此刻（快照不换） | 起来时查不成、每只重置；之后查不成每次都带 | 推的（没实现）；后一种让「没查成」也每满 M 才出一次，前一种只修重置 |
| F3：退出判满之前重读落盘那一份，`reminded_at` 取文件与内存里晚的那个；或文件头改成「几只各自满了各自打」 | 两只都打 | 推的 |
| F4：先打印这一节、`flush`，再落盘 | 落盘在打印之前 | 推的 |
| F5：读回的 `reminded_at`、`recorded_at` 不是有限数、或晚于此刻，当「读不出」 | NaN / 将来 | 推的 |
| F6：「没查成」下面再给一条 `git -c core.quotepath=off -C <仓> ls-files -o -i --exclude-standard -- <范围>`（与 `ignored_knowledge_paths` 同一串参数） | 「没查成」命令漏被忽略的 | 推的 |
| F7：同一路径出两条时两码都留（例：键带上条序号，或计数先数原始条目）；输出里路径过一遍转义（换行、控制字符写成 `\n` 这类）；`run_git` 用 `surrogateescape` 读、算摘要按原字节找文件 | 同一路径两条、换行拆行、非 UTF-8 | 推的 |
| F8：`--processes-only` 告警退 3 也打「→ 接着盯就用 …watch.sh --processes」，或「→ 同步：」那一句按有没有这一行改写 | `--processes-only` 退 3 | 推的 |
| F9：自证「给人抄的命令照原样跑得起来」那一格的小仓放进带 `'`、`$`、反引号与空格的目录；配一条弄坏开关把 `shlex.quote` 换成 `'%s'` | 自证核不出引法 | 推的 |

## 没打中的形状

| 形状 | 取样范围 | 结局（读码；探针格是预期） |
|---|---|---|
| 两只看门狗同时 `os.replace` | 两种先后 | 原子换上，留下的是某一只的「此刻」，只会多点不会漏 |
| 告警密时漏提醒（第一轮第 7 条那一形） | 探针时刻表六种 × 重起间隔三种 | 用户定的改法修到了「告警退出满了也带」：每 10、每 30 分钟告警时每满 60 分钟后第一次退出就带；剩下的是间隔上界那一条 |
| 被忽略条目删掉、新加的标签与来源 | 自证那一格 | 对 |
| 被忽略的符号链接、命名管道、套接字 | 各一个 | 不打开，不卡（`ignored-special-files`） |
| porcelain `-z` 下 R、C 的两段次序 | 自证「状态码计数」那一格的输入 | 对 |
| `git diff --name-only -z` | 带双引号、带换行两种文件名 | 键对 |
| 第一轮三形配置 | 自证 | 不改读数 |
| 只暂存、只改执行位（基线时就在 `git status` 里的） | 自证两格 | 不点名，第一轮判决第 6 条修到 |
| 索引里换内容、工作区原样；改过又改回去 | 各一形 | 不点名，判它对 |
| 特殊字符仓路径上照抄 | 单引号、`$`、反引号、空格同在一个路径里 | 预期六条退 0、不执行反引号里的命令 |
| `repositorymanualcommandssemicolon` 只红点名的格 | 登记表那一行的复跑读数 | 成立 |
| `who-touched.py` 过闸 | 读 `MAIN_AGENT_TOOLS_RUN_DIRECTLY` | 成立 |
| 命名管道格造不造得出卡死 | 照常、弄坏开关下各一次 | 预期照常不卡、开关下卡 |
| 第一轮时序窗口 | 读 `selftest_repository_report` 的先后 | 没了 |

## 这条腿自己的限度

- 一个数都没跑：探针与复跑脚本写好、语法过了（`python3 -m py_compile`、`bash -n` 退 0），读数全是推的；探针第一次跑可能有格自己搭场景失败（`setup_failures` 不为 0 时 `rerun.sh` 退 1、各格照打），那几格的判定退回「读码」。
- 时钟是假的：探针换掉的是被导入模块里的 `time.time`，退出时刻由探针喂；看门狗主循环（`run`、`run_processes_only`）没起，「起来之后多久定时退出」照 `scripts/agents/agent-watch.py:2988` 那一句在探针里重算（`run_chain`）。真起看门狗跑满 60 分钟一圈没做。
- 没喂钩子：`who-touched.py` 过不过闸、照抄的 git 命令过不过检出钩子，只读了表。
- 没在第二台上想：会话目录在共享根上，两台的钟差会不会让 `reminded_at` 落在将来，没量；看门狗只在主 agent 那一台起，推它碰不到。
- 只攻了第 1 到 7 问；第 8 到 11 问是辩方腿的，没读它的产出（禁读清单）。
- `.claude/singlefs-ai-sop/rules/command-safety.md`、`.claude/rules/` 几份规则照「开工先读」读了；`CLAUDE.md` 与 kb 没引条款，没核 kb 行号。

## 要主 agent 跑的命令

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-opus-run /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r2-opus-model`
  - 在哪起：`--cwd` 是这条腿的草稿目录 `/home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-opus`；运行目录 `…-opus-run` 由包一层的脚本先删再建。新 shell 里起。
  - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 算跑完（`exit=1` 是有格搭场景失败、照样读各格；`exit=3` 是完整性闸红、这一趟作废）；`task-server/task log <号> --all` 里 `PROBE name=watchdog … sha256=5b6030270788db16981d3e0813584cf7aef278200eeda03ca5ae9fc4ab86555c`（对不上就不算判的这一版）、`PROBE name=done emitted=<N> setup_failures=0`、`name=rerun_done probe_exit=0 probe_lines=<N>`，两个 N 相等。判决要引的数是这些行：`exit-chain-worst`（两臂）、`unreadable-from-start`、`unreadable-later`、`digest-timeout-chain`、`two-watchers`、`replaced-before-print`、`hand-edited-reminded-at`、`timeout-commands-miss-ignored`、`duplicate-path`、`newline-path`、`non-utf8-path`、`special-root-commands`、`fifo-under-tracked-claude`。
  - 范围声明闸若认它是「实验复跑」而拒，照拒绝说明补 `SINGLEFS_RUN_SCOPE`（第一轮同形的命令任务 20338 交成了，推它不拒）。

## 没做什么

- 不判正推、辩方那几格（第 8 到 11 问）；不替主 agent 采纳；没有副本上的数。
- 没往统一任务服务交任务；判决要引的数都没复跑（命令在上一节）。
- 没改被判的任何文件；改法 F1 到 F9 都没实现。
- 没建仓副本、没编译：第 3b、3c 步的崩溃枚举、稀疏盘、状态数不适用（这一题没有盘与崩溃状态）。

## 草稿与清理

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-opus/` 里只有三份小文本（`switches-r1.txt`、`switches-now.txt`、给 `replace-batch.py` 的 `spec-fixes.json`）与认领、写者锁两份；没有仓副本、编译目录、库拷贝，没有要删的。
- 模型目录 `research/prompts/watchdog-knowledge-sync-governance-r2-opus-model/` 里语法检查落的 `__pycache__` 已删。
