#include "mpvvideo.h"

#include <MpvQt/MpvController>

#include <QDir>
#include <QTimer>
#include <QFileInfo>

using namespace Qt::StringLiterals;

MpvVideo::MpvVideo(QQuickItem *parent)
    : MpvAbstractItem(parent)
{
    const auto hwdec = qEnvironmentVariable("DARWAN_HWDEC");
    setProperty(u"hwdec"_s, hwdec.isEmpty() ? u"auto-safe"_s : hwdec);
    // mpv otherwise holds each render until the frame's display time, pinning the whole scene to the video's rate.
    setProperty(u"video-timing-offset"_s, 0);
    setProperty(u"aid"_s, u"no"_s);
    setProperty(u"sid"_s, u"no"_s);
    setProperty(u"loop-file"_s, u"inf"_s);
    applyPause();
    applyFill();

    // Moving to another window (the saver becoming the lock) takes the render context away for a moment, which mpv
    // can report as a playback error: retry once before giving up on the file.
    connect(mpvController(), &MpvController::endFile, this, [this](const QString &reason) {
        if (reason != u"error"_s)
            return;
        if (!m_retry.isValid() || m_retry.hasExpired(5000)) {
            m_retry.start();
            QTimer::singleShot(300, this, [this] { load(); });
            return;
        }
        Q_EMIT failed(u"cannot play "_s + m_source.toString());
    });
    connect(this, &MpvAbstractItem::ready, this, [this] {
        m_ready = true;
        load();
    });
}

void MpvVideo::setSource(const QUrl &source)
{
    if (source == m_source)
        return;
    m_source = source;
    Q_EMIT sourceChanged();
    load();
}

void MpvVideo::setPlaying(bool playing)
{
    if (playing == m_playing)
        return;
    m_playing = playing;
    applyPause();
    Q_EMIT playingChanged();
}

void MpvVideo::setLoop(bool loop)
{
    if (loop == m_loop)
        return;
    m_loop = loop;
    setProperty(u"loop-file"_s, loop ? u"inf"_s : u"no"_s);
    Q_EMIT loopChanged();
}

void MpvVideo::setFillMode(FillMode mode)
{
    if (mode == m_fillMode)
        return;
    m_fillMode = mode;
    applyFill();
    Q_EMIT fillModeChanged();
}

void MpvVideo::setTier(const QString &tier)
{
    if (tier == m_tier)
        return;
    m_tier = tier;
    Q_EMIT tierChanged();
    applyPause();
    load();
}

void MpvVideo::applyPause()
{
    setProperty(u"pause"_s, !m_playing || m_tier == u"still"_s);
}

QString MpvVideo::ecoPath(const QString &path)
{
    const QFileInfo info(path);
    const QByteArray key = info.canonicalFilePath().toUtf8() + '\0' + QByteArray::number(info.size()) + '\0'
        + QByteArray::number(info.lastModified().toSecsSinceEpoch());
    quint64 hash = 0xcbf29ce484222325ULL;
    for (const char c : key) {
        hash ^= static_cast<unsigned char>(c);
        hash *= 0x100000001b3ULL;
    }
    auto dir = qEnvironmentVariable("XDG_CACHE_HOME");
    if (dir.isEmpty())
        dir = QDir::homePath() + u"/.cache"_s;
    return dir + u"/darwan/media/"_s + QString::number(hash, 16).rightJustified(16, u'0') + u".mp4"_s;
}

void MpvVideo::applyFill()
{
    setProperty(u"keepaspect"_s, m_fillMode != Stretch);
    setProperty(u"panscan"_s, m_fillMode == PreserveAspectCrop ? 1.0 : 0.0);
}

void MpvVideo::load()
{
    if (!m_ready)
        return;
    if (m_source.isEmpty()) {
        command({u"stop"_s});
        return;
    }
    QString path = m_source.isLocalFile() ? m_source.toLocalFile() : m_source.toString();
    // An animated image's video copy decodes on the GPU instead of every frame on the CPU, so it always wins.
    static const QStringList animated{u"gif"_s, u"webp"_s, u"apng"_s, u"png"_s};
    const bool isAnimated = animated.contains(QFileInfo(path).suffix().toLower());
    if ((m_tier == u"eco"_s || isAnimated) && m_source.isLocalFile()) {
        const QString eco = ecoPath(path);
        if (QFileInfo::exists(eco))
            path = eco;
    }
    command({u"loadfile"_s, path});
}
