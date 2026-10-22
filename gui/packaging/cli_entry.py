"""Entry point for the bundled NeoCruncher command-line program."""

import multiprocessing
import sys

from neocruncher_app.cli import main


if __name__ == "__main__":
    multiprocessing.freeze_support()
    try:
        # NeoCruncher's job supervisor starts `sys.executable -m
        # neocruncher_app.job_worker`. In a frozen program sys.executable is
        # this entry point, so dispatch that one declared child module here.
        if sys.argv[1:3] == ["-m", "neocruncher_app.job_worker"]:
            from neocruncher_app.job_worker import main as worker_main

            sys.exit(worker_main())
        sys.exit(main())
    except (
        OSError,
        ArithmeticError,
        ValueError,
        NotImplementedError,
        RuntimeError,
        MemoryError,
        ImportError,
    ) as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        sys.exit(1)
    except KeyboardInterrupt:
        print("\nCancelled.", file=sys.stderr)
        sys.exit(130)
