# m3-close-reuse-closure-r1：判法单元闭包不走内存包装查机器那条提示路径，加上第二台探测改成同一份脚本带参数，站不站得住（推论轮第一轮）正文（2026-10-22）

## 一、题面

复用重型门禁的判定按每条判定的单元键判（`.claude/rules/implementation-workflow.md`「复用重型门禁的判定：按每条判定的单元键判」一节；D13（验证路线） 已定项 17）。形式化门禁 checker-tier-formal-verification 的三个判法入口（`scripts/gate/verus-proof-check.py` 的 check_proof、check_vacuity、check_must_red_worlds）今天的判法单元指纹都是「拒算」，所以每次现判、不复用，`python3 scripts/gate/verus-proof-check.py --selftest` 119 格里 22 格（`reuse-*` 一族）红。己九工具员（报告 `~/.local/share/singlefs/work/m3-close-b16-tools-ji9/result.md` 第 3 条）把四处折不开的写法改掉之后，只剩一处拒算：

判法入口 → 内存包装 `scripts/hosts/run-with-memory-cap.sh` 的 `HostLookup.query`（第 988 行附近；本机内存排队时查别的机器放不放得下，只打一行「第二台放得下，改用……」的提示）→ `scripts/hosts/host-resources.py` 的 `run_on_machine`（第 312–329 行附近，把 `remote_probe_program()` base64 编码后作命令文本）→ `scripts/hosts/peer-host-lib.sh` 的 `peer_host_run`（第 76 行起，本机替身那一支跑 `bash -c "$2"`）。判法单元的算法（`scripts/gate/lib_function_units.py`；Rust 那一份在 `crates/singlefs-unit-graph`）折不开函数里 `bash -c "$N"` 的串，于是拒算。`scripts/hosts/host-dispatch.py` 第 316 行同样经 `peer_host_run` 送 base64 程序。

用户 2026-10-22 两次弹窗的定案（原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`）：「脚本必须是一样的 输入参数来控制跑那台机器 闭包不走内存包装 无法保证机器可以稳定运行。 难道还有第二种可以保证的方法？ 我觉得是没有的 如果有 可以丙 没有就只能甲」；主 agent 解释「闭包只管判定能不能复用，不管包装跑不跑」之后，用户选「丙加甲 要走三方」。

- 丙：判法单元闭包不走内存包装里只打提示的那条查机器路径（`HostLookup` 那一支），配一格自证把「那条路只打提示、不改判定结局」钉住。
- 甲：第二台的探测改成两台跑同一份仓里的脚本、由参数选机器（host-resources 与 host-dispatch 不再送 base64 程序）；判法学会「函数里 `bash -c "$N"` 的串算调用处给的」并在调用处折开。要求第二台有同版本的仓。

这一轮要回答：丙加甲这一组改法，是否保住「键只由判定依赖的函数单元组成、判定依赖的单元变了一定重判」；丙排掉的那条路里有没有会改判定结局的东西；甲把「两台同一份脚本」做成之后，键怎么算第二台那一侧（第二台跑的是同一份脚本，它的单元进不进键、怎么进）。

## 二、实现今天的样子（主 agent 的观测，2026-10-22，行号按主工作区，HEAD 841b3beab 加未提交的改动）

- `crates/singlefs-unit-graph`：Rust 与 WGSL 源码切单元、取去注释指纹、收闭包、算复用键的库（`.claude/rules/verification.md`「定义与名字」函数单元图那一行）；`scripts/gate/lib_function_units.py` 是脚本那一侧的同一套算法。
- `scripts/hosts/run-with-memory-cap.sh`：第 920 行 `class HostLookup`、第 988 行 `def query`（内嵌 python）。
- `scripts/hosts/host-resources.py`：第 227 行 `remote_probe_program`、第 312–329 行 `run_on_machine`（`source "$1"…; peer_host_run / "$5"`，program 经 base64 编码）。
- `scripts/hosts/host-dispatch.py`：第 315–316 行 `runner = 'source "$1" || exit 70; eval "$2"; peer_host_run / "$3"'`。
- `scripts/hosts/peer-host-lib.sh`：第 76 行 `peer_host_run`。
- 己九工具员量过：改那四处之后，`closure . scripts/gate/verus-proof-check.py::<入口>` 三个入口各剩一行 `refused inline-script-not-literal scripts/hosts/peer-host-lib.sh:76`；`VERUS_PROOF_CHECK_BREAK=reuse-key-without-judging-units` 时 22 格只剩 `reuse-judging-units-changed` 一格红。

## 三、格

| 格 | 问什么 |
|---|---|
| V1 | 丙排掉的那条路（`HostLookup.query` 起 host-resources、经 peer_host_run 探第二台）上，有没有任何一处会改变判法入口的判定结局（退出码、判红判绿、写进判定库的内容）；有的话写出在哪种机器状态下改；「只打提示」用什么自证钉得住、弄坏开关怎么设 |
| V2 | 丙的排除规则怎么写才不越界：按「哪一条调用边」排、按「哪个函数」排、还是按「包装的哪个开关」排；写法会不会顺带把真会改结局的代码也排出闭包（列出别的经 run-with-memory-cap.sh 起的判定入口，逐个看） |
| V3 | 甲「两台跑同一份仓里的脚本、参数选机器」：键里怎么算第二台那一侧（同一份脚本的单元已在本机闭包里，第二台的仓版本不同怎么办、要不要核版本、核不过时判定怎么处置）；判法学会「`bash -c "$N"` 的串算调用处给的」会不会让别的现在正确拒算的地方被错误折开 |
| V4 | 丙加甲合起来，`.claude/rules/implementation-workflow.md` 那一节与 D13 已定项 17 的条文要不要改、改成什么（逐字候选）；自证与门禁 code-tooling 的 reuse-selftests 格要加什么 |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | V1–V4 | 从附录条款与上面列的脚本今天的代码正推：先答 V1（那条路会不会改结局），再推 V2–V4 |
| 云端攻方（Opus） | V1–V4 | 找反例：V1 里那条路改判定结局的机器状态；V2 排除写法顺带排掉的真会改结局的代码；V3 甲之后键漏掉第二台版本差、或错误折开的地方；V4 候选句与已定条款打架的地方 |

方案按 `crates/` 与 `scripts/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
