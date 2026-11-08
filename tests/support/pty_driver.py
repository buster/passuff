"""PTY driver for interactive assertions."""

from __future__ import annotations

import os
import pty
import subprocess
import threading
import time


def start_pty(cmd, env, buffer, lock):
    master, slave = pty.openpty()
    process = subprocess.Popen(
        cmd,
        stdin=slave,
        stdout=slave,
        stderr=slave,
        env=env,
        close_fds=True,
    )
    os.close(slave)

    def reader():
        while True:
            try:
                data = os.read(master, 4096)
            except OSError:
                break
            if not data:
                break
            with lock:
                buffer.append(data.decode(errors="replace"))
        try:
            os.close(master)
        except OSError:
            pass

    threading.Thread(target=reader, daemon=True).start()
    return process


def run_pty(cmd, env, keys: str = "q", timeout: float = 20):
    master, slave = pty.openpty()
    process = subprocess.Popen(
        cmd,
        stdin=slave,
        stdout=slave,
        stderr=slave,
        env=env,
        close_fds=True,
    )
    os.close(slave)
    output = []
    done = threading.Event()

    def reader():
        while True:
            try:
                data = os.read(master, 4096)
            except OSError:
                break
            if not data:
                break
            output.append(data.decode(errors="replace"))
        done.set()

    threading.Thread(target=reader, daemon=True).start()
    time.sleep(0.8)
    try:
        os.write(master, keys.encode())
    except OSError:
        pass
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)
    done.wait(timeout=2)
    try:
        os.close(master)
    except OSError:
        pass
    return "".join(output), process.returncode
