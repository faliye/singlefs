#!/usr/bin/env python3
# admission: always 每次照此刻副本里的源码、Cargo.lock 与工具链现算每个项的单元键、现查判定库，上一次的结论不替这一次作保
# run-condition: none 要的 git、rustc 与 cargo 版本由算键那一层自己读进键，读不到的那一项按拒算现判；连不上 prune-server 由客户端退 75
"""E206（发布固定点轮数的代价）入口 e206-fixed-point-cost-run.sh 的判定库：每个 <点>__<臂> 一条判定，经 prune-server 读写。
入口逐步起这一份（plan → record… → finish）；它不起 cargo，也不判 E206 的读数。

用法：
  e206_fixed_point_cost_library.py keys --copy <副本根> --item <项>…
      只算键、不连服务：每项一行 E206KEY item=<项> key=<键> units=<单元数>，拒算的 key=refused reason=<头一处>。退 0。
  e206_fixed_point_cost_library.py plan --copy <副本根> --library-root <仓根> --plan <计划文件> --item <项>…
      算键、查库、抽复用哨兵：每项一行 E206PLAN item=<项> decision=reuse|judge|forced|refused key=<键或 ->，
      一行 E206SENTINEL seed=<种子> sentinels=<项,…或 -> forced=<项或 ->；计划文件写这一趟的键、组成清单、决定与哨兵（整份换上）。
  e206_fixed_point_cost_library.py record --plan <计划文件> --item <项> (--product <产物文件> | --failed)
      一项跑完（产物写好、回读过）或没跑出 item_done（--failed）当场记：现判的照键写进库，强制现判与拒算的不写，只记这一趟的判定留给 finish。
  e206_fixed_point_cost_library.py stored --copy <副本根> --library-root <仓根> --item <项>…
      照此刻的键查库：每项一行 E206STORED item=<项> key=<键或 -> present=yes|no；全在退 0，有不在的退 1。给自证核「做完了」。
  e206_fixed_point_cost_library.py finish --plan <计划文件>
      逐项打「  · <项>：绿|红，现判|强制现判|复用|拒算」与 E206ITEM 行，判复用哨兵，打 name=run_tally 行与成功句。
  e206_fixed_point_cost_library.py selftest-service-problem
      入口 --selftest 开跑前调客户端 selftest_service_problem：是临时 prune-server 退 0，不是退 2（自证不写真服务）。

键（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：每项一把，照 scripts/gate/admission.py 的 compute_unit_key 算（不另写算法），请求是
  入口   crates/singlefs-verification/tests/e206_fixed_point_cost_histories.rs::item_<项>（入口拷进副本的历史用例；闭包收进副本里插过读数的实现）
  项     profile:release（cargo test --release）、line:arm=<臂>（`--item` 名字里 __ 之后那一段；历史用例读 E206_ARM）、line:test-threads=registered（这一项的跑法）
  键里有：闭包里每个单元去注释后的指纹、闭包真引到的 cfg 原子、工具链行、闭包真引到的外部包的依赖行与上面三项；
  不进键：项名、副本在哪个绝对路径、文件名、行号、日期。
  拒算（闭包收不全、入口找不到、工具链起不来）的那一项没有键：照跑、不查库、不入库，成功句报拒算几个。

判定库（.claude/rules/verification.md「判定库只有一份」）：prune-server 那一份，族 apparatus-items，种类 LIBRARY_KIND（e206-fixed-point-cost-run），
  命名空间照客户端按 --library-root 给（只在临时服务底下给，主仓与仓副本读写同一份）。相对键：
    items/<键>.out          这一项的产物整份（入口写进 research/results/ 的那一份）：在、带这一项的 item_done 就是做完了
    compositions/<键>.txt   组成清单（只当索引，键对不上时看变了哪些单元），先于 items/ 写
  一项的判定（verdict_of）：没有 item_done 是 not-done；否则 done:<sha256>，取产物里带 item=<项> 的 E206RESULT 行。
  research/results/ 里已有的产物不读、不导入：库里没有的项下一趟照键现判一次。

复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」的 ReuseSentinel，这里不另写一份）：plan 时候选 = 键算得出、库里 items/<键>.out 在且做完了的项
  （candidate_verdicts 一次成批读，与决定复用的 item_is_stored 分开查）；种子取 SINGLEFS_REUSE_SENTINEL_SEED（不设现取，打进 E206SENTINEL 那一行），
  抽 SINGLEFS_REUSE_SENTINEL_COUNT 条（默认 3）当哨兵、再抽 1 条强制现判（decision=forced：不读库、照现判跑）。
  finish 判：哨兵这一趟被跑了（record 过）⇒「复用失灵」；强制现判那一项这一趟的判定与库里不同 ⇒「判定不稳」；强制现判那一项没 record ⇒「强制现判没起作用」；
  有一样就整趟红、退 1。另核：决定现判、强制现判的项这一趟都 record 过（没 record 的报「没跑到」、退 1），有项 not-done 退 1。

计划文件（入口放在工作目录 <工作目录>/<副本名>-plan.json，每趟 plan 整份换上；只是这一趟的决定与记账，不是判定，判定只在 prune-server）。

退出码：0 成；1 finish 判红（哨兵、没跑到、not-done）；2 用法错、计划文件读不了、不是临时服务（selftest-service-problem）；3 prune-server 客户端别的错；75 prune-server 不可用（照客户端）。

弄坏开关 E206_RUN_BREAK=<项>（只给入口的 --selftest 用：E206_SELFTEST_MARK 指到入口自证建的标记文件时才认，别处设了退 2）。逐项写开关、弄坏哪一处，
下一行起写打开后入口自证红的格（失败行句首 name=selftest_case case=<格名> result=FAILED；与 scripts/experiment-apparatus/e206-fixed-point-cost-run.sh 文件头同一份）：
  E206_RUN_BREAK=library-lookup-ignored  item_is_stored 一律答「不在」（决定复用那一路不读库）
      a_finished_item_is_not_run_again、reuse-three-questions
  E206_RUN_BREAK=record-skipped  record 不往库里写
      a_finished_item_is_not_run_again、reuse-three-questions
  E206_RUN_BREAK=key-with-copy-path  请求另加一项 line:copy=<副本根的绝对路径>
      the_item_key_does_not_depend_on_the_copy_path
  E206_RUN_BREAK=key-by-whole-histories-file  请求另加一项 file:<副本里的历史用例>（整份文件进键）
      reuse-three-questions
  E206_RUN_BREAK=sentinel-skipped  plan 不抽哨兵与强制现判、finish 不判
      reuse_sentinel_draws_and_the_forced_item_is_run_again、a_rejudged_sentinel_turns_the_run_red
  E206_RUN_BREAK=sentinel-rejudge-unobserved  finish 把哨兵一律记成复用（这一趟跑了也不算现判）
      a_rejudged_sentinel_turns_the_run_red
  E206_RUN_BREAK=forced-read-as-reused  抽中强制现判的项照库复用（决定写 reuse）
      reuse_sentinel_draws_and_the_forced_item_is_run_again
  E206_RUN_BREAK=forced-verdict-unchecked  finish 不比强制现判的判定与库里那一份
      a_forced_rejudge_with_another_verdict_turns_the_run_red
  E206_RUN_BREAK=refused-items-reused  拒算的项决定写 reuse（照跳过、不跑）
      an_item_without_a_key_runs_and_is_never_recorded
  E206_RUN_BREAK=unreachable-as-empty  plan 读库时服务连不上照「库里没有」现判
      an_unreachable_library_exits_75_and_runs_nothing
  E206_RUN_BREAK=unrun-items-unchecked  finish 不核决定现判、强制现判的项这一趟 record 过没有
      an_item_planned_for_judging_but_not_run_turns_the_run_red
"""
import hashlib
import json
import os
import re
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "gate"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "prune-server"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
import admission  # noqa: E402
import lib_reuse_checks  # noqa: E402
import prune_server_client  # noqa: E402

ENTRY_SCRIPT = "scripts/experiment-apparatus/e206-fixed-point-cost-run.sh"   # 复用哨兵与复用三问认的脚本名
LIBRARY_KIND = "e206-fixed-point-cost-run"
HISTORIES_IN_THE_COPY = "crates/singlefs-verification/tests/e206_fixed_point_cost_histories.rs"
PLAN_FORM = "e206-fixed-point-cost-plan-v1"
BREAK_VARIABLE = "E206_RUN_BREAK"
SELFTEST_MARK_VARIABLE = "E206_SELFTEST_MARK"
SELFTEST_MARK_TEXT = "e206-selftest"
REFUSALS_SHOWN = 3
EXIT_RED, EXIT_USAGE = 1, 2
STATE_NAMES = {"judge": lib_reuse_checks.JUDGED, "forced": lib_reuse_checks.FORCED, "reuse": lib_reuse_checks.REUSED, "refused": "拒算"}


class UsageError(Exception):
    """命令行写错、计划文件读不了：退 2。"""


def break_switch_allowed():
    """弄坏开关只在入口自证里认：E206_SELFTEST_MARK 指的文件内容是标记。"""
    if not os.environ.get(BREAK_VARIABLE):
        return True
    try:
        with open(os.environ.get(SELFTEST_MARK_VARIABLE, ""), encoding="utf-8") as handle:
            return handle.read().strip() == SELFTEST_MARK_TEXT
    except OSError:
        return False


def broken(switch):
    return os.environ.get(BREAK_VARIABLE, "") == switch


def arm_of(item):
    return item.rsplit("__", 1)[-1]


def item_request(copy_root, item):
    """一项的判定请求（文件头「键」）。"""
    tokens = ["profile:release", f"line:arm={arm_of(item)}", "line:test-threads=registered"]
    if broken("key-with-copy-path"):
        tokens.append(f"line:copy={os.path.realpath(copy_root)}")
    if broken("key-by-whole-histories-file"):
        tokens.append(f"file:{HISTORIES_IN_THE_COPY}")
    return admission.Request(item, f"{HISTORIES_IN_THE_COPY}::item_{item}", tokens)


def item_unit_keys(copy_root, items):
    """{项: admission.UnitKey}：一个 KeyContext 算完这一趟的项（工作区只读一次）。"""
    context = admission.KeyContext(os.path.realpath(copy_root))
    return {item: admission.compute_unit_key(context, item_request(copy_root, item)) for item in items}


def refusal_text(unit_key):
    shown = "；".join(" ".join(line.split("\t")[1:4]) for line in unit_key.refusals[:REFUSALS_SHOWN])
    more = f"；另 {len(unit_key.refusals) - REFUSALS_SHOWN} 处" if len(unit_key.refusals) > REFUSALS_SHOWN else ""
    return shown + more


def product_relative(key):
    return f"items/{key}.out"


def composition_relative(key):
    return f"compositions/{key}.txt"


def library_store(library_root):
    return prune_server_client.LibraryStore(prune_server_client.FAMILY_APPARATUS_ITEMS, LIBRARY_KIND, os.path.realpath(library_root))


def verdict_of(item, text):
    """一项的判定（文件头「判定库」）：not-done，或 done:<sha256>。"""
    lines = text.split("\n")
    if f"E206RESULT name=item_done item={item}" not in lines:
        return "not-done"
    own = re.compile(r"^E206RESULT .*\bitem=" + re.escape(item) + r"( |$)")
    kept = [line for line in lines if own.match(line)]
    return "done:" + hashlib.sha256("\n".join(kept).encode("utf-8", "surrogateescape")).hexdigest()


def decoded(data):
    return data.decode("utf-8", "surrogateescape") if isinstance(data, bytes) else data


def item_is_stored(store, item, key):
    """决定复用的那一路：库里 items/<键>.out 在、带这一项的 item_done。弄坏开关 library-lookup-ignored 下一律答不在。"""
    if broken("library-lookup-ignored"):
        return False
    data = store.read(product_relative(key))
    return data is not None and verdict_of(item, decoded(data)) != "not-done"


def candidate_verdicts(store, keyed):
    """复用哨兵的候选（与 item_is_stored 分开查，一次成批读）：{项: 库里那一份的判定}，只收在、做完了的。"""
    items = sorted(keyed)
    if not items:
        return {}
    found = store.read_many([product_relative(keyed[item]) for item in items])
    candidates = {}
    for item, data in zip(items, found):
        if data is not None:
            verdict = verdict_of(item, decoded(data))
            if verdict != "not-done":
                candidates[item] = verdict
    return candidates


def write_plan(path, plan):
    """整份换上：同目录临时文件写完 fsync，再改名。"""
    temporary = f"{path}.partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump(plan, handle, indent=1)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)


def read_plan(path):
    try:
        with open(path, encoding="utf-8") as handle:
            plan = json.load(handle)
    except (OSError, ValueError) as error:
        raise UsageError(f"计划文件读不了：{path}（{error}）") from error
    if plan.get("form") != PLAN_FORM:
        raise UsageError(f"计划文件 {path} 不是这一版的形态（form 不是 {PLAN_FORM}）：照入口重起 plan")
    return plan


def command_keys(options):
    for item, unit_key in item_unit_keys(options.copy, options.items).items():
        units = sum(1 for line in unit_key.composition if line.count("\t") == 3 and ":" in line.split("\t")[2])
        if unit_key.key is None:
            print(f"E206KEY item={item} key=refused units={units} reason={refusal_text(unit_key)}")
        else:
            print(f"E206KEY item={item} key={unit_key.key} units={units}")
    return 0


def command_plan(options):
    store = library_store(options.library_root)
    unit_keys = item_unit_keys(options.copy, options.items)
    keyed = {item: unit_key.key for item, unit_key in unit_keys.items() if unit_key.key is not None}
    sentinel = lib_reuse_checks.ReuseSentinel(ENTRY_SCRIPT, "e206")
    try:
        candidates = candidate_verdicts(store, keyed)
        stored = {item: item_is_stored(store, item, key) for item, key in keyed.items()}
    except prune_server_client.PruneServerFailure:
        if not broken("unreachable-as-empty"):
            raise
        candidates, stored = {}, {item: False for item in keyed}
    if broken("sentinel-skipped"):
        sentinel.plan({}, reuse=False, off_reason="弄坏开关 sentinel-skipped")
    else:
        sentinel.plan(candidates)
    entries = {}
    for item in options.items:
        unit_key = unit_keys[item]
        if unit_key.key is None:
            decision = "reuse" if broken("refused-items-reused") else "refused"
        elif sentinel.forced(item) and not broken("forced-read-as-reused"):
            decision = "forced"
        elif stored[item]:
            decision = "reuse"
        else:
            decision = "judge"
        entries[item] = {"key": unit_key.key, "decision": decision, "composition": unit_key.composition, "refusals": unit_key.refusals}
    write_plan(options.plan, {"form": PLAN_FORM, "library_root": os.path.realpath(options.library_root),
                              "order": list(options.items), "items": entries, "sentinel": sentinel.to_json(), "recorded": {}})
    for item in options.items:
        entry = entries[item]
        print(f"E206PLAN item={item} decision={entry['decision']} key={entry['key'] or '-'}")
        if entry["key"] is None:
            print(f"  ! {item} 拒算（照跑、不查库、不入库）：{refusal_text(unit_keys[item])}")
    print(f"E206SENTINEL seed={sentinel.seed} sentinels={','.join(sentinel.sentinels) or '-'} forced={sentinel.forced_item or '-'}")
    return 0


def command_record(options):
    plan = read_plan(options.plan)
    if options.item not in plan["items"]:
        raise UsageError(f"计划文件里没有项 {options.item}")
    entry = plan["items"][options.item]
    if options.failed:
        verdict, text = "not-done", ""
    else:
        try:
            with open(options.product, encoding="utf-8", errors="surrogateescape") as handle:
                text = handle.read()
        except OSError as error:
            raise UsageError(f"产物读不了：{options.product}（{error}）") from error
        verdict = verdict_of(options.item, text)
    written = False
    if verdict != "not-done" and entry["key"] is not None and entry["decision"] in ("judge", "reuse") and not broken("record-skipped"):
        store = library_store(plan["library_root"])
        store.write(composition_relative(entry["key"]), "".join(line + "\n" for line in entry["composition"]))
        store.write(product_relative(entry["key"]), text)
        written = True
    plan["recorded"][options.item] = verdict
    write_plan(options.plan, plan)
    print(f"E206RECORD item={options.item} decision={entry['decision']} verdict={verdict} written={'yes' if written else 'no'}")
    return 0


def command_stored(options):
    store = library_store(options.library_root)
    missing = 0
    for item, unit_key in item_unit_keys(options.copy, options.items).items():
        present = unit_key.key is not None and item_is_stored(store, item, unit_key.key)
        missing += 0 if present else 1
        print(f"E206STORED item={item} key={unit_key.key or '-'} present={'yes' if present else 'no'}")
    return 0 if missing == 0 else EXIT_RED


def command_finish(options):
    plan = read_plan(options.plan)
    recorded = plan["recorded"]
    sentinel = lib_reuse_checks.ReuseSentinel.from_json(plan["sentinel"])
    tally = {"judge": 0, "forced": 0, "reuse": 0, "refused": 0}
    not_done, not_run, ran_although_reused = [], [], []
    for item in plan["order"]:
        decision = plan["items"][item]["decision"]
        verdict = recorded.get(item)
        if decision == "reuse" and verdict is not None:
            ran_although_reused.append(item)
            state = "judge"
        elif decision in ("judge", "forced") and verdict is None:
            not_run.append(item)
            state = decision
        else:
            state = decision
        tally[state] += 1
        if verdict == "not-done":
            not_done.append(item)
        green = "红" if verdict == "not-done" or item in not_run else "绿"
        print(f"  · {item}：{green}，{STATE_NAMES[state]}")
        print(f"E206ITEM item={item} decision={decision} state={state} verdict={verdict or '-'}")
    problems = []
    if not broken("sentinel-skipped"):
        for item in sentinel.sentinels:
            rejudged = item in recorded and not broken("sentinel-rejudge-unobserved")
            sentinel.observe(item, lib_reuse_checks.JUDGED if rejudged else lib_reuse_checks.REUSED, "-")
        forced = sentinel.forced_item
        if forced is not None:
            if forced in recorded:
                stored_verdict = sentinel.candidates.get(forced)
                verdict = stored_verdict if broken("forced-verdict-unchecked") else recorded[forced]
                sentinel.observe(forced, lib_reuse_checks.FORCED, verdict)
            else:
                sentinel.observe(forced, lib_reuse_checks.REUSED, "-")
        problems = sentinel.finish()
    for message, howto in problems:
        print(f"  ✗ {message}")
        print(f"     → 怎么办：{howto}")
    if broken("unrun-items-unchecked"):
        not_run = []
    if not_run:
        print(f"  ✗ E206 入口：决定现判的 {len(not_run)} 项这一趟没记（没跑到）：{'、'.join(not_run)}")
        print("     → 怎么办：入口照计划的 decision 跑全 judge 与 forced 的项、每跑完一项 record 一次；查入口哪一步漏了，整条命令重跑（库里有的照键复用）")
    if ran_although_reused:
        print(f"  · E206 入口：计划复用、这一趟却跑了的 {len(ran_although_reused)} 项照现判算：{'、'.join(ran_although_reused)}")  # gate-lint:detail
    print(f"name=run_tally judged={tally['judge'] + tally['forced']} forced={tally['forced']} reused={tally['reuse']} "
          f"refused={tally['refused']} not_done={len(not_done)} not_run={len(not_run)}")
    if problems or not_run or not_done:
        return EXIT_RED
    print(f"E206（发布固定点轮数的代价）入口：现判 {tally['judge'] + tally['forced']}（其中强制现判 {tally['forced']}）、复用 {tally['reuse']}、"
          f"拒算 {tally['refused']}（判定库 {library_store(plan['library_root']).location()}）")
    return 0


def command_selftest_service_problem(_options):
    problem = prune_server_client.selftest_service_problem()
    if problem:
        print(f"  ✗ E206 入口自证：{problem}", file=sys.stderr)
        print("     → 怎么办：照 .claude/hooks/selftest-temporary-service.tsv 文件头 required 那一条写法起：python3 scripts/prune-server/prune_server_client_selftest.py "
              "--with-temporary-service -- bash scripts/experiment-apparatus/e206-fixed-point-cost-run.sh --selftest（经 task-server/task submit 交）", file=sys.stderr)
        return EXIT_USAGE
    return 0


COMMANDS = {"keys": command_keys, "plan": command_plan, "record": command_record, "stored": command_stored, "finish": command_finish,
            "selftest-service-problem": command_selftest_service_problem}
VALUED_OPTIONS = {"--copy": "copy", "--library-root": "library_root", "--plan": "plan", "--product": "product"}
REQUIRED_OPTIONS = {"keys": ("copy", "items"), "plan": ("copy", "library_root", "plan", "items"),
                    "record": ("plan", "item"), "stored": ("copy", "library_root", "items"), "finish": ("plan",),
                    "selftest-service-problem": ()}


class CommandLine:
    """一条子命令的选项（文件头「用法」）。"""

    def __init__(self, command, arguments):
        self.copy = self.library_root = self.plan = self.product = None
        self.items = []
        self.failed = False
        index = 0
        while index < len(arguments):
            flag = arguments[index]
            if flag == "--failed":
                self.failed = True
                index += 1
                continue
            if flag not in VALUED_OPTIONS and flag != "--item":
                raise UsageError(f"认不得的参数：{flag}")
            if index + 1 >= len(arguments):
                raise UsageError(f"{flag} 后面缺值")
            if flag == "--item":
                self.items.append(arguments[index + 1])
            else:
                setattr(self, VALUED_OPTIONS[flag], arguments[index + 1])
            index += 2
        if command == "record":
            if len(self.items) != 1 or (self.product is None) == (not self.failed):
                raise UsageError("record 要恰好一个 --item，与 --product <产物文件>、--failed 二者之一")
            self.item = self.items[0]
        missing = [name for name in REQUIRED_OPTIONS[command] if name not in ("item", "items") and getattr(self, name) is None]
        if "items" in REQUIRED_OPTIONS[command] and not self.items:
            missing.append("items")
        if missing:
            raise UsageError(f"{command} 缺：{'、'.join(missing)}")
        if len(set(self.items)) != len(self.items):
            raise UsageError("同一个 --item 给了两次")


def main(arguments):
    if not break_switch_allowed():
        print(f"  ✗ {BREAK_VARIABLE} 只给入口的 --selftest 证红用，这一趟不是自证起的", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> bash {ENTRY_SCRIPT} --selftest（经临时服务的包装）", file=sys.stderr)
        return EXIT_USAGE
    if not arguments or arguments[0] not in COMMANDS:
        print(f"  ✗ 用法：e206_fixed_point_cost_library.py {'|'.join(COMMANDS)} …", file=sys.stderr)
        print("     → 怎么办：照文件头「用法」写；这一份由入口 e206-fixed-point-cost-run.sh 起", file=sys.stderr)
        return EXIT_USAGE
    try:
        return COMMANDS[arguments[0]](CommandLine(arguments[0], arguments[1:]))
    except UsageError as error:
        print(f"  ✗ e206_fixed_point_cost_library.py {arguments[0]}：{error}", file=sys.stderr)
        print("     → 怎么办：照文件头「用法」改参数；计划文件读不了的，整条入口命令重起（plan 会重写它）", file=sys.stderr)
        return EXIT_USAGE


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
