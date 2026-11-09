# Spec Delta

## ADDED Requirements

### Requirement: Нативный CLI координации
Регистрация задач, добавление в топики, wait и complete SHALL выполняться нативно на macOS, Windows и Linux с одинаковыми состояниями, атомарностью и ошибками. Runtime SHALL не требовать Python, bash или WSL; запуск внешней работы SHALL оставаться обязанностью вызывающей стороны.

#### Scenario: Допуск на Windows
- **WHEN** Windows CLI получает допуск задачи к двум топикам и завершается
- **THEN** задача остаётся running в обоих топиках до отдельного вызова complete

### Requirement: Прерывание нативного CLI
Первый Ctrl+C SHALL возвращать 130. До commit SQLite SHALL откатывать текущую транзакцию, после commit SHALL сохранять результат и владение. Цепочки SHALL оставлять последнюю сохранённую стадию. Native integration tests SHALL посылать реальные SIGINT/CTRL_C_EVENT на всех трёх ОС без skip/ignore и проверять состояния, слоты и последующие операции.

#### Scenario: Сигнал после допуска
- **WHEN** Ctrl+C приходит после commit running до сообщения CLI
- **THEN** задача остаётся running и занимает все слоты до явного complete

#### Scenario: Сигнал до допуска
- **WHEN** Ctrl+C приходит до commit running или при ожидании ресурса
- **THEN** задача queued без частичного владения

## REMOVED Requirements

### Requirement: Нативный CLI и worker
**Reason**: CLI координации заменяет worker, запускающий команды с payload и идентичностью группы.
**Migration**: Использовать task create, topic link TOPIC NAME или task link NAME TOPIC, task wait и task complete на той же ОС; внешние команды запускать самостоятельно.

### Requirement: Завершение дерева процессов
**Reason**: anyq не запускает внешнюю работу и не владеет её деревом процессов. Падение вызывающей стороны не освобождает слоты автоматически.
**Migration**: Управление внешними процессами выполняет вызывающая сторона; после проверки оставшейся работы задачу завершают явно.
