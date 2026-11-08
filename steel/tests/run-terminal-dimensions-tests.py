#!/usr/bin/env python3
"""Exercise Steel terminal helpers with isolated Unix pipes and PTYs.

Requires Python >=3.11, TIOCSCTTY, and a Steel library-test executable containing
logger::terminal_dimensions_tests::dimensions_follow_real_terminal_descriptors.
No Cargo invocation or server startup is performed.
"""
import argparse
import errno
import fcntl
import os
from pathlib import Path
import pty
import selectors
import stat
import subprocess
import sys
import termios
import time

TEST = "logger::terminal_dimensions_tests::dimensions_follow_real_terminal_descriptors"
TIMEOUT = 10.0
MAX_OUTPUT = 65536
TEST_ARGS = ["--exact", TEST, "--ignored", "--nocapture",
             "--test-threads=1", "--color", "never"]
CTTY_MARKER = b"STEEL_CTTY_ATTACHED_STDOUT_PIPE"


def read_ready(fd, selector, deadline):
    remaining = deadline - time.monotonic()
    if remaining <= 0 or not selector.select(remaining):
        raise TimeoutError("terminal test output timeout")
    try:
        return os.read(fd, 4096)
    except OSError as error:
        if error.errno == errno.EIO:  # Linux PTY slave closed.
            return b""
        raise


def drive(binary, slave=None, ctty_slave=None):
    environment = dict(os.environ, TERM="dumb")
    command = [str(binary), *TEST_ARGS]
    options = {}
    if ctty_slave is not None:
        if slave is not None:
            raise ValueError("controlling-tty case must keep stdout as a pipe")
        command = [sys.executable, str(Path(__file__).resolve()),
                   str(binary), "--ctty-child", str(ctty_slave)]
        options["pass_fds"] = (ctty_slave,)
    proc = subprocess.Popen(
        command,
        stdin=subprocess.PIPE,
        stdout=slave if slave is not None else subprocess.PIPE,
        stderr=subprocess.STDOUT,
        start_new_session=True,
        env=environment,
        **options,
    )
    # The session starts without a ctty. Only the explicit wrapper case
    # attaches our own passed slave; the original two cases do not.
    return proc


def exercise(proc, read_fd, commands):
    transcript = bytearray()
    pending = bytearray()
    with selectors.DefaultSelector() as selector:
        selector.register(read_fd, selectors.EVENT_READ)
        for step, (before, command) in enumerate(commands):
            if before is not None:
                before()
            proc.stdin.write(command.encode() + b"\n")
            proc.stdin.flush()
            marker = f"STEEL_TERMINAL_OK {step}".encode()
            deadline = time.monotonic() + TIMEOUT
            while marker not in pending:
                block = read_ready(read_fd, selector, deadline)
                transcript.extend(block)
                pending.extend(block)
                if len(transcript) > MAX_OUTPUT:
                    raise RuntimeError("unexpectedly large terminal test output")
                if not block:
                    raise RuntimeError("child exited before ACK: " + repr(bytes(transcript)))
            del pending[:pending.index(marker) + len(marker)]
        proc.stdin.close()
        deadline = time.monotonic() + TIMEOUT
        # Parent retains the PTY slave for resize, so waiting for PTY EOF
        # here would deadlock. The final harness summary is the completion
        # marker; process exit/return code is checked immediately afterwards.
        while b"1 passed" not in transcript:
            block = read_ready(read_fd, selector, deadline)
            transcript.extend(block)
            if len(transcript) > MAX_OUTPUT:
                raise RuntimeError("unexpectedly large terminal test output")
            if not block:
                break
    code = proc.wait(timeout=TIMEOUT)
    if code != 0:
        raise RuntimeError(f"terminal test exit {code}: {bytes(transcript)!r}")
    # This also rejects a wrong executable/filter that reports zero tests.
    if b"1 passed" not in transcript:
        raise RuntimeError("expected exactly one passed ignored probe test")
    return bytes(transcript)


def stop_child(proc):
    if proc.poll() is None:
        proc.kill()
    proc.wait(timeout=TIMEOUT)
    if proc.stdin and not proc.stdin.closed:
        proc.stdin.close()
    if proc.stdout:
        proc.stdout.close()


def exec_with_ctty(binary, slave):
    # Popen's setsid has already run; do not call setsid again as its leader.
    if os.getsid(0) != os.getpid():
        raise RuntimeError("ctty wrapper is not a fresh session leader")
    if slave <= 2 or not os.isatty(slave):
        raise RuntimeError("ctty wrapper requires its own passed PTY slave")
    if not all(stat.S_ISFIFO(os.fstat(fd).st_mode) for fd in (0, 1, 2)):
        raise RuntimeError("ctty probe stdin/stdout/stderr must remain pipes")
    try:
        previous = os.open("/dev/tty", os.O_RDONLY | os.O_NOCTTY)
    except OSError:
        pass
    else:
        os.close(previous)
        raise RuntimeError("fresh wrapper unexpectedly already has a ctty")
    # arg0 refuses to steal a terminal owned by any other session.
    fcntl.ioctl(slave, termios.TIOCSCTTY, 0)
    controlling = os.open("/dev/tty", os.O_RDONLY | os.O_NOCTTY)
    os.close(controlling)
    os.close(slave)
    print(CTTY_MARKER.decode(), flush=True)
    # Same PID/session/stdio and exact same saved Rust test executable/flags.
    os.execv(str(binary), [str(binary), *TEST_ARGS])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--ctty-child", type=int, help=argparse.SUPPRESS)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    if not hasattr(termios, "tcsetwinsize"):
        parser.error("Python >=3.11 with tcsetwinsize is required")
    if not hasattr(termios, "TIOCSCTTY") or not hasattr(os, "O_NOCTTY"):
        parser.error("controlling-tty probe requires TIOCSCTTY and O_NOCTTY")
    if args.ctty_child is not None:
        exec_with_ctty(binary, args.ctty_child)
        return

    # No controlling tty and stdout is a pipe: actual window query must fail.
    proc = drive(binary)
    try:
        exercise(proc, proc.stdout.fileno(), [(None, "error")])
    finally:
        stop_child(proc)

    # Same child and same real PTY throughout: verifies live size/zero handling,
    # not just a newly initialized process or a pure arithmetic replacement.
    master, slave = pty.openpty()
    try:
        cases = [
            (91, 37, 91, 37),
            (137, 41, 137, 41),
            (0, 41, 80, 41),
            (137, 0, 137, 30),
            (0, 0, 80, 30),
            (65535, 65535, 65535, 65535),
            (91, 37, 91, 37),
        ]
        commands = []
        for columns, rows, width, height in cases:
            def resize(columns=columns, rows=rows):
                termios.tcsetwinsize(slave, (rows, columns))
                if termios.tcgetwinsize(slave) != (rows, columns):
                    raise RuntimeError("kernel did not retain requested PTY dimensions")
            commands.append((resize, f"window {columns} {rows} {width} {height}"))
        proc = drive(binary, slave)
        try:
            exercise(proc, master, commands)
        finally:
            stop_child(proc)

        # A third isolated process acquires this same exclusively owned PTY
        # as its ctty before exec, while its stdout stays a real pipe.
        proc = drive(binary, ctty_slave=slave)
        try:
            transcript = exercise(proc, proc.stdout.fileno(), commands)
            if CTTY_MARKER not in transcript:
                raise RuntimeError("controlling-tty wrapper was not observed")
        finally:
            stop_child(proc)
    finally:
        os.close(slave)
        os.close(master)
    print("PASS: detached pipe defaults, seven stdout-PTY states, "
          "and seven controlling-tty states with stdout piped")


if __name__ == "__main__":
    main()
