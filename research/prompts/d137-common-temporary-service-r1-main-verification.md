# d137-common-temporary-service-r1 判决（第一轮，2026-11-05）

被判：`research/prompts/_d137-common-temporary-service-r1-body.md` 第一节——`.claude/agent-common.md` 第 64 行「子 agent 不交任务」那一条里新加的一句（要临时服务的自证怎么写命令）。腿：正推腿 `research/prompts/d137-common-temporary-service-r1-sonnet-output.md`（F 格），攻方腿 `research/prompts/d137-common-temporary-service-r1-opus-output.md`（A 格）。

## 核查怎么做的

- 开工快照倒推（任务 6782）：「清单点名 7 份，倒推出 6 份、sha256 对上 6 份；倒推不出 1 份」。倒推不出的是 `.claude/gate.d/code-tooling.sh`：开轮之后有一次提交改过它，HEAD 那一份 sha256 3af22603 不是快照的 b53eaece，工作区 diff 里也没有它的基底（另一个会话的外壳组改的：38 格减到 36 格，research-script-selftests 换守卫）。腿引它的两处（`code-tooling.sh:1045`、`:625`）主 agent 照此刻工作区逐处人工核：两处都写读法在 `.claude/hooks/lib_project_code_runs.py`，与攻方 H3 说的相同。
- 引文：`python3 scripts/agents/cite-check.py <两份腿报告> --root <倒推出的树> --background <正文>` 核了 66 处引文，对上 38 处，没判 37 处（引到不在快照里的文件，倒推出的树里没有）；再以主工作区为根跑同一条：「核了 66 处引文、9 处裸行号，对上 66 处，没判 9 处（其中裸行号没有引文可比 9 处）」。裸行号里承重的三处主 agent 手工核：`scripts/prune-server/prune_server_client_selftest.py` 第 143 行起 `cargo build --release --offline --manifest-path … --bin prune-server`，第 1054–1056 行 `resolve_binary` 在没有 `--server-binary`、没有 `PRUNE_SERVER_BINARY` 时交 `build_server_binary()`，第 1105 行起临时服务照 `resolve_binary` 取二进制；`.claude/hooks/selftest-temporary-service.tsv` 第 2、6 行点名 `lib_heavy_tests.py`、`heavy-test-guard.sh`。
- 腿交回的命令，主 agent 经服务复跑：
  - 正推腿：喂三道钩子（任务 6777）「SENTENCE_JUDGE cases=54 mismatches=1」，对不上的那一条是 `MISMATCH case=S scripts/crash/lib_verdict_library.py hook=run-scope-guard.sh want=2 got=0`：子 agent 交这条命令，`heavy-test-guard.sh` 与 `project-code-guard.sh` 都退 2，腿另外预期 `run-scope-guard.sh` 也退 2——它只判验证命令的范围声明，自证不在射程里，是腿的预期写错，不是被判句的毛病；其余 53 条（24 份 required 的包装写法三道钩子都放行、裸写被拒、own-server 两份放行、带 `--server-binary` 的放行）都对上。不带二进制真起（任务 6778，派到本机）退 0，日志「PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1」，`task status` 编译目录一栏「无」；主仓 `prune-server/target/release/prune-server` 的修改时刻落在这一趟跑的那段时间里（日志里没有 cargo 的输出行，是谁编的只能算相容，推的）。带 `--server-binary /home/fy5090/code/singlefs/prune-server/target/release/prune-server` 真起 `crates-mutation-rows.py --selftest`（任务 6779，派到第二台）退 0：「crates-mutation-rows.py 自证 191 格都对」，主仓里的二进制在共享根下、两台读的是同一份。
  - 攻方腿：P1（任务 6780，派到第二台）退 0，日志打出 `PRUNE_SERVER_BINARY` 与 `CARGO_TARGET_DIR` 都在服务给的 `builds/task-b0f6b921-6780-1/target` 下——攻方写的打翻条件「打出了 `CARGO_TARGET_DIR`」在第二台成立；P2（任务 6781）`p2a_rc=0`、`p2b_rc=0`，与预期相同。

## 逐条判

| 条 | 腿 | 判 | 主 agent 现查 |
|---|---|---|---|
| H1 / 正推缺口 1：这一句的写法里没有 `--server-binary` 的位置；不带它时包装回落到现编 prune-server | 两条腿都报 | 采一半 | 回落链源码在（第 143、1054–1056 行）。派到本机时服务不给编译目录（6778 那一栏「无」），主仓里那份二进制在这一趟里被重编（相容，推的）；派到第二台时服务给了编译目录（6780）。攻方「编进主仓、服务不给编译目录」只在本机那一侧成立。句子里写可以带 `--server-binary` 修不了根：子 agent 给不出可信的二进制路径，根在服务把「包装会现编」认成不编（本机那一侧不给编译目录），归做统一任务服务的会话与 prune-server 那一路 |
| H2：共用约束这一句与派发骨架那一行是两处登记、已经分叉 | 攻方 | 采 | 骨架提可以带 `--server-binary`，这一句没提；骨架只给四种类型（`scripts/agents/dispatch-prompt.py:287`），三方腿、回扫员等只靠共用约束，这一句不能删。改成指到登记表文件头那一条整条写法，不再自己抄一份 |
| H3：登记表文件头点名的读法已过时 | 攻方 | 采，交改名那一路 | 文件头第 2、6 行写 `lib_heavy_tests.py`、`heavy-test-guard.sh`，`code-tooling.sh:625` 已写 `lib_project_code_runs.py`；钩子改名正由另一个会话做，文件头随改名一起改 |
| H4：`crash-point-replay-shard-run.sh --selftest-real-sharded-run` 裸起也退 2，句子与执行前拒都只认单独一个 `--selftest` | 攻方 | 不采，记账 | 那条命令不是 `--selftest`，不归这一句；登进拆分提案，下一次动登记表或执行前拒时一并判 |
| H5：同一条里「交给服务的命令不带包装前缀」与这一层同叫「包装」，字面互顶 | 攻方 | 采 | 功能上不顶（24 行 required 照写一行都不被服务退 3，6777）；句子里写明这一层不算那一串里的包装 |
| H6：第 66 行豁免「登记为 light 的单独 `--selftest`」管不到包装形态 | 攻方 | 不采，交改名那一路 | 用户已定不再分轻重快慢，那一条正随钩子重写一起改 |
| H7 / 正推缺口 2：没写 `<解释器>` 怎么取 | 两条腿都报 | 采 | 表里 14 行 `.sh`、10 行 `.py`，按「`.py` 用 `python3`、别的用 `bash`」拼全都起得来（6777） |
| 正推缺口 3：没写跑完看哪一行 | 正推 | 不采 | 这一句给写报告的子 agent 看；读结局的是主 agent，读法在登记表文件头与包装输出里 |
| 正推缺口 4：与「不带包装前缀」字面挨着 | 正推 | 并进 H5 | — |

## 改法（零轮）

被判句改成：

> 登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本，写进报告的自证命令照那张表文件头 required 一条给的整条写法套临时服务（`<解释器>`：`.py` 用 `python3`，别的用 `bash`；这一层不算「交给服务的命令不带包装前缀」那一串里的包装）

| 改法 | 零轮 | 去向 |
|---|---|---|
| 被判句照改（H2、H5、H7） | 零轮 | 主 agent 改，再攻一轮（`.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 2 步「打中的写回代码，再攻一轮」） |
| H1 的根：服务把「包装不带二进制会现编」认成不编，本机那一侧不给编译目录 | — | 交做统一任务服务的会话与 prune-server 那一路，登进拆分提案 |
| H3、H6 | — | 交改钩子名的那一路 |
| H4 | 不采 | 登进拆分提案 |

## 回看决策

不涉及决策：这一轮判的是共用约束里的一句与配套的登记表、钩子、骨架，没有动 `.claude/kb/decisions/` 里的任何分项。
