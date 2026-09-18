# 项目 subagent 的共用约束

`.claude/agents/` 下每个定义开工前先读这一份；与定义冲突时以定义为准。
这份与各份定义只写怎么做：步骤、约束、判据、交付。「为什么」「实测」「经过」与带日期的记录写进 `records/2026-09-16-subagent拆分提案.md` 或 `.claude/kb/`，这里不写、也不留解释的尾巴；要用到那些内容时写成一条读取指令（「开工先读：`文件`「小节」」）。门禁 71 号判这一条（规则：`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）。

## 派发

- 只由主 agent 点名派发；你手里没有 Agent 工具，不派 subagent，也不从 Bash 里起 `claude` 会话。
- 轮名、产出文件路径、草稿目录、这一轮的禁读清单都由主 agent 在派发提示里给。定义里写的路径只是形态（`<轮>`），不是实际路径。
- 输入缺一样就不开工，回复只写缺什么。

## 规则怎么读

`.claude/agents/` 下的定义都开了 `omitClaudeMd`：项目 CLAUDE.md、它 `@` 的规则、用户级 CLAUDE.md 与主 agent 的私有记忆都不进你的上下文。

- 定义「开工先读：」一行点名的规则，只读点名的那几个小节：先 `grep -n '^#' 文件` 找到小节的起止行，再按行读。点的是整份文件的，先看它的节标题，挑与这件活有关的节读。
  不整份读。
- 每个定义都照守、不再写进各自「开工先读：」一行的三处：跑命令照 `.claude/singlefs-ai-sop/rules/command-safety.md`「退不回去的操作，动手前先想一遍」「`pkill -f` / `killall` 一律禁用」两节；
  给人看的文字照 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「说人话」一节；说外部状态之前现查（`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 开头一节）。
- 本机时钟是 UTC，人在东京（JST，UTC+9）；报告里的时刻写清是哪个时区。
- 派发提示里没给、定义里也没写的项目事实（某份 kb 在哪、某条决策的原文），去仓里现查，不凭印象补。

## 写

- 只写派发提示与定义「写范围」一节给的路径。写范围闸（项目 settings 里的 hook，表在 `.claude/hooks/agent-write-scope.tsv`）只看 Write / Edit：目标不在表里你那几行模式内的，当场拒掉；被拒之后不换 Bash 或别的办法写过去，写进报告交回主 agent。Bash 里的写闸看不见，全靠你守。
- tools 里有 Write / Edit 的：手写的改动一律用 Edit（旧串必须在文件里恰好命中一次，与 `replace-once.py` 同义；这一轮自己新建的文件可以 `replace_all`），新建文件用 Write（先 `ls` 确认不存在），让闸看得见；Bash 里只许定义点名的脚本自己写（`relabel-item.py`、`kb-scribe` 的 `replace-batch.py`、门禁阶段的 `--write`、`mutate.sh`、`replay.sh`、cargo、跑产物的重定向）。草稿目录里（仓副本、日志、自己的辅助脚本）用什么写都行，那里只归你。
- tools 只有 Read / Bash 的：新建文件一律排他，`set -o noclobber` 后 `cat > 文件 <<'EOF'`，文件已存在就停下报告；之后 `>>` 追加。
- 报告分段写，每一次写进文件的内容不超过 150 行（`.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」）；表格与代码块整块放进同一段，不从中间切。
- tools 只有 Read / Bash 的，改已有文件只用定点替换：`research/scripts/replace-once.py` 或 `research/scripts/replace-batch.py`（先 `--dry-run`），不整份重写。
- 草稿放派发提示给的草稿目录，不放会话共用的暂存目录；用不上可以空着。

## 不做

- 不做 git 写操作（`add`、`commit`、`stash`、`checkout`、`restore`、`reset`、`worktree`……），不提交，不推送。门禁自带的 git 写只有 `gate-triage` 例外，见它的定义。
- 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
- 不读派发提示给的禁读清单里的文件。
- 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
- 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
  等长活时不起缓存计时器，结束本轮直接等完成通知。
  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。

## 门禁

- 哪个门禁阶段该由谁在干完自己的活之后先跑，登记在 `.claude/gate.d/stage-owners.tsv`（第二列是 agent 名，逗号分隔）。列出登记给你的：
  `awk -F'\t' -v me=<你的名字> '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv`
  逐个 `nice -n 19 bash .claude/gate.d/<文件>`，贴每个的原样末行与退出码。
- 退出码 77 是「本次未跑」，不是通过。红了先看它点名的文件与行在不在这一轮的改动里；不在的不修，照写。
- 提交前的整轮门禁归 `gate-triage`，不归你；表里没登记给你的阶段不用跑。

## 报告

- 交回与报告在不影响正确性的前提下写简练：写结论、依据与没做什么，不写做事的经过、不复述派发提示与定义；下面几条要求的命令与原样输出、行号、推翻条件照样给，嫌长就放进报告文件、交回里指过去。
- 引规则、kb、脚本、代码，写那份文件自己的行号，行号去原文件里现查，不从背景材料里数；没现查过的行号不写进报告，先写「行号待查」；行号用 `grep -n` 或 `awk 'NR==行号'` 现取，不从 `sed -n 'A,Bp'` 的输出里数偏移；贴的命令输出必须是这一次真跑出来的原样；引原文整行抄，不许摘句。
- 能用命令核的事实，贴命令与原样输出；输出不截断，嫌长先数。报告里的条数、阶段数、命中数用命令数出来，不手数。
- 结论写「什么现象会推翻它」。
- 报告末尾有「没做什么」一节：跑不动的、没验的、按定义不归你的，照实列。
- 跑出来的产物先落盘进仓里该在的位置（实验产物与重跑日志进 `research/results/`，报告、提示与判决进 `research/prompts/`），再往下做。停机条款没过、按定义不写实验页、或者主 agent 还没定要不要留的，也要把草稿产物连同「为什么没入库」写进报告——它在哪个 `/tmp` 路径、跑了什么、为什么现在不入库，主 agent 才知道它在哪、还来不来得及拷。门禁 69 号判这一条的形式：装置或变异表改了而 `research/results/` 里没有一份不比它旧的产物、kb 与这一轮新写的提示里把 `/tmp` 路径当依据引用，都红。
- 定义点名的产出文件（跑前登记、腿的 output、样本、运行记录、实验页、代码与 kb 的改动）照定义写进文件。三方论证的云端腿与核查员，报告就是派发提示给的 `research/prompts/<轮>-*-output.md`：用 Bash 分段写进去，交回内容只写文件路径、`sha256sum` 与判定一览（照 `.claude/rules/three-way-inference.md`「云端腿的报告要分段落盘」办）。其余定义的「报告」完整放进交回工具（SubagentHandback）的内容里；派发提示给了报告路径的，同一份全文先用 Bash 写进那个路径（`set -o noclobber` 排他新建，之后 `>>` 分段追加），交回末尾写路径与 `sha256sum`，主 agent 按路径存档。交回只调一次（第二次会被拒）。子 agent 用 Write 建文件名以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 `.md` 会被工具层当场拒，Bash 写的不拦。
- 干到一半收到主 agent 的消息：当成追加的输入并进这一轮做，报告里写明在哪一步收到、改了什么；与定义或原输入冲突的，停在那一处交回。
