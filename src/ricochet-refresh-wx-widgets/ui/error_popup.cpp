#include "error_popup.hpp"

#include "strings.hpp"

void ErrorPopup::show(
    wxWindow* parent,
    const wxString& friendly_message,
    const wxString& detailed_message
) {
    wxRichMessageDialog error_dialog(
        parent,
        friendly_message,
        Strings::Common::error_dialog_title(),
        wxOK | wxCENTRE | wxICON_ERROR
    );
    error_dialog.ShowDetailedText(detailed_message);

    error_dialog.ShowModal();
}