# watchdog-knowledge-sync-governance-r1：判决（2026-11-18）

正文 `research/prompts/_watchdog-knowledge-sync-governance-r1-body.md`，开工快照 `research/prompts/watchdog-knowledge-sync-governance-r1-snapshot/`。这是这一题的第一轮（治理文件轮），没有前几轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推腿（第 1 到 4 问） | `research/prompts/watchdog-knowledge-sync-governance-r1-sonnet-output.md` | 文件头十七句现状句十二句成立、一句不成立（范围第八项点不到）、四句推不出；十七条开关全对；技能段十一句里五句有问题；登记表「另六条还没再复跑」不成立；用户要的三件里两件部分落到 |
| 攻方腿（第 5 到 11 问） | `research/prompts/watchdog-knowledge-sync-governance-r1-opus-output.md` | 第 5、6、7、10、11 问推不出（各有打中），第 8 问不成立（告警密时提醒没有上界），第 9 问读哪个仓成立、本机 git 配置那一处推不出 |

核查命令（原样）：

- `task-server/task submit --cwd ~/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-verdict -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py watchdog-knowledge-sync-governance-r1 --out ~/.local/share/singlefs/work/watchdog-knowledge-sync-governance-r1-verdict/watchdog-knowledge-sync-governance-r1-tree --repository /home/fy5090/code/singlefs`（任务 20340，退 1）：清单点名 8 份，倒推出 7 份、sha256 对上 7 份；倒推不出 1 份：`.claude/gate.d/break-switch-replay-files.tsv`，快照之后被别的会话改过别的行。腿引它的只有 `agent-watch.py` 那一行，主 agent 现读那一行（第 73 行）核过：腿引的「现数一百条」「另六条的点名改过之后还没再复跑」两句此刻逐字还在。
- `python3 scripts/agents/cite-check.py research/prompts/watchdog-knowledge-sync-governance-r1-sonnet-output.md research/prompts/watchdog-knowledge-sync-governance-r1-opus-output.md --root <上面那棵树> --background research/prompts/_watchdog-knowledge-sync-governance-r1-background.md`：「✓ 核了 168 处引文、41 处裸行号，对上 154 处，没判 55 处（其中裸行号没有引文可比 41 处；逐处列在上面）」。没判的里，判决采的那几条主 agent 逐处对着探针读数或现读的行核（见「二、判决」各条的依据）。

判决引的读数由主 agent 复跑：

- 正推腿探针：任务 20336，退 0，`name=probe_done cases=10`。
- 攻方腿探针：任务 20338，退 0，`PROBE name=done emitted=33 setup_failures=0`，`PROBE name=watchdog` 行的 sha256 以 `1c46829b` 起头、与开工快照里 `scripts/agents/agent-watch.py` 那一行相同。
- 主 agent 现查：`.gitignore` 第 8 行是 `/.claude/singlefs-ai-sop/`；`git ls-files .claude/singlefs-ai-sop/rules` 0 个；`git check-ignore -v .claude/singlefs-ai-sop/rules/sop-first.md` 命中 `.gitignore:8`。

## 二、判决

坐实 9 条（都在被判的六份文件里）：

1. **范围第八项在真仓永远点不到。** `KNOWLEDGE_SCOPE` 里的 `.claude/singlefs-ai-sop/rules/` 整个被 `.gitignore` 忽略，两处来源（`git status --short`、`git diff --name-only`）都看不见。依据：上面主 agent 现查的三条；任务 20336 `name=ignored-scope knowledge=[] count=0`；任务 20338 `PROBE name=ignored-parent reminder=unchanged named_count=0 other_count=0`。自证的小仓没有 `.gitignore`，所以自证绿。
2. **工作区里路径带空格或引号的知识类文件不点名。** `git status --short` 给这类路径加引号，前缀对不上。依据：任务 20336 `name=space-and-cjk-path knowledge=[('.claude/kb/中文名.md', '工作区改了没提交')]`、带空格的那一份落进 `others=['    新出现 [??] ".claude/kb/two words.md"']`；任务 20338 `PROBE name=quote-committed … named_count=0`（新提交里文件名带双引号的同样不点名）。
3. **本机 git 配置改得掉读数。** 依据：任务 20338 `PROBE name=global-config-untracked-hidden reminder=unchanged named_count=0`（`status.showUntrackedFiles=no` 时新建的未跟踪规则不点名）、`PROBE name=global-config-color-always reminder=unchanged named_count=0 other_count=1`（`color.status=always` 时状态行带转义、改了的知识页不点名）。`status.branch=true` 那一形照常点名（`named_count=1`），计数多不多 1 没逐字核，按线索记。
4. **继承了 `GIT_COMMON_DIR` 时查不成。** 依据：任务 20338 `PROBE name=worktree-inherited-variables … "git-common-dir-inherited": {"branch": null, "reason": "git log -1 --format=%H%x09%h%x09%s 退出码 128：fatal: bad object HEAD"}`；`GIT_DIR` 那一形去掉之后读得对。
5. **「没查成」与「余下没列」给的手动命令照抄跑不起来。** 两条命令之间是全角分号。依据：任务 20338 `PROBE name=manual-commands-verbatim … exit=129`、`PROBE name=remainder-command-verbatim … exit=128`。
6. **内容一字没变也点名。** 依据：任务 20338 `PROBE name=stage-only … named=[[".claude/kb/k.md", "工作区改了没提交"]]`、`PROBE name=chmod-only … named_count=1`、`PROBE name=reset-soft … named=[[".claude/kb/k.md", "两样都有"]]`；任务 20336 `name=stage-only` 同。只改 mtime 不点名（任务 20336 `name=mtime-only knowledge=[]`）。
7. **该提醒而不提醒。** 这一节只在一只看门狗连着一整轮没有告警时打；按告警退 3 的那一轮不打，基线随那一只没了，两只之间空档里的变动也看不到。依据：主 agent 现读 `scripts/agents/agent-watch.py:2159`、`:2658` 两处只在起来时记基线、`:1732` 只在定时回报里调；告警实际多密没量过。用户定案见「交用户的」。
8. **两条开关点名的格逐字相同、几处代码没有格盯。** `knowledgetagsource` 与 `knowledgeworktreeblind` 点名的格集合相同（两条腿各自读到同一处，正推腿报告第 2 问、攻方腿报告第 10 问）；自证里没有格的：读哪个仓的三级取法、不继承的那几个 git 变量、未跟踪目录的摘要、知识类文件的改名、HEAD 往回挪那一句、范围里被忽略的条目。这一条按两条腿读码一致采，主 agent 没另跑。
9. **文档与代码对不上的四处。** 技能段标签只列三种、漏「起来时未提交的改动已不在 git status 里」；技能段「照末尾「→」那一行」在带子 agent 的回报里末尾是重起看门狗那一行；「核手上正在改的文件」没给命令；`.claude/gate.d/break-switch-replay-files.tsv` 第 73 行「另六条的点名改过之后还没再复跑」已被任务 20085 超过（主 agent 现读那一行）。

按线索记、这一轮一并改而先证红的 2 条（探针走不到，读码成立，推的）：

10. **算内容摘要没有时限、不判文件类型。** 攻方腿说未跟踪的知识类目录里有命名管道时记基线会卡死。任务 20338 `PROBE name=fifo-in-untracked-directory hung=no`、`PROBE name=sparse-ignored-image … baseline_seconds=0.00`：探针的小仓里 `.claude/` 整个未跟踪（同一趟 `PROBE name=space-untracked-new` 的 `other=["新出现 [??] .claude/"]`），状态行是范围外的上层目录、不记摘要，装置走不到这一支，读数不说明「不卡」。`PROBE name=untracked-directory-grows named_count=0`、`PROBE name=ignored-inside-untracked named_count=0` 同一个原因、同样不采。读码：`content_digest`（`scripts/agents/agent-watch.py:1545`）对 `os.walk` 给的每个条目直接 `open`。改之前工具员先在被跟踪的 `.claude/` 底下造红样本。
11. **自证的时序窗口。** 要改仓的那几只看门狗回报周期是 0.4 分钟，机器忙时可能在改动做完之前就到点，相应的格偶发红；没量过。

不采的 4 条：

- 正推腿「最新的 git status 只部分落到，没有此刻的路径清单」：真仓此刻五百多个未提交路径，全列与清单上限相抵；这一节给计数、相对起来时的增量与看全部的命令。在「交用户的」里写明。
- 正推腿「多做 6 项」：范围表、内容摘要、基线加增量、两个选项与配置键、写 0 关闭、提醒句，都是用户补充「知识同步提醒」之后那一批的内容或它的开关，不算多做；范围第八项的毛病归第 1 条。
- 正推腿「`AGENT_WATCH_GIT_TIMEOUT_SECONDS` 设成非数字时抛 `ValueError`」：这个变量只给自证用，不改。
- 攻方腿「行宽封不住」：单行长度由路径与提交标题定，不设上限。

核不动的：任务 20338 `PROBE name=scenario_line_bound exception=TypeError`，是探针自己调错了参数，「上限 20 时最多 70 行」这一句没量到，按腿读码的线索记。

第一轮有打中的，坐实 9 条。照治理文件轮的收法：改完再攻第二轮，攻这一轮的修补与上面写明按线索记的地方，第二轮派一条辩方腿复核这一份判决。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 量过 / 零轮 |
|---|---|---|---|
| `git status` 改用 `--porcelain=v1 -z --untracked-files=normal`，`git diff --name-only` 加 `-z`，按 NUL 切、改名按 porcelain 的两段读 | 第 2、3 条 | 攻方腿 | 零轮：任务 20338 的 `quote-committed`、`global-config-untracked-hidden`、`global-config-color-always` 与任务 20336 的 `space-and-cjk-path` 当红样本 |
| 范围里被 git 忽略的条目不经 git、按内容摘要直接比（起来时逐个文件记摘要，到点点名变了的那几份），标签另起一种 | 第 1 条 | 攻方腿与主 agent | 零轮：自证小仓加 `.gitignore` 忽略那个目录当红样本 |
| 不继承的 git 变量加 `GIT_COMMON_DIR`、`GIT_OBJECT_DIRECTORY`、`GIT_ALTERNATE_OBJECT_DIRECTORIES`、`GIT_NAMESPACE`、`GIT_PREFIX` | 第 4 条 | 主 agent | 零轮：任务 20338 `worktree-inherited-variables` 当红样本 |
| 手动命令拆成两行各一条，写成 `git -c core.quotepath=off -C <仓> …`；「核手上的文件」给 `python3 scripts/agents/who-touched.py <路径>…` | 第 5、9 条 | 攻方腿 | 零轮：照抄交 `bash -c` 退 0 当样本 |
| 起来时有摘要的路径，摘要没变就不点名（状态码变了也不点） | 第 6 条的只暂存、只改执行位两形 | 攻方腿 | 零轮；`reset --soft` 那一形不改，按线索记 |
| 基线与上次提醒的时刻落盘到会话目录，下一只看门狗接着比；不论告警退出还是定时回报，离上次提醒满 `--max-minutes` 就带这一节，打过就把基线换成此刻 | 第 7 条 | 攻方腿提、用户定 | 零轮：自证加「告警退出且离上次提醒已满」「告警退出而没满」「两只之间的空档里变了」三格 |
| 算摘要只读普通文件、整次摘要带时限，超时走「没查成」 | 第 10 条 | 攻方腿 | 零轮：先在被跟踪的 `.claude/` 底下造命名管道的红样本，红了再改 |
| 自证补格：三级取法、不继承的变量、未跟踪目录的摘要、知识类文件改名、HEAD 往回挪；两条开关的点名分开；要改仓的那几只按就绪文件之后再起算，不靠固定窗口 | 第 8、11 条 | 两条腿 | 零轮 |
| 文件头、技能段、登记表那一行照改完的行为重写（标签种数、「→」指哪一行、登记表去掉「另六条还没再复跑」） | 第 9 条 | 正推腿 | 不涉及行为 |

这一批都是治理文件，没有要写进产品代码的零轮改法。每条零轮改法的样本先证红再换上。

## 交用户的

| 事 | 状态 |
|---|---|
| 告警密时提醒不出怎么改 | 用户已定（2026-11-18 弹窗）：「记到文件、满一小时就出」 |
| 这一节不列此刻 `git status` 的全部路径，只给计数、相对起来时的增量与看全部的命令 | 告知，不改 |
| 没派过子 agent 的对话收不到 | 用户早先定的范围，不改 |

## 这一轮被判过的文件

- research/prompts/_watchdog-knowledge-sync-governance-r1-body.md：判了它「实现今天的样子」各句与代码对不对得上，两条腿照它的十一问作答
- scripts/agents/agent-watch.py：判了文件头「知识同步提醒」一段、十七条开关、`run_git` 到 `print_repository_section` 各函数、两处调用点与自证 `selftest_repository_report`；坐实第 1 到 8 条，第 10、11 条按线索记
- scripts/agents/watch.sh：判了第 76、216 行与代码一致，成立
- scripts/agents/watch.conf：判了 `repository-report-lines` 两行注释四句，成立
- .claude/skills/dispatch/SKILL.md：判了「知识同步提醒」那一段十一句；坐实第 9 条里的三处
- .claude/hooks/selftest-registry.tsv：判了 `agent-watch.py` 那一行的说明与自证起的进程一致，成立
- .claude/gate.d/break-switch-replay-files.tsv：判了 `agent-watch.py` 那一行；条数与四族成立，「另六条还没再复跑」一句过时（第 9 条）

## 回看决策

不涉及决策：这一轮判的是看门狗脚本与它的配置、技能、登记表，没有引到任何一条决策分项。
