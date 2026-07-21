#include "session_panel.hpp"

#include "strings.hpp"
#include "ui/events.hpp"
#include "ui/fonts.hpp"
#include "ui/main_frame.hpp"
#include "ui/metrics.hpp"
#include "ui/panels/sessions_notebook.hpp"
#include "ui/panels/sessions_notebook/conversations_panel.hpp"
#include "ui/panels/sessions_notebook/unlock_profile_panel.hpp"

SessionPanel::SessionPanel(wxWindow* parent, const wxString& profile_path) :
    wxPanel(parent),
    profile_path(profile_path) {
    auto v_sizer = new wxBoxSizer(wxVERTICAL);

    this->unlock_profile_panel = new UnlockProfilePanel(this, this->profile_path);
    this->unlock_profile_panel->Bind(wxEVT_PROFILE_UNLOCKED, [this](ProfileUnlockedEvent& evt) {
        this->on_profile_unlocked(std::move(evt.take_profile()));
    });

    // Layout
    v_sizer->Add(this->unlock_profile_panel, 1, wxEXPAND);

    this->SetSizerAndFit(v_sizer);
}

void SessionPanel::on_profile_unlocked(std::unique_ptr<tego_profile>&& profile) {
    LOG_INFO("Profile unlocked");
    auto sizer = this->GetSizer();
    sizer->Detach(this->unlock_profile_panel);
    auto unlock_profile_panel = this->unlock_profile_panel;
    wxGetApp().CallAfter([=]() { unlock_profile_panel->Destroy(); });
    this->unlock_profile_panel = nullptr;

    tego_session_handle session_handle;
    tego_context_begin_session(
        &wxGetApp().get_context_mut(),
        &session_handle,
        profile.release(),
        tego::panic_on_error()
    );
    this->session_handle = session_handle;

    this->conversations_panel = new ConversationsPanel(this, session_handle);
    sizer->Add(this->conversations_panel, 1, wxEXPAND);
    this->Layout();
}

const wxString& SessionPanel::get_profile_path() const {
    return this->profile_path;
}

std::optional<tego_session_handle> SessionPanel::get_session_handle() const {
    return this->session_handle;
}
