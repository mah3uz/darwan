import QtQuick

QtObject {
    id: auth

    property string password: "test"
    property int delay: 400

    signal succeeded()
    signal failed()

    function authenticate(user, password) {
        timer.pending = password === auth.password
        timer.restart()
    }

    property Timer timer: Timer {
        id: timer
        property bool pending: false
        interval: auth.delay
        onTriggered: pending ? auth.succeeded() : auth.failed()
    }
}
