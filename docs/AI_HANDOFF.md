# Registro de handoff entre IAs - Lay OS

Este arquivo existe para que qualquer IA (Claude, ChatGPT, etc.) que
continue o desenvolvimento do Lay OS saiba exatamente onde o trabalho
parou e o que vem a seguir, sem precisar reconstruir o contexto do zero.
Ao terminar uma sessao de trabalho, adicione uma nova secao "Entrada"
no topo (mais recente primeiro) e nao apague as entradas antigas.

---

## Entrada - 2026-09-29 (Claude, sessao 5, via GitHub direto no repo)

### CI estava vermelho desde a sessao 1 - causa raiz encontrada e corrigida
O usuario aplicou a mao a atualizacao pendente do `build.yml` (sessao 4)
no commit `098f09c`, e o Actions mostrou o erro de verdade: o `make`
falhava na montagem do boot loader com
`boot/boot.asm:207: error: TIMES value -18 is negative`. Ou seja, o
setor de boot passou de 512 bytes assim que a rotina `detect_memory`
(E820) foi adicionada na sessao 1 - **todo commit desde entao (sessoes
1 a 4) nunca chegou a rodar o kernel de verdade no CI**, so falhava na
montagem do `.asm`. Eu nao tinha como ver isso sem a ferramenta de
Actions; so descobri quando o usuario mandou prints da tela do Actions.

Corrigido no commit `e68cc45` (`boot/boot.asm`), sem tirar nenhuma
funcionalidade, cortando ~30 bytes do setor de boot:
- `detect_memory`: removida a checagem do bit de atributo ACPI 3.0
  estendido (bit0 de `[di+20]`). Todo BIOS/QEMU usado aqui reporta esse
  bit setado; o filtro de verdade (tipo de regiao == 1) continua no
  kernel (`kernel/memory.rs`).
- Removido `gdt64_descriptor`, que era uma copia byte-a-byte de
  `gdt_descriptor` apontando pra mesma tabela. Os dois `lgdt` (modo
  protegido e modo longo) agora reusam `gdt_descriptor`.
- Mensagem de erro de disco encurtada (`"LAY BOOT: disk err"`).

**Se o CI ainda estiver vermelho depois deste commit**, o proximo
suspeito e outra montagem NASM que também passou dos 512 bytes de novo,
ou um erro diferente - abra o job "Build Lay OS image" no Actions e
leia a mensagem do `nasm`/`make` primeiro, antes de mexer em qualquer
outra coisa.

### Observacao: branches `progress-91` / `progress-92` no historico
Vi no Actions (nao investiguei os arquivos) que ha commits em branches
separadas (`progress-91`, `progress-92`) com nomes como
"debug: report ATA DRQ...", "fix: reset ATA channel...", "ci: run
persistence test...", todos vermelhos - parece que alguem (ChatGPT,
provavelmente) tentou implementar o driver ATA real que eu recomendei
NAO fazer sem validacao (ver sessao 2) e esta iterando por tentativa e
erro direto no CI. Isso nao afeta a `main` (essas branches sao
separadas), mas se essas mudancas forem trazidas pra `main` depois,
vale reler a secao de riscos da sessao 2 antes de aceitar.

### Licao para toda sessao futura (Claude ou ChatGPT)
Qualquer edicao em `boot/boot.asm` deve terminar com uma contagem
mental do tamanho: o arquivo TEM que caber em exatamente 510 bytes de
codigo+dados antes do `times 510-($-$$) db 0` + `dw 0xAA55` finais. Nao
ha folga - qualquer bytes a mais quebra o `nasm` com "TIMES value
negative", e sem CI legivel isso pode passar sessoes inteiras
despercebido (foi o que aconteceu aqui).

### Proximos passos sugeridos
1. Confirmar no Actions que o commit `e68cc45` ficou verde. Se sim, so
   ENTAO os resultados das sessoes 1-4 (E820, allocator, multiprocesso)
   estao de fato validados pela primeira vez.
2. Espaco de enderecos por processo (PML4/PDPT/PD/PT proprios via
   `memory::alloc_page`, trocando CR3 no `activate` do scheduler).
3. Syscalls `spawn`/`wait` e carregar programas de um arquivo do VFS.
4. Driver de armazenamento real - ver riscos na sessao 2 ANTES de
   aceitar qualquer coisa vinda de `progress-91`/`progress-92`.
5. VFS/LayFS persistente (depende do passo 4).

---

## Entrada - 2026-09-28 (Claude, sessao 4, via GitHub direto no repo)

### IMPORTANTE: build.yml precisa ser atualizado manualmente
A integracao do GitHub que a Claude usa NAO tem permissao para escrever
em `.github/workflows/*` (o GitHub bloqueia isso por seguranca para Apps
sem o escopo `workflows`, mesmo com o resto do repo liberado). O usuario
aplicou essa mudanca a mao na sessao 5 (commit `098f09c`).

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
