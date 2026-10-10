# 根环实现第 9 批：代码轮第三轮正文（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 RS1 RS2 RS3 RS4 RS5 RS6 R2 Q5 F3 F5 F6 -->

## 一、题面

第 9 批九份补丁都合进了 `crates/`（`records/2026-09-30-里程碑三开工第一轮.md` 第 9 批各条）。这一轮是根环实现代码轮的第三轮，也是最后一轮（`.claude/rules/three-way-inference.md`「多轮」一节：第三轮之后停）。照第三轮的分工攻三样：前两轮站住的「已定」形态撑不撑得住，未定项与欠账清单全不全，这一批报出的数能不能核。前两轮判决：`research/prompts/m3-root-ring-code-r1-main-verification.md`、`research/prompts/m3-root-ring-code-r2-main-verification.md`。

改动全文在 `research/prompts/_m3-root-ring-code-r3-diff.md`（九份，各带实现员报告的路径），冻结副本 `/tmp/claude-1000/m3-root-ring-code-r3/tree/`：

| # | 改动 | 主要落点 | 压着的条款或定案 |
|---|---|---|---|
| A | RS1、RS2（进程内记根前的重读三种结局、按哪张实例表判）、RS3（准入抬 F 的上限并进窗口里挂载那一刻读不出、之后没写过的槽）、`mount_transparency.rs` 照 R2 判系统配置见证 | `crates/singlefs-core/src/allocator.rs`、`mount.rs`、`crates/singlefs-harness/src/mount_transparency.rs` | `.claude/kb/decisions/16-发布语义.md` 已定项 1「抬 F 的上限」「根槽这一次读坏」两行（用户 2026-10-04 定 Q5：只并读不出的槽）；D22（单元原子性怎么合成） 已定项 8 射程的 R2 |
| B | 实例表清扫器：挂载末尾判删行、抹头、清扫发布，开关与阈值，F5 观察者 | 新文件 `crates/singlefs-core/src/instance_table_sweep.rs`、`mount.rs`、checker 档 `tests/crash_points_of_the_instance_table_sweep.rs` | `.claude/kb/decisions/18-块里携带什么信息.md` 已定项 11（清扫的条件、次序、开关与阈值） |
| C | I-7.9 两支（准入抬 F、带卸载记号）的「之前的根」取两个一圈窗口的交、交空不判；I-7.8 认领按整单元跳；新立 I-9.28 与第四种判定 `PartiallyJudged`；RS4 六张坏镜像 | `crates/singlefs-checker/src/walk.rs`、`image.rs`，全仓 `InvariantVerdict` 穷举 match | `.claude/kb/invariants.md` 的 I-7.8、I-7.9、I-9.28 三行与文件头「部分没判」那句（用户 2026-10-04 定「乙，两支都取两个窗口」与「加第四种判定」） |
| D | GPU 判器：I-3.5 上 GPU、删临时表回全覆盖、RS5（恢复内核卸载记号其余位、七段取记录）、RS6（走读内核施加版取记录），9-9 补 I-9.28、I-7.8 认领、I-7.9 两窗口、覆盖表第 68 号 | `crates/singlefs-checker-tier/src/crash_judge_*.wgsl`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs` | 同 C；GPU 与 CPU 判器逐状态对拍（`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`） |
| E | 改钉：删 `rebuilt_allocator` 里没有消费者的那一句、RS1 测试文件改名、合入后红的那条照 RS3 改钉；点名项上限 67 → 64 的取样点；固定脚本到 E 再卸载那条流 477 → 481（回退那次发布多写谱系容器与谱系树根）与两份崩溃放量用例的断言 | 各份 harness 与 checker 档测试、`crates/mutations.tsv` | 改钉簇、点名项门槛簇、9-8、9-8b 各自的报告 |

## 二、实现今天的样子（主 agent 的观测，2026-10-04）

- 冻结副本取自 2026-10-04 主工作区：第 9 批九份都打上之后。删除与并发那条线第 14、16 批与形式化接入那条线第 21、24 批也已合入，被判文件的 sha256 在 `research/prompts/m3-root-ring-code-r3-snapshot/crates-sha256.txt`。
- 合入后验证三趟（`/tmp/claude-1000/merge-and-verify/2026-10-04-888342/`、`2026-10-05-2211358/`）：harness 档 149、150 个测试二进制全绿，fmt 过；后一趟 clippy 红 4 处，都在形式化接入那条线还没进 git 的 `e171_three_oracle_collusion_after_the_oracle_fixes.rs` 与 RefFS，不在被判范围。checker 档快档主 agent 正在跑，结果出来之前腿不引它。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中，判决里照原去向写）：GPU 判器候选过滤、由记录施加的版本、回收门槛三处没有一圈窗口（`.claude/kb/checks-owed.md` C705）；清扫的块设备错两支没有用例；挂载透明不认清扫发布；I-3.1 在走不到的那一代上改成 RCW、取号跳过改认出生代号（用户 2026-10-04 定，下一轮实现）；回收门槛取一圈窗口在另一轮三方 `research/prompts/_m3-reclaim-window-r1-body.md` 里重判，这一轮不攻回收门槛。
- 实现员自取的读法：层 0 计数把「部分没判」并进「不适用」那一格（改钉簇报告第六节）；带卸载记号那一支两窗口交空时「不判」（CPU 与 GPU 两侧都这样取）；RS3 只并挂载那一刻读不出的槽（用户已定 Q5）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　清扫器（B）
这一批唯一一块前两轮没人看过的新代码。清扫的判删行条件、抹头与清扫发布的次序与 D18 已定项 11 逐句对得上吗；崩在抹头与清扫发布之间的每个状态，再挂载是不是照「那一行本来就未发布」处置、池级 checker 不红、被删实例的单元一个都不漏回收也不早回收；清扫器自取的两格（根环里一条可读根的账读不出就不删、池里有写序实例大于挂载根的单元就不删）会不会在一段合法历史上让一行永远删不掉。

### K2　RS1–RS3 与 R2（A）
前两轮站住的 E2、B2、RR4 之上，RS1 的三种结局、RS2 按哪张实例表、RS3 只并读不出的槽：有没有一段合法历史让 F 越过一条真住着非空有效根的槽（不安全方向），或让抬 F 被一个从没住过根的槽永远挡住；`mount_transparency.rs` 照 R2 判的那几支与 core 的 R2 是不是同一个口径。

### K3　I-7.9 两窗口、I-9.28 与「部分没判」（C）
两支各取两个窗口的交、交空不判：有没有一张坏镜像（F 越过了上限）因为交为空而漏红；I-9.28 四种结局的边界（树表读不出、条目比字段表短、没有一棵树出现在两个单元里）各自判得对吗；「部分没判」在层 0 并进「不适用」那一格之后，有没有一条流因此把一个本该判的状态算成判过；全仓穷举 match 里新成员走的那一臂有没有一处算成了成立或违例。

### K4　GPU 与 CPU 同口径（D）
I-3.5、I-9.28、I-7.8 认领、I-7.9 两窗口、RS5、RS6 各自在 GPU 内核里与 CPU 判法逐要素对得上吗；I-9.28 部分没判与不适用两形 GPU 不置成立位、对拍按 CPU 判 `Holds` 才算成立位，有没有一形两边对不上；覆盖表补第 68 号之后，覆盖断言能不能被一个新立而 GPU 没判的不变量绕过。

### K5　数能不能核（E）
481 次写、全量 1678377204、两态闭式 1678377089、点名项上限 64 的取样点、`root_slot_already_unreadable_at_mount…` 那条改钉后的值：每一个数从哪一段代码推出来、钉它的测试与变异行改回旧值时红不红；有没有一个数只在实现员的临时探针上核过、仓里没有东西钉着。

### K6　欠账与未定项清单全不全
这一批的九份报告里「我取了读法的」「做不出会红用例的」「要主 agent 判的」每一条，在 `.claude/kb/checks-owed.md`、判决或记录里都有去向吗；有没有一条被无声丢了。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K2、K3、K4、K5、K6 | 逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行；前两轮站住的不重推，推这一批改了的与第三轮该问的三样 |
| 云端攻方（Opus） | K1、K2、K3，重点 K1 | 在冻结副本的拷贝上造反例，只跑 harness 档；不许重复前两轮攻过的角度（两份判决第二节各格），攻前两轮站住的形态撑不撑得住、这一批的零轮定案（Q5、乙、部分没判）与清扫器 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；这一轮是第三轮，打中的新形态与腿提的收严都记「零轮」，交用户或登记实验，不再为它开第四轮。
- 第二节列为已知没做的几样，腿报出来不算打中，判决里照原去向写。

交付：腿的报告写 `research/prompts/m3-root-ring-code-r3-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-root-ring-code-r3-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的行号。方案按 `crates/` 今天的实现来谈。
