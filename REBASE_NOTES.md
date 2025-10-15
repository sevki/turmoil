# Rebase Notes

## Current Status

The WASM fork has been successfully rebased with conditional compilation:

### ✅ Working
- **Native build**: `cargo build` compiles successfully
- **WASM build**: `cargo build --target wasm32-unknown-unknown` compiles successfully
- **10 tests pass** in native mode

### ⚠️ Issues
- **14 tests fail** with timeouts (tests run ~10,000 steps but don't complete)

## Root Cause Analysis

The WASM fork made significant changes to remove functionality that doesn't exist in WASM environments:

### Removed Functionality

1. **UDP Multicast Support** (src/net/udp.rs)
   - Removed 416 lines of code
   - Deleted `MulticastGroups` struct and all related functionality
   - Removed from world.rs

2. **UDP Broadcast Support** (src/host.rs)
   - Removed broadcast flag from UDP binds
   - Removed `is_broadcast_enabled()`, `set_broadcast()` methods
   - Removed `DEFAULT_BROADCAST` and `DEFAULT_MULTICAST_LOOP` constants

3. **Examples**
   - Deleted `examples/cluster`
   - Deleted `examples/udp_vpv4_broadcast`
   - Deleted `examples/udp_vpv4_multicast`
   - Deleted `examples/udp_vpv6_multicast`

### Test Failures

The following tests timeout after ~10 seconds:
- `ip::tests::ip_version_v4`
- `ip::tests::ip_version_v6`
- `sim::test::elapsed_time`
- `sim::test::elapsed_time_across_crashes`
- `sim::test::elapsed_time_across_restarts`
- `sim::test::hold_release_peers`
- `sim::test::host_finishes_with_error`
- `sim::test::manual_message_delivery`
- `sim::test::multiple_clients_all_finish`
- `sim::test::override_link_latency`
- `sim::test::partition_peers`
- `sim::test::partition_peers_oneway`
- `sim::test::partition_peers_oneway_many_cases`
- `sim::test::restart_host_after_crash`

These tests don't directly use multicast/broadcast, but they're not completing. The simulation runs ~10,000 steps but never finishes, suggesting a logic issue introduced by the removed functionality or other changes in the WASM fork.

## Next Steps

To fully fix the rebase, one of the following approaches is needed:

### Option 1: Restore Removed Features (Recommended for Full Compatibility)
1. Restore multicast/broadcast functionality from upstream
2. Wrap it in `#[cfg(not(target_arch = "wasm32"))]` guards
3. Provide no-op or error implementations for WASM
4. Fix any logic that depends on these features
5. Restore deleted examples with conditional compilation

### Option 2: Accept Reduced Functionality
1. Skip or remove tests that depend on removed features
2. Document the breaking changes
3. Accept that WASM version has reduced functionality
4. Investigate why basic tests are timing out despite not using removed features

### Option 3: Find and Fix the Actual Bug
The tests are timing out even though they don't use multicast/broadcast. There may be a subtle bug introduced by other changes in the WASM fork. This requires:
1. Detailed debugging of why simulations don't complete
2. Comparing execution flow between upstream and WASM fork
3. Identifying the specific change that causes non-completion

## Changes Made So Far

### Commit 75bd700: Fix build by adding conditional compilation for WASM support

**Files Modified:**
- `.cargo/config.toml`: Removed forced `wasm32-unknown-unknown` target
- `.gitmodules`: Changed tokio submodule from SSH to HTTPS  
- `src/builder.rs`: Added `#[cfg(target_arch = "wasm32")]` guard for wasm_bindgen import
- `src/lib.rs`: Added conditional imports for IpAddr and wasm_bindgen
- `src/net/mod.rs`: Added conditional compilation to export std types for native, custom types for WASM
- `src/sim.rs`: 
  - Added `#[cfg(target_arch = "wasm32")]` guard for wasm_bindgen import
  - Fixed test to use `Instant::now()` instead of non-existent `crate::clock::Clock`
  - Added `Instant` to test imports
- `wasm-test/Cargo.toml`: Removed non-existent `wasm = "0.0.0"` dependency

**Result:**
- Both native and WASM builds compile successfully
- 10 tests pass, 14 tests fail with timeouts
- Basic functionality works but tests don't complete

## Testing

```bash
# Native build
cargo build

# WASM build  
cargo build --target wasm32-unknown-unknown

# Run tests (14 will timeout)
cargo test --package turmoil

# Run a specific test
cargo test --package turmoil ip_version_v4 -- --nocapture
```

## Upstream Comparison

Upstream tokio-rs/turmoil at commit 60f9276 has:
- All multicast/broadcast functionality intact
- All tests passing
- All examples working
- No WASM support

This fork has:
- WASM support with conditional compilation
- Multicast/broadcast removed
- Some tests failing
- Some examples deleted
