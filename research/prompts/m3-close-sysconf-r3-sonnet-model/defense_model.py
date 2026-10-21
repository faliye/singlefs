#!/usr/bin/env python3
"""辩方腿（m3-close-sysconf-r3）的小模型：纯 Python、不碰 crates、不编译。

四个部分，用法 defense_model.py <rollback|coverage|alpha|stale|all>，输出打到标准输出：
  rollback  取号失败之后的落盘窗口：取号写的三种写法 × 回卷的几种做法，数崩溃落下的状态里「零故障就两槽全废」与
            「再坏一槽就拒」的有多少（前两轮只数过每盘不加故障的下一次挂载，没有再加单故障）
  coverage  取号写的写法（今天一轮、两轮、第二轮只写一块盘）× 盘数 × 三种故障模型下的撞号与拒挂载数
  alpha     离线出路 α' 的「查什么才写」四种规则在版本证据真值表上的结局
  stale     旧快照盘被放行的格：今天的判法、改法乙'、「缺的份数不到 2」单用，在一次发布的各个取下时刻上放行哪些

模型是按 crates/singlefs-core/src/transaction.rs 的写序读出来的抽象（读法写在报告「模型读法」一节），不是入库装置，数不进 kb。
槽的状态：("ok", 世代号, 实例代号) / ("torn",) 读得出而自证不过 / ("rderr",) 设备报读错。
一块盘两槽，槽号 = 世代号 mod 2；写系统配置槽 = 这块盘自证槽里最大世代号 + 1（没有就从 1 起）、槽 = 世代号 mod 2
（transaction.rs write_system_configuration_slot）。写发出时目标槽按「发出那一刻盘上的样子」定死，崩溃只决定落了没落。
崩溃状态 = 某一段（两道屏障之间）里每个写落了 / 没落 / 撕裂（系统配置槽的写才会撕裂），之前各段全落；
故障 = 崩溃之后、下一次可写挂载之前，每盘最多一槽出问题（翻坏 = 撕裂那一类，或持续读错）；另有「全池任意 k 槽」两种更宽的模型。
下一次可写挂载的结局：R2 拒（有槽读错且没有一块盘两槽都自证过）/ 盘全废拒（某块盘一份自证过的都没有）/ 挂得上。
"""
import itertools
import sys
from collections import Counter

PREVIOUS_INSTANCE = 1
NEW_INSTANCE = 2
START_GENERATIONS = (4, 5)


def start_pool(disk_count):
    return [[("ok", START_GENERATIONS[0], PREVIOUS_INSTANCE), ("ok", START_GENERATIONS[1], PREVIOUS_INSTANCE)]
            for _ in range(disk_count)]


def copy_pool(pool):
    return [list(disk) for disk in pool]


def target_of_a_system_configuration_write(slots, instance):
    verified = [slot for slot in slots if slot[0] == "ok"]
    generation = (max(slot[1] for slot in verified) if verified else 0) + 1
    return generation % 2, ("ok", generation, instance)


def canonical(pool):
    return tuple(tuple(disk) for disk in pool)


def classify(pool):
    any_read_error = any(slot[0] == "rderr" for disk in pool for slot in disk)
    some_disk_with_both = any(all(slot[0] == "ok" for slot in disk) for disk in pool)
    if any_read_error and not some_disk_with_both:
        return "R2拒"
    if any(not any(slot[0] == "ok" for slot in disk) for disk in pool):
        return "盘全废拒"
    return "挂得上"


def fault_choices_one_slot_per_disk(disk_count):
    per_disk = [None] + [(slot, kind) for slot in (0, 1) for kind in ("torn", "rderr")]
    return list(itertools.product(per_disk, repeat=disk_count))


def apply_one_slot_per_disk(pool, choice):
    result = copy_pool(pool)
    for disk, fault in enumerate(choice):
        if fault:
            result[disk][fault[0]] = (fault[1],)
    return result


def fault_sets_any_k_slots(disk_count, k):
    places = [(disk, slot) for disk in range(disk_count) for slot in (0, 1)]
    out = []
    for size in range(0, k + 1):
        for chosen in itertools.combinations(places, size):
            for kinds in itertools.product(("torn", "rderr"), repeat=size):
                out.append(tuple(zip(chosen, kinds)))
    return out


def apply_any_k(pool, fault_set):
    result = copy_pool(pool)
    for (disk, slot), kind in fault_set:
        result[disk][slot] = (kind,)
    return result


# ---------------------------------------------------------------- 写序机器

def acquisition_segments(variant, disk_count):
    disks = range(disk_count)
    first = [("sys", d, NEW_INSTANCE) for d in disks]
    if variant == "one_round":
        return [first]
    if variant == "two_rounds":
        return [first, [("sys", d, NEW_INSTANCE) for d in disks]]
    if variant == "extra_pass_on_one_disk":
        return [first, [("sys", 0, NEW_INSTANCE)]]
    raise ValueError(variant)


def resolve(segments, init_pool, fail=None):
    """把每个系统配置写的目标槽按「发出那一刻盘上的样子」定死。fail = (段号, 段内序号, 结局)：
    那个写报错，结局 absent / torn / done 是它在盘上留下的样子；它之后的写不发。交回 (定死的各段, 此刻盘上的样子)。"""
    live = copy_pool(init_pool)
    resolved = []
    for seg_index, segment in enumerate(segments):
        resolved_segment = []
        for op_index, op in enumerate(segment):
            kind, disk, instance = op
            if kind != "sys":
                resolved_segment.append((kind, disk, None, None))
                continue
            slot, value = target_of_a_system_configuration_write(live[disk], instance)
            resolved_segment.append((kind, disk, slot, value))
            if fail is not None and fail[0] == seg_index and fail[1] == op_index:
                if fail[2] == "done":
                    live[disk][slot] = value
                elif fail[2] == "torn":
                    live[disk][slot] = ("torn",)
                resolved.append(resolved_segment)
                return resolved, live
            live[disk][slot] = value
        resolved.append(resolved_segment)
    return resolved, live


def apply_all_done(pool, resolved_segment):
    result = copy_pool(pool)
    for kind, disk, slot, value in resolved_segment:
        if kind == "sys":
            result[disk][slot] = value
    return result


def segment_outcomes(base_pool, resolved_segment, fixed_last=None):
    """一段里每个写落了 / 没落 / 撕裂的全部组合，叠在 base_pool 上；fixed_last 给了就是最后一个写的定死结局。"""
    ops = [op for op in resolved_segment if op[0] == "sys"]
    free = ops[:-1] if fixed_last is not None else ops
    for choice in itertools.product(("absent", "done", "torn"), repeat=len(free)):
        pool = copy_pool(base_pool)
        for (kind, disk, slot, value), how in zip(free, choice):
            if how == "done":
                pool[disk][slot] = value
            elif how == "torn":
                pool[disk][slot] = ("torn",)
        if fixed_last is not None:
            kind, disk, slot, value = ops[-1]
            if fixed_last == "done":
                pool[disk][slot] = value
            elif fixed_last == "torn":
                pool[disk][slot] = ("torn",)
        yield pool


def crash_states_of_success_path(segments, init_pool):
    resolved, _ = resolve(segments, init_pool)
    base = copy_pool(init_pool)
    states = set()
    for resolved_segment in resolved:
        for pool in segment_outcomes(base, resolved_segment):
            states.add(canonical(pool))
        base = apply_all_done(base, resolved_segment)
    states.add(canonical(base))
    return states


def rollback_segments(policy, written_devices, failing_segment_index):
    if not written_devices:
        return []
    if policy == "extra_pass_none" and failing_segment_index >= 1:
        return []
    if policy == "none":
        return []
    if policy == "per_written":
        return [[("sys", d, PREVIOUS_INSTANCE) for d in written_devices]]
    unique = []
    for d in written_devices:
        if d not in unique:
            unique.append(d)
    if policy in ("per_device", "extra_pass_none"):
        return [[("sys", d, PREVIOUS_INSTANCE) for d in unique]]
    if policy == "per_device_barriered":
        return [[("sys", d, PREVIOUS_INSTANCE)] for d in unique]
    raise ValueError(policy)


def failure_states(variant, policy, disk_count, outcomes=("absent", "torn", "done"), only_position=None):
    """取号失败（第 k 个系统配置写报错）之后，盘上可能落成的全部状态（崩溃在失败之后任一点，或没崩）。"""
    init = start_pool(disk_count)
    segments = acquisition_segments(variant, disk_count)
    states = set()
    scenarios = 0
    for seg_index, segment in enumerate(segments):
        for op_index in range(len(segment)):
            if only_position is not None and only_position != (seg_index, op_index):
                continue
            written = [op[1] for earlier in segments[:seg_index] for op in earlier]
            written += [op[1] for op in segment[:op_index]]
            for how in outcomes:
                scenarios += 1
                truncated = segments[:seg_index] + [segment[:op_index + 1]]
                resolved, live_after_failure = resolve(truncated, init, fail=(seg_index, op_index, how))
                base = copy_pool(init)
                for resolved_segment in resolved[:-1]:
                    base = apply_all_done(base, resolved_segment)
                for pool in segment_outcomes(base, resolved[-1], fixed_last=how):
                    states.add(canonical(pool))
                base = apply_all_done(base, resolved[-1][:-1])
                if how == "done":
                    base = apply_all_done(base, resolved[-1][-1:])
                elif how == "torn":
                    kind, disk, slot, value = resolved[-1][-1]
                    base[disk][slot] = ("torn",)
                live = live_after_failure
                for rb in rollback_segments(policy, written, seg_index):
                    rb_resolved, live = resolve([rb], base_to_pool(live))
                    for pool in segment_outcomes(base, rb_resolved[0]):
                        states.add(canonical(pool))
                    base = apply_all_done(base, rb_resolved[0])
                states.add(canonical(base))
    return states, scenarios


def base_to_pool(pool):
    return pool


# ---------------------------------------------------------------- 部分一：取号失败之后的落盘窗口

POLICIES_OF = {
    "one_round": ("per_written", "none"),
    "two_rounds": ("per_written", "per_device", "per_device_barriered", "none", "extra_pass_none"),
    "extra_pass_on_one_disk": ("per_device", "extra_pass_none"),
}


def tally(states, disk_count):
    zero = Counter()
    pairs = Counter()
    choices = fault_choices_one_slot_per_disk(disk_count)
    for state in states:
        pool = [list(disk) for disk in state]
        zero[classify(pool)] += 1
        for choice in choices:
            pairs[classify(apply_one_slot_per_disk(pool, choice))] += 1
    return zero, pairs


def fmt(counter):
    return "{" + ", ".join(f"{k}:{counter[k]}" for k in ("挂得上", "R2拒", "盘全废拒")) + "}"


def part_rollback():
    print("# 部分一：取号失败之后的落盘窗口（失败 = 第 k 个系统配置写报错，k 走遍取号的写，报错的写留下 没落 / 撕裂 / 落了 三种样子）")
    print("# new_states = 失败处理才到得了、取号成功那条路的崩溃状态里没有的盘面；zero = 不加故障的下一次挂载；pairs = 再加每盘至多一槽故障（翻坏 / 持续读错）")
    for disk_count in (2, 3):
        init = start_pool(disk_count)
        for variant, policies in POLICIES_OF.items():
            success = crash_states_of_success_path(acquisition_segments(variant, disk_count), init)
            for policy in policies:
                states, scenarios = failure_states(variant, policy, disk_count)
                new = states - success
                zero, pairs = tally(new, disk_count)
                union_zero, union_pairs = tally(states | success, disk_count)
                print(f"probe=rollback disks={disk_count} acquisition={variant} policy={policy} scenarios={scenarios} "
                      f"failure_states={len(states)} new_states={len(new)} zero={fmt(zero)} pairs={fmt(pairs)} "
                      f"pairs_refused={pairs['R2拒'] + pairs['盘全废拒']} "
                      f"union_states={len(states | success)} union_zero_refused={union_zero['R2拒'] + union_zero['盘全废拒']} "
                      f"union_pairs_refused={union_pairs['R2拒'] + union_pairs['盘全废拒']}")
    print("# 对照第二轮攻方：取号写两轮、第 4 个系统配置写报错且那个写没落（回卷前不加故障）")
    for policy in ("per_written", "per_device"):
        states, _ = failure_states("two_rounds", policy, 2, outcomes=("absent",), only_position=(1, 1))
        zero, _ = tally(states, 2)
        print(f"probe=rollback_vs_r2_opus disks=2 acquisition=two_rounds policy={policy} fail_position=write_index_3 failure_states={len(states)} zero={fmt(zero)}")


# ---------------------------------------------------------------- 部分二：取号写的写法 × 盘数 × 故障模型下的撞号与拒挂载

def publish_segments(disk_count):
    disks = range(disk_count)
    return [
        [("unit", d, NEW_INSTANCE) for d in disks],
        [("rec", d, NEW_INSTANCE) for d in disks],
        [("root", 0, NEW_INSTANCE)],
        [("sys", d, NEW_INSTANCE) for d in disks],
    ]


def whole_path_states(variant, disk_count):
    """取号成功之后接着第一次发布（单元写 → 记录写 → 根槽 FUA → 发布末尾轮换），每段的全部落盘组合。
    状态 = (两槽样子, 已落的本实例单元所在盘, 已落的本实例记录所在盘, 根是否已落)。"""
    init = start_pool(disk_count)
    segments = acquisition_segments(variant, disk_count) + publish_segments(disk_count)
    resolved, _ = resolve(segments, init)
    base = (copy_pool(init), frozenset(), frozenset(), False)
    states = set()
    for resolved_segment in resolved:
        options = [("absent", "done", "torn") if op[0] == "sys" else ("absent", "done") for op in resolved_segment]
        for choice in itertools.product(*options):
            pool = copy_pool(base[0])
            units, recs, root = set(base[1]), set(base[2]), base[3]
            for (kind, disk, slot, value), how in zip(resolved_segment, choice):
                if how == "absent":
                    continue
                if kind == "sys":
                    pool[disk][slot] = value if how == "done" else ("torn",)
                elif kind == "unit":
                    units.add(disk)
                elif kind == "rec":
                    recs.add(disk)
                elif kind == "root":
                    root = True
            states.add((canonical(pool), frozenset(units), frozenset(recs), root))
        pool = copy_pool(base[0])
        units, recs, root = set(base[1]), set(base[2]), base[3]
        for kind, disk, slot, value in resolved_segment:
            if kind == "sys":
                pool[disk][slot] = value
            elif kind == "unit":
                units.add(disk)
            elif kind == "rec":
                recs.add(disk)
            elif kind == "root":
                root = True
        base = (pool, frozenset(units), frozenset(recs), root)
    states.add((canonical(base[0]), base[1], base[2], base[3]))
    return states


def collision_tally(states, faulted_pools):
    counts = Counter()
    for pool_key, units, recs, root in states:
        has_artifacts = bool(units or recs or root)
        for pool in faulted_pools(pool_key):
            kind = classify(pool)
            if kind != "挂得上":
                counts[kind] += 1
                continue
            highest = max([slot[2] for disk in pool for slot in disk if slot[0] == "ok"] + [PREVIOUS_INSTANCE])
            if root:
                highest = max(highest, NEW_INSTANCE)
            if highest + 1 <= NEW_INSTANCE and has_artifacts:
                counts["撞号"] += 1
            else:
                counts["干净"] += 1
    return counts


def part_coverage():
    print("# 部分二：取号写的写法 × 盘数 × 故障模型（崩溃之后每盘至多一槽 / 全池任意至多 2 槽 / 全池任意至多 3 槽，每槽 翻坏 或 持续读错）")
    print("# 状态 = 取号成功之后接着第一次发布的各段落盘组合；撞号 = 下一次可写挂载取到的号不大于崩掉那次的号而盘上已有那个号的单元 / 记录 / 根")
    for disk_count in (2, 3):
        models = {
            "每盘至多一槽": lambda key, n=disk_count: (apply_one_slot_per_disk([list(d) for d in key], c) for c in fault_choices_one_slot_per_disk(n)),
            "全池任意至多2槽": lambda key, n=disk_count: (apply_any_k([list(d) for d in key], f) for f in fault_sets_any_k_slots(n, 2)),
            "全池任意至多3槽": lambda key, n=disk_count: (apply_any_k([list(d) for d in key], f) for f in fault_sets_any_k_slots(n, 3)),
        }
        for variant in ("one_round", "two_rounds", "extra_pass_on_one_disk"):
            states = whole_path_states(variant, disk_count)
            acquisition_only = crash_states_of_success_path(acquisition_segments(variant, disk_count), start_pool(disk_count))
            for name, make in models.items():
                counts = collision_tally(states, make)
                print(f"probe=coverage disks={disk_count} acquisition={variant} faults={name} states={len(states)} acquisition_phase_states={len(acquisition_only)} "
                      f"撞号={counts['撞号']} R2拒={counts['R2拒']} 盘全废拒={counts['盘全废拒']} 干净={counts['干净']}")


# ---------------------------------------------------------------- 部分三：离线出路 α' 「查什么才写」四种规则

def alpha_rules():
    """每条规则收 (断言的版本, 真实最新版本, 各版本的根 / 记录读不读得出) 交回写不写。
    readable[(版本, 'root'|'record')] 为真 = 读得出。规则：
      none        不查（第二轮攻方的「不查的 α」）
      root_only   断言那一版的根读得出
      root_or_record  断言那一版的根或记录读得出（用户 2026-10-19 选定的 α' 字面：根或记录读得出、且是它）
      plus_nothing_newer  root_or_record 再加：没有读得出的、比断言那一版新的根或记录（本腿提的，零轮）"""
    def evidence_of(readable, version):
        return readable.get((version, "root"), False) or readable.get((version, "record"), False)

    def newer_evidence(readable, version):
        return any(ok and v > version for (v, _), ok in readable.items())

    return {
        "none": lambda asserted, readable: True,
        "root_only": lambda asserted, readable: readable.get((asserted, "root"), False),
        "root_or_record": lambda asserted, readable: evidence_of(readable, asserted),
        "plus_nothing_newer": lambda asserted, readable: evidence_of(readable, asserted) and not newer_evidence(readable, asserted),
    }


def part_alpha():
    print("# 部分三：α' 的规则 × 断言 × 版本证据真值表。真实最新版本 L=5；断言 A ∈ {4, 5, 6}；L 与 L−1 的根 / 记录各读得出或读不出，6 不存在（都读不出）")
    print("# 结局：写对 = 断言就是真实最新且写了；拒真 = 断言就是真实最新而没写（与今天的只读相同，不更坏）；")
    print("#       写低有证据 = 断言比真实低、写了、而盘上读得出更新的根或记录（本可以看出断言低了）；写低无证据 = 断言比真实低、写了、盘上读不出更新的证据（任何规则都看不出）；")
    print("#       写空 = 断言比真实新（不存在）而写了；拒错 = 断言不对而没写（好）")
    latest = 5
    rules = alpha_rules()
    tally_by_rule = {name: Counter() for name in rules}
    for l_root, l_record, p_root, p_record in itertools.product((True, False), repeat=4):
        readable = {(5, "root"): l_root, (5, "record"): l_record, (4, "root"): p_root, (4, "record"): p_record,
                    (6, "root"): False, (6, "record"): False}
        for asserted in (4, 5, 6):
            for name, rule in rules.items():
                writes = rule(asserted, readable)
                if asserted == latest:
                    label = "写对" if writes else "拒真"
                elif asserted < latest:
                    if not writes:
                        label = "拒错"
                    else:
                        label = "写低有证据" if (l_root or l_record) else "写低无证据"
                else:
                    label = "写空" if writes else "拒错"
                tally_by_rule[name][label] += 1
    for name, counts in tally_by_rule.items():
        print(f"probe=alpha rule={name} 写对={counts['写对']} 拒真={counts['拒真']} 写低有证据={counts['写低有证据']} 写低无证据={counts['写低无证据']} 写空={counts['写空']} 拒错={counts['拒错']}")


# ---------------------------------------------------------------- 部分四：旧快照盘被放行的格

def snapshot_states(unit_count, root_on_this_disk):
    """一次发布的各个取下时刻上，这块盘（快照盘）已经有的东西：单元（子集）、末条记录、根（只在根住这块盘时有）、轮换（tail 更新）。
    次序由屏障定：单元段 → 屏障 → 记录段 → 屏障 → 根槽 FUA → 轮换段。"""
    states = set()
    for subset in itertools.product((False, True), repeat=unit_count):
        states.add((subset, False, False, False))
    all_units = (True,) * unit_count
    for record in (False, True):
        states.add((all_units, record, False, False))
    states.add((all_units, True, False, False))
    if root_on_this_disk:
        states.add((all_units, True, True, False))
        states.add((all_units, True, True, True))
    else:
        states.add((all_units, True, False, True))
        states.add((all_units, True, False, False))
    return states


def refuses(rule, state, unit_count, root_on_this_disk):
    units, record, root, rotated = state
    missing = unit_count - sum(units)
    behind_by_tail = not rotated
    root_missing_here = root_on_this_disk and not root
    if rule == "today":
        return behind_by_tail and (missing >= 1 or root_missing_here)
    if rule == "record_for_units":
        units_half = (not record) and missing >= 1
        root_half = behind_by_tail and root_missing_here
        return units_half or root_half
    if rule == "missing_fewer_than_two_alone":
        threshold = min(2, unit_count)
        units_half = behind_by_tail and missing >= threshold
        root_half = behind_by_tail and root_missing_here
        return units_half or root_half
    raise ValueError(rule)


def part_stale():
    print("# 部分四：旧快照盘（取在一次发布途中）被可写挂载的落后支放行的格。k = 那一版的单元数；根住在快照盘 / 住在另一块盘")
    print("# 状态 = 取下那一刻快照盘上已有的东西（单元子集、末条记录、根、轮换）；放行 = 判法不拒；lack_* = 放行的状态里缺单元 / 缺末条记录 / 缺根槽的有几个")
    for unit_count in (1, 2, 3):
        for root_on_this_disk in (False, True):
            states = snapshot_states(unit_count, root_on_this_disk)
            rows = {}
            for rule in ("today", "record_for_units", "missing_fewer_than_two_alone"):
                admitted = [s for s in states if not refuses(rule, s, unit_count, root_on_this_disk)]
                rows[rule] = admitted
                lack_units = sum(1 for s in admitted if unit_count - sum(s[0]) >= 1)
                lack_record = sum(1 for s in admitted if not s[1])
                lack_root = sum(1 for s in admitted if root_on_this_disk and not s[2])
                print(f"probe=stale k={unit_count} root_on_snapshot_disk={root_on_this_disk} rule={rule} states={len(states)} admitted={len(admitted)} "
                      f"lack_units={lack_units} lack_record={lack_record} lack_root={lack_root}")
            added = {rule: len(set(rows[rule]) - set(rows['today'])) for rule in rows if rule != "today"}
            print(f"probe=stale_added_over_today k={unit_count} root_on_snapshot_disk={root_on_this_disk} record_for_units={added['record_for_units']} missing_fewer_than_two_alone={added['missing_fewer_than_two_alone']}")


if __name__ == "__main__":
    part = sys.argv[1] if len(sys.argv) > 1 else "all"
    if part in ("rollback", "all"):
        part_rollback()
    if part in ("coverage", "all"):
        part_coverage()
    if part in ("alpha", "all"):
        part_alpha()
    if part in ("stale", "all"):
        part_stale()
