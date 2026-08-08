cmake_minimum_required(VERSION 3.10.2)
project("opus_jni")

set(OPUS_PREBUILT ${CMAKE_SOURCE_DIR}/opus_prebuilt/${ANDROID_ABI})

if(EXISTS ${OPUS_PREBUILT}/libopus.a)
    add_library(opus STATIC IMPORTED)
    set_target_properties(opus PROPERTIES IMPORTED_LOCATION ${OPUS_PREBUILT}/libopus.a)
    include_directories(${OPUS_PREBUILT}/include)
    add_definitions(-DHAVE_OPUS=1)
else()
    message(WARNING "libopus.a not found for ${ANDROID_ABI}. Opus support disabled.")
endif()

add_library(
    opus_jni
    SHARED
    opus_jni.cpp)

find_library(log-lib log)

if(TARGET opus)
    target_link_libraries(opus_jni opus ${log-lib})
else()
    target_link_libraries(opus_jni ${log-lib})
endif()
