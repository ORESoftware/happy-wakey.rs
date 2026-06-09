import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "transparent"

    property var theme
    signal navigate(int panel)

    property var cfg: ({})
    property int weatherLocationCount: 0
    property int stockSymbolCount: 0
    property int newsKeywordCount: 0
    property int bookmarkCount: 0

    function parseJson(json, fallback) {
        try {
            if (!json || json.length === 0) return fallback
            return JSON.parse(json)
        } catch(e) {
            return fallback
        }
    }

    function refreshConfig() {
        cfg = parseJson(backend.app_config_json, {})
        weatherLocationCount = cfg.weather_locations ? cfg.weather_locations.length : 0
        stockSymbolCount = cfg.stock_symbols ? cfg.stock_symbols.length : 0
        newsKeywordCount = cfg.news_keywords ? cfg.news_keywords.length : 0
        bookmarkCount = cfg.browser_bookmarks ? cfg.browser_bookmarks.length : 0
        rebuildBookmarks()
    }

    function rebuildCalendar() {
        var arr = parseJson(backend.calendar_json, [])
        calendarModel.clear()
        for (var i = 0; i < Math.min(arr.length, 3); i++) {
            var ev = arr[i]
            var startStr = ev.start || ""
            calendarModel.append({
                title: ev.title || "Untitled",
                meta: (startStr.length >= 16 ? startStr.substring(11, 16) : "Anytime") + " · " + (ev.provider || "calendar")
            })
        }
    }

    function rebuildWeather() {
        var arr = parseJson(backend.weather_json, [])
        weatherModel.clear()
        for (var i = 0; i < Math.min(arr.length, 3); i++) {
            var w = arr[i]
            weatherModel.append({
                title: w.location_name || "Unknown",
                meta: Math.round(w.temperature) + "F · " + (w.condition || "")
            })
        }
    }

    function rebuildStocks() {
        var arr = parseJson(backend.stocks_json, [])
        stocksModel.clear()
        for (var i = 0; i < Math.min(arr.length, 4); i++) {
            var s = arr[i]
            stocksModel.append({
                title: s.symbol || "",
                meta: "$" + Number(s.price || 0).toFixed(2) + " · " + Number(s.change_percent || 0).toFixed(2) + "%"
            })
        }
    }

    function rebuildNews() {
        var arr = parseJson(backend.news_json, [])
        newsModel.clear()
        for (var i = 0; i < Math.min(arr.length, 3); i++) {
            var n = arr[i]
            newsModel.append({
                title: n.title || "",
                meta: n.source || "News"
            })
        }
    }

    function rebuildBookmarks() {
        bookmarkModel.clear()
        var arr = cfg.browser_bookmarks || []
        for (var i = 0; i < Math.min(arr.length, 3); i++) {
            var b = arr[i]
            bookmarkModel.append({
                title: b.title || b.url || "Bookmark",
                meta: b.url || ""
            })
        }
    }

    function refreshAll() {
        backend.refresh_calendar()
        backend.refresh_weather()
        backend.refresh_stocks()
        backend.refresh_news()
    }

    Component.onCompleted: {
        refreshConfig()
        rebuildCalendar()
        rebuildWeather()
        rebuildStocks()
        rebuildNews()
    }

    Connections {
        target: backend
        function onConfig_changed() { refreshConfig() }
        function onCalendar_changed() { rebuildCalendar() }
        function onWeather_changed() { rebuildWeather() }
        function onStocks_changed() { rebuildStocks() }
        function onNews_changed() { rebuildNews() }
    }

    ListModel { id: calendarModel }
    ListModel { id: weatherModel }
    ListModel { id: stocksModel }
    ListModel { id: newsModel }
    ListModel { id: bookmarkModel }

    ColumnLayout {
        anchors.fill: parent
        spacing: 14

        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    text: "Home"
                    font.pixelSize: 22
                    font.bold: true
                    color: theme.text
                }

                Text {
                    text: backend.logged_in ? "Signed in as " + backend.user_email : "Local dashboard preview"
                    font.pixelSize: 12
                    color: theme.muted
                }
            }

            Button {
                text: "Refresh All"
                onClicked: refreshAll()
                flat: true
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            GridLayout {
                id: homeGrid
                width: parent ? parent.width : 0
                columns: width > 980 ? 3 : 2
                columnSpacing: 12
                rowSpacing: 12

                PreviewCard {
                    theme: root.theme
                    Layout.fillWidth: true
                    Layout.preferredHeight: 220
                    title: "Calendar"
                    metric: calendarModel.count > 0 ? calendarModel.count + " loaded" : "Not loaded"
                    detail: backend.logged_in ? "Weekly events and reminders" : "Sign in to pull calendar events"
                    model: calendarModel
                    emptyText: "No events loaded yet."
                    onOpen: root.navigate(1)
                    onRefresh: backend.refresh_calendar()
                }

                PreviewCard {
                    theme: root.theme
                    Layout.fillWidth: true
                    Layout.preferredHeight: 220
                    title: "Weather"
                    metric: weatherLocationCount + " favorites"
                    detail: "Current conditions and Doppler shortcuts"
                    model: weatherModel
                    emptyText: "Add up to 5 locations in Settings."
                    onOpen: root.navigate(2)
                    onRefresh: backend.refresh_weather()
                }

                PreviewCard {
                    theme: root.theme
                    Layout.fillWidth: true
                    Layout.preferredHeight: 220
                    title: "Stocks"
                    metric: stockSymbolCount + " symbols"
                    detail: "Markets, commodities, and securities"
                    model: stocksModel
                    emptyText: "Your watchlist will preview here."
                    onOpen: root.navigate(3)
                    onRefresh: backend.refresh_stocks()
                }

                PreviewCard {
                    theme: root.theme
                    Layout.fillWidth: true
                    Layout.preferredHeight: 220
                    title: "News"
                    metric: newsKeywordCount + " keywords"
                    detail: "Filtered headlines that match your terms"
                    model: newsModel
                    emptyText: "Refresh to load current headlines."
                    onOpen: root.navigate(4)
                    onRefresh: backend.refresh_news()
                }

                PreviewCard {
                    theme: root.theme
                    Layout.fillWidth: true
                    Layout.preferredHeight: 220
                    title: "Browser"
                    metric: bookmarkCount + " shortcuts"
                    detail: "Pinned pages without duplicate tabs"
                    model: bookmarkModel
                    emptyText: "Save important pages in Settings."
                    onOpen: root.navigate(5)
                    onRefresh: rebuildBookmarks()
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 220
                    color: theme.surface
                    radius: 6
                    border.color: theme.border
                    border.width: 1

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8

                        Text {
                            text: "Setup"
                            font.pixelSize: 16
                            font.bold: true
                            color: theme.text
                        }

                        Text {
                            Layout.fillWidth: true
                            text: "Config sync, onboarding state, API keys, and backup repo."
                            color: theme.muted
                            font.pixelSize: 12
                            wrapMode: Text.WordWrap
                        }

                        Text {
                            Layout.fillWidth: true
                            text: cfg.git_repo_path ? cfg.git_repo_path : "No git backup path yet"
                            color: theme.faint
                            font.pixelSize: 11
                            elide: Text.ElideRight
                        }

                        Item { Layout.fillHeight: true }

                        Button {
                            text: "Open Settings"
                            Layout.alignment: Qt.AlignRight
                            onClicked: root.navigate(6)
                        }
                    }
                }
            }
        }
    }

    component PreviewCard: Rectangle {
        id: card

        property string title: ""
        property string metric: ""
        property string detail: ""
        property string emptyText: ""
        property var theme
        property alias model: previewRepeater.model
        signal open()
        signal refresh()

        color: theme.surface
        radius: 6
        border.color: theme.border
        border.width: 1

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 8

            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Text {
                    Layout.fillWidth: true
                    text: card.title
                    font.pixelSize: 16
                    font.bold: true
                    color: theme.text
                }

                Text {
                    text: card.metric
                    font.pixelSize: 11
                    color: theme.muted
                }
            }

            Text {
                Layout.fillWidth: true
                text: card.detail
                color: theme.muted
                font.pixelSize: 12
                wrapMode: Text.WordWrap
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 6

                Repeater {
                    id: previewRepeater

                    delegate: Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 34
                        color: theme.surfaceAlt
                        radius: 4

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: 10
                            anchors.rightMargin: 10
                            spacing: 8

                            Text {
                                Layout.fillWidth: true
                                text: model.title
                                color: theme.text
                                font.pixelSize: 12
                                elide: Text.ElideRight
                            }

                            Text {
                                text: model.meta
                                color: theme.muted
                                font.pixelSize: 11
                                elide: Text.ElideRight
                                Layout.maximumWidth: 130
                            }
                        }
                    }
                }

                Text {
                    Layout.fillWidth: true
                    visible: previewRepeater.count === 0
                    text: card.emptyText
                    color: theme.muted
                    font.pixelSize: 12
                    wrapMode: Text.WordWrap
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                Item { Layout.fillWidth: true }
                Button {
                    text: "Refresh"
                    flat: true
                    onClicked: card.refresh()
                }
                Button {
                    text: "Open"
                    onClicked: card.open()
                }
            }
        }
    }
}
