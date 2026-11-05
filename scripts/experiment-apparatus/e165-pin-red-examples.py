#!/usr/bin/env python3
# admission: always 每一次调都判此刻的红例账、钉住的表与变异表，上一次的结论不替这一次作保
# run-condition: none 追加与 --check 只读写点名的文件，--check-table 只读钉表与变异表；--replay 要的 cargo 与装置由它自己判，缺了判红或报本次未跑
"""E165 红例钉进门禁：把 E165 第二段驱动写出的红例账里的每个红例按输入原样钉进 .claude/gate.d/e165-pinned-red-examples.tsv，并判钉得全不全、今天还绿不绿。

用户 2026-10-01 定「全量形式化红了的例子 可以加到快速形式化中」「这个应该在每一次跑完之后就加入 门禁来控制比较好」
（调度记录 records/2026-10-01-里程碑三形式化接入.md 第三节）。表的格式写在表自己的文件头。

用法（<根> 默认取这份脚本往上两级的仓；--table 默认 <根>/.claude/gate.d/e165-pinned-red-examples.tsv）：
  档（㉒）：出这份账的那一趟装置的档参数。取同一趟 kill-bounds 产物（配法见下）第一行 E7RESULT name=config 的 device_arguments=（参数用 + 连），
      只认装置 split_off_the_profile_options 认的四个：--profile、--marker-parity、--table、--disk-order（钉住的红例逐行重放时装置只认这四个，别的不进逐态行）；
      --bounds、--family、--only、--max-n 只选跑哪些域与 N，不进档；别的参数（例 --gate、--max-u）认不出进不进逐态行，判红、这份账一行都不钉。
      写成规范的一串：「--profile <档名>」，后面只接不是默认值的那几个（--marker-parity odd、--table scan|both、--disk-order reversed，依这个次序），空格分隔；
      产物没有 config 行、config 行没有 device_arguments=、或 device_arguments= 的值是空的，判红、这份账一行都不钉（出路：重跑那一趟，驱动每趟都写这一段；不手填、不补）；
      device_arguments= 里没有 --profile 的是 --profile r1（装置不给 --profile 取 r1）。
      同一个（①, ⑥）在不同档下各钉一行，不算冲突；同一档下 ⑦ 不同照旧判红。① 不带档（装置 --pinned-cases 拿 ②③ 与逐态行的 state= 重算 ①，与档无关）。
  e165-pin-red-examples.py [--root 根] [--table 表] [--kill-bounds 产物]… <红例账…>
      追加：账里还没钉的行追加进表，一行 22 段：自己的 ①–⑧ 后面原样接账那一行的 13 段（⑨–㉑），再接这份账的档（㉒）。
      已钉过的不重复：表里有一行 ⑨–㉑ 与这一行账逐字节相同、㉒ 等于这份账的档，且 ⑥ 等于这一次换出的名字或 ⑤ 就是这一行账。
      同一个（①, ⑥, ㉒）而 ⑨ 起不同的（另一趟的账，⑱ 首红与否或 ⑨⑩ 的变体与表行不同）另钉一行。
      账里没有反例状态的行（序号 none，驱动报 no_single_state_differs）钉不了，只数不钉。
      ⑥ 写「<表>#<变异名>」，换名与 --check 同一套：每份账配同一趟的 kill-bounds 产物，不给 --kill-bounds 按文件名配（与 --check 相同），
      给了就逐份对应（第 i 个产物配第 i 份账，个数要相等）；配对的四样（见 --check 一节）有一样不过就判红、一行都不钉，不退回按今天的行号换名；
      E165 第二段驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh 写完一对 <前缀>-red-examples.tsv 与 <前缀>-kill-bounds.out 之后，
      当场带 --kill-bounds 照这一对追加，再照同一对跑 --check（驱动文件头「钉红例」一段）；
      配上了取产物里这个变体那一行（item= 等于账第 1 列、table_row= 等于账第 2 列的第一个表行）的
      table= 与 title=，table= 要是这种变体的表（tsv-<行号> 是 crates/mutations.tsv，Z<n> 是 scripts/mutation/tables/e165_replay_bounded_enumeration_custom.tsv），
      在今天的那张表里找第 1 列（空格记成 _）等于 title= 的数据行，恰好一行就用它的变异名：那一趟之后变异表增删过行也对得上，不按今天的行号换。
      table= 不对、按名字找不到、找到不止一行的，逐行报出来、不钉，别的行照钉，退 1。
      同一个（①, ⑥, ㉒）在表里（或这一次前面的账里）的 ⑦ 与账里的不同：一行都不钉、判红。
      写：读表时记 sha256，同目录临时文件写完、改名换上之前再算一次，变了就不写（别的会话刚写过，重跑）；写完回读，新钉的每一行都要在。
  e165-pin-red-examples.py --check [--root 根] [--table 表] [--kill-bounds 产物]… <红例账…>
      判点名的账钉全了没有。要显式给账：不给退 2（说明「--check 要显式给账与同一趟的产物」），不扫任何目录。
      --kill-bounds 与追加同一套：给了就逐份对应（第 i 个产物配第 i 份账，个数要相等），不给按文件名配同一目录的 <前缀>-kill-bounds.out。
      E165 第二段驱动写完一对账与产物、追加之后照同一对这样调（显式给账与 --kill-bounds），没钉全那一趟判红；门禁不调 --check。
      账里每一行（有反例状态的）都要钉着：
      表里有一行 ① 等于这一行算出的指纹、⑦ 等于账的第 12 列、⑨–㉑ 用制表符接起来与账那一行逐字节相同、㉒ 等于这份账的档（按档判钉全），
      且 ⑤ 就是这一行账或 ⑥ 是出这份账的那一趟给这个变体的名字。同一个输入钉着、只是 ⑨ 起不同的，逐行报「没整行钉」；
      整行都对、只是 ㉒ 不是这份账的档的，逐行报「钉的档不对」（出路：照这份账的档重钉）。
      那一趟的名字不按今天的变异表换（表在跑完之后删一行，后面的 tsv-<行号> 就全错一位），取同一趟的 kill-bounds 产物
      （--kill-bounds 给的那一份；不给就是同一目录的 <前缀>-kill-bounds.out，E165 第二段驱动 e165-replay-mutation-kill-bounds.sh 一趟写这一对）。
      配对四样都要过，有一样不过就判红：产物在；产物的收尾行 E7RESULT name=verdict 在、它的 ledger_sha256= 等于这份账此刻的 sha256（驱动写完账才写这一行）；
      账里有行的变体与产物里 red_examples= 大于 0 的变体是同一批、每个变体的账行数等于它的 red_examples=；
      账的每一行按（第 1 列变体, 第 2 列的第一个表行）在产物里有 E7RESULT name=kill_bound 那一行（item=、table_row=），带 title=。
      ⑥ 的表（# 之前）等于那一行的 table=、变异名（# 之后，空格记成 _）等于它的 title=，才算同一个变异。
      没钉全的出路逐份给 python3 scripts/experiment-apparatus/e165-pin-red-examples.py <账>（给了 --kill-bounds 的另带 --kill-bounds <产物>）：追加按同一趟产物的 title=
      到今天的变异表里按名字找变异，那一趟之后变异表增删过行也钉得进；按名字找不到或找到几行的逐行报出来、不钉，不钉进错的名字。
      表自己也判（--check、--check-table、--replay 与追加都先判，判法 table_problems）：每行 22 段（段数不是 22 的行，不管差几段，一律报行号与要 22 段）、
      ① 是 64 位十六进制、① 等于拿 ②③⑦ 重算的指纹、⑪⑫⑬ 与 ②③④ 相同、⑳㉑ 与 ⑦⑧ 相同（不同的点名哪一行哪一段，与装置 --pinned-cases 同一种报法）、
      ㉒ 是规范写法的档、（⑥, ⑨–㉑ 的整行, ㉒）不重复。少一行判红，出路给补进去的命令。
  e165-pin-red-examples.py --check-table [--root 根] [--table 表]
      门禁 pinned-red-examples-table 格：只判钉表自己，不读红例账、kill-bounds 产物与 research/ 下的任何文件。判两样，都与 --replay 开头那一半共用同一份判法：
      表本身（上面「表自己也判」那几样，table_problems）；每行 ⑥ 点名的变异（<表>#<名>，几个用 + 连）都还在它的表里（第 1 列等于那个名字的数据行，missing_variants）。
      成功那句报表的行数、按档各几行、点名的变异几处。表一行数据都没有退 77；给了红例账退 2。每趟现判、不复用（只读几份文件）。
  e165-pin-red-examples.py --replay [--root 根] [--table 表] [--binary 可执行文件] [--threads N] [--target 编译目录]
      --threads 缺省取 $SINGLEFS_THREAD_CAP，再取登记的重活默认并行度（.claude/heavy-thread-default）。
      门禁 pinned-red-examples-replay 格：先判每行 ⑥ 点名的变异（<表>#<名>，几个用 + 连）还在它的表里（第 1 列等于那个名字的数据行），
      再用装置 e165_replay_bounded_enumeration --pinned-cases <表> 在今天的实现上逐行判：stdout 里以「E7RESULT name=state 」起头的行
      条数等于表的数据行数、每一行与按行身份配上的那个数据行的 ⑦ 逐字节相同、装置退 0（约定：每行一行结果，有行对不上或指纹漂了退非 0）。
      行身份是逐态行的 domain=、n=、ordinal= 三段，对表行的 ②③④（pair_result_lines）：块里同一个身份有几行、装置打了几行这个身份的结果，两数相同才按次序配，
      不同就这个身份的几行都配不上；不按下标配（块里有一行装置没打结果时，后面的行不错位）。
      按档分组起装置：㉒ 相同的行一组，每组按块起，命令是「<装置> --pinned-cases <块表> <㉒ 按空格切开>」（例 --profile r2）；
      块表只留这一块的行、每行只留 ①–㉑（装置只认 21 段，㉒ 不交给它），别的数据行换成 #、行号不变。
      不给 --binary 时：cargo build --release --locked -p singlefs-verification --bin e165_replay_bounded_enumeration
      （CARGO_TARGET_DIR：--target 给了用它、不删；没给是这一趟自己建的 ${TMPDIR:-/tmp}/singlefs-e165-pinned-target-*，跑完删——删之前核没有进程开着里面的文件，
      有就不删、报出进程、退 1；用户 2026-10-23 定案「自己用完自己删」，不跨轮复用）再跑。
      装置线程取 --threads（或 SINGLEFS_THREAD_CAP、登记的默认并行度），不套内存包装、不设内层上限：整条命令交 task-server，内存额度按记录由服务给，撞上限由服务放大重排；
      装置被信号停了的（subprocess 交负的信号号，或经 bash 起时退 128 + 信号号，例 137 = 外面 kill -9），判红，
      ✗ 行写「装置被信号停了」、这一次的输出不算判定，→ 查是谁在那一刻停了它，不写修实现；
      --binary 以 .sh 结尾的经 bash 起（样本目录里的桩不带执行位）。
      编装置与跑装置时每过 300 秒（PROGRESS_INTERVAL_SECONDS，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」定的进度间隔）往 stderr 打一行
      「E165_PINNED_PROGRESS 挂钟 <秒> 秒：<在做什么> 还在跑；装置已打 n/m 行结果」（编装置那一步不带行数）；判定照旧等它跑完、按全部输出判。
  e165-pin-red-examples.py --list-items [--root 根] [--table 表] [--key-mismatches] [--binary 可执行文件]
      逐行打可点名的项「项<TAB>格<TAB>说明」：table（pinned-red-examples-table，一项），钉表每个数据行一项 pinned:<表里的行号>
      （pinned-red-examples-replay，说明是 ⑤、⑥ 与「档 <㉒>」，段数不是 22 的行写「<段数> 段」）。只读钉表，不编、不起装置；表不在判红退 1。
      给了 --key-mismatches：库放在哪与 --replay 同一份（被判那棵仓定），另往 stderr 逐行一行「ITEM_KEY item=pinned:<行号> state=<键相同|键对不上|无记录|拒算>」
      （钉表每一行此刻的键在库里是哪一种；与 --replay 判这一行时算的键同一份，只读库、不写库、不编装置；table 那一项每趟现判、不按条入库，不打；自证 list-items-item-key-lines）。
      环境里有 FORMAL_ITEM_DONE_FILE 的（门禁给的），--replay 每行判完另追加一行「项<TAB>秒数<TAB>现判条数<TAB>复用条数<TAB>退出码」（record_item_done，门禁收尾转成 ITEM_DONE）：
      复用的 0 秒、复用 1 条；现判的取装置跑那一块的挂钟（一块里几行共用一次装置，没有比一块更细的挂钟）、现判 1 条；绿 0、红 1（自证 item-done-lines）。
  逐项点名与按条复用（只给 --replay；别的模式给 --item 退 2）：
      --item <项>（可给多次，取并集）只重放点名的行，项名写错或缺项名退 2 并列出可点名的项；点了的行照样先判表本身与点名的变异还在。
        点名单跑一行与它在整格里跑的判定相同（自证 item-alone-equals-whole）：装置按块起，一块只留这一块的几行（别的数据行换成 #，行号不变），
        一行的判定只看它自己打的那一行与 ⑦ 相同、装置的报错里没点它的行号；同一块里别的行红不连累它。
      --block-rows N（默认 DEFAULT_BLOCK_ROWS）一块交给装置几行；一块判完，那一块里判绿的行当场落盘：配上了自己那一行结果、与 ⑦ 相同、报错里没点它行号的照键记绿，
        块不干净（装置退非 0、行数对不上）也照记（用户 2026-10-29：「不写库等于没有实现复用的功能」）；配不上的不记，
        被打断之后同一条命令重跑，判过的块复用、只补没判的。
      --replay 按条记判定（判定库 scripts/crash/lib_verdict_library.py）：每一行的键相同就复用，一行都不用现判时不编装置、不起重放；
        table（--check-table）每趟现判、不复用。
        判红的那一行（装置给它出了结果：打了一行与 ⑦ 不同、或报错里点了它的行号）照同一把键调共用库的 record_red 记红，盖掉原有的绿，下一趟这一行现判
        （用户 2026-10-29「记红、撤绿、下趟现判」；自证 reuse-red-withdraws-green）；装置没给它出结果的不碰库。
        库位置只由被判那棵仓（--root）定，命令行不收别的：python3 scripts/gate/verus-proof-check.py library-directory <根> 打的那一份（judged_library_directory，
        被判那棵树的 git common-dir 底下的 singlefs-formal-unit-verdicts）；取不到（不是 git 工作树）参数解析之后一项都不判、退 2（自证 replay-outside-git-exits-2）。
        自证在进程里调 main 时给各格各自的临时库目录（main 的 selftest_library；临时服务底下各占一个命名空间）。
      键分两列（判定库文件头「键」一段）：
        判法单元指纹：这份脚本里 replay 的单元闭包键（判结局的那几个函数：读表、判变异还在、分块、拼起装置的命令、认信号的结局、比 ⑦；
          判定库与 lib_function_units.py 不进；只改注释不变），变了整格每一行都现判，原因写「判法单元指纹 变了」。
          编装置与起装置的那一下（run_reporting_progress：起 cargo 或 --binary 给的程序）不在这个闭包里：replay 经 options["spawn"] 调它（main 给），
          起的是什么程序由下面「装置」那几样进键，不靠判法单元指纹罩；所以判法单元指纹算得出、不拒算。
        输入（逐样的单元键或摘要按名字排好再取 sha256）——每一行都带的装置那几样：装置单元闭包（装置的 Rust bin 入口
          crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs 的 main 起的函数单元闭包键，scripts/gate/lib_function_units.py 的切法：
          装置源码改一个它调到的函数键变，只改注释、改没被调的函数不变；工具链行是 cargo -V、rustc -V 与编装置的那一条构建配置，
          依赖行是 Cargo.lock 里 singlefs-verification 传递引到的外部包）；给了 --binary 的另加「装置替身」：那份程序的内容摘要（样本与自证的桩就是这样，
          换一份桩键变），这时装置单元闭包算不出就记一句原因；不给 --binary 而闭包算不出判红；闭包里有拒算的那几行不复用、每趟现判。
          变了整格每一行都现判，现判的原因点名变了的单元。
        每一行另加：状态输入（①②③④）、期望逐态行（⑦）、档（㉒，单列一样，变了原因写「档 变了」）、变异原文与替换文（⑥ 点名的每一条变异在它的表里那一行的第 3、4 段）。
          出处 ⑤（红例账路径与行号）、变异名、⑧ 与抄来的账行 ⑨–㉑ 不进：重放只读 ②③④⑦ 与档，账改名、变异改名、账里别的段不改这一行的判定。
      只记绿。每行一行「· pinned:<行号>：绿|红，现判（<原因>）|复用（<记下的日期与一句判定>）」，末了「· 按条：现判 N、复用 M（现判的原因：…）」；
        环境里有 FORMAL_ITEM_TALLY_FILE 的（门禁给的）另往那份文件追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」。
  e165-pin-red-examples.py --selftest
      在临时仓里造账、变异表、产物与桩装置，逐格判；弄坏开关逐个打开，每个都要让至少一格判错。
      判定库那几格写进 prune-server：只在临时服务底下跑，裸跑开跑前调客户端 selftest_service_problem、退 2、一格都不跑。起法：
      python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- python3 scripts/experiment-apparatus/e165-pin-red-examples.py --selftest
退出码：0 绿（追加：钉好了或没有要钉的）；1 红；2 用法错；77 本次未跑（--check-table 与 --replay 的表一行数据都没有）。

弄坏开关 E165_PIN_RED_EXAMPLES_BREAK=<项>[,<项>…]（只在 --selftest 与门禁样本根上认：命令行第一个参数不是 --selftest、又不是自证起的子进程（自证设 E165_PIN_RED_EXAMPLES_SELFTEST=1）、--root 给的也不是样本根（有 .gate-cells 或 .gate-break-sample）时设了它，参数解析之前退 2、点名它只在自证里认；--selftest 与门禁样本在开着时都必须判错）。逐项写开关名、弄坏哪一处，下一行起写打开后自证红的格（自证末尾一句「✗ e165-pin-red-examples 自证 N 格里 M 格判错：<格名 格名 …>」，下面的名字就是那一句列的格名；2026-10-29 逐项实测）：
  E165_PIN_RED_EXAMPLES_BREAK=pin-twice  追加不去重
      「pin-again」「check-green」「check-missing」「check-tampered-setup」「check-whole-row-differs」「check-table-without-research」「check-table-archived-ledger」「check-table-variant-missing」「check-run-names」「check-explicit-kill-bounds」「check-explicit-kill-bounds-digest」「pin-whole-row-variant」「check-whole-row-variant」「pin-kill-bounds-missing」「pin-ledger-digest」「pin-explicit-kill-bounds-digest」「pin-deleted-row」「pin-deleted-row-names」「check-deleted-row」「check-shifted-table」「check-title-other-variant」「check-title-other-variant-howto」「check-title-missing」「check-kill-bounds-missing」「check-variant-sets-differ」「check-ledger-digest」「check-run-unfinished」「conflict」「replay-green」「replay-mismatch」「replay-fail」「replay-progress-green」「replay-progress」「replay-killed」「replay-killed-howto」「replay-exit-137」「replay-variant-missing」「list-items」「item-select-one」「item-unknown」「item-alone-equals-whole」「reuse-first-run-records」「reuse-second-run-reuses」「reuse-judging-units-changed」「reuse-mutation-row-changed」「reuse-mutation-row-restored」「reuse-device-changed」「reuse-device-stand-in-swapped」「reuse-ledger-path-renamed」「reuse-mutation-renamed」「reuse-row-own-key」「reuse-red-withdraws-green」「reuse-red-not-recorded」「reuse-resume-after-kill」「replay-unclean-block-records-green」「replay-missing-line-no-mispair」「prune-unchanged-input」「prune-after-change」「reuse-closure-changed」「reuse-closure-unrelated-function」「reuse-closure-unchanged」「reuse-crate-root-file-unchanged」「reuse-device-source-function-changed」「pin-two-profiles」「pin-profile-r2」「check-two-profiles」「check-profile-differs」「conflict-same-profile」「pin-profile-unknown-argument」「pin-product-without-config」「pin-product-without-arguments」「pin-product-empty-arguments」「pin-product-howto」「check-product-without-config」「replay-two-profiles」「replay-profile-arguments」「list-items-profile」「reuse-row-own-profile」
  E165_PIN_RED_EXAMPLES_BREAK=titles-unchecked  追加按 title 找不到变异时，退回按今天的行号换名、不报
      「titles-mismatch」「pin-by-title-not-found」「pin-by-title-not-found-detail」
  E165_PIN_RED_EXAMPLES_BREAK=no-recheck-before-write  改名换上之前不复核 sha256
      「race」「race-wrote」「pin」
  E165_PIN_RED_EXAMPLES_BREAK=check-ignores-missing  --check 不报没钉的行
      「check-empty-table」「check-missing」「check-whole-row-differs」「check-title-other-variant」「check-title-other-variant-howto」「check-profile-differs」
  E165_PIN_RED_EXAMPLES_BREAK=fingerprint-unchecked  表的判法不拿 ②③⑦ 重算 ①
      「check-tampered」「check-table-tampered」
  E165_PIN_RED_EXAMPLES_BREAK=variant-unchecked  --check-table 与 --replay 不判变异还在不在
      「check-table-variant-missing」「replay-variant-missing」
                                                     （样本 pinned-red-examples-replay-variant-red、pinned-red-examples-table-variant-red）判绿
  E165_PIN_RED_EXAMPLES_BREAK=replay-ignores-output  --replay 只看装置退出码，不比 ⑦
      「replay-mismatch」「item-alone-equals-whole」「reuse-red-not-recorded」「prune-after-change」「replay-missing-line-no-mispair」
  E165_PIN_RED_EXAMPLES_BREAK=killed-as-regression  --replay 不认装置被信号停了，照装置退非 0 报回归、出路写修实现
      「replay-killed」「replay-killed-howto」「replay-exit-137」
  E165_PIN_RED_EXAMPLES_BREAK=names-from-today-table  --check 照今天的变异表换 tsv-<行号>，不认产物里的名字
      「check-shifted-table」「check-title-other-variant」「check-title-other-variant-howto」
  E165_PIN_RED_EXAMPLES_BREAK=kill-bounds-optional  --check 账配的产物不在也不红，照今天的变异表换名
      「pin-kill-bounds-missing」「pin-kill-bounds-missing-wrote」「pin-deleted-row」「pin-deleted-row-names」「check-kill-bounds-missing」
  E165_PIN_RED_EXAMPLES_BREAK=title-missing-ignored  --check 产物里那个变体没有那一行或没有 title= 也不红，照今天的变异表换名
      「check-title-missing」
  E165_PIN_RED_EXAMPLES_BREAK=title-not-compared  --check 产物里有 title= 就算钉着，不比 ⑥
      「check-title-other-variant」「check-title-other-variant-howto」
  E165_PIN_RED_EXAMPLES_BREAK=variant-sets-unchecked  --check 不比账与产物的变体与行数
      「check-variant-sets-differ」
  E165_PIN_RED_EXAMPLES_BREAK=ledger-digest-unchecked  --check 不核产物收尾行记的账 sha256、不认没收尾
      「check-explicit-kill-bounds-digest」「pin-ledger-digest」「pin-explicit-kill-bounds-digest」「pin-deleted-row」「check-ledger-digest」「check-run-unfinished」
--check 与 --check-table 不读调用方没点名的账（门禁那一格只判钉表自己）的三项，开关一行、下一行起写打开之后判错的自证格与门禁样本：
  E165_PIN_RED_EXAMPLES_BREAK=table-check-globs-research  --check-table 照改前门禁那一格的判法：取 research/results/e165-*-red-examples.tsv、按文件名配产物判钉全
      「check-table-without-research」「check-table-archived-ledger」「check-table-variant-missing」「check-table-empty」
      样本 pinned-red-examples-table-archived-ledger-green 判红、pinned-red-examples-table-green 退 77
  E165_PIN_RED_EXAMPLES_BREAK=check-globs-by-default  --check 不给账时照改前取 research/results/e165-*-red-examples.tsv，不退 2
      「check-without-ledgers」
  E165_PIN_RED_EXAMPLES_BREAK=check-kill-bounds-ignored  --check 给了 --kill-bounds 也不用，照旧按文件名配同一目录的产物
      「check-explicit-kill-bounds」「check-explicit-kill-bounds-digest」
整行抄红例账（钉表一行 22 段：自己的 8 段、账一行的 13 段、㉒ 档）的几项，开关一行、下一行起写打开之后判错的自证格与门禁样本（样本在 .claude/gate.d/fixtures/formal-verification.sh/ 下）：
  E165_PIN_RED_EXAMPLES_BREAK=whole-row-unchecked  --check 不比 ⑨ 起的整行，只认 ①、⑦ 与 ⑤ / ⑥（整行抄之前的判法）
      「check-whole-row-differs」
  E165_PIN_RED_EXAMPLES_BREAK=copied-columns-unchecked  不核 ⑪⑫⑬ 与 ②③④、⑳㉑ 与 ⑦⑧
      「replay-copied-domain-differs」「replay-copied-original-differs」「check-table-copied-differs」
  E165_PIN_RED_EXAMPLES_BREAK=dedupe-by-input-variant  追加按（①, ⑥）去重：另一趟的账 ⑨ 起不同也算已钉（整行抄之前的判法）
      「pin-whole-row-variant」「check-whole-row-variant」
  E165_PIN_RED_EXAMPLES_BREAK=variant-by-line-number  追加配上了产物也按今天的行号换名、再核 title 相同（按名字找之前的判法：那一趟之后变异表增删过行就一行都不钉）
      「pin-by-title-shifted-table」「check-by-title-shifted-table」「pin-by-title-not-found」「pin-by-title-not-found-detail」「pin-by-title-ambiguous」「pin-by-title-ambiguous-detail」「pin-by-title-same-names」「pin-deleted-row」「pin-deleted-row-names」「check-deleted-row」
      pin-by-title-not-found、pin-by-title-not-found-detail、pin-by-title-ambiguous、pin-by-title-ambiguous-detail 判错（不报按名字找不到 / 找到几行，同名的照行号钉了进去）
追加按文件名配同一趟产物（与 --check 同一套）的两项：
  E165_PIN_RED_EXAMPLES_BREAK=append-names-from-today-table  追加配上了产物也不用它的名字，照今天的变异表按行号换名（不给 --kill-bounds 时的旧判法）
      「titles-mismatch」「pin-by-title-not-found」「pin-by-title-not-found-detail」「pin-by-title-ambiguous」「pin-by-title-ambiguous-detail」「pin-by-title-same-names」「pin-deleted-row-names」
      titles-mismatch、pin-by-title-not-found、pin-by-title-not-found-detail、pin-by-title-ambiguous、pin-by-title-ambiguous-detail、pin-by-title-same-names 同样判错
  E165_PIN_RED_EXAMPLES_BREAK=append-pairing-optional  追加配不上产物（产物不在、收尾行记的账 sha256 不同）也不红，照今天的变异表按行号换名照钉
      「pin-kill-bounds-missing」「pin-kill-bounds-missing-wrote」「pin-ledger-digest」「pin-explicit-kill-bounds-digest」「pin-deleted-row」「pin-deleted-row-names」
--replay 的进度行：
  E165_PIN_RED_EXAMPLES_BREAK=progress-silent  编装置、跑装置时不打进度行（跑完之前什么都看不到）
      「replay-progress」
--replay 的逐项点名与按条复用：
  E165_PIN_RED_EXAMPLES_BREAK=item-filter-ignored  --item 不起作用、照重放全部（写错的项名也不退 2）
      「item-select-one」「item-unknown」「reuse-row-own-key」「reuse-row-own-profile」
  E165_PIN_RED_EXAMPLES_BREAK=reuse-key-without-closure  键里不带装置单元闭包（装置调到的函数改了照旧复用）
      「reuse-closure-changed」「reuse-device-source-function-changed」
  E165_PIN_RED_EXAMPLES_BREAK=reuse-key-without-row  键里不带这一行的状态输入与期望逐态行（同一个输入、只差 ⑦ 的两行共用一个键，只判过一行就拿它的判定当另一行的）
      「reuse-row-own-key」
  E165_PIN_RED_EXAMPLES_BREAK=reuse-key-without-judging-units  判法单元指纹一律交空串（判结局的函数改了照旧复用）
      「reuse-judging-units-changed」
  E165_PIN_RED_EXAMPLES_BREAK=stand-in-unkeyed  给了 --binary 时「装置替身」（那份程序的内容摘要）不进键（换一份桩照旧复用）
      「reuse-device-changed」「reuse-device-stand-in-swapped」「prune-after-change」
  E165_PIN_RED_EXAMPLES_BREAK=judging-units-raw-text  判法单元指纹取这份脚本整份原文的摘要（只改 replay 里一行注释也变）
      「judging-units-comment-only」
  E165_PIN_RED_EXAMPLES_BREAK=spawn-inside-judging-closure  起装置那一下（run_reporting_progress）算进判法入口：起 --binary 给的程序那一段回到闭包里，库折不出那条命令，判法单元指纹拒算、每趟现判
      「reuse-first-run-records」「reuse-second-run-reuses」「reuse-judging-units-changed」「reuse-mutation-row-changed」「reuse-mutation-row-restored」「reuse-device-changed」「reuse-device-stand-in-swapped」「reuse-ledger-path-renamed」「reuse-mutation-renamed」「reuse-row-own-key」「reuse-red-not-recorded」「reuse-resume-after-kill」「prune-unchanged-input」「reuse-closure-changed」「reuse-closure-unrelated-function」「reuse-closure-unchanged」「reuse-crate-root-file-unchanged」「reuse-device-source-function-changed」「judging-units-no-refusal」「judging-units-comment-only」「reuse-row-own-profile」「replay-unclean-block-records-green」「replay-missing-line-no-mispair」
  E165_PIN_RED_EXAMPLES_BREAK=row-key-with-ledger-path  每一行的键另带 ⑤（红例账的路径与行号，旧判法）
      「reuse-ledger-path-renamed」
  E165_PIN_RED_EXAMPLES_BREAK=row-key-with-mutation-name  变异原文与替换文那一样另带 ⑥ 的表与变异名
      「reuse-mutation-renamed」
  E165_PIN_RED_EXAMPLES_BREAK=list-items-pipe-unhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
      「list-items-closed-pipe」
  E165_PIN_RED_EXAMPLES_BREAK=reuse-key-without-mutation-row  键里不带 ⑥ 引的变异行（变异表里那一行改了照旧复用）
      「reuse-mutation-row-changed」「reuse-row-own-key」
  E165_PIN_RED_EXAMPLES_BREAK=reuse-ignores-key  这一行判绿过就复用，不比键
      「reuse-judging-units-changed」「reuse-mutation-row-changed」「reuse-device-changed」「reuse-device-stand-in-swapped」「prune-after-change」「reuse-closure-changed」「reuse-device-source-function-changed」
      reuse-judging-units-changed 跟着判错
  E165_PIN_RED_EXAMPLES_BREAK=reuse-records-red  红的也记成绿
      「reuse-red-not-recorded」「replay-unclean-block-records-green」「replay-missing-line-no-mispair」
  E165_PIN_RED_EXAMPLES_BREAK=reuse-records-whole-cell-only  一块判完不当场落盘，整格判完、一行红都没有才一起记（被打断或有一行红，判过的全不留）
      「reuse-red-not-recorded」「reuse-resume-after-kill」「replay-unclean-block-records-green」「replay-missing-line-no-mispair」

  E165_PIN_RED_EXAMPLES_BREAK=red-keeps-green  判红的那一行不记红（不调共用库的 record_red），同一把键原有的绿留着照复用（改前的样子）
      「reuse-red-withdraws-green」
  E165_PIN_RED_EXAMPLES_BREAK=unclean-block-records-nothing  块不干净（装置退非 0、行数对不上）时那一块一行都不记绿（改前的样子）
      「replay-unclean-block-records-green」「replay-missing-line-no-mispair」
  E165_PIN_RED_EXAMPLES_BREAK=results-paired-by-position  结果行按下标配到块里的登记行（改前的样子：块里少打一行，后面的行错配）
      「replay-missing-line-no-mispair」
钉表认档（㉒）的几项，开关一行、下一行起写打开之后判错的自证格与门禁样本：
  E165_PIN_RED_EXAMPLES_BREAK=profile-read-as-r1  取档不读产物的 device_arguments=，一律当 --profile r1（r2 档的账钉成 r1 档）
      「pin-two-profiles」「pin-profile-r2」「check-two-profiles」「check-profile-differs」「pin-profile-unknown-argument」「profile-normalized」「replay-two-profiles」「replay-profile-arguments」「list-items-profile」
  E165_PIN_RED_EXAMPLES_BREAK=product-profile-defaulted  产物没有 config 行、config 行没有 device_arguments=、或它的值是空的，照取 --profile r1（改前的样子），不判红
      「pin-product-without-config」「pin-product-without-arguments」「pin-product-empty-arguments」「pin-product-howto」「check-product-without-config」「profile-empty-refused」
  E165_PIN_RED_EXAMPLES_BREAK=same-profile-conflict-ignored  追加不判同一档下同一个（①, ⑥）⑦ 不同，照钉
      「conflict」「replay-progress」「conflict-same-profile」「conflict-same-profile-unwritten」
  E165_PIN_RED_EXAMPLES_BREAK=check-ignores-profile  --check 不比 ㉒，钉成别的档也算钉着
      「check-profile-differs」
  E165_PIN_RED_EXAMPLES_BREAK=profile-form-unchecked  表的判法不核 ㉒ 是不是规范写法的档
      「check-table-profile-form」
  E165_PIN_RED_EXAMPLES_BREAK=replay-without-profile  --replay 起装置不带 ㉒ 的档参数（装置一律按 r1 档重放）
      「replay-two-profiles」「replay-profile-arguments」
  E165_PIN_RED_EXAMPLES_BREAK=row-columns-unchecked  表里段数不是 22 的行不报，当表里没有这一行
      「check-columns-8」「check-columns-21」「replay-columns」「replay-columns-howto」
  E165_PIN_RED_EXAMPLES_BREAK=profile-arguments-unchecked  device_arguments= 里认不出的参数（例 --gate）照略过，不判红
      「pin-profile-unknown-argument」
  E165_PIN_RED_EXAMPLES_BREAK=reuse-key-without-profile  按条复用的键不带档（「这一行」不算 ㉒、不单列「档」）
      「reuse-row-own-profile」
  E165_PIN_RED_EXAMPLES_BREAK=threads-default-cpu-count  --threads 的缺省取核数（改前的样子），不取登记的重活默认并行度
      「threads-default-registered」
"""
import concurrent.futures
import glob
import hashlib
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "kb"))
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "crash"))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts"))
from heavy_threads import default_heavy_threads  # noqa: E402
from lib_verdict_library import (NOT_JUDGING_FILES, FunctionUnitIndex, Tally, UnitKey, VerdictLibrary, digest_file, digest_text,  # noqa: E402
                                 judging_unit_key, key_of, locked_dependency_lines, refused_component, rust_toolchain_lines)
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
DEFAULT_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
TABLE = ".claude/gate.d/e165-pinned-red-examples.tsv"
# 只给两个弄坏开关用（table-check-globs-research、check-globs-by-default）：改前门禁那一格不给账时扫的那一批账
LEGACY_LEDGER_GLOB = "research/results/e165-*-red-examples.tsv"
LEDGER_SUFFIX, KILL_BOUNDS_SUFFIX = "-red-examples.tsv", "-kill-bounds.out"
CHECK_NEEDS_LEDGERS = "--check 要显式给账与同一趟的产物"
CRATES_TABLE = "crates/mutations.tsv"
CUSTOM_TABLE = "scripts/mutation/tables/e165_replay_bounded_enumeration_custom.tsv"
DEVICE_SOURCE = "crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs"
DEVICE_BINARY = "e165_replay_bounded_enumeration"
DEVICE_FLAG = "--pinned-cases"
RESULT_PREFIX = "E7RESULT name=state "
LEDGER_COLUMNS, TABLE_OWN_COLUMNS = 13, 8
# 装置 --pinned-cases 认的段数（①–㉑）；钉表另在最后接 ㉒ 档，交给装置之前去掉
DEVICE_COLUMNS = TABLE_OWN_COLUMNS + LEDGER_COLUMNS
PROFILE_INDEX = DEVICE_COLUMNS
TABLE_COLUMNS = DEVICE_COLUMNS + 1
COLUMN_MARKS = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳㉑㉒"
# 抄来的账行与钉表自己的段要相同的几对：（账的第几段, 钉表自己的第几段）；账第 3、4、5 段是域、N、序号，第 12、13 段是两条逐态行
COPIED_PAIRS = ((3, 2), (4, 3), (5, 4), (12, 7), (13, 8))
TABLE_PROBLEMS_HOWTO = ("段数不对、指纹或抄来的段对不上都是手改坏了：表只由这份脚本写，从 git 找回再钉")
# 档（㉒）：装置 split_off_the_profile_options 认的四个参数进档；只选域与 N 的四个不进；别的认不出、判红（文件头「档」一段）
PROFILE_OPTIONS = ("--profile", "--marker-parity", "--table", "--disk-order")
PROFILE_OPTION_DEFAULTS = {"--marker-parity": "even", "--table": "direct", "--disk-order": "normal"}
SELECTION_OPTIONS = ("--bounds", "--family", "--only", "--max-n")
DEFAULT_PROFILE = "--profile r1"
PROFILE_FORM = re.compile(r"--profile r[1-9][0-9]*( --marker-parity odd)?( --table (?:scan|both))?( --disk-order reversed)?")
PROFILE_HOWTO = ("档取同一趟 kill-bounds 产物第一行 E7RESULT name=config 的 device_arguments=：装置钉住的红例逐行重放只认 --profile、--marker-parity、--table、--disk-order，"
                 "--bounds、--family、--only、--max-n 只选域与 N；别的参数先查装置（crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs 的 split_off_the_profile_options）"
                 "认不认它、进不进逐态行，进的加进这份脚本的 PROFILE_OPTIONS（不进的加进 SELECTION_OPTIONS）再钉，不手填 ㉒")
PROFILE_MISSING_HOWTO = ("档取同一趟 kill-bounds 产物第一行 E7RESULT name=config 的 device_arguments=，E165 第二段驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh 每趟都写；"
                         "产物缺了这一段是不认得的产物：重跑那一趟（驱动）得到新的账与产物再钉，不手填、不补")
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
BREAK = {item for item in os.environ.get("E165_PIN_RED_EXAMPLES_BREAK", "").split(",") if item}
SELFTEST_VARIABLE = "E165_PIN_RED_EXAMPLES_SELFTEST"   # --selftest 在自己的环境里设成 1，它起的子进程凭它认 E165_PIN_RED_EXAMPLES_BREAK
SAMPLE_ROOT_MARKERS = (".gate-cells", ".gate-break-sample")   # 门禁样本根的标记（.claude/gate.d/lib/stage-cells.sh 文件头）
# --replay 一块交给装置几行（--block-rows 改）：一块判完那几行当场落盘，被打断之后同一条命令只补没判的块
DEFAULT_BLOCK_ROWS = 32
# 门禁给的一份文件：--replay 判完往里追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（lib_verdict_library.Tally.append_to），门禁整道相加
ITEM_TALLY_FILE_VARIABLE = "FORMAL_ITEM_TALLY_FILE"
# 门禁给的另一份文件：每行判完（现判或复用）往里追加一行「项<TAB>这一行自己的挂钟秒数<TAB>现判条数<TAB>复用条数<TAB>退出码」，门禁收尾转成 ITEM_DONE 行
ITEM_DONE_FILE_VARIABLE = "FORMAL_ITEM_DONE_FILE"
# --list-items --key-mismatches 往 stderr 逐行打的前缀：「ITEM_KEY item=<项名> state=<键相同|键对不上|无记录|拒算>」（门禁 --list-items --with-keys 转成 ITEM_PLAN 行）
ITEM_KEY_PREFIX = "ITEM_KEY"
# --replay 编装置、跑装置时的进度行（文件头 --replay 一段）：间隔照 .claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」；时钟、轮询间隔与去处自证换成假的
PROGRESS_INTERVAL_SECONDS = 300
PROGRESS_POLL_SECONDS = 1.0
PROGRESS_CLOCK = time.monotonic
PROGRESS_SINK = None   # None 是 sys.stderr


class Red(Exception):
    def __init__(self, message, howto):
        super().__init__(message)
        self.message, self.howto = message, howto


class NotRun(Exception):
    pass


def report_red(message, howto):
    print(f"  ✗ {message}")
    print(f"     → 怎么办：{howto}")


def file_digest(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def state_before_outcome(line):
    """逐态行里 state= 的值（整行最后一段），截到 ;first= 之前；认不出交 None。"""
    match = re.search(r"(?:^| )state=(\S+)$", line)
    return match.group(1).split(";first=")[0] if match else None


def input_fingerprint(domain, size, original_line):
    state = state_before_outcome(original_line)
    if state is None:
        return None
    return hashlib.sha256(f"{domain}|{size}|{state}".encode("utf-8")).hexdigest()


# ── 读账、读表、换变异名 ──

def read_ledger(path):
    rows = []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if len(fields) != LEDGER_COLUMNS:
            raise Red(f"红例账 {path} 第 {number} 行是 {len(fields)} 段，要 {LEDGER_COLUMNS} 段", "账由 E165 第二段驱动写，格式在它的表头；手改坏了从 git 找回")
        rows.append({"number": number, "fields": fields})
    return rows


def mutation_names(root, relative):
    """变异表的 {物理行号: 变异名}（数据行）。"""
    path = os.path.join(root, relative)
    if not os.path.isfile(path):
        raise Red(f"变异表 {relative} 不在", "它随仓走；被删了从 git 找回")
    names = {}
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if raw.strip() and not raw.startswith("#"):
            names[number] = raw.split("\t")[0]
    return names


def read_kill_bounds(path):
    """第二段产物里每一行 E7RESULT name=kill_bound 的 item、table、table_row、red_examples、title（缺了记 None），
    与收尾行 E7RESULT name=verdict 的 ledger_sha256（没有收尾行记 None，有行而没有这一段记空串）。"""
    records, ledger_digest = [], None
    for line in open(path, encoding="utf-8").read().split("\n"):
        if line.startswith("E7RESULT name=verdict "):
            match = re.search(r" ledger_sha256=([0-9a-f]{64})(?: |$)", line)
            ledger_digest = match.group(1) if match else ""
        if not line.startswith("E7RESULT name=kill_bound "):
            continue
        record = {}
        for key in ("item", "table", "table_row", "red_examples", "title"):
            match = re.search(rf" {key}=(\S+)", line)
            record[key] = match.group(1) if match else None
        records.append(record)
    return records, ledger_digest


def profile_of_device_arguments(text):
    """device_arguments= 的值（参数用 + 连）→ 规范的档（文件头「档」一段）；值是空的、认不出 raise ValueError（说明哪一个）。
    开关 profile-read-as-r1 下一律交 --profile r1；profile-arguments-unchecked 下认不出的参数略过；product-profile-defaulted 下空值当 --profile r1。"""
    if "profile-read-as-r1" in BREAK:
        return DEFAULT_PROFILE
    tokens = [token for token in text.split("+") if token]
    if not tokens:
        if "product-profile-defaulted" in BREAK:
            return DEFAULT_PROFILE
        raise ValueError("值是空的")
    values, position = {}, 0
    while position < len(tokens):
        token = tokens[position]
        if token in PROFILE_OPTIONS or token in SELECTION_OPTIONS:
            if position + 1 >= len(tokens):
                raise ValueError(f"{token} 后面缺值")
            if token in PROFILE_OPTIONS:
                if token in values:
                    raise ValueError(f"{token} 给了不止一次")
                values[token] = tokens[position + 1]
            position += 2
            continue
        if "profile-arguments-unchecked" in BREAK:
            position += 1
            continue
        raise ValueError(f"认不出的参数 {token}（不知道它进不进钉住的红例逐行重放的逐态行）")
    parts = ["--profile", values.get("--profile", "r1")]
    for option in PROFILE_OPTIONS[1:]:
        if option in values and values[option] != PROFILE_OPTION_DEFAULTS[option]:
            parts += [option, values[option]]
    profile = " ".join(parts)
    if not PROFILE_FORM.fullmatch(profile):
        raise ValueError(f"档「{profile}」不是装置认的写法（{PROFILE_FORM.pattern}）")
    return profile


def run_profile(product):
    """同一趟 kill-bounds 产物 → 规范的档：第一行 E7RESULT name=config 的 device_arguments=；
    没有 config 行、config 行没有 device_arguments=、值是空的、认不出的，一律 raise Red（出路：重跑那一趟）。
    开关 product-profile-defaulted 下前三种当 --profile r1。"""
    name = os.path.basename(product)
    for line in open(product, encoding="utf-8").read().split("\n"):
        if line.startswith("E7RESULT name=config "):
            match = re.search(r" device_arguments=(\S*)", line)
            value = match.group(1) if match else ""
            if not value.strip("+") and "product-profile-defaulted" not in BREAK:
                raise Red(f"{name} 的 config 行{'没有 device_arguments=' if match is None else '的 device_arguments= 的值是空的'}，取不出这一趟的档", PROFILE_MISSING_HOWTO)
            try:
                return profile_of_device_arguments(value)
            except ValueError as problem:
                raise Red(f"{name} 的 device_arguments= 认不出档：{problem}", PROFILE_HOWTO)
    if "product-profile-defaulted" in BREAK:
        return DEFAULT_PROFILE
    raise Red(f"{name} 里没有 E7RESULT name=config 行，取不出这一趟的档", PROFILE_MISSING_HOWTO)


def row_profile(fields):
    """钉表一行的档（㉒）；调用方先过 table_problems（它拦住段数不是 22 的行）。"""
    return fields[PROFILE_INDEX]


def shown_path(path, root):
    """仓里的写相对仓根的路径，仓外的写绝对路径。"""
    relative = os.path.relpath(path, root)
    return os.path.abspath(path) if relative == ".." or relative.startswith("../") else relative


def paired_kill_bounds(ledger_path):
    """账 <目录>/<前缀>-red-examples.tsv 配同一趟的 <目录>/<前缀>-kill-bounds.out；文件名不是这个形态交 None。"""
    base = os.path.basename(ledger_path)
    if not base.endswith(LEDGER_SUFFIX):
        return None
    return os.path.join(os.path.dirname(ledger_path), base[:-len(LEDGER_SUFFIX)] + KILL_BOUNDS_SUFFIX)


def first_table_row(ledger_path, row):
    rows_text = row["fields"][1]
    first_row = rows_text.split("+")[0]
    if not first_row.isdigit():
        raise Red(f"{os.path.basename(ledger_path)}:{row['number']} 第 2 列「{rows_text}」认不出表行", "账的第 2 列是表行号（几行同一个改法用 + 连）")
    return int(first_row)


RUN_PAIRING_HOWTO = ("账与同一趟的产物配对：--kill-bounds 给了用给的那一份，不给按文件名配（<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out，"
                     "驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh 一趟写这一对，写完当场钉、当场核）："
                     "给错了就照驱动那一趟写的那一对重给；产物或账被删、被改了从 git 找回同一趟的原样（git log --all --diff-filter=D --name-only -- '*<文件名>*'，git show <提交>:<路径>）；"
                     "找不回的重跑那一趟再钉")


def run_variant_names(root, ledger_path, ledger_rows, product=None):
    """出这份账的那一趟给每个（变体, 表行）的（表, title）；配对不过 raise Red。开关 kill-bounds-optional 下产物不在交 None。
    product 不给就按文件名配（--check 只走这一支）。"""
    where = shown_path(ledger_path, root)
    product = product or paired_kill_bounds(ledger_path)
    if product is None or not os.path.isfile(product):
        if "kill-bounds-optional" in BREAK:
            return None
        shown = f"配的 kill-bounds 产物 {shown_path(product, root)} 不在" if product else f"文件名不以 {LEDGER_SUFFIX} 收尾，配不出 kill-bounds 产物"
        raise Red(f"红例账 {where} {shown}：认不出出这份账的那一趟给变体起的名字", RUN_PAIRING_HOWTO)
    records, ledger_digest = read_kill_bounds(product)
    if "ledger-digest-unchecked" not in BREAK:
        if ledger_digest is None:
            raise Red(f"{shown_path(product, root)} 没有收尾行 E7RESULT name=verdict：这一趟没跑完，配不上红例账 {where}",
                      "照驱动文件头：重跑同一条命令只补没跑的变体，26 个都落行才写收尾行（账的 sha256 记在那一行）；之后再判、再钉")
        actual = file_digest(ledger_path)
        if ledger_digest != actual:
            raise Red(f"{shown_path(product, root)} 收尾行记的账 sha256 是 {ledger_digest or '（没写）'}，红例账 {where} 此刻是 {actual}：账与产物不是同一趟",
                      RUN_PAIRING_HOWTO)
    if "variant-sets-unchecked" not in BREAK:
        ledger_counts = {}
        for row in ledger_rows:
            ledger_counts[row["fields"][0]] = ledger_counts.get(row["fields"][0], 0) + 1
        product_counts = {}
        for record in records:
            count = int(record["red_examples"]) if record["red_examples"] and record["red_examples"].isdigit() else 0
            product_counts.setdefault(record["item"], set()).add(count)
        differences = []
        for item in sorted(set(ledger_counts) | {item for item, counts in product_counts.items() if counts != {0}}):
            counts = product_counts.get(item)
            if counts is None:
                differences.append(f"{item}：账里 {ledger_counts[item]} 行，产物里没有这个变体")
            elif counts != {ledger_counts.get(item, 0)}:
                differences.append(f"{item}：账里 {ledger_counts.get(item, 0)} 行，产物的 red_examples= 是 {'/'.join(str(count) for count in sorted(counts))}")
        if differences:
            raise Red(f"红例账 {where} 与 {shown_path(product, root)} 的变体集合或行数不同（{len(differences)} 处：{'；'.join(differences[:5])}）：不是同一趟写的", RUN_PAIRING_HOWTO)
    names = {(record["item"], int(record["table_row"])): (record["table"], record["title"])
             for record in records if record["item"] and record["table_row"] and record["table_row"].isdigit()}
    if "title-missing-ignored" not in BREAK:
        for row in ledger_rows:
            key = (row["fields"][0], first_table_row(ledger_path, row))
            if names.get(key, (None, None))[1] is None:
                raise Red(f"{os.path.basename(ledger_path)}:{row['number']}：{shown_path(product, root)} 里没有 item={key[0]} table_row={key[1]} 那一行，或那一行没有 title=",
                          RUN_PAIRING_HOWTO)
    return names


class Unresolved(Exception):
    """追加给了 --kill-bounds 而这一行账按 title 找不到唯一的变异：逐行报、不钉这一行。"""
    pass


class VariantNames:
    def __init__(self, root):
        self.root, self.cache = root, {}

    def names_of(self, relative):
        if relative not in self.cache:
            self.cache[relative] = mutation_names(self.root, relative)
        return self.cache[relative]

    def variant(self, ledger_path, row, titles=None):
        """账的第 1、2 列换成「<表>#<名>」。titles（同一趟产物的 {(变体, 表行): (table=, title=)}，run_variant_names 交的）给了就按 title= 到今天的表里找名字，
        不给（--check 的开关那几支）就按今天的行号换。账写坏了 raise Red；按名字找不到或找到几行 raise Unresolved。"""
        item, rows_text = row["fields"][0], row["fields"][1]
        first_row = rows_text.split("+")[0]
        where = f"{os.path.basename(ledger_path)}:{row['number']}"
        if not first_row.isdigit():
            raise Red(f"{where} 第 2 列「{rows_text}」认不出表行", "账的第 2 列是表行号（几行同一个改法用 + 连）")
        line_number = int(first_row)
        match = re.fullmatch(r"tsv-(\d+)", item)
        if match:
            table = CRATES_TABLE
            if int(match.group(1)) != line_number:
                raise Red(f"{where}：变体 {item} 与第 2 列的表行 {rows_text} 对不上", "账写坏了；从 git 找回原样")
        elif re.fullmatch(r"Z\d+", item):
            table = CUSTOM_TABLE
        else:
            raise Red(f"{where}：变体「{item}」既不是 tsv-<行号> 也不是 Z<n>", "只认 E165 第二段驱动写的两种变体名")
        if titles is not None and "variant-by-line-number" not in BREAK:
            return self.variant_by_title(where, item, line_number, table, titles)
        name = self.names_of(table).get(line_number)
        if name is None or (table == CUSTOM_TABLE and not name.startswith(item + " ")):
            raise Red(f"{where}：{table} 第 {line_number} 行不是变体 {item} 的变异（读到 {name!r}）",
                      "表在跑完之后被改过、行号对不上：拿跑那一趟时的表（git show <那次提交>:<表>）核出变异名，或重跑那一趟再钉")
        if titles is not None and "titles-unchecked" not in BREAK:
            title = titles.get((item, line_number), (None, None))[1]
            if title is None or title != name.replace(" ", "_"):
                raise Red(f"{where}：变体 {item} 在第二段产物里的 title 是 {title!r}，今天的 {table} 第 {line_number} 行是 {name!r}",
                          "表在跑完之后被改过：拿跑那一趟时的表核出变异名，或重跑那一趟再钉")
        return f"{table}#{name}"

    def variant_by_title(self, where, item, line_number, table, titles):
        """出这份账的那一趟给这个变体的 title=，在今天的表里按名字找：第 1 列空格记成 _ 之后等于它的数据行恰好一行。"""
        table_of_run, title = titles.get((item, line_number), (None, None))
        if title is None:
            raise Unresolved(f"{where}：--kill-bounds 产物里没有 item={item} table_row={line_number} 那一行，或那一行没有 title=")
        if table_of_run != table:
            raise Unresolved(f"{where}：产物里变体 {item} 那一行的 table= 是 {table_of_run}，这种变体的表是 {table}")
        hits = [(number, name) for number, name in sorted(self.names_of(table).items()) if name.replace(" ", "_") == title]
        if not hits and "titles-unchecked" in BREAK and line_number in self.names_of(table):
            return f"{table}#{self.names_of(table)[line_number]}"
        if not hits:
            raise Unresolved(f"{where}：变体 {item} 那一趟的 title 是 {title!r}，今天的 {table} 里按名字找不到（第 1 列空格记成 _ 之后没有一行等于它）")
        if len(hits) > 1:
            raise Unresolved(f"{where}：变体 {item} 那一趟的 title 是 {title!r}，今天的 {table} 里找到 {len(hits)} 行同名（第 {'、'.join(str(number) for number, _name in hits)} 行），认不出是哪一个")
        return f"{table}#{hits[0][1]}"


UNRESOLVED_HOWTO = ("按名字找不到的：那一趟之后这条变异被删或改了名，被删的从 git 找回那一行（git log -S '<变异名>' -- <表>）再跑同一条命令，"
                    "改了名的交主 agent 定（在今天的变异表上重跑出这份账的那一趟再钉）；找到几行同名的：先把表里重名的变异改成不同的名字（变异名全表唯一），"
                    "再跑同一条命令；已钉进的不重复，不手填 ⑥")


def pinned_fields(ledger_path, row, variant, profile):
    fields = row["fields"]
    domain, size, ordinal, original, mutant = fields[2], fields[3], fields[4], fields[11], fields[12]
    fingerprint = input_fingerprint(domain, size, original)
    if fingerprint is None:
        raise Red(f"{os.path.basename(ledger_path)}:{row['number']} 第 12 列认不出 state=", "账的第 12 列是原实现的逐态行（E7RESULT name=state …），手改坏了从 git 找回")
    return [fingerprint, domain, size, ordinal, f"{os.path.basename(ledger_path)}:{row['number']}", variant, original, mutant, *fields, profile]


def copied_ledger_row(fields):
    """钉表一行 ⑨–㉑ 用制表符接起来：整行抄的那一行账（㉒ 档不算在里面）。"""
    return "\t".join(fields[TABLE_OWN_COLUMNS:DEVICE_COLUMNS])


def first_differing_column(fields, ledger_fields):
    """钉表一行 ⑨–㉑ 与账那一行第一处不同的段，写成「⑱（账第 10 段）」；段数不同写段数。"""
    copied = fields[TABLE_OWN_COLUMNS:DEVICE_COLUMNS]
    if len(copied) != len(ledger_fields):
        return f"⑨ 起是 {len(copied)} 段、账那一行是 {len(ledger_fields)} 段"
    for index, (pinned_value, ledger_value) in enumerate(zip(copied, ledger_fields)):
        if pinned_value != ledger_value:
            return f"{COLUMN_MARKS[TABLE_OWN_COLUMNS + index]}（账第 {index + 1} 段）"
    return "（逐字节相同）"


def has_example(row):
    return row["fields"][4] != "none" and row["fields"][11] != ""


def read_table(path):
    if not os.path.isfile(path):
        raise Red(f"钉住的表 {path} 不在", "它随仓走，表头写着格式；被删了从 git 找回")
    text = open(path, encoding="utf-8").read()
    rows = []
    for number, raw in enumerate(text.split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if len(fields) != TABLE_COLUMNS and "row-columns-unchecked" in BREAK:   # 弄坏开关：段数不是 22 的行当表里没有这一行
            continue
        rows.append({"number": number, "fields": fields})
    return text, rows


def table_problems(rows):
    problems, seen = [], set()
    for row in rows:
        fields = row["fields"]
        if len(fields) != TABLE_COLUMNS:
            problems.append(f"第 {row['number']} 行是 {len(fields)} 段，要 {TABLE_COLUMNS} 段（自己的 {TABLE_OWN_COLUMNS} 段后面接红例账一行的 {LEDGER_COLUMNS} 段，再接 ㉒ 档）")
            continue
        if not PROFILE_FORM.fullmatch(fields[PROFILE_INDEX]) and "profile-form-unchecked" not in BREAK:
            problems.append(f"第 {row['number']} 行的 ㉒「{fields[PROFILE_INDEX]}」不是规范写法的档（{PROFILE_FORM.pattern}）")
        if not re.fullmatch(r"[0-9a-f]{64}", fields[0]):
            problems.append(f"第 {row['number']} 行的 ① 不是 64 位小写十六进制")
        elif input_fingerprint(fields[1], fields[2], fields[6]) != fields[0] and "fingerprint-unchecked" not in BREAK:
            problems.append(f"第 {row['number']} 行的 ① 与拿 ②③⑦ 重算的指纹不同（手改过这一行）")
        if "copied-columns-unchecked" not in BREAK:
            for ledger_column, own_column in COPIED_PAIRS:
                copied_index = TABLE_OWN_COLUMNS + ledger_column - 1
                if fields[copied_index] != fields[own_column - 1]:
                    problems.append(f"第 {row['number']} 行：{COLUMN_MARKS[copied_index]}（红例账第 {ledger_column} 段）与 {COLUMN_MARKS[own_column - 1]} 不同")
        key = (fields[5], copied_ledger_row(fields), row_profile(fields))
        if key in seen:
            problems.append(f"第 {row['number']} 行的 ⑥、⑨–㉑ 的整行与 ㉒ 都与前面一行相同（重复钉了）")
        seen.add(key)
    return problems


def write_table(table, new_text, digest_at_read):
    """同目录临时文件写完、改名换上之前复核读表时的 sha256（变了就不写）；不成 raise Red。"""

    def unchanged_since_read():
        if "no-recheck-before-write" in BREAK:
            return
        if current_digest(table) != digest_at_read:
            raise ChangedSinceRead()

    try:
        replace_file_contents_by_rename(table, new_text, check_before_rename=unchanged_since_read)
    except ChangedSinceRead:
        raise Red(f"读到写之间 {table} 被别人改过，一行都没写", "别的会话刚写过这张表：重跑同一条命令")
    except ReplaceRefused as refused:
        raise Red(f"没换上：{str(refused).splitlines()[0]}", str(refused).splitlines()[-1] if len(str(refused).splitlines()) > 1 else "看上面的原因")


# ── 追加 ──

def run_names_for_append(root, ledgers, products):
    """追加换名用的那一趟的名字，与 --check 同一套：每份账配同一趟的产物（run_variant_names 的四样），
    products 给了（--kill-bounds，与账逐份对应，第二段驱动收尾时这样调）用给的那一份，不给按文件名配；
    有一份配不上就 raise Red，一行都不钉。交（{账: {(变体, 表行): (table=, title=)} 或 None（只在开关下）}, {账: 档}）：
    档取同一趟产物的 device_arguments=（文件头「档」一段），认不出 raise Red；开关 append-pairing-optional 下配不上的当 --profile r1。"""
    names, profiles = {}, {}
    for index, ledger in enumerate(ledgers):
        product = (products[index] if products else None) or paired_kill_bounds(ledger)
        try:
            names[ledger] = run_variant_names(root, ledger, read_ledger(ledger), products[index] if products else None)
        except Red as red:
            if "append-pairing-optional" not in BREAK:
                raise Red(f"{red.message}（追加按出这份账的那一趟的名字换名，配不上就一行都没钉）", red.howto)
            names[ledger] = None
        if "append-names-from-today-table" in BREAK:
            names[ledger] = None
        if product and os.path.isfile(product):
            try:
                profiles[ledger] = run_profile(product)
            except Red as red:
                raise Red(f"{red.message}（追加按出这份账的那一趟的档钉，认不出就一行都没钉）", red.howto)
        else:
            profiles[ledger] = DEFAULT_PROFILE
    return names, profiles


def append(root, table, ledgers, products):
    text, existing = read_table(table)
    problems = table_problems(existing)
    if problems:
        raise Red(f"钉住的表 {table} 本身不对：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    digest_at_read = file_digest(table)
    run_names, profiles = run_names_for_append(root, ledgers, products)
    # 冲突与去重都按档分开：同一个（①, ⑥）在不同档下各钉一行（文件头「档」一段）
    by_key = {}
    for row in existing:
        by_key.setdefault((row["fields"][0], row["fields"][5], row_profile(row["fields"])), row["fields"])
    by_variant = {(row["fields"][5], copied_ledger_row(row["fields"]), row_profile(row["fields"])) for row in existing}
    by_origin = {(row["fields"][4], copied_ledger_row(row["fields"]), row_profile(row["fields"])) for row in existing}
    names = VariantNames(root)
    new_rows, already, without_example, conflicts, total, unresolved = [], 0, 0, [], 0, []
    for ledger in ledgers:
        profile = profiles[ledger]
        for row in read_ledger(ledger):
            total += 1
            if not has_example(row):
                without_example += 1
                continue
            try:
                variant = names.variant(ledger, row, run_names[ledger])
            except Unresolved as problem:
                unresolved.append(str(problem))
                continue
            fields = pinned_fields(ledger, row, variant, profile)
            key, whole = (fields[0], fields[5], profile), copied_ledger_row(fields)
            if "pin-twice" not in BREAK:
                if key in by_key and by_key[key][6] != fields[6] and "same-profile-conflict-ignored" not in BREAK:
                    conflicts.append(f"{fields[4]}（{fields[5]}，档 {profile}）：表里 ⑦ 是 {by_key[key][6][:120]}…，账里是 {fields[6][:120]}…")
                    continue
                if ((fields[5], whole, profile) in by_variant or (fields[4], whole, profile) in by_origin
                        or ("dedupe-by-input-variant" in BREAK and key in by_key)):
                    already += 1
                    continue
            new_rows.append(fields)
            by_key.setdefault(key, fields)
            by_variant.add((fields[5], whole, profile))
            by_origin.add((fields[4], whole, profile))
    for problem in unresolved:
        print(f"     没钉：{problem}")  # gate-lint:detail
    unresolved_note = f"；{len(unresolved)} 行按 title 找不到唯一的变异、没钉" if unresolved else ""
    if conflicts:
        for conflict in conflicts[:5]:
            print(f"     {conflict}")  # gate-lint:detail
        raise Red(f"{len(conflicts)} 行账与表里（或这一次前面的账里）同一档下同一个（输入, 变异）的原实现逐态行不同，一行都没钉",
                  "同一档下原实现在这个输入上的行为变了：先弄清是实现改了行为（要三方认）还是账跑在别的树上，再决定改表还是重跑")
    if not new_rows:
        if unresolved:
            raise Red(f"{len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态{unresolved_note}，一行都没新钉", UNRESOLVED_HOWTO)
        print(f"  ✓ {len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态，没有要新钉的；表里 {len(existing)} 行")
        return EXIT_GREEN
    new_text = (text if text.endswith("\n") or not text else text + "\n") + "".join("\t".join(fields) + "\n" for fields in new_rows)
    write_table(table, new_text, digest_at_read)
    _text, after = read_table(table)
    present = {"\t".join(row["fields"]) for row in after}
    lost = [fields for fields in new_rows if "\t".join(fields) not in present]
    if lost:
        raise Red(f"写完回读：新钉的 {len(new_rows)} 行里 {len(lost)} 行不在表里", "别的会话在改名换上之后又写过；重跑同一条命令")
    if unresolved:
        raise Red(f"钉进 {len(new_rows)} 行（回读都在；{len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态）{unresolved_note}",
                  UNRESOLVED_HOWTO)
    print(f"  ✓ 钉进 {len(new_rows)} 行（{len(ledgers)} 份账 {total} 行：{already} 行已钉、{without_example} 行没有反例状态），回读都在；表里现有 {len(after)} 行")
    return EXIT_GREEN


class ChangedSinceRead(Exception):
    pass


def current_digest(path):
    return file_digest(path)


# ── 钉全：点名的账每一行都钉着（--check）──

def check(root, table, ledgers, products):
    """点名的每份账（products 与它逐份对应，是同一趟的 kill-bounds 产物；空就按文件名配）里有反例状态的每一行都钉在表里（文件头 --check 一段）。
    不扫任何目录：账由调用方给（main 拦了不给账的那一种）。开关 check-globs-by-default 下不给账照改前的判法取 LEGACY_LEDGER_GLOB；
    check-kill-bounds-ignored 下给了 products 也不用、照旧按文件名配。"""
    if not ledgers and "check-globs-by-default" in BREAK:
        ledgers = sorted(glob.glob(os.path.join(root, LEGACY_LEDGER_GLOB)))
        if not ledgers:
            raise NotRun(f"{LEGACY_LEDGER_GLOB} 一份红例账都没有：这一格无对象可判")
    explicit = {} if "check-kill-bounds-ignored" in BREAK else dict(zip(ledgers, products))
    _text, rows = read_table(table)
    problems = table_problems(rows)
    if problems:
        raise Red(f"钉住的表本身不对：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    by_fingerprint = {}
    for row in rows:
        by_fingerprint.setdefault(row["fields"][0], []).append(row)
    names = VariantNames(root)
    missing, counted, without_example, paired, whole_row_differs, profile_differs = [], 0, 0, 0, 0, 0
    ledger_profiles = {}
    for ledger in ledgers:
        ledger_rows = read_ledger(ledger)
        run_names = run_variant_names(root, ledger, ledger_rows, explicit.get(ledger))
        if run_names is not None:
            paired += 1
        if "names-from-today-table" in BREAK:
            run_names = None
        # 按档判钉全：这份账的档取同一趟产物的 device_arguments=（产物不在只在开关 kill-bounds-optional 下走到这里，那时不比档）
        product = explicit.get(ledger) or paired_kill_bounds(ledger)
        profile = run_profile(product) if product and os.path.isfile(product) else None
        ledger_profiles[profile or "（档认不出）"] = ledger_profiles.get(profile or "（档认不出）", 0) + 1
        for row in ledger_rows:
            if not has_example(row):
                without_example += 1
                continue
            counted += 1
            origin = f"{os.path.basename(ledger)}:{row['number']}"
            fingerprint = input_fingerprint(row["fields"][2], row["fields"][3], row["fields"][11])
            same_input = [pinned_row for pinned_row in by_fingerprint.get(fingerprint, []) if pinned_row["fields"][6] == row["fields"][11]]
            if profile is not None and "check-ignores-profile" not in BREAK:
                other_profile = [pinned_row for pinned_row in same_input if row_profile(pinned_row["fields"]) != profile
                                 and copied_ledger_row(pinned_row["fields"]) == "\t".join(row["fields"])]
                same_input = [pinned_row for pinned_row in same_input if row_profile(pinned_row["fields"]) == profile]
                if other_profile and not same_input:
                    profile_differs += 1
                    missing.append((ledger, f"{origin}（钉的档不对：表第 {other_profile[0]['number']} 行整行与账这一行相同，㉒ 是「{row_profile(other_profile[0]['fields'])}」，"
                                            f"这份账那一趟是「{profile}」）"))
                    continue
            if "whole-row-unchecked" in BREAK:
                candidates = [pinned_row["fields"] for pinned_row in same_input]
            else:
                # 整行抄：钉着的那一行 ⑨–㉑ 用制表符接起来要与账这一行逐字节相同
                candidates = [pinned_row["fields"] for pinned_row in same_input if copied_ledger_row(pinned_row["fields"]) == "\t".join(row["fields"])]
                if same_input and not candidates:
                    whole_row_differs += 1
                    # 点名最像的那一行：⑨⑩（变体与表行）与账这一行相同的优先
                    nearest = next((pinned_row for pinned_row in same_input if pinned_row["fields"][8:10] == row["fields"][0:2]), same_input[0])
                    missing.append((ledger, f"{origin}（没整行钉：表第 {nearest['number']} 行钉着同一个输入与 ⑦，⑨ 起与账这一行不同，"
                                            f"第一处在 {first_differing_column(nearest['fields'], row['fields'])}）"))
                    continue
            pinned = any(fields[4] == origin for fields in candidates)
            if not pinned and candidates:
                table_of_run, title = (run_names or {}).get((row["fields"][0], first_table_row(ledger, row)), (None, None))
                if title is not None:
                    # 出这份账的那一趟的名字：⑥ 的表等于产物的 table=、变异名空格记成 _ 之后等于产物的 title=
                    pinned = "title-not-compared" in BREAK or any(
                        fields[5].partition("#")[0] == table_of_run and fields[5].partition("#")[2].replace(" ", "_") == title for fields in candidates)
                else:
                    # 只在开关下走到这里（产物不在、没有 title=，或不认产物）：照今天的变异表换名
                    try:
                        variant = names.variant(ledger, row)
                    except Red:
                        variant = None
                    pinned = any(fields[5] == variant for fields in candidates)
            if not pinned:
                missing.append((ledger, origin))
    if missing and "check-ignores-missing" not in BREAK:
        for _ledger, origin in missing[:10]:
            print(f"     没钉：{origin}")  # gate-lint:detail
        commands = [f"python3 scripts/experiment-apparatus/e165-pin-red-examples.py {shown_path(ledger, root)}"
                    + (f" --kill-bounds {shown_path(explicit[ledger], root)}" if ledger in explicit else "")
                    for ledger in sorted({ledger for ledger, _origin in missing})]
        whole_note = f"（其中 {whole_row_differs} 行表里钉着同一个输入，但没有一行 ⑨ 起与账那一行逐字节相同）" if whole_row_differs else ""
        if profile_differs:
            whole_note += f"（其中 {profile_differs} 行整行钉着、㉒ 不是这份账那一趟的档）"
        raise Red(f"{len(ledgers)} 份红例账 {counted} 行里 {len(missing)} 行没钉进 {os.path.relpath(table, root)}{whole_note}",
                  f"补进去，一份账一条：{'；'.join(commands)}（已钉的不重复，⑨ 起不同的另钉一行整行抄；追加照这一次的配法配同一趟的产物、按它的 title= 到今天的变异表里按名字找变异，按名字找不到或找到几行的逐行报、不钉，照它的出路办；档取同一趟产物的 device_arguments=，同一个输入与变异在别的档下钉着的照钉一行这一档的）")
    profiles_text = "、".join(f"{profile} {count} 份" for profile, count in sorted(ledger_profiles.items()))
    print(f"  ✓ {len(ledgers)} 份红例账 {counted} 行都钉在 {os.path.relpath(table, root)}，钉着的那一行 ⑨–㉑ 与账那一行逐字节相同、㉒ 是那份账的档（{without_example} 行没有反例状态、不钉；"
          f"{paired} 份账与同一趟的 kill-bounds 产物配对都过，⑥ 按那一趟的名字认；账的档：{profiles_text}）；表里 {len(rows)} 行，① 都与 ②③⑦ 重算的相同，⑪⑫⑬⑳㉑ 与 ②③④⑦⑧ 相同")
    return EXIT_GREEN


# ── 钉表自己：门禁 pinned-red-examples-table 格（--check-table）──

def missing_variants(root, rows):
    """钉表每行 ⑥ 点名的变异（<表>#<名>，几个用 + 连）不在它的表里（第 1 列等于那个名字的数据行）的，逐处交「第 N 行：<表>#<名>」。
    --check-table 与 --replay 共用这一份。开关 variant-unchecked 下一律交空。"""
    if "variant-unchecked" in BREAK:
        return []
    missing, tables = [], {}
    for row in rows:
        for part in row["fields"][5].split("+"):
            relative, _hash, name = part.partition("#")
            if relative not in tables:
                path = os.path.join(root, relative)
                tables[relative] = set(mutation_names(root, relative).values()) if os.path.isfile(path) else None
            if tables[relative] is None or name not in tables[relative]:
                missing.append(f"第 {row['number']} 行：{part}")
    return missing


def raise_missing_variants(rows, missing):
    """逐处列出不在了的变异，再 raise Red（--check-table 与 --replay 共用）。"""
    for item in missing[:10]:
        print(f"     不在了：{item}")  # gate-lint:detail
    raise Red(f"{len(rows)} 行里 {len(missing)} 处点名的「改回去就红」的变异不在它的表里了",
              "变异行被删或改了名：找回那一行（改法被悄悄撤回时它要先响），确是改名就照新名改这张表（要写明为什么）")


def check_table(root, table):
    """门禁 pinned-red-examples-table 格：只判钉表自己（table_problems）与 ⑥ 点名的变异还在（missing_variants），不读红例账、kill-bounds 产物与 research/ 下的任何文件。
    表一行数据都没有 raise NotRun。开关 table-check-globs-research 下照改前那一格的判法：取 LEGACY_LEDGER_GLOB 的账、按文件名配产物判钉全。"""
    if "table-check-globs-research" in BREAK:
        ledgers = sorted(glob.glob(os.path.join(root, LEGACY_LEDGER_GLOB)))
        if not ledgers:
            raise NotRun(f"{LEGACY_LEDGER_GLOB} 一份红例账都没有：这一格无对象可判")
        return check(root, table, ledgers, [])
    _text, rows = read_table(table)
    problems = table_problems(rows)
    if problems:
        raise Red("钉住的表本身不对：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    if not rows:
        raise NotRun(f"{os.path.relpath(table, root)} 一行数据都没有：这一格无对象可判")
    missing = missing_variants(root, rows)
    if missing:
        raise_missing_variants(rows, missing)
    variants = sum(len(row["fields"][5].split("+")) for row in rows)
    by_profile = {}
    for row in rows:
        profile = row_profile(row["fields"])
        by_profile[profile] = by_profile.get(profile, 0) + 1
    profile_text = "、".join(f"{profile} {count} 行" for profile, count in sorted(by_profile.items()))
    print(f"  ✓ 钉表 {os.path.relpath(table, root)} {len(rows)} 行：每行 {TABLE_COLUMNS} 段，① 都与 ②③⑦ 重算的相同，⑪⑫⑬⑳㉑ 与 ②③④⑦⑧ 相同，"
          f"㉒ 都是规范写法的档（{profile_text}），没有重复钉的；⑥ 点名的 {variants} 处变异都还在它们的表里；没读红例账与 kill-bounds 产物")
    return EXIT_GREEN


# ── 格二：变异还在、今天的实现上绿 ──

def run_reporting_progress(command, cwd, environment, what, rows_total=None):
    """起 command，stdout、stderr 收进内存（判定照旧跑完再看）；跑着的时候每过 PROGRESS_INTERVAL_SECONDS 秒（按 PROGRESS_CLOCK）
    往 PROGRESS_SINK（默认 stderr）打一行「E165_PINNED_PROGRESS 挂钟 <秒> 秒：<what> 还在跑[；装置已打 n/m 行结果]」。交 CompletedProcess。"""
    process = subprocess.Popen(command, cwd=cwd, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    received = {"stdout": [], "stderr": []}

    def drain(stream, key):
        for line in stream:
            received[key].append(line)

    readers = [threading.Thread(target=drain, args=(process.stdout, "stdout"), daemon=True),
               threading.Thread(target=drain, args=(process.stderr, "stderr"), daemon=True)]
    for reader in readers:
        reader.start()
    sink = PROGRESS_SINK if PROGRESS_SINK is not None else sys.stderr
    started = last_line = PROGRESS_CLOCK()
    while True:
        try:
            process.wait(timeout=PROGRESS_POLL_SECONDS)
            break
        except subprocess.TimeoutExpired:
            pass
        now = PROGRESS_CLOCK()
        if now - last_line >= PROGRESS_INTERVAL_SECONDS and "progress-silent" not in BREAK:
            counted = ""
            if rows_total is not None:
                counted = f"；装置已打 {sum(1 for line in list(received['stdout']) if line.startswith(RESULT_PREFIX))}/{rows_total} 行结果"
            print(f"E165_PINNED_PROGRESS 挂钟 {now - started:.0f} 秒：{what} 还在跑{counted}", file=sink, flush=True)
            last_line = now
    for reader in readers:
        reader.join()
    return subprocess.CompletedProcess(command, process.returncode, "".join(received["stdout"]), "".join(received["stderr"]))


class UsageError(Exception):
    """--item 点名了没有的项：退 2，列出可点名的项。"""

    def __init__(self, message, available):
        super().__init__(message)
        self.message, self.available = message, available


def pinned_item(row):
    return f"pinned:{row['number']}"


def item_key_state(store, root, row, shared):
    """钉表一行此刻的键在库里是哪一种（只读库、不写库、不编装置）：键相同（会复用）、键对不上（这一行判绿过，键变了）、无记录、拒算（键罩不全，每趟现判）。
    和 replay 判这一行时算的键同一份（row_components）；段数不是 22 的行不进库，算无记录。"""
    if shared is None or len(row["fields"]) != TABLE_COLUMNS:
        return "拒算" if shared is None else "无记录"
    components = row_components(root, row, shared)
    if refused_component(components, store.judging_units) is not None:
        return "拒算"
    if "key-mismatch-by-item-only" in BREAK:   # 弄坏开关：这一行判绿过就算键相同，不比键
        return "键相同" if store.item_record(pinned_item(row)) is not None else "无记录"
    if store.key_exists(key_of(components, store.judging_units)):
        return "键相同"
    return "键对不上" if store.item_record(pinned_item(row)) is not None else "无记录"


def list_items(root, table, verdict_library=None, binary=None, items=None):
    """逐行打「项<TAB>格[<TAB>说明]」：table（pinned-red-examples-table）一项，钉表每个数据行一项 pinned:<表里的行号>（说明是 ⑤、⑥ 与「档 <㉒>」）。只读钉表，不编、不起装置。
    给了 verdict_library（命令行的 --key-mismatches 取库放在哪那一份）另往 stderr 逐行打「ITEM_KEY item=pinned:<行号> state=<键相同|键对不上|无记录|拒算>」
    （item_key_state；table 那一项每趟现判、不按条入库，不打）。只读库，不写库。弄坏开关 item-key-lines-skipped 下不打。"""
    _text, rows = read_table(table)
    print("table\tpinned-red-examples-table")
    for row in rows:
        fields = row["fields"]
        note = f"{fields[4]} {fields[5]} 档 {fields[PROFILE_INDEX]}" if len(fields) == TABLE_COLUMNS else f"{len(fields)} 段"
        print(f"{pinned_item(row)}\tpinned-red-examples-replay\t{note}")
    # 给了 items（--item，可几个）只比点名的行（pinned:<行号>），一行都没点到就连装置那几样的键都不算；成本随点名行数走。弄坏开关 key-items-ignored 下照旧比整表
    keyed_rows = [row for row in rows if not items or pinned_item(row) in items or "key-items-ignored" in BREAK]
    if verdict_library and keyed_rows and "item-key-lines-skipped" not in BREAK:
        store = VerdictLibrary(verdict_library, "pinned-row", judging_units())
        try:
            shared = device_components(root, binary)
        except Red:
            shared = None
        for row in keyed_rows:
            print(f"{ITEM_KEY_PREFIX} item={pinned_item(row)} state={item_key_state(store, root, row, shared)}", file=sys.stderr)
    return EXIT_GREEN


# 自证造「判法单元指纹变了」用：里面放一个值时 judging_units 交它（只有自证放，门禁与命令行从不放）
JUDGING_UNITS_FOR_SELFTEST = []
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
# 判法单元：重放这一格判结局的函数（连同它调到的；判定库与 lib_function_units.py 两份不进）
JUDGING_ENTRY = "replay"
# 装置的构建配置（--replay 不给 --binary 时编装置的那一条），进装置单元闭包那一样的工具链行
DEVICE_BUILD_LINE = f"build: cargo build --release --locked -p singlefs-verification --bin {DEVICE_BINARY}"


_JUDGING_UNITS = []


def judging_units():
    """判法单元指纹：这份脚本里 replay（JUDGING_ENTRY）的单元闭包键，判定库的另一列，不进输入指纹；一个进程里算一次。
    弄坏开关 reuse-key-without-judging-units 下一律交空串。"""
    if "reuse-key-without-judging-units" in BREAK:
        return ""
    if JUDGING_UNITS_FOR_SELFTEST:
        return JUDGING_UNITS_FOR_SELFTEST[0]
    if not _JUDGING_UNITS:
        _JUDGING_UNITS.append(judging_unit_key_of(REPOSITORY_ROOT, os.path.relpath(os.path.realpath(__file__), REPOSITORY_ROOT)))
    return _JUDGING_UNITS[0]


def judging_unit_key_of(root, relative):
    """根 root 里 relative 那份脚本的判法单元指纹（UnitKey）：JUDGING_ENTRY 的单元闭包键，判定库与 lib_function_units.py 不进；算不出交一条拒算。
    弄坏开关：judging-units-raw-text 下取那份脚本整份原文的摘要（改注释也变）；spawn-inside-judging-closure 下另把起装置那一下（run_reporting_progress）
    算进判法入口（起 --binary 给的程序那一段回到闭包里，库折不出那条命令、判法单元指纹拒算）。"""
    if "judging-units-raw-text" in BREAK:
        return UnitKey(digest_file(os.path.join(root, relative)), (), ())
    entries = [f"{relative}::{JUDGING_ENTRY}"] + ([f"{relative}::run_reporting_progress"] if "spawn-inside-judging-closure" in BREAK else [])
    try:
        return judging_unit_key(root, entries)
    except ValueError as error:
        return UnitKey(None, (), (f"refused\tjudging-units\t{os.path.basename(relative)}:1\t{error}",))


def device_components(root, binary):
    """装置那几样：装置 e165_replay_bounded_enumeration 的 main 起的单元闭包键（UnitKey，lib_function_units.py 的切法；工具链行是 cargo -V、rustc -V
    与编装置那一条的构建配置，依赖行是 Cargo.lock 里 singlefs-verification 传递引到的外部包），钉表每一行都带它；给了 --binary 的另加「装置替身」：那份程序的内容摘要
    （样本与自证的桩），这时闭包算不出就记一句算不出的原因。不给 --binary 而闭包算不出：抛 Red（键罩不全就不许复用）。闭包里有拒算的照交（那几行判定不复用、每趟现判）。
    装置的身份只在这里进键：起装置那一下不在判法单元指纹（replay 的闭包）里（文件头「键」一段）。
    弄坏开关：reuse-key-without-closure 下单元闭包那一样不进；stand-in-unkeyed 下装置替身那一样不进。"""
    components = [("装置替身", digest_file(binary))] if binary and "stand-in-unkeyed" not in BREAK else []
    if "reuse-key-without-closure" in BREAK:
        return components
    try:
        units = FunctionUnitIndex(root).key([f"{DEVICE_SOURCE}::main"], toolchain=rust_toolchain_lines(root) + [DEVICE_BUILD_LINE],
                                            dependencies=locked_dependency_lines(root, "singlefs-verification"))
    except ValueError as error:
        if not binary:
            raise Red(f"算不出装置 {DEVICE_BINARY} 的 main 起的单元闭包（{error}），按条判定的键罩不全",
                      f"先让 python3 scripts/gate/lib_function_units.py closure <根> {DEVICE_SOURCE}::main 在这棵树上跑通；判定库的键要罩全装置单元闭包，没有不读不写的形态")
        return components + [("装置单元闭包", digest_text(f"算不出闭包：{error}"))]
    return components + [("装置单元闭包", units)]


def mutation_text(root, variant):
    """⑥ 点名的每一条变异（<表>#<名>，几个用 + 连）在它的表里那几行的原文与替换文（第 3、4 段；第 1 列等于名字的数据行，几行都取）；
    变异名与表的路径不进（只用来找那一行；弄坏开关 row-key-with-mutation-name 下进）。表不在、行不在各记一句。"""
    pieces = []
    for part in variant.split("+"):
        relative, _hash, name = part.partition("#")
        path = os.path.join(root, relative)
        head = f"{part}\t" if "row-key-with-mutation-name" in BREAK else ""
        if not os.path.isfile(path):
            pieces.append(f"{head}表不在")
            continue
        lines = ["\t".join(raw.split("\t")[2:4]) for raw in open(path, encoding="utf-8").read().split("\n")
                 if raw.strip() and not raw.startswith("#") and raw.split("\t")[0] == name]
        pieces.append(head + ("\n".join(lines) if lines else "行不在"))
    return "\n".join(pieces)


def row_components(root, row, shared):
    """钉表一行的键：装置那几样（shared），加这一行的状态输入（①②③④：指纹、域、N、枚举序号）、期望逐态行（⑦）、档（㉒）、⑥ 引的变异的原文与替换文。
    出处 ⑤（红例账路径与行号）、变异名、⑧ 与抄来的账行 ⑨–㉑ 不进：重放只读 ②③④⑦ 与档，改账路径、换变异名、账里别的段不改它的判定。
    弄坏开关：row-key-with-ledger-path 下 ⑤ 进；reuse-key-without-row 下状态输入与期望逐态行不进；reuse-key-without-profile 下档不进；
    reuse-key-without-mutation-row 下变异的原文与替换文不进。"""
    components = list(shared)
    fields = row["fields"]
    if "reuse-key-without-row" not in BREAK:
        components.append(("状态输入", digest_text("\t".join(fields[0:4]))))
        components.append(("期望逐态行", digest_text(fields[6])))
    if "row-key-with-ledger-path" in BREAK:
        components.append(("出处", digest_text(fields[4])))
    if "reuse-key-without-profile" not in BREAK:
        components.append(("档", digest_text(row_profile(fields))))
    if "reuse-key-without-mutation-row" not in BREAK:
        components.append(("变异原文与替换文", digest_text(mutation_text(root, fields[5]))))
    return components


def lookup(store, item, components, forced_judge=False):
    """forced_judge 只给自证用（main 的 selftest_forced_judge）：同一把键、库里已有判绿记录时照样现判，造「记红撤绿」那一格；门禁与命令行从不设。"""
    if forced_judge:
        return False, "强制现判（自证：同键现判）"
    if "reuse-ignores-key" in BREAK and store.directory is not None:
        return store.lookup_by_item_only(item)
    return store.lookup(item, components)


def record_item_done(item, seconds, judged, reused, exit_code):
    """环境里有 FORMAL_ITEM_DONE_FILE 的（门禁给的）往那份文件追加一行「项<TAB>秒<TAB>现判条数<TAB>复用条数<TAB>退出码」，一次 write 写完一整行。
    弄坏开关 item-done-unrecorded 下不写。"""
    path = os.environ.get(ITEM_DONE_FILE_VARIABLE)
    if not path or "item-done-unrecorded" in BREAK:
        return
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(f"{item}\t{seconds:.1f}\t{judged}\t{reused}\t{exit_code}\n")


def report_item(item, verdict, how, seconds=None):
    """每项一行：项名、判定（绿 / 红）、现判还是复用、现判的用了几秒（一块里几行共用一次装置，秒数是那一块的）。同时记一行 ITEM_DONE 的原料（record_item_done）：
    复用的 0 秒、复用 1 条；现判的取这一块的秒数（装置一次起几行，没有比一块更细的挂钟）、现判 1 条；退出码绿 0、红 1。"""
    print(f"  · {item}：{verdict}，{how}" + (f"，{seconds:.1f} 秒（这一块）" if seconds is not None else ""))  # gate-lint:detail
    reused = how.startswith("复用")
    record_item_done(item, 0.0 if reused else (seconds or 0.0), 0 if reused else 1, 1 if reused else 0, 0 if verdict == "绿" else 1)


def blocked_table(table_text, block_numbers):
    """只留 block_numbers 那几个数据行的表：别的数据行换成单独一个 #（装置当注释跳过），行号与原表相同，装置报「第 N 行」照原表的行号；
    留下的数据行去掉 ㉒ 档、只留 ①–㉑（装置 --pinned-cases 只认 21 段，档由命令行带进去）。"""
    lines = table_text.split("\n")
    kept = []
    for number, raw in enumerate(lines, 1):
        if raw.strip() and not raw.startswith("#") and number not in block_numbers:
            kept.append("#")
        elif raw.strip() and not raw.startswith("#"):
            kept.append("\t".join(raw.split("\t")[:DEVICE_COLUMNS]))
        else:
            kept.append(raw)
    return "\n".join(kept)


def run_device_block(root, table_text, block, binary, options, work, profile):
    """一块（同一档的几行）：只留这几行的表交给装置 --pinned-cases，后面接这一档的参数（㉒ 按空格切开），交 CompletedProcess。
    被信号停了抛 Red（这一块之前判完的已经落盘）。开关 replay-without-profile 下不带档参数。"""
    block_table = os.path.join(work, f"pinned-block-{block[0]['number']}.tsv")
    with open(block_table, "w", encoding="utf-8") as handle:
        handle.write(blocked_table(table_text, {row["number"] for row in block}))
    # 以 .sh 结尾的（样本的桩）经 bash 起：样本目录里的文件不带执行位（script-modes.sh 不看 fixtures/）
    profile_arguments = [] if "replay-without-profile" in BREAK else profile.split(" ")
    command = (["bash", binary] if binary.endswith(".sh") else [binary]) + [DEVICE_FLAG, block_table, *profile_arguments]
    environment = dict(os.environ)
    threads_value = str(options["threads"])
    for name in (
        "CARGO_BUILD_JOBS",
        "RUST_TEST_THREADS",
        "SINGLEFS_CRASH_POINT_REPLAY_THREADS",
        "SINGLEFS_CRASH_INJECTION_THREADS",
        "SINGLEFS_FAULT_INJECTION_WORKER_THREADS",
        "SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS",
        "SINGLEFS_RANDOM_HISTORY_THREADS",
        "E161_THREADS",
        "SINGLEFS_CRASH_JUDGE_CPU_THREADS",
        "SINGLEFS_CRASH_AMPLIFICATION_THREADS",
        "E165_THREADS",
        "E168_THREADS",
        "E175_WORKER_THREADS",
        "E180_THREADS",
        "E182_THREADS",
        "E183_THREADS",
        "E184_THREADS",
        "E185_THREADS",
        "GATE_MUTATION_KEY_WORKERS",
        "SINGLEFS_THREAD_CAP",
    ):
        environment[name] = threads_value
    # 起装置那一下经 options["spawn"]（main 给的 run_reporting_progress）：起的是 --binary 给的程序或编出来的装置，它的身份由装置那几样进键
    # （device_components：装置单元闭包、装置替身），不在判法单元指纹（replay 的闭包）里
    completed = options["spawn"](command, root, environment, f"装置 {DEVICE_FLAG} 逐行重放钉住的 {len(block)} 行（档 {profile}，第 {block[0]['number']} 行起这一块）", len(block))
    signal_number = -completed.returncode if completed.returncode < 0 else completed.returncode - 128 if 128 < completed.returncode <= 128 + 64 else None
    if signal_number is not None and "killed-as-regression" not in BREAK:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())
        raise Red(f"装置被信号停了（退 {completed.returncode}，信号 {signal_number}）：钉住的 {len(block)} 行这一次没判完，输出不算判定{'（' + tail[:300] + '）' if tail else ''}",
                  "查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录；整条命令撞了内存上限由 task-server 放大重排，不在这里判），"
                  "那件事不再发生之后重跑这一格；不改钉住的表、变异行或实现")
    return completed


RESULT_IDENTITY_FIELDS = ("domain", "n", "ordinal")   # 逐态行里认行身份的三段，依次对表行的 ②③④


def result_identity(line):
    """装置打的一行结果是哪个输入的：逐态行里头一次出现的 domain=、n=、ordinal= 三段（state= 在最后，前面这三段不会被它冒充）；三段不全交 None。"""
    values = {}
    for token in line.split(" "):
        key, equals, value = token.partition("=")
        if equals and key in RESULT_IDENTITY_FIELDS and key not in values:
            values[key] = value
    return tuple(values[key] for key in RESULT_IDENTITY_FIELDS) if len(values) == len(RESULT_IDENTITY_FIELDS) else None


def pair_result_lines(results, rows):
    """装置打的结果行按行身份配到块里的登记行，交与 rows 等长的列表（配不上的是 None）。身份是逐态行的 domain=、n=、ordinal=，对表行的 ②③④。
    同一个身份块里有几行，装置就该给它打几行：打的行数与块里的行数相同才按次序配（同一个身份、同一个档重放的是同一个状态）；
    不同就这个身份的几行都不配（分不出少打的是哪一行）。认不出身份的结果行不配给任何一行。弄坏开关 results-paired-by-position：按下标配（改前的样子）。"""
    if "results-paired-by-position" in BREAK:
        return [results[index] if index < len(results) else None for index in range(len(rows))]
    lines_by_identity = {}
    for line in results:
        lines_by_identity.setdefault(result_identity(line), []).append(line)
    indexes_by_identity = {}
    for index, row in enumerate(rows):
        indexes_by_identity.setdefault(tuple(row["fields"][1:4]), []).append(index)
    paired = [None] * len(rows)
    for identity, indexes in indexes_by_identity.items():
        lines = lines_by_identity.get(identity, [])
        if len(lines) == len(indexes):
            for index, line in zip(indexes, lines):
                paired[index] = line
    return paired


def replay(root, table, options):
    text, rows = read_table(table)
    problems = table_problems(rows)
    if problems:
        raise Red("钉住的表本身不对：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    if not rows:
        raise NotRun(f"{os.path.relpath(table, root)} 一行都没有：这一格无对象可判")
    available = [pinned_item(row) for row in rows]
    wanted = options.get("items") or []
    if wanted and "item-filter-ignored" not in BREAK:
        unknown = [item for item in wanted if item not in available]
        if unknown:
            raise UsageError(f"pinned-red-examples-replay 格里没有点名的项：{'、'.join(unknown)}", available)
        rows = [row for row in rows if pinned_item(row) in wanted]
    # 变异还在：与 --check-table 共用 missing_variants
    absent_variants = missing_variants(root, rows)
    if absent_variants:
        raise_missing_variants(rows, absent_variants)
    store, tally = VerdictLibrary(options.get("verdict_library"), "pinned-row", judging_units()), Tally()
    binary = options["binary"]
    shared = device_components(root, binary) if store.directory else []
    pending = []
    for row in rows:
        components = row_components(root, row, shared) if shared else []
        reused, reason = lookup(store, pinned_item(row), components, options.get("forced_judge", False))
        tally.count(reused, reason)
        if reused:
            report_item(pinned_item(row), "绿", f"复用（{reason}）")
        else:
            detail = store.detail(pinned_item(row))
            pending.append((row, components, reason + (f"：{detail}" if detail else "")))
    named = f"（--item 点名的 {len(rows)} 行）" if wanted else ""
    if not pending:
        print(f"  · 按条：{tally.text()}")  # gate-lint:detail
        tally.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), "pinned-red-examples-replay")
        print(f"  ✓ 钉住的 {len(rows)} 行{named}：点名的变异都还在它们的表里；每一行都复用上一次判绿的判定，没编装置、没起重放；按条{tally.text()}")
        return EXIT_GREEN
    if binary is None:
        environment = dict(os.environ, CARGO_TARGET_DIR=options["target"])
        command = ["nice", "-n", "19", "cargo", "build", "--release", "--locked", "-p", "singlefs-verification", "--bin", DEVICE_BINARY]
        completed = options["spawn"](command, root, environment, f"编装置 {DEVICE_BINARY}")
        if completed.returncode != 0:
            raise Red(f"编装置没成（{' '.join(command[3:])} 退 {completed.returncode}）：{' | '.join(completed.stderr.strip().split(chr(10))[-4:])}",
                      "先让它在平时的工具链上编得过；缓存缺包 cargo fetch --locked")
        binary = os.path.join(options["target"], "release", DEVICE_BINARY)
    block_rows = max(1, options.get("block_rows") or DEFAULT_BLOCK_ROWS)
    # 按档分组（㉒ 相同的一组，组的次序照第一次出现），每组再按块起装置、每块带这一组的档参数
    groups = {}
    for entry in pending:
        groups.setdefault(row_profile(entry[0]["fields"]), []).append(entry)
    blocks = [(profile, entries[start:start + block_rows]) for profile, entries in groups.items() for start in range(0, len(entries), block_rows)]
    exits, results_count, mismatches, stderr_tails, held = [], 0, [], [], []
    with tempfile.TemporaryDirectory(prefix="e165-pinned-blocks-") as work:
        for profile, block in blocks:
            started = time.monotonic()
            completed = run_device_block(root, text, [row for row, _components, _reason in block], binary, options, work, profile)
            seconds = time.monotonic() - started
            results = [line for line in completed.stdout.split("\n") if line.startswith(RESULT_PREFIX)]
            results_count += len(results)
            exits.append(completed.returncode)
            if completed.stderr.strip():
                stderr_tails.append(completed.stderr.strip().split("\n")[-1][:200])
            block_clean = completed.returncode == 0 and len(results) == len(block)
            paired_lines = pair_result_lines(results, [row for row, _components, _reason in block])
            for index, (row, components, reason) in enumerate(block):
                got_line = paired_lines[index]
                matched = got_line == row["fields"][6] or "replay-ignores-output" in BREAK
                named_in_errors = f"第 {row['number']} 行（" in completed.stderr
                # 这一行的判定只看它自己：按行身份配上的那一行与 ⑦ 相同、装置的报错里没点它的行号（同一块里别的行红不连累它，点名单跑与整格里跑判定相同）；
                # 落盘只认配上了自己那一行结果的：块不干净（装置退非 0、行数对不上）时配得上、判绿的照键记绿，配不上的不记
                green = matched and not named_in_errors
                recordable = green and (got_line is not None or "replay-ignores-output" in BREAK)
                if "unclean-block-records-nothing" in BREAK:   # 弄坏开关：不干净的块一行都不记（改前的样子）
                    recordable = green and (block_clean or "replay-ignores-output" in BREAK)
                if not matched:
                    mismatches.append(f"第 {row['number']} 行（{row['fields'][4]}，档 {profile}）：要 {row['fields'][6][:160]}，装置打的是 {(got_line or '（没打或配不上）')[:160]}")
                report_item(pinned_item(row), "绿" if green else "红", f"现判（{reason}）", seconds)
                # 记红、撤绿（文件头「复用」，用户 2026-10-29 定案）：这一行判红而装置真给它出了结果（打了一行、或报错里点了它的行号），照同一把键调共用库的 record_red
                # 记红，盖掉原有的绿，下一趟这一行现判；装置没给它出结果的（这一块半路停了、行数不够）不碰库。弄坏开关 red-keeps-green：不记（改前的样子）
                if not green and (got_line is not None or named_in_errors) and store.directory and "red-keeps-green" not in BREAK and "reuse-records-red" not in BREAK:
                    failed_record = store.record_red(pinned_item(row), components, (f"第 {row['number']} 行装置打的结果与 ⑦ 不同" if not matched
                                                                                     else f"装置的报错点了第 {row['number']} 行"),
                                                     run=f"{DEVICE_BINARY} {DEVICE_FLAG} <块表> {profile}（这一块 {len(block)} 行）")
                    if failed_record:
                        print(f"  ! {failed_record}：下一趟这一行照判")
                if recordable or "reuse-records-red" in BREAK:
                    if "reuse-records-whole-cell-only" in BREAK:
                        held.append((pinned_item(row), components))
                    else:
                        failed_record = store.record(pinned_item(row), components, "装置逐行打的结果与 ⑦ 逐字节相同",
                                                     run=f"{DEVICE_BINARY} {DEVICE_FLAG} <块表> {profile}（这一块 {len(block)} 行）")
                        if failed_record:
                            print(f"  ! {failed_record}：下一趟这一行照判")
    print(f"  · 按条：{tally.text()}")  # gate-lint:detail
    tally.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), "pinned-red-examples-replay")
    failed_exit = next((code for code in exits if code != 0), 0)
    judged_rows = len(pending)
    if failed_exit != 0 or mismatches or (results_count != judged_rows and "replay-ignores-output" not in BREAK):
        for mismatch in mismatches[:5]:
            print(f"     {mismatch}")  # gate-lint:detail
        raise Red(f"钉住的 {judged_rows} 行（现判的）在今天的实现上：装置退 {failed_exit}、打了 {results_count} 行结果、{len(mismatches)} 行与 ⑦ 不同"
                  f"{'（' + stderr_tails[-1] + '）' if stderr_tails else ''}",
                  "今天的实现在钉住的输入上与原实现不同：是改动引入的回归就修实现；装置报指纹漂了就先查装置的枚举次序与状态编码变没变")
    for item, components in held:
        store.record(item, components, "装置逐行打的结果与 ⑦ 逐字节相同", run=f"{DEVICE_BINARY} {DEVICE_FLAG} <块表>（整格判完一起记：弄坏开关 reuse-records-whole-cell-only）")
    print(f"  ✓ 钉住的 {len(rows)} 行{named}：点名的变异都还在它们的表里；装置 {DEVICE_FLAG} 逐行打的结果都与 ⑦ 逐字节相同"
          f"（按档 {len(groups)} 组：{'、'.join(f'{profile} {len(entries)} 行' for profile, entries in groups.items())}；{len(blocks)} 块、每块至多 {block_rows} 行，每块带它那一组的档）；按条{tally.text()}")
    return EXIT_GREEN


# ── 命令行 ──

def default_threads(environment=os.environ):
    """--threads 的缺省：SINGLEFS_THREAD_CAP 设了取它，否则登记的重活默认并行度（.claude/heavy-thread-default）。"""
    if "threads-default-cpu-count" in BREAK:   # 弄坏开关：改前的样子
        return int(environment.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1)
    return default_heavy_threads(environment)


# 库位置的判法（judged_library_directory 与 main 里取库位置那一段）不写模块级的赋值、不带缺省值：
# import 时执行的模块级语句进 replay 的判法单元闭包（scripts/gate/lib_function_units.py 文件头「import 时执行的模块级语句」），库位置不是判法

def judged_library_directory(root, environment):
    """被判那棵仓的判定库目录：scripts/gate/verus-proof-check.py library-directory <根> 打的那一份（与门禁 formal-verification 的证明同一份库），
    照问它、不另抄一份目录名；取不到（不是 git 工作树）、问不成交 None。
    问的时候去掉 Verus 那一份的弄坏开关（FORMAL_VERIFICATION_BREAK、VERUS_PROOF_CHECK_BREAK）：要的是真位置。"""
    asked = {name: value for name, value in environment.items() if name not in ("FORMAL_VERIFICATION_BREAK", "VERUS_PROOF_CHECK_BREAK")}
    try:
        answered = subprocess.run([sys.executable, os.path.join(SCRIPT_DIRECTORY, "..", "gate", "verus-proof-check.py"), "library-directory", root],
                                  capture_output=True, text=True, env=asked, stdin=subprocess.DEVNULL)
    except OSError:
        return None
    directory = answered.stdout.strip()
    return directory if answered.returncode == EXIT_GREEN and directory else None


def main(arguments, selftest_library=None, selftest_forced_judge=False):
    """命令行入口（文件头「用法」）。selftest_library 只给自证在进程里调时用：--replay 与 --list-items 那一趟的库目录换成它（各格各自一份临时库），命令行不收；
    selftest_forced_judge 同样只给自证：--replay 的每一行不读库、照样现判（同一把键、库里已有判绿记录时造「记红撤绿」），命令行不收。"""
    if arguments[:1] == ["--selftest"]:
        return selftest()
    options = {"root": DEFAULT_ROOT, "table": None, "kill_bounds": [], "binary": None, "threads": default_threads(),
               "target": None,   # 没给 --target：replay_in_own_target 这一趟自己建、跑完删（文件头 --replay 一段）
               "items": [], "verdict_library": None, "forced_judge": selftest_forced_judge, "key_mismatches": False, "block_rows": DEFAULT_BLOCK_ROWS,
               # 编装置与起装置的那一下：不在 replay 里点名，判法单元指纹（replay 的闭包）因此不收起命令行给的程序那一段（文件头「键」一段）
               "spawn": run_reporting_progress}
    mode, ledgers, position = "append", [], 0
    valued = {"--root": "root", "--table": "table", "--binary": "binary", "--threads": "threads", "--target": "target", "--block-rows": "block_rows"}
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("--check", "--check-table", "--replay", "--list-items"):
            mode = argument[2:]
            position += 1
        elif argument == "--key-mismatches":
            options["key_mismatches"] = True
            position += 1
        elif argument in valued or argument in ("--kill-bounds", "--item"):
            if position + 1 >= len(arguments) or (argument == "--item" and not arguments[position + 1]):
                report_red(f"{argument} 后面缺值", "看文件头的用法一节；可点名的项：e165-pin-red-examples.py --list-items")
                return EXIT_USAGE
            if argument == "--kill-bounds":
                options["kill_bounds"].append(arguments[position + 1])
            elif argument == "--item":
                options["items"].append(arguments[position + 1])
            else:
                try:
                    options[valued[argument]] = int(arguments[position + 1]) if argument in ("--threads", "--block-rows") else arguments[position + 1]
                except ValueError:
                    report_red(f"{argument} 要整数，读到 {arguments[position + 1]!r}", "看文件头的用法一节")
                    return EXIT_USAGE
            position += 2
        elif argument.startswith("--"):
            report_red(f"认不出的选项 {argument}", "看文件头的用法一节")
            return EXIT_USAGE
        else:
            ledgers.append(arguments[position])
            position += 1
    root = os.path.abspath(options["root"])
    table = os.path.abspath(options["table"] or os.path.join(root, TABLE))
    if options["key_mismatches"] and mode != "list-items":
        report_red(f"--key-mismatches 只给 --list-items 用（--{mode} 不收）", "去掉 --key-mismatches；要看每一行的键对不对得上：e165-pin-red-examples.py --list-items --key-mismatches")
        return EXIT_USAGE
    if mode == "replay" or (mode == "list-items" and options["key_mismatches"]):
        options["verdict_library"] = selftest_library or judged_library_directory(root, os.environ)
        if not options["verdict_library"]:
            report_red(f"{root} 不是 git 工作树（python3 scripts/gate/verus-proof-check.py library-directory 取不到判定库的位置）：一行都没判",
                       "在 git 工作树里跑；rsync 拷的仓副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）")
            return EXIT_USAGE
    if options["kill_bounds"] and mode not in ("append", "check"):
        report_red(f"--{mode} 不收 --kill-bounds：只有追加与 --check 给每份账配同一趟的产物，--check-table 与 --replay 用不上产物",
                   "去掉 --kill-bounds 再跑")
        return EXIT_USAGE
    if options["kill_bounds"] and len(options["kill_bounds"]) != len(ledgers):
        report_red(f"{'追加' if mode == 'append' else '--check '}给了 {len(options['kill_bounds'])} 个 --kill-bounds、{len(ledgers)} 份红例账：给就逐份对应，第 i 个产物配第 i 份账",
                   "不给 --kill-bounds（按文件名配：<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out），或每份账给一个同一趟的产物")
        return EXIT_USAGE
    if options["items"] and mode not in ("replay", "list-items"):
        report_red(f"--item 只给 --replay 用（--{mode} 不收）：pinned-red-examples-table 只有一项（table）",
                   "去掉 --item；可点名的项：python3 scripts/experiment-apparatus/e165-pin-red-examples.py --list-items")
        return EXIT_USAGE
    if mode == "check" and not ledgers and "check-globs-by-default" not in BREAK:
        report_red(f"{CHECK_NEEDS_LEDGERS}：不扫任何目录，位置参数给要判的红例账，--kill-bounds 逐份给同一趟的产物（不给就按文件名配同一目录的 <前缀>-kill-bounds.out）",
                   "例：python3 scripts/experiment-apparatus/e165-pin-red-examples.py --check --kill-bounds <那一趟>-kill-bounds.out <那一趟>-red-examples.tsv；"
                   "判钉表自己（门禁 pinned-red-examples-table 格）用 --check-table")
        return EXIT_USAGE
    if mode == "check-table" and ledgers:
        report_red("--check-table 不收红例账：它只判钉表自己与点名的变异，不读账与产物", "去掉位置参数再跑；要判某一份账钉全了没有用 --check <账> --kill-bounds <同一趟的产物>")
        return EXIT_USAGE
    if mode == "list-items":
        if ledgers:
            report_red("--list-items 不收红例账", "去掉位置参数再跑")
            return EXIT_USAGE
        if "list-items-pipe-unhandled" not in BREAK:
            # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
            signal.signal(signal.SIGPIPE, signal.SIG_DFL)
        try:
            return list_items(root, table, options["verdict_library"], options["binary"], options["items"])
        except Red as red:
            report_red(red.message, red.howto)
            return EXIT_RED
    if mode == "append" and not ledgers:
        report_red("追加要至少一份红例账", "例：python3 scripts/experiment-apparatus/e165-pin-red-examples.py research/results/<某一趟>-red-examples.tsv")
        return EXIT_USAGE
    missing = [ledger for ledger in ledgers + options["kill_bounds"] if not os.path.isfile(ledger)]
    if missing:
        report_red(f"点名的文件不在：{' '.join(missing)}", "路径相对当前目录；红例账在 research/results/ 下")
        return EXIT_USAGE
    try:
        if mode == "append":
            return append(root, table, [os.path.abspath(ledger) for ledger in ledgers], [os.path.abspath(product) for product in options["kill_bounds"]])
        if mode == "check":
            return check(root, table, [os.path.abspath(ledger) for ledger in ledgers], [os.path.abspath(product) for product in options["kill_bounds"]])
        if mode == "check-table":
            return check_table(root, table)
        if options["target"] is None:
            return replay_in_own_target(root, table, options)
        return replay(root, table, options)
    except UsageError as usage:
        report_red(usage.message, "照下面列的项名写（e165-pin-red-examples.py --list-items 也列），给几次 --item 取并集")
        for item in usage.available:
            print(f"       {item}")  # gate-lint:detail
        return EXIT_USAGE
    except Red as red:
        report_red(red.message, red.howto)
        return EXIT_RED
    except NotRun as not_run:
        print(f"  ! {not_run}")
        return EXIT_NOT_RUN


def replay_in_own_target(root, table, options):
    """--replay 没给 --target：在 $TMPDIR 下建这一趟自己的编译目录交给 replay，跑完（连同 replay 抛出来的红、本次未跑）删掉，
    删之前核本机没有进程开着里面的文件（scripts/hosts/run_build_directory.py）；没删成打 ✗ 与进程，退出码改成红。"""
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts"))
    import run_build_directory
    owned = tempfile.mkdtemp(prefix="singlefs-e165-pinned-target-")
    exit_code = EXIT_RED
    try:
        exit_code = replay(root, table, dict(options, target=owned))
    finally:
        removed, lines = run_build_directory.remove_run_build_directory(owned)
        if not removed:
            print("\n".join(lines))
    return exit_code if removed else (exit_code or EXIT_RED)


# ── 自证 ──

# 自证的开关轮几趟同时起（每趟各用各的临时目录、桩装置不起 cargo）
SELFTEST_BREAK_WORKERS = 8


def selftest():
    global current_digest, PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK
    # 判定库那几格（--replay 的按条复用）经 lib_verdict_library 写进 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py
    #   文件头「命令行」--with-temporary-service），不是就一格都不跑、退 2，不写真服务。客户端照判定库的取法载入（在函数里取：模块级的语句进判法单元闭包）
    client = sys.modules["lib_verdict_library"].prune_server_client()
    service_problem = client.selftest_service_problem() if client is not None else "这棵树里没有 scripts/prune-server/prune_server_client.py"
    if service_problem:
        print(f"  ✗ e165-pin-red-examples.py 自证没跑：{service_problem}")
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY if client is not None else '在带着 scripts/prune-server/ 的仓里跑'}")
        return EXIT_USAGE
    os.environ[SELFTEST_VARIABLE] = "1"   # 自证起的子进程（--list-items 接管道那一格、逐个开关的 --selftest）凭它认弄坏开关
    cases, wrong = 0, []
    import contextlib
    import io
    with tempfile.TemporaryDirectory(prefix="e165-pin-selftest-") as work:
        root = os.path.join(work, "root")
        for directory in (".claude/gate.d", "crates", "scripts/mutation/tables", "research/results"):
            os.makedirs(os.path.join(root, directory))

        def write(relative, text):
            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                handle.write(text)

        crates_table = "# 表头\n红的一条\tcrates/x.rs\ta\tb\t-p x\tt\n红的两条\tcrates/x.rs\tc\td\t-p x\tt\n"
        write(CRATES_TABLE, crates_table)
        write(CUSTOM_TABLE, "# 表头\nZ1 自造的\tcrates/x.rs\te\tf\n")
        header = "# 自证用的钉住的表\n"
        write(TABLE, header)

        def state_line(domain, ordinal, state, first):
            return f"E7RESULT name=state domain={domain} ring_slots=64 n=2 ordinal={ordinal} counters=states state={state};first={first}"

        ledger_rows = [
            ["tsv-2", "2", "L", "2", "3", "counters", "+J4", "true", "true", "true", "cmd", state_line("L", 3, "shape=1c0", "returned:a"), state_line("L", 3, "shape=1c0", "returned:b")],
            ["tsv-3", "3+9", "L", "2", "3", "counters", "+J4", "true", "true", "true", "cmd", state_line("L", 3, "shape=1c0", "returned:a"), state_line("L", 3, "shape=1c0", "returned:c")],
            ["Z1", "2", "X", "2", "5", "outcome", "outcome", "true", "true", "true", "cmd", state_line("X", 5, "shape=2c0", "returned:d"), state_line("X", 5, "shape=2c0", "returned:e")],
            ["tsv-3", "3+9", "X", "2", "none", "no_single_state_differs", "none", "false", "false", "false", "", "", ""],
        ]
        ledger = os.path.join(root, "research/results/e165-selftest-red-examples.tsv")
        write(os.path.relpath(ledger, root), "# 账头\n" + "".join("\t".join(fields) + "\n" for fields in ledger_rows))
        kill_bounds = os.path.join(root, "research/results/e165-selftest-kill-bounds.out")

        def run_product(ledger_path, device_arguments="--profile+r1"):
            """同一趟的产物：每个变体一行（red_examples= 等于账里它的行数），收尾行记账的 sha256；头上写一行 config（档从 device_arguments 取，缺省 --profile r1）；
            device_arguments 给 None 不写 config 行。"""
            config = f"E7RESULT name=config experiment=E165 segment=2 device_arguments={device_arguments} bounds=segment2\n" if device_arguments is not None else ""
            return (config + "E7RESULT name=kill_bound item=baseline table=none table_row=none outcome=baseline title=基线 complete=yes\n"
                    "E7RESULT name=kill_bound item=tsv-2 table=crates/mutations.tsv table_row=2 outcome=red red_examples=1 title=红的一条 complete=yes\n"
                    "E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=3 outcome=red red_examples=2 title=红的两条 complete=yes\n"
                    "E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=9 outcome=red red_examples=2 title=同一处的另一行 complete=yes\n"
                    f"E7RESULT name=kill_bound item=Z1 table={CUSTOM_TABLE} table_row=2 outcome=red red_examples=1 title=Z1_自造的 complete=yes\n"
                    f"E7RESULT name=verdict scope=segment2 variants=4 red=3 ledger_sha256={file_digest(ledger_path)}\n"
                    "E7RESULT name=done emitted=7\n")

        good_titles = run_product(ledger)
        write(os.path.relpath(kill_bounds, root), good_titles)
        stub = os.path.join(work, "stub-device.sh")
        with open(stub, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 桩装置：--pinned-cases <表> 逐个数据行打 ⑦；STUB_MODE=wrong 把最后一行打错，=fail 退 1，=killed 打一行之后 kill -9 自己，=exit-137 打一行之后退 137（被 bash 报成 128 + 9），"
                         "=drift-x 打全、报错点 X 域那几行的行号、退 1，=skip-first 不打头一个数据行、报错点它的行号、退 1\n"
                         "[[ \"$1\" == --pinned-cases ]] || exit 2\n"
                         "[[ -n \"${STUB_TRACE:-}\" ]] && echo 起了一次 >> \"$STUB_TRACE\"\n"
                         "if [[ -n \"${STUB_KILL_DOMAIN:-}\" ]] && grep -v '^#' \"$2\" | grep -q \"domain=$STUB_KILL_DOMAIN \"; then kill -9 \"$$\"; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == wrong-x ]]; then grep -v '^#' \"$2\" | grep . | cut -f7 | sed '/domain=X / s/returned:/returned:WRONG/'; "
                         "echo \"E7RESULT name=pinned_cases rows=$(grep -v '^#' \"$2\" | grep -c .)\"; exit 0; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == killed ]]; then grep -v '^#' \"$2\" | grep . | head -n 1 | cut -f7; kill -9 \"$$\"; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == exit-137 ]]; then grep -v '^#' \"$2\" | grep . | head -n 1 | cut -f7; exit 137; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == slow ]]; then grep -v '^#' \"$2\" | grep . | head -n 1 | cut -f7; sleep 0.6; grep -v '^#' \"$2\" | grep . | tail -n +2 | cut -f7; "
                         "echo \"E7RESULT name=pinned_cases rows=$(grep -v '^#' \"$2\" | grep -c .)\"; exit 0; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == drift-x ]]; then grep -v '^#' \"$2\" | grep . | cut -f7; "
                         "awk -F'\\t' '!/^#/ && NF && $2 == \"X\" { print \"  ✗ 第 \" NR \" 行（\" $2 \":\" $3 \":\" $4 \"）：拿这个状态重算的指纹是 0，表里 ① 是 \" $1 }' \"$2\" >&2; exit 1; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == skip-first ]]; then grep -v '^#' \"$2\" | grep . | tail -n +2 | cut -f7; "
                         "awk -F'\\t' '!/^#/ && NF { print \"  ✗ 第 \" NR \" 行（\" $2 \":\" $3 \":\" $4 \"）：这个 (域, N) 只有 0 个状态，序号不在里面\"; exit }' \"$2\" >&2; exit 1; fi\n"
                         "rows=$(grep -v '^#' \"$2\" | grep -c .)\n"
                         "grep -v '^#' \"$2\" | grep . | cut -f7 | { if [[ \"${STUB_MODE:-}\" == wrong ]]; then sed '$ s/returned:/returned:WRONG/'; else cat; fi; }\n"
                         "echo \"E7RESULT name=pinned_cases rows=$rows\"\n"
                         "[[ \"${STUB_MODE:-}\" == fail ]] && exit 1\nexit 0\n")
        os.chmod(stub, 0o755)

        def expect(label, arguments, want_exit, want_text, environment=None):
            nonlocal cases
            cases += 1
            saved = dict(os.environ)
            os.environ.update(environment or {})
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    # --replay 那一趟各格各给一份新的空库（判定库没有不读不写的形态；临时服务底下各占一个命名空间）
                    got = main(["--root", root, *arguments], selftest_library=os.path.join(work, f"expect-library-{cases}"))
            finally:
                os.environ.clear()
                os.environ.update(saved)
            output = buffer.getvalue()
            if got != want_exit or want_text not in output:
                wrong.append(label)
                print(f"    {label} 这一格判错：期望退 {want_exit}、输出含「{want_text}」，实测退 {got}：{output.strip()[-300:]}")  # gate-lint:detail
            return output

        table_path = os.path.join(root, TABLE)
        expect("check-empty-table", ["--check", ledger], 1, "3 行里 3 行没钉进")
        write(os.path.relpath(kill_bounds, root), good_titles.replace("title=红的两条", "title=别的变异"))
        expect("titles-mismatch", [ledger], 1, "title 是")
        write(os.path.relpath(kill_bounds, root), good_titles)
        write(TABLE, header)
        expect("pin-kill-bounds-count", ["--kill-bounds", kill_bounds, ledger, ledger], 2, "追加给了 1 个 --kill-bounds、2 份红例账")
        # 按 title 找名字：那一趟之后变异表头上插了一行，tsv-2、tsv-3 的行号都错一位，名字照样找得到、钉进 3 行（开关 variant-by-line-number 下一行都不钉）
        write(CRATES_TABLE, crates_table.replace("# 表头\n", "# 表头\n插在前面的一条\tcrates/x.rs\tm\tn\t-p x\tt\n"))
        expect("pin-by-title-shifted-table", [ledger], 0, "钉进 3 行")
        by_title_text = open(table_path, encoding="utf-8").read()
        expect("check-by-title-shifted-table", ["--check", ledger], 0, "3 行都钉在")
        write(TABLE, header)
        # 按名字找不到：tsv-3 的变异在那一趟之后改了名，这一行逐行报、不钉，另外两行照钉，退 1（开关 titles-unchecked 下照行号钉进 3 行）
        write(CRATES_TABLE, crates_table.replace("红的两条", "改了名的两条"))
        not_found = expect("pin-by-title-not-found", [ledger], 1, "钉进 2 行（回读都在；1 份账 4 行：0 行已钉、1 行没有反例状态）；1 行按 title 找不到唯一的变异、没钉")
        not_found_table = open(table_path, encoding="utf-8").read()
        if ("e165-selftest-red-examples.tsv:3：变体 tsv-3 那一趟的 title 是 '红的两条'" not in not_found or "按名字找不到" not in not_found
                or "改了名的两条" in not_found_table or "#红的两条" in not_found_table):
            wrong.append("pin-by-title-not-found-detail")
            print(f"    pin-by-title-not-found-detail 这一格判错：要逐行点名第 3 行、按名字找不到，表里不许有 tsv-3 那一行，实测：{not_found.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        write(TABLE, header)
        # 找到两行同名：变异表末尾多了一行也叫「红的两条」，认不出是哪一个，逐行报、不钉（开关 variant-by-line-number 下照行号钉进 3 行）
        write(CRATES_TABLE, crates_table + "红的两条\tcrates/y.rs\ti\tj\t-p y\tt\n")
        ambiguous = expect("pin-by-title-ambiguous", [ledger], 1, "1 行按 title 找不到唯一的变异、没钉")
        if "找到 2 行同名（第 3、4 行）" not in ambiguous or "#红的两条" in open(table_path, encoding="utf-8").read():
            wrong.append("pin-by-title-ambiguous-detail")
            print(f"    pin-by-title-ambiguous-detail 这一格判错：要点名找到 2 行同名（第 3、4 行），表里不许有 tsv-3 那一行，实测：{ambiguous.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        write(CRATES_TABLE, crates_table)
        write(TABLE, header)
        # 读到写之间被改过：让复核那一次读到不同的 sha256
        original_digest = current_digest
        current_digest = lambda path: "0" * 64
        expect("race", [ledger], 1, "被别人改过")
        current_digest = original_digest
        if open(table_path, encoding="utf-8").read() != header:
            wrong.append("race-wrote")
            print("    race-wrote 这一格判错：复核判红之后表还是被写了")  # gate-lint:detail
        cases += 1
        expect("pin", [ledger], 0, "钉进 3 行")
        if open(table_path, encoding="utf-8").read() != by_title_text:
            wrong.append("pin-by-title-same-names")
            print("    pin-by-title-same-names 这一格判错：行号错位时按 title 钉出的表应当与表没改时钉出的逐字节相同")  # gate-lint:detail
        cases += 1
        expect("pin-again", [ledger], 0, "3 行已钉")
        expect("check-green", ["--check", ledger], 0, "3 行都钉在")
        # --check 不扫目录：不给账退 2（开关 check-globs-by-default 下照改前取 research/results 下的账，这里退 0）
        expect("check-without-ledgers", ["--check"], 2, CHECK_NEEDS_LEDGERS)
        pinned_text = open(table_path, encoding="utf-8").read()
        write(TABLE, "\n".join(line for line in pinned_text.split("\n") if "\tZ1 " not in line and "#Z1" not in line) + "\n")
        expect("check-missing", ["--check", ledger], 1, "3 行里 1 行没钉进")
        # ⑦ 与抄来的 ⑳ 一起改：⑳ 与 ⑦ 仍相同，只剩重算的指纹对不上（开关 fingerprint-unchecked 下这一格判错）
        if pinned_text.count("shape=2c0;first=returned:d\tE7") != 2:
            wrong.append("check-tampered-setup")
            print("    check-tampered-setup 这一格判错：Z1 那一行的 ⑦ 与 ⑳ 应当各有一处「shape=2c0;first=returned:d」")  # gate-lint:detail
        cases += 1
        write(TABLE, pinned_text.replace("shape=2c0;first=returned:d\tE7", "shape=2c1;first=returned:d\tE7"))
        expect("check-tampered", ["--check", ledger], 1, "重算的指纹不同")
        write(TABLE, pinned_text)

        def edited_table(edit):
            """钉好的表逐个数据行交给 edit(段列表, 第几个数据行) 改，交回整份文本。"""
            lines, index = [], 0
            for line in pinned_text.split("\n"):
                if line.strip() and not line.startswith("#"):
                    fields = line.split("\t")
                    edit(fields, index)
                    line, index = "\t".join(fields), index + 1
                lines.append(line)
            return "\n".join(lines)

        def set_field(row_index, column_index, value):
            def edit(fields, index):
                if index == row_index:
                    fields[column_index] = value
            return edit

        # 整行抄：Z1 那一行的 ⑲（账第 11 段，单条重放的命令）与账不同，别的段都对（开关 whole-row-unchecked 下判绿）
        write(TABLE, edited_table(set_field(2, 18, "cmd-hand-edited")))
        expect("check-whole-row-differs", ["--check", ledger], 1, "没整行钉：表第")
        # 段数不是 22 的行（8 段、21 段）：--check 与 --replay 都报行号与要 22 段，出路是手改坏了从 git 找回（开关 row-columns-unchecked 下当表里没有这一行）
        for label, kept in (("8", TABLE_OWN_COLUMNS), ("21", DEVICE_COLUMNS)):
            write(TABLE, edited_table(lambda fields, index, kept=kept: fields.__delitem__(slice(kept, None)) if index == 2 else None))
            expect(f"check-columns-{label}", ["--check", ledger], 1, f"第 4 行是 {label} 段，要 22 段")
        columns_output = expect("replay-columns", ["--replay", "--binary", stub], 1, "第 4 行是 21 段，要 22 段")
        if "从 git 找回再钉" not in columns_output:
            wrong.append("replay-columns-howto")
            print(f"    replay-columns-howto 这一格判错：出路要给从 git 找回再钉，实测：{columns_output.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        # 抄来的账行与自己的段对不上（开关 copied-columns-unchecked 下两格都判绿：桩照 ⑦ 打）
        write(TABLE, edited_table(set_field(0, 10, "X")))
        expect("replay-copied-domain-differs", ["--replay", "--binary", stub], 1, "第 2 行：⑪（红例账第 3 段）与 ② 不同")
        write(TABLE, edited_table(set_field(0, 19, state_line("L", 3, "shape=1c0", "returned:q"))))
        expect("replay-copied-original-differs", ["--replay", "--binary", stub], 1, "第 2 行：⑳（红例账第 12 段）与 ⑦ 不同")
        # --check-table（门禁 pinned-red-examples-table 格）：只判钉表自己与点名的变异，不读账、产物与 research/。
        # 没有 research/ 的根里照判绿（开关 table-check-globs-research 下退 77）；账在而同一趟的产物归档了也照判绿（开关下判红）；
        # 表坏了（重算的指纹、抄来的段、档的写法）、点名的变异不在了判红；表一行数据都没有退 77；给了账退 2
        write(TABLE, pinned_text)
        table_root = os.path.join(work, "table-root")
        for relative in (TABLE, CRATES_TABLE, CUSTOM_TABLE):
            os.makedirs(os.path.dirname(os.path.join(table_root, relative)), exist_ok=True)
            shutil.copyfile(os.path.join(root, relative), os.path.join(table_root, relative))
        table_green_text = "钉表 .claude/gate.d/e165-pinned-red-examples.tsv 3 行：每行 22 段"
        expect("check-table-without-research", ["--check-table", "--root", table_root], 0, table_green_text)
        cases += 1
        if os.path.exists(os.path.join(table_root, "research")):
            wrong.append("check-table-without-research-root")
            print("    check-table-without-research-root 这一格判错：这一格的根里不许有 research/")  # gate-lint:detail
        archived_ledger = os.path.join(root, "research/results/e165-selftest-archived-red-examples.tsv")
        shutil.copyfile(ledger, archived_ledger)
        expect("check-table-archived-ledger", ["--check-table"], 0, table_green_text)
        os.remove(archived_ledger)
        expect("check-table-ledger-arguments", ["--check-table", ledger], 2, "--check-table 不收红例账")
        write(TABLE, pinned_text.replace("shape=2c0;first=returned:d\tE7", "shape=2c1;first=returned:d\tE7"))
        expect("check-table-tampered", ["--check-table"], 1, "重算的指纹不同")
        write(TABLE, edited_table(set_field(0, 10, "X")))
        expect("check-table-copied-differs", ["--check-table"], 1, "第 2 行：⑪（红例账第 3 段）与 ② 不同")
        write(TABLE, edited_table(set_field(1, TABLE_COLUMNS - 1, "--profile r1 --table direct")))
        expect("check-table-profile-form", ["--check-table"], 1, "第 3 行的 ㉒「--profile r1 --table direct」不是规范写法的档")
        write(TABLE, pinned_text)
        write(CRATES_TABLE, crates_table.replace("红的两条", "改了名的"))
        expect("check-table-variant-missing", ["--check-table"], 1, "不在它的表里了")
        write(CRATES_TABLE, crates_table)
        write(TABLE, header)
        expect("check-table-empty", ["--check-table"], 77, "一行数据都没有")
        write(TABLE, pinned_text)
        write(TABLE, pinned_text)
        # 另一趟出了同样的红例：⑤ 不是它的行，只靠 ⑥ 认，名字取它自己那一趟的产物
        run_ledger = os.path.join(root, "research/results/e165-selftest-r1-red-examples.tsv")
        run_kill_bounds = os.path.join(root, "research/results/e165-selftest-r1-kill-bounds.out")
        write(os.path.relpath(run_ledger, root), "# 第二趟的账\n" + "".join("\t".join(fields) + "\n" for fields in ledger_rows))
        run_text = run_product(run_ledger)
        write(os.path.relpath(run_kill_bounds, root), run_text)
        both = ["--check", ledger, run_ledger]
        expect("check-run-names", both, 0, "2 份红例账 6 行都钉在")
        expect("check-kill-bounds-count", [*both, "--kill-bounds", run_kill_bounds], 2, "--check 给了 1 个 --kill-bounds、2 份红例账")
        # --check 显式给同一趟的产物（第二段驱动这样调，文件名可以不配）：用给的那一份，不按文件名配。
        # 按文件名配得上的那一份收尾行记的账 sha256 故意写错、给的那一份对：判绿；给的那一份错、按文件名配的对：判红（开关 check-kill-bounds-ignored 下两格都判错）
        explicit_product = os.path.join(work, "driver-output-elsewhere.out")
        wrong_digest_text = run_text.replace(f"ledger_sha256={file_digest(run_ledger)}", "ledger_sha256=" + "0" * 64)
        with open(explicit_product, "w", encoding="utf-8") as handle:
            handle.write(run_text)
        write(os.path.relpath(run_kill_bounds, root), wrong_digest_text)
        expect("check-explicit-kill-bounds", ["--check", run_ledger, "--kill-bounds", explicit_product], 0, "1 份红例账 3 行都钉在")
        with open(explicit_product, "w", encoding="utf-8") as handle:
            handle.write(wrong_digest_text)
        write(os.path.relpath(run_kill_bounds, root), run_text)
        expect("check-explicit-kill-bounds-digest", ["--check", run_ledger, "--kill-bounds", explicit_product], 1, "账与产物不是同一趟")
        os.remove(explicit_product)
        # 另一趟的账同一个输入、同一个变异，只是 ⑱（账第 10 段，首红与否）不同：追加另钉一行整行抄，之后 --check 才绿（开关 dedupe-by-input-variant 下两格判错）
        variant_ledger = os.path.join(root, "research/results/e165-selftest-r2-red-examples.tsv")
        variant_rows = [list(fields) for fields in ledger_rows]
        variant_rows[0][9] = "false"
        write(os.path.relpath(variant_ledger, root), "# 第三趟的账\n" + "".join("\t".join(fields) + "\n" for fields in variant_rows))
        variant_kill_bounds = os.path.join(root, "research/results/e165-selftest-r2-kill-bounds.out")
        write(os.path.relpath(variant_kill_bounds, root), run_product(variant_ledger))
        expect("pin-whole-row-variant", [variant_ledger], 0, "钉进 1 行（1 份账 4 行：2 行已钉")
        expect("check-whole-row-variant", ["--check", ledger, variant_ledger], 0, "2 份红例账 6 行都钉在")
        os.remove(variant_ledger)
        os.remove(variant_kill_bounds)
        write(TABLE, pinned_text)
        # 追加与 --check 同一套：那一趟时变异表第 2 行是「先删的一条」，之后被删、末尾加了一行；账里 tsv-3、tsv-4 要按那一趟的 title 钉成「红的一条」「红的两条」
        # （开关 append-names-from-today-table 下按今天的行号钉成「红的两条」「后加的一条」，append-pairing-optional 下产物不在、账 sha256 不对也照钉）
        deleted_ledger = os.path.join(root, "research/results/e165-selftest-deleted-red-examples.tsv")
        deleted_kill_bounds = os.path.join(root, "research/results/e165-selftest-deleted-kill-bounds.out")
        deleted_rows = [
            ["tsv-3", "3", "M", "2", "7", "counters", "+J4", "true", "true", "true", "cmd", state_line("M", 7, "shape=3c0", "returned:a"), state_line("M", 7, "shape=3c0", "returned:b")],
            ["tsv-4", "4", "M", "2", "8", "counters", "+J4", "true", "true", "true", "cmd", state_line("M", 8, "shape=3c1", "returned:a"), state_line("M", 8, "shape=3c1", "returned:c")],
        ]
        write(os.path.relpath(deleted_ledger, root), "# 删行之后的账\n" + "".join("\t".join(fields) + "\n" for fields in deleted_rows))
        write(CRATES_TABLE, crates_table + "后加的一条\tcrates/x.rs\tg\th\t-p x\tt\n")
        deleted_product = ("E7RESULT name=config experiment=E165 segment=2 device_arguments=--profile+r1 bounds=segment2\n"
                           "E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=3 outcome=red red_examples=1 title=红的一条 complete=yes\n"
                           "E7RESULT name=kill_bound item=tsv-4 table=crates/mutations.tsv table_row=4 outcome=red red_examples=1 title=红的两条 complete=yes\n"
                           f"E7RESULT name=verdict scope=segment2 variants=2 red=2 ledger_sha256={file_digest(deleted_ledger)}\n")
        expect("pin-kill-bounds-missing", [deleted_ledger], 1, "配的 kill-bounds 产物 research/results/e165-selftest-deleted-kill-bounds.out 不在")
        if open(table_path, encoding="utf-8").read() != pinned_text:
            wrong.append("pin-kill-bounds-missing-wrote")
            print("    pin-kill-bounds-missing-wrote 这一格判错：配不上产物判红之后表还是被写了")  # gate-lint:detail
        cases += 1
        write(os.path.relpath(deleted_kill_bounds, root), deleted_product.replace(f"ledger_sha256={file_digest(deleted_ledger)}", "ledger_sha256=" + "0" * 64))
        expect("pin-ledger-digest", [deleted_ledger], 1, "账与产物不是同一趟")
        # --kill-bounds 显式给的产物（第二段驱动收尾时这样调，文件名可以不配）过同样的四样：收尾行记的账不是这一份照样判红
        driver_product = os.path.join(root, "research/results/driver-output.out")
        write(os.path.relpath(driver_product, root), deleted_product.replace(f"ledger_sha256={file_digest(deleted_ledger)}", "ledger_sha256=" + "0" * 64))
        expect("pin-explicit-kill-bounds-digest", ["--kill-bounds", driver_product, deleted_ledger], 1, "账与产物不是同一趟")
        os.remove(driver_product)
        write(os.path.relpath(deleted_kill_bounds, root), deleted_product)
        expect("pin-deleted-row", [deleted_ledger], 0, "钉进 2 行")
        deleted_text = open(table_path, encoding="utf-8").read()
        if ("e165-selftest-deleted-red-examples.tsv:2\tcrates/mutations.tsv#红的一条\t" not in deleted_text
                or "e165-selftest-deleted-red-examples.tsv:3\tcrates/mutations.tsv#红的两条\t" not in deleted_text or "#后加的一条" in deleted_text):
            wrong.append("pin-deleted-row-names")
            print("    pin-deleted-row-names 这一格判错：tsv-3、tsv-4 要按那一趟的 title 钉成「红的一条」「红的两条」，不许出现今天按行号换出的「后加的一条」")  # gate-lint:detail
        cases += 1
        expect("check-deleted-row", ["--check", deleted_ledger], 0, "1 份红例账 2 行都钉在")
        os.remove(deleted_ledger)
        os.remove(deleted_kill_bounds)
        write(CRATES_TABLE, crates_table)
        write(TABLE, pinned_text)
        # 钉完之后别的会话删了变异表第 2 行、末尾加一行：今天的 tsv-2 是「红的两条」、tsv-3 是新加的那条，那一趟的名字照旧
        write(CRATES_TABLE, crates_table.replace("红的一条\tcrates/x.rs\ta\tb\t-p x\tt\n", "") + "后加的一条\tcrates/x.rs\tg\th\t-p x\tt\n")
        expect("check-shifted-table", both, 0, "2 份红例账 6 行都钉在")
        write(CRATES_TABLE, crates_table)
        # 真没钉全：那一趟给 tsv-3 的名字是表里没钉过的变异
        write(os.path.relpath(run_kill_bounds, root), run_text.replace("title=红的两条", "title=别的变异"))
        missing_text = expect("check-title-other-variant", both, 1, "6 行里 1 行没钉进")
        # 出路逐份给钉的命令（追加按文件名配同一趟的产物，不带 --kill-bounds），只点没钉全的那一份账
        if ("e165-pin-red-examples.py research/results/e165-selftest-r1-red-examples.tsv" not in missing_text
                or "research/results/e165-selftest-red-examples.tsv" in missing_text or "--kill-bounds" in missing_text):
            wrong.append("check-title-other-variant-howto")
            print(f"    check-title-other-variant-howto 这一格判错：出路要给 e165-pin-red-examples.py <没钉全的账>、不点钉全了的账、不带 --kill-bounds，实测：{missing_text.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        # 配对不上：title 缺了、产物不在、变体集合不同、收尾行记的账不是这一份、没收尾
        write(os.path.relpath(run_kill_bounds, root), run_text.replace(" title=红的两条", ""))
        expect("check-title-missing", both, 1, "item=tsv-3 table_row=3 那一行，或那一行没有 title=")
        os.remove(run_kill_bounds)
        expect("check-kill-bounds-missing", both, 1, "配的 kill-bounds 产物 research/results/e165-selftest-r1-kill-bounds.out 不在")
        write(os.path.relpath(run_kill_bounds, root),
              run_text + "E7RESULT name=kill_bound item=tsv-9 table=crates/mutations.tsv table_row=9 outcome=red red_examples=2 title=账里没有的 complete=yes\n")
        expect("check-variant-sets-differ", both, 1, "tsv-9：账里 0 行，产物的 red_examples= 是 2")
        write(os.path.relpath(run_kill_bounds, root), run_text.replace(f"ledger_sha256={file_digest(run_ledger)}", "ledger_sha256=" + "0" * 64))
        expect("check-ledger-digest", both, 1, "账与产物不是同一趟")
        write(os.path.relpath(run_kill_bounds, root), "".join(line + "\n" for line in run_text.split("\n") if line.startswith("E7RESULT name=kill_bound ")))
        expect("check-run-unfinished", both, 1, "没有收尾行 E7RESULT name=verdict")
        os.remove(run_kill_bounds)
        os.remove(run_ledger)
        conflicting = [list(fields) for fields in ledger_rows]
        conflicting[0][11] = state_line("L", 3, "shape=1c0", "returned:z")
        other_ledger = os.path.join(root, "research/results/e165-selftest-conflict-red-examples.tsv")
        write(os.path.relpath(other_ledger, root), "# 账头\n" + "\t".join(conflicting[0]) + "\n")
        other_kill_bounds = os.path.join(root, "research/results/e165-selftest-conflict-kill-bounds.out")
        write(os.path.relpath(other_kill_bounds, root),
              "E7RESULT name=config experiment=E165 segment=2 device_arguments=--profile+r1 bounds=segment2\n"
              "E7RESULT name=kill_bound item=tsv-2 table=crates/mutations.tsv table_row=2 outcome=red red_examples=1 title=红的一条 complete=yes\n"
              f"E7RESULT name=verdict scope=segment2 variants=1 red=1 ledger_sha256={file_digest(other_ledger)}\n")
        expect("conflict", [other_ledger], 1, "原实现逐态行不同")
        os.remove(other_ledger)
        os.remove(other_kill_bounds)
        expect("replay-green", ["--replay", "--binary", stub], 0, "都与 ⑦ 逐字节相同")
        expect("replay-mismatch", ["--replay", "--binary", stub], 1, "行与 ⑦ 不同", {"STUB_MODE": "wrong"})
        expect("replay-fail", ["--replay", "--binary", stub], 1, "装置退 1", {"STUB_MODE": "fail"})
        # 进度行：装置跑得超过进度间隔（假时钟每读一次走 200 秒、间隔 300 秒；桩打一行后停 0.6 秒），跑着时至少打一行 E165_PINNED_PROGRESS，
        # 带挂钟与已打的结果行数；判定照常绿（开关 progress-silent 下一行都不打，这一格判错）
        fake_now, progress_sink = [0.0], io.StringIO()

        def fake_clock():
            fake_now[0] += 200.0
            return fake_now[0]

        saved_progress = (PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK)
        PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK = fake_clock, 0.05, progress_sink
        try:
            expect("replay-progress-green", ["--replay", "--binary", stub], 0, "都与 ⑦ 逐字节相同", {"STUB_MODE": "slow"})
        finally:
            PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK = saved_progress
        progress_lines = [line for line in progress_sink.getvalue().split("\n") if line.startswith("E165_PINNED_PROGRESS ")]
        if not progress_lines or not all(re.search(r"挂钟 \d+ 秒：.*装置已打 \d+/3 行结果", line) for line in progress_lines):
            wrong.append("replay-progress")
            print(f"    replay-progress 这一格判错：装置跑过进度间隔时要至少一行 E165_PINNED_PROGRESS（挂钟、已打 n/3 行结果），实测：{progress_sink.getvalue()[-300:]!r}")  # gate-lint:detail
        cases += 1
        # 装置被信号停了（kill -9 自己；经 bash 起时退 137，外面 kill -9 就是这样）：出路写查是谁停了它，不写修实现（开关 killed-as-regression 下这三格判错）
        killed_howto = "查是谁在那一刻停了它"
        killed = expect("replay-killed", ["--replay", "--binary", stub], 1, "装置被信号停了（退 -9，信号 9）", {"STUB_MODE": "killed"})
        if killed_howto not in killed or "修实现" in killed:
            wrong.append("replay-killed-howto")
            print(f"    replay-killed-howto 这一格判错：出路要写查是谁停了它、不写修实现，实测：{killed.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        expect("replay-exit-137", ["--replay", "--binary", stub], 1, "装置被信号停了（退 137，信号 9）", {"STUB_MODE": "exit-137"})
        write(CRATES_TABLE, crates_table.replace("红的两条", "改了名的"))
        expect("replay-variant-missing", ["--replay", "--binary", stub], 1, "不在它的表里了")
        write(CRATES_TABLE, crates_table)

        # ── 逐项点名与按条复用（文件头「逐项点名与按条复用」）──
        item_line = re.compile(r"^  · (pinned:\d+)：(绿|红)，(现判|复用)", re.M)

        fresh_libraries = []

        def run_items(arguments, environment=None, at=None):
            """跑一次 main（根默认 root），交 (退出码, 输出, {项: (绿/红, 现判/复用)})。参数里写「--selftest-library <目录>」的那一份库交给 main 的 selftest_library
            （几趟共用一份库的格这样写），没写的给一份新的空库（判定库没有不读不写的形态）；这一对不交给 main 的命令行。
            参数里写「--selftest-forced-judge」的交给 main 的 selftest_forced_judge（每一行不读库、同键照样现判，造记红撤绿），同样不交给命令行。"""
            forced_judge = "--selftest-forced-judge" in arguments
            arguments = [argument for argument in arguments if argument != "--selftest-forced-judge"]
            if "--selftest-library" in arguments:
                position = arguments.index("--selftest-library")
                library = arguments[position + 1]
                del arguments[position:position + 2]
            else:
                fresh_libraries.append(os.path.join(work, f"fresh-library-{len(fresh_libraries)}"))
                library = fresh_libraries[-1]
            saved = dict(os.environ)
            os.environ.update(environment or {})
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    got = main(["--root", at or root, *arguments], selftest_library=library, selftest_forced_judge=forced_judge)
            finally:
                os.environ.clear()
                os.environ.update(saved)
            output = buffer.getvalue()
            return got, output, {match.group(1): (match.group(2), match.group(3)) for match in item_line.finditer(output)}

        def judge_cell(label, passed, detail):
            nonlocal cases
            cases += 1
            if not passed:
                wrong.append(label)
                print(f"    {label} 这一格判错：{detail}")  # gate-lint:detail

        write(TABLE, pinned_text)
        replay_arguments = ["--replay", "--binary", stub]
        _text, pinned_rows = read_table(table_path)
        pinned_items = [pinned_item(row) for row in pinned_rows]
        x_items = [pinned_item(row) for row in pinned_rows if row["fields"][1] == "X"]
        stub_trace = os.path.join(work, "stub-trace.txt")
        open(stub_trace, "w").close()
        got, output, _verdicts = run_items(["--list-items"], {"STUB_TRACE": stub_trace})
        listed = [line.split("\t")[0] for line in output.strip().split("\n")]
        judge_cell("list-items", got == 0 and listed == ["table", *pinned_items] and len(pinned_items) == 3, f"退 {got}，列出 {listed}")
        judge_cell("list-items-no-device", open(stub_trace).read() == "", "--list-items 起了装置")
        # 读的一头先关了（接 | head 的样子）：被 SIGPIPE 安静停掉，stderr 一个字都没有
        read_end, write_end = os.pipe()
        os.close(read_end)
        closed_pipe = subprocess.run([sys.executable, os.path.realpath(__file__), "--root", root, "--list-items"], stdout=write_end, stderr=subprocess.PIPE, text=True)
        os.close(write_end)
        judge_cell("list-items-closed-pipe", closed_pipe.returncode == -signal.SIGPIPE and closed_pipe.stderr == "",
                   f"期望被 SIGPIPE 停（{-signal.SIGPIPE}）、stderr 空，实测 {closed_pipe.returncode}：{closed_pipe.stderr.strip()[-200:]!r}")
        got, output, verdicts = run_items([*replay_arguments, "--item", pinned_items[1]])
        judge_cell("item-select-one", got == 0 and list(verdicts) == [pinned_items[1]], f"--item {pinned_items[1]} 期望只判这一行，实测退 {got}、判了 {list(verdicts)}")
        got, output, _verdicts = run_items([*replay_arguments, "--item", "pinned:999"])
        judge_cell("item-unknown", got == EXIT_USAGE and pinned_items[0] in output, f"点名没有的项期望退 2、列出可点名的项，实测退 {got}：{output.strip()[-300:]}")
        got, output, _verdicts = run_items(["--check", ledger, "--item", "table"])
        judge_cell("item-not-for-check", got == EXIT_USAGE, f"--check 给 --item 期望退 2，实测退 {got}")
        got, output, _verdicts = run_items(["--check-table", "--item", "table"])
        judge_cell("item-not-for-check-table", got == EXIT_USAGE and "pinned-red-examples-table 只有一项（table）" in output, f"--check-table 给 --item 期望退 2，实测退 {got}：{output.strip()[-200:]}")
        # 点名单跑一行与它在整格里跑的判定相同（桩把 X 域那一行打错：整格红、X 那一行红、别的两行绿）
        _got, _output, whole = run_items(replay_arguments, {"STUB_MODE": "wrong-x"})
        alone = {item: run_items([*replay_arguments, "--item", item], {"STUB_MODE": "wrong-x"})[2].get(item) for item in whole}
        judge_cell("item-alone-equals-whole", len(whole) == 3 and alone == whole and all(whole[item][0] == "红" for item in x_items)
                   and sorted(verdict for verdict, _how in whole.values()) == ["红", "绿", "绿"], f"整格 {whole}，逐行单跑 {alone}")
        # 按条复用：头一趟全现判、记下；第二趟全复用、不起装置；关了复用全现判；桩（装置）、引的变异行、自己的行各进键
        store_directory = os.path.join(work, "verdict-library")
        with_store = [*replay_arguments, "--selftest-library", store_directory]
        got, output, _verdicts = run_items(with_store)
        judge_cell("reuse-first-run-records", got == 0 and "现判 3、复用 0（现判的原因：无记录 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        open(stub_trace, "w").close()
        got, output, verdicts = run_items(with_store, {"STUB_TRACE": stub_trace})
        judge_cell("reuse-second-run-reuses", got == 0 and "现判 0、复用 3" in output and open(stub_trace).read() == ""
                   and all(how == "复用" for _verdict, how in verdicts.values()), f"退 {got}、装置起了 {open(stub_trace).read().count(chr(10))} 次：{output.strip()[-300:]}")
        # 库位置只由被判那棵仓定（文件头「逐项点名与按条复用」）：--root 是 git 工作树的照 verus-proof-check.py library-directory 打的那一份往下判（这棵树没有钉表，判红退 1）；
        #   不是 git 工作树的一行都不判、退 2、点名它（子进程起，不给自证的临时库）
        library_git_root = os.path.join(work, "library-git-root")
        library_plain_root = os.path.join(work, "library-plain-root")
        os.makedirs(library_git_root)
        os.makedirs(library_plain_root)
        subprocess.run(["git", "init", "-q", library_git_root], check=True, stdin=subprocess.DEVNULL)
        placed = subprocess.run([sys.executable, os.path.realpath(__file__), "--replay", "--root", library_git_root, "--binary", stub],
                                capture_output=True, text=True, stdin=subprocess.DEVNULL)
        unplaced = subprocess.run([sys.executable, os.path.realpath(__file__), "--replay", "--root", library_plain_root, "--binary", stub],
                                  capture_output=True, text=True, stdin=subprocess.DEVNULL)
        judge_cell("replay-outside-git-exits-2",
                   placed.returncode == EXIT_RED and "钉住的表" in placed.stdout and unplaced.returncode == EXIT_USAGE and "不是 git 工作树" in unplaced.stdout,
                   f"git 工作树期望往下报钉住的表不在退 {EXIT_RED}，实测退 {placed.returncode}：{(placed.stdout + placed.stderr)[-300:]}；"
                   f"不是 git 工作树期望退 {EXIT_USAGE} 并点名，实测退 {unplaced.returncode}：{(unplaced.stdout + unplaced.stderr)[-300:]}")
        # 判法单元指纹是另一列：输入一样都没变、只换判法单元指纹，整格全现判、原因点名它；再照真的跑一趟记回去，后面几格比的是真判法单元指纹
        JUDGING_UNITS_FOR_SELFTEST.append("自证造的另一份判法单元")
        try:
            got, output, _verdicts = run_items(with_store)
        finally:
            JUDGING_UNITS_FOR_SELFTEST.clear()
        judge_cell("reuse-judging-units-changed", got == 0 and "现判 3、复用 0（现判的原因：判法单元指纹 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        run_items(with_store)
        write(CRATES_TABLE, crates_table.replace("红的一条\tcrates/x.rs\ta\t", "红的一条\tcrates/x.rs\ta2\t"))
        got, output, _verdicts = run_items(with_store)
        write(CRATES_TABLE, crates_table)
        judge_cell("reuse-mutation-row-changed", got == 0 and "现判 1、复用 2（现判的原因：变异原文与替换文 变了 1）" in output, f"退 {got}：{output.strip()[-300:]}")
        got, output, _verdicts = run_items(with_store)
        judge_cell("reuse-mutation-row-restored", got == 0 and "现判 0、复用 3" in output, f"变异行改回去之后期望全复用（键回到头一趟那一份），实测退 {got}：{output.strip()[-300:]}")
        stub_text = open(stub, encoding="utf-8").read()

        def replace_stub(new_text):
            with open(stub + ".new", "w", encoding="utf-8") as handle:
                handle.write(new_text)
            os.chmod(stub + ".new", 0o755)
            os.replace(stub + ".new", stub)
        replace_stub(stub_text + "# 桩改了一行注释\n")
        got, output, _verdicts = run_items(with_store)
        judge_cell("reuse-device-changed", got == 0 and "现判 3、复用 0（现判的原因：装置替身 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        replace_stub(stub_text)
        run_items(with_store)
        # --binary 换成另一份桩（另一个路径、内容多一行注释）：装置替身那一样变了，三行全现判；换回原来那份再记一趟，后面几格照原来的键比
        other_stub = os.path.join(work, "stub-device-other.sh")
        with open(other_stub, "w", encoding="utf-8") as handle:
            handle.write(stub_text + "# 另一份桩\n")
        os.chmod(other_stub, 0o755)
        got, output, _verdicts = run_items(["--replay", "--binary", other_stub, "--selftest-library", store_directory])
        judge_cell("reuse-device-stand-in-swapped", got == 0 and "现判 3、复用 0（现判的原因：装置替身 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        run_items(with_store)
        # 账路径改名（T13 的账路径那一半）：每行 ⑤ 换一个账文件名，内容不变，三行全复用（开关 row-key-with-ledger-path 下判错）
        renamed_text = "\n".join(raw if not raw.strip() or raw.startswith("#") else
                                  "\t".join(field.replace("-red-examples.tsv:", "-red-examples-renamed.tsv:") if index == 4 else field
                                            for index, field in enumerate(raw.split("\t"))) for raw in pinned_text.split("\n"))
        write(TABLE, renamed_text)
        got, output, _verdicts = run_items(with_store)
        write(TABLE, pinned_text)
        judge_cell("reuse-ledger-path-renamed", renamed_text != pinned_text and got == 0 and "现判 0、复用 3" in output,
                   f"期望账路径改名三行全复用，实测退 {got}：{output.strip()[-300:]}")
        # 变异改名：变异表那一行与 ⑥ 一起改名，原文与替换文不变，照复用（开关 row-key-with-mutation-name 下判错）
        renamed_mutation = crates_table.replace("红的一条\tcrates/x.rs\t", "红的那一条\tcrates/x.rs\t")
        write(CRATES_TABLE, renamed_mutation)
        write(TABLE, pinned_text.replace("#红的一条", "#红的那一条"))
        got, output, _verdicts = run_items(with_store)
        write(CRATES_TABLE, crates_table)
        write(TABLE, pinned_text)
        judge_cell("reuse-mutation-renamed", renamed_mutation != crates_table and got == 0 and "现判 0、复用 3" in output,
                   f"期望变异改名照复用，实测退 {got}：{output.strip()[-300:]}")
        # 每一行的键里有这一行自己：表里另有一行与第一行同一个输入、同一个 ⑥，只是期望逐态行 ⑦（与抄来的 ⑳）多一个计数名；只判过第一行，那一行不能拿它的判定
        twin_fields = list(pinned_rows[0]["fields"])
        twin_fields[6] = twin_fields[6].replace("counters=states", "counters=states,twin", 1)
        twin_fields[19] = twin_fields[6]
        write(TABLE, pinned_text.rstrip("\n") + "\n" + "\t".join(twin_fields) + "\n")
        twin_item = f"pinned:{len(pinned_text.rstrip(chr(10)).split(chr(10))) + 1}"
        own_store = os.path.join(work, "verdict-library-own")
        run_items([*replay_arguments, "--selftest-library", own_store, "--item", pinned_items[0]])
        got, output, verdicts = run_items([*replay_arguments, "--selftest-library", own_store])
        judge_cell("reuse-row-own-key", got == 0 and verdicts.get(twin_item) == ("绿", "现判") and "现判 3、复用 1" in output,
                   f"期望 {pinned_items[0]} 复用、{twin_item} 现判，实测退 {got}、{verdicts}")
        write(TABLE, pinned_text)
        # 记红撤绿：先判绿入库；同一把键（STUB_MODE 只在环境里，不进键）在自证的强制现判下（--selftest-forced-judge：每一行不读库）判一趟、X 那一行红；下一趟 X 现判、别的复用
        # （开关 red-keeps-green 下 X 照复用，这一格判错）
        withdraw_store = os.path.join(work, "verdict-library-red-withdraws-green")
        first_got, _output, _verdicts = run_items([*replay_arguments, "--selftest-library", withdraw_store])
        red_got, _output, red_verdicts = run_items([*replay_arguments, "--selftest-library", withdraw_store, "--selftest-forced-judge"], {"STUB_MODE": "wrong-x"})
        got, output, verdicts = run_items([*replay_arguments, "--selftest-library", withdraw_store])
        judge_cell("reuse-red-withdraws-green", first_got == 0 and red_got == 1 and got == 0 and all(verdicts.get(item) == ("绿", "现判") for item in x_items)
                   and "现判 1、复用 2" in output,
                   f"头一趟退 {first_got}、判红那一趟退 {red_got}（{red_verdicts}），下一趟期望 X 那一行现判、别的复用，实测退 {got}、{verdicts}")
        # --list-items --key-mismatches 逐行往 stderr 打 ITEM_KEY（门禁 --list-items --with-keys 转成 ITEM_PLAN 的原料；只读库、不写库、不起装置）：库是空的三行都「无记录」；
        # 判过一趟之后三行「键相同」；改了头一行的期望逐态行 ⑦（键里有它）之后那一行「键对不上」、别的两行「键相同」
        # （开关 item-key-lines-skipped 下一行都不打；key-mismatch-by-item-only 下改了的那一行也算键相同；这一格判错）
        def item_key_states(library):
            listed_error = io.StringIO()
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(listed_error):
                listed_exit = main(["--root", root, "--list-items", "--key-mismatches", "--binary", stub], selftest_library=library)
            return listed_exit, dict(match.groups() for match in re.finditer(rf"^{ITEM_KEY_PREFIX} item=(\S+) state=(\S+)$", listed_error.getvalue(), re.M))
        key_store = os.path.join(work, "verdict-library-item-key")
        empty_exit, empty_states = item_key_states(key_store)
        # --list-items --key-mismatches 带 --item：只打点名那一行的 ITEM_KEY（开关 key-items-ignored 下照旧打整表，这一格判错）
        named_error = io.StringIO()
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(named_error):
            named_exit = main(["--root", root, "--list-items", "--key-mismatches", "--binary", stub, "--item", pinned_items[1]], selftest_library=key_store)
        named_states = dict(match.groups() for match in re.finditer(rf"^{ITEM_KEY_PREFIX} item=(\\S+) state=(\\S+)$", named_error.getvalue(), re.M))
        judge_cell("list-items-item-key-with-item", named_exit == 0 and named_states == {pinned_items[1]: "无记录"}, f"点名 {pinned_items[1]} 期望只打这一行，实测退 {named_exit}：{named_states}")
        run_items([*replay_arguments, "--selftest-library", key_store])
        same_exit, same_states = item_key_states(key_store)
        first_line = "\t".join(pinned_rows[0]["fields"])
        changed_fields = list(pinned_rows[0]["fields"])
        changed_fields[6] = changed_fields[6].replace("counters=states", "counters=states,key", 1)
        write(TABLE, pinned_text.replace(first_line, "\t".join(changed_fields), 1))
        changed_exit, changed_states = item_key_states(key_store)
        write(TABLE, pinned_text)
        judge_cell("list-items-item-key-lines",
                   empty_exit == 0 and same_exit == 0 and changed_exit == 0 and first_line != "\t".join(changed_fields)
                   and empty_states == {item: "无记录" for item in pinned_items} and same_states == {item: "键相同" for item in pinned_items}
                   and changed_states == {item: ("键对不上" if item == pinned_items[0] else "键相同") for item in pinned_items},
                   f"空库 {empty_states}，判过一趟 {same_states}，改了 {pinned_items[0]} 的 ⑦ {changed_states}")
        # 逐行条数与退出码（ITEM_DONE 的原料，环境里的 FORMAL_ITEM_DONE_FILE）：头一趟三行全现判（现判 1、复用 0、退出码 0）；第二趟全复用（0 秒、现判 0、复用 1）；
        # 桩把 X 那一行打错的那一趟 X 的退出码是 1、别的 0（开关 item-done-unrecorded 下一行都不记，这一格判错）
        done_file = os.path.join(work, "item-done.tsv")

        def item_done_rows(arguments, environment=None):
            open(done_file, "w").close()
            run_items(arguments, dict({ITEM_DONE_FILE_VARIABLE: done_file}, **(environment or {})))
            return {fields[0]: fields[1:] for fields in (line.split("\t") for line in open(done_file, encoding="utf-8").read().split("\n") if line)}
        done_store = os.path.join(work, "verdict-library-item-done")
        first_done = item_done_rows([*replay_arguments, "--selftest-library", done_store])
        second_done = item_done_rows([*replay_arguments, "--selftest-library", done_store])
        red_done = item_done_rows(replay_arguments, {"STUB_MODE": "wrong-x"})
        judge_cell("item-done-lines",
                   set(first_done) == set(pinned_items) and all(first_done[item][1:] == ["1", "0", "0"] for item in pinned_items)
                   and set(second_done) == set(pinned_items) and all(second_done[item] == ["0.0", "0", "1", "0"] for item in pinned_items)
                   and set(red_done) == set(pinned_items) and all(red_done[item][1:3] == ["1", "0"] and red_done[item][3] == ("1" if item in x_items else "0") for item in pinned_items),
                   f"头一趟 {first_done}，第二趟（全复用）{second_done}，X 那一行判红那一趟 {red_done}")
        # 红的不记：桩把 X 那一行打错，第二趟 X 照判、照红
        red_store = os.path.join(work, "verdict-library-red")
        run_items([*replay_arguments, "--selftest-library", red_store], {"STUB_MODE": "wrong-x"})
        got, output, verdicts = run_items([*replay_arguments, "--selftest-library", red_store], {"STUB_MODE": "wrong-x"})
        judge_cell("reuse-red-not-recorded", got == 1 and all(verdicts.get(item) == ("红", "现判") for item in x_items) and "现判 1、复用 2" in output,
                   f"第二趟期望 X 那一行照判照红、别的复用，实测退 {got}、{verdicts}")
        # 一块判完当场落盘：一块一行，X 那一块装置被 kill -9（不在键里的环境），前两块已记；第二趟同一条命令只判 X
        resume_store = os.path.join(work, "verdict-library-resume")
        first, _output, _verdicts = run_items([*replay_arguments, "--selftest-library", resume_store, "--block-rows", "1"], {"STUB_KILL_DOMAIN": "X"})
        got, output, _verdicts = run_items([*replay_arguments, "--selftest-library", resume_store, "--block-rows", "1"])
        judge_cell("reuse-resume-after-kill", first == 1 and got == 0 and "现判 1、复用 2" in output, f"第一趟退 {first}，第二趟退 {got}：{output.strip()[-300:]}")
        # 块不干净时配得上的绿照记：桩打全三行、报错点了 X 那一行的行号（像装置报指纹漂了）、退 1；L 两行判绿、当场记，X 记红；
        # 下一趟照常的桩只现判 X（开关 unclean-block-records-nothing 下这一块一行都不记，下一趟三行全现判）
        unclean_store = os.path.join(work, "verdict-library-unclean-block")
        first, _output, first_verdicts = run_items([*replay_arguments, "--selftest-library", unclean_store], {"STUB_MODE": "drift-x"})
        got, output, verdicts = run_items([*replay_arguments, "--selftest-library", unclean_store])
        judge_cell("replay-unclean-block-records-green",
                   first == 1 and all(first_verdicts.get(item) == (("红" if item in x_items else "绿"), "现判") for item in pinned_items)
                   and got == 0 and all(verdicts.get(item) == ("绿", "现判" if item in x_items else "复用") for item in pinned_items) and "现判 1、复用 2" in output,
                   f"头一趟期望退 1、X 红、L 两行绿，实测退 {first}、{first_verdicts}；下一趟期望 X 现判、L 两行复用，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # 块里少打一行，后面的行不错配：桩不打头一个数据行（L）的结果、报错点它的行号（像装置报序号不在域里）、退 1；
        # 两个 L 行同一个 ②③④、只打了一行这个身份的结果，分不出是谁的、都不配不记绿；X 那一行配上自己打的那一行、判绿、记下；
        # 下一趟照常的桩只现判两个 L 行（开关 results-paired-by-position 下 X 的结果配给了第二个 L 行、X 自己没配上，一行绿都不记，下一趟三行全现判）
        missing_store = os.path.join(work, "verdict-library-missing-line")
        first, _output, first_verdicts = run_items([*replay_arguments, "--selftest-library", missing_store], {"STUB_MODE": "skip-first"})
        got, output, verdicts = run_items([*replay_arguments, "--selftest-library", missing_store])
        judge_cell("replay-missing-line-no-mispair",
                   first == 1 and all(first_verdicts.get(item) == (("绿" if item in x_items else "红"), "现判") for item in pinned_items)
                   and got == 0 and all(verdicts.get(item) == ("绿", "复用" if item in x_items else "现判") for item in pinned_items) and "现判 2、复用 1" in output,
                   f"头一趟期望退 1、X 绿、L 两行红，实测退 {first}、{first_verdicts}；下一趟期望 X 复用、L 两行现判，实测退 {got}、{verdicts}：{output.strip()[-300:]}")
        # 剪枝自证：小输入上开着复用与关掉复用，逐项判定相同；输入没变（全复用）与装置改了（全现判、X 转红）各比一次
        prune_store = os.path.join(work, "verdict-library-prune")
        run_items([*replay_arguments, "--selftest-library", prune_store])
        on = run_items([*replay_arguments, "--selftest-library", prune_store])[2]
        off = run_items(replay_arguments)[2]
        judge_cell("prune-unchanged-input", {item: verdict for item, (verdict, _how) in on.items()} == {item: verdict for item, (verdict, _how) in off.items()}
                   and len(on) == 3 and all(how == "复用" for _verdict, how in on.values()), f"开着复用 {on}，关掉 {off}")
        replace_stub(stub_text.replace("[[ \"$1\" == --pinned-cases ]] || exit 2\n", "[[ \"$1\" == --pinned-cases ]] || exit 2\nSTUB_MODE=wrong-x\n", 1))
        on = run_items([*replay_arguments, "--selftest-library", prune_store])[2]
        off = run_items(replay_arguments)[2]
        replace_stub(stub_text)
        judge_cell("prune-after-change", {item: verdict for item, (verdict, _how) in on.items()} == {item: verdict for item, (verdict, _how) in off.items()}
                   and all(off.get(item, ("?",))[0] == "红" for item in x_items), f"开着复用 {on}，关掉 {off}")
        # 装置的 cargo 依赖闭包进键：临时 git 仓里摆一个最小工作区（验证包带装置 bin、依赖一个 path crate），只改那个依赖的源码，三行全现判
        closure_root = os.path.join(work, "closure-root")
        for relative in (TABLE, CRATES_TABLE, CUSTOM_TABLE):
            os.makedirs(os.path.dirname(os.path.join(closure_root, relative)), exist_ok=True)
            with open(os.path.join(closure_root, relative), "w", encoding="utf-8") as handle:
                handle.write(open(os.path.join(root, relative), encoding="utf-8").read())
        closure_files = {"Cargo.toml": '[workspace]\nmembers = ["crates/singlefs-verification", "crates/dep"]\n', "Cargo.lock": "# 锁\n",
                         "crates/singlefs-verification/Cargo.toml": '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n\n[dependencies]\ndep = { path = "../dep" }\n',
                         f"crates/singlefs-verification/src/bin/{DEVICE_BINARY}.rs": f"fn main() {{\n    dep::one();\n    helper();\n}}\nfn helper() -> u32 {{\n    2\n}}\n",
                         "crates/dep/Cargo.toml": '[package]\nname = "dep"\nversion = "0.0.0"\n', "crates/dep/src/lib.rs": "pub fn one() -> u32 { 1 }\n"}
        for relative, content in closure_files.items():
            os.makedirs(os.path.dirname(os.path.join(closure_root, relative)) or closure_root, exist_ok=True)
            with open(os.path.join(closure_root, relative), "w", encoding="utf-8") as handle:
                handle.write(content)
        subprocess.run(["git", "init", "-q", closure_root], capture_output=True)
        closure_store = os.path.join(work, "verdict-library-closure")
        run_items([*replay_arguments, "--selftest-library", closure_store], at=closure_root)
        with open(os.path.join(closure_root, "crates/dep/src/lib.rs"), "w", encoding="utf-8") as handle:
            handle.write("pub fn one() -> u32 { 1 + 0 }\n")
        got, output, _verdicts = run_items([*replay_arguments, "--selftest-library", closure_store], at=closure_root)
        judge_cell("reuse-closure-changed", got == 0 and "现判 3、复用 0（现判的原因：装置单元闭包 变了 3）" in output and "改了 fn one" in output,
                   f"期望装置调到的函数改了三行全现判、点名 fn one，实测退 {got}：{output.strip()[-300:]}")
        with open(os.path.join(closure_root, "crates/dep/src/lib.rs"), "a", encoding="utf-8") as handle:
            handle.write("pub fn nobody_calls_it() -> u32 { 2 }\n")
        got, output, _verdicts = run_items([*replay_arguments, "--selftest-library", closure_store], at=closure_root)
        judge_cell("reuse-closure-unrelated-function", got == 0 and "现判 0、复用 3" in output, f"依赖里加一个没人调的函数期望全复用，实测退 {got}：{output.strip()[-300:]}")
        os.makedirs(os.path.join(closure_root, "crates/singlefs-verification/tests"))
        with open(os.path.join(closure_root, "crates/singlefs-verification/tests/not_read_by_the_bin.rs"), "w", encoding="utf-8") as handle:
            handle.write("// cargo 编装置读不到 tests/\n")
        got, output, _verdicts = run_items([*replay_arguments, "--selftest-library", closure_store], at=closure_root)
        judge_cell("reuse-closure-unchanged", got == 0 and "现判 0、复用 3" in output, f"退 {got}：{output.strip()[-300:]}")
        # crate 根下 cargo 不读的文件（Verus 必红世界表 verus-must-red-worlds.tsv 这一类）不进装置那一样：新放一份、再改一行，三行都复用
        root_file_outputs = []
        for content in ("名\t第一行\n", "名\t第一行 \n"):
            with open(os.path.join(closure_root, "crates/dep/verus-must-red-worlds.tsv"), "w", encoding="utf-8") as handle:
                handle.write(content)
            root_file_outputs.append(run_items([*replay_arguments, "--selftest-library", closure_store], at=closure_root))
        judge_cell("reuse-crate-root-file-unchanged", all(got == 0 and "现判 0、复用 3" in output for got, output, _verdicts in root_file_outputs),
                   "；".join(f"退 {got}：{output.strip()[-200:]}" for got, output, _verdicts in root_file_outputs))
        # 装置源码（bin 入口那份文件）里 main 调到的一个 fn 改了：装置单元闭包变了，三行全现判、点名那个 fn
        device_path = os.path.join(closure_root, f"crates/singlefs-verification/src/bin/{DEVICE_BINARY}.rs")
        with open(device_path, encoding="utf-8") as handle:
            device_text = handle.read()
        with open(device_path, "w", encoding="utf-8") as handle:
            handle.write(device_text.replace("fn helper() -> u32 {\n    2\n}", "fn helper() -> u32 {\n    3\n}", 1))
        got, output, _verdicts = run_items([*replay_arguments, "--selftest-library", closure_store], at=closure_root)
        judge_cell("reuse-device-source-function-changed", got == 0 and "现判 3、复用 0（现判的原因：装置单元闭包 变了 3）" in output and "改了 fn helper" in output,
                   f"期望装置源码里 main 调到的 fn 改了三行全现判、点名 fn helper，实测退 {got}：{output.strip()[-300:]}")

        # ── 判法单元指纹（replay 的闭包）：算得出、不拒算；replay 里只改与起装置无关的注释不变，改它的代码变 ──
        script_relative = os.path.relpath(os.path.realpath(__file__), REPOSITORY_ROOT)
        base_key = judging_unit_key_of(REPOSITORY_ROOT, script_relative)
        judge_cell("judging-units-no-refusal", base_key.key is not None and not base_key.refusals,
                   f"replay 的判法单元指纹要算得出、不拒算，实测拒算 {list(base_key.refusals)[:3]}")
        copy_root = os.path.join(work, "judging-copy")
        member_files = {line.split("\t")[2].rsplit(":", 1)[0] for line in base_key.units} | set(NOT_JUDGING_FILES) | {script_relative}
        for relative in sorted(member_files):
            os.makedirs(os.path.dirname(os.path.join(copy_root, relative)), exist_ok=True)
            shutil.copyfile(os.path.join(REPOSITORY_ROOT, relative), os.path.join(copy_root, relative))
        copied_script = os.path.join(copy_root, script_relative)
        with open(copied_script, encoding="utf-8") as handle:
            script_text = handle.read()
        # 只在 replay 那一段里改（这份自证自己也写着同样的字，按整份文件找会找错地方）
        replay_start = script_text.index("\ndef replay(")
        replay_end = script_text.index("\ndef ", replay_start + 1)
        replay_text = script_text[replay_start:replay_end]
        comment = "    # 按档分组（㉒ 相同的一组，组的次序照第一次出现），每组再按块起装置、每块带这一组的档参数\n"
        message = 'raise Red("钉住的表本身不对："'
        if replay_text.count(comment) != 1 or replay_text.count(message) != 1:
            judge_cell("judging-units-comment-only", False, f"replay 里那一行注释命中 {replay_text.count(comment)} 次、那一句命中 {replay_text.count(message)} 次（要各恰好一次）")
        else:
            commented_text = replay_text.replace(comment, comment.rstrip("\n") + "（只改了注释的措辞）\n", 1)
            with open(copied_script, "w", encoding="utf-8") as handle:
                handle.write(script_text[:replay_start] + commented_text + script_text[replay_end:])
            commented = judging_unit_key_of(copy_root, script_relative)
            changed_text = replay_text.replace(message, 'raise Red("钉住的表本身就不对："', 1)
            with open(copied_script, "w", encoding="utf-8") as handle:
                handle.write(script_text[:replay_start] + changed_text + script_text[replay_end:])
            changed = judging_unit_key_of(copy_root, script_relative)
            judge_cell("judging-units-comment-only", base_key.key is not None and commented.key == base_key.key and changed.key not in (None, base_key.key),
                       f"原样 {base_key.key}、只改 replay 里一行注释 {commented.key}（拒算 {list(commented.refusals)[:2]}）、改 replay 里一句话 {changed.key}")

        # ── 钉表认档（文件头「档」一段）──
        # r2 档的账：同样三个输入、同样三个变异，原实现逐态行多判一条（counters 后面多 ,j6qn），state= 不变（① 相同）；产物 device_arguments=--profile+r2+--bounds+segment2
        def r2_line(line):
            return line.replace("counters=states", "counters=states,j6qn", 1)

        r2_rows = [[*fields[:11], r2_line(fields[11]), r2_line(fields[12])] if fields[4] != "none" else list(fields) for fields in ledger_rows]
        r2_ledger = os.path.join(root, "research/results/e165-selftest-p2-red-examples.tsv")
        r2_kill_bounds = os.path.join(root, "research/results/e165-selftest-p2-kill-bounds.out")
        write(os.path.relpath(r2_ledger, root), "# r2 档的账\n" + "".join("\t".join(fields) + "\n" for fields in r2_rows))
        write(os.path.relpath(r2_kill_bounds, root), run_product(r2_ledger, "--profile+r2+--bounds+segment2"))
        write(TABLE, pinned_text)
        # 两档：r1 档已钉的三行在，r2 档的同一个（①, ⑥）⑦ 不同，不算冲突、另钉三行（开关 profile-read-as-r1 下当 r1 档、判冲突）
        expect("pin-two-profiles", [r2_ledger], 0, "钉进 3 行（1 份账 4 行：0 行已钉")
        two_profile_text = open(table_path, encoding="utf-8").read()
        new_lines = two_profile_text[len(pinned_text):].strip("\n").split("\n")
        judge_cell("pin-profile-r2", two_profile_text.startswith(pinned_text) and len(new_lines) == 3
                   and all(line.split("\t")[PROFILE_INDEX] == "--profile r2" and len(line.split("\t")) == TABLE_COLUMNS for line in new_lines)
                   and all(line.split("\t")[PROFILE_INDEX] == DEFAULT_PROFILE for line in pinned_text.split("\n") if line.strip() and not line.startswith("#")),
                   f"r2 档的账要钉成 ㉒「--profile r2」的三行、r1 档那三行不动，实测新加的：{[line.split(chr(9))[-1] for line in new_lines]}")
        expect("check-two-profiles", ["--check", ledger, r2_ledger], 0, "账的档：--profile r1 1 份、--profile r2 1 份")
        # 档不对：r2 档那三行的 ㉒ 写成 r1（整行都对），--check 逐行报「钉的档不对」（开关 check-ignores-profile 下判绿）
        write(TABLE, pinned_text + two_profile_text[len(pinned_text):].replace("\t--profile r2\n", "\t--profile r1\n"))
        expect("check-profile-differs", ["--check", r2_ledger], 1, "钉的档不对：表第")
        write(TABLE, two_profile_text)
        # 同一档下 ⑦ 不同：另一份 r2 档的账同一个输入、同一个变异，原实现多判一条（,j7a），一行都不钉、判红（开关 same-profile-conflict-ignored 下照钉）
        conflict_r2_rows = [list(fields) for fields in r2_rows]
        conflict_r2_rows[0][11] = conflict_r2_rows[0][11].replace("counters=states,j6qn", "counters=states,j6qn,j7a", 1)
        conflict_r2_ledger = os.path.join(root, "research/results/e165-selftest-p2b-red-examples.tsv")
        write(os.path.relpath(conflict_r2_ledger, root), "# 另一趟 r2 档的账\n" + "".join("\t".join(fields) + "\n" for fields in conflict_r2_rows))
        write("research/results/e165-selftest-p2b-kill-bounds.out", run_product(conflict_r2_ledger, "--profile+r2"))
        expect("conflict-same-profile", [conflict_r2_ledger], 1, "同一档下同一个（输入, 变异）的原实现逐态行不同，一行都没钉")
        judge_cell("conflict-same-profile-unwritten", open(table_path, encoding="utf-8").read() == two_profile_text, "同档冲突判红之后表还是被写了")
        os.remove(conflict_r2_ledger)
        os.remove(os.path.join(root, "research/results/e165-selftest-p2b-kill-bounds.out"))
        # device_arguments= 认不出的参数（--gate 进不进逐态行不知道）：一行都不钉、判红（开关 profile-arguments-unchecked 下略过它、照钉）
        write(os.path.relpath(r2_kill_bounds, root), run_product(r2_ledger, "--profile+r4+--gate+r2"))
        write(TABLE, pinned_text)
        expect("pin-profile-unknown-argument", [r2_ledger], 1, "认不出的参数 --gate")
        # 产物里取不出档：没有 config 行、config 行没有 device_arguments=、值是空的，一行都不钉、判红、出路写重跑（开关 product-profile-defaulted 下当 --profile r1 照钉）
        for label, product_text in (("without-config", run_product(r2_ledger, None)),
                                    ("without-arguments", run_product(r2_ledger, "--profile+r2").replace(" device_arguments=--profile+r2", "")),
                                    ("empty-arguments", run_product(r2_ledger, ""))):
            write(os.path.relpath(r2_kill_bounds, root), product_text)
            write(TABLE, pinned_text)
            expect(f"pin-product-{label}", [r2_ledger], 1, "取不出这一趟的档")
            judge_cell(f"pin-product-{label}-unwritten", open(table_path, encoding="utf-8").read() == pinned_text, "取不出档判红之后表还是被写了")
        expect("pin-product-howto", [r2_ledger], 1, "重跑那一趟")
        write(os.path.relpath(r2_kill_bounds, root), run_product(r2_ledger, None))
        expect("check-product-without-config", ["--check", r2_ledger], 1, "里没有 E7RESULT name=config 行，取不出这一趟的档")
        write(os.path.relpath(r2_kill_bounds, root), run_product(r2_ledger, "--profile+r2+--bounds+segment2"))
        # 规范写法：默认值不写、只选域与 N 的不进、次序固定（开关 profile-read-as-r1 下全成 r1）
        normalized = []
        for text_given, want in (("--profile+r2+--marker-parity+even+--bounds+segment2+--family+all", "--profile r2"),
                                 ("--table+both+--marker-parity+odd+--profile+r3+--disk-order+normal", "--profile r3 --marker-parity odd --table both"),
                                 ("--bounds+segment2", DEFAULT_PROFILE)):
            try:
                normalized.append(profile_of_device_arguments(text_given) == want)
            except ValueError:
                normalized.append(False)
        judge_cell("profile-normalized", all(normalized), f"三种 device_arguments= 的规范写法逐个对不对：{normalized}")
        # 值是空的不当 --profile r1：认不得（开关 product-profile-defaulted 下交 --profile r1）
        try:
            empty_profile = profile_of_device_arguments("")
        except ValueError:
            empty_profile = None
        judge_cell("profile-empty-refused", empty_profile is None, f"device_arguments= 的值是空的要 raise ValueError，实测交 {empty_profile!r}")
        # 按档重放：桩照档打（r2 档在 counters 后面加 ,j6qn，像装置 r2 档多判 J6Qn），每组带它的档（开关 replay-without-profile 下桩一律按 r1 打，r2 那三行红）
        write(TABLE, two_profile_text)
        profile_stub = os.path.join(work, "profile-stub.sh")
        profile_trace = os.path.join(work, "profile-trace.txt")
        with open(profile_stub, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 照档打的桩：--pinned-cases <表> [档参数…]；⑦ 去掉 ,j6qn 再按档加回（r2 档多判一条）；每起一次把档参数记一行\n"
                         "[[ \"$1\" == --pinned-cases && -f \"$2\" ]] || exit 2\n"
                         "printf '%s\\n' \"${*:3}\" >> \"$PROFILE_TRACE\"\n"
                         "if [[ \" ${*:3} \" == *' --profile r2 '* ]]; then extra=,j6qn; else extra=; fi\n"
                         "grep -v '^#' \"$2\" | grep . | cut -f7 | sed \"s/counters=states,j6qn/counters=states/; s/counters=states/counters=states$extra/\"\n"
                         "echo \"E7RESULT name=pinned_cases rows=$(grep -v '^#' \"$2\" | grep -c .)\"\n")
        open(profile_trace, "w").close()
        expect("replay-two-profiles", ["--replay", "--binary", profile_stub], 0, "按档 2 组：--profile r1 3 行、--profile r2 3 行", {"PROFILE_TRACE": profile_trace})
        judge_cell("replay-profile-arguments", open(profile_trace, encoding="utf-8").read() == "--profile r1\n--profile r2\n",
                   f"要按档起两次、各带 --profile r1 与 --profile r2，实测：{open(profile_trace, encoding='utf-8').read()!r}")
        got, output, _verdicts = run_items(["--list-items"])
        profile_items = [line for line in output.split("\n") if line.startswith("pinned:")]
        judge_cell("list-items-profile", got == 0 and len(profile_items) == 6 and sum(1 for line in profile_items if line.endswith("档 --profile r2")) == 3,
                   f"--list-items 要每行一项、说明带档，实测退 {got}：{profile_items}")
        # 键里有档：表里另有一行与第一行只差 ㉒（r2），只判过第一行，那一行不能拿它的判定（开关 reuse-key-without-profile 下复用）
        twin_profile = list(pinned_rows[0]["fields"])
        twin_profile[PROFILE_INDEX] = "--profile r2"
        write(TABLE, pinned_text.rstrip("\n") + "\n" + "\t".join(twin_profile) + "\n")
        twin_profile_item = f"pinned:{len(pinned_text.rstrip(chr(10)).split(chr(10))) + 1}"
        profile_store = os.path.join(work, "verdict-library-profile")
        run_items([*replay_arguments, "--selftest-library", profile_store, "--item", pinned_items[0]])
        got, output, verdicts = run_items([*replay_arguments, "--selftest-library", profile_store])
        judge_cell("reuse-row-own-profile", got == 0 and verdicts.get(pinned_items[0]) == ("绿", "复用") and verdicts.get(twin_profile_item) == ("绿", "现判"),
                   f"期望 {pinned_items[0]} 复用、只差档的 {twin_profile_item} 现判，实测退 {got}、{verdicts}")
        write(TABLE, pinned_text)
        os.remove(r2_ledger)
        os.remove(r2_kill_bounds)

    # --threads 的缺省：没设 SINGLEFS_THREAD_CAP 取登记文件第一条非注释行的数，设了取它
    with open(os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "heavy-thread-default"), encoding="utf-8") as handle:
        registered_default = next((int(line.split()[0]) for line in handle if line.strip() and not line.lstrip().startswith("#")), None)
    cases += 1
    unset_threads, capped_threads = default_threads({}), default_threads({"SINGLEFS_THREAD_CAP": "7"})
    if registered_default is None or unset_threads != registered_default or capped_threads != 7:
        wrong.append("threads-default-registered")
        print(f"    threads-default-registered 这一格判错：登记 {registered_default}，没设上限得 {unset_threads}，上限 7 得 {capped_threads}")  # gate-lint:detail

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK:
        items = ("pin-twice", "titles-unchecked", "no-recheck-before-write", "check-ignores-missing", "fingerprint-unchecked", "variant-unchecked", "replay-ignores-output",
                     "killed-as-regression", "names-from-today-table", "kill-bounds-optional", "title-missing-ignored",
                     "title-not-compared", "variant-sets-unchecked", "ledger-digest-unchecked",
                     "whole-row-unchecked", "copied-columns-unchecked", "dedupe-by-input-variant",
                     "variant-by-line-number", "append-names-from-today-table", "append-pairing-optional", "progress-silent",
                     "item-filter-ignored", "reuse-key-without-closure", "reuse-key-without-row", "reuse-key-without-mutation-row", "reuse-ignores-key",
                     "reuse-records-red", "reuse-records-whole-cell-only", "red-keeps-green", "unclean-block-records-nothing", "results-paired-by-position",

                     "profile-read-as-r1", "same-profile-conflict-ignored", "check-ignores-profile", "replay-without-profile", "row-columns-unchecked",
                     "profile-arguments-unchecked", "reuse-key-without-profile", "reuse-key-without-judging-units", "row-key-with-ledger-path", "row-key-with-mutation-name", "list-items-pipe-unhandled",
                     "threads-default-cpu-count", "stand-in-unkeyed", "judging-units-raw-text",
                     "spawn-inside-judging-closure", "table-check-globs-research", "check-globs-by-default", "check-kill-bounds-ignored", "profile-form-unchecked", "product-profile-defaulted",
                     "item-key-lines-skipped", "item-done-unrecorded", "key-mismatch-by-item-only", "key-items-ignored")
        # 各开关的那一趟互不依赖、各用各的临时目录：几趟同时起（至多 SELFTEST_BREAK_WORKERS 趟），按开关的次序收退出码、数份数
        def run_break(item):
            return item, subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
                                        env=dict(os.environ, E165_PIN_RED_EXAMPLES_BREAK=item)).returncode

        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(SELFTEST_BREAK_WORKERS, int(default_heavy_threads(os.environ))))) as pool:
            outcomes = list(pool.map(run_break, items))
        if len(outcomes) != len(items):
            breaks_missed.append(f"开关轮收回 {len(outcomes)} 趟、派出 {len(items)} 趟")
        for item, returncode in outcomes:
            if returncode == 0:
                breaks_missed.append(item)
            else:
                breaks_proven += 1
    if wrong:
        report_red(f"e165-pin-red-examples 自证 {cases} 格里 {len(wrong)} 格判错：{' '.join(wrong)}", "按判错那几格的标签找 append / check / replay 里对应的一段修")
        return EXIT_RED
    if breaks_missed:
        report_red(f"弄坏开关打开之后自证照样全对：{' '.join(breaks_missed)}", "那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关")
        return EXIT_RED
    suffix = f"；弄坏开关 {breaks_proven} 个，每个都让至少一格判错" if not BREAK else "（开着弄坏开关，没重跑开关）"
    print(f"  ✓ e165-pin-red-examples 自证 {cases} 格都判对{suffix}")
    return EXIT_GREEN


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证与门禁样本根上认（文件头「弄坏开关」）：命令行第一个参数不是 --selftest、又不是自证起的子进程（自证设了 E165_PIN_RED_EXAMPLES_SELFTEST=1）、
    --root 给的也不是门禁样本根（有 .gate-cells 或 .gate-break-sample，且不是本脚本所在的仓）而 E165_PIN_RED_EXAMPLES_BREAK 非空（不管值是什么），
    退 2 并点名它只在自证里认；在参数解析之前判。门禁 formal-verification 的 table 格与 replay 格喂样本根时带着开关（stage-break-switch-replay 逐条复跑），所以样本根放行。"""
    if not os.environ.get("E165_PIN_RED_EXAMPLES_BREAK") or arguments[:1] == ["--selftest"] or os.environ.get(SELFTEST_VARIABLE) == "1":
        return 0
    if "--root" in arguments[:-1]:
        root = os.path.realpath(arguments[arguments.index("--root") + 1])
        if root != os.path.realpath(DEFAULT_ROOT) and any(os.path.exists(os.path.join(root, marker)) for marker in SAMPLE_ROOT_MARKERS):
            return 0
    print("  ✗ E165_PIN_RED_EXAMPLES_BREAK 只在 --selftest 与门禁样本根上认（弄坏开关只给证红用），这一趟不是自证", file=sys.stderr)
    print("     → 怎么办：去掉 E165_PIN_RED_EXAMPLES_BREAK 再跑；要证红就跑 E165_PIN_RED_EXAMPLES_BREAK=<项> python3 scripts/experiment-apparatus/e165-pin-red-examples.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refusal = refuse_break_outside_selftest(sys.argv[1:])
    if refusal:
        sys.exit(refusal)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
