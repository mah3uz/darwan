import QtQuick
import QtTest
import "../../../../runtime"
import "../../../../runtime/contract/Ini.js" as Ini
import "generated/expected.js" as Expected

TestCase {
    id: tc
    name: "Runtime"
    width: 400
    height: 300

    readonly property string fixtures: Qt.resolvedUrl("fixtures").toString().replace("file://", "")

    Component {
        id: hostComponent
        ThemeHost {
            width: 400
            height: 300
            userName: "owner"
            machineName: "box"
            authBackend: QtObject {
                property string lastUser: ""
                signal succeeded()
                signal failed()
                function authenticate(user, password) {
                    lastUser = user
                    password === "right" ? succeeded() : failed()
                }
            }
        }
    }

    function makeHost(props) {
        var host = createTemporaryObject(hostComponent, tc, props)
        tryVerify(() => host.themeReady || host.usingFallback, 5000)
        return host
    }

    function readFile(url) {
        var xhr = new XMLHttpRequest()
        xhr.open("GET", url, false)
        xhr.send()
        return xhr.responseText
    }

    function test_js_parser_reads_exactly_what_rust_writes() {
        compare(Ini.parseGeneral(readFile(Qt.resolvedUrl("generated/overlay.conf"))), Expected.values)
    }

    function test_overlay_overrides_and_empty_values_do_not() {
        var host = makeHost({ themePath: fixtures + "/good", overlayPath: fixtures + "/overlay-light.conf" })
        compare(host.themeItem.mode, "light")
        compare(host.config.base, "kept", "SDDM ignores empty theme.conf.user values; the lock must too")
    }

    function test_theme_gets_a_keyboard_object() {
        var host = makeHost({ themePath: fixtures + "/good" })
        verify(host.themeItem.sawKeyboard)
        verify(host.keyboard.numLock)
    }

    function test_lock_mode_hides_hostname_and_authenticates_only_the_owner() {
        var host = makeHost({ themePath: fixtures + "/good", hostMode: "lock" })
        compare(host.sddm.hostName, undefined, "38 themes read an undefined hostName as 'on the lockscreen'")
        host.sddm.login("root", "right", 0)
        compare(host.authBackend.lastUser, "owner")
    }

    function test_sddm_mode_exposes_hostname_and_the_chosen_user() {
        var host = makeHost({ themePath: fixtures + "/good", hostMode: "sddm" })
        compare(host.sddm.hostName, "box")
        host.sddm.login("guest", "right", 0)
        compare(host.authBackend.lastUser, "guest")
    }

    function test_login_result_reaches_the_theme_signals() {
        var host = makeHost({ themePath: fixtures + "/good" })
        var ok = 0, bad = 0
        host.sddm.loginSucceeded.connect(() => ok++)
        host.sddm.loginFailed.connect(() => bad++)
        host.sddm.login("", "wrong", 0)
        host.sddm.login("", "right", 0)
        compare(bad, 1)
        compare(ok, 1)
    }

    function test_a_broken_theme_falls_back_to_a_password_prompt() {
        var host = makeHost({ themePath: fixtures + "/broken" })
        verify(host.usingFallback, "the lock must always be unlockable")
        verify(!host.themeReady)
    }
}
