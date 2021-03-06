// SPDX-License-Identifier: Apache-2.0

use sallyport::Block;
use sgx::types::ssa::{Exception, StateSaveArea};

use crate::handler::{Context, Handler};
use crate::Layout;

// Opcode constants, details in Volume 2 of the Intel 64 and IA-32 Architectures Software
// Developer's Manual
const OP_SYSCALL: &[u8] = &[0x0f, 0x05];
const OP_CPUID: &[u8] = &[0x0f, 0xa2];
const SYS_ERESUME: usize = !0;

#[no_mangle]
pub extern "C" fn event(
    _rdi: u64,
    _rsi: u64,
    block: &mut Block,
    layout: &Layout,
    _r8: u64,
    _r9: u64,
    aex: &mut StateSaveArea,
    ctx: &Context,
) {
    let mut h = Handler::new(layout, aex, ctx, block);

    // Exception Vector Table
    match h.aex.gpr.exitinfo.exception() {
        Some(Exception::InvalidOpcode) => {
            match unsafe { h.aex.gpr.rip.into_slice(2usize) } {
                OP_SYSCALL => {
                    let ret = match h.aex.gpr.rax.into() {
                        libc::SYS_read => h.read(),
                        libc::SYS_readv => h.readv(),
                        libc::SYS_write => h.write(),
                        libc::SYS_writev => h.writev(),
                        libc::SYS_exit => h.exit(None),
                        libc::SYS_getuid => h.getuid(),
                        libc::SYS_getgid => h.getgid(),
                        libc::SYS_geteuid => h.geteuid(),
                        libc::SYS_getegid => h.getegid(),
                        libc::SYS_arch_prctl => h.arch_prctl(),
                        libc::SYS_exit_group => h.exit_group(None),
                        libc::SYS_set_tid_address => h.set_tid_address(),
                        libc::SYS_brk => h.brk(),
                        libc::SYS_uname => h.uname(),
                        libc::SYS_mprotect => h.mprotect(),
                        libc::SYS_mmap => h.mmap(),
                        libc::SYS_munmap => h.munmap(),
                        libc::SYS_rt_sigaction => h.rt_sigaction(),
                        libc::SYS_rt_sigprocmask => h.rt_sigprocmask(),
                        libc::SYS_sigaltstack => h.sigaltstack(),
                        libc::SYS_getrandom => h.getrandom(),
                        libc::SYS_clock_gettime => h.clock_gettime(),
                        libc::SYS_madvise => h.madvise(),
                        libc::SYS_close => h.close(),
                        libc::SYS_poll => h.poll(),

                        _ if !crate::handler::TRACE => Err(libc::ENOSYS),
                        syscall => {
                            debugln!(h, "unsupported syscall: 0x{:x}", syscall as u64);
                            Err(libc::ENOSYS)
                        }
                    };

                    aex.gpr.rip = (usize::from(aex.gpr.rip) + 2).into();
                    match ret {
                        Err(e) => aex.gpr.rax = (-e).into(),
                        Ok([rax, rdx]) => {
                            aex.gpr.rax = rax.into();
                            aex.gpr.rdx = rdx.into();
                        }
                    }
                }

                OP_CPUID => {
                    h.cpuid();
                    aex.gpr.rip = (usize::from(aex.gpr.rip) + 2).into();
                }

                // unsupported opcode
                r => {
                    debugln!(h, "unsupported opcode: {:?}", r);
                    h.exit(1)
                }
            }
        }

        // Not InvalidOpcode
        _ => {
            h.attacked();
        }
    }

    block.msg.req.num = SYS_ERESUME.into();
}
