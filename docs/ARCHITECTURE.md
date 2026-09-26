# Arquitetura do Lay OS

## Camadas

Hardware
↓
Lay Boot
↓
Lay Kernel
↓
Drivers / Hardware Abstraction
↓
System Services
↓
Lay Graphics / UI
↓
Lay Shell
↓
Applications

## Kernel inicial

O primeiro núcleo terá somente o necessário para iniciar a máquina com segurança:

- ponto de entrada
- inicialização básica da CPU
- preparação da memória
- interrupções
- temporizador
- scheduler
- processos e threads
- IPC
- syscalls
- gerenciador de módulos

Recursos superiores serão adicionados fora do caminho crítico de boot.

## Regra de eficiência

> Se um componente não está sendo usado, ele não deve consumir recursos desnecessariamente.

O sistema deverá adaptar caches, efeitos gráficos, serviços e módulos aos recursos disponíveis.
