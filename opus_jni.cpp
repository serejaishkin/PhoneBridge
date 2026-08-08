#include <jni.h>
#include <opus/opus.h>
#include <android/log.h>

#define LOG_TAG "OpusJNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

extern "C" {

JNIEXPORT jlong JNICALL
Java_com_phonebridge_app_opus_OpusEncoder_nativeCreateEncoder(
    JNIEnv* env, jobject thiz, jint sampleRate, jint channels, jint bitrate)
{
    int err;
    OpusEncoder* enc = opus_encoder_create(sampleRate, channels, OPUS_APPLICATION_AUDIO, &err);
    if (err != OPUS_OK || enc == nullptr) {
        LOGE("opus_encoder_create failed: %d", err);
        return 0;
    }
    opus_encoder_ctl(enc, OPUS_SET_BITRATE(bitrate));
    opus_encoder_ctl(enc, OPUS_SET_SIGNAL(OPUS_SIGNAL_VOICE));
    return reinterpret_cast<jlong>(enc);
}

JNIEXPORT jint JNICALL
Java_com_phonebridge_app_opus_OpusEncoder_nativeEncode(
    JNIEnv* env, jobject thiz, jlong encoder, jshortArray pcm, jbyteArray output)
{
    if (encoder == 0) return -1;
    OpusEncoder* enc = reinterpret_cast<OpusEncoder*>(encoder);

    jsize pcmLen = env->GetArrayLength(pcm);
    jshort* pcmData = env->GetShortArrayElements(pcm, nullptr);
    jsize outLen = env->GetArrayLength(output);
    jbyte* outData = env->GetByteArrayElements(output, nullptr);

    int ret = opus_encode(enc, pcmData, pcmLen, reinterpret_cast<unsigned char*>(outData), outLen);

    env->ReleaseShortArrayElements(pcm, pcmData, JNI_ABORT);
    env->ReleaseByteArrayElements(output, outData, 0);
    return ret;
}

JNIEXPORT void JNICALL
Java_com_phonebridge_app_opus_OpusEncoder_nativeDestroyEncoder(
    JNIEnv* env, jobject thiz, jlong encoder)
{
    if (encoder != 0) {
        opus_encoder_destroy(reinterpret_cast<OpusEncoder*>(encoder));
    }
}

// --- Decoder ---

JNIEXPORT jlong JNICALL
Java_com_phonebridge_app_opus_OpusDecoder_nativeCreateDecoder(
    JNIEnv* env, jobject thiz, jint sampleRate, jint channels)
{
    int err;
    OpusDecoder* dec = opus_decoder_create(sampleRate, channels, &err);
    if (err != OPUS_OK || dec == nullptr) {
        LOGE("opus_decoder_create failed: %d", err);
        return 0;
    }
    return reinterpret_cast<jlong>(dec);
}

JNIEXPORT jint JNICALL
Java_com_phonebridge_app_opus_OpusDecoder_nativeDecode(
    JNIEnv* env, jobject thiz, jlong decoder, jbyteArray opusData, jshortArray pcmOut)
{
    if (decoder == 0) return -1;
    OpusDecoder* dec = reinterpret_cast<OpusDecoder*>(decoder);

    jsize opusLen = env->GetArrayLength(opusData);
    jbyte* opusBuf = env->GetByteArrayElements(opusData, nullptr);
    jsize pcmLen = env->GetArrayLength(pcmOut);
    jshort* pcmBuf = env->GetShortArrayElements(pcmOut, nullptr);

    int ret = opus_decode(dec,
        reinterpret_cast<const unsigned char*>(opusBuf), opusLen,
        pcmBuf, pcmLen, 0);

    env->ReleaseByteArrayElements(opusData, opusBuf, JNI_ABORT);
    env->ReleaseShortArrayElements(pcmOut, pcmBuf, 0);
    return ret;
}

JNIEXPORT void JNICALL
Java_com_phonebridge_app_opus_OpusDecoder_nativeDestroyDecoder(
    JNIEnv* env, jobject thiz, jlong decoder)
{
    if (decoder != 0) {
        opus_decoder_destroy(reinterpret_cast<OpusDecoder*>(decoder));
    }
}

} // extern "C"
