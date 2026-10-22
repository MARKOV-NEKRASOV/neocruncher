# Kolmogorov Engine and PyKolm

**Released component: 1.0.0a1, in NeoCruncher 1.0 beta.** Kolmogorov is the project's primary exact-arithmetic research engine. It is developed within the project and includes CPU/reference arithmetic, a PyKolm CPU subtree provider, and GPU-resident and host-assisted OpenCL paths. The public NeoCruncher adapter exposes Kolmogorov as a selectable **GPU or hybrid** engine; it rejects `--engine kolmogorov` for CPU-only runs. This distinction matters when interpreting results.

[Download the release source archive](../Neocruncher-1.0.0-release.zip) · [Release paper](../neocruncher-paper.pdf) · [Benchmark companion](../neocruncher-benchmarks.pdf)

The source is packaged inside the ZIP rather than browsable as individual GitHub files. The paths below are **paths within the extracted archive**, beneath `Neocruncher 1.0 release/`. They identify code, not separate repository links.

## PyKolm: exact CPU subtrees

`Kolmogorov-Engine-1.0.0a1/pykolm/tree.py` implements exact binary-splitting subtrees for Chudnovsky π and the supported e series. It uses GMP integers through `gmpy2` when available, with Python integers as a fallback. A `Session` splits a plan into bounded ordered child tasks for larger ranges, exchanges exact hexadecimal integer data with child processes, and merges adjacent results on a balanced frontier. Optional common-factor reduction applies to small Chudnovsky nodes; the application exposes `--pykolm-no-reduce` for comparison.

NeoCruncher selects this experimental **CPU subtree provider** with `--cpu-library pykolm`, independently of the GPU engine choice. It supports CPU and the CPU portion of explicit hybrid routes for Chudnovsky π, Taylor e, and inverse Taylor e. It is not a GPU kernel and does not add GPU-only or checkpoint-reuse support. Legacy CPU remains the reference provider in the 1.0 beta. PyKolm may become a separate distribution after its API, tests, and packaging are reviewed; that is a proposal.

Relevant source paths: `Kolmogorov-Engine-1.0.0a1/pykolm/{tree.py,worker.py}` and `NeoCruncher-1.0.0a1/neocruncher_app/{cpu_library.py,algorithms/host_tree.py,algorithms/classical.py}`.

A small Linux CPU check after extracting the ZIP and running the **bundle-root** `sh install.sh` is:

```sh
cd 'Neocruncher 1.0 release'
sh run_neocruncher.sh --constant pi --digits 1000 --cpu-library pykolm --verify --output pi-pykolm.txt
```

Use a fresh output name: the launcher will not overwrite an existing result.

## GPU and hybrid architecture

The current Kolmogorov GPU paths use **PyOpenCL and an OpenCL device runtime**. Project-owned arithmetic code does not make them independent of that API, the installed driver, or the device ICD. `Kolmogorov-Engine-1.0.0a1/kolmogorov/compute/integration.py` exposes an exact-integer integration API (`1.2.0`) for host-assisted OpenCL multiplication. It provides `prepare`, `multiply`, `multiply_many`, `capabilities`, and `metrics`; `multiply_many` currently invokes ordered products sequentially. OpenCL NTT, CRT/carry, transfer, and modular checking remain part of the product path.

For GPU-only computation, `kolmogorov/compute/resident/` keeps exact large integers in device storage when possible, performs exact limb arithmetic and NTT multiplication, and converts the final integer to decimal through checked quotient/remainder steps. The CPU still coordinates execution and output. Hybrid mode computes CPU subtrees and offloads eligible upper products to the selected GPU engine. A hybrid run can finish without a GPU product, so the result's product counters must be inspected before claiming acceleration. CPU-only NeoCruncher execution uses its CPU route; `--engine kolmogorov` is not a CPU-only choice in this release.

Relevant source paths: `Kolmogorov-Engine-1.0.0a1/kolmogorov/compute/{core.py,integration.py,gpu/opencl.py,resident/runtime.py,resident/drivers.py}` and `NeoCruncher-1.0.0a1/neocruncher_app/{engine_choice.py,arithmetic/kolmogorov_adapter.py,arithmetic/dispatch.py,arithmetic/resident.py}`.

On a Linux host with a compatible GPU, OpenCL driver and ICD, install the GPU option from the extracted bundle root and run `sh doctor.sh --full` before GPU work. The command-line application documents supported algorithms and routes with `sh run_neocruncher.sh --list-algorithms`. The current cloud development environment has no validated OpenCL path, so its CPU checks do not establish GPU correctness or speed.

## Evidence and current limits

The [benchmark companion](../neocruncher-benchmarks.pdf) contains historical CPU and hybrid reports, each with stated measurement and verification limits. They are not a controlled Kolmogorov-versus-NeoBayes comparison or a demonstrated GPU speedup. A 100,000-place independent prefix comparison verifies only that prefix of a larger output. Exact product tests, OpenCL device tests, full-output comparisons, and wall-time benchmarks are separate evidence.

CPU Chudnovsky and hybrid subtree checkpoints support recovery, while GPU-only checkpoints and disk-backed out-of-core arithmetic are not implemented in this release. A bounded disk tier, all-algorithm qualification, and Kolmogorov performance against a future NeoBayes engine remain research goals. A future OpenCL-independent GPU backend is under private study. Unsupported algorithm/route combinations should remain unavailable until implemented and tested.
