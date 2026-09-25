.pragma library

// Must match darwan_core::ini, which writes the overlay this reads.
function parseGeneral(text) {
    var out = {};
    var section = "General";
    var lines = (text || "").split(/\r?\n/);
    for (var i = 0; i < lines.length; i++) {
        var line = lines[i].trim();
        if (line === "" || line[0] === "#" || line[0] === ";")
            continue;
        if (line[0] === "[" && line[line.length - 1] === "]") {
            section = line.slice(1, -1).trim();
            continue;
        }
        if (section !== "General")
            continue;
        var eq = line.indexOf("=");
        if (eq < 0)
            continue;
        out[line.slice(0, eq).trim()] = unquote(line.slice(eq + 1).trim());
    }
    return out;
}

function unquote(v) {
    if (v.length >= 2 && v[0] === "\"" && v[v.length - 1] === "\"") {
        var inner = v.slice(1, -1);
        var s = "";
        for (var i = 0; i < inner.length; i++) {
            if (inner[i] === "\\" && i + 1 < inner.length)
                i++;
            s += inner[i];
        }
        v = s;
    }
    return v.indexOf("@@") === 0 ? v.slice(1) : v;
}

// SDDM only lets non-empty theme.conf.user values override theme.conf.
function merge(base, overlay) {
    var out = {};
    for (var k in base)
        out[k] = base[k];
    for (var j in overlay)
        if (overlay[j] !== "")
            out[j] = overlay[j];
    return out;
}
