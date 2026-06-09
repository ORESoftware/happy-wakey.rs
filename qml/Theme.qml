import QtQuick

Item {
    id: theme
    visible: false
    width: 0
    height: 0

    property int hour: new Date().getHours()
    property bool softMorning: hour >= 5 && hour < 8

    property color page: softMorning ? "#f4efe6" : "#f7f9fc"
    property color content: softMorning ? "#fbf6ed" : "#ffffff"
    property color surface: softMorning ? "#fffaf2" : "#ffffff"
    property color surfaceAlt: softMorning ? "#efe6d8" : "#edf2f7"
    property color sidebar: softMorning ? "#eadfcc" : "#e7edf5"
    property color sidebarHeader: softMorning ? "#dfd1bb" : "#d9e3ef"
    property color selected: softMorning ? "#d8e8d5" : "#d9ecff"
    property color border: softMorning ? "#d8cbb8" : "#cfd9e6"
    property color text: "#1f2937"
    property color muted: "#64748b"
    property color faint: "#94a3b8"
    property color accent: softMorning ? "#4f7f63" : "#2563eb"
    property color accentSoft: softMorning ? "#e1efe0" : "#e8f1ff"
    property color accentText: "#ffffff"
    property color positive: "#15803d"
    property color negative: "#b91c1c"
    property color warning: "#a16207"

    Timer {
        interval: 60000
        repeat: true
        running: true
        onTriggered: theme.hour = new Date().getHours()
    }
}
