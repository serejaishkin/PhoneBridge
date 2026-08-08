package com.phonebridge.app.opus

class OpusDecoder {

    private var nativeDecoder: Long = 0

    init {
        nativeDecoder = nativeCreateDecoder(48000, 1)
    }

    fun decode(opusData: ByteArray, pcmOut: ShortArray): Int {
        if (nativeDecoder == 0L) return -1
        return nativeDecode(nativeDecoder, opusData, pcmOut)
    }

    fun destroy() {
        if (nativeDecoder != 0L) {
            nativeDestroyDecoder(nativeDecoder)
            nativeDecoder = 0
        }
    }

    protected fun finalize() {
        destroy()
    }

    private external fun nativeCreateDecoder(sampleRate: Int, channels: Int): Long
    private external fun nativeDecode(decoder: Long, opusData: ByteArray, pcmOut: ShortArray): Int
    private external fun nativeDestroyDecoder(decoder: Long)

    companion object {
        init {
            System.loadLibrary("opus_jni")
        }
    }
}
