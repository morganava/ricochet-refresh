#include "session_panel.hpp"

#include "strings.hpp"
#include "ui/fonts.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/sessions_notebook/unlock_profile_panel.hpp"

SessionPanel::SessionPanel(wxWindow* parent, const wxString& profile_path) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    auto unlock_profile_panel = new UnlockProfilePanel(this, profile_path);

    // Layout
    v_sizer->Add(unlock_profile_panel, 1, wxEXPAND);

    this->SetSizerAndFit(v_sizer);
}
