import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import com.happywakey
import QtWebEngine

Rectangle {
    id: root
    color: "transparent"
    property var theme

    // Tabs live in a ListModel: a plain `var` array mutated in place (push/splice)
    // does not notify QML, so Repeaters bound to it never refresh.
    ListModel { id: tabsModel }

    function tabLabel(title) {
        var t = (title && title.length > 0) ? title : "Loading…"
        return t.length > 20 ? t.substring(0, 18) + "…" : t
    }

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
                            Backend.set_status("Only http and https URLs can be opened in the browser")
                            return
                        }
                        addTab(url)
                        text = ""
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
            visible: tabsModel.count > 0

            Repeater {
                model: tabsModel

                TabButton {
                    width: 140

                    contentItem: RowLayout {
                        spacing: 4
                        Text {
                            Layout.fillWidth: true
                            text: tabLabel(model.title)
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

            // Friendly empty state when no tabs are open.
            Text {
                anchors.centerIn: parent
                visible: tabsModel.count === 0
                text: "Type a URL above or open a bookmark to start browsing."
                color: theme.muted
                font.pixelSize: 14
            }

            Repeater {
                model: tabsModel

                WebEngineView {
                    anchors.fill: parent
                    visible: tabBar.currentIndex === index
                    url: model.url
                    onTitleChanged: tabsModel.setProperty(index, "title", title || "Untitled")
                }
            }
        }
    }

    function addTab(url) {
        url = normalizeBrowserUrl(url)
        if (!url) return

        // Deduplicate: if the URL is already open, just switch to it.
        for (var i = 0; i < tabsModel.count; i++) {
            if (tabsModel.get(i).url === url) {
                tabBar.currentIndex = i
                return
            }
        }

        tabsModel.append({ title: "Loading…", url: url })
        tabBar.currentIndex = tabsModel.count - 1
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
        if (index < 0 || index >= tabsModel.count) return
        tabsModel.remove(index)
        if (tabBar.currentIndex >= tabsModel.count)
            tabBar.currentIndex = tabsModel.count - 1
    }

    onVisibleChanged: {
        if (!visible) return
        // Open saved bookmarks as tabs (skipping any already open).
        try {
            var cfg = JSON.parse(Backend.app_config_json)
            var marks = cfg.browser_bookmarks || []
            for (var i = 0; i < marks.length; i++) {
                addTab(marks[i].url)
            }
        } catch(e) {}
    }
}
