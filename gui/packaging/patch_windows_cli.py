"""Apply narrowly scoped Windows runtime fixes to the released 1.0 CLI source.

The input files are the public NeoCruncher 1.0 source archive, whose hashes are
checked before edits. This keeps the packaged Windows CPU CLI and the precise
changes used to build it reviewable. It does not enable GPU routes on Windows.
"""

from __future__ import annotations

import hashlib
from pathlib import Path
import sys


EXPECTED = {
    "hardware.py": "46b8e9cde123e64718ad9cb8d604981085927b42293b4c152f124a5a37097489",
    "jobs.py": "347030298a8db81257198535ed942a91f0cd4999c5713d270fd0254bf2077748",
}


def replace_once(source: str, before: str, after: str) -> str:
    if source.count(before) != 1:
        raise ValueError("expected exactly one source patch point")
    return source.replace(before, after)


def patch(source_dir: Path) -> None:
    package = source_dir / "neocruncher_app"
    for name, digest in EXPECTED.items():
        path = package / name
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != digest:
            raise ValueError(f"unexpected NeoCruncher 1.0 source: {path}")

    hardware = package / "hardware.py"
    text = hardware.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "def _meminfo():\n    data={};process_limit=None\n",
        '''def _meminfo():
    if sys.platform == 'win32':
        import ctypes
        class MEMORYSTATUSEX(ctypes.Structure):
            _fields_ = [('dwLength', ctypes.c_ulong),
                        ('dwMemoryLoad', ctypes.c_ulong),
                        ('ullTotalPhys', ctypes.c_ulonglong),
                        ('ullAvailPhys', ctypes.c_ulonglong),
                        ('ullTotalPageFile', ctypes.c_ulonglong),
                        ('ullAvailPageFile', ctypes.c_ulonglong),
                        ('ullTotalVirtual', ctypes.c_ulonglong),
                        ('ullAvailVirtual', ctypes.c_ulonglong),
                        ('ullAvailExtendedVirtual', ctypes.c_ulonglong)]
        status = MEMORYSTATUSEX()
        status.dwLength = ctypes.sizeof(status)
        if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)):
            raise OSError('Windows GlobalMemoryStatusEx failed')
        total, available = int(status.ullTotalPhys), int(status.ullAvailPhys)
        return {'total_bytes': total, 'available_bytes': available,
                'swap_total_bytes': max(0, int(status.ullTotalPageFile)-total),
                'process_limit_bytes': None, 'effective_capacity_bytes': total}
    data={};process_limit=None
''',
    )
    hardware.write_text(text, encoding="utf-8")

    jobs = package / "jobs.py"
    text = jobs.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "             text=True,env=env,start_new_session=os.name=='posix' and not env.get('NEO_AUDIT_GROUP'))",
        "             text=True,encoding='utf-8',env=env,start_new_session=os.name=='posix' and not env.get('NEO_AUDIT_GROUP'),\n"
        "             creationflags=(subprocess.CREATE_NO_WINDOW if os.name=='nt' else 0))",
    )
    jobs.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: patch_windows_cli.py EXTRACTED_NEOCRUNCHER_APP_SOURCE")
    patch(Path(sys.argv[1]))
