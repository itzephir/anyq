# Архитектура anyq

Применяется классическая Clean Architecture Роберта Мартина с организацией по предметным фичам и реальными границами Cargo crates `api`/`impl`. Основа соответствует правилам Ephir в itzmoney и itzdesign; специфические денежные и графические модели сюда не переносятся.

| Владелец | Правила | Внутренние модули |
| --- | --- | --- |
| queue | Топик и его вместимость, независимость consumer groups, неизменность сообщений, доставка/попытки, срок владения и fencing, атомарный handoff | api → entities/контракт Queue; impl/dispatch → интеракторы и порты; impl/sqlite → Records/Store adapter; impl/composition → фабрика |
| worker | Получение задания, heartbeat, интерпретация результата процесса, ack/nack и следующий этап | api → настройки/контракт; impl/execution → интерактор и порты; impl/process → POSIX adapter; impl/composition → фабрика |

Внутренние crates используют std для коллекций, типов и вычислений, без IO. Публичные entities и чистые инварианты живут в api. Внутренние порты Records/Store/Clock/Tokens принадлежат dispatch; Process/Runner/Wait — execution. SQLite и POSIX не входят в эти контракты. Controllers и presenters находятся в apps/cli и только преобразуют внешний JSON/аргументы в доменные команды и обратно.

Межфичевая связь worker → queue идёт только через queue-api. SQL получает конкретные фильтры интерактора, не решает самостоятельно, что выдавать consumer, когда истекает lease и сколько попыток разрешено. Транзакционный порт гарантирует all-or-nothing; интерактор задаёт действия внутри транзакции. Domain policy не переносится в SQL ради атомарности.

Composition roots перечислены явно: features/queue/impl/composition, features/worker/impl/composition и apps/cli/src/main.rs. Первые две собирают только свою фичу; main связывает обе фабрики через API. Ни контроллеры, ни бизнес-сценарии не импортируют чужие impl. Реализации передаются явно, глобальных контейнеров нет.

tooling/check_architecture.py проверяет точный разрешённый граф всех Cargo crates, отсутствие скрытых зависимостей и IO/включения внешних файлов во внутренних слоях, импорты фабрик только из main. Checker имеет отрицательные fixtures. Cargo сам обеспечивает отсутствие незаявленных crate imports, приватность модулей и ацикличность графа. Unit tests проверяют домен и сценарии с doubles; SQLite и CLI интеграции проверяются отдельно.
