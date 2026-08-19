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
import com.phonebridge2.app.connection.ConnectionManager
import com.phonebridge2.app.discovery.Announce
import com.phonebridge2.app.discovery.DiscoveryClient
import com.phonebridge2.app.pairing.Identity
import com.phonebridge2.app.pairing.PairingManager
import com.phonebridge2.app.pairing.TlsClient
import com.phonebridge2.app.ui.onboarding.OnboardingStep

class MainActivity : ComponentActivity() {
    private lateinit var identity: Identity
    private lateinit var callManager: CallManager
    private lateinit var connectionManager: ConnectionManager
    private lateinit var pairingManager: PairingManager

    private val permissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        identity = Identity.loadOrCreate(applicationContext)
        callManager = CallManager(applicationContext).also { it.start() }
        connectionManager = ConnectionManager(TlsClient(), lifecycleScope)
        pairingManager = PairingManager(applicationContext, identity, connectionManager)
        connectionManager.setMessageHandler { message -> pairingManager.onMessage(message) }

        val foundPeers = mutableStateListOf<Pair<Announce, String>>()
        DiscoveryClient.listen(lifecycleScope) { announce, addr ->
            runOnUiThread {
                val index = foundPeers.indexOfFirst { it.first.device_id == announce.device_id }
                if (index >= 0) {
                    foundPeers[index] = announce to addr
                } else {
                    foundPeers.add(announce to addr)
                }
            }
        }

        setContent {
            val connectionState by connectionManager.state.collectAsState()
            val pairingState by pairingManager.state.collectAsState()

            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    MainScreen(
                        deviceId = identity.deviceId,
                        fingerprint = identity.fingerprintHex(),
                        callState = callManager.state.collectAsState().value,
                        connectionState = connectionState.toString(),
                        pairingState = pairingState,
                        foundPeers = foundPeers,
                        onConnect = { peer ->
                            connectionManager.connect(
                                host = peer.second,
                                port = peer.first.pairing_port,
                                fingerprint = peer.first.fingerprint,
                                hello = pairingManager.hello(identity.deviceId),
                            )
                        },
                        onConfirmPairing = { pairingManager.confirmPairing() },
                        onRequestPermissions = {
                            permissionLauncher.launch(
                                arrayOf(
                                    android.Manifest.permission.READ_PHONE_STATE,
                                    android.Manifest.permission.ANSWER_PHONE_CALLS,
                                    android.Manifest.permission.POST_NOTIFICATIONS,
                                )
                            )
                        },
                    )
                }
            }
        }
    }

    override fun onDestroy() {
        connectionManager.disconnect()
        callManager.stop()
        super.onDestroy()
    }
}

@Composable
private fun MainScreen(
    deviceId: String,
    fingerprint: String,
    callState: CallState,
    connectionState: String,
    pairingState: PairingManager.State,
    foundPeers: List<Pair<Announce, String>>,
    onConnect: (Pair<Announce, String>) -> Unit,
    onConfirmPairing: () -> Unit,
    onRequestPermissions: () -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("PhoneBridge2", style = MaterialTheme.typography.headlineSmall)
        Text("device_id: $deviceId")
        Text("fingerprint: ${fingerprint.take(16)}…")
        Text("Соединение: $connectionState")

        Button(onClick = onRequestPermissions) {
            Text("Запросить разрешения")
        }

        Divider()
        Text("Найденные ПК:", style = MaterialTheme.typography.titleMedium)
        if (foundPeers.isEmpty()) {
            Text("Пока никого — убедитесь, что телефон и ПК в одной Wi-Fi сети.")
        } else {
            foundPeers.forEach { peer ->
                val (announce, addr) = peer
                Text("${announce.device_name} (${announce.platform})")
                Text("$addr:${announce.pairing_port}", style = MaterialTheme.typography.bodySmall)
                Text("PC fingerprint: ${announce.fingerprint.take(16)}…", style = MaterialTheme.typography.bodySmall)
                Button(onClick = { onConnect(peer) }) {
                    Text("Подключить")
                }
            }
        }

        when (val state = pairingState) {
            is PairingManager.State.WaitingForConfirmation -> {
                Divider()
                Text("Подтвердите сопряжение", style = MaterialTheme.typography.titleMedium)
                Text("Код: ${state.shortCode}")
                Text("Сверьте этот код на ПК и телефоне. Только после совпадения нажмите подтверждение.")
                Button(onClick = onConfirmPairing) {
                    Text("Подтвердить сопряжение")
                }
            }
            is PairingManager.State.Paired -> {
                Divider()
                Text("ПК сопряжён: ${state.pcDeviceId}")
            }
            is PairingManager.State.Failed -> {
                Divider()
                Text("Ошибка сопряжения: ${state.message}")
            }
            PairingManager.State.Idle -> Unit
        }

        Divider()
        Text("Статус звонка: ${callStateLabel(callState)}")

        Divider()
        Text("Шаги онбординга:", style = MaterialTheme.typography.titleMedium)
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
