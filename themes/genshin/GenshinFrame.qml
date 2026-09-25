import QtQuick

Item {
    property color frameColor: "#c8ab6e"
    property real cornerSize: 10

    // Top-left
    Canvas {
        width: cornerSize; height: cornerSize
        anchors.left: parent.left; anchors.top: parent.top
        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0,0,width,height)
            ctx.strokeStyle = frameColor; ctx.lineWidth = 1.5
            // top bar
            ctx.beginPath(); ctx.moveTo(0,0); ctx.lineTo(width*0.8,0); ctx.stroke()
            // left bar
            ctx.beginPath(); ctx.moveTo(0,0); ctx.lineTo(0,height*0.8); ctx.stroke()
            // diamond
            ctx.fillStyle = frameColor
            ctx.beginPath()
            var s2 = height * 0.28
            ctx.moveTo(width*0.5, height*0.5-s2)
            ctx.lineTo(width*0.5+s2, height*0.5)
            ctx.lineTo(width*0.5, height*0.5+s2)
            ctx.lineTo(width*0.5-s2, height*0.5)
            ctx.closePath(); ctx.fill()
        }
    }
    // Top-right
    Canvas {
        width: cornerSize; height: cornerSize
        anchors.right: parent.right; anchors.top: parent.top
        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0,0,width,height)
            ctx.strokeStyle = frameColor; ctx.lineWidth = 1.5
            ctx.beginPath(); ctx.moveTo(width*0.2,0); ctx.lineTo(width,0); ctx.stroke()
            ctx.beginPath(); ctx.moveTo(width,0); ctx.lineTo(width,height*0.8); ctx.stroke()
            ctx.fillStyle = frameColor
            ctx.beginPath()
            var s2 = height*0.28
            ctx.moveTo(width*0.5,height*0.5-s2); ctx.lineTo(width*0.5+s2,height*0.5)
            ctx.lineTo(width*0.5,height*0.5+s2); ctx.lineTo(width*0.5-s2,height*0.5)
            ctx.closePath(); ctx.fill()
        }
    }
    // Bottom-left
    Canvas {
        width: cornerSize; height: cornerSize
        anchors.left: parent.left; anchors.bottom: parent.bottom
        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0,0,width,height)
            ctx.strokeStyle = frameColor; ctx.lineWidth = 1.5
            ctx.beginPath(); ctx.moveTo(0,height*0.2); ctx.lineTo(0,height); ctx.stroke()
            ctx.beginPath(); ctx.moveTo(0,height); ctx.lineTo(width*0.8,height); ctx.stroke()
            ctx.fillStyle = frameColor
            ctx.beginPath()
            var s2 = height*0.28
            ctx.moveTo(width*0.5,height*0.5-s2); ctx.lineTo(width*0.5+s2,height*0.5)
            ctx.lineTo(width*0.5,height*0.5+s2); ctx.lineTo(width*0.5-s2,height*0.5)
            ctx.closePath(); ctx.fill()
        }
    }
    // Bottom-right
    Canvas {
        width: cornerSize; height: cornerSize
        anchors.right: parent.right; anchors.bottom: parent.bottom
        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0,0,width,height)
            ctx.strokeStyle = frameColor; ctx.lineWidth = 1.5
            ctx.beginPath(); ctx.moveTo(width*0.2,height); ctx.lineTo(width,height); ctx.stroke()
            ctx.beginPath(); ctx.moveTo(width,height*0.2); ctx.lineTo(width,height); ctx.stroke()
            ctx.fillStyle = frameColor
            ctx.beginPath()
            var s2 = height*0.28
            ctx.moveTo(width*0.5,height*0.5-s2); ctx.lineTo(width*0.5+s2,height*0.5)
            ctx.lineTo(width*0.5,height*0.5+s2); ctx.lineTo(width*0.5-s2,height*0.5)
            ctx.closePath(); ctx.fill()
        }
    }
}
