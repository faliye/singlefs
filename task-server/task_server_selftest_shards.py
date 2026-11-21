"""task_server_selftest.py 拆出的一份：分片、取消、现场目录、合并的各格（总说明在 task_server_selftest.py 文件头）。"""
import os
import contextlib
import signal
import threading
import time
import types
import unittest.mock
import task_server_lib as lib
from task_server_selftest_fakes import (
    BLOCK_SCRIPT, GIBIBYTE, HOLD_UNTIL_RELEASED_SCRIPT, MEBIBYTE, PEAKS, POLL_SECONDS, SELFTEST_LOCAL_ADDRESS, SELFTEST_PEER_ADDRESS, SERVER,
    SESSION_A, SESSION_B, STATE_TIMEOUT_SECONDS, UNRECORDED_DEFAULT_PINNED_TEXT,
)
from task_server_selftest_world import (
    KEEP_TEMPORARY_EXTRA, NO_BORROW_ENTRY_SCRIPT, SCENE_COMMAND_EXIT, SelftestFailure, blocking_command, child_roles, describe, fake_cargo_extra,
    first_attempt, loaded_host_dispatch, process_gone, queued_shards, raise_cell_problems, read_text, running_shards, scene_of, shard_children,
    shard_outcomes, start_three_shards, wait_last_line, wait_path,
)


# 钉住 task_server_lib 的 NOT_SHARDED_ARGUMENTS：带这几个第一个参数的分片入口不分片
UNSHARDED_PINNED_ARGUMENTS = ("--selftest", "--list-items", "--help", "-h")
SHARD_PLAN_EXIT = 5     # 表外的退出码：计划、合并那一步自己退非 0，记 command-failed、wait 照它退
SHARD_MERGE_EXIT = 6


def cell_cancel_own_session_only(world):
    world.start_server()
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=SESSION_A)
    unit = world.running(task_id)["attempts"][0].get("unit", "")
    completed = world.client(["cancel", str(task_id)], session=SESSION_B)
    if completed.returncode != 3 or not world.unit_active(unit):
        open(release, "w").close()
        raise SelftestFailure(f"别的会话取消应当被拒（退 3）、单元照跑，实际退 {completed.returncode}、单元{'在跑' if world.unit_active(unit) else '停了'}", [completed.stderr])
    completed = world.client(["cancel", str(task_id)], session=SESSION_A)
    active = world.unit_active(unit)
    if completed.returncode != 0 or active:
        open(release, "w").close()
        raise SelftestFailure(f"同一会话取消应当退 0、回来时单元已停，实际退 {completed.returncode}、单元{'还在跑' if active else '停了'}", [completed.stdout, completed.stderr])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239:
        raise SelftestFailure(f"取消之后结局应当是 cancelled、退 239，实际 {describe(summary)}")
    return "别的会话取消退 3、单元照跑；同会话取消退 0、单元已停、结局 cancelled"


def cell_shard_to_peer_by_plan(world):
    """分片的各片不钉台（task_server_lib.py 文件头「分片」）：计划里写了 pin 的整件记 shard-plan-refused、wait 退 234、说明写「片不钉台，哪台由服务挑」；
    没写 pin 的两片里先排的那一片本机更空也照计划先去第二台（另一片去哪台看它排的时候第二台上那一片还在不在跑：一台同一时刻只跑同一件的一片），
    两片都成了再合并。第二片第一次尝试先睡 3 秒：第一片先成、第二片还在跑的那一段够长，弄坏开关 merge-before-all-green 下合并在那一段里起、只读到第 0 片，
    这一格红（两片几乎同时收尾时合并等不等都读到两片，这一格就不红）。"""
    world.resources(local=(100, 100), peer=(60, 60), peer_cap_gibibytes=64)   # 本机更空：按均衡会挑本机，只有照计划才去第二台
    pinned_spec = world.write_spec("spec-pinned.json", {"shards": [{"pin": "local", "label": "写了 pin 的那一片"}, {"label": "没写的那一片"}]})
    plain_spec = world.write_spec("spec-two-hosts.json", {"shards": [{"label": "第一片"}, {"label": "第二片", "first_attempt_delay": 3}]})
    world.start_server()
    problems = []
    refused_id = world.submit(pinned_spec, cwd=world.repo)
    refused = world.finished(refused_id)
    waited = world.client(["wait", str(refused_id)])
    if refused["task"].get("outcome") != "shard-plan-refused" or waited.returncode != 234 or "片不钉台，哪台由服务挑" not in waited.stdout:
        problems.append(f"计划写了 pin：整件应当记 shard-plan-refused、wait 退 234、说「片不钉台，哪台由服务挑」，实际 {describe(refused)}、wait 退 {waited.returncode}、"
                        f"「{waited.stdout.strip()[-300:]}」")
    summary = world.finished(world.submit(plain_spec, cwd=world.repo))
    hosts = {}
    for child in summary.get("children", []):
        if child["task"].get("role") == "shard":
            hosts[child["task"].get("shard_index")] = [attempt.get("host") for attempt in child["attempts"]]
    merged = world.sharded_calls("merge", present="0,1")
    if summary["task"].get("outcome") != "succeeded" or hosts.get(0) != ["peer"] or len(hosts.get(1) or []) != 1 or not merged:
        problems.append(f"本机空 100G、第二台空 60G，一件分两片、都没写 pin：先排的那一片照计划派到第二台（本机放得下也去）、另一片起一次，两片都成了再合并，"
                        f"实际整件 {summary['task'].get('outcome')}、各片的台 {hosts}、合并 {merged}")
    if problems:
        raise SelftestFailure("分片不钉台：" + "；".join(problems), world.server_log_lines())
    return "计划写了 pin 的记 shard-plan-refused（片不钉台）；没写 pin 的两片里先排的那一片照计划在第二台（本机放得下也去），合并之后整件成了"


def cell_shard_siblings_share_a_host(world):
    """同一件的兄弟片不避开彼此（task_server_lib.py 文件头「挑机器」）：两台都空时两片同时跑；第二台用不了、只有本机接得了而内存放得下时，两片同时在本机跑，不排队等兄弟片。"""
    releases = {name: os.path.join(world.root, f"release-siblings-{name}") for name in ("spread", "together")}
    spread_spec = world.write_spec("spec-siblings-spread.json", {"shards": [{"block": releases["spread"]}, {"block": releases["spread"]}]})
    one_host_spec = world.write_spec("spec-siblings-one-host.json", {"shards": [{"block": releases["together"]}, {"block": releases["together"]}]})
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    problems = []
    spread_id = world.submit(spread_spec, cwd=world.repo)
    try:
        world.wait_for(spread_id, lambda summary: len(running_shards(summary)) == 2, "两片都跑起来")
    finally:
        open(releases["spread"], "w").close()
    spread = world.finished(spread_id)
    if spread["task"].get("outcome") != "succeeded":
        problems.append(f"两台都空、一件两片：整件应当成了，实际 {describe(spread)}")
    world.resources(local=(100, 100), peer_down="自证：第二台此刻用不了")
    one_host_id = world.submit(one_host_spec, cwd=world.repo)
    try:
        summary = world.wait_for(one_host_id, lambda summary: len(running_shards(summary)) == 2, "两片同时在本机跑起来")
        if sorted(running_shards(summary).values()) != ["local", "local"]:
            problems.append(f"只有本机接得了：两片应当同时在本机跑，实际在跑的各片的台 {running_shards(summary)}")
    finally:
        open(releases["together"], "w").close()
    one_host = world.finished(one_host_id)
    if one_host["task"].get("outcome") != "succeeded":
        problems.append(f"只有本机接得了：整件应当成了，实际 {describe(one_host)}")
    if problems:
        raise SelftestFailure("兄弟片同台：" + "；".join(problems), world.server_log_lines())
    return "两台都空时两片同时跑；只有本机接得了而内存放得下时两片同时在本机跑，不排队等兄弟片"


def cell_shard_retry_isolated(world):
    world.resources(local=(60, 60))
    world.start_server()
    # 撞上限的那一片晚 3 秒才退：兄弟片这时已经记成 succeeded（定时缩短下对账一轮 0.1 秒），弄坏开关 shard-retry-restarts-all 下它们会被重排、多跑一次
    retried = world.write_spec("spec-retry.json", {"shards": [{}, {}, {"exits": [250, "run"], "first_attempt_delay": 3}]})
    failing = world.write_spec("spec-failing.json", {"shards": [{}, {"exits": [3]}, {}]})
    broken_plan = world.write_spec("spec-broken-plan.json", {"shards": [{}], "plan_text": "不是 JSON"})
    retried_id = world.submit(retried, cwd=world.repo)
    retried_summary = world.finished(retried_id)
    problems = []
    children = {(child["task"].get("role"), child["task"].get("shard_index")): child for child in retried_summary.get("children", [])}
    task_of = {index: str(children[("shard", index)]["task"]["id"]) for index in range(3) if ("shard", index) in children}
    runs = {index: [row.get("attempt") for row in world.sharded_calls("shard", task=task_of.get(index, "-"))] for index in range(3)}
    merge_task = str(children.get(("merge", 0), {"task": {}})["task"].get("id", "-"))
    merges = world.sharded_calls("merge", task=merge_task)
    if retried_summary["task"].get("outcome") != "succeeded" or runs != {0: ["1"], 1: ["1"], 2: ["1", "2"]} or [row.get("present") for row in merges] != ["0,1,2"]:
        problems.append(f"第 2 片撞一次上限：只重排那一片（第 2 片跑 2 次、别的各 1 次），全部片绿了才合并一次、合并时三片都在，"
                        f"实际整件 {retried_summary['task'].get('outcome')}、各片跑的次数 {runs}、合并 {merges}")
    failing_id = world.submit(failing, cwd=world.repo)
    failing_summary = world.finished(failing_id)
    waited = world.client(["wait", str(failing_id)])
    last = waited.stdout.strip().splitlines()[-1] if waited.stdout.strip() else ""
    has_merge = any(child["task"].get("role") == "merge" for child in failing_summary.get("children", []))
    if (failing_summary["task"].get("outcome") != "command-failed" or waited.returncode != 3 or " shard=1/3" not in last or has_merge
            or not last.startswith(f"{lib.RESULT_MARK} task={failing_id} type=command-failed exit=3 ")):
        problems.append(f"第 1 片退 3：整件结局应当是那一片的 command-failed、wait 退 3、末行点名 shard=1/3、不合并，"
                        f"实际 {describe(failing_summary)}、wait 退 {waited.returncode}、末行「{last}」、{'合并了' if has_merge else '没合并'}")
    plan_id = world.submit(broken_plan, cwd=world.repo)
    plan_summary = world.finished(plan_id)
    waited = world.client(["wait", str(plan_id)])
    last = waited.stdout.strip().splitlines()[-1] if waited.stdout.strip() else ""
    if plan_summary["task"].get("outcome") != "shard-plan-refused" or waited.returncode != 234 or " step=plan" not in last:
        problems.append(f"计划写坏：整件应当记 shard-plan-refused、wait 退 234、末行点名 step=plan，实际 {describe(plan_summary)}、wait 退 {waited.returncode}、末行「{last}」")
    if problems:
        raise SelftestFailure("分片：" + "；".join(problems), world.server_log_lines())
    return "一片撞上限只重排那一片、全部片绿了才合并；一片最终没成整件是那一片的类型、末行点名 shard=1/3；计划写坏记 shard-plan-refused"


def cell_cancel_queued(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    marker = os.path.join(world.root, "queued-ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    completed = world.client(["cancel", str(task_id)])
    summary = world.summary(task_id)
    problems = []
    if (completed.returncode != 0 or "在队里时取消" not in completed.stdout or summary["task"].get("outcome") != "cancelled"
            or summary["task"].get("exit_code") != 239 or summary["attempts"]):
        problems.append(f"取消排着的：应当退 0、当场记 cancelled（退 239）、一次没起，实际退 {completed.returncode}、状态 {summary['task'].get('state')}、{describe(summary)}")
    world.resources(local=(30, 30))
    time.sleep(1.5)
    summary = world.summary(task_id)
    if summary["attempts"] or world.calls_of(task_id) or os.path.exists(marker):
        problems.append(f"放开余量之后，取消了的那一件应当一次都不起，实际 {describe(summary)}、包装调用 {world.calls_of(task_id)}、"
                        f"命令{'跑了' if os.path.exists(marker) else '没跑'}")
    if problems:
        raise SelftestFailure("；".join(problems), [completed.stdout, completed.stderr])
    return "取消排着的：当场记 cancelled、退 0，放开余量之后也一次没起"


def cell_cancel_finished(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 77"])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "not-run":
        raise SelftestFailure(f"要取消的那一件应当先有结局 not-run（命令退 77），实际 {describe(summary)}")
    problems = []
    for times in (1, 2):
        completed = world.client(["cancel", str(task_id)])
        summary = world.summary(task_id)
        if (completed.returncode != 0 or "已经有结局了" not in completed.stdout or summary["task"].get("outcome") != "not-run"
                or summary["task"].get("exit_code") != 77 or len(summary["attempts"]) != 1):
            problems.append(f"第 {times} 回取消已有结局的：应当退 0、说已经有结局了、结局照旧 not-run（退 77）、尝试照旧 1 次，实际退 {completed.returncode}、{describe(summary)}")
    if problems:
        raise SelftestFailure("；".join(problems))
    return "取消已有结局的连取消两回都退 0、结局照旧 not-run（退 77），不改记 cancelled"


def cell_cancel_sharded_in_plan(world):
    world.resources(local=(30, 30), hold=True)
    command = world.write_spec("spec-cancel-in-plan.json", {"shards": [{}]})
    world.start_server()
    task_id = world.submit(command, cwd=world.repo)
    children = world.summary(task_id).get("children", [])
    if [(child["task"].get("role"), child["task"].get("state")) for child in children] != [("plan", "queued")]:
        raise SelftestFailure(f"分片的入口交上去应当先拆出一步排着的计划，实际子任务 {[(child['task'].get('role'), child['task'].get('state')) for child in children]}")
    plan_id = children[0]["task"]["id"]
    completed = world.client(["cancel", str(task_id)])
    summary, plan = world.summary(task_id), world.summary(plan_id)
    problems = []
    if (completed.returncode != 0 or summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239
            or plan["task"].get("outcome") != "cancelled" or plan["attempts"]):
        problems.append(f"计划还排着时取消整件：应当退 0、整件记 cancelled（退 239）、计划那一步当场记 cancelled、一次没起，实际退 {completed.returncode}、"
                        f"整件 {describe(summary)}、计划那一步 {plan['task'].get('state')} {describe(plan)}")
    world.resources(local=(30, 30))
    time.sleep(1.5)
    plan = world.summary(plan_id)
    if plan["attempts"] or world.calls_of(plan_id) or world.sharded_calls("plan") or child_roles(world.summary(task_id)) != ["plan"]:
        problems.append(f"放开余量之后计划那一步应当一次都不起、不再拆出别的子任务，实际计划那一步 {describe(plan)}、计划跑的记录 {world.sharded_calls('plan')}、"
                        f"子任务 {child_roles(world.summary(task_id))}")
    if problems:
        raise SelftestFailure("取消分片的整件：" + "；".join(problems), [completed.stdout, completed.stderr])
    return "计划还排着时取消整件：整件与计划那一步当场记 cancelled，放开余量之后也一次没起"


def cell_cancel_sharded_running(world):
    release = os.path.join(world.root, "release-shards")
    command = world.write_spec("spec-cancel-running.json", {"shards": [{"block": release}, {"block": release}]})
    world.resources(local=(30, 30), peer=(30, 30))   # 两片要同时在跑：一台同一时刻只跑同一件的一片，给两台
    world.start_server()
    task_id = world.submit(command, cwd=world.repo)
    problems = []
    try:
        summary = world.wait_for(task_id, lambda summary: len(shard_children(summary)) == 2 and len(world.sharded_calls("shard")) == 2 and all(
            any(attempt.get("state") == "running" for attempt in child["attempts"]) for child in shard_children(summary).values()), "两片都跑起来")
        units = [child["attempts"][-1].get("unit", "") for child in shard_children(summary).values()]
        completed = world.client(["cancel", str(task_id)])
        still = [unit for unit in units if world.unit_active(unit)]
        summary = world.summary(task_id)
        if completed.returncode != 0 or summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239 or still:
            problems.append(f"两片都在跑时取消整件：应当退 0、整件记 cancelled（退 239）、回来时两片的单元都停了，实际退 {completed.returncode}、"
                            f"整件 {summary['task'].get('outcome')}、还在跑的单元 {still}")
    finally:
        open(release, "w").close()   # 弄坏开关下单元没停：放它们跑完，收尾不等满 60 秒
    for index, child in sorted(shard_children(world.summary(task_id)).items()):
        finished = world.finished(child["task"]["id"])
        if finished["task"].get("outcome") != "cancelled":
            problems.append(f"第 {index} 片应当记 cancelled，实际 {describe(finished)}")
    if "merge" in child_roles(world.summary(task_id)) or world.sharded_calls("merge"):
        problems.append(f"取消了的整件不该再合并，实际子任务 {child_roles(world.summary(task_id))}、合并的记录 {world.sharded_calls('merge')}")
    if problems:
        raise SelftestFailure("取消分片的整件：" + "；".join(problems), world.server_log_lines())
    return "两片都在跑时取消整件：整件记 cancelled，回来时两片的单元都停了、各记 cancelled，不合并"


def cell_shard_plan_failed(world):
    world.start_server()
    command = world.write_spec("spec-plan-exit.json", {"shards": [{}], "plan_exit": SHARD_PLAN_EXIT})
    task_id = world.submit(command, cwd=world.repo)
    summary = world.finished(task_id)
    code, last = wait_last_line(world, task_id)
    if (summary["task"].get("outcome") != "command-failed" or summary["task"].get("exit_code") != SHARD_PLAN_EXIT or code != SHARD_PLAN_EXIT
            or not last.startswith(f"{lib.RESULT_MARK} task={task_id} type=command-failed exit={SHARD_PLAN_EXIT} ") or " step=plan" not in last
            or child_roles(summary) != ["plan"] or world.sharded_calls("shard") or world.sharded_calls("merge")):
        raise SelftestFailure(f"计划那一步退 {SHARD_PLAN_EXIT}（plan.json 照写了）：整件应当是那一步的 command-failed、退 {SHARD_PLAN_EXIT}，wait 退 {SHARD_PLAN_EXIT}、"
                              f"末行点名 step=plan，不拆片、不合并，实际 {describe(summary)}、wait 退 {code}、末行「{last}」、子任务 {child_roles(summary)}、"
                              f"片跑的记录 {len(world.sharded_calls('shard'))} 行、合并的 {len(world.sharded_calls('merge'))} 行", world.server_log_lines())
    return f"计划那一步退 {SHARD_PLAN_EXIT}：整件 command-failed、wait 退 {SHARD_PLAN_EXIT}、末行点名 step=plan，没拆片"


def cell_shard_merge_failed(world):
    world.start_server()
    command = world.write_spec("spec-merge-exit.json", {"shards": [{}], "merge_exit": SHARD_MERGE_EXIT})
    task_id = world.submit(command, cwd=world.repo)
    summary = world.finished(task_id)
    code, last = wait_last_line(world, task_id)
    if (summary["task"].get("outcome") != "command-failed" or summary["task"].get("exit_code") != SHARD_MERGE_EXIT or code != SHARD_MERGE_EXIT
            or not last.startswith(f"{lib.RESULT_MARK} task={task_id} type=command-failed exit={SHARD_MERGE_EXIT} ") or " step=merge" not in last
            or child_roles(summary) != ["merge", "plan", "shard"] or len(world.sharded_calls("shard")) != 1 or len(world.sharded_calls("merge", present="0")) != 1):
        raise SelftestFailure(f"一片跑完、合并那一步退 {SHARD_MERGE_EXIT}：整件应当是那一步的 command-failed、退 {SHARD_MERGE_EXIT}，wait 退 {SHARD_MERGE_EXIT}、"
                              f"末行点名 step=merge，实际 {describe(summary)}、wait 退 {code}、末行「{last}」、子任务 {child_roles(summary)}、"
                              f"片跑的记录 {world.sharded_calls('shard')}、合并的 {world.sharded_calls('merge')}", world.server_log_lines())
    return f"合并那一步退 {SHARD_MERGE_EXIT}：整件 command-failed、wait 退 {SHARD_MERGE_EXIT}、末行点名 step=merge"


SHARD_BINARIES_NAMES = ("crash_enumeration-5f3a91c0", "power_loss_replay-9b21d4e7", "binaries.tsv")   # shard-binaries-removed-at-outcome：计划那一步放进 test-binaries/ 的几份（两份测试二进制与清单）
SHARD_BINARIES_NOTE_SECONDS = 10.0   # shard-binaries-removed-at-outcome：整件跑成之后等那一行操作日志的安静时限（结局记下之后才删、才记）


def shard_plan_directory_of(world, task_id):
    """假分片入口计划那一步记下的计划目录（调用记录里的 sharded-plan-binaries 一行；目录是 <库标签>-<父任务号>/plan）；没记的交空串。"""
    for row in world.sharded_calls("plan-binaries"):
        if row.get("directory", "").endswith(f"-{task_id}/plan"):
            return row["directory"]
    return ""


def shard_binaries_notes(world, task_id):
    """操作日志里这条父任务 shard-binaries 那几行的说明（task_server.py 的 shard_binaries_removal 记的）。"""
    return [operation.get("note", "") for operation in world.operations() if operation.get("action") == "shard-binaries" and operation.get("task") == task_id]


def cell_shard_binaries_removed_at_outcome(world):
    """分片父任务有了结局时服务删计划目录下的 test-binaries/（task_server.py 的 shard_binaries_removal；文件头 ④）：
    一片在跑时取消整件：回来时 test-binaries/ 已删、plan.json 还在，操作日志写删了 3 份；再往那里放一份，task clean 照删、plan.json 还在、操作日志写 1 份；
    计划那一步把 test-binaries 建成指到计划目录外的符号链接、整件跑成：不删，外面那个目录里的文件还在、链接还在，操作日志写没删，服务日志打那一行。
    弄坏开关 shard-binaries-kept 下取消与 clean 之后都还在，shard-binaries-path-unchecked 下计划目录外那一份被删，这一格红。"""
    release = os.path.join(world.root, "release-binaries-shard")
    world.start_server()
    problems = []
    cancelled = world.submit(world.write_spec("spec-binaries-cancel.json", {"shards": [{"block": release}], "plan_binaries": list(SHARD_BINARIES_NAMES)}), cwd=world.repo)
    try:
        world.wait_for(cancelled, lambda summary: any(attempt.get("state") == "running" for child in shard_children(summary).values() for attempt in child["attempts"]),
                       "那一片跑起来")
        completed = world.client(["cancel", str(cancelled)])
    finally:
        open(release, "w").close()   # 取消没停下那一片：放它跑完，收尾不等满 60 秒
    plan = shard_plan_directory_of(world, cancelled)
    binaries = os.path.join(plan, lib.SHARD_PLAN_BINARIES_NAME)
    outcome = world.summary(cancelled)["task"].get("outcome")
    notes = shard_binaries_notes(world, cancelled)
    if (not plan or completed.returncode != 0 or outcome != "cancelled" or os.path.lexists(binaries) or not os.path.exists(os.path.join(plan, lib.PLAN_FILE))
            or not any(f"{len(SHARD_BINARIES_NAMES)} 份" in note for note in notes)):
        problems.append(f"一片在跑时取消整件：应当退 0、记 cancelled，回来时计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}/ 已删、{lib.PLAN_FILE} 还在、操作日志写删了 "
                        f"{len(SHARD_BINARIES_NAMES)} 份，实际退 {completed.returncode}、结局 {outcome}、计划目录「{plan or '没记下'}」、"
                        f"{lib.SHARD_PLAN_BINARIES_NAME}/ {'还在' if os.path.lexists(binaries) else '不在'}、{lib.PLAN_FILE} {'在' if os.path.exists(os.path.join(plan, lib.PLAN_FILE)) else '不在'}、操作日志 {notes}")
    if plan:
        os.makedirs(binaries, exist_ok=True)
        with open(os.path.join(binaries, "late-binary"), "w", encoding="utf-8") as handle:
            handle.write("late\n")
        cleaned = world.client(["clean", str(cancelled)])
        notes = shard_binaries_notes(world, cancelled)
        if cleaned.returncode != 0 or os.path.lexists(binaries) or not os.path.exists(os.path.join(plan, lib.PLAN_FILE)) or not any("1 份" in note for note in notes):
            problems.append(f"再往 {lib.SHARD_PLAN_BINARIES_NAME}/ 放一份之后 task clean：应当退 0、那个子目录删了、{lib.PLAN_FILE} 还在、操作日志写删了 1 份，"
                            f"实际退 {cleaned.returncode}、{lib.SHARD_PLAN_BINARIES_NAME}/ {'还在' if os.path.lexists(binaries) else '不在'}、操作日志 {notes}、"
                            f"输出「{(cleaned.stdout + cleaned.stderr).strip()[-300:]}」")
    outside = os.path.join(world.root, "binaries-elsewhere")
    linked = world.submit(world.write_spec("spec-binaries-link.json", {"shards": [{}], "plan_binaries_link": outside}), cwd=world.repo)
    done = world.finished(linked)
    patience = world.patience(SHARD_BINARIES_NOTE_SECONDS)
    while patience.waiting() and not shard_binaries_notes(world, linked):
        time.sleep(POLL_SECONDS)
    notes = shard_binaries_notes(world, linked)
    link = os.path.join(shard_plan_directory_of(world, linked), lib.SHARD_PLAN_BINARIES_NAME)
    logged = any("不是这一条任务自己的计划目录下的那一份" in line for line in world.server_log_lines())
    if (done["task"].get("outcome") != "succeeded" or not os.path.exists(os.path.join(outside, "kept")) or not os.path.islink(link)
            or not any(note.startswith("没删") for note in notes) or not logged):
        problems.append(f"{lib.SHARD_PLAN_BINARIES_NAME} 是指到计划目录外的符号链接、整件跑成：应当不删（外面那份 kept 还在、链接还在）、操作日志写没删、服务日志打一行，"
                        f"实际结局 {done['task'].get('outcome')}、kept {'在' if os.path.exists(os.path.join(outside, 'kept')) else '不在'}、"
                        f"链接 {'在' if os.path.islink(link) else '不在'}、操作日志 {notes}、服务日志{'有' if logged else '没有'}那一行")
    raise_cell_problems(world, problems, "分片的测试二进制")
    return (f"取消整件时计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}/ 删了（{lib.PLAN_FILE} 留着）、task clean 照删；"
            f"它是指到计划目录外的符号链接的不删，操作日志写「{notes[0][:60] if notes else ''}…」")


def cell_unsharded_arguments(world):
    world.start_server()
    problems = []
    control = world.finished(world.submit(world.write_spec("spec-unsharded-control.json", {"shards": [{}]}), cwd=world.repo))
    if not control["task"].get("sharded_entry") or control["task"].get("outcome") != "succeeded" or child_roles(control) != ["merge", "plan", "shard"]:
        problems.append(f"对照：假分片入口的文件头写着「# task-server-shards: *」，带 run 交上去应当拆成计划、一片、合并跑完，实际认成 "
                        f"{control['task'].get('sharded_entry') or '不分片'}、整件 {control['task'].get('outcome')}、子任务 {child_roles(control)}")
    for argument in UNSHARDED_PINNED_ARGUMENTS:
        task_id = world.submit(["python3", "fake-sharded.py", argument], cwd=world.repo)
        summary = world.finished(task_id)
        whole = world.sharded_calls("whole", task=task_id, first=argument)
        if summary["task"].get("sharded_entry") or summary.get("children") or summary["task"].get("outcome") != "succeeded" or len(whole) != 1:
            problems.append(f"带 {argument} 的应当不分片、整件当普通命令跑一次，实际认成 {summary['task'].get('sharded_entry') or '不分片'}、"
                            f"子任务 {child_roles(summary)}、{describe(summary)}、整件跑的记录 {whole}")
    if problems:
        raise SelftestFailure("分片的入口：" + "；".join(problems), world.server_log_lines())
    return f"文件头写了「*」的分片入口带 run 照分片；带 {'、'.join(UNSHARDED_PINNED_ARGUMENTS)} 的不分片、整件跑一次"


def cell_shard_host_is_address(world):
    """分片的一片拿到它所在那一台在 HOSTS 里的地址（task_server_lib.py 文件头「分片」②的 TASK_SERVER_SHARD_HOST）：两台都放得下、一件分两片，
    每一片每一次尝试记下的 TASK_SERVER_SHARD_HOST 是那一次的台在 HOSTS 里的地址（本机是 SELFTEST_LOCAL_ADDRESS、第二台是 SELFTEST_PEER_ADDRESS），不是 local / peer。
    弄坏开关 shard-host-role-passed 下照改前给 local / peer。"""
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)
    spec = world.write_spec("spec-shard-host-address.json", {"shards": [{"label": "第一片"}, {"label": "第二片"}]})
    world.start_server()
    summary = world.finished(world.submit(spec, cwd=world.repo))
    addresses = {"local": SELFTEST_LOCAL_ADDRESS, "peer": SELFTEST_PEER_ADDRESS}
    seen, wrong = [], []
    for index, child in sorted(shard_children(summary).items()):
        for attempt in child["attempts"]:
            rows = world.sharded_calls("shard", index=index, attempt=attempt.get("number"))
            given = [row.get("host") for row in rows]
            seen.append(f"片 {index} 第 {attempt.get('number')} 次 {attempt.get('host')}：{given}")
            if given != [addresses.get(attempt.get("host"))]:
                wrong.append(seen[-1])
    if summary["task"].get("outcome") != "succeeded" or not seen or wrong:
        raise SelftestFailure(f"分片的每一次尝试拿到的 TASK_SERVER_SHARD_HOST 应当是那一次的台在 HOSTS 里的地址（本机 {SELFTEST_LOCAL_ADDRESS}、第二台 {SELFTEST_PEER_ADDRESS}），"
                              f"整件成了；实际整件 {summary['task'].get('outcome')}、对不上的 {wrong}、全部 {seen}", world.server_log_lines())
    return f"分片的 {len(seen)} 次尝试拿到的 TASK_SERVER_SHARD_HOST 都是那一次的台在 HOSTS 里的地址（{'；'.join(seen)}）"


SHARD_KEY_BASES = ("selftest-shard-keys", "bash scripts/crash/crash-amplification-two-hosts.sh --merged-log merged.log")   # shard-keys-kept-apart：交了键的、没交键按命令取的


def cell_shard_keys_kept_apart(world):
    """分片各步的命令键（lib.shard_child_key）过了键的换算（memory_peaks_lib.py 文件头「键」，服务取键经它）：仍是这件换算过的键加那一段，
    计划、各片、合并两两不同；这件的键本身不再变的，换算前后逐字相同（seed_peak 按它记的峰值服务读得到）。
    弄坏开关 shard-key-position-folded 下片号写成 <片号>/<片数>、被折成〈文件〉，各片落一个键，这一格红。"""
    steps = (("plan", 0, 0), ("shard", 0, 3), ("shard", 1, 3), ("shard", 2, 3), ("merge", 0, 0))
    problems = []
    for base in SHARD_KEY_BASES:
        folded_base = PEAKS.command_key([], base)
        texts = [lib.shard_child_key(base, *step) for step in steps]
        keys = [PEAKS.command_key([], text) for text in texts]
        unchanged = folded_base != base or keys == texts
        if len(set(keys)) != len(steps) or not all(key.startswith(folded_base + " ") for key in keys) or not unchanged:
            problems.append(f"这件的键「{base}」（换算后「{folded_base}」）：各步原文 {texts}，换算后 {keys}")
    if problems:
        raise SelftestFailure("分片各步的命令键应当过了键的换算仍各是各的、是这件换算过的键加那一段，这件的键不变的原文也不变；" + "；".join(problems))
    return f"{len(SHARD_KEY_BASES)} 种这件的键：计划、三片、合并的命令键换算之后两两不同，都是这件的键加那一段"


SHARD_CRASH_POINTS = ("parent-inserted", "child-finished-plan", "shard-inserted", "child-finished-shard", "merge-outputs-written", "merge-inserted",
                      "child-finished-merge")


def cell_sharded_progress_recovered(world):
    """TS-02：分片推进的每一个断点（父任务插入后、计划这一步终态提交后、建了一片之后、一片终态提交后、写了 outputs.tsv 之后、建了合并之后、合并终态提交后）
    服务当场退出、重启之后都推到整件 succeeded，每一步（计划、各片、合并）只有一个子任务。"""
    problems = []
    for point in SHARD_CRASH_POINTS:
        world.stop_server()
        arm = os.path.join(world.root, f"arm-{point}")
        command = world.write_spec(f"spec-recover-{point}.json", {"shards": [{}, {}]})
        world.start_server(extra={"TASK_SERVER_SELFTEST_CRASH_POINT": f"{point}:{arm}"})
        open(arm, "w").close()
        before = {record["id"] for record, _ in world.signed_rows("tasks")}
        submitted = world.client(["submit", "--cwd", world.repo, "--", *command])
        world.confirm_submitted(submitted, required=False)   # 确认之后才进队、建计划那一步（parent-inserted 那一点就在确认里，服务在那一刻退出）
        code = world.wait_server_exit()
        if code != 70:
            problems.append(f"{point}：自证造的崩溃应当让服务退 70，实际退 {code}")
            continue
        world.start_server()
        parents = [record["id"] for record, _ in world.signed_rows("tasks") if record["id"] not in before and record.get("sharded_entry")]
        if len(parents) != 1:
            problems.append(f"{point}：库里应当有这一件的父任务一条，实际 {parents}（提交退 {submitted.returncode}）")
            continue
        summary = world.finished(parents[0])
        roles = [(child["task"].get("role"), child["task"].get("shard_index", 0)) for child in summary.get("children", [])]
        if summary["task"].get("outcome") != "succeeded" or sorted(roles) != [("merge", 0), ("plan", 0), ("shard", 0), ("shard", 1)]:
            problems.append(f"{point}：重启之后应当推到整件 succeeded、计划一个、两片各一个、合并一个，实际 {describe(summary)}、子任务 {sorted(roles)}")
    if problems:
        raise SelftestFailure("分片推进断在半途：" + "；".join(problems), world.server_log_lines())
    return f"{len(SHARD_CRASH_POINTS)} 个断点各崩一次，重启之后都推到整件 succeeded、每一步只一个子任务"


def cell_sharded_cancel_retried(world):
    """TS-04：两片都在跑时取消整件，第一次 systemctl stop 没停成（TASK_SERVER_SELFTEST_STOP_FAILS 演一次）：取消回没停、整件不记结局（留在 cancelling）；
    第二次取消照样重查、重停活动的子任务，都停了、读回过才回 stopped，整件记 cancelled。"""
    release = os.path.join(world.root, "release-cancel-retried")
    stop_fails = os.path.join(world.root, "stop-fails-once")
    command = world.write_spec("spec-cancel-retried.json", {"shards": [{"block": release}, {"block": release}]})
    world.resources(local=(30, 30), peer=(30, 30))   # 两片要同时在跑：一台同一时刻只跑同一件的一片，给两台
    world.start_server(extra={"TASK_SERVER_SELFTEST_STOP_FAILS": stop_fails})
    task_id = world.submit(command, cwd=world.repo)
    problems = []
    try:
        summary = world.wait_for(task_id, lambda summary: len(shard_children(summary)) == 2 and all(
            any(attempt.get("state") == "running" for attempt in child["attempts"]) for child in shard_children(summary).values()), "两片都跑起来")
        units = [child["attempts"][-1].get("unit", "") for child in shard_children(summary).values()]
        open(stop_fails, "w").close()
        first = world.client(["cancel", str(task_id)])
        after_first = world.summary(task_id)
        if first.returncode == 0 or after_first["task"].get("state") == "finished" or not any(world.unit_active(unit) for unit in units):
            problems.append(f"第一次停没停成：取消应当不退 0、整件不记结局、还有子任务的单元在跑，实际退 {first.returncode}、"
                            f"整件 {after_first['task'].get('state')}/{after_first['task'].get('outcome')}、在跑的单元 {[unit for unit in units if world.unit_active(unit)]}")
        second = world.client(["cancel", str(task_id)])
        still = [unit for unit in units if world.unit_active(unit)]
        after_second = world.summary(task_id)
        if second.returncode != 0 or still or after_second["task"].get("outcome") != "cancelled":
            problems.append(f"第二次取消应当重停活动的子任务、都停了才退 0、整件记 cancelled，实际退 {second.returncode}、还在跑的单元 {still}、"
                            f"整件 {after_second['task'].get('state')}/{after_second['task'].get('outcome')}（{second.stdout.strip()[-200:]}）")
    finally:
        open(release, "w").close()
    if problems:
        raise SelftestFailure("分片取消重试：" + "；".join(problems), world.server_log_lines())
    return "第一次停没停成：取消不回 stopped、整件留在 cancelling；第二次取消重停活动的子任务，回读停了才回、整件记 cancelled"


def cell_scene_sessionless_swept(world):
    """没有会话的现场（task_server_lib.py 文件头「现场」；task_server.py 的 scene_due）：人手 --manual 交的退非 0、留了现场，不起 wait：结局满保留期（快档 1 秒）之后服务照样删。
    弄坏开关 scene-sessionless-kept 下要等取走、一直不删，这一格红。"""
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "exit 0", f"fake-wrapper-scene={SCENE_COMMAND_EXIT}"], session=None))
    scene, _ = scene_of(world, summary)
    if summary["task"].get("session") or not scene:
        raise SelftestFailure(f"人手交的应当记无会话、有现场目录，实际会话 {summary['task'].get('session')!r}、现场「{scene}」", world.server_log_lines())
    patience = world.patience(SCENE_GONE_SECONDS)
    while os.path.lexists(scene) and patience.waiting():
        time.sleep(POLL_SECONDS)
    if os.path.lexists(scene):
        raise SelftestFailure(f"没有会话的任务结局满保留期之后现场 {scene} 应当删了，实际还在", world.server_log_lines())
    return "人手交的没起 wait，现场照结局时刻满保留期删了"


def cell_merge_on_peer_reads_shard_outputs(world):
    """合并派到第二台（task_server_lib.py 文件头「分片」③）：本机放不下，计划、两片与合并都在第二台跑（假第二台照真的那样在这一刻的工作目录里起假分片入口）；
    合并那一次的环境里有 TASK_SERVER_SHARD_OUTPUTS，读得到两片的产物，整件成了。第 1 片第一次尝试先睡 3 秒：第 0 片先成、第 1 片还在跑的那一段够长，
    弄坏开关 merge-before-all-green 下合并在那一段里起、只读到第 0 片（两片几乎同时收尾时合并等不等都读到两片，这一格就不红）。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下 4G：都派第二台
    world.start_server()
    summary = world.finished(world.submit(world.write_spec("spec-merge-on-peer.json", {"shards": [{}, {"first_attempt_delay": 3}]}), cwd=world.repo), timeout=120)
    merge_hosts = [attempt.get("host") for child in summary.get("children", []) if child["task"].get("role") == "merge" for attempt in child["attempts"]]
    merged = world.sharded_calls("merge", present="0,1")
    if summary["task"].get("outcome") != "succeeded" or merge_hosts != ["peer"] or not merged:
        raise SelftestFailure(f"合并派到第二台时应当读得到 TASK_SERVER_SHARD_OUTPUTS、两片都在、整件成了，实际整件 {summary['task'].get('outcome')}（{summary['task'].get('note')}）、"
                              f"合并各次的台 {merge_hosts}、合并的记录 {merged}", world.server_log_lines())
    return "计划、两片与合并都派到第二台：合并那一次读到 TASK_SERVER_SHARD_OUTPUTS、两片都在，整件成了"


def cell_shard_outputs_written_by_two_drivers(world):
    """合并读的 outputs.tsv 由两个线程同时写（task_server.py 的 write_outputs：子任务收尾那一侧的 advance_parent 与每一轮对账的 drive_sharded_parents 都推到这一步）：
    直接调 write_outputs，第一个线程写好自己的临时文件、改名之前，让第二个线程把同一份 outputs.tsv 整个写完、改名，第一个线程再改名：两个线程都不抛错、
    outputs.tsv 是完整的两行、目录里不留临时文件。弄坏开关 outputs-temporary-name-shared 下两个线程共用一个临时文件名，第一个线程改名时临时文件已被第二个线程改走，
    抛 FileNotFoundError，这一格红。"""
    directory = os.path.join(world.root, "outputs-two-drivers")
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(attempts_of=lambda shard_id: [({"host": "local", "shard_output": f"/shared/output-{shard_id}"}, True)])
    fake.shard_directory = lambda parent_id: directory
    parent = {"id": 7}
    shards = [{"id": 11, "shard_index": 0}, {"id": 12, "shard_index": 1}]
    problems = []
    second_started = threading.Event()
    real_replace = os.replace

    def write_as_second_driver():
        try:
            SERVER.Scheduler.write_outputs(fake, parent, shards)
        except Exception as error:   # 这一格判的就是两个线程抛不抛
            problems.append(f"后写完的那个线程抛了 {type(error).__name__}: {error}")

    def replace_after_second_driver(source, target):
        if not second_started.is_set():
            second_started.set()
            second = threading.Thread(target=write_as_second_driver)
            second.start()
            second.join()
        real_replace(source, target)

    with unittest.mock.patch.object(os, "replace", replace_after_second_driver):
        try:
            SERVER.Scheduler.write_outputs(fake, parent, shards)
        except Exception as error:
            problems.append(f"先写好临时文件的那个线程改名时抛了 {type(error).__name__}: {error}")
    expected = "0\tlocal\t/shared/output-11\n1\tlocal\t/shared/output-12\n"
    text = read_text(os.path.join(directory, "outputs.tsv"))
    leftovers = sorted(name for name in os.listdir(directory) if name.endswith(".partial")) if os.path.isdir(directory) else []
    if text != expected or leftovers:
        problems.append(f"outputs.tsv 应当是 {expected!r}、目录里不留临时文件，实际 {text!r}、留着 {leftovers}")
    if problems:
        raise SelftestFailure("两个线程同时写 outputs.tsv：" + "；".join(problems))
    return "两个线程同时写 outputs.tsv：各用自己的临时文件改名，都不抛错，outputs.tsv 完整、不留临时文件"


PROBE_CHILD_GONE_SECONDS = 5.0   # stop-ends-probe-children：服务退出之后至多等这么久看探测子进程没了


def cell_stop_ends_probe_children(world):
    """停服务时服务正有一条复核挂载的 ssh 子进程（假 ssh 的 fake-ssh-hang：stat -f 那一句睡着不退）：服务收到 TERM 之后停掉它再退（task_server.py 的
    stop_own_children），服务退出之后那个进程号不在了。弄坏开关下留着的那一个由这一格按它写下的进程号停掉（只停这一格的假 ssh）。"""
    world.start_server()
    hang = os.path.join(world.world, "fake-ssh-hang")
    open(hang, "w").close()
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while not os.path.exists(hang + "-pid") and patience.waiting():
        time.sleep(POLL_SECONDS)
    try:
        with open(hang + "-pid", encoding="utf-8") as handle:
            probe = int(handle.read().strip())
    except (OSError, ValueError):
        raise SelftestFailure(f"等了 {STATE_TIMEOUT_SECONDS:.0f} 秒，服务的 NFS 复核没起到假 ssh 复核挂载那一句", world.server_log_lines()) from None
    world.stop_server()
    patience = world.patience(PROBE_CHILD_GONE_SECONDS)
    while not process_gone(probe) and patience.waiting():
        time.sleep(POLL_SECONDS)
    if not process_gone(probe):
        with contextlib.suppress(OSError):
            with open(f"/proc/{probe}/cmdline", "rb") as handle:
                if b"sleep" in handle.read():
                    os.kill(probe, signal.SIGKILL)   # 只停这一格的假 ssh 写下的那一个进程号
        raise SelftestFailure(f"服务退出之后 {PROBE_CHILD_GONE_SECONDS:.0f} 秒，它起的复核挂载子进程（进程号 {probe}）应当已停，实际还在", world.server_log_lines())
    return f"停服务时正在跑的复核挂载子进程（进程号 {probe}）由服务停掉了，服务退出之后它不在了"


SCENE_GONE_SECONDS = 10.0   # 取走之后至多等这么久看现场删了


def cell_failure_scene_kept_until_collected(world):
    """判红那一趟的现场（task_server_lib.py 文件头「现场」）：服务给内存包装的现场目录在工作根下 SCENES_ROOT_NAME/<尝试标签>、记进尝试的 scene_directory，
    假包装照它建、在日志里点名；没 wait 之前过了取走之后的时限也留着，wait 取走之后删；提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的取走之后也留着、task clean 删。"""
    world.start_server()
    keep_seconds = SERVER.FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS
    problems = []
    plain_id = world.submit(["bash", "-c", "exit 0", f"fake-wrapper-scene={SCENE_COMMAND_EXIT}"])
    plain = world.finished(plain_id)
    scene, wanted = scene_of(world, plain)
    log_text = ""
    with contextlib.suppress(OSError):
        with open(first_attempt(plain).get("log") or "", encoding="utf-8", errors="replace") as handle:
            log_text = handle.read()
    if plain["task"].get("outcome") != "command-failed" or not scene or os.path.realpath(scene) != os.path.realpath(wanted):
        problems.append(f"退 {SCENE_COMMAND_EXIT} 的尝试应当记 command-failed、现场目录记成 {wanted}，实际 {describe(plain)}、记「{scene}」")
    elif f"现场目录：{scene}" not in log_text:
        problems.append(f"日志里应当点名现场目录 {scene}，实际没有")
    time.sleep(3 * keep_seconds)
    if not scene or not os.path.exists(os.path.join(scene, "left-by-command")):
        problems.append(f"还没 wait 取走结局时现场 {scene} 应当留着（过了取走之后的时限 {keep_seconds:g} 秒也不删），实际不在")
    world.client(["wait", str(plain_id)])
    patience = world.patience(SCENE_GONE_SECONDS)
    while scene and os.path.lexists(scene) and patience.waiting():
        time.sleep(POLL_SECONDS)
    if scene and os.path.lexists(scene):
        problems.append(f"wait 取走结局之后 {SCENE_GONE_SECONDS:.0f} 秒里现场 {scene} 应当删了，实际还在")
    kept_id = world.submit(["bash", "-c", "exit 0", f"fake-wrapper-scene={SCENE_COMMAND_EXIT}"], extra=KEEP_TEMPORARY_EXTRA)
    kept_scene, _ = scene_of(world, world.finished(kept_id))
    world.client(["wait", str(kept_id)])
    time.sleep(3 * keep_seconds)
    if not kept_scene or not os.path.exists(os.path.join(kept_scene, "left-by-command")):
        problems.append(f"设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的现场 {kept_scene} 取走之后也应当留着（等 task clean），实际不在")
    cleaned = world.client(["clean", str(kept_id)])
    if cleaned.returncode != 0 or (kept_scene and os.path.lexists(kept_scene)):
        problems.append(f"task clean 应当退 0、删了现场 {kept_scene}，实际退 {cleaned.returncode}、现场{'还在' if kept_scene and os.path.lexists(kept_scene) else '没了'}、"
                        f"输出 {cleaned.stdout.strip()[-200:]} {cleaned.stderr.strip()[-200:]}")
    if problems:
        raise SelftestFailure("判红那一趟的现场：" + "；".join(problems), world.server_log_lines())
    return "现场在工作根下这一次尝试的目录里、日志点名它、没取走之前留着、wait 取走之后删了；设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的取走之后留着、task clean 删了"


TASK_LOCK_ROUNDS = 1000   # task-locks-released 拿放锁的任务条数：表不随它涨才算淘汰了


def cell_shard_failure_waits_for_running(world):
    """一片 command-failed、一片在跑、一片在排（task_server_lib.py 文件头「分片」的「有一步最终没成」）：兄弟片照排、照跑到底，结局照收：
    第 0 片没成之后再排两轮，父任务没结局、第 1 片照跑、第 2 片照排着不撤；第 2 片记过的峰值 30G 只有第二台放得下（start_three_shards），
    这时把本机换成 slice 50G，它在本机起、跑完；第 1 片放行跑完；都收齐之后父任务才记第 0 片的 command-failed、wait 末行点名 shard=0/3，不合并。
    弄坏开关 shard-siblings-abandoned 下父任务当场收尾、在排的那片撤掉，这一格红。"""
    releases = {name: os.path.join(world.root, f"release-failure-{name}") for name in ("failing", "running")}
    task_id, _ = start_three_shards(world, "failure-waits", [{"block": releases["failing"], "exits": [3]}, {"block": releases["running"]}, {}])
    problems = []
    try:
        open(releases["failing"], "w").close()
        world.wait_for(task_id, lambda summary: shard_outcomes(summary).get(0) == "command-failed", "第 0 片记 command-failed")
        world.next_pass()
        world.next_pass()
        middle = world.summary(task_id)
        if middle["task"].get("state") == "finished" or 1 not in running_shards(middle) or queued_shards(middle) != [2]:
            problems.append(f"第 0 片没成、第 1 片还在跑时：父任务应当还没结局、第 1 片照跑、第 2 片照排着不撤，实际父任务 {middle['task'].get('state')}/{middle['task'].get('outcome')}、"
                            f"在跑的片 {running_shards(middle)}、各片 {shard_outcomes(middle)}")
        world.resources(local=(50, 50), peer_down="自证：第二台此刻用不了")   # 第 2 片（30G）在本机放得下了
    finally:
        open(releases["running"], "w").close()
    finished = world.finished(task_id, timeout=120)
    code, last = wait_last_line(world, task_id)
    outcomes = shard_outcomes(finished)
    third_runs = world.sharded_calls("shard", index=2)
    if (finished["task"].get("outcome") != "command-failed" or code != 3 or " shard=0/3" not in last
            or outcomes != {0: "command-failed", 1: "succeeded", 2: "succeeded"} or not third_runs
            or "merge" in child_roles(finished) or world.sharded_calls("merge")):
        problems.append(f"收齐之后：整件应当是第 0 片的 command-failed、wait 退 3、末行点名 shard=0/3，第 1、2 片都 succeeded（第 2 片照排照跑），不合并；"
                        f"实际 {describe(finished)}、wait 退 {code}、末行「{last}」、各片 {outcomes}、第 2 片跑的记录 {third_runs}、子任务 {child_roles(finished)}、"
                        f"合并的记录 {world.sharded_calls('merge')}")
    if problems:
        raise SelftestFailure("一片没成：" + "；".join(problems), world.server_log_lines())
    return "一片没成：兄弟片照排照跑、结局照收，收齐之后整件才记那一片的 command-failed、不合并"


def cell_shard_cancel_alone_merges(world):
    """人手只取消在排的一片（task cancel <那一片的子任务号>）：只那一片 cancelled，别的片照跑；合并照常起、TASK_SERVER_SHARD_MISSING 点名缺第 2 片，
    假合并判本次未跑（退 77），整件 not-run、末行 step=merge。弄坏开关 shard-cancel-ends-whole（整件收尾、不合并）与 shard-missing-unnamed（合并拿不到缺片）下这一格红。"""
    release = os.path.join(world.root, "release-cancel-alone")
    task_id, children = start_three_shards(world, "cancel-alone", [{"block": release}, {"block": release}, {}])
    problems = []
    try:
        completed = world.client(["cancel", str(children[2])])
        middle = world.summary(task_id)
        if (completed.returncode != 0 or shard_outcomes(middle).get(2) != "cancelled" or middle["task"].get("state") == "finished"
                or sorted(running_shards(middle)) != [0, 1]):
            problems.append(f"取消在排的第 2 片：应当退 0、只那一片 cancelled、父任务没结局、第 0 与 1 片照跑，实际退 {completed.returncode}、各片 {shard_outcomes(middle)}、"
                            f"父任务 {middle['task'].get('state')}/{middle['task'].get('outcome')}、在跑的片 {running_shards(middle)}（{completed.stdout.strip()[-200:]}）")
    finally:
        open(release, "w").close()
    finished = world.finished(task_id)
    code, last = wait_last_line(world, task_id)
    merges = world.sharded_calls("merge")
    if ([(row.get("present"), row.get("missing")) for row in merges] != [("0,1", "2")] or finished["task"].get("outcome") != "not-run" or code != 77
            or " step=merge" not in last or shard_outcomes(finished) != {0: "succeeded", 1: "succeeded", 2: "cancelled"}):
        problems.append(f"合并应当照常起一次、拿到第 0 与 1 片的产物与缺片的点名（missing=2），判本次未跑：整件 not-run、wait 退 77、末行 step=merge；"
                        f"实际合并的记录 {merges}、{describe(finished)}、wait 退 {code}、末行「{last}」、各片 {shard_outcomes(finished)}")
    if problems:
        raise SelftestFailure("只取消一片：" + "；".join(problems), world.server_log_lines())
    return "人手只取消在排的一片：别的片照跑，合并照常起、点名缺的那一片，判本次未跑"


def cell_parent_cancel_stops_every_shard(world):
    """两片在跑、一片在排时取消整件（父任务号）：照旧全停——在排的那片当场 cancelled、一次不起，在跑的两片回来时单元都停了、各记 cancelled，整件 cancelled（退 239），
    不合并。弄坏开关 parent-cancel-spares-running 下在跑的不停，这一格红。"""
    release = os.path.join(world.root, "release-parent-cancel")
    task_id, _ = start_three_shards(world, "parent-cancel", [{"block": release}, {"block": release}, {}])
    problems = []
    try:
        units = [child["attempts"][-1].get("unit", "") for index, child in shard_children(world.summary(task_id)).items() if index in (0, 1)]
        completed = world.client(["cancel", str(task_id)])
        still = [unit for unit in units if world.unit_active(unit)]
        summary = world.summary(task_id)
        if completed.returncode != 0 or summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239 or still:
            problems.append(f"取消整件：应当退 0、整件记 cancelled（退 239）、回来时两片的单元都停了，实际退 {completed.returncode}、"
                            f"整件 {summary['task'].get('outcome')}、还在跑的单元 {still}")
    finally:
        open(release, "w").close()
    for index, child in sorted(shard_children(world.summary(task_id)).items()):
        finished = world.finished(child["task"]["id"])
        if finished["task"].get("outcome") != "cancelled":
            problems.append(f"第 {index} 片应当记 cancelled，实际 {describe(finished)}")
    if world.sharded_calls("shard", index=2) or "merge" in child_roles(world.summary(task_id)) or world.sharded_calls("merge"):
        problems.append(f"在排的第 2 片应当一次不起、取消了的整件不合并，实际第 2 片跑的记录 {world.sharded_calls('shard', index=2)}、子任务 {child_roles(world.summary(task_id))}")
    if problems:
        raise SelftestFailure("取消整件：" + "；".join(problems), world.server_log_lines())
    return "两片在跑、一片在排时取消整件：在排的没起，在跑的单元回来时都停了，整件 cancelled、不合并"


def cell_cancel_after_natural_end(world):
    """取消记了 cancel_requested、放了锁、停单元之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=cancel-unlocked），这时命令自己退 0、结局写回 succeeded：
    取消不许回 stopped（task 退非 0、说明点名 succeeded），结局照旧 succeeded（TS-03）。弄坏开关 cancel-outcome-unchecked 下回 stopped，这一格红。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    release = os.path.join(world.root, "release-natural-end")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"cancel-unlocked:{pause}"})
    task_id = world.submit(blocking_command(release))
    world.running(task_id)
    open(os.path.join(pause, "arm-cancel-unlocked"), "w").close()
    canceller = world.client(["cancel", str(task_id)], background=True)
    try:
        wait_path(world, os.path.join(pause, "reached-cancel-unlocked"), "停在记了取消、放了锁、停单元之前")
        open(release, "w").close()
        finished = world.finished(task_id)
    finally:
        open(release, "w").close()
        open(os.path.join(pause, "release-cancel-unlocked"), "w").close()
    stdout, stderr = world.communicate(canceller, STATE_TIMEOUT_SECONDS)
    if canceller.returncode == 0 or "succeeded" not in stdout + stderr or finished["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"取消停住的这一会儿命令自己退 0、结局写回 succeeded：取消应当退非 0、说明点名 succeeded，结局照旧 succeeded；"
                              f"实际退 {canceller.returncode}、{describe(finished)}（{(stdout + stderr).strip()[-300:]}）", world.server_log_lines())
    return f"取消到的时候命令已自己跑完：task cancel 退 {canceller.returncode}、点名结局 succeeded，不报停了"


def cell_task_locks_released(world):
    """Store.task_lock 用完就从锁表删（task_server.py 的 Store 文件头）：在这一格的临时库上对 TASK_LOCK_ROUNDS 条任务各拿放一次锁，表空着；
    一条任务的锁有人拿着时另一个线程等着、拿着的放下之后它才拿到（删表项不破坏互斥），都放下之后表空着。弄坏开关 task-locks-kept 下表随任务数涨，这一格红。
    只用 task_server.py 的 Store 这一个类，不起服务。"""
    store = SERVER.Store(os.path.join(world.root, "locks.sqlite3"), bytes(32), dict(os.environ))
    for task_id in range(1, TASK_LOCK_ROUNDS + 1):
        with store.task_lock(task_id):
            pass
    after_rounds = len(store.task_locks)
    order = []
    waiter_started = threading.Event()

    def waiter():
        waiter_started.set()
        with store.task_lock(1):
            order.append("waiter")

    with store.task_lock(1):
        thread = threading.Thread(target=waiter, daemon=True)
        thread.start()
        waiter_started.wait(timeout=10)
        time.sleep(0.3)   # 让等的那一个真的等在锁上：这时它不该拿到
        order.append("holder")
    thread.join(timeout=10)
    if after_rounds != 0 or order != ["holder", "waiter"] or thread.is_alive() or store.task_locks:
        raise SelftestFailure(f"{TASK_LOCK_ROUNDS} 条任务各拿放一次锁之后锁表应当是空的，实际 {after_rounds} 项；拿着时另一个线程应当等到放下才拿到，"
                              f"实际先后 {order}、等的线程{'还没退' if thread.is_alive() else '退了'}、最后锁表 {len(store.task_locks)} 项")
    return f"{TASK_LOCK_ROUNDS} 条任务的锁用完锁表空着；拿着时另一个线程等到放下才拿到"


# ── 峰值起步、重试不重编、改主工作区的命令没正常结束（task_server.py 文件头 ②「内存」、①「要处置的提醒」）──
SPACED_ITEM_NAME = "带 空格的 条目"   # shard-child-key-folded-by-words：条目名里带空格（后半截不该漏进分片子任务的键里）
SPACED_ITEM_TAIL = "空格的"

CANCEL_ARTIFACT_WAIT_SECONDS = 20.0   # cancelled-artifacts-removed：等在跑的那一条把编译目录、现场与第二台这一趟的本地目录建出来的安静时限
# 本机那一条：假 cargo 往服务给的 CARGO_TARGET_DIR 写一份产物，演内存包装留下现场（往服务给的 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY 写 4096 字节），然后占着到放行为止。
# 那 4096 字节用 printf 的宽度造、不点名路径：命令里点到共享根外的路径（/dev/zero 这类）提交时就被拒（task_server.py 文件头 ②「提交时拒」）
CANCEL_LOCAL_SCRIPT = ('cargo build && mkdir -p "$RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY" && printf "%4096s" x > "$RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY/left-by-command" && '
                       + HOLD_UNTIL_RELEASED_SCRIPT)


def cell_cancelled_before_peer_trip_built(world):
    """取消时第二台这一趟的本地目录还没建（task_server.py 的 cancelled_attempt_artifacts）：派到第二台的一条，假 multi-host-run.sh 停在建 builds/<标签>/ 之前
    （fake-peer-before-trip=，演真的还在比工具链、查共享根与 ssh），清理回执还是服务写的 not-started；这时 task cancel 退 0、输出里没有「没删掉」，
    假第二台本地没有 builds/<标签>，日志里没有 ⑥ 那一行，任务记 cancelled。
    弄坏开关 cancel-not-started-receipt-unread 下照改前只认 ⑥ 那一行、算没删成、cancel 退 5，这一格红。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 只放得下第二台
    world.start_server()
    release = os.path.join(world.repo, "release-before-peer-trip")   # 一直不放：等被取消
    problems = []
    try:
        task_id = world.submit(["bash", "-c", "exit 0", f"fake-peer-before-trip={os.path.basename(release)}"], cwd=world.repo)
        attempt = world.running(task_id)["attempts"][-1]
        patience = world.patience(CANCEL_ARTIFACT_WAIT_SECONDS)
        while not world.calls_of(task_id, "peer-before-trip"):
            if not patience.waiting():
                raise SelftestFailure(f"第二台那一条（任务 {task_id}）等了 {patience.elapsed():.0f} 秒，假 multi-host-run.sh 没停到建这一趟的本地目录之前", world.server_log_lines())
            time.sleep(POLL_SECONDS)
        label = attempt.get("label") or ""
        receipt = SERVER.read_json_file(attempt.get("peer_cleanup_file") or "") or {}
        completed = world.client(["cancel", str(task_id)])
        log = world.client(["log", str(task_id), "--all"]).stdout
    finally:
        open(release, "w").close()
    shown = completed.stdout + completed.stderr
    trip = os.path.join(world.peer_local, "builds", label)
    if receipt.get("status") != "not-started" or not label:
        problems.append(f"取消之前清理回执应当还是服务写的 not-started、尝试记着标签，实际回执 {receipt}、标签「{label}」")
    if completed.returncode != 0 or "没删掉" in shown:
        problems.append(f"这一趟的本地目录还没建：cancel 应当退 0、不打「没删掉」，实际退 {completed.returncode}、输出「{shown.strip()[-500:]}」")
    if os.path.lexists(trip) or SERVER.PEER_TRIP_REMOVED_MARK in log:
        problems.append(f"假第二台本地不该有 builds/{label}、日志里不该有 ⑥ 那一行（还没建），实际{'有' if os.path.lexists(trip) else '没有'}目录、日志末尾「{log.strip()[-300:]}」")
    outcome = world.summary(task_id)["task"].get("outcome")
    if outcome != "cancelled":
        problems.append(f"任务应当记 cancelled，实际 {outcome}")
    if problems:
        raise SelftestFailure("取消时第二台这一趟的本地目录还没建：" + "；".join(problems), world.server_log_lines())
    return f"清理回执还是 not-started、假第二台还没建 builds/{label} 时取消：cancel 退 0、不报没删掉，没有那份目录、日志里没有 ⑥ 那一行"


def cell_shard_child_key_folded_by_words(world):
    """分片子任务的键照父命令按词折叠（task_server.py 的 create_child）：父命令 --item 后面是带空格的条目名（一个词），计划、片、合并的键都是父任务的键接那一步那一段
    （过同一个换算），后半截「空格的」不漏进键里。弄坏开关 shard-key-from-joined-command 下照改前用空格接父命令的词，后半截漏进键里。"""
    world.resources(local=(30, 30))
    command = world.write_spec("spec-spaced-item.json", {"shards": [{}]}) + ["--item", SPACED_ITEM_NAME]
    world.start_server()
    summary = world.finished(world.submit(command, cwd=world.repo))
    parent_key = summary["task"].get("command_key") or ""
    problems = []
    if summary["task"].get("outcome") != "succeeded" or not summary.get("children"):
        problems.append(f"带空格条目名的分片命令应当拆开跑完，实际 {describe(summary)}")
    for child in summary.get("children", []):
        record = child["task"]
        wanted = PEAKS.command_key([], parent_key + lib.shard_child_key("", record.get("role"), int(record.get("shard_index") or 0), int(record.get("shard_count") or 0),
                                                                        compiles=bool(record.get("compiles"))))
        key = record.get("command_key") or ""
        if key != wanted or SPACED_ITEM_TAIL in key.split():
            problems.append(f"{record.get('role')} {record.get('shard_index')} 的键应当是「{wanted}」（父任务的键接那一段），实际「{key}」")
    if SPACED_ITEM_TAIL in parent_key.split():
        problems.append(f"父任务的键里不该有条目名的后半截，实际「{parent_key}」")
    if problems:
        raise SelftestFailure("分片子任务的键按词折叠：" + "；".join(problems), world.server_log_lines())
    return f"条目名带空格的分片命令：{len(summary.get('children', []))} 个子任务的键都是父任务的键「{parent_key}」接那一步那一段"


CHILD_SETTLE_HOLD_SECONDS = 3.0   # parent-cancel-waits-for-children：对账停在收尾子任务那一次的开头，看这么久里取消回没回话


def cell_parent_cancel_waits_for_children(world):
    """取消在跑的分片整件（一片在跑）：停了那一片的单元之后，对账收尾那一次之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=attempt-finishing），
    CHILD_SETTLE_HOLD_SECONDS 秒里 task cancel 不该回话（task_server.py 的 cancel_children 等子任务在库里记成 finished）；放开之后回 stopped、退 0，
    回话那一刻各子任务都已 finished、整件 cancelled，整件的结局时刻不早于任何一个子任务。弄坏开关 parent-finalized-before-children 下停了单元就回，这一格红。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    release = os.path.join(world.root, "release-cancel-waits")
    command = world.write_spec("spec-cancel-waits.json", {"shards": [{"block": release}]})
    world.resources(local=(30, 30))
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"attempt-finishing:{pause}"})
    task_id = world.submit(command, cwd=world.repo)
    problems = []
    canceller = None
    try:
        world.wait_for(task_id, lambda summary: len(shard_children(summary)) == 1 and len(world.sharded_calls("shard")) == 1 and all(
            any(attempt.get("state") == "running" for attempt in child["attempts"]) for child in shard_children(summary).values()), "那一片跑起来")
        open(os.path.join(pause, "arm-attempt-finishing"), "w").close()
        canceller = world.client(["cancel", str(task_id)], background=True)
        wait_path(world, os.path.join(pause, "reached-attempt-finishing"), "停了那一片的单元之后、对账收尾那一次之前停住")
        early = exited_within({"task cancel": canceller}, CHILD_SETTLE_HOLD_SECONDS)
        if early:
            middle = world.summary(task_id)
            problems.append(f"对账还没收尾那一片时 task cancel 不该回话（要等子任务在库里记成 finished），实际 {CHILD_SETTLE_HOLD_SECONDS:.0f} 秒里就回了（退 {canceller.returncode}）、"
                            f"整件 {middle['task'].get('state')}/{middle['task'].get('outcome')}、各子任务 {[(child['task'].get('role'), child['task'].get('state')) for child in middle.get('children', [])]}")
    finally:
        open(os.path.join(pause, "release-attempt-finishing"), "w").close()
        open(release, "w").close()   # 弄坏开关下单元没停的话放它跑完，收尾不等满时限
    stdout, stderr = world.communicate(canceller, STATE_TIMEOUT_SECONDS)
    replied = world.summary(task_id)
    children = replied.get("children", [])
    unsettled = [(child["task"].get("role"), child["task"]["id"], child["task"].get("state")) for child in children if child["task"].get("state") != "finished"]
    if not problems and (canceller.returncode != 0 or replied["task"].get("outcome") != "cancelled" or unsettled):
        problems.append(f"放开之后 task cancel 应当退 0、回话那一刻整件 cancelled、各子任务都已 finished，实际退 {canceller.returncode}、整件 {describe(replied)}、"
                        f"还没收尾的子任务 {unsettled}（{(stdout + stderr).strip()[-300:]}）")
    finished = world.finished(task_id)
    parent_at = float(finished["task"].get("finished_at") or 0)
    later = [(child["task"]["id"], child["task"].get("finished_at")) for child in world.summary(task_id).get("children", [])
             if float(child["task"].get("finished_at") or 0) > parent_at]
    if later:
        problems.append(f"整件的结局时刻 {parent_at} 不该早于子任务，实际比它晚收尾的子任务 {later}")
    if problems:
        raise SelftestFailure("取消整件等子任务收尾：" + "；".join(problems), world.server_log_lines())
    return f"取消在跑的分片整件：对账收尾那一片之前 task cancel 等着，放开之后回 stopped，回话时各子任务都已 finished、整件结局不早于子任务"


def exited_within(processes, seconds):
    """seconds 秒里 processes（{名: Popen}）有哪几个退了：→ [名…]（照给的次序），一有退的就交；到点都还在跑交空的。"""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        done = [name for name, process in processes.items() if process.poll() is not None]
        if done:
            return done
        time.sleep(POLL_SECONDS)
    return [name for name, process in processes.items() if process.poll() is not None]


CARGO_CLASS_BASE = ("cargo", "test", "-p", "singlefs-verification", "--release")   # cargo-test-key-by-package：同包同档的那一类
CARGO_CLASS_PEAK_BYTES = 3 * GIBIBYTE   # 头一条量到的峰值：比没记录的 4096M 小，第二条照它起就是 3072M
CARGO_CLASS_FAKE_WORDS = ("fake-wrapper-exits=0",)   # 假包装照它退 0、不起 cargo；它在 cargo test 的位置参数上（测试名筛选），不进键


def cell_cargo_test_key_by_package(world):
    """cargo test 同包同特性同档归一个键（task_server_lib.py 的 cargo_test_class_words、task_server.py 的 command_key；点名验证包测试目标的每条新命令各从 4096M 起、
    撞三四次才编得过）：cargo test -p singlefs-verification --release --test alpha -- 用例 --exact 跑成、量到 3G；同包同档、点名 --test beta 的那一条命令键与它相同、
    头一次照它的记录起 3072M；再加 --features selftest-extra 的那一条键不同、没有记录，照 4096M 起。键里不带 --test 的值与 `--` 之后的用例；别的命令的词照原样。
    弄坏开关 cargo-test-key-whole-command 下整条命令当键，点名 beta 的那一条也照 4096M 起，这一格红。"""
    world.resources(local=(100, 100))
    world.start_server()
    first = world.finished(world.submit([*CARGO_CLASS_BASE, "--test", "selftest-target-alpha", "--", "selftest_case", "--exact",
                                         f"fake-wrapper-exit-peak={CARGO_CLASS_PEAK_BYTES}", *CARGO_CLASS_FAKE_WORDS]))
    second = world.finished(world.submit([*CARGO_CLASS_BASE, "--test", "selftest-target-beta", *CARGO_CLASS_FAKE_WORDS]))
    third = world.finished(world.submit([*CARGO_CLASS_BASE, "--features", "selftest-extra", "--test", "selftest-target-beta", *CARGO_CLASS_FAKE_WORDS]))
    keys = [summary["task"].get("command_key") or "" for summary in (first, second, third)]
    caps = [first_attempt(summary).get("memory_cap_text") for summary in (second, third)]
    other = ["bash", "-c", "exit 0", "--test", "selftest-target-alpha"]
    problems = []
    if any(summary["task"].get("outcome") != "succeeded" for summary in (first, second, third)):
        problems.append(f"三条都应当跑成，实际 {[describe(summary) for summary in (first, second, third)]}")
    if keys[0] != keys[1] or keys[1] == keys[2] or "selftest-target" in keys[0] or "selftest_case" in keys[0]:
        problems.append(f"点名 alpha 与 beta 的同包同档两条应当同一个键、键里不带测试目标与用例，加了特性的另一个键；实际 {keys}")
    if caps != [f"{CARGO_CLASS_PEAK_BYTES // MEBIBYTE}M", UNRECORDED_DEFAULT_PINNED_TEXT]:
        problems.append(f"点名 beta 的那一条应当照 alpha 量到的 3G 起 {CARGO_CLASS_PEAK_BYTES // MEBIBYTE}M、加了特性的照没记录的 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，实际 {caps}")
    if lib.cargo_test_class_words(other) != other:
        problems.append(f"不是 cargo test 的命令应当照原样，实际 {lib.cargo_test_class_words(other)}")
    if problems:
        raise SelftestFailure("cargo test 同包同特性同档归一个键：" + "；".join(problems), world.server_log_lines())
    return f"点名不同测试目标的同包同档两条共用键「{keys[0]}」、第二条照 3G 起 {caps[0]}；加了特性的另一个键、照 {caps[1]} 起"


def cell_selection_key_by_script(world):
    """本仓门禁与入口脚本点名不同格、不同项归一个键（task_server_lib.py 的 selection_class_words、task_server.py 的 command_key）：
    bash .claude/gate.d/doc-text.sh --check date-once 与 --check=trilingual-files 两条的词相同、都是「--check *」；--item 与 --only 照同样去值；
    不带选法的整道照原样，与点名的不是一个键；不在本仓脚本目录下的命令（cargo fmt --check）照原样。
    弄坏开关 selection-key-whole-command 下照原样，点名不同格的两条词不同，这一格红。"""
    gate = ["bash", ".claude/gate.d/doc-text.sh"]
    first = lib.selection_class_words([*gate, "--check", "date-once"])
    second = lib.selection_class_words([*gate, "--check=trilingual-files"])
    entry = ["env", "A=1", "bash", "/repository/scripts/crash/crash-amplification.sh"]
    items = lib.selection_class_words([*entry, "--item", "one", "--item", "two"])
    whole = lib.selection_class_words(gate)
    other = ["cargo", "fmt", "--check"]
    problems = []
    if first != second or first != [*gate, "--check", "*"]:
        problems.append(f"点名不同格的两条应当同为 {[*gate, '--check', '*']}，实际 {first} 与 {second}")
    if items != [*entry[:-1], "scripts/crash/crash-amplification.sh", "--item", "*"]:
        problems.append(f"点名两项的入口应当命令词折成仓内相对路径、去值只留一个「--item *」，实际 {items}")
    if whole != gate or whole == first:
        problems.append(f"不带选法的整道应当照原样、与点名的不同，实际 {whole}")
    if lib.selection_class_words(other) != other:
        problems.append(f"不是本仓脚本的命令应当照原样，实际 {lib.selection_class_words(other)}")
    if problems:
        raise SelftestFailure("本仓门禁与入口脚本点名不同格、不同项归一个键：" + "；".join(problems), [])
    return f"点名不同格的两条共用「{' '.join(first)}」，点名两项的入口共用「--item *」，整道与别的命令照原样"


CRASH_GATE_WORDS = ["bash", ".claude/gate.d/crash-point-replay.sh"]   # selection-key-positionals-after-options、plan-key-without-shrunk-domain：本仓崩溃点重放门禁的命令词
CRASH_CASE_SELECTION = ["--check", "crash-case-replay", "--crash-cases-only", "--item", "crash-case:selftest-one"]   # 同上：点崩溃枚举用例的那几个选项
SHRUNK_DOMAIN_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"   # plan-key-without-shrunk-domain：缩域变量（lib.PLAN_KEY_NEUTRAL_VARIABLES 点名的）


def cell_selection_key_positionals_after_options(world):
    """本仓门禁与入口脚本命令里不以 - 起头的位置参数统一排在选项之后进键（task_server_lib.py 的 selection_class_words、task_server.py 的 command_key）：同一条崩溃枚举用例命令，
    仓根（副本目录）写在选项前与写在选项后，两条的键相同、键里只有一个〈文件〉、没有用例名；--crash-expand-up-to 与它的值是一对、值不跟着挪，由 memory_peaks_lib.command_key
    连值去掉，带它的那条键也相同；lib 的 SELECTION_ATTACHED_VALUE_OPTIONS 与 memory_peaks_lib 的 SELECTION_VALUE_OPTIONS 是同一组。
    弄坏开关 selection-positionals-in-place 下位置参数留在原位，仓根写的位置不同键就不同，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peaks = dict(os.environ), PEAKS
    copy = os.path.join(world.root, "positional-key-copy")
    shapes = {"仓根在选项前": [*CRASH_GATE_WORDS, copy, *CRASH_CASE_SELECTION],
              "仓根在选项后": [*CRASH_GATE_WORDS, *CRASH_CASE_SELECTION, copy],
              "带 --crash-expand-up-to": [*CRASH_GATE_WORDS, "--crash-expand-up-to", "3", *CRASH_CASE_SELECTION, copy]}
    keys = {name: SERVER.Scheduler.command_key(fake, command, None) for name, command in shapes.items()}
    moved = lib.selection_class_words([*CRASH_GATE_WORDS, "--crash-expand-up-to", "3", copy, "--item", "x"])
    problems = []
    if len(set(keys.values())) != 1 or PEAKS.FILE_TOKEN not in keys["仓根在选项前"] or "crash-case:" in keys["仓根在选项前"] or "--crash-expand-up-to" in keys["带 --crash-expand-up-to"]:
        problems.append(f"仓根写在选项前、写在选项后与带 --crash-expand-up-to 的三条应当同一把键、键里有{PEAKS.FILE_TOKEN}、没有用例名与 --crash-expand-up-to，实际 {keys}")
    if moved != [*CRASH_GATE_WORDS, "--crash-expand-up-to", "3", "--item", "*", copy]:
        problems.append(f"位置参数应当排在选项与「--item *」之后、--crash-expand-up-to 与它的值留在一起，实际 {moved}")
    if set(lib.SELECTION_ATTACHED_VALUE_OPTIONS) != set(PEAKS.SELECTION_VALUE_OPTIONS):
        problems.append(f"lib 的 SELECTION_ATTACHED_VALUE_OPTIONS {lib.SELECTION_ATTACHED_VALUE_OPTIONS} 应当与 memory_peaks_lib 的 SELECTION_VALUE_OPTIONS {PEAKS.SELECTION_VALUE_OPTIONS} 同一组")
    if problems:
        raise SelftestFailure("位置参数排在选项之后进键：" + "；".join(problems))
    return f"仓根写在选项前与后、带 --crash-expand-up-to 的三条共用键「{keys['仓根在选项前']}」"


def cell_plan_key_without_shrunk_domain(world):
    """分片计划那一步的命令键不带缩域变量（task_server.py 的 child_key_text、task_server_lib.py 的 key_words 的 plan_step）：两条父命令只差 env 里
    SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=2 与 =3（后一条前面另带只改配置路径的 SINGLEFS_MULTI_HOST_CONFIG），计划那一步的子任务键相同、键里没有这个变量；
    各片的子任务键仍带它，缩域深度不同就不同。弄坏开关 plan-key-shrunk-domain-kept 下计划那一步也带它，两条计划键不同，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peaks = dict(os.environ), PEAKS
    copy = os.path.join(world.root, "plan-key-copy")

    def parent(depth, leading=()):
        command = ["env", *leading, f"{SHRUNK_DOMAIN_VARIABLE}={depth}", *CRASH_GATE_WORDS, *CRASH_CASE_SELECTION, copy]
        return {"command": command, "environment": {}, "compiles": True}

    parents = (parent(2), parent(3, ("SINGLEFS_MULTI_HOST_CONFIG=/selftest/multi-host.env",)))
    plan = [SERVER.Scheduler.child_key_text(fake, task, "plan", 0, 0) for task in parents]
    shard = [SERVER.Scheduler.child_key_text(fake, task, "shard", 0, 4) for task in parents]
    problems = []
    if plan[0] != plan[1] or SHRUNK_DOMAIN_VARIABLE in plan[0]:
        problems.append(f"缩域深度不同的两条父命令，计划那一步的键应当相同、键里没有 {SHRUNK_DOMAIN_VARIABLE}，实际「{plan[0]}」与「{plan[1]}」")
    if shard[0] == shard[1] or f"{SHRUNK_DOMAIN_VARIABLE}=2" not in shard[0] or f"{SHRUNK_DOMAIN_VARIABLE}=3" not in shard[1]:
        problems.append(f"各片的键应当仍带缩域变量与它的值、两条不同，实际「{shard[0]}」与「{shard[1]}」")
    if problems:
        raise SelftestFailure("分片计划那一步的键不随缩域深度变：" + "；".join(problems))
    return f"缩域深度不同的两条父命令共用计划键「{plan[0]}」，各片的键带着缩域变量各是一个"


def cell_entry_script_key_by_repository_path(world):
    """本仓入口脚本写成绝对路径与仓内相对路径落一把命令键，--fingerprint-of 与它的值不进键（task_server_lib.py 的 selection_class_words、repository_script_word，
    task_server.py 的 command_key，--fingerprint-of 由 memory_peaks_lib.command_key 去掉）：bash scripts/gate/merge-and-verify.sh --item <项> <补丁目录> 与
    bash <仓根>/scripts/gate/merge-and-verify.sh --item <另一项> --fingerprint-of <日志目录> <补丁目录> 两条键相同，键里是仓内相对路径、没有 --fingerprint-of；
    ../ 起头的相对路径同样折；不在本仓脚本目录下的绝对路径命令照原样。弄坏开关 entry-script-absolute-path-kept 下绝对路径照原样进键，两条键不同，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peaks = dict(os.environ), PEAKS
    patches = os.path.join(world.root, "entry-key-patches")
    relative = ["bash", "scripts/gate/merge-and-verify.sh", "--item", "singlefs-verification:src/lib.rs", patches]
    absolute = ["bash", os.path.join(lib.REPOSITORY_ROOT, "scripts", "gate", "merge-and-verify.sh"), "--item", "singlefs-verification:tests/other.rs",
                "--fingerprint-of", os.path.join(world.root, "entry-key-log"), patches]
    keys = [SERVER.Scheduler.command_key(fake, command, None) for command in (relative, absolute)]
    climbed = lib.selection_class_words(["bash", "../singlefs/scripts/gate/merge-and-verify.sh", "--item", "x"])
    other = ["/usr/bin/env", "true"]
    problems = []
    if keys[0] != keys[1] or "scripts/gate/merge-and-verify.sh" not in keys[0] or PEAKS.REPOSITORY_TOKEN in keys[0] or "--fingerprint-of" in keys[1]:
        problems.append(f"相对路径与绝对路径加 --fingerprint-of 的两条应当同一把键、键里是仓内相对路径、没有 --fingerprint-of，实际 {keys}")
    if climbed != ["bash", "scripts/gate/merge-and-verify.sh", "--item", "*"]:
        problems.append(f"../ 起头的相对路径应当折成仓内相对路径，实际 {climbed}")
    if lib.selection_class_words(other) != other:
        problems.append(f"不在本仓脚本目录下的命令应当照原样，实际 {lib.selection_class_words(other)}")
    if problems:
        raise SelftestFailure("本仓入口脚本按仓内相对路径落键：" + "；".join(problems), [])
    return f"相对路径与绝对路径加 --fingerprint-of 的两条共用键「{keys[0]}」"


def cell_shard_key_without_count(world):
    """片的命令键只带片号、不带片数（task_server_lib.py 的 shard_child_key；admission 审核 F7）：同一件第 3 片在共 5 片与共 64 片时键相同（过了键的换算也相同），
    与第 2 片不同。弄坏开关 shard-key-count-kept 下键里带片数、片数一变就换新键，这一格红。"""
    base = "bash .claude/gate.d/selftest-gate.sh"
    five, sixty_four, other = (PEAKS.command_key([], lib.shard_child_key(base, "shard", index, count)) for index, count in ((3, 5), (3, 64), (2, 64)))
    if five != sixty_four or five == other:
        raise SelftestFailure(f"第 3 片共 5 片与共 64 片的键应当相同、与第 2 片不同，实际「{five}」「{sixty_four}」「{other}」")
    return f"第 3 片的键不随片数变：「{five}」"


def cell_compiles_declared_entry(world):
    """入口会编的声明（task_server_lib.py 的 declared_compiles、task_server.py 的 compiles；lifecycle 审核 F1）：入口文件头写「# task-server-compiles: yes」的，
    命令那一层认不出 cargo 也算会编（服务照给编译目录、撞上限重排接着用上一次那一份）；没写的不算；只带 --selftest 的不算；写成别的词的交写坏的那一句（派活打回）。
    弄坏开关 compiles-declaration-ignored 下声明了也不算，这一格红。"""
    def entry(name, header):
        path = os.path.join(world.root, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n" + header + "exit 0\n")
        return path
    declared, plain, written_wrong = (entry(name, header) for name, header in (("entry-compiles.sh", f"{lib.DECLARED_COMPILES_MARK} {lib.COMPILES_DECLARED_WORD}\n"),
                                                                              ("entry-plain.sh", ""), ("entry-wrong.sh", f"{lib.DECLARED_COMPILES_MARK} maybe\n")))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.cargo_lock = types.SimpleNamespace(cargo_invocations=lambda *arguments: [], BUILDING_SUBCOMMANDS={"build"})
    fake.project_code_runs = types.SimpleNamespace(shell_words=None)

    def task(path, *rest):
        return {"command": ["bash", path, *rest], "cwd": world.root, "environment": {}}
    problems = []
    for what, found, wanted in (("声明了会编的入口", SERVER.Scheduler.compiles(fake, task(declared, "--item", "selftest")), True),
                                ("没声明的入口", SERVER.Scheduler.compiles(fake, task(plain, "--item", "selftest")), False),
                                ("声明了、只带 --selftest", SERVER.Scheduler.compiles(fake, task(declared, "--selftest")), False)):
        if found != wanted:
            problems.append(f"{what}：会编应当 {wanted}，实际 {found}")
    wrong = SERVER.Scheduler.compiles_declared(fake, task(written_wrong))
    if wrong[0] or "maybe" not in wrong[1]:
        problems.append(f"写成 maybe 的：应当不算、交写坏的那一句，实际 {wrong}")
    if problems:
        raise SelftestFailure("入口会编的声明：" + "；".join(problems))
    return "声明了 yes 的入口算会编；没声明的、只自证的不算；写坏的交那一句"


def cell_parent_cancelling_child_not_claimed(world):
    """整件正在取消时排着的子任务不起（task_server.py 的 Store.claim_attempt；admission 审核 F10）：父任务记了 cancel_requested、阶段 cancelling，排着的那一片照
    claim_attempt 起：抛 StoreRefused、kind 是 parent-cancelling，库里没插尝试。服务撤的片理由行照 cancel_origin 写（task_server_lib.py 的 reason_fields）：
    hard-limit、sibling、parent 各写服务撤的那一句，不写提交它的会话取消。
    弄坏开关 parent-cancel-unchecked-at-claim 下照起，cancel-origin-unread 下一律写提交它的会话，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "claim-parent-cancelling.sqlite3"), bytes(32), dict(os.environ))
    parent = store.insert("tasks", {"state": "sharded", "session": "", "command_key": "", "task_id": 0, "cancel_requested": True, "phase": "cancelling"})
    child = store.insert("tasks", {"state": "queued", "session": "", "command_key": "", "task_id": parent["id"], "parent_id": parent["id"], "role": "shard",
                                   "shard_index": 0, "attempt_count": 0, "cancel_requested": False})
    problems = []
    kind = ""
    try:
        store.claim_attempt(child["id"], 1, {"state": "starting", "session": "", "command_key": ""}, {"state": "running"})
    except SERVER.StoreRefused as refused:
        kind = refused.kind
    if kind != "parent-cancelling" or store.attempts_of(child["id"]):
        problems.append(f"父任务整件正在取消：排着的片应当不起（parent-cancelling），实际 kind「{kind}」、库里尝试 {len(store.attempts_of(child['id']))} 次")
    for origin in ("hard-limit", "sibling", "parent"):
        fields = dict(lib.reason_fields({"id": child["id"], "outcome": "cancelled", "cancel_origin": origin, "note": "自证"}, [], ""))
        if fields.get("by") != lib.CANCEL_ORIGIN_TEXTS[origin] or "服务撤的" not in str(fields.get("by")):
            problems.append(f"cancel_origin={origin} 的理由行应当写「{lib.CANCEL_ORIGIN_TEXTS[origin]}」，实际「{fields.get('by')}」")
    if problems:
        raise SelftestFailure("整件取消时不起子任务、服务撤的写对理由：" + "；".join(problems))
    return "父任务正在取消时排着的片不起；服务撤的片理由行写服务撤的那一句"


SHARD_PEAK_KEY = "selftest-shard-peak-count"   # shard-peak-record-count-matched：库里记过峰值的那把片的命令键


def cell_shard_peak_record_count_matched(world):
    """定额度读片的记录只认片数相同的那几趟（task_server.py 的 record_fits_task、recorded_measurements、stopped_peak_floor；片的键不带片数，两片那一趟的第 0 片记过 49.5G，
    换成几十片之后同一片号落在同一把键上）：共 7 片的一片，库里同一把键有两片那一趟跑成三次各 49.5G（尝试自己记了 shard_count）、7 片那一趟跑成 5G（尝试记了 shard_count）、
    6G（尝试没记、读子任务的 shard_count 是 7）、尝试没记而子任务是两片那一趟的 48G、尝试没记而子任务读不到的 47G，另有被取消的两片那一趟 30G、7 片那一趟 2G：
    近几次的量应当只有 5G 与 6G，被停的下限是 2G；不是片的任务照旧三样都认。静态核，不起服务。
    弄坏开关 record-shard-count-unchecked 下片数不同的也拿来，这一格红。"""
    def ended(outcome, peak, ended_at, **fields):
        return ({"command_key": SHARD_PEAK_KEY, "outcome": outcome, "memory_peak": int(peak), "ended_at": ended_at, "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC, **fields}, True)
    rows = ([ended("succeeded", 49.5 * GIBIBYTE, 100 + number, shard_count=2) for number in range(3)]
            + [ended("succeeded", 5 * GIBIBYTE, 103, shard_count=7), ended("succeeded", 6 * GIBIBYTE, 104, task_id=71),
               ended("succeeded", 48 * GIBIBYTE, 105, task_id=72), ended("succeeded", 47 * GIBIBYTE, 106, task_id=73),
               ended("cancelled", 30 * GIBIBYTE, 107, shard_count=2), ended("cancelled", 2 * GIBIBYTE, 108, shard_count=7)])
    shard_counts = {71: 7, 72: 2}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(
        select=lambda table, where="1", parameters=(), order="id": [row for row in rows if row[0].get("command_key") == parameters[0]] if table == "attempts" else [],
        task=lambda task_id: ({"id": task_id, "shard_count": shard_counts[task_id]}, True) if task_id in shard_counts else None)
    problems = []
    piece = {"role": "shard", "shard_count": 7, "command_key": SHARD_PEAK_KEY}
    measured = sorted(value for _, value in SERVER.Scheduler.recorded_measurements(fake, piece))
    floor = SERVER.Scheduler.stopped_peak_floor(fake, piece)
    if measured != [5 * GIBIBYTE, 6 * GIBIBYTE] or floor != 2 * GIBIBYTE:
        problems.append(f"共 7 片的一片：近几次的量应当只有 5G 与 6G、被停的下限 2G，实际量 {[round(value / GIBIBYTE, 1) for value in measured]}G、下限 {None if floor is None else round(floor / GIBIBYTE, 1)}G")
    plain = {"role": "", "command_key": SHARD_PEAK_KEY}
    measured_plain = sorted(value for _, value in SERVER.Scheduler.recorded_measurements(fake, plain))
    floor_plain = SERVER.Scheduler.stopped_peak_floor(fake, plain)
    if len(measured_plain) != 7 or floor_plain != 30 * GIBIBYTE:
        problems.append(f"不是片的任务照旧不看片数：应当 7 个量、被停的下限 30G，实际 {len(measured_plain)} 个量、下限 {None if floor_plain is None else round(floor_plain / GIBIBYTE, 1)}G")
    if problems:
        raise SelftestFailure("定额度读片的记录只认片数相同的：" + "；".join(problems))
    return "共 7 片的一片只取 7 片那几趟的 5G、6G（下限 2G），两片那一趟与读不到片数的不取；不是片的照旧都取"


PLAN_KEY_HEAD = f"bash {NO_BORROW_ENTRY_SCRIPT} --check selftest-plan-keys"   # plan-key-by-compiles：这一趟分片的命令键原文（计划那一段之前）
PLAN_KEY_COMPILED_BYTES = 9 * GIBIBYTE     # 编的那一种计划步记下的峰值
PLAN_KEY_UNCOMPILED_BYTES = 2 * GIBIBYTE   # 不编的那一种计划步记下的峰值


def cell_plan_key_by_compiles(world):
    """计划那一步的命令键带「这一步编不编」（task_server_lib.py 的 shard_child_key；task_server.py 的 child_compiles、child_key_text）：同一父键，编与不编两种计划步落两把键，
    过了键的换算（memory_peaks_lib 的 command_key）仍不同；定额度各读各的（task_server.py 的 computed_prediction）：库里只有不编的那把键跑完 2G 时，
    编的那一种照没有记录的缺省 4096M 起（unrecorded）、不编的取 2G；两把键各有记录（编的 9G、不编的 2G）时各取各的。套在假调度器上，不起服务。
    弄坏开关 plan-key-compile-unmarked 下两种计划步落同一把键：编的那一种取了不编的 2G、不编的那一种取了编的 9G，这一格红。"""
    store_rows = []

    def select(table, where="1", parameters=(), order="id"):
        return [item for item in store_rows if item[0].get("command_key") == parameters[0]] if table == "attempts" else []

    def seed(key, peak, when):
        store_rows.append(({"command_key": key, "state": "ended", "outcome": "succeeded", "memory_peak": peak, "memory_cap": peak + GIBIBYTE,
                            "role": "plan", "task_id": 80, "ended_at": float(when), "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}, True))

    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), lib.REPOSITORY_ROOT   # 定额度末了的内存下限（memory_floor_applied）读命令与仓根
    fake.host_dispatch = loaded_host_dispatch()
    fake.reads_log = None
    fake.prediction_lock = threading.Lock()
    fake.store = types.SimpleNamespace(select=select, task=lambda task_id: None)
    parent = {"compiles": True}
    compiled_key = lib.shard_child_key(PLAN_KEY_HEAD, "plan", 0, 0, compiles=SERVER.Scheduler.child_compiles(parent, "plan"))
    uncompiled_key = lib.shard_child_key(PLAN_KEY_HEAD, "plan", 0, 0, compiles=SERVER.Scheduler.child_compiles(dict(parent, compiles=False), "plan"))
    plans = {what: {"id": number, "role": "plan", "parent_id": 80, "shard_index": 0, "shard_count": 0, "command_key": key,
                    "command": ["bash", NO_BORROW_ENTRY_SCRIPT, "--check", "selftest-plan-keys"], "cwd": lib.REPOSITORY_ROOT}
             for number, (what, key) in enumerate((("编", compiled_key), ("不编", uncompiled_key)), 81)}

    def started():
        found = {what: SERVER.Scheduler.computed_prediction(fake, task) for what, task in plans.items()}
        return {what: (prediction.memory_cap, prediction.memory_history) for what, prediction in found.items()}

    problems = []
    folded = {PEAKS.command_key([], compiled_key), PEAKS.command_key([], uncompiled_key)}
    if compiled_key == uncompiled_key or len(folded) != 2:
        problems.append(f"编与不编的计划步应当是两把键（过了键的换算也是），实际「{compiled_key}」「{uncompiled_key}」、换算后 {sorted(folded)}")
    seed(uncompiled_key, PLAN_KEY_UNCOMPILED_BYTES, 100)
    only_uncompiled = started()
    seed(compiled_key, PLAN_KEY_COMPILED_BYTES, 110)
    both = started()
    for what, got, want in (("库里只有不编的 2G", only_uncompiled, {"编": (lib.UNRECORDED_MEMORY_DEFAULT_BYTES, lib.UNRECORDED_HISTORY),
                                                                    "不编": (PLAN_KEY_UNCOMPILED_BYTES, lib.RECORD_HISTORY)}),
                            ("编的 9G、不编的 2G 各有记录", both, {"编": (PLAN_KEY_COMPILED_BYTES, lib.RECORD_HISTORY), "不编": (PLAN_KEY_UNCOMPILED_BYTES, lib.RECORD_HISTORY)})):
        if got != want:
            problems.append(f"{what}：两种计划步的起点与来历应当是 {want}，实际 {got}")
    if problems:
        raise SelftestFailure("计划那一步编与不编各记各的：" + "；".join(problems))
    return "同一父键编与不编的计划步落两把键：只有不编的 2G 时编的照缺省 4096M 起；各有记录时编的取 9G、不编的取 2G"


def cell_cancelled_artifacts_removed(world):
    """取消的不留产物（task_server.py 文件头「接口」的取消；用户：「取消的任务要自己删掉产物 不要残留」）：本机在跑的会编命令（提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1）
    task cancel 之后现场目录回读不在了，cancel 的输出与日志里写删前删后；本机那一条的编译目录归这棵树的组，放回、不删（task_server_lib.py 文件头「编译目录」）。
    派到第二台在跑的那一条，取消之后假第二台本地的 builds/<标签>/ 与 runs/ 下三个文件不在了（假 multi-host-run.sh 照真的那样被 TERM 停时删、打 ⑥ 那一行），cancel 的输出点名它删了。
    弄坏开关 cancelled-artifacts-kept 下取消之后不删、不报，这一格判红。"""
    world.resources(local=(30, 30), peer=(1, 30))   # 本机那一条在本机起
    world.start_server()
    problems = []
    release = os.path.join(world.root, "release-cancelled-artifacts")   # 一直不放：两条都等被取消
    extra = dict(KEEP_TEMPORARY_EXTRA, **fake_cargo_extra(world))
    local_id = world.submit(["bash", "-c", CANCEL_LOCAL_SCRIPT, "cancel-local", release], extra=extra)
    attempt = world.running(local_id)["attempts"][-1]
    build, scene = attempt.get("build_directory") or "", attempt.get("scene_directory") or ""
    patience = world.patience(CANCEL_ARTIFACT_WAIT_SECONDS)
    while not (build and os.path.isfile(os.path.join(build, "target", "debug", "selftest-artifact")) and scene and os.path.isfile(os.path.join(scene, "left-by-command"))):
        if not patience.waiting():
            raise SelftestFailure(f"本机那一条（任务 {local_id}）等了 {patience.elapsed():.0f} 秒没写出编译目录里的产物与现场：编译目录「{build}」、现场「{scene}」", world.server_log_lines())
        time.sleep(POLL_SECONDS)
    completed = world.client(["cancel", str(local_id)])
    log = world.client(["log", str(local_id), "--all"]).stdout
    wanted = f"取消之后删了任务 {local_id} 第 1 次尝试在本机的现场目录 {scene}：删前 "
    if os.path.lexists(scene) or wanted not in completed.stdout or "删后 不在" not in completed.stdout or wanted not in log:
        problems.append(f"本机那一条取消之后现场目录 {scene} 应当回读不在、cancel 的输出与日志里都有「{wanted}…删后 不在」，"
                        f"实际{'还在' if os.path.lexists(scene) else '不在了'}、cancel 退 {completed.returncode}、输出「{completed.stdout.strip()[-400:]}」")
    if not os.path.isfile(os.path.join(build, "target", "debug", "selftest-artifact")) or f"编译目录 {build}" in completed.stdout:
        problems.append(f"本机那一条取消之后编译目录 {build} 应当放回组里、照旧在（不删、不报删了），"
                        f"实际{'在' if os.path.isdir(build) else '不在了'}、cancel 的输出「{completed.stdout.strip()[-400:]}」")
    if completed.returncode != 0:
        problems.append(f"本机那一条 cancel 应当退 0，实际退 {completed.returncode}：{completed.stderr.strip()[-300:]}")
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 第二台那一条只放得下第二台
    peer_id = world.submit(["bash", "-c", "exit 0", f"fake-peer-wait={os.path.basename(release)}-peer"], cwd=world.repo)
    label = world.running(peer_id)["attempts"][-1].get("label") or ""
    trip = [os.path.join(world.peer_local, "builds", label)] + [os.path.join(world.peer_local, "runs", label + suffix) for suffix in (".pid", ".exit", ".log")]
    patience = world.patience(CANCEL_ARTIFACT_WAIT_SECONDS)
    while not (label and all(os.path.lexists(path) for path in trip)):
        if not patience.waiting():
            raise SelftestFailure(f"第二台那一条（任务 {peer_id}）等了 {patience.elapsed():.0f} 秒，假第二台这一趟的本地目录没建出来：{trip}", world.server_log_lines())
        time.sleep(POLL_SECONDS)
    completed = world.client(["cancel", str(peer_id)])
    log = world.client(["log", str(peer_id), "--all"]).stdout
    left = [path for path in trip if os.path.lexists(path)]
    wanted = f"取消之后删了任务 {peer_id} 第 1 次尝试在第二台的本地目录 第二台这一趟的本地目录 builds/{label}"
    if completed.returncode != 0 or left or wanted not in completed.stdout or wanted not in log or SERVER.PEER_TRIP_REMOVED_MARK not in log:
        problems.append(f"第二台那一条取消之后假第二台本地的 builds/{label} 与 runs/ 下三个文件应当都不在、cancel 退 0、输出与日志里有「{wanted}…」、日志里有 ⑥ 那一行，"
                        f"实际还在 {left}、cancel 退 {completed.returncode}、输出「{completed.stdout.strip()[-400:]}」「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("取消的不留产物：" + "；".join(problems), world.server_log_lines())
    return "本机那一条取消之后现场回读不在、输出与日志写删前删后，编译目录放回组里；第二台那一条取消之后这一趟的本地目录与 runs/ 下三个文件不在、输出点名删了"
