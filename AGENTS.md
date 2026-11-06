# Работа над anyq

- Проект — локальная очередь на Rust; спецификация и задачи в OpenSpec. Пользователь выбрал Kafka-подобные consumer groups: каждая группа независимо обрабатывает весь поток, consumers внутри группы конкурируют.
- Применять [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): Clean Architecture Роберта Мартина, сначала доменные фичи, затем слои. Каждая фича имеет отдельные Cargo crates api/impl; межфичевые зависимости только на api.
- Домен владеет инвариантами и смыслом фильтров; интерактор передаёт конкретный фильтр порту. SQLite/процессные адаптеры выполняют IO и преобразования без предметных решений.
- Внутренние crates не используют IO/SDK/SQL/CLI. Порты принадлежат потребителю, реализации явно передаются через constructors/factories. Только перечисленные composition roots собирают impl. Service locator запрещён.
- Перед коммитом: cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace, python3 tooling/check_architecture.py, npm run spec:check, git diff --check.
- Команды gh с сетью/авторизацией/Keychain выполнять вне sandbox. Не просить повторную авторизацию на основании ошибки внутри sandbox.
- Первичный bootstrap допускается в main; следующие изменения через codex/* и PR, без merge по собственной инициативе.
- Не отмечать непроверенные tasks завершёнными и не переносить незавершённое поведение в базовые specs. Пользовательский запрос на реализацию разрешает полный workflow OpenSpec, включая инициализацию в новом проекте.
- Не добавлять служебные даты обновления документов. Для будущего UI читать /Users/ephir/.codex/design/personal-projects.md.
