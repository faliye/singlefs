# gpu-cases-without-cards-code-r1：验证包的 GPU 用例没交卡串报本次未跑，连同变异表的锚点与筛选词，代码轮第一轮正文（2026-11-21）

## 一、题面

攻这一批合进主工作区的 `crates/` 改动，两样：

1. 验证包取卡池的入口照调用方交没交卡串判。改到 `crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs` 与 `crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs`，`crates/mutations.tsv` 追加 4 行。
2. `crates/mutations.tsv` 的整行替换 2102 行（拆分之后失效的锚点与筛选词），加删掉的 1 行。

- 条款：`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」那一条里的两句原话：「验证包的测试与崩溃判段不借卡：GPU 用例没交卡串的报本次未跑」「崩溃判段之外用卡的测试交 GPU 服务代跑（接口待定）」。用户定案的原话在 `records/2026-11-13-全量提交门禁前收尾调度.md` 第 1645、1647 行：「E202 真卡验证才能借卡 其他的拒绝。 白名单借卡」「除了白名单，其他一律走统一的gpuserver。没有其他的逻辑 删掉这部分无用的逻辑」。变异表那一样的条款：`.claude/rules/mutation-sampling.md`「第七类：变异压根没跑——锚点在源码里不再唯一命中」与 `.claude/rules/verification.md`「测试不分档，红了只重跑红的」里「变异行点名的测试要跑得到」那一条。
- 第 1 样合进主工作区的完整改动（合入之前那一刻 → 合入之后）：`~/.local/share/singlefs/work/merge-and-verify/2026-11-21-931572/merge-patch/crates.patch` 与同目录 `mutations-append.tsv`；实现员三份报告依次承接：`~/.local/share/singlefs/work/gpu-cases-without-cards-report-not-run/result.md`、`~/.local/share/singlefs/work/gpu-cases-without-cards-report-not-run-second/result.md`、`~/.local/share/singlefs/work/gpu-cases-without-cards-report-not-run-third/result.md`（最后一份是合进去的那一版）。
- 第 2 样的补丁：`~/.local/share/singlefs/work/mutation-rows-after-judging-keys/patch/mutations-replacements.tsv`（2102 行，十二段，后六段是打之前主工作区那一行的前像）与同目录 `baseline/crates/mutations.tsv`；实现员报告 `~/.local/share/singlefs/work/mutation-rows-after-judging-keys/result.md`（报告写 2103 行，主 agent 拿掉了 E202 V89 那一行，原样在 `~/.local/share/singlefs/work/mutation-rows-after-judging-keys/mutations-replacements.with-7911.tsv` 第 1971 行）。删掉的那一行是「判器键表重算：依赖包并上来的特性不认（重算改回只照本包特性，与 entry_group_key 相同）」，删前的整表在 `~/.local/share/singlefs/work/mutation-rows-after-judging-keys/mutations.before-deleting-4925.tsv`，删的依据在 `records/2026-11-20-拆分带出的门禁与变异红.md` 里点名这一行的那一条。
- 起因（主 agent 的观测）：没交卡串的任务里证红一行 GPU 对拍变异，基线就红：「panicked at crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs:953:47」「拿卡池把手时判过这台机器有 NVIDIA 的卡，起池时却判成没有（起池时的「本次未跑」那一行：NOT_RUN_NO_GPU_ADAPTER … reason=调用方没交卡串…）」。拿把手那一步只看硬件，起池那一步先看卡串。
- 方案按 `crates/` 今天的实现来谈；读码引主工作区今天的路径与行号。

## 二、实现今天的样子（主 agent 的观测，2026-11-21）

读过的 `crates/` 路径与看到的事实：

- `crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start.rs` 第 863–873 行：公开入口 `gpu_card_pool_or_report_not_run` 调 `gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line`，交 `GpuConfigurationOfTheCaller::of_this_process` 与 `nvidia_hardware_of_this_machine`，再 `.into_option_printing_the_not_run_line()`。
- 同文件第 878–889 行：接线那一层，读配置与判硬件两步都从闭包参数交进来。
- 同文件第 902–933 行：判法 `gpu_card_pool_of_the_configuration_or_the_not_run_line`。`cards_for_the_gpu_tests_given` 交 `NotHandedOver { reason_not_run }` 时交回 `NotRun`，不调判硬件的闭包；`HandedOverByTheService(_)` 与 `Err(_)` 并成一臂往下判硬件，`Absent` 交回 `NotRun`，否则交把手。
- 同文件 `card_pool_started_now` 在起池交回 `NotRun` 时照旧 panic（这次只改了那句话的措辞）。
- 同文件 `gpu_card_pool_or_report_not_run_given_the_hardware` 不看调用方交没交卡串（文档注释写明），这次没动它的判法。
- `crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs`：新加四条单测，卡串由测试经 `for_the_unit_tests(…)` 自己给。
- 合入后验证量到的：第 1 步点 `singlefs-verification:src/lib.rs` 一项，「验证红 0 项，验证包那一步 过（按条现判 441、复用 647）」，没缩域；第 2 步在快照上 crash-case-check、命名四格、naming-lint 退 0，拒算列表「拒算 10 条、登记表 10 行」，crates-mutation-replay 点名 4 行「计数：抓到 4 条、没红 0 条、无效 0 条、锚点失效 0 行（共 4 条）」「基线不绿 0 行」。
- 带 `--nocapture` 在没交卡串的任务里跑当初出红的那条对拍：退 0，日志里 1 处 `NOT_RUN_NO_GPU_ADAPTER` 点名它，没有 `panicked`。
- 变异表那一样量到的：打之前在副本上预演「✓ 核过了（--dry-run，没改）…变异表合并之后 8616 行」；合入并删那一行之后主工作区 mutation-tables 格退 1，锚点一句只剩 E202 V89 一行、筛选词一句没有了，另有三行「判不了点名的测试够不够得着被改坏的函数」，四行都不在这一包里（别的会话认下在改）。抽 20 行证红：14 行经 `prove-red.sh` 抓到，5 行经 crates-mutation-replay 点名抓到，1 行（点名一条 GPU 对拍用例）记本次未跑。

## 三、要攻的地方

1. 没交卡串时，GPU 对拍用例拿不到把手就返回，不再往下走。验证包里有没有哪条用例或哪段流程，原先靠「没卡串的任务先在 CPU 上录、往判定库写待判行」再由 GPU 服务判，现在因为在拿把手那一步就返回而连录都不录了。逐个看调 `gpu_card_pool_or_report_not_run` 的调用方。
2. `HandedOverByTheService(_) | Err(_)` 并成一臂：交了卡串而卡项读不成时往下判硬件、交把手，起池时会走到哪里（报本次未跑，还是 panic，还是判红），与条款对不对得上。
3. `gpu_card_pool_or_report_not_run_given_the_hardware` 仍不看卡串：没交卡串而调用方给「有」时，后面的借卡与起池各走到哪一支，有没有还会 panic 的路。
4. 公开入口里「读的是这个进程的配置」那一句（`GpuConfigurationOfTheCaller::of_this_process`）没有测试钉住（实现员第三份报告「交主 agent 判的」）：把它换成别的配置，四条单测与四行变异里有没有一条会红。
5. 点名 GPU 对拍用例的变异行在任务里的结局恒是本次未跑：这类行此刻等于没有变异覆盖。这一批替换的 2102 行里这类行有多少、它们守的行为另外有没有不用卡的测试守着。
6. 2102 行整行替换是不是只动了锚点与筛选词：抽查替换文前后，有没有顺手改了「改坏成什么」的意思、或把点名的测试换成了一条抓不到这处改坏的测试。
7. 删掉的那一行原先守的行为，现在由哪一行或哪条测试守着；没有的话是哪一处行为没人守。
8. 新加的四条单测测的是条款还是实现：把判法改成别的同样满足条款的写法，它们会不会无故红；把判法改坏成不满足条款的写法，它们会不会照样绿。

## 四、检查单

### 检查单：走不到的理由二次查实

三份实现员报告的「走不到的理由」一节都写没有：这一批没有新加 `todo!`、`unreachable!`、`assert!`，也没加错误成员。原有的一处 panic 留着：`gpu_test_start.rs` 的 `card_pool_started_now`，交了卡串、拿把手时判过有卡而起池时判成没卡才走到，文档注释写它只剩「硬件前后不一致那一种」。请腿照调用点逐个追，判这句「只剩那一种」成不成立（「要攻的地方」第 2、3 条是它的两个候选反例）。

### 检查单：类型与名字

新加的名字：函数 `gpu_card_pool_of_the_configuration_of_the_caller_or_the_not_run_line`、`gpu_card_pool_of_the_configuration_or_the_not_run_line`，常量 `HOSTS_NOT_READ_WHEN_TAKING_THE_CARD_POOL_HANDLE`；四条单测名在 `gpu_test_start/tests.rs`。没有新加类型与错误成员。`match` 里有一处 `Err(_)`（第 916 行那一臂），是或模式里的一支、不是通配臂；它丢掉的是那句读不成的原因。没有新加 `expect`、`unwrap`。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：没有认旧版本的分支、旧布局号、迁移路径（查过 `merge-patch/crates.patch` 全文与 2102 行替换文的第 3、4 段）。

### 检查单：不新建判定库

本轮改动没有这一类：没有新的判定存放处。没交卡串时用例直接返回，不写判定库。

### 检查单：所有任务多机可用

四条新单测不读本机状态：卡串由测试自己经 `for_the_unit_tests(…)` 给，硬件由闭包给。公开入口照旧读这个进程的环境（卡串是任务服务不交、GPU 服务才有的那一份），两台同一口径。变异表的替换行不带主机名与路径之外的本机状态。

### 检查单：测试优先多线程

四条新单测是纯函数判法，各自毫秒级，没有可切片的循环；没有新加崩溃检测。本轮改动没有该并行而没并行的。

### 检查单：带 --item 的命令与项

合入后验证的 `--list-items` 列出 1 项 `singlefs-verification:src/lib.rs`，照它点名交的。四行新变异在快照上经 `bash .claude/gate.d/crates-mutation-replay.sh --item '<变异名>' ×4 <快照>` 点名判过，四个名字是：「GPU 卡池把手 没交卡串也往下判硬件、交把手（要池里的卡时 panic）」「GPU 卡池把手 交了卡串、有卡也报本次未跑」「GPU 卡池把手 交了卡串、没卡也交把手」「GPU 卡池把手 取卡池的接线不把调用方那一份交给判法、只看硬件（没交卡串也交把手）」。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放的流或用例。

### 检查单：放的位置

新单测放在被测模块自己的 `#[cfg(test)]` 文件 `crates/singlefs-verification/src/gpu_unit_checks/gpu_test_start/tests.rs`，没有新开包、没有跨包引源码、没有新加二进制目标。变异行照旧在 `crates/mutations.tsv`。

## 五、分工

| 腿 | 立场 |
|---|---|
| 云端正推（Sonnet） | 逐条核代码做的是不是题面两句条款说的；九格逐格答成立 / 不成立 / 推不出；「要攻的地方」第 1、2、3 条照调用点逐个追到底 |
| 云端攻方（Opus） | 假设这一批是错的：造一条没交卡串时本该录而没录、本该报本次未跑而 panic 或静默算过的路；「要攻的地方」第 4 到 8 条，逐行抽查替换的变异行（至少 40 行，按包、按改了哪几段分层抽），找改了意思的、点名测试抓不到的 |

## 六、交付要求

- 引规则与记录写文件名加那份文件自己的行号，行号去那份文件里现查。
- 引代码写主工作区路径与行号，读的时候核一下那一行还在。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。

### 小节清单：`.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md`（kb-sections.py 全量生成，未经任何过滤）

| 小节 | 抄 / 不抄 | 理由 |
|---|---|---|
| ## E202 GPU与CPU两种模式GC对照组 —— 跑前登记已写、装置未跑 | 抄 | 顶上标题，与开篇正文合成行区间 |
| （E202 标题之下、第一个下级标题之前的正文：第 2-8 行） | 抄 | 开篇正文，与顶上标题合成行区间 |
| ### 现状 | 抄 | 默认全抄（脚本生成） |
| ### 复跑 | 抄 | 默认全抄（脚本生成） |
| ### 口径 | 抄 | 默认全抄（脚本生成） |
| ### 判据与失败条款 | 抄 | 默认全抄（脚本生成） |
| ### 影响的决策 | 抄 | 默认全抄（脚本生成） |
| ## 历史版本 | 不抄 | 历史节，正文只引现状 |
| ### 2026-11-13 | 不抄 | 历史节的子节 |
| ### 2026-11-03 | 不抄 | 历史节的子节 |

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:1-8`（整段抄，未转述）**

```markdown
## E202 GPU与CPU两种模式GC对照组 —— 跑前登记已写、装置未跑

**中文** · [English](202-gpu-vs-cpu-gc-comparison.en.md) · [日本語](202-GPUとCPUの2方式GC対照群.ja.md)

跑前登记 `research/prompts/e202-preregistration.md`。回答问题单 `research/prompts/m4-gpugc-compare-questions.md`：比较同一负载、同一池下 `background_compute=cpu` 与 `background_compute=gpu` 两种模式的 GC 挂钟、占用的主机 CPU 时间与省下的时间、前台 fsync 延迟分布，以及省下的 CPU 时间随盘容量的变化。

装置 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`（入库装置，住在产品 GPU 的测试包 background-gpu-tests 里）。**这一页只记跑前登记与装置骨架已写，产物一次没跑。**

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:9-16`（整段抄，未转述）**

```markdown
### 现状

- 跑前登记已写（2026-11-03），装置骨架已立，产物一次没跑。
- 第一段的几何核起过一次、没跑完、没留产物：造起步池太慢，照量到的速度过不了任务服务的四小时上限。量到的：2288MiB 档顺序写那一半约 34 分钟，交错写那一半约 60 秒做 95 步、共 18304 步；8GiB 档 2100 秒做到顺序写 18292 步、共 65536 步；三档同时造，常驻内存 2100 秒时 9.4 GiB。用户定这一次不跑，等造池提速做完再跑。
- 盘容量采用定占用读法（容量涨，活数据占单元区比例固定 50%）。
- 装置约束：GPU 模式每张卡上限 1 GiB、按卡的顺序填满；一张卡与多张卡各跑；判定逐条与 CPU 参照相同才计一组数。
- 多张卡那一臂用几张卡由 `lockstep --cards <张数>` 定（给 0 或 1、或多于服务交来的张数退 2；不给就用服务交来的全部卡）。产物里有两种记卡的行：`name=every_card_arm_cards` 记 `cards_named`、它是怎么定的（`named_by=cards_option` 或 `every_handed_over_card`）与每张卡的 PCI 地址与型号；`name=one_card_arm_card` 记点名一张卡那条臂那一张卡的 PCI 地址与型号，交替跑与并发跑都打。续跑时这几样与要续的产物里已有的同种行有一处不同，或产物里有这条臂跑出的结果行（`name=run`、`name=concurrent_run`）而没有这种行，打 `name=stopped clause=cards_differ_from_the_product_being_resumed` 退 2；一条结果行都没跑出来的那一趟不挡以后的续跑。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:17-23`（整段抄，未转述）**

````markdown
### 复跑

```bash
# 入库装置编译与运行归主 agent；经 task-server 调度
task-server/task submit -- cargo run --release -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --bin e202_gpu_cpu_gc_comparison
```

````

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:24-32`（整段抄，未转述）**

```markdown
### 口径

- 原始输出未留存：实验未跑。
- 一次整理：一次跑里 GC 线程从第一次调后台入口到最后一次调用返回为止做的全部整理。
- GC 挂钟：GC 线程上每一次后台入口调用前后读 `std::time::Instant` 之差相加。
- GC 的主机 CPU 时间：每次 GC 调用前后读 `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` 之差减去同一窗口前台线程增量。
- 省下的 CPU 时间：同一档同一次重复里 CPU 臂的进程口径减 GPU 臂的进程口径。
- 重复次数：每档每臂 5 次，报中位、最小、最大与中位绝对偏差。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:33-37`（整段抄，未转述）**

```markdown
### 判据与失败条款

- 判据：每一档容量上「CPU 模式减 GPU 模式」的差有数、有正负、有离散度；阳性对照下省下的 CPU 时间必须少于正常对照。
- 失败条款：若 GPU 模式省下的主机 CPU 时间在各容量档位均恒 ≤ 0，或前台 fsync 延迟劣化超限，则触发失败条款。

```

**出处 `.claude/kb/experiments/202-GPU与CPU两种模式GC对照组.md:38-43`（整段抄，未转述）**

```markdown
### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-18 不受影响：这一次只把多张卡那一条的说法照登记修订第 12 条回写（两种记卡的行、续跑核卡的停机分句名），判据、臂与口径没改，产物仍一次没跑。2026-11-14 那一次改的是装置搬进测试包与起法、多张卡那一臂的张数定法，判据、臂与口径没改，产物一次没跑，仍是跑前登记已写、装置未跑 |

```
