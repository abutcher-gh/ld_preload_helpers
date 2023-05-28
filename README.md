# ld_preload_helpers

Rust macros to run code at load time and override C functions.

Mostly useful for `LD_PRELOAD` hooks, hence the name.

# Examples

## C function overrides

Here `open` is overridden to print something then forward to the real `open`.
In the rare case that you should wish to recurse into the override, rather than
call the real API, use `crate::open`.  From anywhere else in the module, `open`
will refer to the override.  To refer to the real function from elsewhere in
the program, use `real_open` (as named at the declaration).

Multiple overrides may be specified within a single macro invocation and
multiple `extern_c_overrides` invocations are supported if desired.  This is
demonstrated below by an override of `getuid` which adds 2000 to the user's id.

In `std` mode (the default), panics in the override are caught and cause the catch
block to execute.  In `no_std` mode (not properly supported due to use of
`OnceLock`; see below), there is no special `panic` handling and the catch block is
ignored.

```rust
extern_c_overrides! {
  unsafe fn open/real_open(pathname: *const c_char, flags: c_int, mode: libc::mode_t) -> c_int {
    println!("RUST OPEN {:?} {:x} {:x}", unsafe { CStr::from_ptr(pathname) }, flags, mode);
    // panic!("oops");
    return open(pathname, flags, mode);
  } catch {
    errno::set_errno(errno::Errno(libc::ENODEV));
    return -1;
  }

  unsafe fn getuid/real_getuid() -> libc::uid_t {
    getuid() + 2000
  } catch { u32::MAX }
}
```

## Image load hook

Called whenever the program/library is loaded, prior to the main entry point
being run.  Note this demonstrates calling the overridden `getuid` and real
`getuid`.

Currently there can be only one of these owing to a fixed name being used for
the generated function.  But this may be extended in future to allow the user
to pass a name.

```rust
on_load! {{
  println!("INIT LIB {} {}", unsafe { getuid() }, unsafe { real_getuid() });
}}
```

## Example runs

```sh
$ LD_PRELOAD=target/debug/liboverride.so id
INIT LIB 2501 501
uid=2501 gid=501 euid=501
```

```sh
$ LD_PRELOAD=target/debug/liboverride.so wc Cargo.lock
INIT LIB 2501 501
RUST OPEN "Cargo.lock" 0 52c47940
  820  1588 21572 Cargo.lock
```

# Notes on `no_std` usage

- `std::sync::OnceLock` is the only dependency on `std`.  Should consider `conquer_once` or similar crates to support `no_std`.
- Could also disable the need for this in single thread code to allow `no_std`.
- TODO: In `no_std` client code, the catch block is unused; consider not requiring/accepting it in that case.
