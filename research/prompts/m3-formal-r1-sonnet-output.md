# m3-formal-r1 正推腿报告（Sonnet，2026-09-30）

轮名 m3-formal-r1，格 F1–F5，立场正推。方案按 `crates/` 今天的实现谈；代码引文读的是冻结副本 `/tmp/claude-1000/m3-ring-r1/tree/crates/`（cite-check 用 `--root` 指它）；kb 引文写 kb 文件自己的行号。
记号：**[量]** = 这一次我自己真跑或真数，命令与输出在报告里；**[推]** = 从代码与条款推的，写了「量它要建什么计数实验」；**[线索]** = 别家的事实（`m3-formal-r1-prior-art.md`），未在本项目验证，只提假设用；**[未核]** = 我没核。
模型目录 `research/prompts/m3-formal-r1-sonnet-model/`：`replay_model.py`（`replay_journal` 决策核的 Python 译本 + 三条性质的小界穷举，210 行）、`exits.py`（出口命中）、两份原样输出 `replay_model.out`、`exits.out`。**它翻的是我读 Rust 后写的译本，不是 Rust 本身**，只用来量「翻这一截多少行」「穷举规模怎么长」「出口要多大的界才碰得到」，证不了 Rust 满足性质。

## 0　先摆我量到的（全文引用这一节）

### 0.1　判定核有多大 [量]
事实表（`research/prompts/m3-formal-r1-facts.md`）的逐函数清单在 `/tmp/claude-1000/m3-r1/formal-facts/closure_K.tsv`；我对它重新求和，并把「读字节、算 CRC」的函数剔掉，只留决策：

```
$ awk -F'\t' '$1 ~ /^crates/ {l+=$4; c+=$5; n++} END {print "closure_K 逐行求和 fn="n" lines="l" code="c}' closure_K.tsv
closure_K 逐行求和 fn=36 lines=577 code=527
决策部分（recovery.rs 六个 + journal.rs 的 from_record_flags_byte / back_chain_of / record_offset + system_configuration.rs 的 journal_in_flight_record_limit）：
fn=10 lines=287 code=242
```
- 择根那一簇（`choose_root` 及它下面四个，recovery.rs:1055-1168）82 行；`scan_journal` 38 行（facts 表逐格）。所以「择根 + 扫环 + 取前缀」的**决策**只有约 407 行（287 + 82 + 38），闭包 577 / 770 行里其余是字节解析与 CRC。
- `replay_journal` 里 `break` 出口 10 个（9 个独占一行 + 反向链那一个写在 `match` 臂里），`continue` 1 个，`expect(` 1 处（`recovery.rs` 里在在飞上限的 `usize::try_from`）；只有两个出口依赖 CRC 判出来的结果：反向链 `Broken` 与点名单元校验失败。

### 0.2　我把这一截翻成 Python，量了三件事 [量，一个样本、一个人翻]
```
$ python3 replay_model.py    （nice -n 19，2.3 秒，输出全文在 replay_model.out）
任意记录表 N=1: 表数=97 句法违例=0 连续违例=0 施加序列 txg 非单调=0
任意记录表 N=2: 表数=9153 句法违例=0 连续违例=0 施加序列 txg 非单调=0
任意记录表 N=3: 表数=846369 句法违例=0 连续违例=0 施加序列 txg 非单调=80
合法历史 各发布记录数=(2, 2, 2, 2): 记录数(含锚点)=9 落盘状态数=19683 违例 句法=0 最长完整前缀=0 单调=0
变异（任意一条都算末条）：合法历史 各发布记录数=(2, 2): 落盘状态数=243 违例 句法=81 最长完整前缀=81 单调=0
```
① **行数**：`replay()` + 锚点 + `readable_same_publish_follows` 共 67 行、62 码行（`replay_model.py` 23–89 行），对应 Rust 决策部分 242 码行 + 择根 / 扫环——译本是 Rust 码行的约 0.26 倍，前提是反向链重读与点名校验各抽成一个布尔、错误类型与报告结构省掉。
② **穷举规模**：合法历史下落盘状态数恰为 3^n（n 条记录，每条「没落 / 落了验不过 / 落了验得过」）：n=2,3,4,5,7,9 时输出 9,27,81,243,2187,19683。
③ **出口要多大的界**（`exits.out`）：`replay()` 的 10 个 `break` 出口，N=1 命中 3 个，N=2 时 10 个全部命中；没有一个要 N≥3 才碰得到。
④ **判别力**：把「末条标志」改成「任意一条都算末条」（七·3 写的必出反例的世界），句法性质 81 / 243 个状态红；原译本 0 红。

### 0.3　本机与环境 [量，2026-09-30]
```
$ for t in kani cargo-kani z3 java tlc apalache-mc verus creusot; do printf '%s: ' $t; command -v $t || echo 'not found'; done
kani: not found  cargo-kani: not found  z3: not found  java: not found  tlc: not found  apalache-mc: not found  verus: not found  creusot: not found
$ python3 -c 'import z3'  →  ModuleNotFoundError: No module named 'z3'
$ python3 -m ensurepip --version  →  /usr/bin/python3: No module named ensurepip
$ sudo -n true  →  sudo: a password is required
$ nproc → 32；free -g → 总 60 / 可用 47；df -h /home → 可用 1.9T；rustup toolchain list → stable、nightly、1.90.0
$ curl -sI https://github.com | head -1  →  HTTP/2 200      （一次 HEAD，没下载任何东西）
$ ls ~/.opam/default/bin | grep -c herd7   有 herd7；which opam → /usr/bin/opam
```
事实表已给的 grep 我在冻结副本上重跑：`grep -rn "I-8\.3" crates --include=*.rs | wc -l` → 0；`grep -rn "幂等\|idempot" … | wc -l` → 0；`grep -rn "RefFs\|RefFS" crates | wc -l` → 0；`grep -rn 合谋 crates | wc -l` → 0；`grep -rln "refinement" crates --include=*.rs | wc -l` → 0。
含义：装工具的前置——没有 sudo、没有 pip、没有 Java；有网络（只测了 github 一次 HEAD）、有 32 核 60 GB、有 opam 与三条 Rust 工具链。**能不能装、要多久没试**（这一轮不装）。

## F1　候选与代价

代价表里「量」的格只来自第 0 节；其余全是推的，末列写了要建的计数实验，实验名 E-A…E-E 在 F5 之后的「计数实验清单」里定义。
D13 已定项 1 射程写明的维，逐字：「refinement 的边际价值只在层 0 枚举取不到的那几维——载荷字节、记录条数 / 段数 / txg 数（含环回绕）、上一条时间线的残留、槽数与设备数」（`.claude/kb/decisions/13-验证路线.md:35`）。

### 候选 ①　Python + Z3，另写可符号执行的恢复副本
| 项 | 内容 | 标记 | 要量 |
|---|---|---|---|
| 另写行数 | 副本（符号化）120–160 行：译本实测 62 码行，符号执行要把 `break` 出口改成「停止标志 + ite」，取 ×2；择根 / 扫环抽象 30–50；崩溃状态生成（记录三态、根槽）40–60；spec 谓词（M1、M2、M3 单调要两份记录表）80–120；Rust 侧导出记录表与结果 150–250（`replay_journal` 是 `pub`，`crates/singlefs-harness/tests/row_publish_record_without_its_root.rs:203` 已有直接调用的先例）；Python 差分驱动 60–100。**合计 480–740 行，其中 Rust 150–250** | 推 | E-A |
| 证到哪条 | M2 句法（施加恰是整次发布的拼接）与 M3 单调（在合法历史前提下）；证的对象是**副本**，副本 = Rust 靠差分测 | 推 | — |
| 一次跑多久 | 每条记录进判定的位宽 245（32 + 48 + 64 + 64 + 1 + 32 + 1 + 2 + 1，推）；N≤8 条记录约 2 000 输入位，约 12 个出口；分钟量级 | 推 | E-B、E-E |
| 哈希 / 校验和 | CRC 抽成两个布尔（反向链三态、点名单元校验），`crc32_castagnoli` 不进副本；`replay_journal` 里依赖 CRC 结果的出口 2 / 10 | 量 + 推 | E-D |
| spec 从哪来 | 只照 `invariants.md` 与决策文字手写，与副本分开写（见 F2） | — | — |
| 与实现绑定 | 记录表差分：同一张表喂 Rust `replay_journal` 与副本，比 `prefix_applied`、`effective_root`、`verification_passed`（`JournalScanReport` 已有这些字段） | 推 | — |
| 装工具 | z3-solver：无 pip、无 ensurepip、无 sudo，网络通；装法（推）：curl 取 wheel 解压进用户目录，或 `opam install z3`；未试 | 推 | — |
| 1–2 天小原型 | 能。M2 在 3 条记录：我已用**无 Z3 的显式枚举**做出（0.2 ①，N=3 全部 846 369 张表 2.3 秒）；换 Z3 = 把 97 个取值换成 64 位符号 | 量 + 推 | — |

### 候选 ②　Kani，在 `crates/` 的 Rust 上直接有界模型检查
| 项 | 内容 | 标记 | 要量 |
|---|---|---|---|
| 另写行数 | 无第二语言。Rust 侧：符号记录表构造（`JournalRecord` 带 `Vec<NamedUnit>`、`NodePointer`，要给六七个类型加 `kani::Arbitrary` 或手写构造）60–120；符号 `PoolReader`（`recovery.rs:81` 的 trait，四个方法）40–80；桩 `JournalRecord::parse`、`crc32_castagnoli`、`wide_checksum_field_holds`、`back_chain_of` 40–60；三条性质 harness 60–120；`cfg(kani)` 与 lint 登记 10–20。**合计 210–400 行**。想压 `BTreeMap` 与 `dyn PoolReader` 的代价而抽出纯函数，要改 `recovery.rs` 80–150 行（这是改生产代码） | 推 | E-A |
| 证到哪条 | M2 句法、M1 的「有定义（不 panic）」在决策核内（Kani 默认查 panic 与溢出，闭包里 `expect(` 24 处，facts 表逐格）、M3 单调（同 ①，要合法历史前提）；证的对象是**Rust 决策核 + 桩** | 推 | — |
| 一次跑多久 | 界 = 循环展开 N+1。符号 `BTreeMap` 的插入与 `range`、`Vec<&JournalRecord>`、`named` 的内层循环（≤67 项，要限到 1–2 项）是主要代价；`record_offset` 里符号 `%` 与 `/` 的 64 位除法是另一处。N≤3–4 分钟级，N≥6 不知道；**没有本机数，别家的 CI 时长（线索）无一个是存储代码** | 推 | E-B、E-E |
| 哈希 / 校验和 | `crc32_castagnoli` 整体桩成「同输入同输出的未解释函数」，否则 `OnceLock` 查表（`checksum.rs:10-35`，8×256）与逐字节循环会被展开；`parse` 也要桩，因为反向链重读要 parse 出的记录等于前一条：「if JournalRecord::parse(&bytes, previous.filesystem_identifier).as_ref() == Some(previous) {」（`crates/singlefs-core/src/recovery.rs:3028`）——桩后这一步变成一条假设：**parse 是记录写出方的逆**，它由层 0 的真镜像在盖，形式上没证 | 推 | E-D |
| spec 从哪来 | 断言与合约住在 harness 里，紧挨着实现；独立性靠写断言的人只看 kb（结构上不独立，见 F2） | 推 | — |
| 与实现绑定 | 直接编译同一份源码，无差分；绑定点在桩（parse、CRC、reader） | 推 | — |
| 装工具 | `cargo install kani-verifier` 与 `cargo kani setup`（要下载 CBMC 与专用 nightly，体积 / 时长未核）；网络通，无 sudo 不影响；工作区要加 `cfg(kani)` 的 check-cfg 声明 | 推 | — |
| 1–2 天小原型 | 勉强能：M2 在 3 条记录、桩齐；风险在 `dyn PoolReader` 与 `BTreeMap` 的支持度，「Partial」是别家文档的说法（线索），本机未试 | 推 | — |

### 候选 ③　TLA+ / Apalache 协议模型，与代码靠 trace 对拍
| 项 | 内容 | 标记 | 要量 |
|---|---|---|---|
| 另写行数 | TLA+ 模块：恢复动作（择根、扫环、取前缀）120–200；崩溃状态（记录三态）与合法历史的写方 40–80；性质与 MC 配置 40–60；Rust 导出 150–250（与 ① 同一份）；TLA+ 侧读导出的记录表 50–100。**合计 400–690 行**，其中 Rust 150–250 | 推 | E-A |
| 证到哪条 | M2 句法、M3 单调，对象是**TLA+ 里的算法 = 我读 D23 已定项 14 五条口径写出来的**；M1 的「不 panic」不可达（TLA+ 没有 panic） | 推 | — |
| 一次跑多久 | TLC 显式状态：合法历史下状态数 = 3^n（0.2 ②）；n=12 约 53 万、n=16 约 4 300 万；再乘写方步骤的交错数（不建写方则不乘）。**取值域只能取小集合（txg 三四个值）**；Apalache 可上整数，但别家 10–15 步后 Z3 变慢（线索） | 量 + 推 | E-C、E-E |
| 哈希 / 校验和 | 同 ①：布尔字段；TLA+ 里没有字节 | 推 | E-D |
| spec 从哪来 | 模块本身就是「按文字写出的算法」，性质是它的谓词；它与代码的关系只有对拍（见 F2） | 推 | — |
| 与实现绑定 | trace 对拍。**崩溃中间态罩不罩得住**：一个崩溃状态是「记录表 + 根槽 → 结果」的一次函数求值，没有「下一步」，所谓 trace 只有一个状态。对拍退化成与 ① 完全相同的记录表差分，TLC 那一边只提供「小域上穷举」，不提供「与代码的更强绑定」；MongoDB 的 trace 验证 10 周放弃（线索，未在本项目验证；差异：它验的是并发复制的多步 trace） | 推 | — |
| 装工具 | Java + tla2tools.jar（或 Apalache 发布包）：本机无 java、无 sudo，只能把 JDK 压缩包解到用户目录；未试 | 推 | — |
| 1–2 天小原型 | 能：M2 在 3 条记录约 100 行 TLA+；Java 装不上则不能 | 推 | — |

### 候选 ④　Verus，把 `replay_journal` 那一截改写成 Verus 可验的 Rust
| 项 | 内容 | 标记 | 要量 |
|---|---|---|---|
| 另写行数 | 改：抽出纯函数 `decide_prefix(root_key, 已排序记录切片, 上限, 每条记录的反向链判与点名校验判) -> (施加条数, 末根键)`，Rust 80–150 行（今天的 `replay_journal` 在循环里**按需**读盘：`recovery.rs:3121` 起的反向链重读与 `recovery.rs:3185` 的点名校验，只对走到的记录做；抽成预取会多读，harness 里 `read_tally.rs` 数块层读，改动后会不会动它我没查，[未核]）。加：trusted spec 200–350；证明（循环不变式）取别家证明对代码 2.6–13.3 倍的区间（线索）× 100–150 行 = 260–2 000，取中 500–900。**合计 800–1 500 行，改生产代码 80–150** | 推（类比来源是线索） | E-A |
| 证到哪条 | M2 句法无界（记录条数不设界）、M3 单调；对象是**抽出来的纯函数**；M1 的「不 panic」在纯函数里可证 | 推 | — |
| 一次跑多久 | 验证本身秒级（别家 12–54 秒，线索）；成本在写证明的人天 | 推（线索） | E-E |
| 哈希 / 校验和 | `crc32_castagnoli` 标 `external_body`，配一条 trusted 的 `spec fn`；别家 Verus 日志同做法（线索） | 推 | E-D |
| spec 从哪来 | `spec fn` 手写，与执行体同一 crate、同一文件；Verus 要求 spec 与代码在一起 | 推 | — |
| 与实现绑定 | 直接验 Rust，无对拍；但被验的是抽出来的函数，`replay_journal` 调它的那几行（读盘、写报告）仍是普通 Rust | 推 | — |
| 装工具 | GitHub 发布的二进制 + 它钉的 rustc 版本（本机有 stable / nightly / 1.90.0，钉哪一版[未核]）；网络通 | 推 | — |
| 1–2 天小原型 | 对真的 `replay_journal` 不能：`BTreeMap`、闭包迭代（`.all(...)`）、`&dyn PoolReader`、`?` 都要先改写（支持度是别家文档说的，线索）。只能对**新写的纯函数**做 3 条记录 | 推 | — |

### 候选 ⑤　对照臂：不做形式化，层 0 全枚举 + 放量 + 对拍
| 项 | 内容 | 标记 |
|---|---|---|
| 另写行数 | 0 | — |
| 证到哪条 | 无「证」。M1 在枚举域内每个状态上恢复都跑过（判定住 `recover` 返回值与池级 checker）；M2 在枚举域内由记录核对器与池级 checker 的 I-8.6 / I-8.7 / I-8.8 / I-8.9 旁证，前缀判定本身**没有独立一侧**（facts 表 d2 读法）；M3 幂等 / 单调无判定 | 量（grep 见 0.3）+ 推 |
| 一次跑多久 | 放量七个分项 26.16 亿态一趟 7 小时 37 分（`./CLAUDE.md:8`「一趟 7 小时 37 分」，读的是主 agent 的记载，我没复跑） | 引 |
| 哈希 / 校验和 | 真 CRC；碰撞「知情接受」（D13 已定项 4） | — |
| spec 从哪来 | 记录核对器（`crates/singlefs-checker-tier/src/crash.rs`）与 oracle，各自另写 | — |
| 层 0 / 放量今天覆盖的维 | 载荷字节：固定脚本几个值；记录条数：一次发布最多两条（L8 流，`crash_enumeration_record_spill_over_stream.rs` 头注释）；环回绕：层 0 的环 768 MiB（`crates/singlefs-harness/src/segments.rs:262` 的 `journal_ring_bytes: 768 << 20,`），我找到的最小环是 128 MiB（`allocation_record_tree_root_level_of_writer_and_reader.rs:41`），没找到把计数器写过环长的用例（只 grep 了 `journal_ring_bytes` 的赋值，[推]）；残留：跨实例一形有用例；槽数与设备数：固定 | 量 + 推 |

### 候选 ⑥　零轮新候选：Rust 里的小界记录表穷举（零工具）
我提的，**没被攻过**，进岔路单时标「零轮」。做法：harness 档新增一条用例，直接对 `replay_journal`（`pub`，`recovery.rs:3057`）枚举记录表——N≤3 条、每条 97 个取值，就是 0.2 里 846 369 张表；每张表核「施加恰是整次发布」「jsn 连续」，合法历史子集核单调。
- 行数 [推] 120–200 Rust；不需要装任何东西；一次跑多久 [推] 秒到分钟（Python 译本 2.3 秒）；哈希：反向链需要读盘上前一条的字节，用真 `MemoryPool` 写真记录（`JournalRecord::to_bytes`，`journal.rs:206`），CRC 是真的。
- 证到哪条：**枚举，不是证**；比 ⑤ 多罩的：记录条数 / 发布大小 / 残留实例值的**任意组合**（层 0 只走固定脚本的组合）；罩不到：64 位取值宽度与 N 超界。
- 它是 ①③ 需要的差分胶水的一半（记录表喂 Rust 那一半），所以先做它不浪费。

### 边际价值：比 ⑤ 多罩了 D13 已定项 1 射程里的哪一维（五节第 2 条判据）[推]
| 维 | ⑤ 今天 | ① | ② | ③ | ④ | ⑥ |
|---|---|---|---|---|---|---|
| 载荷字节（字段取值宽度，txg / 计数器 / 事务号 64 位） | 固定脚本的几个值 | 符号 64 位，界内全罩 | 符号，界内全罩 | 只能小域（TLC）；Apalache 可整数 | 全罩，无界 | 小域，罩不到 64 位 |
| 记录条数 / 发布大小 | 一次发布 ≤ 2 条 | N≤8–16 | N≤3–4（不确定） | n≤12–16 | 无界 | N≤3 全表 |
| txg / 环回绕 | 不回绕 | txg 全罩；环槽回绕要建槽数组（副本里加 30–60 行） | 真代码的 `record_offset` 取模，符号环长即罩住 | 同 ① | 全罩 | 环长可调小，回绕靠调小环 |
| 上一条时间线残留 | 一形 | 实例字段符号，全罩 | 同 | 小域 | 全罩 | 实例取 2 值 |
| 槽数与设备数 | 固定 | 建数组才罩，成本高 | 真代码循环，S≤16、盘数小，展开可 | 建数组 | 需改写循环 | 不罩 |
**判据 2 的结果**：五条都说得出至少一维（载荷宽度、记录条数），没有一条出局；但「环回绕」这一维 ⑤ 自己也能罩（给层 0 配小环，`geometry.journal_ring_bytes` 是现成旋钮，[推]），所以形式化在这一维上的边际接近 0；载荷宽度这一维的实证价值也要打折——恢复读到的字段先过 `parse`，计数器只有 6 字节（`crates/singlefs-core/src/journal.rs:292`「let counter = reader.get_six_byte_unsigned();」），`checkpoint_txg + 1` 的溢出已由审阅第 36 条补了（`recovery.rs:3099-3106`），说明这一类界外取值 bug 今天靠读代码抓、不靠枚举。

## F2　spec 从哪来与同源性

### F2.1　条款怎么说，实现今天有什么
- 条款：「spec 写成「对镜像的状态谓词」，model 写成「op → 状态转移」，不一致时至少能互相定位。」（`.claude/kb/decisions/13-验证路线.md:285`）；「若累计 ≥ 5 次不一致而判定「spec 错」的次数为 0，说明两者实际同源，应把两条腿合并、预算移给穷举。」（`.claude/kb/decisions/13-验证路线.md:287`）。
- 条款对 spec 的另一道限：「D13（验证路线） 已定项 4 是 refinement 左边的 `Crash`（模型层的枚举域），不是右边的 `Spec`，拿它当 spec 会让 refinement 恒真。」（`.claude/kb/decisions/13-验证路线.md:35`）——spec 不许从层 0 枚举器出，枚举器只当 `Crash` 输入。
- 实现：模型 `model.rs` 不建崩溃，文件头「不建崩溃：第 1 件的历史里没有断电、没有设备错，每一步的写都落完，所以环里最新的那条根就是最后写出的那一条、journal 里没有要施加的记录」（`crates/singlefs-harness/src/model.rs:8`）；最近的一处是整次发布粒度的 `answer_mount_writable_with_the_roots_rebuilt_from_their_records`（`crates/singlefs-harness/src/model.rs:1309`）；崩溃后允许什么由 `crash_recovery_disagreement` 四条判（`crates/singlefs-harness/src/model.rs:2246`）。
- 读法 [推]：今天 M2 的 spec 与 model 这一对**没有可比的输入**：model 答「落到哪一版允许」，M2 spec 答「施加的记录序列是不是整次发布的拼接」，问题不同；C191 要的「不一致事件」要两边对同一个崩溃状态答同一个问题，即「这个崩溃状态允许恢复到哪些版本」。M2 spec 的语义形（F3 里的 M2 语义）恰好答这个问题，所以能与 `crash_recovery_disagreement` 在层 0 每个崩溃状态上逐个比。

### F2.2　每条候选的 spec 形态与独立程度 [推]
先分清四件东西：**S** = 从 kb 文字写的 spec；**C** = 从 Rust 代码翻的副本（只有 ① 与 ⑥ 有）；**R** = Rust 实现；**K / V / M** = 池级 checker（`walk.rs`）/ 记录核对器（`crash.rs`）/ 理想模型（`model.rs`）。
| 候选 | S 的形态 | S 与 R 的独立 | S 与 K / V 的独立 | S 与 M 能不能产生「不一致事件」 |
|---|---|---|---|---|
| ① | Python 谓词，作用在记录表与崩溃状态上；C 另写、旁注 Rust 行号，S 旁注 kb 行号 | 语言不同、文件不同；靠 S 与 C 不由同一遍阅读写出（见下「纪律」）| 不共享代码；K 判「环里全部自证记录」，S 判「恢复取到的前缀」，判的对象不同（facts d2） | 能：Python 可对具体状态求值 |
| ② | Rust 断言，住 harness 里 | **结构上最弱**：同语言、同 crate 工作区；放在 core 的测试模块里还能调私有函数（`record_ends_its_publish` 是私有的，`recovery.rs:2932`），共用了被测的判定 | 同 ① | 能：断言在 `cargo test` 下也是普通断言 |
| ③ | TLA+ 模块 = 按 D23 已定项 14 五条口径写出的算法，性质是它的谓词 | 语言与文件都不同；没有 C，所以「模块与代码的对应」只在对拍里 | 同 ① | **难**：TLC 对每个具体状态起一次 JVM 或把 26 万个状态写成常量批量喂，前者慢、后者是另一个工程 |
| ④ | Verus `spec fn`，与执行体在同一文件 | 弱：同文件、同作者、同一遍阅读 | 同 ① | **不能**：`spec fn` 是幽灵代码，不可执行 |
| ⑥ | Rust 谓词 | 同 ② | 同 ① | 能 |
**纪律 [推]**：不论选哪条，S 的每条谓词旁写 kb 行号（本报告的 F3 候选原句就是这个形态），C 的每个分支旁写 Rust 行号（`replay_model.py` 已是），两份放在不同文件、不同提交；S 里出现「代码里才有的名字」（`records_of_the_open_publish` 这类）就判同源。
**能提供不一致事件流的只有 ①②⑥**（求值可执行）；③④ 要另建取数通道才谈得上 C191 的计数——这是 F2 里区分候选的一格。

### F2.3　写 spec 这件事本身会撞出条款缺口 [量，译本上；推，Rust 上]
`replay_journal` 对**任意**自证记录表（不带历史约束）的行为，我在译本上穷举 N=3（846 369 张表）：
```
任意记录表 N=3: 表数=846369 句法违例=0 连续违例=0 施加序列 txg 非单调=80
  第一个非单调例子（instance, counter, txg, txn, commit, ordinal, last, chain）：
    (1, 1, 2, 1, True, 1, True, 'H')      ← 所选根（实例 1、txg 2）自己那次发布的末条，锚点
    (1, 2, 4, 1, True, 1, True, 'H')
    (1, 3, 3, 1, True, 1, True, 'H')
```
我按 Rust 逐行手走一遍（`recovery.rs:3109` 起的循环）：锚点在，`expected_next = (1, 2)`（`recovery.rs:3096`）；第 2 条 txg 4 施加，`effective_root` 落在 txg 4；第 3 条 jsn 接得上（`recovery.rs:3111`）、反向链对、`readable_record_of_the_same_publish_follows` 的 `take_while(txg ≤ 4)` 里没有 txg 4 的后继，施加，`effective_root` 退到 txg 3。**Rust 上没跑，是手走的 [推]**。
- 句法性质（整次发布的拼接、jsn 连续）在任意表上都成立（0 违例）——M2 句法不需要历史假设。
- 「施加后的根 txg 单调」不成立，实现靠一条**代码注释里的前提**：「同一实例里计数器与 checkpoint_txg 一起往上走」（`crates/singlefs-core/src/recovery.rs:2939`，续行 2940）。kb 里我读了 I-8.1–I-8.9 九行（`.claude/kb/invariants.md:84-92`），I-8.7 判事务号不减（90 行）、I-8.9 判序号（92 行），没有一行写「同实例内 checkpoint_txg 随计数器不减」，**[推] 这条假设没有 kb 落点也没有 checker 判定**。
- 合法历史下不受影响：我的合法历史枚举里 3^n 个状态 0 违例（0.2 ②）。所以它不是已知 bug，是 spec 必须显式写进「合法历史」的一条前提，也是 M2 语义与 M3 单调要用的；建议进 I-8.x 并让池级 checker 判（`walk.rs` 里已有 I-8.7 / I-8.9 的按实例判法可照抄形态）。
- **什么会推翻**：我手走漏了 `replay_journal` 里某个检查（例如别处比过相邻发布 txg）——只要在冻结副本上把这三条记录喂 `replay_journal` 一次就能分出，本轮不编译 Rust，交主 agent 或攻方复核。

## F3　M1–M3 逐条可达性

### F3.1　三条义务与实现今天对不上的地方（冲突格，两边整行并排）
| 格 | 条款 | 实现 |
|---|---|---|
| M1 的「输出」 | 「输出一个能过 I-7.1（根槽有效集非空） 与 I-8.3（重放前缀严格连续） 的镜像」（`.claude/kb/decisions/13-验证路线.md:31`） | 「pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {」（`crates/singlefs-core/src/recovery.rs:4222`）——交回的是报告，不是镜像；冷启动不写盘 |
| M1 的 I-7.1 | 「全部根槽位中，**自证校验和通过**的记录集合 S 非空」（`.claude/kb/invariants.md:52`） | 这是**输入镜像**的性质，池级 checker 在 `crates/singlefs-checker/src/walk.rs:5852` 判；`recover` 对它的响应是 `NoValidRoot`（`recovery.rs:4238-4248`，facts d1） |
| M3 幂等 | 「`Recover(Recover(c)) = Recover(c)`（I-8.4（重放幂等））」（`.claude/kb/decisions/13-验证路线.md:33`） | 同一行 `recover` 的签名：输入是 `&dyn PoolReader`、输出是报告，类型上不能复合；`Recover∘Recover` 在冷启动那条路上没有定义 |
| I-8.4 字面 | 「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」（`.claude/kb/invariants.md:87`） | `replay_journal` 不写盘：「let mut rebuilt = *root;」（`crates/singlefs-core/src/recovery.rs:3073`）、「Ok((report, rebuilt))」（`crates/singlefs-core/src/recovery.rs:3228`）；盘上状态会变的是可写挂载取号之后（facts d1，`mount.rs:3838` 起） |
| M2 | 「施加的单位是一次发布——合法前缀停在一次发布的两个事务之间时，那次发布整体不施加」（`.claude/kb/decisions/23-journal的角色与格式.md:361`） | 一致：末条没到就 `continue`（`crates/singlefs-core/src/recovery.rs:3200`「if !record_ends_its_publish(record) {」），施加只在末条之后（`crates/singlefs-core/src/recovery.rs:3203`「report.prefix_applied += records_of_the_open_publish.len();」） |

### F3.2　候选原句（M2 拆两层，M3 三种读法各写一句）[推；括号里是译本上的量]
- **M2 句法**：「对任意所选根与任意记录表，`replay_journal` 施加的记录序列是若干整次发布的拼接——每次发布内 `checkpoint_txg` 相同、发布内序号依次 1..k、只有第 k 条带末条标志；序列为空，或末条带末条标志。」依据：D23 已定项 14 第五条（`.claude/kb/decisions/23-journal的角色与格式.md:361`）。译本任意表 N≤3 共 846 369 张，违例 0。
- **M2 语义**（要合法历史）：「对发布路径写出的历史 p1..pm 与任一崩溃状态 c，施加的是 p1..pj 的全部记录，j = 从所选根覆盖的那次发布之后起、每次发布的记录都落盘且点名单元验得过的最长前缀；`effective_root` = pj 末条记录带的新根段。」译本合法历史 n≤9 条记录，违例 0，变异（任意一条都算末条）红。
- **M3a 只读幂等**：「对任意镜像 c 与固定的读者，`recover` 两次返回相同」。冷启动那条路由类型担保（`PoolReader` 没有写方法，`recovery.rs:81` 起；`replay_journal` 只改本地 `rebuilt`），只需假设读者确定；任何候选都不必为它付钱。
- **M3b 记录级幂等（I-8.4 字面）**：「对记录 r 与根 x，`apply(apply(x, r), r) = apply(x, r)`」。`apply` 是 `recovery.rs:3210-3226` 的字段覆盖：除 `filesystem_identifier`、`instance_table`、`allocation_record_tree_root` 照抄旧值、`unmount_marker` 取常量外，其余字段都取自记录 ⇒ 一步不动点，十行内可证；它挡不住 E78 那类「旧记录的点名块已被合法复用」（`.claude/kb/invariants.md:109`），那一类由水位挡，不在幂等里。
- **M3c 挂载读法**（`Recover` 输出镜像）：要把 `mount_writable` 取号之前的整段（facts a4 表：取号之前约 13 500 行、取号前预演 9 401 行）与取号写入放进边界；五条候选**都不可达**，只有 ⑤ 在崩溃注入里有随机覆盖（`crash_injection.rs`，D13 已定项 7 射程）。**这一格要用户或主 agent 先定读法**，否则七·4「`Recover(Recover(c)) = Recover(c)` 就是 I-8.4」按 M3b 交一个近乎平凡的证明，会被读成 M3c 已证。
- **M3 单调**（候选原句，条款「后半全仓无落点」，`.claude/kb/decisions/13-验证路线.md:33`）：「同一历史、同一段内两个崩溃状态 c ⊆ d（d 持久的写集合包含 c 的），`Recover(c)` 施加的记录集合 ⊆ `Recover(d)` 施加的记录集合，`effective_root` 的 `(实例代号, checkpoint_txg)` 不减。前提：历史合法（同实例内 `checkpoint_txg` 随计数器不减；发布内序号与末条标志照 I-8.9；事务号照 I-8.7；点名单元一旦持久字节不变，故校验结果随持久集合单调）。」
  - 根槽不在同一段：D16 已定项 7「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」（`.claude/kb/decisions/16-发布语义.md:179`），所以段内 c ⊆ d 时根环相同、择根相同 [推]。
  - 译本合法历史 n≤9 条记录、3^n 个状态、单步加一条或多验过一条，单调违例 0 [量，译本]。
  - **前提里「`checkpoint_txg` 随计数器不减」是 F2.3 里没有 kb 落点的那条**；不补它，单调只在合法历史上成立、任意表上不成立。

### F3.3　逐条可达性（对象 = 证的是什么）[推]
| 义务 | ① | ② | ③ | ④ | ⑤ | ⑥ |
|---|---|---|---|---|---|---|
| M1 有定义 + 不 panic（决策核） | 副本上有界 | **Rust 决策核，有界；Kani 默认查 panic 与溢出** | 不可达 | 纯函数上无界 | 枚举域内每状态跑过 | 枚举 |
| M1 终止 | 界 = 环槽数（副本参数） | 展开界即终止界，仅小环 | 有限模型 | `decreases` 无界 | 结构上有界（facts b 表） | 枚举 |
| M1 整条 `recover`（走树 3 121 行） | 否 | 否 | 否 | 否 | 是（枚举域内） | 否 |
| M2 句法 | 有界，副本 | 有界，Rust + 桩 | 有界，模块 | 无界，纯函数 | 旁证 | 枚举 |
| M2 语义（合法历史） | 有界，副本 + 写方 spec | 有界，`assume` 表达合法性（约束多，代价高） | 有界 | 无界 | 枚举域内 | 枚举合法历史 |
| M3a | 平凡 | 平凡 | 平凡 | 平凡 | 平凡 | 平凡 |
| M3b | 平凡引理 | 平凡引理 | 平凡引理 | 平凡引理 | — | 平凡 |
| M3c | 否 | 否 | 否 | 否 | 崩溃注入随机覆盖 | 否 |
| M3 单调 | 有界，两份记录表 | 有界，两份符号表，代价翻倍 | 有界 | 无界 | 无判定 | 枚举 |
**「证到了」的高估在哪 [推]**：① ③ 证的是副本 / 模块，Rust 一侧只有差分；差分只在喂进去的记录表上成立，未喂的分支（例如反向链重读时槽已被后来记录覆盖的 `PreviousRecordNotOnDisk`，`recovery.rs:3016`、`3019`）副本与 Rust 可能分歧而无人知。② 把 `parse` 桩成「返回幽灵记录」后，证的是「若 parse 是写出方的逆则 M2」；这条假设今天靠层 0 的真镜像与 `journal.rs` 的解析单测盖，不在证明里。④ 抽出来的纯函数与调用它的那几行读盘代码之间也有一道缝。

## F4　门禁与 timeout（C5）

### F4.1　跑一次的挂钟量级 [推，除标 [量]；工具都没装，没有一个本机数]
| 候选 | 每条义务 | 整张义务表 | 依据 |
|---|---|---|---|
| ① Z3 | 秒到分钟 | 十分钟内（N≤8） | 位宽 245×N（F1 ①）；无量词的比较与 `ite`；M3 单调两份表约 2 倍 |
| ② Kani | 分钟到小时 | 三个 harness × 若干界，小时量级 | 符号 `BTreeMap`、64 位除法；别家 CI 时长无一个是存储代码（线索） |
| ③ TLC | 秒到分钟 | n≤12 个记录约 53 万状态 | 3^n [量，译本]；TLC 状态吞吐没测 |
| ④ Verus | 秒 | 秒到分钟 | 别家 12–54 秒（线索）；人天在写证明 |
| ⑤ | — | 放量一趟 7 小时 37 分（`CLAUDE.md:8`，引） | — |
| ⑥ | 秒 | 秒 | 译本 2.3 秒 [量，Python]；Rust 更快 [推] |
落哪一档 [推]：①②③④ 单条不到 60 秒的部分可进 harness 档，整表按 D13 已定项 15 归 checker 档、命令带 `SINGLEFS_HEAVY_TESTS`（七·5 的写法，`.claude/kb/milestone/03-third-txn.md:166`）；⑥ 是 harness 档一个 `cargo test` 目标。

### F4.2　三种结局在各工具里怎么区分 [别家文档说的是线索，我没读各工具原文，标 [未核]]
| 工具 | 证明 | 反例 | timeout |
|---|---|---|---|
| Z3（① ）| 对「性质的否定」返回 `unsat` | `sat` + 模型 | `unknown`，`reason-unknown` 里写超时 [未核]；**这一条就是 C5**：`unknown` 被当成通过 |
| Kani（②）| `VERIFICATION:- SUCCESSFUL` [未核] | `FAILED` + 反例轨迹；界不够时是 `unwinding assertion` 失败（线索）| 没有内建总超时 [未核]，只有外层 `timeout` 的 124 |
| TLC（③）| `Model checking completed. No error has been found.` [未核] | `Invariant … is violated` + 轨迹 | 没有内建总超时 [未核]；被杀时**没有任何结论行** |
| Verus（④）| `verification results:: N verified, 0 errors` [未核] | `errors` 非 0 | 资源上限超出报成失败 [未核] |
### F4.3　timeout 判红怎么落（设计 [推]，七·5 的验收「timeout 判红（C5（Z3 超时当通过） 还清）」，`.claude/kb/milestone/03-third-txn.md:166`）
1. 现状：C5 写明「门禁的 crash refinement 阶段必须有**三种**结果（证明 / 反例 / timeout），timeout 一律判红」（`.claude/kb/checks-owed.md:19`）。
2. 阶段脚本对每条义务输出一行 `obligation=<名> result=proved|counterexample|timeout|error time_s=<秒>`，末行 `obligations=<登记数> proved=<数>`；退出码：登记数与 `proved` 相等才 0，否则 1。**「没看到成功行」判红**，不用「没看到失败行」判绿——TLC 与 Kani 被外层杀时正是没有任何行。
3. 退出码 77 只给「本次未跑」，不许给 timeout：「退出码 77 是「本次未跑」，不是通过。」（`.claude/agent-common.md:88`）。
4. 登记数钉进 `stage-inputs.tsv`（七·5 已写「输入登记进 `.claude/gate.d/stage-inputs.tsv` 按输入复用」），少一项判红，防「证了 0 项」。
5. 判别力样本四份：一份故意超时（深界）判红；一份带反例判红；一份证明过判绿；**一份假设恒假（空真）判红**——伴随一条「假设集可满足」的 `sat` 查询。第四份七·5 没写，是我加的。
6. 外层 `timeout` 杀掉后脚本 `trap` 记 `result=timeout`。**什么会推翻这一设计**：某个工具的超时在它自己的结果码里已单列（例如 Verus 的资源超限直接报 error），则那一行不需要外层 trap。

## F5　O3 进不进这个里程碑

- 条款：「**用另一种语言写**（不用 Rust），只照 decisions 与 invariants 的文字重新推导」，并且「规约本身可能写错——**必须配阳性对照才有意义**」（同一行，`.claude/kb/decisions/13-验证路线.md:113`）；O3 判「崩溃点截断后的每个磁盘状态是否落在规约允许的集合里」（同行）。实现：零代码，`grep -rn "RefFs\|RefFS" crates | wc -l` → 0，`grep -rn 合谋 crates | wc -l` → 0（0.3）；欠账：C447（`.claude/kb/checks-owed.md:370`）、C407（`.claude/kb/checks-owed.md:334`）。
- **①③ 的 spec 能不能当 O3 [推]**：
  - ① 的 S（Python 谓词，从文字写）满足「另一种语言」「只照文字」两条；但 O3 要对**层 0 每个崩溃状态**求值，输入里的记录字段要靠 Rust 的 `parse` 抽出，等于共享了格式解析——D13 已定项 12 认这一处共享，O3 的「只照文字推导」在解析这一层因此不成立。
  - ③ 的 TLA+ 模块满足语言与文字，但不能批量对具体状态求值（F2.2），当不了「对每个崩溃状态判」的 oracle。
  - **同源性代价**：S 既做 refinement 的右边（Spec）又做 O3，C191 的三对里 O3 × RefFS 与 spec × O1 两对并成一份 spec，两条腿共用一次对 D16 已定项 4 的读法；一次误读会同时穿过两处，而三 oracle 合谋测试（C407）零代码、这类共同误读没有检查。独立腿数 3 → 2 [推]，换来少写一份 spec（100–200 行 [推]）。
  - 阳性对照：七·2 / 七·3 各自写了必出反例的世界（`.claude/kb/milestone/03-third-txn.md:163`、`:164`），译本上「末条标志改成任意一条」已红（0.2 ④）；O3 的阳性对照要另建，不由这两条替代。
- **判定**：O3 **不进**这个里程碑的七·2–七·5，留在七·6（原文「做不做、做到哪，按七·1 的代价数由用户定」，`.claude/kb/milestone/03-third-txn.md:167`）；不因为 ①③ 有 spec 就把它顺手升级成 O3。只做一件零成本的预留：S 写成**可对具体崩溃状态求值**的形态（① 或 ⑥ 的 Python / Rust 谓词），这是 C191 计数本来就要的（F2.2），将来做 O3 时可复用其中的允许集合函数，但 O3 的独立实现仍要另写。

## 推荐与第一步原型 [推]

**推荐：⑥ → ①**——先做零工具的 Rust 小界穷举（⑥，也是 ① 差分的一半），再把译本换成 Z3 符号执行（①）；② 只在下面「翻面观测」成立时替换 ①；③ ④ 现在不做。
理由（都是推的，凭据在前文格里）：
1. 五条都够得着 M2（出口条件通过），但「证的是什么」不同：① 证副本、② 证 Rust + 桩、③ 证模块、④ 证抽出的纯函数（F3.3）。① 的结构最能满足 D13 已定项 13 的「表述层面就不同」与 C191 的事件流（F2.2），代价中等（480–740 行，其中 Rust 150–250）。
2. ③ 的「与代码绑定」退化成 ① 同一份记录表差分，而 spec 不能求值（F2.2），多付一门语言与 Java 的钱、少一个 C191 通道；④ 要改生产代码 80–150 行、行数区间宽到 5 倍（800–1 500），且对真 `replay_journal` 一天原型不可达。
3. ② 的优势是无副本漂移、默认查 panic 与溢出（M1「有定义」直接吃得到）；劣势是 spec 结构上不独立、桩后留一条「parse 是写出方逆」的假设、`BTreeMap` 与 `dyn PoolReader` 的可行性没量。

**第一步原型（一到两天，只证 M2 句法，3 条记录、1 块盘、S=4）**：
- P1 已有：`replay_model.py`，显式枚举，N=3 全部 846 369 张表 2.3 秒（S 与盘数只影响择根 / 扫环，`replay_journal` 这一截与它们无关）。
- P2（半天，Rust）：harness 档一条用例，对 `replay_journal` 枚举同样的 846 369 张表，写 jsonl（表 → `prefix_applied`、`effective_root`）；Python 读它与译本逐表比。**第一次量「副本与 Rust 差分能不能对上」**。对不上就两边都查，不默认哪边对（`.claude/rules/implementation-first.md` 第 4 条）。
- P3（半天到一天，要装 z3）：译本的 `replay` 改成 Z3 符号，字段取 64 位，N=3，证 M2 句法；记三种结局与挂钟。这一步才有 ① 的**本机挂钟**。
- **翻面观测**：P3 在 N=3 上超过 60 秒、或 N=6 超过 30 分钟 ⇒ ① 的「一次跑多久」记不稳定；换 ② 的条件：`cargo kani` 对 M2 句法 harness（N=3、桩 ≤ 4 个）在 5 分钟内出结论 ⇒ ② 比 ① 便宜且证到 Rust 本体，改选 ②。
- **什么会推翻整条推荐**：P2 差分在 846 369 张表上有一处对不上且原因是译本无法表达的 Rust 行为（例如泛型 / trait 分派）；或 F2.3 那条 txg 非单调在 Rust 上手走错（然后合法历史前提就不必补进 kb，① 的 spec 更省一条）。

## 计数实验清单（推的格 → 主 agent 写岔路单时要建的实验；形态照 E109 / E110：纯算术、钉绝对值、带变异）
| 实验 | 量什么 | 钉的绝对值（来自第 0 节，[量]） | 变异（必须让它变红） |
|---|---|---|---|
| E-A 边界行数 | 每条候选「进副本 / 保留原样 / 桩掉 / 删」的划分表下，要另写或要改的行数：输入逐函数行表（`fns.tsv`）+ 翻译系数区间（译本一个样本 0.26，区间取 0.2–0.5） | 决策核 10 fn / 287 行 / 242 码行；择根簇 82；`scan_journal` 38；闭包 K 36 fn / 577 行 | 把 `judge_back_chain_against_the_previous_record_on_disk`（33 行）移出 / 移入边界，总数差 33 |
| E-B 符号输入位宽与出口数 | 每条记录进判定的字段位宽（从 `JournalRecord::parse` 的字节布局与类型抄）与未合并路径数上界 paths(N) = 11·2^N − 10 | 出口 10 个 `break` + `continue` + 施加；N=1 → 12；译本 N=2 时 10 个出口全命中（`exits.out`） | 删一个 `break` 分支，出口 9、paths(1) = 11 |
| E-C 合法历史状态规模 | 合法历史 n 条记录的落盘状态数，对照层 0 L8 流（25 个状态）是这个空间里的哪个点 | states(n) = 3^n；n=9 → 19 683 | 把「没落 / 落了验不过 / 落了验得过」改成两态，n=9 → 512 |
| E-D 桩掉 CRC 之后还剩什么 | `replay_journal` 里依赖 CRC 判出结果的出口数与经 `parse` 重读的调用点数 | 出口 2 / 10；重读点 1（`recovery.rs:3028`） | 把 CRC 桩成恒真，出口 2 个不可达，M2 失去这两个反例来源 |
| E-E 挂钟 | 不是计数实验，是要装了工具才有的量：① 在 N ∈ {2,3,4,6,8}、64 位字段上的 Z3 时间，② 在 N ∈ {2,3,4} 上的 Kani 时间，各三次 | 无（本机没有任何工具，没有绝对值可钉） | 故意超时的样本判红（F4.3 第 5 条） |
**已知不稳定区间（攻方反算大概率差 10 倍以上，先标）**：② 的「一次跑多久」（分钟到小时）、④ 的证明行数（260–2 000）、① 在 N≥8 的求解时间。这三格建议直接进岔路单的「要量」一行，不等反算。

## 判定一览
| 格 | 判定 | 一句话 |
|---|---|---|
| F1 五条候选代价表 | 一致 | 五条都写得出六项代价；行数、挂钟全是推的，仅决策核行数与译本行数 / 穷举规模是量；另提零轮候选 ⑥ |
| F1 装工具与 1–2 天原型 | 规则没说 | 本机无 sudo / pip / java，网络通；能装没试；① ⑥ 的原型本轮已用显式枚举做出，③ 视 Java，④ 对真函数不可达 |
| F1 边际价值（判据 2） | 一致 | 五条都说得出比 ⑤ 多罩的维（载荷宽度、记录条数）；环回绕一维 ⑤ 自己可罩（小环），边际接近 0 |
| F2 spec 来源 | 冲突（一处） | 「只照 kb 手写」与代码注释里的前提「计数器与 checkpoint_txg 一起往上走」（`crates/singlefs-core/src/recovery.rs:2939`）：kb 无此条，译本上任意表 N=3 有 80 张 txg 非单调 |
| F2 同源性观测 | 规则没说 | C191 的事件要两边可求值；① ② ⑥ 可，③ ④ 不可；今天 M2 spec × 模型这一对没有可比输入（`model.rs:8`） |
| F3 M1 | 冲突 | 条款「输出一个……镜像」对 `recover` 返回 `RecoveryReport`（`recovery.rs:4222`）；I-7.1 是输入镜像的性质 |
| F3 M2 | 一致 | 施加只在末条之后（`recovery.rs:3200`、`3203`）；句法性质任意表成立，语义要合法历史 |
| F3 M3 幂等 | 冲突 | `Recover(Recover(c))` 对只读签名没有定义；I-8.4 字面在 `replay_journal` 上是十行引理；M3c 五条都不可达，要先定读法 |
| F3 M3 单调 | 规则没说 | 「后半全仓无落点」；候选原句已写；前提含一条 kb 里没有的「txg 随计数器不减」 |
| F4 挂钟与 timeout | 规则没说 | 各工具的超时结果码我没读原文（[未核]）；判红设计写了五条，第 5 条（空真样本）七·5 没有 |
| F5 O3 | 规则没说 | 判：不进七·2–七·5，留七·6；S=O3 的同源性代价（独立腿 3 → 2）条款没写 |

## 没做什么
- 没装任何工具、没编译 Rust、没跑重型测试与门禁；`replay_journal` 的行为只在我写的 Python 译本上量、在 Rust 上手走（F2.3 那条 txg 非单调**没在 Rust 上跑**）。
- 译本是一个人一次翻译，抽象了反向链重读（三态布尔）与点名校验（布尔），没有 `choose_root`、扫环、多盘、环槽数组；「N=2 全部出口命中」「3^n」只对译本成立。
- Z3、Kani、TLC、Verus 的挂钟一个本机数都没有；F4.2 各工具结果码写的是我记忆里的形态，没读原文，全标 [未核]；`BTreeMap` 节点容量之类 std 常量没核（本机没装 rust-src）。
- 事实表的闭包行数我只对 `closure_K.tsv` 重新求和（577 / 527 一致）并拆出决策部分，没用编译器级调用图重算；`crash_injection.rs` 的覆盖强度、层 0 各流的记录条数上限我只读了 L8 流头注释与环长赋值，没逐流量。
- 没读攻方腿这一轮任何东西；没采纳、没判决；攻方格 F1 / F3 的高估低估交主 agent 与攻方。
- 别家的数（Verus 证明对代码比、Kani CI 时长、MongoDB 10 周放弃等）只作线索，每处已标；引它们的差异一句：别家案例的被验对象多是持久内存或算术小状态，本工程是块设备上的 journal 前缀判定（决策核 242 码行），规模更小、不带并发。
