import QtQuick
import Quickshell.Services.Pam

PamContext {
    id: pam

    property string pending: ""

    signal succeeded()
    signal failed()

    function authenticate(user, password) {
        if (active)
            return
        pending = password
        pam.user = user
        if (!pam.start())
            failed()
    }

    onResponseRequiredChanged: {
        if (responseRequired) {
            respond(pending)
            pending = ""
        }
    }

    onCompleted: result => result === PamResult.Success ? succeeded() : failed()
}
