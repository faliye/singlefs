import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import importlib.util, sys
OPUS = sys.argv[1]
spec = importlib.util.spec_from_file_location('opus_q7', OPUS)
opus = importlib.util.module_from_spec(spec); spec.loader.exec_module(opus)
import q2_three_simplifications as mine
seeds = int(sys.argv[2])
bad = 0; total = 0
for writer in ('admission', 'bad'):
    for S in (4, 8):
        for J in (8, 32, 128):
            for seed in range(seeds):
                steps = opus.rand_steps(seed)
                ho = opus.History(seed, S, J, (0, 1, 0), steps, p_nonempty=0.35, p_lost_root=0.04, p_clean_crash=0.03, p_raise=0.08, writer=writer)
                outs = {r: (legal, ding) for (r, legal, window, yi, yi_tight, ding, c717) in opus.judge_all(ho)}
                hm = mine.History(seed, S, steps, 0.35, 0.04, 0.03, 0.08, writer, False)
                jset = set(hm.journal[-J:])
                mr = {r: legal for (r, legal) in mine.raise_roots(hm)}
                if set(mr) != set(outs):
                    bad += 1; print('raise-set differs', writer, S, J, seed, sorted(mr), sorted(outs)); continue
                for r, legal in mr.items():
                    total += 1
                    v = mine.judge(hm, r, jset, (2, 1, 0))
                    if v != outs[r][1] or legal != outs[r][0]:
                        bad += 1; print('differs', writer, S, J, seed, r, v, outs[r])
print('compared', total, 'raise roots; differing', bad)
