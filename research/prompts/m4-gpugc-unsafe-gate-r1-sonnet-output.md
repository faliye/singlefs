# m4-gpugc-unsafe-gate-r1 正推腿（Sonnet）报告

立场：从判法源码（开工快照里的 `.claude/gate.d/code-source-discipline.sh`）与 `crates/` 今天的实现推到结论。引代码写开工快照那一份的行号（现查：判法、登记表与四份 `.rs` 此刻的 sha256 与 `research/prompts/m4-gpugc-unsafe-gate-r1-snapshot/judged-sha256.txt` 里的逐个相同，见「现查」一节）。样本的预测全是读源码推的；我不起判法，样本由主 agent 跑，跑出来与预测不符时两边都当数据。

## 复跑

- 复跑命令（整条交主 agent；轻型，一个样本一个进程组，十八个样本各起一次判法）：
  `task-server/task submit -- bash research/prompts/m4-gpugc-unsafe-gate-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>`
- 模型目录 `research/prompts/m4-gpugc-unsafe-gate-r1-sonnet-model/`，文件与 sha256（量的，`sha256sum`）：
  - `tools/rerun.sh`　b7cb0628bbaca71e06a5373f3d7c4b7640662ac2b98b57d4e3d46ff02d8b11d1（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只改 `rerun_steps`）
  - `tools/run_samples.py`　1b18fa9fce7dcfd2c24f8cf0d0055457e031424326fe2ef2956fb19f7066fe88（造样本、起判法、比预测；样本定义就在 `build_samples()` 里，一个样本一条 `add(…)`）
- 样本不进 `.claude/gate.d/fixtures/`：主工作区不改。要变成 red 样本的（下面标「放得过」的）可以直接把 `build_samples()` 里那一条的 `rows` 与 `files` 拷成样本目录，`expect` 反过来写。

## 现查（量的）

- 逐个相同（`sha256sum` 对 `judged-sha256.txt`）：判法 a054a21d…4871、登记表 09f73a31…2b95、`background_gpu.rs` ce061d83…、`lib.rs`（后台 GPU 包）161deb60…、E185 装置 3351e1c2…、E183 装置 66c283f3…；开工快照共 201 份（`grep -c .` 得 201）。
- `Cargo.lock` 里叫 `libc` 的包一个（`grep -nE '^name = "(rustix|nix|libc)"' Cargo.lock` 只命中第 360 行 `name = "libc"`）；没有 rustix、nix。本机 `~/.cargo/registry/cache` 里有 `rustix-1.1.4.crate`、`nix-0.31.3.crate` 这类压缩包（`ls` 命中，未解开，不在锁文件里，能否离线解析没核）。
- 工具链 `rustc 1.98.0`，仓里各包 `edition = "2021"`（`grep -n edition crates/*/Cargo.toml`）。

## Q1　判法：放过与误拒

判法的骨架（先读懂再看各问）：`strip_comments_and_strings` 抹注释与字符串（`.claude/gate.d/code-source-discipline.sh:2603` 起）→ 逐个 `.rs` 数 `unsafe` 关键字（「for match in UNSAFE_WORD.finditer(cleaned):」（`.claude/gate.d/code-source-discipline.sh:2953`））→ 把每处归到登记行（`registered_row_holding`，范围最小者）→ 总数（2995 行）、位置（3012 行）、逐行处数、SAFETY 往上找（`safety_comment`，2809 行起）。

### Q1.1　嵌套项、同名方法、trait 与固有 impl

结论：**没有找到放得过的写法；会误拒（键撞）。**

- 归哪一行（读 `items_of`，推的）：闭包、函数里的内部 fn、`macro_rules!` 体里的 unsafe，都不是登记表认得的项，落在包着它们的**登记了的**最小项里，算进那一行的第 5 段。内部 fn 的键是裸函数名（「is_method = innermost is not None and "type" in innermost and BREAK != "methods-keyed-by-name"」（`.claude/gate.d/code-source-discipline.sh:2782`）：最里层包着它的是函数而不是 impl，就不是方法）。样本 `inner-fn-and-closure-counted-in-the-outer-row-green`（预测绿，登记 2 处记在外层函数那一行）。
- 键撞（推的，样本 `same-method-name-in-two-inherent-impls-red`，预测红）：`impl Reader<u8>` 与 `impl Reader<u16>` 的 `read` 同键 `Reader::read`；项取「item = next((item for item in items_by_path[row["file"]] if item["key"] == row["function"]), None)」（`.claude/gate.d/code-source-discipline.sh:2986`）第一个，第二个里的 unsafe 落在范围外，判「不在登记的……函数体里」（3012 行 `if holder is None:`）。trait 名不进键，所以 `impl A for Foo` 与 `impl B for Foo` 的同名方法、固有 impl 与 trait impl 的同名方法同样撞；不同模块里的同名自由函数、同名类型也撞。结果是合规代码登不进去，**不是放过**。
- trait 定义里的默认方法按裸函数名认（`items_of` 里 impl 以外的函数都走 `function["name"]`），与同名自由函数撞。
- 登不进的项种类（推的）：`unsafe trait`、`unsafe extern` 块、`static X: T = unsafe { … };` 这类不在 impl 与 fn 里的 unsafe，没有键写法，只能误拒。
- 一个差一口气的形（推的，没做样本）：放开属性（`#[allow(unsafe_code)]`）只核「紧挨的项的键在登记里」（「following = next((item for item in items_by_path[path] if item["start"] == following_start), None)」（`.claude/gate.d/code-source-discipline.sh:2948`）），键撞时属性可以挂在同名的另一个项上而 unsafe 在登记的那一个里，判法绿。deny 根上 rustc 自己会拒（没有 allow 的项里写 unsafe）；`tests/` 里的文件是各自的 crate 根、没有 deny，rustc 不拦，判法是唯一的拦。推翻它的现象：造一个样本照此写法，判法红。

### Q1.2　SAFETY 按语句开头认

结论：**放得过三种；会误拒几种；match 分支不会认错。**

`statement-finished-between-red` 罩的是「注释与 unsafe 所在语句之间隔着一条以 `;` 收尾的语句」这一形（那份样本的 `expect` 行 `want=crates/singlefs-core/src/block_device.rs:13 的 unsafe 上方没有 // SAFETY: 注释`）。它没罩的：

1. **放得过：行分隔符错位**（推的，样本 `line-separator-shifts-the-safety-lookup-green`，预测绿；对照 `same-text-without-the-separator-red` 预测红）。「comment = safety_comment(files[path][0].splitlines(), files[path][1].split("\n"), line, skipped_by_path[path])」（`.claude/gate.d/code-source-discipline.sh:3036`）：原文按 `str.splitlines()` 切，抹过的按 `"\n"` 切。`splitlines()` 还在 `\x0b \x0c \x1c-\x1e \x85 U+2028 U+2029` 处断行（`python3 -c 'print("a b\x0cc".splitlines())'` 得 `['a', 'b', 'c']`，量的）。文件里出现一个（注释里放就行；Rust 的空白也认 `\x0c`、U+2028），之后每处 unsafe 看的「上一行」其实是再上一行：隔着一条语句的 SAFETY 就被认成这一处的。今天 `crates/**/*.rs` 里这类字符 0 个文件（量的，我的只读脚本数的）。补法：3036 行把 `splitlines()` 换成 `split("\n")`。
2. **放得过：字符串或块注释里写一行 `// SAFETY:`**（推的，样本 `safety-line-inside-a-string-green`，预测绿）。往上找注释读的是原文行（「if stripped.startswith("// SAFETY:"):」（`.claude/gate.d/code-source-discipline.sh:2822`）），不看那一行是不是真注释；字符串里、`/* … */` 里、原始字符串里恰好顶格的 `// SAFETY:` 都认。补法：`strip_comments_and_strings` 顺手记下真注释的行，往上找时只认这些行。
3. **按设计放过：一段 SAFETY 罩住续行链里的几处 unsafe**（样本 `one-comment-covers-the-second-unsafe-of-a-chain-green`，预测绿，登记 2 处）。`let total = unsafe{…} +` 换行 `unsafe{…};`：第二处的上一行以 `+` 结尾，被当续行（「while BREAK != "continuation-lines-ignored" and start > 0 and STATEMENT_CONTINUES.search(cleaned_lines[start - 1]):」（`.claude/gate.d/code-source-discipline.sh:2814`）），两处共用语句头上那一段注释。判法文档写的就是「每一处所在那条语句的开头一行」，所以这不是错，但「每一处有自己的 SAFETY」在这一形上不成立；要的话得改规矩，不是改实现。

误拒（合规写法被拒）：
- 方法链里的 unsafe 不在第一行（样本 `unsafe-below-the-first-line-of-a-method-chain-red`，预测红）：上一行以 `)` 结尾，不算续行，注释写在 `let` 上方判不到。
- 参数表里第二个 unsafe（上一行以 `,` 结尾）、续行链中间夹一行 `//` 注释（抹过之后那一行是空白，链断）、注释与语句之间隔空行：都红（读 2814–2824 行推的，没做样本）。

match 分支（推的，样本 `match-arms-each-with-its-own-comment-green` 预测绿、`match-arm-comment-written-for-the-previous-arm-red` 预测红）：`A =>` 换行 `unsafe{…},` 的 `=>` 以 `>` 结尾算续行，注释写在 `A =>` 上方即认；`0 => 0,` 换行 `_ => unsafe{…},` 上一行以 `,` 结尾，不续，写给上一个分支的注释**不会**认成这一处的。逗号不算续行的取舍成立。

### Q1.3　bin 根与别的根

结论：**bin 根的归属对 `src/bin/<名>.rs`、`src/bin/<名>/main.rs` 成立；Cargo.toml 里另指的路径判法不读，路径在 crate 目录之外的放得过。**

- 判法从不读 `Cargo.toml`（判这一格的 python 在 2486–3105 行之间 `tomllib` 零次出现，量的，`awk … | grep -c tomllib` 得 0）。bin 根只认两个文件名模式：「BIN_ROOT_FILE = re.compile(r"src/main\.rs|src/bin/[^/]+\.rs")」（`.claude/gate.d/code-source-discipline.sh:2687`）和 `BIN_ROOT_DIRECTORY_MAIN`（2688 行）。
- `src/bin/<名>.rs` 只管它自己一个文件。它用 `mod helper;` 引的 `src/bin/<名>/helper.rs` 不匹配两个模式，**归库根那一份**：这份文件里的 unsafe 对着库根（forbid）数，判红，登记表的 bin 根那一行也认不到它（样本 `bin-helper-file-goes-to-the-library-root-red`，预测红）。想在这种布局里放开只能把行登在库根名下，bin 根的 `#![deny]` 与 rustc 的实际级别不是一个根——误拒与登记错位，不是放过。`src/bin/<名>/main.rs` 的写法管整个目录，没有这个问题。
- `[[bin]] path = …`：路径在 crate 目录里、不符合上面两个模式的，归库根那一份（保守地多数）；路径在 crate 目录**外**的，`os.walk` 只走 `crates/<crate>/`（「subdirectories[:] = sorted(entry for entry in subdirectories if entry != "target" and not entry.startswith("."))」（`.claude/gate.d/code-source-discipline.sh:3062`）），整份不扫，**放得过**（推的，没做样本：样本得造出 crate 目录外的 `.rs` 再让 `Cargo.toml` 指它，判法不读 `Cargo.toml`，所以只能从读代码推）。今天各包 `Cargo.toml` 里 `path =` 只有包依赖与 `singlefs-unit-graph` 那一条标准路径（`grep -rn 'path *=' crates/*/Cargo.toml`，量的），仓里没有这样的实例。推翻它的现象：造一个 `[[bin]] path = "../x.rs"` 的 crate，判法对里面的 unsafe 红。
- `build.rs`（`singlefs-core`、`singlefs-checker-tier` 各一份，`ls` 量的）、`tests/`、`examples/`（`crates/singlefs-harness/examples` 有，`benches/` 全仓没有）、`src/` 下 bin 根以外的文件，全归库根那一份：库根写 forbid 就一处都不许，写 deny 就必须落在登记的项里。保守地多数，不放过。
- 没登记的 bin 根：`judge_root` 对它按 `levels = ["forbid"]`、期望放开 0 处、unsafe 0 处判（`judge_root` 起于 2933 行，`registered` 为假时走 2968 行 `if level is None and not registered:`）；`unsafe` 文本在里面就红。样本 `cfg-off-unsafe-is-counted-red`（预测红）用的就是它。仓里的实例：后台 GPU 包的 `src/bin/e202_gpu_cpu_gc_comparison.rs` 没登记，自己写 `#![forbid(unsafe_code)]`（30 行，量的 `grep -n 'unsafe\|#!\[' crates/singlefs-background-gpu/src/bin/*.rs`）。
- 没登记的 bin 根不要求写 `#![forbid]`，这时 rustc 没有 unsafe 的级别要求（`unsafe_code` 默认 allow），判法的文本计数是唯一的拦——下面 Q1.4 的放得过都落在这里。

### Q1.4　宏、条件编译、字符串与注释

- 注释与字符串里的 `unsafe`（含 `r#"…"#`、`b"…"`、嵌套块注释、`unsafe_code` 标识符）：**不认，对**（样本 `words-in-strings-and-comments-green`，预测绿）。
- `cfg` 关掉的分支里的 unsafe：**认（算数），对**——判法按文本数，cfg 一开就在；样本 `cfg-off-unsafe-is-counted-red` 预测红。
- 宏：**认定义处的文本一次，展开处不认**。样本 `macro-expands-three-unsafe-for-one-keyword-green`（登记 1 处，`macro_rules!` 写在登记的函数里、调三次，预测绿）：落在登记项里的宏定义只有一处 `unsafe`，运行时三处；一段 SAFETY 对三处同一前提成立，按「一处 = 一段文本」的规矩算数，我判**按设计**，不当漏洞。定义在登记项**外面**的宏，定义处就红。别的 crate 导出的宏展开出的 unsafe 本 crate 里一个字也看不到（`unsafe_code` 对外部宏本来也不报）；责任在宏所在的 crate，推的。
- **放得过（缺陷）：`cr"…"` 原始 C 字符串**（样本 `raw-c-string-hides-unsafe-green`，预测绿）。抹字符串时认原始字符串的条件是前一个字符不是字母数字、或是单独的 `b`（「or (text[index - 1] == "b" and (index == 1 or not (text[index - 2].isalnum() or text[index - 2] == "_")))」（`.claude/gate.d/code-source-discipline.sh:2624`）），`c` 不在里头。`cr"\"` 在 Rust 里是一个完整的原始 C 字符串（内容一个反斜杠），判法当普通字符串、把 `\"` 当转义往下吞，吞到下一个引号，中间的 `unsafe` 被抹掉。同一个仓里别处早认了这个写法（`crates/singlefs-checker-tier/src/gpu_unit_checks.rs:4018` 的 `matches!(word, "r" | "br" | "cr")`）。rustc 1.98 认 `cr"…"`（1.77 起）。补法：2624 行的 `b` 改成 `b` 或 `c`，`br` 与 `cr` 一并认。在登记了的库根里 rustc 的 forbid/deny 还会拦；落在没登记 bin 根、`tests/` 里没有第二道。
- **放得过（缺陷）：`include!` 或 `#[path = …]` 引进来的非 `.rs` 文件**（样本 `include-of-a-non-rs-file-green`，预测绿）。「if file_name.endswith(".rs"):」（`.claude/gate.d/code-source-discipline.sh:3064`）只读 `.rs`，`include!("x.inc")` 里的 unsafe 看不见。仓里 `#[path = …]` 现有的全是 `.rs`（`grep -rnE 'include!\(|#\[path *=' crates --include=*.rs` 只命中 `#[path = "../../singlefs-harness/tests/…/mod.rs"]` 这类，量的）。补法：判法扫到 `include!`、`include_str!` 以外的 `include!(…)` 与 `#[path = …]` 时，目标不是库里的 `.rs` 就红。
- **放得过（推的，没做样本进红）：不带 `unsafe` 关键字的 unsafe 属性。** 2021 版里 `#[no_mangle]`、`#[export_name]`、`#[link_section]` 不写 `unsafe`，rustc 的 `unsafe_code` 检查却报它们；判法数的是关键字（`UNSAFE_WORD`，2685 行）。样本 `no-mangle-without-the-keyword-green`（预测绿）。同样只在没登记 bin 根与 `tests/` 里没第二道。2024 版要 `unsafe(no_mangle)`，到那时关键字就在了。
- 放开属性的写法：`RELAX`（2677 行）认 `allow`/`expect`/`warn` 里带 `unsafe_code` 的，含 `cfg_attr` 里的；放开属性后面紧跟 `assume_specification` 的不数（「if re.match(r"\s*(?:pub(?:\s*\([^)]*\))?\s+)?assume_specification\b", cleaned[end:]):」（`.claude/gate.d/code-source-discipline.sh:2943`）），这是给 Verus 的豁免（核心层 `checksum.rs` 298、300 行两处就是）；它只让放开属性不数，`unsafe` 关键字仍数、仍要落在登记项里，所以拿它藏不了 unsafe，推的。

## Q2　登记：每一行是不是非放开不可

判法：理由栏说的「没有安全写法」逐条对 `Cargo.lock`（现查：只有 `libc`，没有 rustix、nix、`procfs` 一类）、std、对应的第三方库源码核；有替代的再看准不准、快不快、两台机器上在不在。「第二台」我连不上，凡写「第二台」的都是**复核不了**。

### 核心层（第 31、32 行）

- 第 31 行 `punch_hole_keeping_the_file_length`：**站得住**。`git diff HEAD -- .claude/gate.d/unsafe-allowances.tsv` 里 HEAD 就有这一行（旧六段版的 `-singlefs-core … punch_hole_keeping_the_file_length`），这一批只是改成八段（补处数与必提词）；理由栏里「rustix 在本机 cargo 缓存里」属实（上面「现查」：缓存里有 `rustix-1.1.4.crate`，不在锁文件里）；两条零 unsafe 的路用户在知情后弹窗仍选打洞，理由栏已写。
- 第 32 行 `crc32_castagnoli_on_route`：**站得住**，**新加**（HEAD 的登记表核心层只有第 31 行一行）。调带 `#[target_feature(enable = "sse4.2")]` 的函数要 unsafe：`crates/singlefs-core/src/checksum.rs` 的 `register_after_the_bytes` 是安全的 target_feature 函数（函数声明不带 unsafe），从没开这个特性的调用方调它仍要 `unsafe`；std 没有「先查再调」的封装。调用方 `crc32_castagnoli_on_route` 按运行时选路，没法自己也标 target_feature。

### 后台 GPU 包（第 33–37 行）

- 第 33 行 `largest_device_local_memory_heap`（2 处）：**站得住**。核了三处：wgpu 的适配器信息没有显存字段（`wgpu-types-30.0.1/src/adapter.rs` 的 `AdapterInfo`：name、vendor、device、device_type、device_pci_bus_id、driver、driver_info、backend、subgroup_min_size，量的）；wgpu-hal 的 Vulkan 适配器没有公开的内存堆读法（`wgpu-hal-30.0.1/src/vulkan/device.rs:846` 的 `fn get_memory_properties` 是私有的；`adapter.rs` 里 `grep -n 'MemoryHeap\|memory_heaps\|heap_size'` 零命中，公开的 `physical_device_capabilities` 交回的 `PhysicalDeviceProperties` 只有设备属性与扩展表，量的）；`ash-0.38.0+1.3.281/src/instance.rs:442` 的 `get_physical_device_memory_properties` 是 `pub unsafe fn`。`as_hal` 本身是 `pub unsafe fn`（`wgpu-30.0.1/src/api/adapter.rs:131`）。这两处没有 `unsafe` 的路：sysfs 的 `mem_info_vram_total` 只有 amdgpu 有、NVIDIA 没有，起 `nvidia-smi` 是外部命令且只认 NVIDIA；产品代码按厂商分路不合要求。
- 第 34 行 `host_cpu_time_of_this_thread`（1 处）：**站得住**。安全替代只有 `/proc/thread-self/schedstat` 第一栏或 `stat` 的 utime+stime：后者以 1/100 秒计（本机 `getconf CLK_TCK` 得 100，量的）、测一次提交的主机 CPU 时间（微秒级）不可用；前者在读的是自己时不经 `task_sched_runtime` 现算，会差到一个调度节拍（内核行为，推的，没在本机核）。产品里按次调用（`background_gpu.rs:1098`、`2571` 起多处），每次开文件也太贵。
- 第 35 行 `monotonic_raw_time_now`（1 处）：**站得住**。std 的 `Instant` 读 `CLOCK_MONOTONIC`，没有 `CLOCK_MONOTONIC_RAW` 的读法，锁文件里没有安全封装。出处栏的问题见下面「出处栏」。
- 第 36 行 `with_the_address_space_capped_beyond_the_present_size`（3 处）：**站得住**。要压的上限是「此刻 `/proc/self/status` 的 VmSize 加 32 MiB」，VmSize 在子进程里建完 noop 设备之后才知道，从外面用 `prlimit`、`ulimit -v` 起子进程时事先给不出这个数。三处里「放回原上限」那一处（`background_gpu.rs:4289` 起）子进程只跑这一条用例，不放回也不影响，理由栏没说；不放回可以少一处 unsafe，但「放回」让同一个函数在别的进程里调也安全，我判**站得住**，是取舍。
- 第 37 行 `free_device_local_bytes_of_the_card`（2 处，`tests/`）：**站得住**，同第 33 行，加一条：wgpu-hal 的 `heap_budget`、`heap_usage` 读法也是私有（`wgpu-hal-30.0.1/src/vulkan/device.rs` 的 `MemoryProperties` 方法没有 `pub`）。

### E185 装置（第 41–48 行）

锁文件里只有 `libc`（上面「现查」）：以下都在「已有依赖」这个口径下判。

- 第 41 行 `nanoseconds_of_the_cpu_clock`：**站得住**。线程 CPU 钟读到纳秒是登记第十二节 12.2 的前提（换钟的依据，源码注释 `e185…rs` 的 `CLOCK_IDENTIFIER_OF_THE_THREAD_CPU_TIME` 上方），`/proc` 的替代是节拍或调度节拍粒度。
- 第 42 行 `RawWallClockReading::now`：**站得住**。同第 35 行。产品包里有现成的 `monotonic_raw_time_now`，E185 装置另写一份是因为 checker 档不依赖产品 GPU 包（`crates/singlefs-checker-tier/Cargo.toml` 的依赖表里没有 `singlefs-background-gpu`），两边各一处 unsafe 是独立性的代价，不算多登。
- 第 43 行 `drop_the_page_cache_of_the_file`：**站得住（弱）**。理由栏漏了一条替代：GNU `dd of=<文件> oflag=nocache conv=notrunc count=0`（`count=0` 时对整个文件做 `POSIX_FADV_DONTNEED`，coreutils 文档的写法；本机 `dd (coreutils) 9.4`，量的）。代价：每次起一个进程（毫秒级，这个函数每次测量之前对工作件做一次）、多一个外部命令；仓里的 E185 已经起 `nvidia-smi` 与 `python3`。不准确的风险没有。我判**站得住**，因为换成外部命令是用一个新的运行时依赖换一处五行的 unsafe；要换的话理由栏该补这一条再决定。
- 第 44 行 `cpu_number_now`：**站不住**（推的）。理由栏说 `/proc/thread-self/stat` 第 39 栏「每读一次要多开一次文件、解析一整行」，而 `thread_and_machine_snapshot`（`e185…rs:4408`）在同一个函数里已经读了这份文件（`parsed_value_of_the_file(Path::new("/proc/thread-self/stat"), thread_counters_of_the_counter_record)`），解析助手 `counter_record_field(&fields, N)`（4311 行起）已有，第 39 栏就是 `counter_record_field(&fields, 39)`（本机 `awk '{print $39}' /proc/thread-self/stat` 得 7，52 栏，量的）。调用点 4434 行（快照）零额外开销；14143 行（`host_core_reading_now`）的调用后面紧跟两次 sysfs 读，多读一次 `stat` 在量级之外。写法：`ThreadCounters` 加一栏 `cpu_number`，常量 `COUNTER_RECORD_FIELD_PROCESSOR = 39`，14143 行读一次 `stat`；删 `cpu_number_now` 与它的 `#[allow]`、登记表第 44 行。准度与 `sched_getcpu` 同口径（都是「调用那一刻的核」）。代价：几十行与一条单测；快照里 cpu_number 与计数来自同一次读，反而一致。推翻它的现象：`stat` 的第 39 栏在某台机器上与 `sched_getcpu` 系统性不同，或快照函数的耗时本身落进计时窗（`around_the_intent` 在 4705–4707 行，快照在 `measure()` 之外）。
- 第 45 行 `clock_ticks_per_second`：**站不住**（推的）。理由栏说「写死 100 是替平台作假设」，但有精确的安全读法：glibc 的 `sysconf(_SC_CLK_TCK)` 就是内核交给进程的辅助向量 `AT_CLKTCK`，`/proc/self/auxv` 可读，16 字节一对（类型、值）取类型 17（本机 python 读 `/proc/self/auxv` 得 `AT_CLKTCK 100`，与 `getconf CLK_TCK` 的 100 相同，量的）。写法：`std::fs::read("/proc/self/auxv")` 按 `usize` 对切；读不到交回 `None`，与今天读不到的语义一致。这个值只印进读数（4738 行的 `clock_ticks_per_second={}`）与单测，不进判据。代价十几行与一条单测；第二台没核（复核不了）。
- 第 46 行 `HardwareCrc::crc`（2 处）：**站得住**。同第 32 行；`hardware_single_crc`、`hardware_three_way_crc` 是带 `#[target_feature(enable = "sse4.2")]` 的安全函数（`e185…rs:8720`、`8827` 附近），`crc` 这个调用方不能也标，因为它要照运行时选路。
- 第 47 行 `allowed_cores_of_this_thread`：**站不住**（推的）。`/proc/thread-self/status` 的 `Cpus_allowed_list:` 是同一个集合（`task->cpus_mask`，含 cpuset 的限制）；差别只有一条：`sched_getaffinity` 还和「活动核」求交，有核被下线时 `Cpus_allowed_list` 多出下线的核（内核行为，推的，没在本机核；本机 `/sys/devices/system/cpu/offline` 为空、`online` 与 `possible` 都是 `0-31`，量的）。补一次 `/sys/devices/system/cpu/online` 的读与求交就等价。代价：一个区间表解析（`0-3,8,10-11` 这一形，十几行）与两次小文件读；每条臂放线程时才调（14233、14247 行），不在计时窗里。第二台的下线核没核（复核不了）。
- 第 48 行 `pin_this_thread_to`：**站得住（弱）**。理由栏「std 也不给」线程号不实：`std::fs::read_link("/proc/thread-self")` 得 `<进程号>/task/<线程号>`（本机得 `1671626/task/1671626`，量的），再起 `taskset -cp <核> <线程号>`（util-linux，本机 `/usr/bin/taskset` 在；第二台没核）。代价：起一个外部进程、依赖 `taskset` 在；效果同 `sched_setaffinity`。结论仍判站得住，因为替代是外部命令，不是安全 Rust；理由栏里「std 也不给」应改成「线程号要读 `/proc/thread-self`，且 `taskset` 是外部命令」。

### E183 装置（第 53–57 行）：只判登记与源码对不对得上

对得上（量的，`grep -n 'unsafe\|allow(' e183…rs`）：`unsafe impl GlobalAlloc for CountingAllocator` 217 行 1 处；`alloc`（223、225）、`alloc_zeroed`（237、239）、`dealloc`（251、253）、`realloc`（263、265）各 2 处，合计 9 处，与第 53 行 1 处加 54–57 行各 2 处相符；每处放开属性一个（209、218、232、246、257 行起）；第 6 段必提词逐个在对应 SAFETY 注释里（`System`、`原子`、`线程本地`、`析构` 在 213–216 行；各方法的 `layout`/`pointer`/`new_size` 在 222、224、236、238、250、252、261–262、264 行）。理由栏引的判决话（`research/prompts/m3-close-b19-reason-r3-main-verification.md:39` 的「E183 离不开（三份一致）」）逐字在。不重开甲、丙。

### 出处栏（登记行对不对得上记录）

- 第 33、34、36、37 行写「用户 2026-11-03 弹窗定 records/2026-10-19-里程碑四GPU垃圾回收开工.md」：记录里写到这件事的是第 761 行（用户定「改门禁，一处一行」，没带日期）；`grep -n '2026-11-03' records/2026-10-19-里程碑四GPU垃圾回收开工.md` 只命中第 779 行（别的事）。门禁只核文件在不在（`os.path.isfile`），日期对不对门禁不判。
- 第 35 行写「用户 2026-11-05 弹窗定 records/2026-10-19-里程碑四GPU垃圾回收开工.md」：`grep -c '2026-11-05' records/2026-10-19-里程碑四GPU垃圾回收开工.md` 得 0；这条决定在「挂钟改读不受 NTP 调频的单调钟」（`research/prompts/e202-preregistration.md:473`，里面写「用户 2026-11-05 弹窗定」）。出处路径要么改指那份登记，要么在记录里补上这一天（这一批的记录还没写，第 798 行是现有最后一行；推的）。
- 第 41–48 行出处栏只写记录路径，没带「用户……弹窗定」，门禁允许（只写路径也认）。

## Q3　SAFETY 写的前提成不成立

逐处读注释、对着代码核（行号是开工快照那一份里的）。**结论：一处写了、字面上不严格成立（无 UB 后果）；一处措辞差一行；其余成立。** 绑核两处现在都提到那条编译期断言。

| 处 | 文件:行 | 结论 | 依据 |
|---|---|---|---|
| 打洞 | `crates/singlefs-core/src/block_device.rs:745-750` | 成立 | 「越界的请求到不了这里」：两个调用方 `write_zeroes_at`（`block_device.rs:883`）、`discard_at`（969 行）都先 `check_request`（845、940 行），长度 0 先交回（854、942 行）；`file` 借用到返回 |
| 硬件 CRC（核心层） | `crates/singlefs-core/src/checksum.rs:134-136` | 成立 | `CrcInstructionDetected` 只在 79 行一处构造，前面是 `is_x86_feature_detected!("sse4.2")`（78 行）；字段私有只对包外成立，包内（含该文件的测试模块）能构造，今天没有第二处（`grep -n '_constructed_only_after' checksum.rs` 只命中 70、79 行） |
| 读内存堆（一） | `crates/singlefs-background-gpu/src/background_gpu.rs:861-862` | 成立 | 守卫是局部变量，函数结束丢；只读 |
| 读内存堆（二） | `background_gpu.rs:864-866` | 成立，缺半句 | 物理设备句柄取自同一适配器；依赖「守卫 `hal_adapter` 此刻还没丢（实例没销毁）」没写，函数作用域保证它成立；补法：864 行注释加「守卫到函数末尾才丢」 |
| 线程 CPU 钟 | `background_gpu.rs:2418-2420` | 成立 | 栈上初始化的 `timespec`，内核只写它 |
| 单调原始钟 | `background_gpu.rs:2447-2449` | 成立 | 同上 |
| `getrlimit` | `background_gpu.rs:4278-4279` | 成立 | 同上 |
| `setrlimit`（压） | `background_gpu.rs:4285-4286` | **写了的前提不严格成立** | 注释说「软上限不超过原来的硬上限」；代码 `rlim_cur = VmSize*1024 + room`、`rlim_max = limit_before.rlim_max`（4283 行），硬上限是有限值且小于这个和时内核回 `EINVAL`，随后 `assert_eq!(cap_status, 0, …)` 红，**不是 UB**。补法二选一：4285 行改成「硬上限原样带回；软上限超过它时内核回 EINVAL、下面的断言红」；或 4282 行取 `min(…, limit_before.rlim_max)` |
| `setrlimit`（放回） | `background_gpu.rs:4289-4290` | 成立 | 放回的就是读到的那一份 |
| 真卡测试（一） | `crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:295-296` | 成立 | 同读内存堆（一） |
| 真卡测试（二） | 同文件 `308-310` | 成立 | 注释说「上面查过实例与物理设备都到 Vulkan 1.1、物理设备支持 `VK_EXT_memory_budget`」，299–304 行的 `if … return None` 逐条查了（实例版本、设备版本、`supports_extension`），`push_next` 的链是安全 API |
| 线程/进程 CPU 钟（E185） | `crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:644-646` | 成立 | 注释「clock id 不合法时内核交回 −1、不写内存」属实 |
| 原始单调钟（E185） | 同文件 `675-677` | 成立 | 同上 |
| 扔页缓存 | 同文件 `3717-3719` | 成立，措辞差一行 | 注释写「文件描述符来自上一行打开」，上一行是 `file.sync_all()?;`（3716），`File::open` 在 3715；描述符借自活着的 `file` 属实。补法：3717 行「上一行」改「上面」 |
| `sched_getcpu` | 同文件 `4380-4381` | 成立 | 无参数、不读写内存 |
| `sysconf` | 同文件 `4390-4391` | 成立 | 常量 `libc::_SC_CLK_TCK`（4278 行） |
| 硬件 CRC（E185，单路、三路） | 同文件 `8891-8892`、`8893-8895` | 成立 | `HardwareCrc` 只在 8879 行一处构造（`is_x86_feature_detected!("sse4.2")` 之后），字段 `shifts` 包内私有；两个被调函数都只要 `sse4.2`（8720、8827 行） |
| `sched_getaffinity` | 同文件 `14160-14162` | 成立 | 128 字节 = `size_of_val(&mask)`（长度照实传，glibc 只写到 `cpusetsize`）；`cpu_set_t` 大小与对齐由 14128 行 `const _: () = assert!(` 编译期钉；进程号 0 是调用线程 |
| `sched_setaffinity` | 同文件 `14190-14192` | 成立 | 同上，只读 |
| 计数分配器 `unsafe impl` | `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:213-217` | 成立（线程本地那一半补上了） | 现在写：线程本地两份是常量初始化、没有析构的 `Cell<i64>`，分配器里碰它不再分配、线程退出无析构可跑（124–130 行的 `thread_local!` 用 `const { Cell::new(0) }`，类型 `Cell<i64>` 无 Drop，量的）；`with` 对这类量不会因「已销毁」而 panic。没写的一条前提：`GlobalAlloc` 的实现不许 unwind；分配器里能 panic 的只剩 `record_growth` 的 `fetch_add(…) + bytes`（debug 构建溢出），要在用字节逼近 2^64，不可达（推的）；补法：216 行后加半句「不 unwind」 |
| 四个方法、八处 | 同文件 `222-225`、`236-239`、`250-253`、`261-265` | 成立 | 原样转给 `System`，契约不变；`dealloc`/`realloc` 的 `pointer`、`layout` 来源写了 |

### 同名问题：E183 注释说的「计数是原子的」那一半

判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md:39` 指出旧注释只对一半。现在 213–216 行两份各写了：全进程原子量；线程本地常量初始化无析构。登记第 6 段的四个词（`System`、`原子`、`线程本地`、`析构`）都在这四行里。**对得上**。

## 要主 agent 跑的命令

1. 样本（轻型，十八个样本各起一次这一格；一个样本一个进程组，结束时对整组发信号并核没有留下进程；输出写进 `--work`）：
   `SINGLEFS_RUN_SCOPE='种类=验证; 跑什么=unsafe-allowances 格的十八个最小样本; 范围=只读仓里的判法、样本全写在草稿目录; 为什么=核正推腿对判法放过与误拒的预测' task-server/task submit -- bash research/prompts/m4-gpugc-unsafe-gate-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-unsafe-gate-r1-sonnet-rerun-1`
   在仓根起；读结局：`task-server/task wait <号>`，再看 `<work>/samples-verdicts.txt` 的末行 `name=samples_summary … prediction_fails=…`；`prediction_fails` 不为 0 的样本，我的预测错了，以实测为准。
2. 真仓（已有读数：任务 6915 退 0），不需要重跑。
3. 没有重型命令。

十八个样本与预测（推的，读 `code-source-discipline.sh` 推出）：

| 样本 | 问 | 预测 |
|---|---|---|
| control-registered-green、control-statement-between-red | 校准 | 绿、红 |
| inner-fn-and-closure-counted-in-the-outer-row-green | Q1.1 | 绿 |
| same-method-name-in-two-inherent-impls-red | Q1.1 | 红（「不在登记的」） |
| line-separator-shifts-the-safety-lookup-green、same-text-without-the-separator-red | Q1.2 | 绿、红 |
| safety-line-inside-a-string-green | Q1.2 | 绿 |
| one-comment-covers-the-second-unsafe-of-a-chain-green | Q1.2 | 绿 |
| unsafe-below-the-first-line-of-a-method-chain-red | Q1.2 | 红 |
| match-arms-each-with-its-own-comment-green、match-arm-comment-written-for-the-previous-arm-red | Q1.2 | 绿、红 |
| bin-helper-file-goes-to-the-library-root-red | Q1.3 | 红 |
| include-of-a-non-rs-file-green、raw-c-string-hides-unsafe-green、no-mangle-without-the-keyword-green | Q1.4 | 绿、绿、绿 |
| cfg-off-unsafe-is-counted-red、words-in-strings-and-comments-green | Q1.4 | 红、绿 |
| macro-expands-three-unsafe-for-one-keyword-green | Q1.4 | 绿 |

## 什么现象会推翻

- Q1 放得过的四条（行分隔符错位、字符串里的假 `// SAFETY:`、`cr"…"`、`include!` 非 `.rs`）：样本档判红，即推翻。预测写在上面的表里，样本档给出实测。
- Q1.1 键撞只误拒不放过：造出一个键撞而判法绿且 unsafe 落在登记之外的样本，即推翻。
- Q2 第 44、45、47 行「站不住」：写出替代、替代在 E185 的单测里与 `sched_getcpu`／`sysconf`／`sched_getaffinity` 的读数不同，即推翻；第二台上 `/proc/self/auxv` 读不到或 `Cpus_allowed_list` 与 `sched_getaffinity` 不同，也推翻（我没连第二台）。
- Q3 第 4285 行「前提写了不严格成立」：本机或第二台的 `RLIMIT_AS` 硬上限恒为无限，则只剩措辞问题，不改结论的严重度（本来也不是 UB）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1.1 嵌套项归属 | 推的：归最小的登记项 | 闭包、内部 fn、宏体里的 unsafe 记在外层登记行的处数里；内部 fn 键是裸名 |
| Q1.1 键撞 | 放不过（误拒） | 同名方法（泛型实参不同的固有 impl、不同 trait、不同模块）撞键，第二个登不进；`unsafe trait`、`unsafe extern`、`static` 初值里的 unsafe 无键可登 |
| Q1.2 隔着语句 | 已罩（既有样本） | 隔着 `;` 收尾的语句判红；没罩的见下三行 |
| Q1.2 行分隔符错位 | 放得过（推的，样本待跑） | 文件里一个 U+2028 或 `\x0c`，之后 `splitlines()` 与 `split("\n")` 差一行，隔一条语句的 SAFETY 被认；`code-source-discipline.sh:3036` |
| Q1.2 字符串里的假注释 | 放得过（推的，样本待跑） | 往上找注释读原文行，不看是不是真注释；`code-source-discipline.sh:2822` |
| Q1.2 续行链共用一段 SAFETY | 按设计放过 | 链里第二处 unsafe 不必有自己的注释 |
| Q1.2 方法链、参数表第二处 | 放不过（误拒，合规写法） | 上一行以 `)`、`,` 结尾不算续行 |
| Q1.2 match 分支 | 不会认错 | `,` 不算续行的取舍成立；`=>` 算续行也对 |
| Q1.3 bin 根两种写法 | 成立 | `src/bin/<名>.rs` 管自己，`src/bin/<名>/main.rs` 管目录；前者旁的 `<名>/` 目录里的文件归库根 |
| Q1.3 `[[bin]] path`、`build.rs`、`tests/`、`examples/` | 目录内归库根（保守）；目录外放得过（推的，仓里无实例） | 判法不读 `Cargo.toml` |
| Q1.3 没登记的 bin 根 | 放不过 | 任何 `unsafe` 文本红；仓里 `e202…rs` 自己写 forbid |
| Q1.4 注释、字符串、cfg 关掉的分支 | 对 | 注释字符串不认，cfg 关掉的分支认 |
| Q1.4 宏 | 按设计 | 定义处算一次，展开处不认 |
| Q1.4 `cr"…"` | 放得过（推的，样本待跑） | `code-source-discipline.sh:2624` 不认 `c` 前缀，同仓 `gpu_unit_checks.rs:4018` 认 |
| Q1.4 `include!`／`#[path]` 非 `.rs` | 放得过（推的，样本待跑） | `code-source-discipline.sh:3064` 只读 `.rs` |
| Q1.4 不带关键字的 unsafe 属性 | 放得过（推的） | 2021 版 `#[no_mangle]` 不带 `unsafe`；只在没登记 bin 根与 `tests/` 里无第二道 |
| Q2 第 31、32 行（核心层） | 站得住 | 第 31 行 HEAD 就有；第 32 行是这一批新加 |
| Q2 第 33、34、35、36、37 行（后台 GPU 包） | 站得住 | 第 35 行出处栏对不上记录 |
| Q2 第 41、42、46 行（E185） | 站得住 | |
| Q2 第 43、48 行（E185） | 站得住（弱） | 各有外部命令的替代（`dd`、`taskset`）；第 48 行理由栏「std 也不给线程号」不实 |
| Q2 第 44 行 `cpu_number_now` | 站不住（推的） | 同函数已读 `/proc/thread-self/stat`，第 39 栏现成可取 |
| Q2 第 45 行 `clock_ticks_per_second` | 站不住（推的） | 读 `/proc/self/auxv` 的 `AT_CLKTCK`，与 `sysconf` 同源 |
| Q2 第 47 行 `allowed_cores_of_this_thread` | 站不住（推的，第二台没核） | `Cpus_allowed_list` 与 `/sys/devices/system/cpu/online` 求交 |
| Q2 第 53–57 行（E183） | 对得上 | 处数 9、必提词、判决话、放开属性逐个核过 |
| Q3 SAFETY 逐处（登记表第 31–57 行里放开了的 20 行，第 5 段之和 29 处 `unsafe`） | 成立（除下两行） | 逐处见 Q3 表；数是 `awk` 对登记表第 5 段求和量的 |
| Q3 `background_gpu.rs:4285` | 前提不严格成立（无 UB） | 补法在 Q3 表 |
| Q3 `e185…rs:3717` | 措辞差一行 | 「上一行」改「上面」 |

## 没做什么

- 没起判法、没跑样本、没编译：样本预测全是读 `code-source-discipline.sh` 推的，「放得过」四条在样本档跑出来之前都是推的；`tools/run_samples.py` 里只造样本目录的那一段我在草稿目录跑过一遍确认文件写得出（读出的 `.tsv` 与 `e900_x.rs` 内容与设计一致），没起判法。
- 没连第二台：凡写「第二台」的（`/proc/self/auxv`、`taskset`、`dd`、`Cpus_allowed_list` 与下线核）都是复核不了。
- 没读 `.claude/gate.d/fixtures/` 以外的样本档运行结果；没重核任务 5954–5973 的读数、没重开 E183 的甲丙之争。
- 没判 Q2 里替换后的 E185 装置会不会改变它的读数（只读了调用点在计时窗之外这一层）；`crates/mutations.tsv` 里没有点名这三个函数（`grep -c` 得 0），改的时候变异表与 `unsafe-allowances.tsv` 三行同批改。
- 不替主 agent采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没读攻方腿的报告与模型（禁读清单）。
- 清理：草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-unsafe-gate-r1-sonnet` 里造样本目录的 `dry/`（`du -sh` 约 0.9 MB）已删，目录现 12K，只剩认领与写者锁文件；没有仓副本、没有编译目录、没有起过进程。
