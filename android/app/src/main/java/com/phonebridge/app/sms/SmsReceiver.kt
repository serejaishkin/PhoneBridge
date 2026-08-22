package com.phonebridge.app.sms

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.provider.Telephony

/** Receives SMS_RECEIVED and forwards decoded messages to the PC bridge. */
class SmsReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Telephony.Sms.Intents.SMS_RECEIVED_ACTION) return

        val messages = Telephony.Sms.Intents.getMessagesFromIntent(intent)
        if (messages.isEmpty()) return

        // A long SMS may arrive as multiple PDUs. Reassemble it before sending
        // it to the PC so the desktop never sees duplicate partial messages.
        val grouped = messages
            .filter { !it.originatingAddress.isNullOrBlank() }
            .groupBy { it.originatingAddress.orEmpty() to it.timestampMillis }

        SmsBridge.init(context)
        grouped.forEach { (key, parts) ->
            val body = parts.joinToString(separator = "") { it.messageBody.orEmpty() }
            if (body.isNotBlank()) {
                SmsBridge.publishReceived(key.first, body, key.second)
            }
        }
    }
}
