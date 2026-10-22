# NeoCruncher

<table align="center">
  <tr>
    <td align="center" width="160"><img src="assets/neocruncher-mark-transparent-candidate.png" alt="NeoCruncher monogram candidate" width="112"><br><strong>NeoCruncher</strong></td>
    <td align="center" width="160"><img src="assets/kolmogorov-mark.svg" alt="Kolmogorov Engine gold K and blue epsilon monogram" width="112"><br><strong>Kolmogorov Engine</strong></td>
    <td align="center" width="160"><img src="assets/neobayes-mark.svg" alt="NeoBayes red B and cyan epsilon monogram" width="112"><br><strong>NeoBayes Engine</strong></td>
  </tr>
</table>

<p align="center"><em>NeoCruncher and it's two engines: Kolmogorov and NeoBayes.</em></p>

**Research software for high-precision computation of mathematical constants**

NeoCruncher is a free and open-source command-line application from the
**Markov–Nekrasov Research Team**, developed with **OpenAI's GPT 6.1 Sol and
GPT 6 Astra**. It is a research platform for studying how
series evaluation, exact-integer arithmetic, scheduling, memory use, and decimal
conversion interact in high-precision constant computation. The public 1.0 beta
provides CPU, OpenCL GPU, and CPU/GPU hybrid execution paths. It also includes
the Kolmogorov Engine and PyKolm, an experimental CPU planning layer.

The release is a software and methods contribution. It does not claim a new
formula for π, a GPU speedup, or a performance record.

## Methods and architecture

| Layer | Approach in the 1.0 beta |
| --- | --- |
| Series evaluation | Binary splitting for the Chudnovsky π path, alongside other built-in constant methods. |
| Arithmetic | Exact integer operations on the CPU, with experimental OpenCL device paths and CPU/GPU coordination. |
| Resource planning | Admission estimates for memory and storage before a calculation; estimates are not hard reservations. |
| Output and checks | Integer-interval digit finalization where supported, plus an optional independent comparison of the requested output or a bounded prefix. |

The [release paper](neocruncher-paper.pdf) derives the Chudnovsky recurrence,
describes where work occurs in each backend, and separates internal correctness
arguments from independent output comparisons. PyKolm's planning and subtree
execution are documented there as experimental work.

## Documentation and release files

| Resource | Contents |
| --- | --- |
| [NeoCruncher 1.0 release archive](Neocruncher-1.0.0-release.zip) | Linux launchers, application and engine source, build metadata, and release notes. |
| [Release paper](neocruncher-paper.pdf) | Mathematical derivations, software architecture, execution paths, and verification scope. |
| [Benchmark companion](neocruncher-benchmarks.pdf) | Preliminary CPU and hybrid observations, measurement conditions, and evidence limits. |
| [Kolmogorov Engine and PyKolm](docs/KOLMOGOROV_ENGINE.md) | Released source layout, CPU subtree work, OpenCL paths, and current limits. |
| [NeoBayes Engine](docs/NEOBAYES_ENGINE.md) | Separate proposed OpenCL engine, research goals, and qualification criteria. |
| [Ubuntu 24.04 GUI trial 0.1.3](dist/NeoCruncher-GUI-Prototype-0.1.3-Ubuntu24.04-x86_64.AppImage) | AppImage for x86_64 with the released 1.0 CPU CLI. See the [Spanish run guide](dist/GUI-Prototype-0.1.3-README.md) and [checksum](dist/SHA256SUMS-0.1.3.txt). A [direct-run archive](dist/NeoCruncher-GUI-Prototype-0.1.3-Ubuntu24.04-x86_64.tar.gz) and [its checksum](dist/SHA256SUMS-0.1.3-archive.txt) are available if AppImageLauncher fails. |
| [Rust GUI trial source](gui/README.md) | Frontend, launcher, desktop-menu installer, dependency lockfile and build recipe. |
| [License](LICENSE) | GNU GPL version 3 only (`GPL-3.0-only`). |

The release archive contains its own README, `COPYING`, `NOTICE`, architecture
notes, and privacy and security review documents. The separately approved
GUI 0.1.3 is a desktop trial, not a NeoCruncher 1.1 release; GPU and hybrid
execution are not qualified in it. Other development builds and research
reports are distributed privately. The research auditor, private
testing database, and raw personal reports are not public.

## Run a small CPU calculation

On Linux, use a complete non-Conda Python 3.10 or newer with `venv` and
`ensurepip`. Extract the archive and run its root launchers:

```sh
unzip Neocruncher-1.0.0-release.zip
cd 'Neocruncher 1.0 release'
sh install.sh
sh run_neocruncher.sh --constant pi --digits 1000 --verify --output pi-small.txt
```

The installer also accepts `--gpu`. OpenCL execution requires a compatible
driver and ICD installed on the host; `sh doctor.sh --full` checks that path.
GPU-only and hybrid large-scale performance remain experimental. See the
archive README for interpreter selection, output handling, and verification
options.

## Evaluation and scope of evidence

| Evidence | What it supports | Limit |
| --- | --- | --- |
| Supplied CPU observations | Completion at the reported sizes, including a one-billion-place π run. | One observation per listed size; the billion-place independent comparison covered the first 100,000 decimal places. |
| Reported hybrid console runs | Completion and wall times in the supplied console record. | Complete phase and memory metadata are unavailable; the observations do not establish a hybrid speedup. |
| OpenCL GPU implementation | An experimental device execution path. | No controlled GPU speedup or portable large-size guarantee is established. |

These observations concern specific development configurations and are not a
fresh qualification of a byte-identical final archive. A prefix match does not
independently certify all remaining digits. The
[benchmark companion](neocruncher-benchmarks.pdf) distinguishes calculation
time from process wall time and explains its resource measurements and source
limitations.

## Development direction

The **Kolmogorov Engine** remains the central research engine, with PyKolm as
its experimental CPU planning and subtree-execution layer. A future Vulkan
compute backend and a separate **NeoBayes** engine remain research directions.
Performance work will be assessed against explicit workloads and correctness
checks. Separate production CLI and GUI releases remain planned. The GUI
0.1.3 trial is available for desktop feedback; engine development builds
remain private until their release review.

## Credit and license

We acknowledge **Alexander J. Yee** and
[y-cruncher](https://www.numberworld.org/y-cruncher/) as an important technical
reference and source of inspiration. This does not imply endorsement or a
controlled performance comparison.

NeoCruncher project code is licensed under **GNU GPLv3 only**. Copyright © 2026
Markov–Nekrasov Research Team. Third-party dependencies retain their own
licenses.

For academic reference: Markov–Nekrasov Research Team, *NeoCruncher and the
Kolmogorov Engine: An open-source hybrid-based computation project*, release
paper, 18 October 2026. The linked paper and benchmark companion state the
methods and evidence scope in detail.
