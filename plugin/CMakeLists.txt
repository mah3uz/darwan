cmake_minimum_required(VERSION 3.21)
project(darwan-plugin LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Quick OpenGL)
find_package(PkgConfig REQUIRED)
pkg_check_modules(MPV REQUIRED IMPORTED_TARGET mpv)
qt_standard_project_setup(REQUIRES 6.5)

# Two modules, so that a video plugin that can't load (libmpv missing or changed) costs only mpv: the gate that wakes
# the screensaver needs nothing but Qt, and videos fall back to Qt's player.
qt_add_qml_module(darwanplugin
    URI Darwan
    VERSION 1.0
    PLUGIN_TARGET darwanplugin
    OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/qml/Darwan
    SOURCES inputgate.h inputgate.cpp
)
target_link_libraries(darwanplugin PRIVATE Qt6::Quick)

qt_add_qml_module(darwanvideoplugin
    URI Darwan.Video
    VERSION 1.0
    PLUGIN_TARGET darwanvideoplugin
    OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/qml/Darwan/Video
    SOURCES
        mpvvideo.h mpvvideo.cpp
        mpvqt/mpvabstractitem.h mpvqt/mpvabstractitem_p.h mpvqt/mpvabstractitem.cpp
        mpvqt/mpvcontroller.h mpvqt/mpvcontroller_p.h mpvqt/mpvcontroller.cpp
        mpvqt/mpvrenderer.h mpvqt/mpvrenderer.cpp
        mpvqt/mpvqt_export.h
)
target_include_directories(darwanvideoplugin PRIVATE mpvqt)
target_link_libraries(darwanvideoplugin PRIVATE Qt6::Quick Qt6::OpenGL PkgConfig::MPV)
