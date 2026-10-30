# Princípios de design e experiência

## 1. Todo app é um app (sem marcação de origem)

No Lay-OS, **nenhum aplicativo é marcado pela plataforma de origem**. Isso vale para apps nativos, APKs (Play Store, F-Droid, APK avulso ou APK debug) e qualquer outro formato suportado.

Regras:

- O ícone aparece **igual** ao de qualquer outro app: ícone original do app e nome original, sem selo, badge, etiqueta, moldura ou cor diferente.
- Nenhum texto como "APK", "Android" ou "compatibilidade" aparece na grade de apps, no dock, na tela de apps recentes, em notificações ou no seletor de apps.
- Abrir, fechar, alternar entre apps, notificações, permissões e atalhos funcionam **exatamente do mesmo jeito** para todos os apps.
- O app roda em janela própria do sistema (via window-bridge), sem moldura ou barra que denuncie o runtime.

## 2. A origem existe só internamente

O sistema precisa saber de onde cada app vem (para escolher o runtime, aplicar sandbox e depurar), mas isso fica nos metadados internos:

```json
{
  "id": "com.exemplo.app",
  "name": "Nome do App",
  "icon": "icon.png",
  "runtime": "lay-android"
}
```

O campo `runtime` **nunca** é exibido na interface do usuário. O Lay Shell só lê `id`, `name` e `icon`.

## 3. Consequências para a implementação

- O Lay Shell trata todos os apps por uma única interface (`AppEntry`), sem ramificação por runtime.
- O `lay-pkg` registra apps de qualquer origem no mesmo banco de apps instalados.
- Ícones adaptativos do Android são renderizados no formato de ícone do Lay-OS, para ficarem consistentes com os demais.
- A primeira abertura de um app Android pode iniciar o runtime sob demanda; o sistema deve esconder esse tempo com a animação normal de abertura de app (sem tela ou aviso de "iniciando Android").

## 4. Interface

O visual da interface será desenhado pelo dono do projeto. Este documento define apenas o comportamento: apps de qualquer origem aparecem e se comportam como apps normais.
