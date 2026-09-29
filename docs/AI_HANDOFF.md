# Registro de handoff entre IAs - Lay OS

Este arquivo existe para que qualquer IA (Claude, ChatGPT, etc.) que
continue o desenvolvimento do Lay OS saiba exatamente onde o trabalho
parou e o que vem a seguir, sem precisar reconstruir o contexto do zero.
Ao terminar uma sessao de trabalho, adicione uma nova secao "Entrada"
no topo (mais recente primeiro) e nao apague as entradas antigas.

---

## Entrada - 2026-09-28 (Claude, sessao 4, via GitHub direto no repo)

### IMPORTANTE: build.yml precisa ser atualizado manualmente
A integracao do GitHub que a Claude usa NAO tem permissao para escrever
em `.github/workflows/*` (o GitHub bloqueia isso por seguranca para Apps
sem o escopo `workflows`, mesmo com o resto do repo liberado). Entao as
mudancas desta sessao no kernel foram commitadas, mas a atualizacao do
smoke test do CI (`.github/workflows/build.yml`) NAO foi aplicada e
precisa ser feita por voce (ou pelo ChatGPT, se ele tiver permissao) a
mao. O diff pretendido era, no step "Boot smoke test": imprimir o log
inteiro (`cat build/debug.log`) antes dos `grep`, e adicionar duas
verificacoes novas alem das existentes: `grep -F "selftest: page
allocator ok" build/debug.log` e `grep -F "process: pid 2 exited status
0" build/debug.log` (a checagem de `pid 1 exited status 0` ja existia
implicitamente, so nao era conferida explicitamente - vale adicionar
tambem). Sem isso o CI continua rodando, so nao valida ainda o
allocator nem o multiprocesso desta sessao.

### O que foi feito: multiplos processos de verdade
Ate aqui so existia o pid 1 fixo. Agora o kernel suporta varios
processos de usuario (pid 1..3; pid 0 e o proprio kernel).
- `kernel/process.rs`: tabela real (`TABLE`) com estado, pai e status de
  saida por pid. `create`/`exit`/`state`/`parent` valem para qualquer pid.
- `kernel/scheduler.rs`: slot N da tabela de tarefas = pid N. Novo
  `spawn(parent) -> Option<pid>` (reusa slots Dead). Helper `activate()`
  centraliza a troca de tarefa. No boot sobem 2 processos identicos.
- **Bug latente corrigido antes de escalar:** o TSS tinha um unico RSP0
  (pilha de kernel para ring3 -> ring0). Com 2+ tarefas de usuario, uma
  interrupcao da tarefa B sobrescreveria o contexto salvo da tarefa A.
  Agora cada tarefa de usuario tem pilha de kernel propria e o
  scheduler chama `gdt::set_kernel_stack()` a cada troca.
- **Outro bug latente corrigido:** as paginas fixas do usuario (codigo em
  0x400000 e pilhas perto de 0x4FF000) nao eram reservadas no allocator,
  entao `alloc_page` poderia devolve-las depois de ~512 alocacoes. Novo
  `memory::reserve_page(addr)`; `user::init` reserva codigo e pilhas.
- `kernel/user.rs`: uma pagina de pilha de usuario por slot.
- `kernel/gdt.rs`: `set_kernel_stack(top)`.
- `kernel/shell.rs`: comandos `ps` e `spawn`.
- `kernel/selftest.rs`: teste de `reserve_page`.

### Limites conhecidos (por design, nesta fase)
- Todos os processos compartilham UM espaco de enderecos (mesma PT) e o
  mesmo codigo; so as pilhas de usuario sao separadas. Isolamento real
  (uma PML4/PT por processo) e o proximo passo.
- Nao ha `fork`/`exec`/`wait`; o programa de usuario e fixo.
- Maximo de 3 processos de usuario simultaneos (`MAX_USER_SLOTS`).

### Estado da validacao
- O workflow do CI nao foi atualizado (ver aviso no topo). Ate alguem
  aplicar essa mudanca a mao, o CI so confere o que ja conferia antes
  (`LAY OS KERNEL ONLINE` + a letra `U` no log), entao ele pode ficar
  verde mesmo que o multiprocesso desta sessao tenha um bug. Suspeitos
  mais provaveis se o boot travar: troca de RSP0 (`activate`/
  `gdt::set_kernel_stack`) ou o slot 2 (pilha em 0x4FF000 nao mapeada).

### Proximos passos sugeridos
1. Aplicar a mudanca pendente em `build.yml` (ver aviso acima) e
   conferir o CI.
2. Espaco de enderecos por processo (PML4/PDPT/PD/PT proprios via
   `memory::alloc_page`, trocando CR3 no `activate`).
3. Syscalls `spawn`/`wait` e carregar programas de um arquivo do VFS.
4. Driver de armazenamento real (ver sessao 2, riscos de colisao com a
   imagem de boot).
5. VFS/LayFS persistente (depende do passo 4).

---

## Entrada - 2026-09-28 (Claude, sessao 3, via GitHub direto no repo)

### O que foi feito
- `kernel/memory.rs` endurecido: `free_page` agora ignora enderecos nao
  alinhados, frames fora da RAM utilizavel e double free (bitmap
  `USABLE` separado). Frames liberados sao reutilizados do mais baixo
  para o mais alto. Removidas referencias sobre `static mut`
  (`.iter_mut()`), que quebrariam na edicao Rust 2024.
- `kernel/selftest.rs`: novo `memory_check()` no boot (alocacao,
  contabilidade, escrita/leitura, frees invalidos, double free, reuso).
  Se falhar, panic antes de `LAY OS KERNEL ONLINE` -> CI fica vermelho.

### Como a validacao funciona (importante)
- O usuario definiu que a validacao deve ser feita pelo GitHub Actions.
  Limite da IA: sem ferramenta para ler o resultado do Actions nem
  criar branch/PR nesta sessao. Alem disso (descoberto na sessao 4): a
  integracao tambem nao pode escrever em `.github/workflows/*`.

---

## Entrada - 2026-09-27 (Claude, sessao 2, via GitHub direto no repo)

### O que foi feito
- `kernel/user.rs`: a tabela de paginas (PT) do processo de usuario
  deixou de usar um endereco fisico hardcoded (0x103000) e passou a vir
  de `memory::alloc_page()`.

### O que eu avaliei e decidi NAO fazer (leia antes de mexer em storage)
- Cheguei a considerar substituir `kernel/block.rs` (RAM-disk puro) por
  um driver ATA/IDE PIO real (portas 0x1F0-0x1F7).
- **Nao fiz isso porque e arriscado sem conseguir rodar QEMU/CI a partir
  daqui para validar.** O unico disco que o QEMU usa hoje
  (`-drive format=raw,file=build/lay-os.img`) e a PROPRIA imagem de
  boot+kernel - os primeiros ~1024 setores sao o binario do kernel. O
  `block.rs` atual usa `BLOCK_COUNT = 128` (LBA 0-127), que colide
  direto com essa regiao.
- **Se for implementar o driver ATA real, fazer nesta ordem:**
  1. Aumentar `IMAGE_SIZE` no `Makefile`, reservar `LBA_BASE` bem acima
     do fim do kernel (ex.: 2048).
  2. `kernel/block.rs`: LBA real = `LBA_BASE + block_index`.
  3. Adicionar `inw`/`outw` em `kernel/arch.rs`.
  4. Leitura/escrita LBA28 em PIO com timeout limitado no polling de
     BSY/DRQ (nunca loop infinito).
  5. So depois disso: validar via CI antes de confiar no resultado.

---

## Entrada - 2026-09-27 (Claude, sessao 1, via GitHub direto no repo)

### O que foi feito
- Deteccao de memoria real via BIOS E820 (`boot/boot.asm`, rotina
  `detect_memory`). O boot loader grava contagem de entradas (u16) em
  `0x8FF0` e ate 64 entradas E820 cruas a partir de `0x9000` (contrato
  documentado em `docs/MEMORY_MAP.md`).
- `kernel/memory.rs` reescrito como bitmap allocator que le o mapa E820,
  libera so RAM usavel acima de 2 MiB, rastreia ate 128 MiB, com
  fallback para a janela fixa antiga se nao houver mapa.
- `kernel/console.rs`: `write_dec`. `kernel/main.rs`: RAM detectada em
  KiB no boot. `kernel/shell.rs`: comando `meminfo`.

### Decisoes que quem continuar deve conhecer
- O contrato de enderecos fixos (0x8FF0/0x9000) e deliberadamente
  simples porque boot loader (NASM) e kernel (Rust `no_std`) sao
  compilados separadamente, sem struct de "boot info" compartilhado.
- O teto de 128 MiB do bitmap e uma escolha deliberada para manter o
  allocator simples (array estatico, sem heap) nesta fase de prototipo.
