# 验证两档拆分：代码轮与定义轮合一的第一轮判决（2026-09-27）

正文 `research/prompts/_verification-split-r1-body.md`，背景材料 `research/prompts/_verification-split-r1-background.md`，开工快照 `research/prompts/verification-split-r1-snapshot/`。三条腿交齐：云端攻方 `verification-split-r1-opus-output.md`（Q1、Q2、Q4、Q6、Q9 与归因），云端正推 `verification-split-r1-sonnet-output.md`（Q3、Q5、Q7），本地攻方 `verification-split-r1-local-attack.md`（Q8，两份干净样本 `-output-s1.md`、`-output-s2.md`，运行记录 `-runlog.md`）。

**这一轮没派核查员**：用户 2026-09-27 定「subagent 执行完不再派发新的验证了」「你自己改 不要再派subagent了」。下面每一格的现值都由主 agent 在工作区现查，引的命令与输出写在那一格里；腿引的行号只当线索。

**攻的是拆出第三个包之前的那一版。** 三条腿交回之后，用户定「装置不需要留在 harness 要分清楚」「脚手架库 需要拆分的也拆分 我们宁可要多份 也不要耦合」，于是拆出 checker 档包 `crates/singlefs-checker-tier`。拆包与这一轮判决之后的改法（第二节）被攻过零轮；用户定这一轮之后不再开第二轮，零轮的形态记在第三节。

## 一、各格判定

| 格 | 腿的判定 | 主 agent 判 | 去向（现查） |
|---|---|---|---|
| Q1 闸按包判漏了几种写法 | 攻方：红，`-p` 通配、`name@version`、pkgid URL、看得到值的别名都交 None（量过） | 采纳 | 改了：`lib_heavy_tests.py` 的 `package_specification_members` 认这四种、`alias_expansion` 展开再判；现查 `cargo test -p 'singlefs-checker-t*'`、`-p singlefs-checker-tier@0.1.0`、pkgid URL、`CARGO_ALIAS_XT='test -p singlefs-checker-tier' cargo xt` 四条都交 `checker-tier-cargo` |
| Q1 附 cargo mutants | 攻方：`.cargo/mutants.toml` 写着 `test_workspace = true`，`cargo mutants -p singlefs-core` 交 None（classify 那一半量过，会跑整个工作区是推的） | 采纳 | 改了：`mutants_test_arguments` 读 `--test-package`、`--test-workspace` 与 `.cargo/mutants.toml` 的 `test_workspace`（`--no-config` 不读，`-d` 换源码树）；现查 `cargo mutants -p singlefs-core` 交 `full-cargo`；新弄坏开关 `mutants-configuration-ignored` 下自检 4 格红 |
| Q2 共用模块读不读环境 | 攻方：绿 | 采纳 | 无改动 |
| Q3 54 号报「本次未跑」算不算覆盖 | 正推：兑现了条款 | 采纳 | 无改动 |
| Q4 94 号不读 dev-dependencies 之后的缺口 | 攻方：红，dev-dependency 改名引实现、用在库单测里，94 号判绿（量过） | 采纳；改法换了 | 那一版的做法（池级 checker 加 dev-dependencies、94 号不读它）整个撤掉：三个包之后池级 checker 只依赖 `singlefs-format`，94 号照旧读全部依赖表；另加第 ④ 条，harness 档的依赖闭包（dev-dependencies 也算）与源码里不许有 checker 档包，样本 red 里 harness 经中间包在 dev-dependencies 里依赖它、源码引一处，两条各自判红 |
| Q5 六份定义有没有自相矛盾 | 正推：兑现了条款 | 采纳 | 无改动 |
| Q6 crash-case-check 认不全 | 攻方：红，同文件 helper、自写枚举函数等认不出，8 条登记只认出 3 条；一条豁免理由与实际不符 | 采纳 | 改了：`enumerating_helpers` 跟同一文件里传递地调枚举的函数；现查 `python3 research/scripts/crash-case-check.py .` 查了 35 条、判绿；`tests/<子目录>/` 里的共用函数、宏展开仍认不出，写在文件头与成功句；`crash_enumeration_resumes_from_its_progress_file.rs` 第 387 行那条豁免理由改成实际枚举的东西（第一条流的快档展开，第 12 个状态判红就停） |
| Q7.1 已定项 4、5、7 与 15 | 正推：兑现了条款 | 采纳 | 无改动 |
| Q7.2 已定项 9「范围」列与射程句说反话 | 正推：这一轮改动和条款说反话 | 采纳 | 改了：`.claude/kb/decisions/13-验证路线.md` 已定项 9 层 0 那一行范围改成「固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程）」 |
| Q7.3 已定项 15 的「两档」覆盖面 | 正推：兑现了条款，附一处措辞覆盖面歧义 | 采纳歧义那一半 | 改了：已定项 15 那句改成「checker 档里的崩溃点重放（门禁 54 号）分两档，55、57、59、87 号不分」 |
| Q8 命令分类表 | 本地攻方：两份样本各有 10、11 行与闸的真值对不上 | 逐行判：多数是标签用词不同（写 harness、layer 0，真值写不重型、checker 档），不是分类分歧；实质分歧只在第 5、6 行（`-p singlefs-checker --lib`、`--doc`），出自两包那一版规则文字说「singlefs-checker 包的测试」是重型而闸判不重型 | 三个包之后不再分歧：池级 checker 是 `singlefs-checker`，它自己的库单测归 harness 档（`.claude/rules/verification.md`「定义与名字」表），现查闸对这两条交 None |
| Q9 旧路径与行号 | 攻方：红（小），两处相对 harness 的注释路径，e161 的 5 处行号错开 | 采纳 | 改了：搬进 checker 档包的 35 个文件逐个按旧路径全仓搜，变更史、实验变更史与 records 里 19 处改成新路径（`path-moves.md` 第 3 步）；指向搬走文件的行号引用 15 处里按 HEAD 到现在逐行对应重算了 7 处（e161 装置 6 处、e162 装置 1 处），另 6 处指的是别的包的 `lib.rs`，与搬家无关；`history.rs` 那处相对路径改成全路径 |
| 归因 | 攻方：部分成立，射程比正文说的宽（C577 那道屏障解释第一条流的钉值，解释不了 `removing_the_barrier…` 的钉值） | 采纳，不在这一轮修 | 钉值不是这一轮改的，交 singlefs-99 那一轮「合入后验证」；拆包之后 harness 档 `cargo test -p singlefs-harness --no-fail-fast` 的红见第四节，同样归在飞的改动（推的，没量过） |

## 二、这一轮之后的改法（被攻过零轮）

- 拆出 checker 档包 `crates/singlefs-checker-tier`：崩溃态枚举引擎与记录核对器（`crash.rs`）、断点续跑（`layer0_progress.rs`）、崩溃注入与坏盘输入（`crash_injection.rs`、`bad_disk_input.rs`）、设备日志比对与真设备模式（`device_log.rs`、`on_device_modes.rs`）、8 个装置二进制与 21 个测试文件；harness 留脚手架库，内存池、保留写与崩溃镜像那一半进新文件 `crates/singlefs-harness/src/memory_pool.rs`。
- 闸只按包判，按测试名含 layer0 与按 `--ignored` 认的分支连同自检用例、弄坏开关一起删掉（`lib_heavy_tests.py` 自检 152 种、`heavy-test-guard.sh` 自检 675 种，全绿；弄坏开关各自判红）。
- 54 号快档 `cargo test --release -p singlefs-checker-tier --lib --tests`；`admission.py` 的 54 号样本改成 checker 档布局（280 格全对）。
- 新立门禁 13 号禁空泛名字，改名：`crates/singlefs-core/src/admission.rs` 的 `get` 改 `replicas`，`crates/singlefs-core/src/transaction.rs` 的 `perform` 改 `perform_commit_step`，`crates/singlefs-checker/src/walk.rs` 的 `value` 改 `birth_identity_tail_number`，装置里的 `run_cell` 一族改名。
- 94 号第 ④ 条；看门狗 `agent-watch.py` 的告警说明与假重型进程改成按包判（自检 exit 0）。

## 三、交用户的零轮形态

| 形态 | 被攻过几轮 | 去向 |
|---|---|---|
| 第二节全部 | 零轮 | 用户定这一轮之后不再派验证；被检测内容的正确性由之后的任务验证（用户「被检测内容的正确性 之后检测的时候会由其他任务验证」） |

## 四、拆包之后的测试（只核测试逻辑，不核被测内容）

- `#[test]` 计数：HEAD 1089、工作区 1093；`#[ignore]` 都是 25。差的 4 条新增与 1 条改名都在 singlefs-99 在飞的测试文件里。
- harness 档 `cargo test -p singlefs-harness --no-fail-fast`（日志 `research/prompts/verification-split-tmp-evidence/harness-daily-after-split.log`）：47 个测试目标都编过、跑完，通过 622、失败 14、忽略 3，红在 8 个目标里。红的是抬 F、空间准入与见证那几条行为断言，这 14 条只重跑、不整份重跑（`research/scripts/rerun-failed-tests.py` 从日志取命令）：在只含这一轮改动的暂存树上同样 14 条红（`research/prompts/verification-split-tmp-evidence/rerun-failed-on-staged-tree.log`），在纯 HEAD 上也是这 14 条红（`research/prompts/verification-split-tmp-evidence/rerun-failed-on-head.log`）：拆包之前就红，不是拆包带进来的（量过）。

## 五、按路径点名被判的文件（门禁 56、72 号）

checker 档包（新建，多数从 harness 搬来）：`crates/singlefs-checker-tier/src/lib.rs`、`crates/singlefs-checker-tier/src/crash.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/bad_disk_input.rs`、`crates/singlefs-checker-tier/src/layer0_progress.rs`、`crates/singlefs-checker-tier/src/device_log.rs`、`crates/singlefs-checker-tier/src/on_device_modes.rs`、`crates/singlefs-checker-tier/src/bin/e142_first_transaction_write_dump.rs`、`crates/singlefs-checker-tier/src/bin/e142_first_transaction_write_dump_one_device.rs`、`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`、`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs`、`crates/singlefs-checker-tier/src/bin/first_transaction_device_log_check.rs`、`crates/singlefs-checker-tier/src/bin/first_transaction_on_device.rs`、`crates/singlefs-checker-tier/src/bin/first_transaction_region_bytes.rs`。

搬走的旧路径：`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/crash_injection.rs`、`crates/singlefs-harness/src/bad_disk_input.rs`、`crates/singlefs-harness/src/layer0_progress.rs`、`crates/singlefs-harness/src/device_log.rs`、`crates/singlefs-harness/src/on_device_modes.rs`、`crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs`、`crates/singlefs-harness/src/bin/e142_first_transaction_write_dump_one_device.rs`、`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`、`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs`、`crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs`、`crates/singlefs-harness/src/bin/first_transaction_on_device.rs`、`crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs`。

harness 档：`crates/singlefs-harness/src/lib.rs`、`crates/singlefs-harness/src/memory_pool.rs`、`crates/singlefs-harness/src/first_transaction_regions.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/history.rs`。

改名与注释路径：`crates/singlefs-core/src/admission.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-format/src/lib.rs`、`crates/singlefs-checker/src/walk.rs`。`crates/singlefs-core/src/transaction.rs` 里另有 singlefs-99 的改动（重读一次的生效 F），不归这一轮判。

定义与共用约束：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/gate-triage.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/three-way-attack.md`。前五份 Q5 逐句核过；后四份只改了装置路径与指向的规则小节名，没攻。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 15 | 改了 | 2026-09-27 改了：两档三个包、依赖方向由门禁 94 号第 ④ 条判、崩溃点重放的快档与全量只说 54 号（Q7.3） |
| D13（验证路线） 已定项 9 | 改了 | 2026-09-27 改了：层 0 那一行范围不再写「写请求数两位数」（Q7.2），触发与射程是拆分那一轮定的现值 |
| D13（验证路线） 已定项 5 | 不受影响 | 2026-09-27 不受影响：拆分中途加过的「集成测试的 dev-dependencies 不算共享」一句随三个包撤掉，与拆之前相同（Q4） |
