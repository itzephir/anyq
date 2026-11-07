# anyq

Локальная очередь допуска к ресурсам для именованных задач: сборок, агентов, людей и эмуляторов. Реализация на Rust, без сервера. Все участники используют один SQLite-файл на локальном диске.

Задача может требовать несколько топиков одновременно. Успешный wait атомарно занимает по одному слоту каждого топика. После этого вызывающая сторона выполняет свою работу и завершает задачу через complete.

## Установка

Нужны Rust 1.85+ и инструменты C-сборки для встроенного SQLite. Поддерживаются Linux, macOS и Windows MSVC с Visual Studio Build Tools C++.

```bash
cargo install --path apps/cli --locked
anyq --help
```

База по умолчанию: $XDG_DATA_HOME/anyq/queue.db или ~/.local/share/anyq/queue.db на Unix; %LOCALAPPDATA%\anyq\queue.db на Windows. ANYQ_DB и --db переопределяют путь.

**Версия 0.2 использует схему 2.** База прежних сообщений схемы 1 отклоняется без преобразования и удаления данных. Выберите отдельный файл через --db или ANYQ_DB; старый бинарник и старую базу можно сохранить для просмотра истории.

## Регистрация и допуск

Для новой базы в текущем терминале:

```bash
export ANYQ_DB="$HOME/.local/share/anyq/tasks-v2.db"

anyq topic create itzdesign-commands
anyq topic create itzdesign-repo-lock
anyq topic create pixel-10-emulator
anyq topic --list

anyq task add build-revision-001
anyq topic itzdesign-commands add build-revision-001
anyq topic itzdesign-repo-lock add build-revision-001
anyq task wait build-revision-001
```

После успешного wait выполняется ваша команда в вашем окружении:

```bash
npm run build
anyq task complete build-revision-001
```

На Windows путь можно задать в PowerShell:

```powershell
$env:ANYQ_DB = "$env:LOCALAPPDATA\anyq\tasks-v2.db"
anyq topic create itzdesign-commands
anyq task add windows-build-001
anyq topic itzdesign-commands add windows-build-001
anyq task wait windows-build-001
```

Код 0 у wait означает, что допуск уже сохранён. Сообщение waiting появляется, если задача ещё ждёт. Вызывающая сторона сама решает, как обрабатывать exit code внешней команды и когда безопасно вызвать complete: anyq не запускает её, не определяет её результат и не останавливает её процессы.

## Состояния и ошибки

| Состояние | Значение |
| --- | --- |
| registered | Имя зарегистрировано, топиков пока нет |
| queued | Есть топики, допуск ещё не получен |
| running | Занято по одному слоту всех топиков |
| completed | Владение завершено, запись остаётся в истории |

Имя уникально в пределах базы и не переиспользуется после завершения. Новое выполнение регистрируется с новым именем. Повторное добавление той же связи queued-задачи идемпотентно. Running- и completed-задачи нельзя добавлять в другие топики.

```bash
anyq task add unfinished
anyq task wait unfinished
# error: 'unfinished' is not registered in any topic

anyq task complete unfinished
# error: 'unfinished' was not run

anyq task complete --force unfinished
# 'unfinished' successfully completed
```

Обычный complete разрешён только для running. Force также завершает registered/queued без выполнения работы, снимая задачу с очередей. Отсутствующее имя и повторное завершение дают ошибку. Wait для уже running/completed тоже даёт ошибку; два одновременных wait одного имени не получают два допуска.

Успех CLI возвращает код 0, ошибки входа и состояния — 2 с сообщением в stderr. Операции регистрации и завершения печатают понятные текстовые ответы.

## Вместимость и порядок

```bash
anyq topic create parallel-builds --capacity 2
anyq topic capacity parallel-builds --capacity 3
```

По умолчанию топик имеет один слот. Running-задача занимает один слот каждого своего топика. Уменьшение capacity ниже текущего использования отклоняется.

Во всех топиках используется единый номер регистрации задачи. Задача допускается, когда в каждом топике она входит в первые queued-задачи по числу свободных слотов. Порядок добавления связей не создаёт противоположных очередей одной пары задач.

Queued-задача сохраняет место даже без запущенного wait. Она может задерживать более поздние задачи, пока ждёт другой ресурс. Ненужную задачу можно снять через complete --force. Частично захваченных ресурсов у queued-задачи нет.

## Постоянное владение

Выход wait, закрытие терминала, падение вызывающего процесса и прошедшее время не освобождают слоты. Владение сохраняется до явного complete. Heartbeat и автоматического таймаута нет.

Если внешняя работа аварийно завершилась, сначала проверьте оставшиеся процессы и ресурс, затем завершите известное имя задачи. Complete фиксирует окончание владения, а не проверяет фактическое состояние эмулятора или файловой системы. Это кооперативная координация локальных процессов.

История сохраняется без автоматической очистки. База предназначена для локального диска. SQLite использует WAL, synchronous FULL и короткие IMMEDIATE-транзакции; резервную копию создавайте SQLite backup, а не копированием одного файла при активном WAL.

## Проверки и разработка

```bash
cargo build --locked -p anyq
python examples/demo.py

npm ci --ignore-scripts
npm run spec:check
python -m pip install -r tooling/requirements-dev.txt
python -m ruff check .
python -m ruff format --check .
python tooling/check_architecture.py
cargo run --locked -p anyq-architecture-lint
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Python нужен для tooling и демонстрации внешнего клиента, Node — для закреплённого OpenSpec CLI. Сам runtime anyq их не требует.

Правила кода — [docs/CODE-RULES.md](docs/CODE-RULES.md), устройство модулей — [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), CI и защита main — [docs/CI.md](docs/CI.md). Базовые требования находятся в [OpenSpec specs](openspec/specs/), текущий переход — в [coordinate-named-tasks](openspec/changes/coordinate-named-tasks/), история — в [архиве](openspec/changes/archive/).
