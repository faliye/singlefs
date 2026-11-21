# wrapped-writers-loop-paths-cite-titles-governance-r2：判决（2026-11-21）

正文 `research/prompts/_wrapped-writers-loop-paths-cite-titles-governance-r2-body.md`，开工快照 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-snapshot/`。这是这一题的第二轮（治理文件轮），攻第一轮判决 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-main-verification.md` 的修补与新做的改动 E，辩方腿复核那份判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-output.md` | 第 1 问不成立（10 格各钉住判决点名的那一件，点名新格的开关是 8 条、报告与正文写成九条）；第 2 问打中（花括号组接在管道或 `&` 上、heredoc 正文里的一行 `cd`、`-c` 串里的 `env -C` 三形算出错的当前目录、读错结局文件而漏报；干跑那一趟退 10 的出路漏了「这一条自己是 --dry-run 就能换 --scratch」；已经正常跑完的换钩子也被点名）；第 3 问打中（带反引号的错标题或伪造标题里带「，」「。」时改前判红、改后降成没判；不带反引号的标题后面紧跟括号说明取过头判红；兜底把散文收进没判，292 份腿报告里 67 处）；第 4 问轻度打中（文件头「各工作进程的测试线程合计不超过线程上限」对用例显式给的值不成立：显式给 8、4 个工作进程合计 32） |
| 辩方（Sonnet） | `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-output.md` | 第一轮判决四条都站得住，各有限定：带条件循环的过拒在改前单名字一路上早就有；「改前照报」要看改前文件才证得出，两个写工具只取头一个在被杀时也是回退；「换 --scratch」当且仅当两趟里至少有一趟是 --dry-run 时有用，判决与修补的出路写窄了；主 agent 复测用的标题「不为省空间牺牲自包含（附）」在 `fs-design.md` 里不存在，那两次调用只证明解析结果。四个替代方案各有判错的情形，第一轮选的做法照留。旁见 `.claude/hooks/bash-command-detector.sh:224` 多一个「在」字 |

核查：

- 开工快照：`sha256sum -c research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-snapshot/judged-sha256.txt` 17 份里 6 份对不上（`task-server/README.md`、`task_server_lib_constants.py`、`task_server_lib.py`、`task_server.py`、`task_server_selftest_notifying.py`、`task_server_selftest.py`），是腿跑着的时候别的会话往任务服务加 GPU 服务健康那一块改的；没起 snapshot-tree（那 6 份的改动没提交，倒推不出）。这一轮打中的几处都落在没被改的 `task-server/task_server_scheduler_building.py`、`task_server_scheduler_lifecycle.py`、`scripts/agents/cite-check.py`、`scripts/mutation/mutate.sh` 与检出钩子四份里；落在那 6 份里的只有异常表与 README 的出路文字，主 agent 现读此刻那一版核。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-output.md research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-output.md --background research/prompts/_wrapped-writers-loop-paths-cite-titles-governance-r2-body.md`，末行「核了 38 处引文、42 处裸行号，对上 38 处，没判 42 处」，没有对不上的。
- 主 agent 复跑：
  - 改动 D 改前改后对照（攻方腿的草稿拷进 `~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-main-rerun/` 交）：`task-server/task submit --cwd <复跑目录> -- bash <复跑目录>/tools/run-cite-compare.sh <复跑目录>`，type=succeeded exit=0，末三行「done 报告 1 份 改前认出 7 处 改后认出 17 处 结局不同 14 处」「done 报告 1 份 改前认出 11 处 改后认出 10 处 结局不同 6 处」「done 报告 292 份 改前认出 14992 处 改后认出 15067 处 结局不同 75 处」。明细 `cite/forms.compare.out` 里伪造引文加带「，」的标题：改前「bad … 标题「对不上就当场说，三样都说清」在这份文件里命中 0 次」，改后「skip … 小节标题认不出结尾」。
  - 主 agent 另在本机直接跑此刻那一版：一条伪造引文、位置写成带「，」的不存在的标题，末行「核了 1 处引文、0 处裸行号，对上 0 处，没判 1 处」，退 0。
- 核不动、照「推的」办的：攻方第 2 问三种算错当前目录的写法与多报（腿的样本格 `task-server-cells.py` 没写进任务服务自证，没跑）；攻方第 4 问显式值合计超上限（读码）；辩方腿的七条补充探针没跑（依赖它草稿里的符号链接，与攻方那条对照重叠）。

## 二、判决

1. 攻方第 1 问（改动 A 的修补）：不成立，照腿判；「九条开关」改正为点名新格的开关 8 条（腿用 awk 数的，主 agent 没另数）。辩方的限定采：带条件循环的过拒不是改动 A 新开的一类。`.claude/hooks/bash-command-detector.sh:224` 多的那个「在」字并进改法。
2. 攻方第 2 问（改动 B、C 的修补）：采，标「推的」。三形算错当前目录而读错结局文件、正常跑完的换钩子被点名，都没跑出读数，修补时先把腿的样本格写进任务服务自证、红了再改。退 10 的出路两条腿从两头指到同一处，采辩方的写法：换 `--scratch` 当且仅当两趟里至少有一趟是 `--dry-run` 时有用。辩方另指的「两个写工具只取头一个在被杀时也是回退」第一轮判决写轻了，这里改正；那一处第一轮修补已改成逐条判。
3. 攻方第 3 问（改动 D 的修补）：坐实而采（对照量到）。伪造引文配带「，」「。」的标题由判红降成没判，是第一轮改法 4 带进的放松；取过头与兜底把散文收进没判照腿判。辩方指的主 agent 复测标题不存在属实：第一轮判决引的那两次 `citations_in` 只证明解析结果，「合法括号标题被丢」那一句改正为「带括号的标题写法改后整条不被解析」。
4. 攻方第 4 问（改动 E）：采，轻。判据对服务设的那一路判得对、不分两遍照腿判；文件头那一句对用例显式给的值不成立，改文件头，不改判据（显式值是调用方自己要的）。
5. 辩方第 6 问：四个替代方案各有判错的情形，第一轮选的做法照留；改法 4 那一支另采它指的漏口（编造的标题带「，」时不红）。

## 三、改法

改法 1–4 是治理文件，不进产品代码，不做 A/B；改完开第三轮（最后一轮）。

1. 检出钩子：`.claude/hooks/bash-command-detector.sh:224` 删掉多的「在」字；工具员报告之外没有别的要改。零轮。
2. 任务服务算当前目录：花括号组接在管道或 `&` 上、heredoc 正文里的行、`env -C` 这三形照「算不出、照报」办（不算出一个错的目录）；正常结束的换钩子不点名；攻方腿的样本格先写进 `task-server/task_server_selftest_memory.py`、在改前红。零轮。
3. 退 10 草稿目录被占那一支的出路改成「等占着的那一趟结束再交；两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是），可以换一个 --scratch」，异常表那一行与 README 三语第 174 行同改；自证钉新句。零轮。
4. 引文核对：标题带「，」「。」而文件里没有这样的整行标题时，带反引号的照判红（反引号把标题的头尾定死了，不是「标题后面接了说明」），不带反引号的才列进没判；不带反引号的标题后面紧跟括号说明的，括号前的那一段是文件里恰好一处的整行标题就取它；兜底那一支只收位置形态确是小节写法的，不收散文。自证补这几形红绿样本。零轮。
5. 变异脚本文件头「各工作进程的测试线程合计不超过线程上限」改成只对服务设的值成立、用例显式给的值原样传并写明合计可以超过。零轮。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/hooks/bash_command_detector_literals_and_signals.py：判了按候选逐个判的共用段与文档串，与文件头逐句对得上，成立
- .claude/hooks/bash_command_detector_selftest_entries_and_main.py：判了 ⑯ ⑲ 两格与拒绝说明里的出路，各钉住判决点名的那一件，成立
- .claude/hooks/bash_command_detector_selftest_waits_and_scope.py：判了 ⑦ ⑫ 新格与 stdin 那一格，各钉住判决点名的那一件，成立
- .claude/hooks/bash-command-detector.sh：判了文件头 ⑦ 与开关点名（点名新格的开关 8 条）；第 224 行多一个字（改法 1）
- scripts/agents/cite-check.py：判了成对括号的取法与认不出结尾那一支；带「，」「。」的伪造标题由红降成没判、取过头、兜底收散文（坐实，改法 4）
- scripts/mutation/mutate.sh：判了分线程的判据（对服务设的值成立、不分两遍）与新格、三条开关；文件头合计那一句对显式值不成立（改法 5）
- task-server/README.en.md：判了第 174 行退 10 出路同一句，同改法 3
- task-server/README.ja.md：判了第 174 行退 10 出路同一句，同改法 3
- task-server/README.md：判了第 174 行退 10 出路；换 --scratch 的条件写窄（改法 3）
- task-server/task_server_lib_constants.py：判了异常表那一行分开写退 9 与退 10，成立；出路同改法 3
- task-server/task_server_lib.py：判了文件头开关表与说明写现状，成立
- task-server/task_server.py：判了文件头 previous-swap-running 的现状句，成立；出路同改法 3
- task-server/task_server_scheduler_building.py：判了 shell_text_runs 与逐条交写工具；三形算出错的当前目录（推的，改法 2）
- task-server/task_server_scheduler_lifecycle.py：判了 writer_interruption、wrapped_hook_swap 与起法挪回登记的方法（行为逐条相同，成立）；正常结束的换钩子被点名、退 10 出路写窄（改法 2、3）
- task-server/task_server_selftest_memory.py：判了三格新格；缺算错当前目录三形的样本格（改法 2）
- task-server/task_server_selftest_notifying.py：判了退 10 样本钉住新句，成立；新句随改法 3 再改
- task-server/task_server_selftest.py：判了格名登记与现数，成立

## 回看决策

不涉及决策：这一轮判的是检出钩子、统一任务服务、引文核对脚本与变异脚本的改动，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
