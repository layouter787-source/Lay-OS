# Contrato do mapa de memoria (boot loader -> kernel)

O boot loader (`boot/boot.asm`) detecta a memoria fisica disponivel via
BIOS `INT 0x15, EAX=0xE820` enquanto ainda esta em modo real, e grava o
resultado em enderecos fisicos fixos que o kernel le depois de entrar em
modo longo. O kernel roda com paginacao de identidade nos primeiros
1 GiB (ver `boot/boot.asm`, `protected_mode:`), entao esses enderecos
continuam validos como ponteiros diretos em Rust.

## Layout

| Endereco | Tamanho       | Conteudo                              |
|----------|---------------|----------------------------------------|
| 0x8FF0   | 2 bytes (u16) | Numero de entradas do mapa de memoria  |
| 0x9000   | N x 24 bytes  | Entradas do mapa (formato E820 padrao) |

Cada entrada tem 24 bytes:

| Offset | Tamanho | Campo                              |
|--------|---------|-------------------------------------|
| 0      | 8 bytes | Endereco base (u64)                 |
| 8      | 8 bytes | Tamanho da regiao em bytes (u64)    |
| 16     | 4 bytes | Tipo da regiao (u32)                |
| 20     | 4 bytes | Atributos ACPI 3.0 (u32, ignorado)  |

Tipos de regiao relevantes: `1` = RAM utilizavel. Qualquer outro valor e
tratado como reservado e nunca e entregue pelo allocator de paginas
(`kernel/memory.rs`).

O kernel le no maximo 64 entradas (`kernel/memory.rs::MEMMAP_MAX_ENTRIES`,
que tambem e o limite que `detect_memory` respeita no boot loader). Isso e
folga confortavel: maquinas reais e QEMU tipicamente reportam entre 6 e 15
entradas.

## Reserva abaixo de 2 MiB

O kernel nunca entrega paginas abaixo de `RESERVED_BELOW = 0x0020_0000`
(2 MiB), independente do que o mapa E820 diga sobre essa faixa. E ali que
ficam: o setor de boot, o buffer do mapa de memoria (0x8FF0/0x9000), a
imagem do kernel carregada em 0x20000, as page tables iniciais em
0x100000-0x103000 e a pilha de boot do kernel (definida em
`kernel/linker.ld`).

## Por que enderecos fixos e nao um "boot info" de verdade

O boot loader e o kernel sao compilados/linkados separadamente (o boot
loader e montado com NASM, o kernel e um binario Rust `no_std` carregado
a parte). Nao ha passagem de argumentos nem um struct "boot info"
compartilhado ainda: o contrato e so "estes bytes nestes enderecos". Se no
futuro for adicionado um boot protocol mais completo (ex.: Multiboot2, ou
um `BootInfo` struct proprio passado em registrador), este documento deve
ser atualizado junto.

## Fallback

Se `count` em 0x8FF0 for 0 (BIOS sem suporte a E820, ou um boot loader que
nunca chamou `detect_memory`), `kernel/memory.rs::init` cai de volta para a
janela fixa original de 2..32 MiB, que era o comportamento antes desta
mudanca. O kernel sempre bootia; so a quantidade de RAM utilizavel varia.
