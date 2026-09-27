1.  
Row 1. Seconds: 900. Milliseconds: 900000.  
Row 2. Seconds: 600. Milliseconds: 600000.  
Row 3. Seconds: 240. Milliseconds: 240000.  
Row 4. Sign: positive.  
Row 5. Sign: positive.  
Row 6. Sign: positive.  

The positive sign in Row 4 shows the default 900-second timeout exceeds the Bash tool’s 600-second foreground cap, meaning a 900-second call cannot finish within that cap. Row 5’s positive sign confirms it also exceeds the stricter 240-second ceiling from Fact E3. Row 6’s positive sign indicates the Bash tool’s cap (600s) is higher than E3’s cap (240s), making E3’s ceiling the more restrictive limit for foreground commands. This would be wrong if Row 4’s sign was negative, indicating the default ASK_LOCAL_TIMEOUT was less than the Bash tool’s foreground cap.  

2.  
Yes. The leg can compute in advance whether the call will finish within the foreground cap by checking the value of ASK_LOCAL_TIMEOUT (Fact A) against the Bash tool’s fixed foreground cap of 600 seconds (Fact B). If ASK_LOCAL_TIMEOUT exceeds 600 seconds, the call will not finish within the cap and must run in the background; otherwise, it can run in the foreground. This relies on Facts A and B. This would be wrong if the ASK_LOCAL_TIMEOUT environment variable was not accessible before the call starts or if the Bash tool’s cap was not a fixed value.  

3.  
| Fact | Requirement for ask-local.sh | Specific checkable condition? | Condition description |  
|------|-----------------------------|-------------------------------|-----------------------|  
| C | Foreground by default with background fallback | Yes | ASK_LOCAL_TIMEOUT > Bash tool’s foreground cap (600s) |  
| D | Background unconditionally | No | None (unconditional) |  
| E1 | Background unconditionally if long-lived task | Yes | Whether the task is as "long-lived" (e.g., duration exceeds foreground cap) |  
| Pair | 300s call | 700s call | Same action for both? |  
| C vs D | Different (C: foreground, D: background) | Same (both background) | No |  
| C vs E1 | Different (C: foreground, E1: background) | Same (both background) | No |  
| D vs E1 | Same (both background) | Same (both background) | Yes |  

This would be wrong if Fact D’s wording included a condition (e.g., "when ASK_LOCAL_TIMEOUT exceeds 600s") instead of being unconditional.  

4.  
Sub-case (i):  
- Range of seconds where Bash tool kills before curl: 241–900 seconds.  
- Destination file: Created and partially written.  
- Exit code: Non-zero (eating timeout termination).  
- Derived from facts: Timeout values (240s from Fact E3, 900s from Fact A).  
- Inferred: File partial write and non-zero exit code (shell behavior not explicitly stated in facts).  

Sub-case (ii):  
- Range of seconds where Bash tool kills before curl: 601–900 seconds.  
- Destination file: Created and partially written.  
- Exit code: Non-zero (indicating timeout termination).  
- Derived from facts: Timeout values (600s from Fact B, 900s from Fact A).  
- Inferred: File partial write and non-zero exit code (shell behavior not explicitly stated in facts).  

This would be wrong if the Bash tool’s timeout parameter did not terminate the command early but instead allowed it to complete beyond the cap, contradicting Fact B’s ceiling description.
