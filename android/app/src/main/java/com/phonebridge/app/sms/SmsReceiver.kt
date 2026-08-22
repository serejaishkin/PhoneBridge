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
        for (message in messages) {
            val address = message.originatingAddress.orEmpty()
            val body = message.messageBody.orEmpty()
            if (address.isNotBlank() && body.isNotBlank()) {
                SmsBridge.init(context)
                SmsBridge.publishReceived(address, body, message.timestampMillis)
            }
        }
    }
}
