"""Q1 / Q2 的跑法：照搬的丁（ding_port）在第二轮攻方模型的历史与扩展历史上跑。推的模型。
用法：python3 q1q2_run.py <仓根> <段数> selftest|q1|q2|q2ext [写者 [S [J]]]（q2 / q2ext 可只跑一个写者、一个 S、一个 J；q1 可只跑一个写者、一个 S）"""
import importlib.util, os, sys
from collections import Counter
from ding_port import judge_raise, verdict_name
import history_ext as hx

VARIANTS = (('impl', frozenset()), ('T', frozenset({'T'})), ('T+deep', frozenset({'T', 'deep'})),
            ('gen0', frozenset({'gen0'})), ('all', frozenset({'T', 'deep', 'gen0'})))
PARAMS = dict(p_nonempty=0.35, p_lost_root=0.04, p_clean_crash=0.03, p_raise=0.08)
DEVS = (0, 1, 0)


def load_round2(repo):
    path = os.path.join(repo, 'research/prompts/m3-close-b19-design-r2-opus-model/q7/q7_cost_model.py')
    spec = importlib.util.spec_from_file_location('round2_opus_q7', path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def image_of(h):
    slots = {s: (t, h.versions[t]['inst'], h.versions[t]['F'], h.versions[t]['state']) for s, t in h.ring.items()}
    jv = {}
    for t in h.journal:
        v = h.versions[t]
        jv.setdefault(t, [])
        if (v['inst'], v['F'], v['state']) not in jv[t]:
            jv[t].append((v['inst'], v['F'], v['state']))
    return slots, jv


def raises_of(h, S, relax, ding_always=False):
    slots, jv = image_of(h)
    out = []
    for r in sorted({x[0] for x in slots.values()}):
        if r not in h.raises:
            continue
        root = (r, h.versions[r]['inst'], h.versions[r]['F'], h.versions[r]['state'])
        j = judge_raise(slots, jv, h.rows, root, DEVS, hx.R, S, relax, 0, ding_always)
        if j is not None:
            out.append((r, j))
    return out


def selftest(repo, seeds):
    m = load_round2(repo)
    bad = 0
    for S in (4, 8):
        for writer in ('admission', 'bad'):
            for seed in range(seeds):
                steps = m.rand_steps(seed)
                a = m.History(seed, S, 32, DEVS, steps, writer=writer, **PARAMS)
                b = hx.History2(seed, S, 32, DEVS, steps, writer=writer, **PARAMS)
                same = (a.ring == b.ring and a.rows == b.rows and list(a.journal) == list(b.journal)
                        and {k: v[:4] for k, v in a.raises.items()} == {k: v[:4] for k, v in b.raises.items()})
                bad += not same
    print(f"name=selftest_history_ext_equals_round2 seeds={seeds} mismatches={bad}")
    return bad == 0


def q2(repo, seeds, ext, writers=('admission', 'bad'), Ss=(4, 8), Js=(8, 32, 128)):
    m = load_round2(repo)
    for writer in writers:
        for S in Ss:
            for J in Js:
                tally = Counter()
                for seed in range(seeds):
                    steps = m.rand_steps(seed)
                    if ext:
                        h = hx.History2(seed, S, J, DEVS, steps, writer=writer, gen0=True, p_fallback=0.02, **PARAMS)
                    else:
                        h = m.History(seed, S, J, DEVS, steps, writer=writer, **PARAMS)
                    r2 = {} if ext else {row[0]: row[5] for row in m.judge_all(h)}
                    per_variant = {name: dict(raises_of(h, S, relax, True)) for name, relax in VARIANTS}
                    for r, j in per_variant['impl'].items():
                        legal = h.raises[r][3]
                        key = 'legal' if legal else 'illegal'
                        tally[(key, 'count')] += 1
                        if j['window'] == 'holds_undetermined':
                            tally[(key, 'reaches_ding')] += 1
                        for name, _ in VARIANTS:
                            jj = per_variant[name][r]
                            tally[(key, name, 'final', verdict_name(jj['final']))] += 1
                            tally[(key, name, 'ding_always', verdict_name(jj['ding']))] += 1
                            if not isinstance(jj['final'], str):
                                tally[(key, name, 'why', jj['final'][1])] += 1
                        if r in r2:
                            tally[(key, 'round2_ding', r2[r])] += 1
                tag = 'q2ext' if ext else 'q2'
                for key in ('legal', 'illegal'):
                    n = tally[(key, 'count')]
                    if n == 0:
                        continue
                    parts = [f"{tag} writer={writer} S={S} J={J} {key}_raise_roots={n} reaches_ding={tally[(key, 'reaches_ding')]}"]
                    for name, _ in VARIANTS:
                        fin = ' '.join(f"{v}={tally[(key, name, 'final', v)]}" for v in ('holds', 'red', 'partial', 'not_judged'))
                        da = ' '.join(f"{v}={tally[(key, name, 'ding_always', v)]}" for v in ('holds', 'red', 'partial'))
                        why = ','.join(f"{w}:{tally[(key, name, 'why', w)]}" for w in ('T', 'depth', 'gen0', 'journal0', 'journal2', 'combos', 'ambiguous', 'no_valid_root') if tally[(key, name, 'why', w)])
                        parts.append(f"{name}[final {fin} | ding_always {da} | partial_why {why or '-'}]")
                    if not ext:
                        parts.append('round2_ding[' + ' '.join(f"{v}={tally[(key, 'round2_ding', v)]}" for v in ('holds', 'red', 'partial')) + ']')
                    print(' '.join(parts))
                sys.stdout.flush()


def recon_detail(h, S, r, floor_before):
    """照搬判法重建的抬 F 那一刻的环（impl 口径），交回 (每槽 (区域,槽,镜像住户,u,认作), 重建上限)；认不出交回 (表, None)。"""
    from ding_port import largest_below, root_in_slot, valid_filter, ceiling_of_valid_roots
    slots, jv = image_of(h)
    r_inst = h.versions[r]['inst']
    rows = {i: T for (i, T) in h.rows.items() if i < r_inst}
    table, chosen = [], []
    for reg in range(hx.R):
        for s in range(S):
            occ = slots.get((reg, s))
            if occ is None:
                table.append((reg, s, None, None, None)); continue
            if occ[0] < r:
                table.append((reg, s, occ[0], None, occ[0])); chosen.append(occ); continue
            u = largest_below(reg, s, r, hx.R, S)
            got = root_in_slot(reg, s, u, r, r_inst, rows, floor_before, jv, hx.R, S, frozenset(), 0)
            if isinstance(got, tuple):
                table.append((reg, s, occ[0], u, 'ERR')); return table, None
            table.append((reg, s, occ[0], u, None if got[0] is None else got[0][0]))
            if got[0] is not None:
                chosen.append(got[0])
    valid = sorted([x for x in chosen if valid_filter(x, floor_before, rows)], key=lambda x: (x[0], x[1]))
    c = ceiling_of_valid_roots(valid, DEVS, hx.R)
    return table, (None if c is None else c[0])


def q1(seeds, writers=('admission', 'believes_landed'), Ss=(4, 8)):
    for writer in writers:
        for S in Ss:
            for J in (32, 128):
                tally = Counter(); examples = {}
                for seed in range(seeds):
                    steps = 20 + (seed * 7919) % 41
                    h = hx.History2(seed, S, J, DEVS, steps, writer=writer, p_rootfail=0.03, **PARAMS)
                    for r, j in raises_of(h, S, frozenset({'T', 'deep', 'doubt'})):
                        tally[('doubt', verdict_name(j['final']), h.raises[r][3])] += 1
                    for r, j in raises_of(h, S, frozenset()):
                        legal = h.raises[r][3]
                        fin = verdict_name(j['final'])
                        tally[('count', legal)] += 1
                        tally[(fin, legal)] += 1
                        if j['window'] != 'holds_undetermined' or fin == 'partial':
                            continue
                        table, rc = recon_detail(h, S, r, j['floor_before'])
                        mis = [row for row in table if row[3] is not None and row[3] in h.failed and row[4] == row[3]]
                        if not mis:
                            continue
                        tc = h.raises[r][2]
                        rel = 'recon_gt_true' if rc > tc else ('recon_lt_true' if rc < tc else 'recon_eq_true')
                        tally[('misrecognized', legal, fin, rel)] += 1
                        kind = ('leak' if (not legal and fin == 'holds') else
                                'false_red' if (legal and fin == 'red') else None)
                        if kind and kind not in examples:
                            examples[kind] = (seed, r, j['floor_before'], h.raises[r], rc, table, sorted(h.failed),
                                              h.N, dict(h.rows))
                print(f"q1 writer={writer} S={S} J={J} doubt_variant legal[holds={tally[('doubt', 'holds', True)]} red={tally[('doubt', 'red', True)]} "
                      f"partial={tally[('doubt', 'partial', True)]} not_judged={tally[('doubt', 'not_judged', True)]}] illegal[holds={tally[('doubt', 'holds', False)]} "
                      f"red={tally[('doubt', 'red', False)]} partial={tally[('doubt', 'partial', False)]} not_judged={tally[('doubt', 'not_judged', False)]}]")
                print(f"q1 writer={writer} S={S} J={J} raise_roots legal={tally[('count', True)]} illegal={tally[('count', False)]} "
                      f"final legal[holds={tally[('holds', True)]} red={tally[('red', True)]} partial={tally[('partial', True)]} not_judged={tally[('not_judged', True)]}] "
                      f"illegal[holds={tally[('holds', False)]} red={tally[('red', False)]} partial={tally[('partial', False)]} not_judged={tally[('not_judged', False)]}] "
                      "misrecognized_failed_root_via_ding[" + ' '.join(
                          f"{'legal' if k[1] else 'illegal'}/{k[2]}/{k[3]}={v}" for k, v in sorted(tally.items(), key=str)
                          if k[0] == 'misrecognized') + "]")
                for kind, ex in sorted(examples.items()):
                    seed, r, fb, rz, rc, table, failed, N, rows = ex
                    print(f"  example kind={kind} seed={seed} S={S} J={J} r={r} N={N} floor_before={fb} F_new={rz[0]} "
                          f"true_ceiling={rz[2]} writer_believed_ceiling={rz[4]} recon_ceiling={rc} failed_root_txgs={failed} rows={rows}")
                    for row in table:
                        print(f"    slot region={row[0]} slot={row[1]} image_txg={row[2]} u={row[3]} recognized_as={row[4]}")
                sys.stdout.flush()


if __name__ == '__main__':
    repo, seeds, what = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    if what == 'selftest':
        sys.exit(0 if selftest(repo, seeds) else 1)
    elif what in ('q2', 'q2ext'):
        writers = (sys.argv[4],) if len(sys.argv) > 4 else ('admission', 'bad')
        Ss = (int(sys.argv[5]),) if len(sys.argv) > 5 else (4, 8)
        Js = (int(sys.argv[6]),) if len(sys.argv) > 6 else (8, 32, 128)
        q2(repo, seeds, what == 'q2ext', writers, Ss, Js)
    elif what == 'q1':
        q1(seeds, (sys.argv[4],) if len(sys.argv) > 4 else ('admission', 'believes_landed'),
           (int(sys.argv[5]),) if len(sys.argv) > 5 else (4, 8))
