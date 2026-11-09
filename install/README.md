# Установка готового anyq

Скрипты используют GitHub Releases itzephir/anyq. Первый полный релиз должен
быть опубликован до проверки публичной установки.

macOS/Linux:

```sh
curl -fsSL https://anyq.itzephir.com/install.sh -o install-anyq && chmod +x install-anyq && ./install-anyq
./install-anyq --version v0.1.0 --bin-dir "$HOME/bin"
```

Windows x64 (PowerShell 5.1+):

```powershell
curl.exe -fL https://anyq.itzephir.com/install.ps1 -o install-anyq.ps1; if ($?) { powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 }
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 -Version v0.1.0 -BinDir "$env:USERPROFILE\bin"
```

ExecutionPolicy действует только в новом процессе; постоянная политика Windows
не изменяется. Можно сначала прочитать скачанный скрипт, затем запустить его.
Поддерживаются macOS Intel/Apple Silicon, Linux x86_64/ARM64 (musl static) и
Windows x64 MSVC со статическим C runtime, без WSL и отдельной установки VC++.
Нет Rust, C toolchain, Python или Node в runtime.

Архив и SHA256SUMS берутся из одной версии. До замены бинарника проверяются
SHA-256, распаковка и --version. Ошибка сохраняет старый файл. Запущенный Windows
бинарник может блокировать замену: завершите его и повторите установку.

Unix: default ~/.local/bin; скрипт не редактирует shell config, а сообщает,
если каталог отсутствует в PATH. Windows: LOCALAPPDATA/anyq/bin; пользовательский
PATH дополняется без дубликатов, изменение доступно в новом терминале.
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

ANYQ_RELEASE_BASE_URL (Unix) / -ReleaseBaseUrl (Windows) задают доверенное зеркало
для тестов. HTTP допускается только на loopback. Источник checksums и архивов
остаётся доверенным источником выпуска.
