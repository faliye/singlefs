#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：heavy-thread-defaults、hardcoded-quota、memory-bounded-parallelism、shared-data-root-literals、cp-preserve-mode、peer-launch-direct、verdict-library-code。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# heavy-thread-defaults 格认合规写法的几段判法（HEAVY_THREAD_RECOGNITION_LIBRARY）在拆出去的这一份里
# shellcheck source=code-tooling-heavy-thread-recognition.sh
source "$STAGE_DIRECTORY/lib/code-tooling-heavy-thread-recognition.sh"

# 排除表只缩不涨要的基准那一版，heavy-thread-defaults、hardcoded-quota、shared-data-root-literals 三格共用。在被判的根里调，设两个变量：
#   base_name   改动范围的基准（scripts/gate/changed-paths.sh 的 gate_diff_base gate）；不在 git 仓里、没有 HEAD 时是空的；
#   base_state  table（取到了，写进第二个参数那份文件）、none（不在 git 仓里、没有 HEAD、基准里没有这张表）、unreadable:<为什么>（git 取不到）。
#   取法是 lib-lag-baseline.py show（照登记的搬迁换算路径；退 0 有表、退 3 基准里没有这张表、别的是 git 取不到，问题句在 stderr）。
#   source 不了 changed-paths.sh 时打出路、exit 1（格函数在自己的子 shell 里，结束的是那一格）。
exempt_table_baseline() { # <表的相对路径> <放基准那一版的文件>
  base_state="none"
  base_name=""
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1 && git rev-parse --verify -q HEAD >/dev/null 2>&1; then
    # shellcheck source=../../scripts/gate/changed-paths.sh
    if ! source "$STAGE_REPOSITORY/scripts/gate/changed-paths.sh"; then
      echo "  ✗ source 不了 $STAGE_REPOSITORY/scripts/gate/changed-paths.sh，取不到改动范围的基准，排除表只缩不涨没法比"
      echo "     → 怎么办：它随仓走（scripts/gate/changed-paths.sh），被删了就从 git 找回来。"
      exit 1
    fi
    # gate 基准一趟里不变：第一个调的格把它落进本趟共用的临时目录，别的格读那份（gate_diff_base 要起几次 git）
    base_cache="${CODE_TOOLING_RUN_SCRATCH:+$CODE_TOOLING_RUN_SCRATCH/gate-base}"
    if [[ -n "$base_cache" && -f "$base_cache" ]]; then
      base_name="$(<"$base_cache")"
    else
      base_name="$(gate_diff_base gate)"
      if [[ -n "$base_cache" && -n "$base_name" ]]; then
        printf '%s\n' "$base_name" >"$base_cache.$BASHPID" && mv -f -- "$base_cache.$BASHPID" "$base_cache" || rm -f -- "$base_cache.$BASHPID"
      fi
    fi
    if base_problem="$(python3 "$STAGE_DIRECTORY/lib-lag-baseline.py" show "$base_name" "$1" 2>&1 >"$2")"; then
      base_state="table"
    else
      base_exit=$?
      if ((base_exit == 3)); then base_state="none"; else base_state="unreadable:${base_problem:-lib-lag-baseline.py show 退 $base_exit}"; fi
    fi
  fi
}

# 排除表「只缩不涨」比此刻一行（路径:锚串）有没有在基准里的 python 库，heavy-thread-defaults、hardcoded-quota、repository-copy-closure 三格共用（别的格不用）。
#   装法照 CODE_WITHOUT_TEXT_LIBRARY：exempt_growth = types.ModuleType("lib_exempt_row_growth"); exec(compile(sys.argv[i], "exempt-row-growth-library", "exec"), exempt_growth.__dict__)
#   调法：exempt_growth.grown_exempt_rows(此刻的 [(行号, 路径, 锚串)], 放基准那一版的文件, 读一行的正则, scripts/gate/lib_lag_split_family.py 的路径, 是否不认拆分族
#   [, hit_count=<(路径, 锚串, 是不是基准那一侧) → 计数>])。
#   一份文件拆成几份，基准只经搬迁登记换路径（一个旧前缀只换一个新前缀）认不了：此刻一行的路径照拆分族归到基准里的入口（lib_lag_split_family.py 的 keyed_family_growth），
#   按（入口, 锚串）比计数之和，超了这一组的行都判涨；族外的路径、锚串对不上的照旧涨。给了 hit_count 的按它数（heavy-thread-defaults：锚串在那份文件里命中的被判行数，
#   基准那一侧数基准那一版的入口文件），没给的每一行算 1（hardcoded-quota、repository-copy-closure 比行数）。认族与比的写法只在那一份里；读不进它判红退 1。
EXEMPT_ROW_GROWTH_LIBRARY="$(cat <<'PY_EXEMPT_ROW_GROWTH'
import importlib.util, sys


def grown_exempt_rows(exempt_rows, base_table_file, row_pattern, split_family_file, family_unrecognized, hit_count=None):
    specification = importlib.util.spec_from_file_location("lib_lag_split_family", split_family_file)
    split_family = importlib.util.module_from_spec(specification)
    try:
        specification.loader.exec_module(split_family)
    except (OSError, ImportError, SyntaxError) as error:
        print(f"  ✗ 载不进 {split_family_file}：{error}")
        print("     → 怎么办：它随仓走（scripts/gate/lib_lag_split_family.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；拆分族的认法只在那一份里写。")
        sys.exit(1)
    base_keys = []
    for line in open(base_table_file, encoding="utf-8").read().split("\n"):
        found = row_pattern.fullmatch(line)
        if found and not line.lstrip().startswith("#"):
            base_keys.append((found.group("path"), found.group("anchor")))
    current_keys = [(path, anchor) for _number, path, anchor in exempt_rows]
    defects = frozenset({split_family.UNRECOGNIZED}) if family_unrecognized else frozenset()
    if hit_count is None:
        base_counts, current_counts = set(base_keys), current_keys
    else:
        base_counts = {key: hit_count(key[0], key[1], True) for key in dict.fromkeys(base_keys)}
        current_counts = {key: hit_count(key[0], key[1], False) for key in dict.fromkeys(current_keys)}
    grown_keys = set(split_family.keyed_family_growth(base_counts, current_counts, defects))
    return [row for row in exempt_rows if (row[1], row[2]) in grown_keys]
PY_EXEMPT_ROW_GROWTH
)"

# ── 格 heavy-thread-defaults：scripts/、.claude/gate.d/、.claude/hooks/、crates/ 里自己定并行度默认值的写法，都经每台默认重活线程数的登记位取
#
# 登记位：.claude/heavy-thread-default（写法在它的文件头），判它在不在、写没写对经 scripts/hosts/heavy_threads.py 的 default_heavy_threads
#   （用这一道所在仓里那一份，登记文件取被判的根里那一份，不认调用方的 SINGLEFS_THREAD_CAP）。
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/hooks/、crates/ 里的 *.sh、*.py、*.rs；
#   跳过 target/、__pycache__/ 与 .claude/gate.d/fixtures/（判别力样本是造出来的输入）。
# 只看代码：shell 去掉整行注释、行尾不在引号里的 # 注释与出路文字（echo / printf / print( 的字面量，CODE_WITHOUT_TEXT_LIBRARY 的 OUTPUT_TEXT）；
#   python 经 tokenize 去掉注释、文档串与字符串字面量，只留内容整个是 nproc、以 capped.sh 结尾、纯数字、--threads 一类选项名或线程变量名的字面量；
#   rust 去掉 // 之后的部分与字符串字面量（留下的同 python）。
# 算「自己定并行度默认值」的写法（每处记一个或几个种类，判法在 THREAD_DEFAULT_FORMS 与下面几行）：
#   nproc、cpu_count(、sched_getaffinity、available_parallelism、available_threads（此刻空闲线程，按忙闲现算）、min(16 / .min(16)；
#   核数的一半：/ 2 或 // 2，同一行里有核数、线程、并行一类的词；capped.sh <数字>：capped.sh 不在引号里、或它所在的那个字面量就在 capped.sh 收尾
#   （python 的列表写法 "capped.sh", "16" 也算），引号里整句拿来当样本的不算；
#   写死的线程数默认：${<线程变量>:-<数>}、--threads / --jobs / --workers / --lanes 的 default=<数>、名字里同时有 DEFAULT / WHEN_UNSET / BY_PROCESSOR
#   与 THREADS / JOBS / WORKERS 的常量赋了数、"threads" / "jobs" / "workers" 键赋了数，以及 unwrap_or(<正整数>) / Err(_) => <正整数> /
#   or <正整数>：同一行或往上 5 行代码里点了线程变量名（大写、含 THREAD / JOBS / WORKERS 的那种字面量）。
#   回退值只认线程 / 工作进程 / 并行度的上下文：线程变量名在往上 5 行里、而回退值所在那一行本身写着 timeout / deadline / seconds / duration / 超时 一类的词
#   （原行里找，字符串和键名里的也算，注释不算）的，那个数是超时秒数一类的别的量，不判（例：`"timeout": int(os.environ.get("X_TIMEOUT_SECONDS") or 600)`）；
#   线程变量名就在回退值同一行的照判。
# 排除表 .claude/heavy-thread-exempt：一行「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）；那份文件里含锚串的被判行不算。
#   写不对、指向不存在的文件、锚串一处被判行都不命中（那一处已经改掉了，留着就是白放行）都判红；只缩不涨：与改动范围的基准
#   （scripts/gate/changed-paths.sh 的 gate_diff_base gate）里那一版比，基准里没有的「路径:锚串」判红；基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明；git 取不到基准那一版判红（取法 lib-lag-baseline.py 的 show）。
#   一份文件拆成几份的：此刻一行的路径照拆分族（scripts/gate/lib_lag_split_family.py，认法写在它文件头）归到基准里的入口，按放行的被判行数之和比（EXEMPT_ROW_GROWTH_LIBRARY，下面「排除表只缩不涨按放行的被判行数比」）；族外的路径、锚串对不上的照旧判红。
# 装置读自己的线程变量（G3）：python / rust 代码里点了名的线程变量字面量（含 THREAD / JOBS / WORKERS 的大写名字）出现就算读，不看它在不在读环境变量的写法里
#   （经包装函数读、`ENV = os.environ` 的别名与拷贝、`std::env::vars()`、`use std::env::var as …` 这些写法里名字只是个字面量，认不出读的是哪一个）；只有 ["X"] = 值 这种写给子进程的不算读。
#   `if "X_THREADS" in 某个表` 这种字符串比较、没人用的常量也因此被点名：确实不是读环境变量的，登记进排除表 .claude/heavy-thread-exempt（锚串取那一行里的一段原文，理由写明）。
#   shell 里 $变量 / ${变量}（文件自己赋了值的局部变量不算，行首或 ; && || 之后同一行接着赋的也认；有一处 X="${X:-4}" 这种右边读着自己的，仍算读；
#     赋值在同一拆分族的别的文件里的也不算：入口赋、拆出的库读的变量，拆分之前在同一份文件里，族的认法在 scripts/gate/lib_lag_split_family.py（family_owner 找入口、family_members 取族），族外的文件里赋的照算）；
#   shell 里 `|| X=<数>`、`&& X=<数>`（X 是线程变量名；数是正整数，或 `$变量`、`$((算术))`，即默认值先放进变量、先算好再赋；`[[ -z "$X" ]] && X=8` 这类没设就给默认值的写法）
#   认成「写死的线程数默认（|| / && 赋值）」，点名那一行；`$(命令)` 与 `$SINGLEFS_THREAD_CAP`（取自上限）不算钉数；
#   `&&` 左边紧挨着是弄坏开关的判断（`break_is <项>`、`<前缀>_break_is <项>`、点到 `$X_BREAK` / `${X_BREAK…}` 的测试，到 `&&` 之间不夹 ; | &）的不算：
#   那一支只在自证打开那条开关时走到，取线程是开关要弄坏的那一处，不是没设就给的默认值（pinned_conditional_assignment 的 break_guarded）；
#   `$变量` 那个变量算「登记位取的」、不算钉数，要三样同时满足（registry_sourced_variables）：
#     ① 同一份文件里它的每一处「名=值」，右边整条是登记位读法："$(heavy_thread_default)"（可带 2>/dev/null）、"$(default_heavy_threads)"、"${SINGLEFS_THREAD_CAP}"
#        （引号、花括号可省；右边夹了别的东西的，如除以一个变量、${SINGLEFS_THREAD_CAP:-$另一个变量}、$(( … ))，都不算）；
#     ② 同一份文件里它没出现在 ${名:-…}、${名:=…}（不带冒号的同算）里，也没有别的赋值写法：(( 名 = … ))（含 +=、++ 这类）、let、declare -i、read、printf -v、
#        for 名 in、名+=、eval 后面同一行点了它的名字（other_assignment_forms）；
#     ③ 文件没有 source（或 .）scripts/hosts/heavy-threads.sh 之外的别的文件。
#   有一处不满足就照算钉数。判不了的：变量在别的文件里怎么赋（③ 一概不算）、mapfile / readarray / getopts 这类读进变量的写法、eval 里拼出来的名字。
#   shell 里 `printenv NAME`（NAME 是线程变量名）从环境里读它，认成读，文件里赋没赋过值都一样。
#   rust 行首 #[cfg(test)] 之后第一个非空行（夹着的 #[…] 属性不算）是 mod 的，那个测试模块往后不算，只罩一条 use、fn、const 的不跳过后面；
#   `#[cfg(test)] mod tests;`（外置模块，分号结尾）只是声明，它后面的代码照扫；它引进来的 tests.rs 整份算测试代码，整份不扫：
#   crates/<包>/src/ 下的 .rs 整份算测试的有三种（认法只在 scripts/gate/lib_rust_test_module_files.py 文件头：父文件里引到它的每一处 mod 声明带 #[cfg(test)]、
#   声明所在的祖先文件整份算测试、文件自己带 #![cfg(test)]），这一格不读它们（不进「扫了 N 份」的数），成功句另报「整份算测试代码的 N 份 rust 文件不扫」。
#   这个名字不在 scripts/hosts/capped.sh 的 variable_names 清单里（被判的根里那一份，读不出判红），
#   读它的任何一处所在的那一条语句里也没出现 SINGLEFS_THREAD_CAP、heavy_thread_default、default_heavy_threads、heavy-threads.sh、heavy_threads、capped.sh 之一（注释与文档串里写的不算；
#   同文件别的语句里提到不算回退），capped.sh <上限> 到不了它、没设时也不回退到上限或登记值：点名的是读它的那一行（每个变量第一次被读的那一行，同一行的几个变量写在一起；
#   那一行本来就是别的种类的被判行，就并到那一行上，不是的单列一条）。排除表按被判行匹配，不按文件并组：锚串要落在读它的那一行原文里才放那一行，
#   同文件别的被判行上的锚串（核数、available_parallelism 那几行）放行不了它。
#   一条语句的算法在 statement_range：python 取 tokenize 的逻辑行，shell 取反斜杠续行，rust 往前到上一条语句收尾（; { }）之后、往后到括号配平且以 ; 或 } 收尾（或闭合外层的块）；
#   回退写在另一条语句里（先读进变量、下一句再回退）认不出，要么写进同一条表达式，要么把变量名加进 capped.sh 的 variable_names。
#   crates/ 里的（crash_amplification.rs、两份 GPU 对拍测试）与 scripts/mutation/ 的（与依赖产物共用）这几份落在原有的「代码批」「与依赖产物共用」行上，不另加行；
#   E161 装置只读 E161_THREADS，capped.sh 清单里有它，这一格不点名它，它没设时取 available_parallelism 的那一行照旧由排除表「代码批」行放行。
# 认得出、不点名的四种（改判法，不涨表）：
#   自证与单测的样本值：python 里名字带 selftest / self_test 的函数、基类是 TestCase 的类（ast 读，python_sample_scope_lines），里面字典键给的数（"<键>": <数>，键是 threads / jobs / workers）
#     与列表写法 capped.sh 后面跟的数不算；同一行别的写法照判；
#   只记进报告的核数：一行里取核数的每一次调用都是字典里核数那一键（cpus、cores、processors、cpu_count、core_count、logical_processors）的值（core_count_only_reported）；
#   名字片段：引号里只有线程字样本身（THREADS、THREAD、JOBS、WORKERS、THREAD_CAP，前后可带一个下划线）的不是变量名（拿来比后缀、查子串）；shell 的 $THREADS 照读算；
#   路径变量：名字以 _FILE、_PATH、_DIRECTORY、_DIR 收尾的，值是路径，不算线程变量。
#   跑前登记定了线程与线道的那几处不归这四种（e171 两份分段单元脚本读 $THREADS、e201 的 --lanes 默认 1），照判。
# 认得出的合规写法（判法在 lib/code-tooling-heavy-thread-recognition.sh 的 HEAVY_THREAD_RECOGNITION_LIBRARY，那一份文件头写了判不了的几种）：
#   shell 自证函数里的样本输入：顶格写的、名字里带 selftest / self_test 的 shell 函数（shell_selftest_lines）里，「名=<数>」这种喂给样本的环境词
#     不算写死的线程数默认（同一行别的写法照判），单引号括着的 $变量 是写给样本的正文、不算读（stage-cells.sh 自证里的 SAMPLE_DEFAULT_JOBS 那几行）；
#   经登记位包装函数取：shell 里读线程变量的那条语句调了 $(<包装函数>)（registry_wrapper_functions：函数里调 heavy_thread_default，往标准输出写的别的命令
#     只在条件里有弄坏开关的 if 块里、或写到标准错误；例 stage-cells.sh 的 stage_cells_default_jobs，五道阶段 STAGE_CELLS_JOBS="${STAGE_CELLS_JOBS:-$(stage_cells_default_jobs)}"），
#     算回退到登记值；
#   rust 从登记位取好的线程数再分：「核数的一半」那一种，每一处 `/ 2` 左边是 default_heavy_threads()、或追得到它的名字（registered_split：let 绑定右边整条是它，
#     或外面那个函数的参数、被扫的 rust 文件里每一处调那个函数都在那个位置上交它；例 crash_library/inner_threads_of_the_test_process.rs 的
#     shares_of_the_test_process），不点名；同一行别的写法照判。
#   认不准的照旧判红，不靠登表。
# 排除表只缩不涨按放行的被判行数比：一行「路径:锚串」放行那份文件里含锚串的被判行，比的是这几行命中的被判行数；基准那一侧取基准那一版的那份文件
#   （lib-lag-baseline.py 的 baseline_file_bytes，照生效的搬迁登记倒着找），照这一格此刻的判法（scan_text）数；此刻一行的路径照拆分族归到基准里的入口，
#   按（入口, 锚串）比命中数之和，超了这一组的行都判涨（lib_lag_split_family.py 的 keyed_family_growth）：拆分把同一处写法分到两份文件、各登一行，
#   命中之和不超过基准那一份就不涨；没拆的文件里同一锚串命中的被判行比基准多也判涨。基准那一版的文件 git 取不到判红。
# 判不了的：变量里拼出来的命令、读别的文件定的数、跨 5 行以上的回退值、shell 里嵌的 python 字符串（按 shell 判：引号里整句的样本不算，其余照判）、
#   多行的 rust 字符串字面量、跨语句的回退（见上）、shell 的 if / case 多行复合语句（按行判，回退写在另一行里认不出）。排除表的理由成不成立靠 review。
#   线程数取对了之后用在几处、装置的主循环对取样点串不串行，这一格也判不了（只用在一处的装置也可能就是对的：它唯一的重活就在那一处；
#   串行的循环里调的函数自己可能在并行）：由运行时的在用核数兜：每次尝试的平均在用核数由 task-server 记进 task status（只记不报）。
# 样本：heavy-thread-defaults-red 的登记写对，scripts/hosts/half.sh 用 $(nproc) / 2、scripts/gate/half.py 用 cpu_count 与 min(16 与 processor_count // 2、
#   .claude/hooks/start.py 用列表写法起 capped.sh 16、.claude/gate.d/stage.sh 用 capped.sh 16、crates/demo/src/lib.rs 用 available_parallelism 与
#   隔了几行的 unwrap_or(8)，逐处点名判红（只在注释、echo 与 print 里提到的不点名）；排除表一行指向不存在的文件、一行锚串不命中、
#   setup.sh 建 git 仓提交一版只有那两行的表、再加一行（表涨了），三样都点名。heavy-thread-defaults-green 的登记写对，脚本经 heavy_thread_default
#   与 default_heavy_threads 取、只在注释、文档串、echo、引号里整句的样本里提到 nproc 与 capped.sh 16，一处真用 $(nproc) 的登记在排除表里，
#   判绿并报扫了几份、排除命中几处；scripts/hosts/uses-registry.py 里一行 int(… or 600) 的超时回退值紧跟在
#   SINGLEFS_THREAD_CAP 那一行后面，不点名。heavy-thread-defaults-red 里 scripts/gate/half.py 还有线程变量隔一行才给回退值的 int(cap or 8)，照判（超时那条豁免不吞它）。
#   装置读自己的线程变量那一路，两份样本根里各放一份假的 scripts/hosts/capped.sh（只有 variable_names 清单）：red 的清单没有装置的变量，scripts/experiment-apparatus/own_device.py（环境里读、
#   没设就 KeyError）、scripts/hosts/own_device.sh（${…:?}）、crates/demo/src/device.rs（env::var 加 expect）三份没有别的被判行，落在点名变量的那一行；scripts/experiment-apparatus/own_with_cores.py 与
#   crates/demo/src/lib.rs 已有核数、回退值的被判行，同一行再加这一种；own_device.sh 里 LOCAL_WORKERS 是文件自己赋的局部变量，不点名。green 的清单有 E161_THREADS：
#   crates/demo/src/device.rs 读它，不点名，测试模块里的 SAMPLE_TEST_THREADS 不算；scripts/experiment-apparatus/own_registry.py 读 capped.sh 不设的 SAMPLE_OWN_THREADS 但没设时回退到 default_heavy_threads，不点名；
#   scripts/experiment-apparatus/own_multiline.py 与 crates/demo/src/own_match.rs 把读和回退写在同一条跨行的语句里（python 的括号续行、rust 的 match 加 Err 分支），不点名；
#   scripts/experiment-apparatus/own_child_env.py 只往子进程环境里写 ["SAMPLE_CHILD_THREADS"] = 值，scripts/hosts/local_workers.sh 的 WORKERS 与 SAMPLE_INLINE_WORKERS（同一行 ; 之后赋的）是局部变量，都不算读；
#   scripts/experiment-apparatus/own_in_comparison.py 里 `if "SAMPLE_COMPARED_THREADS" in registry_table:` 是字符串比较，不是读环境变量，但引号里的名字出现就算读，被判行靠排除表放行
#   （W01：scripts/agents/dispatch-prompt.py 的 `in` 比较）；scripts/hosts/env_prefix.sh 里 `true && SAMPLE_PREFIX_THREADS=8 make -j 2` 的 =8 后面跟着命令，是环境前缀，不点名。
#   heavy-thread-defaults-quoted-name-red：线程变量名出现在引号里就点名，不管读环境变量的写法认不认得出：scripts/experiment-apparatus/wrap_helper.py（包装函数读）、alias_env.py（`ENV = os.environ` 别名）、
#   copied_env.py（os.environ.copy() 拷贝）、in_comparison.py（`in` 比较与没人用的常量）、crates/demo/src/project_wrapper.rs、harness_wrapper.rs（包装函数读）、use_alias.rs（`use std::env::var as …`）、
#   vars_iter.rs（`std::env::vars()`）、lib.rs（#[cfg(test)] mod tests; 之后的读），以及 shell 的 if_default.sh、or_default.sh（`|| X=8`）、and_default.sh（`&& X=8`），共 14 处（in_comparison.py 的比较和没人用的常量在两行上，各一处），baseline_read.py 是对照。
#   heavy-thread-defaults-per-line-red：排除表按被判行匹配，两张行只放行各自锚串落在的那一行。crates/demo/src/absorbed.rs 第 1 行 available_parallelism 与 scripts/experiment-apparatus/absorbed.py 第 2 行 os.cpu_count()
#   由排除表放行，同文件新读线程变量的那一行（absorbed.rs:3、absorbed.py:3）借不到它们，各被点名；scripts/hosts/or_var_default.sh（`|| X=$fallback`）、or_arith_default.sh（`|| X=$((4 * 2))`）认成钉数，
#   printenv_read.sh 的 `printenv SAMPLE_PRINTENV_THREADS` 认成读，共 5 处。
#   heavy-thread-defaults-per-line-green：同一批文件，每个新读的那一行都有自己的锚串，判绿并报表 7 行每行都命中；or_registry.sh 里 `|| X=$(heavy_thread_default)`、`|| X=$SINGLEFS_THREAD_CAP` 取自登记位与上限，不点名。
#   heavy-thread-defaults-var-source-green：`|| X=$变量` 的变量在同一份文件里由登记位读法赋值：var_from_registry.sh（default_threads="$(heavy_thread_default)"）、
#   var_after_if.sh（if ! threads="$(heavy_thread_default)"; then …，`|| X=${threads}`）、var_from_cap.sh（cap_jobs="${SINGLEFS_THREAD_CAP}"）、
#   var_stderr_silenced.sh（default_threads="$(heavy_thread_default 2>/dev/null)"，照 run-with-memory-cap.sh 丢掉 stderr），四份都不点名，判绿。
#   heavy-thread-defaults-var-source-forms-red：变量有一处「名=值」从登记位取，但不满足 registry_sourced_variables 三样之一的十二份，`|| X=$default_threads` 那一行各点名一处（共 12 处）：
#   右边不整条是登记位读法的 cap_fallback_via_variable.sh（${SINGLEFS_THREAD_CAP:-$fallback_jobs}）、registry_divided_by_variable.sh（$(( $(heavy_thread_default) / divisor ))）；
#   ${名:=8} 补默认的 colon_equals_fallback.sh；source 了另一份给字面数的 sample_defaults.sh 的 sourced_override.sh；
#   别的赋值写法八份：arith_assignment.sh（(( 名 = 8 ))）、let_assignment.sh（let "名 = 8"）、declare_integer.sh（declare -i 名）、read_assignment.sh（read -r 名 <<< 8）、
#   printf_v_assignment.sh（printf -v 名 %s 8）、for_assignment.sh（for 名 in 8）、plus_equals_assignment.sh（名+=0）、eval_assignment.sh（eval "名=8"）。
#   heavy-thread-defaults-var-source-red：var_from_literal.sh（变量赋了 8）、var_mixed.sh（一处从登记位取、另一处 `&& 变量=8`）、
#   arith_default.sh（`|| X=$((default_threads * 2))`，算术照算钉数，变量从登记位取也一样）、literal_default.sh（`|| X=8`），共 4 处点名。
#   heavy-thread-defaults-wide-cost-red：宽判法的代价，八份不读环境变量的写法仍点名（字符串比较、名字元组、字典实参、没人用的常量、argparse 的 metavar、rust 的比较与 match、`&&` 夹下限），共 8 处；
#   fp_docstring.py（文档串提到）与 fp_prefix.sh（`&& X=8 make` 的环境前缀）是对照，不点名；真树上这类逐行判，不读环境的进排除表并写理由。
#   heavy-thread-defaults-cfg-test-red：crates/demo/src/device_after_cfg_test.rs 第 1 行 #[cfg(test)] 只罩一条 use，第 5 行读 SAMPLE_WORKER_THREADS 没设就 panic，必须点名（共 1 处）；
#   文件末尾另有 #[cfg(test)]、#[allow(dead_code)]、mod tests 的测试模块，里面点名的 SAMPLE_TEST_ONLY_THREADS 不算。
#   heavy-thread-defaults-cfg-test-module-file-red：crates/demo/src/lib.rs 里 `#[cfg(test)] mod tests;` 引进来的 tests.rs 整份算测试代码，里面读 SAMPLE_TEST_MODULE_FILE_THREADS 不点名；
#   没带 #[cfg(test)] 的 `mod worker;` 引进来的 worker.rs 第 2 行读 SAMPLE_WORKER_THREADS 没设就 panic，必须点名（共 1 处）。
#   heavy-thread-defaults-capped-elsewhere-red：scripts/experiment-apparatus/mentions_capped.py 读 SAMPLE_OWN_JOBS 没设就 KeyError，同文件另一条语句里有字符串 "scripts/hosts/capped.sh"；
#   scripts/hosts/mentions_capped.sh 读 SAMPLE_SHELL_WORKERS（${…:?}），下一行 bash scripts/hosts/capped.sh default；两份都必须点名（共 2 处）。
#   heavy-thread-defaults-moved-green：setup.sh 提交 scripts/old-rig/report_cores.sh（$(nproc) 只打核数）与登记它的排除表，再 git mv 到 scripts/new-rig、表里那一行改成新路径，
#   .claude/gate.d/path-moves.tsv 登记 scripts/old-rig → scripts/new-rig：基准那一版的表经 lib-lag-baseline.py show 照登记换成新前缀，判绿并写「那一版比没涨」；
#   heavy-thread-defaults-moved-red：同一个搬迁，不放搬迁登记表，表里换了路径的那一行是基准里没有的，判红点名「排除表涨了」。
#   heavy-thread-defaults-exempt-family-green：setup.sh 提交 scripts/new-rig/report_cores.sh（$(nproc) 只打核数）与登记它的排除表，工作区里 $(nproc) 那一行挪进同目录的
#   report_cores-count.sh（钩子拆法的部分文件）、表里那一行改成新路径、锚串不变：此刻一行归到基准里的 report_cores.sh、锚串相同，判绿并写「那一版比没涨」；
#   heavy-thread-defaults-exempt-family-red：同一个拆分，挪进的文件叫 other-cores.sh（不是 report_cores.sh 的部分文件，族外），表里那一行是基准里没有的，判红点名「排除表涨了」。
#   heavy-thread-defaults-sample-inputs-green：scripts/hosts/sample_inputs.py 里五样不是重活默认的写法都不点名，判绿、报装置自己读的线程变量 0 个：
#   self_test_cells 里列表写法的 capped.sh 后面跟数、selftest 与继承 TestCase 的类里字典键给数（自证与单测的样本值）、字典里核数那一键的值是取核数那一次调用（只记进报告）、
#   endswith 的参数是引号里只有线程字样本身的名字片段、名字以 _FILE 收尾的常量（路径变量）。
#   heavy-thread-defaults-sample-inputs-red：同样的写法挪到不该放过的地方，五处都点名（共 5 处）：near_capped.py 在普通函数里经 capped.sh 跟数起命令、near_dict.py 在普通函数里字典键给数、
#   near_core.py 把取核数那一次调用放在 workers 键下、near_suffix.py 拿完整的线程变量名比后缀、near_path.py 读的名字里 FILE 不在收尾。
#   heavy-thread-defaults-split-locals-green：scripts/hosts/split_run.sh 里 SAMPLE_SPLIT_WORKERS="${1:?…}" 赋值，钩子拆法的部分文件 split_run-body.sh 读 "$SAMPLE_SPLIT_WORKERS"：
#   变量在同一族的入口里赋了值，不点名，判绿并报装置自己读的线程变量 0 个；
#   heavy-thread-defaults-split-locals-red：入口里没赋值的族（orphan_run.sh 与部分文件 orphan_run-body.sh 读 $SAMPLE_ORPHAN_WORKERS）、赋值在族外文件里的读
#   （stranger.sh 赋 SAMPLE_STRANGER_WORKERS，stranger_report.sh 不是它的部分文件、读它）各点名一处（共 2 处）。
# 弄坏开关（CODE_TOOLING_BREAK）：thread-default-strings-scanned（python 与 rust 的字面量与文档串照代码扫）：heavy-thread-defaults-green 判红；
#   thread-default-half-unchecked（不认核数的一半）：heavy-thread-defaults-red 里 scripts/gate/half.py 第 4 行那条 want 找不到；
#   thread-default-exempt-unchecked（排除表不核文件在不在、锚串命不命中）：heavy-thread-defaults-red 里「指向不存在的文件」「锚串一处都不命中」两条 want 找不到；
#   thread-default-exempt-growth-unchecked（不比基准）：heavy-thread-defaults-red 里「排除表涨了」那条 want 找不到；heavy-thread-defaults-moved-red 期望退出 1、实测 0；
#     heavy-thread-defaults-moved-green 里「那一版比没涨」那条 want 找不到；
#   thread-default-exempt-family-unrecognized（排除表比基准时不认拆分族：拆出来的新路径都当族外）：heavy-thread-defaults-exempt-family-green 期望退出 0、实测 1（「排除表涨了」）；
#   thread-default-timeout-unchecked（超时回退值也按线程数判）：heavy-thread-defaults-green 判红（uses-registry.py 的超时那一行被点名）；
#   thread-default-own-variable-unchecked（不判装置读自己的线程变量）：heavy-thread-defaults-red 里「13 处」那一条与五条「读自己的线程变量」want 找不到，heavy-thread-defaults-sample-inputs-red 里「5 处」那一条找不到，
#     heavy-thread-defaults-cfg-test-module-file-red 退 0（期望退出 1：worker.rs 的读不判），heavy-thread-defaults-split-locals-red 退 0（期望退出 1：两处读都不判）；
#   thread-default-capped-list-ignored（capped.sh 的清单读了不认，每个线程变量都算清单外）：heavy-thread-defaults-green 判红（crates/demo/src/device.rs 里 E161_THREADS 被点名）；
#   thread-default-shell-local-unchecked（shell 里文件自己赋了值的变量也算读环境）：heavy-thread-defaults-green 判红（local_workers.sh 的 WORKERS 被点名），heavy-thread-defaults-red 里 scripts/hosts/own_device.sh 那条 want 找不到（点名的变量名多出 LOCAL_WORKERS），heavy-thread-defaults-var-source-green 判红（`|| X=` 赋的 SAMPLE_*_JOBS 被当成读），
#     heavy-thread-defaults-split-locals-green 判红（split_run-body.sh 读的 SAMPLE_SPLIT_WORKERS 被点名）；
#   thread-default-test-module-unskipped（rust 测试模块里点名的变量也算）：heavy-thread-defaults-green 判红（device.rs 测试模块里 #[test] 函数用 std::env::var("SAMPLE_TEST_THREADS") 读的那个变量被点名），
#     heavy-thread-defaults-cfg-test-red 里 device_after_cfg_test.rs:5 那条 want 找不到（测试模块里 std::env::var("SAMPLE_TEST_ONLY_THREADS") 读的变量也被点名，变量名列表成了 SAMPLE_TEST_ONLY_THREADS、SAMPLE_WORKER_THREADS；同一文件的被判行并成一处，「✗ 1 处」那条 want 仍找得到）；
#     测试模块里只把变量名赋给常量、同文件别处没用到的，不算读，样本要用读环境变量的写法；
#   thread-default-cfg-test-without-mod（行首 #[cfg(test)] 不看后面是不是 mod，一律往后都不算）：heavy-thread-defaults-cfg-test-red 退 0，退出码不对；
#   thread-default-fallback-by-file（整份文件的代码里出现回退字样就算回退了）：heavy-thread-defaults-capped-elsewhere-red 退 0，退出码不对；
#   thread-default-statement-single-line（读它的那条语句只算那一行）：heavy-thread-defaults-green 判红（own_multiline.py、own_match.rs 被点名）；
#   thread-default-subscript-write-counted（["X"] = 值 也算读）：heavy-thread-defaults-green 判红（own_child_env.py 被点名）；
#   thread-default-shell-inline-assignment-unseen（shell 里只认行首的赋值）：heavy-thread-defaults-green 判红（local_workers.sh 的 SAMPLE_INLINE_WORKERS 被点名），heavy-thread-defaults-var-source-green 判红（`|| X=` 赋的 SAMPLE_*_JOBS 被当成读）；
#   thread-default-quoted-name-needs-environment-read（引号里的名字又回到只在读环境变量的写法里才算读，ENVIRONMENT_READ）：heavy-thread-defaults-quoted-name-red 里 wrap_helper.py、alias_env.py、copied_env.py、
#     in_comparison.py、project_wrapper.rs、harness_wrapper.rs、use_alias.rs、vars_iter.rs 那八条 want 与「14 处」找不到；heavy-thread-defaults-green 判红（own_in_comparison.py 排除表那一行一处被判行都不命中）；
#     heavy-thread-defaults-sample-inputs-red 里「5 处」那一条与 near_suffix.py 那一条找不到；
#   thread-default-conditional-assignment-unpinned（不认 `|| X=<数>`、`&& X=<数>`）：heavy-thread-defaults-quoted-name-red 里 or_default.sh、and_default.sh 那两条 want 与「14 处」找不到，
#     heavy-thread-defaults-per-line-red 里 or_var_default.sh、or_arith_default.sh 那两条 want 与「5 处」找不到，heavy-thread-defaults-wide-cost-red 里 fp_clamp.sh 那条 want 与「8 处」找不到，
#     heavy-thread-defaults-var-source-red、heavy-thread-defaults-var-source-forms-red 退 0（期望退出 1）；
#   thread-default-external-mod-skipped（`#[cfg(test)] mod tests;` 也当测试模块，往后都不算）：heavy-thread-defaults-quoted-name-red 里 lib.rs:8 那条 want 与「14 处」找不到；
#   thread-default-test-module-file-scanned（整份算测试的 rust 文件也照产品代码扫）：heavy-thread-defaults-cfg-test-module-file-red 里「1 处」那一条找不到（tests.rs 也被点名，成了 2 处），
#     heavy-thread-defaults-quoted-name-red 里「14 处」那一条找不到（tests.rs 的读多出一处，成了 15 处）；
#   thread-default-exempt-by-file（排除表退回按文件并组：新读线程变量这一种挂到文件里已有的每一条被判行上，读它的那一行自己不被点名）：heavy-thread-defaults-per-line-red 里 absorbed.rs:3、absorbed.py:3 那两条 want 与「5 处」找不到，
#     heavy-thread-defaults-per-line-green 判红（锚串落在读它那一行上的排除表行，被判行里一处都不命中），heavy-thread-defaults-red 里 lib.rs:7 那条 want 与「13 处」找不到，
#     heavy-thread-defaults-quoted-name-red 里 in_comparison.py 两条 want 与「14 处」找不到；
#   thread-default-conditional-assignment-literal-only（`|| X=<数>` 只认字面正整数，不认 `$变量`、`$((算术))`）：heavy-thread-defaults-per-line-red 里 or_var_default.sh、or_arith_default.sh 那两条 want 与「5 处」找不到，
#     heavy-thread-defaults-per-line-green 判红（or_var_default.sh、or_arith_default.sh 两行的排除表行一处被判行都不命中），
#     heavy-thread-defaults-var-source-red 里 var_from_literal.sh、var_mixed.sh、arith_default.sh 那三条 want 与「4 处」找不到，heavy-thread-defaults-var-source-forms-red 退 0（期望退出 1）；
#   thread-default-printenv-unseen（不认 shell 里的 `printenv NAME`）：heavy-thread-defaults-per-line-red 里 printenv_read.sh 那条 want 与「5 处」找不到，
#     heavy-thread-defaults-per-line-green 判红（printenv_read.sh 的排除表行一处被判行都不命中）；
#   thread-default-conditional-variable-source-ignored（`|| X=$变量` 不看变量在文件里怎么赋的值，一律算钉数）：heavy-thread-defaults-var-source-green 判红（四份都被点名）；
#   thread-default-break-guarded-counted（弄坏开关护着的 `break_is <项> && X=$变量` 也算钉数）：heavy-thread-defaults-green 判红（scripts/hosts/env_prefix.sh 第 6 行被点名）；
#   thread-default-registry-substring-accepted（registry_sourced_variables 的 ① 下右边含登记位读法、去掉之后没有别的数字就算）：heavy-thread-defaults-var-source-forms-red 里
#     cap_fallback_via_variable.sh、registry_divided_by_variable.sh 那两条 want 与「12 处」找不到，heavy-thread-defaults-var-source-green 判红（var_stderr_silenced.sh 的 2>/dev/null 被当成别的数字）；
#   thread-default-defaulted-variable-accepted（不看 ${名:-…}、${名:=…}）：heavy-thread-defaults-var-source-forms-red 里 colon_equals_fallback.sh 那条 want 与「12 处」找不到；
#   thread-default-other-assignment-forms-unseen（不看别的赋值写法，只认「名=值」）：heavy-thread-defaults-var-source-forms-red 里 arith_assignment.sh、let_assignment.sh、declare_integer.sh、
#     read_assignment.sh、printf_v_assignment.sh、for_assignment.sh、plus_equals_assignment.sh、eval_assignment.sh 那八条 want 与「12 处」找不到；
#   thread-default-sourcing-ignored（不看文件 source 了别的文件）：heavy-thread-defaults-var-source-forms-red 里 sourced_override.sh 那条 want 与「12 处」找不到；
#   thread-default-selftest-samples-counted（python 自证函数与单测类里字典键给的数、capped.sh 后面的数也算）：heavy-thread-defaults-sample-inputs-green 判红（sample_inputs.py 第 6、11、17 行被点名）；
#   thread-default-core-report-counted（只记进报告的核数也算）：heavy-thread-defaults-sample-inputs-green 判红（sample_inputs.py 第 21 行被点名）；
#   thread-default-name-fragment-counted（引号里的名字片段也算读）：heavy-thread-defaults-sample-inputs-green 判红（sample_inputs.py 第 25 行的 THREADS 被点名）；
#   thread-default-path-variable-counted（路径变量也算线程变量）：heavy-thread-defaults-sample-inputs-green 判红（sample_inputs.py 第 28 行的 SAMPLE_PLAN_WORKERS_FILE 被点名）；
#   thread-default-shell-local-by-file（shell 局部变量只认读它的那一份文件里的赋值，不认同族别的文件里的）：heavy-thread-defaults-split-locals-green 判红（split_run-body.sh 读的 SAMPLE_SPLIT_WORKERS 被点名）；
#   thread-default-shell-selftest-samples-counted（shell 自证函数里的样本环境词与单引号里的 $变量 也算）：heavy-thread-defaults-shell-selftest-samples-green 判红（stage_samples.sh 的两行被点名）；
#   thread-default-registry-wrapper-unrecognized（不认登记位包装函数）：heavy-thread-defaults-registry-wrapper-green 判红（stage_entry.sh 那一行被点名）；
#   thread-default-registered-split-unrecognized（不认从登记位取好的线程数再分）：heavy-thread-defaults-registered-split-green 判红（crates/demo/src/shares.rs 两行被点名）；
#   thread-default-exempt-growth-rows-counted（排除表比基准照旧数行，不数放行的被判行）：heavy-thread-defaults-exempt-hits-split-green 判红（拆出的两行各登一行，行数比基准多），
#     heavy-thread-defaults-exempt-hits-grown-red 退 0（期望退出 1：没拆的文件里命中的被判行比基准多一处）。
# 认合规写法的样本：heavy-thread-defaults-shell-selftest-samples-green：scripts/hosts/stage_samples.sh 的 stage_samples_selftest 里一行 `SAMPLE_DEFAULT_JOBS=1` 喂样本、
#   一行单引号里写 ${SAMPLE_DEFAULT_JOBS:-}，不点名，判绿；heavy-thread-defaults-shell-selftest-samples-red：同样两行写在不叫 selftest 的函数里，各点名一处（共 2 处）。
#   heavy-thread-defaults-registry-wrapper-green：scripts/hosts/stage_entry.sh 的 SAMPLE_STAGE_JOBS="${SAMPLE_STAGE_JOBS:-$(sample_default_jobs)}"，sample_default_jobs 在
#   scripts/hosts/stage_library.sh 里 source heavy-threads.sh、调 heavy_thread_default，取不出的报错写到标准错误、弄坏开关护着的那一支打 1，不点名，判绿；
#   heavy-thread-defaults-registry-wrapper-red：两份包装各点名一处（共 2 处）：scripts/hosts/padded_entry.sh 的包装取不出时照打 1（`heavy_thread_default || printf '1\n'`），
#   scripts/hosts/literal_entry.sh 的包装只打一个写死的数、不调 heavy_thread_default。
#   heavy-thread-defaults-registered-split-green：crates/demo/src/shares.rs 里 `(heavy_threads / 2).max(1)` 的 heavy_threads 是函数参数、文件里唯一一处调它交 default_heavy_threads().get()，
#   `registered / 2` 的 registered 由 let 绑定 default_heavy_threads().get()，两行都不点名，判绿；heavy-thread-defaults-registered-split-red：crates/demo/src/shares.rs 里
#   调同一个函数的一处交写死的 8、let 绑定的是 available_parallelism 之外的一个参数名，两行的「核数的一半」照点名（共 2 处）。
#   heavy-thread-defaults-exempt-hits-split-green：setup.sh 提交 scripts/new-rig/report_cores.sh（两行 $(nproc) 只打核数）与登记它的排除表一行，工作区里两行各挪进
#   report_cores-count.sh、report_cores-show.sh（钩子拆法的部分文件）、表里改成两行：命中之和 2 等于基准，判绿并写「按放行的被判行数比」；
#   heavy-thread-defaults-exempt-hits-grown-red：setup.sh 提交只有一行 $(nproc) 的 report_cores.sh 与它那一行排除表，工作区里那份多一行 $(nproc)、表不变：命中 2 比基准 1 多，判红点名「排除表涨了」。
cell_heavy_thread_defaults() {
[[ -n "${HEAVY_THREAD_RECOGNITION_LIBRARY:-}" ]] || { echo "  ✗ 认合规写法的那几段判法是空的：source 不了 $STAGE_DIRECTORY/lib/code-tooling-heavy-thread-recognition.sh"; echo "     → 怎么办：它随仓走（.claude/gate.d/lib/code-tooling-heavy-thread-recognition.sh），被删了就从 git 找回来；这一格的那几段判法只在那一份里写。"; exit 1; }
heavy_threads_library_file="$STAGE_REPOSITORY/scripts/hosts/heavy_threads.py"
[[ -f "$heavy_threads_library_file" ]] || { echo "  ✗ 找不到 $heavy_threads_library_file"; echo "     → 怎么办：它随仓走（scripts/hosts/heavy_threads.py），被删了就从 git 找回来；登记位的判法只在那一份里写。"; exit 1; }
test_module_files_library_file="$STAGE_REPOSITORY/scripts/gate/lib_rust_test_module_files.py"
[[ -f "$test_module_files_library_file" ]] || { echo "  ✗ 找不到 $test_module_files_library_file"; echo "     → 怎么办：它随仓走（scripts/gate/lib_rust_test_module_files.py），被删了就从 git 找回来；整份文件算测试代码的认法只在那一份里写。"; exit 1; }
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
exempt_table_baseline .claude/heavy-thread-exempt "$base_table"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$heavy_threads_library_file" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" "$SOURCE_CODE_LINES_LIBRARY" "$test_module_files_library_file" \
  "$EXEMPT_ROW_GROWTH_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" "$STAGE_DIRECTORY/lib-lag-baseline.py" "$HEAVY_THREAD_RECOGNITION_LIBRARY" <<'PY'
import ast, collections, importlib.util, io, os, re, sys, tokenize, types, warnings
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[7], "source-code-lines-library", "exec"), source_lines.__dict__)
exempt_growth = types.ModuleType("lib_exempt_row_growth")
exec(compile(sys.argv[9], "exempt-row-growth-library", "exec"), exempt_growth.__dict__)
split_family_library_file = sys.argv[10]
split_family_specification = importlib.util.spec_from_file_location("lib_lag_split_family", split_family_library_file)
split_family = importlib.util.module_from_spec(split_family_specification)
try:
    split_family_specification.loader.exec_module(split_family)
except (OSError, ImportError, SyntaxError) as error:
    print(f"  ✗ 载不进 {split_family_library_file}：{error}")
    print("     → 怎么办：它随仓走（scripts/gate/lib_lag_split_family.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；拆分族的认法只在那一份里写。")
    sys.exit(1)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
library_file, broken, base_state, base_name, base_table = sys.argv[2:7]
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(library_file))
try:
    import heavy_threads
except (ImportError, SyntaxError) as error:
    print(f"  ✗ 载不进 {library_file}：{error}")
    print("     → 怎么办：照它的报错修好（python3 -m py_compile 先过），登记位的判法只在那一份里写。")
    sys.exit(1)
test_module_files_library_file = sys.argv[8]
test_module_files_spec = importlib.util.spec_from_file_location("lib_rust_test_module_files", test_module_files_library_file)
test_module_files = importlib.util.module_from_spec(test_module_files_spec)
try:
    test_module_files_spec.loader.exec_module(test_module_files)
except (ImportError, SyntaxError) as error:
    print(f"  ✗ 载不进 {test_module_files_library_file}：{error}")
    print("     → 怎么办：照它的报错修好（python3 -m py_compile 先过），整份文件算测试代码的认法只在那一份里写。")
    sys.exit(1)

REGISTRATION = ".claude/heavy-thread-default"
EXEMPT_TABLE = ".claude/heavy-thread-exempt"
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks", "crates")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/",)
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python", ".rs": "rust"}
THREAD_VARIABLE = r"[A-Z0-9_]*(?:THREADS?|THREAD_CAP|JOBS|WORKERS)[A-Z0-9_]*"
THREAD_DEFAULT_KEPT_LITERAL = re.compile(r"nproc|.*capped\.sh|[0-9]+|--(?:threads|jobs|workers|lanes)|threads|jobs|workers|" + THREAD_VARIABLE)
THREAD_DEFAULT_PREFILTER = re.compile(r"nproc|cpu_count|sched_getaffinity|available_parallelism|available_threads|min\s*\(\s*16|\.min\(|capped\.sh|THREAD|JOBS|WORKERS"
                       r"|threads|jobs|workers|lanes|/\s*2")


def thread_default_form(name, pattern):
    return name, re.compile(pattern)


CONDITIONAL_ASSIGNMENT_FORM = "写死的线程数默认（|| / && 赋值）"      # shell 里没设就给默认值的 `[[ -z "$X" ]] && X=8`、`… || X=8`
CONDITIONAL_ASSIGNMENT_LITERAL = r"[1-9][0-9]*"
CONDITIONAL_ASSIGNMENT_VALUE = (CONDITIONAL_ASSIGNMENT_LITERAL if broken == "thread-default-conditional-assignment-literal-only"
                                else CONDITIONAL_ASSIGNMENT_LITERAL + r"|\$\{?(?!SINGLEFS_THREAD_CAP\b)[A-Za-z_][A-Za-z0-9_]*\}?|\$\(\([^\n]*?\)\)")      # 钉数：字面正整数，或 $变量、$((算术))（默认值先放进变量、先算好再赋）；$(命令) 与 $SINGLEFS_THREAD_CAP 不算
CONDITIONAL_ASSIGNMENT_PATTERN = re.compile(r"(?:\|\||&&)[ \t]*\{?[ \t]*(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?" + THREAD_VARIABLE
                                            + r"=[\"']?(?P<value>" + CONDITIONAL_ASSIGNMENT_VALUE + r")[\"']?(?=[ \t]*(?:$|[;)}&|]))")      # 赋值到此为止（后面不跟命令）：`&& X=8 cmd` 是给 cmd 的环境前缀，不是 shell 里的默认值
CONDITIONAL_ASSIGNMENT_VARIABLE = re.compile(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?")
# shell 里任何变量的赋值（行首、空白或 ; & | ( 之后）；右边取到下一个不在引号与 $( ) 里的 ; & | 或空白为止
SHELL_ANY_ASSIGNMENT = re.compile(r"(?:^|(?<=[\s;&|(!]))(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)="
                                  r"(?P<rhs>(?:\"(?:[^\"\\]|\\.)*\"|'[^']*'|\$\((?:[^()]|\([^()]*\))*\)|[^;&|\"'\s()])*)", re.M)
REGISTRY_READING = re.compile(r"\bheavy_thread_default\b|\bdefault_heavy_threads\b|\$\{?SINGLEFS_THREAD_CAP\b")   # 只有弄坏开关 thread-default-registry-substring-accepted 下还在用
REGISTRY_READING_OTHER_NUMBER = re.compile(r"(?<![\w.])[0-9]+(?![\w.])")   # 同上
# 右边整条是登记位读法："$(heavy_thread_default)"（可带 2>/dev/null）、"$(default_heavy_threads)"、"${SINGLEFS_THREAD_CAP}"，引号与花括号可省
REGISTRY_WHOLE_RIGHT_SIDE = re.compile(r'"?(?:\$\((?:heavy_thread_default(?:[ \t]+2>/dev/null)?|default_heavy_threads)\)|\$\{?SINGLEFS_THREAD_CAP\}?)"?')
VARIABLE_WITH_DEFAULT = re.compile(r"\$\{([A-Za-z_][A-Za-z0-9_]*):?[-=]")   # ${变量:-…}、${变量:=…}（不带冒号的同算）：它可能先由别处给了值或补了默认
SHELL_COMMAND_START = r"(?:^|[;&|({!]|\b(?:then|do|else|if|while|until)\b)[ \t]*"
SHELL_SOURCE = re.compile(SHELL_COMMAND_START + r"(?:source|\.)[ \t]+(?P<target>[^=\s;&|\n][^;&|\n]*)", re.M)   # 被 source 的那一段取到命令收尾（; & | 或行尾）
REGISTRY_LIBRARY_SOURCE = re.compile(r"/heavy-threads\.sh[\"']?[ \t]*$|^[\"']?heavy-threads\.sh[\"']?[ \t]*$")   # source 登记位那一份（scripts/hosts/heavy-threads.sh）不算 source 了别的文件
ASSIGNMENT_OPERATOR = r"[ \t]*(?:[-+*/%^&|]|<<|>>)?=(?!=)"


def other_assignment_forms(name):
    """不写成「名=值」的赋值写法（同一份文件里出现一处，这个变量就不算登记位取的）：(( 名 = … )) 与 (( 名++ ))、let、declare -i、read、printf -v、for 名 in、名+=、eval。"""
    word = r"(?<![\w${])" + re.escape(name) + r"\b"
    return [
        ("(( 名 = … ))", re.compile(r"\(\([^\n]*?(?:" + word + ASSIGNMENT_OPERATOR + "|" + word + r"[ \t]*(?:\+\+|--)|(?:\+\+|--)[ \t]*" + word + ")")),
        ("let", re.compile(SHELL_COMMAND_START + r"let\b[^\n;&|]*?" + word + r"(?:" + ASSIGNMENT_OPERATOR + r"|[ \t]*(?:\+\+|--))", re.M)),
        ("declare -i", re.compile(SHELL_COMMAND_START + r"(?:declare|typeset|local)[ \t]+(?:-[A-Za-z]+[ \t]+)*?-[A-Za-z]*i[A-Za-z]*\b[^\n;&|]*?" + word, re.M)),
        ("read", re.compile(SHELL_COMMAND_START + r"read\b[^\n;&|<>]*?" + word, re.M)),
        ("printf -v", re.compile(r"\bprintf[ \t]+(?:-[A-Za-z]+[ \t]+)*?-v[ \t]*[\"']?" + re.escape(name) + r"\b")),
        ("for 名 in", re.compile(r"\b(?:for|select)[ \t]+" + re.escape(name) + r"(?:[ \t]+in\b|[ \t]*;|[ \t]*$)", re.M)),
        ("名+=", re.compile(word + r"\+=")),
        ("eval", re.compile(SHELL_COMMAND_START + r"eval\b[^\n]*?" + word, re.M)),
    ]


def registry_sourced_variables(shell_code):
    """shell 文件里只由登记位读法赋值的变量名，三样都满足才算：每一处「名=值」的右边整条是登记位读法（REGISTRY_WHOLE_RIGHT_SIDE）；
    同一份文件里这个名字没出现在 ${名:-…}、${名:=…} 里，也没有别的赋值写法（other_assignment_forms）；文件没有 source 登记位那一份之外的别的文件。
    `|| X=$变量` 的变量在这里面的不算钉数。"""
    if broken != "thread-default-sourcing-ignored" and any(not REGISTRY_LIBRARY_SOURCE.search(found.group("target")) for found in SHELL_SOURCE.finditer(shell_code)):
        return set()
    verdicts = {}
    for found in SHELL_ANY_ASSIGNMENT.finditer(shell_code):
        rhs = found.group("rhs")
        if broken == "thread-default-registry-substring-accepted":
            from_registry = REGISTRY_READING.search(rhs) is not None and not REGISTRY_READING_OTHER_NUMBER.search(REGISTRY_READING.sub("", rhs))
        else:
            from_registry = REGISTRY_WHOLE_RIGHT_SIDE.fullmatch(rhs) is not None
        verdicts[found.group("name")] = verdicts.get(found.group("name"), True) and from_registry
    defaulted = set() if broken == "thread-default-defaulted-variable-accepted" else set(VARIABLE_WITH_DEFAULT.findall(shell_code))
    return {name for name, from_registry in verdicts.items()
            if from_registry and name not in defaulted
            and (broken == "thread-default-other-assignment-forms-unseen" or not any(pattern.search(shell_code) for _form, pattern in other_assignment_forms(name)))}


# 「写死的线程数默认」里字典键给数的那一种（键是 threads / jobs / workers、值是整数）：自证与单测里拼输入常这样写，单拿出来认
THREAD_DEFAULT_DICT_LITERAL_PATTERN = r"[\"'](?:threads|jobs|workers)[\"']\s*:\s*[0-9]+\b"
THREAD_DEFAULT_DICT_LITERAL = re.compile(THREAD_DEFAULT_DICT_LITERAL_PATTERN)
PINNED_DEFAULT_FORM = "写死的线程数默认"
THREAD_DEFAULT_FORMS = [
    thread_default_form("nproc", r"\bnproc\b"),
    thread_default_form("cpu_count", r"\bcpu_count\s*\("),
    thread_default_form("sched_getaffinity", r"\bsched_getaffinity\b"),
    thread_default_form("available_parallelism", r"\bavailable_parallelism\b"),
    thread_default_form("available_threads（按此刻忙闲现算）", r"\bavailable_threads\b"),
    thread_default_form("min(16", r"\bmin\s*\(\s*16\b|\.min\(\s*16\s*\)"),
    thread_default_form(PINNED_DEFAULT_FORM, r"\$\{" + THREAD_VARIABLE + r":-[0-9]+\}"
                        r"|[\"']--(?:threads|jobs|workers|lanes)[\"'][^\n]*?\bdefault\s*=\s*[0-9]+"
                        r"|\b(?=[A-Z0-9_]*(?:DEFAULT|WHEN_UNSET|BY_PROCESSOR))(?=[A-Z0-9_]*(?:THREADS|JOBS|WORKERS))[A-Z0-9_]+\s*(?::\s*\w+\s*)?=\s*[0-9]+\b"
                        r"|" + THREAD_DEFAULT_DICT_LITERAL_PATTERN),
    (CONDITIONAL_ASSIGNMENT_FORM, CONDITIONAL_ASSIGNMENT_PATTERN),
]
CPU_COUNT_CALL = dict(THREAD_DEFAULT_FORMS)["cpu_count"]
# 核数只记进报告：字典里键是核数一类（cpus、cores、processors 这几个名字）的那一项，值就是取核数那一次调用
CORE_COUNT_REPORT = re.compile(r"[\"'](?:cpus|cores|processors|cpu_count|core_count|logical_processors)[\"']\s*:\s*(?:os\.)?cpu_count\s*\(\s*\)")
# 自证与单测的代码（python，ast 读）：名字里带 selftest / self_test 的函数体、基类是 TestCase 的类体
SAMPLE_SCOPE_FUNCTION_NAME = re.compile(r"(?i)self_?test")
# 名字片段：引号里只有线程字样本身（前后可带一个下划线），拿来比后缀、查子串，不是哪一个变量的名字
NAME_FRAGMENT = re.compile(r"_?(?:THREADS?|THREAD_CAP|JOBS|WORKERS)_?")
# 路径变量：名字以 _FILE、_PATH、_DIRECTORY、_DIR 收尾的，值是一条路径，不是线程数
PATH_VARIABLE_NAME = re.compile(r"_(?:FILE|PATH|DIRECTORY|DIR)$")
HALF = re.compile(r"(?<![/*\w])/{1,2}\s*2(?![\w.])")
HALF_FORM = "核数的一半"
# shell 自证里喂给样本的环境词：「名=<数>」整个是一个词（前后不接别的字）
SHELL_SAMPLE_ENVIRONMENT_WORD = re.compile(r"(?<![\w$])[A-Za-z_][A-Za-z0-9_]*=[0-9]+(?![\w.])")
THREAD_DEFAULT_HALF_CONTEXT = re.compile(r"(?i)nproc|cpu_count|core|processor|parallel|thread|jobs|worker|核")
LITERAL_FALLBACK = re.compile(r"unwrap_or\(\s*[1-9][0-9]*\s*\)|Err\(_\)\s*=>\s*[1-9][0-9]*\b|\bor\s+[1-9][0-9]*\b")
THREAD_VARIABLE_NAMED = re.compile(r"[\"']" + THREAD_VARIABLE + r"[\"']")
FALLBACK_WINDOW = 5
OTHER_QUANTITY = re.compile(r"(?i)timeout|deadline|seconds|duration|超时")   # 回退值所在那一行写着这些词：那个数是别的量（超时秒数一类），不是线程数
CAPPED = re.compile(r"capped\.sh(?P<close>[\"']?)(?:\s*,\s*[\"']?|\s+)[0-9]+\b")
CAPPED_FILE = "scripts/hosts/capped.sh"
OWN_VARIABLE_QUOTED = re.compile(r"[\"'](" + THREAD_VARIABLE + r")[\"'](?!\s*\]\s*=(?!=))")      # ["X"] = 值 是写给子进程的，不是读
OWN_VARIABLE_QUOTED_WRITE_COUNTED = re.compile(r"[\"'](" + THREAD_VARIABLE + r")[\"']")
ENVIRONMENT_READ = re.compile(r"\benviron\b|\bgetenv\b|\benv::var(?:_os)?\b|\boption_env!|\benv!\s*\(|\bprintenv\b"
                              r"|\b(?:\w+_)?(?:env|environment)\s*(?:\.get\s*\(|\[)|\bin\s+(?:\w+\.)?(?:env|environment)\b")   # 只有弄坏开关 thread-default-quoted-name-needs-environment-read 用：读环境变量的写法：os.environ、os.getenv、env::var、env!、printenv，以及读叫 env / environment 的映射（environment.get(…)、environment[…]、"X" in environment）
OWN_VARIABLE_SHELL = re.compile(r"\$\{?(" + THREAD_VARIABLE + r")")
OWN_VARIABLE_PRINTENV = re.compile(r"\bprintenv\b(?:[ \t]+-[-\w]+)*[ \t]+[\"']?(" + THREAD_VARIABLE + r")")      # shell 里 `printenv NAME` 从环境里读 NAME，文件里赋没赋过值都一样
SHELL_ASSIGNMENT_BODY = r"[ \t]*(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?(?P<name>" + THREAD_VARIABLE + r")=(?P<rhs>(?:\"(?:[^\"\\]|\\.)*\"|'[^']*'|[^;&|\"'\n])*)"
SHELL_ASSIGNMENT = re.compile(r"(?:^|[;&|])" + SHELL_ASSIGNMENT_BODY, re.M)       # 行首，或 ; && || 之后同一行里接着赋值
SHELL_ASSIGNMENT_LINE_START_ONLY = re.compile(r"^" + SHELL_ASSIGNMENT_BODY + r".*$", re.M)
CAP_OR_REGISTRY = re.compile(r"SINGLEFS_THREAD_CAP|heavy_thread_default|default_heavy_threads|heavy-threads\.sh|heavy_threads|capped\.sh")
RUST_MODULE_ITEM = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?mod\b")
RUST_EXTERNAL_MODULE = re.compile(r"(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*;")      # `mod tests;`：外置模块只是声明，测试模块的内容在另一个文件里
STATEMENT_LINE_LIMIT = 40
EXEMPT_ROW = re.compile(r"(?P<path>[^\s:#]+):(?P<anchor>\S(?:.*?\S)?)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
strings_scanned = broken == "thread-default-strings-scanned"
quoted_mask = source_lines.quoted_mask      # 逐行只留代码的读法在共用库 SOURCE_CODE_LINES_LIBRARY，这一格只给它认哪些字面量（THREAD_DEFAULT_KEPT_LITERAL）


def shell_code_lines(text):
    return source_lines.shell_code_lines(text, OUTPUT_TEXT)


def python_code_lines(text, keep_text=False):
    return source_lines.python_code_lines(text, OUTPUT_TEXT, THREAD_DEFAULT_KEPT_LITERAL, strings_scanned or keep_text)


def rust_code_lines(text, keep_text=False):
    return source_lines.rust_code_lines(text, THREAD_DEFAULT_KEPT_LITERAL, strings_scanned or keep_text)


BREAK_SWITCH_TEST_BEFORE_AND = re.compile(r"(?:\b[a-z_]*break_is\b|\$\{?[A-Z][A-Z0-9_]*_BREAK\b)[^;|&]*$")   # `&&` 左边紧挨着的弄坏开关判断


def break_guarded(code, found):
    """`&& X=<值>` 的 `&&` 左边紧挨着是弄坏开关的判断：那一支只在自证打开那条开关时走到，不是没设就给的默认值（格上方注释）。"""
    return (found.group(0).startswith("&&") and broken != "thread-default-break-guarded-counted"
            and BREAK_SWITCH_TEST_BEFORE_AND.search(code[:found.start()]) is not None)


def pinned_conditional_assignment(found, registry_variables, code=""):
    """`|| X=<值>` 这一处算不算钉数：值是 $变量、而那个变量在同一份文件里只由登记位读法赋值（registry_sourced_variables）的不算；弄坏开关护着的（break_guarded）不算。"""
    if break_guarded(code, found):
        return False
    variable = CONDITIONAL_ASSIGNMENT_VARIABLE.fullmatch(found.group("value"))
    return not (variable and variable.group(1) in registry_variables and broken != "thread-default-conditional-variable-source-ignored")


def python_sample_scope_lines(text):
    """python 里自证与单测的行号集合（1 起）：名字里带 selftest / self_test 的函数、基类是 TestCase 的类，从头一行到末一行。
    ast 读不了（语法错、不是这一版 python 认的写法）交空集，照常判。"""
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")      # 不规范的转义一类的语法警告不打进门禁输出
            tree = ast.parse(text)
    except (SyntaxError, ValueError):
        return frozenset()
    lines = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and SAMPLE_SCOPE_FUNCTION_NAME.search(node.name):
            lines.update(range(node.lineno, node.end_lineno + 1))
        elif isinstance(node, ast.ClassDef) and any((base.attr if isinstance(base, ast.Attribute) else getattr(base, "id", "")) == "TestCase" for base in node.bases):
            lines.update(range(node.lineno, node.end_lineno + 1))
    return frozenset(lines)


def core_count_only_reported(code, text_of_line):
    """这一行取核数的每一次调用都是字典里核数那一键的值（CORE_COUNT_REPORT）：只把核数记进报告，不定并行度。"""
    if broken == "thread-default-core-report-counted":
        return False
    calls = len(CPU_COUNT_CALL.findall(code))
    return calls > 0 and len(CORE_COUNT_REPORT.findall(text_of_line())) == calls


def forms_of(code, window, text_of_line, registry_variables=frozenset(), sample_scope=False, shell_sample_scope=False):
    """text_of_line：这一行连字符串带键名的样子（不含注释），只在要判「往上 5 行里点了线程变量名」与核数是不是只记进报告那两路时才取；
    registry_variables：这份 shell 文件里只由登记位读法赋值的变量（registry_sourced_variables），`|| X=$变量` 的变量在里面的不算钉数；
    sample_scope：这一行在 python 的自证函数或单测类里（python_sample_scope_lines），字典键给的数与 capped.sh 后面的数是样本值，不算；
    shell_sample_scope：这一行在 shell 的自证函数里（shell_selftest_lines），「名=<数>」这种喂给样本的环境词是样本值，不算（同一行别的写法照判）。"""
    names = []
    for name, pattern in THREAD_DEFAULT_FORMS:
        if name == CONDITIONAL_ASSIGNMENT_FORM:
            if broken != "thread-default-conditional-assignment-unpinned" and any(pinned_conditional_assignment(found, registry_variables, code) for found in pattern.finditer(code)):
                names.append(name)
        elif not pattern.search(code):
            continue
        elif name == PINNED_DEFAULT_FORM and sample_scope and not pattern.search(THREAD_DEFAULT_DICT_LITERAL.sub(" ", code)):
            continue      # 自证与单测里只有字典键给的样本值
        elif name == PINNED_DEFAULT_FORM and shell_sample_scope and not pattern.search(SHELL_SAMPLE_ENVIRONMENT_WORD.sub(" ", code)):
            continue      # shell 自证里只有喂给样本的环境词
        elif pattern is CPU_COUNT_CALL and core_count_only_reported(code, text_of_line):
            continue      # 只把核数记进报告
        else:
            names.append(name)
    if broken != "thread-default-half-unchecked" and HALF.search(code) and THREAD_DEFAULT_HALF_CONTEXT.search(code):
        names.append(HALF_FORM)
    if LITERAL_FALLBACK.search(code):
        if THREAD_VARIABLE_NAMED.search(code):
            names.append("写死的线程数默认（回退值）")
        elif any(THREAD_VARIABLE_NAMED.search(line) for line in window) and (broken == "thread-default-timeout-unchecked" or not OTHER_QUANTITY.search(text_of_line())):
            names.append("写死的线程数默认（回退值）")
    if not sample_scope and any(found.group("close") or not quoted_mask(code)[found.start()] for found in CAPPED.finditer(code)):
        names.append("capped.sh <数字>")
    return names


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX:
                    yield directory, os.path.join(current, name), LANGUAGE_OF_SUFFIX[os.path.splitext(name)[1]]


def shell_local_names(code):
    """shell 代码（行接起来的文本）里文件自己赋了值的变量（WORKERS="${2:?…}" 这种是局部变量，不是从环境读的）；有一处右边读着它自己（X="${X:-4}"）的，仍是读环境。"""
    pattern = SHELL_ASSIGNMENT_LINE_START_ONLY if broken == "thread-default-shell-inline-assignment-unseen" else SHELL_ASSIGNMENT
    assignments = list(pattern.finditer(code))
    return ({found.group("name") for found in assignments if found.group("name") not in found.group("rhs")}
            - {found.group("name") for found in assignments if found.group("name") in found.group("rhs")})


SHELL_PATHS = [os.path.normpath(path) for _directory, path, language in scanned_files() if language == "shell"]
shell_code_cache = {}


def shell_code_of(path):
    if path not in shell_code_cache:
        try:
            shell_code_cache[path] = "\n".join(shell_code_lines(open(path, encoding="utf-8", errors="replace").read()))
        except OSError:
            shell_code_cache[path] = ""
    return shell_code_cache[path]


def shell_family_of(path):
    """path 所在拆分族里被扫的 shell 文件：先往上找 path 是哪份入口的部分文件（嵌套就一层层往上），再取那份入口的族（scripts/gate/lib_lag_split_family.py 的 family_owner、family_members）。"""
    root = os.path.normpath(path)
    while True:
        owner = split_family.family_owner(root, [other for other in SHELL_PATHS if other != root])
        if owner is None:
            return split_family.family_members(root, SHELL_PATHS)
        root = owner


def family_shell_local_names(path):
    """path 所在拆分族的每份文件合起来里赋了值的局部变量：入口里赋、拆出的库里读的变量，拆分之前在同一份文件里。"""
    return shell_local_names("\n".join(shell_code_of(member) for member in shell_family_of(path)))


def rust_test_module_starts(code_lines, index):
    """行首 #[cfg(test)]（第 index 行，0 起）罩着的是不是带着正文的测试模块：它后面第一个非空行（夹着的 #[…] 属性不算）是 mod、且不是分号结尾的外置模块声明（`mod tests;`）才是；
    只罩一条 use、fn、const 的不是。"""
    for piece in [code_lines[index][len("#[cfg(test)]"):]] + code_lines[index + 1:]:
        text = re.sub(r"^(?:#\[[^\]]*\]\s*)+", "", piece.strip())
        if text:
            return RUST_MODULE_ITEM.match(text) is not None and (broken == "thread-default-external-mod-skipped" or RUST_EXTERNAL_MODULE.match(text) is None)
    return False


def python_statement_ranges(text):
    """python 每一行落在哪条逻辑行（起止行，1 起）里；tokenize 读不了返回空表（那时按反斜杠续行算）。"""
    ranges, first = {}, None
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
    except (tokenize.TokenError, SyntaxError, IndentationError):
        return ranges
    for token in tokens:
        if token.type in (tokenize.NL, tokenize.COMMENT, tokenize.INDENT, tokenize.DEDENT, tokenize.ENCODING, tokenize.ENDMARKER):
            continue
        if token.type == tokenize.NEWLINE:
            if first is not None:
                ranges.update({row: (first, token.start[0]) for row in range(first, token.start[0] + 1)})
            first = None
        elif first is None:
            first = token.start[0]
    return ranges


def statement_range(language, code_lines, number, python_ranges):
    """第 number 行（1 起）所在那一条语句的起止行：python 取逻辑行；shell 取反斜杠续行；
    rust 往前到上一条语句收尾（; { }）之后，往后到括号配平且以 ; 或 } 收尾、或闭合了外层的块，两头各不超过 STATEMENT_LINE_LIMIT 行。"""
    index = number - 1
    if language == "python" and number in python_ranges:
        return python_ranges[number]
    if language != "rust":
        first, last = index, index
        while first > 0 and code_lines[first - 1].rstrip().endswith("\\"):
            first -= 1
        while last + 1 < len(code_lines) and code_lines[last].rstrip().endswith("\\"):
            last += 1
        return first + 1, last + 1
    first, previous = index, index - 1
    while previous >= 0 and index - previous <= STATEMENT_LINE_LIMIT:
        text = code_lines[previous].strip()
        if text:
            if text.endswith((";", "{", "}")):
                break
            first = previous
        previous -= 1
    depth, last = 0, first
    while last + 1 < len(code_lines) and last - first < STATEMENT_LINE_LIMIT:
        text = code_lines[last]
        depth += sum(text.count(opening) for opening in "([{") - sum(text.count(closing) for closing in ")]}")
        if last >= index and text.strip() and (depth < 0 or (depth == 0 and text.rstrip().endswith((";", "}")))):
            break
        last += 1
    return first + 1, last + 1


problems_registration = []
try:
    registered = heavy_threads.default_heavy_threads({}, registration_file=REGISTRATION)
except heavy_threads.HeavyThreadDefaultError as error:
    registered = None
    problems_registration.append((str(error), error.howto))

# capped.sh 设的线程变量（variable_names 清单）：装置读的线程变量在里面，capped.sh <上限> 的数才到得了装置
capped_names = set()
try:
    capped_block = re.search(r"^variable_names=\(\n(.*?)^\)", open(CAPPED_FILE, encoding="utf-8").read(), re.S | re.M)
except OSError as error:
    capped_block = None
    problems_registration.append((f"读不了 {CAPPED_FILE}：{error}", f"这一格拿它的 variable_names 清单当「capped.sh 设了哪些线程变量」；它随仓走，被删了就从 git 找回来"))
else:
    if capped_block is not None:
        capped_names = set(re.findall(r"^\s+([A-Z][A-Z0-9_]*)\s*$", capped_block.group(1), re.M))
    if not capped_names:
        problems_registration.append((f"{CAPPED_FILE} 里读不出 variable_names=( … ) 清单里的变量名", "清单一行一个变量名、独占一行，写在 variable_names=( 与单独一行的 ) 之间；写法照 capped.sh 现在的样子"))
if broken == "thread-default-capped-list-ignored":
    capped_names = set()       # 弄坏：清单读了不认，每个线程变量都算「capped.sh 不设」
# 整份算测试代码的 rust 文件（认法在 scripts/gate/lib_rust_test_module_files.py）不扫；弄坏开关 thread-default-test-module-file-scanned 下照产品代码扫
whole_test_paths = (set() if broken == "thread-default-test-module-file-scanned"
                    else test_module_files.whole_test_files(".", [path for _directory, path, language in scanned_files() if language == "rust"]))
skipped_test_files = 0
own_variable_checked = broken != "thread-default-own-variable-unchecked"
own_reads = set()              # (路径, 变量名)：装置自己读的线程变量，不管 capped.sh 设没设
own_outside = 0                # 其中 capped.sh 不设的个数（这些要么文件里回退到上限或登记值，要么落在被判行上交给排除表）
own_ignoring = []              # (路径, 变量名)：capped.sh 不设、文件里也没回退到上限或登记值

hits = {}                      # 路径 → [(行号, 种类, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
# 认合规写法的几段判法（shell 自证函数的行、登记位包装函数、rust 从登记位取好的线程数再分）在 lib/code-tooling-heavy-thread-recognition.sh 的
# HEAVY_THREAD_RECOGNITION_LIBRARY 里：exec 进模块对象（算键库认的写法，scripts/gate/lib_function_units.py 文件头「exec 进模块对象」），
# 它读的这一格的判法、表与常量逐个赋到模块对象上（名单在那一份的文件头），按 heavy_thread_recognition.<函数>(…) 调
heavy_thread_recognition = types.ModuleType("lib_heavy_thread_recognition")
exec(compile(sys.argv[12], "heavy-thread-recognition-library", "exec"), heavy_thread_recognition.__dict__)
heavy_thread_recognition.HALF = HALF
heavy_thread_recognition.SAMPLE_SCOPE_FUNCTION_NAME = SAMPLE_SCOPE_FUNCTION_NAME
heavy_thread_recognition.SHELL_PATHS = SHELL_PATHS
heavy_thread_recognition.comment_stripped_shell_lines = source_lines.shell_code_lines
heavy_thread_recognition.rust_code_lines = rust_code_lines
heavy_thread_recognition.rust_test_module_starts = rust_test_module_starts
heavy_thread_recognition.scanned_files = scanned_files
heavy_thread_recognition.whole_test_paths = whole_test_paths


def scan_text(path, text, language):
    """一份文件的正文（此刻的，或排除表比基准时基准那一版的）→ (被判行 [(行号, 种类, 原行)], 装置自己读的线程变量 {名: [行号]},
    其中 capped.sh 不设的 {名: [行号]}, 其中读它的那条语句里也没回退到上限或登记值的 {名: 第一次读的行号})。
    跨文件的读法（capped.sh 的清单、同一拆分族的局部变量、登记位包装函数、rust 函数在别的文件里的调用处）都取此刻的树。"""
    entries = []
    original = text.split("\n")
    code_lines = {"shell": shell_code_lines, "python": python_code_lines, "rust": rust_code_lines}[language](text)
    text_lines = []                # 连字符串一起留着的版本：回退值那一行用得着时才算，整份文件最多算一次

    def text_line_at(number):
        if not text_lines:
            text_lines.extend({"shell": shell_code_lines, "python": python_code_lines, "rust": rust_code_lines}[language](text, **({} if language == "shell" else {"keep_text": True})))
        return text_lines[number - 1] if number <= len(text_lines) else ""
    window = collections.deque(maxlen=FALLBACK_WINDOW)
    registry_variables = registry_sourced_variables("\n".join(code_lines)) if language == "shell" else frozenset()
    # 自证函数与单测类的行：只在文件里有 capped.sh 或字典键给数的写法时才读 ast（弄坏开关 thread-default-selftest-samples-counted 下不认）
    sample_scope_lines = (python_sample_scope_lines(text) if language == "python" and broken != "thread-default-selftest-samples-counted"
                          and ("capped.sh" in text or THREAD_DEFAULT_DICT_LITERAL.search(text)) else frozenset())
    # shell 自证函数的行（弄坏开关 thread-default-shell-selftest-samples-counted 下不认）
    shell_sample_lines = heavy_thread_recognition.shell_selftest_lines(code_lines) if language == "shell" and broken != "thread-default-shell-selftest-samples-counted" else frozenset()
    for number, code in enumerate(code_lines, 1):
        if not code.strip():
            continue
        names = forms_of(code, window, lambda number=number: text_line_at(number), registry_variables, number in sample_scope_lines, number in shell_sample_lines)
        window.append(code)
        if (language == "rust" and HALF_FORM in names and broken != "thread-default-registered-split-unrecognized"
                and heavy_thread_recognition.registered_split(path, code_lines, number)):
            names = [name for name in names if name != HALF_FORM]      # 从已经按登记位取好的线程数分出去（弄坏开关 thread-default-registered-split-unrecognized 下不认）
        if names:
            entries.append((number, names, original[number - 1] if number <= len(original) else code))
    # 装置自己读的线程变量：代码里点了名（python / rust 的字面量，shell 的 $变量），rust 的测试模块（行首 #[cfg(test)] 后面第一个非空行是 mod，往后）不算
    own_here = {}
    # shell 里文件自己赋了值的变量（WORKERS="${2:?…}" 这种）是局部变量，不是从环境读的；右边读着它自己（X="${X:-4}"）的仍算读
    shell_local = shell_local_names("\n".join(code_lines)) if language == "shell" and broken != "thread-default-shell-local-unchecked" else set()
    statement_ranges = python_statement_ranges(text) if language == "python" else {}

    def statement_reads_environment(number):
        """第 number 行所在的那条语句里有读环境变量的写法（ENVIRONMENT_READ）；只有弄坏开关 thread-default-quoted-name-needs-environment-read 用它。"""
        first, last = statement_range(language, code_lines, number, statement_ranges)
        return any(ENVIRONMENT_READ.search(code_lines[row - 1]) for row in range(first, last + 1))
    for number, code in enumerate(code_lines, 1):
        if (language == "rust" and code.startswith("#[cfg(test)]") and broken != "thread-default-test-module-unskipped"
                and (broken == "thread-default-cfg-test-without-mod" or rust_test_module_starts(code_lines, number - 1))):
            break
        found_names = [found.group(1) for found in (OWN_VARIABLE_QUOTED_WRITE_COUNTED if broken == "thread-default-subscript-write-counted" else OWN_VARIABLE_QUOTED).finditer(code)]
        if found_names and broken == "thread-default-quoted-name-needs-environment-read" and not statement_reads_environment(number):
            found_names = []      # 弄坏：引号里的名字只在读环境变量的写法里才算读（经包装函数、别名、常量读的认不出）
        if broken != "thread-default-name-fragment-counted":
            found_names = [name for name in found_names if not NAME_FRAGMENT.fullmatch(name)]      # 引号里的名字片段（"THREADS"）是拿来比的，不是变量；shell 的 $THREADS 照读算
        if language == "shell":
            # shell 自证函数里单引号括着的 $变量 是写给样本的正文，这一份不读它（shell_selftest_lines）
            single_quoted = heavy_thread_recognition.single_quoted_mask(code) if number in shell_sample_lines else None
            found_names += [found.group(1) for found in OWN_VARIABLE_SHELL.finditer(code)
                            if found.group(1) not in shell_local and not (single_quoted and single_quoted[found.start()])]
            if broken != "thread-default-printenv-unseen":
                found_names += [found.group(1) for found in OWN_VARIABLE_PRINTENV.finditer(code)]
        if broken != "thread-default-path-variable-counted":
            found_names = [name for name in found_names if not PATH_VARIABLE_NAME.search(name)]   # 路径变量（…_WORKERS_FILE）的值不是线程数
        for name in found_names:
            own_here.setdefault(name, []).append(number)
    if language == "shell" and own_here and broken not in ("thread-default-shell-local-unchecked", "thread-default-shell-local-by-file"):
        family_local = family_shell_local_names(path)      # 变量在同一族的别的文件里赋了值（入口赋、拆出的库读）：同一份文件拆开了，照样是局部变量
        for name in [name for name in own_here if name in family_local]:
            del own_here[name]
    outside_here = {name: numbers for name, numbers in own_here.items() if name not in capped_names}
    unfallen = {}
    if outside_here and own_variable_checked:
        python_ranges = python_statement_ranges(text) if language == "python" else {}
        whole_code = "\n".join(code_lines)

        def falls_back(numbers):
            """这个变量读它的任何一处所在的那条语句里有 SINGLEFS_THREAD_CAP、登记值一类的回退，或 shell 里调了登记位包装函数（calls_registry_wrapper）；
            同文件另一条语句里提到不算（只看代码：注释、文档串里写的不算）。"""
            if broken == "thread-default-fallback-by-file":      # 弄坏：整份文件里出现字样就算回退了
                return CAP_OR_REGISTRY.search(whole_code) is not None
            for number in numbers:
                first, last = (number, number) if broken == "thread-default-statement-single-line" else statement_range(language, code_lines, number, python_ranges)
                statement = "\n".join(code_lines[first - 1:last])
                if CAP_OR_REGISTRY.search(statement) or (language == "shell" and broken != "thread-default-registry-wrapper-unrecognized"
                                                         and heavy_thread_recognition.calls_registry_wrapper(statement)):      # 弄坏开关 thread-default-registry-wrapper-unrecognized 下不认包装函数
                    return True
            return False
        unfallen = {name: numbers[0] for name, numbers in outside_here.items() if not falls_back(numbers)}
        if unfallen:

            def own_kind(variable_names):
                return "读自己的线程变量 " + "、".join(sorted(variable_names)[:3]) + "：capped.sh 不设它，读它的那条语句里也没回退到 SINGLEFS_THREAD_CAP 或登记值"
            if broken == "thread-default-exempt-by-file":      # 弄坏：按文件并组，这一种挂到文件里已有的每一条被判行上（新读的那一行不被点名，借已有行的锚串放行）
                if entries:
                    entries = [(number, names + [own_kind(unfallen)], line) for number, names, line in entries]
                else:
                    first_number = min(unfallen.values())
                    entries = [(first_number, [own_kind(unfallen)], original[first_number - 1])]
            else:                   # 按被判行：每个变量落在它第一次被读的那一行（同一行的几个变量写在一起），该行已是被判行的并上去，不是的单列一条，排除表要有锚串落在这一行里才放
                by_number = {number: (names, line) for number, names, line in entries}
                variables_by_number = {}
                for variable_name, number in unfallen.items():
                    variables_by_number.setdefault(number, []).append(variable_name)
                for number, variable_names in variables_by_number.items():
                    names, line = by_number.get(number, ([], original[number - 1] if number <= len(original) else ""))
                    by_number[number] = (names + [own_kind(variable_names)], line)
                entries = [(number, *by_number[number]) for number in sorted(by_number)]
    return entries, own_here, outside_here, unfallen


for directory, path, language in scanned_files():
    if path in whole_test_paths:
        skipped_test_files += 1
        continue
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems_registration.append((f"读不了 {path}：{error}", "看它的权限；这一格要读到射程里的每一份"))
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if not THREAD_DEFAULT_PREFILTER.search(text):
        continue
    entries, own_here, outside_here, unfallen = scan_text(path, text, language)
    own_reads.update((path, name) for name in own_here)
    own_outside += len(outside_here)
    own_ignoring.extend((path, name) for name in unfallen)
    if entries:
        hits[path] = entries


exempt_rows, exempt_problems = [], []
if os.path.isfile(EXEMPT_TABLE):
    for number, line in enumerate(open(EXEMPT_TABLE, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = EXEMPT_ROW.fullmatch(line)
        if not found or len(found.group("reason")) < 4:
            exempt_problems.append(f"第 {number} 行不是「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）：{line[:120]}")
            continue
        exempt_rows.append((number, found.group("path"), found.group("anchor")))
exempted = 0
unexempted = []
for path, entries in sorted(hits.items()):
    anchors = [anchor for _number, row_path, anchor in exempt_rows if row_path == path]
    for number, names, line in entries:
        if any(anchor in line for anchor in anchors):
            exempted += 1
        else:
            unexempted.append((path, number, names, line))
if broken != "thread-default-exempt-unchecked":
    for number, path, anchor in exempt_rows:
        if not os.path.isfile(path):
            exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
        elif not any(anchor in line for _line_number, _names, line in hits.get(path, [])):
            exempt_problems.append(f"第 {number} 行的锚串「{anchor}」在 {path} 里一处被判行都不命中（那一处已经改掉，或锚串抄错了）")

grown, growth_note, hit_counts = [], "", {}
if base_state == "table" and broken != "thread-default-exempt-growth-unchecked":
    # 比的是放行的被判行数（锚串在那份文件里命中的被判行），基准那一侧取基准那一版的文件、照这一格此刻的判法数（scan_text）；
    # 弄坏开关 thread-default-exempt-growth-rows-counted 下照旧数行（每一行算 1）
    base_entries = {}
    lag_baseline = None
    if broken != "thread-default-exempt-growth-rows-counted":
        lag_baseline_specification = importlib.util.spec_from_file_location("lib_lag_baseline", sys.argv[11])
        lag_baseline = importlib.util.module_from_spec(lag_baseline_specification)
        try:
            lag_baseline_specification.loader.exec_module(lag_baseline)
        except (OSError, ImportError, SyntaxError) as error:
            print(f"  ✗ 载不进 {sys.argv[11]}：{error}")
            print("     → 怎么办：它随仓走（.claude/gate.d/lib-lag-baseline.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；取基准那一版的文件只在那一份里写。")
            sys.exit(1)

    def entries_at_base(path):
        """基准那一版的 path（照生效的搬迁登记倒着找）照这一格此刻的判法判出的被判行；基准里没有、不是这一格判的语言交 []，git 取不到记一条问题。"""
        if path not in base_entries:
            data, _shown, problem = lag_baseline.baseline_file_bytes(base_name, path)
            language = LANGUAGE_OF_SUFFIX.get(os.path.splitext(path)[1])
            if problem is not None:
                exempt_problems.append(f"{problem.sentence()}，这一行放行的被判行数没法与基准那一版比")
            base_entries[path] = scan_text(path, data.decode("utf-8", "replace"), language)[0] if problem is None and data is not None and language else []
        return base_entries[path]

    def hit_count(path, anchor, at_base):
        counted = sum(1 for _number, _names, line in (entries_at_base(path) if at_base else hits.get(path, [])) if anchor in line)
        hit_counts[(path, anchor, at_base)] = counted
        return counted
    grown = exempt_growth.grown_exempt_rows(exempt_rows, base_table, EXEMPT_ROW, split_family_library_file, broken == "thread-default-exempt-family-unrecognized",
                                            None if lag_baseline is None else hit_count)
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨（{'按行数比' if lag_baseline is None else '按放行的被判行数比'}）"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

if scanned == 0 and registered is None and not os.path.isfile(EXEMPT_TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 与 {REGISTRATION} 都不在")
    sys.exit(77)
failed = False
for reason, howto in problems_registration:
    failed = True
    print(f"  ✗ {reason}")
    print(f"     → 怎么办：{howto}")
if unexempted:
    failed = True
    print(f"  ✗ {len(unexempted)} 处自己定并行度默认值的写法没经登记位取（每台默认重活线程数登记在 {REGISTRATION}）：")  # gate-lint:summary
    for path, number, names, line in unexempted:
        print(f"     {path}:{number}  {'、'.join(names)}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：shell 先 source scripts/hosts/heavy-threads.sh 再 heavy_thread_default 取（`|| X=$变量` 被点名的，改成 `|| X=$(heavy_thread_default)` 直接取），python 用 scripts/hosts/heavy_threads.py 的 default_heavy_threads() 取，")
    print("               起命令里不写线程（整条经 task-server/task submit 交，服务套 capped.sh <每台默认重活线程数>）；参数或环境变量给了照给的。装置读自己的线程变量的：把变量名加进 scripts/hosts/capped.sh 的 variable_names，")
    print("               或让装置没设时回退到 SINGLEFS_THREAD_CAP / 登记值（rust 装置 include scripts/experiment-apparatus/heavy_threads_default/threads.rs，python 用 default_heavy_threads）。确实不是重活（自证输入、线程数就是场景本身、只报核数不起线程）的，")
    print(f"               登记进 {EXEMPT_TABLE}：一行「<路径>:<锚串>  # 理由」，锚串取那一行里的一段原文；排除只缩不涨。")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，那一处已经改成经登记位取就删掉这一行，锚串取被判那一行里的一段原文；写法见上一条出路。")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有，或放行的被判行比那一版多（同一份文件、或同一拆分族里同一锚串命中的被判行数之和超过基准那一份）：")  # gate-lint:summary
    for row_number, path, anchor in grown:
        counted = hit_counts.get((path, anchor, False))
        print(f"     第 {row_number} 行 {path}:{anchor}" + ("" if counted is None else f"（命中 {counted} 处被判行）"))  # gate-lint:detail
    print("     → 怎么办：排除只缩不涨。新写的默认值经 heavy_thread_default / default_heavy_threads 取，起命令里不写线程（线程由服务定）；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 每台默认重活线程数只经登记位取：登记值 {registered}；扫了 {scanned} 份（{by_directory}），可疑写法 {exempted} 处全在排除表里"
      f"（表 {len(exempt_rows)} 行，每行都命中）；装置自己读的线程变量 {len(own_reads)} 个，其中 capped.sh 不设的 {own_outside} 个都回退到上限或登记值、或被判行在排除表里"
      f"（capped.sh 清单 {len(capped_names)} 个变量）；{growth_note}"
      + (f"；整份算测试代码的 {skipped_test_files} 份 rust 文件不扫" if skipped_test_files else ""))
PY
}

# ── 格 hardcoded-quota：scripts/、.claude/gate.d/、.claude/hooks/ 里自己定内存上限或台数（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03：上限与机器归 task-server，脚本里不定）
#
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/hooks/ 里的 *.sh、*.py、*.rs；跳过 target/、__pycache__/、.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/（样本是造出来的输入）。
# 只看代码：逐行读法在共用库 SOURCE_CODE_LINES_LIBRARY（shell 去掉注释与出路文字；python 经 tokenize、rust 按引号去掉注释与字符串字面量），
#   留下的字面量只有内容整个是 run-with-memory-cap.sh 结尾的路径、--memory-cap / --hosts（可带 =常量）、内存上限写法（正整数加 K / M / G / T）、整数，
#   或名字像内存上限变量（以 MEMORY_CAP / MEMORY_MAX / MEMORY_LIMIT / MEM_CAP 收尾，不分大小写）、台数变量（以 HOSTS / HOST_COUNT / MACHINE_COUNT / PEER_COUNT 收尾）的那几种。
# 算写死的写法（每处记一个或几个种类，一行一处）：
#   run-with-memory-cap.sh <常量>：包装后面直接跟上限写法；包装不在引号里，或它所在的字面量就在它收尾（"…/run-with-memory-cap.sh" 16G、python 列表写法 "run-with-memory-cap.sh", "16G"）；
#   --memory-cap <常量>：同上的选项写法、整个字面量是「--memory-cap=常量」，以及 argparse 的 "--memory-cap" … default=<常量>；
#   写死的内存上限默认：shell 的 ${<内存上限变量>:-<常量>}（:= 与不带冒号的同算，双引号里也算）、不在引号里的 <内存上限变量>=<常量>（行首、空白或 ; & | ( 之后，环境前缀也算）；
#     python / rust 同一行里有内存上限变量的名字（标识符或引号里的名字）又有一个引号里的上限写法（"6G"）；
#   写死的台数：台数变量赋两台以上的整数（shell 不在引号里的 X=2、${X:-2}；python / rust 的 X = 2、let X = 2、关键字实参 X=2），"hosts": 2 这类键，--hosts <整数> 与它的 argparse 默认值；
#     同一份文件里这个变量另有一处赋值的右边不是整数字面量的（hosts=$(host_count) 这类现算的），整数只是没算之前的初值，不判；写 0、1 的是开关或「只在本机」，不判。
# 上限写 auto 照常量判：内存包装只认 正整数加 K / M / G / T，auto 退 251（scripts/hosts/run-with-memory-cap.sh 文件头），上限归 task-server，写 auto 同样是脚本自己定上限。
# 这一格只判写成常量与 auto 的那几种；从多机配置读、上限放在变量里、经 multi-host-run.sh 起而不写 --memory-cap、台数现算的不在这一格判：
#   那几种还在 scripts/hosts/、scripts/experiment-apparatus/ 的几份里有，判了就要给每一处加排除行，而排除只缩不涨；那几份照
#   records/2026-10-30-编译绕过内存总上限.md 收尾批 M03、M06 删干净之后再把这几种并进来。
#   引号里整句拿来当样本的、注释、文档串与出路文字里写的不算；自证里造的假值、写给人照抄的例子、这一轮不改的文件，逐行登记进排除表。
# 排除表 .claude/gate.d/hardcoded-quota-exempt.tsv：一行「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）；那份文件里含锚串的被判行不算。
#   写不对、指向不存在的文件、锚串一处被判行都不命中，都判红；只缩不涨：与改动范围的基准那一版比（exempt_table_baseline，照登记的搬迁换算路径），基准里没有的「路径:锚串」判红；
#   基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明；git 取不到基准那一版判红。
#   一份文件拆成几份的：此刻一行的路径照拆分族（scripts/gate/lib_lag_split_family.py，认法写在它文件头）归到基准里的入口，按（入口, 锚串）比行数之和（EXEMPT_ROW_GROWTH_LIBRARY），超了这一组的行都判涨；族外的路径、锚串对不上的照旧判红。
# 判不了的：变量里拼出来的命令（上限或包装路径放在变量里：["bash", RUNNER, "16G"]）、读别的文件定的数、表格与 heredoc 里写的上限（e184 的档表那类按 | 分列的行）、
#   跨行的写法（续行之后才写常量）、shell 里嵌的 python 字符串（按 shell 判）、台数写成「for host in 本机 第二台」这类逐台列出来的；排除表的理由成不成立靠 review。
# 样本：hardcoded-quota-red 的 scripts/experiment-apparatus/ 下 fixed_wrapper.sh（包装后跟 16G、引号收尾的包装路径后跟 4G）、fixed_default.sh（${…:-24G}、环境前缀 =40G、局部变量 =8G）、
#   fixed_hosts.sh（HOSTS=2、${SAMPLE_HOSTS:-3}），scripts/gate/fixed_driver.py（DEVICE_MEMORY_CAP = "6G"、host_count = 2、列表写法的包装与 "8G"、argparse 默认 "4G"、"hosts": 3），
#   .claude/hooks/fixed_hook.sh（包装后跟 12G）、.claude/gate.d/fixed_stage.sh（${GATE_SAMPLE_MEMORY_MAX:-8G}）与 scripts/experiment-apparatus/auto_cap.sh（包装后跟 auto、${…:-auto}），共 16 处逐处点名；排除表一行放行自证假值（--memory-cap 256M）、
#   一行指向不存在的文件、一行锚串不命中、setup.sh 建 git 仓提交一版不带 grown-row 那一行的表再加回来（表涨了），后三样都点名。
#   hardcoded-quota-green：从多机配置读的上限、包装只带 --status、multi-host-run.sh 不写 --memory-cap、hosts=1 与 emit_hosts=1 / two_hosts=0 这类开关、
#   peer_count=2 之后又照 --emit-hosts 现算，引号里整句的样本、注释、echo、文档串里写的常量，
#   .claude/gate.d/fixtures/ 下写死的样本，都不点名；排除表一行放行自证假值，判绿并报扫了 6 份、表 1 行每行都命中。
#   hardcoded-quota-exempt-family-green：setup.sh 提交 scripts/hosts/split_values.py（DEVICE_MEMORY_CAP = "6G"）与登记它的排除表，工作区里那一行挪进同目录的 split_values_cap.py
#   （Python 拆法的部分文件）、表里那一行改成新路径、锚串不变：此刻一行归到基准里的 split_values.py、锚串相同，判绿并写「那一版比没涨」；
#   hardcoded-quota-exempt-family-red：同一个拆分，挪进的文件叫 other_values.py（族外），表里那一行是基准里没有的，判红点名「排除表涨了」。
# 弄坏开关（CODE_TOOLING_BREAK）：quota-strings-scanned（python 与 rust 的字面量与文档串照代码扫）：hardcoded-quota-green 判红（sentence_samples.py 文档串里的包装与 --memory-cap 被点名）；
#   quota-sentence-samples-counted（引号里整句的样本也算）：hardcoded-quota-green 判红（sentence_samples.sh 的 feed 那一行被点名）；
#   quota-memory-cap-option-unchecked（不认 --memory-cap <常量>）：hardcoded-quota-red 里 fixed_driver.py:15 那条 want 与「16 处」找不到；
#     hardcoded-quota-green 判红（排除表那一行的锚串落在 --memory-cap 256M 那一行上，那一行不再是被判行，锚串一处都不命中）；
#   quota-memory-variable-unchecked（不认写死的内存上限默认）：hardcoded-quota-red 里 fixed_default.sh、fixed_stage.sh、fixed_driver.py:5 那几条 want 与「16 处」找不到；
#   quota-host-count-unchecked（不认写死的台数）：hardcoded-quota-red 里 fixed_hosts.sh、fixed_driver.py:6、fixed_driver.py:16 那几条 want 与「16 处」找不到；
#   quota-host-computed-unseen（台数变量另有现算的赋值也照判它的整数初值）：hardcoded-quota-green 判红（computed_run.sh 的 peer_count=2 被点名）；
#   quota-auto-allowed（不把 auto 当写死的上限）：hardcoded-quota-red 里 auto_cap.sh 那两条 want 与「16 处」找不到；
#   quota-exempt-unchecked（排除表不核文件在不在、锚串命不命中）：hardcoded-quota-red 里「指向不存在的文件」「一处被判行都不命中」两条 want 找不到；
#   quota-exempt-growth-unchecked（排除表不比基准）：hardcoded-quota-red 里「排除表涨了」那条 want 找不到；
#   quota-exempt-family-unrecognized（排除表比基准时不认拆分族：拆出来的新路径都当族外）：hardcoded-quota-exempt-family-green 期望退出 0、实测 1（「排除表涨了」）。
cell_hardcoded_quota() {
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
exempt_table_baseline .claude/gate.d/hardcoded-quota-exempt.tsv "$base_table"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" \
  "$EXEMPT_ROW_GROWTH_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
exempt_growth = types.ModuleType("lib_exempt_row_growth")
exec(compile(sys.argv[7], "exempt-row-growth-library", "exec"), exempt_growth.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken, base_state, base_name, base_table = sys.argv[3:7]
split_family_library_file = sys.argv[8]

EXEMPT_TABLE = ".claude/gate.d/hardcoded-quota-exempt.tsv"
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python", ".rs": "rust"}
SIZE = r"[0-9]+[KMGT]" if broken == "quota-auto-allowed" else r"(?:[0-9]+[KMGT]|auto)"      # systemd 的 MemoryMax 写法与 auto：包装不认 auto（退 251），写 auto 也是脚本自己定上限
MEMORY_NAME = r"(?i:\b[A-Za-z0-9_]*(?:MEMORY_CAP|MEMORY_MAX|MEMORY_LIMIT|MEM_CAP)\b)"      # 名字以这几样收尾：REPLAY_MEMORY_CAP、GATE_VERUS_MEMORY_MAX、memory_cap；RUN_WITH_MEMORY_CAP_RESERVE 这类包装自己的设置不算
HOST_NAME = r"(?i:\b[A-Za-z0-9_]*(?:HOSTS|HOST_COUNT|MACHINE_COUNT|PEER_COUNT)\b)"
HOST_COUNT_LITERAL = r"(?:[2-9]|[1-9][0-9]+)"      # 两台以上：写 0、1 的是开关或「只在本机」，不算写死台数
KEPT_LITERAL = re.compile(r".*run-with-memory-cap\.sh|--memory-cap(?:=" + SIZE + r")?|--hosts(?:=[0-9]+)?|" + SIZE + r"|[0-9]+|" + MEMORY_NAME + "|" + HOST_NAME)
PREFILTER = re.compile(r"(?i)memory-cap|memory_cap|memory_max|memory_limit|mem_cap|hosts|host_count|machine_count|peer_count")
WRAPPER_FORM = "run-with-memory-cap.sh <常量>"
OPTION_FORM = "--memory-cap <常量>"
MEMORY_VARIABLE_FORM = "写死的内存上限默认"
HOST_FORM = "写死的台数"
# 跟在包装或选项后面的常量：包装或选项不在引号里，或它所在的那个字面量就在它收尾（"…/run-with-memory-cap.sh" 16G、python 的列表写法 "run-with-memory-cap.sh", "16G"），
# 或整个字面量就是「选项=常量」；引号里整句拿来当样本、当出路文字的不算
WRAPPER = re.compile(r"run-with-memory-cap\.sh(?P<close>[\"']?)(?:[ \t]*,[ \t]*[\"']?|[ \t]+)" + SIZE + r"(?![\w.])")
OPTION = re.compile(r"--memory-cap(?P<close>[\"']?)(?:[ \t]*,[ \t]*[\"']?|[ \t]+|=)" + SIZE + r"(?P<tail>[\"']?)(?![\w.])")
OPTION_DEFAULT = re.compile(r"[\"']--memory-cap[\"'][^\n]*?\bdefault\s*=\s*[\"']?" + SIZE + r"(?![\w.])")
HOST_OPTION = re.compile(r"--hosts(?P<close>[\"']?)(?:[ \t]*,[ \t]*[\"']?|[ \t]+|=)" + HOST_COUNT_LITERAL + r"(?P<tail>[\"']?)(?![\w.])")
HOST_OPTION_DEFAULT = re.compile(r"[\"']--hosts[\"'][^\n]*?\bdefault\s*=\s*" + HOST_COUNT_LITERAL + r"(?![\w.])")
SHELL_DEFAULTED = r"\$\{(?P<name>{name}):?[-=]{value}\}"
SHELL_MEMORY_DEFAULTED = re.compile(SHELL_DEFAULTED.replace("{name}", MEMORY_NAME).replace("{value}", SIZE))
SHELL_HOST_DEFAULTED = re.compile(SHELL_DEFAULTED.replace("{name}", HOST_NAME).replace("{value}", HOST_COUNT_LITERAL))
# shell 的赋值（行首、空白或 ; & | ( 之后，可带 export / local / declare / readonly，环境前缀 X=16G 命令 也算）：右边取到下一个不在引号与 $( ) 里的 ; & | ) 或空白为止
SHELL_ASSIGNMENT = re.compile(r"(?:^|(?<=[\s;&|(]))(?:(?:export|declare|local|readonly)[ \t]+(?:-[a-zA-Z]+[ \t]+)?)?(?P<name>[A-Za-z_][A-Za-z0-9_]*)="
                              r"(?P<rhs>(?:\"(?:[^\"\\]|\\.)*\"|'[^']*'|\$\((?:[^()]|\([^()]*\))*\)|[^;&|\"'\s()])*)")
# python / rust 的赋值：NAME = 值、NAME: 类型 = 值、let [mut] NAME = 值、const NAME: 类型 = 值；右边取到行尾
CODE_ASSIGNMENT = re.compile(r"(?:\blet[ \t]+(?:mut[ \t]+)?|\b(?:const|static)[ \t]+)?\b(?P<name>[A-Za-z_][A-Za-z0-9_]*)[ \t]*(?::[ \t]*[^=\n]+?)?(?<![=!<>+\-*/%&|^])=(?!=)[ \t]*(?P<rhs>[^\n]*)")
QUOTED_SIZE = re.compile(r"[\"']" + SIZE + r"[\"']")
QUOTED_HOST_KEY = re.compile(r"[\"']" + HOST_NAME + r"[\"']\s*:\s*" + HOST_COUNT_LITERAL + r"(?![\w.])")
QUOTED_MEMORY_NAME = re.compile(r"[\"']" + MEMORY_NAME + r"[\"']")
BARE_MEMORY_NAME = re.compile(MEMORY_NAME)
INTEGER_LITERAL = re.compile(r"[\"']?[0-9]+[\"']?")
EXEMPT_ROW = re.compile(r"(?P<path>[^\s:#]+):(?P<anchor>\S(?:.*?\S)?)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
strings_scanned = broken == "quota-strings-scanned"


def standalone(code, found, mask):
    """跟在包装或选项后面的常量算不算写在跑法里：包装或选项不在引号里，或它所在的字面量就在它收尾，或整个字面量就是「选项=常量」。"""
    if broken == "quota-sentence-samples-counted":
        return True
    start = found.start()
    if not mask[start] or found.group("close"):
        return True
    tail = found.groupdict().get("tail")
    return bool(tail) and start > 0 and code[start - 1] in "\"'"


def computed_host_names(language, code):
    """这份文件里有哪一处赋值的右边不是整数字面量的台数变量（hosts=$(host_count) 这类现算的）：它在别处给的整数只是没算之前的初值，不判。"""
    if broken == "quota-host-computed-unseen":
        return set()
    pattern = SHELL_ASSIGNMENT if language == "shell" else CODE_ASSIGNMENT
    names = set()
    for found in pattern.finditer(code):
        if re.fullmatch(HOST_NAME, found.group("name")) and not INTEGER_LITERAL.fullmatch(found.group("rhs").strip().rstrip(";,")):
            names.add(found.group("name"))
    return names


def forms_of(language, code, computed):
    names = []
    mask = source_lines.quoted_mask(code)
    if any(standalone(code, found, mask) for found in WRAPPER.finditer(code)):
        names.append(WRAPPER_FORM)
    if broken != "quota-memory-cap-option-unchecked" and (any(standalone(code, found, mask) for found in OPTION.finditer(code)) or OPTION_DEFAULT.search(code)):
        names.append(OPTION_FORM)
    if broken != "quota-memory-variable-unchecked":
        if language == "shell":
            assigned = any(re.fullmatch(MEMORY_NAME, found.group("name")) and not mask[found.start("name")]
                           and re.fullmatch(r"[\"']?" + SIZE + r"[\"']?", found.group("rhs")) for found in SHELL_ASSIGNMENT.finditer(code))
            if assigned or SHELL_MEMORY_DEFAULTED.search(code):
                names.append(MEMORY_VARIABLE_FORM)
        elif QUOTED_SIZE.search(code) and (QUOTED_MEMORY_NAME.search(code) or BARE_MEMORY_NAME.search(re.sub(r"[\"'][^\"']*[\"']", " ", code))):
            names.append(MEMORY_VARIABLE_FORM)
    if broken != "quota-host-count-unchecked":
        if language == "shell":
            assigned = any(re.fullmatch(HOST_NAME, found.group("name")) and found.group("name") not in computed and not mask[found.start("name")]
                           and re.fullmatch(r"[\"']?" + HOST_COUNT_LITERAL + r"[\"']?", found.group("rhs")) for found in SHELL_ASSIGNMENT.finditer(code))
            defaulted = any(found.group("name") not in computed for found in SHELL_HOST_DEFAULTED.finditer(code))
        else:
            assigned = any(re.fullmatch(HOST_NAME, found.group("name")) and found.group("name") not in computed
                           and re.match(HOST_COUNT_LITERAL + r"(?![\w.])[ \t]*(?:[;,)]|$)", found.group("rhs").strip()) for found in CODE_ASSIGNMENT.finditer(code))
            defaulted = QUOTED_HOST_KEY.search(code) is not None
        if assigned or defaulted or any(standalone(code, found, mask) for found in HOST_OPTION.finditer(code)) or HOST_OPTION_DEFAULT.search(code):
            names.append(HOST_FORM)
    return names


def code_lines_of(language, text):
    if language == "shell":
        return source_lines.shell_code_lines(text, OUTPUT_TEXT)
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, KEPT_LITERAL, strings_scanned)
    return source_lines.rust_code_lines(text, KEPT_LITERAL, strings_scanned)


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX:
                    yield directory, os.path.join(current, name), LANGUAGE_OF_SUFFIX[os.path.splitext(name)[1]]


problems = []
hits = {}                      # 路径 → [(行号, 种类, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if not PREFILTER.search(text):
        continue
    original = text.split("\n")
    code_lines = code_lines_of(language, text)
    computed = computed_host_names(language, "\n".join(code_lines))
    for number, code in enumerate(code_lines, 1):
        if not code.strip():
            continue
        names = forms_of(language, code, computed)
        if names:
            hits.setdefault(path, []).append((number, names, original[number - 1] if number <= len(original) else code))

exempt_rows, exempt_problems = [], []
if os.path.isfile(EXEMPT_TABLE):
    for number, line in enumerate(open(EXEMPT_TABLE, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = EXEMPT_ROW.fullmatch(line)
        if not found or len(found.group("reason")) < 4:
            exempt_problems.append(f"第 {number} 行不是「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）：{line[:120]}")
            continue
        exempt_rows.append((number, found.group("path"), found.group("anchor")))
exempted = 0
unexempted = []
for path, entries in sorted(hits.items()):
    anchors = [anchor for _number, row_path, anchor in exempt_rows if row_path == path]
    for number, names, line in entries:
        if any(anchor in line for anchor in anchors):
            exempted += 1
        else:
            unexempted.append((path, number, names, line))
if broken != "quota-exempt-unchecked":
    for number, path, anchor in exempt_rows:
        if not os.path.isfile(path):
            exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
        elif not any(anchor in line for _line_number, _names, line in hits.get(path, [])):
            exempt_problems.append(f"第 {number} 行的锚串「{anchor}」在 {path} 里一处被判行都不命中（那一处已经改掉，或锚串抄错了）")

grown, growth_note = [], ""
if base_state == "table" and broken != "quota-exempt-growth-unchecked":
    grown = exempt_growth.grown_exempt_rows(exempt_rows, base_table, EXEMPT_ROW, split_family_library_file, broken == "quota-exempt-family-unrecognized")
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

if scanned == 0 and not os.path.isfile(EXEMPT_TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有 *.sh、*.py、*.rs，也没有 {EXEMPT_TABLE}")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if unexempted:
    failed = True
    print(f"  ✗ {len(unexempted)} 处在脚本里自己定了内存上限或台数（上限与机器归 task-server）：")  # gate-lint:summary
    for path, number, names, line in unexempted:
        print(f"     {path}:{number}  {'、'.join(names)}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：删掉脚本里定的上限与台数，整条经 task-server/task submit -- <命令> 交：服务按记录给上限、撞了放大重排，机器与分片也归它（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03）。")
    print(f"               确实不是跑法（自证造的假值、写给人照抄的文档例子）的，登记进 {EXEMPT_TABLE}：一行「<路径>:<锚串>  # 理由」，锚串取那一行里的一段原文；排除只缩不涨。")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，那一处已经删掉（上限交给 task-server）就删掉这一行，锚串取被判那一行里的一段原文；写法见上一条出路。")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")  # gate-lint:summary
    for number, path, anchor in grown:
        print(f"     第 {number} 行 {path}:{anchor}")  # gate-lint:detail
    print("     → 怎么办：排除只缩不涨。新写的跑法不定上限与台数，整条经 task-server/task submit 交；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 内存上限与台数都不写死：扫了 {scanned} 份（{by_directory}），可疑写法 {exempted} 处全在排除表里（表 {len(exempt_rows)} 行，每行都命中）；{growth_note}")
PY
}

# ── 格 memory-bounded-parallelism：按内存上限定开几个进程或线程的入口都登记在 .claude/memory-bounded-parallelism.tsv（除了按内存限并发的那几样，
#   别的一律照每台默认重活线程数开；记录在 records/2026-11-05-删轻型并入重型.md「并包」）
#
# 登记表 .claude/memory-bounded-parallelism.tsv（写法在它的文件头）：一行一个入口，制表符分隔两列（入口、理由），空行与 # 开头的行不算；没有程序读表里的内容，这一格只判形状。正方向逐行判：
#   ① 两列都在、都不空（多一列、少一列都判红，旧的六列写法同样判红）；② 入口在：仓根下的相对路径要是一份文件，cargo:<包>:<目标> 要在 crates/*/Cargo.toml 里找到包名是 <包> 的那一份、
#      包里有叫 <目标> 的目标（src/bin/<目标>.rs、src/bin/<目标>/main.rs、tests/<目标>.rs、tests/<目标>/main.rs、examples/<目标>.rs，或 Cargo.toml 的 [[bin]]、[[test]]、[[example]] 里 name 对上的）；
#      同一个入口登记两行也判红；③ 理由至少 REASON_MINIMUM_CHARACTERS 个字（写明为什么不能照每台默认重活线程数同时开）。表不在按空表判。
# 反方向（代码里按内存定并发的写法，所在入口要登记）：扫被判的根下 scripts/、.claude/gate.d/、.claude/hooks/、crates/ 的 *.sh、*.py、*.rs（与 heavy-thread-defaults 同一个范围），
#   跳过 target/、__pycache__/、.git/ 与 .claude/gate.d/fixtures/；只看代码：逐行读法在共用库 SOURCE_CODE_LINES_LIBRARY（shell 去掉注释与出路文字，python 经 tokenize、
#   rust 按引号去掉注释与字符串字面量），留下的字面量只有内容整个是以 memory.max 收尾的路径（读 cgroup 的内存上限）的那一种；之后引号里的一律不认（字面量里有 memory.max 的记成读内存上限）。
#   标识符按 _ 与大小写切成词（小写），一行算「按内存定并发的写法」的两种：
#     按内存定并发的函数：有一个标识符的词含 memory 与 bound（memory_bound_for 这类：内存上限 ÷ 每件峰值定开几个），或含 lease、host、memory（借主机内存额度排队）；
#     内存上限与并发数同在一行：有内存上限的量——字面量里有 memory.max，或标识符的词含 memory 与 cap / max / limit / budget / total / bound 之一、
#       含 host 与 memory、含 gibibytes、含 budget 与 bytes、含 free 与 bytes（以 RUN_WITH_MEMORY_CAP 起头的是内存包装自己的设置，不算）——
#       又有并发数的量——标识符的词含 threads、thread、workers、worker、jobs、lanes、lane、walkers、walker、probes、copies、parallel、concurrency、tasks、lease、leased 之一
#       （同一个标识符的词里还有 arguments、argument、text、name、names、variable、label、option、options 的不算：那是参数、名字或说明，不是个数），或这一行给 count 赋值。
#     行里有标识符的词含 gpu、gpus、card、cards、device、devices、vram、scratch 的整行不算：那是按显存排卡上的活，不是按主机内存开进程或线程。
#   入口罩着哪些文件：入口那份文件（cargo:<包>:<目标> 取目标的源文件，目录式测试目标 tests/<目标>/main.rs 与目录式二进制目标 src/bin/<目标>/main.rs 取 main.rs 连同它 mod 声明到的子模块文件，
#     认法在 scripts/gate/lib_test_target_sources.py；写成路径的入口取它的拆分族：入口自己加按拆法命名的部分文件，门禁阶段连 .claude/gate.d/lib/ 下拆出的库，认法在 scripts/gate/lib_lag_split_family.py 的 family_members），
#   加上它们不以 # 或 // 起头的行里点名的仓根相对路径（scripts/、.claude/gate.d/、.claude/hooks/、crates/ 下的 *.sh、*.py、*.rs，文件在才算；点名的文件同样连它的拆分族），
#   往下再跟一层（共两层：门禁 → 它起的判法模块，分片驱动 → 门禁 → 判法模块）。
#   有这类写法的文件不在任何一个登记入口罩着的文件里，判红，点名文件:行与种类（每份文件列前 3 行）；登记了的入口罩着的文件里一处这类写法都没有，也判红。
# 判不了的：变量里拼出来的路径、按 import / mod / use 引的模块（库文件要被入口罩着，入口的代码行里要写出它的路径，或把库文件本身登成入口）、
#   引号里拼的算式、跨行的写法（内存上限与并发数分在两行）、名字里不带上面那些词的写法（只叫 cap、limit 的）；只读内存上限来显示或记录、不拿它定并发的（内存包装、峰值表）不在这一格判。
#   理由成不成立靠 review；任务服务不读这张表。
# 样本：memory-bounded-parallelism-green 的表六行：.claude/gate.d/sample-stage.sh（代码行点名 scripts/mutation/sample_rows.py，那份里有 memory_bound_for；
#   同目录的 sample_rows_pool.py 是它的 Python 拆法部分文件，里面的 memory_bound_for_pool 随点名的 sample_rows.py 的族一起罩着）、
#   scripts/experiment-apparatus/budget_run.sh（walker 数 = (gibibytes − 8) ÷ 4）、cargo:demo:demo_device（crates/demo/src/bin/demo_device.rs 里 free_threads 与 free_bytes 同一行）、
#   cargo:demo:demo_split_run（目录式测试目标：crates/demo/tests/demo_split_run/main.rs 只声明子模块 part，part.rs 里 workers 与 free_gibibytes 同一行）、
#   cargo:demo:demo_split_bin（目录式二进制目标：crates/demo/src/bin/demo_split_bin/main.rs 只声明子模块 part，part.rs 里同样一行）、
#   scripts/hosts/split_gate.sh（入口里没有这类写法，钩子拆法的部分文件 split_gate-memory.sh 里有 gibibytes 算的 count）；
#   不算的几样：scripts/hosts/wrapper_peak.sh 只读 memory.max 来记录、RUN_WITH_MEMORY_CAP_KEY 与线程变量同一行，crates/demo/src/card_queue.rs 按显存排卡（lanes 与 gpu_memory_total 同一行），
#   注释、echo 与文档串里写的 memory_bound；判绿并报表 6 行、按内存定并发的写法 7 处在 7 份文件里。
#   memory-bounded-parallelism-red：表里三列的一行、旧的六列写法的一行、入口文件不在的一行、cargo:demo:missing_target、同一个入口第二次登记、理由只有几个字，
#   入口在而罩着的文件里没有这类写法（scripts/hosts/plain_run.sh），各点名一处；没登记的 scripts/experiment-apparatus/unregistered_run.sh（probe 数 = gibibytes ÷ 5）、
#   crates/demo/src/lease.rs（借主机内存额度排队）与 scripts/experiment-apparatus/registered_run_extra.sh（名字像是登记入口 registered_run.sh 的部分文件、其实不是拆法命名的族外文件）各点名。
# 弄坏开关（CODE_TOOLING_BREAK）：memory-table-columns-unchecked（不核两列）：memory-bounded-parallelism-red 里「第 3 行不是两列」「第 4 行不是两列」两条 want 找不到；
#   memory-table-entry-unchecked（不核入口在不在、登没登两次）：「入口 scripts/experiment-apparatus/gone_run.sh：仓根下没有这份文件」「入口 cargo:demo:missing_target」「第 2 行已经登记过」三条 want 找不到；
#   memory-table-reason-unchecked（不核理由够不够长）：「理由「样本：太短」」那条 want 找不到；
#   memory-bounded-unregistered-accepted（代码里按内存定并发的写法不核入口登没登记）：unregistered_run.sh、lease.rs、registered_run_extra.sh 三条 want 与「3 份文件」找不到；
#   memory-bounded-stale-accepted（登记的入口不核代码里还有没有这类写法）：「scripts/hosts/plain_run.sh」那条 want 找不到；
#   memory-bounded-prefilter-memory-only（预筛只认 memory）：green 里 budget_run.sh（gibibytes）、demo_device.rs（free_bytes）两处找不到、red 里 unregistered_run.sh 那条 want 找不到；
#   memory-bounded-family-unrecognized（写成路径的入口与代码行里点名的文件只认自己、不认拆分族）：green 判红（split_gate-memory.sh 与 sample_rows_pool.py 的写法没人罩着、split_gate.sh 那一行成了没有这类写法的入口）；
#   memory-table-bin-directory-target-main-only（目录式二进制目标只收 main.rs）：green 判红（demo_split_bin 的 part.rs 没人罩着、cargo:demo:demo_split_bin 那一行成了没有这类写法的入口）。
# 扫描：逐份文件按每台默认重活线程数并行（map_in_processes，fork 的子进程，结果按文件次序合并）；预筛是判红的必要条件：site_form 交非空只有两条路（词集合含 DECISIVE_WORD_SETS 的一组，
#   或含 MEMORY_WORD_SETS 的一组再加并发词），每一组都含 memory、bytes（gibibytes 以它收尾）之一，引号里的 memory.max 也含 memory，所以文件或一行代码小写之后既没有 memory 也没有 bytes 就不可能交非空，
#   不逐行拆标识符的词。
cell_memory_bounded_parallelism() {
local jobs
if ! jobs="$(heavy_thread_default)"; then
  echo "  ✗ 取不了每台默认重活线程数（原因在上一行），逐份扫描的并行度定不了，一份都没扫"
  echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
  exit 1
fi
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$PARALLEL_MAP_LIBRARY" "$jobs" "$STAGE_REPOSITORY/scripts/gate/lib_test_target_sources.py" \
  "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import collections, importlib.util, os, re, sys, tomllib, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
parallel = types.ModuleType("lib_parallel_map")
exec(compile(sys.argv[4], "parallel-map-library", "exec"), parallel.__dict__)
target_sources = types.ModuleType("lib_test_target_sources")
exec(compile(open(sys.argv[6], encoding="utf-8").read(), sys.argv[6], "exec"), target_sources.__dict__)
split_family_library_file = sys.argv[7]
specification = importlib.util.spec_from_file_location("lib_lag_split_family", split_family_library_file)
split_family = importlib.util.module_from_spec(specification)
try:
    specification.loader.exec_module(split_family)
except (OSError, ImportError, SyntaxError) as error:
    print(f"  ✗ 载不进 {split_family_library_file}：{error}")
    print("     → 怎么办：它随仓走（scripts/gate/lib_lag_split_family.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；拆分族的认法只在那一份里写。")
    sys.exit(1)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken = sys.argv[3]
jobs = int(sys.argv[5])

TABLE = ".claude/memory-bounded-parallelism.tsv"
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks", "crates")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/",)
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python", ".rs": "rust"}
KEPT_LITERAL = re.compile(r"[^\s\"']*memory\.max")      # 以 memory.max 收尾的路径字面量：读 cgroup 的内存上限
QUOTED = re.compile(r"\"(?:[^\"\\\n]|\\.)*\"|'(?:[^'\\\n]|\\.)*'")
IDENTIFIER = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")
MEMORY_MAX_MARK = "memory_max_literal"
PARALLEL_WORDS = {"threads", "thread", "workers", "worker", "jobs", "lanes", "lane", "walkers", "walker", "probes", "copies", "parallel", "concurrency", "tasks", "lease", "leased"}
NOT_A_COUNT_WORDS = {"arguments", "argument", "text", "name", "names", "variable", "label", "option", "options"}
DISPLAY_MEMORY_WORDS = {"gpu", "gpus", "card", "cards", "device", "devices", "vram", "scratch"}
MEMORY_WORD_SETS = tuple(map(frozenset, ({"memory", "cap"}, {"memory", "max"}, {"memory", "limit"}, {"memory", "budget"}, {"memory", "total"}, {"memory", "bound"},
                                         {"host", "memory"}, {"gibibytes"}, {"budget", "bytes"}, {"free", "bytes"})))
DECISIVE_WORD_SETS = tuple(map(frozenset, ({"memory", "bound"}, {"lease", "host", "memory"})))
COUNT_ASSIGNMENT = re.compile(r"^\s*(?:local\s+|let\s+(?:mut\s+)?)?count\s*=(?!=)")
TABLE_COLUMNS = 2           # 入口、理由
REASON_MINIMUM_CHARACTERS = 8
REFERENCE = re.compile(r"(?<![A-Za-z0-9_.\-])(?:scripts|\.claude/gate\.d|\.claude/hooks|crates)/[A-Za-z0-9_./\-]*?[A-Za-z0-9_\-]\.(?:sh|py|rs)\b")
CLOSURE_DEPTH = 2
LINES_SHOWN = 3
FUNCTION_FORM = "按内存定并发的函数"
SAME_LINE_FORM = "内存上限与并发数同在一行"

def words_of(name):
    return {word.lower() for word in re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name).split("_") if word}


def site_form(code):
    """这一行代码算不算按内存定并发的写法：交种类或 None（判法在格函数上方的注释）。"""
    marked = QUOTED.sub(lambda found: f" {MEMORY_MAX_MARK} " if "memory.max" in found.group(0) else " ", code)
    names = [(name, words_of(name)) for name in IDENTIFIER.findall(marked)]
    if any(words & DISPLAY_MEMORY_WORDS for _name, words in names):
        return None
    if any(word_set <= words for _name, words in names for word_set in DECISIVE_WORD_SETS):
        return FUNCTION_FORM
    memory = any(name == MEMORY_MAX_MARK or (not name.upper().startswith("RUN_WITH_MEMORY_CAP") and any(word_set <= words for word_set in MEMORY_WORD_SETS))
                 for name, words in names)
    parallel = COUNT_ASSIGNMENT.search(marked) is not None or any(words & PARALLEL_WORDS and not words & NOT_A_COUNT_WORDS for _name, words in names)
    return SAME_LINE_FORM if memory and parallel else None


def code_lines_of(language, text):
    if language == "shell":
        return source_lines.shell_code_lines(text, OUTPUT_TEXT)
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, KEPT_LITERAL)
    return source_lines.rust_code_lines(text, KEPT_LITERAL)

def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX:
                    yield directory, os.path.join(current, name), LANGUAGE_OF_SUFFIX[os.path.splitext(name)[1]]


SCANNED_FILES = list(scanned_files())
SCANNED_PATHS = [os.path.normpath(path) for _directory, path, _language in SCANNED_FILES]
family_cache = {}


def family_of(path):
    """path 的拆分族里被扫的文件，path 自己排头：入口加按它的拆法命名的部分文件（scripts/gate/lib_lag_split_family.py 的 family_members；门禁阶段连 .claude/gate.d/lib/ 下拆出的库）。
    弄坏开关 memory-bounded-family-unrecognized 下只认 path 自己。"""
    path = os.path.normpath(path)
    if path not in family_cache:
        members = [] if broken == "memory-bounded-family-unrecognized" else split_family.family_members(path, SCANNED_PATHS)
        family_cache[path] = [path] + [member for member in members if member != path]
    return family_cache[path]


def package_directories():
    """crates/*/Cargo.toml 里 [package] 的 name → 那个包的目录；读不了的清单不算。"""
    found = {}
    if not os.path.isdir("crates"):
        return found
    for name in sorted(os.listdir("crates")):
        manifest = os.path.join("crates", name, "Cargo.toml")
        try:
            with open(manifest, "rb") as handle:
                package = tomllib.load(handle).get("package", {}).get("name")
        except (OSError, tomllib.TOMLDecodeError):
            continue
        if package:
            found[package] = os.path.join("crates", name)
    return found


def target_source(directory, target):
    """包目录 directory 里叫 target 的目标的源文件清单（目录式测试目标 tests/<目标>/main.rs 与目录式二进制目标 src/bin/<目标>/main.rs 是 main.rs 连同它 mod 声明到的子模块文件，别的一份文件）；没有交空表。
    弄坏开关 memory-table-directory-target-main-only 下目录式测试目标只收 main.rs，memory-table-bin-directory-target-main-only 下目录式二进制目标只收 main.rs。"""
    binaries = target_sources.binary_target_source_files(directory, target, follow_submodules=broken != "memory-table-bin-directory-target-main-only")
    tests = target_sources.target_source_files(os.path.join(directory, "tests"), target, follow_submodules=broken != "memory-table-directory-target-main-only")
    for files in (binaries, tests, [os.path.join(directory, f"examples/{target}.rs")]):
        if files and os.path.isfile(files[0]):
            return files
    try:
        with open(os.path.join(directory, "Cargo.toml"), "rb") as handle:
            manifest = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError):
        return []
    for kind in ("bin", "test", "example"):
        for declared in manifest.get(kind, []):
            if declared.get("name") == target and declared.get("path") and os.path.isfile(os.path.join(directory, declared["path"])):
                return [os.path.join(directory, declared["path"])]
    return []

PACKAGES = package_directories()

def entry_files(entry):
    """入口那份文件：([文件], "") 或 ([], 为什么找不到)。"""
    if entry.startswith("cargo:"):
        parts = entry.split(":")
        if len(parts) != 3 or not parts[1] or not parts[2]:
            return [], "要写成 cargo:<包>:<目标名>"
        directory = PACKAGES.get(parts[1])
        if directory is None:
            return [], f"crates/ 下没有 [package] name 是 {parts[1]} 的 Cargo.toml"
        sources = target_source(directory, parts[2])
        if not sources:
            return [], (f"包 {parts[1]}（{directory}）里没有叫 {parts[2]} 的目标（src/bin/{parts[2]}.rs、src/bin/{parts[2]}/main.rs、tests/{parts[2]}.rs、tests/{parts[2]}/main.rs、examples/{parts[2]}.rs "
                        "与 Cargo.toml 的 [[bin]]、[[test]]、[[example]] 都没对上）")
        return sources, ""
    if entry.startswith("/") or ".." in entry.split("/"):
        return [], "要写仓根下的相对路径"
    if not os.path.isfile(entry):
        return [], "仓根下没有这份文件"
    return family_of(entry), ""

def references_of(path):
    """path 不以 # 或 // 起头的行里点名、此刻在的仓根相对路径（scripts/、.claude/gate.d/、.claude/hooks/、crates/ 下的 *.sh、*.py、*.rs）。"""
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return set()
    code = "\n".join(line for line in text.split("\n") if not line.lstrip().startswith(("#", "//")))
    return {os.path.normpath(found) for found in REFERENCE.findall(code) if os.path.isfile(found)}

def closure_of(files):
    reached, frontier = set(files), set(files)
    for _depth in range(CLOSURE_DEPTH):
        frontier = {member for path in frontier for found in references_of(path) for member in family_of(found)} - reached
        reached |= frontier
    return reached

problems, table_problems = [], []
rows, seen = [], {}
if os.path.isfile(TABLE):
    for number, line in enumerate(open(TABLE, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        columns = line.split("\t")
        if len(columns) != TABLE_COLUMNS or any(not column.strip() for column in columns):
            if broken != "memory-table-columns-unchecked":
                table_problems.append(f"第 {number} 行不是两列（入口、理由，制表符分隔、每列都不空）：分出 {len(columns)} 列「{line[:120]}」")
                continue
            columns = (columns + [""] * TABLE_COLUMNS)[:TABLE_COLUMNS]
        entry, reason = (column.strip() for column in columns)
        files, entry_problem = entry_files(entry)
        if broken != "memory-table-entry-unchecked":
            if entry_problem:
                table_problems.append(f"第 {number} 行的入口 {entry}：{entry_problem}")
            elif entry in seen:
                table_problems.append(f"第 {number} 行的入口 {entry} 第 {seen[entry]} 行已经登记过（一个入口一行）")
        seen.setdefault(entry, number)
        if broken != "memory-table-reason-unchecked" and len(reason) < REASON_MINIMUM_CHARACTERS:
            table_problems.append(f"第 {number} 行的理由「{reason}」太短（至少 {REASON_MINIMUM_CHARACTERS} 个字，写明为什么不能照每台默认重活线程数同时开）")
        rows.append((number, entry, files))

sites = {}                      # 路径 → [(行号, 种类, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
# 预筛（必要条件）：site_form 交非空，只有两条路——词集合含 DECISIVE_WORD_SETS 的一组，或有 MEMORY_WORD_SETS 的一组再加并发词；两处每一组都含 memory、bytes（gibibytes 以它收尾）
#   之一，引号里的 memory.max 也含 memory；词是标识符小写之后拆出来的。所以文件或一行代码小写之后既没有 memory 也没有 bytes，就不可能交非空，不必逐行拆词。
#   弄坏开关 memory-bounded-prefilter-memory-only：预筛只认 memory，只带 bytes 的几处（gibibytes、free_bytes）被漏掉
PREFILTER_WORDS = ("memory",) if broken == "memory-bounded-prefilter-memory-only" else ("memory", "bytes")


def may_contain_site(text):
    lowered = text.lower()
    return any(word in lowered for word in PREFILTER_WORDS)


def scan_file(item):
    """一份文件 → (目录, 路径, 读不了的原因或 None, [(行号, 种类, 原行)])；各份文件互不依赖，在子进程里并行跑。"""
    directory, path, language = item
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        return directory, path, f"读不了 {path}：{error}", []
    if not may_contain_site(text):
        return directory, path, None, []
    original = text.split("\n")
    found = []
    for number, code in enumerate(code_lines_of(language, text), 1):
        if code.strip() and may_contain_site(code):
            form = site_form(code)
            if form:
                found.append((number, form, original[number - 1] if number <= len(original) else code))
    return directory, path, None, found


for directory, path, read_problem, found in parallel.map_in_processes(scan_file, SCANNED_FILES, jobs):
    if read_problem:
        problems.append(read_problem)
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if found:
        sites.setdefault(os.path.normpath(path), []).extend(found)

covered, stale = set(), []
for number, entry, files in rows:
    if not files:
        continue
    reached = sorted(path for path in closure_of(files) if path in sites)
    covered.update(reached)
    if not reached and broken != "memory-bounded-stale-accepted":
        stale.append((number, entry, files[0]))
unregistered = [] if broken == "memory-bounded-unregistered-accepted" else [path for path in sorted(sites) if path not in covered]

if scanned == 0 and not os.path.isfile(TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有 *.sh、*.py、*.rs，也没有 {TABLE}")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if table_problems:
    failed = True
    print(f"  ✗ 登记表 {TABLE} 有 {len(table_problems)} 处不对：")  # gate-lint:summary
    for problem in table_problems:
        print(f"     {problem}")  # gate-lint:detail
    print(f"     → 怎么办：照 {TABLE} 文件头的两列写法改那一行：入口写仓根下的相对路径或 cargo:<包>:<目标名>，理由写为什么不能照每台默认重活线程数同时开（不写峰值、任务号这类没人读的数，旧的六列写法不认）。")
if unregistered:
    failed = True
    print(f"  ✗ {len(unregistered)} 份文件里按内存上限定开几个进程或线程，所在入口没登记在 {TABLE}（没登记的照每台默认重活线程数开）：")  # gate-lint:summary
    for path in unregistered:
        entries = sites[path]
        for number, form, line in entries[:LINES_SHOWN]:
            print(f"     {path}:{number}  {form}  「{line.strip()[:160]}」")  # gate-lint:detail
        if len(entries) > LINES_SHOWN:
            print(f"     {path} 另有 {len(entries) - LINES_SHOWN} 行")  # gate-lint:detail
    print(f"     → 怎么办：确是每件占一大块内存、31 份同时开会撞上限的（用户 2026-11-05 定只有这一类按内存限并发），把入口登进 {TABLE}（入口、理由两列，写法在它文件头）；")
    print("               入口的代码行里写出库文件的路径，或把库文件本身登成入口。不是这一类的，改成照每台默认重活线程数开（.claude/heavy-thread-default），不按内存算份数、不借内存额度排队。")
if stale:
    failed = True
    print(f"  ✗ 登记表里 {len(stale)} 个入口罩着的文件里已经没有按内存定并发的写法：")  # gate-lint:summary
    for number, entry, source in stale:
        print(f"     第 {number} 行 {entry}（入口文件 {source}，连同它代码行里点名的文件往下 {CLOSURE_DEPTH} 层）")  # gate-lint:detail
    print("     → 怎么办：不再按内存限并发的，删掉那一行（照每台默认重活线程数开）；入口写错的改成真起那件活的脚本或 cargo:<包>:<目标名>，库文件的路径要写在入口的代码行里。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
site_count = sum(len(entries) for entries in sites.values())
table_note = f"表 {len(rows)} 行，每行两列成形、入口在" if os.path.isfile(TABLE) else f"{TABLE} 不在（按空表判）"
print(f"  ✓ 按内存上限定并发的入口都登记了：{table_note}；扫了 {scanned} 份（{by_directory}），按内存定并发的写法 {site_count} 处在 {len(sites)} 份文件里，都落在登记的入口罩着的文件里")
PY
}

# ── 格 shared-data-root-literals：scripts/、.claude/hooks/、.claude/gate.d/、.claude/scripts/、task-server/ 里经 ~ 或 $HOME 取数据根下共享的那几项
#   （records/2026-10-31-task-server巡检.md 问题 73：第二台的 ~ 是另一个目录，经它取的写读都落进第二台本地那一份，两台读不到同一份）
#
# 名单：这一道所在仓里 .claude/hooks/lib_work_root.py 的 SHARED_DATA_ENTRIES（共享的，判）与 PER_HOST_DATA_ENTRIES（按台的，不判），读不出就判红、出路是先换上那一版库。
#   运行记录另判（.claude/rules/verification.md 两节：运行记录、产物放共享根，不经 ~ 与 $HOME 取）：这一格里的 RUN_RECORD_ENTRIES 点名的几项（峰值表 memory-peaks.tsv、
#   崩溃放量的运行目录 crash-amplification）分台靠共享表里的「哪一台」列、不靠 ~，经 ~ 或 $HOME 取的一律照「运行记录按台取」判，不看 lib_work_root.py 把它登在哪张名单里。
#   数据根本身（~/.local/share/singlefs 后面不接名字，或接的是变量）与名单外的名字也判：取数据根只经 lib_work_root.py（欠账 C949（任务只在一台机器上跑得起）的静态那一半）。
# 扫哪些：被判的根下这五处的 *.sh、*.py；跳过 target/、__pycache__/、.git/、.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/（样本是造出来的输入），
#   跳过 .claude/hooks/lib_work_root.py 自己（数据根只在它里面定义）。
# 只看代码（放行规则）：逐行读法在共用库 SOURCE_CODE_LINES_LIBRARY。shell 去掉整行注释、行尾注释与出路文字（echo / printf / howto / bad / die / say 的字面量），
#   引号里带空白的那一段是句子（出路文字、样本命令），去掉；嵌在 shell 里的 python 的三引号整段带空白的同样去掉（文档串）。
#   python 经 tokenize 去掉注释与带空白的字符串字面量（文档串、出路文字），内容不带空白的字面量照代码判（路径就在这里头）；f-string 的字面段一概不判。
#   自证函数体（python 的 def …selftest…( 按缩进收尾，shell 的 …selftest…() { 收到同一缩进的 }）里造的样本不判。
# 算经 ~ 或 $HOME 取的写法（每处一行，两种起法）：
#   ~/、$HOME/、${HOME}/ 与 ${HOME:?}/ 这类带修饰的写法起头、后面跟 /.local/share/singlefs（后面不再接字母、数字、_ . -），再接 /<名字> 或不接；
#   expanduser("~") 后面紧接 ".local", "share", "singlefs"[, "<名字>"] 或 ".local/share/singlefs[/<名字>]"（os.path.join 的写法）。
#   按名字归类（一行里几处各归各的，种类去重后点名）：运行记录的名字（连同以它起头的 <名字>-<…>）点名「运行记录按台取」；共享的名字照起法点名；按台的名字不点名；
#   不接名字或接的是变量点名「数据根本身经 ~ 或 $HOME 取」；名单两半里都没有的名字点名「数据根下名单外的一项」。
# 排除表 .claude/gate.d/shared-data-root-literals-exempt.tsv：一行「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）；那份文件里含锚串的被判行不算。
#   写不对、指向不存在的文件、锚串一处被判行都不命中，都判红；只缩不涨：与改动范围的基准那一版比（exempt_table_baseline），基准里没有的「路径:锚串」判红；
#   基准里还没有这张表、不在 git 仓里，这一条不比、成功句里写明；git 取不到基准那一版判红。
# 判不了的：变量里拼出来的路径（home="$HOME"; "$home/.local/share/singlefs/work"）、f-string 里的、带空白的命令串里的（bash -c "… $HOME/.local/share/singlefs/work …"）、
#   Path.home() 拼的、XDG_DATA_HOME 下拼的、读别的文件定的；自证函数之外造的样本与认路径文字的代码（不取目录）逐行登记进排除表，理由成不成立靠 review。
# 样本：shared-data-root-literals-red 的 scripts/hosts/home_literal.sh（$HOME/、~/、${HOME:?}/、${HOME}/ 各一处）、scripts/gate/home_literal.py（~/ 起头的字面量、
#   expanduser("~") 的两种拼法）、.claude/hooks/home_hook.sh、.claude/gate.d/home_stage.sh、.claude/scripts/home_tool.sh、task-server/home_server.py 各一处，共 11 处逐处点名；
#   scripts/agents/text_reader.py 认路径文字的一行由排除表放行；排除表另一行指向不存在的文件、一行锚串不命中、setup.sh 建 git 仓提交一版不带 grown-row 那一行的表再加回来（表涨了），后三样都点名。
#   shared-data-root-literals-green：经 lib_work_root.py 取的、注释、echo、带空白的出路字面量、文档串、按台的 qemu 与 rocksdb-prebuilt、自证函数体里的样本、
#   样本自己的 .claude/hooks/lib_work_root.py、.claude/gate.d/fixtures/ 下的，都不点名；排除表一行放行认路径文字的那一行，判绿并报扫了 3 份、表 1 行每行都命中。
#   shared-data-root-literals-data-root-red：scripts/hosts/data_root.sh（数据根本身一处、名单外的名字一处、crash-amplification 与 memory-peaks.tsv 两处运行记录）、
#   scripts/gate/data_root.py（expanduser("~") 拼出来的数据根本身一处、以 crash-amplification- 起头的运行目录一处），共 6 处逐处点名；没有排除表。
# 弄坏开关（CODE_TOOLING_BREAK）：shared-literals-prose-scanned（带空白的字面量也照代码扫）：shared-data-root-literals-green 判红（出路字面量、文档串与 refuse 的参数被点名）；
#   shared-literals-join-form-unseen（不认 expanduser("~") 的拼法）：shared-data-root-literals-red 里 home_literal.py:4、:5 两条 want 与「11 处」找不到；
#   shared-literals-braced-home-unseen（不认 ${HOME}/ 与 ${HOME:?}/）：shared-data-root-literals-red 里 home_literal.sh:5、:6 两条 want 与「11 处」找不到；
#   shared-literals-per-host-judged（按台的也判）：shared-data-root-literals-green 判红（qemu 与 rocksdb-prebuilt 被点名）；
#   shared-literals-definition-file-judged（连定义那一份也判）：shared-data-root-literals-green 判红（样本的 lib_work_root.py 被点名）；
#   shared-literals-selftest-bodies-judged（连自证函数体也判）：shared-data-root-literals-green 判红（两份自证里造的样本被点名）；
#   shared-literals-exempt-unchecked（排除表不核文件在不在、锚串命不命中）：shared-data-root-literals-red 里「指向不存在的文件」「一处被判行都不命中」两条 want 找不到；
#   shared-literals-exempt-growth-unchecked（排除表不比基准）：shared-data-root-literals-red 里「排除表涨了」那条 want 找不到；
#   shared-literals-data-root-unseen（数据根本身与名单外的名字不判）：shared-data-root-literals-data-root-red 里那三条 want 与「6 处」找不到；
#   shared-literals-run-records-unseen（运行记录不判）：shared-data-root-literals-data-root-red 里那三条运行记录的 want 与「6 处」找不到。
#   shared-literals-join-form-unseen 另让 shared-data-root-literals-data-root-red 里 data_root.py:4 那条 want 与「6 处」找不到。
cell_shared_data_root_literals() {
base_table="$(mktemp)" || { echo "  ✗ 建不了放基准那一版排除表的临时文件"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格。"; exit 1; }
trap 'rm -f "${base_table:?}"' EXIT
exempt_table_baseline .claude/gate.d/shared-data-root-literals-exempt.tsv "$base_table"
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$base_state" "$base_name" "$base_table" "$STAGE_REPOSITORY/.claude/hooks/lib_work_root.py" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken, base_state, base_name, base_table, work_root_library = sys.argv[3:8]

EXEMPT_TABLE = ".claude/gate.d/shared-data-root-literals-exempt.tsv"
SCAN_DIRECTORIES = ("scripts", ".claude/hooks", ".claude/gate.d", ".claude/scripts", "task-server")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
DEFINITION_FILE = ".claude/hooks/lib_work_root.py"
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python"}
EXEMPT_ROW = re.compile(r"(?P<path>[^\s:#]+):(?P<anchor>\S(?:.*?\S)?)[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
WITHOUT_WHITESPACE = re.compile(r"\S*")      # python 的字面量：内容里没有空白的留着当代码判（路径），带空白的是句子（文档串、出路文字），去掉
TRIPLE_QUOTED = re.compile(r"(?s)(\"\"\"|''')(.*?)\1")
PYTHON_SELFTEST_DEF = re.compile(r"^(?P<indent>[ \t]*)(?:async[ \t]+)?def[ \t]+\w*self_?test\w*[ \t]*\(")
SHELL_SELFTEST_FUNCTION = re.compile(r"^(?P<indent>[ \t]*)(?:function[ \t]+)?[A-Za-z_]\w*self_?test\w*[ \t]*\([ \t]*\)[ \t]*\{?[ \t]*$")
prose_scanned = broken == "shared-literals-prose-scanned"

# 名单只在 .claude/hooks/lib_work_root.py 里（这一道所在仓里那一份）：共享的判，按台的不判
try:
    import importlib.util
    specification = importlib.util.spec_from_file_location("lib_work_root_for_shared_data_root_literals", work_root_library)
    work_roots = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(work_roots)
    shared_entries = tuple(work_roots.SHARED_DATA_ENTRIES)
    per_host_entries = tuple(work_roots.PER_HOST_DATA_ENTRIES)
except (OSError, ImportError, AttributeError) as error:
    print(f"  ✗ 读不出数据根的两张名单（{work_root_library} 的 SHARED_DATA_ENTRIES、PER_HOST_DATA_ENTRIES）：{error}")
    print("     → 怎么办：名单只在 .claude/hooks/lib_work_root.py 里定义；那份库还是没有这两张名单的那一版，就先经 scripts/gate/hook-swap.py 把它换上，再跑这一格。")
    sys.exit(1)
# 运行记录（.claude/rules/verification.md「判定库只有一份」：用时表、峰值表、占盘时序、上次红的清单，「所有任务都要多机可用」：运行记录与产物）放共享根；
#   分台靠共享表里的「哪一台」列，不靠各台自己的 ~：这几样经 ~ 或 $HOME 取就是按台取，不论 lib_work_root.py 把它登在哪张名单里（崩溃放量的运行目录已挪到工作根下）
RUN_RECORD_ENTRIES = ("memory-peaks.tsv", "crash-amplification")
data_root_judged = broken != "shared-literals-data-root-unseen"
run_records_judged = broken != "shared-literals-run-records-unseen"
HOME_PREFIX = r"(?:~|\$HOME\b)" if broken == "shared-literals-braced-home-unseen" else r"(?:~|\$HOME\b|\$\{HOME(?:[:?+=-][^}]*)?\})"
NAME = r"(?:/(?P<entry>[A-Za-z0-9_.-]*))?"
LITERAL_FORM = re.compile(r"(?<![\w$])" + HOME_PREFIX + r"/\.local/share/singlefs(?![A-Za-z0-9_.-])" + NAME)
QUOTE = r"[\"']"
EXPANDED_HOME = r"expanduser\(\s*" + QUOTE + r"~" + QUOTE + r"\s*\)\s*,\s*"
JOIN_FORMS = (re.compile(EXPANDED_HOME + QUOTE + r"\.local" + QUOTE + r"\s*,\s*" + QUOTE + r"share" + QUOTE + r"\s*,\s*" + QUOTE + r"singlefs" + QUOTE
                         + r"(?:\s*,\s*" + QUOTE + r"(?P<entry>[A-Za-z0-9_.-]*))?"),
              re.compile(EXPANDED_HOME + QUOTE + r"\.local/share/singlefs(?![A-Za-z0-9_.-])" + NAME))
LITERAL_NAME = "~/、$HOME/ 或 ${HOME}/ 起头的字面量"
JOIN_NAME = "expanduser(\"~\") 拼出来的"
DATA_ROOT_NAME = "数据根本身经 ~ 或 $HOME 取"
UNREGISTERED_NAME = "数据根下名单外的一项"
RUN_RECORD_NAME = "运行记录按台取"


def per_host_entry_of(entry):
    """名字是按台的那一项（连同以它起头的 <名字>-<…>）就交那一项，不是交 None。"""
    return next((name for name in per_host_entries if entry == name or entry.startswith(name + "-")), None)


def run_record_entry_of(entry):
    """名字是运行记录那一项（连同以它起头的 <名字>-<…>）就交那一项，不是交 None。"""
    return next((name for name in RUN_RECORD_ENTRIES if entry == name or entry.startswith(name + "-")), None)


def classified(entry, form_name):
    """一处命中按名字归类：运行记录不论登在哪张名单里都点名「运行记录按台取」；共享的照 form_name；按台的只在弄坏开关 per-host-judged 时判；
    没给名字是数据根本身；名单外的另记。不判交 None。"""
    if entry and run_record_entry_of(entry) is not None:
        return RUN_RECORD_NAME if run_records_judged else None
    if entry and entry in shared_entries:
        return form_name
    per_host = per_host_entry_of(entry) if entry else None
    if per_host is not None:
        return form_name if broken == "shared-literals-per-host-judged" else None
    if not data_root_judged:
        return None
    return DATA_ROOT_NAME if not entry else UNREGISTERED_NAME


def blank_regions(text, pattern, keep):
    """把 pattern 命中、keep 判为不留的那几段换成空格（换行照留，行号不变）。"""
    def replace(found):
        return found.group(0) if keep(found) else re.sub(r"[^\n]", " ", found.group(0))
    return pattern.sub(replace, text)


def without_prose_strings(line):
    """shell 的一行：引号里带空白的那一段是句子（出路文字、样本命令），换成空格；不带空白的照留（路径与赋值的右边）。"""
    if prose_scanned:
        return line
    mask = source_lines.quoted_mask(line)
    characters, start = list(line), None
    for index in range(len(line) + 1):
        inside = index < len(line) and mask[index]
        if inside and start is None:
            start = index
        elif not inside and start is not None:
            if any(character.isspace() for character in line[start:index]):
                characters[start:index] = " " * (index - start)
            start = None
    return "".join(characters)


def selftest_lines(original):
    """自证函数体里的行号（python 的 def …selftest…( 按缩进收尾，shell 的 …selftest…() { 收到同一缩进的 }）：里面造的是样本，不判。"""
    if broken == "shared-literals-selftest-bodies-judged":
        return set()
    inside, index = set(), 0
    while index < len(original):
        python_start, shell_start = PYTHON_SELFTEST_DEF.match(original[index]), SHELL_SELFTEST_FUNCTION.match(original[index])
        if python_start:
            indent = len(python_start.group("indent").expandtabs())
            end = index + 1
            while end < len(original) and (not original[end].strip() or len(original[end].expandtabs()) - len(original[end].expandtabs().lstrip()) > indent):
                end += 1
            inside.update(range(index + 1, end + 1))
            index = end
        elif shell_start:
            closing = re.compile(r"^" + re.escape(shell_start.group("indent")) + r"\}[ \t;]*(?:#.*)?$")
            end = index + 1
            while end < len(original) and not closing.match(original[end]):
                end += 1
            inside.update(range(index + 1, end + 2))
            index = end + 1
        else:
            index += 1
    return inside


def code_lines_of(language, text):
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, WITHOUT_WHITESPACE, prose_scanned)
    if not prose_scanned:   # shell 里嵌的 python 的文档串：三引号里带空白的整段是句子
        text = blank_regions(text, TRIPLE_QUOTED, lambda found: not any(character.isspace() for character in found.group(2)))
    return [without_prose_strings(line) for line in source_lines.shell_code_lines(text, OUTPUT_TEXT)]


def forms_of(code):
    names = []
    for found in LITERAL_FORM.finditer(code):
        name = classified(found.group("entry"), LITERAL_NAME)
        if name and name not in names:
            names.append(name)
    if broken != "shared-literals-join-form-unseen":
        for form in JOIN_FORMS:
            for found in form.finditer(code):
                name = classified(found.group("entry"), JOIN_NAME)
                if name and name not in names:
                    names.append(name)
    return names


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                    and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
            for name in sorted(files):
                path = os.path.join(current, name)
                if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX and (path != DEFINITION_FILE or broken == "shared-literals-definition-file-judged"):
                    yield directory, path, LANGUAGE_OF_SUFFIX[os.path.splitext(name)[1]]


problems = []
hits = {}                      # 路径 → [(行号, 种类, 原行)]
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if "share" not in text or "singlefs" not in text:
        continue
    original = text.split("\n")
    samples = selftest_lines(original)
    for number, code in enumerate(code_lines_of(language, text), 1):
        if number in samples or not code.strip():
            continue
        names = forms_of(code)
        if names:
            hits.setdefault(path, []).append((number, names, original[number - 1] if number <= len(original) else code))

exempt_rows, exempt_problems = [], []
if os.path.isfile(EXEMPT_TABLE):
    for number, line in enumerate(open(EXEMPT_TABLE, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = EXEMPT_ROW.fullmatch(line)
        if not found or len(found.group("reason")) < 4:
            exempt_problems.append(f"第 {number} 行不是「<路径>:<锚串>  # 理由」（锚串与 # 之间至少两个空格，理由至少 4 个字）：{line[:120]}")
            continue
        exempt_rows.append((number, found.group("path"), found.group("anchor")))
exempted = 0
unexempted = []
for path, entries in sorted(hits.items()):
    anchors = [anchor for _number, row_path, anchor in exempt_rows if row_path == path]
    for number, names, line in entries:
        if any(anchor in line for anchor in anchors):
            exempted += 1
        else:
            unexempted.append((path, number, names, line))
if broken != "shared-literals-exempt-unchecked":
    for number, path, anchor in exempt_rows:
        if not os.path.isfile(path):
            exempt_problems.append(f"第 {number} 行指向不存在的文件：{path}")
        elif not any(anchor in line for _line_number, _names, line in hits.get(path, [])):
            exempt_problems.append(f"第 {number} 行的锚串「{anchor}」在 {path} 里一处被判行都不命中（那一处已经改掉，或锚串抄错了）")

grown, growth_note = [], ""
if base_state == "table" and broken != "shared-literals-exempt-growth-unchecked":
    base_keys = set()
    for line in open(base_table, encoding="utf-8").read().split("\n"):
        found = EXEMPT_ROW.fullmatch(line)
        if found and not line.lstrip().startswith("#"):
            base_keys.add((found.group("path"), found.group("anchor")))
    grown = [(number, path, anchor) for number, path, anchor in exempt_rows if (path, anchor) not in base_keys]
    growth_note = f"排除表与基准 {base_name[:12]} 那一版比没涨"
elif base_state.startswith("unreadable:"):
    exempt_problems.append(f"{base_state[len('unreadable:'):][:200]}，只缩不涨没比")
else:
    growth_note = "基准里还没有这张表（或不在 git 仓里），只缩不涨这一条没比"

if scanned == 0 and not os.path.isfile(EXEMPT_TABLE):
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有 *.sh、*.py，也没有 {EXEMPT_TABLE}")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if unexempted:
    failed = True
    print(f"  ✗ {len(unexempted)} 处经 ~ 或 $HOME 取了数据根下共享的那几项（第二台的 ~ 是另一个目录，写的读的都落进它本地那一份）：")  # gate-lint:summary
    for path, number, names, line in unexempted:
        print(f"     {path}:{number}  {'、'.join(names)}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：经 lib_work_root.py 的 shared_data_root() 取：python 载入 .claude/hooks/lib_work_root.py 调 shared_data_path(<名>)（工作根调 configured_work_root()），")
    print("               shell 写 \"$(python3 <仓根>/.claude/hooks/lib_work_root.py shared <名>)\"（工作根不带参数），取不到就停、不退回 ~ 下；名单在那份库文件头「数据根」表。")
    print(f"               确实不是取目录的（认会话记录里写的路径文字、这一轮不改的文件），登记进 {EXEMPT_TABLE}：一行「<路径>:<锚串>  # 理由」，锚串取那一行里的一段原文；排除只缩不涨。")
if exempt_problems:
    failed = True
    print(f"  ✗ 排除表 {EXEMPT_TABLE} 有 {len(exempt_problems)} 处不对：")  # gate-lint:summary
    for problem in exempt_problems:
        print(f"     {problem}")  # gate-lint:detail
    print("     → 怎么办：指向的文件搬了就改路径，那一处已经改成经 lib_work_root.py 取就删掉这一行，锚串取被判那一行里的一段原文；写法见上一条出路。")
if grown:
    failed = True
    print(f"  ✗ 排除表涨了：这几行基准 {base_name[:12]} 那一版里没有：")  # gate-lint:summary
    for number, path, anchor in grown:
        print(f"     第 {number} 行 {path}:{anchor}")  # gate-lint:detail
    print("     → 怎么办：排除只缩不涨。新写的取法经 lib_work_root.py 的 shared_data_root() 取；确实要加的，交主 agent 问用户再加。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 数据根下共享的几项都不经 ~ 与 $HOME 取：扫了 {scanned} 份（{by_directory}），可疑写法 {exempted} 处全在排除表里（表 {len(exempt_rows)} 行，每行都命中）；{growth_note}"
      f"；数据根本身、名单外的名字与运行记录（{'、'.join(RUN_RECORD_ENTRIES)}）一起查了")
PY
}

# ── 格 cp-preserve-mode：项目脚本里带保留权限的 cp（records/2026-10-31-task-server巡检.md 问题 82：从 NFS 共享树往第二台本地盘拷时，cp 保留权限那一步
#   退非 0、报 Operation not supported，后面接 && 的那一步不跑、整条命令红；本机拷的是本地盘不撞，只在第二台红）
# 扫哪些：被判的根下 scripts/、.claude/hooks/、.claude/gate.d/、.claude/scripts/、task-server/、prune-server/ 里的 *.sh、*.py 与不带后缀、第一行是 #! 起 bash / sh / python 的文件；
#   跳过 target/、__pycache__/、.git/；.claude/gate.d/fixtures/ 与 .claude/hooks/fixtures/ 下只扫样本根的搭建脚本（fixtures/<阶段>/<样本>/setup.sh，喂样本时照真跑），
#   别的样本文件是造出来的输入、不扫；规范副本 .claude/singlefs-ai-sop/ 不在射程里（改在上游，照 CLAUDE.md「规范从哪来」）。
# 只看代码：逐行读法在共用库 SOURCE_CODE_LINES_LIBRARY（同 shared-data-root-literals 格）：shell 去掉整行注释、行尾注释与出路文字，引号里带空白的那一段（样本命令、
#   出路文字）去掉，嵌在 shell 里的 python 的三引号整段带空白的同样去掉；python 经 tokenize 去掉注释与带空白的字符串字面量，不带空白的字面量照代码判
#   （argv 列表里的 "cp"、"-a" 就在这里头）。自证函数体里的照判（自证真起 cp）。
# 算保留权限的写法（每处一行，两种）：
#   shell：命令词是 cp（前面是行首、空白、; & | ( { ` / 或 $(），到 ; & | ( ) < > 为止的参数里有一项是 -a、-p（单个 - 起头、几个字母合写的也算，例 -ar、-rp）、
#     --archive、不带值的 --preserve，或 --preserve= 的值里有 mode 或 all；
#   python：argv 列表里 "cp"（或 'cp'）之后的列表项里有同样的一项（整项是带引号的那一个词）。
# 改法：要时间戳的写 --preserve=timestamps，要链接的再加 links（整棵树写 cp -dR --preserve=timestamps,links），都不要的写 cp -R 或 cp；python 改用 shutil.copy2、
#   shutil.copytree(…, symlinks=True)，或 argv 写成同样的选项。没有豁免表：射程里的每一处都改。
# 判不了的：变量里拼出来的命令、带空白的命令串里的（bash -c "cp -a …"、subprocess 的 shell=True 字符串）、install -p、rsync -a 这类别的工具（不报这个错），靠 review。
# 样本：cp-preserve-mode-red 的 scripts/hosts/copy_tree.sh（-a、-rp 合写、--archive、if 里的 --preserve=mode,timestamps、$( 里不带值的 --preserve、/bin/cp -p）、
#   scripts/gate/copy_tree.py（argv 列表里的 "-a" 与 '--preserve=all'）、task-server/sample_server（不带后缀、#! 起 bash 的文件里的 -pR）、
#   .claude/gate.d/fixtures/sample-stage/case/setup.sh（样本根的搭建脚本里的 -a），共 10 处逐处点名；同一份里样本根的别的文件（data.sh 里的 -a）不点名。
#   cp-preserve-mode-green：-dR --preserve=timestamps,links、--preserve=timestamps、-R、-P、--parents、出路文字、注释、带空白的样本命令串、python 的文档串与 shutil、
#   scp -p 与 rsync -a、mkdir -p、样本根下不是 setup.sh 的文件、.claude/singlefs-ai-sop/ 下的 cp -a，都不点名；判绿并报扫了 3 份。
# 弄坏开关（CODE_TOOLING_BREAK）：cp-preserve-short-cluster-unseen（只认单写的 -a、-p，不认合写）：cp-preserve-mode-red 里 copy_tree.sh:4、task-server/sample_server:3
#   两条 want 与「10 处」找不到；cp-preserve-python-unseen（不认 python 的 argv 列表）：cp-preserve-mode-red 里 copy_tree.py:4、:5 两条 want 与「10 处」找不到；
#   cp-preserve-prose-scanned（带空白的字面量也照代码扫）：cp-preserve-mode-green 判红（样本命令串被点名）；
#   cp-preserve-fixture-setups-unscanned（样本根的 setup.sh 也不扫）：cp-preserve-mode-red 里 setup.sh:3 那条 want 与「10 处」找不到；cp-preserve-mode-green 里「扫了 3 份」找不到（只扫了 2 份）。
cell_cp_preserve_mode() {
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" <<'PY'
import collections, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken = sys.argv[3]

SCAN_DIRECTORIES = ("scripts", ".claude/hooks", ".claude/gate.d", ".claude/scripts", "task-server", "prune-server")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
FIXTURE_SETUP = re.compile(r"\.claude/(?:gate\.d|hooks)/fixtures/[^/]+/[^/]+/setup\.sh")
FIXTURE_PREFIXES = (".claude/gate.d/fixtures/", ".claude/hooks/fixtures/")
SHEBANG = re.compile(r"#!\S*(?:\s+\S+)?\s*.*\b(?P<interpreter>bash|sh|python3?)\b")
WITHOUT_WHITESPACE = re.compile(r"\S*")      # python 的字面量：内容里没有空白的留着当代码判（argv 的一项），带空白的是句子，去掉
TRIPLE_QUOTED = re.compile(r"(?s)(\"\"\"|''')(.*?)\1")
prose_scanned = broken == "cp-preserve-prose-scanned"
SHORT_FLAG = r"-[ap]" if broken == "cp-preserve-short-cluster-unseen" else r"-[A-Za-z]*[ap][A-Za-z]*"
PRESERVING = r"(?:" + SHORT_FLAG + r"|--archive|--preserve|--preserve=[A-Za-z,]*(?:mode|all)[A-Za-z,]*)"
SHELL_COMMAND = re.compile(r"(?:^|(?<=[\s;&|({`/])|(?<=\$\())cp(?=\s)(?P<arguments>[^;&|()<>]*)")
SHELL_FLAG = re.compile(r"(?:^|(?<=\s))(?P<flag>" + PRESERVING + r")(?=\s|$)")
PYTHON_LIST = re.compile(r"""(?<![\w"'])["']cp["'](?P<items>(?:\s*,\s*[^,\]\)]+)*)""")
PYTHON_ITEM = re.compile(r"""(["'])(?P<content>.*?)\1""")   # argv 列表里带引号的一项（逗号写在引号里的也整项取）
PRESERVING_WORD = re.compile(PRESERVING)


def without_prose_strings(line):
    """shell 的一行：引号里带空白的那一段是句子（出路文字、样本命令），换成空格；不带空白的照留。"""
    if prose_scanned:
        return line
    mask = source_lines.quoted_mask(line)
    characters, start = list(line), None
    for index in range(len(line) + 1):
        inside = index < len(line) and mask[index]
        if inside and start is None:
            start = index
        elif not inside and start is not None:
            if any(character.isspace() for character in line[start:index]):
                characters[start:index] = " " * (index - start)
            start = None
    return "".join(characters)


def language_of(path, name):
    suffix = os.path.splitext(name)[1]
    if suffix in (".sh", ".py"):
        return "shell" if suffix == ".sh" else "python"
    if suffix:
        return None
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            first = handle.readline()
    except OSError:
        return None
    found = SHEBANG.match(first)
    if not found:
        return None
    return "python" if found.group("interpreter").startswith("python") else "shell"


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES)
            for name in sorted(files):
                path = os.path.join(current, name)
                if path.startswith(FIXTURE_PREFIXES) and (broken == "cp-preserve-fixture-setups-unscanned" or not FIXTURE_SETUP.fullmatch(path)):
                    continue
                language = language_of(path, name)
                if language:
                    yield directory, path, language


def code_lines_of(language, text):
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, WITHOUT_WHITESPACE, prose_scanned)
    if not prose_scanned:   # shell 里嵌的 python 的文档串：三引号里带空白的整段是句子
        text = TRIPLE_QUOTED.sub(lambda found: found.group(0) if not any(character.isspace() for character in found.group(2))
                                 else re.sub(r"[^\n]", " ", found.group(0)), text)
    return [without_prose_strings(line) for line in source_lines.shell_code_lines(text, OUTPUT_TEXT)]


def preserving_flags(code):
    flags = []
    for command in SHELL_COMMAND.finditer(code):
        flags += [found.group("flag") for found in SHELL_FLAG.finditer(command.group("arguments"))]
    if broken != "cp-preserve-python-unseen":
        for listing in PYTHON_LIST.finditer(code):
            flags += [item.group("content") for item in PYTHON_ITEM.finditer(listing.group("items")) if PRESERVING_WORD.fullmatch(item.group("content"))]
    return flags


problems, hits = [], []
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if "cp" not in text:
        continue
    original = text.split("\n")
    for number, code in enumerate(code_lines_of(language, text), 1):
        for flag in preserving_flags(code):
            hits.append((path, number, flag, original[number - 1] if number <= len(original) else code))

if scanned == 0:
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有要扫的脚本")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if hits:
    failed = True
    print(f"  ✗ {len(hits)} 处 cp 带了保留权限（-a、-p、--archive、不带值的 --preserve、带 mode 或 all 的 --preserve=）：从 NFS 共享树拷到第二台本地盘时保留权限那一步退非 0"
          "（Operation not supported），命令跟着红：")  # gate-lint:summary
    for path, number, flag, line in hits:
        print(f"     {path}:{number}  {flag}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：要时间戳的写 --preserve=timestamps，要链接的再加 links（整棵树写 cp -dR --preserve=timestamps,links），都不要的写 cp -R 或 cp；")
    print("               python 改用 shutil.copy2、shutil.copytree(…, symlinks=True)，或 argv 写成同样的选项。这一格没有豁免表，射程里的每一处都改。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 项目脚本里没有带保留权限的 cp：扫了 {scanned} 份（{by_directory}）")
PY
}

# ── 格 peer-launch-direct：项目脚本里直接起 multi-host-run.sh --peer 往第二台起活（records/2026-10-31-task-server巡检.md 问题 90、94：驱动自己连第二台，
#   绕过服务的排队、额度与挑机器；.claude/rules/implementation-workflow.md「分片」一条写的是「任务里面不自己连第二台」）
# 扫哪些：被判的根下 scripts/、.claude/gate.d/、.claude/scripts/ 里的 *.sh、*.py 与不带后缀、第一行是 #! 起 bash / sh / python 的文件；跳过 target/、__pycache__/、.git/；
#   .claude/gate.d/fixtures/ 下只扫样本根的搭建脚本（fixtures/<阶段>/<样本>/setup.sh）；scripts/hosts/multi-host-run.sh 自己连同它的拆分族（scripts/gate/lib_lag_split_family.py 的 in_family：
#   scripts/hosts/ 下按钩子拆法命名的 multi-host-run-<部分>.sh、.py，自证 multi-host-run-selftest.sh 在里面）不扫：它们就是往第二台起活的那一份，
#   task-server 派第二台的尝试经它起；名字像部分文件、其实不是拆法命名的（multi-host-runner.sh）照扫；task-server/ 与钩子不在射程里（服务是唯一起它的一方，钩子里的是判法的样本输入）。规范副本 .claude/singlefs-ai-sop/ 不扫。
# 只看代码：逐行读法同 cp-preserve-mode（共用库 SOURCE_CODE_LINES_LIBRARY：shell 去掉注释、出路文字与引号里带空白的一段，python 去掉注释与带空白的字符串字面量，
#   不带空白的字面量照代码判）。一行代码里既有起它的那个词（multi-host-run，连字符或下划线写的，大小写不论：脚本路径、变量名 MULTI_HOST_RUN、context.multi_host_run 都算）
#   又有一个单独的词 --peer（前后不接字母、数字、减号与下划线）就点名。
# 改法：照服务的分片协议（task-server/task_server_lib.py 文件头「分片」）：脚本文件头写一行「# task-server-shards: <第一个参数>」，照 TASK_SERVER_SHARD_PHASE 跑计划、各片、合并，
#   计划与产物目录在共享工作根下；各片派哪台由服务定，脚本里不再起 multi-host-run.sh --peer。没有豁免表：射程里的每一处都改。
# 判不了的：命令写在带空白的字符串里（bash -c "… multi-host-run.sh --peer …"）、拆成几行拼的 argv、经变量起而那一行不带这两样的，靠 review。
# 样本：peer-launch-direct-red 的 scripts/experiment-apparatus/driver.sh（字面路径、$MULTI_HOST_RUN 两处）、scripts/experiment-apparatus/driver.py（argv 列表里 context.multi_host_run 与 "--peer"）、
#   .claude/gate.d/fixtures/sample-stage/case/setup.sh（样本根的搭建脚本里一处）、scripts/hosts/multi-host-runner.sh（不是 multi-host-run.sh 的部分文件），共 5 处逐处点名；
#   peer-launch-direct-green：分片协议写法的驱动、注释与出路文字里的 multi-host-run.sh --peer、scripts/hosts/multi-host-run.sh 自己与它的部分文件 multi-host-run-selftest.sh、
#   --peer-memory-cap 这类别的词、样本根下不是 setup.sh 的文件，都不点名；判绿并报扫了 3 份。
# 弄坏开关（CODE_TOOLING_BREAK）：peer-launch-variable-unseen（只认字面的 multi-host-run.sh，不认变量名与 python 属性）：peer-launch-direct-red 里 driver.sh:5、driver.py:4
#   两条 want 与「5 处」找不到；peer-launch-launcher-scanned（multi-host-run.sh 与它的部分文件都扫）：peer-launch-direct-green 判红；
#   peer-launch-launcher-parts-scanned（只跳过 multi-host-run.sh 本身，它的部分文件照扫）：peer-launch-direct-green 判红（multi-host-run-selftest.sh 那一行被点名）；
#   peer-launch-prose-scanned（带空白的字面量与注释也照代码扫）：peer-launch-direct-green 判红。
cell_peer_launch_direct() {
python3 - "$CODE_WITHOUT_TEXT_LIBRARY" "$SOURCE_CODE_LINES_LIBRARY" "${CODE_TOOLING_BREAK:-}" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import collections, importlib.util, os, re, sys, types
code_text = types.ModuleType("lib_code_without_text")
exec(compile(sys.argv[1], "code-without-text-library", "exec"), code_text.__dict__)
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
split_family_library_file = sys.argv[4]
specification = importlib.util.spec_from_file_location("lib_lag_split_family", split_family_library_file)
split_family = importlib.util.module_from_spec(specification)
try:
    specification.loader.exec_module(split_family)
except (OSError, ImportError, SyntaxError) as error:
    print(f"  ✗ 载不进 {split_family_library_file}：{error}")
    print("     → 怎么办：它随仓走（scripts/gate/lib_lag_split_family.py），被删了就从 git 找回来、语法错了先过 python3 -m py_compile；拆分族的认法只在那一份里写。")
    sys.exit(1)
OUTPUT_TEXT = code_text.OUTPUT_TEXT
broken = sys.argv[3]

SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/scripts")
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
LAUNCHER = "scripts/hosts/multi-host-run.sh"
FIXTURE_SETUP = re.compile(r"\.claude/gate\.d/fixtures/[^/]+/[^/]+/setup\.sh")
FIXTURE_PREFIX = ".claude/gate.d/fixtures/"
SHEBANG = re.compile(r"#!\S*(?:\s+\S+)?\s*.*\b(?P<interpreter>bash|sh|python3?)\b")
WITHOUT_WHITESPACE = re.compile(r"\S*")
TRIPLE_QUOTED = re.compile(r"(?s)(\"\"\"|''')(.*?)\1")
prose_scanned = broken == "peer-launch-prose-scanned"
LAUNCHER_WORD = re.compile(r"multi-host-run\.sh" if broken == "peer-launch-variable-unseen" else r"(?i)multi[-_]host[-_]run")


def is_launcher(path):
    """path 是 multi-host-run.sh 或它的拆分族里的部分文件（自证在里面）：它们就是往第二台起活的那一份，不扫。
    弄坏开关 peer-launch-launcher-scanned 下一个都不跳过，peer-launch-launcher-parts-scanned 下只跳过 multi-host-run.sh 本身。"""
    if broken == "peer-launch-launcher-scanned":
        return False
    if broken == "peer-launch-launcher-parts-scanned":
        return path == LAUNCHER
    return split_family.in_family(LAUNCHER, path)


PEER_WORD = re.compile(r"(?<![\w-])--peer(?![\w-])")


def without_prose_strings(line):
    if prose_scanned:
        return line
    mask = source_lines.quoted_mask(line)
    characters, start = list(line), None
    for index in range(len(line) + 1):
        inside = index < len(line) and mask[index]
        if inside and start is None:
            start = index
        elif not inside and start is not None:
            if any(character.isspace() for character in line[start:index]):
                characters[start:index] = " " * (index - start)
            start = None
    return "".join(characters)


def language_of(path, name):
    suffix = os.path.splitext(name)[1]
    if suffix in (".sh", ".py"):
        return "shell" if suffix == ".sh" else "python"
    if suffix:
        return None
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            first = handle.readline()
    except OSError:
        return None
    found = SHEBANG.match(first)
    if not found:
        return None
    return "python" if found.group("interpreter").startswith("python") else "shell"


def scanned_files():
    for directory in SCAN_DIRECTORIES:
        if not os.path.isdir(directory):
            continue
        for current, directories, files in os.walk(directory):
            directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES)
            for name in sorted(files):
                path = os.path.join(current, name)
                if is_launcher(path):
                    continue
                if path.startswith(FIXTURE_PREFIX) and not FIXTURE_SETUP.fullmatch(path):
                    continue
                language = language_of(path, name)
                if language:
                    yield directory, path, language


def code_lines_of(language, text):
    if prose_scanned:
        return text.split("\n")
    if language == "python":
        return source_lines.python_code_lines(text, OUTPUT_TEXT, WITHOUT_WHITESPACE, False)
    text = TRIPLE_QUOTED.sub(lambda found: found.group(0) if not any(character.isspace() for character in found.group(2))
                             else re.sub(r"[^\n]", " ", found.group(0)), text)
    return [without_prose_strings(line) for line in source_lines.shell_code_lines(text, OUTPUT_TEXT)]


problems, hits = [], []
scanned_by_directory = collections.Counter()
scanned = 0
for directory, path, language in scanned_files():
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    scanned += 1
    scanned_by_directory[directory] += 1
    if "--peer" not in text:
        continue
    original = text.split("\n")
    for number, code in enumerate(code_lines_of(language, text), 1):
        if LAUNCHER_WORD.search(code) and PEER_WORD.search(code):
            hits.append((path, number, original[number - 1] if number <= len(original) else code))

if scanned == 0:
    print(f"  ⊘ 本次无对象可判：{'、'.join(SCAN_DIRECTORIES)} 下没有要扫的脚本")
    sys.exit(77)
failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：看它的权限；这一格要读到射程里的每一份")
if hits:
    failed = True
    print(f"  ✗ {len(hits)} 处项目脚本直接起 multi-host-run.sh --peer 往第二台起活（绕过服务的排队、额度与挑机器）：")  # gate-lint:summary
    for path, number, line in hits:
        print(f"     {path}:{number}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：照 task-server/task_server_lib.py 文件头「分片」改：文件头写「# task-server-shards: <第一个参数>」，照 TASK_SERVER_SHARD_PHASE 跑计划、各片、合并，")
    print("               计划与产物目录在共享工作根下，各片派哪台由服务定；脚本里不再起它。这一格没有豁免表，射程里的每一处都改。")
if failed:
    sys.exit(1)
by_directory = "、".join(f"{name} {scanned_by_directory[name]}" for name in SCAN_DIRECTORIES if scanned_by_directory[name])
print(f"  ✓ 项目脚本里没有直接起 multi-host-run.sh --peer 的：扫了 {scanned} 份（{by_directory}）")
PY
}

# ── 格 verdict-library-code：登记表 verdict-library-scripts.tsv 与它点名的脚本对账（.claude/rules/verification.md「判定库只有一份」；
#   欠账 C948（判定另存别处的库）的会红那一半：取不到库就照跑不记的分支、第三列写 prune-server 的脚本里还有本地库读写、第三列还不是 prune-server）
# 读什么：被判的根里 .claude/gate.d/verdict-library-scripts.tsv（列的写法在它文件头）与 .claude/gate.d/verdict-library-code-forms.tsv（写法表：种类、名字、正则）。
#   登记表不在、一行写法也没有：退 77；写法表不在、一行写不对、正则编不过：判红。
# 判什么，三样：
#   ① 第三列：第三列写了一份库（不以「不算：」「无判定库：」起头）而不以「prune-server」起头的行判红（接完一族才翻成 prune-server，翻之前这一格照红）。
#   ② 取不到库照跑：第三列写了一份库的每一行点名的那一份，逐行只取代码照写法表里种类 silent-run 的正则搜，命中就点名；
#      出路文字照判（红句里说出「取不到库就照跑、不记」，那个分支就还在）。
#   ③ 本地库读写：第三列以 prune-server 起头的行点名的那一份，另照种类 local-library 的正则搜（git 目录下的库目录、按 git common-dir 拼库目录、
#      python 里自己开 RocksDB 或 sqlite），命中就点名。
#   ②③ 只判代码，放过两样：
#      纯说明不判：shell 的整行注释与行尾注释（不在引号里、前面是空白的 # 之后那一段），python 的注释与文档串（模块、类、函数开头那一句字符串）；
#      只在自证里认的弄坏开关护着的分支不判：开关按名字认，取那一份脚本文件头开关表里登记的值（读法同 break-switch-headers 格，BREAK_SWITCH_TABLE_LIBRARY），
#      写在命中那一行的代码里，或写在包着那一行的块的开头行里（往上逐层取缩进更浅的那一行：if / elif / while 的条件、case 的分支），就放过并计数；
#      else 那一行不写开关名，else 那一支照判；文件头开关表里没登记的名字照判。开关平常跑时被拒由 break-switch-outside-selftest 格判，这一格不重判。
#   同一份脚本登记了几行的，① 逐行判、②③ 按那份脚本判一次（几行里有一行是 prune-server 就照 ③ 判）。点名的路径不在判红。
#   库根的三种选法（选库变量、库根参数、打库根的子命令）这一格不判。
# 判不了的：拼出来的写法、写法表外的说法、经别的库间接开的本地库（例：调一份没登记的共用库去写文件）、不按缩进写的块（一行写完的 if、续行）；
#   自证函数里的本地库读写照判（只放过开关护着的分支与说明）；靠 review。
# 比过（gate-similar）：reuse-selftests 格读同一张登记表，但它跑每份的自证核复用三问（判法在 scripts/gate/lib_reuse_checks.py），不读脚本正文；
#   这一格只读正文与第三列、不起任何自证，并进 reuse-selftests 会让一格静态判据跟着自证一起跑、一起本次未跑。
# 样本：verdict-library-code-red（登记表四行：两行 prune-server 各点名一份脚本；silent.sh 文件头开关表登记 SAMPLE_BREAK=silent-run-tolerated，
#   两句取不到库照跑的、一句在那个开关护着的 if 的 else 那一支、一句在文件头没登记的名字 unlisted-mode 护着的块里，local.py 一处 common-dir 拼库目录与一处 sqlite；
#   一行第三列还写库根的 git common-dir（点名的 old_root.sh 里有 git 目录下的库目录，不是 prune-server 行、③ 不判它）；共点名照跑 4 处、本地库 2 处、第三列 1 行）；
#   verdict-library-code-green（prune-server 行点名的脚本里只有「本地不建不读不写」「不读不写真服务」「按条复用失灵：全部现判、一条都不记库」与不接库目录的
#   git common-dir；clean.sh 的注释与行尾注释、clean.py 的文档串与注释里写着取不到库照跑的说法，clean.sh 文件头登记的开关 SAMPLE_BREAK=silent-run-tolerated
#   护着的 if 里有两句（一句在条件下一层、一句再深一层）；「不算：」行点名的脚本里有取不到库照跑的那一句，不判；判绿并报查了 2 行登记、2 份脚本、放过 2 处）。
# 弄坏开关（CODE_TOOLING_BREAK）：library-code-third-column-unchecked（不判第三列）：verdict-library-code-red 里第三列那条 want 找不到；
#   library-code-silent-run-unseen（不搜 silent-run）：verdict-library-code-red 里 silent.sh 四条 want 找不到，verdict-library-code-green 成功句里的写法种数对不上；
#   library-code-local-library-unseen（不搜 local-library）：verdict-library-code-red 里 local.py 两条 want 找不到，verdict-library-code-green 成功句里的写法种数对不上；
#   library-code-unregistered-rows-scanned（「不算：」「无判定库：」行点名的也搜）：verdict-library-code-green 判红（clean_mention.sh 被点名）；
#   library-code-explanations-scanned（注释与文档串也照判）：verdict-library-code-green 判红（clean.sh、clean.py 的说明被点名）；
#   library-code-break-guards-scanned（开关护着的分支也照判）：verdict-library-code-green 判红（clean.sh 开关护着的两句被点名）；
#   library-code-guard-names-unchecked（包着的块不按文件头开关表认，任何带连字符的词都当开关名）：verdict-library-code-red 里 unlisted-mode 护着的那条 want 与「4 处」找不到；
#   library-code-else-branches-released（往上找包着的块时缩进相同的行也算，else 那一支跟着 if 放过）：verdict-library-code-red 里 else 那一支的那条 want 与「4 处」找不到。
cell_verdict_library_code() {
python3 - "${CODE_TOOLING_BREAK:-}" "$SOURCE_CODE_LINES_LIBRARY" "$BREAK_SWITCH_TABLE_LIBRARY" <<'PY'
import ast, collections, io, os, re, sys, tokenize, types
broken = sys.argv[1]
source_lines = types.ModuleType("lib_source_code_lines")
exec(compile(sys.argv[2], "source-code-lines-library", "exec"), source_lines.__dict__)
switch_table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[3], "break-switch-table-library", "exec"), switch_table.__dict__)
explanations_scanned = broken == "library-code-explanations-scanned"
guards_scanned = broken == "library-code-break-guards-scanned"
guard_names_unchecked = broken == "library-code-guard-names-unchecked"
else_branches_released = broken == "library-code-else-branches-released"

REGISTRY = ".claude/gate.d/verdict-library-scripts.tsv"
FORMS = ".claude/gate.d/verdict-library-code-forms.tsv"
NOT_LIBRARY_PREFIXES = ("不算：", "无判定库：")
SERVICE_PREFIX = "prune-server"
KINDS = ("silent-run", "local-library")

if not os.path.isfile(REGISTRY):
    print(f"  ⊘ 本次无对象可判：没有 {REGISTRY}")
    sys.exit(77)
problems = []
library_rows = []                  # (行号, 路径, 第三列)
for number, line in enumerate(open(REGISTRY, encoding="utf-8").read().split("\n"), 1):
    if not line.strip() or line.startswith("#"):
        continue
    fields = line.split("\t")
    if len(fields) < 4:
        continue   # 列数不对归 reuse-selftests 格判
    if broken != "library-code-unregistered-rows-scanned" and fields[2].startswith(NOT_LIBRARY_PREFIXES):
        continue
    library_rows.append((number, fields[0], fields[2]))
forms = collections.defaultdict(list)
if not os.path.isfile(FORMS):
    problems.append(f"写法表 {FORMS} 不在")
else:
    for number, line in enumerate(open(FORMS, encoding="utf-8").read().split("\n"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 3 or fields[0] not in KINDS or not fields[1].strip() or not fields[2]:
            problems.append(f"{FORMS} 第 {number} 行不是「种类（{' / '.join(KINDS)}）<TAB>名字<TAB>正则[<TAB># 说明]」：{line[:120]}")
            continue
        try:
            forms[fields[0]].append((fields[1], re.compile(fields[2])))
        except re.error as error:
            problems.append(f"{FORMS} 第 {number} 行的正则编不过：{error}")
if not library_rows and not forms:
    print(f"  ⊘ 本次无对象可判：{REGISTRY} 里没有写了判定库的行，也没有写法表")
    sys.exit(77)

third_column = []
service_paths, all_paths = set(), []
for number, path, location in library_rows:
    if not location.startswith(SERVICE_PREFIX):
        if broken != "library-code-third-column-unchecked" and not location.startswith(NOT_LIBRARY_PREFIXES):
            third_column.append((number, path, location))
    else:
        service_paths.add(path)
    if path not in all_paths:
        all_paths.append(path)
silent_forms = [] if broken == "library-code-silent-run-unseen" else forms["silent-run"]
local_forms = [] if broken == "library-code-local-library-unseen" else forms["local-library"]
hits = []                          # (路径, 行号, 种类, 名字, 原行)
released = 0                       # 命中在弄坏开关护着的分支里、放过的处数


def code_lines_of(path, text):
    """只留代码（行号不变）：shell 去掉整行注释与行尾注释（不在引号里、前面是空白的 # 之后），python 去掉注释与文档串；出路文字照留（红句里的那一句就是分支还在）。"""
    if path.endswith(".py"):
        rows = [list(line) for line in text.split("\n")]
        try:
            tokens = list(tokenize.generate_tokens(io.StringIO(text).readline))
            tree = ast.parse(text)
        except (tokenize.TokenError, SyntaxError, IndentationError, ValueError):
            tokens, tree = None, None
        if tokens is not None:
            for token in tokens:
                if token.type == tokenize.COMMENT:
                    (row, start), (_end_row, end) = token.start, token.end
                    rows[row - 1][start:end] = [" "] * (end - start)
            for node in ast.walk(tree):
                body = getattr(node, "body", None)
                if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) and body \
                        and isinstance(body[0], ast.Expr) and isinstance(body[0].value, ast.Constant) and isinstance(body[0].value.value, str):
                    for row in range(body[0].lineno, body[0].end_lineno + 1):
                        rows[row - 1] = []
            return ["".join(characters) for characters in rows]
    kept = []
    for line in text.split("\n"):
        if line.lstrip().startswith("#"):
            kept.append("")
            continue
        mask = source_lines.quoted_mask(line)
        for index, character in enumerate(line):
            if character == "#" and not mask[index] and (index == 0 or line[index - 1] in " \t"):
                line = line[:index]
                break
        kept.append(line)
    return kept


def guard_pattern(text):
    """那一份脚本文件头开关表里登记的弄坏开关的值（只在自证里认的那几条，按名字认），拼成按词边界认的正则；一条都没登记交 None。"""
    values = sorted({entry["value"] for entry in switch_table.switch_entries(text)}, key=len, reverse=True)
    if not values:
        return None
    return re.compile(r"(?<![A-Za-z0-9_-])(?:" + "|".join(re.escape(value) for value in values) + r")(?![A-Za-z0-9_-])")


SWITCH_LIKE = re.compile(r"(?<![A-Za-z0-9_-])[a-z0-9]+(?:-[a-z0-9]+)+(?![A-Za-z0-9_-])")   # 弄坏开关 guard-names-unchecked 下包着的块认的：任何带连字符的词


def indentation(line):
    return len(line.expandtabs()) - len(line.expandtabs().lstrip())


def guarded_by_break_switch(code, index, guard):
    """命中那一行的代码、或包着它的块的开头行（往上逐层取缩进更浅的那一行：if / elif / while 的条件、case 的分支）写着开关名，就是开关护着的分支；
    else 那一行不写开关名，else 那一支不算被护着。"""
    if guard is not None and guard.search(code[index]):
        return True
    block_guard = SWITCH_LIKE if guard_names_unchecked else guard
    if block_guard is None:
        return False
    depth = indentation(code[index])
    for row in range(index - 1, -1, -1):
        if not code[row].strip():
            continue
        level = indentation(code[row])
        if level > depth or (level == depth and not else_branches_released):
            continue
        if block_guard.search(code[row]):
            return True
        depth = level
        if depth == 0 and not else_branches_released:
            return False
    return False

for path in all_paths:
    if not os.path.isfile(path):
        problems.append(f"{REGISTRY} 点名的 {path} 不在")
        continue
    try:
        text = open(path, encoding="utf-8", errors="replace").read()
    except OSError as error:
        problems.append(f"读不了 {path}：{error}")
        continue
    lines = text.split("\n")
    code = lines if explanations_scanned else code_lines_of(path, text)
    guard = guard_pattern(text)
    checked = [("取不到库照跑", name, pattern) for name, pattern in silent_forms]
    if path in service_paths:
        checked += [("本地库读写", name, pattern) for name, pattern in local_forms]
    for number, line in enumerate(code, 1):
        for kind, name, pattern in checked:
            if pattern.search(line):
                if not guards_scanned and guarded_by_break_switch(code, number - 1, guard):
                    released += 1
                else:
                    hits.append((path, number, kind, name, lines[number - 1]))
                break

failed = False
for problem in problems:
    failed = True
    print(f"  ✗ {problem}")
    print("     → 怎么办：写法表照 .claude/gate.d/verdict-library-code-forms.tsv 文件头改，登记表点名的脚本搬了就改登记表那一行的路径（.claude/gate.d/verdict-library-scripts.tsv）。")
if third_column:
    failed = True
    print(f"  ✗ {len(third_column)} 行写了判定库而第三列不是 prune-server（判定库只有 prune-server 那一份）：")  # gate-lint:summary
    for number, path, location in third_column:
        print(f"     {REGISTRY} 第 {number} 行 {path}  「{location[:120]}」")  # gate-lint:detail
    print("     → 怎么办：把那一族接完（.claude/rules/verification.md「判定库只有一份」的三件：入口与门禁只走服务、旧库最后一次导进服务并 --verify、删掉导进去的那几类目录），")
    print("               再把第三列改写成「prune-server（<族> 族，键空间 …）」；接完之前这一格照红，不设过渡、不留放行。")
silent_hits = [entry for entry in hits if entry[2] == "取不到库照跑"]
local_hits = [entry for entry in hits if entry[2] == "本地库读写"]
if silent_hits:
    failed = True
    print(f"  ✗ {len(silent_hits)} 处带判定库的脚本取不到库还照跑、不记：")  # gate-lint:summary
    for path, number, _kind, name, line in silent_hits:
        print(f"     {path}:{number}  {name}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：取不到库名、服务连不上一律退出（连不上退 75，别的退 2 或 1，红句点名为什么），不往下现判；说那个分支的注释与出路文字一并删掉。")
if local_hits:
    failed = True
    print(f"  ✗ {len(local_hits)} 处第三列写 prune-server 的脚本里还有本地库读写：")  # gate-lint:summary
    for path, number, _kind, name, line in local_hits:
        print(f"     {path}:{number}  {name}  「{line.strip()[:160]}」")  # gate-lint:detail
    print("     → 怎么办：判定只经 prune-server 的客户端读写（scripts/prune-server/prune_server_client.py），库名只当名字、本地不建不拼库目录；")
    print("               并回即删的中转目录按 scripts/gate/admission.py library-files 的写法走，自证的库对临时服务起。")
if failed:
    sys.exit(1)
print(f"  ✓ 登记表与脚本对得上：查了 {len(library_rows)} 行写了判定库的登记、{len(all_paths)} 份脚本（第三列 prune-server 的 {len(service_paths)} 份），"
      f"第三列都是 prune-server，取不到库照跑的写法 {len(silent_forms)} 种、本地库读写的写法 {len(local_forms)} 种在代码里一处都没有"
      f"（注释与文档串不判；弄坏开关护着的分支放过 {released} 处）")
PY
}
