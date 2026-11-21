# judging-closure-and-real-card-instance-code-r1：判决（2026-11-18）

正文 `research/prompts/_judging-closure-and-real-card-instance-code-r1-body.md`，开工快照 `research/prompts/judging-closure-and-real-card-instance-code-r1-snapshot/`。这一题的第一轮，之前没有判决。被判的是已经合进主工作区的一批 `crates/` 改动（打补丁任务 19768、回读 19769）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方腿 | `research/prompts/judging-closure-and-real-card-instance-code-r1-opus-output.md`（sha256 `0e3953541a5ddfed9bca1642fca4ed939ef848838441754e0143277f142cc867`） | 「1a 打中」「4b 打中」「5a…推不出谁对…算打中」「面二打中两处」；「3 两步配对没打中」「1c 第二批登记对键没有任何作用」 |
| 正推腿 | `research/prompts/judging-closure-and-real-card-instance-code-r1-sonnet-output.md`（sha256 `dd9ed6f0ddd62f84616adff3ba2b1ccf8a5c2c5bcfb9324594712e4cc2256b6a`） | 「键过窄：不窄」「units_changed_between：成立，不漏报」「「漏拷就拒算」今天没有正例」「没罩到的有三处」 |

核查：

- 开工快照：腿交齐之后在仓根跑 `sha256sum -c` 核快照目录里三份清单（`crates-sha256.txt` 4 份、`judged-sha256.txt` 3 份、`kb-sha256.txt` 1 份），8 行都是 OK；腿跑着的时候没有人改过被判的文件，不用倒推整棵树。
- 引文：两条腿各自跑过引文核对，末行原样——攻方腿「✓ 核了 37 处引文、4 处裸行号，对上 37 处，没判 4 处（其中裸行号没有引文可比 4 处；逐处列在上面）」；正推腿「核了 64 处引文、61 处裸行号，对上 53 处，没判 72 处（其中裸行号没有引文可比 38 处）」，它没判的那些另用 `grep -nF` 逐处核过（这一句是它的说法，主 agent 没逐处复核）。
- 两条腿结论不一致的那一格（「不沿它收」会不会让键过窄）由主 agent 去量能分辨它们的事实：在攻方腿的副本（主工作区加它的探针补丁，补丁在它草稿目录的 `attack-probe.patch`）里交它的两条命令。
  - 任务 19825（带 gpu 特性点名 9 条）退 101：「test result: FAILED. 8 passed; 1 failed」，红的只有探针新加的 `gpu_unit_checks::tests::the_gpu_lifecycle_line_on_a_thread_without_a_name_does_not_panic`（「thread '<unnamed>' … panicked at crates/singlefs-verification/src/gpu_unit_checks.rs:1055:23: GPU 阶段现场那一行只在有名字的测试线程上打」）；守登记的四条、`an_edit_of_the_listed_gpu_lifecycle_line_keeps_the_gpu_judge_key`、探针的 `an_edit_making_the_listed_gpu_lifecycle_line_panic_on_threads_without_a_name_keeps_the_gpu_judge_key`、`the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree` 都是 ok。
  - 任务 19832（不带特性点名 1 条）退 0：副本里删掉了拷数据文件的那个循环，`the_judging_units_keys_compiled_in_are_the_ones_computed_from_the_tree`「1 passed」。
- 核不动的：攻方腿说判器的 `scope.spawn` 线程与 GPU 服务的卡工作线程都没有名字（它引 `gpu-verification-server/src/service_loop.rs:270`），主 agent 没在真服务上量；探针只证明「没名字的线程上那一行会 panic、键不变」，没证明真服务里那些线程此刻没有名字。

## 二、判决

逐条，对着正文「要腿判的」六条。

1. 「不沿它收」的两批登记。
   - 今天的代码：三个登记的单元只打日志、不改判定结局，两条腿读代码都这样判，没有分歧。
   - 守这件事的检查不够，坐实（量的，任务 19825）：把 `gpu_lifecycle_line` 里取线程名的那一处改成 `expect`，没名字的线程上它 panic，而 GPU 判器那一组的键不变、守登记的四条测试照绿。条款要的是「不沿它收的单元不改判定结局」，现在只有「不交值、交的值没人读」一条测试在守，panic 与经借用改卡的状态这两条路没有东西守。正推腿判「不窄」说的是今天的代码，与这一条不冲突：键今天不窄，是守不住以后的改动。
   - 四个「只经它们够得到」的单元：性质成立（两条腿一致）。实现员对边的解释撤回一半：方法名 `name` 算见过不靠 `thread.name()` 那一处，留下的入口里 `crates/singlefs-verification/src/crash_judge_dispatch.rs:1726` 的 `pass.name()` 已经让它算见过（攻方腿读的，主 agent 没另核）。这一批登记对键没有作用，只为那条记账测试服务。
2. 带行号的登记名：实现员那句成立，陈旧之后键不变宽也不变窄，同趟测试会红。代价两条腿都点了：`crates/singlefs-verification/src/lib.rs` 第 78 行之上加删一行，这条登记就陈旧、测试就红，而键一个字不变。不算打中，记成已知代价。
3. `units_changed_between` 的两步配对：两条腿都没打中（攻方腿给了「每次只拿掉种类与指纹都相同的一对，剩下的差与键的差一项不少」的证明，正推腿正推四种输入都不漏报）。这一格停。两条腿各带出一处小的：第二步配对里「指纹也相同」那个条件没有变异行（正推腿）；键变而组成清单没变时红句说成「空表」，这是先前就有的措辞（两条腿都提）。
4. 数据文件清单那条测试的修补。
   - 前提「线程数只决定盘切成几段、不改判定结论」：对被改的那一处（`crates/singlefs-verification/src/crash_amplification.rs:31746` 一带）两条腿读代码都判成立。实现员「唯一读 `checking_threads()`」那一句撤回：攻方腿数出产品代码里有 4 处调用，另外 3 处两条腿都没核出结论。
   - 「漏拷 `crates/` 之外的数据文件就拒算」没有东西守，坐实（量的，任务 19832 与 19825）：副本里删掉拷数据文件的循环，读活树重算键的那条测试两种编法下都照绿。变异行「拷树重算 漏拷带 gpu 特性编时构建读到的 crates 之外的文件」名字写的是漏拷，弄坏的其实是清单（攻方腿读的）。
   - 钉今天清单长什么样的那两条：是该有的警报。正推腿指出带 gpu 那一条里「都在 `prune-server/src/` 底下」那一半会在判定库服务加目录时红，属布局而不属这一条要守的事。
5. 真卡测试共用实例。
   - 进程级 `OnceLock`：留。两条腿都认为该有；它与 `.claude/rules/code-discipline-by-review.md`「类型、数据与封装」里留下的「不用全局可变状态；不用单例」字面冲突，那一条没写出路，同文件已有的先例同样没有豁免。规则要不要补出路交用户。
   - 三条不要卡的测试没罩到的：薄壳里交进去的是不是那一个静态项（实现员已认）；建实例 panic 之后下一次取会再建一个（攻方腿）；共用模块里两行打印没有测试也没有变异行，其中一行是门禁与任务服务都读的 `NOT_RUN_NO_GPU_ADAPTER` 标记行（正推腿，`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:766`、`:771`）；取实例与读 `nvidia-smi` 的先后（正推腿）。算打中：标记行那一行改坏了，门禁就读不到「本次未跑」。
6. 八格检查单：走不到的理由、不兼容旧代码与旧格式、不新建判定库、带 --item 的命令与项、新崩溃流多罩了什么五格两条腿都判成立。「所有任务多机可用」成立，正推腿没核 `--all-features` 等编法下清单的样子。「测试优先多线程」成立，三条拷活树的测试并行时的内存两条腿都推不出（主 agent 的读数：这三条在任务 19825 里与别的六条同跑，整趟 145.86 秒过了）。「类型与名字」攻方腿判推不出、算打中：单元名的格式（限定名里的 `@<行号>`）只在字符串里约定，这一轮的 `unit_name_without_line_numbers` 又加了一处按这个格式拆字符串的地方。
7. 面二（内部矛盾）：登记、守它的测试、变异行三者里，独立守「不改判定结局」的只有交值那一条——并进第 1 条。`.claude/gate.d/crash-point-replay.sh:314` 的注释写「crates 里这几行在判器的单元闭包里，动它们换键，所以滤在这里」，与这一次把打这几行的单元登成不沿它收矛盾：坐实（主 agent 现读那一行），改注释。

## 三、改法

改完再攻一轮（第二轮派辩方腿复核这一份判决）。落 `crates/` 的几条合成一簇交一个实现员，落之前先问正在起验证包整包的那个会话。

1. 给「只打日志的单元」加一条守「不 panic、不经可变借用改东西」的测试（并进 `crates/singlefs-verification/src/crash_identity.rs` 守登记的那一组）：登记的每一个单元的函数体里没有会 panic 的写法（`expect`、`unwrap`、`panic!`、`assert!` 一族、下标取值），参数里没有可变借用；对照：认得出探针那一处 `expect`。配变异行。标「量过」：没有这条检查时探针那一处改动键不变、测试照绿（任务 19825）。新检查写出来之后要在同一处改动上红，那一半等实现员写完由证红量。
2. 拷树重算拷 `crates/` 之外数据文件的那一段配一条带正例的测试：把「要另拷哪些文件」做成拷树那一步收的参数（或等价的可注入形态），用例交一张含一份 `crates/` 之外文件的清单、核它被拷到；删掉拷贝循环这条要红。原来那一行变异的名字与它弄坏的地方对不上，改名或改弄坏的地方。标「量过」（缺口量过：任务 19832）。
3. 真卡共用模块：两行打印各配测试与变异行（至少 `NOT_RUN_NO_GPU_ADAPTER` 那一行的文字与何时打）；薄壳里交进去的是不是那一个静态项、取实例与读 `nvidia-smi` 的先后，实现员判能不能在不要卡的测试里罩住，罩不住的写明并列进「合入后要在真卡上量」。建实例 panic 之后再建一个：写明这是不是要守的行为，是就配测试，不是就在文档里写清。标「零轮」：这几条是腿提的、没量过；它们加的是测试，不改产品代码，不做 A/B。
4. 第二步配对的「指纹也相同」条件补一行变异。标「零轮」，同第 3 条。
5. 带 gpu 的清单那一条测试去掉「都在 `prune-server/src/` 底下」那一半，只留「有 `prune-server/src/lib.rs`」。标「零轮」，同第 3 条。
6. `.claude/gate.d/crash-point-replay.sh:314` 的注释改成现状（打这几行的单元不在判器那一组的键里，滤掉是因为量大而没有判法读）。治理文件，并进这一轮治理文件那一批。
7. 写回 D13（验证路线） 已定项 17：点名的常量从两个变四个、守它们的测试照今天的写；等这一题收了再写回。
8. 不改、记账的：带行号的登记名的代价；红句「空表」的措辞；单元名格式只在字符串里约定（立 newtype 是另一件）；`checking_threads()` 另外 3 处调用没人核——转给今天改了那一处的会话。

## 交用户的

- `.claude/rules/code-discipline-by-review.md` 留下的「不用全局可变状态；不用单例」没有出路：测试进程里「一个进程只建一次」的资源（Vulkan 实例、按需借的卡）今天用进程级 `OnceLock`，同文件已有两处。要不要给这条规则补一句出路（哪一类可以、要配什么），由用户定；定之前这两处照留。
- 这一批合入时带 gpu 特性的 lib 目标有 27 条因任务里拿不到卡而红（与合入之前的基线逐条相同），照「记已知红、不挡合入」合的；那句口径是用户对另一批说的，这一批没有单独问到答案。

## 这一轮被判过的文件

- crates/singlefs-verification/src/judging_units_entries.rs：判了两批新登记的「不沿它收」单元——今天不改判定结局成立，守这件事的检查缺 panic 与可变借用两条路（坐实，改法 1）；第二批对键不起作用
- crates/singlefs-verification/src/crash_identity.rs：判了 `units_changed_between` 的两步配对不漏报（两条腿都这样判）、守登记的五条测试各证了什么、数据文件清单三条测试丢了「漏拷就拒算」的守护（坐实，改法 2）与第二步配对少一行变异（改法 4）
- crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs：判了进程级实例该留、与规则字面冲突交用户；两行打印与薄壳那一处测试罩不到（改法 3）
- crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs：判了三条不要卡的测试证了「建一次、同一份」「服务不交卡就不建」两支，漏了改法 3 列的几处
- research/prompts/_judging-closure-and-real-card-instance-code-r1-body.md：判了八格检查单，七格成立、「类型与名字」一格推不出；正文引的实现员两句说法撤回（方法名那条边的解释、「唯一读 checking_threads()」）
- .claude/rules/verification.md：判了单元键那一条对这一批登记成立，「不沿它收」要配的守护比条款写的少两条路
- .claude/rules/code-discipline-by-review.md：判了「不用单例」那一条对测试进程里只建一次的资源缺出路，交用户

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-11-18 改了：条款点名的不沿它收的登记与守它的测试要照今天的四个常量写回，并加「不 panic、不经可变借用改东西」这一条守护；写回等这一题收了再做 |
