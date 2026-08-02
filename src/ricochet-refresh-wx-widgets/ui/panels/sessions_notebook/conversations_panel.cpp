#include "conversations_panel.hpp"

#include "enums.hpp"
#include "ffi.hpp"
#include "strings.hpp"
#include "ui/events.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/chat_panel.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/contact_list_panel.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/message_entry_panel.hpp"
#include "ui/panels/sessions_notebook/conversations_panel/user_status_panel.hpp"

ConversationsPanel::ConversationsPanel(wxWindow* parent, tego_session_handle session_handle) :
    wxSplitterWindow(parent, wxID_ANY, wxDefaultPosition, wxDefaultSize, wxSP_LIVE_UPDATE),
    session_handle(session_handle) {
    auto left_panel = new wxPanel(this);
    this->right_panel = new wxPanel(this);

    // Contacts List + User Status

    auto left_v_sizer = new wxBoxSizer(wxVERTICAL);

    this->contact_list_panel = new ContactListPanel(left_panel, session_handle);
    this->contact_list_panel->Bind(wxEVT_CONTACT_SELECTED, [this](const ContactSelectedEvent& evt) {
        this->select_contact(evt.get_contact_handle());
    });
    this->contact_list_panel->Bind(wxEVT_CONTACT_REMOVED, [this](ContactRemovedEvent& evt) {
        this->remove_contact(evt.get_contact_handle());
        evt.Skip();
    });
    auto user_status_panel = new UserStatusPanel(left_panel);

    left_v_sizer->Add(this->contact_list_panel, 1, wxEXPAND);
    left_v_sizer->Add(user_status_panel, 0, wxEXPAND);

    left_panel->SetSizer(left_v_sizer);
    left_panel->SetMinSize(wxSize(288, -1));

    // Conversation + Chat Entry

    this->right_v_sizer = new wxBoxSizer(wxVERTICAL);

    this->right_panel->SetSizer(this->right_v_sizer);
    this->right_panel->SetMinSize(wxSize(288, -1));

    // Layout

    this->SetMinimumPaneSize(32); // prevent dbl-click collapse
    this->SplitVertically(left_panel, this->right_panel, 288);
    this->SetSashGravity(0.0);
}

ConversationsPanel::~ConversationsPanel() {
    tego_context_end_session(
        &wxGetApp().get_context_mut(),
        this->session_handle,
        tego::panic_on_error()
    );
}

void ConversationsPanel::add_user(
    const tego_user_handle user_handle,
    const wxString& display_name,
    const wxBitmap& avatar,
    const ContactGroup contact_group
) {
    // add user to ContactList
    this->contact_list_panel->add_contact(user_handle, display_name, avatar, contact_group);

    // add ContactWidgets for chatting

    auto chat_panel = new ChatPanel(this->right_panel);
    // todo: load chat back-log from profile

    auto message_entry_panel = new MessageEntryPanel(this->right_panel);
    message_entry_panel->Bind(wxEVT_SEND_MESSAGE, [=, this](const SendMessageEvent& evt) {
        const auto& timestamp = evt.get_timestamp();
        const auto& text = evt.get_text();
        chat_panel->add_chat_message(timestamp, wxString("Me"), text);
        // todo: remove, this is just test plumbing
        this->receive_message(
            user_handle,
            timestamp + wxTimeSpan(0, 0, 1),
            "auto-reply: I've received your message"
        );
    });

    auto v_sizer = new wxBoxSizer(wxVERTICAL);
    v_sizer->Add(chat_panel, 1, wxEXPAND);
    v_sizer->Add(message_entry_panel, 0, wxEXPAND);

    this->right_v_sizer->Add(v_sizer, 1, wxEXPAND);
    this->right_v_sizer->Show(v_sizer, false);

    this->contact_widgets.insert({user_handle, {v_sizer, chat_panel, message_entry_panel}});
}

void ConversationsPanel::receive_message(
    const tego_user_handle recipient,
    const wxDateTime& timestamp,
    const wxString& message
) {
    if (auto it = this->contact_widgets.find(recipient); it != this->contact_widgets.end()) {
        auto& contact_widgets = it->second;

        contact_widgets.chat_panel->add_chat_message(timestamp, "reply-bot", message);
    }
}

void ConversationsPanel::select_contact(const std::optional<tego_user_handle> contact_handle) {
    // hide everything
    this->right_v_sizer->ShowItems(false);
    if (contact_handle) {
        if (auto it = this->contact_widgets.find(*contact_handle);
            it != this->contact_widgets.end()) {
            // show contact's widgets
            const auto& contact_widgets = it->second;
            this->right_v_sizer->Show(contact_widgets.v_sizer, true);
            this->right_v_sizer->Layout();
        }
    }
}

void ConversationsPanel::remove_contact(const tego_user_handle contact_handle) {
    if (auto it = this->contact_widgets.find(contact_handle); it != this->contact_widgets.end()) {
        auto& v_sizer = it->second.v_sizer;

        // remove and delete children
        v_sizer->Clear(true);
        this->right_v_sizer->Detach(v_sizer);
        delete v_sizer;

        // trigger re-layout
        this->right_v_sizer->Layout();

        // remove our record
        this->contact_widgets.erase(it);
    }
}
