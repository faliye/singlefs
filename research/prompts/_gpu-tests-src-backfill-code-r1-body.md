# 产品 GPU 包的测试包 src 四份在途改动的代码轮（补轮）

日期：2026-11-16。

## 范围

被判的是 `crates/singlefs-background-gpu-tests/src/` 下四份文件，四份在工作区都是暂存的新增：

- `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs`（13207 行）：E202（GPU与CPU两种模式GC对照组） 的装置。它是从 `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs` 搬来的，HEAD 里旧路径那一份 11570 行。这一轮判的是**搬迁之外的改动**（旧路径的 HEAD 版本对现版本的 diff，2345 行、128 块，删 245 行、加 1882 行），加上搬迁本身带来的东西（包名、登记表、路径引用）。
- `crates/singlefs-background-gpu-tests/src/gpu_side_units_entries.rs`（168 行）、`crates/singlefs-background-gpu-tests/src/reference_units_entries.rs`（73 行）：两张入口登记表，全新。
- `crates/singlefs-background-gpu-tests/src/lib.rs`（4 行），全新。

材料都在 `research/prompts/gpu-tests-src-backfill-code-r1-snapshot/`：

- `e202-move-against-head-old-path.diff`：`git show HEAD:<旧路径>` 对现版本的 `diff`（行号左旧右新）。
- `changes.patch`：`git diff HEAD -- <这四份>`（四份都是新增，等于全文）。
- `crates-sha256.txt`：四份开工时的 sha256。
- `facts.md`：事务员出的事实表（整份读过四份文件；搬迁之外改了什么在它的第〇·一节，e202 的职责分段与行区间在第一节第 1 样）。它是线索。
- `e202-expect-src.tsv`：e202 产品代码段（1 到 9238 行）`expect`、`panic!`、`assert` 的逐处表。
- `mutation-rows.tsv`：`crates/mutations.tsv` 里文件那一列指到这四份的 123 行（e202 121 行、`gpu_side_units_entries.rs` 2 行）。`crates/mutations.tsv` 整份另开一题判。

跑前登记是 `research/prompts/e202-preregistration.md`，实验页在 `.claude/kb/experiments/` 下 E202 那一份（文件名在快照的 `kb-sha256.txt`）。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测）

主 agent 现查了：`git diff HEAD --stat -- crates/singlefs-background-gpu-tests/src` 是「4 files changed, 13452 insertions(+)」；`crates/mutations.tsv` 里指到这四份的行数（123）。下面搬迁之外的改动照事务员事实表第〇·一节列，腿自己对着 `e202-move-against-head-old-path.diff` 读：

1. **取卡只用服务交的卡**：`main` 里不再现建卡表，换成读 `GPU_CARDS_BY_PRIORITY` 或领卡通道，造完起步镜像之后才取（`cards_handed_to_the_apparatus`、`cards_taken_for_this_process`、静态 `CARDS_SEEN_BY_THIS_PROCESS`）；交来的卡对不上本机独显报错退 78；文件头加 `// task-server-gpu-cards: all`。
2. **全部卡臂点名服务交的卡**：加 `lockstep --cards <张数>`、`name=every_card_arm_cards` 行与续跑核对；每臂的卡与型号串按臂取。
3. **开卡表整个进程一份**（静态 `OPENINGS_NOT_RETURNED_IN_THIS_PROCESS`）。
4. **跟核心层接口改名**（`ForegroundFsync` 改 `AnswerOfTheCallerBeforeTheNextBatch` 这一类）。
5. **内联真卡测试改成先录后判、经判定库**（`pairing_concluded_on_the_cards_through_the_pairing_library`，实现在 `crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs`，那一份不在这一轮被判的文件里）。
6. 加了一批单测（`mod tests` 里 118 个 `#[test]`，其中 2 个只在开了 `tests-on-a-real-gpu` 特性时编）。
7. 两张入口登记表：入口路径都写 `singlefs_background_gpu_tests::` 前缀，库 `lib.rs` 只有 4 行。

## 要两条腿答的

- 正推：第 1 到 5 件各自做的是不是规则说的——取卡与清场归服务（`.claude/rules/verification.md`「凡用 GPU 先清场，跑完不复原」、`.claude/rules/test-script-discipline.md`「用上现有的硬件加速」）、判定经那一份库（`.claude/rules/verification.md`「判定库只有一份」）、装置的臂与判据照跑前登记（登记原文现读，对着第 2 件「全部卡臂」的定义核）；两张入口登记表列的入口在源码里逐个找得到、单元图认这种写法（`crates/singlefs-background-gpu-tests/build.rs` 怎么读它们）。
- 攻方：哪一格会错。至少攻：续跑核对（`--cards` 与上一趟不同、交来的卡换了时，续跑会不会把两种卡的重复混进同一份产物）；卡对不上、一张都没交时每条子命令的出路，会不会悄悄退回 CPU；先做完只用 CPU 的活再取卡这一改，有没有哪条臂在取卡之前就用了卡；进程级开卡表在多线程下；第 5 件先录后判，录与判之间输入会不会变、判定键罩没罩全输入；装置把判定行（`name=verdict`、`repeat_done`）追加进 `--product` 文件、续跑时读回并据它跳过做完的重复，这算实验产物还是另开的判定存放处；钟速单测前提不成立时打「本次未跑」就返回、cargo 判绿；两张登记表里「参照一侧」那张没有测试逐组核、没有变异行；事务员线索第 3、6、7、9、10、11 条（`facts.md` 第一节第 5 样）逐条判真伪；123 行变异罩不罩得住第 1 到 5 件的每个判红分支。
- 两条腿都逐格判下面的检查单，回「成立」「不成立」或「推不出」。HEAD 里旧路径那一份已有的代码不在这一轮的射程里；只判 diff 里加与改的行与它们所在的函数。

### 检查单：走不到的理由二次查实

这批改动没有实现员报告（补轮）。e202 产品代码段全文的逐处表在 `e202-expect-src.tsv`（事务员数的：`.expect` 128 处、`panic!` 8 处、`assert` 8 处，`unwrap`、`unreachable!` 0 处）；腿从 `e202-move-against-head-old-path.diff` 里取出这一次新加与改过的那几处，逐处追它依赖的前提。两张登记表与 `lib.rs` 的见 `facts.md` 各自第 3 样。

### 检查单：类型与名字

新加的名字从 diff 里取（第 1 到 3 件点了几个）。事务员的线索：`LOCKSTEP_ARMS[0]` 这类拿下标取臂（`facts.md` 第一节第 5 样第 9 条），腿判它是不是这一次新加的、算不算用位置代替名字。裸整数当卡号、张数、片号的参数，`_ =>` 与 `as` 见 `facts.md` 第一节第 3 样「类型与名字」。

### 检查单：不兼容旧代码与旧格式

`facts.md` 第一节第 3 样「不兼容旧代码与旧格式」列了事务员查到的；续跑读回的产物行有没有认旧行形态的分支，腿核。

### 检查单：不新建判定库

写入点四处（`facts.md` 第一节第 3 样「不新建判定库」）：① 结果行追加到 `--product` 给的文件，含 `name=verdict`、`repeat_done`、`premise`、`positive_control` 行，续跑读回、按它跳过做完的重复，不经 prune-server；② 内联真卡测试的对拍结论经验证包判定库；③ 一条单测用自己起的临时服务；④ 构建脚本往 `OUT_DIR` 写两张键表。主 agent 没判 ① 算不算 `.claude/rules/verification.md`「判定库只有一份」说的「另写一张 TSV」：`admission: always` 写着这份装置每跑一次都现量、不复用上一次的判定，产物行是实验产物；续跑读它是「被打断后只补没做的重复」。两条腿各自判，并写明依据是规则的哪一句。

### 检查单：所有任务多机可用

读的本机状态列在 `facts.md` 第一节第 3 样「所有任务多机可用」：环境变量 `GPU_CARDS_BY_PRIORITY` 与领卡通道（服务交的）、`/proc` 下本进程的几样、相对仓根的路径、`.claude/heavy-thread-default`、写死的 `HOST_LOAD_CORES_AT_MOST = 16.0` 与内核节拍数 100。腿判哪一样会让两台判出不同的结局而不该不同。

### 检查单：测试优先多线程

`geometry-check` 按登记的每台默认重活线程数切波并行；`lockstep`、`concurrent` 的两条线程是跑法本身，臂与重复串着跑，装置注释给了理由（成对差的两边要同机先后）；`.claude/heavy-thread-exempt` 里没有这一份。腿判串着跑的理由成不成立。

### 检查单：带 --item 的命令与项

这份装置在登记表里的行列在 `facts.md` 第一节第 3 样「带 --item 的命令与项」：`crates/mutations.tsv` 123 行由门禁 crates-mutation-replay 按行点名；`.claude/gate.d/stage-inputs.tsv:31` 按包登记了这个测试包；搬迁登记在 `.claude/gate.d/path-moves.tsv`。腿核搬迁之后变异行的 cargo 参数都换成了新包名、筛选词筛得到点名的测试。

### 检查单：新崩溃流多罩了什么

本轮改动没有新加崩溃点重放流或崩溃枚举用例（事务员查过全文与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行、`scripts/crash/`）。
