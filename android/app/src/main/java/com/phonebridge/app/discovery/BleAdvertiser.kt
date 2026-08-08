package com.phonebridge.app.discovery

import android.bluetooth.BluetoothAdapter
import android.bluetooth.BluetoothManager
import android.bluetooth.le.AdvertiseCallback
import android.bluetooth.le.AdvertiseData
import android.bluetooth.le.AdvertiseSettings
import android.content.Context
import android.net.wifi.WifiManager
import android.os.ParcelUuid
import java.nio.ByteBuffer
import java.util.UUID

class BleAdvertiser(private val context: Context) {

    private val bluetoothManager = context.getSystemService(Context.BLUETOOTH_SERVICE) as BluetoothManager
    private val adapter: BluetoothAdapter? = bluetoothManager.adapter
    private var advertiser = adapter?.bluetoothLeAdvertiser
    private var callback: AdvertiseCallback? = null

    companion object {
        val SERVICE_UUID: UUID = UUID.fromString("a1b2c3d4-e5f6-7890-abcd-ef1234567890")
        const val MANUFACTURER_ID = 0xFFFF
    }

    fun start() {
        if (adapter == null || !adapter.isEnabled || advertiser == null) {
            return
        }

        val ip = getWifiIp()
        val port: Short = 5003

        // Manufacturer data: 4 bytes IP + 2 bytes port
        val mfrData = ByteBuffer.allocate(6)
            .put(ip[0]).put(ip[1]).put(ip[2]).put(ip[3])
            .putShort(port)
            .array()

        val settings = AdvertiseSettings.Builder()
            .setAdvertiseMode(AdvertiseSettings.ADVERTISE_MODE_LOW_LATENCY)
            .setTxPowerLevel(AdvertiseSettings.ADVERTISE_TX_POWER_HIGH)
            .setConnectable(false)
            .build()

        val data = AdvertiseData.Builder()
            .setIncludeDeviceName(true)
            .addServiceUuid(ParcelUuid(SERVICE_UUID))
            .addManufacturerData(MANUFACTURER_ID, mfrData)
            .build()

        callback = object : AdvertiseCallback() {
            override fun onStartSuccess(settingsInEffect: AdvertiseSettings?) {
                android.util.Log.i("BleAdvertiser", "BLE advertising started. IP=${ip.joinToString(".")}, port=$port")
            }
            override fun onStartFailure(errorCode: Int) {
                android.util.Log.e("BleAdvertiser", "BLE advertising failed: $errorCode")
            }
        }

        advertiser?.startAdvertising(settings, data, callback!!)
    }

    fun stop() {
        callback?.let { advertiser?.stopAdvertising(it) }
        callback = null
    }

    private fun getWifiIp(): ByteArray {
        return try {
            val wm = context.getSystemService(Context.WIFI_SERVICE) as WifiManager
            val ip = wm.connectionInfo.ipAddress
            byteArrayOf(
                (ip and 0xFF).toByte(),
                (ip shr 8 and 0xFF).toByte(),
                (ip shr 16 and 0xFF).toByte(),
                (ip shr 24 and 0xFF).toByte()
            )
        } catch (e: Exception) {
            byteArrayOf(192.toByte(), 168.toByte(), 137.toByte(), 2.toByte())
        }
    }
}
