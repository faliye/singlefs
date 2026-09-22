1  
| action label | what the writer does, in one sentence | which FACT item(s) above establish that this action, by itself, is necessary (cite only by FACT-# label) |  
| ACTION-1 | Set the OLD-FACT text to begin with the "Newly established:" prefix | FACT-9 |  

Q1 overturn condition: observing that a fact table row with OLD-FACT starting with "Newly established:" still produces one or more candidate rows in the candidate table would show that the minimum number of actions is too low.  

| action label | trace in the fact table: write LEAVES-TRACE or NO-TRACE, then one sentence on what the trace looks like if LEAVES-TRACE, referring only to the fact table's own column names from FACT-5 (ID, OLD-FACT, NEW-FACT, SEARCH-TERM, SOURCE) | trace in the candidate table: write LEAVES-TRACE or NO-TRACE, then one sentence on what the trace looks like if LEAVES-TRACE, referring only to the candidate table's own column names from FACT-6 (GROUP, OLD-FACT, NEW-FACT, CARRIER, ORIGINAL-TEXT) |  
| ACTION-1 | LEAVES-TRACE, the OLD-FACT column begins with "Newly established:" | NO-TRACE |  

Q2 overturn condition: observing that a fact table row with OLD-FACT starting with "Newly established:" appears in the candidate table would show that the trace in the candidate table is not NO-TRACE.  

CANNOT-STATE-CRITERION  
The fact table's columns do not include the repository's baseline text or historical content, so it is impossible to verify whether the SEARCH-TERM or OLD-FACT content actually matches prior mentions in the repository.  

Q3 overturn condition: if a rule using only the fact table's columns (ID, OLD-FACT, NEW-FACT, SEARCH-TERM, SOURCE) could reliably distinguish genuine "Newly established:" rows from mislabeled changed facts, that would contradict the conclusion.
