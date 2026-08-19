package com.phonebridge2.app.call

import com.phonebridge2.app.connection.ConnectionManager
import com.phonebridge2.app.pairing.Message
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

/** Bridges Android Telecom state/commands to the authenticated PhoneBridge channel. */
class CallBridge(
    private val connection: ConnectionManager,
    private val scope: CoroutineScope,
    private val inCallService: BridgeInCallService,
) {
    fun start(callManager: CallManager) {
        scope.launch {
            callManager.state.collectLatest { state ->
                when (state) {
                    is CallState.Ringing -> connection.send(
                        Message.IncomingCall(
                            callerNumber = state.number,
                            callerName = null,
                        )
                    )
                    CallState.Active -> Unit
                    CallState.Idle -> connection.send(Message.CallEnded)
                }
            }
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
}
