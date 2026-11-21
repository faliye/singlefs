"""task_server_selftest.py 拆出的一份：排队次序、挑机器、第二台派活、提前与 CPU 闸的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import contextlib
import json
import re
import threading
import time
import types
import task_server_lib as lib
from task_server_selftest_fakes import (
    BLOCK_SCRIPT, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, MEBIBYTE, PLACEMENT_WAIT_SECONDS, POLL_SECONDS, SEEDED_ENDED_AT, SELFTEST_MARGIN_GIBIBYTES,
    SERVER, SESSION_A, SESSION_B, STATE_TIMEOUT_SECONDS,
)
from task_server_selftest_world import (
    ATTENTION_WAIT_SECONDS, CPU_GATE_HEAVY_KEY, CPU_GATE_PROBE_KEY, NO_BORROW_ENTRY_SCRIPT, STARVED_HEALTH_TEXT, SelftestFailure, attention_fields,
    attention_lines, attention_problems, build_slot_tree, cpu_gate_scheduler, describe, fake_cargo_extra, first_attempt, health_until, loaded_host_dispatch,
    process_memory_stat, raise_cell_problems, wait_path,
)


HUNDRED_RELEASE = "release-hundred"       # hundred-five-minute-tasks-balanced 放那 100 条的放行文件（相对假仓根）

# busy-cpu-pinned-at-cap-retried、busy-cpu-below-cap-keeps-running 同一套数：库里 1536 MiB、上限 1536M；前一格假 scope 的 memory.current 钉在 1536 MiB，后一格只到它的八分之一
# aged-head-at-slice-total-starts：本机 slice 总上限 50G、slice 自己挂着回收不掉的那一截就是假余量 2G（SELFTEST_MARGIN_GIBIBYTES；任务 1713 那一回量到 0.7G，向上取整是 1G）；
# 峰值正好是总上限减余量 48G（再大一点哪台都装不下，照预算打回）：上限与记账 48G
AT_TOTAL_CAP_PINNED_TEXT = "49152M"

AGED_BACKDATE_MARGIN_SECONDS = 60   # backdate_past_aged 把提交时刻往前拨过服务的时限再多这么久

PASS_QUEUED_COUNT = 4   # prediction-computed-once 与 pass-reads-resources-once 交几条放不下的：读数随条数涨的那种写法在这几条上就分得出
PASS_COUNT = 3          # 那两格看几轮：每轮都现算的写法下每条算 3 次
RECHECK_FAILED_TEXT = "起之前复核没过"   # task_server.py 的 place_queued_into 写进排队原因的那几个字


def cell_queue_holds_unplaceable(world):
    world.resources(local=(11, 30))   # 空 11G：两条没记录的各记 4G 之后剩 3G，比第三条要的 4G（4G 加余量 2G、不超过上限 4G）小
    world.start_server()
    releases = [os.path.join(world.root, f"release-{number}") for number in (1, 2)]
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[0]])
    world.running(first)
    second = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[1]])
    world.running(second)
    third = world.submit(["bash", "-c", "exit 0"])
    world.next_pass()   # 看交了之后排过一整轮的样子，不比秒数：睡 1.5 秒看不出这期间排没排到第三条，没排到也照「排着」放过
    world.next_pass()
    summary = world.summary(third)
    if summary["task"].get("state") != "queued" or summary["attempts"] or world.calls_of(third):
        raise SelftestFailure(f"本机空 11G，两条没记录的各记 4G 之后只剩 3G，第三条（没记录，要的空闲 = 4G 加余量 2G、不超过上限 4G）此刻放不下：应当留在库里排着、没有它的包装进程，"
                              f"实际状态 {summary['task'].get('state')}、尝试 {len(summary['attempts'])} 次、包装调用 {world.calls_of(third)}")
    open(releases[0], "w").close()
    summary = world.finished(third)
    open(releases[1], "w").close()
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"放出来之后第三条应当起、succeeded，实际 {describe(summary)}")
    return "放不下的那一条留在库里、没起包装，放出来之后才起"


def cell_aged_blocked_task_blocks_later(world):
    """排满时限还起不来的任务挡住排在它后面的全部任务（task_server.py 排队那一轮里 blocked 那一支）：本机 30G，头一条（记 12G）占着，排头（记 28G）放不下；
    等它排满时限（自证里 2 秒）之后再交两条放得下的小任务：两条都不起，排队原因写「排在任务 N 后面：它已排满时限」；放开头一条之后排头先起，后面两条再起，都跑完。
    弄坏开关 aged-blocked-lets-others-start 下后面两条照常先起，这一格红。"""
    world.resources(local=(30, 30))
    for key, peak in (("selftest-aged-block-first", 12 * GIBIBYTE), ("selftest-aged-block-head", 28 * GIBIBYTE)):
        world.seed_peak(key, peak)
    world.start_server(extra={"TASK_SERVER_SELFTEST_AGED_SECONDS": "2"})
    release = os.path.join(world.root, "release")
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-aged-block-first"})
    later, waiting = [], {}
    try:
        world.running(first)
        head = world.submit(["bash", "-c", "exit 0"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-aged-block-head"})
        time.sleep(3)   # 排头满了时限（下限：机器忙时只会更久）
        world.next_pass()
        later = [world.submit(["bash", "-c", "exit 0"]) for _ in range(2)]
        world.next_pass()
        world.next_pass()
        waiting = {task_id: world.summary(task_id) for task_id in later}
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id) for task_id in [first, head, *later]}
    problems = []
    for task_id in later:
        found = waiting.get(task_id) or {"task": {}, "attempts": []}
        if found["attempts"] or f"排在任务 {head} 后面" not in (found.get("waiting_for") or "") or "已排满时限" not in (found.get("waiting_for") or ""):
            problems.append(f"任务 {task_id}（排在排满时限的任务 {head} 后面）：应当没起、排队原因写「排在任务 {head} 后面：它已排满时限」，实际 {describe(found)}、原因「{found.get("waiting_for")}」")
    orders = {task_id: attempt_order(summaries[task_id]) for task_id in [head, *later]}
    if orders[head] is None or any(orders[task_id] is None or orders[task_id] < orders[head] for task_id in later):
        problems.append(f"放开之后排头应当先于后面两条起，实际尝试号 {orders}")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放开之后都应当跑完，实际 {[describe(summary) for summary in summaries.values()]}")
    raise_cell_problems(world, problems, "排满时限还起不来的任务挡住后面的")
    return f"排头（任务 {head}）排满时限还放不下：后面两条放得下的都不起、原因点名它；放开之后排头先起（尝试 {orders[head]}），后面两条再起，都跑完"


def cell_jump_only_shorter_than_running(world):
    """插队规则（task_server.py 的 jump_refusal；规则「插队的任务的时长不得超过正在运行的任务剩余的时长。没有上次运行时间的任务不得插队」）：本机 30G，头一条（记 12G、
    上次跑了 600 秒）占着，排头（记 28G）放不下、还没排满时限。后面交三条放得下的小任务：上次跑了 1 秒的先起、跑完（头一条还占着）；没有上次运行时长的与上次跑了
    100000 秒的都不起，排队原因写「不插队」。放开头一条之后都跑完。弄坏开关 jump-length-unchecked 下后两条也先起，这一格红。"""
    world.resources(local=(30, 30))
    for key, peak in (("selftest-jump-first", 12 * GIBIBYTE), ("selftest-jump-head", 28 * GIBIBYTE)):
        world.seed_peak(key, peak)
    release = os.path.join(world.root, "release")
    blocker = ["bash", "-c", BLOCK_SCRIPT, "block", release]
    short, lengthy, unrecorded = (["bash", "-c", "exit 0", name] for name in ("jump-short", "jump-lengthy", "jump-unrecorded"))
    world.seed_duration("selftest-jump-first", 600, blocker)
    world.seed_duration("selftest-jump-short", 1, short)
    world.seed_duration("selftest-jump-lengthy", 100000, lengthy)
    world.start_server()
    first = world.submit(blocker, extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-first"})
    waiting, jumped = {}, None
    try:
        world.running(first)
        head = world.submit(["bash", "-c", "exit 0"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-head"})
        world.next_pass()
        refused = [world.submit(lengthy, extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-lengthy"}), world.submit(unrecorded)]
        allowed = world.submit(short, extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-short"})
        jumped = world.finished(allowed)   # 头一条还占着、排头还没起，这一条先跑完
        world.next_pass()
        waiting = {task_id: world.summary(task_id) for task_id in [head, *refused]}
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id) for task_id in [first, head, *refused]}
    problems = []
    if jumped is None or jumped["task"].get("outcome") != "succeeded" or (waiting.get(head) or {"attempts": [1]})["attempts"]:
        problems.append(f"上次跑了 1 秒的那一条应当在排头（任务 {head}）起之前跑完，实际 {describe(jumped) if jumped else '没跑完'}、排头 {describe(waiting.get(head) or {'task': {}, 'attempts': []})}")
    for task_id in refused:
        found = waiting.get(task_id) or {"task": {}, "attempts": []}
        if found["attempts"] or "不插队" not in (found.get("waiting_for") or ""):
            problems.append(f"任务 {task_id}（没有上次运行时长，或上次跑得比在跑的剩余长）应当没起、排队原因写「不插队」，实际 {describe(found)}、原因「{found.get('waiting_for')}」")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放开之后都应当跑完，实际 {[describe(summary) for summary in summaries.values()]}")
    raise_cell_problems(world, problems, "插队只许比在跑的剩余短的")
    return "排头没起时：上次跑了 1 秒的先起、跑完；没有上次运行时长的与上次跑了 100000 秒的不插队；放开之后都跑完"


QUEUED_SAMPLE_TASK = 5   # queued_attention_kinds 问的那条在排任务的号


def queued_attention_kinds(judged, budget, idle_short, reasoned=False):
    """假调度器上问一条在排的任务（号 QUEUED_SAMPLE_TASK、没有尝试、排队过久的门槛设得到不了）此刻要处置的提醒的种类（attention_of_record）：
    judged 是排队那一轮对它判过「在每一台上都起不来」了（idle_short_judged 里有它），reasoned 是排队线程给它写了排队原因（waiting_reasons 里有它：judged 的一定有，
    没判过而有的是排在排满时限的任务后面、有一台判不成这几种），budget、idle_short 各是 budget_notices、idle_short_notices 里有没有它那一项。→ 种类表（照报的先后）。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.pause, fake.queue_attention_seconds = dict(os.environ), None, 1e9
    fake.idle_short_judged = {QUEUED_SAMPLE_TASK} if judged else set()
    fake.waiting_reasons = {QUEUED_SAMPLE_TASK: "自证：在等整机空下来"} if judged or reasoned else {}
    fake.budget_notices = {QUEUED_SAMPLE_TASK: {"kind": "oversized-budget"}} if budget else {}
    fake.idle_short_notices = {QUEUED_SAMPLE_TASK: {"kind": "booking-beyond-idle-machine"}} if idle_short else {}
    fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [], select=lambda *arguments, **keywords: [])
    record = {"id": QUEUED_SAMPLE_TASK, "state": "queued", "submitted_at": time.time()}
    return [item["kind"] for item in SERVER.Scheduler.attention_of_record(fake, record, time.time())]


def idle_short_judged_after(before, kind, settled):
    """假调度器上调 note_idle_short_judged：这一条（号 QUEUED_SAMPLE_TASK）这一轮判之前在不在 idle_short_judged 里（before），这一轮的判是 kind（start、blocked）、idle_short_settled 是 settled，
    → 调完在不在。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.idle_short_judged = {QUEUED_SAMPLE_TASK} if before else set()
    SERVER.Scheduler.note_idle_short_judged(fake, {"id": QUEUED_SAMPLE_TASK}, SERVER.Placement(kind, idle_short_settled=settled))
    return QUEUED_SAMPLE_TASK in fake.idle_short_judged


def cell_idle_short_not_blocking(world):
    """在每一台上都起不来的任务不挡后面的（task_server.py 文件头 ②「内存」的整机空闲不够、idle_short_text、place_queued_into）：两台都是 slice 20G、余量 2G、整机空闲 17.5G，
    记过峰值 18G 的命令（记账收到可用上限 18G，要的空闲 18G）在两台上都起不来，而且没有在跑的任务可等：排队原因写「在每一台上都起不来」，wait 打一行 kind=booking-beyond-idle-machine
    （依据逐台写放出多少之后还差什么、退 ATTENTION_EXIT）；只打这一行：这一条的记账 18G 也不小于可用上限 18G 的 85%，oversized-budget 同样判出，但它起不来、「照跑」走不通，由前者罩住、不再打第二行；任务照旧在排。
    这一条的 wait 在后面的小任务交之前起并等到那一行：小任务在跑的那一阵子，这一条等的是它放出内存、不再是「整机自己空下来」，那一阵 booking-beyond-idle-machine 照实判没了、oversized-budget 照报，
    wait 在那一阵才头一回读到就是另一行（交小任务与起 wait 的先后、两个线程谁先读到都不定），所以先读完这一行再交小任务。
    排在它后面的没有上次运行时长的小任务照起、跑完（不算「排在前面还没起的」，没有上次运行时长的也不被判不插队）。
    本机整机空闲回到 20G 之后那一条起来（占着不放），再交一条同键的：本机放不下，但等在跑的收尾就放得下，不是「在每一台上都起不来」，它在排着、「在每一台上都起不来」那一判对它判过、判出不是：
    wait 只打 oversized-budget 一行（只有它成立时判过之后照报）；放行之后两条都跑完（不是 never-fits）。
    两种提醒的先后另在假调度器上直接问 attention_of_record（queued_attention_kinds：判没判过「在每一台上都起不来」、排队线程写没写排队原因、两项记没记的几种组合，不靠起服务时两个线程谁先读到），
    与 note_idle_short_judged 记表的几种（idle_short_judged_after）：
    还没判过它（idle_short_judged 里没有它，排队线程写没写排队原因都一样）而 budget_notices 里有（交的时候 confirmation 的 budget_check 就记下的）的不报 oversized-budget，免得它抢在 booking-beyond-idle-machine 之前；
    判过、idle_short_notices 里也有的只报 booking-beyond-idle-machine；判过、只有 budget_notices 的照报 oversized-budget（起得来、只是贴着整台的，不被推迟到不报）。
    弄坏开关 idle-short-blocks-later 下后面的小任务被判不插队、起不来，idle-short-silent 下 wait 不打那一行，idle-short-unjudged 下两样都不成，oversized-under-idle-short-reported 下 wait 多打一行 oversized-budget、
    假调度器那一问判过且两项都有的多报一项，oversized-reported-before-queue-judged 下那一问还没判过的也报，这一格红。"""
    key = "selftest-idle-short"
    world.resources(local=(17.5, 20), peer=(17.5, 20), peer_cap_gibibytes=20)
    world.seed_peak(key, 18 * GIBIBYTE)
    world.start_server()
    problems = []
    release = os.path.join(world.root, "release-idle-short")
    holder = ["bash", "-c", BLOCK_SCRIPT, "block", release]   # 起来之后占着不放，放行文件在才退
    big = second = None
    try:
        big = world.submit(holder, extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        completed = world.client(["wait", str(big)], timeout=ATTENTION_WAIT_SECONDS)
        more, fields = attention_problems(completed, big, "booking-beyond-idle-machine", lib.ATTENTION_EXIT)
        problems += more
        basis = fields.get("basis", "")
        if "放出" not in basis or "空闲" not in basis:
            problems.append(f"依据应当逐台写服务的任务全收尾、放出多少之后还差什么，实际「{basis}」")
        small = world.submit(["bash", "-c", "exit 0", "idle-short-small"])
        try:
            started = world.finished(small, timeout=PLACEMENT_WAIT_SECONDS)
            if started["task"].get("outcome") != "succeeded":
                problems.append(f"排在后面的小任务应当跑完，实际 {describe(started)}")
        except SelftestFailure as failure:
            problems.append(f"排在在每一台上都起不来的那一条后面的小任务（没有上次运行时长）应当照起、跑完，实际 {failure}")
        held = world.summary(big)
        reason = held.get("waiting_for") or ""
        if held["task"].get("state") != "queued" or held["attempts"] or SERVER.IDLE_SHORT_TEXT not in reason:
            problems.append(f"在每一台上都起不来的那一条应当照旧在排、没起尝试、排队原因写「{SERVER.IDLE_SHORT_TEXT[:20]}…」，实际 {describe(held)}、原因「{reason[:200]}」")
        world.resources(local=(20, 20), peer=(17.5, 20), peer_cap_gibibytes=20)
        world.running(big)   # 本机整机空闲回到 20G 之后起来（不是 never-fits），占着本机
        second = world.submit(holder, extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        completed = world.client(["wait", str(second)], timeout=ATTENTION_WAIT_SECONDS)
        more, _ = attention_problems(completed, second, "oversized-budget", lib.ATTENTION_EXIT)
        problems += [f"同键再交的一条（本机被前一条占着、放不下，等它收尾就放得下，不是在每一台上都起不来）：{problem}" for problem in more]
    finally:
        open(release, "w").close()
    for task_id in (big, second):
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
        if done["task"].get("outcome") != "succeeded":
            problems.append(f"放行之后任务 {task_id} 应当跑完 succeeded（不是 never-fits），实际 {describe(done)}")
    for what, judged, budget, idle_short, reasoned, wanted in (
            ("还没判过它、也没有排队原因、只有交的时候记下的 oversized-budget", False, True, False, False, []),
            ("排队线程给了排队原因（排在排满时限的任务后面这类）、「在每一台上都起不来」那一判还没判到它、只有 oversized-budget", False, True, False, True, []),
            ("判过、两项都开着", True, True, True, True, ["booking-beyond-idle-machine"]),
            ("判过、只有 oversized-budget（起得来、只是贴着整台）", True, True, False, True, ["oversized-budget"])):
        kinds = queued_attention_kinds(judged, budget, idle_short, reasoned)
        if kinds != wanted:
            problems.append(f"{what}：attention_of_record 应当只报 {wanted}，实际 {kinds}")
    for what, before, kind, settled, wanted in (
            ("放得下的（start）", False, "start", False, True),
            ("放不下、idle_short_text 真对每一台判过的（不管判出什么）", False, "blocked", True, True),
            ("放不下、有一台这一轮判不成的", False, "blocked", False, False),
            ("原先判过、这一轮有一台判不成的", True, "blocked", False, False)):
        recorded = idle_short_judged_after(before, kind, settled)
        if recorded != wanted:
            problems.append(f"{what}：note_idle_short_judged 之后 idle_short_judged 里有没有它应当是 {wanted}，实际 {recorded}")
    raise_cell_problems(world, problems, "在每一台上都起不来的任务不挡后面的")
    return (f"整机空闲 17.5G 不够要的 18G：wait 打 kind=booking-beyond-idle-machine，后面的小任务照起、跑完，空闲回到 20G 之后它自己起；同键再交的一条等在跑的收尾："
            f"wait 只打 oversized-budget，放行之后都跑完（依据「{basis[:80]}」）")


def cell_jump_only_same_resource(world):
    """插队只拦争同一样资源的（task_server.py 的 jump_refusal、pool_only_tree）：本机 30G，一棵树里三条会编的任务（假 cargo 编完等放行）占满这棵树在本机的三个编译目录，
    排头是这棵树里第四条会编的任务：内存放得下、只缺编译目录名额，排队原因点名「编译目录组满」。排头之后交三条放得下、都没有上次运行时长的：不编译的与别的树里会编的先起、跑完
    （排头还排着）；同一棵树里会编的争同一样资源，不起。放开之后都跑完。队头缺内存时没有上次运行时长的照旧不插队，是 jump-only-shorter-than-running 那一格。
    弄坏开关 jump-refused-any-resource 下缺编译目录名额的排头也按缺内存、CPU 的判，前两条被判「不插队」，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    tree, other_tree = build_slot_tree(world, "tree-jump-full"), build_slot_tree(world, "tree-jump-other")
    release = os.path.join(world.root, "release-jump-same-resource")
    extra = fake_cargo_extra(world)
    holders, waiting, jumped = [], {}, {}
    try:
        for number in range(SERVER.BUILD_POOL_SLOTS_PER_TREE):
            holders.append(world.submit(["bash", "-c", "cargo build && " + HOLD_UNTIL_RELEASED_SCRIPT, f"jump-holder-{number}", release], extra=extra, cwd=tree))
        for task_id in holders:
            world.running(task_id)
        head = world.submit(["cargo", "build"], extra=extra, cwd=tree)
        world.next_pass()
        jumpers = {"不编译的": world.submit(["bash", "-c", "exit 0"]), "别的树里会编的": world.submit(["cargo", "build"], extra=extra, cwd=other_tree)}
        same_tree = world.submit(["cargo", "build"], extra=extra, cwd=tree)
        jumped = {name: world.finished(task_id) for name, task_id in jumpers.items()}   # 三个编译目录还占着、排头还没起，这两条先跑完
        world.next_pass()
        waiting = {task_id: world.summary(task_id) for task_id in (head, same_tree)}
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id) for task_id in [*holders, head, same_tree]}
    problems = []
    found = waiting.get(head) or {"task": {}, "attempts": []}
    if found["attempts"] or found["task"].get("state") != "queued" or SERVER.BUILD_POOL_FULL_PREFIX not in (found.get("waiting_for") or ""):
        problems.append(f"排头（任务 {head}）应当没起、排队原因点名「{SERVER.BUILD_POOL_FULL_PREFIX}」（这一格的前提），实际 {describe(found)}、原因「{found.get('waiting_for')}」")
    for name, summary in jumped.items():
        if summary["task"].get("outcome") != "succeeded" or not summary["attempts"]:
            problems.append(f"{name}（没有上次运行时长）与排头争的不是同一样资源，应当在排头（任务 {head}）起之前跑完，实际 {describe(summary)}、原因「{summary.get('waiting_for')}」")
    found = waiting.get(same_tree) or {"task": {}, "attempts": []}
    reason = found.get("waiting_for") or ""
    if found["attempts"] or found["task"].get("state") != "queued" or not ("不插队" in reason or SERVER.BUILD_POOL_FULL_PREFIX in reason):
        problems.append(f"同一棵树里会编的（任务 {same_tree}）要的是同一样资源，应当没起、排队原因写「不插队」或「{SERVER.BUILD_POOL_FULL_PREFIX}」，实际 {describe(found)}、原因「{reason}」")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放开之后都应当跑完，实际 {[describe(summary) for summary in summaries.values()]}")
    raise_cell_problems(world, problems, "插队只拦争同一样资源的")
    return "排头只缺编译目录名额时：不编译的与别的树里会编的没有上次运行时长也先起、跑完；同一棵树里会编的不起；放开之后都跑完"


def cell_arrival_order(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_ids = [world.submit(["bash", "-c", "exit 0"]) for _ in range(3)]
    world.resources(local=(30, 30))
    # 起的先后看服务写尝试记录的次序（记录号）：同一轮里先后起的几条，各自单元里的假包装往调用记录写行的先后不定，不拿它判
    records = [world.finished(task_id)["attempts"] for task_id in task_ids]
    started = [attempts[0].get("id") if attempts else None for attempts in records]
    if None in started or started != sorted(started) or len(set(started)) != len(started):
        raise SelftestFailure(f"都放得下时应当按到达次序 {task_ids} 起，实际各条第一次尝试的记录号 {started}")
    return "都放得下时按到达次序起"


def cell_prediction_computed_once(world):
    world.resources(local=(3, 30))   # 空 3G：没有记录的命令记 4G、要的空闲 = 4G 加余量 2G、不超过上限 4G，哪一条都放不下
    world.start_server()
    task_ids = [world.submit(["bash", "-c", "exit 0"]) for _ in range(PASS_QUEUED_COUNT)]
    world.full_passes(PASS_QUEUED_COUNT, PASS_COUNT)
    lines = world.scheduler_reads()
    counts = {task_id: lines.count(f"predict task={task_id}") for task_id in task_ids}
    states = {task_id: world.summary(task_id)["task"].get("state") for task_id in task_ids}
    if any(count != 1 for count in counts.values()) or any(state != "queued" for state in states.values()):
        raise SelftestFailure(f"本机空 3G、{PASS_QUEUED_COUNT} 条没记录的都放不下，排了 {PASS_COUNT} 轮以上：每条的预测应当只算一次（predict 共 {PASS_QUEUED_COUNT} 次），"
                              f"各条照排；实际各条算了 {counts} 次、状态 {states}", world.server_log_lines())
    return f"{PASS_QUEUED_COUNT} 条放不下的排了 {PASS_COUNT} 轮以上，预测共算 {sum(counts.values())} 次（每条一次）"


def cell_pass_reads_resources_once(world):
    world.resources(local=(3, 30))   # 同上：哪一条都放不下，每一轮都把 PASS_QUEUED_COUNT 条判一遍
    world.start_server()
    for _ in range(PASS_QUEUED_COUNT):
        world.submit(["bash", "-c", "exit 0"])
    numbers = world.full_passes(PASS_QUEUED_COUNT, PASS_COUNT)
    lines = world.scheduler_reads()
    expected = ["what=pending host=local", "what=snapshot"]   # 只派得了本机：一轮读一次本机的余量、一次本机的待涨量
    reads = {number: sorted(line.split(" ", 3)[3] for line in lines if line.startswith(f"read view={number} source=pass ")) for number in numbers}
    wrong = {number: got for number, got in reads.items() if got != expected}
    if wrong:
        raise SelftestFailure(f"{PASS_QUEUED_COUNT} 条在排的一轮里，余量与待涨量应当各读一次（{expected}），不随条数涨；实际这几轮读的 {wrong}", world.server_log_lines())
    return f"{PASS_QUEUED_COUNT} 条在排的 {PASS_COUNT} 轮里，每轮本机的余量与待涨量各读一次"


def cell_pass_booking_carried(world):
    world.resources(local=(7, 30), hold=True)   # 空 7G：没记录的记 4G、要的空闲 4G（4G 加余量 2G、不超过上限 4G）；起了一条之后剩 3G，第二条放不下
    world.start_server()
    release = os.path.join(world.root, "release")
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    second = world.submit(["bash", "-c", "exit 0"])
    world.resources(local=(7, 30))   # 两条都在排时放开：同一轮里判这两条
    world.running(first)
    world.next_pass()
    summary = world.summary(second)
    rechecks = [line for line in world.scheduler_reads() if line.startswith(f"recheck task={second} ")]
    open(release, "w").close()
    if summary["task"].get("state") != "queued" or summary["attempts"] or rechecks or RECHECK_FAILED_TEXT in (summary.get("waiting_for") or ""):
        raise SelftestFailure(f"本机空 7G 只放得下一条没记录的：同一轮里起了头一条之后，它的 4G 记账应当加进这一轮本机的待涨量（剩 3G 比要的 4G 小），第二条在判的时候就放不下、"
                              f"不走到起之前复核；实际第二条状态 {summary['task'].get('state')}、尝试 {len(summary['attempts'])} 次、复核 {rechecks}、"
                              f"排队原因「{summary.get('waiting_for')}」", world.server_log_lines())
    summaries = [world.finished(task_id) for task_id in (first, second)]
    if any(item["task"].get("outcome") != "succeeded" for item in summaries):
        raise SelftestFailure(f"头一条放出来之后两条都应当 succeeded，实际 {[describe(item) for item in summaries]}", world.server_log_lines())
    return "一台只放得下一条时同一轮只起一条：起了的记账带到这一轮后面，第二条在判的时候就放不下；头一条跑完第二条照起"


def cell_launch_rechecked(world):
    world.resources(local=(30, 30), recheck_local=(3, 30))   # 排队那一轮空 30G 放得下；起之前现读空 3G，没记录的（要的空闲 4G：4G 加余量 2G、不超过上限 4G）放不下
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    summary = world.summary(task_id)
    while patience.waiting() and not summary["attempts"]:
        if any(line.startswith(f"recheck task={task_id} ") and line.endswith(" fits=0") for line in world.scheduler_reads()):
            world.next_pass()   # 复核没过的那一轮排完：排队原因已换上
            summary = world.summary(task_id)
            break
        time.sleep(POLL_SECONDS)
        summary = world.summary(task_id)
    waiting = summary.get("waiting_for") or ""
    if summary["task"].get("state") != "queued" or summary["attempts"] or world.calls_of(task_id) or RECHECK_FAILED_TEXT not in waiting:
        raise SelftestFailure(f"判的时候本机空 30G 放得下、起之前现读只剩 3G 放不下：这一条应当不起、留在队里，排队原因写「{RECHECK_FAILED_TEXT}」；"
                              f"实际状态 {summary['task'].get('state')}、尝试 {len(summary['attempts'])} 次、包装调用 {world.calls_of(task_id)}、排队原因「{waiting}」",
                              world.server_log_lines())
    world.resources(local=(30, 30))
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"起之前现读也放得下之后这一条应当起、succeeded，实际 {describe(summary)}", world.server_log_lines())
    return f"判的时候放得下、起之前现读放不下的不起，排队原因写「{RECHECK_FAILED_TEXT}」；现读也放得下之后照起"


def cell_queued_revalidated_on_restart(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    stale = world.submit(["bash", "-c", "exit 0"])
    fine = world.submit(["bash", "-c", "exit 0"])
    world.stop_server()
    world.rewrite_signed_task(stale, lambda record: record.update(command=record["command"] + ["--memory", "8G"]))   # 照旧规则交进来的那一条
    world.start_server()
    summary = world.finished(stale, timeout=20)
    waiting = world.summary(fine)
    world.resources(local=(30, 30))
    world.finished(fine)
    if summary["task"].get("outcome") != "resubmit-required" or summary["task"].get("exit_code") != 232 or "--memory" not in (summary["task"].get("note") or "") \
            or summary["attempts"] or waiting["task"].get("state") != "queued":
        raise SelftestFailure(f"服务起来时照现行规则再核在排的：带 --memory 的那一条应当打回 resubmit-required（退 232、说明点名 --memory、一次没起），合规的照排；"
                              f"实际 {describe(summary)}、说明「{summary['task'].get('note')}」、合规的 {waiting['task'].get('state')}", world.server_log_lines())
    return "服务起来时照现行规则再核在排的：不合的打回 resubmit-required（退 232、点名哪一条）、合规的照排"


def cell_starved_head_reported(world):
    """派活判排头能起却连着几轮没起成要报（task_server.py 的 starved_problem 读 head_verdict）：本机空 100G、排头一条小的，排队线程照判、判能起而不起
    （TASK_SERVER_SELFTEST_PLACEMENT_PAUSED），过了阈值（1 秒）/health 回不健康、问题是「派活判它此刻放得下」那一句；放开之后跑完、回 ok。
    弄坏开关 starved-head-unreported 下 /health 不报这一句，这一格红。"""
    world.resources(local=(100, 100))
    paused = os.path.join(world.root, "placement-paused")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused})
    open(paused, "w").close()
    task_id = world.submit(["bash", "-c", "exit 0"])
    health = health_until(world, lambda health: not health.get("ok"), 10)
    os.remove(paused)
    if health.get("ok") or STARVED_HEALTH_TEXT not in (health.get("problem") or ""):
        raise SelftestFailure(f"本机空 100G、排头一条小的、排队线程判能起而不起：过了阈值（1 秒）/health 应当回不健康、问题写「{STARVED_HEALTH_TEXT}」，实际 {health}",
                              world.server_log_lines())
    summary = world.finished(task_id)
    recovered = health_until(world, lambda health: health.get("ok"), 10)
    if summary["task"].get("outcome") != "succeeded" or not recovered.get("ok"):
        raise SelftestFailure(f"排队线程接着派之后那一条应当跑完、/health 回 ok，实际 {describe(summary)}、{recovered}", world.server_log_lines())
    return f"排头放得下却没起时 /health 回「{health.get('problem')}」，接着派之后跑完、回 ok"


def cell_blocked_head_not_starved(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    health = health_until(world, lambda health: not health.get("ok"), 3)
    world.resources(local=(30, 30))
    summary = world.finished(task_id)
    if not health.get("ok"):
        raise SelftestFailure(f"排头放不下（余量 hold）时排着不动是正常的，/health 应当一直回 ok，实际 {health}", world.server_log_lines())
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"放得下之后那一条应当跑完，实际 {describe(summary)}", world.server_log_lines())
    return "排头放不下时排着不动，3 秒里 /health 一直回 ok；放得下之后跑完"


FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES = 90   # starved-head-judged-by-the-formula：先占着的那一条的峰值（上限与记账照它 90G，在 100G、余量 2G 的台上起得来）
FORMULA_HEAD_PEAK_GIBIBYTES = 12           # 那一格的排头的峰值（上限与记账 12G：judge_host 放得下，公式只剩 100 − 2 − 90 = 8G 放不下）
FORMULA_HEAD_ROOMY_GIBIBYTES = 200         # 那一格换上的假余量（空闲与 slice 都是这么多）：公式剩 200 − 2 − 90 = 108G，排头放得下
FORMULA_HEAD_WATCH_SECONDS = 4 * SERVER.FAST_STARVED_SECONDS   # 那一格公式放不下时看 /health 看这么久（过阈值几倍）
PAUSE_SETTLE_SECONDS = 1.0                 # 那一格停排队之后等这么久再换余量（一轮 0.1 秒：开了头的那一轮早排完，换上的余量不会被它读去起排头）


def cell_starved_head_judged_by_the_formula(world):
    """/health 只读派活给排头的结论（task_server.py 的 starved_problem 读 head_verdict；judge_placement 照 judged_on_host 过公式）：本机 100G、余量 2G，一条上限 90G 的在跑、
    它的 scope 已用满 90G（不记待涨量，记账照此刻实测 90G），排头上限 12G：judge_host 照余量放得下，公式（配置的总上限 − 余量 − 在跑的记账之和）只剩 8G 放不下，派活判它不能起，
    /health 一直回 ok；换成 200G 的余量（公式剩 108G）、排队线程照判不起（TASK_SERVER_SELFTEST_PLACEMENT_PAUSED），派活判它能起却没起成，/health 照报「派活判它此刻放得下」；
    放开之后都跑完。弄坏开关 blocked-head-counted-as-starved 下前一半也报，starved-head-unreported 下后一半不报，这一格红。"""
    blocker_key, head_key = "selftest-formula-blocker", "selftest-formula-head"
    world.seed_peak(blocker_key, FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES * GIBIBYTE)
    world.seed_peak(head_key, FORMULA_HEAD_PEAK_GIBIBYTES * GIBIBYTE)
    world.resources(local=(100, 100))
    paused = os.path.join(world.root, "placement-paused")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused})
    release = os.path.join(world.root, "release-formula-blocker")
    blocker = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "formula-blocker", release], extra={lib.KEY_VARIABLE: blocker_key})
    problems = []
    head = None
    try:
        attempt = first_attempt(world.running(blocker))
        cap, label = int(attempt.get("memory_cap") or 0), attempt.get("label")
        world.resources(local=(100, 100), local_scopes={label: cap} if label else None)
        head = world.submit(["bash", "-c", "exit 0", "formula-head"], extra={lib.KEY_VARIABLE: head_key})
        waiting = world.wait_for(head, lambda found: SERVER.RUNNING_BOOKED_SUM_TEXT in (found.get("waiting_for") or "") or found["task"].get("state") != "queued",
                                 "照公式报放不下")
        blocked_health = health_until(world, lambda found: not found.get("ok"), FORMULA_HEAD_WATCH_SECONDS)
        if cap != FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES * GIBIBYTE or not label or waiting["task"].get("state") != "queued" or SERVER.RUNNING_BOOKED_SUM_TEXT not in (waiting.get("waiting_for") or ""):
            problems.append(f"先占着的那一条应当上限 90G、有 label，排头照公式排着、排队原因写「{SERVER.RUNNING_BOOKED_SUM_TEXT}」；实际上限 {cap} 字节、label {label!r}、"
                            f"排头 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
        if not blocked_health.get("ok"):
            problems.append(f"公式放不下（在跑的记账之和占满）时排头排着是正常的，/health 应当一直回 ok，实际 {blocked_health}")
        open(paused, "w").close()
        time.sleep(PAUSE_SETTLE_SECONDS)
        world.resources(local=(FORMULA_HEAD_ROOMY_GIBIBYTES, FORMULA_HEAD_ROOMY_GIBIBYTES), local_scopes={label: cap} if label else None)
        starved_health = health_until(world, lambda found: not found.get("ok"), 10)
        if starved_health.get("ok") or STARVED_HEALTH_TEXT not in (starved_health.get("problem") or ""):
            problems.append(f"换成 {FORMULA_HEAD_ROOMY_GIBIBYTES}G 的余量、公式放得下而排队线程判能起不起：/health 应当回不健康、问题写「{STARVED_HEALTH_TEXT}」，实际 {starved_health}")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(paused)
        open(release, "w").close()
    finished = [world.finished(task_id) for task_id in (blocker, head) if task_id is not None]
    failed = [describe(found) for found in finished if found["task"].get("outcome") != "succeeded"]
    if failed:
        problems.append(f"放开之后都应当跑完，实际没成的 {failed}")
    if problems:
        raise SelftestFailure("判排头放不放得下照公式：" + "；".join(problems), world.server_log_lines())
    return (f"在跑的记账之和占满（公式剩 8G、排头要记 12G）时 /health 一直回 ok；公式放得下而没起时报「{starved_health.get('problem')}」；放开之后都跑完")


def cell_retry_goes_where_it_fits(world):
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-retry-anywhere", 8 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,run", "fake-peer-exits=250,run"],
                           extra={lib.KEY_VARIABLE: "selftest-retry-anywhere"}, cwd=world.repo)
    summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    attempts = summary["attempts"]
    if summary["task"].get("next_host") or len(attempts) < 2 or "钉在同一台" in (summary["task"].get("note") or ""):
        raise SelftestFailure(f"撞上限重排不钉台：第二次照剩余占比挑台、任务不记 next_host、说明不写「钉在同一台」，实际 {describe(summary)}、note「{summary['task'].get('note')}」",
                              world.server_log_lines())
    return f"撞上限重排不钉台：第 1 次 {attempts[0].get('host')}、第 2 次 {attempts[1].get('host')}，任务不记 next_host"


def cell_cpu_unread_not_stalled(world):
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})   # 假 scope 是空目录：CPU 读不到
    summary = world.finished(world.submit(["bash", "-c", "echo started; sleep 6"]), timeout=40)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志过了阈值不涨而 CPU 读不到的那一条应当照跑完（读不到不等于不涨），实际 {describe(summary)}", world.server_log_lines())
    return "日志过了阈值不涨、CPU 读不到的照跑完（读不到不等于不涨）"


DUTY_SPREAD_SCRIPT = "for i in $(seq 8); do echo spread $i; sleep 0.25; done"   # three-duties-two-sessions 分到两台的那 6 条（记的用时与交的命令逐词相同）


def cell_three_duties_two_sessions(world):
    """服务的三件职责放进同一个模拟世界：① 两台分任务 ② 没满时限的排头放不下时后面放得下的照起，排头空出来之后照起 ③ 跑不了的打回提交它的那个会话、别的会话看不见。"""
    world.seed_peak("selftest-duty-spread", 8 * GIBIBYTE)
    world.seed_duration("selftest-duty-spread", 2, ["bash", "-c", DUTY_SPREAD_SCRIPT, "spread", "fake-peer-run"])
    world.seed_peak("selftest-duty-blocker", 16 * GIBIBYTE)
    world.seed_duration("selftest-duty-blocker", 600, ["bash", "-c", BLOCK_SCRIPT, "block", os.path.join(world.root, "release")])   # 占着的那条上次跑了 600 秒：剩余时长算得出
    world.seed_peak("selftest-duty-head", 20 * GIBIBYTE)
    world.seed_peak("selftest-duty-small", 100 * MEBIBYTE)
    world.seed_duration("selftest-duty-small", 1, ["bash", "-c", "exit 0"])   # 小的上次跑了 1 秒：比占着的那条的剩余短，排头没起时许插（jump_refusal）
    world.resources(local=(30, 30), peer=(30, 30), peer_cap_gibibytes=30)
    world.start_server()
    problems = []
    # ① 会话 A 交 6 条 8G：一台 30G 装不下，两台都要用上
    spread = [world.submit(["bash", "-c", DUTY_SPREAD_SCRIPT, "spread", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-duty-spread"}, cwd=world.repo) for _ in range(6)]
    hosts = [(world.finished(task_id, timeout=120)["attempts"] or [{}])[-1].get("host") for task_id in spread]
    if not {"local", "peer"} <= set(hosts):
        problems.append(f"① 6 条 8G 应当两台都分到，实际落在 {hosts}")
    # ② 第二台只剩 10G、本机被 16G 占着：会话 B 的 20G 排头（没满时限）放不下，会话 A 后交的小任务照起；放出来之后排头照起
    world.resources(local=(30, 30), peer=(10, 30), peer_cap_gibibytes=30)
    release = os.path.join(world.root, "release")
    blocker = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=SESSION_B, extra={lib.KEY_VARIABLE: "selftest-duty-blocker"})
    world.running(blocker)
    head = world.submit(["bash", "-c", "exit 0"], session=SESSION_B, extra={lib.KEY_VARIABLE: "selftest-duty-head"}, cwd=world.repo)
    small = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: "selftest-duty-small"}, cwd=world.repo)
    try:
        small_summary = world.finished(small, timeout=PLACEMENT_WAIT_SECONDS)
    except SelftestFailure as failure:
        small_summary = None
        problems.append(f"② 排头没满时限而放不下时后面放得下的小的应当照起、跑完，实际 {failure}")
    head_state = world.summary(head)["task"].get("state")
    open(release, "w").close()
    if small_summary is not None and (small_summary["task"].get("outcome") != "succeeded" or head_state != "queued"):
        problems.append(f"② 小的应当跑完、放不下的排头照排着，实际小的 {describe(small_summary)}、排头 {head_state}")
    head_summary = world.finished(head)
    head_start = float((head_summary["attempts"] or [{}])[0].get("started_at") or 0)
    small_start = float(((small_summary or {}).get("attempts") or [{}])[0].get("started_at") or 0)
    if head_summary["task"].get("outcome") != "succeeded" or not head_start:
        problems.append(f"② 本机放出来之后排头应当起来跑完，实际 {describe(head_summary)}")
    # ③ 会话 A 交一条跑不了的（入口会编的声明写坏了：派活时打回 run-condition-unmet）、会话 B 交一条命令自己失败的：各自的 wait 拿到各自的类型，status 只列自己的
    world.resources(local=(30, 30), peer=(10, 30), peer_cap_gibibytes=30)
    unrunnable = world.submit(write_unmet_script(world, "duty-unmet.sh"))
    failing = world.submit(["bash", "-c", "echo started; sleep 1; exit 3"], session=SESSION_B)   # 跑到一半出错
    world.finished(unrunnable), world.finished(failing)
    for session, own, other in ((SESSION_A, unrunnable, failing), (SESSION_B, failing, unrunnable)):   # wait 收下之前：各自只看得见自己的
        listed = {item.get("id") for item in json.loads(world.client(["status", "--json"], session=session).stdout or "{}").get("tasks", [])}
        if own not in listed or other in listed:
            problems.append(f"③ {session} 的 status 应当列自己的任务 {own}、不列别的会话的 {other}，实际列了 {sorted(listed)}")
    for task_id, session, outcome, code in ((unrunnable, SESSION_A, "run-condition-unmet", None), (failing, SESSION_B, "command-failed", 3)):
        waited = world.client(["wait", str(task_id)], session=session)
        if f"type={outcome}" not in waited.stdout + waited.stderr or (code is not None and waited.returncode != code) or waited.returncode == 0:
            problems.append(f"③ 任务 {task_id} 的 wait 应当退非 0{('、退 ' + str(code)) if code else ''}、末行 type={outcome}，实际退 {waited.returncode}、"
                            f"「{(waited.stdout + waited.stderr).strip()[-200:]}」")
    if problems:
        raise SelftestFailure("三件职责：" + "；".join(problems), world.server_log_lines())
    return (f"① 6 条 8G 分在 {sorted(set(hosts))}；② 排头放不下时小的先起（{small_start:.2f}）、本机放出来之后排头起（{head_start:.2f}）；"
            f"③ 跑不了的打回会话 A、命令失败的打回会话 B，各自 wait 拿到类型、status 互不可见")


def cell_hundred_five_minute_tasks_balanced(world):
    """100 条记录用时 5 分钟的任务一齐放出来、占着到放行为止（两台各 400G，100 条都放得下）：起的时候没有一条收尾，挑台只看记账，
    两台各分四成到六成；判定不随机器忙闲变（不比秒数，也不靠哪条先跑完）。"""
    world.seed_peak("selftest-hundred", 2 * GIBIBYTE)
    command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "hundred", HUNDRED_RELEASE, f"fake-peer-wait={HUNDRED_RELEASE}"]
    world.seed_duration("selftest-hundred", 300, command)
    world.resources(local=(400, 400), peer=(400, 400), peer_cap_gibibytes=400, hold=True)
    world.start_server()
    tasks = [world.submit(command, extra={lib.KEY_VARIABLE: "selftest-hundred"}, cwd=world.repo) for _ in range(100)]
    world.resources(local=(400, 400), peer=(400, 400), peer_cap_gibibytes=400)
    world.api("POST", "/wake")
    try:
        hosts = [first_attempt(world.wait_for(task_id, lambda found: bool(found["attempts"]), "起来")).get("host") for task_id in tasks]
    finally:
        open(os.path.join(world.repo, HUNDRED_RELEASE), "w").close()
    outcomes = [world.finished(task_id, timeout=120)["task"].get("outcome") for task_id in tasks]
    counts = {host: hosts.count(host) for host in ("local", "peer")}
    if sum(counts.values()) != 100 or not all(40 <= count <= 60 for count in counts.values()) or set(outcomes) != {"succeeded"}:
        raise SelftestFailure(f"100 条 5 分钟的任务两台应当各分四成到六成、都跑完，实际 {counts}（落点 {hosts}）、结局 {sorted(set(outcomes))}", world.server_log_lines())
    return f"100 条 5 分钟的任务两台分到 {counts}"


def backdate_past_aged(world, task_ids):
    """服务停着时把这几条的提交时刻往前拨过服务真的时限（AGED_WAIT_SECONDS）：只有它们满时限，之后照真时刻交的离满时限还远，判定不看交得快不快。"""
    world.stop_server()
    for task_id in task_ids:
        world.rewrite_signed_task(task_id, lambda record: record.update(submitted_at=record["submitted_at"] - SERVER.AGED_WAIT_SECONDS - AGED_BACKDATE_MARGIN_SECONDS))


def attempt_order(summary, number=1):
    """第 number 次尝试的记录号（服务起尝试时按先后编的号：同一个库里比得出谁先起，与机器忙闲、交得快慢无关）；那一次没起交 None。"""
    attempts = summary["attempts"]
    return attempts[number - 1].get("id") if len(attempts) >= number else None


def cell_aged_head_at_slice_total_starts(world):
    """满时限的排头要记满一整台：峰值 48G，正好是本机 slice 总上限 50G 减 slice 自己挂着回收不掉的那一截（这台此刻的余量，假余量 2G），上限与记账就是 48G，
    照样起得来、跑完（task_server.py 的 booked_at_cap 与 host_limit）；记满总上限的那种写法永远放不下（slice 放得下的减了余量只剩 48G）。看排两轮之后起没起，不比秒数。"""
    world.resources(local=(50, 50), hold=True)
    world.seed_peak("selftest-head-at-total", 48 * GIBIBYTE)
    world.start_server()
    head = world.submit(["bash", "-c", "exit 0", "head-at-total"], extra={lib.KEY_VARIABLE: "selftest-head-at-total"})
    backdate_past_aged(world, [head])
    world.resources(local=(50, 50))
    world.start_server()
    world.next_pass()
    world.next_pass()
    summary = world.summary(head)
    if not summary["attempts"]:
        world.client(["cancel", str(head)])
        raise SelftestFailure(f"满时限的排头记满一整台、slice 自己挂着余量 {SELFTEST_MARGIN_GIBIBYTES}G：排两轮之后应当已起（上限收到 {AT_TOTAL_CAP_PINNED_TEXT}），"
                              f"实际还排着，排队原因「{summary.get('waiting_for')}」", world.server_log_lines())
    summary = world.finished(head)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    if caps != [AT_TOTAL_CAP_PINNED_TEXT] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"满时限的排头应当以上限 {AT_TOTAL_CAP_PINNED_TEXT}（总上限 50G 减这台的假余量 2G）起、跑完，实际上限 {caps}、{describe(summary)}", world.server_log_lines())
    return f"满时限的排头记满一整台、slice 自己挂着余量 {SELFTEST_MARGIN_GIBIBYTES}G：上限收到 {AT_TOTAL_CAP_PINNED_TEXT} 起来、跑完"

def cell_aged_task_judged_first(world):
    """本机 30G 只放得下一条 20G：先交的那条没满时限、后交的那条满时限，满时限的先判、先起，先交的那条等它跑完。"""
    world.resources(local=(30, 30), hold=True)
    world.seed_peak("selftest-aged-young", 20 * GIBIBYTE)
    world.seed_peak("selftest-aged-old", 20 * GIBIBYTE)
    world.start_server()
    young = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", "release-young"], extra={lib.KEY_VARIABLE: "selftest-aged-young"}, cwd=world.work)
    old = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", "release-old"], extra={lib.KEY_VARIABLE: "selftest-aged-old"}, cwd=world.work)
    backdate_past_aged(world, [old])
    world.resources(local=(30, 30))
    world.start_server()
    try:
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        started = []
        while patience.waiting() and not started:
            started = [task_id for task_id in (young, old) if world.summary(task_id)["attempts"]]
            time.sleep(POLL_SECONDS)
        world.next_pass()   # 先起的那一条之后再排过整轮：没满时限的那条也起了的话这里看得见
        world.next_pass()
        started = [task_id for task_id in (young, old) if world.summary(task_id)["attempts"]]
        shown = describe(world.summary(old))
    finally:
        for name in ("release-young", "release-old"):
            open(os.path.join(world.work, name), "w").close()
    for task_id in (young, old):
        world.finished(task_id)
    if started != [old]:
        raise SelftestFailure(f"本机只放得下一条：满时限的任务 {old}（后交）应当先起、没满时限的任务 {young}（先交）等着，实际先起的 {started}（满时限那一条 {shown}）",
                              world.server_log_lines())
    return f"满时限的任务 {old} 虽然后交、先判先起，先交的任务 {young} 等它跑完"


def cell_peer_starts_throttled(world):
    """本机放不下、10 条一齐派到第二台：每一轮排队至多在第二台起 PEER_STARTS_PER_PASS 条（数服务记的「哪一轮起的」，不比秒数）。"""
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64, hold=True)   # 本机放不下：10 条都去第二台
    world.seed_peak("selftest-peer-burst", 2 * GIBIBYTE)
    world.start_server()
    tasks = [world.submit(["bash", "-c", "sleep 0.5", "burst", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-burst"}, cwd=world.repo) for _ in range(10)]
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64)   # 一齐放出来
    world.api("POST", "/wake")
    summaries = [world.finished(task_id, timeout=120) for task_id in tasks]
    starts = {}
    for line in world.scheduler_reads():
        found = re.match(r"^start task=(\d+) host=(\w+) view=(\d+)$", line)
        if found and int(found.group(1)) in tasks:
            starts.setdefault(int(found.group(3)), []).append((int(found.group(1)), found.group(2)))
    per_pass = {view: len(started) for view, started in sorted(starts.items())}
    hosts = {first_attempt(summary).get("host") for summary in summaries}
    if (hosts != {"peer"} or any(summary["task"].get("outcome") != "succeeded" for summary in summaries) or sum(per_pass.values()) != len(tasks)
            or max(per_pass.values(), default=0) > SERVER.PEER_STARTS_PER_PASS):
        raise SelftestFailure(f"10 条一齐派到第二台：每一轮至多起 {SERVER.PEER_STARTS_PER_PASS} 条、都跑完，实际落在 {hosts}、各轮起的条数 {per_pass}、结局 "
                              f"{[summary['task'].get('outcome') for summary in summaries]}", world.server_log_lines())
    return f"10 条一齐派到第二台：分 {len(per_pass)} 轮起（各轮 {list(per_pass.values())} 条），都跑完"


def cell_queued_shows_waiting_reason(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    time.sleep(1)
    shown = world.client(["status", str(task_id)])
    listed = world.client(["status", "--json"])
    items = [item for item in json.loads(listed.stdout or "{}").get("tasks", []) if item.get("id") == task_id]
    world.resources(local=(30, 30))
    world.finished(task_id)
    if "在等：自证的假余量写了 hold" not in shown.stdout or not items or "hold" not in (items[0].get("waiting_for") or ""):
        raise SelftestFailure(f"放不下而排着的任务 status 应当报在等什么（「在等：自证的假余量写了 hold」），实际「{shown.stdout.strip()[:300]}」、列表 {items}")
    return "排着的任务 status 报「在等：自证的假余量写了 hold」，列表里带 waiting_for"


def recorded_host_line(world, task_id):
    """客户端 status <号> 打的第一次尝试那一行（「第 1 次：本机、上限 …」）；没有交空串。"""
    shown = world.client(["status", str(task_id)])
    return next((line.strip() for line in shown.stdout.splitlines() if "第 1 次：" in line), "")


def cell_balanced_when_both_fit(world):
    world.resources(local=(40, 40), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-long-and-big", 10 * GIBIBYTE)
    world.seed_duration("selftest-long-and-big", 3600, ["bash", "-c", "exit 0"])
    world.start_server()
    placed, lines = {}, {}
    for name, extra in (("plain", {}), ("long-and-big", {lib.KEY_VARIABLE: "selftest-long-and-big"})):
        summary = world.finished(world.submit(["bash", "-c", "exit 0"], extra=extra, cwd=world.repo))
        placed[name] = summary["attempts"][0].get("host") if summary["attempts"] else None
        lines[name] = recorded_host_line(world, summary["task"]["id"])
    if placed != {"plain": "peer", "long-and-big": "peer"} or not all("第 1 次：第二台" in line for line in lines.values()):
        raise SelftestFailure(f"本机总上限 40G、第二台 64G，两台都放得下、都派得到：放下之后第二台剩余占比更大，两条都应当去第二台、status 记「第二台」，"
                              f"实际 {placed}、status 那一行 {lines}", world.server_log_lines())
    return "两台都放得下：挑放下之后剩余占比大的第二台，status 记「第二台」"


def cell_peer_when_local_full(world):
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-overflow", 10 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: "selftest-overflow"}, cwd=world.repo)
    try:
        summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    except SelftestFailure as failure:
        raise SelftestFailure(f"本机此刻只空 8G、放不下预测 10G 的这一条（slice 30G，等得到），第二台空 100G 放得下：应当派到第二台跑完，实际 {failure}",
                              failure.extra_lines) from None
    host = summary["attempts"][0].get("host") if summary["attempts"] else None
    line = recorded_host_line(world, task_id)
    if host != "peer" or summary["task"].get("outcome") != "succeeded" or "第 1 次：第二台" not in line:
        raise SelftestFailure(f"本机此刻放不下、第二台放得下：应当派到第二台、status 记「第二台」，实际 {describe(summary)}、status 那一行「{line}」", world.server_log_lines())
    return "本机此刻放不下（空 8G、要 10G）、第二台放得下：派到第二台，status 记「第二台」"


def cell_every_command_reaches_peer(world):
    """没有只派一台的名单（task_server.py 的 RESTORED_HOST_LISTS 只是删之前的样子）：删之前只在本机的（scripts/hosts/、.claude/gate.d/ 下的与读本机东西的那几个入口）
    与第二台在同一棵共享的树里跑之后放开的那几类（task-server/、scripts/agents/、.claude/hooks/、带 --write 的、scripts/kb/、合入后验证 merge-and-verify、
    改主工作区的 hook-swap.py），本机放不下（各记 15G、本机空 8G）、第二台放得下时都派到第二台跑完；删之前只在第二台跑的 e185-r2-segment-run.sh，第二台用不了时在本机跑完。
    弄坏开关 workspace-writers-local-only 下改主工作区的 hook-swap.py 只派本机、排着，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-every-command", 10 * GIBIBYTE)
    world.start_server()
    key = {lib.KEY_VARIABLE: "selftest-every-command"}
    commands = {"scripts/hosts/ 下的": "scripts/hosts/multi-host-run.sh", ".claude/gate.d/ 下的": ".claude/gate.d/doc-text.sh",
                "task-server/ 下的": "task-server/task_server_lib.py", "scripts/agents/ 下的": "scripts/agents/agent-cost.py",
                ".claude/hooks/ 下的": ".claude/hooks/lib_project_code_runs.py", "带 --write 的": "--write", "scripts/kb/ 下的": "scripts/kb/replace-once.py",
                "合入后验证 merge-and-verify": "scripts/gate/merge-and-verify.sh", "改主工作区的 hook-swap.py": SERVER.HOOK_SWAP_ENTRY,
                "删之前只在第二台的": SERVER.RESTORED_HOST_LISTS["peer_entries"][0]}
    commands.update({f"删之前读本机东西的入口 {entry}": entry for entry in SERVER.RESTORED_HOST_LISTS["local_entries"]})
    submitted = {name: world.submit(["bash", "-c", "exit 0", "x", word], extra=key, cwd=world.repo) for name, word in commands.items()}
    problems = []
    pending = dict(submitted)
    patience = world.patience(3 * PLACEMENT_WAIT_SECONDS)
    while pending and patience.waiting():
        for name, task_id in list(pending.items()):
            summary = world.summary(task_id)
            if summary["task"].get("state") == "finished":
                del pending[name]
                if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
                    problems.append(f"点到{name}应当派到第二台跑完（本机放不下），实际 {describe(summary)}")
        time.sleep(POLL_SECONDS)
    for name, task_id in pending.items():
        found = world.summary(task_id)
        problems.append(f"点到{name}应当派到第二台跑完，{3 * PLACEMENT_WAIT_SECONDS:.0f} 秒里没收尾：{found['task'].get('state')}、"
                        f"在等「{found.get('waiting_for') or ''}」、尝试 {len(found['attempts'])} 次")
        world.client(["cancel", str(task_id)])
    world.resources(local=(100, 100))   # 第二台此刻用不了
    entry = SERVER.RESTORED_HOST_LISTS["peer_entries"][0]
    try:
        local = world.finished(world.submit(["bash", "-c", "exit 0", entry, "peer-gone"], cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
        if first_attempt(local).get("host") != "local" or local["task"].get("outcome") != "succeeded":
            problems.append(f"第二台用不了时点到 {entry} 的应当在本机跑完，实际 {describe(local)}")
    except SelftestFailure as failure:
        problems.append(f"第二台用不了时点到 {entry} 的应当在本机跑完，实际没收尾：{failure}")
    if problems:
        raise SelftestFailure("没有只派一台的名单：" + "；".join(problems), world.server_log_lines())
    return (f"{len(commands)} 类命令（删之前只在本机的、放开的、只在第二台的）本机放不下时都派到第二台跑完；"
            f"第二台用不了时 {entry} 在本机跑完")


ALTERNATION_THRESHOLD_BYTES = int((50 * GIBIBYTE - 2 * GIBIBYTE) * SERVER.WHOLE_FRACTION)   # 50G 的台：可用上限 48G 的 85%（40.8G）


def cell_whole_starts_beside_running_fragment(world):
    """一台 50G（可用上限 48G、界 40.8G）上一条碎片（4G）长跑着，整片（峰值 42G，上限与记账 42G）放得下就起，不等那台腾空
    （任务 2461 留成孤儿的两小时里本机只用 2G、10 条排着等「腾空」）。"""
    world.resources(local=(50, 50))
    world.seed_peak("selftest-whole-beside", 42 * GIBIBYTE)
    world.start_server()
    release = os.path.join(world.root, "release-beside")
    fragment = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", release])
    world.running(fragment)
    # 整片也占着到放行为止：「exit 0」的整片起了就收尾，等「跑起来」的那一下可能一次都没看到它在跑、等满 60 秒判红（任务 4593 里这一格等满了 60 秒，尝试已记 1 次）
    whole_release = os.path.join(world.root, "release-whole-beside")
    whole = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "whole-beside", whole_release], extra={lib.KEY_VARIABLE: "selftest-whole-beside"})
    try:
        summary = world.running(whole)   # 碎片还在跑时整片就起
    finally:
        for path in (release, whole_release):
            open(path, "w").close()
    booked = (summary["attempts"] or [{}])[0].get("booked") or 0
    finished = {name: world.finished(task_id) for name, task_id in (("碎片", fragment), ("整片", whole))}
    problems = []
    if booked < ALTERNATION_THRESHOLD_BYTES:
        problems.append(f"整片应当记到界以上（{ALTERNATION_THRESHOLD_BYTES / GIBIBYTE:.1f}G），实际记 {booked / GIBIBYTE:.1f}G")
    if any(found["task"].get("outcome") != "succeeded" for found in finished.values()):
        problems.append(f"都应当跑完，实际 {({name: describe(found) for name, found in finished.items()})}")
    if problems:
        raise SelftestFailure("整片不等腾空：" + "；".join(problems), world.server_log_lines())
    return "碎片长跑着、整片放得下就起，不等那台腾空"


CPU_GATE_BUSY_KEY = "selftest-cpu-gate-busy"   # cpu-gate-holds-busy-host、health-head-cpu-gate-not-starved：库里记过 31 核的命令键（要的核数多，不走要核数少的名额）


def seed_busy_cores(world, key):
    """往这一格服务自己的库里记一次跑成、峰值段 31 核、CPU 闸线程数 31（平均在用核数也是 31）的尝试（task_server.py 的 threads_by_record 读它）：这把键的命令过 CPU 闸照 31 线程记，不走要核数少的名额。"""
    world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": None, "cores": 31.0, SERVER.PEAK_SEGMENT_CORES_COLUMN: 31.0, SERVER.CPU_GATE_THREADS_COLUMN: 31.0})


def cell_cpu_gate_holds_busy_host(world):
    """本机内存够、负载 200（32 核 × 2 = 64 放不下再加一条的线程）：命令键记过 31 核的任务排着、排队原因写「CPU：」；负载降到 0 之后照常跑完（task_server.py 的 cpu_problem）。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    seed_busy_cores(world, CPU_GATE_BUSY_KEY)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "cpu-gate"], extra={lib.KEY_VARIABLE: CPU_GATE_BUSY_KEY})
    world.next_pass()
    world.next_pass()
    waiting = world.summary(task_id)
    world.resources(local=(50, 50), local_cpu={"load1": 0.0, "procs_running": 1, "cpus": 32})
    summary = world.finished(task_id)
    reason = waiting.get("waiting_for") or ""
    if waiting["attempts"] or "CPU：" not in reason:
        raise SelftestFailure(f"负载 200 的本机上应当排着、排队原因写「CPU：」，实际尝试 {len(waiting['attempts'])} 次、排队原因「{reason}」", world.server_log_lines())
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"负载降下来之后应当跑完，实际 {describe(summary)}", world.server_log_lines())
    return f"负载 200 时排队原因写「{reason[:100]}」，降下来之后跑完"


def cell_unstarted_slice_uses_configured_total(world):
    """本机 slice 还没起（报告里 slice_total 是估的 30G）、多机配置里本机那一项 50G：记过峰值 40G 的命令照 50G 的总上限判、跑完，
    不按估的 30G 判永远放不下（task_server.py 的 unstarted_slices_filled）。上限与记账也照 50G 收：记录峰值 40G 比 50G 减余量 2G 小，上限与记账都是 40G；
    弄坏开关 unstarted-slice-estimated 下照估的 30G 减余量收到 28G（没有第二台，不判永远放不下、照收到的数起跑），这一格红。"""
    world.set_slice_cap("50G")
    world.resources(local=(50, 30), local_slice_unstarted=True)
    world.seed_peak("selftest-unstarted-slice", 40 * GIBIBYTE)
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "unstarted-slice"], extra={lib.KEY_VARIABLE: "selftest-unstarted-slice"}))
    attempt = first_attempt(summary)
    if summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != "40960M" or attempt.get("booked") != 40 * GIBIBYTE:
        raise SelftestFailure(f"slice 还没起、配置 50G、峰值 40G：应当照 50G 判、跑完、上限 40960M、记账 40G，实际 {describe(summary)}、上限 {attempt.get('memory_cap_text')}、"
                              f"记账 {attempt.get('booked')} 字节", world.server_log_lines())
    return f"slice 还没起时照配置的 50G 判，峰值 40G 的命令跑完、上限 40960M（{describe(summary)[:80]}）"


def cell_down_host_waits_not_never(world):
    """本机 slice 20G、第二台此刻用不了：记过峰值 30G 的命令只有第二台放得下，应当排着、不判永远放不下；第二台回来（50G）之后派过去跑完（task_server.py 的 place）。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-down-host", 30 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "down-host"], extra={lib.KEY_VARIABLE: "selftest-down-host"}, cwd=world.repo)
    world.next_pass()
    world.next_pass()
    waiting = world.summary(task_id)
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    summary = world.finished(task_id)
    if waiting["task"].get("outcome") or waiting["attempts"]:
        raise SelftestFailure(f"第二台用不了时只有它放得下的任务应当排着，实际 {describe(waiting)}、排队原因「{waiting.get('waiting_for')}」", world.server_log_lines())
    if summary["task"].get("outcome") != "succeeded" or (summary["attempts"] and summary["attempts"][0].get("host") != "peer"):
        raise SelftestFailure(f"第二台回来之后应当派过去跑完，实际 {describe(summary)}", world.server_log_lines())
    return f"第二台用不了时排着（「{(waiting.get('waiting_for') or '')[:80]}」），回来之后派过去跑完"


def cell_judge_error_skips_only_that_task(world):
    """一条这一轮判不了放不放得下（库里它的命令键有一条签过名、上限一栏写坏了的撞上限记录，定额度时 ValueError）：只跳过它，后面那条照判、起、跑完；
    它排着、排队原因写「这一轮判不了放不放得下」（task_server.py 的 place_queued_into）。"""
    world.resources(local=(30, 30))
    world.start_server()
    world.insert_signed_row("attempts", {"task_id": 0, "number": 1, "host": "local", "state": "ended", "session": "", "command_key": "selftest-judge-error",
                                         "outcome": "memory-cap-hit", "memory_cap": "not-a-number", "ended_at": SEEDED_ENDED_AT,
                                         "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC})
    broken_task = world.submit(["bash", "-c", "exit 0", "judge-error"], extra={lib.KEY_VARIABLE: "selftest-judge-error"})
    problems = []
    try:
        later = world.finished(world.submit(["bash", "-c", "exit 0", "after-judge-error"]), timeout=PLACEMENT_WAIT_SECONDS)
        if later["task"].get("outcome") != "succeeded":
            problems.append(f"后面那条应当跑完，实际 {describe(later)}")
    except SelftestFailure as failure:
        problems.append(f"前面一条判不了时后面那条应当照起、跑完，实际没收尾：{failure}")
    found = world.summary(broken_task)
    if found["task"].get("state") != "queued" or "这一轮判不了放不放得下" not in (found.get("waiting_for") or ""):
        problems.append(f"判不了的那条应当排着、排队原因写「这一轮判不了放不放得下」，实际 {found['task'].get('state')}、「{found.get('waiting_for')}」")
    world.client(["cancel", str(broken_task)])
    if problems:
        raise SelftestFailure("一条判不了：" + "；".join(problems), world.server_log_lines())
    return "一条判不了只跳过它（排着、写明判不了），后面那条照起跑完"


def write_unmet_script(world, name):
    """工作目录里一份会编的声明写坏了（# task-server-compiles: maybe）的假脚本：派活时打回 run-condition-unmet（task_server.py 的 judge_placement）；交提交它的命令。"""
    with open(os.path.join(world.work, name), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_COMPILES_MARK} maybe\nexit 0\n")
    return ["bash", name, "run", "fake-wrapper-exits=0"]


def cell_health_head_cpu_gate_not_starved(world):
    """/health 只读派活给排头的结论（task_server.py 的 starved_problem 读 head_verdict；任务 7568 卡在 CPU 负载闸被报「放得下却没起」）：本机内存够、负载 200
    （32 核 × 2 = 64 放不下再加一条的线程），排头（命令键记过 31 核，不走要核数少的名额）排着、排队原因写「CPU：」；过了阈值四倍那么久 /health 一直回 ok：派活判它不能起，不报；
    看进展（progress_problem）照余量也不报（这台负载过 CPU 闸）。负载降下来之后跑完。
    弄坏开关 health-head-judged-apart、health-judges-head 下 /health 另判、只判内存，blocked-head-counted-as-starved 下判不能起也算，
    progress-gates-ignored 下看进展不看 CPU 闸，/health 都报，这一格红。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    seed_busy_cores(world, CPU_GATE_BUSY_KEY)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "cpu-gate-head"], extra={lib.KEY_VARIABLE: CPU_GATE_BUSY_KEY})
    waiting = world.wait_for(task_id, lambda found: "CPU：" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued", "报 CPU 闸")
    health = health_until(world, lambda found: not found.get("ok"), 4 * SERVER.FAST_STARVED_SECONDS)
    world.resources(local=(50, 50), local_cpu={"load1": 0.0, "procs_running": 1, "cpus": 32})
    summary = world.finished(task_id)
    problems = []
    if waiting["attempts"] or "CPU：" not in (waiting.get("waiting_for") or ""):
        problems.append(f"负载 200 的本机上排头应当排着、排队原因写「CPU：」，实际 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
    if not health.get("ok"):
        problems.append(f"CPU 闸挡着时排头排着是正常的，/health 应当一直回 ok，实际 {health}")
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"负载降下来之后应当跑完，实际 {describe(summary)}")
    if problems:
        raise SelftestFailure("CPU 闸挡着的排头：" + "；".join(problems), world.server_log_lines())
    return f"CPU 闸挡着的排头：/health 一直回 ok（排队原因「{(waiting.get('waiting_for') or '')[:80]}」），负载降下来之后跑完"


                                                     # 整片之外第二台照样空着 8G


def cell_cpu_gate_small_task_slots(world):
    """CPU 闸的要核数少的名额（task_server.py 的 cpu_judgement、cpu_small_slots_used、small_slot_released；文件头 ②「CPU 闸」）：32 核（小世界的线是 64）、负载 100，
    这一条记过 1 核（命令键的记录）：这台靠名额在跑的不到 CPU_SMALL_TASK_SLOTS 条（0 条、3 条）时放行、交名额那一句（排队原因空）；
    四条都占着名额（起了 100 秒、不到 CPU_MEASURED_AFTER_SECONDS）时排着、原因以「CPU：」起头并写名额已用满；四条都起了 1000 秒、实测 5 核（超过 CPU_SMALL_TASK_CORES）的让出名额，放行。
    弄坏开关 cpu-small-slots-withheld 下不给名额（头两种也排着），cpu-small-slots-unlimited 下名额不设上限（占满四条的也放行），
    cpu-small-slot-heavy-still-counted 下实测超了的也照占（最后一种排着），这一格红。"""
    load = {"load1": 100.0, "procs_running": 100, "cpus": 32}
    problems = []
    for what, count, ages, cores, granted in (("名额一条没占", 0, None, 1.0, True), ("占了三条、都没满量测时长", 3, [100] * 3, 1.0, True),
                                              ("占满四条、都没满量测时长", 4, [100] * 4, 1.0, False), ("占满四条、都起了 1000 秒而实测 5 核", 4, [1000] * 4, 5.0, True)):
        fake, snapshot = cpu_gate_scheduler(count, load, ages=ages, cores=cores, recorded_cores=(1.0,))
        original = fake.store.running_attempts
        fake.store.running_attempts = lambda original=original: [dict(attempt, cpu_small_slot=SERVER.CPU_SMALL_SLOT_TEXT) for attempt in original()]
        why, note = SERVER.Scheduler.cpu_judgement(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if granted and (why or SERVER.CPU_SMALL_SLOT_TEXT not in note):
            problems.append(f"{what}：应当靠名额放行（排队原因空、说明写「{SERVER.CPU_SMALL_SLOT_TEXT}」），实际原因「{why}」、说明「{note}」")
        if not granted and (not why.startswith("CPU：") or "已用满" not in why or note):
            problems.append(f"{what}：应当排着（原因以「CPU：」起头并写名额已用满、没有放行说明），实际原因「{why}」、说明「{note}」")
    if problems:
        raise SelftestFailure("CPU 闸的要核数少的名额：" + "；".join(problems))
    return "名额没用满的放行并交说明；占满四条且都没满量测时长的排着；占着名额而实测超过 2 核的让出"


def attention_of_kind(world, task_id, kind):
    """接着起 wait，等到一行 kind 是这一种的提醒：交那一行拆出的栏；任务先有了结局、起了三回都没等到的交空表。"""
    for _ in range(3):
        completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
        for line in attention_lines(completed):
            fields = attention_fields(line)
            if fields.get("kind") == kind:
                return fields
        if completed.returncode != lib.ATTENTION_EXIT:
            return {}
    return {}


def cell_queued_whole_package_rechecked(world):
    """服务起来时重核在排的照交任务那一刻同一份提交时拒判（task_server.py 的 submit_refusal 与 revalidate_queued）：在排的一条绕过服务改成整包测试
    （cargo test -p 不点名测试目标；演规则加上之前交进来的那一种），重启服务：它记 resubmit-required（退 232、一次没起、说明写整包测试那一句），
    task wait 拿到这个结局；合规的那一条照排。弄坏开关 queued-refusals-not-rechecked 下只核环境变量、包装与编译目录，整包测试照排，这一格红。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    stale = world.submit(["bash", "-c", "exit 0", "rechecked-stale"])
    fine = world.submit(["bash", "-c", "exit 0", "rechecked-fine"])
    world.stop_server()
    world.rewrite_signed_task(stale, lambda record: record.update(command=["cargo", "test", "-p", "singlefs-verification"]))
    world.start_server()
    problems = []
    try:
        summary = world.finished(stale, timeout=20)
    except SelftestFailure:
        summary = world.summary(stale)
    waited = world.client(["wait", str(stale)]) if summary["task"].get("state") == "finished" else None   # 没收尾的不等（wait 会一直等下去）
    waiting = world.summary(fine)
    world.resources(local=(30, 30))
    world.finished(fine)
    if summary["task"].get("outcome") != "resubmit-required" or summary["task"].get("exit_code") != 232 or summary["attempts"] \
            or "整包测试没带确认标识" not in (summary["task"].get("note") or ""):
        problems.append(f"在排的整包测试重启之后应当记 resubmit-required（退 232、一次没起、说明写整包测试那一句），实际 {describe(summary)}、"
                        f"说明「{summary['task'].get('note')}」")
    if waited is None or "type=resubmit-required" not in waited.stdout:
        problems.append(f"task wait 应当拿到 resubmit-required 那一行，实际「{(waited.stdout.strip()[-300:] if waited else '没收尾，没等')}」")
    if waiting["task"].get("state") != "queued":
        problems.append(f"合规的那一条应当照排，实际 {waiting['task'].get('state')}")
    if problems:
        raise SelftestFailure("重启之后照同一份提交时拒重核：" + "；".join(problems), world.server_log_lines())
    return "在排的整包测试重启之后记 resubmit-required、wait 拿到结局，合规的照排"


def cell_cpu_gate_counts_running_threads(world):
    """CPU 闸一直记着在跑的线程、读不到不放行（task_server.py 的 cpu_problem；调度溢出审核 CPU-1、CPU-2）：32 核（线 64）、负载 3，在跑的与要起的这一条的命令键都记过 31 核：
    在跑一条（31 线程）再起一条 31 线程放得下；在跑两条（起了 100 秒、不到 CPU_MEASURED_AFTER_SECONDS，照记录各记 31 线程共 62，负载里看不出）再起一条超过、原因以「CPU：」起头；
    报告里没有 CPU 那一项：这一轮不派、原因以 CPU_UNREAD_TEXT 起头。
    弄坏开关 cpu-threads-settling-only 下只记刚起 5 秒的线程（两条那一种放行），cpu-unread-passed 下读不到的放行，这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, count, cpu, starts in (("在跑一条", 1, load, ""), ("在跑两条", 2, load, "CPU："), ("读不到 CPU", 0, None, SERVER.CPU_UNREAD_TEXT)):
        fake, snapshot = cpu_gate_scheduler(count, cpu, recorded_cores=(31.0,), keys=[CPU_GATE_PROBE_KEY] * count)
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if (not starts and why) or (starts and not why.startswith(starts)):
            problems.append(f"{what}：应当{'放行' if not starts else '挡住、原因以「' + starts + '」起头'}，实际「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸：" + "；".join(problems))
    return "在跑一条再起一条放得下；在跑两条再起超过；读不到 CPU 的不派"


def cell_cpu_gate_measured_cores(world):
    """CPU 闸里在跑的照实测核数记（task_server.py 的 running_threads、cpu_problem；任务 13256 排在三条 31 线程的单线程任务后面）：32 核（线 64）、负载 3，
    三条在跑、各登记 31 线程而一路实测 1 核（峰值段 1 核）、都起了 1000 秒（满 CPU_MEASURED_AFTER_SECONDS）：记 3 线程、第四条照起；其中一条只起了 30 秒、命令键记过 31 核：它照记录记 31、共 33 线程。
    弄坏开关 cpu-threads-registered-only 下一律照登记数记（共 93），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, ages, threads in (("三条都满了时长", [1000, 1000, 1000], 3), ("一条只起了 30 秒", [1000, 1000, 30], 33)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=ages, cores=1.0, recorded_cores=(31.0,), keys=[None, None, CPU_GATE_PROBE_KEY])
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1}, SERVER.LOCAL, snapshot)
        if counted != threads or why:
            problems.append(f"{what}：在跑的应当记 {threads} 线程、第四条照起，实际记 {counted} 线程、「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸照实测核数记在跑的：" + "；".join(problems))
    return "满了时长的照实测 1 核记、第四条照起；没满的照命令键的记录 31 记"


def cell_cpu_gate_new_task_by_recorded_cores(world):
    """CPU 闸里新起的这一条照同一命令键近几次跑成的峰值段核数取最大记（task_server.py 的 admission_threads、threads_by_record、cpu_problem；任务 13489 单线程的小脚本照 31 线程排了 30 分钟）：
    32 核（线 64）、负载 3，三条在跑、都起了 1000 秒、实测各 17.5 核（记 18、共 54 线程）。库里跑成三次、峰值段 1、1、0.7 核的那一条记 1 线程，54 + 1 不超过 64，照起；
    库里跑成三次、各 20 核的那一条记 20 线程，54 + 20 超过 64，排着、原因写「20 线程」。弄坏开关 cpu-wanted-by-registered-threads 下有记录的也照 31 记（头一种排着），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, recorded, blocked in (("记过最大 1 核的", (1.0, 1.0, 0.7), False), ("记过 20 核的", (20.0, 20.0, 20.0), True)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=[1000, 1000, 1000], cores=17.5, recorded_cores=recorded)
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if counted != 54 or bool(why) != blocked or (blocked and "（20 线程" not in why):
            problems.append(f"{what}：在跑的应当记 54 线程、这一条{'照 20 线程排着、原因写「20 线程」' if blocked else '照 1 线程起'}，实际记 {counted} 线程、「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸照记录的核数算新起的：" + "；".join(problems))
    return "在跑 54 线程：记过最大 1 核的照 1 线程起；记过 20 核的照 20 线程排着"


def cell_cpu_gate_new_task_counts_failed_runs(world):
    """CPU 闸里新起的这一条照历史核数记时，命令自己退非 0、跑到头的几次也算（task_server.py 的 threads_by_record；弄坏开关轮这类按设计退非 0 的命令也照记录记）：
    32 核（线 64）、负载 3，三条在跑、都起了 1000 秒、实测各 17.5 核（记 18、共 54 线程）。库里三次 command-failed（退 1）各记 20 核的那一条记 20 线程，54 + 20 超过 64，排着；
    一次 command-failed 退 143（被 TERM 杀的）、一次 killed-externally、一次 memory-cap-hit 各记 20 核的不算，只剩一次跑成的 1 核，记 1 线程，照起（算进去最大 20 核、排着）；
    跑成两次（1.0 核）加一次 command-failed（退 1、0.3 核）最大 1 核，记 1 线程，照起。
    弄坏开关 threads-record-succeeded-only 下只认跑成的：头一种没有记录、照登记的 31 线程排着、原因里不是「20 线程」，这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    failed = {"outcome": "command-failed", "exit_code": 1, "cores": 20.0}
    unfinished = [{"outcome": "command-failed", "exit_code": 143, "cores": 20.0}, {"outcome": "killed-externally", "exit_code": 1, "cores": 20.0},
                  {"outcome": "memory-cap-hit", "exit_code": 250, "cores": 20.0}, {"outcome": "succeeded", "exit_code": 0, "cores": 1.0}]
    problems = []
    for what, rows, blocked in (("三次命令自己退 1", [failed] * 3, True), ("被信号杀的、外部杀的、撞上限的加一次跑成", unfinished, False),
                                ("跑成两次加一次退 1", [{"outcome": "succeeded", "exit_code": 0, "cores": 1.0}] * 2 + [dict(failed, cores=0.3)], False)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=[1000, 1000, 1000], cores=17.5, recorded_rows=rows)
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if bool(why) != blocked or (blocked and "（20 线程" not in why):
            problems.append(f"{what}：这一条应当{'照 20 线程排着、原因写「20 线程」' if blocked else '照 1 线程起'}，实际「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸照历史核数记新起的（退非 0 跑到头的也算）：" + "；".join(problems))
    return "历史里命令自己退非 0 的三次照 20 线程排着；被信号杀的、外部杀的、撞上限的不算，照跑成的 1 线程起；跑成加退 1 的混着算"


def cell_cpu_gate_unrecorded_by_registered_threads(world):
    """CPU 闸里没有核数记录的新命令量到之前按登记的线程数记（task_server.py 的 threads_by_record；规则「按 31 核，量到后改实测」；按 1 核算时任务 15262 到 15299
    共 38 条新命令十秒内齐起）：命令键库里一次跑到头的都没有、登记 31 线程的记 31 线程，那一句写「先按登记的 31 线程」；没有命令键的同样；登记 4 线程的记 4。
    库里记过 5 核的照记录记 5。套在假调度器上，不起服务。弄坏开关 cpu-unrecorded-by-one-core 下没有记录的记 1 线程，这一格红。"""
    fake, _ = cpu_gate_scheduler(0, {"load1": 1.0, "procs_running": 1, "cpus": 32}, recorded_cores=(5.0,))
    problems = []
    for what, key, registered, wanted in (("没有记录、登记 31", "selftest-cpu-gate-never-ran", 31, 31), ("没有命令键、登记 31", "", 31, 31),
                                          ("没有记录、登记 4", "selftest-cpu-gate-never-ran", 4, 4), ("记过 5 核", CPU_GATE_PROBE_KEY, 31, 5)):
        threads, how = SERVER.Scheduler.threads_by_record(fake, key, registered)
        if threads != wanted or (wanted == registered and key != CPU_GATE_PROBE_KEY and f"先按登记的 {registered} 线程" not in how):
            problems.append(f"{what}：应当记 {wanted} 线程，没有记录的那一句写「先按登记的 {registered} 线程」，实际 {threads}、「{how}」")
    if problems:
        raise SelftestFailure("CPU 闸里没有记录的新命令按登记线程数：" + "；".join(problems))
    return "没有核数记录的新命令（连同没有命令键的）量到之前按登记的线程数记：登记 31 的记 31、登记 4 的记 4；记过 5 核的照记录记 5"


def cell_cpu_gate_running_after_restart_by_average(world):
    """服务刚重启、还没量满一段时，起跑已满两分钟的在跑尝试照累计 CPU ÷ 已跑秒数记（task_server.py 的 running_threads）：两条登记 31 线程、没有命令键、起了 1000 秒、一路 3.2 核，
    读数表是空的（演服务重启）：各记 4 线程、共 8；另一条只起了 30 秒的照旧按登记的 31 线程记。套在假调度器上，不起服务。
    弄坏开关 cpu-restart-unmeasured-by-record 下起了 1000 秒的两条各记 31 线程，这一格红。"""
    fake, _ = cpu_gate_scheduler(3, {"load1": 1.0, "procs_running": 1, "cpus": 32}, ages=[1000, 1000, 30], cores=3.2)
    fake.running_peak_samples, fake.running_peak_cores, fake.running_latest_cores = {}, {}, {}   # 服务重启：读数表从空的起
    total = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
    if total != 4 + 4 + 31:
        raise SelftestFailure(f"服务刚重启时的在跑线程数：起了 1000 秒、一路 3.2 核的两条应当各记 4 线程，起了 30 秒的一条记登记的 31 线程，共 39，实际 {total}")
    return "读数表是空的（服务刚重启）：起了 1000 秒、一路 3.2 核的两条各记 4 线程，起了 30 秒的照登记的 31 线程，共 39"


def cell_cpu_gate_running_by_latest_reading(world):
    """CPU 闸里在跑、量满了一段的那一条照最近一段的在用核数向上取整、与最近一回的可跑线程数取大，不超过登记数（task_server.py 的 latest_running_cores；规则
    「各个任务的负载的取值要动态的取」）：一条登记 31 线程、起了 1000 秒、一路 17.5 核——没有线程读数时记 18；最近一回可跑 25 个的记 25；可跑 40 个的收到登记的 31。
    套在假调度器上，不起服务。弄坏开关 cpu-latest-runnable-ignored 下可跑 25 个的记 18，这一格红。"""
    fake, _ = cpu_gate_scheduler(1, {"load1": 1.0, "procs_running": 1, "cpus": 32}, ages=[1000], cores=17.5)
    attempt = fake.store.running_attempts()[0]
    now = time.time()
    elapsed = now - attempt["started_at"]
    usage = fake.machines[SERVER.LOCAL].cpu_usec(attempt)
    problems = []
    for runnable, wanted in ((0, 18), (25, 25), (40, 31)):
        first = SERVER.Scheduler.latest_running_cores(fake, attempt, usage, elapsed, now, 31)   # 先算一回：记下最近一段的在用核数
        fake.running_latest_runnable[attempt["id"]] = runnable
        found = SERVER.Scheduler.latest_running_cores(fake, attempt, usage, elapsed, now, 31)
        if found != wanted:
            problems.append(f"最近一回可跑 {runnable} 个：应当记 {wanted} 线程，实际 {found}（头一回算出 {first}）")
    if problems:
        raise SelftestFailure("CPU 闸里在跑的照最近的读数记：" + "；".join(problems))
    return "在跑、一路 17.5 核的一条：没有线程读数记 18，最近一回可跑 25 个记 25，可跑 40 个收到登记的 31"


def cell_cpu_gate_running_started_by_recorded_cores(world):
    """CPU 闸里在跑却还没实测到核数的，照同一命令键近几次跑成的平均在用核数记（task_server.py 的 running_threads、unmeasured_threads、threads_by_record）：
    32 核（线 64）、负载 3，三条在跑、各登记 31 线程、都只起了 30 秒（不到 CPU_MEASURED_AFTER_SECONDS），其中两条的命令键库里记过 1、1、0.7 核（取最大记 1 线程），
    一条的命令键记过 20 核（记 20）：共 1 + 1 + 20 = 22 线程，第四条照起；起了 1000 秒而假本机量不到 CPU 的同样三条也记 22；
    在跑名单里不在了的尝试，存的那一份在下一次算时删。没有记录的在跑尝试量到之前按登记的线程数记，由 cpu-gate-unrecorded-by-registered-threads 钉着。
    弄坏开关 cpu-running-unmeasured-registered 下没实测到的一律照登记数记（共 93），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    keys = [CPU_GATE_PROBE_KEY, CPU_GATE_PROBE_KEY, CPU_GATE_HEAVY_KEY]
    problems = []
    for what, ages, cores in (("三条都只起了 30 秒", [30, 30, 30], 1.0), ("起了 1000 秒而量不到 CPU", [1000, 1000, 1000], None)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=ages, cores=cores, recorded_cores=(1.0, 1.0, 0.7), keys=keys, heavy_cores=(20.0,))
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 9, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if counted != 22 or why:
            problems.append(f"{what}：在跑的应当记 1 + 1 + 20 = 22 线程、第四条照起，实际记 {counted} 线程、「{why}」")
    fake, snapshot = cpu_gate_scheduler(3, load, ages=[30, 30, 30], keys=keys, recorded_cores=(1.0, 1.0, 0.7), heavy_cores=(20.0,))
    SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
    kept_before = sorted(fake.running_threads_cache)
    remaining = fake.store.running_attempts()[:1]
    fake.store.running_attempts = lambda: remaining
    SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
    if kept_before != [1, 2, 3] or sorted(fake.running_threads_cache) != [1]:
        problems.append(f"存的取法按尝试号：算过三条应当存 [1, 2, 3]，只剩尝试 1 在跑之后下一次算应当只留 [1]，实际 {kept_before}、{sorted(fake.running_threads_cache)}")
    if problems:
        raise SelftestFailure("CPU 闸照记录的核数记没实测到的在跑尝试：" + "；".join(problems))
    return "起了 30 秒与量不到 CPU 的在跑三条：记过 0.9 核的各记 1、记过 20 核的记 20，共 22 线程、第四条照起；存的按尝试号、收尾的删"


def cell_cpu_gate_running_average_capped_by_registered(world):
    """实测到的在跑尝试在 CPU 闸里记的核数不超过它登记的线程数，峰值段那一支与窗口那一支同一个上界（task_server.py 的 running_threads、latest_running_cores）：
    32 核（线 64）、负载 3，三条在跑、都起了 1000 秒、一路实测 40 核（峰值段 40 核；命令里编译器、子进程占的比登记线程多）：各登记 4 线程的记 4、共 12；各登记 31 线程的记 31、共 93
    （不记 40、共 120）；不到登记数的照峰值段记（2.5 核、登记 4 线程：记 3、共 9）。
    弄坏开关 cpu-running-cores-over-registered 下峰值段那一支不限登记数（共 120、共 120），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for registered, cores, expected in ((4, 40.0, 12), (31, 40.0, 93), (4, 2.5, 9)):
        fake, _ = cpu_gate_scheduler(3, load, ages=[1000, 1000, 1000], cores=cores)
        listed = fake.store.running_attempts
        fake.store.running_attempts = lambda listed=listed, registered=registered: [dict(attempt, threads=registered) for attempt in listed()]
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        if counted != expected:
            problems.append(f"三条各登记 {registered} 线程、峰值段 {cores:g} 核：应当共记 {expected} 线程，实际 {counted}")
    if problems:
        raise SelftestFailure("CPU 闸记在跑尝试的峰值段核数不超过登记数：" + "；".join(problems))
    return "峰值段 40 核的：登记 4 线程的记 4、登记 31 线程的记 31；2.5 核、登记 4 线程的照峰值段记 3"


def cell_finish_peak_segment_cpu_unread(world):
    """收尾时累计 CPU 读不到、或接口交的不是数：峰值段核数记 None、结局照记（task_server.py 的 finish_peak_segment_cores、cpu_reading、finish_attempt）。
    直接算：读到 None、一个别的对象、一串字、NaN 的，一段都没量满过的都交 None、不抛错；量满过一段（各段最大 7.5）而这一刻读不到的交 7.5。
    整条跑：假 scope 只写内存那几份、没有 cpu.stat（服务一路读不到 CPU），交一条退 0 的：结局 succeeded、退 0，尝试的 peak_segment_cores 是 None。
    弄坏开关 peak-segment-reading-unchecked 下接口交的东西不核、照数用（别的对象、一串字抛错，NaN 交 NaN），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.running_usage_lock = threading.Lock()
    fake.running_peak_samples, fake.running_peak_cores = {}, {}
    attempt = {"id": 1, "host": SERVER.LOCAL, "started_at": time.time() - 30}
    problems = []
    for what, value, measured, expected in (("读到 None", None, None, None), ("接口交了一个别的对象", object(), None, None), ("接口交了一串字", "2000000", None, None),
                                            ("读到 NaN", float("nan"), None, None), ("量满过一段、这一刻接口交了别的对象", object(), 7.5, 7.5)):
        fake.running_peak_samples.clear()
        fake.running_peak_cores.clear()
        if measured is not None:
            fake.running_peak_cores[attempt["id"]] = measured
        machine = types.SimpleNamespace(cpu_usec=lambda attempt, value=value: value)
        try:
            recorded = SERVER.Scheduler.finish_peak_segment_cores(fake, attempt, machine, time.time())
        except Exception as error:   # 这一格判的就是收尾这一步抛不抛：抛了照不对记下
            problems.append(f"{what}：收尾应当照常往下走、峰值段核数记 {expected}，实际抛了 {type(error).__name__}: {error}")
            continue
        if recorded != expected:
            problems.append(f"{what}：峰值段核数应当记 {expected}，实际 {recorded}")
    scope = os.path.join(world.root, "unread-cpu-scope")
    os.makedirs(scope)
    for name, text in (("memory.stat", process_memory_stat(300 * MEBIBYTE)), ("memory.peak", f"{300 * MEBIBYTE}\n")):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    done = world.finished(world.submit(["bash", "-c", "exit 0", "unread-cpu"]))
    attempts = done["attempts"]
    if done["task"].get("outcome") != "succeeded" or done["task"].get("exit_code") != 0 or len(attempts) != 1 or attempts[0].get(SERVER.PEAK_SEGMENT_CORES_COLUMN) is not None:
        problems.append(f"一路读不到 CPU、退 0 的：结局应当照记 succeeded、退 0，峰值段核数记 None，实际 {describe(done)}、"
                        f"峰值段核数 {[attempt.get(SERVER.PEAK_SEGMENT_CORES_COLUMN) for attempt in attempts]}")
    if problems:
        raise SelftestFailure("收尾时累计 CPU 读不到：" + "；".join(problems), world.server_log_lines())
    return "收尾时读不到或交的不是数：峰值段核数记 None（量满过一段的记各段最大），结局照记 succeeded"


CPU_GATE_PEAK_KEY = "selftest-cpu-gate-peak-segment"   # cpu-gate-peak-segment-cores：前 240 秒 2 核、之后 30 核的那把命令键


def cell_cpu_gate_peak_segment_cores(world):
    """CPU 闸记峰值段核数、不记整次平均（task_server.py 的 note_peak_segment、latest_running_cores、running_threads、finish_peak_segment_cores、threads_by_record；
    本机 32 核上四条各登记 31 线程的崩溃点重放同时在抢核：前半段编译、录制用核少，按整次平均每条只记几核，四条都放进来）：
    一条登记 31 线程的在跑尝试前 240 秒用 2 核、之后用 30 核，服务每 60 秒读一回累计 CPU：起了 480 秒时在跑的记 30 线程（整次平均是 16）；
    收尾时记的峰值段核数是 30（CPU 闸线程数同是 30：没有线程读数），同键下一条新任务过闸照记录记 30 线程、取法那半句写「CPU 闸线程数」。
    弄坏开关 cpu-cores-by-average 下在跑的照整次平均记 16、记录照平均在用核数（cores 16）记 16，这一格红。"""
    fake, _ = cpu_gate_scheduler(0, {"load1": 2.0, "procs_running": 3, "cpus": 32})
    clock = [float(int(time.time()))]
    attempt = {"id": 1, "host": SERVER.LOCAL, "threads": 31, "started_at": clock[0], "command_key": CPU_GATE_PEAK_KEY}

    def usage(running):
        age = clock[0] - running["started_at"]
        return 1e6 * (2.0 * min(age, 240.0) + 30.0 * max(age - 240.0, 0.0))

    fake.store.running_attempts = lambda: [dict(attempt)]
    fake.machines[SERVER.LOCAL].cpu_usec = usage
    counted = None
    for age in range(0, 481, 60):
        clock[0] = attempt["started_at"] + age
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, clock[0])
    recorded = SERVER.Scheduler.finish_peak_segment_cores(fake, attempt, fake.machines[SERVER.LOCAL], clock[0])
    average = usage(attempt) / 1e6 / 480.0
    gate = SERVER.Scheduler.finish_cpu_gate_threads(fake, attempt, recorded)
    rows = [({"outcome": "succeeded", "command_key": CPU_GATE_PEAK_KEY, "cores": average, SERVER.PEAK_SEGMENT_CORES_COLUMN: recorded, SERVER.CPU_GATE_THREADS_COLUMN: gate}, True)]
    fake.store.select = lambda table, where="1", parameters=(), order="id": list(rows) if tuple(parameters) == (CPU_GATE_PEAK_KEY,) else []
    wanted, how = SERVER.Scheduler.admission_threads(fake, {"id": 2, "command_key": CPU_GATE_PEAK_KEY})
    problems = []
    if counted != 30:
        problems.append(f"起了 480 秒（前 240 秒 2 核、之后 30 核，整次平均 {average:g}）：在跑的应当记 30 线程，实际 {counted}")
    if recorded != 30.0 or gate != 30.0 or wanted != 30 or "CPU 闸线程数" not in how:
        problems.append(f"收尾记的峰值段核数与 CPU 闸线程数应当都是 30、同键新任务照它记 30 线程（「CPU 闸线程数」），实际记 {recorded}、{gate}、新任务 {wanted} 线程「{how}」")
    if problems:
        raise SelftestFailure("CPU 闸记峰值段核数：" + "；".join(problems))
    return f"前 240 秒 2 核、之后 30 核：在跑的记 30（整次平均 {average:g}）；收尾记 30，同键新任务照记录 30 线程"


def cell_cpu_gate_runnable_threads(world):
    """CPU 闸连同可跑线程数记（task_server.py 的 note_threads、peak_segment_running_cores、finish_cpu_gate_threads、threads_by_record；
    五条各要 31 核的挤在一台上各只拿到七到十核，照在用核数记，下一条同键的照十核放进来）：32 核、负载 3，一条登记 31 线程的在跑尝试起了 1000 秒、一路在用 cores 核，
    服务读到一回线程读数：在用 8 核、可跑线程 31 的在跑记 31、收尾记 cpu_gate_threads 31、同键新任务过闸照记录记 31；在用 3 核、可跑线程 2 的三样都是 3；
    读不到线程读数的照在用核数，三样都是 8。
    弄坏开关 cpu-runnable-threads-unread 下只看在用核数（头一种三样都是 8），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, cores, runnable, wanted in (("在用 8 核、可跑线程 31", 8.0, 31, 31), ("在用 3 核、可跑线程 2", 3.0, 2, 3), ("读不到可跑线程数", 8.0, None, 8)):
        fake, _ = cpu_gate_scheduler(1, load, ages=[1000], cores=cores, keys=[CPU_GATE_PEAK_KEY])
        fake.machines[SERVER.LOCAL].thread_reading = lambda attempt, runnable=runnable: None if runnable is None else SERVER.ThreadReading(1, runnable, runnable, runnable, "selftest")
        attempt = fake.store.running_attempts()[0]
        now = time.time()
        SERVER.Scheduler.note_threads(fake, attempt)
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, now)
        recorded = SERVER.Scheduler.finish_cpu_gate_threads(fake, attempt, SERVER.Scheduler.finish_peak_segment_cores(fake, attempt, fake.machines[SERVER.LOCAL], now))
        rows = [({"outcome": "succeeded", "command_key": CPU_GATE_PEAK_KEY, SERVER.CPU_GATE_THREADS_COLUMN: recorded}, True)]
        fake.store.select = lambda table, where="1", parameters=(), order="id", rows=rows: list(rows) if tuple(parameters) == (CPU_GATE_PEAK_KEY,) else []
        admitted, how = SERVER.Scheduler.admission_threads(fake, {"id": 9, "command_key": CPU_GATE_PEAK_KEY})
        if counted != wanted or recorded is None or round(recorded) != wanted or admitted != wanted:
            problems.append(f"{what}：在跑的、收尾记的与同键新任务都应当是 {wanted}，实际在跑记 {counted}、收尾记 {recorded}、新任务 {admitted}（{how}）")
    if problems:
        raise SelftestFailure("CPU 闸连同可跑线程数记：" + "；".join(problems))
    return "在用 8 核、可跑线程 31 记 31（收尾与同键新任务同是 31）；在用 3 核、可跑线程 2 记 3；读不到线程读数照在用核数 8"


THREADS_OVER_REGISTERED_BINARY = "selftest-test-binary"   # threads-over-registered-reported 里占线程最多的那个进程名


def cell_threads_over_registered_reported(world):
    """线程总数远超登记数推一回提醒（task_server.py 的 note_threads、threads_over_registered_items；文件头 ①「要处置的提醒」的 threads-over-registered；
    量到过一个测试进程开了 347 个线程、占 1780% CPU，登记 31）：一条登记 31 线程的在跑尝试，服务逐轮读线程读数（采样号、可跑的、总数）：
    连着三轮 347 个线程、可跑的 40 个，之后掉回 10 个：推一项 threads-over-registered，依据写 347、登记数 31 与占线程最多的进程名；
    同一个采样号读了三回：只算一轮，不推；连着五轮 347 个线程而可跑的只有 5 个（编译器、链接器开一堆在睡的线程）：不推；超两轮、掉一轮、再超两轮：不推。
    连着五轮 392 个线程、可跑的 39 个而占线程最多的进程只有 34 个线程（编译时一堆 rustc）：不推。
    弄坏开关 threads-over-registered-silent 下头一种不推，threads-over-registered-idle-counted 下可跑的只有 5 个的那一种也推，
    threads-over-registered-many-processes-counted 下占线程最多的进程只有 34 个线程的那一种也推，这一格红。"""
    def busy(sample, runnable=40):
        return SERVER.ThreadReading(sample, runnable, 347, 300, THREADS_OVER_REGISTERED_BINARY)

    def calm(sample):
        return SERVER.ThreadReading(sample, 2, 10, 10, THREADS_OVER_REGISTERED_BINARY)

    cases = (("连着三轮 347 个线程、可跑的 40 个，之后掉回 10 个", [busy(1), busy(2), busy(3), calm(4), calm(5)], True),
             ("同一个采样号读了三回", [busy(1), busy(1), busy(1)], False),
             ("连着五轮 347 个线程、可跑的只有 5 个", [busy(sample, runnable=5) for sample in range(1, 6)], False),
             ("连着五轮 392 个线程、可跑的 39 个、占线程最多的进程只有 34 个线程（编译）", [SERVER.ThreadReading(sample, 39, 392, 34, "cargo") for sample in range(1, 6)], False),
             ("超两轮、掉一轮、再超两轮", [busy(1), busy(2), calm(3), busy(4), busy(5)], False))
    problems = []
    for what, readings, pushed in cases:
        fake, _ = cpu_gate_scheduler(0, {"load1": 2.0, "procs_running": 3, "cpus": 32})
        attempt = {"id": 1, "host": SERVER.LOCAL, "threads": 31, "state": "running", "number": 1}
        queue = list(readings)
        fake.machines[SERVER.LOCAL].thread_reading = lambda running, queue=queue: queue.pop(0) if queue else None
        for _ in readings:
            SERVER.Scheduler.note_threads(fake, attempt)
        items = SERVER.Scheduler.threads_over_registered_items(fake, {"id": 7}, [attempt])
        basis = items[0]["basis"] if items else ""
        if pushed and (len(items) != 1 or items[0]["kind"] != "threads-over-registered" or items[0]["threshold"] != 1
                       or not all(text in basis for text in ("347", "登记数 31", THREADS_OVER_REGISTERED_BINARY))):
            problems.append(f"{what}：应当推一项 threads-over-registered、提醒门槛 1、依据写 347、登记数 31 与 {THREADS_OVER_REGISTERED_BINARY}，实际 {items}")
        if not pushed and items:
            problems.append(f"{what}：不该推，实际 {items}")
    if problems:
        raise SelftestFailure("线程总数远超登记数：" + "；".join(problems))
    return "连着三轮 347 个线程、可跑的 40 个推一回；同一个采样号、可跑的只有 5 个、中间掉过一轮的都不推"


def peer_threads_script(runnable):
    """threads-over-registered-peer-reported 的第二台那条命令：12 轮、每轮 0.5 秒，每轮打一行 MULTI_HOST_RUN_PEER_THREADS（线程总数 100000、可跑的 runnable）与一行 CPU。"""
    return ('for i in $(seq 12); do echo "MULTI_HOST_RUN_PEER_THREADS poll=$i runnable=' + str(runnable) + ' total=100000 top=90000:selftest-peer-binary read=yes"; '
            'echo "MULTI_HOST_RUN_PEER_CPU poll=$i usage_usec=$((i * 1000000)) read=yes"; sleep 0.5; done')


def cell_threads_over_registered_peer_reported(world):
    """第二台的线程读数（task_server.py 的 PEER_THREADS_LINE、log_content 的 peer_threads、PeerMachine.thread_reading）：两条派到第二台、各跑 6 秒，每轮打一行
    MULTI_HOST_RUN_PEER_THREADS total=100000：可跑的 50000 的那一条 wait 打一行 kind=threads-over-registered、依据写 100000 与 selftest-peer-binary、退 ATTENTION_EXIT，
    再起 wait 跑完；可跑的 0 的那一条不推、跑完。弄坏开关 threads-over-registered-silent 下头一条不推，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-threads", 10 * GIBIBYTE)
    world.start_server()
    busy = world.submit(["bash", "-c", peer_threads_script(50000), "busy", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-threads"}, cwd=world.repo)
    fields = attention_of_kind(world, busy, "threads-over-registered")
    problems = []
    if not fields or "100000" not in fields.get("basis", "") or "selftest-peer-binary" not in fields.get("basis", ""):
        problems.append(f"可跑的 50000 的那一条应当推 threads-over-registered、依据写 100000 与 selftest-peer-binary，实际 {fields}")
    done = world.finished(busy, timeout=60)
    host = done["attempts"][0].get("host") if done["attempts"] else None
    if host != "peer" or done["task"].get("outcome") != "succeeded":
        problems.append(f"可跑的 50000 的那一条应当在第二台跑完，实际在 {host}、{describe(done)}")
    calm = world.submit(["bash", "-c", peer_threads_script(0), "calm", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-threads"}, cwd=world.repo)
    completed = world.client(["wait", str(calm)], timeout=60)
    pushed = [line for line in attention_lines(completed) if attention_fields(line).get("kind") == "threads-over-registered"]
    if pushed or completed.returncode != 0:
        problems.append(f"可跑的 0 的那一条不该推、应当跑完退 0，实际退 {completed.returncode}、推了 {pushed}")
    if problems:
        raise SelftestFailure("第二台的线程读数：" + "；".join(problems), world.server_log_lines())
    return "第二台每轮 100000 个线程：可跑的 50000 的推 threads-over-registered（依据写 100000 与进程名）、跑完；可跑的 0 的不推、跑完"


def cell_cpu_gate_line_twice_cores(world):
    """CPU 闸的线是核数 × 倍数（task_server.py 的 cpu_judgement；小世界的倍数是 2，由 TASK_SERVER_SELFTEST_CPU_OVERSUBSCRIPTION 给）：32 核（线 64）、负载 3，在跑两条各记 31 线程（起了 100 秒、照命令键的记录 31），
    第三条 31 线程的排着，原因以「CPU：」起头、写「32 核 × 2 = 64」；在跑一条时 31 + 31 = 62 照起。
    弄坏开关 cpu-oversubscription-three 下线是 192（32 × CPU_OVERSUBSCRIPTION_LOOSE），在跑两条时第三条也照起，这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for count, blocked in ((1, False), (2, True)):
        fake, snapshot = cpu_gate_scheduler(count, load, recorded_cores=(31.0,), keys=[CPU_GATE_PROBE_KEY] * count)
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 9, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if blocked and (not why.startswith("CPU：") or "32 核 × 2 = 64" not in why):
            problems.append(f"在跑两条各 31 线程：第三条 31 线程应当排着、原因以「CPU：」起头并写「32 核 × 2 = 64」，实际「{why}」")
        if not blocked and why:
            problems.append(f"在跑一条 31 线程：再起一条 31 线程（共 62）应当照起，实际「{why}」")
    if SERVER.CPU_OVERSUBSCRIPTION != 3.4375:
        problems.append(f"真服务的倍数应当是 3.4375（32 核的台线是 110，用户定的），实际 {SERVER.CPU_OVERSUBSCRIPTION}")
    if problems:
        raise SelftestFailure("CPU 闸的线是核数 × 倍数：" + "；".join(problems))
    return "小世界照 2 倍（32 核线 64）：在跑一条再起一条照起（62）；在跑两条再起一条排着、原因写「32 核 × 2 = 64」；真服务的倍数是 3.4375（32 核线 110）"


def cell_repass_cleared_when_idle(world):
    """队空的一轮也清 repass_soon（task_server.py 的 place_queued_into；loop 审核发现 12）：上一轮第二台起满记了 repass_soon，这一轮队空：清掉，主循环照 TICK_SECONDS 睡。
    弄坏开关 repass-kept-when-idle 下照留，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.repass_soon = True
    fake.pause_in_force = lambda: None
    fake.store = types.SimpleNamespace(select=lambda *arguments, **options: [])
    fake.selftest_read = lambda line: None
    SERVER.Scheduler.place_queued_into(fake, {}, {})
    if fake.repass_soon:
        raise SelftestFailure("队空的一轮：repass_soon 应当清掉，实际还留着")
    return "队空的一轮清掉 repass_soon"


def peer_probe_resources():
    """peer-probe-off-scheduler 的假 Resources：只有第二台一台，探测照 outcome 交（每次记进 probes）；不读多机配置、不判盘余量。→ (Resources, probes, outcome)。"""
    probes, outcome = [], {"report": {"memory": {}}, "why": ""}
    resources = object.__new__(SERVER.Resources)
    resources.environment = dict(os.environ)
    resources.fake_path, resources.cached, resources.peer = "", None, None
    resources.peer_lock, resources.peer_reading, resources.peer_failed = threading.Lock(), None, None
    resources.peer_refresh_seconds = 0.1
    resources.refresh_peer = lambda: None
    resources.disk_problem = lambda machine, report, fake=None: ""
    resources.note_io_pressure = lambda machine, report, fake=None: None
    resources.machines = {SERVER.PEER: types.SimpleNamespace(
        name=SERVER.PEER, label="第二台", always_reachable=False, reachable=lambda found: (True, ""), configured_cap=lambda found: None,
        probe=lambda found: (probes.append(True), (outcome["report"], outcome["why"]))[1])}
    return resources, probes, outcome


def cell_peer_probe_off_scheduler(world):
    """第二台的余量不在排队线程里读（task_server.py 的 Resources.peer_probe_round、peer_snapshot_entry；loop 审核发现 3）：探测线程读了一回之后，snapshot 取那一份、不再连；
    第二台读不成的那一回记下，PEER_PROBE_FAILURE_HOLD_SECONDS 之内探测线程不再去连，snapshot 当它此刻用不了、原因写不再去连。
    弄坏开关 peer-probe-in-scheduler 下 snapshot 在调用方的线程里现连，这一格红。"""
    resources, probes, outcome = peer_probe_resources()
    problems = []
    resources.peer_probe_round()
    snapshot = resources.snapshot()
    if len(probes) != 1 or SERVER.PEER not in snapshot.reports:
        problems.append(f"探测线程读过一回：snapshot 应当取那一份、不再连，实际连了 {len(probes)} 次、报告里的台 {sorted(snapshot.reports)}")
    outcome["report"], outcome["why"] = None, "自证：第二台卡住"
    resources.peer_probe_round()
    resources.peer_probe_round()
    resources.cached = None
    snapshot = resources.snapshot()
    if len(probes) != 2 or SERVER.PEER in snapshot.reports or "不再去连" not in str(snapshot.down.get(SERVER.PEER)):
        problems.append(f"读不成之后：探测线程应当只再连 1 回、snapshot 当第二台用不了（原因写不再去连），实际共连了 {len(probes)} 次、"
                        f"报告里的台 {sorted(snapshot.reports)}、原因「{snapshot.down.get(SERVER.PEER)}」")
    if problems:
        raise SelftestFailure("第二台的余量不在排队线程里读：" + "；".join(problems))
    return "snapshot 取探测线程那一份；读不成的那一回之后一段时间不再去连"


PROMOTE_WAIT_SECONDS = 20   # promote-skips-cpu-gate：提前之后等它收尾的安静时限（弄坏开关 promote-cpu-gate-applied 下它一直排着，等满就判红）


def cell_promote_skips_cpu_gate(world):
    """task promote 提前（task_server.py 的 promote、路由 /tasks/{task_id}/promote、cpu_judgement）：本机内存够、负载 200（同 cpu-gate-holds-busy-host），命令键记过 31 核的任务排着、
    排队原因写「CPU：」；别的会话提前它：客户端退非 0、说不是这个会话提交的，任务照旧在排；提交它的会话提前：退 0、回话写「提前到队头」；负载不降，它照起、跑完（提前过的不过 CPU 闸）。
    弄坏开关 promote-session-unchecked 下别的会话也提前得了，promote-cpu-gate-applied 下提前过的照过 CPU 闸、一直排着，这一格红。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    seed_busy_cores(world, CPU_GATE_BUSY_KEY)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "promote-cpu-gate"], extra={lib.KEY_VARIABLE: CPU_GATE_BUSY_KEY})
    world.next_pass()
    world.next_pass()
    waiting = world.summary(task_id)
    reason = waiting.get("waiting_for") or ""
    if waiting["attempts"] or "CPU：" not in reason:
        raise SelftestFailure(f"负载 200 的本机上应当排着、排队原因写「CPU：」，实际尝试 {len(waiting['attempts'])} 次、排队原因「{reason}」", world.server_log_lines())
    problems = []
    foreign = world.client(["promote", str(task_id)], session=SESSION_B)
    after_foreign = world.summary(task_id)
    if foreign.returncode == 0 or "不是这个会话提交的" not in foreign.stdout + foreign.stderr or after_foreign["task"].get("state") != "queued":
        problems.append(f"别的会话提前：应当退非 0、说不是这个会话提交的、任务照旧在排，实际退 {foreign.returncode}、「{(foreign.stdout + foreign.stderr).strip()[-200:]}」、"
                        f"状态 {after_foreign['task'].get('state')}")
    own = world.client(["promote", str(task_id)])
    if own.returncode != 0 or "提前到队头" not in own.stdout:
        problems.append(f"提交它的会话提前：应当退 0、回话写「提前到队头」，实际退 {own.returncode}、「{(own.stdout + own.stderr).strip()[-200:]}」")
    try:
        done = world.finished(task_id, timeout=PROMOTE_WAIT_SECONDS)
    except SelftestFailure as failure:
        done = None
        problems.append(f"提前之后负载照旧 200：应当不过 CPU 闸、照起、跑完，实际安静 {PROMOTE_WAIT_SECONDS} 秒没收尾（{str(failure)[:200]}）")
    if done is not None and done["task"].get("outcome") != "succeeded":
        problems.append(f"提前之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("task promote 提前：" + "；".join(problems), world.server_log_lines())
    return f"负载 200 排着（「{reason[:60]}」）；别的会话提前被拒；提交它的会话提前之后不过 CPU 闸、照起、跑完"


def cell_promote_queue_order(world):
    """排队的先后（task_server.py 的 queue_order）：三条在排，提交 7200 秒的（排满 3600 秒的高优先）、提交 60 秒的、提交 30 秒而 task promote 提前过的：先后是提前过的、
    高优先的、其余的。套在假调度器上，不起服务。弄坏开关 promote-queue-order-ignored 下不看提前过没有（高优先的、其余的、提前过的），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.aged_seconds = 3600.0
    now = time.time()
    queued = [({"id": 1, "submitted_at": now - 7200}, True), ({"id": 2, "submitted_at": now - 60}, True), ({"id": 3, "submitted_at": now - 30, "promoted": {"at": now}}, True)]
    order = [row[0]["id"] for row in SERVER.Scheduler.queue_order(fake, queued, now)]
    if order != [3, 1, 2]:
        raise SelftestFailure(f"排队的先后应当是提前过的 3、高优先的 1、其余的 2，实际 {order}")
    return "提前过的排最前，再是排满时限的高优先，其余照原来的次序"


# ── 收尾不在排队线程里做（task_server.py 的 finish_attempt、finish_off_loop）──
PEER_RECOVER_HOLD_SECONDS = 20.0           # peer-cleanup-off-scheduler：第二台收尾卡着的这段里，本机那一条跑完的安静时限


def cell_peer_cleanup_off_scheduler(world):
    """第二台收尾卡住时本机照起（task_server.py 的 finish_off_loop；loop 审核发现 4）：派第二台的一条跑完、清理回执 pending，假 multi-host-run.sh 的 --recover-cleanup 卡着不回
    （世界目录里 recover-cleanup-hold 在、等 recover-cleanup-release）；卡着的这段里第二台演用不了、本机放得下，交一条本机的：它照起、在 PEER_RECOVER_HOLD_SECONDS 的安静时限里跑完，
    第二台那一条照旧在跑、不记结局；放开之后它收尾 succeeded。弄坏开关 finish-in-scheduler 下收尾在排队线程里当场做，排队线程卡在 --recover-cleanup 上，本机那一条一直排着，这一格红。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    hold, reached, release = (os.path.join(world.world, name) for name in ("recover-cleanup-hold", "recover-cleanup-reached", "recover-cleanup-release"))
    open(hold, "w").close()
    problems = []
    try:
        peer_id = world.submit(["bash", "-c", "exit 0", "peer-cleanup-stuck", "fake-peer-cleanup=pending"], cwd=world.repo)
        wait_path(world, reached, "第二台那一条收尾卡在 --recover-cleanup 上")
        world.resources(local=(30, 30), peer_down="自证：第二台这一刻不收新活")
        local_id = world.submit(["bash", "-c", "exit 0", "beside-stuck-peer-cleanup"])
        local = world.finished(local_id, timeout=PEER_RECOVER_HOLD_SECONDS)
        during = world.summary(peer_id)
        if local["task"].get("outcome") != "succeeded" or first_attempt(local).get("host") != SERVER.LOCAL:
            problems.append(f"第二台收尾卡着时本机那一条应当在本机跑完，实际 {describe(local)}、派到 {first_attempt(local).get('host')}")
        if during["task"].get("state") != "running" or first_attempt(during).get("state") not in ("starting", "running") or os.path.exists(release):
            problems.append(f"第二台那一条收尾还卡着：应当照旧在跑、不记结局，实际 {describe(during)}、尝试 {first_attempt(during).get('state')}")
    finally:
        open(release, "w").close()
    done = world.finished(peer_id)
    if done["task"].get("outcome") != "succeeded" or first_attempt(done).get("host") != SERVER.PEER:
        problems.append(f"放开之后第二台那一条应当收尾 succeeded，实际 {describe(done)}")
    raise_cell_problems(world, problems, "第二台收尾卡住时本机照起")
    return "第二台收尾卡在 --recover-cleanup 上：本机那一条照起、跑完，第二台那一条放开之后收尾"


def cell_cpu_reason_splits_io_wait(world):
    """CPU 闸的排队原因把等 IO 的线程拆开写（task_server.py 的 cpu_problem；余量报告 CPU 一项的 blocked_threads，memory_peaks_lib 的 cpu_report）：32 核（线 64）、负载 297，
    其中状态 D 的线程 249 个：再起一条排着，原因写「负载 297（其中等 IO 的线程 249）」；闸的判法不变（同样的负载、blocked_threads 是 0 或没有这一项，排不排一样，没有这一项的原因里不写括号）。
    弄坏开关 cpu-load-io-unsplit 下不拆开写，原因只写「负载 297」，这一格红。"""
    problems = []
    for what, blocked, wanted_text in (("249 个线程在等 IO", 249, "负载 297（其中等 IO 的线程 249）"), ("报告里没有等 IO 的线程数", None, "负载 297、")):
        cpu = {"load1": 297.0, "procs_running": 3, "cpus": 32}
        if blocked is not None:
            cpu["blocked_threads"] = blocked
        fake, snapshot = cpu_gate_scheduler(0, cpu, recorded_cores=(31.0,))   # 这一条照记录要 31 核：不走要核数少的名额
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if not why.startswith("CPU：") or wanted_text not in why:
            problems.append(f"{what}：应当排着、原因以「CPU：」起头并写「{wanted_text}」，实际「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸的原因拆开写等 IO 的线程：" + "；".join(problems))
    return "负载 297 里 249 个线程在等 IO：原因写「负载 297（其中等 IO 的线程 249）」；报告里没有这一项的不写括号；两种都照旧排着"


NO_BORROW_HEAD = f"bash {NO_BORROW_ENTRY_SCRIPT} --check selftest-cell-a"   # 这一趟分片的命令键原文（分片那一段之前）
NO_BORROW_OTHER_HEAD = f"bash {NO_BORROW_ENTRY_SCRIPT} --check selftest-cell-b"   # 同一入口脚本、别的参数的那一趟
NO_BORROW_PARENT, NO_BORROW_SIBLING = 90, 92   # 这一趟的父任务与同一趟的兄弟片（共 4 片的第 0 片）的任务号


def cell_shard_start_borrows_nothing(world):
    """自己这把键没有相当记录的片谁的都不借：同一趟的兄弟片、同入口别的参数那一趟、别的片数都不取，照没记录的缺省起（task_server.py 的 computed_prediction，
    定额度的入口；task_server_lib.py 文件头「资源记录」的定额度）：共 4 片的第 2 片。库里同一趟的兄弟片（同一个父任务名下、共 4 片的第 0 片）跑完峰值 60G，
    同一入口脚本、别的参数那一趟共 4 片的第 2 片（键不同）跑完 40G、共 8 片的第 1 片跑完 70G：起点应当是没记录的缺省 4096M、来历 unrecorded；
    对照：库里再有这一片自己这把键、片数相同的 6G 记录，起点就是 6G、来历是 record。
    弄坏开关 record-ignored-for-cap 下对照那一种也照缺省起，这一格红。"""
    store_rows = []
    children = [({"id": 91, "task_id": NO_BORROW_PARENT}, True), ({"id": NO_BORROW_SIBLING, "task_id": NO_BORROW_PARENT}, True)]

    def row(head, index, count, peak, when, task_id):
        store_rows.append(({"command_key": lib.shard_child_key(head, "shard", index, count), "state": "ended", "outcome": "succeeded", "memory_peak": peak, "memory_cap": peak + GIBIBYTE,
                            "shard_count": count, "role": "shard", "task_id": task_id, "ended_at": float(when), "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}, True))

    def select(table, where="1", parameters=(), order="id"):
        if table == "attempts":
            return [item for item in store_rows if item[0].get("command_key") == parameters[0]]
        return list(children) if table == "tasks" and parameters == (NO_BORROW_PARENT,) else []

    parent = {"id": NO_BORROW_PARENT, "command_key": NO_BORROW_HEAD, "sharded_entry": NO_BORROW_ENTRY_SCRIPT}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), lib.REPOSITORY_ROOT   # 定额度末了的内存下限（memory_floor_applied）读命令与仓根，认是不是换文件、打补丁的工具
    fake.host_dispatch = loaded_host_dispatch()
    fake.reads_log = None
    fake.prediction_lock = threading.Lock()
    fake.store = types.SimpleNamespace(select=select, task=lambda task_id: (parent, True) if task_id == NO_BORROW_PARENT else None)
    shard = {"id": 91, "role": "shard", "parent_id": NO_BORROW_PARENT, "shard_index": 2, "shard_count": 4, "command_key": lib.shard_child_key(NO_BORROW_HEAD, "shard", 2, 4),
             "command": ["bash", NO_BORROW_ENTRY_SCRIPT, "--check", "selftest-cell-a"], "cwd": lib.REPOSITORY_ROOT}

    def started():
        found = SERVER.Scheduler.computed_prediction(fake, shard)
        return found.memory_cap, found.memory_history

    row(NO_BORROW_HEAD, 0, 4, 60 * GIBIBYTE, 100, NO_BORROW_SIBLING)
    row(NO_BORROW_OTHER_HEAD, 2, 4, 40 * GIBIBYTE, 110, 81)
    row(NO_BORROW_OTHER_HEAD, 1, 8, 70 * GIBIBYTE, 115, 82)
    borrowed = started()
    row(NO_BORROW_HEAD, 2, 4, 6 * GIBIBYTE, 120, 71)
    own = started()
    problems = []
    for what, got, want in (("自己没有记录、库里有同一趟的兄弟片 60G、同一入口别的参数那一趟同片数 40G 与片数不同的 70G", borrowed, (lib.UNRECORDED_MEMORY_DEFAULT_BYTES, lib.UNRECORDED_HISTORY)),
                            ("对照：自己这把键有片数相同的 6G 记录", own, (6 * GIBIBYTE, lib.RECORD_HISTORY))):
        if got != want:
            problems.append(f"{what}：起点与来历应当是 {want}，实际 {got}")
    if problems:
        raise SelftestFailure("没有自己记录的片谁的都不借：" + "；".join(problems))
    return "共 4 片的第 2 片自己没有记录：同一趟的兄弟片 60G、同入口别的趟 40G、片数不同的 70G 都不取，照缺省 4096M 起；自己有 6G 记录就照 6G（record）"


def promote_fake(rows):
    """promote-wakes-and-promotes-shards 的假调度器：库是 rows（{任务号: 正文}）上的假 Store（task、change 照真的那样判 guard）。→ (调度器, 叫醒过几回的表)。"""
    def change(table, row_id, changes, guard=None, reopen=False):
        record = dict(rows[row_id])
        why = guard(record) if guard is not None else ""
        if why:
            raise SERVER.StoreRefused("guard", why)
        record.update(changes)
        rows[row_id] = record
        return record
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(task=lambda task_id: (dict(rows[task_id]), True) if task_id in rows else None, change=change,
                                       insert=lambda table, record: dict(record, id=99))
    fake.children_of = lambda parent_id: [dict(row) for row in rows.values() if row.get("parent_id") == parent_id]
    fake.operation = lambda *arguments: None
    woken = []
    fake.wake = lambda: woken.append(True)
    return fake, woken


def cell_promote_wakes_and_promotes_shards(world):
    """task promote 当场叫醒排队、分片的父任务也提前（task_server.py 的 promote、create_child）：分片的父任务 1（sharded）下一片 2 在排、一片 3 在跑：提前退 200、叫醒一回，
    父任务与在排的 2 记了 promoted，在跑的 3 不记；之后建的子任务带上父任务的 promoted。套在假调度器上，不起服务。
    弄坏开关 promote-unwoken 下不叫醒，promote-shards-refused 下分片的父任务照「不在排」回 409、子任务不继承，这一格红。"""
    rows = {1: {"id": 1, "session": SESSION_A, "state": "sharded"}, 2: {"id": 2, "session": SESSION_A, "state": "queued", "parent_id": 1},
            3: {"id": 3, "session": SESSION_A, "state": "running", "parent_id": 1}}
    fake, woken = promote_fake(rows)
    status, reply = SERVER.Scheduler.promote(fake, 1, SESSION_A, 0)
    fake.find_child = lambda children, role, index: None
    fake.child_key_text = lambda parent, role, index, count: "selftest-promoted-child"
    fake.command_key = lambda command, key: "selftest-promoted-child-key"
    child = SERVER.Scheduler.create_child(fake, dict(rows[1], command=["x"], cwd=world.root, environment={}), "shard", 1, 2)
    problems = []
    if status != 200 or not woken:
        problems.append(f"分片的父任务提前应当退 200、当场叫醒排队，实际 {status}「{reply}」、叫醒 {len(woken)} 回")
    if not rows[1].get("promoted") or not rows[2].get("promoted") or rows[3].get("promoted"):
        problems.append(f"父任务与在排的子任务应当记 promoted、在跑的不记，实际父 {rows[1].get('promoted')}、在排 {rows[2].get('promoted')}、在跑 {rows[3].get('promoted')}")
    if not child.get("promoted"):
        problems.append("提前过的父任务之后建的子任务应当带 promoted，实际没带")
    if problems:
        raise SelftestFailure("task promote 叫醒与分片：" + "；".join(problems))
    return "分片的父任务提前：退 200、叫醒排队，父任务与在排的片记 promoted，之后建的片继承"


def cell_promote_placement_rules(world):
    """提前过的排队规矩（task_server.py 的 aged、blocks_later）：提交 3 小时、提前过的不算排满时限（不挡后面）、起不来不算
    「排在前面还没起的」（后面的不照插队判）；同样 3 小时没提前过的照算排满时限、照算排在前面。套在假调度器上，不起服务。
    弄坏开关 promote-aged-blocks-queue、promote-blocked-counts-as-earlier 下各坏一样，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.aged_seconds = 3600.0
    now = time.time()
    promoted = {"id": 1, "submitted_at": now - 3 * 3600, "promoted": {"at": now - 60}}
    plain = {"id": 2, "submitted_at": now - 3 * 3600}
    seen = {name: (judged(fake, promoted, *extra), judged(fake, plain, *extra))
            for name, judged, extra in (("aged", SERVER.Scheduler.aged, (now,)), ("blocks_later", SERVER.Scheduler.blocks_later, ()))}
    wanted = {"aged": (False, True), "blocks_later": (False, True)}
    problems = [f"{name}：提前过的与没提前过的应当是 {wanted[name]}，实际 {found}" for name, found in seen.items() if found != wanted[name]]
    if problems:
        raise SelftestFailure("提前过的排队规矩：" + "；".join(problems))
    return "提前过的不算排满时限、起不来不挡后面"


def cell_stop_joins_peer_probe(world):
    """停服务时等第二台的余量探测线程（task_server.py 的 Scheduler.stop）：探测线程见到停的信号之后还要 0.5 秒才退（演它正在收一个 ssh）：停自己的子进程那一步之前它已退了。
    套在假调度器上，不起服务。弄坏开关 peer-probe-thread-unjoined 下不等它，停子进程时它还在，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.stopping, fake.inbox_event = threading.Event(), threading.Event()
    fake.wake = lambda: None
    for name in ("thread", "nfs_thread", "inbox_thread", "runaway_thread", "booking_audit_thread"):
        setattr(fake, name, threading.Thread(target=lambda: None))   # 没起：is_alive 为假
    finished = threading.Event()

    def probe():
        fake.stopping.wait(30)
        time.sleep(0.5)
        finished.set()
    fake.peer_probe_thread = threading.Thread(target=probe, daemon=True)
    fake.peer_probe_thread.start()
    fake.children = []
    seen = []
    fake.stop_own_children = lambda: seen.append(finished.is_set())
    SERVER.Scheduler.stop(fake)
    if seen != [True]:
        raise SelftestFailure(f"停自己的子进程之前探测线程应当已退，实际{'没停子进程' if not seen else '还在'}")
    return "停服务等探测线程退了才停子进程"


def cell_peer_cooldown_skips_connect(world):
    """第二台上一回读不成、还在冷却里，收尾那几处不去连（task_server.py 的 Resources.peer_cooling_down、PeerMachine 的 attempt_presence_readings、
    remove_build_directory）：两处都当场交「不再去连」、一次都没去重读第二台。套在假调度器上，不起服务。
    弄坏开关 peer-cooldown-ignored-off-scheduler 下照样去连，这一格红。"""
    resources = object.__new__(SERVER.Resources)
    resources.environment, resources.fake_path, resources.peer = dict(os.environ), "", None
    resources.peer_failed = (time.monotonic(), "自证：ssh 被重置")
    connects = []
    resources.refresh_peer = lambda: connects.append(True)
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.resources = dict(os.environ), resources
    fake.peer_runner = os.path.join(world.root, "no-such-multi-host-run.sh")
    machine = SERVER.PeerMachine(fake)
    presence = machine.attempt_presence_readings({})
    removed, lines = machine.remove_build_directory(os.path.join(world.root, "builds", "task-1-1"))
    problems = []
    if presence[0] is not None or "不再去连" not in presence[1]:
        problems.append(f"核进程应当当场交「不再去连」，实际 {presence}")
    if removed or not lines or "不再去连" not in lines[0]:
        problems.append(f"删第二台的编译目录应当当场交没删成、原因写「不再去连」，实际 {removed}、{lines}")
    if connects:
        problems.append(f"冷却里不该去重读第二台，实际去了 {len(connects)} 回")
    if problems:
        raise SelftestFailure("冷却里不去连第二台：" + "；".join(problems))
    return "第二台还在冷却里：核进程、删编译目录都当场交不再去连"
