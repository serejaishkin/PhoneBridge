package com.phonebridge.app.sms

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.telephony.SmsManager
import android.provider.Telephony
import androidx.core.content.ContextCompat
import com.phonebridge.app.network.SignalingClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

/** Phone-side SMS read/send bridge. */
object SmsBridge {
    private const val DEFAULT_PC_URL = SignalingClient.DEFAULT_URL
    private var context: Context? = null

    fun init(appContext: Context) {
        context = appContext.applicationContext
    }

    fun sendFromCommand(data: Map<String, String>) {
        val address = data["address"].orEmpty()
        val body = data["body"].orEmpty()
        if (address.isBlank() || body.isBlank()) return
        sendSms(address, body)
    }

    fun sendSms(address: String, body: String): Boolean {
        val ctx = context ?: return false
        if (ContextCompat.checkSelfPermission(ctx, Manifest.permission.SEND_SMS) != PackageManager.PERMISSION_GRANTED) {
            publish("sms_error", mapOf("error" to "SEND_SMS permission is not granted"))
            return false
        }

        return try {
            val manager = SmsManager.getDefault()
            val parts = manager.divideMessage(body)
            if (parts.size == 1) {
                manager.sendTextMessage(address, null, body, null, null)
            } else {
                manager.sendMultipartTextMessage(address, null, parts, null, null)
            }
            publish("sms_sent", mapOf("address" to address, "body" to body))
            true
        } catch (e: Exception) {
            publish("sms_error", mapOf("error" to (e.message ?: "SMS send failed")))
            false
        }
    }

    fun publishReceived(address: String, body: String, timestamp: Long) {
        publish(
            "sms_received",
            mapOf(
                "address" to address,
                "body" to body,
                "timestamp" to timestamp.toString()
            )
        )
    }

    fun publishRecent(limit: Int = 50) {
        val ctx = context ?: return
        if (ContextCompat.checkSelfPermission(ctx, Manifest.permission.READ_SMS) != PackageManager.PERMISSION_GRANTED) {
            publish("sms_error", mapOf("error" to "READ_SMS permission is not granted"))
            return
        }

        CoroutineScope(Dispatchers.IO).launch {
            try {
                val projection = arrayOf(
                    Telephony.Sms._ID,
                    Telephony.Sms.ADDRESS,
                    Telephony.Sms.BODY,
                    Telephony.Sms.DATE
                )
                ctx.contentResolver.query(
                    Telephony.Sms.Inbox.CONTENT_URI,
                    projection,
                    null,
                    null,
                    "${Telephony.Sms.DATE} DESC"
                )?.use { cursor ->
                    var count = 0
                    val idIndex = cursor.getColumnIndex(Telephony.Sms._ID)
                    val addressIndex = cursor.getColumnIndex(Telephony.Sms.ADDRESS)
                    val bodyIndex = cursor.getColumnIndex(Telephony.Sms.BODY)
                    val dateIndex = cursor.getColumnIndex(Telephony.Sms.DATE)
                    while (cursor.moveToNext() && count < limit) {
                        publish(
                            "sms_item",
                            mapOf(
                                "id" to if (idIndex >= 0) cursor.getString(idIndex) else "",
                                "address" to if (addressIndex >= 0) cursor.getString(addressIndex).orEmpty() else "",
                                "body" to if (bodyIndex >= 0) cursor.getString(bodyIndex).orEmpty() else "",
                                "timestamp" to if (dateIndex >= 0) cursor.getLong(dateIndex).toString() else "0"
                            )
                        )
                        count++
                    }
                }
                publish("sms_list_end", mapOf("count" to limit.coerceAtMost(50).toString()))
            } catch (e: Exception) {
                publish("sms_error", mapOf("error" to (e.message ?: "SMS read failed")))
            }
        }
    }

    private fun publish(type: String, data: Map<String, String>) {
        CoroutineScope(Dispatchers.IO).launch {
            val client = SignalingClient()
            if (client.connectBlocking(DEFAULT_PC_URL)) {
                client.sendEvent(type, data)
                Thread.sleep(100)
                client.disconnect()
            }
        }
    }
}
