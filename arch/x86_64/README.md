# Lay OS x86_64

Código específico da arquitetura x86_64.

O boot inicial prepara long mode. A partir daqui o kernel assume o controle e inicializa:
1. GDT
2. IDT
3. PIC/PIT
4. memória
5. scheduler
6. serviços essenciais
