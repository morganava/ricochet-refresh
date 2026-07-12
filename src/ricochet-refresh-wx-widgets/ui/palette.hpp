#pragma once

class Palette {
public:
    static const Palette& instance();

    const wxColour red;
    const wxColour orange;
    const wxColour yellow;
    const wxColour green;
    const wxColour blue;
    const wxColour purple;
    const wxColour pink;
    const wxColour teal;
};
