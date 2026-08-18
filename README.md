# PhoneBridge2 (скелет v2)

Переосмысленный старт проекта после ревью `serejaishkin/PhoneBridge` — тот же
смысл (PC как беспроводная гарнитура/пульт для Android-звонков и медиа-звука,
без облака), но с исправленной архитектурой:

- **Звонки идут нативным Bluetooth Classic HFP** (телефон = Audio Gateway,
  PC = Hands-Free unit) — приложение НЕ пытается перехватывать аудио звонка
  само, это системно недостижимо без root/Linux+BlueZ. Приложение отвечает
  только за control-plane (показать входящий звонок, Answer/Decline).
- **Есть настоящий TLS-пейринг** (trust-on-first-use по fingerprint
  сертификата, вдохновлено KDE Connect, но реализовано самостоятельно —
  KDE Connect не форкали, см. обсуждение лицензии GPL vs наш MIT).
- **Discovery по UDP broadcast**, не BLE — раз всё равно нужен общий Wi-Fi
  для медиа-потока, BLE не даёт ничего, кроме лишних Android 12+ разрешений.

## Статус: скелет, не MVP

Собирается и проверено в этой сессии:
- `pc/` — компилируется (`cargo check` зелёный: identity/pairing/discovery/protocol/
  call-state/ui-trait). **Не тестировался end-to-end** (два реальных устройства
  друг с другом не соединялись).
- `android/` — код написан по тем же контрактам (протокол, short_code, TelephonyCallback,
  InCallService), но **не собирался** — в этом окружении нет Android SDK/Gradle
  с доступом к google()/mavenCentral(). Синтаксис вычитан вручную, но это не
  замена реальной сборке.

## Структура

```
pc/                      Rust, ПК-часть
  src/pairing/            identity.rs (сертификат), trust.rs (доверие), server.rs (TLS listener)
  src/discovery/           UDP broadcast
  src/call/                состояние звонка + заглушка check_hfp_support()
  src/ui/                  UiBackend trait + HeadlessUi (лог вместо трея — платформенные
                            реализации трея/окон — следующий шаг, см. NEXT_STEPS.md)
  src/protocol.rs          JSON-протокол сообщений

android/app/.../com/phonebridge2/app/
  pairing/                 Identity.kt, TrustStore.kt, Protocol.kt (зеркало pc/src/protocol.rs)
  discovery/               DiscoveryClient.kt
  call/                    CallManager.kt (TelephonyCallback), BridgeInCallService.kt (answer/decline — РАБОЧИЕ, не заглушки)
  ui/onboarding/           OnboardingStep.kt (модель шагов из AI_HANDOFF_GUI.md)
  MainActivity.kt          минимальный экран, связывающий всё вместе

AI_HANDOFF_GUI.md          постановка задачи на GUI-архитектуру (из предыдущего ревью)
NEXT_STEPS.md              что делать дальше и в каком порядке
```

## Сборка PC-части

```
cd pc
cargo check   # или cargo run
```

Зависимости специально запинены точными версиями в `Cargo.toml`
(`rustls`, `tokio-rustls`, `zeroize`, `time`) — новые версии этих крейтов
требуют `edition2024`, который не поддерживает cargo 1.75. Если у вас cargo
новее — можно ослабить пины, но тогда стоит перепроверить сборку.

## Сборка Android-части

Не проверялась в этой сессии. Нужен Android Studio / Gradle с доступом к
`google()` и `mavenCentral()`. minSdk = 31 (нужен `TelephonyCallback`).
