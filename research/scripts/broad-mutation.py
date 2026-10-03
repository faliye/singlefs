#!/usr/bin/env python3
# admission: always 每次调都判此刻库目录里那棵树与它这一版的结果表；哪些条要跑由结果表定，上一次的结论不替这一次作保
# run-condition: command git python3
# broad-mutation-driver-signature: singlefs 广谱变异逐条驱动（重型测试闸 .claude/hooks/lib_heavy_tests.py 按这一行认改名或拷走的一份，别删）
"""广谱变异的逐条驱动（.claude/kb/checks-owed.md 的 C489；用户 2026-09-30 定：一直跑、中断要续接不从头来、一条一条可点名）。

  broad-mutation.py --sync-tree [--commit <提交>]   把主仓某个提交（默认 HEAD）的源码导出成库目录里的专用树（git archive，不动主仓），
                                                   在树上由 cargo-mutants 生成这一版的固定清单；不跑测试；换的是全局的版本指针。
                                                   导出用 tar -x -m（文件的 mtime 取解开那一刻，不还原提交时刻），换树之后清掉编译目录（target/、target-coverage/）里
                                                   工作区各包（树里 Cargo.toml 的 [package] name）的 fingerprint，第三方依赖的不动：两样都做，换树之后工作区各包必重编
  broad-mutation.py --list-items                   逐行打「序号 <TAB> 包 <TAB> 变异名 <TAB> 这一版的状态」，不编不跑
  broad-mutation.py --summary                      这一版八栏条数，加起来要等于清单条数，对不上判红退 1；没红的与驱动错的逐条列名
  broad-mutation.py --check-ignored                只判这一版树里 harness 档的 #[ignore] 用例登记得对不对（见下「忽略用例登记」），打第二段慢档用例那一步的命令；不编不跑、不要前缀
  broad-mutation.py --unit-environment             只打一行：此刻环境里全局键与工具链认的变量（拿掉它全局键里的构建环境就变、或在树里起的 rustc -Vv 就变的那几个，
                                                   例 RUSTUP_TOOLCHAIN；research/scripts/detached-run.sh 本来就带进单元的 UNIT_CARRIED_VARIABLES 除外），
                                                   写成 `env 名=值 …`（按 shell 引好）；一个都没有打空行。不编（只起 rustc -Vv）、不跑、不改库目录。
                                                   提交流程 ③ 经 detached-run.sh 起单元之前跑它，把这一行原样接在 `--` 后面（.claude/skills/crash-test/commit-stages.md 第 2b 步）
  broad-mutation.py [选哪些条] [怎么跑]             跑；重跑同一条命令就是续跑
  broad-mutation.py --import-results <目录>        把另一台机器同一版的结果行导入本机库（两台各跑一片之后合）：核指纹、清单、变异名、序号
  broad-mutation.py --selftest                     自证（假清单、假测试命令，不起 cargo）

选哪些条（不给 --item 就是这一版还没有定论的全部条，按序号）：
  --item <序号或变异名>   可给多次，只跑点名的里面还没有定论的；已有定论的跳过并逐条说一句。不与 --from / --package / --shard 同用
  --rerun                 只与 --item 同用：点名的条已有定论也重跑，追加新行，旧行留着
  --from <序号>           只要序号不小于它的
  --package <包>          只要这个包的
  --shard <k>/<n>         只要「序号 mod n == k」的（k 取 0..n-1），两台机器各跑一半用
怎么跑：
  --skip-stage-two        只跑第一段；第一段没红的记「只过了第一段」（stage-one-only），不算「没红」定论
  --stage-two-only        只给过了第一段的条补第二段；点名的条没过过第一段（这一版里没有它的 stage-one-only 行）拒，退 2
  --stage-two-eatmydata   第二段的文档测试与 checker 档快档也经 eatmydata 跑（默认不经；harness 慢档用例那一步与第一段同一种起法，一直经）
  --threads <N>           线程数，默认取 SINGLEFS_THREAD_CAP，没设取核数减 1；编译与测试都照它（经 research/scripts/capped.sh）
  --memory-cap <上限>     每条命令的内存上限（research/scripts/run-with-memory-cap.sh 的写法），默认 BROAD_MUTATION_MEMORY_CAP，没设 24G
  --timeout-floor <秒>    单步超时的地板，默认 900；--timeout-multiplier <倍> 默认 5。单步超时 = max(地板, 倍数 × 这一版基线里这一步的秒数)：
                          基线是开跑时在这台机器当时的负载下量的（.claude/kb/tooling.md「超时地板按空闲机器标定，就会把「机器上还有别的活」记成「变异抓不到」」）
  --suspect-streak <K>    连续 K 条「无效」、或连续 K 条「测试那一步不到 --suspect-test-seconds 秒就红的抓到」，就在没改过的树上把那一步重跑一遍：
                          红了退 6 报「疑似环境坏了」并列出这 K 条，绿了接着跑。默认 8；--suspect-test-seconds 默认 0.2
                          （2026-09-30 真库头 32 行量的：无效最长连着 4 条，抓到的条测试那一步最短 0.3 秒；样本小，跑多了照实测再调）
  --stop-grace <秒>       被 TERM / INT 停时等在途那一步退出多久，默认 10，过了还在就 KILL
  --lock-wait <秒>        库目录的锁被上一趟留下的进程占着时等多久，默认 120
  --home <库目录>         默认 ${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation}；落在开机会清空的目录里拒

库目录：
  tree/            专用树（导出的源码，不是 git 仓）。变异就地改这里的一个文件、判完拷回原件：编译目录跨条复用，每条只付增量编译
  tree.source      版本指针：这棵树导出自哪个提交、它这一版的指纹、这一次换树开工的时刻（synced_at_ns）。tree.source.pending 在，就是上一次 --sync-tree 没做完
  target/          CARGO_TARGET_DIR；scratch/ 是每一步的 TMPDIR（每一步开跑前清空：被停掉的测试留下的镜像不攒着）
  inflight.json、inflight.original   在途记录：改文件之前先存原件、记下改的是哪一条；开跑时它在就先把原件拷回（临时件 inflight.restore 在库目录里，不在树里）
  versions/<指纹>/ 这一版的 manifest.tsv（树里逐文件的 sha256）、toolchain.txt（rustc -Vv）、list.json（清单）、results.tsv（结果表，只追加，每行带校验列）、
                   baseline.tsv（基线，只追加，按线程数分开记）、imports.tsv（导入记录）、logs/（没过的那一步的输出）
  指纹：源码指纹 = 树里导出的全部文件（crates/、Cargo.toml、Cargo.lock、litmus/、.cargo/）逐文件 sha256 的清单再取 sha256；
    这一版的指纹 = sha256(源码指纹、换行、rustc -Vv 的输出、「judge <判法版本 JUDGE_RULES_VERSION>」一行、「global <全局键的 sha256>」一行)：
    源码、工具链、判法或全局键（下面「结论表」那一段的 E4）变了都是另一版，旧版的清单与结果留在它自己的目录里，不当这一版的结果。
    全局键在 --sync-tree 时按那一次的参数（限时参数、--llvm-bin）现算，全文写进这一版的 global-key.txt、sha256 写进 tree.source 的 global_key。
    全局键全文一行一样，次序固定（global_key_lines，驱动与提交门禁都调它）：rules <判法版本>；toolchain <rustc -Vv 各行用 | 接>；
      file <路径> <sha256>（导出树里每份 Cargo.toml、Cargo.lock 与 .cargo/ 下的）；build <sha256> <名字>（构建环境，build_environment_of：
      research/scripts/admission.py 的 build_environment_lines 在树上算的那一份——目录层级与 CARGO_HOME 下的 cargo 配置与它们指的程序、编译变量
      RUSTFLAGS / CARGO_ENCODED_RUSTFLAGS / RUSTC_WRAPPER / CARGO_PROFILE_* / CARGO_BUILD_* / CARGO_TARGET_*_RUSTFLAGS / CARGO_TARGET_*_RUNNER 与它们指的程序、RUSTC；
      驱动另加文档测试那一步才读的 DOC_TEST_VARIABLES（admission 那一份不管文档测试），名字写「<文档测试：环境变量 名>」）；
      tool <nextest / cargo-mutants / llvm-profdata 的版本>；exclusions <排除表的 sha256>；limits <限时参数>；escalation <升档常量>（见下「升档」）。
      --timeout-floor、--timeout-multiplier、--memory-cap、--threads 不进全局键：加大它们重跑不退 7。
    工具链与这一版记下的（toolchain.txt）不同、或真跑这一趟现算的全局键与 tree.source 记的不同（排除表、工具版本、构建环境、限时参数、升档常量改了），真跑退 7：
    在改后的全局键下照样 --sync-tree（同一个提交、同样的限时参数）就是另一版，整版重判。
    判法版本进指纹之前建的版本目录（名字是源码指纹、或 sha256(源码指纹、换行、工具链)，例 versions/19c45e02…）与全局键进指纹之前建的（tree.source 里没有 global_key）
    不再认：真跑退 7、提交门禁判红，同一个提交重做 --sync-tree 是另一版；全局键全文的写法没变，结论表里键相同的结论照沿用。
    提交流程（.claude/skills/crash-test/commit-stages.md 第 2b 步）② 先 --sync-tree：这样的旧版本在 ② 就换成新的一版（新版本目录、整版现判），③ 不落在旧版上退 7。
  导出范围取门禁 checker-tier-crates-mutation-replay 登记的拷贝范围（research/scripts/crates-mutation-rows.py 的 copied_paths，去掉 research/ 下的）加 .cargo/。

一条一条跑：每条变异一次独立的判定，判完立刻往 results.tsv 追加一行并 fsync（驱动自己不经 eatmydata，只有测试命令经它）。一行十二段，制表符分隔：
  序号、包、变异名、结局、哪一段判的（one / two）、抓到它的第一条测试名、编译秒数、测试秒数、时刻、这一版的指纹、备注、校验列
结果行的校验列（用户 2026-10-03 定：库目录同一个用户手写得了，结果行带密钥算的校验列、门禁逐行核）：
  校验列 = HMAC-SHA256(密钥, 前十一段照写进表里的样子用制表符接起来、UTF-8)，小写十六进制（result_mac）。驱动写的每一行都带（现判的、沿用的、导入的）。
  密钥放库目录之外、仓之外：${XDG_CONFIG_HOME:-~/.config}/singlefs/broad-mutation-result.key（DEFAULT_RESULT_KEY），一行 64 位小写十六进制（32 字节随机数），
    属主是自己、权限 600，所在目录 700。没有时，往结果表里写行的用法（真跑、--import-results）开跑前排他新建一份并说一句；只读的（--summary、--list-items）不建。
    密钥落在库目录里、权限不是 600、内容写坏，写行的用法拒（退 2）；只读的照「校验不了」办。
  读结果表时十一段的行（没有校验列）、校验列对不上的行、没有密钥时的每一行都不算定论（那几条照没有定论选、续跑时重跑），
    --summary 把它们报成对不上、退 1（结局照「--summary」一段）；提交门禁 gate-verdict 格另用同一把密钥逐行核这一版的结果表，一行不过就红。
  导入另一台的行（--import-results）：那一台的校验列是它自己的密钥算的，核不了，去掉后用本机的密钥重签；指纹、清单、变异名照旧逐行核。
  管不到的：结论表（conclusions.tsv）与升档登记表（escalated-verdicts.tsv）的行不带校验列——沿用的结论按结论表落行时由驱动签上，手写结论表的行能被沿用；
    升档登记行的证据靠日志的 sha256 与驱动、包装那两句，门禁另核结果表里那一条的行。
  整库重建（有校验列之前建的库目录、或密钥丢了换了：旧行一行都校验不过）：把库目录挪开留着、新建一个空的（编译目录 target/ 可以挪过去接着用），
    照提交流程 ② 重做 --sync-tree、③ 整版现判；判法版本 JUDGE_RULES_VERSION 随校验列一起改过，旧版的结论一条都不沿用。
  结局：caught 抓到、missed 没红（两段都绿）、unviable 无效（编译那一步编不过）、timeout 超时、memory-cap 内存撞顶、stage-one-only 只过了第一段、
        driver-error 驱动错（某一步退了认不出的码，不是这条变异的结局）。
  一条的状态不取最后一行：有 caught / missed / unviable 行的取其中最后一行；没有而有 timeout / memory-cap 行的取其中最后一行（环境型的结局不盖掉前面的定论）；
  都没有而有 stage-one-only 行的是 stage-one-only（后来的 stage-one-only 行不把定论降级）；只有 driver-error 行的是 driver-error，续跑时重试；一行都没有是 none。
  末行没写完（没有换行结尾）不算数，下一次追加之前截掉并说一句；中间段数不对的行、结果行的变异名与清单同序号的变异名不同、清单里没有的序号，
  --summary 判红退 1，续跑开跑前核后两样、对不上退 1 一条不跑。
两段式：第一段每条只跑 harness 档四个包的测试（eatmydata + cargo nextest，遇第一个失败即停）；第一段没红的先落一行 stage-one-only，再进第二段。
  第二段只补第一段没跑到的（用户 2026-09-30 定）：被变异的两个包的文档测试（cargo test --doc）、harness 档四个包里标了 #[ignore] 的慢档用例
  （与第一段同一种起法，eatmydata + cargo nextest 加 --run-ignored only，再用 -E 排掉排除表里的，步名 two-test-slow-harness）、checker 档快档（cargo test --release -p singlefs-checker-tier --lib --tests）；
  都没红才记 missed。checker 档标了 #[ignore] 的全量崩溃枚举不在这里（另列第三段，驱动还没做）。第二段被打断的条，续跑时只补第二段。
忽略用例登记：harness 档四个包（crates/<包>/src 与 tests 下）里每一条 #[ignore] 用例只有两种归宿，要跑第二段的这一趟开跑前（基线之前）按这一版树里的源码逐条判：
  理由以「harness 慢档用例：」开头的，进第二段慢档用例那一步；按环境变量起的大档与工具（没给环境变量就红、或规模由环境变量定）写在排除表
  research/scripts/broad-mutation-ignored-exclusions.tsv，一行一条：<二进制 id><空格><用例名><TAB><理由>，二进制 id 与用例名照 cargo nextest list 的写法，
  第二段那一步用 -E 'not ((binary_id(=…) & test(=…)) | …)' 把它们排掉。下面每一种判红退 1、一步不跑：
  新加的忽略用例两样都不是（没登记）、排除表指着树里没有的用例、排除表里的用例理由又写着「harness 慢档用例：」（两边都在）、排除表有写坏（不是两段、名字里有不认的字符、理由是空）或重复的行、
  认不出写法的 #[ignore]（多行的、带条件的）。只跑第一段的趟（--skip-stage-two）不判。--check-ignored 只做这一判、打第二段那一步的完整命令。
  第二段那一步的步名是 two-test-slow-harness：旧步名 two-test-ignored 是没排除大档的那一版命令（一条没跑完就被停了、没有绿记录），基线表里旧名的行不认，这一步的基线在这一版上重量；
  排除表改了之后命令随之变、基线的秒数（只用来定这一步的超时）不重量，超时地板与倍数照旧。
中断续接：任何时刻被 TERM / KILL，重跑同一条命令：有定论的条一条都不重跑（带 --item 的一样）；被打断的那一条没有新行、只重跑它（或只补它的第二段）；
  树里留着被改过的源文件时先照在途记录拷回原件；拷不回、树与 manifest 对不上又没有在途记录、树里多出没登记的文件，拒绝跑（退 5），出路是 --sync-tree 重建那棵树。
  被 TERM / INT 停：在途那一步连同内存包装给它开的 scope 一起停（systemctl --user stop 自己那一个 scope），不留着它跑完。
  被 KILL：留下的那一步还活着时占着库目录的锁（锁的描述符在驱动与内存包装最外层那个 bash 手里）：等 --lock-wait 秒，还占着就退 4、不跑。
基线：每一版每一步、每个线程数（--threads）先在没改过的树上跑一遍，全绿才开始跑变异（基线红了一律退 6）；基线的秒数定这一版这个线程数下的超时。
  baseline.tsv 只追加，一行六段：步名、结局（ok / exit-<码>）、秒数、时刻、线程数、这一步的内存峰值字节（包装记的，见「内存」）。
  取基线按（步名, 线程数）取最后一行：是绿的、逐条时间表与峰值都在就沿用，不然在这个线程数下重跑那一步、追加一行；别的线程数的行不动、不盖。
  每一趟真跑在基线之前先核编译目录里没有换树之前编的工作区产物：树里文件的 mtime、target/ 与 target-coverage/ 里工作区各包的 fingerprint
  都不早于版本指针的 synced_at_ns（没有这一项的树同样拒），核不过退 5，出路是重做 --sync-tree。
  每次重起（这一趟没现跑基线的那一段）先在没改过的树上把编译那几步各跑一遍，红了退 6：基线绿过之后环境坏了要在落行之前发现。
  编译那一步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类），不记无效，记 driver-error 并当场在没改过的树上验一次编译。
内存：每一步命令各自经 research/scripts/run-with-memory-cap.sh 跑（与 mutate.sh、门禁 checker-tier-crates-mutation-replay 同一种套法），
  驱动自己不在 scope 里：整条包一层的话一条无界分配的变异会连驱动一起杀掉，续跑又撞同一条、永远过不去。
  撞这一步自己的上限（退 250）：systemd 只停这一步的 scope，记 memory-cap（再照「升档」重跑一次）。
  撞 slice 的总上限（几条合起来超了）：内核在整个 slice 里挑一条杀，挑中的可能是 slice 里别人那一条（别的会话的重活）。所以：
    改坏了源码的那几步带 RUN_WITH_MEMORY_CAP_OOM_FIRST（MUTATION_STEP_OOM_REASON），包装把它的 oom_score_adj 写成 1000，撞总上限时内核先杀它；
    升档那趟带 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP（ESCALATED_QUEUE_REASON），包装按它自己的上限排队（不按峰值表里的小峰值），放不下就等、等满或按上限算
    比 slice 总上限还大就退 252，不放进去再撞总上限；撞过顶的键包装也按上限排队（research/scripts/run-with-memory-cap.sh 文件头「排队」）。
    基线与验环境那几步两样都不带。变异步被总上限挤掉（退 254）是包装自己的结局，照「退出码」3 办；重跑时那个键已记成撞过顶、按上限排队。
  超时用包装的 RUN_WITH_MEMORY_CAP_TIME_LIMIT（退 253）。
  基线那几步另把 RUN_WITH_MEMORY_CAP_PEAKS 指到这一版的 baseline-peaks.<线程数>.tsv、键 broad-mutation baseline <步名>（BASELINE_PEAK_KEY_PREFIX；格式是包装的峰值表）：
  每一步开跑前删掉它，跑完读出包装记的峰值、写进 baseline.tsv 那一行的峰值一段，升档的基线峰值取那一行（包装改写峰值表不动已经记下的行）。
  基线那一步记过绿而这个线程数下那一行没有峰值的，真跑重跑那一步的基线。
升档（用户 2026-10-03 定：超时与内存撞顶不交用户判，由驱动凭证据自动判）：
  一条变异某一步退 253（超时）或 250（撞了内存上限），驱动不换参数、当场在同一棵改过的树上把那一步升档重跑一次（escalate_step）：
    超时：限时 = max(ESCALATION_TIME_FACTOR 倍原限时, ESCALATION_TIME_BASELINE_MULTIPLE 倍基线秒数) 向上取整，内存上限不变；
          没有整步限时的步（经 runner 按单条限 CPU 的测试步，time_limit_of 交 None）不升档，照旧记超时。
    撞顶：内存上限 = max(ESCALATION_MEMORY_FACTOR 倍原上限, ESCALATION_MEMORY_BASELINE_MULTIPLE 倍基线峰值) 按 MiB 向上取整（写成 <MiB>M），限时不变。
  升档那趟跑完（退 0）接着跑这一段后面的步，结局照升档那趟往后判（测试没过记抓到、都过了记没红，与不升档的判法同一份）；
  升档那趟另退一种结局（测试没过、编不过、认不出的码）照那一种判；升档那趟是内存包装自己的结局（251、252、254）不停整趟，照原结局记、备注写没跑成。
  两趟都超时（都退 253）或都撞顶（都退 250），而证据齐，判抓到：结果行结局 caught、「抓到它的测试」一段写标签 caught-by-timeout / caught-by-memory，
    登记表 escalated-verdicts.tsv 追加一行；证据不齐的不写登记表，结果行照旧记 timeout / memory-cap（没结论，备注写缺哪一样）。证据（escalation_evidence_problems 判）：
    ① 同一版、同一步、同一个线程数在没改过的树上的基线：baseline.tsv 里这个线程数下那一步最后一行绿记录，超时取它的秒数，撞顶取它的峰值字节；
       登记行记那一行的行号（baseline_row）与取到的值，之后换线程数追加的行不动它；
    ② 两趟都退同一种（都 253 或都 250），两趟的日志都在、各带包装打的那一句（「超过限时 <限时> 秒」「撞了内存上限 <上限>」）；
    ③ 升档那趟的限时或上限比原来的大，且不低于基线的 K 倍：超时 K = ESCALATION_TIME_BASELINE_MULTIPLE（20），撞顶 K = ESCALATION_MEMORY_BASELINE_MULTIPLE（2）；
    ④ 两趟日志挪进这一版的 escalation/<序号>.<步名>.<一次升档的号>.{first,escalated}.log，末尾由驱动追加一行
       BROAD_MUTATION_EVIDENCE version=<这一版的指纹> sequence=<序号> step=<步名> attempt=<first / escalated> limit=<那一趟的限时或上限> exit=<退出码>，
       登记行记两份的路径（相对这一版的目录）与 sha256；⑤ 登记行的 version 是这一版的指纹（那条变异的版本键）。
  K 为什么够：超时那一边，默认参数下 K 这一道恒满足、不咬：原限时 = max(--timeout-floor 900 秒, 5 倍基线)，升档那趟 = max(4 倍原限时, 20 倍基线) 天然不低于 20 倍基线。
    挡住「机器慢」被判成抓到的是两个绝对数：两趟里头一趟已经过了 900 秒、升档那趟过了不少于 3600 秒；真库的基线是几秒到几十秒，900 秒是它们的几十到几百倍
    （SMT 兄弟被占量过的是 2 倍，「限时（C4）」那一段的 --cpu-multiplier；别的重活排队拖慢的倍数没量过）。K 只在调小 --timeout-floor 或 --timeout-multiplier 时起作用。
    测试步那一边「超 3 倍基线 CPU 加 30 秒算抓到」（用户定「卡住算抓到」）比它松得多。
    撞顶那一边，包装记的峰值含页缓存（cargo 写编译产物的那些，撞上限之前先回收），基线峰值偏大不偏小；两趟都在不低于它 2 倍的上限里被 OOM 杀，
    是这条变异自己要的回收不掉的内存比整步没改过时的总占用还多一倍，不是调度抖动。
  登记表 <库目录>/escalated-verdicts.tsv：只追加、跨版本，一行一条升档判的抓到，列见 ESCALATION_COLUMNS（version、sequence、name、label、step、baseline、baseline_row、
    threads、first_limit、escalated_limit、required_multiple、first_log、first_log_sha256、escalated_log、escalated_log_sha256、finished_at）。只由驱动写，手写的行门禁判红。
    按版本键只认这一版的行：--summary 对这一版里状态是升档判的抓到的条逐条打 ESCALATED <TAB> 序号 <TAB> 标签 <TAB> 变异名，判定行带 escalated=<条数>；
    提交门禁 gate-verdict 格逐行复核这一版的登记行（倍数门槛按此刻驱动里的常量、日志在且 sha256 对、带驱动与包装打的那两句、版本键对；基线按登记行的 baseline_row
    找这一版 baseline.tsv 的那一行：步名、绿、线程数都与登记的相同，秒数或峰值与登记的相同），每一条 ESCALATED 都要有一行复核过的这一版登记行，一条不过就红。
  与结论表、全局键的关系：升档判的抓到不进结论表（conclusions.tsv）、不跨版本沿用，每一版现判；升档的四个常量写成全局键的 escalation 一行，改了是另一版。
前缀：起 cargo 的与改库目录的（跑、--sync-tree、--import-results）都要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，不带退 2；
  --list-items、--summary、--check-ignored、--help（-h）、--selftest 不要。重型测试闸同样这样分（.claude/hooks/lib_heavy_tests.py 文件头「广谱变异的逐条驱动」）。

剪枝（三轮设计判决 research/prompts/tooling-broad-mutation-prune-r{1,2,3}-main-verification.md：C1–C6 加 E1–E6；规则在 research/scripts/mutation-prune.py）：
  --changed-since <提交>  只现判「这次改到的」（C1：对着那个提交，被改的 fn 项、函数外部分变了的文件里全部、新加的函数、宏与 include 依赖变了的文件），
                          加结论表里沿用不了的（没有记录、全局键变了、无效、没红、抓到它的测试的 fn 项指纹变了）；其余落 carried 行沿用（C3）。
                          要现判的条里这一版已经是超时、内存撞顶的（环境型结局，提交门禁算没结论）也重跑一遍（用户 2026-10-02 定：照旧算没结论、门禁红，出路要走得通）；
                          重跑时超时、撞顶的那一步照「升档」自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到；证据不齐、仍是这两种的，
                          加大 --timeout-floor 或 --memory-cap（都不进全局键）再 --item <序号> --rerun 点名重跑（重跑里照样升档）。
                          没红的条里确是等价变异（没有哪条测试杀得了它）的，可写进 .cargo/mutants.toml 的 exclude_re（cargo-mutants 生成清单时就不列它；
                          .cargo/ 进全局键与源码指纹，整版重判），理由写在那一行旁边。
                          与 --list-items 同给是只读的：逐条打「现判 / 沿用」与理由，库目录一个字节不写。
  --milestone-full        里程碑末全量（C5）：开跑时把会沿用的结论记进 milestone-carry.tsv，这一版每条都现判，翻面的逐条打 FLIPPED。
  --summary               另打 BROAD_MUTATION_GATE（C6：每条都有结论、没有没红才 green），NO_CONCLUSION / FLIPPED 逐条列；判红退 8。
  版本内按覆盖挑测试（C2）：每版在没改过的树上插桩跑一遍（--coverage-step，驱动自己起、经内存包装；第一段逐条测试、第二段逐个二进制，E6），
    每条变异先编，再只跑覆盖到它所在 fn 项的测试（-E 按块切，避开单参数 131072 字节上限），遇第一个失败即停；零覆盖记没红（备注写零覆盖）。
    例外一律跑全部（E1）：不在 fn 项里的变异、常量上下文里用到的 const fn、起子进程的测试二进制、文档测试（第二段每条都跑）。
    覆盖要 llvm-profdata、llvm-cov：取 rustup 组件 llvm-tools（rustup component add llvm-tools；与编译器同一份 LLVM，profraw 格式对得上，装一次不再联网），
    装在别处给 --llvm-bin <目录>；它的版本进全局键。
  限时（C4）：测试步经 runner（库目录 test-runner.py，CARGO_TARGET_<三元组>_RUNNER）逐条测试（checker 档逐个二进制）按 CPU 时间限：
    --cpu-multiplier（默认 3，不许小于 2：盖住 SMT 兄弟被占的 2.0 倍）× 基线 CPU 秒 + --cpu-floor（默认 30），墙钟兜底 --wall-multiplier（10）× 基线墙钟 + --wall-floor（300）；
    超了算那条测试没过 = 抓到（用户定「卡住算抓到」），备注写哪一种停的；测试步不再设整步总时限（编译步与文档测试步照旧按 --timeout-floor / --timeout-multiplier）。
    基线按线程数各量一份（test-times.<线程数>.<步名>.tsv），checker 档基线带 --no-fail-fast。
  结论表：库目录 conclusions.tsv（只追加、跨版本，同一个键取最后一行），键 =（文件、限定名、函数内相对起止、替换文）（E5），
    另存结局、抓到它的测试与那条测试 fn 项的指纹（E3）、fn 项与函数外部分的词法单元指纹（E2）、宏与 include 依赖的指纹、全局键（E4：工具链、
    Cargo.toml、Cargo.lock、.cargo/、构建环境、nextest / cargo-mutants / llvm-tools 版本、排除表、限时参数、升档常量、判法版本 JUDGE_RULES_VERSION；全文在这一版的 global-key.txt，
    sha256 进这一版的指纹，见上「指纹」）。
    判法版本此刻是 prune-r3-e1-e6-fresh-build（换树后编译目录不留过时的工作区产物）：它之前落的结论（结论表里记着旧全局键的、旧版目录里的结果行）一条都不沿用，全部重判。
  旧表：加剪枝之前的版本目录（例 versions/19c45e02…，十一段的 results.tsv）照旧留着，但那时没存函数键与测试指纹，结论表里没有它们的行，
    判法版本又进了指纹（换树之后版本指针指不回它），一条都不沿用，只作参考。
  第二段基线红报成退 6 的拒绝，列红的测试与出路（修好的提交再 --sync-tree；在那之前只要第一段的结论就加 --skip-stage-two）。

输出：每条一行 BROAD_MUTATION_ITEM sequence=… outcome=… stage=… build_seconds=… test_seconds=… name=…；
      末尾一行 BROAD_MUTATION_TOTAL selected=… ran=… 各结局条数；--summary 打 BROAD_MUTATION_SUMMARY 一行与 MISSED、DRIVER_ERROR 明细，
      BROAD_MUTATION_GATE 一行与 NO_CONCLUSION、ESCALATED、FLIPPED 明细；升档那一步各打一行 BROAD_MUTATION_ESCALATION sequence=… step=… first=… escalated=… outcome=…；
      --import-results 打 BROAD_MUTATION_IMPORT 一行与 IMPORT_CONFLICT 明细。
      一步跑着时每过 PROGRESS_INTERVAL_SECONDS 秒（300，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」定的进度间隔）往 stderr 打一行
      BROAD_MUTATION_PROGRESS sequence=<序号，基线是 baseline，验环境是 environment> step=<步名> elapsed_seconds=… compiled_crates=… tests_passed=… tests_failed=…
      （后三个数从这一步此刻的日志里数：Compiling 行、nextest 的 PASS 行与 libtest 的 ok 行、失败行）。
退出码：0 选中的条都判完；1 --summary 栏数对不上或结果表有对不上的行（含没有校验列、校验列对不上、没有密钥校验不了的行）、续跑开跑前核出结果行与清单对不上、
  --import-results 有冲突的条、忽略用例登记对不上（含排除表写坏）、--selftest 有格判错；
  2 用法错（项写错、库目录不合、没带前缀、缺工具、只供自证的变量出现在自证之外、要写结果行而密钥用不了）；
  3 内存包装自己的结局（251 起不来、252 排不上、254 被总上限挤掉）：那一条不落行，修好重跑同一条命令；
  4 锁被占着；5 树脏了恢复不了、树里多出文件、上一次 --sync-tree 没做完、编译目录里可能留着换树之前编的工作区产物；6 基线不绿、或疑似环境坏了；
  7 工具链与这一版记下的不同、或这一趟的全局键与这一版记下的不同（含全局键进指纹之前建的版本）；
  8 --summary 判提交门禁红（这一版有没结论的条、或有没红的条；BROAD_MUTATION_GATE 行写 verdict=red）；
  143 被 TERM / INT 停（在途那一条不落行）。
弄坏开关 BROAD_MUTATION_BREAK=<项>（逗号分隔，只给 --selftest 证红用，只在 --selftest 自己起的子进程里生效）。
逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 run_selftest_cells 里那几组格的函数名，括号里是那几条「✗ 自证：」的开头；
每一项红时都另有汇总行「✗ --selftest：<格数> 格里 N 格判错」，退 1）：
  BROAD_MUTATION_BREAK=rerun-done  重起时已有定论的条照样重跑
      selftest_basic_cells（「✗ 自证：--stage-two-only 不对」、「✗ 自证：重起之后已有结果的条或基线被重跑了」、「✗ 自证：--shard 0/2 跑的不是还没结果的偶数序号」）、
      selftest_state_cells（「✗ 自证：点名已有定论的条没跳过」、「✗ 自证：点名加 --rerun 没有重跑并追加新行」、「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的超时行盖掉了」、「✗ 自证：--stage-two-only 点到没过过第一段的条没拒」）、
      selftest_table_cells（「✗ 自证：认不出的退出码没处理对」）、selftest_import_cells（「✗ 自证：两台各一片的结果没合对」）
  BROAD_MUTATION_BREAK=interrupted-as-done  被打断的那一条重起时直接记成抓到不重跑
      selftest_basic_cells（「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」）
  BROAD_MUTATION_BREAK=no-row-flush  结果行攒在内存里到收尾才写
      selftest_basic_cells（「✗ 自证：杀驱动之前，前两条的结果行没有都落盘」、「✗ 自证：重起之后已有结果的条或基线被重跑了」）
  BROAD_MUTATION_BREAK=stage-two-missed-as-caught  第二段没红记成抓到
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」、「✗ 自证：--changed-since 没有只现判改到的与沿用不了的」、「✗ 自证：里程碑全量没列出翻面的条」）、
      selftest_basic_cells（「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」）、selftest_ignored_cells（「✗ 自证：旧步名的基线行被认了」）
  BROAD_MUTATION_BREAK=summary-mismatch-silent  汇总栏数对不上不报
      selftest_basic_cells（「✗ 自证：结果表里有认不出的结局时汇总没判红」）、selftest_table_cells（「✗ 自证：结果行的变异名与清单同序号的不同时没判红」）、
      selftest_result_mac_cells（「✗ 自证：手追加的没有校验列、校验列对不上的 caught 行被当成了结论」、「✗ 自证：密钥不在时 --summary 没判红、或替它建了一份」）
  BROAD_MUTATION_BREAK=no-restore  开跑时不恢复脏树也不核树
      selftest_basic_cells（「✗ 自证：重起之后已有结果的条或基线被重跑了」、「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」、「✗ 自证：树脏了又恢复不了时没拒」）、
      selftest_table_cells（「✗ 自证：末行没写完的结果表没处理对」）、selftest_tree_cells（「✗ 自证：核树没看多出来的文件」、「✗ 自证：恢复原件半途被杀之后树里多出了文件、或重跑没恢复」）
  BROAD_MUTATION_BREAK=accept-unknown-item  不认得的项悄悄略过
      selftest_basic_cells（「✗ 自证：不认得的项没拒」）
  BROAD_MUTATION_BREAK=test-variables-anywhere  只供自证的变量隔一层进程、库目录在自证工作目录之外也认
      selftest_test_variable_cells（「✗ 自证：只供自证的变量指着自证工作目录之外的库时没拒」、「✗ 自证：父进程不是 --selftest 的进程带着假一步的变量时没拒」、「✗ 自证：父进程不是 --selftest 的进程只带着弄坏开关时没拒」）
  BROAD_MUTATION_BREAK=sync-pointer-without-pending  换树不写版本指针的临时件
      selftest_tree_cells（「✗ 自证：--sync-tree 换树半途被杀之后没拦住」）
  BROAD_MUTATION_BREAK=state-is-last-row  一条的状态照旧取最后一行
      selftest_state_cells（「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」、「✗ 自证：抓到的条被后来的超时行盖掉了」）
  BROAD_MUTATION_BREAK=named-items-always-rerun  点名的条已有定论也重跑
      selftest_state_cells（「✗ 自证：点名已有定论的条没跳过」、「✗ 自证：点名加 --rerun 没有重跑并追加新行」、「✗ 自证：抓到的条被后来的超时行盖掉了」）
  BROAD_MUTATION_BREAK=names-unchecked  不核结果行的变异名
      selftest_table_cells（「✗ 自证：结果行的变异名与清单同序号的不同时没判红」）
  BROAD_MUTATION_BREAK=partial-row-kept  末行不完整时照旧补一个换行留着它
      selftest_table_cells（「✗ 自证：末行没写完的结果表没处理对」）
  BROAD_MUTATION_BREAK=no-environment-check  重起不验编译、连着无效或秒红也不查
      selftest_table_cells（「✗ 自证：认不出的退出码没处理对」）、
      selftest_environment_cells（「✗ 自证：基线绿过之后编译坏了，重起没发现」、「✗ 自证：连续无效而原件上编译照过时没有接着跑」、「✗ 自证：连续 3 条秒红的抓到、原件上那一步也红，没有停下报疑似环境坏了」）
  BROAD_MUTATION_BREAK=unviable-by-exit-code  编译步退 101 一律记无效
      selftest_table_cells（「✗ 自证：认不出的退出码没处理对」）
  BROAD_MUTATION_BREAK=unknown-exit-stops  一步退认不出的码照旧整趟停
      selftest_table_cells（「✗ 自证：认不出的退出码没处理对」）
  BROAD_MUTATION_BREAK=stage-two-without-stage-one  点名加 --stage-two-only 不看过没过第一段
      selftest_state_cells（「✗ 自证：--stage-two-only 点到没过过第一段的条没拒」）、selftest_test_variable_cells（「✗ 自证：父进程不是 --selftest 的进程带着假一步的变量时没拒」）
  BROAD_MUTATION_BREAK=stage-two-full-cargo-test  第二段照旧是整遍 cargo test 加快档
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」）、
      selftest_basic_cells（「✗ 自证：--stage-two-only 不对」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」）、
      selftest_state_cells（「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」）、
      selftest_ignored_cells（「✗ 自证：登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对」、「✗ 自证：旧步名的基线行被认了」）
  BROAD_MUTATION_BREAK=term-leaves-step-running  被 TERM 时只给包装发一次 TERM
      selftest_stop_cells（「✗ 自证：被 TERM 停时在途那一步没有一起停」）
  BROAD_MUTATION_BREAK=extra-files-unchecked  核树不看多出来的文件
      selftest_tree_cells（「✗ 自证：核树没看多出来的文件」、「✗ 自证：恢复原件半途被杀之后树里多出了文件、或重跑没恢复」）
  BROAD_MUTATION_BREAK=toolchain-ignored  指纹不含工具链也不核
      selftest_tree_cells（「✗ 自证：工具链没进指纹」、「✗ 自证：排除表改了之后这一版的旧结论照算」：指纹退回源码指纹，全局键也一起没进）、
      selftest_fresh_build_cells（「✗ 自证：判法版本改了旧结论还被沿用」：指纹退回源码指纹，判法版本也一起没进）
  BROAD_MUTATION_BREAK=import-unchecked  导入不核变异名与指纹
      selftest_import_cells（「✗ 自证：该拒的导入没拒」）
  BROAD_MUTATION_BREAK=unregistered-ignored-passes  没登记的忽略用例判绿
      selftest_ignored_cells（「✗ 自证：新加的没登记的忽略用例没判红」）
  BROAD_MUTATION_BREAK=stale-exclusion-passes  排除表指着不存在的用例或两边都在判绿
      selftest_ignored_cells（「✗ 自证：排除表指着不存在的用例、或两边都在，没判红」）
  BROAD_MUTATION_BREAK=malformed-exclusion-row-skipped  排除表写坏或重复的行悄悄略过
      selftest_ignored_cells（「✗ 自证：排除表写坏的行或重复的行没判红」）
  BROAD_MUTATION_BREAK=exclusion-filter-dropped  第二段慢档用例那一步的命令不带排除的 -E
      selftest_basic_cells（「✗ 自证：第二段的步不对」）、selftest_ignored_cells（「✗ 自证：登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对」）
  BROAD_MUTATION_BREAK=ignored-scan-skips-nested-modules  扫源码时模块里的 #[ignore] 略过
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」、「✗ 自证：--changed-since 没有只现判改到的与沿用不了的」、「✗ 自证：里程碑全量没列出翻面的条」、「✗ 自证：第二段基线红没报成带出路的拒绝」）、
      selftest_basic_cells（「✗ 自证：--stage-two-only 不对」、「✗ 自证：杀驱动之前，前两条的结果行没有都落盘」、「✗ 自证：驱动被杀之后树里没有留着改过的文件与在途记录」、「✗ 自证：上一趟留下的进程还占着锁时没拒」、「✗ 自证：重起之后已有结果的条或基线被重跑了」、「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」）；
      之后 selftest_basic_cells 抛 FileNotFoundError（hanging），整趟退 1、不打「N 格判错」汇总行
  BROAD_MUTATION_BREAK=slow-step-keeps-old-name  第二段慢档用例那一步还叫旧步名 two-test-ignored（旧基线记录被认）
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」）、
      selftest_basic_cells（「✗ 自证：--stage-two-only 不对」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」）、
      selftest_state_cells（「✗ 自证：第二段的定论被后来的 stage-one-only 行降级了」）、
      selftest_ignored_cells（「✗ 自证：登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对」、「✗ 自证：旧步名的基线行被认了」）
  BROAD_MUTATION_BREAK=progress-never  一步跑着时进度间隔的判断永不触发、一行 BROAD_MUTATION_PROGRESS 都不打
      selftest_progress_cells（「✗ 自证：一步跑过进度间隔没有报进度」）
  BROAD_MUTATION_BREAK=coverage-selection-ignored  剪枝：不按覆盖挑测试
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」）
  BROAD_MUTATION_BREAK=stage-two-not-by-binary  剪枝：第二段不按二进制挑
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」）
  BROAD_MUTATION_BREAK=doc-tests-skipped  剪枝：文档测试不跑
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」）、selftest_basic_cells（「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」、「✗ 自证：第二段的步不对」）
  BROAD_MUTATION_BREAK=whole-step-time-limit  剪枝：测试步照旧设整步总时限
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」）、
      测试步退 253 照「升档」升档重跑、两趟都超时就判成抓到，于是还有 selftest_environmental_cells（「✗ 自证：--changed-since 没有重跑超时、内存撞顶的条」）、
      selftest_basic_cells（「✗ 自证：被打断的第 3 条没有重跑、或被当成已完成」、「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」）、
      selftest_state_cells（「✗ 自证：抓到的条被后来的超时行盖掉了」）、selftest_escalation_cells（「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」：
      测试步的整步时限不再是 None）、selftest_import_cells（「✗ 自证：两台各一片的结果没合对」）
  BROAD_MUTATION_BREAK=cpu-limit-ignored  剪枝：测试步不经 runner 限 CPU
      selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」）
  BROAD_MUTATION_BREAK=carry-without-store-check  剪枝：结论表里有键就沿用（无效、没红也沿用）
      selftest_prune_cells（「✗ 自证：--changed-since 没有只现判改到的与沿用不了的」）、selftest_fresh_build_cells（「✗ 自证：判法版本改了旧结论还被沿用」）
  BROAD_MUTATION_BREAK=milestone-flips-unlisted  剪枝：里程碑全量不列翻面的
      selftest_prune_cells（「✗ 自证：里程碑全量没列出翻面的条」）
  BROAD_MUTATION_BREAK=summary-gate-silent  剪枝：--summary 有没红或没结论也退 0
      selftest_prune_cells（「✗ 自证：--changed-since 没有只现判改到的与沿用不了的」、「✗ 自证：里程碑全量没列出翻面的条」）、selftest_basic_cells（「✗ 自证：汇总不对」）、
      selftest_table_cells（「✗ 自证：末行没写完的结果表没处理对」、「✗ 自证：认不出的退出码没处理对」）、selftest_import_cells（「✗ 自证：两台各一片的结果没合对」）、
      selftest_environmental_cells（「✗ 自证：--changed-since 没有重跑超时、内存撞顶的条」：没有判红那一句出路）
  BROAD_MUTATION_BREAK=stage-two-baseline-red-generic  剪枝：第二段基线红只报一句不给出路
      selftest_prune_cells（「✗ 自证：第二段基线红没报成带出路的拒绝」）
  BROAD_MUTATION_BREAK=missed-carried  mutation-prune.py 的规则库：没红的条也沿用、不重判（那份文件头 E4）
      selftest_prune_cells（「✗ 自证：--changed-since 没有只现判改到的与沿用不了的」）
  BROAD_MUTATION_BREAK=unviable-carried  mutation-prune.py 的规则库：无效的条也沿用、不重判（那份文件头 E4）
      selftest_prune_cells（「✗ 自证：--changed-since 没有只现判改到的与沿用不了的」）
  BROAD_MUTATION_BREAK=global-key-ignored  mutation-prune.py 的规则库：全局键变了也沿用（那份文件头 E4）
      selftest_fresh_build_cells（「✗ 自证：判法版本改了旧结论还被沿用」）
  BROAD_MUTATION_BREAK=sync-keeps-stale-artifacts  换树：tar -x 不带 -m（还原提交时刻）、换树后不清编译目录里工作区各包的 fingerprint
      selftest_fresh_build_cells（「✗ 自证：换树之后编译目录里留着过时的工作区产物」、「✗ 自证：基线之前没拦住编译目录里的旧产物」、「✗ 自证：判法版本改了旧结论还被沿用」：
      后两格跟着红是因为换出来的树都被基线之前那一核拒）、selftest_prune_cells（「✗ 自证：按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对」、
      「✗ 自证：--changed-since 没有只现判改到的与沿用不了的」、「✗ 自证：里程碑全量没列出翻面的条」、「✗ 自证：每条都抓到的一版提交门禁没判绿」、「✗ 自证：第二段基线红没报成带出路的拒绝」）；
      之后 selftest_prune_cells 末尾抛 FileNotFoundError（selections.log），整趟退 1、不打「N 格判错」汇总行
  BROAD_MUTATION_BREAK=stale-artifact-check-skipped  基线之前不核编译目录里换树之前编的工作区产物
      selftest_fresh_build_cells（「✗ 自证：基线之前没拦住编译目录里的旧产物」）
  BROAD_MUTATION_BREAK=stale-judge-version  判法版本照旧（prune-r3-e1-e6）、不进版本指纹、名字是源码指纹的旧版目录照认
      selftest_fresh_build_cells（「✗ 自证：判法版本改了旧结论还被沿用」）、selftest_tree_cells（「✗ 自证：工具链没进指纹」、「✗ 自证：排除表改了之后这一版的旧结论照算」：
      照旧驱动算指纹、真跑不核全局键），连同 global-key-outside-version 那一项列的 selftest_global_key_class_cells 十五格与 selftest_unit_environment_cells 一格
  BROAD_MUTATION_BREAK=global-key-outside-version  全局键不进版本指纹、真跑也不核全局键
      selftest_tree_cells（「✗ 自证：工具链没进指纹」、「✗ 自证：排除表改了之后这一版的旧结论照算」）、
      selftest_global_key_class_cells（「✗ 自证：--sync-tree 之后换了工具版本 / 限时参数 / 构建环境…，真跑没退 7」与另七类那几格，共十五格：真跑不核全局键）、
      selftest_unit_environment_cells（「✗ 自证：--unit-environment 打的不是全局键认的那几个变量」：不带那几个变量也不退 7）
  BROAD_MUTATION_BREAK=environmental-not-reselected  --changed-since 不重跑这一版里超时、内存撞顶的条
      selftest_environmental_cells（「✗ 自证：--changed-since 没有重跑超时、内存撞顶的条」）
  BROAD_MUTATION_BREAK=global-key-no-toolchain-line  全局键不写 toolchain 那一行
      selftest_global_key_class_cells（「✗ 自证：--sync-tree 写的 global-key.txt 缺了几类行」）
  BROAD_MUTATION_BREAK=global-key-no-file-lines  全局键不写 Cargo.toml、Cargo.lock、.cargo/ 的 file 行
      selftest_global_key_class_cells（「✗ 自证：--sync-tree 写的 global-key.txt 缺了几类行」）
  BROAD_MUTATION_BREAK=global-key-no-build-lines  全局键不写构建环境的 build 行
      selftest_global_key_class_cells（「✗ 自证：--sync-tree 写的 global-key.txt 缺了几类行」、「✗ 自证：--sync-tree 之后换了构建环境，真跑没退 7」六格、
      另七类（「✗ 自证：--sync-tree 之后换了构建环境的「…」，真跑没退 7」四格、「✗ 自证：--sync-tree 之后…程序改了内容 / 配置改了，真跑没退 7」三格））、
      selftest_unit_environment_cells（「✗ 自证：--unit-environment 打的不是全局键认的那几个变量」：不带那几个变量也不退 7）
  BROAD_MUTATION_BREAK=global-key-no-tool-lines  全局键不写工具版本的 tool 行
      selftest_global_key_class_cells（「✗ 自证：--sync-tree 写的 global-key.txt 缺了几类行」、「✗ 自证：--sync-tree 之后换了工具版本，真跑没退 7」）
  BROAD_MUTATION_BREAK=global-key-no-limits-line  全局键不写限时参数那一行
      selftest_global_key_class_cells（「✗ 自证：--sync-tree 写的 global-key.txt 缺了几类行」、「✗ 自证：--sync-tree 之后换了限时参数，真跑没退 7」）
  BROAD_MUTATION_BREAK=fingerprint-path-blind  源码指纹只算各文件的内容、不算路径
      selftest_global_key_class_cells（「✗ 自证：文件改了名、内容不变，--sync-tree 还是同一版」）
  BROAD_MUTATION_BREAK=timeout-floor-in-global-key  --timeout-floor 与 --memory-cap 也写进全局键的限时那一行
      selftest_rerun_larger_limit_cells（「✗ 自证：加大 --timeout-floor、--memory-cap 再 --item --rerun 没判出结论」）、
      selftest_escalation_cells（「✗ 自证：两趟都超时、证据齐的没判成 caught-by-timeout」、「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」、
      「✗ 自证：升档那趟跑完没照它往后的结局判」、「✗ 自证：证据不齐的升档判成了抓到」、「✗ 自证：升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀」：
      那一格带 --memory-cap 1G 跑，退 7）、selftest_baseline_threads_cells（「✗ 自证：换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上」：同样带 --memory-cap 1G）、
      selftest_stop_cells（「✗ 自证：被 TERM 停时在途那一步没有一起停」：那一格另给 --timeout-floor 120，同样退 7）
  BROAD_MUTATION_BREAK=escalation-skipped  超时、撞顶不升档，照旧记没结论
      selftest_escalation_cells（「✗ 自证：两趟都超时、证据齐的没判成 caught-by-timeout」、「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」、
      「✗ 自证：升档那趟跑完没照它往后的结局判」、「✗ 自证：证据不齐的升档判成了抓到」：备注里没有升档那一句；「✗ 自证：升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀」：
      没有升档那趟）、selftest_rerun_larger_limit_cells（「✗ 自证：加大 --timeout-floor、--memory-cap 再 --item --rerun 没判出结论」：证据不齐那一趟没升档、记的是原结局）、
      selftest_baseline_threads_cells（「✗ 自证：换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上」：没有登记行）
  BROAD_MUTATION_BREAK=escalation-same-limit  升档那趟照原来的限时与上限跑
      selftest_escalation_cells（「✗ 自证：两趟都超时、证据齐的没判成 caught-by-timeout」、「✗ 自证：两趟都撞顶、证据齐的没判成 caught-by-memory」）、
      selftest_baseline_threads_cells（「✗ 自证：换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上」：没有登记行）
  BROAD_MUTATION_BREAK=escalation-evidence-unchecked  两趟都撞了什么就判抓到，不核证据（两趟结局不同、基线缺也判）
      selftest_escalation_cells（「✗ 自证：证据不齐的升档判成了抓到」）、
      selftest_rerun_larger_limit_cells（「✗ 自证：加大 --timeout-floor、--memory-cap 再 --item --rerun 没判出结论」：头一趟就判成了抓到）、
      selftest_basic_cells（「✗ 自证：两段式的结局不对」、「✗ 自证：汇总不对」：第 6 条撞顶、没有基线峰值也判成了抓到）、
      selftest_environmental_cells（「✗ 自证：--changed-since 没有重跑超时、内存撞顶的条」）、selftest_import_cells（「✗ 自证：两台各一片的结果没合对」）
  BROAD_MUTATION_BREAK=escalated-not-summarised  --summary 不列 ESCALATED、判定行不带 escalated=
      selftest_escalation_cells（「✗ 自证：--summary 没列升档判的抓到」）
  BROAD_MUTATION_BREAK=unit-environment-fixed-list  --unit-environment 只认 RUSTFLAGS、RUSTDOCFLAGS、CARGO_PROFILE_* 这几个（全局键改调构建环境之前的那一份）
      selftest_unit_environment_cells（「✗ 自证：--unit-environment 打的不是全局键认的那几个变量」、「✗ 自证：--unit-environment 没把换了工具链的 RUSTUP_TOOLCHAIN 打出来」）
  BROAD_MUTATION_BREAK=unit-environment-toolchain-ignored  --unit-environment 不比工具链（只看构建环境）
      selftest_unit_environment_cells（「✗ 自证：--unit-environment 没把换了工具链的 RUSTUP_TOOLCHAIN 打出来」）
  BROAD_MUTATION_BREAK=result-mac-unchecked  读结果表不核校验列（十一段、十二段的行都算）
      selftest_result_mac_cells（「✗ 自证：手追加的没有校验列、校验列对不上的 caught 行被当成了结论」、「✗ 自证：密钥不在时 --summary 没判红、或替它建了一份」）
  BROAD_MUTATION_BREAK=baseline-any-threads  取基线不看线程数（改前的取法：这一步最后一行，哪个线程数的都认）
      selftest_baseline_threads_cells（「✗ 自证：换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上」）
  BROAD_MUTATION_BREAK=escalation-queues-by-peak  升档那趟不带 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP（包装照峰值表排队）
      selftest_escalation_cells（「✗ 自证：升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀」）
  BROAD_MUTATION_BREAK=mutation-steps-not-oom-first  改坏了源码的那几步不带 RUN_WITH_MEMORY_CAP_OOM_FIRST
      selftest_escalation_cells（「✗ 自证：升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀」）
  升档证据的五支（escalation_evidence_problems）没有弄坏开关：selftest_escalation_evidence_cells 各一格，判红输入是治理第三轮攻方的写错版
  （research/prompts/m3-broad-gate-gov-r3-opus-model/j2d-driver-mutants.py 的 d1、d3–d6，各删一支，各让一格红）。
  mutation-prune.py 文件头列的其余那几项（驱动在自证里把它们转给规则库）打开时这份 --selftest 不红，由 mutation-prune.py --selftest 判。
只供测试（--selftest 自己设；这个进程的父进程不是带 --selftest 的这份脚本、或库目录不在那一趟自证的工作目录里，设了任何一个都拒，退 2）：
  BROAD_MUTATION_STEP_DRIVER=<脚本> 换掉每一步的真命令（调法 <脚本> <步名> <序号> <树>，不经内存包装、不要前缀）、
  BROAD_MUTATION_STEP_VIA_WRAPPER=1 假的一步也经内存包装起（证「被 TERM 时连 scope 一起停」那一格用）、
  BROAD_MUTATION_LIST_DRIVER=<脚本> 换掉 cargo mutants --list（调法 <脚本> <包>，打 JSON）、BROAD_MUTATION_TREE_SOURCE=<目录> 换掉 git archive、
  BROAD_MUTATION_TOOLCHAIN=<文字> 换掉 rustc -Vv 的输出、BROAD_MUTATION_KILL_AT=<点> 走到那一点给自己发 KILL、
  BROAD_MUTATION_ALLOW_VOLATILE_HOME=1 许库目录落在 /tmp 下、BROAD_MUTATION_EXCLUSIONS=<文件> 换掉排除表、
  BROAD_MUTATION_PROGRESS_INTERVAL=<秒> 换掉进度间隔、BROAD_MUTATION_TOOL_VERSIONS=<文字> 换掉全局键里的工具版本、
  BROAD_MUTATION_CHANGED_SINCE_SOURCE=<目录> 换掉 --changed-since 那个提交的 git archive、
  BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE=1 照 fresh-build 之前的驱动算版本指纹与全局键（判法 prune-r3-e1-e6、不进指纹、名字是源码指纹的旧版目录照认），模拟旧驱动落的结论、
  BROAD_MUTATION_RESULT_KEY=<文件> 结果行的密钥换成这一份（自证起的每个子进程都带：自证不读写真的密钥）、
  BROAD_MUTATION_RUSTC=<脚本> 工具链换成起这个假 rustc 的 -Vv（--unit-environment 比工具链那一格用）。

gate-similar: checker-tier-crates-mutation-replay.sh 它按 crates/mutations.tsv 逐条改坏、点名的测试必须红，行是人写的、判的是「这条测试抓不抓得到」；这里的清单由 cargo-mutants 生成、判每条变异有没有任何测试抓到，按条记录的形态照它，导出范围取它登记的那一份
gate-similar: mutate.sh 它跑一张手写变异表对一个实验二进制，每条 cargo test 经内存包装；这里同样逐步经内存包装，但清单是生成的、结果按源码与工具链的指纹分版本落在库目录里续接
gate-similar: crash-amplification.sh 它是崩溃放量的分项驱动，--list-items / --item 与「重跑同一条命令就是续跑」的形态照它；判的对象不同（崩溃点状态对变异）
gate-similar: mutation-shard-run.sh 它把 crates/mutations.tsv 的行分给两台、第二台的按条记录经 crates-mutation-rows.py import 核过底座指纹与行键再导入；这里两台各起一趟 --shard k/2、结果表是一张只追加的 TSV，导入（--import-results）核的是这一版的指纹、清单与逐行的变异名，两边的记录形态不同，导入不共用
"""
import argparse
import difflib
import fcntl
import hashlib
import hmac
import importlib.util
import json
import math
import os
import re
import secrets
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from typing import NamedTuple

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

SELF_PATH = os.path.join(SCRIPT_DIRECTORY, os.path.basename(__file__))
REAL_SELF_PATH = os.path.realpath(__file__)
MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "run-with-memory-cap.sh")
THREAD_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "capped.sh")
DEFAULT_HOME = "~/.local/share/singlefs/broad-mutation"
DEFAULT_MEMORY_CAP = "24G"        # 2026-09-30 派发给的上限；各步的实测峰值由内存包装记在 research/scripts/memory-peaks.tsv（键 broad-mutation <步名>）
DEFAULT_TIMEOUT_FLOOR_SECONDS = 900
DEFAULT_TIMEOUT_MULTIPLIER = 5
DEFAULT_LOCK_WAIT_SECONDS = 120
DEFAULT_STOP_GRACE_SECONDS = 10
DEFAULT_SUSPECT_STREAK = 8           # 依据见文件头 --suspect-streak
DEFAULT_SUSPECT_TEST_SECONDS = 0.2
PROGRESS_INTERVAL_SECONDS = 300      # 一步跑着时隔多久报一行进度（文件头「输出」）
LOCK_POLL_SECONDS = 2
MUTATED_PACKAGES = ("singlefs-core", "singlefs-checker")    # 清单的次序：先 core 后 checker，序号从 1 起
HARNESS_TIER_PACKAGES = ("singlefs-harness", "singlefs-checker", "singlefs-core", "singlefs-format")
CHECKER_TIER_PACKAGE = "singlefs-checker-tier"
SLOW_STEP_NAME = "two-test-slow-harness"       # 第二段 harness 慢档用例那一步；旧名 OLD_SLOW_STEP_NAME 是没排除大档的那一版命令，基线表里的旧行因此不认
OLD_SLOW_STEP_NAME = "two-test-ignored"
EXCLUSIONS_FILE = os.path.join(SCRIPT_DIRECTORY, "broad-mutation-ignored-exclusions.tsv")
TIMED_CASE_REASON_PREFIX = "harness 慢档用例："
HEAVY_PREFIX_VARIABLE = "SINGLEFS_HEAVY_TESTS"
HEAVY_PREFIX_VALUES = ("commit", "user-request")
VOLATILE_PREFIXES = ("/tmp", "/run", "/dev/shm", "/var/run")
VOLATILE_FILESYSTEMS = ("tmpfs", "ramfs")
IGNORED_DIRECTORY_NAMES = ("target", "mutants.out", "mutants.out.old")
RESULT_COLUMNS = ("sequence", "package", "name", "outcome", "stage", "first_failing_test", "build_seconds", "test_seconds",
                  "finished_at", "fingerprint", "note", "mac")
RESULT_SIGNED_COLUMNS = len(RESULT_COLUMNS) - 1    # mac 一段签的是它前面那十一段（文件头「结果行的校验列」）
DEFAULT_RESULT_KEY = "singlefs/broad-mutation-result.key"     # 相对 ${XDG_CONFIG_HOME:-~/.config}
RESULT_KEY_HEX_LENGTH = 64
BASELINE_COLUMNS = ("step", "outcome", "seconds", "finished_at", "threads", "peak_bytes")
# 结论表（库目录 conclusions.tsv，跨版本、只追加，同一个键取最后一行）：键是 json 的 [文件, 容器限定名, 函数内相对起止, 替换文]（E5）
CONCLUSION_COLUMNS = ("key", "outcome", "caught_by", "test_fingerprint", "fingerprint", "dependency", "global_key", "version", "commit", "sequence", "finished_at")
CARRIED_STAGE = "carried"           # 结果表「哪一段判的」写 carried：这一版没现判，沿用结论表里的
JUDGE_RULES_VERSION = "prune-r3-e1-e6-fresh-build-mac"   # 判法变了就改它：它进全局键与这一版的指纹，旧结论一条都不沿用（fresh-build：换树后编译目录不留过时的工作区产物；mac：结果行带密钥校验、基线按线程数分开记）
PRE_FRESH_BUILD_JUDGE_RULES_VERSION = "prune-r3-e1-e6"   # fresh-build 之前的判法版本：只供自证模拟旧驱动（BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE）与弄坏开关 stale-judge-version
SYNC_CLOCK_SLACK_NS = 2_000_000_000   # 文件时间戳取内核的粗粒度时钟，可比 time.time_ns() 慢一个节拍：核「早于换树」时放 2 秒
FINGERPRINT_ENTRY = re.compile(r"^(?P<package>.+)-[0-9a-f]{16}$")   # cargo 编译目录 .fingerprint/ 下的条目名：<包名>-<16 位十六进制>
RUNNER_TEST_STEPS = ("one-test", "two-test-slow-harness", "two-test-checker-tier")   # 经 runner 按单条（checker 档按二进制）限 CPU 时间的测试步
EXIT_GATE_RED = 8
# 升档（文件头「升档」）：四个常量写进全局键的 escalation 一行，改了是另一版；提交门禁复核登记行时按此刻的常量判倍数
ESCALATION_TIME_FACTOR = 4                    # 超时那一步升档：限时乘几倍
ESCALATION_TIME_BASELINE_MULTIPLE = 20        # 升档那趟的限时不低于基线秒数的几倍（K，依据见文件头「升档」）
ESCALATION_MEMORY_FACTOR = 1.5                # 撞顶那一步升档：内存上限乘几倍（本机 slice 总上限约 40 GiB，默认上限 24G 乘 1.5 放得下）
ESCALATION_MEMORY_BASELINE_MULTIPLE = 2       # 升档那趟的上限不低于基线峰值的几倍（K，依据见文件头「升档」）
ESCALATED_TIMEOUT_LABEL = "caught-by-timeout"
ESCALATED_MEMORY_LABEL = "caught-by-memory"
ESCALATED_LABELS = (ESCALATED_TIMEOUT_LABEL, ESCALATED_MEMORY_LABEL)
ESCALATION_REGISTRY = "escalated-verdicts.tsv"      # 库目录里的登记表，只追加、跨版本
ESCALATION_DIRECTORY = "escalation"                 # 这一版目录里放两趟日志的子目录
ESCALATION_COLUMNS = ("version", "sequence", "name", "label", "step", "baseline", "baseline_row", "threads", "first_limit", "escalated_limit", "required_multiple",
                      "first_log", "first_log_sha256", "escalated_log", "escalated_log_sha256", "finished_at")
ESCALATED_QUEUE_REASON = "广谱变异升档重跑的那一趟：按它自己的上限排队，放不下就等或退 252"   # 设进 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP
MUTATION_STEP_OOM_REASON = "广谱变异改坏了源码的那一步：撞 slice 总上限时先杀它"           # 设进 RUN_WITH_MEMORY_CAP_OOM_FIRST
ESCALATION_EVIDENCE_PREFIX = "BROAD_MUTATION_EVIDENCE"   # 驱动追加在两趟日志末尾的那一行
BASELINE_PEAK_KEY_PREFIX = "broad-mutation baseline "    # 基线那几步在这一版 baseline-peaks.<线程数>.tsv 里的键：前缀加步名
MEMORY_CAP_FORM = re.compile(r"^([1-9][0-9]*)([KMGT])$")  # run-with-memory-cap.sh 认的上限写法（1024 进制，单位必写）
MEMORY_CAP_UNITS = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
# 构建环境里 admission.py 那一份不管、驱动另加的：文档测试那一步（cargo test --doc）读 rustdoc 的标志
DOC_TEST_VARIABLES = ("RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS")
# research/scripts/detached-run.sh 本来就带进单元的变量：--unit-environment 不再打它们
UNIT_CARRIED_VARIABLES = ("SINGLEFS_HEAVY_TESTS", "PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME")
# 弄坏开关 unit-environment-fixed-list 下认的那几个（全局键改调构建环境之前驱动自己写的那一份）
FIXED_LIST_VARIABLES = ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")
IMPORT_COLUMNS = ("imported_at", "source", "source_results_sha256", "rows_read", "rows_imported", "rows_already_present", "conflicting_items")
DECISIVE_OUTCOMES = ("caught", "missed", "unviable")          # 这条变异自己的结局
ENVIRONMENTAL_OUTCOMES = ("timeout", "memory-cap")            # 也算定论、不再自动重跑，但不盖掉前面的 caught / missed / unviable
FINAL_OUTCOMES = (*DECISIVE_OUTCOMES, *ENVIRONMENTAL_OUTCOMES)
STAGE_ONE_ONLY = "stage-one-only"
DRIVER_ERROR = "driver-error"
NOT_RUN = "none"
ROW_OUTCOMES = (*FINAL_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR)
SUMMARY_COLUMNS = (*FINAL_OUTCOMES, STAGE_ONE_ONLY, DRIVER_ERROR, NOT_RUN)
SUMMARY_KEYS = {"caught": "caught", "missed": "missed", "unviable": "unviable", "timeout": "timeout", "memory-cap": "memory_cap",
                STAGE_ONE_ONLY: "stage_one_only", DRIVER_ERROR: "driver_error", NOT_RUN: "not_run"}
OUTCOME_TEXT = {"caught": "抓到", "missed": "没红", "unviable": "无效", "timeout": "超时", "memory-cap": "内存撞顶",
                STAGE_ONE_ONLY: "只过了第一段", DRIVER_ERROR: "驱动错", NOT_RUN: "这一版还没跑"}
MEMORY_CAP_HIT_EXIT = 250            # run-with-memory-cap.sh 的退出码，含义见它的文件头「退出码」一段
MEMORY_CAP_UNAVAILABLE_EXIT = 251
MEMORY_ADMISSION_REFUSED_EXIT = 252
TIME_LIMIT_HIT_EXIT = 253
SLICE_TOTAL_HIT_EXIT = 254
WRAPPER_INFRASTRUCTURE_EXITS = {MEMORY_CAP_UNAVAILABLE_EXIT: "带上限的 scope 起不来", MEMORY_ADMISSION_REFUSED_EXIT: "内存排不上",
                                SLICE_TOTAL_HIT_EXIT: "被 slice 的总上限挤掉"}
NEXTEST_TEST_FAILED_EXIT = 100       # cargo nextest run：有测试没过
CARGO_FAILED_EXIT = 101              # cargo：编不过、或 cargo test 有测试没过
EXIT_INCONSISTENT, EXIT_USAGE, EXIT_INFRASTRUCTURE, EXIT_LOCKED, EXIT_DIRTY_TREE, EXIT_BASELINE_RED, EXIT_TOOLCHAIN, EXIT_STOPPED = 1, 2, 3, 4, 5, 6, 7, 143
NEXTEST_FAILURE_LINE = re.compile(r"^\s*(?:FAIL|TIMEOUT|ABORT|LEAK-FAIL|SIG[A-Z0-9]+)\s+\[[^\]]*\]\s+(?:\([^)]*\)\s+)?(\S.*?)\s*$")
LIBTEST_FAILURE_LINE = re.compile(r"^test (\S.*?) \.\.\. FAILED\s*$")
# 进度行里数的几种日志行：编了一个 crate、nextest 过了一条、libtest 过了一条
COMPILING_LINE = re.compile(r"^\s*Compiling \S")
NEXTEST_PASS_LINE = re.compile(r"^\s*PASS \[")
LIBTEST_PASS_LINE = re.compile(r"^test \S.* \.\.\. ok\s*$")
# 编译那一步的失败才算无效：日志里有一条带源码位置的 rustc 错误（error 行后面两行之内有 `--> 文件:行:列`），又有 cargo 的「could not compile」
COMPILE_ERROR_LINE = re.compile(r"^error(?:\[E\d+\])?: ")
COMPILE_ERROR_LOCATION = re.compile(r"^\s*--> \S+:\d+:\d+")
COULD_NOT_COMPILE_LINE = re.compile(r"^error: could not compile ")
HUNK_HEADER = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")
UNRECOGNISED_TEST = "（没从输出里认出用例名，看日志）"
# cargo-mutants 自己改文件时跟在替换文后面的那段注释：照样带上，树里改出来的文件与它自己改的逐字相同（generate_list 拿它给的 diff 核）
MUTATION_MARKER = " /* ~ changed by cargo-mutants ~ */"
BASELINE_SEQUENCE = 0
ENVIRONMENT_SEQUENCE = "environment"      # 在没改过的树上验环境的那几步，日志与假一步的调用记录里用这个名字
# 旧版驱动恢复原件时把临时件建在树里，半途被杀会留下 <源文件>.restore.<进程号>：核树时认出来删掉
RESTORE_LEFTOVER = re.compile(r"^(?P<path>.+)\.restore\.\d+$")
# 内存包装给一步开的 scope：名字里第一段数字是包装自己的进程号（research/scripts/run-with-memory-cap.sh 的 unit="singlefs-memory-cap-$$-…"）
OWN_SCOPE_UNIT = re.compile(r"/(singlefs-memory-cap-(\d+)-\d+\.scope)\s*$")
SELFTEST_WORK_VARIABLE = "BROAD_MUTATION_SELFTEST_WORK"
TEST_ONLY_VARIABLES = ("BROAD_MUTATION_STEP_DRIVER", "BROAD_MUTATION_STEP_VIA_WRAPPER", "BROAD_MUTATION_LIST_DRIVER", "BROAD_MUTATION_TREE_SOURCE",
                       "BROAD_MUTATION_TOOLCHAIN", "BROAD_MUTATION_KILL_AT", "BROAD_MUTATION_ALLOW_VOLATILE_HOME", "BROAD_MUTATION_EXCLUSIONS", "BROAD_MUTATION_BREAK",
                       "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_TOOL_VERSIONS", "BROAD_MUTATION_CHANGED_SINCE_SOURCE",
                       "BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE", "BROAD_MUTATION_RESULT_KEY", "BROAD_MUTATION_RUSTC")
SELFTEST_ANCESTOR_DEPTH = 6


class Stopped(BaseException):
    """驱动收到 TERM / INT：在途那一条不落行，拷回原件再退。"""


class Refusal(Exception):
    def __init__(self, message, howto, code=EXIT_USAGE):
        super().__init__(message)
        self.message, self.howto, self.code = message, howto, code


def refuse(message, howto, code=EXIT_USAGE):
    raise Refusal(message, howto, code)


def print_refusal(refusal):
    print(f"  ✗ {refusal.message}", file=sys.stderr)
    print(f"  → 怎么办：{refusal.howto}", file=sys.stderr)


def say(text):
    print(text, flush=True)


def note(text):
    print(text, file=sys.stderr, flush=True)


# ── 只供自证的变量与弄坏开关：只在 --selftest 自己起的子进程里生效 ─────────────────────────────────────────

def process_command_line(process_id):
    """一个进程的 (argv, 当前目录, 父进程号)；读不到交 None。"""
    try:
        with open(f"/proc/{process_id}/cmdline", "rb") as handle:
            argv = [word.decode("utf-8", "replace") for word in handle.read().split(b"\0") if word]
        directory = os.readlink(f"/proc/{process_id}/cwd")
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            parent = int(handle.read().rsplit(")", 1)[1].split()[1])
    except (OSError, ValueError, IndexError):
        return None
    return argv, directory, parent


def selftest_ancestor_distance():
    """往上第几层祖先是「这份脚本带 --selftest」：父进程是 1；往上 SELFTEST_ANCESTOR_DEPTH 层里没有交 None。"""
    process_id = os.getppid()
    for distance in range(1, SELFTEST_ANCESTOR_DEPTH + 1):
        found = process_command_line(process_id)
        if found is None:
            return None
        argv, directory, parent = found
        if "--selftest" in argv[1:] and any(os.path.realpath(os.path.join(directory, word)) == REAL_SELF_PATH for word in argv[1:]):
            return distance
        if parent <= 1:
            return None
        process_id = parent
    return None


def raw_break(name):
    return name in os.environ.get("BROAD_MUTATION_BREAK", "").split(",")


TEST_MODE = None    # 第一次问的时候判一次；探针把驱动当模块载入时可以直接设成 True


def test_mode():
    """只供自证的变量与弄坏开关认不认：这个进程的父进程是带 --selftest 的这份脚本才认。"""
    global TEST_MODE
    if TEST_MODE is None:
        distance = selftest_ancestor_distance()
        TEST_MODE = distance == 1 or (distance is not None and raw_break("test-variables-anywhere"))
    return TEST_MODE


def test_variable(name):
    return os.environ.get(name) if test_mode() else None


def break_is(name):
    return test_mode() and raw_break(name)


def path_is_under(path, directory):
    return path == directory or path.startswith(directory.rstrip("/") + "/")


def heavy_prefix_text():
    """出路里起重活那条命令的前缀：照这一趟自己带的（提交流程里是 =commit、用户要求时 =user-request）；这一趟没带（--summary、--list-items 不要前缀）就两种都列。"""
    given = os.environ.get(HEAVY_PREFIX_VARIABLE)
    if given in HEAVY_PREFIX_VALUES:
        return f"{HEAVY_PREFIX_VARIABLE}={given}"
    return f"{HEAVY_PREFIX_VARIABLE}=commit（用户要求时 ={HEAVY_PREFIX_VALUES[1]}）"


def check_test_only_variables(home):
    """只供自证的变量出现在自证之外、或指着自证工作目录之外的库：拒，一步都不做。"""
    present = [name for name in TEST_ONLY_VARIABLES if os.environ.get(name)]
    if not present:
        return
    unset_command = "env " + " ".join(f"-u {name}" for name in present) + f" python3 {SELF_PATH} …"
    if not test_mode():
        refuse(f"环境里设了只供 --selftest 用的变量（{'、'.join(present)}），而这个进程不是 --selftest 自己起的子进程：假的一步、假清单与弄坏开关不许进真跑",
               f"清掉再跑：{unset_command}；要看这些变量怎么用，读文件头「只供测试」一段")
    work = os.environ.get(SELFTEST_WORK_VARIABLE)
    if raw_break("test-variables-anywhere"):
        return
    if not work or not path_is_under(home, os.path.realpath(work)):
        refuse(f"环境里设了只供 --selftest 用的变量（{'、'.join(present)}），而库目录 {home} 不在这一趟自证的工作目录（{work or '没给'}）里：只供自证的变量不许碰别的库",
               f"自证里的库都建在它自己的工作目录下；真库上跑就清掉这些变量：{unset_command}")


def kill_point(name):
    """自证造强杀的时刻：BROAD_MUTATION_KILL_AT 点到这里就给自己发 KILL。"""
    if test_variable("BROAD_MUTATION_KILL_AT") == name:
        os.kill(os.getpid(), signal.SIGKILL)


def now_text():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())   # clock-times:allow 结果行的「时刻」一段，造时间戳的格式串


def file_digest(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def fsync_directory(directory):
    descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_atomically(path, data):
    """同目录临时文件写完、fsync、改名，再 fsync 目录。data 是 bytes。"""
    temporary = f"{path}.partial.{os.getpid()}"
    with open(temporary, "wb") as handle:
        handle.write(data)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)
    fsync_directory(os.path.dirname(path))


def clean_field(value):
    return str(value).replace("\t", " ").replace("\r", " ").replace("\n", " ")


# ── 只追加的表：结果表、基线表、导入记录 ───────────────────────────────────────────────────────────────────

class TableRead(NamedTuple):
    rows: list              # 成形的行（段数对），按文件里的次序
    malformed_lines: list   # 写完了（有换行结尾）而段数不对的行的行号
    partial_tail: str       # 末行没写完（没有换行结尾）时它的文字，不算数；没有是 ""


def read_table(path, column_count):
    """读只追加的表。# 起头的行与空行不算；末行没有换行结尾的是没写完的半行，不算数也不算不成形。"""
    rows, malformed_lines, partial_tail = [], [], ""
    if not os.path.exists(path):
        return TableRead(rows, malformed_lines, partial_tail)
    with open(path, "rb") as handle:
        data = handle.read()
    lines = data.decode("utf-8", errors="replace").split("\n")
    if lines[-1] != "":
        if break_is("partial-row-kept"):
            lines.append("")
        else:
            partial_tail = lines[-1]
    for number, line in enumerate(lines[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != column_count:
            malformed_lines.append(number)
            continue
        rows.append(fields)
    return TableRead(rows, malformed_lines, partial_tail)


def write_fully(descriptor, data):
    """os.write 可以只写进去一部分（盘快满时）：写到写完为止，写不进去的让它抛出来。"""
    view = memoryview(data)
    while view:
        written = os.write(descriptor, view)
        view = view[written:]


def drop_partial_tail(path):
    """表的末行没写完（上一次写到一半断了）：截到最后一个换行，截掉的文字另存一份并说一句。交回表是不是空了（连表头都没写完）。"""
    with open(path, "rb") as handle:
        data = handle.read()
    if not data or data.endswith(b"\n"):
        return not data
    keep = data.rfind(b"\n") + 1
    dropped = data[keep:]
    with open(path + ".dropped-partial-rows", "ab") as handle:
        handle.write(dropped + b"\n")
    os.truncate(path, keep)
    note(f"  · {path} 的末行没写完（{len(dropped)} 字节，上一次写到一半断了）：已截掉、不算数，那一条会重跑；截掉的文字留在 {path}.dropped-partial-rows")
    return keep == 0


def append_rows(path, header_columns, rows):
    """往只追加的表里加几行并 fsync 一次；表还不在就先写表头（# 起头）。末行没写完的先截掉（drop_partial_tail）。"""
    data = b"".join(("\t".join(clean_field(field) for field in fields) + "\n").encode() for fields in rows)
    is_new = not os.path.exists(path)
    if not is_new:
        if break_is("partial-row-kept"):
            with open(path, "rb") as handle:
                existing = handle.read()
            if existing and not existing.endswith(b"\n"):
                data = b"\n" + data
        else:
            is_new = drop_partial_tail(path)
    if is_new:
        data = ("# " + "\t".join(header_columns) + "\n").encode() + data
    descriptor = os.open(path, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    try:
        if test_variable("BROAD_MUTATION_KILL_AT") == "row-half-written" and os.path.basename(path) == "results.tsv":
            write_fully(descriptor, data[:-5])     # 行差最后几个字节没写完（表头是完整的）就被杀
            kill_point("row-half-written")
        write_fully(descriptor, data)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    if is_new:
        fsync_directory(os.path.dirname(path))


def append_row(path, header_columns, fields):
    append_rows(path, header_columns, [fields])


def read_baseline_rows(path):
    """这一版 baseline.tsv 里成形的行：[(文件里的行号, {列名: 值})]。行号从 1 起、表头那一行也算（升档登记行记的 baseline_row 就是它，门禁按它找那一行）；
    末行没写完（没有换行结尾）的不算。"""
    found = []
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")
    except FileNotFoundError:
        return found
    for number, line in enumerate(lines[:-1], start=1):
        fields = line.split("\t")
        if line and not line.startswith("#") and len(fields) == len(BASELINE_COLUMNS):
            found.append((number, dict(zip(BASELINE_COLUMNS, fields))))
    return found


# ── 结果行的校验列（文件头「结果行的校验列」） ───────────────────────────────────────────────────────────────

def result_key_path():
    """结果行的密钥放哪：${XDG_CONFIG_HOME:-~/.config}/singlefs/broad-mutation-result.key（库目录之外、仓之外）；自证里是 BROAD_MUTATION_RESULT_KEY 指的那一份。"""
    fake = test_variable("BROAD_MUTATION_RESULT_KEY")
    if fake:
        return os.path.realpath(fake)
    if test_mode():
        refuse("自证起的子进程没设 BROAD_MUTATION_RESULT_KEY：自证不许读写真的密钥", "看 Selftest 的环境：每一趟都带 BROAD_MUTATION_RESULT_KEY=<自证工作目录里的一份>")
    base = os.environ.get("XDG_CONFIG_HOME") or os.path.join(os.path.expanduser("~"), ".config")
    return os.path.realpath(os.path.join(base, DEFAULT_RESULT_KEY))


def read_result_key(path):
    """读密钥：交 (字节, "")；用不了交 (None, 为什么)。要只有自己能读写（属主是自己、属组与别人没有任何位）、内容是 64 位小写十六进制。"""
    try:
        status = os.stat(path)
    except FileNotFoundError:
        return None, f"密钥 {path} 不在"
    except OSError as error:
        return None, f"读不了密钥 {path}：{error}"
    if status.st_uid != os.getuid() or status.st_mode & 0o077:
        return None, f"密钥 {path} 的权限是 {status.st_mode & 0o777:o}、属主 {status.st_uid}：要属主是自己、权限 600"
    with open(path, encoding="ascii", errors="replace") as handle:
        text = handle.read().strip()
    if len(text) != RESULT_KEY_HEX_LENGTH or not re.fullmatch(r"[0-9a-f]+", text):
        return None, f"密钥 {path} 的内容不是 {RESULT_KEY_HEX_LENGTH} 位小写十六进制"
    return bytes.fromhex(text), ""


def create_result_key(path):
    """没有密钥时建一份：目录 700、文件 600（排他新建）、32 字节随机数写成十六进制。别的一趟刚建了就照用它的，交 False。"""
    directory = os.path.dirname(path)
    os.makedirs(directory, mode=0o700, exist_ok=True)
    try:
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError:
        return False
    try:
        write_fully(descriptor, (secrets.token_hex(RESULT_KEY_HEX_LENGTH // 2) + "\n").encode())
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    fsync_directory(directory)
    return True


def result_mac(key, fields):
    """一行结果的校验列：HMAC-SHA256(密钥, 前十一段照写进表里的样子用制表符接起来、UTF-8)，小写十六进制。"""
    text = "\t".join(clean_field(field) for field in fields[:RESULT_SIGNED_COLUMNS])
    return hmac.new(key, text.encode("utf-8", "surrogateescape"), hashlib.sha256).hexdigest()


def signed_result_row(key, fields):
    """十一段的结果行加上校验列：交十二段（前十一段已按写表的样子洗过）。"""
    cleaned = [clean_field(field) for field in fields[:RESULT_SIGNED_COLUMNS]]
    return cleaned + [result_mac(key, cleaned)]


class ResultTableRead(NamedTuple):
    rows: list              # 校验对得上的行（只留前十一段），按文件里的次序
    forged: list            # [(文件里的行号, 序号, 为什么不算)]：没有校验列、校验列对不上、没有密钥校验不了
    malformed_lines: list   # 写完了而段数既不是十二也不是十一的行的行号
    partial_tail: str       # 末行没写完时它的文字


def read_result_table(path, key, key_problem=""):
    """读一版的结果表：十二段且校验列对得上的行才算数；十一段的（没有校验列）、校验列对不上的、没有密钥时的每一行都不算、列进 forged。
    弄坏开关 result-mac-unchecked 下十一段、十二段的都算，不核校验列。"""
    rows, forged, malformed_lines, partial_tail = [], [], [], ""
    if not os.path.exists(path):
        return ResultTableRead(rows, forged, malformed_lines, partial_tail)
    with open(path, "rb") as handle:
        lines = handle.read().decode("utf-8", errors="surrogateescape").split("\n")
    if lines[-1] != "":
        if break_is("partial-row-kept"):
            lines.append("")
        else:
            partial_tail = lines[-1]
    unchecked = break_is("result-mac-unchecked")
    for number, line in enumerate(lines[:-1], start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) == RESULT_SIGNED_COLUMNS:
            if unchecked:
                rows.append(fields)
            else:
                forged.append((number, fields[0], "没有校验列（十一段）"))
            continue
        if len(fields) != len(RESULT_COLUMNS):
            malformed_lines.append(number)
            continue
        if not unchecked:
            if key is None:
                forged.append((number, fields[0], key_problem or "没有密钥，校验不了"))
                continue
            if not hmac.compare_digest(result_mac(key, fields), fields[-1]):
                forged.append((number, fields[0], "校验列对不上"))
                continue
        rows.append(fields[:RESULT_SIGNED_COLUMNS])
    return ResultTableRead(rows, forged, malformed_lines, partial_tail)


# ── 库目录、锁 ─────────────────────────────────────────────────────────────────────────────────────────────

def filesystem_type_of(path):
    best, best_type = "", ""
    try:
        with open("/proc/mounts", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                parts = line.split()
                if len(parts) < 3:
                    continue
                mount_point = parts[1].replace("\\040", " ")
                if (path == mount_point or path.startswith(mount_point.rstrip("/") + "/")) and len(mount_point) > len(best):
                    best, best_type = mount_point, parts[2]
    except OSError:
        return ""
    return best_type


def resolve_home(given):
    home = os.path.realpath(os.path.expanduser(given or os.environ.get("SINGLEFS_BROAD_MUTATION_HOME") or DEFAULT_HOME))
    # 只供自证的变量先判：它们带着假的一步与弄坏开关，出现在自证之外比库目录落在哪更根本；放在后面，临时目录那一拒会先把它盖掉
    check_test_only_variables(home)
    if test_variable("BROAD_MUTATION_ALLOW_VOLATILE_HOME") == "1":
        return home
    existing = home
    while not os.path.exists(existing):
        existing = os.path.dirname(existing)
    under_volatile = any(home == prefix or home.startswith(prefix + "/") for prefix in VOLATILE_PREFIXES)
    if under_volatile or filesystem_type_of(existing) in VOLATILE_FILESYSTEMS:
        refuse(f"库目录 {home} 落在开机会清空的目录里（/tmp、/run、/dev/shm 或 tmpfs）：跑了几天的结果一重启就没了",
               f"换一个落在盘上的目录：--home <目录> 或 SINGLEFS_BROAD_MUTATION_HOME=<目录>；默认是 {DEFAULT_HOME}")
    return home


def acquire_lock(home, wait_seconds):
    """库目录的排他锁。描述符可继承：每一步命令都带着它，驱动被 KILL 之后那些进程还活着时锁还占着，下一趟看得见。"""
    os.makedirs(home, exist_ok=True)
    descriptor = os.open(os.path.join(home, "lock"), os.O_RDWR | os.O_CREAT, 0o644)
    os.set_inheritable(descriptor, True)
    deadline = time.monotonic() + wait_seconds
    announced = False
    while True:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            return descriptor
        except OSError:
            if time.monotonic() >= deadline:
                os.close(descriptor)
                refuse(f"库目录 {home} 的锁被占着（等了 {wait_seconds} 秒）：另一趟驱动在跑，或上一趟被 KILL 之后它起的那一步（内存包装与它开的 scope）还没退",
                       "`ps -o pid,ppid,etimes,args -u \"$(id -u)\"` 找出还在跑的那一趟（run-with-memory-cap.sh、cargo、cargo-nextest、测试二进制），"
                       "等它退出再重跑同一条命令；确是上一趟留下的，逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>` 停掉再跑；"
                       "要多等就加 --lock-wait <秒>", EXIT_LOCKED)
            if not announced:
                note(f"  · 库目录的锁被占着，最多等 {wait_seconds} 秒（上一趟留下的进程退出就放开）")
                announced = True
            time.sleep(LOCK_POLL_SECONDS)


# ── 树：导出、指纹、在途记录 ───────────────────────────────────────────────────────────────────────────────

def export_paths():
    """导出哪些路径：门禁 checker-tier-crates-mutation-replay 登记的拷贝范围（去掉 research/ 下的：那是那道门禁自己的脚本）加 .cargo。"""
    specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_broad_mutation",
                                                           os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"))
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    try:
        copied = module.copied_paths(REPOSITORY)
    except Exception as error:  # 登记表读不了、没有那一行
        refuse(f"读不出导出范围（research/scripts/crates-mutation-rows.py 的 copied_paths）：{error}",
               "看 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行还在不在、成不成形")
    paths = [path for path in copied if not path.startswith("research/")] + [".cargo"]
    if "crates" not in paths:
        refuse(f"导出范围里没有 crates（读到的是 {copied}）", "看 .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay.sh 那一行登记的路径")
    return paths


def manifest_of_tree(tree):
    """树里逐文件的 sha256：{相对路径: sha256}；叫 target / mutants.out 的目录不算。"""
    manifest = {}
    for directory, directory_names, file_names in os.walk(tree):
        directory_names[:] = sorted(name for name in directory_names if name not in IGNORED_DIRECTORY_NAMES)
        for file_name in sorted(file_names):
            path = os.path.join(directory, file_name)
            if os.path.islink(path) or not os.path.isfile(path):
                continue
            manifest[os.path.relpath(path, tree)] = file_digest(path)
    return manifest


def files_in_tree(tree):
    """树里每个文件的相对路径（符号链接也算一个）；叫 target / mutants.out 的目录不算。"""
    found = set()
    for directory, directory_names, file_names in os.walk(tree):
        directory_names[:] = sorted(name for name in directory_names if name not in IGNORED_DIRECTORY_NAMES)
        for file_name in file_names:
            found.add(os.path.relpath(os.path.join(directory, file_name), tree))
    return found


def fingerprint_of_manifest(manifest):
    if break_is("fingerprint-path-blind"):
        return hashlib.sha256("".join(f"{digest}\n" for digest in sorted(manifest.values())).encode()).hexdigest()
    text = "".join(f"{manifest[path]}  {path}\n" for path in sorted(manifest))
    return hashlib.sha256(text.encode()).hexdigest()


def read_manifest(path):
    manifest = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            digest, _, relative = line.rstrip("\n").partition("\t")
            manifest[relative] = digest
    return manifest


def toolchain_text(directory, environment=None):
    """这一刻的工具链：在 directory 里跑 rustc -Vv 的输出（rustup 按目录与 RUSTUP_TOOLCHAIN 这类变量挑工具链）；environment 给了就在它下面跑。
    自证里 BROAD_MUTATION_RUSTC 给了就跑它指的假 rustc（--unit-environment 比工具链那一格用）。"""
    fake = test_variable("BROAD_MUTATION_TOOLCHAIN")
    if fake is not None:
        return fake.strip() + "\n"
    fake_rustc = test_variable("BROAD_MUTATION_RUSTC")
    if test_mode() and not fake_rustc:
        return "selftest-toolchain\n"
    try:
        found = subprocess.run([fake_rustc or "rustc", "-Vv"], capture_output=True, text=True, cwd=directory, stdin=subprocess.DEVNULL,
                               env=None if environment is None else dict(environment))
    except OSError as error:
        refuse(f"起不了 rustc -Vv：{error}", "PATH 上要有 rustc（rustup 装的那一个）")
    if found.returncode != 0 or not found.stdout.strip():
        refuse(f"rustc -Vv 退 {found.returncode}：{found.stderr.strip()[-300:]}", f"在 {directory} 里手跑 rustc -Vv 看报错")
    return found.stdout.strip() + "\n"


def pre_fresh_build_judge():
    """照 fresh-build 之前的驱动算版本指纹与全局键（判法版本照旧、不进指纹、加工具链之前建的版本目录照认）：
    只在自证模拟旧驱动（BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE=1）与弄坏开关 stale-judge-version 下成立。"""
    return break_is("stale-judge-version") or test_variable("BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE") == "1"


def judge_rules_version():
    return PRE_FRESH_BUILD_JUDGE_RULES_VERSION if pre_fresh_build_judge() else JUDGE_RULES_VERSION


def version_key(home, source_fingerprint, toolchain, global_key_digest):
    """这一版的指纹：sha256(源码指纹、换行、工具链、「judge <判法版本>」一行、「global <全局键的 sha256>」一行)。判法版本与全局键进指纹：
    判法或全局键（E4）改了，同一个提交重做 --sync-tree 也是另一版，旧版目录里的结果一条都不当这一版的。
    fresh-build 之前建的版本目录（名字是源码指纹、或 sha256(源码指纹、换行、工具链)）与全局键进指纹之前建的不再认。"""
    if break_is("toolchain-ignored"):
        return source_fingerprint
    if not pre_fresh_build_judge():
        if break_is("global-key-outside-version"):
            return hashlib.sha256(f"{source_fingerprint}\n{toolchain}judge {JUDGE_RULES_VERSION}\n".encode()).hexdigest()
        return hashlib.sha256(f"{source_fingerprint}\n{toolchain}judge {JUDGE_RULES_VERSION}\nglobal {global_key_digest}\n".encode()).hexdigest()
    legacy = os.path.join(home, "versions", source_fingerprint)
    if os.path.isdir(legacy):
        recorded_path = os.path.join(legacy, "toolchain.txt")
        if not os.path.isfile(recorded_path):
            return source_fingerprint
        with open(recorded_path, encoding="utf-8") as handle:
            if handle.read() == toolchain:
                return source_fingerprint
    return hashlib.sha256(f"{source_fingerprint}\n{toolchain}".encode()).hexdigest()


def remove_tree(path):
    if os.path.lexists(path):
        shutil.rmtree(path)


class ItemState(NamedTuple):
    state: str               # 这一条的状态（ROW_OUTCOMES 之一或 NOT_RUN），判法见 state_of_outcomes
    passed_stage_one: bool   # 这一版里有它的 stage-one-only 行：第一段跑过而且没红


def state_of_outcomes(outcomes):
    """一条在这一版里各行的结局（按文件里的次序）→ ItemState。不取最后一行：
    caught / missed / unviable 取其中最后一行；没有它们才取 timeout / memory-cap 的最后一行；都没有而有 stage-one-only 的是它；只有 driver-error 的是它。"""
    passed_stage_one = STAGE_ONE_ONLY in outcomes
    if break_is("state-is-last-row"):
        return ItemState(outcomes[-1] if outcomes else NOT_RUN, passed_stage_one)
    decisive = [outcome for outcome in outcomes if outcome in DECISIVE_OUTCOMES]
    environmental = [outcome for outcome in outcomes if outcome in ENVIRONMENTAL_OUTCOMES]
    if decisive:
        return ItemState(decisive[-1], passed_stage_one)
    if environmental:
        return ItemState(environmental[-1], passed_stage_one)
    if passed_stage_one:
        return ItemState(STAGE_ONE_ONLY, True)
    if DRIVER_ERROR in outcomes:
        return ItemState(DRIVER_ERROR, False)
    return ItemState(NOT_RUN, False)


class ResultsRead(NamedTuple):
    rows: list               # 结果表里成形又属于这一版指纹的行（连同还攒在内存里的）
    malformed_lines: list    # 写完了而段数不对的行的行号
    partial_tail: str        # 末行没写完时它的文字
    mismatched_names: list   # [(序号, 结果行里的变异名, 清单里同序号的变异名)]
    unknown_sequences: list  # 清单里没有的序号
    unknown_outcomes: list   # [(序号, 认不出的结局)]
    forged: list = []        # [(文件里的行号, 序号, 为什么不算)]：没有校验列、校验列对不上、没有密钥校验不了的行（不进 rows）

    def forged_text(self):
        number, sequence, why = self.forged[0]
        return f"{len(self.forged)} 行不算（例：文件第 {number} 行、第 {sequence} 条，{why}）"

    def blocking_problems(self):
        """续跑开跑前就要拦的：结果行与清单对不上（行是别的清单的，照着它选条会选错）。"""
        problems = []
        if self.mismatched_names:
            sequence, row_name, list_name = self.mismatched_names[0]
            problems.append(f"{len(self.mismatched_names)} 行的变异名与清单同序号的不同（例：第 {sequence} 条，结果行「{row_name}」、清单「{list_name}」）")
        if self.unknown_sequences:
            problems.append(f"结果表里有清单里没有的序号：{'、'.join(self.unknown_sequences[:5])}")
        return problems


class Library:
    """库目录里的各个位置与这一版的状态。"""

    def __init__(self, home):
        self.home = home
        self.tree = os.path.join(home, "tree")
        self.tree_source = os.path.join(home, "tree.source")
        self.tree_source_pending = os.path.join(home, "tree.source.pending")
        self.target = os.path.join(home, "target")
        self.scratch = os.path.join(home, "scratch")
        self.inflight_record = os.path.join(home, "inflight.json")
        self.inflight_original = os.path.join(home, "inflight.original")
        self.inflight_restore = os.path.join(home, "inflight.restore")
        self.fingerprint = None
        self.source = None
        self.manifest = None
        self.items = None
        self.lock_descriptor = None
        self.deferred_rows = []
        self.result_key_cache = None

    def result_key(self, create=False):
        """结果行的密钥（文件头「结果行的校验列」）：交 (字节, "")；没有或用不了交 (None, 为什么)。create（要往结果表里写行的那几种用法）时没有就建，
        建了还用不了拒（退 2）。密钥落在库目录里拒：库目录里的东西手写得了，密钥就防不住手写的行。"""
        if self.result_key_cache is not None:
            return self.result_key_cache, ""
        path = result_key_path()
        if path_is_under(path, os.path.realpath(self.home)):
            refuse(f"结果行的密钥 {path} 落在库目录 {self.home} 里：手写得了结果行的人也读得到它，校验列就防不住手写的行",
                   f"密钥放库目录之外（默认 ${{XDG_CONFIG_HOME:-~/.config}}/{DEFAULT_RESULT_KEY}）")
        key, problem = read_result_key(path)
        if key is None and create and not os.path.exists(path):
            if create_result_key(path):
                say(f"  · 结果行的密钥还没有：建了 {path}（权限 600）；之后每一行结果都带它算的校验列，门禁逐行核")
            key, problem = read_result_key(path)
        if key is None and create:
            refuse(f"结果行的密钥用不了：{problem}",
                   f"照那一句修（chmod 600 {path}、内容是 64 位小写十六进制）；密钥丢了或换了，旧的结果行就都校验不过：照文件头「结果行的校验列」整库重建")
        self.result_key_cache = key
        return key, problem

    def version_directory(self):
        return os.path.join(self.home, "versions", self.fingerprint)

    def version_file(self, name):
        return os.path.join(self.version_directory(), name)

    def resync_command(self):
        return f"{heavy_prefix_text()} python3 {SELF_PATH} --sync-tree --commit {(self.source or {}).get('commit', 'HEAD')}"

    def source_fingerprint(self):
        """这一版的源码指纹：加工具链之前建的版本，指纹就是它。"""
        return self.source.get("source_fingerprint") or self.fingerprint

    def load_source(self, for_run):
        """读 tree.source 定这一版；树还没导出就拒。上一次 --sync-tree 没做完（版本指针的临时件还在）：要用树的拒，只读的说一句。"""
        if not os.path.isfile(self.tree_source) or not os.path.isdir(self.tree):
            refuse(f"库目录 {self.home} 里还没有专用树", f"先导出一棵：{heavy_prefix_text()} python3 {SELF_PATH} --sync-tree（默认导出主仓 HEAD；不跑测试）")
        with open(self.tree_source, encoding="utf-8") as handle:
            self.source = json.load(handle)
        self.fingerprint = self.source["fingerprint"]
        if os.path.exists(self.tree_source_pending):
            if for_run:
                refuse(f"上一次 --sync-tree 没做完（{self.tree_source_pending} 还在）：树可能已经换成新的，版本指针还是旧的，照旧指纹跑会把新树的结果记在旧版名下",
                       f"把那一次做完：{heavy_prefix_text()} python3 {SELF_PATH} --sync-tree --commit <要的提交>（回到原来那一版就给原来的提交 "
                       f"{str(self.source.get('commit'))[:12]}，指纹不变，已有结果照用）", EXIT_DIRTY_TREE)
            note(f"  · 注意：上一次 --sync-tree 没做完（{self.tree_source_pending} 还在）；下面列的是版本指针此刻指着的那一版")

    def load_manifest(self):
        self.manifest = read_manifest(self.version_file("manifest.tsv"))

    def load_items(self):
        path = self.version_file("list.json")
        if not os.path.isfile(path):
            refuse(f"这一版（指纹 {self.fingerprint[:16]}）还没有清单：{path} 不在", f"重新导出并生成：{self.resync_command()}")
        with open(path, encoding="utf-8") as handle:
            self.items = json.load(handle)

    def read_results(self):
        """这一版的结果表：成形、校验列对得上又属于这一版指纹的行（只留前十一段），连同对不上的地方。"""
        key, key_problem = self.result_key()
        table = read_result_table(self.version_file("results.tsv"), key, key_problem)
        fingerprint_column = RESULT_COLUMNS.index("fingerprint")
        rows = [row for row in table.rows if row[fingerprint_column] == self.fingerprint]
        rows += [[clean_field(field) for field in row] for row in self.deferred_rows]
        names = {str(item["sequence"]): item["name"] for item in self.items}
        mismatched, unknown_sequences, unknown_outcomes = [], [], []
        for row in rows:
            sequence, name, outcome = row[0], row[2], row[3]
            if sequence not in names:
                if sequence not in unknown_sequences:
                    unknown_sequences.append(sequence)
                continue
            if names[sequence] != name and not break_is("names-unchecked"):
                mismatched.append((sequence, name, names[sequence]))
            if outcome not in ROW_OUTCOMES:
                unknown_outcomes.append((sequence, outcome))
        return ResultsRead(rows, table.malformed_lines, table.partial_tail, mismatched, unknown_sequences, unknown_outcomes, table.forged)

    def item_states(self, results=None):
        """每条的状态：{序号（字符串）: ItemState}；一行都没有的不在里面。"""
        results = results or self.read_results()
        outcomes = {}
        for row in results.rows:
            outcomes.setdefault(row[0], []).append(row[3])
        return {sequence: state_of_outcomes(found) for sequence, found in outcomes.items()}

    def record_result(self, fields):
        """落一行结果：十一段加上校验列（signed_result_row）。"""
        if break_is("no-row-flush"):
            self.deferred_rows.append(fields)
            return
        append_row(self.version_file("results.tsv"), RESULT_COLUMNS, signed_result_row(self.result_key(create=True)[0], fields))

    def record_results(self, rows):
        """一次落几行（沿用行、导入的行），各带校验列，fsync 一次。"""
        key = self.result_key(create=True)[0]
        append_rows(self.version_file("results.tsv"), RESULT_COLUMNS, [signed_result_row(key, fields) for fields in rows])

    def flush_deferred_rows(self):
        if self.deferred_rows:
            self.record_results(self.deferred_rows)
        self.deferred_rows = []


def restore_original(library):
    """照在途记录把原件拷回树里、核 sha256、删掉在途记录：交回那份在途记录，没有在途记录交 None。临时件建在库目录里，不进树。"""
    if not os.path.exists(library.inflight_record):
        if os.path.exists(library.inflight_original):
            os.remove(library.inflight_original)   # 存了原件、还没记在途记录就停了：树没动过
        return None
    with open(library.inflight_record, encoding="utf-8") as handle:
        record = json.load(handle)
    target = os.path.join(library.tree, record["file"])
    if not os.path.isfile(library.inflight_original) or file_digest(library.inflight_original) != record["original_digest"]:
        refuse(f"在途记录说 {record['file']} 被第 {record['sequence']} 条改过，但存下的原件不在或对不上（{library.inflight_original}）",
               f"重建那棵树：{library.resync_command()}（同一个提交、同一套工具链导出来指纹不变，已有结果照用）", EXIT_DIRTY_TREE)
    shutil.copyfile(library.inflight_original, library.inflight_restore)
    kill_point("restore-after-copy")
    os.replace(library.inflight_restore, target)
    if file_digest(target) != record["original_digest"]:
        refuse(f"把原件拷回 {target} 之后 sha256 还是对不上", f"重建那棵树：{library.resync_command()}", EXIT_DIRTY_TREE)
    os.remove(library.inflight_record)
    os.remove(library.inflight_original)
    fsync_directory(library.home)
    return record


def verify_tree_is_pristine(library):
    """树里 manifest 登记的每个文件都与登记的 sha256 相同、也没有多出来的文件；不同就拒（没有在途记录可依，恢复不了）。
    旧版驱动恢复原件半途被杀留下的 <登记的文件>.restore.<进程号> 认得出来，删掉并说一句。"""
    different = []
    for path, digest in library.manifest.items():
        full = os.path.join(library.tree, path)
        if not os.path.isfile(full) or file_digest(full) != digest:
            different.append(path)
    if different:
        refuse(f"树里有 {len(different)} 个文件与这一版的 manifest 对不上、又没有在途记录可以恢复：{'、'.join(different[:5])}"
               + ("……" if len(different) > 5 else ""),
               f"重建那棵树：{library.resync_command()}（同一个提交、同一套工具链导出来指纹不变，已有结果照用）", EXIT_DIRTY_TREE)
    if break_is("extra-files-unchecked"):
        return len(library.manifest)
    extra = sorted(files_in_tree(library.tree) - set(library.manifest))
    unexplained = []
    for path in extra:
        leftover = RESTORE_LEFTOVER.match(path)
        if leftover and leftover.group("path") in library.manifest:
            os.remove(os.path.join(library.tree, path))
            say(f"  · 树里留着旧版驱动恢复原件时的临时件 {path}：已删掉")
        else:
            unexplained.append(path)
    if unexplained:
        refuse(f"树里多出 {len(unexplained)} 个这一版 manifest 没登记的文件：{'、'.join(unexplained[:5])}" + ("……" if len(unexplained) > 5 else "")
               + "（多出来的测试文件会被 cargo 一起编进去，判出来的就不是这一版）",
               f"确认是谁留下的之后删掉它们（都在 {library.tree} 下），或重建那棵树：{library.resync_command()}", EXIT_DIRTY_TREE)
    return len(library.manifest)


def check_toolchain(library):
    """真跑之前：此刻的工具链与这一版记下的相同。这一版还没记（加工具链之前建的版本）就把此刻的记进去。"""
    if break_is("toolchain-ignored"):
        return
    current = toolchain_text(library.tree)
    path = library.version_file("toolchain.txt")
    if not os.path.isfile(path):
        write_atomically(path, current.encode())
        say(f"  · 这一版还没记工具链（加工具链之前建的）：已把此刻的 rustc -Vv 记进 {path}（{current.splitlines()[0]}）")
        return
    with open(path, encoding="utf-8") as handle:
        recorded = handle.read()
    if recorded != current:
        refuse(f"工具链与这一版（指纹 {library.fingerprint[:16]}）记下的不同：记下的是「{recorded.splitlines()[0] if recorded.strip() else '空'}」，此刻是「{current.splitlines()[0]}」；"
               "换了工具链判出来的行不能记在同一版名下",
               f"要接着这一版跑就换回原来的工具链（{path} 里是全文）；要在新工具链下跑就另起一版：{library.resync_command()}（旧版的结果留在它自己的目录里）",
               EXIT_TOOLCHAIN)


def workspace_packages(tree):
    """树里每份 Cargo.toml 的 [package] name（导出范围里就是工作区的各个成员）。读不出的 Cargo.toml 拒（退 5）。"""
    import tomllib
    names = set()
    for directory, directory_names, file_names in os.walk(tree):
        directory_names[:] = sorted(name for name in directory_names if name not in IGNORED_DIRECTORY_NAMES)
        if "Cargo.toml" not in file_names:
            continue
        path = os.path.join(directory, "Cargo.toml")
        try:
            with open(path, "rb") as handle:
                package = tomllib.load(handle).get("package")
        except (OSError, tomllib.TOMLDecodeError) as error:
            refuse(f"读不出 {path}：{error}，定不了编译目录里哪些产物是工作区的包", f"看那份 Cargo.toml；树坏了就重建：python3 {SELF_PATH} --sync-tree", EXIT_DIRTY_TREE)
        if isinstance(package, dict) and isinstance(package.get("name"), str):
            names.add(package["name"])
    return names


def build_directories(library):
    """驱动起的 cargo 用的编译目录：每一步的 target/、覆盖那一步的 target-coverage/。"""
    return (library.target, os.path.join(library.home, "target-coverage"))


def workspace_fingerprint_entries(library, packages):
    """编译目录里工作区各包的 fingerprint 条目（<编译目录>/<profile>/.fingerprint/<包名>-<16 位十六进制>，与带三元组的
    <编译目录>/<三元组>/<profile>/.fingerprint/… 两层都看）；名字只认 packages 里的包，第三方依赖的不算。交路径列表。"""
    candidates = []
    for build in build_directories(library):
        if not os.path.isdir(build):
            continue
        for first in sorted(os.listdir(build)):
            level = os.path.join(build, first)
            if not os.path.isdir(level):
                continue
            candidates.append(os.path.join(level, ".fingerprint"))
            candidates += [os.path.join(level, second, ".fingerprint") for second in sorted(os.listdir(level)) if second != ".fingerprint"]
    found = []
    for directory in candidates:
        if not os.path.isdir(directory):
            continue
        for entry in sorted(os.listdir(directory)):
            named = FINGERPRINT_ENTRY.match(entry)
            if named and named.group("package") in packages:
                found.append(os.path.join(directory, entry))
    return found


def newest_mtime_ns(path):
    """一个 fingerprint 条目最近一次被写的时刻：它自己与里面各文件的 mtime 取最大。"""
    newest = os.lstat(path).st_mtime_ns
    if os.path.isdir(path) and not os.path.islink(path):
        for name in os.listdir(path):
            newest = max(newest, os.lstat(os.path.join(path, name)).st_mtime_ns)
    return newest


def clear_workspace_fingerprints(library):
    """换树之后：编译目录里工作区各包的 fingerprint 一律删掉，下一次编译必重编；第三方依赖的不动（清了每趟全编）。交 (删了几个, 工作区包名)。"""
    if break_is("sync-keeps-stale-artifacts"):
        return 0, set()
    packages = workspace_packages(library.tree)
    entries = workspace_fingerprint_entries(library, packages)
    for path in entries:
        if os.path.isdir(path) and not os.path.islink(path):
            shutil.rmtree(path)
        else:
            os.remove(path)
    left = workspace_fingerprint_entries(library, packages)
    if left:
        refuse(f"清了编译目录里工作区各包的 fingerprint 之后回读还剩 {len(left)} 个（例：{left[0]}）", f"看那个目录的权限与盘满没满，再重做：python3 {SELF_PATH} --sync-tree",
               EXIT_DIRTY_TREE)
    return len(entries), packages


def stale_build_problems(library):
    """基线之前核：编译目录里不许有换树之前编的工作区产物，交问题清单（空的是核过）。版本指针的 synced_at_ns 是这一次换树开工的时刻：
    ① 树里有文件的 mtime 早于它：导出时还原了提交时刻，cargo 会把换树之前编的产物当新鲜；没有 synced_at_ns 的是 fresh-build 之前换的树，同样说不清；
    ② 编译目录里有工作区包的 fingerprint 早于它：换树之前编的，没清。"""
    started = library.source.get("synced_at_ns")
    if not isinstance(started, int):
        return [f"版本指针 {library.tree_source} 里没有 synced_at_ns（这棵树是 {library.source.get('exported_at', '?')} 换的，早于「换树后清编译目录」那次修正）："
                "说不清编译目录里工作区各包的产物是不是换树之前编的"]
    floor = started - SYNC_CLOCK_SLACK_NS
    problems = []
    older = sorted(path for path in files_in_tree(library.tree) if os.lstat(os.path.join(library.tree, path)).st_mtime_ns < floor)
    if older:
        problems.append(f"树里 {len(older)} 个文件的 mtime 早于这一次换树（例：{'、'.join(older[:3])}）：导出时还原了提交时刻，cargo 会把换树之前编的产物当新鲜")
    stale = [path for path in workspace_fingerprint_entries(library, workspace_packages(library.tree)) if newest_mtime_ns(path) < floor]
    if stale:
        problems.append(f"编译目录里 {len(stale)} 个工作区包的 fingerprint 早于这一次换树、没清（例：{'、'.join(os.path.relpath(path, library.home) for path in stale[:3])}）")
    return problems


def sync_tree(library, arguments):
    """把主仓某个提交（arguments.commit）导出成专用树，算指纹（全局键按 arguments 的限时参数与 --llvm-bin 现算）、写这一版的 manifest、工具链与全局键，
    换树、换版本指针，没有清单就生成。
    次序：新树导到 tree.new → 写这一版目录里的 manifest / toolchain → 写版本指针的临时件 tree.source.pending → 换树 → 临时件改名成 tree.source。
    任何一步被杀，要么树与指针都还是旧的，要么临时件还在（真跑见到它就拒，重做 --sync-tree），不会让新树挂在旧指纹下。
    编译目录跨版本复用、cargo 按 mtime 判本地包新不新鲜：导出用 tar -x -m（mtime 取解开那一刻，不还原提交时刻），换树之后再清掉编译目录里
    工作区各包的 fingerprint（clear_workspace_fingerprints），两样都做；开工时刻记进版本指针的 synced_at_ns，基线之前照它核（stale_build_problems）。"""
    synced_at_ns = time.time_ns()
    revision = arguments.commit
    fake_source = test_variable("BROAD_MUTATION_TREE_SOURCE")
    new_tree = library.tree + ".new"
    remove_tree(new_tree)
    if fake_source:
        commit, paths = "selftest", ["."]
        packer = ["tar", "-c", "-C", fake_source, "."]     # 自证的假源也走打包、解包这一条，解包那一半与真跑同一份
    else:
        resolved = subprocess.run(["git", "-C", REPOSITORY, "rev-parse", "--verify", f"{revision}^{{commit}}"],
                                  capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if resolved.returncode != 0:
            refuse(f"认不出提交「{revision}」：{resolved.stderr.strip()}", "--commit 给一个主仓里有的提交（默认 HEAD）")
        commit, paths = resolved.stdout.strip(), export_paths()
        packer = ["git", "-C", REPOSITORY, "archive", "--format=tar", commit, "--", *paths]
    os.makedirs(new_tree)
    archive = subprocess.Popen(packer, stdout=subprocess.PIPE, stdin=subprocess.DEVNULL)
    keep_commit_times = break_is("sync-keeps-stale-artifacts")
    unpack = subprocess.run(["tar", "-x", *([] if keep_commit_times else ["-m"]), "-C", new_tree], stdin=archive.stdout)
    archive.stdout.close()
    archive_exit = archive.wait()
    if archive_exit != 0 or unpack.returncode != 0:
        remove_tree(new_tree)
        refuse(f"导出提交 {commit[:12]} 没成（{packer[0]} 打包退 {archive_exit}，tar -x 退 {unpack.returncode}；路径 {' '.join(paths)}）",
               "看上面 git / tar 的报错：导出范围里的每个路径在那个提交里都要在")
    manifest = manifest_of_tree(new_tree)
    source_fingerprint = fingerprint_of_manifest(manifest)
    toolchain = toolchain_text(new_tree)
    global_text = global_key_text(toolchain, manifest, arguments, new_tree)   # 构建环境的名字里不写路径：在 tree.new 上算与换上之后在 tree 上算是同一份
    global_digest = hashlib.sha256(global_text.encode()).hexdigest()
    fingerprint = version_key(library.home, source_fingerprint, toolchain, global_digest)
    library.source = {"commit": commit, "revision": revision, "fingerprint": fingerprint, "source_fingerprint": source_fingerprint,
                      "toolchain_sha256": hashlib.sha256(toolchain.encode()).hexdigest(), "global_key": global_digest, "files": len(manifest),
                      "export_paths": paths, "exported_at": now_text(), "synced_at_ns": synced_at_ns}
    library.fingerprint = fingerprint
    os.makedirs(os.path.join(library.version_directory(), "logs"), exist_ok=True)
    write_atomically(library.version_file("manifest.tsv"), "".join(f"{manifest[path]}\t{path}\n" for path in sorted(manifest)).encode())
    if not break_is("toolchain-ignored") and not os.path.isfile(library.version_file("toolchain.txt")):
        write_atomically(library.version_file("toolchain.txt"), toolchain.encode())
    write_atomically(library.version_file("global-key.txt"), global_text.encode())
    pointer = (json.dumps(library.source, ensure_ascii=False, indent=1) + "\n").encode()
    if not break_is("sync-pointer-without-pending"):
        write_atomically(library.tree_source_pending, pointer)
    for stale in (library.inflight_record, library.inflight_original, library.inflight_restore):
        if os.path.exists(stale):
            os.remove(stale)
    old_tree = library.tree + ".old"
    remove_tree(old_tree)
    if os.path.lexists(library.tree):
        os.replace(library.tree, old_tree)
    os.replace(new_tree, library.tree)
    remove_tree(old_tree)
    cleared, packages = clear_workspace_fingerprints(library)
    kill_point("sync-tree-after-swap")
    if break_is("sync-pointer-without-pending"):
        write_atomically(library.tree_source, pointer)
    else:
        os.replace(library.tree_source_pending, library.tree_source)
        fsync_directory(library.home)
    library.manifest = manifest
    exported = (f"提交 {commit[:12]}、{len(manifest)} 个文件、指纹 {fingerprint[:16]}（源码指纹 {source_fingerprint[:16]}、工具链 {toolchain.splitlines()[0]}、"
                f"判法 {judge_rules_version()}、全局键 {global_digest[:16]}）；编译目录里工作区 {len(packages)} 个包的 fingerprint 清掉 {cleared} 个")
    if os.path.isfile(library.version_file("list.json")):
        library.load_items()
        say(f"  ✓ 树已导出：{exported}；这一版的清单已经在（{len(library.items)} 条），照用")
        return
    library.items = generate_list(library)
    write_atomically(library.version_file("list.json"), (json.dumps(library.items, ensure_ascii=False, indent=0) + "\n").encode())
    per_package = []
    for package in MUTATED_PACKAGES:
        count = sum(1 for item in library.items if item["package"] == package)
        per_package.append(f"{package} {count} 条")
    say(f"  ✓ 树已导出：{exported}；生成清单 {len(library.items)} 条（{'、'.join(per_package)}）")


# ── 清单：由 cargo-mutants 生成，逐条核对它给的 diff ───────────────────────────────────────────────────────

def offset_of(text_lines, line, column):
    """1 起的行、列（按字符数）在整份文本里的下标；text_lines 是 text.split('\\n')。"""
    return sum(len(text_line) + 1 for text_line in text_lines[:line - 1]) + column - 1


def replace_span(text, start, end, replacement):
    """把 [start, end)（各是 (行, 列)，1 起、列按字符数、end 不含）换成 replacement。"""
    lines = text.split("\n")
    start_offset, end_offset = offset_of(lines, *start), offset_of(lines, *end)
    if not 0 <= start_offset <= end_offset <= len(text):
        raise ValueError(f"区间 {start}–{end} 越出文件")
    return text[:start_offset] + replacement + text[end_offset:]


def apply_unified_diff(original_text, diff_text):
    """照一份 unified diff 改 original_text，交回改后的文本；上下文或删掉的行与原文对不上抛 ValueError。"""
    original, result, position = original_text.split("\n"), [], 0
    lines = diff_text.split("\n")
    index = 0
    while index < len(lines) and not lines[index].startswith("@@"):
        index += 1
    while index < len(lines):
        header = HUNK_HEADER.match(lines[index])
        if header is None:
            raise ValueError(f"认不出的 hunk 头：{lines[index]!r}")
        old_start, old_count = int(header.group(1)), int(header.group(2) if header.group(2) is not None else "1")
        hunk_start = old_start - 1 if old_count > 0 else old_start
        if hunk_start < position:
            raise ValueError("hunk 次序倒了")
        result.extend(original[position:hunk_start])
        position = hunk_start
        index += 1
        while index < len(lines) and not lines[index].startswith("@@"):
            line = lines[index]
            index += 1
            if line.startswith("\\"):
                continue
            if line == "" and index == len(lines):
                break
            tag, body = (line[:1], line[1:]) if line else (" ", "")
            if tag in (" ", "-"):
                if position >= len(original) or original[position] != body:
                    raise ValueError(f"diff 的第 {position + 1} 行与原文对不上")
                if tag == " ":
                    result.append(body)
                position += 1
            elif tag == "+":
                result.append(body)
            else:
                raise ValueError(f"认不出的 diff 行：{line!r}")
    result.extend(original[position:])
    return "\n".join(result)


def raw_mutants_of_package(library, package):
    """一个包的原始清单（cargo mutants --list --json --diff 的那个数组，每项补上 name）。"""
    environment = dict(os.environ, CARGO_TARGET_DIR=library.target)
    fake_driver = test_variable("BROAD_MUTATION_LIST_DRIVER")
    if fake_driver:
        listed = subprocess.run([fake_driver, package], capture_output=True, text=True, cwd=library.tree, stdin=subprocess.DEVNULL)
        if listed.returncode != 0:
            refuse(f"假的清单驱动退 {listed.returncode}：{listed.stderr.strip()}", "看 BROAD_MUTATION_LIST_DRIVER 指的脚本")
        return json.loads(listed.stdout)
    if shutil.which("cargo-mutants") is None:
        refuse("PATH 上没有 cargo-mutants，生成不了清单", "cargo install cargo-mutants 之后重跑 --sync-tree")
    base = ["cargo", "mutants", "--list", "--package", package]
    as_json = subprocess.run(base + ["--json", "--diff"], capture_output=True, text=True, cwd=library.tree, env=environment, stdin=subprocess.DEVNULL)
    if as_json.returncode != 0:
        refuse(f"cargo mutants --list --json --diff --package {package} 退 {as_json.returncode}：{as_json.stderr.strip()[-600:]}",
               f"在 {library.tree} 里手跑这一条看报错；树里的 .cargo/mutants.toml 是导出那个提交的那一份")
    mutants = json.loads(as_json.stdout)
    if all(isinstance(mutant.get("name"), str) for mutant in mutants):
        return mutants
    as_text = subprocess.run(base, capture_output=True, text=True, cwd=library.tree, env=environment, stdin=subprocess.DEVNULL)
    names = [line for line in as_text.stdout.split("\n") if line.strip()]
    if as_text.returncode != 0 or len(names) != len(mutants):
        refuse(f"cargo mutants --list --package {package} 的文本清单（{len(names)} 行，退 {as_text.returncode}）与 JSON 清单（{len(mutants)} 项）条数对不上，变异名配不上",
               "看这一版 cargo-mutants 的 --list 输出有没有多出不是变异名的行；对不上就不生成清单，不拿猜的名字往下跑")
    for mutant, name in zip(mutants, names):
        mutant["name"] = name
    return mutants


def generate_list(library):
    """这一版的固定清单：[{sequence, package, name, file, start, end, replacement}]。每条都拿 cargo-mutants 给的 diff 核一遍
    「照区间换掉」与它自己的改法逐字相同：区间的读法错了会让每一条都改错地方，那样整份结果是假的。"""
    items, seen_names = [], {}
    original_texts = {}
    span_howto = "看 replace_span 对区间的读法（1 起、列按字符、end 不含）与这一版 cargo-mutants 是不是还一致；对不上就不生成清单"
    for package in MUTATED_PACKAGES:
        for mutant in raw_mutants_of_package(library, package):
            file_path, span, replacement = mutant.get("file"), mutant.get("span"), mutant.get("replacement")
            if not isinstance(file_path, str) or not isinstance(span, dict) or not isinstance(replacement, str) or "diff" not in mutant:
                refuse(f"cargo-mutants 给的清单里有一项缺 file / span / replacement / diff：{json.dumps(mutant, ensure_ascii=False)[:300]}",
                       "这一版 cargo-mutants 的 --list --json --diff 输出换了形状：对着 https://mutants.rs/ 改 generate_list 的读法，不照猜的往下跑")
            start = (span["start"]["line"], span["start"]["column"])
            end = (span["end"]["line"], span["end"]["column"])
            if file_path not in original_texts:
                with open(os.path.join(library.tree, file_path), encoding="utf-8") as handle:
                    original_texts[file_path] = handle.read()
            original = original_texts[file_path]
            try:
                mutated = replace_span(original, start, end, replacement + MUTATION_MARKER)
                by_diff = apply_unified_diff(original, mutant["diff"])
            except ValueError as error:
                refuse(f"清单里「{mutant['name']}」这一条套不上原文：{error}", span_howto)
            if mutated == original or mutated.rstrip("\n") != by_diff.rstrip("\n"):
                refuse(f"清单里「{mutant['name']}」照区间换掉之后与 cargo-mutants 自己给的 diff 改出来的不一样（或根本没改动）", span_howto)
            name = clean_field(mutant["name"])
            seen_names[name] = seen_names.get(name, 0) + 1
            if seen_names[name] > 1:
                name = f"{name}（第 {seen_names[name]} 个同名）"
            items.append({"sequence": len(items) + 1, "package": mutant.get("package") or package, "name": name, "file": file_path,
                          "start": list(start), "end": list(end), "replacement": replacement})
    if not items:
        refuse("cargo-mutants 生成的清单是空的", "看树里的 .cargo/mutants.toml 的 examine_globs 与 --package 的包名还对不对得上")
    return items


# ── 忽略用例登记：第二段慢档用例那一步排掉哪些、新加的忽略用例有没有登记 ───────────────────────────────────────

IGNORE_ATTRIBUTE = re.compile(r'^(?P<indent>\s*)#\[ignore(?:\s*=\s*"(?P<reason>(?:[^"\\]|\\.)*)")?\s*\]\s*$')
IGNORE_LIKE_ATTRIBUTE = re.compile(r"^\s*#\[(?:ignore\b|cfg_attr\(.*\bignore\b)")
FUNCTION_LINE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)")
MODULE_OPEN_LINE = re.compile(r"^(?P<indent>\s*)(?:pub(?:\([^)]*\))?\s+)?mod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\{\s*$")
EXCLUSION_WORD = re.compile(r"^[A-Za-z0-9_:/.\-]+$")
FUNCTION_LOOKAHEAD_LINES = 40


def load_exclusions(path=None):
    """排除表：[(二进制 id, 用例名, 理由)]。每一行「<二进制 id><空格><用例名><TAB><理由>」；写坏的、重复的行、表不在都拒（退 1）。"""
    path = path or test_variable("BROAD_MUTATION_EXCLUSIONS") or EXCLUSIONS_FILE
    howto = (f"排除表是 {path}：一行一条，<二进制 id><空格><用例名><TAB><理由>（# 起头的行是注释），二进制 id 与用例名照 cargo nextest list 的写法，"
             "例：singlefs-harness::random_histories shrink_one_failing_seed_from_the_environment")
    if not os.path.isfile(path):
        refuse(f"排除表 {path} 不在", howto, EXIT_INCONSISTENT)
    table = read_table(path, 2)
    problems = []
    if table.malformed_lines:
        problems.append(f"第 {'、'.join(str(number) for number in table.malformed_lines[:5])} 行不是「<二进制 id> <用例名><TAB><理由>」两段")
    if table.partial_tail:
        problems.append("末行没有换行结尾")
    rows, seen = [], set()
    for head, reason in table.rows:
        words = head.split(" ")
        if len(words) != 2 or not all(EXCLUSION_WORD.match(word) for word in words) or not reason.strip():
            problems.append(f"「{head}」这一行的头不是「<二进制 id> <用例名>」两个词（只认字母数字 _ : / . -）、或理由是空的")
            continue
        if tuple(words) in seen:
            problems.append(f"「{head}」重复了")
            continue
        seen.add(tuple(words))
        rows.append((words[0], words[1], reason))
    if problems and not break_is("malformed-exclusion-row-skipped"):
        refuse(f"排除表写坏了：{'；'.join(problems)}", howto, EXIT_INCONSISTENT)
    return rows


def exclusion_filter(exclusions):
    """第二段慢档用例那一步的 -E 表达式：排掉排除表里的每一条；表是空的交 None。"""
    if not exclusions or break_is("exclusion-filter-dropped"):
        return None
    terms = " | ".join(f"(binary_id(={binary_id}) & test(={test}))" for binary_id, test, _reason in exclusions)
    return f"not ({terms})"


def source_files_with_identity(root):
    """harness 档四个包里 src/ 与 tests/ 下的每个 .rs：(路径, 相对路径, 二进制 id, 文件自己在那个二进制里的模块路径)。
    二进制 id 与 cargo nextest 的写法一致：tests/<名>.rs 是 <包>::<名>，lib 的 src 是 <包>，src/bin/<名>.rs 是 <包>::bin/<名>。"""
    found = []
    for package in HARNESS_TIER_PACKAGES:
        for kind in ("src", "tests"):
            base = os.path.join(root, "crates", package, kind)
            for directory, directory_names, file_names in os.walk(base):
                directory_names[:] = sorted(directory_names)
                for file_name in sorted(file_names):
                    if not file_name.endswith(".rs"):
                        continue
                    path = os.path.join(directory, file_name)
                    relative = os.path.relpath(path, base).replace(os.sep, "/")
                    parts = relative[:-3].split("/")
                    if kind == "tests":
                        binary_id, module = f"{package}::{parts[0]}", parts[1:]
                    elif parts == ["main"]:
                        binary_id, module = f"{package}::bin/{package}", []
                    elif parts[0] == "bin" and len(parts) > 1:
                        binary_id, module = f"{package}::bin/{parts[1]}", parts[2:]
                    else:
                        binary_id, module = package, ([] if parts == ["lib"] else parts)
                    if module and module[-1] in ("main", "mod"):
                        module = module[:-1]
                    found.append((path, os.path.relpath(path, root), binary_id, module))
    return found


def ignored_cases_in_file(path, display, binary_id, file_module):
    """一个源文件里每条 #[ignore] 用例：({binary_id, test, reason, where}…)、认不出写法的位置。
    模块路径按缩进配对 `mod x {` 与同缩进的 `}` 取（仓里的源码都过 rustfmt）；用例名是 #[ignore] 之后第一个 fn。"""
    with open(path, encoding="utf-8", errors="replace") as handle:
        lines = handle.read().split("\n")
    cases, unparsed, stack = [], [], []
    for index, line in enumerate(lines):
        stripped = line.strip()
        indent = line[:len(line) - len(line.lstrip())]
        if stack and stripped.startswith("}") and stack[-1][0] == indent:
            stack.pop()
            continue
        opened = MODULE_OPEN_LINE.match(line)
        if opened:
            stack.append((opened.group("indent"), opened.group("name")))
            continue
        if stripped.startswith("//"):
            continue
        if stack and break_is("ignored-scan-skips-nested-modules"):
            continue
        attribute = IGNORE_ATTRIBUTE.match(line)
        if attribute is None:
            if IGNORE_LIKE_ATTRIBUTE.match(line):
                unparsed.append(f"{display}:{index + 1}")
            continue
        name = next((found.group("name") for found in (FUNCTION_LINE.match(later) for later in lines[index + 1:index + 1 + FUNCTION_LOOKAHEAD_LINES]) if found), None)
        if name is None:
            unparsed.append(f"{display}:{index + 1}")
            continue
        cases.append({"binary_id": binary_id, "test": "::".join([*file_module, *(module_name for _indent, module_name in stack), name]),
                      "reason": attribute.group("reason") or "", "where": f"{display}:{index + 1}"})
    return cases, unparsed


class IgnoredClassification(NamedTuple):
    timed: list          # 理由写着「harness 慢档用例：」、不在排除表里：进第二段
    excluded: list       # 在排除表里、理由不是慢档用例：第二段排掉
    unregistered: list   # 两样都不是
    both: list           # 理由写着慢档用例、又在排除表里
    stale: list          # 排除表里指着树里没有的用例：[(二进制 id, 用例名, 理由)]
    unparsed: list       # 认不出写法的 #[ignore]

    def problems(self):
        def first(cases):
            case = cases[0]
            return f"例：{case['binary_id']} {case['test']}（{case['where']}，理由「{case['reason'][:30]}」）"
        problems = []
        if self.unregistered:
            problems.append(f"{len(self.unregistered)} 条 #[ignore] 用例既不是「{TIMED_CASE_REASON_PREFIX}」开头的理由、也不在排除表里，{first(self.unregistered)}")
        if self.both:
            problems.append(f"{len(self.both)} 条用例理由写着「{TIMED_CASE_REASON_PREFIX}」又在排除表里，{first(self.both)}")
        if self.stale:
            problems.append(f"排除表里 {len(self.stale)} 行指着树里没有的用例，例：{self.stale[0][0]} {self.stale[0][1]}")
        if self.unparsed:
            problems.append(f"{len(self.unparsed)} 处 #[ignore] 写法认不出（要单独一行的 #[ignore] 或 #[ignore = \"理由\"]，后面 {FUNCTION_LOOKAHEAD_LINES} 行内要有 fn），例：{self.unparsed[0]}")
        return problems

    def line(self):
        return (f"BROAD_MUTATION_IGNORED timed={len(self.timed)} excluded={len(self.excluded)} unregistered={len(self.unregistered)} "
                f"stale={len(self.stale)} both={len(self.both)} unparsed={len(self.unparsed)}")


def classify_ignored_cases(root, exclusions):
    cases, unparsed = [], []
    for path, display, binary_id, module in source_files_with_identity(root):
        found, bad = ignored_cases_in_file(path, display, binary_id, module)
        cases += found
        unparsed += bad
    names = {(case["binary_id"], case["test"]) for case in cases}
    excluded_names = {(binary_id, test) for binary_id, test, _reason in exclusions}
    timed, excluded, unregistered, both = [], [], [], []
    for case in cases:
        is_timed = case["reason"].startswith(TIMED_CASE_REASON_PREFIX)
        is_excluded = (case["binary_id"], case["test"]) in excluded_names
        (both if is_timed and is_excluded else timed if is_timed else excluded if is_excluded else unregistered).append(case)
    stale = [row for row in exclusions if (row[0], row[1]) not in names]
    if break_is("unregistered-ignored-passes"):
        unregistered = []
    if break_is("stale-exclusion-passes"):
        stale, both = [], []
    return IgnoredClassification(timed, excluded, unregistered, both, stale, unparsed)


def check_ignored_registration(library, exclusions):
    """这一版树里 harness 档的每条 #[ignore] 用例都登记了：慢档用例进第二段、大档与工具在排除表里；排除表不指着不存在的。不对就拒（退 1）。"""
    classified = classify_ignored_cases(library.tree, exclusions)
    say(classified.line())
    problems = classified.problems()
    if problems:
        refuse(f"这一版树里的忽略用例登记对不上：{'；'.join(problems)}",
               f"新加的忽略用例：是 harness 慢档用例就把 #[ignore] 的理由写成「{TIMED_CASE_REASON_PREFIX}…」开头（它进第二段）；是按环境变量起的大档或工具，"
               f"就在 {EXCLUSIONS_FILE} 加一行（<二进制 id> <用例名><TAB><理由>，名字照 cargo nextest list）。排除表指着不存在的用例：删掉那一行或改对名字"
               f"（用例是比这棵树（提交 {str(library.source.get('commit'))[:12]}）更新的提交才有的，就先 --sync-tree 到那个提交）",
               EXIT_INCONSISTENT)
    return classified


def check_ignored_command(library, arguments):
    """--check-ignored：只判登记，再打第二段慢档用例那一步的完整命令（看 -E 排掉了哪几条）。"""
    exclusions = load_exclusions()
    check_ignored_registration(library, exclusions)
    step = next(step for step in step_table(arguments, exclusions) if step[0] in (SLOW_STEP_NAME, OLD_SLOW_STEP_NAME))
    say(f"STEP_COMMAND\t{step[0]}\t{shlex.join(step[3])}")
    return 0


# ── 每一步命令 ─────────────────────────────────────────────────────────────────────────────────────────────

def package_arguments(packages):
    return [word for package in packages for word in ("-p", package)]


def step_table(arguments, exclusions):
    """每一步：(步名, 哪一段, build / test, 命令, 测试没过的退出码)。
    第二段只补第一段没跑到的：文档测试（被变异的两个包）、harness 档四个包里标了 #[ignore] 的慢档用例（与第一段同一种起法，加 --run-ignored only 只跑它们，
    再用 -E 排掉排除表里按环境变量起的大档与工具，exclusions 是排除表的行）、checker 档快档。第一段跑过的不标 #[ignore] 的用例第二段不再跑。"""
    harness = package_arguments(HARNESS_TIER_PACKAGES)
    threads = str(arguments.threads)
    sync_free = ["eatmydata"]
    stage_two_prefix = sync_free if arguments.stage_two_eatmydata else []
    checker_tier = ["--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]
    filterset = exclusion_filter(exclusions)
    harness_build = ["cargo", "nextest", "run", "--locked", *harness, "--no-run"]
    harness_test = sync_free + ["cargo", "nextest", "run", "--locked", *harness, "-j", threads]
    stage_one = [
        ("one-build", "one", "build", harness_build, None),
        ("one-test", "one", "test", harness_test, NEXTEST_TEST_FAILED_EXIT),
    ]
    if break_is("stage-two-full-cargo-test"):
        return stage_one + [
            ("two-build-debug", "two", "build", ["cargo", "test", "--locked", *harness, "--no-run"], None),
            ("two-test-debug", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *harness], CARGO_FAILED_EXIT),
            ("two-build-checker-tier", "two", "build", ["cargo", "test", "--locked", *checker_tier, "--no-run"], None),
            ("two-test-checker-tier", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *checker_tier], CARGO_FAILED_EXIT),
        ]
    return stage_one + [
        ("two-build-harness", "two", "build", harness_build, None),
        ("two-test-doc", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", "--doc", *package_arguments(MUTATED_PACKAGES)], CARGO_FAILED_EXIT),
        (OLD_SLOW_STEP_NAME if break_is("slow-step-keeps-old-name") else SLOW_STEP_NAME, "two", "test",
         harness_test + ["--run-ignored", "only"] + (["-E", filterset] if filterset else []), NEXTEST_TEST_FAILED_EXIT),
        ("two-build-checker-tier", "two", "build", ["cargo", "test", "--locked", *checker_tier, "--no-run"], None),
        ("two-test-checker-tier", "two", "test", stage_two_prefix + ["cargo", "test", "--locked", *checker_tier], CARGO_FAILED_EXIT),
    ]


def descendants_of(process_id):
    """一个进程的全部后代进程号（读 /proc/<pid>/task/<tid>/children，逐层往下）；读不到的那一支略过。"""
    found, pending = [], [process_id]
    while pending:
        current = pending.pop()
        try:
            task_directory = f"/proc/{current}/task"
            for task in os.listdir(task_directory):
                with open(f"{task_directory}/{task}/children", encoding="utf-8") as handle:
                    children = [int(word) for word in handle.read().split()]
                found += children
                pending += children
        except (OSError, ValueError):
            continue
    return found


def own_scope_unit(wrapper_process_id):
    """内存包装给这一步开的 scope 的 unit 名：在包装的后代里找 cgroup 落在 singlefs-memory-cap-<包装的进程号>-….scope 里的；找不到交 None。
    名字里的进程号要正好是自己起的那个包装：别人开的 scope（驱动自己被包在外层的那一个也算别人的）不认。"""
    for process_id in descendants_of(wrapper_process_id):
        try:
            with open(f"/proc/{process_id}/cgroup", encoding="utf-8") as handle:
                lines = handle.read().splitlines()
        except OSError:
            continue
        for line in lines:
            found = OWN_SCOPE_UNIT.search(line)
            if found and found.group(2) == str(wrapper_process_id):
                return found.group(1)
    return None


def progress_line(sequence, step_name, elapsed, log_path):
    """一步跑着时的一行进度（文件头「输出」）：哪一条、哪一步、跑了多久，与这一步此刻的日志里数得出的已编 crate、已过与没过的用例。"""
    label = "baseline" if sequence == BASELINE_SEQUENCE else sequence
    compiled = passed = failed = 0
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line = line.rstrip("\n")
                if COMPILING_LINE.match(line):
                    compiled += 1
                elif NEXTEST_PASS_LINE.match(line) or LIBTEST_PASS_LINE.match(line):
                    passed += 1
                elif NEXTEST_FAILURE_LINE.match(line) or LIBTEST_FAILURE_LINE.match(line):
                    failed += 1
    except OSError:
        pass
    return (f"BROAD_MUTATION_PROGRESS sequence={label} step={step_name} elapsed_seconds={elapsed:.0f} "
            f"compiled_crates={compiled} tests_passed={passed} tests_failed={failed}")


class Escalation(NamedTuple):
    exit_code: int          # 交回调用方接着判的退出码：升档那趟的；没升档、或升档那趟没跑成时是原来那一趟的
    seconds: float          # 升档那趟的秒数；0 是没升档（这一步没有整步限时、或上限写法认不出）
    label: str              # 证据齐的抓到：caught-by-timeout / caught-by-memory；否则空
    note: str               # 记进结果行备注的一句
    environmental: str      # 照环境型结局记、没结论：timeout / memory-cap；否则空


def memory_cap_bytes(text):
    """run-with-memory-cap.sh 的上限写法（正整数加 K / M / G / T，1024 进制）→ 字节；认不出交 None。"""
    found = MEMORY_CAP_FORM.match(text or "")
    return int(found.group(1)) * MEMORY_CAP_UNITS[found.group(2)] if found else None


def wrapper_hit_text(exit_code, limit_text):
    """包装撞上限时往 stderr 打的那一句里认的一段（research/scripts/run-with-memory-cap.sh 文件头「退出码」的 253、250）。"""
    return f"超过限时 {limit_text} 秒" if exit_code == TIME_LIMIT_HIT_EXIT else f"撞了内存上限 {limit_text}"


def escalation_evidence_problems(first_exit, escalated_exit, baseline, first_amount, escalated_amount, first_text, escalated_text, first_log_text, escalated_log_text):
    """文件头「升档」①–③：证据缺哪几样（空表是证据齐）。baseline 是基线秒数或峰值字节（没记是 None），amount 是两趟的限时秒数或上限字节。"""
    problems = []
    if escalated_exit != first_exit:
        problems.append(f"两趟结局不同（原来退 {first_exit}、升档那趟退 {escalated_exit}）")
    multiple = ESCALATION_TIME_BASELINE_MULTIPLE if first_exit == TIME_LIMIT_HIT_EXIT else ESCALATION_MEMORY_BASELINE_MULTIPLE
    if baseline is None:
        problems.append("这一版这个线程数下没有这一步在没改过的树上的基线（超时要 baseline.tsv 的绿记录，撞顶要那一行记的峰值）")
    elif escalated_amount is None or escalated_amount < multiple * baseline:
        problems.append(f"升档那趟的 {escalated_text} 不到基线 {baseline:g} 的 {multiple} 倍")
    if escalated_amount is None or first_amount is None or escalated_amount <= first_amount:
        problems.append(f"升档那趟的 {escalated_text} 没比原来的 {first_text} 大")
    if wrapper_hit_text(first_exit, first_text) not in first_log_text:
        problems.append(f"原来那趟的日志里没有包装打的「{wrapper_hit_text(first_exit, first_text)}」")
    if wrapper_hit_text(escalated_exit, escalated_text) not in escalated_log_text:
        problems.append(f"升档那趟的日志里没有包装打的「{wrapper_hit_text(escalated_exit, escalated_text)}」")
    return problems


class Runner:
    def __init__(self, library, arguments):
        self.library, self.arguments = library, arguments
        self.exclusions = [] if arguments.skip_stage_two else load_exclusions()     # 只跑第一段的趟用不着排除表，表坏了也不挡它
        self.steps = step_table(arguments, self.exclusions)
        self.fake_driver = test_variable("BROAD_MUTATION_STEP_DRIVER")
        self.fake_driver_via_wrapper = test_variable("BROAD_MUTATION_STEP_VIA_WRAPPER") == "1"
        self.baseline_seconds = {}
        self.stages_with_fresh_baseline = set()
        self.pruner = None
        self.runner_path = None

    def times_path(self, step_name):
        return self.library.version_file(f"test-times.{self.arguments.threads}.{step_name}.tsv")

    def limits_path(self):
        return self.library.version_file(f"limits.{self.arguments.threads}.tsv")

    def prepare_limits(self):
        """C4：基线逐条（checker 档逐个二进制）的 CPU 秒 → 上限表（倍数 × CPU + 下限，墙钟兜底另算），runner 在 limit 模式读它。"""
        rules = prune_rules()
        times = {}
        for step_name in RUNNER_TEST_STEPS:
            times.update(rules.read_times(self.times_path(step_name)))
        rules.write_limits(self.limits_path(), rules.limits_table(times, self.arguments.cpu_multiplier, self.arguments.cpu_floor,
                                                                  self.arguments.wall_multiplier, self.arguments.wall_floor))

    def command_for(self, step, selection):
        """这一条这一步的命令与「跑了哪些」的一句：按覆盖挑出来的测试（C2）、第二段按二进制（E6）；交 (None, …) 是这一步没有要跑的。"""
        name, command = step[0], list(step[3])
        selection = selection or {}
        if name == "one-test" and selection.get("one") is not None:
            if not selection["one"]:
                return None, "none"
            filters = prune_rules().nextest_filters(selection["one"])
            return command + [word for text in filters for word in ("-E", text)], f"{len(selection['one'])} tests"
        if name == SLOW_STEP_NAME and selection.get("slow") is not None:
            if not selection["slow"]:
                return None, "none"
            binaries = " | ".join(f"binary_id(={binary})" for binary in selection["slow"])
            if "-E" in command:
                command[command.index("-E") + 1] = f"({binaries}) & {command[command.index('-E') + 1]}"
            else:
                command += ["-E", f"({binaries})"]
            return command, ",".join(selection["slow"])
        if name == "two-test-checker-tier" and selection.get("checker") is not None:
            if not selection["checker"]:
                return None, "none"
            command = [word for word in command if word not in ("--lib", "--tests")]
            for target in selection["checker"]:
                command += ["--lib"] if target == "lib" else ["--test", target]
            return command, ",".join(selection["checker"])
        if name == "two-test-doc" and break_is("doc-tests-skipped"):
            return None, "none"
        return command, "all"

    def ensure_coverage(self, stages):
        """C2：这一版在没改过的树上的覆盖（第一段逐条测试、第二段逐个二进制）；没有就现跑，跑红拒（退 6）。"""
        for stage in stages:
            name = f"coverage-{stage}"
            out = self.library.version_file(f"{name}.json")
            if os.path.isfile(out):
                continue
            llvm = llvm_bin(self.arguments)
            if not self.fake_driver and not all(os.path.isfile(os.path.join(llvm, tool)) for tool in ("llvm-profdata", "llvm-cov")):
                refuse(f"按覆盖挑测试要 llvm-profdata 与 llvm-cov，{llvm or 'rustc 的 sysroot'} 下没有",
                       "装 rustup 组件：rustup component add llvm-tools（与编译器同一份 LLVM，profraw 格式对得上）；装在别处就加 --llvm-bin <目录>")
            specification = self.library.version_file(f"{name}.spec.json")
            write_atomically(specification, json.dumps({"kind": stage, "tree": self.library.tree, "target": os.path.join(self.library.home, "target-coverage"),
                                                        "scratch": self.library.scratch, "llvm_bin": llvm, "workers": self.arguments.threads,
                                                        "out": out}).encode())
            say(f"  · 覆盖 {name}：在没改过的树上插桩跑一遍（这一版还没有）")
            step = (name, stage, "test", ["python3", SELF_PATH, "--coverage-step", specification], None)
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, None, extra={"BROAD_MUTATION_STEP_SPEC": specification})
            if exit_code != 0 or not os.path.isfile(out):
                refuse(f"覆盖 {name} 在没改过的树上退 {exit_code}：拿不到「函数 → 覆盖到它的测试」，一条变异都不跑",
                       f"看日志 {self.log_path(BASELINE_SEQUENCE, name)}（插桩下跑红的测试逐条列在里面）；是环境就修环境重跑同一条命令")
            say(f"  · 覆盖 {name} 好了，{seconds:.1f} 秒")

    def step_named(self, step_name):
        return next(step for step in self.steps if step[0] == step_name)

    def log_path(self, sequence, step_name):
        prefix = "baseline" if sequence == BASELINE_SEQUENCE else str(sequence)
        return os.path.join(self.library.version_directory(), "logs", f"{prefix}.{step_name}.log")

    def time_limit_of(self, step_name):
        """整步的墙钟兜底：编译步与文档测试步照旧；经 runner 的测试步按单条测试的 CPU 时间限（C4），不再设整步总时限。"""
        if step_name in RUNNER_TEST_STEPS and not break_is("whole-step-time-limit"):
            return None
        return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))

    def stop_step(self, child):
        """驱动被 TERM / INT 停：在途这一步连同内存包装给它开的 scope 一起停，不留着它跑完。
        包装的 bash 在等前台子进程时把自己的 TERM 处理推到子进程退出之后，只给它发 TERM 那一步照样跑；所以另停它开的那一个 scope
        （systemctl --user stop，systemd 给 scope 里每个进程发 TERM），到点还在就 KILL 那个 scope 与包装。发信号只点自己起的：child 与名字里带 child 进程号的 scope。"""
        grace = self.arguments.stop_grace
        unit = None if break_is("term-leaves-step-running") else own_scope_unit(child.pid)
        child.terminate()   # 只点自己起的这一个进程
        if break_is("term-leaves-step-running"):
            try:
                child.wait(timeout=grace)
            except subprocess.TimeoutExpired:
                pass
            return
        if unit is not None:
            try:
                subprocess.run(["systemctl", "--user", "stop", unit], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                               timeout=grace)
            except (OSError, subprocess.TimeoutExpired):
                subprocess.run(["systemctl", "--user", "kill", "--signal=KILL", unit], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, check=False)
        try:
            child.wait(timeout=grace)
        except subprocess.TimeoutExpired:
            child.kill()    # 只点自己起的这一个进程
            child.wait()

    def progress_interval(self):
        fake = test_variable("BROAD_MUTATION_PROGRESS_INTERVAL")
        return float(fake) if fake else PROGRESS_INTERVAL_SECONDS

    def wait_reporting_progress(self, child, step_name, sequence, started):
        """等这一步退出，交回它的退出码；跑着时每过一个进度间隔往 stderr 报一行 BROAD_MUTATION_PROGRESS（progress_line）。"""
        interval = self.progress_interval()
        next_report = started + interval
        while True:
            if break_is("progress-never"):
                return child.wait()
            try:
                return child.wait(timeout=max(0.0, next_report - time.monotonic()))
            except subprocess.TimeoutExpired:
                pass
            now = time.monotonic()
            note(progress_line(sequence, step_name, now - started, self.log_path(sequence, step_name)))
            while next_report <= now:
                next_report += interval

    def baseline_peaks_path(self):
        """基线那一步跑的时候包装记峰值的那一份（RUN_WITH_MEMORY_CAP_PEAKS，包装的峰值表写法，键 BASELINE_PEAK_KEY_PREFIX + 步名）：
        每一步开跑之前删掉，跑完读出来写进 baseline.tsv 那一行的 peak_bytes（文件头「基线」），之后不再读它。"""
        return self.library.version_file(f"baseline-peaks.{self.arguments.threads}.tsv")

    def wrapper_peak(self, step_name):
        """包装刚给这一步记下的峰值（字节）；没记交 None。"""
        try:
            with open(self.baseline_peaks_path(), encoding="utf-8") as handle:
                for line in handle:
                    fields = line.rstrip("\n").split("\t")
                    if line.startswith("#") or len(fields) < 4 or fields[3] != BASELINE_PEAK_KEY_PREFIX + step_name:
                        continue
                    return int(fields[0]) if fields[0].isdigit() else None
        except OSError:
            return None
        return None

    def baseline_rows_of(self, step_name):
        """这一版 baseline.tsv 里这个线程数下这一步的行：[(行号, {列名: 值})]。弄坏开关 baseline-any-threads 下不看线程数（改前的取法）。"""
        threads = str(self.arguments.threads)
        return [(number, row) for number, row in read_baseline_rows(self.library.version_file("baseline.tsv"))
                if row["step"] == step_name and (row["threads"] == threads or break_is("baseline-any-threads"))]

    def baseline_row(self, step_name):
        """升档取的基线（文件头「升档」①）：这一版这个线程数下这一步最后一行绿记录，交 (行号, {列名: 值})；没有交 None。"""
        rows = [(number, row) for number, row in self.baseline_rows_of(step_name) if row["outcome"] == "ok"]
        return rows[-1] if rows else None

    def baseline_peak(self, step_name):
        """这一版这个线程数下这一步在没改过的树上的内存峰值（字节，baseline_row 那一行的 peak_bytes）；没记过交 None。"""
        found = self.baseline_row(step_name)
        return int(found[1]["peak_bytes"]) if found and found[1]["peak_bytes"].isdigit() else None

    def recorded_baseline_seconds(self, step_name):
        """这一版这个线程数下这一步最后一行绿记录的秒数（原样的文字）；没有交 None。"""
        found = self.baseline_row(step_name)
        return found[1]["seconds"] if found else None

    def run_step(self, step, sequence, time_limit, command=None, selection_text="all", runner_mode=None, extra=None, memory_cap=None, attempt="first"):
        """跑一步：交 (退出码, 秒数)。输出落进这一条这一步的日志。command 不给就用步表里的；runner_mode 给了就经 runner 起测试二进制（measure / limit）。
        memory_cap 不给取 --memory-cap（升档那趟给升过的）；attempt 是 first 或 escalated（只交给假的一步）。基线那几步另让包装把峰值记进这一版的 baseline-peaks。"""
        step_name, _stage, _kind, table_command, _test_failed_exit = step
        command = table_command if command is None else command
        memory_cap = memory_cap or self.arguments.memory_cap
        remove_tree(self.library.scratch)
        os.makedirs(self.library.scratch)
        os.makedirs(self.library.target, exist_ok=True)
        os.makedirs(os.path.join(self.library.version_directory(), "logs"), exist_ok=True)
        environment = dict(os.environ, CARGO_TARGET_DIR=self.library.target, TMPDIR=self.library.scratch, CARGO_TERM_COLOR="never", **(extra or {}))
        if runner_mode and self.runner_path and not break_is("cpu-limit-ignored"):
            environment[f"CARGO_TARGET_{host_triple(self.library).upper().replace('-', '_')}_RUNNER"] = self.runner_path
            environment["BROAD_MUTATION_RUNNER_MODE"] = runner_mode
            environment["BROAD_MUTATION_RUNNER_LIMITS"] = self.limits_path()
            environment["BROAD_MUTATION_RUNNER_TIMES"] = self.times_path(step_name)
        if self.fake_driver:
            environment["BROAD_MUTATION_STEP_SELECTION"] = selection_text
            environment["BROAD_MUTATION_STEP_TIME_LIMIT"] = str(time_limit)
            environment["BROAD_MUTATION_STEP_MEMORY_CAP"] = memory_cap
            environment["BROAD_MUTATION_STEP_ATTEMPT"] = attempt
        is_baseline = sequence == BASELINE_SEQUENCE
        if is_baseline:
            environment["RUN_WITH_MEMORY_CAP_PEAKS"] = self.baseline_peaks_path()
            environment["RUN_WITH_MEMORY_CAP_KEY"] = BASELINE_PEAK_KEY_PREFIX + step_name
        # 文件头「内存」：升档那趟按它自己的上限排队；改坏了源码的那几步撞 slice 总上限时先挨杀（包装的 oom_score_adj）；基线与验环境那几步两样都不设
        environment.pop("RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP", None)
        environment.pop("RUN_WITH_MEMORY_CAP_OOM_FIRST", None)
        if attempt == "escalated":
            if not break_is("escalation-queues-by-peak"):
                environment["RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP"] = ESCALATED_QUEUE_REASON
        elif sequence not in (BASELINE_SEQUENCE, ENVIRONMENT_SEQUENCE) and not break_is("mutation-steps-not-oom-first"):
            environment["RUN_WITH_MEMORY_CAP_OOM_FIRST"] = MUTATION_STEP_OOM_REASON
        if self.fake_driver and not self.fake_driver_via_wrapper:
            argv = [self.fake_driver, step_name, str(sequence), self.library.tree]
        else:
            if self.fake_driver:
                argv = ["bash", MEMORY_CAP_RUNNER, "64M", self.fake_driver, step_name, str(sequence), self.library.tree]
            else:
                argv = ["bash", MEMORY_CAP_RUNNER, memory_cap, "bash", THREAD_CAP_RUNNER, str(self.arguments.threads), *command]
            environment["RUN_WITH_MEMORY_CAP_KEY"] = BASELINE_PEAK_KEY_PREFIX + step_name if is_baseline else f"broad-mutation {step_name}"
            environment.pop("RUN_WITH_MEMORY_CAP_TIME_LIMIT", None)
            if time_limit is not None:
                environment["RUN_WITH_MEMORY_CAP_TIME_LIMIT"] = str(time_limit)
        started = time.monotonic()
        with open(self.log_path(sequence, step_name), "wb") as log:
            child = subprocess.Popen(argv, cwd=self.library.tree, env=environment, stdin=subprocess.DEVNULL, stdout=log,
                                     stderr=subprocess.STDOUT, pass_fds=(self.library.lock_descriptor,))
            try:
                exit_code = self.wait_reporting_progress(child, step_name, sequence, started)
            except Stopped:
                self.stop_step(child)
                raise
        return exit_code, time.monotonic() - started

    def append_evidence_line(self, relative_log, sequence, step_name, attempt, limit_text, exit_code):
        """升档那两趟的日志末尾追加驱动的那一行（文件头「升档」④）：版本键、序号、步名、哪一趟、那一趟的限时或上限、退出码。"""
        path = self.library.version_file(relative_log)
        with open(path, "rb") as handle:
            existing = handle.read()
        with open(path, "ab") as handle:
            handle.write((b"" if not existing or existing.endswith(b"\n") else b"\n")
                         + f"{ESCALATION_EVIDENCE_PREFIX} version={self.library.fingerprint} sequence={sequence} step={step_name} attempt={attempt} "
                           f"limit={limit_text} exit={exit_code}\n".encode())
            handle.flush()
            os.fsync(handle.fileno())

    def escalate_step(self, item, step, command, selection_text, runner_mode, first_exit):
        """文件头「升档」：这一条这一步退 253（超时）或 250（撞顶），当场在同一棵改过的树上升档重跑那一步一次，交 Escalation。
        两趟都超时 / 都撞顶而证据齐的往库目录登记表追加一行、交标签；证据不齐的交环境型结局；升档那趟另有结局的交它的退出码给调用方照常判。"""
        step_name, sequence = step[0], item["sequence"]
        first_time_limit = self.time_limit_of(step_name)
        first_cap = self.arguments.memory_cap
        same_limit = break_is("escalation-same-limit")
        baseline_entry = self.baseline_row(step_name)     # 登记行记它的行号（baseline_row），门禁按行号找那一行复核
        baseline_row_field = str(baseline_entry[0]) if baseline_entry else ""
        if first_exit == TIME_LIMIT_HIT_EXIT:
            if first_time_limit is None:
                return Escalation(first_exit, 0.0, "", f"{step_name} 没有整步限时（经 runner 按单条限 CPU），不升档", "")
            baseline_text = self.recorded_baseline_seconds(step_name)
            baseline = float(baseline_text) if baseline_text else None
            escalated = first_time_limit if same_limit else int(math.ceil(max(ESCALATION_TIME_FACTOR * first_time_limit,
                                                                               ESCALATION_TIME_BASELINE_MULTIPLE * (baseline or 0.0))))
            time_limit, memory_cap = escalated, first_cap
            first_text, escalated_text, first_amount, escalated_amount = str(first_time_limit), str(escalated), first_time_limit, escalated
            baseline_field, label, kind_text, multiple = baseline_text or "", ESCALATED_TIMEOUT_LABEL, "超时", ESCALATION_TIME_BASELINE_MULTIPLE
        else:
            first_bytes = memory_cap_bytes(first_cap)
            if first_bytes is None:
                return Escalation(first_exit, 0.0, "", f"--memory-cap {first_cap} 的写法认不出，不升档", "")
            baseline = self.baseline_peak(step_name)
            if same_limit:
                escalated_text = first_cap
            else:
                wanted = max(ESCALATION_MEMORY_FACTOR * first_bytes, ESCALATION_MEMORY_BASELINE_MULTIPLE * (baseline or 0))
                escalated_text = f"{int(math.ceil(wanted / MEMORY_CAP_UNITS['M']))}M"
            time_limit, memory_cap = first_time_limit, escalated_text
            first_text, first_amount, escalated_amount = first_cap, first_bytes, memory_cap_bytes(escalated_text)
            baseline_field, label, kind_text, multiple = ("" if baseline is None else str(baseline)), ESCALATED_MEMORY_LABEL, "撞顶", ESCALATION_MEMORY_BASELINE_MULTIPLE
        os.makedirs(self.library.version_file(ESCALATION_DIRECTORY), exist_ok=True)
        stem = f"{sequence}.{step_name}.{time.time_ns()}"
        first_log = os.path.join(ESCALATION_DIRECTORY, f"{stem}.first.log")
        os.replace(self.log_path(sequence, step_name), self.library.version_file(first_log))
        self.append_evidence_line(first_log, sequence, step_name, "first", first_text, first_exit)
        exit_code, seconds = self.run_step(step, sequence, time_limit, command=command, selection_text=selection_text, runner_mode=runner_mode,
                                           memory_cap=memory_cap, attempt="escalated")
        say(f"BROAD_MUTATION_ESCALATION sequence={sequence} step={step_name} first={first_text} escalated={escalated_text} exit={exit_code}")
        upgraded = f"{step_name} {kind_text}之后升档重跑（{first_text} → {escalated_text}）"
        first_environmental = "timeout" if first_exit == TIME_LIMIT_HIT_EXIT else "memory-cap"
        if exit_code not in (TIME_LIMIT_HIT_EXIT, MEMORY_CAP_HIT_EXIT):
            if exit_code in WRAPPER_INFRASTRUCTURE_EXITS:
                return Escalation(first_exit, seconds, "", f"{upgraded}：升档那趟没跑成（包装退 {exit_code}，{WRAPPER_INFRASTRUCTURE_EXITS[exit_code]}），照原结局记、没结论",
                                  first_environmental)
            if exit_code == 0:
                os.remove(self.library.version_file(first_log))
            return Escalation(exit_code, seconds, "", upgraded, "")
        escalated_log = os.path.join(ESCALATION_DIRECTORY, f"{stem}.escalated.log")
        os.replace(self.log_path(sequence, step_name), self.library.version_file(escalated_log))
        self.append_evidence_line(escalated_log, sequence, step_name, "escalated", escalated_text, exit_code)
        texts = []
        for relative in (first_log, escalated_log):
            with open(self.library.version_file(relative), encoding="utf-8", errors="replace") as handle:
                texts.append(handle.read())
        problems = escalation_evidence_problems(first_exit, exit_code, None if not baseline_field else float(baseline_field), first_amount, escalated_amount,
                                                first_text, escalated_text, texts[0], texts[1])
        if problems and not break_is("escalation-evidence-unchecked"):
            return Escalation(exit_code, seconds, "", f"{upgraded}，仍{'超时' if exit_code == TIME_LIMIT_HIT_EXIT else '撞顶'}；证据不齐、没结论：{'；'.join(problems)}",
                              "timeout" if exit_code == TIME_LIMIT_HIT_EXIT else "memory-cap")
        append_row(os.path.join(self.library.home, ESCALATION_REGISTRY), ESCALATION_COLUMNS,
                   [self.library.fingerprint, sequence, item["name"], label, step_name, baseline_field, baseline_row_field, self.arguments.threads, first_text,
                    escalated_text, multiple,
                    first_log, file_digest(self.library.version_file(first_log)), escalated_log, file_digest(self.library.version_file(escalated_log)), now_text()])
        return Escalation(exit_code, seconds, label, f"{upgraded}，两趟都{kind_text}（基线 {baseline_field or '没记'}，要不低于它的 {multiple} 倍）："
                                                     f"证据登记在 {ESCALATION_REGISTRY}", "")

    def stop_for_infrastructure(self, step_name, sequence, exit_code):
        refuse(f"第 {sequence} 条的 {step_name} 退 {exit_code}（{WRAPPER_INFRASTRUCTURE_EXITS[exit_code]}）：这是内存包装自己的结局，不是这条变异的结局，不落行",
               f"看日志 {self.log_path(sequence, step_name)}（bash research/scripts/run-with-memory-cap.sh --status 看谁占着），"
               "修好或等别的重活退了，重跑同一条命令接着跑", EXIT_INFRASTRUCTURE)

    def ensure_baseline(self, stage):
        """这一版这一段的每一步在这个线程数下、没改过的树上都跑绿过；没有就现跑（不限时：基线就是用来标定超时的）。
        baseline.tsv 只追加，一行记步名、结局、秒数、时刻、线程数与这一步的内存峰值（文件头「基线」）。"""
        ran_every_step = True
        for step in self.steps:
            step_name, step_stage = step[0], step[1]
            if step_stage != stage:
                continue
            rows = self.baseline_rows_of(step_name)
            row = rows[-1][1] if rows else None
            measured = step_name not in RUNNER_TEST_STEPS or self.fake_driver or os.path.isfile(self.times_path(step_name))
            # 升档的证据要这一步在没改过的树上的内存峰值（文件头「升档」①）：真跑时这个线程数下没记过的重跑这一步的基线；假的一步不经包装、不记峰值
            peak_known = self.fake_driver or (row is not None and row["peak_bytes"].isdigit())
            if row is not None and row["outcome"] == "ok" and measured and peak_known:
                self.baseline_seconds[step_name] = float(row["seconds"])
                ran_every_step = False
                continue
            say(f"  · 基线 {step_name}：在没改过的树上跑（这一版还没有它的绿记录，或这个线程数下还没量过逐条的 CPU 时间、还没记过内存峰值）")
            runner_mode = "measure" if step_name in RUNNER_TEST_STEPS else None
            if runner_mode and os.path.exists(self.times_path(step_name)):
                os.remove(self.times_path(step_name))
            command = list(step[3]) + (["--no-fail-fast"] if step_name == "two-test-checker-tier" else [])
            if os.path.exists(self.baseline_peaks_path()):
                os.remove(self.baseline_peaks_path())     # 包装只记这一步这一趟的峰值，读完就不再用（不读上一趟留下的）
            exit_code, seconds = self.run_step(step, BASELINE_SEQUENCE, None, command=command, runner_mode=runner_mode)
            outcome = "ok" if exit_code == 0 else f"exit-{exit_code}"
            peak = self.wrapper_peak(step_name)
            append_row(self.library.version_file("baseline.tsv"), BASELINE_COLUMNS,
                       [step_name, outcome, f"{seconds:.1f}", now_text(), self.arguments.threads, "" if peak is None else peak])
            if exit_code != 0 and stage == "two" and not break_is("stage-two-baseline-red-generic"):
                log_path = self.log_path(BASELINE_SEQUENCE, step_name)
                failing = []
                try:
                    with open(log_path, encoding="utf-8", errors="replace") as handle:
                        for line in handle:
                            found = LIBTEST_FAILURE_LINE.match(line.rstrip("\n")) or NEXTEST_FAILURE_LINE.match(line.rstrip("\n"))
                            if found and found.group(1) not in failing:
                                failing.append(found.group(1))
                except OSError:
                    pass
                refuse(f"第二段基线 {step_name} 在没改过的树上退 {exit_code}，红的测试：{'、'.join(failing[:8]) or UNRECOGNISED_TEST}；第二段一条结论都给不出",
                       "已知的一条：crates/singlefs-checker-tier/src/crash_identity.rs 里读 .claude/gate.d/stage-inputs.tsv（导出树之外）的那条 lib 测试，"
                       "要实现员改成不读导出树之外的文件（或改门禁 checker-tier-crates-mutation-replay 登记的拷贝范围把登记表并进导出树），修好的提交再 --sync-tree；"
                       f"在那之前只要第一段的结论就加 --skip-stage-two（第一段没红的记「只过了第一段」，不算定论）；全文看日志 {log_path}",
                       EXIT_BASELINE_RED)
            if exit_code != 0:
                refuse(f"基线 {step_name} 在没改过的树上退 {exit_code}：基线不绿，后面每一条「抓到」都不作数，一条变异都不跑",
                       f"看日志 {self.log_path(BASELINE_SEQUENCE, step_name)}；代码本来就红就先修代码再 --sync-tree，是环境（缺工具、内存包装退 250–254）就修环境，重跑同一条命令",
                       EXIT_BASELINE_RED)
            self.baseline_seconds[step_name] = seconds
            limit = self.time_limit_of(step_name)
            say(f"  · 基线 {step_name} 绿，{seconds:.1f} 秒；" + (f"这一步的超时定为 {limit} 秒" if limit is not None else
                                                              f"逐条按 CPU 时间限（{self.arguments.cpu_multiplier} 倍加 {self.arguments.cpu_floor} 秒，墙钟兜底 {self.arguments.wall_multiplier} 倍加 {self.arguments.wall_floor} 秒）"))
        if ran_every_step:
            self.stages_with_fresh_baseline.add(stage)

    def verify_on_pristine_tree(self, step_names, why, suspects=()):
        """在没改过的树上把这几步各跑一遍（调用时树必须是原样）：退非 0 就是环境坏了，退 6。suspects 是因此不可信的那几条的序号。"""
        for step_name in step_names:
            step = self.step_named(step_name)
            exit_code, seconds = self.run_step(step, ENVIRONMENT_SEQUENCE, self.time_limit_of(step_name))
            if exit_code == 0:
                os.remove(self.log_path(ENVIRONMENT_SEQUENCE, step_name))
                continue
            rerun = ""
            if suspects:
                rerun = (f"；这 {len(suspects)} 条（第 {'、'.join(str(sequence) for sequence in suspects)} 条）的行不可信，环境修好之后点名重跑："
                         f"{heavy_prefix_text()} python3 {SELF_PATH} " + " ".join(f"--item {sequence}" for sequence in suspects) + " --rerun")
            refuse(f"疑似环境坏了（{why}）：{step_name} 在没改过的树上退 {exit_code}（{seconds:.1f} 秒），基线绿过的这一步现在过不了",
                   f"看日志 {self.log_path(ENVIRONMENT_SEQUENCE, step_name)}：盘满、编译目录被清、工具链换了、scratch 写不进这一类；修好之后重跑同一条命令{rerun}",
                   EXIT_BASELINE_RED)

    def verify_environment_at_start(self, stages):
        """每次重起：这一趟没现跑基线的那几段，先在没改过的树上把编译那几步各跑一遍。"""
        if break_is("no-environment-check"):
            return
        step_names = [step[0] for step in self.steps if step[2] == "build" and step[1] in stages and step[1] not in self.stages_with_fresh_baseline]
        if step_names:
            self.verify_on_pristine_tree(step_names, "重起先验编译")
            say(f"  · 重起先验编译：{'、'.join(step_names)} 在没改过的树上照过")


def first_failing_test(log_path, test_failed_exit):
    pattern = NEXTEST_FAILURE_LINE if test_failed_exit == NEXTEST_TEST_FAILED_EXIT else LIBTEST_FAILURE_LINE
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                found = pattern.match(line.rstrip("\n"))
                if found:
                    return found.group(1)
    except OSError:
        pass
    return UNRECOGNISED_TEST


def log_shows_compile_error(log_path):
    """编译那一步的日志里有没有真的编译错误：一条带源码位置的 rustc 错误（error 行之后两行内有 `--> 文件:行:列`），又有 cargo 的「could not compile」。
    磁盘满、锁文件不符、链接器被杀这一类也让 cargo 退 101，但没有带源码位置的错误。"""
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            lines = handle.read().split("\n")
    except OSError:
        return False
    located = any(COMPILE_ERROR_LINE.match(line) and any(COMPILE_ERROR_LOCATION.match(later) for later in lines[index + 1:index + 3])
                  for index, line in enumerate(lines))
    return located and any(COULD_NOT_COMPILE_LINE.match(line) for line in lines)


def apply_mutation(library, item):
    """就地改树里的一个文件：先存原件、记在途记录，再写改后的内容。"""
    target = os.path.join(library.tree, item["file"])
    original_digest = library.manifest.get(item["file"])
    if original_digest is None or file_digest(target) != original_digest:
        refuse(f"要改的 {item['file']} 此刻与这一版的 manifest 对不上：上一条没恢复干净",
               f"重跑同一条命令（开跑时照在途记录恢复）；还不行就 {library.resync_command()}", EXIT_DIRTY_TREE)
    with open(target, encoding="utf-8") as handle:
        original = handle.read()
    mutated = replace_span(original, tuple(item["start"]), tuple(item["end"]), item["replacement"] + MUTATION_MARKER)
    if mutated == original:
        refuse(f"第 {item['sequence']} 条改完与原文相同，没有命中", f"这一版的清单坏了：删掉 {library.version_file('list.json')} 再 --sync-tree 重新生成")
    shutil.copyfile(target, library.inflight_original)
    with open(library.inflight_original, "rb") as handle:
        os.fsync(handle.fileno())
    write_atomically(library.inflight_record, json.dumps(
        {"sequence": item["sequence"], "file": item["file"], "original_digest": original_digest, "started_at": now_text()}).encode())
    with open(target, "w", encoding="utf-8") as handle:
        handle.write(mutated)
    if file_digest(target) == original_digest:
        refuse(f"第 {item['sequence']} 条写进 {target} 之后回读还是原文", "看树所在的盘是不是满了或只读")


class ItemRow(NamedTuple):
    fields: list            # 落下的那一行（十一段）
    step_name: str          # 定下结局的那一步（第二段全绿的 missed 是 ""）
    step_seconds: float     # 那一步的秒数
    environment_doubt: str  # 不空：这一条的结局像是环境造成的（编译步退 101 而没有编译错误），调用方当场在原件上验一次


def process_item(library, runner, item, with_stage_one, with_stage_two):
    """判一条：交回这一条落下的最后一行（ItemRow）。中途被停（Stopped）不落行，原件照样拷回。"""
    sequence = item["sequence"]
    totals = {"build": 0.0, "test": 0.0}
    selection = runner.pruner.selection(item) if runner.pruner else {}
    ran_any_test = []
    escalation_notes = []    # 升档那趟跑完、接着往后判的：它那一句跟进这一条最后落的那一行的备注

    def record(outcome, stage, test_name="", note_text="", step_name="", step_seconds=0.0, environment_doubt=""):
        if outcome == "missed" and not ran_any_test:
            note_text = (note_text + "；" if note_text else "") + "零覆盖：没有一条测试执行到它（C2 记没红）"
        note_text = "；".join(text for text in (note_text, *escalation_notes) if text)
        fields = [sequence, item["package"], item["name"], outcome, stage, test_name, f"{totals['build']:.1f}", f"{totals['test']:.1f}",
                  now_text(), library.fingerprint, note_text]
        library.record_result(fields)
        # 升档判的抓到不进结论表、不跨版本沿用（文件头「升档」与结论表的关系）
        if runner.pruner and outcome in DECISIVE_OUTCOMES and test_name not in ESCALATED_LABELS:
            runner.pruner.record_conclusion(item, outcome, test_name)
        return ItemRow(fields, step_name, step_seconds, environment_doubt)

    def limit_note(log_path):
        try:
            with open(log_path, encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            return ""
        return "；CPU 限时停的（卡住算抓到）" if "BROAD_MUTATION_LIMIT kind=cpu" in text else "；墙钟兜底停的（卡住算抓到）" if "BROAD_MUTATION_LIMIT kind=wall" in text else ""

    def run_stage(stage):
        """跑这一段的每一步：交 None（全绿）或已经落下的那一行。"""
        for step in runner.steps:
            step_name, step_stage, kind, _command, test_failed_exit = step
            if step_stage != stage:
                continue
            command, selection_text = runner.command_for(step, selection)
            if command is None:
                continue
            if step_name in RUNNER_TEST_STEPS:
                ran_any_test.append(step_name)
            runner_mode = "limit" if step_name in RUNNER_TEST_STEPS else None
            exit_code, seconds = runner.run_step(step, sequence, runner.time_limit_of(step_name), command=command, selection_text=selection_text,
                                                 runner_mode=runner_mode)
            totals[kind] += seconds
            if exit_code in (TIME_LIMIT_HIT_EXIT, MEMORY_CAP_HIT_EXIT) and not break_is("escalation-skipped"):
                # 文件头「升档」：当场升档重跑那一步一次；两趟都超时 / 撞顶而证据齐的判抓到，证据不齐的照环境型结局记，升档那趟另有结局的往下照常判
                escalation = runner.escalate_step(item, step, command, selection_text, runner_mode, exit_code)
                totals[kind] += escalation.seconds
                if escalation.label:
                    return record("caught", stage, escalation.label, note_text=escalation.note, step_name=step_name, step_seconds=escalation.seconds)
                if escalation.environmental:
                    return record(escalation.environmental, stage, note_text=escalation.note, step_name=step_name, step_seconds=escalation.seconds)
                escalation_notes.append(escalation.note)
                if escalation.seconds:     # 0 是没升档（这一步没有整步限时、或上限写法认不出）：照原来那一趟往下判
                    exit_code, seconds = escalation.exit_code, escalation.seconds
                    if exit_code == 0:
                        say(f"  · 第 {sequence} 条：{escalation.note}，升档那趟过了，接着往后判")
            log_path = runner.log_path(sequence, step_name)
            if exit_code == 0:
                os.remove(log_path)
                continue
            if exit_code == TIME_LIMIT_HIT_EXIT:
                return record("timeout", stage, note_text=f"{step_name} 超过 {runner.time_limit_of(step_name)} 秒", step_name=step_name, step_seconds=seconds)
            if exit_code == MEMORY_CAP_HIT_EXIT:
                return record("memory-cap", stage, note_text=f"{step_name} 撞了内存上限 {runner.arguments.memory_cap}", step_name=step_name, step_seconds=seconds)
            if kind == "build" and exit_code == CARGO_FAILED_EXIT:
                if log_shows_compile_error(log_path) or break_is("unviable-by-exit-code"):
                    return record("unviable", stage, note_text=f"{step_name} 编不过", step_name=step_name, step_seconds=seconds)
                return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code} 而日志里没有带源码位置的编译错误：不算无效", step_name=step_name,
                              step_seconds=seconds, environment_doubt=f"第 {sequence} 条的 {step_name} 退 {exit_code} 而没有编译错误")
            if kind == "test" and exit_code == test_failed_exit:
                return record("caught", stage, first_failing_test(log_path, test_failed_exit), note_text=step_name + limit_note(log_path), step_name=step_name,
                              step_seconds=seconds)
            if exit_code in WRAPPER_INFRASTRUCTURE_EXITS or break_is("unknown-exit-stops"):
                if exit_code not in WRAPPER_INFRASTRUCTURE_EXITS:
                    refuse(f"第 {sequence} 条的 {step_name} 退 {exit_code}（认不出的退出码）：不落行", f"看日志 {log_path}", EXIT_INFRASTRUCTURE)
                runner.stop_for_infrastructure(step_name, sequence, exit_code)
            return record(DRIVER_ERROR, stage, note_text=f"{step_name} 退 {exit_code}（认不出的退出码，不是这条变异的结局；日志 {os.path.basename(log_path)}）",
                          step_name=step_name, step_seconds=seconds)
        return None

    try:
        apply_mutation(library, item)
        if with_stage_one:
            row = run_stage("one")
            if row is not None:
                return row
            row = record(STAGE_ONE_ONLY, "one", note_text="第一段没红；第二段接着跑" if with_stage_two else "第一段没红；这一趟没跑第二段")
            if not with_stage_two:
                return row
        row = run_stage("two")
        if row is not None:
            return row
        return record("caught" if break_is("stage-two-missed-as-caught") else "missed", "two")
    finally:
        restore_original(library)


# ── 选条、列项、汇总 ───────────────────────────────────────────────────────────────────────────────────────

def parse_shard(text):
    found = re.fullmatch(r"(\d+)/(\d+)", text or "")
    if not found or int(found.group(2)) < 1 or int(found.group(1)) >= int(found.group(2)):
        refuse(f"--shard 写成了「{text}」", "写成 <k>/<n>，k 取 0..n-1，例：两台机器一台 --shard 0/2、一台 --shard 1/2")
    return int(found.group(1)), int(found.group(2))


def select_items(library, arguments, plans=None):
    """交 [(条, 要不要跑第一段, 要不要跑第二段)]，按序号（点名的按点名的次序）。
    --changed-since（plans 给了）只选要现判的；--milestone-full 与 --changed-since 只看这一版现判的行，沿用行不算定论。"""
    states = {} if break_is("rerun-done") else library.item_states()
    if plans is not None or arguments.milestone_full:
        states = item_rows_by_stage(library.read_results())[0]
    nothing = ItemState(NOT_RUN, False)
    want_stage_two = not arguments.skip_stage_two
    how_to_look = f"python3 {SELF_PATH} --list-items 逐行列「序号、包、变异名、状态」；序号写数字，变异名整条照抄"
    if arguments.rerun and not arguments.item:
        refuse("--rerun 只与 --item 同用", "要重跑已有定论的条就点名：--item <序号> --rerun；不点名的续跑只跑还没有定论的")
    if arguments.item:
        if arguments.start_from is not None or arguments.package or arguments.shard:
            refuse("--item 与 --from / --package / --shard 同时给了", "点名就只给 --item（可给多次）；按范围跑就不给 --item")
        by_sequence = {str(item["sequence"]): item for item in library.items}
        by_name = {item["name"]: item for item in library.items}
        named = []
        for wanted in arguments.item:
            item = by_sequence.get(wanted) or by_name.get(wanted)
            if item is None:
                if break_is("accept-unknown-item"):
                    continue
                refuse(f"不认得的项：{wanted}（这一版清单 {len(library.items)} 条，指纹 {library.fingerprint[:16]}）", how_to_look)
            named.append(item)
        if arguments.stage_two_only and not break_is("stage-two-without-stage-one"):
            not_passed = [item["sequence"] for item in named if not states.get(str(item["sequence"]), nothing).passed_stage_one]
            if not_passed:
                refuse(f"--stage-two-only 点到的第 {'、'.join(str(sequence) for sequence in not_passed)} 条在这一版里没过过第一段（没有 stage-one-only 行）："
                       "不经第一段直接记第二段的结局不作数",
                       "先让它们过第一段：去掉 --stage-two-only 重跑这几条（第一段没红的会接着跑第二段）")
        selected = []
        for item in named:
            state = states.get(str(item["sequence"]), nothing)
            if arguments.rerun or break_is("named-items-always-rerun"):
                selected.append((item, not arguments.stage_two_only, want_stage_two))
            elif state.state in FINAL_OUTCOMES:
                say(f"  · 第 {item['sequence']} 条已有定论（{OUTCOME_TEXT[state.state]}），跳过；要重跑加 --rerun")
            elif state.state == STAGE_ONE_ONLY:
                if want_stage_two:
                    selected.append((item, False, True))
                else:
                    say(f"  · 第 {item['sequence']} 条已过第一段，这一趟带着 --skip-stage-two，跳过；要重跑第一段加 --rerun")
            else:
                selected.append((item, True, want_stage_two))
        return selected
    if arguments.package and arguments.package not in MUTATED_PACKAGES:
        refuse(f"--package 给的「{arguments.package}」不在清单的包里", f"只有这几个：{'、'.join(MUTATED_PACKAGES)}；{how_to_look}")
    if arguments.start_from is not None and not 1 <= arguments.start_from <= len(library.items):
        refuse(f"--from {arguments.start_from} 不在 1..{len(library.items)} 里", how_to_look)
    shard = parse_shard(arguments.shard) if arguments.shard else None
    selected = []
    for item in library.items:
        if arguments.start_from is not None and item["sequence"] < arguments.start_from:
            continue
        if arguments.package and item["package"] != arguments.package:
            continue
        if shard and item["sequence"] % shard[1] != shard[0]:
            continue
        if plans is not None and not plans[item["sequence"]][0]:
            continue
        found = states.get(str(item["sequence"]), nothing)
        state = found.state
        if state in (NOT_RUN, DRIVER_ERROR) and not arguments.stage_two_only:
            selected.append((item, True, want_stage_two))
        elif state == STAGE_ONE_ONLY and want_stage_two:
            selected.append((item, False, True))
        elif (plans is not None and state in ENVIRONMENTAL_OUTCOMES and not arguments.stage_two_only
              and not break_is("environmental-not-reselected")):
            # --changed-since 要现判的条这一版里是超时、内存撞顶：提交门禁算没结论，重跑一遍（过过第一段的只补第二段）
            if found.passed_stage_one and want_stage_two:
                selected.append((item, False, True))
            else:
                selected.append((item, True, want_stage_two))
    return selected


def list_items(library):
    states = library.item_states()
    for item in library.items:
        state = states.get(str(item["sequence"]))
        say(f"{item['sequence']}\t{item['package']}\t{item['name']}\t{state.state if state else NOT_RUN}")


def summarise(library):
    """八栏条数；加起来不等于清单条数、结果表里有段数不对的行、认不出的结局、清单里没有的序号、变异名与清单同序号的不同，判红退 1。"""
    results = library.read_results()
    states = library.item_states(results)
    counts = dict.fromkeys(SUMMARY_COLUMNS, 0)
    unknown_states = []
    for item in library.items:
        state = states.get(str(item["sequence"]))
        state = state.state if state else NOT_RUN
        if state in counts:
            counts[state] += 1
        else:
            unknown_states.append((item["sequence"], state))
    total = len(library.items)
    say(f"BROAD_MUTATION_SUMMARY fingerprint={library.fingerprint[:16]} commit={str(library.source.get('commit'))[:12]} total={total} "
        + " ".join(f"{SUMMARY_KEYS[column]}={counts[column]}" for column in SUMMARY_COLUMNS))
    say("  " + "、".join(f"{OUTCOME_TEXT[column]} {counts[column]}" for column in SUMMARY_COLUMNS) + f"；清单 {total} 条")
    last_notes = {row[0]: row[RESULT_COLUMNS.index("note")] for row in results.rows if row[3] == DRIVER_ERROR}
    for item in library.items:
        state = states.get(str(item["sequence"]))
        if state and state.state == "missed":
            say(f"MISSED\t{item['sequence']}\t{item['package']}\t{item['name']}")
    for item in library.items:
        state = states.get(str(item["sequence"]))
        if state and state.state == DRIVER_ERROR:
            say(f"DRIVER_ERROR\t{item['sequence']}\t{item['package']}\t{item['name']}\t{last_notes.get(str(item['sequence']), '')}")
    if results.partial_tail:
        say(f"  · 结果表末行没写完（{len(results.partial_tail.encode())} 字节），不算数；下一次追加之前会截掉，那一条会重跑")
    problems = results.blocking_problems()
    if sum(counts.values()) != total:
        problems.append(f"八栏加起来 {sum(counts.values())}，清单 {total} 条")
    if unknown_states:
        problems.append(f"{len(unknown_states)} 条的状态认不出（例：第 {unknown_states[0][0]} 条「{unknown_states[0][1]}」）")
    if results.unknown_outcomes:
        problems.append(f"{len(results.unknown_outcomes)} 行是认不出的结局（例：第 {results.unknown_outcomes[0][0]} 条「{results.unknown_outcomes[0][1]}」）")
    if results.malformed_lines:
        problems.append(f"结果表里有 {len(results.malformed_lines)} 行段数不是 {len(RESULT_COLUMNS)}（文件第 {'、'.join(str(number) for number in results.malformed_lines[:5])} 行）")
    if results.forged:
        problems.append(f"结果表里没有校验列或校验列对不上的 {results.forged_text()}：不是驱动用这台机器的密钥写的")
    # C6：这一版每条都有结论（抓到、没红、无效；沿用的抓到也算）、没有一条没红，提交门禁才绿
    fresh, carried = item_rows_by_stage(results)
    without = [item for item in library.items if (states.get(str(item["sequence"])) or ItemState(NOT_RUN, False)).state not in DECISIVE_OUTCOMES]
    carried_count = sum(1 for item in library.items if str(item["sequence"]) in carried
                        and (fresh.get(str(item["sequence"])) or ItemState(NOT_RUN, False)).state not in DECISIVE_OUTCOMES)
    flips = milestone_flips(library, results)
    gate_red = bool(without or counts["missed"])
    # 文件头「升档」：状态是升档判的抓到的条（定下状态的那一行「抓到它的测试」一段是标签）逐条列，提交门禁按库目录登记表逐条复核证据
    last_decisive = {}
    for row in results.rows:
        if row[3] in DECISIVE_OUTCOMES:
            last_decisive[row[0]] = row
    escalated = [(item, last_decisive[str(item["sequence"])][5]) for item in library.items
                 if (states.get(str(item["sequence"])) or ItemState(NOT_RUN, False)).state == "caught"
                 and last_decisive.get(str(item["sequence"]), [""] * 6)[5] in ESCALATED_LABELS]
    if break_is("escalated-not-summarised"):
        escalated_field, escalated = "", []
    else:
        escalated_field = f" escalated={len(escalated)}"
    say(f"BROAD_MUTATION_GATE total={total} concluded={total - len(without)} carried={carried_count} fresh={total - len(without) - carried_count} "
        f"missed={counts['missed']} without_conclusion={len(without)} flipped={len(flips)}{escalated_field} verdict={'red' if gate_red else 'green'}")
    for item in without:
        say(f"NO_CONCLUSION\t{item['sequence']}\t{(states.get(str(item['sequence'])) or ItemState(NOT_RUN, False)).state}\t{item['name']}")
    for item, label in escalated:
        say(f"ESCALATED\t{item['sequence']}\t{label}\t{item['name']}")
    for sequence, carried_outcome, fresh_outcome, name in flips:
        say(f"FLIPPED\t{sequence}\t{carried_outcome}\t{fresh_outcome}\t{name}")
    if problems and not break_is("summary-mismatch-silent"):
        results_path = library.version_file("results.tsv")
        print(f"  ✗ 汇总对不上：{'；'.join(problems)}", file=sys.stderr)
        print(f"  → 怎么办：看 {results_path} 里点到的那几行。变异名或序号对不上：清单（list.json）与结果表不是同一份，别删清单重新生成，"
              f"换回生成这些行时的那一份，或 --sync-tree 到另一版；段数不对、结局认不出、没有校验列或校验列对不上的行是写坏或手写的：停掉驱动，把结果表拷一份留底，"
              "删掉点到的那几行（别的行不动），那几条再用 --item <序号> 重跑；每一行都校验不过（密钥丢了、换了，或库目录是有校验列之前建的）就照驱动文件头「结果行的校验列」整库重建",
              file=sys.stderr)
        return EXIT_INCONSISTENT
    if gate_red and not break_is("summary-gate-silent"):
        print(f"  ✗ 提交门禁判红（C6）：{len(without)} 条这一版没有结论、{counts['missed']} 条没红", file=sys.stderr)
        print(f"  → 怎么办：没有结论的用 {heavy_prefix_text()} python3 {SELF_PATH} --changed-since <上一次判过的提交> 补齐（沿用得了的落沿用行，"
              "沿用不了的现判；要现判的条里超时、内存撞顶的也重跑一遍，那一步照文件头「升档」自动升档重跑一次，两趟都超时 / 撞顶而证据齐的判抓到）；"
              "证据不齐、仍是超时或内存撞顶的（NO_CONCLUSION 那几行），加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun；"
              "没红的加一条能杀它的测试再判；确是等价变异的可写进 .cargo/mutants.toml 的 exclude_re（整版重判），理由写在那一行旁边", file=sys.stderr)
        return EXIT_GATE_RED
    return 0


# ── 导入另一台机器同一版的结果行 ───────────────────────────────────────────────────────────────────────────

def import_results(library, source):
    """两台各跑一片（--shard k/2）之后，把另一台那一版的结果行导入本机库。source 是取回来的那一版的目录（或其中的 results.tsv）：
    里面要有 results.tsv、list.json、manifest.tsv（另一台库目录 versions/<指纹>/ 下的那三份），有 toolchain.txt 就一起核。
    全部核完才写：源码指纹（manifest）、清单逐字相同（序号与变异名因此逐条相同）、工具链、每行的指纹段、每行的变异名与本机清单同序号的相同、结局认得。
    本机已有的相同行不重复导；同一条两边都有定论而结局不同的不导、逐条列出、退 1。
    另一台的行带的是它那台的密钥算的校验列，这里核不了：去掉，照本机的密钥重签（文件头「结果行的校验列」）。"""
    library.result_key(create=True)
    directory = source if os.path.isdir(source) else os.path.dirname(os.path.abspath(source))
    paths = {name: os.path.join(directory, name) for name in ("results.tsv", "list.json", "manifest.tsv", "toolchain.txt")}
    fetch_howto = ("把另一台库目录里 versions/<同一个指纹>/ 下的 results.tsv、list.json、manifest.tsv、toolchain.txt 取回到一个目录里再给它："
                   f"这一版的指纹是 {library.fingerprint}")
    missing = [name for name in ("results.tsv", "list.json", "manifest.tsv") if not os.path.isfile(paths[name])]
    if missing:
        refuse(f"{directory} 里缺 {'、'.join(missing)}：没有它们核不了那些行是不是这一版的", fetch_howto)
    unchecked = break_is("import-unchecked")
    peer_source_fingerprint = fingerprint_of_manifest(read_manifest(paths["manifest.tsv"]))
    if peer_source_fingerprint != library.source_fingerprint() and not unchecked:
        refuse(f"另一台那一版的源码指纹是 {peer_source_fingerprint[:16]}，本机这一版是 {library.source_fingerprint()[:16]}：两台不是同一份源码，结果不能合",
               "两台各自 --sync-tree --commit <同一个提交> 之后跑出来的才合得了；" + fetch_howto)
    if file_digest(paths["list.json"]) != file_digest(library.version_file("list.json")) and not unchecked:
        refuse("另一台那一版的清单（list.json）与本机的不逐字相同：同一个序号在两台上不是同一条变异",
               "看两台的 cargo-mutants 版本是不是相同（cargo mutants --version）；清单对不上的结果不能合，得有一台照另一台的清单重跑")
    local_toolchain = library.version_file("toolchain.txt")
    if os.path.isfile(paths["toolchain.txt"]) and os.path.isfile(local_toolchain):
        if file_digest(paths["toolchain.txt"]) != file_digest(local_toolchain) and not unchecked:
            refuse("另一台那一版记下的工具链（toolchain.txt）与本机这一版的不同：两套工具链判出来的行不能记在同一版名下", "两台换成同一套工具链重跑不同的那一台")
        toolchain_checked = "核过"
    else:
        toolchain_checked = "没核（有一边的版本目录里没有 toolchain.txt：加工具链之前建的版本要用这一版驱动真跑过一次才记）"
    table = read_table(paths["results.tsv"], len(RESULT_COLUMNS))
    if table.malformed_lines:
        refuse(f"{paths['results.tsv']} 里有 {len(table.malformed_lines)} 行段数不是 {len(RESULT_COLUMNS)}（第 {'、'.join(str(number) for number in table.malformed_lines[:5])} 行）：取回来的表不完整或被改过",
               "重新取一遍；另一台上先跑 --summary 看那张表自己对不对得上")
    names = {str(item["sequence"]): item["name"] for item in library.items}
    wrong = []
    for row in table.rows:
        sequence, name, outcome, fingerprint = row[0], row[2], row[3], row[RESULT_COLUMNS.index("fingerprint")]
        if unchecked:
            continue
        if fingerprint != library.fingerprint:
            wrong.append(f"第 {sequence} 条的行带的指纹是 {fingerprint[:16]}，不是这一版的 {library.fingerprint[:16]}")
        elif names.get(sequence) != name:
            wrong.append(f"第 {sequence} 条的变异名「{name}」与本机清单同序号的「{names.get(sequence, '清单里没有这个序号')}」不同")
        elif outcome not in ROW_OUTCOMES:
            wrong.append(f"第 {sequence} 条的结局「{outcome}」认不出")
    if wrong:
        refuse(f"{paths['results.tsv']} 里有 {len(wrong)} 行不是这一版的：{wrong[0]}" + ("；……" if len(wrong) > 1 else ""),
               "一行都没导入。指纹段不同：取错了版本目录；变异名不同：两台的清单不是同一份；" + fetch_howto, EXIT_INCONSISTENT)
    local = library.read_results()
    local_problems = local.blocking_problems()
    if local_problems and not unchecked:
        refuse(f"本机这一版的结果表自己就对不上：{'；'.join(local_problems)}", f"先修本机的：python3 {SELF_PATH} --summary 看出路", EXIT_INCONSISTENT)
    local_rows = {tuple(row) for row in local.rows}
    local_states = library.item_states(local)
    peer_rows = [row[:RESULT_SIGNED_COLUMNS] for row in table.rows]    # 另一台的校验列是它自己的密钥签的，核不了：去掉，导入时用本机的密钥重签
    peer_outcomes = {}
    for row in peer_rows:
        peer_outcomes.setdefault(row[0], []).append(row[3])
    peer_states = {sequence: state_of_outcomes(outcomes) for sequence, outcomes in peer_outcomes.items()}
    conflicts = {}
    for sequence, peer_state in peer_states.items():
        local_state = local_states.get(sequence)
        if local_state and local_state.state in FINAL_OUTCOMES and peer_state.state in FINAL_OUTCOMES and local_state.state != peer_state.state:
            conflicts[sequence] = (local_state.state, peer_state.state)
    to_import, already_present, withheld = [], 0, 0
    for row in peer_rows:
        if tuple(row) in local_rows:
            already_present += 1
        elif row[0] in conflicts:
            withheld += 1
        else:
            to_import.append(row)
    if to_import:
        library.record_results(to_import)
    after = {tuple(row) for row in library.read_results().rows}
    lost = [row for row in to_import if tuple(row) not in after]
    append_row(library.version_file("imports.tsv"), IMPORT_COLUMNS,
               [now_text(), os.path.abspath(paths["results.tsv"]), file_digest(paths["results.tsv"]), len(table.rows), len(to_import), already_present, len(conflicts)])
    say(f"BROAD_MUTATION_IMPORT fingerprint={library.fingerprint[:16]} rows_read={len(table.rows)} imported={len(to_import)} already_present={already_present} "
        f"conflicting_items={len(conflicts)} withheld_rows={withheld} partial_tail_ignored={1 if table.partial_tail else 0}")
    say(f"  工具链：{toolchain_checked}")
    for sequence in sorted(conflicts, key=int):
        say(f"IMPORT_CONFLICT\t{sequence}\t本机 {conflicts[sequence][0]}\t另一台 {conflicts[sequence][1]}\t{names.get(sequence, '')}")
    if lost:
        refuse(f"导入之后回读，{len(lost)} 行不在结果表里（例：第 {lost[0][0]} 条）", f"看 {library.version_file('results.tsv')} 所在的盘是不是满了；修好重跑同一条导入（已在的行不重复导）",
               EXIT_INCONSISTENT)
    if conflicts:
        print(f"  ✗ {len(conflicts)} 条在两台上都有定论而结局不同，这几条的行没导入（别的 {len(to_import)} 行已导入）", file=sys.stderr)
        print(f"  → 怎么办：两台本该各跑各的那一片（--shard k/2），同一条两边都跑过说明分片给错了或有人点名跑过；看上面 IMPORT_CONFLICT 各行，"
              f"在本机点名重跑定一次：{heavy_prefix_text()} python3 {SELF_PATH} --item <序号> --rerun", file=sys.stderr)
        return EXIT_INCONSISTENT
    say(f"  ✓ 导入 {len(to_import)} 行（读到 {len(table.rows)} 行，本机已有 {already_present} 行）；python3 {SELF_PATH} --summary 看合起来的八栏")
    return 0


# ── 剪枝（三轮设计判决 C1–C6、E1–E6；规则本身在 research/scripts/mutation-prune.py） ─────────────────────────────

PRUNE_BREAKS = ("regex-function-text", "body-only-function-text", "innermost-function-key", "outside-changes-ignored", "macro-dependencies-ignored",
                "include-dependencies-ignored", "test-fingerprint-ignored", "global-key-ignored", "unviable-carried", "missed-carried", "key-by-bare-name",
                "doc-comments-dropped", "outside-mutation-selected", "const-context-runs-selected", "spawning-binary-selected",
                "coverage-paths-unnormalized", "nextest-filter-single-argument", "limit-below-smt")
PRUNE_MODULE = None


def prune_rules():
    global PRUNE_MODULE
    if PRUNE_MODULE is None:
        specification = importlib.util.spec_from_file_location("mutation_prune_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "mutation-prune.py"))
        PRUNE_MODULE = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(PRUNE_MODULE)
        PRUNE_MODULE.BREAKS = frozenset(name for name in PRUNE_BREAKS if break_is(name))
    return PRUNE_MODULE


def tool_versions(arguments):
    """全局键里的工具版本：nextest、cargo-mutants、llvm-tools（llvm-profdata --version）。"""
    fake = test_variable("BROAD_MUTATION_TOOL_VERSIONS")
    if fake is not None or test_mode():
        return [fake or "selftest-tools"]
    found = []
    for command in (["cargo", "nextest", "--version"], ["cargo", "mutants", "--version"], [os.path.join(llvm_bin(arguments), "llvm-profdata"), "--version"]):
        try:
            done = subprocess.run(command, capture_output=True, text=True, stdin=subprocess.DEVNULL)
            found.append(" ".join(done.stdout.split()[:6]) or f"{command[-2]} 退 {done.returncode}")
        except OSError:
            found.append(f"{command[0]} 不在")
    return found


def llvm_bin(arguments):
    """llvm-profdata、llvm-cov 在哪：--llvm-bin 给的，没给取 rustc 的 sysroot 下 rustup 组件 llvm-tools 的 bin（与编译器同一份 LLVM，profraw 格式对得上）。"""
    if arguments.llvm_bin:
        return arguments.llvm_bin
    try:
        sysroot = subprocess.run(["rustc", "--print", "sysroot"], capture_output=True, text=True, stdin=subprocess.DEVNULL).stdout.strip()
        host = next(line.split(":", 1)[1].strip() for line in toolchain_text(REPOSITORY).splitlines() if line.startswith("host:"))
    except (OSError, StopIteration):
        return ""
    return os.path.join(sysroot, "lib", "rustlib", host, "bin")


def host_triple(library):
    if test_mode():
        return "x86_64-unknown-linux-gnu"
    return next((line.split(":", 1)[1].strip() for line in toolchain_text(library.tree).splitlines() if line.startswith("host:")), "x86_64-unknown-linux-gnu")


ADMISSION_MODULE = None


def admission_module():
    """research/scripts/admission.py（构建环境的定义只认它那一份 build_environment_lines）。"""
    global ADMISSION_MODULE
    if ADMISSION_MODULE is None:
        specification = importlib.util.spec_from_file_location("admission_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "admission.py"))
        ADMISSION_MODULE = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(ADMISSION_MODULE)
    return ADMISSION_MODULE


def build_environment_of(root, environment):
    """全局键里的构建环境：[(名字, 内容字节)]。admission.py 的 build_environment_lines 在 root（驱动是库目录的树，门禁是同一棵树）上按 environment 算的那一份
    （目录层级与 CARGO_HOME 下的 cargo 配置与它们指的程序、编译变量与它们指的程序、RUSTC），再加文档测试那一步才读的 DOC_TEST_VARIABLES。
    算不出（配置文件读不了、RUSTC 的 -V 跑不出）抛 Refusal（退 2）。"""
    admission = admission_module()
    try:
        lines = admission.build_environment_lines(root, environment)
    except admission.InputManifestError as error:
        refuse(f"算不出构建环境（research/scripts/admission.py 的 build_environment_lines）：{error}",
               "看报错里点的那份 cargo 配置或那个编译器：配置文件要读得了，RUSTC / build.rustc 指的编译器要跑得出 -V")
    return list(lines) + [(f"<文档测试：环境变量 {name}>", environment[name].encode("utf-8", "surrogateescape"))
                          for name in DOC_TEST_VARIABLES if name in environment]


def escalation_line():
    return (f"escalation time={ESCALATION_TIME_FACTOR}x,{ESCALATION_TIME_BASELINE_MULTIPLE}xbaseline "
            f"memory={ESCALATION_MEMORY_FACTOR}x,{ESCALATION_MEMORY_BASELINE_MULTIPLE}xbaseline")


def global_key_lines(rules, toolchain, manifest, build_environment, tools, exclusions_digest, limits_line):
    """全局键全文（E4）的唯一写法：驱动（global_key_text）与提交门禁（.claude/gate.d/checker-tier-broad-mutation.sh 现算此刻的全局键）都调它。
    manifest 是 {相对树根的路径: sha256}，只取 Cargo.toml、Cargo.lock 与 .cargo/ 下的；build_environment 是 build_environment_of 交的 [(名字, 内容字节)]。"""
    lines = [f"rules {rules}"]
    if not break_is("global-key-no-toolchain-line"):
        lines.append("toolchain " + " | ".join(toolchain.strip().splitlines()))
    for path in sorted(manifest):
        if (os.path.basename(path) in ("Cargo.toml", "Cargo.lock") or path.startswith(".cargo/")) and not break_is("global-key-no-file-lines"):
            lines.append(f"file {path} {manifest[path]}")
    if not break_is("global-key-no-build-lines"):
        lines += [f"build {hashlib.sha256(content).hexdigest()} {name}" for name, content in build_environment]
    if not break_is("global-key-no-tool-lines"):
        lines += [f"tool {version}" for version in tools]
    lines.append(f"exclusions {exclusions_digest}")
    if not break_is("global-key-no-limits-line"):
        lines.append(limits_line)
    lines.append(escalation_line())
    return "\n".join(lines) + "\n"


def exclusions_digest_of(path):
    return file_digest(path) if os.path.isfile(path) else "none"


def limits_line_of(arguments):
    line = f"limits cpu={arguments.cpu_multiplier}x+{arguments.cpu_floor} wall={arguments.wall_multiplier}x+{arguments.wall_floor}"
    if break_is("timeout-floor-in-global-key"):
        line += f" floor={getattr(arguments, 'timeout_floor', '')} memory={getattr(arguments, 'memory_cap', '')}"
    return line


def global_key_text(toolchain, manifest, arguments, build_root):
    """此刻的全局键全文：工具链、Cargo.toml、Cargo.lock、.cargo/、构建环境（在 build_root 上算）、nextest / cargo-mutants / llvm-tools 版本、排除表、
    限时参数、升档常量、本驱动的判法版本。"""
    return global_key_lines(judge_rules_version(), toolchain, manifest, build_environment_of(build_root, os.environ), tool_versions(arguments),
                            exclusions_digest_of(test_variable("BROAD_MUTATION_EXCLUSIONS") or EXCLUSIONS_FILE), limits_line_of(arguments))


def global_key(library, arguments):
    """E4：交 (全文, sha256)；全文在 --sync-tree 时写进这一版的 global-key.txt、sha256 进这一版的指纹。键变了，记着别的键的结论一条都不沿用（整版重判）。"""
    text = global_key_text(toolchain_text(library.tree), library.manifest, arguments, library.tree)
    return text, hashlib.sha256(text.encode()).hexdigest()


def check_global_key(library, arguments):
    """真跑之前：这一趟现算的全局键与这一版 --sync-tree 时记下的（tree.source 的 global_key）相同。不同、或这一版是全局键进指纹之前建的，退 7。
    旧驱动的模拟（BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE、stale-judge-version）与弄坏开关 global-key-outside-version 下不核。"""
    if pre_fresh_build_judge() or break_is("global-key-outside-version"):
        return
    text, digest = global_key(library, arguments)
    recorded = library.source.get("global_key")
    if recorded == digest:
        return
    recorded_path = library.version_file("global-key.txt")
    recorded_lines = []
    if recorded and os.path.isfile(recorded_path):
        with open(recorded_path, encoding="utf-8") as handle:
            recorded_lines = handle.read().splitlines()
    differing = [f"这一版记的「{line}」" for line in recorded_lines if line not in text.splitlines()]
    differing += [f"此刻是「{line}」" for line in text.splitlines() if line not in recorded_lines]
    what = ("这一版是全局键进版本指纹之前建的（tree.source 里没有 global_key）" if not recorded
            else f"全局键与这一版（指纹 {library.fingerprint[:16]}）--sync-tree 时记下的不同：{'；'.join(differing[:6]) or '全文相同而 sha256 不同'}")
    refuse(f"{what}；换了全局键判出来的行不能记在同一版名下（E4：全局键变了就整版重判）",
           f"要在此刻的全局键下判就另起一版：{library.resync_command()}（限时参数、--llvm-bin 与这一趟给的相同；同一个提交在新全局键下是另一版，结论表里键相同的照沿用）；"
           f"要接着这一版跑就把排除表、工具版本、构建环境（cargo 配置、编译变量）、限时参数换回去（{recorded_path} 里是这一版的全文）；"
           f"会话 shell 里设的编译变量经 detached-run.sh 起的单元带不进去时，照 python3 {SELF_PATH} --unit-environment 打的那一行带进单元",
           EXIT_TOOLCHAIN)


class Pruner:
    """这一版的剪枝：每条的键与指纹、结论表（库目录 conclusions.tsv，跨版本、只追加）、沿用判定、按覆盖挑测试。"""

    def __init__(self, library, arguments):
        self.library, self.arguments, self.rules = library, arguments, prune_rules()
        self.tree = self.rules.TreeIndex(library.tree, library.manifest)
        self.global_text, self.global_key = global_key(library, arguments)
        self.fingerprints, self.fallbacks = self.rules.test_fingerprints(self.tree, (*HARNESS_TIER_PACKAGES, CHECKER_TIER_PACKAGE))
        self.keys, self.coverage_one, self.coverage_two = {}, None, None
        self.store_path = os.path.join(library.home, "conclusions.tsv")
        self.store = {}
        for row in read_table(self.store_path, len(CONCLUSION_COLUMNS)).rows:
            self.store[row[0]] = dict(zip(CONCLUSION_COLUMNS, row))

    def key_of(self, item):
        sequence = item["sequence"]
        if sequence not in self.keys:
            self.keys[sequence] = self.rules.mutation_key(self.tree, item)
        return self.keys[sequence]

    def key_text(self, item):
        return json.dumps(self.key_of(item)["key"], ensure_ascii=False)

    def test_fingerprint(self, caught_by):
        return self.rules.catching_test_fingerprint(self.fingerprints, self.fallbacks, caught_by)

    def carry(self, item):
        """交 (能不能沿用, 理由, 结论表里那一行)。"""
        record = self.store.get(self.key_text(item))
        if record is not None and break_is("carry-without-store-check"):
            return True, "carried", record
        test_now = self.test_fingerprint(record["caught_by"]) if record else ""
        carry, reason = self.rules.carry_decision(record, self.key_of(item), self.global_key, test_now)
        return carry, reason, record

    def record_conclusion(self, item, outcome, caught_by, sequence_note=""):
        current = self.key_of(item)
        fields = [self.key_text(item), outcome, caught_by, self.test_fingerprint(caught_by) if outcome == "caught" else "", current["fingerprint"],
                  current["dependency"], self.global_key, self.library.fingerprint, str(self.library.source.get("commit")), item["sequence"], now_text()]
        append_row(self.store_path, CONCLUSION_COLUMNS, fields)
        self.store[fields[0]] = dict(zip(CONCLUSION_COLUMNS, [clean_field(field) for field in fields]))

    def load_coverage(self):
        for name in ("coverage-one.json", "coverage-two.json"):
            path = self.library.version_file(name)
            if os.path.isfile(path):
                with open(path, encoding="utf-8") as handle:
                    setattr(self, "coverage_" + name.split("-")[1].split(".")[0], json.load(handle))

    def selection(self, item):
        """C2 / E1 / E6：这条变异每一步跑哪些测试。{"one": None 全部 / [(二进制, 用例)], "slow": None / [二进制], "checker": None / [目标], "reason": …}。"""
        current = self.key_of(item)
        everything = self.rules.run_everything_reason(current)
        if everything or break_is("coverage-selection-ignored") or self.coverage_one is None:
            return {"one": None, "slow": None, "checker": None, "reason": everything or "no-coverage"}
        container = (item["file"], current["key"][1] if not break_is("key-by-bare-name") else (current["function"].qualified if current["function"] else current["key"][1]))
        tests = {(entry["binary"], entry["test"]): {tuple(found) for found in entry["covered"]} for entry in self.coverage_one.get("tests", [])}
        counts = {(entry["binary"], entry["test"]): entry.get("profiles", 1) for entry in self.coverage_one.get("tests", [])}
        spawning = self.rules.spawning_binaries(self.tree, HARNESS_TIER_PACKAGES, counts)
        chosen = {"one": self.rules.select_tests(tests, container, spawning), "reason": "coverage"}
        two = self.coverage_two or {}
        checker_spawning = {binary.split("::", 1)[1] if "::" in binary else "lib" for binary in self.rules.spawning_binaries(self.tree, (CHECKER_TIER_PACKAGE,))}
        for part, extra in (("slow", set()), ("checker", checker_spawning)):
            entries = two.get(part)
            if entries is None or break_is("stage-two-not-by-binary"):
                chosen[part] = None
                continue
            chosen[part] = sorted({entry["binary"] for entry in entries if container in {tuple(found) for found in entry["covered"]}} | extra)
        return chosen


def write_runner(library):
    path = os.path.join(library.home, "test-runner.py")
    write_atomically(path, prune_rules().RUNNER_SCRIPT.encode())
    os.chmod(path, 0o755)
    return path


def coverage_step(specification_path):
    """--coverage-step：驱动在没改过的树上起的一步（经内存包装）。插桩编一遍，第一段逐条测试单独一个进程跑、第二段逐个二进制跑，
    每条（每个二进制）的 profraw 合并后 llvm-cov export，记它执行到的容器；写 spec 里的 out，有跑红的退 1。"""
    import concurrent.futures
    with open(specification_path, encoding="utf-8") as handle:
        spec = json.load(handle)
    rules = prune_rules()
    tree = rules.TreeIndex(spec["tree"])
    llvm = spec["llvm_bin"]
    profiles = os.path.join(spec["scratch"], "profiles")
    os.makedirs(profiles, exist_ok=True)
    environment = dict(os.environ, CARGO_TARGET_DIR=spec["target"], RUSTFLAGS="-C instrument-coverage", CARGO_TERM_COLOR="never",
                       LLVM_PROFILE_FILE=os.path.join(spec["scratch"], "build-%p-%m.profraw"))

    def build(arguments):
        done = subprocess.run(["cargo", "test", "--locked", "--no-run", "--message-format=json", *arguments], cwd=spec["tree"], env=environment,
                              capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if done.returncode != 0:
            print(done.stderr[-4000:], file=sys.stderr)
            sys.exit(1)
        found = []
        for line in done.stdout.splitlines():
            message = json.loads(line) if line.startswith("{") else {}
            if message.get("reason") == "compiler-artifact" and message.get("executable") and message.get("profile", {}).get("test"):
                package = message["package_id"].split("#")[0].rsplit("/", 1)[-1].split("@")[0]
                kind, name = message["target"]["kind"][0], message["target"]["name"]
                binary = package if kind == "lib" else (f"{package}::bin/{name}" if kind == "bin" else f"{package}::{name}")
                found.append((binary, kind, name, message["executable"], os.path.dirname(message["manifest_path"])))
        return found

    def listed(executable, cwd, ignored):
        done = subprocess.run([executable, "--list", "--format", "terse", *(["--ignored"] if ignored else [])], cwd=cwd, env=environment,
                              capture_output=True, text=True, stdin=subprocess.DEVNULL)
        return [line[:-len(": test")] for line in done.stdout.splitlines() if line.endswith(": test")]

    def cover(job):
        number, binary, executable, cwd, argv, single = job
        directory = os.path.join(profiles, str(number))
        os.makedirs(directory, exist_ok=True)
        child = dict(environment, LLVM_PROFILE_FILE=os.path.join(directory, "%p-%m.profraw"))
        if single:
            child.update({name: "1" for name in ("RUST_TEST_THREADS", "SINGLEFS_THREAD_CAP")})
        done = subprocess.run(argv, cwd=cwd, env=child, capture_output=True, text=True, stdin=subprocess.DEVNULL)
        files = sorted(os.path.join(directory, name) for name in os.listdir(directory) if name.endswith(".profraw"))
        covered = set()
        if files:
            merged = os.path.join(directory, "merged.profdata")
            subprocess.run([os.path.join(llvm, "llvm-profdata"), "merge", "-sparse", *files, "-o", merged], check=True, capture_output=True)
            exported = subprocess.run([os.path.join(llvm, "llvm-cov"), "export", executable, f"-instr-profile={merged}"], capture_output=True, text=True, check=True)
            covered = rules.covered_containers(json.loads(exported.stdout), tree)
        shutil.rmtree(directory, ignore_errors=True)
        return {"binary": binary, "test": argv[argv.index("--exact") + 1] if "--exact" in argv else "*", "covered": sorted(covered),
                "profiles": len(files), "exit": done.returncode}

    jobs, harness = [], package_arguments(HARNESS_TIER_PACKAGES)
    if spec["kind"] == "one":
        for binary, _kind, _name, executable, cwd in build(harness):
            for test in listed(executable, cwd, False):
                jobs.append((len(jobs), binary, executable, cwd, [executable, "--exact", test, "--test-threads=1"], True))
    else:
        excluded = {(binary, test) for binary, test, _reason in load_exclusions()}
        for binary, _kind, _name, executable, cwd in build(harness):
            for test in listed(executable, cwd, True):
                if (binary, test) not in excluded:
                    jobs.append((len(jobs), binary, executable, cwd, [executable, "--ignored", "--exact", test, "--test-threads=1"], True))
        for binary, kind, name, executable, cwd in build(["--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]):
            jobs.append((len(jobs), "checker:" + ("lib" if kind == "lib" else name), executable, cwd, [executable], False))
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, spec["workers"] if spec["kind"] == "one" else 1)) as pool:
        records = list(pool.map(cover, jobs))
    failed = [record for record in records if record["exit"] != 0]
    if spec["kind"] == "one":
        result = {"tests": records, "failed": failed}
    else:
        slow = {}
        for record in records:
            if not record["binary"].startswith("checker:"):
                slow.setdefault(record["binary"], set()).update(map(tuple, record["covered"]))
        result = {"slow": [{"binary": binary, "covered": sorted(covered)} for binary, covered in sorted(slow.items())],
                  "checker": [{"binary": record["binary"][len("checker:"):], "covered": record["covered"]} for record in records if record["binary"].startswith("checker:")],
                  "failed": failed}
    write_atomically(spec["out"], (json.dumps(result, ensure_ascii=False) + "\n").encode())
    say(f"BROAD_MUTATION_COVERAGE kind={spec['kind']} jobs={len(jobs)} failed={len(failed)}")
    for record in failed:
        print(f"  ✗ 覆盖那一遍在没改过的树上跑红：{record['binary']} {record['test']}（退 {record['exit']}）", file=sys.stderr)  # gate-lint:detail
    return 1 if failed else 0


def item_rows_by_stage(results):
    """{序号: (这一版现判的 ItemState, 有没有沿用行, 沿用行的结局)}：现判的不算沿用行。"""
    fresh, carried = {}, {}
    for row in results.rows:
        if row[4] == CARRIED_STAGE:
            carried[row[0]] = row[3]
        else:
            fresh.setdefault(row[0], []).append(row[3])
    return {sequence: state_of_outcomes(outcomes) for sequence, outcomes in fresh.items()}, carried


def old_tree_index(library, arguments, scratch):
    """--changed-since 的那个提交：只导出它的源码到草稿目录，切成 TreeIndex。"""
    fake = test_variable("BROAD_MUTATION_CHANGED_SINCE_SOURCE")
    remove_tree(scratch)
    if fake:
        shutil.copytree(fake, scratch)
    else:
        os.makedirs(scratch)
        archive = subprocess.Popen(["git", "-C", REPOSITORY, "archive", "--format=tar", arguments.changed_since, "--", *(library.source.get("export_paths") or ["crates"])],
                                   stdout=subprocess.PIPE, stdin=subprocess.DEVNULL)
        unpack = subprocess.run(["tar", "-x", "-C", scratch], stdin=archive.stdout)
        archive.stdout.close()
        if archive.wait() != 0 or unpack.returncode != 0:
            refuse(f"导出 --changed-since 给的提交「{arguments.changed_since}」没成", "给一个主仓里有的提交，例 HEAD~1；导出范围照这一版树的 export_paths")
    return prune_rules().TreeIndex(scratch)


def plan_changed_since(library, arguments, pruner):
    """C1 + C3：每条交 (要不要现判, 理由, 沿用的那一行)。对着 --changed-since 的提交算「这次改到的」，没改到的要沿用得了才沿用，沿用不了的也现判。"""
    old = old_tree_index(library, arguments, os.path.join(library.home, "changed-since-tree"))
    plans = {}
    for item in library.items:
        changed = prune_rules().changed_since(old, pruner.tree, item, pruner.key_of(item))
        carry, reason, record = pruner.carry(item)
        if changed is None and carry:
            plans[item["sequence"]] = (False, "carried", record)
        else:
            plans[item["sequence"]] = (True, changed or reason, record)
    return plans


def write_carried_rows(library, plans, results):
    """沿用的条在这一版的结果表里落一行（结局 caught，哪一段写 carried），已有这一版定论的不再落。"""
    fresh, carried = item_rows_by_stage(results)
    rows = []
    for item in library.items:
        sequence = str(item["sequence"])
        judge, _reason, record = plans[item["sequence"]]
        if judge or sequence in carried or (sequence in fresh and fresh[sequence].state in DECISIVE_OUTCOMES):
            continue
        rows.append([item["sequence"], item["package"], item["name"], record["outcome"], CARRIED_STAGE, record["caught_by"], "0.0", "0.0", now_text(),
                     library.fingerprint, f"沿用 {record['version'][:16]} 的结论（{record['finished_at']}，提交 {record['commit'][:12]}）"])
    if rows:
        library.record_results(rows)
    return len(rows)


def milestone_flips(library, results):
    """C5：里程碑全量现判的结局与开跑时记下的「会沿用的结论」比，交 [(序号, 沿用的, 现判的, 变异名)]。"""
    path = library.version_file("milestone-carry.tsv")
    if not os.path.isfile(path) or break_is("milestone-flips-unlisted"):
        return []
    snapshot = {row[0]: row[1] for row in read_table(path, 2).rows}
    fresh, _carried = item_rows_by_stage(results)
    names = {str(item["sequence"]): item["name"] for item in library.items}
    return [(sequence, snapshot[sequence], fresh[sequence].state, names.get(sequence, ""))
            for sequence in sorted(snapshot, key=int) if sequence in fresh and fresh[sequence].state in FINAL_OUTCOMES and fresh[sequence].state != snapshot[sequence]]


# ── 主流程 ─────────────────────────────────────────────────────────────────────────────────────────────────

class ArgumentParser(argparse.ArgumentParser):
    def error(self, message):
        print(f"  ✗ 用法错：{message}", file=sys.stderr)
        print(f"  → 怎么办：python3 {SELF_PATH} --help 看每个参数；项的写法照 --list-items 列的", file=sys.stderr)
        sys.exit(EXIT_USAGE)


def default_threads():
    configured = os.environ.get("SINGLEFS_THREAD_CAP", "")
    if re.fullmatch(r"[1-9][0-9]*", configured):
        return int(configured)
    return max(1, (os.cpu_count() or 2) - 1)


def parse_arguments(argv):
    parser = ArgumentParser(description="广谱变异的逐条驱动；用法与库目录的布局见文件头", add_help=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--sync-tree", action="store_true")
    mode.add_argument("--list-items", action="store_true")
    mode.add_argument("--summary", action="store_true")
    mode.add_argument("--check-ignored", action="store_true")
    mode.add_argument("--unit-environment", action="store_true")
    mode.add_argument("--import-results", metavar="目录")
    mode.add_argument("--selftest", action="store_true")
    mode.add_argument("--coverage-step", metavar="spec")
    parser.add_argument("--changed-since", metavar="提交")
    parser.add_argument("--milestone-full", action="store_true")
    parser.add_argument("--llvm-bin")
    parser.add_argument("--cpu-multiplier", type=float, default=None)
    parser.add_argument("--cpu-floor", type=float, default=None)
    parser.add_argument("--wall-multiplier", type=float, default=None)
    parser.add_argument("--wall-floor", type=float, default=None)
    parser.add_argument("--commit", default="HEAD")
    parser.add_argument("--item", action="append", default=[])
    parser.add_argument("--rerun", action="store_true")
    parser.add_argument("--from", dest="start_from", type=int)
    parser.add_argument("--package")
    parser.add_argument("--shard")
    stage = parser.add_mutually_exclusive_group()
    stage.add_argument("--skip-stage-two", action="store_true")
    stage.add_argument("--stage-two-only", action="store_true")
    parser.add_argument("--stage-two-eatmydata", action="store_true")
    parser.add_argument("--threads", type=int, default=default_threads())
    parser.add_argument("--memory-cap", default=os.environ.get("BROAD_MUTATION_MEMORY_CAP") or DEFAULT_MEMORY_CAP)
    parser.add_argument("--timeout-floor", type=int, default=DEFAULT_TIMEOUT_FLOOR_SECONDS)
    parser.add_argument("--timeout-multiplier", type=float, default=DEFAULT_TIMEOUT_MULTIPLIER)
    parser.add_argument("--suspect-streak", type=int, default=DEFAULT_SUSPECT_STREAK)
    parser.add_argument("--suspect-test-seconds", type=float, default=DEFAULT_SUSPECT_TEST_SECONDS)
    parser.add_argument("--stop-grace", type=int, default=DEFAULT_STOP_GRACE_SECONDS)
    parser.add_argument("--lock-wait", type=int, default=DEFAULT_LOCK_WAIT_SECONDS)
    parser.add_argument("--home")
    arguments = parser.parse_args(argv)
    if arguments.threads < 1:
        parser.error("--threads 要是正整数")
    if arguments.suspect_streak < 1:
        parser.error("--suspect-streak 要是正整数")
    if arguments.coverage_step:
        return arguments
    rules = prune_rules()
    for name, default in (("cpu_multiplier", rules.DEFAULT_CPU_MULTIPLIER), ("cpu_floor", rules.DEFAULT_CPU_FLOOR_SECONDS),
                          ("wall_multiplier", rules.DEFAULT_WALL_MULTIPLIER), ("wall_floor", rules.DEFAULT_WALL_FLOOR_SECONDS)):
        if getattr(arguments, name) is None:
            setattr(arguments, name, default)
    refused_multiplier = rules.check_cpu_multiplier(arguments.cpu_multiplier)
    if refused_multiplier:
        parser.error(refused_multiplier)
    if arguments.changed_since and arguments.milestone_full:
        parser.error("--changed-since 与 --milestone-full 只给一个：提交前判改到的用前者，里程碑末全量用后者")
    if (arguments.changed_since or arguments.milestone_full) and (arguments.item or arguments.rerun or arguments.stage_two_only):
        parser.error("--changed-since / --milestone-full 不与 --item、--rerun、--stage-two-only 同用：它们自己定选哪些条")
    return arguments


def check_prefix(what):
    """起 cargo 的与改库目录的都要带前缀；自证自己起的子进程（假的一步、假的树）不要。"""
    if test_mode():
        return
    if os.environ.get(HEAVY_PREFIX_VARIABLE) not in HEAVY_PREFIX_VALUES:
        refuse(f"没带 {HEAVY_PREFIX_VARIABLE}=commit 或 =user-request：{what}，只在提交时或用户要求时做",
               f"用户要求时这样起：{HEAVY_PREFIX_VARIABLE}=user-request nice -n 19 python3 {SELF_PATH} [参数…]；"
               "只列项、看汇总不用前缀（--list-items、--summary）")


def check_real_run_is_allowed():
    """真跑（起 cargo）之前：带没带前缀、工具在不在。"""
    check_prefix("广谱变异是重型测试")
    if test_mode():
        return
    missing = [tool for tool in ("cargo", "rustc", "cargo-nextest", "eatmydata", "systemd-run", "systemctl") if shutil.which(tool) is None]
    if missing:
        refuse(f"PATH 上缺：{'、'.join(missing)}", "cargo-nextest 用 cargo install cargo-nextest --locked 装，eatmydata 用系统包管理器装（apt install eatmydata）")


class SuspectStreaks:
    """连着几条像是环境造成的结局（文件头 --suspect-streak）：连续无效、连续「测试那一步几乎立刻红」的抓到。到了 K 条就在没改过的树上把那一步重跑一遍。"""

    def __init__(self, runner, arguments):
        self.runner, self.arguments = runner, arguments
        self.unviable, self.fast_caught = [], []

    def saw(self, sequence, row):
        outcome = row.fields[3]
        if break_is("no-environment-check"):
            return
        if row.environment_doubt:
            self.runner.verify_on_pristine_tree([row.step_name], row.environment_doubt, [sequence])
        self.unviable = self.unviable + [(sequence, row.step_name)] if outcome == "unviable" else []
        fast = outcome == "caught" and row.step_seconds < self.arguments.suspect_test_seconds
        self.fast_caught = self.fast_caught + [(sequence, row.step_name)] if fast else []
        for streak, why in ((self.unviable, "无效"), (self.fast_caught, f"测试那一步不到 {self.arguments.suspect_test_seconds} 秒就红的抓到")):
            if len(streak) < self.arguments.suspect_streak:
                continue
            sequences = [found_sequence for found_sequence, _step in streak]
            step_names = list(dict.fromkeys(step_name for _sequence, step_name in streak))
            self.runner.verify_on_pristine_tree(step_names, f"连续 {len(streak)} 条{why}", sequences)
            say(f"  · 连续 {len(streak)} 条{why}（第 {sequences[0]}–{sequences[-1]} 条）：{'、'.join(step_names)} 在没改过的树上照过，不是环境，接着跑")
            streak.clear()


def run_items(library, arguments):
    runner = Runner(library, arguments)
    pruner = runner.pruner = Pruner(library, arguments)
    write_atomically(library.version_file("global-key.txt"), pruner.global_text.encode())
    plans = None
    if arguments.changed_since:
        plans = plan_changed_since(library, arguments, pruner)
        carried_now = write_carried_rows(library, plans, library.read_results())
        reasons = {}
        for judge, reason, _record in plans.values():
            if judge:
                reasons[reason] = reasons.get(reason, 0) + 1
        say(f"BROAD_MUTATION_CHANGED since={arguments.changed_since} judge={sum(reasons.values())} carried={len(plans) - sum(reasons.values())} "
            f"carried_rows_written={carried_now} " + " ".join(f"{reason}={count}" for reason, count in sorted(reasons.items())))
    if arguments.milestone_full and not os.path.isfile(library.version_file("milestone-carry.tsv")):
        snapshot = []
        for item in library.items:
            carry, _reason, record = pruner.carry(item)
            if carry:
                snapshot.append([item["sequence"], record["outcome"]])
        append_rows(library.version_file("milestone-carry.tsv"), ("sequence", "carried_outcome"), snapshot)
        say(f"  · 里程碑全量：开跑时会沿用的结论 {len(snapshot)} 条记进 milestone-carry.tsv，全部现判之后逐条比")
    selected = select_items(library, arguments, plans)
    say(f"BROAD_MUTATION_PLAN fingerprint={library.fingerprint[:16]} commit={str(library.source.get('commit'))[:12]} list={len(library.items)} "
        f"selected={len(selected)} threads={arguments.threads} memory_cap={arguments.memory_cap} global_key={pruner.global_key[:16]}")
    if not selected:
        say(f"  ✓ 选中 0 条要跑：照这一次的选法，清单 {len(library.items)} 条里没有还差定论的（--summary 看八栏与提交门禁那一行）")
        report_flips(library)
        return 0
    stages = [stage for stage, wanted in (("one", any(one for _item, one, _two in selected)), ("two", any(two for _item, _one, two in selected))) if wanted]
    if "two" in stages:
        check_ignored_registration(library, runner.exclusions)    # 基线之前：一步 cargo 都还没起
    runner.runner_path = write_runner(library)
    stale = [] if break_is("stale-artifact-check-skipped") else stale_build_problems(library)
    if stale:
        refuse(f"编译目录里可能留着换树之前编的工作区产物，基线与每条变异会跑在过时的代码上，一步都不跑：{'；'.join(stale)}",
               f"重做换树：{library.resync_command()}（导出带 -m，换树后清掉 target/ 与 target-coverage/ 里工作区各包的 fingerprint，第三方依赖不动），"
               "再重跑同一条命令", EXIT_DIRTY_TREE)
    for stage in stages:
        runner.ensure_baseline(stage)
    runner.prepare_limits()
    runner.ensure_coverage(stages)
    pruner.load_coverage()
    runner.verify_environment_at_start(stages)
    ran = dict.fromkeys(ROW_OUTCOMES, 0)
    streaks = SuspectStreaks(runner, arguments)
    started = time.monotonic()
    try:
        for item, with_stage_one, with_stage_two in selected:
            item_started = time.monotonic()
            row = process_item(library, runner, item, with_stage_one, with_stage_two)
            fields = row.fields
            ran[fields[3]] += 1
            say(f"BROAD_MUTATION_ITEM sequence={item['sequence']} outcome={fields[3]} stage={fields[4]} build_seconds={fields[6]} test_seconds={fields[7]} "
                f"wall_seconds={time.monotonic() - item_started:.1f} name={item['name']}" + (f" first_failing_test={fields[5]}" if fields[5] else ""))
            streaks.saw(item["sequence"], row)
    finally:
        library.flush_deferred_rows()
    say(f"BROAD_MUTATION_TOTAL selected={len(selected)} ran={sum(ran.values())} "
        + " ".join(f"{SUMMARY_KEYS[name]}={count}" for name, count in ran.items()) + f" seconds={time.monotonic() - started:.0f}")
    say(f"  ✓ 选中的 {len(selected)} 条都判完（{'、'.join(f'{OUTCOME_TEXT[name]} {count}' for name, count in ran.items())}）")
    report_flips(library)
    return 0


def unit_environment_names(environment, root):
    """全局键与工具链认的那几个会话变量：拿掉它、在 root 上算的构建环境（build_environment_of）就变的，或在 root 里起的 rustc -Vv（toolchain_text）就变的
    （RUSTUP_TOOLCHAIN 这一类：换了工具链，真跑退 7）；detached-run.sh 本来带进单元的（UNIT_CARRIED_VARIABLES）与只供自证的变量不算。
    拿掉它就算不出构建环境、起不了 rustc 的也算。弄坏开关 unit-environment-fixed-list 下只认全局键改调构建环境之前那几个，
    unit-environment-toolchain-ignored 下不比工具链。"""
    if break_is("unit-environment-fixed-list"):
        return [name for name in sorted(environment) if name in FIXED_LIST_VARIABLES or name.startswith("CARGO_PROFILE_")]
    base = build_environment_of(root, environment)
    base_toolchain = None if break_is("unit-environment-toolchain-ignored") else toolchain_text(root, environment)
    names = []
    for name in sorted(environment):
        if name in UNIT_CARRIED_VARIABLES or name in TEST_ONLY_VARIABLES or name == SELFTEST_WORK_VARIABLE:
            continue
        without = {other: value for other, value in environment.items() if other != name}
        try:
            changed = build_environment_of(root, without) != base
            if not changed and base_toolchain is not None:
                changed = toolchain_text(root, without) != base_toolchain
        except Refusal:
            changed = True
        if changed:
            names.append(name)
    return names


def print_unit_environment(root):
    """--unit-environment：打一行 `env 名=值 …`（按 shell 引好；一个都没有打空行），提交流程 ③ 把它原样接在 detached-run.sh start 的 `--` 后面。"""
    names = unit_environment_names(os.environ, root)
    say(("env " + " ".join(shlex.quote(f"{name}={os.environ[name]}") for name in names)) if names else "")
    note(f"  · 全局键与工具链认的会话变量（{'、'.join(names) if names else '没有'}）：经 detached-run.sh 起单元时把上一行原样接在 `--` 后面；"
         f"{'、'.join(UNIT_CARRIED_VARIABLES)} 单元本来就带")
    return 0


def report_flips(library):
    """里程碑全量跑过的这一版：逐条列翻面的（FLIPPED 序号、沿用的、现判的、变异名）。"""
    if not os.path.isfile(library.version_file("milestone-carry.tsv")):
        return
    flips = milestone_flips(library, library.read_results())
    for sequence, carried, fresh, name in flips:
        say(f"FLIPPED\t{sequence}\t{carried}\t{fresh}\t{name}")
    say(f"BROAD_MUTATION_MILESTONE flipped={len(flips)}")


def notice_if_head_moved(library):
    """主仓 HEAD 在导出范围上已经与树不同：只提示，不自己换树（换树就是另一版，跑了几天的这一版会被晾下）。"""
    commit = library.source.get("commit")
    if not commit or test_variable("BROAD_MUTATION_TREE_SOURCE"):
        return
    paths = library.source.get("export_paths") or []
    changed = subprocess.run(["git", "-C", REPOSITORY, "diff", "--quiet", commit, "HEAD", "--", *paths], stdin=subprocess.DEVNULL,
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if changed.returncode == 1:
        say(f"  · 注意：主仓 HEAD 在导出范围上已经与树（提交 {commit[:12]}）不同；这一趟仍跑树的那一版。要换到 HEAD：--sync-tree（旧版结果留着，不当新一版的结果）")


def stop_on_signal(_signal_number, _frame):
    raise Stopped()


def main(argv):
    arguments = parse_arguments(argv)
    if arguments.selftest:
        return run_selftest()
    if arguments.coverage_step:
        check_prefix("--coverage-step 在没改过的树上插桩跑全部测试")
        return coverage_step(arguments.coverage_step)
    library = Library(resolve_home(arguments.home))   # 里面先核只供自证的变量（check_test_only_variables），再核库目录落在哪
    if arguments.unit_environment:
        return print_unit_environment(library.tree if os.path.isdir(library.tree) else REPOSITORY)
    if arguments.list_items or arguments.summary or arguments.check_ignored:
        library.load_source(for_run=False)
        library.load_items()
        if arguments.list_items and arguments.changed_since:
            # 只读：列每条在 --changed-since 下是现判还是沿用、为什么；旧提交导出到临时目录，库目录一个字节不写
            library.load_manifest()
            pruner = Pruner(library, arguments)
            scratch = tempfile.mkdtemp(prefix="broad-mutation-changed-since.")
            try:
                old = old_tree_index(library, arguments, os.path.join(scratch, "tree"))
                for item in library.items:
                    changed = prune_rules().changed_since(old, pruner.tree, item, pruner.key_of(item))
                    carry, reason, _record = pruner.carry(item)
                    plan = "carry" if changed is None and carry else f"judge:{changed or reason}"
                    say(f"{item['sequence']}\t{item['package']}\t{item['name']}\t{plan}")
            finally:
                shutil.rmtree(scratch, ignore_errors=True)
            return 0
        if arguments.list_items:
            list_items(library)
            return 0
        if arguments.check_ignored:
            return check_ignored_command(library, arguments)
        return summarise(library)
    if arguments.sync_tree or arguments.import_results:
        check_prefix("--sync-tree 换的是库目录里全局的版本指针" if arguments.sync_tree else "--import-results 往本机这一版的结果表里加行")
    else:
        check_real_run_is_allowed()
    signal.signal(signal.SIGTERM, stop_on_signal)
    signal.signal(signal.SIGINT, stop_on_signal)
    library.lock_descriptor = acquire_lock(library.home, arguments.lock_wait)
    if arguments.sync_tree:
        sync_tree(library, arguments)
        return 0
    library.load_source(for_run=True)
    library.load_manifest()
    library.load_items()
    if arguments.import_results:
        return import_results(library, arguments.import_results)
    check_toolchain(library)
    check_global_key(library, arguments)
    if not break_is("no-restore"):
        interrupted = restore_original(library)
        if interrupted is not None:
            say(f"  · 树里留着第 {interrupted['sequence']} 条改过的 {interrupted['file']}：已照在途记录拷回原件")
            if break_is("interrupted-as-done"):
                item = library.items[interrupted["sequence"] - 1]
                library.record_result([item["sequence"], item["package"], item["name"], "caught", "one", "", "0.0", "0.0", now_text(), library.fingerprint, "break"])
        checked = verify_tree_is_pristine(library)
        say(f"  · 树与这一版的 manifest 逐文件相同、没有多出来的文件（核了 {checked} 个文件）")
    library.result_key(create=True)     # 要往结果表里写行：密钥没有就先建（文件头「结果行的校验列」），建不了当场拒
    results = library.read_results()
    blocking = results.blocking_problems()
    if blocking:
        refuse(f"这一版的结果表与清单对不上，照它选条会选错，一条都不跑：{'；'.join(blocking)}",
               f"python3 {SELF_PATH} --summary 看是哪几行与出路（清单被重新生成过、或结果表里混进了别的清单的行）", EXIT_INCONSISTENT)
    if results.forged:
        note(f"  · 结果表里 {results.forged_text()}：没有校验列或校验列对不上的行不当结论，那几条照没有定论选；--summary 照样判红，出路在它那里")
    notice_if_head_moved(library)
    return run_items(library, arguments)


# ── 自证 ───────────────────────────────────────────────────────────────────────────────────────────────────

SELFTEST_STEP_DRIVER = r'''#!/usr/bin/env python3
# 假的一步：照 plan.tsv（序号 <TAB> 步名 <TAB> 动作）办，记一笔调用（序号、步名、树里那份源码此刻是不是被改过）。
import os, sys, time
step, sequence, tree = sys.argv[1], sys.argv[2], sys.argv[3]
work = os.environ["BROAD_MUTATION_SELFTEST_WORK"]
if step.startswith("coverage-"):
    # 覆盖那一步：照 <工作目录>/<步名>.json 写这一版的覆盖（没备就写 null：不按覆盖挑，跑全部）
    import json
    spec = json.load(open(os.environ["BROAD_MUTATION_STEP_SPEC"]))
    prepared = os.path.join(work, step + ".json")
    with open(spec["out"], "w") as out:
        out.write(open(prepared).read() if os.path.exists(prepared) else "null\n")
    sys.exit(0)
with open(os.path.join(work, "selections.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_SELECTION', '')}\t{os.environ.get('BROAD_MUTATION_RUNNER_MODE', '')}\t"
              f"{os.environ.get('BROAD_MUTATION_STEP_TIME_LIMIT', '')}\n")
with open(os.path.join(work, "limits.log"), "a") as log:
    # 升档那几格读：哪一趟（first / escalated）、整步时限、内存上限
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_ATTEMPT', '')}\t{os.environ.get('BROAD_MUTATION_STEP_TIME_LIMIT', '')}\t"
              f"{os.environ.get('BROAD_MUTATION_STEP_MEMORY_CAP', '')}\n")
with open(os.path.join(work, "wrapper-env.log"), "a") as log:
    # 驱动交给内存包装的两样：按自己的上限排队（升档那趟）、撞总上限时先挨杀（改坏了源码的那几步）
    log.write(f"{sequence}\t{step}\t{os.environ.get('BROAD_MUTATION_STEP_ATTEMPT', '')}\t"
              f"{'queue-by-cap' if os.environ.get('RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP') else '-'}\t{'oom-first' if os.environ.get('RUN_WITH_MEMORY_CAP_OOM_FIRST') else '-'}\n")
mutated = any("MUTATED_" in open(os.path.join(tree, path)).read()
              for path in ("crates/singlefs-core/src/lib.rs", "crates/singlefs-checker/src/lib.rs"))
with open(os.path.join(work, "calls.log"), "a") as log:
    log.write(f"{sequence}\t{step}\t{'mutated' if mutated else 'pristine'}\n")
action = "ok"
for line in open(os.path.join(work, "plan.tsv")):
    fields = line.rstrip("\n").split("\t")
    if fields[:2] == [sequence, step]:
        action = fields[2]
if "|" in action:
    # 「原来那趟|升档那趟」：驱动升档重跑时 BROAD_MUTATION_STEP_ATTEMPT=escalated
    action = action.split("|", 1)[1 if os.environ.get("BROAD_MUTATION_STEP_ATTEMPT") == "escalated" else 0]
if action.startswith("peak:"):
    # 基线那一步：照包装的峰值表写一行（峰值字节、量的时刻、上限、键、线程数；时刻一段假的一步写 selftest），驱动拿它当这一步的基线峰值
    peaks = os.environ.get("RUN_WITH_MEMORY_CAP_PEAKS")
    if peaks:
        with open(peaks, "a") as handle:
            handle.write(f"{action.split(':', 1)[1]}\tselftest\t{os.environ.get('BROAD_MUTATION_STEP_MEMORY_CAP', '')}\t"
                         f"{os.environ.get('RUN_WITH_MEMORY_CAP_KEY', '')}\t1\n")
    sys.exit(0)
if action == "exit:253":
    print(f"run-with-memory-cap: 超过限时 {os.environ.get('BROAD_MUTATION_STEP_TIME_LIMIT')} 秒（自证的假一步）", flush=True)
    sys.exit(253)
if action == "exit:250":
    print(f"run-with-memory-cap: 撞了内存上限 {os.environ.get('BROAD_MUTATION_STEP_MEMORY_CAP')}（自证的假一步）", flush=True)
    sys.exit(250)
if action == "fail-nextest":
    print("        FAIL [   0.010s] (3/9) fake-package::fake_target the_case_that_catches_it")
    sys.exit(100)
if action.startswith("fail-nextest:"):
    print("        FAIL [   0.010s] (3/9) " + action.split(":", 1)[1])
    sys.exit(100)
if action == "fail-cargo":
    print("test tests::the_stage_two_case ... FAILED")
    sys.exit(101)
if action == "fail-compile":
    print("error[E0277]: the trait bound `Fake: Default` is not satisfied")
    print("  --> crates/singlefs-core/src/lib.rs:2:5")
    print("error: could not compile `singlefs-core` (lib) due to 1 previous error")
    sys.exit(101)
if action == "hang":
    # 挂住，并且自己再分出一个子进程一起挂着（真链上一步底下还有 cargo 与测试进程）；两个的进程号都记下来
    os.fork()
    with open(os.path.join(work, "hang.pids"), "a") as pids:
        pids.write(f"{os.getpid()}\n")
    open(os.path.join(work, "hanging"), "w").close()
    deadline = time.time() + 60
    while time.time() < deadline and not os.path.exists(os.path.join(work, "release")):
        time.sleep(0.05)
    sys.exit(0)
if action.startswith("sleep:"):
    # 慢的一步：先过一条用例（进度行要数到它），再睡那么多秒、绿着退出
    print("        PASS [   0.010s] (1/9) fake-package::fake_target the_case_that_passes", flush=True)
    time.sleep(float(action.split(":")[1]))
    sys.exit(0)
if action in ("cargo-build", "cargo-test"):
    # 假 cargo（换树后不留过时产物那一格）：照 cargo 的判法编两个包——fingerprint 不在、或有源文件比上次编出的新，才重编；
    # 测试那一步核编出来的是不是此刻树里的源码，不是就红（过时的产物）
    import hashlib
    target, stale = os.environ["CARGO_TARGET_DIR"], []
    for package in ("singlefs-core", "singlefs-checker"):
        root = os.path.join(tree, "crates", package)
        files = sorted(os.path.join(directory, name) for directory, _names, names in os.walk(root) for name in names)
        content = hashlib.sha256(b"".join(open(path, "rb").read() for path in files)).hexdigest()
        output = os.path.join(target, "debug", ".fingerprint", package + "-0123456789abcdef", "built-from")
        fresh = os.path.isfile(output) and all(os.stat(path).st_mtime_ns <= os.stat(output).st_mtime_ns for path in files)
        if action == "cargo-build" and not fresh:
            os.makedirs(os.path.dirname(output), exist_ok=True)
            with open(output, "w") as handle:
                handle.write(content)
            print(f"   Compiling {package}", flush=True)
            with open(os.path.join(work, "cargo.log"), "a") as log:
                log.write(f"{sequence}\tcompiled\t{package}\n")
        elif action == "cargo-test" and (not os.path.isfile(output) or open(output).read() != content):
            stale.append(package)
    if stale:
        print("        FAIL [   0.010s] (1/1) fake-package::fake_target built_from_stale_" + "_and_".join(stale))
        sys.exit(100)
    sys.exit(0)
if action.startswith("exit:"):
    sys.exit(int(action.split(":")[1]))
sys.exit(0)
'''

SELFTEST_LIST_DRIVER = r'''#!/usr/bin/env python3
# 假的 cargo mutants --list --json --diff：把备好的那一份按包打出来。
import json, os, sys
with open(os.path.join(os.environ["BROAD_MUTATION_SELFTEST_WORK"], "mutants.json")) as handle:
    print(json.dumps([mutant for mutant in json.load(handle) if mutant["package"] == sys.argv[1]]))
'''

SELFTEST_SOURCES = {
    "crates/singlefs-core/src/lib.rs": "pub fn one() -> u32 {\n    1\n}\npub fn two() -> u32 {\n    2\n}\npub fn three() -> u32 {\n    3\n}\n"
                                       "pub fn four() -> u32 {\n    4\n}\npub fn five() -> u32 {\n    5\n}\npub fn six() -> u32 {\n    6\n}\n",
    "crates/singlefs-checker/src/lib.rs": "pub fn seven() -> u32 {\n    7\n}\npub fn eight() -> u32 {\n    8\n}\n",
    "Cargo.toml": "[workspace]\n",
    # 忽略用例登记那几格用：lib 里模块中的两条（一条慢档用例、一条工具）、tests 里顶层的慢档用例与大档、嵌套模块里的工具；包是 singlefs-format，不碰清单的两个包
    "crates/singlefs-format/src/lib.rs": '#[cfg(test)]\nmod tests {\n    #[test]\n    #[ignore = "harness 慢档用例：debug 下单条跑过 60 秒"]\n    fn in_lib_slow() {}\n\n'
                                         '    #[test]\n    #[ignore = "收缩一个种子：SINGLEFS_SELFTEST_SEED 必给"]\n    fn in_lib_tool() {}\n}\n',
    "crates/singlefs-format/tests/ignored_cases.rs": '#[test]\n#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒"]\nfn slow_one() {}\n\n'
                                                     '#[test]\n#[ignore = "大档：规模从环境变量取"]\nfn env_large_tier() {}\n\n'
                                                     'mod inner {\n    #[test]\n    #[ignore = "收缩一个种子：SINGLEFS_SELFTEST_SEED 必给"]\n    fn env_tool() {}\n}\n',
}
SELFTEST_EXCLUSIONS = ("# 自证用的排除表\n"
                       "singlefs-format tests::in_lib_tool\t自证：库里模块中的工具用例\n"
                       "singlefs-format::ignored_cases env_large_tier\t自证：按环境变量起的大档\n"
                       "singlefs-format::ignored_cases inner::env_tool\t自证：嵌套模块里的工具用例\n")
# 序号 → 假的一步怎么答（没写的步答 0）：各种结局，两段各有；第二段的三种测试步各答一次
SELFTEST_PLAN = [("1", "one-test", "fail-nextest"), ("2", "one-build", "fail-compile"), ("3", "one-test", "exit:253"),
                 ("5", "two-test-doc", "fail-cargo"), ("6", "one-test", "exit:250"), ("7", SLOW_STEP_NAME, "fail-nextest"),
                 ("8", "two-test-checker-tier", "exit:253")]
SELFTEST_EXPECTED = {"1": ("caught", "one"), "2": ("unviable", "one"), "3": ("timeout", "one"), "4": ("missed", "two"),
                     "5": ("caught", "two"), "6": ("memory-cap", "one"), "7": ("caught", "two"), "8": ("timeout", "two")}
SELFTEST_EXPECTED_SUMMARY = "total=8 caught=3 missed=1 unviable=1 timeout=2 memory_cap=1 stage_one_only=0 driver_error=0 not_run=0"
SELFTEST_STAGE_TWO_STEPS = ["two-build-harness", "two-test-doc", SLOW_STEP_NAME, "two-build-checker-tier", "two-test-checker-tier"]
SELFTEST_NEXTEST_CASE = "fake-package::fake_target the_case_that_catches_it"


def selftest_mutants(sources=None):
    """备一份假的原始清单：每个文件里每个返回值各一条，把那个数字换成 MUTATED_<序号>()。"""
    mutants, sequence = [], 0
    for package, path in (("singlefs-core", "crates/singlefs-core/src/lib.rs"), ("singlefs-checker", "crates/singlefs-checker/src/lib.rs")):
        original = (sources or SELFTEST_SOURCES)[path]
        for line_index, line in enumerate(original.split("\n")):
            if not line.strip().isdigit():
                continue
            sequence += 1
            start, end = (line_index + 1, 5), (line_index + 1, len(line) + 1)
            replacement = f"MUTATED_{sequence}()"
            mutated = replace_span(original, start, end, replacement + MUTATION_MARKER)
            diff = "".join(difflib.unified_diff(original.splitlines(keepends=True), mutated.splitlines(keepends=True), f"a/{path}", f"b/{path}"))
            mutants.append({"package": package, "file": path, "name": f"{path}:{line_index + 1}:5: replace the value with {replacement}",
                            "span": {"start": {"line": start[0], "column": start[1]}, "end": {"line": end[0], "column": end[1]}},
                            "replacement": replacement, "diff": diff})
    return mutants


def process_is_running(process_id):
    """进程还在跑（不算已经退出、等着被收的）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] != "Z"
    except (OSError, IndexError):
        return False


class Selftest:
    def __init__(self, work):
        self.work = work
        self.cells = 0
        self.failures = 0
        self.source = os.path.join(work, "source")
        for path, text in SELFTEST_SOURCES.items():
            os.makedirs(os.path.dirname(os.path.join(self.source, path)), exist_ok=True)
            with open(os.path.join(self.source, path), "w") as handle:
                handle.write(text)
        self.mutants = selftest_mutants()
        with open(os.path.join(work, "mutants.json"), "w") as handle:
            json.dump(self.mutants, handle)
        for name, text in (("step-driver.py", SELFTEST_STEP_DRIVER), ("list-driver.py", SELFTEST_LIST_DRIVER)):
            with open(os.path.join(work, name), "w") as handle:
                handle.write(text)
            os.chmod(os.path.join(work, name), 0o755)
        self.write_plan(SELFTEST_PLAN)
        self.exclusions_path = os.path.join(work, "exclusions.tsv")
        with open(self.exclusions_path, "w") as handle:
            handle.write(SELFTEST_EXCLUSIONS)
        self.environment = dict(os.environ, BROAD_MUTATION_SELFTEST_WORK=work, BROAD_MUTATION_STEP_DRIVER=os.path.join(work, "step-driver.py"),
                                BROAD_MUTATION_LIST_DRIVER=os.path.join(work, "list-driver.py"), BROAD_MUTATION_TREE_SOURCE=self.source,
                                BROAD_MUTATION_ALLOW_VOLATILE_HOME="1", BROAD_MUTATION_EXCLUSIONS=self.exclusions_path,
                                BROAD_MUTATION_RESULT_KEY=self.result_key_path())
        for name in ("SINGLEFS_BROAD_MUTATION_HOME", HEAVY_PREFIX_VARIABLE, "BROAD_MUTATION_KILL_AT", "BROAD_MUTATION_STEP_VIA_WRAPPER", "BROAD_MUTATION_TOOLCHAIN",
                     "BROAD_MUTATION_PROGRESS_INTERVAL", "BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE", "BROAD_MUTATION_RUSTC", "RUSTUP_TOOLCHAIN"):
            self.environment.pop(name, None)
        self.slice = f"singlefs_broad_mutation_selftest_{os.getpid()}.slice"
        self.slice_used = False

    def write_plan(self, plan):
        with open(os.path.join(self.work, "plan.tsv"), "w") as handle:
            handle.write("".join("\t".join(row) + "\n" for row in plan))

    def result_key_path(self):
        """自证里的结果行密钥（库目录之外：各个库都在工作目录下各自的子目录里，密钥在 result-key/ 下），驱动头一回写行时建它。"""
        return os.path.join(self.work, "result-key", "broad-mutation-result.key")

    def signed(self, fields):
        """自证手写的行照驱动的写法签上校验列（用自证这一份密钥）；密钥还没建交原样。"""
        key, _problem = read_result_key(self.result_key_path())
        return signed_result_row(key, fields) if key is not None else fields

    def command(self, home, *arguments):
        return [sys.executable, SELF_PATH, "--home", os.path.join(self.work, home), "--timeout-floor", "5", "--lock-wait", "30", "--stop-grace", "2", *arguments]

    def run(self, home, *arguments, environment=None, **extra_environment):
        environment = dict(environment or self.environment, **extra_environment)
        return subprocess.run(self.command(home, *arguments), capture_output=True, text=True, stdin=subprocess.DEVNULL, env=environment)

    def calls(self):
        """调用记录：[(序号, 步名, mutated / pristine)]。"""
        path = os.path.join(self.work, "calls.log")
        if not os.path.exists(path):
            return []
        with open(path) as handle:
            return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]

    def called_sequences(self):
        """被调过的序号（基线是 0）；在没改过的树上验环境的那几步不算在里面。"""
        return {call[0] for call in self.calls() if call[0] != ENVIRONMENT_SEQUENCE}

    def environment_steps(self):
        return [call[1] for call in self.calls() if call[0] == ENVIRONMENT_SEQUENCE]

    def clear_calls(self):
        path = os.path.join(self.work, "calls.log")
        if os.path.exists(path):
            os.remove(path)

    def version_directory(self, home):
        """这份库的版本指针此刻指着的那一版的目录。"""
        with open(os.path.join(self.work, home, "tree.source")) as handle:
            return os.path.join(self.work, home, "versions", json.load(handle)["fingerprint"])

    def results_path(self, home):
        return os.path.join(self.version_directory(home), "results.tsv")

    def rows(self, home):
        """这份库的结果表里成形的行。"""
        return read_table(self.results_path(home), len(RESULT_COLUMNS)).rows

    def states(self, home):
        """--list-items 报的各条状态：{序号: 状态}。"""
        listed = self.run(home, "--list-items")
        return {fields[0]: fields[3] for fields in (line.split("\t") for line in listed.stdout.splitlines()) if len(fields) == 4}

    def tree_is_pristine(self, home, sources=None):
        tree = os.path.join(self.work, home, "tree")
        for path, text in (sources or SELFTEST_SOURCES).items():
            with open(os.path.join(tree, path)) as handle:
                if handle.read() != text:
                    return False
        return files_in_tree(tree) == set(sources or SELFTEST_SOURCES)

    def check(self, condition, wrong, howto):
        self.cells += 1
        if not condition:
            self.failures += 1
            print(f"  ✗ 自证：{wrong}", file=sys.stderr)
            print(f"     → 怎么办：{howto}", file=sys.stderr)


def wait_until(condition, seconds):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if condition():
            return True
        time.sleep(0.05)
    return condition()


def summary_field(text, key):
    found = re.search(rf"\b{key}=(\d+)", text)
    return int(found.group(1)) if found else None


def selftest_basic_cells(selftest):
    """第一版的那几格：临时库目录拒、导出与列项、项写错、点名与两段、杀掉再起、各种结局、汇总、--from 与 --shard、脏树。"""
    work = selftest.work
    # 库目录落在开机会清空的目录里拒（不带只供测试的放行）
    volatile_home = f"/dev/shm/broad-mutation-selftest-{os.getpid()}"
    # 不带任何只供测试的变量：带着的话库目录在自证工作目录之外，会先被「只供自证的变量不碰别的库」那一拒拦下，验不到这一格要验的「开机会清空」
    volatile_environment = {name: value for name, value in selftest.environment.items() if name not in TEST_ONLY_VARIABLES}
    refused = subprocess.run([sys.executable, SELF_PATH, "--home", volatile_home, "--sync-tree"], capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, env=volatile_environment)
    selftest.check(refused.returncode == EXIT_USAGE and not os.path.exists(volatile_home) and "开机会清空" in refused.stderr,
                   f"库目录落在 /dev/shm 里没拒（退 {refused.returncode}）：{refused.stderr}", "看 resolve_home：/tmp、/run、/dev/shm 与 tmpfs 都要拒")
    # 导出树、生成清单；--list-items 逐行四段、一步都不跑
    synced = selftest.run("first", "--sync-tree")
    listed = selftest.run("first", "--list-items")
    listed_lines = [line.split("\t") for line in listed.stdout.splitlines()]
    selftest.check(synced.returncode == 0 and listed.returncode == 0 and len(listed_lines) == len(selftest.mutants)
                   and all(len(fields) == 4 and fields[3] == NOT_RUN for fields in listed_lines)
                   and [fields[0] for fields in listed_lines] == [str(index + 1) for index in range(len(selftest.mutants))] and not selftest.calls(),
                   f"--sync-tree / --list-items 不对（退 {synced.returncode} / {listed.returncode}，{len(listed_lines)} 行，调了 {len(selftest.calls())} 步）：{synced.stderr}{listed.stderr}",
                   "看 sync_tree、generate_list 与 list_items：清单每条一行「序号、包、变异名、状态」，列项不许起任何一步")
    # 项写错退 2、一步都不跑
    unknown = selftest.run("first", "--item", "no-such-mutant", "--skip-stage-two")
    selftest.check(unknown.returncode == EXIT_USAGE and not selftest.calls(), f"不认得的项没拒（退 {unknown.returncode}，调了 {len(selftest.calls())} 步）",
                   "看 select_items：清单里找不到的序号或变异名要退 2 并写怎么查")
    # 点名（一条按序号、一条按变异名）只跑第一段：只有基线与这两条被调，第二段的步一次都没调；判完树回到原样
    name_of_seven = listed_lines[6][2] if len(listed_lines) >= 7 and len(listed_lines[6]) == 4 else "?"
    named = selftest.run("first", "--item", "1", "--item", name_of_seven, "--skip-stage-two")
    calls = selftest.calls()
    rows = {row[0]: row for row in selftest.rows("first")}
    selftest.check(named.returncode == 0 and selftest.called_sequences() == {"0", "1", "7"} and not any(call[1].startswith("two-") for call in calls)
                   and all(call[2] == ("pristine" if call[0] == "0" else "mutated") for call in calls)
                   and set(rows) == {"1", "7"} and rows["1"][3:6] == ["caught", "one", SELFTEST_NEXTEST_CASE]
                   and rows["7"][3:5] == [STAGE_ONE_ONLY, "one"] and selftest.tree_is_pristine("first"),
                   f"点名两条只跑第一段不对（退 {named.returncode}；调用 {calls}；行 {sorted(rows.values())}）：{named.stderr}",
                   "看 select_items 的点名那一支、process_item 的第一段与 --skip-stage-two：基线在没改过的树上跑、每条在改过的树上跑、判完拷回原件")
    # --stage-two-only：只给「只过了第一段」的那一条补第二段，第一段的步不再调；第二段里 harness 慢档用例那一步抓到它，带测试名
    selftest.clear_calls()
    second = selftest.run("first", "--stage-two-only")
    calls = selftest.calls()
    selftest.check(second.returncode == 0 and selftest.called_sequences() == {"0", "7"} and all(call[1].startswith("two-") for call in calls)
                   and [row[3:6] for row in selftest.rows("first") if row[0] == "7"] == [[STAGE_ONE_ONLY, "one", ""], ["caught", "two", SELFTEST_NEXTEST_CASE]],
                   f"--stage-two-only 不对（退 {second.returncode}；调用 {calls}；第 7 条的行 {[row[3:6] for row in selftest.rows('first') if row[0] == '7']}）：{second.stderr}",
                   "看 select_items：只选状态是 stage-one-only 的、不跑第一段；process_item 第二段哪一步红了就记抓到并带测试名")
    # 跑到一半 KILL：第 3 条的测试挂住时杀驱动。杀之前前两条的行已经落盘；杀之后树里留着改过的文件与在途记录
    selftest.clear_calls()
    selftest.write_plan([row for row in SELFTEST_PLAN if row[0] != "3"] + [("3", "one-test", "hang")])
    selftest.run("second", "--sync-tree")
    driver = subprocess.Popen(selftest.command("second"), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL, env=selftest.environment)
    hanging = wait_until(lambda: os.path.exists(os.path.join(work, "hanging")) or driver.poll() is not None, 60)
    rows_before_stop = {row[0] for row in selftest.rows("second")}
    driver.kill()   # 只点自己起的这一个进程
    driver.wait()
    selftest.check(hanging and os.path.exists(os.path.join(work, "hanging")) and rows_before_stop == {"1", "2"},
                   f"杀驱动之前，前两条的结果行没有都落盘（落了 {sorted(rows_before_stop)}）", "看 record_result：每条判完立刻追加并 fsync，不攒到收尾")
    selftest.check(not selftest.tree_is_pristine("second") and os.path.exists(os.path.join(work, "second", "inflight.json")),
                   "驱动被杀之后树里没有留着改过的文件与在途记录（后面几格的前提不成立）", "看 apply_mutation：先存原件、记在途记录，再改文件")
    # 被杀的驱动留下的那一步还活着时锁还占着：不等就退 4、一步都不跑
    selftest.clear_calls()
    locked = selftest.run("second", "--lock-wait", "0")
    selftest.check(locked.returncode == EXIT_LOCKED and not selftest.calls(), f"上一趟留下的进程还占着锁时没拒（退 {locked.returncode}）",
                   "看 acquire_lock 与 run_step 的 pass_fds：锁的描述符要传给每一步")
    # 放开挂住的那一步，重跑同一条命令：有结果的条调用 0 次、基线不重跑；被打断的那一条重跑；树先恢复干净、跑完还是原样
    open(os.path.join(work, "release"), "w").close()
    selftest.write_plan(SELFTEST_PLAN)
    resumed = selftest.run("second")
    rows = selftest.rows("second")
    called = selftest.called_sequences()
    selftest.check(resumed.returncode == 0 and not called & {"0", "1", "2"},
                   f"重起之后已有结果的条或基线被重跑了（退 {resumed.returncode}；被调的序号 {sorted(called)}）：{resumed.stderr}",
                   "看 select_items：这一版有定论的条一条都不选；基线有绿记录的不重跑")
    selftest.check("3" in called and [row[3] for row in rows if row[0] == "3"] == ["timeout"] and "已照在途记录拷回原件" in resumed.stdout,
                   f"被打断的第 3 条没有重跑、或被当成已完成（它的行：{[row[3:5] for row in rows if row[0] == '3']}）",
                   "看 main 里开跑时的 restore_original：只拷回原件，不给被打断的那一条落行")
    selftest.check(selftest.tree_is_pristine("second") and not os.path.exists(os.path.join(work, "second", "inflight.json")),
                   "重起跑完之后树没有回到原样、或在途记录还在", "看 restore_original 与 process_item 的 finally")
    os.remove(os.path.join(work, "release"))
    os.remove(os.path.join(work, "hanging"))
    # 两段式的结局：抓到（第一段、文档测试、慢档用例各一）、没红、无效、超时（两段各一）、内存撞顶，逐条对
    final = {row[0]: (row[3], row[4]) for row in rows if row[3] != STAGE_ONE_ONLY}
    selftest.check(final == SELFTEST_EXPECTED and [row[5] for row in rows if row[0] == "5" and row[3] == "caught"] == ["tests::the_stage_two_case"],
                   f"两段式的结局不对：{sorted(final.items())}", "看 process_item 的 run_stage：第二段全绿才记 missed，抓到要带第一条失败的测试名")
    # 汇总八栏加起来等于清单条数，没红的逐条列名
    summary = selftest.run("second", "--summary")
    selftest.check(summary.returncode == EXIT_GATE_RED and SELFTEST_EXPECTED_SUMMARY in summary.stdout and "verdict=red" in summary.stdout
                   and len([line for line in summary.stdout.splitlines() if line.startswith("MISSED\t")]) == 1,
                   f"汇总不对（退 {summary.returncode}）：{summary.stdout}{summary.stderr}", "看 summarise 的八栏与 MISSED 明细")
    # 第二段只补第一段没跑到的：没红的第 4 条第二段依次调的就是这五步（文档测试、慢档用例、快档），命令里没有重跑第一段的那一遍
    stage_two_of_four = [call[1] for call in selftest.calls() if call[0] == "4" and call[1].startswith("two-")]
    selftest_exclusions = load_exclusions(selftest.exclusions_path)
    slow_filter = exclusion_filter(selftest_exclusions)
    table = {step[0]: step[3] for step in step_table(argparse.Namespace(threads=3, stage_two_eatmydata=False), selftest_exclusions)}
    table_without_exclusions = {step[0]: step[3] for step in step_table(argparse.Namespace(threads=3, stage_two_eatmydata=False), [])}
    commands_are_right = (
        table.get("two-test-doc") == ["cargo", "test", "--locked", "--doc", "-p", "singlefs-core", "-p", "singlefs-checker"]
        and table.get(SLOW_STEP_NAME, [])[:3] == ["eatmydata", "cargo", "nextest"] and OLD_SLOW_STEP_NAME not in table
        and table_without_exclusions[SLOW_STEP_NAME][-2:] == ["--run-ignored", "only"]
        and table.get("two-test-checker-tier") == ["cargo", "test", "--locked", "--release", "-p", CHECKER_TIER_PACKAGE, "--lib", "--tests"]
        and not any("--include-ignored" in command or "--ignored" in command for command in table.values())
        and slow_filter is not None and slow_filter.startswith("not (") and table["one-test"] + ["--run-ignored", "only", "-E", slow_filter] == table[SLOW_STEP_NAME]
        and not any(name.startswith("two-test") and command[-len(table["one-test"]):] == table["one-test"] for name, command in table.items()))
    selftest.check(stage_two_of_four == SELFTEST_STAGE_TWO_STEPS and commands_are_right,
                   f"第二段的步不对：第 4 条调的是 {stage_two_of_four}；各步的命令 {table}",
                   "看 step_table：第二段 = 文档测试（被变异的两个包）+ harness 慢档用例（第一段的起法加 --run-ignored only 与排除表的 -E）+ checker 档快档，不重跑第一段那一遍")
    # 汇总对不上必须报：给第 4 条追加一行认不出的结局
    results_path = selftest.results_path("second")
    fingerprint = os.path.basename(os.path.dirname(results_path))
    name_of_four = listed_lines[3][2] if len(listed_lines) >= 4 and len(listed_lines[3]) == 4 else "?"
    append_row(results_path, RESULT_COLUMNS, selftest.signed(["4", "singlefs-core", name_of_four, "no-such-outcome", "one", "", "0", "0", now_text(), fingerprint, ""]))
    mismatched = selftest.run("second", "--summary")
    selftest.check(mismatched.returncode == EXIT_INCONSISTENT, f"结果表里有认不出的结局时汇总没判红（退 {mismatched.returncode}）",
                   "看 summarise 末尾的 problems：对不上要退 1")
    # --from 与 --shard（只跑第一段）
    selftest.clear_calls()
    selftest.run("third", "--sync-tree")
    from_seven = selftest.run("third", "--from", "7", "--skip-stage-two")
    selftest.check(from_seven.returncode == 0 and selftest.called_sequences() == {"0", "7", "8"},
                   f"--from 7 跑的不是第 7、8 条（退 {from_seven.returncode}；{sorted(selftest.called_sequences())}）", "看 select_items 的 start_from")
    selftest.clear_calls()
    shard = selftest.run("third", "--shard", "0/2", "--skip-stage-two")
    selftest.check(shard.returncode == 0 and selftest.called_sequences() == {"2", "4", "6"},
                   f"--shard 0/2 跑的不是还没结果的偶数序号（退 {shard.returncode}；{sorted(selftest.called_sequences())}）", "看 select_items 的 shard：序号 mod n == k")
    # 树脏了又没有在途记录：拒绝跑、退 5、一步都不调
    selftest.clear_calls()
    with open(os.path.join(work, "third", "tree", "crates/singlefs-core/src/lib.rs"), "a") as handle:
        handle.write("// 手改\n")
    dirty = selftest.run("third", "--skip-stage-two")
    selftest.check(dirty.returncode == EXIT_DIRTY_TREE and not selftest.calls(), f"树脏了又恢复不了时没拒（退 {dirty.returncode}，调了 {len(selftest.calls())} 步）",
                   "看 verify_tree_is_pristine：与 manifest 对不上又没有在途记录要退 5")
    return listed_lines


def selftest_state_cells(selftest):
    """一条的状态不取最后一行、点名默认跳过已有定论的条、--stage-two-only 要过过第一段（库「first」接着用：第 1 条第一段抓到，第 7 条第二段抓到）。"""
    # 点名已有定论的条：默认跳过、一步不调、不加行；带 --rerun 才重跑并追加一行，旧行留着
    selftest.clear_calls()
    skipped = selftest.run("first", "--item", "1", "--skip-stage-two")
    rows_of_one = [row for row in selftest.rows("first") if row[0] == "1"]
    selftest.check(skipped.returncode == 0 and "1" not in selftest.called_sequences() and len(rows_of_one) == 1 and "跳过" in skipped.stdout,
                   f"点名已有定论的条没跳过（退 {skipped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；第 1 条 {len(rows_of_one)} 行）：{skipped.stdout}",
                   "看 select_items 的点名那一支：已有定论的条不带 --rerun 不选，逐条说一句")
    again = selftest.run("first", "--item", "1", "--skip-stage-two", "--rerun")
    selftest.check(again.returncode == 0 and len([row for row in selftest.rows("first") if row[0] == "1"]) == 2,
                   f"点名加 --rerun 没有重跑并追加新行（退 {again.returncode}）：{again.stderr}", "看 select_items：带 --rerun 的点名不看状态，结果表只追加")
    # 第二段的定论不被后来的 stage-one-only 行降级：第 7 条（第二段抓到）点名重跑只跑第一段，状态还是 caught，之后不带 --rerun 的续跑不再碰它
    downgrade = selftest.run("first", "--item", "7", "--skip-stage-two", "--rerun")
    last_of_seven = [row[3] for row in selftest.rows("first") if row[0] == "7"][-1:]
    selftest.clear_calls()
    resumed = selftest.run("first", "--from", "7", "--package", "singlefs-checker")
    selftest.check(downgrade.returncode == 0 and last_of_seven == [STAGE_ONE_ONLY] and selftest.states("first").get("7") == "caught"
                   and resumed.returncode == 0 and "7" not in selftest.called_sequences(),
                   f"第二段的定论被后来的 stage-one-only 行降级了（第 7 条最后一行 {last_of_seven}，状态 {selftest.states('first').get('7')}；续跑里被调的 {sorted(selftest.called_sequences())}）",
                   "看 state_of_outcomes：caught / missed / unviable 取其中最后一行，stage-one-only 行不盖它们")
    # 抓到的条点名重跑时机器忙、记了超时：状态还是 caught
    selftest.write_plan([("1", "one-test", "exit:253")])
    busy = selftest.run("first", "--item", "1", "--skip-stage-two", "--rerun")
    selftest.write_plan(SELFTEST_PLAN)
    outcomes_of_one = [row[3] for row in selftest.rows("first") if row[0] == "1"]
    selftest.check(busy.returncode == 0 and outcomes_of_one == ["caught", "caught", "timeout"] and selftest.states("first").get("1") == "caught",
                   f"抓到的条被后来的超时行盖掉了（第 1 条各行 {outcomes_of_one}，状态 {selftest.states('first').get('1')}）",
                   "看 state_of_outcomes：timeout / memory-cap 是环境型的结局，不盖掉前面的 caught / missed / unviable")
    # --stage-two-only 点到没过过第一段的条：拒，退 2，一步都不调
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    not_passed = selftest.run("first", "--item", "2", "--item", "7", "--stage-two-only")
    selftest.check(not_passed.returncode == EXIT_USAGE and not selftest.calls() and len(selftest.rows("first")) == rows_before and "第 2 条" in not_passed.stderr,
                   f"--stage-two-only 点到没过过第一段的条没拒（退 {not_passed.returncode}，调了 {len(selftest.calls())} 步）：{not_passed.stderr}",
                   "看 select_items：点名加 --stage-two-only 的条要有 stage-one-only 行，没有就拒")


def selftest_table_cells(selftest, listed_lines):
    """结果行与清单核变异名、末行没写完、认不出的退出码。"""
    work = selftest.work
    # 清单被重新生成、次序变了：结果行的变异名与清单同序号的对不上，汇总判红、续跑拒
    selftest.run("names", "--sync-tree")
    selftest.run("names", "--from", "7", "--skip-stage-two")
    list_path = os.path.join(selftest.version_directory("names"), "list.json")
    with open(list_path) as handle:
        items = json.load(handle)
    items[6], items[7] = dict(items[7], sequence=7), dict(items[6], sequence=8)
    with open(list_path, "w") as handle:
        json.dump(items, handle)
    selftest.clear_calls()
    summary = selftest.run("names", "--summary")
    resumed = selftest.run("names", "--skip-stage-two")
    selftest.check(summary.returncode == EXIT_INCONSISTENT and "变异名" in summary.stderr and resumed.returncode == EXIT_INCONSISTENT and not selftest.calls(),
                   f"结果行的变异名与清单同序号的不同时没判红（汇总退 {summary.returncode}，续跑退 {resumed.returncode}、调了 {len(selftest.calls())} 步）",
                   "看 Library.read_results 的 mismatched_names 与 main 里开跑前的 blocking_problems")
    # 结果行写到一半被杀：末行不算数、汇总不因此红；下一次追加之前截掉，那一条重跑、只留一行完整的
    selftest.run("partial", "--sync-tree")
    half = selftest.run("partial", "--item", "1", "--skip-stage-two", BROAD_MUTATION_KILL_AT="row-half-written")
    results_path = selftest.results_path("partial")
    with open(results_path, "rb") as handle:
        ends_mid_row = not handle.read().endswith(b"\n")
    summary = selftest.run("partial", "--summary")
    rerun = selftest.run("partial", "--item", "1", "--skip-stage-two")
    with open(results_path, "rb") as handle:
        text_after = handle.read()
    rows_of_one = [row[3] for row in selftest.rows("partial") if row[0] == "1"]
    after = selftest.run("partial", "--summary")
    selftest.check(half.returncode == -signal.SIGKILL and ends_mid_row and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "not_run") == 8
                   and rerun.returncode == 0 and text_after.endswith(b"\n") and rows_of_one == ["caught"] and after.returncode == EXIT_GATE_RED
                   and summary_field(after.stdout, "caught") == 1 and os.path.exists(results_path + ".dropped-partial-rows"),
                   f"末行没写完的结果表没处理对（写一半那趟退 {half.returncode}、末行不完整 {ends_mid_row}；汇总退 {summary.returncode}；重跑退 {rerun.returncode}、"
                   f"第 1 条各行 {rows_of_one}；再汇总退 {after.returncode}）：{summary.stderr}{after.stderr}",
                   "看 read_table 的 partial_tail（末行没有换行结尾的不算数）与 append_rows 里的 drop_partial_tail（追加之前截掉）")
    # 一步退了认不出的码：落一行 driver-error、往下走，汇总单列；编译步退 101 而没有编译错误的不算无效；续跑时重试这两条
    selftest.run("unknown", "--sync-tree")
    selftest.write_plan([("2", "one-build", "exit:101"), ("3", "one-test", "exit:104")])
    selftest.clear_calls()
    first = selftest.run("unknown", "--skip-stage-two")
    outcomes = {row[0]: row[3] for row in selftest.rows("unknown")}
    summary = selftest.run("unknown", "--summary")
    doubt_checked = selftest.environment_steps()
    selftest.clear_calls()
    retried = selftest.run("unknown", "--skip-stage-two")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and outcomes.get("2") == DRIVER_ERROR and outcomes.get("3") == DRIVER_ERROR and outcomes.get("4") == STAGE_ONE_ONLY
                   and len(outcomes) == 8 and summary.returncode == EXIT_GATE_RED and summary_field(summary.stdout, "driver_error") == 2
                   and len([line for line in summary.stdout.splitlines() if line.startswith("DRIVER_ERROR\t")]) == 2
                   and doubt_checked == ["one-build"] and retried.returncode == 0 and selftest.called_sequences() == {"2", "3"},
                   f"认不出的退出码没处理对（退 {first.returncode}；各条 {sorted(outcomes.items())}；汇总退 {summary.returncode}：{summary.stdout.splitlines()[:1]}；"
                   f"当场验的步 {doubt_checked}；续跑退 {retried.returncode}、被调的序号 {sorted(selftest.called_sequences())}）：{first.stderr}",
                   "看 process_item 的 run_stage：认不出的码记 driver-error 往下走；编译步退 101 要日志里有带源码位置的编译错误才记 unviable；"
                   "select_items 把 driver-error 的条当还没跑")
    return work


def selftest_environment_cells(selftest):
    """基线绿过之后环境坏了要发现得了：重起先验编译、连着几条无效或秒红的抓到就在原件上重验。"""
    # 重起（基线已有绿记录）先在没改过的树上验一次编译：红了退 6，一条都不跑
    selftest.run("environment", "--sync-tree")
    selftest.run("environment", "--item", "1", "--skip-stage-two")
    selftest.write_plan(SELFTEST_PLAN + [(ENVIRONMENT_SEQUENCE, "one-build", "exit:101")])
    selftest.clear_calls()
    broken = selftest.run("environment", "--item", "3", "--skip-stage-two")
    selftest.check(broken.returncode == EXIT_BASELINE_RED and selftest.environment_steps() == ["one-build"] and not selftest.called_sequences()
                   and not [row for row in selftest.rows("environment") if row[0] == "3"] and "疑似环境坏了" in broken.stderr,
                   f"基线绿过之后编译坏了，重起没发现（退 {broken.returncode}；验的步 {selftest.environment_steps()}；被调的序号 {sorted(selftest.called_sequences())}）：{broken.stderr}",
                   "看 Runner.verify_environment_at_start：这一趟没现跑基线的那一段，开跑前在没改过的树上把编译步各跑一遍")
    # 连续 K 条无效而原件上编译照过：不是环境，接着跑
    selftest.write_plan([])
    selftest.run("streak", "--sync-tree")
    selftest.run("streak", "--item", "8", "--skip-stage-two")
    selftest.write_plan([(str(sequence), "one-build", "fail-compile") for sequence in (1, 2, 3)])
    selftest.clear_calls()
    continued = selftest.run("streak", "--package", "singlefs-core", "--skip-stage-two", "--suspect-streak", "3")
    outcomes = {row[0]: row[3] for row in selftest.rows("streak")}
    selftest.check(continued.returncode == 0 and selftest.environment_steps() == ["one-build", "one-build"]
                   and [outcomes.get(str(sequence)) for sequence in (1, 2, 3, 4, 5, 6)] == ["unviable"] * 3 + [STAGE_ONE_ONLY] * 3 and "不是环境" in continued.stdout,
                   f"连续无效而原件上编译照过时没有接着跑（退 {continued.returncode}；验的步 {selftest.environment_steps()}；各条 {sorted(outcomes.items())}）：{continued.stderr}",
                   "看 SuspectStreaks.saw：到了 K 条就在没改过的树上重跑那一步，绿了清掉这一串接着跑")
    # 连续 K 条秒红的抓到而原件上那一步也红：退 6 报疑似环境坏了、列出这 K 条的点名重跑命令，后面的条不跑
    selftest.write_plan([])
    selftest.run("fast", "--sync-tree")
    selftest.run("fast", "--item", "8", "--skip-stage-two")
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in (1, 2, 3, 4)] + [(ENVIRONMENT_SEQUENCE, "one-test", "exit:100")])
    selftest.clear_calls()
    stopped = selftest.run("fast", "--skip-stage-two", "--suspect-streak", "3", "--suspect-test-seconds", "30")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(stopped.returncode == EXIT_BASELINE_RED and selftest.called_sequences() == {"1", "2", "3"} and selftest.environment_steps() == ["one-build", "one-test"]
                   and "--item 1 --item 2 --item 3 --rerun" in stopped.stderr,
                   f"连续 3 条秒红的抓到、原件上那一步也红，没有停下报疑似环境坏了（退 {stopped.returncode}；被调的序号 {sorted(selftest.called_sequences())}；"
                   f"验的步 {selftest.environment_steps()}）：{stopped.stderr}",
                   "看 SuspectStreaks.saw 与 Runner.verify_on_pristine_tree：原件上那一步红了退 6，出路里列出这几条的点名重跑命令")


def selftest_tree_cells(selftest):
    """换树的次序、核树看多出来的文件、恢复原件的临时件不进树、工具链进指纹。"""
    work = selftest.work
    # 树里留着旧版驱动恢复原件的临时件：认出来删掉、照跑；多出别的文件：拒，退 5，一步都不调
    selftest.run("extras", "--sync-tree")
    leftover = os.path.join(work, "extras", "tree", "crates/singlefs-core/src/lib.rs.restore.4242")
    with open(leftover, "w") as handle:
        handle.write("旧版驱动留下的临时件\n")
    cleaned = selftest.run("extras", "--item", "1", "--skip-stage-two")
    extra = os.path.join(work, "extras", "tree", "crates/singlefs-core/tests/new_case.rs")
    os.makedirs(os.path.dirname(extra))
    with open(extra, "w") as handle:
        handle.write("#[test]\nfn always_red() { panic!() }\n")
    selftest.clear_calls()
    refused = selftest.run("extras", "--item", "2", "--skip-stage-two")
    os.remove(extra)
    os.rmdir(os.path.dirname(extra))
    selftest.check(cleaned.returncode == 0 and not os.path.exists(leftover) and "已删掉" in cleaned.stdout
                   and refused.returncode == EXIT_DIRTY_TREE and not selftest.calls() and "new_case.rs" in refused.stderr,
                   f"核树没看多出来的文件（留着临时件那趟退 {cleaned.returncode}、临时件{'还在' if os.path.exists(leftover) else '删了'}；多出测试文件那趟退 {refused.returncode}、"
                   f"调了 {len(selftest.calls())} 步）：{refused.stderr}",
                   "看 verify_tree_is_pristine：树里比 manifest 多出来的文件，<登记的文件>.restore.<数字> 删掉，别的拒")
    # 恢复原件拷了临时件、还没换上就被杀：树里不多出文件，重跑同一条命令照常恢复、判完树回原样
    killed = selftest.run("extras", "--item", "3", "--skip-stage-two", BROAD_MUTATION_KILL_AT="restore-after-copy")
    stray = sorted(files_in_tree(os.path.join(work, "extras", "tree")) - set(SELFTEST_SOURCES))
    rerun = selftest.run("extras", "--item", "3", "--skip-stage-two")
    selftest.check(killed.returncode == -signal.SIGKILL and not stray and rerun.returncode == 0 and selftest.tree_is_pristine("extras"),
                   f"恢复原件半途被杀之后树里多出了文件、或重跑没恢复（被杀那趟退 {killed.returncode}；多出 {stray}；重跑退 {rerun.returncode}）：{rerun.stderr}",
                   "看 restore_original：临时件建在库目录（inflight.restore），不建在树里")
    # --sync-tree 换上新树、还没换版本指针就被杀：版本指针的临时件在，真跑拒（退 5）；重做 --sync-tree 之后指到新一版
    selftest.run("sync", "--sync-tree")
    with open(os.path.join(work, "sync", "tree.source")) as handle:
        old_fingerprint = json.load(handle)["fingerprint"]
    added = os.path.join(selftest.source, "crates/singlefs-harness/tests/new_case.rs")
    os.makedirs(os.path.dirname(added))
    with open(added, "w") as handle:
        handle.write("#[test]\nfn added_by_the_new_commit() {}\n")
    try:
        killed = selftest.run("sync", "--sync-tree", BROAD_MUTATION_KILL_AT="sync-tree-after-swap")
        pending_exists = os.path.exists(os.path.join(work, "sync", "tree.source.pending"))
        with open(os.path.join(work, "sync", "tree.source")) as handle:
            fingerprint_after_kill = json.load(handle)["fingerprint"]
        selftest.clear_calls()
        refused = selftest.run("sync", "--skip-stage-two")
        redone = selftest.run("sync", "--sync-tree")
        with open(os.path.join(work, "sync", "tree.source")) as handle:
            new_fingerprint = json.load(handle)["fingerprint"]
    finally:
        os.remove(added)
        os.rmdir(os.path.dirname(added))
        os.rmdir(os.path.dirname(os.path.dirname(added)))
    selftest.check(killed.returncode == -signal.SIGKILL and pending_exists and fingerprint_after_kill == old_fingerprint
                   and refused.returncode == EXIT_DIRTY_TREE and "--sync-tree 没做完" in refused.stderr and not selftest.calls()
                   and redone.returncode == 0 and new_fingerprint != old_fingerprint and not os.path.exists(os.path.join(work, "sync", "tree.source.pending")),
                   f"--sync-tree 换树半途被杀之后没拦住（被杀那趟退 {killed.returncode}、临时件{'在' if pending_exists else '不在'}；真跑退 {refused.returncode}、"
                   f"调了 {len(selftest.calls())} 步；重做退 {redone.returncode}；指纹 {old_fingerprint[:8]} → {fingerprint_after_kill[:8]} → {new_fingerprint[:8]}）：{refused.stderr}",
                   "看 sync_tree 的次序：先写 tree.source.pending、换树、再改名成 tree.source；Library.load_source 见到临时件，真跑拒")
    # 工具链进指纹：工具链换了真跑退 7；在新工具链下 --sync-tree 是另一版。加工具链之前建的版本目录（名字是源码指纹）早于 fresh-build，不再认
    source_fingerprint = fingerprint_of_manifest(manifest_of_tree(selftest.source))

    def recorded_global_digest(version_directory):
        """那一版 --sync-tree 时写下的全局键全文的 sha256（版本指纹里的那一行 global 照它算）。"""
        path = os.path.join(version_directory, "global-key.txt")
        if not os.path.isfile(path):
            return ""
        with open(path, "rb") as handle:
            return hashlib.sha256(handle.read()).hexdigest()

    def expected_version(toolchain, version_directory):
        return hashlib.sha256(f"{source_fingerprint}\n{toolchain}\njudge {JUDGE_RULES_VERSION}\nglobal {recorded_global_digest(version_directory)}\n".encode()).hexdigest()
    os.makedirs(os.path.join(work, "toolchain", "versions", source_fingerprint))
    first_toolchain, second_toolchain = "rustc 1.0.0 (selftest a)", "rustc 2.0.0 (selftest b)"
    selftest.run("toolchain", "--sync-tree", BROAD_MUTATION_TOOLCHAIN=first_toolchain)
    adopted = selftest.version_directory("toolchain")
    ran = selftest.run("toolchain", "--item", "1", "--skip-stage-two", BROAD_MUTATION_TOOLCHAIN=first_toolchain)
    selftest.clear_calls()
    changed = selftest.run("toolchain", "--item", "3", "--skip-stage-two", BROAD_MUTATION_TOOLCHAIN=second_toolchain)
    calls_after_change = len(selftest.calls())
    selftest.run("toolchain", "--sync-tree", BROAD_MUTATION_TOOLCHAIN=second_toolchain)
    second_version = selftest.version_directory("toolchain")
    summary = selftest.run("toolchain", "--summary", BROAD_MUTATION_TOOLCHAIN=second_toolchain)
    expected_first, expected_second = expected_version(first_toolchain, adopted), expected_version(second_toolchain, second_version)
    selftest.check(os.path.basename(adopted) == expected_first and ran.returncode == 0 and changed.returncode == EXIT_TOOLCHAIN and calls_after_change == 0
                   and os.path.basename(second_version) == expected_second and summary_field(summary.stdout, "not_run") == 8
                   and len(read_table(os.path.join(adopted, "results.tsv"), len(RESULT_COLUMNS)).rows) == 1,
                   f"工具链没进指纹（第一版 {os.path.basename(adopted)[:8]}、该是 {expected_first[:8]}、源码指纹 {source_fingerprint[:8]}；换工具链之后真跑退 {changed.returncode}、"
                   f"调了 {calls_after_change} 步；新一版 {os.path.basename(second_version)[:8]}、该是 {expected_second[:8]}）：{changed.stderr}",
                   "看 version_key（指纹 = 源码指纹、工具链、判法版本、全局键，名字是源码指纹的旧版目录不认）与 check_toolchain（真跑之前核，不同退 7）")
    # 全局键进指纹（E4）：判完一条之后排除表改了，真跑退 7、一步不调；在改后的排除表下重做 --sync-tree（同一份源码）是另一版，那一条在新一版里没有结论
    selftest.run("global", "--sync-tree")
    first_global = selftest.version_directory("global")
    judged = selftest.run("global", "--item", "1", "--skip-stage-two")
    with open(selftest.exclusions_path, encoding="utf-8") as handle:
        exclusions_before = handle.read()
    try:
        with open(selftest.exclusions_path, "a", encoding="utf-8") as handle:
            handle.write("singlefs-format::ignored_cases extra_case\t自证：排除表加了一行\n")
        selftest.clear_calls()
        stale = selftest.run("global", "--item", "2", "--skip-stage-two")
        calls_after_exclusions = len(selftest.calls())
        resynced = selftest.run("global", "--sync-tree")
        second_global = selftest.version_directory("global")
        summary = selftest.run("global", "--summary")
    finally:
        with open(selftest.exclusions_path, "w", encoding="utf-8") as handle:
            handle.write(exclusions_before)
    with open(os.path.join(work, "global", "tree.source"), encoding="utf-8") as handle:
        pointer_digest = json.load(handle).get("global_key")
    selftest.check(judged.returncode == 0 and stale.returncode == EXIT_TOOLCHAIN and calls_after_exclusions == 0 and "全局键" in stale.stderr and "exclusions" in stale.stderr
                   and resynced.returncode == 0 and second_global != first_global and pointer_digest == recorded_global_digest(second_global)
                   and summary_field(summary.stdout, "not_run") == 8,
                   f"排除表改了之后这一版的旧结论照算（判第 1 条退 {judged.returncode}；改排除表之后真跑退 {stale.returncode}、调了 {calls_after_exclusions} 步；"
                   f"重做换树退 {resynced.returncode}、版本 {os.path.basename(first_global)[:8]} → {os.path.basename(second_global)[:8]}；"
                   f"新一版没跑的 {summary_field(summary.stdout, 'not_run')} 条）：{stale.stderr[-400:]}",
                   "看 global_key_text、sync_tree（全局键全文写进 global-key.txt、sha256 进 tree.source 与 version_key）与 check_global_key（真跑之前核，不同退 7）")


def selftest_import_cells(selftest):
    """两台各跑一片之后合并导入：核源码指纹、清单、每行的指纹与变异名；本机已有的不重复导；两边定论不同的不导。"""
    work = selftest.work
    for home in ("peer", "local"):
        selftest.run(home, "--sync-tree")
    selftest.run("peer", "--shard", "1/2", "--skip-stage-two")
    selftest.run("local", "--shard", "0/2", "--skip-stage-two")
    peer_version = selftest.version_directory("peer")
    imported = selftest.run("local", "--import-results", peer_version)
    summary = selftest.run("local", "--summary")
    again = selftest.run("local", "--import-results", peer_version)
    selftest.clear_calls()
    nothing_left = selftest.run("local", "--skip-stage-two")
    selftest.check(imported.returncode == 0 and "imported=4" in imported.stdout and summary.returncode == EXIT_GATE_RED
                   and "caught=1 missed=0 unviable=1 timeout=1 memory_cap=1 stage_one_only=4 driver_error=0 not_run=0" in summary.stdout
                   and again.returncode == 0 and "imported=0 already_present=4" in again.stdout
                   and nothing_left.returncode == 0 and not selftest.called_sequences() and "selected=0" in nothing_left.stdout,
                   f"两台各一片的结果没合对（导入退 {imported.returncode}：{imported.stdout.splitlines()[:1]}；汇总 {summary.stdout.splitlines()[:1]}；"
                   f"再导一遍 {again.stdout.splitlines()[:1]}；之后续跑被调的序号 {sorted(selftest.called_sequences())}）：{imported.stderr}",
                   "看 import_results：核完才写、相同的行不重复导；导入之后这一版的状态照合起来的行算")
    # 该红的样本：另一台的行里变异名不是本机清单同序号的那一条（一行都不导）；另一台是别的源码；同一条两边定论不同
    rows_before = len(selftest.rows("local"))

    def peer_copy(name, edit):
        directory = os.path.join(work, name)
        shutil.copytree(peer_version, directory)
        with open(os.path.join(directory, "results.tsv")) as handle:
            lines = handle.read().split("\n")
        with open(os.path.join(directory, "results.tsv"), "w") as handle:
            handle.write("\n".join(edit(lines)))
        return directory

    def rename_first_row(lines):
        fields = lines[1].split("\t")
        fields[2] = "crates/singlefs-core/src/lib.rs:2:5: 不是这一条的变异名"
        return [lines[0], "\t".join(fields)] + lines[2:]

    wrong_name = selftest.run("local", "--import-results", peer_copy("peer-wrong-name", rename_first_row))
    rows_after_wrong_name = len(selftest.rows("local"))
    with open(os.path.join(selftest.source, "Cargo.toml"), "a") as handle:
        handle.write("# 另一个提交\n")
    try:
        selftest.run("foreign", "--sync-tree")
        selftest.run("foreign", "--item", "1", "--skip-stage-two")
    finally:
        with open(os.path.join(selftest.source, "Cargo.toml"), "w") as handle:
            handle.write(SELFTEST_SOURCES["Cargo.toml"])
    other_source = selftest.run("local", "--import-results", selftest.version_directory("foreign"))
    local_two = [row for row in selftest.rows("local") if row[0] == "2"][0]

    def add_conflicting_row(lines):
        return lines[:-1] + ["\t".join(local_two[:3] + ["caught", "one", "某条测试"] + local_two[6:])] + lines[-1:]

    conflict = selftest.run("local", "--import-results", peer_copy("peer-conflict", add_conflicting_row))
    selftest.check(wrong_name.returncode == EXIT_INCONSISTENT and rows_after_wrong_name == rows_before and other_source.returncode == EXIT_USAGE and "源码指纹" in other_source.stderr
                   and conflict.returncode == EXIT_INCONSISTENT and "IMPORT_CONFLICT\t2\t" in conflict.stdout and len(selftest.rows("local")) == rows_before
                   and selftest.states("local").get("2") == "unviable",
                   f"该拒的导入没拒（变异名不对的退 {wrong_name.returncode}、之后 {rows_after_wrong_name} 行对 {rows_before} 行；别的源码的退 {other_source.returncode}；"
                   f"定论不同的退 {conflict.returncode}、之后 {len(selftest.rows('local'))} 行）：{wrong_name.stderr}{conflict.stdout}",
                   "看 import_results：每行的指纹段、变异名对本机清单同序号的，全部核完才写；两边都有定论而不同的条不导、列出来、退 1")


def selftest_ignored_cells(selftest):
    """忽略用例登记：树里每条 #[ignore] 用例不是慢档用例（进第二段）就在排除表里；新加的没登记、排除表指着不存在的或写坏的，判红；第二段那一步的命令带排除的 -E、步名换了、旧基线行不认。"""
    work = selftest.work
    cases_file = os.path.join(selftest.source, "crates/singlefs-format/tests/ignored_cases.rs")
    # 绿：库「ignored」里 5 条忽略用例（2 条耗时、3 条排除），--check-ignored 判绿并打命令：-E 里有排除表的三条、没有慢档用例
    selftest.run("ignored", "--sync-tree")
    green = selftest.run("ignored", "--check-ignored")
    command_line = next((line for line in green.stdout.splitlines() if line.startswith("STEP_COMMAND\t")), "")
    selftest.check(green.returncode == 0 and "timed=2 excluded=3 unregistered=0 stale=0 both=0 unparsed=0" in green.stdout
                   and f"\t{SLOW_STEP_NAME}\t" in command_line and "--run-ignored only -E " in command_line
                   and all(term in command_line for term in ("binary_id(=singlefs-format) & test(=tests::in_lib_tool)",
                                                              "binary_id(=singlefs-format::ignored_cases) & test(=env_large_tier)",
                                                              "binary_id(=singlefs-format::ignored_cases) & test(=inner::env_tool)"))
                   and "slow_one" not in command_line and "in_lib_slow" not in command_line,
                   f"登记对的树上 --check-ignored 没判绿、或第二段那一步的命令不对（退 {green.returncode}）：{green.stdout}{green.stderr}",
                   "看 classify_ignored_cases（按缩进配模块、算二进制 id）、exclusion_filter 与 step_table：慢档用例进第二段，排除表的每一条进 -E")
    # 旧步名的基线行不认：库里先塞一行「two-test-ignored 绿」，两段都跑的一条要在没改过的树上把新步名的基线重量
    append_row(os.path.join(selftest.version_directory("ignored"), "baseline.tsv"), BASELINE_COLUMNS,
               [OLD_SLOW_STEP_NAME, "ok", "1.0", now_text(), default_threads(), ""])
    selftest.clear_calls()
    both_stages = selftest.run("ignored", "--item", "4")
    calls = selftest.calls()
    selftest.check(both_stages.returncode == 0 and ("0", SLOW_STEP_NAME, "pristine") in calls and not any(call[1] == OLD_SLOW_STEP_NAME for call in calls)
                   and [row[3] for row in selftest.rows("ignored") if row[0] == "4"] == [STAGE_ONE_ONLY, "missed"],
                   f"旧步名的基线行被认了（退 {both_stages.returncode}；调用 {calls}）：{both_stages.stderr}",
                   "看 SLOW_STEP_NAME 与 Runner.ensure_baseline：命令改了的一步换了步名，基线表里旧名的行就不认、在这一版上重量")
    # 红：新加两条没登记的（一条理由是大档、一条光秃秃的 #[ignore]）。两段都跑的趟在基线之前拒、一步不调；只跑第一段的趟照跑
    with open(cases_file) as handle:
        original_cases = handle.read()
    try:
        with open(cases_file, "a") as handle:
            handle.write('\n#[test]\n#[ignore = "大档：新加的、没登记"]\nfn new_env_tier() {}\n\n#[test]\n#[ignore]\nfn bare_ignore() {}\n')
        selftest.run("ignored-new", "--sync-tree")
    finally:
        with open(cases_file, "w") as handle:
            handle.write(original_cases)
    unregistered = selftest.run("ignored-new", "--check-ignored")
    selftest.clear_calls()
    refused = selftest.run("ignored-new", "--item", "4")
    calls_when_refused = selftest.calls()
    stage_one = selftest.run("ignored-new", "--item", "4", "--skip-stage-two")
    selftest.check(unregistered.returncode == EXIT_INCONSISTENT and "unregistered=2" in unregistered.stdout
                   and "既不是" in unregistered.stderr and refused.returncode == EXIT_INCONSISTENT and "既不是" in refused.stderr and not calls_when_refused
                   and stage_one.returncode == 0 and [row[3] for row in selftest.rows("ignored-new") if row[0] == "4"] == [STAGE_ONE_ONLY],
                   f"新加的没登记的忽略用例没判红（--check-ignored 退 {unregistered.returncode}；两段都跑退 {refused.returncode}、调了 {len(calls_when_refused)} 步；只跑第一段退 {stage_one.returncode}）："
                   f"{unregistered.stdout}{unregistered.stderr}{refused.stderr}",
                   "看 check_ignored_registration 与 run_items：要跑第二段的趟在基线之前核一遍，两样都不是的判红退 1；只跑第一段的趟不判")

    def with_rows(name, extra):
        path = os.path.join(work, name)
        with open(selftest.exclusions_path) as handle:
            text = handle.read()
        with open(path, "w") as handle:
            handle.write(text + extra)
        return selftest.run("ignored", "--check-ignored", BROAD_MUTATION_EXCLUSIONS=path)

    # 红：排除表指着树里没有的用例；排除表里的用例理由又写着慢档用例（两边都在）
    stale = with_rows("exclusions-stale.tsv", "singlefs-format::ignored_cases no_such_test\t自证：树里没有的用例\n")
    both = with_rows("exclusions-both.tsv", "singlefs-format::ignored_cases slow_one\t自证：理由写着慢档用例的又被排除\n")
    selftest.check(stale.returncode == EXIT_INCONSISTENT and "no_such_test" in stale.stderr and "stale=1" in stale.stdout
                   and both.returncode == EXIT_INCONSISTENT and "slow_one" in both.stderr and "both=1" in both.stdout,
                   f"排除表指着不存在的用例、或两边都在，没判红（不存在的退 {stale.returncode}：{stale.stderr}；两边都在退 {both.returncode}：{both.stderr}）",
                   "看 classify_ignored_cases 的 stale 与 both：排除表的每一行都要在树里找得到对应的 #[ignore] 用例，理由写着慢档用例的不许同时在排除表里")
    # 红：排除表写坏的行（没有制表符）、重复的行：判红退 1、点名哪一行
    garbled = with_rows("exclusions-garbled.tsv", "这一行没有制表符\n")
    duplicate = with_rows("exclusions-duplicate.tsv", "singlefs-format::ignored_cases env_large_tier\t自证：重复的一行\n")
    selftest.check(garbled.returncode == EXIT_INCONSISTENT and "两段" in garbled.stderr
                   and duplicate.returncode == EXIT_INCONSISTENT and "重复" in duplicate.stderr and "env_large_tier" in duplicate.stderr,
                   f"排除表写坏的行或重复的行没判红（写坏的退 {garbled.returncode}：{garbled.stderr}；重复的退 {duplicate.returncode}：{duplicate.stderr}）",
                   "看 load_exclusions：不是两段、头不是两个词、理由是空、重复的行都拒（退 1），不悄悄略过")


def selftest_test_variable_cells(selftest):
    """只供自证的变量：库目录在自证工作目录之外拒；隔了一层别的进程（父进程不是带 --selftest 的这份脚本）拒。"""
    outside = tempfile.mkdtemp(prefix="broad-mutation-selftest-outside.")
    try:
        elsewhere = subprocess.run([sys.executable, SELF_PATH, "--home", os.path.join(outside, "home"), "--sync-tree"], capture_output=True, text=True,
                                   stdin=subprocess.DEVNULL, env=selftest.environment)
        created = os.path.exists(os.path.join(outside, "home"))
    finally:
        shutil.rmtree(outside, ignore_errors=True)
    selftest.check(elsewhere.returncode == EXIT_USAGE and not created and "自证的工作目录" in elsewhere.stderr,
                   f"只供自证的变量指着自证工作目录之外的库时没拒（退 {elsewhere.returncode}，库目录{'建了' if created else '没建'}）：{elsewhere.stderr}",
                   "看 check_test_only_variables：设了只供自证的变量，库目录就得在 BROAD_MUTATION_SELFTEST_WORK 底下")
    # 库「first」的第 2 条一直没跑过：隔一层别的进程带着假一步的变量去跑它，要拒、一步不调、状态还是没跑
    selftest.clear_calls()
    relay = "import subprocess, sys; sys.exit(subprocess.run(sys.argv[1:]).returncode)"
    relayed = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2", "--skip-stage-two")], capture_output=True, text=True,
                             stdin=subprocess.DEVNULL, env=selftest.environment)
    selftest.check(relayed.returncode == EXIT_USAGE and not selftest.calls() and "不是 --selftest 自己起的子进程" in relayed.stderr
                   and selftest.states("first").get("2") == NOT_RUN,
                   f"父进程不是 --selftest 的进程带着假一步的变量时没拒（退 {relayed.returncode}，调了 {len(selftest.calls())} 步，第 2 条状态 {selftest.states('first').get('2')}）：{relayed.stderr}",
                   "看 test_mode 与 resolve_home：只供自证的变量与弄坏开关只在父进程是带 --selftest 的这份脚本时认，这一判要排在「库目录不在临时目录」之前")
    # 隔一层进程、只带弄坏开关（没有假一步的变量）：同样拒；库「first」的结果行数不变
    selftest.clear_calls()
    rows_before = len(selftest.rows("first"))
    break_only ={name: value for name, value in selftest.environment.items() if name not in TEST_ONLY_VARIABLES}
    break_only["BROAD_MUTATION_BREAK"] = ",".join(part for part in (os.environ.get("BROAD_MUTATION_BREAK", ""), "rerun-done") if part)
    relayed_break = subprocess.run([sys.executable, "-c", relay, *selftest.command("first", "--item", "2", "--skip-stage-two")], capture_output=True, text=True,
                                   stdin=subprocess.DEVNULL, env=break_only)
    selftest.check(relayed_break.returncode == EXIT_USAGE and not selftest.calls() and "不是 --selftest 自己起的子进程" in relayed_break.stderr
                   and len(selftest.rows("first")) == rows_before,
                   f"父进程不是 --selftest 的进程只带着弄坏开关时没拒（退 {relayed_break.returncode}，调了 {len(selftest.calls())} 步，结果行 {rows_before} → {len(selftest.rows('first'))}）：{relayed_break.stderr}",
                   "看 check_test_only_variables：TEST_ONLY_VARIABLES 里要有 BROAD_MUTATION_BREAK，非自证的进程带着它就拒")


def selftest_progress_cells(selftest):
    """一步跑过进度间隔要往 stderr 报进度：间隔换成 0.3 秒、第 1 条的测试那一步跑 1.5 秒，要有点名这一条这一步的行，其中有一行数到了日志里已过的那一条用例。"""
    selftest.write_plan([("1", "one-test", "sleep:1.5")])
    selftest.run("progress", "--sync-tree")
    ran = selftest.run("progress", "--item", "1", "--skip-stage-two", BROAD_MUTATION_PROGRESS_INTERVAL="0.3")
    selftest.write_plan(SELFTEST_PLAN)
    lines = [line for line in ran.stderr.splitlines() if line.startswith("BROAD_MUTATION_PROGRESS sequence=1 step=one-test ")]
    selftest.check(ran.returncode == 0 and lines and any(" tests_passed=1 tests_failed=0" in line for line in lines),
                   f"一步跑过进度间隔没有报进度（退 {ran.returncode}；点名第 1 条 one-test 的进度行 {lines[:3]}）：{ran.stderr[-600:]}",
                   "看 Runner.wait_reporting_progress 与 progress_line：一步跑着时每过 PROGRESS_INTERVAL_SECONDS 秒往 stderr 打一行，数日志里已过的用例")


def selftest_stop_cells(selftest):
    """被 TERM 停：在途那一步连同内存包装给它开的 scope 一起停（这一格的假一步经真的内存包装起，要用户级 systemd）。"""
    work = selftest.work
    selftest.slice_used = True
    wrapper_environment = dict(selftest.environment, BROAD_MUTATION_STEP_VIA_WRAPPER="1", RUN_WITH_MEMORY_CAP_SLICE=selftest.slice,
                               RUN_WITH_MEMORY_CAP_SLICE_TOTAL="256M", RUN_WITH_MEMORY_CAP_STATE_DIR=os.path.join(work, "admission"),
                               RUN_WITH_MEMORY_CAP_PEAKS=os.path.join(work, "memory-peaks.tsv"),
                               RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="broad-mutation 自证里的假一步，上限是写死的 64M")
    wrapper_environment.pop("CLAUDE_CODE_SESSION_ID", None)
    selftest.run("stop", "--sync-tree")
    selftest.write_plan([("1", "one-test", "hang")])
    pid_file = os.path.join(work, "hang.pids")
    if os.path.exists(pid_file):
        os.remove(pid_file)
    driver = subprocess.Popen(selftest.command("stop", "--item", "1", "--skip-stage-two", "--timeout-floor", "120"), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              text=True, stdin=subprocess.DEVNULL, env=wrapper_environment)
    hanging = wait_until(lambda: os.path.exists(os.path.join(work, "hanging")) or driver.poll() is not None, 90)
    wait_until(lambda: os.path.exists(pid_file) and len(open(pid_file).read().split()) >= 2, 5)
    driver.terminate()   # 只点自己起的这一个进程
    try:
        output, errors = driver.communicate(timeout=60)
    except subprocess.TimeoutExpired:
        driver.kill()    # 只点自己起的这一个进程
        output, errors = driver.communicate()
    step_processes = [int(word) for word in open(pid_file).read().split()] if os.path.exists(pid_file) else []
    all_stopped = wait_until(lambda: not any(process_is_running(process_id) for process_id in step_processes), 10)
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    rerun = subprocess.run(selftest.command("stop", "--item", "1", "--skip-stage-two", "--lock-wait", "0"), capture_output=True, text=True,
                           stdin=subprocess.DEVNULL, env=wrapper_environment)
    open(os.path.join(work, "release"), "w").close()
    rows_of_one = [row[3] for row in selftest.rows("stop") if row[0] == "1"] if os.path.isdir(os.path.join(work, "stop", "versions")) else []
    selftest.check(hanging and os.path.exists(os.path.join(work, "hanging")) and driver.returncode == EXIT_STOPPED and len(step_processes) >= 2 and all_stopped
                   and rerun.returncode == 0 and rows_of_one == ["caught"] and selftest.tree_is_pristine("stop"),
                   f"被 TERM 停时在途那一步没有一起停（那一步{'挂住了' if hanging else '没起来'}；驱动退 {driver.returncode}；那一步的进程 {step_processes}、"
                   f"{'都停了' if all_stopped else '还有活着的'}；紧接着重跑退 {rerun.returncode}、第 1 条各行 {rows_of_one}）：{errors}{rerun.stderr}",
                   "看 Runner.stop_step：给包装发 TERM 之外，还要停它开的那一个 scope（own_scope_unit 找、systemctl --user stop）。"
                   "驱动退 6、日志里是 251：这台机器起不了带上限的 scope（没有用户级 systemd），真跑同样起不来，先修那一头")


def selftest_selections(selftest):
    """假一步记的 [(序号, 步名, 跑了哪些, runner 模式, 整步时限)]。"""
    path = os.path.join(selftest.work, "selections.log")
    if not os.path.exists(path):
        return []
    with open(path) as handle:
        return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]


def selftest_prune_cells(selftest):
    """剪枝接进驱动：按覆盖挑测试（C2）、第二段按二进制（E6）、文档测试每次跑（E1）、测试步不设整步时限而经 runner 限 CPU（C4）、
    结论表与 --changed-since 沿用（C1、C3、E4）、里程碑全量列翻面（C5）、--summary 的提交门禁（C6）、第二段基线红的出路（E6）。"""
    work, core = selftest.work, "crates/singlefs-core/src/lib.rs"
    for name, content in (("coverage-one", {"tests": [{"binary": "fake-package::fake_target", "test": "covers_one", "covered": [[core, "one"]], "profiles": 1},
                                                       {"binary": "fake-package::fake_target", "test": "covers_two", "covered": [[core, "two"]], "profiles": 1}]}),
                          ("coverage-two", {"slow": [{"binary": "fake-package::slow", "covered": [[core, "one"]]}],
                                            "checker": [{"binary": "tier_one", "covered": [[core, "one"]]}, {"binary": "lib", "covered": []}]})):
        with open(os.path.join(work, name + ".json"), "w") as handle:
            json.dump(content, handle)
    selftest.write_plan([])
    selftest.run("prune", "--sync-tree")
    for name in ("selections.log",):
        if os.path.exists(os.path.join(work, name)):
            os.remove(os.path.join(work, name))
    ran = selftest.run("prune", "--item", "1", "--item", "3")
    for name in ("coverage-one", "coverage-two"):
        os.remove(os.path.join(work, name + ".json"))
    picked = {(row[0], row[1]): list(row[2:]) for row in selftest_selections(selftest)}
    rows = {row[0]: row for row in selftest.rows("prune")}
    expected = {("1", "one-test"): ["1 tests", "limit", "None"], ("1", SLOW_STEP_NAME): ["fake-package::slow", "limit", "None"],
                ("1", "two-test-checker-tier"): ["tier_one", "limit", "None"], ("1", "two-test-doc"): ["all", "", picked.get(("1", "two-test-doc"), ["", "", ""])[2]],
                ("3", "two-test-doc"): ["all", "", picked.get(("3", "two-test-doc"), ["", "", ""])[2]], ("0", "one-test"): ["all", "measure", "None"]}
    selftest.check(ran.returncode == 0 and all(picked.get(key) == value for key, value in expected.items())
                   and not any(key in picked for key in (("3", "one-test"), ("3", SLOW_STEP_NAME), ("3", "two-test-checker-tier")))
                   and ("3", "one-build") in picked and rows.get("3", [""] * 11)[3] == "missed" and "零覆盖" in rows.get("3", [""] * 11)[10],
                   f"按覆盖挑测试、第二段按二进制、文档测试每次跑、测试步经 runner 限 CPU 不对（退 {ran.returncode}；各步 {sorted(picked.items())}；第 3 条 {rows.get('3')}）：{ran.stderr[-400:]}",
                   "看 Pruner.selection（覆盖到被改函数的测试、第二段覆盖到它的二进制）、Runner.command_for（挑空的步不跑，文档测试一直跑）"
                   "与 Runner.run_step / time_limit_of（测试步 runner 模式 limit、基线 measure，不设整步时限）")
    # 结论表与 --changed-since：第一版判完（抓到 6 条、无效 1 条、没红 1 条），第二版只改 five 的函数体
    catching = "fail-nextest:singlefs-format::ignored_cases slow_one"     # 测试指纹认得出的一条（E3）：第二版没动它
    selftest.write_plan([(str(sequence), "one-test", catching) for sequence in (1, 3, 5, 6, 7, 8)] + [("2", "one-build", "fail-compile")])
    selftest.run("carry", "--sync-tree")
    first = selftest.run("carry")
    first_source = os.path.join(work, "first-source")
    shutil.copytree(selftest.source, first_source)
    second_sources = dict(SELFTEST_SOURCES)
    second_sources[core] = SELFTEST_SOURCES[core].replace("pub fn five() -> u32 {\n", "pub fn five() -> u32 {\n    let _unused = 0;\n")
    with open(os.path.join(selftest.source, core), "w") as handle:
        handle.write(second_sources[core])
    with open(os.path.join(work, "mutants.json"), "w") as handle:
        json.dump(selftest_mutants(second_sources), handle)
    try:
        selftest.run("carry", "--sync-tree")
        selftest.clear_calls()
        changed = selftest.run("carry", "--changed-since", "selftest", BROAD_MUTATION_CHANGED_SINCE_SOURCE=first_source)
        called = selftest.called_sequences() - {"0"}
        stages = {row[0]: row[4] for row in selftest.rows("carry")}
        summary = selftest.run("carry", "--summary")
        selftest.check(first.returncode == 0 and changed.returncode == 0 and called == {"2", "4", "5"}
                       and all(stages.get(str(sequence)) == CARRIED_STAGE for sequence in (1, 3, 6, 7, 8)) and "judge=3 carried=5" in changed.stdout
                       and summary.returncode == EXIT_GATE_RED and "missed=1 without_conclusion=0" in summary.stdout and "carried=5" in summary.stdout,
                       f"--changed-since 没有只现判改到的与沿用不了的（退 {changed.returncode}；现判的 {sorted(called)}；各条哪一段 {sorted(stages.items())}）："
                       f"{changed.stdout[-300:]}{changed.stderr[-300:]}；汇总 {[line for line in summary.stdout.splitlines() if 'GATE' in line]}",
                       "看 plan_changed_since 与 Pruner.carry：函数改了的、无效的、没红的现判，其余沿用结论表里同一个键的抓到并落 carried 行（C1、C3、E4）")
        # 里程碑全量：第 6 条这一回没红（「别处改动」让沿用的抓到翻了面）：要逐条列出来，汇总也报
        selftest.write_plan([(str(sequence), "one-test", catching) for sequence in (1, 3, 5, 7, 8)] + [("2", "one-build", "fail-compile")])
        milestone = selftest.run("carry", "--milestone-full")
        summary = selftest.run("carry", "--summary")
        selftest.check(milestone.returncode == 0 and "FLIPPED\t6\tcaught\tmissed\t" in milestone.stdout and "flipped=1" in summary.stdout
                       and summary.returncode == EXIT_GATE_RED,
                       f"里程碑全量没列出翻面的条（退 {milestone.returncode}：{milestone.stdout[-400:]}；汇总 {[line for line in summary.stdout.splitlines() if 'GATE' in line]}）",
                       "看 milestone_flips 与 report_flips：开跑时记下会沿用的结论（milestone-carry.tsv），全部现判之后逐条比，翻面的列 FLIPPED（第一轮 S1、S2、S4，"
                       "第二轮 FA、FA2、FB、FC、FD、FH，第三轮 Y6 这一类靠它兜底）")
    finally:
        shutil.rmtree(selftest.source)
        shutil.copytree(first_source, selftest.source)
        with open(os.path.join(work, "mutants.json"), "w") as handle:
            json.dump(selftest.mutants, handle)
        selftest.write_plan(SELFTEST_PLAN)
    # 全部有结论、没有没红：提交门禁绿
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest") for sequence in range(1, 9)])
    selftest.run("green", "--sync-tree")
    selftest.run("green", "--skip-stage-two")
    green = selftest.run("green", "--summary")
    selftest.check(green.returncode == 0 and "verdict=green" in green.stdout, f"每条都抓到的一版提交门禁没判绿（退 {green.returncode}）：{green.stdout[-300:]}",
                   "看 summarise 的 C6 那一段：抓到、没红、无效都算有结论，没红为零才绿")
    # 第二段基线红：报拒绝、点名红的测试、给出路
    selftest.write_plan([("0", "two-test-checker-tier", "fail-cargo")])
    selftest.run("stage-two-red", "--sync-tree")
    red = selftest.run("stage-two-red", "--item", "4")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(red.returncode == EXIT_BASELINE_RED and "tests::the_stage_two_case" in red.stderr and "--skip-stage-two" in red.stderr and "crash_identity" in red.stderr,
                   f"第二段基线红没报成带出路的拒绝（退 {red.returncode}）：{red.stderr[-400:]}",
                   "看 Runner.ensure_baseline 第二段那一支：列出红的测试，出路写已知的那条测试要谁改、修好之前 --skip-stage-two")
    selftest.clear_calls()
    os.remove(os.path.join(work, "selections.log"))


def selftest_environmental_cells(selftest):
    """--changed-since 重跑这一版里结局是超时、内存撞顶的条（提交门禁照旧算它们没结论，出路要走得通）：第 3 条超时、第 6 条撞顶，
    下一趟 --changed-since（源码没改）把这两条选中重跑、这一回抓到；没定论的别的条照选法不动。"""
    selftest.write_plan([("3", "one-test", "exit:253"), ("6", "one-test", "exit:250")])
    selftest.run("environmental", "--sync-tree")
    first = selftest.run("environmental", "--skip-stage-two")
    before = selftest.run("environmental", "--summary")
    selftest.write_plan([("3", "one-test", "fail-nextest"), ("6", "one-test", "fail-nextest")])
    selftest.clear_calls()
    rerun = selftest.run("environmental", "--changed-since", "selftest", "--skip-stage-two", BROAD_MUTATION_CHANGED_SINCE_SOURCE=selftest.source)
    called = selftest.called_sequences() - {"0"}
    states = selftest.states("environmental")
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    selftest.check(first.returncode == 0 and "NO_CONCLUSION\t3\ttimeout" in before.stdout and "NO_CONCLUSION\t6\tmemory-cap" in before.stdout
                   and "--changed-since" in before.stderr and rerun.returncode == 0 and called == {"3", "6"}
                   and states.get("3") == "caught" and states.get("6") == "caught",
                   f"--changed-since 没有重跑超时、内存撞顶的条（头一趟退 {first.returncode}；--changed-since 退 {rerun.returncode}、现判的 {sorted(called)}；"
                   f"之后第 3 条 {states.get('3')}、第 6 条 {states.get('6')}）：{rerun.stdout[-300:]}{rerun.stderr[-300:]}",
                   "看 select_items：给了 plans（--changed-since）时，要现判的条这一版里是 timeout / memory-cap 的也选上重跑（过过第一段的只补第二段）")


def selftest_limits(selftest):
    """假一步记的 [(序号, 步名, 哪一趟, 整步时限, 内存上限)]。"""
    path = os.path.join(selftest.work, "limits.log")
    if not os.path.exists(path):
        return []
    with open(path) as handle:
        return [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]


def selftest_global_key_class_cells(selftest):
    """全局键逐类进键（治理第二轮判决第 3、6 行）：--sync-tree 写的 global-key.txt 每一类行都在；之后逐类换工具链、工具版本、限时参数、
    构建环境（编译变量、RUSTC_WRAPPER、CARGO_BUILD_TARGET、CARGO_HOME 下的 cargo 配置、RUSTDOCFLAGS），真跑各要退 7、一步不调；什么都不换照跑。
    文件改了名、内容不变，--sync-tree 是另一版（源码指纹算路径）。"""
    work = selftest.work
    cargo_home = os.path.join(work, "cargo-home")
    os.makedirs(cargo_home, exist_ok=True)
    configuration = os.path.join(cargo_home, "config.toml")
    with open(configuration, "w") as handle:
        handle.write("[build]\njobs = 2\n")
    base = dict(selftest.environment, CARGO_HOME=cargo_home, RUSTFLAGS="-C selftest-flag", BROAD_MUTATION_TOOL_VERSIONS="selftest-tools a")
    synced = selftest.run("classes", "--sync-tree", environment=base)
    global_path = os.path.join(selftest.version_directory("classes"), "global-key.txt") if synced.returncode == 0 else ""
    text = open(global_path, encoding="utf-8").read() if global_path and os.path.isfile(global_path) else ""
    wanted = ["rules ", "toolchain ", "file Cargo.toml ", "build ", "tool ", "exclusions ", "limits ", "escalation "]
    missing = [prefix.strip() for prefix in wanted if not any(line.startswith(prefix) for line in text.splitlines())]
    missing += [name for name in ("<构建环境：CARGO_HOME 下的 config.toml>", "<构建环境：环境变量 RUSTFLAGS>") if name not in text]
    selftest.check(synced.returncode == 0 and not missing,
                   f"--sync-tree 写的 global-key.txt 缺了几类行（换树退 {synced.returncode}；缺 {missing}）：{text[:600]}{synced.stderr[-300:]}",
                   "看 global_key_lines：rules、toolchain、file（Cargo.toml / Cargo.lock / .cargo/）、build（build_environment_of）、tool、exclusions、limits、escalation 各一类")
    selftest.clear_calls()
    unchanged = selftest.run("classes", "--item", "1", "--skip-stage-two", environment=base)
    selftest.check(unchanged.returncode == 0 and "1" in selftest.called_sequences(),
                   f"--sync-tree 之后什么都没换，真跑却没照跑（退 {unchanged.returncode}）：{unchanged.stderr[-400:]}",
                   "看 global_key：同一份环境、同样的参数现算的全局键要与 --sync-tree 时记的逐字相同")
    wrapper_program = os.path.join(work, "step-driver.py")
    cases = [("工具链", dict(base, BROAD_MUTATION_TOOLCHAIN="rustc 9.9.9 (selftest other)"), [], None),
             ("工具版本", dict(base, BROAD_MUTATION_TOOL_VERSIONS="selftest-tools b"), [], None),
             ("限时参数", base, ["--cpu-multiplier", "4"], None),
             ("构建环境", dict(base, CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C overflow-checks=off"), [], None),
             ("构建环境", dict(base, RUSTC_WRAPPER=wrapper_program), [], None),
             ("构建环境", dict(base, CARGO_BUILD_TARGET="x86_64-unknown-linux-musl"), [], None),
             ("构建环境", dict(base, RUSTFLAGS="-C other-flag"), [], None),
             ("构建环境", dict(base, RUSTDOCFLAGS="--cfg selftest"), [], None),
             ("构建环境", base, [], "[build]\njobs = 2\n[profile.dev]\noverflow-checks = false\n")]
    for category, environment, extra, new_configuration in cases:
        if new_configuration is not None:
            with open(configuration, "w") as handle:
                handle.write(new_configuration)
        selftest.clear_calls()
        changed = selftest.run("classes", "--item", "2", "--skip-stage-two", *extra, environment=environment)
        calls = len(selftest.calls())
        if new_configuration is not None:
            with open(configuration, "w") as handle:
                handle.write("[build]\njobs = 2\n")
        detail = ([f"{name}={environment[name]}" for name in sorted(environment) if environment.get(name) != base.get(name)]
                  + extra + (["CARGO_HOME 下的 config.toml 改了 overflow-checks"] if new_configuration is not None else []))
        selftest.check(changed.returncode == EXIT_TOOLCHAIN and calls == 0,
                       f"--sync-tree 之后换了{category}，真跑没退 7（换的是 {detail}；退 {changed.returncode}、调了 {calls} 步）：{changed.stderr[-300:]}",
                       "看 check_toolchain 与 check_global_key：工具链、工具版本、限时参数、构建环境（build_environment_of）任一样与这一版记的不同都退 7")
    # 构建环境另七类（治理第三轮判决第 6 行）：research/scripts/admission.py 的 build_environment_lines 逐类写错时，各让下面至少一格翻红
    fake_compiler = os.path.join(work, "fake-rustc-v")
    with open(fake_compiler, "w") as handle:
        handle.write('#!/usr/bin/env bash\necho "rustc 0.0.0-selftest (自证的编译器)"\n')
    os.chmod(fake_compiler, 0o755)
    toolchain_file = os.path.join(work, "classes", "rust-toolchain.toml")
    simple_cases = [("环境变量 CARGO_PROFILE_*", dict(base, CARGO_PROFILE_DEV_OVERFLOW_CHECKS="false"), None),
                    ("环境变量 RUSTC（值与它的 -V）", dict(base, RUSTC=fake_compiler), None),
                    ("环境变量 CARGO_ENCODED_RUSTFLAGS", dict(base, CARGO_ENCODED_RUSTFLAGS="-Coverflow-checks=off"), None),
                    ("树的上层（库目录那一层）放了 rust-toolchain.toml", base, toolchain_file)]
    for what, environment, placed in simple_cases:
        if placed:
            with open(placed, "w") as handle:
                handle.write('[toolchain]\nchannel = "nightly"\n')
        selftest.clear_calls()
        try:
            changed = selftest.run("classes", "--item", "2", "--skip-stage-two", environment=environment)
        finally:
            if placed:
                os.remove(placed)
        calls = len(selftest.calls())
        selftest.check(changed.returncode == EXIT_TOOLCHAIN and calls == 0,
                       f"--sync-tree 之后换了构建环境的「{what}」，真跑没退 7（退 {changed.returncode}、调了 {calls} 步）：{changed.stderr[-300:]}",
                       "看 research/scripts/admission.py 的 build_environment_lines：CARGO_PROFILE_*、RUSTC 的值与 -V、CARGO_ENCODED_RUSTFLAGS、"
                       "从树往上每一层的 rust-toolchain(.toml) 都进构建环境")
    # 指的程序按内容进、往上层走、配置里指的程序：另一份库，换树时库目录那一层（树的上层）就有 .cargo/config.toml（build.rustc-wrapper 指一个程序），会话里 RUSTC_WRAPPER 指另一个程序
    programs_home = os.path.join(work, "classes-programs")
    os.makedirs(os.path.join(programs_home, ".cargo"), exist_ok=True)
    session_wrapper = os.path.join(work, "selftest-session-wrapper.sh")
    configured_wrapper = os.path.join(work, "selftest-configured-wrapper.sh")
    ancestor_configuration = os.path.join(programs_home, ".cargo", "config.toml")
    for path in (session_wrapper, configured_wrapper):
        with open(path, "w") as handle:
            handle.write('#!/bin/sh\nexec "$@"\n')
        os.chmod(path, 0o755)
    with open(ancestor_configuration, "w") as handle:
        handle.write(f'[build]\nrustc-wrapper = "{configured_wrapper}"\n')
    programs_base = dict(base, RUSTC_WRAPPER=session_wrapper)
    programs_synced = selftest.run("classes-programs", "--sync-tree", environment=programs_base)
    selftest.clear_calls()
    programs_unchanged = selftest.run("classes-programs", "--item", "2", "--skip-stage-two", environment=programs_base)
    selftest.check(programs_synced.returncode == 0 and programs_unchanged.returncode == 0,
                   f"库目录那一层有 cargo 配置、会话设了 RUSTC_WRAPPER 时，--sync-tree 之后什么都没换真跑却没照跑（换树退 {programs_synced.returncode}、"
                   f"真跑退 {programs_unchanged.returncode}）：{programs_synced.stderr[-300:]}{programs_unchanged.stderr[-300:]}",
                   "看 build_environment_lines 与 global_key：同一份环境现算的全局键要与 --sync-tree 时记的逐字相同")
    for what, path in (("RUSTC_WRAPPER 的值不变、它指的程序改了内容", session_wrapper),
                       ("树的上层（库目录那一层）的 .cargo/config.toml 里 build.rustc-wrapper 指的程序改了内容", configured_wrapper),
                       ("树的上层（库目录那一层）的 .cargo/config.toml 改了", ancestor_configuration)):
        with open(path) as handle:
            original = handle.read()
        with open(path, "a") as handle:
            handle.write("# 自证：内容改了\n")
        selftest.clear_calls()
        try:
            changed = selftest.run("classes-programs", "--item", "2", "--skip-stage-two", environment=programs_base)
        finally:
            with open(path, "w") as handle:
                handle.write(original)
        calls = len(selftest.calls())
        selftest.check(changed.returncode == EXIT_TOOLCHAIN and calls == 0,
                       f"--sync-tree 之后{what}，真跑没退 7（退 {changed.returncode}、调了 {calls} 步）：{changed.stderr[-300:]}",
                       "看 research/scripts/admission.py 的 build_environment_lines：指程序的变量与配置里指的程序按内容进、cargo 配置从树往上每一层都读")
    # 源码指纹算路径：同一份内容换个文件名是另一版
    first_path = os.path.join(selftest.source, "crates/singlefs-format/src/renamed_a.rs")
    second_path = os.path.join(selftest.source, "crates/singlefs-format/src/renamed_b.rs")
    try:
        with open(first_path, "w") as handle:
            handle.write("// 自证：改名不改内容\n")
        selftest.run("rename", "--sync-tree")
        before = selftest.version_directory("rename")
        os.replace(first_path, second_path)
        selftest.run("rename", "--sync-tree")
        after = selftest.version_directory("rename")
    finally:
        for path in (first_path, second_path):
            if os.path.exists(path):
                os.remove(path)
    selftest.check(before != after, f"文件改了名、内容不变，--sync-tree 还是同一版（{os.path.basename(before)[:16]}）",
                   "看 fingerprint_of_manifest：源码指纹的清单每行是「sha256  路径」，路径要进")


def selftest_rerun_larger_limit_cells(selftest):
    """证据不齐的超时、撞顶（治理第二轮判决第 4 行）：第 2 条编译那一步超时、升档那趟撞顶（两趟结局不同，证据不齐）→ 照撞顶记、没结论、登记表不写；
    加大 --timeout-floor 与 --memory-cap 再 --item 2 --rerun：不退 7（两样不进全局键），用的是加大的限时与上限，这一回判出结论。"""
    selftest.write_plan([("2", "one-build", "exit:253|exit:250"), ("2", "one-test", "fail-nextest")])
    selftest.run("larger", "--sync-tree")
    first = selftest.run("larger", "--item", "2", "--skip-stage-two")
    first_rows = [row for row in selftest.rows("larger") if row[0] == "2"]
    first_state = selftest.states("larger").get("2")
    registry = os.path.join(selftest.work, "larger", ESCALATION_REGISTRY)
    registry_rows_after_first = len(read_table(registry, len(ESCALATION_COLUMNS)).rows)
    selftest.write_plan([("2", "one-test", "fail-nextest")])
    if os.path.exists(os.path.join(selftest.work, "limits.log")):
        os.remove(os.path.join(selftest.work, "limits.log"))
    rerun = selftest.run("larger", "--item", "2", "--skip-stage-two", "--rerun", "--timeout-floor", "60", "--memory-cap", "32G")
    limits = [entry for entry in selftest_limits(selftest) if entry[:2] == ("2", "one-build")]
    selftest.write_plan(SELFTEST_PLAN)
    selftest.check(first.returncode == 0 and first_state == "memory-cap" and registry_rows_after_first == 0
                   and first_rows and "证据不齐" in first_rows[-1][RESULT_COLUMNS.index("note")]
                   and rerun.returncode == 0 and selftest.states("larger").get("2") == "caught" and limits and limits[-1][3:] == ("60", "32G"),
                   f"加大 --timeout-floor、--memory-cap 再 --item --rerun 没判出结论（头一趟退 {first.returncode}、状态 {first_state}、登记 {registry_rows_after_first} 行、"
                   f"备注 {first_rows[-1][RESULT_COLUMNS.index('note')] if first_rows else '没有行'}；重跑退 {rerun.returncode}、状态 {selftest.states('larger').get('2')}、"
                   f"编译那一步的时限与上限 {limits[-1][3:] if limits else '没调'}）：{rerun.stderr[-400:]}",
                   "看 escalate_step（两趟结局不同是证据不齐，照环境型结局记）与 limits_line_of（--timeout-floor、--memory-cap 不进全局键，加大了重跑不退 7）")


def selftest_escalation_cells(selftest):
    """文件头「升档」：超时、撞顶的那一步当场升档重跑一次；两趟都超时 / 撞顶、证据齐的判抓到并登记，升档那趟另有结局的照它往后判，证据不齐的没结论，
    --summary 逐条列升档判的抓到。"""
    work = selftest.work
    selftest.write_plan([("0", "one-build", "peak:52428800"), ("0", "one-test", "peak:104857600"),
                         ("1", "one-build", "exit:253|exit:253"), ("2", "one-test", "exit:250|exit:250"),
                         ("3", "one-build", "exit:253|ok"), ("3", "one-test", "fail-nextest"),
                         ("4", "one-build", "exit:253|exit:250"), ("5", "one-test", "exit:250|fail-nextest")])
    for log_name in ("limits.log", "wrapper-env.log"):
        if os.path.exists(os.path.join(work, log_name)):
            os.remove(os.path.join(work, log_name))
    selftest.run("escalation", "--sync-tree")
    ran = selftest.run("escalation", *[word for sequence in "12345" for word in ("--item", sequence)], "--skip-stage-two", "--memory-cap", "1G")
    summary = selftest.run("escalation", "--summary")
    selftest.write_plan(SELFTEST_PLAN)
    version = selftest.version_directory("escalation")
    fingerprint = os.path.basename(version)
    states = selftest.states("escalation")
    rows = selftest.rows("escalation")
    last = {}
    for row in rows:
        if row[3] in DECISIVE_OUTCOMES + ENVIRONMENTAL_OUTCOMES:
            last[row[0]] = row
    registry = {row[1]: dict(zip(ESCALATION_COLUMNS, row)) for row in read_table(os.path.join(work, "escalation", ESCALATION_REGISTRY), len(ESCALATION_COLUMNS)).rows}
    limits = selftest_limits(selftest)

    def evidence_holds(entry, first_limit, escalated_limit):
        """登记行的两份日志在、sha256 对、末尾有驱动那一行，限时或上限是该升到的那个。"""
        if not entry or entry["version"] != fingerprint or (entry["first_limit"], entry["escalated_limit"]) != (first_limit, escalated_limit):
            return False
        for attempt in ("first", "escalated"):
            path = os.path.join(version, entry[f"{attempt}_log"])
            if not os.path.isfile(path) or file_digest(path) != entry[f"{attempt}_log_sha256"]:
                return False
            with open(path, encoding="utf-8") as handle:
                if f"{ESCALATION_EVIDENCE_PREFIX} version={fingerprint} sequence={entry['sequence']} step={entry['step']} attempt={attempt}" not in handle.read():
                    return False
        return True
    timeout_entry, memory_entry = registry.get("1"), registry.get("2")
    selftest.check(ran.returncode == 0 and states.get("1") == "caught" and last.get("1", [""] * 6)[5] == ESCALATED_TIMEOUT_LABEL
                   and evidence_holds(timeout_entry, "5", "20") and ("1", "one-build", "escalated", "20", "1G") in limits,
                   f"两趟都超时、证据齐的没判成 caught-by-timeout（退 {ran.returncode}；状态 {states.get('1')}；末行 {last.get('1')}；登记 {timeout_entry}）：{ran.stderr[-400:]}",
                   "看 escalate_step 超时那一支：限时升到 max(4 倍原限时, 20 倍基线)，两趟都退 253 而基线、日志都在，写登记表、结果行记 caught 带标签")
    selftest.check(states.get("2") == "caught" and last.get("2", [""] * 6)[5] == ESCALATED_MEMORY_LABEL
                   and evidence_holds(memory_entry, "1G", "1536M") and memory_entry.get("baseline") == "104857600" and ("2", "one-test", "escalated", "None", "1536M") in limits,
                   f"两趟都撞顶、证据齐的没判成 caught-by-memory（状态 {states.get('2')}；末行 {last.get('2')}；登记 {memory_entry}）",
                   "看 escalate_step 撞顶那一支：上限升到 max(1.5 倍原上限, 2 倍基线峰值)，基线峰值取 baseline.tsv 里这个线程数下那一步最后一行绿记录的 peak_bytes")
    selftest.check(states.get("3") == "caught" and last.get("3", [""] * 6)[5] == SELFTEST_NEXTEST_CASE and "升档重跑" in last.get("3", [""] * 11)[10]
                   and states.get("5") == "caught" and last.get("5", [""] * 6)[5] == SELFTEST_NEXTEST_CASE and not {"3", "5"} & set(registry),
                   f"升档那趟跑完没照它往后的结局判（第 3 条 {last.get('3')}；第 5 条 {last.get('5')}；登记了 {sorted(registry)}）",
                   "看 process_item：升档那趟退 0 接着跑这一段后面的步，退测试没过照抓到判，都不写登记表")
    selftest.check(states.get("4") in ENVIRONMENTAL_OUTCOMES and "4" not in registry and "证据不齐" in last.get("4", [""] * 11)[10],
                   f"证据不齐的升档判成了抓到（第 4 条状态 {states.get('4')}、末行 {last.get('4')}；登记了 {sorted(registry)}）",
                   "看 escalation_evidence_problems：两趟结局不同、基线缺、限时或上限不到基线的 K 倍、日志里没有包装那一句，都是证据不齐，照环境型结局记")
    labelled = {sequence: row[5] for sequence, row in last.items() if row[3] == "caught" and row[5] in ESCALATED_LABELS}
    listed = {fields[1]: fields[2] for fields in (line.split("\t") for line in summary.stdout.splitlines()) if fields[0] == "ESCALATED" and len(fields) == 4}
    selftest.check(listed == labelled and summary_field(summary.stdout, "escalated") == len(labelled),
                   f"--summary 没列升档判的抓到（结果表里 {labelled}，--summary 列了 {listed}、判定行 escalated={summary_field(summary.stdout, 'escalated')}）",
                   "看 summarise：状态是升档判的抓到的条逐条打 ESCALATED，判定行带 escalated=<条数>")
    # 文件头「内存」：升档那趟带 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP（按自己的上限排队），改坏了源码的头一趟带 RUN_WITH_MEMORY_CAP_OOM_FIRST，基线两样都不带
    wrapper_environment = []
    if os.path.exists(os.path.join(work, "wrapper-env.log")):
        with open(os.path.join(work, "wrapper-env.log")) as handle:
            wrapper_environment = [tuple(line.rstrip("\n").split("\t")) for line in handle if line.strip()]
    escalated_entries = [entry for entry in wrapper_environment if entry[2:3] == ("escalated",)]
    first_entries = [entry for entry in wrapper_environment if entry[0] not in ("0", ENVIRONMENT_SEQUENCE) and entry[2:3] == ("first",)]
    baseline_entries = [entry for entry in wrapper_environment if entry[0] == "0"]
    selftest.check(escalated_entries and all(entry[3:] == ("queue-by-cap", "-") for entry in escalated_entries)
                   and first_entries and all(entry[3:] == ("-", "oom-first") for entry in first_entries)
                   and baseline_entries and all(entry[3:] == ("-", "-") for entry in baseline_entries),
                   f"升档那趟没按自己的上限排队、或改坏了源码的那几步没设撞总上限时先挨杀（升档那趟 {escalated_entries}；头一趟 {first_entries}；基线 {baseline_entries}）",
                   "看 Runner.run_step：attempt 是 escalated 的设 RUN_WITH_MEMORY_CAP_QUEUE_BY_CAP，别的变异步设 RUN_WITH_MEMORY_CAP_OOM_FIRST，基线与验环境那几步两样都不设")


def selftest_unit_environment_cells(selftest):
    """--unit-environment（治理第二轮判决第 5 行）：会话里设了全局键认的变量，--sync-tree 之后经单元起（只带 detached-run.sh 那五个变量）真跑退 7；
    照 --unit-environment 打的那一行带进去就照跑；不认的变量不打。"""
    session = dict(selftest.environment, RUSTFLAGS="-C session flag", CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C overflow-checks=off",
                   RUSTDOCFLAGS="--cfg session", SELFTEST_UNRELATED_VARIABLE="1")
    selftest.run("unitenv", "--sync-tree", environment=session)
    printed = selftest.run("unitenv", "--unit-environment", environment=session)
    words = shlex.split(printed.stdout.strip()) if printed.returncode == 0 else []
    carried = dict(word.split("=", 1) for word in words[1:] if "=" in word) if words[:1] == ["env"] else {}
    selftest.clear_calls()
    without = selftest.run("unitenv", "--item", "1", "--skip-stage-two")
    selftest.clear_calls()
    unit = selftest.run("unitenv", "--item", "1", "--skip-stage-two", environment=dict(selftest.environment, **carried))
    selftest.check(printed.returncode == 0 and {"RUSTFLAGS", "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS", "RUSTDOCFLAGS"} <= set(carried)
                   and "SELFTEST_UNRELATED_VARIABLE" not in carried and not set(UNIT_CARRIED_VARIABLES) & set(carried)
                   and without.returncode == EXIT_TOOLCHAIN and unit.returncode == 0,
                   f"--unit-environment 打的不是全局键认的那几个变量（打了「{printed.stdout.strip()}」；不带它退 {without.returncode}、带着退 {unit.returncode}）："
                   f"{printed.stderr[-300:]}{unit.stderr[-300:]}",
                   "看 unit_environment_names：拿掉一个变量构建环境（build_environment_of）就变的才打，detached-run.sh 本来带的五个不打")
    # 工具链也比（治理第三轮判决第 8 行）：会话里设了 RUSTUP_TOOLCHAIN，rustc -Vv 就换了一个；不带进单元真跑退 7，--unit-environment 要把它打出来
    fake_rustc = os.path.join(selftest.work, "fake-rustc-vv")
    with open(fake_rustc, "w") as handle:
        handle.write('#!/usr/bin/env bash\necho "rustc 1.0.0-selftest (${RUSTUP_TOOLCHAIN:-default})"\necho "host: x86_64-unknown-linux-gnu"\n')
    os.chmod(fake_rustc, 0o755)
    with_rustc = dict(selftest.environment, BROAD_MUTATION_RUSTC=fake_rustc)
    toolchain_session = dict(with_rustc, RUSTUP_TOOLCHAIN="nightly-selftest")
    selftest.run("unittool", "--sync-tree", environment=toolchain_session)
    printed = selftest.run("unittool", "--unit-environment", environment=toolchain_session)
    words = shlex.split(printed.stdout.strip()) if printed.returncode == 0 else []
    carried = dict(word.split("=", 1) for word in words[1:] if "=" in word) if words[:1] == ["env"] else {}
    selftest.clear_calls()
    without = selftest.run("unittool", "--item", "1", "--skip-stage-two", environment=with_rustc)
    selftest.clear_calls()
    unit = selftest.run("unittool", "--item", "1", "--skip-stage-two", environment=dict(with_rustc, **carried))
    selftest.check(printed.returncode == 0 and carried.get("RUSTUP_TOOLCHAIN") == "nightly-selftest" and not set(TEST_ONLY_VARIABLES) & set(carried)
                   and without.returncode == EXIT_TOOLCHAIN and unit.returncode == 0,
                   f"--unit-environment 没把换了工具链的 RUSTUP_TOOLCHAIN 打出来（打了「{printed.stdout.strip()}」；不带它退 {without.returncode}、带着退 {unit.returncode}）："
                   f"{printed.stderr[-300:]}{unit.stderr[-300:]}",
                   "看 unit_environment_names：拿掉一个变量 rustc -Vv（toolchain_text）就变的也打")


def selftest_escalation_evidence_cells(selftest):
    """文件头「升档」②③ 的五支（escalation_evidence_problems）各一格：只坏那一样的证据，要报那一样。对照是证据齐的一份，要报空表。"""
    good = dict(first_exit=TIME_LIMIT_HIT_EXIT, escalated_exit=TIME_LIMIT_HIT_EXIT, baseline=10.0, first_amount=900, escalated_amount=3600,
                first_text="900", escalated_text="3600", first_log_text="run-with-memory-cap: 超过限时 900 秒（自证）\n",
                escalated_log_text="run-with-memory-cap: 超过限时 3600 秒（自证）\n")
    control = escalation_evidence_problems(**good)
    selftest.check(control == [], f"证据齐的一份（两趟都 253、3600 对基线 10 秒、两份日志都带包装那一句）报了证据不齐：{control}",
                   "看 escalation_evidence_problems：五样都齐时交空表")
    cases = [("两趟结局不同", dict(good, escalated_exit=MEMORY_CAP_HIT_EXIT, escalated_log_text="run-with-memory-cap: 撞了内存上限 3600（自证）\n"), "两趟结局不同"),
             ("不到 K 倍", dict(good, baseline=200.0), "不到基线"),
             ("没比原来大", dict(good, escalated_amount=900, escalated_text="900", escalated_log_text="run-with-memory-cap: 超过限时 900 秒（自证）\n"), "没比原来的"),
             ("原来那趟缺包装那一句", dict(good, first_log_text="   Compiling singlefs-core（自证）\n"), "原来那趟的日志里没有包装打的"),
             ("升档那趟缺包装那一句", dict(good, escalated_log_text="   Compiling singlefs-core（自证）\n"), "升档那趟的日志里没有包装打的")]
    for what, given, wanted in cases:
        found = escalation_evidence_problems(**given)
        selftest.check(any(wanted in problem for problem in found),
                       f"升档的证据{what}没报出来（要有一条带「{wanted}」，实际 {found}）",
                       f"看 escalation_evidence_problems 里管「{what}」的那一支：只坏这一样也要报证据不齐")


def selftest_baseline_threads_cells(selftest):
    """基线按线程数分开记（治理第三轮判决第 1 行）：--threads 2 下升档判的抓到登记了基线那一行的行号；换 --threads 3 之后基线在新线程数下重量、追加新行，
    旧登记行指的那一行照旧是它自己线程数、它记的那个值（门禁按行号复核就对得上），新登记行指新线程数那一行。"""
    work = selftest.work
    selftest.write_plan([("0", "one-test", "peak:104857600"), ("1", "one-build", "exit:253|exit:253"), ("2", "one-test", "exit:250|exit:250")])
    selftest.run("threads", "--sync-tree")
    first = selftest.run("threads", "--item", "1", "--item", "2", "--skip-stage-two", "--threads", "2", "--memory-cap", "1G")
    selftest.write_plan([("0", "one-build", "sleep:0.3"), ("0", "one-test", "peak:209715200"), ("3", "one-build", "exit:253|exit:253")])
    second = selftest.run("threads", "--item", "3", "--skip-stage-two", "--threads", "3", "--memory-cap", "1G")
    selftest.write_plan(SELFTEST_PLAN)
    version = selftest.version_directory("threads")
    baseline = dict(read_baseline_rows(os.path.join(version, "baseline.tsv")))
    registry = [dict(zip(ESCALATION_COLUMNS, row)) for row in read_table(os.path.join(work, "threads", ESCALATION_REGISTRY), len(ESCALATION_COLUMNS)).rows]
    problems = []
    for entry in registry:
        row = baseline.get(int(entry["baseline_row"])) if entry["baseline_row"].isdigit() else None
        value_column = "seconds" if entry["label"] == ESCALATED_TIMEOUT_LABEL else "peak_bytes"
        if row is None or (row["step"], row["outcome"], row["threads"], row[value_column]) != (entry["step"], "ok", entry["threads"], entry["baseline"]):
            problems.append(f"第 {entry['sequence']} 条登记的基线 {entry['baseline']}（线程 {entry['threads']}）与 baseline.tsv 第 {entry['baseline_row']} 行 {row} 对不上")
    threads_seen = {(row["step"], row["threads"]) for row in baseline.values() if row["outcome"] == "ok"}
    by_sequence = {entry["sequence"]: entry for entry in registry}
    selftest.check(first.returncode == 0 and second.returncode == 0 and not problems and {"1", "2", "3"} <= set(by_sequence)
                   and {("one-build", "2"), ("one-test", "2"), ("one-build", "3"), ("one-test", "3")} <= threads_seen
                   and by_sequence["3"]["threads"] == "3" and by_sequence["3"]["baseline_row"] != by_sequence["1"]["baseline_row"],
                   f"换了 --threads 之后基线没在新线程数下另记、或登记行指的基线行对不上（头一趟退 {first.returncode}、换线程那一趟退 {second.returncode}；"
                   f"基线表里的（步名, 线程数）{sorted(threads_seen)}；登记 {[(entry['sequence'], entry['threads'], entry['baseline_row'], entry['baseline']) for entry in registry]}；"
                   f"{'；'.join(problems)}）：{second.stderr[-300:]}",
                   "看 Runner.baseline_rows_of、ensure_baseline 与 escalate_step：基线按（步名, 线程数）取最后一行，没有就在这个线程数下重量、追加一行（秒数与峰值），"
                   "登记行记它的行号（baseline_row）")


def selftest_result_mac_cells(selftest):
    """结果行的校验列（文件头「结果行的校验列」）：驱动写的每一行都带对得上的校验列、密钥在库目录之外且权限 600；手追加的没有校验列的 caught 行、
    校验列对不上的 caught 行都不算，--summary 判红（退 1）、点到那几行；密钥不在时 --summary 判红、不替它建。"""
    work = selftest.work
    selftest.run("mac", "--sync-tree")
    ran = selftest.run("mac", "--item", "1", "--item", "2", "--skip-stage-two")
    key_path = selftest.result_key_path()
    key, key_problem = read_result_key(key_path)
    rows = selftest.rows("mac")
    bad = [row[0] for row in rows if key is None
           or row[-1] != hmac.new(key, "\t".join(row[:RESULT_SIGNED_COLUMNS]).encode(), hashlib.sha256).hexdigest()]
    mode = os.stat(key_path).st_mode & 0o777 if os.path.exists(key_path) else None
    selftest.check(ran.returncode == 0 and rows and not bad and mode == 0o600 and not path_is_under(key_path, os.path.join(work, "mac")),
                   f"驱动写的结果行没带对得上的校验列、或密钥的位置与权限不对（退 {ran.returncode}；{len(rows)} 行里校验对不上 {bad}；密钥 {key_path} 权限 "
                   f"{'不在' if mode is None else oct(mode)}，{key_problem}）：{ran.stderr[-300:]}",
                   "看 Library.record_result 与 result_key：每一行照 signed_result_row 带 HMAC-SHA256 校验列，密钥没有就在库目录之外建一份、权限 600")
    results_path = selftest.results_path("mac")
    fingerprint = os.path.basename(os.path.dirname(results_path))
    names = {fields[0]: fields[2] for fields in (line.split("\t") for line in selftest.run("mac", "--list-items").stdout.splitlines()) if len(fields) == 4}
    unsigned = ["4", "singlefs-core", names.get("4", "?"), "caught", "one", "某条测试", "0.0", "0.0", now_text(), fingerprint, ""]
    wrong_mac = ["5", "singlefs-core", names.get("5", "?"), "caught", "one", "某条测试", "0.0", "0.0", now_text(), fingerprint, "", "0" * 64]
    with open(results_path, "a") as handle:
        handle.write("\t".join(unsigned) + "\n" + "\t".join(wrong_mac) + "\n")
    summary = selftest.run("mac", "--summary")
    states = selftest.states("mac")
    selftest.check(summary.returncode == EXIT_INCONSISTENT and "校验列" in summary.stderr and states.get("4") != "caught" and states.get("5") != "caught",
                   f"手追加的没有校验列、校验列对不上的 caught 行被当成了结论（--summary 退 {summary.returncode}；第 4 条 {states.get('4')}、第 5 条 {states.get('5')}）："
                   f"{summary.stderr[-300:]}",
                   "看 read_result_table：十一段的行与校验列对不上的行都不算、列进 forged，summarise 把它们报成对不上、退 1")
    missing_key = selftest.run("mac", "--summary", BROAD_MUTATION_RESULT_KEY=os.path.join(work, "no-such-key", "result.key"))
    selftest.check(missing_key.returncode == EXIT_INCONSISTENT and "不在" in missing_key.stderr and not os.path.exists(os.path.join(work, "no-such-key")),
                   f"密钥不在时 --summary 没判红、或替它建了一份（退 {missing_key.returncode}）：{missing_key.stderr[-300:]}",
                   "看 Library.result_key：只读的用法不建密钥，没有密钥时每一行都校验不了、照不算报")


FRESH_BUILD_COMMIT_TIME_NS = 1_790_000_000 * 10 ** 9    # 自证里的「提交时刻」：早于自证里编出的每一份产物


def selftest_fresh_build_cells(selftest):
    """换树之后编译目录不留过时的工作区产物（导出带 -m、清工作区各包的 fingerprint）、基线之前核旧产物、判法版本进版本指纹与全局键（旧结论一条都不沿用）。"""
    work = selftest.work
    core_extra = "crates/singlefs-core/src/extra.rs"

    def commit_source(name, version):
        """一个提交的源码：两个被变异的包有 Cargo.toml（工作区的包名从这里认），core 里多一个随提交变的文件；每个文件的 mtime 都是提交时刻。"""
        directory = os.path.join(work, name)
        shutil.copytree(selftest.source, directory)
        for package in MUTATED_PACKAGES:
            with open(os.path.join(directory, "crates", package, "Cargo.toml"), "w") as handle:
                handle.write(f'[package]\nname = "{package}"\nversion = "0.1.0"\n')
        with open(os.path.join(directory, core_extra), "w") as handle:
            handle.write(f"pub const VERSION: u32 = {version};\n")
        for found_directory, _directory_names, file_names in os.walk(directory):
            for file_name in file_names:
                os.utime(os.path.join(found_directory, file_name), ns=(FRESH_BUILD_COMMIT_TIME_NS, FRESH_BUILD_COMMIT_TIME_NS))
        return directory

    def compiled():
        path = os.path.join(work, "cargo.log")
        if not os.path.exists(path):
            return set()
        with open(path) as handle:
            return {(fields[0], fields[2]) for fields in (line.rstrip("\n").split("\t") for line in handle) if len(fields) == 3}

    # 换树之后必重编：提交 A 导出、基线编过（产物新于提交时刻）；换到提交 B（源文件的提交时刻早于那些产物），基线必须重编两个包、测试跑的是 B 编出来的
    selftest.write_plan([("0", "one-build", "cargo-build"), ("0", "one-test", "cargo-test")])
    source_a, source_b = commit_source("fresh-commit-a", 1), commit_source("fresh-commit-b", 2)
    selftest.run("fresh", "--sync-tree", BROAD_MUTATION_TREE_SOURCE=source_a)
    first = selftest.run("fresh", "--item", "1", "--skip-stage-two")
    target = os.path.join(work, "fresh", "target")
    kept = [os.path.join(target, "debug", ".fingerprint", "serde-0123456789abcdef"), os.path.join(target, "debug", ".fingerprint", "singlefs-core-extra-0123456789abcdef")]
    cleared = [os.path.join(target, "release", ".fingerprint", "singlefs-checker-fedcba9876543210"),
               os.path.join(target, "x86_64-unknown-linux-gnu", "debug", ".fingerprint", "singlefs-core-00000000000000aa"),
               os.path.join(work, "fresh", "target-coverage", "debug", ".fingerprint", "singlefs-core-1111111111111111")]
    for path in kept + cleared:
        os.makedirs(path, exist_ok=True)
        with open(os.path.join(path, "lib-planted"), "w") as handle:
            handle.write("换树之前编的\n")
    cleared.append(os.path.join(target, "debug", ".fingerprint", "singlefs-core-0123456789abcdef"))     # 假 cargo 编 A 时留下的
    built_before = os.path.isdir(cleared[-1])
    if os.path.exists(os.path.join(work, "cargo.log")):
        os.remove(os.path.join(work, "cargo.log"))
    before_sync = time.time_ns()
    synced = selftest.run("fresh", "--sync-tree", BROAD_MUTATION_TREE_SOURCE=source_b)
    tree = os.path.join(work, "fresh", "tree")
    older = sorted(path for path in files_in_tree(tree) if os.lstat(os.path.join(tree, path)).st_mtime_ns < before_sync - SYNC_CLOCK_SLACK_NS)
    left = [os.path.relpath(path, work) for path in cleared if os.path.exists(path)]
    kept_all = all(os.path.isdir(path) for path in kept)
    second = selftest.run("fresh", "--item", "1", "--skip-stage-two")
    rebuilt = compiled()
    selftest.check(first.returncode == 0 and built_before and synced.returncode == 0 and not older and not left and kept_all
                   and second.returncode == 0 and {("0", package) for package in MUTATED_PACKAGES} <= rebuilt,
                   f"换树之后编译目录里留着过时的工作区产物（换到 B 之前那趟退 {first.returncode}、编过 {built_before}；换树退 {synced.returncode}；"
                   f"树里 mtime 早于换树的 {older[:3]}；该清没清的 fingerprint {left}；第三方的{'都在' if kept_all else '被清了'}；"
                   f"换树后那趟退 {second.returncode}、重编的 {sorted(rebuilt)}）：{synced.stderr[-300:]}{second.stderr[-400:]}",
                   "看 sync_tree：tar -x 要带 -m（mtime 取解开那一刻），换树之后 clear_workspace_fingerprints 清掉 target/ 与 target-coverage/ 里"
                   "工作区各包（树里 Cargo.toml 的 [package] name）的 fingerprint，第三方依赖的不动")
    # 基线之前核旧产物：编译目录里有换树之前编的工作区 fingerprint、树里有文件的 mtime 是提交时刻、版本指针里没有 synced_at_ns，各拒一次（退 5、一步不调）；还原之后照跑
    samples = []
    stale_entry = os.path.join(target, "debug", ".fingerprint", "singlefs-checker-aaaaaaaaaaaaaaaa")
    os.makedirs(stale_entry)
    with open(os.path.join(stale_entry, "lib-singlefs_checker"), "w") as handle:
        handle.write("换树之前编的\n")
    for path in (os.path.join(stale_entry, "lib-singlefs_checker"), stale_entry):
        os.utime(path, ns=(FRESH_BUILD_COMMIT_TIME_NS, FRESH_BUILD_COMMIT_TIME_NS))
    selftest.clear_calls()
    samples.append(("fingerprint", "singlefs-checker-aaaaaaaaaaaaaaaa", selftest.run("fresh", "--item", "2", "--skip-stage-two"), selftest.calls()))
    shutil.rmtree(stale_entry)
    extra = os.path.join(tree, core_extra)
    os.utime(extra, ns=(FRESH_BUILD_COMMIT_TIME_NS, FRESH_BUILD_COMMIT_TIME_NS))
    selftest.clear_calls()
    samples.append(("mtime", "extra.rs", selftest.run("fresh", "--item", "2", "--skip-stage-two"), selftest.calls()))
    os.utime(extra)
    pointer_path = os.path.join(work, "fresh", "tree.source")
    with open(pointer_path, "rb") as handle:
        pointer = handle.read()
    with open(pointer_path, "w") as handle:
        json.dump({key: value for key, value in json.loads(pointer).items() if key != "synced_at_ns"}, handle)
    selftest.clear_calls()
    samples.append(("pointer", "synced_at_ns", selftest.run("fresh", "--item", "2", "--skip-stage-two"), selftest.calls()))
    with open(pointer_path, "wb") as handle:
        handle.write(pointer)
    selftest.clear_calls()
    clean = selftest.run("fresh", "--item", "2", "--skip-stage-two")
    wrong = [f"{name}：退 {ran.returncode}、调了 {len(calls)} 步、报错里{'有' if token in ran.stderr else '没有'}「{token}」"
             for name, token, ran, calls in samples if not (ran.returncode == EXIT_DIRTY_TREE and token in ran.stderr and "--sync-tree" in ran.stderr and not calls)]
    selftest.check(not wrong and clean.returncode == 0 and "2" in selftest.called_sequences(),
                   f"基线之前没拦住编译目录里的旧产物（{'；'.join(wrong) or '三个样本都拒了'}；还原之后那趟退 {clean.returncode}、被调的序号 {sorted(selftest.called_sequences())}）："
                   f"{samples[0][2].stderr[-300:]}{clean.stderr[-300:]}",
                   "看 run_items 里基线之前那一核（stale_build_problems）：树里文件的 mtime、工作区各包的 fingerprint 都不许早于版本指针的 synced_at_ns，"
                   "没有 synced_at_ns 的树同样拒；拒时退 5，出路写重做 --sync-tree")
    # 判法版本进版本指纹与全局键：旧驱动（判法 prune-r3-e1-e6）在名字是源码指纹的旧版目录里判完 8 条、结论表里记着旧全局键；
    # 换上这一版驱动重做 --sync-tree（同一份源码）要是另一版，--changed-since（源码一字没改）8 条都以全局键变了现判，一条不沿用
    selftest.write_plan([(str(sequence), "one-test", "fail-nextest:singlefs-format::ignored_cases slow_one") for sequence in range(1, 9)])
    source_fingerprint = fingerprint_of_manifest(manifest_of_tree(selftest.source))
    os.makedirs(os.path.join(work, "judge", "versions", source_fingerprint))
    selftest.run("judge", "--sync-tree", BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE="1")
    old_version = selftest.version_directory("judge")
    old_run = selftest.run("judge", "--skip-stage-two", BROAD_MUTATION_PRE_FRESH_BUILD_JUDGE="1")
    old_outcomes = [row[3] for row in selftest.rows("judge")]
    old_keys = {row[CONCLUSION_COLUMNS.index("global_key")] for row in read_table(os.path.join(work, "judge", "conclusions.tsv"), len(CONCLUSION_COLUMNS)).rows}
    synced = selftest.run("judge", "--sync-tree")
    new_version = selftest.version_directory("judge")
    selftest.clear_calls()
    rejudged = selftest.run("judge", "--changed-since", "selftest", "--skip-stage-two", BROAD_MUTATION_CHANGED_SINCE_SOURCE=selftest.source)
    called = selftest.called_sequences() - {"0"}
    new_key_path = os.path.join(new_version, "global-key.txt")
    new_key_text = open(new_key_path).read() if os.path.isfile(new_key_path) else ""
    old_rows_after = len(read_table(os.path.join(old_version, "results.tsv"), len(RESULT_COLUMNS)).rows)
    selftest.write_plan(SELFTEST_PLAN)
    selftest.clear_calls()
    selftest.check(os.path.basename(old_version) == source_fingerprint and old_run.returncode == 0 and old_outcomes == ["caught"] * 8 and old_keys
                   and synced.returncode == 0 and new_version != old_version and rejudged.returncode == 0 and "judge=8 carried=0" in rejudged.stdout
                   and "global-key-changed=8" in rejudged.stdout and called == {str(sequence) for sequence in range(1, 9)}
                   and new_key_text.startswith(f"rules {JUDGE_RULES_VERSION}\n") and hashlib.sha256(new_key_text.encode()).hexdigest() not in old_keys and old_rows_after == 8,
                   f"判法版本改了旧结论还被沿用（旧驱动那一版 {os.path.basename(old_version)[:8]}、源码指纹 {source_fingerprint[:8]}、结局 {old_outcomes}；"
                   f"重做换树后 {os.path.basename(new_version)[:8]}；--changed-since 退 {rejudged.returncode}、现判的 {sorted(called)}；"
                   f"新全局键首行 {new_key_text.splitlines()[:1]}）：{[line for line in rejudged.stdout.splitlines() if 'CHANGED' in line]}{rejudged.stderr[-300:]}",
                   "看 JUDGE_RULES_VERSION、version_key（判法版本进这一版的指纹，名字是源码指纹的旧版目录不认）与 global_key（首行 rules <判法版本>）：判法改了，"
                   "同一个提交重做 --sync-tree 是另一版，结论表里记着旧全局键的一条都不沿用")


def run_selftest_cells(selftest):
    selftest_fresh_build_cells(selftest)   # 放最前：弄坏换树那几项时后面各组的库都换不好树，跑在后面会被前面的异常截掉
    selftest_prune_cells(selftest)
    selftest_environmental_cells(selftest)
    listed_lines = selftest_basic_cells(selftest)
    selftest_state_cells(selftest)
    selftest_table_cells(selftest, listed_lines)
    selftest_environment_cells(selftest)
    selftest_tree_cells(selftest)
    selftest_global_key_class_cells(selftest)
    selftest_rerun_larger_limit_cells(selftest)
    selftest_escalation_cells(selftest)
    selftest_escalation_evidence_cells(selftest)
    selftest_baseline_threads_cells(selftest)
    selftest_result_mac_cells(selftest)
    selftest_unit_environment_cells(selftest)
    selftest_import_cells(selftest)
    selftest_ignored_cells(selftest)
    selftest_test_variable_cells(selftest)
    selftest_progress_cells(selftest)
    selftest_stop_cells(selftest)


def run_selftest():
    work = tempfile.mkdtemp(prefix="broad-mutation-selftest.")
    selftest = Selftest(work)
    try:
        run_selftest_cells(selftest)
    finally:
        open(os.path.join(work, "release"), "w").close()   # 挂住的那一步（有的话）见到它就退
        time.sleep(0.2)
        if selftest.slice_used:
            # 自证自己开的那一个 slice（名字带这个进程的进程号）：停掉，不留着
            subprocess.run(["systemctl", "--user", "stop", selftest.slice], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ --selftest：{selftest.cells} 格里 {selftest.failures} 格判错", file=sys.stderr)
        print(f"     → 怎么办：照上面每一格给的出路改，再跑 python3 {SELF_PATH} --selftest", file=sys.stderr)
        return 1
    say("  ✓ --selftest：临时库目录拒、导出与列项、项写错拒、点名与两段、杀掉再起只补没做完的、各种结局与汇总八栏、第二段只补第一段没跑到的、--from 与 --shard、脏树拒、"
        "点名跳过已有定论的条与 --rerun、定论不被降级、--stage-two-only 要过过第一段、变异名对清单、末行没写完、认不出的退出码、重起验编译与连着无效或秒红、"
        "核树看多出来的文件、换树的次序、工具链进指纹、全局键进指纹与真跑核全局键、全局键逐类进键（工具链、工具版本、限时参数、构建环境另七类，源码指纹算路径）、"
        "加大限时与上限再点名重跑判出结论、超时与撞顶自动升档（证据齐的判抓到并登记、升档那趟跑完照它往后判、证据不齐的没结论、--summary 逐条列、"
        "升档那趟按自己的上限排队、变异步撞总上限时先挨杀）、升档证据五支各一格、基线按线程数分开记且登记行指的那一行对得上、结果行带密钥校验列（手追加的不算）、"
        "--unit-environment 打全局键与工具链认的会话变量、--changed-since 重跑超时与内存撞顶的条、换树后编译目录不留过时的工作区产物、基线之前核旧产物、判法版本进指纹与全局键、合并导入、忽略用例登记与排除表（没登记、指着不存在、两边都在、写坏、命令带 -E、旧基线行不认）、只供自证的变量不出自证、一步跑过进度间隔要报进度、被 TERM 连 scope 一起停，"
        f"{selftest.cells} 格都对")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    try:
        sys.exit(main(sys.argv[1:]))
    except Refusal as refusal:
        print_refusal(refusal)
        sys.exit(refusal.code)
    except Stopped:
        print("  ✗ 被 TERM / INT 停了：在途那一条没落行，在途那一步已一起停掉", file=sys.stderr)
        print("  → 怎么办：重跑同一条命令接着跑（树里留着改过的文件时开跑先照在途记录拷回原件）", file=sys.stderr)
        sys.exit(EXIT_STOPPED)
