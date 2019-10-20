# For more information about using CMake with Android Studio, read the
# documentation: https://d.android.com/studio/projects/add-native-code.html

# Sets the minimum version of CMake required to build the native library.
cmake_minimum_required(VERSION 3.4.1)

project(android)

# Creates and names a library, sets it as either STATIC
# or SHARED, and provides the relative paths to its source code.
# You can define multiple libraries, and CMake builds them for you.
# Gradle automatically packages shared libraries with your APK.

#Includes node's header files.
include_directories(rust/libnode/include/node/)

# add imported target
add_library(librust STATIC IMPORTED)
# point the imported target at the real file
set_property(TARGET librust PROPERTY
        IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/rust/target/${ANDROID_ABI}/librust.a)

add_library(
        rust-prebuild
        rust/utf8_util/build/util.cpp
        rust/build/util/util.cpp
        rust/build/v8_jni/wrapper.cpp
        rust/build/api.cpp)

add_library( # Sets the name of the library.
        native-lib
        # Sets the library as a shared library.
        SHARED
        # Provides a relative path to your source file(s).
        src/main/cpp/utils/utils.cpp
        src/main/cpp/lib/native-lib.cpp
        src/main/cpp/lib/node-ext.cpp
        # for v8
        src/main/cpp/v8/v8context.cpp)

add_library(libnode
        SHARED
        IMPORTED)

set_target_properties( # Specifies the target library.
        libnode
        # Specifies the parameter you want to define.
        PROPERTIES IMPORTED_LOCATION
        # Provides the path to the library you want to import.
        ${CMAKE_SOURCE_DIR}/rust/libnode/bin/${ANDROID_ABI}/libnode.so)

# Searches for a specified prebuilt library and stores the path as a
# variable. Because CMake includes system libraries in the search path by
# default, you only need to specify the name of the public NDK library
# you want to add. CMake verifies that the library exists before
# completing its build.
find_library( # Sets the name of the path variable.
        log-lib
        # Specifies the name of the NDK library that
        # you want CMake to locate.
        log)

# Specifies libraries CMake should link to your target library. You
# can link multiple libraries, such as libraries you define in this
# build script, prebuilt third-party libraries, or system libraries.
target_link_libraries( # Specifies the target library.
        native-lib
        librust
        jnigraphics
        libnode
        # Links the target library to the log library
        # included in the NDK.
        log
        android)

# add imported target
add_library(image-gen SHARED src/main/cpp/lib/image-gen.cpp)

target_link_libraries(image-gen jnigraphics ${log-lib} librust)
