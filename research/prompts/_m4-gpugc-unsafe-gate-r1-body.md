# m4-gpugc-unsafe-gate-r1：unsafe-allowances 那一格的判法与登记行（2026-11-05）

## 一、题面

门禁 code-source-discipline 的 unsafe-allowances 格（`.claude/gate.d/code-source-discipline.sh`）与它读的登记表 `.claude/gate.d/unsafe-allowances.tsv`，这一批两路一起改过，都没走过三方。表头最后一行写「增一处放开：改这张表的行并走三方」，这一轮用户点了名（里程碑四 GPU 垃圾回收那一路：用户 2026-11-03 弹窗定「改门禁，一处一行」）。两路各管的部分：

| 部分 | 谁改的 | 出处 |
|---|---|---|
| 判法：一处一行（一个根的放开处数等于登记行数，每个登记的项上恰好一处 `#[allow(unsafe_code)]`，项里 unsafe 处数等于第 5 段，每处 `// SAFETY:` 提到第 6 段的词）；SAFETY 按所在语句的开头一行往上认（续行结尾 `=`、`(`、`[`、运算符） | GPU 垃圾回收那一路（第 70、71 批工具员） | `records/2026-10-19-里程碑四GPU垃圾回收开工.md`（第 70、71 批两条）；工具员报告 `~/.local/share/singlefs/work/m4-gpugc-unsafe-rows-tooling/result.md`、`~/.local/share/singlefs/work/m4-gpugc-unsafe-rows-tooling-2/result.md` |
| 判法：bin 根逐个判（每个 bin 根有自己的级别，不跟库根）；`unsafe fn` 签名与 `unsafe impl` 项头算进项里、能登；impl 块里的方法按 `<类型>::<方法>` 认；几行都罩住同一处 unsafe 时归范围最小的那一行；表头 bin 根写法 | 里程碑三收尾那一路（批 19） | `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2409 行（用户定 Q5「丙：E197 改不用、E183 登」，同行写「门禁那一格要改成认 bin 根」）；工具员报告 `~/.local/share/singlefs/work/m3-close-b19-unsafe-bin-roots/result.md`；这一半判法本身没单独走过三方 |
| 登记：产品 GPU 包 5 行（`.claude/gate.d/unsafe-allowances.tsv` 第 33–37 行） | GPU 垃圾回收那一路 | 同上记录；第 35 行 `monotonic_raw_time_now` 是用户 2026-11-05 定 E202 挂钟换法「A」之后新加的（登记 `research/prompts/e202-preregistration.md` 第十二节修订第 10 条） |
| 登记：E185 装置 bin 根 8 行（第 41–48 行） | GPU 垃圾回收那一路 | 同上记录（第 73 批） |
| 登记：E183 装置 bin 根 5 行（第 53–57 行） | 里程碑三收尾那一路 | 取舍过了三方：`research/prompts/m3-close-b19-reason-r3-main-verification.md` 第 39 行（Q5）、第 45、47 行（甲、丙两条候选），用户选丙 |
| 登记：核心层 2 行（第 31、32 行） | 与 HEAD 比现查（`git diff HEAD -- .claude/gate.d/unsafe-allowances.tsv`） | 腿逐行核这两行是这一批新加还是 HEAD 里就有 |

要判的是：判法会不会放过一处没登记、或 SAFETY 没写前提的 unsafe，会不会拒掉合规的写法；每一行登记是不是真的非放开不可；每处 SAFETY 写的前提在代码里成不成立。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

- 判法住在 `.claude/gate.d/code-source-discipline.sh` 内嵌的 python 里：`bin_roots_of`（第 2691 行起）认 bin 根；`items_of`（第 2754 行）、`item_span`（第 2787 行）切项；`safety_comment`（第 2809 行）往上找 SAFETY；`registered_row_holding`（第 2835 行）把一处 unsafe 归到登记行；`judge_root`（第 2933 行）逐个根判。格名表在文件头第 25 行，弄坏开关在文件头第 131 行起（`GATE_UNSAFE_ALLOWANCES_BREAK=…`）。这一份相对 HEAD 改了 947 行加、261 行删（两路合计，`git diff HEAD --stat`）。
- 样本在 `.claude/gate.d/fixtures/code-source-discipline.sh/` 下以 `unsafe-allowances-` 起头的那几份。读数：任务 5954（不开开关的样本档）判错 1 份（`unsafe-allowances-same-function-two-rows-red`），后来批 19 那一路说是判法拒绝句函数名后少一个空格、已改判法；17 个 `GATE_UNSAFE_ALLOWANCES_BREAK` 开关与 `STAGE_CELLS_BREAK` 两条，减掉那 1 份基线之后判错集合都与文件头点名的相同（任务 5955–5973）。
- 真仓读数：任务 6915（这一格在主工作区）退 0。任务 6235 时这一格只红 E183 的 5 行，那 5 行之后由批 19 那一路改了源码。
- `crates/` 里放开的地方（行号是主工作区此刻的）：
  - 产品 GPU 包：
    - `crates/singlefs-background-gpu/src/background_gpu.rs`：`host_cpu_time_of_this_thread`（第 2407–2429 行）与 `monotonic_raw_time_now`（紧挨着它）各一处 `clock_gettime`；`largest_device_local_memory_heap`、`with_the_address_space_capped_beyond_the_present_size` 各在同一份文件里；
    - 测试 `crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs` 里的 `free_device_local_bytes_of_the_card` 读 `VK_EXT_memory_budget`；
    - 根级别在 `crates/singlefs-background-gpu/src/lib.rs`（`#![deny(unsafe_code)]`，文档第 18–20 行写五处）。
  - E185：`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs` 顶上 `#![deny(unsafe_code)]`，8 项 9 处：`clock_gettime` 两项、`posix_fadvise`、`sched_getcpu`、`sysconf`、硬件 CRC 指令一项两处、`sched_getaffinity`、`sched_setaffinity`；绑核两项靠一条编译期大小与对齐断言把 16 个 u64 当 `cpu_set_t` 传（同文件「`const _: () = assert!(`」那一段）。
  - E183：`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs` 的计数分配器 `unsafe impl GlobalAlloc for CountingAllocator` 与它的四个方法。`research/prompts/m3-close-b19-reason-r3-main-verification.md` 第 39 行判过：原 `// SAFETY:` 写「计数是原子的」，加了线程本地计数之后只对一半；批 19 已改写那几处 SAFETY（同文件第 213–252 行一带，里程碑三收尾那一路现查）。Q3 核前提时专门看线程本地计数那一半。
- E185 那一行实现员自己报的（`~/.local/share/singlefs/work/m4-gpugc-e185-unsafe-rows-writer/result.md`）：`cpu_number_now`、`clock_ticks_per_second`、`allowed_cores_of_this_thread` 三项有不用 unsafe 的替代（读 `/proc/thread-self/stat`、写死 100、读 `/proc/thread-self/status` 的 `Cpus_allowed_list`），理由栏照实写了替代是什么、为什么没用。

## 三、格

### Q1　判法：放过与误拒

要判的是：照今天的判法，有没有一种写法让一处 unsafe 不在任何登记行的项里、或 SAFETY 没写登记的词，而这一格照样绿；反过来，有没有合规的写法被它拒掉。分几问：

1. 一处一行与「几行都罩住的一处归范围最小的那一行」：嵌套项（impl 块里的方法、函数里的闭包与内部 fn、宏展开）里的 unsafe 归哪一行；同名方法在两个 impl 里、trait 默认方法、`impl <Trait> for <类型>` 与固有 impl 并存时，键撞不撞。
2. SAFETY 按语句开头认：续行的认法（结尾 `=`、`(`、`[`、运算符）放不放过「两条语句之间隔着别的语句、SAFETY 其实写给上一条」那一形（样本 `unsafe-allowances-statement-finished-between-red` 罩的是哪一形、还有没有它没罩的）；逗号结尾不算续行（第 71 批工具员报告写明没收这一种），`match` 分支里的 unsafe 会不会认错。
3. bin 根：`src/bin/<名>.rs` 与 `src/bin/<名>/main.rs` 两种写法、`[[bin]]` 在 `Cargo.toml` 里另指路径的、`build.rs`、`examples/`、`benches/`，各归哪个根；表里没登记的 bin 根照 forbid 判，放不放过不在这几处的 unsafe。
4. 宏与条件编译：`cfg` 关掉的分支里的 unsafe、宏里展开出的 unsafe、字符串与注释里的「unsafe」字样，判法认不认、该不该认。

要交的：每一问一个结论（放得过 / 放不过 / 推的），放得过的给最小的一份源码样本（能直接做成 red 样本），放不过的给判法里挡住它的那一行。

### Q2　登记：每一行是不是非放开不可

对 `.claude/gate.d/unsafe-allowances.tsv` 里放开了的每一行（第 31–37、41–48、53–57 行），判第 7 段理由站不站得住：std、`libc` 以外已有依赖（`Cargo.lock` 现查）有没有安全的写法；有替代而没用的，替代的代价是什么（准不准、快不快、两台机器上在不在）。E185 那三项（`cpu_number_now`、`clock_ticks_per_second`、`allowed_cores_of_this_thread`）尤其要判：理由里写的「为什么没用替代」成不成立。E183 五行的取舍已过三方（第一节出处），这一轮只判登记行与源码对不对得上、理由与 SAFETY 写的是不是那一轮判的那件事，不重开甲、丙之争。

要交的：每一行一个结论（站得住 / 站不住 / 推的），站不住的写出替代的写法与它在这个项目里的代价。

### Q3　SAFETY 写的前提成不成立

逐处读 SAFETY 注释，对着代码核它写的前提：栈帧里活着、初始化过、调用期间没有别的引用、长度照实传、内核至多写这么多、指令集在造出这个类型时已检出、`cpu_set_t` 的大小与对齐靠编译期断言等。有没有一处前提写了而代码里不成立，或成立却靠一条注释没写出的前提（例：绑核两处在补 SAFETY 之前就没提那条断言）。

要交的：每一处一个结论，不成立或缺前提的给文件:行与补法。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | Q1–Q3 | 从判法源码与 `crates/` 今天的实现推到结论：Q1 逐问给挡住那一形的判法行号或放得过的样本；Q2 逐行判理由；Q3 逐处核前提 |
| 攻方腿（Opus） | Q1–Q3 | 找反例：Q1 造让这一格照样绿的坏源码（嵌套项、同名方法、续行、bin 根、宏与 cfg），Q2 找理由站不住的行与现成的安全替代，Q3 找前提不成立的 SAFETY |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码与门禁写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要造样本核判法的，样本放 `research/prompts/m4-gpugc-unsafe-gate-r1-<模型>-model/`，草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。
- 腿不交任务：要跑的判法与样本（喂这一格的命令）写成 `task-server/task submit -- …` 的整条，写进报告「要主 agent 跑的命令」一节。
- 报告写进 `research/prompts/m4-gpugc-unsafe-gate-r1-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
