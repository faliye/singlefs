"""一条命令是不是重型测试、属于哪一类。

heavy-test-guard.sh（执行前拒绝）与看门狗（research/scripts/agent-watch.py，在进程这一层）共用这一份判定，各自按文件路径用 importlib 导入；
同目录的 lib_shell_words.py（切词、剥前缀与包装）由这份自己导入，调用方从 `shell_words` 属性拿同一份。
谁能跑哪一类、要带什么前缀，是 heavy-test-guard.sh 的事，不在这里。

入口：
  classify(words, directory, environment=None) -> HeavyTest | None
      一条已经剥掉前缀与包装的命令：words[0] 是命令词（照写的样子，没取 basename），其后是参数；directory 是它的当前目录，认不出时 None；
      environment 是这条命令看得到的、命令文本里设过的变量（lib_shell_words 的 CommandAtPosition.environment），认别名要用。
      命令词带通配的先照 bash 展开再判（下面「命令词带通配」）；花括号与参数里的通配同样先展开再判（下面「花括号与参数里的通配」）。
  command_under_launcher(words, directory=None) -> (list[str], str | None) | None
      words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序（/usr/bin/time、flock、rustup run、chrt、prlimit、systemd-run、strace、perf）时，
      剥掉它交回它起的那条命令与那条命令的当前目录（None 是不变）；调用方把交回的命令再交给 lib_shell_words 切一遍、逐条判。
      短选项合写（`-fo <文件>`、`-xw 10`、`-qu <名>`）逐个字母查那张表：头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
      长选项查要值与不要值两张表（照各自的 --help 列全），值另起一个词的（`--expand-environment no`、`--output <文件>`）跳过那个值，
      只写前缀的（getopt_long 认唯一的前缀，`--outp`）按前缀认；
      systemd-run 的 `-E NAME=VALUE` / `--setenv=NAME=VALUE`、`-p Environment=…` / `--property=Environment=…`（`EnvironmentFile=<文件>` 按内容读，
      读不了当它没设）、strace 的 `-E NAME=VALUE` / `--env=NAME=VALUE` 设的变量写在交回那条命令的最前面
      （再切一遍时当它的环境变量，认别名要用；strace 的 `-E NAME` 只清变量，不带）。
      这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
      research/scripts/detached-run.sh 的 `start [--label <名>] [--log <文件>] [--stop-timeout <秒>] -- <命令…>`（参数里的 --force 照它的 preflight 每一个都摘）同样剥掉，交回 `--` 之后那条命令，
      当前目录与环境照外面那一条（它把调用方的 SINGLEFS_HEAVY_TESTS 与当前目录带进单元）；list、watch、stop 与没写 `--` 的不剥（command_under_detached_run）。
      认它照 lib_shell_words 的 detached_run_script：名字叫 detached-run.sh（路径不论）、解到的真实路径落在 research/scripts/detached-run.sh（符号链接）、
      文件开头有签名行（行首 `# detached-run-script-signature:`，改名或拷走的一份）；后两种按 directory 解相对路径、读盘，文件还不在就认不出
      （同一条命令里先 cp 再起的由 heavy-test-guard.sh 拿它记下的写出内容另判）。
  classify_process(argv, cwd) -> list[HeavyTest]
      一个在跑的进程：argv 是 /proc/<pid>/cmdline 按 NUL 切开的那一串，cwd 是 /proc/<pid>/cwd 指向的目录。
      先照 lib_shell_words 剥掉 bash / sh 起脚本、bash -c、nice / timeout / env / taskset、capped.sh N 这类包装，
      再照 command_under_launcher 剥掉 /usr/bin/time 这一类，逐条 classify；空列表就是不重型。
  runs_compiled_code(words, directory) -> str | None
      一条已经剥掉前缀与包装的命令会不会跑编译出来的代码：cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算），
      或直接执行 cargo 编出来的二进制（测试二进制，与名字里带 target 的目录底下 debug / release 里的）；会就交一句说明。
      heavy-test-guard.sh 拿它判子 agent 跑这一类经没经 research/scripts/run-with-memory-cap.sh（与重型不重型无关）。
  judged_by_name(word, directory) -> bool
      这个命令词是不是按名字判的仓内脚本（.claude/gate.d/ 下的阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在它们的仓内位置上）：
      算不算重型、带什么参数才算，都在名字那一格判完了，要读脚本正文的一方不再读进去。
  selftest_judged_light(words, directory, environment=None) -> bool
      这一条是不是登记表里记成不起重活、又写成单独一个 --selftest 的自证（classify_selftest 判它放行）：要读脚本正文的一方同样不再读进去
      （正文里平常那条路上起的 cargo 不是自证跑的）。
  selftest_registry_problems(path, root) -> list[str]
      登记表写坏的行（判定认不出、理由太短、不在三处目录里、重复）与指到 root 下不存在的文件的行，逐条一句；空表是没毛病。
  stage_sample_run_allowed(stage, word, arguments, directory) -> bool
      把一道门禁阶段直接指到它的样本根（见下面「门禁样本」）：放行就为真，classify 交 None；要读脚本正文的一方本来就不读阶段的正文。
  python3 lib_heavy_tests.py --selftest

认的输入：cargo 命令行（test / t、run / r，以及起测试的 nextest run、miri、llvm-cov、hack、mutants）、按名字认的脚本与门禁阶段、
qemu-system-*、herd7，以及直接执行的测试二进制 `<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制哈希>`。
**checker 档按包判，是唯一的判法**（D13（验证路线） 已定项 15，.claude/rules/verification.md「定义与名字」）：cargo 起测试、范围里有
checker 档包 singlefs-checker-tier（-p 点名它，-p 的值认光名字、name@version、pkgid URL 与通配；不挑包而范围含它；当前目录在它里面裸跑），
不管挑哪个目标（--lib、--doc、--bin、--test、--tests 都算：整个包都是 checker 档，库单测与装置二进制的内联单测也在内），算重型；
直接执行它的测试二进制（名字是它 tests/ 下的测试目标、src/bin/ 下的装置、或库本身 singlefs_checker_tier）也算。
双机分片的驱动脚本 research/scripts/layer0-shard-run.sh（--selftest 照自证登记表判，见下面「自证」一段）按它开跑要读的那一行登记判（shard_driver_runs_checker_tier）：参数里的树根下
.claude/gate.d/stage-inputs.tsv 里键那一行登记的包是 checker 档包、带 shard=across-machines 的算 checker 档；登记的是别的包的不算
（驱动自检 research/scripts/layer0-shard-run-selftest.sh 在 mktemp 建的临时仓里登记的替身用例是 harness 档包，自检一趟趟起的驱动与假 cargo 都不算）；
树根里没有这一行、没登记分片的不算（驱动开跑就拒，什么都不跑）；键不是字面写的 crash-case:<名>、树根解不出或登记读不了的照算。
门禁 checker-tier-crates-mutation-replay.sh 的双机分片驱动 research/scripts/mutation-shard-run.sh（--selftest 照自证登记表判）直接起就算重型，
与那一道同一类（crates-mutation-stage）：它在两台各跑一趟那一道；只由那一道调，那一道本身已按名字算。
不按「只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE」判：执行前的闸看不见进程环境，开着它也能拿登记成 checker 档的用例在本机真跑两片。
harness 档（singlefs-harness）、core、format、池级 checker（singlefs-checker）的测试随时跑，不算重型；工作区全量（--workspace / --all、
工作区根裸跑、范围是全部成员）算「全量测试」。
别名：--config 里 alias.<名> 或环境变量 CARGO_ALIAS_<名> 定的、子命令就是它的，看得到值就展开再判；值读不出（TOML 写坏了）按 checker 档算。
libtest 参数里 --list 当选项出现的（只列用例、一条都不跑）不算重型；跟在带一个值的 libtest 选项（--skip、--logfile、
--test-threads、--format、--color、-Z、--shuffle-seed：LIBTEST_OPTIONS_WITH_VALUE）后面的 --list 是那个选项的值，照样全跑，不算只列；
libtest 在 `--` 处停止认选项，`--` 之后的 --list 是过滤词，照样全跑，不算只列。
门禁阶段按文件名认（.claude/gate.d/ 顶层的 *.sh，lib-*.sh 与 lib/ 底下的共用库不是阶段；STAGE_KIND）：重型的是 checker-tier-layer0-replay.sh、
checker-tier-qemu-device-streams.sh、checker-tier-lkmm.sh、checker-tier-crates-mutation-replay.sh、checker-tier-research-build-and-replay.sh、
checker-tier-broad-mutation.sh（广谱变异的提交门禁：判广谱变异驱动库目录里那一版是不是这一次的暂存树、每一条变异都有结论又没有没红的；
它自己只读库目录、起驱动的 --summary，驱动由主 agent 带前缀先跑；与驱动同归「广谱变异」一类，kind broad-mutation-stage，放行表在 heavy-test-guard.sh）、
checker-tier-formal-verification.sh（起 Verus 这个外部验证器、编装置逐行重放钉住的红例，照 .claude/rules/verification.md 的归类判据归 checker 档；类名「Verus」）；
直接起的 verus 与 cargo verus 不算重型：写证明的实现员改一处验一次，验一个 crate 是秒级（接回可行性那一趟量到 8–21 秒），与 cargo build / clippy 同一类，
认成重型就验不了自己写的证明；只有门禁那一道（逐格验、空真与必红世界逐处起 Verus、编装置）算。
这一道的两份判法脚本 research/scripts/verus-proof-check.py、research/scripts/e165-pin-red-examples.py 直接起（verus_judge_use）：先认出是这两份，再一律算这一类，
只放过字面写明的只读用法：verus-proof-check.py 的第一个参数是 toolchain（只读安装、Cargo.lock 与 cargo 缓存）、fingerprint、tool-versions 之一，
e165-pin-red-examples.py 照它自己的参数切法落到追加或 --check 的；参数里有没展开的 `$` 或反引号（变量、命令替换、for 循环的变量、函数转的 "$@"）的一律算，
不带参数与认不出的子命令同样算（接受的误拒）。判参数之前照两份脚本开头的 preflight 摘掉每一个整词 --force（preflight 在 main 之前从 sys.argv 里摘：
`--replay --target --force --check` 真跑的是 --replay，`--force tool-versions` 真跑的是 tool-versions）。
认这两份照广谱变异那一套（verus_judge_script）：名字与位置（当前目录与命令文本里设的 PYTHONPATH 各试一遍，
`python3 -m verus-proof-check` 由切词交成这个名字）、命令词解到的文件的真实路径落在 research/scripts/<那一份>（符号链接）。
`python3 -c '…'` 的代码里出现这两份的名字（verus-proof-check、e165-pin-red-examples：文件名与 __import__、importlib.import_module、runpy.run_module 的模块名都含它）
就一律算这一类，不看代码之后的词（verus_judge_code_use：代码自己能改 sys.argv、直接调函数，执行前判不出）；只放过每一句都是 print(<字符串字面量>…) 的代码
（ast 判，关键字参数的值也只许是字面量；别的写法哪怕只读也算，接受的误拒）。仓内位置上那一份的 --selftest 照自证登记表判，改名链接与 PYTHONPATH 起的 --selftest 照旧算提交时才跑的检查，
python -c 起的算这一类。
罩不住的：改名拷贝（两份没有签名行）、同一条命令里先 ln -s / cp 再起的、`env -C` 与目录放变量、`script -qc`、xargs（lib_shell_words.py 那一侧的老口子）；
python -c 的代码里把名字拼出来的（"verus-" + "proof-check.py"）、`python3 - < 脚本` 与 heredoc 喂给 python 的代码（与广谱变异那一段同一类老口子）。
.claude/scripts/lkmm.sh 只带 --herd7-version 一个参数的（只找 herd7、打它版本输出的第一行；门禁准入登记 stage-inputs.tsv 的 environment= 起的就是它）不算，别的用法照旧算 herd7。
最后那一道（research 构建与单测、实验复跑两道合成）只有复跑那一格是重型：--check 只点名 research-unit-tests 的不算全部实验复跑。带 --list、--list-items 或 --tool-versions 的
（只打格名表、逐项点名的清单、工具版本）哪一道都不算；带 --write 的（生成）不算。
另一类「提交时才跑的检查」（COMMIT_CHECK_KINDS，类名 COMMIT_CHECK_CATEGORY；用户 2026-09-28 定：扫仓的门禁与各道 lint 只在提交前由主 agent 跑的
research/scripts/gate-staged.sh 跑）：不重型的门禁阶段；.claude/singlefs-ai-sop/scripts/ 下的 doc-lint.sh、gate-lint.sh、shell-lint.sh、preflight-lint.py、
gate-overlap.py（--list 查清单不算）、rules-lint.sh、hooks-registered.sh、stage-selftest.sh；research/scripts/gate-structure-check.py（它的 --selftest 照下一段判）；
自证里起重活的、没登记的那几份（下一段）。
谁能跑这一类由 heavy-test-guard.sh 定（与重型同一套放行）；看门狗 research/scripts/agent-watch.py 在进程这一层不报这一类。
自证（classify_selftest）：research/scripts/、.claude/hooks/ 下脚本的 --selftest 与 .claude/gate.d/lib/ 下共用库的直接执行（就是跑它的自证），
按同目录的登记表 selftest-registry.tsv 判（上游 test-script-discipline.md「测试脚本自己也要被测试证明有效」：自证用假的驱动、不起编译与重活，拦重型测试的闸不拦自证）。
表里一行一份：light（自证不起 cargo、编译、虚机、herd7、Verus）而命令写成单独一个 --selftest（门禁共用库：不带参数或单独一个 --selftest）的，不算哪一类，
谁都能跑，selftest_judged_light 为真；heavy 的照旧算提交时才跑的检查（kind commit-check-selftest）；heavy-if-set:<变量> 在命令里把那个变量设成非空时算 heavy，
没设算 light。表外的、带了 --selftest 之外参数的（`mutate.sh <表> --selftest` 这种，脚本只认头一个参数时跑的是正事）照旧算提交时才跑的检查（kind
commit-check-selftest-unregistered，拒绝说明写怎么登记）。表读不了、某一行写坏了，那几份按表外算。vm-bench.sh --selftest 起虚机，按名字照旧算 QEMU。
认脚本照 located_in：命令词正好落在三处目录之一，表里的键是「目录/文件名」；命令词解到的真实路径（符号链接按指到的算）还要正是登记表所在仓里键指的那一份
（selftest_registered_file），别的树（worktree、仓副本、临时仓）里路径尾部相同的脚本、相对路径而当前目录认不出的、路径写在变量里的不按这一行判，照旧算提交时才跑的检查
（kind commit-check-selftest-unregistered，拒绝说明写「跑本仓那一份，写它在本仓的字面路径」）。命令词带通配（*、?、[）的照字面不当自证（SELFTEST_PATTERN_CHARACTERS）：
classify 先照 bash 展开、按展开出来的那几份判（下面「命令词带通配」），照字面判时它不是哪一份自证；case 的模式行（`.claude/gate.d/lib/*.py) …`）被切成命令时也是这样的词，
照字面不当成起门禁共用库（展开出来是登记成轻的那几份，放行）。不带通配的 case 模式行（`research/scripts/x.sh) …`）照旧被切成一条命令（lib_shell_words.py 不认 case，老口子）。
门禁样本（上游 test-script-discipline.md「测试脚本自己也要被测试证明有效」认的第三种自证形态：红绿样本目录）：同一份登记表里键是 .claude/gate.d/<阶段>.sh 的行登记
那一道的样本档起不起重活，判定写 samples-light（样本档只读文本与录好的输出）、samples-light-marker:<相对样本根的路径>（样本根放了这个假工具标记，阶段自己走假工具那一支）、
samples-heavy（样本档本身起真 cargo、Verus 这类重活）；逐道读阶段文件头的样本说明判。两种起法放行，不算哪一类、谁都能跑：
  ① 上游 .claude/singlefs-ai-sop/scripts/stage-selftest.sh 喂一个阶段目录（参数里写的，不写是它自己算的那一个）：命令词解到的真实路径正是本仓那一份
     （stage_selftest_is_own_copy；放行信的是它只喂样本，正文不读），那个目录此刻在，里面有样本目录（fixtures/<阶段>/）的每一道
     都登记成 samples-light 或 samples-light-marker:，阶段文件与这份文件所在仓里那一份逐字节相同，
     样本目录（fixtures/<阶段>/）与这份文件所在仓 HEAD 加暂存区里那一道的样本逐字节相同（sample_directory_changes：多出的文件——连同 setup.sh 与没跟踪的新文件——、
     少了的、内容或执行位不同的都算改动，拿 git ls-files -s 的对象号比；样本改完还没暂存的照拒，暂存之后才放行）（stage_selftest_problems）。真仓的 .claude/gate.d 里有登记成 samples-heavy 的几道，
     喂它照旧算提交时才跑的检查，拒绝说明点名那几道、写单跑一道的写法。stage-selftest.sh 进按名字判的脚本（KNOWN_SCRIPT_LOCATIONS），放行时也不读它的正文。
     带 --item <阶段文件名>（给几次取并集）时只判点名的那几道（stage_selftest_item_problems）：参数照 stage-selftest.sh 自己的循环切（stage_selftest_arguments：先照 preflight
     摘掉每一个 --force，--item 后面的词是它的值、不当阶段目录），阶段目录里配了样本（fixtures/<阶段>/）、又被点到的每一道（照它的比法：点名的值用空格连起来、两头加空格，
     阶段名两边带空格落在里面就算点到，一个值里带空格能点到几道）都要登记成 samples-light 或 samples-light-marker:、阶段文件与仓里那一份相同、样本与 HEAD 加暂存区逐字节相同，
     有一道登记成 samples-heavy、没登记或样本有改动就拒，说明点名是哪一道、为什么（stage_feed_problem）。登记了标记的那一道不逐份核样本里的标记（stage_sample_marker_matches 是 ② 的）：
     样本已与暂存区逐字节相同，仓里的红样本本来就放了故意坏掉的标记（checker-tier-layer0-replay 的 sample-cargo-not-executable：假 cargo 不可执行，阶段开跑前判红）。
     点名里没有一道是配了样本的阶段（名字写错、阶段不存在、--item 缺值）的放行：stage-selftest.sh 先核点名，一份样本都不喂就退 2、列出可点名的阶段，拒了反倒看不到那份清单。
     带 --list-items 的放行（只列可点名的阶段，不喂样本）。这两种放行信的是 stage-selftest.sh 自己只喂点名的几道，命令词同样要解到本仓那一份，
     别处树里路径尾部相同的拷贝照拒（不带 --item 的也是）。阶段目录写成 $PWD/… 或 ${PWD}/… 的，
     当前目录认得出、命令文本里没另设 PWD 时换成当前目录再判（WORKING_DIRECTORY_VARIABLE；不带 --item 的照旧不换）；点名写在变量或通配里的照拒。
  ② 直接起一道登记成 samples-light-marker:<标记> 的阶段：当前目录是样本根、参数里每一个解到已有目录的词都是这个样本根（至少一个，就是根参数）、样本根里的标记
     是仓里那一道样本放的那种（stage_sample_marker_matches：标记是目录的（checker-tier-layer0-replay），STAGE_SAMPLE_FAKE_TOOLS 登记的假工具（cargo）要可执行、与仓里
     fixtures/<阶段>/*/<标记>/ 某一份的同名文件逐字节相同，空标记目录、内容不同、不可执行的照拒；标记是文件的（herd7、QEMU），样本根里的也要是文件、与仓里某份样本的标记
     逐字节相同）、阶段文件与登记的那一份逐字节相同（stage_sample_run_allowed）。不写根参数（阶段默认判真仓）、根参数指到别处、样本根没放标记或标记不对、根参数写在变量里的，
     照旧按那一道判。
  样本档登记成 samples-light 而没有标记的，只走 ①（样本根里没有能让阶段自己避开真家伙的东西，直接起照旧按那一道判）。
  罩不住的：同一条命令里先拷样本、放标记再起的（判的那一刻样本根还不在，照拒；分两条命令写）；判完之后、阶段开跑之前才删掉标记或改阶段正文的；
  改了阶段正文让样本档起重活、又没改登记行的（这一行的判定靠写的人逐道读文件头，闸不读阶段正文）；① 喂的阶段目录里阶段 source 的共用库（lib/、lib.sh）
  与阶段读的登记表（stage-inputs.tsv 这类）不与仓里比；② 只核 STAGE_SAMPLE_FAKE_TOOLS 登记的假工具（checker-tier-layer0-replay 的假 rustc 不核），参照的是仓里工作区那几份样本，不是暂存区；
  本仓那一份 stage-selftest.sh 自己判完之后、开跑之前被改了正文的。
命令词带通配（test-script-align-r2 判决 U1-1；*、?、[，GLOB_CHARACTERS）：classify 先照 bash 展开（command_word_expansions：相对路径按这条命令的当前目录 glob，
  `[^…]` 换成 python 认的 `[!…]`，排好序；只是 `[`、`[[` 的不展开），再逐个判（pattern_expansion_use）：bash 真起的那一条（头一份当命令、其余几份接在写的参数前面）
  与每一份单独当命令、带写的参数（最多 PATTERN_EXPANSION_LIMIT 份）各判一遍，展开出来的那条当一段命令文本再切一遍、剥包装（写成通配的 capped.sh、nice、
  /usr/bin/time、detached-run.sh 也认）；有一条判成哪一类（重型、按名字判的重型脚本、门禁阶段、提交时才跑的检查）就照它拒，说明里写展开成了什么；
  都不是、或零命中（bash 照字面起）、当前目录认不出的，照字面判。合法的通配（`ls research/scripts/*.sh`，命令词不带通配）不受影响。
  接受的误拒：脚本里 case 的模式行被切成命令（lib_shell_words.py 不认 case），模式写成通配的同样展开，展开出重型的照拒（`*)` 在 research/scripts 里、
  `.claude/gate.d/*.sh)` 这一类），与不带通配的 case 模式行同一个老口子；几份里只要有一份单独当命令判成哪一类就拒，不管 bash 真起的是不是它。
  罩不住的：bash 按区域设置排序、头一份与这里排的不同时，
  真起的那一条只在「每一份单独判」里按写的参数判过；展开超过 PATTERN_EXPANSION_LIMIT 份时后面几份不单独判；
  展开出来的脚本不读正文（heavy-test-guard.sh 的 NOT_A_LITERAL_PATH 那一侧照旧）；cd 到变量路径之后的相对通配（当前目录认不出，照字面判）。
  方括号里的 POSIX 字符类、等价类、整理符号（[[:alpha:]]、[[=h=]]、[[.h.]]，连同 [a[:lower:]]、[![:digit:]] 这种夹在别的字符后面的；BASH_BRACKET_PATTERN 认 `[:`、`[=`、`[.`）
  python glob 不认（test-script-align-r3 攻方 A1：`bash research/scripts/mutat[[:alpha:]].sh crates/mutations.tsv` 零命中、照字面放行，bash 起的却是 mutate.sh）：
  带这几种写法的词（命令词与参数）交给 bash 自己展开（bash_pattern_expansions：`bash --norc --noprofile -c`、shopt -s nullglob，词经 $1 传进去、IFS 设空只做路径展开，
  当前目录是这条命令的当前目录，不起被判的命令），次序照 bash 的；零命中照字面判（bash 照字面起）。展不开或判不出的——相对路径而当前目录认不出、bash 起不来、
  bash 退出码不是 0、超过 BASH_PATTERN_TIMEOUT 秒——命令词按 pattern-unresolved（类名「判不出的通配」）拒，说明写「写成展开之后的字面路径再起」；参数照字面判。
  罩不住的：bash 用的是这里起的那一份的默认选项与这个进程的区域设置，会话 shell 里另开了 extglob、globstar、nocaseglob、dotglob 或区域设置不同的，展开出来的可能不同；
  参数里的这几种写法判不出时照字面（与参数里别的通配同一个口径）。
花括号与参数里的通配（2026-10-01 主 agent 喂本仓钩子实测：`bash research/scripts/mutate.{sh,x} crates/mutations.tsv`、`bash .claude/gate.d/checker-tier-layer0-replay.{sh,} <根>`、
  `bash research/scripts/mutate.sh crates/mutation?.tsv` 都放行，bash 展开后都是真重活）：classify 先做 bash 的两样展开（expanded_words）再判（word_expansion_use）。
  ① 每个词（命令词与参数）照 bash 的花括号展开（brace_expansion，照 bash braces.c：{a,b}、嵌套、空的一份、序列 {1..3}、{a..e}、{01..10..3} 与补零；
  配不上的、里面最外层没有逗号又不是序列的照字面；`${…}` 不展开；打头的 `{}` 不算；展开出来的空串去掉），命令词展开成几份的，头一份是命令、其余接在参数前面；
  ② 再把参数里带 *、?、[ 的按这条命令的当前目录展开（expanded_arguments，与命令词同一个 command_word_expansions；零命中照字面）。
  展开结果与写的不同就判 bash 真起的那一条；命令词自己展开成几份的，再把每一份单独当命令、带展开过的参数各判一遍（与命令词带通配同一个口径），
  命令词展开过的那几条当一段命令文本再切一遍、剥包装（{bash,x.sh} 这种写在包装上的也认），只有参数展开过的直接 classify。
  有一条判成哪一类就照它拒，说明里写展开成了什么；都不是再照字面判（照字面判出的照旧拒）。合法的写法（`ls research/scripts/{a,b}.sh`、`cat crates/*.toml`）展开之后照样不重型，放行。
  接受的误拒：引号在切词时已经去掉，照字面写的花括号与通配（'{a,b}'、'crates/*.tsv'、\\{a,b\\}）这里也展开；命令词展开出的几份里只要有一份单独当命令判成哪一类就拒，
  不管 bash 真起的是不是它；argv 已经展开过的进程（看门狗那一侧）再展开一遍，同样只会多认、不会少认。
  罩不住的：一个词展开超过 WORD_EXPANSION_LIMIT 份的后面几份；一个词里配不上的花括号多到扫描超过 BRACE_SCAN_LIMIT 个记号的，这个词照字面；
  展开出来的文件名里本身带花括号或通配、要再套超过 WORD_EXPANSION_DEPTH_LIMIT 层的；cd 到变量路径之后的相对通配；赋值词（X={a,b}）与重定向目标照 bash 另有规矩，这里不分；
  花括号展开出来的脚本同样不读正文。
看不见的：.cargo/config.toml 与 `--config <文件>` 里定的别名；拷走改名的 checker 档测试二进制；systemd-run -p EnvironmentFile= 指到读不了的文件时里面定没定别名；
cargo mutants -d 指到别处的树。
弄坏开关（只给自证用，证明那几格会红）：LIB_HEAVY_TESTS_BREAK 设成下面一个或几个（逗号分隔），--selftest 与 heavy-test-guard.sh --selftest 都必须判红：
  launcher-whole-word-options（短选项合写不拆，照旧按整词查表）、launcher-drops-setenv（systemd-run -E / --setenv 设的变量不带进里面那条命令）、
  list-by-presence（见到 --list 这个词就算只列）、strace-drops-env（strace -E / --env 设的变量不带进里面那条命令）、
  launcher-long-options-partial（要值的长选项照补全之前的表、不认前缀）、alias-not-expanded（--config / CARGO_ALIAS_ 定的别名看得到值也不展开）、
  package-specification-literal（-p 的值只按字面名比，不认通配、@版本、pkgid URL）、launcher-drops-property-environment（systemd-run -p Environment= /
  EnvironmentFile= 设的变量不带进里面那条命令）、list-past-separator（找 --list 时见到 `--` 不停）、configuration-before-subcommand-only（只收子命令之前的 --config）、mutants-configuration-ignored（cargo mutants 不看 --test-package、--test-workspace 与 .cargo/mutants.toml 的 test_workspace）、no-run-counted-as-running（`--no-run` 只编译也按跑了算）、
  shard-driver-by-name（双机分片的驱动脚本只按名字判、不读树根里的登记行，驱动自检起的那几趟也报）、
  stage-by-number（门禁阶段照旧按两位数编号认，去了编号的重型那几道认不出）、commit-checks-ignored（提交时才跑的检查这一类一样都不认）、
  verus-judges-by-stage-only（Verus 那一道的两份判法脚本直接起不认，干活的用法放过去）、tool-versions-counted（阶段带 --tool-versions 只打工具版本的照旧算那一道）、
  verus-judges-by-working-subcommand（两份判法脚本照旧只按名字与位置认、只算点名的干活子命令，认不出的放过去：PYTHONPATH 加 -m、python -c、符号链接、
    参数放变量、for 循环、函数转参数起的那几格红）、herd7-version-counted（lkmm.sh --herd7-version 照旧算 herd7：「lkmm.sh 只打 herd7 版本」那一格红）、
  verus-judges-force-kept（两份判法脚本的参数不照 preflight 摘 --force：「--target 的值位写 --force」两格与「首参前写 --force、只打工具版本」红）、
  verus-judges-code-by-later-words（python -c 照旧只认文件名与 run_module、按代码之后的词判：「python3 -c 里改 sys.argv」两格、「__import__ / import_module 按模块名」两格、
    「spec_from_file_location 载入再调 run_check」「print 包着 __import__」两格、「runpy.run_path 起红例钉表、代码之后写 --check」共八格红）、
  verus-judges-code-print-unchecked（python -c 的代码每一句是 print 调用就放过，不看参数是不是字符串字面量：「print 包着 __import__、代码只有这一句」那一格红）、
  replay-cells-ignored（research 构建与复跑那一道不分格，只跑单测那一格也算全部实验复跑）、
  broad-mutation-ignored（广谱变异的逐条驱动不按名字认，整趟跑也放过去）、
  broad-mutation-first-round-exemptions（「--item 点名又带 --skip-stage-two」与 --sync-tree 照第一版放过去）、
  broad-mutation-by-name-only（驱动只按名字与位置认：指到它的符号链接、带签名行的改名拷贝、python -c 的代码里点它的都放过去）、
  broad-mutation-read-only-counted（驱动 --check-ignored 与提交门禁 --check gate-verdict 照旧算广谱变异：「广谱变异 --check-ignored 只判忽略用例登记」
    「广谱变异的提交门禁经 nice 只跑判定那一格」两格红）、
  detached-run-opaque（research/scripts/detached-run.sh start … -- <命令> 不剥，里面那条重型命令放过去）、
  detached-run-by-name-only（detached-run.sh 只按名字认：指到它的符号链接、带签名行的改名拷贝、同一条命令里先拷再起的都不剥，里面那条重型命令放过去）、
  selftest-registry-ignored（不读自证登记表，自证一律照旧算提交时才跑的检查）：判自证的格里凡期望放行（None）或期望 commit-check-selftest-unregistered 的都红（判成了 commit-check-selftest）、
  selftest-heavy-rows-as-light（登记成 heavy 的行当 light 判）：「造出来的登记表：登记成 heavy 的 --selftest 算提交时才跑的检查」起的三格、「设了变量的条件行算起重活」、
    「样本表：heavy 行算提交时才跑的检查」「样本表：设了变量的条件行算起重活」红、
  selftest-extra-arguments-allowed（不要求单独一个 --selftest，带别的参数也按登记的轻重判）：「登记成轻的脚本带别的参数再带 --selftest 照拒」三格红、
  selftest-condition-ignored（heavy-if-set:<变量> 不看变量，一律当 light）：「设了变量的条件行算起重活」「样本表：设了变量的条件行算起重活」两格红、
  selftest-unregistered-as-light（表外的 --selftest 当 light 放行）：「登记表外的 --selftest 照拒」两格、「广谱变异的改名拷贝 --selftest 不在登记表里」、
    「样本表：判定写坏的一行不算登记」「样本表：理由太短的一行不算登记」「登记表读不了：一律按表外拒」红、
  selftest-registry-lenient（判定认不出、理由太短的行照收成 light，写坏的行与指到不存在的文件的行不报）：「样本表：判定写坏的一行不算登记」
    「样本表：理由太短的一行不算登记」「样本登记表里写坏的几行逐条报出来」三格红、
  stage-samples-ignored（门禁样本两种起法一样都不认，照旧按那一道或提交时才跑的检查判）：「checker-tier-layer0-replay 指到放了 .layer0-sample-tools 的样本根：样本档放行」
    「stage-selftest.sh 喂样本档全登记成轻的阶段目录：放行」「stage-selftest.sh 被拒的说明点名起重活的那一道」「herd7 指到放了原样标记的样本根：样本档放行」
    「样本与暂存区不同被拒的说明点名那份文件」「--item 点名一道轻的」「--item 点名带标记的一道」「--list-items 只列」「--item 点名不存在的阶段」
    「阶段目录写成 $PWD/…、--item 点名一道轻的」「--item 与值之间夹的 --force 照 preflight 摘掉」「--item 点名里夹起重活的被拒，说明点名那一道」十二格红、
  stage-samples-root-unchecked（直接起登记了标记的阶段不看样本根：不看当前目录与根参数、不看标记、不看阶段正文，一律放行）：
    「checker-tier-layer0-replay 在真仓上跑」「checker-tier-layer0-replay 指到没放标记的目录」「checker-tier-layer0-replay 不写根参数」「改过正文的 checker-tier-layer0-replay 拷贝指到样本根」「checker-tier-layer0-replay 照算层 0 时说明里写样本档怎么跑」五格红
    （cargo 那张表里「bash 起 checker-tier-layer0-replay」与 QEMU、herd7 那两道不带样本根起的几格、stage-samples-marker-unchecked 那六格跟着红）、
  stage-samples-heavy-as-light（登记成 samples-heavy 的阶段当轻的）：「stage-selftest.sh 被拒的说明点名起重活的那一道」「--item 点名里夹起重活的被拒，说明点名那一道」两格红
    （「stage-selftest.sh 喂有起重活样本的阶段目录」照拒：样本仓里没有 crates 变异表那一道的样本，与暂存区不同）、
  stage-samples-content-unchecked（不比阶段文件与登记的那一份）：「改过正文的 checker-tier-layer0-replay 拷贝指到样本根」「stage-selftest.sh 喂轻阶段正文改过的目录」「--item 点名的阶段正文改过」三格红、
  stage-samples-marker-unchecked（直接起带标记的阶段只看标记在不在，不核标记里的假工具与标记文件本身）：「checker-tier-layer0-replay 样本根的标记目录是空的」「checker-tier-layer0-replay 样本根只放了 quick-tier」
    「checker-tier-layer0-replay 样本根的假 cargo 内容不同」「checker-tier-layer0-replay 样本根的假 cargo 不可执行」「herd7 样本根的标记写成目录」「herd7 样本根的标记内容不同」六格红、
  stage-samples-fixtures-unchecked（stage-selftest.sh 喂的样本目录不与暂存区比）：「喂的样本多一份 setup.sh」「喂的样本内容改过」「喂的样本执行位改过」「喂的样本少了文件」
    「暂存之前新加的样本」「样本与暂存区不同被拒的说明点名那份文件」「--item 点名的样本多一份 setup.sh」七格红、
  selftest-registry-by-path-tail（登记表的键照旧只按路径尾部认，不核解到的是不是本仓那一份）：「别处树里路径尾部相同的 claim-experiment.sh 照拒」
    「仓副本里的 admission.py 照拒」「路径写在变量里的照拒」「样本表：别的树里路径尾部相同的 light 脚本不算登记」四格红、
  selftest-pattern-words-judged（命令词带通配的照旧当自证判）：「case 模式行切出的 .claude/gate.d/lib/*.py 不当起共用库」一格红、
  stage-items-as-directory（stage-selftest.sh 的 --item、--list-items 照旧不认：值算成阶段目录、整个目录照判）：「--item 点名一道轻的」「--item 点名带标记的一道」
    「--list-items 只列」「--item 点名不存在的阶段」「阶段目录写成 $PWD/…、--item 点名一道轻的」「--item 与值之间夹的 --force 照 preflight 摘掉」
    「--item 点名里夹起重活的被拒，说明点名那一道」七格红、
  stage-items-unjudged（带 --item 时点名的几道不判，一律放行）：「--item 点名里夹一道起重活的」「--item 一个值里带空格点到起重活的」「--item 点名里有不存在的、又有起重活的」
    「--item 点名登记表外的阶段」「--item 点名的样本多一份 setup.sh」「--item 点名的阶段正文改过」「--item 点名写在变量里」「--item 点名里夹起重活的被拒，说明点名那一道」八格红、
  stage-items-script-by-name（带 --item / --list-items 时 stage-selftest.sh 只按名字与位置认，不核是不是本仓那一份）：「别处树里的 stage-selftest.sh 拷贝带 --list-items」
    「别处树里的 stage-selftest.sh 拷贝 --item 点名一道轻的」两格红、
  stage-directory-pwd-unexpanded（阶段目录写成 $PWD/… 的不换成当前目录）：「阶段目录写成 $PWD/…、--item 点名一道轻的」一格红、
  stage-feed-script-by-name（不带 --item 时 stage-selftest.sh 照旧只按名字与位置认）：「别处树里的 stage-selftest.sh 拷贝不带 --item 喂全登记成轻的阶段目录」
    「相对着写、当前目录不在本仓的 stage-selftest.sh 不带 --item」两格红、
  pattern-command-words-literal（命令词带通配的照旧只照字面判、不展开）：cargo 那张表里「通配起 checker-tier-layer0-replay」到「写成通配的 nice 包全量」十格与「造出来的登记表：通配起登记成 heavy 的自证」红、
  pattern-expansions-bare（展开出来的那条直接 classify，不再切词剥包装）：「写成通配的 capped.sh」「写成通配的 detached-run.sh」「写成通配的 nice」三格红、
  pattern-caret-negation-literal（[^…] 不换成 [!…]）：「[^…] 写的字符类照 bash 认」一格红、
  brace-expansion-literal（花括号一律照字面、不展开）：cargo 那张表里「花括号起 mutate.sh」到「花括号里的整数序列展开成几份」七格与「花括号展开与 bash 一致」那一组红、
  brace-sequence-literal（{1..3}、{a..e} 这类序列不展开）：「花括号里的字母序列点到 crates 整表」「花括号里的整数序列展开成几份」两格与「花括号展开与 bash 一致」里的序列那几格红、
  argument-patterns-literal（参数里的通配不展开）：「参数里的通配点到 crates 整表」一格红、
  brace-command-words-bare（命令词展开过的那几条直接 classify，不再切词剥包装）：「花括号写在包装上」一格红。
  下面几个照 research/scripts/mutate.sh 文件头的写法（开关一行，下一行起写打开后红的自证项；用户 2026-10-01 定 C625 取这种）：
  LIB_HEAVY_TESTS_BREAK=posix-brackets-by-python-glob  带 `[:`、`[=`、`[.` 的词照旧交给 python glob（POSIX 字符类、等价类、整理符号认不出，零命中照字面判）
      cargo 那张表里「POSIX 字符类起 mutate.sh 跑 crates 整表」到「cd 到变量路径之后 POSIX 字符类起脚本照拒」七格、「POSIX 字符类起崩溃放量全量」、
      「bash 起不来时 POSIX 字符类起脚本照拒」与「造出来的登记表：POSIX 字符类起登记成 heavy 的自证」共十格红；
      heavy-test-guard.sh --selftest 里「实现员 POSIX 字符类起 mutate.sh 跑 crates 整表：照拒」到「实现员 cd 到变量路径之后 POSIX 字符类起脚本：判不出，照拒」四格红
  LIB_HEAVY_TESTS_BREAK=posix-brackets-unresolved-literal  bash 展不开或判不出时照字面判（与零命中一样），不拒
      cargo 那张表里「cd 到变量路径之后 POSIX 字符类起脚本照拒」与「bash 起不来时 POSIX 字符类起脚本照拒」两格红；
      heavy-test-guard.sh --selftest 里「实现员 cd 到变量路径之后 POSIX 字符类起脚本：判不出，照拒」一格红
  LIB_HEAVY_TESTS_BREAK=crash-amplification-unnamed  崩溃放量的三份入口不按名字算重型（真跑放过去）
      cargo 那张表里「崩溃放量全量（crash-amplification.sh 不带参数）」到「POSIX 字符类起崩溃放量全量」六格红；
      heavy-test-guard.sh --selftest 里「实现员起崩溃放量全量：照拒」到「主 agent 不带前缀起崩溃放量全量：照拒」四格红
  上面这三个开关之外，已有的开关下新加的这几格跟着红（cargo 那张表 / heavy-test-guard.sh --selftest）：pattern-command-words-literal 十格（POSIX 字符类、
  等价类、整理符号起脚本的七格、「造出来的登记表：POSIX 字符类起登记成 heavy 的自证」与「通配起崩溃放量全量」「POSIX 字符类起崩溃放量全量」，参数那一格与「零命中」「合法」两格除外）/ 四格；argument-patterns-literal
  「参数里的 POSIX 字符类点到 crates 整表」一格；stage-by-number「整理符号起 QEMU 那一道」「POSIX 字符类夹在别的字符后面起 herd7 那一道」两格 / 一格；
  stage-samples-root-unchecked 再加「等价类起 checker-tier-layer0-replay」三格 / 两格；selftest-heavy-rows-as-light「造出来的登记表：POSIX 字符类起登记成 heavy 的自证」一格；selftest-pattern-words-judged
  「POSIX 字符类起登记成轻的自证放行」一格；selftest-registry-ignored 再加「崩溃放量分项驱动单独 --selftest 照登记表放行」两格 / 一格；commit-checks-ignored
  「造出来的登记表：POSIX 字符类起登记成 heavy 的自证」「崩溃放量分项驱动单独 --selftest 照登记表放行」与原有的「登记成不起重活的研究脚本单独 --selftest 放行：crash-amplification.sh」
  三格 / 「工具实现员单独跑 crash-amplification.sh --selftest」、它的 stdin 那一格与「实现员单独跑分项驱动的自证」三格（崩溃放量入口的 --selftest 不按登记表判、
  落到按名字算重型；原先红的「工具实现员单独跑 crash-amplification.sh --selftest：读进去几份脚本」不再红，按名字判不读正文了）。
  heavy-test-guard.sh --selftest 里对应的几格（文件头写在各格旁边）在上面这几个开关下同样红。
  LIB_HEAVY_TESTS_BREAK=qemu-build-unnamed  research/scripts/build-patched-qemu.sh 不按名字算真编 QEMU（不带参数、--rebuild 的真编放过去）
      cargo 那张表里「真编 QEMU（build-patched-qemu.sh 不带参数）」到「capped.sh 与 bash -c 包着真编 QEMU」五格红；
      heavy-test-guard.sh --selftest 里「实现员真编 QEMU（--rebuild）：照拒」到「主 agent 不带前缀经内存包装真编 QEMU：照拒」四格红；
      已有的开关 commit-checks-ignored、selftest-registry-ignored 下「真编 QEMU 的 --selftest 照登记表放行」与 heavy-test-guard.sh --selftest 里
      「实现员单独跑 build-patched-qemu.sh 的自证」两格跟着红
崩溃放量的三份入口（test-script-align-r3 攻方 A2；类名「崩溃放量」，kind crash-amplification）：research/scripts/crash-amplification.sh（不带参数是崩溃放量全量，
  --item 点名分项、--expand-up-to 缩过规模的同算）、crash-amplification-item.sh（分项驱动）、crash-amplification-two-hosts.sh（双机驱动），每一种跑法都经
  crash-amplification-lib.sh 起 checker 档 cargo test；正文里的 cargo 拼在 bash 数组里、分项驱动经变量起，读正文看不见，所以按名字判（CRASH_AMPLIFICATION_SCRIPTS：
  名字是这三个就算，路径不论；KNOWN_SCRIPT_LOCATIONS 登记了仓内位置，那几份不读正文）。不算的：crash-amplification.sh 带 --list-items 的（只列分项，不清场、不跑）；
  仓内那几份单独一个 --selftest 的照自证登记表判（三份都登记成 light，放行）。别处同名拷贝的 --selftest 不在登记表里，
  按这一类算（接受的误拒）。子 agent 一律不跑；主 agent 带 =commit 或 =user-request 都能跑。
真编 QEMU（类名「真编 QEMU」，kind qemu-build）：research/scripts/build-patched-qemu.sh 取源码、在 ${TMPDIR:-/tmp} 下约 1 GiB 的编译目录里按给的线程数编 QEMU、
  装进私有目录。按名字判（QEMU_BUILD_SCRIPT：名字是它就算，路径不论；KNOWN_SCRIPT_LOCATIONS 登记了仓内位置，那一份不读正文）：不带参数、--rebuild、别的参数都算，
  包装（bash -c、capped.sh、run-with-memory-cap.sh、nice 这类）里面的同样算。不算的只有仓内那一份单独一个 --selftest（照自证登记表判，登记成 light，放行）；
  --selftest 带了别的参数的照旧算提交时才跑的检查，别处同名拷贝的 --selftest 不在登记表里、按这一类算（接受的误拒）。
  子 agent 一律不跑，只有主 agent 带前缀能跑（heavy-test-guard.sh 的 refusal_reason）。
广谱变异的逐条驱动 research/scripts/broad-mutation.py（类名「广谱变异」，kind broad-mutation）：它逐条改坏 core / checker 的源码，每条起 cargo，整趟要几天。
  算的：凡要起 cargo 或改库目录的——整趟、--shard、--from、--package、--item 点名（点几条、带不带 --skip-stage-two 都算）、--stage-two-only、
  --sync-tree（换全局的版本指针）、--import-results（往本机结果表里导入）；参数只认整词，argparse 认的缩写（--sum、--list）与贴着写的（--item=1）照算，接受的误拒。
  不算的：带 --list-items、--summary、--check-ignored、--help、-h 之一的（只读：驱动走查询那一支就返回，--check-ignored 只扫这一版树里的源码与排除表、不起 cargo）；
  仓内位置上的 --selftest 照自证登记表判，
  改名拷贝、符号链接、python -c 起的 --selftest 不在表里，照旧算提交时才跑的检查。
  认驱动的三种办法（broad_mutation_driver）：名字是 broad-mutation.py 又落在 research/scripts（当前目录与命令文本里设的 PYTHONPATH 各试一遍，
  `python3 -m broad-mutation` 由切词交成这个名字）；命令词解到的文件的真实路径落在 research/scripts/broad-mutation.py（符号链接）；
  文件开头 BROAD_MUTATION_HEAD_BYTES 里有驱动的签名行（行首 `# broad-mutation-driver-signature:`，改名或拷走的一份）。
  `python3 -c '…'` 的代码里出现 broad-mutation.py、或 run_module(…broad-mutation…) 的照驱动算（参数取代码之后的词）；代码里只是读它的也照算，接受的误拒。
  它是 python 脚本，闸不读它的正文，只按这里的名字、位置、签名行与参数判。
  提交门禁那一道 .claude/gate.d/checker-tier-broad-mutation.sh 按文件名算同一类（kind broad-mutation-stage，STAGE_KIND）；带 --list、--tool-versions 的不算，
  --check 只点名 gate-verdict 一格的不算（BROAD_MUTATION_READ_ONLY_CELL：那一格只读库目录、起驱动的 --summary；点了别的格或不带 --check 的照算）。
  子 agent 一律不跑；主 agent 带前缀能跑（提交时先 --sync-tree、再 --changed-since HEAD 起驱动，再跑那一道），放的是整类，不按参数再分。
  罩不住的：同一条命令里先 cp / ln -s 再起（判的那一刻文件还不在）；删掉签名行的拷贝；python 代码里拼出来的路径、subprocess 起的；
  `python3 - < 脚本` 与 heredoc 喂给 python 的代码；uv run、pypy 这类别的解释器；multi-host-run.sh --、xargs、find -exec、eval（对每一类重型都在的老口子）。
切词那一份的弄坏开关（lib_shell_words.py；开关几个用逗号连）下这里的自检同样必须判红，照 research/scripts/mutate.sh 文件头的写法逐个写：开关一行，
下一行起写打开后红的格（格名是 selftest 里的说明，失败行「✗ lib_heavy_tests 自检：<格名> 应当是 <应得>，实际 <实得>」以它起头，括号里是那几格的应得与实得）：
  LIB_SHELL_WORDS_BREAK=arguments-as-executed  `python3 -` 后面的词、`bash -n` 的目标当被执行的脚本
      「python3 - 后面的门禁文件名是参数（门禁目录里跑）」「bash -n 只查门禁的语法」两格红（应当是 None，实际 commit-check-stage）
  LIB_SHELL_WORDS_BREAK=python-options-by-dash  python 带值选项的值被当成脚本名，`python3 -W ignore <驱动>`、`-X`、`-m`、`-c` 起的放过去
      「python3 -W ignore 起广谱变异」「python3 -X dev 起广谱变异」「python3 -BW default（合写、值另起一个词）起广谱变异」
      「python3 --check-hash-based-pycs never 起广谱变异」「cd 进 research/scripts 再 python3 -m broad-mutation」「python3 -m research.scripts.broad-mutation（命名空间包）」
      「python3 -m runpy broad-mutation」「PYTHONPATH 指到 research/scripts 再 python3 -m broad-mutation」「python3 -c runpy.run_path 起广谱变异」
      「python3 -c exec(open(驱动).read())」「python3 -c runpy.run_module 按模块名起」十一格（应当是 broad-mutation，实际 None）；
      「PYTHONPATH 指到 research/scripts 再 python3 -m 起判法脚本跑必红世界」「export PYTHONPATH 再 python3 -m 起红例重放」「python3 -c runpy.run_path 起判法脚本跑必红世界」
      「python3 -c 里改 sys.argv 成 proof、代码之后写 tool-versions」「python3 -c 里改 sys.argv 成 --replay、代码之后不写词」「python3 -c 里 __import__ 按模块名起判法脚本验证明」
      「python3 -c 里 importlib.import_module 起红例重放」「python3 -c 里 spec_from_file_location 载入判法脚本再调 run_check、代码之后写 toolchain」
      「python3 -c 的 print 包着 __import__ 起判法脚本」「python3 -c 的 print 包着 __import__、代码只有这一句」「python3 -c runpy.run_path 起红例钉表、代码之后写 --check」
      十一格（应当是 verus-stage，实际 None）；「经 PYTHONPATH 起判法脚本的 --selftest：不在登记表的位置上，照旧算提交时才跑的检查」一格（应当是 commit-check-selftest，
      实际 None）；共二十三格红
  LIB_SHELL_WORDS_BREAK=detached-run-force-kept  detached-run.sh 的参数不照它的 preflight 摘 --force，`-- --force <重型命令>` 放过去
      「detached-run.sh start 的 -- 之后先写 --force 照摘」一格红（应当是 full-cargo，实际 None）
  heavy-test-guard.sh --selftest 原样转出上面这几格的失败行，另加它自己的格（写在它的文件头）。
"""
import ast, contextlib, fnmatch, functools, glob, hashlib, importlib.util, os, re, shlex, shutil, stat, subprocess, sys, tempfile, tomllib
from typing import NamedTuple


def load_sibling_module(module_name):
    spec = importlib.util.spec_from_file_location(module_name, os.path.join(os.path.dirname(os.path.abspath(__file__)), module_name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


shell_words = load_sibling_module("lib_shell_words")


class HeavyTest(NamedTuple):
    kind: str      # 细分的用法（checker-tier-cargo、gate-sh……），heavy-test-guard.sh 按它定谁能跑
    category: str  # 报给人看的类名（层 0、全量测试……），KIND_CATEGORY 的值
    detail: str    # 这一条为什么算：认出的是什么


# 每一种重型用法：kind → 它属于哪一类
KIND_CATEGORY = {
    "checker-tier-cargo": "checker 档", "checker-tier-binary": "checker 档",
    "layer0-stage": "层 0",
    "qemu-stage": "QEMU", "qemu-system": "QEMU", "vm-bench": "QEMU",
    "herd7-stage": "herd7", "lkmm": "herd7", "herd7": "herd7",
    "crates-mutation-stage": "crates 变异整表", "crates-mutation-mutate": "crates 变异整表",
    "full-cargo": "全量测试", "check-sh": "全量测试",
    "gate-sh": "整轮门禁", "gate-staged": "整轮门禁", "replay-all-stage": "全部实验复跑",
    "e152": "E152 装置",
    "broad-mutation": "广谱变异", "broad-mutation-stage": "广谱变异",
    "verus-stage": "Verus",
    "crash-amplification": "崩溃放量", "pattern-unresolved": "判不出的通配",
    "qemu-build": "真编 QEMU",
}
# 提交时才跑的检查（用户 2026-09-28 定「现在改到提交时候才验证的部分 都删掉」「门禁是靠 脚本和 hook来解决的」）：
# 扫仓的门禁阶段、上游的几道 lint、样本自检与阶段结构判法、自证里起重活的与没登记的那几份，都归提交前主 agent 跑的 research/scripts/gate-staged.sh
COMMIT_CHECK_CATEGORY = "提交时才跑的检查"
# commit-check-selftest：登记表记它起重活的自证；commit-check-selftest-unregistered：表外的、或带了 --selftest 之外参数的（拒绝说明另写怎么登记）
UNREGISTERED_SELFTEST_KIND = "commit-check-selftest-unregistered"
KIND_CATEGORY.update({"commit-check-stage": COMMIT_CHECK_CATEGORY, "commit-check-lint": COMMIT_CHECK_CATEGORY,
                      "commit-check-selftest": COMMIT_CHECK_CATEGORY, UNREGISTERED_SELFTEST_KIND: COMMIT_CHECK_CATEGORY,
                      "commit-check-structure": COMMIT_CHECK_CATEGORY})
COMMIT_CHECK_KINDS = frozenset(kind for kind, category in KIND_CATEGORY.items() if category == COMMIT_CHECK_CATEGORY)
# 上游规范副本里的几道：按名字、又落在 .claude/singlefs-ai-sop/scripts/ 下才算（gate.sh 按名字在别处已算整轮门禁）
COMMIT_CHECK_SOP_DIRECTORY = ".claude/singlefs-ai-sop/scripts"
COMMIT_CHECK_SOP_SCRIPTS = frozenset({"doc-lint.sh", "gate-lint.sh", "shell-lint.sh", "preflight-lint.py", "gate-overlap.py", "rules-lint.sh",
                                      "hooks-registered.sh", "stage-selftest.sh"})
# 这几个目录下的脚本带 --selftest 跑是跑它的自证；.claude/gate.d/lib/ 下的共用库直接执行就是跑自证，带不带 --selftest 都算。轻重按登记表判（classify_selftest）
COMMIT_CHECK_SELFTEST_DIRECTORIES = ("research/scripts", ".claude/hooks", ".claude/gate.d/lib")
GATE_LIBRARY_DIRECTORY = ".claude/gate.d/lib"
# 自证登记表：一行 `<相对仓根的路径><制表符><判定><制表符># <理由>`，写法在表头。放在这份文件旁边；自证换成样本表时改这个变量
SELFTEST_REGISTRY_NAME = "selftest-registry.tsv"
SELFTEST_REGISTRY_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), SELFTEST_REGISTRY_NAME)
SELFTEST_LIGHT, SELFTEST_HEAVY, SELFTEST_HEAVY_IF_SET = "light", "heavy", "heavy-if-set:"
SELFTEST_CONDITION_VARIABLE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
SELFTEST_REASON_MINIMUM = 4   # 理由（# 之后）至少几个字
# 门禁阶段的样本档（文件头「门禁样本」）：登记表里键是 .claude/gate.d/<阶段>.sh 的行，判定写这三种之一
GATE_STAGE_DIRECTORY = ".claude/gate.d"
STAGE_SAMPLES_LIGHT, STAGE_SAMPLES_MARKER, STAGE_SAMPLES_HEAVY = "samples-light", "samples-light-marker:", "samples-heavy"
STAGE_SELFTEST_SCRIPT = "stage-selftest.sh"
# stage-selftest.sh 自己认的两个参数（它文件头「用法」）：--item <阶段文件名> 只喂点名那几道（给几次取并集），--list-items 只列可点名的阶段、不喂样本；别的词是阶段目录
STAGE_SELFTEST_ITEM_OPTION, STAGE_SELFTEST_LIST_OPTION = "--item", "--list-items"
# 阶段目录写成 $PWD/… 或 ${PWD}/…（拒绝说明里给的写法）：当前目录认得出、命令文本里没另设 PWD 时换成当前目录（stage_selftest_item_problems）
WORKING_DIRECTORY_VARIABLE = re.compile(r"^\$(?:PWD|\{PWD\})(?=/|$)")
# 带标记的样本档（samples-light-marker:）里标记是目录时要核的假工具：可执行、与仓里那一道某份样本标记目录里的同名文件逐字节相同（stage_sample_marker_matches）
STAGE_SAMPLE_FAKE_TOOLS = {"checker-tier-layer0-replay.sh": ("cargo",)}
# 命令词里带这几个通配字符的不当自证（classify_selftest）：bash 展开之后才知道起的是哪一份；case 的模式行被切成命令时也是这样的词
SELFTEST_PATTERN_CHARACTERS = set("*?[")
# 阶段参数里认不出是哪个目录的词：没展开的变量、命令替换、通配
UNRESOLVED_WORD_CHARACTERS = set("$`*?[]{}")
# 只查清单、不判的参数：阶段的 --list（格名表）、--list-items（重型逐项点名的清单）与 --tool-versions（Verus 那一道只打工具版本）、gate-overlap.py --list
STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items", "--tool-versions"})
TOOL_VERSIONS_ARGUMENT = "--tool-versions"
# 重型的那几道阶段，按阶段文件名认；.claude/gate.d/ 下其余阶段是提交时才跑的检查
STAGE_KIND = {"checker-tier-layer0-replay.sh": "layer0-stage", "checker-tier-qemu-device-streams.sh": "qemu-stage", "checker-tier-lkmm.sh": "herd7-stage",
              "checker-tier-crates-mutation-replay.sh": "crates-mutation-stage", "checker-tier-research-build-and-replay.sh": "replay-all-stage",
              "checker-tier-formal-verification.sh": "verus-stage", "checker-tier-broad-mutation.sh": "broad-mutation-stage"}
# research 构建与单测、实验复跑两道合成的那一道只有复跑那一格是重型：--check 只点名单测那一格的，算提交时才跑的检查；不给 --check、或点名里有复跑那一格的，算全部实验复跑
RESEARCH_BUILD_AND_REPLAY_STAGE = "checker-tier-research-build-and-replay.sh"
RESEARCH_UNIT_TESTS_CELL = "research-unit-tests"
BROAD_MUTATION_STAGE = "checker-tier-broad-mutation.sh"
BROAD_MUTATION_READ_ONLY_CELL = "gate-verdict"   # 广谱变异提交门禁里只读库目录、起驱动 --summary 的那一格
# 弄坏开关 stage-by-number 用的旧认法（按两位数编号认，认不出去了编号的新名字）
STAGE_KIND_BY_NUMBER = {"54": "layer0-stage", "55": "qemu-stage", "57": "herd7-stage", "59": "crates-mutation-stage", "87": "replay-all-stage"}
# 按名字判的仓内脚本：名字 → 它在仓里的位置。命令词是这个名字、又落在这个位置上，judged_by_name 为真
KNOWN_SCRIPT_LOCATIONS = {
    "gate.sh": ".claude/scripts/gate.sh", "check.sh": ".claude/scripts/check.sh", "lkmm.sh": ".claude/scripts/lkmm.sh",
    "gate-staged.sh": "research/scripts/gate-staged.sh", "mutate.sh": "research/scripts/mutate.sh",
    "vm-bench.sh": "research/scripts/vm-bench.sh", "e152-run.sh": "research/scripts/e152-run.sh",
    "capped.sh": "research/scripts/capped.sh", "run-with-memory-cap.sh": "research/scripts/run-with-memory-cap.sh",
    "layer0-shard-run.sh": "research/scripts/layer0-shard-run.sh", "mutation-shard-run.sh": "research/scripts/mutation-shard-run.sh",
    "broad-mutation.py": "research/scripts/broad-mutation.py",
    "crash-amplification.sh": "research/scripts/crash-amplification.sh", "crash-amplification-item.sh": "research/scripts/crash-amplification-item.sh",
    "crash-amplification-two-hosts.sh": "research/scripts/crash-amplification-two-hosts.sh",
    "build-patched-qemu.sh": "research/scripts/build-patched-qemu.sh",
    STAGE_SELFTEST_SCRIPT: ".claude/singlefs-ai-sop/scripts/stage-selftest.sh",
}
# 广谱变异的逐条驱动（research/scripts/broad-mutation.py）：带这几个参数之一的只读（列项、汇总、帮助），不起 cargo、不改库目录
BROAD_MUTATION_SCRIPT_NAME = "broad-mutation.py"
BROAD_MUTATION_SCRIPT_DIRECTORY = "research/scripts"
BROAD_MUTATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--summary", "--check-ignored", "--help", "-h"})
# 弄坏开关 broad-mutation-first-round-exemptions 下多放过去的（第一版的分类）：--sync-tree 当查询、「--item 加 --skip-stage-two」不算
BROAD_MUTATION_FIRST_ROUND_QUERY_ARGUMENTS = BROAD_MUTATION_QUERY_ARGUMENTS | {"--sync-tree"}
# 驱动文件头的签名行（行首）：改名、拷走的一份按它认。读文件开头这么多字节找它；这份文件自己的开头里没有以它起头的行
BROAD_MUTATION_SIGNATURE_LINE = re.compile(rb"^# broad-mutation-driver-signature:", re.MULTILINE)
BROAD_MUTATION_HEAD_BYTES = 4096
# python -c 的代码里点到驱动：文件名，或 runpy.run_module 按模块名起它
BROAD_MUTATION_CODE_REFERENCE = re.compile(r"broad-mutation\.py|run_module\s*\([^)]*broad-mutation")
# Verus 那一道（checker-tier-formal-verification.sh）的两份判法脚本：直接起时干活的用法算 verus-stage（verus_judge_use）
VERUS_JUDGE_DIRECTORY = "research/scripts"
VERUS_PROOF_CHECK_SCRIPT = "verus-proof-check.py"
# 它文件头「用法」里起 Verus 的三个子命令（main 只认第一个参数）；只在弄坏开关 verus-judges-by-working-subcommand 下当判据用
VERUS_PROOF_CHECK_WORKING_SUBCOMMANDS = frozenset({"proof", "vacuity", "must-red-worlds"})
# 字面写明时放过的只读子命令：toolchain 只读安装与缓存，fingerprint、tool-versions 只打印
VERUS_PROOF_CHECK_READ_ONLY_SUBCOMMANDS = frozenset({"toolchain", "fingerprint", "tool-versions"})
# python -c 的代码里点到两份判法脚本：名字出现就算（文件名，与 __import__、importlib.import_module、runpy.run_module 的模块名都含它）
VERUS_JUDGE_CODE_NAME = {
    "verus-proof-check.py": re.compile(r"verus-proof-check"),
    "e165-pin-red-examples.py": re.compile(r"e165-pin-red-examples"),
}
# 弄坏开关 verus-judges-code-by-later-words 下的旧认法：文件名，或 runpy.run_module 按模块名起它
VERUS_JUDGE_CODE_REFERENCE = {
    "verus-proof-check.py": re.compile(r"verus-proof-check\.py|run_module\s*\([^)]*verus-proof-check"),
    "e165-pin-red-examples.py": re.compile(r"e165-pin-red-examples\.py|run_module\s*\([^)]*e165-pin-red-examples"),
}
# .claude/scripts/lkmm.sh 只打 herd7 版本的那一种用法（stage-inputs.tsv 里 checker-tier-lkmm 的 environment= 起的）
LKMM_VERSION_ONLY_ARGUMENTS = ["--herd7-version"]
E165_PIN_SCRIPT = "e165-pin-red-examples.py"
# 它 main 里带一个值的选项（切参数时跳过那个值）；--check、--replay 里写在后面的那个定模式
E165_PIN_OPTIONS_WITH_VALUE = frozenset({"--root", "--table", "--binary", "--memory-cap", "--threads", "--target", "--kill-bounds"})


def heavy_test(kind, detail):
    return HeavyTest(kind, KIND_CATEGORY[kind], detail)


def break_is_set(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


def libtest_lists_only(libtest_arguments):
    """libtest 的参数里 --list 当选项出现（只列用例、一条都不跑）：跳过带一个值的选项的那个值再找，`--skip --list` 里的 --list 不算；
    见到 `--` 就停、交「不是只列」：libtest 在 `--` 处停止认选项，之后的 --list 是过滤词（`-- --ignored -- --list` 一条都不跑，照样算不是只列，接受的误拒）。"""
    if break_is_set("list-by-presence"):
        return LIST_ONLY_TEST_ARGUMENT in libtest_arguments
    position = 0
    while position < len(libtest_arguments):
        argument = libtest_arguments[position]
        if argument == "--" and not break_is_set("list-past-separator"):
            return False
        if argument == LIST_ONLY_TEST_ARGUMENT:
            return True
        position += 2 if argument in LIBTEST_OPTIONS_WITH_VALUE else 1
    return False


CHECKER_PACKAGE_NAME = "singlefs-checker-tier"   # checker 档住的包（D13 已定项 15）：它的集成测试、库单测与装置二进制都算重型
HOOK_REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# 自证登记表的键相对哪个仓根解成真实路径（classify_selftest 只认解到这一份的命令词）；自检的样本表那几格换成临时目录
SELFTEST_REGISTRY_ROOT = HOOK_REPOSITORY
# stage-selftest.sh 喂的样本拿哪个仓的 HEAD 加暂存区比（sample_directory_changes）；自检换成临时建的样本仓
SAMPLE_INDEX_REPOSITORY = HOOK_REPOSITORY
# libtest 只列用例、一条都不跑的参数
LIST_ONLY_TEST_ARGUMENT = "--list"
# libtest 带一个值的选项：`--skip --list` 里的 --list 是 --skip 的值（照样全跑），不是「只列」
LIBTEST_OPTIONS_WITH_VALUE = {"--logfile", "--skip", "--test-threads", "--format", "--color", "-Z", "--shuffle-seed"}
# 弄坏开关的环境变量（写法见文件头）
BREAK_VARIABLE = "LIB_HEAVY_TESTS_BREAK"
def manifest_sections(path):
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except (OSError, ValueError):  # TOMLDecodeError 与编码错都是 ValueError
        return None


def nearest_manifest(directory):
    while directory:
        candidate = os.path.join(directory, "Cargo.toml")
        if os.path.isfile(candidate):
            return candidate
        parent = os.path.dirname(directory)
        if parent == directory:
            return None
        directory = parent
    return None


def workspace_root_manifest(manifest_path):
    directory = os.path.dirname(manifest_path)
    while True:
        candidate = os.path.join(directory, "Cargo.toml")
        sections = manifest_sections(candidate) if os.path.isfile(candidate) else None
        if sections and "workspace" in sections:
            return candidate, sections
        parent = os.path.dirname(directory)
        if parent == directory:
            return None, None
        directory = parent


def workspace_packages(root_manifest, root_sections):
    """工作区成员：包名 → 包目录。"""
    root = os.path.dirname(root_manifest)
    packages = {}
    for pattern in (root_sections.get("workspace") or {}).get("members") or []:
        for directory in sorted(glob.glob(os.path.join(root, pattern))):
            sections = manifest_sections(os.path.join(directory, "Cargo.toml"))
            name = ((sections or {}).get("package") or {}).get("name")
            if name:
                packages[name] = os.path.normpath(directory)
    return packages


def integration_test_targets(package_directory):
    """包里全部集成测试目标：tests/*.rs、tests/<名>/main.rs 与 [[test]] 的 name。"""
    names = set()
    for path in glob.glob(os.path.join(package_directory, "tests", "*.rs")):
        names.add(os.path.basename(path)[:-3])
    for path in glob.glob(os.path.join(package_directory, "tests", "*", "main.rs")):
        names.add(os.path.basename(os.path.dirname(path)))
    for target in (manifest_sections(os.path.join(package_directory, "Cargo.toml")) or {}).get("test") or []:
        if isinstance(target, dict) and target.get("name"):
            names.add(target["name"])
    return sorted(names)


def checker_package_directories(directory):
    """directory 往上找到的第一个含 crates/<checker 档包> 的根里那个包目录；一个都找不到（仓外执行）才取本 hook 所在仓的那一个。"""
    while directory:
        candidate = os.path.join(directory, "crates", CHECKER_PACKAGE_NAME)
        if os.path.isdir(candidate):
            return [candidate]
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    own = os.path.join(HOOK_REPOSITORY, "crates", CHECKER_PACKAGE_NAME)
    return [own] if os.path.isdir(own) else []


def checker_test_targets(directory):
    """checker 档包的测试二进制名：tests/ 下的测试目标、src/bin/ 下的装置（它们的内联单测编成同名测试二进制）、库本身（包名把 - 换成 _）。"""
    names = set()
    for package_directory in checker_package_directories(directory):
        names.update(integration_test_targets(package_directory))
        names.update(os.path.splitext(os.path.basename(path))[0] for path in glob.glob(os.path.join(package_directory, "src", "bin", "*.rs")))
        names.add(CHECKER_PACKAGE_NAME.replace("-", "_"))
    return names


CARGO_GLOBAL_OPTIONS_WITH_VALUE = {"--color", "--config", "-Z"}
# cargo test 与 cargo nextest run 带一个值的选项（nextest 的 --run-ignored、-E 过滤式这一类也在里面：cargo test 不认它们，放一张表不误读）
CARGO_TEST_OPTIONS_WITH_VALUE = {"-p", "--package", "--exclude", "--test", "--bin", "--example", "--bench", "-F", "--features",
                                 "--target", "--target-dir", "--manifest-path", "-j", "--jobs", "--profile", "--color",
                                 "--message-format", "--config", "-Z", "--lockfile-path",
                                 "--run-ignored", "-E", "--filterset", "-P", "--retries", "--partition", "--test-threads", "--max-fail"}
NARROWING_SELECTORS = {"--test", "--lib", "--bin", "--bins", "--example", "--examples", "--bench", "--benches", "--doc"}
GLOB_CHARACTERS = set("*?[")
# 命令词带通配（classify 先照 bash 展开）：每一份单独当命令判，一次最多判这么多份；bash 真起的那一条（头一份当命令、其余当参数）另外总判
PATTERN_EXPANSION_LIMIT = 256
# bash 的 [^…] 与 [!…] 同义，python 的 glob 只认 [!…]
CARET_NEGATION = re.compile(r"\[\^")
# 方括号里的 POSIX 字符类、等价类、整理符号（[:alpha:]、[=h=]、[.h.]）：python glob 不认，带它的词交给 bash 展开（bash_pattern_expansions）
BASH_BRACKET_PATTERN = re.compile(r"\[[:=.]")
# 交给 bash 展开时最多等几秒；起 bash 用的程序（自证换成不存在的路径，演 bash 起不来）
BASH_PATTERN_TIMEOUT = 5
BASH_FOR_PATTERNS = "bash"
# 崩溃放量的三份入口（文件头「崩溃放量的三份入口」）：名字是这几个就算 crash-amplification；crash-amplification.sh 带 --list-items 的只列分项
CRASH_AMPLIFICATION_SCRIPTS = frozenset({"crash-amplification.sh", "crash-amplification-item.sh", "crash-amplification-two-hosts.sh"})
CRASH_AMPLIFICATION_LISTING_SCRIPT = "crash-amplification.sh"
CRASH_AMPLIFICATION_QUERY_ARGUMENTS = frozenset({"--list-items"})
# 真编 QEMU（文件头「真编 QEMU」）：名字是它就算 qemu-build；仓内那一份单独一个 --selftest 由 classify_selftest 先照登记表判
QEMU_BUILD_SCRIPT = "build-patched-qemu.sh"
# 花括号展开与参数里的通配（classify 先照 bash 展开，文件头「花括号与参数里的通配」）：一个词最多展开成这么多份，超出的不判
WORD_EXPANSION_LIMIT = 1024
# 展开出来的命令再切一遍、再判时又带花括号或通配的（文件名里本身带这几个字符），最多再套这么多层
WORD_EXPANSION_DEPTH_LIMIT = 4
# 花括号展开只看这几种记号（brace_gobble）；一个词最多看这么多个，配不上的花括号多到超过的这个词不展开
BRACE_TOKENS = re.compile(r"\$\{|[{},]|\.\.")
BRACE_SCAN_LIMIT = 200000
# 花括号里的序列：{<起>..<止>[..<步长>]}，起止同是整数或同是一个字母；步长是整数（0 当 1，正负只看起止的方向）
BRACE_SEQUENCE = re.compile(r"([+-]?[0-9]+|[A-Za-z])\.\.([+-]?[0-9]+|[A-Za-z])(?:\.\.([+-]?[0-9]+))?")
# cargo llvm-cov 不起测试的子命令；cargo hack 后面接的、起测试或跑程序的子命令
LLVM_COV_NON_RUNNING_SUBCOMMANDS = {"report", "clean", "show-env"}
HACK_INNER_SUBCOMMANDS = {"test", "t", "run", "r", "nextest", "miri", "llvm-cov", "mutants"}
# cargo mutants 的选项里 cargo test 也认、照搬过去的（选包的那几个）
MUTANTS_PACKAGE_OPTIONS_WITH_VALUE = {"-p", "--package", "--manifest-path"}
ALIAS_CONFIGURATION = re.compile(r"^\s*alias\.([A-Za-z0-9_-]+)\s*=")
ALIAS_ENVIRONMENT_PREFIX = "CARGO_ALIAS_"


class CargoCommand(NamedTuple):
    subcommand: str
    rest: list                  # 子命令后面的参数
    directory: str | None       # 按 -C 换过之后的目录
    configuration_values: list  # 全局选项 --config 的值（--config=<值> 与 --config <值> 两种写法）


def cargo_subcommand(arguments, directory):
    """cargo 的全局选项（+工具链、--config、-C 目录……）跳过之后的子命令：交 CargoCommand；没有子命令交 None。"""
    index, configuration_values = 0, []
    while index < len(arguments):
        argument = arguments[index]
        if argument.startswith("+"):
            index += 1
        elif argument in CARGO_GLOBAL_OPTIONS_WITH_VALUE:
            if argument == "--config" and index + 1 < len(arguments):
                configuration_values.append(arguments[index + 1])
            index += 2
        elif argument == "-C":
            directory = shell_words.resolve_path(directory, arguments[index + 1]) if index + 1 < len(arguments) else directory
            index += 2
        elif argument.startswith("-"):
            if argument.startswith("--config="):
                configuration_values.append(argument[len("--config="):])
            index += 1
        else:
            break
    if index >= len(arguments):
        return None
    rest = arguments[index + 1:]
    if not break_is_set("configuration-before-subcommand-only"):
        configuration_values += configuration_values_after_subcommand(rest)
    return CargoCommand(arguments[index], rest, directory, configuration_values)


def configuration_values_after_subcommand(rest):
    """子命令之后、`--` 之前的 --config 值（--config 是 cargo 的全局选项，写在 test 之后照认）：--config <值> 与 --config=<值> 两种写法；
    按 cargo test 的带值选项表跳过别的选项的值（`--features --config` 里的 --config 是 --features 的值）。"""
    values, position = [], 0
    while position < len(rest):
        word = rest[position]
        if word == "--":
            break
        if word == "--config":
            if position + 1 < len(rest):
                values.append(rest[position + 1])
            position += 2
        elif word.startswith("--config="):
            values.append(word[len("--config="):])
            position += 1
        else:
            position += 2 if word in CARGO_TEST_OPTIONS_WITH_VALUE else 1
    return values


MUTANTS_TEST_WORKSPACE_SETTING = re.compile(r"^\s*test_workspace\s*=\s*(true|false)\b", re.MULTILINE)


def mutants_configured_test_workspace(source_directory):
    """源码树里 .cargo/mutants.toml 的 test_workspace（True / False），没有这份配置或没写这一项交 None。
    cargo mutants 从源码树（-d 给的目录，不给就是当前目录）往上找工作区根，读那里的 .cargo/mutants.toml；这里从 source_directory 往上逐级找第一份。"""
    current = os.path.abspath(source_directory or ".")
    while True:
        path = os.path.join(current, ".cargo", "mutants.toml")
        if os.path.isfile(path):
            try:
                found = MUTANTS_TEST_WORKSPACE_SETTING.search(open(path, encoding="utf-8").read())
            except (OSError, UnicodeDecodeError):
                return None
            return None if found is None else found.group(1) == "true"
        parent = os.path.dirname(current)
        if parent == current:
            return None
        current = parent


def mutants_test_arguments(rest, directory=None):
    """cargo mutants 的参数 → 它反复起的 cargo test 的参数。选包的选项（-p / --package / --manifest-path / --workspace）照搬，`--` 之后的原样接上；
    测哪些包另看三样（cargo mutants 的 --test-workspace / --test-package 与 .cargo/mutants.toml 的 test_workspace）：
    --test-package X 测的是 X（换成 -p X）；--test-workspace=true 或配置里 test_workspace = true（没带 --no-config、没被 --test-workspace=false 盖掉）
    测整个工作区（加 --workspace）。-d / --dir 给的源码树换掉当前目录去找配置与工作区。"""
    before, after = (rest[:rest.index("--")], rest[rest.index("--") + 1:]) if "--" in rest else (rest, [])
    kept, test_packages, test_workspace_option, source_directory, reads_configuration = [], [], None, directory, True
    index = 0
    while index < len(before):
        word = before[index]
        if word in MUTANTS_PACKAGE_OPTIONS_WITH_VALUE:
            kept += before[index:index + 2]
            index += 2
            continue
        if word in ("-d", "--dir") and index + 1 < len(before):
            source_directory = shell_words.resolve_path(directory, before[index + 1]) if directory else before[index + 1]
            index += 2
            continue
        if word.startswith("--dir="):
            value = word.split("=", 1)[1]
            source_directory = shell_words.resolve_path(directory, value) if directory else value
        elif word == "--test-package" and index + 1 < len(before):
            test_packages += [name for name in before[index + 1].split(",") if name]
            index += 2
            continue
        elif word.startswith("--test-package="):
            test_packages += [name for name in word.split("=", 1)[1].split(",") if name]
        elif word == "--test-workspace" and index + 1 < len(before) and before[index + 1] in ("true", "false"):
            test_workspace_option = before[index + 1] == "true"
            index += 2
            continue
        elif word.startswith("--test-workspace="):
            test_workspace_option = word.split("=", 1)[1] == "true"
        elif word == "--no-config":
            reads_configuration = False
        elif word == "--workspace" or word.startswith(("--package=", "--manifest-path=")) or (word.startswith("-p") and len(word) > 2):
            kept.append(word)
        index += 1
    if break_is_set("mutants-configuration-ignored"):
        return kept + after
    if source_directory is not None and source_directory != directory and not any(
            word == "--manifest-path" or word.startswith("--manifest-path=") for word in kept):
        kept += ["--manifest-path", os.path.join(source_directory, "Cargo.toml")]
    if test_packages:
        kept = [word for position, word in enumerate(kept)
                if not (word in ("-p", "--package") or word.startswith(("--package=",)) or (word.startswith("-p") and len(word) > 2)
                        or (position > 0 and kept[position - 1] in ("-p", "--package")))]
        for name in test_packages:
            kept += ["-p", name]
        return kept + after
    test_workspace = test_workspace_option
    if test_workspace is None and reads_configuration:
        test_workspace = mutants_configured_test_workspace(source_directory)
    if test_workspace:
        kept.append("--workspace")
    return kept + after


def test_invocation(subcommand, rest, directory=None):
    """cargo 的子命令与它后面的参数 → (怎么起的：'test'、'nextest'、'run' 或 None, 照 cargo test 的写法读的参数)。
    nextest 只认 run（r）；miri、llvm-cov、hack 剥掉自己那一层再看里面那个子命令（llvm-cov 不带子命令时就是跑测试）；mutants 反复起 cargo test。"""
    if subcommand in ("test", "t"):
        return "test", rest
    if subcommand in ("run", "r"):
        return "run", rest
    if subcommand == "nextest":
        return ("nextest", rest[1:]) if rest[:1] in (["run"], ["r"]) else (None, [])
    if subcommand == "miri":
        return test_invocation(rest[0], rest[1:]) if rest else (None, [])
    if subcommand == "llvm-cov":
        if rest[:1] == ["nextest"]:
            return "nextest", rest[1:]
        if rest and rest[0] in LLVM_COV_NON_RUNNING_SUBCOMMANDS:
            return None, []
        if rest and not rest[0].startswith("-"):
            return test_invocation(rest[0], rest[1:])
        return "test", rest
    if subcommand == "hack":
        options = rest[:rest.index("--")] if "--" in rest else rest
        for index, word in enumerate(options):
            if word in HACK_INNER_SUBCOMMANDS:
                return test_invocation(word, rest[:index] + rest[index + 1:])
        return None, []
    if subcommand == "mutants":
        return "test", mutants_test_arguments(rest, directory)
    return None, []


def package_specification_members(specification, members):
    """-p 的一个值对上的工作区成员名（集合）：认 cargo 的几种包规格——光名字、name@version、pkgid URL（path+file://…#name@version、…#name）、
    通配（*、?、[…]，cargo 1.64 起 -p 认 glob）。一个都对不上交空集。"""
    text = specification.strip().strip("'\"")
    if "#" in text:
        text = text.rsplit("#", 1)[1]
    text = text.split("@", 1)[0]
    if GLOB_CHARACTERS & set(text):
        return set(fnmatch.filter(list(members), text))
    return {text} if text in members else set()


def alias_expansion(command, environment):
    """别名展开成的那几个词：--config 里 alias.<名> = "…" 或环境变量 CARGO_ALIAS_<名> 的值，按空白切；看不到值交 None。"""
    for value in command.configuration_values:
        match = re.match(r"^\s*alias\.([A-Za-z0-9_-]+)\s*=\s*(.+?)\s*$", value)
        if match and match.group(1) == command.subcommand:
            raw = match.group(2).strip()
            try:
                parsed = tomllib.loads(f"value = {raw}")["value"]
            except (tomllib.TOMLDecodeError, KeyError):
                return None
            return parsed if isinstance(parsed, list) else str(parsed).split()
    alias_variable = ALIAS_ENVIRONMENT_PREFIX + command.subcommand.upper().replace("-", "_")
    if alias_variable in environment and environment[alias_variable]:
        return environment[alias_variable].split()
    return None


def alias_of_subcommand(command, environment):
    """子命令是 --config 的 alias.<名> 或环境变量 CARGO_ALIAS_<名> 定的别名时交一句（展开之后是什么、带什么参数都看不见，当 cargo test 判）；不是交 None。"""
    aliases = {match.group(1) for value in command.configuration_values for match in [ALIAS_CONFIGURATION.match(value)] if match}
    if command.subcommand in aliases:
        return f"--config 里定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    alias_variable = ALIAS_ENVIRONMENT_PREFIX + command.subcommand.upper().replace("-", "_")
    if alias_variable in environment:
        return f"环境变量 {alias_variable} 定的别名 {command.subcommand}（展开之后带什么参数看不见）"
    return None


def cargo_use(arguments, directory, environment=None):
    """cargo 这一条算不算重型：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。"""
    found = cargo_subcommand(arguments, directory)
    if found is None:
        return None
    environment = environment or {}
    expansion = alias_expansion(found, environment) if not break_is_set("alias-not-expanded") else None
    if expansion:
        expanded = cargo_use(list(expansion) + list(found.rest), found.directory,
                             {name: value for name, value in environment.items() if not name.startswith(ALIAS_ENVIRONMENT_PREFIX)})
        # 展开看得见：展开之后是什么就判什么（跑 harness 档的别名不重型），不再走「看不见」那一支
        return expanded._replace(detail=f"别名 {found.subcommand} 展开成 cargo {' '.join(expansion)}：{expanded.detail}") if expanded else None
    alias_reason = alias_of_subcommand(found, environment)
    if alias_reason:
        return heavy_test("checker-tier-cargo", f"cargo {found.subcommand}：{alias_reason}，跑到哪个包看不见，按 checker 档算")
    invocation, rest = test_invocation(found.subcommand, found.rest, found.directory)
    # `--no-run`（在 `--` 之前）只编译测试、一条都不跑：编译不是重型，谁都能做（子 agent 交回前也要编）。
    cargo_options = rest[:rest.index("--")] if "--" in rest else rest
    if invocation in ("test", "nextest") and "--no-run" in cargo_options and not break_is_set("no-run-counted-as-running"):
        return None
    directory = found.directory
    packages, test_names, selectors, binaries = [], [], set(), []
    whole_workspace, manifest_argument = False, None
    position = 0
    while position < len(rest):
        argument = rest[position]
        if argument == "--":
            break
        if argument.startswith("--") and "=" in argument:
            option, value = argument.split("=", 1)
            position += 1
        elif argument.startswith("-p") and len(argument) > 2 and not argument.startswith("--"):
            option, value = "-p", argument[2:].lstrip("=")
            position += 1
        elif argument in CARGO_TEST_OPTIONS_WITH_VALUE:
            option = argument
            value = rest[position + 1] if position + 1 < len(rest) else ""
            position += 2
        else:
            option, value = argument, None
            position += 1
        if option in ("-p", "--package"):
            packages.append(value)
        elif option == "--test":
            test_names.append(value)
            selectors.add(option)
        elif option in ("--bin", "--example", "--bench"):
            selectors.add(option)
            if option == "--bin":
                binaries.append(value)
        elif option == "--manifest-path":
            manifest_argument = value
        elif option in ("--workspace", "--all"):
            whole_workspace = True
        elif option in ("--lib", "--bins", "--examples", "--tests", "--benches", "--all-targets", "--doc"):
            selectors.add(option)
    if invocation == "run":
        if "e152-file-system-benchmark" in binaries:
            return heavy_test("e152", "cargo run --bin e152-file-system-benchmark")
        return None
    if invocation is None:
        return None
    described = f"cargo {found.subcommand}"
    libtest_arguments = rest[rest.index("--") + 1:] if "--" in rest else []
    if libtest_lists_only(libtest_arguments):
        return None
    if whole_workspace:
        return heavy_test("full-cargo", f"{described} 带 --workspace / --all")
    manifest_path = shell_words.resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
    sections = manifest_sections(manifest_path) if manifest_path else None
    root_manifest, root_sections = workspace_root_manifest(manifest_path) if sections is not None else (None, None)
    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
    narrowed = bool(selectors & NARROWING_SELECTORS)
    scope, is_virtual_workspace_root = [], False
    if sections is not None:
        is_virtual_workspace_root = "workspace" in sections and "package" not in sections
        if packages:
            named = set()
            for specification in packages:
                named |= (package_specification_members(specification, members) if not break_is_set("package-specification-literal")
                          else ({specification} if specification in members else set()))
            scope = [members[name] for name in sorted(named)]
        elif is_virtual_workspace_root:
            scope = list(members.values())
        else:
            scope = [os.path.normpath(os.path.dirname(manifest_path))]
    if sections is not None and not packages and is_virtual_workspace_root and not narrowed:
        return heavy_test("full-cargo", f"在工作区根（{os.path.dirname(manifest_path)}）上不带 -p / --test / --lib / --bin 的 {described}")
    if sections is not None and not narrowed and members and set(scope) == set(members.values()):
        return heavy_test("full-cargo", (f"{described} 不挑目标，而包的范围是整个工作区（{os.path.dirname(root_manifest)} 的全部 "
                                         f"{len(members)} 个成员），等于全量"))
    # checker 档按包判（D13 已定项 15）：范围里有 checker 档包，不管挑哪个目标（库单测、集成测试、装置二进制的内联单测、doctest）都是重型；
    # 别的包（harness 档、core、format、池级 checker）的测试随时跑
    checker_directory = members.get(CHECKER_PACKAGE_NAME)
    if checker_directory and os.path.normpath(checker_directory) in {os.path.normpath(path) for path in scope}:
        return heavy_test("checker-tier-cargo", f"{described} 跑到 {CHECKER_PACKAGE_NAME} 包的测试（checker 档，默认只在提交时跑；"
                                                f"{'--test ' + '、'.join(test_names) if test_names else '按包'}）")
    return None


# 门禁阶段：.claude/gate.d/ 顶层的 *.sh（lib-*.sh 是共用库、lib/ 底下的不是阶段），按文件名认，不按编号认
GATE_STAGE_PATH = re.compile(r"(?:^|/)\.claude/gate\.d/([^/]+\.sh)$")
GATE_STAGE_NUMBER_PATH = re.compile(r"(?:^|/)\.claude/gate\.d/(\d+)-[^/]+\.sh$")
# cargo 编出来的测试二进制：<target 目录>/[<目标三元组>/]<profile>/deps/<名字>-<16 位十六进制的元数据哈希>，名字里的 - 已换成 _
TEST_BINARY_PATH = re.compile(r"(?:^|/)deps/([A-Za-z0-9_]+)-[0-9a-f]{16}$")


def gate_stage_name(word, directory):
    """命令词是 .claude/gate.d/ 顶层的一道阶段时交回它的文件名，不是交 None。"""
    for candidate in (word, shell_words.resolve_path(directory, word) or ""):
        match = GATE_STAGE_PATH.search(candidate)
        if match and not match.group(1).startswith("lib-"):
            return match.group(1)
    return None


def located_in(word, directory, relative_directory):
    """命令词（照写的样子，或按当前目录解开之后）是不是正好落在 relative_directory 这一层（相对仓根的写法，按路径尾部比）。"""
    for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""):
        parent = os.path.dirname(candidate)
        if parent == relative_directory or parent.endswith("/" + relative_directory):
            return True
    return False


def checked_cells(arguments):
    """阶段参数里 --check 点名的格（--check <格>[,<格>…] 与 --check=<…>，给几次取并集）；没给 --check 交 None（全跑）。"""
    cells, given, position = set(), False, 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == "--check" and position + 1 < len(arguments):
            cells.update(filter(None, arguments[position + 1].split(",")))
            given, position = True, position + 2
            continue
        if argument.startswith("--check="):
            cells.update(filter(None, argument[len("--check="):].split(",")))
            given = True
        position += 1
    return cells if given else None


def classify_gate_stage(stage, arguments):
    """一道门禁阶段（按文件名）：只查清单的（--list、--list-items）与生成的（--write）不算；重型的那几道按 STAGE_KIND；其余是提交时才跑的检查。"""
    if break_is_set("stage-by-number"):
        match = GATE_STAGE_NUMBER_PATH.search(".claude/gate.d/" + stage)
        kind = STAGE_KIND_BY_NUMBER.get(match.group(1)) if match else None
        return heavy_test(kind, f"门禁 {stage}") if kind else None
    query_arguments = STAGE_QUERY_ARGUMENTS - {TOOL_VERSIONS_ARGUMENT} if break_is_set("tool-versions-counted") else STAGE_QUERY_ARGUMENTS
    if query_arguments.intersection(arguments):
        return None
    kind = STAGE_KIND.get(stage)
    if (stage == RESEARCH_BUILD_AND_REPLAY_STAGE and not break_is_set("replay-cells-ignored")
            and checked_cells(arguments) == {RESEARCH_UNIT_TESTS_CELL}):
        kind = None   # 只跑单测那一格：不是全部实验复跑，落到下面「提交时才跑的检查」
    if (stage == BROAD_MUTATION_STAGE and not break_is_set("broad-mutation-read-only-counted")
            and checked_cells(arguments) == {BROAD_MUTATION_READ_ONLY_CELL}):
        return None   # 只跑判定那一格：只读库目录、起驱动的 --summary（驱动 --summary 本身也不算）
    if kind:
        return heavy_test(kind, f"门禁 {stage}")
    if "--write" in arguments or break_is_set("commit-checks-ignored"):
        return None
    return heavy_test("commit-check-stage", f"门禁阶段 {stage}（扫仓的门禁归提交前主 agent 跑的 research/scripts/gate-staged.sh）")


def classify_commit_check(name, word, arguments, directory, environment=None):
    """上游规范副本里的几道 lint 与样本自检、阶段结构判法：是就交 HeavyTest（提交时才跑的检查），不是交 None。自证另由 classify_selftest 判。
    environment 是命令文本里设过的变量（stage-selftest.sh 的阶段目录写成 $PWD/… 时，命令里另设过 PWD 就不换）。"""
    if break_is_set("commit-checks-ignored"):
        return None
    if name in COMMIT_CHECK_SOP_SCRIPTS and located_in(word, directory, COMMIT_CHECK_SOP_DIRECTORY):
        if name == "gate-overlap.py" and "--list" in arguments:
            return None   # 查已有门禁与钩子的清单，不判
        if name == STAGE_SELFTEST_SCRIPT:
            problems = stage_selftest_problems(word, arguments, directory, environment)
            if not problems:
                return None   # 喂的（带 --item 时点名的）每一道样本档都登记成轻的，或只列不喂（文件头「门禁样本」①）
            return heavy_test("commit-check-lint", f"{COMMIT_CHECK_SOP_DIRECTORY}/{name}（不放行：{problems}；不带 --item 时喂的阶段目录里有样本的每一道都登记成轻的才放行，"
                                                   "只喂几道写 --item <阶段文件名>（可给几次，点名的每一道都要登记成轻的；可点名的阶段 --list-items 列），"
                                                   "单跑一道带假工具标记的样本：cd 进样本根、根参数写同一个目录直接起那一道，登记表 .claude/hooks/selftest-registry.tsv；"
                                                   "样本与暂存区不同的：改完的样本先交主 agent 暂存，或喂一份与暂存区相同的样本）")
        return heavy_test("commit-check-lint", f"{COMMIT_CHECK_SOP_DIRECTORY}/{name}")
    if name == "gate-structure-check.py" and located_in(word, directory, "research/scripts"):
        return heavy_test("commit-check-structure", "research/scripts/gate-structure-check.py（门禁阶段的结构）")
    return None


class SelftestRow(NamedTuple):
    weight: str   # light、heavy 或 heavy-if-set:<变量>
    reason: str   # # 之后那段
    line: int     # 在表里第几行


@functools.lru_cache(maxsize=8)
def selftest_registry_at(path, modified, size):
    """登记表 → ({键: SelftestRow}, [写坏的行，逐条一句])。modified、size 只当缓存的键（看门狗常驻，表改了要重读）。
    认不出的判定、没有 # 理由或理由太短、不在三处目录里的、重复的（留第一行）不收；弄坏开关 selftest-registry-lenient 下照收、不报。"""
    rows, problems = {}, []
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except (OSError, UnicodeDecodeError) as error:
        return {}, [f"读不了登记表 {path}：{error}"]
    lenient = break_is_set("selftest-registry-lenient")
    for number, line in enumerate(lines, start=1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        fields = line.split(None, 2)
        key, weight = fields[0], fields[1] if len(fields) > 1 else ""
        reason = fields[2][1:].strip() if len(fields) > 2 and fields[2].startswith("#") else ""
        condition = weight[len(SELFTEST_HEAVY_IF_SET):] if weight.startswith(SELFTEST_HEAVY_IF_SET) else None
        stage_row = os.path.dirname(key) == GATE_STAGE_DIRECTORY
        wrong = []
        if stage_row:
            if not stage_samples_weight_is_valid(weight):
                wrong.append(f"门禁阶段的判定「{weight}」认不出（只认 samples-light、samples-light-marker:<相对样本根、不带 .. 的路径>、samples-heavy）")
        elif weight not in (SELFTEST_LIGHT, SELFTEST_HEAVY) and not (condition is not None and SELFTEST_CONDITION_VARIABLE.match(condition)):
            wrong.append(f"判定「{weight}」认不出（只认 light、heavy、heavy-if-set:<变量>；门禁阶段的样本档写在 .claude/gate.d/<阶段>.sh 那一行）")
        if len(reason) < SELFTEST_REASON_MINIMUM:
            wrong.append(f"# 之后的理由不到 {SELFTEST_REASON_MINIMUM} 个字")
        if not stage_row and os.path.dirname(key) not in COMMIT_CHECK_SELFTEST_DIRECTORIES:
            wrong.append(f"不在 {'、'.join(COMMIT_CHECK_SELFTEST_DIRECTORIES)} 之一的这一层（门禁阶段的样本档写在 {GATE_STAGE_DIRECTORY} 这一层）")
        if key in rows:
            wrong.append(f"与第 {rows[key].line} 行重复")
        if wrong and not lenient:
            problems.append(f"第 {number} 行 {key}：{'；'.join(wrong)}")
            continue
        if key not in rows:
            rows[key] = SelftestRow(weight if not wrong or weight.startswith(SELFTEST_HEAVY) else SELFTEST_LIGHT, reason, number)
    return rows, problems


def read_selftest_registry(path):
    """登记表此刻的内容（selftest_registry_at 按修改时刻与大小缓存）；文件不在交 ({}, [一句])。"""
    try:
        status = os.stat(path)
    except OSError as error:
        return {}, [f"读不了登记表 {path}：{error.strerror}"]
    return selftest_registry_at(path, status.st_mtime_ns, status.st_size)


def selftest_registry_problems(path, root):
    """登记表写坏的行，加上指到 root 下不存在的文件的行，逐条一句；空表是没毛病。弄坏开关 selftest-registry-lenient 下只报读不了。"""
    rows, problems = read_selftest_registry(path)
    problems = list(problems)
    for key, row in sorted(rows.items(), key=lambda item: item[1].line):
        if not os.path.isfile(os.path.join(root, key)) and not break_is_set("selftest-registry-lenient"):
            problems.append(f"第 {row.line} 行 {key}：{root} 下没有这个文件")
    return problems


def selftest_registry_key(word, directory):
    """命令词正好落在三处目录之一（照写的样子，或按当前目录解开）时交登记表里的键「目录/文件名」，不是交 None。"""
    for place in COMMIT_CHECK_SELFTEST_DIRECTORIES:
        if located_in(word, directory, place):
            return f"{place}/{os.path.basename(os.path.normpath(word))}"
    return None


def selftest_registered_file(word, directory, key):
    """命令词解到的真实路径（符号链接按指到的算）是不是登记表所在仓（SELFTEST_REGISTRY_ROOT）里键指的那一份文件；相对路径而当前目录认不出的交 False。
    弄坏开关 selftest-registry-by-path-tail 下一律为真（照旧只按路径尾部认）。"""
    if break_is_set("selftest-registry-by-path-tail"):
        return True
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    if resolved is None:
        return False
    return os.path.realpath(resolved) == os.path.realpath(os.path.join(SELFTEST_REGISTRY_ROOT, key))


def classify_selftest(word, arguments, directory, environment=None):
    """研究脚本、钩子、门禁共用库的自证：不是自证交 (False, None)；是自证交 (True, HeavyTest 或 None)。None 是放行（登记成 light、写成单独一个 --selftest）。
    门禁共用库直接执行就是跑自证，不带参数或单独一个 --selftest 算「单独」；别的两处只认参数正好是 ["--selftest"]。
    登记了的还要命令词解到登记表所在仓里那一份文件（selftest_registered_file），不是的照表外拒；命令词带通配的不当自证（弄坏开关 selftest-pattern-words-judged 下照判）。"""
    if SELFTEST_PATTERN_CHARACTERS & set(word) and not break_is_set("selftest-pattern-words-judged"):
        return False, None   # 通配写出的命令词：展开之前判不出起的是哪一份（case 的模式行切成命令时也是这样的词），不当自证
    key = selftest_registry_key(word, directory)
    if key is None:
        return False, None
    gate_library = os.path.dirname(key) == GATE_LIBRARY_DIRECTORY
    if not gate_library and "--selftest" not in arguments:
        return False, None
    if break_is_set("selftest-registry-ignored"):
        return True, heavy_test("commit-check-selftest", f"{word} --selftest")
    alone = arguments in ([], ["--selftest"]) if gate_library else arguments == ["--selftest"]
    if break_is_set("selftest-extra-arguments-allowed"):
        alone = True
    row = read_selftest_registry(SELFTEST_REGISTRY_PATH)[0].get(key)
    written = f"{key}{' ' + ' '.join(arguments) if arguments else ''}"
    if row is not None and not selftest_registered_file(word, directory, key):
        return True, heavy_test(UNREGISTERED_SELFTEST_KIND, f"{word}{' ' + ' '.join(arguments) if arguments else ''}（解到的不是登记表登记的那一份 "
                                                            f"{os.path.join(SELFTEST_REGISTRY_ROOT, key)}：别处路径尾部相同的脚本、路径写在变量里或当前目录认不出的"
                                                            "不按登记表放行；跑本仓那一份，写它在本仓的字面路径）")
    if row is None:
        if break_is_set("selftest-unregistered-as-light"):
            return True, None
        return True, heavy_test(UNREGISTERED_SELFTEST_KIND, f"{written}（这份自证没在登记表 .claude/hooks/{SELFTEST_REGISTRY_NAME} 里）")
    if not alone:
        return True, heavy_test(UNREGISTERED_SELFTEST_KIND, f"{written}（带了 --selftest 之外的参数：登记表只管单独一个 --selftest，"
                                                            f"{'门禁共用库只管不带参数或单独一个 --selftest' if gate_library else '脚本只认头一个参数时跑的是正事'}）")
    heavy = row.weight == SELFTEST_HEAVY
    if row.weight.startswith(SELFTEST_HEAVY_IF_SET) and not break_is_set("selftest-condition-ignored"):
        variable = row.weight[len(SELFTEST_HEAVY_IF_SET):]
        heavy = bool((environment or {}).get(variable))
    if break_is_set("selftest-heavy-rows-as-light"):
        heavy = False
    if heavy:
        return True, heavy_test("commit-check-selftest", f"{written}（登记表第 {row.line} 行记它起重活：{row.reason}）")
    return True, None


def selftest_judged_light(words, directory, environment=None):
    """这一条是不是登记成不起重活、又写成单独一个 --selftest 的自证（classify_selftest 判它放行）。弄坏开关 commit-checks-ignored 下一律为假。"""
    if break_is_set("commit-checks-ignored"):
        return False
    is_selftest, test = classify_selftest(words[0], words[1:], directory, environment)
    return is_selftest and test is None


def stage_samples_weight_is_valid(weight):
    """门禁阶段那一行的判定认不认得：samples-light、samples-heavy，或 samples-light-marker: 后面跟一个相对样本根、不带 .. 与空白的路径。"""
    if weight in (STAGE_SAMPLES_LIGHT, STAGE_SAMPLES_HEAVY):
        return True
    if not weight.startswith(STAGE_SAMPLES_MARKER):
        return False
    marker = weight[len(STAGE_SAMPLES_MARKER):]
    return bool(marker) and not os.path.isabs(marker) and ".." not in marker.split("/") and not any(character.isspace() for character in marker)


def stage_samples_row(stage):
    """登记表里 .claude/gate.d/<阶段> 那一行（SelftestRow）；没登记交 None。"""
    return read_selftest_registry(SELFTEST_REGISTRY_PATH)[0].get(f"{GATE_STAGE_DIRECTORY}/{stage}")


def same_as_registered_stage(path, stage):
    """要起的阶段文件与这份文件所在仓里那一份（登记的就是它）逐字节相同；读不了交 False。弄坏开关 stage-samples-content-unchecked 下一律为真。"""
    if break_is_set("stage-samples-content-unchecked"):
        return True
    try:
        with open(path, "rb") as candidate, open(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, stage), "rb") as registered:
            return candidate.read() == registered.read()
    except OSError:
        return False


def stage_sample_root(arguments, directory):
    """阶段的参数里点名的样本根：当前目录认得出、参数里每一个解到已有目录的词都是当前目录、至少有一个，交它的实际路径；
    有词带没展开的变量、通配，或解到别的目录、一个都没有（阶段默认判真仓），交 None。"""
    if directory is None:
        return None
    here, named = os.path.realpath(directory), []
    for argument in arguments:
        if argument.startswith("-"):
            continue
        if UNRESOLVED_WORD_CHARACTERS & set(argument):
            return None
        resolved = shell_words.resolve_path(directory, os.path.expanduser(argument))
        if resolved and os.path.isdir(resolved):
            named.append(os.path.realpath(resolved))
    return here if named and all(path == here for path in named) else None


def stage_sample_marker_matches(stage, root, marker):
    """样本根里的标记是不是仓里那一道样本放的那种（文件头「门禁样本」②）：仓里 fixtures/<阶段>/*/<标记> 是目录的，样本根里的也要是目录，
    STAGE_SAMPLE_FAKE_TOOLS 里那几个假工具要可执行、与仓里某份样本标记目录里的同名文件逐字节相同；是文件的，样本根里的也要是文件、与仓里某份逐字节相同。
    仓里没有这一道的样本、标记是目录而那一道没登记要核的假工具，交 False。弄坏开关 stage-samples-marker-unchecked 下只看标记在不在。"""
    candidate = os.path.join(root, marker)
    if break_is_set("stage-samples-marker-unchecked"):
        return os.path.lexists(candidate)
    references = glob.glob(os.path.join(glob.escape(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "fixtures", stage)), "*", glob.escape(marker)))
    def content(path):
        try:
            with open(path, "rb") as handle:
                return handle.read()
        except OSError:
            return None
    if os.path.isdir(candidate):
        tools = STAGE_SAMPLE_FAKE_TOOLS.get(stage)
        directories = [reference for reference in references if os.path.isdir(reference)]
        if not tools or not directories:
            return False
        for tool in tools:
            path = os.path.join(candidate, tool)
            body = content(path) if os.path.isfile(path) and os.access(path, os.X_OK) else None
            if body is None or not any(content(os.path.join(reference, tool)) == body for reference in directories):
                return False
        return True
    if os.path.isfile(candidate):
        body = content(candidate)
        return body is not None and any(os.path.isfile(reference) and content(reference) == body for reference in references)
    return False


def stage_sample_run_allowed(stage, word, arguments, directory):
    """直接把一道门禁阶段指到它的样本根（文件头「门禁样本」②）：登记成 samples-light-marker:<标记>、当前目录是样本根、根参数写同一个目录、
    样本根里的标记是仓里那一道样本放的那种（stage_sample_marker_matches）、阶段文件与登记的那一份相同，交 True（放行）；别的交 False，照旧按那一道判。"""
    if break_is_set("stage-samples-ignored"):
        return False
    row = stage_samples_row(stage)
    if row is None or not row.weight.startswith(STAGE_SAMPLES_MARKER):
        return False
    if break_is_set("stage-samples-root-unchecked"):
        return True
    root = stage_sample_root(arguments, directory)
    if root is None or not stage_sample_marker_matches(stage, root, row.weight[len(STAGE_SAMPLES_MARKER):]):
        return False
    return same_as_registered_stage(shell_words.resolve_path(directory, word) or word, stage)


def sample_index_entries(repository, relative_directory):
    """repository 的暂存区（HEAD 加暂存）里 relative_directory 底下的每个文件：({相对它的路径: (模式, 对象号)}, "")；读不了交 (None, 一句原因)。
    清掉 GIT_ 开头的环境变量再起 git（GIT_DIR、GIT_INDEX_FILE 漏进来会读到别的仓或别的暂存区）。"""
    environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    try:
        done = subprocess.run(["git", "--literal-pathspecs", "-C", repository, "ls-files", "-s", "-z", "--", relative_directory + "/"],
                              capture_output=True, env=environment, timeout=60)
    except (OSError, subprocess.SubprocessError) as error:
        return None, f"读不了 {repository} 的暂存区（{error}）"
    if done.returncode != 0:
        return None, f"读不了 {repository} 的暂存区（git ls-files 退 {done.returncode}：{done.stderr.decode(errors='replace').strip()[:200]}）"
    entries, prefix = {}, relative_directory + "/"
    for record in done.stdout.split(b"\0"):
        if not record:
            continue
        metadata, _, path = record.partition(b"\t")
        mode, object_name, stage_number = metadata.decode().split()
        name = os.fsdecode(path)
        if stage_number != "0":
            return None, f"{repository} 的暂存区里 {name} 在合并冲突里"
        entries[name[len(prefix):]] = (mode, object_name)
    return entries, ""


def git_blob_name(data, object_name_length):
    """一段内容当 git 的 blob 时的对象号（对象号 64 位的是 sha256 仓，40 位的是 sha1 仓）。"""
    algorithm = hashlib.sha256 if object_name_length == 64 else hashlib.sha1
    return algorithm(b"blob %d\0" % len(data) + data).hexdigest()


def sample_directory_changes(sample_directory, stage):
    """喂的样本目录（<阶段目录>/fixtures/<阶段>）与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区里那一道的样本比：多出的（没跟踪的新文件，连同 setup.sh）、
    少了的、内容或执行位（符号链接的指向）不同的、不是普通文件的，逐个交一句（相对样本目录的路径与哪里不同）；读不了暂存区交 [一句]。
    弄坏开关 stage-samples-fixtures-unchecked 下一律交 []。"""
    if break_is_set("stage-samples-fixtures-unchecked"):
        return []
    entries, problem = sample_index_entries(SAMPLE_INDEX_REPOSITORY, f"{GATE_STAGE_DIRECTORY}/fixtures/{stage}")
    if entries is None:
        return [problem]
    changed, seen = [], set()
    for current, directories, files in os.walk(sample_directory):
        links = [name for name in directories if os.path.islink(os.path.join(current, name))]
        directories[:] = sorted(name for name in directories if name not in links)
        for name in sorted(files + links):
            path = os.path.join(current, name)
            relative = os.path.relpath(path, sample_directory)
            seen.add(relative)
            entry = entries.get(relative)
            if entry is None:
                changed.append(f"{relative}（暂存区里没有）")
                continue
            try:
                information = os.lstat(path)
                if stat.S_ISLNK(information.st_mode):
                    data, actual_mode = os.fsencode(os.readlink(path)), "120000"
                elif stat.S_ISREG(information.st_mode):
                    with open(path, "rb") as handle:
                        data = handle.read()
                    actual_mode = "100755" if information.st_mode & stat.S_IXUSR else "100644"
                else:
                    changed.append(f"{relative}（不是普通文件）")
                    continue
            except OSError as error:
                changed.append(f"{relative}（读不了：{error.strerror}）")
                continue
            if actual_mode != entry[0]:
                changed.append(f"{relative}（模式 {actual_mode}，暂存区里是 {entry[0]}）")
            elif git_blob_name(data, len(entry[1])) != entry[1]:
                changed.append(f"{relative}（内容与暂存区不同）")
    changed += [f"{relative}（暂存区里有、喂的样本里没有）" for relative in sorted(set(entries) - seen)]
    return changed


class StageSelftestArguments(NamedTuple):
    directories: list   # 当阶段目录的词，照写的样子（stage-selftest.sh 取最后一个）
    items: list | None  # --item 的值，照写的样子（缺值记空串）；一个 --item 都没写是 None
    list_only: bool     # 带了 --list-items


def stage_selftest_arguments(arguments):
    """照 stage-selftest.sh 自己的参数循环切：先照它开头的 preflight 摘掉每一个整词 --force；--item 后面还有词、又不是空串的，取它当值（`--item --list-items` 的值
    是 --list-items），缺值或值是空串记一个空名（它自己退 2）；--list-items 只列；别的词都当阶段目录。"""
    words = [argument for argument in arguments if argument != "--force"]
    directories, items, list_only, position = [], [], False, 0
    while position < len(words):
        word = words[position]
        if word == STAGE_SELFTEST_ITEM_OPTION:
            if position + 1 < len(words) and words[position + 1]:
                items.append(words[position + 1])
                position += 2
            else:
                items.append("")
                position += 1
            continue
        if word == STAGE_SELFTEST_LIST_OPTION:
            list_only = True
        else:
            directories.append(word)
        position += 1
    return StageSelftestArguments(directories, items or None, list_only)


def stage_selftest_directory(word, written, directory):
    """stage-selftest.sh 喂的阶段目录：(实际路径, "")；判不出交 (None, 一句原因)。written 是参数里当阶段目录的那个词，没写交 None（它自己算的那一个：<脚本所在包的根>/.claude/gate.d）。"""
    if written is not None and UNRESOLVED_WORD_CHARACTERS & set(written):
        return None, f"阶段目录 {written} 写在变量或通配里，判不出它下面有哪些阶段"
    if written is not None:
        gate_directory = shell_words.resolve_path(directory, os.path.expanduser(written))
    else:
        script = shell_words.resolve_path(directory, word)
        gate_directory = os.path.join(os.path.dirname(os.path.dirname(os.path.realpath(script))), ".claude", "gate.d") if script else None
    if gate_directory is None or not os.path.isdir(gate_directory):
        return None, f"阶段目录 {written if written is not None else '（它默认的那一个）'} 此刻不在，判不出它下面有哪些阶段"
    return gate_directory, ""


def stage_feed_problem(gate_directory, stage):
    """stage-selftest.sh 喂的阶段目录里配了样本的一道放不放行（文件头「门禁样本」①）：不放行交一句，放行交 ""。带不带 --item 都用这一份。"""
    row = stage_samples_row(stage)
    if row is None:
        return f"{stage} 的样本档没在登记表里"
    if row.weight == STAGE_SAMPLES_HEAVY and not break_is_set("stage-samples-heavy-as-light"):
        return f"{stage} 的样本档登记成起重活（第 {row.line} 行：{row.reason}）"
    if not same_as_registered_stage(os.path.join(gate_directory, stage), stage):
        return f"{stage} 与 {HOOK_REPOSITORY}/{GATE_STAGE_DIRECTORY}/{stage} 不是同一份"
    changes = sample_directory_changes(os.path.join(gate_directory, "fixtures", stage), stage)
    if changes:
        return (f"{stage} 的样本与 {SAMPLE_INDEX_REPOSITORY} 的 HEAD 加暂存区不同（{len(changes)} 处：{'、'.join(changes[:3])}"
                f"{' 等' if len(changes) > 3 else ''}）")
    return ""


def stage_selftest_is_own_copy(word, directory):
    """命令词解到的真实路径正是本仓那一份 stage-selftest.sh（--item / --list-items 放行信的是它只喂点名的几道、只列不喂）。
    弄坏开关 stage-items-script-by-name 下一律为真（照旧只按名字与位置认）。"""
    if break_is_set("stage-items-script-by-name"):
        return True
    resolved = shell_words.resolve_path(directory, os.path.expanduser(word))
    return resolved is not None and os.path.realpath(resolved) == os.path.realpath(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT]))


def stage_selftest_item_problems(word, parsed, directory, environment=None):
    """stage-selftest.sh 带 --item 或 --list-items（文件头「门禁样本」①）：不放行的原因一句，放行交 ""。只判点名后真喂的那几道；点名里没有一道是配了样本的阶段，
    它自己退 2、不喂样本，放行。弄坏开关 stage-items-unjudged 下点名的几道不判，stage-directory-pwd-unexpanded 下 $PWD 不换。"""
    if not stage_selftest_is_own_copy(word, directory):
        return (f"{word} 解到的不是本仓那一份 {os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])}：带 --item / --list-items 时信的是"
                "它只喂点名的几道、只列不喂，别处路径尾部相同的拷贝不信；跑本仓那一份，写它在本仓的字面路径")
    if parsed.list_only:
        return ""   # 只列可点名的阶段，不喂样本
    if len(parsed.directories) > 1:
        return f"参数里有 {len(parsed.directories)} 个当阶段目录的词（{'、'.join(parsed.directories)}），认不出喂的是哪一个"
    written = parsed.directories[0] if parsed.directories else ""
    if directory is not None and "PWD" not in (environment or {}) and not break_is_set("stage-directory-pwd-unexpanded"):
        written = WORKING_DIRECTORY_VARIABLE.sub(lambda _: directory, written, count=1)
    gate_directory, problem = stage_selftest_directory(word, written or None, directory)
    if problem:
        return problem
    if break_is_set("stage-items-unjudged"):
        return ""
    unresolved = [item for item in parsed.items if UNRESOLVED_WORD_CHARACTERS & set(item)]
    if unresolved:
        return f"--item {unresolved[0]} 写在变量或通配里，判不出点的是哪几道"
    requested = f" {' '.join(parsed.items)} "
    fed = [stage for stage in sorted(os.listdir(gate_directory))
           if stage.endswith(".sh") and not stage.startswith(".") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage)) and f" {stage} " in requested]
    return "；".join(problem for problem in (stage_feed_problem(gate_directory, stage) for stage in fed) if problem)


def stage_selftest_problems(word, arguments, directory, environment=None):
    """stage-selftest.sh 喂样本不放行的原因（文件头「门禁样本」①），一句；放行交 ""。带 --item / --list-items 的走 stage_selftest_item_problems
    （弄坏开关 stage-items-as-directory 下不认这两个参数，照不带的判）；不带的，命令词要解到本仓那一份 stage-selftest.sh（stage_selftest_is_own_copy，
    弄坏开关 stage-feed-script-by-name 下不核），阶段目录里配了样本的每一道都判。"""
    if break_is_set("stage-samples-ignored"):
        return "弄坏开关 stage-samples-ignored：门禁样本一律不认"
    parsed = stage_selftest_arguments(arguments)
    if (parsed.items is not None or parsed.list_only) and not break_is_set("stage-items-as-directory"):
        return stage_selftest_item_problems(word, parsed, directory, environment)
    if not break_is_set("stage-feed-script-by-name") and not stage_selftest_is_own_copy(word, directory):
        return (f"{word} 解到的不是本仓那一份 {os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])}：放行信的是它只喂样本、"
                "正文不读，别处路径尾部相同的拷贝不信；跑本仓那一份，写它在本仓的字面路径")
    positional = [argument for argument in arguments if not argument.startswith("-")]
    if len(positional) > 1:
        return f"参数里有 {len(positional)} 个像阶段目录的词，认不出喂的是哪一个"
    gate_directory, problem = stage_selftest_directory(word, positional[0] if positional else None, directory)
    if problem:
        return problem
    return "；".join(problem for problem in (stage_feed_problem(gate_directory, stage) for stage in sorted(os.listdir(gate_directory))
                                              if stage.endswith(".sh") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))) if problem)


def stage_samples_hint(stage, test):
    """这一道照旧被判成重型或提交时才跑的检查时，拒绝说明里接一句它的样本档怎么跑（登记表里有这一道的才接）。"""
    row = stage_samples_row(stage) if test is not None else None
    if row is None:
        return test
    if row.weight.startswith(STAGE_SAMPLES_MARKER):
        hint = (f"；要跑它的样本档：cd 进放了 {row.weight[len(STAGE_SAMPLES_MARKER):]} 的样本根、根参数写同一个目录（登记表第 {row.line} 行；标记里的假工具要可执行、与仓里那一道某份样本的逐字节相同），"
                f"或在仓根 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item {stage} 只喂它的样本（样本要与暂存区相同）")
    elif row.weight == STAGE_SAMPLES_HEAVY:
        hint = f"；它的样本档也起重活（登记表第 {row.line} 行：{row.reason}），样本照拒"
    else:
        hint = (f"；它的样本档只读文本（登记表第 {row.line} 行），只喂它的样本：在仓根 bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item {stage}"
                "（样本要与暂存区相同）")
    return test._replace(detail=test.detail + hint)


def test_binary_name(word):
    """直接执行的是 cargo 编出来的测试二进制时交回它的名字（去掉哈希），不是时交 None。"""
    match = TEST_BINARY_PATH.search(word)
    return match.group(1) if match else None


# 跑编译出来的代码的 cargo 子命令（t、r 是 test、run 的内置简写）；只编不跑的 build、check、clippy 不在内
COMPILED_CODE_SUBCOMMANDS = {"test", "t", "run", "r", "bench"}
# cargo 编出来的二进制直接执行：<名字里带 target 的目录>/[<目标三元组>/]<debug|release>/[deps/]<名字>；名字里不许有点（排掉 .d、.rlib、.so）。
# CARGO_TARGET_DIR 设成名字里不带 target 的目录、再直接执行里面的二进制，认不出（测试二进制按 TEST_BINARY_PATH 照认）
BUILT_BINARY_PATH = re.compile(r"(?:^|/)[^/]*target[^/]*/(?:[^/]+/)?(?:debug|release)/(?:deps/)?[A-Za-z0-9_][A-Za-z0-9_-]*$")


def runs_compiled_code(words, directory):
    """一条剥掉前缀与包装的命令会不会跑编译出来的代码（子 agent 要经 run-with-memory-cap.sh 跑的那一类）：会就交一句说明，不会交 None。
    cargo 的子命令按 test_invocation 认（nextest run、miri、llvm-cov、hack、mutants 同样算），另认 bench 与别名（展开之后看不见，当会跑）。"""
    if os.path.basename(words[0]) == "cargo":
        found = cargo_subcommand(words[1:], directory)
        if found is None:
            return None
        invocation, rest = test_invocation(found.subcommand, found.rest)
        if found.subcommand not in COMPILED_CODE_SUBCOMMANDS and invocation is None and not alias_of_subcommand(found, {}):
            return None
        options = rest[:rest.index("--")] if "--" in rest else rest
        if "--no-run" in options:
            return None
        return f"cargo {found.subcommand}"
    if TEST_BINARY_PATH.search(words[0]) or BUILT_BINARY_PATH.search(words[0]):
        return f"直接执行 cargo 编出来的二进制 {words[0]}"
    return None


# 包在命令外面、之后照样起那条命令的程序（lib_shell_words 那张前缀表之外的）：名字 → 它带一个值的选项。
# 这张表不进 lib_shell_words 的前缀表：bash-command-detector.sh 要按命令词认出 systemd-run，判它等不等结束。
LAUNCHER_OPTIONS_WITH_VALUE = {
    "time": {"-f", "--format", "-o", "--output"},
    "flock": {"-w", "--wait", "--timeout", "-E", "--conflict-exit-code"},
    "chrt": {"-T", "--sched-runtime", "-P", "--sched-period", "-D", "--sched-deadline"},
    "prlimit": {"-o", "--output", "-p", "--pid"},
    "systemd-run": {"-u", "--unit", "-p", "--property", "--description", "--slice", "-E", "--setenv", "-M", "--machine", "-H", "--host",
                    "--uid", "--gid", "--nice", "--working-directory", "--on-active", "--on-boot", "--on-startup", "--on-unit-active",
                    "--on-unit-inactive", "--on-calendar", "--path-property", "--socket-property", "--timer-property", "--service-type",
                    "-C", "--capsule", "--expand-environment"},
    "strace": {"-a", "-b", "-e", "-E", "--env", "-I", "-o", "-O", "-p", "-P", "-s", "-S", "-u", "-U", "-X",
               "--columns", "--detach-on", "--interruptible", "--stack-trace-frame-limit", "--output", "--summary-syscall-overhead", "--attach",
               "--trace-path", "--string-limit", "--summary-sort-by", "--user", "--summary-columns", "--const-print-style", "--syscall-limit",
               "--argv0", "--trace", "--trace-fds", "--abbrev", "--verbose", "--raw", "--signal", "--signals", "--status", "--read", "--write",
               "--fault", "--inject", "--kvm", "--decode-pids"},
    "perf": {"-e", "--event", "-o", "--output", "-p", "--pid", "-t", "--tid", "-C", "--cpu", "-r", "--repeat", "-I", "--interval-print",
             "-G", "--cgroup", "-x", "--field-separator", "-F", "--freq", "-c", "--count", "-m", "--mmap-pages", "-u", "--uid", "-j",
             "--branch-filter", "--call-graph", "-D", "--delay", "-M", "--metrics"},
    "rustup": set(),
}
# 各包装程序不带值（或值只能用 = 贴着写）的长选项：长选项可以只写一个前缀（getopt_long 认），前缀对不上这里的、却是某个要值的长选项的前缀时，
# 按要值算（下一个词是它的值）。两张表照各自的 --help（systemd-run 照 systemd 255 的 man 页）列；strace 那几个「要不要值」照 strace -o /dev/null --<选项> true 量过
LAUNCHER_LONG_OPTIONS_WITHOUT_VALUE = {
    "time": {"--append", "--portability", "--quiet", "--verbose", "--help", "--version"},
    "flock": {"--shared", "--exclusive", "--unlock", "--nonblock", "--nb", "--close", "--no-fork", "--verbose", "--help", "--version"},
    "chrt": {"--batch", "--deadline", "--fifo", "--idle", "--other", "--rr", "--reset-on-fork", "--all-tasks", "--max", "--pid", "--verbose",
             "--help", "--version"},
    "prlimit": {"--noheadings", "--raw", "--verbose", "--help", "--version", "--core", "--data", "--nice", "--fsize", "--sigpending", "--memlock",
                "--rss", "--nofile", "--msgqueue", "--rtprio", "--stack", "--cpu", "--nproc", "--as", "--locks", "--rttime"},
    "systemd-run": {"--collect", "--help", "--no-ask-password", "--no-block", "--on-clock-change", "--on-timezone-change", "--pipe", "--pty",
                    "--quiet", "--remain-after-exit", "--same-dir", "--scope", "--send-sighup", "--shell", "--slice-inherit", "--system", "--user",
                    "--version", "--wait"},
    "strace": {"--output-append-mode", "--summary-only", "--summary", "--debug", "--daemonize", "--follow-forks", "--output-separately", "--help",
               "--instruction-pointer", "--kill-on-exit", "--stack-trace", "--syscall-number", "--relative-timestamps", "--absolute-timestamps",
               "--timestamps", "--syscall-times", "--no-abbrev", "--version", "--summary-wall-clock", "--strings-in-hex", "--pidns-translation",
               "--successful-only", "--failed-only", "--failing-only", "--seccomp-bpf", "--tips", "--quiet", "--silent", "--silence", "--decode-fds"},
    "perf": set(),
    "rustup": set(),
}
# 弄坏开关 launcher-long-options-partial 下从要值的那张表里拿掉的（补全之前的表），同时不认长选项的前缀
LAUNCHER_LONG_OPTIONS_COMPLETED = {
    "systemd-run": {"--expand-environment"},
    "strace": {"--columns", "--detach-on", "--interruptible", "--stack-trace-frame-limit", "--output", "--summary-syscall-overhead", "--attach",
               "--trace-path", "--string-limit", "--summary-sort-by", "--user", "--summary-columns", "--const-print-style", "--syscall-limit",
               "--argv0", "--trace", "--trace-fds", "--abbrev", "--verbose", "--raw", "--signal", "--signals", "--status", "--read", "--write",
               "--fault", "--inject", "--kvm", "--decode-pids"},
}
# 选项之后、那条命令之前还有几个位置参数：flock 的锁文件、chrt 的优先级、rustup run 的工具链
LAUNCHER_POSITIONAL_COUNT = {"flock": 1, "chrt": 1, "rustup": 1}
# 给里面那条命令设环境变量的选项（值是 NAME=VALUE）：设的变量写在交回那条命令的最前面
LAUNCHER_ENVIRONMENT_OPTIONS = {"systemd-run": {"-E", "--setenv"}, "strace": {"-E", "--env"}}
# 交回的命令以 NAME=VALUE 打头时，那是设的变量、不是程序（值里的路径 basename 碰巧叫 strace 也不当包装剥）
ENVIRONMENT_ASSIGNMENT_WORD = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
# perf 起一条命令的子命令；后面还要再跟一个 record 的那几组
PERF_SUBCOMMANDS_RUNNING_A_COMMAND = {"stat", "record", "trace"}
PERF_GROUPS_WITH_RECORD = {"mem", "c2c", "sched", "lock", "kmem", "kwork"}


def resolved_long_option(option, options_with_value, options_without_value):
    """一个长选项词（`=` 之前那一段）：交回 (认成的选项名, 要不要值)。整词在两张表里的照表；都不在时按前缀认（getopt_long 认唯一的前缀）：
    恰好是一个已知长选项的前缀就当那一个；是几个的前缀时程序自己报「有歧义」、一条都不起，其中有要值的就按要值算（怎么算都一样）。
    弄坏开关 launcher-long-options-partial 下只按整词查表。"""
    if option in options_with_value:
        return option, True
    if option in options_without_value or break_is_set("launcher-long-options-partial") or not option.startswith("--"):
        return option, False
    matches = sorted(candidate for candidate in options_with_value | options_without_value if candidate.startswith("--") and candidate.startswith(option))
    if len(matches) == 1:
        return matches[0], matches[0] in options_with_value
    return option, any(candidate in options_with_value for candidate in matches)


def launcher_option(word, next_word, options_with_value, options_without_value=frozenset()):
    """包装程序的一个选项词：交回 (选项名, 它的值, 这个选项占几个词)。长选项 `--名=值` 与 `--名 值` 查两张表（resolved_long_option，认前缀，
    交回的选项名是认成的那一个）；短选项合写（`-fo 文件`、`-xw 10`、`-o文件`）逐个字母查表，头一个要值的字母之后剩下的是它的值，没剩下就取下一个词；
    一个要值的字母都没有的只占自己。"""
    if word.startswith("--") or break_is_set("launcher-whole-word-options"):
        option, has_attached_value, attached_value = word.partition("=")
        option, takes_value = resolved_long_option(option, options_with_value, options_without_value)
        if has_attached_value:
            return option, attached_value, 1
        return (option, next_word, 2) if takes_value else (option, "", 1)
    for index, letter in enumerate(word[1:], start=1):
        if "-" + letter in options_with_value:
            attached_value = word[index + 1:]
            return ("-" + letter, attached_value, 1) if attached_value else ("-" + letter, next_word, 2)
    return word, "", 1


# systemd-run -p EnvironmentFile= 读不了时替它交的那一个变量：名字照 CARGO_TARGET_*_RUNNER 的形状，判法按「环境变量里定了 runner」算（判不出，按宽）
UNREADABLE_ENVIRONMENT_FILE_RUNNER = "CARGO_TARGET_UNREADABLE_ENVIRONMENT_FILE_RUNNER"


def assignments_of_environment_property(value):
    """systemd-run -p / --property 的一个值设给里面那条命令的 NAME=VALUE：`Environment=A=1 "B=2 3"` 按 systemd 的引号规矩切；
    `EnvironmentFile=[-]<文件>` 按内容读（一行一个 NAME=VALUE，# 与 ; 起头的行、空行不算，行首的 export 去掉）；
    文件读不了（不在、没权限）交一个 UNREADABLE_ENVIRONMENT_FILE_RUNNER=<文件>：里面设了什么判不出，按设了 runner 算。别的属性交空表。"""
    name, _equals, rest = value.partition("=")
    if name == "Environment":
        try:
            words = shlex.split(rest)
        except ValueError:
            words = rest.split()
        return [word for word in words if ENVIRONMENT_ASSIGNMENT_WORD.match(word)]
    if name == "EnvironmentFile":
        path = rest[1:] if rest.startswith("-") else rest
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                lines = handle.read().split("\n")
        except OSError:
            return [f"{UNREADABLE_ENVIRONMENT_FILE_RUNNER}={path}"]
        assignments = []
        for line in lines:
            line = line.strip()
            if not line or line.startswith(("#", ";")):
                continue
            line = line[len("export "):].lstrip() if line.startswith("export ") else line
            if ENVIRONMENT_ASSIGNMENT_WORD.match(line):
                key, _separator, assigned = line.partition("=")
                assignments.append(f"{key}={assigned.strip().strip(chr(34)).strip(chr(39))}")
        return assignments
    return []


# research/scripts/detached-run.sh start [--label <名>] [--log <文件>] [--stop-timeout <秒>] -- <命令…>：把那条命令挂到用户级 systemd 的服务单元下跑，
# 调用方的 SINGLEFS_HEAVY_TESTS 与当前目录带进去；它的 preflight 摘掉每一个 --force（连 `--` 之后的），切参数时照摘，判的就是单元里真跑的那条。
# 没有 `--`、带认不出的选项的，它按用法错拒、什么都不跑。
# 认脚本与切参数都在 lib_shell_words（detached_run_script、detached_run_start_command），bash-command-detector.sh 用同一份


def command_under_detached_run(words, directory=None, recognized=False):
    """words[0] 是 detached-run.sh（lib_shell_words 的 detached_run_script：名字、指到它的符号链接、开头带签名行的改名拷贝；后两种按 directory 解相对路径、读盘；
    调用方已经按别的办法认出来的给 recognized=True）而参数是 `[--force] start <选项> -- <命令…>` 时交回 (<命令…> 的词, None)：当前目录与环境照外面那一条；
    别的交 None。弄坏开关 detached-run-opaque 下一律交 None；detached-run-by-name-only 下只按名字认。"""
    if break_is_set("detached-run-opaque") or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    if not recognized and not shell_words.detached_run_script(words[0], directory, by_name_only=break_is_set("detached-run-by-name-only")):
        return None
    inner = shell_words.detached_run_start_command(words[1:])
    return None if inner is None else (inner, None)


def command_under_launcher(words, directory=None):
    """words[0] 是 LAUNCHER_OPTIONS_WITH_VALUE 里的程序时，剥掉它（连同它的选项与位置参数），交回 (它起的那条命令的词, 那条命令的当前目录：
    None 是不变，systemd-run --working-directory 给的是相对当前目录的写法)；不是这几样、或后面没有要起的命令（strace -p、flock 只给 fd）交 None。
    选项词怎么切见 launcher_option；LAUNCHER_ENVIRONMENT_OPTIONS 里的选项（systemd-run 的 -E / --setenv、strace 的 -E / --env）设的 NAME=VALUE，
    与 systemd-run -p / --property 的 Environment= / EnvironmentFile= 设的（assignments_of_environment_property）写在交回那条命令的最前面。
    flock 的 -c <字符串> 交回成 sh -c <字符串>。rustup 只认 rustup run，perf 只认 stat / record / trace 与 <组> record。
    research/scripts/detached-run.sh 的 start 照 command_under_detached_run 剥（directory 是 words 的当前目录，认改名拷贝与符号链接时解相对路径用）。"""
    detached = command_under_detached_run(words, directory)
    if detached is not None:
        return detached
    name = os.path.basename(words[0])
    if name not in LAUNCHER_OPTIONS_WITH_VALUE or ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    position = 1
    if name == "rustup":
        if words[1:2] != ["run"]:
            return None
        position = 2
    elif name == "perf":
        if words[1:2] and words[1] in PERF_SUBCOMMANDS_RUNNING_A_COMMAND:
            position = 2
        elif words[1:2] and words[1] in PERF_GROUPS_WITH_RECORD and words[2:3] == ["record"]:
            position = 3
        else:
            return None
    options_with_value, working_directory, assignments = LAUNCHER_OPTIONS_WITH_VALUE[name], None, []
    if break_is_set("launcher-long-options-partial"):
        options_with_value = options_with_value - LAUNCHER_LONG_OPTIONS_COMPLETED.get(name, set())
    options_without_value = LAUNCHER_LONG_OPTIONS_WITHOUT_VALUE.get(name, set())
    environment_options = set() if name == "strace" and break_is_set("strace-drops-env") else LAUNCHER_ENVIRONMENT_OPTIONS.get(name, set())
    while position < len(words):
        word = words[position]
        if word == "--":
            position += 1
            break
        if not word.startswith("-") or word == "-":
            break
        option, value, word_count = launcher_option(word, words[position + 1] if position + 1 < len(words) else "", options_with_value,
                                                    options_without_value)
        if name == "systemd-run" and option == "--working-directory":
            working_directory = value
        if option in environment_options and "=" in value and not break_is_set("launcher-drops-setenv"):
            assignments.append(value)
        if name == "systemd-run" and option in ("-p", "--property") and not break_is_set("launcher-drops-property-environment"):
            assignments += assignments_of_environment_property(value)
        position += word_count
    if name == "flock" and position + 1 < len(words) and words[position + 1] in ("-c", "--command"):
        return (["sh", "-c", words[position + 2]], working_directory) if position + 2 < len(words) else None
    position += LAUNCHER_POSITIONAL_COUNT.get(name, 0)
    return (assignments + words[position:], working_directory) if position < len(words) else None


SHARD_DRIVER_CASE_KEY = re.compile(r"^crash-case:[\w.-]+$")
SHARD_DRIVER_REGISTRATION = os.path.join(".claude", "gate.d", "stage-inputs.tsv")


def shard_driver_runs_checker_tier(arguments, directory):
    """research/scripts/layer0-shard-run.sh 的参数（`<键> <树根>` 或 `--merged-log <键> <树根> …`）：按它开跑要读的那一行登记判这一趟跑不跑 checker 档
    （树根的 .claude/gate.d/stage-inputs.tsv 里键那一行，与驱动脚本同一处、同一个键）。
    True：登记的包是 checker 档包、带 shard=across-machines；False：登记的是别的包（驱动自检在临时仓里登记的替身用例），或树根里没有这一行、
    没登记 shard=across-machines（驱动开跑就拒，什么都不跑）；None：参数形态认不出、键不是字面写的、树根解不出或那份登记读不了（调用方照重型算）。"""
    positional = arguments[1:] if arguments[:1] == ["--merged-log"] else arguments
    if len(positional) < 2 or not SHARD_DRIVER_CASE_KEY.match(positional[0]):
        return None
    tree_root = shell_words.resolve_path(directory, os.path.expanduser(positional[1]))
    if tree_root is None:
        return None
    try:
        with open(os.path.join(tree_root, SHARD_DRIVER_REGISTRATION), encoding="utf-8") as handle:
            row = next((line.rstrip("\n").split("\t") for line in handle if line.split("\t", 1)[0] == positional[0]), None)
    except (OSError, UnicodeDecodeError):
        return None
    if row is None or len(row) < 3:
        return False
    tokens = row[2].split()
    package = next((token[len("test="):].split(":", 1)[0] for token in tokens if token.startswith("test=")), "")
    return "shard=across-machines" in tokens and package == CHECKER_PACKAGE_NAME


def broad_mutation_driver(word, directory, environment=None):
    """命令词是不是广谱变异的逐条驱动：是就交一句「凭什么认出来的」，不是交 None。三种办法（文件头「广谱变异的逐条驱动」那一段）：
    名字与位置（当前目录与命令文本里设的 PYTHONPATH 各试一遍：`python3 -m broad-mutation` 交来的就是这个名字）、
    解到的文件的真实路径（符号链接）、文件开头的签名行（改名或拷走的一份）。后两种读盘，文件不在就认不出。"""
    expanded = os.path.expanduser(word)
    bases = [directory]
    for entry in ((environment or {}).get("PYTHONPATH") or "").split(os.pathsep):
        if entry:
            bases.append(shell_words.resolve_path(directory, os.path.expanduser(entry)))
    location = f"{BROAD_MUTATION_SCRIPT_DIRECTORY}/{BROAD_MUTATION_SCRIPT_NAME}"
    if os.path.basename(expanded) == BROAD_MUTATION_SCRIPT_NAME and any(located_in(expanded, base, BROAD_MUTATION_SCRIPT_DIRECTORY) for base in bases):
        return location
    if break_is_set("broad-mutation-by-name-only"):
        return None
    for base in bases:
        resolved = shell_words.resolve_path(base, expanded)
        if resolved is None or not os.path.isfile(resolved):
            continue
        real = os.path.realpath(resolved)
        if real.endswith("/" + location):
            return f"{word}（符号链接，指到 {location}）"
        try:
            with open(real, "rb") as handle:
                head = handle.read(BROAD_MUTATION_HEAD_BYTES)
        except OSError:
            continue
        if BROAD_MUTATION_SIGNATURE_LINE.search(head):
            return f"{word}（开头带 {location} 的签名行：改名或拷走的一份）"
    return None


def broad_mutation_use(arguments, driver):
    """认出是广谱变异的驱动之后按参数判：--selftest 是提交时才跑的检查（走到这里的是改名拷贝、符号链接、python -c 起的；仓内位置上那一份在这之前已由
    classify_selftest 照登记表判完）；只读的（--list-items、--summary、--help）不算；其余一律算。"""
    given = set(arguments)
    if "--selftest" in given and not break_is_set("commit-checks-ignored"):
        return heavy_test("commit-check-selftest", f"{driver} --selftest")
    if break_is_set("broad-mutation-first-round-exemptions"):
        if BROAD_MUTATION_FIRST_ROUND_QUERY_ARGUMENTS & given or {"--item", "--skip-stage-two"} <= given:
            return None
    query_arguments = (BROAD_MUTATION_QUERY_ARGUMENTS - {"--check-ignored"} if break_is_set("broad-mutation-read-only-counted")
                       else BROAD_MUTATION_QUERY_ARGUMENTS)
    if query_arguments & given:
        return None
    return heavy_test("broad-mutation", f"{driver}（逐条改坏 core / checker 的源码、每条起 cargo；--sync-tree 与 --import-results 改库目录里全局的版本指针与结果表；"
                                        "只有 --list-items、--summary、--check-ignored、--help（-h）不算）")


def e165_pin_mode(arguments):
    """e165-pin-red-examples.py 的参数照它 main 的切法走一遍：--check、--replay 里写在后面的那个定模式，带值的选项跳过它的值；交 append、check 或 replay。"""
    mode, position = "append", 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("--check", "--replay"):
            mode = argument[2:]
            position += 1
        else:
            position += 2 if argument in E165_PIN_OPTIONS_WITH_VALUE else 1
    return mode


def verus_judge_script(word, directory, environment=None):
    """命令词是不是 Verus 那一道的两份判法脚本之一：是就交 (脚本名, 凭什么认出来的)，不是交 None。认法照 broad_mutation_driver：
    名字与位置（当前目录与命令文本里设的 PYTHONPATH 各试一遍）、解到的文件的真实路径落在 research/scripts/<那一份>（符号链接；读盘，文件不在就认不出）。"""
    expanded = os.path.expanduser(word)
    bases = [directory]
    for entry in ((environment or {}).get("PYTHONPATH") or "").split(os.pathsep):
        if entry:
            bases.append(shell_words.resolve_path(directory, os.path.expanduser(entry)))
    for script in (VERUS_PROOF_CHECK_SCRIPT, E165_PIN_SCRIPT):
        if os.path.basename(expanded) == script and any(located_in(expanded, base, VERUS_JUDGE_DIRECTORY) for base in bases):
            return script, f"{VERUS_JUDGE_DIRECTORY}/{script}"
    for base in bases:
        resolved = shell_words.resolve_path(base, expanded)
        if resolved is None or not os.path.isfile(resolved):
            continue
        real = os.path.realpath(resolved)
        for script in (VERUS_PROOF_CHECK_SCRIPT, E165_PIN_SCRIPT):
            if real.endswith(f"/{VERUS_JUDGE_DIRECTORY}/{script}"):
                return script, f"{word}（符号链接，指到 {VERUS_JUDGE_DIRECTORY}/{script}）"
    return None


def verus_judge_arguments(script, arguments, how):
    """认出是两份判法脚本之一之后按参数判：只有字面写明的只读用法交 None，--selftest 交提交时才跑的检查（走到这里的是改名链接、PYTHONPATH、python -c 起的；
    仓内位置上那一份在这之前已由 classify_selftest 照登记表判完），其余一律交 verus-stage。
    先照两份脚本开头的 preflight 摘掉每一个整词 --force（它在 main 之前从 sys.argv 里摘）；弄坏开关 verus-judges-force-kept 下不摘。"""
    if not break_is_set("verus-judges-force-kept"):
        arguments = [argument for argument in arguments if argument != "--force"]
    literal = not any("$" in argument or "`" in argument for argument in arguments)
    if literal and arguments == ["--selftest"] and not break_is_set("commit-checks-ignored"):
        return heavy_test("commit-check-selftest", f"{how} --selftest")
    if script == VERUS_PROOF_CHECK_SCRIPT and literal and arguments[:1] and arguments[0] in VERUS_PROOF_CHECK_READ_ONLY_SUBCOMMANDS:
        return None
    if script == E165_PIN_SCRIPT and literal and e165_pin_mode(arguments) in ("append", "check"):
        return None
    return heavy_test("verus-stage", f"{how} {' '.join(arguments[:3])}（门禁 checker-tier-formal-verification 的判法：起 Verus 或编 checker 档装置重放；"
                                     "只有字面写明的 toolchain、fingerprint、tool-versions 与红例钉表的追加、--check 不算）")


def verus_judge_use(name, word, arguments, directory, environment=None):
    """Verus 那一道的两份判法脚本直接起：认出是这两份（verus_judge_script）就照 verus_judge_arguments 判，别的脚本交 None。
    弄坏开关 verus-judges-by-stage-only 下一律交 None；verus-judges-by-working-subcommand 下照旧只按名字与位置认、只算点名的干活子命令。"""
    if break_is_set("verus-judges-by-stage-only"):
        return None
    if break_is_set("verus-judges-by-working-subcommand"):
        if not located_in(word, directory, VERUS_JUDGE_DIRECTORY):
            return None
        if name == VERUS_PROOF_CHECK_SCRIPT and arguments[:1] and arguments[0] in VERUS_PROOF_CHECK_WORKING_SUBCOMMANDS:
            return heavy_test("verus-stage", f"research/scripts/{name} {arguments[0]}（门禁 checker-tier-formal-verification 那一格的判法，逐处起 Verus）")
        if name == E165_PIN_SCRIPT and e165_pin_mode(arguments) == "replay":
            return heavy_test("verus-stage", f"research/scripts/{name} --replay（门禁 checker-tier-formal-verification 的 pinned-red-examples-replay 格：编 checker 档装置、逐行重放）")
        return None
    found = verus_judge_script(word, directory, environment)
    if found is None:
        return None
    script, how = found
    return verus_judge_arguments(script, arguments, how)


def code_only_prints_string_literals(code):
    """python -c 的代码每一句都是 print(<字符串字面量>, …)（不带 * / ** 展开，关键字参数的值也只许是字面量，end=""、sep="" 这类）：只打印字面写出的字，起不了别的。
    语法错、空代码交 False。弄坏开关 verus-judges-code-print-unchecked 下只看每一句是不是 print 调用，不看参数。"""
    try:
        tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return False

    def prints_literals(statement):
        call = statement.value if isinstance(statement, ast.Expr) else None
        if not (isinstance(call, ast.Call) and isinstance(call.func, ast.Name) and call.func.id == "print"):
            return False
        if break_is_set("verus-judges-code-print-unchecked"):
            return True
        return (all(isinstance(argument, ast.Constant) and isinstance(argument.value, str) for argument in call.args)
                and all(keyword.arg is not None and isinstance(keyword.value, ast.Constant) for keyword in call.keywords))

    return bool(tree.body) and all(prints_literals(statement) for statement in tree.body)


def verus_judge_code_use(name, arguments):
    """`python3 -c '…'` 的代码里出现两份判法脚本的名字（VERUS_JUDGE_CODE_NAME）：一律算 verus-stage，不看代码之后的词
    （代码能自己改 sys.argv、直接调函数）；只放过每一句都是 print(<字符串字面量>…) 的代码（code_only_prints_string_literals）。没出现交 None。
    弄坏开关 verus-judges-code-by-later-words 下照旧：只认文件名与 run_module（VERUS_JUDGE_CODE_REFERENCE），照那一份判代码之后的词（verus_judge_arguments）。"""
    if not shell_words.PYTHON_NAME.match(name) or break_is_set("verus-judges-by-stage-only") or break_is_set("verus-judges-by-working-subcommand"):
        return None
    invocation = shell_words.python_invocation(arguments)
    if invocation.code_string is None:
        return None
    if break_is_set("verus-judges-code-by-later-words"):
        named = [script for script, pattern in VERUS_JUDGE_CODE_REFERENCE.items() if pattern.search(invocation.code_string)]
        if not named:
            return None
        if len(named) > 1:
            return heavy_test("verus-stage", "python -c 的代码里同时点了 Verus 那一道的两份判法脚本")
        return verus_judge_arguments(named[0], arguments[invocation.script_index:], f"python -c 的代码里点了 {named[0]}")
    named = [script for script, pattern in VERUS_JUDGE_CODE_NAME.items() if pattern.search(invocation.code_string)]
    if not named or code_only_prints_string_literals(invocation.code_string):
        return None
    return heavy_test("verus-stage", f"python -c 的代码里点了 {'、'.join(named)}（门禁 checker-tier-formal-verification 的判法；代码能自己改 sys.argv、直接调函数，"
                                     "执行前判不出它跑哪种用法，不看代码之后的词，一律算；只打印字面字符串的不算）")


def command_word_expansions(word, directory):
    """命令词带通配时 bash 展开成的那几份路径（相对的按 directory 解、交回仍是相对的写法），排好序；零命中、相对路径而当前目录认不出的交 []
    （bash 照字面起）。只是 `[` / `[[` 的不展开（test 的内建命令）。bash 的 [^…] 换成 python glob 认的 [!…]（弄坏开关 pattern-caret-negation-literal 下不换）。
    带 POSIX 字符类、等价类、整理符号的（BASH_BRACKET_PATTERN）交给 bash 展开（bash_pattern_expansions，次序照 bash 的），判不出交 None
    （弄坏开关 posix-brackets-by-python-glob 下照旧交给 python glob）。"""
    if word in ("[", "[["):
        return []
    pattern = os.path.expanduser(word)
    if BASH_BRACKET_PATTERN.search(pattern) and not break_is_set("posix-brackets-by-python-glob"):
        return bash_pattern_expansions(pattern, directory)
    if not break_is_set("pattern-caret-negation-literal"):
        pattern = CARET_NEGATION.sub("[!", pattern)
    if os.path.isabs(pattern):
        return sorted(glob.glob(pattern))
    if directory is None:
        return []
    return sorted(glob.glob(pattern, root_dir=directory))


def bash_pattern_expansions(pattern, directory):
    """带 POSIX 字符类、等价类、整理符号的词交给 bash 展开成的那几份（次序照 bash 的，相对的仍是相对的写法）；零命中交 []。
    不起被判的命令：词经 $1 传进去、IFS 设空，只做路径展开；不读 rc 与 BASH_ENV。展不开或判不出的（相对路径而当前目录认不出、
    bash 起不来、退出码不是 0、超过 BASH_PATTERN_TIMEOUT 秒）交 None；弄坏开关 posix-brackets-unresolved-literal 下这几种也交 []（照字面判）。"""
    unresolved = [] if break_is_set("posix-brackets-unresolved-literal") else None
    absolute = os.path.isabs(pattern)
    if not absolute and directory is None:
        return unresolved
    environment = {key: value for key, value in os.environ.items() if key not in ("BASH_ENV", "ENV")}
    try:
        finished = subprocess.run([BASH_FOR_PATTERNS, "--norc", "--noprofile", "-c", 'shopt -s nullglob; IFS=; for f in $1; do printf "%s\\0" "$f"; done', "bash", pattern],
                                  cwd="/" if absolute else directory, env=environment, capture_output=True, timeout=BASH_PATTERN_TIMEOUT)
    except (OSError, subprocess.SubprocessError):
        return unresolved
    if finished.returncode != 0:
        return unresolved
    return [os.fsdecode(item) for item in finished.stdout.split(b"\0") if item]


def pattern_expansion_use(words, directory, environment=None):
    """命令词带通配（文件头「命令词带通配」）：按 directory 展开，bash 真起的那一条（头一份当命令、其余几份接在写的参数前面）与每一份单独当命令
    （带写的参数，最多 PATTERN_EXPANSION_LIMIT 份）各判一遍；有一条判成哪一类就交它（说明里写展开成了什么），都不是交 None（调用方再照字面判）。
    展开出来的那条当一段命令文本再切一遍、剥包装（写成通配的 capped.sh、nice、/usr/bin/time 这一类也认）；弄坏开关 pattern-expansions-bare 下直接 classify。"""
    matches = command_word_expansions(words[0], directory)
    if matches is None:
        where = "当前目录认不出" if directory is None and not os.path.isabs(os.path.expanduser(words[0])) else "交给 bash 展开没展成"
        return heavy_test("pattern-unresolved", f"命令词 {words[0]} 带方括号里的 POSIX 字符类、等价类或整理符号，{where}，判不出 bash 起的是哪一份"
                                                "（写成展开之后的字面路径再起）")
    if not matches:
        return None
    candidates = ([[*matches, *words[1:]]] if len(matches) > 1 else []) + [[match, *words[1:]] for match in matches[:PATTERN_EXPANSION_LIMIT]]
    for candidate in candidates:
        if GLOB_CHARACTERS & set(candidate[0]) or break_is_set("pattern-expansions-bare"):
            literal = classify(candidate, directory, environment, expand_patterns=False)
            tests = [literal] if literal else []
        else:
            tests = heavy_tests_in_command_text(shlex.join(candidate), directory, environment)
        if tests:
            more = f" 等 {len(matches)} 份" if len(matches) > 1 else ""
            where = "" if os.path.isabs(os.path.expanduser(words[0])) else f"按 {directory} "
            return tests[0]._replace(detail=f"{tests[0].detail}（命令词 {words[0]} {where}展开成 {candidate[0]}{more}，照展开的判）")
    return None


class BraceScanTooLong(Exception):
    """一个词里配不上的花括号太多，扫描步数超过 BRACE_SCAN_LIMIT：这个词不展开。"""


def brace_gobble(word, position, wanted, budget):
    """照 bash braces.c 的 brace_gobbler：从 position 起找最外层的 wanted（`{`、`}` 或 `,`），交下标，找不到交 None。
    `${` 算进一层、它的 `{` 不算开头；里层的 `{`、`}` 按层数配，最外层多出来的 `}` 当普通字符；
    找 `}` 时要先见过最外层的逗号或 `..`（后头紧跟的不是 `}`），之前见到的最外层 `}` 当普通字符（{}a,b} 配的是后头那个）。
    只看这几种记号（BRACE_TOKENS），每看一个扣 budget[0] 一步，扣完抛 BraceScanTooLong。"""
    level, seen = 0, wanted != "}"
    for found in BRACE_TOKENS.finditer(word, position):
        budget[0] -= 1
        if budget[0] < 0:
            raise BraceScanTooLong()
        token, at = found.group(), found.start()
        if token == "${":
            level += 1
        elif token == wanted and level == 0 and seen:
            if token == "{" and at == 0 and word[1:2] in ("}", ""):
                continue   # 打头的 `{}`（find -exec 的那种）与单独一个 `{` 照 bash 不算开头
            return at
        elif token == "{":
            level += 1
        elif token == "}":
            level -= 1 if level else 0
        elif wanted == "}" and level == 0 and (token == "," or word[at + 2:at + 3] != "}"):
            seen = True
    return None


def brace_sequence(amble):
    """{<起>..<止>[..<步长>]} 里面那一段照 bash 展开成的几份（最多 WORD_EXPANSION_LIMIT 份）；不是合法的序列交 None。
    整数照 bash 补零：起或止写成 0 起头的两位以上（或 -0 起头的三位以上）时，宽度取这样写的那几边的长度里大的那个。弄坏开关 brace-sequence-literal 下一律交 None。"""
    found = BRACE_SEQUENCE.fullmatch(amble)
    if found is None or break_is_set("brace-sequence-literal"):
        return None
    first, last, step = found.group(1), found.group(2), abs(int(found.group(3) or 1)) or 1
    if first.isalpha() != last.isalpha():
        return None
    if first.isalpha():
        begin, end, render = ord(first), ord(last), chr
    else:
        width = max([len(text) for text in (first, last) if (len(text) > 1 and text[0] == "0") or (len(text) > 2 and text[:2] == "-0")], default=0)
        begin, end = int(first), int(last)
        render = (lambda value: f"{value:0{width}d}") if width else str
    step = step if begin <= end else -step
    return [render(value) for value in range(begin, end + (1 if step > 0 else -1), step)[:WORD_EXPANSION_LIMIT]]


def brace_expansion(word):
    """一个词照 bash 的花括号展开成的几份（最多 WORD_EXPANSION_LIMIT 份），没有能展开的交 [word]；展开出来的空串照 bash 去掉。
    认的写法：{a,b}（最外层至少一个逗号，各份再各自展开）与序列 {1..3}、{a..e}、{01..10..3}；前后缀照接，几处花括号照乘。
    配不上的 `{`、里面最外层没有逗号又不是序列的（{a}、{}、{1...3}）照字面，接着往后找；`${…}` 是变量，不展开、里面也不找。
    引号与反斜杠在切词时已经去掉，照字面写的花括号（'{a,b}'、\\{a,b\\}）这里也展开（接受的误拒）。弄坏开关 brace-expansion-literal 下一律交 [word]。"""
    if break_is_set("brace-expansion-literal") or "{" not in word or "}" not in word or ("," not in word and ".." not in word):
        return [word]
    try:
        results = [result for result in brace_expansion_with_empties(word, [BRACE_SCAN_LIMIT]) if result]
    except BraceScanTooLong:
        return [word]
    return results[:WORD_EXPANSION_LIMIT]


def brace_expansion_with_empties(word, budget):
    """照 bash braces.c 的 brace_expand（空串留着，brace_expansion 再去掉）；budget 见 brace_gobble。"""
    opening = brace_gobble(word, 0, "{", budget)
    if opening is None:
        return [word]
    prefix, start = word[:opening], opening + 1
    close = brace_gobble(word, start, "}", budget)
    if close is None:   # 配不上：这个 `{` 照字面，接着往后找
        return [prefix + "{" + rest for rest in brace_expansion_with_empties(word[start:], budget)][:WORD_EXPANSION_LIMIT]
    amble = word[start:close]
    if "," in amble:   # 照 bash：里面哪一层有逗号都按逗号切（只切最外层的），各份再各自展开
        middles, position = [], 0
        while True:
            comma = brace_gobble(amble, position, ",", budget)
            middles += brace_expansion_with_empties(amble[position:len(amble) if comma is None else comma], budget)
            if comma is None:
                break
            position = comma + 1
    else:
        middles = brace_sequence(amble)
        if middles is None:
            if close + 1 >= len(word):
                return [word]
            middles = [word[opening:close + 1]]   # 不是序列（{1...3}）：照字面，后头接着展开
    suffixes = brace_expansion_with_empties(word[close + 1:], budget)
    return [prefix + middle + suffix for middle in middles[:WORD_EXPANSION_LIMIT] for suffix in suffixes][:WORD_EXPANSION_LIMIT]


def expanded_words(words, directory):
    """bash 对这一条命令的词做的两样展开：每个词的花括号展开（命令词展开成几份时，头一份是命令、其余接在参数前面），
    再把参数里带通配（*、?、[）的按 directory 展开（expanded_arguments；命令词的通配留给 pattern_expansion_use）。"""
    braced = [expanded for word in words for expanded in brace_expansion(word)]
    if not braced:
        return words
    return braced[:1] + expanded_arguments(braced[1:], directory)


def expanded_arguments(arguments, directory):
    """参数里带通配（*、?、[）的按 directory 展开（每个最多 WORD_EXPANSION_LIMIT 份），零命中、当前目录认不出的照字面。弄坏开关 argument-patterns-literal 下不展开。"""
    if break_is_set("argument-patterns-literal"):
        return list(arguments)
    result = []
    for word in arguments:
        matches = (command_word_expansions(word, directory) or []) if GLOB_CHARACTERS & set(word) else []   # 判不出（None）的照字面
        result += matches[:WORD_EXPANSION_LIMIT] or [word]
    return result


word_expansion_depth = 0   # word_expansion_use 套了几层（展开出来的文件名里本身带花括号或通配时，再展开有个头）


def word_expansion_use(words, directory, environment=None):
    """花括号与参数里的通配（文件头「花括号与参数里的通配」）：照 bash 展开（expanded_words），与写的不同就判 bash 真起的那一条；
    命令词自己花括号展开成几份的，再把每一份单独当命令、带展开过的参数各判一遍（最多 PATTERN_EXPANSION_LIMIT 份，与命令词带通配同一个口径）。
    命令词展开过的，展开出来的那条当一段命令文本再切一遍、剥包装（花括号写在 bash、nice 这类包装上的也认；弄坏开关 brace-command-words-bare 下直接 classify）；
    只有参数展开过的直接 classify（包装在切词时已经剥掉）。有一条判成哪一类就交它（说明里写展开成了什么；判成重型的先于只判成提交时才跑的检查），
    都不是、或没有能展开的交 None（调用方再照字面判）。"""
    global word_expansion_depth
    expanded = expanded_words(words, directory)
    if expanded == words or word_expansion_depth >= WORD_EXPANSION_DEPTH_LIMIT:
        return None
    alternatives = brace_expansion(words[0])
    candidates = [expanded]
    if len(alternatives) > 1:
        arguments = expanded_arguments([braced for word in words[1:] for braced in brace_expansion(word)], directory)
        candidates += [[alternative, *arguments] for alternative in alternatives[:PATTERN_EXPANSION_LIMIT]]
    resplit = alternatives != [words[0]] and not break_is_set("brace-command-words-bare")
    found = []   # (判出的那一类, 那一条)：重型的先交，几条都只判成提交时才跑的检查才交头一条
    word_expansion_depth += 1
    try:
        for candidate in candidates:
            if resplit:
                tests = heavy_tests_in_command_text(shlex.join(candidate), directory, environment)
            else:
                literal = classify(candidate, directory, environment)
                tests = [literal] if literal else []
            found += [(test, candidate) for test in tests]
            if any(test.kind not in COMMIT_CHECK_KINDS for test in tests):
                break
    finally:
        word_expansion_depth -= 1
    if not found:
        return None
    test, candidate = next(((test, candidate) for test, candidate in found if test.kind not in COMMIT_CHECK_KINDS), found[0])
    shown = shlex.join(candidate)
    shown = shown if len(shown) <= 200 else shown[:200] + " …"
    how = "bash 真起的那一条" if candidate is expanded else f"命令词 {words[0]} 展开出来的一份单独当命令"
    return test._replace(detail=f"{test.detail}（照 bash 的花括号展开与参数里的通配展开，{how}是 {shown}，照展开的判）")


def classify(words, directory, environment=None, expand_patterns=True):
    """一条已经剥掉前缀与包装的命令：返回 HeavyTest 或 None。environment 是这条命令看得到的变量（认 runner 与别名）。
    先照 bash 做花括号展开与参数里的通配展开、展开出来的那一条判成哪一类就交它（word_expansion_use）；
    命令词带通配（*、?、[）的再照 bash 展开、逐个判（pattern_expansion_use），都不重型再照字面判；expand_patterns 为假时只照字面判
    （展开出来的文件名里本身带通配字符时用，免得再展开一遍）。弄坏开关 pattern-command-words-literal 下命令词的通配不展开。"""
    if expand_patterns:
        expanded = word_expansion_use(words, directory, environment)
        if expanded is not None:
            return expanded
    if expand_patterns and GLOB_CHARACTERS & set(words[0]) and not break_is_set("pattern-command-words-literal"):
        expanded = pattern_expansion_use(words, directory, environment)
        if expanded is not None:
            return expanded
    name, arguments = os.path.basename(words[0]), words[1:]
    stage = gate_stage_name(words[0], directory)
    if stage is not None:
        if stage_sample_run_allowed(stage, words[0], arguments, directory):
            return None   # 指到样本根、样本根放了登记的假工具标记（文件头「门禁样本」②）
        return stage_samples_hint(stage, classify_gate_stage(stage, arguments))
    binary = test_binary_name(words[0])
    if binary is not None:
        if libtest_lists_only(arguments):
            return None
        if binary in checker_test_targets(directory or HOOK_REPOSITORY):
            return heavy_test("checker-tier-binary", f"直接执行 {CHECKER_PACKAGE_NAME} 包的测试二进制（{binary}）：checker 档，默认只在提交时跑")
        return None
    if name.startswith("qemu-system"):
        return heavy_test("qemu-system", name)
    if name == "vm-bench.sh":
        return heavy_test("vm-bench", "research/scripts/vm-bench.sh（--selftest 也起虚机）")
    if not break_is_set("commit-checks-ignored"):
        is_selftest, selftest_use = classify_selftest(words[0], arguments, directory, environment)
        if is_selftest:
            return selftest_use   # 自证判完就是判完：登记成 light 的不再落到下面按名字判的那几格（广谱变异的 --selftest 那一支）
    commit_check = classify_commit_check(name, words[0], arguments, directory, environment)
    if commit_check is not None:
        return commit_check
    verus_judge = verus_judge_use(name, words[0], arguments, directory, environment) or verus_judge_code_use(name, arguments)
    if verus_judge is not None:
        return verus_judge
    if name == "lkmm.sh":
        if arguments == LKMM_VERSION_ONLY_ARGUMENTS and not break_is_set("herd7-version-counted"):
            return None   # 只找 herd7、打它版本输出的第一行（stage-inputs.tsv 里 checker-tier-lkmm 的 environment= 起的那一条）
        return heavy_test("lkmm", ".claude/scripts/lkmm.sh")
    if name == "herd7":
        return heavy_test("herd7", "herd7")
    if name == "mutate.sh":
        for argument in arguments:
            resolved = shell_words.resolve_path(directory, argument) or argument
            if argument.endswith("crates/mutations.tsv") or resolved.endswith("/crates/mutations.tsv"):
                return heavy_test("crates-mutation-mutate", "research/scripts/mutate.sh 跑 crates/mutations.tsv 整表")
        return None
    if name in CRASH_AMPLIFICATION_SCRIPTS and not break_is_set("crash-amplification-unnamed"):
        if name == CRASH_AMPLIFICATION_LISTING_SCRIPT and CRASH_AMPLIFICATION_QUERY_ARGUMENTS.intersection(arguments):
            return None   # 只列分项，不清场、不跑
        return heavy_test("crash-amplification", f"research/scripts/{name}（崩溃放量：每一项经 crash-amplification-lib.sh 起 checker 档 cargo test；"
                                                 "只有 crash-amplification.sh --list-items 与登记表里的单独 --selftest 不算）")
    if name == QEMU_BUILD_SCRIPT and not break_is_set("qemu-build-unnamed"):
        return heavy_test("qemu-build", "research/scripts/build-patched-qemu.sh（真编 QEMU：取源码、在 TMPDIR 下约 1 GiB 的编译目录里按给的线程数编、装进私有目录；"
                                        "只有登记表里的单独 --selftest 不算）")
    if name == "check.sh":
        return heavy_test("check-sh", "check.sh（里面是全量 cargo test）")
    if name == "gate.sh":
        return heavy_test("gate-sh", " ".join(["gate.sh", *arguments]))
    if name == "gate-staged.sh":
        return None if "--selftest" in arguments else heavy_test("gate-staged", "research/scripts/gate-staged.sh（跑 gate.sh --staged）")
    if name == "layer0-shard-run.sh":
        if "--selftest" in arguments or (not break_is_set("shard-driver-by-name") and shard_driver_runs_checker_tier(arguments, directory) is False):
            return None
        return heavy_test(
            "checker-tier-cargo", "research/scripts/layer0-shard-run.sh（两台各跑一片 checker 档登记的崩溃枚举用例、带 --include-ignored，再本机 merge）")
    if name == "mutation-shard-run.sh":
        return None if "--selftest" in arguments else heavy_test(
            "crates-mutation-stage", "research/scripts/mutation-shard-run.sh（门禁 checker-tier-crates-mutation-replay.sh 的双机分片驱动：两台各跑一趟那一道、判 crates/mutations.tsv 的一部分）")
    if not break_is_set("broad-mutation-ignored"):
        driver = broad_mutation_driver(words[0], directory, environment)
        if driver is not None:
            return broad_mutation_use(arguments, driver)
        if shell_words.PYTHON_NAME.match(name) and not break_is_set("broad-mutation-by-name-only"):
            invocation = shell_words.python_invocation(arguments)
            if invocation.code_string is not None and BROAD_MUTATION_CODE_REFERENCE.search(invocation.code_string):
                return broad_mutation_use(arguments[invocation.script_index:], f"python -c 的代码里点了 {BROAD_MUTATION_SCRIPT_NAME}")
    if name in ("e152-file-system-benchmark", "e152-run.sh"):
        return heavy_test("e152", name)
    if name == "cargo":
        return cargo_use(arguments, directory, environment)
    return None


def heavy_tests_in_command_text(text, directory, environment=None):
    """一段命令文本里命令位置上的每一处重型用法：lib_shell_words 剥掉包装，command_under_launcher 剥掉 /usr/bin/time 这一类（剥完再切一遍），逐条 classify。"""
    found = []
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        launched = command_under_launcher(command.words, command.directory)
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            found += heavy_tests_in_command_text(shlex.join(inner_words), inner_start, command.environment)
            continue
        test = classify(command.words, command.directory, command.environment)
        if test:
            found.append(test)
    return found


def classify_process(argv, cwd):
    """一个在跑的进程（argv 与 cwd）里认出的重型用法；包装（bash 起脚本、bash -c、nice、timeout、env、capped.sh……）照共用切词剥掉，
    /usr/bin/time、flock、systemd-run 这一类照 command_under_launcher 剥掉。"""
    if not argv:
        return []
    # 进程的 argv[0] 是 time 时它就是那个程序（不是 bash 的关键字 time）：先按 argv 剥，再交给切词
    launched = command_under_launcher(argv, cwd)
    if launched is not None:
        inner_words, inner_directory = launched
        return classify_process(inner_words, shell_words.resolve_path(cwd, inner_directory) if inner_directory else cwd)
    return heavy_tests_in_command_text(shlex.join(argv), cwd)


def judged_by_name(word, directory):
    """按名字判的仓内脚本（门禁阶段、KNOWN_SCRIPT_LOCATIONS 里的脚本在仓内位置上）：判定在名字那一格做完，不再读脚本正文。"""
    if gate_stage_name(word, directory) is not None:
        return True
    location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))
    if location is None:
        return False
    return any(candidate == location or candidate.endswith("/" + location)
               for candidate in (os.path.normpath(word), shell_words.resolve_path(directory, word) or ""))


def build_sample_workspace(work):
    """仓根虚工作区四个成员：core、harness 档（singlefs-harness）、池级 checker（singlefs-checker，只有库）、checker 档（singlefs-checker-tier：
    两个集成测试目标、一个装置二进制、库）；research/ 是另一个虚工作区；mutants-sample/ 是两个成员的虚工作区，带一份 test_workspace = true 的 .cargo/mutants.toml。另有两份给 systemd-run -p EnvironmentFile= 读的环境文件。"""
    def write(relative, text):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-harness", "crates/singlefs-checker", "crates/singlefs-checker-tier"]\n'
                        'exclude = ["research"]\n')
    write("crates/singlefs-core/Cargo.toml", '[package]\nname = "singlefs-core"\nversion = "0.0.0"\n')
    write("crates/singlefs-core/tests/core_contract.rs", "")
    write("crates/singlefs-harness/Cargo.toml", '[package]\nname = "singlefs-harness"\nversion = "0.0.0"\n')
    write("crates/singlefs-harness/tests/overwrite_in_one_instance.rs", "#[test]\nfn a_daily_case() {}\n")
    write("crates/singlefs-checker/Cargo.toml", '[package]\nname = "singlefs-checker"\nversion = "0.0.0"\n')
    write("crates/singlefs-checker/src/lib.rs", "pub fn check_pool_image() {}\n")
    write("crates/singlefs-checker-tier/Cargo.toml", '[package]\nname = "singlefs-checker-tier"\nversion = "0.1.0"\n')
    write("crates/singlefs-checker-tier/src/lib.rs", "pub fn judge() {}\n")
    write("crates/singlefs-checker-tier/src/bin/e161_sample_device.rs", "fn main() {}\n")
    write("crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs",
          '#[test]\nfn quick_case() {}\n#[test]\n#[ignore = "崩溃枚举（样本）：平时不跑"]\nfn the_full_case() {}\n')
    write("crates/singlefs-checker-tier/tests/checker_quick_stream.rs", "#[test]\nfn a_quick_case() {}\n")
    write("alias.env", '# 样本：systemd-run -p EnvironmentFile= 读的\nCARGO_ALIAS_XT="test -p singlefs-checker-tier"\n')
    write("plain.env", "# 样本\nRUST_BACKTRACE=1\n")
    write("research/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["e7-index-bench"]\n')
    write("research/e7-index-bench/Cargo.toml", '[package]\nname = "e7-index-bench"\nversion = "0.0.0"\n')
    write("mutants-sample/Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["mutated", "other"]\n')
    write("mutants-sample/mutated/Cargo.toml", '[package]\nname = "mutated"\nversion = "0.0.0"\n')
    write("mutants-sample/other/Cargo.toml", '[package]\nname = "other"\nversion = "0.0.0"\n')
    write("mutants-sample/.cargo/mutants.toml", "# 样本：cargo mutants 每次测整个工作区\ntest_workspace = true\n")
    # 广谱变异的驱动（样本只有文件头）：仓内位置上的一份、指到它的两个符号链接（仓内改名、仓外）、带签名行的改名拷贝（仓内、仓外各一份）、
    # 删掉签名行的拷贝（罩不住的那一种）、不相干的 python 脚本
    driver_sample = ("#!/usr/bin/env python3\n# admission: always 样本\n# run-condition: command python3\n"
                     "# broad-mutation-driver-signature: 样本（lib_heavy_tests 自检造的）\nprint('sample')\n")
    write("research/scripts/broad-mutation.py", driver_sample)
    write("research/scripts/renamed-driver-copy.py", driver_sample)
    write("elsewhere/renamed-driver-copy.py", driver_sample)
    write("elsewhere/copy-without-signature.py", driver_sample.replace("# broad-mutation-driver-signature:", "# 签名行删掉了："))
    write("elsewhere/unrelated.py", "#!/usr/bin/env python3\nprint('broad-mutation-driver-signature: 不在行首的不算')\n")
    # detached-run.sh（样本只有文件头）：仓内位置上的一份、带签名行的改名拷贝（仓内、仓外）、指到仓内那一份的符号链接、删掉签名行的拷贝
    detached_sample = "#!/usr/bin/env bash\n# detached-run-script-signature: 样本（lib_heavy_tests 自检造的）\necho sample\n"
    write("research/scripts/detached-run.sh", detached_sample)
    write("research/scripts/dr-renamed.sh", detached_sample)
    write("elsewhere/dr-renamed.sh", detached_sample)
    write("elsewhere/dr-without-signature.sh", detached_sample.replace("# detached-run-script-signature:", "# 签名行删掉了："))
    os.symlink(os.path.join(work, "research", "scripts", "detached-run.sh"), os.path.join(work, "elsewhere", "dr-link.sh"))
    os.symlink("broad-mutation.py", os.path.join(work, "research", "scripts", "linked-driver.py"))
    os.symlink(os.path.join(work, "research", "scripts", "broad-mutation.py"), os.path.join(work, "elsewhere", "linked-driver.py"))
    # Verus 那一道的两份判法脚本（样本只有文件头）：仓内位置上的各一份、仓外指到判法脚本的符号链接、仓内改名指到红例钉表的符号链接
    write("research/scripts/verus-proof-check.py", "#!/usr/bin/env python3\n# 样本（lib_heavy_tests 自检造的）\nprint('sample')\n")
    write("research/scripts/e165-pin-red-examples.py", "#!/usr/bin/env python3\n# 样本（lib_heavy_tests 自检造的）\nprint('sample')\n")
    os.symlink(os.path.join(work, "research", "scripts", "verus-proof-check.py"), os.path.join(work, "elsewhere", "linked-proof-check.py"))
    os.symlink("e165-pin-red-examples.py", os.path.join(work, "research", "scripts", "linked-pin.py"))
    # 双机分片的驱动脚本读的登记表：驱动自检在临时仓里登记的那种（harness 档包的替身用例）、真跑的那种（checker 档包）、没登记分片的（驱动开跑就拒）
    # 门禁样本（文件头「门禁样本」）：这份文件所在仓里那几道阶段的原样拷贝（判「与登记的那一份相同」）、改过正文的 checker-tier-layer0-replay 拷贝、
    # 放了 .layer0-sample-tools 的样本根、没放标记的目录、crates 变异表那一道的样本根，与喂给 stage-selftest.sh 的四份阶段目录
    def registered_stage(stage):
        try:
            with open(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, stage), encoding="utf-8") as handle:
                return handle.read()
        except OSError:
            return f"# 这份文件所在仓里没有 {stage}（门禁样本那几格会红）\n"
    for stage in ("checker-tier-layer0-replay.sh", "checker-tier-crates-mutation-replay.sh", "doc-text.sh"):
        write(f".claude/gate.d/{stage}", registered_stage(stage))
    write("elsewhere/.claude/gate.d/checker-tier-layer0-replay.sh", registered_stage("checker-tier-layer0-replay.sh") + "cargo test --all\n")
    # checker-tier-layer0-replay 样本根的假 cargo 拿仓里 green 样本那一份原样、可执行（判「与仓里某份样本逐字节相同」）；另几份照拒：空标记目录、只放 quick-tier（攻方 2026-10-01 的造法）、
    # 冒充的假 cargo（转给 PATH 上的真 cargo）、内容相同而不可执行的。herd7 那一道的标记文件：原样、写成目录、内容改过
    def registered_fixture(relative):
        try:
            with open(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "fixtures", relative), "rb") as handle:
                return handle.read()
        except OSError:
            return f"# 这份文件所在仓里没有 fixtures/{relative}（门禁样本那几格会红）\n".encode()
    def write_bytes(relative, data, mode=0o644):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as handle:
            handle.write(data)
        os.chmod(path, mode)
    fake_cargo = registered_fixture("checker-tier-layer0-replay.sh/green/.layer0-sample-tools/cargo")
    write_bytes("sample-layer0/.layer0-sample-tools/cargo", fake_cargo, 0o755)
    os.makedirs(os.path.join(work, "sample-layer0-empty", ".layer0-sample-tools"))
    write_bytes("sample-layer0-quick-tier-only/.layer0-sample-tools/quick-tier", b"")
    write_bytes("sample-layer0-forged/.layer0-sample-tools/cargo", "#!/usr/bin/env bash\n# 样本：冒充的假 cargo，转给 PATH 上的真 cargo\nexec cargo \"$@\"\n".encode(), 0o755)
    write_bytes("sample-layer0-noexec/.layer0-sample-tools/cargo", fake_cargo, 0o644)
    write_bytes("sample-lkmm/.lkmm-static-only", registered_fixture("checker-tier-lkmm.sh/green/.lkmm-static-only"))
    os.makedirs(os.path.join(work, "sample-lkmm-directory", ".lkmm-static-only"))
    write_bytes("sample-lkmm-content/.lkmm-static-only", "样本：标记文件内容改过\n".encode())
    write("sample-layer0/expect", "exit=3\n")
    write("sample-layer0-bare/expect", "exit=3\n")
    write("sample-crates/crates/mutations.tsv", "# 样本\n")
    for gate_directory, stages in (("gd-light", {"doc-text.sh": registered_stage("doc-text.sh"), "checker-tier-layer0-replay.sh": registered_stage("checker-tier-layer0-replay.sh")}),
                                   ("gd-heavy", {"doc-text.sh": registered_stage("doc-text.sh"),
                                                 "checker-tier-crates-mutation-replay.sh": registered_stage("checker-tier-crates-mutation-replay.sh")}),
                                   ("gd-unregistered", {"brand-new-stage.sh": "#!/usr/bin/env bash\n# 样本：登记表外的阶段\n"}),
                                   ("gd-edited", {"doc-text.sh": registered_stage("doc-text.sh") + "cargo test --all\n"})):
        for stage, body in stages.items():
            write(f"{gate_directory}/.claude/gate.d/{stage}", body)
            write(f"{gate_directory}/.claude/gate.d/fixtures/{stage}/red/expect", "exit=1\n")
    write("gd-light/.claude/gate.d/code-tooling.sh", "#!/usr/bin/env bash\n# 样本：没有样本目录的阶段，stage-selftest.sh 不喂它\n")
    # --item 点名带标记的一道：gd-light 的 checker-tier-layer0-replay 样本里放一份照仓里 green 原样、可执行的假 cargo，另一份红样本的假 cargo 不可执行（仓里 sample-cargo-not-executable 那种，
    # 故意坏掉的标记）；样本仓里同样暂存，与暂存区相同就放行，不逐份核标记
    write_bytes("gd-light/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/green/.layer0-sample-tools/cargo", fake_cargo, 0o755)
    write_bytes("gd-light/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/cargo-not-executable/.layer0-sample-tools/cargo", fake_cargo, 0o644)
    # stage-selftest.sh 喂的样本与暂存区比（文件头「门禁样本」①）：index-reference 是临时建的样本仓（自检把 SAMPLE_INDEX_REPOSITORY 换到它），
    # 暂存区里是 gd-light 那两道的样本；另造几份与它不同的阶段目录：样本里多一份起 checker 档的 setup.sh（攻方 2026-10-01 的造法）、内容改过、执行位改过、
    # 少了文件，与样本仓自己的阶段目录（暂存之后才新加的一份样本）
    reference = os.path.join(work, SAMPLE_INDEX_REFERENCE_NAME)
    for stage in ("doc-text.sh", "checker-tier-layer0-replay.sh"):
        write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/{stage}/red/expect", "exit=1\n")
    write_bytes(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/green/.layer0-sample-tools/cargo", fake_cargo, 0o755)
    write_bytes(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/cargo-not-executable/.layer0-sample-tools/cargo", fake_cargo, 0o644)
    git_environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    for git_arguments in (["init", "-q"], ["add", "-A"]):
        subprocess.run(["git", "-C", reference, *git_arguments], check=True, capture_output=True, env=git_environment)
    write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/doc-text.sh", registered_stage("doc-text.sh"))
    write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/doc-text.sh/green/expect", "exit=0\n")
    for gate_directory, extra in (("gd-sample-setup", ("red/setup.sh", "#!/usr/bin/env bash\ncargo test --release -p singlefs-checker-tier --lib --tests\n")),
                                  ("gd-sample-edited", ("red/expect", "exit=0\n")), ("gd-sample-mode", None), ("gd-sample-missing", None)):
        write(f"{gate_directory}/.claude/gate.d/doc-text.sh", registered_stage("doc-text.sh"))
        if gate_directory == "gd-sample-missing":
            os.makedirs(os.path.join(work, gate_directory, ".claude", "gate.d", "fixtures", "doc-text.sh", "green"))   # 只剩一个空的样本目录
        elif extra is None or extra[0] != "red/expect":
            write(f"{gate_directory}/.claude/gate.d/fixtures/doc-text.sh/red/expect", "exit=1\n")
        if extra:
            write(f"{gate_directory}/.claude/gate.d/fixtures/doc-text.sh/{extra[0]}", extra[1])
    os.chmod(os.path.join(work, "gd-sample-mode", ".claude", "gate.d", "fixtures", "doc-text.sh", "red", "expect"), 0o755)
    # 自证登记表按真实路径认：别处树里路径尾部相同的拷贝（攻方 2026-10-01 的造法：正文起 checker 档）、仓副本里的同名拷贝、指到本仓那一份的符号链接
    write("evil-tree/research/scripts/claim-experiment.sh", "#!/usr/bin/env bash\n# 改过的自证\ncargo test --release -p singlefs-checker-tier --lib --tests\n")
    # 别处树里路径尾部相同的 stage-selftest.sh：不管 --item、--list-items，把每一道都喂
    write("evil-tree/.claude/singlefs-ai-sop/scripts/stage-selftest.sh", "#!/usr/bin/env bash\n# 样本：改过的 stage-selftest.sh，不认点名\n")
    write("repository-copy/research/scripts/admission.py", "# 样本：仓副本里的同名拷贝\n")
    os.makedirs(os.path.join(work, "linked-tree", "research", "scripts"))
    os.symlink(os.path.join(HOOK_REPOSITORY, "research", "scripts", "admission.py"), os.path.join(work, "linked-tree", "research", "scripts", "admission.py"))
    # case 的模式行（攻方扫描脚本 light-selftest-scan.sh 那一行原样）：切词把模式切成一条命令
    write("case-pattern.sh", "#!/usr/bin/env bash\nkey=\"$1\"; root=\"$2\"\ncase \"$key\" in\n  .claude/gate.d/lib/*.py) cmd=(python3 \"$root/$key\") ;;\n"
                             "  *) cmd=(bash \"$root/$key\" --selftest) ;;\nesac\necho \"${cmd[@]}\"\n")
    write(".claude/gate.d/stage-inputs.tsv",
          "# 样本：layer0-shard-run.sh 按树根里键那一行判重不重型\n"
          "crash-case:sharded-selftest\tcrates/\ttest=singlefs-harness:overwrite_in_one_instance:a_daily_case shard=across-machines\t# 驱动自检那样的替身用例\n"
          "crash-case:sharded-checker\tcrates/\ttest=singlefs-checker-tier:crash_enumeration_new_pool_file_creation_stream:the_full_case shard=across-machines\t# 真跑的那种\n"
          "crash-case:single-machine\tcrates/\ttest=singlefs-checker-tier:crash_enumeration_new_pool_file_creation_stream:the_full_case\t# 没登记分片\n")


# 自检临时建的样本仓（build_sample_workspace 造在临时工作区里，自检把 SAMPLE_INDEX_REPOSITORY 换到它）
SAMPLE_INDEX_REFERENCE_NAME = "index-reference"
SAMPLE_SELFTEST_REGISTRY = (
    "# 样本登记表（lib_heavy_tests 自证造的）\n"
    "research/scripts/sample-light.sh\tlight\t# 样本：假驱动\n"
    "research/scripts/sample-heavy.py\theavy\t# 样本：真编一个 crate\n"
    "research/scripts/sample-conditional.sh\theavy-if-set:SAMPLE_REAL_TOOLS\t# 样本：设了变量换真家伙\n"
    "research/scripts/sample-bad-weight.sh\tmedium\t# 样本：判定写坏了\n"
    "research/scripts/sample-short-reason.sh\tlight\t# 短\n"
    "research/scripts/sample-missing.sh\tlight\t# 样本：文件不在\n"
    "elsewhere/sample-outside.sh\tlight\t# 样本：不在三处目录里\n"
    "research/scripts/sample-light.sh\theavy\t# 样本：重复的一行\n"
    ".claude/gate.d/sample-stage.sh\tsamples-light-marker:.sample-tools\t# 样本：样本根放了假工具标记\n"
    ".claude/gate.d/sample-stage-wrong.sh\tlight\t# 样本：门禁阶段写成自证的判定\n"
    ".claude/gate.d/sample-stage-escape.sh\tsamples-light-marker:../outside\t# 样本：标记跳出样本根\n"
    "research/scripts/sample-samples-misplaced.sh\tsamples-light\t# 样本：样本档的判定写在研究脚本上\n")
# 样本表里该报出来的行（键）：判定认不出、理由太短、不在三处目录里、重复、文件不在
SAMPLE_SELFTEST_REGISTRY_PROBLEM_KEYS = ["research/scripts/sample-bad-weight.sh", "research/scripts/sample-short-reason.sh", "elsewhere/sample-outside.sh",
                                         "research/scripts/sample-light.sh", "research/scripts/sample-missing.sh", ".claude/gate.d/sample-stage-wrong.sh",
                                         ".claude/gate.d/sample-stage-escape.sh", "research/scripts/sample-samples-misplaced.sh"]
# 只含一行假 heavy 脚本的登记表：两份钩子自证里「登记为 heavy」的那几格拿它判（仓里登记的脚本自证都换成了假驱动，不拿真脚本当 heavy 样例）
HEAVY_ONLY_REGISTRY_NAME = "heavy-only-selftest-registry.tsv"
HEAVY_ONLY_SCRIPT = "research/scripts/sample-only-heavy.py"


def build_heavy_only_registry(root):
    """在 root 里写只含一行 heavy 的登记表与它点名的假脚本（只有一行注释，自证只判命令行、不起它）；交登记表的路径。"""
    script = os.path.join(root, HEAVY_ONLY_SCRIPT)
    os.makedirs(os.path.dirname(script), exist_ok=True)
    with open(script, "w", encoding="utf-8") as handle:
        handle.write("# 样本：登记成 heavy 的假脚本\n")
    registry = os.path.join(root, HEAVY_ONLY_REGISTRY_NAME)
    with open(registry, "w", encoding="utf-8") as handle:
        handle.write(f"{HEAVY_ONLY_SCRIPT}\theavy\t# 偏离上游：样本，当它的自证在 demo crate 上真编\n")
    return registry


@contextlib.contextmanager
def selftest_registry_swapped(registry, root):
    """只给自证用（平常跑的判法不经它，读的是这份文件旁边那份表）：这一段里登记表换成 registry、键相对 root 认，出来换回。"""
    global SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    saved = SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = registry, root
    try:
        yield
    finally:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = saved


def heavy_only_registry_cells(work, results):
    """造出来的登记表（只含一行假 heavy 脚本）：直接起、通配起、POSIX 字符类起它的 --selftest 都算提交时才跑的检查。格记进 results，交回判了几格。
    开关 selftest-heavy-rows-as-light 与 commit-checks-ignored 下三格红，pattern-command-words-literal 下第二、三格红，posix-brackets-by-python-glob 下第三格红。"""
    root = os.path.join(work, "heavy-only-registry")
    registry = build_heavy_only_registry(root)
    cases = [
        ("造出来的登记表：登记成 heavy 的 --selftest 算提交时才跑的检查", ["python3", HEAVY_ONLY_SCRIPT, "--selftest"]),
        ("造出来的登记表：通配起登记成 heavy 的自证", ["python3", HEAVY_ONLY_SCRIPT[:-1] + "?", "--selftest"]),
        ("造出来的登记表：POSIX 字符类起登记成 heavy 的自证", ["python3", HEAVY_ONLY_SCRIPT[:-1] + "[[:alpha:]]", "--selftest"]),
    ]
    with selftest_registry_swapped(registry, root):
        for label, argv in cases:
            found = classify_process(argv, root)
            results.append((label, "commit-check-selftest", found[0].kind if found else None))
    return len(cases)


def selftest_registry_cells(work, results):
    """自证登记表的读法：在 work 里写样本表与它点名的文件（sample-missing.sh 不写），把 SELFTEST_REGISTRY_PATH 换过去判几条命令，判完换回；
    再核仓里那份表（这份文件旁边）没有写坏的行、指到的文件在这份文件所在的仓里都在。格记进 results，交回判了几格。"""
    global SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    real_registry, real_root = SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT
    sample_registry = os.path.join(work, "sample-selftest-registry.tsv")
    with open(sample_registry, "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_SELFTEST_REGISTRY)
    for relative in ("research/scripts/sample-light.sh", "research/scripts/sample-heavy.py", "research/scripts/sample-conditional.sh",
                     "research/scripts/sample-bad-weight.sh", "research/scripts/sample-short-reason.sh", "elsewhere/sample-outside.sh",
                     ".claude/gate.d/sample-stage.sh", "elsewhere-tree/research/scripts/sample-light.sh"):
        os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
        with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
            handle.write("# 样本\n")
    before = len(results)
    sample_cases = [
        ("样本表：light 行单独 --selftest 放行", ["bash", "research/scripts/sample-light.sh", "--selftest"], None),
        ("样本表：heavy 行算提交时才跑的检查", ["python3", "research/scripts/sample-heavy.py", "--selftest"], "commit-check-selftest"),
        ("样本表：条件行没设变量算 light", ["bash", "research/scripts/sample-conditional.sh", "--selftest"], None),
        ("样本表：设了变量的条件行算起重活", ["env", "SAMPLE_REAL_TOOLS=1", "bash", "research/scripts/sample-conditional.sh", "--selftest"], "commit-check-selftest"),
        ("样本表：判定写坏的一行不算登记", ["bash", "research/scripts/sample-bad-weight.sh", "--selftest"], UNREGISTERED_SELFTEST_KIND),
        ("样本表：理由太短的一行不算登记", ["bash", "research/scripts/sample-short-reason.sh", "--selftest"], UNREGISTERED_SELFTEST_KIND),
        ("样本表：别的树里路径尾部相同的 light 脚本不算登记", ["bash", os.path.join(work, "elsewhere-tree", "research", "scripts", "sample-light.sh"), "--selftest"],
         UNREGISTERED_SELFTEST_KIND),
    ]
    try:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = sample_registry, work
        for label, argv, want in sample_cases:
            found = classify_process(argv, work)
            results.append((label, want, found[0].kind if found else None))
        found_keys = sorted(match.group(1) for problem in selftest_registry_problems(sample_registry, work)
                            for match in [re.match(r"^第 \d+ 行 (\S+)：", problem)] if match)
        results.append(("样本登记表里写坏的几行逐条报出来（判定、理由、目录、重复、文件不在、门禁阶段那几行的判定与标记）", sorted(SAMPLE_SELFTEST_REGISTRY_PROBLEM_KEYS), found_keys))
        sample_stage_row = read_selftest_registry(sample_registry)[0].get(".claude/gate.d/sample-stage.sh")
        results.append(("样本表：门禁阶段的样本档那一行照收", "samples-light-marker:.sample-tools", sample_stage_row.weight if sample_stage_row else None))
        SELFTEST_REGISTRY_PATH = os.path.join(work, "no-such-registry.tsv")
        found = classify_process(["bash", "research/scripts/sample-light.sh", "--selftest"], work)
        results.append(("登记表读不了：一律按表外拒", UNREGISTERED_SELFTEST_KIND, found[0].kind if found else None))
    finally:
        SELFTEST_REGISTRY_PATH, SELFTEST_REGISTRY_ROOT = real_registry, real_root
    heavy_only_registry_cells(work, results)
    results.append((f"仓里的登记表（{real_registry}）没有写坏的行、指到的文件在 {HOOK_REPOSITORY} 里都在", [],
                    selftest_registry_problems(real_registry, HOOK_REPOSITORY)))
    return len(results) - before


def selftest():
    """在临时工作区里判 cargo 命令行、直接执行的测试二进制与按名字判的脚本；返回退出码。"""
    global SAMPLE_INDEX_REPOSITORY, BASH_FOR_PATTERNS
    work = tempfile.mkdtemp(prefix="lib-heavy-tests-")
    results = []
    real_sample_index = SAMPLE_INDEX_REPOSITORY
    # 登记表那几格判的是本仓里那几份脚本：当前目录写本仓（登记表的键按解析后的真实路径认，临时工作区里同名的不算）
    repository = HOOK_REPOSITORY
    try:
        build_sample_workspace(work)
        SAMPLE_INDEX_REPOSITORY = os.path.join(work, SAMPLE_INDEX_REFERENCE_NAME)
        harness = os.path.join(work, "crates", "singlefs-harness")
        tier = os.path.join(work, "crates", "singlefs-checker-tier")
        tier_binary = "crash_enumeration_new_pool_file_creation_stream-0123456789abcdef"
        alias_to_tier = "CARGO_ALIAS_XT=test -p singlefs-checker-tier"
        driver = f"{BROAD_MUTATION_SCRIPT_DIRECTORY}/{BROAD_MUTATION_SCRIPT_NAME}"
        every_item = [word for number in range(1, 5092) for word in ("--item", str(number))]   # 真清单的条数那么多个 --item
        # (说明, 进程的 argv, 进程的 cwd, 该认出的 kind；None 是不重型)
        cargo_cases = [
            # 工作区全量
            ("cargo test --all", ["cargo", "test", "--all"], work, "full-cargo"),
            ("argv[0] 是 cargo 的全路径", ["/home/user/.rustup/toolchains/stable/bin/cargo", "test", "--workspace"], work, "full-cargo"),
            ("在工作区根裸跑", ["cargo", "test"], work, "full-cargo"),
            ("经 nice 包一层", ["/usr/bin/nice", "-n", "19", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("cargo nextest run 在工作区根裸跑", ["cargo", "nextest", "run"], work, "full-cargo"),
            ("cargo hack 在子命令前带自己的选项、test --workspace", ["cargo", "hack", "--each-feature", "test", "--workspace"], work, "full-cargo"),
            # harness 档、core、池级 checker：随时跑
            ("harness 档整包", ["cargo", "test", "-p", "singlefs-harness"], work, None),
            ("在 harness 档目录里裸跑", ["cargo", "test", "--release"], harness, None),
            ("bash -c 里进 harness 档裸跑", ["bash", "-c", "cd crates/singlefs-harness && cargo test"], work, None),
            ("harness 档带 --ignored（它的重用例随时跑）", ["cargo", "test", "-p", "singlefs-harness", "--", "--ignored"], work, None),
            ("cargo mutants 整个 harness 档", ["cargo", "mutants", "-p", "singlefs-harness"], work, None),
            ("checker 档只编不跑（--no-run）", ["cargo", "test", "-p", "singlefs-checker-tier", "--no-run"], work, None),
            ("checker 档 -- 之后的 --no-run 是测试二进制的参数，照跑", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--no-run"], work, "checker-tier-cargo"),
            ("cargo mutants 变异 core、--test-package 点名 harness 档", ["cargo", "mutants", "-p", "singlefs-core", "--test-package", "singlefs-harness"], work, None),
            ("cargo mutants 变异 core、--test-package 点名 checker 档", ["cargo", "mutants", "-p", "singlefs-core", "--test-package=singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo mutants --test-workspace=true 测整个工作区", ["cargo", "mutants", "-p", "singlefs-harness", "--test-workspace=true"], work, "full-cargo"),
            ("cargo mutants 配置里 test_workspace = true", ["cargo", "mutants", "-p", "mutated"], os.path.join(work, "mutants-sample"), "full-cargo"),
            ("cargo mutants -d 指到带 test_workspace = true 配置的树", ["cargo", "mutants", "-d", "mutants-sample", "-p", "mutated"], work, "full-cargo"),
            ("cargo mutants 配置里 test_workspace = true、命令行 --test-workspace false 盖掉", ["cargo", "mutants", "-p", "mutated", "--test-workspace", "false"], os.path.join(work, "mutants-sample"), None),
            ("cargo mutants --no-config 不读 test_workspace", ["cargo", "mutants", "--no-config", "-p", "mutated"], os.path.join(work, "mutants-sample"), None),
            ("core 的一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
            ("池级 checker 不是 checker 档", ["cargo", "test", "-p", "singlefs-checker"], work, None),
            ("池级 checker 的库单测", ["cargo", "test", "-p", "singlefs-checker", "--lib"], work, None),
            ("clippy --all-targets 不起测试", ["cargo", "clippy", "--all-targets"], work, None),
            ("cargo nextest list 不起测试", ["cargo", "nextest", "list", "--workspace"], work, None),
            ("cargo llvm-cov report 不起测试", ["cargo", "llvm-cov", "report"], work, None),
            # checker 档：按包判，挑哪个目标都算
            ("checker 档整包", ["cargo", "test", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("checker 档点名一个集成测试", ["cargo", "test", "--release", "-p", "singlefs-checker-tier", "--test", "crash_enumeration_new_pool_file_creation_stream"], work,
             "checker-tier-cargo"),
            ("checker 档 --tests", ["cargo", "test", "-p", "singlefs-checker-tier", "--tests"], work, "checker-tier-cargo"),
            ("checker 档 --lib", ["cargo", "test", "-p", "singlefs-checker-tier", "--lib"], work, "checker-tier-cargo"),
            ("checker 档 --doc", ["cargo", "test", "-p", "singlefs-checker-tier", "--doc"], work, "checker-tier-cargo"),
            ("checker 档 --bin 装置的内联单测", ["cargo", "test", "-p", "singlefs-checker-tier", "--bin", "e161_sample_device"], work, "checker-tier-cargo"),
            ("checker 档 --test 通配", ["cargo", "test", "-p", "singlefs-checker-tier", "--test", "*"], work, "checker-tier-cargo"),
            ("checker 档带 --include-ignored --exact", ["cargo", "test", "-p", "singlefs-checker-tier", "--test", "crash_enumeration_new_pool_file_creation_stream", "--",
                                                     "--include-ignored", "--exact", "the_full_case"], work, "checker-tier-cargo"),
            ("在 checker 档目录里裸跑", ["cargo", "test", "--release"], tier, "checker-tier-cargo"),
            ("-p core 加 -p checker 档", ["cargo", "test", "-p", "singlefs-core", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("--manifest-path 指 checker 档", ["cargo", "test", "--manifest-path", "crates/singlefs-checker-tier/Cargo.toml"], work, "checker-tier-cargo"),
            ("cargo nextest run -p checker 档", ["cargo", "nextest", "run", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo miri test -p checker 档", ["cargo", "miri", "test", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo llvm-cov 不带子命令 -p checker 档", ["cargo", "llvm-cov", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("cargo mutants -p checker 档", ["cargo", "mutants", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            # -p 的几种包规格（开关 package-specification-literal 下这三格红）
            ("-p 通配命中 checker 档", ["cargo", "test", "-p", "singlefs-checker-t*"], work, "checker-tier-cargo"),
            ("-p name@version", ["cargo", "test", "-p", "singlefs-checker-tier@0.1.0"], work, "checker-tier-cargo"),
            ("-p pkgid URL", ["cargo", "test", "-p", "path+file:///x/crates/singlefs-checker-tier#singlefs-checker-tier@0.1.0"], work, "checker-tier-cargo"),
            # 别名：看得到值就展开再判（开关 alias-not-expanded 下前三格红；configuration-before-subcommand-only 下第四格红）
            ("--config 别名展开成 test -p checker 档", ["cargo", "--config", 'alias.xt="test -p singlefs-checker-tier"', "xt"], harness, "checker-tier-cargo"),
            ("--config 别名写成数组", ["cargo", "--config", 'alias.xt=["test", "-p", "singlefs-checker-tier"]', "xt"], work, "checker-tier-cargo"),
            ("环境变量 CARGO_ALIAS_XT 定的别名", ["env", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("--config 写在子命令之后定的别名", ["cargo", "xt", "--config", 'alias.xt="test -p singlefs-checker-tier"'], work, "checker-tier-cargo"),
            ("别名展开成 harness 档", ["env", "CARGO_ALIAS_XT=test -p singlefs-harness", "cargo", "xt"], work, None),
            ("--config 定了别名、子命令不是它（cargo build）", ["cargo", "--config", 'alias.xt="test -p singlefs-checker-tier"', "build"], work, None),
            ("别名的值读不出（TOML 写坏了）：按 checker 档算", ["cargo", "--config", "alias.xt=[unclosed", "xt"], work, "checker-tier-cargo"),
            # --list：只列不跑（开关 list-by-presence 下前三格红；list-past-separator 下后两格红）
            ("checker 档 -- --include-ignored --skip --list（--list 是 --skip 的值）",
             ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--include-ignored", "--skip", "--list"], work, "checker-tier-cargo"),
            ("checker 档 -- --logfile --list（--list 是日志文件名）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--logfile", "--list"], work,
             "checker-tier-cargo"),
            ("checker 档 -- --ignored --skip=--list（一个词，没有 --list 选项）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--ignored", "--skip=--list"],
             work, "checker-tier-cargo"),
            ("checker 档 -- --list（只列）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--list"], work, None),
            ("checker 档 -- --ignored --test-threads 4 --list（只列）", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--ignored", "--test-threads", "4",
                                                                  "--list"], work, None),
            ("checker 档 -- --include-ignored the_full_case -- --list（第二个 -- 之后 --list 是过滤词）",
             ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--include-ignored", "the_full_case", "--", "--list"], work, "checker-tier-cargo"),
            ("checker 档 -- --exact the_full_case -- --list", ["cargo", "test", "-p", "singlefs-checker-tier", "--", "--exact", "the_full_case", "--", "--list"],
             work, "checker-tier-cargo"),
            # 按名字认的脚本与门禁阶段
            ("bash 起 checker-tier-layer0-replay", ["bash", ".claude/gate.d/checker-tier-layer0-replay.sh", "--full", "/tmp/wt"], work, "layer0-stage"),
            ("bash 起双机分片的驱动脚本", ["bash", "research/scripts/layer0-shard-run.sh", "crash-case:sample", "/tmp/wt"], work, "checker-tier-cargo"),
            ("双机分片的驱动脚本单独 --selftest：登记成没设 SINGLEFS_HEAVY_TESTS 时用假 cargo，放行", ["bash", "research/scripts/layer0-shard-run.sh", "--selftest"], repository, None),
            # 驱动按树根里键那一行判（shard_driver_runs_checker_tier）：驱动自检在临时仓里起的那几趟（登记的是 harness 档包）不重型，真跑（checker 档包、带 shard=across-machines）照算
            ("驱动自检那样：树根里登记的分片用例是 harness 档包", ["bash", os.path.join(work, "research", "scripts", "layer0-shard-run.sh"), "crash-case:sharded-selftest", work],
             "/", None),
            ("驱动自检那样 --merged-log，树根相对当前目录写", ["bash", "research/scripts/layer0-shard-run.sh", "--merged-log", "crash-case:sharded-selftest", ".", "0123", "/tmp/m.log"],
             work, None),
            ("真跑没带前缀：树根里登记的分片用例是 checker 档包", ["bash", "research/scripts/layer0-shard-run.sh", "crash-case:sharded-checker", work], work, "checker-tier-cargo"),
            ("真跑 --merged-log（checker-tier-layer0-replay 调的那一条）", ["bash", "research/scripts/layer0-shard-run.sh", "--merged-log", "crash-case:sharded-checker", work, "0123", "/tmp/m.log"],
             work, "checker-tier-cargo"),
            ("树根里这条没登记分片（驱动开跑就拒）", ["bash", "research/scripts/layer0-shard-run.sh", "crash-case:single-machine", work], work, None),
            ("树根里没有这一行（驱动开跑就拒）", ["bash", "research/scripts/layer0-shard-run.sh", "crash-case:nowhere", work], work, None),
            ("键是没展开的变量照算", ["bash", "research/scripts/layer0-shard-run.sh", "$case_key", work], work, "checker-tier-cargo"),
            ("bash 起 checker-tier-crates-mutation-replay 的双机分片驱动", ["bash", "research/scripts/mutation-shard-run.sh", work], work, "crates-mutation-stage"),
            ("checker-tier-crates-mutation-replay 的双机分片驱动经 nice 与绝对路径起", ["nice", "-n", "19", "bash", os.path.join(work, "research", "scripts", "mutation-shard-run.sh"), "/tmp/wt"], "/",
             "crates-mutation-stage"),
            ("checker-tier-crates-mutation-replay 的双机分片驱动单独 --selftest：登记成假 cargo，放行", ["bash", "research/scripts/mutation-shard-run.sh", "--selftest"], repository, None),
            ("cargo run E152", ["cargo", "run", "--release", "--bin", "e152-file-system-benchmark"], os.path.join(work, "research"), "e152"),
            # 广谱变异的逐条驱动按名字与参数认（开关 broad-mutation-ignored 下算的各格红）
            ("广谱变异整趟跑", ["python3", driver], work, "broad-mutation"),
            ("广谱变异经 nice 与绝对路径、按分片跑", ["nice", "-n", "19", "python3", os.path.join(work, "research", "scripts", "broad-mutation.py"), "--shard", "0/2"], "/",
             "broad-mutation"),
            ("广谱变异点名但两段都跑", ["python3", driver, "--item", "12"], work, "broad-mutation"),
            ("广谱变异只补第二段", ["python3", driver, "--stage-two-only"], work, "broad-mutation"),
            ("广谱变异两条命令分号相连、前一条只看汇总", ["bash", "-c", f"python3 {driver} --summary; python3 {driver}"], work, "broad-mutation"),
            # 凡要起 cargo 或改库目录的都算（开关 broad-mutation-first-round-exemptions 下这八格红）
            ("广谱变异点名一条、只跑第一段：照样起 cargo，算", ["python3", driver, "--item", "12", "--skip-stage-two"], work, "broad-mutation"),
            ("广谱变异点名 5091 条、只跑第一段：就是整趟第一段", ["python3", driver, *every_item, "--skip-stage-two"], work, "broad-mutation"),
            ("广谱变异点名全清单、自己带前缀", ["env", "SINGLEFS_HEAVY_TESTS=user-request", "nice", "-n", "19", "python3", driver, *every_item, "--skip-stage-two"],
             work, "broad-mutation"),
            ("广谱变异点名只跑第一段、自己放大线程与内存", ["python3", driver, "--item", "1", "--skip-stage-two", "--threads", "64", "--memory-cap", "200G"],
             work, "broad-mutation"),
            ("广谱变异点名只跑第一段、带只供自证的假一步变量", ["env", "BROAD_MUTATION_STEP_DRIVER=/bin/true", "python3", driver, "--item", "7", "--skip-stage-two"],
             work, "broad-mutation"),
            ("广谱变异点名只跑第一段、带弄坏开关", ["env", "BROAD_MUTATION_BREAK=interrupted-as-done", "python3", driver, "--item", "7", "--skip-stage-two"],
             work, "broad-mutation"),
            ("广谱变异 --sync-tree 换全局的版本指针，算", ["python3", driver, "--sync-tree"], work, "broad-mutation"),
            ("广谱变异 --sync-tree 换到别的提交", ["python3", driver, "--sync-tree", "--commit", "HEAD~1"], work, "broad-mutation"),
            ("广谱变异 --import-results 往本机结果表里导入，算", ["python3", driver, "--import-results", "/tmp/claude-1000/peer"], work, "broad-mutation"),
            # 只读的不算；仓内那一份的 --selftest 照自证登记表判，改名拷贝的 --selftest 照旧算提交时才跑的检查
            ("广谱变异 --list-items 只列项", ["python3", driver, "--list-items"], work, None),
            ("广谱变异 --summary 只看汇总", ["python3", driver, "--summary"], work, None),
            ("广谱变异 --summary 带 --shard（驱动走汇总那一支就返回）", ["python3", driver, "--summary", "--shard", "0/2"], work, None),
            ("广谱变异 --list-items 带 --from", ["python3", driver, "--list-items", "--from", "1"], work, None),
            ("广谱变异 -h 带 --shard", ["python3", driver, "-h", "--shard", "0/2"], work, None),
            # 开关 broad-mutation-read-only-counted 下这一格红
            ("广谱变异 --check-ignored 只判忽略用例登记", ["python3", driver, "--check-ignored"], work, None),
            ("广谱变异单独 --selftest：登记成假一步驱动，放行（不落到按驱动判的 --selftest 那一支）", ["python3", driver, "--selftest"], repository, None),
            ("登记成轻的脚本带别的参数再带 --selftest 照拒：广谱变异 --selftest 混运行参数", ["python3", driver, "--selftest", "--shard", "0/2"], repository,
             UNREGISTERED_SELFTEST_KIND),
            ("广谱变异的改名拷贝 --selftest 不在登记表里：照旧算提交时才跑的检查", ["python3", "research/scripts/renamed-driver-copy.py", "--selftest"], work,
             UNREGISTERED_SELFTEST_KIND),
            ("广谱变异仓外的改名拷贝 --selftest：按驱动判的那一支，照旧算提交时才跑的检查", [os.path.join(work, "elsewhere", "renamed-driver-copy.py"), "--selftest"], "/",
             "commit-check-selftest"),
            ("仓外同名的 broad-mutation.py（不在盘上）不按名字判", ["python3", "/tmp/claude-1000/elsewhere/broad-mutation.py"], work, None),
            # argparse 认的缩写与贴着写：分类只认整词，照算（接受的误拒）
            ("广谱变异 --item=1 贴着写、只跑第一段", ["python3", driver, "--item=1", "--skip-stage-two"], work, "broad-mutation"),
            ("广谱变异缩写 --sum（驱动当 --summary 办，分类照算）", ["python3", driver, "--sum"], work, "broad-mutation"),
            # python 带值的解释器选项照语义切（lib_shell_words.py 的开关 LIB_SHELL_WORDS_BREAK=python-options-by-dash 下前八格红；
            # 这个开关下全部红的二十三格逐格写在文件头末尾那一段）
            ("python3 -W ignore 起广谱变异", ["python3", "-W", "ignore", driver, "--shard", "0/2"], work, "broad-mutation"),
            ("python3 -X dev 起广谱变异", ["python3", "-X", "dev", driver], work, "broad-mutation"),
            ("python3 -BW default（合写、值另起一个词）起广谱变异", ["python3", "-BW", "default", driver], work, "broad-mutation"),
            ("python3 --check-hash-based-pycs never 起广谱变异", ["python3", "--check-hash-based-pycs", "never", driver], work, "broad-mutation"),
            ("cd 进 research/scripts 再 python3 -m broad-mutation", ["bash", "-c", "cd research/scripts && python3 -m broad-mutation"], work, "broad-mutation"),
            ("python3 -m research.scripts.broad-mutation（命名空间包）", ["python3", "-m", "research.scripts.broad-mutation", "--shard", "0/2"], work, "broad-mutation"),
            ("python3 -m runpy broad-mutation", ["python3", "-m", "runpy", "broad-mutation"], os.path.join(work, "research", "scripts"), "broad-mutation"),
            ("PYTHONPATH 指到 research/scripts 再 python3 -m broad-mutation", ["env", "PYTHONPATH=research/scripts", "python3", "-m", "broad-mutation"], work,
             "broad-mutation"),
            ("python3 -W ignore 起研究脚本的自证（登记成假 cargo，放行）", ["python3", "-W", "ignore", "research/scripts/admission.py", "--selftest"], repository, None),
            ("python3 -m broad-mutation 带 --summary", ["python3", "-m", "broad-mutation", "--summary"], os.path.join(work, "research", "scripts"), None),
            ("python3 -u（不带值的选项）起广谱变异", ["python3", "-u", driver], work, "broad-mutation"),
            ("python3 -Wignore（值贴着写）起广谱变异", ["python3", "-Wignore", driver], work, "broad-mutation"),
            ("python3 -m py_compile 只查驱动的语法", ["python3", "-m", "py_compile", driver], work, None),
            ("python3 -W ignore 起别的研究脚本照常用", ["python3", "-W", "ignore", "research/scripts/replace-once.py", "a.md", "x", "y"], work, None),
            # python -c 的代码里点到驱动、指到它的符号链接、带签名行的改名拷贝（开关 broad-mutation-by-name-only 下前七格红）
            ("python3 -c runpy.run_path 起广谱变异", ["python3", "-c", f"import runpy; runpy.run_path('{driver}', run_name='__main__')", "--shard", "0/2"], work,
             "broad-mutation"),
            ("python3 -c exec(open(驱动).read())", ["python3", "-c", f"__file__='{driver}'; exec(open(__file__).read())"], work, "broad-mutation"),
            ("python3 -c runpy.run_module 按模块名起", ["python3", "-c", "import runpy; runpy.run_module('broad-mutation', run_name='__main__')"],
             os.path.join(work, "research", "scripts"), "broad-mutation"),
            ("仓内改名的符号链接指到驱动", ["python3", "research/scripts/linked-driver.py"], work, "broad-mutation"),
            ("仓外的符号链接指到驱动、按分片跑", ["python3", os.path.join(work, "elsewhere", "linked-driver.py"), "--shard", "0/2"], "/", "broad-mutation"),
            ("仓内改名拷贝（开头带签名行）", ["python3", "research/scripts/renamed-driver-copy.py"], work, "broad-mutation"),
            ("仓外改名拷贝直接执行", [os.path.join(work, "elsewhere", "renamed-driver-copy.py"), "--item", "3", "--skip-stage-two"], "/", "broad-mutation"),
            ("python3 -c 的代码带 --summary", ["python3", "-c", f"import runpy; runpy.run_path('{driver}', run_name='__main__')", "--summary"], work, None),
            ("符号链接带 --list-items", ["python3", "research/scripts/linked-driver.py", "--list-items"], work, None),
            ("python3 -c 不相干的代码", ["python3", "-c", "print(1)"], work, None),
            ("签名行不在行首的不相干脚本", ["python3", os.path.join(work, "elsewhere", "unrelated.py")], work, None),
            ("删掉签名行的仓外拷贝认不出（罩不住的那一种，钉住现状）", ["python3", os.path.join(work, "elsewhere", "copy-without-signature.py")], work, None),
            ("grep 读驱动", ["grep", "-n", "main", driver], work, None),
            ("sha256sum 驱动", ["sha256sum", driver], work, None),
            # 门禁阶段按文件名认（开关 stage-by-number 下去了编号的那四格红）；不重型的阶段是提交时才跑的检查（开关 commit-checks-ignored 下这一类各格红）
            ("bash 起轻阶段 doc-text：提交时才跑的检查", ["bash", ".claude/gate.d/doc-text.sh"], work, "commit-check-stage"),
            ("去了编号的轻阶段 --check 一格：提交时才跑的检查", ["bash", ".claude/gate.d/code-tooling.sh", "--check", "agent-write-scope"], work, "commit-check-stage"),
            ("QEMU 那一道（去了编号）", ["bash", ".claude/gate.d/checker-tier-qemu-device-streams.sh"], work, "qemu-stage"),
            ("herd7 那一道（去了编号）经 nice 与绝对路径起", ["nice", "-n", "19", "bash", os.path.join(work, ".claude", "gate.d", "checker-tier-lkmm.sh")], "/", "herd7-stage"),
            ("crates 变异表那一道（去了编号）按变异名点名", ["bash", ".claude/gate.d/checker-tier-crates-mutation-replay.sh", "--item", "加改成减"], work,
             "crates-mutation-stage"),
            ("research 构建与复跑那一道整道跑", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh"], work, "replay-all-stage"),
            ("Verus 那一道（形式化验证）只跑一格", ["bash", ".claude/gate.d/checker-tier-formal-verification.sh", "--check", "verus-proof"], work, "verus-stage"),
            ("Verus 那一道 --list 只打格名表", ["bash", ".claude/gate.d/checker-tier-formal-verification.sh", "--list"], work, None),
            # 广谱变异的提交门禁按文件名归「广谱变异」（开关 stage-by-number 下前两格红：按编号认不出、判成不算；tool-versions-counted 下末一格红）
            ("广谱变异的提交门禁整道跑", ["bash", ".claude/gate.d/checker-tier-broad-mutation.sh"], work, "broad-mutation-stage"),
            ("广谱变异的提交门禁只跑库目录那一格与判定那一格", ["bash", ".claude/gate.d/checker-tier-broad-mutation.sh", "--check",
                                                    "library-version-is-staged-tree,gate-verdict"], work, "broad-mutation-stage"),
            # 只跑判定那一格只读库目录、起驱动的 --summary，不算（开关 broad-mutation-read-only-counted 下这一格红）
            ("广谱变异的提交门禁经 nice 只跑判定那一格", ["nice", "-n", "19", "bash", ".claude/gate.d/checker-tier-broad-mutation.sh", "--check", "gate-verdict"], work,
             None),
            ("广谱变异的提交门禁 --list 只打格名表", ["bash", ".claude/gate.d/checker-tier-broad-mutation.sh", "--list"], work, None),
            ("广谱变异的提交门禁 --tool-versions 只打工具版本", ["bash", ".claude/gate.d/checker-tier-broad-mutation.sh", "--tool-versions"], work, None),
            ("直接起 verus 验一个 crate 不算重型", ["verus", "--crate-type=lib", "--no-cheating", "src/lib.rs"], work, None),
            ("cargo verus verify 不算重型", ["cargo", "verus", "verify", "-p", "singlefs-core"], work, None),
            # Verus 那一道只打工具版本不算（开关 tool-versions-counted 下这一格红）
            ("Verus 那一道 --tool-versions 只打工具版本", ["bash", ".claude/gate.d/checker-tier-formal-verification.sh", "--tool-versions"], work, None),
            # 两份判法脚本直接起：干活的用法算那一道（开关 verus-judges-by-stage-only 下前五格红），只读的不算
            ("直接起判法脚本跑必红世界", ["python3", "research/scripts/verus-proof-check.py", "must-red-worlds", "."], work, "verus-stage"),
            ("直接起判法脚本跑空真、经 nice 与绝对路径", ["nice", "-n", "19", "python3", os.path.join(work, "research", "scripts", "verus-proof-check.py"), "vacuity", work], "/",
             "verus-stage"),
            ("直接起判法脚本验证明", ["python3", "research/scripts/verus-proof-check.py", "proof", "."], work, "verus-stage"),
            ("直接起红例重放", ["python3", "research/scripts/e165-pin-red-examples.py", "--replay", "--root", "."], work, "verus-stage"),
            ("经内存包装起红例重放、--check 写在 --replay 前面", ["bash", "research/scripts/run-with-memory-cap.sh", "4G", "python3", "research/scripts/e165-pin-red-examples.py",
                                                            "--check", "--replay"], work, "verus-stage"),
            ("判法脚本只打工具版本", ["python3", "research/scripts/verus-proof-check.py", "tool-versions", "."], work, None),
            ("判法脚本核版本一致（只读安装与缓存）", ["python3", "research/scripts/verus-proof-check.py", "toolchain", "."], work, None),
            ("判法脚本打包源码指纹", ["python3", "research/scripts/verus-proof-check.py", "fingerprint", "/tmp/claude-1000/vstd"], work, None),
            ("红例钉表只核钉全", ["python3", "research/scripts/e165-pin-red-examples.py", "--check"], work, None),
            ("红例钉表 --replay 写在 --check 前面、以 --check 为准", ["python3", "research/scripts/e165-pin-red-examples.py", "--replay", "--check"], work, None),
            ("红例钉表 --root 的值碰巧叫 --replay", ["python3", "research/scripts/e165-pin-red-examples.py", "--check", "--root", "--replay"], work, None),
            ("红例钉表追加", ["python3", "research/scripts/e165-pin-red-examples.py", "research/results/e165-sample-red-examples.tsv"], work, None),
            ("判法脚本单独 --selftest：登记成假 Verus，放行", ["python3", "research/scripts/verus-proof-check.py", "--selftest"], repository, None),
            ("仓外同名的判法脚本不按名字认", ["python3", "/tmp/claude-1000/elsewhere/verus-proof-check.py", "must-red-worlds", "."], work, None),
            # 认出是两份判法脚本就算、只放过字面写明的只读用法，认法照广谱变异那一套（开关 verus-judges-by-working-subcommand、verus-judges-by-stage-only
            # 下前十二格红，末两格照绿）
            ("PYTHONPATH 指到 research/scripts 再 python3 -m 起判法脚本跑必红世界",
             ["env", "PYTHONPATH=research/scripts", "python3", "-m", "verus-proof-check", "must-red-worlds", "."], work, "verus-stage"),
            ("export PYTHONPATH 再 python3 -m 起红例重放", ["bash", "-c", "export PYTHONPATH=research/scripts; python3 -m e165-pin-red-examples --replay"], work, "verus-stage"),
            ("python3 -c runpy.run_path 起判法脚本跑必红世界",
             ["python3", "-c", "import runpy; runpy.run_path('research/scripts/verus-proof-check.py', run_name='__main__')", "must-red-worlds", "."], work, "verus-stage"),
            ("仓外的符号链接指到判法脚本、跑必红世界", ["python3", os.path.join(work, "elsewhere", "linked-proof-check.py"), "must-red-worlds", work], "/", "verus-stage"),
            ("仓内改名的符号链接指到红例钉表、--replay", ["python3", "research/scripts/linked-pin.py", "--replay"], work, "verus-stage"),
            ("判法脚本的子命令用命令替换写", ["bash", "-c", 'python3 research/scripts/verus-proof-check.py "$(echo must-red-worlds)" .'], work, "verus-stage"),
            ("for 循环逐格跑判法脚本三格", ["bash", "-c", "for c in proof vacuity must-red-worlds; do python3 research/scripts/verus-proof-check.py $c .; done"], work,
             "verus-stage"),
            ("shell 函数转参数起判法脚本", ["bash", "-c", 'v() { python3 research/scripts/verus-proof-check.py "$@"; }; v must-red-worlds .'], work, "verus-stage"),
            ("for 循环先核钉全再重放红例", ["bash", "-c", "for m in --check --replay; do python3 research/scripts/e165-pin-red-examples.py $m; done"], work, "verus-stage"),
            ("只读子命令带没展开的变量照算（接受的误拒）", ["bash", "-c", "python3 research/scripts/verus-proof-check.py tool-versions $ROOT"], work, "verus-stage"),
            ("判法脚本不带参数照算（接受的误拒）", ["python3", "research/scripts/verus-proof-check.py"], work, "verus-stage"),
            ("经 PYTHONPATH 起判法脚本的 --selftest：不在登记表的位置上，照旧算提交时才跑的检查",
             ["env", "PYTHONPATH=research/scripts", "python3", "-m", "verus-proof-check", "--selftest"], work, "commit-check-selftest"),
            ("仓外的符号链接指到判法脚本、只打工具版本", ["python3", os.path.join(work, "elsewhere", "linked-proof-check.py"), "tool-versions", work], "/", None),
            ("PYTHONPATH 再 python3 -m 起判法脚本核版本一致", ["env", "PYTHONPATH=research/scripts", "python3", "-m", "verus-proof-check", "toolchain", "."], work, None),
            # 判参数之前照两份脚本开头的 preflight 摘掉每一个整词 --force（开关 verus-judges-force-kept 下这三格红）
            ("红例钉表 --target 的值位写 --force、后面写 --check：preflight 摘掉 --force，真跑的是 --replay",
             ["python3", "research/scripts/e165-pin-red-examples.py", "--replay", "--target", "--force", "--check"], work, "verus-stage"),
            ("红例钉表带 --root、--target 的值位写 --force、后面写 --check",
             ["python3", "research/scripts/e165-pin-red-examples.py", "--root", ".", "--replay", "--target", "--force", "--check"], work, "verus-stage"),
            ("判法脚本首参前写 --force、只打工具版本：preflight 摘掉之后是 tool-versions",
             ["python3", "research/scripts/verus-proof-check.py", "--force", "tool-versions", "."], work, None),
            # python -c 的代码里出现两份判法脚本的名字就算那一道、不看代码之后的词（开关 verus-judges-code-by-later-words、verus-judges-by-working-subcommand、
            # verus-judges-by-stage-only 下前八格红；verus-judges-code-print-unchecked 下第七格红；末格照绿）
            ("python3 -c 里改 sys.argv 成 proof、代码之后写 tool-versions",
             ["python3", "-c", "import runpy, sys; sys.argv = ['v', 'proof', '.']; runpy.run_path('research/scripts/verus-proof-check.py', run_name='__main__')",
              "tool-versions"], work, "verus-stage"),
            ("python3 -c 里改 sys.argv 成 --replay、代码之后不写词",
             ["python3", "-c", "import runpy, sys; sys.argv = ['p', '--replay']; runpy.run_path('research/scripts/e165-pin-red-examples.py', run_name='__main__')"],
             work, "verus-stage"),
            ("python3 -c 里 __import__ 按模块名起判法脚本验证明",
             ["python3", "-c", "import sys; sys.path.insert(0, 'research/scripts'); sys.exit(__import__('verus-proof-check').main(['proof', '.']))"], work, "verus-stage"),
            ("python3 -c 里 importlib.import_module 起红例重放",
             ["python3", "-c", "import importlib, sys; sys.path.insert(0, 'research/scripts'); sys.exit(importlib.import_module('e165-pin-red-examples').main(['--replay']))"],
             work, "verus-stage"),
            ("python3 -c 里 spec_from_file_location 载入判法脚本再调 run_check、代码之后写 toolchain",
             ["python3", "-c", "import importlib.util as u; s = u.spec_from_file_location('v', 'research/scripts/verus-proof-check.py'); m = u.module_from_spec(s); "
              "s.loader.exec_module(m); m.run_check('proof', '.', m.parse_options([])[1])", "toolchain"], work, "verus-stage"),
            ("python3 -c 的 print 包着 __import__ 起判法脚本",
             ["python3", "-c", "import sys; sys.path.insert(0, 'research/scripts')\nprint(__import__('verus-proof-check').main(['proof', '.']))"], work, "verus-stage"),
            ("python3 -c 的 print 包着 __import__、代码只有这一句",
             ["python3", "-c", "print(__import__('verus-proof-check').main(['proof', '.']))"], work, "verus-stage"),
            ("python3 -c runpy.run_path 起红例钉表、代码之后写 --check",
             ["python3", "-c", "import runpy; runpy.run_path('research/scripts/e165-pin-red-examples.py', run_name='__main__')", "--check"], work, "verus-stage"),
            ("python3 -c 只 print 判法脚本的路径（字面字符串）：不算",
             ["python3", "-c", "print('research/scripts/e165-pin-red-examples.py', 'research/scripts/verus-proof-check.py')"], work, None),
            # lkmm.sh 只打 herd7 版本不算（开关 herd7-version-counted 下第一格红）；别的用法照旧算 herd7
            ("lkmm.sh 只打 herd7 版本", ["bash", ".claude/scripts/lkmm.sh", "--herd7-version"], work, None),
            ("lkmm.sh 整套判定", ["bash", ".claude/scripts/lkmm.sh"], work, "lkmm"),
            ("lkmm.sh --herd7-version 后面还带别的参数照算", ["bash", ".claude/scripts/lkmm.sh", "--herd7-version", "--force"], work, "lkmm"),
            ("research 构建与复跑那一道只跑复跑那一格、按实验点名", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh", "--check", "experiment-replay",
                                                            "--item", "E142"], work, "replay-all-stage"),
            ("research 构建与复跑那一道两格一起点名", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh", "--check",
                                                  "research-unit-tests,experiment-replay"], work, "replay-all-stage"),
            # 只跑单测那一格不是全部实验复跑（开关 replay-cells-ignored 下这两格红）
            ("research 构建与复跑那一道只跑单测那一格：提交时才跑的检查", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh", "--check",
                                                                "research-unit-tests"], work, "commit-check-stage"),
            ("research 构建与复跑那一道 --check=research-unit-tests", ["bash", ".claude/gate.d/checker-tier-research-build-and-replay.sh",
                                                                  "--check=research-unit-tests", "/tmp/wt"], work, "commit-check-stage"),
            # 只查清单、只生成的不算
            ("重型那一道 --list 只打格名表", ["bash", ".claude/gate.d/checker-tier-crates-mutation-replay.sh", "--list"], work, None),
            ("checker-tier-layer0-replay --list-items 只列崩溃枚举用例", ["bash", ".claude/gate.d/checker-tier-layer0-replay.sh", "--list-items"], work, None),
            ("轻阶段 --list", ["bash", ".claude/gate.d/code-tooling.sh", "--list"], work, None),
            ("轻阶段 --write 是生成", ["bash", ".claude/gate.d/doc-decisions.sh", "--write"], work, None),
            # 上游规范副本里的几道 lint 与样本自检、阶段结构判法（开关 commit-checks-ignored 下这一类各格红）
            ("doc-lint.sh", ["bash", ".claude/singlefs-ai-sop/scripts/doc-lint.sh", "."], work, "commit-check-lint"),
            ("gate-lint.sh 经 env 设变量", ["env", "GATE_LINT_DIR=research/scripts", "bash", ".claude/singlefs-ai-sop/scripts/gate-lint.sh"], work, "commit-check-lint"),
            ("shell-lint.sh", ["bash", ".claude/singlefs-ai-sop/scripts/shell-lint.sh"], work, "commit-check-lint"),
            ("preflight-lint.py", ["python3", ".claude/singlefs-ai-sop/scripts/preflight-lint.py", "."], work, "commit-check-lint"),
            ("gate-overlap.py 判", ["python3", ".claude/singlefs-ai-sop/scripts/gate-overlap.py", "."], work, "commit-check-lint"),
            ("gate-overlap.py --list 查清单", ["python3", ".claude/singlefs-ai-sop/scripts/gate-overlap.py", "--list"], work, None),
            ("rules-lint.sh", ["bash", ".claude/singlefs-ai-sop/scripts/rules-lint.sh", "."], work, "commit-check-lint"),
            ("hooks-registered.sh", ["bash", ".claude/singlefs-ai-sop/scripts/hooks-registered.sh", "."], work, "commit-check-lint"),
            ("stage-selftest.sh 喂样本", ["nice", "-n", "19", "bash", ".claude/singlefs-ai-sop/scripts/stage-selftest.sh", "/tmp/d/.claude/gate.d"], work,
             "commit-check-lint"),
            ("规范副本的 gate.sh 是整轮门禁", ["bash", ".claude/singlefs-ai-sop/scripts/gate.sh"], work, "gate-sh"),
            ("仓外同名的 doc-lint.sh 不算", ["bash", "/tmp/claude-1000/elsewhere/doc-lint.sh"], work, None),
            # 自证照登记表 selftest-registry.tsv 判（这几格读仓里那份表）：登记成不起重活的研究脚本单独 --selftest 放行
            # （开关 selftest-registry-ignored 下这一组 None 的格红，commit-checks-ignored 下这一组拒的格红）
            ("登记成不起重活的研究脚本单独 --selftest 放行：admission.py", ["python3", "research/scripts/admission.py", "--selftest"], repository, None),
            ("登记成不起重活的研究脚本单独 --selftest 放行：crash-amplification.sh（2026-10-01 被拒的那一条，假分项驱动）",
             ["bash", "research/scripts/crash-amplification.sh", "--selftest"], repository, None),
            ("登记成不起重活的研究脚本单独 --selftest 放行：从 research 里相对着写的 stage-must-run.sh", ["bash", "scripts/stage-must-run.sh", "--selftest"],
             os.path.join(repository, "research"), None),
            ("登记成不起重活的钩子单独 --selftest 放行", ["bash", ".claude/hooks/heavy-test-guard.sh", "--selftest"], repository, None),
            ("登记成不起重活的门禁共用库直接执行（就是跑它的自证）放行", ["bash", ".claude/gate.d/lib/stage-cells.sh"], repository, None),
            ("登记成不起重活的门禁共用库单独带 --selftest 放行", ["bash", ".claude/gate.d/lib/stage-cells.sh", "--selftest"], repository, None),
            ("登记成不起重活的 gate-staged.sh --selftest 放行（假 gate.sh）", ["bash", "research/scripts/gate-staged.sh", "--selftest"], repository, None),
            ("登记成不起重活的 run-with-memory-cap.sh --selftest 放行（包装认不出上限，当它自己跑）", ["bash", "research/scripts/run-with-memory-cap.sh", "--selftest"],
             repository, None),
            ("登记成不起重活的 gate-structure-check.py --selftest 放行（替身 cargo），不落到阶段结构判法那一格", ["python3", "research/scripts/gate-structure-check.py", "--selftest"],
             repository, None),
            # 自证换成假驱动、登记成不起重活的两份单独 --selftest 放行（开关 selftest-registry-ignored 下这两格红，commit-checks-ignored 下第二格红：
            # 不读登记表时双机驱动落到崩溃放量按名字判）；登记成起重活的那一种拿造出来的登记表判（heavy_only_registry_cells）
            ("登记成不起重活的 --selftest 放行：compile-then-swap.py（每项一份临时仓、PATH 最前面放假 cargo）", ["python3", "research/scripts/compile-then-swap.py", "--selftest"],
             repository, None),
            ("登记成不起重活的 --selftest 放行：crash-amplification-two-hosts.sh（本机、第二台、清场与用例全是假的）",
             ["bash", "research/scripts/crash-amplification-two-hosts.sh", "--selftest"], repository, None),
            # 条件行：命令里把变量设成非空时自证换真 cargo（开关 selftest-condition-ignored 与 selftest-heavy-rows-as-light 下第一格红）
            ("设了变量的条件行算起重活：带 SINGLEFS_HEAVY_TESTS 的 layer0-shard-run.sh --selftest 换真 cargo 跑 checker 档包",
             ["env", "SINGLEFS_HEAVY_TESTS=commit", "bash", "research/scripts/layer0-shard-run.sh", "--selftest"], repository, "commit-check-selftest"),
            ("条件行的变量设成空串算没设", ["env", "SINGLEFS_HEAVY_TESTS=", "bash", "research/scripts/layer0-shard-run.sh", "--selftest"], repository, None),
            # 登记表外的 --selftest 照拒（开关 selftest-unregistered-as-light 下这两格红）
            ("登记表外的 --selftest 照拒：research/scripts 下新写的脚本", ["python3", "research/scripts/brand-new-tool.py", "--selftest"], work, UNREGISTERED_SELFTEST_KIND),
            ("登记表外的 --selftest 照拒：钩子目录下没有自证入口、也没登记的共用模块", ["python3", ".claude/hooks/lib_shell_words.py", "--selftest"], work,
             UNREGISTERED_SELFTEST_KIND),
            # 只认单独一个 --selftest（开关 selftest-extra-arguments-allowed 下这两格与上面广谱变异混运行参数那一格红）
            ("登记成轻的脚本带别的参数再带 --selftest 照拒：mutate.sh 只认头一个参数，这样跑的是 crates 整表",
             ["bash", "research/scripts/mutate.sh", "x", "y", "crates/mutations.tsv", "--selftest"], repository, UNREGISTERED_SELFTEST_KIND),
            ("登记成轻的脚本带别的参数再带 --selftest 照拒：门禁共用库带认不出的参数", ["bash", ".claude/gate.d/lib/stage-cells.sh", "--other"], repository,
             UNREGISTERED_SELFTEST_KIND),
            ("gate-structure-check.py", ["python3", "research/scripts/gate-structure-check.py"], work, "commit-check-structure"),
            ("研究脚本不带 --selftest 照常用", ["python3", "research/scripts/replace-once.py", "a.md", "x", "y"], work, None),
            ("仓外的 --selftest 不算", ["python3", "/tmp/claude-1000/elsewhere/tool.py", "--selftest"], work, None),
            # 登记表的键按解析后的真实路径认（开关 selftest-registry-by-path-tail 下前三格红）
            ("别处树里路径尾部相同的 claim-experiment.sh 照拒（攻方 2026-10-01：正文起 checker 档，带 --selftest 就放行）",
             ["bash", os.path.join(work, "evil-tree", "research", "scripts", "claim-experiment.sh"), "--selftest"], work, UNREGISTERED_SELFTEST_KIND),
            ("仓副本里的 admission.py 照拒（当前目录在副本里、相对着写）", ["python3", "research/scripts/admission.py", "--selftest"],
             os.path.join(work, "repository-copy"), UNREGISTERED_SELFTEST_KIND),
            ("路径写在变量里的照拒（解不出是哪一份）", ["bash", "$R/research/scripts/claim-experiment.sh", "--selftest"], work, UNREGISTERED_SELFTEST_KIND),
            ("指到本仓那一份的符号链接按指到的算，放行", ["python3", os.path.join(work, "linked-tree", "research", "scripts", "admission.py"), "--selftest"], work, None),
            # 命令词带通配的照 bash 展开、逐个判（test-script-align-r2 攻方 U1-1：一个 ? 就让真层 0、真 QEMU、真变异整表、重型自证一起放行；
            # 开关 pattern-command-words-literal 下放行四格之前的十格红（找不到 nice 时少「写成通配的 nice」那一格；通配起登记成重的自证那一格在 heavy_only_registry_cells，同样红），
            # pattern-expansions-bare 下「写成通配的 capped.sh」「写成通配的 detached-run.sh」「写成通配的 nice」三格红，
            # pattern-caret-negation-literal 下「[^…]」那一格红）
            ("通配起 checker-tier-layer0-replay", ["bash", ".claude/gate.d/checker-tier-layer0-replay.s?", repository], repository, "layer0-stage"),
            ("通配起 QEMU 那一道", ["bash", ".claude/gate.d/checker-tier-qemu-device-streams.s?", repository], repository, "qemu-stage"),
            ("通配起 mutate.sh 跑 crates 整表", ["bash", "research/scripts/mutate.s?", "crates/mutations.tsv"], repository, "crates-mutation-mutate"),
            ("通配起 vm-bench.sh --selftest", ["bash", "research/scripts/vm-bench*.sh", "--selftest"], repository, "vm-bench"),
            ("绝对路径里带通配起 checker-tier-layer0-replay", ["bash", os.path.join(repository, ".claude", "gate.d", "checker-tier-layer0-*.sh"), repository], "/", "layer0-stage"),
            ("展开成几份：bash 真起头一份、其余当参数（--selftest 之外还有参数）", ["bash", "research/scripts/crash-amplification*.sh", "--selftest"], repository,
             UNREGISTERED_SELFTEST_KIND),
            ("[^…] 写的字符类照 bash 认", ["bash", ".claude/gate.d/checker-tier-layer0-replay.[^x]h", repository], repository, "layer0-stage"),
            ("写成通配的 capped.sh 包全量", ["bash", "research/scripts/capped.s?", "4", "cargo", "test", "--all"], repository, "full-cargo"),
            ("写成通配的 detached-run.sh start 包全量", ["bash", "research/scripts/detached-run.s?", "start", "--label", "m", "--", "cargo", "test", "--all"],
             repository, "full-cargo"),
            *([("写成通配的 nice 包全量", [shutil.which("nice")[:-1] + "?", "-n", "19", "cargo", "test", "--all"], repository, "full-cargo")]
              if shutil.which("nice") else []),
            ("通配起登记成轻的自证放行", ["bash", "research/scripts/admission*.py", "--selftest"], repository, None),
            ("合法的通配：ls research/scripts/*.sh", ["ls", "research/scripts/*.sh"], repository, None),
            ("零命中的通配照字面判", ["bash", "research/scripts/no-such-script-*.sh"], repository, None),
            ("[ 内建命令不展开", ["[", "-f", "x", "]"], repository, None),
            # 方括号里的 POSIX 字符类、等价类、整理符号交给 bash 展开（test-script-align-r3 攻方 A1：python glob 零命中、照字面放行；
            # 开关 posix-brackets-by-python-glob 下前七格红，posix-brackets-unresolved-literal 下第七格红；POSIX 字符类起登记成重的自证那一格在 heavy_only_registry_cells）
            ("POSIX 字符类起 mutate.sh 跑 crates 整表", ["bash", "research/scripts/mutat[[:alpha:]].sh", "crates/mutations.tsv"], repository, "crates-mutation-mutate"),
            ("等价类起 checker-tier-layer0-replay", ["bash", ".claude/gate.d/checker-tier-layer0-replay.s[[=h=]]", repository], repository, "layer0-stage"),
            ("整理符号起 QEMU 那一道", ["bash", ".claude/gate.d/checker-tier-qemu-device-streams.s[[.h.]]", repository], repository, "qemu-stage"),
            ("POSIX 字符类夹在别的字符后面起 herd7 那一道（s[a[:lower:]]）", ["bash", ".claude/gate.d/checker-tier-lkmm.s[a[:lower:]]", repository], repository, "herd7-stage"),
            ("绝对路径里带 POSIX 字符类起 vm-bench.sh", ["bash", os.path.join(repository, "research", "scripts", "vm-bench.s[[:alpha:]]"), "--selftest"], "/", "vm-bench"),
            ("参数里的 POSIX 字符类点到 crates 整表（crates/mutation[[:alpha:]].tsv）", ["bash", "research/scripts/mutate.sh", "crates/mutation[[:alpha:]].tsv"], repository,
             "crates-mutation-mutate"),
            ("cd 到变量路径之后 POSIX 字符类起脚本照拒", ["bash", "-c", 'cd "$X" && bash research/scripts/mutat[[:alpha:]].sh crates/mutations.tsv'], repository,
             "pattern-unresolved"),
            ("合法的 POSIX 字符类：ls research/scripts/[[:alpha:]]*.sh", ["ls", "research/scripts/[[:alpha:]]*.sh"], repository, None),
            ("零命中的 POSIX 字符类照字面判", ["bash", "research/scripts/no-such-[[:alpha:]].sh"], repository, None),
            ("POSIX 字符类起登记成轻的自证放行", ["python3", "research/scripts/admissio[[:alpha:]].py", "--selftest"], repository, None),
            # 崩溃放量的三份入口按名字算重型（test-script-align-r3 攻方 A2：真跑放行、同一道的 --selftest 倒照拒；开关 crash-amplification-unnamed 下前六格红，
            # posix-brackets-by-python-glob 下第六格红）
            ("崩溃放量全量（crash-amplification.sh 不带参数）", ["bash", "research/scripts/crash-amplification.sh"], repository, "crash-amplification"),
            ("崩溃放量点名分项", ["bash", "research/scripts/crash-amplification.sh", "--item", "fixed-script", "--threads", "4"], repository, "crash-amplification"),
            ("崩溃放量的分项驱动", ["bash", "research/scripts/crash-amplification-item.sh", "crash_amplification_fixed_script_through_rollback"], repository,
             "crash-amplification"),
            ("崩溃放量的双机驱动真跑", ["bash", "research/scripts/crash-amplification-two-hosts.sh", "crash_amplification_fixed_script_through_rollback"], repository,
             "crash-amplification"),
            ("通配起崩溃放量全量（crash-amplificatio?.sh）", ["bash", "research/scripts/crash-amplificatio?.sh"], repository, "crash-amplification"),
            ("POSIX 字符类起崩溃放量全量", ["bash", "research/scripts/crash-amplification.s[[:alpha:]]"], repository, "crash-amplification"),
            ("崩溃放量只列分项（--list-items）放行", ["bash", "research/scripts/crash-amplification.sh", "--list-items"], repository, None),
            ("崩溃放量分项驱动单独 --selftest 照登记表放行", ["bash", "research/scripts/crash-amplification-item.sh", "--selftest"], repository, None),
            # 真编 QEMU 按名字算重型、包装里面的同样算（开关 qemu-build-unnamed 下前五格红；commit-checks-ignored、selftest-registry-ignored 下末格红）
            ("真编 QEMU（build-patched-qemu.sh 不带参数）", ["bash", "research/scripts/build-patched-qemu.sh"], repository, "qemu-build"),
            ("真编 QEMU --rebuild", ["bash", "research/scripts/build-patched-qemu.sh", "--rebuild"], repository, "qemu-build"),
            ("真编 QEMU 带别的参数（--force 由 preflight 摘掉，照编）", ["bash", "research/scripts/build-patched-qemu.sh", "--force"], repository, "qemu-build"),
            ("run-with-memory-cap.sh 包着真编 QEMU --rebuild", ["bash", "research/scripts/run-with-memory-cap.sh", "8G", "bash", "research/scripts/build-patched-qemu.sh",
                                                            "--rebuild"], repository, "qemu-build"),
            ("capped.sh 与 bash -c 包着真编 QEMU", ["nice", "-n", "19", "bash", os.path.join(repository, "research", "scripts", "capped.sh"), "16", "bash", "-c",
                                                "bash " + os.path.join(repository, "research", "scripts", "build-patched-qemu.sh")], "/", "qemu-build"),
            ("真编 QEMU 的 --selftest 照登记表放行", ["bash", "research/scripts/build-patched-qemu.sh", "--selftest"], repository, None),
            # 花括号与参数里的通配照 bash 展开再判（2026-10-01 主 agent 喂本仓钩子：前三格那样写都放行；开关 brace-expansion-literal 下「花括号起 mutate.sh」
            # 到「花括号里的整数序列展开成几份」七格红，brace-sequence-literal 下两格序列红，argument-patterns-literal 下「参数里的通配」一格红，
            # brace-command-words-bare 下「花括号写在包装上」一格红）
            ("花括号起 mutate.sh 跑 crates 整表（mutate.{sh,x}）", ["bash", "research/scripts/mutate.{sh,x}", "crates/mutations.tsv"], repository, "crates-mutation-mutate"),
            ("花括号起 checker-tier-layer0-replay （checker-tier-layer0-replay.{sh,}）", ["bash", ".claude/gate.d/checker-tier-layer0-replay.{sh,}", repository], repository, "layer0-stage"),
            ("参数里的花括号点到 checker 档包（-p singlefs-checker-{tier,x}）", ["cargo", "test", "-p", "singlefs-checker-{tier,x}"], work, "checker-tier-cargo"),
            ("花括号写在包装上（{bash,-x} research/scripts/mutate.sh：展开成 bash -x 起它）", ["{bash,-x}", "research/scripts/mutate.sh", "crates/mutations.tsv"], repository,
             "crates-mutation-mutate"),
            ("花括号里的字母序列点到 crates 整表（mutations.ts{v..w}）", ["bash", "research/scripts/mutate.sh", "crates/mutations.ts{v..w}"], repository,
             "crates-mutation-mutate"),
            ("命令词的花括号展开出的一份单独当命令（{x,mutate}.sh：bash 起的是 x.sh，mutate.sh 那一份照判）",
             ["bash", "research/scripts/{x,mutate}.sh", "crates/mutations.tsv"], repository, "crates-mutation-mutate"),
            ("花括号里的整数序列展开成几份（checker-tier-layer{0..1}-replay.sh）", ["bash", ".claude/gate.d/checker-tier-layer{0..1}-replay.sh", repository], repository, "layer0-stage"),
            ("参数里的通配点到 crates 整表（crates/mutation?.tsv）", ["bash", "research/scripts/mutate.sh", "crates/mutation?.tsv"], repository, "crates-mutation-mutate"),
            ("合法的花括号：ls research/scripts/{a,b}.sh", ["ls", "research/scripts/{a,b}.sh"], repository, None),
            ("合法的参数通配：cat crates/*.toml", ["cat", "crates/*.toml"], repository, None),
            ("参数的花括号照 bash 拆成两个词：-p singlefs-{harness,checker-tier} 只挑 harness 档、后一个是过滤词",
             ["cargo", "test", "-p", "singlefs-{harness,checker-tier}"], work, None),
            ("find -exec 的 {} 与 awk 的 {…} 不当重型", ["find", ".", "-name", "*.rs", "-exec", "awk", "{print $1, $2}", "{}", "+"], repository, None),
            # 命令词带通配的不当自证：case 的模式行切成的命令（攻方扫描脚本那一行；开关 selftest-pattern-words-judged 下这一格红）
            ("case 模式行切出的 .claude/gate.d/lib/*.py 不当起共用库", ["bash", "-c", "case \"$k\" in\n  .claude/gate.d/lib/*.py) x=1 ;;\nesac"], repository, None),
            # 只有脚本位上的词算被执行：`python3 -` 后面的门禁文件名是给标准输入那段程序的参数、`bash -n` 只查语法、读文件的命令只读
            # （lib_shell_words.py 的开关 LIB_SHELL_WORDS_BREAK=arguments-as-executed 下前两格红）
            ("python3 - 后面的门禁文件名是参数（门禁目录里跑）", ["python3", "-", "doc-text.sh", "doc-process-records.sh", "doc-experiments.sh"],
             os.path.join(work, ".claude", "gate.d"), None),
            ("bash -n 只查门禁的语法", ["bash", "-n", ".claude/gate.d/code-tooling.sh"], work, None),
            ("grep 读门禁文件", ["grep", "-n", "gate-cell", ".claude/gate.d/checker-tier-lkmm.sh"], work, None),
            ("python3 起的门禁共用库以外的脚本、门禁文件名只当它的参数", ["python3", "research/scripts/replace-once.py", ".claude/gate.d/code-tooling.sh", "a", "b"],
             work, None),
        ]
        # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的程序（command_under_launcher）
        launcher_cases = [
            ("/usr/bin/time -v 包 checker 档", ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("time -f 格式 -o 文件包 checker 档的集成测试",
             ["time", "-f", "%M", "-o", "/tmp/t", "cargo", "test", "--release", "-p", "singlefs-checker-tier", "--test", "crash_enumeration_new_pool_file_creation_stream"],
             work, "checker-tier-cargo"),
            ("systemd-run --working-directory= 进 checker 档裸跑",
             ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-checker-tier", "cargo", "test"], work, "checker-tier-cargo"),
            ("systemd-run --working-directory= 进 harness 档裸跑",
             ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-harness", "cargo", "test"], work, None),
            ("flock -w 秒数 锁文件 包一层", ["flock", "-w", "5", "/tmp/lock", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("flock 锁文件 -c 字符串", ["flock", "/tmp/lock", "-c", "cargo test --workspace"], work, "full-cargo"),
            ("rustup run --install stable 包一层", ["rustup", "run", "--install", "stable", "cargo", "test", "--all"], work, "full-cargo"),
            ("chrt -i 0 包一层", ["chrt", "-i", "0", "cargo", "test", "--all"], work, "full-cargo"),
            ("prlimit --as= 包一层", ["prlimit", "--as=8000000000", "cargo", "test", "--all"], work, "full-cargo"),
            ("systemd-run --user --scope -p MemoryMax=8G 包一层", ["systemd-run", "--user", "--scope", "-q", "-p", "MemoryMax=8G", "cargo", "test", "--all"],
             work, "full-cargo"),
            ("strace -f -o 文件包一层", ["strace", "-f", "-o", "/tmp/trace", "cargo", "test", "--all"], work, "full-cargo"),
            ("perf stat -e 事件包一层", ["perf", "stat", "-e", "cycles", "cargo", "test", "--all"], work, "full-cargo"),
            ("perf record -g -- 之后", ["perf", "record", "-g", "--", "cargo", "test", "--all"], work, "full-cargo"),
            ("perf mem record 包一层", ["perf", "mem", "record", "cargo", "test", "--all"], work, "full-cargo"),
            ("nice 里 /usr/bin/time 里 flock 一层套一层", ["nice", "-n", "19", "/usr/bin/time", "-v", "flock", "/tmp/l", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace -p 进程号：没有要起的命令", ["strace", "-p", "1234"], work, None),
            ("chrt -p 查优先级：没有要起的命令", ["chrt", "-p", "1234"], work, None),
            ("rustup show 不是 rustup run", ["rustup", "show"], work, None),
            ("perf report 不起命令", ["perf", "report", "-i", "perf.data"], work, None),
            ("/usr/bin/time 包一个不重型的测试目标", ["/usr/bin/time", "-v", "cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
            # 短选项合写：逐个字母查表，头一个要值的字母之后剩下的是值、没剩下取下一个词（开关 launcher-whole-word-options 下前四格红）
            ("strace -fo 文件（短选项合写）", ["strace", "-fo", "/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
            ("flock -xw 10 锁文件（短选项合写）", ["flock", "-xw", "10", "/tmp/l", "cargo", "test", "--all"], work, "full-cargo"),
            ("systemd-run --user --scope -qu 名字（短选项合写）", ["systemd-run", "--user", "--scope", "-qu", "k1", "cargo", "test", "--all"], work, "full-cargo"),
            ("/usr/bin/time -ao 文件（短选项合写）", ["/usr/bin/time", "-ao", "/tmp/t", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace -o文件（值贴着写）", ["strace", "-o/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
            # systemd-run -E / --setenv 设的变量带进里面那条命令（开关 launcher-drops-setenv 下前三格红）
            ("systemd-run -E 设别名", ["systemd-run", "--user", "--scope", "-E", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run --setenv= 设别名", ["systemd-run", "--user", "--scope", "--setenv=" + alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -qE 合写设别名", ["systemd-run", "--user", "--scope", "-qE", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -E 设的不是别名（值的末段叫 strace 也不当包装剥）", ["systemd-run", "--user", "--scope", "-E", "TRACER=/usr/bin/strace", "cargo", "xt"],
             work, None),
            # strace -E / --env 设的变量同样带进里面那条命令（开关 strace-drops-env 下前四格红）
            ("strace -E 设别名", ["strace", "-f", "-E", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace --env= 设别名", ["strace", "--env=" + alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace --env 值另起一个词设别名", ["strace", "--env", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace -fE 合写设别名", ["strace", "-fE", alias_to_tier, "cargo", "xt"], work, "checker-tier-cargo"),
            ("strace -E 只清变量（不带 =）", ["strace", "-E", "CARGO_ALIAS_XT", "cargo", "xt"], work, None),
            # 要值的长选项、值另起一个词：跳过那个值（开关 launcher-long-options-partial 下前四格红）
            ("systemd-run --user --scope --expand-environment no（值另起一个词）包一层",
             ["systemd-run", "--user", "--scope", "--expand-environment", "no", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace --output 文件（值另起一个词）包一层", ["strace", "--output", "/tmp/s", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace --string-limit 200 包一层", ["strace", "--string-limit", "200", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace -c --summary-col calls（长选项只写前缀）包一层", ["strace", "-c", "--summary-col", "calls", "cargo", "test", "--all"], work, "full-cargo"),
            ("strace --quiet（不带值的长选项）包一层", ["strace", "--quiet", "cargo", "test", "--all"], work, "full-cargo"),
            ("systemd-run --user --scope --collect --same-dir（不带值的长选项）包一层",
             ["systemd-run", "--user", "--scope", "--collect", "--same-dir", "cargo", "test", "--all"], work, "full-cargo"),
            # systemd-run -p Environment= / EnvironmentFile= 设给里面那条命令的变量（开关 launcher-drops-property-environment 下前四格红）
            ("systemd-run -p Environment= 设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", 'Environment="' + alias_to_tier + '"', "cargo", "xt"],
             work, "checker-tier-cargo"),
            ("systemd-run --property=Environment= 设两个变量、第二个是别名",
             ["systemd-run", "--user", "--wait", "--pipe", '--property=Environment=A=1 "' + alias_to_tier + '"', "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run --prop Environment=（长选项只写前缀）设别名", ["systemd-run", "--user", "--wait", "--pipe", "--prop", 'Environment="' + alias_to_tier + '"',
                                                                  "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -p EnvironmentFile= 文件里设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "alias.env"),
                                                         "cargo", "xt"], work, "checker-tier-cargo"),
            ("systemd-run -p Environment= 设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "Environment=RUST_BACKTRACE=1", "cargo", "xt"], work, None),
            ("systemd-run -p EnvironmentFile= 文件里设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "plain.env"),
                                                               "cargo", "xt"], work, None),
            # research/scripts/detached-run.sh start … -- <命令>：剥掉之后里面那条照判（开关 detached-run-opaque 下前四格红）
            ("bash 起 detached-run.sh start 包全量 cargo test", ["bash", "research/scripts/detached-run.sh", "start", "--label", "m", "--", "cargo", "test", "--all"],
             work, "full-cargo"),
            ("detached-run.sh start --label= --log 文件 包 checker 档",
             ["research/scripts/detached-run.sh", "start", "--label=m", "--log", "/tmp/m.log", "--", "cargo", "test", "-p", "singlefs-checker-tier"], work,
             "checker-tier-cargo"),
            ("detached-run.sh --force start 包一层 nice 里的全量", ["bash", "research/scripts/detached-run.sh", "--force", "start", "--label", "m", "--",
                                                          "nice", "-n", "19", "cargo", "test", "--workspace"], work, "full-cargo"),
            ("detached-run.sh start 里面再套 /usr/bin/time", ["bash", "/abs/research/scripts/detached-run.sh", "start", "--label", "m", "--",
                                                       "/usr/bin/time", "-v", "cargo", "test", "--all"], work, "full-cargo"),
            ("detached-run.sh start 没写 --（它按用法错拒、什么都不跑）", ["bash", "research/scripts/detached-run.sh", "start", "--label", "m", "cargo", "test", "--all"],
             work, None),
            ("detached-run.sh list 不起命令", ["bash", "research/scripts/detached-run.sh", "list"], work, None),
            ("detached-run.sh start 包不重型的测试目标", ["bash", "research/scripts/detached-run.sh", "start", "--label", "m", "--",
                                                    "cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], work, None),
            ("detached-run.sh start 带 --stop-timeout 照剥", ["bash", "research/scripts/detached-run.sh", "start", "--label", "m", "--stop-timeout", "60", "--",
                                                          "cargo", "test", "--all"], work, "full-cargo"),
            # 它的 preflight 摘掉每一个 --force，单元里跑的是后面那条（lib_shell_words 的 LIB_SHELL_WORDS_BREAK=detached-run-force-kept 下这一格红）
            ("detached-run.sh start 的 -- 之后先写 --force 照摘", ["bash", "research/scripts/detached-run.sh", "start", "--label", "m", "--",
                                                              "--force", "cargo", "test", "--all"], work, "full-cargo"),
            # 按内容认 detached-run.sh：开头带签名行的改名拷贝、指到它的符号链接、source 读进来的（开关 detached-run-by-name-only 下前三格红）
            ("bash 起仓内改名的 detached-run.sh 拷贝包全量", ["bash", "research/scripts/dr-renamed.sh", "start", "--label", "m", "--", "cargo", "test", "--all"],
             work, "full-cargo"),
            ("直接执行仓外改名拷贝包 checker 档", [os.path.join(work, "elsewhere", "dr-renamed.sh"), "start", "--label", "m", "--",
                                                 "cargo", "test", "-p", "singlefs-checker-tier"], work, "checker-tier-cargo"),
            ("source 读进来的符号链接包全量", ["bash", "-c", "source elsewhere/dr-link.sh start --label m -- cargo test --all"], work, "full-cargo"),
            ("删掉签名行的改名拷贝认不出（罩不住的那一种，钉住现状）", ["bash", os.path.join(work, "elsewhere", "dr-without-signature.sh"), "start", "--label", "m", "--",
                                                          "cargo", "test", "--all"], work, None),
        ]
        binary_cases = [
            ("绝对路径的 checker 档集成测试二进制", [os.path.join(work, "target", "release", "deps", tier_binary), "--test-threads", "4"], work, "checker-tier-binary"),
            ("相对路径的 checker 档集成测试二进制", [os.path.join("target", "debug", "deps", tier_binary)], work, "checker-tier-binary"),
            ("经 timeout 包一层", ["timeout", "600", os.path.join(".", "target", "release", "deps", tier_binary), "--exact", "case"], work, "checker-tier-binary"),
            ("checker 档装置二进制的内联单测", [os.path.join(work, "target", "debug", "deps", "e161_sample_device-0123456789abcdef")], work, "checker-tier-binary"),
            ("checker 档库的单测", [os.path.join(work, "target", "debug", "deps", "singlefs_checker_tier-0123456789abcdef")], work, "checker-tier-binary"),
            ("checker 档二进制 --logfile --list（--list 是日志文件名）", [os.path.join(work, "target", "release", "deps", tier_binary), "--logfile", "--list"], work,
             "checker-tier-binary"),
            ("checker 档二进制 --list（只列）", [os.path.join(work, "target", "release", "deps", tier_binary), "--list"], work, None),
            ("harness 档的测试二进制", [os.path.join(work, "target", "release", "deps", "overwrite_in_one_instance-0123456789abcdef")], work, None),
            ("名字带 layer0 而不是 checker 档的二进制（按包判，不按名字）", ["/tmp/target-elsewhere/release/deps/some_layer0_stream-fedcba9876543210"], work, None),
            ("deps 下的 .d 依赖文件不是二进制", [os.path.join(work, "target", "release", "deps", tier_binary + ".d")], work, None),
            ("哈希不是 16 位", [os.path.join(work, "target", "release", "deps", "crash_enumeration_new_pool_file_creation_stream-0123abcd")], work, None),
            ("名字只当参数", ["grep", "-c", "x", os.path.join("target", "release", "deps", tier_binary)], work, None),
        ]
        own_repository_targets = sorted(checker_test_targets(HOOK_REPOSITORY))
        if own_repository_targets:
            binary_cases.append(("仓外执行、取这份文件所在仓的 checker 档包：直接执行它的测试二进制按包判",
                                 [f"/tmp/target-elsewhere/release/deps/{own_repository_targets[0]}-0123456789abcdef"], "/", "checker-tier-binary"))
        else:
            results.append((f"这份文件所在仓（{HOOK_REPOSITORY}）里找得到 checker 档包的测试目标", True, False))
        # 门禁样本（文件头「门禁样本」）：读仓里那份登记表。期望放行的两格在 stage-samples-ignored 下红，其余几格各对应一个开关（写在文件头）
        # 喂样本只信本仓那一份 stage-selftest.sh（写它在本仓的绝对路径），带不带 --item 都是；别处树里路径尾部相同的拷贝照拒
        own_gate_selftest = os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])
        sample_layer0, gate_selftest = os.path.join(work, "sample-layer0"), own_gate_selftest
        evil_gate_selftest = os.path.join(work, "evil-tree", KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])
        stage_layer0 = os.path.join(work, ".claude", "gate.d", "checker-tier-layer0-replay.sh")
        stage_lkmm = os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "checker-tier-lkmm.sh")
        sample = {name: os.path.join(work, name) for name in ("sample-layer0-empty", "sample-layer0-quick-tier-only", "sample-layer0-forged", "sample-layer0-noexec",
                                                             "sample-lkmm", "sample-lkmm-directory", "sample-lkmm-content")}
        def fed(name):
            return os.path.join(work, name, ".claude", "gate.d")
        stage_sample_cases = [
            # 标记里装的是不是仓里那一道样本放的那种（开关 stage-samples-marker-unchecked 下这六格红）
            ("checker-tier-layer0-replay 样本根的标记目录是空的照算层 0", ["bash", stage_layer0, sample["sample-layer0-empty"]], sample["sample-layer0-empty"], "layer0-stage"),
            ("checker-tier-layer0-replay 样本根只放了 quick-tier（攻方 2026-10-01 的造法）照算层 0", ["bash", stage_layer0, "--full", sample["sample-layer0-quick-tier-only"]],
             sample["sample-layer0-quick-tier-only"], "layer0-stage"),
            ("checker-tier-layer0-replay 样本根的假 cargo 内容不同照算层 0", ["bash", stage_layer0, sample["sample-layer0-forged"]], sample["sample-layer0-forged"], "layer0-stage"),
            ("checker-tier-layer0-replay 样本根的假 cargo 不可执行照算层 0", ["bash", stage_layer0, "."], sample["sample-layer0-noexec"], "layer0-stage"),
            ("herd7 样本根的标记写成目录照算 herd7", ["bash", stage_lkmm, sample["sample-lkmm-directory"]], sample["sample-lkmm-directory"], "herd7-stage"),
            ("herd7 样本根的标记内容不同照算 herd7", ["bash", stage_lkmm, sample["sample-lkmm-content"]], sample["sample-lkmm-content"], "herd7-stage"),
            ("herd7 指到放了原样标记的样本根：样本档放行", ["bash", stage_lkmm, sample["sample-lkmm"]], sample["sample-lkmm"], None),
            # 喂的样本与暂存区比（SAMPLE_INDEX_REPOSITORY 换成临时样本仓；开关 stage-samples-fixtures-unchecked 下这五格红）
            ("stage-selftest.sh 喂的样本多一份 setup.sh（攻方 2026-10-01：setup.sh 起 checker 档）照算", ["bash", gate_selftest, fed("gd-sample-setup")], work,
             "commit-check-lint"),
            ("stage-selftest.sh 喂的样本内容改过照算", ["bash", gate_selftest, fed("gd-sample-edited")], work, "commit-check-lint"),
            ("stage-selftest.sh 喂的样本执行位改过照算", ["bash", gate_selftest, fed("gd-sample-mode")], work, "commit-check-lint"),
            ("stage-selftest.sh 喂的样本少了文件照算", ["bash", gate_selftest, fed("gd-sample-missing")], work, "commit-check-lint"),
            ("stage-selftest.sh 喂暂存之前新加的样本照算（样本仓自己的阶段目录）", ["bash", gate_selftest, fed(SAMPLE_INDEX_REFERENCE_NAME)], work, "commit-check-lint"),
            ("checker-tier-layer0-replay 指到放了 .layer0-sample-tools 的样本根（cd 进去、根参数写同一个目录）：样本档放行", ["bash", stage_layer0, sample_layer0, "--force"], sample_layer0, None),
            ("checker-tier-layer0-replay 在真仓上跑（当前目录是样本根，根参数指到别处）照算层 0", ["bash", stage_layer0, work], sample_layer0, "layer0-stage"),
            ("checker-tier-layer0-replay 指到没放标记的目录照算层 0", ["bash", stage_layer0, os.path.join(work, "sample-layer0-bare"), "--force"], os.path.join(work, "sample-layer0-bare"), "layer0-stage"),
            ("checker-tier-layer0-replay 不写根参数（阶段默认判真仓）照算层 0", ["bash", stage_layer0, "--force"], sample_layer0, "layer0-stage"),
            ("改过正文的 checker-tier-layer0-replay 拷贝指到样本根照算层 0", ["bash", os.path.join(work, "elsewhere", ".claude", "gate.d", "checker-tier-layer0-replay.sh"), sample_layer0], sample_layer0, "layer0-stage"),
            ("crates 变异表那一道指到样本根（样本档登记成起重活）照算", ["bash", os.path.join(work, ".claude", "gate.d", "checker-tier-crates-mutation-replay.sh"),
                                                              os.path.join(work, "sample-crates"), "--force"], os.path.join(work, "sample-crates"), "crates-mutation-stage"),
            ("stage-selftest.sh 喂样本档全登记成轻的阶段目录：放行", ["bash", gate_selftest, os.path.join(work, "gd-light", ".claude", "gate.d")], work, None),
            ("stage-selftest.sh 喂有起重活样本的阶段目录（crates 变异表）照算提交时才跑的检查", ["bash", gate_selftest, os.path.join(work, "gd-heavy", ".claude", "gate.d")],
             work, "commit-check-lint"),
            ("stage-selftest.sh 喂有登记表外阶段的目录照算", ["bash", gate_selftest, os.path.join(work, "gd-unregistered", ".claude", "gate.d")], work, "commit-check-lint"),
            ("stage-selftest.sh 喂轻阶段正文改过的目录照算", ["bash", gate_selftest, os.path.join(work, "gd-edited", ".claude", "gate.d")], work, "commit-check-lint"),
            ("stage-selftest.sh 阶段目录写在变量里照算", ["bash", gate_selftest, "$GD"], work, "commit-check-lint"),
            # --item 只判点名的几道（文件头「门禁样本」①）：放行的六格在 stage-items-as-directory 下红，拒的几格在 stage-items-unjudged 下红
            ("stage-selftest.sh --item 点名一道轻的（目录里另有起重活的 crates 变异表）：放行", ["bash", own_gate_selftest, fed("gd-heavy"), "--item", "doc-text.sh"], work, None),
            ("stage-selftest.sh --item 点名带标记的一道（红样本里的假 cargo 故意不可执行，与暂存区相同）：放行", ["bash", own_gate_selftest, fed("gd-light"), "--item",
                                                                                    "checker-tier-layer0-replay.sh"], work, None),
            ("stage-selftest.sh --list-items 只列、不喂样本：放行", ["bash", own_gate_selftest, fed("gd-heavy"), "--list-items"], work, None),
            ("stage-selftest.sh --item 点名不存在的阶段（它自己退 2、一份样本都不喂）：放行", ["bash", own_gate_selftest, fed("gd-heavy"), "--item", "no-such-stage.sh"],
             work, None),
            ("stage-selftest.sh 阶段目录写成 $PWD/…、--item 点名一道轻的：放行", ["bash", own_gate_selftest, "$PWD/gd-heavy/.claude/gate.d", "--item", "doc-text.sh"],
             work, None),
            ("stage-selftest.sh --item 与值之间夹的 --force 照 preflight 摘掉：放行", ["bash", own_gate_selftest, fed("gd-heavy"), "--item", "--force", "doc-text.sh"],
             work, None),
            ("stage-selftest.sh --item 点名里夹一道起重活的照算", ["bash", own_gate_selftest, fed("gd-heavy"), "--item", "doc-text.sh",
                                                         "--item", "checker-tier-crates-mutation-replay.sh"], work, "commit-check-lint"),
            ("stage-selftest.sh --item 一个值里带空格点到起重活的照算", ["bash", own_gate_selftest, fed("gd-heavy"), "--item",
                                                          "checker-tier-crates-mutation-replay.sh doc-text.sh"], work, "commit-check-lint"),
            ("stage-selftest.sh --item 点名里有不存在的、又有起重活的照算", ["bash", own_gate_selftest, fed("gd-heavy"), "--item", "no-such-stage.sh",
                                                           "--item", "checker-tier-crates-mutation-replay.sh"], work, "commit-check-lint"),
            ("stage-selftest.sh --item 点名登记表外的阶段照算", ["bash", own_gate_selftest, fed("gd-unregistered"), "--item", "brand-new-stage.sh"], work,
             "commit-check-lint"),
            ("stage-selftest.sh --item 点名的样本多一份 setup.sh 照算", ["bash", own_gate_selftest, fed("gd-sample-setup"), "--item", "doc-text.sh"], work,
             "commit-check-lint"),
            ("stage-selftest.sh --item 点名的阶段正文改过照算", ["bash", own_gate_selftest, fed("gd-edited"), "--item", "doc-text.sh"], work, "commit-check-lint"),
            ("stage-selftest.sh --item 点名写在变量里照算", ["bash", own_gate_selftest, fed("gd-heavy"), "--item", "$STAGE"], work, "commit-check-lint"),
            ("stage-selftest.sh 阶段目录写成 $PWD/…、命令里另设了 PWD 的不换，照算", ["env", "PWD=/elsewhere", "bash", own_gate_selftest, "$PWD/gd-heavy/.claude/gate.d",
                                                                   "--item", "doc-text.sh"], work, "commit-check-lint"),
            ("别处树里的 stage-selftest.sh 拷贝带 --list-items 照算", ["bash", evil_gate_selftest, fed("gd-heavy"), "--list-items"], work, "commit-check-lint"),
            ("别处树里的 stage-selftest.sh 拷贝 --item 点名一道轻的照算", ["bash", evil_gate_selftest, fed("gd-heavy"), "--item", "doc-text.sh"], work, "commit-check-lint"),
            # 不带 --item 也只信本仓那一份（test-script-align-r2 攻方 U1-2：改过正文的拷贝喂字节相同的轻样本放行；开关 stage-feed-script-by-name 下这两格红）
            ("别处树里的 stage-selftest.sh 拷贝不带 --item 喂全登记成轻的阶段目录照算", ["bash", evil_gate_selftest, fed("gd-light")], work, "commit-check-lint"),
            ("相对着写、当前目录不在本仓的 stage-selftest.sh 不带 --item 照算", ["bash", ".claude/singlefs-ai-sop/scripts/stage-selftest.sh", fed("gd-light")], work,
             "commit-check-lint"),
        ]
        for label, argv, cwd, want in stage_sample_cases:
            found = classify_process(argv, cwd)
            results.append((f"门禁样本：{label}", want, found[0].kind if found else None))
        refused_heavy_directory = classify_process(["bash", gate_selftest, os.path.join(work, "gd-heavy", ".claude", "gate.d")], work)
        results.append(("门禁样本：stage-selftest.sh 被拒的说明点名起重活的那一道、写单跑样本的写法", True,
                        bool(refused_heavy_directory) and "checker-tier-crates-mutation-replay.sh 的样本档登记成起重活" in refused_heavy_directory[0].detail
                        and "cd 进样本根、根参数写同一个目录" in refused_heavy_directory[0].detail))
        refused_setup = classify_process(["bash", gate_selftest, fed("gd-sample-setup")], work)
        results.append(("门禁样本：样本与暂存区不同被拒的说明点名那份文件、写怎么办", True,
                        bool(refused_setup) and "red/setup.sh（暂存区里没有）" in refused_setup[0].detail and "先交主 agent 暂存" in refused_setup[0].detail))
        refused_item = classify_process(["bash", own_gate_selftest, fed("gd-heavy"), "--item", "doc-text.sh", "--item", "checker-tier-crates-mutation-replay.sh"], work)
        results.append(("门禁样本：--item 点名里夹起重活的被拒，说明点名那一道、不点没毛病的那一道、写 --item 的写法", True,
                        bool(refused_item) and "checker-tier-crates-mutation-replay.sh 的样本档登记成起重活" in refused_item[0].detail
                        and "doc-text.sh 的" not in refused_item[0].detail and "--item <阶段文件名>" in refused_item[0].detail))
        refused_bare = classify_process(["bash", stage_layer0, os.path.join(work, "sample-layer0-bare")], os.path.join(work, "sample-layer0-bare"))
        results.append(("门禁样本：checker-tier-layer0-replay 照算层 0 时说明里写样本档怎么跑", True,
                        bool(refused_bare) and "cd 进放了 .layer0-sample-tools 的样本根" in refused_bare[0].detail))
        for label, argv, cwd, want in cargo_cases + launcher_cases + binary_cases:
            found = classify_process(argv, cwd)
            results.append((label, want, found[0].kind if found else None))
        # bash 起不来时带 POSIX 字符类的命令词照拒（开关 posix-brackets-unresolved-literal 与 posix-brackets-by-python-glob 下这一格红）
        real_bash, BASH_FOR_PATTERNS = BASH_FOR_PATTERNS, os.path.join(work, "no-such-bash")
        try:
            bash_missing_cases = [("bash 起不来时 POSIX 字符类起脚本照拒", ["bash", "research/scripts/mutat[[:alpha:]].sh", "crates/mutations.tsv"], repository,
                                   "pattern-unresolved")]
            for label, argv, cwd, want in bash_missing_cases:
                found = classify_process(argv, cwd)
                results.append((label, want, found[0].kind if found else None))
        finally:
            BASH_FOR_PATTERNS = real_bash
        # 花括号展开与 bash 一致：同一批词交给 bash（set -f 关掉通配，只剩花括号展开）逐个比（开关 brace-expansion-literal 下大半红，brace-sequence-literal 下序列那几格红）
        brace_words = ["{a,b}", "a{b,c}d", "{a{b,c}}", "{a,b{c,d}}e", "{a,b", "{{a,b}", "{a,b}}", "{a}", "{}", "{,}", "x{,}", "{,a}", "mutate.{sh,x}",
                       "{a,{b}}", "a{{b}}", "a{b{c,d}", "{a,b}{", "}{a,b}", "{}a,b}", "..{},aa,}a.", "ab{}1..,}{.{", "{1..3}", "{3..1}", "{01..10}", "{1..10..3}",
                       "{a..e}", "{a..e..2}", "{1..a}", "{-2..2}", "{-01..3}", "{1..-03}", "{0..00}", "{1..05..2}", "{+1..3}", "{1..3..0}", "{1..3..-1}",
                       "{1...3}", "{a..}", "x{1..3}y{a,b}", "{1..2}{3..4}", "{a,b}{c,d}{e,f}"]
        script = "set -f\n" + "".join(f"set -- {word}\nprintf '%s\\0' \"$#\" \"$@\"\n" for word in brace_words)
        answer = subprocess.run(["bash"], input=script, capture_output=True, text=True).stdout.split("\0")
        position = 0
        try:
            for word in brace_words:
                count = int(answer[position])
                results.append((f"花括号展开与 bash 一致：{word}", answer[position + 1:position + 1 + count], brace_expansion(word)))
                position += 1 + count
        except (ValueError, IndexError):
            results.append(("花括号展开与 bash 一致：bash 的输出读得出", True, False))
        # 自证登记表的读法：换成样本表（写坏的几种行都有）判；仓里那份表另核没有写坏的行、指到的文件都在
        registry_cases = selftest_registry_cells(work, results)
        name_cases = [
            ("门禁阶段按名字判", ".claude/gate.d/doc-text.sh", work, True),
            ("去了编号的门禁阶段按名字判", ".claude/gate.d/checker-tier-lkmm.sh", work, True),
            ("门禁目录顶层的共用库 lib-*.sh 不是阶段", ".claude/gate.d/lib-sample.sh", work, False),
            ("lib/ 底下的共用库不是阶段", ".claude/gate.d/lib/stage-cells.sh", work, False),
            ("仓内位置上的 gate-staged.sh 按名字判", "research/scripts/gate-staged.sh", work, True),
            ("仓内位置上的 layer0-shard-run.sh 按名字判", "research/scripts/layer0-shard-run.sh", work, True),
            ("仓内位置上的 mutation-shard-run.sh 按名字判", "research/scripts/mutation-shard-run.sh", work, True),
            ("仓内位置上的 broad-mutation.py 按名字判", "research/scripts/broad-mutation.py", work, True),
            ("规范副本里的 stage-selftest.sh 按名字判（放行时不读正文）", ".claude/singlefs-ai-sop/scripts/stage-selftest.sh", work, True),
            ("仓外的同名 broad-mutation.py 不按名字判", "/tmp/claude-1000/somewhere/broad-mutation.py", work, False),
            ("从 research 里相对着写的 mutate.sh", "scripts/mutate.sh", os.path.join(work, "research"), True),
            ("仓外的同名 mutate.sh 不按名字判", "/tmp/claude-1000/somewhere/mutate.sh", work, False),
            ("仓内位置上的 crash-amplification.sh 按名字判（不读正文）", "research/scripts/crash-amplification.sh", work, True),
            ("仓内位置上的 crash-amplification-item.sh 按名字判", "research/scripts/crash-amplification-item.sh", work, True),
            ("仓内位置上的 crash-amplification-two-hosts.sh 按名字判", "research/scripts/crash-amplification-two-hosts.sh", work, True),
            ("仓内位置上的 build-patched-qemu.sh 按名字判（不读正文）", "research/scripts/build-patched-qemu.sh", work, True),
            ("没登记的脚本", "run-chain.sh", work, False),
        ]
        for label, word, directory, want in name_cases:
            results.append((label, want, judged_by_name(word, directory)))
        # 跑不跑编译出来的代码（与重型不重型无关）：真 / 假
        compiled_cases = [
            ("cargo test 一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], True),
            ("cargo nextest run", ["cargo", "nextest", "run", "-p", "singlefs-core"], True),
            ("cargo miri test", ["cargo", "miri", "test", "-p", "singlefs-core"], True),
            ("cargo llvm-cov 不带子命令", ["cargo", "llvm-cov", "-p", "singlefs-core"], True),
            ("cargo mutants", ["cargo", "mutants", "-p", "singlefs-core"], True),
            ("--config 定的别名", ["cargo", "--config", 'alias.xt="test"', "xt"], True),
            ("cargo nextest list 不跑", ["cargo", "nextest", "list"], False),
            ("cargo llvm-cov report 不跑", ["cargo", "llvm-cov", "report"], False),
            ("cargo t 简写", ["cargo", "t", "--lib"], True),
            ("cargo run 实验二进制", ["cargo", "run", "--release", "--bin", "e160-random-small-read-share"], True),
            ("cargo r 简写", ["cargo", "r"], True),
            ("cargo bench", ["cargo", "bench"], True),
            ("全局选项在子命令前", ["cargo", "+stable", "--offline", "-C", "crates/singlefs-core", "test"], True),
            ("cargo test --no-run 只编不跑", ["cargo", "test", "--no-run", "-p", "singlefs-core"], False),
            ("-- 之后的 --no-run 是测试二进制的参数", ["cargo", "test", "--", "--no-run"], True),
            ("cargo build", ["cargo", "build", "--release"], False),
            ("cargo b 简写是 build", ["cargo", "b"], False),
            ("cargo clippy", ["cargo", "clippy", "--all-targets"], False),
            ("cargo 只带全局选项", ["cargo", "--version"], False),
            ("直接执行测试二进制", [os.path.join("target", "debug", "deps", "overwrite_in_one_instance-0123456789abcdef")], True),
            ("直接执行 research 里的实验二进制", ["research/target/release/e142_region_diff_independent"], True),
            ("直接执行名字里带 target 的自定编译目录里的二进制", ["/tmp/singlefs-crates-mutation-target/release/tiny"], True),
            ("带目标三元组的", ["./target/x86_64-unknown-linux-musl/release/new_pool_file_creation_on_device"], True),
            ("deps 下的 .d 依赖文件", [os.path.join("target", "release", "deps", "tiny.d")], False),
            ("名字只当参数", ["ls", "target/release/tiny"], False),
            ("不在 target 底下的程序", ["./prog"], False),
        ]
        for label, words, want in compiled_cases:
            results.append((f"跑编译出来的代码：{label}", want, runs_compiled_code(words, work) is not None))
    finally:
        SAMPLE_INDEX_REPOSITORY = real_sample_index
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ lib_heavy_tests 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 classify()、cargo_subcommand()、cargo_use()、package_specification_members()、alias_expansion()、test_invocation()、"
              "command_under_launcher()、launcher_option()、libtest_lists_only()、checker_test_targets()、test_binary_name()、"
              "runs_compiled_code()、classify_process()、judged_by_name()、classify_selftest()、selftest_registry_at()、selftest_registry_problems()、"
              "stage_sample_run_allowed()、stage_sample_root()、stage_selftest_problems()、stage_selftest_arguments()、stage_selftest_item_problems()、stage_feed_problem()、"
              "same_as_registered_stage()、brace_expansion()、brace_gobble()、brace_sequence()、expanded_words()、word_expansion_use() 与登记表 "
              f"{SELFTEST_REGISTRY_NAME}、broad_mutation_driver()、broad_mutation_use()、verus_judge_script()、verus_judge_arguments()、verus_judge_use()、verus_judge_code_use()、e165_pin_mode()、"
              "command_under_detached_run()、lib_shell_words.py 的 detached_run_script()、detached_run_start_command()、python_invocation() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / "
              f"KNOWN_SCRIPT_LOCATIONS / LAUNCHER_OPTIONS_WITH_VALUE / LIBTEST_OPTIONS_WITH_VALUE 几张表；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_heavy_tests 自检通过（查了 {len(results)} 种：cargo 与包装过的命令行 {len(cargo_cases)} 种、"
          f"/usr/bin/time、flock、systemd-run 这一类包在外面的 {len(launcher_cases)} 种、"
          f"bash 起不来时的 POSIX 字符类 {len(bash_missing_cases)} 种、直接执行的测试二进制 {len(binary_cases)} 种、花括号展开与 bash 对照 {len(brace_words)} 种、门禁样本 {len(stage_sample_cases) + 4} 种、样本登记表与仓里那份登记表 {registry_cases} 种、按名字判的脚本 {len(name_cases)} 种、"
          f"跑不跑编译出来的代码 {len(compiled_cases)} 种）")
    return 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    print("用法：python3 .claude/hooks/lib_heavy_tests.py --selftest（判定入口按文件路径导入这份模块再调，见文件头）", file=sys.stderr)
    sys.exit(2)
