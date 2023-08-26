use std::sync::atomic::AtomicIsize;

/// Pads and aligns a value to the length of a cache line.
/// Code borrowed from `https://github.com/ibraheemdev/seize/blob/master/src/utils.rs`
#[cfg_attr(
    any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "powerpc64",
    ),
    repr(align(128))
)]
#[cfg_attr(
    any(
        target_arch = "arm",
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "riscv64",
    ),
    repr(align(32))
)]
#[cfg_attr(target_arch = "s390x", repr(align(256)))]
#[cfg_attr(
    not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "powerpc64",
        target_arch = "arm",
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "riscv64",
        target_arch = "s390x",
    )),
    repr(align(64))
)]
pub struct CachePadded<T> {
    pub value: T,
}

pub fn make_new_padded_counter() -> CachePadded<AtomicIsize> {
    CachePadded {
        value: AtomicIsize::new(0),
    }
}
