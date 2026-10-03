# 里程碑三形式化验证还要用户定的几问：岔路单（2026-10-02）

<!-- doc-lint:not-numbers Y1 Y2 Y3 Y4 Y5 Y6 Y7 K7 K12 M2 M3 O1 O2 O3 S4 -->

批次 `records/2026-10-01-里程碑三形式化接入.md#7`。出处是现状表 `/tmp/claude-1000/m3-formal-remaining/report.md`「要用户定」一组第 1、2、3、5 条；第一轮岔路单 `research/prompts/m3-formal-r1-forks.md` 的 K12（O3）与 K7（挂载读法）是这里 Y1–Y3、Y7 的前身。

第一节照 `.claude/rules/three-way-inference.md`「交岔路时写岔路单」只写问题与候选定义，可以单独发给实验设计员；派发提示另要每个候选带代价，放第二节，量的与推的逐格标明。量法与原样输出在同目录 `report.md`。

`核` 指 `crates/singlefs-replay-prefix/src/lib.rs`（Verus 验的取前缀纯函数核）；`E165` 指装置 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`；O1、O2、O3 指 D13（验证路线） 已定项 6 表里的三个 oracle。

## 一、岔路

| # | 问题 | 候选（各自的定义） | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| Y1 | 七·6 的 O3（独立规约执行器）用什么形态 | **O3-两用**：把核的 Verus spec 改写成能对一个具体崩溃状态求值的形态（vstd 的 `exec_spec_verified!`，或手写可执行版并在 Verus 里证它与 spec 相等），拿它当 O3 判崩溃状态；D13 已定项 6「用另一种语言写（不用 Rust）」随之改。**O3-单写·Python**：用 Python 只照 `.claude/kb/decisions/` 与 `.claude/kb/invariants.md` 的文字另写系统配置、根环、journal 环三样的解析，择系统配置、择根、取前缀，以及「崩溃后允许落到哪几版」的判定；崩溃状态由它自己从录下的写流枚举；配阳性对照。**O3-单写·OCaml**：同上，语言换 OCaml | 两用：把同一处条款误读写进 Verus spec，两用时 refinement 与 O3 一起放过，而单写那一份抓得到；单写：最小原型的解析行数与判据行数同量级（K12 原翻面观测），或每态挂钟让它连层 0 快档那批状态都判不完 | 两用：改写后的可执行 spec 过不过 `--no-cheating`、核 spec 要改写多少行，量过；单写：一份只解析三样、判一个崩溃状态的最小原型，行数与每态挂钟量过 | 用户已定（2026-10-02）：O3 用 Python 单写（`records/2026-10-01-里程碑三形式化接入.md` 第 419 行） |
| Y2 | RefFS 做到哪一档 | **RefFS-语义级**：只在内存里按 POSIX 操作走，自己定义每次 fsync 之后什么算持久、崩溃之后允许看到哪几种 POSIX 状态，没有盘上格式。**RefFS-盘级**：另写一份带自己盘上格式的文件系统，自己写盘、自己在崩溃之后恢复（D13 已定项 3 射程里「工程量接近再写一个文件系统」那一档） | 同一段历史上两档给出的「崩溃后允许的集合」不同；或有一类错只在盘上表现、语义级判不出 | 两档各在一段短历史（例如崩溃注入快档的一段）上给出允许集合；语义级一份最小原型的行数量过 | 用户已定（2026-10-02）：RefFS 语义级（D13 已定项 3 换读法，原话在变更史）（`records/2026-10-01-里程碑三形式化接入.md` 第 419 行） |
| Y3 | 三 oracle 合谋测试怎么接 | 三个 oracle 取 O1（`crates/singlefs-harness/src/model.rs` 对拍）、O2（`crates/singlefs-checker`）、O3（Y1 选定那一份）。**合谋-坏实现**：登记一张坏实现表（形态照 `crates/mutations.tsv`），每行把实现改坏一处，在同一批历史与崩溃状态上三条 oracle 都跑，三条全放行即红。**合谋-坏镜像**：手造坏镜像喂 O2、O3，全放行即红（O1 不读盘，这一族只到两条）。**合谋-关一条**：拿已知被抓的坏实现，逐条关掉一条 oracle 的判定，剩下的仍要红（C407 判别力自证那一句） | 坏实现表里出现一行三条全放行（共同盲区）；或某一族根本到不了三条 | 坏实现表头几行在三条 oracle 上各跑一遍的挂钟；O3 有代码 | 用户已定（2026-10-02）：合谋测试三族都接（坏实现表、坏镜像、关一条自证）（`records/2026-10-01-里程碑三形式化接入.md` 第 419 行） |
| Y4 | C191（同源性计数器）的不一致事件从哪来 | **事件-E165**：E165 照条款手写的全函数 S（`specification_expectation`）当可执行的 M2 判定，与核在同一批状态上的结局不同记一条。**事件-checker**：池级 checker 实现 I-8.3（今天「未实现」），在层 0 每个崩溃状态上与恢复的结局不同记一条。**事件-可执行spec**：核的 Verus spec 改写成可执行（同 Y1 O3-两用那一步），在层 0 崩溃状态上与记录核对器的结论不同记一条。**事件-O3**：等七·6 落地，O3 与 RefFS 不同记一条 | 某一来源与它对拍的那一方是同一份读法，一条事件都产生不了；或事件出来之后「判谁错」无从判 | 每个来源写明对上 C191 列的哪一对；事件-E165 先证核的 spec 对给定输入只允许一个结局（否则 S 与核不同不等于 S 与 spec 不同） | 用户已定（2026-10-02）：C191 事件取 O3 对 RefFS（`records/2026-10-01-里程碑三形式化接入.md` 第 419 行） |
| Y5 | Verus 往哪扩（现状表 A2c） | **不扩**：维持「核证、其余由层 0 枚举与 E165 罩」。**扩到择根**：`choose_root` 的取最大那一截抽成纯函数核、写 spec。**择根加扫环**：再把 `scan_journal` 的「按 (实例代号, 计数器) 先到先得建表」抽成核。**再加择系统配置**：`choose_system_configuration` 与它的取值范围判定一起抽 | 某一截在 cargo 侧变异表里没有能红它的行而 Verus 必红世界红得出；或扩之后 Verus 一次超过门禁限时 | 每档一份原型量过行数与挂钟 | 用户已定（2026-10-02）：Verus 扩到择根加扫环（`records/2026-10-01-里程碑三形式化接入.md` 第 419 行） |
| Y6 | R3-7 择根与重放的合判放哪 | **合判-E165**：E165 新加一族，在内存盘上同时摆根环与 journal 环，调真 `choose_root` 与 `replay_journal`。**合判-Verus**：择根核与取前缀核在 Verus 里合成一条性质（要 Y5 至少到「扩到择根」） | 合判-E165 在界内找到择根与重放合起来才出现的反例；或 Verus 合成的那条性质在核的可信件之外还要新的可信假设 | 两种各给行数与一次跑多久 | 用户已定（2026-10-02）：合判 Verus 与 E165 两边都做（`records/2026-10-01-里程碑三形式化接入.md` 第 419 行） |
| Y7 | r1-四：可写挂载之后第二次恢复要不要量 | **不量**：K7 只取记录级与不动点两读，挂载读法在 I-8.4 的射程里写明没量。**崩溃注入里量**：在每个崩溃状态起可写挂载之后，对挂载后的池再做一次只读恢复，与第一次恢复的结局按「模掉什么」那一句比。**层 0 上量**：层 0 每个崩溃状态都起一次可写挂载再恢复 | 模掉约定的分量（实例代号、txg 这一类）之后仍有字段不等；或崩溃注入那一路一次都没走到「挂载之后再恢复」 | 「模掉什么」那一句条款先定（K7 够判条件原文）；崩溃注入快档一段上量一次 | 用户已定（2026-10-02）：在崩溃注入里量可写挂载之后第二次恢复（`records/2026-10-01-里程碑三形式化接入.md` 第 419 行） |

## 二、每个候选的代价

「量」是这一轮在本机量的（命令与原样输出在 `report.md`）；「现数」是这一轮用 `wc -l` / `grep` 数仓里参照文件的行数；「推」是拿参照行数乘比例推的，没量过。行数里「代码行」指去掉空行与整行注释之后的行。

两个比例都是现数的：择根原型 77 代码行对它替的 `choose_root` 21 代码行，约 3.7 倍；核 1 795 个非空行（类型 58、spec 577、证明 343、可执行与循环不变式 817）对它从 HEAD 的 `recovery.rs` 搬过来的取前缀逻辑 174 代码行（`replay_journal` 131 加三个辅助函数 43），约 10.3 倍。下面「推」的 Verus 行数都取这两个比例之间。

### Y1 O3 形态

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住 | 罩不住 |
|---|---|---|---|---|---|---|
| O3-两用 | 量：`exec_spec_verified!` 吃不下今天核的 spec 写法，抄核的形状（`CoreRecord<RootSource>` 加 `int` 下标量词）报 `generics not supported`；核 spec 32 个函数里 31 个带泛型、`Fn(usize)` 参数 14 处、`int` 57 处（现数）。量：择根那种不带泛型、`usize` 下标的 spec 它编得过（9 verified），但加 `--no-cheating` 报 `external_body/assume_specification not allowed`。推：核 spec 第 93–700 行（608 行）改写成它能吃的形态 600–900 行，或手写可执行版加相等证明 1 000–2 000 行（参照核的可执行部分 825 行与证明 351 行），另加崩溃状态转成核入参的胶水约 140 行（参照胶水 `replay_journal` 139 代码行） | `crates/singlefs-replay-prefix/src/lib.rs`（或新 crate）、`.claude/gate.d/verus-pins.tsv`、走 `exec_spec_verified!` 时 `research/scripts/verus-proof-check.py` 的作弊判法要开口子、接崩溃状态的 `crates/singlefs-checker-tier/src/crash.rs`、D13 已定项 6 正文 | 不要新工具；与 D13 已定项 6「不用 Rust」字面相反 | 推：每态微秒级（参照 E165 `light` 档 6 923 358 态 2.671 秒，那是真 `replay_journal` 加判据的数，不是可执行 spec 的） | 取前缀那一截在具体崩溃状态上的判定 | 解析、择根、择系统配置；Verus spec 自己读错的地方（两用时与 refinement 是同一份读法，第一轮判决 F5 一行） |
| O3-单写·Python | 现数参照：Rust 池级 checker 里同三样的解析与择取 644 代码行（`crates/singlefs-checker/src/lib.rs` 第 38–95 行 CRC 52、第 169–360 行系统配置与根槽 151、第 731–839 行 journal 记录 96；`image.rs` 第 201–627 行几何、系统配置、根环 345），checker 扫 journal 环的几个函数 143 代码行（`walk.rs` 第 4929–5107 行），E165 照条款写的 S 与辅助函数 202 代码行（第 1013–1243 行）；另一种语言照代码重写恢复路径的先例：`crash_judge_common.wgsl` 3 141 行加 `crash_judge_recovery.wgsl` 410 行。推：判一个状态 800–1 500 行，自己枚举崩溃状态 200–500 行（参照 `crates/singlefs-harness/src/segments.rs` 405 行），阳性对照另算 | 新目录（位置待定）、新门禁一道（参照 `checker-tier-lkmm.sh` 89 行、`checker-tier-formal-verification.sh` 147 行）、`.claude/gate.d/stage-inputs.tsv`、D13 已定项 6 射程句、`checks-owed.md` C447 | 不要：python3 3.12.3 在，`research/scripts/` 下 50 份 `.py` | 量：纯 Python 表驱动 CRC-32C 算 4 KiB 一块 0.299–0.320 毫秒（三次）。推：每态要验几块到几十块，每态 1–10 毫秒；层 0 D 全域 51 773 558 态单线程 14–144 小时；崩溃放量 26.16 亿态按 1 毫秒单线程 727 小时，只够跑快档或抽样 | 照文字另推一遍的解析与判定，Rust 作者的误读不共享 | 条文本身没写的；POSIX 语义（O1 管）；单写的人自己读错（靠阳性对照） |
| O3-单写·OCaml | 推：与 Python 同量级 | 同上，另加 OCaml 构建 | 新语言：仓里 0 份 `.ml`（`git ls-files '*.ml'`），本机有 `/usr/bin/ocaml`、`ocamlfind` | 推：比 Python 快一个量级以上，没量 | 同 Python | 同 Python |

### Y2 RefFS

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住 | 罩不住 |
|---|---|---|---|---|---|---|
| RefFS-语义级 | 现数参照：O1 `model.rs` 3 059 行加 `model_comparison.rs` 1 305 行，不管崩溃。推：加持久化与崩溃语义再 1 000–3 000 行 | 新模块或新 crate、对拍接线、门禁 `harness-model-differential-and-scenarios.sh`（239 行）或新一道；守 D13 已定项 3「不许共用崩溃点枚举的那段代码」 | 不要（用 Rust）；用别的语言时同 Y1 | 推：每段历史毫秒级 | fsync 的持久性承诺；崩溃后 POSIX 可见状态的允许集合（D17 已定项 4 交给 RefFS 的那一半） | 盘上格式、Merkle、记账、恢复算法本身 |
| RefFS-盘级 | 现数参照：`crates/singlefs-core/src/*.rs` 合计 35 235 行。推：1–3 万行 | 新 crate、崩溃注入与层 0 的对拍接线、门禁新一道 | 不要；换语言时同 Y1 | 推：与 core 同量级 | 盘级崩溃语义的第二份实现，可在同一历史上逐镜像比 | 两份实现对同一句条文的共同误读（Y3 管） |

### Y3 三 oracle 合谋测试

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住 | 罩不住 |
|---|---|---|---|---|---|---|
| 合谋-坏实现 | 推：门禁一道加驱动 300–800 行（参照 `checker-tier-formal-verification.sh` 147 行加判法 `research/scripts/verus-proof-check.py` 1 434 行、变异跑道 `research/scripts/mutate.sh` 1 173 行），坏实现表每行一条 | 新坏实现表、新门禁一道、`stage-inputs.tsv`；三条 oracle 各要一个「对同一批状态出结论」的入口 | 不要（O3 的语言照 Y1） | 推：每行一次冷编译加三条 oracle 各跑一遍；参照 E165 第二段 26 个变体 `--jobs 4 --threads 4` 整段 32 分钟（E165 页「复跑」一节，别的会话量的），折每变体约 5 分钟 | 三条一起放过的坏实现 | 表里没登记的坏法；D13 已定项 6 举的「nonce 永远为 0」今天造不出：加密关着，I-6.1–I-6.9 状态列都是「未实现」，checker 本来就判 nonce 全 0（`crates/singlefs-checker/src/image.rs` 第 696–699 行） |
| 合谋-坏镜像 | 推：100–300 行；参照坏镜像用例 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 5 442 行（O2 一家的） | 同上 | 不要 | 推：秒级 | 坏镜像上 O2、O3 的共同盲区 | O1（不读盘） |
| 合谋-关一条 | 推：在坏实现那一道上加开关，50–150 行 | 同合谋-坏实现 | 不要 | 推：合谋-坏实现的三倍 | C407 判别力自证那一句 | 同合谋-坏实现 |

### Y4 C191 事件来源

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住（对上 C191 的哪一对） | 罩不住 |
|---|---|---|---|---|---|---|
| 事件-E165 | 现数：S 已在（202 代码行）。推：核 spec 对给定输入只允许一个结局的 Verus 引理 50–200 行；事件表与它的判法（≥ 5 次不一致而判 spec 错 0 次即红、喂一张「5 次全判 model 错」的表必红）100–200 行 | E165 装置、核（引理）、新事件表、门禁一格 | 不要 | 现数参照：E165 `light` 档 2.671 秒、第一段合法历史一族 29.061 秒（E165 页） | S 与核（经 Verus spec）的分歧；它是 spec 对 spec，C191 列的三对里没有这一对 | E165 界外；解析与择根 |
| 事件-checker | 推：I-8.3 的 checker 判定 150–400 行，参照 `walk.rs` 里 `judge_publish_ordinals_and_last_record_flags` 158 代码行（第 5397–5587 行）与扫 journal 环几个函数 143 代码行；坏镜像用例另算 | `crates/singlefs-checker/src/walk.rs`、`image.rs` 的 `IMPLEMENTED_INVARIANTS`、`.claude/kb/invariants.md` I-8.3 状态列、`checker_known_bad_images.rs`、事件表 | 不要 | 推：层 0 每态多一判，微秒级 | O2 与恢复（恢复那一侧由 Verus spec 罩）的分歧；对不上 C191 列的三对中任一对 | checker 与实现同一个作者同一周写的那种同源（D13 已定项 13 原文那一句） |
| 事件-可执行spec | 同 Y1 O3-两用，另加接记录核对器（`crates/singlefs-checker-tier/src/crash.rs` 的 `check_records` 第 48 行起）推 50–150 行 | 同 Y1 O3-两用，加 `crash.rs` | 同 Y1 O3-两用 | 同 Y1 O3-两用 | C191 第三对（M2 的 spec 与记录核对器） | 同 Y1 O3-两用 |
| 事件-O3 | Y1 选定的 O3 加 Y2 选定的 RefFS | 同 Y1、Y2 | 同 Y1 | 同 Y1、Y2 | C191 第一对（O3 与 RefFS） | 两样都落地之前一条事件都没有 |

### Y5 Verus 往哪扩

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住 | 罩不住 |
|---|---|---|---|---|---|---|
| 不扩 | 0 | 只改 D13 已定项 1 落点列的说法 | 不要 | 0 | — | 择根、扫环、择系统配置在 Verus 里一截都没有；今天 `crates/mutations.tsv` 里改 `choose_root` 比较键的行 0 行（`grep -c 'candidate_key\|current.instance'`） |
| 扩到择根 | 量：原型 89 行，77 代码行（类型与头 7、spec 39、可执行 31，其中 6 行是 `ensures` / 循环不变式 / `decreases`，没有证明引理）；胶水草稿 32 行，31 代码行（今天的 `choose_root` 21 代码行） | 核（或核所在 crate 加一个模块）、`recovery.rs` 的 `choose_root`、`verus-pins.tsv` 的 verified 数（推：加 3）、`verus-mutations.tsv` 加 3 行（原型量过的三条必红世界）、`crates/mutations.tsv` 加 cargo 侧的行 | 不要 | 量：Verus 0.2026.09.20 `--no-cheating` 单文件 3 verified，挂钟 0.71–0.81 秒（四次），Verus 自报 total-time 354–389 毫秒；三条必红世界各「2 verified, 1 errors」、空真探针红 | 自证过的槽里取 (checkpoint_txg, 实例代号) 最大、同键取槽序最前 | 读槽、自证校验和、fsid、区域到盘的映射、槽的遍历次序（都在胶水里） |
| 择根加扫环 | 推：扫环核 140–400 行（`scan_journal` 39 代码行乘 3.7–10.3）；vstd 有 `BTreeMap` 的 `insert` / `contains_key` / `get` 规格，没有 `entry` / `or_insert`（vstd `std_specs/btree.rs` 里 grep 0 处），先到先得要改写成先查后插 | 同上，加 `scan_journal`；R3-10（先到先得）可以由必红世界判 | 不要 | 推：再加约 1 秒 | 加上 (实例代号, 计数器) 唯一与先到先得 | 偏移枚举、提示、解析与 CRC |
| 再加择系统配置 | 推：1 300–3 600 行（`recovery.rs` 第 593–1061 行系统配置读法、取值范围、择取 351 代码行乘 3.7–10.3） | 同上，加 `choose_system_configuration` 一族 | 不要 | 推：几秒；rlimit 要不要调没量 | 加上系统配置的择取与取值范围判定 | 槽的读与解析 |

### Y6 合判放哪

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住 | 罩不住 |
|---|---|---|---|---|---|---|
| 合判-E165 | 推：200–500 行（参照 E165 `generate_slot` 第 336–422 行 87 行、`record_table_and_disk_of` 第 729–812 行 84 行） | E165 装置、跑前登记、复跑登记、钉表 | 不要 | 推：根环取值多一维，状态数按根环摆法成倍涨；参照第一段合法历史一族 49 753 566 态 29.061 秒 | 真 `choose_root` 加真 `replay_journal` 在界内合起来 | 界外 |
| 合判-Verus | 推：在 Y5「扩到择根」之上再 30–80 行（合成性质与胶水） | 核、`verus-pins.tsv`、`verus-mutations.tsv` | 不要 | 推：秒级 | 两个核合起来无界 | 胶水与读盘（同 Y5） |

### Y7 r1-四

| 候选 | 另写多少行 | 动哪些文件 | 新工具或新语言 | 一次跑多久 | 罩住 | 罩不住 |
|---|---|---|---|---|---|---|
| 不量 | 0；I-8.4 射程补一句 | `.claude/kb/invariants.md` 或 D13 已定项 1 | 不要 | 0 | — | 挂载读法 |
| 崩溃注入里量 | 推：判法 60–150 行，参照 `crates/singlefs-checker-tier/src/crash_injection.rs` 里已有的可写挂载 `mount_and_publish_once_on_the_crash_image`（第 1095–1333 行）与 `judge_the_writable_mount_on_the_crash_image`（第 1334–1410 行，77 行）；先要一句「模掉什么」的条款 | `crash_injection.rs`、`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs`、`crates/mutations.tsv`、D13 已定项 1 或 I-8.4 | 不要 | 推：每个崩溃状态多一次只读恢复，加不到一成；参照快档「debug 下单跑一百来秒」（`.claude/gate.d/stage-inputs.tsv` 第 44 行注释，别人量的） | 抽样历史上挂载之后再恢复 | 层 0 全量；抽不到的历史 |
| 层 0 上量 | 推：在崩溃注入那一份之上再 100–300 行（层 0 今天不跑可写挂载，`crash_injection.rs` 文件头写着） | `crates/singlefs-checker-tier/src/crash.rs` 与层 0 的钉数 | 不要 | 推：每态多一次可写挂载加恢复，层 0 挂钟成倍涨 | 层 0 全量 | 同不量以外的其余维 |
