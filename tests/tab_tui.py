"""Pilote un bash interactif dans un pty : `z proj` + Tab, choix 2 dans la TUI,
puis vérifie que le shell est arrivé dans le dossier sans Entrée supplémentaire.
Usage : python3 -I tests/tab_tui.py <dossier contenant le binaire zust>"""
import os, pty, re, select, sys, tempfile, time

bin_dir = os.path.abspath(sys.argv[1])
tmp = tempfile.mkdtemp()
for name in ("proj-a", "proj-b"):
    os.makedirs(os.path.join(tmp, name))
db = os.path.join(tmp, "db")
with open(db, "w") as f:
    f.write(f"{tmp}/proj-a;10;{int(time.time())}\n{tmp}/proj-b;5;{int(time.time())}\n")

env = {"HOME": tmp, "TERM": "xterm", "PATH": f"{bin_dir}:/usr/bin:/bin", "ZUST_DATA": db, "PS1": "$ "}
pid, fd = pty.fork()
if pid == 0:
    os.execve("/bin/bash", ["bash", "--norc", "--noprofile", "-i"], env)

def pump(seconds, answers=True):
    """Lit la sortie et joue le rôle du terminal pour les requêtes DSR."""
    out, end = b"", time.time() + seconds
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.05)
        if not r:
            continue
        try:
            chunk = os.read(fd, 4096)
        except OSError:  # le shell s'est terminé
            break
        out += chunk
        if answers and b"\x1b[6n" in chunk:
            os.write(fd, b"\x1b[1;8R")
        if answers and b"\x1b[5n" in chunk:
            os.write(fd, b"\x1b[0n")
    return out

os.write(fd, b'eval "$(zust init bash)"\r'); pump(0.5)
os.write(fd, b"z proj\t"); pump(0.5)
os.write(fd, b"2"); pump(0.5)
os.write(fd, b"echo RESULT=$PWD\r")
out = pump(0.5).decode(errors="replace")
os.write(fd, b"exit\r"); pump(0.2, answers=False)

expected = f"RESULT={tmp}/proj-b"
# Retire les séquences d'échappement (ex. bracketed paste `ESC[?2004l` émis par bash >= 5.1)
clean = re.sub(r"\x1b\[[0-9;?]*[A-Za-z]", "", out).replace("\r", "")
if expected in clean.splitlines():
    print("OK", expected)
else:
    print("FAIL, attendu", expected, "\n--- sortie:\n", out[-600:])
    sys.exit(1)
