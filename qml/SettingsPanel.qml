import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: root
    color: "transparent"
    property var theme

    ColumnLayout {
        anchors.fill: parent
        spacing: 16

        Text {
            text: "⚙ Settings"
            font.pixelSize: 22
            font.bold: true
            color: theme.text
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            ColumnLayout {
                width: parent ? parent.width : 0
                spacing: 20

                // ---- Account Section ----
                SectionBox {
                    title: "Account"
                    Layout.fillWidth: true

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8

                        Text {
                            text: backend.logged_in
                                ? "Signed in as " + backend.user_email
                                : "Not signed in"
                            font.pixelSize: 14
                            color: theme.text
                        }

                        RowLayout {
                            spacing: 8
                            Button {
                                text: "Sign in with Google"
                                visible: !backend.logged_in
                                onClicked: backend.login("google")
                            }
                            Button {
                                text: "Sign in with Apple"
                                visible: !backend.logged_in
                                onClicked: backend.login("apple")
                            }
                            Button {
                                text: "Sign in with Microsoft"
                                visible: !backend.logged_in
                                onClicked: backend.login("microsoft")
                            }
                            Button {
                                text: "Sign Out"
                                visible: backend.logged_in
                                onClicked: backend.logout()
                            }
                        }
                    }
                }

                // ---- Weather Locations ----
                SectionBox {
                    title: "Weather Locations (max 5)"
                    Layout.fillWidth: true

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8

                        ListView {
                            id: weatherLocList
                            Layout.fillWidth: true
                            Layout.preferredHeight: 120
                            model: weatherLocModel
                            delegate: RowLayout {
                                width: weatherLocList.width
                                spacing: 8
                                Text {
                                    text: model.name + " (" + model.lat + ", " + model.lon + ")"
                                    color: theme.text
                                    font.pixelSize: 13
                                    Layout.fillWidth: true
                                }
                                Button {
                                    text: "✕"
                                    flat: true
                                    onClicked: weatherLocModel.remove(index)
                                }
                            }
                        }

                        RowLayout {
                            spacing: 4
                            TextField {
                                id: locName
                                placeholderText: "Name"
                                Layout.preferredWidth: 100
                            }
                            TextField {
                                id: locLat
                                placeholderText: "Lat"
                                Layout.preferredWidth: 80
                                validator: DoubleValidator {}
                            }
                            TextField {
                                id: locLon
                                placeholderText: "Lon"
                                Layout.preferredWidth: 80
                                validator: DoubleValidator {}
                            }
                            Button {
                                text: "Add"
                                onClicked: {
                                    if (weatherLocModel.count >= 5) return
                                    if (locName.text && locLat.text && locLon.text) {
                                        weatherLocModel.append({
                                            name: locName.text,
                                            lat: locLat.text,
                                            lon: locLon.text
                                        })
                                        locName.text = ""
                                        locLat.text = ""
                                        locLon.text = ""
                                    }
                                }
                            }
                        }
                    }
                }

                // ---- Stock Symbols ----
                SectionBox {
                    title: "Stock Symbols (max 20)"
                    Layout.fillWidth: true

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8

                        ListView {
                            id: stockList
                            Layout.fillWidth: true
                            Layout.preferredHeight: 120
                            model: stockSymbolModel
                            delegate: RowLayout {
                                width: stockList.width
                                spacing: 8
                                Text {
                                    text: model.symbol
                                    color: theme.text
                                    font.pixelSize: 13
                                    Layout.fillWidth: true
                                }
                                Button {
                                    text: "✕"
                                    flat: true
                                    onClicked: stockSymbolModel.remove(index)
                                }
                            }
                        }

                        RowLayout {
                            spacing: 4
                            TextField {
                                id: newSymbol
                                placeholderText: "Symbol (e.g. AAPL)"
                                Layout.preferredWidth: 120
                            }
                            Button {
                                text: "Add"
                                onClicked: {
                                    if (stockSymbolModel.count >= 20) return
                                    var sym = newSymbol.text.trim().toUpperCase()
                                    if (sym) {
                                        stockSymbolModel.append({ symbol: sym })
                                        newSymbol.text = ""
                                    }
                                }
                            }
                        }
                    }
                }

                // ---- News Keywords ----
                SectionBox {
                    title: "News Keywords"
                    Layout.fillWidth: true

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8

                        Flow {
                            Layout.fillWidth: true
                            spacing: 6
                            Repeater {
                                model: newsKeywordModel
                                Rectangle {
                                    height: 28
                                    width: keywordLabel.implicitWidth + 20
                                    color: theme.border
                                    radius: 4
                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 6
                                        anchors.rightMargin: 4
                                        spacing: 4
                                        Text {
                                            id: keywordLabel
                                            text: model.keyword
                                            color: theme.text
                                            font.pixelSize: 12
                                        }
                                        Text {
                                            text: "✕"
                                            font.pixelSize: 10
                                            color: theme.faint
                                            MouseArea {
                                                anchors.fill: parent
                                                onClicked: newsKeywordModel.remove(index)
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        RowLayout {
                            spacing: 4
                            TextField {
                                id: newKeyword
                                placeholderText: "Add keyword…"
                                Layout.fillWidth: true
                            }
                            Button {
                                text: "Add"
                                onClicked: {
                                    var kw = newKeyword.text.trim()
                                    if (kw) {
                                        newsKeywordModel.append({ keyword: kw })
                                        newKeyword.text = ""
                                    }
                                }
                            }
                        }
                    }
                }

                // ---- Bookmarks (Quick Browser URLs) ----
                SectionBox {
                    title: "Browser Bookmarks"
                    Layout.fillWidth: true

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8

                        ListView {
                            id: bookmarkList
                            Layout.fillWidth: true
                            Layout.preferredHeight: 100
                            model: bookmarkModel
                            delegate: RowLayout {
                                width: bookmarkList.width
                                spacing: 8
                                Text {
                                    text: model.title + " (" + model.url + ")"
                                    color: theme.text
                                    font.pixelSize: 12
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }
                                Button {
                                    text: "✕"
                                    flat: true
                                    onClicked: bookmarkModel.remove(index)
                                }
                            }
                        }

                        RowLayout {
                            spacing: 4
                            TextField {
                                id: bmTitle
                                placeholderText: "Title"
                                Layout.preferredWidth: 120
                            }
                            TextField {
                                id: bmUrl
                                placeholderText: "URL"
                                Layout.fillWidth: true
                            }
                            Button {
                                text: "Add"
                                onClicked: {
                                    if (bmTitle.text && bmUrl.text) {
                                        bookmarkModel.append({
                                            id: "",
                                            title: bmTitle.text,
                                            url: bmUrl.text
                                        })
                                        bmTitle.text = ""
                                        bmUrl.text = ""
                                    }
                                }
                            }
                        }
                    }
                }

                // ---- Git Backup ----
                SectionBox {
                    title: "Git Backup (Optional)"
                    Layout.fillWidth: true

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 16
                        spacing: 8

                        Text {
                            text: "Back up your config to a private git repo."
                            font.pixelSize: 12
                            color: theme.muted
                        }

                        RowLayout {
                            spacing: 4
                            TextField {
                                id: gitRepoPath
                                placeholderText: "git@github.com:user/repo.git or /local/path"
                                Layout.fillWidth: true
                                text: {
                                    try {
                                        var cfg = JSON.parse(backend.app_config_json)
                                        return cfg.git_repo_path || ""
                                    } catch(e) { return "" }
                                }
                            }
                            Button {
                                text: "Save & Sync"
                                onClicked: {
                                    backend.set_status("Git sync not yet implemented in this build")
                                }
                            }
                        }
                    }
                }

                // ---- Save / Reset ----
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    Item { Layout.fillWidth: true }
                    Button {
                        text: "Save Settings"
                        highlighted: true
                        onClicked: saveAllSettings()
                    }
                    Button {
                        text: "Reset to Defaults"
                        onClicked: {
                            var defaults = {
                                version: "0.1.0",
                                user_id: backend.user_id,
                                supabase_session: null,
                                calendar_providers: [],
                                weather_locations: [],
                                stock_symbols: ["AAPL","GOOGL","MSFT","AMZN","NVDA","META","TSLA","SPY","QQQ","GLD","BTC-USD","ETH-USD","JPM","V","KO","DIS","NFLX","BA","XOM","PG"],
                                news_keywords: ["technology","AI","markets"],
                                browser_bookmarks: [],
                                git_repo_path: "",
                                supabase_sync_enabled: true,
                                onboarding: {
                                    completed: false,
                                    current_step: "welcome",
                                    step_index: 0,
                                    updated_at: null
                                }
                            }
                            var savedCfg = JSON.stringify(defaults)
                            backend.save_config(savedCfg)
                            backend.reload_config()
                        }
                    }
                }

                Item { height: 32 }
            }
        }
    }

    // Helper to collect and save all settings
    function saveAllSettings() {
        try {
            var cfg = JSON.parse(backend.app_config_json)

            // Weather locations
            var locs = []
            for (var i = 0; i < weatherLocModel.count; i++) {
                locs.push({
                    name: weatherLocModel.get(i).name,
                    lat: parseFloat(weatherLocModel.get(i).lat),
                    lon: parseFloat(weatherLocModel.get(i).lon)
                })
            }
            cfg.weather_locations = locs

            // Stock symbols
            var syms = []
            for (var j = 0; j < stockSymbolModel.count; j++) {
                syms.push(stockSymbolModel.get(j).symbol)
            }
            cfg.stock_symbols = syms

            // News keywords
            var kws = []
            for (var k = 0; k < newsKeywordModel.count; k++) {
                kws.push(newsKeywordModel.get(k).keyword)
            }
            cfg.news_keywords = kws

            // Bookmarks
            var bms = []
            for (var m = 0; m < bookmarkModel.count; m++) {
                bms.push({
                    id: bookmarkModel.get(m).id || "",
                    title: bookmarkModel.get(m).title,
                    url: bookmarkModel.get(m).url
                })
            }
            cfg.browser_bookmarks = bms

            // Git repo
            cfg.git_repo_path = gitRepoPath.text || ""

            backend.save_config(JSON.stringify(cfg))
            backend.reload_config()
        } catch(e) {
            backend.set_status("Save error: " + e)
        }
    }

    // ---- Models ----
    ListModel { id: weatherLocModel }
    ListModel { id: stockSymbolModel }
    ListModel { id: newsKeywordModel }
    ListModel { id: bookmarkModel }

    // Load existing config when panel becomes visible
    onVisibleChanged: {
        if (!visible) return

        try {
            var cfg = JSON.parse(backend.app_config_json)

            weatherLocModel.clear()
            if (cfg.weather_locations) {
                for (var i = 0; i < cfg.weather_locations.length; i++) {
                    var w = cfg.weather_locations[i]
                    weatherLocModel.append({
                        name: w.name,
                        lat: String(w.lat),
                        lon: String(w.lon)
                    })
                }
            }

            stockSymbolModel.clear()
            if (cfg.stock_symbols) {
                for (var j = 0; j < cfg.stock_symbols.length; j++) {
                    stockSymbolModel.append({ symbol: cfg.stock_symbols[j] })
                }
            }

            newsKeywordModel.clear()
            if (cfg.news_keywords) {
                for (var k = 0; k < cfg.news_keywords.length; k++) {
                    newsKeywordModel.append({ keyword: cfg.news_keywords[k] })
                }
            }

            bookmarkModel.clear()
            if (cfg.browser_bookmarks) {
                for (var m = 0; m < cfg.browser_bookmarks.length; m++) {
                    var b = cfg.browser_bookmarks[m]
                    bookmarkModel.append({
                        id: b.id || "",
                        title: b.title || "",
                        url: b.url || ""
                    })
                }
            }
        } catch(e) {}
    }

    // ---- Section Box Component ----
    component SectionBox: Rectangle {
        property string title: ""
        property var panelTheme: root.theme

        color: panelTheme.surface
        radius: 6
        border.color: panelTheme.border
        border.width: 1
        implicitHeight: 200

        Text {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.topMargin: 8
            anchors.leftMargin: 12
            text: title
            font.pixelSize: 13
            font.bold: true
            color: panelTheme.muted
        }
    }
}
