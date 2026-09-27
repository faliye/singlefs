# defs-gatebatch-m2-r1 本地攻方：转述核对表

原文：`research/scripts/admission.py:955-960`（崩溃枚举用例排除规则的整段说明，本轮 J2 攻击面点名的那一节）。
提示文件：`research/prompts/defs-gatebatch-m2-r1-local-attack.md`。

| 英文项（提示里的 R 编号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| R1「A crash case's input set equals the files that git lists under its registered paths…」 | `admission.py:955` 「一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去别的测试目标独占的测试文件，」 | 首稿写成「crates/, Cargo.toml, and Cargo.lock」，漏了「整个」（crates/ 是整棵树，不是任选几个文件）；首稿把三条路径当抄近似，没有照 `.claude/gate.d/stage-inputs.tsv:32-35` 核对四条用例登记的字面路径是不是都恰好是这三项——核对后确认四条都恰好是 `crates/ Cargo.toml Cargo.lock`，才敢在提示里写成「for every one of the four cases here」 | 定稿：「the files that git lists under its registered paths, which for every one of the four cases here are: crates/ in full, plus the top-level Cargo.toml, plus the top-level Cargo.lock」——补回「整个」（in full），补回「顶层」（top-level，区分工作区根的 Cargo.toml/Cargo.lock 与 crates/ 子目录下别的 Cargo.toml） |
| R1（续）「…minus the test files exclusive to other test targets…plus gate 54's own file, the toolchain description, the build environment description, and this case's own registration row」 | `admission.py:956` 「再加判它的 54 号、工具链、构建环境与这条用例的登记行。」 | 首稿把「判它的 54 号」译成笼统的「the gate」，丢了「54 号」这个具体编号；首稿也没写「这条用例」（this case's own），读起来像是四条用例共用同一行登记，而登记行其实按用例各自不同 | 定稿明写「gate 54's own file」与「this case's own registration row」，把「它的」「这条用例的」两处限定词都补回 |
| R2「A test target named X has files exclusive to it only when…」 | `admission.py:956-957` 「别的测试目标 X 独占的文件：它所在的包没有 build.rs、没写 [[test]]、package.autotests 与 package.build，X 是 tests/X.rs（或 tests/X/main.rs 那种目录目标、连同那个目录）」 | 首稿把四个否定条件写成「no build.rs or [[test]] or autotests or build」的松散并列，没有逐一点名 `Cargo.toml` 里的键名（`package.autotests`、`package.build`），容易被读成源码里任意位置的同名标识符；首稿也漏了「连同那个目录」，只译出 `tests/X/main.rs` 本身，没说目录目标要把整个目录一起算进独占文件 | 定稿逐一点名「no package.autotests key, and no package.build key」，并把目录目标写全「tests/X/main.rs together with every other file under that same tests/X/ directory」 |
| R3「X's files are not excluded…if X's name occurs as a whole word, after all comments are stripped…」 | `admission.py:958` 「而 X 的名字不以整词出现在别处任何 .rs（注释去掉之后）与 Cargo.toml 里」 | 首稿写成「isn't named elsewhere as a whole word, ignoring comments」，没有点名是在哪两类文件里找（.rs 与 Cargo.toml），读起来像是任何文件都算；也没说「别处」是排除 X 自己的文件这一层 | 定稿补回「in any other .rs file or in any Cargo.toml file」，并在前一句用「outside its own file」把「别处」的范围钉住 |
| R3（续）「mod X;、#[path]、include_str!、runtime path reads…any single one of them is enough…」 | `admission.py:958-959` 「`mod X;`、`#[path = "…X.rs"]`、`include_str!("…X.rs")`、运行期按字面路径读它，都让它留在输入里。」 | 首稿漏了「都」这个字——「都」是「这四种任何一种单独出现都够」，首稿只是罗列四种写法，没有明说其中任何单独一种就足够，容易被读成要凑齐几种才算 | 定稿加一句「any single one of them is enough to keep X's files in the input set」，把「都」译出来；另加一句「occurrences inside a stripped comment do not count」——这是给 R3 补的显式否定边界，附录原文本身在 958 行已经用「（注释去掉之后）」表达了同一件事，这句英文只是把它单独摘出来强调，不是多加的限定词 |
| R4「The registration…does not hand-list, case by case, which specific files that case reads…defaults to being included…」 | `admission.py:959-960` 「登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、build.rs、新拆出的 crate）默认就在输入里。」 | 首稿举例只写了「tests/common.rs, build.rs」，漏了「新拆出的 crate」这个例子，会让人以为默认收录只适用于 tests/ 下的文件，不适用于整个新增的 crate | 定稿补回第三个例子「a newly split-out crate」，与原文三个例子一一对应 |

多出来的限定词（原文没有、英文提示加了的）：

| 英文提示里的话 | 加在哪（提示文件行号，Read 时现查） | 为什么加 |
|---|---|---|
| 「for every one of the four cases here」（R1 之后紧跟的一句） | `defs-gatebatch-m2-r1-local-attack.md` 的 THE SPECIFICATION 一节，R1 那一行 | `admission.py:955` 本身没写"四条用例都是这三项"——那是登记表 `stage-inputs.tsv:32-35` 才有的事实，把它挂在 R1 后面是为了让模型不必去猜这四条登记路径是不是彼此不同；这是转述之外新加的一条事实陈述，不是对条款字面的翻译，因此单列在这里说明来源与理由，不算摘句失真 |
| G2/G3 里「这一节判据用哪条」的提示（比如 G2 每行末尾写「这名字属于 R2/R3/R4 里哪一条决定」） | 提示文件 FACT TABLE G2 与 ANSWER FORMAT 两节 | 事实表本身只给现象（保留/减去），不点名判它的是哪一条规则；这是给模型的追问角度，不是条款译文的一部分，不影响 R1–R4 的忠实度 |
