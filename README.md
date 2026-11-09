# anyq

Локальная очередь допуска к ресурсам для именованных задач: сборок, агентов, людей и эмуляторов. Rust, без сервера; все участники используют один SQLite-файл на локальном диске.

Задача требует несколько топиков одновременно. Успешный wait атомарно занимает по одному слоту каждого топика. Вызывающая сторона выполняет работу и освобождает владение через complete.

## Установка

Rust 1.85+ и инструменты C-сборки для встроенного SQLite. Поддерживаются Linux, macOS и Windows MSVC с Visual Studio Build Tools C++.

```bash
cargo install --path apps/cli --locked
anyq --help
```

Приоритет базы: --db PATH > ANYQ_DB > db.path в config > системный default. Default: $XDG_DATA_HOME/anyq/queue.db или ~/.local/share/anyq/queue.db на Unix; %LOCALAPPDATA%\anyq\queue.db на Windows.

**Версия 0.2 использует схему 2.** База прежних сообщений схемы 1 отклоняется без изменения или удаления. Выберите отдельный файл через --db; старый бинарник и базу можно сохранить для истории.

## Команды

| Команда | Действие |
| --- | --- |
| `anyq topic list` / `topic --list` / `topic -l` | Список топиков |
| `anyq topic create TOPIC [-c N]` | Создать топик; -c = --capacity, default 1 |
| `anyq topic TOPIC` | Текущие running/queued по регистрации и capacity |
| `anyq topic capacity TOPIC [N]` | Прочитать вместимость или изменить на N |
| `anyq topic link TOPIC NAME [-w] [-c]` | Связать, затем опционально ждать/завершить |
| `anyq task list` / `task --list` / `task -l` | Все имена по алфавиту, включая completed |
| `anyq task NAME` | Состояние и сохранённые топики |
| `anyq task create NAME [-t TOPIC] [-w] [-c]` | Зарегистрировать с начальными связями и следующими шагами |
| `anyq task link NAME TOPIC [-w] [-c]` | Та же связь, что topic link |
| `anyq task wait NAME [-c]` | Получить полный допуск; -c сразу завершает |
| `anyq task complete NAME [-f]` | Завершить; -f = --force |
| `anyq config KEY [VALUE]` | Прочитать или записать настройку |
| `anyq config --get KEY` / `--unset KEY` | Прочитать или сбросить настройку |
| `anyq config list` / `--list` / `-l` | Действующие настройки |
| `anyq help [command]` | Справка, например help task link |

-v = --version, -h = --help, -w = --wait, -c = --complete у задач. --db PATH доступен на любом уровне. Task add заменён на task create; старый add не является командой.

Для всех действий есть name-first форма: topic link cpu job = topic cpu link job; task link job cpu = task job link cpu; topic cpu capacity [N]; topic cpu create [-c N]; task job create/wait/complete. Имена и глобальные параметры сохраняются при нормализации.

Create принимает повторяемый --topic/-t. Все начальные связи и задача сохраняются одной транзакцией. Wait требует хотя бы один topic; complete требует wait. У link complete тоже требует wait. Ошибки сочетаний отклоняются до открытия базы.

```bash
anyq task create build-001 -t itzdesign-commands -t itzdesign-repo-lock
# queued, без владения
anyq task complete build-001 -f

anyq task create build-002 -tw itzdesign-commands -t itzdesign-repo-lock
# running после допуска, до явного complete
npm run build
anyq task complete build-002

anyq task create probe-001 -twc pixel-10-emulator
# queued → running → completed
```

-twc TOPIC = --topic TOPIC --wait --complete. Шаги цепочки имеют отдельные commits. Ошибка прекращает последующие шаги. Complete не запускает внешнюю команду между допуском и завершением.

## Имена

У задач и топиков общая доменная схема. Bare-имя начинается с латинской a-z/A-Z; остальные символы произвольны, включая пробелы, Unicode и обратные слеши. Bare-слова list/create/capacity/link/add/wait/complete/config/help зарезервированы. Начальный дефис без обёртки запрещён.

Одна буквальная внешняя пара обратных кавычек разрешает любое содержимое: пустое, Unicode, слова команд, начальные дефисы и вложенные кавычки. Она снимается ровно один раз: build и `build` — одна сущность. Внутренние escapes не интерпретируются. Вывод даёт обратимое написание канонического имени.

Shell должен передать обратные кавычки как текст. Примеры bash/zsh:

```bash
anyq topic create '`list`'
anyq task create '`--задача`' -twc '`list`'
anyq task create build\ with\ spaces -t '`list`'
```

В PowerShell буквальную обёртку тоже можно передать в одинарных кавычках. Shell-кавычки и буквальные обратные кавычки — разные уровни. Разделитель -- сам по себе не обходит доменные ограничения.

## Настройки

Файл ~/.anyq/config.toml. На Unix home — $HOME, на Windows — %USERPROFILE%. Без файла оба autocreate выключены; чтение defaults не создаёт файл.

```bash
anyq config task.autocreate true
anyq config topic.autocreate true
anyq config db.path '~/queues/tasks.db'
anyq config list
anyq config --get task.autocreate
anyq config --unset db.path
```

task.autocreate и topic.autocreate принимают true/false, default false. db.path — непустая строка; ~/ раскрывается относительно home, относительный путь — относительно рабочего каталога. Unset возвращает default. Неизвестные ключи и неверные значения дают ошибку. list — подкоманда, ключом быть не может. Старые autoadd-ключи не поддерживаются.

Autocreate создаёт отсутствующий именованный объект внутри транзакции основной операции; новый топик capacity 1. Link может создать обе сущности и связь вместе. Успешный просмотр тоже может создать отсутствующую сущность. Ошибка откатывает автоматические создания: обычный task complete missing даёт was not run и не оставляет missing; --force может успешно создать и завершить его. Существующие состояния и правила имён сохраняются.

Запись config атомарна и защищена системным файловым lock. Конкурентные изменения перечитывают актуальные значения. Config/help/version не открывают базу; help/version работают при повреждённом config. Чтение отсутствующего db.path возвращает 1 без сообщения; остальные ошибки — 2.

Пустые разделы task/topic/db допустимы и используют defaults. Таблица вместо конечного bool/string, неизвестный раздел или list = {} дают ошибку; чтение и set/unset сохраняют исходные байты файла.

## Регистрация и допуск

```bash
export ANYQ_DB="$HOME/.local/share/anyq/tasks-v2.db"
anyq topic create itzdesign-commands
anyq topic create itzdesign-repo-lock
anyq topic create pixel-10-emulator
anyq topic list

anyq task create build-revision-001
anyq topic link itzdesign-commands build-revision-001
anyq task link build-revision-001 itzdesign-repo-lock
anyq task wait build-revision-001
npm run build
anyq task complete build-revision-001
anyq task build-revision-001
```

В PowerShell путь задаётся через $env:ANYQ_DB. Код 0 у wait означает, что running уже сохранён. Waiting появляется при занятых ресурсах. Вызывающая сторона решает, как обработать результат внешней команды и когда завершать владение.

## Состояния, порядок и ошибки

| Состояние | Значение |
| --- | --- |
| registered | Имя есть, топиков нет |
| queued | Есть топики без допуска |
| running | По одному слоту всех топиков |
| completed | Владение завершено, история сохраняется |

Имя уникально и не переиспользуется после complete. Повтор queued-связи идемпотентен. Running/completed не меняют членство. Обычный complete допустим только для running; force также завершает registered/queued. Повторный wait для running/completed и повторное завершение дают ошибку. Два wait одного имени не получают два допуска.

```bash
anyq task create unfinished
anyq task wait unfinished
# error: 'unfinished' is not registered in any topic
anyq task complete unfinished
# error: 'unfinished' was not run
anyq task complete --force unfinished
# 'unfinished' successfully completed
```

Capacity положительна, default 1; уменьшение ниже текущего использования отклоняется. Во всех топиках один номер регистрации задачи. Допуск возможен, когда она входит в первые queued-задачи по числу свободных слотов каждого топика. Частичного захвата нет. Queued без активного wait сохраняет место; ненужную задачу снимают через complete --force.

Просмотр не меняет существующее владение; completed сохраняет связи, но не входит в текущую очередь. Неизвестное имя даёт ошибку, если соответствующий autocreate выключен.

Успех CLI — 0, ошибки входа/состояния — 2 в stderr, первый Ctrl+C — 130. После commit состояние сохраняется даже при прерывании до успешного сообщения.

## Постоянное владение и прерывания

Выход wait, падение вызывающего процесса и время не освобождают слоты. Нет heartbeat и таймаута владения. После аварии проверьте внешнюю работу, затем явно завершите известную задачу.

Ctrl+C до commit откатывает текущую транзакцию; после commit сохраняет результат. В цепочке -twc это может быть queued/running/completed. При коде 130 проверьте task NAME: сохранённое running требует complete. Таблица исходов и native integration suite — [INTERRUPTIONS.md](docs/INTERRUPTIONS.md).

Первый Ctrl+C работает и при заполненном stdout/stderr, когда потребитель не читает вывод. При таком прерывании вывод может быть неполным; сохранённые состояния и владение не меняются.

История не очищается автоматически. База — на локальном диске; SQLite использует WAL, synchronous FULL, короткие IMMEDIATE-транзакции. Резервную копию активной базы создавайте SQLite backup.

## Проверки

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
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cargo test --locked -p anyq --test interruptions --features integration-test-hooks
```

Python нужен для tooling/демонстрации, Node — для закреплённого OpenSpec. Runtime anyq их не требует.

[Правила кода](docs/CODE-RULES.md), [архитектура](docs/ARCHITECTURE.md), [CI и защита main](docs/CI.md). [Базовые specs](openspec/specs/), [текущий change](openspec/changes/coordinate-named-tasks/), [архив](openspec/changes/archive/).
