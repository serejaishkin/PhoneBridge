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
import com.phonebridge2.app.discovery.DiscoveredPeer
import com.phonebridge2.app.discovery.DiscoveryClient
import com.phonebridge2.app.discovery.EndpointStore
import com.phonebridge2.app.discovery.PeerConnectionStore
import com.phonebridge2.app.discovery.PreferredRouteStore
import com.phonebridge2.app.pairing.Identity
import com.phonebridge2.app.pairing.PairingManager
import com.phonebridge2.app.pairing.TlsClient
import com.phonebridge2.app.ui.PairingScreen
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

        val endpointStore = EndpointStore(applicationContext)
        val peerStore = PeerConnectionStore(applicationContext)
        val routeStore = PreferredRouteStore(applicationContext)
        connectionManager = ConnectionManager(
            tlsClient = TlsClient(),
            scope = lifecycleScope,
            endpointStore = endpointStore,
            peerStore = peerStore,
            routeStore = routeStore,
        )
        pairingManager = PairingManager(applicationContext, identity, connectionManager)
        connectionManager.setMessageHandler { message -> pairingManager.onMessage(message) }

        val foundPeers = mutableStateListOf<DiscoveredPeer>()
        DiscoveryClient.listen(lifecycleScope) { announce, addr ->
            runOnUiThread {
                val peer = DiscoveredPeer(
                    deviceId = announce.device_id,
                    deviceName = announce.device_name,
                    platform = announce.platform,
                    host = addr,
                    port = announce.pairing_port,
                    fingerprint = announce.fingerprint,
                )
                val index = foundPeers.indexOfFirst { it.deviceId == peer.deviceId }
                if (index >= 0) foundPeers[index] = peer else foundPeers.add(peer)
            }
        }

        setContent {
            val connectionState by connectionManager.state.collectAsState()
            val pairingState by pairingManager.state.collectAsState()
            var showPairing by remember { mutableStateOf(false) }

            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    if (showPairing) {
                        PairingScreen(
                            state = pairingUiState(pairingState),
                            onConfirm = { pairingManager.confirmPairing() },
                            onReset = { showPairing = false },
                        )
                    } else {
                        MainScreen(
                            deviceId = identity.deviceId,
                            fingerprint = identity.fingerprintHex(),
                            callState = callManager.state.collectAsState().value,
                            connectionState = connectionState.toString(),
                            pairingState = pairingState,
                            foundPeers = foundPeers,
                            onConnect = { peer ->
                                peerStore.select(peer.deviceId)
                                endpointStore.save(peer)
                                connectionManager.connect(
                                    peer = peer,
                                    hello = pairingManager.hello(identity.deviceId),
                                )
                            },
                            onPairing = { showPairing = true },
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
    foundPeers: List<DiscoveredPeer>,
    onConnect: (DiscoveredPeer) -> Unit,
    onPairing: () -> Unit,
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
        Text("Connection: $connectionState")

        Button(onClick = onRequestPermissions) { Text("Permissions") }
        Button(onClick = onPairing) { Text("Pairing wizard") }

        HorizontalDivider()
        Text("Discovered PCs", style = MaterialTheme.typography.titleMedium)
        if (foundPeers.isEmpty()) {
            Text("No PCs discovered yet. Make sure the phone and PC share a network.")
        } else {
            foundPeers.forEach { peer ->
                Text("${peer.deviceName} (${peer.platform})")
                Text("${peer.host}:${peer.port}", style = MaterialTheme.typography.bodySmall)
                Text("PC fingerprint: ${peer.fingerprint.take(16)}…", style = MaterialTheme.typography.bodySmall)
                Button(onClick = { onConnect(peer) }) { Text("Connect") }
            }
        }

        when (val state = pairingState) {
            is PairingManager.State.WaitingForConfirmation -> {
                HorizontalDivider()
                Text("Pairing code: ${state.shortCode}", style = MaterialTheme.typography.titleMedium)
                Text("Compare this code with the PC before confirming.")
                Button(onClick = onConfirmPairing) { Text("Confirm pairing") }
            }
            is PairingManager.State.Paired -> {
                HorizontalDivider()
                Text("PC paired: ${state.pcDeviceId}")
            }
            is PairingManager.State.Failed -> {
                HorizontalDivider()
                Text("Pairing error: ${state.message}")
            }
            PairingManager.State.Idle -> Unit
        }

        HorizontalDivider()
        Text("Call status: ${callStateLabel(callState)}")
        Text("Onboarding", style = MaterialTheme.typography.titleMedium)
        OnboardingStep.ORDER.forEach { step ->
            Text("• ${step.title}: ${step.explanation}", style = MaterialTheme.typography.bodySmall)
        }
    }
}

private fun pairingUiState(state: PairingManager.State): com.phonebridge2.app.ui.PairingUiState = when (state) {
    is PairingManager.State.WaitingForConfirmation -> com.phonebridge2.app.ui.PairingUiState(
        peerName = state.pcDeviceId,
        code = state.shortCode,
        step = com.phonebridge2.app.ui.PairingStep.ConfirmCode,
    )
    is PairingManager.State.Paired -> com.phonebridge2.app.ui.PairingUiState(
        peerName = state.pcDeviceId,
        step = com.phonebridge2.app.ui.PairingStep.Complete,
    )
    is PairingManager.State.Failed -> com.phonebridge2.app.ui.PairingUiState()
    PairingManager.State.Idle -> com.phonebridge2.app.ui.PairingUiState()
}

private fun callStateLabel(state: CallState): String = when (state) {
    is CallState.Idle -> "idle"
    is CallState.Ringing -> "ringing: ${state.number ?: "hidden number"}"
    is CallState.Active -> "active"
}
