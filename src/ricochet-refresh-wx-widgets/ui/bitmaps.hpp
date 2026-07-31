#pragma once

#include "ui/metrics.hpp"

class Bitmaps {
public:
    static const wxBitmap& default_avatar() {
        const static wxImage image(
            Metrics::AVATAR_SIZE,
            Metrics::AVATAR_SIZE,
            const_cast<unsigned char*>(Bitmaps::default_avatar_bmp),
            true
        );
        const static wxBitmap bmp(image);
        return bmp;
    }

private:
    const static uint8_t default_avatar_bmp[Metrics::AVATAR_SIZE * Metrics::AVATAR_SIZE * 3];
};
