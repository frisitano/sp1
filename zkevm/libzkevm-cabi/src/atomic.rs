//! The `__atomic_*` library calls, which the succinct toolchain's prebuilt `std` makes for its
//! atomics (for example in `std::panicking`). SP1 guests built with `cargo prove` lower atomics
//! with `-C passes=lower-atomic` instead, but a program linking `libzkevm.a` is built by its own
//! toolchain, which does not provide them. SP1 runs a single thread, so each is a plain memory
//! operation; the memory order arguments do not apply.

macro_rules! atomic_libcalls {
    ($ty:ty, $load:ident, $store:ident, $exchange:ident, $compare_exchange:ident,
     $fetch_add:ident, $fetch_sub:ident, $fetch_and:ident, $fetch_or:ident, $fetch_xor:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $load(ptr: *const $ty, _order: i32) -> $ty {
            ptr.read_volatile()
        }

        #[no_mangle]
        pub unsafe extern "C" fn $store(ptr: *mut $ty, value: $ty, _order: i32) {
            ptr.write_volatile(value)
        }

        #[no_mangle]
        pub unsafe extern "C" fn $exchange(ptr: *mut $ty, value: $ty, _order: i32) -> $ty {
            let old = ptr.read_volatile();
            ptr.write_volatile(value);
            old
        }

        #[no_mangle]
        pub unsafe extern "C" fn $compare_exchange(
            ptr: *mut $ty,
            expected: *mut $ty,
            desired: $ty,
            _success: i32,
            _failure: i32,
        ) -> bool {
            let old = ptr.read_volatile();
            if old == expected.read() {
                ptr.write_volatile(desired);
                true
            } else {
                expected.write(old);
                false
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn $fetch_add(ptr: *mut $ty, value: $ty, _order: i32) -> $ty {
            let old = ptr.read_volatile();
            ptr.write_volatile(old.wrapping_add(value));
            old
        }

        #[no_mangle]
        pub unsafe extern "C" fn $fetch_sub(ptr: *mut $ty, value: $ty, _order: i32) -> $ty {
            let old = ptr.read_volatile();
            ptr.write_volatile(old.wrapping_sub(value));
            old
        }

        #[no_mangle]
        pub unsafe extern "C" fn $fetch_and(ptr: *mut $ty, value: $ty, _order: i32) -> $ty {
            let old = ptr.read_volatile();
            ptr.write_volatile(old & value);
            old
        }

        #[no_mangle]
        pub unsafe extern "C" fn $fetch_or(ptr: *mut $ty, value: $ty, _order: i32) -> $ty {
            let old = ptr.read_volatile();
            ptr.write_volatile(old | value);
            old
        }

        #[no_mangle]
        pub unsafe extern "C" fn $fetch_xor(ptr: *mut $ty, value: $ty, _order: i32) -> $ty {
            let old = ptr.read_volatile();
            ptr.write_volatile(old ^ value);
            old
        }
    };
}

atomic_libcalls!(
    u8,
    __atomic_load_1,
    __atomic_store_1,
    __atomic_exchange_1,
    __atomic_compare_exchange_1,
    __atomic_fetch_add_1,
    __atomic_fetch_sub_1,
    __atomic_fetch_and_1,
    __atomic_fetch_or_1,
    __atomic_fetch_xor_1
);
atomic_libcalls!(
    u16,
    __atomic_load_2,
    __atomic_store_2,
    __atomic_exchange_2,
    __atomic_compare_exchange_2,
    __atomic_fetch_add_2,
    __atomic_fetch_sub_2,
    __atomic_fetch_and_2,
    __atomic_fetch_or_2,
    __atomic_fetch_xor_2
);
atomic_libcalls!(
    u32,
    __atomic_load_4,
    __atomic_store_4,
    __atomic_exchange_4,
    __atomic_compare_exchange_4,
    __atomic_fetch_add_4,
    __atomic_fetch_sub_4,
    __atomic_fetch_and_4,
    __atomic_fetch_or_4,
    __atomic_fetch_xor_4
);
atomic_libcalls!(
    u64,
    __atomic_load_8,
    __atomic_store_8,
    __atomic_exchange_8,
    __atomic_compare_exchange_8,
    __atomic_fetch_add_8,
    __atomic_fetch_sub_8,
    __atomic_fetch_and_8,
    __atomic_fetch_or_8,
    __atomic_fetch_xor_8
);
