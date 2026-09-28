import QtQuick
import Darwan

MpvVideo {
    readonly property Item output: parent ? parent.parent : null

    source: output ? output.source : ""
    playing: output ? output.playing : false
    loop: output ? output.loop : true
    fillMode: output ? output.fillMode : MpvVideo.PreserveAspectFit
    // The host (ThemeHost) that holds the theme says which tier to play; anywhere else it is "full".
    tier: {
        for (let p = output; p; p = p.parent) {
            if (p.mediaTier !== undefined)
                return p.mediaTier
        }
        return "full"
    }
    onFailed: reason => output.fail(reason)
}
