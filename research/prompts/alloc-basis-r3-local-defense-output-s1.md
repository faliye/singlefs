Question 1  
G8 does not violate the "same piece of code" prohibition because the runtime and checker use distinct code paths for the same logical predicate. The runtime’s predicate is embedded in its operational logic for block reclaim decisions, while G8’s checker implements the predicate as part of a standalone audit that walks the on-disk allocation records. The key mechanical difference is that the runtime’s code executes during live operations (e.g., when reclaiming blocks), whereas G8’s checker computes the predicate solely during audit time using disk data, without any shared code or function calls between the two. Quote 1’s prohibition targets identical code reuse, not identical logical formulas implemented separately.  

What would refute this: If the checker’s code for the threshold formula directly calls the runtime’s function (e.g., `max(F_effective, oldest_valid_root)` via a shared library), proving the same code is reused.  

Question 2  
G8 falls under the permitted "independent second implementation" of the formula, not a shared formula. The decision explicitly allows separate implementations for checksums and format parsing, where each side writes its own code. Here, the checker’s threshold computation is a distinct implementation (e.g., a separate function in the checker codebase), not a shared module or copied code. The boundary rule forbids sharing non-scalar values like arithmetic logic, but permits independent reimplementation. G8’s checker does not use the generated constants module for this formula (which is scalar-only), and instead writes its own code to compute the max, satisfying the "separate implementations" rule.  

What would refute this: If the checker’s threshold computation is implemented by importing and reusing the runtime’s function definition (e.g., via a shared header file), not writing it anewew.  

Question 3  
I-3.1 checks allocated bytes by walking reachable blocks from roots; it would not catch a wrong threshold because defer blocks are released and not reachable, so they don’t affect allocated counts. I-7.4 checks physical range reassignment of referenced blocks; defer blocks are unreferenced, so this check doesn’t apply to them. I-4.8 verifies checksums along root walks; defer blocks aren’t part of the root structure, so their checksums aren’t validated. None of these checks verify the defer count’s arithmetic value—they only validate structure, references, or checksums, not the count of blocks in defer state.  

What would refute this: If I-3.1’s allocated-byte sum included defer blocks (which it doesn’t, as defer blocks are unreferenced), proving it could detect defer errors.  

Question 4  
Yes: a cross-statistic check could be `allocated + free + unreclaimable + pending-delete + committed-reserve + defer == capacity per device`. Each term is maintained independently (allocated, free, etc., are existing stats), and the check sums them without using the reclaim-threshold predicate. This would catch a wrong defer count because if defer is miscounted, the sum would deviate from capacity, even if other stats are correct. G8 shares the threshold predicate and would miss such errors, but this check does not depend on it.  

What would refute this: If unreclaimable, pending-delete, or committed-reserve statistics do not exist (e.g., the project lacks these stats), making the sum impossible to compute.
