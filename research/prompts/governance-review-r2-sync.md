<!-- knowledge-sync -->
# governance-review-r2 阶段同步

触发文件：.claude/agent-common.md、.claude/agents/crash-verifier.md、.claude/agents/experiment-runner.md、.claude/agents/kb-scribe.md、.claude/agents/mutation-triage.md、.claude/agents/three-way-verifier.md、.claude/gate.d/10-kb-rot.sh、.claude/gate.d/20-kb-shape.sh、.claude/gate.d/54-layer0-replay.sh、.claude/gate.d/75-decision-experiment-links.sh、.claude/gate.d/90-term-renames.sh、.claude/gate.d/lib-governance-refs.py、.claude/hooks/heavy-test-guard.sh、.claude/hooks/runner-dispatch-guard.sh、.claude/rules/format-evolution.md、.claude/rules/implementation-workflow.md、.claude/rules/mutation-sampling.md、.claude/rules/three-way-inference.md、research/scripts/ask-local-selftest.sh、research/scripts/ask-local.sh；另改了 .claude/main-agent.md、.claude/skills/crash-test/SKILL.md、.claude/skills/gate/SKILL.md 与门禁 10 号的绿样本（不在触发登记里，一并记）；改动范围以 1c58cfa 为基准（第二轮严查开跑时的 HEAD）

这一阶段做成的事：对分支相对 73ba4a4 的全部改动做第二轮严查（甲组 9 条、乙组 9 条、丙组 16 条，报告原样入库，见「原始证据」），逐条现查：甲组 A1 不成立（只改文档的提交 54 号经 change-touches-crates.sh 退 77，不会红），甲组 A3 与乙组第 1 条同一处，其余全部改掉；改法只被这一轮之后的确认轮攻过。

## 搜索

- `grep -rnF '那一行各栏都有' .claude research/scripts` → 0；`grep -rnF '输出里有编译错误记' .claude` → 0（改后）
- 54 号三行拿桩在临时仓里跑：`--full` 退 0 / 1 / 250 时整次调用退同一个数；一次提交都没有的仓（worktree 建不起来）退 1、`--full` 没跑、打出「--full 没跑」；四种情形临时目录都不留
- `ASK_LOCAL_SCRIPT=<删掉清零 VOID_SAVED 的副本> bash research/scripts/ask-local-selftest.sh` → 「✗ 判红了却没留下 case1-output-void1.md」；真脚本自检通过
- `HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1 bash .claude/hooks/heavy-test-guard.sh --selftest` → 「崩溃验证员带前缀不经内存包装跑测试目标 应当是 2，实际 0」判红，内存包装那一道对崩溃验证员仍有判别力
- `python3 .claude/gate.d/lib-governance-refs.py` 真仓 → 「扫 29 份治理文档：门禁号 121 处、路径 326 处、小节 94 处；没判 10 处；跳过 68 处（被 .gitignore 挡着 67、不像仓内路径 1）」退 0；在 `.claude/gate.d/` 里直接跑 10 号退 0
- `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d` → 10、20、75、90 号红绿都判得对；红的只有 55、59、74（与改动前相同，容器没有 qemu、用户级 systemd）

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/gate.d/54-layer0-replay.sh:232 | worktree add / apply 失败时第三行照跑 --full | 改了：丙 1：`layer0_tree_ready` 为 1 才跑，否则退 1；甲 A9：说明改成「经内存包装的那条命令的」 |
| .claude/main-agent.md:59 | 退出码就是 `--full` 的 | 改了：甲 A9 |
| .claude/gate.d/10-kb-rot.sh:29 | cd 之后按相对 $0 找共用库 | 改了：丙 2 |
| .claude/gate.d/10-kb-rot.sh:177 | 退 2 当「一份治理文档都没扫到」 | 改了：丙 2：3 是没有对象，其余非 0 是没跑成 |
| .claude/gate.d/lib-governance-refs.py:135 | 跳过的不计数 | 改了：丙 3：成功行报跳过的两种各几处 |
| .claude/gate.d/lib-governance-refs.py:35 | 「、」连着的号；「-」后面的号 | 改了：丙 6：「/」连着的也判，「-」「–」后面的不判 |
| .claude/gate.d/lib-governance-refs.py:18 | 带非 ASCII 字符 | 改了：丙 4；丙 5 嵌套「」写进文件头 |
| .claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md:10 | （样本） | 补了：丙 7：「/」连着的号、日期后的号、两种没判的 |
| .claude/gate.d/20-kb-shape.sh:46 | 历史类文件整份跳过的理由 | 改了：丙 10：写明是射程不是豁免 |
| .claude/gate.d/90-term-renames.sh:23 | 逐文件与冻结证据目录矛盾 | 改了：丙 9 |
| .claude/gate.d/75-decision-experiment-links.sh:226 | 「（3 项未定）」带空格判红 | 改了：丙 11 / 乙 3 |
| .claude/hooks/runner-dispatch-guard.sh:441 | 提交时的重阶段派 crash-verifier；gate.sh 整轮 | 改了：丙 12 |
| .claude/hooks/runner-dispatch-guard.sh:42 | 否定词与名字写进同一个分句 | 改了：丙 13 |
| .claude/hooks/heavy-test-guard.sh:767 | 崩溃验证员不经包装跑层 0 测试目标 | 改了：丙 14：换成非层 0 目标，内存包装那一道重新有判别力 |
| research/scripts/ask-local.sh:13 | 退出码表 | 补了：丙 15 |
| research/scripts/ask-local-selftest.sh:28 | 只证 UNCHECKED 那一半 | 补了：丙 16：带 VOID_SAVED=1 跑损坏那一格 |
| .claude/rules/mutation-sampling.md:82 | 输出里有编译错误记「无效」 | 改了：乙 1 / 甲 A3；乙 4 另外三栏 |
| .claude/agents/mutation-triage.md:28 | 同上 | 改了：乙 1、乙 4 |
| .claude/agents/mutation-triage.md:27 | 编不过的在「有变异无效」一栏 | 改了：乙 1 |
| .claude/agents/three-way-verifier.md:21 | 设计轮对不上一律分不清 | 改了：乙 2：拿开工时刻现查，没改过记 ✗ |
| .claude/agents/three-way-verifier.md:27 | 判别力自证 | 改了：乙 2 |
| .claude/agents/kb-scribe.md:20 | N 写阿拉伯数字或汉字数字都认 | 改了：乙 3 |
| .claude/agents/kb-scribe.md:32 | 49、75 号 | 补了：乙 8：21、30 号 |
| .claude/agents/experiment-runner.md:30 | 二进制编在 research/target/ | 补了：乙 5：入库装置编在仓根 target/ |
| .claude/agents/experiment-runner.md:32 | 登记复跑 | 补了：乙 5：入库装置写 driver_e<号> |
| .claude/agents/experiment-runner.md:27 | name 写连字符 e<号>-<英文名> | 改了：乙 6 |
| .claude/agents/crash-verifier.md:24 | 未跟踪文件按全部输入路径查 | 改了：乙 9：只查 crates litmus 与 .lkmm-static-only |
| .claude/agents/crash-verifier.md:37 | 产出表 | 补了：乙 7：日志路径 |
| .claude/rules/three-way-inference.md:163 | 报告带 `-s2` 后缀 | 改了：甲 A2 |
| .claude/rules/format-evolution.md:70 | 27 号不扫的只列脚本与产物 | 改了：甲 A4 |
| .claude/skills/crash-test/SKILL.md:12 | 门禁分诊员跑登记给自己的那几道 | 改了：甲 A5 |
| .claude/skills/gate/SKILL.md:12 | 其余时候不跑 | 改了：甲 A6 |
| .claude/agent-common.md:46 | 内存包装退出码与 cargo bench | 补了：甲 A7 |
| .claude/rules/implementation-workflow.md:52 | 主 agent 在提交流程里后台起 | 改了：甲 A8 |
| .claude/main-agent.md:59 | 这一批改了 54 号的输入…才跑全量 | 不改：甲 A1 不成立，只改文档的提交 54 号经 research/scripts/change-touches-crates.sh 退 77，不判红 |

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 甲组审查报告（规则与入口） | research/prompts/governance-review-r2-report-A.md | 63 |
| 乙组审查报告（agent 定义） | research/prompts/governance-review-r2-report-B.md | 55 |
| 丙组审查报告（门禁钩子脚本） | research/prompts/governance-review-r2-report-C.md | 56 |
