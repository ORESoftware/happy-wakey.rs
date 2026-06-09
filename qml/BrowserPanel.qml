import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtWebEngine

Rectangle {
    color: "transparent"

    property var tabs: []
    property int currentIndex: -1

    ColumnLayout {
        anchors.fill: parent
        spacing: 6

        // Toolbar
        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Text {
                text: "🌐 Quick Browser"
                font.pixelSize: 22
                font.bold: true
                color: "#cdd6f4"
            }
            Item { Layout.fillWidth: true }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                color: "#1e1e2e"
                radius: 4
                border.color: "#313244"
                border.width: 1

                TextField {
                    id: urlInput
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    verticalAlignment: TextInput.AlignVCenter
                    color: "#cdd6f4"
                    font.pixelSize: 13
                    selectByMouse: true
                    onAccepted: {
                        var url = text.trim()
                        if (url.length === 0) return
                        if (url.indexOf("://") === -1) url = "https://" + url
                        addTab(url)
                    }
                    placeholderText: "Enter URL and press Enter…"

                }
            }

            Button {
                text: "Go"
                onClicked: urlInput.accepted()
                flat: true
            }
            Button {
                text: "➕"
                flat: true
                onClicked: addTab("about:blank")
            }
        }

        // Tab bar
        TabBar {
            id: tabBar
            Layout.fillWidth: true
            visible: tabs.length > 0

            Repeater {
                model: tabs

                TabButton {
                    text: modelData.title.length > 20 ? modelData.title.substring(0, 18) + "…" : modelData.title
                    width: 140

                    contentItem: RowLayout {
                        spacing: 4
                        Text {
                            Layout.fillWidth: true
                            text: parent.parent.text
                            font.pixelSize: 11
                            elide: Text.ElideRight
                            color: tabBar.currentIndex === index ? "#cdd6f4" : "#6c7086"
                            horizontalAlignment: Text.AlignHCenter
                        }
                        Text {
                            text: "✕"
                            font.pixelSize: 10
                            color: "#585b70"
                            MouseArea {
                                anchors.fill: parent
                                onClicked: closeTab(index)
                            }
                        }
                    }
                }
            }
        }

        // Web view stack
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Repeater {
                model: tabs

                WebEngineView {
                    visible: tabBar.currentIndex === index
                    anchors.fill: parent
                    url: modelData.url
                    onTitleChanged: {
                        if (visible) {
                            tabs[index].title = title
                            tabBar.itemAt(index).text = title.length > 20 ? title.substring(0, 18) + "…" : title
                        }
                    }
                }
            }
        }
    }

    function addTab(url) {
        // Deduplicate: if url already open, switch to it
        for (var i = 0; i < tabs.length; i++) {
            if (tabs[i].url.toString() === url) {
                tabBar.currentIndex = i
                return
            }
        }

        tabs.push({ title: "Loading…", url: url })
        tabBar.currentIndex = tabs.length - 1
    }

    function closeTab(index) {
        if (index < 0 || index >= tabs.length) return
        tabs.splice(index, 1)
        if (tabBar.currentIndex >= tabs.length)
            tabBar.currentIndex = tabs.length - 1
    }

    onVisibleChanged: {
        if (visible) {
            // Check config for bookmarks
            try {
                var cfg = JSON.parse(backend.app_config_json)
                if (cfg.browser_bookmarks && cfg.browser_bookmarks.length > 0) {
                    for (var i = 0; i < cfg.browser_bookmarks.length; i++) {
                        // Only add if not already open
                        var found = false
                        for (var j = 0; j < tabs.length; j++) {
                            if (tabs[j].url.toString() === cfg.browser_bookmarks[i].url) {
                                found = true
                                break
                            }
                        }
                        if (!found) {
                            addTab(cfg.browser_bookmarks[i].url)
                        }
                    }
                }
            } catch(e) {}
        }
    }
}
