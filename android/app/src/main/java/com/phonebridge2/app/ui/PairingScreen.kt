package com.phonebridge2.app.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun PairingScreen(
    state: PairingUiState,
    onConfirm: () -> Unit,
    onReset: () -> Unit,
) {
    Column(
        modifier = Modifier.padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("PhoneBridge", style = MaterialTheme.typography.headlineLarge)
        when (state.step) {
            PairingStep.SelectPc -> {
                Text("Select the PhoneBridge PC you want to trust.")
                Text(state.peerName ?: "No PC selected")
                Text(state.peerAddress ?: "Discover a PC or enter its address in Settings.")
            }
            PairingStep.ConfirmCode -> {
                Text("Confirm pairing")
                Text("Verify that the same code is shown on the PC.")
                Card(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        state.code ?: "------",
                        modifier = Modifier.padding(24.dp),
                        style = MaterialTheme.typography.headlineMedium,
                    )
                }
                Button(onClick = onConfirm, enabled = !state.busy) {
                    Text(if (state.busy) "Pairing…" else "Trust this PC")
                }
            }
            PairingStep.Complete -> {
                Text("Pairing complete")
                Text("This PC is now trusted for future reconnects.")
                OutlinedButton(onClick = onReset) { Text("Pair another PC") }
            }
        }
    }
}
