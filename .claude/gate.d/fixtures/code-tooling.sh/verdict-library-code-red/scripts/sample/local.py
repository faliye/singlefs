import os
import sqlite3
common = os.environ.get("SAMPLE_COMMON", ".")
library = os.path.join(common, "singlefs-sample-verdicts")
connection = sqlite3.connect(os.path.join(library, "verdicts.sqlite"))
