# verification-split-r1 云端攻方腿（Opus）报告

- 轮名 verification-split-r1；攻击面 Q1、Q2、Q4、Q6、Q9，另核正文第二节「54 号快档红 = C577 之后钉值没改」的归因。
- 判的是开工快照 `refs/sop/verification-split-r1-snapshot`（fe0bbf13），用 `git archive` 解到草稿目录再读、再跑；工作区一律不读。
- 报告写于 2026-09-27 12:0x–12:3x UTC（21:0x–21:3x JST）。
- ⚠️ 报告写的时候，工作区有 8 份被引文件已经和快照不一样了（别的会话在改）：`.claude/gate.d/94-checker-implementation-disjoint.sh`（工作区里 `dev-` 已经加回去了），`crates/singlefs-checker/tests/first_transaction_step_seven_layer0.rs`、`record_checker_judges_absence_by_the_persisted_set.rs`、`crates/singlefs-checker/Cargo.toml`、harness 的 `e161_…rs`、`history.rs`、`crash_enumeration_resumes_from_its_progress_file.rs`、`crash_segments_per_device_and_torn_in_place_overwrites.rs`。
  这几份在下文只放在标着「快照」的代码块里整行抄，不写成「」引文；「」引文只给交回前现查过、还和快照逐字节相同的文件。

## 复跑

模型目录 `research/prompts/verification-split-r1-opus-model/`。在仓根依次跑：

```
bash research/prompts/verification-split-r1-opus-model/setup.sh /tmp/claude-1000/three-way-attack/verification-split-r1/snap2
python3 research/prompts/verification-split-r1-opus-model/q1_probe.py /tmp/claude-1000/three-way-attack/verification-split-r1/snap2
bash research/prompts/verification-split-r1-opus-model/q1_guard.sh /tmp/claude-1000/three-way-attack/verification-split-r1/snap2
bash research/prompts/verification-split-r1-opus-model/toy_cargo_semantics.sh /tmp/claude-1000/three-way-attack/verification-split-r1/toyws2
bash research/prompts/verification-split-r1-opus-model/q4_gate94.sh /tmp/claude-1000/three-way-attack/verification-split-r1/snap2 /tmp/claude-1000/three-way-attack/verification-split-r1/q4-rerun
bash research/prompts/verification-split-r1-opus-model/q6_fixture.sh /tmp/claude-1000/three-way-attack/verification-split-r1/snap2 /tmp/claude-1000/three-way-attack/verification-split-r1/q6-rerun
python3 research/prompts/verification-split-r1-opus-model/q6_indirect.py /tmp/claude-1000/three-way-attack/verification-split-r1/snap2
bash research/prompts/verification-split-r1-opus-model/c577_attribution.sh /tmp/claude-1000/three-way-attack/verification-split-r1/snap2 /tmp/claude-1000/three-way-attack/verification-split-r1/c577-rerun
```

复跑要求：
- 要编译的只有玩具工作区与 `c577_attribution.sh`。后者编三份副本，跑的只是 harness 档：它自己的原型 `zz_attack_first_stream_segment_counts`，加四个仓里已有的 harness 测试目标，一律经内存包装与 `capped.sh 16`。
- 名字带 layer0 的目标、checker 包的测试一条都不跑。
- 原型不枚举崩溃状态，只录流、算闭式数。

```
fa79419fb1c23a4719314bf60b3097a20da3f278b8664ce8c429457b36cbc894  c577_attribution.sh
170bc0e97816281370964354fcc10d0eeb97ce261410fdb864275fef9929d822  q1_guard.sh
cc96e3e77cd47dfd5374a422c3312c1f5872bf65765d8b2e2aeb14e3c5194a29  q1_probe.py
2ad5d9d0d3a032a34ad50b7e1e8f9f4aa7bbcf6968f031223b4c88c71f0b9275  q4_gate94.sh
6cfa4ad513274f4f5a81c794e1fbab8d1aa48de5d50e28d70148f940944ec84b  q6_fixture.sh
7981221aec23c81210effd55653c084ea1dc2de6b9ff3c719fcf22dce4ad966d  q6_indirect.py
5d4a7e981eb033565863ba3eb14c73a9b8f464f84c0c29b7b57e27c12cedec9a  setup.sh
bf5415b1d96a6ba3aed07347730e66e5f78b49a23a04d126b5b15d55a9b02ff9  toy_cargo_semantics.sh
ac60b5ff63a55bc1524f8b6cf0ca0c54f3575b6b7fa8367a8a30705702fb2f45  zz_attack_first_stream_segment_counts.rs
```

## 各格判定一览

| 格 | 判定 | 一句话 | 量的还是推的 |
|---|---|---|---|
| Q1 | **红** | `-p 'singlefs-check*'`、`-p singlefs-checker@0.1.0`、`-p path+file://…#singlefs-checker@0.1.0`，以及在 harness 目录里用 `CARGO_ALIAS_XT='test -p singlefs-checker' cargo xt`，`classify` 都交 None。快照上的 heavy-test-guard.sh 对主 agent 与子 agent 都放行（退 0）。在玩具工作区上量过：这几种写法 cargo 都真会跑到被点名包的集成测试 | 量过 |
| Q1-附 | 红（旧缺口） | 仓里 `.cargo/mutants.toml` 写着 `test_workspace = true`，`cargo mutants -p singlefs-core` 与 `--test-package singlefs-checker` 都交 None | classify 那一半量过；cargo-mutants 会跑整个工作区的测试是推的（依据是二进制里的帮助串） |
| Q2 | 绿 | 三个共用模块里：读环境的只有 `std::env::temp_dir()` 与 `std::process::id()`，镜像名里带 pid；没有 `env!`、`CARGO_*`、`include*!`、`file!()`、cfg 分支；`crate::common` 在两个用到它的 checker 二进制里都指同一份 `#[path]` 模块 | 读源码 |
| Q4 | **红** | 94 号不读 `[dev-dependencies]` 之后，dev-dependency 改名引进来的实现（`implementation = { package = "singlefs-core" }`）在 checker 库的 `#[cfg(test)]` 块里用，94 号判绿；HEAD 那一份 94 号对同一个样本判红（别名那一条）。doctest 与 `#[path]`/`include!` 抄实现源码，94 号 ② 也判绿。D13 已定项 5 的射程句与 94 号文件头都只说了 `tests/`，没说库单测与 doctest | 94 号的判定量过；dev-dependency 进库单测与 doctest 在玩具工作区上量过 |
| Q6 | **红** | 真仓里 harness 包有 17 条测试经同一文件里的 helper 调 `enumerate_layer0*`（没豁免）；checker 包里登记的 8 条崩溃枚举用例，crash-case-check 只认出 3 条。脚本射程句只写了「tests/<子目录>/ 里的共用函数」与宏，没写同文件 helper、自写枚举函数（c561 那条 262144 个状态）、改名 import、单行 `#[test] fn`、属性值里带 `]`。豁免注明的理由不核：这一轮新加的那条豁免写的是「几步的合成流」，实际用的是第一条真流 | 量过 |
| Q9 | 红（小） | 11 个文件名 grep 下来，功能性引用都改了。还指旧路径的有两处 harness 注释：`history.rs:2102`、`e161…rs:316` 写 `tests/first_transaction_step_seven_layer0.rs`，是相对 harness 包的路径。e161 里 5 处行号引用因为插了一行 `#[path]` 而错开 1 行。另外正文判绿观测写的 `lib_heavy_tests.py:916`，真实行号是 959，同类样本还有 6 行 | grep 量过 |
| 归因 | **部分成立，射程比正文说的宽** | 在副本上只把 `transaction.rs:1208` 那道 C577 屏障换成 `Ok(())`，第一条流就回到钉值：`[2,2,1,2,2,1,26,2,1,2]`、67108885、快档 54、全量 150994980。快照上是 `[2,2,1,2,2,1,2,24,2,1,2]`、16777240、46、16777260。但是：① harness 档同样红，4 个 harness 测试目标里 8 条，去掉屏障就绿；② checker 那条 `removing_the_barrier…` 钉的 `[…,26,3,2]` 在 C577 之前、在它被钉进去的那次提交 45d49aaf 上量出来都是 `[…,26,5]`，C577 解释不了它 | 量过（副本上的数，不是入库装置上的数） |

## Q1 闸按包判有没有漏

### 取样

快照树的 `lib_heavy_tests.heavy_tests_in_command_text` 上跑了 56 种写法（`q1_probe.py`）。其中 35 种判 checker 档：
- `-p`、`--package=`、`-psinglefs-checker`、`-p=`；
- 当前目录在包里、在包的 `tests/` 里；
- `--manifest-path`，包括从 `/tmp` 起、带 `=` 的写法；
- `--test <名>`、`--tests`、`--all-targets`、`--test '*'`；
- nextest（`-p` 与在包目录里裸跑）；
- 两个 `-p`；
- 直接执行 release、debug 下的二进制，以及从仓外起、绝对路径的；
- 经 `bash -c`、`env`、`nice`、`capped.sh`、`run-with-memory-cap.sh` 套 `capped.sh`、`rustup run` 包装的；
- `cd … &&`、子 shell `( cd …)`、`-C`；
- `cargo hack`、`llvm-cov`、`miri`、`cargo mutants -p singlefs-checker`、`cargo t`；
- `--lib --tests`。

### 打中：返回 None 的，与会跑到 checker 包集成测试的真 cargo 对照

`q1_probe.py` 原样输出（这一次重跑，只摘返回 None 与非 checker 档的行；全表在草稿目录 `rerun/q1_probe.out`，56 行）：

```
12 --config alias from root	cargo --config 'alias.xt="test -p singlefs-checker"' xt	cwd=<snap>	=> [('full-cargo', '全量测试')]
22 -p glob	cargo test -p 'singlefs-check*'	cwd=<snap>	=> None
23 -p spec with version	cargo test -p singlefs-checker@0.1.0	cwd=<snap>	=> None
24 -p pkgid url	cargo test -p 'path+file://<snap>/crates/singlefs-checker#singlefs-checker@0.1.0'	cwd=<snap>	=> None
25 --config alias from harness dir	cargo --config 'alias.xt="test -p singlefs-checker"' xt	cwd=<snap>/crates/singlefs-harness	=> None
26 CARGO_ALIAS env from harness dir	CARGO_ALIAS_XT='test -p singlefs-checker' cargo xt	cwd=<snap>/crates/singlefs-harness	=> None
27 --config alias from core dir	cargo --config 'alias.xt="test -p singlefs-checker"' xt	cwd=<snap>/crates/singlefs-core	=> None
28 --workspace	cargo test --workspace	cwd=<snap>	=> [('full-cargo', '全量测试')]
35 nextest archive run from harness	cargo nextest run --archive-file /tmp/a.tar.zst	cwd=<snap>/crates/singlefs-harness	=> None
38 binary via find -exec	find target -name 'second_transaction_step_zero_layer0-*' -type f -executable -exec {} \;	cwd=<snap>	=> None
39 binary via $(ls)	$(ls target/release/deps/second_transaction_step_zero_layer0-* | head -1)	cwd=<snap>	=> None
40 binary copied renamed	cp target/release/deps/second_transaction_step_zero_layer0-0123456789abcdef /tmp/t && /tmp/t	cwd=<snap>	=> None
41 xargs	ls target/release/deps/second_transaction_step_zero_layer0-* | xargs -I{} {}	cwd=<snap>	=> None
51 cd to var dir	D=crates/singlefs-checker; cd $D && cargo test	cwd=<snap>	=> None
54 env -C	env -C crates/singlefs-checker cargo test	cwd=<snap>	=> [('full-cargo', '全量测试')]
57 eval	eval 'cargo test -p singlefs-checker'	cwd=<snap>	=> None
59 variable pkg	P=singlefs-checker; cargo test -p $P	cwd=<snap>	=> None
M1 mutants -p core (mutants.toml test_workspace=true)	cargo mutants -p singlefs-core	cwd=<snap>	=> None
M2 mutants --test-package checker	cargo mutants -p singlefs-core --test-package singlefs-checker	cwd=<snap>	=> None
X1 harness-only target from root	cargo test --test crash_segments_per_device_and_torn_in_place_overwrites	cwd=<snap>	=> [('checker-tier-cargo', 'checker 档')]
```

cargo 真的会跑到被点名包的集成测试吗？在玩具工作区（toy-core、toy-harness、toy-checker，与仓无关）上量的，`toy_cargo_semantics.sh` 原样输出节选：

```
== cargo test --offline -p toy-check* -- --nocapture
MARKER-CHECKER-INTEGRATION-RAN
== cargo test --offline -p toy-checker@0.1.0 -- --nocapture
MARKER-CHECKER-INTEGRATION-RAN
== cargo test --offline -p path+file:///tmp/claude-1000/three-way-attack/verification-split-r1/toyws2/crates/toy-checker#toy-checker@0.1.0 -- --nocapture
MARKER-CHECKER-INTEGRATION-RAN
== cargo --config alias.xt="test -p toy-checker" xt --offline -- --nocapture
error: no such command: `xt`
rc=101
== cargo xt --offline -- --nocapture
MARKER-CHECKER-INTEGRATION-RAN
== cargo bench --offline -p toy-checker --test big_stream -- --nocapture
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

（上面 `== cargo xt` 那一行，是在 `crates/toy-harness` 里带 `CARGO_ALIAS_XT='test -p toy-checker'` 起的。heavy-test-guard 那段输出的最后一行被 `cut -c1-120` 截在多字节字符中间，原样末尾是一个坏字节，这里写成 `…`。）

读法（cargo 1.98.0）：
- 22、23、24、26 是真漏，cargo 会跑到被点名包的测试。
- `--config alias.<名>` 在这版 cargo 上根本不生效（`no such command`），所以 12、25、27 是假警报，不算漏。
- `cargo bench --test` 把 `#[test]` 记成 ignored，不跑，也不算漏。

在快照的 heavy-test-guard.sh 上真判一遍，`q1_guard.sh` 原样输出：

```
exit=0 agent=three-way-attack cwd= [bash research/scripts/run-with-memory-cap.sh 8G cargo test -p 'singlefs-check*']
exit=0 agent=implementation-writer cwd= [bash research/scripts/run-with-memory-cap.sh 8G cargo test -p singlefs-checker@0.1.0]
exit=0 agent=main cwd= [cargo test --release -p singlefs-checker@0.1.0]
exit=0 agent=main cwd= [cargo test --release -p 'singlefs-check*']
exit=0 agent=main cwd=/crates/singlefs-harness [CARGO_ALIAS_XT='test --release -p singlefs-checker' cargo xt]
exit=2 agent=main cwd= [cargo test --release -p singlefs-checker] ✗ 重型测试被拒：cargo test 跑到 singlefs-checker 包的测试（checker 档，默认只在提交时跑；不…
```

漏在哪一行：`-p` 的值按字面去成员表里查，查不到的就从范围里丢掉——「scope = [members[name] for name in packages if name in members]」（`.claude/hooks/lib_heavy_tests.py:561`）。之后按包判的那一条只看范围——「checker_directory = members.get(CHECKER_PACKAGE_NAME)」（`.claude/hooks/lib_heavy_tests.py:572`）。
别名那一路只当成「在当前目录裸跑 cargo test」——「invocation, rest = ("test", found.rest) if alias_reason else test_invocation(found.subcommand, found.rest)」（`.claude/hooks/lib_heavy_tests.py:499`）。当前目录在 harness 里时，范围就是 harness。这一段在拆分之前会被「名字含 layer0」那一条兜住：harness 里当时有 layer0 目标。拆完之后 harness 里一个 layer0 目标都没有了，所以 26 是这一轮新出现的漏；在 core 目录里起，拆分之前也漏。
文件头「看不见的」只写了配置文件里的别名：「看不见的：.cargo/config.toml 与 `--config <文件>` 里定的别名与 runner；拷走改名、又不点名用例函数的测试二进制；」（`.claude/hooks/lib_heavy_tests.py:59`）。`CARGO_ALIAS_` 那一种它自称认得，实际上只在当前目录的范围里认。

38–41、51、57、59 这几种由包装与变量拼出来的写法，属于 `lib_shell_words.py` 文件头「判不到的」那一类。看门狗在进程这一层能认出真起的测试二进制：把 argv[0] 设成 `…/deps/first_transaction_step_seven_layer0-<哈希>`、cwd 设成 checker 包，调 `classify_process`，交 `checker-tier-binary`。但看门狗只记检出，不拦（这一点量过：草稿 `q1/proc.py`，交 `[('checker-tier-binary', 'checker 档')]`）。

M1、M2 是旧缺口，拆分之后按包判让它更要紧：
- 仓里 cargo-mutants 的配置写着「test_workspace = true」（`.cargo/mutants.toml:21`）。
- `mutants_test_arguments` 把 cargo mutants 的 `-p`（它的意思是「只变异这个包」）照搬成 cargo test 的 `-p`——「if word in MUTANTS_PACKAGE_OPTIONS_WITH_VALUE:」（`.claude/hooks/lib_heavy_tests.py:419`）。
- 本机装的 cargo-mutants 25.3.1 二进制里有 `test-package` 与「Run all tests in the workspace」的帮助串，所以这两条大概率会跑整个工作区的测试（推的，没跑：闸拒了 `cargo mutants --version`，我没有换写法绕过去）。

X1 反方向：从工作区根 `cargo test --test <只在 harness 里的目标>` 被判成 checker 档，这是误拒，不是漏。

### 四句

1. 分不分辨臂：打中的是「按包判」这一条臂自己的前提——包名按字面比。旧的「按名字判」对这几种写法同样漏，所以它不分辨新旧两条臂。
2. 被判的系统看得见判别它的东西：`-p` 的值就在命令行上，闸可以照 cargo 的 pkgid 规则做归一（去掉 `@版本`、取 `#` 之后的名字、按通配去成员表里匹配）；`CARGO_ALIAS_*` 的值也在 environment 里，看得见。
3. 满足的分句：判据表 Q1「判红的观测」里的「任一条返回 `None`」——22、23、24、26、M1、M2。
4. 跑前条款没给改法。我自己提的改法（只在我的模型上想过，被攻过零轮，都没实现）：`-p` 的值照 pkgid 与通配规则归一之后再查成员表；别名展开值读得到时（`CARGO_ALIAS_*`、`--config alias.*=` 的字符串值），把展开值当参数重判一遍，读不到时只要工作区里有 checker 包就按 checker 档判；cargo mutants 读 `.cargo/mutants.toml` 与 `--test-workspace`、`--test-package`。这几条各修哪一格都是推的。

## Q2 `#[path]` 把 harness 的共用模块编进 checker 的测试二进制

判定：绿（读源码，没编 checker 的测试二进制）。依据：

- 三个 `mod.rs`（`common` 807 行、`common_tree_split` 424 行、`common_admission` 361 行）里，
  `env!|option_env!|CARGO_|include_str!|include_bytes!|include!|file!()|module_path!|current_dir|env::var|#[cfg(`
  一处都没有。读运行环境的只有 `common/mod.rs` 的 `std::env::temp_dir()` 与 `std::process::id()`，拼镜像文件名用的。
  名字里带进程号，还有一个每个二进制各一份的静态计数器。两个包的测试二进制同时跑，镜像名也不撞。
- 11 个搬过去的测试文件里也没有 `env!`、`CARGO_`、`include*!`、`fs::`、`Path::new`、以 `"../` 或 `"tests/` 起头的串。
  cargo 跑集成测试时当前目录是包根，这次从 harness 换成了 checker。这个变化碰不到读相对路径的代码，因为一处这样的代码都没有。
  会读的环境变量只有 `SINGLEFS_CRASH_INJECTION_*`（`second_transaction_supplement_three_crash_injection.rs`），与包无关。
- 「use crate::common::{file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};」（`crates/singlefs-harness/tests/common_tree_split/mod.rs:27`）。
  checker 包里声明 `mod common_tree_split;` 的两份文件，都在它前面声明了 `#[path = "../../singlefs-harness/tests/common/mod.rs"] mod common;`：
  - `second_transaction_position_addressed_trees_layer0.rs` 第 25–28 行；
  - `second_transaction_supplement_two_tree_split_layer0.rs` 第 23–26 行。

  所以 `crate::common` 在各自的二进制里都解析到那一份。
- 一个没被问到、但和语义有关的点：harness 包的 `[dependencies]` 依赖 checker 库——「singlefs-checker = { path = "../singlefs-checker" }」（`crates/singlefs-harness/Cargo.toml:9`）；checker 又 dev-depends 回 harness。
  - checker 的**集成测试**二进制里，checker 库只有一份，与 harness 链的是同一份，不影响。
  - checker **库自己的单测**（`cargo test --lib`）要链 harness 的话，会编出第二份 checker 库，两份的类型互不相同。
  - 这一点是推的，没编。今天 checker 库的单测不链 harness，所以不打中任何东西。

## Q4 94 号不读 `[dev-dependencies]`

### 被判的样子（快照，整行抄；工作区里这两行已经被别的会话改回 HEAD 的写法，所以不写成「」引文）

```
快照 .claude/gate.d/94-checker-implementation-disjoint.sh:21
# `[dev-dependencies]` 不认——它只编进 tests/ 与 examples/，库本身链不到；checker 档的崩溃枚举用例就住在 checker 包的 tests/ 里、
快照 .claude/gate.d/94-checker-implementation-disjoint.sh:64
DEPENDENCY_TABLE = re.compile(r"^\[(?:target\.(?:'[^']*'|\"[^\"]*\"|[^\]]+)\.)?(?:build-)?dependencies\]$")
快照 .claude/gate.d/94-checker-implementation-disjoint.sh:187
        code = line.split("//", 1)[0]
```

D13 已定项 5 的射程句是 `.claude/kb/decisions/13-验证路线.md:93`，1297 字节的一整行，不抄进来。它和这一格有关的只有末尾一句：集成测试不算共享，以及 94 号只读 `[dependencies]` 与 `[build-dependencies]`。

### dev-dependency 进不进 checker 库自己的编译单元（玩具工作区，`toy_cargo_semantics.sh` 原样输出节选）

toy-checker 的 `[dev-dependencies]` 写 `implementation = { package = "toy-core", … }`，库里放三样：一段 doctest、`#[path = "../../toy-core/src/lib.rs"] mod copied_from_the_implementation;`、一个 `#[cfg(test)]` 单测。

```
== cargo build -p toy-checker
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
== cargo test --offline -p toy-checker --lib -- --nocapture
MARKER-LIB-UNIT-LINKED-CORE
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
== cargo test --offline -p toy-checker --doc
   Doc-tests toy_checker
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

读法：
- 库的单测与 doctest 都链到 dev-dependency。94 号文件头「只编进 tests/ 与 examples/，库本身链不到」这句话，对库给别人链的那一份成立，对 `cargo test --lib` 与 `--doc` 不成立。
- `#[path]` 把实现的源码整份编进库（不是 cfg(test)，是正式那一份），用不着任何依赖。
- 这两条都有人跑：闸把 checker 的 `--lib` 与 `--doc` 判成不重型（Q1 表 49、50 行交 None），谁都能跑。

### 94 号对各种形态的判定（`q4_gate94.sh` 原样输出）

每个样本 = 快照四个包的 `Cargo.toml` 与 `src/`，再各加一处：
- V1：checker `lib.rs` 里 `#[cfg(test)] mod … { use singlefs_core as _; }`
- V2：checker 的 `[dev-dependencies]` 加 `implementation = { package = "singlefs-core", path = "../singlefs-core" }`，`lib.rs` 的 `#[cfg(test)]` 里写 `implementation::transaction::…`
- V3：`lib.rs` 的 doctest 里 `use singlefs_core as _;`
- V4：`#[path = "../../singlefs-core/src/transaction.rs"] mod copied_from_the_implementation;`
- V5：`include!("../../singlefs-core/src/transaction.rs");`
- V6：`#[cfg(test)]` 里 `use singlefs_harness::crash as _;`

```
== V0 snapshot-94 rc=0:   ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减…
== V1 snapshot-94 rc=1:   ✗ checker 的源码里有 1 处引实现侧的 `singlefs_core`：
== V2 snapshot-94 rc=0:   ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减…
== V3 snapshot-94 rc=0:   ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减…
== V4 snapshot-94 rc=0:   ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减…
== V5 snapshot-94 rc=0:   ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减…
== V6 snapshot-94 rc=0:   ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减…
== V0 HEAD-94 rc=1:   ✗ checker 与实现除常量模块之外还共享 1 个依赖（按本仓内部依赖…
== V2 HEAD-94 rc=1:   ✗ checker 与实现除常量模块之外还共享 1 个依赖（按本仓内部依赖…   ✗ checker 的源码里有 1 处用别名 `implementation::` 引实现侧（Cargo.toml
```

（行尾的 `…` 是脚本 `cut -c1-90` 截在多字节字符中间留下的，原样里是一个坏字节；其余逐字。）

读法：
- **判据表 Q4 字面的红分句没打中**：② 对 `#[cfg(test)]` 块里字面的 `singlefs_core` 照判（V1 红）。
- **旁边的三格打中了**：
  - V2 是这一轮造成的回归：别名表只从 94 号认的那几种节头里收，而 `[dev-dependencies]` 已经不在其中，所以 `implementation::` 这条别名引用一处都看不见。HEAD 那一份判红。
  - V3（doctest）：`line.split("//", 1)[0]` 把 `///` 整行当注释丢掉了。
  - V4、V5：`#[path]`/`include!` 里写的是 `singlefs-core`（连字符），`singlefs_core` 那条字面查找碰不到。这是旧缺口，与 dev-dependency 无关，但它恰好是这一轮 checker 测试用来引 harness 共用模块的同一个手法。
- V6 算不算违反已定项 5，是政策问题，不归我判：库单测经 harness 跑实现，与集成测试同构。我只指出射程句没写它。
- 射程句与 94 号文件头都只说了 `tests/`，没说库的 `cfg(test)` 单测与 doctest 也能链 dev-dependency，而文件头那句「库本身链不到」在这两种情形下是错的。

### 四句

1. 不分辨臂（这一格没有对立的臂）。
2. 94 号读得到判别它的东西：Cargo.toml 的 dev 节与 `///` 行都在它的输入里。
3. 满足的分句：Q4「判红的观测」的字面写的是「对 `#[cfg(test)]` 块里的 `singlefs_core` 引用不判」，这一句没打中。打中的是同一格「说清……或指出它不会红」那一边——我指出了三种它不红的形态（V2、V3、V4/V5），射程句一种都没写。按字面这一格算没量到，主 agent 判它归哪一边。
4. 跑前条款没给改法。我的改法（被攻过零轮，推的）：
   - 别名表照旧从 dev 节收，只让 ① 的闭包不读 dev 节，两件事分开；
   - ② 扫 `///` 与 `//!` 里代码围栏中的行；
   - ② 另扫 `#[path = "…singlefs-core…"]` 与 `include!(…singlefs-core…)`；
   - 射程句加一句「库的 `cfg(test)` 单测与 doctest 也链得到 dev-dependency，94 号 ② 只按源码字面判它们」。

## Q6 crash-case-check.py 新判法的漏洞

### 判法与它自己写的射程（快照与工作区这份文件相同，下面是「」引文）

- 认调用：「ENUMERATION_CALL = re.compile(r"\b(enumerate_layer0\w*)\s*(?:::<[^>]*>)?\s*\(")」（`research/scripts/crash-case-check.py:36`）。只认名字以 `enumerate_layer0` 起头的函数，快档靠名字里带不带 `quick_tier` 分。
- 认测试函数：「TEST_FUNCTION = re.compile(r"((?:[ \t]*#\[[^\]]*\][ \t]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+(\w+)\s*\(")」（`research/scripts/crash-case-check.py:37`）。属性要单独占一行，属性里不能有 `]`。
- 豁免：「waiver = re.search(r"crash-case-check:not-a-crash-case\s+(\S.{3,})", attributes)」（`research/scripts/crash-case-check.py:72`）。理由只要 4 个字以上，内容不核。
- 射程句：「判不了的：经 tests/<子目录>/ 里的共用函数间接调枚举函数的、宏展开出来的调用——认不出，不判（报告里写明是按直接调用认的）。」（`research/scripts/crash-case-check.py:19`）

### 造的形状（`q6_fixture.sh` 原样输出）

```
  ✗ crates/singlefs-harness/tests/f10_control_direct.rs:2 f10_control_direct_full_in_harness：直接调全量崩溃枚举，却写在 singlefs-harness 包里（崩溃枚举用例要住在 crates/singlefs-checker/tests/）
  ✗ crates/singlefs-harness/tests/f11_cfg_attr_ignore.rs:3 f11_cfg_attr：直接调全量崩溃枚举，却写在 singlefs-harness 包里（崩溃枚举用例要住在 crates/singlefs-checker/tests/）
  ✗ 崩溃枚举用例 4 条里有 2 处写在 checker 包之外、或标了 ignore 没登记（逐处列在上面）
rc=1
recognized: ['f7_full_enumeration_of_a_big_stream_in_the_quick_tier', 'registered_full', 'f10_control_direct_full_in_harness', 'f11_cfg_attr']
```

13 个样本里只认出 4 个。没认出、也没判红的有 9 个，其中 8 个是射程句没写的：

| 样本 | 形状 | 射程句写了没 |
|---|---|---|
| f1 | harness 里 `#[test]` 调同一文件里的 helper，helper 调 `enumerate_layer0` | 没写（射程只写了 `tests/<子目录>/`） |
| f2 | `#[test] fn …() { enumerate_layer0(…) }` 写在一行 | 没写 |
| f3 | harness 里 `#[ignore = "全量 [层 0]：平时不跑"]`（属性值里带 `]`） | 没写 |
| f3b | 同上，放在 checker 包里、没登记：本该报「标了 ignore 没登记」，没报 | 没写 |
| f4 | 宏生成 | 写了 |
| f5 | harness 里带豁免注明、`&full_expansion` 的全量 | 没写（豁免不核内容） |
| f6 | checker 包里 `#[ignore]` 加豁免：不登记，谁都不跑 | 没写 |
| f8 | 自写的枚举函数（不叫 `enumerate_layer0*`） | 没写 |
| f9 | `use …::enumerate_layer0 as run_all;` 改名 | 没写 |
| f7 | checker 包里不标 ignore 的大流：按设计算快档、放行（问的「不标 ignore 的大流」这一格，判法本身就不设上限） | 判法如此，射程句没写「大小不判」 |
| f11 | `#[cfg_attr(…, ignore)]` 被当成没标 ignore | 只影响 checker 包里快档与全量的归类，射程句没写 |

### 真仓里（快照）的间接调用

`q6_indirect.py` 在每个 `tests/*.rs` 里找出「调 `enumerate_layer0*` 或 `enumerate_every_subset*` 的非测试函数」，再找调这些函数的 `#[test]`，只在同一文件里往下追。数出来的结果（原样行数用 awk 数）：

```
harness helper-routed unwaived tests: 17
harness waived: 3
```

- **harness 包 17 条**，分在 3 个文件里：`crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs` 3 条，`crash_enumeration_resumes_from_its_progress_file.rs` 5 条，`crash_enumeration_sharded_across_processes.rs` 9 条。
  - 17 条都经同一文件里的 `run` 一族，没有豁免，crash-case-check 一条都看不见。
  - 我追到的 expansion 实参都是 `&quick_tier_expansion`，sharded 那 5 条 `merge_stops_*` 的 helper 再往下一层没追。所以按今天的实参它们都是快档小流。
  - 但是：同样这 17 个调用点，只要把实参换成 `&full_expansion`，crash-case-check 照旧一声不吭。
  - `tests/common*/` 里一处 `enumerate_layer0` 都没有，射程句点名的「子目录共用函数」这一形，真仓里是 0 处。
- **checker 包 6 条经同一文件里的 helper**：`first_transaction_step_seven_layer0.rs` 490、581，`…position_addressed_trees_layer0.rs` 466、477，`…supplement_two_tree_split_layer0.rs` 394、412。其中 3 条标了 ignore、已经登记。
- **登记表 8 行 `crash-case:`，crash-case-check 只认出其中 3 条**（floor-raise、parallel-line-one、second-stream）。另外 5 条的来路：
  - first-stream、tree-split、position-addressed：经 helper；
  - c561-sigma-full：自写枚举，名字不在 `enumerate_layer0` 一族里；
  - crash-injection：不走枚举。

  脚本成功那句报「查了 11 条」，读的人会以为 8 条登记的全在里面。

c561 那一条（快照，整行抄；工作区的这份文件已经被改，不写成「」引文）：

```
快照 crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs:712
    let enumerated = enumerate_every_subset_of_one_segment(
快照 crates/singlefs-checker/tests/record_checker_judges_absence_by_the_persisted_set.rs:720
    assert_eq!(enumerated.states, 262_144, "σ 有 18 个写");
```

它今天住在 checker 包、标了 ignore、已经登记。可要是写在 harness 里，crash-case-check 照样判绿。

### 豁免注明被用来写不真的理由：真仓一例

这一轮新加的那条豁免（快照，整行抄）：

```
快照 crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs:382
// crash-case-check:not-a-crash-case 测的是断点续跑装置在判红时删进度文件，枚举的是几步的合成流、单跑几秒；不是要登记的放量用例
快照 crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs:385
    let stream = first_stream("resume-red-run");
```

同一文件的 `first_stream` 调的是 `build_pool`：mkfs → 取号 → 暖机两次 → 第一个文件，41 次写，是第一条真流，不是「几步的合成流」。枚举用的是 `&quick_tier_expansion`，观察者到第 12 个状态就 panic，所以规模确实小，结论（不是放量用例）站得住，只是写下的理由不真，而脚本不核理由。
为什么要加这一条豁免：`enumerate_layer0_in_state_slices` 的名字里没有 `quick_tier`，快档是靠实参 `&quick_tier_expansion` 选的。按名字认快档，在这个调用点上会误判，只好用豁免压下去。换句话说，快慢由实参定，判法却按名字认。

### 四句

1. 不分辨臂。
2. 脚本看得见判别它的东西：实参 `&full_expansion` / `&quick_tier_expansion`、同文件 helper 的函数体、`use … as …` 都在源码里。
3. 满足的分句：Q6「判红的观测」——「有一处间接调用没被任何检查罩住而没写进射程」。真仓 17 条经同文件 helper 的调用没被 crash-case-check 罩住，射程句只写了子目录与宏。闸那一边：harness 包不点名用例时不算重型，也罩不住。按今天的实参这 17 条都是小流，所以「没罩住」目前没有造成放量用例漏网；判的是射程句没写。
4. 跑前条款没给改法。我的改法（被攻过零轮，推的）：
   - 同一文件里按调用图往下追 helper；
   - 快档按实参 `&quick_tier_expansion` 认，不按函数名；
   - 豁免注明要带一个可核的量（例：写上闭式状态数，脚本用 `closed_form_state_count` 的钉值核）；
   - 成功那句报「登记的 N 条里认出 M 条」；
   - 射程句补上同文件 helper、改名 import、自写枚举、单行属性、属性值带 `]`。

## Q9 搬迁遗漏

取法：对 11 个搬走的文件名在快照树里各 `grep -rnF` 一遍，排掉 `target`、`.git`、`.claude/singlefs-ai-sop`、`research/prompts/`、`research/results/`。一共命中 174 行、31 个文件（草稿 `q9/all-hits.txt`）。逐个看过之后：

- **功能性引用都已改好**：
  - `crates/mutations.tsv` 65 行，第 5 列没有一行还是 `-p singlefs-harness --test <搬走的目标>`（awk 逐行核过）；
  - `stage-inputs.tsv` 8 行都是 `test=singlefs-checker:`；
  - `e158_root_choice_repair.rs:17758` 的源文件常量已改；
  - `research/scripts/admission.py` 15 处都是自检里的假目标名；
  - `agent-watch.py:2243` 是自检里的假二进制名；
  - `replay.sh`、55 号、`vm-harness.md`、根 `Cargo.toml`、`.claude/gate.d/*.sh` 对 11 个名字零命中。`replay.sh`、55 号、`e152-*.sh` 里的 `-p singlefs-harness` 都是 `--bin`，二进制留在 harness，是对的。
  - `layer0-shard-run-selftest.sh` 在自己的临时仓里登记 `test=singlefs-harness:…`，是它自己造的替身。admission.py 不按包拒，所以不打中。
- **还指旧路径的（注释，不影响跑）**。快照整行抄，这两份在工作区已被改，不写成「」引文：
  ```
  快照 crates/singlefs-harness/src/history.rs:2102
      /// （层 0 那一路同样从 `mkfs_operation_count` 之后起枚举，见 `tests/first_transaction_step_seven_layer0.rs`）。
  快照 crates/singlefs-harness/src/bin/e161_crash_state_dedup_and_time_split.rs:316
  /// 第一条流：照 `tests/first_transaction_step_seven_layer0.rs:450-481` 的 `prepare`（S2 的段长与写数断言照它）。
  ```
  这两处都在 harness 包里，写的是 `tests/…`，搬家之后 harness 的 `tests/` 里已经没有这个文件了。
- **行号错开 1 行**：搬过去的 10 个文件在 `mod …;` 前各插了一行 `#[path]`，其后所有行号 +1。
  - 快照上 `second_transaction_step_zero_layer0.rs` 的 `fn prepare` 在 204 行，HEAD 在 203 行；`first_transaction_step_seven_layer0.rs` 的 `fn prepare` 在 451 行，HEAD 在 450 行。
  - 引用旧行号的有：e161 第 12、316、362、465、567、568 行（第 12 行路径改了、行号 `203-474` 没改）；kb `experiments/142-第一个事务的干跑.md:340` 的「第 443、445、447 行」。那一句里的值本来就已经过期（`ROOT_PERSISTED_STATES` 今天是 9，不是 4）。
- **正文判绿观测本身写错一处**：正文写「命中只剩 `lib_heavy_tests.py:916` 那处样本」。
  - 真实行号是 959——「write("crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs", "")」（`.claude/hooks/lib_heavy_tests.py:959`）。
  - 同类的样本行还有 6 行：`lib_heavy_tests.py` 1011、1051、1148，`heavy-test-guard.sh` 716、760、889，都是 `-p singlefs-harness --test first_transaction_step_seven_layer0`，喂的是自检的样本工作区。这些不是漏改，但与判绿观测「只剩一处」的字面不符。
- `records/` 下 21 行命中，是带日期的记录，没算。

四句：不分辨臂；是文本遗漏，不涉及被判系统看得见什么；满足 Q9「列出漏改的行」；跑前条款没给改法。改法是照上面逐行改注释与行号（推的，改法本身不需要攻）。

## 归因句「54 号快档红 = C577 之后钉值没改」

### 装置

`c577_attribution.sh`：三份副本，各用自己的 `CARGO_TARGET_DIR`。
- A = 快照的 crates；
- B = A 只把 C577 那道屏障换成 `Ok(())`，即「writer.perform(CommitStep::Barrier)」（`crates/singlefs-core/src/transaction.rs:1208`，`persist_the_root_then_rotate_the_system_configuration` 的最后一句）；
- E = 45d49aaf 的 crates（第一条流的钉值与 `[…,26,3,2]` 都是那次提交钉进去的）。

每份做两件事：
- 跑我的原型 `zz_attack_first_stream_segment_counts`：照 `common::build_pool` 那一串调用，只把镜像换成两块 `SparseBlockDevice` 内存稀疏盘，录流、切段、算闭式数。原型不枚举。
- 跑 harness 档里四个钉这条流的测试目标。

正文引的 `transaction.rs:553`、`:1188` 是两处注释，真正发屏障的是第 1208 行。

`c577_attribution.sh` 原样输出（harness 那一段的 `Running` 行去掉了括号里的路径）：

```
== A prototype rc=0
ATTACK_FIRST_STREAM mkfs_operations=23 writes=41 unit_writes=24 segments=[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]
ATTACK_FIRST_STREAM closed_form_two_states=16777240 quick_two_states=26 quick_torn=46 full_torn=16777260
ATTACK_FIRST_STREAM barrier_steps_before_root=2 segments_without_that_barrier=[2, 2, 1, 2, 2, 1, 2, 24, 5]
== B prototype rc=0
ATTACK_FIRST_STREAM mkfs_operations=23 writes=41 unit_writes=24 segments=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2]
ATTACK_FIRST_STREAM closed_form_two_states=67108885 quick_two_states=29 quick_torn=54 full_torn=150994980
ATTACK_FIRST_STREAM barrier_steps_before_root=2 segments_without_that_barrier=[2, 2, 1, 2, 2, 1, 26, 5]
== E prototype rc=0
ATTACK_FIRST_STREAM mkfs_operations=23 writes=41 unit_writes=24 segments=[2, 2, 1, 2, 2, 1, 26, 2, 1, 2]
ATTACK_FIRST_STREAM closed_form_two_states=67108885 quick_two_states=29 quick_torn=54 full_torn=150994980
ATTACK_FIRST_STREAM barrier_steps_before_root=2 segments_without_that_barrier=[2, 2, 1, 2, 2, 1, 26, 5]
== A harness rc=101
     Running tests/crash_enumeration_sharded_across_processes.rs 
test the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts ... FAILED
test two_and_three_shards_merge_into_the_same_tally_and_count_line_as_one_unsharded_run ... FAILED
test merge_stops_when_a_ledger_was_sliced_another_way ... FAILED
test print_the_unsharded_enumerations_for_the_golden_comparison ... FAILED
test unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 9.51s
test result: FAILED. 5 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.52s
     Running tests/crash_segments_per_device_and_torn_in_place_overwrites.rs 
test the_first_transaction_stream_keeps_its_segments_and_takes_the_third_state_on_its_system_configuration_writes ... FAILED
test one_device_missing_its_barrier_before_the_root_slot_is_reported_by_the_record_checker ... FAILED
test result: FAILED. 7 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.10s
     Running tests/first_transaction_step_five_publish.rs 
test recorded_paths_match_the_registered_segment_sequences ... FAILED
test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
     Running tests/in_place_overwrite_torn_state_count.rs 
test the_first_transaction_stream_counts_with_the_torn_third_state_as_measured ... FAILED
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.59s
== B harness rc=101
     Running tests/crash_enumeration_sharded_across_processes.rs 
test unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding ... FAILED
test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 76.69s
     Running tests/crash_segments_per_device_and_torn_in_place_overwrites.rs 
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.53s
     Running tests/first_transaction_step_five_publish.rs 
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.65s
     Running tests/in_place_overwrite_torn_state_count.rs 
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s
c577 rc=0
```

（A harness 那一段第一行 `0 passed; 1 failed; … 9 filtered out` 是 sharded 那个目标的 golden 用例自己起的子进程的输出。）

### 判定

- **「钉值没随 C577 改」这一半核实了（副本上量的）。** A 与 B 只差第 1208 行那一道屏障，B 与 E 的原型输出逐字相同。
  这道屏障把第二次暖机末尾那两次系统配置槽写，从第一个事务的 24 个单元写那一段里切了出去：26 变成 2 + 24。
  A 上各项的真值（副本数，要在入库装置上重做才能引）：
  - 段序列 `[2,2,1,2,2,1,2,24,2,1,2]`；
  - 两态闭式 16777240；
  - 甲二快档两态 26、三态 46；
  - 全量三态 16777260（原钉值 150994980）。

  下面这两个钉值（快照，整行抄；工作区这份文件已被改）在 A 上都不再成立：
  ```
  快照 crates/singlefs-checker/tests/first_transaction_step_seven_layer0.rs:437
  const FULL_STATES_WITH_TWO_STATES_PER_WRITE: u64 = 67_108_885;
  快照 crates/singlefs-checker/tests/first_transaction_step_seven_layer0.rs:443
  const QUICK_TIER_STATES: u64 = 54;
  ```
  54 号那 3 条 FAILED 里，`layer0_quick_tier_matches_the_full_tally_shape` 与 `positive_control_…` 都先经 `prepare`。`prepare` 第 459 行钉的段序列 `vec![2, 2, 1, 2, 2, 1, 26, 2, 1, 2]` 在 A 上是 `[…,2,24,…]`，所以这两条红，C577 解释得了（推的，依据是原型的段序列；checker 包的测试我没跑）。
- **「只有 54 号快档红」这一半不成立：harness 档也红。**
  - A 上 4 个 harness 测试目标共 8 条 FAILED；B 上这 8 条全绿。只有 `unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding` 在 A、B 上都红，是另一个原因，我没追。
  - 这几条是 harness 档，主 agent 与实现员「随时跑」的那一档。所以快照上 harness 档本身不绿，这一点正文第二节没写。
- **第三条 `removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker` 的红，C577 解释不了。** 它钉的段序列是（快照，整行抄）：
  ```
  快照 crates/singlefs-checker/tests/first_transaction_step_seven_layer0.rs:764
          vec![2, 2, 1, 2, 2, 1, 26, 3, 2],
  ```
  同样的删法（删掉根槽前那道两盘各一步的池屏障）在 B（去掉 C577）与 E（这一行被钉进去的那次提交 45d49aaf）上量出来都是 `[2, 2, 1, 2, 2, 1, 26, 5]`。
  同一次提交在 harness 里钉的相邻形状，正是 `[…, 26, 5]`：快照里 `crash_segments_per_device_and_torn_in_place_overwrites.rs` 第 429 行 `vec![2, 2, 1, 2, 2, 1, 26, 5],`，下一行的断言消息说盘 1 漏发那一道时记录两份、根槽、两次系统配置槽写并成一段。
  按每盘各自释放的切段规则，FUA 只释放它自己那块盘，所以 `[…,3,2]` 切不出来。于是：就算只把 C577 相关的钉值改回去，这一条照样红；它在 45d49aaf 钉进去时大概率就已经是红的（推的：E 上量的是原型里照搬的那几行切法，不是那条测试本身，checker 档的测试我不跑）。
- 原型用内存稀疏盘，`build_pool` 用文件镜像。我认为两者录下的写流相同，理由：
  - `RecordingBlockDevice` 记的是 core 发出的请求，不看底下是什么盘；两种盘探到的物理块都是 512；
  - B、E 上原型的段序列、写数 41 与钉值逐项相同。

  这是推的，没用文件镜像对照：规矩要求原型不在盘上建镜像。

### 四句

1. 分不分辨臂：这一格不判臂，判的是一句归因。打中的是那句归因的射程（「只有 54 号快档」、三条红都算 C577 的账）。
2. 系统看得见判别它的东西：段序列是确定性的，录流就看得见。
3. 满足的分句：正文第二节「这一归因是推的，没在 HEAD 上单独复现——腿要核」。核的结果是：2 条红由 C577 引起（副本量过）；第 3 条 C577 解释不了（副本量过，它钉的形状在 C577 之前的提交上同样对不上）；harness 档 8 条红也由 C577 引起（副本量过）。
4. 改法：正文说会话 singlefs-99 下一步改钉值。在打中的格上：
   - 改钉值能修那 2 条 checker 与 8 条 harness（推的）；
   - 修不了 `removing_the_barrier…`：那一条要把期望改成 `[…,2,24,5]`，还要重新判它原本要证的东西（「记录两份与根槽并成一段」）在按盘释放的规则下还成不成立，推的。

## 没打中的形状

- Q1：下面这些都判 checker 档，没漏：
  - `-p` / `--package=` / `-p=` / `-psinglefs-checker`；
  - cwd 在包里、在 `tests/` 里、`( cd … )`、`cd … &&`、`pushd`、`-C`；
  - `--manifest-path`，从仓外起也一样；
  - `--test <名>` / `--tests` / `--all-targets` / `--test '*'`；
  - nextest（`-p` 与包目录裸跑）；
  - 两个 `-p`；
  - release / debug 二进制，以及仓外 cwd 起的绝对路径二进制；
  - 经 `bash -c`、`env`、`nice`、`capped.sh`、`run-with-memory-cap.sh` 套 `capped.sh`、`rustup run` 包装的；
  - hack、llvm-cov、miri、`mutants -p singlefs-checker`、`cargo t`。

  `cargo bench --test` 与 `--config alias.*=`：cargo 真行为上不跑集成测试或不生效，不算漏。
  `--benches`、`--examples`、`--doc`、`--lib` 交 None：checker 包没有 bench 与 example 目标，`--doc`、`--lib` 按设计放行（放行后果见 Q4）。
- Q2：`cfg`、`env!`、相对路径、`CARGO_*`、`crate::common` 解析，逐项查过，没有语义变化（见 Q2 那一节）。
- Q4：② 对 `#[cfg(test)]` 块里字面的 `singlefs_core` 照判（V1 红）。
- Q9：`replay.sh`、55 号、`vm-harness.md`、根 `Cargo.toml`、`.claude/gate.d/*.sh`、`mutations.tsv`、`stage-inputs.tsv` 没有漏改。
- 取样范围：
  - Q1 56 种写法（`q1_probe.py` 全列）；
  - Q4 7 个样本；
  - Q6 13 个样本加真仓全部 `tests/*.rs`；
  - Q9 11 个文件名全仓；
  - 归因 3 份副本 × 1 条流，另跑 4 个 harness 测试目标。

## 这条腿自己的限度

- 副本上的数（归因那一节的段序列、闭式数、harness 8 条红绿）只在我的副本装置上量过，不算入库装置上的数。原型 `zz_attack_first_stream_segment_counts.rs` 是我自己的，被攻过零轮。
- checker 包的测试我一条都没跑：54 号那 3 条红的具体断言值，是从原型的段序列推的。`removing_the_barrier…` 在 45d49aaf 上「就已经红」也是推的。
- 我自己提的改法（Q1、Q4、Q6 各节「四句」第 4 条）一条都没实现，只在我的模型上想过，被攻过零轮。「修哪一格」全是推的。
- cargo-mutants 带 `test_workspace = true` 会跑整个工作区，是按二进制里的帮助串推的，没跑：闸拒了 `cargo mutants --version`，我没有换写法绕过去。
- Q6「17 条」只沿同一文件的调用图往下追。helper 的实参经局部变量、闭包传进去的，我的 `EXPANSION` 正则看不到（表里那几行 `expansion=-`）。
- 「没打中」只抽样了这一次，按规矩不能单独拿来支撑结论。
- 工作区里 8 份被引文件在我跑的时候被别的会话改了（第一节列了）。我判的是快照，没判工作区今天的样子。

## 没做什么

- 没判 Q3、Q5、Q7、Q8，也没做命令分类表（不归这条腿）。
- 没跑 checker 包的任何测试、54 号、全量 `cargo test`；没跑 cargo-mutants。
- 没追 `unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding` 在 A、B 上都红的原因。
- 没用文件镜像对照内存稀疏盘录的流是否相同（推的，理由见归因那一节末条）。
- 没在工作区的现状上复判（工作区在变）。

## 草稿与删除

删了（先 `du -sh` 记下大小）：
- 之前的副本 `/tmp/claude-1000/three-way-attack/verification-split-r1/c577`（20G）与 `…/toyws`（25M）；
- 快照解包 `…/snap`（294M）、`…/snap2`（294M）；
- 玩具工作区 `…/toyws2`（17M）；
- 94 号样本树 `…/q4`（37M）、`…/q4-rerun`（37M）；
- 归因副本 `…/c577-rerun/A`（9.3M）、`/B`（9.3M）、`/E`（7.6M）；
- 编译目录 `…/c577-rerun/target-A`（2.4G）、`target-B`（2.4G）、`target-E`（1.5G）；
- 临时镜像目录 `…/c577-rerun/tmp-images`（200K）。

留着的：`/tmp/claude-1000/three-way-attack/verification-split-r1/` 下的日志与小文件，一共 608K。
- `rerun/*.out` 是报告里贴的原样输出；
- `c577-rerun/*.log` 是归因那一节的完整日志；
- `q9/all-hits.txt`、`q1/`、`q6/`、`q6-rerun/` 是 grep 结果、探针与 13 个样本源码。

这些是给主 agent 核的复跑材料，没有仓副本、没有编译目录。它们没入库，因为派发提示给的写范围只有报告、模型目录与草稿目录；要留进 `research/results/` 的，由主 agent 定。
