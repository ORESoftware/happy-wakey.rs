import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtWebEngine

Rectangle {
    color: "transparent"
    property var theme

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
                color: theme.text
            }
            Item { Layout.fillWidth: true }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                color: theme.surface
                radius: 4
                border.color: theme.border
                border.width: 1

                TextField {
                    id: urlInput
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    verticalAlignment: TextInput.AlignVCenter
                    color: theme.text
                    font.pixelSize: 13
                    selectByMouse: true
                    onAccepted: {
                        var url = normalizeBrowserUrl(text)
                        if (!url) {
                            backend.set_status("Only http and https URLs can be opened in the browser")
                            return
                        }
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
                            color: tabBar.currentIndex === index ? theme.text : theme.muted
                            horizontalAlignment: Text.AlignHCenter
                        }
                        Text {
                            text: "✕"
                            font.pixelSize: 10
                            color: theme.faint
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
        url = normalizeBrowserUrl(url)
        if (!url) return

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

    function normalizeBrowserUrl(raw) {
        var url = String(raw || "").trim()
        if (url.length === 0) return null
        if (url === "about:blank") return url
        if (url.indexOf("://") === -1) url = "https://" + url
        if (url.indexOf("https://") !== 0 && url.indexOf("http://") !== 0) return null
        return url
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
