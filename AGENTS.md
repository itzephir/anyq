# Работа над anyq

- Проект — локальная очередь на Rust; спецификация и задачи в OpenSpec. Пользователь выбрал Kafka-подобные consumer groups: каждая группа независимо обрабатывает весь поток, consumers внутри группы конкурируют.
- Применять [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): Clean Architecture Роберта Мартина, сначала доменные фичи, затем слои. Каждая фича имеет отдельные Cargo crates api/impl; межфичевые зависимости только на api.
- Домен владеет инвариантами и смыслом фильтров; интерактор передаёт конкретный фильтр порту. SQLite/процессные адаптеры выполняют IO и преобразования без предметных решений.
- Один самостоятельный тип — один файл: struct, enum и trait в Rust, включая DTO и test doubles. Связанные impl и локальные вложенные вспомогательные типы остаются с владельцем; lib.rs/mod.rs служат явными entry points.
- Соблюдать SRP, SOLID, DRY и KISS. Сценарии имеют отдельные типизированные входные контракты и интеракторы. Запрещены универсальные бизнес-API вида execute(Command) -> Output, god objects и зависимости на ненужные потребителю сценарии. Общий transaction boundary допустим для атомарности; он не заменяет контракты сценариев.
- Внутренние crates не используют IO/SDK/SQL/CLI. Порты принадлежат потребителю, реализации явно передаются через constructors/factories. Только перечисленные composition roots собирают impl. Service locator запрещён.
- Обязательные правила кода и примеры: [docs/CODE-RULES.md](docs/CODE-RULES.md). Применять при реализации и ревью; исправлять причину нарушения, не добавлять подавление линтера ради зелёного CI.
- Перед коммитом: cargo fmt --all --check; cargo clippy --workspace --all-targets --locked -- -D warnings; cargo test --workspace --locked; cargo run --locked -p anyq-architecture-lint; python tooling/check_architecture.py; python -m ruff check .; python -m ruff format --check .; npm run spec:check; git diff --check. Полная матрица CI: Linux/macOS/Windows.
- Команды gh с сетью/авторизацией/Keychain выполнять вне sandbox. Не просить повторную авторизацию на основании ошибки внутри sandbox.
- Первичный bootstrap допускается в main; следующие изменения через codex/* и PR, без merge по собственной инициативе.
- Default branch защищена: PR, актуальный CI passed от GitHub Actions, без force push/удаления и admin bypass. Конфигурация — .github/branch-protection.json, категории CI — docs/CI.md. Не отключать защиту ради слияния.
- Не отмечать непроверенные tasks завершёнными и не переносить незавершённое поведение в базовые specs. Пользовательский запрос на реализацию разрешает полный workflow OpenSpec, включая инициализацию в новом проекте.
- Не добавлять служебные даты обновления документов. Для будущего UI читать /Users/ephir/.codex/design/personal-projects.md.
