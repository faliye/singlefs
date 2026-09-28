#!/usr/bin/env python3
# admission: always 判的是此刻的登记表、输入与产物，门禁阶段与实验每一次开跑之前都要现判
# run-condition: none 它要的 git 与工具链版本由它自己读进输入指纹，读不到的按它的约定判（判不出来按要跑、前提没齐退 3），不交给调用方预判
"""准入：门禁阶段与实验开跑之前判「这一趟该不该跑」。门禁与实验共用这一个模块、一张登记表。

登记表是 .claude/gate.d/stage-inputs.tsv（唯一登记位，门禁阶段与实验各占几行）。一行一条，制表符分隔：
    <键> <制表符> <读的路径，空格分隔> [<制表符> <准入条件，空格分隔>] <制表符> #<为什么是这几条>
  键：门禁阶段写文件名（checker-tier-crates-mutation-replay.sh）；实验写实验号（E142），同一个实验另有一种调用方式的写
      实验号加斜杠加方式名（E142/layer0）。
  路径：从仓库根起；目录写成带斜杠的前缀。写成 @<别的键> 就是「那个键登记的全部路径」（只在实验行里用）。
  准入条件（第三列）：实验行认两种：
      question-row=<文件>#<行号>:<正则>          那份文件里第一列是 <行号> 的表格行，最后一格（状态）要匹配正则
      product-field=<实验键>:<行名>:<字段>=<值>   research/scripts/replay.sh 登记给那个实验键的产物里，
                                                  最后一行 `E7RESULT name=<行名>` 的 <字段> 要等于 <值>
    门禁行认四种：
      command=<可执行名>                  PATH 里找得到它
      readwrite=<路径>                    那个路径在、可读也可写（相对路径从仓库根起）
      probe=<仓内脚本>[:<参数>…]          bash <根>/<脚本> <参数…> 退 0（工作目录是仓库根）
      environment=<仓内脚本>[:<参数>…]    不是前提，是门禁复用判定的一项输入：bash <根>/<脚本> <参数…> 打出来的东西
                                          （例 herd7 的版本）。不在 git 树里的东西靠它进复用判定，见 ①
      正则、值、路径与参数里不许有空白（条件之间按空白切）。实验行写了门禁的种类、门禁行写了实验的种类，自查都判错。
  崩溃枚举用例行（键是 crash-case:<名>，门禁 54 号逐条按复用判定跑、逐条记全绿标记）：路径写整个 crates/ 与 Cargo 清单、锁，
    不按用例手列；算输入时自动减去用例读不到的文件（见「崩溃枚举用例」一节）。第三列认六种：
      test=<包>:<测试目标>:<用例函数>     恰好一条：跑的是 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数>；
                                         用例函数的每一处定义都要标 #[ignore]（不带 --ignored 的 cargo test 不跑它，重型测试闸按这一条认它）：
                                         同名的每一处 fn <用例函数>( 都算（cfg 二选一的、子模块里同名的也算），有一处没标就算没标；
                                         测试目标顺着 include!("<字面路径>")、#[path = "<字面路径>"] mod、mod <名>; 带进来的源文件一起判
                                         （include! 的参数不是字符串字面量、带进来的找不到、用 use 改名 include 族宏的，判不出，按没标算）；
                                         # 与 [ 之间许空白；找不到字面的定义（宏生成的用例）判错
      count-line=<前缀>                  日志里以「<前缀> 」开头的行恰好一行，原样记进全绿标记
      exhaustive=<前缀>                  那一行带 exhaustive=true（前缀要先登记成 count-line=）
      threads=<前缀>                     按那一行的 states= 找 LAYER0_PARALLEL_FINISHED 行判工作线程（前缀要先登记成 count-line=）；
                                         那一行自己带 worker_threads= 与 slices= 的（CRASH_INJECTION_FINISHED 这一类）拿它自己判；
                                         那一行带 shards= 的（双机分片 merge 那一趟）逐片判，见 judge_threads_of_each_shard
      threads-variable=<环境变量名>      至多一条：用例读线程数的环境变量（没登记是 SINGLEFS_LAYER0_THREADS）；crash-case-command 把它设成配的线程数
                                         （盖掉调用方环境里的），标记里 configured_worker_threads= 记它
      shard=across-machines              至多一条：这条用例的枚举认双机分片开关（SINGLEFS_LAYER0_SHARD，crates/singlefs-checker-tier/src/layer0_progress.rs），
                                         门禁 54 号 --full 在本地分片配置可用时把它交给 research/scripts/layer0-shard-run.sh 两台各跑一片再 merge；
                                         登记了它的用例，驱动脚本与它 eval 的配置判法（SHARD_DRIVER_FILES）按内容进这条用例的输入清单

输入指纹里的构建环境（build_environment_lines；实验的指纹与 manifest / crash-case-manifest 带 --build-environment 时进清单）：
  仓根与它往上每一层目录的 .cargo/config、.cargo/config.toml、rust-toolchain、rust-toolchain.toml，CARGO_HOME（没设取 ~/.cargo）下的
  config、config.toml——在的才进，按内容进，名字里不带绝对路径（同一批内容在主工作区与临时 worktree 里算出同一个数）；
  环境变量 RUSTFLAGS、CARGO_ENCODED_RUSTFLAGS、CARGO_PROFILE_*、CARGO_BUILD_*（CARGO_BUILD_JOBS 除外：只定编译并行度，
  research/scripts/capped.sh 与 mutate.sh 会设它）、CARGO_TARGET_*_RUSTFLAGS、CARGO_TARGET_*_RUNNER、RUSTC_WRAPPER、
  RUSTC_WORKSPACE_WRAPPER，设了的才进；RUSTC 设了的进它的值与 `$RUSTC -V`。
  runner、wrapper 与编译器指的可执行文件另进一行：环境变量 CARGO_TARGET_*_RUNNER、RUSTC_WRAPPER、RUSTC_WORKSPACE_WRAPPER、
  CARGO_BUILD_RUSTC_WRAPPER、CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER，与上面那几份配置文件里的 target.<三元组或 cfg>.runner、
  build.rustc-wrapper、build.rustc-workspace-wrapper，指的那个程序按内容进（值的首词是程序；带斜杠的相对路径，环境变量从仓根起、
  配置文件从 .cargo 所在的那一层起；不带斜杠的在 PATH 里找；找不到的进「找不到」一句）；首词之后指到现存文件的参数（绝对路径，或从同一层起的
  相对路径，另从登记的每条崩溃枚举用例的包目录起再解一遍：cargo 起 runner 时当前目录是包目录，`runner = "sh ../../tools/r.sh"` 这一类，
  解得到的都进）接着按内容进（定行为的是参数里的脚本）；CARGO_BUILD_RUSTC 与配置文件里的 build.rustc
  进它的 `-V`（与 RUSTC 同一种）。登记路径下指向目录的符号链接按链接指向的目录里的文件计入。

准入条件分三类，这里管前两类：
  ① 输入没变 ⇒ 不跑，沿用上次结论。门禁：登记的路径在 refs/sop/staged-green 那棵树与这一次的暂存树之间 git 说没变
     （gate-reuse，research/scripts/stage-must-run.sh 转到这里，判法与原来那份逐格相同）。
     登记了 environment= 的门禁另要两样：环境与这一道上一次判绿时记下的相同（阶段判绿之后调 gate-record-environment，
     记进 git common-dir 的 singlefs-gate-environment.<阶段>），且记下的那一趟判的树在这几条路径上与这一次的暂存树相同；
     没记下、取不出、对不上都按要跑处理。
     实验：登记的路径下每个文件的 sha256、登记行本身、cargo -V 与 rustc -V 汇成一个输入指纹；
     research/results/ 里有一份产物头上记着同一个键、同一个指纹 ⇒ 拒绝重跑（退 77）。
  ② 前提没齐 ⇒ 拒。实验退 3；门禁经 gate-preconditions 退 1，阶段照判红（不退 77：前提没齐时这一道一格都没判）。
     条件写在登记表那一行，不写在调用方的脚本里。
  ③ 跑的场合（重型测试前缀、内存包装）归 .claude/hooks/heavy-test-guard.sh 与 run-with-memory-cap.sh，这里不管。

子命令（<根> 是被判仓的根）：
  stage-fingerprint <根> <阶段文件名>    这一道登记输入在被判那棵树上的指纹：打「<sha256> <文件数>」
  stage-marker-check <根> <阶段文件名>   这一道这批输入有没有作数的全绿标记：退 0 有（第一行 ok <路径> <时刻>），退 1 没有或过了 SINGLEFS_REUSE_HOURS
  stage-marker-write <根> <阶段文件名>   判绿之后写这一格标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 自己调），打路径；gate-reuse 先看它再看暂存树
  gate-reuse <根> <阶段文件名>   门禁的复用判定：退 0 要跑，退 10 可跳过（stage-must-run.sh 翻成它原来的 1）；
                                 模块自己出错一律退 0（按要跑处理），不许出错就跳过
  experiment <根> <实验键>       实验的准入：退 0 放行，stdout 是要写在产物最前面的几行（E7INPUT 开头）；
                                 退 77 输入自上次产物以来没变；退 3 前提没齐；退 2 登记或调用有错。说明一律打到 stderr
  paths <根> <键>                列出登记给这个键的路径（@ 引用已展开），一行一条
  manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--registration-row] [--toolchain] [--build-environment]
                                 把这个键的输入逐文件清单（「sha256  路径」，按路径字节序）写进清单文件，末尾按参数次序
                                 加「sha256  <名字>」行；stdout 打「<整张清单的 sha256> <文件数>」。另算一遍逐字节相同的写法见 --selftest 那一格
  keys <根>                      列出登记表里的实验键，一行一个
  crash-cases <根>               核登记的崩溃枚举用例（第三列的写法、包与测试目标在不在、用例函数在不在、它标没标 #[ignore]），每条打一行
                                 「<键><制表符><包><制表符><测试目标><制表符><用例函数>」；登记有错退 2，原因打在 stdout（不带 ✗，阶段自己打 ✗ 与出路）
  crash-case-manifest <根> <键> <清单文件> [--extra-file <名字> <文件>]... [--judging-digest] [--toolchain] [--build-environment]
                                 这条用例的输入清单：登记路径下的文件减去别的测试目标独占的测试文件，末尾按参数次序加名字行，
                                 登记了 shard=across-machines 的再按内容加双机分片的驱动脚本与配置判法两行（SHARD_DRIVER_FILES，参数不管），
                                 最后一行是这条用例的登记行（键、路径与第三列）；stdout 打「<指纹> <文件数> <减去的文件数>」。
                                 --judging-digest 加一行判法摘要：这一份准入模块按 ast 从判法入口（CRASH_CASE_JUDGING_ENTRIES，连同分派表 COMMANDS 里
                                 CRASH_CASE_JUDGING_SUBCOMMANDS 那几项指的函数）顺着引用的模块级名字求闭包，闭包里每个定义的原文按名字排好，
                                 加分派表那几项、main 与模块级其余语句（不是 def / class / 名字赋值的：import、元组解包、if / try 块、`if __name__` 那一段）；
                                 分派表那几项的值不是函数名、闭包里用到非标准库的 import 时算不出（退 2）；判法之外的部分改了摘要不变
                                 （门禁 54 号给它，不再给整份模块与 54 号）
  crash-case-command <根> <键> <输入指纹> [--start-over]
                                 --full 跑这一条的命令与环境（54 号原样执行）：stdout 以 NUL 分隔，本机核数、线程数、explicit|default、续跑的进度目录，
                                 其后是整条命令（env 设 SINGLEFS_LAYER0_PROGRESS_DIRECTORY、SINGLEFS_LAYER0_INPUT_FINGERPRINT、SINGLEFS_LAYER0_THREADS，
                                 登记了 threads-variable= 的另把那个变量设成同一个数（盖掉调用方环境里的），
                                 --start-over 时设 SINGLEFS_LAYER0_START_OVER=1、不然清掉它，分片开关 SINGLEFS_LAYER0_SHARD 一律清掉，
                                 再起 cargo test --release …）；线程数：调用方设了
                                 SINGLEFS_LAYER0_THREADS 用它（explicit），没设取本机核数（default）；本机核数取 os.cpu_count() 与 CPU 亲和的小者，
                                 不认 OMP_NUM_THREADS。交不出退 2，原因打在 stdout
  crash-case-marker-check <根> <键> <指纹> <清单文件>
                                 这批输入那一格全绿标记在不在、作不作数：退 0 作数（stdout 第一行「ok <标记路径> <跑完的时刻>」，
                                 其后是标记里的计数行）；退 1 不作数（stdout 是原因，没有那一格时再比最近写的一格与这一次的清单）
  crash-case-judge <根> <键> <日志> <记录行文件>
                                 判 --full 跑那一条的日志：退 0 判绿（记进标记的行写进记录行文件，stdout 是线程那一句）；退 1 判红（stdout 逐条原因）。
                                 本机核数、线程数与是不是显式设的由它现取（与 crash-case-command 同一个算法，调用方在同一份环境里调），不收转过来的
  crash-case-record <根> <键> <指纹> <清单文件> <记录行文件> --files <数> --excluded <数> --started <时刻> --judged-root <路径>
                                 判绿之后写那一格全绿标记（同目录临时文件写完再改名换上）；stdout 打标记路径。线程分两格记：
                                 configured_worker_threads= 是配的（这条用例读的线程变量与它的值、显式设的还是取本机核数、本机几核，由它现取），
                                 started_worker_threads= 是 crash-case-judge 记下的起了几个（登记了 threads= 的取跑完那一行，没登记的写读不到）
  crash-case-marker-path <根> <键> <指纹>
                                 那一格全绿标记的路径（判红时阶段删它）
  crash-case-shardable <根> <键>  这条用例登记了 shard=across-machines 没有：退 0 登记了、退 1 没登记（stdout 各一句）、退 2 登记有错
  gate-preconditions <阶段所在的仓根> <阶段文件名>
                                 门禁开跑前判第三列的前提（command=、readwrite=、probe=）：退 0 齐了（stdout 一个字不打）；
                                 退 1 没齐、退 2 登记有错，两种都在 stdout 打 ✗、明细与出路，阶段照判红。
                                 读阶段自己那一份仓的登记表：判别力样本的目录里没有登记表，前提照样按真表判
  gate-record-environment <根> <阶段文件名>
                                 阶段判绿之后调：登记了 environment= 的，把环境与这一次的暂存树（SINGLEFS_STAGED_TREE）
                                 记进 git common-dir 的 singlefs-gate-environment.<阶段>，打一行「·」；没登记 environment=、
                                 不是 gate-staged.sh 起的，一个字不打、不记。记不下只打一行「!」，退 0（下一趟照跑，不判红）
  --selftest                     自证（格数由成功行现算）

强制重跑：设 SINGLEFS_EXPERIMENT_RERUN_REASON=<理由>。只越过 ①，不越过 ②；理由原样写进产物头。
弄坏开关（只给自证用，证明那几格会红）：ADMISSION_BREAK=skip-unchanged（不比指纹）、skip-preconditions（不判前提）、
raise-in-gate-reuse（门禁复用判定里抛异常，stage-must-run.sh 必须按要跑处理）、skip-gate-preconditions（门禁的前提不判）、
ignore-gate-environment（门禁复用判定不比环境）、skip-build-environment（构建环境不进指纹）、ignore-linked-directories（指向目录的
符号链接照旧滤掉）、exclude-mentioned-test-targets（别的测试目标被代码点了名也减掉）、threads-by-worker-count（照旧只看
worker_threads=1 判「只用了一个线程」，不看这一趟跑了几片）、first-definition-only（用例函数只看第一处定义）、
ignore-attribute-without-space（`# [ignore]` 不认）、concat-include-only（只认不带路径前缀的 include!(concat!(…))）、
computed-include-subtracts-non-test-files（有算出来的 include 时照样减 mutations.tsv 与 src/bin/）、runner-program-only（runner / wrapper
只按首词的程序进指纹）、whole-module-in-judging-digest（判法摘要换回整份准入模块）、judging-digest-without-dispatch（分派表那几项不进摘要）、
single-worker-threads-field（标记里线程只记一格 worker_threads=）、threads-ignore-shards（merge 那一行带 shards= 也不逐片判，照旧看 n 片之和）、
shardable-outside-judging-digest（分派表里 crash-case-shardable 那一项不进判法摘要）、shard-driver-outside-manifest（登记了 shard=across-machines 的
用例，驱动脚本与配置判法不进输入清单）、keep-caller-shard-switch（crash-case-command 起用例时不清调用方环境里的 SINGLEFS_LAYER0_SHARD）、
target-own-files-only（判 #[ignore] 只读测试目标自己的源文件，不顺 include! / #[path] / mod）、include-alias-subtracts（有 use 引进 include 族宏
照样减文件）、runner-arguments-from-configuration-directory-only（runner 参数的相对路径不从登记用例的包目录解）、digest-skips-other-statements
（模块级其余语句只进 `if __name__` 那一段）、digest-allows-non-name-dispatch（分派表的值不是名字照原文进）、digest-allows-imported-names
（判法闭包里的 import 不查）、judge-takes-forwarded-threads（crash-case-judge / record 照旧收 54 号转过来的核数与线程数）、cores-from-nproc
（本机核数照旧取 nproc）、thread-variable-ignored（threads-variable= 不认：命令只设、标记只记 SINGLEFS_LAYER0_THREADS）、
threads-skip-self-contained-finish-line（计数行自己带线程数的不判线程）。

管不到的：跑的是不是按今天的源码编出来的二进制（指纹按源码算，跑的是 target/ 里的旧二进制时两边对不上，
replay.sh 与 cargo run 开跑前都会重编）；登记的路径少写了一条（那条输入变了不会放行，要靠强制开关，登记行本身进指纹，
补登记之后自然放行）；产物头的指纹行是不是真由那一趟写的（由装置入口调本模块写，拷来的产物照样带着）。
崩溃枚举用例减去的文件认不出的：用拼出来的名字在运行期读别的测试文件（名字不以整词出现在任何代码里）、构建脚本（build.rs 与 package.build
指的那一份）之外的构建期代码按目录读 tests/（构建脚本 `mod` 进来的文件也算在这一类）、编译期算出来的名字（include!(concat!(…)) 这一类）
指到别的包的测试文件；这几种会让那份文件被减掉而它其实被读了。认得出形状、认不出读的是哪一份的两种按宽处理：包里有 include! / include_str! /
include_bytes! 的参数不以字符串字面量开头（套 concat!、env!、option_env! 或别的宏，宏名前带不带 ::core:: / std:: 都算）的，这个包的测试文件
一份都不减，任何一份 .rs 里有这种 include 的，crates/mutations.tsv 与 src/bin/ 下的也一份都不减；任何一份 .rs 里有 use 引进 include 族宏的
（`use core::include_str as grab;` 这一类，改不改名都算），哪个包、哪一类文件都不减（那种写法出现时多重跑，接受）；任何一个包的构建脚本
（注释去掉之后）出现整词 tests 的，哪个包的测试文件都不减。
runner / wrapper 指的程序与参数里的脚本按内容进指纹，它们再 source、再读的文件不进（不跑它就不知道它读什么）：runner 脚本 source 的那一份改了，
指纹不变、旧标记照样作数。判法摘要看不见的：靠 ast 里的静态引用求闭包，getattr、字符串拼出来的函数名、exec、globals() / setattr 改模块级名字
这一类引到的定义不进；标准库模块被改（monkeypatch）、判法运行时读的文件（登记表之外的）不进；Python 自身（解释器、标准库）升级不进。
元组解包、if / try / for 块里的定义与 import 整条原文进摘要（模块级其余语句）；分派表的值写成 lambda、判法挪进 import 的非标准库模块，算不出摘要（拒算）。
54 号不进指纹：核数与线程数不再由它转给 crash-case-judge / crash-case-record（它们自己现取），它里面还定着结论的只剩流程的次序
（开跑与跑完各算一次指纹、先判日志再写标记）与两处第二道判红（cargo 退非 0、发现日志）——这两处的第一道（test result 恰好 1 passed、
登记的计数行）在判法摘要里；改了它旧标记照样作数，由 --selftest 的「54 号」那几格核，快档的范围那一问照样把它算进去。双机分片的驱动脚本按内容进登记了 shard= 的用例，
它 source 的 preflight.sh 这一类不进；分片关着（照单机跑）时驱动脚本照样在指纹里，改了它这两条照样重跑（多跑，接受）。
用例函数标没标 #[ignore] 按同名的每一处 fn <名>( 判（连同顺着 include! / #[path] / mod 带进来的源文件）：同一个测试目标里别的模块有同名、
合法不标 ignore 的快用例时误判没标（多拒、自查判错，方向是多跑一步，接受）；带进来的判不出时（include! 参数不是字面量、mod 指的文件找不到、
cfg 关掉的 mod 指的文件不在也算）按没标算（多拒）。macro_rules! 里写的 `mod $名;` 看不见（macro_rules! 里的 include! 按参数不是字面量算，判不出）。
减得少的（改了照样让用例重跑）：几个测试目标共用的测试模块（tests/common_*/ 这一类）要顺着模块图才减得准，这里不走模块图；
包里有构建脚本、[[test]]、autotests 时整包不减。登记的四条里第二条流全量约 2.3 天（推的），这几种改动都会让它重跑。
减掉之后仍可能让用例红的：src/bin/ 下的文件编不过时 cargo test 也编不过（集成测试要先编本包的 bin），这一种交给构建与 clippy 那几道，
不靠崩溃枚举用例的标记。linker（配置文件里的 target.*.linker、环境变量 CARGO_TARGET_*_LINKER）指的程序不按内容进。
"""
import ast
import glob
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

REGISTRATION_TABLE = ".claude/gate.d/stage-inputs.tsv"
REPLAY_SCRIPT = "research/scripts/replay.sh"
RESULTS_DIRECTORY = "research/results"
GATE_REUSE_REFERENCE = "refs/sop/staged-green"
FORCE_RERUN_VARIABLE = "SINGLEFS_EXPERIMENT_RERUN_REASON"
BREAK_VARIABLE = "ADMISSION_BREAK"
HEADER_PREFIX = "E7INPUT "
HEADER_LINES_READ_AT_MOST = 20

EXIT_ADMITTED = 0
EXIT_REGISTRATION_ERROR = 2
EXIT_PRECONDITION_MISSING = 3
EXIT_INPUTS_UNCHANGED = 77
EXIT_GATE_MUST_RUN = 0
EXIT_GATE_MAY_SKIP = 10
EXIT_GATE_PRECONDITION_MISSING = 1
GATE_ENVIRONMENT_RECORD_PREFIX = "singlefs-gate-environment."
PROBE_TIMEOUT_SECONDS = 300
GATE_PRECONDITION_KINDS = ("command", "readwrite", "probe")

EXPERIMENT_KEY_FORM = re.compile(r"^E[0-9]+[A-Z0-9]*(/[a-z0-9][a-z0-9-]*)?$")
QUESTION_ROW_FORM = re.compile(r"^question-row=(?P<path>[^#]+)#(?P<row>[^:]+):(?P<pattern>.+)$")
PRODUCT_FIELD_FORM = re.compile(r"^product-field=(?P<key>[^:]+):(?P<line_name>[^:]+):(?P<field>[^=]+)=(?P<value>\S+)$")
REPLAY_ROW_FORM = re.compile(r"^(E[0-9]+[A-Z0-9]*)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$")
GATE_CONDITION_FORM = re.compile(r"^(?P<kind>command|readwrite|probe|environment)=(?P<value>\S+)$")

# 崩溃枚举用例（门禁 54 号逐条跑、逐条记全绿标记）
CRASH_CASE_KEY_PREFIX = "crash-case:"
CRASH_CASE_KEY_FORM = re.compile(r"^crash-case:[a-z0-9][a-z0-9-]*$")
CRASH_CASE_CONDITION_FORM = re.compile(r"^(?P<kind>test|count-line|exhaustive|threads|threads-variable|shard)=(?P<value>\S+)$")
# threads-variable= 的值：用例读线程数的环境变量名
THREADS_VARIABLE_FORM = re.compile(r"^[A-Z_][A-Z0-9_]*$")
# shard= 只认这一个值：两台机器各跑一片再 merge（双机分片，里程碑三第六项）
CRASH_CASE_SHARD_ACROSS_MACHINES = "across-machines"
# 登记了 shard=across-machines 的用例，54 号 --full 在分片开着时交给驱动脚本跑：它自己起两片与 merge 的 cargo、判两片与账本，不经 crash-case-command，
# 起法不在判法摘要里；它 eval 配置判法打出的赋值。这两份按内容进这条用例的输入清单（路径从被判的仓根起；不在的进「找不到」一句）
SHARD_DRIVER_FILES = ("research/scripts/layer0-shard-run.sh", "research/scripts/layer0-shard-configuration-check.sh")
# crash-case-command 起用例时从调用方环境里清掉的分片开关：调用方设着它，单机跑的那一趟会只跑一片或去 merge
LAYER0_SHARD_VARIABLE = "SINGLEFS_LAYER0_SHARD"
CRASH_CASE_TEST_FORM = re.compile(r"^(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
COUNT_LINE_PREFIX_FORM = re.compile(r"^[A-Z][A-Z0-9_]*$")
CRASH_CASE_MARKER_PREFIX = "singlefs-crash-case-green."
STAGE_MARKER_PREFIX = "singlefs-stage-green."          # checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿之后按输入指纹写的全绿标记（用户 2026-09-27 定：照 54 号写标记、只跑变了的）
LAYER0_PARALLEL_FINISHED_PREFIX = "LAYER0_PARALLEL_FINISHED "
PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\. 1 passed; 0 failed; ")
MARKER_DIFFERENCES_LISTED_AT_MOST = 20
POSITIVE_INTEGER_FORM = re.compile(r"[1-9][0-9]*")
# --full 起用例时设的环境变量与续跑的进度目录（crash_case_launch）
LAYER0_THREADS_VARIABLE = "SINGLEFS_LAYER0_THREADS"
LAYER0_START_OVER_VARIABLE = "SINGLEFS_LAYER0_START_OVER"
LAYER0_PROGRESS_DIRECTORY_NAME = "singlefs-layer0-progress"
# 判法摘要（crash_case_judging_digest_text）：崩溃枚举用例「日志判不判绿、标记作不作数、登记行怎么读、起用例的命令与环境」从这几个入口起求闭包
CRASH_CASE_JUDGING_ENTRIES = ("judge_crash_case_log", "judge_worker_threads", "crash_case_marker_problems", "read_crash_case_marker",
                              "check_crash_case_marker", "write_crash_case_marker", "crash_case_marker_path", "parse_crash_case",
                              "crash_case_of_key", "crash_cases_of", "crash_case_launch", "started_worker_threads_text",
                              "configured_worker_threads_text")
# 分派表 COMMANDS 里这几项子命令：「子命令 → 函数」进摘要，函数也当入口（有人把 crash-case-judge 指到别的函数，摘要跟着变）；
# crash-case-shardable 定 54 号 --full 这一条单机跑还是交给双机分片的驱动脚本
CRASH_CASE_JUDGING_SUBCOMMANDS = ("crash-case-command", "crash-case-judge", "crash-case-record", "crash-case-marker-check", "crash-case-marker-path",
                                  "crash-case-shardable")
# 原文进摘要、不顺着它的引用往下走的定义：main 引自证，顺下去自证就进来了
CRASH_CASE_JUDGING_LEAVES = ("main",)
CRASH_CASE_JUDGING_DIGEST_NAME = "<判法摘要：admission.py 里崩溃枚举用例的判法与起用例的命令>"
# 用例函数要标的属性：#[ignore] 或 #[ignore = "…"]，# 与 [ 之间许空白（rustc 认 `# [ignore]`）；cfg_attr 里的条件 ignore 不算：条件不成立时不带 --ignored 照样跑它
IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\s*\[\s*ignore\s*(?:\]|=)")
IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE = re.compile(r"^#\[\s*ignore\s*(?:\]|=)")  # 弄坏开关 ignore-attribute-without-space 换回的旧写法
ATTRIBUTE_START = re.compile(r"#!?\s*\[")
# 属性与 fn 之间可以夹的限定词（一次认一个，从 fn 往前剥）
FUNCTION_QUALIFIER_AT_END = re.compile(r'(?:\bpub(?:\s*\([^()]*\))?|\basync|\bunsafe|\bconst|\bextern(?:\s*"[^"]*")?)$')
# 编译期算出来的文件名：include! / include_str! / include_bytes!（宏名前带不带 ::core:: / std:: 这类路径都算）的参数不以字符串字面量开头——
# 套 concat!、env!、option_env! 或别的宏，读的是哪一份认不出，按宽处理（这个包的测试文件、mutations.tsv 与 src/bin/ 都不减）
COMPILE_TIME_COMPUTED_INCLUDE = re.compile(r'\binclude(?:_str|_bytes)?\s*!\s*[(\[{]\s*(?!b?r#*"|b?")\S')
# 弄坏开关 concat-include-only 换回的旧写法：只认紧跟着不带路径的 concat!
COMPILE_TIME_CONCATENATED_INCLUDE_ONLY = re.compile(r"\binclude(?:_str|_bytes)?!\s*[(\[{]\s*concat!")
# 崩溃枚举用例的输入里减去的、不是测试文件的那几份：仓根起的路径 → 为什么用例读不到它（有代码按文件名点名它时照留）
CRASH_CASE_FILES_NOT_READ = {"crates/mutations.tsv": "crates 变异表：checker-tier-crates-mutation-replay 按它改源码再跑点名的测试，用例编译期与运行期都不读它"}
# 集成测试拿本包 bin 的路径靠这个前缀的环境变量；没有代码读它时，src/bin/ 下的文件编不进也读不到用例
BIN_EXECUTABLE_VARIABLE_PREFIX = "CARGO_BIN_EXE_"

# 构建环境（build_environment_lines）：进指纹的配置文件名与环境变量
CARGO_CONFIGURATION_FILE_NAMES = ("config", "config.toml")
TOOLCHAIN_FILE_NAMES = ("rust-toolchain", "rust-toolchain.toml")
BUILD_ENVIRONMENT_EXACT_VARIABLES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")
BUILD_ENVIRONMENT_VARIABLE_FORMS = (re.compile(r"^CARGO_PROFILE_"), re.compile(r"^CARGO_BUILD_"),
                                    re.compile(r"^CARGO_TARGET_.+_RUSTFLAGS$"), re.compile(r"^CARGO_TARGET_.+_RUNNER$"))
# 名字对得上而不进指纹的：它们不改编出来的东西，而本仓的包装会设它们，进了指纹同一批输入在包装里外算出两个数、全绿标记永远对不上
BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT = {"CARGO_BUILD_JOBS": "只定编译并行度；research/scripts/capped.sh 与 mutate.sh 会设它"}
# 指一个程序的环境变量（值的首词是程序）：那个程序按内容另进一行——换了 runner、wrapper 的内容而值不变，编出来或跑起来的就不是同一样
BUILD_ENVIRONMENT_PROGRAM_VARIABLES = ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS = (re.compile(r"^CARGO_TARGET_.+_RUNNER$"),)
# 指编译器的环境变量：进它的值与它的 -V
BUILD_ENVIRONMENT_COMPILER_VARIABLES = ("RUSTC", "CARGO_BUILD_RUSTC")
# 配置文件 [build] 里指程序、按内容进的键（build.rustc 另取 -V，target.<…>.runner 另按内容进）
CONFIGURATION_PROGRAM_BUILD_KEYS = ("rustc-wrapper", "rustc-workspace-wrapper")


class RegistrationError(Exception):
    """登记表或调用写错了：退 2，出路是改登记表那一行或改调用。"""


class InputManifestError(Exception):
    """算不出输入指纹：git 列不出文件、读文件出错、或 cargo -V / rustc -V 跑不出来。"""


class RegistrationRow:
    def __init__(self, key, input_paths, conditions, line_number, comment):
        self.key = key
        self.input_paths = input_paths
        self.conditions = conditions
        self.line_number = line_number
        self.comment = comment

    def fingerprint_text(self):
        """进指纹的只有键与路径：准入条件判的是「能不能跑」，不是「算什么」，改它不该让没变的输入放行。"""
        return f"{self.key}\t{' '.join(self.input_paths)}\n"

    def text_with_conditions(self):
        """崩溃枚举用例的登记行进指纹时连第三列一起进：第三列定的是跑哪一条用例、日志怎么判，改了前一趟的结论就不再作数。"""
        return f"{self.key}\t{' '.join(self.input_paths)}\t{' '.join(self.conditions)}\n"


def break_is_set(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


def say(message):
    print(message, file=sys.stderr)


# ── 登记表 ────────────────────────────────────────────────────────────────────

def read_registration_rows(root):
    """整张表的行；表不在返回空表。列的切法与原来 stage-must-run.sh 的 awk -F'\\t' 相同：第一列是键，第二列是路径。"""
    table_path = os.path.join(root, REGISTRATION_TABLE)
    if not os.path.isfile(table_path):
        return []
    rows = []
    with open(table_path, encoding="utf-8", errors="surrogateescape") as handle:
        for line_number, line in enumerate(handle.read().split("\n"), 1):
            if not line.strip() or line.startswith("#"):
                continue
            columns = line.split("\t")
            conditions = []
            comment = ""
            for column in columns[2:]:
                if column.lstrip().startswith("#"):
                    comment = column.strip().lstrip("#").strip()
                    break
                conditions.extend(column.split())
            input_paths = columns[1].split() if len(columns) > 1 else []
            rows.append(RegistrationRow(columns[0], input_paths, conditions, line_number, comment))
    return rows


def rows_of_key(rows, key):
    return [row for row in rows if row.key == key]


def resolve_input_paths(rows, key, referencing_keys=()):
    """这个键登记的全部路径与贡献了路径的行；@<别的键> 展开成那个键的路径。"""
    own_rows = rows_of_key(rows, key)
    if not own_rows:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {key} 这一行")
    resolved_paths = []
    contributing_rows = []
    for row in own_rows:
        contributing_rows.append(row)
        for input_path in row.input_paths:
            if not input_path.startswith("@"):
                resolved_paths.append(input_path)
                continue
            referenced_key = input_path[1:]
            if referenced_key == key or referenced_key in referencing_keys:
                raise RegistrationError(f"{REGISTRATION_TABLE} 第 {row.line_number} 行：{key} 的 @{referenced_key} 引用绕成了圈")
            referenced_paths, referenced_rows = resolve_input_paths(rows, referenced_key, referencing_keys + (key,))
            resolved_paths.extend(referenced_paths)
            contributing_rows.extend(referenced_rows)
    return resolved_paths, contributing_rows


def parse_condition(token):
    """把一条准入条件切成 (种类, 各段)；认不出、正则编不过都是登记错。"""
    question_row = QUESTION_ROW_FORM.match(token)
    if question_row:
        try:
            re.compile(question_row.group("pattern"))
        except re.error as error:
            raise RegistrationError(f"准入条件 {token} 的正则编不过：{error}") from error
        return ("question-row", question_row.groupdict())
    product_field = PRODUCT_FIELD_FORM.match(token)
    if product_field:
        return ("product-field", product_field.groupdict())
    raise RegistrationError(f"认不出的准入条件：{token}（只认 question-row=<文件>#<行号>:<正则> 与 product-field=<实验键>:<行名>:<字段>=<值>）")


def parse_gate_condition(token):
    """门禁行第三列的一条，切成 (种类, 各段)：command / readwrite 是 {"value"}，probe / environment 是 {"script", "arguments"}。
    认不出、脚本写成绝对路径或带 .. 都是登记错。"""
    match = GATE_CONDITION_FORM.match(token)
    if not match:
        raise RegistrationError(f"认不出的门禁准入条件：{token}（门禁行只认 command=<可执行名>、readwrite=<路径>、"
                                "probe=<仓内脚本>[:<参数>…]、environment=<仓内脚本>[:<参数>…]）")
    kind, value = match.group("kind"), match.group("value")
    if kind in ("probe", "environment"):
        script, *arguments = value.split(":")
        if not script or script.startswith("/") or ".." in script.split("/"):
            raise RegistrationError(f"门禁准入条件 {token} 的脚本要写成仓库根起的相对路径（不许绝对路径、不许带 ..）")
        return kind, {"script": script, "arguments": arguments}
    return kind, {"value": value}


def self_invalidating_input_paths(input_paths):
    """实验登记的路径里会罩住「存产物」那一步本身要改的文件的：research/results/ 整个目录（新产物存进来就改了它），
    与 research/scripts/replay.sh（跑完要把登记行指到新产物）。罩住了，新产物一存进来指纹就对不上自己，准入永远判不了「没变」。"""
    self_edited_targets = (RESULTS_DIRECTORY + "/", REPLAY_SCRIPT)
    return [input_path for input_path in input_paths
            if any(input_path == target or (input_path.endswith("/") and target.startswith(input_path)) for target in self_edited_targets)]


def lint_registration_rows(rows, root=None):
    """登记表自己写对没有：实验键的形状、准入条件认不认得（实验行只认实验的种类、门禁行只认门禁的种类）、@ 引用解不解得开；
    给了 root 还核门禁行 probe= / environment= 指的脚本在不在。返回问题清单。"""
    problems = []
    for row in rows:
        is_experiment = bool(EXPERIMENT_KEY_FORM.match(row.key))
        is_crash_case = bool(CRASH_CASE_KEY_FORM.match(row.key))
        if not is_experiment and not is_crash_case and not row.key.endswith(".sh"):
            problems.append(f"第 {row.line_number} 行：键 {row.key} 既不是门禁阶段文件名（*.sh）、实验键（E<号>[/<方式>]），"
                            "也不是崩溃枚举用例（crash-case:<小写字母、数字、->）")
        if not row.input_paths:
            problems.append(f"第 {row.line_number} 行：{row.key} 没有登记路径")
        if not is_experiment and any(path.startswith("@") for path in row.input_paths):
            problems.append(f"第 {row.line_number} 行：{row.key} 用了 @ 引用，门禁与崩溃枚举用例的判定还不展开它")
        if is_crash_case:
            try:
                crash_case = parse_crash_case(row)
                if root is not None:
                    crash_case_test_files(root, crash_case)
            except RegistrationError as error:
                problems.append(f"第 {row.line_number} 行：{error}")
            continue
        for token in row.conditions:
            try:
                if is_experiment:
                    parse_condition(token)
                    continue
                kind, parts = parse_gate_condition(token)
                if root is not None and kind in ("probe", "environment") and not os.path.isfile(os.path.join(root, parts["script"])):
                    problems.append(f"第 {row.line_number} 行：门禁行 {row.key} 的 {token} 指的脚本 {parts['script']} 不在")
            except RegistrationError as error:
                problems.append(f"第 {row.line_number} 行：{error}")
        if is_experiment:
            try:
                resolve_input_paths(rows, row.key)
            except RegistrationError as error:
                problems.append(f"第 {row.line_number} 行：{error}")
            for input_path in self_invalidating_input_paths(row.input_paths):
                problems.append(f"第 {row.line_number} 行：实验 {row.key} 登记了 {input_path}，它罩住存产物那一步要改的文件，新产物一存进来指纹就对不上自己")
    crash_case_lines = {}
    for row in rows:
        if CRASH_CASE_KEY_FORM.match(row.key):
            crash_case_lines.setdefault(row.key, []).append(row.line_number)
    for key, line_numbers in crash_case_lines.items():
        if len(line_numbers) > 1:
            problems.append(f"第 {'、'.join(str(number) for number in line_numbers)} 行：崩溃枚举用例 {key} 登记了 {len(line_numbers)} 行，一条用例只许一行")
    return problems


# ── 输入清单与指纹 ────────────────────────────────────────────────────────────

def sha256_of_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def sha256sum_line(digest_hex, name_bytes):
    """GNU sha256sum 的输出行：文件名带反斜杠、换行或回车时整行前面加一个反斜杠、名字里的这三样转义。"""
    if b"\\" in name_bytes or b"\n" in name_bytes or b"\r" in name_bytes:
        escaped_name = name_bytes.replace(b"\\", b"\\\\").replace(b"\n", b"\\n").replace(b"\r", b"\\r")
        return b"\\" + digest_hex.encode() + b"  " + escaped_name + b"\n"
    return digest_hex.encode() + b"  " + name_bytes + b"\n"


def listed_input_files(root, input_paths):
    """git 眼里这些路径下磁盘上真有的文件（已跟踪的加没被忽略的未跟踪的），按字节序去重排好。
    git 把指向目录的符号链接当一个文件列出来：它指向的目录里的文件按「链接路径/目录里的相对路径」逐个计入（绕回自己的只走一遍）；
    指不到东西的链接照旧不计。"""
    completed = subprocess.run(
        ["git", "-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *input_paths],
        capture_output=True)
    if completed.returncode != 0:
        raise InputManifestError(f"git ls-files 退 {completed.returncode}：{completed.stderr.decode('utf-8', 'replace').strip()}")
    names = sorted({name for name in completed.stdout.split(b"\0") if name})
    root_bytes = os.fsencode(root)
    files = []
    for name in names:
        full_path = os.path.join(root_bytes, name)
        if os.path.isfile(full_path):
            files.append(name)
        elif os.path.islink(full_path) and os.path.isdir(full_path) and not break_is_set("ignore-linked-directories"):
            files.extend(files_under_linked_directory(root_bytes, name))
    return sorted(set(files))


def files_under_linked_directory(root_bytes, link_name):
    """指向目录的符号链接 link_name（仓根起）底下的普通文件，名字写成「链接路径/相对路径」；目录里再有绕回去的链接，同一个目录只走一遍。"""
    link_path = os.path.join(root_bytes, link_name)
    walked_directories = set()
    files = []
    for directory, subdirectories, file_names in os.walk(link_path, followlinks=True):
        real_directory = os.path.realpath(directory)
        if real_directory in walked_directories:
            subdirectories[:] = []
            continue
        walked_directories.add(real_directory)
        subdirectories.sort()
        for file_name in sorted(file_names):
            full_path = os.path.join(directory, file_name)
            if os.path.isfile(full_path):
                files.append(link_name + b"/" + os.path.relpath(full_path, link_path))
    return files


def unlisted_input_paths(root, input_paths):
    """登记的路径里 git 一个文件都列不出来的那几条（写错的路径在比对里永远算「没变」，要单独拦）。"""
    return [input_path for input_path in input_paths if not listed_input_files(root, [input_path])]


def toolchain_line(root):
    """`cargo -V && rustc -V` 原样输出去掉末尾换行、再补一个换行的 sha256，名字 <工具链：…>（门禁 54 号的输入指纹也是这一行）。"""
    outputs = []
    for command in (["cargo", "-V"], ["rustc", "-V"]):
        try:
            completed = subprocess.run(command, cwd=root, capture_output=True)
        except OSError as error:
            raise InputManifestError(f"{' '.join(command)} 起不来：{error}") from error
        if completed.returncode != 0:
            raise InputManifestError(f"{' '.join(command)} 退 {completed.returncode}")
        outputs.append(completed.stdout)
    versions = b"".join(outputs).rstrip(b"\n")
    if not versions:
        raise InputManifestError("cargo -V 与 rustc -V 一个字都没打")
    name = "<工具链：" + versions.decode("utf-8", "surrogateescape").replace("\n", "；") + ">"
    return (name, versions + b"\n")


def read_configuration_file(path):
    try:
        with open(path, "rb") as handle:
            return handle.read()
    except OSError as error:
        raise InputManifestError(f"读不了构建环境里的 {path}：{error}") from error


def build_environment_lines(root, environment=None):
    """进指纹的构建环境（写法见文件头「输入指纹里的构建环境」）：[(名字, 内容)]，一样都没有时是空表（清单与不带它时逐字节相同）。
    名字里只写是哪一类、不写绝对路径：同一批内容在主工作区与临时 worktree 里要算出同一个数。
    cargo 按当前目录往上一层层读配置、最后读 CARGO_HOME 的；CARGO_HOME 那一份同时是某一层的，只按 CARGO_HOME 算一次。"""
    if break_is_set("skip-build-environment"):
        return []
    environment = os.environ if environment is None else environment
    argument_directories = runner_argument_directories(root)
    cargo_home = environment.get("CARGO_HOME") or os.path.join(environment.get("HOME") or os.path.expanduser("~"), ".cargo")
    cargo_home_files = [os.path.join(cargo_home, name) for name in CARGO_CONFIGURATION_FILE_NAMES]
    cargo_home_real_paths = {os.path.realpath(path) for path in cargo_home_files}
    lines = []
    directory = os.path.realpath(root)
    while True:
        for name in CARGO_CONFIGURATION_FILE_NAMES:
            path = os.path.join(directory, ".cargo", name)
            if os.path.isfile(path) and os.path.realpath(path) not in cargo_home_real_paths:
                content = read_configuration_file(path)
                lines.append((f"<构建环境：目录层级里的 .cargo/{name}>", content))
                lines += configured_program_lines(f"目录层级里的 .cargo/{name}", content, directory, root, environment, argument_directories)
        for name in TOOLCHAIN_FILE_NAMES:
            path = os.path.join(directory, name)
            if os.path.isfile(path):
                lines.append((f"<构建环境：目录层级里的 {name}>", read_configuration_file(path)))
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    for path in cargo_home_files:
        if os.path.isfile(path):
            content = read_configuration_file(path)
            lines.append((f"<构建环境：CARGO_HOME 下的 {os.path.basename(path)}>", content))
            lines += configured_program_lines(f"CARGO_HOME 下的 {os.path.basename(path)}", content, os.path.dirname(cargo_home), root, environment,
                                              argument_directories)
    for variable in sorted(environment):
        if variable in BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT:
            continue
        if variable in BUILD_ENVIRONMENT_EXACT_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS):
            lines.append((f"<构建环境：环境变量 {variable}>", environment[variable].encode("utf-8", "surrogateescape")))
    for variable in sorted(environment):
        if variable in BUILD_ENVIRONMENT_PROGRAM_VARIABLES or any(form.match(variable) for form in BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS):
            lines.append((f"<构建环境：环境变量 {variable} 指的程序>", program_contents(environment[variable].split(), root, environment,
                                                                                argument_directories)))
    for variable in BUILD_ENVIRONMENT_COMPILER_VARIABLES:
        if variable in environment:
            compiler = environment[variable]
            lines.append((f"<构建环境：{variable} 的值与它的 -V>", compiler.encode("utf-8", "surrogateescape") + b"\n"
                          + compiler_version(compiler, root, environment, f"{variable}={compiler}")))
    return lines


def resolved_program(program, base_directory, environment):
    """cargo 配置或环境变量里写的程序：带斜杠的按 base_directory 解开相对路径，不带斜杠的在 environment 的 PATH 里找；找不到交 None。"""
    if not program:
        return None
    if "/" in program:
        return program if os.path.isabs(program) else os.path.join(base_directory, program)
    return shutil.which(program, path=environment.get("PATH"))


def runner_argument_directories(root):
    """runner / wrapper 参数里的相对路径另从哪几个目录解：登记的每条崩溃枚举用例的包目录（cargo 起 runner 时当前目录是包目录）。
    交 [(仓根起的包目录, 绝对路径)]，按包目录排好；登记表读不了、写错的交空表（登记行本身另有自查）。弄坏开关
    runner-arguments-from-configuration-directory-only 下交空表（只从 .cargo 那一层 / 仓根解，改前的判法）。"""
    if break_is_set("runner-arguments-from-configuration-directory-only"):
        return []
    try:
        cases = crash_cases_of(read_registration_rows(root))
    except (RegistrationError, OSError):
        return []
    packages = workspace_package_directories(root)
    relatives = sorted({packages[case.package] for case in cases if case.package in packages})
    return [(relative, os.path.join(root, relative)) for relative in relatives]


def program_contents(words, base_directory, environment, argument_directories=()):
    """首词是程序、其后是参数的一个值：交回那个程序的内容（进指纹），找不到的交「找不到」一句（值本身另有一行进指纹）；
    首词之后指到现存文件的参数（绝对路径，或从 base_directory 起的相对路径）接着按内容进：`bash tools/r.sh` 这一类，真正定行为的脚本在参数里；
    相对路径另从 argument_directories（runner_argument_directories：登记用例的包目录，cargo 在那里起 runner）逐个解，解得到的都进，
    名字里写从哪个包目录解的（仓根起，不写绝对路径）。那个脚本再 source、再读的文件不进（文件头「管不到的」）。"""
    program = words[0] if words else ""
    path = resolved_program(program, base_directory, environment)
    contents = read_configuration_file(path) if path and os.path.isfile(path) else f"找不到：{program}".encode("utf-8", "surrogateescape")
    if break_is_set("runner-program-only"):
        return contents
    for argument in words[1:]:
        candidate = argument if os.path.isabs(argument) else os.path.join(base_directory, argument)
        if os.path.isfile(candidate):
            contents += b"\n<" + argument.encode("utf-8", "surrogateescape") + b">\n" + read_configuration_file(candidate)
        if os.path.isabs(argument):
            continue
        for relative_directory, directory in argument_directories:
            candidate = os.path.join(directory, argument)
            if os.path.isfile(candidate):
                contents += (b"\n<" + argument.encode("utf-8", "surrogateescape") + "，从包目录 ".encode("utf-8")
                             + relative_directory.encode("utf-8", "surrogateescape") + " 解>\n".encode("utf-8") + read_configuration_file(candidate))
    return contents


def compiler_version(program, root, environment, described_as):
    """编译器的 `-V`（在仓根跑）；起不来、退非 0、一个字不打都抛 InputManifestError。"""
    try:
        completed = subprocess.run([program, "-V"], cwd=root, capture_output=True, env=dict(environment))
    except OSError as error:
        raise InputManifestError(f"{described_as} 起不来：{error}") from error
    if completed.returncode != 0 or not completed.stdout.strip():
        raise InputManifestError(f"{described_as} -V 退 {completed.returncode}、打了「{completed.stdout.decode('utf-8', 'replace').strip()}」")
    return completed.stdout


def configured_program_lines(label, content, base_directory, root, environment, argument_directories=()):
    """一份 cargo 配置文件里指程序的键（build.rustc 取 -V；build.rustc-wrapper、build.rustc-workspace-wrapper、target.<…>.runner 按内容）：
    [(名字, 内容)]。base_directory 是 .cargo 所在的那一层（相对路径从它起）。读不成 TOML 的不另进（原文已经进了，cargo 也读不了它）。"""
    try:
        settings = tomllib.loads(content.decode("utf-8"))
    except (UnicodeDecodeError, ValueError):
        return []
    lines = []
    build = settings.get("build") if isinstance(settings.get("build"), dict) else {}
    compiler = build.get("rustc")
    if isinstance(compiler, str):
        lines.append((f"<构建环境：{label} 里 build.rustc 的 -V>",
                      compiler_version(resolved_program(compiler, base_directory, environment) or compiler, root, environment,
                                       f"{label} 里的 build.rustc = {compiler}")))
    for key in CONFIGURATION_PROGRAM_BUILD_KEYS:
        if isinstance(build.get(key), str):
            lines.append((f"<构建环境：{label} 里 build.{key} 指的程序>", program_contents([build[key]], base_directory, environment, argument_directories)))
    targets = settings.get("target") if isinstance(settings.get("target"), dict) else {}
    for target_name in sorted(targets):
        runner = targets[target_name].get("runner") if isinstance(targets[target_name], dict) else None
        words = runner.split() if isinstance(runner, str) else [str(word) for word in runner] if isinstance(runner, list) else []
        if words:
            lines.append((f"<构建环境：{label} 里 target.{target_name}.runner 指的程序>", program_contents(words, base_directory, environment,
                                                                                                      argument_directories)))
    return lines


def input_manifest(root, input_paths, named_lines):
    """逐文件清单加末尾几行「sha256  <名字>」；返回 (清单字节, 整张的 sha256, 文件数)。"""
    listed_files = listed_input_files(root, input_paths)
    if not listed_files:
        raise InputManifestError(f"登记的路径（{' '.join(input_paths)}）下 git 一个文件都列不出来")
    return manifest_of_files(root, listed_files, named_lines)


def manifest_of_files(root, listed_files, named_lines):
    """给定的文件（仓根起、按字节序排好）逐个「sha256  路径」，再加末尾几行「sha256  <名字>」；返回 (清单字节, 整张的 sha256, 文件数)。"""
    manifest = bytearray()
    for name in listed_files:
        try:
            digest_hex = sha256_of_file(os.path.join(os.fsencode(root), name))
        except OSError as error:
            raise InputManifestError(f"读不了 {name.decode('utf-8', 'replace')}：{error}") from error
        manifest += sha256sum_line(digest_hex, name)
    for name, content in named_lines:
        manifest += f"{hashlib.sha256(content).hexdigest()}  {name}\n".encode("utf-8", "surrogateescape")
    return bytes(manifest), hashlib.sha256(bytes(manifest)).hexdigest(), len(listed_files)


def experiment_fingerprint(root, rows, key):
    """实验的输入指纹：登记路径下的逐文件清单 + 登记行本身 + 工具链 + 构建环境。返回 (sha256, 文件数, 路径)。"""
    input_paths, contributing_rows = resolve_input_paths(rows, key)
    registration_text = "".join(row.fingerprint_text() for row in contributing_rows).encode("utf-8", "surrogateescape")
    named_lines = [(f"<登记行：{key}>", registration_text), toolchain_line(root)] + build_environment_lines(root)
    _manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
    return fingerprint, file_count, input_paths


# ── 产物 ──────────────────────────────────────────────────────────────────────

def header_fields(header_line):
    """E7INPUT 行的字段；reason= 取到行尾（理由里可以有空格）。"""
    body = header_line[len(HEADER_PREFIX):]
    reason_start = body.find(" reason=")
    reason = None
    if reason_start >= 0:
        reason = body[reason_start + len(" reason="):]
        body = body[:reason_start]
    fields = dict(token.split("=", 1) for token in body.split() if "=" in token)
    if reason is not None:
        fields["reason"] = reason
    return fields


def header_lines_of(product_path):
    header_lines = []
    try:
        with open(product_path, "rb") as handle:
            for _line_index in range(HEADER_LINES_READ_AT_MOST):
                raw_line = handle.readline()
                if not raw_line:
                    break
                text = raw_line.decode("utf-8", "replace").rstrip("\n")
                if not text.startswith(HEADER_PREFIX):
                    break
                header_lines.append(text)
    except OSError:
        return []
    return header_lines


class StoredFingerprint:
    def __init__(self, product_path, fingerprint, modified_at):
        self.product_path = product_path
        self.fingerprint = fingerprint
        self.modified_at = modified_at


def stored_fingerprints(root, key):
    """research/results/ 下（含子目录）头上记着这个键的输入指纹的每一份产物。"""
    found = []
    results_directory = os.path.join(root, RESULTS_DIRECTORY)
    for directory, _subdirectories, file_names in os.walk(results_directory):
        for file_name in sorted(file_names):
            product_path = os.path.join(directory, file_name)
            for header_line in header_lines_of(product_path):
                fields = header_fields(header_line)
                if fields.get("name") == "input_fingerprint" and fields.get("key") == key and fields.get("sha256"):
                    found.append(StoredFingerprint(os.path.relpath(product_path, root), fields["sha256"], os.path.getmtime(product_path)))
                    break
    return found


def replay_products_of(root, experiment_key):
    """research/scripts/replay.sh 登记表里这个实验号那几行的留存产物（仓库根起的路径）。"""
    replay_path = os.path.join(root, REPLAY_SCRIPT)
    if not os.path.isfile(replay_path):
        return []
    products = []
    with open(replay_path, encoding="utf-8", errors="replace") as handle:
        for line in handle.read().split("\n"):
            match = REPLAY_ROW_FORM.match(line)
            if match and match.group(1) == experiment_key:
                products.append(os.path.join(RESULTS_DIRECTORY, match.group(4)))
    return products


def judge_inputs_unchanged(root, key, fingerprint):
    """① 输入没变：返回 (没变?, 说明)。有一份产物头上记着同一个键、同一个指纹就算没变。"""
    stored = stored_fingerprints(root, key)
    replay_products = replay_products_of(root, key) if "/" not in key else []
    matching = [entry for entry in stored if entry.fingerprint == fingerprint]
    if matching:
        preferred = next((entry for entry in matching if entry.product_path in replay_products), matching[0])
        return True, f"{preferred.product_path} 头上的输入指纹与今天的相同（{fingerprint[:12]}）"
    if stored:
        newest = max(stored, key=lambda entry: entry.modified_at)
        return False, f"输入自 {newest.product_path} 以来变了（那一份 {newest.fingerprint[:12]}，今天 {fingerprint[:12]}）"
    if replay_products:
        described = []
        for product in replay_products:
            if os.path.isfile(os.path.join(root, product)):
                described.append(f"{product} 没有指纹行")
            else:
                described.append(f"{product} 不在树里")
        return False, f"上一份产物没有指纹，判不了（replay.sh 登记的 {'；'.join(described)}；research/results/ 里也没有别的一份带 {key} 的指纹）"
    return False, f"上一份产物没有指纹，判不了（research/results/ 里没有一份头上带 {key} 输入指纹的产物）"


# ── 前提 ──────────────────────────────────────────────────────────────────────

def markdown_row_status(root, path, row_label):
    """那份文件里第一格是 row_label 的第一行表格行的最后一格；文件或行不在返回 None 与原因。"""
    full_path = os.path.join(root, path)
    if not os.path.isfile(full_path):
        return None, f"{path} 不在"
    with open(full_path, encoding="utf-8", errors="replace") as handle:
        for line in handle.read().split("\n"):
            stripped = line.strip()
            if not stripped.startswith("|"):
                continue
            cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", stripped)[1:-1]]
            if cells and cells[0] == row_label:
                return cells[-1], None
    return None, f"{path} 里没有第一格是 {row_label} 的表格行"


def product_field_value(root, experiment_key, line_name, field):
    """replay.sh 登记给这个实验键的产物里，最后一行 `E7RESULT name=<行名>` 的字段值；取不到返回 None 与原因。"""
    products = replay_products_of(root, experiment_key)
    if len(products) != 1:
        raise RegistrationError(f"product-field 要 replay.sh 登记表里 {experiment_key} 恰好一行，实际 {len(products)} 行")
    product = products[0]
    full_path = os.path.join(root, product)
    if not os.path.isfile(full_path):
        return None, product, f"{product} 不在树里"
    last_line = None
    with open(full_path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.startswith(f"E7RESULT name={line_name} ") or line.rstrip("\n") == f"E7RESULT name={line_name}":
                last_line = line.rstrip("\n")
    if last_line is None:
        return None, product, f"{product} 里没有 name={line_name} 这一行"
    fields = dict(token.split("=", 1) for token in last_line.split()[1:] if "=" in token)
    if field not in fields:
        return None, product, f"{product} 的 name={line_name} 行没有 {field} 字段"
    return fields[field], product, None


def judge_preconditions(root, key, conditions):
    """② 前提：返回 (没齐的清单, 齐了的清单)，每项是一句说明。登记写错抛 RegistrationError。"""
    missing = []
    satisfied = []
    for token in conditions:
        kind, parts = parse_condition(token)
        if kind == "question-row":
            status, problem = markdown_row_status(root, parts["path"], parts["row"])
            if problem is not None:
                raise RegistrationError(f"{key} 的前提 {token} 读不到：{problem}")
            if re.search(parts["pattern"], status):
                satisfied.append(f"{parts['path']} 第 {parts['row']} 行的状态「{status}」匹配「{parts['pattern']}」")
            else:
                missing.append(f"{parts['path']} 第 {parts['row']} 行的状态是「{status}」，要匹配「{parts['pattern']}」")
        elif kind == "product-field":
            value, product, problem = product_field_value(root, parts["key"], parts["line_name"], parts["field"])
            if problem is not None:
                missing.append(f"{problem}（要 name={parts['line_name']} 的 {parts['field']}={parts['value']}）")
            elif value == parts["value"]:
                satisfied.append(f"{product} 的 name={parts['line_name']} {parts['field']}={value}")
            else:
                missing.append(f"{product} 的 name={parts['line_name']} {parts['field']}={value}，要 {parts['value']}")
    return missing, satisfied


# ── 实验的准入 ────────────────────────────────────────────────────────────────

def admit_experiment(root, key):
    """返回 (退出码, 产物头的行)。说明打到 stderr。"""
    rows = read_registration_rows(root)
    own_rows = rows_of_key(rows, key)
    if not EXPERIMENT_KEY_FORM.match(key) or not own_rows:
        say(f"  ✗ 准入登记里没有实验键 {key}（{REGISTRATION_TABLE}）")
        say(f"     → 怎么办：在 {REGISTRATION_TABLE} 加一行「{key}<制表符><它读的路径，空格分隔>[<制表符><准入条件>]<制表符>#<为什么>」，"
            "照 E142 那一行写；装置入口传的键与登记的键逐字相同")
        return EXIT_REGISTRATION_ERROR, []
    conditions = [token for row in own_rows for token in row.conditions]
    try:
        input_paths, _contributing_rows = resolve_input_paths(rows, key)
        self_invalidating = self_invalidating_input_paths(input_paths)
        if self_invalidating:
            raise RegistrationError(f"登记给 {key} 的路径罩住了存产物那一步要改的文件（{' '.join(self_invalidating)}）：新产物一存进来指纹就对不上自己；"
                                    "只登记装置真读的那一份产物文件，不登记整个 research/results/ 与 replay.sh")
        unlisted = unlisted_input_paths(root, input_paths)
        if unlisted:
            raise RegistrationError(f"登记给 {key} 的路径里有 {len(unlisted)} 条 git 一个文件都列不出来：{' '.join(unlisted)}")
        for token in conditions:
            parse_condition(token)
        fingerprint, file_count, _input_paths = experiment_fingerprint(root, rows, key)
        missing, satisfied = ([], []) if break_is_set("skip-preconditions") else judge_preconditions(root, key, conditions)
    except RegistrationError as error:
        say(f"  ✗ {key} 的准入登记有错：{error}")
        say(f"     → 怎么办：改 {REGISTRATION_TABLE} 里 {key} 那一行（路径拼错、目录少了斜杠、指到不存在的文件、条件写错），"
            "改到 python3 research/scripts/admission.py paths <根> <键> 列得出、这里不再报错为止")
        return EXIT_REGISTRATION_ERROR, []
    except InputManifestError as error:
        say(f"  ✗ 算不出 {key} 的输入指纹：{error}")
        say("     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- <登记的路径> 看列不列得出文件，"
            "再跑 cargo -V && rustc -V 看工具链在不在（bash .claude/scripts/env.sh 报缺什么）")
        return EXIT_REGISTRATION_ERROR, []
    for description in satisfied:
        say(f"  ✓ 前提齐了：{description}")
    if missing:
        say(f"  ✗ 前提没齐，不许开跑 {key}：共 {len(missing)} 条")  # gate-lint:summary
        for description in missing:
            say(f"       {description}")  # gate-lint:detail
        say(f"     → 怎么办：这几条前提登记在 {REGISTRATION_TABLE} 的 {key} 那一行（跑前登记写的「只在……之后跑」）；"
            "先把它们办齐（问题单那一行判完写成够判、上一段产物的判决字段是 true），再跑。"
            "条件本身写错了就改登记表那一行，不许绕过；强制开关只越过「输入没变」，越不过这一条")
        return EXIT_PRECONDITION_MISSING, []
    if break_is_set("skip-unchanged"):
        unchanged, description = False, "（弄坏开关 skip-unchanged：没比指纹）"
    else:
        unchanged, description = judge_inputs_unchanged(root, key, fingerprint)
    header = [f"{HEADER_PREFIX}name=input_fingerprint key={key} sha256={fingerprint} files={file_count}"]
    force_reason = os.environ.get(FORCE_RERUN_VARIABLE)
    if force_reason is not None:
        cleaned_reason = " ".join(force_reason.split())
        if not cleaned_reason:
            say(f"  ✗ {FORCE_RERUN_VARIABLE} 设了却是空的：强制重跑要写理由")
            say(f"     → 怎么办：写成 {FORCE_RERUN_VARIABLE}=<为什么输入没变还要重跑>，理由会原样写进产物头")
            return EXIT_REGISTRATION_ERROR, []
        header.append(f"{HEADER_PREFIX}name=forced_rerun key={key} overrode_unchanged={'true' if unchanged else 'false'} reason={cleaned_reason}")
        say(f"  ✓ 放行（{FORCE_RERUN_VARIABLE} 给了理由，理由写进产物头）：{description}")
        return EXIT_ADMITTED, header
    if unchanged:
        say(f"  ✗ 拒绝开跑 {key}：输入自上次产物以来没变，不必重跑——{description}")
        say(f"     → 怎么办：沿用那一份的结论；确要重跑，设 {FORCE_RERUN_VARIABLE}=<理由> 再跑，理由会写进产物。"
            f"输入清单登记在 {REGISTRATION_TABLE} 的 {key} 那一行，少登了一条就补上")
        return EXIT_INPUTS_UNCHANGED, []
    say(f"  ✓ 放行 {key}：{description}")
    return EXIT_ADMITTED, header


def stored_product_status(root, key):
    """给门禁 doc-experiments 的 evidence-in-repo 格：这个实验键的产物跟不跟得上今天的输入。
    返回 (状态, 说明)：unregistered（没登记准入）、unfingerprinted（没有一份带指纹的产物）、
    matched（有一份的指纹与今天的相同）、stale（带指纹的都与今天的不同）。算不出指纹抛 InputManifestError。"""
    rows = read_registration_rows(root)
    if not rows_of_key(rows, key):
        return "unregistered", ""
    stored = stored_fingerprints(root, key)
    if not stored:
        return "unfingerprinted", ""
    fingerprint, _file_count, _input_paths = experiment_fingerprint(root, rows, key)
    matching = [entry for entry in stored if entry.fingerprint == fingerprint]
    if matching:
        return "matched", f"{matching[0].product_path}（指纹 {fingerprint[:12]}）"
    newest = max(stored, key=lambda entry: entry.modified_at)
    return "stale", f"最新带指纹的产物 {newest.product_path} 记的是 {newest.fingerprint[:12]}，今天的输入是 {fingerprint[:12]}"


# ── 门禁的前提与环境（登记表门禁行的第三列）──────────────────────────────────

def run_probe(root, parts):
    """bash <根>/<脚本> <参数…>，工作目录是根；返回 (退出码, stdout 字节, stderr 字节)。起不来、超时都算退 127。"""
    command = ["bash", os.path.join(root, parts["script"]), *parts["arguments"]]
    try:
        completed = subprocess.run(command, cwd=root, capture_output=True, timeout=PROBE_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return 127, b"", str(error).encode("utf-8", "replace")
    return completed.returncode, completed.stdout, completed.stderr


def probe_output_tail(output, errors):
    """探针输出的最后三行（stdout 与 stderr 接起来），给没齐的那一条当明细。"""
    lines = [line.strip() for line in (output + b"\n" + errors).decode("utf-8", "replace").split("\n") if line.strip()]
    return "｜".join(lines[-3:]) if lines else "一个字都没打"


def gate_conditions_of(rows, wanted_kinds):
    """这几行登记的门禁条件里种类在 wanted_kinds 之中的，[(条件原文, 种类, 各段)]；写错了抛 RegistrationError。"""
    found = []
    for row in rows:
        for token in row.conditions:
            kind, parts = parse_gate_condition(token)
            if kind in wanted_kinds:
                found.append((token, kind, parts))
    return found


def judge_gate_preconditions(root, rows):
    """门禁的前提逐条判：返回没齐的清单，每项一句说明。"""
    missing = []
    for token, kind, parts in gate_conditions_of(rows, GATE_PRECONDITION_KINDS):
        if kind == "command":
            if shutil.which(parts["value"]) is None:
                missing.append(f"{parts['value']} 不在 PATH 里（{token}）")
        elif kind == "readwrite":
            path = parts["value"] if os.path.isabs(parts["value"]) else os.path.join(root, parts["value"])
            if not os.access(path, os.R_OK | os.W_OK):
                missing.append(f"{parts['value']} 不在，或当前用户不可读写（{token}）")
        else:
            exit_code, output, errors = run_probe(root, parts)
            if exit_code != 0:
                missing.append(f"bash {' '.join([parts['script'], *parts['arguments']])} 退 {exit_code}（{token}）：{probe_output_tail(output, errors)}")
    return missing


def current_gate_environment(root, rows):
    """这一道登记的环境（environment=）逐条跑一遍：返回 ([(条件原文, 输出的第一行)], 整组的 sha256)；没登记返回 ([], None)。
    有一条退非 0 或一个字都没打，抛 InputManifestError。"""
    environment_lines = []
    digest = hashlib.sha256()
    for token, _kind, parts in gate_conditions_of(rows, ("environment",)):
        exit_code, output, errors = run_probe(root, parts)
        if exit_code != 0 or not output.strip():
            raise InputManifestError(f"{token} 退 {exit_code}：{probe_output_tail(output, errors)}")
        digest.update(token.encode("utf-8", "surrogateescape") + b"\n" + output + b"\0")
        environment_lines.append((token, output.decode("utf-8", "replace").strip().split("\n")[0]))
    if not environment_lines:
        return [], None
    return environment_lines, digest.hexdigest()


def environment_text(environment_lines):
    return "；".join(f"{token} → {first_line}" for token, first_line in environment_lines)


def gate_environment_record_path(root, stage):
    """git common-dir 里这一道的环境记录（各 worktree 共用一份，与门禁 54 号的全绿标记放在一处）；取不到 common-dir 返回 None。"""
    return_code, output = git_output(root, "rev-parse", "--path-format=absolute", "--git-common-dir")
    common_directory = output.strip() if return_code == 0 else ""
    if not common_directory:
        return None
    return os.path.join(common_directory, GATE_ENVIRONMENT_RECORD_PREFIX + stage)


def read_gate_environment_record(record_path):
    """记录里的 key=value 行与 environment_line 行；文件不在或读不了返回 None。"""
    try:
        with open(record_path, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    record = {"environment_lines": []}
    for line in text.split("\n"):
        if line.startswith("environment_line="):
            record["environment_lines"].append(line[len("environment_line="):])
        elif "=" in line and not line.startswith("#"):
            name, value = line.split("=", 1)
            record[name] = value
    return record


def judge_gate_environment(root, stage, stage_rows, staged_tree, input_paths, reason_so_far):
    """输入已判相同之后，登记了 environment= 的门禁再判环境：返回 (退出码, 一句依据)。没登记 environment= 的不进这里。"""
    if break_is_set("ignore-gate-environment"):
        return EXIT_GATE_MAY_SKIP, f"{reason_so_far}；（弄坏开关 ignore-gate-environment：没比环境）"
    try:
        environment_lines, environment_digest = current_gate_environment(root, stage_rows)
    except InputManifestError as error:
        return EXIT_GATE_MUST_RUN, f"判不出来：这一道登记的环境取不出来（{error}），按要跑处理"
    record_path = gate_environment_record_path(root, stage)
    record = read_gate_environment_record(record_path) if record_path else None
    if record is None:
        return EXIT_GATE_MUST_RUN, (f"输入与上次整轮全绿那棵树相同，但这一道上一次判绿时的环境没记下（{record_path or '取不到 git common-dir'}），"
                                    "判不出环境变没变，按要跑处理")
    if record.get("environment_sha256") != environment_digest:
        return EXIT_GATE_MUST_RUN, (f"输入与上次整轮全绿那棵树相同，但这一道的环境与上次判绿时不同：记下的是「{'；'.join(record['environment_lines'])}」，"
                                    f"现在是「{environment_text(environment_lines)}」")
    recorded_tree = record.get("tree", "")
    return_code, _output = git_output(root, "rev-parse", "-q", "--verify", f"{recorded_tree}^{{tree}}")
    if not recorded_tree or return_code != 0:
        return EXIT_GATE_MUST_RUN, f"判不出来：记下环境的那一趟判的树（{recorded_tree or '空'}）不是这个仓里的树对象，按要跑处理"
    return_code, _output = git_output(root, "diff", "--quiet", recorded_tree, staged_tree, "--", *input_paths)
    if return_code != 0:
        _return_code, changed_output = git_output(root, "-c", "core.quotepath=false", "diff", "--name-only", recorded_tree, staged_tree, "--", *input_paths)
        changed = "".join(f"{name} " for name in changed_output.split("\n")[:5] if name)
        return EXIT_GATE_MUST_RUN, (f"环境与上次判绿时相同，但记下环境的那一趟判的树（{recorded_tree[:12]}）与这一次在这几条路径上不同：{changed}"
                                    "（那一趟判绿之后整轮没绿，或绿的是另一批输入），按要跑处理")
    return EXIT_GATE_MAY_SKIP, (f"{reason_so_far}；环境与这一道上次判绿时（{record.get('recorded_utc', '时刻没记')}，"
                                f"树 {recorded_tree[:12]}）相同：{environment_text(environment_lines)}")


def record_gate_environment(root, stage):
    """阶段判绿之后记下环境：返回要打的一行（空串表示一个字不打）。记不下只返回「!」那一行，不抛。"""
    try:
        stage_rows = rows_of_key(read_registration_rows(root), stage)
        if not gate_conditions_of(stage_rows, ("environment",)):
            return ""
        staged_tree = os.environ.get("SINGLEFS_STAGED_TREE", "")
        if not staged_tree:
            return ""
        return_code, tree_output = git_output(root, "rev-parse", "-q", "--verify", f"{staged_tree}^{{tree}}")
        if return_code != 0 or not tree_output.strip():
            raise InputManifestError(f"SINGLEFS_STAGED_TREE={staged_tree} 不是这个仓里的树对象")
        judged_tree = tree_output.strip()
        environment_lines, environment_digest = current_gate_environment(root, stage_rows)
        record_path = gate_environment_record_path(root, stage)
        if record_path is None:
            raise InputManifestError("取不到 git common-dir")
        record_text = (f"# 门禁 {stage} 上一次判绿时的环境：research/scripts/admission.py gate-record-environment 写、gate-reuse 读。不进工作树，别手改。\n"
                       f"stage={stage}\ntree={judged_tree}\nenvironment_sha256={environment_digest}\n"
                       f"recorded_utc={time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}\n"
                       + "".join(f"environment_line={token} → {first_line}\n" for token, first_line in environment_lines))
        handle_number, temporary_path = tempfile.mkstemp(dir=os.path.dirname(record_path), prefix=os.path.basename(record_path) + ".partial.")
        try:
            with os.fdopen(handle_number, "w", encoding="utf-8") as handle:
                handle.write(record_text)
            os.replace(temporary_path, record_path)
        except OSError:
            os.unlink(temporary_path)
            raise
    except (RegistrationError, InputManifestError, OSError) as error:
        return f"  ! {stage} 判绿时的环境没记下（{error}）：下一趟整轮照跑这一道，不复用这一次的判定"
    return (f"  · {stage} 判绿时的环境记下了（{record_path}；树 {judged_tree[:12]}；{environment_text(environment_lines)}）："
            "下一趟输入与环境都没变就复用这一次的判定")


# ── 门禁的复用判定（原 stage-must-run.sh，判法与消息逐字相同）─────────────────

def git_output(root, *git_arguments):
    completed = subprocess.run(["git", "-C", root, *git_arguments], capture_output=True, text=True, errors="replace")
    return completed.returncode, completed.stdout


def gate_reuse(root, stage):
    """返回 (退出码, 一句依据)。退 EXIT_GATE_MUST_RUN 要跑，EXIT_GATE_MAY_SKIP 可跳过。"""
    if os.environ.get("SINGLEFS_GATE_FULL", "") == "1":
        return EXIT_GATE_MUST_RUN, "强制全跑（SINGLEFS_GATE_FULL=1）"
    return_code, _output = git_output(root, "rev-parse", "--is-inside-work-tree")
    if return_code != 0:
        return EXIT_GATE_MUST_RUN, f"判不出来：{root} 不是 git 工作树，按要跑处理"
    # 先看这一道在被判那棵树上的全绿标记（checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿时自己写，按登记输入的指纹分格，住 git common-dir）：
    # 在且没过复用上限就可跳过——崩溃验证员在工作区跑过的那一趟，整轮门禁不用再跑一遍。指纹算不出、没登记这一行的照旧往下按暂存树判。
    if not break_is_set("stage-marker-ignored"):
        try:
            fingerprint, file_count, _paths = stage_fingerprint(root, stage)
        except (RegistrationError, InputManifestError):
            fingerprint = None
        if fingerprint is not None:
            valid, lines = check_stage_marker(root, stage, fingerprint)
            if valid:
                return EXIT_GATE_MAY_SKIP, f"这一道这批输入（指纹 {fingerprint[:16]}…，{file_count} 个文件）有全绿标记：{lines[0]}"
    staged_tree = os.environ.get("SINGLEFS_STAGED_TREE", "")
    if not staged_tree:
        return EXIT_GATE_MUST_RUN, "这一趟不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE），被判的可能是工作区，不许复用"
    return_code, _output = git_output(root, "rev-parse", "-q", "--verify", f"{staged_tree}^{{tree}}")
    if return_code != 0:
        return EXIT_GATE_MUST_RUN, f"判不出来：SINGLEFS_STAGED_TREE={staged_tree} 不是这个仓里的树对象"
    return_code, green_output = git_output(root, "rev-parse", "-q", "--verify", f"{GATE_REUSE_REFERENCE}^{{commit}}")
    green = green_output.strip() if return_code == 0 else ""
    if not green:
        return EXIT_GATE_MUST_RUN, f"还没有过整轮全绿的暂存树（{GATE_REUSE_REFERENCE} 不存在）"
    if not os.path.isfile(os.path.join(root, REGISTRATION_TABLE)):
        return EXIT_GATE_MUST_RUN, f"读不到路径清单 {REGISTRATION_TABLE}，按要跑处理"
    stage_rows = rows_of_key(read_registration_rows(root), stage)
    if not any(row.input_paths for row in stage_rows):
        return EXIT_GATE_MUST_RUN, f"{REGISTRATION_TABLE} 里没有 {stage} 这一行，按要跑处理"
    # 清单里这道阶段的每一行都进比对；清单自己进比对（少写一条输入，那条输入就永远不会让这道阶段重跑）；
    # 阶段脚本自己进比对（判据本身变了，上一次的判定就不再作数）。
    input_paths = [input_path for row in stage_rows for input_path in row.input_paths]
    input_paths += [REGISTRATION_TABLE, f".claude/gate.d/{stage}"]
    hours_text = os.environ.get("SINGLEFS_REUSE_HOURS", "") or "24"
    try:
        reuse_limit_hours = int(hours_text)
    except ValueError:
        reuse_limit_hours = 0
    # 时刻读那个提交对象自己的：ref 指的是「包着那棵暂存树的提交」而不是裸树，git update-ref 对 refs/heads/ 之外的 ref 默认不写 reflog。
    return_code, epoch_output = git_output(root, "log", "-1", "--format=%ct", green)
    green_epoch_text = epoch_output.strip() if return_code == 0 else ""
    if not re.fullmatch(r"[0-9]+", green_epoch_text):
        return EXIT_GATE_MUST_RUN, f"取不到 {GATE_REUSE_REFERENCE} 的时刻（它不是包着暂存树的提交？），判不出复用上限，按要跑处理"
    elapsed_seconds = int(time.time()) - int(green_epoch_text)
    age_hours = elapsed_seconds // 3600 if elapsed_seconds >= 0 else -((-elapsed_seconds) // 3600)
    if age_hours >= reuse_limit_hours:
        return EXIT_GATE_MUST_RUN, (f"上次整轮全绿在 {age_hours} 小时前，到了复用上限 {hours_text} 小时：内容没变不代表环境没变"
                                    "（工具链升级这一格比对看不见），强制跑一趟")
    return_code, _output = git_output(root, "diff", "--quiet", green, staged_tree, "--", *input_paths)
    if return_code == 0:
        reason = f"与上次整轮全绿那棵树（{green[:12]}）在这几条路径上逐字相同：{' '.join(input_paths)}"
        try:
            has_environment = bool(gate_conditions_of(stage_rows, ("environment",)))
        except RegistrationError as error:
            return EXIT_GATE_MUST_RUN, f"判不出来：{REGISTRATION_TABLE} 里 {stage} 那一行的第三列写错了（{error}），按要跑处理"
        if not has_environment:
            return EXIT_GATE_MAY_SKIP, reason
        return judge_gate_environment(root, stage, stage_rows, staged_tree, input_paths, reason)
    # 路径按原样打（core.quotepath=false）：不带它时中文路径被转成带引号的八进制串，依据句里认不出是哪个文件
    _return_code, changed_output = git_output(root, "-c", "core.quotepath=false", "diff", "--name-only", green, staged_tree, "--", *input_paths)
    changed = "".join(f"{name} " for name in changed_output.split("\n")[:5] if name)
    return EXIT_GATE_MUST_RUN, f"这几条路径与上次整轮全绿那棵树不同：{changed}"


# ── 崩溃枚举用例（门禁 54 号：逐条按复用判定跑、逐条记全绿标记）────────────────
#
# 一条用例的输入 = 登记路径（整个 crates/ 与 Cargo 清单、锁）下 git 列得出的文件，减去用例读不到的文件，再加准入模块（这一份）里
# 崩溃枚举用例的判法摘要、工具链、构建环境与这条用例的登记行（54 号按 --judging-digest、--toolchain、--build-environment 给；
# 54 号自己不进：起用例的命令与环境由这一份的 crash-case-command 交出，在判法摘要里）；登记了 shard=across-machines 的另加双机分片的驱动脚本与
# 配置判法（SHARD_DRIVER_FILES，按内容：分片开着时起两片与 merge 的是驱动脚本，不经 crash-case-command）。用例读不到的文件有三种：
#   ① 别的测试目标 X 独占的文件：它所在的包没有构建脚本（build.rs 或 package.build）、没写 [[test]] 与 package.autotests，包里没有
#      参数是算出来的 include! / include_str! / include_bytes!（COMPILE_TIME_COMPUTED_INCLUDE：套 concat!、env!、option_env! 或别的宏，
#      宏名前带不带 ::core:: / std:: 都算），任何一个包的构建脚本（注释去掉之后）都不出现整词 tests；
#      X 是 tests/X.rs（或 tests/X/main.rs 那种目录目标、连同那个目录），而 X 的名字不以整词出现在别处任何 .rs（注释去掉之后）与
#      Cargo.toml 里——`mod X;`、`#[path = "…X.rs"]`、`include_str!("…X.rs")`、运行期按字面路径读它，都让它留在输入里；
#   ② CRASH_CASE_FILES_NOT_READ 里的（crates/mutations.tsv），没有代码与 Cargo.toml 按文件名点名它；
#   ③ 没有任何代码读 CARGO_BIN_EXE_ 时，各包 src/bin/ 下的文件（包里有构建脚本、[[test]]、autotests 的不减；
#      src/bin/ 之外的代码按 `bin/<它在 src/bin/ 下的路径>` 点名它的不减）。
#   ② ③ 两类在任何一份 .rs 里有参数是算出来的 include 时一份都不减（拼出来的可能就是它们）。
# 登记只写整个 crates/，不按用例手列它读哪些文件：新加的共用文件（tests/common.rs、build.rs、新拆出的 crate）默认就在输入里。

class CrashCase:
    def __init__(self, row, package, target, function, count_lines, exhaustive_lines, thread_lines, is_shardable_across_machines,
                 thread_variable=None):
        self.row = row
        # 用例读线程数的环境变量（登记行的 threads-variable=，没登记是 SINGLEFS_LAYER0_THREADS）：crash-case-command 设它、标记里记它
        self.thread_variable = thread_variable or LAYER0_THREADS_VARIABLE
        self.is_shardable_across_machines = is_shardable_across_machines
        self.key = row.key
        self.name = row.key[len(CRASH_CASE_KEY_PREFIX):]
        self.package = package
        self.target = target
        self.function = function
        self.count_lines = count_lines
        self.exhaustive_lines = exhaustive_lines
        self.thread_lines = thread_lines

    def test_text(self):
        return f"{self.package}:{self.target}:{self.function}"


def parse_crash_case(row):
    """崩溃枚举用例行的第三列：写法认不出、test= 不是恰好一条、exhaustive= / threads= 点名的前缀没登记成 count-line=、
    shard= 不是 across-machines 或多于一条、threads-variable= 不是环境变量名或多于一条，都抛 RegistrationError。"""
    tests, count_lines, exhaustive_lines, thread_lines, shard_values, thread_variables = [], [], [], [], [], []
    for token in row.conditions:
        match = CRASH_CASE_CONDITION_FORM.match(token)
        if not match:
            raise RegistrationError(f"崩溃枚举用例 {row.key} 的第三列认不出 {token}（只认 test=<包>:<测试目标>:<用例函数>、"
                                    f"count-line=<前缀>、exhaustive=<前缀>、threads=<前缀>、threads-variable=<环境变量名>、"
                                    f"shard={CRASH_CASE_SHARD_ACROSS_MACHINES}）")
        kind, value = match.group("kind"), match.group("value")
        if kind == "test":
            test = CRASH_CASE_TEST_FORM.match(value)
            if not test:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token} 要写成 test=<包>:<测试目标>:<用例函数>（字母、数字、_，包名另许 -）")
            tests.append(test)
            continue
        if kind == "shard":
            if value != CRASH_CASE_SHARD_ACROSS_MACHINES:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：shard= 只认 {CRASH_CASE_SHARD_ACROSS_MACHINES}")
            shard_values.append(value)
            continue
        if kind == "threads-variable":
            if not THREADS_VARIABLE_FORM.match(value):
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：threads-variable= 要写用例读线程数的环境变量名（大写字母、数字、_）")
            thread_variables.append(value)
            continue
        if not COUNT_LINE_PREFIX_FORM.match(value):
            raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：前缀只许大写字母、数字、_，以字母开头")
        {"count-line": count_lines, "exhaustive": exhaustive_lines, "threads": thread_lines}[kind].append(value)
    if len(tests) != 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 要恰好一条 test=，实际 {len(tests)} 条")
    if len(shard_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 shard= 至多一条，实际 {len(shard_values)} 条")
    if len(thread_variables) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 threads-variable= 至多一条，实际 {len(thread_variables)} 条")
    if len(set(count_lines)) != len(count_lines):
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 count-line= 有重复的前缀")
    unregistered = [prefix for prefix in exhaustive_lines + thread_lines if prefix not in count_lines]
    if unregistered:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 exhaustive= / threads= 点名的 {' '.join(unregistered)} 没有登记成 count-line=")
    test = tests[0]
    thread_variable = None if break_is_set("thread-variable-ignored") else (thread_variables[0] if thread_variables else None)
    return CrashCase(row, test.group("package"), test.group("target"), test.group("function"),
                     count_lines, exhaustive_lines, thread_lines, bool(shard_values), thread_variable)


def crash_cases_of(rows):
    """登记表里全部崩溃枚举用例，照登记表的次序；一行写错抛 RegistrationError。"""
    return [parse_crash_case(row) for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]


def crash_case_of_key(rows, key):
    own_rows = rows_of_key(rows, key)
    if not CRASH_CASE_KEY_FORM.match(key) or len(own_rows) != 1:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里崩溃枚举用例 {key} 要恰好一行，实际 {len(own_rows)} 行")
    return parse_crash_case(own_rows[0])


def manifest_sections(path):
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except (OSError, ValueError):  # TOMLDecodeError 与编码错都是 ValueError
        return None


def workspace_package_directories(root):
    """仓根 Cargo.toml 的工作区成员（members 按 glob 展开；仓根自己是包也算）：包名 → 仓根起的目录。"""
    root_sections = manifest_sections(os.path.join(root, "Cargo.toml")) or {}
    directories = []
    if "package" in root_sections:
        directories.append(".")
    for pattern in (root_sections.get("workspace") or {}).get("members") or []:
        for directory in sorted(glob.glob(os.path.join(root, pattern))):
            directories.append(os.path.relpath(directory, root))
    packages = {}
    for directory in directories:
        name = ((manifest_sections(os.path.join(root, directory, "Cargo.toml")) or {}).get("package") or {}).get("name")
        if name:
            packages[name] = os.path.normpath(directory)
    return packages


def test_target_source_files(root, package_directory, target):
    """包目录（相对 root）下测试目标 target 的源文件（相对 root）：tests/<目标>.rs，或 tests/<目标>/main.rs 那种目录目标里的全部 .rs；
    目标不在交空表。"""
    single_file = os.path.join(package_directory, "tests", target + ".rs")
    directory_target = os.path.join(package_directory, "tests", target)
    if os.path.isfile(os.path.join(root, single_file)):
        return [single_file]
    if os.path.isfile(os.path.join(root, directory_target, "main.rs")):
        return sorted(os.path.relpath(path, root)
                      for path in glob.glob(os.path.join(root, directory_target, "**", "*.rs"), recursive=True))
    return []


# 测试目标顺着带进来的源文件（files_brought_in_by_target）：include!（宏名前带不带 ::core:: / std:: 都算）、mod <名>; 与 #[path = "…"] mod <名>;
INCLUDE_CODE_MACRO = re.compile(r"\binclude\s*!\s*[(\[{]\s*")
MODULE_DECLARATION = re.compile(r"\bmod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*;")
PATH_ATTRIBUTE = re.compile(r'^#\s*\[\s*path\s*=\s*(b?r?#*".*")\s*\]$', re.S)
# 用 use 把 include 族宏（include!、include_str!、include_bytes!）引进来（改不改名都算）：之后怎么写调用认不出
USE_OF_INCLUDE_MACRO = re.compile(r"\buse\b[^;]*\binclude(?:_str|_bytes)?\b[^;]*;")
# 带进来的源文件最多顺几份（防成环的 include!、mod 互相带）
TARGET_FILES_FOLLOWED_AT_MOST = 500


def string_literal_value(literal):
    """Rust 字符串字面量（"…" 或 r#"…"#）的值；带转义的（反斜杠）交 None（路径里用不到，判不出就按判不出算）。"""
    raw = re.match(r'^b?r(#*)"(.*)"\1$', literal, re.S)
    if raw:
        return raw.group(2)
    plain = re.match(r'^b?"(.*)"$', literal, re.S)
    if plain and "\\" not in plain.group(1):
        return plain.group(1)
    return None


def files_brought_in_by_target(root, relative_files):
    """测试目标的源文件（relative_files，相对 root）顺着 include!("<字面路径>")、#[path = "<字面路径>"] mod <名>; 与 mod <名>;（<名>.rs 或 <名>/mod.rs）
    带进来的全部源文件（相对 root，按找到的次序，头几份是 relative_files 本身）：返回 (文件清单, 判不出的原因或 None)。
    判不出：include! 的参数不是字符串字面量（套 concat!、env! 这一类）、带进来的文件找不到或读不了、有代码用 use 把 include 族宏引进来、带进来的超过上限。
    路径的起点照 rustc：include! 与 #[path] 从写它的那份文件所在的目录起；mod <名>; 在测试目标的根文件与 mod.rs 里从所在目录起，别的文件从「所在目录/文件名」起。
    弄坏开关 target-own-files-only 下只交 relative_files 本身（改前的判法）。"""
    if break_is_set("target-own-files-only"):
        return list(relative_files), None
    roots = set(relative_files)
    found, queue = [], list(relative_files)
    while queue:
        relative = os.path.normpath(queue.pop(0))
        if relative in found:
            continue
        if len(found) >= TARGET_FILES_FOLLOWED_AT_MOST:
            return found, f"带进来的源文件超过 {TARGET_FILES_FOLLOWED_AT_MOST} 份"
        try:
            with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
                code = rust_code_without_comments(handle.read())
        except OSError as error:
            return found, f"读不了 {relative}：{error}"
        found.append(relative)
        directory = os.path.dirname(relative)
        if USE_OF_INCLUDE_MACRO.search(code):
            return found, f"{relative} 里用 use 把 include 族宏引进来了（之后的调用认不出读的是哪一份）"
        for call in INCLUDE_CODE_MACRO.finditer(code):
            literal_end = end_of_literal(code, call.end())
            value = string_literal_value(code[call.end():literal_end]) if literal_end is not None and code[call.end()] in 'br"' else None
            if value is None or not re.match(r"\s*[)\]}]", code[literal_end:]):
                return found, f"{relative} 里 include! 的参数不是一个字符串字面量（{code[call.start():call.end() + 40].strip()}…）：读的是哪一份判不出"
            queue.append(os.path.relpath(value, root) if os.path.isabs(value) else os.path.join(directory, value))
        attribute_end_to_start = {end: start for start, end in attribute_spans(code)}
        module_base = directory if (relative in roots or os.path.basename(relative) in ("mod.rs", "main.rs", "lib.rs")) \
            else os.path.join(directory, os.path.splitext(os.path.basename(relative))[0])
        for declaration in MODULE_DECLARATION.finditer(code):
            head = code[:declaration.start()].rstrip()
            qualifier = re.search(r"\bpub(?:\s*\([^()]*\))?$", head)
            if qualifier:
                head = head[:qualifier.start()].rstrip()
            path_value, cursor = None, len(head)
            while cursor in attribute_end_to_start:
                attribute = head[attribute_end_to_start[cursor]:cursor]
                path_attribute = PATH_ATTRIBUTE.match(attribute)
                if path_attribute:
                    path_value = string_literal_value(path_attribute.group(1))
                    if path_value is None:
                        return found, f"{relative} 里 {attribute} 的路径判不出"
                cursor = len(head[:attribute_end_to_start[cursor]].rstrip())
            if path_value is not None:
                queue.append(os.path.join(directory, path_value))
                continue
            name = declaration.group(1)
            candidates = [os.path.join(module_base, name + ".rs"), os.path.join(module_base, name, "mod.rs")]
            existing = [candidate for candidate in candidates if os.path.isfile(os.path.join(root, candidate))]
            if not existing:
                return found, f"{relative} 里 mod {name}; 找不到 {' 或 '.join(candidates)}"
            queue += existing
    return found, None


def crash_case_test_files(root, case):
    """这条用例的测试目标的源文件（仓根起）：tests/<目标>.rs，或 tests/<目标>/main.rs 那种目录目标里的全部 .rs，连同它们顺着
    include! / #[path] / mod 带进来的（files_brought_in_by_target）。
    包不是工作区成员、目标不在、带进来的判不出、用例函数在目标里找不到、用例函数没标 #[ignore]，都抛 RegistrationError。"""
    package_directory = workspace_package_directories(root).get(case.package)
    if package_directory is None:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
    files = test_target_source_files(root, package_directory, case.target)
    if not files:
        single_file = os.path.join(package_directory, "tests", case.target + ".rs")
        directory_target = os.path.join(package_directory, "tests", case.target)
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 不在（没有 {single_file}，也没有 {directory_target}/main.rs）")
    files, undecided = files_brought_in_by_target(root, files)
    if undecided:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的测试目标 {case.target} 带进来的源文件判不出（{undecided}）：用例函数标没标 #[ignore] "
                                "判不出，按没标算；include! / #[path] 写成字符串字面量、不用 use 改名 include 族宏")
    marks = definitions_marked_ignored(root, files, case.function)
    if not marks:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 在 {'、'.join(files)} 里找不到字面的 fn {case.function}(："
                                "改了名？宏生成的用例判不出标没标 #[ignore]，要写成字面的函数")
    unmarked = [relative for relative, marked in marks if not marked]
    if unmarked:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的用例函数 {case.function} 有 {len(unmarked)} 处定义没标 #[ignore]（{'、'.join(unmarked)}；"
                                f"共 {len(marks)} 处，cfg 二选一的、子模块里同名的都算）：不带 --ignored 的 cargo test 也会跑到全量；"
                                "给每一处补上 #[ignore = \"<为什么平时不跑>\"]")
    return files


def rust_code_without_comments(text):
    """去掉 Rust 源码里的注释（行注释、可嵌套的块注释），字符串、原始字符串、字符字面量原样留着：判「别处有没有代码点名一个测试目标」时，
    注释里提到它不算，字符串里写它的路径（include_str!、#[path]、运行期按路径读）算。"""
    pieces = []
    position = 0
    length = len(text)
    while position < length:
        if text.startswith("//", position):
            line_end = text.find("\n", position)
            position = length if line_end < 0 else line_end
            continue
        if text.startswith("/*", position):
            depth = 0
            while position < length:
                if text.startswith("/*", position):
                    depth += 1
                    position += 2
                elif text.startswith("*/", position):
                    depth -= 1
                    position += 2
                    if depth == 0:
                        break
                else:
                    position += 1
            pieces.append(" ")
            continue
        end = end_of_literal(text, position)
        if end is not None:
            pieces.append(text[position:end])
            position = end
            continue
        pieces.append(text[position])
        position += 1
    return "".join(pieces)


RAW_STRING_START = re.compile(r'b?r(?P<hashes>#*)"')


def end_of_literal(code, position):
    """code[position] 起是字符串、原始字符串或字符字面量时交回它之后那个下标；不是时交 None（生命周期不算字面量）。"""
    character = code[position]
    if character in "br":
        raw_string = RAW_STRING_START.match(code, position)
        previous_is_identifier = position > 0 and (code[position - 1].isalnum() or code[position - 1] == "_")
        if raw_string and not previous_is_identifier:
            closing = '"' + raw_string.group("hashes")
            end = code.find(closing, raw_string.end())
            return len(code) if end < 0 else end + len(closing)
    if character == '"':
        end = position + 1
        while end < len(code) and code[end] != '"':
            end += 2 if code[end] == "\\" else 1
        return min(end + 1, len(code))
    if character == "'":
        if code.startswith("\\", position + 1):
            closing = code.find("'", position + 3)
            return len(code) if closing < 0 else closing + 1
        if position + 2 < len(code) and code[position + 2] == "'":
            return position + 3
    return None


def attribute_spans(code):
    """去掉注释之后的代码里每一条属性（`#[…]`、`#![…]`）的 (起, 止)：方括号按层数配对，字符串、原始字符串与字符字面量里的括号不算。"""
    spans = []
    position = 0
    while position < len(code):
        end = end_of_literal(code, position)
        if end is not None:
            position = end
            continue
        attribute = ATTRIBUTE_START.match(code, position)
        if not attribute:
            position += 1
            continue
        depth, cursor = 0, attribute.end() - 1
        while cursor < len(code):
            end = end_of_literal(code, cursor)
            if end is not None:
                cursor = end
                continue
            if code[cursor] == "[":
                depth += 1
            elif code[cursor] == "]":
                depth -= 1
                if depth == 0:
                    break
            cursor += 1
        spans.append((position, min(cursor + 1, len(code))))
        position = cursor + 1
    return spans


def attributes_before_each_definition(code, function):
    """去掉注释之后的代码里每一处 `fn <function>(`（cfg 二选一的、子模块里同名的都算，按出现的次序）前面紧挨着的那串属性：
    [[属性原文，一条一个，按出现的次序], …]；一处都没有交空表。属性与 fn 之间可以夹 pub、pub(…)、async、unsafe、const、extern "…"。
    弄坏开关 first-definition-only 下只交第一处。"""
    attribute_lists = []
    for found in re.finditer(r"\bfn\s+" + re.escape(function) + r"\s*\(", code):
        head = code[:found.start()].rstrip()
        while True:
            qualifier = FUNCTION_QUALIFIER_AT_END.search(head)
            if not qualifier:
                break
            head = head[:qualifier.start()].rstrip()
        start_of_attribute_ending_at = {end: start for start, end in attribute_spans(head)}
        attributes = []
        cursor = len(head)
        while cursor in start_of_attribute_ending_at:
            start = start_of_attribute_ending_at[cursor]
            attributes.append(head[start:cursor])
            cursor = len(head[:start].rstrip())
        attribute_lists.append(list(reversed(attributes)))
        if break_is_set("first-definition-only"):
            break
    return attribute_lists


def attributes_mark_ignored(attributes):
    """那串属性里有没有 #[ignore] 或 #[ignore = "…"]（# 与 [ 之间许空白）。"""
    form = IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE if break_is_set("ignore-attribute-without-space") else IGNORE_ATTRIBUTE_FORM
    return any(form.match(attribute) for attribute in attributes)


def definitions_marked_ignored(root, relative_files, function):
    """测试目标的源文件（relative_files，相对 root）里每一处 `fn <function>(` 标没标 #[ignore]：[(文件, 标了?)]，按文件与出现的次序；
    一处都没有交空表，读不了抛 OSError。弄坏开关 first-definition-only 下只看头一份有定义的文件里的第一处。"""
    marks = []
    for relative in relative_files:
        with open(os.path.join(root, relative), encoding="utf-8", errors="replace") as handle:
            attribute_lists = attributes_before_each_definition(rust_code_without_comments(handle.read()), function)
        marks += [(relative, attributes_mark_ignored(attributes)) for attributes in attribute_lists]
        if marks and break_is_set("first-definition-only"):
            break
    return marks


def test_function_is_marked_ignored(root, package_directory, target, function):
    """root 底下包目录 package_directory（相对 root）里测试目标 target 的用例函数 function 标没标 #[ignore]：同名的每一处定义都标了 True，
    有一处没标 False，目标不在、读不了、一处定义都找不到（宏生成的用例这一类）、顺着 include! / #[path] / mod 带进来的判不出交 None。
    .claude/hooks/lib_heavy_tests.py（重型测试闸）按文件路径导入本模块调它，与 crash-cases 自查同一套判法；闸把 None 按没标算。"""
    files, undecided = files_brought_in_by_target(root, test_target_source_files(root, package_directory, target))
    if undecided:
        return None
    try:
        marks = definitions_marked_ignored(root, files, function)
    except OSError:
        return None
    if not marks:
        return None
    return all(marked for _relative, marked in marks)


def whole_word_form(word):
    return re.compile(r"(?<![A-Za-z0-9_])" + re.escape(word) + r"(?![A-Za-z0-9_])")


def test_targets_of_package(package_directory, listed_names):
    """包目录下 tests/ 里 cargo 自动认的集成测试目标：目标名 → 它的文件（tests/<名>.rs；tests/<名>/main.rs 那种目标是 tests/<名>/ 下全部文件）。
    package_directory 与 listed_names 都是仓根起的 str 路径。"""
    tests_prefix = package_prefix(package_directory) + "tests/"
    listed_set = set(listed_names)
    targets = {}
    for name in listed_names:
        if not name.startswith(tests_prefix):
            continue
        rest = name[len(tests_prefix):]
        parts = rest.split("/")
        if len(parts) == 1 and rest.endswith(".rs"):
            targets.setdefault(rest[:-3], []).append(name)
        elif len(parts) >= 2 and tests_prefix + parts[0] + "/main.rs" in listed_set:
            targets.setdefault(parts[0], []).append(name)
    return targets


def package_prefix(package_directory):
    return "" if package_directory == "." else package_directory + "/"


def package_directories_among(root, listed_names):
    """listed_names（仓根起）里的包目录：有 [package] 的 Cargo.toml 所在的目录，仓根是 "."。"""
    return sorted({os.path.dirname(name) or "." for name in listed_names
                   if os.path.basename(name) == "Cargo.toml" and "package" in (manifest_sections(os.path.join(root, name)) or {})})


def package_keeps_every_test_file(root, package_directory, listed_set):
    """包里有 build.rs、写了 [[test]]、package.autotests 或 package.build：测试目标怎么编、读什么由它们另定，这一个包的测试文件一份都不减。"""
    sections = manifest_sections(os.path.join(root, package_directory, "Cargo.toml")) or {}
    package = sections.get("package") or {}
    return (package_prefix(package_directory) + "build.rs" in listed_set or "test" in sections
            or "autotests" in package or "build" in package)


def build_script_names(root, package_directories, listed_set):
    """各包的构建脚本（仓根起、列在 listed_set 里的）：build.rs，或 package.build 指的那一份（package.build = false 的没有）。"""
    names = []
    for package_directory in package_directories:
        build = ((manifest_sections(os.path.join(root, package_directory, "Cargo.toml")) or {}).get("package") or {}).get("build")
        relative = build if isinstance(build, str) else (None if build is False else "build.rs")
        name = os.path.normpath(package_prefix(package_directory) + relative) if relative else None
        if name in listed_set:
            names.append(name)
    return names


def listed_code_texts(root, listed_names):
    """listed_names（仓根起）里的 .rs（注释去掉之后）与 Cargo.toml（原文）：路径 → 文字。"""
    code_texts = {}
    for name in listed_names:
        if name.endswith(".rs") or os.path.basename(name) == "Cargo.toml":
            try:
                with open(os.path.join(root, name), encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
            except OSError as error:
                raise InputManifestError(f"读不了 {name}：{error}") from error
            code_texts[name] = rust_code_without_comments(text) if name.endswith(".rs") else text
    return code_texts


def files_exclusive_to_other_test_targets(root, listed_names, code_texts, own_package_directory, own_target):
    """listed_names（仓根起的 str）里别的测试目标独占的文件（str 的集合）：判法见本节开头的 ①。"""
    listed_set = set(listed_names)
    package_directories = package_directories_among(root, listed_names)
    tests_word = whole_word_form("tests")
    if any(tests_word.search(code_texts.get(name, "")) for name in build_script_names(root, package_directories, listed_set)):
        return set()
    computing = files_with_computed_include(code_texts)
    candidates = {}
    for package_directory in package_directories:
        if package_keeps_every_test_file(root, package_directory, listed_set):
            continue
        if any(name.startswith(package_prefix(package_directory)) for name in computing):
            continue
        for target, files in test_targets_of_package(package_directory, listed_names).items():
            if (os.path.normpath(package_directory), target) != (os.path.normpath(own_package_directory), own_target):
                candidates[(package_directory, target)] = files
    if files_importing_include_macro(code_texts):
        return set()
    exclusive = set()
    for (_package_directory, target), files in candidates.items():
        own_files = set(files)
        named_elsewhere = any(whole_word_form(target).search(text) for name, text in code_texts.items() if name not in own_files)
        if named_elsewhere and not break_is_set("exclude-mentioned-test-targets"):
            continue
        exclusive.update(files)
    return exclusive


def files_importing_include_macro(code_texts):
    """code_texts 里用 use 把 include 族宏（include!、include_str!、include_bytes!）引进来的 .rs（改不改名都算）：路径的清单。
    有一份就按宽处理，哪个包、哪一类文件都不减（改名之后的调用认不出，读的是哪一份判不出）。弄坏开关 include-alias-subtracts 下交空表。"""
    if break_is_set("include-alias-subtracts"):
        return []
    return [name for name, text in code_texts.items() if name.endswith(".rs") and USE_OF_INCLUDE_MACRO.search(text)]


def files_with_computed_include(code_texts):
    """code_texts 里有 include! / include_str! / include_bytes! 的参数是算出来的（不以字符串字面量开头）的 .rs：路径的清单。"""
    form = COMPILE_TIME_CONCATENATED_INCLUDE_ONLY if break_is_set("concat-include-only") else COMPILE_TIME_COMPUTED_INCLUDE
    return [name for name, text in code_texts.items() if name.endswith(".rs") and form.search(text)]


def files_crash_cases_do_not_read(root, listed_names, code_texts):
    """listed_names（仓根起的 str）里不是测试文件、用例也读不到的（str 的集合）：判法见本节开头的 ② ③。
    任何一份 .rs 里有算出来的 include（files_with_computed_include）时一份都不减：它拼出来的可能就是 mutations.tsv 或 src/bin/ 下的文件。"""
    listed_set = set(listed_names)
    if files_with_computed_include(code_texts) and not break_is_set("computed-include-subtracts-non-test-files"):
        return set()
    if files_importing_include_macro(code_texts):
        return set()
    left_out = {name for name in CRASH_CASE_FILES_NOT_READ
                if name in listed_set and not any(os.path.basename(name) in text for text in code_texts.values())}
    if any(BIN_EXECUTABLE_VARIABLE_PREFIX in text for text in code_texts.values()):
        return left_out
    for package_directory in package_directories_among(root, listed_names):
        if package_keeps_every_test_file(root, package_directory, listed_set):
            continue
        bin_prefix = package_prefix(package_directory) + "src/bin/"
        for name in listed_names:
            if not name.startswith(bin_prefix):
                continue
            mention = "bin/" + name[len(bin_prefix):]
            if not any(mention in text for other, text in code_texts.items() if not other.startswith(bin_prefix)):
                left_out.add(name)
    return left_out


def crash_case_manifest(root, case, named_lines):
    """这条用例的输入清单：返回 (清单字节, 指纹, 文件数, 减去的文件 [bytes])。末尾在 named_lines 之后加这条用例的登记行（连第三列）。"""
    listed_files = listed_input_files(root, case.row.input_paths)
    if not listed_files:
        raise InputManifestError(f"{case.key} 登记的路径（{' '.join(case.row.input_paths)}）下 git 一个文件都列不出来")
    package_directory = workspace_package_directories(root).get(case.package)
    if package_directory is None:
        raise RegistrationError(f"崩溃枚举用例 {case.key} 的包 {case.package} 不是仓根 Cargo.toml 的工作区成员")
    listed_names = [os.fsdecode(name) for name in listed_files]
    code_texts = listed_code_texts(root, listed_names)
    excluded_names = (files_exclusive_to_other_test_targets(root, listed_names, code_texts, package_directory, case.target)
                      | files_crash_cases_do_not_read(root, listed_names, code_texts))
    excluded = {os.fsencode(name) for name in excluded_names}
    kept = [name for name in listed_files if name not in excluded]
    registration = (f"<登记行：{case.key}>", case.row.text_with_conditions().encode("utf-8", "surrogateescape"))
    manifest, fingerprint, file_count = manifest_of_files(root, kept, list(named_lines) + shard_driver_lines(root, case) + [registration])
    return manifest, fingerprint, file_count, sorted(excluded)


def shard_driver_lines(root, case):
    """登记了 shard=across-machines 的用例：SHARD_DRIVER_FILES 每一份一行 (名字, 内容)，排在登记行之前；没登记的交空表。
    不在的那一份内容记「找不到 <路径>」（驱动脚本不在时 54 号照单机跑，指纹照样算得出）。弄坏开关 shard-driver-outside-manifest 下交空表。"""
    if not case.is_shardable_across_machines or break_is_set("shard-driver-outside-manifest"):
        return []
    lines = []
    for relative_path in SHARD_DRIVER_FILES:
        try:
            with open(os.path.join(root, relative_path), "rb") as handle:
                content = handle.read()
        except FileNotFoundError:
            content = f"找不到 {relative_path}".encode("utf-8")
        lines.append((f"<双机分片：{relative_path}>", content))
    return lines


def git_common_directory(root):
    return_code, output = git_output(root, "rev-parse", "--path-format=absolute", "--git-common-dir")
    common_directory = output.strip() if return_code == 0 else ""
    return common_directory or None


def crash_case_marker_path(root, case, fingerprint):
    """那一格全绿标记：git common-dir 里「前缀 + 用例名 + . + 指纹」，不进工作树，各 worktree 共用一份；取不到 common-dir 返回 None。"""
    common_directory = git_common_directory(root)
    if common_directory is None:
        return None
    return os.path.join(common_directory, f"{CRASH_CASE_MARKER_PREFIX}{case.name}.{fingerprint}")


def fields_of_line(line):
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


def judge_worker_threads(prefix, count_line, log_lines, machine_cores, threads_explicitly_one):
    """按计数行的 states= 找那一行 LAYER0_PARALLEL_FINISHED，判工作线程：返回 (原因或 None, 一句说明)。
    计数行自己带 worker_threads= 与 slices= 的（跑完那一行就是计数行：crash-injection 快档的 CRASH_INJECTION_FINISHED 这一类）拿它自己判，
    它不续跑、不带 resumed_slices= / freshly_run_slices= 的按读回 0 片、这一趟跑了全部片算（弄坏开关 threads-skip-self-contained-finish-line 下
    这一类不判线程）。
    判红只在这一趟真跑了至少两片、却只起了 1 个工作线程、本机多于 1 核、线程数没显式设成 1 时：全部片从进度文件读回时起 0 个线程，
    只剩 1 片要跑时最多起 1 个，这两种都不是「线程数没传进去」。"""
    count_fields = fields_of_line(count_line)
    if "worker_threads" in count_fields and "slices" in count_fields:
        if break_is_set("threads-skip-self-contained-finish-line"):
            return None, f"{prefix}：跑完那一行自己带线程数，没判（弄坏开关 threads-skip-self-contained-finish-line）"
        finished_line, fields = count_line, dict(count_fields)
        if "resumed_slices" not in fields and "freshly_run_slices" not in fields:
            fields.update(resumed_slices="0", freshly_run_slices=fields["slices"])
    else:
        states = count_fields.get("states")
        finished = [line for line in log_lines if line.startswith(f"{LAYER0_PARALLEL_FINISHED_PREFIX}states={states} ")] if states else []
        if not finished:
            return (f"{prefix} 那一行（states={states or '读不到'}）找不到状态数对得上的 LAYER0_PARALLEL_FINISHED 行，判不出起了几个工作线程"
                    "（全量要经 crates/singlefs-checker-tier/src/crash.rs 的 enumerate_layer0_in_state_slices 跑）"), ""
        finished_line, fields = finished[0], fields_of_line(finished[0])
    try:
        worker_threads = int(fields["worker_threads"])
        slices = int(fields["slices"])
        resumed_slices = int(fields["resumed_slices"])
        freshly_run_slices = int(fields["freshly_run_slices"])
    except (KeyError, ValueError):
        return (f"跑完那一行缺 worker_threads= / slices= / resumed_slices= / freshly_run_slices=，或不是整数："
                f"{finished_line}"), ""
    if resumed_slices + freshly_run_slices != slices:
        return (f"跑完那一行读回的片 {resumed_slices} + 这一趟跑的片 {freshly_run_slices} ≠ 总片数 {slices}："
                f"{finished_line}"), ""
    if freshly_run_slices > 0 and worker_threads == 0:
        return f"这一趟跑了 {freshly_run_slices} 片，却报起了 0 个工作线程：{finished_line}", ""
    if "shards" in fields and not break_is_set("threads-ignore-shards"):
        return judge_threads_of_each_shard(prefix, finished_line, fields)
    if break_is_set("threads-by-worker-count"):
        judged_on_one_thread = worker_threads == 1
    else:
        judged_on_one_thread = freshly_run_slices >= 2 and worker_threads == 1
    if judged_on_one_thread and machine_cores > 1 and not threads_explicitly_one:
        return (f"本机 {machine_cores} 核、线程数没显式设成 1，这一趟跑了 {freshly_run_slices} 片却只起了 1 个工作线程"
                f"（多半是线程数没传进去）：{finished_line}"), ""
    if freshly_run_slices == 0:
        note = f"{prefix}：全部 {slices} 片从进度文件读回，这一趟没起工作线程"
    else:
        note = f"{prefix}：{worker_threads} 个工作线程跑了 {freshly_run_slices} 片、从进度文件读回 {resumed_slices} 片（共 {slices} 片）"
    return None, note


# merge 那一行逐片报的数（crates/singlefs-checker-tier/src/crash.rs 的 merge_the_shard_ledgers，逗号分隔、按第几片排）
SHARD_FIELD_NAMES = ("worker_threads", "configured_worker_threads", "worker_threads_sources", "available_parallelism",
                     "resumed_slices", "freshly_run_slices")


def judge_threads_of_each_shard(prefix, finished_line, fields):
    """双机分片 merge 那一趟的 LAYER0_PARALLEL_FINISHED（带 shards= 与逐片的 shard_…=）：每一片照单机的判法判。返回 (原因或 None, 一句说明)。
    判红：逐片字段缺、片数不一、不是整数；某一片跑了片却报 0 个线程；某一片这一趟跑了至少两片、只起了 1 个工作线程、那台机器多于 1 核、
    那一片的线程数不是显式设成 1（来源是环境变量、配的是 1）。机器核数与「显式设成 1」都取那一片账本里记的，不取 merge 这台的。"""
    try:
        shard_count = int(fields["shards"])
        columns = {name: fields[f"shard_{name}"].split(",") for name in SHARD_FIELD_NAMES}
        if shard_count < 1 or any(len(values) != shard_count for values in columns.values()):
            raise ValueError("片数不一")
        numbers = {name: [int(value) for value in values] for name, values in columns.items() if name != "worker_threads_sources"}
    except (KeyError, ValueError):
        return (f"{prefix} 对得上的 LAYER0_PARALLEL_FINISHED 带 shards=，逐片的 shard_…= 字段缺、片数与 shards= 不一或不是整数：{finished_line}"), ""
    notes = []
    for shard_index in range(shard_count):
        spawned = numbers["worker_threads"][shard_index]
        configured = numbers["configured_worker_threads"][shard_index]
        source = columns["worker_threads_sources"][shard_index]
        cores = numbers["available_parallelism"][shard_index]
        resumed = numbers["resumed_slices"][shard_index]
        freshly_run = numbers["freshly_run_slices"][shard_index]
        shard_text = f"第 {shard_index}/{shard_count} 片"
        if freshly_run > 0 and spawned == 0:
            return f"{shard_text}跑了 {freshly_run} 片，却报起了 0 个工作线程：{finished_line}", ""
        explicitly_one = source == "environment_variable" and configured == 1
        if freshly_run >= 2 and spawned == 1 and cores > 1 and not explicitly_one:
            return (f"{shard_text}：那台机器 {cores} 核、线程数没显式设成 1，跑了 {freshly_run} 片却只起了 1 个工作线程"
                    f"（多半是线程数没传进去）：{finished_line}"), ""
        notes.append(f"{shard_text} {spawned} 个线程跑了 {freshly_run} 片、读回 {resumed} 片（那台 {cores} 核）")
    return None, f"{prefix}：双机分片 {shard_count} 片再 merge，{'；'.join(notes)}"


def judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one):
    """判 --full 跑一条用例的日志：返回 (原因的清单, 要记进全绿标记的行, 线程那几句)。原因的清单为空才算判绿。"""
    log_lines = log_text.split("\n")
    problems, recorded_lines, thread_notes = [], [], []
    results = [line for line in log_lines if line.startswith("test result: ")]
    if len(results) != 1 or not PASSED_ONE_TEST_FORM.match(results[0]):
        problems.append(f"读不到恰好一行「test result: ok. 1 passed; 0 failed; …」：过滤到 {case.function} 应当恰好跑一条用例，"
                        f"读到 {len(results)} 行 test result（{'｜'.join(results[:3]) or '一行都没有'}）")
    else:
        recorded_lines.append("test_result=" + results[0])
    count_line_of_prefix = {}
    for prefix in case.count_lines:
        found = [line for line in log_lines if line.startswith(prefix + " ")]
        if len(found) != 1:
            problems.append(f"以「{prefix} 」开头的计数行要恰好一行，读到 {len(found)} 行")
            continue
        count_line_of_prefix[prefix] = found[0]
        recorded_lines.append(found[0])
    for prefix in case.exhaustive_lines:
        line = count_line_of_prefix.get(prefix)
        if line is not None and fields_of_line(line).get("exhaustive") != "true":
            problems.append(f"{prefix} 那一行不带 exhaustive=true，不是全量（枚举到的状态数要等于闭式 1 + Σ(2^|段| − 1)）：{line}")
    for prefix in case.thread_lines:
        line = count_line_of_prefix.get(prefix)
        if line is None:
            continue
        problem, note = judge_worker_threads(prefix, line, log_lines, machine_cores, threads_explicitly_one)
        if problem:
            problems.append(problem)
        else:
            thread_notes.append(note)
    recorded_lines += ["parallel_finished=" + line for line in log_lines if line.startswith(LAYER0_PARALLEL_FINISHED_PREFIX)]
    if not break_is_set("single-worker-threads-field"):
        recorded_lines.append("started_worker_threads=" + started_worker_threads_text(case, thread_notes))
    return problems, recorded_lines, thread_notes


def started_worker_threads_text(case, thread_notes):
    """全绿标记里「实际起的工作线程」那一格：登记了 threads= 的取判线程那几句（LAYER0_PARALLEL_FINISHED 里记的 worker_threads），
    没登记的写「读不到」——配的是几个另记在 configured_worker_threads=，那不是起了几个。"""
    if thread_notes:
        return "；".join(thread_notes)
    if case.thread_lines:
        return "读不到：日志没判绿，线程那一格没判"
    return ("读不到：这条用例没登记 threads=，门禁不读它起了几个工作线程（配的线程数在 configured_worker_threads=，不是起了几个；"
            "日志里有 LAYER0_PARALLEL_FINISHED 的另原样记在 parallel_finished=）")


def configured_worker_threads_text(case, machine_cores, threads, threads_origin):
    """全绿标记里「配的线程数」那一格：传给用例的线程变量（这条用例读的那一个，登记行的 threads-variable=，没登记是 SINGLEFS_LAYER0_THREADS）
    与它的值、配的数是显式设的（调用方设了 SINGLEFS_LAYER0_THREADS）还是取的本机核数、本机几核。"""
    origin = "显式设的" if threads_origin == "explicit" else "没设，取本机核数"
    return f"{case.thread_variable}={threads}（{origin}），本机 {machine_cores} 核"


def read_crash_case_marker(path):
    """全绿标记的 key=value 行、原样的计数行与 input_file 行；读不了返回 None。"""
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            text = handle.read()
    except OSError:
        return None
    marker = {"fields": {}, "raw_lines": [], "input_files": []}
    for line in text.split("\n"):
        if not line or line.startswith("#"):
            continue
        if line.startswith("input_file "):
            marker["input_files"].append(line[len("input_file "):])
        elif re.match(r"^[a-z_]+=", line):
            name, value = line.split("=", 1)
            marker["fields"].setdefault(name, value)
        else:
            marker["raw_lines"].append(line)
    return marker


def crash_case_marker_problems(case, fingerprint, marker):
    """标记内容作不作数：返回原因的清单（空表是作数）。判的与 --full 写它之前判日志的是同一组：换了判法、被手改过、拷错了都在这里现形。"""
    problems = []
    fields = marker["fields"]
    if fields.get("input_hash") != fingerprint:
        problems.append(f"标记里记的输入指纹（{fields.get('input_hash', '读不到')}）与这一次的（{fingerprint}）不同：那一格被改过或拷错了")
    if fields.get("case") != case.key:
        problems.append(f"标记里记的用例（{fields.get('case', '读不到')}）不是 {case.key}")
    if not PASSED_ONE_TEST_FORM.match(fields.get("test_result", "")):
        problems.append(f"标记里没有「test result: ok. 1 passed; 0 failed; …」那一行（{fields.get('test_result', '读不到')}）")
    for prefix in case.count_lines:
        found = [line for line in marker["raw_lines"] if line.startswith(prefix + " ")]
        if len(found) != 1:
            problems.append(f"标记里以「{prefix} 」开头的计数行不是恰好一行（{len(found)} 行）：标记不是 --full 写的，或被改过")
        elif prefix in case.exhaustive_lines and fields_of_line(found[0]).get("exhaustive") != "true":
            problems.append(f"标记里 {prefix} 那一行不带 exhaustive=true：写它的那一趟 --full 没把「不是全量」判红（{found[0]}）")
    return problems


def manifest_differences(earlier_lines, later_lines, earlier_name, later_name):
    """两份「sha256  名字」清单里不同的名字，按字节序排；返回说明句的清单。"""
    earlier = {line[66:]: line[:64] for line in earlier_lines if len(line) > 66}
    later = {line[66:]: line[:64] for line in later_lines if len(line) > 66}
    differences = []
    for name in sorted(set(earlier) | set(later)):
        if name not in earlier:
            differences.append(f"只在{later_name}里：{name}")
        elif name not in later:
            differences.append(f"只在{earlier_name}里：{name}")
        elif earlier[name] != later[name]:
            differences.append(f"内容不同：{name}")
    return differences


def check_crash_case_marker(root, case, fingerprint, manifest_text):
    """这批输入那一格全绿标记作不作数：返回 (作数?, 说明行)。作数时说明行第一行是「ok <路径> <跑完的时刻>」，其后是标记里的计数行；
    不作数时是原因，没有那一格时再拿这条用例最近写的一格与这一次的清单比。"""
    marker_path = crash_case_marker_path(root, case, fingerprint)
    if marker_path is None:
        return False, [f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处读"]
    marker = read_crash_case_marker(marker_path)
    if marker is None:
        lines = [f"这批输入（指纹 {fingerprint[:16]}…）没有全绿标记：{marker_path}"]
        others = sorted(glob.glob(os.path.join(os.path.dirname(marker_path), f"{CRASH_CASE_MARKER_PREFIX}{case.name}.*")),
                        key=os.path.getmtime, reverse=True)
        others = [path for path in others if ".partial." not in os.path.basename(path)]
        if not others:
            lines.append(f"common-dir 里 {case.key} 一格全绿标记都没有。")
            return False, lines
        newest = read_crash_case_marker(others[0]) or {"fields": {}, "input_files": []}
        differences = manifest_differences(newest["input_files"], manifest_text.rstrip("\n").split("\n"), "那一格", "这一次")
        lines.append(f"最近写的一格是 {os.path.basename(others[0])}（跑完于 {newest['fields'].get('finished_utc', '没记')}），"
                     f"与这一次的输入比，不同的共 {len(differences)} 处（最多列 {MARKER_DIFFERENCES_LISTED_AT_MOST} 处）：")
        lines += ["  " + difference for difference in differences[:MARKER_DIFFERENCES_LISTED_AT_MOST]]
        return False, lines
    problems = crash_case_marker_problems(case, fingerprint, marker)
    if problems:
        return False, [f"这批输入那一格全绿标记（{marker_path}）不作数："] + ["  " + problem for problem in problems]
    shown = [line for line in marker["raw_lines"] if any(line.startswith(prefix + " ") for prefix in case.count_lines)]
    return True, [f"ok {marker_path} {marker['fields'].get('finished_utc', '没记')}"] + shown


def write_crash_case_marker(root, case, fingerprint, manifest_text, recorded_lines, details):
    """判绿之后写那一格全绿标记：同目录排他建临时文件、写完改名换上。返回标记路径；取不到 common-dir、写不了抛 InputManifestError。"""
    marker_path = crash_case_marker_path(root, case, fingerprint)
    if marker_path is None:
        raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处写")
    text = ("# 崩溃枚举用例的全绿标记：.claude/gate.d/54-layer0-replay.sh --full 判绿之后经 research/scripts/admission.py crash-case-record 写，"
            "按用例与它的输入指纹分格；整轮门禁的快档与下一趟 --full 按这一格判复用。不进工作树，别手改。\n"
            f"case={case.key}\ntest={case.test_text()}\ninput_hash={fingerprint}\n"
            + "".join(f"{name}={value}\n" for name, value in details)
            + "".join(line + "\n" for line in recorded_lines)
            + "".join(f"input_file {line}\n" for line in manifest_text.rstrip("\n").split("\n") if line))
    try:
        handle_number, temporary_path = tempfile.mkstemp(dir=os.path.dirname(marker_path), prefix=os.path.basename(marker_path) + ".partial.")
        try:
            with os.fdopen(handle_number, "w", encoding="utf-8", errors="surrogateescape") as handle:
                handle.write(text)
            os.replace(temporary_path, marker_path)
        except OSError:
            os.unlink(temporary_path)
            raise
    except OSError as error:
        raise InputManifestError(f"写不了 {marker_path}：{error}") from error
    return marker_path


def machine_core_count(environment):
    """本机核数：os.cpu_count() 与这个进程的 CPU 亲和（os.sched_getaffinity）取小，不认 OMP_NUM_THREADS 这一类环境变量。
    读不到抛 InputManifestError。弄坏开关 cores-from-nproc 下照旧取 nproc（在 environment 的 PATH 里找，它认 OMP_NUM_THREADS）。"""
    if break_is_set("cores-from-nproc"):
        nproc = shutil.which("nproc", path=environment.get("PATH")) or "nproc"
        try:
            completed = subprocess.run([nproc], capture_output=True, text=True, errors="replace", env=dict(environment))
        except OSError as error:
            raise InputManifestError(f"nproc 起不来：{error}") from error
        cores_text = completed.stdout.strip()
        if completed.returncode != 0 or not POSITIVE_INTEGER_FORM.fullmatch(cores_text):
            raise InputManifestError(f"nproc 退 {completed.returncode}、打了「{cores_text}」，不是正整数")
        return int(cores_text)
    counts = [count for count in (os.cpu_count(), len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None) if count]
    if not counts:
        raise InputManifestError("os.cpu_count() 与 CPU 亲和都读不到本机核数")
    return min(counts)


def crash_case_worker_threads(case, environment):
    """--full 跑崩溃枚举用例的线程数：(本机核数, 线程数, "explicit" 或 "default")。调用方设了 SINGLEFS_LAYER0_THREADS（非空）就用它，
    没设取本机核数（machine_core_count）；用例读的线程变量（case.thread_variable）调用方设着什么都不认，由 crash-case-command 盖成这个数。
    crash-case-command、crash-case-judge、crash-case-record 各自现取（54 号不转线程参数）。
    核数读不到、SINGLEFS_LAYER0_THREADS 不是正整数，都抛 InputManifestError。"""
    cores = machine_core_count(environment)
    configured = environment.get(LAYER0_THREADS_VARIABLE, "")
    if not configured:
        return cores, cores, "default"
    if not POSITIVE_INTEGER_FORM.fullmatch(configured):
        raise InputManifestError(f"{LAYER0_THREADS_VARIABLE}={configured} 不是正整数")
    return cores, int(configured), "explicit"


def crash_case_launch(root, case, fingerprint, start_over, environment):
    """--full 跑一条崩溃枚举用例的命令与环境（门禁 54 号原样执行；54 号不进指纹，这一段在判法摘要里）：
    返回 (本机核数, 线程数, "explicit" 或 "default", 续跑的进度目录, 整条命令的词)。
    命令是 env 设好续跑的三个变量与线程数再起 cargo test --release -p <包> --test <测试目标> -- --include-ignored --exact <用例函数> --nocapture：
    进度目录 <git common-dir>/singlefs-layer0-progress/<这批输入的指纹>、SINGLEFS_LAYER0_INPUT_FINGERPRINT=<指纹>；start_over 时
    SINGLEFS_LAYER0_START_OVER=1，不然从调用方的环境里清掉它；调用方环境里的分片开关 SINGLEFS_LAYER0_SHARD 一律清掉（这一趟单机跑整条流，
    分片只由 research/scripts/layer0-shard-run.sh 自己设）；线程数设进 SINGLEFS_LAYER0_THREADS，这条用例读的线程变量（登记行的 threads-variable=）
    不是它时另设成同一个数，盖掉调用方环境里的（弄坏开关 thread-variable-ignored 下只设 SINGLEFS_LAYER0_THREADS）。
    取不到 common-dir、线程数取不到抛 InputManifestError。"""
    common_directory = git_common_directory(root)
    if common_directory is None:
        raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），续跑的进度文件没处放")
    machine_cores, threads, threads_origin = crash_case_worker_threads(case, environment)
    thread_settings = [f"{LAYER0_THREADS_VARIABLE}={threads}"]
    if case.thread_variable != LAYER0_THREADS_VARIABLE:
        thread_settings.append(f"{case.thread_variable}={threads}")
    progress_directory = os.path.join(common_directory, LAYER0_PROGRESS_DIRECTORY_NAME, fingerprint)
    start_over_setting = [f"{LAYER0_START_OVER_VARIABLE}=1"] if start_over else ["-u", LAYER0_START_OVER_VARIABLE]
    # env 的 -u 要写在第一个 NAME=VALUE 之前（之后的 -u 被当成要起的命令）
    shard_setting = [] if break_is_set("keep-caller-shard-switch") else ["-u", LAYER0_SHARD_VARIABLE]
    command = ["env", *shard_setting, *start_over_setting, f"SINGLEFS_LAYER0_PROGRESS_DIRECTORY={progress_directory}",
               f"SINGLEFS_LAYER0_INPUT_FINGERPRINT={fingerprint}", *thread_settings,
               "cargo", "test", "--release", "-p", case.package, "--test", case.target,
               "--", "--include-ignored", "--exact", case.function, "--nocapture"]
    return machine_cores, threads, threads_origin, progress_directory, command


def top_level_definitions(tree):
    """模块级的 def、class 与赋值：名字 → [节点]（同名的几处都留着）。"""
    definitions = {}
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            definitions.setdefault(node.name, []).append(node)
        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
            for target in (node.targets if isinstance(node, ast.Assign) else [node.target]):
                if isinstance(target, ast.Name):
                    definitions.setdefault(target.id, []).append(node)
    return definitions


def source_of_node(lines, node):
    """一个模块级节点的原文（连同它的装饰器）。"""
    first = min([node.lineno] + [decorator.lineno for decorator in getattr(node, "decorator_list", [])])
    return "\n".join(lines[first - 1:node.end_lineno])


def is_named_definition(node):
    """模块级的 def、class，或每个目标都是名字的赋值（top_level_definitions 按名字收的那几种）。"""
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
        return True
    if isinstance(node, (ast.Assign, ast.AnnAssign)):
        return all(isinstance(target, ast.Name) for target in (node.targets if isinstance(node, ast.Assign) else [node.target]))
    return False


def is_docstring(node, tree):
    return node is tree.body[0] and isinstance(node, ast.Expr) and isinstance(node.value, ast.Constant) and isinstance(node.value.value, str)


def imported_module_of_names(tree):
    """模块级 import 进来的名字 → 它来自的模块（import a.b as c 记 c → a.b；from a import b 记 b → a；相对导入的模块名前带点）。"""
    modules = {}
    for node in tree.body:
        if isinstance(node, ast.Import):
            for alias in node.names:
                modules[alias.asname or alias.name.split(".")[0]] = alias.name
        elif isinstance(node, ast.ImportFrom):
            for alias in node.names:
                modules[alias.asname or alias.name] = "." * node.level + (node.module or "")
    return modules


def is_standard_library_module(module):
    return not module.startswith(".") and module.split(".")[0] in sys.stdlib_module_names


def crash_case_judging_digest_text(source):
    """准入模块 source 里崩溃枚举用例的判法摘要原文：返回 (原文的 bytes, 闭包里定义的个数)。
    从 CRASH_CASE_JUDGING_ENTRIES 与分派表 COMMANDS 里 CRASH_CASE_JUDGING_SUBCOMMANDS 那几项指的函数起，顺着 ast 里引用的模块级名字
    （def、class、名字赋值）求闭包；闭包里每个定义的原文按名字排好，再加分派表那几项（子命令 → 函数）、main 的原文（不往下顺：main 引自证，
    顺下去自证就进来了）与模块级其余语句（不是 def / class / 名字赋值的：import、元组解包、if / try / for 块、`if __name__` 那一段，模块文档串除外）
    的原文，按出现的次序；其余语句引到的模块级定义也进闭包。闭包之外的部分（实验准入、门禁复用、排除法、构建环境、自证）改了摘要不变。
    拒算（抛 InputManifestError，54 号照判红）：分派表里少了那几项、那几项的值不是名字（lambda、functools.partial 这一类，指的函数顺不下去）、
    闭包里用到 import 进来的非标准库名字或闭包里的定义自己 import 非标准库模块（判法挪进别的模块，那份模块的原文不进）。
    弄坏开关：whole-module-in-judging-digest 下原文是整份模块，judging-digest-without-dispatch 下不加分派那几项，digest-skips-other-statements 下
    其余语句只加 `if __name__` 那一段（改前的判法），digest-allows-non-name-dispatch 下值不是名字的照原文进，digest-allows-imported-names 下不查 import。"""
    if break_is_set("whole-module-in-judging-digest"):
        return source.encode("utf-8", "surrogateescape"), 0
    tree = ast.parse(source)
    lines = source.split("\n")
    definitions = top_level_definitions(tree)
    subcommands = tuple(name for name in CRASH_CASE_JUDGING_SUBCOMMANDS
                        if not (name == "crash-case-shardable" and break_is_set("shardable-outside-judging-digest")))
    dispatch = {}
    for node in definitions.get("COMMANDS", []):
        if isinstance(node, ast.Assign) and isinstance(node.value, ast.Dict):
            for key, value in zip(node.value.keys, node.value.values):
                if isinstance(key, ast.Constant) and key.value in subcommands:
                    if not isinstance(value, ast.Name) and not break_is_set("digest-allows-non-name-dispatch"):
                        raise InputManifestError(f"准入模块的分派表 COMMANDS 里 {key.value} 的值不是函数名（{ast.unparse(value)}）：它指的判法顺不下去，"
                                                 "判法摘要算不全；写成 \"<子命令>\": <函数名>")
                    dispatch[key.value] = ast.unparse(value)
    missing = [name for name in subcommands if name not in dispatch]
    if missing:
        raise InputManifestError(f"准入模块的分派表 COMMANDS 里没有 {'、'.join(missing)}：判法摘要算不全")
    skips_other_statements = break_is_set("digest-skips-other-statements")
    other_statements = [node for node in tree.body if not is_named_definition(node) and not is_docstring(node, tree)
                        and (not skips_other_statements or (isinstance(node, ast.If) and "__name__" in ast.unparse(node.test)))]
    wanted = set()
    queue = [name for name in CRASH_CASE_JUDGING_ENTRIES + tuple(dispatch.values()) if name in definitions]
    if not skips_other_statements:
        queue += [inner.id for node in other_statements for inner in ast.walk(node) if isinstance(inner, ast.Name) and inner.id in definitions]
    while queue:
        name = queue.pop()
        if name in wanted or name in CRASH_CASE_JUDGING_LEAVES:
            continue
        wanted.add(name)
        for node in definitions[name]:
            queue += [inner.id for inner in ast.walk(node) if isinstance(inner, ast.Name) and inner.id in definitions and inner.id not in wanted]
    if not break_is_set("digest-allows-imported-names"):
        imported = imported_module_of_names(tree)
        for name in sorted(wanted):
            for node in definitions[name]:
                for inner in ast.walk(node):
                    if isinstance(inner, ast.Name) and inner.id in imported and not is_standard_library_module(imported[inner.id]):
                        raise InputManifestError(f"判法闭包里的 {name} 用到 import 进来的 {inner.id}（来自 {imported[inner.id]}，不是标准库）："
                                                 "那份模块的原文不进摘要，判法摘要算不全；判法写在准入模块里")
                    if isinstance(inner, ast.Import):
                        modules = [alias.name for alias in inner.names]
                    elif isinstance(inner, ast.ImportFrom):
                        modules = ["." * inner.level + (inner.module or "")]
                    else:
                        continue
                    outside = [module for module in modules if not is_standard_library_module(module)]
                    if outside:
                        raise InputManifestError(f"判法闭包里的 {name} 自己 import 了 {'、'.join(outside)}（不是标准库）：那份模块的原文不进摘要，"
                                                 "判法摘要算不全；判法写在准入模块里")
    pieces = [f"## {name}\n" + "\n".join(source_of_node(lines, node) for node in definitions[name]) for name in sorted(wanted)]
    if not break_is_set("judging-digest-without-dispatch"):
        pieces.append("## 分派\n" + "".join(f"{name} → {dispatch[name]}\n" for name in subcommands))
    pieces += [f"## {name}\n" + "\n".join(source_of_node(lines, node) for node in definitions.get(name, [])) for name in CRASH_CASE_JUDGING_LEAVES]
    if skips_other_statements:
        pieces += ["## 入口\n" + source_of_node(lines, node) for node in other_statements]
    else:
        pieces.append("## 模块级其余语句\n" + "\n".join(source_of_node(lines, node) for node in other_statements))
    return "\n".join(pieces).encode("utf-8", "surrogateescape"), len(wanted)


def crash_case_judging_digest_line():
    """清单末尾那一行「判法摘要」：(名字, 原文)。原文取正在跑的这一份准入模块（__file__）。"""
    with open(os.path.abspath(__file__), encoding="utf-8", errors="surrogateescape") as handle:
        text, _count = crash_case_judging_digest_text(handle.read())
    return (CRASH_CASE_JUDGING_DIGEST_NAME, text)


# ── 命令行 ────────────────────────────────────────────────────────────────────

def stage_fingerprint(root, stage):
    """一道门禁阶段的输入指纹：登记路径下逐文件清单 + 登记表 + 阶段脚本自己 + 登记行 + 工具链 + 构建环境。返回 (sha256, 文件数, 路径)。
    与 gate_reuse 比对的路径同一份（少一条输入就永远不重跑的那个理由同样成立）。"""
    stage_rows = rows_of_key(read_registration_rows(root), stage)
    if not any(row.input_paths for row in stage_rows):
        raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {stage} 这一行")
    input_paths = [input_path for row in stage_rows for input_path in row.input_paths]
    input_paths += [REGISTRATION_TABLE, f".claude/gate.d/{stage}"]
    registration_text = "".join(row.fingerprint_text() for row in stage_rows).encode("utf-8", "surrogateescape")
    named_lines = [(f"<登记行：{stage}>", registration_text), toolchain_line(root)] + build_environment_lines(root)
    _manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
    return fingerprint, file_count, input_paths


def stage_marker_path(root, stage, fingerprint):
    """这一道这批输入那一格全绿标记：git common-dir 里「前缀 + 阶段文件名 + . + 指纹」；取不到 common-dir 返回 None。"""
    common_directory = git_common_directory(root)
    if common_directory is None:
        return None
    return os.path.join(common_directory, f"{STAGE_MARKER_PREFIX}{stage}.{fingerprint}")


def check_stage_marker(root, stage, fingerprint):
    """这一道这批输入的全绿标记作不作数：返回 (作数?, 说明行)。作数时第一行是「ok <路径> <跑完的时刻>」；
    过了复用上限（SINGLEFS_REUSE_HOURS，默认 24 小时；内容没变不代表环境没变）不作数。"""
    marker_path = stage_marker_path(root, stage, fingerprint)
    if marker_path is None:
        return False, [f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处读"]
    marker = read_crash_case_marker(marker_path)
    if marker is None:
        return False, [f"这批输入（指纹 {fingerprint[:16]}…）没有全绿标记：{marker_path}"]
    finished_text = marker["fields"].get("finished_utc", "")
    try:
        finished_epoch = int(time.mktime(time.strptime(finished_text, "%Y-%m-%dT%H:%M:%SZ"))) - (time.mktime(time.localtime(0)) - time.mktime(time.gmtime(0)))
    except (ValueError, OverflowError):
        return False, [f"全绿标记 {marker_path} 的 finished_utc 读不出（{finished_text!r}），不作数"]
    hours_text = os.environ.get("SINGLEFS_REUSE_HOURS", "") or "24"
    try:
        reuse_limit_hours = int(hours_text)
    except ValueError:
        reuse_limit_hours = 0
    age_hours = max(0, int(time.time()) - int(finished_epoch)) // 3600
    if age_hours >= reuse_limit_hours:
        return False, [f"全绿标记 {marker_path} 跑完于 {finished_text}、{age_hours} 小时前，到了复用上限 {hours_text} 小时，不作数"]
    return True, [f"ok {marker_path} {finished_text}"]


def write_stage_marker(root, stage, fingerprint, file_count):
    """判绿之后写这一道这批输入的全绿标记：同目录排他建临时文件、写完改名换上。返回标记路径；取不到 common-dir、写不了抛 InputManifestError。"""
    marker_path = stage_marker_path(root, stage, fingerprint)
    if marker_path is None:
        raise InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir），全绿标记没处写")
    text = ("# 门禁阶段的全绿标记：checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 判绿之后经 research/scripts/admission.py stage-marker-write 写，按阶段与它登记输入的指纹分格；"
            "gate-reuse 按这一格判复用。不进工作树，别手改。\n"
            f"stage={stage}\ninput_hash={fingerprint}\ninput_file_count={file_count}\n"
            f"finished_utc={time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}\njudged_root={os.path.abspath(root)}\n"
            f"heavy_prefix={os.environ.get('SINGLEFS_HEAVY_TESTS', '')}\n")
    try:
        handle_number, temporary_path = tempfile.mkstemp(dir=os.path.dirname(marker_path), prefix=os.path.basename(marker_path) + ".partial.")
        try:
            with os.fdopen(handle_number, "w", encoding="utf-8", errors="surrogateescape") as handle:
                handle.write(text)
            os.replace(temporary_path, marker_path)
        except OSError:
            os.unlink(temporary_path)
            raise
    except OSError as error:
        raise InputManifestError(f"写不了 {marker_path}：{error}") from error
    return marker_path


def command_stage_fingerprint(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py stage-fingerprint <项目根> <阶段文件名>")
        print("     → 怎么办：阶段文件名是 .claude/gate.d/ 下那一份的文件名，要在 stage-inputs.tsv 里登记过")
        return EXIT_REGISTRATION_ERROR
    try:
        fingerprint, file_count, _paths = stage_fingerprint(*arguments)
    except (RegistrationError, InputManifestError) as error:
        print(f"算不出 {arguments[1]} 的输入指纹：{error}")
        return EXIT_REGISTRATION_ERROR
    print(f"{fingerprint} {file_count}")
    return 0


def command_stage_marker_check(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py stage-marker-check <项目根> <阶段文件名>")
        print("     → 怎么办：退 0 有作数的全绿标记（第一行 ok <路径> <时刻>），退 1 没有或不作数")
        return EXIT_REGISTRATION_ERROR
    try:
        fingerprint, _file_count, _paths = stage_fingerprint(*arguments)
    except (RegistrationError, InputManifestError) as error:
        print(f"判不了 {arguments[1]} 的全绿标记：{error}")
        return EXIT_REGISTRATION_ERROR
    valid, lines = check_stage_marker(arguments[0], arguments[1], fingerprint)
    for line in lines:
        print(line)
    return 0 if valid else 1


def command_stage_marker_write(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py stage-marker-write <项目根> <阶段文件名>")
        print("     → 怎么办：只在这一道判绿之后调，它按此刻的输入指纹写标记、打路径")
        return EXIT_REGISTRATION_ERROR
    try:
        fingerprint, file_count, _paths = stage_fingerprint(*arguments)
        marker_path = write_stage_marker(arguments[0], arguments[1], fingerprint, file_count)
    except (RegistrationError, InputManifestError) as error:
        print(f"{arguments[1]} 的全绿标记没写成：{error}")
        return 1
    print(marker_path)
    return 0


def command_gate_reuse(arguments):
    if len(arguments) < 2 or not arguments[0] or not arguments[1]:
        print("  ✗ 用法：stage-must-run.sh <项目根> <阶段文件名>")
        print("     → 怎么办：阶段名照 .claude/gate.d/ 下的文件名写，例 checker-tier-crates-mutation-replay.sh。")
        return EXIT_GATE_MUST_RUN
    if break_is_set("raise-in-gate-reuse"):
        # 故意不接住：进程带着 traceback 退 1，看 stage-must-run.sh 会不会把它当成「可跳过」
        raise RuntimeError("弄坏开关 raise-in-gate-reuse")
    try:
        exit_code, reason = gate_reuse(arguments[0], arguments[1])
    except Exception as error:  # 模块自己出错一律按要跑处理：出错就跳过等于把这道门禁关掉
        print(f"判不出来：准入模块出错（{type(error).__name__}：{error}），按要跑处理")
        return EXIT_GATE_MUST_RUN
    print(reason)
    return exit_code


def command_experiment(arguments):
    if len(arguments) != 2:
        say("  ✗ 用法：admission.py experiment <项目根> <实验键>")
        say("     → 怎么办：实验键照 .claude/gate.d/stage-inputs.tsv 的第一列写，例 E142、E142/layer0")
        return EXIT_REGISTRATION_ERROR
    exit_code, header = admit_experiment(os.path.abspath(arguments[0]), arguments[1])
    for header_line in header:
        print(header_line)
    return exit_code


def command_paths(arguments):
    if len(arguments) != 2:
        say("  ✗ 用法：admission.py paths <项目根> <键>")
        say("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 的第一列写")
        return EXIT_REGISTRATION_ERROR
    try:
        input_paths, _rows = resolve_input_paths(read_registration_rows(arguments[0]), arguments[1])
    except RegistrationError as error:
        say(f"  ✗ {error}")
        say(f"     → 怎么办：在 {REGISTRATION_TABLE} 给这个键登记它读的路径（制表符分隔，照别的行写）")
        return EXIT_REGISTRATION_ERROR
    for input_path in input_paths:
        print(input_path)
    return 0


def named_line_requests(options, allowed_flags):
    """清单末尾那几行的参数：[(种类, 名字, 文件)]，种类是 file 或 flag 本身；写法不对返回 None。"""
    requested = []
    index = 0
    while index < len(options):
        option = options[index]
        if option == "--extra-file" and index + 2 < len(options):
            requested.append(("file", options[index + 1], options[index + 2]))
            index += 3
        elif option in allowed_flags:
            requested.append((option, None, None))
            index += 1
        else:
            return None
    return requested


def named_lines_of(root, requested, registration_line=None):
    """按参数次序算末尾那几行；--registration-row 取 registration_line。读不了 --extra-file 的文件抛 OSError。"""
    named_lines = []
    for kind, name, path in requested:
        if kind == "file":
            with open(path, "rb") as handle:
                named_lines.append((name, handle.read()))
        elif kind == "--toolchain":
            named_lines.append(toolchain_line(root))
        elif kind == "--build-environment":
            named_lines.extend(build_environment_lines(root))
        elif kind == "--judging-digest":
            named_lines.append(crash_case_judging_digest_line())
        else:
            named_lines.append(registration_line)
    return named_lines


def command_manifest(arguments):
    requested = named_line_requests(arguments[3:], ("--toolchain", "--registration-row", "--build-environment")) if len(arguments) >= 3 else None
    if requested is None:
        say("  ✗ 用法：admission.py manifest <项目根> <键> <清单文件> [--extra-file <名字> <文件>]... [--registration-row] [--toolchain] [--build-environment]")
        say("     → 怎么办：末尾那几行按参数次序加，例 --extra-file \"<判它的 54 号：54-layer0-replay.sh>\" <54 号的路径> --toolchain --build-environment")
        return EXIT_REGISTRATION_ERROR
    root, key, manifest_file = arguments[0], arguments[1], arguments[2]
    try:
        rows = read_registration_rows(root)
        input_paths, contributing_rows = resolve_input_paths(rows, key)
        registration_text = "".join(row.fingerprint_text() for row in contributing_rows)
        named_lines = named_lines_of(root, requested, (f"<登记行：{key}>", registration_text.encode("utf-8", "surrogateescape")))
        manifest, fingerprint, file_count = input_manifest(root, input_paths, named_lines)
    except (RegistrationError, InputManifestError, OSError) as error:
        say(f"  ✗ 写不出 {key} 的输入清单：{error}")
        say("     → 怎么办：在项目根跑 git ls-files -co --exclude-standard -- <登记的路径> 看列不列得出文件；--extra-file 给的文件要读得了；"
            "cargo -V && rustc -V 要跑得出来")
        return EXIT_REGISTRATION_ERROR
    with open(manifest_file, "wb") as handle:
        handle.write(manifest)
    print(f"{fingerprint} {file_count}")
    return 0


def command_gate_preconditions(arguments):
    if len(arguments) != 2 or not arguments[0] or not arguments[1]:
        print("  ✗ 用法：admission.py gate-preconditions <阶段所在的仓根> <阶段文件名>")
        print("     → 怎么办：阶段里写成 python3 <仓根>/research/scripts/admission.py gate-preconditions <仓根> \"$(basename \"$0\")\"")
        return EXIT_REGISTRATION_ERROR
    root, stage = arguments
    try:
        stage_rows = rows_of_key(read_registration_rows(root), stage)
        if not stage_rows:
            raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {stage} 这一行，判不出这一道开跑前要什么")
        missing = [] if break_is_set("skip-gate-preconditions") else judge_gate_preconditions(root, stage_rows)
    except RegistrationError as error:
        print(f"  ✗ {stage} 的准入登记有错：{error}")
        print(f"     → 怎么办：在 {REGISTRATION_TABLE} 给 {stage} 登记一行，或改那一行（门禁行第三列只认 command=、readwrite=、probe=、environment=，"
              "写法见 research/scripts/admission.py 文件头）；python3 research/scripts/admission.py --selftest 的最后一格点得出登记表哪一行写错")
        return EXIT_REGISTRATION_ERROR
    except Exception as error:  # 判不出前提就不开跑：出错就放行等于把前提这一道关掉
        print(f"  ✗ 判不出 {stage} 的前提：准入模块出错（{type(error).__name__}：{error}）")
        print("     → 怎么办：单跑 python3 research/scripts/admission.py gate-preconditions <仓根> <阶段文件名> 看报错，修准入模块或登记表那一行")
        return EXIT_GATE_PRECONDITION_MISSING
    if missing:
        why = "；".join(row.comment for row in stage_rows if row.comment) or "那一行没写注释"
        print(f"  ✗ 前提没齐，{stage} 不开跑：共 {len(missing)} 条（登记在 {REGISTRATION_TABLE} 的 {stage} 那一行第三列）")  # gate-lint:summary
        for description in missing:
            print(f"       {description}")  # gate-lint:detail
        print(f"     → 怎么办：把这几条办齐再跑这一道（这一道为什么要它们：{why}）；"
              "前提写错了就改登记表那一行，不许绕过。这一道判红、不记本次未跑：前提没齐时它一格都没判")
        return EXIT_GATE_PRECONDITION_MISSING
    return 0


def command_gate_record_environment(arguments):
    if len(arguments) != 2 or not arguments[0] or not arguments[1]:
        print("  ! 用法：admission.py gate-record-environment <项目根> <阶段文件名>；这一趟的环境没记下，下一趟照跑")
        return 0
    line = record_gate_environment(arguments[0], arguments[1])
    if line:
        print(line)
    return 0


def command_keys(arguments):
    if len(arguments) != 1:
        say("  ✗ 用法：admission.py keys <项目根>")
        say("     → 怎么办：只给项目根一个参数")
        return EXIT_REGISTRATION_ERROR
    seen = []
    for row in read_registration_rows(arguments[0]):
        if EXPERIMENT_KEY_FORM.match(row.key) and row.key not in seen:
            seen.append(row.key)
    for key in seen:
        print(key)
    return 0


def command_crash_cases(arguments):
    """核登记的崩溃枚举用例，逐条打「键、包、测试目标、用例函数」；登记有错退 2，原因不带 ✗（阶段自己打 ✗ 与出路）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：admission.py crash-cases <项目根>")
        print("     → 怎么办：只给项目根一个参数")
        return EXIT_REGISTRATION_ERROR
    root = arguments[0]
    rows = read_registration_rows(root)
    crash_case_rows = [row for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]
    problems = [problem for problem in lint_registration_rows(crash_case_rows, root)]
    if problems:
        for problem in problems:
            print(f"{REGISTRATION_TABLE} {problem}")
        return EXIT_REGISTRATION_ERROR
    for case in crash_cases_of(crash_case_rows):
        print(f"{case.key}\t{case.package}\t{case.target}\t{case.function}")
    return 0


def command_crash_case_manifest(arguments):
    requested = (named_line_requests(arguments[3:], ("--toolchain", "--build-environment", "--judging-digest")) if len(arguments) >= 3
                 else None)
    if requested is None:
        print("  ✗ 用法：admission.py crash-case-manifest <项目根> <键> <清单文件> [--extra-file <名字> <文件>]... [--judging-digest] [--toolchain] "
              "[--build-environment]")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写；末尾那几行按参数次序加")
        return EXIT_REGISTRATION_ERROR
    root, key, manifest_file = arguments[0], arguments[1], arguments[2]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        manifest, fingerprint, file_count, excluded = crash_case_manifest(root, case, named_lines_of(root, requested))
    except (RegistrationError, InputManifestError, OSError, SyntaxError) as error:
        print(f"写不出 {key} 的输入清单：{error}")
        return EXIT_REGISTRATION_ERROR
    with open(manifest_file, "wb") as handle:
        handle.write(manifest)
    print(f"{fingerprint} {file_count} {len(excluded)}")
    return 0


def command_crash_case_marker_check(arguments):
    if len(arguments) != 4:
        print("  ✗ 用法：admission.py crash-case-marker-check <项目根> <键> <指纹> <清单文件>")
        print("     → 怎么办：指纹与清单文件取 crash-case-manifest 那一趟的")
        return EXIT_REGISTRATION_ERROR
    root, key, fingerprint, manifest_file = arguments
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        with open(manifest_file, encoding="utf-8", errors="surrogateescape") as handle:
            manifest_text = handle.read()
    except (RegistrationError, OSError) as error:
        print(f"判不了 {key} 的全绿标记：{error}")
        return EXIT_REGISTRATION_ERROR
    valid, lines = check_crash_case_marker(root, case, fingerprint, manifest_text)
    for line in lines:
        print(line)
    return 0 if valid else 1


def option_values(options, names):
    """--名字 值 成对的参数：返回 {名字: 值}；有不认得的、缺值的返回 None。"""
    values = {}
    index = 0
    while index < len(options):
        if options[index] not in names or index + 1 >= len(options):
            return None
        values[options[index]] = options[index + 1]
        index += 2
    return values


def forwarded_worker_threads(arguments):
    """弄坏开关 judge-takes-forwarded-threads 下 crash-case-judge / crash-case-record 照旧收的 --machine-cores / --threads / --threads-origin
    （54 号转过来的，改前的写法）：交 (核数, 线程数, 来源) 或 None。开关没开时交 None、调用方不认这三个参数。"""
    if not break_is_set("judge-takes-forwarded-threads"):
        return None
    values = option_values(arguments, ("--machine-cores", "--threads", "--threads-origin"))
    if not values or set(values) != {"--machine-cores", "--threads", "--threads-origin"}:
        return None
    return int(values["--machine-cores"]), int(values["--threads"]), values["--threads-origin"]


def command_crash_case_judge(arguments):
    """判 --full 跑那一条的日志：核数、线程数与它是不是显式设的由这里现取（crash_case_worker_threads，与 crash-case-command 交给用例的同一个算法、
    同一份环境），不收调用方转过来的。"""
    forwarded = forwarded_worker_threads(arguments[4:]) if len(arguments) > 4 else None
    if len(arguments) != 4 and forwarded is None:
        print("  ✗ 用法：admission.py crash-case-judge <项目根> <键> <日志> <记录行文件>")
        print("     → 怎么办：核数与线程数由它自己现取（与 crash-case-command 同一份环境），不再带 --machine-cores / --threads / --threads-origin")
        return EXIT_REGISTRATION_ERROR
    root, key, log_file, recorded_file = arguments[:4]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        with open(log_file, encoding="utf-8", errors="surrogateescape") as handle:
            log_text = handle.read()
        machine_cores, threads, threads_origin = forwarded or crash_case_worker_threads(case, os.environ)
    except (RegistrationError, InputManifestError, OSError) as error:
        print(f"判不了 {key} 的日志：{error}")
        return EXIT_REGISTRATION_ERROR
    threads_explicitly_one = threads_origin == "explicit" and threads == 1
    problems, recorded_lines, thread_notes = judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one)
    if problems:
        for problem in problems:
            print(problem)
        return 1
    with open(recorded_file, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write("".join(line + "\n" for line in recorded_lines))
    for note in thread_notes:
        print(note)
    return 0


def command_crash_case_command(arguments):
    """--full 跑一条崩溃枚举用例的命令与环境（crash_case_launch），stdout 以 NUL 分隔：本机核数、线程数、explicit 或 default、续跑的进度目录，
    其后是整条命令的词。有错退 2，原因打在 stdout（不带 ✗，54 号自己打 ✗ 与出路）。"""
    start_over = arguments[3:] == ["--start-over"]
    if len(arguments) not in (3, 4) or (len(arguments) == 4 and not start_over) or not re.fullmatch(r"[0-9a-f]{64}", arguments[2] if len(arguments) > 2 else ""):
        print("  ✗ 用法：admission.py crash-case-command <项目根> <键> <输入指纹> [--start-over]")
        print("     → 怎么办：指纹取开跑时 crash-case-manifest 那一趟的（64 位十六进制）；要丢掉进度文件、从头跑才带 --start-over")
        return EXIT_REGISTRATION_ERROR
    root, key, fingerprint = arguments[:3]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        machine_cores, threads, threads_origin, progress_directory, command = crash_case_launch(root, case, fingerprint, start_over, os.environ)
    except (RegistrationError, InputManifestError) as error:
        print(f"交不出 {key} 的命令：{error}")
        return EXIT_REGISTRATION_ERROR
    words = [str(machine_cores), str(threads), threads_origin, progress_directory, *command]
    sys.stdout.write("".join(word + "\0" for word in words))
    return 0


def command_crash_case_record(arguments):
    """判绿之后写那一格全绿标记；配的线程那一格（configured_worker_threads=）由这里现取（crash_case_worker_threads），不收调用方转过来的。"""
    names = ("--files", "--excluded", "--started", "--judged-root")
    forwarded = forwarded_worker_threads(arguments[13:]) if len(arguments) > 13 else None
    values = option_values(arguments[5:13] if forwarded else arguments[5:], names) if len(arguments) >= 5 else None
    if values is None or set(values) != set(names):
        print("  ✗ 用法：admission.py crash-case-record <项目根> <键> <指纹> <清单文件> <记录行文件> --files <数> --excluded <数> "
              "--started <时刻> --judged-root <路径>")
        print("     → 怎么办：指纹、清单文件、文件数与减去的文件数取开跑时 crash-case-manifest 那一趟的，记录行文件取 crash-case-judge 写的；"
              "核数与线程数由它自己现取，不再带 --machine-cores / --threads / --threads-origin")
        return EXIT_REGISTRATION_ERROR
    root, key, fingerprint, manifest_file, recorded_file = arguments[:5]
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
        with open(manifest_file, encoding="utf-8", errors="surrogateescape") as handle:
            manifest_text = handle.read()
        with open(recorded_file, encoding="utf-8", errors="surrogateescape") as handle:
            recorded_lines = [line for line in handle.read().split("\n") if line]
        configured = configured_worker_threads_text(case, *(forwarded or crash_case_worker_threads(case, os.environ)))
        thread_field = ("worker_threads", configured) if break_is_set("single-worker-threads-field") else ("configured_worker_threads", configured)
        details = [("input_file_count", values["--files"]), ("excluded_file_count", values["--excluded"]),
                   ("started_utc", values["--started"]), ("finished_utc", time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())),
                   ("judged_root", values["--judged-root"]), thread_field]
        marker_path = write_crash_case_marker(root, case, fingerprint, manifest_text, recorded_lines, details)
    except (RegistrationError, InputManifestError, OSError) as error:
        print(f"{key} 的全绿标记没写成：{error}")
        return 1
    print(marker_path)
    return 0


def command_crash_case_shardable(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：admission.py crash-case-shardable <项目根> <键>")
        print("     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 开头的那一行写")
        return EXIT_REGISTRATION_ERROR
    root, key = arguments
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
    except RegistrationError as error:
        print(f"找不到 {key}：{error}")
        return EXIT_REGISTRATION_ERROR
    if case.is_shardable_across_machines:
        print(f"{key} 登记了 shard={CRASH_CASE_SHARD_ACROSS_MACHINES}")
        return 0
    print(f"{key} 没登记 shard={CRASH_CASE_SHARD_ACROSS_MACHINES}：单机跑")
    return 1


def command_crash_case_marker_path(arguments):
    if len(arguments) != 3:
        print("  ✗ 用法：admission.py crash-case-marker-path <项目根> <键> <指纹>")
        print("     → 怎么办：指纹取 crash-case-manifest 那一趟的")
        return EXIT_REGISTRATION_ERROR
    root, key, fingerprint = arguments
    try:
        case = crash_case_of_key(read_registration_rows(root), key)
    except RegistrationError as error:
        print(f"找不到 {key}：{error}")
        return EXIT_REGISTRATION_ERROR
    marker_path = crash_case_marker_path(root, case, fingerprint)
    if marker_path is None:
        print(f"{root} 不是 git 工作树（取不到 git common-dir）")
        return EXIT_REGISTRATION_ERROR
    print(marker_path)
    return 0


# ── 自证 ──────────────────────────────────────────────────────────────────────

SELFTEST_HERE = os.path.dirname(os.path.abspath(__file__))


class Selftest:
    def __init__(self):
        self.failures = 0
        self.checked = 0
        self.names = []

    def expect(self, name, condition, detail):
        self.checked += 1
        self.names.append(name)
        if condition:
            print(f"  ✓ {name}")
        else:
            print(f"  ✗ {name}：{detail}")  # gate-lint:detail
            self.failures += 1


def run_quietly(command, environment_changes=None, working_directory=None):
    """外面设的 ADMISSION_BREAK 照样传给子进程：`ADMISSION_BREAK=skip-unchanged admission.py --selftest` 要看到「输入没变」那几格红。"""
    environment = dict(os.environ)
    for variable in (FORCE_RERUN_VARIABLE, "SINGLEFS_GATE_FULL", "SINGLEFS_STAGED_TREE", "SINGLEFS_REUSE_HOURS"):
        environment.pop(variable, None)
    environment.update(environment_changes or {})
    completed = subprocess.run(command, capture_output=True, text=True, errors="replace", env=environment, cwd=working_directory)
    return completed.returncode, completed.stdout, completed.stderr


def write_text(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def build_selftest_repository(work):
    """一个临时小仓：一道门禁行、一个实验 E900 与它的第二段 E900/second、replay.sh 登记表、一份没有指纹的老产物、问题单。"""
    environment = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t", "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}
    run_quietly(["git", "init", "-q", "-b", "master", work], environment)
    write_text(os.path.join(work, REGISTRATION_TABLE),
               "# 样本登记表\n"
               "59-demo.sh\tcrates/ Cargo.toml\t# 门禁行\n"
               "E900\tresearch/src/e900.rs research/data/\t# 样本实验\n"
               "E900/second\t@E900\tquestion-row=research/questions.md#6:够判[：:][^（(|]*对照(本身)?是好的"
               " product-field=E900:verdict:control_ok=true\t# 样本第二段\n")
    write_text(os.path.join(work, "research/src/e900.rs"), "fn main() {}\n")
    write_text(os.path.join(work, "research/data/input.txt"), "one\n")
    write_text(os.path.join(work, "crates/demo/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
    write_text(os.path.join(work, "Cargo.toml"), "[workspace]\n")
    write_text(os.path.join(work, REPLAY_SCRIPT), "TABLE=$(cat <<'TSV'\nE900|@driver_e900||e900-2026-09-01.out|exact\nTSV\n)\n")
    write_text(os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-01.out"),
               "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n")
    write_questions(work, "开着")
    run_quietly(["git", "-C", work, "add", "-A"], environment)
    run_quietly(["git", "-C", work, "commit", "-qm", "base"], environment)


def write_questions(work, row_six_status):
    write_text(os.path.join(work, "research/questions.md"),
               "| # | 问题 | 状态 |\n|---|---|---|\n"
               "| 5 | 别的 | **够判：能** |\n"
               f"| 6 | 阳性对照是好的还是坏的 | {row_six_status} |\n")


def point_replay_row_at(work, product_name):
    write_text(os.path.join(work, REPLAY_SCRIPT), f"TABLE=$(cat <<'TSV'\nE900|@driver_e900||{product_name}|exact\nTSV\n)\n")


def run_stage_marker_cells(selftest, module):
    """checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 的全绿标记：没有时判不了、写了之后 gate-reuse 不看暂存树也可跳过、弄坏开关下转红、过上限不作数、输入改了就没有它的格。"""
    work = tempfile.mkdtemp(prefix="admission-stage-marker-")
    try:
        build_selftest_repository(work)
        def call(*arguments, environment_changes=None):
            return run_quietly([sys.executable, module, *arguments], environment_changes)
        exit_code, output, _messages = call("stage-fingerprint", work, "59-demo.sh")
        selftest.expect("stage-fingerprint 打「<64 位指纹> <文件数>」", exit_code == 0 and re.fullmatch(r"[0-9a-f]{64} [0-9]+", output.strip()) is not None,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = call("stage-marker-check", work, "59-demo.sh")
        selftest.expect("没有全绿标记时 stage-marker-check 退 1、说没有", exit_code == 1 and "没有全绿标记" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh")
        selftest.expect("没有标记、也不是 gate-staged.sh 起的：gate-reuse 判要跑", exit_code == EXIT_GATE_MUST_RUN, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = call("stage-marker-write", work, "59-demo.sh")
        marker_path = output.strip()
        selftest.expect("stage-marker-write 把标记写进 git common-dir、打它的路径",
                        exit_code == 0 and os.path.isfile(marker_path) and os.path.basename(marker_path).startswith(STAGE_MARKER_PREFIX + "59-demo.sh."),
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = call("stage-marker-check", work, "59-demo.sh")
        selftest.expect("写过之后 stage-marker-check 退 0、第一行 ok", exit_code == 0 and output.startswith("ok "), f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh")
        selftest.expect("有全绿标记时 gate-reuse 不看暂存树也判可跳过（退 10）", exit_code == EXIT_GATE_MAY_SKIP and "全绿标记" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh", environment_changes={BREAK_VARIABLE: "stage-marker-ignored"})
        selftest.expect("弄坏开关 stage-marker-ignored 下「有标记可跳过」那一格转红（判要跑）", exit_code == EXIT_GATE_MUST_RUN,
                        f"弄坏之后仍退 {exit_code}：这一格分不出标记看没看")
        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh", environment_changes={"SINGLEFS_REUSE_HOURS": "0"})
        selftest.expect("复用上限 0 小时：标记再新也不作数、判要跑", exit_code == EXIT_GATE_MUST_RUN, f"退 {exit_code}，stdout「{output.strip()}」")
        write_text(os.path.join(work, "crates/demo/src/lib.rs"), "pub fn one() -> u32 { 2 }\n")
        exit_code, output, _messages = call("gate-reuse", work, "59-demo.sh")
        selftest.expect("登记的输入改了一个字节就没有它那一格标记、判要跑", exit_code == EXIT_GATE_MUST_RUN, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = call("stage-marker-check", work, "not-registered.sh")
        selftest.expect("没登记的阶段 stage-marker-check 退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}，stdout「{output.strip()}」")
    finally:
        shutil.rmtree(work, ignore_errors=True)


def run_selftest():
    selftest = Selftest()
    work = tempfile.mkdtemp(prefix="admission-selftest-")
    module = os.path.abspath(__file__)

    def experiment(key, environment_changes=None):
        return run_quietly([sys.executable, module, "experiment", work, key], environment_changes)
    try:
        build_selftest_repository(work)

        # ① 老产物没有指纹 ⇒ 放行，报「判不了」，产物头带键与 64 位十六进制指纹
        exit_code, output, messages = experiment("E900")
        first_header = output.strip().split("\n")[0] if output.strip() else ""
        selftest.expect("老产物没有指纹行时放行并报判不了", exit_code == 0 and "判不了" in messages,
                        f"退 {exit_code}，stderr：{messages.strip()}")
        selftest.expect("放行时产物头第一行是带键的输入指纹",
                        re.fullmatch(r"E7INPUT name=input_fingerprint key=E900 sha256=[0-9a-f]{64} files=2", first_header) is not None,
                        f"头一行是「{first_header}」")

        # ② 存一份带这个指纹的产物、replay 指到它 ⇒ 输入没变，拒绝，退 77，出路里有强制开关
        write_text(os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-02.out"),
                   first_header + "\nE7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n")
        point_replay_row_at(work, "e900-2026-09-02.out")
        # replay.sh 不是 E900 的输入，改它不动指纹；产物也不在输入里
        exit_code, output, messages = experiment("E900")
        unchanged_cell_ok = exit_code == EXIT_INPUTS_UNCHANGED and output == "" and FORCE_RERUN_VARIABLE in messages
        selftest.expect("输入没变时拒绝（退 77、stdout 为空、出路写强制开关）", unchanged_cell_ok,
                        f"退 {exit_code}，stdout「{output.strip()}」，stderr：{messages.strip()}")
        exit_code, _output, _messages = experiment("E900", {BREAK_VARIABLE: "skip-unchanged"})
        selftest.expect("弄坏开关 skip-unchanged 下「输入没变」那一格红（放行了）", exit_code == 0,
                        f"弄坏之后仍退 {exit_code}：这一格判不出比对被跳过")

        # ③ 改一个输入字节 ⇒ 放行，报「变了」；写回原样 ⇒ 又拒绝
        write_text(os.path.join(work, "research/data/input.txt"), "One\n")
        exit_code, _output, messages = experiment("E900")
        selftest.expect("改一个输入字节就放行并报变了", exit_code == 0 and "变了" in messages, f"退 {exit_code}，stderr：{messages.strip()}")
        write_text(os.path.join(work, "research/data/input.txt"), "one\n")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("字节写回原样又判没变", exit_code == EXIT_INPUTS_UNCHANGED, f"退 {exit_code}")

        # ③' 登记路径下新加一个未跟踪的文件 ⇒ 放行（git 列得出没被忽略的未跟踪文件）
        write_text(os.path.join(work, "research/data/extra.txt"), "new\n")
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("登记目录下新加一个未跟踪文件就放行", exit_code == 0, f"退 {exit_code}")
        os.remove(os.path.join(work, "research/data/extra.txt"))

        # ④ 强制开关带理由 ⇒ 放行，理由进产物头；空理由 ⇒ 退 2
        exit_code, output, _messages = experiment("E900", {FORCE_RERUN_VARIABLE: "换了机器  复核一次"})
        selftest.expect("强制开关带理由时放行、理由原样进产物头",
                        exit_code == 0 and "E7INPUT name=forced_rerun key=E900 overrode_unchanged=true reason=换了机器 复核一次" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, _output, _messages = experiment("E900", {FORCE_RERUN_VARIABLE: "   "})
        selftest.expect("强制开关理由为空时退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")

        # ⑤ 前提：问题单第 6 行「开着」⇒ 拒；办齐了 ⇒ 放行（别的键 E900 的同一个指纹不算数，照样判不了）；产物判决字段不对 ⇒ 拒
        exit_code, _output, messages = experiment("E900/second")
        selftest.expect("前提没齐时拒绝（退 3）并点名那一行", exit_code == EXIT_PRECONDITION_MISSING and "第 6 行的状态是「开着」" in messages,
                        f"退 {exit_code}，stderr：{messages.strip()}")
        exit_code, _output, _messages = experiment("E900/second", {BREAK_VARIABLE: "skip-preconditions"})
        selftest.expect("弄坏开关 skip-preconditions 下「前提没齐」那一格红（放行了）", exit_code == 0,
                        f"弄坏之后仍退 {exit_code}：这一格判不出前提被跳过")
        write_questions(work, "**够判：装置的期望值过时、对照本身是好的**（样本）")
        exit_code, _output, messages = experiment("E900/second")
        selftest.expect("前提齐了放行，且别的键的同一个指纹不算数", exit_code == 0 and "判不了" in messages,
                        f"退 {exit_code}，stderr：{messages.strip()}")
        write_text(os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-02.out"),
                   first_header + "\nE7RESULT name=verdict control_ok=false\nE7RESULT name=done emitted=2\n")
        exit_code, _output, messages = experiment("E900/second")
        selftest.expect("产物判决字段不是要的值时拒绝（退 3）", exit_code == EXIT_PRECONDITION_MISSING and "control_ok=false" in messages,
                        f"退 {exit_code}，stderr：{messages.strip()}")
        write_questions(work, "**够判：对照本身坏了**")
        exit_code, _output, _messages = experiment("E900/second")
        selftest.expect("问题单判的是另一个候选时拒绝（退 3）", exit_code == EXIT_PRECONDITION_MISSING, f"退 {exit_code}")

        # ⑥ 登记错一律退 2：没登记的键、列不出文件的路径、认不出的条件
        exit_code, _output, _messages = experiment("E901")
        selftest.expect("没登记的实验键退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        table_path = os.path.join(work, REGISTRATION_TABLE)
        with open(table_path, encoding="utf-8") as handle:
            good_table = handle.read()
        write_text(table_path, good_table + "E902\tresearch/nowhere/\t# 写错的路径\nE903\tresearch/src/\tbogus=1\t# 认不出的条件\n"
                               "E905\tresearch/\t# 罩住了 research/results/ 与 replay.sh\n")
        exit_code, _output, _messages = experiment("E902")
        selftest.expect("登记的路径列不出文件时退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        exit_code, _output, messages = experiment("E905")
        selftest.expect("登记的路径罩住 research/results/ 或 replay.sh 时退 2（新产物一存进来指纹就对不上自己）",
                        exit_code == EXIT_REGISTRATION_ERROR and "research/" in messages, f"退 {exit_code}，stderr：{messages.strip()}")
        exit_code, _output, _messages = experiment("E903")
        selftest.expect("认不出的准入条件退 2", exit_code == EXIT_REGISTRATION_ERROR, f"退 {exit_code}")
        problems = lint_registration_rows(read_registration_rows(work))
        selftest.expect("登记表自查点得出认不出的条件", any("bogus=1" in problem for problem in problems), f"自查结果：{problems}")
        write_text(table_path, good_table)

        # ⑦ 登记行本身进指纹：给 E900 多登一条路径（文件集不变也算）⇒ 放行
        write_text(table_path, good_table.replace("research/src/e900.rs research/data/", "research/src/e900.rs research/data/ research/src/"))
        exit_code, _output, _messages = experiment("E900")
        selftest.expect("登记行改了就放行（登记行本身进指纹）", exit_code == 0, f"退 {exit_code}")
        write_text(table_path, good_table)

        # ⑧ 给门禁 doc-experiments 的 evidence-in-repo 格的状态：matched / stale / unfingerprinted / unregistered
        status, _detail = stored_product_status(work, "E900")
        selftest.expect("产物指纹对得上时状态是 matched", status == "matched", f"实际 {status}")
        write_text(os.path.join(work, "research/src/e900.rs"), "fn main() { let changed = 1; }\n")
        status, _detail = stored_product_status(work, "E900")
        selftest.expect("输入变了时状态是 stale", status == "stale", f"实际 {status}")
        status, _detail = stored_product_status(work, "E900/second")
        selftest.expect("没有一份带这个键指纹的产物时状态是 unfingerprinted", status == "unfingerprinted", f"实际 {status}")
        status, _detail = stored_product_status(work, "E904")
        selftest.expect("没登记的键状态是 unregistered", status == "unregistered", f"实际 {status}")

        # ⑨ manifest 的清单写法：拿 sha256sum、sort -zu 与 cargo -V 另算一遍，逐字节比（crash-case-manifest 写清单用的是同一个 manifest_of_files）
        independent = subprocess.run(["bash", "-c", INDEPENDENT_MANIFEST_SCRIPT, "_", work, os.path.join(work, REGISTRATION_TABLE)],
                                     capture_output=True)
        manifest_file = os.path.join(work, "manifest.out")
        exit_code, output, messages = run_quietly([sys.executable, module, "manifest", work, "59-demo.sh", manifest_file,
                                                   "--extra-file", "<判它的 54 号：54-layer0-replay.sh>", table_path, "--toolchain"])
        with open(manifest_file, "rb") as handle:
            module_manifest = handle.read()
        selftest.expect("清单与另算的一遍（sha256sum、sort -zu、cargo -V && rustc -V）逐字节相同",
                        independent.returncode == 0 and exit_code == 0 and module_manifest == independent.stdout
                        and output.split()[0] == hashlib.sha256(independent.stdout).hexdigest(),
                        f"另算退 {independent.returncode}，模块退 {exit_code}，{messages.strip()}")
        os.remove(manifest_file)

        # ⑩ 门禁复用判定的进程带着异常退 1 时，stage-must-run.sh 按要跑处理（退 0），不许把 1 当成「可跳过」
        exit_code, output, _messages = run_quietly(["bash", os.path.join(SELFTEST_HERE, "stage-must-run.sh"), work, "59-demo.sh"],
                                                   {BREAK_VARIABLE: "raise-in-gate-reuse", "SINGLEFS_STAGED_TREE": "0" * 40})
        selftest.expect("门禁复用判定出异常时 stage-must-run.sh 判要跑", exit_code == 0 and "按要跑处理" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        # ⑩b checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay 的全绿标记（用户 2026-09-27 定：照 54 号写标记、只跑变了的）
        run_stage_marker_cells(selftest, module)

        # ⑪ 门禁行的第三列：前提（command= / readwrite= / probe=）、环境进复用判定（environment=，拿假 herd7 当桩）、中文路径
        run_gate_precondition_cells(selftest, module)
        run_gate_environment_cells(selftest, module)
        # ⑫ replay.sh 接准入的那几处：输入没变不跑、前提没齐拒、比对前删产物头、装置没打指纹行
        run_replay_cells(selftest, module)
        # ⑭ 构建环境与指向目录的符号链接进指纹（层 0 规模第三轮判决 U4）：每一样改了，指纹必须变
        run_build_environment_cells(selftest, module)
        # ⑮ 崩溃枚举用例：去注释找点名、减去别的测试目标独占的文件、登记行自查、日志怎么判、全绿标记读写
        run_rust_comment_cells(selftest)
        run_crash_case_input_cells(selftest, module)
        run_crash_case_log_cells(selftest)
        run_judging_digest_cells(selftest, module)
        run_thread_source_cells(selftest, module)
        # ⑯ 门禁 54 号这一份的流程：拷进临时仓（不在 .claude/gate.d/ 下）、拿打合成日志的假 cargo 跑
        run_layer0_stage_cells(selftest, module)

        # ⑬ 真仓的登记表自己写对没有（语法：实验键、条件的种类分门禁与实验、@ 引用、probe= / environment= 指的脚本在不在）
        repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
        real_problems = lint_registration_rows(read_registration_rows(repository_root), repository_root)
        selftest.expect(f"真仓的登记表 {REGISTRATION_TABLE} 自查没有问题", not real_problems, "；".join(real_problems))
        run_real_crash_case_row_cells(selftest, read_registration_rows(repository_root))
    finally:
        shutil.rmtree(work, ignore_errors=True)

    if selftest.failures:
        print(f"  ✗ admission.py 自证没过：{selftest.failures} 格判错（共 {selftest.checked} 格）")
        print("     → 怎么办：照上面每一格的说明改被测的那一段；判据与为什么这么定见文件头。弄坏开关那几格红不了，说明那一格分不出差别")
        return 1
    print(f"  ✓ admission.py 自证通过：{selftest.checked} 格都对（含弄坏开关 skip-unchanged、skip-preconditions、skip-gate-preconditions、"
          "ignore-gate-environment、skip-build-environment、ignore-linked-directories、exclude-mentioned-test-targets、threads-by-worker-count、"
          "first-definition-only、ignore-attribute-without-space、concat-include-only、computed-include-subtracts-non-test-files、runner-program-only、"
          "whole-module-in-judging-digest、judging-digest-without-dispatch、single-worker-threads-field、threads-ignore-shards、"
          "shardable-outside-judging-digest、shard-driver-outside-manifest、keep-caller-shard-switch、target-own-files-only、include-alias-subtracts、"
          "runner-arguments-from-configuration-directory-only、digest-skips-other-statements、digest-allows-non-name-dispatch、digest-allows-imported-names、"
          "judge-takes-forwarded-threads、cores-from-nproc、thread-variable-ignored、threads-skip-self-contained-finish-line、stage-marker-ignored "
          "下各自那一格转红，弄坏 replay.sh 比对前删产物头那一步判对不上）")
    return 0


GIT_IDENTITY = {"GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@t", "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@t"}


def write_executable(path, text):
    write_text(path, text)
    os.chmod(path, 0o755)


def staged_tree_of(work):
    run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
    _exit_code, output, _messages = run_quietly(["git", "-C", work, "write-tree"])
    return output.strip()


def mark_tree_green(work, tree):
    """照 research/scripts/gate-staged.sh 的做法：refs/sop/staged-green 指一个包着那棵暂存树的提交。"""
    _exit_code, commit, _messages = run_quietly(["git", "-C", work, "commit-tree", tree, "-m", "green"], GIT_IDENTITY)
    run_quietly(["git", "-C", work, "update-ref", GATE_REUSE_REFERENCE, commit.strip()])


def run_gate_precondition_cells(selftest, module):
    """门禁行的前提：齐了一个字不打；command= / readwrite= / probe= 各缺一样判红并点名；没登记退 2；种类写串了自查点得出。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-gate-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        present_tool = os.path.join(tools, "selftest-present-tool")
        write_executable(present_tool, "#!/usr/bin/env bash\nexit 0\n")
        device = os.path.join(work, "device.bin")
        kernel = os.path.join(work, "kernel.img")
        write_text(device, "")
        write_text(kernel, "")
        write_text(os.path.join(work, "tools/check.sh"),
                   'if [[ "${1:-}" == --check && -f kernel.img ]]; then exit 0; fi\necho "  ✗ 找不到内核镜像（样本）"\nexit 1\n')
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "55-demo.sh\tcrates/\tcommand=selftest-present-tool readwrite=device.bin probe=tools/check.sh:--check\t# 样本：三个前置\n")

        def preconditions(stage="55-demo.sh", extra_environment=None):
            changes = {"PATH": tools + os.pathsep + os.environ.get("PATH", "")}
            changes.update(extra_environment or {})
            return run_quietly([sys.executable, module, "gate-preconditions", work, stage], changes)

        exit_code, output, messages = preconditions()
        selftest.expect("门禁前提都齐时退 0、stdout 一个字不打", exit_code == 0 and output == "",
                        f"退 {exit_code}，stdout「{output.strip()}」，stderr：{messages.strip()}")
        os.remove(present_tool)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 command= 缺了判红（退 1）：点名那个可执行名，带出路与那一行的注释",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "✗ 前提没齐" in output and "selftest-present-tool 不在 PATH 里" in output
                        and "→ 怎么办" in output and "样本：三个前置" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, _output, _messages = preconditions(extra_environment={BREAK_VARIABLE: "skip-gate-preconditions"})
        selftest.expect("弄坏开关 skip-gate-preconditions 下「前提没齐」那一格红（放行了）", exit_code == 0,
                        f"弄坏之后仍退 {exit_code}：这一格判不出前提被跳过")
        write_executable(present_tool, "#!/usr/bin/env bash\nexit 0\n")
        os.chmod(device, 0o444)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 readwrite= 不可写判红（退 1）并点名那个路径（以 root 跑这一格判不出）",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "device.bin 不在，或当前用户不可读写" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        os.chmod(device, 0o644)
        os.remove(kernel)
        exit_code, output, _messages = preconditions()
        selftest.expect("门禁前提 probe= 退非 0 判红（退 1），明细带出探针自己打的那几行",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "tools/check.sh --check 退 1" in output and "找不到内核镜像（样本）" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = preconditions(stage="99-unregistered.sh")
        selftest.expect("登记表里没有那一道时退 2，照样打 ✗ 与出路", exit_code == EXIT_REGISTRATION_ERROR and "✗" in output and "→ 怎么办" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        mixed_rows = [RegistrationRow("55-demo.sh", ["crates/"], ["question-row=a.md#1:x"], 1, ""),
                      RegistrationRow("E950", ["crates/"], ["command=cargo"], 2, ""),
                      RegistrationRow("57-demo.sh", ["crates/"], ["probe=tools/nowhere.sh"], 3, "")]
        problems = lint_registration_rows(mixed_rows, work)
        selftest.expect("自查点得出门禁行写实验的种类、实验行写门禁的种类、probe= 指的脚本不在",
                        any(problem.startswith("第 1 行") and "认不出的门禁准入条件" in problem for problem in problems)
                        and any(problem.startswith("第 2 行") and "认不出的准入条件" in problem for problem in problems)
                        and any(problem.startswith("第 3 行") and "tools/nowhere.sh 不在" in problem for problem in problems),
                        f"自查结果：{problems}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)


FAKE_HERD7_SCRIPT = '#!/usr/bin/env bash\nif [[ "${1:-}" == -version ]]; then echo "herd7 version 7.57-selftest-%s"; exit 0; fi\nexit 2\n'


def run_gate_environment_cells(selftest, module):
    """门禁的环境进复用判定（照 checker-tier-lkmm：herd7 的版本不在 git 树里）：拿 PATH 前面的假 herd7 当桩，经 stage-must-run.sh 判。
    同一个仓里再核两道没登记环境的阶段在中文路径上判得对、依据句里是原样的中文路径。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-environment-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        fake_herd7 = os.path.join(tools, "herd7")
        write_executable(fake_herd7, FAKE_HERD7_SCRIPT % "1")
        write_text(os.path.join(work, "tools/herd7-version.sh"),
                   'command -v herd7 >/dev/null 2>&1 || { echo "  ✗ herd7 缺失（样本）"; exit 1; }\nherd7 -version 2>&1 | head -n 1\n')
        write_text(os.path.join(work, "litmus/a.litmus"), "C a\n")
        write_text(os.path.join(work, "docs/中文说明.md"), "一\n")
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "57-demo.sh\tlitmus/\tprobe=tools/herd7-version.sh environment=tools/herd7-version.sh\t# 样本：herd7 版本进复用判定\n"
                   "59-demo.sh\tdocs/\t# 样本：登记的目录底下有中文文件名\n"
                   "74-demo.sh\tdocs/中文说明.md\t# 样本：登记的路径本身是中文\n")
        base_tree = staged_tree_of(work)
        mark_tree_green(work, base_tree)
        with_tools = {"PATH": tools + os.pathsep + os.environ.get("PATH", "")}
        record_path = os.path.join(work, ".git", GATE_ENVIRONMENT_RECORD_PREFIX + "57-demo.sh")

        def reuse(stage, tree, extra_environment=None):
            changes = dict(with_tools, SINGLEFS_STAGED_TREE=tree)
            changes.update(extra_environment or {})
            return run_quietly(["bash", os.path.join(SELFTEST_HERE, "stage-must-run.sh"), work, stage], changes)

        def record(environment_changes):
            return run_quietly([sys.executable, module, "gate-record-environment", work, "57-demo.sh"], environment_changes)

        exit_code, output, _messages = reuse("57-demo.sh", base_tree)
        selftest.expect("登记了 environment= 的门禁，输入相同而环境没记下时判要跑", exit_code == 0 and "没记下" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = record(with_tools)
        selftest.expect("不是 gate-staged.sh 起的（没有 SINGLEFS_STAGED_TREE）不记环境、一个字不打",
                        exit_code == 0 and output == "" and not os.path.exists(record_path), f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = record(dict(with_tools, SINGLEFS_STAGED_TREE=base_tree))
        selftest.expect("判绿之后记下环境：打一行「·」，记录落在 git common-dir",
                        exit_code == 0 and "记下了" in output and os.path.isfile(record_path), f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = reuse("57-demo.sh", base_tree)
        selftest.expect("输入与环境都没变时判可跳过，依据句带出 herd7 的版本", exit_code == 1 and "7.57-selftest-1" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        write_executable(fake_herd7, FAKE_HERD7_SCRIPT % "2")
        exit_code, output, _messages = reuse("57-demo.sh", base_tree)
        selftest.expect("herd7 的版本变了判要跑，依据句带出记下的与现在的两个版本",
                        exit_code == 0 and "不同" in output and "7.57-selftest-1" in output and "7.57-selftest-2" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, _output, _messages = reuse("57-demo.sh", base_tree, {BREAK_VARIABLE: "ignore-gate-environment"})
        selftest.expect("弄坏开关 ignore-gate-environment 下「版本变了」那一格红（判成可跳过）", exit_code == 1,
                        f"弄坏之后仍退 {exit_code}：这一格判不出环境没比")
        os.remove(fake_herd7)
        exit_code, output, _messages = reuse("57-demo.sh", base_tree)
        selftest.expect("herd7 不在时环境取不出来，判要跑", exit_code == 0 and "取不出来" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = run_quietly([sys.executable, module, "gate-preconditions", work, "57-demo.sh"], with_tools)
        selftest.expect("herd7 不在时 gate-preconditions 判红（退 1），明细带出探针打的那一行",
                        exit_code == EXIT_GATE_PRECONDITION_MISSING and "herd7 缺失（样本）" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        write_executable(fake_herd7, FAKE_HERD7_SCRIPT % "1")
        write_text(os.path.join(work, "litmus/a.litmus"), "C a changed\n")
        changed_tree = staged_tree_of(work)
        mark_tree_green(work, changed_tree)
        exit_code, output, _messages = reuse("57-demo.sh", changed_tree)
        selftest.expect("环境相同、但记下环境的那一趟判的是另一批输入时判要跑", exit_code == 0 and "litmus/a.litmus" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
        record(dict(with_tools, SINGLEFS_STAGED_TREE=changed_tree))
        exit_code, output, _messages = reuse("57-demo.sh", changed_tree)
        selftest.expect("在这一批输入上重新记下环境之后又判可跳过", exit_code == 1, f"退 {exit_code}，stdout「{output.strip()}」")

        exit_code, output, _messages = reuse("59-demo.sh", changed_tree)
        selftest.expect("登记目录底下的中文文件名没变时判可跳过", exit_code == 1, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = reuse("74-demo.sh", changed_tree)
        selftest.expect("登记的路径本身是中文、没变时判可跳过", exit_code == 1, f"退 {exit_code}，stdout「{output.strip()}」")
        write_text(os.path.join(work, "docs/中文说明.md"), "二\n")
        chinese_tree = staged_tree_of(work)
        exit_code, output, _messages = reuse("59-demo.sh", chinese_tree)
        selftest.expect("登记目录底下的中文文件改了判要跑，依据句里是原样的中文路径（不是八进制转义）",
                        exit_code == 0 and "docs/中文说明.md" in output, f"退 {exit_code}，stdout「{output.strip()}」")
        exit_code, output, _messages = reuse("74-demo.sh", chinese_tree)
        selftest.expect("登记的路径本身是中文、改了判要跑", exit_code == 0 and "docs/中文说明.md" in output,
                        f"退 {exit_code}，stdout「{output.strip()}」")
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)


# 拼进 replay.sh 副本的桩驱动：照装置入口的做法先问准入、放行才打产物头与结果；E902 故意不问（装置 main 没接准入的形态）
REPLAY_SELFTEST_DRIVERS = r'''
# ── admission.py --selftest 拼进来的桩驱动（真仓的 replay.sh 没有这一段）──
driver_e900() {
  : >"$OUT_DIR/driver-ran-E900"
  python3 scripts/admission.py experiment .. E900 || return $?
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e901() {
  : >"$OUT_DIR/driver-ran-E901"
  python3 scripts/admission.py experiment .. E901 || return $?
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
driver_e902() {
  : >"$OUT_DIR/driver-ran-E902"
  printf 'E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n'
}
'''
REPLAY_TABLE_START = "TABLE=$(cat <<'TSV'\n"
REPLAY_TABLE_END = "\nTSV\n)\n"
REPLAY_STRIP_HEADER_COMMAND = "sed '/^E7INPUT /d'"


# 临时仓里跑真仓脚本的副本时要从真仓链进来的两样：规范副本与项目的 preflight 垫片（.claude/scripts/preflight.sh）所在的目录
LINKED_INTO_SELFTEST_REPOSITORIES = (".claude/singlefs-ai-sop", ".claude/scripts")


def link_sop_copy(work):
    """临时仓里跑真仓脚本的副本（replay.sh、54 号与它们调的 stage-must-run.sh 这一类）时，副本开头按相对路径 source .claude/scripts/preflight.sh，
    垫片再找规范副本里的 preflight.sh：LINKED_INTO_SELFTEST_REPOSITORIES 那两样在临时仓里建成指向真仓那一份的符号链接（缺了 source 落空、
    preflight 一步没判，随后的 set -- 把参数清空），并写进临时仓的 .gitignore（不算进改动范围）。"""
    os.makedirs(os.path.join(work, ".claude"), exist_ok=True)
    real_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
    with open(os.path.join(work, ".gitignore"), "a", encoding="utf-8") as handle:
        for relative_path in LINKED_INTO_SELFTEST_REPOSITORIES:
            os.symlink(os.path.join(real_root, relative_path), os.path.join(work, relative_path))
            handle.write(relative_path + "\n")


def replay_copy_text(table_rows, break_header_strip):
    """真仓 replay.sh 的副本：只换掉复跑表、在表后面拼上桩驱动；break_header_strip 时把比对前删产物头那一步换成 cat。
    replay.sh 的形状变了（表的起止、删头那一句不是恰好一处）抛 RegistrationError，那一格判错。"""
    with open(os.path.join(SELFTEST_HERE, "replay.sh"), encoding="utf-8") as handle:
        text = handle.read()
    if text.count(REPLAY_TABLE_START) != 1 or text.count(REPLAY_STRIP_HEADER_COMMAND) != 1:
        raise RegistrationError("replay.sh 里复跑表的开头或比对前删产物头那一句不是恰好一处，自证拼不出副本")
    start = text.index(REPLAY_TABLE_START) + len(REPLAY_TABLE_START)
    end = text.index(REPLAY_TABLE_END, start)
    copy = (text[:start] + "\n".join(table_rows) + REPLAY_TABLE_END + REPLAY_SELFTEST_DRIVERS
            + text[end + len(REPLAY_TABLE_END):])
    if break_header_strip:
        copy = copy.replace(REPLAY_STRIP_HEADER_COMMAND, "cat")
    return copy


def run_replay_cells(selftest, module):
    """replay.sh 接准入的那几处，在临时仓里拿真仓 replay.sh 的副本（只换复跑表、加桩驱动）跑：cargo 与 rustc 用桩。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-replay-")
    tools = tempfile.mkdtemp(prefix="admission-selftest-tools-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        write_executable(os.path.join(tools, "cargo"), '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "cargo 0.0.0-selftest"\nexit 0\n')
        write_executable(os.path.join(tools, "rustc"), '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "rustc 0.0.0-selftest"\nexit 0\n')
        os.makedirs(os.path.join(work, "research/scripts"), exist_ok=True)
        shutil.copy(module, os.path.join(work, "research/scripts/admission.py"))
        write_text(os.path.join(work, REGISTRATION_TABLE),
                   "E900\tresearch/src/e900.txt\t# 样本\n"
                   "E901\tresearch/src/e901.txt\tquestion-row=research/questions.md#6:够判\t# 样本：前提没齐\n"
                   "E902\tresearch/src/e902.txt\t# 样本：装置没打指纹行\n")
        stored_body = "E7RESULT name=verdict control_ok=true\nE7RESULT name=done emitted=2\n"
        for number in ("900", "901", "902"):
            write_text(os.path.join(work, f"research/src/e{number}.txt"), f"输入 {number}\n")
            write_text(os.path.join(work, RESULTS_DIRECTORY, f"e{number}-2026-09-01.out"), stored_body)
        write_questions(work, "开着")
        link_sop_copy(work)
        run_quietly(["git", "-C", work, "add", "-A"], GIT_IDENTITY)
        run_quietly(["git", "-C", work, "commit", "-qm", "base"], GIT_IDENTITY)
        table_rows = [f"E{number}|@driver_e{number}||e{number}-2026-09-01.out|exact" for number in ("900", "901", "902")]

        def replay(experiment, rows, break_header_strip=False):
            write_text(os.path.join(work, REPLAY_SCRIPT), replay_copy_text(rows, break_header_strip))
            output_directory = tempfile.mkdtemp(prefix="out-", dir=work)
            exit_code, output, messages = run_quietly(
                ["bash", os.path.join(work, REPLAY_SCRIPT), experiment],
                {"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "REPLAY_OUT": output_directory, "REPLAY_JOBS": "1"})
            return exit_code, output + messages, output_directory

        exit_code, output, first_run = replay("E900", table_rows)
        selftest.expect("replay.sh：老产物没有产物头、新跑的有，比对前删掉产物头之后判字节一致",
                        exit_code == 0 and re.search(r"^E900 .*字节一致", output, re.M) is not None
                        and os.path.exists(os.path.join(first_run, "driver-ran-E900")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, _run = replay("E900", table_rows, break_header_strip=True)
        selftest.expect("弄坏 replay.sh 比对前删产物头那一步（换成 cat）判对不上：上一格分得出差别",
                        exit_code == 1 and re.search(r"^E900 .*对不上", output, re.M) is not None, f"退 {exit_code}，输出：{output.strip()[-600:]}")
        shutil.copy(os.path.join(first_run, "E900.1.out"), os.path.join(work, RESULTS_DIRECTORY, "e900-2026-09-02.out"))
        rows_with_fingerprinted_product = [row.replace("e900-2026-09-01.out", "e900-2026-09-02.out") for row in table_rows]
        exit_code, output, unchanged_run = replay("E900", rows_with_fingerprinted_product)
        selftest.expect("replay.sh：留存产物头上的指纹与今天的输入相同时判「输入没变」、装置一次都没跑、汇总报没跑 1 行",
                        exit_code == 0 and re.search(r"^E900 .*输入没变", output, re.M) is not None and "输入没变没跑 1" in output
                        and not os.path.exists(os.path.join(unchanged_run, "driver-ran-E900")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, refused_run = replay("E901", table_rows)
        selftest.expect("replay.sh：前提没齐时判跑不了（准入退 3）、装置一次都没跑",
                        exit_code == 1 and re.search(r"^E901 .*跑不了 .*准入没放行（退 3）", output, re.M) is not None
                        and not os.path.exists(os.path.join(refused_run, "driver-ran-E901")), f"退 {exit_code}，输出：{output.strip()[-600:]}")
        exit_code, output, _run = replay("E902", table_rows)
        selftest.expect("replay.sh：登记了准入、装置却没打指纹行时判跑不了",
                        exit_code == 1 and "产物头却没有 key=E902 的输入指纹行" in output, f"退 {exit_code}，输出：{output.strip()[-600:]}")
    except RegistrationError as error:
        selftest.expect("replay.sh 的副本拼得出来", False, str(error))
    finally:
        shutil.rmtree(work, ignore_errors=True)
        shutil.rmtree(tools, ignore_errors=True)


def environment_without_build_settings(changes=None):
    """子进程的环境：去掉会进构建环境那一格的变量（CARGO_HOME、RUSTC 与名字对得上的那几类）与握手用的几个，再套上 changes。"""
    environment = {name: value for name, value in os.environ.items()
                   if not (name in BUILD_ENVIRONMENT_EXACT_VARIABLES or name in ("RUSTC", "CARGO_HOME")
                           or any(form.match(name) for form in BUILD_ENVIRONMENT_VARIABLE_FORMS))}
    for variable in (FORCE_RERUN_VARIABLE, "SINGLEFS_GATE_FULL", "SINGLEFS_STAGED_TREE", "SINGLEFS_REUSE_HOURS"):
        environment.pop(variable, None)
    environment.update(changes or {})
    return environment


def run_in_environment(command, environment, working_directory=None):
    completed = subprocess.run(command, capture_output=True, text=True, errors="replace", env=environment, cwd=working_directory)
    return completed.returncode, completed.stdout, completed.stderr


class BreakSwitch:
    """在本进程里临时设 ADMISSION_BREAK（给纯函数那几格证明会红），退出时照原样还回去。"""
    def __init__(self, switch_name):
        self.switch_name = switch_name
        self.saved = None

    def __enter__(self):
        self.saved = os.environ.get(BREAK_VARIABLE)
        os.environ[BREAK_VARIABLE] = self.switch_name
        return self

    def __exit__(self, *_exception):
        if self.saved is None:
            os.environ.pop(BREAK_VARIABLE, None)
        else:
            os.environ[BREAK_VARIABLE] = self.saved
        return False


FAKE_TOOLCHAIN_SCRIPTS = {
    "cargo": '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "cargo 0.0.0-selftest"\nexit 0\n',
    "rustc": '#!/usr/bin/env bash\n[[ "${1:-}" == -V ]] && echo "rustc 0.0.0-selftest"\nexit 0\n',
}


def run_build_environment_cells(selftest, module):
    """构建环境与指向目录的符号链接进指纹：仓根往上一层与仓根自己的 cargo 配置、CARGO_HOME 下的配置、rust-toolchain 两种、
    环境变量那几类、RUSTC 的 -V、登记路径下指向目录的符号链接，每一样改了指纹必须变、改回去又相同；CARGO_BUILD_JOBS 改了指纹不变。"""
    base = tempfile.mkdtemp(prefix="admission-selftest-build-")
    try:
        outer = os.path.join(base, "outer")
        work = os.path.join(outer, "repo")
        cargo_home = os.path.join(base, "cargo-home")
        tools = os.path.join(base, "tools")
        outside = os.path.join(base, "outside")
        os.makedirs(cargo_home)
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        write_text(os.path.join(work, REGISTRATION_TABLE), "59-demo.sh\tcrates/ Cargo.toml\t# 样本：门禁行\nE900\tcrates/\t# 样本：实验\n")
        write_text(os.path.join(work, "crates/demo/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
        write_text(os.path.join(work, "Cargo.toml"), "[workspace]\n")
        write_text(os.path.join(outside, "shared/shared.rs"), "pub fn shared() {}\n")
        os.symlink(os.path.join(outside, "shared"), os.path.join(work, "crates/linked"))
        for name, script in FAKE_TOOLCHAIN_SCRIPTS.items():
            write_executable(os.path.join(tools, name), script)
        version_file = os.path.join(base, "switchable-version")
        write_text(version_file, "rustc 9.9.9-a\n")
        switchable_rustc = os.path.join(tools, "switchable-rustc")
        write_executable(switchable_rustc, f'#!/usr/bin/env bash\ncat "{version_file}"\n')
        environment = environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""), "CARGO_HOME": cargo_home})
        manifest_file = os.path.join(base, "manifest.out")

        def fingerprint(changes=None):
            exit_code, output, messages = run_in_environment(
                [sys.executable, module, "manifest", work, "59-demo.sh", manifest_file, "--toolchain", "--build-environment"],
                dict(environment, **(changes or {})))
            return output.split()[0] if exit_code == 0 and output.split() else f"退 {exit_code}：{messages.strip()}"

        baseline = fingerprint()
        file_cells = [
            ("仓根往上一层的 .cargo/config.toml", os.path.join(outer, ".cargo/config.toml")),
            ("仓根往上一层的 .cargo/config", os.path.join(outer, ".cargo/config")),
            ("仓根自己的 .cargo/config.toml", os.path.join(work, ".cargo/config.toml")),
            ("CARGO_HOME 下的 config.toml", os.path.join(cargo_home, "config.toml")),
            ("CARGO_HOME 下的 config", os.path.join(cargo_home, "config")),
            ("仓根往上一层的 rust-toolchain", os.path.join(outer, "rust-toolchain")),
            ("仓根往上一层的 rust-toolchain.toml", os.path.join(outer, "rust-toolchain.toml")),
        ]
        for label, path in file_cells:
            write_text(path, '[build]\nrustflags = ["-C", "overflow-checks=off"]\n')
            changed = fingerprint()
            os.remove(path)
            restored = fingerprint()
            selftest.expect(f"构建环境：加一份{label}，指纹变；删掉又回到原样", changed != baseline and restored == baseline,
                            f"原样 {baseline[:16]}，加了之后 {changed[:16]}，删掉之后 {restored[:16]}")
        variable_cells = ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS", "CARGO_BUILD_RUSTFLAGS",
                          "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
                          "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]
        for variable in variable_cells:
            changed = fingerprint({variable: "selftest-value"})
            selftest.expect(f"构建环境：设了环境变量 {variable}，指纹变", changed != baseline, f"原样与设了之后都是 {baseline[:16]}")
        left_out = fingerprint({"CARGO_BUILD_JOBS": "7"})
        selftest.expect("构建环境：只设 CARGO_BUILD_JOBS（编译并行度，包装会设它），指纹不变", left_out == baseline,
                        f"原样 {baseline[:16]}，设了之后 {left_out[:16]}")
        with_rustc_a = fingerprint({"RUSTC": switchable_rustc})
        write_text(version_file, "rustc 9.9.9-b\n")
        with_rustc_b = fingerprint({"RUSTC": switchable_rustc})
        selftest.expect("构建环境：RUSTC 设了指纹变；同一个 RUSTC 的 -V 变了，指纹也变",
                        with_rustc_a != baseline and with_rustc_b != with_rustc_a,
                        f"原样 {baseline[:16]}，RUSTC 报 a {with_rustc_a[:16]}，报 b {with_rustc_b[:16]}")
        linked_file = os.path.join(outside, "shared/shared.rs")
        write_text(linked_file, "pub fn shared() { let changed = 1; }\n")
        through_link = fingerprint()
        before_break = fingerprint({BREAK_VARIABLE: "ignore-linked-directories"})
        write_text(linked_file, "pub fn shared() {}\n")
        after_break = fingerprint({BREAK_VARIABLE: "ignore-linked-directories"})
        selftest.expect("登记路径下指向目录的符号链接：链接指向的目录里的文件改了，指纹变", through_link != baseline and fingerprint() == baseline,
                        f"原样 {baseline[:16]}，改了链接那头之后 {through_link[:16]}")
        selftest.expect("弄坏开关 ignore-linked-directories 下「链接那头改了」那一格红（指纹不变）", before_break == after_break,
                        f"弄坏之后仍然一变一不变：{before_break[:16]} → {after_break[:16]}")
        config_path = os.path.join(outer, ".cargo/config.toml")
        broken_before = fingerprint({BREAK_VARIABLE: "skip-build-environment"})
        write_text(config_path, '[build]\nrustflags = ["-C", "overflow-checks=off"]\n')
        broken_after = fingerprint({BREAK_VARIABLE: "skip-build-environment"})
        os.remove(config_path)
        selftest.expect("弄坏开关 skip-build-environment 下「上一层的 .cargo/config.toml」那一格红（指纹不变）", broken_before == broken_after,
                        f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
        # runner、wrapper 指的程序换了内容而值不变、配置文件里 build.rustc 的 -V 变了而配置原文不变：指纹都要变
        program = os.path.join(tools, "selftest-runner")
        write_executable(program, '#!/usr/bin/env bash\nexec "$@"\n')

        def program_edit_fingerprints(changes=None):
            before = fingerprint(changes)
            write_executable(program, '#!/usr/bin/env bash\nexec "$@" --skip every_crash_state\n')
            after = fingerprint(changes)
            write_executable(program, '#!/usr/bin/env bash\nexec "$@"\n')
            return before, after
        for label, changes in [("环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER（值带参数）",
                                {"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": program + " --flag"}),
                               ("环境变量 RUSTC_WRAPPER", {"RUSTC_WRAPPER": program}),
                               ("环境变量 CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER（不带斜杠、在 PATH 里找）", {"CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER": "selftest-runner"}),
                               ("环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=bash <脚本>（首词是解释器，脚本在参数里）",
                                {"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": "bash " + program})]:
            before, after = program_edit_fingerprints(changes)
            selftest.expect(f"构建环境：{label}指的程序换了内容、值不变，指纹变", before != after and not before.startswith("退"),
                            f"换之前 {before[:16]}，换之后 {after[:16]}")
        broken_before, broken_after = program_edit_fingerprints({"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": "bash " + program,
                                                                 BREAK_VARIABLE: "runner-program-only"})
        selftest.expect("弄坏开关 runner-program-only 下「runner = bash <脚本>，脚本换了内容」那一格红（指纹不变）", broken_before == broken_after,
                        f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
        repository_configuration = os.path.join(work, ".cargo/config.toml")
        for label, configuration_text in [
                ("仓根 .cargo/config.toml 里 target.<三元组>.runner", f'[target.x86_64-unknown-linux-gnu]\nrunner = ["{program}", "--flag"]\n'),
                ("仓根 .cargo/config.toml 里 target.<三元组>.runner = [\"bash\", <从 .cargo 那一层起的相对路径>]",
                 f'[target.x86_64-unknown-linux-gnu]\nrunner = ["bash", "{os.path.relpath(program, work)}"]\n'),
                ("仓根 .cargo/config.toml 里 build.rustc-wrapper（从 .cargo 所在的那一层起的相对路径）",
                 f'[build]\nrustc-wrapper = "{os.path.relpath(program, work)}"\n')]:
            write_text(repository_configuration, configuration_text)
            before, after = program_edit_fingerprints()
            os.remove(repository_configuration)
            selftest.expect(f"构建环境：{label}指的程序换了内容、配置原文不变，指纹变", before != after and not before.startswith("退"),
                            f"换之前 {before[:16]}，换之后 {after[:16]}")
        write_text(repository_configuration, f'[build]\nrustc = "{switchable_rustc}"\n')
        write_text(version_file, "rustc 9.9.9-c\n")
        with_version_c = fingerprint()
        write_text(version_file, "rustc 9.9.9-d\n")
        with_version_d = fingerprint()
        os.remove(repository_configuration)
        selftest.expect("构建环境：仓根 .cargo/config.toml 里 build.rustc 指的编译器 -V 变了、配置原文不变，指纹变",
                        with_version_c != with_version_d and not with_version_c.startswith("退"), f"报 c {with_version_c[:16]}，报 d {with_version_d[:16]}")
        selftest.expect("构建环境：配置文件与环境变量都拿掉之后，指纹回到原样", fingerprint() == baseline, f"原样 {baseline[:16]}")

        def experiment_fingerprint_line(changes=None):
            _exit_code, output, _messages = run_in_environment([sys.executable, module, "experiment", work, "E900"],
                                                               dict(environment, **(changes or {})))
            return output.strip().split("\n")[0]
        plain = experiment_fingerprint_line()
        with_flags = experiment_fingerprint_line({"RUSTFLAGS": "-C overflow-checks=off"})
        selftest.expect("实验的输入指纹同样带构建环境：设了 RUSTFLAGS，产物头的指纹变",
                        plain.startswith("E7INPUT name=input_fingerprint key=E900 ") and with_flags.startswith("E7INPUT ") and plain != with_flags,
                        f"没设「{plain}」，设了「{with_flags}」")
        # runner 参数里的相对路径另从登记用例的包目录解（cargo 在包目录起 runner：`sh ../../tools/r.sh` 从 crates/demo 起才解得到仓根的 tools/r.sh）
        write_text(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/demo"]\n')
        write_text(os.path.join(work, "crates/demo/Cargo.toml"), '[package]\nname = "demo"\nversion = "0.0.0"\n')
        write_text(os.path.join(work, REGISTRATION_TABLE), "59-demo.sh\tcrates/ Cargo.toml\t# 样本：门禁行\nE900\tcrates/\t# 样本：实验\n"
                                                           "crash-case:demo\tcrates/ Cargo.toml\ttest=demo:demo_case:the_case\t# 样本：崩溃枚举用例\n")
        runner_script = os.path.join(work, "tools/r.sh")
        for label, changes, configuration_text in [
                ("仓根 .cargo/config.toml 里 runner = \"sh ../../tools/r.sh\"（从包目录起才解得到）", {},
                 '[target.x86_64-unknown-linux-gnu]\nrunner = "sh ../../tools/r.sh"\n'),
                ("环境变量 CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER=\"sh ../../tools/r.sh\"",
                 {"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER": "sh ../../tools/r.sh"}, None)]:
            if configuration_text:
                write_text(repository_configuration, configuration_text)
            outcomes = []
            for switch_changes in ({}, {BREAK_VARIABLE: "runner-arguments-from-configuration-directory-only"}):
                write_text(runner_script, '#!/bin/sh\nexec "$@"\n')
                before = fingerprint(dict(changes, **switch_changes))
                write_text(runner_script, '#!/bin/sh\nexec "$@" --include-ignored\n')
                after = fingerprint(dict(changes, **switch_changes))
                outcomes.append((before, after))
            if configuration_text:
                os.remove(repository_configuration)
            (before, after), (broken_before, broken_after) = outcomes
            selftest.expect(f"构建环境：{label}，改 tools/r.sh 指纹变", before != after and not before.startswith("退"),
                            f"改之前 {before[:16]}，改之后 {after[:16]}")
            selftest.expect(f"弄坏开关 runner-arguments-from-configuration-directory-only 下「{label}」那一格红（指纹不变）",
                            broken_before == broken_after, f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
    finally:
        shutil.rmtree(base, ignore_errors=True)


RUST_COMMENT_CELLS = [
    ("行注释里提到不算", "// helper_x\nfn a() {}\n", False),
    ("可嵌套的块注释里提到不算", "/* outer /* inner helper_x */ still helper_x */ fn a() {}", False),
    ("文档注释里提到不算", "/// 见 helper_x\n//! helper_x\nfn a() {}", False),
    ("mod 声明算", "mod helper_x;\n", True),
    ("字符串里的路径算", 'const P: &str = "tests/helper_x.rs";', True),
    ("字符串里带 // 之后的代码照样算", 'let u = "a//b"; mod helper_x;', True),
    ("原始字符串里带引号之后的代码照样算", 'let r = r#"a"b"#; mod helper_x;', True),
    ("字符字面量是双引号之后的代码照样算", "let q = '\"'; mod helper_x;", True),
    ("生命周期之后的代码照样算", "fn f<'a>(x: &'a str) -> &'a str { x } mod helper_x;", True),
    ("转义的单引号字符之后的代码照样算", "let c = '\\''; mod helper_x;", True),
    ("更长的名字里含它不算（按整词）", "mod helper_x_more;", False),
]


def run_rust_comment_cells(selftest):
    """判「别处有没有代码点名一个测试目标」的那一步：注释里提到不算，代码与字符串里算，字符串、字符字面量、生命周期不把后面的代码吞掉。"""
    for label, code, expected in RUST_COMMENT_CELLS:
        found = bool(whole_word_form("helper_x").search(rust_code_without_comments(code)))
        selftest.expect(f"去注释找点名：{label}", found == expected, f"应当 {'找到' if expected else '找不到'}，实际 {'找到' if found else '找不到'}")


def build_crash_case_repository(work, registration_rows):
    """一个临时小仓：包 pkg 里有用例 own_case（mod common、mod helper_target）、被注释提到的 other_target、被字符串点名的 string_named。"""
    run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
    write_text(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
    write_text(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\n')
    write_text(os.path.join(work, "crates/pkg/src/lib.rs"),
               '//! 注释里提到 other_target 不算\npub const PATH: &str = "tests/string_named.rs";\npub fn one() -> u32 { 1 }\n')
    write_text(os.path.join(work, "crates/pkg/tests/own_case.rs"),
               'mod common;\nmod helper_target;\n#[test]\n#[ignore = "崩溃枚举（样本）"]\nfn the_case() {}\n')
    write_text(os.path.join(work, "crates/pkg/tests/common/mod.rs"), "pub fn shared() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/other_target.rs"), "#[test]\nfn other() {}\n")
    write_text(os.path.join(work, "crates/pkg/tests/string_named.rs"), "#[test]\nfn named() {}\n")
    write_text(os.path.join(work, REGISTRATION_TABLE), "".join(row + "\n" for row in registration_rows))


def run_crash_case_input_cells(selftest, module):
    """崩溃枚举用例的输入：减去的恰好是别的测试目标独占、没被代码点名的文件；改它指纹不变，改被点名的、共用的、登记行、新加的共用文件指纹变。
    另核登记行自查与全绿标记的读写。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-crash-case-")
    try:
        own_row = "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 exhaustive=LAYER0\t# 样本"
        build_crash_case_repository(work, [own_row])
        tools = os.path.join(work, ".tools")
        for name, script in FAKE_TOOLCHAIN_SCRIPTS.items():
            write_executable(os.path.join(tools, name), script)
        write_text(os.path.join(work, ".gitignore"), ".tools/\n*.out\n")
        environment = environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""),
                                                          "CARGO_HOME": os.path.join(work, ".tools/cargo-home")})
        manifest_file = os.path.join(work, "manifest.out")

        def admission(*arguments, changes=None):
            return run_in_environment([sys.executable, module, *arguments], dict(environment, **(changes or {})))

        def fingerprint(changes=None):
            exit_code, output, _messages = admission("crash-case-manifest", work, "crash-case:own", manifest_file, "--toolchain",
                                                     "--build-environment", changes=changes)
            parts = output.split()
            return (parts[0], int(parts[2])) if exit_code == 0 and len(parts) == 3 else (f"退 {exit_code}：{output.strip()}", -1)

        exit_code, output, _messages = admission("crash-cases", work)
        selftest.expect("crash-cases 逐条打「键、包、测试目标、用例函数」", exit_code == 0 and output == "crash-case:own\tpkg\town_case\tthe_case\n",
                        f"退 {exit_code}，stdout「{output.strip()}」")
        baseline, excluded_count = fingerprint()
        with open(manifest_file, encoding="utf-8") as handle:
            manifest_text = handle.read()
        kept_names = sorted(line[66:] for line in manifest_text.split("\n") if len(line) > 66 and not line[66:].startswith("<"))
        selftest.expect("减去的恰好是没被代码点名的 other_target.rs（注释里提到不算）；被 mod 点名的、被字符串点名的、共用的 common/ 与用例自己都留着",
                        excluded_count == 1 and "crates/pkg/tests/other_target.rs" not in kept_names
                        and all(name in kept_names for name in ("crates/pkg/tests/own_case.rs", "crates/pkg/tests/helper_target.rs",
                                                                "crates/pkg/tests/string_named.rs", "crates/pkg/tests/common/mod.rs")),
                        f"减去 {excluded_count} 个，留下的是 {kept_names}")
        changes = [
            ("改别的测试目标独占的 other_target.rs", "crates/pkg/tests/other_target.rs", "#[test]\nfn other() { assert!(true); }\n", False),
            ("新加一个没人点名的测试目标 new_unrelated.rs", "crates/pkg/tests/new_unrelated.rs", "#[test]\nfn new() {}\n", False),
            ("改被用例 mod 点名的 helper_target.rs", "crates/pkg/tests/helper_target.rs", "pub fn helper() { let changed = 1; }\n", True),
            ("改被 lib.rs 里的字符串点名的 string_named.rs", "crates/pkg/tests/string_named.rs", "#[test]\nfn named() { let changed = 1; }\n", True),
            ("改共用的 tests/common/mod.rs", "crates/pkg/tests/common/mod.rs", "pub fn shared() { let changed = 1; }\n", True),
            ("新加一份与 common/mod.rs 并存的 tests/common.rs（用例写着 mod common）", "crates/pkg/tests/common.rs", "pub fn clash() {}\n", True),
            ("改 src/lib.rs", "crates/pkg/src/lib.rs", "//! 注释里提到 other_target 不算\npub const PATH: &str = \"tests/string_named.rs\";\npub fn one() -> u32 { 2 }\n", True),
        ]
        for label, relative, text, should_change in changes:
            path = os.path.join(work, relative)
            original = open(path, encoding="utf-8").read() if os.path.exists(path) else None
            write_text(path, text)
            changed, _count = fingerprint()
            if original is None:
                os.remove(path)
            else:
                write_text(path, original)
            restored, _count = fingerprint()
            selftest.expect(f"崩溃枚举用例的输入：{label}，指纹{'变' if should_change else '不变'}",
                            (changed != baseline) == should_change and restored == baseline,
                            f"原样 {baseline[:16]}，改了之后 {changed[:16]}，改回之后 {restored[:16]}")
        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() { let changed = 1; }\n")
        broken_changed, _count = fingerprint({BREAK_VARIABLE: "exclude-mentioned-test-targets"})
        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
        broken_restored, _count = fingerprint({BREAK_VARIABLE: "exclude-mentioned-test-targets"})
        selftest.expect("弄坏开关 exclude-mentioned-test-targets 下「改被 mod 点名的 helper_target.rs」那一格红（指纹不变）",
                        broken_changed == broken_restored, f"弄坏之后仍然一变一不变：{broken_changed[:16]} → {broken_restored[:16]}")
        build_script = os.path.join(work, "crates/pkg/build.rs")
        write_text(build_script, "fn main() {}\n")
        with_build_script, excluded_with_build_script = fingerprint()
        os.remove(build_script)
        selftest.expect("包里有 build.rs 时这个包的测试文件一份都不减（指纹变、减去 0 个）",
                        with_build_script != baseline and excluded_with_build_script == 0, f"减去 {excluded_with_build_script} 个")
        table_path = os.path.join(work, REGISTRATION_TABLE)
        write_text(table_path, own_row.replace("exhaustive=LAYER0", "exhaustive=LAYER0 threads=LAYER0") + "\n")
        with_threads, _count = fingerprint()
        write_text(table_path, own_row + "\n")
        selftest.expect("改了这条用例登记行的第三列，指纹变", with_threads != baseline, f"原样与改了之后都是 {baseline[:16]}")

        lint_rows = [
            ("用例函数在测试目标里找不到", "crash-case:own\tcrates/\ttest=pkg:own_case:no_such_function\t# 样本", "找不到"),
            ("测试目标不在", "crash-case:own\tcrates/\ttest=pkg:no_such_target:the_case\t# 样本", "不在"),
            ("包不是工作区成员", "crash-case:own\tcrates/\ttest=nowhere:own_case:the_case\t# 样本", "不是仓根 Cargo.toml 的工作区成员"),
            ("test= 写了两条", "crash-case:own\tcrates/\ttest=pkg:own_case:the_case test=pkg:own_case:the_case\t# 样本", "恰好一条 test="),
            ("exhaustive= 点名的前缀没登记成 count-line=", "crash-case:own\tcrates/\ttest=pkg:own_case:the_case exhaustive=LAYER0\t# 样本", "没有登记成 count-line="),
            ("认不出的条件", "crash-case:own\tcrates/\ttest=pkg:own_case:the_case command=cargo\t# 样本", "认不出"),
            ("同一个键两行", own_row + "\n" + own_row, "一条用例只许一行"),
        ]
        for label, rows_text, expected_phrase in lint_rows:
            write_text(table_path, rows_text + "\n")
            exit_code, output, _messages = admission("crash-cases", work)
            selftest.expect(f"crash-cases 自查：{label}，退 2 并说出来", exit_code == EXIT_REGISTRATION_ERROR and expected_phrase in output,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        write_text(table_path, own_row + "\n")

        # 登记的用例函数要标 #[ignore]：没标的、只有 cfg_attr 里条件 ignore 的、ignore 挂在前一个函数上的、只写在注释里的，crash-cases 退 2
        own_case_path = os.path.join(work, "crates/pkg/tests/own_case.rs")
        with open(own_case_path, encoding="utf-8") as handle:
            own_case_text = handle.read()
        modules = "mod common;\nmod helper_target;\n"
        cfg_split_case = "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\nfn the_case() {}\n"
        submodule_first_case = "mod slow {\n    #[test]\n    #[ignore]\n    fn the_case() {}\n}\n#[test]\nfn the_case() {}\n"
        spaced_ignore_case = "#[test]\n# [ignore]\nfn the_case() {}\n"
        ignore_cells = [
            ("用例函数没标 #[ignore]", modules + "#[test]\nfn the_case() {}\n", False),
            ("只有 cfg_attr 里的条件 ignore", modules + "#[test]\n#[cfg_attr(miri, ignore)]\nfn the_case() {}\n", False),
            ("#[ignore] 挂在前一个函数上", modules + "#[test]\n#[ignore]\nfn other() {}\n#[test]\nfn the_case() {}\n", False),
            ("#[ignore] 只写在注释里", modules + "#[test]\n// #[ignore]\nfn the_case() {}\n", False),
            ("#[ignore] 在 #[test] 前面、fn 前面有 pub", modules + "#[ignore]\n#[test]\npub fn the_case() {}\n", True),
            ("ignore 的理由里带方括号、引号与 //", modules + '#[test]\n#[ignore = "样本 [x] \\"y\\" // z"]\nfn the_case() {}\n', True),
            ("同名函数 cfg 二选一：debug 那一份标 ignore、release 那一份不标", modules + cfg_split_case, False),
            ("子模块里同名标 ignore 的写在前面、顶层那一份（--exact the_case 跑的）不标", modules + submodule_first_case, False),
            ("# [ignore]（# 与 [ 之间有空格，rustc 认）", modules + spaced_ignore_case, True),
            ("同名的两处都标了 ignore（cfg 二选一）", modules + cfg_split_case.replace("#[test]\nfn the_case", "#[test]\n#[ignore]\nfn the_case"), True),
        ]
        for label, source_text, marked in ignore_cells:
            write_text(own_case_path, source_text)
            exit_code, output, _messages = admission("crash-cases", work)
            green = exit_code == 0 if marked else (exit_code == EXIT_REGISTRATION_ERROR and "没标 #[ignore]" in output)
            selftest.expect(f"crash-cases 自查：{label} ⇒ {'退 0' if marked else '退 2 并说没标 #[ignore]'}", green,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        for switch, label, source_text, broken_exit_code in (
                ("first-definition-only", "同名函数 cfg 二选一、一份不标", modules + cfg_split_case, 0),
                ("first-definition-only", "子模块里同名标 ignore 的写在前面、顶层那一份不标", modules + submodule_first_case, 0),
                ("ignore-attribute-without-space", "# [ignore]", modules + spaced_ignore_case, EXIT_REGISTRATION_ERROR)):
            write_text(own_case_path, source_text)
            exit_code, output, _messages = admission("crash-cases", work, changes={BREAK_VARIABLE: switch})
            selftest.expect(f"弄坏开关 {switch} 下「{label}」那一格红（crash-cases 退 {broken_exit_code}）", exit_code == broken_exit_code,
                            f"弄坏之后退 {exit_code}：这一格分不出差别；stdout「{output.strip()}」")
        # 顺着 include!("<字面路径>") / #[path = "…"] mod / mod <名>; 带进来的源文件一起判；判不出的按没标算（弄坏开关 target-own-files-only 下头一格红）
        marked_here = "#[test]\n#[ignore]\nfn the_case() {}\n"
        release_unmarked = {"crates/pkg/tests/common/release_case.rs": "#[test]\nfn the_case() {}\n"}
        cfg_include = ("#[cfg(debug_assertions)]\n" + marked_here + "#[cfg(not(debug_assertions))]\ninclude!(\"common/release_case.rs\");\n")
        included_cells = [
            ("debug 下标 ignore、release 下 include!(\"common/release_case.rs\") 进来一份不标的同名用例", modules + cfg_include, release_unmarked,
             "没标 #[ignore]"),
            ("include! 的参数是 concat!(…)（读的是哪一份判不出）", modules + marked_here + 'include!(concat!("common/", "release_case.rs"));\n',
             release_unmarked, "判不出"),
            ("#[path = \"common/p.rs\"] mod p; 里有一份不标的同名用例", modules + marked_here + '#[path = "common/p.rs"]\nmod p;\n',
             {"crates/pkg/tests/common/p.rs": "#[test]\nfn the_case() {}\n"}, "没标 #[ignore]"),
            ("mod 指的文件找不到", modules + marked_here + "mod missing_module;\n", {}, "判不出"),
            ("use core::include as pull; 把 include! 改名引进来", modules + "use core::include as pull;\n" + marked_here, {}, "判不出"),
            ("对照：include! 进来的同名用例也标了 ignore", modules + cfg_include.replace("release_case.rs", "marked_case.rs"),
             {"crates/pkg/tests/common/marked_case.rs": marked_here}, None),
        ]
        for label, source_text, extra_files, phrase in included_cells:
            write_text(own_case_path, source_text)
            for relative, content in extra_files.items():
                write_text(os.path.join(work, relative), content)
            exit_code, output, _messages = admission("crash-cases", work)
            if label == included_cells[0][0]:
                broken_exit_code, broken_output, _messages = admission("crash-cases", work, changes={BREAK_VARIABLE: "target-own-files-only"})
            for relative in extra_files:
                os.remove(os.path.join(work, relative))
            green = exit_code == 0 if phrase is None else (exit_code == EXIT_REGISTRATION_ERROR and phrase in output)
            selftest.expect(f"crash-cases 自查（带进来的源文件）：{label} ⇒ {'退 0' if phrase is None else '退 2 并说' + phrase}", green,
                            f"退 {exit_code}，stdout「{output.strip()}」")
        selftest.expect("弄坏开关 target-own-files-only 下「release 下 include! 进来一份不标的同名用例」那一格红（crash-cases 退 0）", broken_exit_code == 0,
                        f"弄坏之后退 {broken_exit_code}：这一格分不出差别；stdout「{broken_output.strip()}」")
        write_text(own_case_path, own_case_text)

        # 认得出形状、认不出读的是哪一份的按宽处理：包里 include!(concat!(…))、任何一个包的构建脚本出现整词 tests ⇒ 改 other_target.rs 指纹变
        other_target_path = os.path.join(work, "crates/pkg/tests/other_target.rs")
        root_manifest_path = os.path.join(work, "Cargo.toml")
        with open(root_manifest_path, encoding="utf-8") as handle:
            root_manifest_text = handle.read()
        two_members = '[workspace]\nmembers = ["crates/pkg", "crates/gen"]\n'
        gen_package = '[package]\nname = "gen"\nversion = "0.0.0"\n'
        reads_tests = 'fn main() { let _ = std::fs::read_dir("../pkg/tests"); }\n'
        qualified_include = ('const PIECE: &str = ::core::include_str!(::core::concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oth", '
                             '"er_target.rs"));\n')
        environment_include = 'const PIECE: &str = include_str!(env!("SAMPLE_FIXTURE"));\n'
        aliased_include = ('use core::include_str as grab;\nconst PIECE: &str = grab!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oth", '
                           '"er_target.rs"));\n')
        wide_cells = [
            ("用例里 include!(concat!(\"other_\", \"target.rs\"))", {"crates/pkg/tests/own_case.rs": own_case_text + 'include!(concat!("other_", "target.rs"));\n'},
             True),
            ("用例里 ::core::include_str!(::core::concat!(…))（宏名带路径）", {"crates/pkg/tests/own_case.rs": own_case_text + qualified_include}, True),
            ("用例里 include_str!(env!(\"…\"))（路径在构建环境里）", {"crates/pkg/tests/own_case.rs": own_case_text + environment_include}, True),
            ("用例里 use core::include_str as grab; 再 grab!(concat!(…))（改名之后的调用认不出）", {"crates/pkg/tests/own_case.rs": own_case_text + aliased_include},
             True),
            ("对照：用例里 include_str!(\"helper_target.rs\")（字面量，读的是哪一份认得出）",
             {"crates/pkg/tests/own_case.rs": own_case_text + 'const PIECE: &str = include_str!("helper_target.rs");\n'}, False),
            ("别的包的 build.rs 按目录读 ../pkg/tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package, "crates/gen/src/lib.rs": "",
                                                    "crates/gen/build.rs": reads_tests}, True),
            ("别的包 package.build 指的构建脚本按目录读 tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package + 'build = "generate.rs"\n',
                                                          "crates/gen/src/lib.rs": "", "crates/gen/generate.rs": reads_tests}, True),
            ("对照：别的包的 build.rs 不读 tests", {"Cargo.toml": two_members, "crates/gen/Cargo.toml": gen_package, "crates/gen/src/lib.rs": "",
                                               "crates/gen/build.rs": "fn main() {}\n"}, False),
        ]
        for label, files, should_move in wide_cells:
            for relative, content in files.items():
                write_text(os.path.join(work, relative), content)
            before, _count = fingerprint()
            write_text(other_target_path, "#[test]\nfn other() { assert!(true); }\n")
            after, _count = fingerprint()
            write_text(other_target_path, "#[test]\nfn other() {}\n")
            write_text(own_case_path, own_case_text)
            write_text(root_manifest_path, root_manifest_text)
            shutil.rmtree(os.path.join(work, "crates/gen"), ignore_errors=True)
            selftest.expect(f"崩溃枚举用例的输入：{label}，改 other_target.rs 指纹{'变' if should_move else '不变'}",
                            (before != after) == should_move and not before.startswith("退"), f"改之前 {before[:16]}，改之后 {after[:16]}")
        write_text(own_case_path, own_case_text + qualified_include)
        broken_before, _count = fingerprint({BREAK_VARIABLE: "concat-include-only"})
        write_text(other_target_path, "#[test]\nfn other() { assert!(true); }\n")
        broken_after, _count = fingerprint({BREAK_VARIABLE: "concat-include-only"})
        write_text(other_target_path, "#[test]\nfn other() {}\n")
        write_text(own_case_path, own_case_text)
        selftest.expect("弄坏开关 concat-include-only 下「::core::include_str!(::core::concat!(…))」那一格红（改 other_target.rs 指纹不变）",
                        broken_before == broken_after, f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
        write_text(own_case_path, own_case_text + aliased_include)
        broken_before, _count = fingerprint({BREAK_VARIABLE: "include-alias-subtracts"})
        write_text(other_target_path, "#[test]\nfn other() { assert!(true); }\n")
        broken_after, _count = fingerprint({BREAK_VARIABLE: "include-alias-subtracts"})
        write_text(other_target_path, "#[test]\nfn other() {}\n")
        write_text(own_case_path, own_case_text)
        selftest.expect("弄坏开关 include-alias-subtracts 下「use core::include_str as grab; 再 grab!(concat!(…))」那一格红（改 other_target.rs 指纹不变）",
                        broken_before == broken_after, f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")

        # 用例读不到的非测试文件：crates/mutations.tsv 与（没有代码读 CARGO_BIN_EXE_ 时）src/bin/ 下的，改了指纹不变；有代码点名它们时照留
        lib_path = os.path.join(work, "crates/pkg/src/lib.rs")
        with open(lib_path, encoding="utf-8") as handle:
            lib_text = handle.read()
        table_versions = ("# 样本变异表\n", "# 样本变异表\n多一行\n")
        tool_versions = ("fn main() {}\n", "fn main() { let changed = 1; }\n")
        concatenated_table_include = 'const TABLE_TEXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../mutations", ".tsv"));\n'
        not_read_cells = [
            ("crates/mutations.tsv 加一行（没有代码点名它）", {}, "crates/mutations.tsv", table_versions, False),
            ("lib.rs 的字符串里点名 mutations.tsv 时，crates/mutations.tsv 加一行", {"crates/pkg/src/lib.rs": lib_text + 'pub const TABLE: &str = "../mutations.tsv";\n'},
             "crates/mutations.tsv", table_versions, True),
            ("src/bin/tool.rs 改了（没有代码读 CARGO_BIN_EXE_）", {}, "crates/pkg/src/bin/tool.rs", tool_versions, False),
            ("用例读 CARGO_BIN_EXE_tool 时，src/bin/tool.rs 改了", {"crates/pkg/tests/own_case.rs": own_case_text + 'const TOOL: &str = env!("CARGO_BIN_EXE_tool");\n'},
             "crates/pkg/src/bin/tool.rs", tool_versions, True),
            ("lib.rs 用 #[path = \"bin/tool.rs\"] 把它编进来时，src/bin/tool.rs 改了", {"crates/pkg/src/lib.rs": lib_text + '#[path = "bin/tool.rs"]\nmod tool;\n'},
             "crates/pkg/src/bin/tool.rs", tool_versions, True),
            ("用例里 include_str!(concat!(…)) 拼出 ../mutations.tsv 时，crates/mutations.tsv 加一行",
             {"crates/pkg/tests/own_case.rs": own_case_text + concatenated_table_include}, "crates/mutations.tsv", table_versions, True),
            ("用例里 use core::include_str as grab; 再 grab!(concat!(…)) 拼出 ../mutations.tsv 时，crates/mutations.tsv 加一行",
             {"crates/pkg/tests/own_case.rs": own_case_text + 'use core::include_str as grab;\nconst TABLE_TEXT: &str = grab!(concat!(env!("CARGO_MANIFEST_DIR"), '
                                                              '"/../mutations", ".tsv"));\n'}, "crates/mutations.tsv", table_versions, True),
            ("用例里 include!(concat!(…)) 拼出 src/bin/tool.rs 时，src/bin/tool.rs 改了",
             {"crates/pkg/tests/own_case.rs": own_case_text + 'mod tool_code { include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/", "tool.rs")); }\n'},
             "crates/pkg/src/bin/tool.rs", tool_versions, True),
        ]
        for label, setup, relative, (first_text, second_text), should_move in not_read_cells:
            for setup_relative, content in setup.items():
                write_text(os.path.join(work, setup_relative), content)
            write_text(os.path.join(work, relative), first_text)
            before, _count = fingerprint()
            write_text(os.path.join(work, relative), second_text)
            after, _count = fingerprint()
            os.remove(os.path.join(work, relative))
            write_text(own_case_path, own_case_text)
            write_text(lib_path, lib_text)
            selftest.expect(f"崩溃枚举用例的输入：{label}，指纹{'变' if should_move else '不变'}", (before != after) == should_move and not before.startswith("退"),
                            f"改之前 {before[:16]}，改之后 {after[:16]}")
        write_text(own_case_path, own_case_text + concatenated_table_include)
        write_text(os.path.join(work, "crates/mutations.tsv"), table_versions[0])
        broken_before, _count = fingerprint({BREAK_VARIABLE: "computed-include-subtracts-non-test-files"})
        write_text(os.path.join(work, "crates/mutations.tsv"), table_versions[1])
        broken_after, _count = fingerprint({BREAK_VARIABLE: "computed-include-subtracts-non-test-files"})
        os.remove(os.path.join(work, "crates/mutations.tsv"))
        write_text(own_case_path, own_case_text)
        selftest.expect("弄坏开关 computed-include-subtracts-non-test-files 下「include_str!(concat!(…)) 拼出 ../mutations.tsv」那一格红（指纹不变）",
                        broken_before == broken_after, f"弄坏之后仍然变了：{broken_before[:16]} → {broken_after[:16]}")
        restored_after_wide_cells, _count = fingerprint()
        selftest.expect("崩溃枚举用例的输入：按宽处理与读不到的那几格改回之后，指纹回到原样", restored_after_wide_cells == baseline,
                        f"原样 {baseline[:16]}，改回之后 {restored_after_wide_cells[:16]}")

        # 全绿标记：写了再核作数；换一个指纹不作数且比出不同；标记被改（指纹、exhaustive）不作数
        judged_file = os.path.join(work, "judged.out")
        write_text(judged_file, "test_result=test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s\n"
                                "LAYER0 states=5 closed_form=5 exhaustive=true\nparallel_finished=LAYER0_PARALLEL_FINISHED states=5 slices=1\n")
        fingerprint_now, _count = fingerprint()
        record_arguments = ("crash-case-record", work, "crash-case:own", fingerprint_now, manifest_file, judged_file, "--files", "6", "--excluded", "1",
                            "--started", "2026-09-26T00:00:00Z", "--judged-root", work)   # clock-times:allow 自检造的开跑时刻，格式照真记录
        explicit_seven = {"SINGLEFS_LAYER0_THREADS": "7"}
        cores = machine_core_count(os.environ)
        exit_code, marker_path, _messages = admission(*record_arguments, changes={BREAK_VARIABLE: "single-worker-threads-field", **explicit_seven})
        with open(marker_path.strip(), encoding="utf-8") as handle:
            broken_marker_lines = handle.read().split("\n")
        selftest.expect("弄坏开关 single-worker-threads-field 下「配的与起的线程分两格记」那一格红（只剩一格 worker_threads=）",
                        exit_code == 0 and any(line.startswith("worker_threads=") for line in broken_marker_lines)
                        and not any(line.startswith("configured_worker_threads=") for line in broken_marker_lines),
                        f"写退 {exit_code}，标记里 {[line for line in broken_marker_lines if 'threads' in line]}")
        exit_code, marker_path, _messages = admission(*record_arguments, changes=explicit_seven)
        marker_path = marker_path.strip()
        with open(marker_path, encoding="utf-8") as handle:
            marker_lines = handle.read().split("\n")
        selftest.expect("crash-case-record 把配的线程数记成 configured_worker_threads=（线程数、显式设的、本机几核，由它自己现取），不记旧的 worker_threads=",
                        exit_code == 0 and f"configured_worker_threads=SINGLEFS_LAYER0_THREADS=7（显式设的），本机 {cores} 核" in marker_lines
                        and not any(line.startswith("worker_threads=") for line in marker_lines),
                        f"写退 {exit_code}，标记里 {[line for line in marker_lines if 'threads' in line]}")
        exit_code_check, output, _messages = admission("crash-case-marker-check", work, "crash-case:own", fingerprint_now, manifest_file)
        selftest.expect("crash-case-record 写的那一格，crash-case-marker-check 判作数并带出计数行",
                        exit_code == 0 and os.path.isfile(marker_path) and exit_code_check == 0 and output.startswith(f"ok {marker_path} ")
                        and "LAYER0 states=5 closed_form=5 exhaustive=true" in output, f"写退 {exit_code}，核退 {exit_code_check}，stdout「{output.strip()}」")
        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() { let changed = 1; }\n")
        other_fingerprint, _count = fingerprint()
        exit_code_check, output, _messages = admission("crash-case-marker-check", work, "crash-case:own", other_fingerprint, manifest_file)
        write_text(os.path.join(work, "crates/pkg/tests/helper_target.rs"), "pub fn helper() {}\n")
        selftest.expect("输入变了的那一批没有自己那一格：不作数，并比出与最近写的一格不同的是哪个文件",
                        exit_code_check == 1 and "没有全绿标记" in output and "内容不同：crates/pkg/tests/helper_target.rs" in output,
                        f"退 {exit_code_check}，stdout「{output.strip()}」")
        with open(marker_path, encoding="utf-8") as handle:
            good_marker = handle.read()
        for label, old, new, phrase in (("记的指纹被改了", f"input_hash={fingerprint_now}", "input_hash=" + "0" * 64, "输入指纹"),
                                        ("计数行不带 exhaustive=true", "exhaustive=true", "exhaustive=false", "exhaustive=true"),
                                        ("计数行抄了两遍", "LAYER0 states=5 closed_form=5 exhaustive=true\n",
                                         "LAYER0 states=5 closed_form=5 exhaustive=true\n" * 2, "恰好一行"),
                                        ("test result 不是 1 passed", "1 passed;", "0 passed;", "1 passed")):
            write_text(marker_path, good_marker.replace(old, new, 1))
            exit_code_check, output, _messages = admission("crash-case-marker-check", work, "crash-case:own", fingerprint_now, manifest_file)
            selftest.expect(f"全绿标记{label}：不作数并说出来", exit_code_check == 1 and phrase in output, f"退 {exit_code_check}，stdout「{output.strip()}」")
        write_text(marker_path, good_marker)
    finally:
        shutil.rmtree(work, ignore_errors=True)


def synthetic_layer0_log(states=100, slices=64, worker_threads=32, resumed_slices=0, freshly_run_slices=64, exhaustive="true",
                         result="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.00s",
                         layer0_lines=1, with_checker=True, with_finished=True, finished_fields=None):
    """合成的一趟 --full 日志：LAYER0_PARALLEL_FINISHED、LAYER0 计数行、CHECKER 行、test result 各几行由参数定。"""
    lines = [f"LAYER0_PARALLEL_START states={states} slices={slices}"]
    if with_finished:
        fields = finished_fields or (f"worker_threads={worker_threads} configured_worker_threads=32 worker_threads_source=environment_variable "
                                     f"resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices}")
        lines.append(f"LAYER0_PARALLEL_FINISHED states={states} slices={slices} {fields} progress_file_after_completion=deleted elapsed_seconds=1.0")
    lines += [f"LAYER0 states={states} closed_form={states} violations=0 exhaustive={exhaustive}"] * layer0_lines
    if with_checker:
        lines.append("CHECKER record_root_without_record=0 record_claimed_state_missing_unit=0")
    if result is not None:
        lines.append(result)
    return "\n".join(lines) + "\n"


def run_crash_case_log_cells(selftest):
    """--full 跑一条用例的日志怎么判（纯函数，合成日志）：test result 恰好 1 passed、计数行恰好一行、exhaustive=true、工作线程按这一趟跑了几片判。"""
    case = parse_crash_case(RegistrationRow("crash-case:demo", ["crates/"], ["test=pkg:target:function", "count-line=LAYER0",
                                                                             "count-line=CHECKER", "exhaustive=LAYER0", "threads=LAYER0"], 1, ""))
    cells = [
        ("全量、32 个线程跑了 64 片", synthetic_layer0_log(), 32, False, True),
        ("全部 64 片从进度文件读回、起 0 个线程", synthetic_layer0_log(worker_threads=0, resumed_slices=64, freshly_run_slices=0), 32, False, True),
        ("读回 63 片、只剩 1 片要跑、起 1 个线程", synthetic_layer0_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), 32, False, True),
        ("跑了 64 片却只起 1 个线程（本机 32 核、线程数没显式设）", synthetic_layer0_log(worker_threads=1), 32, False, False),
        ("跑了 64 片只起 1 个线程，线程数显式设成 1", synthetic_layer0_log(worker_threads=1), 32, True, True),
        ("跑了 64 片只起 1 个线程，本机只有 1 核", synthetic_layer0_log(worker_threads=1), 1, False, True),
        ("跑了 5 片却报起了 0 个线程", synthetic_layer0_log(worker_threads=0, resumed_slices=59, freshly_run_slices=5), 32, False, False),
        ("读回的片 + 这一趟跑的片 ≠ 总片数", synthetic_layer0_log(resumed_slices=10, freshly_run_slices=10), 32, False, False),
        ("LAYER0_PARALLEL_FINISHED 缺续跑片数（旧格式）", synthetic_layer0_log(finished_fields="worker_threads=32"), 32, False, False),
        ("没有状态数对得上的 LAYER0_PARALLEL_FINISHED", synthetic_layer0_log(with_finished=False), 32, False, False),
        ("过滤之后一条用例都没跑（0 passed）",
         synthetic_layer0_log(result="test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s"), 32, False, False),
        ("没有 test result 行", synthetic_layer0_log(result=None), 32, False, False),
        ("不是全量（exhaustive=false）", synthetic_layer0_log(exhaustive="false"), 32, False, False),
        ("LAYER0 计数行抄了两遍", synthetic_layer0_log(layer0_lines=2), 32, False, False),
        ("没有 CHECKER 行", synthetic_layer0_log(with_checker=False), 32, False, False),
    ]
    for label, log_text, machine_cores, explicitly_one, expected_green in cells:
        problems, recorded_lines, _notes = judge_crash_case_log(case, log_text, machine_cores, explicitly_one)
        green = not problems
        recorded_ok = not green or (recorded_lines[0].startswith("test_result=test result: ok. 1 passed")
                                    and any(line.startswith("LAYER0 ") for line in recorded_lines)
                                    and any(line.startswith("parallel_finished=LAYER0_PARALLEL_FINISHED ") for line in recorded_lines))
        selftest.expect(f"判 --full 日志：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green and recorded_ok,
                        f"判成{'绿' if green else '红'}：{problems}；记下的行 {recorded_lines}")
    with BreakSwitch("threads-by-worker-count"):
        problems, _recorded, _notes = judge_crash_case_log(
            case, synthetic_layer0_log(worker_threads=1, resumed_slices=63, freshly_run_slices=1), 32, False)
    selftest.expect("弄坏开关 threads-by-worker-count 下「只剩 1 片要跑、起 1 个线程」那一格红（照旧只看 worker_threads=1）", bool(problems),
                    "弄坏之后仍判绿：这一格分不出「这一趟跑了几片」")
    # 标记里「实际起的工作线程」那一格（started_worker_threads=）：登记了 threads= 的取 LAYER0_PARALLEL_FINISHED 里起了几个，没登记的写读不到
    problems, recorded_lines, _notes = judge_crash_case_log(case, synthetic_layer0_log(worker_threads=24), 32, False)
    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
    selftest.expect("判绿时记 started_worker_threads=：登记了 threads= 的记 LAYER0_PARALLEL_FINISHED 里起了几个（24 个，不是配的 32）",
                    not problems and started == ["started_worker_threads=LAYER0：24 个工作线程跑了 64 片、从进度文件读回 0 片（共 64 片）"],
                    f"判出 {problems}，记下 {started}")
    unthreaded = parse_crash_case(RegistrationRow("crash-case:plain", ["crates/"], ["test=pkg:target:function", "count-line=PLAIN"], 1, ""))
    problems, recorded_lines, _notes = judge_crash_case_log(unthreaded, "PLAIN states=4\n" + PASSED_ONE_LINE + "\n", 32, False)
    started = [line for line in recorded_lines if line.startswith("started_worker_threads=")]
    selftest.expect("判绿时记 started_worker_threads=：没登记 threads= 的记「读不到」（门禁不读它起了几个）",
                    not problems and len(started) == 1 and started[0].startswith("started_worker_threads=读不到：这条用例没登记 threads="),
                    f"判出 {problems}，记下 {started}")
    run_shard_merge_log_cells(selftest, case)


def synthetic_merge_log(shard_worker_threads=(16, 16), shard_configured=(16, 16), shard_sources=("environment_variable", "environment_variable"),
                        shard_cores=(32, 32), shard_resumed=(0, 0), shard_freshly_run=(32, 32), shards=None):
    """合成的双机分片 merge 那一趟日志：LAYER0_PARALLEL_FINISHED 的合计照 crates/singlefs-checker-tier/src/crash.rs 的 merge_the_shard_ledgers
    （线程与片数是各片之和、来源写 shard_ledgers），另带 shards= 与逐片的 shard_…=。"""
    joined = lambda values: ",".join(str(value) for value in values)
    finished_fields = (f"worker_threads={sum(shard_worker_threads)} configured_worker_threads={sum(shard_configured)} worker_threads_source=shard_ledgers "
                       f"resumed_slices={sum(shard_resumed)} freshly_run_slices={sum(shard_freshly_run)} "
                       f"shards={shards if shards is not None else len(shard_worker_threads)} shard_worker_threads={joined(shard_worker_threads)} "
                       f"shard_configured_worker_threads={joined(shard_configured)} shard_worker_threads_sources={joined(shard_sources)} "
                       f"shard_available_parallelism={joined(shard_cores)} shard_resumed_slices={joined(shard_resumed)} "
                       f"shard_freshly_run_slices={joined(shard_freshly_run)} shard_elapsed_milliseconds=1000,1000")
    return synthetic_layer0_log(finished_fields=finished_fields)


def run_shard_merge_log_cells(selftest, case):
    """双机分片 merge 那一趟的日志：带 shards= 的 LAYER0_PARALLEL_FINISHED 逐片判工作线程（机器核数与「显式设成 1」取那一片自己的）；
    登记行的 shard= 只认 across-machines、至多一条。"""
    cells = [
        ("merge：两片各 16 个线程各跑 32 片", synthetic_merge_log(), True),
        ("merge：第 1 片那台 32 核、线程数没显式设，跑了 32 片却只起 1 个线程",
         synthetic_merge_log(shard_worker_threads=(16, 1), shard_configured=(16, 32), shard_sources=("environment_variable", "available_parallelism")), False),
        ("merge：第 1 片只起 1 个线程，那一片的线程数显式设成 1",
         synthetic_merge_log(shard_worker_threads=(16, 1), shard_configured=(16, 1)), True),
        ("merge：第 1 片只起 1 个线程，那台只有 1 核", synthetic_merge_log(shard_worker_threads=(16, 1), shard_configured=(16, 1),
                                                          shard_sources=("environment_variable", "available_parallelism"), shard_cores=(32, 1)), True),
        ("merge：第 1 片跑了 32 片却报起了 0 个线程", synthetic_merge_log(shard_worker_threads=(16, 0)), False),
        ("merge：shards=3 而逐片字段只有两片", synthetic_merge_log(shards=3), False),
    ]
    for label, log_text, expected_green in cells:
        problems, _recorded, notes = judge_crash_case_log(case, log_text, 32, False)
        green = not problems
        selftest.expect(f"判 --full 日志：{label} ⇒ {'绿' if expected_green else '红'}",
                        green == expected_green and (not green or any("双机分片 2 片再 merge" in note for note in notes)),
                        f"判成{'绿' if green else '红'}：{problems}；线程那一句 {notes}")
    with BreakSwitch("threads-ignore-shards"):
        problems, _recorded, _notes = judge_crash_case_log(case, cells[1][1], 32, False)
    selftest.expect("弄坏开关 threads-ignore-shards 下「第 1 片只起 1 个线程跑了 32 片」那一格判绿（只看各片之和 17 个线程，分不出是哪一片只起了 1 个）",
                    not problems, f"弄坏之后仍判红：{problems}")
    for label, conditions, expected in (
            ("登记了 shard=across-machines", ["test=pkg:target:function", "shard=across-machines"], True),
            ("没登记 shard=", ["test=pkg:target:function"], False),
            ("shard= 写成别的值", ["test=pkg:target:function", "shard=yes"], "error"),
            ("shard= 写了两条", ["test=pkg:target:function", "shard=across-machines", "shard=across-machines"], "error")):
        try:
            outcome = parse_crash_case(RegistrationRow("crash-case:demo", ["crates/"], conditions, 1, "")).is_shardable_across_machines
        except RegistrationError:
            outcome = "error"
        selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ {expected}", outcome == expected, f"读成 {outcome}")


def edit_inside_definition(source, name, old, new):
    """只在模块级定义 name（def、class 或赋值）的原文里把 old 换成 new，old 在那里要恰好一处；返回换过的整份源码。
    自证拿它改准入模块自己的一份拷贝：只认那个定义里的那一处，自证里写着的同一串字不算。命中不是一处抛 ValueError。"""
    nodes = top_level_definitions(ast.parse(source)).get(name, [])
    if len(nodes) != 1:
        raise ValueError(f"模块级定义 {name} 有 {len(nodes)} 处，要恰好一处")
    lines = source.split("\n")
    first = min([nodes[0].lineno] + [decorator.lineno for decorator in getattr(nodes[0], "decorator_list", [])])
    head, body, tail = "\n".join(lines[:first - 1]), "\n".join(lines[first - 1:nodes[0].end_lineno]), "\n".join(lines[nodes[0].end_lineno:])
    if body.count(old) != 1:
        raise ValueError(f"{name} 里「{old}」有 {body.count(old)} 处，要恰好一处")
    return head + "\n" + body.replace(old, new) + "\n" + tail


def run_thread_source_cells(selftest, module):
    """门禁批第三轮 FM 与 FD3：线程从哪来、判法自己现取。
    FM：登记行的 threads-variable= 由 crash-case-command 设成配的线程数（调用方设着的不认，弄坏开关 thread-variable-ignored 下漏进来）、
    标记里 configured_worker_threads= 记这个变量；本机核数不认 OMP_NUM_THREADS 与 PATH 里的 nproc（弄坏开关 cores-from-nproc 下认）。
    FD3：crash-case-judge 自己现取线程数：显式设 SINGLEFS_LAYER0_THREADS=1 的 64 片 1 线程判绿、没设的判红；转过来的 --threads 1 --threads-origin explicit
    是用法错（退 2），翻不了判定（弄坏开关 judge-takes-forwarded-threads 下照旧收、判绿）。"""
    work = tempfile.mkdtemp(prefix="admission-selftest-threads-")
    try:
        row = ("crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0 count-line=CHECKER exhaustive=LAYER0 threads=LAYER0 "
               "threads-variable=SINGLEFS_SAMPLE_CASE_THREADS\t# 样本")
        build_crash_case_repository(work, [row])
        case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
        cores = machine_core_count(os.environ)
        caller = dict(os.environ, SINGLEFS_SAMPLE_CASE_THREADS="1")
        caller.pop(LAYER0_THREADS_VARIABLE, None)

        def seen_by_the_case(case_now):
            command = crash_case_launch(work, case_now, "0" * 64, False, caller)[4]
            launched = command[:command.index("cargo")] + ["sh", "-c", 'printf "%s %s" "${SINGLEFS_SAMPLE_CASE_THREADS-unset}" "${SINGLEFS_LAYER0_THREADS-unset}"']
            return subprocess.run(launched, env=caller, capture_output=True, text=True, check=False).stdout
        seen = seen_by_the_case(case)
        selftest.expect(f"crash-case-command：登记了 threads-variable=SINGLEFS_SAMPLE_CASE_THREADS、调用方设着它等于 1 ⇒ 起的用例看到配的 {cores}（盖掉调用方的）",
                        seen == f"{cores} {cores}", f"用例看到「{seen}」")
        with BreakSwitch("thread-variable-ignored"):
            broken_case = crash_case_of_key(read_registration_rows(work), "crash-case:own")
            broken_seen = seen_by_the_case(broken_case)
            broken_text = configured_worker_threads_text(broken_case, cores, cores, "default")
        selftest.expect("弄坏开关 thread-variable-ignored 下「调用方设着用例读的线程变量」那一格红（用例看到 1）", broken_seen.startswith("1 "),
                        f"弄坏之后用例看到「{broken_seen}」：这一格分不出命令设没设它")
        text = configured_worker_threads_text(case, cores, cores, "default")
        selftest.expect("标记里 configured_worker_threads= 记的是这条用例读的那个变量（SINGLEFS_SAMPLE_CASE_THREADS=…），不是 SINGLEFS_LAYER0_THREADS",
                        text.startswith(f"SINGLEFS_SAMPLE_CASE_THREADS={cores}（没设，取本机核数）"), f"记成「{text}」")
        selftest.expect("弄坏开关 thread-variable-ignored 下「标记记的变量」那一格红（记成 SINGLEFS_LAYER0_THREADS）", broken_text.startswith("SINGLEFS_LAYER0_THREADS="),
                        f"弄坏之后记成「{broken_text}」")
        registration_cells = [("threads-variable= 写成小写", "threads-variable=sample_threads"), ("threads-variable= 写了两条",
                              "threads-variable=A_THREADS threads-variable=B_THREADS")]
        for label, conditions in registration_cells:
            try:
                parse_crash_case(RegistrationRow("crash-case:demo", ["crates/"], ["test=pkg:target:function", *conditions.split()], 1, ""))
                outcome = "读得出"
            except RegistrationError as error:
                outcome = f"拒：{error}"
            selftest.expect(f"崩溃枚举用例登记行：{label} ⇒ 拒", outcome.startswith("拒") and "threads-variable=" in outcome, outcome)
        # 本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 都不认
        tools = os.path.join(work, ".tools")
        write_executable(os.path.join(tools, "nproc"), "#!/usr/bin/env bash\necho 1\n")
        leaky = dict(os.environ, PATH=tools + os.pathsep + os.environ.get("PATH", ""), OMP_NUM_THREADS="1")
        leaky.pop(LAYER0_THREADS_VARIABLE, None)
        got = crash_case_worker_threads(case, leaky)
        selftest.expect(f"本机核数：PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 ⇒ 仍是 os.cpu_count() 与 CPU 亲和的小者（{cores}），线程数同它、default",
                        got == (cores, cores, "default"), f"交的是 {got}")
        with BreakSwitch("cores-from-nproc"):
            broken_got = crash_case_worker_threads(case, leaky)
        selftest.expect("弄坏开关 cores-from-nproc 下「nproc 打 1」那一格红（交的核数是 1）", broken_got[0] == 1, f"弄坏之后交的是 {broken_got}")
        # FD3：crash-case-judge 自己现取线程数
        one_thread_log = os.path.join(work, "one-thread.log")
        write_text(one_thread_log, synthetic_layer0_log(worker_threads=1))
        judged = os.path.join(work, "judged.out")
        plain_environment = {name: value for name, value in os.environ.items() if name != LAYER0_THREADS_VARIABLE}

        def judge(extra_arguments=(), changes=None):
            return run_in_environment([sys.executable, module, "crash-case-judge", work, "crash-case:own", one_thread_log, judged, *extra_arguments],
                                      dict(plain_environment, **(changes or {})))[0]
        forwarded = ("--machine-cores", str(cores), "--threads", "1", "--threads-origin", "explicit")
        cells = [("没设 SINGLEFS_LAYER0_THREADS：64 片 1 个线程", (), None, 1 if cores > 1 else 0),
                 ("调用方显式设 SINGLEFS_LAYER0_THREADS=1：64 片 1 个线程", (), {LAYER0_THREADS_VARIABLE: "1"}, 0),
                 ("没设 SINGLEFS_LAYER0_THREADS、转过来 --threads 1 --threads-origin explicit（54 号改前的转法）", forwarded, None, EXIT_REGISTRATION_ERROR)]
        for label, extra_arguments, changes, expected in cells:
            got_exit = judge(extra_arguments, changes)
            selftest.expect(f"crash-case-judge 自己现取线程数：{label} ⇒ 退 {expected}", got_exit == expected, f"退 {got_exit}")
        broken_exit = judge(forwarded, {BREAK_VARIABLE: "judge-takes-forwarded-threads"})
        selftest.expect("弄坏开关 judge-takes-forwarded-threads 下「转过来 --threads 1 explicit」那一格红（照旧收、判绿退 0）", broken_exit == 0,
                        f"弄坏之后退 {broken_exit}")
    finally:
        shutil.rmtree(work, ignore_errors=True)


# command_crash_case_shardable 里「没登记」那一支的末两行（自证把 return 1 改成 return 0 造一个判法改动）
SHARDABLE_ANSWER_UNREGISTERED = '：单机跑")\n    return 1'
# (说明, 改哪个模块级定义, 旧串, 新串, 判法摘要该不该变)：前四种是判法之外的改动（K4 那一轮量过它们让四条用例全重跑），后几种是判法本身
JUDGING_DIGEST_VARIANTS = [
    ("实验准入：admit_experiment 里加一行注释", "admit_experiment", "def admit_experiment(root, key):\n", "def admit_experiment(root, key):\n    # 样本\n",
     False),
    ("门禁复用：gate_reuse 的一句说明改字", "gate_reuse", "强制跑一趟", "强制跑一次", False),
    ("自证：run_selftest 里加一行注释", "run_selftest", "def run_selftest():\n", "def run_selftest():\n    # 样本\n", False),
    ("排除规则：files_crash_cases_do_not_read 里加一行注释（它的结果变了，清单自己会变）", "files_crash_cases_do_not_read",
     "    listed_set = set(listed_names)\n", "    # 样本\n    listed_set = set(listed_names)\n", False),
    ("判法：judge_worker_threads 把「至少两片」改成「至少三片」", "judge_worker_threads", "freshly_run_slices >= 2 and", "freshly_run_slices >= 3 and", True),
    ("判法调的小函数：fields_of_line 改写法", "fields_of_line", "line.split()", 'line.split(" ")', True),
    ("判法用的常量：PASSED_ONE_TEST_FORM 放宽", "PASSED_ONE_TEST_FORM", "1 passed", "[0-9]+ passed", True),
    ("登记行怎么读：parse_crash_case 不再要求 exhaustive= / threads= 先登记成 count-line=", "parse_crash_case",
     "if prefix not in count_lines]", "if False]", True),
    ("起用例的命令：crash_case_launch 把 --include-ignored 换成 --ignored", "crash_case_launch", '"--include-ignored"', '"--ignored"', True),
    ("标记里配的线程那一格：configured_worker_threads_text 改字", "configured_worker_threads_text", "本机 {machine_cores} 核", "{machine_cores} 核", True),
    ("main：分派的写法改了", "main", "return COMMANDS[arguments[0]](arguments[1:])", "return COMMANDS[arguments[0]](arguments[2:])", True),
    ("单机跑还是双机分片：command_crash_case_shardable 把「没登记 shard=」也答成登记了", "command_crash_case_shardable",
     SHARDABLE_ANSWER_UNREGISTERED, SHARDABLE_ANSWER_UNREGISTERED.replace("return 1", "return 0"), True),
]


def replace_definition(source, name, new_text):
    """把模块级定义 name（恰好一处）的整段原文换成 new_text（自证造「换一种写法」用）；返回换过的整份源码。命中不是一处抛 ValueError。"""
    nodes = top_level_definitions(ast.parse(source)).get(name, [])
    if len(nodes) != 1:
        raise ValueError(f"模块级定义 {name} 有 {len(nodes)} 处，要恰好一处")
    lines = source.split("\n")
    first = min([nodes[0].lineno] + [decorator.lineno for decorator in getattr(nodes[0], "decorator_list", [])])
    return "\n".join(lines[:first - 1] + [new_text] + lines[nodes[0].end_lineno:])


def replace_once_in(text, old, new):
    if text.count(old) != 1:
        raise ValueError(f"「{old[:60]}」有 {text.count(old)} 处，要恰好一处")
    return text.replace(old, new)


def digest_or_refusal(text):
    """判法摘要，或拒算时的「拒算：<原因>」。"""
    try:
        return crash_case_judging_digest_text(text)[0]
    except InputManifestError as error:
        return f"拒算：{error}".encode("utf-8")


def run_judging_digest_writing_cells(selftest, source):
    """判法摘要看得见换了写法之后的判法（门禁批第三轮 FD2）：先把判法改成那种写法，再改判法本身，第二步摘要要变、或者拒算。
    四种：元组解包赋值、if 块里的 def（模块级其余语句整条进摘要）、分派表的值写成 lambda（拒算）、判法挪进 import 的非标准库模块（拒算）；
    各自的弄坏开关下那一格转红。"""
    passed_value = source.split("PASSED_ONE_TEST_FORM = ", 1)[1].split("\n", 1)[0]
    fields_body = '    return dict(token.split("=", 1) for token in line.split() if "=" in token)'
    judge_call = "    problems, recorded_lines, thread_notes = judge_crash_case_log(case, log_text, machine_cores, threads_explicitly_one)\n"
    variants = [
        ("元组解包赋值：PASSED_ONE_TEST_FORM 挪进 `A, B = …`，再把判绿的正则放宽成「test result: 」开头都算", "digest-skips-other-statements",
         lambda text: replace_definition(text, "PASSED_ONE_TEST_FORM", f"PASSED_ONE_TEST_FORM, _SPARE_FORM = {passed_value}, None"),
         lambda text: replace_once_in(text, f"PASSED_ONE_TEST_FORM, _SPARE_FORM = {passed_value}, None",
                                      'PASSED_ONE_TEST_FORM, _SPARE_FORM = re.compile(r"^test result: "), None'), False),
        ("if 块里的 def：fields_of_line 挪进 `if True:` 块，再让它把 exhaustive=false 读成 true", "digest-skips-other-statements",
         lambda text: replace_definition(text, "fields_of_line", "if True:\n    def fields_of_line(line):\n    " + fields_body),
         lambda text: replace_once_in(text, "    " + fields_body, "    " + fields_body.replace("return dict(", 'return {"exhaustive": "true", **dict(') + "}"),
         False),
        ("分派表的值不是名字：crash-case-judge 那一项写成 lambda，再改 command_crash_case_judge 让它一律判绿", "digest-allows-non-name-dispatch",
         lambda text: edit_inside_definition(text, "COMMANDS", '"crash-case-judge": command_crash_case_judge,',
                                             '"crash-case-judge": lambda arguments: command_crash_case_judge(arguments),'),
         lambda text: replace_once_in(text, judge_call, judge_call + "    problems = []\n"), True),
        ("判法挪进 import 的别的模块：`from judging_helpers import fields_of_line`（再改那份模块，准入模块原文一个字不变）", "digest-allows-imported-names",
         lambda text: replace_definition(text, "fields_of_line", "from judging_helpers import fields_of_line  # noqa: E402"),
         lambda text: text, True),
    ]
    for label, switch, first_edit, second_edit, refused in variants:
        try:
            first = first_edit(source)
            second = second_edit(first)
        except ValueError as error:
            selftest.expect(f"判法摘要换写法：{label} 这一格的改动做得出来", False, str(error))
            continue
        first_digest, second_digest = digest_or_refusal(first), digest_or_refusal(second)
        if refused:
            holds = first_digest.startswith("拒算".encode("utf-8")) and second_digest.startswith("拒算".encode("utf-8"))
            expected = "两步都拒算"
        else:
            holds = first_digest != second_digest and not first_digest.startswith("拒算".encode("utf-8"))
            expected = "第二步摘要变"
        selftest.expect(f"判法摘要换写法：{label} ⇒ {expected}", holds,
                        f"第一步 {first_digest[:80]!r}，第二步 {second_digest[:80]!r}")
        with BreakSwitch(switch):
            broken_first, broken_second = digest_or_refusal(first), digest_or_refusal(second)
        selftest.expect(f"弄坏开关 {switch} 下「{label}」那一格红（第二步摘要不变、不拒算）",
                        broken_first == broken_second and not broken_first.startswith("拒算".encode("utf-8")),
                        f"弄坏之后第一步 {broken_first[:60]!r}，第二步 {broken_second[:60]!r}")
    unchanged = digest_or_refusal(source)
    selftest.expect("判法摘要：真准入模块自己不拒算（分派表那几项都是名字、判法闭包里没有非标准库的 import）", not unchanged.startswith("拒算".encode("utf-8")),
                    unchanged[:200].decode("utf-8", "replace"))


def run_judging_digest_cells(selftest, module):
    """判法摘要（D2）：判法之外的改动摘要不变、判法（连同它调的小函数、常量、起用例的命令、main 与分派表那几项）改了摘要变；
    两个弄坏开关下各自那一格转红。拿本模块自己的源码按 edit_inside_definition 改一份算，不动文件。"""
    with open(module, encoding="utf-8") as handle:
        source = handle.read()
    baseline, definition_count = crash_case_judging_digest_text(source)
    selftest.expect("判法摘要算得出来，闭包里有判日志、读写标记、起用例的命令与 crash-case-command 那几个定义",
                    definition_count > 0 and all(f"## {name}\n".encode() in baseline for name in
                                                 ("judge_crash_case_log", "crash_case_launch", "command_crash_case_command", "crash_case_marker_problems")),
                    f"闭包 {definition_count} 个定义")
    for label, name, old, new, should_change in JUDGING_DIGEST_VARIANTS:
        try:
            changed = crash_case_judging_digest_text(edit_inside_definition(source, name, old, new))[0]
        except ValueError as error:
            selftest.expect(f"判法摘要：{label} 这一格的改动做得出来", False, str(error))
            continue
        selftest.expect(f"判法摘要：{label} ⇒ 摘要{'变' if should_change else '不变'}", (changed != baseline) == should_change,
                        f"摘要{'变了' if changed != baseline else '没变'}")
    swapped = edit_inside_definition(source, "COMMANDS", '"crash-case-judge": command_crash_case_judge,\n    "crash-case-record": command_crash_case_record,',
                                     '"crash-case-judge": command_crash_case_record,\n    "crash-case-record": command_crash_case_judge,')
    selftest.expect("判法摘要：分派表里 crash-case-judge 与 crash-case-record 指的函数对调（闭包里的定义一个没变） ⇒ 摘要变",
                    crash_case_judging_digest_text(swapped)[0] != baseline, "摘要没变：分派表那几项没进摘要")
    with BreakSwitch("judging-digest-without-dispatch"):
        broken_baseline, broken_swapped = crash_case_judging_digest_text(source)[0], crash_case_judging_digest_text(swapped)[0]
    selftest.expect("弄坏开关 judging-digest-without-dispatch 下「分派表两项对调」那一格红（摘要没变）", broken_baseline == broken_swapped,
                    "弄坏之后摘要照样变了：这一格分不出分派表进没进摘要")
    commented = edit_inside_definition(source, "admit_experiment", "def admit_experiment(root, key):\n", "def admit_experiment(root, key):\n    # 样本\n")
    with BreakSwitch("whole-module-in-judging-digest"):
        broken_baseline, broken_commented = crash_case_judging_digest_text(source)[0], crash_case_judging_digest_text(commented)[0]
    selftest.expect("弄坏开关 whole-module-in-judging-digest 下「实验准入加一行注释」那一格红（摘要变了）", broken_baseline != broken_commented,
                    "弄坏之后摘要照样没变：这一格分不出摘要是不是整份模块")
    run_judging_digest_writing_cells(selftest, source)
    shardable_answer = edit_inside_definition(source, "command_crash_case_shardable", SHARDABLE_ANSWER_UNREGISTERED,
                                              SHARDABLE_ANSWER_UNREGISTERED.replace("return 1", "return 0"))
    with BreakSwitch("shardable-outside-judging-digest"):
        broken_baseline, broken_answer = crash_case_judging_digest_text(source)[0], crash_case_judging_digest_text(shardable_answer)[0]
    selftest.expect("弄坏开关 shardable-outside-judging-digest 下「crash-case-shardable 把没登记答成登记了」那一格红（摘要没变）",
                    broken_baseline == broken_answer, "弄坏之后摘要照样变了：这一格分不出 crash-case-shardable 进没进摘要")


# 假 cargo：-V 打版本；test 带 --include-ignored 的记下目标、过滤、续跑的三个环境变量，照控制目录里那条目标的日志与退出码打；
# 不带的（快档）打一行 2 passed。控制目录里有 touch-during-run 时，跑的过程中往它写的那个文件追加一行（造「跑的过程中输入变了」）。
FAKE_CARGO_FOR_STAGE = r'''#!/usr/bin/env bash
if [[ "${1:-}" == -V ]]; then echo "cargo 0.0.0-selftest"; exit 0; fi
target=""; filter=""; include_ignored=0; exact=0; release=0; after_separator=0
arguments=("$@")
for ((position = 0; position < ${#arguments[@]}; position++)); do
  word="${arguments[position]}"
  if [[ "$word" == -- ]]; then after_separator=1; continue; fi
  if (( after_separator )); then
    case "$word" in --include-ignored) include_ignored=1 ;; --exact) exact=1 ;; --nocapture) ;; *) filter="$word" ;; esac
  elif [[ "$word" == --test ]]; then target="${arguments[position + 1]}"
  elif [[ "$word" == --release ]]; then release=1; fi
done
printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$target" "$filter" "$include_ignored" "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY-unset}" \
  "${SINGLEFS_LAYER0_INPUT_FINGERPRINT-unset}" "${SINGLEFS_LAYER0_START_OVER-unset}" "$exact" "$release" "${SINGLEFS_LAYER0_THREADS-unset}" \
  >> "$FAKE_CARGO_CONTROL/invocations"
if (( ! include_ignored )); then echo "test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.01s"; exit 0; fi
if [[ -f "$FAKE_CARGO_CONTROL/touch-during-run" ]]; then
  echo "// 跑的过程中改的" >> "$(cat "$FAKE_CARGO_CONTROL/touch-during-run")"
  rm -f "$FAKE_CARGO_CONTROL/touch-during-run"
fi
cat "$FAKE_CARGO_CONTROL/log.$target"
exit "$(cat "$FAKE_CARGO_CONTROL/exit.$target" 2>/dev/null || echo 0)"
'''
LAYER0_STAGE_CASES = [
    ("crash-case:stream-a", "crash_enumeration_new_pool_file_creation_stream", "stream_a_full", "count-line=LAYER0 exhaustive=LAYER0 threads=LAYER0 shard=across-machines"),
    ("crash-case:stream-b", "crash_enumeration_fixed_script_stream", "stream_b_full", "count-line=LAYER0B exhaustive=LAYER0B threads=LAYER0B"),
    ("crash-case:case-c", "case_c", "case_c_full", ""),
]
PASSED_ONE_LINE = "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s"


def stream_log(prefix, worker_threads=32, resumed_slices=0, freshly_run_slices=64, result=PASSED_ONE_LINE):
    return (f"LAYER0_PROGRESS slice=1/64\nLAYER0_PARALLEL_FINISHED states=100 slices=64 worker_threads={worker_threads} configured_worker_threads=32 "
            f"worker_threads_source=environment_variable resumed_slices={resumed_slices} freshly_run_slices={freshly_run_slices} "
            f"progress_file_after_completion=deleted elapsed_seconds=1.0\n{prefix} states=100 closed_form=100 violations=0 exhaustive=true\n{result}\n")


def run_layer0_stage_cells(selftest, module):
    """门禁 54 号这一份的流程：拷进临时仓的 .claude/stage-under-test/（不在 .claude/gate.d/ 下，名字照旧是 54-layer0-replay.sh），
    假 cargo、rustc、nproc 在 PATH 最前面。核：逐条跑、续跑的三个环境变量与线程数（由 crash-case-command 交出）、判绿写标记（线程分记配的与起的）、
    快档跑一遍 checker 档包并核标记、缺一格报本次未跑不判红、第二趟全复用、只重跑输入变了的那一条、--start-over、显式设线程数、只剩 1 片要跑不误红、1 个线程跑了 64 片判红、0 passed 判红、
    cargo 退非 0 判红而别的照跑、跑的过程中输入变了不写标记、快档不带 SINGLEFS_GATE_FULL=1 时只改登记表报本次未跑、只改 54 号与只改准入模块判法之外的部分
    照样核标记而判绿（54 号不进指纹）、改准入模块的判法判红（都不退 77）。"""
    repository_root = os.path.dirname(os.path.dirname(SELFTEST_HERE))
    real_stage = os.path.join(repository_root, ".claude/gate.d/54-layer0-replay.sh")
    work = tempfile.mkdtemp(prefix="admission-selftest-stage54-")
    try:
        run_quietly(["git", "init", "-q", "-b", "master", work], GIT_IDENTITY)
        write_text(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/singlefs-harness", "crates/singlefs-checker-tier"]\n')
        write_text(os.path.join(work, "crates/singlefs-harness/Cargo.toml"), '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
        write_text(os.path.join(work, "crates/singlefs-harness/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
        write_text(os.path.join(work, "crates/singlefs-checker-tier/Cargo.toml"), '[package]\nname = "singlefs-checker-tier"\nversion = "0.0.0"\n')
        write_text(os.path.join(work, "crates/singlefs-checker-tier/src/lib.rs"), "pub fn two() -> u32 { 2 }\n")
        for _key, target, function, _conditions in LAYER0_STAGE_CASES:
            write_text(os.path.join(work, f"crates/singlefs-checker-tier/tests/{target}.rs"), f"#[test]\n#[ignore]\nfn {function}() {{}}\n")
        rows = ["54-layer0-replay.sh\tcrates/ Cargo.toml\tcommand=cargo command=rustc\t# 样本"]
        rows += [f"{key}\tcrates/ Cargo.toml\ttest=singlefs-checker-tier:{target}:{function}{' ' + conditions if conditions else ''}\t# 样本"
                 for key, target, function, conditions in LAYER0_STAGE_CASES]
        write_text(os.path.join(work, REGISTRATION_TABLE), "".join(row + "\n" for row in rows))
        os.makedirs(os.path.join(work, "research/scripts"))
        shutil.copy(module, os.path.join(work, "research/scripts/admission.py"))
        for helper in ("stage-must-run.sh", "change-touches-crates.sh", "stage-run-or-skip.sh"):
            shutil.copy(os.path.join(SELFTEST_HERE, helper), os.path.join(work, "research/scripts", helper))
        stage_copy = os.path.join(work, ".claude/stage-under-test/54-layer0-replay.sh")
        os.makedirs(os.path.dirname(stage_copy))
        shutil.copy(real_stage, stage_copy)
        write_text(os.path.join(work, ".gitignore"), ".control/\n.tools/\n")
        link_sop_copy(work)
        tools = os.path.join(work, ".tools")
        control = os.path.join(work, ".control")
        os.makedirs(control)
        write_executable(os.path.join(tools, "cargo"), FAKE_CARGO_FOR_STAGE)
        write_executable(os.path.join(tools, "rustc"), FAKE_TOOLCHAIN_SCRIPTS["rustc"])
        # PATH 里的 nproc 打 1、调用方环境里 OMP_NUM_THREADS=1：本机核数不认这两样（取 os.cpu_count() 与 CPU 亲和的小者；弄坏开关 cores-from-nproc 下认）
        write_executable(os.path.join(tools, "nproc"), "#!/usr/bin/env bash\necho 1\n")
        cores = machine_core_count(os.environ)
        environment = environment_without_build_settings({"PATH": tools + os.pathsep + os.environ.get("PATH", ""),
                                                          "CARGO_HOME": os.path.join(tools, "cargo-home"), "FAKE_CARGO_CONTROL": control,
                                                          "SINGLEFS_GATE_FULL": "1", "SINGLEFS_LAYER0_START_OVER": "1", "OMP_NUM_THREADS": "1"})
        environment.pop("SINGLEFS_LAYER0_THREADS", None)
        common_directory = os.path.join(work, ".git")

        def set_logs(stream_a=None, stream_b=None, case_c=None, exits=None):
            write_text(os.path.join(control, "log.crash_enumeration_new_pool_file_creation_stream"), stream_a or stream_log("LAYER0") + "CHECKER x=0\n")
            write_text(os.path.join(control, "log.crash_enumeration_fixed_script_stream"), stream_b or stream_log("LAYER0B"))
            write_text(os.path.join(control, "log.case_c"), case_c or PASSED_ONE_LINE + "\n")
            for target in ("crash_enumeration_new_pool_file_creation_stream", "crash_enumeration_fixed_script_stream", "case_c"):
                exit_path = os.path.join(control, f"exit.{target}")
                if os.path.exists(exit_path):
                    os.remove(exit_path)
            for target, code in (exits or {}).items():
                write_text(os.path.join(control, f"exit.{target}"), f"{code}\n")

        def invocations():
            path = os.path.join(control, "invocations")
            if not os.path.exists(path):
                return []
            with open(path, encoding="utf-8") as handle:
                lines = [line.split("\t") for line in handle.read().split("\n") if line]
            os.remove(path)
            return lines

        def stage(*arguments):
            exit_code, output, messages = run_in_environment(["bash", stage_copy, *arguments, work], environment)
            return exit_code, output + messages, invocations()

        def expected_fingerprint(key):
            _exit_code, output, _messages = run_in_environment(
                [sys.executable, os.path.join(work, "research/scripts/admission.py"), "crash-case-manifest", work, key,
                 os.path.join(control, "expected-manifest"), "--judging-digest", "--toolchain", "--build-environment"], environment)
            return output.split()[0] if output.split() else ""

        def markers():
            return sorted(name for name in os.listdir(common_directory) if name.startswith(CRASH_CASE_MARKER_PREFIX) and ".partial." not in name)

        def full_runs(calls):
            return [call for call in calls if call[2] == "1"]

        set_logs()
        exit_code, output, calls = stage("--full")
        runs = full_runs(calls)
        selftest.expect("54 号 --full：没有本地分片配置（临时仓里没有 layer0-shard-configuration-check.sh 判得过的配置）⇒ 打一行「双机分片：关」、逐条单机跑",
                        "双机分片：关" in output, f"输出尾部：{output.strip()[-600:]}")
        fingerprints = {key: expected_fingerprint(key) for key, _target, _function, _conditions in LAYER0_STAGE_CASES}
        progress_settings_ok = all(
            call[3] == os.path.join(common_directory, "singlefs-layer0-progress", fingerprints[key]) and call[4] == fingerprints[key] and call[5] == "unset"
            for call, (key, _target, _function, _conditions) in zip(runs, LAYER0_STAGE_CASES))
        selftest.expect("54 号 --full 头一趟：三条用例在 release 下逐条跑（--include-ignored --exact 各自的用例函数），各写一格标记",
                        exit_code == 0 and [(call[0], call[1], call[6], call[7]) for call in runs]
                        == [(target, function, "1", "1") for _key, target, function, _c in LAYER0_STAGE_CASES]
                        and len(markers()) == 3, f"退 {exit_code}，跑了 {runs}，标记 {markers()}，输出尾部：{output.strip()[-800:]}")
        selftest.expect("54 号 --full 设续跑的环境变量：进度目录 <common-dir>/singlefs-layer0-progress/<这条用例的指纹>、输入指纹是这条用例的；"
                        "调用方环境里的 SINGLEFS_LAYER0_START_OVER=1 在不带 --start-over 时被清掉",
                        progress_settings_ok and len(set(fingerprints.values())) == 3, f"跑的时候看到 {runs}，这几条用例的指纹 {fingerprints}")
        selftest.expect(f"54 号 --full 的线程数由 crash-case-command 交出：调用方没设 SINGLEFS_LAYER0_THREADS，用例看到的是本机核数 {cores}"
                        "（PATH 里的 nproc 打 1、OMP_NUM_THREADS=1 都不认）",
                        len(runs) == 3 and all(call[8] == str(cores) for call in runs), f"跑的时候看到 {runs}")

        def marker_text(case_name):
            names = [name for name in markers() if f".{case_name}." in name]
            if len(names) != 1:
                return ""
            with open(os.path.join(common_directory, names[0]), encoding="utf-8") as handle:
                return handle.read()
        stream_a_marker, case_c_marker = marker_text("stream-a"), marker_text("case-c")
        selftest.expect(f"54 号 --full 写的标记里线程分两格：configured_worker_threads= 记配的（没设，取本机核数 {cores}），"
                        "started_worker_threads= 记登记了 threads= 的用例起了几个（32 个工作线程），没登记的记读不到；没有旧的 worker_threads= 那一格",
                        f"\nconfigured_worker_threads=SINGLEFS_LAYER0_THREADS={cores}（没设，取本机核数），本机 {cores} 核\n" in stream_a_marker
                        and "\nstarted_worker_threads=LAYER0：32 个工作线程跑了 64 片" in stream_a_marker
                        and "\nstarted_worker_threads=读不到：这条用例没登记 threads=" in case_c_marker
                        and not any(line.startswith("worker_threads=") for line in (stream_a_marker + case_c_marker).split("\n")),
                        f"stream-a 那一格：{stream_a_marker[:600]}；case-c 那一格：{case_c_marker[:400]}")
        exit_code, output, calls = stage()
        selftest.expect("54 号快档：跑一遍 checker 档包不标 ignored 的用例（一次 cargo test -p singlefs-checker-tier），三条用例的标记都作数，判绿、不报本次未跑",
                        exit_code == 0 and all(key in output for key, _t, _f, _c in LAYER0_STAGE_CASES) and not full_runs(calls)
                        and len(calls) == 1 and calls[0][0] == "" and "本次未跑" not in output,
                        f"退 {exit_code}，cargo 调了 {calls}，输出尾部：{output.strip()[-600:]}")
        exit_code, output, calls = stage("--full")
        selftest.expect("54 号 --full 第二趟：输入没变，三条全复用、一条都不跑", exit_code == 0 and not full_runs(calls) and output.count(" 复用：") == 3,
                        f"退 {exit_code}，跑了 {full_runs(calls)}，输出尾部：{output.strip()[-600:]}")
        write_text(os.path.join(work, "crates/singlefs-checker-tier/tests/case_c.rs"), "#[test]\n#[ignore]\nfn case_c_full() { let changed = 1; }\n")
        exit_code, output, calls = stage("--full")
        selftest.expect("54 号 --full：只改了 case_c 独占的测试文件，只重跑 case_c，另两条复用",
                        exit_code == 0 and [call[0] for call in full_runs(calls)] == ["case_c"], f"退 {exit_code}，跑了 {full_runs(calls)}")
        stream_b_marker = [name for name in markers() if ".stream-b." in name]
        for name in stream_b_marker:
            os.remove(os.path.join(common_directory, name))
        exit_code, output, calls = stage()
        selftest.expect("54 号快档：删掉 stream-b 那一格标记，快档照样绿、退 0，那一条报「本次未跑」并点名 crash-case:stream-b，出路是 --full（D13 已定项 15：全量默认不在提交时跑）",
                        exit_code == 0 and "✗" not in output and "本次未跑" in output and "crash-case:stream-b" in output and "--full" in output,
                        f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
        set_logs(stream_b=stream_log("LAYER0B", worker_threads=1, resumed_slices=63, freshly_run_slices=1))
        exit_code, output, calls = stage("--full", "--start-over")
        runs = full_runs(calls)
        selftest.expect("54 号 --full --start-over：设 SINGLEFS_LAYER0_START_OVER=1；只剩 1 片要跑、起 1 个线程不误红，写回那一格",
                        exit_code == 0 and [(call[0], call[5]) for call in runs] == [("crash_enumeration_fixed_script_stream", "1")]
                        and any(".stream-b." in name for name in markers()), f"退 {exit_code}，跑了 {runs}，输出尾部：{output.strip()[-600:]}")
        for name in [name for name in markers() if ".stream-b." in name]:
            os.remove(os.path.join(common_directory, name))
        set_logs()
        exit_code, output, messages = run_in_environment(["bash", stage_copy, "--full", work], dict(environment, SINGLEFS_LAYER0_THREADS="7"))
        runs = full_runs(invocations())
        selftest.expect("54 号 --full 显式设 SINGLEFS_LAYER0_THREADS=7：用例看到 7，标记里 configured_worker_threads= 记「显式设的」",
                        exit_code == 0 and [(call[0], call[8]) for call in runs] == [("crash_enumeration_fixed_script_stream", "7")]
                        and f"configured_worker_threads=SINGLEFS_LAYER0_THREADS=7（显式设的），本机 {cores} 核" in marker_text("stream-b"),
                        f"退 {exit_code}，跑了 {runs}，输出尾部：{(output + messages).strip()[-600:]}")
        exit_code, output, _calls = run_in_environment(["bash", stage_copy, "--start-over", work], environment)
        selftest.expect("54 号 --start-over 不带 --full：退 2，说只跟 --full 一起用", exit_code == 2 and "只跟 --full 一起用" in output,
                        f"退 {exit_code}，输出「{output.strip()}」")
        stream_b_markers = [name for name in markers() if ".stream-b." in name]
        stream_b_marker_path = os.path.join(common_directory, stream_b_markers[0]) if stream_b_markers else ""
        if stream_b_marker_path:
            with open(stream_b_marker_path, encoding="utf-8") as handle:
                tampered_marker = handle.read().replace("exhaustive=true", "exhaustive=false")
            write_text(stream_b_marker_path, tampered_marker)
        set_logs(stream_b=stream_log("LAYER0B", worker_threads=1))
        exit_code, output, calls = stage("--full")
        selftest.expect("54 号 --full：这批输入那一格在而不作数就重跑，重跑判红时删掉那一格（先绿后红，前一趟的不再作数）",
                        bool(stream_b_marker_path) and exit_code == 1 and [call[0] for call in full_runs(calls)] == ["crash_enumeration_fixed_script_stream"]
                        and not os.path.exists(stream_b_marker_path),
                        f"改之前 stream-b 那一格{'在' if stream_b_marker_path else '不在（上一格没写成）'}；退 {exit_code}，跑了 {full_runs(calls)}，标记 {markers()}")
        red_cells = [
            ("跑了 64 片却只起 1 个线程", {"stream_b": stream_log("LAYER0B", worker_threads=1)}, None, "只起了 1 个工作线程"),
            ("过滤之后一条用例都没跑（0 passed）",
             {"stream_b": stream_log("LAYER0B", result="test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s")},
             None, "1 passed"),
            ("cargo test 退非 0", {}, {"crash_enumeration_fixed_script_stream": 101}, "cargo test 退非 0"),
        ]
        for label, logs, exits, phrase in red_cells:
            for name in [name for name in markers() if ".stream-b." in name]:
                os.remove(os.path.join(common_directory, name))
            set_logs(stream_b=logs.get("stream_b"), exits=exits)
            exit_code, output, calls = stage("--full")
            selftest.expect(f"54 号 --full：stream-b {label} ⇒ 这一条判红、不写标记，退 1",
                            exit_code == 1 and phrase in output and not any(".stream-b." in name for name in markers())
                            and [call[0] for call in full_runs(calls)] == ["crash_enumeration_fixed_script_stream"],
                            f"退 {exit_code}，跑了 {full_runs(calls)}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
        for name in [name for name in markers() if ".case-c." in name]:
            os.remove(os.path.join(common_directory, name))
        set_logs(exits={"crash_enumeration_new_pool_file_creation_stream": 101})
        for name in [name for name in markers() if ".stream-a." in name]:
            os.remove(os.path.join(common_directory, name))
        exit_code, output, calls = stage("--full")
        selftest.expect("54 号 --full：stream-a 判红之后接着跑下一条，stream-b 与 case-c 照样判绿、写标记",
                        exit_code == 1 and [call[0] for call in full_runs(calls)] == [target for _k, target, _f, _c in LAYER0_STAGE_CASES]
                        and not any(".stream-a." in name for name in markers())
                        and any(".stream-b." in name for name in markers()) and any(".case-c." in name for name in markers()),
                        f"退 {exit_code}，跑了 {full_runs(calls)}，标记 {markers()}")
        set_logs()
        write_text(os.path.join(control, "touch-during-run"), os.path.join(work, "crates/singlefs-checker-tier/src/lib.rs"))
        exit_code, output, calls = stage("--full")
        selftest.expect("54 号 --full：跑的过程中共用的 src/lib.rs 被改了 ⇒ 那一条判红、不写标记，出路里列出变了的文件",
                        exit_code == 1 and "跑的过程中它的输入变了" in output and "crates/singlefs-checker-tier/src/lib.rs" in output
                        and not any(".stream-a." in name for name in markers()), f"退 {exit_code}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")

        run_layer0_stage_shard_cells(selftest, work, control, stage, full_runs, markers, set_logs, expected_fingerprint, common_directory)
        run_crash_case_command_shard_switch_cells(selftest, work)

        # 范围那一问：只改登记表、只改 54 号、只改准入模块的改动，快档不带 SINGLEFS_GATE_FULL=1 照样核标记（不退 77）
        set_logs()
        stage("--full")
        run_quietly(["git", "-C", work, "add", "-A"])
        run_quietly(["git", "-C", work, "commit", "-q", "-m", "样本"], GIT_IDENTITY)
        quick_environment = {name: value for name, value in environment.items() if name != "SINGLEFS_GATE_FULL"}

        def quick_tier_without_forcing():
            exit_code, output, messages = run_in_environment(["bash", stage_copy, work], quick_environment)
            invocations()
            return exit_code, output + messages
        exit_code, output = quick_tier_without_forcing()
        selftest.expect("54 号快档（不带 SINGLEFS_GATE_FULL=1）：提交之后一个改动都没有，范围那一问答「没碰」，退 77", exit_code == 77,
                        f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
        every_case = [key for key, _target, _function, _conditions in LAYER0_STAGE_CASES]
        copied_module = os.path.join(work, "research/scripts/admission.py")
        # (说明, 改哪份, 怎么改, 该判红点名的用例；空表是标记照样作数、判绿)
        scope_cells = [
            ("只改登记表（stream-b 那一行第三列去掉 threads=LAYER0B）", os.path.join(work, REGISTRATION_TABLE),
             lambda text: text.replace(" threads=LAYER0B", "", 1), ["crash-case:stream-b"]),
            ("只改 54 号（加一行注释；54 号不进指纹）", stage_copy, lambda text: text + "# 样本：只改 54 号\n", []),
            ("只改准入模块判法之外的部分（文件尾加一行注释）", copied_module, lambda text: text + "# 样本：只改准入模块\n", []),
            ("只改准入模块的判法（judge_worker_threads 把「至少两片」改成「至少三片」）", copied_module,
             lambda text: edit_inside_definition(text, "judge_worker_threads", "freshly_run_slices >= 2 and", "freshly_run_slices >= 3 and"), every_case),
        ]
        for label, path, change, named_cases in scope_cells:
            with open(path, encoding="utf-8") as handle:
                original_text = handle.read()
            try:
                changed_text = change(original_text)
            except ValueError as error:
                selftest.expect(f"54 号快档：{label} 这一格的改动做得出来", False, str(error))
                continue
            write_text(path, changed_text)
            exit_code, output = quick_tier_without_forcing()
            write_text(path, original_text)
            if named_cases:
                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记，退 0 而报「本次未跑」并点名 {'、'.join(named_cases)}，不退 77",
                                exit_code == 0 and "没有作数的全绿标记" in output and "本次未跑" in output and all(key in output for key in named_cases),
                                f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
            else:
                selftest.expect(f"54 号快档（不带 SINGLEFS_GATE_FULL=1）：{label} ⇒ 照样核标记（不退 77），三条标记都作数，判绿、不报本次未跑",
                                exit_code == 0 and all(key in output for key in every_case) and "没有作数的全绿标记" not in output and "本次未跑" not in output,
                                f"退 {exit_code}，输出尾部：{output.strip()[-600:]}")
    except (OSError, shutil.Error) as error:
        selftest.expect("54 号这一份拷得进临时仓、跑得起来", False, f"{error}（真仓的 54 号在 {real_stage}）")
    finally:
        shutil.rmtree(work, ignore_errors=True)


# 假的分片配置检查：退 0（能分片）；假的驱动脚本：--merged-log 时记下参数与续跑开关，把控制目录里 stream-a 那一份日志写进日志文件，退出码照控制目录
FAKE_SHARD_CONFIGURATION_CHECK = "#!/usr/bin/env bash\necho \"样本配置，第二台是样本\"\n"
FAKE_SHARD_DRIVER = r'''#!/usr/bin/env bash
printf '%s\t%s\n' "$*" "${SINGLEFS_LAYER0_START_OVER-unset}" >> "$FAKE_CARGO_CONTROL/driver-invocations"
[[ "$1" == --merged-log ]] || exit 9
cat "$FAKE_CARGO_CONTROL/log.crash_enumeration_new_pool_file_creation_stream" > "$5"
exit "$(cat "$FAKE_CARGO_CONTROL/driver-exit" 2>/dev/null || echo 0)"
'''


C561_SIGMA_STATES = 262144


def c561_sigma_log(exhaustive="true", worker_threads=32):
    """合成的 crash-case:c561-sigma-full 一趟 --full 日志：用例打的计数行与同形的线程行（照 crates/singlefs-checker-tier/tests/
    record_checker_judges_absence_by_the_persisted_set.rs 的 count_line 与线程行）。"""
    return (f"LAYER0_PARALLEL_FINISHED states={C561_SIGMA_STATES} slices=64 worker_threads={worker_threads} configured_worker_threads=32 "
            f"worker_threads_source=available_parallelism resumed_slices=0 freshly_run_slices=64 progress_file_after_completion=none elapsed_seconds=1.0\n"
            f"C561_SIGMA_FULL states={C561_SIGMA_STATES} closed_form_states={C561_SIGMA_STATES} exhaustive={exhaustive} "
            f"record_claimed_state_missing_unit=0\n{PASSED_ONE_LINE}\n")


def run_real_crash_case_row_cells(selftest, rows):
    """真仓登记表里两条崩溃枚举用例的第三列拿合成日志判：c561-sigma-full 的计数行不带 exhaustive=true、1 个线程跑了 64 片都判红，两样都对判绿
    （只登记 count-line= 时这两种都判绿）；crash-injection-fast-tier 登记着，计数行 CRASH_INJECTION_FINISHED 恰好一行判绿、两行判红，
    跑完那一行报 1 个线程跑了 24 片判红（它登记了 threads=CRASH_INJECTION_FINISHED；弄坏开关 threads-skip-self-contained-finish-line 下这一格转绿）；
    它登记的 threads-variable= 是 crates/singlefs-checker-tier/src/crash_injection.rs 里的 CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE。"""
    fast_tier_line = "CRASH_INJECTION_FINISHED seeds=[1,25) slices=24 worker_threads=4 elapsed_seconds=1.0\n"
    cells = [
        ("crash-case:c561-sigma-full", "两行都对", c561_sigma_log(), True),
        ("crash-case:c561-sigma-full", "计数行 exhaustive=false（状态数不等于闭式）", c561_sigma_log(exhaustive="false"), False),
        ("crash-case:c561-sigma-full", "本机 32 核、线程数没显式设，1 个线程跑了 64 片", c561_sigma_log(worker_threads=1), False),
        ("crash-case:crash-injection-fast-tier", "计数行恰好一行", fast_tier_line + PASSED_ONE_LINE + "\n", True),
        ("crash-case:crash-injection-fast-tier", "计数行打了两行", fast_tier_line * 2 + PASSED_ONE_LINE + "\n", False),
        ("crash-case:crash-injection-fast-tier", "本机 32 核、线程数没显式设，跑完那一行报 1 个线程跑了 24 片",
         fast_tier_line.replace("worker_threads=4", "worker_threads=1") + PASSED_ONE_LINE + "\n", False),
    ]
    for key, label, log_text, expected_green in cells:
        try:
            problems = judge_crash_case_log(crash_case_of_key(rows, key), log_text, 32, False)[0]
        except RegistrationError as error:
            problems = [f"真仓登记表里读不出这一条：{error}"]
        green = not problems
        selftest.expect(f"真仓登记的 {key}：{label} ⇒ {'绿' if expected_green else '红'}", green == expected_green,
                        f"判成{'绿' if green else '红'}：{problems}")
    one_thread_fast_tier = cells[-1][2]
    with BreakSwitch("threads-skip-self-contained-finish-line"):
        problems = judge_crash_case_log(crash_case_of_key(rows, "crash-case:crash-injection-fast-tier"), one_thread_fast_tier, 32, False)[0]
    selftest.expect("弄坏开关 threads-skip-self-contained-finish-line 下「crash-injection 快档 1 个线程跑了 24 片」那一格红（判绿）", not problems,
                    f"弄坏之后仍判红：{problems}")
    source_path = os.path.join(os.path.dirname(os.path.dirname(SELFTEST_HERE)), "crates/singlefs-checker-tier/src/crash_injection.rs")
    try:
        with open(source_path, encoding="utf-8") as handle:
            read_variables = re.findall(r'CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =\s*"([A-Z0-9_]+)"', handle.read())
        registered = crash_case_of_key(rows, "crash-case:crash-injection-fast-tier").thread_variable
    except (OSError, RegistrationError) as error:
        read_variables, registered = [], f"读不出：{error}"
    selftest.expect("真仓登记的 crash-case:crash-injection-fast-tier 的 threads-variable= 就是用例读的那个变量", read_variables == [registered],
                    f"登记的 {registered}，{source_path} 里读的 {read_variables}")


def shard_switch_seen_by_the_case(work, caller_shard_switch):
    """照 crash_case_launch 交的命令起一趟（cargo 起往后换成打 SINGLEFS_LAYER0_SHARD 的 sh），调用方环境里设着 caller_shard_switch：交回那一趟看到的值。"""
    case = crash_case_of_key(read_registration_rows(work), "crash-case:stream-b")
    environment = dict(os.environ, SINGLEFS_LAYER0_THREADS="2", SINGLEFS_LAYER0_SHARD=caller_shard_switch)
    command = crash_case_launch(work, case, "0" * 64, False, environment)[4]
    launched = command[:command.index("cargo")] + ["sh", "-c", 'printf "%s" "${SINGLEFS_LAYER0_SHARD-unset}"']
    return subprocess.run(launched, env=environment, capture_output=True, text=True, check=False).stdout


def run_crash_case_command_shard_switch_cells(selftest, work):
    """crash-case-command 交的命令起用例时清掉调用方环境里的 SINGLEFS_LAYER0_SHARD（单机跑的那一趟不许只跑一片或去 merge）；
    弄坏开关 keep-caller-shard-switch 下那一格转红。"""
    seen = shard_switch_seen_by_the_case(work, "0/2")
    selftest.expect("crash-case-command：调用方环境里设着 SINGLEFS_LAYER0_SHARD=0/2 ⇒ 起的用例看不到它", seen == "unset", f"用例看到的是「{seen}」")
    with BreakSwitch("keep-caller-shard-switch"):
        seen = shard_switch_seen_by_the_case(work, "0/2")
    selftest.expect("弄坏开关 keep-caller-shard-switch 下「调用方设着分片开关」那一格红（用例看到 0/2）", seen == "0/2",
                    f"弄坏之后用例看到的是「{seen}」：这一格分不出命令清没清它")


def run_layer0_stage_shard_cells(selftest, work, control, stage, full_runs, markers, set_logs, expected_fingerprint, common_directory):
    """54 号 --full 在分片开着时：登记了 shard=across-machines 的 stream-a 交给驱动脚本 --merged-log（参数是键、树根、这批输入的指纹、日志文件），
    另两条照旧单机跑；驱动脚本交回的日志照单机的判法判、写同一格标记；驱动脚本退非 0 判红、删那一格。假的检查与驱动脚本放进临时仓，跑完删掉。"""
    check_path = os.path.join(work, "research/scripts/layer0-shard-configuration-check.sh")
    driver_path = os.path.join(work, "research/scripts/layer0-shard-run.sh")
    invocations_path = os.path.join(control, "driver-invocations")
    rows = read_registration_rows(work)
    sharded_case, unsharded_case = crash_case_of_key(rows, "crash-case:stream-a"), crash_case_of_key(rows, "crash-case:stream-b")

    def fingerprints_of_both():
        return crash_case_manifest(work, sharded_case, [])[1], crash_case_manifest(work, unsharded_case, [])[1]

    try:
        before = fingerprints_of_both()
        with BreakSwitch("shard-driver-outside-manifest"):
            broken_before = fingerprints_of_both()
        write_executable(check_path, FAKE_SHARD_CONFIGURATION_CHECK)
        write_executable(driver_path, FAKE_SHARD_DRIVER)
        after = fingerprints_of_both()
        with BreakSwitch("shard-driver-outside-manifest"):
            broken_after = fingerprints_of_both()
        selftest.expect("输入清单：放进双机分片的驱动脚本与配置判法 ⇒ 登记了 shard=across-machines 的 stream-a 指纹变、没登记的 stream-b 不变",
                        before[0] != after[0] and before[1] == after[1], f"stream-a {before[0][:16]} → {after[0][:16]}，stream-b {before[1][:16]} → {after[1][:16]}")
        selftest.expect("弄坏开关 shard-driver-outside-manifest 下「放进驱动脚本」那一格红（stream-a 指纹没变）", broken_before[0] == broken_after[0],
                        "弄坏之后 stream-a 的指纹照样变了：这一格分不出驱动脚本进没进清单")
        for name in markers():
            os.remove(os.path.join(common_directory, name))
        set_logs()
        exit_code, output, calls = stage("--full")
        driver_calls = []
        if os.path.exists(invocations_path):
            with open(invocations_path, encoding="utf-8") as handle:
                driver_calls = [line.split("\t") for line in handle.read().split("\n") if line]
            os.remove(invocations_path)
        expected_arguments = f"--merged-log crash-case:stream-a {work} {expected_fingerprint('crash-case:stream-a')} "
        selftest.expect("54 号 --full 分片开着：打一行「双机分片：开」；stream-a（登记了 shard=across-machines）交给驱动脚本 --merged-log，"
                        "另两条单机跑；三条都判绿、各写一格",
                        exit_code == 0 and "双机分片：开" in output and len(driver_calls) == 1 and driver_calls[0][0].startswith(expected_arguments)
                        and driver_calls[0][1] == "unset"
                        and [call[0] for call in full_runs(calls)] == ["crash_enumeration_fixed_script_stream", "case_c"] and len(markers()) == 3,
                        f"退 {exit_code}，驱动脚本被调 {driver_calls}，cargo 跑了 {full_runs(calls)}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
        for name in [name for name in markers() if ".stream-a." in name]:
            os.remove(os.path.join(common_directory, name))
        write_text(os.path.join(control, "driver-exit"), "1\n")
        exit_code, output, calls = stage("--full")
        os.remove(os.path.join(control, "driver-exit"))
        if os.path.exists(invocations_path):
            os.remove(invocations_path)
        selftest.expect("54 号 --full 分片开着：驱动脚本退非 0 ⇒ stream-a 判红、不写标记，另两条复用",
                        exit_code == 1 and "stream-a" in output and not any(".stream-a." in name for name in markers()) and not full_runs(calls),
                        f"退 {exit_code}，cargo 跑了 {full_runs(calls)}，标记 {markers()}，输出尾部：{output.strip()[-600:]}")
    finally:
        for path in (check_path, driver_path):
            if os.path.exists(path):
                os.remove(path)
    set_logs()
    stage("--full")


# manifest 子命令（不带 --build-environment）同一套步骤的另一份实现，只给自证当对照：它不 import 本模块，用的是 sha256sum 与 sort。
INDEPENDENT_MANIFEST_SCRIPT = r'''
set -uo pipefail
root="$1"; stage_script="$2"
existing_files=()
while IFS= read -r -d '' listed_file; do
  if [[ -f "$root/$listed_file" ]]; then existing_files+=("$listed_file"); fi
done < <(git -C "$root" ls-files -z --cached --others --exclude-standard -- crates/ Cargo.toml | LC_ALL=C sort -zu)
( cd "$root" && sha256sum -- "${existing_files[@]}" ) || exit 1
stage_script_hash="$(sha256sum < "$stage_script" | cut -d' ' -f1)"
toolchain_versions="$(cd "$root" && cargo -V && rustc -V)" || exit 1
toolchain_versions_hash="$(printf '%s\n' "$toolchain_versions" | sha256sum | cut -d' ' -f1)"
printf '%s  %s\n' "$stage_script_hash" "<判它的 54 号：54-layer0-replay.sh>"
printf '%s  %s\n' "$toolchain_versions_hash" "<工具链：${toolchain_versions//$'\n'/；}>"
'''

COMMANDS = {
    "gate-reuse": command_gate_reuse,
    "stage-fingerprint": command_stage_fingerprint,
    "stage-marker-check": command_stage_marker_check,
    "stage-marker-write": command_stage_marker_write,
    "experiment": command_experiment,
    "paths": command_paths,
    "manifest": command_manifest,
    "keys": command_keys,
    "gate-preconditions": command_gate_preconditions,
    "gate-record-environment": command_gate_record_environment,
    "crash-cases": command_crash_cases,
    "crash-case-manifest": command_crash_case_manifest,
    "crash-case-marker-check": command_crash_case_marker_check,
    "crash-case-command": command_crash_case_command,
    "crash-case-judge": command_crash_case_judge,
    "crash-case-record": command_crash_case_record,
    "crash-case-marker-path": command_crash_case_marker_path,
    "crash-case-shardable": command_crash_case_shardable,
}


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest()
    if not arguments or arguments[0] not in COMMANDS:
        say("  ✗ 用法：admission.py {gate-reuse|gate-preconditions|gate-record-environment|experiment|paths|manifest|keys|crash-cases|"
            "crash-case-manifest|crash-case-marker-check|crash-case-command|crash-case-judge|crash-case-record|crash-case-marker-path|"
            "crash-case-shardable} … 或 --selftest")
        say("     → 怎么办：各子命令的参数见文件头")
        return EXIT_REGISTRATION_ERROR
    return COMMANDS[arguments[0]](arguments[1:])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
