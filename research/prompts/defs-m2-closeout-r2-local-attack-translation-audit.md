# 转述核对表：defs-m2-closeout-r2-local-attack（2026-09-26）

逐句核对 `research/prompts/defs-m2-closeout-r2-local-attack.md`（本地攻方 E1 的
F14：按事实表把 `implementation-workflow.md` 改后重型清单的每一类，与
`.claude/hooks/lib_heavy_tests.py` 的 `classify`、`cargo_use` 逐格核对，
Q1–Q16）里每一句英文转述与中文原文的对照。行号现查：`.claude/rules/implementation-workflow.md`
与 `.claude/hooks/lib_heavy_tests.py` 都用工作区当前文件（这一轮被判的正是这两份文件改后
/ 现有的字面本身，不是历史快照）。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与派发消息
「提示里不写`::`这类Rust路径」，以及本地攻方定义「答复不写代码行号与文件行号」），
只在这份核对表里写死来源文件与行号；核对表与运行记录里，样本自带的行号
（若模型自己写出行号）一律标「模型自给、未核」。

格式：英文项 / 原文文件:行 / 首稿缺的 / 定稿理由。核对粒度按「同一句中文」分组。

## 表一：清单前两句与 C1–C5（源文件 `.claude/rules/implementation-workflow.md`）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| 提示第二段（判定逻辑在 classify 与 cargo_use 两个函数里） | `:48`（「**重型测试**，与`.claude/hooks/heavy-test-guard.sh`拒的逐类相同（判定在`.claude/hooks/lib_heavy_tests.py`的`classify`）：」） | 无遗漏：「判定在classify」这一句转述为「classification logic ... lives in two Python functions ... classify and cargo_use」——多写了cargo_use一个名字，原句只点名classify；这一处补充见文末「多出来的」表 | classify会调用cargo_use（源码`:315`-`:316`「if name == "cargo": return cargo_use(...)」），提示若只提classify而不提cargo_use，模型无法知道cargo命令的判法在哪个函数里，Q1–Q16逐格核对必须两个函数都给 |
| C1（层0全量） | `:50` | 无遗漏：四种判法（54号、`--test`名字含layer0或通配命中、不挑目标而包里有layer0目标、直接执行layer0测试二进制）全部逐条译出（C1.1–C1.4） | C1.5是加的逻辑补集，见文末「多出来的」表 |
| C2（QEMU） | `:51` | 无遗漏：三种判法（55号、`qemu-system-*`、vm-bench.sh含--selftest）全部译出 | 按字面直译 |
| C3（herd7） | `:52` | 无遗漏：三种判法（57号、lkmm.sh、herd7，且都注明「带什么参数都算，只取版本号的也算」）全部译出 | 按字面直译，「只取版本号的也算」这个限定词在C3.2、C3.3里各自保留（「including an argument that only asks for a version number」） |
| C4（crates变异整表） | `:53` | 无遗漏：两种判法（59号、mutate.sh参数含crates/mutations.tsv）全部译出 | 按字面直译 |
| C5（全量cargo test） | `:54` | 无遗漏：四种判法（`--all`/`--workspace`、工作区根裸跑、不挑目标而范围等于全部成员、check.sh）全部译出（C5.1–C5.4） | C5.5是加的逻辑补集，见文末「多出来的」表 |

## 表二：C6–C12（源文件 `.claude/rules/implementation-workflow.md`）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| C6（整轮门禁） | `:55`（「整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest`不算）。」） | 无遗漏：gate.sh（带什么参数都算，C6.1）、gate-staged.sh不带--selftest（C6.2）、gate-staged.sh带--selftest不算（C6.3）全部译出；C6.3不是加的，原句「--selftest不算」本身就是这一句自带的例外分句 | 按字面直译 |
| C7（全部实验复跑） | `:56`（「全部实验复跑：87号。」） | 无遗漏 | 按字面直译 |
| C8（E152装置） | `:57`（「E152装置：`e152-file-system-benchmark`（直接起、或`cargo run --bin`它）、`research/scripts/e152-run.sh`。」） | 无遗漏：两种判法（直接起或cargo run --bin、e152-run.sh）全部译出 | 按字面直译 |
| C9（包装透传） | `:59`第一句（「只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env`这类包装里面的同样算。」） | 无遗漏：六种包装（bash -c、capped.sh、run-with-memory-cap.sh、nice、timeout、env）全部逐一列出 | 按字面直译 |
| C10+C11（脚本逐行读、按名字判两件事同一句） | `:59`第二句（「命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个`classify`判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check`第81行调`herd7 -version`，整条算herd7）；门禁阶段与`lib_heavy_tests.py`的`KNOWN_SCRIPT_LOCATIONS`登记的仓内脚本按名字判、不读正文。」） | 原文「第81行」这个行号首稿照抄进了C10.1与K17，核对时依「答复不写代码行号与文件行号」这条纪律去掉——改写成「one specific line inside that script's own file」，不点具体第几行，避免把一个行号事实喂给模型、诱它在自己的答复里也写行号；「逐行照同一个classify判」「按名字判、不读正文」两个事实完整保留 | 具体行号（81）不影响C10.1要判的核心事实（脚本内部有一行会命中某个判法、因此整条被算成那个判法），去掉行号不丢判据；C10（脚本逐行读）与C11（按名字判、不读正文）来自同一句里用「；」分隔的两个并列分句，按「同一句中文」的核对粒度合并成一行 |
| C12（非重型门禁号） | `:59`第三句（「`.claude/gate.d/`下54、55、57、59、87之外的阶段不是重型，谁都能跑。」） | 无遗漏 | 按字面直译，不是加的逻辑补集——这句本身就是允许性陈述 |

## 表三：源码文档字符串译成英文（源文件 `.claude/hooks/lib_heavy_tests.py`）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| classify() 的文档字符串 | `:285`（「一条已经剥掉前缀与包装的命令：返回 HeavyTest 或 None。」） | 无遗漏 | 按字面直译，放进代码里当注释保留 |
| cargo_use() 的文档字符串 | `:159`（「cargo 这一条算不算重型：返回 HeavyTest 或 None。」） | 无遗漏 | 按字面直译 |
| test_binary_name() 的文档字符串 | `:256`（「直接执行的是 cargo 编出来的测试二进制时交回它的名字（去掉哈希），不是时交 None。」） | 无遗漏 | 按字面直译 |
| STAGE_KIND 上方注释 | `:55`（「# 只有这几道阶段是重型；.claude/gate.d/ 下其余阶段谁都能跑」） | 无遗漏 | 按字面直译，放进STAGE_KIND说明段落 |

## 表四：classify() 内部拼接给人看的中文消息串译成英文（源文件同上）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| 门禁阶段消息 | `:289`（`f"门禁 {stage} 号（{name}）"`） | 无遗漏 | 译成`f"gate stage {stage} ({name})"`，占位符与结构不变 |
| layer0测试二进制消息 | `:292`（`f"直接执行名字含 layer0 的测试二进制（{binary}）"`） | 无遗漏 | 直译，占位符不变 |
| vm-bench.sh消息 | `:296`（「research/scripts/vm-bench.sh（--selftest 也起虚机）」） | 无遗漏 | 直译 |
| mutate.sh消息 | `:305`（「research/scripts/mutate.sh 跑 crates/mutations.tsv 整表」） | 无遗漏 | 直译 |
| check.sh消息 | `:308`（「check.sh（里面是全量 cargo test）」） | 无遗漏 | 直译 |
| gate-staged.sh消息 | `:312`（「research/scripts/gate-staged.sh（跑 gate.sh --staged）」） | 无遗漏 | 直译 |

## 表五：cargo_use() 内部拼接给人看的中文消息串译成英文（源文件同上）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿理由 |
|---|---|---|---|
| 全量workspace/all消息 | `:206`（「cargo test 带 --workspace / --all」） | 无遗漏 | 直译 |
| 工作区根裸跑消息 | `:217`（`f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 cargo test"`） | 无遗漏 | 直译，占位符不变 |
| 范围等于全部成员消息 | `:227`（`f"cargo test 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 ..."`，续行在`:228`） | 无遗漏：跨两行的f-string整句译出 | 直译，占位符不变 |
| 不挑目标命中layer0消息 | `:233`（`f"cargo test 不挑目标，会跑到名字含 layer0 的测试二进制（{'、'.join(layer0_targets)}）"`） | 无遗漏 | 直译，中文顿号连接符`、`换成英文逗号加空格`, ` |
| 通配命中layer0消息 | `:238`（`f"cargo test --test {pattern} 命中名字含 layer0 的测试二进制（{'、'.join(matched)}）"`） | 无遗漏 | 直译，顿号同上换成逗号 |

## 英文比原文多出来的限定词、条目（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的内容 | 为什么加 |
|---|---|---|---|
| C1.5（narrow away from layer0 → allow） | `.claude/rules/implementation-workflow.md:50` | 整条C1.5都是加的：原文只列了「不挑目标而包里有layer0目标」算重型，没有反过来明说「挑了别的目标（比如--lib）且没有--test命中layer0就不算」 | 这是「不挑目标」这个前提的直接逻辑补集；不加这一条，K18、K29两条EXIT 0的探针就没有任何C项可以匹配，Q14的漏列检查也覆盖不到它们 |
| C5.5（narrow + -p sole member → allow） | `.claude/rules/implementation-workflow.md:54` | 整条C5.5都是加的：原文「不挑目标而包的范围是工作区全部成员的」只说了不挑目标这一种情形算重型，没有反过来明说「挑了目标（比如--lib）时，即使那个包恰好是工作区唯一成员，也不算」 | 同上，直接逻辑补集，用于核对探针K29（cargo test -p e7-index-bench --lib，e7-index-bench是research/工作区唯一成员） |

## 首稿里与源码或格式规则本身对不上、写完之后自查改正的两处（不是中文转述缺口，另记）

- 黑箱助手函数清单首稿漏了`nearest_manifest`：`cargo_use`第`:210`行（`manifest_path = ... (nearest_manifest(directory) if directory else None)`）直接按名字调用了这个函数，首稿的「不给出函数体、当黑箱」清单里只写了`cargo_subcommand`、`manifest_sections`、`workspace_root_manifest`、`workspace_packages`、`layer0_test_targets`五个，漏了`nearest_manifest`——凡是没给`--manifest-path`的K探针（K1–K29全部如此）都会走到这一行，若不把它列进黑箱清单，模型看到这个未声明的函数名会缺一处可依据的说明。用`research/scripts/replace-once.py`定点改成六个，加`nearest_manifest`。
- C10.1与K17首稿照抄了原文「第81行」这个具体行号，核对时依本地攻方定义「答复不写代码行号与文件行号」这条纪律去掉，改写成不带行号的「one specific line inside that script's own file」——原文本身带着这个行号是可以的（`.claude/rules/implementation-workflow.md:59`自己的措辞），但把它原样喂进提示会让模型有样学样、在自己的Q10/Q14/Q15答案里也写出行号，与提示末尾「不得引用任何行号」的格式要求自相矛盾。

## 没做什么（本核对表）

- 未核对分给云端攻方腿（Opus，E1、E2）与云端辩方腿（Sonnet，E3）的材料与判词——那两支材料按分工表不归本地腿，这份提示本身也没有引用它们。
- 未判多次抽样之间Q1–Q16的具体答案方向是否一致——那是运行记录与主agent的事，不是这份核对表的事。
- 未核对`implementation-workflow.md`「重型测试只在提交时跑」一节里`:61`–`:78`的「跑不跑」场合表、herd7/QEMU阶段判什么两张表——F14这一格只判「清单与classify/cargo_use逐类对不对得上」，这两张表讲的是「谁在什么场合能跑」，不是分类判法本身，不在这一格的射程。
