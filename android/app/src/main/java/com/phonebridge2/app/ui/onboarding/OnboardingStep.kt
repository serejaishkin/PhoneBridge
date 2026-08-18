package com.phonebridge2.app.ui.onboarding

/**
 * Шаги онбординга ровно по AI_HANDOFF_GUI.md, раздел 3.2. Здесь — только модель
 * состояния и тексты, конкретные Composable-экраны под каждый шаг — задача Kimi
 * (см. HANDOFF-документ в корне архива, "что дальше" -> GUI).
 */
enum class OnboardingStep(val title: String, val explanation: String) {
    PERMISSIONS(
        title = "Разрешения",
        explanation = "Каждое разрешение объясняется отдельно, а не одним " +
            "непрозрачным батчем, как было в PhoneBridge v1."
    ),
    WIFI_CONNECT(
        title = "Подключение к ПК по Wi-Fi",
        explanation = "Телефон и ПК должны быть в одной сети (общий Wi-Fi " +
            "или hotspot ПК) — без этого не заработают ни discovery, ни медиа-поток."
    ),
    DISCOVERY(
        title = "Поиск ПК",
        explanation = "UDP broadcast discovery в локальной сети (см. DiscoveryClient.kt)."
    ),
    PAIRING_CONFIRM(
        title = "Подтверждение пейринга",
        explanation = "Сверьте код на телефоне и на ПК — они должны совпадать " +
            "(см. TrustStore.shortCode). Это защита от подключения не того устройства."
    ),
    BLUETOOTH_HFP(
        title = "Bluetooth-звонки",
        explanation = "Включите переключатель \"Calls\"/\"Phone audio\" в настройках " +
            "Bluetooth-сопряжения с этим ПК — без этого шага звук звонков слышен не будет, " +
            "и это никак не зависит от самого приложения."
    ),
    DONE(
        title = "Готово",
        explanation = "Все каналы активны: control-plane, медиа-аудио и Bluetooth HFP."
    );

    companion object {
        val ORDER = listOf(PERMISSIONS, WIFI_CONNECT, DISCOVERY, PAIRING_CONFIRM, BLUETOOTH_HFP, DONE)
    }
}
