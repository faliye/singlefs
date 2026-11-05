# d137-ledger-defs-r1 攻方腿报告（格 A：照字面做会出错的边角、这一批该改而没改的句子）

日期：2026-11-02。腿：three-way-attack（Opus）。背景材料：`research/prompts/_d137-ledger-defs-r1-body.md`。前几轮判决：无。
被判文件此刻的 sha256 与快照 `research/prompts/d137-ledger-defs-r1-snapshot/files-sha256.txt` 逐份相同（开工时 `sha256sum … | diff - files-sha256.txt` 无输出）。
只攻正文第一节六条改动写出来的句子与它们没带上的句子；做法本身（子 agent 不交任务、经服务交、判定库一份）不攻。

## 复跑命令与模型文件

模型目录 `research/prompts/d137-ledger-defs-r1-opus-model/`：七个样本根（`worlds/`），造根的 `tools/make-worlds.sh`，单跑门禁一格的 `tools/judge-worlds.sh`，期望 `out/expected.txt`，喂钩子的输入 `inputs/`，`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷、只改了 `rerun_steps`（只造样本根；单跑门禁那一步另交，见「要主 agent 跑的命令」第 1、2 条）。
模型里的门禁退出码全是推的：腿没往服务交任务，一个都没跑过。

```
9eed87d6b10af2e2f40944114a51169268689640b63c53484c8a668fbdfbb6d0  ./inputs/run-scope-designer-literal.json
763fd4a462b07bd86ecfa2e4b626886654cb8cb9d2a83499d014e85388a02ab6  ./out/expected.txt
a867af27b34816af5f0e8663da752bb7f9f699300c771bd09d85f94ac494a643  ./tools/judge-worlds.sh
e9a3f105996b576ff759ca402c998bdd7168ccf548c772f0091f0b79bd1a6077  ./tools/make-worlds.sh
c7ad3e8b7ef73aff9a19f8a5d5af1d4298bd4d4a23fbeb6d8a2c301de89be709  ./tools/rerun.sh
4f3c310fb6c09ecfa8a4b999ce82ddfe8b6a4ef71948c25f1ebd17972516441f  ./worlds/alias-import-judge/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/alias-import-judge/before/crates/singlefs-checker-tier/Cargo.toml
b670343f22eaec19b415f39a43540f3454b52facf25e4ee1c6ed982b9d74245c  ./worlds/alias-import-judge/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/alias-import-judge/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/alias-import-judge/before/research/prompts/README.md
f59d6c0af74bfb6f840e97b103c6e7f374e08c7f78b5e64ff6078c977bda6884  ./worlds/continued-bin-on-next-line/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/continued-bin-on-next-line/before/crates/singlefs-checker-tier/Cargo.toml
07017964b340d25358ab069818a4c9cefb0b427f11ac31a6338e772f738c053a  ./worlds/continued-bin-on-next-line/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/continued-bin-on-next-line/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/continued-bin-on-next-line/before/research/prompts/README.md
276d19a27b8e294baa3aa77b19c7a2c27f098f3680f5de8b37b2287466f471af  ./worlds/continued-features-on-next-line/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/continued-features-on-next-line/before/crates/singlefs-checker-tier/Cargo.toml
07017964b340d25358ab069818a4c9cefb0b427f11ac31a6338e772f738c053a  ./worlds/continued-features-on-next-line/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/continued-features-on-next-line/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/continued-features-on-next-line/before/research/prompts/README.md
1e3c79c6412484ac687de6baff8f19abc19d182995bf8722e63cee71d778c34c  ./worlds/control-single-line-green/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/control-single-line-green/before/crates/singlefs-checker-tier/Cargo.toml
07017964b340d25358ab069818a4c9cefb0b427f11ac31a6338e772f738c053a  ./worlds/control-single-line-green/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/control-single-line-green/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/control-single-line-green/before/research/prompts/README.md
d645b57c9d19b42c18955eff9df85223132ff3c78fb8f8a92b9ef7a997689c0e  ./worlds/control-single-line-red/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/control-single-line-red/before/crates/singlefs-checker-tier/Cargo.toml
07017964b340d25358ab069818a4c9cefb0b427f11ac31a6338e772f738c053a  ./worlds/control-single-line-red/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/control-single-line-red/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/control-single-line-red/before/research/prompts/README.md
c55e84d29256fba2b0d7f23977857cb3f5e586965c30766e87e97a96fe6fb259  ./worlds/declared-cargo-test-green/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/declared-cargo-test-green/before/crates/singlefs-checker-tier/Cargo.toml
07017964b340d25358ab069818a4c9cefb0b427f11ac31a6338e772f738c053a  ./worlds/declared-cargo-test-green/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/declared-cargo-test-green/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/declared-cargo-test-green/before/research/prompts/README.md
fa8339ee67e5f14ea62b1360201f2c52e9f1e5e36f685dad60b95c25775c6c07  ./worlds/marker-without-judge/after/research/prompts/e990-preregistration.md
a553a644682e22c919fb1c125b5bf4e23b964b1021f9086a4d29ae9fac58e5da  ./worlds/marker-without-judge/before/crates/singlefs-checker-tier/Cargo.toml
b1325516ac243ed16d4139fbb186f56cbff0ae4349bfb572ae389d12a48a8e73  ./worlds/marker-without-judge/before/crates/singlefs-checker-tier/src/bin/e990_judge_sample.rs
cd5925c268e7893f9fe597f1dd1d701d4c9389127b42a73b927c07eae05b8c68  ./worlds/marker-without-judge/before/.gate-cells
bda71e6bb51aa8063e5bc26f6ef07a9c4f61f57359eae578726423480bb11d95  ./worlds/marker-without-judge/before/research/prompts/README.md
```

## 各格判定一览

格 A 只有一格，下面按打中的处所拆成十三处；「强弱」是腿自己的估计，「依据」一列标读代码推的还是量过的（这一轮一个都没量，全是推的）。

| 编号 | 改动条目 | 打中什么 | 强弱 | 依据 |
|---|---|---|---|---|
| A1 | 一百二十六 ①② | 走判段的装置只能是入库装置，而执行员定义里入库装置的产物照 `compile-then-swap.py` 编的二进制跑，那一份编法不带任何特性：照新句子登记 `cargo run --features gpu`、执行员照第 1b 步跑二进制，判段在开了 GPU 的配置下一律打 NOT_RUN；执行员新加的核对「起它的 cargo run / cargo test 带 --features gpu」在这一支没有对象 | 强 | 推的（代码与定义逐行） |
| A2 | 一百二十六 ① | 「装置走到崩溃判段（源文件里有 …）」的括注比「走到判段」宽：e184、e170 只取崩溃点、只在 CPU 上取样，照字面也要带 gpu，服务因此给它分卡、清场、跑完不复原，清场回读没过退 78，一个纯 CPU 计数装置被判本次未跑；反方向，`use … crash_amplification as …` 的别名导入走判段而括注认不出 | 中 | 推的；样本根 `marker-without-judge`、`alias-import-judge` 待跑 |
| A3 | 一百二十六 ① | 定义说「门禁 doc-experiments 的 preregistration-judge-features 格判特性」，那一格按行认命令：反斜杠续行把 `--bin` 拆到下一行时一条都不判（退 77），把 `--features gpu` 拆到下一行时误红 | 中 | 推的；样本根 `continued-*` 待跑 |
| A4 | 一百二十六 ①② | 「产物里 `grep -c NOT_RUN` 得 0」：NOT_RUN 那两行打在标准输出，产物是装置另写的文件（`--output-directory`、`--out`）或 `.gz` 时，数到 0 而判段没跑 | 中 | 推的 |
| A5 | 一百二十六 ① | 「整条写成 `task-server/task submit -- <命令>`」没带范围声明；`cargo test` 那一种照范围声明闸要声明，照字面抄的整条被拒 | 低 | 推的；喂钩子的输入在 `inputs/` 待跑 |
| A6 | `agent-common.md` 删半句 | 删完之后同一句仍写「退 2 的拒绝说明分两种出路」，派发闸此刻分三条（多出「环境里设着已删的变量」那一条）；照共用约束字面，那种退 2 落进「先让 admission.py 能跑」，出路不对 | 中 | 推的（两份文字逐字比） |
| A7 | 一百三十二 | 「它要登记五处」：少了 must-red 那一样（`crates/mutations.tsv` 里点名这个测试目标的变异行，`--only` 也没带它）；观察者登记表「那一行」照字面每条用例一行，而那张表按观察者登记、每多一行就让带 gpu 那一组的键变；轻型变异登记表那一行只在核心代码行点到这个测试目标时才该有，多登会红；第三列放量分项还要 `crash-amplification.sh` 的 ITEMS 里有那一项，不在五处里 | 中 | 推的 |
| A8 | 一百三十 | `commit-stages.md` 第 19 行改过的那一句，末尾出路仍是「`--check feature-gated-tests` 只重跑这一格」，新加的 product-gpu-real-card-tests 格同样要卡，照做它不重跑 | 中 | 推的 |
| A9 | 一百三十（漏改） | `.claude/main-agent.md` 第 32 行合入后验证的快档仍写「crash-case-replay、feature-gated-tests 两格」，`merge-and-verify.sh` 此刻起三格；`.claude/main-agent.md` 第 101 行与 `.claude/skills/crash-test/SKILL.md` 第 21 行的「快档两趟」没补这一格 | 中 | 推的（逐字比） |
| A10 | 一百三十（对照脚本） | `merge-and-verify.sh` 判快档起不起只看 checker 档包的依赖闭包，补丁只改产品 GPU 包时判「没碰」、整步不起：新加的那一格恰好在它守的那种补丁上不跑 | 中 | 推的；不在被判句子里，作旁证交主 agent |
| A11 | 一百三十 | `implementation-workflow.md` 两处与 `commit-stages.md` 第 10 行写「没卡报「本次未跑」」；服务没交卡时那一格判红，只有测试自己找不到显卡才报本次未跑 | 低 | 推的 |
| A12 | 一百六十六 ⑨ | `mutation-triage.md` 第 32 行的逐条次序只认 `NOT_RUN_NO_GPU_ADAPTER` 那一种本次未跑；门禁另有「带 gpu 特性而给定状态全交 CPU 判」也记本次未跑，那一行是 `ok`、没有 NOT_RUN 行，照定义记成没红；另「计数：」写「六栏」，`mutate.sh` 那一行此刻是九栏 | 低到中 | 推的 |
| A13 | 一百六十六 ② | 整份点名的规则里有与点名它的定义相冲的条：`pushback-discipline.md` 要把警告写进 `.claude/warnings/`，攻方腿写不到那里；`machine-first.md` 的「留下的」列「对外格式与协议兼容」，与共用约束「里程碑 17 收尾之前不兼容任何旧东西」相反；`clerk.md` 同一句先说「只读那三处」后说「另读」；同一种病另有三条规则只在共用约束的括注里被提到，格判绿而没有一份定义叫子 agent 读它 | 低到中 | 推的 |

没打中的形状见末尾「没打中的形状」一节。

## 一、一百二十六 ①②：设计员与执行员的 gpu 特性与 NOT_RUN 核对（A1–A5）

### A1 走判段的装置只能是入库装置，入库装置的产物不经 cargo、编法不带特性（强）

句子：
- `.claude/agents/experiment-designer.md:31`（9f1edf3a）：「登记里起它的 `cargo run` / `cargo test` 一律带 `--features gpu`」
- `.claude/agents/experiment-runner.md:40`（467bf57a）：「起它的 `cargo run` / `cargo test` 带 `--features gpu`」

照做的那条路上另几句：
- `.claude/agents/experiment-runner.md:30`（467bf57a）：「执行员交回的产物命令只执行这一份（写成 `task-server/task submit -- <二进制…>`）」
- `.claude/agents/experiment-runner.md:32`（467bf57a）：「入库装置先在草稿目录的副本里改，编过再整份换进主工作区」
- `scripts/hosts/compile-then-swap.py:300`（e9d0288c）：「commands = [["nice", "-n", "19", "cargo", "build", "--offline", "-p", package, "--bin", bin_name, *profile],」——编法里没有特性，用法行 `scripts/hosts/compile-then-swap.py:7`：「compile-then-swap.py <草稿副本> <目标> --scratch <草稿目录> [--root 仓根] [--release]」也不收特性参数。
- 研究装置那一支编不出走判段的装置：`grep -c 'singlefs-checker-tier\|singlefs_checker_tier' scripts/experiment-apparatus/Cargo.toml` 得 0（be26f6db，量的：此刻文件），`scripts/experiment-apparatus/*.rs` 里 `singlefs_checker_tier` 命中 0 份。源文件里写着判段入口的 bin 全在 `crates/singlefs-checker-tier/src/bin/` 下（e156、e158、e161、e168、e170、e171、e184 等）。
- 判段在没编进 gpu 时只打两行、不判：`crates/singlefs-checker-tier/src/crash_amplification.rs:17511`（672e599d）：「for line in not_run_lines_without_the_gpu_feature(flow_name, reason) {」；本机多机配置此刻开着 GPU（`grep -c '^ENABLE_GPU=1' multi-host.env` 得 1，量的）。
- 服务认要不要卡只看命令：`task-server/task_server.py:171`（c22f6922）：「GPU：认命令（gpu_need：带 gpu 特性而跑编出来的代码的 cargo」——裸二进制不在认法里，不分卡、不清场。

照字面走一遍（推的）：设计员照第 31 行在登记里写 `cargo run --release -p singlefs-checker-tier --features gpu --bin e<号>_<名>`，门禁那一格判绿；执行员照第 1b 步入库装置那一支交 `compile-then-swap.py`，主 agent 编出的二进制没有 gpu，产物命令照第 30 行只执行这一份；开着 GPU 的配置下判段打 `CRASH_AMPLIFICATION_NOT_RUN step=judge …` 与 `NOT_RUN_NO_GPU_ADAPTER …` 两行、不判；执行员照第 40 行核「起它的 cargo run 带 --features gpu」，产物命令里没有 cargo，这一样核不了，`grep -c NOT_RUN` 得 2，按本次未跑报。每一个走判段的实验都停在这里，定义里找不到一句让它带上 gpu 的出路。
就算主 agent 自己带特性编了二进制，服务照第 171 行不认裸二进制要卡，判段在没分卡、没清场的环境里起。

会红的世界：照执行员第 1b 步对任一入库判段装置（例 e161）跑 `compile-then-swap.py … --release`，再交 `task-server/task submit -- <草稿目录>/compile-then-swap/target/release/e161_crash_state_dedup_and_time_split <登记的参数>`，开着 GPU 时输出里 `grep -c NOT_RUN` ≥ 1。要编 checker 档包的 release 二进制，代价大；代码三处已经够判，腿不建议为它单跑，命令不列进「要主 agent 跑的命令」。

四句：
- 分不分辨改法：打的是「设计员写 cargo run、执行员跑二进制」这条链本身，与新句子写 `--features gpu` 还是 `-F gpu` 无关。
- 被判的执行员当时看得到什么：它只看得到自己写的产物命令与输出，看不到二进制是怎么编的（编法在 `compile-then-swap.py` 里）；第 40 行要它核的东西它看不到。
- 满足的分句：第 40 行「起它的 `cargo run` / `cargo test`」这个宾语在入库装置那一支不存在；第 31 行「登记里起它的」那一条命令与真正产出产物的命令不是同一条。
- 改法在这几格上中不中（全是推的，只在腿的推演上量过、被攻过零轮）：甲 给 `compile-then-swap.py` 加特性参数——二进制带上 gpu，但服务仍不认裸二进制要卡，还中一半；乙 执行员第 1b 步入库装置那一支的产物命令改成经服务交 `cargo run --release -p singlefs-checker-tier --features gpu --bin …`（服务认卡、清场）——两格都修；丙 只改第 40 行核法（核编法日志）——让本次未跑报得对，判段仍不跑，不修。

### A2 「源文件里有 …」比「走到判段」宽，也会漏（中）

句子同 A1 的第 31、40 行括注「源文件里有 `crash_amplification::` 或 `crash_state_enumeration::`」。门禁那一格用同一对记号（`.claude/gate.d/doc-experiments.sh:2910`（96c46f07）：「JUDGE_ENTRY_MARKERS="crash_amplification:: crash_state_enumeration::"」）。

宽的一边：
- `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs:63`（07d47d6c）：「use singlefs_checker_tier::crash_amplification::{plan_crash_points, CrashFlow, STATES_PER_BLOCK};」
- `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:59`（7bf68ca1）：「sample_crash_states_on_the_cpu, CrashPointReplayItem,」
- 两份都不进判段（量的，此刻源文件）：

```
$ for f in crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs; do printf '%s 处  %s\n' "$(/usr/bin/grep -c 'run_from_environment\|GivenStatesEngine\|judging_test_not_run\|judge_not_run' "$f")" "$f"; done
0 处  crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs
0 处  crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
2 处  crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
```

照字面做：这类只数崩溃点、只在 CPU 上取样的装置也带 `--features gpu`，服务照 `task-server/task_server.py:171` 认成要卡，套清场。`.claude/rules/verification.md:63`（0072bcc9）：「跑完、跑红或被打断都不复原」；同一行：「清场回读没过不跑、退 78」。一个不用卡的计数模型要排卡、清场、不复原，卡上有别的计算进程时整条退 78，被记成本次未跑。

漏的一边：`use singlefs_checker_tier::crash_amplification as amplification;` 之后写 `amplification::…`，源文件里没有 `crash_amplification::` 这串字，括注与门禁都当不走判段，不要 gpu；开着 GPU 时判段照样只录不判。门禁文件头自己只列了另一种漏法（`.claude/gate.d/doc-experiments.sh:2901`：「管不到的：判段入口写在 bin 引的别的模块里」），别名导入没列。

会红的世界：样本根 `marker-without-judge`（只数不判，门禁照样要 gpu，推的退 1）与 `alias-import-judge`（别名导入走判段，门禁放过，推的退 0），命令见「要主 agent 跑的命令」第 1、2 条。

四句：分不分辨改法——宽的一边让「按源文件记号认」与「按调用了判段入口认」分开（后一种不中），漏的一边两种都中，要按编出来的符号或调用图认才不中；系统看得到什么——设计员写登记时只看得到源文件，看不到这个装置这一趟会不会进判段（取决于参数）；满足的分句——括注「源文件里有 …」被当成「走到崩溃判段」的定义，字面上是充分条件，实际两边都不等价；改法——把括注改成「调到 `run_from_environment`、`GivenStatesEngine` 或 `judging_test_not_run_without_the_gpu_feature` 的」修宽的一边、不修别名那一边（推的）。

### A3 「门禁 … 格判特性」：那一格按行认命令，续行两头都错（中）

句子：`.claude/agents/experiment-designer.md:31`（9f1edf3a）：「门禁 doc-experiments 的 preregistration-judge-features 格判特性」。设计员读到这句，会把门禁当成这一条的兜底。

门禁那一格逐行找 `cargo run` / `cargo test`、只在同一行里找 `--bin` 与特性：
- `.claude/gate.d/doc-experiments.sh:3040`（96c46f07）：「for found in CARGO.finditer(line):」
- `.claude/gate.d/doc-experiments.sh:3049`：「if not binary:」，下一行 `.claude/gate.d/doc-experiments.sh:3050`：「without_binary += 1」
- 截命令的记号里没有行尾反斜杠：`.claude/gate.d/doc-experiments.sh:2924`：「COMMAND_END = re.compile(r"`|\\?\||;|&|\s--(?:\s|$)")」

两种写法（都是围栏里、照设计员定义「整条写成 task-server/task submit -- <命令>」写的长命令，折行是常见写法）：
- `--bin` 折到下一行、没带 gpu：首行没有 `--bin`，记「没点名 --bin」不判；次行没有 `cargo`，不看。整份一条都没判，退 77「无对象可判」。照定义该红。样本根 `continued-bin-on-next-line`。
- `--bin` 在首行、`--features gpu` 折到下一行：首行认出走判段的 bin、特性集合是空的，判红。照定义该绿。样本根 `continued-features-on-next-line`。
- 对照：单行没带 gpu 判红（`control-single-line-red`），单行带了判绿（`control-single-line-green`），带范围声明的整条照样认得出（`declared-cargo-test-green`）。

期望退出码（推的）逐个写在 `research/prompts/d137-ledger-defs-r1-opus-model/out/expected.txt`。这两格打的是「门禁兜底」那半句，不是 gpu 那半句：门禁文件头列了它管不到的几样，续行不在其中。

四句：分不分辨改法——门禁按逻辑行（先把行尾反斜杠接起来）认就两种都不中，只在定义里删「门禁判」那半句不修门禁的误红；系统看得到——门禁拿到的是整份登记，续行在文本里看得见；满足的分句——「格判特性」这半句许诺的范围比那一格的认法宽；改法见前。

### A4 「产物里 `grep -c NOT_RUN` 得 0」：NOT_RUN 行不在产物里时数到 0（中）

句子：
- `.claude/agents/experiment-designer.md:31`：「判据里写一条「产物里 `grep -c NOT_RUN` 得 0」」
- `.claude/agents/experiment-runner.md:40`：「`grep -c NOT_RUN <产物>` 得 0」

两种形态（推的，代码逐行）：
- 产物不是标准输出：判段那两行用 `println!` 打到标准输出（`crates/singlefs-checker-tier/src/crash_amplification.rs:17511`），而有的入库判段装置把结果另写进文件或目录：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:5322`（b22e832d）：「"--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),」；`crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs:1439`（07d47d6c）：「parsed.single("--out")」。对这些文件 `grep -c NOT_RUN` 得 0，判段没跑也过。
- 产物是 `.gz`：仓里已有压缩的产物（`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz`，883108 字节，量的），门禁 doc-experiments 有专认 `.gz` 产物的样本；`grep -c` 读压缩字节。量的（腿在草稿目录造了一份两行 NOT_RUN 的产物再压，GNU grep 与 zgrep 各数一次）：

```
$ /usr/bin/grep -c NOT_RUN /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r1-opus/gz-sample.out.gz; echo "rc=$?"
0
rc=1
$ zgrep -c NOT_RUN gz-sample.out.gz
2
```

另一种不算打中的：门禁与入口自己打的别的 `NOT_RUN_<原因>`（例 `replay.sh` 的 `NOT_RUN_ARCHIVED_PRODUCT`，打在标准错误）被一并数进来，那也是「本次未跑」，数进来不错。没找到正常判完的装置会打出带 `NOT_RUN` 的行（`crates/singlefs-checker-tier/src/bin/*.rs` 里 `NOT_RUN` 只出现在常量名与退出码名里，不在打印的字串里），所以「数到无关的行」这一格没打中。

四句：分不分辨改法——写成「装置的标准输出与它写的每份产物（`.gz` 先解压）」就两种都不中；系统看得到——执行员看得到产物路径与任务日志；满足的分句——「产物」一词在入库装置那一支可以指装置写的文件，不一定是标准输出；改法——上面那一种写法（推的）。

### A5 「整条写成 `task-server/task submit -- <命令>`」没带范围声明（低）

句子：`.claude/agents/experiment-designer.md:31`：「整条写成 `task-server/task submit -- <命令>`」。范围声明闸对 `cargo test` 要声明（`.claude/hooks/run-scope-guard.sh:41`（c520949e）：「cargo test  cargo test 与 cargo t」那一行起的表），共用约束也写「范围声明写在 submit 前面」。照这一句字面抄进登记、主 agent 照抄交的 `cargo test` 那一种被闸当场拒；`cargo run` 不要声明，不受影响。喂钩子的输入在 `research/prompts/d137-ledger-defs-r1-opus-model/inputs/run-scope-designer-literal.json`，命令见「要主 agent 跑的命令」第 3 条，期望退 2（推的）。执行员定义第 1b 步写了声明（`.claude/agents/experiment-runner.md:29`：「SINGLEFS_RUN_SCOPE='种类=target; 跑什么=装置 <bin 名> 的单测」），两份定义对同一种命令的写法不一样。
改法：第 31 行写成「`SINGLEFS_RUN_SCOPE='…' task-server/task submit -- <命令>`（`cargo test` 要声明，照范围声明闸文件头）」（推的）。

## 二、`agent-common.md` 删 `SINGLEFS_EXPERIMENT_RERUN_REASON` 那半句（A6）

删完的那一句：`.claude/agent-common.md:98`（b4ee6573）：「退 2 的拒绝说明分两种出路」，接着「标准错误里有它自己报「没登记」「登记有错」的句子的，先接准入；没有的，先让 `admission.py` 能跑」。
派发闸此刻：`.claude/hooks/runner-dispatch-guard.sh:68`（acde6287）：「退 2 一律照拒，拒绝说明按输出分三条出路」，多出来的那一条在 `.claude/hooks/runner-dispatch-guard.sh:69`：「它照用法错退 2；派发闸起它时不清这个变量），出路是从起会话的环境里去掉它、输入没变不重跑、要重跑先改输入」。准入一侧见了这个变量就退 2：`scripts/gate/admission.py:475`（15b28dc4）：「RETIRED_RERUN_REASON_VARIABLE = "SINGLEFS_EXPERIMENT_RERUN_REASON"」。

照字面走一遍（推的）：主 agent 起会话的环境里还设着这个变量（旧习惯、旧 shell 的 export），派设计员写重跑登记，派发闸起准入、退 2，标准错误是「✗ SINGLEFS_EXPERIMENT_RERUN_REASON 删了…」。主 agent 拿共用约束对：这一句里没有「没登记」「登记有错」，落进「没有的，先让 `admission.py` 能跑」——去核脚本在不在、python 起不起得来，两样都好好的。出路指错了；拒绝说明本身写着对的出路，所以最坏是绕一圈。
删的那半句原来说的「起它之前从环境里清掉」与钩子此刻「不清」相反，删得对；没跟着改的是同一句的「两种出路」。这是格 A「删完之后上下文还通不通」那一类。
改法：「两种出路」改成三种，补「标准错误里有「SINGLEFS_EXPERIMENT_RERUN_REASON 删了」的，从起会话的环境里去掉它」（推的）。
四句：不涉及臂；主 agent 看得到标准错误；满足的分句是「分两种出路」的列举不全；上面的改法在这一格上不中。

## 三、一百三十二：实现员第 7a 步「它要登记五处」（A7）

句子：`.claude/agents/implementation-writer.md:47`（70ab774a）：「它要登记五处，报告里逐处给出行原文」，同一行：「交回前把 `task-server/task submit -- python3 scripts/crash/crash-case-check.py --only placement,modules,provenance` 写进报告「要主 agent 跑的命令」一节」。

### A7 甲 少了一处：must-red（中）

`scripts/crash/crash-case-check.py:11`（2ce57160）：「must-red      每条崩溃枚举用例在 crates/mutations.tsv 里至少有一行变异点名它的测试目标」，提交时由 `.claude/gate.d/checker-tier-crash-point-replay.sh:15`（d19991e5）：「gate-cell: crash-case-must-red」那一格判。五处里没有它，`--only` 也没带 `must-red`。
实现员第 3、4 步要求每条新测试写一条能让它红的变异行，多半会顺带满足；但 must-red 只认参数里带 `-p singlefs-checker-tier` 与 `--test <这个测试目标>` 的行。实现员照第 4 步「checker 档的测试不写进「要主 agent 跑的命令」」、为了能在副本里证红，把变异行写成点名 harness 档某个测试目标的，主 agent 照第 47 行交的 `--only placement,modules,provenance` 判绿，红留到提交时那一格（推的）。
改法：「五处」改成「六处」，补「`crates/mutations.tsv` 里至少一行参数带 `-p singlefs-checker-tier --test <这个测试目标>` 的变异行」，`--only` 加 `must-red`（推的）。

### A7 乙 观察者登记表「那一行」照字面每条用例一行（中）

`crates/singlefs-checker-tier/src/judging_units_entries.rs:134`（00f4508f）：「一个观察者的入口」——这张表按观察者登记，不按用例。此刻 64 条观察者入口、52 条 `crash-case:` 行（量的：`grep -c '^    ObserverEntry {'`、`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv`），同一个测试目标挂 4 个观察者的有两处。新用例沿用已有观察者、或不挂观察者时不该加行。
多加一行不是无害的：`crates/singlefs-checker-tier/src/judging_units_entries.rs:7`：「所以 [`OBSERVER_ENTRIES`] 每加、改一行，这一组带 `gpu`」，下一行：「的键就变，交回镜像的观察者在判段重判一次」。照字面每条新用例补「那一行」，每次都把带 gpu 那一组判定作废一遍（推的）。
改法：写成「新写了观察者的，每个观察者一行；沿用已有观察者的不加」（推的）。

### A7 丙 轻型变异登记表那一行不是每条都要（低）

`.claude/gate.d/crates-mutation-light-rows.tsv:5`（e4e0042d）：「改坏对象不在核心代码下的行不登记，它们的测试目标也不要求登记」；多登了会红：`scripts/mutation/crates-mutation-light-rows.py:35`（ad9c365f）：「CRATES_MUTATION_LIGHT_ROWS_BREAK=non-core-rows-accepted  ⑤ 不核登记的行改坏对象在核心代码下」。第 47 行写了「挑法在那份表的文件头」，读文件头的实现员不会错；只读「五处」的会把它当成必有的一处。

### A7 丁 放量分项要 ITEMS 里有那一项，不在五处里（低）

`.claude/gate.d/crash-point-replay-items.tsv:6`（ee2f64d4）：「③ 重型里放量那一套的部分：crash-amplification.sh 的 ITEMS 里的分项名」，核法 `.claude/gate.d/crash-point-replay-items.tsv:10`：「③ 的每一项在 ITEMS 里」。第 47 行要求写「每个放量分项的 `amplification_of_the_item` 行」，却没把 `scripts/crash/crash-amplification.sh` 的 ITEMS 列进登记处；新用例带放量分项时 placement 红。

### A7 戊 交 crash-case-check 的时机（低）

`.claude/gate.d/` 下三处由主 agent 或工具员登记，第 47 行没说命令要在登记之后交。登记之前交：placement 照判红（标了 ignore 没登记），provenance 只扫合表里已有的项（`scripts/crash/crash-case-check.py:54`：「第二列点名的 crash-case: 键里有一条的测试目标文件」），新用例的来历表那一行缺了也判不到（推的）。

四句（A7 合起来）：不涉及臂；实现员看得到 `crash-case-check.py` 文件头与三张表的表头；满足的分句是「它要登记五处」的列举（多一处、少两处、一处是条件性的）；改法逐条写在上面，各修各的那一格，互不覆盖。

## 四、一百三十：快档字句补 product-gpu-real-card-tests（A8–A11）

### A8 改过的那一句，末尾出路没跟上（中）

`.claude/skills/crash-test/commit-stages.md:19`（21973562）这一批插进了「与 product-gpu-real-card-tests 格的真卡对拍」，同一句往后接着：「清场回读没过时等卡上那个不归清场白名单的进程跑完再 `--check feature-gated-tests` 只重跑这一格」。两格现在都要卡；照字面清场回读没过之后只重跑 feature-gated-tests，产品 GPU 那一格不重跑，这一趟的结论里它一直缺着（推的）。
改法：写成「再 `--check feature-gated-tests,product-gpu-real-card-tests` 只重跑要卡的这两格」（推的）。

### A9 别处还在说「两格」「两趟」（中，该改没改）

- `.claude/main-agent.md:32`（439b97be）：「快档的 crash-case-replay、feature-gated-tests 两格只现判单元键对不上的测试函数」。合入后验证此刻起三格：`scripts/gate/merge-and-verify.sh:275`（bdf206f6）：「CHECKER_QUICK_CELLS="crash-case-replay,feature-gated-tests,product-gpu-real-card-tests"」。同一句后半「快档的别的格与 checker 档全量归提交时」照字面把 product-gpu-real-card-tests 归到提交时，与脚本相反；主 agent 读日志时会把那一格的红当成「不该在这一步出现」。
- `.claude/main-agent.md:101`：「`checker-tier-crash-point-replay` 跑快档两趟」，括号里列的只有不带特性、带特性两趟，没有产品 GPU 那一格。
- `.claude/skills/crash-test/SKILL.md:21`（38fc400a）：「快档两趟（`cargo test --release -p singlefs-checker-tier --lib --tests`，再带 `tests/` 里 `cfg` 点到的特性另跑一趟）」，同样没补。
账本第 144 行写「main-agent.md 只写「快档几格」，不用改」，第 32、101 行都不是「几格」的写法；`SKILL.md` 账本没点名。这三处不在这一轮被判文件里，交主 agent 定归哪一批。

### A10 合入后验证判「没碰」时整步不起，新加的那一格恰好在它守的补丁上不跑（中，对照脚本）

`scripts/gate/merge-and-verify.sh:643`（bdf206f6）：「return "unaffected", f"补丁只改到」——判法只看补丁改到的包在不在 checker 档包的依赖闭包里。产品 GPU 包不在闭包里（`singlefs-background-gpu` 在 checker 档包、harness、core、checker 四份 `Cargo.toml` 里命中都是 0，量的）。只改 `crates/singlefs-background-gpu/` 的补丁判「没碰」，整步快档不起、报本次未跑，product-gpu-real-card-tests 那一格正是测这个包的，却在这种补丁上一次都不跑；要等提交时整轮快档（推的）。
`.claude/main-agent.md:32` 写的「补丁没碰 checker 档包与它依赖的包时报本次未跑」与脚本一致，所以这不是字句对不上，是这一批把格加进去时没跟着改起不起的判法。不在被判句子里，作旁证交主 agent。
会红的世界：补丁目录里 `crates.patch` 只改 `crates/singlefs-background-gpu/src/` 下一处，交 `merge-and-verify.sh`，汇总行报「checker 档快档本次未跑」、原因「补丁只改到 singlefs-background-gpu…」。要真编、真跑测试，代价大，腿不列进命令；判法那段 python 在 `scripts/gate/merge-and-verify.sh:612` 起，代码已够判。

### A11 「没卡报「本次未跑」」把两种没卡并成一种（低）

`.claude/rules/implementation-workflow.md:44`（262d8682）与 `.claude/rules/implementation-workflow.md:68`、`.claude/skills/crash-test/commit-stages.md:10` 都写「`--test-threads 1`，没卡报」本次未跑。门禁里两种没卡结局不同：服务没交卡（环境里没有 `GPU_CARDS_BY_PRIORITY`）时那一格判红、不起——`.claude/gate.d/checker-tier-crash-point-replay.sh:191`（d19991e5）：「没有就那一格判红、那一趟不起」；测试起来了、本机找不到显卡时才逐条报 `NOT_RUN_NO_GPU_ADAPTER`、记本次未跑。`.claude/rules/verification.md:35` 写的是后一种（「没卡打的 `NOT_RUN_NO_GPU_ADAPTER` 逐条报「本次未跑」」），这三处短写成「没卡」，主 agent 照它读一次直接起门禁的红时，会当成「没卡、本次未跑」放过（推的）。
改法：照 verification.md 那一处写全「没卡打的 `NOT_RUN_NO_GPU_ADAPTER` 逐条报本次未跑；服务没交卡判红」（推的）。

四句（A8–A11）：不涉及臂；主 agent 看得到门禁日志里点名的格；满足的分句分别是「只重跑这一格」的宾语、「两格」「两趟」的计数、「没碰」的判法射程、「没卡」的指代；改法各修各的那一处。

## 五、一百六十六 ⑨ 计数栏（A12）

句子：`.claude/agents/mutation-triage.md:32`（b60ebc1c），与 `.claude/rules/mutation-sampling.md:79`（0ebfe2eb）同一套字。

### A12 甲 本次未跑只认一种（低到中）

`.claude/agents/mutation-triage.md:32`：「那一行是 `ok`、输出里又有一行 `NOT_RUN_NO_GPU_ADAPTER` 点名它，记「本次未跑」」，再往下「那一行出现了（跑了没红，或跑到它、跑完它之后进程被杀）记没红」。门禁另有一种本次未跑：`scripts/mutation/crates-mutation-rows.py:207`（4a31ff70）：「带 gpu 特性的格另有一种记本次未跑（给定状态全交 CPU 判）」，汇总那一句 `scripts/mutation/crates-mutation-rows.py:4762`：「或带 gpu 特性而多机配置不在、给定状态全交 CPU 判」。这种行点名的测试那一行是 `ok`、输出里没有 NOT_RUN 行；分诊员照第 32 行的次序记「没红」，门禁记「本次未跑」，两边的数对不上，分诊员会去给一条门禁并没判没红的变异写分类（推的）。
第 41 行要求把门禁的「本次未跑：」行原样贴上，贴出来的数与分诊员自己数的没红会互相矛盾，第 32 行没说以谁为准。
注：`crates-mutation-rows.py` 开工之后被别的会话改过（开工读时 63f49976，此刻 4a31ff70，`git status` 是 `MM`），行号以此刻为准，主 agent 核时现查。
改法：第 32 行与 `mutation-sampling.md:79` 一起补「或门禁的「本次未跑：」行点名它」（推的）。

### A12 乙 「六栏」与 `mutate.sh` 此刻那一行（低）

`.claude/agents/mutation-triage.md:32`：「报抓到、没红、无效、没跑完、抓名字有盲区、锚点失效六栏」。`mutate.sh` 收尾那一行在六栏后面总接三栏：`scripts/mutation/mutate.sh:2030`（1e064d0d）：「LIBRARY_COUNT_TEXT="、现判 $LIBRARY_FRESH 条、复用 $LIBRARY_REUSED 条、拒算 $LIBRARY_REFUSED 条"」，自证里的样子 `计数：…、锚点失效 1 条、现判 N 条、复用 N 条、拒算 N 条`。同一句后面写「照原样另列」，贴原样不会错；只有把「六栏」当核对条件（数一数栏数）的读法会以为这一行坏了。这一条撞的是第 32 行与第 79 行同一句，两份字相同，不分高低。

## 六、一百六十六 ②：四份 read-once 规则的点名（A13）

### A13 甲 攻方腿读的 `pushback-discipline.md` 要它写一处它不写的文件（低到中）

`.claude/agents/three-way-attack.md:17`（a4260bda）把这份规则整份点名（没点小节）；共用约束说整份点名的规则整份照做（`.claude/agent-common.md:21`：「点名的小节是这件活要照做的那几条」，没点小节就是全篇）。规则里：`.claude/singlefs-ai-sop/rules/pushback-discipline.md:46`（a182572c）：「下按日期建文件」，即 `.claude/warnings/<日期>.md`，一次警告一节、四项都要有。攻方腿的写范围：`.claude/agents/three-way-attack.md:51`：「报告文件、模型目录、草稿目录。除此之外不写。」攻方腿照规则写警告就越界，不写就没照做规则；同一份规则的「对方坚持，就照做」一节说的是与用户对话，攻方腿没有这种对话。写范围闸对攻方腿的 Bash 写不判（`.claude/hooks/agent-write-scope.tsv` 里没有 three-way-attack 的行），越界不会被当场拦下（推的）。
改法：点名时带小节，写成「`pushback-discipline.md`「对不上就当场说」「分清「我核过」和「我印象里」」」（推的）。

### A13 乙 实现员、工具员整份读的 `machine-first.md` 里有一条与共用约束相反（低到中）

`.claude/singlefs-ai-sop/rules/machine-first.md:87`（92e21312）：「- 对外格式与协议兼容」，在「留下的（与谁读代码无关）」那一节，意思是兼容要留。共用约束：`.claude/agent-common.md:52`：「里程碑 17 收尾之前不兼容任何旧东西」。两份都被实现员整份照做；共用约束开头只写「与定义冲突时以定义为准」，没写与规则冲突时以谁为准。实现员改格式时可以拿 machine-first 那一条留旧版本的认法（推的）。`engineering-philosophy.md` 只讲两条轴的分工，没找到相冲的条。
改法：点名时带小节（「遇到一条通行准则，走这个流程」「判据」），或在共用约束第 52 行注明它压过 machine-first 的那一条（推的）。

### A13 丙 书记员那一句自己相冲（低）

`.claude/agents/clerk.md:16`（44c8ae8b）：「一节写的那三处」，同一句后半「改 `records/` 下的文件时另读」design-doc-discipline。前半的「规则只读」与后半的「另读」字面相反，照前半读的书记员不读它。`design-doc-discipline.md` 开头写它管 `records/`，点给书记员不算点错。

### A13 丁 同一种病还有三条（低到中，该改没改）

格 rules-named-by-definitions 只认路径出现，`.claude/gate.d/code-tooling.sh:1715`（45de49a4）：「管不到的：点名的那一处是不是真让子 agent 读它」。腿现算了每条登记规则被哪几份定义在哪一行点名（量的：`named.py` 在草稿目录，读 `CLAUDE.md` 与 `.claude/agents/*.md`、`.claude/agent-common.md`），三条只在共用约束的闸说明或括注里出现、没有一份「开工先读：」点它们：`session-wrapup.md`（只在 `.claude/agent-common.md:95`）、`timer-only-remind.md`（只在 `.claude/agent-common.md:100`）、`path-moves.md`（`.claude/agent-common.md:27`、`.claude/agent-common.md:94` 与 `sweep.md` 第 34 行，都是「写法见」一类的指路）。共用约束「规则怎么读」只把「开工先读：」一行点名的规则定为整份读，这三条门禁判绿，而没有子 agent 被要求读它们。账本第 357 行只点了四条，这三条不在这一批里。

## 要主 agent 跑的命令

都在仓根 `/home/fy5090/code/singlefs` 起（不给 `--cwd`）。三条都不是重型；第 2 条起的是门禁 doc-experiments 的一格，范围声明写在 `judge-worlds.sh` 起门禁的那一行里。

1. 造七个样本根（模型目录的 `tools/rerun.sh` 整份跑一次）：
   `task-server/task submit -- bash research/prompts/d137-ledger-defs-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r1-opus/rerun-1`
   看结果：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；log 里七行 `name=world_built world=<名> untracked=1` 与末行 `name=rerun_worlds …`。`--frozen` 只为过模板的参数检查，这个模型不读冻结副本（这一轮没有冻结副本）。
2. 在每个样本根上单跑 preregistration-judge-features 格（第 1 条之后）：
   `task-server/task submit -- bash research/prompts/d137-ledger-defs-r1-opus-model/tools/judge-worlds.sh /home/fy5090/.local/share/singlefs/work/d137-ledger-defs-r1-opus/rerun-1/worlds`
   看结果：`exit=0` 只说明七个都跑了；判对错看 log 里七行 `name=judge_world world=<名> rc=<码>`，逐行与 `research/prompts/d137-ledger-defs-r1-opus-model/out/expected.txt` 第二列（门禁退出码，推的）比，再与第三列（照定义该判成什么）比。腿的推断成立的样子：`continued-bin-on-next-line` rc=77、`continued-features-on-next-line` rc=1、`alias-import-judge` rc=0（这三格门禁与定义不一致），`marker-without-judge` rc=1（门禁照样要 gpu），两个对照 rc=1、rc=0，`declared-cargo-test-green` rc=0。两个对照不是 1、0 的，整批作废（样本根没造对）。每个根的门禁原样输出在 `<worlds>/<名>.gate.out`。
3. 喂范围声明闸（A5，可选）：
   `task-server/task submit -- bash -c 'bash .claude/hooks/run-scope-guard.sh < research/prompts/d137-ledger-defs-r1-opus-model/inputs/run-scope-designer-literal.json; echo "rc=$?"'`
   看结果：log 末行 `rc=2`，标准错误里说缺范围声明。输入里没有 `agent_type`，按主 agent 判；放行时钩子会往检出记录追加一行，退 2 时不追加（推的，照它文件头）。

没列进来的（代价大，代码行已够判）：A1 的「compile-then-swap 编的判段装置跑出 NOT_RUN」与 A10 的「只改产品 GPU 包的补丁让合入后验证跳过快档」，会红的世界的造法写在各自那一节。

## 没打中的形状

- 「`grep -c NOT_RUN` 数到无关的行」：在 `crates/singlefs-checker-tier/src`、`crates/singlefs-harness/src` 与装置 bin 里找打印的串，`NOT_RUN` 只出现在本次未跑那几种记号、常量名与退出码名里；`task-server/*.py` 不打 `NOT_RUN`。没找到正常判完也会打出它的行。
- gpu 特性名：`crates/singlefs-checker-tier/Cargo.toml` 的 `[features]` 此刻只有 `gpu`（第 39 行），`-p singlefs-checker-tier --features gpu` 编得过的写法；被判的定义里不剩 `verdict-store`。
- 计数栏删掉的四种符号：`scripts/mutation/mutate.sh` 里 `⏱`、`🧱`、`⏳`、`🚧`、`🎴` 合计 0 处（量的），删得对；`⚓` 在、`💥` 与 `⚠️` 在。门禁那一行「计数：抓到 … 锚点失效 N 行（共 N 条）」与另起的「基线：」「清场没过没跑：」「本次未跑：」行，与第 32、41 行写的对得上（`scripts/mutation/crates-mutation-rows.py` 的 `report` 函数）。
- `--only placement,modules,provenance` 三个名字都在 `crash-case-check.py` 文件头的八样里；`OBSERVER_ENTRIES`、`PROVENANCE_OF_THE_FLOWS` 的路径都在。
- 「崩溃枚举用例住哪、怎么登记」这个死节名：定义、共用约束、规则、skill、CLAUDE.md 里 0 处；剩下两处在门禁注释里，是描述那一样判据的话，不是指节名。
- 四份 read-once 规则此刻都被至少一份定义按整段路径点名（`named.py` 的输出：engineering-philosophy 2 份、machine-first 2 份、design-doc-discipline 1 份、pushback-discipline 1 份）。
- `SINGLEFS_EXPERIMENT_RERUN_REASON`：在 `.py`、`.sh`、`.rs`、`.md`、`.tsv` 里（`research/` 除外）只剩派发闸、`admission.py` 与 `records/`，定义、规则、skill 里 0 处。
- product-gpu-real-card-tests 的几样事实（`tests-on-a-real-gpu`、`--test-threads 1`、按单元键复用、对拍判红整格红）与门禁文件头第 11、149–164 行对得上；规则里不写「经清场」与「凡用 GPU 先清场」那一节一致。
- `design-doc-discipline.md` 开头写它管 `records/`，点给书记员不算点错；`engineering-philosophy.md` 没找到与实现员、工具员定义相冲的条。

取样范围：被判 11 份文件、正文第二节点名的对照文件，外加 `.claude/main-agent.md`、`.claude/skills/crash-test/SKILL.md`、`scripts/hosts/compile-then-swap.py`、`task-server/task_server.py` 的 GPU 认法、`crates/singlefs-checker-tier/src/bin/` 下 9 份带判段记号的装置、`scripts/mutation/crates-mutation-rows.py` 与 `crates-mutation-light-rows.py`、`.claude/hooks/runner-dispatch-guard.sh` 文件头。

## 这条腿自己的限度

- 一条命令都没往服务交；模型里的门禁退出码、各处「照做会发生什么」全是读代码推的。打中的每一格是线索，要主 agent 现查。
- A1 只打执行员第 1b 步那一条路（头一次跑产物）；重跑经 `replay.sh` 的驱动函数起，开了 GPU 时带 `--features gpu`（`scripts/experiment-apparatus/replay.sh` 文件头「GPU」一段），那条路不中。
- A2 的「不进判段」是按调用名数的（`run_from_environment`、`GivenStatesEngine`、`judging_test_not_run`、`judge_not_run`），没读 e184、e170 的全部调用图；经别的公共函数间接进判段的，这个数会漏。
- A10 是对照脚本的判法，不在被判句子里；判「该不该在合入后验证跑」是做法问题，腿只报边角。
- 没读正推腿的报告，与它重不重复不知道。
- `scripts/mutation/crates-mutation-rows.py` 在这一轮里被别的会话改着，A12 的行号以写报告这一刻为准。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；改法全是推的，只在腿的推演上过过、被攻过零轮。
- 没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）；没做 git 写操作。
- 没改被判文件与对照文件；只写了这份报告、模型目录 `research/prompts/d137-ledger-defs-r1-opus-model/` 与草稿目录。草稿目录里只有几份小脚本（`wdiff.py`、`ctx.py`、`named.py`）与一份两行的 `.gz` 样本，没有仓副本、编译目录与库拷贝。
