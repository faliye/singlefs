import re
import sys


def landing(line):
    body = line.split(": ", 1)[1]
    if body.startswith("err ") or body.startswith("failed "):
        return "拒"
    m = re.search(r"CheckpointTxg\((\d+)\)", body)
    if not m:
        return "?"
    return {"9": "R", "8": "R之前一版"}.get(m.group(1), "txg" + m.group(1))


def parse(path):
    out = []
    cur = None
    for raw in open(path):
        line = raw.rstrip("\n")
        m = re.search(r"W2PROBE change=(\w+)(?: target_kind=(\d+) edit=(\w+))? face=(\w+)", line)
        if m:
            cur = {
                "change": m.group(1),
                "kind": m.group(2) or "2(inode)",
                "edit": m.group(3) or ("UnknownFlagsBit" if "Unknown" in m.group(1) else m.group(1)),
                "face": m.group(4),
            }
            out.append(cur)
            continue
        m = re.match(r"W2PROBE   (recovery|read_only_mount|writable_mount): ", line)
        if m and cur is not None and m.group(1) not in cur:
            cur[m.group(1)] = landing(line[len("W2PROBE   "):])
    return out


FACE = {
    "SystemConfigurationWitnessedR": "见证了R",
    "CrashBetweenTheRootOfRAndItsRotation": "崩在轮换前",
    "RConfirmedButItsRotationSlotsReadBackZero": "R已确认槽读回零",
}
for path in sys.argv[1:]:
    print("##", path.split("/")[-1])
    for r in parse(path):
        if r["change"] == "HarmlessFieldWithChecksumsRecomputed":
            continue
        print(
            f'kind={r["kind"]:9} edit={r["edit"]:16} face={FACE[r["face"]]:10} '
            f'恢复={r["recovery"]:6} 只读={r["read_only_mount"]:6} 可写={r["writable_mount"]}'
        )
