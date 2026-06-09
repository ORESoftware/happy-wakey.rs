import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    color: "transparent"
    property var theme

    ColumnLayout {
        anchors.fill: parent
        spacing: 12

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "📰 News"
                font.pixelSize: 22
                font.bold: true
                color: theme.text
            }
            Item { Layout.fillWidth: true }
            Button {
                text: "Refresh"
                onClicked: backend.refresh_news()
                flat: true
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ColumnLayout {
                width: parent ? parent.width : 0
                spacing: 8

                Repeater {
                    model: newsModel

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 96
                        color: theme.surface
                        radius: 6

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: backend.open_url(model.url)
                        }

                        RowLayout {
                            anchors.fill: parent
                            anchors.margins: 14
                            spacing: 12

                            Rectangle {
                                Layout.preferredWidth: 80
                                Layout.preferredHeight: 80
                                Layout.alignment: Qt.AlignVCenter
                                radius: 4
                                color: theme.surfaceAlt
                                clip: true
                                Image {
                                    anchors.fill: parent
                                    source: model.image || ""
                                    fillMode: Image.PreserveAspectCrop
                                    visible: model.image && model.image.length > 0
                                }
                                Text {
                                    anchors.centerIn: parent
                                    text: "📰"
                                    font.pixelSize: 28
                                    visible: !(model.image && model.image.length > 0)
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignVCenter
                                spacing: 4

                                Text {
                                    text: model.title
                                    font.pixelSize: 14
                                    font.bold: true
                                    color: theme.text
                                    wrapMode: Text.WordWrap
                                    maximumLineCount: 2
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                                Text {
                                    text: model.description || ""
                                    font.pixelSize: 12
                                    color: theme.muted
                                    wrapMode: Text.WordWrap
                                    maximumLineCount: 2
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                    Layout.maximumHeight: 32
                                }
                                Text {
                                    text: model.source + " · " + model.date
                                    font.pixelSize: 10
                                    color: theme.faint
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ListModel { id: newsModel }

    onVisibleChanged: {
        if (visible) backend.refresh_news()
    }

    Connections {
        target: backend
        function onNews_changed() {
            try {
                var arr = JSON.parse(backend.news_json)
                newsModel.clear()
                for (var i = 0; i < arr.length; i++) {
                    var n = arr[i]
                    newsModel.append({
                        title: n.title || "",
                        description: n.description || "",
                        source: n.source || "",
                        url: n.url || "",
                        image: n.url_to_image || "",
                        date: n.published_at ? n.published_at.substring(0, 10) : ""
                    })
                }
            } catch(e) {}
        }
    }
}
