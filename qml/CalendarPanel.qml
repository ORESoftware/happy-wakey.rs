import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    color: "transparent"

    ColumnLayout {
        anchors.fill: parent
        spacing: 12

        // Header
        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "📅 Calendar"
                font.pixelSize: 22
                font.bold: true
                color: "#cdd6f4"
            }
            Item { Layout.fillWidth: true }
            Button {
                text: "Refresh"
                onClicked: backend.refresh_calendar()
                flat: true
            }
        }

        // Simple weekly view header
        RowLayout {
            Layout.fillWidth: true
            spacing: 2
            Repeater {
                model: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 32
                    color: "#1e1e2e"
                    radius: 4
                    Text {
                        anchors.centerIn: parent
                        text: modelData
                        font.pixelSize: 12
                        font.bold: true
                        color: "#a6adc8"
                    }
                }
            }
        }

        // Event list (parsed from backend.calendar_json)
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ListView {
                id: eventList
                model: eventModel
                delegate: Rectangle {
                    width: parent ? parent.width : 0
                    height: 52
                    color: index % 2 === 0 ? "#181825" : "#1e1e2e"
                    radius: 4

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 8
                        spacing: 8

                        Rectangle {
                            width: 4
                            height: parent.height
                            radius: 2
                            color: {
                                switch (model.provider) {
                                    case "google": return "#4285f4"
                                    case "outlook": return "#00a4ef"
                                    default: return "#a6adc8"
                                }
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 2
                            Text {
                                text: model.title
                                font.pixelSize: 14
                                font.bold: true
                                color: "#cdd6f4"
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                            Text {
                                text: model.start_time + " - " + model.end_time
                                font.pixelSize: 11
                                color: "#6c7086"
                            }
                        }

                        Text {
                            text: model.provider
                            font.pixelSize: 10
                            color: "#585b70"
                        }
                    }
                }
            }
        }
    }

    // Parse JSON from backend into a ListModel
    ListModel { id: eventModel }

    onVisibleChanged: {
        if (visible) backend.refresh_calendar()
    }

    Connections {
        target: backend
        function onCalendar_changed() {
            try {
                var arr = JSON.parse(backend.calendar_json)
                eventModel.clear()
                for (var i = 0; i < arr.length; i++) {
                    var ev = arr[i]
                    var startStr = ev.start || ""
                    var endStr = ev.end || ""
                    eventModel.append({
                        title: ev.title || "Untitled",
                        start_time: startStr.substring(11, 16),
                        end_time: endStr.substring(11, 16),
                        provider: ev.provider || ""
                    })
                }
            } catch(e) { /* no data yet */ }
        }
    }
}
