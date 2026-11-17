# gate-key-closure-narrowing-r1：判决（2026-11-14）

正文 `research/prompts/_gate-key-closure-narrowing-r1-body.md`，开工快照 `research/prompts/gate-key-closure-narrowing-r1-snapshot/`。这一题的第一轮，之前没有判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/gate-key-closure-narrowing-r1-sonnet-output.md` | 第 1 条照字面站不住、集合要取 `rust_lex_cached` 的整个闭包；第 2、3、6 条推不出反例、各要补条件；第 4 条出键成立、另有 python 一条路把登记文件带进闭包；第 5 条名单的极性要取反；第 7 条 `debug`、`incremental` 推不出反例、`codegen-units` 不建议出键；第 8 条「只留数」会被三个读者拒 |
| 攻方（Opus） | `research/prompts/gate-key-closure-narrowing-r1-opus-output.md` | 第 1、5、8 条条件式构造得出反例；第 6 条照字面打中、方向收错；第 2 条今天推不出、病根在认读文档串的认法；第 3、4 条推不出；第 7 条推不出确定性反例 |

核查命令（原样）：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/gate-key-closure-r1-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py gate-key-closure-narrowing-r1 --out /home/fy5090/.local/share/singlefs/work/gate-key-closure-r1-main/gate-key-closure-narrowing-r1-tree --repository /home/fy5090/code/singlefs`（任务 16884，退 0）：「清单点名 11 份，倒推出 11 份，sha256 对上 11 份」，其中 10 份取工作区此刻那一份（sha256 与快照相同），1 份取 HEAD。
- `python3 scripts/agents/cite-check.py research/prompts/gate-key-closure-narrowing-r1-opus-output.md research/prompts/gate-key-closure-narrowing-r1-sonnet-output.md --root <上面那棵树> --background research/prompts/_gate-key-closure-narrowing-r1-background.md`：「核了 31 处引文、89 处裸行号，对上 25 处，没判 95 处（其中裸行号没有引文可比 71 处）」。没有判成对不上的。

它没判的裸行号里，判决要用的几处主 agent 逐处读了源文件：

| 腿的说法 | 读到的 | 判 |
|---|---|---|
| 复用只读记录的 `verdict` 字段 | `scripts/mutation/crates-mutation-rows.py:3001-3002`：`record.get("verdict") == REUSABLE_VERDICT["mutation"]` 才复用 | 属实 |
| 写这个字段、定盖不盖旧结论的是写记录那一段 | 同文件 5402-5412：`keeps_counted`、`red-not-recorded` 的条件与 `write_record_quietly("records", …)` | 属实 |
| 词法缓存的内容还取决于词种名常量、格式号与读写两个函数 | `scripts/gate/lib_function_units.py:1628`（`IDENT, LITERAL, LIFETIME, PUNCT`）、1806（`RUST_LEX_CACHE_FORM`）、2047（`load_rust_lex`）、2068（`store_rust_lex`）、2101（`rust_lex_cached`） | 属实 |
| `#!` 行由 `kept_shell_comments` 另收 | 同文件 10553：`kept = [rows[0]] if rows and rows[0].startswith("#!") else []` | 属实 |
| 登记文件的读法要求理由非空 | `crates/singlefs-verification/src/heavy_threads.rs:72-73`：`strip_prefix('#')?` 之后理由为空就不认 | 属实 |
| 变异门禁有一条开关断言构建档取值进行键 | `scripts/mutation/crates-mutation-rows.py:1010`、1411-1412（`debuginfo-unkeyed`） | 属实 |
| 登记文件另有 python 一条路进闭包 | 同文件 `with_gpu_test_threads` 的文档串点名 `scripts/hosts/heavy_threads.py` 的 `default_heavy_threads` | 调用在；「因此进了判法闭包」没量，当推的 |

两条腿都没建模型、没在装置上量过；它们的结论全是读代码推的。正推腿报告里要主 agent 跑的四条 `closure … --explain` 与攻方腿的一条探针都没跑：判决不引它们的读数。

## 二、判决

逐条。两条腿在同一条上结论相同的写「一致」。

1. **算键库不读自己整份文件**：正文的写法（目录名只取 `rust_lex` 与帮手的指纹）出局，两条腿一致，记一次输。收严后的形态站得住：目录名取 `rust_lex_cached` 整个闭包的单元指纹（含读写缓存的两个函数、格式号、词种名常量）加弄坏开关。
2. **写 `__doc__` 不算读**：站住，补两处。先读后写（`AugAssign`）照算读；认读文档串的认法补上 `getattr(<对象>, "__doc__")` 与 `inspect.getdoc(<对象>)`，不然去掉第 1469 行那条写带来的整批文档串之后，这两种读法的文档串一段都收不到。
3. **自证夹具搬出被判法载入的模块**：站住。按「谁读」分：只被 `--selftest` 与弄坏开关读的才搬，运行期函数也读的不搬。
4. **判法指纹里去掉被判树的数据文件**：站住，范围照正文（算键库给自己编程序算缓存键时读的那三样）。同一份文件在别的读点上照旧按那一处的读法收，不按路径一起排。
5. **`kept-comments` 只收登记词**：正文的写法（允许名单）出局，两条腿一致：漏登就是少收。这一轮不做。
6. **记账代码出判法闭包**：正文的写法出局，攻方打中、主 agent 读代码坐实：写记录那一段定「红的记不记、盖不盖旧的抓到」，搬出闭包后改它就是键不变而复用结局变。收严后的形态：只搬不碰判定记录的那几样（用时表、量编译目录占盘、心跳与进度行），写 `records/<行键>` 的那一段连同 `keeps_counted` 的判法留在闭包里。
7. **编译档设置移出键**：`debug`、`incremental` 移出站住，两条腿一致。`codegen-units` 留在键里：两条腿都指出它能改栈深与代码分区，本仓有栈深敏感的用例，余量没量。
8. **登记文件只把数编进二进制**：两种变体都出局。「只留数」被读法拒；「只取第一条非注释行」在两份 Rust 读法切行不同的地方构造得出反例。这一轮不做。

D13（验证路线） 已定项 17 定案第四条「crates 变异判法除外」那一句：照第 1、3、4、6 条收严后的形态要改写，改写的句子取正推腿报告第 123-125 行那一段，去掉其中「写记录」三个字（第 6 条收严后写记录留在闭包里）。

## 三、改法

都是腿提的或主 agent 照两条腿的结论收严的，被攻过零轮；它们改的是门禁脚本与键算法，不是产品代码。每一条先造该红的输入再改。

| # | 改哪 | 做什么 | 该红的输入（自证格） | 状态 |
|---|---|---|---|---|
| 一 | `scripts/gate/lib_function_units.py` 的 `rust_lex_cache_namespace` | 目录名取 `rust_lex_cached` 闭包的单元指纹，不 `open(__file__)`；现有断言「改判法文件一行注释键变」的那一格照新口径改写 | 改 `RUST_LEX_CACHE_FORM` 或一个词种名常量，目录名不变就红；改这份库的一行注释，crates 变异判法指纹变了就红 | 零轮 |
| 二 | 同文件认读文档串那一段（9086-9098 一带） | 只在读（`Load`、`AugAssign`）上记；补 `getattr(…, "__doc__")`、`inspect.getdoc(…)` | 一个单元经这两种写法读另一个函数的文档串，改那段文档串键不变就红；模块级写 `X.__doc__ = …` 把整批文档串带进闭包就红 | 零轮 |
| 三 | 算键库、`scripts/mutation/crates-mutation-rows.py`、`scripts/mutation/broad-mutation.py` | 只被自证与弄坏开关读的模块级常量搬到只在 `--selftest` 里载入的文件 | 改一份夹具文本，判法指纹变了就红；运行期函数读到的常量被搬走（判法入口闭包里出现读不到的名字）就红 | 零轮 |
| 四 | 算键库编 unit-graph 程序算缓存键的那一段 | 经不进闭包的引用取那三样输入 | 改 `Cargo.lock` 里判法没点名的一个包，crates 变异判法指纹变了就红 | 零轮 |
| 五 | `scripts/mutation/crates-mutation-rows.py` 的 `judge_row` 一带 | 用时表、量占盘、心跳搬到不进闭包的外壳；写判定记录的留下 | 改写记录的条件（开 `red-not-recorded`）判法指纹不变就红；改用时表的写法判法指纹变了就红 | 零轮 |
| 六 | `crates/singlefs-unit-graph/src/build_toolchain.rs` 的 `PROFILE_SETTING_KEYS`、`scripts/gate/admission.py` 与 `scripts/mutation/lib_mutation_units.py` 摊开 profile 的各处 | 去掉 `debug`、`incremental`，`codegen-units` 留；python 一侧按整个键名比，不按前缀（`debug-assertions` 照留）；`debuginfo-unkeyed` 那一格与开关照新口径改写；Rust 与 python 对拍格同步 | 改 `debug` 或 `incremental` 键变了就红；改 `debug-assertions`、`opt-level`、`codegen-units` 键不变就红 | 零轮 |
| 七 | 仓根 `Cargo.toml` 的 `[profile.release]`、门禁起 cargo 的环境 | `debug = "line-tables-only"`；起 cargo 时设 `CARGO_PROFILE_RELEASE_INCREMENTAL=true` | 归第六条的格 | 零轮 |

改六条让全部判定的键变一次，照 D13（验证路线） 已定项 17 定案第七条不算代价；一到七一次改完、只作废一次。

## 交用户的

| 事 | 状态 |
|---|---|
| 第 5 条（`kept-comments`）这一轮不做：要先逐词核谁在读，再定登记「没人读的词」那一种形态 | 零轮，没排期 |
| 第 8 条（登记文件）这一轮不做 | 两种变体都出局 |
| `codegen-units` 留在键里；要移出先量栈深余量 | 没量 |
| 正推腿点名的同类宽：算键库 `<module-body>` 一个单元占一万七千行、整模块载入只用一个函数的五份、驱动里 143 个非自证的模块级赋值、13 份 `Cargo.toml` 整份字节进判法指纹 | 腿的线索，没核，没排期 |
| D13（验证路线） 已定项 17 定案第四条那一句要改写 | 等改法落地后写回 |
| 改法一到七都被攻过零轮 | 靠各自的自证格与弄坏开关立住 |

## 这一轮被判过的文件

- scripts/gate/lib_function_units.py：词法缓存目录名取什么、认读文档串的认法、整份原文单元、算缓存键时读的三样输入、`kept-comments` 的收法各自收窄之后键够不够定结局
- scripts/mutation/crates-mutation-rows.py：复用读记录的哪个字段、写记录那一段留在判法闭包里、记账里哪几样可以搬出去、构建档取值进行键的那条开关
- scripts/mutation/broad-mutation.py：只给自证用的模块级常量搬出去对它自己的判法闭包成立
- scripts/mutation/lib_mutation_units.py：python 一侧摊开 profile 时去掉哪两项
- scripts/gate/admission.py：`[profile.<名>]` 摊开进键的各处去掉 `debug`、`incremental`，`codegen-units` 留
- scripts/crash/lib_verdict_library.py：`NOT_JUDGING_FILES` 的文件级排除照旧，改写后的条款对它怎么说
- crates/singlefs-unit-graph/src/build_toolchain.rs：`PROFILE_SETTING_KEYS` 八项里移出两项、留六项
- crates/singlefs-verification/src/heavy_threads.rs：登记文件的读法要求理由非空，「只留数」与「只取第一条非注释行」两种变体都出局
- Cargo.toml：`[profile.release]` 的 `debug` 改成只留行号表之后键与结局的关系
- .claude/heavy-thread-default：整份被 `include_str!` 编进二进制这件事这一轮维持原样

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 备料 | 2026-11-14 改了：定案第四条「crates 变异判法除外」那一句等改法一到五落地后改写，定案第一条的「工具链（含 cargo 解析后的生效构建环境…）」等改法六落地后写明不含 `debug`、`incremental` |
