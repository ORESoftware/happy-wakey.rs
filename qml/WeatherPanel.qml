import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import com.happywakey

Rectangle {
    id: root
    color: "transparent"
    property var theme

    function dayLabel(date) {
        var parsed = new Date(date + "T12:00:00")
        return isNaN(parsed.getTime()) ? date : parsed.toLocaleDateString(Qt.locale(), "ddd")
    }

    function weatherGlyph(code, isDay) {
        if (code === 0) return isDay ? "☀" : "☾"
        if (code <= 2) return "⛅"
        if (code === 3) return "☁"
        if (code === 45 || code === 48) return "≋"
        if ((code >= 51 && code <= 67) || (code >= 80 && code <= 82)) return "☂"
        if ((code >= 71 && code <= 77) || code === 85 || code === 86) return "❄"
        if (code >= 95) return "ϟ"
        return "·"
    }

    function parseForecast(json) {
        try {
            return JSON.parse(json || "[]")
        } catch(error) {
            return []
        }
    }

    function rebuildWeather() {
        try {
            var items = JSON.parse(Backend.weather_json || "[]")
            weatherModel.clear()
            for (var i = 0; i < items.length; i++) {
                var weather = items[i]
                weatherModel.append({
                    name: weather.location_name || "Unknown",
                    temp: Math.round(Number(weather.temperature || 0)),
                    feelsLike: Math.round(Number(weather.feels_like || 0)),
                    condition: weather.condition || "Current conditions",
                    weatherCode: Number(weather.weather_code || 0),
                    wind: Math.round(Number(weather.wind_speed || 0)),
                    humidity: Math.round(Number(weather.humidity || 0)),
                    precipitation: Number(weather.precipitation || 0).toFixed(2),
                    isDay: weather.is_day !== false,
                    observedAt: weather.observed_at || "",
                    sourceName: weather.source || "Weather provider",
                    sourceUrl: weather.source_url || "",
                    forecastJson: JSON.stringify(weather.forecast || []),
                    lat: Number(weather.lat || 0),
                    lon: Number(weather.lon || 0)
                })
            }
        } catch(error) {
            Backend.set_status("Weather data could not be displayed")
        }
    }

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
                    text: "Weather"
                    font.pixelSize: 24
                    font.bold: true
                    color: theme.text
                }
                Text {
                    text: "Current conditions and a five-day outlook for your favorite places"
                    font.pixelSize: 12
                    color: theme.muted
                }
            }

            BusyIndicator {
                running: Backend.weather_loading
                visible: running
                Layout.preferredWidth: 26
                Layout.preferredHeight: 26
            }

            Button {
                text: Backend.weather_loading ? "Refreshing..." : "Refresh"
                enabled: !Backend.weather_loading
                onClicked: Backend.refresh_weather()
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 42
            color: theme.accentSoft
            radius: 6
            visible: weatherModel.count > 0

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 12
                anchors.rightMargin: 8
                spacing: 8

                Text {
                    Layout.fillWidth: true
                    text: "Forecast data from Open-Meteo. Radar opens an interactive precipitation map."
                    color: theme.muted
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }
                Button {
                    text: "About data"
                    flat: true
                    onClicked: Backend.open_url("https://open-meteo.com/")
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.centerIn: parent
                spacing: 10
                visible: weatherModel.count === 0 && !Backend.weather_loading

                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: "No weather locations yet"
                    font.pixelSize: 18
                    font.bold: true
                    color: theme.text
                }
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: "Add up to five locations in Settings. No weather API key is required."
                    font.pixelSize: 12
                    color: theme.muted
                }
            }

            ScrollView {
                id: weatherScroll
                anchors.fill: parent
                clip: true
                visible: weatherModel.count > 0
                ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

                GridLayout {
                    id: weatherGrid
                    width: weatherScroll.availableWidth
                    columns: width >= 940 ? 2 : 1
                    property real cardWidth: Math.max(440,
                        (width - columnSpacing * (columns - 1)) / columns)
                    columnSpacing: 12
                    rowSpacing: 12

                    Repeater {
                        model: weatherModel

                        Rectangle {
                            id: weatherCard
                            property var days: root.parseForecast(model.forecastJson)

                            Layout.preferredWidth: weatherGrid.cardWidth
                            Layout.preferredHeight: 286
                            color: theme.surface
                            radius: 7
                            border.color: theme.border
                            border.width: 1

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 10

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 10

                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        Text {
                                            text: model.name
                                            font.pixelSize: 18
                                            font.bold: true
                                            color: theme.text
                                            elide: Text.ElideRight
                                            Layout.fillWidth: true
                                        }
                                        Text {
                                            text: model.condition
                                            font.pixelSize: 12
                                            color: theme.muted
                                        }
                                    }

                                    Text {
                                        text: root.weatherGlyph(model.weatherCode, model.isDay)
                                        font.pixelSize: 32
                                        color: theme.accent
                                    }

                                    Text {
                                        text: model.temp + "°"
                                        font.pixelSize: 38
                                        font.bold: true
                                        color: theme.text
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 18
                                    Metric { label: "Feels"; value: model.feelsLike + "°"; theme: root.theme }
                                    Metric { label: "Humidity"; value: model.humidity + "%"; theme: root.theme }
                                    Metric { label: "Wind"; value: model.wind + " mph"; theme: root.theme }
                                    Metric { label: "Precip"; value: model.precipitation + " in"; theme: root.theme }
                                }

                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 1
                                    color: theme.border
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 76
                                    spacing: 2

                                    Repeater {
                                        model: weatherCard.days

                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: 2
                                            Text {
                                                Layout.alignment: Qt.AlignHCenter
                                                text: root.dayLabel(modelData.date)
                                                font.pixelSize: 10
                                                font.bold: true
                                                color: theme.muted
                                            }
                                            Text {
                                                Layout.alignment: Qt.AlignHCenter
                                                text: root.weatherGlyph(Number(modelData.weather_code || 0), true)
                                                font.pixelSize: 18
                                                color: theme.accent
                                            }
                                            Text {
                                                Layout.alignment: Qt.AlignHCenter
                                                text: Math.round(Number(modelData.high || 0)) + "°  " + Math.round(Number(modelData.low || 0)) + "°"
                                                font.pixelSize: 10
                                                color: theme.text
                                            }
                                            Text {
                                                Layout.alignment: Qt.AlignHCenter
                                                text: Math.round(Number(modelData.precipitation_probability || 0)) + "%"
                                                font.pixelSize: 9
                                                color: theme.muted
                                            }
                                        }
                                    }

                                    Text {
                                        visible: weatherCard.days.length === 0
                                        text: "Five-day forecast unavailable from fallback provider"
                                        color: theme.muted
                                        font.pixelSize: 11
                                    }
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8
                                    Text {
                                        Layout.fillWidth: true
                                        text: model.sourceName + (model.observedAt ? " · " + model.observedAt.replace("T", " ") : "")
                                        color: theme.faint
                                        font.pixelSize: 10
                                        elide: Text.ElideRight
                                    }
                                    Button {
                                        text: "Radar"
                                        onClicked: Backend.open_url(
                                            "https://www.windy.com/-Weather-radar-radar?radar," + model.lat + "," + model.lon + ",8"
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    ListModel { id: weatherModel }

    component Metric: ColumnLayout {
        property string label: ""
        property string value: ""
        property var theme
        spacing: 1
        Text { text: label; color: theme.faint; font.pixelSize: 9 }
        Text { text: value; color: theme.text; font.pixelSize: 11; font.bold: true }
    }

    onVisibleChanged: {
        if (visible && weatherModel.count === 0) Backend.refresh_weather()
    }

    Component.onCompleted: rebuildWeather()

    Connections {
        target: Backend
        function onWeather_jsonChanged() { rebuildWeather() }
    }
}
