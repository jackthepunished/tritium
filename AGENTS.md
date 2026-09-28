# Working context

- Work in the WSL-native checkout at `/home/bahadir/dev/tritium`.
- Bahadir confirmed on 2026-09-22 that he currently has no access to a real FPGA board or other target hardware. Do not assume access to ARM devices, physical accelerators, or external power-measurement equipment.
- Prioritize work possible with the existing development machine: software correctness, CPU optimization, benchmark tooling, cross-compilation, RTL simulation, and generic synthesis.
- Physical board bring-up, on-device ARM measurements, timing validation on a board, and hardware energy measurements remain blocked on access. Do not make acquiring hardware a prerequisite for useful software progress or repeatedly ask about access; revisit when the user says availability has changed.
- Clearly distinguish simulation and cross-compilation evidence from measurements on physical target hardware.
