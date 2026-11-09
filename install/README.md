# Установка готового anyq

Скрипты используют GitHub Releases itzephir/anyq. Первый полный релиз должен
быть опубликован до проверки публичной установки.

macOS/Linux:

```sh
curl -fsSL https://anyq.itzephir.com/install.sh -o install-anyq && chmod +x install-anyq && ./install-anyq
./install-anyq --user --version v0.1.0 --bin-dir "$HOME/bin"
```

Windows x64 (PowerShell 5.1+):

```powershell
curl.exe -fL https://anyq.itzephir.com/install.ps1 -o install-anyq.ps1; if ($?) { powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 }
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 -Scope User -Version v0.1.0 -BinDir "$env:USERPROFILE\bin"
```

ExecutionPolicy действует только в новом процессе; постоянная политика Windows
не изменяется. Можно сначала прочитать скачанный скрипт, затем запустить его.
Поддерживаются macOS Intel/Apple Silicon, Linux x86_64/ARM64 (musl static) и
Windows x64 MSVC со статическим C runtime, без WSL и отдельной установки VC++.
Нет Rust, C toolchain, Python или Node в runtime.

Архив и SHA256SUMS берутся из одной версии. До замены бинарника проверяются
SHA-256, распаковка и --version. Ошибка сохраняет старый файл. Запущенный Windows
бинарник может блокировать замену: завершите его и повторите установку.

По умолчанию установка системная, для всех пользователей:

| ОС | Каталог | Права и PATH |
| --- | --- | --- |
| macOS/Linux | /usr/local/bin | sudo запрашивается только при отсутствии прав записи; shell config не меняется |
| Windows | %ProgramFiles%\anyq | При необходимости появляется UAC; обновляется системный PATH |

Скачивание, checksum и запуск --version происходят до повышения прав. Только
размещение и системный PATH выполняются через sudo/UAC. Копия проверяется повторно
перед атомарной заменой. Отказ в повышении прав сохраняет предыдущую версию.
Ошибка записи Windows PATH откатывает замену бинарника.

Для установки одному пользователю без повышения прав:

```sh
./install-anyq --user
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 -Scope User
```

--user выбирает ~/.local/bin; -Scope User — %LOCALAPPDATA%\Programs\anyq и
пользовательский PATH. --system / -Scope System явно выбирают системный режим.
--bin-dir / -BinDir переопределяют каталог выбранного режима; ANYQ_BIN_DIR —
дополнительный Unix default. Пользовательский режим не вызывает sudo/UAC даже
для недоступного каталога: в таком случае установка завершается ошибкой.

Unix сообщает, если каталог отсутствует в PATH. Windows PATH дополняется без
дубликатов в контексте режима; после установки перезапустите терминал.
-NoPathUpdate отключает изменение PATH. Очереди, настройки и история сохраняются.
Удаление: удалить anyq/anyq.exe и при необходимости добавленный каталог из PATH.

## Выпуск

После review/merge commit с версией Cargo.toml:

```sh
git tag v0.1.0 MAIN_COMMIT
git push origin v0.1.0
```

Workflow отклоняет tag вне main и несовпадающую Cargo-версию. Пять нативных
сборок проверяют настоящую установку latest/pin и ошибки загрузки/checksum.
Полный набор загружается в draft до публикации. Повтор продолжает draft;
опубликованный релиз не перезаписывается. Apple/Windows signing не настроен.
SHA-256 проверяет целостность и не заменяет независимую подпись издателя.

```sh
python3 tooling/package_release.py --target aarch64-apple-darwin
python3 tooling/test_installers.py --target aarch64-apple-darwin
```

CI добавляет --system к тестам на одноразовых runners: проверяет настоящий
системный каталог, обновление и PATH, затем удаляет только свой бинарник и
восстанавливает PATH. Локальные тесты используют временные пользовательские пути;
они также проверяют отказ sudo, передачу аргументов UAC и откат Windows PATH.

ANYQ_RELEASE_BASE_URL (Unix) / -ReleaseBaseUrl (Windows) задают доверенное зеркало
для тестов. HTTP допускается только на loopback. Источник checksums и архивов
остаётся доверенным источником выпуска.
