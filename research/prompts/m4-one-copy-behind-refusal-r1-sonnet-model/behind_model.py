#!/usr/bin/env python3
"""两盘池「落后且缺单元」放行条件的状态模型。

不读 crates/ 的代码，也不跑实现：从 D18 已定项 11 与 D23 已定项 14 的条款字面与写序
（单元写每块盘 → 屏障 → 记录写每块盘 → 屏障 → 根槽 FUA → 系统配置轮换 → 屏障）重述判法，
逐态比今天的判法、候选「点名窗口」与候选「紧邻一版的点名」两种写法，与「这块盘到底有没有收过那个单元」的真值。

模型的范围（读结果时要带着）：
  - 两块盘、三个角色、至多四次发布、单实例（跨实例窗口不在模型里）；
  - 只判落后支「缺单元」那一半，根槽那一半不判；
  - 崩溃只发生在最后一次发布；之前的发布全部落盘；
  - 一份单元坏 = 那一块盘上它的字节验不过；没收过 = 那一块盘从没写进过它。
"""
import itertools
import sys

DEVICES = (0, 1)
ROLES = ("a", "b", "c")


class Publish:
    def __init__(self, index, written_roles, record_count, first_counter):
        self.index = index
        self.written_roles = tuple(written_roles)
        self.record_count = record_count
        self.first_counter = first_counter
        self.last_counter = first_counter + record_count - 1
        # 一次发布切几条记录：多条时第 k 条只点名第 k 个角色，最后一条点名剩下的全部（D23 已定项 17 的口径，按条款字面重述）
        self.named_by_counter = {}
        if record_count == 1:
            self.named_by_counter[first_counter] = tuple(written_roles)
        else:
            assert len(written_roles) >= record_count
            for offset in range(record_count - 1):
                self.named_by_counter[first_counter + offset] = (written_roles[offset],)
            self.named_by_counter[self.last_counter] = tuple(written_roles[record_count - 1:])


def build_history(publish_shapes):
    """publish_shapes：[(写哪几个角色, 切几条记录), ...]，第 1 次发布起。"""
    publishes = []
    counter = 1
    for index, (roles, record_count) in enumerate(publish_shapes, start=1):
        publish = Publish(index, roles, record_count, counter)
        publishes.append(publish)
        counter = publish.last_counter + 1
    return publishes


def live_units_of_version(publishes, version):
    """版本 version 的活单元：每个角色取 version 之前最后写它的那次发布（0 = mkfs）。"""
    live = {}
    for role in ROLES:
        owner = 0
        for publish in publishes:
            if publish.index <= version and role in publish.written_roles:
                owner = publish.index
        live[role] = (role, owner)
    return live


def last_counter_of_version(publishes, version):
    if version == 0:
        return 0
    return publishes[version - 1].last_counter


class Image:
    """崩溃之后两块盘的盘面：每块盘的单元（good / bad，没有这一项就是没收过）、记录计数器集合、系统配置 tail；
    池级的根环里持久的根（发布号集合）。"""

    def __init__(self):
        self.units = {d: {} for d in DEVICES}
        self.records = {d: set() for d in DEVICES}
        self.tail = {d: 0 for d in DEVICES}
        self.roots = {0}

    def copy(self):
        other = Image()
        other.units = {d: dict(self.units[d]) for d in DEVICES}
        other.records = {d: set(self.records[d]) for d in DEVICES}
        other.tail = dict(self.tail)
        other.roots = set(self.roots)
        return other


def complete_through(publishes, version):
    """前 version 次发布全部落盘（两块盘、单元、记录、根、tail），再加 mkfs 的三个单元。"""
    image = Image()
    for role in ROLES:
        for d in DEVICES:
            image.units[d][(role, 0)] = "good"
    for publish in publishes:
        if publish.index > version:
            break
        for role in publish.written_roles:
            for d in DEVICES:
                image.units[d][(role, publish.index)] = "good"
        for counter in publish.named_by_counter:
            for d in DEVICES:
                image.records[d].add(counter)
        image.roots.add(publish.index)
        for d in DEVICES:
            image.tail[d] = publish.last_counter
    return image


def crash_images_of_the_last_publish(publishes, last):
    """最后一次发布（last）的崩溃态，按写序的三段枚举，返回 (描述, 盘面)。
    单元段：屏障之前，每块盘每个单元写独立地落或没落，记录与根都没有；
    记录段：单元全落（屏障已过），每块盘每条记录写独立地落或没落，根没落；
    轮换段：单元、记录、根全落，每块盘的轮换写独立地落或没落。"""
    publish = publishes[last - 1]
    base = complete_through(publishes, last - 1)
    unit_writes = [(role, d) for role in publish.written_roles for d in DEVICES]
    for mask in itertools.product((False, True), repeat=len(unit_writes)):
        image = base.copy()
        for landed, (role, d) in zip(mask, unit_writes):
            if landed:
                image.units[d][(role, last)] = "good"
        yield ("单元段", mask), image
    record_writes = [(counter, d) for counter in publish.named_by_counter for d in DEVICES]
    for mask in itertools.product((False, True), repeat=len(record_writes)):
        image = base.copy()
        for role in publish.written_roles:
            for d in DEVICES:
                image.units[d][(role, last)] = "good"
        for landed, (counter, d) in zip(mask, record_writes):
            if landed:
                image.records[d].add(counter)
        yield ("记录段", mask), image
    for mask in itertools.product((False, True), repeat=len(DEVICES)):
        image = complete_through(publishes, last)
        for landed, d in zip(mask, DEVICES):
            if not landed:
                image.tail[d] = last_counter_of_version(publishes, last - 1)
        yield ("轮换段", mask), image


def selected_version(publishes, image, last):
    """恢复落到哪一版：根在环里的最大发布；或最后一次发布的全部记录在某块盘上、点名单元两块盘上两份都验得过（前缀第四条）。"""
    if last in image.roots:
        return last
    publish = publishes[last - 1]
    records_everywhere = set().union(*image.records.values())
    if not set(publish.named_by_counter) <= records_everywhere:
        return last - 1
    for role in publish.written_roles:
        for d in DEVICES:
            if image.units[d].get((role, last)) != "good":
                return last - 1
    return last


def damage_options(image):
    yield None
    for d in DEVICES:
        for unit in sorted(image.units[d]):
            yield (d, unit)


def counter_owner(publishes):
    owner = {}
    for publish in publishes:
        for counter, roles in publish.named_by_counter.items():
            owner[counter] = (publish.index, roles)
    return owner


ARMS = ("today", "window", "adjacent_literal", "adjacent_publish", "always_pass", "always_refuse")


def refuses_on_device(arm, publishes, image, version, d):
    """这块盘 d 落后支缺单元那一半拒不拒（只判单元那一半）。"""
    if arm == "always_pass":
        return False
    if arm == "always_refuse":
        return True
    position = last_counter_of_version(publishes, version)
    tail = image.tail[d]
    if tail >= position:
        return False
    if version > 0 and position in image.records[d]:
        return False
    live = live_units_of_version(publishes, version).values()
    missing_all = {unit for unit in live if image.units[d].get(unit) != "good"}
    if arm == "today":
        return bool(missing_all)
    window = range(tail + 1, position + 1)
    owner = counter_owner(publishes)
    window_readable = all(any(c in image.records[x] for x in DEVICES) for c in window)
    use_window = {
        "window": window_readable,
        "adjacent_literal": window_readable and tail + 1 == position,
        "adjacent_publish": window_readable and version >= 1
        and tail == last_counter_of_version(publishes, version - 1),
    }[arm]
    if not use_window:
        return bool(missing_all)
    named = set()
    for counter in window:
        publish_index, roles = owner[counter]
        named |= {(role, publish_index) for role in roles}
    return bool(missing_all & named)


def pool_truth_refuses(publishes, image, version):
    live = live_units_of_version(publishes, version).values()
    return any(image.units[d].get(unit) is None for d in DEVICES for unit in live)


def buildable(publishes, image, version):
    live = live_units_of_version(publishes, version).values()
    return all(any(image.units[d].get(unit) == "good" for d in DEVICES) for unit in live)


SHAPE_OPTIONS = (
    (("a",), 1), (("b",), 1), (("c",), 1),
    (("a", "b"), 2), (("b", "c"), 1), (("a", "b", "c"), 2),
)


def histories(publish_count):
    for shapes in itertools.product(SHAPE_OPTIONS, repeat=publish_count):
        yield build_history(list(shapes))


def states_of_family(family, publishes, last):
    """(描述, 盘面)：崩溃态按家族加工。"""
    for what, crashed in crash_images_of_the_last_publish(publishes, last):
        if family == "F5_crash_and_two_damaged_copies":
            copies = [(d, unit) for d in DEVICES for unit in sorted(crashed.units[d])]
            for first, second in itertools.combinations(copies, 2):
                image = crashed.copy()
                for d, unit in (first, second):
                    image.units[d][unit] = "bad"
                yield (what, "两处损坏", (first, second)), image
            continue
        if family == "F1_crash_and_one_damaged_copy":
            for damage in damage_options(crashed):
                image = crashed.copy()
                if damage is not None:
                    d, unit = damage
                    image.units[d][unit] = "bad"
                yield (what, "损坏", damage), image
            continue
        if family == "F3_stale_disk_tail_raised_without_units":
            # 盘 1 缺 (snapshot_after, last-1] 那几次发布的单元与记录，系统配置 tail 却写着它跟上了（离线出路只核记录与单元区出生 txg、
            # 不核本盘单元，写 tail 不过本盘单元那一关），最后一次发布两块盘照常写。
            for snapshot_after in range(0, last - 1):
                image = crashed.copy()
                for earlier in publishes[snapshot_after:last - 1]:
                    for role in earlier.written_roles:
                        image.units[1].pop((role, earlier.index), None)
                    for counter in earlier.named_by_counter:
                        image.records[1].discard(counter)
                yield (what, "盘1缺的发布", snapshot_after), image
            continue
        for snapshot_after in range(0, last - 1):
            for snapshot_kind in ("整点", "发布中途"):
                if snapshot_kind == "发布中途" and snapshot_after + 1 > last - 1:
                    continue
                snap = complete_through(publishes, snapshot_after)
                if snapshot_kind == "发布中途":
                    middle = publishes[snapshot_after]
                    for role in middle.written_roles:
                        snap.units[1][(role, middle.index)] = "good"
                    for counter in middle.named_by_counter:
                        snap.records[1].add(counter)
                image = crashed.copy()
                image.units[1] = dict(snap.units[1])
                image.records[1] = set(snap.records[1])
                image.tail[1] = last_counter_of_version(publishes, snapshot_after)
                yield (what, snapshot_kind, snapshot_after), image


FAMILIES = (
    "F1_crash_and_one_damaged_copy",
    "F2_stale_disk_swapped_in",
    "F3_stale_disk_tail_raised_without_units",
    "F5_crash_and_two_damaged_copies",
)
# 今天的判法在 F3 上本来就放错：tail 已不落后时单元一个都不读（D18 已定项 11 落后支的前一半），所以 F3 另报「候选比今天多放错的」。


def run(publish_counts=(2, 3), example_limit=2):
    tally = {}
    examples = {}
    for family in FAMILIES:
        for arm in ARMS:
            tally[(family, arm)] = {
                "states": 0, "refused": 0, "over_refuse": 0, "false_allow": 0, "newly_false_allow": 0,
            }
            examples[(family, arm, "over_refuse")] = []
            examples[(family, arm, "false_allow")] = []
            examples[(family, arm, "newly_false_allow")] = []
    for count in publish_counts:
        for publishes in histories(count):
            last = count
            shape = [(p.written_roles, p.record_count) for p in publishes]
            for family in FAMILIES:
                for description, image in states_of_family(family, publishes, last):
                    version = selected_version(publishes, image, last)
                    if not buildable(publishes, image, version):
                        continue
                    truth = pool_truth_refuses(publishes, image, version)
                    today_refused = any(
                        refuses_on_device("today", publishes, image, version, d) for d in DEVICES
                    )
                    for arm in ARMS:
                        refused = any(
                            refuses_on_device(arm, publishes, image, version, d) for d in DEVICES
                        )
                        cell = tally[(family, arm)]
                        cell["states"] += 1
                        cell["refused"] += refused
                        kind = None
                        if refused and not truth:
                            kind = "over_refuse"
                        elif truth and not refused:
                            kind = "false_allow"
                        kinds = [kind] if kind else []
                        if truth and not refused and today_refused:
                            kinds.append("newly_false_allow")
                        for each in kinds:
                            cell[each] += 1
                            bucket = examples[(family, arm, each)]
                            if len(bucket) < example_limit:
                                bucket.append((shape, description, version, dict(image.tail)))
    return tally, examples


def report(tally, examples):
    lines = []
    for family in FAMILIES:
        for arm in ARMS:
            cell = tally[(family, arm)]
            lines.append(
                f"RESULT family={family} arm={arm} states={cell['states']} refused={cell['refused']} "
                f"over_refuse={cell['over_refuse']} false_allow={cell['false_allow']} "
                f"newly_false_allow={cell['newly_false_allow']}"
            )
    for (family, arm, kind), bucket in sorted(examples.items()):
        if arm in ("always_pass", "always_refuse"):
            continue
        for shape, description, version, tails in bucket:
            lines.append(
                f"EXAMPLE family={family} arm={arm} kind={kind} selected_version={version} "
                f"tails={tails} crash={description} shape={shape}"
            )
    return lines


def pins(tally):
    """模型自己的会红检查：阳性对照与已知读数。"""
    failures = []
    f1, f2, f3 = FAMILIES[:3]
    if tally[(f2, "always_pass")]["false_allow"] == 0:
        failures.append("对照臂 always_pass 在 F2 上没有放错：false_allow 的计数器分不出放错")
    if tally[(f1, "always_refuse")]["over_refuse"] == 0:
        failures.append("对照臂 always_refuse 在 F1 上没有拒错：over_refuse 的计数器分不出拒错")
    if tally[(f1, "today")]["over_refuse"] == 0:
        failures.append("今天的判法在 F1 上没拒错：与三个被拒状态对不上，模型没造出那一形")
    for family in (f1, f2, FAMILIES[3]):
        if tally[(family, "today")]["false_allow"] != 0:
            failures.append(f"今天的判法在 {family} 上放错了")
    if tally[(f3, "today")]["false_allow"] == 0:
        failures.append("F3 上今天的判法没放错：tail 不落后时不读单元这一条没造出来")
    return failures


def main():
    counts = tuple(int(x) for x in sys.argv[1:]) or (2, 3)
    tally, examples = run(counts)
    for line in report(tally, examples):
        print(line)
    failures = pins(tally)
    for failure in failures:
        print("PIN_FAILED " + failure)
    print("PINS " + ("failed" if failures else "ok"))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
