# Design

## Context

Текущий PR #1 уже прошёл единый CI job на macOS, Windows и Linux. Default branch — main, доступ пользователя ADMIN; защиты и rulesets пока нет. GitHub Actions app id 15368 подтверждён существующими check runs.

## Goals / Non-Goals

**Goals:** независимые категории проверок, стабильный обязательный статус и фактически применённая защита main.

**Non-Goals:** merge PR по собственной инициативе, обязательный сторонний reviewer для персонального проекта, релизы и автоматический deploy.

## Decisions

- Три matrix jobs: lint (Rustfmt, Clippy, AST, Cargo graph, Ruff, OpenSpec), build-and-unit-tests (cargo build workspace, cargo test --lib --bins с исключением SQLite adapter suite), integration-tests (SQLite adapter suite, CLI workflow suite и переносимый demo). Все категории запускаются на трёх ОС; результат каждой матрицы учитывается итоговым job.
- CI passed использует always(), needs всех трёх матриц и явную проверку каждого needs.*.result == success. Обычный пропуск зависимого job не подходит: GitHub считает skipped успешным для required checks.
- Workflow запускается на PR и push в main. Branch pushes с открытым PR проверяются событием synchronize, без дублирования push/PR runs. Concurrency отменяет устаревшие запуски одной ветки/PR.
- Classic branch protection для проверенной default branch main: CI passed только от GitHub Actions, strict up-to-date, enforce_admins, PR с нулём обязательных approvals, conversation resolution, запрет force/deletion и linear history. Squash/rebase должны оставаться доступны. Пользователь не ограничивается требованием чужого review своего личного проекта.
- Точное тело REST update хранится в .github/branch-protection.json; после применения GET проверяет фактические настройки. Защита сама по себе не является workflow, payload и документация не считаются доказательством её включения.

## Risks / Trade-offs

- [Тариф или права не позволяют private branch protection] → проверить реальную ошибку API, сохранить готовую конфигурацию и явно сообщить, что enforcement не включён; не менять видимость и оплату автоматически.
- [Изменение имени итогового job] → имя CI passed является стабильным контрактом с настройкой protection, менять вместе.
- [Владелец снимает защиту через настройки] → как администратор он управляет политикой; текущая политика запрещает обход при обычном merge/push.
