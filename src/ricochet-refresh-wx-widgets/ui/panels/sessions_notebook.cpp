#include "sessions_notebook.hpp"

#include "ui/main_frame.hpp"
#include "ui/panels/sessions_notebook/session_panel.hpp"

SessionsNotebook::SessionsNotebook(wxWindow* parent) : wxPanel(parent) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    this->session_notebook =
        new wxNotebook(this, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxNB_TOP);

    v_sizer->Add(this->session_notebook, 1, wxEXPAND);

    this->SetSizerAndFit(v_sizer);
}

void SessionsNotebook::open_session(const wxString& profile_path) {
    const auto profile_file = wxFileName(profile_path);
    const auto absolute_profile_path = profile_file.GetAbsolutePath();

    if (auto panel = this->get_session_data_by_profile_path(absolute_profile_path); panel) {
        auto session_panel = panel.value();
        const auto tab_index = this->session_notebook->FindPage(session_panel);
        if (tab_index != wxNOT_FOUND) {
            this->session_notebook->SetSelection(tab_index);
        }
    } else {
        auto session_panel = new SessionPanel(this->session_notebook, absolute_profile_path);

        const auto profile_filename = profile_file.GetFullName();

        this->session_notebook->AddPage(session_panel, profile_filename, true);
    }
}

void SessionsNotebook::close_focused_session() {
    LOG_INFO("close focused session");
    auto window = this->session_notebook->GetCurrentPage();
    this->close_session(dynamic_cast<SessionPanel*>(window));
}

void SessionsNotebook::close_session(SessionPanel* session_panel) {
    LOG_INFO(fmt::format("close session: {}", static_cast<void*>(session_panel)));
    for (size_t i = 0; i < this->session_notebook->GetPageCount(); ++i) {
        auto window = this->session_notebook->GetPage(i);
        if (window == session_panel) {
            this->session_notebook->RemovePage(i);
            session_panel->Destroy();

            if (this->session_notebook->GetPageCount() == 0) {
                wxGetApp().get_main_frame().show_bootstrap_panel();
            }
            return;
        }
    }
}

std::optional<SessionPanel*> SessionsNotebook::get_session_data_by_profile_path(const wxString& path
) {
    for (size_t i = 0; i < this->session_notebook->GetPageCount(); ++i) {
        auto window = this->session_notebook->GetPage(i);
        auto panel = dynamic_cast<SessionPanel*>(window);
        if (panel->get_profile_path() == path) {
            return panel;
        }
    }
    return std::nullopt;
}

std::optional<SessionPanel*>
SessionsNotebook::get_session_data_by_session_handle(tego_session_handle handle) {
    for (size_t i = 0; i < this->session_notebook->GetPageCount(); ++i) {
        auto window = this->session_notebook->GetPage(i);
        auto panel = dynamic_cast<SessionPanel*>(window);
        if (panel->get_session_handle() == handle) {
            return panel;
        }
    }
    return std::nullopt;
}
