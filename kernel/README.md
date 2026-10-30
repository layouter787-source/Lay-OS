# kernel/

Configuracoes e patches do kernel Linux para o Lay-OS.

Planejado:
- `configs/layos_defconfig`: config base (cgroups v2, namespaces, seccomp, Landlock, Binder, memfd)
- `configs/<dispositivo>.config`: fragmentos por aparelho
- `patches/`: patches mantidos sobre o kernel LTS

O kernel Linux e licenciado sob GPL-2.0.
