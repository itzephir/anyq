# platform-support Specification

## Purpose

Пользователь может запускать одну и ту же локальную очередь нативно на macOS, Windows и Linux, а CI проверяет реальные процессы и долговечность на каждой платформе.

## Requirements

### Requirement: Путь базы Windows
Без --db, ANYQ_DB и db.path в config CLI Windows SHALL использовать LOCALAPPDATA/anyq/queue.db. Явные пути SHALL иметь приоритет над стандартным расположением.

#### Scenario: Локальная база
- **WHEN** Windows CLI запускается с LOCALAPPDATA и без пути базы в параметрах, окружении или config
- **THEN** база создаётся в подкаталоге anyq внутри LOCALAPPDATA

### Requirement: Матрица CI
CI SHALL проверять форматирование, архитектуру, OpenSpec и lint на Linux, а сборку, доменные и сквозные тесты и демонстрацию — нативно на macOS, Windows и Linux. Ошибка одной платформы SHALL не отменять проверку остальных.

#### Scenario: Изменение реализации
- **WHEN** отправлен commit или открыт PR
- **THEN** lint проходит на Linux, а сборка и тесты — на всех трёх нативных runner

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

### Requirement: Прерывание заблокированного вывода
Первый Ctrl+C SHALL завершать CLI с кодом 130 даже при заполненном stdout или stderr без чтения потребителем. Отмена вывода SHALL не менять сохранённые состояния, владение и настройки. Native integration tests SHALL воспроизводить реальное заполнение pipe и посылать один сигнал на каждой ОС.

#### Scenario: Потребитель перестал читать
- **WHEN** CLI заблокирован на записи в полный stdout или stderr и получает первый Ctrl+C
- **THEN** процесс завершается с 130 без второго сигнала и без ожидания чтения pipe

#### Scenario: Вывод прочитан полностью
- **WHEN** потребитель читает stdout без прерывания
- **THEN** CLI возвращает полный результат, сохраняя обычный код завершения
