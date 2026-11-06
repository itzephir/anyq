# Tasks

## 1. Доменные границы

- [x] 1.1 Создать Cargo workspace с фичами queue/worker и отдельными api/impl; закрепить правила в AGENTS.md и docs/ARCHITECTURE.md; проверить граф и отрицательные fixtures архитектурным checker.
- [x] 1.2 Реализовать чистые модели очереди, leases, retries и интеракторы через подставляемые Store/Clock/Tokens; проверить доменные сценарии без SQLite.

## 2. Долговечность

- [x] 2.1 Реализовать SQLite-порт и фабрику DI; проверить независимые группы, позднюю подписку, дедупликацию, rollback handoff, expiry/fencing и конкурентный захват реальными соединениями.

## 3. Использование

- [x] 3.1 Реализовать worker через queue api и порты процесса/ожидания; unit-тестами проверить heartbeat, потерю lease и неуспех без настоящего процесса.
- [x] 3.2 Реализовать CLI, JSON-презентеры и POSIX process adapter; документировать и проверить ручной/автоматический build → emulator сценарий, stdin и exit codes.

## 4. Интеграция

- [ ] 4.1 Пройти fmt, clippy, cargo tests, architecture checker, строгую OpenSpec-валидацию и сквозные проверки; создать приватный GitHub, отправить bootstrap и проверить CI.

## Workflow follow-up

- После реализации и проверки архивировать change средствами OpenSpec; незавершённые требования не переносить в базовые specs.
