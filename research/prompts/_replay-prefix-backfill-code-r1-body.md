# 重放前缀包在途改动的代码轮（补轮）

日期：2026-11-16。

## 范围

被判的是 `crates/singlefs-replay-prefix/` 下十四份文件工作区相对 HEAD 的改动：`src/` 的八份 `.rs`（`compaction_plan_proofs.rs`、`file_range_proofs.rs`、`gpu_gc_capacity_proofs.rs`、`gpu_gc_proofs.rs`、`gpu_gc_scoring_proofs.rs`、`lib.rs`、`partial_write_proofs.rs`、`unlink_proofs.rs`）与三份说明文件（`GPU_GC_PROOFS.md`、`GPU_GC_SELFTEST.md`、`README.md`），`tests/` 的两份新测试，必红表 `verus-must-red-worlds.tsv`。材料都在 `research/prompts/replay-prefix-backfill-code-r1-snapshot/`：

- `changes.patch`：`git diff HEAD -- <这十四份>` 的原样输出。
- `crates-sha256.txt`：十四份文件开工时的 sha256。
- `facts.md`：事务员出的逐文件事实表，是线索。

`crates/mutations.tsv` 里指到这十四份、相对 HEAD 新出现的行是 0 行（事务员按文件、原文、替换文三段为键比的）：三份新证明文件的反例都在必红表里（行名前缀 gpu-gc 35 行、gpu-score 15 行、overwrite-contract 5 行）。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测）

读过 `git diff HEAD --numstat -- crates/singlefs-replay-prefix` 的输出与事务员的概述；下面各件腿自己读源文件核：

1. **三份新证明文件**（整份新增）：`crates/singlefs-replay-prefix/src/gpu_gc_proofs.rs`（385 行，头注释说是「GPU GC 的主机端纯核与着色器算术模型」，仲裁、预算、分片、回读与区域准入的 exec 函数由产品直接调用）、`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs`（218 行，切批准入、最大分片、显存账本和派发尺寸）、`crates/singlefs-replay-prefix/src/gpu_gc_scoring_proofs.rs`（271 行，两份活槽打分算法的等价证明）。三份头注释都写了边界：不证 WGSL 编译器、GPU 执行、设备 I/O、主机排序与打包。`.claude/gate.d/verus-pins.tsv` 里这一包登记的义务数由 155 改成 208，并加了三行指到这三份的行（登记表不在这一轮被判的文件里）。
2. **固定长度覆盖写的字节规律**（`crates/singlefs-replay-prefix/src/partial_write_proofs.rs` 新增约 115 行：`overwrite_fits`、`overwritten_bytes` 与四条引理、一个见证）与形参改名（`ws`、`we`、`us`、`ue` 改成全名，`requires` 里跟着换）。
3. **两份新测试**：`crates/singlefs-replay-prefix/tests/gpu_gc_arithmetic_matches_wide_integer_reference.rs`（4 条，拿 `u128` 参照在九个边界值上对拍）、`crates/singlefs-replay-prefix/tests/gpu_gc_capacity_matches_independent_reference.rs`（5 条，独立全宽算术与穷举对拍）。
4. **必红表**加 68 行反例、改 8 行锚点（随改名）。
5. **纯改名**：`lib.rs` 的 spec 函数 `a_later_record_of_the_same_publish_is_readable` 去掉 `a_`（定义与 6 处引用，`ensures` 里的被调名跟着变）、两个证明函数与四个测试去 `a_`，加三行模块声明；`compaction_plan_proofs.rs`、`file_range_proofs.rs`、`unlink_proofs.rs` 的函数名与局部变量改名。
6. **三份说明文件**：`GPU_GC_PROOFS.md`、`GPU_GC_SELFTEST.md` 新增，`README.md` 加一节与一行。

## 要两条腿答的

- 正推：三份新证明文件里每条 `ensures` 说的性质，与它头注释、`GPU_GC_PROOFS.md` 列的 G01 到 G30 逐条对得上对不上；产品「直接调用同一份 exec 核」这句在 `crates/singlefs-background-gpu/src/` 里对不对（grep 调用点，逐个列）；覆盖写四条引理的 `requires` 是不是产品写路径真满足的前提；改名那一批是不是纯改名、`requires`、`ensures`、`decreases` 的结构动没动。
- 攻方：哪一格会错。至少攻：证明是不是空真（`requires` 互相矛盾、见证函数罩不罩住每个 `requires`）；`ensures` 太弱（性质名说的比 `ensures` 证的多）；exec 函数里 `as` 转换（`gpu_gc_capacity_proofs.rs:151-152`、`:187`，`gpu_gc_proofs.rs:333`）丢不丢位、`ensures` 管没管住；模型与 WGSL、与产品调用点之间的缺口有没有被说明文件说成证过了；两份新测试的参照与被测是不是共用了算式（对拍两边同错）、九个边界值够不够；必红表新加的每一类反例是不是真红在它点名的那条性质上、哪条 `ensures` 没有反例罩着；三份说明文件里与代码对不上的数（事务员的线索：`README.md` 写「八份源文件」「另七份」而 `src/` 下 `.rs` 是 10 份；`GPU_GC_SELFTEST.md` 标题写「20 项」而别处写到 G30）。
- 两条腿都逐格判下面的检查单，回「成立」「不成立」或「推不出」。

### 检查单：走不到的理由二次查实

这批改动没有实现员报告（补轮）。事实表查各份 `.rs` 的加行，生产段没有新加 `unreachable!`、`panic!`、`expect`、`unwrap`；Verus 文件里的 `assert` 是待证命题，由验证器判。exec 函数返回 `Option` 的那几处（`checked_` 起头的与 `decoded_gpu_live_slots`）`None` 那一支各自在什么输入上出现、调用方怎么接，腿逐个追到 `crates/singlefs-background-gpu/src/` 的调用点。

### 检查单：类型与名字

新加的名字列在 `facts.md` 各文件「类型与名字」一行（`gpu_gc_proofs.rs` 14 个、`gpu_gc_capacity_proofs.rs` 12 个、两份测试 12 个）。`as` 转换生产段四处：`gpu_gc_capacity_proofs.rs:151`、`:152`、`:187`，`gpu_gc_proofs.rs:333`。exec 函数的参数多是裸 `u32`、`u64`（字节数、条数、槽号、显存上限），腿判同一个函数里不同语义的量有没有共用裸整数而该用 newtype 的。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：事实表按关键词查加行零命中。

### 检查单：不新建判定库

`.rs` 与必红表的加行没有新的判定存放处。`GPU_GC_SELFTEST.md` 记了「自测用临时判定库的跑法」，腿读它判那是不是 `.claude/rules/verification.md`「判定库只有一份」许的「自己起的临时服务，格完即删」。

### 检查单：所有任务多机可用

本轮改动没有这一类：事实表按关键词（hostname、`/proc/`、`/sys/`、`env::var`）查加行零命中；说明文件里的复跑命令腿核有没有写死主机名、地址与本机路径。

### 检查单：测试优先多线程

两份新测试是边界值与小范围穷举的对拍，没有重活；没有写死或覆盖线程数的地方。

### 检查单：带 --item 的命令与项

新证明项、空真项与必红项由门禁 formal-verification 按条点名；说明文件里写的复跑命令带没带范围声明、项名在 `--list-items` 里列不列得出，腿对着 `.claude/gate.d/formal-verification.sh` 文件头与 `scripts/gate/verus-proof-check.py` 文件头「逐项点名与按条复用」核写法（列项要跑的写进报告交主 agent）。

### 检查单：新崩溃流多罩了什么

本轮改动没有新加崩溃点重放流或崩溃枚举用例。
