# Deadlock Postmortem: RwLock in Async Rust with Trait Objects

**Date**: 2026-01-31  
**Severity**: High (Test suite hanging indefinitely)  
**Component**: `auth-service/src/routes/signup.rs`  
**Status**: Resolved

---

## Executive Summary

Integration tests for the signup handler were hanging indefinitely due to a **read-write lock deadlock** when attempting to acquire a write lock while holding a read lock. The issue was caused by improper lock lifetime management in async code, where a `RwLockReadGuard` was not explicitly dropped before attempting to acquire a `RwLockWriteGuard`.

---

## Technical Background

### What is a Deadlock?

A **deadlock** occurs when two or more tasks are waiting for each other to release resources, creating a circular dependency that prevents any task from proceeding. In Rust, this commonly happens with synchronization primitives like `RwLock` when:

1. A task holds a lock and tries to acquire another lock
2. Lock acquisition order is inconsistent across tasks
3. A lock guard's lifetime extends beyond where it's needed

### RwLock Behavior

`tokio::sync::RwLock` allows:
- **Multiple concurrent readers** (`read().await`)
- **One exclusive writer** (`write().await`)
- **Writers are blocked** while any read locks are held

**Critical rule**: You cannot upgrade a read lock to a write lock. Attempting to acquire a write lock while holding a read lock will deadlock if no one else can release the read lock.

---

## Root Cause Analysis

### The Problematic Code

```rust
// Line 34-41 in signup.rs (BEFORE FIX)
let store_guard = state.user_store.read().await;  // ← Acquire read lock
if store_guard.get_user(&req_email).await.is_ok() {
    return Err(AuthAPIError::UserAlreadyExists);  // ← Early return with lock held!
}

let user = User::new(payload.email, payload.password, payload.requires_2fa);

match state.user_store.write().await.add_user(user).await {  // ← DEADLOCK: Try to acquire write lock
    Ok(_) => Ok(SignupResponse::new("User created successfully!")),
    Err(_) => Err(AuthAPIError::UserAlreadyExists),
}
```

### Why This Deadlocked

**Execution Flow**:

1. **Line 34**: Handler acquires `RwLockReadGuard` and stores it in `store_guard`
2. **Line 35**: Calls async trait method `.get_user().await` while holding read lock
3. **Line 35-36**: In the success case, `store_guard` remains in scope
4. **Line 41**: Attempts to call `state.user_store.write().await`
5. **⚠️ DEADLOCK**: Write lock cannot be acquired because the read lock is still held by the same task

**Key Issue**: The `store_guard` variable was not dropped before attempting to acquire the write lock. Rust's drop checker doesn't automatically drop the guard because:
- The variable is still in scope
- Rust doesn't know the guard won't be used again
- Early returns can make lifetimes unpredictable

### Why Tests Hung (Not Panicked)

Rust deadlocks don't panic—they **block forever**:
- `write().await` waits indefinitely for all read locks to be released
- The read lock is held by the same task trying to acquire the write lock
- Single-threaded async runtime (tokio) can't make progress
- No timeout or deadlock detection in `RwLock`

---

## The Fix

### Solution: Explicit Lock Scoping

Use a scoped block to ensure the read lock is dropped before acquiring the write lock:

```rust
// Line 33-47 in signup.rs (AFTER FIX)
// If one attempts to create a new user with an existing email address, a 409 HTTP status code should be returned.
// Check with read lock first
let user_exists = {
    let store_guard = state.user_store.read().await;
    store_guard.get_user(&req_email).await.is_ok()
}; // ← Read lock dropped here when `store_guard` goes out of scope

if user_exists {
    return Err(AuthAPIError::UserAlreadyExists);
}

let user = User::new(payload.email, payload.password, payload.requires_2fa);

// Now safe to acquire write lock
match state.user_store.write().await.add_user(user).await {
    Ok(_) => Ok(SignupResponse::new("User created successfully!")),
    Err(_) => Err(AuthAPIError::UserAlreadyExists),
}
```

### Why This Works

1. **Scoped block (`{ ... }`)**: Creates an explicit scope for `store_guard`
2. **Immediate drop**: When the block ends, `store_guard` is immediately dropped
3. **Lock released**: Read lock is released before line 47
4. **Safe acquisition**: Write lock can now be acquired without deadlock

### Alternative Fix (Using `drop()`)

You can also explicitly drop the guard:

```rust
let store_guard = state.user_store.read().await;
let user_exists = store_guard.get_user(&req_email).await.is_ok();
drop(store_guard);  // ← Explicit drop

if user_exists {
    return Err(AuthAPIError::UserAlreadyExists);
}
```

Both approaches work, but scoped blocks are more idiomatic and harder to forget.

---

## Security & Correctness Implications

### Race Condition Introduced (Acceptable Trade-off)

The fix introduces a **time-of-check to time-of-use (TOCTOU)** race condition:

```rust
// Step 1: Check if user exists (read lock held, then released)
let user_exists = {
    let store_guard = state.user_store.read().await;
    store_guard.get_user(&req_email).await.is_ok()
};

// ⚠️ Gap: Another task could insert the same email here

// Step 2: Add user (write lock held)
match state.user_store.write().await.add_user(user).await {
```

**Why this is acceptable**:
- `HashmapUserStore::add_user()` checks for duplicates again while holding the write lock
- The check-then-insert is atomic at the data structure level
- The handler will return `UserAlreadyExists` error correctly

**Proper solution for production** (not needed here since `add_user` is atomic):
```rust
// Hold write lock for entire check-and-insert operation
let mut store = state.user_store.write().await;

if store.get_user(&req_email).await.is_ok() {
    return Err(AuthAPIError::UserAlreadyExists);
}

match store.add_user(user).await {
    Ok(_) => Ok(SignupResponse::new("User created successfully!")),
    Err(_) => Err(AuthAPIError::UserAlreadyExists),
}
```

This is more defensive but holds the write lock longer (blocking all other operations).

---

## Testing & Verification

### Before Fix
```bash
$ cargo test --test api signup
running 4 tests
test signup::should_return_422_if_malformed_input ... ok
test signup::should_return_400_if_invalid_input ... ok
test signup::should_return_201_if_valid_input ... [HANGS INDEFINITELY]
^C
```

### After Fix
```bash
$ cargo test --test api signup
running 4 tests
test signup::should_return_201_if_valid_input ... ok
test signup::should_return_400_if_invalid_input ... ok
test signup::should_return_409_if_email_already_exists ... ok
test signup::should_return_422_if_malformed_input ... ok

test result: ok. 4 passed; 0 failed
```

---

## Lessons Learned

### 1. **Always Drop Locks Before Acquiring Different Lock Types**

**❌ Bad**:
```rust
let read_guard = lock.read().await;
// ... use read_guard ...
let write_guard = lock.write().await;  // DEADLOCK
```

**✅ Good**:
```rust
let read_guard = lock.read().await;
// ... use read_guard ...
drop(read_guard);  // Or use scoped block
let write_guard = lock.write().await;
```

### 2. **Use Scoped Blocks for Explicit Lock Lifetimes**

Scoped blocks make lock lifetimes explicit and prevent accidental holding:

```rust
let data = {
    let guard = lock.read().await;
    guard.clone()  // Extract data
};  // Lock automatically dropped here
// Use `data` without holding lock
```

### 3. **Beware of Early Returns**

Early returns can extend lock lifetimes unexpectedly:

```rust
let guard = lock.read().await;
if condition {
    return Err(...);  // ← Lock still held!
}
```

**Fix**: Extract the check result before returning:
```rust
let should_error = {
    let guard = lock.read().await;
    condition
};

if should_error {
    return Err(...);  // Lock already dropped
}
```

### 4. **RwLock != Recursive Lock**

You cannot "upgrade" a read lock to a write lock:
- Read locks are shared (multiple readers)
- Write locks are exclusive (one writer)
- Attempting to acquire write while holding read = deadlock

### 5. **Use Clippy Lint: `await_holding_lock`**

Enable this lint to catch locks held across await points:

```toml
# .cargo/config.toml or Cargo.toml
[lints.clippy]
await_holding_lock = "deny"
```

### 6. **Consider Alternative Patterns**

For simple cases, consider:
- **Single write lock**: Hold write lock for entire check-and-modify operation
- **`parking_lot::RwLock`**: Detects deadlocks in debug builds
- **Message passing**: Use channels instead of shared state
- **Atomic operations**: For simple data types

### 7. **Test Async Code Thoroughly**

- Integration tests caught this deadlock (unit tests wouldn't)
- Always test concurrent access patterns
- Use `--test-threads=1` to debug hangs sequentially
- Consider timeout wrappers for tests:
  ```rust
  tokio::time::timeout(Duration::from_secs(5), test_fn()).await.unwrap()
  ```

---

## Checklist for Future Development

When working with `RwLock` in async Rust:

- [ ] Lock guards are scoped with `{ }` blocks or explicit `drop()`
- [ ] No early returns while holding locks
- [ ] Write locks acquired only after all read locks are dropped
- [ ] Locks held for minimal duration
- [ ] `clippy::await_holding_lock` lint enabled
- [ ] Integration tests cover concurrent scenarios
- [ ] Timeout wrappers added to async tests

---

## References

- [Tokio RwLock Documentation](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html)
- [Rust Async Book - Shared State](https://rust-lang.github.io/async-book/03_async_await/01_chapter.html)
- [RFC: Deadlock Detection in Rust](https://github.com/rust-lang/rfcs/issues/1815)
- [Clippy: await_holding_lock](https://rust-lang.github.io/rust-clippy/master/index.html#await_holding_lock)

---

**Author**: Authentication Service Team  
**Reviewers**: Senior Rust Engineers  
**Distribution**: All backend developers working with async Rust
