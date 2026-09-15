#!/usr/bin/env python3
"""
C161 round 3, defense leg. Model E: is "yi_G's message dwell time has no
bound independent of total write count" an inductive extrapolation from 18
sample points, or a provable consequence of the stated flush policy ("一个
缓冲区只在攒满时下推")?

Closed form: a buffer with capacity B, fed at long-run rate r = (writes that
route to this specific buffer) / (total writes issued to the pool), fills and
drains roughly every B/r writes if -- and only if -- draining is triggered
purely by "buffer is full", with no time/count fallback. Expected dwell for
a message that just missed a drain is ~ B/r writes. r itself is a workload
parameter (skew, leaf count) with no floor other than 0 -- for any target
dwell D, there exists a workload (r = B/D) that produces it. Hence there is
no bound on dwell as a function of the CANDIDATE's mechanism alone; only the
WORKLOAD bounds it, and total write count is one axis attackers/users don't
control by design.
"""


def dwell_for_rate(capacity, rate_per_write):
    if rate_per_write <= 0:
        return float('inf')
    return capacity / rate_per_write


def main():
    capacity = 313  # B (round-2 opus report's "今天的宽" pivot 117 / msg 34 => F=49,B=313)
    print(f"capacity B = {capacity}")
    print("share of writes routed to this buffer (r) -> expected dwell (writes):")
    for leaves in (1024, 32768, 2**20, 2**26):
        # uniform share if this buffer is one leaf's worth of traffic among
        # `leaves` leaves under a uniform workload
        r = 1.0 / leaves
        d = dwell_for_rate(capacity, r)
        print(f"  leaves={leaves:>10d} r=1/{leaves:<10d} dwell~={d:,.0f} writes")
    print()
    print("same capacity, workload skew varied at fixed leaf count 32768 "
          "(cold share of traffic going to a 1-in-N cold slice):")
    for cold_frac in (0.5, 0.1, 0.01, 0.001, 0.0001):
        r = cold_frac / 32768
        d = dwell_for_rate(capacity, r)
        print(f"  cold_frac={cold_frac:<8g} r={r:.3e} dwell~={d:,.0f} writes")
    print()
    print("conclusion: dwell = B / r has no finite supremum over the set of "
          "workloads the fill-only-triggered policy must tolerate (r -> 0 "
          "gives dwell -> infinity for ANY fixed B); this is provable "
          "directly from the stated flush policy, the 18 sampled points in "
          "the round-2 report are a numerical calibration of this formula, "
          "not the basis for the no-bound claim -- the no-bound claim is "
          "stronger than 'extrapolation', it is a closed-form consequence.")
    print("emitted=1")


if __name__ == '__main__':
    main()
