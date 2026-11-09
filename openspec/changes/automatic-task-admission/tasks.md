# Tasks

## 1. Домен и сценарии

- [x] 1.1 Реализовать автоматический допуск, пакетные typed-сценарии link и наблюдающий CheckTaskRunning; проверить domain/dispatch tests, включая несколько ресурсов, force, capacity и rollback; обновить архитектурное описание.

- [x] 1.2 Добавить доменные defaults/приоритеты autostart, постоянный stop и явный StartTask; проверить domain/dispatch/config tests, rollback и сохранение настроек.

## 2. SQLite и CLI

- [x] 2.1 Поддержать фильтр всех queued в SQLite и собрать наблюдающий wait в CLI; проверить SQLite/CLI tests на автономное продвижение, обе пакетные формы link с rollback, конкуренцию и повторный wait; обновить README и справку.

- [x] 2.2 Добавить config task.autostart, topic autostart, create --start/--stop/-s/-tswc и start -w/-c; проверить CLI-матрицу и миграцию SQLite 2 → 3, обновить README/справку.

## 3. Границы сохранения

- [x] 3.1 Обновить native Ctrl+C tests и docs/INTERRUPTIONS.md для атомарного продвижения при регистрации/link/complete/capacity/start/autostart и обновлении схемы; проверить suite с integration-test-hooks.

## 4. Итоговая проверка

- [ ] 4.1 Выполнить все обязательные локальные проверки из AGENTS.md, создать PR и дождаться результатов штатного CI Linux/macOS/Windows.

## Workflow follow-up

- Синхронизировать базовые specs и архивировать изменение после review и разрешённого слияния.
