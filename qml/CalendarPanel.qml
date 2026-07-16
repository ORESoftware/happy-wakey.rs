import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import com.happywakey

Rectangle {
    id: root
    color: "transparent"
    property var theme
    property var agenda: ({
        date_label: "Today",
        headline: "Refresh to build your daily agenda",
        total_events: 0,
        remaining_events: 0,
        meeting_minutes: 0,
        conflict_count: 0
    })

    function parseJson(value, fallback) {
        try {
            return value && value.length > 0 ? JSON.parse(value) : fallback
        } catch (error) {
            return fallback
        }
    }

    function rebuildAgenda() {
        agenda = parseJson(Backend.calendar_agenda_json, agenda)
    }

    function rebuildEvents() {
        var events = parseJson(Backend.calendar_json, [])
        eventModel.clear()
        for (var i = 0; i < events.length; i++) {
            var event = events[i]
            var joinUrl = event.join_url || ""
            var eventUrl = event.event_url || ""
            eventModel.append({
                title: event.title || "Untitled",
                time_label: event.time_label || "Anytime",
                day_label: event.day_label || "Upcoming",
                provider: event.provider || "calendar",
                location: event.location || "",
                action_url: joinUrl || eventUrl,
                action_label: joinUrl ? "Join" : (eventUrl ? "Open" : ""),
                all_day: event.all_day === true
            })
        }
    }

    function weekDays() {
        var today = new Date()
        today.setHours(0, 0, 0, 0)
        var monday = new Date(today)
        monday.setDate(today.getDate() - ((today.getDay() + 6) % 7))
        var days = []
        var names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        for (var i = 0; i < 7; i++) {
            var date = new Date(monday)
            date.setDate(monday.getDate() + i)
            days.push({
                label: names[i],
                date: String(date.getDate()),
                today: date.getTime() === today.getTime()
            })
        }
        return days
    }

    Component.onCompleted: {
        rebuildAgenda()
        rebuildEvents()
    }

    onVisibleChanged: {
        if (visible && eventModel.count === 0) Backend.refresh_calendar()
    }

    Connections {
        target: Backend
        function onCalendar_jsonChanged() { root.rebuildEvents() }
        function onCalendar_agenda_jsonChanged() { root.rebuildAgenda() }
    }

    ListModel { id: eventModel }

    ColumnLayout {
        anchors.fill: parent
        spacing: 12

        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    text: "Calendar"
                    font.pixelSize: 22
                    font.bold: true
                    color: theme.text
                }
                Text {
                    text: root.agenda.date_label || "This week"
                    font.pixelSize: 12
                    color: theme.muted
                }
            }

            BusyIndicator {
                running: Backend.calendar_loading
                visible: running
                Layout.preferredWidth: 24
                Layout.preferredHeight: 24
            }
            Button {
                text: "Test reminder"
                enabled: !Backend.calendar_loading
                onClicked: Backend.test_notification()
            }
            Button {
                text: Backend.calendar_loading ? "Refreshing..." : "Refresh"
                enabled: !Backend.calendar_loading
                highlighted: true
                onClicked: Backend.refresh_calendar()
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 108
            color: theme.accentSoft
            radius: 6

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 18

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 6

                    Text {
                        Layout.fillWidth: true
                        text: root.agenda.headline || "Your day is clear"
                        font.pixelSize: 17
                        font.bold: true
                        color: theme.text
                        wrapMode: Text.WordWrap
                    }
                    Text {
                        Layout.fillWidth: true
                        text: root.agenda.next_event_title
                            ? (root.agenda.minutes_until_next !== null
                                ? root.agenda.minutes_until_next + " minutes until " + root.agenda.next_event_title
                                : root.agenda.next_event_title)
                            : "No timed event is waiting"
                        font.pixelSize: 12
                        color: theme.muted
                        elide: Text.ElideRight
                    }
                }

                AgendaMetric {
                    value: String(root.agenda.remaining_events || 0)
                    label: "remaining"
                }
                AgendaMetric {
                    value: String(root.agenda.meeting_minutes || 0)
                    label: "minutes"
                }
                AgendaMetric {
                    value: String(root.agenda.conflict_count || 0)
                    label: "conflicts"
                    warning: (root.agenda.conflict_count || 0) > 0
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Repeater {
                model: root.weekDays()

                Rectangle {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: 48
                    color: modelData.today ? theme.selected : theme.surface
                    radius: 4
                    border.color: modelData.today ? theme.accent : theme.border
                    border.width: 1

                    Column {
                        anchors.centerIn: parent
                        spacing: 1
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: modelData.label
                            font.pixelSize: 11
                            font.bold: true
                            color: modelData.today ? theme.accent : theme.muted
                        }
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: modelData.date
                            font.pixelSize: 13
                            color: theme.text
                        }
                    }
                }
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

            ListView {
                id: eventList
                width: parent ? parent.width : 0
                spacing: 4
                model: eventModel
                section.property: "day_label"
                section.criteria: ViewSection.FullString

                section.delegate: Rectangle {
                    required property string section
                    width: eventList.width
                    height: 38
                    color: theme.page

                    Text {
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        text: parent.section
                        font.pixelSize: 12
                        font.bold: true
                        color: theme.muted
                    }
                }

                delegate: Rectangle {
                    required property int index
                    required property string title
                    required property string time_label
                    required property string provider
                    required property string location
                    required property string action_url
                    required property string action_label
                    width: eventList.width
                    height: location ? 72 : 62
                    color: index % 2 === 0 ? theme.surface : theme.surfaceAlt
                    radius: 4
                    border.color: theme.border
                    border.width: 1

                    RowLayout {
                        anchors.fill: parent
                        anchors.margins: 10
                        spacing: 10

                        Rectangle {
                            Layout.preferredWidth: 4
                            Layout.fillHeight: true
                            radius: 2
                            color: provider === "google"
                                ? "#4285f4"
                                : (provider === "outlook" ? "#0078d4" : theme.muted)
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 3

                            Text {
                                Layout.fillWidth: true
                                text: title
                                font.pixelSize: 14
                                font.bold: true
                                color: theme.text
                                elide: Text.ElideRight
                            }
                            Text {
                                Layout.fillWidth: true
                                text: location
                                    ? time_label + "  ·  " + location
                                    : time_label
                                font.pixelSize: 11
                                color: theme.muted
                                elide: Text.ElideRight
                            }
                        }

                        Text {
                            text: provider === "outlook" ? "Microsoft" : "Google"
                            font.pixelSize: 10
                            color: theme.faint
                        }

                        Button {
                            visible: action_url.length > 0
                            text: action_label
                            onClicked: Backend.open_url(action_url)
                        }
                    }
                }
            }
        }

        Label {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: eventModel.count === 0 && !Backend.calendar_loading
            text: Backend.logged_in
                ? "No events are scheduled in this week."
                : "Sign in with Google or Microsoft to load your calendar."
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            color: theme.muted
        }
    }

    component AgendaMetric: ColumnLayout {
        property string value: "0"
        property string label: ""
        property bool warning: false
        spacing: 1
        Layout.preferredWidth: 72

        Text {
            Layout.alignment: Qt.AlignHCenter
            text: parent.value
            font.pixelSize: 20
            font.bold: true
            color: parent.warning ? theme.warning : theme.text
        }
        Text {
            Layout.alignment: Qt.AlignHCenter
            text: parent.label
            font.pixelSize: 10
            color: theme.muted
        }
    }
}
