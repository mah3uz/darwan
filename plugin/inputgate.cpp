#include "inputgate.h"

#include <QKeyEvent>
#include <QMouseEvent>
#include <QQuickWindow>

InputGate::InputGate(QQuickItem *parent)
    : QQuickItem(parent)
{
}

InputGate::~InputGate()
{
    watch(nullptr);
}

void InputGate::setActive(bool active)
{
    if (active == m_active)
        return;
    m_active = active;
    m_hasOrigin = false;
    Q_EMIT activeChanged();
}

void InputGate::setPassText(bool pass)
{
    if (pass == m_passText)
        return;
    m_passText = pass;
    Q_EMIT passTextChanged();
}

void InputGate::itemChange(ItemChange change, const ItemChangeData &data)
{
    if (change == ItemSceneChange)
        watch(data.window);
    QQuickItem::itemChange(change, data);
}

void InputGate::watch(QQuickWindow *window)
{
    if (m_window)
        m_window->removeEventFilter(this);
    m_window = window;
    if (m_window)
        m_window->installEventFilter(this);
}

static bool isText(const QKeyEvent *key)
{
    const auto mods = key->modifiers() & ~(Qt::ShiftModifier | Qt::KeypadModifier);
    return mods == Qt::NoModifier && !key->text().isEmpty() && key->text().at(0).isPrint();
}

// Input is always reported, so a revealed lock can tell when it has sat idle; it is only swallowed while active.
bool InputGate::eventFilter(QObject *watched, QEvent *event)
{
    if (watched != m_window)
        return false;
    switch (event->type()) {
    case QEvent::KeyPress: {
        const bool pass = !m_active || (m_passText && isText(static_cast<QKeyEvent *>(event)));
        Q_EMIT activity();
        return !pass;
    }
    case QEvent::MouseButtonPress:
    case QEvent::MouseButtonDblClick:
    case QEvent::Wheel:
    case QEvent::TouchBegin:
        Q_EMIT activity();
        return m_active;
    case QEvent::KeyRelease:
    case QEvent::MouseButtonRelease:
    case QEvent::TouchUpdate:
    case QEvent::TouchEnd:
        return m_active;
    case QEvent::MouseMove: {
        const QPointF p = static_cast<QMouseEvent *>(event)->scenePosition();
        if (!m_hasOrigin) {
            m_origin = p;
            m_hasOrigin = true;
        } else if (QLineF(m_origin, p).length() > m_threshold) {
            m_origin = p;
            Q_EMIT activity();
        }
        return m_active;
    }
    default:
        return false;
    }
}
