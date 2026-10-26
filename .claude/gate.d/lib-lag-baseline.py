#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓里那几张表与 gate 基准提交里的那一版，上一次的结论不替这一次作保
# run-condition: command git
# gate-similar: lib-item-ref-status.py 它管按编号取 kb 文件与条目引用的状态，读的是 kb 正文；这一份管「滞后表与基准那一版比」与复跑表的行，读的是登记表与 git 基准，不并进它
# gate-similar: lib-owed.py 它读欠账表里哪些还开着，不碰 git 基准；这一份取基准那一版的表，不并进它
"""滞后表、豁免表与登记表「只缩不涨」那一段的共用读法，加 scripts/experiment-apparatus/replay.sh 复跑表的一份读法。

「只缩不涨」：此刻的表与 gate 基准提交（scripts/gate/changed-paths.sh 的 gate_diff_base gate）里那一版比，基准里没有的行判红。
取基准那一版要分清三种结局，不许把第三种当成第二种静默放过：
  ① 基准里有这张表：交那一版的全文（或内容行）；
  ② 基准里没有这张表（初次登记）：不比；
  ③ git 取不到（ls-tree 或 show 退非 0：基准名不在、对象库缺了那一份）：交一句问题，调用方判红。
这一份只管怎么取、怎么切行；拿哪几列当键、比出来怎么报，留在各处调用方。

调它的各处（各自只留键与报法；现查 `grep -rln lib-lag-baseline .claude/gate.d .claude/hooks scripts`，样本目录 fixtures/ 除外）：
  取基准那一版（baseline_rows 交内容行，baseline_text 交全文，命令行 show 把全文写到标准输出、退 3 是基准里没有这份）：
    门禁 doc-experiments 的 decision-links 格（待回填清单 .claude/decision-links-pending）、multipath-registry 格（multipath-registry-lag.tsv）
    与 replay-comparison-units 格（登记表里复跑写 none: 的行），都经 baseline_rows；decision-links 格另经 baseline_text 取实验页基准那一版
    （baseline_page_text，基准那一版的页照生效的登记换算之后，再比两版正文是不是只差语言切换行或日期整理）；
    门禁 doc-decisions 的 cross-decision-status、inference-needs-verdict、three-way-leg-record 三格共用的挂账表（decision-evidence-lag.tsv，
    它的 base_ledger_text 调 baseline_text，表的行仍按它自己的读法切）与 history-loss 格（变更史，命令行 show）；
    门禁 doc-process-records 的 crates-adversarial-review、agent-def-adversarial-review 两格开头共用的点名一节挂账表核对
    （review-section-pending.tsv，check_review_pending_table，经 baseline_rows）；
    门禁 code-source-discipline 的 mutation-tables 格（mutation-duplicates-lag.tsv，baseline_text）与 reading-discipline 格（两张豁免表，baseline_rows）；
    门禁 checker-independence-and-sync 的 layout-checker-sync 格（格式定义，baseline_text）；
    门禁 code-tooling 的 heavy-thread-defaults 格（.claude/heavy-thread-exempt，命令行 show）与 long-run-header-exempt 格
    （经 .claude/hooks/lib_long_run.py 的 exempt-check --base：它在 exempt_main 里调 baseline_text，同一处另调 effective_moves、baseline_file_bytes
    按搬迁之前的路径到基准里取那份脚本、rewrite_moved_bytes 换算之后再比）。
    取到的全文都照下面「搬迁登记」换算过。
  切内容行（content_rows）：multipath-registry、replay-comparison-units、mutation-tables、reading-discipline 四格与点名一节挂账表那一处读此刻的表。
  复跑表的行（replay_rows；命令行 replay-rows）：门禁 doc-experiments 的 results-cited 格（命令行）、verdict-false-named、quoted-result-lines、
    replay-comparison-units 三格。
  搬迁那一层（下面「搬迁登记」一节）：命令行 moved-only-filter、moved-lines-filter 由 scripts/gate/changed-paths.sh 的 gate_drop_moved_only、
    gate_added_lines 起（经它取改动范围与新增行的各道门禁都走这一条，门禁 doc-decisions 的 entry-added 格直接调 gate_drop_moved_only）；
    path_moves 与 drop_moved_lines：门禁 doc-experiments 的 quoted-result-lines 格（历史节）、门禁 doc-decisions 的 shape 格；
    path_move_report：门禁 code-tooling 的 change-range-single-source 格；
    path_moves：E171 冻结的两份 O3 判法 scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py 与
    scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py（读被判的仓里那一份，没有再取所在仓的），
    两份都另调 rewrite_moved_bytes 倒推搬迁之前的内容；后一份同一目录的 fake_multi_host_run.py 把这一份连同 path-moves.tsv 链进假的第二台；
    path_moves、rewrite_moved_paths、rewrite_moved_bytes、moved_prefix_pattern：scripts/agents/snapshot-tree.py 按全部登记行（不只生效的）反着换（MovesBack、moves_back_for），
    开工快照倒推不出时把清单与 diff 里的新前缀换回旧前缀再倒推一次。
  自证（命令行 --selftest）：门禁 code-tooling 的 research-script-selftests 格（SELFTEST_RUNNERS），.claude/hooks/selftest-registry.tsv 里登记成 light。
注释行的认法并进来时变了（content_rows）：multipath-registry、replay-comparison-units、mutation-tables、reading-discipline 四处改前只认行首 `#` 是注释，
  缩进之后才是 `#` 的行、首列为空而第二列以 `#` 起头的行（行首是制表符），改前进各格的形状判法、判形状错（列数不对、有空格子），并进来之后当注释静默跳过。
  方向是跳过、不报形状错：跳过一行只让豁免少一行、基准少一行，判得更严，丢的只是那一行的形状报错，不放过违例。doc-process-records 改前就去掉行首空白再认 `#`，没变；
  doc-decisions 的挂账表不经 content_rows（只取全文，行照它自己的读法切），没变；decision-links 的待回填清单经 content_rows 之后再照它的形状正则
  （E<号> 或 D<号> 后接空白）认行，`#` 起头的行不管缩没缩进都配不上那个正则，并进来前后认出的行相同。
路径按当前目录算：ls-tree 的路径与 show 的 <基准>:./<路径> 都相对当前目录（调用方给 cwd 就是 cwd），被判的根是别的仓里的一个子目录时，
  取到的是这个子目录里的那一份，不会读到外面那个仓根上同名的表（门禁 doc-decisions 的挂账表原来就这样取）。
scripts/gate/admission.py 另读复跑表（按 TABLE 那一段取复用键），不归这一份。

搬迁登记（目录或文件搬了家、改了名，表里换成新路径的行不算新增）：
  登记表 .claude/gate.d/path-moves.tsv（PATH_MOVES_TABLE，相对当前目录取，就是被判的根里那一份；不在就是一行都没有），一行一条，制表符分隔四列：
    <旧前缀><TAB><新前缀><TAB><YYYY-MM-DD><TAB>records/<文件>.md；前缀是相对仓根的路径，不带开头的 / 与 ./、不带结尾的 /，不含空白与 . / .. 段。
  门禁这一侧只用此刻生效的登记行（effective_moves）：一行此刻生效，当且仅当这一行（旧前缀、新前缀）不在 gate 基准那一版的登记表里，
    而且 gate 基准那一版的树里旧前缀下有文件、新前缀下没有文件（子模块不算文件）；
    提交进基准之后（基准那一版的登记表里有了这一行）这一行对门禁不再起作用，基准树之后又满足「旧前缀下有文件、新前缀下没有文件」也不醒来。
    下面几样换算、剔除与实据都只认生效的行；
    装置里的钉值核对（E171、E170 的 frozen_o3、reuse_library）与 scripts/agents/snapshot-tree.py 的倒推照旧用全部登记行（path_moves），不归这一节。
  gate 基准不总是 HEAD：有上游取上游的 merge-base，gate.sh --staged 推完之后里层取 HEAD~1，GATE_BASE 给了取它（scripts/gate/changed-paths.sh 的 gate_diff_base gate）。
    搬迁已提交而基准还没前移（没推，或 --staged 推完之后基准取 HEAD~1）时，这一行照样生效：此时删掉一份搬过去的文件、在旧前缀下重用旧名字、搬第二步，
    都照搬迁之前的基准判（落点上有文件不在、旧前缀还在磁盘上判红）。要分几步搬的，搬一步、等 gate 基准前移（推上去，或 gate 基准那一版里已有新前缀）再登记下一步。
  新行必须生效（move_evidence）：不在基准那一版登记表里的行，却不生效（旧前缀在基准里没有文件，或新前缀下有基准里本来就在的文件，含只放着占位文件的目录），
    登记表那一格判红「这一行不是一次搬迁」（弄坏开关 idle-new-row-unchecked 下不核）；搬迁推出去、基准里已经是新前缀之后才补的登记同样判红。
    基准那一版登记表里已有的行不生效是对的，成功句里点名是哪一条条件不满足。
  baseline_text 交出全文之前，按登记的次序逐行把全文里生效的旧前缀换成新前缀；
    只按仓根起的路径前缀换（前缀写法表）：
      旧前缀前面不接字母、数字、_、-、.、/：换（行首、空白、引号、反引号、(、[、=、:、, 等后面的前缀，都在这一档）；
      旧前缀紧挨在 / 后面：/ 前面那一段要是下面的仓根写法才换，前面又不能接字母、数字、_、-、.、/：
        ./ 与 ../（可连写）、$变量/、${变量}/（变量名后可带一个收尾引号）、<仓根>/、本仓的绝对路径（rewrite_moved_paths 的 roots，默认是这一份所在的仓根与被判的根；仓根在家目录下时 ~/…、$HOME/…、${HOME}/… 的写法也认）；
      别的目录名后面的同名段（tools/docs、vendor/tools、$FROZEN/tree/docs、别处的绝对路径、只有一个 / 开头的 /docs）不是仓根起的前缀，不换（弄坏开关 slash-prefix-rewritten 下退回前面接 / 也换）；
    后面接 / 或串尾、或别的字符，不接字母、数字、_、-，也不接「. 再接字母、数字、_、-」（scripts/experiments2、scripts/experiments.md 不换）；
    紧跟在「<7–40 位十六进制>:」（可带 ./，十六进制前面不接字母、数字、_）后面的不换：git show 31e47115:scripts/experiments/… 指的是那个提交里的文件，仓里有意留旧路径。
    paths_before_moves、moved_path 作用在仓根起的整条路径上，同样只认仓根起的前缀。
  改动范围剔除只差搬迁的（moved_only_paths、drop_moved_lines，scripts/gate/changed-paths.sh 的 gate_changed_paths 与 gate_added_lines 经命令行
    moved-only-filter / moved-lines-filter 调，门禁 doc-decisions 的 entry-added 格经 changed-paths.sh 的 gate_drop_moved_only 调；
    自己按 git diff -U0 取新增行号的门禁 doc-decisions 的 shape 格、doc-experiments 的 quoted-result-lines 格（历史节）直接调 drop_moved_lines）：
    文件与基准比，原地改的按原路径取，改了路径的只在新路径恰好等于旧路径照一行生效的登记换前缀时按旧路径取（单个文件登记的，文件名即最后一段不许变；move_source、move_target），
    模式相同、字节原样相同或照生效的登记换算之后相同就不算改动；
    工作区里不在的旧路径，照登记换到落点、落点那一份与基准里这一份比；暂存区与 HEAD 不同的，暂存区那一份与 HEAD 照同一种比法也要只差搬迁。
    整份搬了家而新旧路径落在不同的按路径管的类（.claude/agents/、.claude/agent-common.md、.claude/main-agent.md、.claude/rules/、.claude/hooks/、
    crates/<包>/src/ 各是一类，governed_class）的，照算改动，新增行一条都不剔。
    新增行与基准那一版按 diff 的位置配对：git diff -U0 的同一块里被删的第 k 行只配新增的第 k 行，照登记换算之后与它相同（换算前后又不同）的剔掉，
    删与加条数不等的块一行都不剔；整份搬了家的另剔 diff 里没动的行；别处新加的同文行、原地留着旧行又另加的换算之后的行照算新增（drop_moved_lines、moved_line_numbers）。
    登记表没有内容行、没有生效的行、被判的根不是仓根时一条都不剔，与没有这一层时同判；登记表写坏了照没有登记时交回、打一行提示；git 取不到判红。
  被取的表自己搬了家：基准里没有这条路径时，按生效的登记倒着换回搬迁之前的路径去取，取到的那一份照样换成新前缀再交。
  登记表里一行内容行都没有（不在、只有注释与空行）时，取法与交出的全文与没有这一节时逐字节相同；有内容行而一行都不生效时，交出的全文也与没有这一节时逐字节相同。
  登记表写坏了（列数不对、前缀不是相对路径、日期不是 YYYY-MM-DD、出处不是 records/<文件>.md）：baseline_text 交 PathMovesProblem（问题句点名登记表），调用方照「git 取不到」判红。
  登记表的判法（path_move_problems / path_move_report，门禁 code-tooling 的 change-range-single-source 格调）：每一行写法对、出处文件在；
    此刻生效的行之间，任一行的旧前缀或新前缀不许等于、包含或落在另一行的旧前缀或新前缀下面（不许串，A→B、B→A 来回也算）；
    此刻生效的每一行要有搬迁实据：基准里旧前缀下的每份文件，此刻在新前缀下照这一行换前缀的同一相对路径上都在，而且被 git 跟踪（在索引里，或是没被忽略的工作区文件）；
    新前缀在基准里没有文件；旧前缀此刻不在磁盘上；换算之后相同的份数只报不判。不生效的行：不在基准那一版登记表里的判红（新行必须生效），已在的不核实据。
    基准取 gate 基准；被判的根不在 git 仓里、不是仓根时判不了哪几行生效，门禁一行都不按它换算，实据不核，成功句写明。
  关不住的（门禁不判，改动范围里留给人看的那些照算改动）：
    ① 钉值一侧与 snapshot-tree.py 按全部登记行逐次倒换：登记表里留着一行老行（旧前缀下已没有文件、新前缀是早就在的另一份实现），再加一行真搬迁，
       钉住的文件改指那一份早就在的实现，倒换倒得回钉值，钉值一侧认；门禁一侧这份文件照算改动；
    ② ${变量:-旧前缀/…} 这类默认值写法：旧前缀前面接 -，不换，那一处照算改动；
    ③ 新前缀下基准里已有占位文件（.gitkeep 这类）的目录：这一行不生效，起它的整次真搬迁都照算改动，登记表那一格判红，问题句点名新前缀下基准里已有的文件；
    ④ 换了实现的不登记：落点逐份在、被跟踪就过实据，换了内容的那几份文件与新增行照算改动；
    ⑤ 被判的根在仓里但不是仓根时，判不了哪几行生效，门禁一行都不按它换算，实据不核。

函数（python 里用 importlib.util.spec_from_file_location 载入这份文件）：
  content_rows(text) → [(行号, 各列)]：去掉行首空白后以 # 起头的行与空行不算，行尾的 \\r 去掉，按制表符切；行号是原文里的行号。
  baseline_text(base, path, cwd=None) → (全文或 None, 问题或 None)：上面三种结局依次是 (全文, None)、(None, None)、(None, BaselineProblem)。
  baseline_rows(base, path, cwd=None) → (内容行或 None, 问题或 None)：baseline_text 的全文过一遍 content_rows。
  BaselineProblem.sentence(stderr_limit=80)：「取不到基准 <基准> 里的 <路径>（git ls-tree 退 N：<stderr 前若干字>）」或「读不出基准 …（git show 退 N：…）」；
    stderr_limit 给 None 时不带「：<stderr>」那一段（code-source-discipline 的 reading-discipline 格把 stderr 另起一行列）。
  replay_rows(path) → [ReplayRow(实验号, 第 2 列, 参数, 产物, 方式)] 或 None（文件不在或读不了）：整份文件里每一行整行形如
    「E<数字>|<第 2 列>|<参数>|<产物>|exact 或 timing」的算复跑行（与门禁 checker-tier-research-build-and-replay 的 experiment-replay 格同一个形状），别的行不算。
  path_moves(cwd=None) → ([PathMove(行号, 旧前缀, 新前缀, 日期, 出处)], [问题句])：读搬迁登记表的全部行；弄坏开关 path-moves-ignored 下交 ([], [])。
  effective_moves(base, cwd=None) → ([PathMove], 问题或 None)：此刻生效的登记行（见「搬迁登记」）；登记表写坏了交 PathMovesProblem，git 取不到交 BaselineProblem。
    effective_among(base, moves, cwd=None, tree=None, registered=None) 同一种判法，作用在给定的登记行上（tree 给了就不另起 git，registered 给了就不再取基准那一版的登记表）。
    base_registered_pairs(base, cwd=None) → ({(旧前缀, 新前缀)}, 问题或 None)：gate 基准那一版的登记表里已有的行（基准里没有这张表交空集）。
  rewrite_moved_paths(text, moves, roots=None) → 文本：按登记的次序把旧前缀换成新前缀（边界与仓根写法见「搬迁登记」；roots 是本仓绝对路径的认法要用的仓根，不给取 default_roots()）；
    rewrite_moved_bytes(data, moves, roots=None) 同一种换法作用在字节上（UTF-8，坏字节原样留着）。
    moved_prefix_pattern(prefix, roots=None) 交认前缀的对象（match(text, 位置)、search(text)），正着换、倒着换、snapshot-tree.py 认新前缀都用它。
  paths_before_moves(path, moves) → [路径]：path 自己，再依次是倒着换回每一次搬迁之前的路径。
  move_source(path, moves) / move_target(path, moves) → 路径或 None：改了路径的文件恰好换前缀的那一头（见「搬迁登记」），不是就交 None。
  baseline_file_bytes(base, path, cwd=None) → (字节或 None, 取到的那条路径或 None, 问题或 None)：基准里那份文件的原样字节，
    按生效的登记 paths_before_moves 依次去取（搬迁之前的路径也认）；基准里都没有交 (None, None, None)，git 取不到交 BaselineProblem。不换前缀，要换的调用方调 rewrite_moved_bytes。
  path_move_problems(root, base=None) → (内容行数, [(说什么, 怎么办)])：搬迁登记表写对没有、生效的行串没串、有没有搬迁实据；
    path_move_report(root, base=None) 另交一句说哪几行生效、实据核了什么（门禁 code-tooling 的 change-range-single-source 格调它）。base 不给在根上取 gate 基准
    （CHANGED_PATHS_LIBRARY 的 gate_diff_base gate），给空串判不了哪几行生效、实据不核。
  moved_only_paths(base, paths, cwd=None) → (集合, 问题或 None)：paths 里与基准相比内容只差登记搬迁的那几条（判法见「搬迁登记」）。
  drop_moved_lines(base, entries, cwd=None) → (留下的 entries, 问题或 None)：entries 是 [(路径, 行文, …)]，剔掉只因登记搬迁才变了的新增行；
    第三项是整数的当工作区那一份的行号，按行号剔，别的按行文在可剔的行里逐条抵。

命令行：
  lib-lag-baseline.py show <基准> <路径>              在当前目录的仓里取基准那一版（路径相对当前目录，在仓根上起就是相对仓根）：① 退 0，全文打 stdout；② 退 3，什么都不打；③ 退 1，问题句打 stderr
  lib-lag-baseline.py replay-rows <复跑表路径>         每条复跑行打一行，五列用制表符连；退 0；文件不在或读不了退 1，原因打 stderr
  lib-lag-baseline.py path-moves-check [根 [--base <基准>]]  判根（不给就是当前目录）下的搬迁登记表（实据照 --base，不给在根上取 gate 基准）：
                                                       好退 0、stdout 打一行（带行数与实据那一句）；有问题退 1、逐条 ✗ 加 → 出路
  lib-lag-baseline.py moved-only-filter <基准>         标准输入一行一条路径，标准输出交回不只差登记搬迁的那几条（次序不变）；退 0；git 取不到退 1，问题句打 stderr
  lib-lag-baseline.py moved-lines-filter <基准>        标准输入一行一条「路径<TAB>行文」，标准输出交回剔掉只因登记搬迁才变了的行之后的那些；退 0；git 取不到退 1
  lib-lag-baseline.py --selftest                       临时 git 仓里造三种结局、搬迁登记、只差搬迁的改动与复跑表样本，逐项打 ✓ / ✗；全过退 0，有一项不过退 1
  用法错退 2。

弄坏开关（只给自证与门禁样本判红用，逗号连几项；认不出的值抛 ValueError；门禁阶段里只在样本根上认，平常跑由 lib/stage-cells.sh 拒）：
  LAG_BASELINE_BREAK=unreadable-as-absent  baseline_text() 把 git 取不到基准那一版（ls-tree、show 退非 0）当成「基准里没有这张表」，不交问题句
      基准名取不到（「✗ 自证失败：基准名取不到」）、基准那一版的对象缺了（「✗ 自证失败：基准那一版的对象缺了」）、命令行三种结局（「✗ 自证失败：命令行三种结局」）；
      门禁 code-source-discipline 的样本 reading-discipline-red（输出里找不到「读不出基准 HEAD 里的 .claude/gate.d/experiment-seed-fold-lag.tsv（git show 退 128）」）；
      门禁 doc-experiments 的样本 decision-links-pending-base-unreadable-red（输出里找不到「[待回填] 读不出基准 HEAD 里的 .claude/decision-links-pending（git show 退 128」）；
      门禁 doc-decisions 的样本 evidence-ledger-base-unreadable-red（期望退出 1，实测 0）
  LAG_BASELINE_BREAK=show-root-relative  baseline_text() 的 git show 照仓根算路径（<基准>:<路径>，改前的写法），ls-tree 仍按当前目录：被判的根是子目录时读到仓根上同名的那一份
      子目录里按当前目录取（「✗ 自证失败：子目录里按当前目录取」）
  LAG_BASELINE_BREAK=path-moves-ignored  不读搬迁登记表：baseline_text() 不换前缀、不按搬迁之前的路径取，path_moves() 交空表
      搬迁登记换前缀（「✗ 自证失败：搬迁登记：旧前缀换成新前缀，只按前缀边界换」）、按搬迁之前的路径取表（「✗ 自证失败：搬迁登记：按搬迁之前的路径取表」）、
      取文件（「✗ 自证失败：搬迁登记：文件按搬迁之前的路径取、换算之后与搬完的相同」）、只用生效的行（「✗ 自证失败：搬迁登记：门禁只用此刻生效的行…」）、
      写坏的登记表（「✗ 自证失败：搬迁登记：写坏的登记表交问题句」）、实据与窗口那几项（「搬迁登记：此刻生效的行要有搬迁实据…」「搬迁登记：生效窗口跟着 gate 基准走…」
      「搬迁登记：生效窗口，提交进基准之后…」「搬迁登记：旧名字重新用起来…」「搬迁登记：新行插在老行前面…」）、仓根起的前缀（「搬迁登记：换算只认仓根起的前缀…」）；
      门禁 code-tooling 的样本 heavy-thread-defaults-moved-green（期望退出 0，实测 1：排除表涨了）与 long-run-header-exempt-moved-green（期望退出 0，实测 1：豁免表涨了）；
      只差搬迁那几项也红（「✗ 自证失败：只差搬迁：改名的、原地换了路径的…」「只差搬迁：新增行剔掉…」「只差搬迁：命令行 moved-only-filter 与 moved-lines-filter」
      「只差搬迁：登记表写坏了照没有登记时交回…」「只差搬迁：新增行按 diff 的位置配对…」「只差搬迁：整份搬进别的按路径管的类（含 .claude/gate.d/ 顶层、.claude/singlefs-ai-sop/rules/）照算改动…」「只差搬迁：改了路径的文件只在恰好换前缀时剔…」），
      scripts/gate/changed-paths.sh 的自证「搬迁：只差登记搬迁的文件不算改动」「搬迁：新增行剔掉只差搬迁的行」「搬迁：换算只认仓根起的前缀…」「搬迁：搬迁已提交、有上游没推…」四项红；
      门禁 doc-decisions 的样本 entry-added-moved-only-not-run（期望退出 77，实测 1）、shape-moved-only-green（期望退出 0，实测 1），
      门禁 doc-experiments 的样本 quoted-result-lines-moved-history-green（期望退出 0，实测 1）
  LAG_BASELINE_BREAK=commit-ref-rewritten  rewrite_moved_paths() 连「<提交哈希>:」后面的路径也换（改前的换法）
      提交引用（「✗ 自证失败：搬迁登记：提交引用后面的路径不换」）、只差搬迁（「✗ 自证失败：只差搬迁：改名的、原地换了路径的…」、「只差搬迁：命令行 moved-only-filter 与 moved-lines-filter」）；
      门禁 doc-decisions 的样本 history-loss-moved-commit-ref-green（期望退出 0，实测 1）
  LAG_BASELINE_BREAK=moved-only-kept  moved_only_paths() 一条都不剔、drop_moved_lines() 原样交回：与基准只差登记搬迁的文件与行照旧算改动
      只差搬迁（「✗ 自证失败：只差搬迁：改名的、原地换了路径的…」「只差搬迁：新增行剔掉…」「只差搬迁：命令行 moved-only-filter 与 moved-lines-filter」
      「只差搬迁：登记表写坏了照没有登记时交回、提示一行；基准取不到退 1 不交半截」「只差搬迁：新增行按 diff 的位置配对…」「只差搬迁：整份搬进别的按路径管的类照算改动…」
      「只差搬迁：改了路径的文件只在恰好换前缀时剔…」）、「搬迁登记：此刻生效的行要有搬迁实据…」（真搬迁那一份的 docs/howto.md 没剔）、「搬迁登记：换算只认仓根起的前缀…」、
      「搬迁登记：生效窗口跟着 gate 基准走…」；
      scripts/gate/changed-paths.sh 的自证「搬迁：只差登记搬迁的文件不算改动」「搬迁：新增行剔掉只差搬迁的行」「搬迁：换算只认仓根起的前缀…」「搬迁：搬迁已提交、有上游没推…」四项红；
      门禁 doc-decisions 的样本 entry-added-moved-only-not-run（期望退出 77，实测 1）、shape-moved-only-green（期望退出 0，实测 1），
      门禁 doc-experiments 的样本 quoted-result-lines-moved-history-green（期望退出 0，实测 1）
  LAG_BASELINE_BREAK=move-evidence-unchecked  path_move_report() 不核此刻生效的行串没串、有没有搬迁实据（只核写法与出处）
      「✗ 自证失败：搬迁登记：此刻生效的行要有搬迁实据…」，另有成功句或问题句里读生效行数与实据的几项红（「搬迁登记：门禁只用此刻生效的行…」「搬迁登记：登记表的判法…」
      「搬迁登记：不在基准那一版登记表里的新行必须此刻生效…」「搬迁登记：换算只认仓根起的前缀…」「搬迁登记：基准那一版登记表里已有的行不生效…」
      「搬迁登记：生效窗口跟着 gate 基准走…」「搬迁登记：生效窗口，提交进基准之后…」「搬迁登记：旧名字重新用起来…」「搬迁登记：新行插在老行前面…」「搬迁登记：此刻生效的行不许串…」）；
      门禁 code-tooling 的样本 change-range-single-source-path-moves-evidence-red 判绿（期望退出 1，实测 0）、change-range-single-source-path-moves-evidence-green、
      change-range-single-source-path-moves-committed-green、change-range-single-source-path-moves-base-row-green 里「此刻生效」那条 want 找不到
  LAG_BASELINE_BREAK=window-ignored  effective_among() 把每一行都当此刻生效（去掉生效窗口：登记行提交以后照旧作用在门禁上）
      「✗ 自证失败：搬迁登记：生效窗口，提交进基准之后这一行对门禁不再起作用…」（基准那一版被换、改回新路径的那一行被剔），
      「✗ 自证失败：搬迁登记：生效窗口跟着 gate 基准走…」（推上去之后基准取 HEAD，这一行还当生效），
      另红「搬迁登记：旧名字重新用起来…」「搬迁登记：已提交的两行换了次序…」「搬迁登记：新行插在老行前面…」「搬迁登记：门禁只用此刻生效的行…」「搬迁登记：登记表的判法…」
      「搬迁登记：此刻生效的行要有搬迁实据…」「搬迁登记：不在基准那一版登记表里的新行必须此刻生效…」「搬迁登记：基准那一版登记表里已有的行不生效…」「搬迁登记：此刻生效的行不许串…」；
      scripts/gate/changed-paths.sh 的自证「搬迁：推上去之后基准取 HEAD，登记行不再作用…」红；
      门禁 code-tooling 的样本 change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green、change-range-single-source-path-moves-base-row-green 判红（期望退出 0，实测 1）
  LAG_BASELINE_BREAK=occupied-unchecked  effective_among() 不看新前缀在基准里有没有文件、move_evidence() 不报新前缀下有基准里本来就在的文件（去掉占用核对）
      「✗ 自证失败：搬迁登记：不在基准那一版登记表里的新行必须此刻生效…」（新前缀下早有文件的新行当生效、问题句不出，ops/launch.sh 改起 B 被剔）；
      门禁 code-tooling 的样本 change-range-single-source-path-moves-evidence-red（输出里找不到「第 2 行的新前缀 tools/real-b 下有基准 HEAD 里本来就在的文件」）
  LAG_BASELINE_BREAK=idle-new-row-unchecked  move_evidence() 不核不在基准那一版登记表里的新行生不生效（不生效的新行不判红「这一行不是一次搬迁」）
      「✗ 自证失败：搬迁登记：不在基准那一版登记表里的新行必须此刻生效…」，另红「搬迁登记：门禁只用此刻生效的行…」「搬迁登记：登记表的判法…」「搬迁登记：生效窗口跟着 gate 基准走…」（推上去之后才补登记的新行不红）；
      门禁 code-tooling 的样本 change-range-single-source-path-moves-evidence-red（输出里找不到「这一行不是一次搬迁」与「有 4 处不对」）
  LAG_BASELINE_BREAK=base-row-stays-effective  effective_among() 不看这一行在不在基准那一版的登记表里（基准那一版登记表里的老行，基准树又满足窗口时照样生效）
      「✗ 自证失败：搬迁登记：基准那一版登记表里已有的行不生效…」（旧名字重新有了文件、新前缀又空了，老行醒来：登记表红、没改的表被换算），
      另红「搬迁登记：此刻生效的行要有搬迁实据…」「搬迁登记：生效窗口跟着 gate 基准走…」「搬迁登记：生效窗口，提交进基准之后…」「搬迁登记：旧名字重新用起来…」；
      门禁 code-tooling 的样本 change-range-single-source-path-moves-evidence-green、change-range-single-source-path-moves-committed-green、change-range-single-source-path-moves-base-row-green 判红（期望退出 0，实测 1）
  LAG_BASELINE_BREAK=slash-prefix-rewritten  rewrite_moved_paths() 与 moved_prefix_pattern() 退回旧边界：旧前缀前面紧挨 / 也换（别的目录下同名的一段一起换）
      「✗ 自证失败：搬迁登记：换算只认仓根起的前缀…」（tools/docs 被当成只差搬迁：排除表的基准那一版被换、调用方被剔、新增行被剔）；
      scripts/gate/changed-paths.sh 的自证「搬迁：换算只认仓根起的前缀…」「搬迁：搬迁已提交、有上游没推…」两项红
  LAG_BASELINE_BREAK=chain-allowed  move_evidence() 不核此刻生效的行串没串（放开串）
      「✗ 自证失败：搬迁登记：此刻生效的行不许串…」（同一前缀下两行不再判红）
  LAG_BASELINE_BREAK=untracked-landing-accepted  move_evidence() 落点只要在磁盘上，不核被 git 跟踪
      「✗ 自证失败：搬迁登记：此刻生效的行要有搬迁实据…」（落点被 .gitignore 忽略的那一份不再判红）
  LAG_BASELINE_BREAK=name-change-dropped  exact_rename() 单个文件登记换了文件名也算恰好换前缀（改名的文件照剔）
      「✗ 自证失败：只差搬迁：改了路径的文件只在恰好换前缀时剔…」
  LAG_BASELINE_BREAK=lines-by-count  drop_moved_lines() 不看位置，按行文逐条去抵（整份搬了家的每一行、原地改的换算前后不同的那几行都可抵）
      「✗ 自证失败：只差搬迁：新增行按 diff 的位置配对…」
  LAG_BASELINE_BREAK=hunk-any-position  moved_line_numbers() 同一块里任一被删的行都能配新增的行，删加条数不等的块也配（不按第 k 行配第 k 行）
      「✗ 自证失败：只差搬迁：新增行按 diff 的位置配对…」
  LAG_BASELINE_BREAK=governed-moves-dropped  moved_only_paths() 整份搬进别的按路径管的类也照剔
      「✗ 自证失败：只差搬迁：整份搬进别的按路径管的类照算改动…」
  LAG_BASELINE_BREAK=governed-lines-dropped  drop_moved_lines() 整份搬进别的按路径管的类的文件，新增行照整份搬家剔（diff 里没动的行剔掉）
      「✗ 自证失败：只差搬迁：整份搬进别的按路径管的类照算改动…」
"""
import collections
import datetime
import os
import re
import stat
import subprocess
import sys
import tempfile
from typing import NamedTuple

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__)))), ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "LAG_BASELINE_BREAK"
KNOWN_BREAKS = ("unreadable-as-absent", "show-root-relative", "path-moves-ignored", "commit-ref-rewritten", "moved-only-kept",
                "move-evidence-unchecked", "lines-by-count", "governed-moves-dropped", "window-ignored", "occupied-unchecked",
                "chain-allowed", "untracked-landing-accepted", "name-change-dropped", "hunk-any-position", "governed-lines-dropped",
                "slash-prefix-rewritten", "idle-new-row-unchecked", "base-row-stays-effective")
EXIT_ABSENT = 3
# 这一份所在的仓根（往上三层）：被判的根没另给时，本仓的绝对路径写法按它认
OWN_REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
# 被判的根上取 gate 基准的共用取法（这一份所在仓里那一份；path_move_problems 不给基准时在根上起它的 gate_diff_base gate）
CHANGED_PATHS_LIBRARY = os.path.join(OWN_REPOSITORY_ROOT, "scripts", "gate", "changed-paths.sh")
# 按路径管的几类（各是一类，crates/<包>/src/ 每个包各一类）：整份搬了家、新旧路径不在同一类的，不算只差搬迁
GOVERNED_FILES = (".claude/agent-common.md", ".claude/main-agent.md")
GOVERNED_DIRECTORIES = (".claude/agents/", ".claude/rules/", ".claude/hooks/", ".claude/singlefs-ai-sop/rules/")
GATE_STAGE_DIRECTORY = ".claude/gate.d/"   # 只算顶层（共享门禁把顶层每个 *.sh 当阶段跑），fixtures/ 这类子目录不算
CRATE_SOURCE_DIRECTORY = re.compile(r"^crates/[^/]+/src/")
REPLAY_ROW = re.compile(r"^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$")
PATH_MOVES_TABLE = ".claude/gate.d/path-moves.tsv"   # 搬迁登记表，相对被判的根（当前目录）
PATH_MOVE_ROW_SHAPE = "<旧前缀><TAB><新前缀><TAB><YYYY-MM-DD><TAB>records/<文件>.md"
PATH_MOVE_DATE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$")
PATH_MOVE_SOURCE = re.compile(r"^records/[^/\s][^/]*\.md$")
# 旧前缀前面不接的字符，后面不接的字符；后面接「. 再接这些字符」也不算（scripts/experiments.md）。前面紧挨 / 的另看 slash_prefix_anchored
PATH_BEFORE_BOUNDARY = r"(?<![A-Za-z0-9_.\-])"
PATH_AFTER_BOUNDARY = r"(?![A-Za-z0-9_\-]|\.[A-Za-z0-9_\-])"
PATH_BEFORE_FORBIDDEN = frozenset("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-")
# 旧前缀紧挨在 / 后面时，/ 前面那一段只认这些仓根写法：./ 与 ../（可连写）、$变量/ 与 ${变量}/（变量名后可带一个收尾引号）、<仓根>/；
# 另认本仓的绝对路径（root_spellings）。别的目录名后面的同名段（tools/docs、$FROZEN/tree/docs）不是仓根起的前缀，不换
ROOT_FORM_TAIL = re.compile(r"""(?:(?:\.{1,2}/)+|\$[A-Za-z_][A-Za-z0-9_]*["']?/|\$\{[^{}\s]*\}["']?/|<仓根>/)$""")
ROOT_FORM_REACH = 512   # 往前看多少个字符认仓根写法
# 紧跟在「<7–40 位十六进制>:」（可带 ./）后面的路径指的是那个提交里的文件（git show <提交>:<路径>），换算时不换；往前看 COMMIT_REFERENCE_REACH 个字符认它
COMMIT_REFERENCE_BEFORE = re.compile(r"(?<![0-9A-Za-z_])[0-9a-fA-F]{7,40}:(?:\./)?$")
COMMIT_REFERENCE_REACH = 48
GITLINK_MODE = "160000"


def breaks():
    """此刻设了哪几项弄坏开关（每次现读，同一个进程里切换也认）；认不出的值抛 ValueError。"""
    given = [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item]
    unknown = [item for item in given if item not in KNOWN_BREAKS]
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE} 认不出 {'、'.join(unknown)}（只认 {'、'.join(KNOWN_BREAKS)}）")
    return set(given)


def content_rows(text):
    """表的内容行：[(行号, 各列)]。去掉行首空白后以 # 起头的行与空行不算，行尾 \\r 去掉，按制表符切。"""
    rows = []
    for number, line in enumerate(text.split("\n"), 1):
        line = line.rstrip("\r")
        if line.strip() and not line.lstrip().startswith("#"):
            rows.append((number, line.split("\t")))
    return rows


class BaselineProblem(NamedTuple):
    """git 取不到基准那一版：哪一步（ls-tree / show）、退出码、stderr。"""
    base: str
    path: str
    command: str
    returncode: int
    stderr: str

    def sentence(self, stderr_limit=80):
        verb = "取不到" if self.command == "ls-tree" else "读不出"
        detail = "" if stderr_limit is None else f"：{self.stderr.strip()[:stderr_limit]}"
        return f"{verb}基准 {self.base} 里的 {self.path}（git {self.command} 退 {self.returncode}{detail}）"


class PathMovesProblem(BaselineProblem):
    """搬迁登记表写坏了：基准那一版没法照登记换算（stderr 一栏放逐行的问题）。"""

    def sentence(self, stderr_limit=80):
        detail = "" if stderr_limit is None else f"：{self.stderr.strip()[:stderr_limit]}"
        return f"搬迁登记表 {PATH_MOVES_TABLE} 写得不对，基准 {self.base} 里的 {self.path} 没照登记的搬迁换算{detail}"


class PathMove(NamedTuple):
    """搬迁登记表的一行：行号、旧前缀、新前缀、日期、出处。"""
    line: int
    old: str
    new: str
    date: str
    source: str


def prefix_problem(prefix):
    """前缀是不是相对仓根的路径前缀：是交 None，不是交一句原因。"""
    if not prefix:
        return "是空的"
    if re.search(r"\s", prefix):
        return "含空白"
    if prefix.startswith("/") or prefix.startswith("./"):
        return "要写相对仓根的路径，不带开头的 / 与 ./"
    if prefix.endswith("/"):
        return "不带结尾的 /"
    if any(part in ("", ".", "..") for part in prefix.split("/")):
        return "不许有空段、. 段与 .. 段"
    return None


def read_path_moves(cwd=None):
    """搬迁登记表的内容行（不看弄坏开关）：([PathMove], [问题句])。表不在交 ([], [])。"""
    table = os.path.join(cwd or ".", PATH_MOVES_TABLE)
    if not os.path.isfile(table):
        return [], []
    try:
        with open(table, encoding="utf-8") as handle:
            text = handle.read()
    except (OSError, UnicodeDecodeError) as error:
        return [], [f"读不了 {PATH_MOVES_TABLE}：{error}"]
    moves, problems = [], []
    for number, fields in content_rows(text):
        if len(fields) != 4:
            problems.append(f"第 {number} 行有 {len(fields)} 列，要 4 列（{PATH_MOVE_ROW_SHAPE}）")
            continue
        old, new, date, source = fields
        row_problems = [f"旧前缀「{old}」{why}" for why in [prefix_problem(old)] if why]
        row_problems += [f"新前缀「{new}」{why}" for why in [prefix_problem(new)] if why]
        if not row_problems and old == new:
            row_problems.append(f"旧前缀与新前缀都是「{old}」")
        try:
            dated = bool(PATH_MOVE_DATE.match(date)) and datetime.date.fromisoformat(date) is not None
        except ValueError:
            dated = False
        if not dated:
            row_problems.append(f"日期「{date}」不是 YYYY-MM-DD")
        if not PATH_MOVE_SOURCE.match(source):
            row_problems.append(f"出处「{source}」不是 records/<文件>.md")
        if row_problems:
            problems.append(f"第 {number} 行" + "；".join(row_problems))
        else:
            moves.append(PathMove(number, old, new, date, source))
    return moves, problems


def path_moves(cwd=None):
    """搬迁登记表的全部行：([PathMove], [问题句])；弄坏开关 path-moves-ignored 下交 ([], [])。门禁那一侧要的是此刻生效的那几行（effective_moves）。"""
    if "path-moves-ignored" in breaks():
        return [], []
    return read_path_moves(cwd)


def files_under(base, prefixes, cwd=None):
    """基准 base 那棵树里落在这几个前缀下的文件（路径相对 cwd，子模块不算）：([路径], None)；git 取不到交 ([], BaselineProblem)。"""
    listed = subprocess.run(["git", "--literal-pathspecs", "-c", "core.quotepath=false", "ls-tree", "-r", "-z", base, "--", *sorted(set(prefixes))],
                            capture_output=True, cwd=cwd)
    if listed.returncode != 0:
        return [], BaselineProblem(base, "（搬迁登记的前缀）", "ls-tree", listed.returncode, listed.stderr.decode("utf-8", "replace"))
    files = []
    for record in listed.stdout.split(b"\0"):
        if record:
            meta, _tab, name = record.partition(b"\t")
            if meta.split(b" ")[0].decode("ascii") != GITLINK_MODE:
                files.append(name.decode("utf-8", "surrogateescape"))
    return files, None


def base_registered_pairs(base, cwd=None):
    """gate 基准 base 那一版的搬迁登记表里已有的（旧前缀, 新前缀）：({对}, None)；基准里没有这张表交 (空集, None)；git 取不到交 (空集, BaselineProblem)。
    列数不够的行不算。路径相对 cwd。"""
    text, problem = baseline_text_at(base, PATH_MOVES_TABLE, cwd)
    if problem is not None:
        return set(), problem
    if text is None:
        return set(), None
    return {(fields[0], fields[1]) for _number, fields in content_rows(text) if len(fields) >= 2}, None


def effective_among(base, moves, cwd=None, tree=None, registered=None):
    """moves 里此刻生效的那几行（次序不变）：([PathMove], 问题或 None)。一行生效，当且仅当这一行（旧前缀、新前缀）不在 gate 基准 base 那一版的登记表里，
    而且基准 base 那一版的树里旧前缀下有文件、新前缀下没有文件；
    tree 给了（{路径: TreeEntry}，路径相对 cwd）就用它，不另起 git；registered 给了（base_registered_pairs 的第一项）就不再去取基准那一版的登记表。
    git 取不到交 ([], BaselineProblem)。
    弄坏开关 window-ignored 下全部行都算生效；occupied-unchecked 下新前缀下有没有文件不看，只看旧前缀下有文件；
    base-row-stays-effective 下基准那一版的登记表里有没有这一行不看。"""
    if not moves:
        return [], None
    modes = breaks()
    if "window-ignored" in modes:
        return list(moves), None
    if registered is None and "base-row-stays-effective" not in modes:
        registered, problem = base_registered_pairs(base, cwd)
        if problem is not None:
            return [], problem
    registered = registered or set()
    if "base-row-stays-effective" in modes:
        registered = set()
    if tree is not None:
        files = [path for path, entry in tree.items() if entry.mode != GITLINK_MODE]
    else:
        files, problem = files_under(base, [prefix for move in moves for prefix in (move.old, move.new)], cwd)
        if problem is not None:
            return [], problem
    return [move for move in moves if (move.old, move.new) not in registered and any(under_prefix(path, move.old) for path in files)
            and ("occupied-unchecked" in modes or not any(under_prefix(path, move.new) for path in files))], None


def effective_moves(base, cwd=None):
    """此刻生效的登记行：([PathMove], 问题或 None)；判法见 effective_among。登记表写坏了交 ([], PathMovesProblem)，git 取不到交 ([], BaselineProblem)。"""
    moves, problems = path_moves(cwd)
    if problems:
        return [], PathMovesProblem(base, "（生效的登记行）", "path-moves", 1, "；".join(problems))
    return effective_among(base, moves, cwd)


def default_roots(cwd=None):
    """本仓绝对路径的认法要用的仓根：这一份所在的仓，加被判的根（cwd，不给就是当前目录）。"""
    return tuple(dict.fromkeys((OWN_REPOSITORY_ROOT, os.path.realpath(cwd or "."))))


def root_spellings(roots):
    """仓根 roots 在文本里的写法：绝对路径，仓根在家目录下时再加 ~/…、$HOME/…、${HOME}/…（长的在前）。"""
    home = os.path.expanduser("~").rstrip("/")
    spellings = set()
    for root in roots:
        root = root.rstrip("/")
        if not root:
            continue
        spellings.add(root)
        if home and root.startswith(home + "/"):
            relative = root[len(home):]
            spellings.update(("~" + relative, "$HOME" + relative, "${HOME}" + relative))
    return tuple(sorted(spellings, key=len, reverse=True))


def slash_prefix_anchored(text, position, spellings):
    """text[position:] 开头的前缀紧挨在 / 后面时，/ 前面那一段是不是仓根写法（ROOT_FORM_TAIL 的形态，或 spellings 里本仓绝对路径的某种写法），
    而且那一段的前面又是起点或不接路径字符的地方（字母、数字、_、.、-、/ 之外）。"""
    before = text[max(0, position - ROOT_FORM_REACH):position]
    offset = position - len(before)
    starts = []
    found = ROOT_FORM_TAIL.search(before)
    if found:
        starts.append(found.start())
    starts += [len(before) - len(spelling) - 1 for spelling in spellings if before.endswith(spelling + "/")]
    return any(offset + start == 0 or (text[offset + start - 1] not in PATH_BEFORE_FORBIDDEN and text[offset + start - 1] != "/") for start in starts)


class MovedPrefixPattern:
    """前缀在文本里的认法：前面不接字母、数字、_、.、-，后面不接字母、数字、_、-（也不接「. 再接这些」）；前面紧挨 / 的，/ 前面那一段要是仓根写法
    （slash_prefix_anchored）。弄坏开关 slash-prefix-rewritten 下退回只看前后字符（前面接 / 也换）。match(text, position)、search(text, position=0) 交 re.Match 或 None。"""

    def __init__(self, prefix, spellings):
        self.prefix, self.spellings = prefix, spellings
        self.core = re.compile(PATH_BEFORE_BOUNDARY + re.escape(prefix) + PATH_AFTER_BOUNDARY)
        self.anchored_only = "slash-prefix-rewritten" not in breaks()

    def match(self, text, position=0):
        found = self.core.match(text, position)
        if found and self.anchored_only and position > 0 and text[position - 1] == "/" and not slash_prefix_anchored(text, position, self.spellings):
            return None
        return found

    def search(self, text, position=0):
        position = text.find(self.prefix, position)
        while position >= 0:
            found = self.match(text, position)
            if found:
                return found
            position = text.find(self.prefix, position + 1)
        return None


def moved_prefix_pattern(prefix, roots=None):
    return MovedPrefixPattern(prefix, root_spellings(default_roots() if roots is None else roots))


def rewrite_moved_paths(text, moves, roots=None):
    """按登记的次序把 text 里的旧前缀换成新前缀（只按路径前缀的边界换：前面紧挨 / 的只认仓根写法，见 MovedPrefixPattern）；moves 空的原样交回。
    roots 是本仓绝对路径的认法要用的仓根（不给取 default_roots()）。
    紧跟在「<7–40 位十六进制>:」（可带 ./）后面的不换：那是某个提交里的路径（git show <提交>:<路径>），换了就指不到那个提交里的文件。"""
    keep_commit_references = "commit-ref-rewritten" not in breaks()
    if not any(move.old in text for move in moves):
        return text
    spellings = root_spellings(default_roots() if roots is None else roots)
    for move in moves:
        # 先用 str.find 跳到旧前缀出现的地方再判边界（从左往右、不重叠）：大文件上比整份正则快得多
        pattern, pieces, last = MovedPrefixPattern(move.old, spellings), [], 0
        position = text.find(move.old)
        while position >= 0:
            end = position + len(move.old)
            if pattern.match(text, position) and not (
                    keep_commit_references and COMMIT_REFERENCE_BEFORE.search(text, max(0, position - COMMIT_REFERENCE_REACH), position)):
                pieces += [text[last:position], move.new]
                last = end
                position = text.find(move.old, end)
            else:
                position = text.find(move.old, position + 1)
        text = "".join(pieces) + text[last:]
    return text


def rewrite_moved_bytes(data, moves, roots=None):
    """rewrite_moved_paths 作用在字节上：按 UTF-8 解，坏字节原样留着（surrogateescape）。"""
    if not any(move.old.encode("utf-8") in data for move in moves):
        return data
    return rewrite_moved_paths(data.decode("utf-8", "surrogateescape"), moves, roots).encode("utf-8", "surrogateescape")


def moved_path(path, move, backwards=False):
    """一条路径照一行登记换算（backwards 时从新前缀换回旧前缀）；不在那个前缀下的原样交回。"""
    source, target = (move.new, move.old) if backwards else (move.old, move.new)
    if path == source or path.startswith(source + "/"):
        return target + path[len(source):]
    return path


def paths_before_moves(path, moves):
    """[path, 倒着换回最近一次搬迁之前的路径, 再之前的……]（去重、保持次序）；moves 空的只有 [path]。"""
    candidates = [path]
    current = path[2:] if path.startswith("./") else path
    seen = {path, current}
    for move in reversed(moves):
        current = moved_path(current, move, backwards=True)
        if current not in seen:
            seen.add(current)
            candidates.append(current)
    return candidates


def exact_rename(old_path, new_path, move):
    """old_path 照 move 恰好换前缀就是 new_path：old_path 在旧前缀下、new_path 是把旧前缀换成新前缀的那一条；
    单个文件登记（old_path 正是旧前缀）的，文件名（最后一段）还要不变（弄坏开关 name-change-dropped 下不看文件名）。"""
    if not under_prefix(old_path, move.old) or new_path != move.new + old_path[len(move.old):]:
        return False
    return old_path != move.old or "name-change-dropped" in breaks() or old_path.rsplit("/", 1)[-1] == new_path.rsplit("/", 1)[-1]


def move_source(path, moves):
    """此刻的 path 是从哪一条路径恰好换前缀搬来的（exact_rename，按登记的次序取第一行对得上的）：那条旧路径；不是搬来的交 None。"""
    for move in moves:
        if under_prefix(path, move.new):
            source = move.old + path[len(move.new):]
            if exact_rename(source, path, move):
                return source
    return None


def move_target(path, moves):
    """基准里的 path 恰好换前缀搬到了哪里（exact_rename，按登记的次序取第一行对得上的）：那条新路径；不在任何旧前缀下、或换了文件名的交 None。"""
    for move in moves:
        if under_prefix(path, move.old):
            target = move.new + path[len(move.old):]
            if exact_rename(path, target, move):
                return target
    return None


def baseline_text(base, path, cwd=None):
    """基准提交里 path 那一版：(全文, None)；基准里没有这张表 (None, None)；git 取不到 (None, BaselineProblem)。
    path 相对当前目录（给了 cwd 就是 cwd）：ls-tree 本来就按当前目录认路径，show 写成 <基准>:./<路径> 跟它一致。
    搬迁登记表有此刻生效的行时：基准里没有 path 就依次取搬迁之前的路径，取到的全文照生效的行换成新前缀再交；登记表写坏了交 (None, PathMovesProblem)。"""
    moves, problems = path_moves(cwd)
    if problems:
        return None, PathMovesProblem(base, path, "path-moves", 1, "；".join(problems))
    if not moves and "path-moves-ignored" in breaks() and read_path_moves(cwd)[0]:
        print(f"  ! 弄坏开关 {BREAK_VARIABLE}=path-moves-ignored 生效：不读 {PATH_MOVES_TABLE}，{path} 不按登记的搬迁换算", file=sys.stderr)
    moves, problem = effective_among(base, moves, cwd)
    if problem is not None:
        return None, problem._replace(path=path)
    for candidate in paths_before_moves(path, moves):
        text, problem = baseline_text_at(base, candidate, cwd)
        if text is not None or problem is not None:
            break
    if text is not None and moves:
        text = rewrite_moved_paths(text, moves, default_roots(cwd))
    return text, problem


def baseline_file_bytes(base, path, cwd=None):
    """基准提交里 path 那份文件的原样字节，按此刻生效的登记 paths_before_moves 依次取：(字节, 取到的路径, None)；基准里都没有 (None, None, None)；
    git 取不到 (None, None, BaselineProblem)；搬迁登记表写坏了 (None, None, PathMovesProblem)。不换前缀。"""
    moves, problems = path_moves(cwd)
    if problems:
        return None, None, PathMovesProblem(base, path, "path-moves", 1, "；".join(problems))
    moves, problem = effective_among(base, moves, cwd)
    if problem is not None:
        return None, None, problem._replace(path=path)
    for candidate in paths_before_moves(path, moves):
        relative = candidate[2:] if candidate.startswith("./") else candidate
        listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", relative], capture_output=True, text=True, cwd=cwd)
        if listed.returncode != 0:
            return None, None, BaselineProblem(base, relative, "ls-tree", listed.returncode, listed.stderr)
        if not listed.stdout.strip():
            continue
        shown = subprocess.run(["git", "-c", "core.quotepath=false", "show", f"{base}:./{relative}"], capture_output=True, cwd=cwd)
        if shown.returncode != 0:
            return None, None, BaselineProblem(base, relative, "show", shown.returncode, shown.stderr.decode("utf-8", "replace"))
        return shown.stdout, relative, None
    return None, None, None


class TreeEntry(NamedTuple):
    """树或索引里的一条：模式（100644、100755、120000、160000）与对象名。"""
    mode: str
    oid: str


class OtherKind(NamedTuple):
    """这条路径上不是普通文件也不是符号链接（目录、子模块、没合并完的索引条目）：一律不算只差搬迁。"""
    kind: str


def git_run(arguments, cwd=None, data=None):
    return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True, cwd=cwd, input=data)


def at_top_level(cwd=None):
    """被判的根是不是仓根：(是否, 问题或 None)。不是仓根时改动范围里的路径（相对仓根）与当前目录对不上，只差搬迁那一层不判。"""
    shown = git_run(["rev-parse", "--show-prefix"], cwd)
    if shown.returncode != 0:
        return False, BaselineProblem("仓", "（rev-parse --show-prefix）", "rev-parse", shown.returncode, shown.stderr.decode("utf-8", "replace"))
    return not shown.stdout.strip(), None


def tree_entries(revision, cwd=None):
    """revision 那棵树里的全部文件（相对仓根）：({路径: TreeEntry}, None)；git 取不到 (None, BaselineProblem)。"""
    listed = git_run(["ls-tree", "-r", "-z", "--full-tree", revision], cwd)
    if listed.returncode != 0:
        return None, BaselineProblem(revision, "（整棵树）", "ls-tree", listed.returncode, listed.stderr.decode("utf-8", "replace"))
    entries = {}
    for record in listed.stdout.split(b"\0"):
        if record:
            meta, _tab, name = record.partition(b"\t")
            mode, _kind, oid = meta.decode("ascii").split(" ")
            entries[name.decode("utf-8", "surrogateescape")] = TreeEntry(mode, oid)
    return entries, None


def index_entries(cwd=None):
    """索引里的全部条目：({路径: [(TreeEntry, 合并阶段)]}, None)；git 取不到 (None, BaselineProblem)。"""
    listed = git_run(["ls-files", "--stage", "-z"], cwd)
    if listed.returncode != 0:
        return None, BaselineProblem("索引", "（全部条目）", "ls-files", listed.returncode, listed.stderr.decode("utf-8", "replace"))
    entries = {}
    for record in listed.stdout.split(b"\0"):
        if record:
            meta, _tab, name = record.partition(b"\t")
            mode, oid, merge_stage = meta.decode("ascii").split(" ")
            entries.setdefault(name.decode("utf-8", "surrogateescape"), []).append((TreeEntry(mode, oid), merge_stage))
    return entries, None


def blob_bytes(oids, cwd=None):
    """一次 git cat-file --batch 读回这几份对象的原样字节：({对象名: 字节}, None)；读不到 (None, BaselineProblem)。"""
    wanted = sorted(set(oids))
    if not wanted:
        return {}, None
    shown = git_run(["cat-file", "--batch"], cwd, "".join(oid + "\n" for oid in wanted).encode("ascii"))
    if shown.returncode != 0:
        return None, BaselineProblem("对象库", "（cat-file --batch）", "cat-file", shown.returncode, shown.stderr.decode("utf-8", "replace"))
    blobs, offset, output = {}, 0, shown.stdout
    for oid in wanted:
        end = output.find(b"\n", offset)
        header = output[offset:end].decode("ascii", "replace").split(" ") if end >= 0 else []
        if len(header) != 3 or not header[2].isdigit():
            return None, BaselineProblem("对象库", oid, "cat-file", 1, " ".join(header) or "输出截断了")
        size = int(header[2])
        blobs[oid] = output[end + 1:end + 1 + size]
        offset = end + 1 + size + 1
    return blobs, None


def worktree_version(root, path):
    """工作区里 path 那一份：(模式, 字节)；不在交 None；目录这类交 OtherKind。"""
    full = os.path.join(root, path)
    if os.path.islink(full):
        return "120000", os.fsencode(os.readlink(full))
    if os.path.isfile(full):
        with open(full, "rb") as handle:
            data = handle.read()
        return ("100755" if os.stat(full).st_mode & stat.S_IXUSR else "100644"), data
    if os.path.lexists(full):
        return OtherKind("不是普通文件")
    return None


def governed_class(path):
    """path 落在哪一个按路径管的类（GOVERNED_FILES 各是一类，GOVERNED_DIRECTORIES 各是一类，.claude/gate.d/ 只算顶层的文件，crates/<包>/src/ 每个包各一类）；都不是交 None。"""
    path = path[2:] if path.startswith("./") else path
    if path in GOVERNED_FILES:
        return path
    if path.startswith(GATE_STAGE_DIRECTORY) and "/" not in path[len(GATE_STAGE_DIRECTORY):]:
        return GATE_STAGE_DIRECTORY
    for directory in GOVERNED_DIRECTORIES:
        if path.startswith(directory):
            return directory
    found = CRATE_SOURCE_DIRECTORY.match(path)
    return found.group(0) if found else None


def moved_only_inputs(base, cwd=None):
    """只差搬迁那一层要的东西：(此刻生效的登记行, 基准那棵树, 问题或 None)。
    登记表没有内容行、弄坏开关 moved-only-kept、被判的根不是仓根、没有生效的行时搬迁登记交空表（调用方照没有这一层判）；
    登记表写坏了交 PathMovesProblem；git 取不到交 BaselineProblem。"""
    moves, problems = path_moves(cwd)
    if problems:
        return [], None, PathMovesProblem(base, "（改动范围）", "path-moves", 1, "；".join(problems))
    if not moves:
        return [], None, None
    if "moved-only-kept" in breaks():
        print(f"  ! 弄坏开关 {BREAK_VARIABLE}=moved-only-kept 生效：与基准只差登记搬迁的文件与行照旧算改动", file=sys.stderr)
        return [], None, None
    top_level, problem = at_top_level(cwd)
    if problem is not None or not top_level:
        return [], None, problem
    base_tree, problem = tree_entries(base, cwd)
    if problem is not None:
        return [], None, problem
    moves, problem = effective_among(base, moves, cwd, tree=base_tree)
    return moves, base_tree, problem


def blob_sizes(oids, cwd=None):
    """一次 git cat-file --batch-check 取这几份对象的字节数（不读内容）：({对象名: 字节数}, None)；读不到 (None, BaselineProblem)。"""
    wanted = sorted(set(oids))
    if not wanted:
        return {}, None
    shown = git_run(["cat-file", "--batch-check"], cwd, "".join(oid + "\n" for oid in wanted).encode("ascii"))
    if shown.returncode != 0:
        return None, BaselineProblem("对象库", "（cat-file --batch-check）", "cat-file", shown.returncode, shown.stderr.decode("utf-8", "replace"))
    sizes = {}
    for oid, line in zip(wanted, shown.stdout.decode("ascii", "replace").split("\n")):
        fields = line.split(" ")
        if len(fields) != 3 or not fields[2].isdigit():
            return None, BaselineProblem("对象库", oid, "cat-file", 1, line or "输出截断了")
        sizes[oid] = int(fields[2])
    return sizes, None


def size_may_match(current_size, source_size, moves):
    """只看字节数，换算之后还可能与这一份相同吗：只有一行登记时，差值要是「新前缀比旧前缀长出的字节数」的非负整数倍；几行登记时不筛。"""
    if len(moves) != 1:
        return True
    difference = current_size - source_size
    step = len(moves[0].new.encode("utf-8")) - len(moves[0].old.encode("utf-8"))
    return difference == 0 or (step != 0 and difference % step == 0 and difference // step > 0)


def moved_only_paths(base, paths, cwd=None):
    """paths（改动范围里的路径，相对仓根）里与基准相比内容只差此刻生效的登记搬迁的那几条：(集合, 问题或 None)。
    一条路径要两边都只差搬迁才算：基准到工作区（基准里有这条路径的与它自己比；没有的，按 move_source 取恰好换前缀之前的那一条比；
    工作区里不在的，按 move_target 换到落点，落点那一份与基准里这一份比），HEAD 到暂存区（索引里这一条与 HEAD 不同时，索引那一份与 HEAD 里取到的比）。
    比法：模式相同，字节原样相同（只改了名）或照登记换算（rewrite_moved_bytes）之后相同；先按字节数筛（size_may_match），筛得过的才读内容。
    改了路径而不是恰好换前缀的（单个文件登记换了文件名）照算改动（弄坏开关 name-change-dropped 下照剔）；
    整份搬了家而新旧路径的按路径管的类（governed_class）不同的，不算只差搬迁（弄坏开关 governed-moves-dropped 下照剔）。
    登记表没有内容行、没有生效的行、弄坏开关 moved-only-kept、被判的根不是仓根时交 (空集, None)，与没有这一层时同判；
    登记表写坏了交 (空集, PathMovesProblem)；git 取不到交 (空集, BaselineProblem)。"""
    paths = [path[2:] if path.startswith("./") else path for path in paths if path]
    if not paths:
        return set(), None
    moves, base_tree, problem = moved_only_inputs(base, cwd)
    if problem is not None or not moves:
        return set(), problem
    if git_run(["rev-parse", "--verify", "-q", "HEAD"], cwd).returncode == 0:
        head_tree, problem = tree_entries("HEAD", cwd)
    else:
        head_tree, problem = {}, None
    index, index_problem = index_entries(cwd)
    if problem is not None or index_problem is not None:
        return set(), problem or index_problem
    root = cwd or "."
    roots = default_roots(cwd)
    governed_kept = "governed-moves-dropped" not in breaks()

    def crosses_governed(old_path, new_path):
        return governed_kept and old_path != new_path and governed_class(old_path) != governed_class(new_path)

    def staged_entry(path):
        rows = index.get(path)
        if not rows:
            return None
        if len(rows) != 1 or rows[0][1] != "0":
            return OtherKind("没合并完的索引条目")
        return rows[0][0]

    listed = set()
    for path in paths:
        for tree in (base_tree, head_tree):
            listed.update(tree[candidate].oid for candidate in (path, move_source(path, moves))
                          if candidate in tree and tree[candidate].mode != GITLINK_MODE)
        for candidate in (path, move_target(path, moves)):
            entry = staged_entry(candidate)
            if isinstance(entry, TreeEntry) and entry.mode != GITLINK_MODE:
                listed.add(entry.oid)
    sizes, problem = blob_sizes(listed, cwd)
    if problem is not None:
        return set(), problem
    blobs, wanted = {}, set()

    def version(side, path):
        """(模式, 字节数, 取字节的办法) / None（不在）/ OtherKind。"""
        if side == "worktree":
            full = os.path.join(root, path)
            if os.path.islink(full):
                target = os.fsencode(os.readlink(full))
                return "120000", len(target), lambda: target
            if os.path.isfile(full):
                status = os.stat(full)
                return ("100755" if status.st_mode & stat.S_IXUSR else "100644"), status.st_size, lambda: worktree_version(root, path)[1]
            return OtherKind("不是普通文件") if os.path.lexists(full) else None
        entry = staged_entry(path)
        if entry is None or isinstance(entry, OtherKind):
            return entry
        if entry.mode == GITLINK_MODE:
            return OtherKind("子模块")
        wanted.add(entry.oid)
        return entry.mode, sizes[entry.oid], lambda: blobs[entry.oid]

    def same_after_moves(current, source, collecting):
        if current is None or isinstance(current, OtherKind) or source.mode == GITLINK_MODE:
            return False
        mode, size, read = current
        if mode != source.mode or not size_may_match(size, sizes[source.oid], moves):
            return False
        if collecting:
            wanted.add(source.oid)
            return True
        data, before = read(), blobs[source.oid]
        return data == before or data == rewrite_moved_bytes(before, moves, roots)

    def only_moved(tree, side, path, collecting):
        current = version(side, path)
        if isinstance(current, OtherKind):
            return False
        if current is None:
            # 两边都没有（路径被 git 加了引号之类认不出的）：判不了，照算改动
            if path not in tree:
                return False
            target = move_target(path, moves)
            return target is not None and not crosses_governed(path, target) and same_after_moves(version(side, target), tree[path], collecting)
        source = path if path in tree else move_source(path, moves)
        return source is not None and source in tree and not crosses_governed(source, path) and same_after_moves(current, tree[source], collecting)

    def moved_only(path, collecting):
        return only_moved(base_tree, "worktree", path, collecting) \
            and (staged_entry(path) == head_tree.get(path) or only_moved(head_tree, "index", path, collecting))

    # 先只按模式与字节数筛、记下要读的对象，一次读完，再逐字节比
    survivors = [path for path in paths if moved_only(path, True)]
    blobs, problem = blob_bytes(wanted, cwd)
    if problem is not None:
        return set(), problem
    return {path for path in survivors if moved_only(path, False)}, None


def drop_moved_lines(base, entries, cwd=None):
    """这一次新增的行里剔掉只因此刻生效的登记搬迁才变了的：entries 是 [(路径, 行文, …)]（路径相对仓根，行文不带换行），交 (留下的 entries, 问题或 None)。
    每份文件在基准里取那一版（基准里有这条路径的取它自己，没有的取 move_source 恰好换前缀之前的那一条），与工作区里这条路径上的那一份按 diff 的位置配对（moved_line_numbers）：
    同一处（git diff -U0 的同一块）里被删的第 k 行照登记换算之后与新增的第 k 行相同、换算前后又不同的，这一行剔掉；删与加条数不等的块一行都不剔；
    取到的是搬迁之前的路径（整份搬了家）时，diff 里没动的那几行也剔掉。别处新加的同文行、原地留着旧行又另加的换算之后的行，照算新增。
    改了路径而不是恰好换前缀的、整份搬进别的按路径管的类的（governed_class 不同），基准里没有可比的那一版，新增行一条都不剔（弄坏开关 governed-lines-dropped 下跨类的照剔）。
    entry 第三项是整数的（工作区里那一份的行号，从 1 数）按行号剔；别的按行文在这份文件可剔的行里逐条抵、抵上一条剔一条。
    弄坏开关 lines-by-count 下退回不看位置：整份搬了家的每一行、原地改的换算前后不同的那几行都可抵，新增行按行文逐条抵。
    登记表没有内容行、没有生效的行、弄坏开关 moved-only-kept、被判的根不是仓根时原样交回；登记表写坏了、git 取不到时原样交回并交问题。"""
    entries = list(entries)
    if not entries:
        return entries, None
    moves, base_tree, problem = moved_only_inputs(base, cwd)
    if problem is not None or not moves:
        return entries, problem
    governed_kept = "governed-lines-dropped" not in breaks()
    sources = {}
    for path in dict.fromkeys(entry[0] for entry in entries):
        clean = path[2:] if path.startswith("./") else path
        candidate = clean if clean in base_tree else move_source(clean, moves)
        if candidate is None or candidate not in base_tree or base_tree[candidate].mode == GITLINK_MODE:
            continue
        if governed_kept and candidate != clean and governed_class(candidate) != governed_class(clean):
            continue
        sources[path] = (candidate, clean, base_tree[candidate].oid)
    blobs, problem = blob_bytes([oid for _candidate, _clean, oid in sources.values()], cwd)
    if problem is not None:
        return entries, problem
    by_count = "lines-by-count" in breaks()
    roots = default_roots(cwd)
    droppable = {}   # 路径 → (可剔的行号集合, 可剔的行文计数)
    for path, (candidate, clean, oid) in sources.items():
        before = blobs[oid].decode("utf-8", "surrogateescape")
        relocated = candidate != clean
        if by_count:
            before_lines, after_lines = text_lines(before), text_lines(rewrite_moved_paths(before, moves, roots))
            droppable[path] = (set(), collections.Counter(new for old, new in zip(before_lines, after_lines) if relocated or new != old))
            continue
        numbers, current_lines, problem = moved_line_numbers(before, os.path.join(cwd or ".", clean), moves, relocated, cwd)
        if problem is not None:
            return entries, problem
        droppable[path] = (numbers, collections.Counter(current_lines[number - 1] for number in numbers))
    kept = []
    for entry in entries:
        numbers, budget = droppable.get(entry[0], (set(), collections.Counter()))
        if not by_count and len(entry) >= 3 and isinstance(entry[2], int) and not isinstance(entry[2], bool):
            if entry[2] in numbers:
                continue
        elif budget[entry[1]] > 0:
            budget[entry[1]] -= 1
            continue
        kept.append(entry)
    return kept, None


def text_lines(text):
    """按 \\n 切行，结尾的 \\n 不多出一行空行（与 git diff 的行号对得上）。"""
    lines = text.split("\n")
    return lines[:-1] if text.endswith("\n") else lines


DIFF_HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")


def moved_line_numbers(before, current_path, moves, relocated, cwd=None):
    """基准那一版（before，原文）与工作区里 current_path 那一份按 diff 的位置配对，交 (可剔的行号集合, 工作区那一份的行, 问题或 None)：
    git diff --no-index -U0 的同一块里，被删的第 k 行照登记换算之后与新增的第 k 行相同、换算前后又不同的，那一行新增行可剔；
    删与加条数不等的块一行都不剔（弄坏开关 hunk-any-position 下退回：同一块里任一被删的行都能配，一行旧行只配一行新行）；
    relocated（整份搬了家）时 diff 里没动的行也可剔。工作区里不在、读不了的交空集。
    工作区那一份正是换算之后的基准时不起 git：原地改的可剔换算前后不同的那几行，整份搬了家的每一行都可剔。"""
    try:
        with open(current_path, "rb") as handle:
            current = handle.read().decode("utf-8", "surrogateescape")
    except OSError:
        return set(), [], None
    after = rewrite_moved_paths(before, moves, default_roots(cwd))
    before_lines, after_lines, current_lines = text_lines(before), text_lines(after), text_lines(current)
    if current == after:
        return {number for number, (old, new) in enumerate(zip(before_lines, after_lines), 1) if relocated or old != new}, current_lines, None
    if current == before:
        return (set(range(1, len(current_lines) + 1)) if relocated else set()), current_lines, None
    with tempfile.TemporaryDirectory() as directory:
        before_file = os.path.join(directory, "before")
        with open(before_file, "wb") as handle:
            handle.write(before.encode("utf-8", "surrogateescape"))
        shown = git_run(["diff", "--no-index", "--no-color", "--no-ext-diff", "--text", "-U0", "--", before_file, os.path.abspath(current_path)], cwd)
    if shown.returncode not in (0, 1):
        return set(), current_lines, BaselineProblem("工作区", current_path, "diff --no-index", shown.returncode, shown.stderr.decode("utf-8", "replace"))
    numbers, touched, hunk = set(), set(), None
    any_position = "hunk-any-position" in breaks()

    def settle(hunk):
        old_start, deleted, new_start, added = hunk
        touched.update(new_start + offset for offset in range(len(added)))
        if not any_position:
            if len(deleted) == len(added):
                for offset, text in enumerate(added):
                    old_number = old_start + offset
                    if 0 < old_number <= len(after_lines) and after_lines[old_number - 1] == text and before_lines[old_number - 1] != text:
                        numbers.add(new_start + offset)
            return
        used = [False] * len(deleted)
        for offset, text in enumerate(added):
            for index in range(len(deleted)):
                old_number = old_start + index
                if used[index] or not 0 < old_number <= len(after_lines):
                    continue
                if after_lines[old_number - 1] == text and before_lines[old_number - 1] != text:
                    used[index] = True
                    numbers.add(new_start + offset)
                    break

    for line in shown.stdout.decode("utf-8", "surrogateescape").split("\n"):
        found = DIFF_HUNK.match(line)
        if found:
            if hunk is not None:
                settle(hunk)
            hunk = (int(found.group(1)), [], int(found.group(3)), [])
        elif hunk is not None and line.startswith("-"):
            hunk[1].append(line[1:])
        elif hunk is not None and line.startswith("+"):
            hunk[3].append(line[1:])
    if hunk is not None:
        settle(hunk)
    if relocated:
        numbers |= set(range(1, len(current_lines) + 1)) - touched
    return numbers, current_lines, None


def path_move_problems(root, base=None):
    """搬迁登记表写对没有、指不指得到、新加的行有没有搬迁实据：(内容行数, [(说什么, 怎么办)])；判法同 path_move_report。"""
    count, problems, _note = path_move_report(root, base)
    return count, problems


def under_prefix(path, prefix):
    return path == prefix or path.startswith(prefix + "/")


def gate_base_of(root):
    """被判的根上的 gate 基准：在根上起 CHANGED_PATHS_LIBRARY 的 gate_diff_base gate，交 (基准, None)；
    根不在 git 仓里、不是仓根交 (None, None)（实据不核）；取不到交 (None, 一句话)。"""
    inside = git_run(["rev-parse", "--is-inside-work-tree"], root)
    if inside.returncode != 0 or inside.stdout.strip() != b"true":
        return None, None
    top_level, problem = at_top_level(root)
    if problem is not None:
        return None, problem.sentence()
    if not top_level:
        return None, None
    if not os.path.isfile(CHANGED_PATHS_LIBRARY):
        return None, f"找不到取 gate 基准的共用取法 {CHANGED_PATHS_LIBRARY}"
    shown = subprocess.run(["bash", "-c", 'source "$1" && gate_diff_base gate', "_", CHANGED_PATHS_LIBRARY], cwd=root, capture_output=True, text=True)
    base = shown.stdout.strip()
    if shown.returncode != 0 or not base:
        return None, f"取不到 gate 基准（{CHANGED_PATHS_LIBRARY} 的 gate_diff_base gate 退 {shown.returncode}：{shown.stderr.strip()[:80]}）"
    return base, None


def untracked_unignored(root, prefixes):
    """工作区里落在这几个前缀下、没被跟踪也没被忽略的文件（git ls-files --others --exclude-standard）：(集合, None)；git 取不到交 (空集, BaselineProblem)。"""
    if not prefixes:
        return set(), None
    listed = git_run(["--literal-pathspecs", "ls-files", "-z", "--others", "--exclude-standard", "--", *sorted(set(prefixes))], root)
    if listed.returncode != 0:
        return set(), BaselineProblem("工作区", "（没跟踪的文件）", "ls-files", listed.returncode, listed.stderr.decode("utf-8", "replace"))
    return {name.decode("utf-8", "surrogateescape") for name in listed.stdout.split(b"\0") if name}, None


def chained_rows(effective):
    """此刻生效的几行里串了的那几对：[(前一行, 后一行, 前一行的前缀, 后一行的前缀)]。任一行的旧前缀或新前缀等于、包含或落在另一行的旧前缀或新前缀下面就算串。"""
    pairs = []
    for index, first in enumerate(effective):
        for second in effective[index + 1:]:
            clash = next(((one, other) for one in (first.old, first.new) for other in (second.old, second.new)
                          if under_prefix(one, other) or under_prefix(other, one)), None)
            if clash is not None:
                pairs.append((first, second, *clash))
    return pairs


def not_a_move_sentences(move, subjects, occupied, shown_base):
    """一行登记不是一次搬迁的问题句：新前缀下有基准里本来就在的文件（occupied 非空），旧前缀在基准里没有文件（subjects 空）；都不是交 []。"""
    sentences = []
    if occupied:
        sentences.append(f"第 {move.line} 行的新前缀 {move.new} 下有基准 {shown_base} 里本来就在的文件（{'、'.join(occupied[:3])}{'…' if len(occupied) > 3 else ''}，"
                         f"共 {len(occupied)} 份）：这一行不是一次搬迁")
    if not subjects:
        sentences.append(f"第 {move.line} 行的旧前缀 {move.old} 在基准 {shown_base} 里没有文件：这一行不是一次搬迁")
    return sentences


def move_evidence(root, base, moves):
    """此刻生效的登记行串没串、有没有搬迁实据；不在基准那一版登记表里的新行不生效的，判红：(生效的行数, [(说什么, 怎么办)], 一句说核了什么)。
    哪几行生效照 effective_among（基准 base 那棵树与基准那一版的登记表）。
    新行必须生效：这一行不在基准那一版的登记表里，却不生效（旧前缀在基准里没有文件，或新前缀下有基准里本来就在的文件），判红「这一行不是一次搬迁」
    （弄坏开关 idle-new-row-unchecked 下不核）；已在基准那一版登记表里的行不生效是对的，只在成功句里点名是哪一条条件不满足。
    不许串：生效的几行之间，任一行的旧前缀或新前缀不许等于、包含或落在另一行的旧前缀或新前缀下面（chained_rows；弄坏开关 chain-allowed 下不核）。
    实据（每一行生效的）：基准里旧前缀下的每份文件，此刻在新前缀下照这一行换前缀的同一相对路径上都在，而且被 git 跟踪（在索引里，或是没被忽略的工作区文件；
    弄坏开关 untracked-landing-accepted 下只要在磁盘上）；新前缀在基准里没有文件；旧前缀此刻不在磁盘上。换算之后相同的份数只报不判。"""
    howto = ("登记表只登记真搬了家的目录或文件（git mv 之后再登记，新前缀是搬迁之前没有的路径、搬过去的文件都要跟踪着）；把命令里的目录换成另一个已有的目录、换了实现的，"
             "不登记，照真改动交审")
    idle_howto = ("登记要和搬迁写在同一次改动里：旧前缀在基准里没有文件的（搬迁已经进了 gate 基准、上一步还没进 gate 基准，或根本没搬），删掉这一行；"
                  "分几步搬的，搬一步、等 gate 基准前移（推上去，或 gate 基准那一版里已有新前缀）再登记下一步；"
                  "新前缀下基准里已有文件的（含只放着占位文件的目录）不是一次搬迁，要搬进那样的目录，先删掉占位文件、等 gate 基准前移，再搬再登记")
    chain_howto = ("此刻生效的几行不许串：一次改动里一条路径只搬一次；要分几步搬的，搬一步、等 gate 基准前移（推上去，或 gate 基准那一版里已有新前缀）再登记下一步")
    shown_base = base[:9] if re.fullmatch(r"[0-9a-f]{40}", base) else base
    tree, problem = tree_entries(base, root)
    if problem is not None:
        return 0, [(f"搬迁登记表哪几行生效判不了：{problem.sentence()}", "照括号里 git 的报错修仓库状态（基准名不在、对象库缺了那一份）再跑")], ""
    registered, problem = (set(), None) if "base-row-stays-effective" in breaks() else base_registered_pairs(base, root)
    if problem is not None:
        return 0, [(f"搬迁登记表哪几行生效判不了：{problem.sentence()}", "照括号里 git 的报错修仓库状态再跑")], ""
    effective, problem = effective_among(base, moves, root, tree=tree, registered=registered)
    if problem is not None:
        return 0, [(f"搬迁登记表哪几行生效判不了：{problem.sentence()}", "照括号里 git 的报错修仓库状态再跑")], ""
    files = [path for path, entry in tree.items() if entry.mode != GITLINK_MODE]
    problems, details, idle_reasons = [], [], []
    for move in moves:
        if move in effective:
            continue
        in_base_table = (move.old, move.new) in registered
        subjects = [path for path in files if under_prefix(path, move.old)]
        occupied = sorted(path for path in files if under_prefix(path, move.new))
        reasons = [f"已在基准 {shown_base} 那一版的登记表里"] if in_base_table else []
        if not subjects:
            reasons.append(f"旧前缀 {move.old} 在基准里没有文件")
        if occupied:
            reasons.append(f"新前缀 {move.new} 下基准里已有文件")
        idle_reasons.append((move, "、".join(reasons)))
        if not in_base_table and "idle-new-row-unchecked" not in breaks():
            problems += [(what, idle_howto) for what in not_a_move_sentences(move, subjects, occupied, shown_base)]
    idle = [move for move, _why in idle_reasons]
    if len(idle) == 1:
        idle_note = f"不生效 1 行（第 {idle[0].line} 行：{idle_reasons[0][1]}；门禁不按它换算，实据不核）"
    elif idle:
        idle_note = (f"不生效 {len(idle)} 行（第 {'、'.join(str(move.line) for move in idle)} 行：{'；'.join(f'第 {move.line} 行（{why}）' for move, why in idle_reasons)}；"
                     "门禁不按它们换算，实据不核）")
    else:
        idle_note = ""
    if not effective:
        return 0, problems, f"此刻生效的 0 行；{idle_note}"
    if "chain-allowed" not in breaks():
        for first, second, one, other in chained_rows(effective):
            problems.append((f"第 {first.line} 行与第 {second.line} 行串了（{one} 与 {other} 相同或一个在另一个下面）：此刻生效的几行，旧前缀、新前缀互相不许相同、包含或落在下面",
                             chain_howto))
    index, problem = index_entries(root)
    untracked, untracked_problem = untracked_unignored(root, [move.new for move in effective])
    if problem is not None or untracked_problem is not None:
        return len(effective), [(f"搬迁登记表的实据判不了：{(problem or untracked_problem).sentence()}", "照括号里 git 的报错修仓库状态再跑")], ""
    accept_untracked = "untracked-landing-accepted" in breaks()
    roots = default_roots(root)
    for move in effective:
        subjects = sorted(path for path in files if under_prefix(path, move.old))
        occupied = sorted(path for path in files if under_prefix(path, move.new))
        problems += [(what, howto) for what in not_a_move_sentences(move, subjects, [] if "occupied-unchecked" in breaks() else occupied, shown_base)]
        if os.path.lexists(os.path.join(root, move.old)):
            problems.append((f"第 {move.line} 行的旧前缀 {move.old} 还在磁盘上（搬完以后旧路径不该还在）",
                             "把旧路径下剩下的东西搬完（git mv 没搬全的补上）或删掉；没搬就删掉这一行登记"))
        if not subjects:
            continue
        blobs, problem = blob_bytes([tree[path].oid for path in subjects], root)
        if problem is not None:
            return len(effective), [(f"搬迁登记表的实据判不了：{problem.sentence()}", "照括号里 git 的报错修仓库状态再跑")], ""
        missing, unfollowed, same = [], [], 0
        for path in subjects:
            target = moved_path(path, move)
            current = worktree_version(root, target)
            if current is None or isinstance(current, OtherKind):
                missing.append(target)
                continue
            if not accept_untracked and target not in index and target not in untracked:
                unfollowed.append(target)
                continue
            mode, data = current
            before = blobs[tree[path].oid]
            same += mode == tree[path].mode and (data == before or data == rewrite_moved_bytes(before, effective, roots))
        if missing:
            problems.append((f"第 {move.line} 行：基准 {shown_base} 里旧前缀 {move.old} 下的 {len(subjects)} 份，照这一行换前缀的落点上有 {len(missing)} 份此刻不在"
                             f"（{'、'.join(missing[:3])}{'…' if len(missing) > 3 else ''}）", howto))
        if unfollowed:
            problems.append((f"第 {move.line} 行：照这一行换前缀的落点上有 {len(unfollowed)} 份没被 git 跟踪（被 .gitignore 忽略，或在 git 看不到的地方："
                             f"{'、'.join(unfollowed[:3])}{'…' if len(unfollowed) > 3 else ''}）", howto))
        if not missing and not unfollowed:
            details.append(f"第 {move.line} 行 {len(subjects)} 份里 {same} 份换算之后相同")
    note = f"此刻生效 {len(effective)} 行（第 {'、'.join(str(move.line) for move in effective)} 行）"
    note += f"，不串、有搬迁实据（{'；'.join(details)}；新前缀下没有基准里本来就在的文件）" if not problems else "，实据不够或串了，见上"
    if idle_note:
        note += f"；{idle_note}"
    return len(effective), problems, note


def path_move_report(root, base=None):
    """搬迁登记表写对没有、此刻生效的行串没串、有没有搬迁实据：(内容行数, [(说什么, 怎么办)], 一句说哪几行生效、实据核了什么)。不看弄坏开关 path-moves-ignored。
    每一行都核写法与出处文件在；生效的行另核不许串与实据（move_evidence）。base 是 gate 基准，不给就在根上取（gate_base_of）；
    根不在 git 仓里、不是仓根时判不了哪几行生效，实据不核，那一句写明；弄坏开关 move-evidence-unchecked 下串与实据都不核。"""
    table_text = ""
    table = os.path.join(root, PATH_MOVES_TABLE)
    if os.path.isfile(table):
        try:
            with open(table, encoding="utf-8") as handle:
                table_text = handle.read()
        except (OSError, UnicodeDecodeError):
            table_text = ""
    count = len(content_rows(table_text))
    moves, format_problems = read_path_moves(root)
    problems = [(problem, f"照写法改：一行 {PATH_MOVE_ROW_SHAPE}，前缀写相对仓根的路径（不带开头的 / 与 ./、不带结尾的 /）") for problem in format_problems]
    for move in moves:
        if not os.path.isfile(os.path.join(root, move.source)):
            problems.append((f"第 {move.line} 行的出处 {move.source} 不在", "出处写记这次搬迁的那份 records/<文件>.md，文件名照仓里现有的抄"))
    if not moves:
        return count, problems, "没有内容行，实据不用核"
    if "move-evidence-unchecked" in breaks():
        return count, problems, f"弄坏开关 {BREAK_VARIABLE}=move-evidence-unchecked 生效：此刻生效的登记行串没串、有没有搬迁实据没核"
    if base is None:
        base, base_problem = gate_base_of(root)
        if base_problem is not None:
            return count, problems + [(f"搬迁登记表的实据判不了：{base_problem}", "照括号里的报错修好再跑；取不到基准不是「没有生效的行」")], ""
    if not base:
        return count, problems, "被判的根不在 git 仓里（或不是仓根、没给基准），判不了哪几行此刻生效：门禁一行都不按它换算，串与实据没核"
    _effective, evidence_problems, note = move_evidence(root, base, moves)
    return count, problems + evidence_problems, note


def baseline_text_at(base, path, cwd=None):
    """基准提交里 path 这一条路径上的那一版（不看搬迁登记）：三种结局同 baseline_text。"""
    listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-tree", "--name-only", base, "--", path], capture_output=True, text=True, cwd=cwd)
    problem = None
    if listed.returncode != 0:
        problem = BaselineProblem(base, path, "ls-tree", listed.returncode, listed.stderr)
    elif not listed.stdout.strip():
        return None, None
    else:
        relative = path[2:] if path.startswith("./") else path
        shown_path = relative if "show-root-relative" in breaks() else f"./{relative}"
        shown = subprocess.run(["git", "-c", "core.quotepath=false", "show", f"{base}:{shown_path}"], capture_output=True, text=True, cwd=cwd)
        if shown.returncode == 0:
            return shown.stdout, None
        problem = BaselineProblem(base, path, "show", shown.returncode, shown.stderr)
    if "unreadable-as-absent" in breaks():
        print(f"  ! 弄坏开关 {BREAK_VARIABLE}=unreadable-as-absent 生效：{problem.sentence()}，当成基准里没有这张表", file=sys.stderr)
        return None, None
    return None, problem


def baseline_rows(base, path, cwd=None):
    """基准那一版的内容行：(内容行, None)；基准里没有这张表 (None, None)；git 取不到 (None, BaselineProblem)。"""
    text, problem = baseline_text(base, path, cwd)
    return (content_rows(text) if text is not None else None), problem


class ReplayRow(NamedTuple):
    """复跑表的一行：实验号、第 2 列（二进制或 @驱动名）、参数、产物、方式（exact / timing）。"""
    experiment: str
    driver: str
    arguments: str
    product: str
    mode: str


def replay_rows(path):
    """复跑表 path 里的复跑行（整份文件逐行认 REPLAY_ROW）；文件不在或读不了交 None。"""
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    return [ReplayRow(*found.groups()) for found in map(REPLAY_ROW.match, text.split("\n")) if found]


def show_main(base, path):
    try:
        text, problem = baseline_text(base, path)
    except ValueError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE}，或只写认得的项", file=sys.stderr)
        return 2
    if problem is not None:
        print(problem.sentence(), file=sys.stderr)
        return 1
    if text is None:
        return EXIT_ABSENT
    sys.stdout.write(text)
    return 0


def path_moves_check_main(root, base=None):
    count, problems, note = path_move_report(root, base)
    if problems:
        print(f"  ✗ 搬迁登记表 {PATH_MOVES_TABLE} 有 {len(problems)} 处不对（共 {count} 行）：")  # gate-lint:summary
        for what, _howto in problems:
            print(f"     {what}")  # gate-lint:detail
        print(f"     → 怎么办：{'；'.join(dict.fromkeys(howto for _what, howto in problems))}")
        return 1
    print(f"  ✓ 搬迁登记表 {PATH_MOVES_TABLE} {count} 行：写法对、出处都在；{note}")
    return 0


def replay_rows_main(path):
    rows = replay_rows(path)
    if rows is None:
        print(f"  ✗ 读不了复跑表 {path}（不在，或没有读权限）", file=sys.stderr)
        print("     → 怎么办：核对路径；replay.sh 搬了家就把调用方点名的路径一起改（.claude/rules/path-moves.md）", file=sys.stderr)
        return 1
    for row in rows:
        print("\t".join(row))
    return 0


def report_moved_only_problem(problem, what):
    """moved-only-filter / moved-lines-filter 的问题怎么报：登记表写坏了打一行提示、照没有登记时交回（退 0）；git 取不到判红（退 1）。"""
    if isinstance(problem, PathMovesProblem):
        print(f"  ! {problem.sentence()}；{what}没剔除只差搬迁的，照没有登记时交回", file=sys.stderr)
        return 0
    print(f"  ✗ {problem.sentence()}，{what}里哪些只差登记的搬迁判不了", file=sys.stderr)
    print("     → 怎么办：照括号里 git 的报错修仓库状态（基准名不在、对象库缺了那一份）再跑；判不了时不交半截结果", file=sys.stderr)
    return 1


def moved_only_filter_main(base):
    """标准输入一行一条路径，标准输出交回其中不只差登记搬迁的那几条（次序不变）。"""
    paths = [line for line in sys.stdin.buffer.read().decode("utf-8", "surrogateescape").split("\n") if line]
    moved, problem = moved_only_paths(base, paths)
    if problem is not None:
        status = report_moved_only_problem(problem, "改动范围")
        if status:
            return status
    kept = [path for path in paths if (path[2:] if path.startswith("./") else path) not in moved]
    sys.stdout.buffer.write("".join(path + "\n" for path in kept).encode("utf-8", "surrogateescape"))
    return 0


def moved_lines_filter_main(base):
    """标准输入一行一条「路径<TAB>行文」（scripts/gate/changed-paths.sh 的 gate_added_lines），标准输出交回剔掉只因登记搬迁才变了的行之后的那些（次序不变）。"""
    entries = []
    for line in sys.stdin.buffer.read().decode("utf-8", "surrogateescape").split("\n"):
        if line:
            path, _tab, text = line.partition("\t")
            entries.append((path, text, line))
    kept, problem = drop_moved_lines(base, entries)
    if problem is not None:
        status = report_moved_only_problem(problem, "新增行")
        if status:
            return status
    sys.stdout.buffer.write("".join(entry[2] + "\n" for entry in kept).encode("utf-8", "surrogateescape"))
    return 0


def path_move_selftest(check):
    """搬迁登记那几项：临时 git 仓里提交 scripts/old-place 下的脚本与几张表，再 git mv 到 scripts/new-place，换着写登记表取基准那一版。"""
    with tempfile.TemporaryDirectory() as directory:
        def run_git(*arguments):
            subprocess.run(["git", "-C", directory, *arguments], check=True, capture_output=True)

        def write(relative, text):
            os.makedirs(os.path.dirname(os.path.join(directory, relative)) or directory, exist_ok=True)
            with open(os.path.join(directory, relative), "w", encoding="utf-8") as handle:
                handle.write(text)

        def register(*rows):
            write(PATH_MOVES_TABLE, "# 搬迁登记（自证）\n\n" + "".join(row + "\n" for row in rows))

        run_git("init", "-q")
        run_git("config", "user.email", "selftest@example.invalid")
        run_git("config", "user.name", "selftest")
        script_before = "#!/bin/sh\n# 自己在 scripts/old-place/run.sh；./scripts/old-place/x 也换，scripts/old-place2 与 scripts/old-place.md 不换\necho scripts/old-place\n"
        table_before = ("scripts/old-place/run.sh:nproc\t理由\nscripts/old-place2/keep.sh\t名字长一截的不换\nscripts/old-place.md\t接 .md 的不换\n"
                        "scripts/old-place\t串尾\n$ROOT/scripts/old-place/run.sh\t前面接 / 的换\nxscripts/old-place/run.sh\t前面接字母的不换\n")
        table_after = ("scripts/new-place/run.sh:nproc\t理由\nscripts/old-place2/keep.sh\t名字长一截的不换\nscripts/old-place.md\t接 .md 的不换\n"
                       "scripts/new-place\t串尾\n$ROOT/scripts/new-place/run.sh\t前面接 / 的换\nxscripts/old-place/run.sh\t前面接字母的不换\n")
        write("scripts/old-place/run.sh", script_before)
        write("scripts/old-place/inner-lag.tsv", "甲\tscripts/old-place/run.sh\n")
        write("lag.tsv", table_before)
        write("records/2026-01-01-搬迁.md", "# 搬迁\n")
        run_git("add", "-A")
        run_git("commit", "-qm", "基准")
        run_git("mv", "scripts/old-place", "scripts/new-place")
        script_after = script_before.replace("scripts/old-place/", "scripts/new-place/").replace("echo scripts/old-place\n", "echo scripts/new-place\n")
        write("scripts/new-place/run.sh", script_after)

        text, problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        moved_table, moved_problem = baseline_text("HEAD", "scripts/new-place/inner-lag.tsv", cwd=directory)
        register("# 只有注释")
        commented, commented_problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        check("搬迁登记：一行都没有时逐字节相同", text == table_before and problem is None and commented == table_before and commented_problem is None
              and moved_table is None and moved_problem is None, f"取到 {text!r}、{commented!r}、{moved_table!r}")

        register("scripts/old-place\tscripts/new-place\t2026-01-01\trecords/2026-01-01-搬迁.md")
        text, problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        shown = subprocess.run([sys.executable, os.path.realpath(__file__), "show", "HEAD", "lag.tsv"], capture_output=True, text=True, cwd=directory)
        check("搬迁登记：旧前缀换成新前缀，只按前缀边界换", text == table_after and problem is None and shown.returncode == 0 and shown.stdout == table_after,
              f"取到 {text!r}、{problem}；命令行退 {shown.returncode}、{shown.stdout!r}")

        moved_table, moved_problem = baseline_text("HEAD", "scripts/new-place/inner-lag.tsv", cwd=directory)
        check("搬迁登记：按搬迁之前的路径取表", moved_table == "甲\tscripts/new-place/run.sh\n" and moved_problem is None, f"取到 {moved_table!r}、{moved_problem}")

        data, taken, file_problem = baseline_file_bytes("HEAD", "scripts/new-place/run.sh", cwd=directory)
        moves, _problems = path_moves(directory)
        with open(os.path.join(directory, "scripts/new-place/run.sh"), "rb") as handle:
            current = handle.read()
        edited = current.replace(b"echo scripts/new-place\n", b"echo scripts/new-place again\n")
        check("搬迁登记：文件按搬迁之前的路径取、换算之后与搬完的相同",
              file_problem is None and taken == "scripts/old-place/run.sh" and data == script_before.encode("utf-8")
              and rewrite_moved_bytes(data, moves) == current and rewrite_moved_bytes(data, moves) != edited,
              f"取到 {taken}、{data!r}、{file_problem}")

        register("scripts/ancient-place\tscripts/old-place\t2026-01-01\trecords/2026-01-01-搬迁.md",
                 "scripts/old-place\tscripts/new-place\t2026-01-01\trecords/2026-01-01-搬迁.md")
        text, problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        moved_table, moved_problem = baseline_text("HEAD", "scripts/new-place/inner-lag.tsv", cwd=directory)
        effective, effective_problem = effective_moves("HEAD", directory)
        idle_count, idle_problems, idle_note = path_move_report(directory, "HEAD")
        check("搬迁登记：门禁只用此刻生效的行（基准里旧前缀下没有文件的那一行不换，又是新行，登记表那一格判红「不是一次搬迁」）", text == table_after and problem is None
              and moved_table == "甲\tscripts/new-place/run.sh\n" and moved_problem is None and effective_problem is None
              and [move.line for move in effective] == [4] and idle_count == 2 and "此刻生效 1 行（第 4 行）" in idle_note
              and len(idle_problems) == 2 and "第 3 行的新前缀 scripts/old-place 下有基准 HEAD 里本来就在的文件" in idle_problems[0][0]
              and "第 3 行的旧前缀 scripts/ancient-place 在基准 HEAD 里没有文件：这一行不是一次搬迁" in idle_problems[1][0]
              and "不生效 1 行（第 3 行" in idle_note,
              f"取到 {text!r}、{problem}、{moved_table!r}、{moved_problem}；生效 {effective}、{effective_problem}；判法 {idle_count} 行、{idle_problems}、{idle_note!r}")

        register("scripts/old-place\tscripts/new-place\t2026-01-01")
        text, problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        shown = subprocess.run([sys.executable, os.path.realpath(__file__), "show", "HEAD", "lag.tsv"], capture_output=True, text=True, cwd=directory)
        check("搬迁登记：写坏的登记表交问题句", text is None and isinstance(problem, PathMovesProblem)
              and problem.sentence().startswith(f"搬迁登记表 {PATH_MOVES_TABLE} 写得不对") and "有 3 列" in problem.sentence()
              and shown.returncode == 1 and "搬迁登记表" in shown.stderr, f"取到 {text!r}、{problem}；命令行退 {shown.returncode}、{shown.stderr.strip()!r}")

        write("scripts/old-place/left-behind.sh", "echo 没搬全\n")
        register("scripts/old-place\tscripts/new-place\t2026-01-01\trecords/2026-01-01-搬迁.md",
                 "scripts/elsewhere\tscripts/nowhere\t2026-01-01\trecords/2026-01-01-搬迁.md",
                 "scripts/a\tscripts/b\t2026-13-01\trecords/2026-01-01-搬迁.md",
                 "/scripts/c\tscripts/new-place\t2026-01-01\tnotes/搬迁.md",
                 "scripts/d\tscripts/new-place\t2026-01-01\trecords/没有这一份.md")
        count, problems, note = path_move_report(directory, "HEAD")
        whats = "\n".join(what for what, _howto in problems)
        expected = ("第 3 行的旧前缀 scripts/old-place 还在磁盘上", "第 5 行日期「2026-13-01」不是 YYYY-MM-DD",
                    "第 6 行旧前缀「/scripts/c」要写相对仓根的路径", "出处「notes/搬迁.md」不是 records/<文件>.md", "第 7 行的出处 records/没有这一份.md 不在")
        check("搬迁登记：登记表的判法（每行核写法与出处，生效的那一行另核旧前缀不在磁盘上；不生效的新行只判「不是一次搬迁」、不核磁盘）",
              count == 5 and len(problems) == 6 and all(item in whats for item in expected) and "scripts/nowhere" not in whats
              and "第 4 行的旧前缀 scripts/elsewhere 在基准 HEAD 里没有文件：这一行不是一次搬迁" in whats
              and "第 7 行的旧前缀 scripts/d 在基准 HEAD 里没有文件：这一行不是一次搬迁" in whats
              and "不生效 2 行（第 4、7 行" in note, f"{count} 行、问题 {whats!r}、{note!r}")


def moved_only_selftest(check):
    """只差搬迁那几项：临时 git 仓里把 tools/old-rig 搬到 tools/new-rig（一份 git mv、一份只在磁盘上 mv），
    造只差搬迁的（改名的、原地换了路径的、留着提交引用的）与还改了别处的（多一行、只改模式、暂存区里多一行）文件，逐条判。"""
    with tempfile.TemporaryDirectory() as directory:
        def run_git(*arguments):
            subprocess.run(["git", "-C", directory, *arguments], check=True, capture_output=True)

        def write(relative, text, executable=False):
            full = os.path.join(directory, relative)
            os.makedirs(os.path.dirname(full) or directory, exist_ok=True)
            with open(full, "w", encoding="utf-8") as handle:
                handle.write(text)
            os.chmod(full, 0o755 if executable else 0o644)

        def register(*rows):
            write(PATH_MOVES_TABLE, "# 搬迁登记（自证）\n\n" + "".join(row + "\n" for row in rows))

        def command_line(name, data):
            completed = subprocess.run([sys.executable, os.path.realpath(__file__), name, "HEAD"], input=data.encode("utf-8"),
                                       capture_output=True, cwd=directory)
            return completed.returncode, completed.stdout.decode("utf-8"), completed.stderr.decode("utf-8")

        rewritten = "git show 31e47115:tools/old-rig/run.sh 与 31e47115:./tools/old-rig/run.sh 不换；HEAD:tools/old-rig/run.sh、abc123:tools/old-rig/run.sh 与 tools/old-rig/run.sh 换"
        expected_rewrite = "git show 31e47115:tools/old-rig/run.sh 与 31e47115:./tools/old-rig/run.sh 不换；HEAD:tools/new-rig/run.sh、abc123:tools/new-rig/run.sh 与 tools/new-rig/run.sh 换"
        one_move = [PathMove(1, "tools/old-rig", "tools/new-rig", "2026-01-01", "records/2026-01-01-搬迁.md")]
        got = rewrite_moved_paths(rewritten, one_move)
        check("搬迁登记：提交引用后面的路径不换", got == expected_rewrite, f"换成 {got!r}")

        run_git("init", "-q")
        run_git("config", "user.email", "selftest@example.invalid")
        run_git("config", "user.name", "selftest")
        write("tools/old-rig/run.sh", "#!/bin/sh\n# 在 tools/old-rig/run.sh\n", executable=True)
        write("tools/old-rig/data.txt", "看 tools/old-rig/run.sh\n")
        write("notes.md", "见 tools/old-rig/run.sh\n改前 git show 31e47115:tools/old-rig/run.sh\n")
        write("mixed.md", "见 tools/old-rig/run.sh\n第一版\n")
        write("mode.sh", "echo tools/old-rig\n")
        write("staged.md", "见 tools/old-rig/run.sh\n")
        write("partial.md", "见 tools/old-rig/run.sh 与 tools/old-rig/data.txt\n")
        write("records/2026-01-01-搬迁.md", "# 搬迁\n")
        run_git("add", "-A")
        run_git("commit", "-qm", "基准")
        os.makedirs(os.path.join(directory, "tools/new-rig"))
        run_git("mv", "tools/old-rig/run.sh", "tools/new-rig/run.sh")
        os.rename(os.path.join(directory, "tools/old-rig/data.txt"), os.path.join(directory, "tools/new-rig/data.txt"))
        os.rmdir(os.path.join(directory, "tools/old-rig"))
        write("tools/new-rig/run.sh", "#!/bin/sh\n# 在 tools/new-rig/run.sh\n", executable=True)
        write("tools/new-rig/data.txt", "看 tools/new-rig/run.sh\n")
        write("notes.md", "见 tools/new-rig/run.sh\n改前 git show 31e47115:tools/old-rig/run.sh\n")
        write("mixed.md", "见 tools/new-rig/run.sh\n第二版\n")
        write("mode.sh", "echo tools/new-rig\n", executable=True)
        write("staged.md", "见 tools/new-rig/run.sh\n暂存区里多一行\n")
        run_git("add", "staged.md")
        write("staged.md", "见 tools/new-rig/run.sh\n")
        write("partial.md", "见 tools/new-rig/run.sh 与 tools/old-rig/data.txt\n")   # 只换了一半（第一轮 G2-1 的形状）：照算改动
        paths = ["tools/new-rig/run.sh", "tools/old-rig/run.sh", "tools/old-rig/data.txt", "tools/new-rig/data.txt", "notes.md",
                 "mixed.md", "mode.sh", "staged.md", "partial.md", PATH_MOVES_TABLE]
        moved_expected = {"tools/new-rig/run.sh", "tools/old-rig/run.sh", "tools/old-rig/data.txt", "tools/new-rig/data.txt", "notes.md"}
        entries = [("mixed.md", "见 tools/new-rig/run.sh"), ("mixed.md", "第二版"), ("mixed.md", "见 tools/new-rig/run.sh"),
                   ("tools/new-rig/data.txt", "看 tools/new-rig/run.sh"), ("notes.md", "见 tools/new-rig/run.sh"),
                   ("notes.md", "改前 git show 31e47115:tools/old-rig/run.sh")]
        lines_expected = [("mixed.md", "第二版"), ("mixed.md", "见 tools/new-rig/run.sh"), ("notes.md", "改前 git show 31e47115:tools/old-rig/run.sh")]

        register("# 只有注释")
        moved, problem = moved_only_paths("HEAD", paths, cwd=directory)
        kept, lines_problem = drop_moved_lines("HEAD", entries, cwd=directory)
        check("只差搬迁：登记表没有内容行时一条都不剔", moved == set() and problem is None and kept == entries and lines_problem is None,
              f"剔了 {sorted(moved)}、{problem}；新增行留下 {kept}、{lines_problem}")

        register("tools/old-rig\ttools/new-rig\t2026-01-01\trecords/2026-01-01-搬迁.md")
        moved, problem = moved_only_paths("HEAD", paths, cwd=directory)
        check("只差搬迁：改名的、原地换了路径的、只删了旧路径的不算改动，多一行、只改模式、暂存区里多一行、只换了一半的照算",
              moved == moved_expected and problem is None, f"算只差搬迁的 {sorted(moved)}，应当 {sorted(moved_expected)}；{problem}")
        kept, lines_problem = drop_moved_lines("HEAD", entries, cwd=directory)
        check("只差搬迁：新增行剔掉只因搬迁才变了的行，不带行号的按行文在可剔的行里抵", kept == lines_expected and lines_problem is None, f"留下 {kept}；{lines_problem}")
        paths_status, paths_out, _paths_err = command_line("moved-only-filter", "".join(path + "\n" for path in paths))
        lines_status, lines_out, _lines_err = command_line("moved-lines-filter", "".join(f"{path}\t{text}\n" for path, text in entries))
        check("只差搬迁：命令行 moved-only-filter 与 moved-lines-filter",
              paths_status == 0 and paths_out == "".join(path + "\n" for path in paths if path not in moved_expected)
              and lines_status == 0 and lines_out == "".join(f"{path}\t{text}\n" for path, text in lines_expected),
              f"moved-only-filter 退 {paths_status}、{paths_out!r}；moved-lines-filter 退 {lines_status}、{lines_out!r}")

        register("tools/old-rig\ttools/new-rig\t2026-01-01")
        broken_status, broken_out, broken_err = command_line("moved-only-filter", "".join(path + "\n" for path in paths))
        register("tools/old-rig\ttools/new-rig\t2026-01-01\trecords/2026-01-01-搬迁.md")
        unreadable = subprocess.run([sys.executable, os.path.realpath(__file__), "moved-only-filter", "没有这个基准"], input=b"mixed.md\n",
                                    capture_output=True, cwd=directory)
        check("只差搬迁：登记表写坏了照没有登记时交回、提示一行；基准取不到退 1 不交半截",
              broken_status == 0 and broken_out == "".join(path + "\n" for path in paths) and "写得不对" in broken_err
              and unreadable.returncode == 1 and unreadable.stdout == b"",
              f"写坏的表退 {broken_status}、{broken_out!r}、{broken_err.strip()!r}；取不到的基准退 {unreadable.returncode}、{unreadable.stdout!r}")


class SelftestRepository:
    """自证的临时 git 仓：写文件、提交、git 命令。"""

    def __init__(self, directory, fresh=True):
        self.root = directory
        if fresh:
            self.git("init", "-q")
            self.git("config", "user.email", "selftest@example.invalid")
            self.git("config", "user.name", "selftest")

    def git(self, *arguments):
        subprocess.run(["git", "-C", self.root, *arguments], check=True, capture_output=True)

    def write(self, relative, text):
        full = os.path.join(self.root, relative)
        os.makedirs(os.path.dirname(full) or self.root, exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(text)

    def remove(self, relative):
        os.remove(os.path.join(self.root, relative))

    def register(self, *rows):
        self.write(PATH_MOVES_TABLE, "# 搬迁登记（自证）\n" + "".join("\t".join(row) + "\n" for row in rows))

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-qm", message)


SELFTEST_SOURCE = "records/2026-01-01-搬迁.md"


def selftest_row(old, new):
    """自证里登记表的一行（日期与出处写死）。"""
    return (old, new, "2026-01-01", SELFTEST_SOURCE)


def selftest_copy(directory, base_root, name):
    """把 base_root 那个临时仓整份拷成 directory 下的 name，交它的 SelftestRepository。"""
    target = os.path.join(directory, name)
    subprocess.run(["cp", "-a", base_root, target], check=True)
    return SelftestRepository(target, fresh=False)


def problem_whats(root):
    """path_move_report 交基准 HEAD 判出的问题句。"""
    return [what for what, _howto in path_move_report(root, "HEAD")[1]]


def path_move_evidence_selftest(check):
    """此刻生效的行要有搬迁实据（move_evidence）：临时 git 仓里提交 scripts/old-rig 下三份文件、点名它的 docs/howto.md、写着 build/ 的 .gitignore
    与一张已有一行（scripts/ancient-rig → scripts/old-rig，基准里旧前缀下没有文件，不生效）的登记表，在几份副本上各造一种局面，path_move_report 交基准 HEAD 判：
    真搬迁（三份里两份只换了路径）照过、点名它的 docs/howto.md 剔出改动范围；搬丢一份的、落点在被 .gitignore 忽略的目录里的（第二轮 H1-3）判红；
    换实现的（旧目录删掉、新目录里同名文件是另一份实现，第二轮辩方）实据照过，换了的那几份文件照算改动、新增行一条不剔。"""
    with tempfile.TemporaryDirectory() as directory:
        base_root = os.path.join(directory, "base")
        os.makedirs(base_root)
        repository = SelftestRepository(base_root)
        repository.write("scripts/old-rig/run.sh", "# 在 scripts/old-rig/run.sh\necho old\n")
        repository.write("scripts/old-rig/data.txt", "看 scripts/old-rig/run.sh\n")
        repository.write("scripts/old-rig/notes.txt", "笔记\n")
        repository.write("docs/howto.md", "复现：bash scripts/old-rig/run.sh\n")
        repository.write(".gitignore", "build/\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.register(selftest_row("scripts/ancient-rig", "scripts/old-rig"))
        repository.commit("基准")

        moved = selftest_copy(directory, base_root, "moved")
        moved.git("mv", "scripts/old-rig", "scripts/new-rig")
        moved.write("scripts/new-rig/run.sh", "# 在 scripts/new-rig/run.sh\necho old\n")
        moved.write("scripts/new-rig/data.txt", "看 scripts/new-rig/run.sh\n")
        moved.write("scripts/new-rig/notes.txt", "笔记，搬完又改了一行\n")
        moved.write("docs/howto.md", "复现：bash scripts/new-rig/run.sh\n")
        moved.register(selftest_row("scripts/ancient-rig", "scripts/old-rig"), selftest_row("scripts/old-rig", "scripts/new-rig"))
        moved_count, moved_problems, moved_note = path_move_report(moved.root, "HEAD")
        moved_paths, moved_problem = moved_only_paths("HEAD", ["docs/howto.md", "scripts/new-rig/notes.txt"], cwd=moved.root)

        partial = selftest_copy(directory, base_root, "partial")
        partial.git("mv", "scripts/old-rig", "scripts/new-rig")
        partial.remove("scripts/new-rig/notes.txt")
        partial.register(selftest_row("scripts/old-rig", "scripts/new-rig"))
        partial_whats = problem_whats(partial.root)

        ignored = selftest_copy(directory, base_root, "ignored")
        ignored.git("rm", "-q", "-r", "scripts/old-rig")
        for name in ("run.sh", "data.txt", "notes.txt"):
            ignored.write(f"build/old-rig/{name}", "echo 早就在、没进 git 的另一份\n")
        ignored.write("docs/howto.md", "复现：bash build/old-rig/run.sh\n")
        ignored.register(selftest_row("scripts/old-rig", "build/old-rig"))
        ignored_whats = problem_whats(ignored.root)

        rewritten = selftest_copy(directory, base_root, "rewritten")
        rewritten.git("rm", "-q", "-r", "scripts/old-rig")
        rewritten.write("scripts/new-rig/run.sh", "echo 另一份实现\n")
        rewritten.write("scripts/new-rig/data.txt", "别的数据\n")
        rewritten.write("scripts/new-rig/notes.txt", "别的笔记\n")
        rewritten.write("docs/howto.md", "复现：bash scripts/new-rig/run.sh\n")
        rewritten.register(selftest_row("scripts/old-rig", "scripts/new-rig"))
        rewritten_whats = problem_whats(rewritten.root)
        replaced = ["scripts/new-rig/run.sh", "scripts/new-rig/data.txt", "scripts/new-rig/notes.txt"]
        rewritten_moved, rewritten_problem = moved_only_paths("HEAD", replaced + ["docs/howto.md"], cwd=rewritten.root)
        replaced_lines = [(path, text, 1) for path, text in zip(replaced, ("echo 另一份实现", "别的数据", "别的笔记"))]
        rewritten_kept, rewritten_lines_problem = drop_moved_lines("HEAD", replaced_lines, cwd=rewritten.root)
        check("搬迁登记：此刻生效的行要有搬迁实据（真搬迁照过；搬丢一份、落点被 .gitignore 忽略的判红；换实现的实据照过、换了的文件与新增行照算改动）",
              moved_count == 2 and moved_problems == [] and "此刻生效 1 行（第 3 行）" in moved_note and "3 份里 2 份换算之后相同" in moved_note
              and "不生效 1 行（第 2 行" in moved_note and moved_paths == {"docs/howto.md"} and moved_problem is None
              and any("有 1 份此刻不在（scripts/new-rig/notes.txt）" in what for what in partial_whats)
              and any("没被 git 跟踪" in what and "build/old-rig/run.sh" in what for what in ignored_whats)
              and rewritten_whats == [] and rewritten_moved == {"docs/howto.md"} and rewritten_problem is None
              and rewritten_kept == replaced_lines and rewritten_lines_problem is None,
              f"真搬迁 {moved_count} 行、{moved_problems}、{moved_note!r}、剔了 {sorted(moved_paths)}；搬丢一份 {partial_whats}；落点被忽略 {ignored_whats}；"
              f"换实现 {rewritten_whats}、剔了 {sorted(rewritten_moved)}、新增行留下 {rewritten_kept}")


def path_move_occupied_selftest(check):
    """不在基准那一版登记表里的新行必须此刻生效，否则登记表那一格判红「这一行不是一次搬迁」（第三轮 J1-C）：
    基准里两份实现 tools/real-a、tools/real-b 与起 A 的 ops/launch.sh、limits/config.py 一行 RETRIES = 3。
    deleted：删掉 tools/real-a、ops/launch.sh 改成起 B，登记 tools/real-a → tools/real-b（第一轮 G1-2）；
    parent：两个目录都不动，ops/launch.sh 改成起 B，登记 tools/real-a → tools/real-b 与 not-on-disk → tools（第一轮 G1-1）；
    token：limits/config.py 改成 RETRIES = 4，登记 3 → 4（单个词当前缀，旧前缀下基准里没有文件）。
    三种都一行不生效：起它的改动（ops/launch.sh、limits/config.py）照算改动、新增行照留，登记表那一格判红，问题句点名是哪一条条件不满足。"""
    with tempfile.TemporaryDirectory() as directory:
        base_root = os.path.join(directory, "base")
        os.makedirs(base_root)
        repository = SelftestRepository(base_root)
        repository.write("tools/real-a/run.sh", "echo 走的是 A\n")
        repository.write("tools/real-b/run.sh", "echo 走的是 B\n")
        repository.write("ops/launch.sh", "bash tools/real-a/run.sh\n")
        repository.write("limits/config.py", "RETRIES = 3\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("基准")
        outcomes = {}
        for name, rows, remove, edited, line in (
                ("deleted", [selftest_row("tools/real-a", "tools/real-b")], True, "ops/launch.sh", "bash tools/real-b/run.sh"),
                ("parent", [selftest_row("tools/real-a", "tools/real-b"), selftest_row("not-on-disk", "tools")], False, "ops/launch.sh", "bash tools/real-b/run.sh"),
                ("token", [selftest_row("3", "4")], False, "limits/config.py", "RETRIES = 4")):
            copy = selftest_copy(directory, base_root, name)
            if remove:
                copy.git("rm", "-q", "-r", "tools/real-a")
            copy.write(edited, line + "\n")
            copy.register(*rows)
            effective, effective_problem = effective_moves("HEAD", copy.root)
            moved, moved_problem = moved_only_paths("HEAD", [edited], cwd=copy.root)
            kept, lines_problem = drop_moved_lines("HEAD", [(edited, line, 1)], cwd=copy.root)
            outcomes[name] = (effective, effective_problem, moved, moved_problem, kept, lines_problem, path_move_report(copy.root, "HEAD"))
        sentences = {name: "\n".join(what for what, _howto in outcome[6][1]) for name, outcome in outcomes.items()}
        check("搬迁登记：不在基准那一版登记表里的新行必须此刻生效，否则判红「不是一次搬迁」（新前缀下早有文件、旧前缀下没有文件），起它的改动照算",
              all(effective == [] and effective_problem is None and moved == set() and moved_problem is None and len(kept) == 1 and lines_problem is None
                  and report[1] != [] and "此刻生效的 0 行" in report[2] for effective, effective_problem, moved, moved_problem, kept, lines_problem, report in outcomes.values())
              and "第 2 行的新前缀 tools/real-b 下有基准 HEAD 里本来就在的文件" in sentences["deleted"]
              and "第 2 行的新前缀 tools/real-b 下有基准 HEAD 里本来就在的文件" in sentences["parent"]
              and "第 3 行的旧前缀 not-on-disk 在基准 HEAD 里没有文件：这一行不是一次搬迁" in sentences["parent"]
              and "第 2 行的旧前缀 3 在基准 HEAD 里没有文件：这一行不是一次搬迁" in sentences["token"],
              f"{outcomes}")


def path_move_window_selftest(check):
    """生效窗口（提交之后这一行对门禁不再起作用）：临时 git 仓里先提交 scripts/old-rig、点名它的 docs/howto.md 与 records/log.md 里一行抄下来的
    git log 命令（留旧路径），再 git mv 到 scripts/new-rig、改 docs/howto.md、登记，连同登记表一起提交（模拟「这一批提交之后」，基准里已经是新前缀）。
    此刻：这一行不生效；baseline_text 交出的 records/log.md 原样（旧路径不被换走）；有人把留旧路径的那一行改成新路径，照算改动、新增行照留（第二轮 H1-4 的另一半）；
    钉值那一侧读的 path_moves 照旧有这一行。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        kept_line = "git log --format=%s -- scripts/old-rig/run.sh"
        repository.write("scripts/old-rig/run.sh", "echo 跑\n")
        repository.write("docs/howto.md", "复现：bash scripts/old-rig/run.sh\n")
        repository.write("records/log.md", kept_line + "\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("更早")
        repository.git("mv", "scripts/old-rig", "scripts/new-rig")
        repository.write("docs/howto.md", "复现：bash scripts/new-rig/run.sh\n")
        repository.register(selftest_row("scripts/old-rig", "scripts/new-rig"))
        repository.commit("搬迁那一次")
        effective, effective_problem = effective_moves("HEAD", directory)
        all_moves, _all_problems = path_moves(directory)
        text, text_problem = baseline_text("HEAD", "records/log.md", cwd=directory)
        count, problems, note = path_move_report(directory, "HEAD")
        rewritten_line = kept_line.replace("scripts/old-rig", "scripts/new-rig")
        repository.write("records/log.md", rewritten_line + "\n")
        moved, moved_problem = moved_only_paths("HEAD", ["records/log.md"], cwd=directory)
        kept, lines_problem = drop_moved_lines("HEAD", [("records/log.md", rewritten_line, 1)], cwd=directory)
        check("搬迁登记：生效窗口，提交进基准之后这一行对门禁不再起作用（基准不换、改回新路径的那一行照算改动），钉值那一侧照读得到",
              effective == [] and effective_problem is None and len(all_moves) == 1 and text == kept_line + "\n" and text_problem is None
              and count == 1 and problems == [] and "此刻生效的 0 行" in note and moved == set() and moved_problem is None
              and kept == [("records/log.md", rewritten_line, 1)] and lines_problem is None,
              f"生效 {effective}、{effective_problem}；全部 {len(all_moves)} 行；基准那一版 {text!r}、{text_problem}；判法 {count} 行、{problems}、{note!r}；"
              f"剔了 {sorted(moved)}、{moved_problem}；新增行留下 {kept}、{lines_problem}")


def path_move_reused_name_selftest(check):
    """旧名字重新用起来（第二轮 H1-4）：临时 git 仓里先把 scripts/old-rig 搬到 scripts/new-rig、登记并提交；再把 tools/new-thing 搬进空出来的 scripts/old-rig，
    lag.tsv 里那一行跟着换路径、登记 tools/new-thing → scripts/old-rig。提交之前与提交之后（工作区一个字节不改）各取一次基准那一版的 lag.tsv：都要与此刻逐字相同（不判涨）。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        repository.write("scripts/old-rig/run.sh", "echo 旧的那一份\n")
        repository.write("tools/new-thing/live.sh", "echo 新东西\n")
        repository.write("lag.tsv", "tools/new-thing/live.sh\t理由\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("更早")
        repository.git("mv", "scripts/old-rig", "scripts/new-rig")
        repository.register(selftest_row("scripts/old-rig", "scripts/new-rig"))
        repository.commit("第一次搬迁")
        repository.git("mv", "tools/new-thing", "scripts/old-rig")
        repository.write("lag.tsv", "scripts/old-rig/live.sh\t理由\n")
        repository.register(selftest_row("scripts/old-rig", "scripts/new-rig"), selftest_row("tools/new-thing", "scripts/old-rig"))
        before_text, before_problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        before_count, before_problems, before_note = path_move_report(directory, "HEAD")
        repository.commit("第二次搬迁")
        after_text, after_problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        after_count, after_problems, after_note = path_move_report(directory, "HEAD")
        current = "scripts/old-rig/live.sh\t理由\n"
        check("搬迁登记：旧名字重新用起来，提交前后没改的表都不判涨（提交之后老行不再换基准）",
              before_text == current and before_problem is None and after_text == current and after_problem is None
              and before_problems == [] and "此刻生效 1 行（第 3 行）" in before_note and after_problems == [] and "此刻生效的 0 行" in after_note,
              f"提交前 {before_text!r}、{before_problem}、{before_problems}、{before_note!r}；提交后 {after_text!r}、{after_problem}、{after_problems}、{after_note!r}")


def path_move_reordered_selftest(check):
    """登记行换次序、再起一个中间目录（第二轮辩方的无序键）：临时 git 仓里 rig-x → rig-a、rig-a → rig-b 两次真搬迁都已提交，docs/note.md 留一行指 rig-x/foo.sh 的旧说法；
    此刻把两行换次序、另起一个 rig-a/foo.sh、docs/note.md 那一行改指 rig-a/foo.sh：两行都不生效，docs/note.md 照算改动、新增行照留。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        repository.write("rig-x/foo.sh", "echo foo\n")
        repository.write("docs/note.md", "当年在 rig-x/foo.sh 跑过\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("更早")
        repository.git("mv", "rig-x", "rig-a")
        repository.register(selftest_row("rig-x", "rig-a"))
        repository.commit("第一次搬迁")
        repository.git("mv", "rig-a", "rig-b")
        repository.register(selftest_row("rig-x", "rig-a"), selftest_row("rig-a", "rig-b"))
        repository.commit("第二次搬迁")
        repository.register(selftest_row("rig-a", "rig-b"), selftest_row("rig-x", "rig-a"))
        repository.write("rig-a/foo.sh", "echo 随便什么\n")
        repository.write("docs/note.md", "当年在 rig-a/foo.sh 跑过\n")
        effective, effective_problem = effective_moves("HEAD", directory)
        moved, moved_problem = moved_only_paths("HEAD", ["docs/note.md"], cwd=directory)
        kept, lines_problem = drop_moved_lines("HEAD", [("docs/note.md", "当年在 rig-a/foo.sh 跑过", 1)], cwd=directory)
        check("搬迁登记：已提交的两行换了次序、另起中间目录，两行都不生效，改了的那一行照算改动",
              effective == [] and effective_problem is None and moved == set() and moved_problem is None and len(kept) == 1 and lines_problem is None,
              f"生效 {effective}、{effective_problem}；剔了 {sorted(moved)}、{moved_problem}；新增行留下 {kept}、{lines_problem}")


def path_move_inserted_row_selftest(check):
    """新行插在已提交的老行前面（第二轮 H1-1）：临时 git 仓里 old-b → tools/real-b 已搬迁、登记并提交，基准里另有 tools/real-a 与起它的 ops/launch.sh；
    此刻删掉 tools/real-a、ops/launch.sh 改成起 B，在老行前面插一行 tools/real-a → old-b。老行不生效，新行生效而 old-b 下什么都没有：登记表判红，ops/launch.sh 照算改动。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        repository.write("old-b/run.sh", "echo 走的是 B\n")
        repository.write("tools/real-a/run.sh", "echo 走的是 A\n")
        repository.write("ops/launch.sh", "bash tools/real-a/run.sh\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("更早")
        os.makedirs(os.path.join(directory, "tools"), exist_ok=True)
        repository.git("mv", "old-b", "tools/real-b")
        repository.register(selftest_row("old-b", "tools/real-b"))
        repository.commit("基准")
        repository.git("rm", "-q", "-r", "tools/real-a")
        repository.write("ops/launch.sh", "bash tools/real-b/run.sh\n")
        repository.register(selftest_row("tools/real-a", "old-b"), selftest_row("old-b", "tools/real-b"))
        effective, effective_problem = effective_moves("HEAD", directory)
        whats = problem_whats(directory)
        moved, moved_problem = moved_only_paths("HEAD", ["ops/launch.sh"], cwd=directory)
        check("搬迁登记：新行插在老行前面串到早就在的实现上，老行不生效、新行没有实据判红，起它的改动照算",
              [move.line for move in effective] == [2] and effective_problem is None
              and any("第 2 行" in what and "有 1 份此刻不在（old-b/run.sh）" in what for what in whats)
              and moved == set() and moved_problem is None,
              f"生效 {effective}、{effective_problem}；问题 {whats}；剔了 {sorted(moved)}、{moved_problem}")


def path_move_chain_selftest(check):
    """此刻生效的行不许串：临时 git 仓里 old/x/f.sh、old/y/g.sh 搬成 new/x/f.sh、new/y/g.sh，另拷一份 other/f.sh，登记 old → new 与 old/x → other：
    两行都生效、实据都齐，只红在「串了」。另一份副本造来回登记（第二轮 H1-2）：tools/live → build/out、build/out → tools/live，什么都没搬，红在实据。"""
    with tempfile.TemporaryDirectory() as directory:
        base_root = os.path.join(directory, "base")
        os.makedirs(base_root)
        repository = SelftestRepository(base_root)
        repository.write("old/x/f.sh", "echo f\n")
        repository.write("old/y/g.sh", "echo g\n")
        repository.write("tools/live/run.sh", "echo 仓里那一份\n")
        repository.write("ops/launch.sh", "bash build/out/run.sh\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("基准")
        nested = selftest_copy(directory, base_root, "nested")
        for relative in ("new/x", "new/y", "other"):
            os.makedirs(os.path.join(nested.root, relative))
        nested.git("mv", "old/y/g.sh", "new/y/g.sh")
        nested.git("mv", "old/x/f.sh", "new/x/f.sh")
        for relative in ("old/x", "old/y", "old"):
            if os.path.isdir(os.path.join(nested.root, relative)):
                os.rmdir(os.path.join(nested.root, relative))
        nested.write("other/f.sh", "echo f\n")
        nested.git("add", "other/f.sh")
        nested.register(selftest_row("old", "new"), selftest_row("old/x", "other"))
        nested_whats = problem_whats(nested.root)
        round_trip = selftest_copy(directory, base_root, "round-trip")
        round_trip.write("ops/launch.sh", "bash tools/live/run.sh\n")
        round_trip.register(selftest_row("tools/live", "build/out"), selftest_row("build/out", "tools/live"))
        round_trip_whats = problem_whats(round_trip.root)
        round_trip_moved, round_trip_problem = moved_only_paths("HEAD", ["ops/launch.sh"], cwd=round_trip.root)
        check("搬迁登记：此刻生效的行不许串（同一前缀下两行判红；来回登记红在实据、起它的改动照算）",
              len(nested_whats) == 1 and "第 2 行与第 3 行串了（old 与 old/x" in nested_whats[0]
              and any("第 2 行：" in what and "有 1 份此刻不在（build/out/run.sh）" in what for what in round_trip_whats)
              and any("第 2 行的旧前缀 tools/live 还在磁盘上" in what for what in round_trip_whats)
              and round_trip_moved == set() and round_trip_problem is None,
              f"同一前缀下两行 {nested_whats}；来回登记 {round_trip_whats}、剔了 {sorted(round_trip_moved)}、{round_trip_problem}")


def moved_lines_position_selftest(check):
    """新增行按 diff 的位置配对（drop_moved_lines）：临时 git 仓里把 scripts/old-rig 搬到 scripts/new-rig、docs/old 搬到 docs/new；
    原地留着旧行又在别的节加一行换算之后的同文行、整份搬了家又把行挪到别的节、原地留着旧行又多写一行新前缀的拷贝，这几行照算新增；原地只换了路径的那一行剔掉。
    另造两种：同一块里删加条数相等、挪了节的那一行与被删的旧行不在同一个位置（第二轮 H1-6），删加条数不等的块里换算之后相同的那一行，都照算新增。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        repository.write("scripts/old-rig/x.sh", "echo x\n")
        repository.write("kb/page.md", "## 正文\n结论甲\n## 历史\n- 旧说法见 scripts/old-rig/x.sh\n")
        repository.write("docs/old/page.md", "## 正文\n结论乙\n## 历史\n- 旧说法见 scripts/old-rig/x.sh\n")
        repository.write("docs/copy.md", "bash scripts/old-rig/x.sh\n")
        repository.write("notes.md", "开头\n见 scripts/old-rig/x.sh\n结尾\n")
        repository.write("kb/relocated.md", "## 正文\n结论丙\n- 结论丙的复跑见 scripts/old-rig/x.sh\n## 历史\n- 2026-01-01 立页\n")
        repository.write("kb/uneven.md", "开头\n见 scripts/old-rig/x.sh\n多余的一行\n结尾\n")
        repository.write("records/2026-01-01-搬迁.md", "# 搬迁\n")
        repository.commit("基准")
        repository.git("mv", "scripts/old-rig", "scripts/new-rig")
        repository.git("mv", "docs/old", "docs/new")
        repository.write("kb/page.md", "## 正文\n结论甲\n- 旧说法见 scripts/new-rig/x.sh\n## 历史\n- 旧说法见 scripts/old-rig/x.sh\n")
        repository.write("docs/new/page.md", "## 正文\n- 旧说法见 scripts/new-rig/x.sh\n## 历史\n结论乙\n")
        repository.write("docs/copy.md", "bash scripts/old-rig/x.sh\nbash scripts/new-rig/x.sh\n")
        repository.write("notes.md", "开头\n见 scripts/new-rig/x.sh\n结尾\n")
        repository.write("kb/relocated.md", "## 正文\n结论丙（改）\n## 历史（改）\n- 结论丙的复跑见 scripts/new-rig/x.sh\n- 2026-01-01 立页\n")
        repository.write("kb/uneven.md", "开头\n见 scripts/new-rig/x.sh\n结尾\n")
        repository.register(("scripts/old-rig", "scripts/new-rig", "2026-01-01", "records/2026-01-01-搬迁.md"),
                            ("docs/old", "docs/new", "2026-01-01", "records/2026-01-01-搬迁.md"))
        relocated_lines = ["## 正文", "- 旧说法见 scripts/new-rig/x.sh", "## 历史", "结论乙"]
        entries = [("kb/page.md", "- 旧说法见 scripts/new-rig/x.sh"), ("docs/copy.md", "bash scripts/new-rig/x.sh"), ("notes.md", "见 scripts/new-rig/x.sh"),
                   ("kb/relocated.md", "- 结论丙的复跑见 scripts/new-rig/x.sh"), ("kb/uneven.md", "见 scripts/new-rig/x.sh")]
        entries += [("docs/new/page.md", text) for text in relocated_lines]
        numbered = [("kb/page.md", "- 旧说法见 scripts/new-rig/x.sh", 3), ("docs/copy.md", "bash scripts/new-rig/x.sh", 2), ("notes.md", "见 scripts/new-rig/x.sh", 2),
                    ("kb/relocated.md", "- 结论丙的复跑见 scripts/new-rig/x.sh", 4), ("kb/uneven.md", "见 scripts/new-rig/x.sh", 2)]
        numbered += [("docs/new/page.md", text, number) for number, text in enumerate(relocated_lines, 1)]
        kept, problem = drop_moved_lines("HEAD", entries, cwd=directory)
        kept_numbered, numbered_problem = drop_moved_lines("HEAD", numbered, cwd=directory)
        must_stay = {("kb/page.md", "- 旧说法见 scripts/new-rig/x.sh"), ("docs/copy.md", "bash scripts/new-rig/x.sh"), ("docs/new/page.md", "- 旧说法见 scripts/new-rig/x.sh"),
                     ("kb/relocated.md", "- 结论丙的复跑见 scripts/new-rig/x.sh"), ("kb/uneven.md", "见 scripts/new-rig/x.sh")}
        must_go = {("notes.md", "见 scripts/new-rig/x.sh"), ("docs/new/page.md", "## 正文")}
        check("只差搬迁：新增行按 diff 的位置配对（同一块里第 k 行配第 k 行，删加条数不等的块不剔），别处加的同文行照算新增",
              problem is None and numbered_problem is None and must_stay <= {entry[:2] for entry in kept} and not must_go & {entry[:2] for entry in kept}
              and {entry[:2] for entry in kept} == {entry[:2] for entry in kept_numbered},
              f"按行文留下 {kept}、{problem}；按行号留下 {kept_numbered}、{numbered_problem}")


def governed_moves_selftest(check):
    """整份搬家而新旧路径的按路径管的类不同（moved_only_paths、drop_moved_lines）：临时 git 仓里用单文件登记把 drafts/agents/reviewer.md 搬进 .claude/agents/、
    research/proto/fast_path.rs 搬进 crates/core/src/、.claude/rules/draft.md 搬进 .claude/kb/（第二轮辩方：跨管辖类单文件搬家）、
    drafts/gates/check.sh 搬进 .claude/gate.d/ 顶层、drafts/sop/shared.md 搬进 .claude/singlefs-ai-sop/rules/（第三轮 J2-5：这两处也是按路径起作用的类），
    再把 .claude/rules/old-group 整个目录在同一类里改名、.claude/gate.d/fixtures/old-sample 在 gate.d 的子目录里改名（子目录不算 gate.d 那一类）；
    跨类的五对新旧路径照算改动、新增行一条不剔，同一类里只换了目录的、子目录里改名的照剔。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        repository.write("drafts/agents/reviewer.md", "---\nname: reviewer\n---\n")
        repository.write("research/proto/fast_path.rs", "pub fn fast_path() -> u32 { 7 }\n")
        repository.write(".claude/rules/draft.md", "# 草稿\n见 drafts/agents/reviewer.md\n")
        repository.write(".claude/rules/old-group/x.md", "# 规则\n")
        repository.write("drafts/gates/check.sh", "#!/usr/bin/env bash\necho 检查\n")
        repository.write("drafts/sop/shared.md", "# 草稿\n不许跑重型测试\n")
        repository.write(".claude/gate.d/fixtures/old-sample/case.txt", "样本\n")
        repository.write("records/2026-01-01-搬迁.md", "# 搬迁\n")
        repository.commit("基准")
        for relative in ("crates/core/src", ".claude/kb", ".claude/agents", ".claude/singlefs-ai-sop/rules"):
            os.makedirs(os.path.join(directory, relative))
        pairs = (("drafts/agents/reviewer.md", ".claude/agents/reviewer.md"), ("research/proto/fast_path.rs", "crates/core/src/fast_path.rs"),
                 (".claude/rules/draft.md", ".claude/kb/draft.md"), (".claude/rules/old-group", ".claude/rules/new-group"),
                 ("drafts/gates/check.sh", ".claude/gate.d/check.sh"), ("drafts/sop/shared.md", ".claude/singlefs-ai-sop/rules/shared.md"),
                 (".claude/gate.d/fixtures/old-sample", ".claude/gate.d/fixtures/new-sample"))
        for old, new in pairs:
            repository.git("mv", old, new)
        repository.write(".claude/kb/draft.md", "# 草稿\n见 .claude/agents/reviewer.md\n")
        repository.register(*(selftest_row(old, new) for old, new in pairs))
        paths = [".claude/agents/reviewer.md", "drafts/agents/reviewer.md", "crates/core/src/fast_path.rs", "research/proto/fast_path.rs",
                 ".claude/kb/draft.md", ".claude/rules/draft.md", ".claude/rules/old-group/x.md", ".claude/rules/new-group/x.md",
                 ".claude/gate.d/check.sh", "drafts/gates/check.sh", ".claude/singlefs-ai-sop/rules/shared.md", "drafts/sop/shared.md",
                 ".claude/gate.d/fixtures/old-sample/case.txt", ".claude/gate.d/fixtures/new-sample/case.txt"]
        moved, problem = moved_only_paths("HEAD", paths, cwd=directory)
        lines = [(".claude/kb/draft.md", "# 草稿", 1), (".claude/kb/draft.md", "见 .claude/agents/reviewer.md", 2), (".claude/agents/reviewer.md", "name: reviewer", 2),
                 (".claude/gate.d/check.sh", "echo 检查", 2), (".claude/singlefs-ai-sop/rules/shared.md", "不许跑重型测试", 2)]
        kept, lines_problem = drop_moved_lines("HEAD", lines, cwd=directory)
        check("只差搬迁：整份搬进别的按路径管的类（含 .claude/gate.d/ 顶层、.claude/singlefs-ai-sop/rules/）照算改动、新增行一条不剔，同一类里只换了目录的、gate.d 子目录里改名的照剔",
              moved == {".claude/rules/old-group/x.md", ".claude/rules/new-group/x.md", ".claude/gate.d/fixtures/old-sample/case.txt", ".claude/gate.d/fixtures/new-sample/case.txt"}
              and problem is None and kept == lines and lines_problem is None,
              f"剔了 {sorted(moved)}；{problem}；新增行留下 {kept}；{lines_problem}")


def renamed_files_selftest(check):
    """改了路径的文件只在恰好换前缀时剔（move_source、move_target）：临时 git 仓里用单文件登记把 .claude/agents/reviewer.md.draft 改名成 .claude/agents/reviewer.md、
    crates/core/src/fast_path.rs.txt 改名成 crates/core/src/fast_path.rs（第二轮 H1-5：同一类里从挑不中的名字改成挑得中的名字），
    再用目录登记把 tools/old-rig 搬到 tools/new-rig（文件名不变）：换了文件名的两对照算改动、新增行一条不剔，只换了目录的照剔。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        repository.write(".claude/agents/reviewer.md.draft", "---\nname: reviewer\n---\n放行一切\n")
        repository.write("crates/core/src/fast_path.rs.txt", "pub fn fast_path() -> u32 { 7 }\n")
        repository.write("tools/old-rig/run.sh", "echo 跑\n")
        repository.write("records/2026-01-01-搬迁.md", "# 搬迁\n")
        repository.commit("基准")
        for old, new in ((".claude/agents/reviewer.md.draft", ".claude/agents/reviewer.md"), ("crates/core/src/fast_path.rs.txt", "crates/core/src/fast_path.rs"),
                         ("tools/old-rig", "tools/new-rig")):
            repository.git("mv", old, new)
        repository.register(*(selftest_row(old, new) for old, new in (
            (".claude/agents/reviewer.md.draft", ".claude/agents/reviewer.md"), ("crates/core/src/fast_path.rs.txt", "crates/core/src/fast_path.rs"),
            ("tools/old-rig", "tools/new-rig"))))
        paths = [".claude/agents/reviewer.md.draft", ".claude/agents/reviewer.md", "crates/core/src/fast_path.rs.txt", "crates/core/src/fast_path.rs",
                 "tools/old-rig/run.sh", "tools/new-rig/run.sh"]
        moved, problem = moved_only_paths("HEAD", paths, cwd=directory)
        lines = [(".claude/agents/reviewer.md", "放行一切", 4), ("crates/core/src/fast_path.rs", "pub fn fast_path() -> u32 { 7 }", 1)]
        kept, lines_problem = drop_moved_lines("HEAD", lines, cwd=directory)
        check("只差搬迁：改了路径的文件只在恰好换前缀时剔（单个文件登记换了文件名的照算改动、新增行一条不剔）",
              moved == {"tools/old-rig/run.sh", "tools/new-rig/run.sh"} and problem is None and kept == lines and lines_problem is None,
              f"剔了 {sorted(moved)}；{problem}；新增行留下 {kept}；{lines_problem}")


def path_move_anchor_selftest(check):
    """换算只认仓根起的前缀（第三轮 J1-A）：前缀紧挨在 / 后面时，/ 前面要是 ./ 与 ../、$变量/、${变量}/、<仓根>/、本仓的绝对路径；
    别的目录名后面的同名段（vendor/tools、$FROZEN/tree/tools）不换。文本层逐种认；再在临时 git 仓里登记 docs → manual（真搬迁），
    另把没登记的 tools/docs/run.sh 换成 tools/manual/run.sh 的另一份实现、排除表那一行与调用方跟着改：表的基准那一版、调用方、新增行都不被当成只差搬迁。"""
    with tempfile.TemporaryDirectory() as directory:
        move = PathMove(1, "tools", "tools2", "2026-01-01", SELFTEST_SOURCE)
        replaced = ["bash tools/a.sh", "读 tools 再起", "(tools/a.sh)", "x=tools/a.sh", "./tools/a.sh", "../../tools/a.sh", "$ROOT/tools/a.sh", "${ROOT}/tools/a.sh",
                    '"$ROOT"/tools/a.sh', "<仓根>/tools/a.sh", f"{directory}/tools/a.sh"]
        kept = ["vendor/tools/run.sh", "$FROZEN/tree/tools/run.sh", "/tools/a.sh", "/other/root/tools/a.sh", f"{directory}-copy/tools/a.sh", "x./tools/a.sh", "a/./tools/a.sh"]
        wrong = []
        for text in replaced:
            at = text.rindex("tools")
            expected = text[:at] + "tools2" + text[at + len("tools"):]
            got = rewrite_moved_paths(text, [move], (directory,))
            if got != expected:
                wrong.append(f"该换没换 {text!r} → {got!r}")
        for text in kept:
            got = rewrite_moved_paths(text, [move], (directory,))
            if got != text:
                wrong.append(f"不该换却换了 {text!r} → {got!r}")
        repository = SelftestRepository(directory)
        repository.write("docs/guide.md", "见 docs/howto.md\n")
        repository.write("docs/howto.md", "# 怎么做\n")
        repository.write("tools/docs/run.sh", 'echo "本机核数 $(nproc)"\n')
        repository.write("lag.tsv", "tools/docs/run.sh:nproc\t只打核数，不是重活\n")
        repository.write("ci.sh", "bash tools/docs/run.sh\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("基准")
        repository.git("mv", "docs", "manual")
        repository.write("manual/guide.md", "见 manual/howto.md\n")
        repository.git("rm", "-q", "tools/docs/run.sh")
        repository.write("tools/manual/run.sh", 'exec make -j"$(nproc)" all\n')
        repository.write("lag.tsv", "tools/manual/run.sh:nproc\t只打核数，不是重活\n")
        repository.write("ci.sh", "bash tools/manual/run.sh\n")
        repository.register(selftest_row("docs", "manual"))
        count, problems, note = path_move_report(directory, "HEAD")
        base_table, table_problem = baseline_text("HEAD", "lag.tsv", cwd=directory)
        moved, moved_problem = moved_only_paths("HEAD", ["ci.sh", "lag.tsv", "manual/guide.md", "manual/howto.md"], cwd=directory)
        entries = [("ci.sh", "bash tools/manual/run.sh", 1), ("lag.tsv", "tools/manual/run.sh:nproc\t只打核数，不是重活", 1), ("manual/guide.md", "见 manual/howto.md", 1)]
        kept_lines, lines_problem = drop_moved_lines("HEAD", entries, cwd=directory)
        check("搬迁登记：换算只认仓根起的前缀（./ ../ $变量/ ${变量}/ <仓根>/ 本仓绝对路径），别的目录名后面的同名段不换，排除表的基准那一版、调用方、新增行照留",
              not wrong and problems == [] and "此刻生效 1 行" in note and base_table == "tools/docs/run.sh:nproc\t只打核数，不是重活\n" and table_problem is None
              and moved == {"manual/guide.md", "manual/howto.md"} and moved_problem is None and kept_lines == entries[:2] and lines_problem is None,
              f"文本层 {wrong}；登记表 {count} 行、{problems}、{note!r}；基准那一版 {base_table!r}、{table_problem}；剔了 {sorted(moved)}、{moved_problem}；新增行留下 {kept_lines}、{lines_problem}")


def path_move_base_row_selftest(check):
    """基准那一版登记表里已有的行不再生效（第三轮 Y，关睡着的老行醒来）：临时 git 仓里 rig → rig2 真搬迁、登记并提交；之后 rig2 又改名 rig3、
    旧名字 rig 下另起一份新东西 rig/new.sh，排除表 exempt.tsv 提交时写 rig/new.sh。基准是这一次提交：基准树满足「旧前缀下有文件、新前缀下没有文件」，
    可那一行已在基准那一版的登记表里，不生效：登记表那一格不红，没改的表不被换成 rig2/new.sh、不判涨。"""
    with tempfile.TemporaryDirectory() as directory:
        repository = SelftestRepository(directory)
        repository.write("rig/a.sh", "echo a\n")
        repository.write(SELFTEST_SOURCE, "# 搬迁\n")
        repository.commit("更早")
        repository.git("mv", "rig", "rig2")
        repository.register(selftest_row("rig", "rig2"))
        repository.commit("搬迁已提交")
        repository.git("mv", "rig2", "rig3")
        repository.write("rig/new.sh", "echo 旧名字上的新东西\n")
        repository.write("exempt.tsv", "rig/new.sh\t豁免\n")
        repository.commit("新目录又改了名、旧名字另起一份新东西")
        effective, effective_problem = effective_moves("HEAD", directory)
        count, problems, note = path_move_report(directory, "HEAD")
        text, text_problem = baseline_text("HEAD", "exempt.tsv", cwd=directory)
        moved, moved_problem = moved_only_paths("HEAD", ["rig/new.sh", "exempt.tsv"], cwd=directory)
        check("搬迁登记：基准那一版登记表里已有的行不生效（旧名字重新有了文件、新前缀又空了也不醒来），没改的表不被换算",
              effective == [] and effective_problem is None and count == 1 and problems == [] and "此刻生效的 0 行" in note
              and f"已在基准 {'HEAD'} 那一版的登记表里" in note and text == "rig/new.sh\t豁免\n" and text_problem is None and moved == set() and moved_problem is None,
              f"生效 {effective}、{effective_problem}；判法 {count} 行、{problems}、{note!r}；基准那一版 {text!r}、{text_problem}；剔了 {sorted(moved)}、{moved_problem}")


def path_move_base_shape_selftest(check):
    """生效窗口跟着 gate 基准走，基准不总是 HEAD（第三轮 J1-B、J3）：临时 git 仓有上游（本地的裸仓），基准提交 rig → … 推上去，
    搬迁（git mv、登记、点名它的 docs/howto.md 一起改）提交而不推。三种基准各判一次：
    ① 仓有上游、搬迁已提交没推：基准取上游的 merge-base，这一行生效、实据齐、docs/howto.md 剔出改动范围；
    ② GATE_BASE=HEAD~1（gate.sh --staged 推完之后里层的基准）：同样生效；
    ③ 推上去之后基准取 HEAD：不生效；这时才补登记的新行（旧前缀在基准里没有文件）判红「不是一次搬迁」。"""
    saved = os.environ.pop("GATE_BASE", None)
    try:
        with tempfile.TemporaryDirectory() as directory:
            remote = os.path.join(directory, "origin.git")
            subprocess.run(["git", "init", "-q", "--bare", remote], check=True, capture_output=True)
            root = os.path.join(directory, "work")
            os.makedirs(root)
            repository = SelftestRepository(root)
            repository.write("scripts/old-rig/run.sh", "# 在 scripts/old-rig/run.sh\necho old\n")
            repository.write("docs/howto.md", "复现：bash scripts/old-rig/run.sh\n")
            repository.write(SELFTEST_SOURCE, "# 搬迁\n")
            repository.commit("基准")
            first = subprocess.run(["git", "-C", root, "rev-parse", "HEAD"], check=True, capture_output=True, text=True).stdout.strip()
            repository.git("remote", "add", "origin", remote)
            repository.git("push", "-q", "-u", "origin", "HEAD:refs/heads/selftest")
            repository.git("mv", "scripts/old-rig", "scripts/new-rig")
            repository.write("scripts/new-rig/run.sh", "# 在 scripts/new-rig/run.sh\necho old\n")
            repository.write("docs/howto.md", "复现：bash scripts/new-rig/run.sh\n")
            repository.register(selftest_row("scripts/old-rig", "scripts/new-rig"))
            repository.commit("搬迁")
            upstream_base, upstream_problem = gate_base_of(root)
            upstream_effective, _ = effective_moves(upstream_base or "HEAD", root)
            upstream_report = path_move_report(root)
            upstream_moved, _ = moved_only_paths(upstream_base or "HEAD", ["docs/howto.md"], cwd=root)
            os.environ["GATE_BASE"] = "HEAD~1"
            try:
                staged_base, staged_problem = gate_base_of(root)
                staged_effective, _ = effective_moves(staged_base or "HEAD", root)
                staged_report = path_move_report(root)
            finally:
                os.environ.pop("GATE_BASE", None)
            repository.git("push", "-q", "origin", "HEAD:refs/heads/selftest")
            pushed_base, _ = gate_base_of(root)
            pushed_effective, _ = effective_moves(pushed_base or "HEAD", root)
            pushed_report = path_move_report(root)
            repository.register(selftest_row("scripts/old-rig", "scripts/new-rig"), selftest_row("scripts/older-rig", "scripts/old-rig"))
            late_report = path_move_report(root)
            check("搬迁登记：生效窗口跟着 gate 基准走（仓有上游、搬迁已提交没推取 merge-base；GATE_BASE=HEAD~1；推上去之后取 HEAD 不生效，这时才补登记的新行判红）",
                  upstream_problem is None and upstream_base == first and [move.line for move in upstream_effective] == [2]
                  and upstream_report[1] == [] and "此刻生效 1 行（第 2 行）" in upstream_report[2] and upstream_moved == {"docs/howto.md"}
                  and staged_problem is None and staged_base == "HEAD~1" and [move.line for move in staged_effective] == [2]
                  and staged_report[1] == [] and "此刻生效 1 行（第 2 行）" in staged_report[2]
                  and pushed_effective == [] and pushed_report[1] == [] and "此刻生效的 0 行" in pushed_report[2]
                  and any("第 3 行的旧前缀 scripts/older-rig 在基准" in what and "这一行不是一次搬迁" in what for what, _howto in late_report[1]),
                  f"上游 {upstream_base}、{upstream_problem}、生效 {upstream_effective}、{upstream_report}、剔了 {sorted(upstream_moved)}；"
                  f"HEAD~1 {staged_base}、{staged_problem}、生效 {staged_effective}、{staged_report}；推上去 {pushed_base}、生效 {pushed_effective}、{pushed_report}；补登记 {late_report}")
    finally:
        if saved is not None:
            os.environ["GATE_BASE"] = saved


def selftest():
    results = []

    def check(name, ok, detail=""):
        results.append((name, ok, detail))

    with tempfile.TemporaryDirectory() as directory:
        def run_git(*arguments):
            subprocess.run(["git", "-C", directory, *arguments], check=True, capture_output=True)

        table = "lag.tsv"
        run_git("init", "-q")
        run_git("config", "user.email", "selftest@example.invalid")
        run_git("config", "user.name", "selftest")
        with open(os.path.join(directory, table), "w", encoding="utf-8") as handle:
            handle.write("# 表头注释\n\n甲\t理由一\r\n  # 缩进的注释\n乙\t理由二\n")
        with open(os.path.join(directory, "other.txt"), "w", encoding="utf-8") as handle:
            handle.write("占位\n")
        # 子目录里一份同名的表（被判的根是别的仓里的一个子目录时，取的是它，不是仓根上那一份）
        os.mkdir(os.path.join(directory, "sub"))
        with open(os.path.join(directory, "sub", table), "w", encoding="utf-8") as handle:
            handle.write("子目录\t那一份\n")
        run_git("add", "-A")
        run_git("commit", "-qm", "基准")

        rows = content_rows(open(os.path.join(directory, table), encoding="utf-8").read())
        check("表的内容行", rows == [(3, ["甲", "理由一"]), (5, ["乙", "理由二"])], f"取到 {rows}")

        text, problem = baseline_text("HEAD", table, cwd=directory)
        base_rows, rows_problem = baseline_rows("HEAD", table, cwd=directory)
        check("基准里有这张表", problem is None and rows_problem is None and text is not None and text.startswith("# 表头注释")
              and base_rows == [(3, ["甲", "理由一"]), (5, ["乙", "理由二"])], f"取到 {text!r}、{base_rows}、{problem}、{rows_problem}")

        sub_text, sub_problem = baseline_text("HEAD", table, cwd=os.path.join(directory, "sub"))
        dotted_text, dotted_problem = baseline_text("HEAD", f"./{table}", cwd=os.path.join(directory, "sub"))
        check("子目录里按当前目录取", sub_problem is None and sub_text == "子目录\t那一份\n" and dotted_problem is None and dotted_text == sub_text,
              f"取到 {sub_text!r}、{sub_problem}；写成 ./ 开头取到 {dotted_text!r}、{dotted_problem}")

        text, problem = baseline_text("HEAD", "not-there.tsv", cwd=directory)
        check("基准里没有这张表", text is None and problem is None, f"取到 {text!r}、{problem}")

        text, problem = baseline_text("no-such-base", table, cwd=directory)
        check("基准名取不到", text is None and problem is not None and problem.command == "ls-tree"
              and problem.sentence().startswith(f"取不到基准 no-such-base 里的 {table}（git ls-tree 退 "), f"取到 {text!r}、{problem}")

        def command_line(*arguments):
            completed = subprocess.run([sys.executable, os.path.realpath(__file__), "show", *arguments], capture_output=True, text=True, cwd=directory)
            return completed.returncode, completed.stdout, completed.stderr

        present, absent = command_line("HEAD", table), command_line("HEAD", "not-there.tsv")
        # 对象库缺了基准那一版的那一份（门禁样本 reading-discipline-red 的 setup.sh 同一种造法）：ls-tree 认得出名字，show 读不出
        blob = subprocess.run(["git", "-C", directory, "rev-parse", f"HEAD:{table}"], capture_output=True, text=True, check=True).stdout.strip()
        os.remove(os.path.join(directory, ".git", "objects", blob[:2], blob[2:]))
        text, problem = baseline_text("HEAD", table, cwd=directory)
        check("基准那一版的对象缺了", text is None and problem is not None and problem.command == "show"
              and problem.sentence(None) == f"读不出基准 HEAD 里的 {table}（git show 退 {problem.returncode}）", f"取到 {text!r}、{problem}")
        unreadable = command_line("HEAD", table)
        check("命令行三种结局", present[0] == 0 and present[1].startswith("# 表头注释") and absent[0] == EXIT_ABSENT and absent[1] == ""
              and unreadable[0] == 1 and "读不出基准 HEAD 里的" in unreadable[2],
              f"有表 {present[0]}、没有表 {absent[0]}、取不到 {unreadable[0]}（stderr {unreadable[2].strip()[-120:]!r}）")

        replay = os.path.join(directory, "replay.sh")
        with open(replay, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\nTABLE=$(cat <<'TSV'\nE1|e1_bin|--n 3|e1-a.out|exact\nE12|@driver_e12||sub/e12-b.out|timing\n"
                         "E3|e3_bin||e3.out|exact|多一列\nX4|x4||x4.out|exact\nE5|e5||e5.out|loose\nTSV\n)\nE6|@driver_e6||e6.out|exact\n")
        found = replay_rows(replay)
        check("复跑表的行", found == [ReplayRow("E1", "e1_bin", "--n 3", "e1-a.out", "exact"), ReplayRow("E12", "@driver_e12", "", "sub/e12-b.out", "timing"),
                                    ReplayRow("E6", "@driver_e6", "", "e6.out", "exact")] and replay_rows(os.path.join(directory, "absent.sh")) is None,
              f"取到 {found}")

        saved = os.environ.get(BREAK_VARIABLE)
        os.environ[BREAK_VARIABLE] = "no-such-break"
        try:
            breaks()
            refused = False
        except ValueError:
            refused = True
        finally:
            if saved is None:
                os.environ.pop(BREAK_VARIABLE, None)
            else:
                os.environ[BREAK_VARIABLE] = saved
        check("认不出的弄坏开关被拒", refused)

    path_move_selftest(check)
    moved_only_selftest(check)
    path_move_evidence_selftest(check)
    path_move_occupied_selftest(check)
    path_move_anchor_selftest(check)
    path_move_base_row_selftest(check)
    path_move_base_shape_selftest(check)
    path_move_window_selftest(check)
    path_move_reused_name_selftest(check)
    path_move_reordered_selftest(check)
    path_move_inserted_row_selftest(check)
    path_move_chain_selftest(check)
    moved_lines_position_selftest(check)
    governed_moves_selftest(check)
    renamed_files_selftest(check)
    failed = [(name, detail) for name, ok, detail in results if not ok]
    for name, ok, detail in results:
        if ok:
            print(f"  ✓ {name}")
        else:
            print(f"  ✗ 自证失败：{name}：{detail}")  # gate-lint:detail
    if failed:
        print(f"  ✗ lib-lag-baseline.py 自证：{len(results)} 项里 {len(failed)} 项没过")  # gate-lint:summary
        print("     → 怎么办：照上面点名的那一项修 baseline_text / content_rows / replay_rows；设了 LAG_BASELINE_BREAK 时这几项该红")
        return 1
    print(f"  ✓ lib-lag-baseline.py 自证：{len(results)} 项都过")
    return 0


def main(argv):
    if argv == ["--selftest"]:
        return selftest()
    if len(argv) == 3 and argv[0] == "show":
        return show_main(argv[1], argv[2])
    if len(argv) == 2 and argv[0] == "replay-rows":
        return replay_rows_main(argv[1])
    checking = argv[:1] == ["path-moves-check"] and (len(argv) in (1, 2) or (len(argv) == 4 and argv[2] == "--base"))
    if checking or (len(argv) == 2 and argv[0] in ("moved-only-filter", "moved-lines-filter")):
        try:
            if checking:
                return path_moves_check_main(argv[1] if len(argv) >= 2 else ".", argv[3] if len(argv) == 4 else None)
            return (moved_only_filter_main if argv[0] == "moved-only-filter" else moved_lines_filter_main)(argv[1])
        except ValueError as error:
            print(f"  ✗ {error}", file=sys.stderr)
            print(f"     → 怎么办：去掉 {BREAK_VARIABLE}，或只写认得的项", file=sys.stderr)
            return 2
    print("  ✗ 用法：lib-lag-baseline.py show <基准> <路径> | replay-rows <复跑表路径> | path-moves-check [根 [--base <基准>]] | moved-only-filter <基准> | moved-lines-filter <基准> | --selftest", file=sys.stderr)
    print("     → 怎么办：照文件头「命令行」一段写；python 里调用见文件头「函数」一段", file=sys.stderr)
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
