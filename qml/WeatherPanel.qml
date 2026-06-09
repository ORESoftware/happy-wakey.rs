import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    color: "transparent"

    ColumnLayout {
        anchors.fill: parent
        spacing: 12

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "🌤 Weather"
                font.pixelSize: 22
                font.bold: true
                color: "#cdd6f4"
            }
            Item { Layout.fillWidth: true }
            Button {
                text: "Refresh"
                onClicked: backend.refresh_weather()
                flat: true
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            GridLayout {
                columns: 3
                columnSpacing: 12
                rowSpacing: 12
                width: parent ? parent.width : 0

                Repeater {
                    id: weatherRepeater
                    model: weatherModel

                    Rectangle {
                        Layout.preferredWidth: 240
                        Layout.preferredHeight: 160
                        color: "#1e1e2e"
                        radius: 8
                        border.color: "#313244"
                        border.width: 1

                        ColumnLayout {
                            anchors.fill: parent
                            anchors.margins: 16
                            spacing: 6

                            Text {
                                text: model.name
                                font.pixelSize: 18
                                font.bold: true
                                color: "#cdd6f4"
                            }
                            Text {
                                text: model.condition
                                font.pixelSize: 13
                                color: "#a6adc8"
                            }
                            Item { height: 4 }
                            Text {
                                text: model.temp + "°F"
                                font.pixelSize: 36
                                font.bold: true
                                color: "#f9e2af"
                            }
                            Text {
                                text: "Feels like " + model.feels_like + "°F"
                                font.pixelSize: 11
                                color: "#6c7086"
                            }
                            Text {
                                text: "💨 " + model.wind + " mph  💧 " + model.humidity + "%"
                                font.pixelSize: 11
                                color: "#6c7086"
                            }

                            Item { Layout.fillHeight: true }

                            Button {
                                text: "☁ Doppler Radar"
                                font.pixelSize: 11
                                onClicked: Qt.openUrlExternally(
                                    "https://www.windy.com/?" + model.lat + "," + model.lon
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    ListModel { id: weatherModel }

    onVisibleChanged: {
        if (visible) backend.refresh_weather()
    }

    Connections {
        target: backend
        function onWeather_changed() {
            try {
                var arr = JSON.parse(backend.weather_json)
                weatherModel.clear()
                for (var i = 0; i < arr.length; i++) {
                    var w = arr[i]
                    weatherModel.append({
                        name: w.location_name || "Unknown",
                        temp: Math.round(w.temperature),
                        feels_like: Math.round(w.feels_like),
                        condition: w.condition || "",
                        wind: Math.round(w.wind_speed),
                        humidity: Math.round(w.humidity),
                        lat: w.lat,
                        lon: w.lon
                    })
                }
            } catch(e) {}
        }
    }
}
