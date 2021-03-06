#include <sys/syscall.h>
#include <sys/uio.h> /* struct iovec */
#include <unistd.h>
#include <errno.h>
#include <time.h>

int *__errno_location(void) {
    static __thread int errnum = 0;
    return &errnum;
}

void _exit(int status) {
    asm(
        "syscall; ud2"
        :
        : "a" (SYS_exit), "D" (status)
        : "%rcx", "%r11"
    );

    while (1) {}
}

int main(void);
void _start(void) {
    _exit(main());
}

ssize_t read(int fd, void *buf, size_t count) {
    ssize_t rax;

    asm(
        "syscall"
        : "=a" (rax)
        : "a" (SYS_read), "D" (fd), "S" (buf), "d" (count)
        : "%rcx", "%r11"
    );

    if (rax < 0) {
        errno = -rax;
        return -1;
    }

    return rax;
}

ssize_t readv(int fd, const struct iovec *iov, int iovcnt) {
    ssize_t rax;

    asm(
        "syscall"
        : "=a" (rax)
        : "a" (SYS_readv), "D" (fd), "S" (iov), "d" (iovcnt)
        : "%rcx", "%r11"
    );

    if (rax < 0) {
        errno = -rax;
        return -1;
    }

    return rax;
}

ssize_t write(int fd, const void *buf, size_t count) {
    ssize_t rax;

    asm(
        "syscall"
        : "=a" (rax)
        : "a" (SYS_write), "D" (fd), "S" (buf), "d" (count)
        : "%rcx", "%r11"
    );

    if (rax < 0) {
        errno = -rax;
        return -1;
    }

    return rax;
}

int clock_gettime(clockid_t clk_id, struct timespec *tp) {
    int rax;

    asm(
        "syscall"
        : "=a" (rax)
        : "a" (SYS_clock_gettime), "D" (clk_id), "S" (tp)
        : "%rcx", "%r11"
    );

    if (rax < 0) {
        errno = -rax;
        return -1;
    }

    return rax;
}

int is_enarx() {
    ssize_t rax;

    asm(
        "syscall"
        : "=a" (rax)
        : "a" (SYS_fork)
        : "%rcx", "%r11"
    );

    switch (rax) {
        case 0: _exit(0);
        case -ENOSYS: return 1;
        default: return 0;
    }
}
