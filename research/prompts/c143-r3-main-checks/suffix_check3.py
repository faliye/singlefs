"""Main-agent check, third part: the four scripts suffix_check2.py left unrescued (whole-disk
faults, 7-11 faults). Here non-creating publishes may be inserted before every creating step
("u") and right after every mount, d = 0..4 at each insertion point. Same prefix ops, same
faults. Imports the leg's model read-only (-B)."""
import itertools
import sys

sys.path.insert(0, "/home/fy5090/code/singlefs/research/prompts/c143-r3-opus-model")
import c143r3_model as m  # noqa: E402

b = m.b
NO_FAULT = b.NO_FAULT
UNRESCUED = [
    ("f5", "CJ2", [("p", 1), ("f", 1), ("rb", 3, NO_FAULT, 0), ("f", 1), ("u", 1), ("crash",),
                   ("mount", (1, 4, 3, (), ())), ("u", 1)]),
    ("f5", "CJ2", [("p", 1), ("f", 1), ("rb", 3, NO_FAULT, 0), ("f", 1), ("u", 1), ("crash",),
                   ("mount", (1, 4, 4, (), ())), ("u", 1)]),
    ("f12d", "CJ2", [("p", 1), ("rb", 2, NO_FAULT, 0), ("u", 1), ("crash",),
                     ("mount", (0, 1, 3, (), ())), ("rb", 1, (1, 2, 3, (), ()), 0), ("u", 1)]),
    ("f12d", "CJ2", [("p", 1), ("f", 1), ("rb", 2, NO_FAULT, 0), ("u", 1), ("crash",),
                     ("mount", (0, 1, 0, (), ())), ("rb", 1, (0, 2, 3, (), ()), 0), ("u", 1)]),
]


def hit(arm, script):
    run = m.run3(arm, script)
    if run is None:
        return None, None
    return bool(b.violations(run, b.CRIT[arm])), run.fs


def insertion_points(script):
    """indices before which padding may go: before each 'u', and right after each 'mount'."""
    points = set()
    for index, op in enumerate(script):
        if op[0] == "u":
            points.add(index)
        if op[0] == "mount":
            points.add(index + 1)
    return sorted(points)


def padded(script, delays_by_point):
    out = []
    for index, op in enumerate(script):
        delay = delays_by_point.get(index, 0)
        if delay:
            out += [("u", 0)] + [("p", 0)] * delay
        out.append(op)
    return out


def main():
    for family, arm_x, script in UNRESCUED:
        x_hit, x_fs = hit(arm_x, script)
        c_hit, c_fs = hit("C", script)
        points = insertion_points(script)
        best = None
        tried = 0
        for delays in itertools.product(range(5), repeat=len(points)):
            if not any(delays):
                continue
            tried += 1
            variant = padded(script, dict(zip(points, delays)))
            c_hit_v, c_fs_v = hit("C", variant)
            if c_hit_v and (best is None or c_fs_v < best[0]):
                best = (c_fs_v, delays, b.fmt(variant))
        print(f"family={family} x={arm_x} x_hit={x_hit} x_fs={x_fs} c_hit_unpadded={c_hit} "
              f"points={points} variants_tried={tried} "
              f"c_best={'none' if best is None else (best[0], best[1])}")
        if best:
            print(f"  c_variant=[{best[2]}]")
    print("done")


if __name__ == "__main__":
    main()
