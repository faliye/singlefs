# 转述核对表：defs-m2-closeout-r1-local-attack（2026-09-26）

逐句核对 `research/prompts/defs-m2-closeout-r1-local-attack.md`（本地攻方 D2：
按事实表把 `agent-common.md`「执行前拒绝的写法」列的每一种写法与探针日志逐格核对，
Q1–Q12）里每一条 Exhibit A（C 码）与 Exhibit B（P 码）跟中文原文的对照。行号现查：
`agent-common.md` 用工作区当前文件（`git status --short` 显示这一轮它是工作区未提交
改动之一，但「执行前拒绝的写法」那十行——59-70 行——不在这一轮 D1/D2/D3 被判的改动
范围内，是共用约束里早就有的既有内容，本轮 diff 未动这十行，`git diff HEAD -- .claude/agent-common.md`
核过，59-70 行不在任何一个 hunk 里）；探针日志用 `research/prompts/_defs-m2-closeout-r1-diff.md`
（这一轮发给我的背景材料本身，不是 `/tmp` 草稿）。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与派发消息
「提示用英文写」），只在这份核对表里写死来源文件与行号；核对表与运行记录里，样本自带的
行号（若模型自己写出行号）一律标「模型自给、未核」。

格式：英文项（C/P 码范围）/ 原文文件:行 / 首稿缺的 / 定稿理由。核对粒度按「同一句中文」
分组——C 组与 P 组各自的成员码大多来自原文同一句里并列的几个逗号分隔项或探针日志里同一个
中文标签簇，逐字拆分成表格里的独立行不算另一次转述，核对表按句子为单位核，不逐码重复核。

## 表一：Exhibit A，C1–C6（源文件 `.claude/agent-common.md`）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| C1.1–C1.6（① 看门狗错误写法） | `.claude/agent-common.md:61`（「①起看门狗的错误写法：research/scripts/watch.sh、research/scripts/agent-watch.py watch 不用 run_in_background 起，或命令里带单独的 &、nohup、disown、setsid、把输出丢进 /dev/null。看门狗只有主agent起。」） | 无遗漏：六种错误写法（不用 run_in_background、单独 &、nohup、disown、setsid、输出丢 /dev/null）全部逐条译出；「看门狗只有主 agent 起」这一句不是写法，是「谁能起」，未译入 C 项（不属于本轮「写法」逐格核的射程），只留在提示 Q1 前的组说明句里提了一句「起看门狗」是谁的事，未展开成单独条目 | 六项按原文顿号并列直译，未增未减 |
| C2.1（② 无超时等待循环） | `.claude/agent-common.md:62`（「②前台没有超时的等待循环：until / while 里有 sleep，外面没套 timeout。要等就用 run_in_background 起、结束本轮等完成通知，或 python3 proc.py wait <pid> --timeout <秒>。」） | 首稿只译了违规写法本身（until/while 里有 sleep、外面没套 timeout），两条替代做法（run_in_background 或 proc.py wait --timeout）未译入 C 项——核对时认为这两条是「怎么正确地等」的操作指引，不是「这一种写法」本身，与 C1 对「谁能起」的处理一致，故意不展开成单独 C 条目，只在 C2.1 里保留「没套 timeout」这个判据本身 | 有意略去：两条替代做法不影响 C2.1/C2.2 这一组要判的核心事实（有没有套 timeout），展开成条目反而分散 Q2 的两行判断 |
| C3.1–C3.6（③ 把活放出追踪） | `.claude/agent-common.md:63`（「③把活放出追踪：disown、coproc、setsid -f、nohup … &、tmux / screen 的分离模式、不等到结束的 systemd-run。」） | 无遗漏：六种写法（disown、coproc、setsid -f、nohup…&、tmux/screen 分离模式、systemd-run 不等到结束）全部逐条译出 | 按原文顿号并列直译 |
| C4.1（④ 后台内作业无 wait） | `.claude/agent-common.md:64`（「④run_in_background 里有作业以单独的 & 收尾，之后同一条命令里没有 wait。」） | 无遗漏：唯一一种写法（后台里作业单独 & 收尾、之后同一条命令没有 wait）完整译出 | 按字面直译 |
| C5.1–C5.9（⑤ 覆盖未跟踪产物的九种写法） | `.claude/agent-common.md:65`（「⑤整份覆盖 research/results/ 下已存在又没进 git 的产物：>、>\|、&>、不带 -a 的 tee、cp / mv / install 的目标、dd of=、truncate。追加（>>、tee -a）与新文件名不拦；要换就按日期另存新文件名，旧的留着。」） | 无遗漏：九种覆盖写法（>、>\|、&>、tee 不带 -a、cp、mv、install、dd of=、truncate）全部逐条译出；三条不拦的例外（>>、tee -a、新文件名）也全部译出（见 C5.10–C5.12） | 按字面直译，cp/mv/install 原文顿号并列成一句，拆成三条各自独立的 C 项，不改变各自的判据条件（「…的目标」这个限定词三条都保留） |
| C6.1–C6.18（⑥ 终止进程的十八种错误写法） | `.claude/agent-common.md:66`（「⑥终止进程不是点名一个自己起的进程号或任务号：kill 的目标带负号、是 0 或 $PPID、一次给几个、是命令替换或通配，在循环里逐个发（proc.py stop 同样），按名字、按 cgroup 或 /proc 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，systemctl 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：kill "$!"、kill %1、单独一条 python3 proc.py stop <pid>。」） | 无遗漏：负号、0、$PPID、一次给几个、命令替换、通配、循环里 kill、循环里 proc.py stop、按名字挑、按 cgroup 挑、按 /proc 挑、指到自己会话祖先、指到别的会话进程、指到 SSH/VSCode/本地模型服务（拆成一条）、systemctl 停 ssh、systemctl 停登录会话、systemctl 停本地模型服务（原文「SSH、登录会话、本地模型服务这类单元」一句拆成三条独立 C 项）、关机重启，十八条全部逐条译出；三条放行例外见 C6.19–C6.21 | 「SSH / VSCode / 本地模型服务」在「写死的进程号指到…」这半句里拆成一条 C6.14（三者算一种写法的三个目标，未再拆三条，因为原句本身把三者并列在同一个「写死的进程号指到」判据下，拆分过细会跟 C6.15–C6.17 的 systemctl 三条重复判据结构）；systemctl 那半句因为原文明确列了三类不同的「单元」，且探针只测了 ssh 一类，故拆成三条独立 C 项以便和探针逐条比对 |

## 表二：Exhibit A，C7–C10（源文件 `.claude/agent-common.md`）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| C7.1–C7.8（⑦ 改已有脚本的八种写法） | `.claude/agent-common.md:67`（「⑦在同一个 inode 上改已经存在的脚本（.sh、.py 或带执行位的文件，在仓里或 /tmp/claude-1000/ 下）：>、>\|、&>、不带 -a 的 tee、cp 的目标、dd of=、truncate、python 的 open(…, 'w') 这一类。改脚本写到同目录临时文件再 mv 换上，或用 research/scripts/replace-once.py / insert-row.py 定点改；追加与新建不拦。」） | 无遗漏：八种写法（>、>\|、&>、tee 不带 -a、cp 目标、dd of=、truncate、python open(w)）与两条例外（追加、新建）全部译出；「或用 replace-once.py / insert-row.py 定点改」这一条正确的改法未单独译成 C 项——它是「怎么正确改」的操作指引，不是「不拦的写法」本身，与 C1、C2 的处理口径一致 | 有意略去 replace-once.py / insert-row.py 这条正确做法：它和 C7.11（temp file + mv）性质相同、都是「怎么正确地改」，选一条最直接对应探针（探针只测了 mv 换上这一种正确做法，没测 replace-once.py），另一条不展开成 C 项避免探针查无对应时产生假的「未测」噪音 |
| C8.1–C8.3（pattern-process-guard 三种写法） | `.claude/agent-common.md:68`（「上游的 .claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh：命令位置上的 pgrep -f、pkill -f、killall。」） | 无遗漏：三种写法（pgrep -f、pkill -f、killall）与「命令位置上的」这个限定词全部译出 | 按字面直译，「命令位置上的」限定词逐字保留在 C8.1–C8.3 每一条里（写成「in command position」） |
| C9.1–C9.2（heavy-test-guard 两大类） | `.claude/agent-common.md:69`（「重型测试闸（.claude/hooks/heavy-test-guard.sh）：越出重型测试那一条的命令（主agent不带 SINGLEFS_HEAVY_TESTS前缀也拒）；子agent不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。」） | 无遗漏：两大类（越出重型测试范围的命令、主agent不带前缀、子agent不经内存包装跑编译代码）全部译出 | 按字面直译；「「跑编译出来的代码经内存包装」那一条」是原句自带的指路括注，指向的具体内容（cargo build/clippy/fmt 不要求经包装）不在本条原句里，转到 C9.5 单独处理，见下表 |
| C10.1–C10.3（write-guard 三种写法） | `.claude/agent-common.md:70`（「写闸（.claude/hooks/write-guard.sh，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（.claude/hooks/agent-write-scope.tsv）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 .claude/rules/path-moves.md「变体起新名字，不用角标」）。」） | 无遗漏：三种写法（Write 整份覆盖未跟踪已有文件、子agent写到写范围外、内容含撇号类角标）与「只看 Write / Edit 工具」这个射程限定全部译出；角标字符具体是哪几个（U+2032 等）与「变体起新名字」那条指路未展开译入 C10.3，只写「几种这一类的角标字符之一」 | 略去角标具体字符编号与「变体起名」那条指路：C10.3 只需要判「含不含这一类角标」这一件事，具体是哪个码点不影响 Q10 的匹配判断，展开成逐字符列举会让这一条变成好几条，而探针（P41、P42）本身也只各测了一个字符，没有穷举 |

## 表三：Exhibit B，P1–P29（探针原样输出，源文件 `research/prompts/_defs-m2-closeout-r1-diff.md`）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| P1–P3（① 看门狗三条探针） | `_defs-m2-closeout-r1-diff.md:706-708`（「① 看门狗前台起 exit=2」「① 看门狗后台加 & exit=2」「① 看门狗后台正确起（对照放行）exit=0」三行，含各自 stderr 原文） | 无遗漏：三条探针（前台起、后台加 &、正确起对照）各自的调用形态与 exit 码全部译出 | 按字面直译；「认出的调用：watch.sh abc123」译成「watch.sh followed by an argument」，因为 abc123 只是探针喂的占位参数，不是要判断的写法本身 |
| P4–P5（② 等待循环两条探针） | `:709-710`（「② 前台无超时等待循环 exit=2」「② 外套 timeout（对照放行）exit=0」） | 无遗漏 | 按字面直译，「认出的循环：until grep -q x log」保留成具体例子 |
| P6–P8（③ 三条探针） | `:711-713`（「③ disown exit=2」「③ nohup … & exit=2」「③ setsid -f exit=2」） | 无遗漏 | 按字面直译 |
| P9–P10（④ 两条探针） | `:714-715`（「④ 后台里单独 & 无 wait exit=2」「④ a & b & wait（对照放行）exit=0」） | 无遗漏 | 按字面直译，认出的作业「sleep 100」保留成具体例子 |
| P11–P20（⑤ 十条探针） | `:716-725`（九种覆盖写法各一条 exit=2，加 `>>` 对照放行 exit=0，共十行；目标文件原样是 `research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out`） | 无遗漏：九种覆盖写法与一条对照全部译出 | P11 起把目标文件具体名字统一简化译成「a file that already exists under this project's results directory and is not tracked by version control」，P12 起用「the same file」承接，不重复整个文件名——这是转述压缩，不丢判据（判据是「已存在且未跟踪」这个属性，不是那个具体文件名） |
| P21–P29（⑥ 九条探针） | `:726-734`（kill 负号、kill 0、kill 两个目标、kill 命令替换、循环里 kill、proc.py stop 循环、systemctl stop ssh 各一条 exit=2，加 kill "$!"、proc.py stop 单个两条对照放行 exit=0，共九行） | 无遗漏：七种违规写法与两条对照全部译出 | systemctl 那条探针原文只测了 `systemctl stop ssh` 这一个具体单元，未把 stderr 括注里提到的「登录会话、本地模型服务」也当成被测过的例子——P27 译文只写 ssh，不替探针没测过的部分补例子，避免让模型误以为登录会话与本地模型服务两种也被测过 |

## 表四：Exhibit B，P30–P64（探针原样输出，源文件同上）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| P30–P36（⑦ 七条探针） | `:735-741`（>、cp、tee、python open(w) 四种改脚本写法各一条 exit=2，加 mv 换上、>> 追加两条对照放行 exit=0，再加「旁：sed -i 改脚本」exit=0，共七行；目标脚本原样是 `research/scripts/replay.sh`） | 无遗漏：四种违规写法、两条例外对照、一条「旁」（sed -i，不在 C7 清单里的写法）全部译出 | 「旁：sed -i 改脚本」保留成独立 P36，且不给它配一个对应的 C 项——它本来就不在清单 C7.1–C7.11 任何一条里，译文里如实标成「a form not named in clause 7's list at all」，让 Q7/Q12 的模型自己判断这条不该被强行配对 |
| P37–P42（write-guard 六条探针） | `:742-747`（Write 覆盖未跟踪已有文件、子agent越出写范围、未登记的项目agent、Edit 写进撇号角标、Write 写进撇号角标各一条 exit=2，加 Write 新文件一条对照放行 exit=0，共六行） | 首稿把 P40「未登记的项目 agent」直接当成 C10.2（写到写范围外）的一个例子译入同一条；核对时发现原文「gate-triage 在写范围表里没有登记，按拒绝处理」讲的是「表里根本没有这个 agent 的行」，跟 C10.2「有登记但写到自己那几行之外」字面上是两种不同的情形，改成独立的 P40 描述（「has no row at all registered」），不预先替模型判定它算不算 C10.2 的例子，留给 Q10/Q12 自己判断 | 拆分理由：这正是这一轮 D2 攻击面要抓的那类「清单漏列而探针是拒」的候选，若我在核对表或提示里就先替它配好对应的 C 项，等于替模型做了这一格的推理，违反「不替它补推理」 |
| P43–P46（pattern-process-guard 四条探针） | `:748-751`（pgrep -f foo、pkill -f foo、killall foo 各一条 exit=2，加 grep -n pgrep x.sh 一条对照放行 exit=0） | 无遗漏 | 按字面直译；「grep -n pgrep x.sh」译文加了一句「pgrep 只作为 grep 的搜索参数出现、不是被运行的那个命令」——这句解释是从「命令位置上的」这个判据（C8 组说明句已有）直接反推出来的，不是外加信息 |
| P47–P64（heavy-test-guard 十八条探针） | `:752-769`（crash-verifier、gate-triage、implementation-writer、experiment-runner 四个 agent 各自跑不同命令的十八行，逐行 agent 名、命令、exit 码见提示原文 P47–P64） | 无遗漏：十八条命令、各自的 agent 身份、是否带 SINGLEFS_HEAVY_TESTS 前缀、是否套 8G 内存包装、exit 码全部逐条译出 | 「8G」在提示译文里保留成具体数值（原文命令行里的 `run-with-memory-cap.sh 8G`），未抽象成「a memory cap」，因为 C9.4/C9.5 判据只关心「有没有经过包装」这件事本身，具体数值不影响匹配判断，但保留具体数值方便模型核对同一条命令在不同探针行之间是不是「同一个包装」 |

## 英文比原文多出来的限定词、条目（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的内容 | 为什么加 |
|---|---|---|---|
| C1.7（正确起看门狗，允许） | `.claude/agent-common.md:61` | 整条 C1.7 都是加的：原文①这一句只列了六种错误写法，没有反过来说「用 run_in_background 正确起、不带 & / nohup / disown / setsid、不丢 /dev/null 就允许」 | 加的理由：这是六种错误写法各自否定之后的直接逻辑补集，不引入①原文之外的新事实；加它是为了让 Q1 能核对探针 P3（对照放行）——没有这一条，P3 就没有对应的 C 项可核，Q12 的「漏列」检查也无法覆盖 P3 这类对照放行探针 |
| C2.2（等待循环套了 timeout，允许） | `.claude/agent-common.md:62` | 整条 C2.2：原文②只说「外面没套 timeout」是违规，没有反过来明说「套了就允许」 | 同上，直接逻辑补集，用于核对探针 P5 |
| C4.2（a & b & wait 模式，允许） | `.claude/agent-common.md:64` | 整条 C4.2：原文④只说「单独 & 收尾之后没有 wait」违规，没有反过来明说「之后有 wait 就允许」；补充的具体形态「两个作业各自 & 收尾、之后一个不带参数的 wait」照抄探针 P10 的具体写法，不是原文④本身给出的例子 | 加的理由同上；具体形态照抄 P10 是为了让 C4.2 与 P10 精确对应，不产生「大致类似但细节不同」的模糊匹配 |
| C8.4（非命令位置出现，允许） | `.claude/agent-common.md:68` | 整条 C8.4：原文只说「命令位置上的」三种名字触发，没有反过来明说「不在命令位置就允许」 | 「命令位置上的」这个限定词本身就是原文明写的判据边界，C8.4 只是把这同一个限定词的另一面显式化，不是引入新信息；用于核对探针 P46 |

| 英文项 | 原文文件:行 | 多出来的内容 | 为什么加 |
|---|---|---|---|
| C9.3（子agent跑自己分内的重型阶段并带前缀，允许） | `.claude/agent-common.md:69` | 整条 C9.3：原文只说「越出重型测试那一条的命令…拒」，没有反过来明说「没越出、而且带前缀就允许」 | 直接逻辑补集；用于核对探针 P47、P49、P50、P51、P53、P54 这几条「带前缀、在分内」的对照 |
| C9.4（子agent经内存包装跑编译代码，允许） | `.claude/agent-common.md:69` | 整条 C9.4：原文只说「子agent不经内存包装跑编译出来的代码」违规，没有反过来明说「经了就允许」 | 直接逻辑补集；用于核对探针 P48、P56、P58、P62、P63、P64 |
| C9.5（cargo build / clippy / fmt 不要求经包装） | 主体内容实际来自 `.claude/agent-common.md:48`（「跑编译出来的代码经内存包装」那一条正文：「…cargo build、cargo clippy、cargo fmt 不跑编出来的代码，不要求经它。」），由 `:69` 的括注「「跑编译出来的代码经内存包装」那一条」指过去 | C9.5 整条内容不在 :69 这一行字面里，是顺着 :69 自带的指路括注去 :48 取来的 | 这是 rules 允许的「指路不写数」的反向操作：:69 明确点名了「那一条」，我沿着这条指路去取内容，而不是在 :69 原地编一个新事实；取来的内容与 :69 描述的同一件事（内存包装规则）确实相关，用于核对探针 P59、P60（cargo build、cargo clippy 直接跑，未包装，放行） |
| C10.4（Write 新建文件，允许） | `.claude/agent-common.md:70` | 整条 C10.4：原文只说「Write 整份覆盖仓里已存在又没进 git 的文件」违规，没有反过来明说「建新文件就允许」 | 直接逻辑补集；用于核对探针 P38 |

## 没做什么（本核对表）

- 未核对分给云端攻方腿（Opus，D1/D2/D3/D4）与云端正推腿（Sonnet，D1/D2/D3）的材料与判词——那两支材料按分工表不归本地腿，这份提示本身也没有引用它们。
- 未判多次抽样之间 Q1–Q12 的具体答案方向是否一致——那是运行记录与主 agent 的事，不是这份核对表的事。
- 未核对 `.claude/agent-common.md` 里「执行前拒绝的写法」（59-70 行）之外的其余内容（例如「不做」一节 48 行「跑编译出来的代码经内存包装」那一条的其余分句、「派发」「写」「门禁」「报告」几节）——只有 :69 显式指路取用的 C9.5 那一句例外，理由见上表。
- 未核对四份被探针喂过的 hook 脚本（`bash-command-detector.sh`、`write-guard.sh`、`pattern-process-guard.sh`、`heavy-test-guard.sh`）自己的源码逻辑，只核对了它们各自那一天对具体探针输入产生的 exit 码原样记录——本轮任务是清单与探针记录的逐格核对，不是清单与源码逻辑的核对。
