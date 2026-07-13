#pragma once

class ErrorPopup {
public:
    static void
    show(wxWindow* parent, const wxString& friendly_message, const wxString& detailed_message);
};