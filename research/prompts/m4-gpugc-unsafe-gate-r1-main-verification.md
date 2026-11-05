# m4-gpugc-unsafe-gate-r1 判决：unsafe-allowances 那一格的判法与登记行（2026-11-05）

被判的文件：`.claude/gate.d/code-source-discipline.sh`（unsafe-allowances 格）、`.claude/gate.d/unsafe-allowances.tsv`，以及登记行指到的 `crates/singlefs-background-gpu/src/background_gpu.rs`、`crates/singlefs-background-gpu/src/lib.rs`、`crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs`、`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs`、`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs`。正文 `research/prompts/_m4-gpugc-unsafe-gate-r1-body.md`，背景 `research/prompts/_m4-gpugc-unsafe-gate-r1-background.md`。

## 一、腿与核查

| 腿 | 报告 | 结局 |
|---|---|---|
| 正推（Sonnet） | `research/prompts/m4-gpugc-unsafe-gate-r1-sonnet-output.md`（sha256 `903388e1…`），模型 `research/prompts/m4-gpugc-unsafe-gate-r1-sonnet-model/` | Q1–Q3 都有逐项结论 |
| 攻方第一条（Opus） | 没有报告 | 读判法源码之后一条回复被安全分类器截停（会话记录第 78 条 `stop_reason=refusal`），没有结论，不算「没打中」 |
| 攻方重派（Opus） | `research/prompts/m4-gpugc-unsafe-gate-r1-opus-output.md` | 同样被截停（第 99 条），Q1–Q3 都「未判」 |
| 攻方第三条（Opus，只做 Q2、Q3） | `research/prompts/m4-gpugc-unsafe-gate-r1-opus-3-output.md`（sha256 `cd61f788…`） | Q2、Q3 有逐项结论，没被截停 |

截停的原因（主 agent 对照三条攻方腿与正推腿的会话记录）：读同一批源码的正推腿 117 次工具调用 0 次截停，只做 Q2、Q3 的第三条攻方腿也没被截停；前两条都停在做 Q1「找门禁放过的写法」的时候。被截停的输出没有留下。所以 Q1 这一轮只有正推一方，没有攻方。

核查（照 `.claude/rules/three-way-inference.md`「核查与判决由主 agent 做」）：
1. 开工快照倒推：任务 7270，`python3 scripts/agents/snapshot-tree.py m4-gpugc-unsafe-gate-r1 --out ~/.local/share/singlefs/work/m4-gpugc-unsafe-gate-r1-main-check/m4-gpugc-unsafe-gate-r1-tree --repository /home/fy5090/code/singlefs`。被判的几份逐份与 `research/prompts/m4-gpugc-unsafe-gate-r1-snapshot/judged-sha256.txt` 同 sha256。
2. 引文：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-unsafe-gate-r1-sonnet-output.md research/prompts/m4-gpugc-unsafe-gate-r1-opus-3-output.md --root <上面那棵树> --background research/prompts/_m4-gpugc-unsafe-gate-r1-background.md`，末行「✓ 核了 50 处引文、47 处裸行号，对上 43 处，没判 54 处」，没有一处对不上。判决下面要引的行，主 agent 在那棵树里逐处人工核过：
   - `background_gpu.rs:4282`–`:4286`：`rlim_cur` 取「present_size_in_kibibytes * 1024 + room_beyond_the_present_size」，没与 `rlim_max` 取小；
   - `e185…rs:3715`–`:3717`：`File::open` 与 SAFETY 之间隔一行 `sync_all`；
   - `e185…rs:4419`–`:4434`：同一次快照读了 `/proc/thread-self/stat`，再调 `cpu_number_now()`。
3. 复跑：正推腿模型目录拷进 `~/.local/share/singlefs/work/m4-gpugc-unsafe-gate-r1-main-check/sonnet-model/`，任务 7281 跑它的 18 个样本，末行「name=samples_summary total=18 prediction_holds=18 prediction_fails=0 leftover_process_groups=0」。判决里样本的结局都引这一趟（量的）。

## 二、判决

### Q1　判法

量到判错的样本（任务 7281，`samples-verdicts.txt`）：

| 方向 | 样本 | 实测 | 判 |
|---|---|---|---|
| 放过 | `line-separator-shifts-the-safety-lookup-green` | green（同文去掉 U+2028 的 `same-text-without-the-separator-red` 是 red） | 缺陷：`splitlines()` 与按 `\n` 切的行号对不上，SAFETY 往上找错位 |
| 放过 | `safety-line-inside-a-string-green` | green | 缺陷：往上找注释读原文行，不看是不是真注释 |
| 放过 | `raw-c-string-hides-unsafe-green` | green | 缺陷：抹字符串不认 `cr"…"`，失步吞掉后面的 unsafe |
| 放过 | `include-of-a-non-rs-file-green` | green | 缺陷：只读 `.rs`，`include!` 进来的别的扩展名不扫 |
| 放过 | `no-mangle-without-the-keyword-green` | green | 缺陷：2021 版不带 `unsafe` 的 `#[no_mangle]` 不算放开 |
| 误拒 | `bin-helper-file-goes-to-the-library-root-red` | red | 缺陷：`src/bin/<名>.rs` 旁 `<名>/` 下的文件归了库根 |
| 误拒 | `unsafe-below-the-first-line-of-a-method-chain-red` | red | 缺陷：方法链里不在第一行的 unsafe 找不到它的 SAFETY |
| 误拒 | `same-method-name-in-two-inherent-impls-red` | red | 缺陷：两个固有 impl 同名方法撞键，第二个登不进 |
| 按设计 | `macro-expands-three-unsafe-for-one-keyword-green`、`one-comment-covers-the-second-unsafe-of-a-chain-green` | green | 照判法设计如此；登记表头写明 |

其余样本（校准两份、`cfg` 关掉的照算、字符串与注释里的字样不认、`match` 分支、内部 fn 与闭包记在外层）判得对。

结论：判法有 5 处放过、3 处误拒，都量到了。改法交下一批工具员：每一处改判法，并把对应样本收成 `.claude/gate.d/fixtures/code-source-discipline.sh/` 下的红绿样本，配弄坏开关。`[[bin]] path` 指到 crate 外的那一形，正推腿报的是推的（仓里没有实例），只记进下一批的样本清单，不单独判。

这一格这一轮没有攻方，只有正推一方。改法做完之后开第二轮，攻方改用「给定一批样本，判门禁判得对不对」的问法：样本由主 agent 造好交腿，腿不产出绕过写法。

### Q2　登记

| 行 | 判 | 依据 |
|---|---|---|
| 31、32、33、34、35、37、41、46、53–57 | 站得住 | 两腿都判站得住；`Cargo.lock` 146 个包里没有 rustix、nix 这类安全封装（攻方量的）；E183 五行的处数、必提词与源码对上，不重开甲、丙 |
| 44 `cpu_number_now` | 站不住，改 | 两腿都打中。同一次快照已读 `/proc/thread-self/stat`（`e185…rs:4419`–`:4434`，主 agent 核过），第 39 栏就是当前核号；改读它，删这一处 unsafe 与这一行登记 |
| 45 `clock_ticks_per_second` | 站不住，改 | 两腿都打中：`/proc/self/auxv` 的 `AT_CLKTCK` 不用 unsafe（攻方在本机读到 100，量的）；改读它，删这一处 unsafe 与这一行 |
| 47 `allowed_cores_of_this_thread` | 站不住，改 | 两腿都打中：`/proc/thread-self/status` 的 `Cpus_allowed_list` 给的就是这个集合；「要另解析区间列表」是写法偏好。改读它（与 `/sys/devices/system/cpu/online` 的关系照正推腿的写法求交），删这一处 unsafe 与这一行 |
| 36、42、43、48 | 照留，理由栏补写 | 只有攻方打中，替代是起外部命令（`prlimit`、`dd iflag=nocache`、`taskset`）或给 checker 档包加产品 GPU 包这条非可选依赖（wgpu 进它的编译闭包）。起外部命令会进计时或测试路径，新依赖破坏 checker 档的依赖边界，代价都比一处登记大（推的）。理由栏要把这些替代与不用的原因写进去；第二轮复核 |
| 33–37 第 8 段出处 | 改 | 两腿都打中：「用户 2026-11-03 弹窗定」那一次定的是门禁写法（记录里「改门禁，一处一行」那一条），不是逐处定案；第 35 行写的日期在指向的记录里查不到。出处改成指到真有的那一行（第 35 行指 `research/prompts/e202-preregistration.md` 第十二节修订第 10 条），其余指到这一份判决 |

### Q3　SAFETY

| 处 | 判 | 改法 |
|---|---|---|
| `background_gpu.rs:4286` | 前提写了、代码不保证 | 两腿都打中，主 agent 核过：`capped_limit.rlim_cur` 不与 `rlim_max` 取小。不是内存安全问题（硬上限偏小时 setrlimit 报 EINVAL、断言红）。改代码取小，SAFETY 那半句照留 |
| `e185…rs:3717` | 措辞不准 | 「上一行打开」改成「这个函数里打开」 |
| E183 `:217` | 成立，靠两条没写出的前提 | 攻方报的：平台用原生线程本地存储、计数路径不 unwind（第 142 行不饱和加法在开溢出检查的 release 档会 panic）。E183 归里程碑三收尾那一路，转它改 |
| `background_gpu.rs:864`、E183 `unsafe impl` | 缺半句，前提成立 | 正推腿报的；改时一并补 |
| 其余（绑核两处 `cpu_set_t` 强转等） | 成立 | 两腿都判成立 |

## 三、去向

| 事 | 去向 |
|---|---|
| Q1 八处判法缺陷与 18 个样本收成红绿样本 | 下一批派工具员（GPU 垃圾回收那一路与里程碑三收尾那一路共改这一格，派之前告诉 d2） |
| Q2 第 44、45、47 行改装置、删登记 | 下一批派实现员改 E185 装置（checker 档的实验装置，测试代码，不开代码轮） |
| Q2 第 36、42、43、48 行理由栏、第 33–37 行出处栏 | 主 agent 改登记表 |
| Q3 `background_gpu.rs:4286` 取小、`:864` 补半句、E185 `:3717` 措辞 | 并进下一批实现员 |
| Q3 E183 两条前提与 `unsafe impl` 半句 | 转 d2（里程碑三收尾那一路） |
| Q1 攻方缺席、Q2 单方打中的四行 | 第二轮：攻改法本身与这四行的理由 |

## 回看决策

不涉及决策：这一轮判的是一道门禁的判法与 `unsafe` 登记行，不推翻也不确立 `.claude/kb/decisions/` 下任何一条分项。
