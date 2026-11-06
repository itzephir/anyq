# Spec Delta

## Purpose

Изменения проходят отдельные проверяемые этапы CI, а default branch принимает их только через PR с успешным актуальным результатом всех обязательных проверок.

## ADDED Requirements

### Requirement: Отдельные категории CI
CI SHALL запускать отдельный lint job только на Linux, а build-and-unit-tests и integration-tests — нативно на Linux, macOS и Windows. Ошибка одной ОС SHALL не отменять остальные ОС.

#### Scenario: Один запуск линтеров
- **WHEN** создан или обновлён PR
- **THEN** lint запускается один раз на Linux, а сборка и тесты проверяются на трёх ОС

#### Scenario: Падение интеграции
- **WHEN** integration test падает на Windows
- **THEN** job Windows integration завершается ошибкой, а остальные платформы продолжают свои проверки

### Requirement: Обязательный итоговый статус
Итоговый CI passed SHALL выполняться после всех категорий и проходить только при success каждой категории. Failure, cancelled и skipped SHALL не давать успешный итоговый статус.

#### Scenario: Пропущенный job
- **WHEN** любой обязательный job пропущен или отменён
- **THEN** итоговый CI passed не разрешает слияние

### Requirement: Защита default branch
GitHub SHALL требовать PR и успешный актуальный CI passed от GitHub Actions перед слиянием в default branch, в том числе для администраторов. Force push и удаление default branch SHALL быть запрещены, обсуждения PR SHALL быть разрешены до слияния.

#### Scenario: CI не прошёл
- **WHEN** PR не имеет успешного актуального CI passed
- **THEN** GitHub блокирует слияние в default branch

#### Scenario: Прямой push
- **WHEN** администратор пытается отправить изменение прямо в default branch или выполнить force push
- **THEN** GitHub отклоняет обход установленного PR/CI workflow
