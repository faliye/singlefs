# 看门狗知识同步提醒治理文件轮第二轮：辩方腿报告（watchdog-knowledge-sync-governance-r2，2026-11-18）

辩方腿，做第 8 到 11 问：复核第一轮判决 `research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md` 够不够得着、是不是同样打中别的写法。方案按 `crates/` 与这些治理文件今天的实现来谈；这一轮没有改 `crates/`。判定用三个词：站得住、够不着、同样打中别的臂；推不出的算打中，单列标出。

## 读到的版本与范围

被判五份读主工作区此刻的版本，`sha256sum` 现算，与 `research/prompts/watchdog-knowledge-sync-governance-r2-snapshot/judged-sha256.txt` 里五行逐行相同（量的）：

| 文件 | sha256 |
|---|---|
| scripts/agents/agent-watch.py | 5b6030270788db16981d3e0813584cf7aef278200eeda03ca5ae9fc4ab86555c |
| scripts/agents/watch.conf | 57eb0dc5fdf6fcbd0f12061e3c09811b8b68c4a939c71d8621741b00c99bc7c5 |
| .claude/skills/dispatch/SKILL.md | 38470a8cad2f9701d0ba279437ad77bba68b6ec294445c618a4f0f7b0e05bbcc |
| .claude/hooks/selftest-registry.tsv | b20fcef98128ec9f81a2a10e156292975706bd5c24e980f2b4be9ca8dda303fe |
| .claude/gate.d/break-switch-replay-files.tsv | c81f96c535192f173dbe9f0e77a12e0bbbe07d531bafbc23a19e2b62c158d10f |

读了：判决、两条腿的第一轮报告、两批工具员报告（`~/.local/share/singlefs/work/watchdog-knowledge-sync-r1-fixes/result.md`、`~/.local/share/singlefs/work/watchdog-knowledge-sync-r1-fixes-key/result.md`）、任务 20336、20338、20378、20379 的日志（只读 `task log`）、第一轮攻方腿的探针源码、`records/2026-11-17-subagent账本自上而下清欠.md` 第二百六十二节。禁读清单里的文件与目录一个没读。

## 模型与复跑

一份探针（不是独立模型：它直接导入被测的 `agent-watch.py`，在临时小仓里复现下文点名的几处现象，只打现象、不判对错）。模型目录 `research/prompts/watchdog-knowledge-sync-governance-r2-sonnet-model/`，行数与 sha256 是 `wc -l`、`sha256sum` 现数的：

| 文件 | 行数 | sha256 |
|---|---|---|
| probe/defense_probe.py | 201 | 5663aecc8fce3d9deff496e4d5b1a0cb06f8f08817c0323a14a390000e3b79af |
| tools/rerun.sh | 172 | c90d3a01ad23025b3fc54049ef88745c77252bb948e881c0cba211010fd68b44 |
| tools/rerun-from-a-clean-work-directory.sh | 23 | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |

`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps`（调探针、末行完整性闸：`DEFENSE name=done emitted=N` 的 N 等于前面的 DEFENSE 行数，对不上退 3）；它自带的 `--selftest` 测的是模板原来的步骤，改后不适用，不当自证。`rerun-from-a-clean-work-directory.sh` 是第一轮正推腿同名脚本的原样拷贝（只删名字以 `-run` 收尾、在工作根下的那一个运行目录）。语法检查：两份 `bash -n`、探针 `py_compile` 都过（量的，已删缓存目录）。探针一格一个小仓，只起 git（带 60 秒时限），命名管道只造、不打开，没有常驻进程；内存与时间没量过，推的：秒级、几十 MiB。下文凡写「探针 <格名>」的读数都没跑出来，一律是「推的，待主 agent 跑」，命令在末尾「要主 agent 跑的命令」。

## 判定一览

### 第 8 问：第一轮坐实的 9 条（条号照判决「二、判决」）

| 判决条目 | 辩方判定 | 一句话（细节在「第 8 问」一节） |
|---|---|---|
| 1 范围第八项永远点不到 | 站得住；改法的比较缺一个更小的对照（够不着） | `.gitignore:8` 忽略整个目录是量的；判决没拿「只把仓根被跟踪的版本号文件算进范围」这个更小的改法比过 |
| 2 带空格或引号的路径不点名 | 站得住；改 porcelain 不是唯一出路 | 去引号解析也能修这一条，但只有 porcelain 加 -z 一处同时盖住第 2、3 条 |
| 3 本机 git 配置改得掉读数 | 站得住；改法同样打中别的配置（推的，待探针） | 三种配置修了；`status.renames` 这类还能改读数；判决把 `status.branch` 记成线索偏弱，任务 20338 已量到 |
| 4 继承了 GIT_COMMON_DIR 时查不成 | 站得住；同样打中另 8 个变量（量的） | git 自己列 15 个仓内变量，补丁只盖了其中 7 个 |
| 5 给人抄的命令跑不起来 | 站得住 | 任务 20338 退 129、128；现在每条一行，任务 20379 的开关复跑照这一格对上 |
| 6 内容一字没变也点名 | 站得住；改法行够不着它自己的红样本 | 任务 20338 的 chmod-only 是起来时干净的文件，修补后仍点名 |
| 7 该提醒而不提醒 | 站得住；用户定的改法比另三种好，代价表漏四项 | 没查成不推进上次提醒的时刻、上界不是一小时、没有锁、写盘有逃出的异常 |
| 8 两条开关点名的格逐字相同 | 站得住；判据（格名不同）与实际保护的东西（断言不同）不是一回事 | 修补后又出一对同名的，断言不同，不算缺陷 |
| 9 文档与代码对不上的四处 | 站得住 | 四处现查都已改；`grep -c 另六条` 得 0 |

### 第 9 问：按线索记的 2 条与核不动的 1 条

| 条目 | 判定 | 一句话 |
|---|---|---|
| 10 命名管道（无时限、不判类型） | 「按线索记、一并改」够了 | 任务 20379 里 `knowledgefiletypeunchecked` 第 44 条对上，被跟踪的 `.claude/` 底下红样本红过；同一处还漏一个逃出的异常 |
| 11 自证时序窗口 | 「一并改」够了，不必先量 | 窗口靠「先落盘、再改仓、改完才起」整个去掉；剩下的 60 秒、120 秒是防卡死的挂钟上限，FIFO 那一格共用起算点且最少给 1 秒（推的） |
| 行数上界（核不动） | 站得住，该先量 | 读码上界现在是 3×上限+15（上限 20 时 75 行），不是 70；探针 line-bound 量 |

### 第 10 问：不采的 4 条

| 条目 | 判定 | 一句话 |
|---|---|---|
| 不列此刻全部路径 | 结论站得住，理由够不着 | 「与清单上限相抵」不成立（清单本来就有上限）；站得住的理由是前 20 行不带信息；这是用户原话的字面，该留在交用户里 |
| 「多做 6 项」 | 够不着（对第八项范围与内容摘要） | 记录里主 agent 写给工具员的范围是七项、没有摘要；判决「不算多做」只对另四项成立 |
| 超时变量设成非数字 | 站得住（就这个变量）；够不着它背后的隔离 | 新代码在看门狗起来时与每次退出时有三处没防护的异常路径，会让退出码不再是 3 或 4 |
| 行宽封不住 | 站得住 | 仓里全部 427 条提交标题最长 156 个 awk 字符，范围内 844 个被跟踪路径最长 124（量的，`git log --format=%s`、`git ls-files` 的输出过 awk 的 length） |

### 第 11 问

| 条目 | 判定 |
|---|---|
| 两批修补对「三、改法」九行 | 九行都有实现；少做 1 处（改法行 61 只修了判决红样本的一半）、做偏 3 处（行 58、60、64，都无害）、多做若干（无害，列在「第 11 问」） |
| 工具员写明没做的五件 | 只有「基线时干净、之后只改执行位」算缺口（归判决改法行，不归工具员）；没加锁算判决漏项；另三件不算 |
| 主 agent 用 python 整行换登记表一行 | 字面对不上 `./CLAUDE.md:100`；实质与 `replace-once.py` 等效的部分有（恰好一行断言），原子换上与新串判别没有 |

## 第 8 问：逐条

### 条 1：范围第八项永远点不到

- 理由站得住。现查（量的，主工作区只读）：`.gitignore:8`「/.claude/singlefs-ai-sop/」；`git check-ignore -v .claude/singlefs-ai-sop/rules/sop-first.md` 命中 `.gitignore:8`；`git ls-files -z -o -i --exclude-standard -- <范围八项>` 列出 11 份、全在 `.claude/singlefs-ai-sop/rules/`、用时 0.007 秒。`./CLAUDE.md:25`「@.claude/singlefs-ai-sop/rules/sop-first.md」起七份规则就在这个目录里，漏掉它们等于漏掉每个会话开头载入的规则。
- 理由只打这一个条目：范围另七项都被跟踪（`git ls-files` 在它们底下 844 个路径，量的）。
- 够不着的地方在改法的比较：判决「三、改法」第 2 行写「按内容摘要直接比」，没有和更小的改法并排。第一轮攻方腿自己列过：范围表删掉这一项，或把仓根的 `.singlefs-ai-sop-version` 算进知识类文件（它的「自己提的改法」表）。该文件被跟踪、内容是 0.0.74、`git log` 有 3 次提交动过它（量的）。更小的改法抓得到「换包」，抓不到「原地改一份上游规则」；而 `.claude/agent-common.md:50`「`.claude/singlefs-ai-sop/` 与 `~/.claude/` 谁都不碰」说明原地改本来就是违规动作。实现的版本（`scripts/agents/agent-watch.py:1718`「listed = run_git(root, ["ls-files", "-z", "--others", "--ignored", "--exclude-standard", "--", *KNOWLEDGE_SCOPE])」加摘要、标签、一条开关、一格自证）覆盖更全，代价是每次快照多读 11 份小文件。不推翻，但「改法选得对」没有被比过。
- 什么会推翻：`git check-ignore` 不再命中 `.gitignore:8`，或 `git ls-files .claude/singlefs-ai-sop` 非零。

### 条 2：带空格或引号的路径不点名

- 理由站得住：判决第 28 行引的两条读数，任务 20338 日志里 `PROBE name=quote-committed reminder=unchanged named_count=0 other_count=0` 我现读到（量的）。
- 「改 porcelain 是不是唯一出路」：不是。三条路：一，`status --porcelain=v1 -z`（实现，`scripts/agents/agent-watch.py:614`「GIT_STATUS_ARGUMENTS = ("status", "--porcelain=v1", "-z", "--untracked-files=normal")」）；二，保留 `--short`、自己按 C 字符串规则去引号，只修条 2、不修条 3 的 `status.branch` 与颜色；三，`run_git` 里设 `GIT_CONFIG_GLOBAL=/dev/null` 与 `GIT_CONFIG_NOSYSTEM=1`，修条 3、不修条 2，还会让只认全局配置的项（例：`safe.directory`）失效（推的，没试）。只有第一条一处盖住条 2 与条 3，所以判决的选择站得住；「唯一」不成立。
- 同一形状在别处：`git diff --name-only` 那一边（`:1866`）已同改 `-z`；`git log --format=%h %s` 只带标题，不涉路径。
- 什么会推翻：任务 20378 换上后自证里「工作区路径带空格与双引号」「新提交文件名带双引号」两格红。

### 条 3：本机 git 配置改得掉读数

- 理由站得住：任务 20338 `global-config-untracked-hidden … named_count=0`、`global-config-color-always … other=["新出现 [ \u001b] 31mM\u001b[m .claude/kb/k.md"]`（量的）。判决第 29 行把 `status.branch=true` 那一形记成「计数多不多 1 没逐字核，按线索记」偏弱：同一趟的 `PROBE name=global-config-status-branch` 行 `baseline_status="[('main', '##')]"` 已经把 `## main` 读成路径 main、状态码 `##`，是量到的。
- 修补盖的是复现过的三形：命令行显式 `--untracked-files=normal` 压过 `status.showUntrackedFiles`；`--porcelain` 不上色（推的，按 git 手册）；自证加了三种配置的格（`scripts/agents/agent-watch.py:4185`「for label, config_text in (("status.branch=true"」）。不是整类：`status.renames=false` 会把改名读成「D 加 A」两个键，`core.excludesFile` 会改「被 git 忽略」那张清单与未跟踪的显示（推的）；这些仍是 `.claude/agent-common.md:30`「不读会改变判定结局的本机独有状态」说的那一类。读数不致错（改名两头仍都点名），所以不推翻判决，判「同样打中别的配置，修补只盖了复现过的三个」。探针 rename-config 量改名那一形。
- 什么会推翻：探针 rename-config 里 `status.renames=false` 那一行 `named` 与「没有配置」那一行不同。

### 条 4：继承了 GIT_COMMON_DIR 时查不成

- 理由站得住：任务 20338 `worktree-inherited-variables` 行里 `"git-common-dir-inherited": {"branch": null, "reason": "git log -1 --format=%H%x09%h%x09%s 退出码 128：fatal: bad object HEAD"}`（量的）。
- 同样打中别的臂：改法行 3 是主 agent 提的一张五个变量的补丁（`scripts/agents/agent-watch.py:611`「GIT_ENVIRONMENT_DROPPED = ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY", "GIT_ALTERNATE_OBJECT_DIRECTORIES",」）。`git rev-parse --local-env-vars` 此刻列 15 个（量的）：GIT_ALTERNATE_OBJECT_DIRECTORIES、GIT_CONFIG、GIT_CONFIG_PARAMETERS、GIT_CONFIG_COUNT、GIT_OBJECT_DIRECTORY、GIT_DIR、GIT_WORK_TREE、GIT_IMPLICIT_WORK_TREE、GIT_GRAFT_FILE、GIT_INDEX_FILE、GIT_NO_REPLACE_OBJECTS、GIT_REPLACE_REF_BASE、GIT_PREFIX、GIT_SHALLOW_FILE、GIT_COMMON_DIR；补丁盖了其中 7 个（另加 GIT_NAMESPACE，不在那张表里），没盖：GIT_CONFIG、GIT_CONFIG_PARAMETERS、GIT_CONFIG_COUNT、GIT_IMPLICIT_WORK_TREE、GIT_GRAFT_FILE、GIT_NO_REPLACE_OBJECTS、GIT_REPLACE_REF_BASE、GIT_SHALLOW_FILE 共 8 个。同一个理由（从别的 git 命令里继承来、把查询指到别处）同样打中这 8 个；哪几个真会让这里读错没造样本（推的，其中 GIT_CONFIG_PARAMETERS 会让读数受 `git -c` 的配置影响）。按 git 自己的表去掉，或改成白名单，比补五个稳。自证只有 GIT_COMMON_DIR 一格；另四个新加的变量没有格（开关 repositoryinheritedcommondir 一条管五个，但格只造 GIT_COMMON_DIR；`grep -n 'GIT_OBJECT_DIRECTORY\|GIT_NAMESPACE\|GIT_PREFIX\|GIT_ALTERNATE_OBJECT' scripts/agents/agent-watch.py` 只命中文件头、开关说明与 611、612 行）。
- 什么会推翻：`git rev-parse --local-env-vars` 此刻的输出少于 15 个名字、或补丁表里已含上面 8 个。

### 条 5：给人抄的命令照抄跑不起来

- 理由站得住：任务 20338 `manual-commands-verbatim … exit=129 … stderr_first="error: unknown option `short；git'"`、`remainder-command-verbatim … exit=128`（量的）。
- 修补：`scripts/agents/agent-watch.py:1950`「def command_rows(heading, commands):」每条一行；自证逐条真跑 6 条（`:4231` 起）；任务 20379 日志「break-switch-replay 第 85 / 118 条对上：… AGENT_WATCH_BREAK=repositorymanualcommandssemicolon」（量的，日志第 248 行）。没有别的写法被同一个理由打中：技能与出路句里别处的命令各占一行。
- 什么会推翻：任务 20378 之后自证里「给人抄的命令照原样跑得起来」格红。

### 条 6：内容一字没变也点名

- 理由站得住：任务 20338 的 `stage-only`、`chmod-only`、`reset-soft` 三行都是 `named_count=1`（量的）。
- 改法行够不着自己的红样本。判决第 61 行「起来时有摘要的路径，摘要没变就不点名（状态码变了也不点）」；实现只给 `git status` 里的路径记起来时的摘要（`scripts/agents/agent-watch.py:1737`「digest_by_path = {path: content_digest(root, path, deadline) for key in status_by_path if digest_wanted(key) for path in key}」），到点两头都有摘要才只看摘要（`:1876`「if BROKEN_DETECTION == "knowledgedigestblind" or start_digest is None or now_digest is None:」）。任务 20338 的 chmod-only 起来时是干净文件（探针源码 `research/prompts/watchdog-knowledge-sync-governance-r1-opus-model/probe/knowledge_sync_probe.py:292`「repository = new_repository(run, "chmod-only", {".claude/kb/k.md": "原文\n"})」），不在 status 里、没有起来时的摘要，修补后仍点名；自证那格「只改执行位不点名」造的是起来时已改且已暂存、之后再 chmod，是另一个形状。工具员 result.md「没照判决字面做、或按线索记的」第一条自己写明了这一点。
- 更宽的改法没被比过：起来时给范围里全部被跟踪的文件记摘要，就同时盖住干净文件上的 chmod。量（`git ls-files`、`du -cb`）：范围内 844 个被跟踪文件合计 51599555 字节；按 1 GB/s 推每次快照多读约 0.05 秒（推的），每只看门狗起来一次、到点一次。它仍盖不住 `reset --soft`（那一形判决明说不改，来自新提交那一半）。
- 什么会推翻：探针 chmod-clean-at-baseline 的 `named` 是空表。

### 条 7：该提醒而不提醒（用户定的改法对另三种）

- 理由站得住（读码）：只在一只看门狗连着没有告警时才打，按告警退出基线随那一只丢掉；判决第 33 行自己写「告警实际多密没量过」。我试着从本机会话记录里数看门狗每次退出的类别，`~/.claude/projects/` 下最近 12 份记录里「定时回报：已经看了」与「→ 怎么办：」的计数被读码、引文污染（同一串字出现在读 `agent-watch.py` 的工具结果里），分不出真输出，没得到干净的数；所以仍是没量过。
- 另三种改法的代价（第一轮攻方腿报告第 8 问的表，判决沿用），对着现在的实现重算：

| 改法 | 攻方腿表里的代价 | 对着实现 | 比得对不对 |
|---|---|---|---|
| 告警退 3 也带 | 每次告警最多 70 行；多起 3 到 5 条 git；命名管道卡死搬到告警上 | 一节最多 75 行（读码，探针 line-bound 量）；一次快照 4 条 git（log、branch、status、ls-files），HEAD 动了再多 log、diff 两条，共 4 到 6 条，表里少算了 ls-files；卡死那一条现在有 30 秒总时限，不再成立 | 大体对，两处过时 |
| 告警那一支只改成离这只起来满 60 分钟才带 | 改动最小；告警频繁时仍一次都不打 | 被用户定的改法完全包含（后者也在告警退出时判满没满，但「满」从上一只落盘的时刻算） | 对 |
| 改技能：读到告警先自己跑 git status | 「不是机检，与用户「只加在看门狗里」对不上」 | 后半句不对：`records/2026-11-17-subagent账本自上而下清欠.md:784`「只加在看门狗里」答的是没起看门狗的对话要不要罩到；技能那一段只对派发、起看门狗的会话生效，并不冲突。真正的缺点是靠自觉，用户要的是不要做久了忘完了 | 结论对，理由有一半歪 |

- 用户定的改法（落盘、满一小时就出）：攻方腿表里列的代价，实现做了三项：`scripts/agents/agent-watch.py:1840`「run_git(root, ["cat-file", "-e"」核起点 HEAD 还在（改写历史走重新记）；自证三格；落在会话目录。没做锁（工具员 result.md 写明）。表漏了四项，各配读码与探针格：
  1. 没查成不推进上次提醒的时刻：`scripts/agents/agent-watch.py:2056`「if lines and now_snapshot is not None and baseline.state_path is not None and BROKEN_DETECTION != "repositorybaselinekept":」只在读到此刻快照时才记，而 `:2048`「due = now - baseline.reminded_at >= arguments.max_minutes * 60」一直为真；git 一直查不成或摘要一直超时，每一次告警退出都会打这一节的没查成几行，故障时它退化成第一种改法的噪声。探针 persistent-failure-repeats：预期三次 `printed_lines_per_alert_exit` 都不是 0、`reminded_at_unchanged` 为真。
  2. 上界不是一小时：「离上次提醒满」只在退出时判，定时回报的到点从这一只起来算（`:2490`、`:2988`「report_deadline = watch_started + arguments.max_minutes * 60」）。上次提醒后 59 分钟按告警退出、立刻重起，下一次到点退 4 在上次提醒后约 119 分钟；中间再有告警则在那次告警退出时补上。「每满 max-minutes 至少出一次」应写成「不晚于约两倍 max-minutes」（算术，推的，没造样本）。
  3. 锁：`--discover` 有单实例锁（`:2965`「lock_handle, holder = acquire_single_instance(state_dir)」，在 `if discover:` 之下），`--processes-only` 与点名 `--agents` 的看门狗不在锁里，会同时读写同一份落盘文件。同文件里 `watch-acks.tsv` 的同类问题登过并加了锁与开关（`:2877`「with open(os.path.join(state_dir, "watch-acks.lock"), "a", encoding="utf-8") as lock:」，开关 `acksunlocked`）。这里换上是原子的、读不到半截，代价是重复：两只在内存里的上次提醒时刻都是旧的，先后退出各打一节。探针 duplicate-reminders：预期 `first_printed`、`second_printed` 都不是 0。
  4. 写盘有逃出的异常：`save_knowledge_state` 只接 OSError（`:1778`「json.dump(record, handle, ensure_ascii=False)」所在的 try 只有 `except OSError`），摘要里 `scripts/agents/agent-watch.py:1667`「return "链接 " + os.readlink(path)」遇到目标不是合法 UTF-8 的符号链接会带代理字符，按 UTF-8 写文件抛 UnicodeEncodeError（不是 OSError），临时文件也不删。触发条件极窄（范围里一条这样的链接）。探针 non-utf8-symlink-target：预期 `outcome` 是「抛出 UnicodeEncodeError」。
- 判：用户定的改法仍是四种里最好的，判决站得住；「告警也带」那一种噪声更大且不修空档，「告警那一支满 60 分钟才带」被它包含，「改技能」靠自觉。表漏的四项该补进「交用户的」那张表。
- 什么会推翻：探针 persistent-failure-repeats 的三次打印行数有一次是 0（则「一直为真」不成立）；上界一项看 `:2490` 附近改成从 `baseline.reminded_at` 起算。

### 条 8：两条开关点名的格逐字相同

- 理由站得住：两条腿读到同一处，攻方腿 `cmp` 退 0；现在 `knowledgetagsource` 点名三格、`knowledgeworktreeblind` 点名七格，两边都有对方没有的（文件头 `scripts/agents/agent-watch.py:305` 起，量的）。
- 同一形状在别处：对文件头开关表（114 条 `AGENT_WATCH_BREAK`）逐条取「知识同步提醒（格名）」集合比，39 条有格；按失败行原文（「✗ …」引号里的全部）比，逐字相同的有 4 组（量的，`identical-cell-sets.py` 现跑的输出：`switches_total 114 with_knowledge_cells 39`，四组是 keptcopyage 与 keptcopyexists、loopnoise 与 otheragent 与 ownrefusal、outcomeignored 与 refusedwakes、repositorymanualcommandsjoined 与 repositorymanualcommandssemicolon；脚本 sha256 `783469031ed45b1e584f947e492b43b3237b9ab5a2eca48580a614dc2aa63404`，在草稿目录，只读文件头）。
- 判：前三组是这一轮之前就有的别的检测（不在这一轮判），说明这份文件头里早有「几条开关点名的失败行原文相同」而复跑照样对上的先例；只比格名时 finished 与 repositoryonfinish、repositoryonalert 与 rootfree 也同名，但前者各自另点名别的格、后者带「…（--processes-only）…」（文件头 `:441`），失败行原文不同。这一轮新出的只有最后一组：`joined` 弄坏条数检查、`semicolon` 弄坏真跑那一步，红在同一格的不同断言上，失败行前缀相同；任务 20379 日志「第 85 / 118 条对上」是格级的对上。这一对不算缺陷（断言不同、工具员报告写了为什么要加），但第 8 条「点名的格不同」这个判据挡不住它：判据该写成「断言不同」，不是「格名不同」。
- 什么会推翻：`identical-cell-sets.py` 现跑的 `identical_failure_lines` 里 repository 起头或 knowledge 起头的开关不只 joined 与 semicolon 这一组。

### 条 9：文档与代码对不上的四处

- 理由站得住，四处现查都已改（量的）：技能段 `.claude/skills/dispatch/SKILL.md:39`「各标五种来源之一」并带「→ 同步：」一行与 `python3 scripts/agents/who-touched.py <路径>…`；文件头五种标签（`scripts/agents/agent-watch.py:144` 起）；登记行里 `grep -c '另六条'` 得 0；技能里「末尾「→」那一行」改成「照「→ 同步：」那一行」。
- `.claude/hooks/selftest-registry.tsv:24` 的 git 子命令「mv、update-ref、log、branch、status、diff、ls-files、cat-file」加前四个共十二个，与代码并集逐个相同：`selftest_git` 用 add、commit、init、log、mv、symbolic-ref、update-ref 七个，`run_git` 用 branch、cat-file、diff、log、ls-files、status 六个，去掉重复的 log 是十二个（量的，`grep -o`）。
- 没改、也不在判决里的一句：文件头 `scripts/agents/agent-watch.py:23`「4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）」；这一句在基线提交里就有，不是这一轮写的；`--discover` 找不到在跑的子 agent 时它不成立（第一轮攻方腿第 11 问提过，判决没列入坐实也没列入不采）。
- 什么会推翻：技能段或文件头再次与 `GIT_STATUS_ARGUMENTS`、五个标签常量（`scripts/agents/agent-watch.py:625` 起）对不上。

## 第 9 问：按线索记的 2 条与核不动的 1 条

### 条 10：算摘要没有时限、不判文件类型（命名管道）

- 判决第 39 行写「改之前工具员先在被跟踪的 `.claude/` 底下造红样本」。做到了：`scripts/agents/agent-watch.py:4425` 造被跟踪 `.claude/` 底下的未跟踪目录、里面一份普通文件加一个没有写端的命名管道；`file_digest` 判类型（`:1669`「return f"不是普通文件 {stat.filemode(mode)[0]}"」）、整次摘要带时限（`:1646` 起的 `digest_deadline_from_now`）；开关 `knowledgefiletypeunchecked`（文件头 `:293`）把类型判断拿掉，任务 20379 日志「break-switch-replay 第 44 / 118 条对上：… AGENT_WATCH_BREAK=knowledgefiletypeunchecked，用了 158 秒」（量的）：命名管道那一格红在点名的格上；同一份自证里第 42 条 knowledgedigestuntimed 用了 98 秒，多出的约 60 秒与那一格等满 60 秒一致（推的）。先证红在顺序上变成了换上之后用开关反向证红，等价。
- 触发条件窄：git 不列命名管道，目录里还要有一份普通文件才会把目录列出来（`:4425` 行内注释）；被忽略的范围条目里命名管道不进列表（探针 fifo-reach：预期 `ignored_listed` 只有 plain.md、`status_keys` 里没有 only-pipe）。所以「先量发生频率」没有意义：量的代价比修的代价大，修的代价是一个函数、两条开关、一格自证。
- 判：「按线索记、一并改」够了，不必先量。同一处还剩三处没防护的异常路径，归第 10 问的「超时变量」一条。
- 什么会推翻：任务 20379 里第 44 条不是「对上」；或探针 fifo-reach 的 `ignored_listed` 含 channel。

### 条 11：自证的时序窗口

- 判决第 40 行：「机器忙时可能在改动做完之前就到点，相应的格偶发红」，没量过。修补把窗口整个去掉：要改仓的看门狗先在各自会话目录落盘一份上次提醒在两小时前的基线（`scripts/agents/agent-watch.py:4432`「due_states = {key: write_due_knowledge_state(」），再改仓，改完才起，回报周期从起来才算（`SELFTEST_DUE_SECONDS_AGO` 7200 秒比各格的 `--max-minutes` 都长，退出时一定满）；读仓那一半在本进程里直接调（`:4121`「没有时序窗口」）。基线在改动之前落、看门狗在改动之后起，「改动做完之前就到点」不可能发生，所以不需要先量。
- 还剩靠固定时间的地方：一，命名管道那一格的 60 秒（`:3996`「SELFTEST_FIFO_WATCH_SECONDS = 60」）从第一只看门狗起来算（`:4457` 的 `launched_at`），不是从它自己算（`:4478`「collect("fifo", launched_at + SELFTEST_FIFO_WATCH_SECONDS - time.time())」）；前面十几只各自等最多 120 秒（`:4477`），`collect` 里 `max(timeout_seconds, 1)`（`:4411` 的 `watcher.communicate(timeout=max(timeout_seconds, 1))`）最少给 1 秒，所以机器忙到前面那些耗过 60 秒时，命名管道那一只只剩 1 秒，没退出就被停并判红：偶发红的另一种形状（推的，没造样本）。二，60 与 120 秒是防真卡死的挂钟上限，去掉它们自证会被卡死的看门狗拖住，该留。任务 20379 现跑 119 趟、在开关复跑并行的忙机器上，汇总行「点名的格与实际红的格逐格相同 118 条」，没有一趟因此多红（量的，一次抽样，不支撑「没有」）。
- 判：「一并改」够了；剩下的命名管道那一格该把起算点改成它自己起来的时刻，量的代价是零（推的）。
- 什么会推翻：自证在忙的机器上出现「未跟踪知识类目录里有命名管道」格红而 `--selftest` 里没有开关设着。

### 行数上界（核不动）

- 判决第 49 行：探针 `scenario_line_bound` 因调错参数没量到，「上限 20 时最多 70 行」按读码记。现在的读码上界不是 70：固定 8 行（提醒句、「仓的现状」、分支与 HEAD、基线来源、新提交条数、git status 计数、别的路径、「→ 同步：」），三张清单各 20 行加各自的「…余 N 没列」一行与每行一条的命令（`scripts/agents/agent-watch.py:1960` 的 `rows_with_remainder`：点名那张 2 条、新提交与别的路径各 1 条），合计 3×20+8+7=75 行；`:2059` 在落不了盘时再插一行「基线没换成此刻」是 76。70 是命令改成一行一条之前的数（推的）。
- 判：核不动不改，站得住；该先量，探针 line-bound 一格秒级（造三张都超过上限的清单），预期 `lines=75`。行宽没有上界，见第 10 问。
- 什么会推翻：探针 line-bound 的 `lines` 不是 75。

## 第 10 问：不采的 4 条，替「该采」那一方写最强理由

### 不列此刻全部路径

- 该采的最强理由：一，用户原话的字面是「最新的git statis」（`records/2026-11-17-subagent账本自上而下清欠.md:784`「答「最新的git statis」」），实现给计数与相对上次的增量、没有此刻的清单；二，判决的理由（`research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md:44`「全列与清单上限相抵」）不成立：这一节本来就有封顶机制，`rows_with_remainder` 把三张清单都封到 `--repository-report-lines`（缺省 20）条再给「看全部」的命令，多一张「此刻 git status 前 N 条」只多 N+3 行，不需要全列；三，判决把它放在「交用户的」写「告知，不改」（`research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md:74`「这一节不列此刻 `git status` 的全部路径」），用户没被问，这是 agent 替用户把「git status」读成「状态的信息」。
- 另一边：此刻 `git status --short` 566 行，其中 346 行是 `A `、193 行 ` M`、12 行 `MM`、11 行 `??`（量的，`cut -c1-2 | sort | uniq -c`）；清单按路径排，前 20 行是别的会话暂存的新增，对「知识同步」几乎没信息；有信息的是增量，增量已经在。
- 判：结论（不列）站得住；理由够不着，该改写成「前 N 行不带信息、增量已覆盖」；因为这是用户字面的一种读法，该留在「交用户的」让他点头，而不是只「告知」。什么会推翻：用户回「清单要有」。

### 「多做 6 项」

- 该采的最强理由：`.claude/agent-common.md:52`「禁止自行扩大任务范围，只改自己的部分」；`records/2026-11-17-subagent账本自上而下清欠.md:794` 里主 agent 写给工具员的范围是七项、只要求逐份点名与排序，没有第八项、没有内容摘要；这两样出自工具员报告（`~/.local/share/singlefs/work/watchdog-knowledge-sync-reminder/result.md` 第 2、3 点），用户在 `:801` 答「换上并验」批的是主 agent 呈给他的那一版，查不到那一版含不含这两样；修补条 1 时又把第八项做大（列被忽略文件、新标签、开关、格）。
- 另一边：第八项是 `./CLAUDE.md:25`「@.claude/singlefs-ai-sop/rules/sop-first.md」起载入的七份上游规则所在的目录，没有它这一节漏掉每个会话开头读的规则，与用户的目的（知识不同步）一致；内容摘要补的是「状态码没变而内容变了」的漏报。
- 逐项判：范围第八项与内容摘要两项，判决「不算多做」够不着（出处没查到，「归第 1 条」把它当成既有范围里的 bug）；基线加增量站得住（`:786`「看门狗起来之后新进的提交」）；两个选项与配置键、写 0 关闭站得住（实现需要，与 `watch.conf` 各键写 0 同风格）；提醒句与「→ 同步」站得住（`:794`「这主要是一个提醒」）。
- 判：不必撤销，但这两项该写进「交用户的」让用户确认。什么会推翻：在派发提示或用户原话里找到这两项的出处。

### `AGENT_WATCH_GIT_TIMEOUT_SECONDS` 设成非数字抛 ValueError

- 该采的最强理由不是这个变量的概率，是隔离：文件头 `scripts/agents/agent-watch.py:122`「没满的退出不带，全部交回退 0、report 那几路不打，退出码不因这一节改变。」，承诺只在这一节没有逃出的异常时成立。新代码把暴露面放大了：看门狗起来时 `take_repository_baseline` 在循环之前跑（`:2492`），退出时 `knowledge_sync_section` 在告警打印之后跑；三处没人接的异常路径：`:1560`「timeout = float(os.environ.get(GIT_TIMEOUT_VARIABLE) or GIT_TIMEOUT_SECONDS_DEFAULT)」、`:1646`「seconds = float(os.environ.get(DIGEST_TIMEOUT_VARIABLE) or DIGEST_TIMEOUT_SECONDS_DEFAULT)」（这一轮新加，基线提交里没有）、`:1778` 写盘的 UnicodeEncodeError（条 7 的写盘异常一项）。起来时抛，看门狗起不来、告警一并没了；退出时抛，退出码是 1 不是 3，主对话读到 traceback。同文件有先例做防护：`root_free_report` 接 `(OSError, ValueError)`（`:1538`）。
- 判：就这个变量，判决站得住（只给自证用、概率近零）；够不着的是它背后的隔离问题，第一轮只想到一个触发点。出路（交主 agent 判，不是我替判决改）：`take_repository_baseline` 与 `knowledge_sync_section` 外层各接一次 `Exception`、记成「没查成：<异常名与原因>」，不静默吞，代价两处 try。什么会推翻：探针 non-utf8-symlink-target 的 `outcome` 不是「抛出 UnicodeEncodeError」（则第三处不成立，前两处仍是读码）。

### 行宽封不住

- 该采的最强理由：这一节写进主对话上下文，该封的是总字节数，不是行数；一行路径与标题没有上限。
- 量（量的）：仓里全部 427 条提交标题最长 156 个 awk 字符，范围内 844 个被跟踪路径最长 124，此刻 `git status --short` 最长一行 338；按 75 行都撞到最长算上界约 25 KB（推的），实际远小于它。没有一条超过千字的标题。
- 判：站得住。什么会推翻：探针 line-bound 的 `widest` 过千，或仓里出现过千字的提交标题。

## 第 11 问

### 九行改法逐行（判决「三、改法」那张表自上而下，行号是判决文件里的行）

| 判决行 | 实现 | 判定 |
|---|---|---|
| 57 porcelain、`diff -z`、改名两段 | `scripts/agents/agent-watch.py:614` 的 `GIT_STATUS_ARGUMENTS`、`:1577` 的 `parse_status_porcelain`、`:1866` 的 `diff --name-only -z` | 做到。多做（无害）：`git log` 加 `--no-show-signature`（`:1726`、`:1862`，挡 `log.showSignature`）、开关 `repositoryshortstatus`、`repositorydiffnewline` |
| 58 被忽略的条目按摘要比、标签另起 | `:1713` 的 `ignored_knowledge_paths`、`:629` 的 `IGNORED_KNOWLEDGE_TAG`、自证格「范围里被 git 忽略的条目」 | 做到。做偏（无害）：判决写「不经 git」，列文件经 `git ls-files`、内容不经 git（`:1718`）；列的是范围里全部被忽略的文件，不只第八项（量的 11 份） |
| 59 不继承的变量加五个 | `:611` | 做到；只有 GIT_COMMON_DIR 一格（条 4）；另 8 个仓内变量没盖（条 4） |
| 60 手动命令一行一条、`-c core.quotepath=off`、who-touched | `:1945` 的 `git_command_line`、`:1950` 的 `command_rows`、`:616` 的 `WHO_TOUCHED_COMMAND` | 做到。做偏（无害）：判决写「照抄交 `bash -c` 退 0 当样本」，第二批改成写进临时脚本再交 `bash`（`:4100` 起的 `run_selftest_shell_command`），因为算键的库对 `bash -c <现出的文本>` 整份拒算（`scripts/gate/lib_function_units.py:536`「折不出拒算 inline-script-not-literal」）；任务 20379 日志「拒算 0」（量的）。这不是 `.claude/agent-common.md` 说的「换写法绕过去」：拒的是「折不出 -c 的串」，命令文本本来就出自这份脚本的闭包，改成库认得的脚本文件没丢键里的信息（推的） |
| 61 摘要没变就不点名 | `:1870` 的 `worktree_path_changed` | 字面做到，红样本只修一半（条 6）：判决行写「起来时有摘要的路径」，任务 20338 的 chmod-only 起来时干净、不在这条里 |
| 62 落盘、满 `--max-minutes` 就带 | `:1751` 起的落盘与读取、`:2041` 的 `knowledge_sync_section`、自证三格（`:4586`、`:4590`、`:4597` 起） | 做到；没加锁、没查成不推进时刻、上界不是一小时（条 7） |
| 63 只读普通文件、整次摘要带时限 | `:1656` 的 `file_digest`、`:1650` 的 `check_digest_deadline` | 做到（条 10） |
| 64 自证补格、两开关分开、不靠固定窗口 | 五格（`:4204` 继承变量、`:4260` 三级取法、`:4278` 改名、`:4286` HEAD 往回挪、`:4295` 未跟踪目录）；`:305` 起两条开关点名的格不同；`:4432` 先落盘再改仓 | 做到。做偏（无害，更强）：判决写「按就绪文件之后再起算」，工具员 result.md 改成「先落盘基线、再改仓、改完才起」并去掉 `REPOSITORY_WATCH_MINUTES` 与就绪文件等待；剩一处固定时间（条 11） |
| 65 文档与登记行照行为重写 | 文件头、`.claude/skills/dispatch/SKILL.md:39`、`.claude/gate.d/break-switch-replay-files.tsv:73` | 做到（条 9） |

多做的量（量的，`grep -c`）：repository 起头的开关此刻 21 条、knowledge 起头 14 条，共 35，第一轮之前是 11 加 6 共 17，多 18（第一批 17、第二批 1）；改了点名的旧开关 12 条（工具员 result.md）。判决写「自证补格」，没有限开关条数，不算超出。

### 工具员写明没做的五件

| 件 | 判定 | 理由 |
|---|---|---|
| 文件头退出码行「4 定时回报（没有告警、子 agent 还在跑…」在 `--discover` 找不到在跑的子 agent 时不成立，没改 | 不算这一轮的缺口；算判决漏了去向 | 这一句在基线提交里就有（`scripts/agents/agent-watch.py:23`「4 定时回报（没有告警、子 agent 还在跑，到点叫醒主 agent 看一眼）」），不是这一轮写的；第一轮攻方腿第 11 问提过，判决没列入坐实也没列入不采。影响小：`.claude/skills/dispatch/SKILL.md:26`「它退出了（按告警退出、定时回报、全部交回）才再起一只」已规定任何一种退出都重起，没有子 agent 时链也接得上，靠技能、不靠看门狗输出 |
| `--processes-only` 没给 `--session-dir` 时不落盘 | 不算缺口 | `watch.sh --processes` 找得到会话就带 `--session-dir`（`scripts/agents/watch.sh:137` 的自检），找不到时「基线：」一行写明（`scripts/agents/agent-watch.py:1832`「取不到会话目录，基线只记在这一只看门狗里，下一只接不上」） |
| 几只看门狗同时盯一个会话时谁先满谁打、没加锁 | 不是照判决的缺口（行 62 没要求锁）；是判决漏项 | 条 7 的锁一项：`--processes-only` 不在单实例锁里，会重复提醒；同文件 `watch-acks.tsv` 的先例加了锁与开关；该补，代价一把 flock、一条开关、一格自证（仿 `:2871` 的 `update_acks`） |
| 基线时干净、之后只改执行位的文件照状态码点名 | 算缺口，归判决的改法行，不归工具员 | 它就是判决第 32 行 `chmod-only` 的形状（条 6）；工具员照行 61 的字面做并写明 |
| `reset --soft` 那一形不改 | 不算缺口 | 判决行 61 自己写「`reset --soft` 那一形不改，按线索记」（`research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md:61`） |

### 主 agent 用 python 整行换登记表一行，对不对得上 `CLAUDE.md` 那一行

- 条文：`./CLAUDE.md:100`「几个会话共写一批文件时只许这样改，不许整份重写」。`.claude/gate.d/break-switch-replay-files.tsv` 正是几个会话共写的：`git status` 此刻是 `MM`，判决第 14 行写「快照之后被别的会话改过别的行」（`research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md:14`）。
- 主 agent 的做法（背景材料「主 agent 打的两行」）：python 整行换、断言恰好一行以 `scripts/agents/agent-watch.py\t` 起头且第二列是 600。「恰好一行」与 `scripts/kb/replace-once.py` 的「恰好命中一次」同义；`grep -c '^scripts/agents/agent-watch.py'` 现得 1。`replace-once.py` 还多两样：写回「同目录排他新建临时文件、fsync、照原权限位与属主、改名换上」（`scripts/kb/replace-once.py:14`），与写前判新串里的会话号；那段 python 的代码没贴出来，这两样有没有无从核（推的）。
- 没有非用 python 不可的理由：旧串就是工具员报告给的那一整行，`replace-once.py` 吃整行字面。
- 判：字面对不上；「恰好一行」那一半实质等效；内容无误（第 73 行现读与任务 20379 日志逐字相符：118 条、790.1 秒）。是流程偏离、不是内容错误；下次整行换也经 `replace-once.py`（旧串给整行）。什么会推翻：主 agent 贴出那段 python，且它是同目录临时文件加改名换上。
- 另：`.claude/hooks/selftest-registry.tsv` 那一行主 agent 经两次定点替换改的，对得上；该行的 git 子命令十二个与代码并集逐个相同（条 9）。

## 要主 agent 跑的命令

一条，探针（它不起编译、门禁、测试，只在运行目录底下建 9 个小仓、起 git、导入被测脚本；`rerun-from-a-clean-work-directory.sh` 先删掉上一趟的运行目录那一个，只删工作根下以 `-run` 收尾的）。运行目录是草稿目录下的 `probe-run`。

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-sonnet -- bash /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-sonnet/probe-run /home/fy5090/code/singlefs/research/prompts/watchdog-knowledge-sync-governance-r2-sonnet-model
```

- 在哪个目录起：submit 的 `--cwd` 是这条腿的草稿目录（写在命令里）；第二个参数（冻结副本）填仓根，探针读主工作区此刻的 `scripts/agents/agent-watch.py`。
- 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0` 算跑完（`exit=3` 是完整性闸红、整趟作废）；`task-server/task log <号> --all` 里 `DEFENSE name=watchdog` 一行的 `sha256=` 要等于 `5b6030270788db16981d3e0813584cf7aef278200eeda03ca5ae9fc4ab86555c`；`DEFENSE name=done emitted=10`，`name=rerun_done probe_exit=0 probe_lines=11`。
- 逐格预期（推的，跑出来对不上的以跑出来的为准，那一条按读数改判）：line-bound `lines=75 bound_by_reading=75`；chmod-clean-at-baseline `baseline_status=[]`、`named` 含 `.claude/kb/k.md`；chmod-dirty-staged-at-baseline `named=[]`；rename-config 两行都点名 old-name 与 new-name，`status.renames=false` 那行的 `status` 是两个键（一个 `D `、一个 `A `）；persistent-failure-repeats `printed_lines_per_alert_exit=[5, 5, 5]`、`reminded_at_unchanged=True`；duplicate-reminders `both_loaded_same_state=True`、两次 printed 都大于 0；non-utf8-symlink-target `outcome='抛出 UnicodeEncodeError'`、`state_directory_entries` 里多一个 `.agent-watch-knowledge-sync.json.…tmp`；fifo-reach `ignored_listed=['.claude/singlefs-ai-sop/rules/plain.md']`、`status_keys` 里没有 only-pipe。
- 探针的内存与时间：没跑过，推的：每格一个小仓、几十条 git、毫秒级，整趟秒级，几十 MiB。
- 跑完：读完日志删运行目录 `/home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-sonnet/probe-run`（里面是带 `.git` 的小仓，完工清理闸会把它们当仓副本列）。

只读复查（不是必须，条 8 的读数用它复现，不起项目代码、只解析文件头）：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-sonnet -- python3 /home/fy5090/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r2-sonnet/identical-cell-sets.py /home/fy5090/code/singlefs/scripts/agents/agent-watch.py`，看 `identical_failure_lines` 里 repository、knowledge 起头的只有 joined 与 semicolon 一组。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，探针读数都是推的（命令在上一节）；没读禁读清单里的文件与目录。
- 没造样本、只读码推的：忙机器上命名管道那一格的 1 秒下限偶发红、`GIT_CONFIG_PARAMETERS` 与另 7 个仓内变量改读数、`core.excludesFile` 改被忽略清单。
- 没量到告警退出与定时回报的真实比例：会话记录里的字串被读码、引文污染，分不出真输出；判决「没量过」那一句仍然成立。
- 没看主 agent 那段 python 的代码（没贴出来），条 7、第 11 问里写「原子换上、新串判别无从核」是推的。
- 直接跑过的只读命令：`git ls-files`、`git status --short`、`git log`、`git rev-parse --local-env-vars`、`git check-ignore`、`du -cb`、`grep`、`awk`、`task-server/task log <号>`，以及一份只读解析文件头的 python（`identical-cell-sets.py`，草稿目录，sha256 `783469031ed45b1e584f947e492b43b3237b9ab5a2eca48580a614dc2aa63404`；它的第一版只比「知识同步提醒」格名、把没有区别的两对也报成相同，已换成现在这版并改了报告）。读数里的 `git status --short` 行数随别的会话在变（先后读到 563、566），引的是各自读到的那一刻。
- 没建仓副本与编译目录。草稿目录里只有几份文本（`scope-tracked.txt`、`identical-cell-sets.py`、`rerun-steps-body.txt`），没有要删的副本；模型目录在仓里 `research/prompts/watchdog-knowledge-sync-governance-r2-sonnet-model/`，留给主 agent 复跑。
- 别的项目怎么做：没引。同类先例都在本仓：`watch-acks.tsv` 的锁（`scripts/agents/agent-watch.py:2877`）、`root_free_report` 对 ValueError 的防护（`:1538`）、文件头里早有的几组失败行原文相同的开关（`identical-cell-sets.py`）。
