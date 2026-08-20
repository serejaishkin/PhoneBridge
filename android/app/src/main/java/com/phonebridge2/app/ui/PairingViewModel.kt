package com.phonebridge2.app.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** Owns pairing UI state; transport and trust decisions stay in connection/pairing layers. */
class PairingViewModel : ViewModel() {
    private val _state = MutableStateFlow(PairingUiState())
    val state: StateFlow<PairingUiState> = _state.asStateFlow()

    fun showCode(code: String) { _state.value = _state.value.copy(code = code, step = PairingStep.ConfirmCode) }
    fun setPeer(name: String, address: String) { _state.value = _state.value.copy(peerName = name, peerAddress = address) }
    fun setBusy(value: Boolean) { _state.value = _state.value.copy(busy = value) }
    fun paired() { _state.value = _state.value.copy(step = PairingStep.Complete, busy = false) }
    fun reset() { _state.value = PairingUiState() }
}

data class PairingUiState(
    val peerName: String? = null,
    val peerAddress: String? = null,
    val code: String? = null,
    val step: PairingStep = PairingStep.SelectPc,
    val busy: Boolean = false,
)

enum class PairingStep { SelectPc, ConfirmCode, Complete }
