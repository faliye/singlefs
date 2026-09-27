SRC-CLASS ENUMERATES-PATHS
CLAIM-A FALSE
REG-FIELD-TWO kb-scribe,experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-1 to read the content of an experiments file, for example 'content = open('.claude/kb/experiments.md', 'r').read()'

SRC-CLASS NAMES-PATH-ONLY
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-2 to read an experiments file, such as 'with open(IDX, 'r') as f: data = f.read()'

SRC-CLASS NAMES-PATH-ONLY
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-3 to read an experiments file, for example 'content = open(EXP_DIR, 'r').read()'

SRC-CLASS NAMES-PATH-ONLY
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner,kb-scribe,gate-triage
CLAIM-B TRUE
This would be refuted by: changing SRC-4 to read an experiments file, such as 'with open(os.path.join(kb_dir, "experiments", "file.md"), 'r') as f: content = f.read()'

SRC-CLASS CHECKS-EXISTENCE
CLAIM-A FALSE
REG-FIELD-TWO kb-scribe,experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-5 to read an experiments file, for example 'content = open('.claude/kb/experiments', 'r').read()'

SRC-CLASS NAMES-PATH-ONLY
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-6 to read an experiments file, such as 'content = open(EXP_DIR, 'r').read()'

SRC-CLASS NAMES-PATH-ONLY
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-7 to read an experiments file, for example 'content = open(EXP_DIR, 'r').read()'

SRC-CLASS NAMES-PATH-ONLY
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-8 to read an experiments file, such as 'content = open(EXP_DIR, 'r').read()'

SRC-CLASS ENUMERATES-PATHS
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-9 to read the content of an experiments file, for example 'kb = [open(f).read() for f in glob.glob('.claude/kb/**/*.md', recursive=True)]'

SRC-CLASS NAMES-PATH-ONLY
CLAIM-A FALSE
REG-FIELD-TWO experiment-runner
CLAIM-B TRUE
This would be refuted by: changing SRC-10 to read an experiments file, such as 'content = open(EXPERIMENTS, 'r').read()'

NONE
This would be refuted by: changing SRC-1 to read an experiments file content and ensuring REG-1 includes experiment-runner

T1 ENUMERATES-PATHS T2 NAMES-PATH-ONLY T3 NAMES-PATH-ONLY T4 NAMES-PATH-ONLY T5 CHECKS-EXISTENCE T6 NAMES-PATH-ONLY T7 NAMES-PATH-ONLY T8 NAMES-PATH-ONLY T9 ENUMERATES-PATHS T10 NAMES-PATH-ONLY
This would be refuted by: changing any SRC item to read an experiments file content, for example SRC-1 to 'content = open('.claude/kb/experiments.md', 'r').read()'

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
This would be refuted by: changing SRC-1 to read an experiments file content, for example 'content = open('.claude/kb/experiments.md', 'r').read()'