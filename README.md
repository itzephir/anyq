# anyq

Локальная долговечная очередь на Rust для тяжёлых сборок, агентов, людей и эмуляторов. Без сервера: процессы используют один SQLite-файл на локальном диске.

Топик содержит сохранённый поток сообщений. Каждая **consumer group** независимо получает все сообщения; consumers внутри группы делят задания. Подтверждение в одной группе не влияет на другую. Новая группа читает историю с начала.

## Установка

Нужны Rust 1.85+ и инструменты C-сборки для встроенного SQLite. Поддерживаются macOS, Linux и Windows (MSVC; нужны Visual Studio Build Tools с C++).

```bash
cargo install --path apps/cli --locked
anyq --help
```

По умолчанию база на Unix — `$XDG_DATA_HOME/anyq/queue.db` или `~/.local/share/anyq/queue.db`, на Windows — `%LOCALAPPDATA%\anyq\queue.db`. `ANYQ_DB` и `--db` переопределяют путь. Все участники очереди должны указывать один файл. Отдельного фонового брокера нет.

## Сборка → эмулятор

```bash
anyq topic create large-builds
anyq topic create emulator-use
anyq group create large-builds builders
anyq group create emulator-use emulator-tests

anyq publish large-builds \
  --payload '{"project":"product","revision":"abc123"}' \
  --key product-abc123
```

В каталоге продукта запускаем worker с нужной командой. Payload приходит на stdin; команда выбирается явно и не извлекается из сообщения. Следующий payload задаётся отдельно; для динамического пути к артефакту удобен ручной ack из агента.

```bash
anyq worker large-builds --group builders --consumer build-host \
  --lease-ms 300000 \
  --next-topic emulator-use \
  --next-payload '{"artifact":"/absolute/path/product.apk"}' \
  -- bash ./scripts/build.sh

# В другом терминале, в каталоге тестового проекта:
anyq worker emulator-use --group emulator-tests --consumer emulator-1 \
  -- bash ./scripts/test-on-emulator.sh
```

Успешный exit code 0 подтверждает доставку и атомарно публикует следующее сообщение. Неуспех возвращает доставку для повтора; следующий этап не публикуется. `--once` обрабатывает максимум одно задание и завершает worker. Без `--once` он ждёт новые задания. Команды наследуют текущий каталог и окружение worker.

Полностью запускаемый пример с имитацией сборки и эмулятора:

```bash
cargo build --locked -p anyq
python examples/demo.py
```

## Агент или человек как consumer

```bash
anyq claim large-builds --group builders --consumer agent-1 --lease-ms 300000
```

Ответ содержит `message.id`, payload и `delivery.token`. Сохраните ID и token. Для долгой ручной работы продлевайте владение до истечения срока:

```bash
anyq renew 1 --group builders --token "$TOKEN" --lease-ms 300000

# После реального завершения работы:
anyq ack 1 --group builders --token "$TOKEN" \
  --result '{"artifact":"/absolute/path/product.apk"}' \
  --next-topic emulator-use \
  --next-payload '{"artifact":"/absolute/path/product.apk"}'

# Или вернуть задание с ошибкой и задержкой:
anyq nack 1 --group builders --token "$TOKEN" \
  --reason 'emulator unavailable' --delay-ms 5000
```

Здесь `1` — пример ID из ответа claim, `$TOKEN` — его token. Token относится только к конкретной обработке одной группы. После истечения или повторной выдачи старый token отклоняется.

## Группы, параллелизм и повторы

```bash
# Независимая обработка того же потока:
anyq group create large-builds audit
anyq claim large-builds --group audit --consumer reviewer

# Общий лимит выполняемых доставок топика, включая все группы:
anyq topic capacity large-builds --capacity 2

anyq jobs list --topic large-builds --group builders
anyq jobs show 1
anyq retry 1 --group builders
```

По умолчанию топик допускает **одно активное владение суммарно по всем группам**. Группы получают независимые копии потока, но делят физический ресурс. Для отдельных эмуляторов создайте отдельные топики. Готовые сообщения выдаются по порядку публикации; отложенный повтор пропускается до срока готовности. При параллелизме больше одного порядок завершения не гарантируется. Справедливое распределение слотов между непрерывно активными группами не гарантируется.

По умолчанию разрешено 3 попытки на группу; `publish --max-attempts N` меняет бюджет. Истёкший lease или nack исчерпавшей попытки переводит доставку в `failed`. `retry` разрешён только для `failed` и сбрасывает счётчик попыток. Повторная публикация с тем же `--key` в том же топике возвращает исходное сообщение, включая исходный payload. Публикация без групп тоже сохраняется и будет прочитана будущей группой.

`jobs list` выдаёт до 100 сообщений с обработками групп. Для следующей страницы передайте `--after ID`; `--limit` допускает 1..1000. `publish --file request.json` читает файл, `--file -` — stdin; payload ограничен 1 MiB. Большие артефакты храните отдельно, передавайте пути.

## Протокол worker

Команда получает JSON на stdin и переменные `ANYQ_MESSAGE_ID`, `ANYQ_TOPIC`, `ANYQ_GROUP`, `ANYQ_CONSUMER`, `ANYQ_ATTEMPT`. Lease token остаётся у worker. stdout/stderr команды направлены в stderr worker; stdout CLI содержит JSON, по одной записи на строку. Пустой claim возвращает `null`.

Коды выхода: `0` — успех или отсутствие работы, `1` — неуспех команды в режиме once, `2` — неверные аргументы/ошибка очереди, `130` — остановка worker сигналом SIGINT/SIGTERM или консольным прерыванием Windows. Ошибки очереди — JSON в stderr. `--poll-ms` задаёт ожидание пустой очереди, `--retry-delay-ms` — задержку повторной попытки. Worker продлевает lease примерно каждые `min(lease / 3, 1 сек)`; минимальный lease worker — 300 ms.

На macOS/Linux SIGINT/SIGTERM останавливает группу процессов команды и возвращает доставку. На Windows Ctrl+C/Ctrl+Break останавливает дерево команды через Job Object; bash/WSL не нужны. При обнаруженной потере lease worker также останавливает группу процессов и не подтверждает результат. Завершившаяся команда не должна оставлять фоновые процессы: worker завершает оставшихся потомков в её process group.

## Гарантии и ограничения

- Публикация, group replay, claim и ack со следующим этапом атомарны между локальными процессами. Хранилище — SQLite WAL, synchronous FULL, ожидание блокировки до 5 секунд.
- Доставка — **at least once**. Команды должны учитывать повтор внешнего действия после сбоя. Если два независимых group consumers оба публикуют следующий этап, будут два сообщения; общий `--next-key` позволяет намеренно дедуплицировать их.
- Lease защищает состояние очереди. На Unix SIGKILL, остановка ОС или скачок системного времени могут оставить внешнюю команду живой до повторной выдачи. На Windows закрытие job handle при завершении worker убивает управляемых потомков. Для физически эксклюзивного эмулятора при таком сбое нужна проверка оставшегося процесса. Exactly-once и аппаратное fencing не заявляются.
- История сообщений и конечные состояния сохраняются без автоматической очистки. Просмотр/захват/ручной retry восстанавливает истёкшие leases; фонового reaper нет. Это поток с независимыми группами, без Kafka-протокола, partitions и offset API.
- База предназначена для локального диска, не NFS. Для резервного копирования остановите workers и используйте SQLite `.backup`; простое копирование одного файла при активном WAL недостаточно. Неизвестная версия схемы отклоняется.

## Разработка

Правила кода — [docs/CODE-RULES.md](docs/CODE-RULES.md), архитектура и владельцы правил — [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Работа ведётся через [OpenSpec change](openspec/changes/build-local-queue/): proposal, specs, design, tasks. `api`/`impl` — отдельные Cargo crates, домен не зависит от SQLite/CLI/процессов.

```bash
npm ci --ignore-scripts
npm run spec:check
python tooling/check_architecture.py
cargo run --locked -p anyq-architecture-lint
python -m pip install -r tooling/requirements-dev.txt
python -m ruff check .
python -m ruff format --check .
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Node нужен только для закреплённого OpenSpec CLI 1.16.0. Python 3.11+ нужен для архитектурной проверки. CI выполняет эти проверки на Linux, macOS и Windows. Тесты запускают команды-имитации; реальные тяжёлые продукты и конкретные эмуляторы подключаются пользовательскими командами.

Нативный Windows worker можно запускать из PowerShell с явно выбранным интерпретатором:

```powershell
anyq worker large-builds --group builders --consumer windows-build -- powershell.exe -NoProfile -File C:\project\build.ps1
```

Контракты сценариев очереди типизированы отдельно; универсального `execute(Command) -> Output` нет. Правило одного самостоятельного типа на файл проверяет Rust AST-линтер, а остальные границы — Cargo и архитектурная проверка.
