# judging-closure-and-real-card-instance-code-r3：判决（2026-11-19）

正文 `research/prompts/_judging-closure-and-real-card-instance-code-r3-body.md`，开工快照 `research/prompts/judging-closure-and-real-card-instance-code-r3-snapshot/`。这是这一题代码轮的第三轮，也是最后一轮；前两轮判决 `research/prompts/judging-closure-and-real-card-instance-code-r1-main-verification.md`、`research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（第 1 到 4 问、第 6 问） | `research/prompts/judging-closure-and-real-card-instance-code-r3-opus-output.md`（sha256 `940ce772…`） | 白名单放行两个登记单元互调；删掉打行那一层的 println 没有测试红；块后断言只证判表写进自己的库；拿掉守卫那两行走 prove-red.sh 会写真服务；G 行不在这一趟的 14 行里；撤回第二轮攻方腿「对两批都跑」 |
| 辩方（第 5 问、第 6 问） | `research/prompts/judging-closure-and-real-card-instance-code-r3-sonnet-output.md`（sha256 `bade5e43…`） | 第二轮坐实第 2、3、5 条的去处没落（欠账表那一行、同包另 4 处记号行）；认下第 6、7、9 条各有够不着的一半；八格里「测试优先多线程」推不出 |

核查命令（在主 agent 的草稿目录 `~/.local/share/singlefs/work/code-r3-judging-closure-verify/` 下，日志在它的 `main-agent-logs/`）：

- 倒推快照：`python3 scripts/agents/snapshot-tree.py judging-closure-and-real-card-instance-code-r3 --out <草稿>/code-r3-tree --repository <仓根>`，`x0-倒推快照.log`「清单点名 10 份，倒推出 10 份，sha256 对上 10 份」。
- 引文：`python3 scripts/agents/cite-check.py <两份报告> --root <草稿>/code-r3-tree --background research/prompts/_judging-closure-and-real-card-instance-code-r3-background.md`，末行「核了 123 处引文、31 处裸行号，对上 84 处，没判 70 处」，判错 0 处。
- 攻方腿的副本拷进 `copy/`（两处改动的 sha256 与腿报告相同：`mod.rs` 以 `2e58bf51` 起头、`mutations.tsv` 以 `dcb8904d` 起头），三条复跑：
  - `x1-删println.log`：副本里删掉打行那一层的 `println!` 之后，点名那条 e202 测试「test result: ok. 1 passed」，日志里 `NOT_RUN_NO_GPU_ADAPTER test=a_process_whose_cards_cannot_be_read` 0 处，任务状态里没有「本次未跑」。
  - `x2-对照.log`：仓根原样跑同一条，服务推了 gpu-cases-not-run 提醒，原文点名 `a_process_whose_cards_cannot_be_read_is_reported_not_run_to_the_caller`，日志里 1 处记号行。
  - `x3-三行探针变异.log`：合并那一步核账红（「第 0 片（peer）判的 2 行按此刻的行键在 prune-server 里一行都读不到」，跑的中途有人改了判法读的文件），各行的判档取第 0 片日志：`crates/mutations.tsv:8380`（两个登记单元互调）守护测试「没红（退出码 0）」；`:8382`（CPU 参照取样挪出一次性库那一段）派活宽度对拍「没红（退出码 0）」；`:8381`（带 gpu 特性那一行）计划那一步没分给任何一片。
- 主 agent 现查：
  - G 行是 `crates/mutations.tsv:8224`「真卡取卡模块：没跑成的结局打成交了卡的那一行（本次未跑的记号行没打）」，这一批验证段点名的 14 行里没有它；这一轮在主工作区点名交了一趟，结局「计数：抓到 1 条、没红 0 条、无效 0 条、锚点失效 0 行（共 1 条）」，基线不绿 0 行，这一趟取的是判定库里键相同的那一条（「复用：现判 0 条、复用 1 条」），键由此刻被判的函数单元与工具链组成，判的就是今天的代码。
  - 同包另 4 处手写记号行原样在：`crates/singlefs-background-gpu-tests/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:575`、`:599`，`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/construction_phases_and_timing/tests.rs:53`、`:13745`。
  - 两份对拍文件里 `drop(disposable_library` 零命中。
  - `.claude/kb/checks-owed.md` 里 `grep -n 薄壳` 零命中。

## 二、判决

坐实，这一轮改（最后一轮，改法都标零轮；只改测试代码，不做 A/B）：

1. **白名单按名字放行 `gpu_work_trace(`、`gpu_lifecycle_line(`，不看是谁在调**（攻方第 1 问，`x3` 第 8380 行没红）：两个登记的打行单元只用名单里的写法就能互相调用，进程栈溢出、判器倒下，键不变。认下第 6 条那一半（`format!` 带调用方 `Display` 的放行理由推不出，辩方第 5 问）并进这一条：名单里放行的那几样，各写清放行之后不会调到别的登记单元、不会经调用方的 `Display` 走到会 panic 的写法。
2. **打行那一层的 `println!` 被删掉没有东西发现**（攻方第 2 问，`x1` 对 `x2`）：第二轮第 3 条要守的「记号行打得出、任务服务认得出」在打行这一层又空了一块。「上了交来的卡之外的卡」那一支既没有不要卡的测试，也没有变异行。
3. **两条串库对拍的 CPU 参照取样开的是哪一份库没有断言**（攻方第 3 问，`x3` 第 8382 行没红）：块后那句只证判表写进了自己的库。一次性库从建到用例结束一直占着临时服务名额、跨借卡等待（辩方第 6 问「测试优先多线程」一格，`drop(disposable_library` 零命中）。
4. **同包另 4 处手写记号行没改、也没有钉字的测试**（辩方第 5 问，第二轮坐实第 3 条的去处没落；主 agent 现查 4 处原样在）。
5. **第二轮坐实第 5 条（欠账表登一行）与认下第 10 条（D13 已定项 17 写回一句「登记名里的行号只用来认单元、不进键」）都没写回**（辩方第 5 问；主 agent 现查欠账表零命中）：写回 kb，不是代码。
6. **G 行没现判**（攻方第 6 问「带 --item 的命令与项」一格；主 agent 现查不在 14 行里）：这一轮已点名现判。

认下，这一轮不改：

- 拿掉守卫那两行变异（第二轮修补新加的两行）走 `scripts/mutation/prove-red.sh` 时会写真服务的库（攻方第 4 问）：走门禁 crates-mutation-replay 时每次起自己的临时服务，不写真服务；`prove-red.sh` 只在写段证红时由主 agent 交。改成在那两行的备注里写明「证红只走变异门禁，不走 prove-red.sh」，并进第 3 条那一批。
- 显存记账那一批「不沿它收」的登记过不了白名单（第二轮认下第 7 条）：攻方腿撤回第二轮攻方腿「对两批都跑」的改法，辩方腿查到同一批已由前一轮三方判过、欠账 C1033 在 `.claude/kb/checks-owed.md`。这一批照 C1033 那一行办，不在这一题里另立。
- `checking_threads()` 另 3 处调用没人核（第二轮认下第 9 条一半）：行号又漂了（辩方现查 `crash_amplification.rs:34192`），转给管那一处的会话，写今天的行号。

不采：

- 攻方腿第 4 问「命名三处另加检查」：它自己判 naming-lint 改回旧名就会红，不用另加。

核不动：

- `crates/mutations.tsv:8381`（带 gpu 特性编、两个登记单元经 `format_args!` 互调）：计划那一步没分给任何一片，这一趟没有读数；第 8380 行已坐实同一个缺口，第 1 条的改法罩它。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 标 |
|---|---|---|---|
| 守护多判一样：登记的打行单元的函数体里不许调到别的登记单元；`format!` 一族的放行理由写明不经调用方 `Display` 走到会 panic 的写法，推不出的那几样改成不放行；探针那两行写法各做成一行变异 | 第 1 条 | 攻方腿与辩方腿 | 零轮；缺口量过（第 8380 行没红） |
| 打行那一层能在不要卡的测试里核到：判定交回要打的那一行、打行那一层把它写进调用方给的输出（测试里交一个缓冲），一条测试核写进去的就是那一行；「上了交来的卡之外的卡」那一支配一条不要卡的测试与一行变异 | 第 2 条 | 攻方腿 | 零轮；缺口量过（`x1`、`x2`） |
| 两条对拍：CPU 参照取样之后再断言一句「取样结论写进了自己的库」；断言完就丢掉一次性库，再借卡；拿掉守卫那两行变异的备注写明只走变异门禁 | 第 3 条与认下第一条 | 攻方腿与辩方腿 | 零轮；缺口量过（第 8382 行没红） |
| 同包另 4 处手写记号行改用 `not_run_line_of_the_test`（e202 装置那两处与真卡对拍那两处），被它们锚住的变异行跟着改锚点 | 第 4 条 | 第二轮攻方腿、这一轮辩方腿 | 零轮 |
| 欠账表登一行（真卡取卡薄壳要在真卡上量的那两样，三语）；D13 已定项 17 补一句登记名里的行号不进键（三语） | 第 5 条 | 第二轮判决 | 写回 kb |
| G 行点名现判 | 第 6 条 | 第二轮判决 | 这一轮已交，抓到 1 条 |

第 1 至 4 条与用户定的「`crates/` 里 31 处只读查表的 `OnceLock` 改掉」里落在 `crates/singlefs-verification/src/crash_identity.rs` 与 `crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs` 的那几处，文件相交，并进同一个实现员；其余各处另派一个实现员。

## 交用户的

没有要用户定的岔路。告知两件：这一轮是最后一轮，第 1 至 4 条的修补没被再攻过，标零轮；代码轮修补里白名单守护换一种写法（两个登记单元互调）还绕得过，这一轮坐实，下一批补。

## 这一轮被判过的文件

- crates/singlefs-verification/src/crash_identity.rs：判了白名单守护认得出与认不出的写法（第二节第 1 条）、派活宽度那条对拍的一次性库罩到哪（第 3 条）、单元名格式断言与切字样的认法成立。
- crates/singlefs-verification/src/crash_judge_gpu.rs：判了记账分片那条对拍的一次性库与块后断言（第 3 条）。
- crates/singlefs-verification/src/judging_units_entries.rs：判了两处文档注释改写成立，键不变（文档注释不进条目指纹）。
- crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs：判了判定与打行拆开之后打行那一层没有东西守（第 2 条）、`process_identifier` 改名成立。
- crates/singlefs-background-gpu-tests/tests/real_gpu_tests_create_one_vulkan_instance_per_process_and_none_without_handed_over_gpus.rs：判了两条改写改名的测试经判定核交回的整个结构，「上了别的卡」那一支没有测试（第 2 条）。
- crates/mutations.tsv：判了 14 行的名字与改坏的地方对得上；拿掉守卫那两行走 prove-red.sh 会写真服务（认下）；G 行没现判（第 6 条）。
- research/prompts/_judging-closure-and-real-card-instance-code-r3-body.md：两条腿照它的六问作答；「24 条少掉的正是那两条」一句分不出修好与没卡（攻方腿），判决改引对拍日志。
- research/prompts/judging-closure-and-real-card-instance-code-r2-main-verification.md：辩方腿复核了它，坐实第 1、4 条与认下第 8、10、11 条、不采第 12 条站得住；坐实第 2、3、5 条的去处没落（第二节第 4、5 条）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 不影响 | 2026-11-19 不受影响：这一轮判的是测试代码与变异行；写回的一句（登记名里的行号不进键）只把已定的键组成写明，不改定案 |
