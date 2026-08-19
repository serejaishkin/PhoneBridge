package com.phonebridge2.app.call

import com.phonebridge2.app.connection.ConnectionManager
import com.phonebridge2.app.pairing.Message
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch

/** Bridges Android telephony state/commands to the authenticated PhoneBridge channel. */
class CallBridge(
    private val connection: ConnectionManager,
    private val scope: CoroutineScope,
    private val inCallService: BridgeInCallService,
) {
    private var stateJob: Job? = null

    fun start(callManager: CallManager) {
        stateJob?.cancel()
        stateJob = scope.launch {
            callManager.state
                .map { it.toMessage() }
                .distinctUntilChanged()
                .collect { message -> connection.send(message) }
        }

        connection.setMessageHandler { message ->
            when (message) {
                Message.CallAnswer -> inCallService.answerCurrentCall()
                Message.CallDecline -> inCallService.declineCurrentCall()
                Message.CallEnded -> inCallService.endCurrentCall()
                else -> Unit
            }
        }
    }

    fun stop() {
        stateJob?.cancel()
        stateJob = null
    }

    private fun CallState.toMessage(): Message = when (this) {
        is CallState.Ringing -> Message.IncomingCall(
            callerNumber = number,
            callerName = null,
        )
        CallState.Active -> Message.CallAnswer
        CallState.Idle -> Message.CallEnded
    }
}
