"""Q3 的跑法：今天的丁（ding_today.py）在第一轮攻方模型的合法 / 坏写者历史（history_today.py = history_ext.py 加 ring_at_raise）上跑。推的模型。
用法：python3 -B q3_today_run.py <段数> <写者> <S> <J> <p_rootfail> <配置名,配置名,…>
  写者：admission（合法）/ bad（越界）/ believes_landed；p_rootfail 非 0 时造「根槽写失败而本实例接着发」的历史（D23 已定项 14 第 406 行那一段落地之后的写法，今天的写者走不到）
输出每个配置一行：合法 / 非法抬 F 根的条数、窗口定不住而走到丁的条数、最终判词计数、部分没判的原因；
再两行：丁枚举得全不全（拿 ring_at_raise 当真值，逐槽核真住户在不在 possible_roots 交回的那几种里）、窗口外那一停与不停的判词逐条比。"""
import sys
from collections import Counter

import ding_today as dt
import history_today as hx

DEVS = (0, 1, 0)
PARAMS = dict(p_nonempty=0.35, p_lost_root=0.04, p_clean_crash=0.03, p_raise=0.08)


def image_of(h):
    slots = {s: (t, h.versions[t]['inst'], h.versions[t]['F'], h.versions[t]['state']) for s, t in h.ring.items()}
    jv = {}
    for t in h.journal:
        v = h.versions[t]
        jv.setdefault(t, [])
        if (v['inst'], v['F'], v['state']) not in jv[t]:
            jv[t].append((v['inst'], v['F'], v['state']))
    return slots, jv


def main():
    seeds, writer, S, J, p_rootfail = int(sys.argv[1]), sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5])
    names = sys.argv[6].split(',')
    tally = Counter()
    examples = {}
    for seed in range(seeds):
        steps = 20 + (seed * 7919) % 41
        h = hx.History2(seed, S, J, DEVS, steps, writer=writer, gen0=True, p_fallback=0.02, p_rootfail=p_rootfail, **PARAMS)
        slots, jv = image_of(h)
        for r in sorted({x[0] for x in slots.values()}):
            if r not in h.raises:
                continue
            root = (r, h.versions[r]['inst'], h.versions[r]['F'], h.versions[r]['state'])
            key = 'legal' if h.raises[r][3] else 'illegal'
            per_cfg = {}
            for name in names:
                j = dt.judge_raise_today(slots, jv, h.rows, root, DEVS, hx.R, S, dt.CONFIGS[name])
                if j is None:
                    continue
                per_cfg[name] = j
                tally[(name, key, 'count')] += 1
                if j['window'] == 'holds_undetermined':
                    tally[(name, key, 'reaches_ding')] += 1
                verdict, why = j['final']
                tally[(name, key, 'final', verdict)] += 1
                if verdict == 'partial':
                    tally[(name, key, 'why', why)] += 1
            if 'today' in per_cfg and per_cfg['today']['window'] == 'holds_undetermined':
                j = per_cfg['today']
                r_inst = root[1]
                rows = {i: T for (i, T) in h.rows.items() if i < r_inst}
                for region in range(hx.R):
                    for slot in range(S):
                        occ = slots.get((region, slot))
                        if occ is None or occ[0] < r:
                            continue
                        truth = h.ring_at_raise[r].get((region, slot))
                        ok = dt.true_occupant_is_enumerated(truth, region, slot, r, r_inst, rows, j['floor_before'], jv,
                                                            hx.R, S, dt.CONFIGS['today'])
                        tally[('enum', key, 'slots_overwritten_since_the_raise')] += 1
                        if ok is None:
                            tally[('enum', key, 'not_recognized')] += 1
                        elif ok:
                            tally[('enum', key, 'covered')] += 1
                        else:
                            tally[('enum', key, 'NOT_COVERED')] += 1
                            examples.setdefault(('not_covered', key), (seed, r, region, slot, truth, j['floor_before'], dict(h.rows),
                                                                      sorted(h.failed)))
            if 'today' in per_cfg and 'today_no_stop' in per_cfg:
                a, b = per_cfg['today']['final'], per_cfg['today_no_stop']['final']
                if a == b:
                    tally[('stop', key, 'same')] += 1
                elif b[0] == 'partial' and a[0] != 'partial':
                    tally[('stop', key, 'stop_resolves_what_no_stop_leaves_partial_' + b[1])] += 1
                else:
                    tally[('stop', key, 'DIFFERENT_%s_vs_%s' % (a[0], b[0]))] += 1
                    examples.setdefault(('stop_different', key), (seed, r, per_cfg['today'], per_cfg['today_no_stop']))
    for name in names:
        for key in ('legal', 'illegal'):
            n = tally[(name, key, 'count')]
            if not n:
                continue
            finals = ' '.join('%s=%d' % (v, tally[(name, key, 'final', v)]) for v in ('holds', 'red', 'partial', 'not_judged'))
            whys = ','.join('%s:%d' % (w, tally[(name, key, 'why', w)]) for w in ('journal0', 'journal2', 'combos', 'ambiguous', 'between', 'no_valid_root')
                            if tally[(name, key, 'why', w)])
            print('q3 writer=%s S=%d J=%d p_rootfail=%s cfg=%s %s_raise_roots=%d reaches_ding=%d final[%s] partial_why[%s] partial_ratio=%.4f'
                  % (writer, S, J, p_rootfail, name, key, n, tally[(name, key, 'reaches_ding')], finals, whys or '-',
                     tally[(name, key, 'final', 'partial')] / n))
    for key in ('legal', 'illegal'):
        n = tally[('enum', key, 'slots_overwritten_since_the_raise')]
        if n:
            print('q3enum writer=%s S=%d J=%d p_rootfail=%s %s slots_overwritten_since_the_raise=%d covered=%d NOT_COVERED=%d not_recognized=%d'
                  % (writer, S, J, p_rootfail, key, n, tally[('enum', key, 'covered')], tally[('enum', key, 'NOT_COVERED')],
                     tally[('enum', key, 'not_recognized')]))
        stop = sorted((k[2], v) for k, v in tally.items() if k[0] == 'stop' and k[1] == key)
        if stop:
            print('q3stop writer=%s S=%d J=%d p_rootfail=%s %s %s' % (writer, S, J, p_rootfail, key, ' '.join('%s=%d' % (k, v) for k, v in stop)))
    for k in sorted(examples, key=str):
        print('example', k, examples[k])


if __name__ == '__main__':
    main()
