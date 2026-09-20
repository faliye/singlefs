# 运行记录：m2-supp3-item3-code-r1 本地攻方（K5、K6）

提示：`research/prompts/m2-supp3-item3-code-r1-local-attack.md`
核对表：`research/prompts/m2-supp3-item3-code-r1-local-attack-translation-audit.md`
调用：`nice -n 19 bash research/scripts/ask-local.sh <提示文件>`，前台跑，未用 `setsid` / `&` / `disown`（第 2 次调用因超过工具 120 秒的前台展示上限被工具自身移入后台继续跑，本 agent 没有主动加 `&`/`disown`/`setsid`，等到工具发的完成通知后才读取结果）。

跑前检查：`ps -o pid,args -u "$(id -u)"` 未见 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`、别的 `ask-local.sh` 或对 `:8200` 的并发调用。

提示写好之后、第一次调用之前，先用 `python3 research/scripts/replace-once.py`（定点替换）核实并修了三处翻译保真问题（详见核对表：Q1「按种子从小到大」被首稿弱化成「按片」、Q7 丢了「第一轮」「三方」、F12「与它报的第一处」被首稿多译出一层「只留第一条」的去重断言），修完才开始第一次调用。

第 1 次调用（判红作废，`m2-supp3-item3-code-r1-local-attack-output-void1.md`）：退出码 5。`corruption-check.py` 判红的不是提示文件（提示当时也判绿），是模型输出：英文复读 7 处（9.16/千），样本 `not checked | not checked | nothing additional | nothing additional`——是提示原先要求「column i column ii column iii」裸空格并排导致同一个合法短答案连续出现两次，被复读检测的二元组正则命中，不是模型真的多吐字。判定：提示的输出格式设计有结构性缺陷，会让每一次调用都大概率撞上同一条误判，不算「三方不一致」也不算「本地腿缺席」，是提示本身要改；改法：在提示格式规则里加一条（第 9 条），要求同一行填多列时列与列之间用分号分隔、每个值前都写列标签，不许裸空格并排两个相同短语。

第 2 次调用（判红作废，`m2-supp3-item3-code-r1-local-attack-output-void2.md`）：退出码 5。这一次模型输出本身判绿（`绿 ... words=1189 英文复读=0`），但 `ask-local.sh` 同时把提示文件自己也喂给了 `corruption-check.py`（脚本原文 `python3 "$CHECK" "$TXT" "$1"`），提示文件被判红：粘连=8，全部命中 Rust 路径分隔符 `::`（`CrashInjectionReport::render`、`CrashInjectionTally::absorb`、`GenerationWeights::BROAD`、`CrashInjectionWorkerThreads::GivenByCaller` 等 8 处），因为「粘连」判据是「标点不接在字母数字后面、又紧跟着一个词」，`::` 的第二个冒号恰好命中。核实：`python3 research/scripts/corruption-check.py research/prompts/m2-supp3-item3-code-r1-local-attack.md` 单独跑，退出码 1，与 `ask-local.sh` 内部报的「粘连=8」一致。改法：把提示正文里全部 8 处 `::` 改写成不含冒号的英文短语（例如「the render function belonging to CrashInjectionReport」），改完单独重跑 `corruption-check.py` 判绿（退出码 0）才继续。这一条只影响提示文件本身能不能过闸，与模型答得对不对无关。

第 3 次调用（`m2-supp3-item3-code-r1-local-attack-output-s1.md`）：退出码 0。`wc -w` 1101 词（`corruption-check.py` 自己数得的英文词数 1175，两个计数器口径不同，都列出）。`oov-check.py` 判红：生词=36，拼接=1（`FailureObservation(=failure+observation)`），生词表 `overturned RaiseRollbackFloor FailureObservation`。核实：`grep -no "FailureObservation[A-Za-z]*"` 在样本里只命中一次、拼写与提示正文给出的类型名（`crates/singlefs-harness/src/history.rs` 的 `pub struct FailureObservation`，提示 Section 6 标题与多条 Fact 都原样写出这个名字）逐字节相同，不缺头、不多字母，不是「缺头的粘连词」（与定义举的反例 `achievesceives` 不同类，那个例子缺了「re」）；`overturned` 是提示自己要求的收尾短语「Would be overturned by」里的词，`RaiseRollbackFloor` 是提示 Fact F11 给出的真实枚举成员名。三个词都现查过来源，判定不带损坏。另通读全文（33 问全部作答、每问都跟一行「Would be overturned by」、无缺词断句、无孤立标点、无 `::`、无中文字符），归类：干净。

第 4 次调用（`m2-supp3-item3-code-r1-local-attack-output-s2.md`，前台起跑后超过工具 120 秒展示上限被工具移入后台，等完成通知后读取）：退出码 0。`wc -w` 1055 词（`corruption-check.py` 自己数得 1136）。`oov-check.py` 判红：生词=36，拼接=1，与第 3 次完全同型（同一个 `FailureObservation(=failure+observation)`、同一张生词表），判定同上，不带损坏。通读全文：33 问全部作答，格式与第 3 次一致，无缺词断句、无孤立标点、无 `::`、无中文字符。归类：干净。

四次调用（两次判红作废、两次退出码 0 且过人工复核）已达到「至少两份干净样本」的停止条件，未再继续抽样，在「连续五次调用拿不到两份干净的才停下」这条线之内。

汇总表

| 次序 | 文件 | 退出码 | 词数（wc -w） | oov-check.py 判定 | 拼接命中 | 归类 |
|---|---|---|---|---|---|---|
| 1 | `m2-supp3-item3-code-r1-local-attack-output-void1.md` | 5（corruption-check 判红：英文复读 7 处） | 679 | 未跑（判红作废，不计入干净/带损坏统计） | 未跑 | 作废 |
| 2 | `m2-supp3-item3-code-r1-local-attack-output-void2.md` | 5（corruption-check 判红：判的是提示文件自己的 `::` 粘连，模型输出本身判绿） | 1039 | 红（生词=36，拼接=1，`FailureObservation`，判定见上文，不算带损坏） | `FailureObservation(=failure+observation)` | 作废（按脚本判定退出码 5 一律作废，不因为「其实是提示的问题」而改记别的类别） |
| s1 | `m2-supp3-item3-code-r1-local-attack-output-s1.md` | 0 | 1101 | 红（生词=36，拼接=1，判定为非损坏，见上文） | `FailureObservation(=failure+observation)` | 干净 |
| s2 | `m2-supp3-item3-code-r1-local-attack-output-s2.md` | 0 | 1055 | 红（生词=36，拼接=1，判定为非损坏，见上文） | `FailureObservation(=failure+observation)` | 干净 |

没做什么：不解读、不总结、不采纳 s1、s2 两份样本回答的具体内容、判断或方向是否一致，那是主 agent 的事，本运行记录不下这方面的结论。模型答复里如出现行号，一律记「模型自给、未核」——现查两份干净样本（s1、s2）全文，均未出现任何形如文件名加冒号加数字的行号引用，也未出现 `::`（Rust 路径分隔符）或中文字符。K5、K6 两个攻击面本身出题覆不覆盖得住、云端攻方与云端正推两条腿是否与本条腿重叠，不归本条腿判断。
