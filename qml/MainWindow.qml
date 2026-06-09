import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    visible: true
    width: 1280
    height: 860
    title: "Happy Wakey"
    minimumWidth: 900
    minimumHeight: 600

    // ---- Backend context property (set from Rust) ----

    property bool firstRun: backend.app_config_json === "{}" || backend.app_config_json.length < 10

    Component.onCompleted: {
        if (firstRun) {
            stack.currentIndex = 5 // onboarding
        }
    }

    // ---- Status Bar ----

    footer: ToolBar {
        height: 28
        Label {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: 8
            text: backend.status_msg
            font.pixelSize: 11
            color: "#888"
        }
        Label {
            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: 8
            text: backend.logged_in ? "✓ " + backend.user_email : "Not logged in"
            font.pixelSize: 11
            color: backend.logged_in ? "#2a2" : "#a88"
        }
    }

    // ---- Main Layout ----

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Sidebar
        Rectangle {
            id: sidebar
            Layout.preferredWidth: 200
            Layout.fillHeight: true
            color: "#1e1e2e"

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // App title
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 56
                    color: "#181825"

                    Text {
                        anchors.centerIn: parent
                        text: "☀ Happy Wakey"
                        font.pixelSize: 18
                        font.bold: true
                        color: "#cdd6f4"
                    }
                }

                // Nav buttons
                Repeater {
                    model: [
                        { icon: "📅", label: "Calendar",     panel: 0 },
                        { icon: "🌤", label: "Weather",      panel: 1 },
                        { icon: "📈", label: "Stocks",       panel: 2 },
                        { icon: "📰", label: "News",         panel: 3 },
                        { icon: "🌐", label: "Browser",      panel: 4 },
                        { icon: "⚙",  label: "Settings",     panel: 5 },
                    ]

                    ItemDelegate {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 44
                        highlighted: stack.currentIndex === modelData.panel

                        contentItem: RowLayout {
                            spacing: 10
                            Text {
                                text: modelData.icon
                                font.pixelSize: 16
                            }
                            Text {
                                text: modelData.label
                                font.pixelSize: 14
                                color: highlighted ? "#cdd6f4" : "#6c7086"
                            }
                        }

                        background: Rectangle {
                            color: highlighted ? "#313244" : "transparent"
                        }

                        onClicked: stack.currentIndex = modelData.panel
                    }
                }

                Item { Layout.fillHeight: true }
            }
        }

        // Content area
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: "#11111b"

            StackLayout {
                id: stack
                anchors.fill: parent
                anchors.margins: 16

                CalendarPanel {}
                WeatherPanel {}
                StocksPanel {}
                NewsPanel {}
                BrowserPanel {}
                SettingsPanel {}
            }
        }
    }
}
