cmake_minimum_required(VERSION 3.21)
project(darwan-plugin LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Quick)
find_package(MpvQt REQUIRED)
qt_standard_project_setup(REQUIRES 6.5)

qt_add_qml_module(darwanplugin
    URI Darwan
    VERSION 1.0
    PLUGIN_TARGET darwanplugin
    OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/qml/Darwan
    SOURCES mpvvideo.h mpvvideo.cpp inputgate.h inputgate.cpp
)
target_link_libraries(darwanplugin PRIVATE Qt6::Quick MpvQt::MpvQt)
