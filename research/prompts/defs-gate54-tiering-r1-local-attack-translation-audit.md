# 转述核对表：defs-gate54-tiering-r1-local-attack（2026-09-23）

逐句核对 `research/prompts/defs-gate54-tiering-r1-local-attack.md`（本地攻方：K1、K2 逐格填表）
里每一条 FACT 与中文原文。行号现查各自文件（工作区 2026-09-23，`git status --short` 显示
`.claude/agents/crash-verifier.md`、`.claude/agents/experiment-runner.md`、`.claude/main-agent.md`、
`.claude/hooks/agent-write-scope.tsv`、`.claude/gate.d/54-layer0-replay.sh`、
`.claude/gate.d/stage-inputs.tsv` 六份是工作区未提交改动，与
`research/prompts/_defs-gate54-tiering-r1-diff.md` 的 diff 逐处比对一致）。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与派发消息
「提示用英文写」「按你的定义抽样与过损坏闸」），只在这份核对表里写死来源文件与行号；
核对表与运行记录里，样本自带的行号（若模型自己写出行号）一律标「模型自给、未核」。

格式：英文项（FACT 编号）/ 原文文件:行 / 首稿缺的（核对时发现并已在定稿里补上的限定词）/ 定稿理由。

## K1（前 6 条 FACT）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| FACT 1：实验执行员写范围新增的 crates/mutations.tsv 一句 | `.claude/agents/experiment-runner.md:39`（该行内嵌括注：「入库装置的变异行（`crates/mutations.tsv` 末尾，只追加这个实验的行，不改别人的行）」） | 无遗漏：位置（末尾）、只追加本实验的行、不改别人的行三层限定全部译出 | 按字面译；「别人的行」译成「any row already present in that file」——因为这一条目下这个实验唯一会写的行就是自己新追加的行，「别人的」等价于「已存在的全部行」，语义不变 |
| FACT 2：agent-write-scope.tsv 新增一行 | `.claude/hooks/agent-write-scope.tsv:13`（整行三段：`experiment-runner`\t`crates/mutations.tsv`\t「入库装置的变异行只追加进末尾（experiment-runner.md 第 2 步与「写范围」；2026-09-23 E158 撞上：闸拒了那次追加，三行落进 research/mutations/ 下、门禁 59 号复跑不到）」） | 首稿把「门禁 59 号复跑不到」直译成「gate 59 cannot reach them」，核对时改成更贴合 FACT 5 机制的「a location that gate 59's replay does not read from」 | 改动理由：FACT 5 明确 59 号只读 `crates/mutations.tsv`，「复跑不到」的准确含义是「不在 59 号读取的范围内」，不是抽象的「够不着」；改法不丢信息，反而把两条 FACT 之间的因果钉得更死 |
| FACT 3：写范围闸只判文件级路径的机制 | `.claude/hooks/write-guard.sh:10`（注释「输入里没有 agent_type（主 agent）放行；agent_type 没有项目定义（内置 agent）放行；有项目定义而表里没有它的写范围，拒绝；目标路径规范化之后命中它的任一模式才放行。」，跨到 `:11`：「表在同目录的 agent-write-scope.tsv。拦不住 Bash 里的写，定义的「没做什么」照写。」）、代码 `decide_scope` 函数（`:75`-`:98`，尤其 `:84` 的 patterns 查找与 `:92`-`:95` 的 glob 匹配循环） | 首稿只译了「有项目定义而表里没有它的写范围，拒绝」与「按路径模式匹配、不检查内容」「不拦 Bash」三层，漏译「输入里没有 agent_type（主 agent）放行；agent_type 没有项目定义（内置 agent）放行」这两条放行分支——核对时发现这两条分支对 K1PATCHCOPY、K1BATCHRENAME 两格可能承重（若那两格里动手的是主 agent 本身或没有项目定义的内置 agent，写范围闸根本不检查），已补进定稿 | 补回理由：这两条放行分支决定了「谁在写」这件事本身是否落在闸的管辖范围内，直接影响 K1PATCHCOPY（实现员会不会是主 agent 亲自动手）与 K1BATCHRENAME（门禁修复会话是哪个 agent 身份）两格的可达性判断，属于承重限定词，不能略去 |
| FACT 4：门禁 33 号对 crates/mutations.tsv 的锚点扫描 | `.claude/gate.d/33-mutation-tables.sh:76`-`:77`（注释「crates/mutations.tsv 的锚点（六段：…）——口径与 59 号的预扫一致：原文里的 \n 还原成换行，在「文件」那一段指的源码里恰好命中一次；文件不在也算腐化。」）、代码 `:98`-`:121`（`CRATES_DUP` 两种重复判定、`:114`-`:118` 文件不存在时报 `CRATES_BAD`、`:119`-`:121` 命中次数 ≠1 报 `CRATES_BAD`） | 首稿漏译「文件不在也算腐化」这一分支（代码 `:116`-`:118` 的 `FileNotFoundError` 分支），核对时发现并已在定稿里补上（「if that file does not exist at all, or if that count is not exactly one」） | 补回理由：K1PATCHCOPY、K1BATCHRENAME 两格都可能产生「某一行指向的文件路径不再存在」这种结果（补丁合并顺序错乱、批量改名误删行），漏了这一分支会让模型误以为门禁 33 号只查命中次数、不查文件存在与否 |
| FACT 5：门禁 59 号的复跑机制 | `.claude/gate.d/59-crates-mutation-replay.sh:4`-`:7`（判据注释）、`:33`-`:43`（复用判定，`reuse_reason`/`reuse_rc`，答「可跳过」就退出码 77）、`:92`-`:106`（锚点腐化 python 检查）、`:129`-`:139`（拷贝范围之外的预检查，派活之前判）；`.claude/gate.d/33-mutation-tables.sh:11`-`:12`（「59 号整道要跑几个钟头，改 crates/ 的实现员不跑它」） | 无遗漏：三种判红条件（锚点腐化、拷贝范围之外、测试未红）、复用可跳过、耗时几个钟头、实现员不跑它，五层限定全部译出 | 「gate 33's cheap check ... exists specifically to catch the same kind of row level problem cheaply」一句是对 33-mutation-tables.sh:11「59 号开跑前的预扫查的是同一件事」的直接改写，不是外加推断，按原意保留 |
| FACT 6：两条工作流历史（PATCHCOPY、BATCHRENAME） | `research/prompts/_defs-gate54-tiering-r1-body.md:30`（「几个实现员在各自的仓副本里做、交补丁，补丁里也改 `crates/mutations.tsv`；门禁修复会话按表批量改名」） | 首稿逐字照抄这一句的密度太低，模型无法据此逐步推演——核对时把两句压缩的中文各自展开成可逐步追踪的具体历史 | 展开理由见下方「英文比原文多出来的限定词」表：两条历史都补了原文没有的操作细节，为的是让 WHATHAPPENS 这一问能被机械地逐步判断，而不是让模型自己去猜一个含糊的历史 |

## K2（后 5 条 FACT）

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| FACT 7：崩溃验证员定义第 1 条（等别的 gate.sh 跑完 54 号） | `.claude/agents/crash-verifier.md:23`（「每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号时，不起 54 号，等它结束。」） | 首稿略去「照共用约束「不做」一节」这个引用指路（只译成「following the project's shared conventions」，未点名具体是哪一节） | 有意略去：这是一处引用指路，不是行为限定词——不点名具体是「不做」一节，不改变 K2OWNRULE1 这一格要判的「等待另一个 54 号跑完」这件事本身；点名具体小节名对本轮九格判断不承重 |
| FACT 8：崩溃验证员定义第 2 条改成带 --full | `.claude/agents/crash-verifier.md:24`（变更前后对照，变更处见 `research/prompts/_defs-gate54-tiering-r1-diff.md` 对应 hunk：旧「一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`；每个记开始…」→ 新「一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号带 `--full`（不带只跑快档）；每个记开始…」） | 无遗漏：「一次只跑一个」这条改前改后都成立的限定、「不带只跑快档」（旧行为）、「带 --full」（新行为）三层都译出 | 按字面译，改前/改后两句分别对应 diff 的删除行与新增行 |
| FACT 9：共用约束「长活可以等」 | `.claude/agent-common.md:48`（整条：「长活可以等，不给它设超时、不自己中途杀掉：…用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。」）、`:49`-`:50`（等待时的记录与主 agent 判断，未译入） | 有意略去两段：（一）「不用 disown、coproc、setsid -f…」这一串具体禁用的后台脱钩手法清单；（二）`:49`-`:50` 「不起缓存计时器」「看到进程不动/报错/等的条件不会成立时的记录与升级路径」 | 两段都是「怎么正确地后台起活、等的时候记录什么」的操作细节，不改变 K2LONGWAIT 这一格要判的核心事实（长活可以等、不设超时、不中途杀、起完结束本轮、完成时才收到通知、前台命令另设 240000 毫秒上限）；FACT 9 已把这五层核心限定全部译出 |
| FACT 10：主 agent 阶段同步收尾跑 54 号 --full 那一行（K3） | `.claude/main-agent.md:43`（整行极长，本表只取与 54 号相关的那一段：「一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前」…（省略 sweep 相关步骤）…「这一批碰了 `crates/` 就后台跑 `bash .claude/gate.d/54-layer0-replay.sh --full`（层 0 全量只在这一步跑，判绿写全绿标记，整轮门禁的 54 号只核这个标记）→ 下一行」） | 有意略去该行前半段全部 `sweep` 相关步骤（写事实表、切段派 `sweep`、`stale-candidates.py` 复判等）；首稿加了一句「it does not itself rerun the full layer 0 replay」，原文没有这句显式否定，是从「整轮门禁的 54 号只核这个标记」反推出的 | 略去理由：`sweep` 那一大段与「谁在什么时候跑 54 号 --full」这件事无关，不影响 K2MAINSYNC 判断；补的那句显式否定是「只核标记」这句话按字面唯一能推出的意思（只核 = 不重放），不是新增事实，只是把隐含义显式化，避免模型误读成「54 号在整轮门禁里既核标记又顺带重放」 |
| FACT 11：门禁 54 号 --full 判红/被打断时不写标记 | `.claude/gate.d/54-layer0-replay.sh:8`（「…不问复用、不问改动范围，照跑。开跑先删旧的全绿标记，判红、被打断都不留标记；」）、`:9`（「开跑与跑完各算一次输入的内容哈希，对不上…判红。全绿才写标记…」）、`:34`（「标记那一半（相等判绿、改一个输入字节判红、没有标记判红、--full 判红不写标记）拿临时仓加一个打合成日志的假 cargo 核过…」）、代码 `:243`（`rm -f -- "${full_green_marker_path:?}"`）、`:308`-`:337`（跑完再核一次输入哈希，相等才写标记，`marker_being_written` 先写临时文件再 `mv`） | 首稿漏译「不问复用、不问改动范围，照跑」这一句（--full 与快档不同，不检查能否跳过），核对时发现并补上（「a run with the flag called full never checks whether it can be skipped or reused」） | 补回理由：K2MAINSYNC、K2REDMARKER 两格都需要「--full 每次都从头整个重跑，不会因为『上次跑过没变』就跳过」这一层限定——少了它，模型可能以为两处各自触发的 --full 调用有一次会被「复用判定」自动省掉，从而低估两次 --full 真的会各自完整重跑一遍的代价 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定/括注 | 为什么加 |
|---|---|---|---|
| FACT 6 History PATCHCOPY | `research/prompts/_defs-gate54-tiering-r1-body.md:30`「几个实现员在各自的仓副本里做、交补丁，补丁里也改 `crates/mutations.tsv`」 | 加了「each of them...appends only their own new row」「those patches are then applied one after another, in some order」两处操作细节 | 原句只给了历史的标题（谁在哪做了什么），没给到「补丁应用顺序」「各自只追加自己那一行」这两个可供机械追踪的细节；WHATHAPPENS 这一问要求「一步步追踪」，没有这两个细节模型没有抓手可以推演，只能泛泛作答；两处细节都从「补丁也改 crates/mutations.tsv」这句本身可以唯一确定地推出（既然定义要求只追加，遵守定义的实现员当然只追加自己那一行；多份补丁要合成一份文件当然要按某个顺序应用），不是凭空外加的情节 |
| FACT 6 History BATCHRENAME | `research/prompts/_defs-gate54-tiering-r1-body.md:30`「门禁修复会话按表批量改名」 | 加了「for instance, two rows gate 33 reported as sharing the same mutation name, per fact 4」这个具体触发例子，以及「not only the specific row or rows it was asked to fix」这句限定 | 「批量改名」本身没说清楚触发原因与「批量」这个词到底意味着「连带改了多少行」；补一个具体触发例子（挂钩 FACT 4 已经译出的重复变异名判红）让历史落地成可判断的具体情形；补「不只改要修的那一行」是这一格能不能打中写范围闸的关键限定——「批量」如果只批量改了一行就不构成新问题，必须显式说清楚它会连带重写别的行 |
| FACT 3 结尾两个放行分支 | `.claude/hooks/write-guard.sh:10` | 「a write made without going through any of this project's own agent definitions at all」「a write made through an agent type this project has no definition file for」 | 见上方 K1 核对表 FACT 3 那一行：这是核对时发现首稿漏译、后补回原文本来就有的限定，不是外加的新内容，放在这里是因为它在最终文件里的位置紧跟在「无一符合就拒绝」那句后面，容易被误读成「新增」，特此说明它其实是「补回」不是「新加」 |
| FACT 10 「it does not itself rerun the full layer 0 replay」 | `.claude/main-agent.md:43` | 见上方 K2 核对表 FACT 10 那一行：从「只核这个标记」显式化出来的否定句 | 同上，理由已在 K2 核对表说明，此处不重复 |

## 没做什么（本核对表）

- 未核对分给云端攻方腿的 K3、以及跨会话/跨工作区历史相关的表述——那一支材料按分工表不归本地腿，
  这份提示本身也没有引用它。
- 未判两次抽样之间答复方向是否一致——那是运行记录与主 agent 的事，不是这份核对表的事。
- 未核对 `.claude/gate.d/59-crates-mutation-replay.sh` 里与本轮九格无关的部分（分片并发、进程数计算、
  `CRATES_STRAY` 反斜杠转义检查等），因为这些机制不改变本轮任何一格的判断，按「引用要射程覆盖用得到的部分」
  的原则不强行译入，以免提示膨胀到超出模型能完整作答的篇幅。
