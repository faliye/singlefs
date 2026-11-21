# wrapped-writers-loop-paths-cite-titles-governance-r3：判决（2026-11-21）

正文 `research/prompts/_wrapped-writers-loop-paths-cite-titles-governance-r3-body.md`，开工快照 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-snapshot/`。这是这一题的第三轮（治理文件轮，最后一轮），攻第二轮判决 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-main-verification.md` 的修补；第一轮判决是 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-main-verification.md`。这一轮之后不开第四轮，改法都标零轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-opus-output.md` | 第 1 问成立（打中）：五类八形平常写法被连带降成算不出（多报），env 合写的 `-C`、行首反引号里的 cd、`<< -EOF` 仍算出错目录（漏报）；第 2 问成立（轻，偏严）：退 3、退 4 归别的一趟时多报，「写开始行之前不报」没漏；第 3 问不成立，只有「真换先占这个仓」一处字面会被读反；第 4 问成立（打中）：标题本身带反引号的小节写法、位置与引文隔一个「，」的小节写法掉出去既不判也不列，不带反引号的散文被误红，去括号没打中 |
| 正推（Sonnet） | `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet-output.md` | 第 5 问「实现今天的样子」逐条成立，推不出的只有各趟自证与复跑的结局、服务重启与真任务上那一次提醒；第 6 问三形算错当前目录在改前读码成立、正常结束的换钩子被点名在改前读码成立，辩方七条补充探针能只读核的都核掉；旁见：不接管道的花括号组今天被降成算不出 |

核查：

- 开工快照：`sha256sum -c research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-snapshot/judged-sha256.txt` 14 份全对得上，腿跑着的时候被判的文件没人改过，不用倒推。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-opus-output.md research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet-output.md --background research/prompts/_wrapped-writers-loop-paths-cite-titles-governance-r3-body.md` 退 0，末行「✓ 核了 106 处引文、44 处裸行号，对上 106 处，没判 44 处（其中裸行号没有引文可比 44 处；逐处列在上面）」。
- 主 agent 复跑（腿的探针拷进 `~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-main-rerun/`，在副本里交；四条都是 type=succeeded exit=0）：
  - 改前三形直接读数（正推腿第 6 问）：`task-server/task submit --cwd <复跑目录> -- python3 -I <复跑目录>/probe/runs_probe.py <改前那一份 task_server_scheduler_building.py> <复跑目录>/probe/wrapper_words_source.txt <复跑目录>/probe-root`，打补丁那一条的目录依次是 `nested`、`nested`、`.`，与正推腿读码的相同。
  - 引文核对改前改后对照（正推腿）：`task-server/task submit --cwd <复跑目录> -- bash <复跑目录>/tools/run-cite-compare-r2fix.sh <仓根> <改前那一份 cite-check.py> <复跑目录>`，末行「done 报告 294 份 改前认出 15105 处 改后认出 15031 处 结局不同 74 处」；对照输出里「小节标题认不出结尾」67 行，其中 65 行改后「没认出」，这 65 行取到的标题里都带一个收尾的反引号（`grep -c` 读数 65）：写法是带反引号的小节位置后面接几个连接字再接引文。正推腿把它们写成散文，读数不支持。
  - 攻方第 1 问探针：`task-server/task submit --cwd <复跑目录>/opus -- bash <复跑目录>/opus/tools/run-runs-probe.sh <复跑目录>/opus`，改后与改前各 13 段，逐行与攻方腿写的期望相同：1a 八形改后 `directory=None`（赋值那一形是 `patch=None`）、改前 `nested`；1b 四形改前改后都是 `directory` 与 `real` 不同；`shell_directory_option(['env', '-Cnested', 'bash'])=False`。
  - 攻方第 4 问对照：`task-server/task submit --cwd <复跑目录>/opus -- bash <复跑目录>/opus/tools/run-cite-compare-r3.sh <复跑目录>/opus`，两行「done 报告 1 份 改前认出 11 处 改后认出 6 处 结局不同 8 处」「done 报告 295 份 改前认出 15112 处 改后认出 15038 处 结局不同 74 处」；探针串第 6 到 10 行改前 `skip`、改后「没认出」，第 4 行改前 `skip`、改后 `bad`，第 13、14 行改前 `bad`、改后 `ok`，与攻方腿的期望相同。
- 核不动、照「推的」办的：攻方第 2 问的多报两种（结局文件里记的是更早一趟或别的任务的那一趟）没有样本、没跑；反引号那一形 bash 真跑的目录是腿照语义推的；判的那一刻目录在、跑的那一刻不在那一形没进探针。

## 二、判决

1. 攻方第 1 问：坐实而采（探针量到）。第二轮改法 2 把三形降成「算不出」时连带了平常写法（heredoc 正文末行是空行或注释行、正文里单独一行的花括号、花括号组之后的下一条、不接管道的花括号组），这是那一条改法带进的多报；env 合写的 `-C`、行首反引号、`<< -EOF` 三形算出错目录是漏报，其中 env 合写是改法 2 第三形没罩到的同类。正推腿的旁见与它同一件。接管道的组里自己的写工具与赋值两形只多报、改动大，不改，记在「交用户的」。
2. 攻方第 2 问：「写开始行之前不报」成立（两条腿各自读 `scripts/gate/hook_swap_swap.py` 的写序得出同一结论：开始行连同文件行一次原子写、排在换第一份之前）。退 3、退 4 归别的一趟时多报，照腿判为偏严、没跑出读数；按开始时刻认归属的改法认错了会漏报，方向比多报坏，不改。自证没钉的两处（退 1 没有样本、`cd` 之后写相对 `--scratch` 没有样本）采，补样本。
3. 攻方第 3 问：不成立，照腿判；「真换先占这个仓」字面与 `scripts/gate/hook-swap.py` 文件头写的先后相反，改字面。
4. 攻方第 4 问：坐实而采（对照量到）。第二轮改法 4「兜底不收散文」把带反引号的小节位置一并丢了：位置后面接连接字再接引文的 65 处、标题本身带反引号的、位置与引文隔「，」的，改前列进没判，改后既不判也不列；同形的行号写法照列进裸行号，两种位置写法不一致。不带反引号、位置后接散文的被判红，采为误红。去括号一支没打中，照留。
5. 正推第 5、6 问：逐条成立照腿判；它推不出的各趟结局由调度记录里主 agent 自己交的那几趟顶着，不靠腿。第 6 问「三形在改前真算错」由主 agent 复跑坐实，第二轮判决那一条的「推的」改成「量到」。

## 三、改法

都是治理文件，不进产品代码，不做 A/B；这一轮是最后一轮，改完不再攻，全部标零轮，换上后以自证与开关复跑为准。

1. 任务服务算当前目录（多报那一侧）：读到 heredoc 定界词那一行时清掉正文留下的记号；正文的行不记花括号；花括号组不接管道与 `&` 的照这一层算（组里的 cd 留得住），组收尾之后的下一条照平铺看。攻方探针的 heredoc-blank-last-line、heredoc-comment-last-line、python-heredoc-dict、json-heredoc-two-objects、line-after-brace-group、brace-group-plain 六形改后算出 `nested`；第二轮的三形照旧算不出。零轮。
2. 任务服务算当前目录（漏报那一侧）：env 换目录的选项认合写（`-Cnested`、`-vC nested`），串里与命令那一层同改；命令位置上带反引号的词、`<<` 后面隔空白接以 `-` 起头的词，之后整段算不出。env-attached-chdir、env-grouped-chdir、backtick-cd-at-command-position、heredoc-space-dash-delimiter 四形与命令那一层一形各进自证，改前红。零轮。
3. 任务服务自证：`hook-swap-outcome-judged` 补退 1 的样本与 `cd` 之后写相对 `--scratch` 的样本，各配一条开关。零轮。
4. 「真换先占这个仓」改成「真换动手之前另占这个仓」，服务源码几处、异常表那一行与 README 三语第 174 行同改。零轮。
5. 引文核对：带反引号的小节位置（反引号收尾）没有紧挨引文的，照裸行号那样列进没判，不丢；`@` 后面的标题本身带反引号的，照那份文件里的整行标题认（认得出就照常判，认不出列进没判）；位置与引文隔「，」的列进没判；不带反引号、取到的标题里有空白而整串命中 0 次的列进没判、不判红。攻方探针串第 6 到 10 行与第 12 行、对照里那 65 处的写法进自证样本。零轮。

## 交用户的

无要定的。留着没改的三样（都只多报、不漏报）：接管道的花括号组里自己的写工具与赋值算不出；换钩子的结局文件里记的是更早一趟或别的任务的一趟时照报这一条；同一条命令两条换钩子用同一个 `--scratch` 时两条都点名。

## 这一轮被判过的文件

- .claude/hooks/bash-command-detector.sh：判了第 224 行那一处改字，对改前逐行比只这一处，成立
- scripts/agents/cite-check.py：判了带反引号判红、逐层去括号与兜底三种理由；带反引号的小节位置掉出去既不判也不列、不带反引号的散文误红（坐实，改法 5），去括号成立
- scripts/mutation/mutate.sh：判了文件头「并行」那一段，对改前逐行比只这一处，显式值合计对码成立
- task-server/README.en.md：判了第 174 行与中文同一个条件，成立；「先占」字面同改法 4
- task-server/README.ja.md：判了第 174 行与中文同一个条件，成立；「先占」字面同改法 4
- task-server/README.md：判了第 174 行退 10 出路对四种组合成立；「真换先占这个仓」字面与代码先后相反（改法 4）
- task-server/task_server_lib_constants.py：判了异常表那一行与提醒合起来是当且仅当、五条开关名已登记，成立；「先占」字面同改法 4
- task-server/task_server_lib.py：判了开关表行数与文件头现状句，成立
- task-server/task_server.py：判了文件头退 10 那一句，成立；「先占」字面同改法 4
- task-server/task_server_scheduler_building.py：判了 shell_text_pieces 与 shell_text_runs；连带降成算不出的六形与算出错目录的四形（坐实，改法 1、2），正文点名的四种成立
- task-server/task_server_scheduler_lifecycle.py：判了 hook_swap_read_back、workspace_write_judged 与退 10 的新句；写开始行之前不报成立，四种组合成立，归别的一趟时多报（偏严，照留）
- task-server/task_server_selftest_memory.py：判了两格新格与五条开关各红在名字说的那件事，成立；缺退 1 与相对草稿目录两个样本（改法 3）
- task-server/task_server_selftest_notifying.py：判了退 10 样本钉住新句，成立；「先占」字面同改法 4
- task-server/task_server_selftest.py：判了两格登记与现数 455，成立

## 回看决策

不涉及决策：这一轮判的是检出钩子、统一任务服务、引文核对脚本与变异脚本的改动，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
