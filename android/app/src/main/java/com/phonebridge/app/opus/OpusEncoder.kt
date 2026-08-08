package com.phonebridge.app.opus

class OpusEncoder {

    private var nativeEncoder: Long = 0

    init {
        nativeEncoder = nativeCreateEncoder(48000, 1, 2048)
    }

    fun encode(pcm: ShortArray, output: ByteArray): Int {
        if (nativeEncoder == 0L) return -1
        return nativeEncode(nativeEncoder, pcm, output)
    }

    fun destroy() {
        if (nativeEncoder != 0L) {
            nativeDestroyEncoder(nativeEncoder)
            nativeEncoder = 0
        }
    }

    protected fun finalize() {
        destroy()
    }

    private external fun nativeCreateEncoder(sampleRate: Int, channels: Int, bitrate: Int): Long
    private external fun nativeEncode(encoder: Long, pcm: ShortArray, output: ByteArray): Int
    private external fun nativeDestroyEncoder(encoder: Long)

    companion object {
        init {
            System.loadLibrary("opus_jni")
        }
    }
}
