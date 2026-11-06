# Design

## Context

Новый пустой репозиторий; запрос и границы — в proposal. Правила прочитаны в itzmoney/docs/architecture/FEATURES.md и itzdesign/docs/ARCHITECTURE.md. Пользователь выбрал независимые consumer groups, как в Kafka.

## Goals / Non-Goals

**Goals:** один локальный файл, атомарные операции разных процессов, понятный CLI для человека и агента, домен без IO.

**Non-Goals:** сетевой протокол Kafka, partitions, offsets API, строгий порядок завершения, exactly-once внешних команд, daemon и UI.

## Decisions

- SQLite WAL с IMMEDIATE transactions и busy timeout вместо отдельного сервера. База находится в локальной файловой системе. Сообщения неизменяемы; обработка хранится отдельно на пару message/group. Публикация создаёт доставки всем текущим группам; создание группы копирует сохранённый поток с начала в одной транзакции.
- Топик владеет общим лимитом активных leases всех групп (по умолчанию 1): это разделяемый ресурс машины. Группы независимо подтверждают свои копии, но разделяют ресурсный лимит. На каждый физический эмулятор удобно создать отдельный топик. Готовые доставки выдаются по message ID; retry с задержкой пропускается до срока готовности.
- At-least-once: время хранится в Unix milliseconds; UUID token меняется при каждом захвате. Домен проверяет статус, token и срок перед ack/nack/renew. Просроченные leases восстанавливаются перед изменениями/просмотром. Попытки принадлежат доставке группы. Failed можно явно повторить со сбросом попыток.
- Ack и необязательный follow-up — одна транзакция, без промежутка потери следующего этапа. Повторный ack отклоняется; повторять публикации producer может с idempotency key, уникальным в топике. Независимые ack разных групп намеренно могут публиковать разные следующие сообщения.
- Фича queue владеет Topic, ConsumerGroup, Message и Delivery: вместимость, порядок, дедупликация, retries, fencing, handoff. `api` содержит чистые модели/контракты; `impl/dispatch` — интеракторы и порты Records/Store/Clock/Tokens; `impl/sqlite` — только отображение запросов и транзакций; `impl/composition` — фабрика. Domain query filters задаются интерактором; SQL выполняет эти фильтры без самостоятельной политики.
- Фича worker: `api` — настройки и контракт Worker; `impl/execution` — получение, heartbeat, завершение, follow-up и порты Runner/Process/Wait; `impl/process` — POSIX command/process-group adapter; `impl/composition` — фабрика. Зависит только от queue api.
- `apps/cli` — явный composition root, контроллеры и JSON-презентеры. Только main связывает фабрики обеих фич. Cargo crates разделяют слои, Python architecture checker проверяет разрешённый граф, отрицательные fixtures и запрещённый IO во внутренних crates. Внутренние crates используют только std и публичные API.
- Worker читает JSON через stdin, передаёт ANYQ_MESSAGE_ID/ANYQ_GROUP/ANYQ_CONSUMER/ANYQ_ATTEMPT; команда задаётся аргументами после `--`. Child stdout/stderr направлены в stderr worker. Heartbeat делает interactor; процессный адаптер только запускает, опрашивает и останавливает process group. Непредвиденная гибель worker может оставить внешнюю команду; lease не является физическим fencing эмулятора.

## Risks / Trade-offs

- [Повтор внешнего действия после сбоя] → idempotent команды и producer keys; exactly-once не обещается.
- [Скачок системных часов / SIGKILL worker] → документировать ограничение, достаточный lease, ручная проверка ресурса. При обнаруженной потере lease worker убивает process group.
- [Рост локального потока] → история сохраняется; retention/compaction после v1. CLI использует ограниченные страницы, claim выбирает только нужные записи.
- [Ошибки следующего топика] → rollback ack; worker проверяет наличие топика до запуска команды.

## Migration Plan

Schema v1 создаётся при первом открытии; неизвестная версия отклоняется. Для резервной копии остановить workers и использовать SQLite backup. Выпуск — один бинарник через cargo install; Node требуется только для разработки OpenSpec.
