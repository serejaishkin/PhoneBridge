package com.phonebridge2.app.pairing

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.BufferedReader
import java.io.BufferedWriter

/**
 * Single owner of the PhoneBridge wire framing rules.
 * Every frame is exactly one UTF-8 JSON message followed by LF.
 */
class FramedChannel(
    private val reader: BufferedReader,
    private val writer: BufferedWriter,
) {
    @Synchronized
    fun write(message: Message) {
        writer.write(ProtocolJson.encode(message))
        writer.newLine()
        writer.flush()
    }

    @Synchronized
    fun read(): Message {
        val line = reader.readLine() ?: error("PhoneBridge connection closed")
        require(line.isNotBlank()) { "PhoneBridge received an empty frame" }
        return ProtocolJson.decode(line)
    }

    suspend fun writeAsync(message: Message) = withContext(Dispatchers.IO) {
        write(message)
    }

    suspend fun readAsync(): Message = withContext(Dispatchers.IO) {
        read()
    }
}
