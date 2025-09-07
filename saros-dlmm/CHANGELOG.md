# Changelog

## [Unreleased] - Security Fixes

### Fixed
- **CRITICAL**: Fixed panic in `get_protocol_fee` function when overflow occurs
- **CRITICAL**: Fixed panic in `get_price_from_id` function with invalid inputs
- **CRITICAL**: Fixed panic in variable fee calculation with large values
- **MEDIUM**: Fixed unwrap in transfer fee calculation that could cause panics
- **MEDIUM**: Implemented missing `get_swap_and_account_metas` function

### Added
- Input validation for zero amounts in fee calculations
- Bin step validation to prevent division by zero
- Comprehensive error handling with proper Result types
- Edge case tests for overflow conditions
- Fuzz testing for mathematical operations
- New error code `InvalidBinStep`

### Changed
- `get_protocol_fee` now returns `Result<u64>` instead of `u64`
- `get_price_from_id` now returns `Result<u128>` instead of `u128`
- All mathematical operations now use checked arithmetic
- Improved error messages and error codes

### Security
- Eliminated all potential panic conditions in mathematical operations
- Added proper overflow/underflow protection
- Enhanced input validation throughout the codebase
- Improved error handling to prevent unexpected crashes

### Testing
- Added comprehensive test suite for edge cases
- Added overflow/underflow test scenarios
- Added zero input validation tests
- Added maximum value boundary tests
