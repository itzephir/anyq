# Архитектура anyq

Применяется классическая Clean Architecture Роберта Мартина с организацией по предметным фичам и реальными границами Cargo crates `api`/`impl`. Основа соответствует правилам Ephir в itzmoney и itzdesign; специфические денежные и графические модели сюда не переносятся.

| Владелец | Правила | Внутренние модули |
| --- | --- | --- |
| queue | Топик и его вместимость, независимость consumer groups, неизменность сообщений, доставка/попытки, срок владения и fencing, атомарный handoff | api → entities и отдельные контракты сценариев; impl/dispatch → один interactor на сценарий и его порты; impl/sqlite → отдельные адаптеры записей и транзакции; impl/composition → фабрика |
| worker | Получение задания, heartbeat, интерпретация результата процесса, ack/nack и следующий этап | api → настройки/контракт; impl/execution → интерактор и порты; impl/process → POSIX adapter; impl/composition → фабрика |

Внутренние crates используют std для коллекций, типов и вычислений, без IO. Публичные entities и чистые инварианты живут в api. Порты TopicRecords/GroupRecords/MessageRecords/DeliveryRecords, Store/UnitOfWork/Clock/Tokens принадлежат dispatch; Process/Runner/Wait — execution. SQLite, POSIX и Win32 не входят в эти контракты. Controllers и presenters находятся в apps/cli и только преобразуют внешний JSON/аргументы в типизированный вход сценария и результат обратно в JSON.

У каждого сценария отдельный контракт и interactor: CreateTopic, SetTopicCapacity, ListTopics, GetTopic, CreateConsumerGroup, ListConsumerGroups, PublishMessage, ClaimDelivery, RenewLease, CompleteDelivery, RejectDelivery, RetryDelivery, InspectMessage, ListMessages. Универсальных Queue/Command/Output и execute-dispatcher нет. Worker получает только ClaimDelivery/RenewLease/CompleteDelivery/RejectDelivery/GetTopic. UnitOfWork объединяет границу атомарности и четыре persistence-порта; он не выполняет бизнес-маршрутизацию. Composition structures содержат явно созданные handles, без поведения и string lookup.

Межфичевая связь worker → queue идёт только через queue-api. SQL получает конкретные фильтры интерактора, не решает самостоятельно, что выдавать consumer, когда истекает lease и сколько попыток разрешено. Транзакционный порт гарантирует all-or-nothing; интерактор задаёт действия внутри транзакции. Domain policy не переносится в SQL ради атомарности.

Composition roots перечислены явно: features/queue/impl/composition, features/worker/impl/composition и apps/cli/src/main.rs. Первые две собирают только свою фичу; main связывает обе фабрики через API. Ни контроллеры, ни бизнес-сценарии не импортируют чужие impl. Реализации передаются явно, глобальных контейнеров нет.

Один самостоятельный struct/enum/trait — один файл, включая DTO и doubles. Подробные SRP/SOLID/DRY/KISS и правила ревью закреплены в [CODE-RULES.md](CODE-RULES.md). AST-линтер tooling/architecture-lint проверяет структуру реального Rust-синтаксиса и запрет универсального бизнес-dispatcher. tooling/check_architecture.py проверяет точный Cargo graph, отсутствие IO во внутренних слоях и factory imports только из main. Обе проверки имеют отрицательные fixtures. Cargo обеспечивает отсутствие незаявленных crate imports, приватность модулей и ацикличность графа. Unit tests проверяют домен и конкретные сценарии с узкими doubles; SQLite и нативные CLI/process integration проверяются отдельно.

Платформенные детали worker находятся в impl/process: общие CommandRunner/ManagedProcess/SystemWait и отдельные unix/windows ChildTree/Interrupt. Unix использует process groups и SIGINT/SIGTERM; Windows — Job Object, suspended launch перед назначением в job и консольное прерывание. Их порты и бизнес-сценарии одинаковы.
