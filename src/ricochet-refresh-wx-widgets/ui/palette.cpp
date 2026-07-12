#include "palette.hpp"

// light mode palette
const Palette LIGHT_MODE_PALETTE = {
    .red = wxColour(0xff, 0x85, 0x85),
    .orange = wxColour(0xff, 0xac, 0x75),
    .yellow = wxColour(0xff, 0xf5, 0x9e),
    .green = wxColour(0xa2, 0xe7, 0xb5),
    .blue = wxColour(0xa8, 0xa8, 0xff),
    .purple = wxColour(0xd0, 0xb5, 0xe3),
    .pink = wxColour(0xf9, 0x86, 0xae),
    .teal = wxColour(0x8b, 0xda, 0xd2),
};

// dark mode palette
const Palette DARK_MODE_PALETTE = {
    .red = wxColour(0x9f, 0x22, 0x14),
    .orange = wxColour(0xb3, 0x5e, 0x14),
    .yellow = wxColour(0x90, 0x90, 0x00),
    .green = wxColour(0x1e, 0x85, 0x49),
    .blue = wxColour(0x2d, 0x2d, 0xcd),
    .purple = wxColour(0x8e, 0x44, 0xad),
    .pink = wxColour(0xe0, 0x29, 0x60),
    .teal = wxColour(0x12, 0x92, 0x6c),
};

const Palette& Palette::instance() {
    const auto text_colour = wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOWTEXT);
    LOG_INFO(fmt::format("Text Colour: {}", text_colour));

    // High text luminance implies dark mode
    auto luminance = text_colour.GetLuminance();
    LOG_INFO(fmt::format("Text Luminance: {}", luminance));
    if (luminance > 0.5) {
        return DARK_MODE_PALETTE;
    } else {
        return LIGHT_MODE_PALETTE;
    }
}
