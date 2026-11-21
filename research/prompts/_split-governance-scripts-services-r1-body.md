# crates 代码拆分换上的脚本与三个服务：治理文件轮第一轮（split-governance-scripts-services-r1，2026-11-20）

这一轮攻的是按 2000 行拆分之后换上主工作区的脚本与服务：`scripts/` 下各子目录拆成入口加同目录模块（python）或被 source 的库（shell），`task-server/` 三份 py、`prune-server/` 与 `gpu-verification-server/` 两个 Rust 服务的六份，以及拆分换上之后为它修的两件（hook-swap 的自证、看门狗的检出前缀表）。拆分期间用户定「不需要走三方 先拆分代码 全部拆完再说」（`records/2026-11-19-crates代码拆分.md` 第 261 行），这一轮补上。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码；与 `crates/` 有交点的是算键库 `scripts/gate/lib_function_units.py` 一族切 `crates/` 下的 Rust 单元、变异脚本 `scripts/mutation/crates-mutation-rows.py` 一族改坏 `crates/` 源码跑测试、合入后验证 `scripts/gate/merge-and-verify.sh` 一族挑 `crates/` 下的测试目标。

## 问题

1. 拆分声称只挪不改：python 入口留原路径并再导出，拆出的放同目录被 import 的模块；shell 拆成被 source 的库；task-server 的 `Scheduler` 拆成五个混入类；两个 Rust 服务的源码拆成子模块。挪动之外，有没有一处行为变了（攻方腿）。
2. 拆分带出的同一类缺陷有没有漏网的：判法只读入口原文、模块级可变状态存成两份、自证把替身换在错的模块上、`from x import *` 漏带下划线名字与 `__all__` 之外的名字、测试里 `patch.object` 换的不是调用方读的那一份绑定、子进程起的脚本路径与 `sys.path` 跟着挪没有（攻方腿）。
3. 拆分之后换上的两件修补（这一题：hook-swap 自证换替身的模块、看门狗检出前缀表补登），做的是不是它说的事（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是此刻主工作区；拆前原文在 HEAD，`git show HEAD:<路径>`）：

- task-server 三份（`records/2026-11-19-crates代码拆分.md` 第 293 行）：`task_server_lib.py` 入口加 5 个模块、`task_server.py` 入口加 8 个模块（`Scheduler` 拆成五个混入类）、`task_server_selftest.py` 入口加 13 个模块，最长 1718 行；`task-server/tests/test_cleanup_protocol.py` 12 处 `patch.object` 改成换每个模块里的绑定；整份自证「438 格都对」，`control stop`、`start`、`check` 退 0。
- 两个 Rust 服务六份（第 287 行）：`git apply -p1` 打进主工作区，39 份新文件；`gpu-verification-server/control` 读的那几个函数与常量留在根里；集成测试根补一处 `#[path = "../common/mod.rs"]`；服务进程没重编重起。
- 算键库 `scripts/gate/lib_function_units.py` 一族（第 299 行）：入口加 23 份库内模块、夹具三份；拆前拆后自证都是 5 格判错、红格逐条相同；它自己 `main` 的闭包除 `PythonModule` 方法换限定名之外另少 34 个名字、多两种拒算。
- 变异脚本三份（第 300 行）：`crates-mutation-rows.py` 自证红在判法单元闭包有拒算；`broad-mutation.py` 自证两条用例按片段改拷贝时片段在几份模块里都命中；`tests/test_audit_regressions.py` 4 条红在 `CARGO_BUILD_JOBS` 读成空。
- `scripts/gate` 五份拆后与拆前自证比（第 306 行）：`admission.py` 没多出新红格；`merge-and-verify.sh`、`apply-writer-patch.py` 绿；`verus-proof-check.py` 拆后新红一批「拒算（判法单元指纹）」。
- 拆分之后修的两件（这一题；记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 508–509 行）：
  - hook-swap：自证把替身写在 `globals()["scratch_lock_content"]`（`scripts/gate/hook_swap_selftest.py`），拆后被测的 `claim_record_removal` 按 `hook_swap_swap` 的全局名读，改成换在 `hook_swap_swap.scratch_lock_content` 上，两格另核 `.taking` 机器一段确是替身写的。副本里拆前绿、拆后恰好红那一格、修后绿，七条开关红格与文件头点名逐条相同；换上后主工作区自证「查了 119 种」通过。
  - 看门狗：`scripts/agents/agent_watch_alerts.py` 的 `UNJUDGED_DETECTION_PREFIXES` 补登 `ask-user-placement-guard.sh`、`session-number-stop-check.sh` 两份新钩子的起头，自证加一格拆分族的该红输入；换上后自证绿；开关复跑红在门禁那一格「代码里不读弄坏开关了」，没跑成。
- 已知开着、各有会话在修、这一轮不重复报的：算键闭包跨不过拆出的模块（算键库、变异判法、verus 证明项、门禁三道的拒算与少收单元，mainagent(C)）；门禁 break-switch 三格把部分文件当独立脚本判（mainagent(B)）。

## 范围

这一轮被判的文件（202 份，逐个列在 `research/prompts/split-governance-scripts-services-r1-snapshot/judged-sha256.txt`；都读主工作区此刻那一版，拆前那一版 `git show HEAD:<入口路径>`，拆出的模块在 HEAD 里没有）：`scripts/gate`、`scripts/mutation`、`scripts/crash`、`scripts/agents`、`scripts/hosts`、`scripts/prune-server`、`scripts/experiment-apparatus` 下拆过的入口与模块，`task-server/`、`prune-server/src/`、`gpu-verification-server/` 下拆过的文件。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 拆前原文与「入口加模块」拼起来之后，挪动之外有没有行为变了：判法的分支、默认值、退出码、输出的成功句与拒绝说明、弄坏开关的认法、子进程命令行。给得出一条命令或一份输入，拆前拆后判得不一样就是打中。
2. 模块级可变状态：拆后有没有一张表、一个计数、一个缓存被两个模块各存一份（入口再导出的是拷贝而不是同一个对象、`from x import NAME` 之后对 NAME 重新赋值只改了本模块的绑定）。task-server 的五个混入类之间有没有类属性、`super()` 次序的问题。
3. 自证与测试换替身、打补丁（`globals()`、模块属性、`patch.object`、`monkeypatch`、环境变量）在拆分后是不是换在被测代码真正读的那个绑定上；给出一处换错而那一格因此测空了的。
4. 子进程与路径：入口按 `__file__` 找同目录文件、按脚本路径起自己（开关轮、`--selftest-child`）、`sys.path` 插入的目录，拆分后还指得对不对；两个 Rust 服务拆成子模块之后，`gpu-verification-server/control` 读的函数与常量、集成测试的 `#[path]` 还对不对，服务没重编重起这件事会让哪一格的结局与源码不符。

正推腿（逐条回「成立」「不成立」「推不出」）：

5. 两件修补（hook-swap 自证换替身的模块、看门狗检出前缀表补登）逐条核今天的代码做的是不是「实现今天的样子」那一条说的事，写出 文件:行 与那一句原文。
6. 两件修补新加的自证格：每一格红的地方是不是它名字说的那件事。
7. task-server 拆分：`task_server.py`、`task_server_lib.py`、`task_server_selftest.py` 三个入口再导出的名字，覆盖不覆盖拆前三份文件里被别的文件（`task-server/task`、`task-server/control`、`scripts/` 下调它的脚本、`task-server/tests/`）引到的每一个名字；给出缺的那一个。

## 分工

攻方腿做第 1 到 4 问，正推腿做第 5 到 7 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
