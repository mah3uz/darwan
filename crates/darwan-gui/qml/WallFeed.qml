import QtQuick
import org.darwan

// A list of wallpapers kept in a ListModel that is changed in place: entries that stay are left alone, new ones are
// appended, so a view keeps its scroll position and its cards while results arrive. With `channel` set it follows
// that online list; without, `set()` feeds it (the Library).
QtObject {
    id: feed

    property var backend: null
    property string channel: ""
    property var info: ({ items: [], loading: false, more: false, error: "", refused: "" })
    property var items: []
    readonly property ListModel model: ListModel {}
    readonly property int revision: backend && channel !== "" ? backend.wallFeedRevision : -1
    property int stamp: -2

    // Read again only when this channel changed; another channel's news costs one number.
    function refresh() {
        if (!backend || channel === "")
            return
        const now = backend.wallStamp(channel)
        if (now === stamp)
            return
        stamp = now
        const d = JSON.parse(backend.wallOnline(channel))
        info = d
        set(d.items)
    }

    function set(list) {
        items = list
        const m = model
        const shared = Math.min(m.count, list.length)
        let same = 0
        while (same < shared && m.get(same).key === (list[same].key || list[same].path))
            same++
        if (same < m.count)
            m.remove(same, m.count - same)
        // Only a card whose picture or marks changed is told; comparing these few fields is far cheaper than the
        // whole entry.
        for (let i = 0; i < same; i++) {
            const it = list[i]
            const sig = signature(it)
            if (m.get(i).sig !== sig) {
                m.setProperty(i, "sig", sig)
                m.setProperty(i, "json", JSON.stringify(it))
            }
        }
        for (let i = same; i < list.length; i++)
            m.append({ key: list[i].key || list[i].path, sig: signature(list[i]), json: JSON.stringify(list[i]) })
    }

    function signature(it) {
        return (it.thumb || "") + "|" + (it.size || "") + "|" + (it.downloaded ? 1 : 0) + (it.inUse ? 1 : 0) + (it.isNew ? 1 : 0)
    }

    onRevisionChanged: refresh()
    onChannelChanged: refresh()
    Component.onCompleted: refresh()
}
