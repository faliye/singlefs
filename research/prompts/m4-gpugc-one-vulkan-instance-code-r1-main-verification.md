# m4-gpugc-one-vulkan-instance-code-r1：判决（2026-11-09）

正文 `research/prompts/_m4-gpugc-one-vulkan-instance-code-r1-body.md`，改动全文 `research/prompts/_m4-gpugc-one-vulkan-instance-code-r1-diff.md`，开工快照 `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-snapshot/`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-sonnet-output.md` | T1、T2、T4 成立，T3 部分成立；检查单八格成立，三处带限定；`self.clone()` 没钉住，给了钉法 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-opus-output.md`，模型 `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-opus-model/` | 打中 7 处，其中打中 3、5、6、7 与这一批的改法有关，打中 1、2、4 的代码形状改前就有；测试优先多线程一格字面不成立 |

主 agent 的核查（量的标「量的」）：

1. 开工快照：`sha256sum -c research/prompts/m4-gpugc-one-vulkan-instance-code-r1-snapshot/crates-sha256.txt` 三份都 OK（量的）。`kb-sha256.txt` 里 `.claude/kb/milestone/04-fourth-txn.md` 对不上：腿跑着的时候主 agent 改了这一份三语里「优化项」表第一行，违反 `.claude/rules/implementation-workflow.md`「代码轮派腿之前记一份开工快照」那一句。快照值与 `git show HEAD:.claude/kb/milestone/04-fourth-txn.md` 的 sha256 同为 `56720fe8b0726bef`（量的），所以开工那一刻的原样就是 HEAD；两份腿报告都没有引这份文件（`grep`，量的），不影响判决。
2. 引文：`python3 scripts/agents/cite-check.py <两份腿报告> --root ~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1/repo --background research/prompts/_m4-gpugc-one-vulkan-instance-code-r1-background.md`，末行「✗ 核了 25 处引文、99 处裸行号，1 处对不上，100 处没判」（量的）。对不上的一处是攻方腿把 `.claude/rules/implementation-workflow.md` 的行号写成 79（条款「不能并行的写明为什么」在 `.claude/rules/test-script-discipline.md`「编译与算法都要快」），不改结论。没判的多是仓外 wgpu 源码路径，判决只依赖下面几条量到的读数。根用的是冻结副本（`git archive` 自提交 `8ed2ea484`），它就是开工快照那棵树，没另经 `snapshot-tree.py` 倒推。
3. 攻方模型复跑，任务 10414（`~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-main-check/opus-model/tools/rerun-from-a-clean-work-directory.sh`）退 0，末行「name=rerun_summary mismatches=0」（量的）；`name=product_takes_another_instance_under_the_new_test expected_rc=101 rc=101`、`name=product_takes_another_instance_under_the_existing_library_tests expected_rc=0 rc=0`、`name=present_now_on_another_instance_under_the_existing_library_tests expected_rc=0 rc=0`；`OPEN_ARM name=wait_limit_zero_not_named refused=Some(AdapterEnumerationFailed { cause: WaitForTheGpuExceededTheLimit { wait_limit: 0ns } })`；`FUTURE_BUILT_BEFORE_THE_DEADLINE outcome=Ok(7) elapsed_millis=300`；`LOADER_MAPPED lines_naming_libvulkan=0`。外层脚本只多一步「每次尝试先清草稿目录」。
4. 真卡模型，任务 10428（第二台）退 0：`VIEW hidden_at_creation=0 same_instance_after_the_drivers_came_back=0 new_instance_after_they_came_back=3 same_visible_instance_after_hiding=3 new_instance_after_hiding=0 lines_naming_libvulkan=45`（量的）。模型只换环境变量，没真拔卡、真装驱动。
5. 正推腿钉法，任务 10375 退 0：`HELD_INSTANCE_GREEN rc=0`、`HELD_INSTANCE_NEIGHBOUR_GREEN rc=0`、`HELD_INSTANCE_MUTATION_APPLIED count=1`、`HELD_INSTANCE_MUTATED_RED rc=101`（量的）。核查副本的脚本多了范围声明，并把「拷贝目录已在就拒」换成先删。

「没打中」两腿各只抽了一份样本（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）；下面的「照旧」靠读码与第 3–5 条的读数，不靠腿的否定结论。

## 二、判决

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-background-gpu/src/background_gpu.rs` | **要改**：打中 2、5 改代码与测试，打中 1、4、6 改文档 |
| `crates/singlefs-background-gpu/src/background_gpu_compute.rs` | 照旧 |
| `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs` | 照旧；`vulkan_instance_for_the_gpu_arm` 一处文档措辞小误随修补一起改 |

打中逐条：

| 打中 | 现查 | 判决 |
|---|---|---|
| 1 枚举真挂住时上限管不到 | 第 3 条 `FUTURE_BUILT…elapsed_millis=300` 量的是「造 future 那一步的活不在上限之内」；`background_gpu.rs:179` 与 `:823` 的文档说得比代码强。改前就有 | 文档改成实话；真挂住时怎么拒（枚举放线程）登欠账，归运行库那一轮设计。kb 字面没点名枚举这一步，不算违反条款 |
| 2 等待上限 0 拒成 `AdapterEnumerationFailed` | 第 3 条量的。违反 `background_gpu.rs` 「成员按调用方要改哪一样分」那一句，没有落盘 | **采**：枚举之前先判，拒成 `SettingsRefused(GpuWaitLimitOfZero)`，配测试与变异 |
| 3 没点名时只报 `GpuNotPresent`、没有适配器清单 | 第 3 条 `PURE_ARM name=not_named_only_software refused=Some(SettingsRefused(GpuNotPresent))`，符合 D26（后台整理与放置回收） 已定项 13 第 4 条 | **不采**：不违反条款，是射程；要清单另议 |
| 4 切换那条注释只说对一半 | 腿引的注释在报告「打中 4」一节 | **采**：只改注释 |
| 5 `self.clone()` 没钉住 | 第 3、5 条量的，两腿独立给出同一个钉法。正文那行变异删掉之后确实空着 | **采**：用例 `the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one` 与变异行（正推腿 `proposed/`）。这条钉法被攻过零轮。现查本机独显那一份实例不要卡钉不住，不做 |
| 6 进程级实例把驱动清单冻在建它的那一刻 | 第 4 条量的（只换环境变量那一支）。这一批新带来的行为，今天够不着产品（`crates/` 里没有跨挂载的长进程调用方） | **文档写明**；怎么处置归运行库那一轮设计：运行库是第一个跨多次挂载的调用方 |
| 7 「8 小于 12」只对一套驱动成立 | 是正文的外推，今天不破。两腿数法不同：攻方按用例数，正推按真正建实例的点（最多 5） | 不改代码；账本第二百零二节已登 |

检查单：测试优先多线程一格采「不成立（字面）」：30 轮那条补一句为什么串行（同一份实例上反复开卡，次序就是被测对象）。其余七格成立；「带 --item」里变异行能不能被 `crates-mutation-replay --list-items` 列出由修补那一批交命令核。

这一轮打中的语义只有打中 2（拒绝归错成员，不落盘）。修补并进运行库那一批，随那一批的代码轮一起攻，不为它单开第二轮（`.claude/main-agent.md`「一轮怎么开、怎么收」第 10 条：代码三方一轮攻这一批合入）。

### 交用户的

- 打中 5 的钉法被攻过零轮。
- 打中 1、6 都落到运行库那一轮设计。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 不影响 | 2026-11-09 不受影响：第 4 条「配错时拒」照旧成立，打中 2 改的是拒成哪个成员；打中 1 的枚举条款字面没点名 |
