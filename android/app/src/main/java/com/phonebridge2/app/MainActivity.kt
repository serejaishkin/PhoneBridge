package com.phonebridge2.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import com.phonebridge2.app.call.CallManager
import com.phonebridge2.app.call.CallState
import com.phonebridge2.app.discovery.Announce
import com.phonebridge2.app.discovery.DiscoveryClient
import com.phonebridge2.app.pairing.Identity
import com.phonebridge2.app.pairing.TrustStore
import com.phonebridge2.app.ui.onboarding.OnboardingStep

/**
 * Минимальный, но НЕ фиктивный экран: в отличие от PhoneBridge v1
 * (один экран с двумя кнопками "Start"/"Stop"), здесь уже видна структура
 * онбординга и реальное состояние компонентов. Полноценные экраны под каждый
 * OnboardingStep — задача для Kimi (см. AI_HANDOFF_GUI.md).
 */
class MainActivity : ComponentActivity() {

    private lateinit var identity: Identity
    private lateinit var trustStore: TrustStore
    private lateinit var callManager: CallManager

    private val permissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { /* TODO(Kimi): по каждому разрешению — свой экран-объяснение, не общий батч */ }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        identity = Identity.loadOrCreate(applicationContext)
        trustStore = TrustStore(applicationContext)
        callManager = CallManager(applicationContext).also { it.start() }

        val foundPeers = mutableStateListOf<Pair<Announce, String>>()
        DiscoveryClient.listen(lifecycleScope) { announce, addr ->
            if (foundPeers.none { it.first.device_id == announce.device_id }) {
                foundPeers.add(announce to addr)
            }
        }

        setContent {
            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    MainScreen(
                        deviceId = identity.deviceId,
                        fingerprint = identity.fingerprintHex(),
                        callState = callManager.state.collectAsState().value,
                        foundPeers = foundPeers,
                        onRequestPermissions = {
                            permissionLauncher.launch(
                                arrayOf(
                                    android.Manifest.permission.READ_PHONE_STATE,
                                    android.Manifest.permission.ANSWER_PHONE_CALLS,
                                    android.Manifest.permission.POST_NOTIFICATIONS
                                )
                            )
                        }
                    )
                }
            }
        }
    }

    override fun onDestroy() {
        callManager.stop()
        super.onDestroy()
    }
}

@Composable
private fun MainScreen(
    deviceId: String,
    fingerprint: String,
    callState: CallState,
    foundPeers: List<Pair<Announce, String>>,
    onRequestPermissions: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text("PhoneBridge2 (skeleton)", style = MaterialTheme.typography.headlineSmall)
        Text("device_id: $deviceId")
        Text("fingerprint: ${fingerprint.take(16)}…")

        Button(onClick = onRequestPermissions) {
            Text("Запросить разрешения")
        }

        Divider()

        Text("Найденные ПК:", style = MaterialTheme.typography.titleMedium)
        if (foundPeers.isEmpty()) {
            Text("Пока никого — убедитесь, что телефон в той же Wi-Fi сети, что и ПК.")
        } else {
            foundPeers.forEach { (announce, addr) ->
                Text("• ${announce.device_name} (${announce.platform}) — $addr:${announce.pairing_port}")
            }
        }

        Divider()

        Text("Статус звонка: ${callStateLabel(callState)}")

        Divider()

        Text("Шаги онбординга (модель, экраны — TODO):", style = MaterialTheme.typography.titleMedium)
        OnboardingStep.ORDER.forEach { step ->
            Text("• ${step.title}: ${step.explanation}", style = MaterialTheme.typography.bodySmall)
        }
    }
}

private fun callStateLabel(state: CallState): String = when (state) {
    is CallState.Idle -> "нет звонка"
    is CallState.Ringing -> "звонит: ${state.number ?: "номер скрыт"}"
    is CallState.Active -> "разговор идёт"
}
