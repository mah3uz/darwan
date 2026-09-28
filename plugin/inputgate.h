#pragma once

#include <QPointF>
#include <QPointer>
#include <QQuickItem>
#include <QtQml/qqmlregistration.h>

// Reports user input on its window: keys, clicks, wheel and touch at once, pointer motion past a threshold (a surface
// that maps under the pointer gets motion that isn't the user). While active it swallows that input, so the input that
// ends ambient mode never reaches the theme.
class InputGate : public QQuickItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(bool active READ active WRITE setActive NOTIFY activeChanged)
    Q_PROPERTY(bool passText READ passText WRITE setPassText NOTIFY passTextChanged)
    Q_PROPERTY(qreal threshold MEMBER m_threshold NOTIFY thresholdChanged)

public:
    explicit InputGate(QQuickItem *parent = nullptr);
    ~InputGate() override;

    bool active() const { return m_active; }
    void setActive(bool active);
    // Printable keys reach the focused item (the hidden password field) instead of being swallowed.
    bool passText() const { return m_passText; }
    void setPassText(bool pass);

Q_SIGNALS:
    void activeChanged();
    void passTextChanged();
    void thresholdChanged();
    void activity();

protected:
    bool eventFilter(QObject *watched, QEvent *event) override;
    void itemChange(ItemChange change, const ItemChangeData &data) override;

private:
    void watch(QQuickWindow *window);

    QPointer<QQuickWindow> m_window;
    bool m_active = false;
    bool m_passText = false;
    qreal m_threshold = 8;
    bool m_hasOrigin = false;
    QPointF m_origin;
};
