# Design

## Context

CI уже зелёный на macOS/Linux. Только worker process adapter импортирует std::os::unix и POSIX signals; CLI-тесты и demo используют sh. Область изменения описана в proposal; домен и транзакции очереди остаются прежними.

## Goals / Non-Goals

**Goals:** нативные Windows MSVC build/test без bash/WSL, одинаковые сценарии очереди, остановка дерева процессов с проверкой на runner Windows.

**Non-Goals:** Windows Service, распределённая база, installer и релизные бинарники, поддержка фоновых процессов вне управляемой группы.

## Decisions

- Разделить process adapter на общий stdin/payload/heartbeat bridge и cfg(unix)/cfg(windows) детали. Существующие consumer-owned порты и DI сохраняются; SDK не попадают во внутренние crates.
- На Windows использовать Job Object с KILL_ON_JOB_CLOSE. Создавать команду с CREATE_SUSPENDED, назначать в job и только затем возобновлять её начальный поток через ToolHelp/OpenThread/ResumeThread. Так команда не успевает породить потомка вне job. При любой ошибке настройки убивать и дожидаться child; OwnedHandle закрывает Win32 handles. Обычный child.kill и taskkill после запуска не обеспечивают эту границу.
- Windows console cancellation через ctrlc, локальный AtomicBool и существующий Wait port. Unix сохраняет signal-hook. windows-sys/ctrlc объявляются только для Windows; libc/signal-hook — только для Unix. Минимальная версия Rust 1.85 сохраняется.
- Нативные process fixtures — дочерний запуск integration-test executable в специальном режиме. Проверки echo, exit code, heartbeat и cleanup не зависят от shell. Отдельный Windows-тест создаёт собственную консоль worker и отправляет Ctrl+C из присоединённого helper, не затрагивая консоль остальных тестов.
- Unix cleanup assertion читает состояние процесса через ps: zombie уже завершён и не считается работающим. Отсутствующий PID также завершён; ошибка запуска или выполнения ps не трактуется как успешный cleanup. Регрессионный тест сохраняет завершённого child без wait() до assertion и затем явно reap-ит его.
- Переносимый demo на Python 3, уже нужном разработчикам для architecture check. Для самого runtime Python не требуется. CI использует python с setup-python; матрица Ubuntu/macOS/Windows, fail-fast false, ограничение времени job.
- Стандартный Windows-путь — LOCALAPPDATA/anyq/queue.db, Unix сохраняет XDG/HOME. Архитектурный checker нормализует пути, чтобы Windows-разделители не меняли сравнение main.rs.

## Risks / Trade-offs

- [Поведение Win32 проверяется только на удалённом runner] → нативные интеграционные тесты, завершение tree и console cancellation; не объявлять поддержку проверенной до зелёного Windows CI.
- [Job assignment запрещён внешним sandbox/job policy] → отказ запуска с cleanup, без выполнения команды вне job.
- [Принудительное завершение Windows] → ОС закрывает job handle и завершает управляемых потомков; lease всё равно восстанавливается по сроку. На Unix исходное ограничение SIGKILL сохраняется.
