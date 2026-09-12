#!/usr/bin/env python3
# C126 r1 Opus reverse-inference probe.  Two independent counting models:
#   A) switch -> rebuild -> replay -> redo, allocator view vs pointer view vs published-predicate
#   B) instance-table chain arithmetic: ceil((r+k+1)/509) vs ceil(r/508)
# Mutation switches (self-proof that the assertions can go red):
#   C126_MUT=replay_registers_alloc   replay registers named extents into the allocator overlay
#   C126_MUT=w_zero                   W forced to 0 (probe must then see 0 contradictions)
#   C126_MUT=chain_one_link           one chain-pointer record for the whole chain (jia's reading)
import math, os, sys

MUT = os.environ.get("C126_MUT", "")
SLOT = 16384                      # D18 item9 / D3 item7: 16 KiB landing slot
PACK_UNIT = 32768                 # D18 item11: code-3 packed record unit
PACK_HDR = 135                    # D18 item11 / item14: code-3 header incl nonce/MAC reserve
ROW = 64                          # format-const INSTANCE_ROW_BYTES
NSW = 3                           # D23 item14: N_switch
REPLICAS = 2                      # D22 item7 / D19 item8: position entries 14 x 2
out = []
def say(s): out.append(s); print(s)

# ---------------- model A: one concrete switch sequence -------------------
# published root R100 (instance 7, txg 100) owns slots 0..999 on dev0.
PUB_ALLOC = set(range(0, 1000))       # alloc-record tree entries under R100
PUB_PTR   = set(range(0, 1000))       # blocks the R100 tree references
BUMP_AT_R100 = 1000                   # D3 item5 cluster-segment cursor as of R100

# in-flight checkpoint C, txg 101, instance 7
# txn 41: data unit 1000..1001 (32 KiB = 2 slots) + COW ancestors 1004..1007
# txn 42: data unit 1002..1003                    + COW ancestors 1008..1011
# txn 43: data unit 1012..1013, unit write fails -> probe write ok -> instance switch
TXN = {41: {"data": [1000, 1001], "meta": [1004, 1005, 1006, 1007], "committed": True},
       42: {"data": [1002, 1003], "meta": [1008, 1009, 1010, 1011], "committed": True},
       43: {"data": [1012, 1013], "meta": [],                        "committed": False}}
BIRTH = 101
T_PUB = 100                            # row written = (7, selected root txg, W)

W = 0 if MUT == "w_zero" else max(t for t, v in TXN.items() if v["committed"])
say(f"W={W}  T_pub={T_PUB}  birth of in-flight units={BIRTH}")

# rebuild: alloc-record tree comes from the selected root; C284 decidable form says
# "the recovery path does not call the allocator", so the overlay is empty.
alloc_tree = set(PUB_ALLOC)
overlay = set()
ptr = set(PUB_PTR)
for t, v in sorted(TXN.items()):
    if v["committed"] and t <= W:                      # replay applies these records
        ptr |= set(v["data"]) | set(v["meta"])
        if MUT == "replay_registers_alloc":
            overlay |= set(v["data"]) | set(v["meta"])

def published_predicate(slot):
    """D18 item11 global published predicate, row (7, T_PUB, W)."""
    for t, v in TXN.items():
        if slot in v["data"]:                          # code 1: b <= T_pub  or  n <= W
            return (BIRTH <= T_PUB) or (t <= W)
        if slot in v["meta"]:                          # code 2/3: b <= T_pub only
            return BIRTH <= T_PUB
    return True

allocator_free = {s for s in range(1000, 1014) if s not in alloc_tree and s not in overlay}
referenced     = {s for s in ptr if s >= 1000}
must_keep      = {s for s in referenced if published_predicate(s)}
must_rewrite   = {s for s in referenced if not published_predicate(s)}

contradiction = sorted(must_keep & allocator_free)
say(f"rebuilt pointer layer references (>=1000): {sorted(referenced)}")
say(f"allocator sees free               (>=1000): {sorted(allocator_free)}")
say(f"published-predicate says KEEP            : {sorted(must_keep)}")
say(f"published-predicate says REWRITE         : {sorted(must_rewrite)}")
say(f"CONTRADICTION (keep AND free)            : {contradiction}  count={len(contradiction)}")

# the redo allocates from the bump cursor reloaded with the root
cursor = BUMP_AT_R100
redo_needs = 2 + len(must_rewrite)     # txn 43 redone (2 slots) + rewrite every unpublished metadata slot
picked, c = [], cursor
while len(picked) < redo_needs:
    if c not in alloc_tree and c not in overlay:
        picked.append(c)
    c += 1
overwrote = sorted(set(picked) & must_keep)
say(f"redo needs {redo_needs} slots, allocator hands out {picked}")
say(f"redo OVERWRITES still-referenced published slots: {overwrote}  count={len(overwrote)}")
extra = sorted(set(picked) - set(range(1000, 1014)))
say(f"redo slots taken beyond the failed checkpoint's own footprint: {extra}  count={len(extra)}")

if MUT == "" or os.environ.get("C126_REDPROOF"):
    assert len(contradiction) == 4 and contradiction == [1000, 1001, 1002, 1003], contradiction
    assert len(must_rewrite) == 8, must_rewrite
    assert len(overwrote) == 4, overwrote
    assert len(extra) == 0, extra
    say("MODEL A OK: 4 contradiction slots, 8 forced metadata rewrites, 4 overwritten, 0 fresh slots")
elif MUT == "replay_registers_alloc":
    assert len(contradiction) == 0, contradiction
    assert len(extra) == len(must_rewrite) == 8, extra
    say(f"MODEL A OK under the other reading: 0 contradictions, but {len(extra)} FRESH slots needed "
        "(exactly the metadata the published predicate orders the recovery instance to rewrite)")
elif MUT == "w_zero":
    assert len(contradiction) == 0, contradiction
    say("MODEL A OK: W=0 leaves nothing that must be kept -> 0 contradictions (probe is watching W)")

# ---------------- model B: instance-table chain arithmetic ----------------
say("")
PER_PAGE = (PACK_UNIT - PACK_HDR) // ROW            # 509, chain-pointer record included
say(f"records per page = ({PACK_UNIT} - {PACK_HDR}) // {ROW} = {PER_PAGE}")
assert PER_PAGE == 509

def pages_jia(r):    return math.ceil((r + 1) / PER_PAGE)               # material's formula
def pages_true(r):
    if MUT == "chain_one_link":          # one chain pointer for the WHOLE chain (jia's reading)
        return max(1, math.ceil((r + 1) / PER_PAGE))
    return max(1, math.ceil(r / (PER_PAGE - 1)))   # clause: one chain ptr per PAGE

diff = [r for r in range(0, 5000) if pages_jia(r) != pages_true(r)]
first = diff[0] if diff else -1
say(f"first row count where the two disagree: r={first}" + ("" if first < 0 else
    f" (jia={pages_jia(first)} pages, literal={pages_true(first)} pages)"))

def reserve_blocks(rows0, pages):
    return sum(REPLICAS * PACK_UNIT * pages(rows0 + k) for k in range(1, NSW + 1)) // SLOT

for rows0 in (0, 508, 1016, 4064):
    say(f"rows0={rows0:5d}  jia={reserve_blocks(rows0, pages_jia):4d} blocks   "
        f"literal={reserve_blocks(rows0, pages_true):4d} blocks")

if MUT == "" or os.environ.get("C126_REDPROOF"):
    assert first == 1017, first
    assert reserve_blocks(0, pages_jia) == 12 and reserve_blocks(0, pages_true) == 12
    assert reserve_blocks(1016, pages_jia) == 32 and reserve_blocks(1016, pages_true) == 36
    say("MODEL B OK: rows0=0 cannot tell the two formulas apart (both 12 blocks); "
        "they first split at rows0+k=1017, and at rows0=1016 jia under-counts by 4 blocks")
elif MUT == "chain_one_link":
    assert first == -1, first
    say("MODEL B OK: forcing one chain pointer for the whole chain makes the gap vanish "
        "-> the gap is exactly the 'one chain-pointer record per page' clause")

# C304 recompute: the kind=1 record with an 83-wide pointer
w304 = 1 + 1 + 83 + 3
per304 = (PACK_UNIT - PACK_HDR) // w304
say(f"C304 recompute: kind=1 record = 1+1+83+3 = {w304} bytes > {ROW}; "
    f"if the row widens to {w304}, records/page = {per304}, rows/page = {per304 - 1}")
if MUT == "" or os.environ.get("C126_REDPROOF"):
    assert w304 == 88 and per304 == 370
    r0 = sum(REPLICAS * PACK_UNIT * max(1, math.ceil((0 + k) / (per304 - 1))) for k in range(1, NSW + 1)) // SLOT
    say(f"  and at rows0=0 the widened row still gives {r0} blocks -> third constant the sample point cannot see")
    assert r0 == 12

# ---------------- model C: journal ring occupancy during a switch ---------
say("")
CKPT_KIB = 7388            # E91 settled coding (56 B/item, 71 items/record): T_dirty full window
RING_F2  = 14776           # E91: I-8.1 lower bound at F=2
RING_F3  = 22164           # E91: at F=3
live_one_switch = 3 * CKPT_KIB      # publishing C + open C' + the redo's re-emission of C
live_all_switches = (2 + NSW) * CKPT_KIB
say(f"one checkpoint full window = {CKPT_KIB} KiB (1847 records, 131072 named items)")
say(f"in ring during ONE switch  = {live_one_switch} KiB = {live_one_switch/RING_F2:.2f}x ring(F=2), "
    f"{live_one_switch/RING_F3:.2f}x ring(F=3)")
say(f"in ring after N_switch={NSW}    = {live_all_switches} KiB = {live_all_switches/RING_F2:.2f}x ring(F=2), "
    f"{live_all_switches/RING_F3:.2f}x ring(F=3)")
if MUT == "" or os.environ.get("C126_REDPROOF"):
    assert live_one_switch == 22164 and live_all_switches == 36940
    assert abs(live_all_switches / RING_F2 - 2.50) < 1e-9

# ---------------- model D: reserve pool and the per-device dimension ------
say("")
CKPT_COST = 64             # E19: reserve >= ckpt_cost, 64 blocks drove ckpt_stall 199 -> 0
say(f"reserve pool sized for ONE checkpoint = {CKPT_COST} blocks; "
    f"{NSW} consecutive redos with no publish in between need {NSW * CKPT_COST} blocks")
free_dev0, free_dev1, need_total = 200, 0, 12
per_dev_need = need_total // 2
say(f"pool-scalar admission: free={free_dev0 + free_dev1} >= reserve={need_total} -> PASS")
say(f"per-device truth     : dev0 free={free_dev0}, dev1 free={free_dev1}, "
    f"each mirror leg needs {per_dev_need} -> dev1 SHORT by {per_dev_need - free_dev1}")
if MUT == "" or os.environ.get("C126_REDPROOF"):
    assert free_dev0 + free_dev1 >= need_total and free_dev1 < per_dev_need
    assert NSW * CKPT_COST == 192
    say("MODEL C/D OK")
