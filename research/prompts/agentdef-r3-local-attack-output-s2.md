1
| action label | what the writer does, in one sentence | which FACT item(s) above establish that this action, by itself, is necessary |
ACTION-1 | Set the OLD-FACT column to begin with the prefix "Newly established:" | FACT-8 and FACT-9 |
Q1 overturn condition: Observing a fact table row with OLD-FACT starting with "Newly established:" that produces one or more rows in the candidate table would overturn the answer.

| action label | trace in the fact table: write LEAVES-TRACE or NO-TRACE, then one sentence on what the trace looks like if LEAVES-TRACE, referring only to the fact table's own column names from FACT-5 (ID, OLD-FACT, NEW-FACT, SEARCH-TERM, SOURCE) | trace in the candidate table: write LEAVES-TRACE or NO-TRACE, then one sentence on what the trace looks like if LEAVES-TRACE, referring only to the candidate table's own column names from FACT-6 (GROUP, OLD-FACT, NEW-FACT, CARRIER, ORIGINAL-TEXT) |
ACTION-1 | LEAVES-TRACE because the OLD-FACT column explicitly contains the "Newly established:" prefix | NO-TRACE because no candidate rows are generated for this group due to the skip rule |
Q2 overturn condition: Observing a fact table row with OLD-FACT starting with "Newly established:" that appears in the candidate table would overturn the answer.

CANNOT-STATE-CRITERION
The fact table's columns do not include any information about the repository's prior state or historical mentions of the fact, so there is no way to determine from the fact table alone whether the "Newly established:" prefix was applied correctly or misapplied.
Q3 overturn condition: Observing a rule that correctly distinguishes genuine from mislabeled "Newly established:" rows using only the fact table's columns (ID, OLD-FACT, NEW-FACT, SEARCH-TERM, SOURCE) would overturn the answer.
