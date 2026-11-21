# judging-closure-and-real-card-instance-code-r2：判决（2026-11-18）

正文 `research/prompts/_judging-closure-and-real-card-instance-code-r2-body.md`，开工快照 `research/prompts/judging-closure-and-real-card-instance-code-r2-snapshot/`。这是这一题的第二轮；第一轮的判决是 `research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方腿（three-way-attack） | `research/prompts/judging-closure-and-real-card-instance-code-r2-opus-output.md`（sha256 以 bc602412 起头） | 第 1、2 问推不出（算打中），第 3 问行格式成立、另两处手写打印会漂开（打中），第 4 问成立；八格里「类型与名字」推不出；要跑 4 条探针 |
| 辩方腿（three-way-defense） | `research/prompts/judging-closure-and-real-card-instance-code-r2-sonnet-output.md`（sha256 以 868a6e64 起头） | 第一轮坐实的两处站得住、有两句说宽了；认下的四件站得住（单元名那一件站住一半）；撤回的两句各有一半成立；「两步配对不漏报、停」成立；修补字面都做到、少做两处；没有要跑的命令 |

倒推开工快照（任务 20048，退 0）：

`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/code-r2-verify -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py judging-closure-and-real-card-instance-code-r2 --out /home/fy5090/.local/share/singlefs/work/code-r2-verify/tree --repository /home/fy5090/code/singlefs`

「✓ 快照 judging-closure-and-real-card-instance-code-r2：清单点名 6 份，倒推出 6 份，sha256 对上 6 份」。腿跑着的时候快照里的文件谁都没改。

引文核对（退 0）：

`python3 scripts/agents/cite-check.py research/prompts/judging-closure-and-real-card-instance-code-r2-opus-output.md research/prompts/judging-closure-and-real-card-instance-code-r2-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/code-r2-verify/tree --background research/prompts/_judging-closure-and-real-card-instance-code-r2-background.md`

「✓ 核了 71 处引文、129 处裸行号，对上 30 处，没判 170 处（其中裸行号没有引文可比 56 处；逐处列在上面）」，判错 0 处。没判的另 114 处引的是快照之外的文件（`gpu_unit_checks.rs`、`judging_units_entries.rs`、任务服务与门禁的脚本、第一轮腿报告、调度记录）；判决采的几条，主 agent 在此刻的文件里现查过的写在第二节各条里。

攻方腿的四条探针由主 agent 交（副本是它在草稿里建的那一份，五处探针改动在 `~/.local/share/singlefs/work/code-r2-judging-closure-attack/probe.patch`）：

| 探针 | 任务 | 结局 | 读数 |
|---|---|---|---|
| 1 不带特性，守护测试与活树键测试 | 20043 | 退 0 | 「test result: ok. 2 passed」 |
| 2 带 gpu 特性，守护、活树键、清单三条 | 20044 | 退 0 | 「test result: ok. 3 passed」 |
| 3 记号行写坏（记号后多一个冒号） | 20045 | 退 0 | 「test result: ok. 1 passed」，那一行是 `NOT_RUN_NO_GPU_ADAPTER: test=…`；`task-server/task status` 里「本次未跑」0 处 |
| 4 对照：同一条在主工作区 | 20046 | 退 0 | 「test result: ok. 1 passed」，那一行是 `NOT_RUN_NO_GPU_ADAPTER test=…`；服务推了 `gpu-cases-not-run` 提醒，`task status` 里「本次未跑」1 处 |

## 二、判决

两条腿都有打中的。逐条现查之后：

### 坐实、这一轮改

1. **守护测试认方法调用要求词前面带点，换一种写法就过得去**（攻方腿第 1 问）。现读 `crates/singlefs-verification/src/crash_identity.rs:1817`：`if before.ends_with('.') && (after.starts_with('(') || after.starts_with("::")) && METHODS_THAT_PANIC.contains(&word)`。探针把三个被守的单元改成了三种它认不出的写法——`Option::expect(thread.name(), …)`（与第一轮探针同义）、`std::panic::panic_any(…)`、经共享借用 `card.device_condition.record_an_out_of_memory(…)` 改卡的状态——守护照绿（任务 20043、20044）。这三种都不在实现员列的「认不出」里。第一轮坐实的那条缺口（只打日志的单元里加会 panic 的写法、键不变、没有测试红）换一种写法仍然在。辩方腿另指出第一轮判决「改法 1」字面只写「可变借用」、窄于它自己的诊断（共享借用加内部可变同样改得了卡的状态），成立。
2. **拷树薄壳交空清单，两种编法下都没有测试红**（攻方腿第 2 问）。探针把 `crash_identity.rs:2670` 的 `&data_files_outside_the_crates_the_build_read()` 换成 `&[]`：不带特性那一趟活树键测试照绿（任务 20043），带 gpu 特性那一趟活树键与清单两条都照绿（任务 20044）。现读 `:2706`：`crates/` 之外的包目录另有一段整目录拷，清单里 `prune-server` 的那几份已经被它拷过。实现员报告里「带 gpu 特性那条清单测试照样红」这一句不成立：那条测试不经薄壳。第一轮改法 2 要的「拷贝循环有守护」做到了（现造小仓那一条），薄壳交的是不是那张清单没有守。
3. **同模块另两处手写的记号行打印没有测试钉字，写坏了任务服务认不出**（攻方腿第 3 问，辩方腿第 5 条少做的第一处）。`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:634`、`:642` 现读是两句手写的 `"NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=…"`。探针在第 642 行的记号后加一个冒号：测试照绿，任务服务不报本次未跑（任务 20045）；没改坏的对照有提醒（任务 20046）。攻方腿另读到同一个包里还有 4 处同形的手写记号行（`checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs` 第 575、599 行，`src/bin/e202_gpu_cpu_gc_comparison.rs` 第 8924、12415 行，按线索记），不在这一轮被判的三份里。
4. **`judging_units_entries.rs:1189` 的文档还写着第一轮撤回的说法**（辩方腿第 3 条）。现读第 1189 行：「`gpu_lifecycle_line` 里 `thread.name()` 让方法名 `name` 算见过」。第一轮撤回的是「靠它」；辩方腿查到 `judge_one_dispatch` 里本来就有 `pass.name()`（`crash_judge_dispatch.rs:1726`、`:1811`，按线索记）。文档改成不把「见过」归给这一处。同一份文件的常量文档还要点名新的守护测试（实现员交主 agent 的那一处）。
5. **「合入后要在真卡上量的」没有仓里的登记位**（辩方腿第 5 条少做的第二处）：薄壳交进去的是不是进程级那一格、打没打出拼好的那一行，只写在实现员的草稿报告里。登进欠账表（`.claude/kb/checks-owed.md`，三语），写明量法与前置（GPU 服务代跑测试的接口）。

### 认下、记账不在这一轮改

6. 守护认不出而今天走不到 panic 的五样（格式化宏展开、`writeln!`、`std::thread::current()`、`static ORIGIN`、带锁的 `&GpuCard`）：攻方腿读代码推的，今天都不 panic。第 1 条的改法（见第三节）若改成白名单，这五样要逐个列进去并写明为什么放行。
7. 更早登记的显存记账那一批「不沿它收」的单元有同一个形状的缺口（辩方腿第 1 条）：这一题只落在新登记的三个单元上。记账：第 1 条的改法做成对两批登记都跑。
8. 第一轮判决两句说宽了（辩方腿第 1 条）：「经借用改卡的状态」那一半第一轮没有读数、并在「量的」下面，这一轮探针 1、2 量到了；「拷循环没有东西守」其实有一道有条件的守（摘要比对，今天没有正例）。第一轮判决是冻结证据不回改，更正写在这里。
9. 第一轮认下没改的四件，辩方腿都判站得住；「单元名只在字符串里约定」那一件它替另一方写的更轻的做法（一条钉格式的清单断言，产出处今天数得 8 处）并进第 1 条那一批的实现员、由它判做不做得成一条测试。`checking_threads()` 另 3 处的行号已经漂了（辩方腿现查 `:31746` 现在是 `:32438`，按线索记），转给管那一处的会话时给今天的行号。
10. 红句「空表」的措辞、带行号的登记名与 kb 条款「行号不进键」的关系（辩方腿第 2 条）：D13（验证路线） 已定项 17 写回时补一句登记名里的行号只用来认单元、不进键。
11. 攻方腿的两处小处：D 行少 `--exact`；G 行在任务 19955 里是复用的判定、它头一次现判在哪一趟腿核不动。D 行随第 1 条那一批补上 `--exact`；G 行在那一批的验证段点名现判一次。

### 不采

12. 攻方腿第 1 问反方向（守护误报会逼出绕法）：今天没有发生，三个函数体自登记以来没改过；往后的影响是推的，不采成打中，写进第 1 条改法的验收里（误红时出路写清）。

### 两步配对那一格

辩方腿复核第一轮「两步配对不漏报、停」：两条腿当时都收到这一问、都没打中，停成立，不再派同立场腿。它补的两件（名字归属可能点到内容逐字相同的另一个单元；按行号起名的单元被删、新加还没有测试输入）记账，并进第 1 条那一批由实现员判能不能各补一条用例。

### 这一轮的结局

第二轮有打中的，坐实 5 条。照代码轮的收法：改完再攻第三轮（最后一轮），攻这一轮的修补与这一轮写明按线索记的地方；第三轮改的修补在判决里标零轮。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 标 |
|---|---|---|---|
| 守护从「认会 panic 的写法」改成白名单：被守的单元的函数体里只许出现列出的调用与宏，别的一律判红；对两批「不沿它收」的登记都跑；探针的三种写法各做成一行变异 | 第 1、6、7 条 | 攻方腿 | 零轮；缺口量过（任务 20043、20044）。只改测试代码，不做 A/B |
| 薄壳交的清单配守护：一条变异把 `crash_identity.rs:2670` 换成空清单，配一条在带 gpu 特性时会红的测试（清单里放一份不在 `crates/` 之外任何包目录底下的文件才分得出，实现员判今天的清单里有没有这样的文件；没有就写明今天判不出、登欠账） | 第 2 条 | 攻方腿与主 agent | 零轮；缺口量过（同上） |
| 模块里另两处手写的记号行改用 `not_run_line_of_the_test`，被它们锚住的两行旧变异跟着改锚点；同包另 4 处同形的一起改或各配钉字的测试 | 第 3 条 | 攻方腿 | 零轮；缺口量过（任务 20045、20046） |
| `judging_units_entries.rs` 两处文档：第 1189 行那一句改写；常量文档点名新的守护测试 | 第 4 条 | 辩方腿与实现员 | 只改文档注释；文档注释的增删照代码算，随这一批合入后验证 |
| 欠账表登一行「真卡取卡薄壳交的是不是进程级那一格、打没打出拼好的那一行」 | 第 5 条 | 辩方腿 | 写回 kb，不是代码 |
| D 行补 `--exact`；G 行验证段点名现判 | 第 11 条 | 攻方腿 | 量过的是现状 |

这一批不改产品代码（验证包的测试模块、产品 GPU 测试包的 `tests/` 与一处文档注释），没有要先做 A/B 的零轮改法。落之前照旧先问此刻在往验证包里落补丁的会话。

## 交用户的

没有要用户定的岔路。告知一件：第一轮的修补加的守护测试，换一种写法就绕得过（第二节第 1 条），这一轮坐实了、下一批改成白名单。

## 这一轮被判过的文件

- crates/singlefs-verification/src/crash_identity.rs：判了守护测试认得出什么（第二节第 1 条）、拷树薄壳交的清单有没有守护（第 2 条）、新变异行与名字对不对得上（成立）
- crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs：判了拼行函数的行格式与读它的三方认法（成立）、另两处手写的记号行（第 3 条）
- crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs：判了四条新测试证明的是不是它们名字说的事：成立，「先读后建」那一条的限度是判红那一句不看 panic 红在哪一步
- research/prompts/_judging-closure-and-real-card-instance-code-r2-body.md：这一轮的正文；八格检查单七格成立，「类型与名字」一格推不出（守护里又多一处按 `::` 拆限定名，属第一轮记账的那一类）
- research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md：辩方腿复核了它：坐实的两处与「停」站得住，两句说宽了、改法 1 字面窄于诊断（第二节第 1、8 条）

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-11-18 不受影响：这一轮判的是守「不沿它收」登记的测试罩不罩得住，条款本身（键只由函数单元、工具链、依赖版本组成）没被推翻；写回时点名的常量与守护测试等第三轮收了再写 |
