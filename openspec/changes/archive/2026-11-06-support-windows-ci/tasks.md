# Tasks

## 1. Платформенный адаптер

- [x] 1.1 Выделить Unix и Windows детали worker, реализовать Job Object с безопасным suspended launch, cleanup и console cancellation; проверить компиляцию, нативные fixtures дерева процессов и прерывания на соответствующих ОС; обновить архитектурное описание.
- [x] 1.2 Добавить LOCALAPPDATA как стандартную базу Windows, проверить изолированным CLI-тестом и документировать приоритет явного пути.
- [x] 1.3 Исправить Unix cleanup assertion: завершённый, но ещё не reaped child считать завершённым, ошибки наблюдения не маскировать; проверить живой процесс и регрессионный сценарий zombie до wait().

## 2. Переносимые проверки

- [x] 2.1 Перевести общие CLI integration-тесты и demo на переносимые fixtures, сохранить проверки stdin, handoff, heartbeat и failure; выполнить cargo tests и demo, добавить Windows-пример в README.
- [x] 2.2 Добавить Windows в CI, нормализовать пути architecture checker, выполнить fmt/clippy/tests/OpenSpec/architecture и дождаться зелёной матрицы macOS/Windows/Linux в PR.

## Workflow follow-up

- После ревью и merge архивировать завершённый change. Не сливать PR без просьбы пользователя.
