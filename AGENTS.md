# NeoCruncher project agent roles

Use the relevant role brief in `.agents/roles/` when working on NeoCruncher. These are durable task profiles based on the supplied project specifications. Assign a role only when the task calls for that work; the profiles do not authorize publishing, deploying, or changing project scope.

Current project priorities: first improve the GitHub landing page with an academic description, accurate links, and the three project marks. Advance **Kolmogorov/PyKolm** as the central research track, and investigate **NeoBayes** as a separate, coexisting OpenCL engine. The Math Researcher, Software Architect, and Software Engineer should use the user's forthcoming reports as inputs and share their findings. The performance goals are a measured 10× improvement for π at one billion decimal places on CPU and at 100 million places in hybrid and GPU-only execution, compared on the same hardware with matched correctness checks. Never present a goal as achieved without reproducible evidence. Test all supported algorithms and constants, and label unsupported routes explicitly. Investigate disk-backed arithmetic as a research pretrial; it is not an implemented feature. Develop a comfortable Linux-first Rust GUI and prepare separate CLI and GUI distributions. **Keep all LAB versions, engine prototypes, and their technical documentation out of public GitHub. The owner explicitly authorized only the GUI 0.1.3 trial package, source, and usage guide for public download; this does not authorize publishing other prototypes or a 1.1 release.**

The current GPU implementation of Kolmogorov uses OpenCL/PyOpenCL. The active research goal is a truly OpenCL-independent Kolmogorov GPU backend, with a bounded Vulkan feasibility prototype first. Keep Kolmogorov's interim legacy route explicit, and require a Vulkan-only install and exact-product/device test before describing the new route as independent. Coordinate mathematical proof obligations, architecture, and engineering in private research documents until a final release is approved.

The Git checkout currently packages the application and Kolmogorov Engine as `Neocruncher-1.0.0-release.zip`. For development, inspect the extracted release under `/workspace/neocruncher-bundle/Neocruncher 1.0 release` if present. The testing database supplied separately is private historical material; do not put it in the public checkout or treat its reports as freshly reproduced tests.

- [Math Researcher](.agents/roles/math-researcher.md)
- [Software Architect and Designer](.agents/roles/software-architect.md)
- [Software Engineer](.agents/roles/software-engineer.md)
- [UI Designer](.agents/roles/ui-designer.md)
