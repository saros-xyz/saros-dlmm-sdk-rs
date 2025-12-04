# Changelog

## [0.1.1] - 2025-10-01

### Fixes

#### 1. Corrected `bin_array_index()` logic to prevent out-of-range or misaligned bin selection
This new logic ensures:
	•	If the active bin is closer to the lower half of the current bin-array group, we snap to the previous group
	•	Guarantees the middle bin-array always contains or surrounds the active bin
	•	Improves compatibility with hook_bin_array_key and ensures reward accounting logic behaves as expected

#### 2. Standardized `compute_bin_array_swap` to always simulate from [middle, upper]
Now:
	•	Always uses [middle, upper] as the simulated bin-array range
	•	Simplifies assumptions for hooks and reward range pairing
	•	Ensures consistency and reproducibility between simulations and real swaps

### Packages updated
- `saros-dlmm-sdk` → `0.1.1`
- `saros-sdk` → `0.1.1`