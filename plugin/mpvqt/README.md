# MpvQt

KDE's [MpvQt](https://invent.kde.org/libraries/mpvqt) 1.2.1 (`src/` of `mpvqt-1.2.1.tar.xz`, sha256
`6ff5f56a0edd8248073446e5cc038f47d4b17c46b6af43cf438123d0ac911189`), compiled into darwan's video plugin so that it
links only to libmpv and Qt. Arch's `mpvqt` changed its soname in a patch release (1.2.1), which stopped an installed
darwan from loading its plugin until it was rebuilt.

Unchanged except `mpvqt_export.h`, which ECM generates upstream. Licensed LGPL-2.1-only OR LGPL-3.0-only OR
LicenseRef-KDE-Accepted-LGPL (`LICENSES/`).
