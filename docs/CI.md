# CI и защита default branch

Workflow [.github/workflows/ci.yml](../.github/workflows/ci.yml) запускается на PR и push в `main`. Изменения ветки открытого PR проверяются через событие synchronize; отдельного дублирующего запуска на push этой ветки нет. Старые запуски одного PR отменяются при новом commit.

| Job | Платформы | Содержание |
| --- | --- | --- |
| Lint | Только Linux | Rustfmt, Clippy с запретом предупреждений, Rust AST-линтер, Cargo/API boundary checker, Ruff, OpenSpec |
| Build & unit tests | Linux, macOS, Windows | Сборка workspace; unit tests библиотек и бинарников без SQLite и CLI integration suite |
| Integration tests | Linux, macOS, Windows | SQLite adapter suite; атомарная регистрация со связями, link, списки/просмотр, зависимые флаги -twc, несколько топиков, конкурентные wait/complete, падение внешнего вызывающего процесса и demo |
| CI passed | Итог всех категорий | Проходит только при success Lint и обеих матриц сборки/тестов |

`CI passed` использует `always()` и проверяет конкретные результаты зависимостей. Failure, cancelled и skipped не дают успешного gate. Матрицы используют `fail-fast: false`, чтобы ошибка одной ОС не скрывала результаты остальных. Только этот стабильный итоговый статус связан с правилом branch protection; изменение имени требует одновременного изменения защиты.

Default branch `main` защищена на GitHub. Точное тело настройки хранится в [.github/branch-protection.json](../.github/branch-protection.json):

- Изменения только через PR, включая администраторов.
- Обязателен `CI passed` от GitHub Actions app 15368, подтверждённого check runs этого репозитория.
- Ветка PR должна быть актуальна относительно `main` (`strict: true`).
- Force push и удаление запрещены.
- Требуются разрешённые обсуждения и линейная история; squash/rebase доступны.
- Обязательное число сторонних approvals — 0: самостоятельный автор может влить свой проверенный PR.

Защита применена через API и проверяется фактическим GET, а не наличием JSON в репозитории. Для повторного применения с административными правами:

```bash
gh api --method PUT repos/itzephir/anyq/branches/main/protection --input .github/branch-protection.json
gh api repos/itzephir/anyq/branches/main/protection
```

Если default branch меняется, правило нужно перенести на новую default branch и обновить push filter workflow. Workflow не содержит административных токенов и сам не изменяет свою защиту.
