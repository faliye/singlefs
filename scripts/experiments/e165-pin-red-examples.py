#!/usr/bin/env python3
# admission: always 每一次调都判此刻的红例账、钉住的表与变异表，上一次的结论不替这一次作保
# run-condition: none 追加、补钉与 --check 只读写点名的文件；--replay 要的 cargo 与装置由它自己判，缺了判红或报本次未跑
"""E165 红例钉进快档：把全量那一档红例账里的每个红例按输入原样钉进 .claude/gate.d/e165-pinned-red-examples.tsv，并判钉得全不全、今天还绿不绿。

用户 2026-10-01 定「全量形式化红了的例子 可以加到快速形式化中」「这个应该在每一次跑完之后就加入 门禁来控制比较好」
（调度记录 records/2026-10-01-里程碑三形式化接入.md 第三节）。表的格式写在表自己的文件头。

用法（<根> 默认取这份脚本往上两级的仓；--table 默认 <根>/.claude/gate.d/e165-pinned-red-examples.tsv）：
  档（㉒）：出这份账的那一趟装置的档参数。取同一趟 kill-bounds 产物（配法见下）第一行 E7RESULT name=config 的 device_arguments=（参数用 + 连），
      只认装置 split_off_the_profile_options 认的四个：--profile、--marker-parity、--table、--disk-order（钉住的红例逐行重放时装置只认这四个，别的不进逐态行）；
      --bounds、--family、--only、--max-n 只选跑哪些域与 N，不进档；别的参数（例 --gate、--max-u）认不出进不进逐态行，判红、这份账一行都不钉。
      写成规范的一串：「--profile <档名>」，后面只接不是默认值的那几个（--marker-parity odd、--table scan|both、--disk-order reversed，依这个次序），空格分隔；
      产物没有 device_arguments=（装置有 --profile 之前的几趟）或没有 config 行是 --profile r1（装置不给 --profile 取 r1）。
      同一个（①, ⑥）在不同档下各钉一行，不算冲突；同一档下 ⑦ 不同照旧判红。① 不带档（装置 --pinned-cases 拿 ②③ 与逐态行的 state= 重算 ①，与档无关）。
  e165-pin-red-examples.py [--root 根] [--table 表] [--kill-bounds 产物]… <红例账…>
      追加：账里还没钉的行追加进表，一行 22 段：自己的 ①–⑧ 后面原样接账那一行的 13 段（⑨–㉑），再接这份账的档（㉒）。
      已钉过的不重复：表里有一行 ⑨–㉑ 与这一行账逐字节相同、㉒ 等于这份账的档，且 ⑥ 等于这一次换出的名字或 ⑤ 就是这一行账。
      同一个（①, ⑥, ㉒）而 ⑨ 起不同的（另一趟的账，⑱ 首红与否或 ⑨⑩ 的变体与表行不同）另钉一行。
      账里没有反例状态的行（序号 none，驱动报 no_single_state_differs）钉不了，只数不钉。
      ⑥ 写「<表>#<变异名>」，换名与 --check 同一套：每份账配同一趟的 kill-bounds 产物，不给 --kill-bounds 按文件名配（与 --check 相同），
      给了就逐份对应（第 i 个产物配第 i 份账，个数要相等；第二段驱动收尾时这样调）；配对的四样（见 --check 一节）有一样不过就判红、一行都不钉，不退回按今天的行号换名；
      配上了取产物里这个变体那一行（item= 等于账第 1 列、table_row= 等于账第 2 列的第一个表行）的
      table= 与 title=，table= 要是这种变体的表（tsv-<行号> 是 crates/mutations.tsv，Z<n> 是 scripts/mutations/e165_replay_bounded_enumeration_custom.tsv），
      在今天的那张表里找第 1 列（空格记成 _）等于 title= 的数据行，恰好一行就用它的变异名：那一趟之后变异表增删过行也对得上，不按今天的行号换。
      table= 不对、按名字找不到、找到不止一行的，逐行报出来、不钉，别的行照钉，退 1。
      同一个（①, ⑥, ㉒）在表里（或这一次前面的账里）的 ⑦ 与账里的不同：一行都不钉、判红。
      写：读表时记 sha256，同目录临时文件写完、改名换上之前再算一次，变了就不写（别的会话刚写过，重跑）；写完回读，新钉的每一行都要在。
  e165-pin-red-examples.py --check [--root 根] [--table 表] [<红例账…>]
      门禁 pinned-red-examples-complete 格：不给账就取 <根>/research/results/e165-*-red-examples.tsv 全部。
      作废的账登记进 <根>/.claude/gate.d/e165-void-red-example-ledgers.tsv（一行三段：账的路径、理由不少于 8 个字、实验页里写明作废的那一处 文件:行号；表不在当一行都没登记），
      登记了的跳过（给了账也跳过）；登记表先判：缺段或有段空着、理由太短、出处不是「文件:行号」、登记的账不在、
      同一份登记两次、登记的账配得上一份有收尾行且 ledger_sha256 等于它此刻 sha256 的同一趟产物（那一趟是完整的），有一样就判红、一份账都不判，
      出路照表头改那一行（完整的那一种：删掉那一行）。出处只判写法，不核那一行写没写作废。账都登记作废了退 77。
      被同档后一趟取代的完整账登记进 <根>/.claude/gate.d/e165-superseded-red-example-ledgers.tsv（一行四段：被取代的账、取代它的账、理由不少于 8 个字、
      实验页里写明取代关系的那一处 文件:行号；表不在当一行都没登记），登记了的跳过（给了账也跳过），取代它的账照常判钉全（给了账而没给它的，补进这一次判的账）。
      这张表先判：缺段或有段空着、理由太短、出处不是「文件:行号」、两份账有一份不在、两份是同一份、被取代的登记两次或也登记作废、
      取代它的账自己登记作废或登记成被取代；两份账各自同一趟 kill-bounds 产物的 device_arguments= 照「档」一段取档，取不到或档不同；
      被取代的账每一行按（第 3 列域, 第 4 列 N, 第 5 列枚举序号, ①）在取代它的账里一行配一行（① 是第 3、4 列与第 12 列 state= 算的输入指纹，同一个键几行就要几行），
      配不上的；有一样就判红、一份账都不判，出路照表头改那一行。成功那句报「跳过作废账 N 份、被取代的账 M 份：逐个列名（被取代的写取代它的是哪份）」。
      账里每一行（有反例状态的）都要钉着：
      表里有一行 ① 等于这一行算出的指纹、⑦ 等于账的第 12 列、⑨–㉑ 用制表符接起来与账那一行逐字节相同、㉒ 等于这份账的档（按档判钉全），
      且 ⑤ 就是这一行账或 ⑥ 是出这份账的那一趟给这个变体的名字。同一个输入钉着、只是 ⑨ 起不同的，逐行报「没整行钉」；
      整行都对、只是 ㉒ 不是这份账的档的，逐行报「钉的档不对」（出路：照这份账的档重钉）。
      那一趟的名字不按今天的变异表换（表在跑完之后删一行，后面的 tsv-<行号> 就全错一位），取同一趟的 kill-bounds 产物：
      账 <目录>/<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out（E165 第二段驱动 e165-replay-mutation-kill-bounds.sh 一趟写这一对）。
      配对四样都要过，有一样不过就判红：产物在；产物的收尾行 E7RESULT name=verdict 在、它的 ledger_sha256= 等于这份账此刻的 sha256（驱动写完账才写这一行）；
      账里有行的变体与产物里 red_examples= 大于 0 的变体是同一批、每个变体的账行数等于它的 red_examples=；
      账的每一行按（第 1 列变体, 第 2 列的第一个表行）在产物里有 E7RESULT name=kill_bound 那一行（item=、table_row=），带 title=。
      ⑥ 的表（# 之前）等于那一行的 table=、变异名（# 之后，空格记成 _）等于它的 title=，才算同一个变异。
      门禁那一格照原样起（--check --root <根>），按账的文件名找产物，不多传参数；--check 不收 --kill-bounds。
      没钉全的出路逐份给 python3 scripts/e165-pin-red-examples.py <账>：追加按同一趟产物的 title= 到今天的变异表里按名字找变异，
      那一趟之后变异表增删过行也钉得进；按名字找不到或找到几行的逐行报出来、不钉，不钉进错的名字。
      表自己也判（--check、--replay 与追加都先判）：每行 22 段（8 段的旧行报「没整行抄红例账」、21 段的报「没写档」，出路都给 --backfill）、① 是 64 位十六进制、
      ① 等于拿 ②③⑦ 重算的指纹、⑪⑫⑬ 与 ②③④ 相同、⑳㉑ 与 ⑦⑧ 相同（不同的点名哪一行哪一段，与装置 --pinned-cases 同一种报法）、
      ㉒ 是规范写法的档、（⑥, ⑨–㉑ 的整行, ㉒）不重复。少一行判红，出路给补进去的命令。一份账都没有退 77。
  e165-pin-red-examples.py --backfill [--root 根] [--table 表]
      补钉：表里 8 段的旧行（整行抄红例账之前钉的）按 ⑤（<账文件名>:<行号>）回 <根>/research/results/<账文件名> 取那一行，原样接成 ⑨–㉑；
      8 段与 21 段的行（钉档之前钉的）都补 ㉒：⑤ 点的那份账按文件名配同一趟的 kill-bounds 产物，照上面「档」一段取。
      取回的要是 13 段的账行、第 3、4、5、12、13 段与这一行的 ②③④⑦⑧ 逐字相同；账不在、没有那一行、段数不对或对不上、配的产物不在或 device_arguments= 认不出的逐行报出来、不填，
      取得到的照补；有取不到的退 1。写法与追加相同（读表时记 sha256、换上之前复核、写完回读补过的每一行）。没有要补的行退 0。
  e165-pin-red-examples.py --replay [--root 根] [--table 表] [--binary 可执行文件] [--memory-cap 上限|none] [--threads N] [--target 编译目录]
      门禁 pinned-red-examples-replay 格：先判每行 ⑥ 点名的变异（<表>#<名>，几个用 + 连）还在它的表里（第 1 列等于那个名字的数据行），
      再用装置 e165_replay_bounded_enumeration --pinned-cases <表> 在今天的实现上逐行判：stdout 里以「E7RESULT name=state 」起头的行
      条数等于表的数据行数、第 i 行与第 i 个数据行的 ⑦ 逐字节相同、装置退 0（约定：每行一行结果，有行对不上或指纹漂了退非 0）。
      按档分组起装置：㉒ 相同的行一组，每组按块起，命令是「<装置> --pinned-cases <块表> <㉒ 按空格切开>」（例 --profile r2）；
      块表只留这一块的行、每行只留 ①–㉑（装置只认 21 段，㉒ 不交给它），别的数据行换成 #、行号不变。
      不给 --binary 时：装置源码（crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs）里没有 --pinned-cases 就退 77
      （变异那一半照判，红了照红）；有就 cargo build --release --locked -p singlefs-checker-tier --bin e165_replay_bounded_enumeration
      （CARGO_TARGET_DIR 默认 ${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/singlefs-e165-pinned-target）再跑。
      装置经 scripts/run-with-memory-cap.sh（上限默认 $GATE_E165_PINNED_MEMORY_MAX，再默认 2G）与 capped.sh N 起；--memory-cap none 直接起（样本的桩用）；
      经包装起而包装自己退 245 或 250–254 的，判「没跑成」：退 1，✗ 行写「没跑成：内存包装 scripts/run-with-memory-cap.sh 退 <码>」，
      → 照包装文件头「退出码」一段，不指向变异行或实现；
      装置被信号停了的（直接起时 subprocess 交负的信号号，经包装或 bash 起时退 128 + 信号号、不在 250–254 里，例 137 = 外面 kill -9），判红，
      ✗ 行写「装置被信号停了」、这一次的输出不算判定，→ 照包装文件头「退出码」一段（撞自己上限的由包装认成 250，外面停的同样退 137），不写修实现；
      --binary 以 .sh 结尾的经 bash 起（样本目录里的桩不带执行位）。
      编装置与跑装置时每过 300 秒（PROGRESS_INTERVAL_SECONDS，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」定的进度间隔）往 stderr 打一行
      「E165_PINNED_PROGRESS 挂钟 <秒> 秒：<在做什么> 还在跑；装置已打 n/m 行结果」（编装置那一步不带行数）；判定照旧等它跑完、按全部输出判。
  e165-pin-red-examples.py --list-items [--root 根] [--table 表]
      逐行打可点名的项「项<TAB>格<TAB>说明」：complete（pinned-red-examples-complete，一项），钉表每个数据行一项 pinned:<表里的行号>
      （pinned-red-examples-replay，说明是 ⑤、⑥ 与「档 <㉒>」，没写档的写「没写档」）。只读钉表，不编、不起装置；表不在判红退 1。
  逐项点名与按条复用（只给 --replay；别的模式给 --item 退 2）：
      --item <项>（可给多次，取并集）只重放点名的行，项名写错或缺项名退 2 并列出可点名的项；点了的行照样先判表本身与点名的变异还在。
        点名单跑一行与它在整格里跑的判定相同（自证 item-alone-equals-whole）：装置按块起，一块只留这一块的几行（别的数据行换成 #，行号不变），
        一行的判定只看它自己打的那一行与 ⑦ 相同、装置的报错里没点它的行号；同一块里别的行红不连累它。
      --block-rows N（默认 DEFAULT_BLOCK_ROWS）一块交给装置几行；一块判完，那一块里判绿的行当场落盘（装置退 0、行数对得上的块才记），
        被打断或包装没跑成之后同一条命令重跑，判过的块复用、只补没判的。
      --verdict-library <目录> 给了就按条记判定（判定库 scripts/lib_verdict_library.py）：每一行的键相同就复用，一行都不用现判时不编装置、不起重放；
        --no-reuse 只记不读。不给 --verdict-library 不读不写。complete（--check）每趟现判、不复用。
      键分两列（判定库文件头「键分两列」）：
        判法版本：这份脚本与 lib_verdict_library.py 两份内容的摘要（judge_version）；变了整格每一行都现判，原因写「判法版本 变了」。
        输入指纹（逐样摘要按名字排好再取 sha256）——共用的一样：装置（e165_replay_bounded_enumeration 的 cargo 依赖闭包，scripts/admission.py 的 cargo_closure_input_paths 现算，
          crate 根下 cargo 不读的文件不在里面，例 crates/singlefs-replay-prefix/verus-must-red-worlds.tsv；
          input_manifest 逐文件取内容，加仓根 Cargo.toml、Cargo.lock 与 cargo -V、rustc -V；给了 --binary 的另加那份可执行文件的内容，这时闭包算不出就记一句原因；
          不给 --binary 而闭包算不出判红）；变了整格每一行都现判。
        输入指纹里每一行另加：这一行（22 段原样）、档（㉒，单列一样，变了原因写「档 变了」）、引的变异行（⑥ 点名的每一条变异在 crates/mutations.tsv 或 scripts/mutations/e165_replay_bounded_enumeration_custom.tsv 里那一行原样）。
      只记绿。每行一行「· pinned:<行号>：绿|红，现判（<原因>）|复用（<记下的日期与一句判定>）」，末了「· 按条：现判 N、复用 M（现判的原因：…）」；
        环境里有 FORMAL_ITEM_TALLY_FILE 的（门禁给的）另往那份文件追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」。
  e165-pin-red-examples.py --selftest
      在临时仓里造账、变异表、产物与桩装置，逐格判；弄坏开关逐个打开，每个都要让至少一格判错。
退出码：0 绿（追加：钉好了或没有要钉的；补钉：都补上了或没有要补的）；1 红；2 用法错；77 本次未跑（--check 没有账、--replay 装置不认 --pinned-cases）。

弄坏开关 E165_PIN_RED_EXAMPLES_BREAK=<项>[,<项>…]（只给证红用；--selftest 与门禁样本在开着时都必须判错）：
  pin-twice              追加不去重                        pin-again 那一格判错
  titles-unchecked       追加按 title 找不到变异时，退回按今天的行号换名、不报   titles-mismatch、pin-by-title-not-found、pin-by-title-not-found-detail 三格判错
  no-recheck-before-write  改名换上之前不复核 sha256       race 那一格判错
  check-ignores-missing  --check 不报没钉的行               check-missing 那一格（样本 pinned-red-examples-complete-red）判绿
  fingerprint-unchecked  --check 不拿 ②③⑦ 重算 ①         check-tampered 那一格判绿
  variant-unchecked      --replay 不判变异还在不在          replay-variant-missing 那一格（样本 pinned-red-examples-replay-variant-red）判绿
  replay-ignores-output  --replay 只看装置退出码，不比 ⑦    replay-mismatch 那一格（样本 pinned-red-examples-replay-red）判绿
  wrapper-exit-as-replay-red  --replay 不认包装自己的 245、250–254，照装置退非 0 报回归   replay-wrapper-exit（自证里的假包装退 252）、
                                                     replay-wrapper-245（假包装退 245，显存不够）两格判错
  killed-as-regression   --replay 不认装置被信号停了，照装置退非 0 报回归、出路写修实现
                                                     replay-killed（桩装置 kill -9 自己）、replay-wrapper-137（假包装退 137）两格判错
  names-from-today-table --check 照今天的变异表换 tsv-<行号>，不认产物里的名字   check-shifted-table（自证里删一行、末尾加一行、行号错位的合成表）判红
  kill-bounds-optional   --check 账配的产物不在也不红，照今天的变异表换名     check-kill-bounds-missing 那一格判绿
  title-missing-ignored  --check 产物里那个变体没有那一行或没有 title= 也不红，照今天的变异表换名   check-title-missing 那一格判绿
  title-not-compared     --check 产物里有 title= 就算钉着，不比 ⑥               check-title-other-variant 那一格判绿
  variant-sets-unchecked --check 不比账与产物的变体与行数                   check-variant-sets-differ 那一格判绿
  ledger-digest-unchecked  --check 不核产物收尾行记的账 sha256、不认没收尾     check-ledger-digest、check-run-unfinished 两格判绿
作废红例账登记表（.claude/gate.d/e165-void-red-example-ledgers.tsv）的三项，开关一行、下一行起写打开之后判错的自证格与门禁样本：
  void-ledgers-ignored   --check 不读登记表，登记作废的账照判
      自证 check-void-skipped、check-void-skipped-given、check-void-all-skipped 判错（没收尾的账照判、判红）；样本 pinned-red-examples-complete-void-green 判红
  void-rows-unchecked    登记表的缺段、理由太短、出处不是「文件:行号」、账不在、登记两次都不判
      自证 check-void-missing、check-void-short-reason、check-void-columns、check-void-place、check-void-duplicate 判绿；样本 pinned-red-examples-complete-void-missing-red 判绿
  void-complete-accepted   登记的账配得上完整的同一趟产物也不判
      自证 check-void-complete、check-void-complete-howto 判错
被取代红例账登记表（.claude/gate.d/e165-superseded-red-example-ledgers.tsv）的四项，开关一行、下一行起写打开之后判错的自证格与门禁样本：
  superseded-replacement-pin-unchecked  被取代的账跳过时不把取代它的账补进这一次判的账（只点名被取代的账时，取代它的账没钉全也判不到）
      自证 check-superseded-given、check-superseded-replacement-unpinned 判错（只给被取代的账：退 77，没判取代它的账）
  superseded-coverage-unchecked  不核被取代的账每一行按（域, N, 枚举序号, ①）在取代它的账里有对应行
      自证 check-superseded-coverage、check-superseded-coverage-count、check-superseded-coverage-state 判绿；样本 pinned-red-examples-complete-superseded-uncovered-red 判绿
  superseded-profile-unchecked   不核两份账同档（各自同一趟产物的 device_arguments=）
      自证 check-superseded-profile、check-superseded-profile-unreadable 判绿
  superseded-rows-unchecked      登记表的缺段、理由太短、出处写法、账不在、两份同一份、登记两次、与作废表重登、取代它的账自己登记了都不判（解析不了的行略过）
      自证 check-superseded-columns、check-superseded-short-reason、check-superseded-place、check-superseded-missing、check-superseded-replacement-missing、
      check-superseded-self、check-superseded-duplicate、check-superseded-also-void、check-superseded-replacement-void、check-superseded-chain 判错
整行抄红例账（一行 21 段）的几项，开关一行、下一行起写打开之后判错的自证格与门禁样本（样本在 .claude/gate.d/fixtures/checker-tier-formal-verification.sh/ 下）：
  whole-row-unchecked    --check 不比 ⑨ 起的整行，只认 ①、⑦ 与 ⑤ / ⑥（整行抄之前的判法）
      自证 check-whole-row-differs 判绿；样本 pinned-red-examples-complete-whole-row-red 判绿
  old-rows-accepted      表里 8 段的旧行照收、不报没整行抄（装置改收 21 段之前的判法）
      自证 check-old-row、replay-old-row、replay-old-row-howto 判错；样本 pinned-red-examples-replay-old-row-red 判绿
  copied-columns-unchecked  不核 ⑪⑫⑬ 与 ②③④、⑳㉑ 与 ⑦⑧
      自证 replay-copied-domain-differs、replay-copied-original-differs 判绿；样本 pinned-red-examples-replay-copied-columns-red 判绿
  dedupe-by-input-variant  追加按（①, ⑥）去重：另一趟的账 ⑨ 起不同也算已钉（整行抄之前的判法）
      自证 pin-whole-row-variant 判错（该新钉 1 行，钉了 0 行），接着 check-whole-row-variant 判红
  backfill-unverified    --backfill 按 ⑤ 取回的账行不核第 3、4、5、12、13 段就接上
      自证 backfill-mismatched-origin、backfill-mismatched-origin-detail 判错（把别的账行接了上去，不报取不到原行）
  补钉写表的复核与追加共用 no-recheck-before-write：打开之后自证 backfill-race、backfill-race-wrote 同样判错
  variant-by-line-number  追加配上了产物也按今天的行号换名、再核 title 相同（按名字找之前的判法：那一趟之后变异表增删过行就一行都不钉）
      自证 pin-by-title-shifted-table、check-by-title-shifted-table、pin-by-title-same-names 判错（该钉进 3 行，换出的名字与 title 不同、一行都没钉）；
      pin-by-title-not-found、pin-by-title-not-found-detail、pin-by-title-ambiguous、pin-by-title-ambiguous-detail 判错（不报按名字找不到 / 找到几行，同名的照行号钉了进去）
追加按文件名配同一趟产物（与 --check 同一套）的两项：
  append-names-from-today-table  追加配上了产物也不用它的名字，照今天的变异表按行号换名（不给 --kill-bounds 时的旧判法）
      自证 pin-deleted-row-names 判错（今天的表删了一行、末尾加了一行，钉进的是错位那两行的名字）；
      titles-mismatch、pin-by-title-not-found、pin-by-title-not-found-detail、pin-by-title-ambiguous、pin-by-title-ambiguous-detail、pin-by-title-same-names 同样判错
  append-pairing-optional  追加配不上产物（产物不在、收尾行记的账 sha256 不同）也不红，照今天的变异表按行号换名照钉
      自证 pin-kill-bounds-missing、pin-kill-bounds-missing-wrote、pin-ledger-digest、pin-explicit-kill-bounds-digest、pin-deleted-row、pin-deleted-row-names 判错
--replay 的进度行：
  progress-silent        编装置、跑装置时不打进度行（跑完之前什么都看不到）
      自证 replay-progress 判错（假时钟下装置跑过进度间隔，一行 E165_PINNED_PROGRESS 都没有）
--replay 的逐项点名与按条复用：
  item-filter-ignored    --item 不起作用、照重放全部（写错的项名也不退 2）
      自证 item-select-one、item-unknown 判错；reuse-row-own-key 跟着判错
  reuse-key-without-closure  键里不带装置的 cargo 依赖闭包（装置依赖的 crate 改了照旧复用）
      自证 reuse-closure-changed 判错
  （闭包减掉 crate 根下 cargo 不读的文件归 scripts/admission.py：它的 ADMISSION_BREAK=closure-keeps-crate-root-files 下
      自证 reuse-crate-root-file-unchanged 判错：依赖 crate 根下放一份、改一行 verus-must-red-worlds.tsv，三行全现判）
  reuse-key-without-row  键里不带这一行自己（同一个 ⑥、同一个 ⑦ 的两行共用一个键，只判过一行就拿它的判定当另一行的）
      自证 reuse-row-own-key 判错
  reuse-key-without-judge-version  判法版本一律交空串（这份脚本或判定库改了照旧复用）
      自证 reuse-judge-version-changed 判错
  list-items-pipe-unhandled  --list-items 不照 SIGPIPE 惯例退（接 | head 时打 BrokenPipeError 的回溯、退 1）
      自证 list-items-closed-pipe 判错
  reuse-key-without-mutation-row  键里不带 ⑥ 引的变异行（变异表里那一行改了照旧复用）
      自证 reuse-mutation-row-changed 判错
  reuse-ignores-key      这一行判绿过就复用，不比键
      自证 prune-after-change 判错（剪枝自证：桩改了之后开着复用与关掉复用逐行判定不同）；reuse-mutation-row-changed、reuse-device-changed、reuse-closure-changed 跟着判错
  reuse-records-red      红的也记成绿
      自证 reuse-red-not-recorded 判错
  reuse-records-whole-cell-only  一块判完不当场落盘，整格判完、一行红都没有才一起记（被打断或有一行红，判过的全不留）
      自证 reuse-resume-after-kill 判错（一块一行，X 那一块被 kill -9，第二趟同一条命令要只判 X）；reuse-red-not-recorded 跟着判错
钉表认档（㉒）的几项，开关一行、下一行起写打开之后判错的自证格与门禁样本：
  profile-read-as-r1     取档不读产物的 device_arguments=，一律当 --profile r1（r2 档的账钉成 r1 档）
      自证 pin-profile-r2、pin-two-profiles、backfill-profile 判错
  same-profile-conflict-ignored  追加不判同一档下同一个（①, ⑥）⑦ 不同，照钉
      自证 conflict、conflict-same-profile 判错
  check-ignores-profile  --check 不比 ㉒，钉成别的档也算钉着
      自证 check-profile-differs 判绿；样本 pinned-red-examples-complete-profile-red 判绿
  replay-without-profile --replay 起装置不带 ㉒ 的档参数（装置一律按 r1 档重放）
      自证 replay-two-profiles、replay-profile-arguments 判错；样本 pinned-red-examples-replay-profiles-green 判红
  profile-column-unchecked  表里 21 段的行（没写档）照收、当 --profile r1
      自证 check-profile-missing、replay-profile-missing 判错；样本 pinned-red-examples-replay-profile-missing-red 判绿
  profile-arguments-unchecked  device_arguments= 里认不出的参数（例 --gate）照略过，不判红
      自证 pin-profile-unknown-argument 判错
  reuse-key-without-profile  按条复用的键不带档（「这一行」不算 ㉒、不单列「档」）
      自证 reuse-row-own-profile 判错（只差 ㉒ 的两行，只判过一行就拿它的判定当另一行的）
"""
import glob
import hashlib
import os
import re
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
from lib_verdict_library import Tally, VerdictLibrary, digest_file, digest_text  # noqa: E402
import importlib.util  # noqa: E402
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
DEFAULT_ROOT = os.path.dirname(SCRIPT_DIRECTORY)
TABLE = ".claude/gate.d/e165-pinned-red-examples.tsv"
LEDGER_GLOB = "research/results/e165-*-red-examples.tsv"
LEDGER_SUFFIX, KILL_BOUNDS_SUFFIX = "-red-examples.tsv", "-kill-bounds.out"
# 作废红例账登记表（--check 跳过登记的账）：一行三段「账的路径<TAB>理由<TAB>实验页里写明作废的那一处 文件:行号」，写法在表自己的文件头
VOID_LEDGERS_TABLE = ".claude/gate.d/e165-void-red-example-ledgers.tsv"
VOID_LEDGER_COLUMNS, VOID_REASON_MINIMUM_CHARACTERS = 3, 8
VOID_PLACE_FORM = re.compile(r"^\S.*:[1-9][0-9]*$")
# 被取代红例账登记表（--check 跳过登记的被取代账、取代它的账照常判钉全）：一行四段「被取代的账<TAB>取代它的账<TAB>理由<TAB>实验页里写明取代关系的那一处 文件:行号」，
# 写法在表自己的文件头；理由字数与出处写法与作废表同一套（VOID_REASON_MINIMUM_CHARACTERS、VOID_PLACE_FORM）
SUPERSEDED_LEDGERS_TABLE = ".claude/gate.d/e165-superseded-red-example-ledgers.tsv"
SUPERSEDED_LEDGER_COLUMNS = 4
VOID_LEDGERS_HOWTO = (f"照 {VOID_LEDGERS_TABLE} 的文件头改那一行：账不在的，路径写错就改对、账已删就删掉那一行；缺段、理由不到 8 个字、作废出处不是「文件:行号」的补全；"
                      "登记的账配得上一份有收尾行、ledger_sha256 对得上的产物的，那一趟是完整的、不是作废的：删掉那一行，它照常要钉全；"
                      f"它被同档后一趟的账取代了的，删掉那一行、改登记进 {SUPERSEDED_LEDGERS_TABLE}（写法在那张表的文件头）")
SUPERSEDED_LEDGERS_HOWTO = (f"照 {SUPERSEDED_LEDGERS_TABLE} 的文件头改那一行：缺段、理由不到 8 个字、出处不是「文件:行号」的补全；账不在的路径写错就改对、账已删就删掉那一行；"
                            "两份写成同一份、登记两次、也登记在作废表里的删掉多的那一行；取代它的账自己登记了作废或被取代的，第二段改成真正取代它、照常钉全的那一份；"
                            "两份账的档不同（各自同一趟产物的 device_arguments=）或取不到档的，不是同档取代：删掉那一行，被取代的那份照常钉全"
                            "（python3 scripts/e165-pin-red-examples.py <被取代的账>；同档 ⑦ 冲突照追加的出路办）；"
                            "被取代的账有行在取代它的账里没有同一个（域, N, 枚举序号, ①）的，取代它的账没覆盖它：删掉那一行，被取代的那份照常钉全，或交主 agent 定重跑出一份覆盖它的账再登记")
CRATES_TABLE = "crates/mutations.tsv"
CUSTOM_TABLE = "scripts/mutations/e165_replay_bounded_enumeration_custom.tsv"
DEVICE_SOURCE = "crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs"
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
TABLE_PROBLEMS_HOWTO = ("8 段的旧行（没整行抄红例账）与 21 段的行（没写档）跑 python3 scripts/e165-pin-red-examples.py --backfill 补成 22 段"
                        "（按 ⑤ 回账里取原行、回同一趟的 kill-bounds 产物取档）；别的是手改坏了：表只由这份脚本写，从 git 找回再钉")
# 档（㉒）：装置 split_off_the_profile_options 认的四个参数进档；只选域与 N 的四个不进；别的认不出、判红（文件头「档」一段）
PROFILE_OPTIONS = ("--profile", "--marker-parity", "--table", "--disk-order")
PROFILE_OPTION_DEFAULTS = {"--marker-parity": "even", "--table": "direct", "--disk-order": "normal"}
SELECTION_OPTIONS = ("--bounds", "--family", "--only", "--max-n")
DEFAULT_PROFILE = "--profile r1"
PROFILE_FORM = re.compile(r"--profile r[1-9][0-9]*( --marker-parity odd)?( --table (?:scan|both))?( --disk-order reversed)?")
PROFILE_HOWTO = ("档取同一趟 kill-bounds 产物第一行 E7RESULT name=config 的 device_arguments=：装置钉住的红例逐行重放只认 --profile、--marker-parity、--table、--disk-order，"
                 "--bounds、--family、--only、--max-n 只选域与 N；别的参数先查装置（crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs 的 split_off_the_profile_options）"
                 "认不认它、进不进逐态行，进的加进这份脚本的 PROFILE_OPTIONS（不进的加进 SELECTION_OPTIONS）再钉，不手填 ㉒")
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_NOT_RUN = 0, 1, 2, 77
BREAK = {item for item in os.environ.get("E165_PIN_RED_EXAMPLES_BREAK", "").split(",") if item}
# --replay 一块交给装置几行（--block-rows 改）：一块判完那几行当场落盘，被打断之后同一条命令只补没判的块
DEFAULT_BLOCK_ROWS = 32
# 门禁给的一份文件：--replay 判完往里追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」（lib_verdict_library.Tally.append_to），门禁整道相加
ITEM_TALLY_FILE_VARIABLE = "FORMAL_ITEM_TALLY_FILE"
_ADMISSION_MODULE = []
MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run-with-memory-cap.sh")
# 内存包装自己的结局（run-with-memory-cap.sh 文件头「退出码」一段）：经它起时装置这一次没跑成，输出不算判定
WRAPPER_OWN_EXITS = {245: "显存不够（起止之间内核报了 NV_ERR_NO_MEMORY）", 250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上",
                     253: "超过限时", 254: "被总上限挤掉"}
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
    """device_arguments= 的值（参数用 + 连）→ 规范的档（文件头「档」一段）；认不出 raise ValueError（说明哪一个）。
    开关 profile-read-as-r1 下一律交 --profile r1；profile-arguments-unchecked 下认不出的参数略过。"""
    if "profile-read-as-r1" in BREAK:
        return DEFAULT_PROFILE
    tokens = [token for token in (text or "").split("+") if token]
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
    """同一趟 kill-bounds 产物 → 规范的档：第一行 E7RESULT name=config 的 device_arguments=；没有那一段或没有 config 行是 --profile r1。
    认不出 raise Red。"""
    for line in open(product, encoding="utf-8").read().split("\n"):
        if line.startswith("E7RESULT name=config "):
            match = re.search(r" device_arguments=(\S*)", line)
            try:
                return profile_of_device_arguments(match.group(1) if match else "")
            except ValueError as problem:
                raise Red(f"{os.path.basename(product)} 的 device_arguments= 认不出档：{problem}", PROFILE_HOWTO)
    return DEFAULT_PROFILE


def row_profile(fields):
    """钉表一行的档（㉒）；21 段的行（没写档）只在开关 profile-column-unchecked 下当 --profile r1，平时交 None。"""
    if len(fields) == TABLE_COLUMNS:
        return fields[PROFILE_INDEX]
    return DEFAULT_PROFILE if len(fields) == DEVICE_COLUMNS and "profile-column-unchecked" in BREAK else None


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


RUN_PAIRING_HOWTO = ("账与同一趟的产物按文件名配对（<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out，驱动 scripts/e165-replay-mutation-kill-bounds.sh 一趟写这一对）："
                     "产物或账被删、被改了从 git 找回同一趟的原样（git log --all --diff-filter=D --name-only -- '*<文件名>*'，git show <提交>:<路径>）；"
                     "找不回的重跑那一趟再钉；没有同一趟产物的账（手造的、改过名的）不放进 research/results/")


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
    """钉表一行 ⑨–㉑ 用制表符接起来：整行抄的那一行账（8 段的旧行交空串；㉒ 档不算在里面）。"""
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
        rows.append({"number": number, "fields": raw.split("\t")})
    return text, rows


def table_problems(rows):
    problems, seen = [], set()
    for row in rows:
        fields = row["fields"]
        if len(fields) == TABLE_OWN_COLUMNS:
            if "old-rows-accepted" in BREAK:
                continue
            problems.append(f"第 {row['number']} 行没整行抄红例账（{TABLE_OWN_COLUMNS} 段的旧格式，要 {TABLE_COLUMNS} 段）")
            continue
        if len(fields) == DEVICE_COLUMNS and "profile-column-unchecked" not in BREAK:
            problems.append(f"第 {row['number']} 行没写档（{DEVICE_COLUMNS} 段，要 {TABLE_COLUMNS} 段：㉒ 是出这一行的那一趟的档）")
            continue
        if len(fields) not in (TABLE_COLUMNS, DEVICE_COLUMNS):
            problems.append(f"第 {row['number']} 行是 {len(fields)} 段，要 {TABLE_COLUMNS} 段（自己的 {TABLE_OWN_COLUMNS} 段后面接红例账一行的 {LEDGER_COLUMNS} 段，再接 ㉒ 档）")
            continue
        if len(fields) == TABLE_COLUMNS and not PROFILE_FORM.fullmatch(fields[PROFILE_INDEX]):
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


# ── 补钉：8 段的旧行按 ⑤ 回账里取原行，接成 21 段 ──

def backfill(root, table):
    text, rows = read_table(table)
    digest_at_read = file_digest(table)
    old_rows = [row for row in rows if len(row["fields"]) in (TABLE_OWN_COLUMNS, DEVICE_COLUMNS)]
    if not old_rows:
        print(f"  ✓ {os.path.relpath(table, root)} 的 {len(rows)} 行都不是 {TABLE_OWN_COLUMNS} 段或 {DEVICE_COLUMNS} 段的旧行，没有要补的")
        return EXIT_GREEN
    ledger_directory = os.path.dirname(LEDGER_GLOB)
    ledger_lines, profiles, filled, unrecoverable = {}, {}, {}, []
    for row in old_rows:
        fields = row["fields"]
        name, _colon, number = fields[4].rpartition(":")
        if not name or not number.isdigit() or "/" in name:
            unrecoverable.append((row, f"⑤「{fields[4]}」不是 <红例账文件名>:<行号>"))
            continue
        if name not in ledger_lines:
            path = os.path.join(root, ledger_directory, name)
            ledger_lines[name] = open(path, encoding="utf-8").read().split("\n") if os.path.isfile(path) else None
            # 档：⑤ 点的那份账按文件名配同一趟的产物，取它的 device_arguments=（文件头「档」一段）；取不到记原因
            product = paired_kill_bounds(path)
            if product is None or not os.path.isfile(product):
                profiles[name] = (None, f"⑤ 点的红例账配的 kill-bounds 产物 {shown_path(product, root) if product else '（文件名不以 ' + LEDGER_SUFFIX + ' 收尾）'} 不在，取不到档")
            else:
                try:
                    profiles[name] = (run_profile(product), None)
                except Red as red:
                    profiles[name] = (None, red.message)
        lines = ledger_lines[name]
        if lines is None:
            unrecoverable.append((row, f"⑤ 点的红例账 {ledger_directory}/{name} 不在"))
            continue
        profile, profile_problem = profiles[name]
        if profile is None:
            unrecoverable.append((row, profile_problem))
            continue
        if len(fields) == DEVICE_COLUMNS:
            filled[row["number"]] = fields + [profile]
            continue
        if not 1 <= int(number) <= len(lines):
            unrecoverable.append((row, f"{ledger_directory}/{name} 没有第 {number} 行（共 {len(lines)} 行）"))
            continue
        line = lines[int(number) - 1]
        ledger_fields = line.split("\t")
        if line.startswith("#") or len(ledger_fields) != LEDGER_COLUMNS:
            unrecoverable.append((row, f"{ledger_directory}/{name} 第 {number} 行不是 {LEDGER_COLUMNS} 段的账行（{len(ledger_fields)} 段）"))
            continue
        if "backfill-unverified" not in BREAK:
            differing = [f"账第 {ledger_column} 段与 {COLUMN_MARKS[own_column - 1]}" for ledger_column, own_column in COPIED_PAIRS
                         if ledger_fields[ledger_column - 1] != fields[own_column - 1]]
            if differing:
                unrecoverable.append((row, f"{ledger_directory}/{name} 第 {number} 行与这一行对不上（{'、'.join(differing)} 不同）：账在钉过之后被改过，或 ⑤ 的行号错位"))
                continue
        filled[row["number"]] = fields + ledger_fields + [profile]
    for row, reason in unrecoverable:
        print(f"     取不到原行：第 {row['number']} 行（⑤ {row['fields'][4]}）：{reason}")  # gate-lint:detail
    unrecoverable_howto = ("逐行照上面的原因找回那份账钉这一行时的原样（git log --all --diff-filter=D --name-only -- '*<账文件名>*'，git show <提交>:<路径>，"
                           "放回 research/results/）再跑 --backfill；找不回的这一行交主 agent 定（重跑出那份账的那一趟再钉），不手填、不照别的账行填")
    if not filled:
        raise Red(f"{os.path.relpath(table, root)} 里 {len(old_rows)} 行 {TABLE_OWN_COLUMNS} 段或 {DEVICE_COLUMNS} 段的旧行一行都取不到原行或档，一行都没补", unrecoverable_howto)
    new_text = "\n".join("\t".join(filled[number]) if number in filled else line for number, line in enumerate(text.split("\n"), 1))
    new_rows = [{"number": row["number"], "fields": filled.get(row["number"], row["fields"])} for row in rows]
    problems = table_problems([row for row in new_rows if len(row["fields"]) not in (TABLE_OWN_COLUMNS, DEVICE_COLUMNS)])
    if problems:
        raise Red(f"补完之后的表本身不对，一行都没写：" + "；".join(problems[:5]), TABLE_PROBLEMS_HOWTO)
    write_table(table, new_text, digest_at_read)
    _text, after = read_table(table)
    present = {(row["number"], "\t".join(row["fields"])) for row in after}
    lost = [number for number, fields in filled.items() if (number, "\t".join(fields)) not in present]
    if lost:
        raise Red(f"写完回读：补过的 {len(filled)} 行里 {len(lost)} 行不在原来的行号上（第 {'、'.join(str(number) for number in lost[:5])} 行）",
                  "别的会话在改名换上之后又写过；重跑 --backfill")
    if unrecoverable:
        raise Red(f"补了 {len(filled)} 行（回读都在）；{len(unrecoverable)} 行取不到原行或档，没补、留着原来的段数", unrecoverable_howto)
    whole_rows = sum(1 for row in old_rows if row["number"] in filled and len(row["fields"]) == TABLE_OWN_COLUMNS)
    by_profile = {}
    for fields in filled.values():
        by_profile[fields[PROFILE_INDEX]] = by_profile.get(fields[PROFILE_INDEX], 0) + 1
    profile_text = "、".join(f"{profile} {count} 行" for profile, count in sorted(by_profile.items()))
    print(f"  ✓ 补钉：{len(filled)} 行都补成 {TABLE_COLUMNS} 段（{whole_rows} 行 {TABLE_OWN_COLUMNS} 段的旧行按 ⑤ 取回账行；档按 ⑤ 那份账同一趟的产物取：{profile_text}），"
          f"回读都在；表里现有 {len(after)} 行")
    return EXIT_GREEN


def current_digest(path):
    return file_digest(path)


# ── 格一：账都钉着 ──

def read_void_ledgers(root):
    """作废红例账登记表：交（{账的绝对路径: 仓根起的路径}, [问题]）。表不在交（{}, []）。
    判：每行三段都不空、理由不少于 VOID_REASON_MINIMUM_CHARACTERS 个字、作废出处写成「文件:行号」、登记的账在、同一份不登记两次，
    登记的账配不上一份有收尾行、ledger_sha256 等于它此刻 sha256 的同一趟产物（配得上说明那一趟是完整的）。
    开关 void-rows-unchecked 下前四样不判（缺段的行照收），void-complete-accepted 下最后一样不判。"""
    path = os.path.join(root, VOID_LEDGERS_TABLE)
    if not os.path.isfile(path):
        return {}, []
    registered, problems = {}, []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if (len(fields) != VOID_LEDGER_COLUMNS or not all(field.strip() for field in fields)) and "void-rows-unchecked" not in BREAK:
            problems.append(f"第 {number} 行是 {len(fields)} 段、{sum(1 for field in fields if not field.strip())} 段空着（要 {VOID_LEDGER_COLUMNS} 段：账的路径、理由、作废出处 文件:行号，制表符分隔，都不许空）")
            continue
        relative, reason, place = (fields + ["", "", ""])[:VOID_LEDGER_COLUMNS]
        ledger = os.path.normpath(os.path.join(root, relative))
        if "void-rows-unchecked" not in BREAK:
            if len(reason.strip()) < VOID_REASON_MINIMUM_CHARACTERS:
                problems.append(f"第 {number} 行的理由「{reason.strip()}」不到 {VOID_REASON_MINIMUM_CHARACTERS} 个字")
            if not VOID_PLACE_FORM.match(place.strip()):
                problems.append(f"第 {number} 行的作废出处「{place.strip()}」不是「文件:行号」")
            if not os.path.isfile(ledger):
                problems.append(f"第 {number} 行登记的账 {relative} 不在")
                continue
            if ledger in registered:
                problems.append(f"第 {number} 行登记的账 {relative} 前面已登记过")
                continue
        product = paired_kill_bounds(ledger)
        if product and os.path.isfile(product) and os.path.isfile(ledger) and "void-complete-accepted" not in BREAK:
            _records, ledger_digest = read_kill_bounds(product)
            if ledger_digest == file_digest(ledger):
                problems.append(f"第 {number} 行登记的账 {relative} 配得上一份有收尾行、ledger_sha256 对得上的产物 {shown_path(product, root)}：那一趟是完整的，不是作废的")
        registered[ledger] = relative
    return registered, problems


def coverage_rows(ledger_path):
    """账每一行的（第 3 列域, 第 4 列 N, 第 5 列枚举序号, ①）→ 这个键的行号（按出现次序）。
    ① 是 input_fingerprint(第 3 列, 第 4 列, 第 12 列)：钉表算 ① 用的那几列；没有反例状态的行（第 12 列空）记 None。账写坏了 raise Red。"""
    rows = {}
    for row in read_ledger(ledger_path):
        fields = row["fields"]
        rows.setdefault((fields[2], fields[3], fields[4], input_fingerprint(fields[2], fields[3], fields[11])), []).append(row["number"])
    return rows


def read_superseded_ledgers(root, void_registered):
    """被取代红例账登记表：交（{被取代的账的绝对路径: (仓根起的路径, 取代它的账的绝对路径)}, [问题]）。表不在交（{}, []）。
    判（文件头 --check 一段）：写法（四段不空、理由、出处、两份都在、不是同一份、不登记两次、不与作废表重登、取代它的账自己没登记），
    两份同档、被取代的账每一行在取代它的账里一行配一行。开关 superseded-rows-unchecked 下写法不判（解析不了的行略过），
    superseded-profile-unchecked 下不判档，superseded-coverage-unchecked 下不判覆盖。"""
    path = os.path.join(root, SUPERSEDED_LEDGERS_TABLE)
    if not os.path.isfile(path):
        return {}, []
    rows_unchecked = "superseded-rows-unchecked" in BREAK
    entries, problems, parsed = {}, [], []
    for number, raw in enumerate(open(path, encoding="utf-8").read().split("\n"), 1):
        if not raw.strip() or raw.startswith("#"):
            continue
        fields = raw.split("\t")
        if len(fields) != SUPERSEDED_LEDGER_COLUMNS or not all(field.strip() for field in fields):
            if not rows_unchecked:
                problems.append(f"第 {number} 行是 {len(fields)} 段、{sum(1 for field in fields if not field.strip())} 段空着（要 {SUPERSEDED_LEDGER_COLUMNS} 段："
                                "被取代的账、取代它的账、理由、取代出处 文件:行号，制表符分隔，都不许空）")
            continue
        old_relative, new_relative, reason, place = (field.strip() for field in fields)
        old, new = os.path.normpath(os.path.join(root, old_relative)), os.path.normpath(os.path.join(root, new_relative))
        row_problems = []
        if len(reason) < VOID_REASON_MINIMUM_CHARACTERS:
            row_problems.append(f"第 {number} 行的理由「{reason}」不到 {VOID_REASON_MINIMUM_CHARACTERS} 个字")
        if not VOID_PLACE_FORM.match(place):
            row_problems.append(f"第 {number} 行的取代出处「{place}」不是「文件:行号」")
        for which, relative, ledger in (("被取代的账", old_relative, old), ("取代它的账", new_relative, new)):
            if not os.path.isfile(ledger):
                row_problems.append(f"第 {number} 行{which} {relative} 不在")
        if old == new:
            row_problems.append(f"第 {number} 行被取代的账与取代它的账是同一份 {old_relative}")
        if old in entries:
            row_problems.append(f"第 {number} 行被取代的账 {old_relative} 前面已登记过")
        if old in void_registered:
            row_problems.append(f"第 {number} 行被取代的账 {old_relative} 也登记在作废表 {VOID_LEDGERS_TABLE} 里")
        if new in void_registered:
            row_problems.append(f"第 {number} 行取代它的账 {new_relative} 登记在作废表 {VOID_LEDGERS_TABLE} 里：取代它的账要照常钉全")
        if not rows_unchecked:
            problems.extend(row_problems)
        present = os.path.isfile(old) and os.path.isfile(new) and old != new
        parsed.append((number, old_relative, old, new_relative, new, present))
        entries.setdefault(old, (old_relative, new))
    for number, old_relative, old, new_relative, new, present in parsed:
        if new in entries and not rows_unchecked:
            problems.append(f"第 {number} 行取代它的账 {new_relative} 自己也登记成被取代（被 {shown_path(entries[new][1], root)} 取代）：取代它的账要照常钉全，不能再被跳过")
        if not present:
            continue
        if "superseded-profile-unchecked" not in BREAK:
            profiles = []
            for relative, ledger in ((old_relative, old), (new_relative, new)):
                product = paired_kill_bounds(ledger)
                if product is None or not os.path.isfile(product):
                    shown = shown_path(product, root) if product else f"（文件名不以 {LEDGER_SUFFIX} 收尾）"
                    problems.append(f"第 {number} 行：{relative} 配的 kill-bounds 产物 {shown} 不在，取不到档")
                    profiles.append(None)
                    continue
                try:
                    profiles.append(run_profile(product))
                except Red as red:
                    problems.append(f"第 {number} 行：{red.message}")
                    profiles.append(None)
            if None not in profiles and profiles[0] != profiles[1]:
                problems.append(f"第 {number} 行：被取代的账 {old_relative} 那一趟的档是「{profiles[0]}」，取代它的账 {new_relative} 那一趟是「{profiles[1]}」：档不同，不是同档取代")
        if "superseded-coverage-unchecked" not in BREAK:
            try:
                old_rows, new_rows = coverage_rows(old), coverage_rows(new)
            except Red as red:
                problems.append(f"第 {number} 行：{red.message}")
                continue
            uncovered = []
            for key, numbers in old_rows.items():
                extra = len(numbers) - len(new_rows.get(key, []))
                if extra > 0:
                    uncovered.extend((line, key) for line in numbers[-extra:])
            if uncovered:
                shown = "、".join(f"第 {line} 行（域 {key[0]}、N {key[1]}、序号 {key[2]}）" for line, key in uncovered[:5])
                problems.append(f"第 {number} 行：被取代的账 {old_relative} 有 {len(uncovered)} 行在取代它的账 {new_relative} 里没有对应行"
                                f"（按域、N、枚举序号与 ① 一行配一行）：{shown}")
    return entries, problems


def check(root, table, ledgers):
    void_ledgers, void_problems = ({}, []) if "void-ledgers-ignored" in BREAK else read_void_ledgers(root)
    if void_problems:
        for problem in void_problems[:10]:
            print(f"     作废账登记：{problem}")  # gate-lint:detail
        raise Red(f"作废红例账登记表 {VOID_LEDGERS_TABLE} 有 {len(void_problems)} 处不对", VOID_LEDGERS_HOWTO)
    superseded, superseded_problems = read_superseded_ledgers(root, void_ledgers)
    if superseded_problems:
        for problem in superseded_problems[:10]:
            print(f"     被取代账登记：{problem}")  # gate-lint:detail
        raise Red(f"被取代红例账登记表 {SUPERSEDED_LEDGERS_TABLE} 有 {len(superseded_problems)} 处不对", SUPERSEDED_LEDGERS_HOWTO)
    if not ledgers:
        ledgers = sorted(glob.glob(os.path.join(root, LEDGER_GLOB)))
    skipped = [ledger for ledger in ledgers if os.path.normpath(ledger) in void_ledgers]
    skipped_superseded = [ledger for ledger in ledgers if os.path.normpath(ledger) in superseded and os.path.normpath(ledger) not in void_ledgers]
    ledgers = [ledger for ledger in ledgers if os.path.normpath(ledger) not in void_ledgers and os.path.normpath(ledger) not in superseded]
    # 被取代的账跳过的前提是取代它的账钉全：没在这一次判的账里的补进来（开关 superseded-replacement-pin-unchecked 下不补）
    replacing = {}
    for ledger in skipped_superseded:
        replacement = superseded[os.path.normpath(ledger)][1]
        replacing.setdefault(replacement, []).append(ledger)
        if ("superseded-replacement-pin-unchecked" not in BREAK and os.path.isfile(replacement)
                and replacement not in {os.path.normpath(judged) for judged in ledgers}):
            ledgers.append(replacement)
    skipped_names = ([shown_path(ledger, root) for ledger in skipped]
                     + [f"{shown_path(ledger, root)}（被 {shown_path(superseded[os.path.normpath(ledger)][1], root)} 取代）" for ledger in skipped_superseded])
    skipped_text = (f"跳过作废账 {len(skipped)} 份、被取代的账 {len(skipped_superseded)} 份"
                    + ("：" + "、".join(skipped_names) if skipped_names else ""))
    if not ledgers:
        raise NotRun(f"{LEDGER_GLOB} 除了登记作废的（{skipped_text}）一份红例账都没有：这一格无对象可判")
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
        run_names = run_variant_names(root, ledger, ledger_rows)
        if run_names is not None:
            paired += 1
        if "names-from-today-table" in BREAK:
            run_names = None
        # 按档判钉全：这份账的档取同一趟产物的 device_arguments=（产物不在只在开关 kill-bounds-optional 下走到这里，那时不比档）
        product = paired_kill_bounds(ledger)
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
        commands = [f"python3 scripts/e165-pin-red-examples.py {shown_path(ledger, root)}"
                    for ledger in sorted({ledger for ledger, _origin in missing})]
        whole_note = f"（其中 {whole_row_differs} 行表里钉着同一个输入，但没有一行 ⑨ 起与账那一行逐字节相同）" if whole_row_differs else ""
        if profile_differs:
            whole_note += f"（其中 {profile_differs} 行整行钉着、㉒ 不是这份账那一趟的档）"
        unpinned_replacements = [ledger for ledger in sorted({os.path.normpath(ledger) for ledger, _origin in missing}) if ledger in replacing]
        for replacement in unpinned_replacements:
            whole_note += (f"（{shown_path(replacement, root)} 取代了登记的被取代账 {'、'.join(shown_path(ledger, root) for ledger in replacing[replacement])}："
                           "取代它的账钉全了，被取代的才跳过）")
        raise Red(f"{len(ledgers)} 份红例账 {counted} 行里 {len(missing)} 行没钉进 {os.path.relpath(table, root)}{whole_note}",
                  f"补进去，一份账一条：{'；'.join(commands)}（已钉的不重复，⑨ 起不同的另钉一行整行抄；追加按文件名配同一趟的产物、按它的 title= 到今天的变异表里按名字找变异，按名字找不到或找到几行的逐行报、不钉，照它的出路办；档取同一趟产物的 device_arguments=，同一个输入与变异在别的档下钉着的照钉一行这一档的）")
    profiles_text = "、".join(f"{profile} {count} 份" for profile, count in sorted(ledger_profiles.items()))
    print(f"  ✓ {len(ledgers)} 份红例账 {counted} 行都钉在 {os.path.relpath(table, root)}，钉着的那一行 ⑨–㉑ 与账那一行逐字节相同、㉒ 是那份账的档（{without_example} 行没有反例状态、不钉；"
          f"{paired} 份账与同一趟的 kill-bounds 产物配对都过，⑥ 按那一趟的名字认；账的档：{profiles_text}）；表里 {len(rows)} 行，① 都与 ②③⑦ 重算的相同，⑪⑫⑬⑳㉑ 与 ②③④⑦⑧ 相同；{skipped_text}")
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


def admission_module():
    """scripts/admission.py（装置的 cargo 依赖闭包 cargo_closure_input_paths、逐文件清单 input_manifest、工具链 toolchain_line），第一次用到时才读进来。"""
    if not _ADMISSION_MODULE:
        specification = importlib.util.spec_from_file_location("admission_for_e165_pin_red_examples", os.path.join(SCRIPT_DIRECTORY, "..", "gate", "admission.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _ADMISSION_MODULE.append(module)
    return _ADMISSION_MODULE[0]


def pinned_item(row):
    return f"pinned:{row['number']}"


def list_items(root, table):
    """逐行打「项<TAB>格[<TAB>说明]」：complete（pinned-red-examples-complete）一项，钉表每个数据行一项 pinned:<表里的行号>（说明是 ⑤、⑥ 与「档 <㉒>」）。只读钉表，不编、不起装置。"""
    _text, rows = read_table(table)
    print("complete\tpinned-red-examples-complete")
    for row in rows:
        fields = row["fields"]
        note = f"{fields[4]} {fields[5]}" if len(fields) >= 6 else f"{len(fields)} 段"
        if len(fields) >= 6:
            note += f" 档 {fields[PROFILE_INDEX]}" if len(fields) == TABLE_COLUMNS else " 没写档"
        print(f"{pinned_item(row)}\tpinned-red-examples-replay\t{note}")
    return EXIT_GREEN


# 自证造「判法版本变了」用：里面放一个值时 judge_version 交它（只有自证放，门禁与命令行从不放）
JUDGE_VERSION_FOR_SELFTEST = []


def judge_version():
    """判法版本：这份脚本与它 import 的判定库，两份内容的摘要（判定库的另一列，不进输入指纹）。弄坏开关 reuse-key-without-judge-version 下一律交空串。"""
    if "reuse-key-without-judge-version" in BREAK:
        return ""
    if JUDGE_VERSION_FOR_SELFTEST:
        return JUDGE_VERSION_FOR_SELFTEST[0]
    return digest_text(digest_file(os.path.realpath(__file__)) + digest_file(os.path.join(SCRIPT_DIRECTORY, "..", "crash", "lib_verdict_library.py")))


def device_inputs_digest(root, binary):
    """装置那一样：e165_replay_bounded_enumeration 的 cargo 依赖闭包（admission.py 的 cargo_closure_input_paths 现算）逐文件内容、仓根 Cargo.toml 与 Cargo.lock、
    工具链（cargo -V 与 rustc -V）；给了 --binary 的另加那份可执行文件的内容（样本与自证的桩），这时闭包算不出就记一句算不出的原因。
    不给 --binary 而闭包算不出：抛 Red（键罩不全就不许复用）。弄坏开关 reuse-key-without-closure 下闭包不进。"""
    admission = admission_module()
    parts = [digest_file(binary)] if binary else []
    if "reuse-key-without-closure" not in BREAK:
        try:
            paths = admission.cargo_closure_input_paths(root, "singlefs-checker-tier", DEVICE_BINARY) + ["Cargo.toml", "Cargo.lock"]
            _manifest, fingerprint, _count = admission.input_manifest(root, paths, [admission.toolchain_line(root)])
            parts.append(fingerprint)
        except (admission.RegistrationError, admission.InputManifestError) as error:
            if not binary:
                raise Red(f"算不出装置 {DEVICE_BINARY} 的 cargo 依赖闭包（{error}），按条判定的键罩不全",
                          "先让 python3 scripts/admission.py paths <根> checker-tier-formal-verification.sh:pinned-red-examples-replay 列得出路径；不给判定库（去掉 --verdict-library）照旧整格现判")
            parts.append(digest_text(f"算不出闭包：{error}"))
    return digest_text("\n".join(parts))


def mutation_rows_text(root, variant):
    """⑥ 点名的每一条变异（<表>#<名>，几个用 + 连）在它的表里那一行原样（第 1 列等于名字的数据行，几行都取）；表不在、行不在各记一句。"""
    pieces = []
    for part in variant.split("+"):
        relative, _hash, name = part.partition("#")
        path = os.path.join(root, relative)
        if not os.path.isfile(path):
            pieces.append(f"{part}\t表不在")
            continue
        lines = [raw for raw in open(path, encoding="utf-8").read().split("\n") if raw.strip() and not raw.startswith("#") and raw.split("\t")[0] == name]
        pieces.append(f"{part}\t" + ("\n".join(lines) if lines else "行不在"))
    return "\n".join(pieces)


def row_components(root, row, shared):
    components = list(shared)
    # 档进键：「这一行」带 ㉒，另单列一样「档」（开关 reuse-key-without-profile 下两处都不带）
    row_text = "\t".join(row["fields"][:DEVICE_COLUMNS] if "reuse-key-without-profile" in BREAK else row["fields"])
    if "reuse-key-without-row" not in BREAK:
        components.append(("这一行", digest_text(row_text)))
    if "reuse-key-without-profile" not in BREAK:
        components.append(("档", digest_text(row_profile(row["fields"]) or "")))
    if "reuse-key-without-mutation-row" not in BREAK:
        components.append(("引的变异行", digest_text(mutation_rows_text(root, row["fields"][5]))))
    return components


def lookup(store, item, components):
    if "reuse-ignores-key" in BREAK and store.directory is not None and store.reuse:
        return store.lookup_by_item_only(item)
    return store.lookup(item, components)


def report_item(item, verdict, how, seconds=None):
    """每项一行：项名、判定（绿 / 红）、现判还是复用、现判的用了几秒（一块里几行共用一次装置，秒数是那一块的）。"""
    print(f"  · {item}：{verdict}，{how}" + (f"，{seconds:.1f} 秒（这一块）" if seconds is not None else ""))  # gate-lint:detail


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
    包装自己的结局与被信号停了照旧抛 Red（这一块之前判完的已经落盘）。开关 replay-without-profile 下不带档参数。"""
    block_table = os.path.join(work, f"pinned-block-{block[0]['number']}.tsv")
    with open(block_table, "w", encoding="utf-8") as handle:
        handle.write(blocked_table(table_text, {row["number"] for row in block}))
    # 以 .sh 结尾的（样本的桩）经 bash 起：样本目录里的文件不带执行位（script-modes.sh 不看 fixtures/）
    profile_arguments = [] if "replay-without-profile" in BREAK else profile.split(" ")
    command = (["bash", binary] if binary.endswith(".sh") else [binary]) + [DEVICE_FLAG, block_table, *profile_arguments]
    wrapped = options["memory_cap"] != "none"
    if wrapped:
        command = ["bash", MEMORY_CAP_RUNNER, options["memory_cap"],
                   "bash", os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "capped.sh"), str(options["threads"]), *command]
    environment = dict(os.environ, RUN_WITH_MEMORY_CAP_PARALLEL_REASON="门禁里逐行重放钉住的红例，与同一趟别的格各用各的上限",
                       RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="门禁的上限按钉住的表最大时给，表只涨不缩")
    completed = run_reporting_progress(command, root, environment, f"装置 {DEVICE_FLAG} 逐行重放钉住的 {len(block)} 行（档 {profile}，第 {block[0]['number']} 行起这一块）", len(block))
    if wrapped and completed.returncode in WRAPPER_OWN_EXITS and "wrapper-exit-as-replay-red" not in BREAK:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())
        raise Red(f"没跑成：内存包装 scripts/run-with-memory-cap.sh 退 {completed.returncode}（{WRAPPER_OWN_EXITS[completed.returncode]}，上限 {options['memory_cap']}），"
                  f"钉住的 {len(block)} 行这一次的输出不算判定{'（' + tail[:300] + '）' if tail else ''}",
                  "照 scripts/run-with-memory-cap.sh 文件头「退出码」一段办：250 是装置要的比上限多，把 GATE_E165_PINNED_MEMORY_MAX（或 --memory-cap）设大再跑；"
                  "251、252、254 先跑 bash scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；"
                  "253 是超过限时：把 RUN_WITH_MEMORY_CAP_TIME_LIMIT 拿掉或放宽再跑；245 是同一段时间内核报了显存不够：nvidia-smi 看谁在用卡，等它跑完再重跑这一格；"
                  "不拿掉包装跑，不改钉住的表、变异行或实现")
    signal_number = -completed.returncode if completed.returncode < 0 else completed.returncode - 128 if 128 < completed.returncode <= 128 + 64 else None
    if signal_number is not None and "killed-as-regression" not in BREAK:
        tail = " | ".join(line for line in completed.stderr.strip().split("\n")[-4:] if line.strip())
        raise Red(f"装置被信号停了（退 {completed.returncode}，信号 {signal_number}）：钉住的 {len(block)} 行这一次没判完，输出不算判定{'（' + tail[:300] + '）' if tail else ''}",
                  "照 scripts/run-with-memory-cap.sh 文件头「退出码」一段办：撞了这一条自己的上限由包装认成 250，外面 kill -9 同样退 137——这一次是外面停的："
                  "查是谁在那一刻停了它（同一时刻的 ps、journalctl --user、看门狗的记录），那件事不再发生之后重跑这一格；不拿掉包装跑，不改钉住的表、变异行或实现")
    return completed


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
    missing_variants = []
    if "variant-unchecked" not in BREAK:
        tables = {}
        for row in rows:
            for part in row["fields"][5].split("+"):
                relative, _hash, name = part.partition("#")
                if relative not in tables:
                    path = os.path.join(root, relative)
                    tables[relative] = set(mutation_names(root, relative).values()) if os.path.isfile(path) else None
                if tables[relative] is None or name not in tables[relative]:
                    missing_variants.append(f"第 {row['number']} 行：{part}")
    if missing_variants:
        for item in missing_variants[:10]:
            print(f"     不在了：{item}")  # gate-lint:detail
        raise Red(f"{len(rows)} 行里 {len(missing_variants)} 处点名的「改回去就红」的变异不在它的表里了",
                  "变异行被删或改了名：找回那一行（改法被悄悄撤回时它要先响），确是改名就照新名改这张表（要写明为什么）")
    store, tally = VerdictLibrary(options.get("verdict_library"), "pinned-red-examples-replay", judge_version(), reuse=options.get("reuse", True)), Tally()
    binary = options["binary"]
    shared = [("装置", device_inputs_digest(root, binary))] if store.directory else []
    pending = []
    for row in rows:
        components = row_components(root, row, shared) if shared else []
        reused, reason = lookup(store, pinned_item(row), components)
        tally.count(reused, reason)
        if reused:
            report_item(pinned_item(row), "绿", f"复用（{reason}）")
        else:
            pending.append((row, components, reason))
    named = f"（--item 点名的 {len(rows)} 行）" if wanted else ""
    if not pending:
        print(f"  · 按条：{tally.text()}")  # gate-lint:detail
        tally.append_to(os.environ.get(ITEM_TALLY_FILE_VARIABLE), "pinned-red-examples-replay")
        print(f"  ✓ 钉住的 {len(rows)} 行{named}：点名的变异都还在它们的表里；每一行都复用上一次判绿的判定，没编装置、没起重放；按条{tally.text()}")
        return EXIT_GREEN
    if binary is None:
        source = os.path.join(root, DEVICE_SOURCE)
        if not os.path.isfile(source) or DEVICE_FLAG not in open(source, encoding="utf-8").read():
            raise NotRun(f"{len(rows)} 行点名的变异都还在；装置 {DEVICE_SOURCE} 还不认 {DEVICE_FLAG}（第 4 批 3-6 才加），逐行重放这一半本次未跑")
        environment = dict(os.environ, CARGO_TARGET_DIR=options["target"])
        command = ["nice", "-n", "19", "cargo", "build", "--release", "--locked", "-p", "singlefs-checker-tier", "--bin", DEVICE_BINARY]
        completed = run_reporting_progress(command, root, environment, f"编装置 {DEVICE_BINARY}")
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
            for index, (row, components, reason) in enumerate(block):
                got_line = results[index] if index < len(results) else None
                matched = got_line == row["fields"][6] or "replay-ignores-output" in BREAK
                named_in_errors = f"第 {row['number']} 行（" in completed.stderr
                # 这一行的判定只看它自己：打的那一行与 ⑦ 相同、装置的报错里没点它的行号（同一块里别的行红不连累它，点名单跑与整格里跑判定相同）；
                # 落盘还要这一块干净（装置退 0、行数对得上），不干净的这一块一行都不记
                green = matched and not named_in_errors
                recordable = green and (block_clean or "replay-ignores-output" in BREAK)
                if not matched:
                    mismatches.append(f"第 {row['number']} 行（{row['fields'][4]}，档 {profile}）：要 {row['fields'][6][:160]}，装置打的是 {(got_line or '（没打）')[:160]}")
                report_item(pinned_item(row), "绿" if green else "红", f"现判（{reason}）", seconds)
                if recordable or "reuse-records-red" in BREAK:
                    if "reuse-records-whole-cell-only" in BREAK:
                        held.append((pinned_item(row), components))
                    else:
                        failed_record = store.record(pinned_item(row), components, "装置逐行打的结果与 ⑦ 逐字节相同")
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
        store.record(item, components, "装置逐行打的结果与 ⑦ 逐字节相同")
    print(f"  ✓ 钉住的 {len(rows)} 行{named}：点名的变异都还在它们的表里；装置 {DEVICE_FLAG} 逐行打的结果都与 ⑦ 逐字节相同"
          f"（按档 {len(groups)} 组：{'、'.join(f'{profile} {len(entries)} 行' for profile, entries in groups.items())}；{len(blocks)} 块、每块至多 {block_rows} 行，每块带它那一组的档）；按条{tally.text()}")
    return EXIT_GREEN


# ── 命令行 ──

def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest()
    options = {"root": DEFAULT_ROOT, "table": None, "kill_bounds": [], "binary": None, "threads": int(os.environ.get("SINGLEFS_THREAD_CAP") or os.cpu_count() or 1),
               "memory_cap": os.environ.get("GATE_E165_PINNED_MEMORY_MAX") or "2G",
               "target": os.path.join(os.environ.get("GATE_CROSS_RUN_TMPDIR") or os.environ.get("TMPDIR") or "/tmp", "singlefs-e165-pinned-target"),
               "items": [], "verdict_library": None, "reuse": True, "block_rows": DEFAULT_BLOCK_ROWS}
    mode, ledgers, position = "append", [], 0
    valued = {"--root": "root", "--table": "table", "--binary": "binary", "--memory-cap": "memory_cap", "--threads": "threads", "--target": "target",
              "--verdict-library": "verdict_library", "--block-rows": "block_rows"}
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("--check", "--replay", "--backfill", "--list-items"):
            mode = argument[2:]
            position += 1
        elif argument == "--no-reuse":
            options["reuse"] = False
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
    if options["kill_bounds"] and mode != "append":
        report_red(f"--{mode} 不收 --kill-bounds：--check 按文件名给每份账配同一趟的产物"
                   "（<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out），--replay 与 --backfill 用不上产物",
                   "去掉 --kill-bounds 再跑；要判的账照位置参数点名")
        return EXIT_USAGE
    if options["kill_bounds"] and len(options["kill_bounds"]) != len(ledgers):
        report_red(f"追加给了 {len(options['kill_bounds'])} 个 --kill-bounds、{len(ledgers)} 份红例账：给就逐份对应，第 i 个产物配第 i 份账",
                   "不给 --kill-bounds（按文件名配：<前缀>-red-examples.tsv 配同一目录的 <前缀>-kill-bounds.out），或每份账给一个同一趟的产物")
        return EXIT_USAGE
    if options["items"] and mode != "replay":
        report_red(f"--item 只给 --replay 用（--{mode} 不收）：pinned-red-examples-complete 只有一项（complete）",
                   "去掉 --item；可点名的项：python3 scripts/e165-pin-red-examples.py --list-items")
        return EXIT_USAGE
    if mode == "list-items":
        if ledgers:
            report_red("--list-items 不收红例账", "去掉位置参数再跑")
            return EXIT_USAGE
        if "list-items-pipe-unhandled" not in BREAK:
            # 接 | head 时读的一头先关了：照 SIGPIPE 的惯例安静退出（被 SIGPIPE 停，shell 里退 141），不打 BrokenPipeError 的回溯
            signal.signal(signal.SIGPIPE, signal.SIG_DFL)
        try:
            return list_items(root, table)
        except Red as red:
            report_red(red.message, red.howto)
            return EXIT_RED
    if mode == "backfill" and ledgers:
        report_red("--backfill 不收红例账：每行按自己的 ⑤ 回 research/results/ 取原行",
                   "去掉位置参数再跑：python3 scripts/e165-pin-red-examples.py --backfill")
        return EXIT_USAGE
    if mode == "append" and not ledgers:
        report_red("追加要至少一份红例账", "例：python3 scripts/e165-pin-red-examples.py research/results/<某一趟>-red-examples.tsv")
        return EXIT_USAGE
    missing = [ledger for ledger in ledgers + options["kill_bounds"] if not os.path.isfile(ledger)]
    if missing:
        report_red(f"点名的文件不在：{' '.join(missing)}", "路径相对当前目录；红例账在 research/results/ 下")
        return EXIT_USAGE
    try:
        if mode == "append":
            return append(root, table, [os.path.abspath(ledger) for ledger in ledgers], [os.path.abspath(product) for product in options["kill_bounds"]])
        if mode == "check":
            return check(root, table, [os.path.abspath(ledger) for ledger in ledgers])
        if mode == "backfill":
            return backfill(root, table)
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


# ── 自证 ──

def selftest():
    global current_digest, MEMORY_CAP_RUNNER, PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK
    cases, wrong = 0, []
    import contextlib
    import io
    with tempfile.TemporaryDirectory(prefix="e165-pin-selftest-") as work:
        root = os.path.join(work, "root")
        for directory in (".claude/gate.d", "crates", "scripts/mutations", "research/results"):
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

        def run_product(ledger_path, device_arguments=None):
            """同一趟的产物：每个变体一行（red_examples= 等于账里它的行数），收尾行记账的 sha256；给了 device_arguments 另在头上写一行 config（档从它取）。"""
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
            handle.write("#!/usr/bin/env bash\n# 桩装置：--pinned-cases <表> 逐个数据行打 ⑦；STUB_MODE=wrong 把最后一行打错，=fail 退 1，=killed 打一行之后 kill -9 自己\n"
                         "[[ \"$1\" == --pinned-cases ]] || exit 2\n"
                         "[[ -n \"${STUB_TRACE:-}\" ]] && echo 起了一次 >> \"$STUB_TRACE\"\n"
                         "if [[ -n \"${STUB_KILL_DOMAIN:-}\" ]] && grep -v '^#' \"$2\" | grep -q \"domain=$STUB_KILL_DOMAIN \"; then kill -9 \"$$\"; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == wrong-x ]]; then grep -v '^#' \"$2\" | grep . | cut -f7 | sed '/domain=X / s/returned:/returned:WRONG/'; "
                         "echo \"E7RESULT name=pinned_cases rows=$(grep -v '^#' \"$2\" | grep -c .)\"; exit 0; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == killed ]]; then grep -v '^#' \"$2\" | grep . | head -n 1 | cut -f7; kill -9 \"$$\"; fi\n"
                         "if [[ \"${STUB_MODE:-}\" == slow ]]; then grep -v '^#' \"$2\" | grep . | head -n 1 | cut -f7; sleep 0.6; grep -v '^#' \"$2\" | grep . | tail -n +2 | cut -f7; "
                         "echo \"E7RESULT name=pinned_cases rows=$(grep -v '^#' \"$2\" | grep -c .)\"; exit 0; fi\n"
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
                    got = main(["--root", root, *arguments])
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
        expect("check-default-ledgers", ["--check"], 0, "1 份红例账 3 行都钉在")
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
        # 8 段的旧行：--check 与 --replay 都报没整行抄、出路给 --backfill（开关 old-rows-accepted 下 replay 判绿）
        old_row_table = edited_table(lambda fields, index: fields.__delitem__(slice(TABLE_OWN_COLUMNS, None)) if index == 2 else None)
        write(TABLE, old_row_table)
        expect("check-old-row", ["--check", ledger], 1, "没整行抄红例账（8 段的旧格式，要 22 段）")
        old_row_output = expect("replay-old-row", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "第 4 行没整行抄红例账")
        if "--backfill" not in old_row_output:
            wrong.append("replay-old-row-howto")
            print(f"    replay-old-row-howto 这一格判错：出路要给 --backfill，实测：{old_row_output.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        # 抄来的账行与自己的段对不上（开关 copied-columns-unchecked 下两格都判绿：桩照 ⑦ 打）
        write(TABLE, edited_table(set_field(0, 10, "X")))
        expect("replay-copied-domain-differs", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "第 2 行：⑪（红例账第 3 段）与 ② 不同")
        write(TABLE, edited_table(set_field(0, 19, state_line("L", 3, "shape=1c0", "returned:q"))))
        expect("replay-copied-original-differs", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "第 2 行：⑳（红例账第 12 段）与 ⑦ 不同")
        # 补钉：三行都截成 8 段，--backfill 按 ⑤ 回账里取原行，补完与追加钉出来的逐字节相同
        all_old = edited_table(lambda fields, index: fields.__delitem__(slice(TABLE_OWN_COLUMNS, None)))
        write(TABLE, all_old)
        expect("backfill", ["--backfill"], 0, "补钉：3 行都补成 22 段（3 行 8 段的旧行按 ⑤ 取回账行；档按 ⑤ 那份账同一趟的产物取：--profile r1 3 行）")
        if open(table_path, encoding="utf-8").read() != pinned_text:
            wrong.append("backfill-restores-whole-rows")
            print("    backfill-restores-whole-rows 这一格判错：补完的表应当与追加钉出来的逐字节相同")  # gate-lint:detail
        cases += 1
        expect("backfill-again", ["--backfill"], 0, "没有要补的")
        expect("backfill-arguments", ["--backfill", ledger], 2, "--backfill 不收红例账")
        # ⑤ 的行号错位（Z1 那一行指到 tsv-2 那一行账）：取回的对不上，逐行报、不填，另外两行照补（开关 backfill-unverified 下判错）
        if "e165-selftest-red-examples.tsv:4\t" not in all_old:
            wrong.append("backfill-setup")
            print("    backfill-setup 这一格判错：Z1 那一行的 ⑤ 应当是 e165-selftest-red-examples.tsv:4")  # gate-lint:detail
        cases += 1
        write(TABLE, all_old.replace("e165-selftest-red-examples.tsv:4\t", "e165-selftest-red-examples.tsv:2\t"))
        mismatched = expect("backfill-mismatched-origin", ["--backfill"], 1, "补了 2 行（回读都在）；1 行取不到原行")
        if "取不到原行：第 4 行" not in mismatched or "账第 5 段与 ④" not in mismatched:
            wrong.append("backfill-mismatched-origin-detail")
            print(f"    backfill-mismatched-origin-detail 这一格判错：要逐行点名第 4 行、哪一段对不上，实测：{mismatched.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        write(TABLE, all_old.replace("e165-selftest-red-examples.tsv:4\t", "e165-gone-red-examples.tsv:4\t"))
        expect("backfill-ledger-missing", ["--backfill"], 1, "红例账 research/results/e165-gone-red-examples.tsv 不在")
        write(TABLE, all_old)
        current_digest = lambda path: "0" * 64
        expect("backfill-race", ["--backfill"], 1, "被别人改过")
        current_digest = original_digest
        if open(table_path, encoding="utf-8").read() != all_old:
            wrong.append("backfill-race-wrote")
            print("    backfill-race-wrote 这一格判错：复核判红之后表还是被写了")  # gate-lint:detail
        cases += 1
        write(TABLE, pinned_text)
        # 另一趟出了同样的红例：⑤ 不是它的行，只靠 ⑥ 认，名字取它自己那一趟的产物
        run_ledger = os.path.join(root, "research/results/e165-selftest-r1-red-examples.tsv")
        run_kill_bounds = os.path.join(root, "research/results/e165-selftest-r1-kill-bounds.out")
        write(os.path.relpath(run_ledger, root), "# 第二趟的账\n" + "".join("\t".join(fields) + "\n" for fields in ledger_rows))
        run_text = run_product(run_ledger)
        write(os.path.relpath(run_kill_bounds, root), run_text)
        both = ["--check", ledger, run_ledger]
        expect("check-run-names", both, 0, "2 份红例账 6 行都钉在")
        expect("check-run-names-kill-bounds-option", [*both, "--kill-bounds", run_kill_bounds], 2, "--check 不收 --kill-bounds")
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
        deleted_product = ("E7RESULT name=kill_bound item=tsv-3 table=crates/mutations.tsv table_row=3 outcome=red red_examples=1 title=红的一条 complete=yes\n"
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
        # 出路逐份给补钉命令（追加按文件名配同一趟的产物，不带 --kill-bounds），只点没钉全的那一份账
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
        # 作废红例账登记表：没收尾的那一趟（r1）登记作废，--check 跳过它、成功那句逐个列名（开关 void-ledgers-ignored 下照判、判红）；
        # 登记表本身不对的五种判红（开关 void-rows-unchecked 下坏的那一行不判、r1 照跳过，判绿）；登记了配得上完整产物的账判红、出路删掉那一行（开关 void-complete-accepted 下判 77）
        void_table = os.path.join(root, VOID_LEDGERS_TABLE)
        void_good = "research/results/e165-selftest-r1-red-examples.tsv\t段中断了、没有收尾行，作废\tkb/165.md:346\n"
        void_header = "# 自证用的作废红例账登记表\n"
        write(VOID_LEDGERS_TABLE, void_header + void_good)
        expect("check-void-skipped", ["--check"], 0, "1 份红例账 3 行都钉在")
        expect("check-void-skipped-given", both, 0, "跳过作废账 1 份、被取代的账 0 份：research/results/e165-selftest-r1-red-examples.tsv")
        expect("check-void-all-skipped", ["--check", run_ledger], 77, "一份红例账都没有")
        for label, bad_line, want_text in (
                ("check-void-missing", "research/results/e165-gone-red-examples.tsv\t删掉了的一趟，登记还留着\tkb/165.md:1\n", "登记的账 research/results/e165-gone-red-examples.tsv 不在"),
                ("check-void-short-reason", "research/results/e165-selftest-red-examples.tsv\t作废\tkb/165.md:1\n", "第 3 行的理由「作废」不到 8 个字"),
                ("check-void-columns", "research/results/e165-selftest-red-examples.tsv\t只写了理由没写出处的一行\n", "是 2 段"),
                ("check-void-place", "research/results/e165-selftest-red-examples.tsv\t出处没写行号的一行登记\tkb/165.md\n", "第 3 行的作废出处「kb/165.md」不是「文件:行号」"),
                ("check-void-duplicate", void_good, "前面已登记过")):
            write(VOID_LEDGERS_TABLE, void_header + void_good + bad_line)
            expect(label, both, 1, want_text)
        write(VOID_LEDGERS_TABLE, void_header + void_good + "research/results/e165-selftest-red-examples.tsv\t其实跑完了却登记成作废\tkb/165.md:2\n")
        complete_output = expect("check-void-complete", both, 1, "配得上一份有收尾行、ledger_sha256 对得上的产物 research/results/e165-selftest-kill-bounds.out")
        if "删掉那一行" not in complete_output:
            wrong.append("check-void-complete-howto")
            print(f"    check-void-complete-howto 这一格判错：出路要写删掉那一行，实测：{complete_output.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        os.remove(void_table)
        os.remove(run_kill_bounds)
        os.remove(run_ledger)
        # 被取代红例账登记表：old 那一趟（同档、完整）与 ledger 是同一批反例状态，变体名、表行与第 7 列不同、没钉；登记成被 ledger 取代，--check 跳过它、
        # 成功那句列名带取代它的那份；取代它的账照常判钉全（只给 old 时补进来判，开关 superseded-replacement-pin-unchecked 下不补、退 77）；
        # 覆盖不全（开关 superseded-coverage-unchecked 下判绿）、档不同或取不到档（superseded-profile-unchecked 下判绿）、写法不对（superseded-rows-unchecked 下不判）都判红
        superseded_table = os.path.join(root, SUPERSEDED_LEDGERS_TABLE)
        old_ledger = os.path.join(root, "research/results/e165-selftest-old-red-examples.tsv")
        old_kill_bounds = os.path.join(root, "research/results/e165-selftest-old-kill-bounds.out")
        new_ledger = os.path.join(root, "research/results/e165-selftest-new-red-examples.tsv")
        new_kill_bounds = os.path.join(root, "research/results/e165-selftest-new-kill-bounds.out")
        old_rows = [list(fields) for fields in ledger_rows]
        for fields, (variant, rows_text) in zip(old_rows, (("tsv-4", "4"), ("tsv-5", "5+11"), ("Z1", "2"), ("tsv-5", "5+11"))):
            fields[0], fields[1] = variant, rows_text
            if has_example({"fields": fields}):
                fields[6] += ",+J7a"

        def write_run(ledger_path, product_path, rows, device_arguments=None, finished=True):
            """写一趟的账与同一趟的产物：每个（变体, 第一个表行）一行 kill_bound（red_examples= 等于账里这个变体的行数），finished 时写收尾行。"""
            write(os.path.relpath(ledger_path, root), "# 账头\n" + "".join("\t".join(fields) + "\n" for fields in rows))
            first_rows = {}
            for fields in rows:
                first_rows.setdefault(fields[0], set()).add(fields[1].split("+")[0])
            product_text = f"E7RESULT name=config experiment=E165 segment=2 device_arguments={device_arguments}\n" if device_arguments is not None else ""
            for item, table_rows in first_rows.items():
                for table_row in sorted(table_rows):
                    product_text += (f"E7RESULT name=kill_bound item={item} table={CUSTOM_TABLE if item.startswith('Z') else CRATES_TABLE} table_row={table_row} outcome=red "
                                     f"red_examples={sum(1 for fields in rows if fields[0] == item)} title=变体{item} complete=yes\n")
            if finished:
                product_text += f"E7RESULT name=verdict scope=segment2 red={len(first_rows)} ledger_sha256={file_digest(ledger_path)}\n"
            write(os.path.relpath(product_path, root), product_text)

        def superseded_rows(*lines):
            write(SUPERSEDED_LEDGERS_TABLE, "# 自证用的被取代红例账登记表\n" + "".join("\t".join(line) + "\n" for line in lines))

        old_relative, ledger_relative = "research/results/e165-selftest-old-red-examples.tsv", "research/results/e165-selftest-red-examples.tsv"
        new_relative, gone_relative = "research/results/e165-selftest-new-red-examples.tsv", "research/results/e165-gone-red-examples.tsv"
        superseded_good = (old_relative, ledger_relative, "同档后一趟覆盖了同一批反例状态", "kb/165.md:1011")
        write_run(old_ledger, old_kill_bounds, old_rows)
        expect("check-superseded-unregistered", ["--check"], 1, "2 份红例账 6 行里 3 行没钉进")
        superseded_rows(superseded_good)
        expect("check-superseded-skipped", ["--check"], 0, f"跳过作废账 0 份、被取代的账 1 份：{old_relative}（被 {ledger_relative} 取代）")
        expect("check-superseded-given", ["--check", old_ledger], 0, "1 份红例账 3 行都钉在")
        write(TABLE, "\n".join(line for line in pinned_text.split("\n") if "\tZ1 " not in line and "#Z1" not in line) + "\n")
        expect("check-superseded-replacement-unpinned", ["--check", old_ledger], 1, f"{ledger_relative} 取代了登记的被取代账 {old_relative}")
        write(TABLE, pinned_text)
        for label, rows, want_text in (
                ("check-superseded-coverage", old_rows + [["Z1", "2", "M", "2", "9", "counters", "+J4", "true", "true", "true", "cmd",
                                                            state_line("M", 9, "shape=9c0", "returned:a"), state_line("M", 9, "shape=9c0", "returned:b")]],
                 f"有 1 行在取代它的账 {ledger_relative} 里没有对应行（按域、N、枚举序号与 ① 一行配一行）：第 6 行（域 M、N 2、序号 9）"),
                ("check-superseded-coverage-count", old_rows + [list(old_rows[0])], "有 1 行在取代它的账"),
                ("check-superseded-coverage-state", [[*old_rows[0][:11], state_line("L", 3, "shape=1c9", "returned:a"), old_rows[0][12]], *old_rows[1:]],
                 "第 2 行（域 L、N 2、序号 3）")):
            write_run(old_ledger, old_kill_bounds, rows)
            expect(label, ["--check"], 1, want_text)
        write_run(old_ledger, old_kill_bounds, old_rows, "--profile+r2")
        expect("check-superseded-profile", ["--check"], 1, f"被取代的账 {old_relative} 那一趟的档是「--profile r2」，取代它的账 {ledger_relative} 那一趟是「--profile r1」：档不同")
        os.remove(old_kill_bounds)
        expect("check-superseded-profile-unreadable", ["--check"], 1, "配的 kill-bounds 产物 research/results/e165-selftest-old-kill-bounds.out 不在，取不到档")
        write_run(old_ledger, old_kill_bounds, old_rows)
        reason, place = "同档后一趟覆盖了同一批反例状态", "kb/165.md:1011"
        for label, lines, want_text in (
                ("check-superseded-columns", [superseded_good, (old_relative, ledger_relative, "只写了理由没写出处的一行")], "第 3 行是 3 段"),
                ("check-superseded-short-reason", [(old_relative, ledger_relative, "取代", place)], "第 2 行的理由「取代」不到 8 个字"),
                ("check-superseded-place", [(old_relative, ledger_relative, reason, "kb/165.md")], "第 2 行的取代出处「kb/165.md」不是「文件:行号」"),
                ("check-superseded-missing", [superseded_good, (gone_relative, ledger_relative, reason, place)], f"第 3 行被取代的账 {gone_relative} 不在"),
                ("check-superseded-replacement-missing", [(old_relative, gone_relative, reason, place)], f"第 2 行取代它的账 {gone_relative} 不在"),
                ("check-superseded-self", [(old_relative, old_relative, reason, place)], "是同一份"),
                ("check-superseded-duplicate", [superseded_good, superseded_good], f"第 3 行被取代的账 {old_relative} 前面已登记过")):
            superseded_rows(*lines)
            expect(label, ["--check"], 1, want_text)
        # 取代它的账自己登记成被取代（old 被 ledger 取代、ledger 又被 new 取代）
        write_run(new_ledger, new_kill_bounds, ledger_rows)
        superseded_rows(superseded_good, (ledger_relative, new_relative, reason, place))
        expect("check-superseded-chain", ["--check"], 1, f"第 2 行取代它的账 {ledger_relative} 自己也登记成被取代")
        # 与作废表重登：没收尾的那一趟登记作废，同时登记成被取代或当取代它的账
        void_header = "# 自证用的作废红例账登记表\n"
        write_run(old_ledger, old_kill_bounds, old_rows, finished=False)
        write(VOID_LEDGERS_TABLE, void_header + f"{old_relative}\t段中断了、没有收尾行，作废\tkb/165.md:346\n")
        superseded_rows(superseded_good)
        expect("check-superseded-also-void", ["--check"], 1, f"第 2 行被取代的账 {old_relative} 也登记在作废表")
        write_run(old_ledger, old_kill_bounds, old_rows)
        write_run(new_ledger, new_kill_bounds, ledger_rows, finished=False)
        write(VOID_LEDGERS_TABLE, void_header + f"{new_relative}\t段中断了、没有收尾行，作废\tkb/165.md:346\n")
        superseded_rows((old_relative, new_relative, reason, place))
        expect("check-superseded-replacement-void", ["--check"], 1, f"第 2 行取代它的账 {new_relative} 登记在作废表")
        # 两种都有：成功那句先列作废的、再列被取代的（带取代它的那份）
        superseded_rows(superseded_good)
        expect("check-superseded-and-void", ["--check"], 0,
               f"跳过作废账 1 份、被取代的账 1 份：{new_relative}、{old_relative}（被 {ledger_relative} 取代）")
        for path in (void_table, superseded_table, old_ledger, old_kill_bounds, new_ledger, new_kill_bounds):
            os.remove(path)
        conflicting = [list(fields) for fields in ledger_rows]
        conflicting[0][11] = state_line("L", 3, "shape=1c0", "returned:z")
        other_ledger = os.path.join(root, "research/results/e165-selftest-conflict-red-examples.tsv")
        write(os.path.relpath(other_ledger, root), "# 账头\n" + "\t".join(conflicting[0]) + "\n")
        other_kill_bounds = os.path.join(root, "research/results/e165-selftest-conflict-kill-bounds.out")
        write(os.path.relpath(other_kill_bounds, root),
              "E7RESULT name=kill_bound item=tsv-2 table=crates/mutations.tsv table_row=2 outcome=red red_examples=1 title=红的一条 complete=yes\n"
              f"E7RESULT name=verdict scope=segment2 variants=1 red=1 ledger_sha256={file_digest(other_ledger)}\n")
        expect("conflict", [other_ledger], 1, "原实现逐态行不同")
        os.remove(other_ledger)
        os.remove(other_kill_bounds)
        expect("replay-green", ["--replay", "--binary", stub, "--memory-cap", "none"], 0, "都与 ⑦ 逐字节相同")
        expect("replay-mismatch", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "行与 ⑦ 不同", {"STUB_MODE": "wrong"})
        expect("replay-fail", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "装置退 1", {"STUB_MODE": "fail"})
        expect("replay-device-without-flag", ["--replay"], 77, "还不认 --pinned-cases")
        # 进度行：装置跑得超过进度间隔（假时钟每读一次走 200 秒、间隔 300 秒；桩打一行后停 0.6 秒），跑着时至少打一行 E165_PINNED_PROGRESS，
        # 带挂钟与已打的结果行数；判定照常绿（开关 progress-silent 下一行都不打，这一格判错）
        fake_now, progress_sink = [0.0], io.StringIO()

        def fake_clock():
            fake_now[0] += 200.0
            return fake_now[0]

        saved_progress = (PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK)
        PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK = fake_clock, 0.05, progress_sink
        try:
            expect("replay-progress-green", ["--replay", "--binary", stub, "--memory-cap", "none"], 0, "都与 ⑦ 逐字节相同", {"STUB_MODE": "slow"})
        finally:
            PROGRESS_CLOCK, PROGRESS_POLL_SECONDS, PROGRESS_SINK = saved_progress
        progress_lines = [line for line in progress_sink.getvalue().split("\n") if line.startswith("E165_PINNED_PROGRESS ")]
        if not progress_lines or not all(re.search(r"挂钟 \d+ 秒：.*装置已打 \d+/3 行结果", line) for line in progress_lines):
            wrong.append("replay-progress")
            print(f"    replay-progress 这一格判错：装置跑过进度间隔时要至少一行 E165_PINNED_PROGRESS（挂钟、已打 n/3 行结果），实测：{progress_sink.getvalue()[-300:]!r}")  # gate-lint:detail
        cases += 1
        # 经内存包装起、包装自己退 252（排不上）：判「没跑成」，出路照包装文件头，不报回归（开关 wrapper-exit-as-replay-red 下这一格判错）
        fake_wrapper =os.path.join(work, "fake-memory-cap-wrapper.sh")
        with open(fake_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 252（内存不够排不上）\nexit 252\n")
        saved_runner, MEMORY_CAP_RUNNER = MEMORY_CAP_RUNNER, fake_wrapper
        try:
            expect("replay-wrapper-exit", ["--replay", "--binary", stub, "--memory-cap", "1G"], 1, "没跑成：内存包装 scripts/run-with-memory-cap.sh 退 252")
        finally:
            MEMORY_CAP_RUNNER = saved_runner
        # 包装退 245（显存不够）：同样判「没跑成」、点名显存不够，不报回归（开关 wrapper-exit-as-replay-red 下这一格同样判错）
        gpu_wrapper = os.path.join(work, "gpu-memory-cap-wrapper.sh")
        with open(gpu_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 245（显存不够）\nexit 245\n")
        saved_runner, MEMORY_CAP_RUNNER = MEMORY_CAP_RUNNER, gpu_wrapper
        try:
            expect("replay-wrapper-245", ["--replay", "--binary", stub, "--memory-cap", "1G"], 1,
                   "没跑成：内存包装 scripts/run-with-memory-cap.sh 退 245（显存不够")
        finally:
            MEMORY_CAP_RUNNER = saved_runner
        # 装置被信号停了（直接起时 kill -9 自己；经包装起时包装退 137，外面 kill -9 就是这样）：出路照包装文件头，不写修实现（开关 killed-as-regression 下这两格判错）
        killed_howto = "照 scripts/run-with-memory-cap.sh 文件头「退出码」一段办"
        killed = expect("replay-killed", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "装置被信号停了（退 -9，信号 9）", {"STUB_MODE": "killed"})
        if killed_howto not in killed or "修实现" in killed:
            wrong.append("replay-killed-howto")
            print(f"    replay-killed-howto 这一格判错：出路要照包装文件头、不写修实现，实测：{killed.strip()[-300:]}")  # gate-lint:detail
        cases += 1
        killing_wrapper = os.path.join(work, "killed-memory-cap-wrapper.sh")
        with open(killing_wrapper, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 自证用的假内存包装：什么都不起，退 137（命令被外面 kill -9）\nexit 137\n")
        saved_runner, MEMORY_CAP_RUNNER = MEMORY_CAP_RUNNER, killing_wrapper
        try:
            expect("replay-wrapper-137", ["--replay", "--binary", stub, "--memory-cap", "1G"], 1, "装置被信号停了（退 137，信号 9）")
        finally:
            MEMORY_CAP_RUNNER = saved_runner
        write(CRATES_TABLE, crates_table.replace("红的两条", "改了名的"))
        expect("replay-variant-missing", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "不在它的表里了")
        write(CRATES_TABLE, crates_table)

        # ── 逐项点名与按条复用（文件头「逐项点名与按条复用」）──
        item_line = re.compile(r"^  · (pinned:\d+)：(绿|红)，(现判|复用)", re.M)

        def run_items(arguments, environment=None, at=None):
            """跑一次 main（根默认 root），交 (退出码, 输出, {项: (绿/红, 现判/复用)})。"""
            saved = dict(os.environ)
            os.environ.update(environment or {})
            buffer = io.StringIO()
            try:
                with contextlib.redirect_stdout(buffer):
                    got = main(["--root", at or root, *arguments])
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
        replay_arguments = ["--replay", "--binary", stub, "--memory-cap", "none"]
        _text, pinned_rows = read_table(table_path)
        pinned_items = [pinned_item(row) for row in pinned_rows]
        x_items = [pinned_item(row) for row in pinned_rows if row["fields"][1] == "X"]
        stub_trace = os.path.join(work, "stub-trace.txt")
        open(stub_trace, "w").close()
        got, output, _verdicts = run_items(["--list-items"], {"STUB_TRACE": stub_trace})
        listed = [line.split("\t")[0] for line in output.strip().split("\n")]
        judge_cell("list-items", got == 0 and listed == ["complete", *pinned_items] and len(pinned_items) == 3, f"退 {got}，列出 {listed}")
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
        got, output, _verdicts = run_items(["--check", "--item", "complete"])
        judge_cell("item-not-for-check", got == EXIT_USAGE, f"--check 给 --item 期望退 2，实测退 {got}")
        # 点名单跑一行与它在整格里跑的判定相同（桩把 X 域那一行打错：整格红、X 那一行红、别的两行绿）
        _got, _output, whole = run_items(replay_arguments, {"STUB_MODE": "wrong-x"})
        alone = {item: run_items([*replay_arguments, "--item", item], {"STUB_MODE": "wrong-x"})[2].get(item) for item in whole}
        judge_cell("item-alone-equals-whole", len(whole) == 3 and alone == whole and all(whole[item][0] == "红" for item in x_items)
                   and sorted(verdict for verdict, _how in whole.values()) == ["红", "绿", "绿"], f"整格 {whole}，逐行单跑 {alone}")
        # 按条复用：头一趟全现判、记下；第二趟全复用、不起装置；关了复用全现判；桩（装置）、引的变异行、自己的行各进键
        store_directory = os.path.join(work, "verdict-library")
        with_store = [*replay_arguments, "--verdict-library", store_directory]
        got, output, _verdicts = run_items(with_store)
        judge_cell("reuse-first-run-records", got == 0 and "现判 3、复用 0（现判的原因：无记录 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        open(stub_trace, "w").close()
        got, output, verdicts = run_items(with_store, {"STUB_TRACE": stub_trace})
        judge_cell("reuse-second-run-reuses", got == 0 and "现判 0、复用 3" in output and open(stub_trace).read() == ""
                   and all(how == "复用" for _verdict, how in verdicts.values()), f"退 {got}、装置起了 {open(stub_trace).read().count(chr(10))} 次：{output.strip()[-300:]}")
        got, output, _verdicts = run_items([*with_store, "--no-reuse"])
        judge_cell("reuse-off", got == 0 and "现判 3、复用 0（现判的原因：关了复用 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        # 判法版本是另一列：输入一样都没变、只换判法版本，整格全现判、原因点名判法版本；再照真判法版本跑一趟记回去，后面几格比的是真判法版本
        JUDGE_VERSION_FOR_SELFTEST.append("自证造的另一版判法")
        try:
            got, output, _verdicts = run_items(with_store)
        finally:
            JUDGE_VERSION_FOR_SELFTEST.clear()
        judge_cell("reuse-judge-version-changed", got == 0 and "现判 3、复用 0（现判的原因：判法版本 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        run_items(with_store)
        write(CRATES_TABLE, crates_table.replace("红的一条\tcrates/x.rs\ta\t", "红的一条\tcrates/x.rs\ta2\t"))
        got, output, _verdicts = run_items(with_store)
        write(CRATES_TABLE, crates_table)
        judge_cell("reuse-mutation-row-changed", got == 0 and "现判 1、复用 2（现判的原因：引的变异行 变了 1）" in output, f"退 {got}：{output.strip()[-300:]}")
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
        judge_cell("reuse-device-changed", got == 0 and "现判 3、复用 0（现判的原因：装置 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        replace_stub(stub_text)
        # 每一行的键里有这一行自己：表里另有一行与第一行同一个 ⑥、同一个 ⑦，只 ⑩ 不同；只判过第一行，那一行不能拿它的判定
        twin_fields = list(pinned_rows[0]["fields"])
        twin_fields[9] = twin_fields[9] + "+5"
        write(TABLE, pinned_text.rstrip("\n") + "\n" + "\t".join(twin_fields) + "\n")
        twin_item = f"pinned:{len(pinned_text.rstrip(chr(10)).split(chr(10))) + 1}"
        own_store = os.path.join(work, "verdict-library-own")
        run_items([*replay_arguments, "--verdict-library", own_store, "--item", pinned_items[0]])
        got, output, verdicts = run_items([*replay_arguments, "--verdict-library", own_store])
        judge_cell("reuse-row-own-key", got == 0 and verdicts.get(twin_item) == ("绿", "现判") and "现判 3、复用 1" in output,
                   f"期望 {pinned_items[0]} 复用、{twin_item} 现判，实测退 {got}、{verdicts}")
        write(TABLE, pinned_text)
        # 红的不记：桩把 X 那一行打错，第二趟 X 照判、照红
        red_store = os.path.join(work, "verdict-library-red")
        run_items([*replay_arguments, "--verdict-library", red_store], {"STUB_MODE": "wrong-x"})
        got, output, verdicts = run_items([*replay_arguments, "--verdict-library", red_store], {"STUB_MODE": "wrong-x"})
        judge_cell("reuse-red-not-recorded", got == 1 and all(verdicts.get(item) == ("红", "现判") for item in x_items) and "现判 1、复用 2" in output,
                   f"第二趟期望 X 那一行照判照红、别的复用，实测退 {got}、{verdicts}")
        # 一块判完当场落盘：一块一行，X 那一块装置被 kill -9（不在键里的环境），前两块已记；第二趟同一条命令只判 X
        resume_store = os.path.join(work, "verdict-library-resume")
        first, _output, _verdicts = run_items([*replay_arguments, "--verdict-library", resume_store, "--block-rows", "1"], {"STUB_KILL_DOMAIN": "X"})
        got, output, _verdicts = run_items([*replay_arguments, "--verdict-library", resume_store, "--block-rows", "1"])
        judge_cell("reuse-resume-after-kill", first == 1 and got == 0 and "现判 1、复用 2" in output, f"第一趟退 {first}，第二趟退 {got}：{output.strip()[-300:]}")
        # 剪枝自证：小输入上开着复用与关掉复用，逐项判定相同；输入没变（全复用）与装置改了（全现判、X 转红）各比一次
        prune_store = os.path.join(work, "verdict-library-prune")
        run_items([*replay_arguments, "--verdict-library", prune_store])
        on = run_items([*replay_arguments, "--verdict-library", prune_store])[2]
        off = run_items(replay_arguments)[2]
        judge_cell("prune-unchanged-input", {item: verdict for item, (verdict, _how) in on.items()} == {item: verdict for item, (verdict, _how) in off.items()}
                   and len(on) == 3 and all(how == "复用" for _verdict, how in on.values()), f"开着复用 {on}，关掉 {off}")
        replace_stub(stub_text.replace("[[ \"$1\" == --pinned-cases ]] || exit 2\n", "[[ \"$1\" == --pinned-cases ]] || exit 2\nSTUB_MODE=wrong-x\n", 1))
        on = run_items([*replay_arguments, "--verdict-library", prune_store])[2]
        off = run_items(replay_arguments)[2]
        replace_stub(stub_text)
        judge_cell("prune-after-change", {item: verdict for item, (verdict, _how) in on.items()} == {item: verdict for item, (verdict, _how) in off.items()}
                   and all(off.get(item, ("?",))[0] == "红" for item in x_items), f"开着复用 {on}，关掉 {off}")
        # 装置的 cargo 依赖闭包进键：临时 git 仓里摆一个最小工作区（checker 档包带装置 bin、依赖一个 path crate），只改那个依赖的源码，三行全现判
        closure_root = os.path.join(work, "closure-root")
        for relative in (TABLE, CRATES_TABLE, CUSTOM_TABLE):
            os.makedirs(os.path.dirname(os.path.join(closure_root, relative)), exist_ok=True)
            with open(os.path.join(closure_root, relative), "w", encoding="utf-8") as handle:
                handle.write(open(os.path.join(root, relative), encoding="utf-8").read())
        closure_files = {"Cargo.toml": '[workspace]\nmembers = ["crates/singlefs-checker-tier", "crates/dep"]\n', "Cargo.lock": "# 锁\n",
                         "crates/singlefs-checker-tier/Cargo.toml": '[package]\nname = "singlefs-checker-tier"\nversion = "0.0.0"\n\n[dependencies]\ndep = { path = "../dep" }\n',
                         f"crates/singlefs-checker-tier/src/bin/{DEVICE_BINARY}.rs": f"// 认 {DEVICE_FLAG}\nfn main() {{}}\n",
                         "crates/dep/Cargo.toml": '[package]\nname = "dep"\nversion = "0.0.0"\n', "crates/dep/src/lib.rs": "pub fn one() -> u32 { 1 }\n"}
        for relative, content in closure_files.items():
            os.makedirs(os.path.dirname(os.path.join(closure_root, relative)) or closure_root, exist_ok=True)
            with open(os.path.join(closure_root, relative), "w", encoding="utf-8") as handle:
                handle.write(content)
        subprocess.run(["git", "init", "-q", closure_root], capture_output=True)
        closure_store = os.path.join(work, "verdict-library-closure")
        run_items([*replay_arguments, "--verdict-library", closure_store], at=closure_root)
        with open(os.path.join(closure_root, "crates/dep/src/lib.rs"), "a", encoding="utf-8") as handle:
            handle.write("// 依赖改了一行\n")
        got, output, _verdicts = run_items([*replay_arguments, "--verdict-library", closure_store], at=closure_root)
        judge_cell("reuse-closure-changed", got == 0 and "现判 3、复用 0（现判的原因：装置 变了 3）" in output, f"退 {got}：{output.strip()[-300:]}")
        os.makedirs(os.path.join(closure_root, "crates/singlefs-checker-tier/tests"))
        with open(os.path.join(closure_root, "crates/singlefs-checker-tier/tests/not_read_by_the_bin.rs"), "w", encoding="utf-8") as handle:
            handle.write("// cargo 编装置读不到 tests/\n")
        got, output, _verdicts = run_items([*replay_arguments, "--verdict-library", closure_store], at=closure_root)
        judge_cell("reuse-closure-unchanged", got == 0 and "现判 0、复用 3" in output, f"退 {got}：{output.strip()[-300:]}")
        # crate 根下 cargo 不读的文件（Verus 必红世界表 verus-must-red-worlds.tsv 这一类）不进装置那一样：新放一份、再改一行，三行都复用
        # （admission.py 的 ADMISSION_BREAK=closure-keeps-crate-root-files 下这一格判错：三行全现判，原因「装置 变了」）
        root_file_outputs = []
        for content in ("名\t第一行\n", "名\t第一行 \n"):
            with open(os.path.join(closure_root, "crates/dep/verus-must-red-worlds.tsv"), "w", encoding="utf-8") as handle:
                handle.write(content)
            root_file_outputs.append(run_items([*replay_arguments, "--verdict-library", closure_store], at=closure_root))
        judge_cell("reuse-crate-root-file-unchanged", all(got == 0 and "现判 0、复用 3" in output for got, output, _verdicts in root_file_outputs),
                   "；".join(f"退 {got}：{output.strip()[-200:]}" for got, output, _verdicts in root_file_outputs))

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
        # 没写档：r2 档那三行截成 21 段，--check 与 --replay 都报「没写档」、出路给 --backfill（开关 profile-column-unchecked 下当 r1、不报没写档）
        missing_profile_text = pinned_text + two_profile_text[len(pinned_text):].replace("\t--profile r2\n", "\n")
        write(TABLE, missing_profile_text)
        expect("check-profile-missing", ["--check", ledger, r2_ledger], 1, f"行没写档（21 段，要 {TABLE_COLUMNS} 段")
        missing_output = expect("replay-profile-missing", ["--replay", "--binary", stub, "--memory-cap", "none"], 1, "行没写档")
        judge_cell("replay-profile-missing-howto", "--backfill" in missing_output, f"出路要给 --backfill，实测：{missing_output.strip()[-300:]}")
        # 补档：r1 三行与 r2 三行都截成 21 段（再把一行截成 8 段），--backfill 按 ⑤ 那份账同一趟的产物补回两档，与追加钉出来的逐字节相同（开关 profile-read-as-r1 下 r2 那三行补成 r1）
        stripped_text = "\n".join(line[:-len("\t--profile r1")] if line.endswith("\t--profile r1") else line[:-len("\t--profile r2")] if line.endswith("\t--profile r2") else line
                                  for line in two_profile_text.split("\n"))
        write(TABLE, "\n".join("\t".join(line.split("\t")[:TABLE_OWN_COLUMNS]) if line.startswith(two_profile_text.split("\n")[-2][:64]) else line
                               for line in stripped_text.split("\n")))
        expect("backfill-profile", ["--backfill"], 0, "档按 ⑤ 那份账同一趟的产物取：--profile r1 3 行、--profile r2 3 行")
        judge_cell("backfill-profile-restores", open(table_path, encoding="utf-8").read() == two_profile_text, "补完两档的表应当与追加钉出来的逐字节相同")
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
        write(os.path.relpath(r2_kill_bounds, root), run_product(r2_ledger, "--profile+r2+--bounds+segment2"))
        # 规范写法：默认值不写、只选域与 N 的不进、次序固定（开关 profile-read-as-r1 下全成 r1）
        normalized = []
        for text_given, want in (("--profile+r2+--marker-parity+even+--bounds+segment2+--family+all", "--profile r2"),
                                 ("--table+both+--marker-parity+odd+--profile+r3+--disk-order+normal", "--profile r3 --marker-parity odd --table both"),
                                 ("", DEFAULT_PROFILE)):
            try:
                normalized.append(profile_of_device_arguments(text_given) == want)
            except ValueError:
                normalized.append(False)
        judge_cell("profile-normalized", all(normalized), f"三种 device_arguments= 的规范写法逐个对不对：{normalized}")
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
        expect("replay-two-profiles", ["--replay", "--binary", profile_stub, "--memory-cap", "none"], 0, "按档 2 组：--profile r1 3 行、--profile r2 3 行", {"PROFILE_TRACE": profile_trace})
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
        run_items([*replay_arguments, "--verdict-library", profile_store, "--item", pinned_items[0]])
        got, output, verdicts = run_items([*replay_arguments, "--verdict-library", profile_store])
        judge_cell("reuse-row-own-profile", got == 0 and verdicts.get(pinned_items[0]) == ("绿", "复用") and verdicts.get(twin_profile_item) == ("绿", "现判"),
                   f"期望 {pinned_items[0]} 复用、只差档的 {twin_profile_item} 现判，实测退 {got}、{verdicts}")
        write(TABLE, pinned_text)
        os.remove(r2_ledger)
        os.remove(r2_kill_bounds)

    breaks_missed, breaks_proven = [], 0
    if not wrong and not BREAK:
        for item in ("pin-twice", "titles-unchecked", "no-recheck-before-write", "check-ignores-missing", "fingerprint-unchecked", "variant-unchecked", "replay-ignores-output",
                     "wrapper-exit-as-replay-red", "killed-as-regression", "names-from-today-table", "kill-bounds-optional", "title-missing-ignored",
                     "title-not-compared", "variant-sets-unchecked", "ledger-digest-unchecked",
                     "whole-row-unchecked", "old-rows-accepted", "copied-columns-unchecked", "dedupe-by-input-variant", "backfill-unverified",
                     "variant-by-line-number", "append-names-from-today-table", "append-pairing-optional", "progress-silent",
                     "item-filter-ignored", "reuse-key-without-closure", "reuse-key-without-row", "reuse-key-without-mutation-row", "reuse-ignores-key",
                     "reuse-records-red", "reuse-records-whole-cell-only", "void-ledgers-ignored", "void-rows-unchecked", "void-complete-accepted",
                     "profile-read-as-r1", "same-profile-conflict-ignored", "check-ignores-profile", "replay-without-profile", "profile-column-unchecked",
                     "profile-arguments-unchecked", "reuse-key-without-profile", "reuse-key-without-judge-version", "list-items-pipe-unhandled", "superseded-replacement-pin-unchecked", "superseded-coverage-unchecked",
                     "superseded-profile-unchecked", "superseded-rows-unchecked"):
            completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
                                       env=dict(os.environ, E165_PIN_RED_EXAMPLES_BREAK=item))
            if completed.returncode == 0:
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


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
