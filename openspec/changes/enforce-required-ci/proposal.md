# Proposal

## Why

Один job объединяет линтеры, сборку и интеграции, а default branch пока не защищена. Пользователь запросил отдельные задачи CI и запрет слияния до их успешного завершения.

## What Changes

- Разделить CI на lint, build-and-unit-tests и integration-tests с нативной матрицей macOS/Windows/Linux.
- Добавить итоговый CI passed, который требует успех всех категорий, включая ошибку/отмену/пропуск.
- Защитить default branch: PR, обязательный свежий CI passed, применение к администраторам, запрет force push/удаления и разрешение обсуждений.
- Сохранить конфигурацию защиты в репозитории и проверить фактические настройки GitHub API.

## Capabilities

### New Capabilities

- `repository-delivery`: категории проверок CI и защита default branch от обхода CI.

### Modified Capabilities

Нет базовых specs; платформенные проверки уже описаны в завершённом support-windows-ci и сохраняются.

## Impact

GitHub Actions workflow, конфигурация branch protection и настройки GitHub. Runtime очереди и данные не меняются.
