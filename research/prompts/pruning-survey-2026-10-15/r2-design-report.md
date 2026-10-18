# 第二轮复查 design 路（可行性）：函数单元切分与调用图，合成一份放哪

日期：2026-10-15。只读：主仓一个文件没改；探针（python）只跑在草稿目录的仓副本与自建夹具上，没起 cargo、没起测试二进制、没跑 Verus 验证（只跑了 `verus --help` 与对二进制的 `strings`）。
探针源码、夹具与原样输出都在 `~/.local/share/singlefs/work/survey-pruning-r2-design/probe/`（没入库：本路派发是只读，主 agent 定要不要拷进 `research/`）。

## 结论一览

| 问 | 答（量的 / 推的） |
|---|---|
| ① 三份现有实现到哪一步 | 都到不了函数单元的调用图。`judging_closure.rs` 是文件级、顺 `use` 与路径收文件；`mutation-prune.py` 的 FileIndex 能切出 fn 项与 impl 方法，但没有调用图，函数外部分、宏依赖按整文件，认不出的测试退整包；`lib_source_digest.py` 只有文件级摘要；`admission.py:2716` 的 python 闭包只看一个模块、只认 `ast.Name`、单元原文带注释。逐形态见第一节（量的，行号逐条现查） |
| ② 统一设计可不可行 | 可行（量的，探针）：全仓 405 份 `.rs` 切出 24 966 个单元，加三个入口的闭包共 2.86 秒；V5、V11 两个入口与 CPU 判器入口都抓到，拆掉 const 单元两个都漏；13 格形态夹具全对，四条规则逐条拆掉各自红在该红的格；CPU 判器闭包 2 705 个单元、49 份文件，流水线七份一个单元都不在里面（不靠手写停止表）。调用图的解法必须是「名字 + 编译目标 + RTA 收窄方法 + `dyn` 与开放入口处 CHA」，纯裸名过近似把格式库一条单测拉到 10 627 个单元（量的）。放法：Rust 与 wgsl 一份 std-only crate（build.rs 与运行期共用），python 一份（python、shell 两种语言 + Rust 的独立第二实现），快档对拍 |
| ③ Verus 能不能按函数拆项 | 文法上能，语义没量。`--verify-function` 要配 `--verify-root` 或单个 `--verify-only-module`、按唯一子串匹配（量的，二进制里的报错串）；核 crate 只有根模块（`mod` 只有测试模块，量的），按模块拆就等于整 crate；90 个 fn 里 4 个名字是别的 fn 名字的子串（量的）。`--no-cheating` 在只验一个函数时查不查整 crate、各函数的 verified 加起来等不等于登记的 56，派发不许跑 Verus，没量，命令写在第九节 |
| ④ QEMU、LKMM | QEMU：内核取镜像字节 sha256；initramfs 不取 `cpio.gz` 的哈希（每趟新建的文件 mtime 进 cpio 头，只 `touch` 一下两份 cpio 就不同，量的；gzip 从管道压不写时间，不是变数），取打包前的清单（路径、权限、内容 sha256）；里面的 bench 二进制按它 `main` 的单元闭包 + 构建配置记，二进制哈希只当属性。LKMM：锚点文法已经细到 fn（`lkmm.sh:277`），但写不了 impl 方法、只判「有一处」不判唯一；herd7 的判定根本不读代码，锚点单元指纹不该进 herd7 那一条的键，该进绑定测试的键 |

推翻条件（总的）：在本仓里找到一处真改了行为的改动，新切法（第三节规则）对它所在入口算出的键不变——夹具或副本里造得出这种改动，第三节的规则就要补，探针结论作废那一格。

## 一、问①：三份现有实现逐形态怎么处理（量的：行号现取自主仓当前文件）

`judging_closure.rs` = `crates/singlefs-checker-tier/src/judging_closure.rs`；FileIndex = `scripts/mutation/mutation-prune.py`；摘要库 = `scripts/gate/lib_source_digest.py`。

| 形态 | judging_closure.rs | mutation-prune.py FileIndex / TreeIndex | lib_source_digest.py |
|---|---|---|---|
| 单元粒度 | 文件。摘要每份文件一段整份词法串（`:1022` 起 `text.push_str(file)`，`:1081` `"file {file} {} {}\n"` 每行带路径） | fn 项（含嵌套 fn、impl / trait 里的方法，`:366`、`:390`）；函数外部分合成一个 `outside_fingerprint`（`:285`），进每条变异的指纹（`:613`）；括号配不上退整文件（`:604`） | 文件（`digests_of` `:532`，逐份一个摘要） |
| 去注释 | 去行注释、块注释与全部文档注释（`:18-21` 文档串与切法） | 去普通注释，文档注释留着当 `doc` 单元（`:333`；文件头说它带着文档测试） | `.rs` 交上游 `comment-only-change.py`，另记 doctest 代码行与 SAFETY 正文（`:244` `rust_analysis`）；`.wgsl` 归「其余」取原文（`classify` `:152`，`:162` `return "raw"`） |
| trait 动态分派、泛型 | 同 crate 里 `impl` 头 `for` 之后点名了闭包已声明类型的文件整份收进来（`:601` 取 `for` 之后的名字；`:936` 只扫闭包已触到的 crate；`:947` 跳过 `src/bin/`）；给闭包里的 trait 写 impl、而自身类型不在闭包里的文件不收：落在别的 crate 或类型没声明在闭包里的实现漏 | 没有调用图：一条变异只看包住它的 fn 项与本文件函数外部分，被调函数改了、抓它的测试经别的函数调到它，键都不变（`carry_decision` `:640` 只比本函数、依赖与抓它那条测试自己的指纹） | 不涉及（文件级） |
| 宏 | 不认 `macro_rules!`；宏调用只有写成路径（`a::b!`）时顺路径落到文件 | `macro_rules!` 记名字（`:407`）；调到的宏按名字取定义文件的整份指纹（`:519`） | 不涉及 |
| `include!` | 不认（`:498` 只认 `include_str` / `include_bytes`） | 认，读的文件取 sha256 | `include!` 字面路径记进来；被读的 `.md` 才进指纹 |
| `include_str!` 字面路径 | 收、按原文（`:498-505`） | 收、按原文 sha256 | 收、按原文 |
| `include_str!(concat!(…))` | 静默跳过：`:501` 只取第 4 个单元是串字面量的，`concat` 不是，读的文件不进闭包 | 参数不是单个字面量的取整棵树指纹（`:523` `include-unresolved`） | `concat!(env!("OUT_DIR"), …)` 当编译产物跳过（`:238`）；别的认不出时写它那份文件所在 crate 整个取原文（`:239` 起） |
| 泛型与关联常量 | 文件级，常量跟着所在文件整份进；常量在没被闭包收的文件里就漏 | const、static 在函数外部分里：改一个常量让本文件每条变异重判（过宽），别的文件里用它的函数不重判（漏）——E164 F4 的 V5、V11 正是这种 | 文件级 |
| `impl` 块方法 | 随文件 | 限定名带 impl 头（`<impl …>`），单元级 | 随文件 |
| `#[cfg]` | 不认：`#[cfg(test)]` 模块的词法照收，测试模块里的 `use` 照样带文件进闭包 | 属性算进 fn 项的词法；不判条件真假 | 不认 |
| `#[path] mod` | 只在测试目标里认（`:485` `&& is_test_crate`）；库与 build.rs 里的 `#[path] mod` 不跟 | 不认 | 不认 |
| 跨 crate | 工作区 crate 按名字落到它的 `lib.rs` 与模块文件；外部 crate 不跟（`:415` std / core / alloc 交 None），只拼 `Cargo.lock` 整份原文（`:994`） | 全局键收 Cargo 清单原文（底稿 L22）；测试认不出退整包指纹（`:725` `"package:"`） | 不涉及 |

`admission.py` 的 python 闭包（`scripts/gate/admission.py:2716` `crash_case_judging_digest_text`）：从入口名字起，在**同一个模块**里顺 `ast.Name` 收模块级 def / class / 赋值（`:2758`）；`obj.attr`、`getattr`、`globals()[…]` 按字符串取的都不收；单元内容是原文行（`source_of_node` `:2680`，注释、文档串都在）；闭包里用到非标准库的 import 名字、或自己 import 非标准库就拒算（`:2765`、`:2775`）。这一条「认不出就拒算」是四份里唯一现成的拒算口子，新设计照它扩到全部认不出的形态。

小结（推的，依据上表）：没有一份能直接扩成函数单元的调用图。可以拿来用的零件：FileIndex 的 fn / impl 切分骨架（`scan` `:327`）、摘要库的 bash `declare -f` 规范化（`bash_normal_forms` `:312`）、admission 的拒算口子。三份 Rust 词法互不相同（文档注释一份去、一份留，第三份只留 doctest 代码行），合并时只留一份切法。

## 二、探针在本仓上量到的（量的；副本 = 主仓 `crates/`、`Cargo.toml`、`Cargo.lock` 的 rsync 拷贝，26M）

探针（全在 `probe/`）：`rust_units.py` 词法（去全部注释与文档注释）与条目切分（fn、方法、const、static、struct / enum / union、type、trait 头、impl 头、`macro_rules!`、use 条目、条目级宏调用、`mod x;`）；
其上逐层加规则：`scoped.py`（编译目标与 `pub`、字段访问不解）→ `contexted.py`（按（单元，编译目标）走、use 只做别名）→ `rta.py`（方法按闭包里点过名的类型收窄）→ `rta_open.py` / `rta_fast.py`（`dyn Trait` 与入口签名处的 CHA；`rta_fast.py` 是同一套规则的一遍增量实现）。键 = 闭包里每个单元「种类 + 去注释词法单元串的 sha256」排序后取 sha256，不带路径与名字。三个入口：V5 点名的测试 `root_slots_system_configurations_and_journal_ring_hold_the_published_state`（harness `tests/new_pool_file_creation_publish.rs`）、V11 点名的测试 `pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for`（`singlefs-format/src/lib.rs`）、CPU 判器 `judge_crash_image`（`singlefs-checker-tier/src/crash.rs:1176`，不含 `#[cfg(test)]`）。

### 2.1 闭包有多大，取决于名字怎么解

| 解法 | V11 测试 | V5 测试 | CPU 判器 | 出处 |
|---|---|---|---|---|
| 裸名、全工作区 | 10 627 单元 / 291 文件 | 10 630 / 291 | 10 358 / 289 | `probe/base.json` |
| + 编译目标、`pub`、字段访问 | 18 / 1 | 7 074 / 145 | 6 829 / 145 | 本会话 `scoped.py` 输出 |
| + 按（单元，目标）走、use 只做别名 | 18 / 1 | 3 632 / 54 | 5 522 / 79 | 本会话 `contexted.py` 输出 |
| + RTA 收窄方法 | 18 / 1 | 1 987 / 43 | 2 631 / 47 | 本会话 `rta.py` 输出 |
| + `dyn` 与开放入口的 CHA（定稿） | 18 / 1 | 2 070 / 45 | 2 705 / 49 | 本会话 `rta_fast.py` 输出 |

前四行是修「条目级宏调用按可见处理」（2.3 末尾）之前量的，定稿那一行在修之后；切分（405 份、24 966 个单元）五行相同。

定稿那一行的原样输出（`timeout 200 nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 8G python3 -I rta_fast.py <副本> "$(cat spec.json)"`，`/usr/bin/time` 报 `wall=2.86s maxrss=349908KB`）：

```
{"v5_test": {"key": "c9c7dd1622377e18f2ea9dee213387c51a25f9215b09e929f6ffcc8f56f842f2", "units": 2070, "files": 45, "fn_units": 929, "has": ["DATA_POINTER_BYTES", "JOURNAL_PAYLOAD_CHECKSUM_OFFSET"]}, "v11_test": {"key": "104da96fb9619485f8b78a6b9624a1e05eb008d023cb8fafdda049cbc1e679e5", "units": 18, "files": 1, "fn_units": 1, "has": ["DATA_POINTER_BYTES"]}, "cpu_judge": {"key": "ca4fa8660bcb3a0401b1b01cefe26ba432db880ca659b19ca63971f5d84db3f5", "units": 2705, "files": 49, "fn_units": 1232, "has": ["DATA_POINTER_BYTES", "JOURNAL_PAYLOAD_CHECKSUM_OFFSET"]}}
```

噪声从哪来（`contexted.py` 那一版的诊断，量的）：判器闭包被 `.u64()`、`.u8()`、`.new()`、`.name()`、`.of()`、`.map()` 这类方法名一路带进流水线文件，例如进 `crash_amplification.rs` 的那条链（`judge_files.py` 逐级打出的，从入口往下读）是 `judge_crash_image` →（名字 `with_a_file_only`）→ `<impl CrashPointReplayVersionTable>::versions_without_a_file` →（它模块里的 use）→ `const SECTOR_BYTES`（`crash_identity.rs`）→（名字 `u64`：常量的类型名撞上了方法名）→ `<impl Cursor>::u64`（`crash_facts.rs`）→（名字 `u64`）→ `<impl RecordedBytesReader>::u64`（`crash_amplification.rs`）。RTA 版（`judge_files_rta.py`、`judge_files_fast.py`）判器闭包里流水线七份一个单元都没有，原样：`units in closure: 2705  units in the touched files: 5597`，`流水线` 标记 0 行。

### 2.2 阳性与阴性对照（`controls_fast.py`，每格一份新拷贝施加一处改动，原文恰好命中一次才施加）

`probe/controls-fast.out` 原样：

```
{"variant": "V5", "breaks": [], "v5_test": "changed", "v11_test": "same", "cpu_judge": "changed"}
{"variant": "V11", "breaks": [], "v5_test": "changed", "v11_test": "changed", "cpu_judge": "changed"}
{"variant": "comment_only_in_judge_fn", "breaks": [], "v5_test": "same", "v11_test": "same", "cpu_judge": "same"}
{"variant": "unreached_fn_body_same_file", "breaks": [], "v5_test": "same", "v11_test": "same", "cpu_judge": "same"}
{"variant": "reached_fn_body", "breaks": [], "v5_test": "same", "v11_test": "same", "cpu_judge": "changed"}
{"variant": "unrelated_new_fn_before_the_doc_comment", "breaks": [], "v5_test": "same", "v11_test": "same", "cpu_judge": "same"}
{"variant": "attribute_moved_off_the_judge_fn", "breaks": [], "v5_test": "same", "v11_test": "same", "cpu_judge": "changed"}
```

把 const 与 static 单元关掉（`probe/controls-fast-no-const.out`）：

```
{"variant": "V5", "breaks": ["const"], "v5_test": "same", "v11_test": "same", "cpu_judge": "same"}
{"variant": "V11", "breaks": ["const"], "v5_test": "same", "v11_test": "same", "cpu_judge": "same"}
```

V5 = `crates/mutations.tsv:439`（`const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;` → `91`，`singlefs-checker/src/lib.rs:712`）；V11 = `crates/mutations.tsv:315`（`pub const DATA_POINTER_BYTES: u64 = 88;` → `89`，`singlefs-format/src/lib.rs`）。
V11 那一行里 V5 测试的键也变：V5 测试的闭包里也有 `DATA_POINTER_BYTES`。V5 那一行里格式库单测的键不变：V5 的常量是 `singlefs-checker` 私有的，格式库够不到。
造对照时踩到一次自己的错：第一次把不相干的新函数插在 `#[must_use]` 与 `judge_crash_image` 之间，属性挪到了新函数上，判器单元真变了、键变（`probe/controls.out` 里 `unrelated_new_fn_same_file … cpu_judge: changed`）；挪到文档注释之前重造就不变（`unrelated_new_fn_before_the_doc_comment` 那一行），属性单独挪走就变（`attribute_moved_off_the_judge_fn` 那一行）。

### 2.3 形态夹具（`probe/fixture/`：三个 crate `fcore` ← `fharness` ← `ftier`，入口都在 `ftier/src/lib.rs`；用例与期望在 `probe/fixture_cases.json`）

`timeout 100 nice -n 19 python3 -I fixture_run.py <夹具> <草稿> rta_fast Fast` 原样（`probe/fixture-rta-fast.out`）：

```
✓ dyn 分派到下游 crate 的 impl 期望 changed 得 changed
✓ dyn 分派：没被点名的另一个 impl（CHA 过近似） 期望 changed 得 changed
✓ 泛型约束分派 期望 changed 得 changed
✓ trait 默认方法 期望 changed 得 changed
✓ 运算符重载 期望 changed 得 changed
✓ Drop 隐式调用 期望 changed 得 changed
✓ 声明宏展开出的函数 期望 changed 得 changed
✓ 关联常量（经泛型） 期望 changed 得 changed
✓ 函数指针表 期望 changed 得 changed
✓ 开放入口：参数是 dyn 期望 changed 得 changed
✓ 数组长度里的常量 期望 changed 得 changed
✓ 阴性：够不到的函数 期望 same 得 same
✓ 阴性：只改注释 期望 same 得 same
不符 0 格 / 共 13 格
```

规则逐条拆掉（`breaks.py`），夹具红在哪（原样）：

```
== 拆掉：NoCha
✗ dyn 分派：没被点名的另一个 impl（CHA 过近似） 期望 changed 得 same
✗ 开放入口：参数是 dyn 期望 changed 得 same
不符 2 格 / 共 13 格
== 拆掉：NoConst
✗ 关联常量（经泛型） 期望 changed 得 same
✗ 函数指针表 期望 changed 得 same
✗ 数组长度里的常量 期望 changed 得 same
不符 3 格 / 共 13 格
== 拆掉：NoMacro
✗ 声明宏展开出的函数 期望 changed 得 same
不符 1 格 / 共 13 格
== 拆掉：NoTraitImpl
✗ 运算符重载 期望 changed 得 same
✗ Drop 隐式调用 期望 changed 得 same
✗ 关联常量（经泛型） 期望 changed 得 same
不符 3 格 / 共 13 格
```

两处在夹具上抓到的探针自己的漏（量的，修之前的读数）：只用 RTA、不加 CHA 时「开放入口：参数是 dyn」漏（`rta RTA` 那一趟 `不符 3 格`，修完宏可见性后 `不符 2 格`）——判器这种库入口的参数可能是调用方构造的对象，类型名不在闭包里，RTA 只对封闭程序成立；三个版本都漏「声明宏展开出的函数」，原因是条目级宏调用单元前面没有 `pub`，跨 crate 解名字时被可见性那一关滤掉。宏展开出来的条目可见性按词法定不了，改成按可见处理后补上。这两条都写进第三节的规则。

### 2.4 拿真实提交量收益（逐提交与父提交比；`git archive` 导出到草稿目录、用完即删）

CPU 判器入口，单元键对「闭包触到的文件各取整份词法串」（`history.py`，用的是 `contexted.py` 那一版解法；`probe/history.out` 前 14 行）：能比的 14 个提交里两种键逐个同向，12 个都变、2 个都不变；更早的提交里还没有 `judge_crash_image`（`fn judge_crash_image 命中 0 处`）。这一段历史是大批量提交（例 `d6dd6da42834` 动了 269 份文件、76 737 行增），逐提交这一层按单元没有省下判器重判。

按测试函数（`tests_history_fast2.py`，`rta_fast` 解法，全工作区每条 `#[test]` 一把键；对照「测试所在 crate 与它依赖的 crate 里任一份 `.rs` 词法变了就算变」）。`probe/tests-history-fast.out` 原样：

```
{"commit": "40412d3045a7", "tests_both": 1956, "unit_key_changed": 1283, "crate_key_changed": 1911, "judge_unit_key": "changed", "seconds_for_both_trees": 152.1}
{"commit": "c34be3b40bd2", "tests_both": 1932, "unit_key_changed": 282, "crate_key_changed": 1731, "judge_unit_key": "same", "seconds_for_both_trees": 148.3}
{"commit": "c3e9d9edbe45", "tests_both": 1581, "unit_key_changed": 1241, "crate_key_changed": 1581, "judge_unit_key": "changed", "seconds_for_both_trees": 93.7}
```

三个都是只动 `crates/` 下一份 `.rs` 的提交：`40412d30` 补进 `singlefs-replay-prefix` 的源码（父树里 core 的 `Cargo.toml:13` 已经依赖它、`recovery.rs` 有 5 处 `singlefs_replay_prefix`）、`c34be3b4` 补进 `singlefs-reference-filesystem` 的源码（父树里 harness 的 `Cargo.toml:14` 已经依赖它）、`c3e9d9ed` 改格式常量（`singlefs-format/src/lib.rs` 加减 `TOMBSTONE_*`、`LINEAGE_*` 这些 `pub const`）。按单元作废 282 / 1 932 条、按 crate 1 731 / 1 932 条是这三个里差得最开的；格式常量那一次单元作废 1 241 / 1 581（格式常量几乎人人用）。三个点不是「最坏情况」，只说明量级（推的）。
探针按测试逐条算一棵树要 47–76 秒（python、一条一条从头收）；正式实现要按单元记忆化，这个数不代表它。

`tests_history.py`（`contexted` 解法、只扫格式库、core 与 checker 三处）跑超了 Bash 默认的 120 秒被转到后台、撞上自带的 `timeout 235` 退 143，`probe/tests-history.out` 只有前两个提交的读数：`7b5711fd` 293 条 0 变、`d6dd6da4` 287 条 243 变；`rta_open.py` 那一版同样转后台、退 143、没有读数（换成增量的 `rta_fast.py` 后 2.86 秒），进程我用 `proc.py wait` 核过都已退出。

## 三、问②：「函数单元指纹 + 调用图闭包」统一设计

### 3.1 键

一条判定的键 = 入口目标的生效 cfg 集 + 工具链（`rustc -vV`；cargo 解析后的 RUSTFLAGS、profile、target、runner，随底稿第二节第 4 条）+ 外部依赖解析结果（`Cargo.lock` 里入口链接集真引到的包：名、版本、来源、checksum，及传递依赖；随底稿 L4）+ 闭包里每个单元的（种类，去注释指纹）多重集 + 闭包里数据单元的内容摘要。不带路径、名字、行号、手写版本号。组成清单（单元限定名、文件:行，只当索引）随键落盘，作废时报出变了哪几个单元（底稿第二节第 9 条 ⑦）。

### 3.2 Rust：怎么切、怎么去注释

- 词法一份：去 `//`、可嵌套 `/* */`、`///`、`//!`、`/** */`、`/*! */`；串、原始串、字节串、C 串、字符、生命周期、数字（带指数与后缀）各一个单元；单元之间用 0x1f 连起来取 sha256（探针 `rust_units.py` 的 `tokenize`）。今天三份切法在文档注释上不一致（第一节小结），统一成：文档注释不进条目指纹；文档注释里的 ```` ``` ```` 代码块另成「文档测试单元」，挂在所属条目下，只进文档测试那几条入口的闭包。
- 单元：自由 fn（属性、签名、where、函数体；函数体里的嵌套条目算它的）；impl 头（`impl<…> Trait for Type where …` 到 `{`，连属性）与它的每个方法、关联 const、关联 type；trait 头与它的每个成员；const；static；struct / enum / union（连 derive 属性、字段、判别式）；type 别名；`macro_rules!` 整个定义；每条 use；条目级宏调用（`verus!`、`thread_local!` 这类只包条目的，展开成里面的条目再切）；`mod x;`（连 `#[path]`）；extern 块；extern crate。
- `#[cfg]` / `#[cfg_attr]`：每个单元带它与外层模块的条件链，按入口目标的生效 cfg 集（features、`test`、`target_arch`、`debug_assertions`、`--cfg`）求值，假的不进闭包；`cfg(any())` 恒假（今天 26 处）；认不出的谓词拒算。
- 数据单元：`include_str!` / `include_bytes!` 读的文件（参数是字面量、字面量的 `concat!`、`env!("CARGO_MANIFEST_DIR")` 起头的 `concat!`）取字节 sha256；`concat!(env!("OUT_DIR"), …)`（今天 3 处 `OUT_DIR`）连到产出它的 build.rs 的 `main` 闭包与 build.rs 读的输入；`include!` 把被含文件的单元并进来（今天 0 处）；运行期按 `env!("CARGO_MANIFEST_DIR")` 拼路径读文件的（今天 24 处）拼得出字面路径就取那份文件当数据单元。读 `.rs` 原文的测试（`include_str!` 读 `.rs` 今天 19 处）照数据单元取原文——它们真读注释，去注释会漏（E164 F6 的 V8 就是这一类）。

### 3.3 Rust：从入口怎么收（与探针 `rta_fast.py` 逐条同义；规则名给派发提示引）

- R-目标：文件属于哪几个编译目标（lib、每个 bin、每个测试目标、build.rs；`#[path] mod` 把被引文件并进声明它的那个目标），名字只在当前目标与它依赖 crate 的 lib 里解，进了依赖 crate 的单元就换成那个目标接着解；跨 crate 只解到 `pub` 的单元，trait 成员、trait impl 成员、`#[macro_export]` 宏与条目级宏调用一律算可见。
- R-裸名：不在 `.` 与 `::` 后面的标识符，解到同名的非成员单元（自由 fn、const、static、类型、trait、宏、条目级宏调用里出现过的名字）。
- R-路径：`Q::x` 解到 Q 的 impl 成员、trait Q 的成员（`Self::x` 落到本 impl 自身类型）、以 Q 为 trait 写的 impl 里自身类型已被点名的成员，再按裸名解 x（Q 是模块路径时）。
- R-方法（RTA）：`.x(`、`.x::<` 解到闭包里出现过的 trait 的成员 x，与自身类型已被闭包点过名的 impl 的成员 x；类型后来才被点名的，补上之前见过的方法名。基本类型与 `Self`、`Vec`、`Option`、`Result`、`Box`、`String` 视为恒被点名。
- R-隐式：类型被点名时，给它写的 trait impl 头与全部成员进闭包（运算符、Drop、Deref、Index、From、derive 一类隐式调用）。
- R-开放（CHA）：闭包里任一单元写了 `dyn Trait`，或入口签名里出现 trait 约束（`impl Trait`、`T: Trait`、`dyn Trait`），入口链接集里 Trait 的全部 impl 头与成员进闭包。
- R-结构：成员连它的 impl / trait 头；单元连它所在模块里引进了它用到的名字的 use（glob 一律连），`use a::b as c` 引用 c 按 b 再解；字段访问 `.f`（后面不接 `(`）不解。
- R-宏：`m!` 按裸名解到 `macro_rules! m`；宏定义体里的标识符算宏单元的引用；条目级宏调用里的标识符都算它可能定义的名字。

V5、V11 为什么抓得到（正推）：const 是单元，指纹里有字面量 `95` / `88`；用到它的 fn 单元里有标识符 `JOURNAL_PAYLOAD_CHECKSUM_OFFSET` / `DATA_POINTER_BYTES`，按 R-裸名（V11 跨 crate 经 use 与 `pub`，R-目标）连到它；改一个数只改这个 const 单元的指纹，多重集变、键变。反推：拆掉 const 与 static 单元（`breaks.py` 的 NoConst、`controls_fast.py … '["const"]'`），V5、V11 在三个入口上都 `same`（2.2 节原样）；按函数切（E164 的 `digest_by_function` 只取 fn 项）就是这种漏法。校验（不共用探针代码的另一条路）：这两处改动真改了行为——E164 的产物行 `E7RESULT name=q4_flip variant=V5 base_exit=0 variant_exit=101 green_to_red=true` 与 `E7RESULT name=q4_flip variant=V11 base_exit=0 variant_exit=101 green_to_red=true`（`.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md:64`、`:65`，当时 `cargo test` 跑的，读数是旧的、没重跑），所以「键该变」成立，探针在这两个入口上也确实变了。另：V5 在格式库单测上不变（`controls-fast.out` 第 1 行 `v11_test: same`），它是 `singlefs-checker` 的私有常量，格式库够不到，这是该不变的那一格。

### 3.4 拒算（认不出不静默跳过：这一条判定不复用、每趟现判，红句点名单元与形态，`--list-items` 报拒算条数）

括号配不上；认不出的条目起头（探针在当前仓上报 `problems: 4`，`probe/base.json`；是哪四处、怎么补切法见本节末尾）；cfg 谓词认不出；`include_*!` 参数不在 3.2 那几种里；运行期读文件拼不出字面路径；`concat_idents!`、`paste!`（今天 0 处）；工作区里出现 proc-macro crate（今天 0 个，`proc-macro = true` 一处都没有）；测试起子进程：`env!("CARGO_BIN_EXE_<名>")`（今天 0 处）连到那个 bin 的 `main` 闭包，`current_exe`（今天 1 处）拒算；`extern "C"` 调到工作区外的符号连到提供它的 `-sys` 包版本，认不出拒算。

漏判风险（推的）：R-方法的前提是「运行时出现的工作区类型，闭包里都有单元点过它的名字」。封闭程序（测试函数、bin 的 `main`）里构造一个值要点类型名或调一个签名点了它的函数；库入口的参数由调用方给，靠 R-开放补（夹具「开放入口」那一格，量的）。剩下没覆盖的形态：外部 crate 的泛型函数构造、回传工作区类型（`Default::default()` 经外部泛型、`serde` 反序列化这类），要补夹具；补不出就把「外部 crate 函数的泛型返回」也按 R-开放处理。

探针那 4 处「认不出的条目」（`rust_units.py` 在副本上逐个打出来，原样）：

```
crates/singlefs-checker-tier/src/bad_disk_input.rs | 认不出的条目 singlefs_harness : : enum_listing_every_member ! { # [
crates/singlefs-checker-tier/src/crash_injection.rs | 认不出的条目 singlefs_harness : : enum_listing_every_member ! { # [
crates/singlefs-harness/tests/fault_injection_fast_tier.rs | 认不出的条目 singlefs_harness : : enum_listing_every_member ! { # [
crates/singlefs-replay-prefix/src/lib.rs | 认不出的条目 else { Seq : : < JournalCounter >
```

前三处是带路径的条目级宏调用：探针只认不带路径的 `m! { … }`，这三段因此没当宏调用单元，宏里定义出来的枚举名不进名字表，引用它的单元解不到——是真漏（推的，这三个枚举在不在三个入口的闭包里没核）。切法要认 `a::b::m! { … }` 与 `m! { … }` 同一种。
第四处在 `verus!` 里：Verus 的 `requires` / `ensures` 子句可以写带花括号的表达式（`ensures if … { … } else { … }`），「签名之后第一个 `{` 是函数体」不成立。`verus!` 里的条目改成「从这个条目起头到下一个同层条目起头之前」整段算一个单元（括号配平，同层的条目起头按关键字认）。

### 3.5 WGSL

- 单元：`fn`、`const` / `override`、`struct`、`alias`、`var<…>` 绑定声明、`@compute` 入口；去 `//` 与可嵌套的 `/* */`，词法同 Rust 那一份（WGSL 的字面量更少，同一个切词器够用）。
- 模块从哪来：Rust 侧拼内核源码的函数（`crash_judge_gpu.rs:552` `recovery_kernel_source`、`:617` `checker_walk_kernel_source`、`:648` `checker_scan_kernel_source`）在 Rust 闭包里，它们经 `include_str!` 的 const（`crash_judge_gpu.rs:73-81`）读到哪几份 `.wgsl`，那几份就是这个模块。WGSL 没有重载、动态分派与宏，一个模块里名字唯一：从模块里的 `@compute` 入口按名字精确收。
- WGSL 里用到而 `.wgsl` 里没定义的名字，必须落在闭包里某个 Rust 单元的串字面量里（Rust 生成的常量前缀，例 `kernel_prelude_wgsl_constants` `crash_judge_gpu.rs:281`、`verdict_layout_wgsl_constants` `:228`）——那个 Rust 单元已在 Rust 闭包里，指纹管住它；落不到任何 Rust 串里的拒算。
- 今天按整份原文摘内核的两处（`crash_judge_gpu.rs:531-548` 那段与 `crash_judge_dispatch.rs:243` `kernel_sources_digest`）由这一把单元键代替。

### 3.6 Python

- 单元：模块级 def / async def；类头（基类、装饰器、类体里的非 def 语句）与类里每个方法各一单元；模块级赋值按目标名各一单元；模块级其余语句（import、if / try / for、`if __name__ == "__main__"`）合成「模块体」单元，当这份脚本被当程序起时的入口。
- 指纹：`ast.dump(节点, include_attributes=False)`（不带行列号；注释本来不在 ast 里）。量的：只改注释与空白时两份 dump 相等、改文档串时不等（本会话 `python3 -` 那一小段，输出 `只改注释与空白： True`、`改文档串： False`）。文档串从指纹里去掉；闭包里有单元读 `__doc__`（今天 10 份文件读它）时，被读的那几段文档串当数据单元进键；读自己原文的（`inspect.getsource`、按 `__file__` 读，今天 6 份）整份原文当数据单元。
- 名字：`ast.Name` → 本模块同名单元；`mod.f`（mod 是 import 进来的仓内模块）→ 那份模块的 f；`self.m` / `obj.m` → 闭包里出现过的类的同名方法（与 Rust 的 R-方法同一条 RTA）。跨模块：`import` / `from … import` 落到仓内文件；`importlib.util.spec_from_file_location(<名>, <路径>)`（今天 52 处 / 35 份）与 `sys.path.insert`（97 份）的路径按「`__file__` 所在目录拼字面量」折叠。`getattr(x, <变量>)`（10 处）、`globals()[…]`（15 处）、折叠不出的 import 路径拒算。`subprocess` 起 `python3 <路径>` / `bash <路径>` 的连到那份脚本的模块体入口，路径折叠不出拒算。
- 今天 `admission.py:2716` 那份闭包（同模块、只认 `ast.Name`、原文带注释）由这一份代替。

### 3.7 Shell

- 单元：用摘要库现成的规范化（`lib_source_digest.py:312` `bash_normal_forms`：`bash -n` 过了、把整份包进一个函数定义里 eval、不执行它、取 `declare -f`），在规范文本上按缩进切：`function 名 () ` 一行起、同缩进的 `};` 止，每个函数一个单元（嵌套的另算一个单元，外层单元里留着它）；顶层语句合成「脚本体」单元。量的（`probe/sh/split.sh` 跑 `probe/sh/sample.sh`）：注释与行尾注释、缩进都去掉，串里的 `#` 留着，here-doc 正文原样、不缩进——切的时候按 here-doc 起止跳过正文，不按缩进切它。`bash -n` 不过或有告警的整份拒算。程序读的注释（「# <词>:」与 `# shellcheck`，摘要库 `KEPT_SHELL_COMMENT` 那一条）当数据单元进键。
- 名字：任何一个词（命令位置或参数、引号里也算）等于本闭包可见的函数名就连上——`stage_cell device-streams cell_device_streams …` 这种把函数名当参数传的照样连到（过近似，不漏）。
- 跨文件：`source` / `.`、`bash`、`python3` 后面的路径是字面量，或 `$X/<字面量>` 而 X 的赋值折叠得出（`$(dirname "${BASH_SOURCE[0]}")`、`$(cd "$(dirname "$0")/../.." && pwd)` 这一类）→ 连到那份脚本的脚本体入口；折叠不出的拒算。今天「整个路径就是一个变量」的写法 966 处 / 155 份（量的，`grep` 不分折得出折不出），这是 shell 那一份最大的拒算面：先实现折叠，再数剩下几处，剩下的改写成字面路径。
- 外部程序（qemu、herd7、busybox、cargo）：`--version` 一行与二进制 sha256 进键。

### 3.8 放在哪、谁调谁

- `crates/singlefs-unit-graph/`（新 crate，只用 std）：Rust 与 WGSL 的切分、指纹、闭包、拒算。`singlefs-checker-tier` 的库与它的 `build.rs` 都经 Cargo 依赖用它（`[dependencies]` 与 `[build-dependencies]`），取代今天 `build.rs:6-19` 经 `#[path]` 把 `judging_closure.rs`、harness 的 `sha256.rs`、`hexadecimal.rs` 拉进来的写法；sha256 搬进这个 crate（`singlefs-harness` 改成依赖它）。另带一个 bin `unit-graph`：`dump <根>`（全部单元：种类、限定名、文件:行、指纹）、`closure <根> <入口>`、`key <根> <入口>`。
- build.rs 做什么：在编它的那棵树上，按登记的判器入口（`<crate>::<模块路径>::<fn>` 这种 fn 路径，不是文件路径）各算一把键与组成清单，写进 `OUT_DIR`，二进制里带着「我这一份判器代码的键」。判定入库、查库都用这把键：二进制比盘上的树旧时，它写的键对不上新树算出来的键，查不中就现判——不再需要「编译时整张表与盘上现算的整张表比、不等整趟不写库」（底稿 L9 那一条整个删），运行期也不再读仓（`crash_identity.rs:792` 那一处读树）。
- `scripts/gate/lib_function_units.py`：python 与 shell 两种语言的切分与闭包，外加 Rust 的独立第二份实现（照第三节规则另写，与 crate 那一份不共用代码）。门禁里 python 写的判法（广谱变异、crates 变异表、准入、复跑）直接 import 它算键，不为算键先编 Rust。
- 对拍进快档（新的一格）：两份 Rust 实现在 `crates/` 全部 `.rs` 与夹具上逐单元比（种类、限定名、起止、指纹），再对登记的每个入口比闭包成员与键；任何一处不同判红、点名单元。这一格就是 `evidence-discipline.md`「校验」要的两条不共用代码的路子；它自己的证红：在任一边打开一个弄坏开关（例：python 那份不切 static），这一格必须红。
- 入口登记：随底稿第二节第 9 条 ⑨，`stage-inputs.tsv` 第二列写入口（测试函数、bin、litmus、虚机档）；crates 侧判器入口登记在 `singlefs-unit-graph` 读的一张表里（fn 路径）。登记表的文字不进键，只答「从哪些入口起算」。
- 退役：`judging_closure.rs` 整份（闭包、停止表 `:666`、`:691`、`FILES_THE_JUDGES_MUST_NOT_REACH`、摘要表与 `CONCLUSION_CODE_DIGEST_HEADER` `:783`）；`mutation-prune.py` 的 `outside_fingerprint`、`dependency_fingerprint`、`tree_fingerprint`、`package:` 退路；`lib_source_digest.py` 的文件级摘要（它的 bash 规范化与 python 去注释搬进新库）；`admission.py:2716` 那份闭包。「判器闭包里不许有流水线」这条设计约束可以留成一条测试（断言判器入口的闭包里没有登记的流水线单元），它不进键。

## 四、问③：Verus 能不能按函数拆项

量的：

- 登记的 Verus 是 `0.2026.09.20.aef82ed`（`.claude/gate.d/verus-pins.tsv` release 行；安装目录 `version.json` 的 commit `aef82eda71838deef4a8cd0260fda6c9504be9b9`），核 crate `crates/singlefs-replay-prefix`、登记 verified 56、rlimit 10（同表 `crate`、`rlimit` 行）。
- `verus --help`（原样落在 `verus-help.txt`）：`--verify-module MODULE` 验一个子模块连同后代、可重复；`--verify-only-module MODULE` 验一个子模块不含后代、可重复；`--verify-function MODULE` 「Verify just one function within the one module specified by verify-module or verify-root, matches on unique substring (foo) or wildcards at ends of the argument (*foo, foo*, *foo*)」；`--no-cheating`「Do not allow assume, admit, verifier::external_body, and assume_specification」。
- `rust_verify` 二进制里的报错串（`strings` 读出，原样摘这几条）：`--verify-function option requires --verify-only-module or --verify-root`、`Must pass at most one --verify-only-module or --verify-root when using --verify-function`、`When using --verify-function, use --verify-only-module instead of --verify-module`、`more than one match found for --verify-function`；JSON 的键里有 `is-verifying-entire-crate`。
- 核 crate 的形态：证明全在一个 `verus! {` 块里（`lib.rs:30` 起、`lib.rs:4089` `} // verus!` 止），块外只有测试模块（`lib.rs:4094` `mod tests {`），没有别的 `mod`：只有根模块。按模块拆在这里就是整 crate。
- 块里 90 个 fn：spec 53、proof 28、exec 9；其中 4 个名字是别的 fn 名字的子串（`above_the_chosen_root`、`candidates_before`、`recovery_replays_from_the_newest_self_verified_root`、`replay_core`），按子串点名会撞「more than one match」，要用通配写法（例 `*replay_core` 与 `replay_core*` 都可能多中，能不能写精确名没量）。
- `verus-proof-check.py` 今天的判法：整 crate 一项（`check_proof` 收 `--item` 就退 2），带 `--no-cheating`（`:470`），判 `is-verifying-entire-crate` 为真（`:889`，「别给 --verify-module 这类只验一部分的选项」），verified 等于登记的 56；键是核 crate 整个目录（`:807`）+ `verus-pins.tsv` 整份 + 工具链 + 限时（`shared_components`），判法版本是两份脚本整份原文（`judge_version` `:788`）。

没量、要主 agent 量的（派发只许跑 `--help`）：「部分验证下的作弊检查」——只验一个函数时 `--no-cheating` 查的是整 crate 还是只查被验的那个函数（决定按函数拆项后要不要另起一项「整 crate 作弊扫描」）；「逐函数计数」——逐个函数验时各自报的 verified 加起来等不等于整 crate 的 56（spec fn 算不算、proof fn 怎么计）；「撞名」——`--verify-function` 能不能写成只中一个的精确名。命令在第九节。

设计（两种结局各写好，量完照结局取一种）：

- 能按函数拆（三问都答「能」）：每个 exec / proof fn 一项，项的键 = 这个 fn 单元 + 它 requires / ensures / decreases 与函数体里用到的 spec fn、proof fn（引理）、类型定义、`broadcast use` 组的单元闭包（Rust 那一套切法，`verus!` 里按 3.4 的整段切）+ Verus 与 vstd 版本 + rlimit（rlimit 改变结论，进键；限时只影响超时，移出键，随底稿 L24）。spec fn 自己的良构检查跟着用到它的那一项。项名只当索引。全部项的 verified 之和 = 整 crate 那一次的数，这一条当对照钉在自证里；另立一项「作弊扫描」（若 `--no-cheating` 在部分验证下不查整 crate）：键 = 全部单元指纹多重集，逐 fn 查 `assume` / `admit` / `external_body` / `assume_specification`（判法照 `--no-cheating` 的四样）。
- 不能按函数拆：替代单元退到「模块」，这个 crate 只有根模块，就是整 crate 一项；要拿到单元级，得先把核 crate 按证明的层次拆成几个子模块（这是改代码，归 kb 定），每个模块一项、`--verify-only-module` 起，键 = 该模块里全部单元 + 它从别的模块引到的单元闭包。理由：Verus 的验证单位是函数，部分验证的入口只有模块与函数两级，没有第三种（`--help` 原样）。
- 两种结局里都一样：空真项的身份改成「所属函数单元指纹 + 函数内第几处 requires」，不带路径与行号（今天 `vacuity:<路径>:<行号>`，`:1015`）；必红世界表每行的键 = 被改坏的那个函数单元指纹 + 原文 + 替换文 + 期望 + 被改坏单元的闭包，不带变异名与路径（今天项名 `must-red:<名>`，`:1120`）。

## 五、问④：QEMU 每档的来宾输入怎么取；LKMM 锚点能不能细到 fn

### 5.1 QEMU（量的：读 `.claude/gate.d/checker-tier-qemu-device-streams.sh`、`scripts/devices/vm-bench.sh`、`scripts/devices/vm-kernel.sh`）

今天：六档（`checker-tier-qemu-device-streams.sh:64` `MODES=(direct skip-new-pool-file-creation-barrier page-cache file-overwrite second-instance raise-rollback-floor)`）共一个整道全绿标记（`:453-456`，`--item` 跑的不写）；登记行把 `crates/ Cargo.toml Cargo.lock scripts/ research/results/ .claude/kb/layout/01-first-txn.md` 整列当输入（`stage-inputs.tsv:37`）。每档起虚机时 `vm-bench.sh` 的 `run_one`（`:48`）在一个新建的临时目录里现拼 initramfs：拷宿主的 busybox（`:55`）与它的全部链接名、拷被测二进制成 `/bench`（`:57`）、按需并进 `VM_EXTRA_ROOT`、写 `args` 与 `devs` 两个小文件、用 here-doc 写 `/init`（`:78`），再 `find . | cpio -o -H newc --quiet | gzip -1`（`:93`）。内核由 `find_kernel`（`:37`）取 `SINGLEFS_KERNEL` 或 `vm-kernel.sh` 找到的可读镜像；blklogwrites 那一支用 `qemu.env` 里 `SINGLEFS_QEMU_HOME` 指的补丁版 QEMU（`:234`），开跑前 `qemu-blklogwrites-check.sh` 核过。

每档一条判定，键（设计）：

| 组成 | 怎么取 | 为什么这样取 |
|---|---|---|
| 被测二进制 `new_pool_file_creation_on_device` | 它的 `main`（`src/bin/`）起的 Rust 单元闭包 + 构建配置（`--target x86_64-unknown-linux-gnu`、`+crt-static`，`checker-tier-qemu-device-streams.sh:242`）+ 工具链 + 依赖解析；二进制的 sha256 只当属性记 | 二进制字节随路径、调试信息、链接顺序可变，单元键才是「代码没变」的判据（推的；没量过两次编出来的二进制字节是否相同） |
| 宿主一侧的设备日志检查 `new_pool_file_creation_device_log_check` | 同上，它的 `main` 闭包 | 同上 |
| 档位参数 | 这一档传给 `/bench` 的参数串（`args` 的内容）与 `VM_DISKS`、`VM_DISK_MB`、`VM_LOG_MB` 的值 | 档名不进键，档的内容进 |
| initramfs 其余内容 | 打包前的清单：每个条目的（路径、权限、内容 sha256；链接记目标），`/bench` 那一项换成上一行的单元键；`/init` 的正文是 `vm-bench.sh` 里 here-doc 的原文——它属于 `run_one` 这个 shell 单元，单元指纹已管住 | 不取 `initramfs.cpio.gz` 的 sha256：cpio newc 头里有 mtime，这些文件每趟新建。量的（草稿目录里打两次包，中间只 `touch` 两份文件）：`one.cpio` `f3e37f0330ca3986…`、`two.cpio` `74af3100fc614a78…` 不同；同一份 cpio 内容隔一段时间各 `gzip -1` 一次，`two.cpio.gz` 与 `three.cpio.gz` 都是 `479a8b01e6c5724a…`——从管道压不写时间，变的只有 cpio 头 |
| busybox | 宿主 busybox 二进制的 sha256 与 `busybox --help` 第一行 | 外部程序，没有单元 |
| 内核 | 镜像文件字节的 sha256（`SINGLEFS_KERNEL` 或 `vm-kernel.sh` 交的路径），连同它的版本串 | 外部产物，没有单元；不按路径记（`vm-kernel.sh` 会把 `/boot` 的镜像拷到 TMPDIR，路径会变） |
| QEMU | 补丁版 `qemu-system-x86_64` 与 `qemu-io` 的 sha256、`--version` 一行、补丁文件（`scripts/devices/qemu/blklogwrites-linearize-over-iov-max.patch`）内容 | `scripts/devices/build-patched-qemu.sh` 编出来的二进制是外部产物 |
| 宿主脚本 | 这一档真走到的 shell 单元：本阶段脚本里 `cell_device_streams` 起的闭包、`vm-bench.sh` 的 `run_one` 与它调的函数、`check-segment-registry.py` 的 python 单元闭包 | 取代整个 `scripts/` 前缀 |
| 对照数据 | 只取真读的那几行：E142 产物里 `name=segments`（`path=mkfs` 之外）与 `name=root_record` 两种行，布局表八「mkfs 种根」那一行（`check-segment-registry.py --print-mkfs-segments-line` 现取的那一行） | 取代整个 `research/results/` 与整份 `.md` |

### 5.2 LKMM（量的：读 `.claude/scripts/lkmm.sh`、`.claude/gate.d/checker-tier-lkmm.sh`、`litmus/`）

- 锚点文法已经是 fn 级：`singlefs-models: crates/<crate>/src/<文件>.rs::<函数名>`（`lkmm.sh:23`），拆法 `anchor_path="${anchor%::*}"; anchor_function="${anchor##*::}"`（`:277`），只判「文件在、文件里 grep 得到 `fn <名>`」（`:289`）。
- 它细不到的两处：impl 方法写不进去——`文件.rs::Type::method` 按最后一个 `::` 拆，路径部分成了 `文件.rs::Type`，判「文件不存在」；同名 fn 在一份文件里出现几次也照过（只要有一处）。今天 `litmus/` 里 3 份带锚点、共 7 行锚点、6 个不同的 fn（`grep -n 'singlefs-models: crates' litmus/*.litmus` 数出 7 行），这 6 个 fn 在各自文件里都恰好一处 `fn <名>`（`grep -cE` 逐个数，全是 1）。改法：锚点写成「文件 :: 单元限定名」（限定名同 3.3 的单元，`<impl Type>::method` 一类），判法改成「新切法在这份文件里恰好找到一个这个限定名的单元」，找不到或多于一个都红。
- herd7 的判定不读代码：`lkmm.sh:366` `herd7 -conf linux-kernel.cfg "$f"`，cwd 是内核树的 `tools/memory-model`。所以每份 litmus 一条判定，键 = litmus 正文（去 `(* … *)` 注释；`singlefs-expect:` 声明另取，它在注释里却决定判据）+ `linux-kernel.cfg` 与它引的 `.cat`、`.bell`、`.def` 文件内容 + herd7 版本（`herd7 -version` 第一行）。锚点函数单元的指纹**不进**这一条的键（herd7 不读它，进了只会多跑）；代码与 litmus 对不对得上，由绑定测试判（`crates/singlefs-harness/tests/publish_order_matches_litmus.rs`、`…/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs`，量的：`grep -rlF` 按 litmus 文件名找到的），它们是普通测试函数入口，键里有它们读的 litmus 原文（运行期按 `CARGO_MANIFEST_DIR` 拼路径读，3.2 的数据单元）与被录制的写路单元。底稿 L20 写的「锚点指到的函数单元指纹」进 herd7 键，按这一条改。
- 静态那几格（`rN` 声明、atomic 初值、对照组是不是去掉屏障的形态、锚点在不在）只读 litmus 与锚点，现判就几毫秒（推的），不建库。

## 六、附：`lib_libtest_result_lines.py` 能不能逐条认出测试名与结局（底稿第三节第 4 问）

能，有一个前提（量的：读 `scripts/gate/lib_libtest_result_lines.py` 文件头与 `:68` `result_line`）：结果行 `test <全名> ... <结局>` 交（全名、结局、should_panic），结局只认 `ok`、`FAILED`、`ignored`；`--nocapture` 下测试自己打的字接在 ` ... ` 后面的那一行不算结果行（文件头「认的行」一段）。所以按测试函数入库的那几趟要么不带 `--nocapture`，要么每条测试单独起；每趟都核「结果行条数 = 汇总行 passed + failed + ignored」，对不上整趟不入库（推的设计，今天的库没有这一核）。

## 七、能直接拆成派发提示的条目（每条：改哪份文件、做什么、自证怎么造红）

| 号 | 改哪份文件 | 做什么 | 自证怎么造红 |
|---|---|---|---|
| U1 | 新建 `crates/singlefs-unit-graph/`（`Cargo.toml`、`src/lib.rs`、`src/bin/unit-graph.rs`、`tests/`）；工作区 `Cargo.toml` 的 members | Rust 与 WGSL 的词法、条目切分、指纹（3.2、3.4 的两处修正、3.5）；只用 std；sha256 搬进来 | 夹具搬进 `tests/`：本报告 2.3 的 13 格，加 6 格新的——带路径的条目级宏调用定义了枚举、`verus!` 里 `ensures` 带花括号、`#[cfg(feature)]` 关着的单元改了（不作废）与开着的改了（作废）、只改文档注释（不作废）、改文档注释里的 doctest 代码（只作废文档测试入口）、`include_str!(concat!(env!("CARGO_MANIFEST_DIR"), …))` 读的文件改了（作废）。弄坏开关逐个列出它转红的格 |
| U2 | 同 U1 | 闭包规则 R-目标、R-裸名、R-路径、R-方法、R-隐式、R-开放、R-结构、R-宏（3.3）与拒算（3.4）；`unit-graph dump / closure / key` | 每条规则一个弄坏开关，至少照 2.3 量过的四个：去掉 CHA（红「dyn 另一个 impl」「开放入口」两格）、去掉 const / static 单元（红「关联常量」「函数指针表」「数组长度」三格）、去掉宏单元（红「声明宏展开」）、去掉 trait impl 隐式边（红「运算符」「Drop」「关联常量」）；再加一条集成测试：在临时拷贝上施加 `crates/mutations.tsv:315`（V11）与 `:439`（V5），登记的三个入口的键照 2.2 节那张表变 / 不变，打开去掉 const 的开关两格都转成「不变」 |
| U3 | `crates/singlefs-checker-tier/build.rs`、`Cargo.toml`（`[build-dependencies]`）、`src/crash_identity.rs`；删 `src/judging_closure.rs` | build.rs 按登记的判器入口（fn 路径）算键与组成清单写进 `OUT_DIR`；库读回这一把当判器键；删「编译时整表与盘上现算整表比」与运行期读仓（`crash_identity.rs:792`） | 测试在临时拷贝上：改判器闭包里一个 const，键变；改流水线里一个 fn（`crash_amplification.rs` 里任一个），键不变；断言判器入口闭包里没有流水线七份的单元（取代停止表的那条约束） |
| U4 | 新建 `scripts/gate/lib_function_units.py`（带 `--selftest`） | python 与 shell 的切分与闭包（3.6、3.7）；Rust 的第二份独立实现（照 3.3，不 import U1 的任何东西） | `--selftest`：U1 的夹具原样喂给 Rust 那一份；python 夹具（只改注释不作废、改文档串不作废但有单元读 `__doc__` 时作废、`getattr` 变量名拒算、`spec_from_file_location` 字面路径折叠出跨模块边）；shell 夹具（here-doc 正文里有一行 `};` 不许被切开、函数名当参数传照样连上、`bash "$x"` 折不出拒算） |
| U5 | 快档里的一格（放在哪一道归主 agent 定） | U1 的 `unit-graph dump` 与 U4 的 Rust 实现在 `crates/` 全部 `.rs` 与夹具上逐单元比，对登记入口逐个比闭包与键 | 在 U4 那份里打开一个弄坏开关（例：不切 static），这一格必须红并点名单元 |
| U6 | `scripts/mutation/mutation-prune.py`（`TreeIndex`、`mutation_key`、`carry_decision`、`changed_since`、`test_fingerprints`）与调它的 `scripts/mutation/broad-mutation.py` | 键改成：被改坏的那个函数单元指纹 + 原文 + 替换文 + 抓它那条测试的单元闭包键 + 工具链与依赖（底稿 L22）；删 `outside_fingerprint`、`dependency_fingerprint`、`tree_fingerprint`、`package:` 退路 | 新样本：抓它的测试经另一份文件里的函数调到被改坏的函数，那个中间函数改了而被改坏的函数没改——今天判 carry（`carry_decision` 只比本函数与抓它那条测试自己），改后必须重判；原有 Y4、Y8、OUT、T1 改写成单元口径照旧红 |
| U7 | `scripts/gate/verus-proof-check.py`（`check_proof`、`shared_components`、`vacuity_points`、`must_red_rows` 的键），可能还有 `crates/singlefs-replay-prefix/src/lib.rs`（按模块拆的那一种结局） | 先跑第九节的 Verus 三问，照结局取第四节两种设计之一 | 自证里的假 Verus（今天 `:1530` 一带按函数数造 JSON）加三格：改一个 proof fn 只重验那一项、别的复用；改它 `ensures` 里用到的 spec fn，用到它的几项都重验；各项 verified 之和不等于登记数判红 |
| U8 | `.claude/gate.d/checker-tier-qemu-device-streams.sh`、`scripts/devices/vm-bench.sh`（`run_one` 打包之前打出清单） | 每档一条判定，键照 5.1 那张表；删整道标记 | 预录样本档：改 `run_one` 里 `/init` 的 here-doc，六档都作废；只改一档的参数，只那一档作废；内核镜像换一个字节，六档都作废；改 E142 产物里不读的行，一档都不作废 |
| U9 | `.claude/scripts/lkmm.sh`（锚点文法与判法）、`.claude/gate.d/checker-tier-lkmm.sh`（按 litmus 存判定） | 锚点写「文件 :: 单元限定名」、判恰好一个单元；每份 litmus 一条判定，键照 5.2，不含锚点单元 | 样本：锚点写 `<impl T>::m` 判绿；同一限定名两处判红；只改 litmus 的普通注释，herd7 那一条复用；改 `singlefs-expect:`，现判；改锚点 fn 的函数体，herd7 那一条复用、绑定测试那一条作废 |

条目之间的依赖：U1 → U2 → U3、U5、U6；U4 → U5、U6、U8；U7、U9 只依赖 U1 / U4 的切法。底稿里别的机制（L5–L12 的块键、L14–L26 的各道门禁）用的都是 U2 / U4 交出来的键，本表不重复。

## 八、推翻条件（逐条结论各一句；能在这一轮造的已经造过）

| 结论 | 什么现象推翻它 | 这一轮造过没有 |
|---|---|---|
| 第三节规则不漏（夹具 13 格、V5、V11） | 一处真改了行为的改动，在它所在入口上键不变 | 造了反面：四条规则逐条拆掉，夹具各自红（2.3）；const 拆掉 V5、V11 漏（2.2）。没造：外部 crate 泛型回传工作区类型（3.4 末尾） |
| RTA 只多收不漏是对封闭程序 | 封闭入口里出现了一个运行时存在、名字却不在闭包里的工作区类型，它的方法被调到 | 开放入口那一格量到了只用 RTA 会漏（2.3），所以加了 R-开放 |
| 判器闭包里没有流水线 | `rta_fast` 判器闭包里出现流水线七份的单元，或判器真调了流水线的代码 | 量了：0 个（2.1） |
| 按单元比按 crate 少作废 | 同一批提交上按单元作废的条数不少于按 crate | 量了三个单文件提交（2.4）；只三个点 |
| 逐提交这一层按单元省不下判器重判 | 判器闭包所在文件只改了注释或闭包外的函数的提交 | 量了 14 个提交（2.4）；那一段全是大批量提交，换一段历史可能翻 |
| Verus 只能在根模块或单个函数上拆 | `--help` 之外还有别的部分验证入口 | 只读了 `--help` 与二进制串（第四节） |
| initramfs 的包不能拿来当键 | 同样内容、不同 mtime 打出来的 cpio 字节相同 | 量了：不同（5.1） |
| herd7 判定不读代码 | `herd7` 的输入里有 `crates/` 下的文件 | 读了 `lkmm.sh:366` 的起法：cwd 是内核树、参数只有 cfg 与 litmus |

## 九、要主 agent 跑的命令

Verus 三问（第四节）。它与门禁 checker-tier-formal-verification 的 proof 那一格做的是同一件事（起 Verus 验核 crate），按重型办、要不要跑问用户。工作目录：草稿目录里 `rsync -a crates/singlefs-replay-prefix/ <草稿>/verus-probe/` 得到的拷贝；内存 16G（`scripts/experiments/replay.sh:62` 的默认 `REPLAY_MEMORY_CAP` 是 8G；Verus 验整 crate 要多少没量，16G 是推的，按需改），线程 31。下面命令里的 `scripts/hosts/…` 指主仓根下那两份，在拷贝目录里起时写成主仓根的绝对路径。每条看 stdout 的 JSON 里 `verification-results` 的 `verified`、`errors`、`is-verifying-entire-crate`，与 stderr 的报错。

```
V=~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux
# 整 crate 对照（应得 verified 56、is-verifying-entire-crate true）
bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/hosts/capped.sh 31 $V/verus --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads 31 --output-json --rlimit 10 src/lib.rs
# 逐函数计数：对 verus! 块里每个 fn 名（第四节那 90 个，减去 spec fn 看要不要）各跑一次，把 verified 加起来与 56 比
bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/hosts/capped.sh 31 $V/verus --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads 31 --output-json --rlimit 10 --verify-root --verify-function <名> src/lib.rs
# 部分验证下的作弊检查：在拷贝里给 proof fn A 的函数体加一行 assume(false);，只验另一个函数 B，看报不报作弊
bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/hosts/capped.sh 31 $V/verus --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads 31 --output-json --rlimit 10 --verify-root --verify-function <B> src/lib.rs
# 撞名：名字是别人子串的那 4 个，replay_core 等，看是报「more than one match」还是只中一个
bash scripts/hosts/run-with-memory-cap.sh 16G bash scripts/hosts/capped.sh 31 $V/verus --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads 31 --output-json --rlimit 10 --verify-root --verify-function replay_core src/lib.rs
```

判结果：整 crate 对照的 verified 不是 56 先停（拷贝或参数不对）；逐函数计数的总和等于 56、部分验证下报作弊、撞名那几个都有写法只中一个，就走第四节「能按函数拆」那一种；部分验证下不报作弊，就另立整 crate 作弊扫描那一项；撞名的没有写法只中一个，就把撞名的几个函数改名或合并到同一项。

崩溃与变异两条轻型、门禁与重型：本路一条都没起，也没有要它们判的改动。

## 十、没做什么

- 主仓一个文件没改；没起 cargo、没起测试二进制、没跑门禁阶段；Verus 只跑了 `--help`（与对 `rust_verify` 的 `strings`），没验证过一个函数；QEMU、herd7 没起。
- 探针是 python 原型，不是第三节设计的实现：没做 cfg 求值、数据单元、拒算、WGSL、python 与 shell 的闭包（shell 只量了 `declare -f` 规范文本长什么样）；Rust 那一份也没认带路径的条目级宏调用与 `verus!` 的子句花括号（3.4 那 4 处）。
- 收益只在 3 个单文件提交与 14 个判器提交上量过，没扫更长的历史；按测试函数的那一组没算判器以外的入口（门禁的 python / shell 入口）。
- 底稿第三节第 5–8 问不归这一路，没答（第 4 问顺手答在第六节）。
- 第一轮五路报告的正文没读，只读了底稿；底稿里的行号没逐条复核，本报告引的行号都是在主仓当前文件上现取的。
- 产物没入库：探针源码、夹具、原样输出都在 `~/.local/share/singlefs/work/survey-pruning-r2-design/probe/`（356K），`verus --help` 原样在 `…/verus-help.txt`；本路只读，要不要拷进 `research/` 由主 agent 定。`probe/fixture/` 是自建的三 crate 小夹具（6 份文件），不是仓副本，留着给 U1 搬夹具用。
- 仓副本 `…/copy`（删前 `du -sh` 26M）与历史导出目录 `…/hist`（删前 4.0K，里面导出的树每次用完就删了）都已删。
- 跑的过程里有两条命令（`tests_history.py`、`rta_open.py`）超过 Bash 默认 120 秒被转到后台（我没给 Bash 设超时），两条都带自己的 `timeout`、都撞上上限退 143：前一条只留两个提交的读数，后一条没有读数；另有一条 `tests_history_fast2.py` 在前台撞上自带的 `timeout 230`，只出了 `40412d30` 那一行，`c34be3b4` 与 `c3e9d9ed` 之后各自单跑补上（2.4）。交回前核过没有残留进程（`ps` 里 `tests_history`、`rta_open` 都是 0 条）。
