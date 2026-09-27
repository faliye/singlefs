T1 ENUMERATES-PATHS FALSE kb-scribe,experiment-runner TRUE
This would be refuted by: changing SRC-1 to read a file, e.g., 'with open('.claude/kb/experiments.md', 'r') as f: content = f.read()'

T2 NAMES-PATH-ONLY FALSE experiment-runner TRUE
This would be refuted by: changing REG-2's second field to 'kb-scribe' or changing SRC-2 to read a file, e.g., 'content = open(IDX, 'r').read()'

T3 NAMES-PATH-ONLY FALSE experiment-runner TRUE
This would be refuted by: changing REG-3's second field to 'kb-scribe' or changing SRC-3 to read a file, e.g., 'content = open(EXP_DIR, 'r').read()'

T4 NAMES-PATH-ONLY FALSE experiment-runner,kb-scribe,gate-triage TRUE
This would be refuted by: changing REG-4's second field to 'kb-scribe,gate-triage' or changing SRC-4 to read a file, e.g., 'with open(directory, 'r') as f: content = f.read()'

T5 CHECKS-EXISTENCE FALSE kb-scribe,experiment-runner TRUE
This would be refuted by: changing SRC-5 to read content, e.g., 'cat .claude/kb/experiments/*' or changing REG-5's second field to 'kb-scribe'

T6 NAMES-PATH-ONLY FALSE experiment-runner TRUE
This would be refuted by: changing REG-6's second field to 'kb-scribe' or changing SRC-6 to read a file, e.g., 'content = open(EXP_DIR, 'r').read()'

T7 NAMES-PATH-ONLY FALSE experiment-runner TRUE
This would be refuted by: changing REG-7's second field to 'kb-scribe' or changing SRC-7 to read a file, e.g., 'content = open(EXP_DIR, 'r').read()'

T8 NAMES-PATH-ONLY FALSE experiment-runner TRUE
This would be refuted by: changing REG-8's second field to 'kb-scribe' or changing SRC-8 to read a file, e.g., 'content = open(EXP_DIR, 'r').read()'

T9 ENUMERATES-PATHS FALSE experiment-runner TRUE
This would be refuted by: changing SRC-9 to read a file, e.g., 'with open(kb[0], 'r') as f: data = f.read()' or changing REG-9's second field to 'kb-scribe'

T10 NAMES-PATH-ONLY FALSE experiment-runner TRUE
This would be refuted by: changing REG-10's second field to 'kb-scribe' or changing SRC-10 to read a file, e.g., 'content = open(EXPERIMENTS, 'r').read()'

NONE
This would be refuted by: changing one SRC line to read content and keeping REG field with experiment-runner, e.g., changing SRC-1 to read a file

T1 ENUMERATES-PATHS
T2 NAMES-PATH-ONLY
T3 NAMES-PATH-ONLY
T4 NAMES-PATH-ONLY
T5 CHECKS-EXISTENCE
T6 NAMES-PATH-ONLY
T7 NAMES-PATH-ONLY
T8 NAMES-PATH-ONLY
T9 ENUMERATES-PATHS
T10 NAMES-PATH-ONLY
This would be refuted by: changing SRC-1 to read a file like 'content = open('.claude/kb/experiments.md', 'r').read()'

SRC-1 NO
SRC-2 NO
SRC-3 NO
SRC-4 NO
SRC-5 NO
SRC-6 NO
SRC-7 NO
SRC-8 NO
SRC-9 NO
SRC-10 NO
This would be refuted by: changing SRC-1 to read a file like 'open('.claude/kb/experiments.md').read()' which would make it YES but CLAIM-A was FALSE
