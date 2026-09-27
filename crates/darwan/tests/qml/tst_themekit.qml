import QtQuick
import QtTest
import "../../../../runtime/theme-kit/darwan"

TestCase {
    id: tc
    name: "ThemeKit"
    width: 200
    height: 120

    // Custom and Background find `config` through their creation context, as themes do under ThemeHost.
    property var config: ({})

    readonly property string generated: Qt.resolvedUrl("generated").toString().replace("file://", "")

    Component { id: customComponent; Custom {} }
    Component { id: backgroundComponent; Background { width: 200; height: 120 } }

    function custom(cfg) {
        tc.config = cfg
        return createTemporaryObject(customComponent, tc)
    }

    function test_theme_values_stand_when_nothing_is_set() {
        const c = custom({})
        compare(c.dur(400), 400)
        compare(c.ease(Easing.OutQuad), Easing.OutQuad)
        compare(c.color("accent", "#e6bb5c"), "#e6bb5c")
        compare(c.font("text", "Pixel"), "Pixel")
        compare(c.dark, false)
    }

    function test_speed_scales_transitions_and_reduce_motion_removes_them() {
        compare(custom({ animSpeed: "2" }).dur(400), 200)
        compare(custom({ animSpeed: "0.5" }).dur(400), 800)
        compare(custom({ animSpeed: "2", reduceMotion: "true" }).dur(400), 0)
        compare(custom({ animSpeed: "nonsense" }).dur(400), 400)
    }

    function test_the_users_curve_replaces_the_themes_and_unknown_names_are_ignored() {
        compare(custom({ animEasing: "OutExpo" }).ease(Easing.Linear), Easing.OutExpo)
        compare(custom({ animEasing: "Wobble" }).ease(Easing.Linear), Easing.Linear)
    }

    function test_colours_and_fonts_by_role() {
        const c = custom({ colorAccent: "#ff0000", colorText: "#eeeeee", colorLamp: "#123456", fontClock: "Inter", colorScheme: "dark" })
        compare(c.color("accent", "#000"), "#ff0000")
        compare(c.color("text", "#000"), "#eeeeee")
        compare(c.color("colorLamp", "#000"), "#123456")
        compare(c.color("colorRain", "#abcdef"), "#abcdef")
        compare(c.font("clock", "Pixel"), "Inter")
        compare(c.font("text", "Pixel"), "Pixel")
        verify(c.dark)
    }

    function test_no_background_setting_leaves_the_theme_background() {
        tc.config = {}
        const b = createTemporaryObject(backgroundComponent, tc)
        verify(!b.active)
        verify(!b.visible)
    }

    function test_an_image_background_shows_with_fit_and_dim() {
        tc.config = { backgroundType: "image", backgroundPath: generated + "/bg.png", backgroundFit: "contain", backgroundDim: "30" }
        const b = createTemporaryObject(backgroundComponent, tc)
        verify(b.active)
        verify(!b.cover)
        fuzzyCompare(b.dim, 0.3, 0.001)
        wait(200)
        verify(!b.failed, "the fixture image loads")
    }

    function test_a_missing_file_gives_the_theme_its_own_background_back() {
        tc.config = { backgroundType: "image", backgroundPath: "/nonexistent/wall.png" }
        const b = createTemporaryObject(backgroundComponent, tc)
        tryVerify(() => b.failed, 2000)
        verify(!b.active, "never an empty screen")
    }

    function test_a_colour_background() {
        tc.config = { backgroundType: "color", backgroundColor: "#224466" }
        const b = createTemporaryObject(backgroundComponent, tc)
        verify(b.active)
        compare(b.children[0].color, "#224466")
    }
}
