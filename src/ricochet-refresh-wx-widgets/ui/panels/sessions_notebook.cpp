#include "sessions_notebook.hpp"

#include "ui/panels/sessions_notebook/session_panel.hpp"

SessionsNotebook::SessionsNotebook(wxWindow* parent) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    this->session_notebook =
        new wxNotebook(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxNB_TOP);
    // this->session_notebook->Bind(wxEVT_NOTEBOOK_PAGE_CHANGED, [this](wxBookCtrlEvent& evt) {
    //     if (auto tab_index = evt.GetSelection(); tab_index != wxNOT_FOUND) {
    //         this->session_notebook->GetPage(tab_index)->Layout();
    //     }
    // });

    v_sizer->Add(this->session_notebook, 1, wxEXPAND);

    this->SetSizerAndFit(v_sizer);
}

void SessionsNotebook::open_session(const wxString& profile_path) {
    auto session_panel = new SessionPanel(this->session_notebook, profile_path);

    const auto profile_file = wxFileName(profile_path);
    const auto profile_filename = profile_file.GetFullName();

    this->session_notebook->AddPage(session_panel, profile_filename, true);
}
