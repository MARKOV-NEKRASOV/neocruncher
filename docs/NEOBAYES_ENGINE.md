# NeoBayes Engine

**Status: proposed separate engine; no implementation or release in NeoCruncher 1.0 beta.** NeoBayes is a planned OpenCL-based exact-arithmetic engine to coexist with Kolmogorov/PyKolm. It is not a rename, successor package, or compatibility alias for Kolmogorov. The project intends to compare the engines on matched hardware and workloads. No NeoBayes speed or correctness result is available yet.

[Current release source archive](../Neocruncher-1.0.0-release.zip) · [Kolmogorov and PyKolm documentation](KOLMOGOROV_ENGINE.md)

## Proposed architecture and interface

NeoBayes should have its own package identity, version, CLI engine choice, adapter, and result metadata. Its planned OpenCL implementation will need a compatible driver, ICD, and device. It may follow a versioned exact-integer interface with operations such as `prepare`, `multiply`, `multiply_many`, `capabilities`, and `metrics`, but this is a **design proposal**, not an existing NeoBayes API. Backend availability, supported algorithms, transform limits, memory budgets, and verification scope must be reported by the actual implementation before a job starts. A failed NeoBayes operation must not silently become a Kolmogorov or CPU result.

The 1.0 archive has no NeoBayes source tree or executable. Existing OpenCL source can inform design and benchmarking, but must not be presented as NeoBayes code. In the extracted archive, relevant **existing** paths are `Kolmogorov-Engine-1.0.0a1/kolmogorov/compute/{integration.py,gpu/opencl.py,resident/runtime.py}` and `NeoCruncher-1.0.0a1/neocruncher_app/{engine_choice.py,arithmetic/dispatch.py,arithmetic/ntt.py}`. These paths live inside the [release ZIP](../Neocruncher-1.0.0-release.zip); GitHub cannot browse them as separate files until a future reviewed source layout is published.

## Research and qualification

The first NeoBayes experiments should instrument exact multiplication, host/device transfers, NTT kernels, CPU/GPU routing, memory use, and decimal conversion. Compare exact products with an independent CPU implementation at representative lengths and boundaries. Modular residues alone cannot establish equality. For π output, retain the internal interval check and specify the independent comparison scope; a 100,000-digit prefix check does not certify the remaining digits of a 100-million-digit result.

The proposed large-scale goals are π at **100 million decimal places** in hybrid and GPU-only modes. Compare completed NeoBayes runs with current-release and revised Kolmogorov runs on the same host and GPU, with identical algorithm, output, verification, and resource limits. Record raw samples, full process wall, phase timings, peak RAM/VRAM, hashes, and failures. A current-release GPU-only run that times out supplies no finite speedup denominator. The broader NeoCruncher program also has a **1-billion-digit CPU** goal through its CPU/PyKolm research path; NeoBayes does not currently have a CPU route.

Disk-backed arithmetic is an urgent proposed pretrial, distinct from the current CPU/hybrid subtree checkpoints. It requires exact spill/reload, bounded memory, recovery and integrity checks, and a measured wall-time cost before any out-of-core claim. Every implemented algorithm and constant must be tested and measured on the routes the engine advertises. Planned formulas and CPU numerical-reference methods do not acquire GPU support merely because an exact multiplier exists.

## Publication requirements

Before release, publish direct links to NeoBayes's actual source and standalone technical documentation, after the implementation exists. Keep Kolmogorov and NeoBayes identities separate in the GitHub landing page, source, versioned dependencies, reports, and release notes. The planned CLI-only and Linux-first Rust GUI distributions need separate validated installation paths; neither is a NeoBayes artifact today. Audit public files and artifacts so private testing data and sensitive records are not included.
