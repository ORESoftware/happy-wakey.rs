#pragma once
// Qt requires QtWebEngineQuick::initialize() to be called before the QML engine
// loads any WebEngineView. cxx-qt-lib has no wrapper for this, so we expose a
// tiny C++ shim that the Rust bridge calls once at startup.
#include <QtWebEngineQuick/qtwebenginequickglobal.h>

inline void happy_init_web_engine() {
    QtWebEngineQuick::initialize();
}
