# d137-ledger-defs-r2 攻方腿报告（格 A：第一轮修补写出来的新句子与新代码）

日期：2026-11-02。背景材料 `research/prompts/_d137-ledger-defs-r2-body.md`；第一轮判决 `research/prompts/d137-ledger-defs-r1-main-verification.md`；两件修补报告 `~/.local/share/singlefs/work/d137-ledger-defs-r1-fix-defs/result.md`、`~/.local/share/singlefs/work/d137-ledger-defs-r1-fix-scripts/result.md`。禁读的辩方报告没读。

被判的 16 份此刻与快照 `research/prompts/d137-ledger-defs-r2-snapshot/files-sha256.txt` 逐份相同（`sha256sum -c` 只打 OK 行，量的）。下文写的「sha256 前 8 位」都是此刻主工作区那一份；快照里的 16 份照快照那一列，快照外的另写。

格名用 B1–B11，与第一轮的 A1–A13 不重。第一轮已攻过的角度不原样再提；B2 与第一轮 A1 同一条链，只攻修补落地之后的新状态，那一节开头写明差在哪。

## 复跑命令与模型文件

模型目录 `research/prompts/d137-ledger-defs-r2-opus-model/`：三个样本根（`worlds/`）、造根的 `tools/make-worlds.sh`、单跑门禁一格的 `tools/judge-worlds.sh`、`compile-then-swap.py` 命令行探针 `tools/cli-features-probe.py`、期望 `out/expected.txt`；`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷、只改了 `rerun_steps`（只跑 bash 自己、造样本根、造 `compile-then-swap.py` 的变异体；单跑门禁与探针另交，见「要主 agent 跑的命令」第 2–4 条）。命令都没跑过，下文模型的读数全是推的。

```
628d4c0a2690c9695cfbb75e97dedbf82278c8eac90726006d16473f9a8cca98  ./out/expected.txt
4fe796d420678cf5483c596e4d0b418a171bb4f0e33e4964b71967c504203039  ./tools/cli-features-probe.py
e359ecf4d0b243181a54791f76086411bcccb22fed02533408fe0afb6906cf00  ./tools/judge-worlds.sh
295e90b0d4e8e57d1ab9fc25891ed3b0f84b3ad99781f12cff3eda4b0cb8f21c  ./tools/make-worlds.sh
efb9b1dd694d85f631a5a0f35dd0cc6d5e657f8c4f1b10fc64abb9d41b0cafea  ./tools/rerun.sh
2b5e77f9dbdf32813b30668725a2f62ffa68bb1f691034d7d07fcae16e8b1747  ./worlds/comment-backslash-hides-second-command/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/comment-backslash-hides-second-command/before/crates/singlefs-checker-tier/Cargo.toml
49025886cfef5579b8ab996f3e7c7b5f2c9eb89342bac6705417f24648ebd430  ./worlds/comment-backslash-hides-second-command/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/comment-backslash-hides-second-command/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/comment-backslash-hides-second-command/before/research/prompts/README.md
9d1f6ccd3f1eb5546c26024488c1b3c783de20b220a5adf692e07b90267ceff5  ./worlds/comment-backslash-swallows-command/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/comment-backslash-swallows-command/before/crates/singlefs-checker-tier/Cargo.toml
49025886cfef5579b8ab996f3e7c7b5f2c9eb89342bac6705417f24648ebd430  ./worlds/comment-backslash-swallows-command/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/comment-backslash-swallows-command/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/comment-backslash-swallows-command/before/research/prompts/README.md
376bad28b39333982468b407d41dbda33282f7f0b09131f28e36e731cfc9f62a  ./worlds/comment-without-backslash-control/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/comment-without-backslash-control/before/crates/singlefs-checker-tier/Cargo.toml
49025886cfef5579b8ab996f3e7c7b5f2c9eb89342bac6705417f24648ebd430  ./worlds/comment-without-backslash-control/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/comment-without-backslash-control/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/comment-without-backslash-control/before/research/prompts/README.md
```

这一腿没有调崩溃枚举函数的原型，全量崩溃状态数是 0（定义第 3b 步那几条不适用）。

## 各格判定一览

| 格 | 打的是哪一句 / 哪段代码 | 照做会发生什么 | 强度 | 量的 / 推的 |
|---|---|---|---|---|
| B1 | `.claude/agents/experiment-designer.md:31`（ee574a46）末句仍说门禁按 `crash_amplification::` 认判段 | 门禁那一格已改成按判段入口认；设计员照这一句把不存在的「对不上」报给主 agent，读者以为门禁还是改前那一版 | 中低 | 推的（逐字比） |
| B2 | `.claude/agents/experiment-runner.md:30`、`:32`（4a8e5c16）：入库装置带 `--features gpu` 编、产物命令只执行裸二进制 | 修补前判段打 NOT_RUN、报本次未跑；修补后同一条链在服务没分卡、没清场的环境里真上卡判，卡从多机配置里自己挑，与服务正派着的 GPU 任务撞卡 | 中 | 推的（代码逐行） |
| B3 | `.claude/agents/experiment-runner.md:40`（4a8e5c16）「装置的标准输出（任务日志）…`grep -c NOT_RUN`」 | 执行员读任务日志只有 `task-server/task log <号>`，默认只给最后 200 行；NOT_RUN 那一行打在判段开头，输出一长就在 200 行之外，数到 0、当判段跑了 | 中低 | 推的 |
| B4 | `.claude/gate.d/doc-experiments.sh:3084–3086`（690a35ef）新加的续行并接 | 以反斜杠收尾的注释行把下一行命令并进注释，那一条不在命令位置、不判；bash 照样跑它（没带 gpu 也放过） | 低 | 推的；世界在 `worlds/` 待跑 |
| B5 | 特性的几种写法：门禁 `:2914` 认 `--features=`、`-F`、写几次、`--all-features`；`scripts/hosts/compile-then-swap.py:839`（22ced829）只认一个 `--features <值>`、后写的盖前写的；定义说「原样给」 | `--features=gpu`、`-F gpu`、`--all-features` 门禁判绿、原样交给编装置的脚本退 2；`--features gpu --features x` 门禁判绿、编出来的二进制没有 gpu | 低 | 推的；探针待跑 |
| B6 | `compile-then-swap.py` 新自证项 gpu-feature-passed-through 直接调 `swap(features=("gpu",))` | 命令行那一层（主 agent 真用的那一层）把 `--features` 丢掉的变异体，自证照样全过：新自证分不出命令行上的改前改后 | 中 | 推的；变异体与探针待跑 |
| B7 | `scripts/gate/merge-and-verify.sh:680`（d1e14eb0）「读不出 PRODUCT_GPU_PACKAGE_NAME → 照跑」 | 没有一格自证走到这一支、也没有开关；把它改成「没碰、本次未跑」的变异自证照过 | 低 | 推的（逐格看自证用的那一道） |
| B8 | `.claude/main-agent.md:55`（d3fb6fd6）验证段第 1 步仍写「补丁没碰 checker 档包与它依赖的包时报本次未跑」 | 第 32 行改了、这一行没改；共用约束点名照这一步跑 | 低 | 推的（逐字比） |
| B9 | `.claude/rules/verification.md:63`（0038f835）末句：feature-gated-tests 格「里面自己经 gpu-quiesce-run.sh 起」 | 与同一份第 35 行（这一批改的「服务交的卡」）、`commit-stages.md:23`（这一批改的「清场由服务在最外层做」）与门禁文件头相反 | 低到中 | 推的（逐字比） |
| B10 | `.claude/skills/crash-test/commit-stages.md:23`（1581ca99）新句「清场由服务在最外层做（驱动文件头 `# task-server-gpu-arguments: *`）」与同一行「没卡或不清场的那一台加 `--skip-gpu`」 | 写了 `*`，服务对带 `--skip-gpu` 的那一片照样认要卡、分卡、清场：没卡的那台派不到，「不清场的那一台」照样被清场 | 低到中 | 推的（代码逐行） |
| B11 | `.claude/agents/mutation-triage.md:15`（4c81979a）「`mutation-sampling.md` 全篇（分类以它为准）」 | 正文已列的「六栏」还在那份规则里；分诊定义自己说以规则为准，第 32 行改的九栏照它自己的优先级被盖掉 | 低 | 推的（逐字比） |

## 一、三处说法对不上（B1、B8、B9、B11）

### B1 设计员定义末句仍描述改前的门禁（中低）

句子：「门禁 doc-experiments 的 preregistration-judge-features 格按源文件里有没有 `crash_amplification::`、`crash_state_enumeration::` 判特性」（`.claude/agents/experiment-designer.md:31`，ee574a46），同一句接着叫设计员把「源文件写了这两个模块名而不进判段、经别名导入而进判段」的写进报告交主 agent。

门禁此刻：「③ 源文件调了判段入口的算走崩溃判段」（`.claude/gate.d/doc-experiments.sh:2910`，690a35ef）；按模块名认只剩弄坏开关那一支：「if "judge-by-module-marker" in broken:」（`.claude/gate.d/doc-experiments.sh:2999`）。

定义那一句是定义工具员照改前门禁写的（修补报告 `d137-ledger-defs-r1-fix-defs/result.md` 第 1 条「留下的不一致」写明门禁不在它射程），脚本工具员随后改了门禁，两件修补之间没回头对。

照做会发生什么（推的）：
- 设计员遇到 e170、e184 这类只取崩溃点的装置，照判段入口不带 gpu（对的），再照末句把「门禁会因模块名判红」写进报告交主 agent；门禁此刻判它不走判段、不报。主 agent 收到一条不存在的不一致。
- 读定义的人（设计员、执行员、下一轮的腿）以为门禁还按模块名认，别名导入那一类会以为门禁漏判而另想办法。
- 两边的入口名单也不是同一份：定义列 `run_from_environment` 一族、`GivenStatesEngine`、`judge_not_run_in_this_binary`、`judging_test_not_run_without_the_gpu_feature` 与「crash_state_enumeration 里走崩溃点重放判段的入口」；门禁 `JUDGE_ENTRIES`（`.claude/gate.d/doc-experiments.sh:2932–2934`）另有 `crash_point_replay_judge_not_run_line`，那是只拼一行字的函数（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:187`），不选判段去路。只调它的 bin 照定义不算走判段、照门禁要 gpu。此刻 26 份 checker 档 bin 里调它的只有 e161，e161 同时调 `enumerate_crash_states_with_expansion`，两边判的一样（量的：对 `crates/singlefs-checker-tier/src/bin/*.rs` 逐份 `grep -owE` 入口名与 `grep -cwE` 模块名）。

四句：
- 分不分辨改法：分辨。改法甲「删掉末句」、乙「末句改写成按 `JUDGE_ENTRIES` 认、名单指到门禁那一行」都修；丙「只改门禁」不修（门禁已经是对的）。
- 设计员当时看不看得到：看得到门禁文件头，但定义叫它照定义写，它没有理由去翻门禁。
- 满足的分句：交付要求「新定义与新脚本、新门禁三处的说法对不对得上」那一项。
- 跑前条款给的改法：判决「改法与去向」只写了「`.claude/gate.d/doc-experiments.sh` 那一格认续行」，没写定义末句跟着门禁改；两件修补各按自己的派发做完，这一格没人修。

推翻它的现象：门禁那一格此刻的判法仍按模块名（不开弄坏开关时第 2999 行那一支被走到）。

### B8 验证段第 1 步没跟着补产品 GPU 包（低）

第 32 行改成了「补丁没碰 checker 档包、它依赖的包与产品 GPU 包 `singlefs-background-gpu` 时报本次未跑」（`.claude/main-agent.md:32`，d3fb6fd6）；同一份第 55 行（「写与验分两段」验证段第 1 步）仍是「补丁没碰 checker 档包与它依赖的包时报本次未跑」（`.claude/main-agent.md:55`）。共用约束点名的是第 55 行那一步（「写与验分两段」验证段第 1 步）：`.claude/agent-common.md:64`：「验证段第 1 步」（sha256 前 8 位 7abe0915）。

脚本自证的成功句也还是旧说法：「补丁没碰 checker 档包与它的依赖闭包时报本次未跑」（`scripts/gate/merge-and-verify.sh:1540`，d1e14eb0），同一句后半另接了产品 GPU 包那一格，前后两半各说一套。

照做会发生什么（推的）：主 agent 照第 55 行判「只改产品 GPU 包的补丁快档本次未跑」，而脚本此刻照跑那一格；结局以脚本为准，不会漏跑，只是主 agent 预期与输出对不上，红了时会以为脚本多跑了。

四句：只改字句；主 agent 看得到脚本输出；满足的是「该跟改而没改」；改法「第 55 行照第 32 行补半句」修，脚本成功句前半同改。

### B9 `verification.md` 末句说门禁格自己清场（低到中）

这一批改了第 35 行：「带 `gpu` 特性那一趟与 product-gpu-real-card-tests 的对拍那一趟开跑前核环境里有服务交的卡」（`.claude/rules/verification.md:35`，0038f835）；主 agent 同批把 `commit-stages.md` 第 23 行改成「清场由服务在最外层做（驱动文件头 `# task-server-gpu-arguments: *`）」（`.claude/skills/crash-test/commit-stages.md:23`，1581ca99）。

同一份 `verification.md` 第 63 行末句没动：「与门禁 `checker-tier-crash-point-replay` 的 feature-gated-tests 格里面自己经 `scripts/crash/gpu-quiesce-run.sh` 起」（`.claude/rules/verification.md:63`）。门禁文件头说的是相反的：「这一道不自己清场、不挑卡」（`.claude/gate.d/checker-tier-crash-point-replay.sh:187`，e8ce42b9），要卡那一处打的行是「（清场归 task-server，这里不另套清场包装）」（`.claude/gate.d/checker-tier-crash-point-replay.sh:2551`）；门禁里起 `gpu-quiesce-run.sh` 的只有弄坏开关 `stage-quiesces-by-itself` 那一支（同一份第 2545–2547 行）。崩溃放量入口同样：`scripts/crash/crash-amplification.sh` 文件头第 34 行写清场与分卡归 task-server。

另一处小的：第 35 行新句只点了两趟要核卡，门禁核卡的是三处：「要上卡的三处（查库那一批里有带 gpu 特性的命令、feature-gated-tests 带 gpu 特性那一趟、product-gpu-real-card-tests 的对拍那一趟）」（`.claude/gate.d/checker-tier-crash-point-replay.sh:189`）。查库那一批没交卡同样判红，规则里没写。

照做会发生什么（推的）：照第 63 行末句，有人在服务外直接起崩溃放量入口或门禁，以为它们自己会清场；它们此刻不清场，第 35 行说服务没交卡那一格判红、那一趟不起，于是在服务外起是红而不是「自己清场再跑」。规则里同一件事两种说法，读的人挑哪句全看先读到哪句。

四句：只改字句；被判对象（读规则的 agent）看得到两句，分不出哪句是现状；满足的是「该跟改而没改」（同一份文件里这一批改过的那一行的近邻）；改法「删第 63 行末句、改成门禁与入口都不自己清场、经服务交」修。

### B11 分诊定义自己写了以规则为准（低）

正文第二节已列 `.claude/rules/mutation-sampling.md` 第 79 行还写六栏，这里只补一个它会盖掉修补的理由：分诊定义开工先读那一行是「`.claude/rules/mutation-sampling.md` 全篇（分类以它为准）」（`.claude/agents/mutation-triage.md:15`，4c81979a），规则那一句是「报抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏」（`.claude/rules/mutation-sampling.md:79`，0ebfe2eb，不在快照里），定义第 32 行改成的是「报九栏：抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏，后接判定库那一侧的现判、复用、拒算三栏」（`.claude/agents/mutation-triage.md:32`）。两句冲突时定义自己判规则赢，九栏那一改照字面不生效。改法：规则那一句同改（正文已列），不需另改定义。

## 二、新代码与新自证分不分得出改前改后（B4、B5、B6、B7）

### B4 续行并接把注释行与下一条命令并成一条（低）

新代码：围栏里以没转义的反斜杠收尾的行，与下一行并成一条再认：「line = line[:-1] + lines[following]」（`.claude/gate.d/doc-experiments.sh:3086`，690a35ef），判不判续行只看行尾反斜杠的奇偶（同一份第 2994 行），不看这一行是不是注释。并好的行再认命令位置，命令起头只认这些字：「COMMAND_START = re.compile(r"[A-Za-z0-9_./~$'\"(]")」（`.claude/gate.d/doc-experiments.sh:2955`），`#` 不在里面。

bash 不给注释续行：以反斜杠收尾的注释行，下一行照样单独当一条命令跑（起草时在本机 `bash -c $'# 注释行以反斜杠收尾 \\\necho second-line-ran'` 打出 `second-line-ran`，量的；同一条进了 `tools/rerun.sh`，复跑打 `name=bash_comment_backslash second_line_ran=yes`）。

世界（`worlds/`，推的读数在 `out/expected.txt` 第二段）：
- comment-backslash-swallows-command：围栏里头一行 `# 可行性档，下一行起，没带 gpu 特性 \`，下一行起走判段的 e990 没带 gpu。门禁把两行并成一条、以 `#` 起头、不在命令位置，一条都没判，退 77；照 bash 该判红。
- comment-backslash-hides-second-command：同一围栏里另有一条带 gpu 的判了、绿；第二条被并进注释，整格判绿（退 0）；bash 照样跑那条没带 gpu 的。
- comment-without-backslash-control：注释不以反斜杠收尾，下一行单独认、判红（退 1）。只差一个反斜杠，说明前两格的差别全来自新加的并接。

改前（逐行认）这三格都判红，所以这是修补新开的一个漏口。现实里注释行以反斜杠收尾少见（把一条续行命令注释掉一半时会出现），强度给低。

四句：
- 分不分辨改法：分辨。改法甲「并接前跳过 `lstrip()` 后以 `#` 起头的行」修三格；乙「并好之后再按 bash 的注释规则剥注释」同样修；丙「只加样本」不修。
- 门禁当时看得到判别它的东西：看得到，注释行就在那一行。
- 满足的分句：门禁文件头判据「围栏里以没转义的反斜杠收尾的行（shell 续行）先与下一行并成一条再认」——注释行尾的反斜杠在 shell 里不是续行，判据字面把它算成了续行。
- 弄坏开关 continuation-unjoined 管不到这一格：开关照改前逐行认，三格都判红，与「对」的读法一致，开关分不出这一种错。

推翻它的现象：主 agent 跑 `judge-worlds.sh`，前两格的 rc 是 1。

### B5 特性的几种写法：门禁认的、编装置的脚本收的、定义说的三处不是同一套（低）

门禁认的写法：「特性（--features / --features= / -F，可写几次，逗号或空白分隔；--all-features 算带上）」（`.claude/gate.d/doc-experiments.sh:2914`），绿样本自己就用了几种：「三处走判段的 bin 各用 --features verdict-store,gpu、--features=gpu、-F gpu」（`.claude/gate.d/doc-experiments.sh:2921`）。

定义的说法：「命令末尾加 `--features gpu`，照跑前登记里起它那条命令的 `--features` 原样给」（`.claude/agents/experiment-runner.md:32`，4a8e5c16）。

编装置的脚本收的：参数表里同名的后写盖前写，「options[argument] = arguments[position + 1]」（`scripts/hosts/compile-then-swap.py:839`，22ced829）；`--features=gpu`、`-F`、`--all-features` 都不在参数表里，落进位置参数，位置参数不是两个就退 2；之后「features = tuple(options["--features"].replace(",", " ").split())」（`scripts/hosts/compile-then-swap.py:859`）。

照做会发生什么（推的；探针 `tools/cli-features-probe.py` 对真脚本逐种调它的 `main()`，期望在 `out/expected.txt` 第三段）：
- 登记写 `--features=gpu`、`-F gpu` 或 `--all-features`：门禁绿；执行员「原样给」，主 agent 交的 `compile-then-swap.py` 退 2、没编。出声，不漏，多一轮续派。
- 登记写 `--features gpu --features <别的>`：门禁绿（特性按集合并）；原样给之后只剩后一个，二进制没有 gpu，判段照旧 NOT_RUN。不出声。此刻 checker 档包只有 `gpu` 一个特性（`crates/singlefs-checker-tier/Cargo.toml` 的 `[features]` 只有一行，量的），这一种要有人写两个 `--features` 才会遇到，强度给低。

四句：分辨改法——甲「脚本收与门禁同一套写法（`=`、`-F`、可写几次、`--all-features`）」修全部；乙「定义改成「写成一个 `--features <逗号连起来>`」」修定义那一侧，登记里照写 `-F` 的仍退 2；脚本当时看得到参数原文；满足的分句是交付要求点名的「定义里 `compile-then-swap.py --features` 的写法」；跑前条款没写写法。

### B6 新自证项碰不到命令行那一层（中）

新自证项直接调 `swap()`，不经命令行解析：「BOTH_CARGO_COMMANDS, features=("gpu",))」（`scripts/hosts/compile-then-swap.py:726`）经「code, lines = swap(draft_path, target, fixture.scratch, fixture.root, after_build=after_build, after_swap=after_swap,」（`scripts/hosts/compile-then-swap.py:577`）。命令行 `--features` 到 `swap(features=…)` 那一段（第 859–861 行）没有一项自证走到；弄坏开关 features-dropped 也只弄坏 `build()` 里那一处（第 306 行），不碰命令行。

主 agent 真用的是命令行：执行员第 2 步 ④ 写的整条 `task-server/task submit -- python3 scripts/hosts/compile-then-swap.py <草稿副本> … --release --features gpu`。

变异体（`tools/rerun.sh` 造在 `<--work>/mutant/scripts/hosts/compile-then-swap.py`）：只把第 859 行换成 `features = ()`，即命令行上的 `--features` 解析完就丢——正是第一轮 A1 那种「编法不带特性」在命令行这一层重现。推的读数：探针对变异体调 `main(… --features gpu)`，假 cargo 两次调用都不带 `--features`；同一进程里调变异体的 `selftest()` 退 0（16 项全过）。

四句：
- 分不分辨改法：分辨。甲「加一项经 `main()` 起的自证（假 cargo 照收调用参数）」修；乙「features-dropped 开关挪到命令行解析那一处」让开关打中命令行，但没有对着命令行的自证项，开关照样只红 gpu-feature-passed-through（它不经命令行，改了开关位置之后它反而不红）——乙要与甲一起做。
- 自证当时看得到：看不到，它不起命令行。
- 满足的分句：交付要求「新开关与新自证格分不分得出改前改后」——在 `swap()` 这一层分得出，在命令行这一层分不出。
- 推翻它的现象：变异体的 `selftest()` 退 1，或探针里变异体的 features-gpu 那一行 `gpu_in_every_call=yes`。

### B7 合入后验证「读不出包名就照跑」那一支没人证（低）

新代码：「return "affected", f"算不出补丁碰没碰门禁 product-gpu-real-card-tests 格守的产品 GPU 包（{guarded_problem}），照跑"」（`scripts/gate/merge-and-verify.sh:680`，d1e14eb0）。

自证里每一格用的那一道都有那一行：假的那一道写着「PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu"」（`scripts/gate/merge-and-verify.sh:1025`），由「export MERGE_AND_VERIFY_CHECKER_QUICK_STAGE="$scratch/fake-checker-quick.sh"」（`scripts/gate/merge-and-verify.sh:1030`）设给全部格；quick-judge、quick-reuse、quick-red 三格换成真的那一道（第 1432–1444 行），真那一道第 600 行同样有这一行；第 1516 行那一格用快照里的真那一道，也有。没有一格的那一道缺这一行，弄坏开关表（文件头第 210–223 行）也没有对应的一条。

变异：第 680 行改成 `return "unaffected", …`（读不出就当没碰）——推的：自证照过（没有一格走到这一支）。后果只在真那一道把这一行改名或改写法（例如写成 `readonly PRODUCT_GPU_PACKAGE_NAME=…`、或包名改由函数算）时出现：只改产品 GPU 包的补丁那一格又一次都不跑，正是第一轮 A10 那个洞，而自证不红。

四句：分辨改法——加一格「那一道里没有这一行 → 照跑、报原因」与一条开关就修；脚本当时看得到那一道的正文；满足的分句同 B6；没造世界（要编 prune-server、起临时服务，代价大；各格用的是哪一份那一道、那一份有没有这一行，上面三处行号已经够判）。

## 三、照新定义做会交出判不对、或越过别的规则的东西（B2、B3、B10）

### B2 入库装置带 gpu 编之后，判段在服务没分卡、没清场的环境里真上卡（中）

与第一轮 A1 差在哪：第一轮 A1 打的是「编法不带特性，判段只打 NOT_RUN」，改法甲（给 `compile-then-swap.py` 加特性）已预告「还中一半」。判决只采了甲；这里攻的是甲落地之后这条链的新去向，补的是第一轮没核的代码路径：二进制拿不到服务的卡时不报错，自己挑卡。

照新定义走一遍：
- 编：「命令末尾加 `--features gpu`，照跑前登记里起它那条命令的 `--features` 原样给」（`.claude/agents/experiment-runner.md:32`，4a8e5c16），二进制编进 gpu。
- 跑：「执行员交回的产物命令只执行这一份（写成 `task-server/task submit -- <二进制…>`）」（`.claude/agents/experiment-runner.md:30`）。
- 服务认不认要卡：「GPU：认命令（gpu_need：带上卡特性」（`task-server/task_server.py:181`，c84e1739），认的是带 gpu 特性的 cargo、按名字认的几份入口、python 入口与文件头声明了的脚本（同一份第 1450–1472 行 `recognize_gpu`），裸二进制不在里面：不分卡、不清场、环境里没有 `GPU_CARDS_BY_PRIORITY`。
- 二进制里：配置开着 GPU、编进了 gpu，判段走 GPU 路；卡从哪来：「(None, GpuQuiesce::NotQuiesced) => configured_value(CARDS_BY_PRIORITY_KEY),」（`crates/singlefs-checker-tier/src/gpu_card_configuration.rs:394`，33c81935）——环境里没有、又没清场，就读多机配置里不清场那一份卡单；那一份也没写就「本机能用的卡按空闲显存从大到小用」（同一份第 343 行文档）。crates 里找不到「要服务交的卡」那种核法（对 `crates/` 下 `.rs` 搜 `require_service_card`、`TASK_SERVER_TASK` 都是 0 处，量的）。

照做会发生什么（推的）：修补前这条链在开着 GPU 的配置下打 NOT_RUN、执行员照第 4c 步报本次未跑（出声、不判）；修补后同一条链真上卡判，而服务的账上那张卡是空的，下一件服务派的 GPU 任务（门禁快档带 gpu 那一趟、崩溃放量、广谱变异 GPU 趟）可能同时落到同一张卡上；也不清场。这与「**凡用 GPU 先清场，跑完不复原**，清场由服务做」（`.claude/rules/verification.md:63`，0038f835）相反。出声的本次未跑换成了不出声的抢卡。

四句：
- 分不分辨改法：第一轮的甲（已落地）在这一格上仍中；乙「入库装置的产物命令改经服务交 `cargo run --release -p singlefs-checker-tier --features gpu --bin …`」修（服务照 cargo 认卡、清场）；新提丁「服务认 `*/compile-then-swap/target/*/e*` 下、编译日志里带 `--features gpu` 的二进制要卡」也修——丁只在我的推演上过、被攻过零轮。
- 执行员当时看得到：看不到服务认没认卡（只看得到 `task wait` 末行），也看不到二进制挑了哪张卡，除非装置自己打。
- 满足的分句：交付要求「照新定义做会不会撞拒或交出门禁判不对的东西」——判段结论本身照算，但跑法越过了规则那一条，结论是不是被撞卡扰过判不出。
- 推翻它的现象：服务的 `gpu_need` 认裸二进制要卡（`task-server/task status <号>` 里那一件有分卡与清场的记录），或二进制在拿不到服务交的卡时拒跑。

### B3 「任务日志里数 NOT_RUN」：执行员读得到的日志只有最后 200 行（中低）

新句子：「装置的标准输出（任务日志）与它写出的每份产物（`--out`、`--output-directory` 这类另写的文件也算）逐份数，合计得 0」（`.claude/agents/experiment-runner.md:40`）。

执行员读任务日志只能起 `task-server/task log <号>`（共用约束「子 agent 只读 `task-server/task status` 与主 agent 给了号的 `task-server/task log <号>`」），那条命令默认只给尾巴：「默认最后 LOG_TAIL_LINES 行，--all 整份」（`task-server/task:25`，dfed1f94），「LOG_TAIL_LINES = 200」（`task-server/task:58`）。第 4c 步这一句没写 `--all`（第 3 步跑 `mutate.sh` 那一句写了）。

NOT_RUN 那一行打在判段起头：给定状态表的引擎在开引擎时打一次，「given_states_not_run_line_without_the_gpu_feature(&item.item_name, reason)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:18609`，672e599d），之后装置照常往下打它自己的结果行；崩溃放量那一路同样在录入之前打（同一份第 17509–17513 行）。走判段的装置产物有很长的：`research/results/e168-delete-ring-window-borrow-segment1-2026-10-03.out` 177054 行（`wc -l`，量的）。

照做会发生什么（推的）：标准输出进任务日志、产物另写文件的装置，判段没跑而后面又打了 200 行以上，执行员照第 4c 步数到 0，当判段跑了，产物照绿交。

四句：分辨改法——甲「这一句写明 `task-server/task log <号> --all`」修；乙「改成数装置自己的退出码或末尾汇总行」要装置配合；执行员当时看得到的只有尾巴，看不到开头；满足的是第 4c 步自己的「合计得 0」；推翻它的现象：`task-server/task log` 不带 `--all` 也打整份。

### B10 `*` 让带 `--skip-gpu` 的那一片照样要卡、照样被清场（低到中）

新句：「清场由服务在最外层做（驱动文件头 `# task-server-gpu-arguments: *`）」（`.claude/skills/crash-test/commit-stages.md:23`，1581ca99）。同一行没改的一句：「没卡或不清场的那一台加 `--skip-gpu`」（同一行）。

`*` 的判法：「return "*" in declared or first_non_option(arguments) in declared」（`task-server/task_server_lib.py:1913`，14be8f61）——写了 `*` 就不看参数；免卡的参数只有三个：「GPU_NOT_RUN_ARGUMENTS = ("--list-items", "--help", "-h")」（`task-server/task_server.py:301`），没有 `--skip-gpu`。驱动文件头自己写的是「# task-server-gpu-arguments: *   不带 --skip-gpu 就有带 gpu 特性的那一趟」（`scripts/mutation/broad-mutation.py:4`，9b29e7af），意思是带了 `--skip-gpu` 就没有，声明却没表达这一半。

照做会发生什么（推的）：两台各跑一片时，带 `--skip-gpu` 的那一片服务照样认要卡：服务挑台时只挑有空卡的那台（多机配置没点名卡的那台派不到，`task-server/task_server.py:3421` 那一支记 run-condition-unmet），派到之后照样套清场。「没卡的那一台」跑不到这一片；「不清场的那一台」被清场、跑完不复原。两片都要卡，有卡的那台只有一张空卡时两片排队、谈不上两台并行。

四句：分辨改法——甲「驱动文件头的 `*` 改成按参数声明，或服务认 `--skip-gpu` 免卡」修；乙「删掉 commit-stages 那半句」只让字句不再承诺做不到的事；服务当时看得到 `--skip-gpu` 这个参数；满足的分句是「新句子与同一行旧句子相冲」；推翻它的现象：`task-server/task status` 里带 `--skip-gpu` 的那一件没有分卡与清场。

## 要主 agent 跑的命令

都不是重型；子 agent 不交，由主 agent 经服务交。`<W>` = `/home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-opus`（腿的草稿目录，在工作根下、两台同一路径）。期望逐行在 `research/prompts/d137-ledger-defs-r2-opus-model/out/expected.txt`。

1. 造样本根与变异体（模型目录的 `tools/rerun.sh` 整份跑一次；只起 bash 与 git、拷文件，不起本仓的代码）：
   `task-server/task submit -- bash research/prompts/d137-ledger-defs-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-opus/rerun-1`
   在哪起：仓根 `/home/fy5090/code/singlefs`（不给 `--cwd`）。
   看结果：末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号>` 里 `name=bash_comment_backslash second_line_ran=yes`、三行 `name=world_built … untracked=1`、`name=mutant_built source_sha256=22ced829 … changed_lines=2`（`source_sha256` 不是 22ced829 的，`compile-then-swap.py` 在开工之后改过，下面第 3、4 条的读数与本报告对不上时先看它）。

2. 三个样本根上单跑门禁那一格（B4）：
   `task-server/task submit -- bash research/prompts/d137-ledger-defs-r2-opus-model/tools/judge-worlds.sh /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-opus/rerun-1/worlds`
   在哪起：仓根（脚本要在仓根起；门禁那一行的范围声明写在脚本里那一行）。
   看结果：`exit=0`（三格都跑了，不判对错）；`name=judge_inputs gate_sha256=690a35ef`；三行 `name=judge_world` 的 `rc=` 依次 77、0、1 与 `out/expected.txt` 第二段第二列逐行相同，B4 成立；前两格是 1，B4 不成立。第三格（对照）不是 1，这一批作废。

3. `compile-then-swap.py` 命令行探针，对真脚本（B5，外加 B6 的对照）：
   `task-server/task submit -- python3 research/prompts/d137-ledger-defs-r2-opus-model/tools/cli-features-probe.py --script scripts/hosts/compile-then-swap.py --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-opus/probe-real --selftest`
   在哪起：仓根。不起真编译（假 cargo 放在 PATH 最前面，照那份脚本自己的自证搭法）；`--selftest` 在进程里调它的 `selftest()`，自证项各自的临时目录照它原样建在 `${TMPDIR:-/tmp}` 下、跑完自删。
   看结果：`exit=0`；七行 `name=cli_features` 的 `rc`、`calls`、`gpu_in_every_call` 与 `out/expected.txt` 第三段逐行相同；末行 `name=selftest_in_process rc=0`。features-twice-gpu-first 那一行 `gpu_in_every_call=no` 是 B5 不出声的那一半；三种 `rc=2` 是出声的那一半。

4. 同一探针对变异体（B6）：
   `task-server/task submit -- python3 research/prompts/d137-ledger-defs-r2-opus-model/tools/cli-features-probe.py --script /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-opus/rerun-1/mutant/scripts/hosts/compile-then-swap.py --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r2-opus/probe-mutant --selftest`
   在哪起：仓根。要第 1 条先跑完。
   看结果：`exit=0`；features-gpu 与 features-comma 两行 `gpu_in_every_call=no`；末行 `name=selftest_in_process rc=0`。两样同时出现，B6 成立（变异体把命令行上的特性丢了，自证照过）；`rc=1`，B6 不成立。

跑完之后草稿目录 `<W>/rerun-1`（样本根、变异体）、`<W>/probe-real`、`<W>/probe-mutant` 由主 agent 照判决复跑那一步的规矩删。

## 没打中的形状

- 门禁判段入口表 `JUDGE_ENTRIES` 漏没漏：对 `crash_amplification.rs`、`crash_state_enumeration.rs` 里 `cfg!(feature = "gpu")` 与 `uses_gpu()` 的每一处，找到包着它的函数，往上找公开入口：`run_from_environment`、`run_from_environment_with_observers`（第 17509 行那一处）、`GivenStatesEngine::start_from_environment`（第 18605 行那一处）、`enumerate_crash_states_with_every_kind_of_observer` 经 `enumerate_crash_states_through_the_library`（第 1088 行那一处），都在表里；别的模块里有没有转出这些入口（`pub use`）：`crates/singlefs-checker-tier/src/` 下 15 处 `pub use` 没有一处转出入口名；`crash_injection.rs` 第 6109 行起引擎的那个函数在 `#[cfg(test)]` 里。没找到绕开模块名调判段的公开路子。
- 26 份 checker 档 bin 照新认法分：走判段的 6 份（e156、e158、e161、e168、e171 两份），与脚本修补报告列的相同；e185 有 33 处 `cfg(feature = "gpu")` 而不调判段入口，它自己的 GPU 段与崩溃判段无关，不算漏。
- 合入后验证读包名：产品 GPU 包在工作区成员里；它的依赖只有 `singlefs-core`（与 wgpu、ash、libc），`singlefs-core` 已在 checker 档闭包里，产品闭包只多它自己一个；快照里有那一道（快照整份拷工作区）；`merge-and-verify.sh` 在服务按名字认要卡的名单里（`task-server/task_server.py:300`），产品那一格拿得到卡；正则对得上那一道第 600 行。没找到漏跑。
- 复跑表 `replay.sh` 起入库装置：配置开着 GPU 时一律带 `--features gpu`（`resolve_checker_tier_features`），不随这一批改。
- `compile-then-swap.py --features gpu` 编 checker 档包要的 `BINDGEN_EXTRA_CLANG_ARGS` 在 `.cargo/config.toml` 的 `[env]` 里，那份脚本拷 `.cargo/` 进仓副本，编得到。
- 实现员第 47 行新列的三处加四处，与 `scripts/crash/crash-case-check.py` 文件头 must-red、provenance 两段逐条对过，对得上。
- `mutate.sh` 的另一种「计数：」行（只编不跑，第 2130 行）只在门禁的 mutation-tables-compile 格里出现，分诊收不到它。
- 门禁两条新开关的判别力：continuation-unjoined 下 continued-red 退 77、continued-green 误红；judge-by-module-marker 下 alias-red 第 6 行不报、第 7 行误红——照代码逐行推，两条都分得出改前改后。能活过全部样本的几种变异（并接时不去掉反斜杠、奇偶判法换成 `endswith`、只认入口名不认模块名）后果都只是多要 gpu 或在罕见写法上误红，不列为打中。
- 共用约束第 98 行三种出路的字句与修补报告引的派发闸第 68–71 行、第 1356 行对得上（只比了字句，没起派发闸）。

## 改法各修哪一格

全部只在我的推演上过、被攻过零轮；没有一格在副本上跑过，都标「推的」。

| 改法 | B1 | B2 | B3 | B4 | B5 | B6 | B7 | B8 | B9 | B10 | B11 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 设计员第 31 行末句删掉或改成按 `JUDGE_ENTRIES` 认 | 修（推的） | — | — | — | — | — | — | — | — | — | — |
| 入库装置产物命令改经服务交 `cargo run … --features gpu` | — | 修（推的） | — | — | — | — | — | — | — | — | — |
| 服务认 compile-then-swap 编出的带 gpu 二进制要卡 | — | 修（推的） | — | — | — | — | — | — | — | — | — |
| 第 4c 步写明 `task-server/task log <号> --all` | — | — | 修（推的） | — | — | — | — | — | — | — | — |
| 并接前跳过以 `#` 起头的行 | — | — | — | 修（推的，待主 agent 跑第 2 条看现状） | — | — | — | — | — | — | — |
| `compile-then-swap.py` 收门禁认的全部写法 | — | — | — | — | 修（推的） | — | — | — | — | — | — |
| `compile-then-swap.py` 加一项经 `main()` 的自证 | — | — | — | — | 只在加了写法样本时修（推的） | 修（推的，待主 agent 跑第 4 条看现状） | — | — | — | — | — |
| 合入后验证加「那一道没有包名行」一格与一条开关 | — | — | — | — | — | — | 修（推的） | — | — | — | — |
| 主 agent 第 55 行、脚本成功句前半补产品 GPU 包 | — | — | — | — | — | — | — | 修（推的） | — | — | — |
| `verification.md` 第 63 行删末句、第 35 行补查库那一处 | — | — | — | — | — | — | — | — | 修（推的） | — | — |
| 驱动文件头的 `*` 改成按参数声明，或服务认 `--skip-gpu` 免卡 | — | 不碰 | — | — | — | — | — | — | — | 修（推的） | — |
| 删 commit-stages 第 23 行那半句 | — | — | — | — | — | — | — | — | — | 只修字句（推的） | — |
| `mutation-sampling.md` 第 79 行同改九栏（正文已列） | — | — | — | — | — | — | — | — | — | — | 修（推的） |

## 这条腿自己的限度

- 一条命令都没跑过：门禁、探针、变异体的读数全是照代码逐行推的，等主 agent 跑「要主 agent 跑的命令」第 1–4 条。B4、B5、B6 有世界或探针；B2、B3、B7、B10 没造世界（B2 要编 checker 档 release 二进制、要卡；B7 要编 prune-server、起临时服务；B10 要往服务交带 `--skip-gpu` 的广谱变异；B3 要一个判段打 NOT_RUN 后又打 200 行以上的装置），只有代码行。
- B2 的「撞卡」是推的：服务派的下一件 GPU 任务是不是真落在同一张卡上、两边会不会出错，没量过；只核到「服务账上那张卡是空的、二进制自己挑卡」这一步。
- B3 的「200 行之外」依赖装置的输出长度；只举了一份走判段装置的产物行数，没逐个装置数它判段起头之后还打多少行。
- `.claude/main-agent.md` 只读了第 32、55、101 行；共用约束第 98 行只比了字句。别的会话此刻在改的文件（`scripts/mutation/crates-mutation-rows.py`，判决 A12 前一半的去处）没看。
- 定义第 3c 步并行收退出码、第 3b 步的崩溃状态枚举都不适用：模型里没有并行件，也没有枚举。

## 没做什么

- 不判辩方那一格（D），没读辩方报告（禁读）；不替主 agent 采纳。
- 没往统一任务服务交任务，没跑重型，没跑任何门禁、自证、探针、模型；判决要引的数没复跑（命令在「要主 agent 跑的命令」）。
- 没改被判文件；只写了本报告、模型目录 `research/prompts/d137-ledger-defs-r2-opus-model/` 与草稿目录（`rules.txt`、`now.txt`、`rerun-steps.txt`、`rerun.sh` 四份草稿；没建仓副本、没建编译目录）。
- 第一轮判决里留作欠账的两样（门禁另一种本次未跑、只在共用约束括注里提到的三份规则）没攻：前一样的去处文件别的会话正在改；后一样不是这一批修补写出来的句子。
