#pragma once

#undef NDEBUG

// std
#include <cassert>
#include <filesystem>
#include <iostream>
#include <optional>
#include <span>
#include <string_view>
#include <tuple>
#include <unordered_map>
#include <utility>

// wxWidgets
#include <wx/app.h>
#include <wx/artprov.h>
#include <wx/clipbrd.h>
#include <wx/cmdline.h>
#include <wx/dcbuffer.h>
#include <wx/event.h>
#include <wx/intl.h>
#include <wx/renderer.h>
#include <wx/splitter.h>
#include <wx/stdpaths.h>
#include <wx/textwrapper.h>
#include <wx/uilocale.h>
#include <wx/valnum.h>
#include <wx/wx.h>

// {fmt}
#ifdef RICOCHET_REFRESH_LOGGING
    #include <fmt/format.h>

// enable wxString usage with fmt
inline auto format_as(const wxString& str) {
    return str.utf8_string();
}
#endif

// tego-rs
#include <tego/tego.hpp>

// ricochet-refresh
#include <main.hpp>
