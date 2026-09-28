#pragma once

#include <MpvQt/MpvAbstractItem>
#include <QtQml/qqmlregistration.h>
#include <QUrl>

// A looping, silent background video played by libmpv, rendered on the GPU Qt renders with.
class MpvVideo : public MpvAbstractItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QUrl source READ source WRITE setSource NOTIFY sourceChanged)
    Q_PROPERTY(bool playing READ playing WRITE setPlaying NOTIFY playingChanged)
    Q_PROPERTY(bool loop READ loop WRITE setLoop NOTIFY loopChanged)
    Q_PROPERTY(FillMode fillMode READ fillMode WRITE setFillMode NOTIFY fillModeChanged)
    // "full"; "eco": the transcoded copy darwan cached for this file, if there is one; "still": the first frame only.
    Q_PROPERTY(QString tier READ tier WRITE setTier NOTIFY tierChanged)

public:
    enum FillMode { Stretch, PreserveAspectFit, PreserveAspectCrop };
    Q_ENUM(FillMode)

    explicit MpvVideo(QQuickItem *parent = nullptr);

    QUrl source() const { return m_source; }
    void setSource(const QUrl &source);
    bool playing() const { return m_playing; }
    void setPlaying(bool playing);
    bool loop() const { return m_loop; }
    void setLoop(bool loop);
    FillMode fillMode() const { return m_fillMode; }
    void setFillMode(FillMode mode);
    QString tier() const { return m_tier; }
    void setTier(const QString &tier);

    // Where darwan caches the eco copy of `path`; the same key as darwan_core::media::eco_path.
    static QString ecoPath(const QString &path);

Q_SIGNALS:
    void sourceChanged();
    void playingChanged();
    void loopChanged();
    void fillModeChanged();
    void tierChanged();
    void failed(const QString &reason);

private:
    void load();
    void applyFill();
    void applyPause();

    QUrl m_source;
    bool m_playing = true;
    bool m_loop = true;
    FillMode m_fillMode = PreserveAspectFit;
    QString m_tier = QStringLiteral("full");
    bool m_ready = false;
};
